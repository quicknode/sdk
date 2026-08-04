//! x402/Solana SPL `TransferChecked` signer.
//!
//! Builds a partially-signed Solana transaction: the gateway's `feePayer`
//! (from the challenge `extra.feePayer`) is the transaction fee payer and the
//! first required signature slot, so the payer needs no SOL — only the token.
//! The payer signs its own slot; the gateway co-signs the fee-payer slot
//! server-side before submitting.
//!
//! The instructions are hand-rolled rather than pulling `spl-token`, which
//! drags `solana-program` → curve25519/MSRV conflicts under cross+zig at
//! glibc-2.17/musl.
//!
//! The message is a **v0** message carrying four instructions, matching the
//! canonical x402 Solana scheme:
//!
//! 1. `SetComputeUnitLimit`
//! 2. `SetComputeUnitPrice`
//! 3. SPL `TransferChecked`
//! 4. `Memo` — the challenge's `extra.memo`, else a random nonce. This is the
//!    payment's replay-protection nonce, so it is not optional.
//!
//! Async: the mint (for decimals and its owning token program) and a recent
//! blockhash are read from a Solana RPC (source precedence resolved by the
//! driver: explicit override → tooling endpoint → public default). The gateway
//! 402s keyless sub-reads, so these reads go to a plain Solana RPC, not the
//! gateway.

use ed25519_dalek::{Signer as _, SigningKey};
use sha2::{Digest, Sha256};

use super::Signer;
use crate::errors::SdkError;

// Program ids are embedded to avoid the spl-token dependency.
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const COMPUTE_BUDGET_PROGRAM_ID: &str = "ComputeBudget111111111111111111111111111111";
const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

// SPL TransferChecked instruction discriminant.
const TRANSFER_CHECKED: u8 = 12;
// ComputeBudget instruction discriminants.
const SET_COMPUTE_UNIT_LIMIT: u8 = 2;
const SET_COMPUTE_UNIT_PRICE: u8 = 3;

// Compute-budget defaults, matching the canonical scheme.
const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 20_000;
const DEFAULT_COMPUTE_UNIT_PRICE_MICROLAMPORTS: u64 = 1;

/// Upper bound on the memo payload, matching the canonical scheme.
pub(crate) const MAX_MEMO_BYTES: usize = 256;

// High bit marks a versioned message; legacy messages start with a signature count.
const V0_MESSAGE_PREFIX: u8 = 0x80;

/// Inputs for one x402/Solana payment, derived from the decoded challenge.
#[derive(Debug, Clone)]
pub struct SvmTransferRequest {
    /// Token mint (challenge `asset`), base58.
    pub mint: String,
    /// Payment recipient owner (challenge `payTo`), base58.
    pub pay_to: String,
    /// Gateway fee payer (challenge `extra.feePayer`), base58.
    pub fee_payer: String,
    /// Amount in token base units.
    pub amount: u64,
    /// Token decimals (TransferChecked requires them).
    pub decimals: u8,
    /// Recent blockhash (base58), read from the Solana RPC by the driver.
    pub recent_blockhash: String,
    /// The mint's owning token program (base58), read from the mint account by
    /// the driver — SPL Token or Token-2022.
    pub token_program: String,
    /// Memo payload: the challenge's `extra.memo` when present, else a random
    /// nonce minted by the driver. Carries the payment's replay protection.
    pub memo: String,
}

impl Signer {
    // base58 ed25519 pubkey (Solana address) of the payer.
    pub(super) fn svm_address(&self) -> Result<String, SdkError> {
        let key = svm_signing_key(self)?;
        Ok(bs58::encode(key.verifying_key().to_bytes()).into_string())
    }

