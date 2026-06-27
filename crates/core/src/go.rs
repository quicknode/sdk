//! Go binding facade (UniFFI), compiled only under the `go` feature.
//!
//! This module is the synchronous, FFI-friendly surface that
//! `uniffi-bindgen-go` turns into the Go package. It holds no HTTP or business
//! logic: it wraps the async core API with a shared Tokio runtime + `block_on`
//! (the same approach as the Ruby binding) and maps [`SdkError`] to a
//! UniFFI-representable error enum at the boundary.
//!
//! It lives inside the core crate (rather than a separate facade crate) because
//! uniffi 0.31 requires the crate that derives `uniffi::Record`/`Error` on the
//! data types to be the same crate that calls `setup_scaffolding!`. Everything
//! here is gated on the `go` feature, so default and python/node/ruby builds
//! are unaffected.
//!
//! Shape: a root [`QuicknodeSdkClient`] exposes one accessor per product area
//! (`admin()`, `streams()`, `webhooks()`, `kvstore()`, `sql()`) returning a
//! sub-client object, mirroring the core `QuicknodeSdk` struct. Each sub-client
//! holds a clone of the core client (cheap — backed by `Arc<SdkConfigInner>`).

// `#[uniffi::export]` methods must take owned arguments — references cannot
// cross the FFI boundary — so `needless_pass_by_value` is a false positive
// here. Runtime initialization failure is unrecoverable at this boundary, so
// `expect` is the honest choice (the Ruby binding allows the same crate-wide).
#![allow(clippy::needless_pass_by_value, clippy::expect_used)]

use std::sync::Arc;
use std::sync::OnceLock;

use crate::admin::{
    AdminApiClient, BulkAddTagRequest, BulkAddTagResponse, BulkRemoveTagRequest,
    BulkRemoveTagResponse, BulkUpdateEndpointStatusRequest, BulkUpdateEndpointStatusResponse,
    CreateDomainMaskRequest, CreateEndpointRequest, CreateEndpointResponse, CreateIpRequest,
    CreateJwtRequest, CreateMethodRateLimitRequest, CreateMethodRateLimitResponse,
    CreateOrUpdateIpCustomHeaderRequest, CreateOrUpdateIpCustomHeaderResponse,
    CreateReferrerRequest, CreateRequestFilterRequest, CreateRequestFilterResponse,
    CreateTagRequest, CreateTeamRequest, CreateTeamResponse, DeleteAccountTagResponse,
    DeleteBoolResponse, DeleteTeamResponse, GetAccountMetricsRequest, GetAccountMetricsResponse,
    GetEndpointLogsRequest, GetEndpointLogsResponse, GetEndpointMetricsRequest,
    GetEndpointMetricsResponse, GetEndpointSecurityResponse, GetEndpointUrlsResponse,
    GetEndpointsRequest, GetEndpointsResponse, GetLogDetailsResponse, GetMethodRateLimitsResponse,
    GetRateLimitsResponse, GetSecurityOptionsResponse, GetTeamResponse, GetUsageByChainResponse,
    GetUsageByEndpointResponse, GetUsageByMethodResponse, GetUsageByTagResponse, GetUsageRequest,
    GetUsageResponse, InviteTeamMemberRequest, InviteTeamMemberResponse, ListChainsResponse,
    ListInvoicesResponse, ListPaymentsResponse, ListTagsResponse, ListTeamEndpointsResponse,
    ListTeamsResponse, RemoveTeamMemberRequest, RemoveTeamMemberResponse, RenameTagRequest,
    RenameTagResponse, ResendTeamInviteResponse, ShowEndpointResponse, UpdateEndpointRequest,
    UpdateEndpointStatusRequest, UpdateEndpointStatusResponse, UpdateMethodRateLimitRequest,
    UpdateMethodRateLimitResponse, UpdateRateLimitsRequest, UpdateRequestFilterRequest,
    UpdateSecurityOptionsRequest, UpdateSecurityOptionsResponse, UpdateTeamEndpointsRequest,
    UpdateTeamEndpointsResponse,
};
use crate::config::{
    AdminConfig, KvStoreConfig, SdkFullConfig, SqlConfig, StreamsConfig, WebhooksConfig,
};
use crate::errors::{HttpKind, SdkError};
use crate::kvstore::{
    AddListItemParams, BulkSetsParams, CreateListParams, CreateSetParams, GetListParams,
    GetListResponse, GetListsParams, GetListsResponse, GetSetResponse, GetSetsParams,
    GetSetsResponse, KvStoreApiClient, ListContainsItemResponse, UpdateListParams,
};
use crate::sql::{ChainSchema, QueryParams, QueryResponse, SqlApiClient};
// Imported as a bare ident because `uniffi::custom_type!` requires a
// single-component type name (it calls `path.get_ident()` internally).
use crate::streams::{
    CreateStreamParams, EnabledCountResponse, ListStreamsParams, ListStreamsResponse, Stream,
    StreamsApiClient, TestFilterParams, TestFilterResponse, UpdateStreamParams,
};
use crate::webhooks::{
    ActivateWebhookParams, CreateWebhookFromTemplateParams, GetWebhooksParams,
    ListWebhooksResponse, UpdateWebhookParams, UpdateWebhookTemplateParams, Webhook,
    WebhookEnabledCountResponse, WebhooksApiClient,
};
use crate::{ClientInfo, QuicknodeSdk};
use serde_json::Value as JsonValue;

