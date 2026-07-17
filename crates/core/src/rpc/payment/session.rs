//! MPP session (payment-channel) lane for `rpc.call`.
//!
//! The counterpart to the per-request MPP charge in the parent module: instead
//! of signing a fresh Tempo transaction per request, the caller opens a payment
//! channel by depositing into the TIP-1034 TIP-20 Channel Reserve escrow
//! precompile, then authorizes spend with cumulative EIP-712 vouchers
//! (`Authorization: Payment`) — one `ecrecover` server-side, no on-chain tx per
//! call. The gateway settles the channel on-chain in batches on its own
//! schedule; the client cooperatively closes to settle + refund the unused
//! deposit.
//!
//! Wire protocol (matches the `mppx` reference client, github.com/wevm/mppx):
//! - Endpoints under `{mpp}/session/:network`.
//! - Channel lifecycle credentials are a discriminated union on `action`
//!   (`open`/`topUp`/`voucher`/`close`), each a `Payment <base64url JSON>`
//!   credential of `{challenge, payload, source}`.
//! - `open`/`topUp` carry a fee-sponsored Tempo tx that calls the escrow
//!   precompile; `voucher`/`close` are pure EIP-712 voucher signatures.
//! - The channelId is derived locally (TIP-1034) so client state can be
//!   reconstructed; `status` is the recovery path (the gateway is the source of
//!   truth for the channel high-water mark).

use serde::Deserialize;
use serde_json::Value;

use crate::errors::{HttpKind, SdkError};

use super::signer::tempo::{
    ChannelDescriptor, EscrowAction, TempoEscrowRequest, TIP20_CHANNEL_ESCROW,
};
use super::{now_unix, random_nonce, PaymentScheme, ResolvedPayment};

/// Local state for an open MPP payment channel. The CLI persists this between
/// runs (like the drawdown session JWT); `status` re-derives it from the
/// gateway if the local copy is lost.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ChannelState {
    /// TIP-1034 channel id (`0x`-hex bytes32).
    pub channel_id: String,
    /// The escrow token (TIP-20 currency) the channel is denominated in.
    pub token: String,
    /// The channel payee (settlement recipient), from the open challenge.
    pub payee: String,
    /// The channel operator (or the zero address when unset).
    pub operator: String,
    /// Payer entropy used to derive the channel (`0x`-hex bytes32).
    pub salt: String,
    /// Voucher signer (or the zero address, delegating to the payer).
    pub authorized_signer: String,
    /// The open tx's TIP-1034 expiringNonceHash (`0x`-hex bytes32).
    pub expiring_nonce_hash: String,
    /// Total deposited into the channel so far, in token base units.
    pub deposit: u128,
    /// Highest cumulative amount authorized by a voucher so far.
    pub cumulative_spent: u128,
    /// The gateway's per-call price (from the open challenge), so the caller can
    /// advance `cumulative_spent` by one unit per session call.
    pub per_call: u128,
    /// CAIP-2 chain id the channel lives on.
    pub chain_id: u64,
}

impl ChannelState {
    fn descriptor(&self, payer: &str) -> ChannelDescriptor {
        ChannelDescriptor {
            payer: payer.to_string(),
            payee: self.payee.clone(),
            operator: self.operator.clone(),
            token: self.token.clone(),
            salt: self.salt.clone(),
            authorized_signer: self.authorized_signer.clone(),
            expiring_nonce_hash: self.expiring_nonce_hash.clone(),
        }
    }
}

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

