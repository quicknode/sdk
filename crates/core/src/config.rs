#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::errors::SdkError;

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HttpConfig {
    pub timeout_secs: Option<i64>,
    pub pool_max_idle_per_host: Option<i32>,
    /// Custom HTTP headers added to every outbound request.
    ///
    /// **These headers OVERRIDE any SDK-managed header with the same name**,
    /// including `User-Agent`, `x-api-key`, `Accept`, and `Content-Type`.
    /// Header names are matched case-insensitively. Use this to override the
    /// auto-generated User-Agent or inject correlation IDs, proxy auth, etc.
    pub headers: Option<std::collections::HashMap<String, String>>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl HttpConfig {
    #[new]
    #[pyo3(signature = (timeout_secs=None, pool_max_idle_per_host=None, headers=None))]
    pub fn new(
        timeout_secs: Option<i64>,
        pool_max_idle_per_host: Option<i32>,
        headers: Option<std::collections::HashMap<String, String>>,
    ) -> Self {
        HttpConfig {
            timeout_secs,
            pool_max_idle_per_host,
            headers,
        }
    }
}

/// Identifies the language and runtime making SDK calls. Each binding crate
/// (Python, Node, Ruby) constructs this and passes it through
/// [`SdkConfig::new_with_client_info`] so the SDK's auto-generated
/// `User-Agent` reflects the actual caller, not the underlying Rust core.
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Short language identifier, e.g. `"python"`, `"node"`, `"ruby"`, `"rust"`.
    pub language: String,
    /// Runtime version of the language, e.g. `"3.12.4"`, `"20.10.0"`, `"3.3.0"`.
    pub language_version: String,
    /// Version string of the language-specific SDK package — read from the
    /// language's own manifest (PyPI version, npm version, gem version).
    pub sdk_version: String,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AdminConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl AdminConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        AdminConfig { base_url }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct StreamsConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl StreamsConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        StreamsConfig { base_url }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct WebhooksConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl WebhooksConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        WebhooksConfig { base_url }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KvStoreConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl KvStoreConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        KvStoreConfig { base_url }
    }
}

