use napi_derive::napi;
use quicknode_sdk as core;

// Node-facing SQL response types.
//
// Two reasons these wrap the core types rather than exposing them directly:
//  1. `QueryResponse.data` holds `serde_json::Value` rows whose shape depends on
//     the SQL query; napi serializes `serde_json::Value` to a plain JS object.
//  2. The column-type field is `column_type` in core (`type` is a Rust keyword).
//     napi would emit it as `columnType`; these wrappers expose it as `type` to
//     match the REST API and the other language bindings.

#[napi(object)]
pub struct ColumnMetaNode {
    /// Column name as it appears in the result set.
    pub name: String,
    /// Column data type (e.g. `"DateTime('UTC')"`).
    #[napi(js_name = "type")]
    pub type_: String,
}

impl From<core::sql::ColumnMeta> for ColumnMetaNode {
    fn from(c: core::sql::ColumnMeta) -> Self {
        Self {
            name: c.name,
            type_: c.column_type,
        }
    }
}

#[napi(object)]
pub struct QueryStatisticsNode {
    /// Total query execution time in seconds.
    pub elapsed: f64,
    /// Total number of rows scanned during execution.
    pub rows_read: i64,
    /// Total data scanned in bytes.
    pub bytes_read: i64,
}

impl From<core::sql::QueryStatistics> for QueryStatisticsNode {
    fn from(s: core::sql::QueryStatistics) -> Self {
        Self {
            elapsed: s.elapsed,
            rows_read: s.rows_read,
            bytes_read: s.bytes_read,
        }
    }
}

#[napi(object)]
pub struct QueryResponseNode {
    /// Column metadata for each column in the result set.
    pub meta: Vec<ColumnMetaNode>,
    /// Result rows. Each row is an object keyed by the selected columns; shape
    /// varies per query.
    pub data: Vec<serde_json::Value>,
    /// Number of rows returned in this response.
    pub rows: i64,
    /// Total rows that matched the query before applying `LIMIT`.
    pub rows_before_limit_at_least: i64,
    /// Query execution statistics.
    pub statistics: QueryStatisticsNode,
    /// Credits consumed by the query.
    pub credits: i64,
}

impl From<core::sql::QueryResponse> for QueryResponseNode {
    fn from(r: core::sql::QueryResponse) -> Self {
        Self {
            meta: r.meta.into_iter().map(ColumnMetaNode::from).collect(),
            data: r.data,
            rows: r.rows,
            rows_before_limit_at_least: r.rows_before_limit_at_least,
            statistics: r.statistics.into(),
            credits: r.credits,
        }
    }
}

#[napi(object)]
pub struct ColumnSchemaNode {
    /// Column name.
    pub name: String,
    /// Column data type (e.g. `"UInt64"`, `"FixedString(42)"`).
    #[napi(js_name = "type")]
    pub type_: String,
}

impl From<core::sql::ColumnSchema> for ColumnSchemaNode {
    fn from(c: core::sql::ColumnSchema) -> Self {
        Self {
            name: c.name,
            type_: c.column_type,
        }
    }
}

#[napi(object)]
pub struct TableSchemaNode {
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
    pub columns: Vec<ColumnSchemaNode>,
}

impl From<core::sql::TableSchema> for TableSchemaNode {
    fn from(t: core::sql::TableSchema) -> Self {
        Self {
            name: t.name,
            engine: t.engine,
            total_rows: t.total_rows,
            partition_key: t.partition_key,
            sorting_key: t.sorting_key,
            columns: t.columns.into_iter().map(ColumnSchemaNode::from).collect(),
        }
    }
}

#[napi(object)]
pub struct ChainSchemaNode {
    /// Human-readable chain name.
    pub chain: String,
    /// Cluster identifier the schema belongs to.
    pub cluster_id: String,
    /// Tables available in this cluster.
    pub tables: Vec<TableSchemaNode>,
}

impl From<core::sql::ChainSchema> for ChainSchemaNode {
    fn from(s: core::sql::ChainSchema) -> Self {
        Self {
            chain: s.chain,
            cluster_id: s.cluster_id,
            tables: s.tables.into_iter().map(TableSchemaNode::from).collect(),
        }
    }
}
