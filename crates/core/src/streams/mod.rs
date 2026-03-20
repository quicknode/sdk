pub mod stream;

pub use stream::{
    AddressBookConfig, AzureAttributes, ClickhouseAttributes, CreateStreamParams,
    DestinationAttributes, FilterLanguage, KafkaAttributes, MongoAttributes, MysqlAttributes,
    PostgresAttributes, ProductType, RedisAttributes, S3Attributes, SnowflakeAttributes, Stream,
    StreamDataset, StreamDestination, StreamMetadataLocation, StreamRegion, StreamStatus,
    WebhookAttributes,
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
                security_token: None,
                compression: None,
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
}
