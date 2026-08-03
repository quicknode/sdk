//! Crypto-micropayment lanes for `rpc.call`.
//!
//! A caller can pay per RPC request with a stablecoin instead of a provisioned
//! account + API key, against Quicknode's `x402.quicknode.com` and
//! `mpp.quicknode.com` gateways. [`pay_and_call`] runs the shared 402 loop; the
//! per-protocol differences (challenge parse, entry select, credential build,
//! receipt parse) live inline on [`PaymentScheme`].
//!
//! The flow: POST the JSON-RPC body keyless → the gateway answers `402` with a
//! menu of payment options → select the entry matching the caller's selector
//! (skipping unsupported shapes and anything over `max_amount`) → sign a
//! credential → resend **exactly once** with the credential attached → `200`.
//! A second 402 is terminal ([`SdkError::PaymentRejected`]); a lost response
//! after the paid resend is [`SdkError::PaymentIndeterminate`].

pub mod drawdown;
#[cfg(feature = "payments-tempo")]
pub mod session;
pub mod signer;

use serde::Deserialize;
use serde_json::Value;

use crate::errors::{HttpKind, SdkError};
use signer::Signer;

#[cfg(feature = "payments-tempo")]
use signer::TempoChargeRequest;

/// The payment protocol. `X402` (pay-per-request via `x402.quicknode.com`)
/// covers EVM + Solana; `MppCharge` (via `mpp.quicknode.com`) covers Tempo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentScheme {
    X402,
    MppCharge,
}

impl PaymentScheme {
    /// The gateway host base for this scheme. `base_url_override` (tests / the
    /// wiremock harness) wins when set.
    pub fn host_base<'a>(&self, override_base: Option<&'a str>) -> &'a str
    where
        'static: 'a,
    {
        override_base.unwrap_or(match self {
            PaymentScheme::X402 => "https://x402.quicknode.com",
            PaymentScheme::MppCharge => "https://mpp.quicknode.com",
        })
    }
}

/// A typed MPP settlement receipt (the caller's proof of payment). `reference`
/// is the settlement transaction hash. `None` for x402 and non-payment lanes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentReceipt {
    pub method: String,
    pub status: String,
    pub timestamp: String,
    /// Settlement transaction hash.
    pub reference: String,
}

/// The caller's payment selector + custody parameters, resolved from the
/// binding-facing `PaymentConfig` at the config boundary. Holds the live
/// `Signer` (which custodies the key in a `SecretString`).
pub struct ResolvedPayment {
    pub scheme: PaymentScheme,
    pub signer: Signer,
    /// CAIP-2 pay network, e.g. `eip155:84532`, `solana:5eykt4…`, or a Tempo
    /// chain selector. Used to match the offered menu entry.
    pub pay_network: String,
    /// Asset (token) address/mint to pay in — matches the menu entry's `asset`.
    pub asset: String,
    /// Spend ceiling in base units of `asset`. The selector skips any entry
    /// above this and the driver refuses to sign one.
    pub max_amount: u128,
    /// Test-only gateway base override.
    pub base_url_override: Option<String>,
    /// Resolved Solana RPC URL for x402/Solana payment-build reads (blockhash,
    /// token program). `None` for EVM/Tempo.
    pub svm_rpc_url: Option<String>,
}

impl std::fmt::Debug for ResolvedPayment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResolvedPayment")
            .field("scheme", &self.scheme)
            .field("signer", &self.signer) // renders [redacted]
            .field("pay_network", &self.pay_network)
            .field("asset", &self.asset)
            .field("max_amount", &self.max_amount)
            .finish()
    }
}

impl ResolvedPayment {
    /// Convert the binding-facing plain-data `PaymentConfig` into the internal
    /// resolved form. The signer variant is DERIVED — never stated by the
    /// caller — from the scheme + pay_network CAIP-2 prefix:
    /// MPP ⇒ Tempo; x402 + `eip155:` ⇒ Evm; x402 + `solana:` ⇒ Svm. A
    /// `max_amount` that isn't an integer is a `Config` error at construction,
    /// not at call time.
    pub fn from_config(config: &crate::config::PaymentConfig) -> Result<Self, SdkError> {
        use secrecy::SecretString;

        let scheme = match config.scheme.as_str() {
            "x402" => PaymentScheme::X402,
            "mpp" | "mpp-charge" => PaymentScheme::MppCharge,
            other => {
                return Err(SdkError::Config(format!(
                    "unknown payment scheme {other:?} (expected \"x402\" or \"mpp\")"
                )))
            }
        };

        let key = SecretString::new(config.key.clone());
        let signer = match scheme {
            PaymentScheme::MppCharge => Signer::Tempo(key),
            PaymentScheme::X402 => {
                if config.pay_network.starts_with("eip155:") {
                    Signer::Evm(key)
                } else if config.pay_network.starts_with("solana:") {
                    Signer::Svm(key)
                } else {
                    return Err(SdkError::Config(format!(
                        "x402 pay_network must start with eip155: or solana:, got {:?}",
                        config.pay_network
                    )));
                }
            }
        };

        let max_amount = config.max_amount.parse::<u128>().map_err(|_| {
            SdkError::Config(format!(
                "max_amount must be an integer in base units, got {:?}",
                config.max_amount
            ))
        })?;

        // Resolve the Solana RPC source for x402/Solana payment-build reads. The
        // caller's explicit override wins; otherwise fall back to a public
        // Solana RPC matching the pay cluster. (The tooling-endpoint step is
        // wired by RpcApiClient, which has the network map; this default is the
        // last resort — the READMEs push the explicit override at any volume.)
        let svm_rpc_url = if matches!(signer.kind(), signer::ChainKind::Svm) {
            Some(
                config
                    .svm_rpc_url
                    .clone()
                    .unwrap_or_else(|| default_solana_rpc(&config.pay_network).to_string()),
            )
        } else {
            None
        };

        Ok(ResolvedPayment {
            scheme,
            signer,
            pay_network: config.pay_network.clone(),
            asset: config.asset.clone(),
            max_amount,
            base_url_override: config.base_url_override.clone(),
            svm_rpc_url,
        })
    }
}

// Solana CAIP-2 ids are `solana:<genesis-hash-prefix>`. Devnet's genesis hash
// begins `EtWTRAB…`; the literal string "devnet" never appears in a CAIP-2 id,
// so both the RPC default and the tooling-key resolution must key off this
// prefix (not `contains("devnet")`). Returns true for the devnet cluster.
pub(crate) fn solana_pay_network_is_devnet(pay_network: &str) -> bool {
    pay_network.contains("EtWTRABZaYq6iMfeYKouRu166VU2xqa1")
}

// Public Solana RPC default matching the pay cluster. Rate-limits aggressively;
// callers at any volume should set an explicit `svm_rpc_url`.
fn default_solana_rpc(pay_network: &str) -> &'static str {
    if solana_pay_network_is_devnet(pay_network) {
        "https://api.devnet.solana.com"
    } else {
        "https://api.mainnet-beta.solana.com"
    }
}

// ── x402 challenge shapes (v2) ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct X402Body {
    #[serde(rename = "x402Version")]
    x402_version: u32,
    accepts: Vec<Value>,
}

// ── The 402 driver ───────────────────────────────────────────────────────────

