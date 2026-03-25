pub mod admin;
pub mod config;
pub mod errors;
pub mod kvstore;
pub mod streams;
pub mod webhooks;

pub use config::{AdminConfig, HttpConfig, KvStoreConfig, SdkFullConfig, StreamsConfig, WebhooksConfig};
pub use kvstore::{
    AddListItemParams, BulkSetsParams, CreateListParams, CreateSetParams, GetListData, GetListParams,
    GetListResponse, GetListsData, GetListsParams, GetListsResponse, GetSetResponse, GetSetsParams,
    GetSetsResponse, KvSetEntry, KvStoreApiClient, ListContainsItemResponse, UpdateListParams,
};

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client as ReqwestClient;
use std::sync::Arc;

use errors::SdkError;

const DEFAULT_TIMEOUT_SECS: u64 = 30;

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
            .finish()
    }
}

struct SdkConfigInner {
    http_client: ReqwestClient,
    admin: admin::ResolvedAdminConfig,
    streams: streams::ResolvedStreamsConfig,
    webhooks: webhooks::ResolvedWebhooksConfig,
    kvstore: kvstore::ResolvedKvStoreConfig,
}

impl SdkConfig {
    pub fn new(config: &SdkFullConfig) -> Result<Self, SdkError> {
        let mut builder = ReqwestClient::builder();

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
        builder = builder.timeout(std::time::Duration::from_secs(timeout_secs));

        if let Some(http) = &config.http {
            if let Some(max_idle) = http.pool_max_idle_per_host {
                builder = builder.pool_max_idle_per_host(max_idle as usize);
            }
        }

        let mut default_headers = HeaderMap::new();
        default_headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("application/json"));
        default_headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
        default_headers.insert(
            "x-api-key",
            HeaderValue::from_str(&config.api_key)
                .map_err(|e| SdkError::Config(e.to_string()))?,
        );
        builder = builder.default_headers(default_headers);

        let http_client = builder
            .build()
            .map_err(|e| SdkError::Config(e.to_string()))?;

        Ok(Self(Arc::new(SdkConfigInner {
            http_client,
            admin: admin::ResolvedAdminConfig::from_config(config.admin.as_ref())?,
            streams: streams::ResolvedStreamsConfig::from_config(config.streams.as_ref())?,
            webhooks: webhooks::ResolvedWebhooksConfig::from_config(config.webhooks.as_ref())?,
            kvstore: kvstore::ResolvedKvStoreConfig::from_config(config.kvstore.as_ref())?,
        })))
    }

    pub(crate) fn http_client(&self) -> &ReqwestClient {
        &self.0.http_client
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
}

pub struct QuickNodeSdk {
    pub admin: admin::AdminApiClient,
    pub streams: streams::StreamsApiClient,
    pub webhooks: webhooks::WebhooksApiClient,
    pub kvstore: kvstore::KvStoreApiClient,
}

impl QuickNodeSdk {
    pub fn new(config: &SdkFullConfig) -> Result<Self, SdkError> {
        let sdk_config = SdkConfig::new(config)?;
        Ok(Self {
            admin: admin::AdminApiClient::new(sdk_config.clone()),
            streams: streams::StreamsApiClient::new(sdk_config.clone()),
            webhooks: webhooks::WebhooksApiClient::new(sdk_config.clone()),
            kvstore: kvstore::KvStoreApiClient::new(sdk_config),
        })
    }

    pub fn from_env() -> Result<Self, SdkError> {
        Self::new(&SdkFullConfig::from_env()?)
    }
}
