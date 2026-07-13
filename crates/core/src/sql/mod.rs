#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Serialize};

use crate::{config::SqlConfig, errors::SdkError, SdkConfig};

const SQL_BASE_URL: &str = "https://api.quicknode.com/sql/rest/v1/";

// ── Resolved config ────────────────────────────────────────────────────────

pub(crate) struct ResolvedSqlConfig {
    pub(crate) base_url: reqwest::Url,
}

impl ResolvedSqlConfig {
    pub(crate) fn from_config(config: Option<&SqlConfig>) -> Result<Self, SdkError> {
        let url_str = config
            .and_then(|s| s.base_url.as_deref())
            .unwrap_or(SQL_BASE_URL);
        let mut base_url =
            reqwest::Url::parse(url_str).map_err(|e| SdkError::Config(e.to_string()))?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Ok(Self { base_url })
    }
}

// ── Request types ──────────────────────────────────────────────────────────

/// Parameters for `query`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryParams {
    /// The SQL query to execute. Pagination is expressed in the SQL itself via
    /// `LIMIT`/`OFFSET`; the API caps results at 1000 rows per request.
    pub query: String,
    /// The blockchain network identifier (e.g. `"hyperliquid-core-mainnet"`).
    // The request body uses camelCase `clusterId`, unlike the schema response
    // which returns snake_case `cluster_id`.
    #[serde(rename = "clusterId")]
    pub cluster_id: String,
}

// ── Query response types ───────────────────────────────────────────────────

/// Metadata describing a single column in a query result set.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    /// Column name as it appears in the result set.
    pub name: String,
    /// Column data type (e.g. `"DateTime('UTC')"`, `"LowCardinality(String)"`).
    // Field is `column_type` in Rust because `type` is a keyword; serde and the
    // Node binding rename it to `type` on their respective surfaces. Using a raw
    // `r#type` ident instead breaks pyo3 stub generation, so the Python surface
    // exposes this as `column_type`.
    #[serde(rename = "type")]
    pub column_type: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl ColumnMeta {
    #[new]
    pub fn new(name: String, column_type: String) -> Self {
        Self { name, column_type }
    }
}

/// Execution statistics returned alongside query results.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStatistics {
    /// Total query execution time in seconds.
    pub elapsed: f64,
    /// Total number of rows scanned during execution.
    pub rows_read: i64,
    /// Total data scanned in bytes.
    pub bytes_read: i64,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl QueryStatistics {
    #[new]
    pub fn new(elapsed: f64, rows_read: i64, bytes_read: i64) -> Self {
        Self {
            elapsed,
            rows_read,
            bytes_read,
        }
    }
}

/// Response from `query`.
//
// Holds `serde_json::Value` rows whose columns depend on the SQL query, so this
// type cannot derive `#[pyclass]`/`#[napi(object)]`. It stays pure-Rust in core;
// each binding wraps it and exposes `data` as the language's native dynamic type
// (Python via `pythonize`, Node via napi's `serde_json::Value` support, Ruby via
// `serde_magnus`). Mirrors the `DestinationAttributes` wrapping pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResponse {
    /// Column metadata for each column in the result set.
    pub meta: Vec<ColumnMeta>,
    /// Result rows. Each row is a JSON object whose keys are the selected
    /// columns; shape varies per query.
    pub data: Vec<serde_json::Value>,
    /// Number of rows returned in this response.
    pub rows: i64,
    /// Total rows that matched the query before applying `LIMIT`; use for
    /// pagination.
    pub rows_before_limit_at_least: i64,
    /// Query execution statistics.
    pub statistics: QueryStatistics,
    /// Credits consumed by the query.
    pub credits: i64,
}

// ── Schema response types ──────────────────────────────────────────────────

/// A single column in a table schema.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    /// Column name.
    pub name: String,
    /// Column data type (e.g. `"UInt64"`, `"FixedString(42)"`).
    // See `ColumnMeta::column_type` for why this is not a raw `r#type` ident.
    #[serde(rename = "type")]
    pub column_type: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl ColumnSchema {
    #[new]
    pub fn new(name: String, column_type: String) -> Self {
        Self { name, column_type }
    }
}

/// Schema for a single table.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table name.
    pub name: String,
    /// Storage engine backing the table.
    pub engine: String,
    /// Approximate total number of rows in the table.
    pub total_rows: i64,
    /// Partition key expression; empty string for views.
    pub partition_key: String,
    /// Sorting key columns; empty for views.
    pub sorting_key: Vec<String>,
    /// Columns in the table.
    pub columns: Vec<ColumnSchema>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl TableSchema {
    #[new]
    pub fn new(
        name: String,
        engine: String,
        total_rows: i64,
        partition_key: String,
        sorting_key: Vec<String>,
        columns: Vec<ColumnSchema>,
    ) -> Self {
        Self {
            name,
            engine,
            total_rows,
            partition_key,
            sorting_key,
            columns,
        }
    }
}

/// Response from `get_schema`: the schema for a single chain/cluster.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainSchema {
    /// Human-readable chain name (e.g. `"Hyperliquid (HyperCore)"`).
    pub chain: String,
    /// Cluster identifier the schema belongs to.
    pub cluster_id: String,
    /// Tables available in this cluster.
    pub tables: Vec<TableSchema>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl ChainSchema {
    #[new]
    pub fn new(chain: String, cluster_id: String, tables: Vec<TableSchema>) -> Self {
        Self {
            chain,
            cluster_id,
            tables,
        }
    }
}