/// Runs the payment handshake for one JSON-RPC call and returns the raw
/// JSON-RPC envelope text plus an optional settlement receipt. The caller
/// (`RpcApiClient`) parses the JSON-RPC envelope; this layer owns only the
/// 402 dance.
pub async fn pay_and_call(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    query_network: &str,
    body: &Value,
) -> Result<(String, Option<PaymentReceipt>), SdkError> {
    let base = payment
        .scheme
        .host_base(payment.base_url_override.as_deref());
    let url = format!("{}/{}", base.trim_end_matches('/'), query_network);

    // 1. Unpaid probe. A transport error here is a plain Http error — no
    //    payment exists yet.
    let first = client
        .post(&url)
        .json(body)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let status = first.status().as_u16();

    // A non-402 first response means the gateway did not demand payment (or
    // errored). Pass it back to the caller's JSON-RPC parser via the text.
    if status != 402 {
        let text = first.text().await.map_err(SdkError::Http)?;
        return Ok((text, None));
    }

    // 2. Parse the challenge and build a credential for the matching entry.
    let www_authenticate = first
        .headers()
        .get("www-authenticate")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let challenge_body = first.text().await.map_err(SdkError::Http)?;

    let authorized = match payment.scheme {
        PaymentScheme::X402 => authorize_x402(client, payment, &challenge_body).await?,
        PaymentScheme::MppCharge => {
            let header = www_authenticate.ok_or_else(|| SdkError::PaymentUnsupported {
                offered: "MPP 402 without a WWW-Authenticate header".into(),
            })?;
            authorize_mpp(payment, &header)?
        }
    };

    // 3. Paid resend — exactly once. Transport errors here are classified so a
    //    lost response after the bytes may have reached the gateway surfaces as
    //    PaymentIndeterminate (do not blind-retry), while a refused connection
    //    (nothing sent) stays a plain retryable Http error.
    let mut req = client.post(&url).json(body);
    req = match &authorized {
        Authorized::X402 { header } => req.header("PAYMENT-SIGNATURE", header),
        #[cfg(feature = "payments-tempo")]
        Authorized::Mpp { credential } => {
            req.header("Authorization", format!("Payment {credential}"))
        }
    };
    let paid = match req.send().await {
        Ok(resp) => resp,
        Err(e) => {
            let err = SdkError::Http(e);
            return Err(match err.http_kind() {
                Some(HttpKind::Connect) => err, // TCP never established: safe to retry
                _ => SdkError::PaymentIndeterminate, // Timeout/Other: bytes may have landed
            });
        }
    };
    let paid_status = paid.status().as_u16();

    // Any non-2xx on the paid resend is terminal: the payment credential was
    // submitted and the gateway did not accept it. This covers a second 402
    // (rejected credential) AND a 5xx/other settlement failure — both must
    // surface as PaymentRejected so the caller keeps the "payment was
    // submitted" signal, rather than the 5xx body falling through to a Decode
    // error on a non-JSON-RPC response.
    if !(200..300).contains(&paid_status) {
        let body = paid.text().await.unwrap_or_default();
        return Err(SdkError::PaymentRejected {
            status: paid_status,
            body: enrich_rejection(payment, body),
        });
    }

    // Capture the MPP receipt before consuming the body.
    let receipt = paid
        .headers()
        .get("payment-receipt")
        .and_then(|v| v.to_str().ok())
        .and_then(parse_receipt);

    // Reading the body can itself fail on a lost connection after headers.
    let text = match paid.text().await {
        Ok(t) => t,
        Err(e) => {
            let err = SdkError::Http(e);
            return Err(match err.http_kind() {
                Some(HttpKind::Connect) => err,
                _ => SdkError::PaymentIndeterminate,
            });
        }
    };
    Ok((text, receipt))
}

pub(super) enum Authorized {
    X402 {
        header: String,
    },
    #[cfg(feature = "payments-tempo")]
    Mpp {
        credential: String,
    },
}

impl Authorized {
    /// The x402 `PAYMENT-SIGNATURE` header value, for reuse by the drawdown
    /// credit-purchase path. `None` for non-x402 credentials.
    pub(super) fn x402_header(&self) -> Option<&str> {
        match self {
            Authorized::X402 { header } => Some(header),
            #[cfg(feature = "payments-tempo")]
            Authorized::Mpp { .. } => None,
        }
    }
}

// ── x402 authorize (EVM + Solana) ────────────────────────────────────────────

pub(super) async fn authorize_x402(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    challenge_body: &str,
) -> Result<Authorized, SdkError> {
    // Pre-payment: nothing has been signed or sent yet, so an unreadable menu
    // is "no usable offer" (PaymentUnsupported), never a Decode — paid-lane
    // callers treat Decode as a post-payment failure whose outcome is unknown.
    let parsed: X402Body =
        serde_json::from_str(challenge_body).map_err(|source| SdkError::PaymentUnsupported {
            offered: format!("an unparseable x402 challenge (invalid JSON: {source})"),
        })?;

    authorize_x402_entry(client, payment, &parsed).await
}

/// Like [`authorize_x402`], but selects the credit-drawdown offer rather than
/// the per-request one. The credit tier is identified by its `extra.name`
/// (`GatewayWalletBatched`) and its long `maxTimeoutSeconds`, NOT by amount —
/// it is typically the *cheapest* entry on the menu, so picking by size would
/// select a per-request offer and sign the wrong scheme against it.
///
/// Signing a Circle Gateway batched transfer is a different construction from
/// the EIP-3009 `TransferWithAuthorization` used by the per-request lane: its
/// EIP-712 domain separator is `extra.verifyingContract`, not the asset. When the
/// credit tier cannot be signed, refuse — never fall back to a per-request offer,
/// which would settle a far larger amount than the caller asked for.
pub(super) async fn authorize_x402_credit(
    _client: &reqwest::Client,
    payment: &ResolvedPayment,
    challenge_body: &str,
) -> Result<Authorized, SdkError> {
    let parsed: X402Body =
        serde_json::from_str(challenge_body).map_err(|source| SdkError::PaymentUnsupported {
            offered: format!("an unparseable x402 challenge (invalid JSON: {source})"),
        })?;

    let credit_offered = parsed.accepts.iter().any(|entry| {
        let network = entry.get("network").and_then(Value::as_str).unwrap_or("");
        let asset = entry.get("asset").and_then(Value::as_str).unwrap_or("");
        network == payment.pay_network
            && asset.eq_ignore_ascii_case(&payment.asset)
            && entry.pointer("/extra/name").and_then(Value::as_str) == Some(GATEWAY_BATCHED)
    });

    Err(SdkError::PaymentUnsupported {
        offered: if credit_offered {
            format!(
                "the credit-drawdown offer uses the {GATEWAY_BATCHED} scheme, which this \
                 version cannot sign. Pay per request instead: call rpc.call rather than \
                 buying credits."
            )
        } else {
            format!(
                "no credit-drawdown offer for {}/{}. {}",
                payment.pay_network,
                payment.asset,
                describe_offered(&parsed.accepts, &[])
            )
        },
    })
}

