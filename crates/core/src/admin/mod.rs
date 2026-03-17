pub mod endpoints;
pub use endpoints::*;

use crate::{errors::SdkError, SdkConfig};

#[derive(Debug, Clone)]
pub struct AdminApiClient {
    config: SdkConfig,
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
                    "security": {
                        "options": { "tokens": true, "jwts": false, "domainMasks": false, "ips": false, "referrers": false, "requestFilters": false },
                        "tokens": [{"id": "tok1", "token": "abc123"}],
                        "jwts": null,
                        "referrers": null,
                        "domain_masks": null,
                        "ips": null,
                        "request_filters": null
                    },
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
        let security = resp.data.security.unwrap();
        assert!(security.tokens.unwrap().len() == 1);
        assert!(security.jwts.is_none());
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
