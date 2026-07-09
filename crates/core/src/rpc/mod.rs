//! Data-plane JSON-RPC client.
//!
//! Makes JSON-RPC calls directly against the account's provisioned Tooling
//! Access endpoint, authenticating with a short-lived session JWT. The JWT is
//! minted via the Admin control plane ([`crate::admin::AdminApiClient::mint_tooling_token`]),
//! cached in memory, and refreshed proactively before expiry (or reactively on
//! a 401). The signing key never leaves the server; this client only ever holds
//! a minted JWT.
//!
//! A host that outlives a single process (e.g. the CLI) can persist the cached
//! token between runs by seeding [`crate::config::RpcConfig::seed`] on startup
//! and snapshotting [`RpcApiClient::current_token`] afterwards.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::admin::AdminApiClient;
use crate::config::{CachedToken, RpcConfig};
use crate::errors::SdkError;
use crate::SdkConfig;

// Default seconds before `exp` at which we proactively refresh. Also absorbs
// clock skew between client and endpoint.
const DEFAULT_REFRESH_MARGIN_SECS: i64 = 60;

/// JSON-RPC client for the Tooling Access endpoint.
#[derive(Clone)]
pub struct RpcApiClient {
    // Used to mint/refresh session tokens against the control plane.
    admin: AdminApiClient,
    config: SdkConfig,
    refresh_margin_secs: i64,
    // Current cached token. Guarded by a std Mutex held only for synchronous
    // read/write — never across an await.
    cache: Arc<Mutex<Option<CachedToken>>>,
    // Serializes refreshes so concurrent callers that all see an expired token
    // trigger a single mint, not a stampede. Held across the mint await, hence
    // an async mutex.
    refresh_lock: Arc<tokio::sync::Mutex<()>>,
    // Per-network URL map for multichain routing: key (e.g. "solana-mainnet")
    // -> full http_url. The endpoint is multichain by subdomain and the URLs
    // are not derivable by string munging, so callers seed this map (from
    // `admin.get_endpoint_urls`). `None` until seeded; a `call` with a network
    // then errors with a clear message.
    networks: Arc<Mutex<Option<HashMap<String, String>>>>,
    // Client-wide default custom endpoint URL. When set, calls bypass the
    // Tooling Access endpoint and the JWT entirely (see `RpcConfig::endpoint_url`).
    // A per-call `endpoint_url` overrides this. Immutable after construction.
    endpoint_url: Option<String>,
}

impl std::fmt::Debug for RpcApiClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never print the cached JWT.
        f.debug_struct("RpcApiClient")
            .field("refresh_margin_secs", &self.refresh_margin_secs)
            .field(
                "has_cached_token",
                &self.cache.lock().is_ok_and(|c| c.is_some()),
            )
            .finish()
    }
}

impl RpcApiClient {
    pub fn new(config: SdkConfig, rpc_config: Option<&RpcConfig>) -> Self {
        let refresh_margin_secs = rpc_config
            .and_then(|c| c.refresh_margin_secs)
            .filter(|&m| m >= 0)
            .unwrap_or(DEFAULT_REFRESH_MARGIN_SECS);
        // Seed is advisory: a stale/expired seed simply produces a cache miss on
        // the first call and is replaced by a fresh mint.
        let seed = rpc_config.and_then(|c| c.seed.clone());
        let networks = rpc_config.and_then(|c| c.networks.clone());
        let endpoint_url = rpc_config.and_then(|c| c.endpoint_url.clone());
        Self {
            admin: AdminApiClient::new(config.clone()),
            config,
            refresh_margin_secs,
            cache: Arc::new(Mutex::new(seed)),
            refresh_lock: Arc::new(tokio::sync::Mutex::new(())),
            networks: Arc::new(Mutex::new(networks)),
            endpoint_url,
        }
    }