// Select an accepts[] entry (the cheapest match) and authorize it with the
// chain-appropriate signer.
async fn authorize_x402_entry(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    parsed: &X402Body,
) -> Result<Authorized, SdkError> {
    let mut skipped: Vec<String> = Vec::new();
    let chosen = select_x402_entry(payment, &parsed.accepts, &mut skipped);
    let Some(entry) = chosen else {
        // Lead with the one lever the caller can pull. The full menu follows,
        // but a 20-entry dump should not bury the actionable sentence.
        let offered = match cheapest_over_ceiling(payment, &parsed.accepts) {
            Some(cheapest) => format!(
                "every offer for {}/{} is above max_amount {}; the cheapest is \
                 {cheapest} base units — raise max_amount to at least that. \
                 Full menu: {}",
                payment.pay_network,
                payment.asset,
                payment.max_amount,
                describe_offered(&parsed.accepts, &skipped)
            ),
            None => describe_offered(&parsed.accepts, &skipped),
        };
        return Err(SdkError::PaymentUnsupported { offered });
    };

    match payment.signer.kind() {
        signer::ChainKind::Evm => authorize_x402_evm(payment, &parsed.x402_version, &entry),
        signer::ChainKind::Svm => {
            authorize_x402_svm(client, payment, &parsed.x402_version, &entry).await
        }
        signer::ChainKind::Tempo => Err(SdkError::PaymentUnsupported {
            offered: "a Tempo signer cannot pay an x402 challenge (use the MPP scheme)".into(),
        }),
    }
}

// Circle Gateway batched-transfer scheme, advertised as `extra.name`. Its
// EIP-712 domain separator is `extra.verifyingContract` rather than the asset,
// so it needs a signing construction the per-request lane does not have.
const GATEWAY_BATCHED: &str = "GatewayWalletBatched";

// Select an accepts[] entry that matches {pay_network, asset}, has a supported
// `extra` shape, and whose amount is a non-negative integer ≤ max_amount.
//
// Returns the CHEAPEST such entry — the per-request tier. Menu order carries no
// meaning: a gateway may advertise tiers in any order, and where a network
// distinguishes its tiers only by amount (no `extra.name`), taking the first
// match can land on a tier this lane cannot pay. Picking the cheapest also
// makes `max_amount` a true ceiling rather than a tier selector.
//
// Records skip reasons for the PaymentUnsupported message.
fn select_x402_entry(
    payment: &ResolvedPayment,
    accepts: &[Value],
    skipped: &mut Vec<String>,
) -> Option<Value> {
    let mut best: Option<(u128, &Value)> = None;
    for entry in accepts {
        let network = entry.get("network").and_then(Value::as_str).unwrap_or("");
        let asset = entry.get("asset").and_then(Value::as_str).unwrap_or("");
        if network != payment.pay_network || !asset.eq_ignore_ascii_case(&payment.asset) {
            continue;
        }
        // Skip Circle Gateway nanopayment (GatewayWalletBatched): its
        // verifyingContract is a separate field, not the asset — a different
        // signing construction, deferred from v1.
        if entry.pointer("/extra/name").and_then(Value::as_str) == Some(GATEWAY_BATCHED) {
            skipped.push(format!("{network}/{asset}: {GATEWAY_BATCHED} (deferred)"));
            continue;
        }
        // Amount must be an integer base-unit string ≤ max_amount.
        let amount_str = entry.get("amount").and_then(Value::as_str).unwrap_or("");
        match amount_str.parse::<u128>() {
            Ok(amount) if amount <= payment.max_amount => {
                if best.is_none_or(|(best_amount, _)| amount < best_amount) {
                    best = Some((amount, entry));
                }
            }
            Ok(amount) => skipped.push(format!(
                "{network}/{asset}: amount {amount} exceeds max_amount {}",
                payment.max_amount
            )),
            Err(_) => skipped.push(format!(
                "{network}/{asset}: amount {amount_str:?} is not an integer"
            )),
        }
    }
    best.map(|(_, entry)| entry.clone())
}

fn authorize_x402_evm(
    payment: &ResolvedPayment,
    x402_version: &u32,
    entry: &Value,
) -> Result<Authorized, SdkError> {
    let chain_id = caip2_evm_chain_id(&payment.pay_network)?;
    let name = entry
        .pointer("/extra/name")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("x402 EVM entry missing extra.name".into()))?;
    let version = entry
        .pointer("/extra/version")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("x402 EVM entry missing extra.version".into()))?;
    let pay_to = entry
        .get("payTo")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("x402 entry missing payTo".into()))?;
    let amount = entry
        .get("amount")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<u128>().ok())
        .ok_or_else(|| SdkError::Config("x402 entry missing/invalid amount".into()))?;
    let max_timeout = entry
        .get("maxTimeoutSeconds")
        .and_then(Value::as_u64)
        .unwrap_or(60);

    let from = payment.signer.address()?;
    let now = now_unix();
    let valid_before = now + max_timeout;
    let nonce = random_nonce();

    let domain = signer::Eip712Domain {
        name: name.to_string(),
        version: version.to_string(),
        chain_id,
        verifying_contract: payment.asset.clone(),
    };
    let message = signer::TransferWithAuthorization {
        from: from.clone(),
        to: pay_to.to_string(),
        value: amount,
        valid_after: 0,
        valid_before,
        nonce,
    };
    let sig = payment.signer.sign_eip712(&domain, &message)?;

    // Envelope: {x402Version, accepted:<entry>, payload:{signature, authorization}}
    let envelope = serde_json::json!({
        "x402Version": x402_version,
        "accepted": entry,
        "payload": {
            "signature": format!("0x{}", hex::encode(sig)),
            "authorization": {
                "from": from,
                "to": pay_to,
                "value": amount.to_string(),
                "validAfter": "0",
                "validBefore": valid_before.to_string(),
                "nonce": format!("0x{}", hex::encode(nonce)),
            }
        }
    });
    // Never fall back to an empty credential: sending zero bytes turns a local
    // serialization bug into an opaque gateway rejection.
    let header = base64_std(serde_json::to_vec(&envelope).map_err(|e| {
        SdkError::Config(format!(
            "could not serialize the x402 payment credential: {e}"
        ))
    })?);
    Ok(Authorized::X402 { header })
}

