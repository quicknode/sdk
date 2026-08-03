//! MPP session (payment-channel) lane for `rpc.call`.
//!
//! The counterpart to the per-request MPP charge in the parent module: instead
//! of signing a fresh Tempo transaction per request, the caller opens a payment
//! channel by depositing into the escrow contract the gateway advertises in its
//! session challenge (`methodDetails.escrowContract`), then authorizes spend
//! with cumulative EIP-712 vouchers (`Authorization: Payment`) — one
//! `ecrecover` server-side, no on-chain tx per call. The gateway settles the
//! channel on-chain in batches on its own schedule; the client cooperatively
//! closes to settle + refund the unused deposit.
//!
//! Wire protocol (matches the `mppx` reference client's contract-backed
//! session, `tempo/legacy/session`):
//! - Endpoints under `{mpp}/session/:network`. The gateway requires the slug to
//!   name a network it serves, but selects the challenge by the caller's pay
//!   chain, so the value only matters for `voucher_call` (which routes an RPC
//!   method). The lifecycle verbs pin `SESSION_ROUTE_NETWORK`.
//! - Channel lifecycle credentials are a discriminated union on `action`
//!   (`open`/`topUp`/`voucher`/`close`), each a `Payment <base64url JSON>`
//!   credential of `{challenge, payload, source}`.
//! - `open`/`topUp` carry a fee-sponsored Tempo tx with two calls — a token
//!   `approve(escrow, amount)` plus the escrow `open`/`topUp` call;
//!   `voucher`/`close` are pure EIP-712 voucher signatures.
//! - The channelId is derived locally (keccak over the channel parameters, as
//!   the escrow contract derives it) and the gateway re-derives it from the
//!   open calldata.
//! - The gateway exposes no read-only channel endpoint, and it prices every
//!   `/session/:network` POST as a chargeable request: the available balance is
//!   the NEW spend a voucher authorizes, so re-presenting the current
//!   high-water voucher is always refused with `insufficient-balance`. `status`
//!   therefore advances the voucher by one request unit like any session call,
//!   and reads the `Payment-Receipt` header.

use serde_json::Value;

use crate::errors::{HttpKind, SdkError};

use super::signer::tempo::{EscrowAction, TempoEscrowRequest};
use super::{now_unix, parse_iso_unix, random_nonce, PaymentScheme, ResolvedPayment};

// The path segment the channel-lifecycle requests route on. The gateway
// requires `/session/:network` to name a network it serves — an unknown slug
// 404s — but for open/topUp/close/voucher-status the value has no effect: the
// challenge it answers with is selected by the caller's pay chain, so every
// supported slug yields the same escrow, currency, and price. The lifecycle
// operates on the channel (pay chain + asset), never on a queried chain, so it
// pins one slug rather than making callers supply an arbitrary one. Only
// `voucher_call` takes a real query network, because it routes an RPC method.
const SESSION_ROUTE_NETWORK: &str = "tempo-testnet";

// validBefore for a fee-sponsored escrow tx = min(now+25s, challenge expiry) —
// the same TIP-1009 expiring-nonce envelope the charge lane uses. Clamping to
// the challenge matters because the gateway rejects an authorization that
// outlives the challenge it answers; an unparseable expiry is an error rather
// than an unbounded window.
fn session_valid_before(challenge: &SessionChallenge) -> Result<u64, SdkError> {
    let expiry = parse_iso_unix(&challenge.expires).ok_or_else(|| {
        SdkError::Config(format!(
            "MPP session challenge has an unparseable `expires` value: {}",
            challenge.expires
        ))
    })?;
    Ok((now_unix() + 25).min(expiry))
}

/// Local state for an open MPP payment channel. The CLI persists this between
/// runs (like the drawdown session JWT); the gateway has no read-only channel
/// endpoint, so a lost local record means opening a new channel.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ChannelState {
    /// Channel id (`0x`-hex bytes32), derived from the channel parameters.
    pub channel_id: String,
    /// The escrow token (TIP-20 currency) the channel is denominated in.
    pub token: String,
    /// The channel payee (settlement recipient), from the open challenge.
    pub payee: String,
    /// Payer entropy used to derive the channel (`0x`-hex bytes32).
    pub salt: String,
    /// Voucher signer (the payer; the SDK delegates to no separate signer).
    pub authorized_signer: String,
    /// The escrow contract the channel lives in, from the open challenge.
    pub escrow_contract: String,
    /// Total deposited into the channel so far, in token base units.
    pub deposit: u128,
    /// Highest cumulative amount authorized by a voucher so far.
    pub cumulative_spent: u128,
    /// The gateway's per-call price (from the open challenge), so the caller can
    /// advance `cumulative_spent` by one unit per session call.
    pub per_call: u128,
    /// EIP-155 chain id the channel lives on.
    pub chain_id: u64,
}

