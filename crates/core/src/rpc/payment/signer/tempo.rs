//! MPP/Tempo native type-0x76 transaction signer.
//!
//! Matches the wire format produced by the `ox/tempo` (viem) reference encoder.
//! The credential's `payload.signature` is the **0x78 fee-payer handoff
//! envelope**: the sender signs a type-0x76 preimage (fee-payer slot = `0x00`
//! placeholder, `feeToken` skipped — the gateway sponsors gas), then
//! re-serializes with its own address in the fee-payer slot and the sig
//! appended. The gateway relay co-signs server-side.
//!
//! Sync, zero chain reads: `nonceKey:"expiring"` resolves locally
//! (`nonceKey = U256::MAX`, `nonce = 0`, `validBefore = min(now+25s, expiry)`)
//! and gas/fee caps are preset generous constants (the sponsor pays the fee, so
//! the caps cost the payer nothing — they only need to clear inclusion).

use std::num::NonZeroU64;

use alloy_primitives::{Address, Bytes, Signature, TxKind, U256};
use alloy_rlp::Encodable;
use secrecy::ExposeSecret;
use sha3::{Digest, Keccak256};
use tempo_primitives::transaction::tempo_transaction::{Call, TempoTransaction};

use super::secp;
use super::Signer;
use crate::errors::SdkError;

// TIP20 transferWithMemo(address,uint256,bytes32) selector.
const TRANSFER_WITH_MEMO_SELECTOR: [u8; 4] = [0x95, 0x77, 0x7d, 0x59];

// ERC-20/TIP-20 approve(address,uint256) selector.
const APPROVE_SELECTOR: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];

// Generous fixed gas/fee caps. Under `feePayer:true` the gateway sponsors the
// fee, so the sender's caps cost it nothing and only need to exceed inclusion
// cost — no fee/gas RPC estimation is required.
const DEFAULT_GAS_LIMIT: u64 = 150_000;
const DEFAULT_MAX_FEE_PER_GAS: u128 = 10_000_000_000; // 10 gwei
const DEFAULT_MAX_PRIORITY_FEE_PER_GAS: u128 = 2_000_000_000; // 2 gwei

// Gas cap for escrow channel txs: the sponsor policy maximum. These carry two
// calls (a token `approve` plus the escrow `open`/`topUp`), which together need
// well over 1.5M gas — a smaller budget runs the open frame out of gas. The
// sponsor pays the fee, and 2M × the 10 gwei fee cap stays under the sponsor
// policy's total-fee ceiling.
const ESCROW_GAS_LIMIT: u64 = 2_000_000;

/// Inputs for one MPP/Tempo charge, derived from the decoded challenge.
#[derive(Debug, Clone)]
pub struct TempoChargeRequest {
    pub chain_id: u64,
    /// TIP20 token id (challenge `currency`), `0x`-hex.
    pub currency: String,
    /// Payment recipient (challenge `request.recipient`), `0x`-hex.
    pub recipient: String,
    /// Amount in token base units (challenge `request.amount`).
    pub amount: u128,
    /// Challenge id (for the attribution memo).
    pub challenge_id: String,
    /// Challenge realm / server id (for the attribution memo).
    pub realm: String,
    /// `validBefore` = min(now+25s, challenge expiry) as unix seconds,
    /// computed by the driver against the local clock.
    pub valid_before: u64,
    /// Optional overrides for the fixed gas/fee caps.
    pub gas_limit: Option<u64>,
    pub max_fee_per_gas: Option<u128>,
    pub max_priority_fee_per_gas: Option<u128>,
}

