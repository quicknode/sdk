use napi_derive::napi;
use pyo3::pyclass;
use serde::{Deserialize, Serialize};

use crate::{errors::SdkError, SdkConfig};

const BASE_URL: &str = "https://api.quicknode.com/v0";

#[derive(Debug, Clone)]
pub struct AdminApiClient {
    config: SdkConfig,
}

#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
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

#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct GetEndpointsResponse {
    #[serde(default)]
    pub data: Vec<Endpoint>,
    pub error: Option<String>,
}

#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct Endpoint {
    pub id: String,
    pub label: String,
    pub chain: String,
    pub network: String,
    pub http_url: String,
    pub wss_url: String,
    #[serde(default)]
    pub tags: Vec<EndpointTag>,
}

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
        let resp: GetEndpointsResponse = self
            .config
            .http_client()
            .get(format!("{BASE_URL}endpoints"))
            .header("accept", "application/json")
            .header("x-api-key", self.config.api_key())
            .query(params)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }
}
