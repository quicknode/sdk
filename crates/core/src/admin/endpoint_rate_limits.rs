#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// A per-method rate limiter configured on an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodRateLimiter {
    /// Rate limiter identifier.
    pub id: String,
    /// Interval over which the rate applies (e.g. `second`, `minute`).
    pub interval: String,
    /// RPC methods the limiter applies to.
    #[serde(default)]
    pub methods: Vec<String>,
    /// Maximum number of calls allowed per interval.
    pub rate: i32,
    /// Whether the limiter is `enabled` or `disabled`.
    pub status: String,
    /// Creation timestamp.
    pub created: String,
}

/// Inner data for `get_method_rate_limits`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMethodRateLimitsData {
    /// Rate limiters configured on the endpoint.
    #[serde(default)]
    pub rate_limiters: Vec<MethodRateLimiter>,
}

/// Response from `get_method_rate_limits`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMethodRateLimitsResponse {
    /// Rate limiters payload.
    pub data: Option<GetMethodRateLimitsData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `create_method_rate_limit`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateMethodRateLimitRequest {
    /// Interval over which the rate applies (e.g. `second`).
    pub interval: String,
    /// RPC methods the limiter applies to.
    pub methods: Vec<String>,
    /// Maximum number of calls allowed per interval.
    pub rate: i32,
}

/// Response from `create_method_rate_limit`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMethodRateLimitResponse {
    /// The created rate limiter.
    pub data: Option<MethodRateLimiter>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `update_method_rate_limit`. Only provided fields are changed.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateMethodRateLimitRequest {
    /// New set of RPC methods the limiter applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    /// New status (`enabled` or `disabled`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// New rate value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<i32>,
}

/// Response from `update_method_rate_limit`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMethodRateLimitResponse {
    /// The updated rate limiter.
    pub data: Option<MethodRateLimiter>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Endpoint-wide rate limit settings.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct RateLimitSettings {
    /// Requests per second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rps: Option<i32>,
    /// Requests per minute.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpm: Option<i32>,
    /// Requests per day.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpd: Option<i32>,
}

/// Parameters for `update_rate_limits`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRateLimitsRequest {
    /// Rate limit values to apply.
    pub rate_limits: RateLimitSettings,
}

/// A single rate-limit row returned by `get_rate_limits`, identifying the
/// bucket (`rps`/`rpm`/`rpd`), the value enforced, and whether the value comes
/// from the plan default or a user-set override.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEntry {
    /// Which bucket this row applies to: `rps`, `rpm`, or `rpd`.
    pub bucket: String,
    /// The enforced value for this bucket.
    pub rate_limit: i32,
    /// Where the value comes from: `plan_default` or `user_override`.
    pub source: String,
    /// Row identifier. Present on `user_override` rows — pass it to
    /// `delete_rate_limit_override` to remove the override. May be absent on
    /// `plan_default` rows and cannot be deleted there.
    #[serde(default)]
    pub id: Option<String>,
}

/// Inner data for `get_rate_limits`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRateLimitsData {
    /// One row per enforced bucket.
    #[serde(default)]
    pub rate_limits: Vec<RateLimitEntry>,
}

/// Response from `get_rate_limits`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRateLimitsResponse {
    /// Rate-limit rows with their source.
    pub data: Option<GetRateLimitsData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