// The escrow amounts are uint96 on-chain; reject anything wider before signing.
fn assert_uint96(value: u128, what: &str) -> Result<(), SdkError> {
    if value > (1u128 << 96) - 1 {
        return Err(SdkError::Config(format!(
            "{what} {value} exceeds the uint96 escrow ceiling"
        )));
    }
    Ok(())
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
    query_network: &str,
    deposit: u128,
) -> Result<ChannelState, SdkError> {
    assert_uint96(deposit, "deposit")?;
    if deposit > payment.max_amount {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "requested channel deposit {deposit} exceeds max_amount {}",
                payment.max_amount
            ),
        });
    }
    let challenge = probe_session_challenge(client, payment, query_network).await?;
    let chain_id = challenge_chain_id(&challenge)?;
    let token = require_str(&challenge.request, "currency")?;
    let payee = require_str(&challenge.request, "recipient")?;
    let payer = payment.signer.address()?;

    // Sign the escrow `open` tx → channelId + expiringNonceHash. salt is fresh
    // payer entropy; operator/authorizedSigner default to the zero address
    // (payee-operator unset; voucher signer delegates to the payer).
    let salt = format!("0x{}", hex::encode(random_nonce()));
    let signed = payment.signer.sign_escrow_tx(&TempoEscrowRequest {
        chain_id,
        valid_before: now_unix() + 25,
        action: EscrowAction::Open {
            payee: payee.clone(),
            operator: ZERO_ADDRESS.to_string(),
            token: token.clone(),
            deposit,
            salt: salt.clone(),
            authorized_signer: ZERO_ADDRESS.to_string(),
        },
    })?;
    let channel_id = signed
        .channel_id
        .map(|c| format!("0x{}", hex::encode(c)))
        .ok_or_else(|| SdkError::Config("open did not derive a channelId".into()))?;

    // The opening voucher authorizes the first unit of spend (the per-call
    // amount from the challenge). cumulativeAmount starts at that amount.
    let per_unit = require_amount(&challenge.request)?;
    let voucher_sig = payment.signer.sign_session_voucher(
        &channel_id,
        per_unit,
        chain_id,
        TIP20_CHANNEL_ESCROW,
    )?;

    let descriptor = descriptor_json(&payer, &payee, &token, &salt, &signed.expiring_nonce_hash);
    let payload = serde_json::json!({
        "action": "open",
        "type": "transaction",
        "channelId": channel_id,
        "transaction": format!("0x{}", hex::encode(&signed.transaction)),
        "signature": voucher_sig,
        "descriptor": descriptor,
        "cumulativeAmount": per_unit.to_string(),
    });
    post_session_credential(client, payment, query_network, &challenge, &payer, payload).await?;

    Ok(ChannelState {
        channel_id,
        token,
        payee,
        operator: ZERO_ADDRESS.to_string(),
        salt,
        authorized_signer: ZERO_ADDRESS.to_string(),
        expiring_nonce_hash: signed.expiring_nonce_hash,
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
    query_network: &str,
    channel: &ChannelState,
    additional_deposit: u128,
) -> Result<ChannelState, SdkError> {
    assert_uint96(additional_deposit, "additionalDeposit")?;
    let payer = payment.signer.address()?;
    let challenge = probe_session_challenge(client, payment, query_network).await?;

    let signed = payment.signer.sign_escrow_tx(&TempoEscrowRequest {
        chain_id: channel.chain_id,
        valid_before: now_unix() + 25,
        action: EscrowAction::TopUp {
            descriptor: channel.descriptor(&payer),
            additional_deposit,
        },
    })?;
    let payload = serde_json::json!({
        "action": "topUp",
        "type": "transaction",
        "channelId": channel.channel_id,
        "transaction": format!("0x{}", hex::encode(&signed.transaction)),
        "descriptor": descriptor_json(
            &payer, &channel.payee, &channel.token, &channel.salt, &channel.expiring_nonce_hash,
        ),
        "additionalDeposit": additional_deposit.to_string(),
    });
    post_session_credential(client, payment, query_network, &challenge, &payer, payload).await?;

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
    query_network: &str,
    channel: &ChannelState,
) -> Result<(), SdkError> {
    let payer = payment.signer.address()?;
    let challenge = probe_session_challenge(client, payment, query_network).await?;
    let signature = payment.signer.sign_session_voucher(
        &channel.channel_id,
        channel.cumulative_spent,
        channel.chain_id,
        TIP20_CHANNEL_ESCROW,
    )?;
    let payload = serde_json::json!({
        "action": "close",
        "channelId": channel.channel_id,
        "descriptor": descriptor_json(
            &payer, &channel.payee, &channel.token, &channel.salt, &channel.expiring_nonce_hash,
        ),
        "cumulativeAmount": channel.cumulative_spent.to_string(),
        "signature": signature,
    });
    post_session_credential(client, payment, query_network, &challenge, &payer, payload).await?;
    Ok(())
}

/// The gateway's view of a channel — the recovery path when local state is
/// lost. Returns the on-chain deposit ceiling and the accepted cumulative
/// high-water mark for `channel_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelStatus {
    pub channel_id: String,
    pub deposit: u128,
    pub accepted_cumulative: u128,
}