#[cfg(feature = "payments-svm")]
async fn authorize_x402_svm(
    client: &reqwest::Client,
    payment: &ResolvedPayment,
    x402_version: &u32,
    entry: &Value,
) -> Result<Authorized, SdkError> {
    use signer::SvmTransferRequest;

    let pay_to = entry
        .get("payTo")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("x402 Solana entry missing payTo".into()))?;
    let fee_payer = entry
        .pointer("/extra/feePayer")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("x402 Solana entry missing extra.feePayer".into()))?;
    // The menu selector admits amounts as u128, but SPL TransferChecked encodes
    // the amount as a u64 (the Solana token-program ABI ceiling). Parse as u128
    // and narrow explicitly so an over-u64 amount surfaces as a clear overflow
    // error rather than being conflated with a missing/malformed field.
    let amount_str = entry
        .get("amount")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("x402 Solana entry missing amount".into()))?;
    let amount = amount_str
        .parse::<u128>()
        .ok()
        .filter(|a| *a <= u128::from(u64::MAX))
        .and_then(|a| u64::try_from(a).ok())
        .ok_or_else(|| {
            SdkError::Config(format!(
                "x402 Solana amount {amount_str:?} is not a valid u64 base-unit integer"
            ))
        })?;
    // The gateway 402s keyless sub-reads, so the mint and the recent blockhash
    // come from a plain Solana RPC (resolved source: override → tooling →
    // public default).
    let rpc_url = payment
        .svm_rpc_url
        .as_deref()
        .ok_or_else(|| SdkError::Config("x402/Solana requires a resolved Solana RPC URL".into()))?;

    // Read decimals and the owning token program off the mint itself rather
    // than trusting the challenge: `extra.decimals` is optional (and absent on
    // the live menu), and a wrong value silently transfers the wrong amount,
    // since TransferChecked validates decimals against the mint on-chain.
    let mint = fetch_mint_metadata(client, rpc_url, &payment.asset).await?;
    let recent_blockhash = fetch_latest_blockhash(client, rpc_url).await?;

    // The memo carries the payment's replay-protection nonce. Honour a
    // seller-supplied `extra.memo`; otherwise mint a random one.
    let memo = match entry.pointer("/extra/memo").and_then(Value::as_str) {
        Some(seller_memo) => seller_memo.to_string(),
        None => random_memo_nonce(),
    };

    let req = SvmTransferRequest {
        mint: payment.asset.clone(),
        pay_to: pay_to.to_string(),
        fee_payer: fee_payer.to_string(),
        amount,
        decimals: mint.decimals,
        recent_blockhash,
        token_program: mint.token_program,
        memo,
    };
    let tx = payment.signer.sign_svm_transfer(&req)?;

    // Envelope: {x402Version, accepted:<entry>, payload:{transaction:<base64>}}.
    // `payload` is an object, not a bare string — the x402 v2 payload schema
    // requires a record, and a string is rejected before verification.
    let envelope = serde_json::json!({
        "x402Version": x402_version,
        "accepted": entry,
        "payload": { "transaction": base64_std(tx) },
    });
    // Never fall back to an empty credential: sending zero bytes turns a local
    // serialization bug into an opaque gateway rejection.
    let header = base64_std(serde_json::to_vec(&envelope).map_err(|e| {
        SdkError::Config(format!(
            "could not serialize the x402 payment credential: {e}"
        ))
    })?);
    Ok(Authorized::X402 { header })
}

#[cfg(feature = "payments-svm")]
async fn fetch_latest_blockhash(
    client: &reqwest::Client,
    rpc_url: &str,
) -> Result<String, SdkError> {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "getLatestBlockhash",
        "params": [{ "commitment": "finalized" }]
    });
    let resp = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let text = resp.text().await.map_err(SdkError::Http)?;
    // Also pre-payment (the blockhash goes into a transaction that has not
    // been signed yet): a bad RPC response is a Config-class failure, not a
    // Decode.
    let parsed: Value = serde_json::from_str(&text).map_err(|source| {
        SdkError::Config(format!(
            "could not parse the Solana RPC response as JSON: {source}"
        ))
    })?;
    parsed
        .pointer("/result/value/blockhash")
        .and_then(Value::as_str)
        .map(String::from)
        .ok_or_else(|| {
            SdkError::Config(format!("could not read blockhash from Solana RPC: {text}"))
        })
}

/// A mint's decimals and the token program that owns it.
#[cfg(feature = "payments-svm")]
struct MintMetadata {
    decimals: u8,
    token_program: String,
}

/// Reads a mint account to learn its decimals and owning token program. Both
/// matter for `TransferChecked`: the instruction re-checks decimals against the
/// mint on-chain, and SPL Token vs Token-2022 changes the program the
/// instruction must target.
#[cfg(feature = "payments-svm")]
async fn fetch_mint_metadata(
    client: &reqwest::Client,
    rpc_url: &str,
    mint: &str,
) -> Result<MintMetadata, SdkError> {
    let body = serde_json::json!({
        "jsonrpc": "2.0", "id": 1, "method": "getAccountInfo",
        "params": [mint, { "encoding": "jsonParsed", "commitment": "finalized" }]
    });
    let resp = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await
        .map_err(SdkError::Http)?;
    let text = resp.text().await.map_err(SdkError::Http)?;
    // Pre-payment, like the blockhash read: a bad RPC response is Config-class.
    let parsed: Value = serde_json::from_str(&text).map_err(|source| {
        SdkError::Config(format!(
            "could not parse the Solana RPC response as JSON: {source}"
        ))
    })?;
    let account = parsed.pointer("/result/value").ok_or_else(|| {
        SdkError::Config(format!(
            "Solana mint {mint} was not found (asset may be wrong for this network)"
        ))
    })?;
    let token_program = account
        .get("owner")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            SdkError::Config(format!("could not read the owning program of mint {mint}"))
        })?
        .to_string();
    let decimals = account
        .pointer("/data/parsed/info/decimals")
        .and_then(Value::as_u64)
        .and_then(|d| u8::try_from(d).ok())
        .ok_or_else(|| SdkError::Config(format!("could not read decimals of mint {mint}")))?;
    Ok(MintMetadata {
        decimals,
        token_program,
    })
}

/// A 16-byte random nonce, hex-encoded, for the payment's memo. Randomness
/// comes from `rand::thread_rng` (the OS CSPRNG), matching the other nonce
/// generators in this module.
#[cfg(feature = "payments-svm")]
fn random_memo_nonce() -> String {
    use rand::RngCore;
    let mut nonce = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(not(feature = "payments-svm"))]
async fn authorize_x402_svm(
    _client: &reqwest::Client,
    _payment: &ResolvedPayment,
    _x402_version: &u32,
    _entry: &Value,
) -> Result<Authorized, SdkError> {
    Err(SdkError::PaymentUnsupported {
        offered: "x402/Solana requires the `payments-svm` feature".into(),
    })
}

// ── MPP authorize (Tempo) ────────────────────────────────────────────────────

#[cfg(feature = "payments-tempo")]
fn authorize_mpp(
    payment: &ResolvedPayment,
    www_authenticate: &str,
) -> Result<Authorized, SdkError> {
    let challenges = parse_mpp_challenges(www_authenticate);
    let target_chain = caip2_or_bare_chain_id(&payment.pay_network)?;

    // Find the tempo challenge for our chain id.
    let mut skipped = Vec::new();
    for challenge in &challenges {
        if challenge.method != "tempo" {
            skipped.push(format!("method={}", challenge.method));
            continue;
        }
        let request = match decode_b64url_json(&challenge.request) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let chain_id = request
            .pointer("/methodDetails/chainId")
            .and_then(Value::as_u64);
        if chain_id != Some(target_chain) {
            skipped.push(format!("tempo chainId={chain_id:?}"));
            continue;
        }
        return build_mpp_credential(payment, challenge, &request, target_chain);
    }
    Err(SdkError::PaymentUnsupported {
        offered: format!("MPP challenges: [{}]", skipped.join(", ")),
    })
}

#[cfg(not(feature = "payments-tempo"))]
fn authorize_mpp(
    _payment: &ResolvedPayment,
    _www_authenticate: &str,
) -> Result<Authorized, SdkError> {
    Err(SdkError::PaymentUnsupported {
        offered: "MPP/Tempo requires the `payments-tempo` feature".into(),
    })
}