// SQL query result rows are arbitrary JSON whose shape depends on the query, so
// they cannot be a fixed uniffi Record. Marshal each `serde_json::Value` as a
// JSON string; Go callers `json.Unmarshal` it. This is the Go analog of how the
// other bindings expose `data` as their native dynamic type. `serde_json::Value`
// is a foreign type, so the `remote` flag is required to sidestep the orphan
// rule. `Vec<serde_json::Value>` then crosses as Go `[]string`.
uniffi::custom_type!(JsonValue, String, {
    remote,
    lower: |v| serde_json::to_string(&v).unwrap_or_else(|_| "null".to_string()),
    try_lift: |s| Ok(serde_json::from_str(&s)?),
});

// A single shared runtime for all blocking HTTP calls, mirroring the Ruby
// binding. `uniffi-bindgen-go` surfaces calls as blocking on the Go side
// regardless, so exposing a synchronous API here costs no ergonomics; Go
// callers do concurrency with their own goroutines.
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME
        .get_or_init(|| tokio::runtime::Runtime::new().expect("failed to initialize tokio runtime"))
}

/// Errors surfaced across the Go FFI boundary. Mirrors the typed hierarchy used
/// by the other bindings (see CLAUDE.md §Error Handling): `Config` covers
/// configuration/URL-parse failures; `Http`/`Timeout`/`Connection` cover
/// transport-level failures; `Api` carries the HTTP status and raw body;
/// `Decode` carries the raw body that failed to parse.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum QuicknodeError {
    #[error("configuration error: {message}")]
    Config { message: String },
    #[error("HTTP error: {message}")]
    Http { message: String },
    #[error("request timed out: {message}")]
    Timeout { message: String },
    #[error("connection error: {message}")]
    Connection { message: String },
    #[error("API error (status {status}): {body}")]
    Api {
        message: String,
        status: u16,
        body: String,
    },
    #[error("failed to decode response: {message}")]
    Decode { message: String, body: String },
}

impl From<SdkError> for QuicknodeError {
    fn from(e: SdkError) -> Self {
        let message = e.to_string();
        match &e {
            SdkError::Config(_) | SdkError::UrlParse(_) => QuicknodeError::Config { message },
            SdkError::Api { status, body } => QuicknodeError::Api {
                message,
                status: status.as_u16(),
                body: body.clone(),
            },
            SdkError::Decode { body, .. } => QuicknodeError::Decode {
                message,
                body: body.clone(),
            },
            SdkError::Http(_) => match e.http_kind() {
                Some(HttpKind::Timeout) => QuicknodeError::Timeout { message },
                Some(HttpKind::Connect) => QuicknodeError::Connection { message },
                _ => QuicknodeError::Http { message },
            },
        }
    }
}

/// Root Go-facing handle to the SDK. Wraps the core [`QuicknodeSdk`] and hands
/// out per-product sub-clients.
#[derive(uniffi::Object)]
pub struct QuicknodeSdkClient {
    inner: Arc<QuicknodeSdk>,
}

