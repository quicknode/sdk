//! x402 credit drawdown lane for `rpc.call`.
//!
//! Distinct from the per-request 402 loop in the parent module: instead of
//! signing a fresh settlement per call, the caller authenticates once with a
//! SIWX (Sign-In-With-X) message and receives a session JWT. Each drawdown call
//! then presents `Authorization: Bearer <JWT>` and draws 1 credit per successful
//! response — no per-call signing.
//!
//! The flow:
//! 1. [`authenticate`] — build a SIWE (EIP-4361) message, sign it with the
//!    payment key, POST `/auth`, and cache the returned [`GatewaySession`].
//! 2. [`drawdown_call`] — POST `/:network` with the Bearer JWT; returns the raw
//!    JSON-RPC envelope text.
//! 3. [`credits`] — GET `/credits` with the Bearer JWT → the current balance.
//! 4. [`drip`] — POST `/drip` (testnet faucet, once per account) — funds the
//!    wallet, not the credit ledger.
//!
//! [`buy_credits`] settles a credit block by signing the gateway's credit-tier
//! offer. It is reachable only where that offer's construction is signable; see
//! [`super::authorize_x402_credit`].
//!
//! State (the JWT) is held by the caller: the SDK is stateless here, so a host
//! persists [`GatewaySession`] between runs exactly as it does the tooling
//! [`crate::config::CachedToken`].

use serde::Deserialize;
use serde_json::Value;

use crate::admin::tooling_access::parse_rfc3339_to_unix;
use crate::errors::SdkError;

use super::{now_unix, random_nonce, ResolvedPayment};

/// A gateway session JWT plus its expiry and the account it authenticates.
/// This is the unit the drawdown lane caches; a host (the CLI) persists it
/// between processes and re-seeds it next run, the same pattern as
/// [`crate::config::CachedToken`].
///
/// `token` is a live bearer credential and is redacted in `Debug`.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct GatewaySession {
    /// The session JWT, presented as `Authorization: Bearer <token>`.
    pub token: String,
    /// JWT expiry in unix seconds (from the gateway's `expiresAt`).
    pub exp_unix: i64,
    /// The CAIP-10 account the JWT authenticates (the payer's address on the
    /// pay chain). Used as the cache key so distinct wallets don't collide.
    pub account_id: String,
}

// Never print the JWT: it is a live credential.
impl std::fmt::Debug for GatewaySession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewaySession")
            .field("token", &"[redacted]")
            .field("exp_unix", &self.exp_unix)
            .field("account_id", &self.account_id)
            .finish()
    }
}

impl GatewaySession {
    /// Whether the session is still valid `margin_secs` before its expiry.
    /// A caller re-authenticates when this is false.
    pub fn is_fresh(&self, margin_secs: i64) -> bool {
        now_unix() as i64 + margin_secs < self.exp_unix
    }
}

/// The gateway `/auth` response.
#[derive(Deserialize)]
struct AuthResponse {
    token: String,
    #[serde(rename = "expiresAt")]
    expires_at: String,
    #[serde(rename = "accountId")]
    account_id: String,
}

/// The gateway `/credits` response.
#[derive(Deserialize)]
struct CreditsResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    credits: u64,
}

/// The current credit balance for an account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditBalance {
    pub account_id: String,
    pub credits: u64,
}

// The exact SIWX statement the gateway requires, verbatim — the /auth endpoint
// rejects any other text as `invalid_statement`.
const SIWX_STATEMENT: &str =
    "I accept the Quicknode Terms of Service: https://www.quicknode.com/terms";

