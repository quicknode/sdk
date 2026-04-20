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

#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamRegion {
    UsaEast,
    EuropeCentral,
    AsiaEast,
}

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

#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamDestination {
    Webhook,
    S3,
    Azure,
    Postgres,
    Clickhouse,
    Snowflake,
    Mysql,
    Mongo,
    Kafka,
    Redis,
}

#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterLanguage {
    Javascript,
    Go,
    Wasm,
}

#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamMetadataLocation {
    Body,
    Header,
    None,
}

#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    Stream,
    Webhook,
}

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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAttributes {
    pub url: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
    pub post_timeout_sec: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_token: Option<String>,
    pub compression: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl WebhookAttributes {
    #[new]
    #[pyo3(signature = (url, max_retry, retry_interval_sec, post_timeout_sec, compression, security_token=None))]
    pub fn new(
        url: String,
        max_retry: i32,
        retry_interval_sec: i32,
        post_timeout_sec: i32,
        compression: String,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Attributes {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket: String,
    pub object_prefix: String,
    pub compression: String,
    pub file_type: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAttributes {
    pub storage_account: String,
    pub sas_token: String,
    pub container: String,
    pub compression: String,
    pub file_type: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresAttributes {
    pub host: String,
    pub port: i32,
    pub database: String,
    pub username: String,
    pub password: String,
    pub table_name: String,
    pub sslmode: String,
    pub max_retry: i32,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MysqlAttributes {
    pub host: String,
    pub port: i32,
    pub database: String,
    pub username: String,
    pub password: String,
    pub table_name: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl MysqlAttributes {
    #[new]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: String,
        port: i32,
        database: String,
        username: String,
        password: String,
        table_name: String,
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
            max_retry,
            retry_interval_sec,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoAttributes {
    pub host: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub collection_name: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl MongoAttributes {
    #[new]
    pub fn new(
        host: String,
        database: String,
        username: String,
        password: String,
        collection_name: String,
        max_retry: i32,
        retry_interval_sec: i32,
    ) -> Self {
        Self {
            host,
            database,
            username,
            password,
            collection_name,
            max_retry,
            retry_interval_sec,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickhouseAttributes {
    pub hosts: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub table_name: String,
    pub default_table_engine_opts: String,
    pub default_granularity: i32,
    pub default_compression: String,
    pub default_index_type: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_datetime_precision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dont_support_rename_column: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dont_support_empty_default_value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_initialize_with_version: Option<bool>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl ClickhouseAttributes {
    #[new]
    #[pyo3(signature = (hosts, database, username, password, table_name, default_table_engine_opts, default_granularity, default_compression, default_index_type, max_retry, retry_interval_sec, disable_datetime_precision=None, dont_support_rename_column=None, dont_support_empty_default_value=None, skip_initialize_with_version=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hosts: String,
        database: String,
        username: String,
        password: String,
        table_name: String,
        default_table_engine_opts: String,
        default_granularity: i32,
        default_compression: String,
        default_index_type: String,
        max_retry: i32,
        retry_interval_sec: i32,
        disable_datetime_precision: Option<bool>,
        dont_support_rename_column: Option<bool>,
        dont_support_empty_default_value: Option<bool>,
        skip_initialize_with_version: Option<bool>,
    ) -> Self {
        Self {
            hosts,
            database,
            username,
            password,
            table_name,
            default_table_engine_opts,
            default_granularity,
            default_compression,
            default_index_type,
            max_retry,
            retry_interval_sec,
            disable_datetime_precision,
            dont_support_rename_column,
            dont_support_empty_default_value,
            skip_initialize_with_version,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowflakeAttributes {
    pub account: String,
    pub host: String,
    pub port: i32,
    pub protocol: String,
    pub database: String,
    pub schema: String,
    pub warehouse: String,
    pub username: String,
    pub password: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table_name: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl SnowflakeAttributes {
    #[new]
    #[pyo3(signature = (account, host, port, protocol, database, schema, warehouse, username, password, max_retry, retry_interval_sec, table_name=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account: String,
        host: String,
        port: i32,
        protocol: String,
        database: String,
        schema: String,
        warehouse: String,
        username: String,
        password: String,
        max_retry: i32,
        retry_interval_sec: i32,
        table_name: Option<String>,
    ) -> Self {
        Self {
            account,
            host,
            port,
            protocol,
            database,
            schema,
            warehouse,
            username,
            password,
            max_retry,
            retry_interval_sec,
            table_name,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaAttributes {
    pub bootstrap_servers: String,
    pub topic_name: String,
    pub compression_type: String,
    pub batch_size: i32,
    pub linger_ms: i32,
    pub max_request_size: i32,
    pub timeout_sec: i32,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mechanisms: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl KafkaAttributes {
    #[new]
    #[pyo3(signature = (bootstrap_servers, topic_name, compression_type, batch_size, linger_ms, max_request_size, timeout_sec, max_retry, retry_interval_sec, username=None, password=None, protocol=None, mechanisms=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bootstrap_servers: String,
        topic_name: String,
        compression_type: String,
        batch_size: i32,
        linger_ms: i32,
        max_request_size: i32,
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
            max_request_size,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisAttributes {
    pub host: String,
    pub port: i32,
    pub database: i32,
    pub username: String,
    pub password: String,
    pub key_name: String,
    pub max_retry: i32,
    pub retry_interval_sec: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<bool>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl RedisAttributes {
    #[new]
    #[pyo3(signature = (host, port, database, username, password, key_name, max_retry, retry_interval_sec, tls=None))]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: String,
        port: i32,
        database: i32,
        username: String,
        password: String,
        key_name: String,
        max_retry: i32,
        retry_interval_sec: i32,
        tls: Option<bool>,
    ) -> Self {
        Self {
            host,
            port,
            database,
            username,
            password,
            key_name,
            max_retry,
            retry_interval_sec,
            tls,
        }
    }
}

// ── Address Book Config ────────────────────────────────────────────────────

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressBookConfig {
    pub address_book_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objects_filter_path: Option<String>,
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
    Webhook(WebhookAttributes),
    S3(S3Attributes),
    Azure(AzureAttributes),
    Postgres(PostgresAttributes),
    Mysql(MysqlAttributes),
    Mongo(MongoAttributes),
    Clickhouse(ClickhouseAttributes),
    Snowflake(SnowflakeAttributes),
    Kafka(KafkaAttributes),
    Redis(RedisAttributes),
}

impl DestinationAttributes {
    pub fn tag(&self) -> StreamDestination {
        match self {
            Self::Webhook(_) => StreamDestination::Webhook,
            Self::S3(_) => StreamDestination::S3,
            Self::Azure(_) => StreamDestination::Azure,
            Self::Postgres(_) => StreamDestination::Postgres,
            Self::Mysql(_) => StreamDestination::Mysql,
            Self::Mongo(_) => StreamDestination::Mongo,
            Self::Clickhouse(_) => StreamDestination::Clickhouse,
            Self::Snowflake(_) => StreamDestination::Snowflake,
            Self::Kafka(_) => StreamDestination::Kafka,
            Self::Redis(_) => StreamDestination::Redis,
        }
    }
}

// ── Request (public-facing) ────────────────────────────────────────────────

#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStreamParams {
    pub name: String,
    pub region: StreamRegion,
    pub network: String,
    pub dataset: StreamDataset,
    pub start_range: i64,
    pub end_range: i64,
    // Flattening the enum's tag/content produces { destination, destination_attributes }.
    #[serde(flatten)]
    pub destination_attributes: DestinationAttributes,
    pub plan: String,
    pub threshold_fetch_buffer: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_batch_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_range_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_processing_workers: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_distance_from_tip: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<FilterLanguage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stream_metadata: Option<StreamMetadataLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<ProductType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StreamStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_min_cap: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_block_reorgs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_batch_enabled: Option<bool>,
}

// ── Response ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: String,
    pub name: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub sequence: i64,
    pub network: String,
    pub dataset: String,
    pub region: String,
    pub start_range: i64,
    pub end_range: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_fetch_buffer: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_batch_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_range_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_processing_workers: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_distance_from_tip: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stream_metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_block_reorgs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_hash: Option<String>,
    // Optional because partial responses (e.g. list) may omit the destination pair.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<DestinationAttributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_batch_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qn_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_min_cap: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
}

// ── New Request/Response Types ─────────────────────────────────────────────

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub limit: i64,
    pub offset: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStreamsResponse {
    pub data: Vec<Stream>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ListStreamsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_direction: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UpdateStreamParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<StreamRegion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<StreamDataset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_range: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_range: Option<i64>,
    // Flattening Option<enum> omits the keys entirely when None.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<DestinationAttributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_fetch_buffer: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_batch_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_batch_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_range_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_buffer_processing_workers: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_distance_from_tip: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<FilterLanguage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_stream_metadata: Option<StreamMetadataLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub charge_min_cap: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_block_reorgs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elastic_batch_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StreamStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
}

#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct TestFilterParams {
    pub network: String,
    pub dataset: StreamDataset,
    pub block: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_language: Option<FilterLanguage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_book_config: Option<AddressBookConfig>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFilterResponse {
    /// Filter output as a JSON string. Shape depends on the dataset and the user's filter function.
    #[serde(deserialize_with = "deserialize_as_json_string")]
    pub result: String,
    pub logs: Vec<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledCountResponse {
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
            compression: "none".to_string(),
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
    fn mysql_roundtrip() {
        let attrs = DestinationAttributes::Mysql(MysqlAttributes {
            host: "h".to_string(),
            port: 3306,
            database: "db".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            table_name: "t".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"mysql""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Mysql(_)));
    }

    #[test]
    fn mongo_roundtrip() {
        let attrs = DestinationAttributes::Mongo(MongoAttributes {
            host: "h".to_string(),
            database: "db".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            collection_name: "c".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"mongo""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Mongo(_)));
    }

    #[test]
    fn clickhouse_roundtrip() {
        let attrs = DestinationAttributes::Clickhouse(ClickhouseAttributes {
            hosts: "h".to_string(),
            database: "db".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            table_name: "t".to_string(),
            default_table_engine_opts: "()".to_string(),
            default_granularity: 8192,
            default_compression: "lz4".to_string(),
            default_index_type: "minmax".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
            disable_datetime_precision: None,
            dont_support_rename_column: None,
            dont_support_empty_default_value: None,
            skip_initialize_with_version: None,
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"clickhouse""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Clickhouse(_)));
    }

    #[test]
    fn snowflake_roundtrip() {
        let attrs = DestinationAttributes::Snowflake(SnowflakeAttributes {
            account: "acct".to_string(),
            host: "h".to_string(),
            port: 443,
            protocol: "https".to_string(),
            database: "db".to_string(),
            schema: "s".to_string(),
            warehouse: "w".to_string(),
            username: "u".to_string(),
            password: "p".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
            table_name: Some("t".to_string()),
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"snowflake""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Snowflake(_)));
    }

    #[test]
    fn kafka_roundtrip() {
        let attrs = DestinationAttributes::Kafka(KafkaAttributes {
            bootstrap_servers: "host:9092".to_string(),
            topic_name: "t".to_string(),
            compression_type: "gzip".to_string(),
            batch_size: 100,
            linger_ms: 10,
            max_request_size: 1024,
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

    #[test]
    fn redis_roundtrip() {
        let attrs = DestinationAttributes::Redis(RedisAttributes {
            host: "h".to_string(),
            port: 6379,
            database: 0,
            username: "u".to_string(),
            password: "p".to_string(),
            key_name: "k".to_string(),
            max_retry: 3,
            retry_interval_sec: 5,
            tls: Some(false),
        });
        let json = serde_json::to_string(&attrs).unwrap();
        assert!(json.contains(r#""destination":"redis""#));
        let parsed: DestinationAttributes = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, DestinationAttributes::Redis(_)));
    }
}
