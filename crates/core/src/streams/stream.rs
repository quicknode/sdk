#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyResult};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Deserializer, Serialize};

use crate::errors::SdkError;

fn deserialize_as_json_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    serde_json::to_string(&value).map_err(serde::de::Error::custom)
}

fn deserialize_as_optional_json_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(v) => serde_json::to_string(&v)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
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

// The API expects a `destination` string and a `destination_attributes` object
// whose shape depends on which destination is selected. The natural Rust model
// would be an enum with per-variant data, but napi-rs and PyO3 cannot represent
// Rust discriminated unions at the FFI boundary — they require flat structs.
// Instead, `DestinationAttributes` is a flat wrapper struct that bundles the
// destination variant with its pre-serialized JSON value. Callers construct it
// via typed static factory methods (one per destination), so they never interact
// with raw JSON. The `CreateStreamParams` holds one `DestinationAttributes`
// field instead of 10 separate `Option<XAttributes>` fields — making mismatches
// a compile-time error in Rust and a clear constructor error in Python/Node.

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationAttributes {
    // pub fields required for napi(object) to expose them in TypeScript.
    // Callers should use the typed factory methods rather than setting fields
    // directly — the value field is a pre-serialized JSON string.
    pub destination: StreamDestination,
    // Stored as a JSON string so napi(object) can represent it (serde_json::Value
    // is not supported by napi-rs). Parsed back to Value in create_stream.
    pub value: String,
}

// napi(object) on CreateStreamParams requires all fields to implement Default
// so napi can handle cases where the field is absent in JS. In practice,
// destination_attributes is always required — the default is never used.
impl Default for DestinationAttributes {
    fn default() -> Self {
        Self {
            destination: StreamDestination::Webhook,
            value: "null".to_string(),
        }
    }
}

impl DestinationAttributes {
    pub fn webhook(attrs: &WebhookAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Webhook,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn s3(attrs: &S3Attributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::S3,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn azure(attrs: &AzureAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Azure,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn postgres(attrs: &PostgresAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Postgres,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn mysql(attrs: &MysqlAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Mysql,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn mongo(attrs: &MongoAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Mongo,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn clickhouse(attrs: &ClickhouseAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Clickhouse,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn snowflake(attrs: &SnowflakeAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Snowflake,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn kafka(attrs: &KafkaAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Kafka,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn redis(attrs: &RedisAttributes) -> Result<Self, SdkError> {
        Ok(Self {
            destination: StreamDestination::Redis,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl DestinationAttributes {
    #[staticmethod]
    #[pyo3(name = "webhook")]
    fn py_webhook(attrs: &WebhookAttributes) -> PyResult<Self> {
        Self::webhook(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "s3")]
    fn py_s3(attrs: &S3Attributes) -> PyResult<Self> {
        Self::s3(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "azure")]
    fn py_azure(attrs: &AzureAttributes) -> PyResult<Self> {
        Self::azure(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "postgres")]
    fn py_postgres(attrs: &PostgresAttributes) -> PyResult<Self> {
        Self::postgres(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "mysql")]
    fn py_mysql(attrs: &MysqlAttributes) -> PyResult<Self> {
        Self::mysql(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "mongo")]
    fn py_mongo(attrs: &MongoAttributes) -> PyResult<Self> {
        Self::mongo(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "clickhouse")]
    fn py_clickhouse(attrs: &ClickhouseAttributes) -> PyResult<Self> {
        Self::clickhouse(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "snowflake")]
    fn py_snowflake(attrs: &SnowflakeAttributes) -> PyResult<Self> {
        Self::snowflake(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "kafka")]
    fn py_kafka(attrs: &KafkaAttributes) -> PyResult<Self> {
        Self::kafka(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "redis")]
    fn py_redis(attrs: &RedisAttributes) -> PyResult<Self> {
        Self::redis(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

// ── Request (public-facing) ────────────────────────────────────────────────

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
    // destination and destination_attributes are skipped here and inserted manually
    // into the request body in StreamsApiClient::create_stream, so serde doesn't
    // try to serialize them as fields of this struct.
    #[serde(skip)]
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
    /// Destination-specific configuration as a JSON string. Shape depends on the destination type.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_as_optional_json_string"
    )]
    pub destination_attributes: Option<String>,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
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

#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
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
    #[serde(skip)]
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