/// Authenticates against the x402 gateway with a SIWE (EIP-4361) message and
/// returns a cached [`GatewaySession`]. Free — no funds move — so a caller may
/// (re)auth transparently on a missing/expired session without user consent.
///
/// EVM signers only (SIWE). An SVM signer errors — SIWS is a separate
/// construction.
pub async fn authenticate(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
) -> Result<GatewaySession, SdkError> {
    let base = super::PaymentScheme::X402.host_base(payment.base_url_override.as_deref());
    // The SIWE `address` line must be EIP-55 checksummed: the gateway recovers
    // the signer and compares it case-sensitively to the address in the message.
    // The signer derives a lowercase address, so checksum it here.
    let address = to_checksum_address(&payment.signer.address()?);
    // EIP-4361's `Chain ID` field is the decimal EIP-155 chain id, NOT the
    // CAIP-2 string: the gateway matches it numerically (a CAIP-2 value like
    // "eip155:84532" is rejected as unsupported_chain). Derive it from the
    // eip155 pay_network prefix.
    let chain_id = eip155_chain_id(&payment.pay_network)?;

    // Build and sign the SIWE message. The domain/uri and statement are fixed
    // by the gateway; the nonce is a fresh random hex (≥8 chars) and issuedAt
    // is the current time (the gateway enforces a 5-minute freshness window).
    let host = host_only(base);
    let nonce = hex::encode(&random_nonce()[..8]);
    let issued_at = rfc3339_now();
    let message = siwe_message(
        &host,
        &address,
        chain_id,
        &nonce,
        &issued_at,
        SIWX_STATEMENT,
    );
    let signature = payment.signer.sign_siwe(&message)?;

    let url = format!("{}/auth", base.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .json(&serde_json::json!({
            "type": "siwx",
            "message": message,
            "signature": signature,
        }))
        .send()
        .await
        .map_err(SdkError::Http)?;

    let status = resp.status();
    let body = resp.text().await.map_err(SdkError::Http)?;
    if !status.is_success() {
        return Err(SdkError::Api { status, body });
    }
    let parsed: AuthResponse =
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
    let exp_unix = parse_rfc3339_to_unix(&parsed.expires_at)?;
    Ok(GatewaySession {
        token: parsed.token,
        exp_unix,
        account_id: parsed.account_id,
    })
}

/// Makes one drawdown JSON-RPC call against `/:query_network` with the session
/// JWT as a Bearer token, drawing 1 credit on success. Returns the raw
/// JSON-RPC envelope text for the caller to parse.
///
/// Never retries on its own: the caller decides, and a paid lane never
/// blind-retries. A 401/403 surfaces as [`SdkError::Api`] so the caller can
/// map `token_expired` → re-auth and `monthly_limit_reached` → an actionable
/// error.
pub async fn drawdown_call(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    session: &GatewaySession,
    query_network: &str,
    body: &Value,
) -> Result<String, SdkError> {
    let base = super::PaymentScheme::X402.host_base(payment.base_url_override.as_deref());
    let url = format!("{}/{}", base.trim_end_matches('/'), query_network);
    let resp = client
        .post(&url)
        .bearer_auth(&session.token)
        .json(body)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let status = resp.status();
    let text = resp.text().await.map_err(SdkError::Http)?;
    if !status.is_success() {
        return Err(SdkError::Api { status, body: text });
    }
    Ok(text)
}

/// Fetches the account's current credit balance (GET `/credits`, Bearer JWT).
pub async fn credits(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    session: &GatewaySession,
) -> Result<CreditBalance, SdkError> {
    let base = super::PaymentScheme::X402.host_base(payment.base_url_override.as_deref());
    let url = format!("{}/credits", base.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .bearer_auth(&session.token)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let status = resp.status();
    let body = resp.text().await.map_err(SdkError::Http)?;
    if !status.is_success() {
        return Err(SdkError::Api { status, body });
    }
    let parsed: CreditsResponse =
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
    Ok(CreditBalance {
        account_id: parsed.account_id,
        credits: parsed.credits,
    })
}

/// The faucet drip result: the on-chain funding transaction. The gateway's
/// `/drip` returns the settlement tx, not a credit balance — call [`credits`]
/// afterwards to read the updated balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DripReceipt {
    pub account_id: String,
    /// The faucet funding transaction hash.
    pub transaction_hash: String,
}

/// Requests testnet tokens from the faucet (POST `/drip`, Bearer JWT). The
/// gateway allows this once per account on Base Sepolia and returns the funding
/// transaction (NOT a balance).
pub async fn drip(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    session: &GatewaySession,
) -> Result<DripReceipt, SdkError> {
    let base = super::PaymentScheme::X402.host_base(payment.base_url_override.as_deref());
    let url = format!("{}/drip", base.trim_end_matches('/'));
    let resp = client
        .post(&url)
        .bearer_auth(&session.token)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let status = resp.status();
    let body = resp.text().await.map_err(SdkError::Http)?;
    if !status.is_success() {
        return Err(SdkError::Api { status, body });
    }
    #[derive(Deserialize)]
    struct DripBody {
        #[serde(rename = "accountId")]
        account_id: String,
        #[serde(rename = "transactionHash")]
        transaction_hash: String,
    }
    let parsed: DripBody =
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
    Ok(DripReceipt {
        account_id: parsed.account_id,
        transaction_hash: parsed.transaction_hash,
    })
}

/// Buys a block of credits: POST `/credits` with the Bearer JWT, settle the
/// `402` credit offer with the SAME x402 signer construction as the
/// per-request lane, and resend exactly once. Returns the balance after the
/// purchase settles.
///
/// The amount is chosen by the gateway's offer (bounded by `payment.max_amount`
/// like every signed payment); the caller does not name it. A second 402 is a
/// terminal [`SdkError::PaymentRejected`]; a lost response after the paid
/// resend is [`SdkError::PaymentIndeterminate`] — never blind-retry.
pub async fn buy_credits(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    session: &GatewaySession,
    query_network: &str,
) -> Result<CreditBalance, SdkError> {
    use crate::errors::HttpKind;

    // Credits are purchased by settling the credit-drawdown offer on a
    // network-scoped RPC request (there is no dedicated /credits POST): the
    // gateway 402s a keyed request with an `accepts` menu, and the highest-tier
    // offer is the credit block. The 200 body is the RPC result (credits are
    // funded as a side effect), so the new balance is read via GET /credits.
    let base = super::PaymentScheme::X402.host_base(payment.base_url_override.as_deref());
    let url = format!("{}/{}", base.trim_end_matches('/'), query_network);
    let rpc_body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "eth_chainId", "params": []
    });

    // 1. Offer probe with the Bearer JWT. A non-402 means credits are already
    //    available (the RPC ran) — nothing to buy; report the current balance.
    let first = client
        .post(&url)
        .bearer_auth(&session.token)
        .json(&rpc_body)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let status = first.status();
    if status.as_u16() != 402 {
        if !status.is_success() {
            let body = first.text().await.unwrap_or_default();
            return Err(SdkError::Api { status, body });
        }
        return credits(client, payment, session).await;
    }

    // 2. Settle the credit-drawdown tier (identified by its `extra.name`, not
    //    by amount — it is typically the cheapest entry on the menu). Refuses
    //    rather than falling back to a per-request offer, which would settle a
    //    far larger amount than the caller asked for. Pre-payment failures stay
    //    PaymentUnsupported: nothing was signed.
    let challenge_body = first.text().await.map_err(SdkError::Http)?;
    let authorized = super::authorize_x402_credit(client, payment, &challenge_body).await?;
    let header = authorized
        .x402_header()
        .ok_or_else(|| SdkError::Config("credit purchase produced no x402 credential".into()))?;

    // 3. Paid resend — exactly once, same indeterminate-outcome handling as the
    //    per-request driver. A lost response here means the credit purchase may
    //    have settled, so it is indeterminate and never blind-retried.
    let paid = match client
        .post(&url)
        .bearer_auth(&session.token)
        .header("PAYMENT-SIGNATURE", header)
        .json(&rpc_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let err = SdkError::Http(e);
            return Err(match err.http_kind() {
                Some(HttpKind::Connect) => err,
                _ => SdkError::PaymentIndeterminate,
            });
        }
    };
    let paid_status = paid.status().as_u16();
    if !(200..300).contains(&paid_status) {
        let body = paid.text().await.unwrap_or_default();
        return Err(SdkError::PaymentRejected {
            status: paid_status,
            body,
        });
    }
    // Drain the (RPC-result) body so the connection completes, then read the
    // freshly-funded balance from GET /credits.
    let _ = paid.text().await;
    credits(client, payment, session).await
}

