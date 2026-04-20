pub mod stream;

pub use stream::{
    AddressBookConfig, AzureAttributes, ClickhouseAttributes, CreateStreamParams,
    DestinationAttributes, EnabledCountResponse, FilterLanguage, KafkaAttributes,
    ListStreamsParams, ListStreamsResponse, MongoAttributes, MysqlAttributes, PageInfo,
    PostgresAttributes, ProductType, RedisAttributes, S3Attributes, SnowflakeAttributes, Stream,
    StreamDataset, StreamDestination, StreamMetadataLocation, StreamRegion, StreamStatus,
    TestFilterParams, TestFilterResponse, UpdateStreamParams, WebhookAttributes,
};

use crate::{config::StreamsConfig, errors::SdkError, SdkConfig};

const STREAMS_BASE_URL: &str = "https://api.quicknode.com/streams/rest/v1/";

pub(crate) struct ResolvedStreamsConfig {
    pub(crate) base_url: reqwest::Url,
}

impl ResolvedStreamsConfig {
    pub(crate) fn from_config(config: Option<&StreamsConfig>) -> Result<Self, SdkError> {
        let url_str = config
            .and_then(|s| s.base_url.as_deref())
            .unwrap_or(STREAMS_BASE_URL);
        let mut base_url = reqwest::Url::parse(url_str)?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Ok(Self { base_url })
    }
}

// ── Client ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StreamsApiClient {
    config: SdkConfig,
}

impl StreamsApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    pub async fn create_stream(&self, params: &CreateStreamParams) -> Result<Stream, SdkError> {
        let url = self.config.streams().base_url.join("streams")?;
        let resp = self
            .config
            .http_client()
            .post(url)
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

    pub async fn list_streams(
        &self,
        params: &ListStreamsParams,
    ) -> Result<ListStreamsResponse, SdkError> {
        let mut url = self.config.streams().base_url.join("streams")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(t) = &params.stream_type {
                pairs.append_pair("type", t);
            }
            if let Some(v) = params.offset {
                pairs.append_pair("offset", &v.to_string());
            }
            if let Some(v) = params.limit {
                pairs.append_pair("limit", &v.to_string());
            }
            if let Some(v) = &params.order_by {
                pairs.append_pair("order_by", v);
            }
            if let Some(v) = &params.order_direction {
                pairs.append_pair("order_direction", v);
            }
        }
        let resp = self
            .config
            .http_client()
            .get(url)
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