    /// Seeds (or replaces) the per-network URL map used for multichain routing.
    /// The map is `network key -> full http_url`, typically built from
    /// `admin.get_endpoint_urls(endpoint_id).multichain_urls`. A host that
    /// didn't seed it via [`RpcConfig`] can install it here before calling with
    /// a `network`.
    pub fn set_networks(&self, networks: HashMap<String, String>) {
        if let Ok(mut guard) = self.networks.lock() {
            *guard = Some(networks);
        }
    }

    /// Returns a snapshot of the current cached token, if any. Hosts use this to
    /// persist the token between processes. Returns `None` if no token has been
    /// minted (or seeded) yet.
    pub fn current_token(&self) -> Option<CachedToken> {
        self.cache.lock().ok().and_then(|c| c.clone())
    }

    /// Discards the in-memory cached token, forcing the next call to mint a
    /// fresh one. Use when the cached token is known stale beyond expiry — e.g.
    /// the endpoint was disabled and re-enabled out of band.
    pub fn clear_cached_token(&self) {
        self.invalidate();
    }

    /// Makes a JSON-RPC call. `params` defaults to an empty array when `None`;
    /// it accepts both a positional array and a by-name object.
    ///
    /// `endpoint_url` sends this call to a custom HTTP URL, bypassing the
    /// Tooling Access endpoint and the session JWT entirely — the URL is treated
    /// as self-authenticating and gets no Authorization header. It overrides the
    /// client-wide [`RpcConfig::endpoint_url`] default for this call. Because a
    /// custom URL is not multichain-routed, passing both `endpoint_url` and
    /// `network` is a [`SdkError::Config`] error.
    ///
    /// `network` selects which chain to route to on a multichain endpoint: it
    /// is a key in the seeded network map (e.g. `"solana-mainnet"`, `"polygon"`).
    /// When `None`, the call goes to the endpoint's default network. When `Some`,
    /// the map must be seeded (via [`RpcConfig`] or [`Self::set_networks`]) and
    /// contain the key, otherwise a [`SdkError::Config`] is returned.
    ///
    /// Returns the unwrapped `result`. A JSON-RPC `error` member is surfaced as
    /// [`SdkError::Rpc`].
    pub async fn call(
        &self,
        method: &str,
        params: Option<Value>,
        network: Option<String>,
        endpoint_url: Option<String>,
    ) -> Result<Value, SdkError> {
        // Precedence: a per-call custom URL wins; then a per-call network; then
        // the client-wide custom URL default; then the tooling default endpoint.
        // A per-call URL and network are mutually exclusive (custom URLs are not
        // multichain-routed).
        if endpoint_url.is_some() && network.is_some() {
            return Err(SdkError::Config(
                "`endpoint_url` and `network` are mutually exclusive: a custom \
                 URL is not multichain-routed"
                    .into(),
            ));
        }
        let custom_url = endpoint_url.or_else(|| self.endpoint_url.clone());

        // Custom mode: no token minted or attached; the URL authenticates itself.
        // There is no JWT to refresh, so no reactive-401 retry path.
        if let Some(url) = custom_url {
            let resp = self.send(None, &url, method, &params).await?;
            return Self::parse_rpc(resp);
        }

        // Tooling mode: mint/refresh the JWT and route via the token/network map.
        let token = self.valid_token().await?;
        let url = self.resolve_url(&token, network.as_deref())?;
        let resp = self.send(Some(&token), &url, method, &params).await?;

        // Reactive refresh: a 401 means the token was rejected (expired at the
        // edge, revoked, clock skew past the margin). Discard, mint once, retry
        // once. A second 401 surfaces as an Api error.
        if resp.status == 401 {
            self.invalidate();
            let token = self.refresh().await?;
            let url = self.resolve_url(&token, network.as_deref())?;
            let retry = self.send(Some(&token), &url, method, &params).await?;
            return Self::parse_rpc(retry);
        }
        Self::parse_rpc(resp)
    }