// ── Session challenge parse ──────────────────────────────────────────────────

// One MPP `Payment` challenge parsed from a WWW-Authenticate header, plus its
// decoded request body. Mirrors the charge-side parser in the parent module but
// keeps the raw pieces the credential must echo back verbatim.
struct SessionChallenge {
    id: String,
    realm: String,
    intent: String,
    description: String,
    expires: String,
    /// The original base64url request string, re-embedded in the credential.
    request_b64: String,
    /// The decoded request JSON (currency, recipient, amount, chainId, …).
    request: Value,
}

// ── Channel lifecycle ────────────────────────────────────────────────────────

/// Opens a payment channel: deposit `deposit` base units into the escrow, sign
/// the opening voucher for `initial_cumulative`, and POST the `open` credential
/// to `{mpp}/session/:network`. Returns the new [`ChannelState`].
///
/// `deposit` moves real funds on-chain; the caller gates this. Single-attempt.
pub async fn open(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    deposit: u128,
) -> Result<ChannelState, SdkError> {
    if deposit > payment.max_amount {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "requested channel deposit {deposit} exceeds max_amount {}",
                payment.max_amount
            ),
        });
    }
    let challenge = probe_session_challenge(client, payment).await?;
    let chain_id = challenge_chain_id(&challenge)?;
    let token = require_str(&challenge.request, "currency")?;
    let payee = require_str(&challenge.request, "recipient")?;
    let escrow = challenge_escrow_contract(&challenge)?;
    let payer = payment.signer.address()?;

    // Sign the escrow `open` tx (approve + open) → channelId. salt is fresh
    // payer entropy; the payer is its own voucher signer, and the gateway
    // re-derives the channelId from these exact calldata parameters.
    let salt = format!("0x{}", hex::encode(random_nonce()));
    let signed = payment.signer.sign_escrow_tx(&TempoEscrowRequest {
        chain_id,
        valid_before: session_valid_before(&challenge)?,
        escrow_contract: escrow.clone(),
        action: EscrowAction::Open {
            payee: payee.clone(),
            token: token.clone(),
            deposit,
            salt: salt.clone(),
            authorized_signer: payer.clone(),
        },
    })?;
    let channel_id = signed
        .channel_id
        .map(|c| format!("0x{}", hex::encode(c)))
        .ok_or_else(|| SdkError::Config("open did not derive a channelId".into()))?;

    // The opening voucher authorizes the first unit of spend (the per-call
    // amount from the challenge). cumulativeAmount starts at that amount.
    let per_unit = require_amount(&challenge.request)?;
    let voucher_sig =
        payment
            .signer
            .sign_session_voucher(&channel_id, per_unit, chain_id, &escrow)?;

    let payload = serde_json::json!({
        "action": "open",
        "type": "transaction",
        "channelId": channel_id,
        "transaction": format!("0x{}", hex::encode(&signed.transaction)),
        "signature": voucher_sig,
        "authorizedSigner": payer,
        "cumulativeAmount": per_unit.to_string(),
    });
    post_session_credential(client, payment, &challenge, &payer, payload).await?;

    Ok(ChannelState {
        channel_id,
        token,
        payee,
        salt,
        authorized_signer: payer,
        escrow_contract: escrow,
        deposit,
        cumulative_spent: per_unit,
        per_call: per_unit,
        chain_id,
    })
}

/// Adds `additional_deposit` base units to an open channel: sign the escrow
/// `topUp` tx and POST the `topUp` credential. Moves real funds; single-attempt.
pub async fn top_up(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    channel: &ChannelState,
    additional_deposit: u128,
) -> Result<ChannelState, SdkError> {
    let payer = payment.signer.address()?;
    let challenge = probe_session_challenge(client, payment).await?;

    let signed = payment.signer.sign_escrow_tx(&TempoEscrowRequest {
        chain_id: channel.chain_id,
        valid_before: session_valid_before(&challenge)?,
        escrow_contract: channel.escrow_contract.clone(),
        action: EscrowAction::TopUp {
            channel_id: channel.channel_id.clone(),
            token: channel.token.clone(),
            additional_deposit,
        },
    })?;
    let payload = serde_json::json!({
        "action": "topUp",
        "type": "transaction",
        "channelId": channel.channel_id,
        "transaction": format!("0x{}", hex::encode(&signed.transaction)),
        "additionalDeposit": additional_deposit.to_string(),
    });
    post_session_credential(client, payment, &challenge, &payer, payload).await?;

    let mut updated = channel.clone();
    updated.deposit = channel.deposit.saturating_add(additional_deposit);
    Ok(updated)
}