impl Signer {
    /// Sign an MPP/Tempo charge. Returns the 0x78 fee-payer handoff envelope
    /// bytes (the credential's `payload.signature`). Sync, no chain reads.
    pub fn sign_tempo_tx(&self, req: &TempoChargeRequest) -> Result<Vec<u8>, SdkError> {
        let Signer::Tempo(secret) = self else {
            return Err(SdkError::Config(
                "sign_tempo_tx requires a Tempo signer".into(),
            ));
        };
        let key = secp::signing_key(secret.expose_secret())?;
        let sender_hex = secp::evm_address(&key);
        let sender: Address = sender_hex
            .parse()
            .map_err(|_| SdkError::Config("derived sender address is invalid".into()))?;

        let token: Address = parse_address(&req.currency)?;
        let calldata = transfer_with_memo_calldata(req)?;
        let gas_limit = req.gas_limit.unwrap_or(DEFAULT_GAS_LIMIT);
        let max_fee = req.max_fee_per_gas.unwrap_or(DEFAULT_MAX_FEE_PER_GAS);
        let max_prio = req
            .max_priority_fee_per_gas
            .unwrap_or(DEFAULT_MAX_PRIORITY_FEE_PER_GAS);
        let valid_before = NonZeroU64::new(req.valid_before)
            .ok_or_else(|| SdkError::Config("validBefore must be non-zero".into()))?;

        let tx = TempoTransaction {
            chain_id: req.chain_id,
            fee_token: None,
            max_priority_fee_per_gas: max_prio,
            max_fee_per_gas: max_fee,
            gas_limit,
            calls: vec![Call {
                to: TxKind::Call(token),
                value: U256::ZERO,
                input: Bytes::from(calldata),
            }],
            access_list: Default::default(),
            nonce_key: U256::MAX, // TEMPO_EXPIRING_NONCE_KEY (TIP-1009)
            nonce: 0,
            // Presence of a fee-payer signature drives the 0x00 placeholder +
            // feeToken skip in encode_for_signing; the value is not encoded.
            fee_payer_signature: Some(Signature::new(U256::from(1), U256::from(1), false)),
            valid_before: Some(valid_before),
            valid_after: None,
            key_authorization: None,
            tempo_authorization_list: vec![],
        };

        // 1. Sender preimage (0x76, fee-payer placeholder, feeToken skipped).
        let sign_hash = tx.signature_hash();
        let sig65 = secp::sign_prehash_65(&key, &sign_hash.0);

        // 2. Fee-payer handoff envelope (0x78): the same fields with the sender
        //    address in the fee-payer slot and the sender sig appended.
        //    `tempo-primitives` has no public serializer for this exact form, so
        //    it is assembled field-by-field with alloy-rlp (see encode_handoff).
        Ok(encode_handoff(
            req.chain_id,
            max_prio,
            max_fee,
            gas_limit,
            &tx.calls,
            &tx.access_list,
            req.valid_before,
            sender,
            &sig65,
        ))
    }

