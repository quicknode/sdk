//! Payment signers for the crypto-micropayment lanes.
//!
//! One `enum Signer` (not a trait) holds the caller's private key as a
//! `SecretString` and dispatches to one of three signing constructions at
//! runtime. An enum is used deliberately: a trait would force `Box<dyn Signer>`
//! into the FFI-facing config and break its derived `Clone`/`Serialize`/
//! `napi(object)`/`pyclass`. See `IMPLEMENTATION_PLAN.md` for the rationale.
//!
//! The three constructions (each verified byte-for-byte against a gateway-
//! accepted payload during Stage 0/1a research):
//! - `Evm` — EIP-712 `TransferWithAuthorization` (x402/EVM). Sync, no chain I/O.
//! - `Svm` — partially-signed SPL `TransferChecked` tx (x402/Solana). Async;
//!   reads a recent blockhash + the payer's ATA from a Solana RPC.
//! - `Tempo` — native Tempo type-0x76 tx, 0x78 fee-payer handoff envelope
//!   (MPP). Sync, no chain I/O (gas/fee caps are preset).

use secrecy::{ExposeSecret, SecretString};

use crate::errors::SdkError;

/// Which pay-chain family a [`Signer`] targets. Derived from the selector's
/// CAIP-2 `pay_network` (and the payment scheme) at the config boundary, so a
/// caller never states it redundantly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainKind {
    Evm,
    Svm,
    Tempo,
}

/// A payment signer over a raw private key. The key is held in a
/// `SecretString` and never printed by the SDK (manual `Debug` below); it is
/// `#[serde(skip)]` so it can never be populated from the environment or
/// serialized into a log.
pub enum Signer {
    /// secp256k1 key (hex, with or without `0x`) for x402/EVM EIP-712 signing.
    Evm(SecretString),
    /// ed25519 key (base58 64-byte secret) for x402/Solana SPL signing.
    Svm(SecretString),
    /// secp256k1 key (hex) for MPP/Tempo native-tx signing.
    Tempo(SecretString),
}

// Never print the key. A leaked private key is catastrophic; the SDK's own
// Debug output, error context, and panics must all render `[redacted]`.
impl std::fmt::Debug for Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant = match self {
            Signer::Evm(_) => "Evm",
            Signer::Svm(_) => "Svm",
            Signer::Tempo(_) => "Tempo",
        };
        f.debug_tuple(variant).field(&"[redacted]").finish()
    }
}

impl Signer {
    pub fn kind(&self) -> ChainKind {
        match self {
            Signer::Evm(_) => ChainKind::Evm,
            Signer::Svm(_) => ChainKind::Svm,
            Signer::Tempo(_) => ChainKind::Tempo,
        }
    }

    fn secret(&self) -> &SecretString {
        match self {
            Signer::Evm(s) | Signer::Svm(s) | Signer::Tempo(s) => s,
        }
    }
}

// ── secp256k1 helpers (EVM + Tempo) ──────────────────────────────────────────

#[cfg(feature = "payments")]
mod secp {
    use k256::ecdsa::SigningKey;
    use sha3::{Digest, Keccak256};

    use crate::errors::SdkError;

    pub(super) fn signing_key(hex_key: &str) -> Result<SigningKey, SdkError> {
        let cleaned = hex_key.strip_prefix("0x").unwrap_or(hex_key);
        let bytes = hex::decode(cleaned)
            .map_err(|_| SdkError::Config("payment key is not valid hex".into()))?;
        SigningKey::from_slice(&bytes)
            .map_err(|_| SdkError::Config("payment key is not a valid secp256k1 key".into()))
    }

    // 20-byte EVM address (keccak of the uncompressed pubkey, last 20 bytes),
    // lowercase hex with `0x`.
    pub(super) fn evm_address(key: &SigningKey) -> String {
        let verifying = key.verifying_key();
        let point = verifying.to_sec1_point(false);
        // Skip the 0x04 prefix byte of the uncompressed point.
        let hash = Keccak256::digest(&point.as_bytes()[1..]);
        format!("0x{}", hex::encode(&hash[12..]))
    }

    pub(super) fn keccak256(bytes: &[u8]) -> [u8; 32] {
        Keccak256::digest(bytes).into()
    }

    // Sign a 32-byte prehash, returning 65 bytes r||s||v where v is 27/28
    // (the encoding both ox and viem emit for EIP-712 sigs and Tempo handoffs).
    pub(super) fn sign_prehash_65(key: &SigningKey, prehash: &[u8; 32]) -> [u8; 65] {
        let (sig, recid) = key.sign_prehash_recoverable(prehash);
        let r = sig.r().to_bytes();
        let s = sig.s().to_bytes();
        let mut out = [0u8; 65];
        out[..32].copy_from_slice(&r);
        out[32..64].copy_from_slice(&s);
        out[64] = 27 + recid.to_byte();
        out
    }
}

