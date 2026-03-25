pub mod stream;

pub use stream::{
    AddressBookConfig, AzureAttributes, ClickhouseAttributes, CreateStreamParams,
    DestinationAttributes, EnabledCountResponse, FilterLanguage, KafkaAttributes, ListStreamsParams,
    ListStreamsResponse, MongoAttributes, MysqlAttributes, PageInfo, PostgresAttributes, ProductType,
    RedisAttributes, S3Attributes, SnowflakeAttributes, Stream, StreamDataset, StreamDestination,
    StreamMetadataLocation, StreamRegion, StreamStatus, TestFilterParams, TestFilterResponse,
    UpdateStreamParams, WebhookAttributes,
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
        let mut body = serde_json::to_value(params).map_err(|e| SdkError::Config(e.to_string()))?;
        let obj = body
            .as_object_mut()
            .ok_or_else(|| SdkError::Config("failed to serialize request body as JSON object".into()))?;
        #[allow(clippy::needless_borrows_for_generic_args)]
        obj.insert(
            "destination".to_string(),
            serde_json::to_value(&params.destination_attributes.destination)
                .map_err(|e| SdkError::Config(e.to_string()))?,
        );
        obj.insert(
            "destination_attributes".to_string(),
            serde_json::from_str(&params.destination_attributes.value)
                .map_err(|e| SdkError::Config(e.to_string()))?,
        );

        let url = self.config.streams().base_url.join("streams")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(&body)
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

    pub async fn list_streams(&self, params: &ListStreamsParams) -> Result<ListStreamsResponse, SdkError> {
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
        let url = self.config.streams().base_url.join(&format!("streams/{id}"))?;
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

    pub async fn update_stream(&self, id: &str, params: &UpdateStreamParams) -> Result<Stream, SdkError> {
        let mut body = serde_json::to_value(params).map_err(|e| SdkError::Config(e.to_string()))?;
        if let Some(da) = &params.destination_attributes {
            let obj = body
                .as_object_mut()
                .ok_or_else(|| SdkError::Config("failed to serialize request body as JSON object".into()))?;
            #[allow(clippy::needless_borrows_for_generic_args)]
            obj.insert(
                "destination".to_string(),
                serde_json::to_value(&da.destination).map_err(|e| SdkError::Config(e.to_string()))?,
            );
            obj.insert(
                "destination_attributes".to_string(),
                serde_json::from_str(&da.value).map_err(|e| SdkError::Config(e.to_string()))?,
            );
        }
        let url = self.config.streams().base_url.join(&format!("streams/{id}"))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(&body)
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
        let url = self.config.streams().base_url.join(&format!("streams/{id}"))?;
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
        let url = self.config.streams().base_url.join(&format!("streams/{id}/activate"))?;
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
        let url = self.config.streams().base_url.join(&format!("streams/{id}/pause"))?;
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

    pub async fn test_filter(&self, params: &TestFilterParams) -> Result<TestFilterResponse, SdkError> {
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

    pub async fn get_enabled_count(&self, stream_type: Option<&str>) -> Result<EnabledCountResponse, SdkError> {
        let mut url = self.config.streams().base_url.join("streams/enabled_count")?;
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
            destination_attributes: DestinationAttributes::webhook(&WebhookAttributes {
                url: "https://example.com/webhook".to_string(),
                max_retry: 3,
                retry_interval_sec: 1,
                post_timeout_sec: 10,
                compression: "none".to_string(),
                security_token: None,
            }).unwrap(),
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
        let err = sdk.streams.create_stream(&webhook_params()).await.unwrap_err();

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
        let err = sdk.streams.create_stream(&webhook_params()).await.unwrap_err();

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
        let resp = sdk.streams.list_streams(&ListStreamsParams::default()).await.unwrap();
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
        let err = sdk.streams.list_streams(&ListStreamsParams::default()).await.unwrap_err();
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
        let err = sdk.streams.list_streams(&ListStreamsParams::default()).await.unwrap_err();
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
        let params = UpdateStreamParams { name: Some("updated-name".to_string()), ..Default::default() };
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
        let err = sdk.streams.update_stream("test-id", &params).await.unwrap_err();
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
        let err = sdk.streams.update_stream("test-id", &params).await.unwrap_err();
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