    /// Sign a legacy contract-backed escrow channel `open` or `topUp`
    /// transaction. The tx carries two calls — a token `approve(escrow, amount)`
    /// plus the escrow call — and rides the same fee-sponsored 0x78 handoff
    /// envelope as a charge. Returns the envelope bytes (the credential's
    /// `transaction`) plus, for `open`, the derived channelId. Sync, no chain
    /// reads.
    pub fn sign_escrow_tx(&self, req: &TempoEscrowRequest) -> Result<TempoEscrowSigned, SdkError> {
        let Signer::Tempo(secret) = self else {
            return Err(SdkError::Config(
                "sign_escrow_tx requires a Tempo signer".into(),
            ));
        };
        let key = secp::signing_key(secret.expose_secret())?;
        let sender_hex = secp::evm_address(&key);
        let sender: Address = sender_hex
            .parse()
            .map_err(|_| SdkError::Config("derived sender address is invalid".into()))?;

        let escrow: Address = parse_address(&req.escrow_contract)?;
        let token: Address = parse_address(req.action.token())?;
        let approve = approve_calldata(&req.escrow_contract, req.action.amount())?;
        let escrow_call = req.action.calldata()?;
        let gas_limit = ESCROW_GAS_LIMIT;
        let max_fee = DEFAULT_MAX_FEE_PER_GAS;
        let max_prio = DEFAULT_MAX_PRIORITY_FEE_PER_GAS;
        let valid_before = NonZeroU64::new(req.valid_before)
            .ok_or_else(|| SdkError::Config("validBefore must be non-zero".into()))?;

        let tx = TempoTransaction {
            chain_id: req.chain_id,
            fee_token: None,
            max_priority_fee_per_gas: max_prio,
            max_fee_per_gas: max_fee,
            gas_limit,
            calls: vec![
                Call {
                    to: TxKind::Call(token),
                    value: U256::ZERO,
                    input: Bytes::from(approve),
                },
                Call {
                    to: TxKind::Call(escrow),
                    value: U256::ZERO,
                    input: Bytes::from(escrow_call),
                },
            ],
            access_list: Default::default(),
            nonce_key: U256::MAX,
            nonce: 0,
            fee_payer_signature: Some(Signature::new(U256::from(1), U256::from(1), false)),
            valid_before: Some(valid_before),
            valid_after: None,
            key_authorization: None,
            tempo_authorization_list: vec![],
        };

        let sign_hash = tx.signature_hash();
        let sig65 = secp::sign_prehash_65(&key, &sign_hash.0);
        let transaction = encode_handoff(
            req.chain_id,
            max_prio,
            max_fee,
            gas_limit,
            &tx.calls,
            &tx.access_list,
            req.valid_before,
            sender,
            &sig65,
        );

        // channelId is only defined for open; a top-up references an existing one.
        let channel_id = match &req.action {
            EscrowAction::Open {
                payee,
                token,
                salt,
                authorized_signer,
                ..
            } => Some(compute_channel_id(
                &sender_hex,
                payee,
                token,
                salt,
                authorized_signer,
                &req.escrow_contract,
                req.chain_id,
            )?),
            EscrowAction::TopUp { .. } => None,
        };

        Ok(TempoEscrowSigned {
            transaction,
            channel_id,
        })
    }
}

/// A legacy contract-backed escrow channel management transaction to sign.
#[derive(Debug, Clone)]
pub struct TempoEscrowRequest {
    pub chain_id: u64,
    /// `validBefore` = min(now+25s, expiry), computed by the caller.
    pub valid_before: u64,
    /// The escrow contract from the session challenge's `methodDetails`.
    pub escrow_contract: String,
    pub action: EscrowAction,
}

/// The escrow contract call carried by a [`TempoEscrowRequest`].
#[derive(Debug, Clone)]
pub enum EscrowAction {
    /// `open(payee, token, deposit, salt, authorizedSigner)`.
    Open {
        payee: String,
        token: String,
        deposit: u128,
        /// 32-byte payer entropy, `0x`-hex.
        salt: String,
        authorized_signer: String,
    },
    /// `topUp(channelId, additionalDeposit)`.
    TopUp {
        /// TIP-1034-style channel id (`0x`-hex bytes32).
        channel_id: String,
        /// The channel token, needed for the paired `approve` call.
        token: String,
        additional_deposit: u128,
    },
}

/// The result of signing an escrow management transaction.
#[derive(Debug, Clone)]
pub struct TempoEscrowSigned {
    /// 0x78 fee-payer handoff envelope bytes (the credential `transaction`).
    pub transaction: Vec<u8>,
    /// Derived channelId (`open` only; `None` for `topUp`).
    pub channel_id: Option<[u8; 32]>,
}

impl EscrowAction {
    // The channel token: the target of the paired `approve` call.
    fn token(&self) -> &str {
        match self {
            EscrowAction::Open { token, .. } => token,
            EscrowAction::TopUp { token, .. } => token,
        }
    }

    // The deposit moved by this action: the amount the `approve` must cover.
    fn amount(&self) -> u128 {
        match self {
            EscrowAction::Open { deposit, .. } => *deposit,
            EscrowAction::TopUp {
                additional_deposit, ..
            } => *additional_deposit,
        }
    }