/// Cooperatively closes a channel at its final cumulative spend: sign the close
/// voucher and POST the `close` credential. The gateway settles the final
/// amount on-chain and refunds the unused deposit. Single-attempt.
pub async fn close(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    channel: &ChannelState,
) -> Result<(), SdkError> {
    let payer = payment.signer.address()?;
    let challenge = probe_session_challenge(client, payment).await?;
    let signature = payment.signer.sign_session_voucher(
        &channel.channel_id,
        channel.cumulative_spent,
        channel.chain_id,
        &channel.escrow_contract,
    )?;
    let payload = serde_json::json!({
        "action": "close",
        "channelId": channel.channel_id,
        "cumulativeAmount": channel.cumulative_spent.to_string(),
        "signature": signature,
    });
    post_session_credential(client, payment, &challenge, &payer, payload).await?;
    Ok(())
}

/// The gateway's view of a channel: the accepted cumulative high-water mark
/// and the amount it counts as spent. (The deposit is tracked locally; the
/// gateway exposes no read-only channel endpoint.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelStatus {
    pub channel_id: String,
    pub accepted_cumulative: u128,
    pub spent: u128,
}

/// Fetches the gateway's view of the channel and reads the `Payment-Receipt`
/// header.
///
/// **This costs one request unit.** The gateway prices every `/session/:network`
/// POST as a chargeable request and computes the available balance as the *new*
/// spend a voucher authorizes, so re-presenting the current high-water voucher
/// authorizes zero and is always refused with `insufficient-balance` — however
/// much deposit remains. The voucher therefore advances by `per_call`, exactly
/// like a session RPC call, and the caller must persist the new
/// `cumulative_spent` on success.
///
/// Returns [`SdkError::PaymentUnsupported`] before any network I/O when the
/// channel has no room left for the probe.
pub async fn status(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    channel: &ChannelState,
) -> Result<ChannelStatus, SdkError> {
    let probe_cumulative = channel.cumulative_spent.saturating_add(channel.per_call);
    if probe_cumulative > channel.deposit {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "the channel has no room for a status probe (it costs {} of the {} remaining); \
                 top up first",
                channel.per_call,
                channel.deposit.saturating_sub(channel.cumulative_spent),
            ),
        });
    }
    let payer = payment.signer.address()?;
    let signature = payment.signer.sign_session_voucher(
        &channel.channel_id,
        probe_cumulative,
        channel.chain_id,
        &channel.escrow_contract,
    )?;
    let challenge = probe_session_challenge(client, payment).await?;
    let payload = serde_json::json!({
        "action": "voucher",
        "channelId": channel.channel_id,
        "cumulativeAmount": probe_cumulative.to_string(),
        "signature": signature,
    });
    let resp = post_session_credential(client, payment, &challenge, &payer, payload).await?;

    let receipt_b64 = resp
        .headers()
        .get("payment-receipt")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or_else(|| {
            SdkError::Config("the gateway's response carried no Payment-Receipt header".into())
        })?;
    let receipt = super::decode_b64url_json(&receipt_b64)
        .map_err(|_| SdkError::Config("the gateway's Payment-Receipt did not decode".into()))?;
    let accepted = require_str(&receipt, "acceptedCumulative")?;
    let spent = require_str(&receipt, "spent")?;
    Ok(ChannelStatus {
        channel_id: channel.channel_id.clone(),
        accepted_cumulative: parse_u128(&accepted)?,
        spent: parse_u128(&spent)?,
    })
}

/// Makes one session-lane JSON-RPC call, authorizing it with a cumulative
/// voucher for `new_cumulative` (the running total after this call). Returns the
/// raw JSON-RPC envelope text. Single-attempt: a paid lane never blind-retries.
///
/// The caller advances `cumulative_spent` in the persisted channel state after
/// a success; a `NeedVoucher` / insufficient-deposit refusal surfaces as an
/// `Api` error the caller maps to a top-up hint.
pub async fn voucher_call(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    query_network: &str,
    channel: &ChannelState,
    new_cumulative: u128,
    body: &Value,
) -> Result<String, SdkError> {
    if new_cumulative > channel.deposit {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "voucher cumulative {new_cumulative} exceeds channel deposit {}; top up first",
                channel.deposit
            ),
        });
    }
    let payer = payment.signer.address()?;
    let signature = payment.signer.sign_session_voucher(
        &channel.channel_id,
        new_cumulative,
        channel.chain_id,
        &channel.escrow_contract,
    )?;
    // A voucher credential needs the challenge it answers; the gateway echoes it
    // on the 402. Probe once (free) to obtain the current session challenge. The
    // probe uses the pinned lifecycle route, not `query_network`: the challenge
    // is selected by the pay chain and is identical on every served slug, while
    // the paid POST below must go to the network the caller is querying.
    let challenge = probe_session_challenge(client, payment).await?;
    let payload = serde_json::json!({
        "action": "voucher",
        "channelId": channel.channel_id,
        "cumulativeAmount": new_cumulative.to_string(),
        "signature": signature,
    });
    let credential = build_credential(&challenge, &payer, channel.chain_id, &payload)?;

    let base = session_base(payment, query_network);
    let paid = match client
        .post(&base)
        .header("Authorization", format!("Payment {credential}"))
        .json(body)
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
    let paid_status = paid.status();
    let text = paid.text().await.map_err(SdkError::Http)?;
    if !paid_status.is_success() {
        return Err(SdkError::Api {
            status: paid_status,
            body: text,
        });
    }
    Ok(text)
}