#[uniffi::export]
impl QuicknodeSdkClient {
    /// Construct an SDK client from an API key. The `User-Agent` is attributed
    /// to the Go binding.
    #[uniffi::constructor]
    pub fn new(api_key: String) -> Result<Self, QuicknodeError> {
        Self::build(api_key, BaseUrlOverrides::default())
    }

    /// Construct an SDK client overriding one or more sub-client base URLs.
    /// Useful for testing against a mock server or pointing at a proxy;
    /// production callers use [`Self::new`]. Any field left `None` uses the
    /// default Quicknode endpoint.
    #[uniffi::constructor]
    pub fn new_with_base_urls(
        api_key: String,
        overrides: BaseUrlOverrides,
    ) -> Result<Self, QuicknodeError> {
        Self::build(api_key, overrides)
    }

    /// Admin API sub-client: endpoints, tags, teams, billing, usage, metrics,
    /// security, and rate limits.
    pub fn admin(&self) -> Arc<AdminClient> {
        Arc::new(AdminClient {
            inner: self.inner.admin.clone(),
        })
    }

    /// Streams API sub-client: create and manage blockchain data streams.
    pub fn streams(&self) -> Arc<StreamsClient> {
        Arc::new(StreamsClient {
            inner: self.inner.streams.clone(),
        })
    }

    /// Webhooks API sub-client: create webhooks from templates and manage their
    /// lifecycle.
    pub fn webhooks(&self) -> Arc<WebhooksClient> {
        Arc::new(WebhooksClient {
            inner: self.inner.webhooks.clone(),
        })
    }

    /// Key-Value Store sub-client: manage sets and lists.
    pub fn kvstore(&self) -> Arc<KvStoreClient> {
        Arc::new(KvStoreClient {
            inner: self.inner.kvstore.clone(),
        })
    }

    /// SQL API sub-client: run SQL queries and fetch schemas.
    pub fn sql(&self) -> Arc<SqlClient> {
        Arc::new(SqlClient {
            inner: self.inner.sql.clone(),
        })
    }
}

impl QuicknodeSdkClient {
    fn build(api_key: String, overrides: BaseUrlOverrides) -> Result<Self, QuicknodeError> {
        let config = SdkFullConfig {
            api_key,
            http: None,
            admin: overrides.admin.map(|base_url| AdminConfig {
                base_url: Some(base_url),
            }),
            streams: overrides.streams.map(|base_url| StreamsConfig {
                base_url: Some(base_url),
            }),
            webhooks: overrides.webhooks.map(|base_url| WebhooksConfig {
                base_url: Some(base_url),
            }),
            kvstore: overrides.kvstore.map(|base_url| KvStoreConfig {
                base_url: Some(base_url),
            }),
            sql: overrides.sql.map(|base_url| SqlConfig {
                base_url: Some(base_url),
            }),
        };
        let client_info = ClientInfo {
            language: "go".to_string(),
            language_version: "unknown".to_string(),
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let inner = QuicknodeSdk::new_with_client_info(&config, Some(client_info))?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

/// Per-sub-client base URL overrides for [`QuicknodeSdkClient::new_with_base_urls`].
/// Each `None` field falls back to the default Quicknode endpoint.
#[derive(Debug, Default, uniffi::Record)]
pub struct BaseUrlOverrides {
    #[uniffi(default = None)]
    pub admin: Option<String>,
    #[uniffi(default = None)]
    pub streams: Option<String>,
    #[uniffi(default = None)]
    pub webhooks: Option<String>,
    #[uniffi(default = None)]
    pub kvstore: Option<String>,
    #[uniffi(default = None)]
    pub sql: Option<String>,
}

/// Admin API sub-client.
#[derive(uniffi::Object)]
pub struct AdminClient {
    inner: AdminApiClient,
}

#[uniffi::export]
impl AdminClient {
    /// List endpoints on the account. Supports searching, filtering, sorting,
    /// and pagination.
    pub fn get_endpoints(
        &self,
        params: GetEndpointsRequest,
    ) -> Result<GetEndpointsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_endpoints(&params))
            .map_err(QuicknodeError::from)
    }

    /// Create a new endpoint for a given blockchain and network.
    pub fn create_endpoint(
        &self,
        params: CreateEndpointRequest,
    ) -> Result<CreateEndpointResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_endpoint(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch details for a specific endpoint by ID.
    pub fn show_endpoint(&self, id: String) -> Result<ShowEndpointResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.show_endpoint(&id))
            .map_err(QuicknodeError::from)
    }