#[cfg(feature = "payments-tempo")]
fn build_mpp_credential(
    payment: &ResolvedPayment,
    challenge: &MppChallenge,
    request: &Value,
    chain_id: u64,
) -> Result<Authorized, SdkError> {
    let recipient = request
        .get("recipient")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("MPP request missing recipient".into()))?;
    let currency = request
        .get("currency")
        .and_then(Value::as_str)
        .ok_or_else(|| SdkError::Config("MPP request missing currency".into()))?;
    let amount = request
        .get("amount")
        .and_then(Value::as_str)
        .and_then(|s| s.parse::<u128>().ok())
        .ok_or_else(|| SdkError::Config("MPP request missing/invalid amount".into()))?;

    if amount > payment.max_amount {
        return Err(SdkError::PaymentUnsupported {
            offered: format!(
                "MPP amount {amount} exceeds max_amount {}",
                payment.max_amount
            ),
        });
    }

    // validBefore = min(now+25s, challenge expiry) — TIP-1009 expiring nonce.
    // An unparseable expiry is an error, not an unbounded window: falling back
    // to u64::MAX would sign an authorization that never expires.
    let expiry = parse_iso_unix(&challenge.expires).ok_or_else(|| {
        SdkError::Config(format!(
            "MPP challenge has an unparseable `expires` value: {}",
            challenge.expires
        ))
    })?;
    let valid_before = (now_unix() + 25).min(expiry);

    let req = TempoChargeRequest {
        chain_id,
        currency: currency.to_string(),
        recipient: recipient.to_string(),
        amount,
        challenge_id: challenge.id.clone(),
        realm: challenge.realm.clone(),
        valid_before,
        gas_limit: None,
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    };
    let handoff = payment.signer.sign_tempo_tx(&req)?;
    let sender = payment.signer.address()?;

    let credential_json = serde_json::json!({
        "challenge": {
            "description": challenge.description,
            "expires": challenge.expires,
            "id": challenge.id,
            "intent": challenge.intent,
            "method": challenge.method,
            "realm": challenge.realm,
            "request": challenge.request,
        },
        "payload": { "signature": format!("0x{}", hex::encode(handoff)), "type": "transaction" },
        "source": format!("did:pkh:eip155:{chain_id}:{sender}"),
    });
    let credential = base64_url_nopad(serde_json::to_vec(&credential_json).map_err(|e| {
        SdkError::Config(format!(
            "could not serialize the MPP charge credential: {e}"
        ))
    })?);
    Ok(Authorized::Mpp { credential })
}

// One parsed MPP `Payment` challenge from the WWW-Authenticate header.
#[cfg(feature = "payments-tempo")]
#[derive(Debug, Clone)]
struct MppChallenge {
    id: String,
    realm: String,
    method: String,
    intent: String,
    description: String,
    expires: String,
    /// Original base64url request string (re-embedded verbatim in the credential).
    request: String,
}

// Split "Payment k1="v1", k2="v2", Payment ..." into challenge objects.
#[cfg(feature = "payments-tempo")]
fn parse_mpp_challenges(header: &str) -> Vec<MppChallenge> {
    let mut out = Vec::new();
    for part in split_payment_challenges(header) {
        let get = |key: &str| extract_quoted(&part, key).unwrap_or_default();
        let method = get("method");
        if method.is_empty() {
            continue;
        }
        out.push(MppChallenge {
            id: get("id"),
            realm: get("realm"),
            method,
            intent: get("intent"),
            description: get("description"),
            expires: get("expires"),
            request: get("request"),
        });
    }
    out
}

// Split on `Payment ` boundaries (at start or after a comma-space).
#[cfg(feature = "payments-tempo")]
pub(super) fn split_payment_challenges(header: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest = header.trim();
    // Strip a leading "Payment ".
    while let Some(idx) = rest.find("Payment ") {
        let after = &rest[idx + "Payment ".len()..];
        // Find the next ", Payment " boundary.
        if let Some(next) = after.find(", Payment ") {
            parts.push(after[..next].to_string());
            rest = &after[next + 2..]; // keep "Payment ..."
        } else {
            parts.push(after.to_string());
            break;
        }
    }
    parts
}

// Extract key="value" (values contain no escaped quotes in the challenge).
#[cfg(feature = "payments-tempo")]
pub(super) fn extract_quoted(part: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = part.find(&needle)? + needle.len();
    let end = part[start..].find('"')? + start;
    Some(part[start..end].to_string())
}

// ── Shared helpers ───────────────────────────────────────────────────────────

fn parse_receipt(header: &str) -> Option<PaymentReceipt> {
    let value = decode_b64url_json(header).ok()?;
    Some(PaymentReceipt {
        method: value.get("method").and_then(Value::as_str)?.to_string(),
        status: value.get("status").and_then(Value::as_str)?.to_string(),
        timestamp: value
            .get("timestamp")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        reference: value
            .get("reference")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
    })
}

pub(super) fn decode_b64url_json(s: &str) -> Result<Value, SdkError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s.trim_end_matches('='))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(s))
        .map_err(|_| SdkError::Config("invalid base64url payload".into()))?;
    serde_json::from_slice(&bytes).map_err(|source| SdkError::Decode {
        source,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    })
}

fn base64_std(bytes: Vec<u8>) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

// Only the MPP/Tempo credential builder uses this in non-test code; the
// receipt-parse test exercises it regardless of features.
#[cfg_attr(not(feature = "payments-tempo"), allow(dead_code))]
pub(super) fn base64_url_nopad(bytes: Vec<u8>) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

// eip155:84532 → 84532.
fn caip2_evm_chain_id(pay_network: &str) -> Result<u64, SdkError> {
    pay_network
        .strip_prefix("eip155:")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| {
            SdkError::Config(format!(
                "pay_network must be an eip155 CAIP-2 id for x402/EVM, got {pay_network:?}"
            ))
        })
}

// Accept either an eip155 CAIP-2 id or a bare numeric chain id (MPP/Tempo
// selectors are sometimes stated as the bare Tempo chain id). Only the MPP
// path uses this in non-test code.
#[cfg_attr(not(feature = "payments-tempo"), allow(dead_code))]
fn caip2_or_bare_chain_id(pay_network: &str) -> Result<u64, SdkError> {
    if let Some(rest) = pay_network.strip_prefix("eip155:") {
        return rest
            .parse()
            .map_err(|_| SdkError::Config(format!("invalid eip155 chain id: {pay_network:?}")));
    }
    pay_network.parse().map_err(|_| {
        SdkError::Config(format!(
            "pay_network must be an eip155 CAIP-2 id or a bare chain id, got {pay_network:?}"
        ))
    })
}

// When every candidate for the requested network+asset was rejected only for
// exceeding max_amount, the caller's ceiling is the single thing to change —
// so name the cheapest offer outright rather than leaving them to read it off
// the menu.
fn cheapest_over_ceiling(payment: &ResolvedPayment, accepts: &[Value]) -> Option<u128> {
    let mut cheapest: Option<u128> = None;
    for entry in accepts {
        let network = entry.get("network").and_then(Value::as_str).unwrap_or("");
        let asset = entry.get("asset").and_then(Value::as_str).unwrap_or("");
        if network != payment.pay_network || !asset.eq_ignore_ascii_case(&payment.asset) {
            continue;
        }
        if entry.pointer("/extra/name").and_then(Value::as_str) == Some(GATEWAY_BATCHED) {
            continue;
        }
        if let Ok(amount) = entry
            .get("amount")
            .and_then(Value::as_str)
            .unwrap_or("")
            .parse::<u128>()
        {
            if amount > payment.max_amount && cheapest.is_none_or(|c| amount < c) {
                cheapest = Some(amount);
            }
        }
    }
    cheapest
}