    // Resolve the target URL for a call. `None` network -> the token's default
    // endpoint_url. `Some(key)` -> the mapped per-network URL; errors if no map
    // is seeded or the key is unknown (listing available keys).
    fn resolve_url(&self, token: &CachedToken, network: Option<&str>) -> Result<String, SdkError> {
        let Some(key) = network else {
            return Ok(token.endpoint_url.clone());
        };
        let guard = self
            .networks
            .lock()
            .map_err(|_| SdkError::Config("network map lock poisoned".into()))?;
        let Some(map) = guard.as_ref() else {
            return Err(SdkError::Config(format!(
                "network '{key}' requested but no network map is available; \
                 seed it via RpcConfig.networks or set_networks()"
            )));
        };
        match map.get(key) {
            Some(url) => Ok(url.clone()),
            None => {
                let mut keys: Vec<&str> = map.keys().map(String::as_str).collect();
                keys.sort_unstable();
                Err(SdkError::Config(format!(
                    "unknown network '{key}'. Available: {}",
                    keys.join(", ")
                )))
            }
        }
    }

    // ── Token lifecycle ──────────────────────────────────────────────────────

    // Returns a token that is valid past the refresh margin, minting if needed.
    async fn valid_token(&self) -> Result<CachedToken, SdkError> {
        if let Some(tok) = self.cached_if_fresh() {
            return Ok(tok);
        }
        self.refresh().await
    }

    // Returns the cached token only if present and not within the refresh margin.
    fn cached_if_fresh(&self) -> Option<CachedToken> {
        let now = now_unix();
        let guard = self.cache.lock().ok()?;
        guard
            .as_ref()
            .filter(|t| now + self.refresh_margin_secs < t.exp_unix)
            .cloned()
    }

    // Single-flight refresh: only one caller mints at a time; others re-check
    // the cache after acquiring the lock and reuse the just-minted token.
    async fn refresh(&self) -> Result<CachedToken, SdkError> {
        let _guard = self.refresh_lock.lock().await;
        // Another caller may have refreshed while we waited for the lock.
        if let Some(tok) = self.cached_if_fresh() {
            return Ok(tok);
        }
        let fresh = self.admin.mint_tooling_token().await?;
        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(fresh.clone());
        }
        Ok(fresh)
    }

    fn invalidate(&self) {
        if let Ok(mut guard) = self.cache.lock() {
            *guard = None;
        }
    }

    // ── Transport ─────────────────────────────────────────────────────────────

    // Sends the JSON-RPC request. `token` is `Some` in tooling mode (attaches a
    // Bearer JWT) and `None` for a custom endpoint URL, which is treated as
    // self-authenticating and gets no Authorization header. Either way the
    // request goes through the keyless `rpc_http_client`, so the account
    // `x-api-key` never reaches the data plane.
    async fn send(
        &self,
        token: Option<&CachedToken>,
        target_url: &str,
        method: &str,
        params: &Option<Value>,
    ) -> Result<RawResponse, SdkError> {
        let url = reqwest::Url::parse(target_url).map_err(|e| SdkError::Config(e.to_string()))?;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params.clone().unwrap_or_else(|| Value::Array(vec![])),
        });
        let mut req = self.config.rpc_http_client().post(url).json(&body);
        if let Some(token) = token {
            req = req.bearer_auth(&token.token);
        }
        let resp = req.send().await.map_err(SdkError::Http)?;
        let status = resp.status().as_u16();
        let text = resp.text().await.map_err(SdkError::Http)?;
        Ok(RawResponse { status, text })
    }

    // Parse a JSON-RPC envelope: surface `error` as SdkError::Rpc, else return
    // `result`. Non-2xx HTTP without a usable JSON-RPC body is an Api error.
    fn parse_rpc(resp: RawResponse) -> Result<Value, SdkError> {
        // Try to decode the JSON-RPC envelope regardless of HTTP status — some
        // endpoints return a JSON-RPC error with a 200, others with 4xx.
        let parsed: Result<JsonRpcEnvelope, _> = serde_json::from_str(&resp.text);
        match parsed {
            Ok(env) => {
                if let Some(err) = env.error {
                    return Err(SdkError::Rpc {
                        code: err.code,
                        message: err.message,
                    });
                }
                if let Some(result) = env.result {
                    return Ok(result);
                }
                // No result and no error: if the HTTP status was a failure,
                // surface it; otherwise return null.
                if !(200..300).contains(&resp.status) {
                    return Err(SdkError::Api {
                        status: status_code(resp.status),
                        body: resp.text,
                    });
                }
                Ok(Value::Null)
            }
            Err(source) => {
                if !(200..300).contains(&resp.status) {
                    Err(SdkError::Api {
                        status: status_code(resp.status),
                        body: resp.text,
                    })
                } else {
                    Err(SdkError::Decode {
                        source,
                        body: resp.text,
                    })
                }
            }
        }
    }
}

