#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_as_json_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    serde_json::to_string(&value).map_err(serde::de::Error::custom)
}

// ── Enums ──────────────────────────────────────────────────────────────────

/// Geographic region where a stream runs.
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamRegion {
    UsaEast,
    EuropeCentral,
    AsiaEast,
}

/// Type of on-chain data a stream delivers (blocks, transactions, logs, etc.).
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamDataset {
    Block,
    BlockWithReceipts,
    Transactions,
    Logs,
    Receipts,
    TraceBlocks,
    DebugTraces,
    BlockWithReceiptsDebugTrace,
    BlockWithReceiptsTraceBlock,
    BlobSidecars,
    ProgramsWithLogs,
    Ledger,
    Events,
    Orders,
    Trades,
    BookUpdates,
    Twap,
    WriterActions,
}

/// Destination kind a stream delivers to (webhook, S3, Postgres, etc.).
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamDestination {
    Webhook,
    S3,
    Azure,
    Postgres,
    Kafka,
}

/// Language a stream's filter function is written in.
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterLanguage {
    Javascript,
    Go,
    Wasm,
}

/// Where stream metadata is included in delivered payloads.
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMetadataLocation {
    Body,
    Header,
    None,
}

/// Billing product type the stream is associated with.
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Stream,
    Webhook,
}

/// Operational state of a stream.
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamStatus {
    Active,
    Paused,
    Terminated,
    Completed,
    Blocked,
}

// ── Destination Attribute Structs ──────────────────────────────────────────
//
// Each struct corresponds to one StreamDestination variant. Set exactly one
// on CreateStreamParams — see that struct's documentation for details.

/// Configuration for delivering stream batches to an HTTP webhook endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAttributes {
    /// Destination URL that receives batched stream payloads.
    pub url: String,
    /// Maximum number of retry attempts for a failed delivery. Must be in the range 1–10.
    pub max_retry: i32,
    /// Seconds to wait between retry attempts.
    pub retry_interval_sec: i32,
    /// Timeout in seconds for each POST request.
    pub post_timeout_sec: i32,
    /// Optional token included with each request so the receiver can verify authenticity. When supplied, must be at least 32 bytes (256 bits).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_token: Option<String>,
    /// Compression applied to the payload (e.g. `none`, `gzip`). When omitted the server defaults to no compression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl WebhookAttributes {
    #[new]
    #[pyo3(signature = (url, max_retry, retry_interval_sec, post_timeout_sec, compression=None, security_token=None))]
    pub fn new(
        url: String,
        max_retry: i32,
        retry_interval_sec: i32,
        post_timeout_sec: i32,
        compression: Option<String>,
        security_token: Option<String>,
    ) -> Self {
        Self {
            url,
            max_retry,
            retry_interval_sec,
            post_timeout_sec,
            security_token,
            compression,
        }
    }
}

/// Configuration for delivering stream batches to an S3-compatible object store.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Attributes {
    /// S3 service endpoint (e.g. `s3.amazonaws.com`).
    pub endpoint: String,
    /// Access key used to authenticate with the S3 endpoint.
    pub access_key: String,
    /// Secret key used to authenticate with the S3 endpoint.
    pub secret_key: String,
    /// Target bucket name.
    pub bucket: String,
    /// Key prefix prepended to each written object.
    pub object_prefix: String,
    /// Compression applied to written objects (e.g. `none`, `gzip`).
    pub compression: String,
    /// File format/extension for written objects (e.g. `.json`).
    pub file_type: String,
    /// Maximum number of retry attempts for a failed write.
    pub max_retry: i32,
    /// Seconds to wait between retry attempts.
    pub retry_interval_sec: i32,
    /// Whether to use TLS when connecting to the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_ssl: Option<bool>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl S3Attributes {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (endpoint, access_key, secret_key, bucket, object_prefix, compression, file_type, max_retry, retry_interval_sec, use_ssl=None))]
    pub fn new(
        endpoint: String,
        access_key: String,
        secret_key: String,
        bucket: String,
        object_prefix: String,
        compression: String,
        file_type: String,
        max_retry: i32,
        retry_interval_sec: i32,
        use_ssl: Option<bool>,
    ) -> Self {
        Self {
            endpoint,
            access_key,
            secret_key,
            bucket,
            object_prefix,
            compression,
            file_type,
            max_retry,
            retry_interval_sec,
            use_ssl,
        }
    }
}

