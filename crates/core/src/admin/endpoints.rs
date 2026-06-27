#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// Parameters for `get_endpoints`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEndpointsRequest {
    /// Maximum number of endpoints returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Starting index into the result set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i32>,
    /// Search by subdomain or label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// Field to sort results by.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    /// Sort direction (`asc` or `desc`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<String>,
    /// Filter results to endpoints on these networks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    /// Filter results to endpoints in these statuses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
    /// Filter results by label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    /// When true, return only dedicated endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated: Option<bool>,
    /// When true, return only flat-rate endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_flat_rate: Option<bool>,
    /// Filter results by associated tag ids.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<i32>>,
    /// Filter results by associated tag labels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_labels: Option<Vec<String>>,
}

/// Response from `get_endpoints`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEndpointsResponse {
    /// Endpoints on the current page.
    #[serde(default)]
    pub data: Vec<Endpoint>,
    /// Pagination metadata for the response.
    pub pagination: Option<Pagination>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Pagination metadata for admin list responses.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Total number of items matching the query across all pages.
    pub total: i64,
    /// Page size used for this response.
    pub limit: i32,
    /// Starting index of this page within the full result set.
    pub offset: i32,
}

/// Summary representation of an endpoint in list responses.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Unique endpoint identifier.
    pub id: String,
    /// Quicknode-assigned subdomain.
    pub name: String,
    /// Human-readable label.
    pub label: Option<String>,
    /// Current operational status (e.g. `active`, `paused`).
    pub status: String,
    /// Blockchain the endpoint serves (e.g. `ethereum`).
    pub chain: String,
    /// Specific network within the chain (e.g. `mainnet`).
    pub network: String,
    /// Whether the endpoint is dedicated.
    pub is_dedicated: bool,
    /// Whether the endpoint is billed at a flat rate.
    pub is_flat_rate: bool,
    /// HTTP RPC URL.
    pub http_url: String,
    /// WebSocket RPC URL, when available.
    pub wss_url: Option<String>,
    /// Tags applied to the endpoint.
    #[serde(default)]
    pub tags: Vec<EndpointTag>,
    /// Whether the endpoint is configured to serve multiple chains/networks.
    #[serde(default)]
    pub is_multichain: bool,
}

/// Tag reference as returned on an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTag {
    /// Tag identifier.
    pub tag_id: i32,
    /// Tag label.
    pub label: String,
}

/// Parameters for `create_endpoint`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateEndpointRequest {
    /// Blockchain the endpoint should serve (e.g. `ethereum`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    /// Specific network within the chain (e.g. `mainnet`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

/// Response from `create_endpoint`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEndpointResponse {
    /// The newly created endpoint.
    pub data: SingleEndpoint,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Full representation of a single endpoint, including its security and rate limits.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleEndpoint {
    /// Unique endpoint identifier.
    pub id: String,
    /// Human-readable label.
    pub label: Option<String>,
    /// Current operational status.
    pub status: Option<String>,
    /// Blockchain the endpoint serves.
    pub chain: String,
    /// Specific network within the chain.
    pub network: String,
    /// HTTP RPC URL.
    pub http_url: String,
    /// WebSocket RPC URL, when available.
    pub wss_url: Option<String>,
    /// Endpoint security configuration.
    pub security: Option<EndpointSecurity>,
    /// Endpoint rate limits.
    pub rate_limits: Option<EndpointRateLimits>,
    /// Tags applied to the endpoint.
    #[serde(default)]
    pub tags: Vec<EndpointTag>,
    /// Whether the endpoint is configured to serve multiple chains/networks.
    #[serde(default)]
    pub is_multichain: bool,
}

/// Rate limits applied to an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRateLimits {
    /// Whether rate limits are applied per client IP instead of per endpoint.
    pub rate_limit_by_ip: Option<bool>,
    /// Account-level rate limit, when applicable.
    pub account: Option<i32>,
    /// Requests per second.
    pub rps: Option<i32>,
    /// Requests per minute.
    pub rpm: Option<i32>,
    /// Requests per day.
    pub rpd: Option<i32>,
}