// ── SIWE message construction ────────────────────────────────────────────────

/// Build a Sign-In-With-Ethereum (EIP-4361) message. The gateway pins the
/// domain/uri to its own host and requires the ToS `statement`; `chain_id` is
/// the caller's CAIP-2 pay network. Deterministic given its inputs so the
/// message tests are byte-exact.
pub(super) fn siwe_message(
    host: &str,
    address: &str,
    chain_id: u64,
    nonce: &str,
    issued_at: &str,
    statement: &str,
) -> String {
    // EIP-4361 field order is fixed. `Version` is always 1; `Chain ID` is the
    // decimal EIP-155 chain id (the gateway matches it numerically). `URI` is
    // https://<host>.
    format!(
        "{host} wants you to sign in with your Ethereum account:\n\
         {address}\n\
         \n\
         {statement}\n\
         \n\
         URI: https://{host}\n\
         Version: 1\n\
         Chain ID: {chain_id}\n\
         Nonce: {nonce}\n\
         Issued At: {issued_at}"
    )
}

// EIP-55 mixed-case checksum of a `0x`-hex EVM address: uppercase each hex
// digit whose corresponding nibble in keccak256(lowercase-addr-without-0x) is
// >= 8. SIWE requires the checksummed form in the `address` line.
fn to_checksum_address(addr: &str) -> String {
    use sha3::{Digest, Keccak256};
    let lower = addr.strip_prefix("0x").unwrap_or(addr).to_lowercase();
    let hash = Keccak256::digest(lower.as_bytes());
    let mut out = String::with_capacity(42);
    out.push_str("0x");
    for (i, c) in lower.chars().enumerate() {
        if c.is_ascii_digit() {
            out.push(c);
        } else {
            // nibble i of the hash: high nibble for even i, low for odd.
            let nibble = if i % 2 == 0 {
                hash[i / 2] >> 4
            } else {
                hash[i / 2] & 0x0f
            };
            if nibble >= 8 {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c);
            }
        }
    }
    out
}