/// Configuration for delivering stream batches to Azure Blob Storage.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAttributes {
    /// Azure storage account name.
    pub storage_account: String,
    /// SAS token used to authorize writes.
    pub sas_token: String,
    /// Container that receives written blobs.
    pub container: String,
    /// Compression applied to written blobs (e.g. `none`, `gzip`).
    pub compression: String,
    /// File format/extension for written blobs (e.g. `.json`).
    pub file_type: String,
    /// Maximum number of retry attempts for a failed write.
    pub max_retry: i32,
    /// Seconds to wait between retry attempts.
    pub retry_interval_sec: i32,
    /// Optional name prefix prepended to each written blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob_prefix: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl AzureAttributes {
    #[new]
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (storage_account, sas_token, container, compression, file_type, max_retry, retry_interval_sec, blob_prefix=None))]
    pub fn new(
        storage_account: String,
        sas_token: String,
        container: String,
        compression: String,
        file_type: String,
        max_retry: i32,
        retry_interval_sec: i32,
        blob_prefix: Option<String>,
    ) -> Self {
        Self {
            storage_account,
            sas_token,
            container,
            compression,
            file_type,
            max_retry,
            retry_interval_sec,
            blob_prefix,
        }
    }
}

/// Configuration for delivering stream batches to a PostgreSQL database.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresAttributes {
    /// Database host.
    pub host: String,
    /// Database port.
    pub port: i32,
    /// Database name.
    pub database: String,
    /// Username used to authenticate.
    pub username: String,
    /// Password used to authenticate.
    pub password: String,
    /// Destination table for inserted rows.
    pub table_name: String,
    /// Postgres SSL mode. The Quicknode API accepts only `disable` or `require`.
    pub sslmode: String,
    /// Maximum number of retry attempts for a failed write.
    pub max_retry: i32,
    /// Seconds to wait between retry attempts.
    pub retry_interval_sec: i32,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl PostgresAttributes {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: String,
        port: i32,
        database: String,
        username: String,
        password: String,
        table_name: String,
        sslmode: String,
        max_retry: i32,
        retry_interval_sec: i32,
    ) -> Self {
        Self {
            host,
            port,
            database,
            username,
            password,
            table_name,
            sslmode,
            max_retry,
            retry_interval_sec,
        }
    }
}

/// Configuration for delivering stream batches to a Kafka topic.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaAttributes {
    /// Comma-separated list of Kafka broker addresses (host:port).
    pub bootstrap_servers: String,
    /// Destination topic.
    pub topic_name: String,
    /// Compression codec applied to produced messages (e.g. `none`, `gzip`).
    pub compression_type: String,
    /// Maximum number of messages grouped per produce request.
    pub batch_size: i32,
    /// Milliseconds the producer waits to batch additional messages.
    pub linger_ms: i32,
    /// Maximum size in bytes of a single Kafka message (`max_message_bytes`).
    pub max_message_bytes: i32,
    /// Request timeout in seconds.
    pub timeout_sec: i32,
    /// Maximum number of retry attempts for a failed produce.
    pub max_retry: i32,
    /// Seconds to wait between retry attempts.
    pub retry_interval_sec: i32,
    /// Optional SASL username.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Optional SASL password.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    /// Optional security protocol (e.g. `SASL_SSL`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    /// Optional SASL mechanism (e.g. `PLAIN`, `SCRAM-SHA-256`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanisms: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl KafkaAttributes {
    #[new]
    #[pyo3(signature = (bootstrap_servers, topic_name, compression_type, batch_size, linger_ms, max_message_bytes, timeout_sec, max_retry, retry_interval_sec, username=None, password=None, protocol=None, mechanisms=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bootstrap_servers: String,
        topic_name: String,
        compression_type: String,
        batch_size: i32,
        linger_ms: i32,
        max_message_bytes: i32,
        timeout_sec: i32,
        max_retry: i32,
        retry_interval_sec: i32,
        username: Option<String>,
        password: Option<String>,
        protocol: Option<String>,
        mechanisms: Option<String>,
    ) -> Self {
        Self {
            bootstrap_servers,
            topic_name,
            compression_type,
            batch_size,
            linger_ms,
            max_message_bytes,
            timeout_sec,
            max_retry,
            retry_interval_sec,
            username,
            password,
            protocol,
            mechanisms,
        }
    }
}

