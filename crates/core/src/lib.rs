pub mod admin;
pub mod config;
pub mod errors;

pub use config::{AdminConfig, HttpConfig, SdkFullConfig};

use reqwest::Client as ReqwestClient;
use std::sync::Arc;

use errors::SdkError;

const ADMIN_BASE_URL: &str = "https://api.quicknode.com/v0/";

// Using Arc for the inner config to keep as a cheap clone
#[derive(Debug, Clone)]
pub struct SdkConfig(Arc<SdkConfigInner>);

#[derive(Debug)]
struct SdkConfigInner {
    http_client: ReqwestClient,
    api_key: String,
    admin_base_url: reqwest::Url,
}

impl SdkConfig {
    pub fn new(config: SdkFullConfig) -> Self {
        let mut builder = ReqwestClient::builder();
        if let Some(http) = &config.http {
            if let Some(secs) = http.timeout_secs {
                builder = builder.timeout(std::time::Duration::from_secs(secs as u64));
            }
            if let Some(max_idle) = http.pool_max_idle_per_host {
                builder = builder.pool_max_idle_per_host(max_idle as usize);
            }
        }
        let http_client = builder.build().expect("failed to build HTTP client");

        let admin_base_url_str = config
            .admin
            .as_ref()
            .and_then(|a| a.base_url.as_deref())
            .unwrap_or(ADMIN_BASE_URL);
        let admin_base_url =
            reqwest::Url::parse(admin_base_url_str).expect("invalid admin base URL");

        Self(Arc::new(SdkConfigInner {
            http_client,
            api_key: config.api_key,
            admin_base_url,
        }))
    }

    pub(crate) fn http_client(&self) -> &ReqwestClient {
        &self.0.http_client
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.0.api_key
    }

    pub(crate) fn admin_base_url(&self) -> &reqwest::Url {
        &self.0.admin_base_url
    }
}

pub struct QuickNodeSdk {
    pub admin: admin::AdminApiClient,
}

impl QuickNodeSdk {
    pub fn new(config: SdkFullConfig) -> Self {
        let sdk_config = SdkConfig::new(config);
        Self {
            admin: admin::AdminApiClient::new(sdk_config),
        }
    }

    pub fn from_env() -> Result<Self, SdkError> {
        Ok(Self::new(SdkFullConfig::from_env()?))
    }
}