    /// Update editable fields on an endpoint (e.g. its label).
    pub fn update_endpoint(
        &self,
        id: String,
        params: UpdateEndpointRequest,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.update_endpoint(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Archive an endpoint.
    pub fn archive_endpoint(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.archive_endpoint(&id))
            .map_err(QuicknodeError::from)
    }

    /// Pause or unpause an endpoint by setting its status.
    pub fn update_endpoint_status(
        &self,
        id: String,
        params: UpdateEndpointStatusRequest,
    ) -> Result<UpdateEndpointStatusResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.update_endpoint_status(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Create a new tag on a specific endpoint from a label.
    pub fn create_tag(&self, id: String, params: CreateTagRequest) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_tag(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a tag from a specific endpoint by tag id.
    pub fn delete_tag(&self, id: String, tag_id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_tag(&id, &tag_id))
            .map_err(QuicknodeError::from)
    }

    /// List all teams on the account.
    pub fn list_teams(&self) -> Result<ListTeamsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_teams())
            .map_err(QuicknodeError::from)
    }

    /// Create a new team.
    pub fn create_team(
        &self,
        params: CreateTeamRequest,
    ) -> Result<CreateTeamResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_team(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch a specific team by id.
    pub fn get_team(&self, id: i64) -> Result<GetTeamResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_team(id))
            .map_err(QuicknodeError::from)
    }

    /// Delete a team by id. The team must have no members.
    pub fn delete_team(&self, id: i64) -> Result<DeleteTeamResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_team(id))
            .map_err(QuicknodeError::from)
    }

    /// List the endpoints accessible to a given team.
    pub fn list_team_endpoints(
        &self,
        id: i64,
    ) -> Result<ListTeamEndpointsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_team_endpoints(id))
            .map_err(QuicknodeError::from)
    }

    /// Assign or unassign endpoints for a team.
    pub fn update_team_endpoints(
        &self,
        id: i64,
        params: UpdateTeamEndpointsRequest,
    ) -> Result<UpdateTeamEndpointsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.update_team_endpoints(id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Invite a user to a team by email.
    pub fn invite_team_member(
        &self,
        id: i64,
        params: InviteTeamMemberRequest,
    ) -> Result<InviteTeamMemberResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.invite_team_member(id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a user from a team by team id and user id.
    pub fn remove_team_member(
        &self,
        id: i64,
        user_id: i64,
        params: RemoveTeamMemberRequest,
    ) -> Result<RemoveTeamMemberResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.remove_team_member(id, user_id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Resend the invitation email to a pending team member.
    pub fn resend_team_invite(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<ResendTeamInviteResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.resend_team_invite(id, user_id))
            .map_err(QuicknodeError::from)
    }

    /// Fetch account RPC usage totals for an optional time range.
    pub fn get_usage(&self, params: GetUsageRequest) -> Result<GetUsageResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_usage(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch RPC usage broken down per endpoint over an optional time range.
    pub fn get_usage_by_endpoint(
        &self,
        params: GetUsageRequest,
    ) -> Result<GetUsageByEndpointResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_usage_by_endpoint(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch RPC usage grouped by method over an optional time range.
    pub fn get_usage_by_method(
        &self,
        params: GetUsageRequest,
    ) -> Result<GetUsageByMethodResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_usage_by_method(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch RPC usage grouped by chain over an optional time range.
    pub fn get_usage_by_chain(
        &self,
        params: GetUsageRequest,
    ) -> Result<GetUsageByChainResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_usage_by_chain(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch activity logs for a specific endpoint.
    pub fn get_endpoint_logs(
        &self,
        id: String,
        params: GetEndpointLogsRequest,
    ) -> Result<GetEndpointLogsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_endpoint_logs(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the raw request/response payloads for a specific log entry.
    pub fn get_log_details(
        &self,
        id: String,
        request_id: String,
    ) -> Result<GetLogDetailsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_log_details(&id, &request_id))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the security options (feature toggles) for an endpoint.
    pub fn get_security_options(
        &self,
        id: String,
    ) -> Result<GetSecurityOptionsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_security_options(&id))
            .map_err(QuicknodeError::from)
    }

    /// Update which security features are enabled on an endpoint.
    pub fn update_security_options(
        &self,
        id: String,
        params: UpdateSecurityOptionsRequest,
    ) -> Result<UpdateSecurityOptionsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.update_security_options(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Generate a new authentication token for an endpoint.
    pub fn create_token(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_token(&id))
            .map_err(QuicknodeError::from)
    }

    /// Revoke a token on an endpoint by token id.
    pub fn delete_token(
        &self,
        id: String,
        token_id: String,
    ) -> Result<DeleteBoolResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_token(&id, &token_id))
            .map_err(QuicknodeError::from)
    }

    /// Add a referrer to an endpoint's security settings.
    pub fn create_referrer(
        &self,
        id: String,
        params: CreateReferrerRequest,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_referrer(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a referrer from an endpoint's security settings by referrer id.
    pub fn delete_referrer(
        &self,
        id: String,
        referrer_id: String,
    ) -> Result<DeleteBoolResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_referrer(&id, &referrer_id))
            .map_err(QuicknodeError::from)
    }

    /// Add an IP address to an endpoint's security whitelist.
    pub fn create_ip(&self, id: String, params: CreateIpRequest) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_ip(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove an IP address from an endpoint's security whitelist by ip id.
    pub fn delete_ip(
        &self,
        id: String,
        ip_id: String,
    ) -> Result<DeleteBoolResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_ip(&id, &ip_id))
            .map_err(QuicknodeError::from)
    }

    /// Add a domain mask to an endpoint.
    pub fn create_domain_mask(
        &self,
        id: String,
        params: CreateDomainMaskRequest,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_domain_mask(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a domain mask from an endpoint by domain mask id.
    pub fn delete_domain_mask(
        &self,
        id: String,
        domain_mask_id: String,
    ) -> Result<DeleteBoolResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_domain_mask(&id, &domain_mask_id))
            .map_err(QuicknodeError::from)
    }

    /// Create a new JWT for endpoint authentication.
    pub fn create_jwt(&self, id: String, params: CreateJwtRequest) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_jwt(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a JWT from an endpoint's security configuration by jwt id.
    pub fn delete_jwt(&self, id: String, jwt_id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_jwt(&id, &jwt_id))
            .map_err(QuicknodeError::from)
    }

    /// Create a request filter (method whitelist) on an endpoint.
    pub fn create_request_filter(
        &self,
        id: String,
        params: CreateRequestFilterRequest,
    ) -> Result<CreateRequestFilterResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_request_filter(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Update an existing request filter on an endpoint.
    pub fn update_request_filter(
        &self,
        id: String,
        request_filter_id: String,
        params: UpdateRequestFilterRequest,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(
                self.inner
                    .update_request_filter(&id, &request_filter_id, &params),
            )
            .map_err(QuicknodeError::from)
    }

    /// Remove a request filter from an endpoint by request filter id.
    pub fn delete_request_filter(
        &self,
        id: String,
        request_filter_id: String,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_request_filter(&id, &request_filter_id))
            .map_err(QuicknodeError::from)
    }

    /// Enable multichain functionality on an endpoint.
    pub fn enable_multichain(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.enable_multichain(&id))
            .map_err(QuicknodeError::from)
    }

    /// Disable multichain functionality on an endpoint.
    pub fn disable_multichain(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.disable_multichain(&id))
            .map_err(QuicknodeError::from)
    }

    /// Set the custom HTTP header used to identify the client IP for an
    /// endpoint.
    pub fn create_or_update_ip_custom_header(
        &self,
        id: String,
        params: CreateOrUpdateIpCustomHeaderRequest,
    ) -> Result<CreateOrUpdateIpCustomHeaderResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_or_update_ip_custom_header(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Remove the custom IP header configuration from an endpoint.
    pub fn delete_ip_custom_header(
        &self,
        id: String,
    ) -> Result<DeleteBoolResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_ip_custom_header(&id))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the method rate limits configured on an endpoint.
    pub fn get_method_rate_limits(
        &self,
        id: String,
    ) -> Result<GetMethodRateLimitsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_method_rate_limits(&id))
            .map_err(QuicknodeError::from)
    }

    /// Create a per-method rate limit on an endpoint.
    pub fn create_method_rate_limit(
        &self,
        id: String,
        params: CreateMethodRateLimitRequest,
    ) -> Result<CreateMethodRateLimitResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_method_rate_limit(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Update an existing method rate limit on an endpoint.
    pub fn update_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
        params: UpdateMethodRateLimitRequest,
    ) -> Result<UpdateMethodRateLimitResponse, QuicknodeError> {
        runtime()
            .block_on(
                self.inner
                    .update_method_rate_limit(&id, &method_rate_limit_id, &params),
            )
            .map_err(QuicknodeError::from)
    }

    /// Remove a method rate limit from an endpoint by method rate limit id.
    pub fn delete_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(
                self.inner
                    .delete_method_rate_limit(&id, &method_rate_limit_id),
            )
            .map_err(QuicknodeError::from)
    }

    /// Partially update the endpoint-level rate-limit overrides.
    pub fn update_rate_limits(
        &self,
        id: String,
        params: UpdateRateLimitsRequest,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.update_rate_limits(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the endpoint-level rate limits currently enforced.
    pub fn get_rate_limits(&self, id: String) -> Result<GetRateLimitsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_rate_limits(&id))
            .map_err(QuicknodeError::from)
    }

    /// Delete a user-set rate-limit override by its UUID.
    pub fn delete_rate_limit_override(
        &self,
        id: String,
        override_id: String,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_rate_limit_override(&id, &override_id))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the HTTP and WebSocket URLs for the endpoint.
    pub fn get_endpoint_urls(&self, id: String) -> Result<GetEndpointUrlsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_endpoint_urls(&id))
            .map_err(QuicknodeError::from)
    }

    /// Fetch time-series metrics for a specific endpoint.
    pub fn get_endpoint_metrics(
        &self,
        id: String,
        params: GetEndpointMetricsRequest,
    ) -> Result<GetEndpointMetricsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_endpoint_metrics(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch aggregated metrics across all endpoints on the account.
    pub fn get_account_metrics(
        &self,
        params: GetAccountMetricsRequest,
    ) -> Result<GetAccountMetricsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_account_metrics(&params))
            .map_err(QuicknodeError::from)
    }

    /// List all chains supported by Quicknode along with their networks.
    pub fn list_chains(&self) -> Result<ListChainsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_chains())
            .map_err(QuicknodeError::from)
    }

    /// List the account's invoices.
    pub fn list_invoices(&self) -> Result<ListInvoicesResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_invoices())
            .map_err(QuicknodeError::from)
    }

    /// List all payments on the account.
    pub fn list_payments(&self) -> Result<ListPaymentsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_payments())
            .map_err(QuicknodeError::from)
    }

    /// Pause or unpause multiple endpoints in a single call.
    pub fn bulk_update_endpoint_status(
        &self,
        params: BulkUpdateEndpointStatusRequest,
    ) -> Result<BulkUpdateEndpointStatusResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.bulk_update_endpoint_status(&params))
            .map_err(QuicknodeError::from)
    }

    /// Apply a single tag label to multiple endpoints in one call.
    pub fn bulk_add_tag(
        &self,
        params: BulkAddTagRequest,
    ) -> Result<BulkAddTagResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.bulk_add_tag(&params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a tag from multiple endpoints in one call.
    pub fn bulk_remove_tag(
        &self,
        params: BulkRemoveTagRequest,
    ) -> Result<BulkRemoveTagResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.bulk_remove_tag(&params))
            .map_err(QuicknodeError::from)
    }

    /// List all account-level tags, including tags with zero endpoints.
    pub fn list_tags(&self) -> Result<ListTagsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_tags())
            .map_err(QuicknodeError::from)
    }

    /// Update the label of an account tag.
    pub fn rename_tag(
        &self,
        id: i32,
        params: RenameTagRequest,
    ) -> Result<RenameTagResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.rename_tag(id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Delete an account-level tag. It must first be removed from all endpoints.
    pub fn delete_account_tag(&self, id: i32) -> Result<DeleteAccountTagResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_account_tag(id))
            .map_err(QuicknodeError::from)
    }

    /// Fetch RPC usage grouped by endpoint tag over an optional time range.
    pub fn get_usage_by_tag(
        &self,
        params: GetUsageRequest,
    ) -> Result<GetUsageByTagResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_usage_by_tag(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the full security configuration for an endpoint in a single call.
    pub fn get_endpoint_security(
        &self,
        id: String,
    ) -> Result<GetEndpointSecurityResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_endpoint_security(&id))
            .map_err(QuicknodeError::from)
    }
}

