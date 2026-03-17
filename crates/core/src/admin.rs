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
        let url = self.config.admin_base_url().join("endpoints")?;
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{AdminConfig, QuickNodeSdk, SdkFullConfig};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuickNodeSdk {
        QuickNodeSdk::new(SdkFullConfig {
            api_key: "test-key".to_string(),
            http: None,
            admin: Some(AdminConfig {
                base_url: Some(base_url),
            }),
        })
        .unwrap()
    }

    #[tokio::test]
    async fn get_endpoints_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "id": "abc123",
                        "label": "My Endpoint",
                        "chain": "ethereum",
                        "network": "mainnet",
                        "http_url": "https://example.quicknode.pro/abc123",
                        "wss_url": null,
                        "tags": []
                    }
                ],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .get_endpoints(&GetEndpointsRequest::default())
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, "abc123");
        assert_eq!(resp.data[0].chain, "ethereum");
    }

    #[tokio::test]
    async fn get_endpoints_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .admin
            .get_endpoints(&GetEndpointsRequest::default())
            .await
            .unwrap_err();

        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 401),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_endpoints_sends_query_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .and(query_param("limit", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetEndpointsRequest {
            limit: Some(10),
            ..Default::default()
        };
        let resp = sdk.admin.get_endpoints(&params).await.unwrap();

        assert_eq!(resp.data.len(), 0);
    }

    #[tokio::test]
    async fn get_endpoints_base_url_without_trailing_slash() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "error": null
            })))
            .mount(&server)
            .await;

        let base_url_no_slash = server.uri();
        let sdk = make_sdk(base_url_no_slash);
        let resp = sdk
            .admin
            .get_endpoints(&GetEndpointsRequest::default())
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 0);
    }

    #[test]
    fn negative_timeout_secs_returns_error() {
        use crate::{HttpConfig, SdkConfig, SdkFullConfig};
        let result = SdkConfig::new(SdkFullConfig {
            api_key: "test-key".to_string(),
            http: Some(HttpConfig {
                timeout_secs: Some(-1),
                pool_max_idle_per_host: None,
            }),
            admin: None,
        });
        assert!(matches!(result, Err(crate::errors::SdkError::Config(_))));
    }
}
