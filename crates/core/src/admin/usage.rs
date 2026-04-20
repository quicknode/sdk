#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// Parameters for the account usage methods (`get_usage`, `get_usage_by_*`).
/// Both bounds are optional; omit for account-to-date totals.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetUsageRequest {
    /// Start of the query window (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// End of the query window (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

/// Aggregate account usage for a time window.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    /// Credits consumed during the window.
    pub credits_used: i64,
    /// Credits still available, when the plan has a finite limit.
    pub credits_remaining: Option<i64>,
    /// Plan's credit limit, when applicable.
    pub limit: Option<i64>,
    /// Credits consumed beyond the plan limit.
    pub overages: Option<i64>,
    /// Start of the queried window.
    pub start_time: i64,
    /// End of the queried window.
    pub end_time: i64,
}

/// Response from `get_usage`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageResponse {
    /// Aggregate usage payload.
    pub data: Option<UsageData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Per-endpoint usage row.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointUsage {
    /// Endpoint subdomain.
    pub name: String,
    /// Blockchain the endpoint serves.
    pub chain: Option<String>,
    /// Network within the chain.
    pub network: Option<String>,
    /// Operational status during the window.
    pub status: Option<String>,
    /// Total credits consumed by this endpoint.
    pub credits_used: i64,
    /// Human-readable label.
    pub label: Option<String>,
    /// Per-method credit breakdown.
    #[serde(default)]
    pub methods_breakdown: Vec<MethodUsage>,
    /// Request count during the window.
    pub requests: Option<i64>,
}

/// Per-method usage row.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodUsage {
    /// RPC method name.
    pub method_name: String,
    /// Credits consumed by this method.
    pub credits_used: i64,
    /// Whether the call required an archival node.
    pub archive: Option<bool>,
    /// Network the calls targeted.
    pub network: Option<String>,
    /// Chain the calls targeted.
    pub chain: Option<String>,
}

/// Per-chain usage row.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainUsage {
    /// Chain name or slug.
    pub name: String,
    /// Credits consumed on the chain.
    pub credits_used: i64,
}

/// Inner data for `get_usage_by_endpoint`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByEndpointData {
    /// Per-endpoint rows.
    #[serde(default)]
    pub endpoints: Vec<EndpointUsage>,
    /// Start of the queried window.
    pub start_time: Option<i64>,
    /// End of the queried window.
    pub end_time: Option<i64>,
}

/// Response from `get_usage_by_endpoint`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageByEndpointResponse {
    /// Per-endpoint usage payload.
    pub data: Option<UsageByEndpointData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Inner data for `get_usage_by_method`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByMethodData {
    /// Per-method rows.
    #[serde(default)]
    pub methods: Vec<MethodUsage>,
    /// Start of the queried window.
    pub start_time: Option<i64>,
    /// End of the queried window.
    pub end_time: Option<i64>,
}

/// Response from `get_usage_by_method`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageByMethodResponse {
    /// Per-method usage payload.
    pub data: Option<UsageByMethodData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Inner data for `get_usage_by_chain`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByChainData {
    /// Per-chain rows.
    #[serde(default)]
    pub chains: Vec<ChainUsage>,
    /// Start of the queried window.
    pub start_time: Option<i64>,
    /// End of the queried window.
    pub end_time: Option<i64>,
}

/// Response from `get_usage_by_chain`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageByChainResponse {
    /// Per-chain usage payload.
    pub data: Option<UsageByChainData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Per-tag usage row.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagUsage {
    /// Tag identifier.
    pub tag_id: Option<i32>,
    /// Tag label.
    pub label: String,
    /// Credits consumed by endpoints with this tag.
    pub credits_used: i64,
    /// Request count during the window.
    pub requests: i64,
}

/// Inner data for `get_usage_by_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageByTagData {
    /// Per-tag rows.
    #[serde(default)]
    pub tags: Vec<TagUsage>,
    /// Start of the queried window.
    pub start_time: Option<i64>,
    /// End of the queried window.
    pub end_time: Option<i64>,
}

/// Response from `get_usage_by_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageByTagResponse {
    /// Per-tag usage payload.
    pub data: Option<UsageByTagData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