// ── Address Book Config ────────────────────────────────────────────────────

/// Links a stream's filter to an address book so JSON paths resolve against its
/// managed address set.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookConfig {
    /// Identifier of the address book to use.
    pub address_book_id: String,
    /// Optional JSON path that resolves to an object whose fields are matched against the book.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objects_filter_path: Option<String>,
    /// JSON paths whose resolved values are matched against the book's addresses.
    pub elements_filter_paths: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl AddressBookConfig {
    #[new]
    #[pyo3(signature = (address_book_id, elements_filter_paths, objects_filter_path=None))]
    pub fn new(
        address_book_id: String,
        elements_filter_paths: Vec<String>,
        objects_filter_path: Option<String>,
    ) -> Self {
        Self {
            address_book_id,
            objects_filter_path,
            elements_filter_paths,
        }
    }
}

// ── Destination Attributes ─────────────────────────────────────────────────

/// Destination-specific configuration for a stream. Exactly one variant
/// selects where and how batches are delivered.
// Pure-Rust discriminated union; no #[pyclass] / #[napi(object)] because PyO3
// and napi-rs cannot represent enum-with-data. Each language binding crate
// wraps this type for its own FFI surface.
// The serde tag/content pair matches the API wire format when flattened into
// a request/response struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "destination",
    content = "destination_attributes",
    rename_all = "snake_case"
)]
pub enum DestinationAttributes {
    /// HTTP webhook endpoint that receives batches in real time.
    Webhook(WebhookAttributes),
    /// S3-compatible object storage for archival or batch processing.
    S3(S3Attributes),
    /// Azure Blob Storage destination.
    Azure(AzureAttributes),
    /// PostgreSQL database destination.
    Postgres(PostgresAttributes),
    /// Kafka topic destination.
    Kafka(KafkaAttributes),
}

impl DestinationAttributes {
    pub fn tag(&self) -> StreamDestination {
        match self {
            Self::Webhook(_) => StreamDestination::Webhook,
            Self::S3(_) => StreamDestination::S3,
            Self::Azure(_) => StreamDestination::Azure,
            Self::Postgres(_) => StreamDestination::Postgres,
            Self::Kafka(_) => StreamDestination::Kafka,
        }
    }
}

// ── Request (public-facing) ────────────────────────────────────────────────

/// Parameters for creating a new stream.
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStreamParams {
    /// Human-readable label identifying the stream.
    pub name: String,
    /// Geographic region where the stream runs.
    pub region: StreamRegion,
    /// Blockchain network to stream from (e.g. `ethereum-mainnet`).
    pub network: String,
    /// Type of on-chain data to stream.
    pub dataset: StreamDataset,
    /// Block number to begin streaming from.
    pub start_range: i64,
    /// Block number to stop streaming at; `-1` for continuous operation.
    pub end_range: i64,
    /// Destination-specific configuration (webhook URL, S3 bucket, DB credentials, etc.).
    // Flattening the enum's tag/content produces { destination, destination_attributes }.
    #[serde(flatten)]
    pub destination_attributes: DestinationAttributes,
    /// Billing plan associated with the stream. Optional; the server applies the account default when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// Buffer size used by the stream fetcher before delivery. Optional; the server applies its default when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_fetch_buffer: Option<i64>,
    /// Number of blocks grouped together per delivered batch. Required by the API.
    pub dataset_batch_size: i64,
    /// Upper bound on batch size when elastic batching is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<i64>,
    /// Maximum number of buffered blocks waiting to be processed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_range_size: Option<i64>,
    /// Maximum number of worker threads processing buffered batches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_processing_workers: Option<i64>,
    /// Number of blocks to stay behind the chain tip to reduce exposure to reorgs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_distance_from_tip: Option<i64>,
    /// Base64-encoded filter function applied to each batch before delivery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    /// Language the filter function is written in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<FilterLanguage>,
    /// Optional address book to evaluate the filter against.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
    /// Where to include stream metadata in delivered payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stream_metadata: Option<StreamMetadataLocation>,
    /// Billing product type the stream is associated with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<ProductType>,
    /// Initial stream state (`active` or `paused`). Defaults to `active` when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StreamStatus>,
    /// Email address that receives stream termination or failure alerts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// Minimum charge cap applied to the stream's billing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_min_cap: Option<i32>,
    /// Flag (0 or 1) enabling automatic re-streaming of blocks affected by chain reorganizations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_block_reorgs: Option<i32>,
    /// When enabled, batch size is reduced toward 1 as the stream catches up to the chain tip. Required by the API.
    pub elastic_batch_enabled: bool,
    /// Additional destinations that receive the same batches alongside the primary.
    // Not flattened: each element serializes as its own {destination, destination_attributes} pair.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_destinations: Option<Vec<DestinationAttributes>>,
}