// ── HTTP + credential helpers ────────────────────────────────────────────────

fn session_base(payment: &ResolvedPayment, query_network: &str) -> String {
    let base = PaymentScheme::MppCharge.host_base(payment.base_url_override.as_deref());
    format!("{}/session/{}", base.trim_end_matches('/'), query_network)
}

// Probe the session endpoint keyless to obtain the current 402 session
// challenge (its WWW-Authenticate carries the tempo/session offer). Pre-payment:
// a non-402 or a missing header is "no usable session offer", never a Decode.
async fn probe_session_challenge(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
) -> Result<SessionChallenge, SdkError> {
    let base = session_base(payment, SESSION_ROUTE_NETWORK);
    let resp = client
        .post(&base)
        .json(&serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_chainId", "params": [] }))
        .send()
        .await
        .map_err(SdkError::Http)?;
    // A 404 means the pinned route slug is no longer one the gateway serves —
    // an SDK-side fix, not a payment problem. Name it so the cause is obvious
    // rather than reading as an outage.
    if resp.status().as_u16() == 404 {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "the gateway does not serve the channel-lifecycle route \
                 (/session/{SESSION_ROUTE_NETWORK} returned 404); the SDK's pinned \
                 route network needs updating to one the gateway lists"
            ),
        });
    }
    if resp.status().as_u16() != 402 {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "the session endpoint did not return a 402 challenge (status {})",
                resp.status().as_u16()
            ),
        });
    }
    let header = resp
        .headers()
        .get("www-authenticate")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .ok_or_else(|| SdkError::PaymentUnsupported {
            offered: "session 402 without a WWW-Authenticate header".into(),
        })?;
    parse_session_challenge(
        &header,
        super::caip2_or_bare_chain_id(&payment.pay_network)?,
    )
}

// Parse the tempo/session challenge for `want_chain_id` from the
// WWW-Authenticate header.
//
// The gateway offers SEVERAL session challenges on one 402 — different chains
// (Tempo testnet and mainnet) and different currencies, each with its own
// escrow contract. Taking the first would depend on the gateway's ordering and
// could open a channel on mainnet for a testnet request, so the offer is
// matched on `methodDetails.chainId` against the caller's resolved pay network.
fn parse_session_challenge(header: &str, want_chain_id: u64) -> Result<SessionChallenge, SdkError> {
    let mut offered: Vec<String> = Vec::new();
    for part in split_payment_challenges(header) {
        let get = |k: &str| extract_quoted(&part, k).unwrap_or_default();
        if get("method") != "tempo" || get("intent") != "session" {
            continue;
        }
        let request_b64 = get("request");
        let request =
            super::decode_b64url_json(&request_b64).map_err(|_| SdkError::PaymentUnsupported {
                offered: "session challenge has an undecodable request".into(),
            })?;
        let challenge = SessionChallenge {
            id: get("id"),
            realm: get("realm"),
            intent: "session".into(),
            description: get("description"),
            expires: get("expires"),
            request_b64,
            request,
        };
        match challenge_chain_id(&challenge) {
            Ok(chain_id) if chain_id == want_chain_id => return Ok(challenge),
            Ok(chain_id) => offered.push(format!("eip155:{chain_id}")),
            Err(_) => offered.push("a challenge with no chainId".into()),
        }
    }
    Err(SdkError::PaymentUnsupported {
        offered: if offered.is_empty() {
            "no tempo/session challenge offered".into()
        } else {
            format!(
                "no tempo/session challenge for eip155:{want_chain_id} (offered: {})",
                offered.join(", ")
            )
        },
    })
}

// Build the `Payment <base64url JSON>` credential: {challenge, payload, source}
// with the challenge's original request echoed verbatim (matches mppx's
// Credential.serialize wire shape). `source` is the CAIP-10 did:pkh of the
// payer on the channel's chain.
fn build_credential(
    challenge: &SessionChallenge,
    payer: &str,
    chain_id: u64,
    payload: &Value,
) -> Result<String, SdkError> {
    let credential = serde_json::json!({
        "challenge": {
            "id": challenge.id,
            "realm": challenge.realm,
            "method": "tempo",
            "intent": challenge.intent,
            "description": challenge.description,
            "expires": challenge.expires,
            "request": challenge.request_b64,
        },
        "payload": payload,
        "source": format!("did:pkh:eip155:{chain_id}:{payer}"),
    });
    // Never fall back to an empty credential: sending zero bytes turns a local
    // serialization bug into an opaque gateway rejection.
    Ok(super::base64_url_nopad(
        serde_json::to_vec(&credential).map_err(|e| {
            SdkError::Config(format!(
                "could not serialize the MPP session credential: {e}"
            ))
        })?,
    ))
}

