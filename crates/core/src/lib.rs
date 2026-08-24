pub mod admin;
pub mod config;
pub mod errors;
pub mod kvstore;
pub mod rpc;
pub mod sql;
pub mod streams;
pub mod webhooks;

pub use admin::ToolingAccessStatus;
pub use config::{
    AdminConfig, CachedToken, ClientInfo, HttpConfig, KvStoreConfig, RpcConfig, SdkFullConfig,
    SqlConfig, StreamsConfig, WebhooksConfig,
};
pub use kvstore::{
    AddListItemParams, BulkSetsParams, CreateListParams, CreateSetParams, GetListData,
    GetListParams, GetListResponse, GetListsData, GetListsParams, GetListsResponse, GetSetResponse,
    GetSetsParams, GetSetsResponse, KvSetEntry, KvStoreApiClient, ListContainsItemResponse,
    UpdateListParams,
};
pub use rpc::RpcApiClient;
#[cfg(feature = "payments")]
pub use rpc::{
    generate_payment_wallet, ChainKind, CreditBalance, DripReceipt, GatewaySession,
    GeneratedWallet, PaymentConfig, PaymentReceipt, PaymentScheme, RpcCallResponse,
};
#[cfg(feature = "payments-tempo")]
pub use rpc::{ChannelState, ChannelStatus};
#[cfg(feature = "payments-tempo")]
pub use sql::MppQueryResult;
pub use sql::{
    ChainSchema, ColumnMeta, ColumnSchema, QueryParams, QueryResponse, QueryStatistics,
    SqlApiClient, SqlCluster, TableSchema, X402_SQL_BASE_URL,
};

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client as ReqwestClient;
use std::sync::Arc;

use errors::SdkError;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Build the auto-generated `User-Agent` value for a given caller.
///
/// Shape: `quicknode-sdk-{language}/{sdk_version} ({os}-{arch}; {language}-{language_version})`
fn build_user_agent(info: &ClientInfo) -> String {
    format!(
        "quicknode-sdk-{lang}/{ver} ({os}-{arch}; {lang}-{lang_ver})",
        lang = info.language,
        ver = info.sdk_version,
        os = std::env::consts::OS,
        arch = std::env::consts::ARCH,
        lang_ver = info.language_version,
    )
}

/// `ClientInfo` used when `SdkConfig::new` is called directly (pure-Rust path).
fn default_rust_client_info() -> ClientInfo {
    ClientInfo {
        language: "rust".to_string(),
        // CARGO_PKG_RUST_VERSION is the MSRV declared in Cargo.toml. We have
        // no way to read the actual rustc version that compiled the caller,
        // so MSRV is the closest stable identifier.
        language_version: option_env!("CARGO_PKG_RUST_VERSION")
            .unwrap_or("unknown")
            .to_string(),
        sdk_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

// Using Arc for the inner config to keep as a cheap clone
#[derive(Clone)]
pub struct SdkConfig(Arc<SdkConfigInner>);

impl std::fmt::Debug for SdkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SdkConfig")
            .field("api_key", &"[redacted]")
            .field("admin_base_url", &self.0.admin.base_url)
            .field("streams_base_url", &self.0.streams.base_url)
            .field("webhooks_base_url", &self.0.webhooks.base_url)
            .field("kvstore_base_url", &self.0.kvstore.base_url)
            .field("sql_base_url", &self.0.sql.base_url)
            .finish()
    }
}

struct SdkConfigInner {
    http_client: ReqwestClient,
    // Separate client for RPC data-plane calls: no account `x-api-key` header,
    // authenticates per-request with a Bearer JWT. See `new_with_client_info`.
    rpc_http_client: ReqwestClient,
    admin: admin::ResolvedAdminConfig,
    streams: streams::ResolvedStreamsConfig,
    webhooks: webhooks::ResolvedWebhooksConfig,
    kvstore: kvstore::ResolvedKvStoreConfig,
    sql: sql::ResolvedSqlConfig,
}