// ── Response ───────────────────────────────────────────────────────────────

/// A stream's full configuration and current state, as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    /// Unique stream identifier.
    pub id: String,
    /// Human-readable stream name.
    pub name: String,
    /// Current operational state (e.g. `active`, `paused`).
    pub status: String,
    /// Timestamp when the stream was created.
    pub created_at: String,
    /// Timestamp of the most recent modification.
    pub updated_at: String,
    /// Sequence number tracking stream progress.
    pub sequence: i64,
    /// Blockchain network the stream is reading from.
    pub network: String,
    /// Dataset being streamed.
    pub dataset: String,
    /// Geographic region where the stream runs.
    pub region: String,
    /// Starting block for the stream.
    pub start_range: i64,
    /// Ending block for the stream; `-1` indicates continuous operation.
    pub end_range: i64,
    /// Billing plan associated with the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// Buffer size used by the stream fetcher before delivery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_fetch_buffer: Option<i64>,
    /// Number of blocks grouped together per delivered batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_batch_size: Option<i64>,
    /// Upper bound on batch size when elastic batching is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<i64>,
    /// Maximum number of buffered blocks waiting to be processed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_range_size: Option<i64>,
    /// Maximum number of worker threads processing buffered batches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_processing_workers: Option<i64>,
    /// Number of blocks the stream stays behind the chain tip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_distance_from_tip: Option<i64>,
    /// Base64-encoded filter function applied to each batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    /// Language the filter function is written in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<String>,
    /// Where stream metadata is included in delivered payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stream_metadata: Option<String>,
    /// Billing product type the stream is associated with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    /// Email address notified of stream termination or failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// Whether chain-reorg handling is enabled (0 or 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_block_reorgs: Option<i32>,
    /// Most recent block hash processed by the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    /// Destination-specific configuration (present on single-stream responses).
    // Optional because partial responses (e.g. list) may omit the destination pair.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<DestinationAttributes>,
    /// Whether elastic batching is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_batch_enabled: Option<bool>,
    /// Quicknode account ID that owns the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qn_account_id: Option<String>,
    /// Minimum charge cap applied to the stream's billing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_min_cap: Option<i32>,
    /// Free-text memo attached to the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// Address book linked to the stream's filter, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
    /// Additional destinations receiving the same batches alongside the primary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_destinations: Option<Vec<DestinationAttributes>>,
}

// ── New Request/Response Types ─────────────────────────────────────────────

/// Pagination metadata returned alongside a paginated result set.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    /// Page size used for this response.
    pub limit: i64,
    /// Starting index of this page within the full result set.
    pub offset: i64,
    /// Total number of items matching the query across all pages.
    pub total: i64,
}

/// Paginated response from `list_streams`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStreamsResponse {
    /// Streams on the current page.
    pub data: Vec<Stream>,
    /// Pagination metadata for the response.
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

/// Parameters for `list_streams`.
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListStreamsParams {
    /// Filter results by stream type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_type: Option<String>,
    /// Starting index into the result set; defaults to 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// Maximum number of streams returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Field to sort results by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    /// Sort direction (`asc` or `desc`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_direction: Option<String>,
}