    pub async fn delete_all_streams(&self) -> Result<(), SdkError> {
        let url = self.config.streams().base_url.join("streams")?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    pub async fn get_stream(&self, id: &str) -> Result<Stream, SdkError> {
        let url = self
            .config
            .streams()
            .base_url
            .join(&format!("streams/{id}"))?;
        let resp = self
            .config
            .http_client()
            .get(url)
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

    pub async fn update_stream(
        &self,
        id: &str,
        params: &UpdateStreamParams,
    ) -> Result<Stream, SdkError> {
        let url = self
            .config
            .streams()
            .base_url
            .join(&format!("streams/{id}"))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
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

    pub async fn delete_stream(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .streams()
            .base_url
            .join(&format!("streams/{id}"))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    pub async fn activate_stream(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .streams()
            .base_url
            .join(&format!("streams/{id}/activate"))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    pub async fn pause_stream(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .streams()
            .base_url
            .join(&format!("streams/{id}/pause"))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    pub async fn test_filter(
        &self,
        params: &TestFilterParams,
    ) -> Result<TestFilterResponse, SdkError> {
        let url = self.config.streams().base_url.join("streams/test_filter")?;
        let resp = self
            .config
            .http_client()
            .post(url)
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

    pub async fn get_enabled_count(
        &self,
        stream_type: Option<&str>,
    ) -> Result<EnabledCountResponse, SdkError> {
        let mut url = self
            .config
            .streams()
            .base_url
            .join("streams/enabled_count")?;
        if let Some(t) = stream_type {
            url.query_pairs_mut().append_pair("type", t);
        }
        let resp = self
            .config
            .http_client()
            .get(url)
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

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{QuickNodeSdk, SdkFullConfig, StreamsConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuickNodeSdk {
        QuickNodeSdk::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: None,
            admin: None,
            streams: Some(StreamsConfig {
                base_url: Some(base_url),
            }),
            webhooks: None,
            kvstore: None,
        })
        .unwrap()
    }

    fn webhook_params() -> CreateStreamParams {
        CreateStreamParams {
            name: "test-stream".to_string(),
            region: StreamRegion::UsaEast,
            network: "ethereum-mainnet".to_string(),
            dataset: StreamDataset::Block,
            start_range: 17000000,
            end_range: -1,
            destination_attributes: DestinationAttributes::Webhook(WebhookAttributes {
                url: "https://example.com/webhook".to_string(),
                max_retry: 3,
                retry_interval_sec: 1,
                post_timeout_sec: 10,
                compression: "none".to_string(),
                security_token: None,
            }),
            plan: "growth_plan".to_string(),
            threshold_fetch_buffer: 1000,
            dataset_batch_size: None,
            max_batch_size: None,
            max_buffer_range_size: None,
            max_buffer_processing_workers: None,
            keep_distance_from_tip: None,
            filter_function: None,
            filter_language: None,
            address_book_config: None,
            include_stream_metadata: None,
            product_type: None,
            status: None,
            notification_email: None,
            charge_min_cap: None,
            fix_block_reorgs: None,
            elastic_batch_enabled: None,
            extra_destinations: None,
        }
    }

    fn stream_response_json() -> serde_json::Value {
        serde_json::json!({
            "id": "7d3c1a22-4f9e-4b1e-8b3d-1234567890ab",
            "name": "test-stream",
            "status": "active",
            "created_at": "2026-03-19T12:00:00Z",
            "updated_at": "2026-03-19T12:00:00Z",
            "sequence": 0,
            "network": "ethereum-mainnet",
            "dataset": "block",
            "region": "usa_east",
            "destination": "webhook",
            "destination_attributes": {
                "url": "https://example.com/webhook",
                "max_retry": 3,
                "retry_interval_sec": 1,
                "post_timeout_sec": 10,
                "compression": "none"
            },
            "start_range": 17000000,
            "end_range": -1
        })
    }

    #[tokio::test]
    async fn create_stream_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(200).set_body_json(stream_response_json()))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.streams.create_stream(&webhook_params()).await.unwrap();

        assert_eq!(resp.id, "7d3c1a22-4f9e-4b1e-8b3d-1234567890ab");
        assert_eq!(resp.name, "test-stream");
        assert_eq!(resp.status, "active");
        assert_eq!(resp.network, "ethereum-mainnet");
        assert_eq!(resp.dataset, "block");
        // Verify the full destination_attributes round-trip on the response
        // side. Without this assertion, serde's flatten+Option silently
        // swallows malformed destination_attributes as None.
        match resp.destination_attributes {
            Some(DestinationAttributes::Webhook(attrs)) => {
                assert_eq!(attrs.url, "https://example.com/webhook");
                assert_eq!(attrs.max_retry, 3);
            }
            other => panic!("expected Webhook destination, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_stream_sends_typed_webhook_destination() {
        use wiremock::matchers::body_partial_json;
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/streams"))
            .and(body_partial_json(serde_json::json!({
                "destination": "webhook",
                "destination_attributes": {
                    "url": "https://example.com/webhook",
                    "max_retry": 3,
                    "retry_interval_sec": 1,
                    "post_timeout_sec": 10,
                    "compression": "none"
                }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(stream_response_json()))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateStreamParams {
            destination_attributes: DestinationAttributes::Webhook(WebhookAttributes {
                url: "https://example.com/webhook".to_string(),
                max_retry: 3,
                retry_interval_sec: 1,
                post_timeout_sec: 10,
                compression: "none".to_string(),
                security_token: None,
            }),
            ..webhook_params()
        };
        sdk.streams.create_stream(&params).await.unwrap();
    }

    #[tokio::test]
    async fn create_stream_sends_extra_destinations() {
        use wiremock::matchers::body_partial_json;
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/streams"))
            .and(body_partial_json(serde_json::json!({
                "extra_destinations": [
                    {
                        "destination": "webhook",
                        "destination_attributes": {
                            "url": "https://example.com/extra-hook",
                            "max_retry": 5,
                            "retry_interval_sec": 2,
                            "post_timeout_sec": 15,
                            "compression": "none"
                        }
                    },
                    {
                        "destination": "s3",
                        "destination_attributes": {
                            "endpoint": "s3.example.com",
                            "access_key": "AKIA",
                            "secret_key": "secret",
                            "bucket": "my-bucket",
                            "object_prefix": "streams/",
                            "compression": "gzip",
                            "file_type": ".json",
                            "max_retry": 3,
                            "retry_interval_sec": 1
                        }
                    }
                ]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(stream_response_json()))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateStreamParams {
            extra_destinations: Some(vec![
                DestinationAttributes::Webhook(WebhookAttributes {
                    url: "https://example.com/extra-hook".to_string(),
                    max_retry: 5,
                    retry_interval_sec: 2,
                    post_timeout_sec: 15,
                    compression: "none".to_string(),
                    security_token: None,
                }),
                DestinationAttributes::S3(S3Attributes {
                    endpoint: "s3.example.com".to_string(),
                    access_key: "AKIA".to_string(),
                    secret_key: "secret".to_string(),
                    bucket: "my-bucket".to_string(),
                    object_prefix: "streams/".to_string(),
                    compression: "gzip".to_string(),
                    file_type: ".json".to_string(),
                    max_retry: 3,
                    retry_interval_sec: 1,
                    use_ssl: None,
                }),
            ]),
            ..webhook_params()
        };
        sdk.streams.create_stream(&params).await.unwrap();
    }

    #[tokio::test]
    async fn update_stream_sends_extra_destinations() {
        use wiremock::matchers::body_partial_json;
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/streams/test-id"))
            .and(body_partial_json(serde_json::json!({
                "extra_destinations": [
                    {
                        "destination": "webhook",
                        "destination_attributes": {
                            "url": "https://example.com/patched-hook",
                            "max_retry": 1,
                            "retry_interval_sec": 1,
                            "post_timeout_sec": 5,
                            "compression": "none"
                        }
                    }
                ]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(stream_response_json()))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateStreamParams {
            extra_destinations: Some(vec![DestinationAttributes::Webhook(WebhookAttributes {
                url: "https://example.com/patched-hook".to_string(),
                max_retry: 1,
                retry_interval_sec: 1,
                post_timeout_sec: 5,
                compression: "none".to_string(),
                security_token: None,
            })]),
            ..Default::default()
        };
        sdk.streams.update_stream("test-id", &params).await.unwrap();
    }

    #[tokio::test]
    async fn get_stream_parses_extra_destinations() {
        let server = MockServer::start().await;
        let mut body = stream_response_json();
        body["extra_destinations"] = serde_json::json!([
            {
                "destination": "webhook",
                "destination_attributes": {
                    "url": "https://example.com/extra",
                    "max_retry": 2,
                    "retry_interval_sec": 1,
                    "post_timeout_sec": 10,
                    "compression": "none"
                }
            }
        ]);
        Mock::given(method("GET"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.streams.get_stream("test-id").await.unwrap();
        let extras = resp.extra_destinations.expect("extra_destinations present");
        assert_eq!(extras.len(), 1);
        match &extras[0] {
            DestinationAttributes::Webhook(w) => {
                assert_eq!(w.url, "https://example.com/extra");
                assert_eq!(w.max_retry, 2);
            }
            other => panic!("expected Webhook, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_stream_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .streams
            .create_stream(&webhook_params())
            .await
            .unwrap_err();

        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn create_stream_server_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .streams
            .create_stream(&webhook_params())
            .await
            .unwrap_err();

        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn list_streams_success() {
        let server = MockServer::start().await;
        let response = serde_json::json!({
            "data": [stream_response_json()],
            "pageInfo": { "limit": 100, "offset": 0, "total": 1 }
        });
        Mock::given(method("GET"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .streams
            .list_streams(&ListStreamsParams::default())
            .await
            .unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.page_info.total, 1);
    }

    #[tokio::test]
    async fn list_streams_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .streams
            .list_streams(&ListStreamsParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn list_streams_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .streams
            .list_streams(&ListStreamsParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_stream_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(stream_response_json()))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.streams.get_stream("test-id").await.unwrap();
        assert_eq!(resp.id, "7d3c1a22-4f9e-4b1e-8b3d-1234567890ab");
    }

    #[tokio::test]
    async fn get_stream_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.get_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_stream_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.get_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn update_stream_success() {
        let server = MockServer::start().await;
        let mut updated = stream_response_json();
        updated["name"] = serde_json::json!("updated-name");
        Mock::given(method("PATCH"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateStreamParams {
            name: Some("updated-name".to_string()),
            ..Default::default()
        };
        let resp = sdk.streams.update_stream("test-id", &params).await.unwrap();
        assert_eq!(resp.name, "updated-name");
    }

    #[tokio::test]
    async fn update_stream_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateStreamParams::default();
        let err = sdk
            .streams
            .update_stream("test-id", &params)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn update_stream_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateStreamParams::default();
        let err = sdk
            .streams
            .update_stream("test-id", &params)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_stream_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.streams.delete_stream("test-id").await.unwrap();
    }

    #[tokio::test]
    async fn delete_stream_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.delete_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_stream_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/streams/test-id"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.delete_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_all_streams_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.streams.delete_all_streams().await.unwrap();
    }

    #[tokio::test]
    async fn delete_all_streams_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/streams"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.delete_all_streams().await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn activate_stream_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test-id/activate"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.streams.activate_stream("test-id").await.unwrap();
    }

    #[tokio::test]
    async fn activate_stream_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test-id/activate"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.activate_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn activate_stream_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test-id/activate"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.activate_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn pause_stream_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test-id/pause"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.streams.pause_stream("test-id").await.unwrap();
    }

    #[tokio::test]
    async fn pause_stream_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test-id/pause"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.pause_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn pause_stream_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test-id/pause"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.pause_stream("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn test_filter_success() {
        let server = MockServer::start().await;
        let response = serde_json::json!({ "result": {"hash": "0xabc"}, "logs": [] });
        Mock::given(method("POST"))
            .and(path("/streams/test_filter"))
            .respond_with(ResponseTemplate::new(201).set_body_json(response))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = TestFilterParams {
            network: "ethereum-mainnet".to_string(),
            dataset: StreamDataset::Block,
            block: "17811625".to_string(),
            filter_function: None,
            filter_language: None,
            address_book_config: None,
        };
        let resp = sdk.streams.test_filter(&params).await.unwrap();
        assert!(resp.logs.is_empty());
    }

    #[tokio::test]
    async fn test_filter_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/streams/test_filter"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = TestFilterParams {
            network: "ethereum-mainnet".to_string(),
            dataset: StreamDataset::Block,
            block: "17811625".to_string(),
            filter_function: None,
            filter_language: None,
            address_book_config: None,
        };
        let err = sdk.streams.test_filter(&params).await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_enabled_count_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams/enabled_count"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"total": 3})))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.streams.get_enabled_count(None).await.unwrap();
        assert_eq!(resp.total, 3);
    }

    #[tokio::test]
    async fn get_enabled_count_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams/enabled_count"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.get_enabled_count(None).await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_enabled_count_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/streams/enabled_count"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.streams.get_enabled_count(None).await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }
}
