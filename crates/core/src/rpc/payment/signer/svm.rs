//! x402/Solana SPL `TransferChecked` signer.
//!
//! Builds a partially-signed Solana transaction: the gateway's `feePayer`
//! (from the challenge `extra.feePayer`) is the transaction fee payer and the
//! first required signature slot, so the payer needs no SOL — only the token.
//! The payer signs its own slot; the gateway co-signs the fee-payer slot
//! server-side before submitting.
//!
//! The SPL `TransferChecked` instruction is hand-rolled (a 4-account,
//! 10-byte-data instruction) rather than pulling `spl-token`, which drags
//! `solana-program` → curve25519/MSRV conflicts under cross+zig at
//! glibc-2.17/musl.
//!
//! Async: the payer's associated token account and a recent blockhash are read
//! from a Solana RPC (source precedence resolved by the driver: explicit
//! override → tooling endpoint → public default). The gateway 402s keyless
//! sub-reads, so these reads go to a plain Solana RPC, not the gateway.

use ed25519_dalek::{Signer as _, SigningKey};
use sha2::{Digest, Sha256};

use super::Signer;
use crate::errors::SdkError;

// SPL Token and Token-2022 program ids (base58), and the Associated Token
// Account program id. Hand-embedded to avoid the spl-token dependency.
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

// SPL TransferChecked instruction discriminant.
const TRANSFER_CHECKED: u8 = 12;

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
    /// Whether the mint is a Token-2022 mint (selects the token program).
    pub token_2022: bool,
}

impl Signer {
    // base58 ed25519 pubkey (Solana address) of the payer.
    pub(super) fn svm_address(&self) -> Result<String, SdkError> {
        let key = svm_signing_key(self)?;
        Ok(bs58::encode(key.verifying_key().to_bytes()).into_string())
    }

    /// Build a partially-signed SPL `TransferChecked` transaction (x402/Solana).
    /// Returns the serialized signed transaction bytes (the gateway base64s
    /// them into the payment envelope's `payload`).
    pub fn sign_svm_transfer(&self, req: &SvmTransferRequest) -> Result<Vec<u8>, SdkError> {
        let key = svm_signing_key(self)?;
        let payer = key.verifying_key().to_bytes();

        let token_program = decode_pubkey(if req.token_2022 {
            TOKEN_2022_PROGRAM_ID
        } else {
            TOKEN_PROGRAM_ID
        })?;
        let mint = decode_pubkey(&req.mint)?;
        let pay_to_owner = decode_pubkey(&req.pay_to)?;
        let fee_payer = decode_pubkey(&req.fee_payer)?;

        // Derive the source and destination associated token accounts.
        let source_ata = associated_token_address(&payer, &token_program, &mint)?;
        let dest_ata = associated_token_address(&pay_to_owner, &token_program, &mint)?;

        // TransferChecked: accounts = [source, mint, dest, owner(=payer signer)].
        // data = discriminant(1) || amount(u64 LE) || decimals(1).
        let mut data = Vec::with_capacity(10);
        data.push(TRANSFER_CHECKED);
        data.extend_from_slice(&req.amount.to_le_bytes());
        data.push(req.decimals);

        let message = build_message(
            &fee_payer,
            &payer,
            &token_program,
            &decode_pubkey(SYSTEM_PROGRAM_ID)?,
            &source_ata,
            &mint,
            &dest_ata,
            &req.recent_blockhash,
            &data,
        )?;

        // Legacy transaction wire format:
        //   compact-u16 signature count || signatures(64B each) || message.
        // Two signers (fee payer + payer); we fill the payer's slot and leave
        // the fee-payer slot zeroed for the gateway to co-sign.
        let payer_sig = key.sign(&message).to_bytes();
        let mut tx = Vec::new();
        write_compact_u16(&mut tx, 2);
        tx.extend_from_slice(&[0u8; 64]); // fee-payer slot (gateway fills)
        tx.extend_from_slice(&payer_sig); // payer slot
        tx.extend_from_slice(&message);
        Ok(tx)
    }
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

// Associated Token Account = find_program_address([owner, token_program, mint],
// ATA program). We search for the off-curve PDA by decrementing the bump.
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
        // A valid PDA must be OFF the ed25519 curve.
        if !is_on_curve(&candidate) {
            return Ok(candidate);
        }
    }
    Err(SdkError::Config(
        "could not derive associated token account (no off-curve bump)".into(),
    ))
}

