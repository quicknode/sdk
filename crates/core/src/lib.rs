pub mod admin_api;
pub mod errors;

use std::sync::Arc;

use reqwest::Client as ReqwestClient;

#[derive(Debug, Clone)]
pub struct SdkConfig(Arc<SdkConfigInner>);

#[derive(Debug)]
struct SdkConfigInner {
    http_client: ReqwestClient,
    api_key: String,
    // config
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