// ── Client ─────────────────────────────────────────────────────────────────

/// Client for the Quicknode SQL Explorer. Executes SQL queries against indexed
/// blockchain data and fetches the database schema.
#[derive(Debug, Clone)]
pub struct SqlApiClient {
    config: SdkConfig,
}

impl SqlApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    /// Executes a SQL query against the given cluster and returns the result
    /// set.
    pub async fn query(&self, params: &QueryParams) -> Result<QueryResponse, SdkError> {
        let url = self.config.sql().base_url.join("query")?;
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

    /// Fetches the database schema for a cluster, including table names,
    /// columns, types, sort keys, and partition strategies.
    pub async fn get_schema(&self, cluster_id: &str) -> Result<ChainSchema, SdkError> {
        let url = self
            .config
            .sql()
            .base_url
            .join(&format!("schema/{cluster_id}"))?;
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
    use crate::{QuicknodeSdk, SdkFullConfig, SqlConfig};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuicknodeSdk {
        QuicknodeSdk::new(&SdkFullConfig {
            api_key: Some("test-key".to_string()),
            http: None,
            admin: None,
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: Some(SqlConfig {
                base_url: Some(base_url),
            }),
            rpc: None,
        })
        .unwrap()
    }

    fn query_params() -> QueryParams {
        QueryParams {
            query: "SELECT 1".to_string(),
            cluster_id: "hyperliquid-core-mainnet".to_string(),
        }
    }

    // ── query ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn query_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/query"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": [
                    {"name": "time", "type": "DateTime('UTC')"},
                    {"name": "action_type", "type": "LowCardinality(String)"}
                ],
                "data": [
                    {"time": "2026-06-24 19:43:44", "action_type": "SystemSpotSendAction"},
                    {"time": "2026-06-24 19:43:42", "action_type": "SystemSendAssetAction"}
                ],
                "rows": 2,
                "rows_before_limit_at_least": 18251,
                "statistics": {"elapsed": 0.0067, "rows_read": 31341, "bytes_read": 1247178},
                "credits": 135
            })))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.sql.query(&query_params()).await.unwrap();
        assert_eq!(resp.rows, 2);
        assert_eq!(resp.rows_before_limit_at_least, 18251);
        assert_eq!(resp.credits, 135);
        assert_eq!(resp.meta.len(), 2);
        assert_eq!(resp.meta[0].name, "time");
        assert_eq!(resp.statistics.rows_read, 31341);
        // Dynamic row: confirm a value reads through.
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0]["action_type"], "SystemSpotSendAction");
    }

    // Wire-inspection regression: confirm the request body sends `clusterId`
    // (camelCase) so a future serde rename of `cluster_id` fails loudly.
    #[tokio::test]
    async fn query_wire_body_cluster_id() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/query"))
            .and(body_json(serde_json::json!({
                "query": "SELECT 1",
                "clusterId": "hyperliquid-core-mainnet"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "meta": [],
                "data": [],
                "rows": 0,
                "rows_before_limit_at_least": 0,
                "statistics": {"elapsed": 0.001, "rows_read": 0, "bytes_read": 0},
                "credits": 1
            })))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.sql.query(&query_params()).await.unwrap();
    }

    #[tokio::test]
    async fn query_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/query"))
            .respond_with(ResponseTemplate::new(403).set_body_json(
                serde_json::json!({"statusCode": 403, "message": "only SELECT queries are allowed"}),
            ))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.sql.query(&query_params()).await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn query_decode_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/query"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.sql.query(&query_params()).await.unwrap_err();
        assert!(matches!(err, SdkError::Decode { .. }));
    }

    // ── get_schema ───────────────────────────────────────────────────────────

    #[tokio::test]
    async fn get_schema_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/schema/hyperliquid-core-mainnet"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "chain": "Hyperliquid (HyperCore)",
                "cluster_id": "hyperliquid-core-mainnet",
                "tables": [
                    {
                        "name": "hyperliquid_agents",
                        "engine": "SharedReplacingMergeTree",
                        "total_rows": 3322574607i64,
                        "partition_key": "toYYYYMM(snapshot_time)",
                        "sorting_key": ["block_number", "agent"],
                        "columns": [
                            {"name": "agent", "type": "FixedString(42)"},
                            {"name": "block_number", "type": "UInt64"}
                        ]
                    }
                ]
            })))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .sql
            .get_schema("hyperliquid-core-mainnet")
            .await
            .unwrap();
        assert_eq!(resp.cluster_id, "hyperliquid-core-mainnet");
        assert_eq!(resp.tables.len(), 1);
        let table = &resp.tables[0];
        assert_eq!(table.name, "hyperliquid_agents");
        assert_eq!(table.total_rows, 3322574607);
        assert_eq!(table.sorting_key, vec!["block_number", "agent"]);
        assert_eq!(table.columns[0].name, "agent");
        assert_eq!(table.columns[0].column_type, "FixedString(42)");
    }

    #[tokio::test]
    async fn get_schema_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/schema/bad-cluster"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.sql.get_schema("bad-cluster").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }
}