fn describe_offered(accepts: &[Value], skipped: &[String]) -> String {
    let offered: Vec<String> = accepts
        .iter()
        .filter_map(|e| {
            let network = e.get("network").and_then(Value::as_str)?;
            let asset = e.get("asset").and_then(Value::as_str)?;
            Some(format!("{network}/{asset}"))
        })
        .collect();
    if skipped.is_empty() {
        format!("[{}]", offered.join(", "))
    } else {
        format!(
            "[{}]; skipped: [{}]",
            offered.join(", "),
            skipped.join(", ")
        )
    }
}

// Append a clock-skew hint when a Tempo credential's window has already passed
// at response time — a skewed local clock (>~25s behind) signs already-expired
// credentials and every call ends in PaymentRejected.
fn enrich_rejection(payment: &ResolvedPayment, body: String) -> String {
    let out = reduce_rejection_body(body);
    if payment.signer.kind() == signer::ChainKind::Tempo {
        format!("{out} (if this persists, check the system clock — Tempo payment windows are ~25s)")
    } else {
        out
    }
}

/// Reduces the gateway's rejection body to its own reason when the body is the
/// JSON error shape (`{"error": ..., "message": ...}`), so the terse reason can
/// lead the caller's message. A body that isn't that shape (e.g. a full 402
/// payment menu, or plain text) is returned unchanged.
fn reduce_rejection_body(body: String) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
        let err = v.get("error").and_then(|e| e.as_str());
        let msg = v.get("message").and_then(|m| m.as_str());
        let reason = match (err, msg) {
            (Some(e), Some(m)) if e != m => format!("{e}: {m}"),
            (_, Some(m)) => m.to_string(),
            (Some(e), None) => e.to_string(),
            _ => String::new(),
        };
        if !reason.is_empty() {
            return reason;
        }
    }
    body
}

pub(super) fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

// Parse an ISO-8601 timestamp to unix seconds. The challenge uses
// "2026-07-13T02:05:10.119Z"; we only need whole seconds. Minimal parser to
// avoid a chrono dependency.
#[cfg(feature = "payments-tempo")]
pub(super) fn parse_iso_unix(iso: &str) -> Option<u64> {
    // Expect YYYY-MM-DDTHH:MM:SS...
    let bytes = iso.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let num = |a: usize, b: usize| iso.get(a..b)?.parse::<i64>().ok();
    let year = num(0, 4)?;
    let month = num(5, 7)?;
    let day = num(8, 10)?;
    let hour = num(11, 13)?;
    let min = num(14, 16)?;
    let sec = num(17, 19)?;
    // Days from civil (Howard Hinnant's algorithm).
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    let secs = days * 86400 + hour * 3600 + min * 60 + sec;
    u64::try_from(secs).ok()
}