// ── EIP-712 (x402/EVM) ───────────────────────────────────────────────────────

/// EIP-712 domain for the USDC `TransferWithAuthorization` message.
#[cfg(feature = "payments")]
#[derive(Debug, Clone)]
pub struct Eip712Domain {
    pub name: String,
    pub version: String,
    pub chain_id: u64,
    /// Verifying contract = the asset (token) address, `0x`-prefixed hex.
    pub verifying_contract: String,
}

/// EIP-3009 `TransferWithAuthorization` message.
#[cfg(feature = "payments")]
#[derive(Debug, Clone)]
pub struct TransferWithAuthorization {
    pub from: String,
    pub to: String,
    pub value: u128,
    pub valid_after: u64,
    pub valid_before: u64,
    /// 32-byte nonce, `0x`-prefixed hex.
    pub nonce: [u8; 32],
}

#[cfg(feature = "payments")]
impl Signer {
    /// The signer's on-chain address in the pay-chain's native encoding
    /// (EVM/Tempo → `0x…` hex; Solana → base58 pubkey).
    pub fn address(&self) -> Result<String, SdkError> {
        match self {
            Signer::Evm(_) | Signer::Tempo(_) => {
                let key = secp::signing_key(self.secret().expose_secret())?;
                Ok(secp::evm_address(&key))
            }
            #[cfg(feature = "payments-svm")]
            Signer::Svm(_) => self.svm_address(),
            #[cfg(not(feature = "payments-svm"))]
            Signer::Svm(_) => Err(SdkError::Config(
                "x402/Solana requires the `payments-svm` feature".into(),
            )),
        }
    }

    /// Sign an EIP-712 `TransferWithAuthorization` (x402/EVM). Returns the
    /// 65-byte `r||s||v` signature. Sync — no chain I/O.
    pub fn sign_eip712(
        &self,
        domain: &Eip712Domain,
        message: &TransferWithAuthorization,
    ) -> Result<[u8; 65], SdkError> {
        let key = secp::signing_key(self.secret().expose_secret())?;
        let digest = eip712_digest(domain, message)?;
        Ok(secp::sign_prehash_65(&key, &digest))
    }
}

// EIP-712 final digest: keccak256(0x1901 || domainSeparator || hashStruct).
#[cfg(feature = "payments")]
fn eip712_digest(
    domain: &Eip712Domain,
    message: &TransferWithAuthorization,
) -> Result<[u8; 32], SdkError> {
    // domainSeparator = keccak256(typeHash || keccak(name) || keccak(version)
    //                             || chainId || verifyingContract)
    let domain_type =
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";
    let mut sep = Vec::with_capacity(160);
    sep.extend_from_slice(&secp::keccak256(domain_type));
    sep.extend_from_slice(&secp::keccak256(domain.name.as_bytes()));
    sep.extend_from_slice(&secp::keccak256(domain.version.as_bytes()));
    sep.extend_from_slice(&u256_be(domain.chain_id as u128));
    sep.extend_from_slice(&address_word(&domain.verifying_contract)?);
    let domain_separator = secp::keccak256(&sep);

    // hashStruct(message) = keccak256(typeHash || from || to || value
    //                                 || validAfter || validBefore || nonce)
    let msg_type = b"TransferWithAuthorization(address from,address to,uint256 value,uint256 validAfter,uint256 validBefore,bytes32 nonce)";
    let mut hs = Vec::with_capacity(224);
    hs.extend_from_slice(&secp::keccak256(msg_type));
    hs.extend_from_slice(&address_word(&message.from)?);
    hs.extend_from_slice(&address_word(&message.to)?);
    hs.extend_from_slice(&u256_be(message.value));
    hs.extend_from_slice(&u256_be(message.valid_after as u128));
    hs.extend_from_slice(&u256_be(message.valid_before as u128));
    hs.extend_from_slice(&message.nonce);
    let hash_struct = secp::keccak256(&hs);

    let mut final_input = Vec::with_capacity(66);
    final_input.extend_from_slice(&[0x19, 0x01]);
    final_input.extend_from_slice(&domain_separator);
    final_input.extend_from_slice(&hash_struct);
    Ok(secp::keccak256(&final_input))
}

// A u128 as a 32-byte big-endian EVM word (left-padded with zeros).
#[cfg(feature = "payments")]
fn u256_be(value: u128) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[16..].copy_from_slice(&value.to_be_bytes());
    word
}