/// A minted session JWT plus the endpoint it authenticates against and its
/// wall-clock expiry. This is the unit cached by the RPC client and the unit a
/// host persists between processes (e.g. the CLI's on-disk token cache).
///
/// `exp_unix` is the JWT `exp` claim (unix seconds), used directly so it
/// survives a process restart (unlike a monotonic `Instant`).
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CachedToken {
    /// The provisioned tooling-access endpoint URL the JWT authenticates against.
    pub endpoint_url: String,
    /// The minted ES256 session JWT, presented as a Bearer token.
    pub token: String,
    /// JWT `exp` claim in unix seconds.
    pub exp_unix: i64,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl CachedToken {
    #[new]
    pub fn new(endpoint_url: String, token: String, exp_unix: i64) -> Self {
        CachedToken {
            endpoint_url,
            token,
            exp_unix,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RpcConfig {
    /// Override for the tooling-access control-plane base URL used to mint
    /// session tokens. When unset, falls back to the tooling-access default.
    pub base_url: Option<String>,
    /// Optional pre-existing token to seed the in-memory cache (e.g. loaded
    /// from a host's on-disk cache). Advisory: a malformed or expired seed is
    /// treated as a cache miss and a fresh token is minted.
    pub seed: Option<CachedToken>,
    /// Seconds before `exp` at which the client proactively refreshes. The
    /// margin also absorbs clock skew between client and endpoint. Defaults to
    /// 60 when unset.
    pub refresh_margin_secs: Option<i64>,
    /// Per-network URL map for multichain routing: network key (e.g.
    /// `"solana-mainnet"`, `"polygon"`) -> full http_url. Built from
    /// `admin.get_endpoint_urls(...).multichain_urls`. When set, `rpc.call` with
    /// a `network` resolves the target URL here. Optional; the default-network
    /// call path needs no map.
    pub networks: Option<std::collections::HashMap<String, String>>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl RpcConfig {
    #[new]
    #[pyo3(signature = (base_url=None, seed=None, refresh_margin_secs=None, networks=None))]
    pub fn new(
        base_url: Option<String>,
        seed: Option<CachedToken>,
        refresh_margin_secs: Option<i64>,
        networks: Option<std::collections::HashMap<String, String>>,
    ) -> Self {
        RpcConfig {
            base_url,
            seed,
            refresh_margin_secs,
            networks,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SdkFullConfig {
    pub api_key: String,
    pub http: Option<HttpConfig>,
    pub admin: Option<AdminConfig>,
    pub streams: Option<StreamsConfig>,
    pub webhooks: Option<WebhooksConfig>,
    pub kvstore: Option<KvStoreConfig>,
    pub rpc: Option<RpcConfig>,
}

impl SdkFullConfig {
    pub fn from_api_key(api_key: String) -> Self {
        SdkFullConfig {
            api_key,
            http: None,
            admin: None,
            streams: None,
            webhooks: None,
            kvstore: None,
            rpc: None,
        }
    }

    pub fn from_env() -> Result<Self, SdkError> {
        config::Config::builder()
            .add_source(
                config::Environment::with_prefix("QN_SDK")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| SdkError::Config(e.to_string()))
            .and_then(Self::from_config)
    }

    fn from_config(cfg: config::Config) -> Result<Self, SdkError> {
        cfg.try_deserialize::<SdkFullConfig>()
            .map_err(|e| SdkError::Config(e.to_string()))
    }
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl SdkFullConfig {
    #[new]
    #[pyo3(signature = (api_key, http=None, admin=None, streams=None, webhooks=None, kvstore=None, rpc=None))]
    pub fn new(
        api_key: String,
        http: Option<HttpConfig>,
        admin: Option<AdminConfig>,
        streams: Option<StreamsConfig>,
        webhooks: Option<WebhooksConfig>,
        kvstore: Option<KvStoreConfig>,
        rpc: Option<RpcConfig>,
    ) -> Self {
        SdkFullConfig {
            api_key,
            http,
            admin,
            streams,
            webhooks,
            kvstore,
            rpc,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn build_config(pairs: &[(&str, &str)]) -> config::Config {
        let mut builder = config::Config::builder();
        for (k, v) in pairs {
            builder = builder.set_override(*k, *v).unwrap();
        }
        builder.build().unwrap()
    }

    #[test]
    fn from_env_missing_api_key_returns_error() {
        let cfg = config::Config::builder().build().unwrap();
        assert!(matches!(
            SdkFullConfig::from_config(cfg),
            Err(SdkError::Config(_))
        ));
    }

    #[test]
    fn from_env_only_api_key() {
        let cfg = build_config(&[("api_key", "test-key")]);
        let config = SdkFullConfig::from_config(cfg).unwrap();
        assert_eq!(config.api_key, "test-key");
        assert!(config.http.is_none());
        assert!(config.admin.is_none());
    }

    #[test]
    fn from_env_all_fields() {
        let cfg = build_config(&[
            ("api_key", "my-api-key"),
            ("http.timeout_secs", "30"),
            ("http.pool_max_idle_per_host", "5"),
            ("admin.base_url", "https://example.com/"),
        ]);
        let config = SdkFullConfig::from_config(cfg).unwrap();
        assert_eq!(config.api_key, "my-api-key");
        let http = config.http.unwrap();
        assert_eq!(http.timeout_secs, Some(30));
        assert_eq!(http.pool_max_idle_per_host, Some(5));
        let admin = config.admin.unwrap();
        assert_eq!(admin.base_url, Some("https://example.com/".to_string()));
    }

    #[test]
    fn from_env_invalid_timeout_secs() {
        let cfg = build_config(&[("api_key", "test-key"), ("http.timeout_secs", "abc")]);
        assert!(matches!(
            SdkFullConfig::from_config(cfg),
            Err(SdkError::Config(_))
        ));
    }

    #[test]
    fn from_env_headers_round_trip() {
        let cfg = build_config(&[
            ("api_key", "k"),
            ("http.headers.x-correlation-id", "abc"),
            ("http.headers.user-agent", "custom-ua/1.0"),
        ]);
        let config = SdkFullConfig::from_config(cfg).unwrap();
        let headers = config.http.unwrap().headers.unwrap();
        assert_eq!(
            headers.get("x-correlation-id").map(String::as_str),
            Some("abc")
        );
        assert_eq!(
            headers.get("user-agent").map(String::as_str),
            Some("custom-ua/1.0")
        );
    }
}