/// Parameters for `update_stream`. Only fields that are set are modified;
/// omitted fields leave the current value unchanged.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UpdateStreamParams {
    /// New human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<StreamRegion>,
    /// New blockchain network.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// New dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<StreamDataset>,
    /// New start block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_range: Option<i64>,
    /// New end block; `-1` for continuous operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_range: Option<i64>,
    /// New primary destination configuration.
    // Flattening Option<enum> omits the keys entirely when None.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<DestinationAttributes>,
    /// New billing plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// New fetcher buffer threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_fetch_buffer: Option<i64>,
    /// New batch size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_batch_size: Option<i64>,
    /// New upper bound on elastic batch size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<i64>,
    /// New maximum buffered block range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_range_size: Option<i64>,
    /// New maximum number of buffer-processing workers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_processing_workers: Option<i64>,
    /// New distance from the chain tip.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_distance_from_tip: Option<i64>,
    /// New base64-encoded filter function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    /// New filter function language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<FilterLanguage>,
    /// New address book configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
    /// New stream-metadata location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stream_metadata: Option<StreamMetadataLocation>,
    /// New notification email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// New minimum charge cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_min_cap: Option<i32>,
    /// New reorg-handling flag (0 or 1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_block_reorgs: Option<i32>,
    /// Whether elastic batching is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_batch_enabled: Option<bool>,
    /// New operational state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StreamStatus>,
    /// Free-text memo to attach to the stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// New set of extra destinations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_destinations: Option<Vec<DestinationAttributes>>,
}

/// Parameters for `test_filter`.
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct TestFilterParams {
    /// Blockchain network to run the test against (e.g. `ethereum-mainnet`).
    pub network: String,
    /// Dataset the filter operates on.
    pub dataset: StreamDataset,
    /// Specific block number to feed into the filter for the test.
    pub block: String,
    /// Base64-encoded filter function to evaluate. Required by the API. To inspect raw block data with no transformation, supply a base64-encoded identity function such as `function main(d){return d;}`.
    pub filter_function: String,
    /// Language the filter function is written in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<FilterLanguage>,
    /// Address book linked to the filter, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
}

/// Result of a `test_filter` call.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFilterResponse {
    /// Filter output as a JSON string. Shape depends on the dataset and the user's filter function.
    #[serde(deserialize_with = "deserialize_as_json_string")]
    pub result: String,
    /// Log lines emitted by the filter function during evaluation.
    pub logs: Vec<String>,
}

/// Result of `get_enabled_count`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledCountResponse {
    /// Total count of currently enabled streams.
    pub total: i64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod destination_attributes_tests {
    use super::*;

    #[test]
    fn webhook_roundtrip() {
        let attrs = DestinationAttributes::Webhook(WebhookAttributes {
            url: "https://x.example/hook".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
            post_timeout_sec: 10,
            compression: Some("none".to_string()),
            security_token: None,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"webhook""#));
        assert!(json.contains(r#""url":"https://x.example/hook""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Webhook(_)));
        assert!(matches!(parsed.tag(), StreamDestination::Webhook));
    }

    #[test]
    fn s3_roundtrip() {
        let attrs = DestinationAttributes::S3(S3Attributes {
            endpoint: "s3.amazonaws.com".to_string(),
            access_key: "AK".to_string(),
            secret_key: "SK".to_string(),
            bucket: "b".to_string(),
            object_prefix: "p".to_string(),
            compression: "none".to_string(),
            file_type: "json".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
            use_ssl: Some(true),
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"s3""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::S3(_)));
    }

    #[test]
    fn azure_roundtrip() {
        let attrs = DestinationAttributes::Azure(AzureAttributes {
            storage_account: "acct".to_string(),
            sas_token: "tok".to_string(),
            container: "c".to_string(),
            compression: "none".to_string(),
            file_type: "json".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
            blob_prefix: None,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"azure""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Azure(_)));
    }

    #[test]
    fn postgres_roundtrip() {
        let attrs = DestinationAttributes::Postgres(PostgresAttributes {
            host: "h".to_string(),
            port: 5432,
            database: "db".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            table_name: "t".to_string(),
            sslmode: "disable".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"postgres""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Postgres(_)));
    }

    #[test]
    fn kafka_roundtrip() {
        let attrs = DestinationAttributes::Kafka(KafkaAttributes {
            bootstrap_servers: "host:9092".to_string(),
            topic_name: "t".to_string(),
            compression_type: "gzip".to_string(),
            batch_size: 100,
            linger_ms: 10,
            max_message_bytes: 1024,
            timeout_sec: 30,
            max_retry: 3,
            retry_interval_sec: 5,
            username: None,
            password: None,
            protocol: None,
            mechanisms: None,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"kafka""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Kafka(_)));
    }
}