/// Streams API sub-client.
#[derive(uniffi::Object)]
pub struct StreamsClient {
    inner: StreamsApiClient,
}

#[uniffi::export]
impl StreamsClient {
    /// Create a new stream.
    pub fn create_stream(&self, params: CreateStreamParams) -> Result<Stream, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_stream(&params))
            .map_err(QuicknodeError::from)
    }

    /// List streams on the account.
    pub fn list_streams(
        &self,
        params: ListStreamsParams,
    ) -> Result<ListStreamsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_streams(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch a single stream by id.
    pub fn get_stream(&self, id: String) -> Result<Stream, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_stream(&id))
            .map_err(QuicknodeError::from)
    }

    /// Update a stream. Only set fields are modified.
    pub fn update_stream(
        &self,
        id: String,
        params: UpdateStreamParams,
    ) -> Result<Stream, QuicknodeError> {
        runtime()
            .block_on(self.inner.update_stream(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Delete a stream by id.
    pub fn delete_stream(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_stream(&id))
            .map_err(QuicknodeError::from)
    }

    /// Activate a stream by id.
    pub fn activate_stream(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.activate_stream(&id))
            .map_err(QuicknodeError::from)
    }

    /// Pause a stream by id.
    pub fn pause_stream(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.pause_stream(&id))
            .map_err(QuicknodeError::from)
    }

    /// Delete all streams on the account.
    pub fn delete_all_streams(&self) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_all_streams())
            .map_err(QuicknodeError::from)
    }

    /// Test a filter function against a stream configuration.
    pub fn test_filter(
        &self,
        params: TestFilterParams,
    ) -> Result<TestFilterResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.test_filter(&params))
            .map_err(QuicknodeError::from)
    }

    /// Count currently enabled streams, optionally filtered by type.
    pub fn get_enabled_count(
        &self,
        stream_type: Option<String>,
    ) -> Result<EnabledCountResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_enabled_count(stream_type.as_deref()))
            .map_err(QuicknodeError::from)
    }
}