struct RawResponse {
    status: u16,
    text: String,
}

#[derive(serde::Deserialize)]
struct JsonRpcEnvelope {
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(serde::Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        // Pre-epoch system clock is implausible; treat as 0 so a fresh token is
        // always considered valid rather than panicking.
        .unwrap_or(0)
}

fn status_code(status: u16) -> reqwest::StatusCode {
    reqwest::StatusCode::from_u16(status).unwrap_or(reqwest::StatusCode::BAD_GATEWAY)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::config::{AdminConfig, SdkFullConfig};
    use crate::QuicknodeSdk;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

    // A future exp so seeded tokens are considered fresh.
    fn future_exp() -> i64 {
        now_unix() + 3600
    }

    fn token_body(endpoint_url: &str, exp: i64) -> serde_json::Value {
        // The mint route returns an ISO timestamp; build one far in the future.
        // We feed exp directly via seed in most tests, but mint tests use this.
        let _ = exp;
        serde_json::json!({
            "data": {
                "endpoint_url": endpoint_url,
                "token": "minted.jwt.value",
                "expires_at": "2099-01-01T00:00:00.000Z"
            },
            "error": null
        })
    }

    fn sdk_with_seed(admin_base: &str, rpc_endpoint: &str) -> QuicknodeSdk {
        let mut cfg = SdkFullConfig::from_api_key("test-key".to_string());
        cfg.admin = Some(AdminConfig {
            base_url: Some(format!("{admin_base}/")),
        });
        cfg.rpc = Some(RpcConfig {
            endpoint_url: None,
            seed: Some(CachedToken {
                endpoint_url: rpc_endpoint.to_string(),
                token: "seeded.jwt".to_string(),
                exp_unix: future_exp(),
            }),
            refresh_margin_secs: None,
            networks: None,
        });
        QuicknodeSdk::new(&cfg).unwrap()
    }

    #[tokio::test]
    async fn call_uses_seed_without_minting() {
        let server = MockServer::start().await;
        // RPC endpoint returns a result.
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(
                serde_json::json!({ "method": "eth_blockNumber" }),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": "0x1335f9a"
            })))
            .mount(&server)
            .await;

        // Use the same server for both admin and rpc; if mint were called it
        // would 404 (no mock for /tooling-access/token) and the test would fail.
        let sdk = sdk_with_seed(&server.uri(), &server.uri());
        let result = sdk
            .rpc
            .call("eth_blockNumber", None, None, None)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("0x1335f9a"));
    }

    #[tokio::test]
    async fn call_sends_bearer_jwt_but_not_account_api_key() {
        let server = MockServer::start().await;
        // Match only requests that carry the Bearer JWT and omit the account
        // key: the data-plane client must never leak `x-api-key`. If the key
        // were present this mock would not match and the call would 404.
        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("authorization", "Bearer seeded.jwt"))
            .and(|req: &Request| !req.headers.contains_key("x-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": "0xok"
            })))
            .mount(&server)
            .await;

        let sdk = sdk_with_seed(&server.uri(), &server.uri());
        let result = sdk
            .rpc
            .call("eth_blockNumber", None, None, None)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("0xok"));
    }

    // Builds an SDK whose RPC client has a client-wide custom `endpoint_url` and
    // NO seed. The admin base points at a dead address, so any attempt to mint a
    // tooling token would fail — proving custom mode never touches the JWT path.
    fn sdk_with_custom_url(endpoint_url: &str) -> QuicknodeSdk {
        let mut cfg = SdkFullConfig::from_api_key("test-key".to_string());
        cfg.admin = Some(AdminConfig {
            base_url: Some("http://127.0.0.1:1/".to_string()),
        });
        cfg.rpc = Some(RpcConfig {
            endpoint_url: Some(endpoint_url.to_string()),
            seed: None,
            refresh_margin_secs: None,
            networks: None,
        });
        QuicknodeSdk::new(&cfg).unwrap()
    }

    #[tokio::test]
    async fn config_endpoint_url_bypasses_jwt_and_minting() {
        let server = MockServer::start().await;
        // Custom endpoint must receive the call with NO Authorization header and
        // NO account key. If minting were attempted it would fail against the
        // dead admin base and the call would error instead.
        Mock::given(method("POST"))
            .and(path("/custom"))
            .and(|req: &Request| {
                !req.headers.contains_key("authorization") && !req.headers.contains_key("x-api-key")
            })
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": "0xcustom"
            })))
            .mount(&server)
            .await;

        let sdk = sdk_with_custom_url(&format!("{}/custom", server.uri()));
        let result = sdk
            .rpc
            .call("eth_blockNumber", None, None, None)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("0xcustom"));
        // No token was ever minted or cached.
        assert!(sdk.rpc.current_token().is_none());
    }

    #[tokio::test]
    async fn per_call_endpoint_url_overrides_config_default() {
        let server = MockServer::start().await;
        // The per-call URL points here; the config default points at /wrong,
        // which has no mock and would 404.
        Mock::given(method("POST"))
            .and(path("/override"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": "0xoverride"
            })))
            .mount(&server)
            .await;

        let sdk = sdk_with_custom_url(&format!("{}/wrong", server.uri()));
        let result = sdk
            .rpc
            .call(
                "eth_blockNumber",
                None,
                None,
                Some(format!("{}/override", server.uri())),
            )
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("0xoverride"));
    }

    #[tokio::test]
    async fn endpoint_url_and_network_together_is_config_error() {
        let sdk = sdk_with_custom_url("https://example.invalid/rpc");
        let err = sdk
            .rpc
            .call(
                "eth_blockNumber",
                None,
                Some("solana-mainnet".to_string()),
                Some("https://example.invalid/other".to_string()),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Config(msg) if msg.contains("mutually exclusive")));
    }

    #[tokio::test]
    async fn json_rpc_error_maps_to_rpc_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1,
                "error": { "code": -32602, "message": "invalid params" }
            })))
            .mount(&server)
            .await;

        let sdk = sdk_with_seed(&server.uri(), &server.uri());
        let err = sdk
            .rpc
            .call("eth_getBalance", None, None, None)
            .await
            .unwrap_err();
        match err {
            SdkError::Rpc { code, message } => {
                assert_eq!(code, -32602);
                assert!(message.contains("invalid params"));
            }
            other => panic!("expected Rpc error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn reactive_401_refreshes_and_retries_once() {
        let server = MockServer::start().await;

        // First RPC call returns 401, second (after refresh) returns a result.
        struct Sequence {
            calls: AtomicUsize,
        }
        impl Respond for Sequence {
            fn respond(&self, _: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    ResponseTemplate::new(401).set_body_string("unauthorized")
                } else {
                    ResponseTemplate::new(200).set_body_json(serde_json::json!({
                        "jsonrpc": "2.0", "id": 1, "result": "0xokay"
                    }))
                }
            }
        }

        // RPC endpoint lives at /rpc; mint route at /tooling-access/token.
        Mock::given(method("POST"))
            .and(path("/rpc"))
            .respond_with(Sequence {
                calls: AtomicUsize::new(0),
            })
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/tooling-access/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(token_body(&format!("{}/rpc", server.uri()), future_exp())),
            )
            .mount(&server)
            .await;

        let sdk = sdk_with_seed(&server.uri(), &format!("{}/rpc", server.uri()));
        let result = sdk
            .rpc
            .call("eth_blockNumber", None, None, None)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("0xokay"));
    }

    #[tokio::test]
    async fn second_401_surfaces_as_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/rpc"))
            .respond_with(ResponseTemplate::new(401).set_body_string("nope"))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/tooling-access/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(token_body(&format!("{}/rpc", server.uri()), future_exp())),
            )
            .mount(&server)
            .await;

        let sdk = sdk_with_seed(&server.uri(), &format!("{}/rpc", server.uri()));
        let err = sdk
            .rpc
            .call("eth_blockNumber", None, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { status, .. } if status.as_u16() == 401));
    }

    #[tokio::test]
    async fn expired_seed_triggers_mint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/tooling-access/token"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(token_body(&format!("{}/rpc", server.uri()), future_exp())),
            )
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/rpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": "0xfresh"
            })))
            .mount(&server)
            .await;

        // Seed an already-expired token.
        let mut cfg = SdkFullConfig::from_api_key("test-key".to_string());
        cfg.admin = Some(AdminConfig {
            base_url: Some(format!("{}/", server.uri())),
        });
        cfg.rpc = Some(RpcConfig {
            endpoint_url: None,
            seed: Some(CachedToken {
                endpoint_url: format!("{}/rpc", server.uri()),
                token: "expired.jwt".to_string(),
                exp_unix: now_unix() - 10,
            }),
            refresh_margin_secs: None,
            networks: None,
        });
        let sdk = QuicknodeSdk::new(&cfg).unwrap();

        let result = sdk
            .rpc
            .call("eth_blockNumber", None, None, None)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("0xfresh"));
        // current_token now reflects the minted token.
        assert_eq!(sdk.rpc.current_token().unwrap().token, "minted.jwt.value");
    }

    #[tokio::test]
    async fn network_routes_to_mapped_url() {
        let server = MockServer::start().await;
        // The default endpoint is /default; the "solana-mainnet" network maps to
        // /solana. A call with that network must POST to /solana, not /default.
        Mock::given(method("POST"))
            .and(path("/solana"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": "12345"
            })))
            .mount(&server)
            .await;

        let mut cfg = SdkFullConfig::from_api_key("test-key".to_string());
        cfg.admin = Some(AdminConfig {
            base_url: Some(format!("{}/", server.uri())),
        });
        let mut networks = std::collections::HashMap::new();
        networks.insert(
            "solana-mainnet".to_string(),
            format!("{}/solana", server.uri()),
        );
        cfg.rpc = Some(RpcConfig {
            endpoint_url: None,
            seed: Some(CachedToken {
                endpoint_url: format!("{}/default", server.uri()),
                token: "seeded.jwt".to_string(),
                exp_unix: future_exp(),
            }),
            refresh_margin_secs: None,
            networks: Some(networks),
        });
        let sdk = QuicknodeSdk::new(&cfg).unwrap();

        let result = sdk
            .rpc
            .call("getSlot", None, Some("solana-mainnet".to_string()), None)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!("12345"));
    }

    #[tokio::test]
    async fn unknown_network_is_config_error_listing_keys() {
        let server = MockServer::start().await;
        let sdk = sdk_with_seed(&server.uri(), &server.uri());
        // sdk_with_seed seeds no network map.
        sdk.rpc.set_networks(std::collections::HashMap::from([(
            "solana-mainnet".to_string(),
            "https://x/solana".to_string(),
        )]));
        let err = sdk
            .rpc
            .call("getSlot", None, Some("polygon".to_string()), None)
            .await
            .unwrap_err();
        match err {
            SdkError::Config(msg) => {
                assert!(msg.contains("unknown network 'polygon'"), "msg: {msg}");
                assert!(
                    msg.contains("solana-mainnet"),
                    "msg should list keys: {msg}"
                );
            }
            other => panic!("expected Config error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn network_without_seeded_map_errors() {
        let server = MockServer::start().await;
        let sdk = sdk_with_seed(&server.uri(), &server.uri());
        let err = sdk
            .rpc
            .call("getSlot", None, Some("solana-mainnet".to_string()), None)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Config(msg) if msg.contains("no network map")));
    }
}
