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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateEndpointRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEndpointResponse {
    pub data: SingleEndpoint,
    pub error: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct SingleEndpoint {
    pub id: String,
    pub label: Option<String>,
    pub status: Option<String>,
    pub chain: String,
    pub network: String,
    pub http_url: String,
    pub wss_url: Option<String>,
    pub security: Option<EndpointSecurity>,
    pub rate_limits: Option<EndpointRateLimits>,
    #[serde(default)]
    pub tags: Vec<EndpointTag>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointRateLimits {
    pub rate_limit_by_ip: Option<bool>,
    pub account: Option<i32>,
    pub rps: Option<i32>,
    pub rpm: Option<i32>,
    pub rpd: Option<i32>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointSecurity {
    pub options: Option<EndpointSecurityOptions>,
    #[serde(default)]
    pub tokens: Vec<EndpointToken>,
    #[serde(default)]
    pub jwts: Vec<EndpointJwt>,
    #[serde(default)]
    pub referrers: Vec<EndpointReferrer>,
    #[serde(default)]
    pub domain_masks: Vec<EndpointDomainMask>,
    #[serde(default)]
    pub ips: Vec<EndpointIp>,
    #[serde(default)]
    pub request_filters: Vec<EndpointRequestFilter>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointSecurityOptions {
    pub tokens: Option<bool>,
    pub jwts: Option<bool>,
    #[serde(rename = "domainMasks")]
    pub domain_masks: Option<bool>,
    pub ips: Option<bool>,
    pub referrers: Option<bool>,
    #[serde(rename = "requestFilters")]
    pub request_filters: Option<bool>,
    #[serde(rename = "ipCustomHeader")]
    pub ip_custom_header: Option<EndpointIpCustomHeaderOption>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointIpCustomHeaderOption {
    pub value: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointToken {
    pub id: String,
    pub token: String,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointJwt {
    pub id: String,
    pub public_key: String,
    pub kid: String,
    pub name: String,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointReferrer {
    pub id: String,
    pub referrer: String,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointDomainMask {
    pub id: String,
    pub domain: String,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointIp {
    pub id: String,
    pub ip: String,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct EndpointRequestFilter {
    pub id: String,
    #[serde(default)]
    pub method: Vec<String>,
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

    pub async fn create_endpoint(
        &self,
        params: &CreateEndpointRequest,
    ) -> Result<CreateEndpointResponse, SdkError> {
        let url = self.config.admin_base_url().join("endpoints")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .header("x-api-key", self.config.api_key())
            .json(params)
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

    #[tokio::test]
    async fn create_endpoint_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": "ep123",
                    "label": null,
                    "status": "active",
                    "chain": "ethereum",
                    "network": "mainnet",
                    "http_url": "https://example.quicknode.pro/ep123",
                    "wss_url": null,
                    "security": null,
                    "rate_limits": null,
                    "tags": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .create_endpoint(&CreateEndpointRequest::default())
            .await
            .unwrap();

        assert_eq!(resp.data.id, "ep123");
        assert_eq!(resp.data.chain, "ethereum");
        assert_eq!(resp.data.network, "mainnet");
    }

    #[tokio::test]
    async fn create_endpoint_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .admin
            .create_endpoint(&CreateEndpointRequest::default())
            .await
            .unwrap_err();

        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 400),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn create_endpoint_sends_body() {
        use wiremock::matchers::body_json;

        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints"))
            .and(body_json(serde_json::json!({
                "chain": "solana",
                "network": "mainnet-beta"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": "ep456",
                    "label": null,
                    "status": "active",
                    "chain": "solana",
                    "network": "mainnet-beta",
                    "http_url": "https://example.quicknode.pro/ep456",
                    "wss_url": null,
                    "security": null,
                    "rate_limits": null,
                    "tags": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateEndpointRequest {
            chain: Some("solana".to_string()),
            network: Some("mainnet-beta".to_string()),
        };
        let resp = sdk.admin.create_endpoint(&params).await.unwrap();

        assert_eq!(resp.data.id, "ep456");
        assert_eq!(resp.data.chain, "solana");
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
