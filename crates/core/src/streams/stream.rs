#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Serialize};

use crate::errors::SdkError;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl WebhookAttributes {
    #[new]
    #[pyo3(signature = (url, max_retry, retry_interval_sec, post_timeout_sec, security_token=None, compression=None))]
    pub fn new(
        url: String,
        max_retry: i32,
        retry_interval_sec: i32,
        post_timeout_sec: i32,
        security_token: Option<String>,
        compression: Option<String>,
    ) -> Self {
        Self { url, max_retry, retry_interval_sec, post_timeout_sec, security_token, compression }
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
        Self { endpoint, access_key, secret_key, bucket, object_prefix, compression, file_type, max_retry, retry_interval_sec, use_ssl }
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
        Self { storage_account, sas_token, container, compression, file_type, max_retry, retry_interval_sec, blob_prefix }
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
        Self { host, port, database, username, password, table_name, sslmode, max_retry, retry_interval_sec }
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
        Self { host, port, database, username, password, table_name, max_retry, retry_interval_sec }
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
        Self { host, database, username, password, collection_name, max_retry, retry_interval_sec }
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
            hosts, database, username, password, table_name,
            default_table_engine_opts, default_granularity, default_compression,
            default_index_type, max_retry, retry_interval_sec,
            disable_datetime_precision, dont_support_rename_column,
            dont_support_empty_default_value, skip_initialize_with_version,
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
        Self { account, host, port, protocol, database, schema, warehouse, username, password, max_retry, retry_interval_sec, table_name }
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
        Self { bootstrap_servers, topic_name, compression_type, batch_size, linger_ms, max_request_size, timeout_sec, max_retry, retry_interval_sec, username, password, protocol, mechanisms }
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
        Self { host, port, database, username, password, key_name, max_retry, retry_interval_sec, tls }
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
        Self { address_book_id, objects_filter_path, elements_filter_paths }
    }
}

// ── Request (public-facing) ────────────────────────────────────────────────

/// Parameters for creating a stream. Set exactly one attribute field matching
/// `destination`. Only the field corresponding to the chosen destination will be
/// used; all others must be `None`. Mismatches produce a `SdkError::Config` at
/// call time.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateStreamParams {
    pub name: String,
    pub region: StreamRegion,
    pub network: String,
    pub dataset: StreamDataset,
    pub start_range: i64,
    pub end_range: i64,
    pub destination: StreamDestination,
    // Exactly one of the following attribute fields must be set, corresponding to
    // the value of `destination`. The API expects a single `destination_attributes`
    // object whose shape depends on the destination type. Because napi-rs and PyO3
    // cannot represent a Rust discriminated union directly, these are modeled as
    // separate optional fields instead. The correct field is selected and validated
    // at call time in `extract_destination_attributes` — setting the wrong field or
    // leaving all fields `None` returns a `SdkError::Config` before any HTTP request
    // is made.
    #[serde(skip)]
    pub webhook_attributes: Option<WebhookAttributes>,       // use with StreamDestination::Webhook
    #[serde(skip)]
    pub s3_attributes: Option<S3Attributes>,                 // use with StreamDestination::S3
    #[serde(skip)]
    pub azure_attributes: Option<AzureAttributes>,           // use with StreamDestination::Azure
    #[serde(skip)]
    pub postgres_attributes: Option<PostgresAttributes>,     // use with StreamDestination::Postgres
    #[serde(skip)]
    pub mysql_attributes: Option<MysqlAttributes>,           // use with StreamDestination::Mysql
    #[serde(skip)]
    pub mongo_attributes: Option<MongoAttributes>,           // use with StreamDestination::Mongo
    #[serde(skip)]
    pub clickhouse_attributes: Option<ClickhouseAttributes>, // use with StreamDestination::Clickhouse
    #[serde(skip)]
    pub snowflake_attributes: Option<SnowflakeAttributes>,   // use with StreamDestination::Snowflake
    #[serde(skip)]
    pub kafka_attributes: Option<KafkaAttributes>,           // use with StreamDestination::Kafka
    #[serde(skip)]
    pub redis_attributes: Option<RedisAttributes>,           // use with StreamDestination::Redis
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

impl CreateStreamParams {
    pub(crate) fn destination_attributes(&self) -> Result<serde_json::Value, SdkError> {
        // Enforces: the attribute field matching `destination` must be set.
        // Returns SdkError::Config with a clear message if the wrong or no field is provided.
        match self.destination {
            StreamDestination::Webhook => self.webhook_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("webhook_attributes must be set when destination is Webhook".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::S3 => self.s3_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("s3_attributes must be set when destination is S3".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Azure => self.azure_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("azure_attributes must be set when destination is Azure".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Postgres => self.postgres_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("postgres_attributes must be set when destination is Postgres".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Mysql => self.mysql_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("mysql_attributes must be set when destination is Mysql".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Mongo => self.mongo_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("mongo_attributes must be set when destination is Mongo".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Clickhouse => self.clickhouse_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("clickhouse_attributes must be set when destination is Clickhouse".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Snowflake => self.snowflake_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("snowflake_attributes must be set when destination is Snowflake".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Kafka => self.kafka_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("kafka_attributes must be set when destination is Kafka".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
            StreamDestination::Redis => self.redis_attributes.as_ref()
                .ok_or_else(|| SdkError::Config("redis_attributes must be set when destination is Redis".into()))
                .and_then(|a| serde_json::to_value(a).map_err(|e| SdkError::Config(e.to_string()))),
        }
    }
}

// ── Response ───────────────────────────────────────────────────────────────

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
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
    pub destination: String,
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
}
