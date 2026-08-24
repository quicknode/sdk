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
/// Public catalog + x402 drawdown SQL host. Discovery always uses this
/// prefix (or a `--base-url` remap). The account host above stays on `query`.
pub const X402_SQL_BASE_URL: &str = "https://x402.quicknode.com/sql/rest/v1/";

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

// ── Catalog types ──────────────────────────────────────────────────────────

/// One cluster from `GET /sql/rest/v1/clusters`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SqlCluster {
    /// Cluster identifier (e.g. `"hyperliquid-core-mainnet"`).
    pub id: String,
    /// Human-readable name (e.g. `"Hyperliquid (HyperCore)"`).
    pub display_name: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl SqlCluster {
    #[new]
    pub fn new(id: String, display_name: String) -> Self {
        Self { id, display_name }
    }
}

/// A successful MPP-session SQL query plus the receipt's accepted cumulative
/// spend. The caller persists `accepted_cumulative` on the channel; do not
/// advance the channel by `query.credits` (SQL credits ≠ voucher increment).
#[cfg(feature = "payments-tempo")]
#[derive(Debug, Clone)]
pub struct MppQueryResult {
    pub query: QueryResponse,
    pub accepted_cumulative: u128,
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

    /// Lists clusters from the public SQL catalog (`GET clusters`). Always
    /// unauthenticated: uses the keyless client so an `x-api-key` is never
    /// sent, even when the SDK was built with one.
    pub async fn list_clusters(&self) -> Result<Vec<SqlCluster>, SdkError> {
        let url = self.config.sql().base_url.join("clusters")?;
        let resp = self
            .config
            .rpc_http_client()
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

    /// Executes a SQL query against the x402 drawdown host with a SIWX session
    /// JWT (`Authorization: Bearer`). Single attempt; 401/403 stay
    /// [`SdkError::Api`] so the caller can re-auth on `token_expired`. A 402
    /// `requires_payment` is also [`SdkError::Api`] — this lane never signs a
    /// per-request payment.
    #[cfg(feature = "payments")]
    pub async fn query_with_session(
        &self,
        params: &QueryParams,
        session: &crate::rpc::payment::drawdown::GatewaySession,
    ) -> Result<QueryResponse, SdkError> {
        let url = self.config.sql().base_url.join("query")?;
        let resp = self
            .config
            .rpc_http_client()
            .post(url)
            .bearer_auth(&session.token)
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

    /// Executes a SQL query on the MPP session route
    /// (`POST {mpp}/session/sql/rest/v1/query`) with a cumulative voucher.
    /// The increment is the SQL challenge `amount` (observed 100), not
    /// [`crate::rpc::payment::session::ChannelState::per_call`]. A 402
    /// insufficient-balance is terminal — this method does not sign a smaller
    /// increment. Persist [`MppQueryResult::accepted_cumulative`] after 200.
    #[cfg(feature = "payments-tempo")]
    pub async fn query_with_mpp_session(
        &self,
        params: &QueryParams,
        payment: &crate::config::PaymentConfig,
        channel: &crate::rpc::payment::session::ChannelState,
    ) -> Result<MppQueryResult, SdkError> {
        let resolved = crate::rpc::payment::ResolvedPayment::from_config(payment)?;
        let body = serde_json::to_value(params).map_err(|e| {
            SdkError::Config(format!("could not serialize the SQL query body: {e}"))
        })?;
        let (text, accepted_cumulative) = crate::rpc::payment::session::sql_voucher_call(
            self.config.rpc_http_client(),
            &resolved,
            channel,
            &body,
        )
        .await?;
        let query = serde_json::from_str(&text)
            .map_err(|source| SdkError::Decode { source, body: text })?;
        Ok(MppQueryResult {
            query,
            accepted_cumulative,
        })
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

    // ── list_clusters ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn list_clusters_decodes_id_and_display_name() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/clusters"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"id": "hyperliquid-core-mainnet", "display_name": "Hyperliquid (HyperCore)"},
                {"id": "solana-mainnet", "display_name": "Solana"}
            ])))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let clusters = sdk.sql.list_clusters().await.unwrap();
        assert_eq!(clusters.len(), 2);
        assert_eq!(clusters[0].id, "hyperliquid-core-mainnet");
        assert_eq!(clusters[0].display_name, "Hyperliquid (HyperCore)");
        assert_eq!(clusters[1].id, "solana-mainnet");
        assert_eq!(clusters[1].display_name, "Solana");
    }

    // ── query_with_session ───────────────────────────────────────────────────

    #[cfg(feature = "payments")]
    fn session() -> crate::rpc::payment::drawdown::GatewaySession {
        crate::rpc::payment::drawdown::GatewaySession {
            token: "jwt-abc".into(),
            exp_unix: 4_102_444_800,
            account_id: "a".into(),
        }
    }

    #[cfg(feature = "payments")]
    fn ok_query_body() -> serde_json::Value {
        serde_json::json!({
            "meta": [{"name": "1", "type": "UInt8"}],
            "data": [{"1": 1}],
            "rows": 1,
            "rows_before_limit_at_least": 1,
            "statistics": {"elapsed": 0.001, "rows_read": 1, "bytes_read": 1},
            "credits": 117
        })
    }

    #[cfg(feature = "payments")]
    #[tokio::test]
    async fn query_with_session_attaches_bearer_and_no_api_key() {
        use wiremock::matchers::header;

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/query"))
            .and(header("authorization", "Bearer jwt-abc"))
            .and(|req: &wiremock::Request| !req.headers.contains_key("x-api-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_query_body()))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .sql
            .query_with_session(&query_params(), &session())
            .await
            .unwrap();
        assert_eq!(resp.credits, 117);
        assert_eq!(resp.rows, 1);
    }

    #[cfg(feature = "payments")]
    #[tokio::test]
    async fn query_with_session_402_requires_payment_does_not_sign() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/query"))
            .respond_with(ResponseTemplate::new(402).set_body_json(serde_json::json!({
                "error": "requires_payment",
                "message": "SIWX drawdown required"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .sql
            .query_with_session(&query_params(), &session())
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::Api { status, body }
                if status.as_u16() == 402 && body.contains("requires_payment")),
            "unexpected error: {err:?}"
        );
    }

    // ── query_with_mpp_session ───────────────────────────────────────────────

    #[cfg(feature = "payments-tempo")]
    fn tempo_payment(base: &str) -> crate::config::PaymentConfig {
        crate::config::PaymentConfig {
            scheme: "mpp".into(),
            key: "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".into(),
            pay_network: "eip155:42431".into(),
            asset: "0x20c0000000000000000000000000000000000000".into(),
            max_amount: "1000000".into(),
            svm_rpc_url: None,
            base_url_override: Some(base.to_string()),
        }
    }

    #[cfg(feature = "payments-tempo")]
    fn sample_channel() -> crate::rpc::payment::session::ChannelState {
        crate::rpc::payment::session::ChannelState {
            channel_id: format!("0x{}", "11".repeat(32)),
            token: "0x20c0000000000000000000000000000000000000".into(),
            payee: "0xfd24114c3981aba78ae2441991b1bdb89329c556".into(),
            salt: format!("0x{}", "22".repeat(32)),
            authorized_signer: "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266".into(),
            escrow_contract: "0x33b901018174DDabE4841042ab76ba85D4e24f25".into(),
            deposit: 1_000_000,
            cumulative_spent: 10,
            per_call: 10,
            chain_id: 42431,
        }
    }

    #[cfg(feature = "payments-tempo")]
    fn sql_session_offer(amount: &str) -> String {
        let request = {
            use base64::Engine;
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&serde_json::json!({
                    "amount": amount,
                    "currency": "0x20c0000000000000000000000000000000000000",
                    "recipient": "0xfd24114c3981aba78ae2441991b1bdb89329c556",
                    "methodDetails": {
                        "chainId": 42431,
                        "escrowContract": "0x33b901018174DDabE4841042ab76ba85D4e24f25"
                    }
                }))
                .unwrap(),
            )
        };
        format!(
            "Payment id=\"sql1\", realm=\"mpp.quicknode.com\", method=\"tempo\", \
             intent=\"session\", description=\"d\", expires=\"2099-01-01T00:00:00Z\", \
             request=\"{request}\""
        )
    }

    #[cfg(feature = "payments-tempo")]
    fn receipt_header(accepted: &str) -> String {
        use base64::Engine;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&serde_json::json!({
                "acceptedCumulative": accepted,
                "spent": accepted,
                "status": "success",
                "intent": "session",
                "method": "tempo",
            }))
            .unwrap(),
        )
    }

    #[cfg(feature = "payments-tempo")]
    #[tokio::test]
    async fn query_with_mpp_session_uses_challenge_amount_not_per_call() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use wiremock::{Request, Respond};

        struct SqlSeq {
            calls: AtomicUsize,
        }
        impl Respond for SqlSeq {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 && !req.headers.contains_key("authorization") {
                    return ResponseTemplate::new(402)
                        .insert_header("www-authenticate", sql_session_offer("100"));
                }
                // Decode the voucher increment. increment 10 (per_call) is
                // insufficient; increment 100 (challenge amount) is accepted.
                let auth = req
                    .headers
                    .get("authorization")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");
                let b64 = auth.strip_prefix("Payment ").unwrap_or(auth);
                let cred: serde_json::Value = {
                    use base64::Engine;
                    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
                        .decode(b64.trim_end_matches('='))
                        .unwrap();
                    serde_json::from_slice(&bytes).unwrap()
                };
                let cumulative = cred["payload"]["cumulativeAmount"]
                    .as_str()
                    .unwrap()
                    .parse::<u128>()
                    .unwrap();
                // channel.cumulative_spent is 10; per_call would yield 20.
                if cumulative == 20 {
                    return ResponseTemplate::new(402).set_body_json(serde_json::json!({
                        "title": "Insufficient Balance",
                        "detail": "Insufficient balance: requested 100, available 10."
                    }));
                }
                assert_eq!(cumulative, 110, "expected challenge increment 100");
                ResponseTemplate::new(200)
                    .insert_header("payment-receipt", receipt_header("110").as_str())
                    .set_body_json(ok_query_body())
            }
        }

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/session/sql/rest/v1/query"))
            .respond_with(SqlSeq {
                calls: AtomicUsize::new(0),
            })
            .expect(2)
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let result = sdk
            .sql
            .query_with_mpp_session(
                &query_params(),
                &tempo_payment(&server.uri()),
                &sample_channel(),
            )
            .await
            .unwrap();
        assert_eq!(result.query.credits, 117);
        assert_eq!(result.accepted_cumulative, 110);
    }

    #[cfg(feature = "payments-tempo")]
    #[tokio::test]
    async fn query_with_mpp_session_insufficient_balance_does_not_resign() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use wiremock::{Request, Respond};

        struct OnceThenRefuse {
            calls: AtomicUsize,
        }
        impl Respond for OnceThenRefuse {
            fn respond(&self, req: &Request) -> ResponseTemplate {
                let n = self.calls.fetch_add(1, Ordering::SeqCst);
                if n == 0 && !req.headers.contains_key("authorization") {
                    return ResponseTemplate::new(402)
                        .insert_header("www-authenticate", sql_session_offer("100"));
                }
                ResponseTemplate::new(402).set_body_json(serde_json::json!({
                    "title": "Insufficient Balance",
                    "detail": "Insufficient balance: requested 100, available 10."
                }))
            }
        }

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/session/sql/rest/v1/query"))
            .respond_with(OnceThenRefuse {
                calls: AtomicUsize::new(0),
            })
            .expect(2)
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .sql
            .query_with_mpp_session(
                &query_params(),
                &tempo_payment(&server.uri()),
                &sample_channel(),
            )
            .await
            .unwrap_err();
        assert!(
            matches!(&err, SdkError::Api { status, body }
                if status.as_u16() == 402 && body.contains("Insufficient")),
            "unexpected error: {err:?}"
        );
    }
}