// POST a channel-management credential to the session endpoint and require a
// 2xx, returning the response (its `Payment-Receipt` header carries the
// gateway's channel view). Management POSTs settle nothing off the caller's
// per-call amount (they commit deposits / close), so a non-2xx is a plain Api
// refusal.
async fn post_session_credential(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    challenge: &SessionChallenge,
    payer: &str,
    payload: Value,
) -> Result<reqwest::Response, SdkError> {
    let chain_id = challenge_chain_id(challenge)?;
    let credential = build_credential(challenge, payer, chain_id, &payload)?;
    let base = session_base(payment, SESSION_ROUTE_NETWORK);
    let resp = client
        .post(&base)
        .header("Authorization", format!("Payment {credential}"))
        .json(&serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_chainId", "params": [] }))
        .send()
        .await
        .map_err(SdkError::Http)?;
    let http_status = resp.status();
    if !http_status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(SdkError::Api {
            status: http_status,
            body,
        });
    }
    Ok(resp)
}

// ── small parse helpers ──────────────────────────────────────────────────────

fn require_str(request: &Value, key: &str) -> Result<String, SdkError> {
    request
        .get(key)
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| SdkError::Config(format!("session challenge missing {key}")))
}

fn require_amount(request: &Value) -> Result<u128, SdkError> {
    let s = require_str(request, "amount")?;
    parse_u128(&s)
}

fn challenge_chain_id(challenge: &SessionChallenge) -> Result<u64, SdkError> {
    challenge
        .request
        .pointer("/methodDetails/chainId")
        .and_then(Value::as_u64)
        .or_else(|| challenge.request.get("chainId").and_then(Value::as_u64))
        .ok_or_else(|| SdkError::Config("session challenge missing chainId".into()))
}

// The escrow contract the gateway expects deposits in. Its absence means the
// gateway is not offering a contract-backed session — a protocol mismatch, not
// a malformed response, so it maps to PaymentUnsupported.
fn challenge_escrow_contract(challenge: &SessionChallenge) -> Result<String, SdkError> {
    challenge
        .request
        .pointer("/methodDetails/escrowContract")
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| SdkError::PaymentUnsupported {
            offered: "the session challenge named no escrowContract".into(),
        })
}

fn parse_u128(s: &str) -> Result<u128, SdkError> {
    s.parse::<u128>()
        .map_err(|_| SdkError::Config(format!("expected an integer base-unit amount, got {s:?}")))
}