/// Webhooks API sub-client.
#[derive(uniffi::Object)]
pub struct WebhooksClient {
    inner: WebhooksApiClient,
}

#[uniffi::export]
impl WebhooksClient {
    /// List webhooks on the account with pagination.
    pub fn list_webhooks(
        &self,
        params: GetWebhooksParams,
    ) -> Result<ListWebhooksResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_webhooks(&params))
            .map_err(QuicknodeError::from)
    }

    /// Remove every webhook on the account.
    pub fn delete_all_webhooks(&self) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_all_webhooks())
            .map_err(QuicknodeError::from)
    }

    /// Fetch a single webhook's full configuration and status by ID.
    pub fn get_webhook(&self, id: String) -> Result<Webhook, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_webhook(&id))
            .map_err(QuicknodeError::from)
    }

    /// Modify an existing webhook's configuration.
    pub fn update_webhook(
        &self,
        id: String,
        params: UpdateWebhookParams,
    ) -> Result<Webhook, QuicknodeError> {
        runtime()
            .block_on(self.inner.update_webhook(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Permanently remove a single webhook by ID.
    pub fn delete_webhook(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_webhook(&id))
            .map_err(QuicknodeError::from)
    }

    /// Pause a webhook by ID so it stops delivering events until reactivated.
    pub fn pause_webhook(&self, id: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.pause_webhook(&id))
            .map_err(QuicknodeError::from)
    }

    /// Activate a created or paused webhook so it begins delivering events.
    pub fn activate_webhook(
        &self,
        id: String,
        params: ActivateWebhookParams,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.activate_webhook(&id, &params))
            .map_err(QuicknodeError::from)
    }

    /// Count the enabled webhooks currently configured on the account.
    pub fn get_enabled_count(&self) -> Result<WebhookEnabledCountResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_enabled_count())
            .map_err(QuicknodeError::from)
    }

    /// Create a new webhook from a predefined filter template.
    pub fn create_webhook_from_template(
        &self,
        params: CreateWebhookFromTemplateParams,
    ) -> Result<Webhook, QuicknodeError> {
        runtime()
            .block_on(self.inner.create_webhook_from_template(&params))
            .map_err(QuicknodeError::from)
    }

    /// Update an existing template-backed webhook's template arguments.
    pub fn update_webhook_template(
        &self,
        webhook_id: String,
        params: UpdateWebhookTemplateParams,
    ) -> Result<Webhook, QuicknodeError> {
        runtime()
            .block_on(self.inner.update_webhook_template(&webhook_id, &params))
            .map_err(QuicknodeError::from)
    }
}