/// Fetches the gateway's status for `channel_id` (GET
/// `{mpp}/session/:network/channels/:id`).
pub async fn status(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    query_network: &str,
    channel_id: &str,
) -> Result<ChannelStatus, SdkError> {
    let base = session_base(payment, query_network);
    let url = format!("{base}/channels/{channel_id}");
    let resp = client.get(&url).send().await.map_err(SdkError::Http)?;
    let http_status = resp.status();
    let body = resp.text().await.map_err(SdkError::Http)?;
    if !http_status.is_success() {
        return Err(SdkError::Api {
            status: http_status,
            body,
        });
    }
    #[derive(Deserialize)]
    struct StatusBody {
        deposit: String,
        #[serde(rename = "acceptedCumulative")]
        accepted_cumulative: String,
    }
    let parsed: StatusBody =
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
    Ok(ChannelStatus {
        channel_id: channel_id.to_string(),
        deposit: parse_u128(&parsed.deposit)?,
        accepted_cumulative: parse_u128(&parsed.accepted_cumulative)?,
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
    assert_uint96(new_cumulative, "cumulativeAmount")?;
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
        TIP20_CHANNEL_ESCROW,
    )?;
    // A voucher credential needs the challenge it answers; the gateway echoes it
    // on the 402. Probe once (free) to obtain the current session challenge.
    let challenge = probe_session_challenge(client, payment, query_network).await?;
    let payload = serde_json::json!({
        "action": "voucher",
        "channelId": channel.channel_id,
        "descriptor": descriptor_json(
            &payer, &channel.payee, &channel.token, &channel.salt, &channel.expiring_nonce_hash,
        ),
        "cumulativeAmount": new_cumulative.to_string(),
        "signature": signature,
    });
    let credential = build_credential(&challenge, &payer, &payload);

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
    query_network: &str,
) -> Result<SessionChallenge, SdkError> {
    let base = session_base(payment, query_network);
    let resp = client
        .post(&base)
        .json(&serde_json::json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_chainId", "params": [] }))
        .send()
        .await
        .map_err(SdkError::Http)?;
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
    parse_session_challenge(&header)
}

// Parse the FIRST tempo/session challenge from the WWW-Authenticate header.
fn parse_session_challenge(header: &str) -> Result<SessionChallenge, SdkError> {
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
        return Ok(SessionChallenge {
            id: get("id"),
            realm: get("realm"),
            intent: "session".into(),
            description: get("description"),
            expires: get("expires"),
            request_b64,
            request,
        });
    }
    Err(SdkError::PaymentUnsupported {
        offered: "no tempo/session challenge offered".into(),
    })
}

// Build the `Payment <base64url JSON>` credential: {challenge, payload, source}
// with the challenge's original request echoed verbatim (matches mppx's
// Credential.serialize wire shape).
fn build_credential(challenge: &SessionChallenge, payer: &str, payload: &Value) -> String {
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
        "source": format!("did:pkh:eip155:{payer}"),
    });
    super::base64_url_nopad(serde_json::to_vec(&credential).unwrap_or_default())
}

// POST a channel-management credential to the session endpoint and require a
// 2xx. Management POSTs settle nothing off the caller's per-call amount (they
// commit deposits / close), so a non-2xx is a plain Api refusal.
async fn post_session_credential(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    query_network: &str,
    challenge: &SessionChallenge,
    payer: &str,
    payload: Value,
) -> Result<(), SdkError> {
    let credential = build_credential(challenge, payer, &payload);
    let base = session_base(payment, query_network);
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
    Ok(())
}

fn descriptor_json(
    payer: &str,
    payee: &str,
    token: &str,
    salt: &str,
    expiring_nonce_hash: &str,
) -> Value {
    serde_json::json!({
        "payer": payer,
        "payee": payee,
        "operator": ZERO_ADDRESS,
        "token": token,
        "salt": salt,
        "authorizedSigner": ZERO_ADDRESS,
        "expiringNonceHash": expiring_nonce_hash,
    })
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

    const EVM_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

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
            operator: ZERO_ADDRESS.into(),
            salt: format!("0x{}", "22".repeat(32)),
            authorized_signer: ZERO_ADDRESS.into(),
            expiring_nonce_hash: format!("0x{}", "33".repeat(32)),
            deposit: 100_000,
            cumulative_spent: 500,
            per_call: 500,
            chain_id: 42431,
        }
    }

    #[test]
    fn assert_uint96_rejects_over_ceiling() {
        assert!(assert_uint96((1u128 << 96) - 1, "x").is_ok());
        assert!(assert_uint96(1u128 << 96, "x").is_err());
    }

    #[test]
    fn descriptor_json_has_all_seven_fields() {
        let d = descriptor_json("0xpayer", "0xpayee", "0xtoken", "0xsalt", "0xhash");
        for k in [
            "payer",
            "payee",
            "operator",
            "token",
            "salt",
            "authorizedSigner",
            "expiringNonceHash",
        ] {
            assert!(d.get(k).is_some(), "missing {k}");
        }
    }

    #[test]
    fn parse_session_challenge_selects_tempo_session() {
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
            "Payment id=\"c1\", realm=\"mpp.quicknode.com\", method=\"tempo\", intent=\"charge\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", request=\"ey000\", Payment id=\"c2\", realm=\"mpp.quicknode.com\", method=\"tempo\", intent=\"session\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", request=\"{request}\""
        );
        let parsed = parse_session_challenge(&header).unwrap();
        assert_eq!(parsed.intent, "session");
        assert_eq!(parsed.id, "c2");
        assert_eq!(challenge_chain_id(&parsed).unwrap(), 42431);
        assert_eq!(require_amount(&parsed.request).unwrap(), 500);
    }

    #[test]
    fn channel_descriptor_round_trips_the_payer() {
        let ch = sample_channel();
        let d = ch.descriptor("0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
        assert_eq!(d.payer, "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266");
        assert_eq!(d.token, ch.token);
        assert_eq!(d.expiring_nonce_hash, ch.expiring_nonce_hash);
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
}