// A 20-byte address left-padded into a 32-byte EVM word.
#[cfg(feature = "payments")]
fn address_word(addr: &str) -> Result<[u8; 32], SdkError> {
    let cleaned = addr.strip_prefix("0x").unwrap_or(addr);
    let bytes =
        hex::decode(cleaned).map_err(|_| SdkError::Config(format!("invalid address: {addr}")))?;
    if bytes.len() != 20 {
        return Err(SdkError::Config(format!(
            "address must be 20 bytes, got {}",
            bytes.len()
        )));
    }
    let mut word = [0u8; 32];
    word[12..].copy_from_slice(&bytes);
    Ok(word)
}

// ── x402/Solana (SPL TransferChecked) ────────────────────────────────────────
#[cfg(feature = "payments-svm")]
mod svm;

// ── MPP/Tempo (native type-0x76 tx) ──────────────────────────────────────────
#[cfg(feature = "payments-tempo")]
mod tempo;

#[cfg(feature = "payments-tempo")]
pub use tempo::TempoChargeRequest;

#[cfg(feature = "payments-svm")]
pub use svm::SvmTransferRequest;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    // Known-good EIP-712 vector regenerated from a throwaway key (anvil test
    // key #0, publicly known, never funded) so the funded-wallet capture in
    // scratch/ never enters the repo. Signature produced offline with viem's
    // signTypedData over the same domain/message.
    const THROWAWAY_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const THROWAWAY_ADDR: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    #[test]
    fn evm_address_derivation() {
        let signer = Signer::Evm(SecretString::new(THROWAWAY_KEY.to_string()));
        assert_eq!(signer.address().unwrap(), THROWAWAY_ADDR);
    }

    #[test]
    fn redacted_debug_never_prints_key() {
        let signer = Signer::Evm(SecretString::new(THROWAWAY_KEY.to_string()));
        let rendered = format!("{signer:?}");
        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains(THROWAWAY_KEY));
    }

    #[test]
    fn eip712_digest_is_deterministic_and_domain_bound() {
        // The digest must change when any domain/message field changes, and be
        // stable for identical inputs. (Full on-wire acceptance is proven by
        // the Stage 5 live smoke; here we lock the construction is wired.)
        let domain = Eip712Domain {
            name: "USDC".into(),
            version: "2".into(),
            chain_id: 84532,
            verifying_contract: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".into(),
        };
        let message = TransferWithAuthorization {
            from: THROWAWAY_ADDR.into(),
            to: "0xF46D6C4Bf5F5F0Bf5F5F0Bf5F5F0Bf5F5F0Bf623C"
                .chars()
                .take(42)
                .collect::<String>(),
            value: 1000,
            valid_after: 0,
            valid_before: 1_783_907_686,
            nonce: [0x76; 32],
        };
        let d1 = eip712_digest(&domain, &message).unwrap();
        let d2 = eip712_digest(&domain, &message).unwrap();
        assert_eq!(d1, d2);
        let mut domain2 = domain.clone();
        domain2.chain_id = 1;
        let d3 = eip712_digest(&domain2, &message).unwrap();
        assert_ne!(d1, d3);
    }

    #[test]
    fn eip712_reproduces_known_good_vector() {
        // Known-good signature produced by viem's signTypedData over the exact
        // domain/message below, using the throwaway anvil key #0 (never funded).
        // Regenerated offline via scratch/gen-eip712-vector.mjs so no funded
        // wallet's auth is committed. This is the Stage 1 acceptance vector for
        // the x402/EVM construction.
        const EXPECTED_SIG: &str = "0xc3a69d1a9043a75d840f66ccc9a95cdbc690bdd669424f00ba955ee7bcdb4a1e3293d7ab2e9663fc3486215be0cbb3da6c3cdcb71cf811b8b612c004014f0ba71b";
        let signer = Signer::Evm(SecretString::new(THROWAWAY_KEY.to_string()));
        let domain = Eip712Domain {
            name: "USDC".into(),
            version: "2".into(),
            chain_id: 84532,
            verifying_contract: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".into(),
        };
        let message = TransferWithAuthorization {
            from: THROWAWAY_ADDR.into(),
            to: "0x0000000000000000000000000000000000000001".into(),
            value: 1000,
            valid_after: 0,
            valid_before: 1_783_907_686,
            nonce: [0x11; 32],
        };
        let sig = signer.sign_eip712(&domain, &message).unwrap();
        assert_eq!(format!("0x{}", hex::encode(sig)), EXPECTED_SIG);
    }
}