/// KvStore API sub-client.
#[derive(uniffi::Object)]
pub struct KvStoreClient {
    inner: KvStoreApiClient,
}

#[uniffi::export]
impl KvStoreClient {
    /// Create a new set, storing a single string value under the given key.
    pub fn create_set(&self, params: CreateSetParams) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_set(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch a paginated page of key/value entries from the store.
    pub fn get_sets(&self, params: GetSetsParams) -> Result<GetSetsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_sets(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the string value stored for a single set by key.
    pub fn get_set(&self, key: String) -> Result<GetSetResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_set(&key))
            .map_err(QuicknodeError::from)
    }

    /// Add and remove multiple sets in a single request.
    pub fn bulk_sets(&self, params: BulkSetsParams) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.bulk_sets(&params))
            .map_err(QuicknodeError::from)
    }

    /// Remove a single set by key.
    pub fn delete_set(&self, key: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_set(&key))
            .map_err(QuicknodeError::from)
    }

    /// Create a new list under the given key, seeded with the provided items.
    pub fn create_list(&self, params: CreateListParams) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.create_list(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch a paginated page of list keys from the store.
    pub fn get_lists(&self, params: GetListsParams) -> Result<GetListsResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_lists(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch a paginated page of items from the list identified by `key`.
    pub fn get_list(
        &self,
        key: String,
        params: GetListParams,
    ) -> Result<GetListResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_list(&key, &params))
            .map_err(QuicknodeError::from)
    }

    /// Update an existing list by adding and/or removing items.
    pub fn update_list(&self, key: String, params: UpdateListParams) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.update_list(&key, &params))
            .map_err(QuicknodeError::from)
    }

    /// Append a single item to the list identified by `key`.
    pub fn add_list_item(
        &self,
        key: String,
        params: AddListItemParams,
    ) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.add_list_item(&key, &params))
            .map_err(QuicknodeError::from)
    }

    /// Check whether the specified list contains the given item.
    pub fn list_contains_item(
        &self,
        key: String,
        item: String,
    ) -> Result<ListContainsItemResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.list_contains_item(&key, &item))
            .map_err(QuicknodeError::from)
    }

    /// Remove a specific item from the list identified by `key`.
    pub fn delete_list_item(&self, key: String, item: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_list_item(&key, &item))
            .map_err(QuicknodeError::from)
    }

    /// Remove a list and all of its items by key.
    pub fn delete_list(&self, key: String) -> Result<(), QuicknodeError> {
        runtime()
            .block_on(self.inner.delete_list(&key))
            .map_err(QuicknodeError::from)
    }
}

/// SQL API sub-client.
#[derive(uniffi::Object)]
pub struct SqlClient {
    inner: SqlApiClient,
}

#[uniffi::export]
impl SqlClient {
    /// Execute a SQL query against the given cluster and return the result set.
    pub fn query(&self, params: QueryParams) -> Result<QueryResponse, QuicknodeError> {
        runtime()
            .block_on(self.inner.query(&params))
            .map_err(QuicknodeError::from)
    }

    /// Fetch the database schema for a cluster.
    pub fn get_schema(&self, cluster_id: String) -> Result<ChainSchema, QuicknodeError> {
        runtime()
            .block_on(self.inner.get_schema(&cluster_id))
            .map_err(QuicknodeError::from)
    }
}