pub(super) fn random_nonce() -> [u8; 32] {
    use rand::RngCore;
    let mut nonce = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn reduce_rejection_body_extracts_json_reason() {
        // error + message → "error: message".
        assert_eq!(
            reduce_rejection_body(
                r#"{"error":"auth_required","message":"SIWX authentication required"}"#.to_string()
            ),
            "auth_required: SIWX authentication required"
        );
        // message only.
        assert_eq!(
            reduce_rejection_body(r#"{"message":"insufficient funds"}"#.to_string()),
            "insufficient funds"
        );
        // A body that isn't the error shape (e.g. a payment menu) is unchanged.
        let menu = r#"{"accepts":[{"amount":"1000"}]}"#.to_string();
        assert_eq!(reduce_rejection_body(menu.clone()), menu);
        // Plain text is unchanged.
        assert_eq!(
            reduce_rejection_body("bad signature".to_string()),
            "bad signature"
        );
    }

    #[test]
    fn caip2_evm_parse() {
        assert_eq!(caip2_evm_chain_id("eip155:84532").unwrap(), 84532);
        assert!(caip2_evm_chain_id("solana:foo").is_err());
    }

    #[test]
    fn solana_devnet_detection_by_genesis_hash() {
        // CAIP-2 ids carry a genesis-hash prefix, never the literal "devnet".
        assert!(solana_pay_network_is_devnet(
            "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1"
        ));
        assert!(!solana_pay_network_is_devnet(
            "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"
        ));
        assert_eq!(
            default_solana_rpc("solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1"),
            "https://api.devnet.solana.com"
        );
        assert_eq!(
            default_solana_rpc("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp"),
            "https://api.mainnet-beta.solana.com"
        );
    }

    #[test]
    fn caip2_or_bare_parse() {
        assert_eq!(caip2_or_bare_chain_id("eip155:42431").unwrap(), 42431);
        assert_eq!(caip2_or_bare_chain_id("42431").unwrap(), 42431);
        assert!(caip2_or_bare_chain_id("solana:foo").is_err());
    }

    #[cfg(feature = "payments-tempo")]
    #[test]
    fn iso_to_unix() {
        // 2026-07-13T02:05:10Z — sanity check against a known value range.
        let t = parse_iso_unix("2026-07-13T02:05:10.119Z").unwrap();
        // 2026-07-13 is ~1.78e9 seconds after epoch.
        assert!((1_783_000_000..1_785_000_000).contains(&t), "got {t}");
    }

    #[cfg(feature = "payments-tempo")]
    #[test]
    fn mpp_multi_challenge_split() {
        let header = r#"Payment id="c1", realm="mpp.quicknode.com", method="tempo", intent="charge", description="d", expires="2026-07-13T02:05:10Z", request="eyJ4IjoxfQ", Payment id="c2", realm="mpp.quicknode.com", method="solana", intent="charge", description="d2", expires="2026-07-13T02:05:10Z", request="eyJ5IjoyfQ""#;
        let challenges = parse_mpp_challenges(header);
        assert_eq!(challenges.len(), 2);
        assert_eq!(challenges[0].method, "tempo");
        assert_eq!(challenges[0].id, "c1");
        assert_eq!(challenges[1].method, "solana");
    }

    #[test]
    fn receipt_parse_from_b64url() {
        let json = r#"{"method":"tempo","status":"success","timestamp":"2026-07-13T02:05:10.119Z","reference":"0xabc"}"#;
        let header = base64_url_nopad(json.as_bytes().to_vec());
        let receipt = parse_receipt(&header).unwrap();
        assert_eq!(receipt.method, "tempo");
        assert_eq!(receipt.reference, "0xabc");
    }

    // ── Driver wiremock tests ────────────────────────────────────────────────
    //
    // These exercise the 402 loop end-to-end against a mock gateway. Signing
    // correctness is covered byte-for-byte by the signer unit tests; here we
    // assert the parse → select → authorize → resend → capture flow.
    use secrecy::SecretString;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::matchers::{header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    // anvil key #0 (public throwaway, never funded).
    const EVM_KEY: &str = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    const USDC: &str = "0x036CbD53842c5426634e7929541eC2318f3dCF7e";

    fn evm_payment(base: &str, max_amount: u128) -> ResolvedPayment {
        ResolvedPayment {
            scheme: PaymentScheme::X402,
            signer: Signer::Evm(SecretString::new(EVM_KEY.to_string())),
            pay_network: "eip155:84532".into(),
            asset: USDC.into(),
            max_amount,
            base_url_override: Some(base.to_string()),
            svm_rpc_url: None,
        }
    }

    fn x402_accepts_entry(amount: &str, name: &str) -> Value {
        json!({
            "scheme": "exact",
            "network": "eip155:84532",
            "amount": amount,
            "payTo": "0x000000000000000000000000000000000000dEaD",
            "maxTimeoutSeconds": 60,
            "asset": USDC,
            "extra": { "name": name, "version": "2" }
        })
    }

    fn rpc_body() -> Value {
        json!({ "jsonrpc": "2.0", "id": 1, "method": "eth_blockNumber", "params": [] })
    }

    #[tokio::test]
    async fn x402_evm_happy_path() {
        let server = MockServer::start().await;
        // First (unpaid) POST -> 402 with a menu; the paid POST carries a
        // PAYMENT-SIGNATURE header and gets a 200 result.
        struct Seq {
            calls: AtomicUsize,
        }
        impl Respond for Seq {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                let has_sig = req.headers.contains_key("payment-signature");
                if n == 0 && !has_sig {
                    ResponseTemplate::new(402).set_body_json(json!({
                        "x402Version": 2,
                        "accepts": [ x402_accepts_entry("1000", "USDC") ]
                    }))
                } else {
                    ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc": "2.0", "id": 1, "result": "0x1335f9a"
                    }))
                }
            }
        }
        Mock::given(method("POST"))
            .and(path("/base-sepolia"))
            .respond_with(Seq {
                calls: AtomicUsize::new(0),
            })
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let (text, receipt) = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap();
        assert!(text.contains("0x1335f9a"));
        assert!(receipt.is_none()); // x402 has no receipt
    }

    #[tokio::test]
    async fn over_max_amount_is_unsupported() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(402).set_body_json(json!({
                "x402Version": 2,
                "accepts": [ x402_accepts_entry("999999", "USDC") ]
            })))
            .mount(&server)
            .await;

        // max_amount below the only offered entry.
        let payment = evm_payment(&server.uri(), 1000);
        let client = reqwest::Client::new();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("exceeds max_amount"))
        );
    }

    #[tokio::test]
    async fn gateway_wallet_batched_is_skipped() {
        let server = MockServer::start().await;
        // Only a GatewayWalletBatched entry is offered -> nothing to sign.
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(402).set_body_json(json!({
                "x402Version": 2,
                "accepts": [ x402_accepts_entry("1000", "GatewayWalletBatched") ]
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("GatewayWalletBatched"))
        );
    }

    #[tokio::test]
    async fn non_integer_amount_is_skipped() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(402).set_body_json(json!({
                "x402Version": 2,
                "accepts": [ x402_accepts_entry("0.001", "USDC") ]
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentUnsupported { offered } if offered.contains("not an integer"))
        );
    }

    #[tokio::test]
    async fn huge_amount_over_u64_compares_correctly() {
        let server = MockServer::start().await;
        // An 18-decimal asset amount that overflows u64 but fits u128, below a
        // large max_amount -> must be selectable (proves u128 comparison).
        let huge = "20000000000000000000"; // 2e19 > u64::MAX (~1.8e19)
        struct Seq {
            calls: AtomicUsize,
        }
        impl Respond for Seq {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 && !req.headers.contains_key("payment-signature") {
                    ResponseTemplate::new(402).set_body_json(json!({
                        "x402Version": 2,
                        "accepts": [ x402_accepts_entry("20000000000000000000", "USDC") ]
                    }))
                } else {
                    ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc": "2.0", "id": 1, "result": "0xok"
                    }))
                }
            }
        }
        Mock::given(method("POST"))
            .respond_with(Seq {
                calls: AtomicUsize::new(0),
            })
            .mount(&server)
            .await;
        let _ = huge;

        let payment = evm_payment(&server.uri(), 30_000_000_000_000_000_000u128);
        let client = reqwest::Client::new();
        let (text, _) = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap();
        assert!(text.contains("0xok"));
    }

    #[tokio::test]
    async fn second_402_is_terminal_rejection() {
        let server = MockServer::start().await;
        // Every POST returns 402 -> the paid resend also 402s -> PaymentRejected.
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(402).set_body_json(json!({
                "x402Version": 2,
                "accepts": [ x402_accepts_entry("1000", "USDC") ]
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::PaymentRejected { status, .. } if status == 402));
    }

    #[tokio::test]
    async fn malformed_challenge_menu_is_unsupported_not_decode() {
        let server = MockServer::start().await;
        // The 402 challenge body is not JSON. Nothing has been signed, so this
        // must surface as PaymentUnsupported (nothing charged), never Decode.
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(402).set_body_string("<html>menu?</html>"))
            .expect(1)
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::PaymentUnsupported { offered } if offered.contains("unparseable")),
            "expected PaymentUnsupported, got {err:?}"
        );
    }

    #[tokio::test]
    async fn gateway_5xx_on_paid_resend_is_rejection_not_decode() {
        // The unpaid probe 402s; the paid resend returns a 500 with a non-JSON
        // body. This must surface as PaymentRejected (payment was submitted),
        // NOT fall through to a Decode error.
        let server = MockServer::start().await;
        struct Seq {
            calls: AtomicUsize,
        }
        impl Respond for Seq {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 && !req.headers.contains_key("payment-signature") {
                    ResponseTemplate::new(402).set_body_json(json!({
                        "x402Version": 2,
                        "accepts": [ x402_accepts_entry("1000", "USDC") ]
                    }))
                } else {
                    ResponseTemplate::new(500).set_body_string("upstream settlement error")
                }
            }
        }
        Mock::given(method("POST"))
            .respond_with(Seq {
                calls: AtomicUsize::new(0),
            })
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::PaymentRejected { status, body } if *status == 500 && body.contains("settlement error")),
            "expected PaymentRejected(500), got {err:?}"
        );
    }

    #[tokio::test]
    async fn paid_resend_sends_exactly_one_credential() {
        // Assert the paid resend carries PAYMENT-SIGNATURE and the flow stops
        // after one resend (mock counts total POSTs = 2).
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(header_exists("payment-signature"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": 1, "result": "0xpaid"
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(402).set_body_json(json!({
                "x402Version": 2,
                "accepts": [ x402_accepts_entry("1000", "USDC") ]
            })))
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::new();
        let (text, _) = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap();
        assert!(text.contains("0xpaid"));
    }

    #[tokio::test]
    async fn lost_response_after_payment_is_indeterminate() {
        // The paid resend times out (mock delays past the client timeout) AFTER
        // the request was sent -> PaymentIndeterminate (do not blind-retry).
        let server = MockServer::start().await;
        struct Seq {
            calls: AtomicUsize,
        }
        impl Respond for Seq {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 && !req.headers.contains_key("payment-signature") {
                    ResponseTemplate::new(402).set_body_json(json!({
                        "x402Version": 2,
                        "accepts": [ x402_accepts_entry("1000", "USDC") ]
                    }))
                } else {
                    // Delay well past the client timeout to simulate a lost
                    // response after the paid bytes were sent.
                    ResponseTemplate::new(200)
                        .set_delay(std::time::Duration::from_secs(30))
                        .set_body_json(json!({ "jsonrpc": "2.0", "id": 1, "result": "0xlate" }))
                }
            }
        }
        Mock::given(method("POST"))
            .respond_with(Seq {
                calls: AtomicUsize::new(0),
            })
            .mount(&server)
            .await;

        let payment = evm_payment(&server.uri(), 10_000);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(300))
            .build()
            .unwrap();
        let err = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap_err();
        assert!(
            matches!(err, SdkError::PaymentIndeterminate),
            "expected PaymentIndeterminate, got {err:?}"
        );
    }

    #[cfg(feature = "payments-tempo")]
    #[tokio::test]
    async fn mpp_happy_path_captures_receipt() {
        let server = MockServer::start().await;
        // The tempo challenge request (base64url JSON) for chain 42431.
        let request = base64_url_nopad(
            serde_json::to_vec(&json!({
                "amount": "1000",
                "currency": "0x20c0000000000000000000000000000000000000",
                "recipient": "0xfd24114c3981aba78ae2441991b1bdb89329c556",
                "methodDetails": { "chainId": 42431, "feePayer": true }
            }))
            .unwrap(),
        );
        let www = format!(
            "Payment id=\"c1\", realm=\"mpp.quicknode.com\", method=\"tempo\", intent=\"charge\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", request=\"{request}\""
        );
        let receipt = base64_url_nopad(
            serde_json::to_vec(&json!({
                "method": "tempo", "status": "success",
                "timestamp": "2026-07-13T02:05:10.119Z",
                "reference": "0xdeadbeef"
            }))
            .unwrap(),
        );

        struct Seq {
            www: String,
            receipt: String,
            calls: AtomicUsize,
        }
        impl Respond for Seq {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 && !req.headers.contains_key("authorization") {
                    ResponseTemplate::new(402)
                        .insert_header("WWW-Authenticate", self.www.as_str())
                        .set_body_json(json!({ "type": "about:blank" }))
                } else {
                    ResponseTemplate::new(200)
                        .insert_header("Payment-Receipt", self.receipt.as_str())
                        .set_body_json(json!({ "jsonrpc": "2.0", "id": 1, "result": "0xok" }))
                }
            }
        }
        Mock::given(method("POST"))
            .respond_with(Seq {
                www,
                receipt,
                calls: AtomicUsize::new(0),
            })
            .mount(&server)
            .await;

        let payment = ResolvedPayment {
            scheme: PaymentScheme::MppCharge,
            signer: Signer::Tempo(SecretString::new(EVM_KEY.to_string())),
            pay_network: "eip155:42431".into(),
            asset: "0x20c0000000000000000000000000000000000000".into(),
            max_amount: 10_000,
            base_url_override: Some(server.uri()),
            svm_rpc_url: None,
        };
        let client = reqwest::Client::new();
        let (text, receipt) = pay_and_call(&client, &payment, "base-sepolia", &rpc_body())
            .await
            .unwrap();
        assert!(text.contains("0xok"));
        let receipt = receipt.expect("MPP happy path must capture a receipt");
        assert_eq!(receipt.method, "tempo");
        assert_eq!(receipt.reference, "0xdeadbeef");
    }

    // The menu selector compares amounts as u128, but SPL TransferChecked can
    // only encode a u64. An amount the selector admits but that overflows u64
    // must fail with a clear overflow message, not a vague "missing amount".
    #[cfg(feature = "payments-svm")]
    #[tokio::test]
    async fn x402_svm_amount_over_u64_is_clear_error() {
        let over_u64 = (u128::from(u64::MAX) + 1).to_string();
        let entry = json!({
            "scheme": "exact",
            "network": "solana:mainnet",
            "amount": over_u64,
            "payTo": "11111111111111111111111111111112",
            "asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "extra": { "feePayer": "11111111111111111111111111111112", "decimals": 6 }
        });
        let payment = ResolvedPayment {
            scheme: PaymentScheme::X402,
            signer: Signer::Svm(SecretString::new(EVM_KEY.to_string())),
            pay_network: "solana:mainnet".into(),
            asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
            max_amount: u128::MAX,
            base_url_override: None,
            svm_rpc_url: Some("http://127.0.0.1:1".into()),
        };
        let client = reqwest::Client::new();
        // The amount check runs before any Solana RPC read, so the unreachable
        // svm_rpc_url is never contacted.
        let Err(err) = authorize_x402_svm(&client, &payment, &2, &entry).await else {
            unreachable!("over-u64 amount must be rejected");
        };
        let msg = err.to_string();
        assert!(
            msg.contains("not a valid u64"),
            "expected u64 overflow error, got: {msg}"
        );
    }

    // Solana's menu distinguishes its tiers only by amount — no `extra.name` on
    // either entry — and advertises the dearer one FIRST. Taking the first match
    // lands on a tier the per-request lane cannot pay, so selection must pick
    // the cheapest that fits the ceiling regardless of menu order.
    fn solana_menu() -> Vec<Value> {
        let offer = |amount: &str| {
            json!({
                "scheme": "exact",
                "network": "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1",
                "amount": amount,
                "payTo": "2LWbc9Mi6dRUrdEHBttoNS4udDtH1A4xwBdm1EKqcT57",
                "maxTimeoutSeconds": 60,
                "asset": "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
                "extra": { "feePayer": "CPZSjRmyfTS95UjQD8ZdeTEWbQvW9QvEXnn6aGP7yyMN" }
            })
        };
        vec![offer("1000000"), offer("1000")]
    }

    fn solana_payment(max_amount: u128) -> ResolvedPayment {
        ResolvedPayment {
            scheme: PaymentScheme::X402,
            signer: Signer::Svm(SecretString::new(EVM_KEY.to_string())),
            pay_network: "solana:EtWTRABZaYq6iMfeYKouRu166VU2xqa1".into(),
            asset: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".into(),
            max_amount,
            base_url_override: None,
            svm_rpc_url: Some("http://127.0.0.1:1".into()),
        }
    }

    #[test]
    fn select_prefers_the_cheapest_offer_over_menu_order() {
        let payment = solana_payment(1_000_000);
        let mut skipped = Vec::new();
        let chosen = select_x402_entry(&payment, &solana_menu(), &mut skipped)
            .expect("a matching offer exists");
        assert_eq!(
            chosen.get("amount").and_then(Value::as_str),
            Some("1000"),
            "must pick the cheapest, not the first listed"
        );
    }

    #[test]
    fn select_skips_offers_over_the_ceiling() {
        // A ceiling between the two tiers admits only the cheaper one.
        let payment = solana_payment(2_000);
        let mut skipped = Vec::new();
        let chosen = select_x402_entry(&payment, &solana_menu(), &mut skipped)
            .expect("the cheap offer fits");
        assert_eq!(chosen.get("amount").and_then(Value::as_str), Some("1000"));
        assert_eq!(skipped.len(), 1, "the dearer offer is reported as skipped");
        assert!(
            skipped[0].contains("exceeds max_amount"),
            "got: {skipped:?}"
        );
    }

    // When the ceiling is under every offer, the caller's one lever is
    // max_amount — so the error names the cheapest price outright.
    #[test]
    fn ceiling_under_every_offer_names_the_cheapest() {
        let payment = solana_payment(100);
        let mut skipped = Vec::new();
        assert!(select_x402_entry(&payment, &solana_menu(), &mut skipped).is_none());
        assert_eq!(cheapest_over_ceiling(&payment, &solana_menu()), Some(1_000));
    }
}