// Parse the decimal EIP-155 chain id from an `eip155:<n>` CAIP-2 pay network,
// for the SIWE `Chain ID` field. x402 drawdown is EVM-only; a non-eip155 (e.g.
// solana:) pay network is an unsupported config here.
fn eip155_chain_id(pay_network: &str) -> Result<u64, SdkError> {
    pay_network
        .strip_prefix("eip155:")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            SdkError::Config(format!(
                "x402 drawdown requires an eip155 pay network (e.g. eip155:84532), got {pay_network:?}"
            ))
        })
}

// Strip the scheme (and any trailing slash) from a gateway base URL, leaving
// the host[:port] the SIWE domain/uri fields use. A base_url_override for the
// wiremock harness is http://127.0.0.1:PORT, which reduces to 127.0.0.1:PORT.
fn host_only(base: &str) -> String {
    base.trim_end_matches('/')
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_string()
}

// Current time as an RFC-3339 UTC timestamp to whole seconds, e.g.
// "2026-07-17T12:00:00Z". Hand-rolled to avoid a date crate, mirroring the
// parse side in admin::parse_rfc3339_to_unix (civil-from-days, Hinnant).
fn rfc3339_now() -> String {
    let secs = now_unix() as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, min, sec) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days);
    // Millisecond precision (.000) matches the canonical EIP-4361 `Issued At`
    // the reference SIWE libraries emit; whole-second precision can trip the
    // gateway's format validation.
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.000Z")
}