    /// Build a partially-signed x402/Solana payment transaction: a v0 message
    /// carrying compute-budget, SPL `TransferChecked` and memo instructions.
    /// Returns the serialized signed transaction bytes (the driver base64s them
    /// into the payment envelope's `payload.transaction`).
    pub fn sign_svm_transfer(&self, req: &SvmTransferRequest) -> Result<Vec<u8>, SdkError> {
        let key = svm_signing_key(self)?;
        let payer = key.verifying_key().to_bytes();

        // TransferChecked is supported only by these token programs.
        if req.token_program != TOKEN_PROGRAM_ID && req.token_program != TOKEN_2022_PROGRAM_ID {
            return Err(SdkError::Config(format!(
                "mint {} is owned by {}, which is not a known SPL token program",
                req.mint, req.token_program
            )));
        }
        let token_program = decode_pubkey(&req.token_program)?;
        let mint = decode_pubkey(&req.mint)?;
        let pay_to_owner = decode_pubkey(&req.pay_to)?;
        let fee_payer = decode_pubkey(&req.fee_payer)?;
        let compute_budget = decode_pubkey(COMPUTE_BUDGET_PROGRAM_ID)?;
        let memo_program = decode_pubkey(MEMO_PROGRAM_ID)?;

        let memo_data = req.memo.as_bytes();
        if memo_data.len() > MAX_MEMO_BYTES {
            return Err(SdkError::Config(format!(
                "x402 Solana memo exceeds the maximum {MAX_MEMO_BYTES} bytes"
            )));
        }

        // Derive the source and destination ATAs.
        let source_ata = associated_token_address(&payer, &token_program, &mint)?;
        let dest_ata = associated_token_address(&pay_to_owner, &token_program, &mint)?;

        // Runtime-required order: writable signers, readonly signers,
        // writable nonsigners, readonly nonsigners.
        let accounts = vec![
            fee_payer,
            payer,
            source_ata,
            dest_ata,
            mint,
            token_program,
            compute_budget,
            memo_program,
        ];
        let header = MessageHeader {
            num_required_signatures: 2,
            num_readonly_signed: 0,
            // mint, token_program, compute_budget, memo_program
            num_readonly_unsigned: 4,
        };
        let index =
            |pk: &[u8; 32]| -> u8 { accounts.iter().position(|a| a == pk).map_or(0, |p| p as u8) };

        // Compute budget instructions must come first.
        let mut cu_limit_data = Vec::with_capacity(5);
        cu_limit_data.push(SET_COMPUTE_UNIT_LIMIT);
        cu_limit_data.extend_from_slice(&DEFAULT_COMPUTE_UNIT_LIMIT.to_le_bytes());

        let mut cu_price_data = Vec::with_capacity(9);
        cu_price_data.push(SET_COMPUTE_UNIT_PRICE);
        cu_price_data.extend_from_slice(&DEFAULT_COMPUTE_UNIT_PRICE_MICROLAMPORTS.to_le_bytes());

        // TransferChecked accounts and data layout.
        let mut transfer_data = Vec::with_capacity(10);
        transfer_data.push(TRANSFER_CHECKED);
        transfer_data.extend_from_slice(&req.amount.to_le_bytes());
        transfer_data.push(req.decimals);

        let instructions = vec![
            Instruction {
                program_index: index(&compute_budget),
                account_indexes: Vec::new(),
                data: cu_limit_data,
            },
            Instruction {
                program_index: index(&compute_budget),
                account_indexes: Vec::new(),
                data: cu_price_data,
            },
            Instruction {
                program_index: index(&token_program),
                account_indexes: vec![
                    index(&source_ata),
                    index(&mint),
                    index(&dest_ata),
                    index(&payer),
                ],
                data: transfer_data,
            },
            Instruction {
                program_index: index(&memo_program),
                account_indexes: Vec::new(),
                data: memo_data.to_vec(),
            },
        ];

        let message = build_message(&header, &accounts, &req.recent_blockhash, &instructions)?;

        // Leave the gateway's fee-payer signature slot empty.
        let payer_sig = key.sign(&message).to_bytes();
        let mut tx = Vec::new();
        write_compact_u16(&mut tx, 2);
        tx.extend_from_slice(&[0u8; 64]); // fee-payer slot (gateway fills)
        tx.extend_from_slice(&payer_sig); // payer slot
        tx.extend_from_slice(&message);
        Ok(tx)
    }
}

/// Sign a CAIP-122 SIWS message and return the Base58 Ed25519 signature.
pub(super) fn sign_siws(signer: &Signer, message: &str) -> Result<String, SdkError> {
    let key = svm_signing_key(signer)?;
    let signature = key.sign(message.as_bytes());
    Ok(bs58::encode(signature.to_bytes()).into_string())
}

/// A v0 message's account-permission counts.
struct MessageHeader {
    num_required_signatures: u8,
    num_readonly_signed: u8,
    num_readonly_unsigned: u8,
}

/// One compiled instruction: indexes into the message's account list.
struct Instruction {
    program_index: u8,
    account_indexes: Vec<u8>,
    data: Vec<u8>,
}

