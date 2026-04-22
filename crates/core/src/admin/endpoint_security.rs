#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// A single security feature's name, status, and optional value.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityOption {
    /// Name of the security feature (e.g. `tokens`, `jwts`, `ips`).
    pub option: String,
    /// Whether the feature is `enabled` or `disabled`.
    pub status: String,
    /// Optional configuration value associated with the feature.
    pub value: Option<String>,
}

/// Response from `get_security_options`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSecurityOptionsResponse {
    /// Security options on the endpoint.
    #[serde(default)]
    pub data: Vec<SecurityOption>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Per-feature toggles for `update_security_options`. Each field accepts
/// `enabled` or `disabled`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct SecurityOptionsUpdate {
    /// Token authentication toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<String>,
    /// Referrer validation toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrers: Option<String>,
    /// JWT validation toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwts: Option<String>,
    /// IP whitelist toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ips: Option<String>,
    /// Domain masking toggle.
    #[serde(rename = "domainMasks", skip_serializing_if = "Option::is_none")]
    pub domain_masks: Option<String>,
    /// HSTS (HTTP Strict Transport Security) toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hsts: Option<String>,
    /// CORS toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<String>,
    /// Request (method) filter toggle.
    #[serde(rename = "requestFilters", skip_serializing_if = "Option::is_none")]
    pub request_filters: Option<String>,
    /// Custom IP header toggle.
    #[serde(rename = "ipCustomHeader", skip_serializing_if = "Option::is_none")]
    pub ip_custom_header: Option<String>,
}

/// Parameters for `update_security_options`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateSecurityOptionsRequest {
    /// Security toggles to apply.
    pub options: SecurityOptionsUpdate,
}

/// Response from `update_security_options`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSecurityOptionsResponse {
    /// Updated security options.
    #[serde(default)]
    pub data: Vec<SecurityOption>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `create_referrer`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateReferrerRequest {
    /// Allowed referrer URL or domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
}

/// Parameters for `create_ip`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateIpRequest {
    /// IP address to whitelist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
}

/// Parameters for `create_domain_mask`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDomainMaskRequest {
    /// Custom domain that will mask the endpoint's Quicknode URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_mask: Option<String>,
}

/// Parameters for `create_jwt`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateJwtRequest {
    /// Public key used to verify signed JWTs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Key identifier (`kid`) embedded in JWT headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    /// Human-readable name for the JWT configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Parameters for `create_request_filter`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRequestFilterRequest {
    /// Whitelisted RPC methods; other methods will be blocked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<Vec<String>>,
}

/// Response from `create_request_filter`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestFilterResponse {
    /// The created filter payload.
    pub data: Option<CreateRequestFilterData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Data wrapper for a created request filter.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestFilterData {
    /// Identifier of the newly created request filter.
    pub id: String,
}

/// Parameters for `update_request_filter`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateRequestFilterRequest {
    /// New set of whitelisted RPC methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<Vec<String>>,
}

/// Parameters for `create_or_update_ip_custom_header`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateOrUpdateIpCustomHeaderRequest {
    /// Header name used to identify the client IP (e.g. `X-Forwarded-For`).
    pub header_name: String,
}

/// Data wrapper for the IP custom header configuration.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpCustomHeaderData {
    /// Configured header name.
    pub header_name: String,
}

/// Response from `create_or_update_ip_custom_header`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrUpdateIpCustomHeaderResponse {
    /// Stored header configuration.
    pub data: Option<IpCustomHeaderData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Response wrapper for delete operations that return a boolean success flag.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBoolResponse {
    /// `true` when the deletion succeeded.
    pub data: Option<bool>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