    // ABI-encode the escrow contract calldata (selector ++ head words). All
    // args are static, so head-only encoding matches abi.encode exactly.
    fn calldata(&self) -> Result<Vec<u8>, SdkError> {
        match self {
            EscrowAction::Open {
                payee,
                token,
                deposit,
                salt,
                authorized_signer,
            } => {
                let selector = fn_selector(b"open(address,address,uint128,bytes32,address)");
                let mut data = Vec::with_capacity(4 + 5 * 32);
                data.extend_from_slice(&selector);
                data.extend_from_slice(&super::address_word(payee)?);
                data.extend_from_slice(&super::address_word(token)?);
                data.extend_from_slice(&u128_word(*deposit));
                data.extend_from_slice(&bytes32(salt)?);
                data.extend_from_slice(&super::address_word(authorized_signer)?);
                Ok(data)
            }
            EscrowAction::TopUp {
                channel_id,
                additional_deposit,
                ..
            } => {
                let selector = fn_selector(b"topUp(bytes32,uint256)");
                let mut data = Vec::with_capacity(4 + 2 * 32);
                data.extend_from_slice(&selector);
                data.extend_from_slice(&bytes32(channel_id)?);
                data.extend_from_slice(&u128_word(*additional_deposit));
                Ok(data)
            }
        }
    }
}

// ERC-20/TIP-20 approve(spender, amount): selector ++ 2×32-byte words.
fn approve_calldata(spender: &str, amount: u128) -> Result<Vec<u8>, SdkError> {
    let mut data = Vec::with_capacity(4 + 2 * 32);
    data.extend_from_slice(&APPROVE_SELECTOR);
    data.extend_from_slice(&super::address_word(spender)?);
    data.extend_from_slice(&u128_word(amount));
    Ok(data)
}

// keccak256(signature)[..4] function selector.
fn fn_selector(signature: &[u8]) -> [u8; 4] {
    let h = keccak(signature);
    [h[0], h[1], h[2], h[3]]
}

// A uint value as a 32-byte left-padded EVM word (uint128/uint256 encode
// identically for values that fit in 128 bits).
fn u128_word(value: u128) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[16..].copy_from_slice(&value.to_be_bytes());
    word
}

// A bytes32 hex value as a raw 32-byte word.
fn bytes32(hex_str: &str) -> Result<[u8; 32], SdkError> {
    let cleaned = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let bytes = hex::decode(cleaned)
        .map_err(|_| SdkError::Config(format!("invalid bytes32: {hex_str}")))?;
    if bytes.len() != 32 {
        return Err(SdkError::Config(format!(
            "bytes32 must be 32 bytes, got {}",
            bytes.len()
        )));
    }
    let mut word = [0u8; 32];
    word.copy_from_slice(&bytes);
    Ok(word)
}

// channelId = keccak256(abi.encode(payer, payee, token, salt,
//   authorizedSigner, escrowContract, uint256 chainId)) — all static words.
// Mirrors the escrow contract's computeChannelId (mppx Channel.computeId);
// the gateway re-derives this from the open calldata and requires a match.
fn compute_channel_id(
    payer: &str,
    payee: &str,
    token: &str,
    salt: &str,
    authorized_signer: &str,
    escrow: &str,
    chain_id: u64,
) -> Result<[u8; 32], SdkError> {
    let mut buf = Vec::with_capacity(7 * 32);
    buf.extend_from_slice(&super::address_word(payer)?);
    buf.extend_from_slice(&super::address_word(payee)?);
    buf.extend_from_slice(&super::address_word(token)?);
    buf.extend_from_slice(&bytes32(salt)?);
    buf.extend_from_slice(&super::address_word(authorized_signer)?);
    buf.extend_from_slice(&super::address_word(escrow)?);
    let mut chain_word = [0u8; 32];
    chain_word[24..].copy_from_slice(&chain_id.to_be_bytes());
    buf.extend_from_slice(&chain_word);
    Ok(keccak(&buf))
}

fn parse_address(addr: &str) -> Result<Address, SdkError> {
    addr.parse()
        .map_err(|_| SdkError::Config(format!("invalid address: {addr}")))
}

