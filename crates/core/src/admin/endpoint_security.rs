use super::endpoints::{EndpointIp, EndpointJwt, EndpointToken};
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
    /// Response logging toggle.
    #[serde(rename = "responseLogging", skip_serializing_if = "Option::is_none")]
    pub response_logging: Option<String>,
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
    pub referrer: String,
}

/// Parameters for `create_ip`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateIpRequest {
    /// IP address to whitelist.
    pub ip: String,
}

/// Parameters for `create_domain_mask`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateDomainMaskRequest {
    /// Custom domain that will mask the endpoint's Quicknode URL; required by the API.
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
    pub public_key: String,
    /// Key identifier (`kid`) embedded in JWT headers.
    pub kid: String,
    /// Human-readable name for the JWT configuration.
    pub name: String,
}

/// Parameters for `create_request_filter`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateRequestFilterRequest {
    /// Whitelisted RPC methods; other methods will be blocked.
    pub method: Vec<String>,
}

/// Response from `create_request_filter`.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestFilterResponse {
    /// The created filter payload.
    pub data: Option<CreateRequestFilterData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Data wrapper for a created request filter.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestFilterData {
    /// Identifier of the newly created request filter.
    pub id: String,
    /// Custom pattern, when returned by the API.
    #[cfg_attr(feature = "node", napi(ts_type = "Record<string, unknown>"))]
    pub pattern: Option<std::collections::HashMap<String, serde_json::Value>>,
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

/// Custom JSON pattern for a request filter.
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize)]
pub struct CreateRequestFilterPatternRequest {
    #[cfg_attr(feature = "node", napi(ts_type = "Record<string, unknown>"))]
    pub pattern: std::collections::HashMap<String, serde_json::Value>,
}

/// Custom JSON pattern for a request filter.
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize)]
pub struct UpdateRequestFilterPatternRequest {
    #[cfg_attr(feature = "node", napi(ts_type = "Record<string, unknown>"))]
    pub pattern: std::collections::HashMap<String, serde_json::Value>,
}

/// Filter as returned by the request-filter list API.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestFilter {
    pub id: String,
    #[cfg_attr(feature = "node", napi(ts_type = "Record<string, unknown>"))]
    pub pattern: std::collections::HashMap<String, serde_json::Value>,
    pub method_whitelist: bool,
}

/// Response from `get_request_filters`.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRequestFiltersResponse {
    pub data: Vec<RequestFilter>,
    pub error: Option<String>,
}

/// Created security record.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedReferrer {
    pub id: String,
    pub url: String,
}

/// Created security record.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedDomainMask {
    pub id: String,
    pub domain_mask: String,
}

/// Created security record and API error.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    pub data: EndpointToken,
    pub error: Option<String>,
}

/// Created security record and API error.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReferrerResponse {
    pub data: CreatedReferrer,
    pub error: Option<String>,
}

/// Created security record and API error.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIpResponse {
    pub data: EndpointIp,
    pub error: Option<String>,
}

/// Created security record and API error.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDomainMaskResponse {
    pub data: CreatedDomainMask,
    pub error: Option<String>,
}

/// Created security record and API error.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJwtResponse {
    pub data: EndpointJwt,
    pub error: Option<String>,
}

/// Client certificate metadata.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCertificate {
    pub fingerprint: String,
    pub subject: String,
    pub issuer: String,
    pub expires_at: String,
}

/// Current client-certificate authentication state.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCertsData {
    pub enabled: bool,
    pub certs: Vec<ClientCertificate>,
}

/// PEM certificate to add.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientCertRequest {
    pub certificate: String,
}

/// Enable or disable client-certificate authentication.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientCertsRequest {
    pub enabled: bool,
}

/// Full certificate state after the operation.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetClientCertsResponse {
    pub data: ClientCertsData,
    pub error: Option<String>,
}

/// Full certificate state after the operation.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientCertResponse {
    pub data: ClientCertsData,
    pub error: Option<String>,
}

/// Full certificate state after the operation.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateClientCertsResponse {
    pub data: ClientCertsData,
    pub error: Option<String>,
}

/// Full certificate state after the operation.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteClientCertResponse {
    pub data: ClientCertsData,
    pub error: Option<String>,
}

/// Blocked methods.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodBlacklistData {
    pub methods: Vec<String>,
}

/// Replacement blacklist; an empty list clears it.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMethodBlacklistRequest {
    pub methods: Vec<String>,
}

/// Current method blacklist.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMethodBlacklistResponse {
    pub data: MethodBlacklistData,
    pub error: Option<String>,
}

/// Current method blacklist.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMethodBlacklistResponse {
    pub data: MethodBlacklistData,
    pub error: Option<String>,
}

/// Range limits and supported methods for this endpoint chain.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeLimitsData {
    pub limits: std::collections::HashMap<String, i64>,
    pub supported_methods: Vec<String>,
}

/// Replacement positive range limits; an empty object clears them.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRangeLimitsRequest {
    pub limits: std::collections::HashMap<String, i64>,
}

/// Current method range limits.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetRangeLimitsResponse {
    pub data: RangeLimitsData,
    pub error: Option<String>,
}

/// Current method range limits.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRangeLimitsResponse {
    pub data: RangeLimitsData,
    pub error: Option<String>,
}

/// Blocked IP address or CIDR range. Does not enable IP security.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBlockedIpRequest {
    pub ip: String,
}

/// Blocked IP records visible for the current security state.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlockedIpsResponse {
    pub data: Vec<EndpointIp>,
    pub error: Option<String>,
}

/// Created blocked IP record.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBlockedIpResponse {
    pub data: EndpointIp,
    pub error: Option<String>,
}
