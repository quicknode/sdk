#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// Parameters for `get_endpoint_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEndpointMetricsRequest {
    /// Time period (`hour`, `day`, `week`, or `month`).
    pub period: String,
    /// Metric name (e.g. `method_calls_over_time`, `response_status_breakdown`).
    pub metric: String,
}

/// Parameters for `get_account_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetAccountMetricsRequest {
    /// Time period (`hour`, `day`, `week`, or `month`).
    pub period: String,
    /// Metric name (e.g. `method_calls_over_time`, `credits_over_time`).
    pub metric: String,
    /// Optional percentile for latency metrics (e.g. `p50`, `p95`, `p99`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentile: Option<String>,
}

/// A single metric series, consisting of a descriptive tag and timestamped data points.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetric {
    /// Data points, each as `[timestamp, value]`.
    pub data: Vec<Vec<i64>>,
    /// Human-readable tag identifying the series.
    pub tag: String,
}

/// Response from `get_endpoint_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEndpointMetricsResponse {
    /// Metric series returned for the endpoint.
    #[serde(default)]
    pub data: Vec<EndpointMetric>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Response from `get_account_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountMetricsResponse {
    /// Metric series returned for the account.
    #[serde(default)]
    pub data: Vec<EndpointMetric>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