impl SdkConfig {
    /// Build an `SdkConfig` for a pure-Rust caller. The `User-Agent` will
    /// identify the core crate (`quicknode-sdk-rust/<version>`).
    pub fn new(config: &SdkFullConfig) -> Result<Self, SdkError> {
        Self::new_with_client_info(config, None)
    }

    /// Build an `SdkConfig` while attributing the `User-Agent` to a specific
    /// language binding (Python/Node/Ruby). Used by the binding crates so
    /// telemetry on the server side reflects the actual caller.
    ///
    /// If `client_info` is `None`, falls back to the pure-Rust identity.
    pub fn new_with_client_info(
        config: &SdkFullConfig,
        client_info: Option<ClientInfo>,
    ) -> Result<Self, SdkError> {
        let timeout_secs = match &config.http {
            Some(h) => match h.timeout_secs {
                Some(secs) if secs < 0 => {
                    return Err(SdkError::Config("timeout_secs must be non-negative".into()));
                }
                Some(secs) => secs as u64,
                None => DEFAULT_TIMEOUT_SECS,
            },
            None => DEFAULT_TIMEOUT_SECS,
        };
        let pool_max_idle_per_host = config
            .http
            .as_ref()
            .and_then(|http| http.pool_max_idle_per_host);

        // `ClientBuilder` is not cloneable, so build a freshly-configured
        // builder for each client from the shared transport settings.
        let make_builder = || {
            let mut builder =
                ReqwestClient::builder().timeout(std::time::Duration::from_secs(timeout_secs));
            if let Some(max_idle) = pool_max_idle_per_host {
                builder = builder.pool_max_idle_per_host(max_idle as usize);
            }
            builder
        };

        // Headers common to every client. The account `x-api-key` is added on
        // top of this only for the control-plane client below — RPC data-plane
        // calls authenticate with a short-lived Bearer JWT and must never carry
        // the long-lived account key.
        let mut common_headers = HeaderMap::new();
        common_headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        common_headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        let ua = build_user_agent(&client_info.unwrap_or_else(default_rust_client_info));
        common_headers.insert(
            reqwest::header::USER_AGENT,
            HeaderValue::from_str(&ua).map_err(|e| SdkError::Config(e.to_string()))?,
        );

        // Caller-supplied headers override anything above. `HeaderMap::insert`
        // replaces existing values for the same name.
        if let Some(http) = &config.http {
            if let Some(custom) = &http.headers {
                for (name, value) in custom {
                    let header_name = HeaderName::from_bytes(name.as_bytes()).map_err(|e| {
                        SdkError::Config(format!("invalid header name {name:?}: {e}"))
                    })?;
                    let header_value = HeaderValue::from_str(value).map_err(|e| {
                        SdkError::Config(format!("invalid header value for {name:?}: {e}"))
                    })?;
                    common_headers.insert(header_name, header_value);
                }
            }
        }

        // Control-plane client: carries the account `x-api-key` used to reach
        // the QuickNode management APIs (admin, streams, webhooks, etc.). Insert
        // the key first, then overlay `common_headers` so a caller-supplied
        // `x-api-key` in custom headers still wins (custom headers override all
        // SDK-managed defaults).
        //
        // A keyless config (no `api_key`) installs NO key header: the payment
        // lane needs no account key. Keyed surfaces then send un-authenticated
        // requests that the gateway rejects (surfacing as `ApiError`), rather
        // than sending an empty key header.
        let mut main_headers = HeaderMap::new();
        if let Some(api_key) = config.api_key.as_deref() {
            main_headers.insert(
                "x-api-key",
                HeaderValue::from_str(api_key).map_err(|e| SdkError::Config(e.to_string()))?,
            );
        }
        main_headers.extend(common_headers.clone());
        let http_client = make_builder()
            .default_headers(main_headers)
            .build()
            .map_err(|e| SdkError::Config(e.to_string()))?;

        // RPC data-plane client: identical transport config, but WITHOUT the
        // account `x-api-key` default header. RPC calls attach a Bearer JWT
        // per-request so the account key never leaves the control plane.
        let rpc_http_client = make_builder()
            .default_headers(common_headers)
            .build()
            .map_err(|e| SdkError::Config(e.to_string()))?;

        Ok(Self(Arc::new(SdkConfigInner {
            http_client,
            rpc_http_client,
            admin: admin::ResolvedAdminConfig::from_config(config.admin.as_ref())?,
            streams: streams::ResolvedStreamsConfig::from_config(config.streams.as_ref())?,
            webhooks: webhooks::ResolvedWebhooksConfig::from_config(config.webhooks.as_ref())?,
            kvstore: kvstore::ResolvedKvStoreConfig::from_config(config.kvstore.as_ref())?,
            sql: sql::ResolvedSqlConfig::from_config(config.sql.as_ref())?,
        })))
    }