// Reuse the parent module's WWW-Authenticate splitters via re-export.
use super::{extract_quoted, split_payment_challenges};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use secrecy::SecretString;
    use wiremock::matchers::{header_exists, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const EVM_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const EVM_ADDR_LOWER: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";

    fn tempo_payment(base: &str) -> ResolvedPayment {
        ResolvedPayment {
            scheme: PaymentScheme::MppCharge,
            signer: super::super::signer::Signer::Tempo(SecretString::new(EVM_KEY.to_string())),
            pay_network: "eip155:42431".into(),
            asset: "0x20c0000000000000000000000000000000000000".into(),
            max_amount: 1_000_000,
            base_url_override: Some(base.to_string()),
            svm_rpc_url: None,
        }
    }

    fn sample_channel() -> ChannelState {
        ChannelState {
            channel_id: format!("0x{}", "11".repeat(32)),
            token: "0x20c0000000000000000000000000000000000000".into(),
            payee: "0xfd24114c3981aba78ae2441991b1bdb89329c556".into(),
            salt: format!("0x{}", "22".repeat(32)),
            authorized_signer: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".into(),
            escrow_contract: "0x33b901018174DDabE4841042ab76ba85D4e24f25".into(),
            deposit: 100_000,
            cumulative_spent: 500,
            per_call: 500,
            chain_id: 42431,
        }
    }

    // One `Payment` challenge entry, as it appears in a WWW-Authenticate header.
    fn session_offer(id: &str, chain_id: u64, escrow: &str) -> String {
        let request = super::super::base64_url_nopad(
            serde_json::to_vec(&serde_json::json!({
                "amount": "10",
                "currency": "0x20c0000000000000000000000000000000000000",
                "recipient": "0xfd24114c3981aba78ae2441991b1bdb89329c556",
                "methodDetails": { "chainId": chain_id, "escrowContract": escrow }
            }))
            .unwrap(),
        );
        format!(
            "Payment id=\"{id}\", realm=\"mpp.quicknode.com\", method=\"tempo\", \
             intent=\"session\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", \
             request=\"{request}\""
        )
    }

    // The live gateway offers several session challenges on one 402 — testnet
    // and mainnet, each with its own escrow. The parser must match on chainId,
    // not take the first: picking by position would open a mainnet channel for
    // a testnet request if the gateway ever reorders the menu.
    #[test]
    fn parse_session_challenge_selects_the_offer_for_the_pay_chain() {
        let header = format!(
            "Payment id=\"c0\", realm=\"mpp.quicknode.com\", method=\"tempo\", \
             intent=\"charge\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", \
             request=\"ey000\", {}, {}",
            session_offer(
                "mainnet",
                4217,
                "0x33b901018174DDabE4841042ab76ba85D4e24f25"
            ),
            session_offer(
                "testnet",
                42431,
                "0xe1c4d3dce17bc111181ddf716f75bae49e61a336"
            ),
        );

        // Testnet is offered SECOND: a first-match parser would pick mainnet.
        let parsed = parse_session_challenge(&header, 42431).unwrap();
        assert_eq!(parsed.id, "testnet");
        assert_eq!(challenge_chain_id(&parsed).unwrap(), 42431);
        assert_eq!(
            challenge_escrow_contract(&parsed).unwrap(),
            "0xe1c4d3dce17bc111181ddf716f75bae49e61a336"
        );

        // The same header resolves mainnet when that is what was asked for.
        let parsed = parse_session_challenge(&header, 4217).unwrap();
        assert_eq!(parsed.id, "mainnet");
        assert_eq!(
            challenge_escrow_contract(&parsed).unwrap(),
            "0x33b901018174DDabE4841042ab76ba85D4e24f25"
        );
    }

    #[test]
    fn session_challenge_for_an_unoffered_chain_names_what_was_offered() {
        let header = session_offer(
            "mainnet",
            4217,
            "0x33b901018174DDabE4841042ab76ba85D4e24f25",
        );
        let Err(err) = parse_session_challenge(&header, 42431) else {
            panic!("a challenge for an unoffered chain must not resolve");
        };
        assert!(
            matches!(&err, SdkError::PaymentUnsupported { offered }
                if offered.contains("eip155:42431") && offered.contains("eip155:4217")),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn challenge_without_escrow_contract_is_unsupported() {
        let request = super::super::base64_url_nopad(
            serde_json::to_vec(&serde_json::json!({
                "amount": "500",
                "currency": "0x20c0000000000000000000000000000000000000",
                "recipient": "0xfd24114c3981aba78ae2441991b1bdb89329c556",
                "methodDetails": { "chainId": 42431 }
            }))
            .unwrap(),
        );
        let header = format!(
            "Payment id=\"c1\", realm=\"mpp.quicknode.com\", method=\"tempo\", intent=\"session\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", request=\"{request}\""
        );
        let parsed = parse_session_challenge(&header, 42431).unwrap();
        let err = challenge_escrow_contract(&parsed).unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("escrowContract"))
        );
    }

    #[tokio::test]
    async fn voucher_over_deposit_is_rejected_before_signing() {
        let payment = tempo_payment("http://127.0.0.1:1");
        let ch = sample_channel();
        let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_blockNumber" });
        // new_cumulative above the deposit: must fail before any network I/O.
        let err = voucher_call(
            &reqwest::Client::new(),
            &payment,
            "tempo-testnet",
            &ch,
            ch.deposit + 1,
            &body,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("exceeds channel deposit"))
        );
    }

    // A near-term challenge expiry must cap validBefore: the gateway's fee
    // sponsor refuses an escrow authorization that outlives the challenge it
    // answers, so `open`/`top_up` cannot just use now+25s unconditionally.
    #[test]
    fn valid_before_is_clamped_to_a_near_term_challenge_expiry() {
        // A fixed past timestamp is always nearer than now+25s, so the clamp
        // must return exactly it. Using a literal keeps the test independent of
        // any unix→ISO formatting helper.
        const EXPIRES: &str = "2026-07-17T12:00:00Z";
        let mut parsed = parse_session_challenge(
            &session_offer("c1", 42431, "0x33b901018174DDabE4841042ab76ba85D4e24f25"),
            42431,
        )
        .unwrap();
        parsed.expires = EXPIRES.into();
        assert_eq!(
            session_valid_before(&parsed).unwrap(),
            parse_iso_unix(EXPIRES).unwrap()
        );
    }

    // A far-future expiry leaves the now+25s envelope in force.
    #[test]
    fn valid_before_uses_the_25s_envelope_when_the_challenge_outlives_it() {
        let parsed = parse_session_challenge(
            &session_offer("c1", 42431, "0x33b901018174DDabE4841042ab76ba85D4e24f25"),
            42431,
        )
        .unwrap();
        assert_eq!(session_valid_before(&parsed).unwrap(), now_unix() + 25);
    }

    // ── wiremock lifecycle tests ─────────────────────────────────────────────
    //
    // Every session operation is two POSTs to the same `/session/:network` URL:
    // an unauthenticated probe the gateway answers with a 402 + WWW-Authenticate
    // menu, then the credential POST. Both mocks therefore match the same method
    // and path and are told apart by the Authorization header alone — the probe
    // requires its absence, the credential its presence. Without the negative
    // matcher the probe mock (registered first) also answers the credential POST
    // and every lifecycle call fails with a bare 402.

    const ESCROW: &str = "0x33b901018174DDabE4841042ab76ba85D4e24f25";

    // The 402 challenge menu the probe receives.
    fn probe_mock(chain_id: u64) -> Mock {
        Mock::given(method("POST"))
            .and(path("/session/tempo-testnet"))
            // wiremock 0.6 has no `not` combinator; `Match` is implemented for
            // closures, so the negative check goes inline.
            .and(|req: &wiremock::Request| !req.headers.contains_key("authorization"))
            .respond_with(
                ResponseTemplate::new(402)
                    .insert_header("www-authenticate", session_offer("c1", chain_id, ESCROW)),
            )
    }

    // A base64url `Payment-Receipt` header, as the gateway emits it.
    fn receipt_header(accepted: &str, spent: &str) -> String {
        super::super::base64_url_nopad(
            serde_json::to_vec(&serde_json::json!({
                "acceptedCumulative": accepted,
                "spent": spent,
            }))
            .unwrap(),
        )
    }

    // The credential POST: matched on the Authorization header the probe lacks.
    fn credential_mock(resp: ResponseTemplate) -> Mock {
        Mock::given(method("POST"))
            .and(path("/session/tempo-testnet"))
            .and(header_exists("authorization"))
            .respond_with(resp)
    }

    fn rpc_ok() -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "result": "0xa5bf"
        }))
    }

    #[tokio::test]
    async fn open_deposits_and_returns_the_new_channel() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(rpc_ok()).expect(1).mount(&server).await;

        let payment = tempo_payment(&server.uri());
        let ch = open(&reqwest::Client::new(), &payment, 100_000)
            .await
            .unwrap();

        assert_eq!(ch.chain_id, 42431);
        assert_eq!(ch.escrow_contract, ESCROW);
        assert_eq!(ch.deposit, 100_000);
        // The opening voucher authorizes the first per-call unit (amount "10").
        assert_eq!(ch.per_call, 10);
        assert_eq!(ch.cumulative_spent, 10);
        assert!(ch.channel_id.starts_with("0x"));
        assert_eq!(ch.authorized_signer.to_lowercase(), EVM_ADDR_LOWER);
    }

    #[tokio::test]
    async fn open_above_max_amount_is_refused_before_any_request() {
        // base_url points at a closed port: reaching the network would error
        // differently, so this also proves the guard runs before I/O.
        let payment = tempo_payment("http://127.0.0.1:1");
        let err = open(
            &reqwest::Client::new(),
            &payment,
            payment.max_amount + 1,
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("exceeds max_amount"))
        );
    }

    #[tokio::test]
    async fn open_surfaces_a_gateway_refusal_as_api() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(
            ResponseTemplate::new(400)
                .set_body_string("transaction does not contain a valid escrow open call"),
        )
        .mount(&server)
        .await;

        let payment = tempo_payment(&server.uri());
        let err = open(&reqwest::Client::new(), &payment, 100_000)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { status, .. } if status == 400));
    }

    // A 402 offering only another chain must not open a channel on it.
    #[tokio::test]
    async fn open_for_an_unoffered_chain_is_unsupported() {
        let server = MockServer::start().await;
        probe_mock(1).mount(&server).await;

        let payment = tempo_payment(&server.uri());
        let err = open(&reqwest::Client::new(), &payment, 100_000)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::PaymentUnsupported { .. }));
    }

    // A non-402 probe response means the endpoint is not offering a session.
    #[tokio::test]
    async fn open_without_a_402_challenge_is_unsupported() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/session/tempo-testnet"))
            .respond_with(rpc_ok())
            .mount(&server)
            .await;

        let payment = tempo_payment(&server.uri());
        let err = open(&reqwest::Client::new(), &payment, 100_000)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("did not return a 402"))
        );
    }

    // A 404 means the pinned lifecycle route is no longer served: that is an SDK
    // fix, not a payment problem, so it must not read as a generic refusal.
    #[tokio::test]
    async fn a_404_on_the_lifecycle_route_names_the_pinned_network() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(format!("/session/{SESSION_ROUTE_NETWORK}")))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let payment = tempo_payment(&server.uri());
        let err = open(&reqwest::Client::new(), &payment, 100_000)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered }
                if offered.contains(SESSION_ROUTE_NETWORK) && offered.contains("404")),
            "a 404 should name the pinned route network"
        );
    }

    #[tokio::test]
    async fn top_up_adds_to_the_local_deposit() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(rpc_ok()).expect(1).mount(&server).await;

        let payment = tempo_payment(&server.uri());
        let ch = sample_channel();
        let after = top_up(
            &reqwest::Client::new(),
            &payment,
            &ch,
            50_000,
        )
        .await
        .unwrap();

        assert_eq!(after.deposit, ch.deposit + 50_000);
        // Top-up moves deposit only; the spend high-water mark is untouched.
        assert_eq!(after.cumulative_spent, ch.cumulative_spent);
        assert_eq!(after.channel_id, ch.channel_id);
    }

    #[tokio::test]
    async fn top_up_surfaces_a_gateway_refusal_as_api() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(ResponseTemplate::new(402).set_body_string("insufficient-balance"))
            .mount(&server)
            .await;

        let payment = tempo_payment(&server.uri());
        let err = top_up(
            &reqwest::Client::new(),
            &payment,
            &sample_channel(),
            50_000,
        )
        .await
        .unwrap_err();
        assert!(matches!(err, SdkError::Api { status, .. } if status == 402));
    }

    #[tokio::test]
    async fn close_settles_the_final_cumulative() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(rpc_ok()).expect(1).mount(&server).await;

        let payment = tempo_payment(&server.uri());
        close(
            &reqwest::Client::new(),
            &payment,
            &sample_channel(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn close_surfaces_a_gateway_refusal_as_api() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(ResponseTemplate::new(409).set_body_string("channel already closed"))
            .mount(&server)
            .await;

        let payment = tempo_payment(&server.uri());
        let err = close(
            &reqwest::Client::new(),
            &payment,
            &sample_channel(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, SdkError::Api { status, .. } if status == 409));
    }

    #[tokio::test]
    async fn status_reads_the_gateways_channel_view_from_the_receipt() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(
            rpc_ok().insert_header("payment-receipt", receipt_header("1000", "1000").as_str()),
        )
        .expect(1)
        .mount(&server)
        .await;

        let payment = tempo_payment(&server.uri());
        let ch = sample_channel();
        let st = status(&reqwest::Client::new(), &payment, &ch)
            .await
            .unwrap();

        assert_eq!(st.channel_id, ch.channel_id);
        assert_eq!(st.accepted_cumulative, 1000);
        assert_eq!(st.spent, 1000);
    }

    // The status probe costs one request unit, so it must refuse before any I/O
    // when the channel has no room left for it.
    #[tokio::test]
    async fn status_without_room_for_the_probe_is_refused_before_any_request() {
        let payment = tempo_payment("http://127.0.0.1:1");
        let mut ch = sample_channel();
        ch.cumulative_spent = ch.deposit;
        let err = status(&reqwest::Client::new(), &payment, &ch)
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("no room for a status probe"))
        );
    }

    #[tokio::test]
    async fn status_without_a_receipt_header_is_a_config_error() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(rpc_ok()).mount(&server).await;

        let payment = tempo_payment(&server.uri());
        let err = status(
            &reqwest::Client::new(),
            &payment,
            &sample_channel(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, SdkError::Config(m) if m.contains("no Payment-Receipt")));
    }

    #[tokio::test]
    async fn voucher_call_returns_the_rpc_envelope() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(rpc_ok()).expect(1).mount(&server).await;

        let payment = tempo_payment(&server.uri());
        let ch = sample_channel();
        let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_chainId" });
        let out = voucher_call(
            &reqwest::Client::new(),
            &payment,
            "tempo-testnet",
            &ch,
            ch.cumulative_spent + ch.per_call,
            &body,
        )
        .await
        .unwrap();
        assert!(out.contains("0xa5bf"));
    }

    #[tokio::test]
    async fn voucher_call_refusal_surfaces_the_gateway_status() {
        let server = MockServer::start().await;
        probe_mock(42431).mount(&server).await;
        credential_mock(ResponseTemplate::new(402).set_body_string("insufficient-balance"))
            .mount(&server)
            .await;

        let payment = tempo_payment(&server.uri());
        let ch = sample_channel();
        let body = serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_chainId" });
        let err = voucher_call(
            &reqwest::Client::new(),
            &payment,
            "tempo-testnet",
            &ch,
            ch.cumulative_spent + ch.per_call,
            &body,
        )
        .await
        .unwrap_err();
        assert!(matches!(
            err,
            SdkError::Api { .. } | SdkError::PaymentRejected { .. }
        ));
    }

    #[test]
    fn unparseable_challenge_expiry_is_an_error_not_an_unbounded_window() {
        let mut parsed = parse_session_challenge(
            &session_offer("c1", 42431, "0x33b901018174DDabE4841042ab76ba85D4e24f25"),
            42431,
        )
        .unwrap();
        parsed.expires = "not-a-timestamp".into();
        let err = session_valid_before(&parsed).unwrap_err();
        assert!(matches!(err, SdkError::Config(m) if m.contains("unparseable")));
    }
}
