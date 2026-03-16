pub mod admin;
pub mod errors;

use reqwest::Client as ReqwestClient;
use std::sync::Arc;

// Using Arc for the inner config to keep as a cheap clone
#[derive(Debug, Clone)]
pub struct SdkConfig(Arc<SdkConfigInner>);

#[derive(Debug)]
struct SdkConfigInner {
    http_client: ReqwestClient,
    api_key: String,
}

impl SdkConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self(Arc::new(SdkConfigInner {
            http_client: ReqwestClient::new(),
            api_key: api_key.into(),
        }))
    }

    pub(crate) fn http_client(&self) -> &ReqwestClient {
        &self.0.http_client
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.0.api_key
    }
}

pub struct QuickNodeSdk {
    pub admin: admin::AdminApiClient,
}

impl QuickNodeSdk {
    pub fn new(api_key: impl Into<String>) -> Self {
        let config = SdkConfig::new(api_key);
        Self {
            admin: admin::AdminApiClient::new(config),
        }
    }
}