    pub(crate) fn http_client(&self) -> &ReqwestClient {
        &self.0.http_client
    }

    pub(crate) fn rpc_http_client(&self) -> &ReqwestClient {
        &self.0.rpc_http_client
    }

    pub(crate) fn admin(&self) -> &admin::ResolvedAdminConfig {
        &self.0.admin
    }

    pub(crate) fn streams(&self) -> &streams::ResolvedStreamsConfig {
        &self.0.streams
    }

    pub(crate) fn webhooks(&self) -> &webhooks::ResolvedWebhooksConfig {
        &self.0.webhooks
    }

    pub(crate) fn kvstore(&self) -> &kvstore::ResolvedKvStoreConfig {
        &self.0.kvstore
    }

    pub(crate) fn sql(&self) -> &sql::ResolvedSqlConfig {
        &self.0.sql
    }
}

/// Top-level entry point for the Quicknode SDK. Holds sub-clients for each
/// product area; all share a single HTTP client and API key.
pub struct QuicknodeSdk {
    /// Admin API client: manages endpoints, tags, teams, billing, usage,
    /// metrics, security, and rate limits.
    pub admin: admin::AdminApiClient,
    /// Streams API client: creates and manages blockchain data streams.
    pub streams: streams::StreamsApiClient,
    /// Webhooks API client: creates and manages filter-template webhooks.
    pub webhooks: webhooks::WebhooksApiClient,
    /// Key-Value Store client: manages sets (single values) and lists
    /// (ordered collections) under string keys.
    pub kvstore: kvstore::KvStoreApiClient,
    /// SQL Explorer client: executes SQL queries against indexed blockchain
    /// data and fetches the database schema.
    pub sql: sql::SqlApiClient,
    /// JSON-RPC client: makes on-chain calls against the account's Tooling
    /// Access endpoint using short-lived session JWTs.
    pub rpc: rpc::RpcApiClient,
}

impl QuicknodeSdk {
    /// Creates a new SDK instance from an explicit configuration.
    pub fn new(config: &SdkFullConfig) -> Result<Self, SdkError> {
        Self::new_with_client_info(config, None)
    }

    /// Creates a new SDK instance, attributing the auto-generated `User-Agent`
    /// to a specific language binding. Used internally by Python/Node/Ruby
    /// binding crates.
    pub fn new_with_client_info(
        config: &SdkFullConfig,
        client_info: Option<ClientInfo>,
    ) -> Result<Self, SdkError> {
        let sdk_config = SdkConfig::new_with_client_info(config, client_info)?;
        Ok(Self {
            admin: admin::AdminApiClient::new(sdk_config.clone()),
            streams: streams::StreamsApiClient::new(sdk_config.clone()),
            webhooks: webhooks::WebhooksApiClient::new(sdk_config.clone()),
            kvstore: kvstore::KvStoreApiClient::new(sdk_config.clone()),
            sql: sql::SqlApiClient::new(sdk_config.clone()),
            rpc: rpc::RpcApiClient::new(sdk_config, config.rpc.as_ref()),
        })
    }