// TIP20 transferWithMemo(address,uint256,bytes32): selector ++ 3×32-byte words.
fn transfer_with_memo_calldata(req: &TempoChargeRequest) -> Result<Vec<u8>, SdkError> {
    let recipient = super::address_word(&req.recipient)?;
    let mut amount_word = [0u8; 32];
    amount_word[16..].copy_from_slice(&req.amount.to_be_bytes());
    let memo = attribution_memo(&req.realm, &req.challenge_id);

    let mut data = Vec::with_capacity(4 + 96);
    data.extend_from_slice(&TRANSFER_WITH_MEMO_SELECTOR);
    data.extend_from_slice(&recipient);
    data.extend_from_slice(&amount_word);
    data.extend_from_slice(&memo);
    Ok(data)
}

// mppx Attribution memo (bytes32):
//   keccak("mpp")[0..4] ++ 0x01 ++ keccak(realm)[0..10] ++ zeros[10] ++ keccak(challengeId)[0..7]
fn attribution_memo(realm: &str, challenge_id: &str) -> [u8; 32] {
    let mut memo = [0u8; 32];
    let mpp = keccak(b"mpp");
    memo[0..4].copy_from_slice(&mpp[0..4]);
    memo[4] = 0x01;
    let realm_hash = keccak(realm.as_bytes());
    memo[5..15].copy_from_slice(&realm_hash[0..10]);
    // bytes 15..25 stay zero (no clientId).
    let challenge_hash = keccak(challenge_id.as_bytes());
    memo[25..32].copy_from_slice(&challenge_hash[0..7]);
    memo
}

fn keccak(bytes: &[u8]) -> [u8; 32] {
    Keccak256::digest(bytes).into()
}

// 0x78 || rlp([chainId, maxPrioFee, maxFee, gas, calls, accessList, nonceKey,
//              nonce, validBefore, validAfter='', feeToken='', senderAddr,
//              authList=[], senderSig(65B)]).
#[allow(clippy::too_many_arguments)]
fn encode_handoff<A: Encodable>(
    chain_id: u64,
    max_prio: u128,
    max_fee: u128,
    gas_limit: u64,
    calls: &[Call],
    access_list: &A,
    valid_before: u64,
    sender: Address,
    sig65: &[u8; 65],
) -> Vec<u8> {
    let mut fields = Vec::new();
    chain_id.encode(&mut fields);
    max_prio.encode(&mut fields);
    max_fee.encode(&mut fields);
    gas_limit.encode(&mut fields);
    encode_calls(calls, &mut fields);
    access_list.encode(&mut fields);
    U256::MAX.encode(&mut fields);
    0u64.encode(&mut fields);
    valid_before.encode(&mut fields);
    fields.push(alloy_rlp::EMPTY_STRING_CODE); // validAfter absent
    fields.push(alloy_rlp::EMPTY_STRING_CODE); // feeToken (sender didn't commit)
    sender.encode(&mut fields); // fee-payer slot carries the sender address
    fields.push(alloy_rlp::EMPTY_LIST_CODE); // empty authorization list
    Bytes::from(sig65.to_vec()).encode(&mut fields); // sender SignatureEnvelope

    let mut out = Vec::with_capacity(fields.len() + 4);
    out.push(0x78);
    alloy_rlp::Header {
        list: true,
        payload_length: fields.len(),
    }
    .encode(&mut out);
    out.extend_from_slice(&fields);
    out
}