// Days-since-epoch → (year, month, day), Howard Hinnant's civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use secrecy::SecretString;
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // anvil key #0 (public throwaway, never funded).
    const EVM_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const EVM_ADDR: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
    const USDC: &str = "0x036CbD53842c5426634e7929541eC2318f3dCF7e";

    fn evm_payment(base: &str) -> ResolvedPayment {
        ResolvedPayment {
            scheme: super::super::PaymentScheme::X402,
            signer: super::super::signer::Signer::Evm(SecretString::new(EVM_KEY.to_string())),
            pay_network: "eip155:84532".into(),
            asset: USDC.into(),
            max_amount: 10_000_000,
            base_url_override: Some(base.to_string()),
            svm_rpc_url: None,
        }
    }

    fn x402_credit_offer(amount: &str) -> Value {
        json!({
            "x402Version": 2,
            "accepts": [{
                "scheme": "exact",
                "network": "eip155:84532",
                "amount": amount,
                "payTo": "0x000000000000000000000000000000000000dEaD",
                "maxTimeoutSeconds": 60,
                "asset": USDC,
                "extra": { "name": "USDC", "version": "2" }
            }]
        })
    }

    #[test]
    fn siwe_message_is_byte_exact() {
        let msg = siwe_message(
            "x402.quicknode.com",
            EVM_ADDR,
            84532,
            "abc12345",
            "2026-07-17T12:00:00Z",
            SIWX_STATEMENT,
        );
        let expected = "x402.quicknode.com wants you to sign in with your Ethereum account:\n\
             0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266\n\
             \n\
             I accept the Quicknode Terms of Service: https://www.quicknode.com/terms\n\
             \n\
             URI: https://x402.quicknode.com\n\
             Version: 1\n\
             Chain ID: 84532\n\
             Nonce: abc12345\n\
             Issued At: 2026-07-17T12:00:00Z";
        assert_eq!(msg, expected);
    }

    // The byte-exact test above supplies `issued_at` directly, so it cannot
    // catch the format the gateway actually receives — that comes from
    // `rfc3339_now()`. The gateway's format validation rejects whole-second
    // precision, so assert the millisecond `.000Z` suffix at the source and in
    // the assembled message.
    #[test]
    fn issued_at_carries_millisecond_precision() {
        let iso = rfc3339_now();
        assert!(iso.ends_with(".000Z"), "issued_at was {iso}");
        assert_eq!(
            iso.len(),
            24,
            "expected YYYY-MM-DDTHH:MM:SS.000Z, got {iso}"
        );

        let msg = siwe_message(
            "x402.quicknode.com",
            EVM_ADDR,
            84532,
            "abc12345",
            &iso,
            SIWX_STATEMENT,
        );
        assert!(msg.ends_with(&format!("Issued At: {iso}")));
    }

    #[test]
    fn rfc3339_now_round_trips_through_the_parser() {
        // The timestamp we emit must parse back to (approximately) the same
        // unix time the parser reads — locks the civil-from-days math.
        let iso = rfc3339_now();
        let back = parse_rfc3339_to_unix(&iso).unwrap();
        let now = now_unix() as i64;
        assert!((now - back).abs() <= 1, "iso={iso} back={back} now={now}");
    }

    #[test]
    fn checksum_address_matches_eip55() {
        // Known-good EIP-55 checksum (anvil key #0's address), matching the
        // reference SIWE libraries' output.
        assert_eq!(
            to_checksum_address("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
        // Idempotent on already-checksummed input.
        assert_eq!(
            to_checksum_address("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
            "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
        );
    }

    #[test]
    fn host_only_strips_scheme_and_slash() {
        assert_eq!(
            host_only("https://x402.quicknode.com/"),
            "x402.quicknode.com"
        );
        assert_eq!(host_only("http://127.0.0.1:8080"), "127.0.0.1:8080");
    }

    #[test]
    fn session_freshness() {
        let s = GatewaySession {
            token: "t".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        assert!(s.is_fresh(60));
        let stale = GatewaySession {
            token: "t".into(),
            exp_unix: now_unix() as i64 + 10,
            account_id: "a".into(),
        };
        assert!(!stale.is_fresh(60));
    }

    #[test]
    fn session_debug_redacts_the_jwt() {
        let s = GatewaySession {
            token: "super-secret-jwt".into(),
            exp_unix: 0,
            account_id: "a".into(),
        };
        let rendered = format!("{s:?}");
        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains("super-secret-jwt"));
    }

    #[tokio::test]
    async fn authenticate_posts_siwx_and_caches_session() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/auth"))
            .and(body_partial_json(json!({ "type": "siwx" })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "token": "jwt-abc",
                "expiresAt": "2099-01-01T00:00:00Z",
                "accountId": "eip155:84532:0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let client = reqwest::Client::new();
        let session = authenticate(&client, &payment).await.unwrap();
        assert_eq!(session.token, "jwt-abc");
        assert!(session.account_id.contains("0xf39fd6e5"));
        assert!(session.is_fresh(60));
    }

    #[tokio::test]
    async fn authenticate_error_surfaces_as_api() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/auth"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({
                "error": "invalid_signature", "message": "bad SIWX signature"
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let client = reqwest::Client::new();
        let err = authenticate(&client, &payment).await.unwrap_err();
        assert!(matches!(err, SdkError::Api { status, .. } if status == 401));
    }

    #[tokio::test]
    async fn drawdown_call_attaches_bearer_and_returns_envelope() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/base-sepolia"))
            .and(header("authorization", "Bearer jwt-abc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 1, "result": "0x1335f9a"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_blockNumber", "params": [] });
        let client = reqwest::Client::new();
        let text = drawdown_call(&client, &payment, &session, "base-sepolia", &body)
            .await
            .unwrap();
        assert!(text.contains("0x1335f9a"));
    }

    #[tokio::test]
    async fn drawdown_call_403_monthly_limit_is_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/base-sepolia"))
            .respond_with(ResponseTemplate::new(403).set_body_json(json!({
                "error": "monthly_limit_reached", "message": "monthly credit limit reached"
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_blockNumber", "params": [] });
        let client = reqwest::Client::new();
        let err = drawdown_call(&client, &payment, &session, "base-sepolia", &body)
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::Api { status, body } if *status == 403 && body.contains("monthly_limit_reached"))
        );
    }

    #[tokio::test]
    async fn credits_reads_the_balance() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/credits"))
            .and(header("authorization", "Bearer jwt-abc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "accountId": "eip155:84532:0xabc", "credits": 1_000_095u64
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let client = reqwest::Client::new();
        let bal = credits(&client, &payment, &session).await.unwrap();
        assert_eq!(bal.credits, 1_000_095);
    }

    #[tokio::test]
    async fn drip_returns_the_funding_transaction() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/drip"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "accountId": "eip155:84532:0xabc",
                "walletAddress": "0xabc",
                "transactionHash": "0xfeed"
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let client = reqwest::Client::new();
        let receipt = drip(&client, &payment, &session).await.unwrap();
        assert_eq!(receipt.transaction_hash, "0xfeed");
        assert_eq!(receipt.account_id, "eip155:84532:0xabc");
    }

    // The live gateway's 402 menu: two per-request USDC tiers plus the
    // credit-drawdown tier, which is the CHEAPEST entry and carries the Circle
    // Gateway batched `extra` (its own verifyingContract, not the asset).
    fn gateway_menu() -> Value {
        let mut credit = x402_credit_offer("100")
            .pointer("/accepts/0")
            .cloned()
            .unwrap();
        credit["maxTimeoutSeconds"] = json!(604_900);
        credit["extra"] = json!({
            "name": "GatewayWalletBatched",
            "version": "1",
            "verifyingContract": "0x0077777d7EBA4688BDeF3E311b846F25870A19B9"
        });
        json!({
            "x402Version": 2,
            "accepts": [
                x402_credit_offer("1000000").pointer("/accepts/0").cloned().unwrap(),
                x402_credit_offer("1000").pointer("/accepts/0").cloned().unwrap(),
                credit,
            ]
        })
    }

    // The credit tier uses a signing construction the per-request lane does not
    // have. Refusing is the point: falling back to a per-request offer would
    // settle 1000000 base units when the caller asked for a 100-unit credit
    // block, and the gateway rejects the wrong-scheme signature anyway.
    #[tokio::test]
    async fn buy_credits_refuses_the_batched_scheme_and_settles_nothing() {
        let server = MockServer::start().await;
        // Exactly one POST: the offer probe. Nothing is ever signed or resent.
        Mock::given(method("POST"))
            .and(path("/base-sepolia"))
            .respond_with(ResponseTemplate::new(402).set_body_json(gateway_menu()))
            .expect(1)
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let client = reqwest::Client::new();
        let err = buy_credits(&client, &payment, &session, "base-sepolia")
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::PaymentUnsupported { offered }
                if offered.contains("GatewayWalletBatched")),
            "unexpected error: {err:?}"
        );
    }

    // A menu with no credit tier at all: still a refusal, and still nothing
    // signed — never a silent fallback onto a per-request offer.
    #[tokio::test]
    async fn buy_credits_without_a_credit_offer_settles_nothing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/base-sepolia"))
            .respond_with(ResponseTemplate::new(402).set_body_json(x402_credit_offer("1000")))
            .expect(1)
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let client = reqwest::Client::new();
        let err = buy_credits(&client, &payment, &session, "base-sepolia")
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::PaymentUnsupported { offered }
                if offered.contains("no credit-drawdown offer")),
            "unexpected error: {err:?}"
        );
    }

    // A non-402 first response means credits are already available: the probe
    // RPC ran, so report the balance without buying anything.
    #[tokio::test]
    async fn buy_credits_with_existing_credits_reads_the_balance() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/base-sepolia"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 1, "result": "0x1"
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/credits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "accountId": "eip155:84532:0xabc", "credits": 42u64
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri());
        let session = GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: now_unix() as i64 + 3600,
            account_id: "a".into(),
        };
        let client = reqwest::Client::new();
        let bal = buy_credits(&client, &payment, &session, "base-sepolia")
            .await
            .unwrap();
        assert_eq!(bal.credits, 42);
    }
}