    /// Creates a new SDK instance using configuration from environment variables.
    pub fn from_env() -> Result<Self, SdkError> {
        Self::new(&SdkFullConfig::from_env()?)
    }

    /// Same as [`Self::from_env`] but with a binding-supplied [`ClientInfo`].
    pub fn from_env_with_client_info(client_info: Option<ClientInfo>) -> Result<Self, SdkError> {
        Self::new_with_client_info(&SdkFullConfig::from_env()?, client_info)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod headers_tests {
    use super::*;
    use std::collections::HashMap;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn base_config(api_key: &str) -> SdkFullConfig {
        SdkFullConfig {
            api_key: Some(api_key.to_string()),
            http: None,
            admin: None,
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
            rpc: None,
        }
    }

    fn binding_info() -> ClientInfo {
        ClientInfo {
            language: "python".to_string(),
            language_version: "3.12.4".to_string(),
            sdk_version: "1.2.3".to_string(),
        }
    }

    #[test]
    fn default_user_agent_identifies_rust_core() {
        let ua = build_user_agent(&default_rust_client_info());
        assert!(ua.starts_with("quicknode-sdk-rust/"));
        assert!(ua.contains(env!("CARGO_PKG_VERSION")));
        assert!(ua.contains(std::env::consts::OS));
        assert!(ua.contains(std::env::consts::ARCH));
    }

    #[test]
    fn binding_user_agent_identifies_language() {
        let ua = build_user_agent(&binding_info());
        let expected_prefix = "quicknode-sdk-python/1.2.3";
        assert!(ua.starts_with(expected_prefix), "got: {ua}");
        assert!(ua.contains("python-3.12.4"));
    }

    #[test]
    fn invalid_custom_header_name_errors() {
        let mut cfg = base_config("k");
        let mut h = HashMap::new();
        h.insert("bad header".to_string(), "v".to_string());
        cfg.http = Some(HttpConfig {
            timeout_secs: None,
            pool_max_idle_per_host: None,
            headers: Some(h),
        });
        assert!(matches!(SdkConfig::new(&cfg), Err(SdkError::Config(_))));
    }

    #[test]
    fn invalid_custom_header_value_errors() {
        let mut cfg = base_config("k");
        let mut h = HashMap::new();
        // Newline is not a valid header value byte.
        h.insert("X-Test".to_string(), "bad\nvalue".to_string());
        cfg.http = Some(HttpConfig {
            timeout_secs: None,
            pool_max_idle_per_host: None,
            headers: Some(h),
        });
        assert!(matches!(SdkConfig::new(&cfg), Err(SdkError::Config(_))));
    }

    #[tokio::test]
    async fn default_user_agent_reaches_wire_and_custom_headers_override() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .and(header("user-agent", "custom-ua/9.9"))
            .and(header("x-correlation-id", "abc"))
            // x-api-key override also wins
            .and(header("x-api-key", "override-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [], "error": null, "pagination": null
            })))
            .mount(&server)
            .await;

        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), "custom-ua/9.9".to_string());
        headers.insert("X-Correlation-Id".to_string(), "abc".to_string());
        headers.insert("x-api-key".to_string(), "override-key".to_string());

        let cfg = SdkFullConfig {
            api_key: Some("real-key".to_string()),
            http: Some(HttpConfig {
                timeout_secs: None,
                pool_max_idle_per_host: None,
                headers: Some(headers),
            }),
            admin: Some(AdminConfig {
                base_url: Some(format!("{}/", server.uri())),
            }),
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
            rpc: None,
        };

        let sdk = QuicknodeSdk::new(&cfg).unwrap();
        sdk.admin
            .get_endpoints(&admin::GetEndpointsRequest::default())
            .await
            .unwrap();
    }
}
