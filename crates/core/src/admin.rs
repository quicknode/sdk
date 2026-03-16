#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

use crate::{errors::SdkError, SdkConfig};

static BASE_URL: std::sync::LazyLock<reqwest::Url> = std::sync::LazyLock::new(|| {
    reqwest::Url::parse("https://api.quicknode.com/v0/").expect("invalid base URL")
});

#[derive(Debug, Clone)]
pub struct AdminApiClient {
    config: SdkConfig,
}

// In core, any data structs get python stub pycalss to generate typing file
#[cfg_attr(feature = "python", gen_stub_pyclass)]
// Pyo3 macro to genrate python class from rust struct
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
// napi macro to generate typescript types from rust crate
#[cfg_attr(feature = "node", napi(object))]
// Bon builder for builder pattern added to request params for easy building in rust sdk
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEndpointsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_labels: Option<Vec<String>>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct GetEndpointsResponse {
    #[serde(default)]
    pub data: Vec<Endpoint>,
    pub error: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub label: Option<String>,
    pub chain: String,
    pub network: String,
    pub http_url: String,
    pub wss_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<EndpointTag>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointTag {
    pub tag_id: i32,
    pub label: String,
}

impl AdminApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    pub async fn get_endpoints(
        &self,
        params: &GetEndpointsRequest,
    ) -> Result<GetEndpointsResponse, SdkError> {
        let url = BASE_URL.join("endpoints")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .header("accept", "application/json")
            .header("x-api-key", self.config.api_key())
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }
}