/// Security configuration for an endpoint — the aggregate of tokens, JWTs,
/// referrers, domain masks, IPs, and request filters plus their enabled
/// toggles.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSecurity {
    /// Per-feature enabled/disabled toggles.
    pub options: Option<EndpointSecurityOptions>,
    /// Authentication tokens configured on the endpoint.
    pub tokens: Option<Vec<EndpointToken>>,
    /// JWTs configured on the endpoint.
    pub jwts: Option<Vec<EndpointJwt>>,
    /// Allowed referrer URLs/domains.
    pub referrers: Option<Vec<EndpointReferrer>>,
    /// Configured domain masks.
    pub domain_masks: Option<Vec<EndpointDomainMask>>,
    /// Whitelisted IP addresses.
    pub ips: Option<Vec<EndpointIp>>,
    /// Request (method) filters.
    pub request_filters: Option<Vec<EndpointRequestFilter>>,
}

/// Boolean toggles controlling which security features are enabled.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSecurityOptions {
    /// Whether token authentication is enforced.
    pub tokens: Option<bool>,
    /// Whether JWT validation is enforced.
    pub jwts: Option<bool>,
    /// Whether domain masking is enabled.
    #[serde(rename = "domainMasks")]
    pub domain_masks: Option<bool>,
    /// Whether IP whitelisting is enforced.
    pub ips: Option<bool>,
    /// Whether referrer validation is enforced.
    pub referrers: Option<bool>,
    /// Whether request (method) filtering is enforced.
    #[serde(rename = "requestFilters")]
    pub request_filters: Option<bool>,
    /// Custom header used to identify the client IP.
    #[serde(rename = "ipCustomHeader")]
    pub ip_custom_header: Option<EndpointIpCustomHeaderOption>,
}

/// Custom header option value for IP identification.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointIpCustomHeaderOption {
    /// Header name (e.g. `X-Forwarded-For`).
    pub value: Option<String>,
}

/// Authentication token configured on an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Clone, Serialize, Deserialize)]
pub struct EndpointToken {
    /// Token identifier.
    pub id: String,
    /// Token secret.
    pub token: String,
}

// Manual Debug redacts `token` so accidental println!/tracing of an
// EndpointSecurity response does not leak the raw RPC access token.
impl std::fmt::Debug for EndpointToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EndpointToken")
            .field("id", &self.id)
            .field("token", &"[redacted]")
            .finish()
    }
}

/// JWT configured on an endpoint for signed-request authentication.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Clone, Serialize, Deserialize)]
pub struct EndpointJwt {
    /// JWT identifier.
    pub id: String,
    /// Public key used to verify signed JWTs.
    pub public_key: String,
    /// Key identifier (`kid`) embedded in JWT headers.
    pub kid: String,
    /// Human-readable name.
    pub name: String,
}

// Manual Debug redacts `public_key` — credential material per CLAUDE.md policy.
impl std::fmt::Debug for EndpointJwt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EndpointJwt")
            .field("id", &self.id)
            .field("public_key", &"[redacted]")
            .field("kid", &self.kid)
            .field("name", &self.name)
            .finish()
    }
}

/// Allowed referrer entry for request-origin validation.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointReferrer {
    /// Referrer entry identifier.
    pub id: String,
    /// Allowed referrer URL or domain.
    pub referrer: Option<String>,
}

/// Domain mask configured on an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointDomainMask {
    /// Domain mask identifier.
    pub id: String,
    /// Masking domain.
    pub domain: String,
}

/// Whitelisted IP address on an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointIp {
    /// IP entry identifier.
    pub id: String,
    /// Whitelisted IP address.
    pub ip: String,
}

/// Request (method) filter configured on an endpoint.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRequestFilter {
    /// Filter identifier.
    pub id: String,
    /// Whitelisted RPC methods.
    #[serde(default)]
    pub method: Vec<String>,
}

/// Response from `show_endpoint`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowEndpointResponse {
    /// The endpoint, when found.
    pub data: Option<SingleEndpoint>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `update_endpoint`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateEndpointRequest {
    /// New human-readable label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Parameters for `update_endpoint_status`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateEndpointStatusRequest {
    /// New status (`active` or `paused`).
    pub status: String,
}

/// Response from `update_endpoint_status`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEndpointStatusResponse {
    /// Confirmation string returned by the API.
    pub data: Option<String>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `create_tag` (on a specific endpoint).
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateTagRequest {
    /// Label for the new tag. Maximum 25 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Response from `get_endpoint_security`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEndpointSecurityResponse {
    /// The endpoint's security configuration.
    pub data: Option<EndpointSecurity>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