/// Generates a fresh Solana keypair. Returns the base58-encoded 64-byte
/// `[seed(32) || public(32)]` secret key (the format `svm_signing_key` reads).
/// Randomness comes from `rand::thread_rng` (the OS CSPRNG), matching the
/// nonce generator used elsewhere in this module.
pub(super) fn generate_svm_key() -> String {
    use rand::RngCore;
    let mut seed = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut seed);
    let key = SigningKey::from_bytes(&seed);
    let mut full = Vec::with_capacity(64);
    full.extend_from_slice(&seed);
    full.extend_from_slice(&key.verifying_key().to_bytes());
    bs58::encode(full).into_string()
}

fn svm_signing_key(signer: &Signer) -> Result<SigningKey, SdkError> {
    let Signer::Svm(secret) = signer else {
        return Err(SdkError::Config(
            "sign_svm_transfer requires an Svm signer".into(),
        ));
    };
    use secrecy::ExposeSecret;
    let raw = secret.expose_secret();
    let bytes = bs58::decode(raw.trim())
        .into_vec()
        .map_err(|_| SdkError::Config("Solana key is not valid base58".into()))?;
    // Solana secret keys are the 64-byte [secret(32) || public(32)] form.
    let seed: [u8; 32] = bytes
        .get(..32)
        .and_then(|s| s.try_into().ok())
        .ok_or_else(|| SdkError::Config("Solana key must be at least 32 bytes".into()))?;
    Ok(SigningKey::from_bytes(&seed))
}

fn decode_pubkey(b58: &str) -> Result<[u8; 32], SdkError> {
    let bytes = bs58::decode(b58)
        .into_vec()
        .map_err(|_| SdkError::Config(format!("invalid base58 pubkey: {b58}")))?;
    bytes
        .try_into()
        .map_err(|_| SdkError::Config(format!("pubkey must be 32 bytes: {b58}")))
}

// Derive the ATA PDA by searching bump values from 255 down.
fn associated_token_address(
    owner: &[u8; 32],
    token_program: &[u8; 32],
    mint: &[u8; 32],
) -> Result<[u8; 32], SdkError> {
    let ata_program = decode_pubkey(ASSOCIATED_TOKEN_PROGRAM_ID)?;
    for bump in (0u8..=255).rev() {
        let mut hasher = Sha256::new();
        hasher.update(owner);
        hasher.update(token_program);
        hasher.update(mint);
        hasher.update([bump]);
        hasher.update(ata_program);
        hasher.update(b"ProgramDerivedAddress");
        let candidate: [u8; 32] = hasher.finalize().into();
        // PDAs must be off-curve.
        if !is_on_curve(&candidate) {
            return Ok(candidate);
        }
    }
    Err(SdkError::Config(
        "could not derive associated token account (no off-curve bump)".into(),
    ))
}

// VerifyingKey parsing distinguishes on-curve points without another curve
// dependency.
fn is_on_curve(bytes: &[u8; 32]) -> bool {
    ed25519_dalek::VerifyingKey::from_bytes(bytes).is_ok()
}

// Build a versioned message for the payment instructions.
fn build_message(
    header: &MessageHeader,
    accounts: &[[u8; 32]],
    recent_blockhash: &str,
    instructions: &[Instruction],
) -> Result<Vec<u8>, SdkError> {
    let blockhash = decode_pubkey(recent_blockhash)?; // 32-byte hash, base58

    // Version prefix followed by account-permission counts.
    let mut msg = vec![
        V0_MESSAGE_PREFIX,
        header.num_required_signatures,
        header.num_readonly_signed,
        header.num_readonly_unsigned,
    ];
    write_compact_u16(&mut msg, accounts.len() as u16);
    for acct in accounts {
        msg.extend_from_slice(acct);
    }
    msg.extend_from_slice(&blockhash);
    write_compact_u16(&mut msg, instructions.len() as u16);
    for ix in instructions {
        msg.push(ix.program_index);
        write_compact_u16(&mut msg, ix.account_indexes.len() as u16);
        msg.extend_from_slice(&ix.account_indexes);
        write_compact_u16(&mut msg, ix.data.len() as u16);
        msg.extend_from_slice(&ix.data);
    }
    // No address-table lookups.
    write_compact_u16(&mut msg, 0);
    Ok(msg)
}