// A point is on the ed25519 curve if it decompresses to a valid point. A valid
// PDA must be OFF the curve; `VerifyingKey::from_bytes` succeeds exactly when
// the bytes decompress to a curve point, so we reuse it (no direct
// curve25519-dalek dependency).
fn is_on_curve(bytes: &[u8; 32]) -> bool {
    ed25519_dalek::VerifyingKey::from_bytes(bytes).is_ok()
}

// Build a legacy Solana transaction message for a single TransferChecked ix.
// Account ordering (writable-signers, readonly-signers, writable-nonsigners,
// readonly-nonsigners) is required by the runtime's header semantics.
#[allow(clippy::too_many_arguments)]
fn build_message(
    fee_payer: &[u8; 32],
    payer_signer: &[u8; 32],
    token_program: &[u8; 32],
    _system_program: &[u8; 32],
    source_ata: &[u8; 32],
    mint: &[u8; 32],
    dest_ata: &[u8; 32],
    recent_blockhash: &str,
    ix_data: &[u8],
) -> Result<Vec<u8>, SdkError> {
    // Ordered account list:
    //   0: fee_payer     (writable signer)   — gateway
    //   1: payer_signer  (writable signer)   — the SPL token owner
    //   2: source_ata    (writable nonsigner)
    //   3: dest_ata      (writable nonsigner)
    //   4: mint          (readonly nonsigner)
    //   5: token_program (readonly nonsigner)
    let accounts: Vec<[u8; 32]> = vec![
        *fee_payer,
        *payer_signer,
        *source_ata,
        *dest_ata,
        *mint,
        *token_program,
    ];
    let num_required_signatures: u8 = 2;
    let num_readonly_signed: u8 = 0;
    let num_readonly_unsigned: u8 = 2; // mint + token_program

    let index =
        |pk: &[u8; 32]| -> u8 { accounts.iter().position(|a| a == pk).map_or(0, |p| p as u8) };

    // TransferChecked account metas: [source, mint, dest, owner].
    let ix_accounts = [
        index(source_ata),
        index(mint),
        index(dest_ata),
        index(payer_signer),
    ];
    let program_index = index(token_program);

    let blockhash = decode_pubkey(recent_blockhash)?; // 32-byte hash, base58

    let mut msg = Vec::new();
    msg.push(num_required_signatures);
    msg.push(num_readonly_signed);
    msg.push(num_readonly_unsigned);
    write_compact_u16(&mut msg, accounts.len() as u16);
    for acct in &accounts {
        msg.extend_from_slice(acct);
    }
    msg.extend_from_slice(&blockhash);
    // One instruction.
    write_compact_u16(&mut msg, 1);
    msg.push(program_index);
    write_compact_u16(&mut msg, ix_accounts.len() as u16);
    msg.extend_from_slice(&ix_accounts);
    write_compact_u16(&mut msg, ix_data.len() as u16);
    msg.extend_from_slice(ix_data);
    Ok(msg)
}

// Solana compact-u16 (shortvec) length encoding.
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

    // A throwaway Solana keypair (32-byte seed, publicly known anvil-style
    // filler — never funded). base58 of 64 bytes [seed||pub].
    fn throwaway_signer() -> Signer {
        // Seed of all 1s; deterministic for the test.
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

    #[test]
    fn transfer_produces_two_sig_slots_with_payer_filled() {
        let signer = throwaway_signer();
        let req = SvmTransferRequest {
            mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
            pay_to: "2LWbc9MihDfP4JR7YrE5MNrCq4Yd6qcT57tAt1v1qcT5"
                .chars()
                .take(44)
                .collect(),
            fee_payer: "GVJJ7rdGqjNjBqKxY9YqZ3xQ5vN8dKZ8Q9dVebDveb1"
                .chars()
                .take(43)
                .collect(),
            amount: 1000,
            decimals: 6,
            recent_blockhash: "11111111111111111111111111111111".into(),
            token_2022: false,
        };
        // pay_to / fee_payer above may not be valid base58 pubkeys; use real
        // 32-byte-decodable values instead.
        let req = SvmTransferRequest {
            pay_to: bs58::encode([4u8; 32]).into_string(),
            fee_payer: bs58::encode([5u8; 32]).into_string(),
            recent_blockhash: bs58::encode([6u8; 32]).into_string(),
            ..req
        };
        let tx = signer.sign_svm_transfer(&req).unwrap();
        // compact-u16(2) = 1 byte, then 2×64 sig bytes, then message.
        assert_eq!(tx[0], 2);
        // Fee-payer slot (bytes 1..65) is zeroed for the gateway.
        assert_eq!(&tx[1..65], &[0u8; 64]);
        // Payer slot (65..129) is filled (non-zero).
        assert!(tx[65..129].iter().any(|&b| b != 0));
    }
}