// RLP-encode the calls as a list: header(list, sum of encoded lengths) ++ each
// Call. Done explicitly rather than relying on a slice `Encodable` blanket so
// the encoding is independent of alloy-rlp's slice-impl surface.
fn encode_calls(calls: &[Call], out: &mut Vec<u8>) {
    let mut inner = Vec::new();
    for call in calls {
        call.encode(&mut inner);
    }
    alloy_rlp::Header {
        list: true,
        payload_length: inner.len(),
    }
    .encode(out);
    out.extend_from_slice(&inner);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // Reference vector generated offline by the `ox/tempo` encoder with the
    // publicly-known throwaway anvil key #0 (never funded) and fixed
    // validBefore/gas/fee inputs. Reproducing the 0x78 handoff bytes exactly
    // proves the MPP/Tempo construction matches the reference encoder.
    const KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const EXPECTED_HANDOFF: &str = "78f9011382a5bf830f4240843b9aca0083019a28f87ef87c9420c000000000000000000000000000000000000080b86495777d59000000000000000000000000fd24114c3981aba78ae2441991b1bdb89329c55600000000000000000000000000000000000000000000000000000000000003e8ef1ed712013846ebb93fa448b84b800000000000000000000060f498736fd943c0a0ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff80846a543ee5808094f39fd6e51aad88f6f4ce6ab8827279cfffb92266c0b841ca92118d9f7da00c84c2445bd3ee164cef9f60742771ca8a1700f15357f1437122ff663f076b0a54bbbfc614fb28f6c8e69a29735ad555ca71c25a889180e0c01c";

    // Reconstruct the exact calldata the vector used: transferWithMemo to
    // 0xfd24…c556, amount 1000, memo ef1e…d943.
    fn vector_request() -> TempoChargeRequest {
        // The vector's memo was computed from specific realm/challenge inputs;
        // to reproduce the exact bytes we bypass the memo builder by encoding
        // calldata directly in this test via a crafted request is not possible
        // (memo is derived). Instead we assert the handoff for the known memo
        // by constructing calldata to match. See below.
        TempoChargeRequest {
            chain_id: 42431,
            currency: "0x20c0000000000000000000000000000000000000".into(),
            recipient: "0xfd24114c3981aba78ae2441991b1bdb89329c556".into(),
            amount: 1000,
            challenge_id: String::new(),
            realm: String::new(),
            valid_before: 1_783_906_021,
            gas_limit: Some(105_000),
            max_fee_per_gas: Some(1_000_000_000),
            max_priority_fee_per_gas: Some(1_000_000),
        }
    }

    // The vector's memo bytes (ef1e…d943) — fixed by the captured challenge.
    const VECTOR_MEMO: &str = "ef1ed712013846ebb93fa448b84b800000000000000000000060f498736fd943";

    #[test]
    fn handoff_reproduces_stage1a_vector() {
        // Build calldata with the vector's exact memo (the builder is exercised
        // separately below); this isolates the tx-encoding + signing path.
        let key = secp::signing_key(KEY).unwrap();
        let sender: Address = secp::evm_address(&key).parse().unwrap();
        let token: Address = "0x20c0000000000000000000000000000000000000"
            .parse()
            .unwrap();

        let recipient = super::super::address_word(&vector_request().recipient).unwrap();
        let mut amount_word = [0u8; 32];
        amount_word[16..].copy_from_slice(&1000u128.to_be_bytes());
        let memo = hex::decode(VECTOR_MEMO).unwrap();
        let mut calldata = Vec::new();
        calldata.extend_from_slice(&TRANSFER_WITH_MEMO_SELECTOR);
        calldata.extend_from_slice(&recipient);
        calldata.extend_from_slice(&amount_word);
        calldata.extend_from_slice(&memo);

        let tx = TempoTransaction {
            chain_id: 42431,
            fee_token: None,
            max_priority_fee_per_gas: 1_000_000,
            max_fee_per_gas: 1_000_000_000,
            gas_limit: 105_000,
            calls: vec![Call {
                to: TxKind::Call(token),
                value: U256::ZERO,
                input: Bytes::from(calldata),
            }],
            access_list: Default::default(),
            nonce_key: U256::MAX,
            nonce: 0,
            fee_payer_signature: Some(Signature::new(U256::from(1), U256::from(1), false)),
            valid_before: NonZeroU64::new(1_783_906_021),
            valid_after: None,
            key_authorization: None,
            tempo_authorization_list: vec![],
        };
        let sign_hash = tx.signature_hash();
        let sig65 = secp::sign_prehash_65(&key, &sign_hash.0);
        let handoff = encode_handoff(
            42431,
            1_000_000,
            1_000_000_000,
            105_000,
            &tx.calls,
            &tx.access_list,
            1_783_906_021,
            sender,
            &sig65,
        );
        assert_eq!(hex::encode(&handoff), EXPECTED_HANDOFF);
    }

    #[test]
    fn attribution_memo_layout() {
        // Prefix + version byte are fixed regardless of inputs.
        let memo = attribution_memo("mpp.quicknode.com", "challenge-1");
        assert_eq!(memo[4], 0x01);
        // bytes 15..25 are the zero clientId gap.
        assert_eq!(&memo[15..25], &[0u8; 10]);
    }

    // Legacy contract-backed session vectors, generated offline with viem's
    // encodeFunctionData/encodeAbiParameters: anvil key #0 as payer, payee
    // 0xfd24…c556, token 0x20c0…0000, salt 0x22…22, authorizedSigner = payer,
    // escrow 0x33b9…4f25, chainId 42431. Reproducing them byte-for-byte proves
    // the ABI encodings match the reference client (mppx tempo/legacy/session).
    const V_PAYER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    const V_PAYEE: &str = "0xfd24114c3981aba78ae2441991b1bdb89329c556";
    const V_TOKEN: &str = "0x20c0000000000000000000000000000000000000";
    const V_ESCROW: &str = "0x33b901018174DDabE4841042ab76ba85D4e24f25";
    const V_CHANNEL_ID: &str = "0xfb56137dcb0089f01877bcdb72d5e028ef04aec578fb00a642f65ee293c73dec";

    fn v_salt() -> String {
        format!("0x{}", "22".repeat(32))
    }

    #[test]
    fn channel_id_reproduces_legacy_reference_vector() {
        let id = compute_channel_id(
            V_PAYER,
            V_PAYEE,
            V_TOKEN,
            &v_salt(),
            V_PAYER,
            V_ESCROW,
            42431,
        )
        .unwrap();
        assert_eq!(format!("0x{}", hex::encode(id)), V_CHANNEL_ID);
    }

    #[test]
    fn escrow_open_calldata_reproduces_legacy_reference_vector() {
        let action = EscrowAction::Open {
            payee: V_PAYEE.into(),
            token: V_TOKEN.into(),
            deposit: 1_000_000,
            salt: v_salt(),
            authorized_signer: V_PAYER.into(),
        };
        let data = action.calldata().unwrap();
        assert_eq!(
            format!("0x{}", hex::encode(data)),
            "0xc79ea485000000000000000000000000fd24114c3981aba78ae2441991b1bdb89329c556\
             00000000000000000000000020c0000000000000000000000000000000000000\
             00000000000000000000000000000000000000000000000000000000000f4240\
             2222222222222222222222222222222222222222222222222222222222222222\
             000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                .replace([' ', '\n'], "")
        );
    }

    #[test]
    fn escrow_top_up_calldata_reproduces_legacy_reference_vector() {
        let action = EscrowAction::TopUp {
            channel_id: V_CHANNEL_ID.into(),
            token: V_TOKEN.into(),
            additional_deposit: 500_000,
        };
        let data = action.calldata().unwrap();
        assert_eq!(
            format!("0x{}", hex::encode(data)),
            "0xb67644b9fb56137dcb0089f01877bcdb72d5e028ef04aec578fb00a642f65ee293c73dec\
             000000000000000000000000000000000000000000000000000000000007a120"
                .replace([' ', '\n'], "")
        );
    }

    #[test]
    fn approve_calldata_reproduces_legacy_reference_vector() {
        let data = approve_calldata(V_ESCROW, 1_000_000).unwrap();
        assert_eq!(
            format!("0x{}", hex::encode(data)),
            "0x095ea7b300000000000000000000000033b901018174ddabe4841042ab76ba85d4e24f25\
             00000000000000000000000000000000000000000000000000000000000f4240"
                .replace([' ', '\n'], "")
        );
    }
}