// Encode a Solana compact-u16 length.
fn write_compact_u16(out: &mut Vec<u8>, mut value: u16) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // Deterministic, unfunded test keypair.
    fn throwaway_signer() -> Signer {
        // Fixed seed for deterministic tests.
        let seed = [1u8; 32];
        let key = SigningKey::from_bytes(&seed);
        let mut full = Vec::with_capacity(64);
        full.extend_from_slice(&seed);
        full.extend_from_slice(&key.verifying_key().to_bytes());
        Signer::Svm(bs58::encode(full).into_string().into())
    }

    #[test]
    fn compact_u16_encoding() {
        let mut buf = Vec::new();
        write_compact_u16(&mut buf, 1);
        assert_eq!(buf, vec![1]);
        buf.clear();
        write_compact_u16(&mut buf, 128);
        assert_eq!(buf, vec![0x80, 0x01]);
    }

    #[test]
    fn svm_address_is_base58_pubkey() {
        let signer = throwaway_signer();
        let addr = signer.svm_address().unwrap();
        // 32-byte pubkey → 43-44 base58 chars.
        assert!(addr.len() >= 43 && addr.len() <= 44, "addr: {addr}");
        assert!(bs58::decode(&addr).into_vec().unwrap().len() == 32);
    }

    #[test]
    fn ata_is_deterministic_and_off_curve() {
        let owner = [2u8; 32];
        let token_program = decode_pubkey(TOKEN_PROGRAM_ID).unwrap();
        let mint = [3u8; 32];
        let a = associated_token_address(&owner, &token_program, &mint).unwrap();
        let b = associated_token_address(&owner, &token_program, &mint).unwrap();
        assert_eq!(a, b);
        assert!(!is_on_curve(&a));
    }

    fn test_request() -> SvmTransferRequest {
        SvmTransferRequest {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
            pay_to: bs58::encode([4u8; 32]).into_string(),
            fee_payer: bs58::encode([5u8; 32]).into_string(),
            amount: 1000,
            decimals: 6,
            recent_blockhash: bs58::encode([6u8; 32]).into_string(),
            token_program: TOKEN_PROGRAM_ID.into(),
            memo: "0123456789abcdef".into(),
        }
    }

    #[test]
    fn transfer_produces_two_sig_slots_with_payer_filled() {
        let signer = throwaway_signer();
        let tx = signer.sign_svm_transfer(&test_request()).unwrap();
        // Two signatures precede the message; the gateway slot is empty.
        assert_eq!(tx[0], 2);
        assert_eq!(&tx[1..65], &[0u8; 64]);
        assert!(tx[65..129].iter().any(|&b| b != 0));
    }

    // Lock the gateway-required v0 message shape.
    #[test]
    fn message_is_v0_with_four_instructions() {
        let signer = throwaway_signer();
        let tx = signer.sign_svm_transfer(&test_request()).unwrap();
        let msg = &tx[129..];

        assert_eq!(msg[0], V0_MESSAGE_PREFIX, "v0 version prefix");
        assert_eq!(&msg[1..4], &[2, 0, 4], "header: 2 signers, 4 readonly");
        assert_eq!(msg[4], 8, "account count");

        // Prefix, header, and account count.
        let after_accounts = 5 + 8 * 32;
        let after_blockhash = after_accounts + 32;
        assert_eq!(msg[after_blockhash], 4, "four instructions");

        // Collect each instruction's program and opcode.
        let mut cursor = after_blockhash + 1;
        let mut seen = Vec::new();
        for _ in 0..4 {
            let program_index = msg[cursor];
            cursor += 1;
            let n_accounts = msg[cursor] as usize;
            cursor += 1 + n_accounts;
            let n_data = msg[cursor] as usize;
            cursor += 1;
            seen.push((program_index, msg[cursor], n_accounts));
            cursor += n_data;
        }

        // 6 = ComputeBudget, 5 = token program, 7 = Memo.
        assert_eq!(
            seen,
            vec![
                (6, SET_COMPUTE_UNIT_LIMIT, 0),
                (6, SET_COMPUTE_UNIT_PRICE, 0),
                (5, TRANSFER_CHECKED, 4),
                (7, b'0', 0),
            ]
        );

        // Empty address-table-lookup vector.
        assert_eq!(msg[cursor], 0, "no address table lookups");
        assert_eq!(cursor + 1, msg.len(), "message fully consumed");
    }

    #[test]
    fn oversized_memo_is_rejected() {
        let signer = throwaway_signer();
        let req = SvmTransferRequest {
            memo: "x".repeat(MAX_MEMO_BYTES + 1),
            ..test_request()
        };
        let err = signer.sign_svm_transfer(&req).unwrap_err();
        assert!(
            matches!(err, SdkError::Config(msg) if msg.contains("memo")),
            "expected a memo-size Config error"
        );
    }
}
