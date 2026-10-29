use napi_derive::napi;
use quicknode_sdk::admin as core;
use serde_json::Value;

// Required nullable counts must remain present. napi omits Option::None object
// fields, so only these scalar fields use JSON null at the Node boundary.
#[napi(object)]
pub struct UsageRpcSummaryRow {
    #[napi(ts_type = "number | null")]
    pub requests: Value,
    pub bytes: f64,
    pub rpc_credits: f64,
    pub data_credits: f64,
    pub credits: f64,
    pub method: Option<Option<String>>,
    pub endpoint: Option<Option<i64>>,
    pub chain: Option<Option<String>>,
    pub network: Option<Option<String>>,
}

impl From<core::UsageRpcSummaryRow> for UsageRpcSummaryRow {
    fn from(value: core::UsageRpcSummaryRow) -> Self {
        Self {
            requests: value.requests.map_or(Value::Null, Value::from),
            bytes: value.bytes,
            rpc_credits: value.rpc_credits,
            data_credits: value.data_credits,
            credits: value.credits,
            method: value.method,
            endpoint: value.endpoint,
            chain: value.chain,
            network: value.network,
        }
    }
}

#[napi(object)]
pub struct UsageRpcSummaryData {
    #[napi(js_name = "from")]
    pub from_time: String,
    #[napi(js_name = "to")]
    pub to_time: String,
    pub totals: core::UsageRpcSummaryTotals,
    pub breakdown: Option<Vec<UsageRpcSummaryRow>>,
    pub next_cursor: Option<String>,
}

impl From<core::UsageRpcSummaryData> for UsageRpcSummaryData {
    fn from(value: core::UsageRpcSummaryData) -> Self {
        Self {
            from_time: value.from_time,
            to_time: value.to_time,
            totals: value.totals,
            breakdown: value
                .breakdown
                .map(|rows| rows.into_iter().map(UsageRpcSummaryRow::from).collect()),
            next_cursor: value.next_cursor,
        }
    }
}

#[napi(object)]
pub struct GetUsageRpcSummaryResponse {
    pub data: UsageRpcSummaryData,
    pub error: Option<String>,
}

impl From<core::GetUsageRpcSummaryResponse> for GetUsageRpcSummaryResponse {
    fn from(value: core::GetUsageRpcSummaryResponse) -> Self {
        Self {
            data: value.data.into(),
            error: value.error,
        }
    }
}

#[napi(object)]
pub struct TrafficSummaryRow {
    #[napi(ts_type = "number | null")]
    pub successful_requests: Value,
    pub bytes: f64,
    pub request_credits: f64,
    pub bytes_credits: f64,
    pub credits: f64,
    pub active_methods: f64,
    pub method: Option<Option<String>>,
    pub endpoint: Option<Option<i64>>,
}

impl From<core::TrafficSummaryRow> for TrafficSummaryRow {
    fn from(value: core::TrafficSummaryRow) -> Self {
        Self {
            successful_requests: value.successful_requests.map_or(Value::Null, Value::from),
            bytes: value.bytes,
            request_credits: value.request_credits,
            bytes_credits: value.bytes_credits,
            credits: value.credits,
            active_methods: value.active_methods,
            method: value.method,
            endpoint: value.endpoint,
        }
    }
}

#[napi(object)]
pub struct TrafficSummaryData {
    #[napi(js_name = "from")]
    pub from_time: String,
    #[napi(js_name = "to")]
    pub to_time: String,
    pub totals: core::TrafficSummaryTotals,
    pub breakdown: Option<Vec<TrafficSummaryRow>>,
    pub next_cursor: Option<String>,
}

impl From<core::TrafficSummaryData> for TrafficSummaryData {
    fn from(value: core::TrafficSummaryData) -> Self {
        Self {
            from_time: value.from_time,
            to_time: value.to_time,
            totals: value.totals,
            breakdown: value
                .breakdown
                .map(|rows| rows.into_iter().map(TrafficSummaryRow::from).collect()),
            next_cursor: value.next_cursor,
        }
    }
}

#[napi(object)]
pub struct GetTrafficSummaryResponse {
    pub data: TrafficSummaryData,
    pub error: Option<String>,
}

impl From<core::GetTrafficSummaryResponse> for GetTrafficSummaryResponse {
    fn from(value: core::GetTrafficSummaryResponse) -> Self {
        Self {
            data: value.data.into(),
            error: value.error,
        }
    }
}
