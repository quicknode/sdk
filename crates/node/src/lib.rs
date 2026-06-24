use napi::bindgen_prelude::*;
use napi_derive::napi;
use quicknode_sdk as core;

mod errors;
mod key_case;
mod sql;
mod streams_destination;
mod webhooks_template;

// ── Top-level SDK ──────────────────────────────────────────────

#[napi]
pub struct QuicknodeSdk {
    admin: AdminApiClient,
    streams: StreamsApiClient,
    webhooks: WebhooksApiClient,
    kvstore: KvStoreApiClient,
    sql: SqlApiClient,
    rpc: RpcApiClient,
}

/// Build a [`core::ClientInfo`] from the live Node.js runtime so the SDK's
/// `User-Agent` reflects the installed `@quicknode/sdk` npm package and the
/// running Node.js interpreter. Failures fall back to `"unknown"` rather
/// than aborting the SDK constructor.
fn node_client_info(env: &Env) -> core::ClientInfo {
    let language_version = (|| -> Result<String> {
        let process: Object = env.get_global()?.get_named_property("process")?;
        let s: String = process.get_named_property("version")?;
        // Node's process.version is e.g. "v20.10.0" — strip the leading "v".
        Ok(s.strip_prefix('v').unwrap_or(&s).to_string())
    })()
    .unwrap_or_else(|_| "unknown".to_string());

    core::ClientInfo {
        language: "node".to_string(),
        language_version,
        sdk_version: env!("NPM_PACKAGE_VERSION").to_string(),
    }
}

#[napi]
impl QuicknodeSdk {
    /// Creates a new SDK instance from an explicit configuration.
    #[napi(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(env: Env, config: core::SdkFullConfig) -> Result<Self> {
        let sdk_config =
            core::SdkConfig::new_with_client_info(&config, Some(node_client_info(&env)))
                .map_err(errors::map_sdk_err)?;
        Ok(Self {
            admin: AdminApiClient {
                inner: core::admin::AdminApiClient::new(sdk_config.clone()),
            },
            webhooks: WebhooksApiClient {
                inner: core::webhooks::WebhooksApiClient::new(sdk_config.clone()),
            },
            streams: StreamsApiClient {
                inner: core::streams::StreamsApiClient::new(sdk_config.clone()),
            },
            kvstore: KvStoreApiClient {
                inner: core::kvstore::KvStoreApiClient::new(sdk_config.clone()),
            },
            sql: SqlApiClient {
                inner: core::sql::SqlApiClient::new(sdk_config.clone()),
            },
            rpc: RpcApiClient {
                inner: core::rpc::RpcApiClient::new(sdk_config, config.rpc.as_ref()),
            },
        })
    }

    /// Returns the admin sub-client.
    #[napi(getter)]
    pub fn admin(&self) -> AdminApiClient {
        self.admin.clone()
    }

    /// Returns the streams sub-client.
    #[napi(getter)]
    pub fn streams(&self) -> StreamsApiClient {
        self.streams.clone()
    }

    /// Returns the webhooks sub-client.
    #[napi(getter)]
    pub fn webhooks(&self) -> WebhooksApiClient {
        self.webhooks.clone()
    }

    /// Returns the kvstore sub-client.
    #[napi(getter)]
    pub fn kvstore(&self) -> KvStoreApiClient {
        self.kvstore.clone()
    }

    /// Returns the sql sub-client.
    #[napi(getter)]
    pub fn sql(&self) -> SqlApiClient {
        self.sql.clone()
    }

    /// Returns the JSON-RPC sub-client.
    #[napi(getter)]
    pub fn rpc(&self) -> RpcApiClient {
        self.rpc.clone()
    }

    /// Creates a new SDK instance using configuration from environment variables.
    #[napi(factory)]
    pub fn from_env() -> Result<Self> {
        core::QuicknodeSdk::from_env()
            .map(|sdk| Self {
                admin: AdminApiClient { inner: sdk.admin },
                streams: StreamsApiClient { inner: sdk.streams },
                webhooks: WebhooksApiClient {
                    inner: sdk.webhooks,
                },
                kvstore: KvStoreApiClient { inner: sdk.kvstore },
                sql: SqlApiClient { inner: sdk.sql },
                rpc: RpcApiClient { inner: sdk.rpc },
            })
            .map_err(errors::map_sdk_err)
    }
}

// ── Sub-clients ───────────────────────────────────────

#[derive(Clone)]
#[napi]
pub struct AdminApiClient {
    inner: core::admin::AdminApiClient,
}

#[napi]
impl AdminApiClient {
    /// Returns a paginated list of endpoints on the account. Supports searching
    /// by subdomain or label, filtering by networks, statuses, labels, and
    /// tags, and sorting. The response includes endpoint metadata (id, label,
    /// status, chain/network, HTTP and WebSocket URLs, tags) plus
    /// total/limit/offset pagination info.
    #[napi]
    pub async fn get_endpoints(
        &self,
        params: Option<core::admin::GetEndpointsRequest>,
    ) -> Result<core::admin::GetEndpointsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_endpoints(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a new endpoint for a given blockchain and network. Requires
    /// `chain` and `network`; returns the new endpoint with its HTTP and
    /// WebSocket URLs, default security configuration (tokens, JWTs, IPs,
    /// domain masks, CORS), and rate limits.
    #[napi]
    pub async fn create_endpoint(
        &self,
        params: Option<core::admin::CreateEndpointRequest>,
    ) -> Result<core::admin::CreateEndpointResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .create_endpoint(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns details for a specific endpoint by ID.
    #[napi]
    pub async fn show_endpoint(&self, id: String) -> Result<core::admin::ShowEndpointResponse> {
        self.inner
            .show_endpoint(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Updates editable fields on an endpoint (e.g. its label). Returns a
    /// boolean indicating whether the update succeeded.
    #[napi]
    pub async fn update_endpoint(
        &self,
        id: String,
        params: Option<core::admin::UpdateEndpointRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .update_endpoint(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Archives an endpoint. The API uses `DELETE` but the effect is archival
    /// rather than permanent deletion.
    #[napi]
    pub async fn archive_endpoint(&self, id: String) -> Result<()> {
        self.inner
            .archive_endpoint(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Pauses or unpauses an endpoint by setting its status to `active` or
    /// `paused`.
    #[napi]
    pub async fn update_endpoint_status(
        &self,
        id: String,
        params: core::admin::UpdateEndpointStatusRequest,
    ) -> Result<core::admin::UpdateEndpointStatusResponse> {
        self.inner
            .update_endpoint_status(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a new tag on a specific endpoint from a label. Returns the new
    /// tag with its id, account info, and timestamps.
    #[napi]
    pub async fn create_tag(
        &self,
        id: String,
        params: Option<core::admin::CreateTagRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .create_tag(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a tag from a specific endpoint by tag id.
    #[napi]
    pub async fn delete_tag(&self, id: String, tag_id: String) -> Result<()> {
        self.inner
            .delete_tag(&id, &tag_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns account RPC usage totals for an optional time range. The
    /// response includes `credits_used`, `credits_remaining`, the account
    /// `limit`, any `overages`, and the queried time window.
    #[napi]
    pub async fn get_usage(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns RPC usage broken down per endpoint over an optional time range.
    /// Each entry includes endpoint metadata, aggregate `credits_used` and
    /// `requests`, and a per-method credit breakdown.
    #[napi]
    pub async fn get_usage_by_endpoint(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByEndpointResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_endpoint(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns RPC usage grouped by method over an optional time range. Each
    /// entry includes the method name, credits consumed, and archival status.
    /// Ranges longer than one week are rounded to midnight UTC.
    #[napi]
    pub async fn get_usage_by_method(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByMethodResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_method(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns RPC usage grouped by chain over an optional time range. Each
    /// entry includes the chain and its credit consumption.
    #[napi]
    pub async fn get_usage_by_chain(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByChainResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_chain(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns activity logs for a specific endpoint. Supports filtering by
    /// timestamp range and pagination. Each log entry includes timestamp,
    /// HTTP method, network, status code, and error data; full request/response
    /// bodies can be included when requested.
    #[napi]
    pub async fn get_endpoint_logs(
        &self,
        id: String,
        params: core::admin::GetEndpointLogsRequest,
    ) -> Result<core::admin::GetEndpointLogsResponse> {
        self.inner
            .get_endpoint_logs(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the raw request and response payloads for a specific log entry
    /// on an endpoint, identified by request UUID. Both payloads are
    /// JSON-encoded strings and are truncated at 2KB.
    #[napi]
    pub async fn get_log_details(
        &self,
        id: String,
        request_id: String,
    ) -> Result<core::admin::GetLogDetailsResponse> {
        self.inner
            .get_log_details(&id, &request_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the security options for an endpoint — an object of security
    /// feature toggles with their current enabled/disabled status.
    #[napi]
    pub async fn get_security_options(
        &self,
        id: String,
    ) -> Result<core::admin::GetSecurityOptionsResponse> {
        self.inner
            .get_security_options(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Updates which security features are enabled on an endpoint. Each option
    /// in the submitted object can be toggled `enabled` or `disabled` —
    /// examples include token auth, JWT validation, IP restrictions, CORS,
    /// HSTS, referrer validation, and domain masking.
    #[napi]
    pub async fn update_security_options(
        &self,
        id: String,
        params: core::admin::UpdateSecurityOptionsRequest,
    ) -> Result<core::admin::UpdateSecurityOptionsResponse> {
        self.inner
            .update_security_options(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Generates a new authentication token for an endpoint.
    #[napi]
    pub async fn create_token(&self, id: String) -> Result<()> {
        self.inner
            .create_token(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Revokes a token on an endpoint by token id.
    #[napi]
    pub async fn delete_token(
        &self,
        id: String,
        token_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_token(&id, &token_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Adds a referrer to an endpoint's security settings, specifying which
    /// external URL or domain is permitted to call the endpoint.
    #[napi]
    pub async fn create_referrer(
        &self,
        id: String,
        params: Option<core::admin::CreateReferrerRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .create_referrer(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a referrer from an endpoint's security settings by referrer id.
    #[napi]
    pub async fn delete_referrer(
        &self,
        id: String,
        referrer_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_referrer(&id, &referrer_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Adds an IP address to an endpoint's security whitelist.
    #[napi]
    pub async fn create_ip(
        &self,
        id: String,
        params: Option<core::admin::CreateIpRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .create_ip(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes an IP address from an endpoint's security whitelist by ip id.
    #[napi]
    pub async fn delete_ip(
        &self,
        id: String,
        ip_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_ip(&id, &ip_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Adds a domain mask to an endpoint — a custom domain used to hide the
    /// endpoint's Quicknode URL so requests can be routed through your own
    /// domain.
    #[napi]
    pub async fn create_domain_mask(
        &self,
        id: String,
        params: Option<core::admin::CreateDomainMaskRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .create_domain_mask(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a domain mask from an endpoint by domain mask id.
    #[napi]
    pub async fn delete_domain_mask(
        &self,
        id: String,
        domain_mask_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_domain_mask(&id, &domain_mask_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a new JWT for endpoint authentication. Accepts a public key,
    /// key id (`kid`), and token name.
    #[napi]
    pub async fn create_jwt(
        &self,
        id: String,
        params: Option<core::admin::CreateJwtRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .create_jwt(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a JWT from an endpoint's security configuration by jwt id,
    /// revoking its access.
    #[napi]
    pub async fn delete_jwt(&self, id: String, jwt_id: String) -> Result<()> {
        self.inner
            .delete_jwt(&id, &jwt_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a request filter on an endpoint — a method whitelist that
    /// restricts which RPC methods may be called. Accepts an array of method
    /// names; other methods are blocked.
    #[napi]
    pub async fn create_request_filter(
        &self,
        id: String,
        params: Option<core::admin::CreateRequestFilterRequest>,
    ) -> Result<core::admin::CreateRequestFilterResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .create_request_filter(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Updates an existing request filter on an endpoint, replacing the
    /// whitelisted method list.
    #[napi]
    pub async fn update_request_filter(
        &self,
        id: String,
        request_filter_id: String,
        params: Option<core::admin::UpdateRequestFilterRequest>,
    ) -> Result<()> {
        let params = params.unwrap_or_default();
        self.inner
            .update_request_filter(&id, &request_filter_id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a request filter from an endpoint's security configuration by
    /// request filter id.
    #[napi]
    pub async fn delete_request_filter(&self, id: String, request_filter_id: String) -> Result<()> {
        self.inner
            .delete_request_filter(&id, &request_filter_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Enables multichain functionality on an endpoint, allowing a single
    /// endpoint to serve multiple chains.
    #[napi]
    pub async fn enable_multichain(&self, id: String) -> Result<()> {
        self.inner
            .enable_multichain(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Disables multichain functionality on an endpoint.
    #[napi]
    pub async fn disable_multichain(&self, id: String) -> Result<()> {
        self.inner
            .disable_multichain(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Sets the custom HTTP header used to identify the client IP for an
    /// endpoint (for example, `X-Forwarded-For`). This header is used by
    /// IP-based security features to resolve the real client address when
    /// requests are proxied.
    #[napi]
    pub async fn create_or_update_ip_custom_header(
        &self,
        id: String,
        params: core::admin::CreateOrUpdateIpCustomHeaderRequest,
    ) -> Result<core::admin::CreateOrUpdateIpCustomHeaderResponse> {
        self.inner
            .create_or_update_ip_custom_header(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes the custom IP header configuration from an endpoint.
    #[napi]
    pub async fn delete_ip_custom_header(
        &self,
        id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_ip_custom_header(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the method rate limits configured on an endpoint, including
    /// each limiter's interval, methods, rate, and status.
    #[napi]
    pub async fn get_method_rate_limits(
        &self,
        id: String,
    ) -> Result<core::admin::GetMethodRateLimitsResponse> {
        self.inner
            .get_method_rate_limits(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a per-method rate limit on an endpoint. A method rate limit
    /// caps specific RPC methods rather than the endpoint as a whole, defined
    /// by an `interval` (e.g. `second`), the target `methods`, and a `rate`.
    #[napi]
    pub async fn create_method_rate_limit(
        &self,
        id: String,
        params: core::admin::CreateMethodRateLimitRequest,
    ) -> Result<core::admin::CreateMethodRateLimitResponse> {
        self.inner
            .create_method_rate_limit(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Updates an existing method rate limit on an endpoint. Accepts the
    /// methods to apply the limit to, the desired `status`, and the `rate`.
    #[napi]
    pub async fn update_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
        params: Option<core::admin::UpdateMethodRateLimitRequest>,
    ) -> Result<core::admin::UpdateMethodRateLimitResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .update_method_rate_limit(&id, &method_rate_limit_id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a method rate limit from an endpoint by method rate limit id.
    #[napi]
    pub async fn delete_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
    ) -> Result<()> {
        self.inner
            .delete_method_rate_limit(&id, &method_rate_limit_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Partial update of the endpoint-level rate-limit overrides. Accepts
    /// `rps` (requests per second), `rpm` (requests per minute), and `rpd`
    /// (requests per day). Only buckets included are modified — omitted
    /// buckets are left unchanged. Values are capped by the account's plan
    /// tier.
    #[napi]
    pub async fn update_rate_limits(
        &self,
        id: String,
        params: core::admin::UpdateRateLimitsRequest,
    ) -> Result<()> {
        self.inner
            .update_rate_limits(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the endpoint-level rate limits currently enforced, with each
    /// row identifying its bucket (`rps`/`rpm`/`rpd`), value, and source
    /// (`plan_default` or `user_override`). User-set overrides expose an
    /// `overrideId` that can be passed to `deleteRateLimitOverride`.
    #[napi]
    pub async fn get_rate_limits(&self, id: String) -> Result<core::admin::GetRateLimitsResponse> {
        self.inner
            .get_rate_limits(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Deletes a user-set rate-limit override by its UUID. Plan defaults are
    /// not deletable.
    #[napi]
    pub async fn delete_rate_limit_override(&self, id: String, override_id: String) -> Result<()> {
        self.inner
            .delete_rate_limit_override(&id, &override_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the HTTP and WebSocket URLs for the endpoint without fetching
    /// the full endpoint record. For multichain endpoints, `multichainUrls`
    /// is a per-network mapping of additional URLs; for single-chain endpoints
    /// it is `null`.
    #[napi]
    pub async fn get_endpoint_urls(
        &self,
        id: String,
    ) -> Result<core::admin::GetEndpointUrlsResponse> {
        self.inner
            .get_endpoint_urls(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns time-series metrics for a specific endpoint. Requires a
    /// `period` (`hour`, `day`, `week`, or `month`) and a metric type such as
    /// `method_calls_over_time` or `response_status_breakdown`.
    #[napi]
    pub async fn get_endpoint_metrics(
        &self,
        id: String,
        params: core::admin::GetEndpointMetricsRequest,
    ) -> Result<core::admin::GetEndpointMetricsResponse> {
        self.inner
            .get_endpoint_metrics(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns aggregated metrics across all endpoints on the account. Accepts
    /// a `period` (`hour`, `day`, `week`, or `month`) and a metric type such
    /// as `method_calls_over_time` or `credits_over_time`.
    #[napi]
    pub async fn get_account_metrics(
        &self,
        params: core::admin::GetAccountMetricsRequest,
    ) -> Result<core::admin::GetAccountMetricsResponse> {
        self.inner
            .get_account_metrics(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns all chains supported by Quicknode along with their networks.
    /// Each entry includes the chain slug and its network slugs and names.
    #[napi]
    pub async fn list_chains(&self) -> Result<core::admin::ListChainsResponse> {
        self.inner.list_chains().await.map_err(errors::map_sdk_err)
    }

    /// Returns details about the account, including its id, name, creation
    /// timestamp, billing version, and current subscription.
    #[napi]
    pub async fn account_info(&self) -> Result<core::admin::AccountInfoResponse> {
        self.inner.account_info().await.map_err(errors::map_sdk_err)
    }

    /// Returns the per-method API credit costs for a chain, identified by its
    /// slug (the same slugs returned by `list_chains`, e.g. `ethereum`). Each
    /// item carries the RPC `method` name and its `credits` cost. An unknown
    /// chain slug rejects with `ApiError` (status 404).
    #[napi]
    pub async fn get_api_credits(
        &self,
        chain: String,
    ) -> Result<core::admin::GetApiCreditsResponse> {
        self.inner
            .get_api_credits(&chain)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the current Tooling Access status for the account. Inspect
    /// `enabled` to decide whether to enable provisioning.
    #[napi]
    pub async fn tooling_access_status(&self) -> Result<core::admin::ToolingAccessStatus> {
        self.inner
            .tooling_access_status()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Enables (provisions) Tooling Access. Idempotent. Requires an admin role
    /// and an eligible plan.
    #[napi]
    pub async fn enable_tooling_access(&self) -> Result<core::admin::ToolingAccessStatus> {
        self.inner
            .enable_tooling_access()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Disables Tooling Access, pausing the endpoint. Idempotent.
    #[napi]
    pub async fn disable_tooling_access(&self) -> Result<core::admin::ToolingAccessStatus> {
        self.inner
            .disable_tooling_access()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Mints a short-lived session JWT for the provisioned Tooling Access
    /// endpoint. Returns the endpoint URL, the JWT, and its expiry. Requires
    /// Tooling Access to be enabled first.
    #[napi]
    pub async fn mint_tooling_token(&self) -> Result<core::CachedToken> {
        self.inner
            .mint_tooling_token()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the account's invoices, including id, status, billing reason,
    /// amounts due and paid, line items with descriptions and billing periods,
    /// and creation timestamps.
    #[napi]
    pub async fn list_invoices(&self) -> Result<core::admin::ListInvoicesResponse> {
        self.inner
            .list_invoices()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns all payments on the account, including amount, status, card
    /// last-four, timestamp, currency, and marketplace spending.
    #[napi]
    pub async fn list_payments(&self) -> Result<core::admin::ListPaymentsResponse> {
        self.inner
            .list_payments()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns all teams on the account. Each team includes its id, name,
    /// member count, and member details (roles, contact info, account status).
    #[napi]
    pub async fn list_teams(&self) -> Result<core::admin::ListTeamsResponse> {
        self.inner.list_teams().await.map_err(errors::map_sdk_err)
    }

    /// Creates a new team. Requires a `name`; returns the new team with its
    /// id, name, default role, and member count.
    #[napi]
    pub async fn create_team(
        &self,
        params: core::admin::CreateTeamRequest,
    ) -> Result<core::admin::CreateTeamResponse> {
        self.inner
            .create_team(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns a specific team by id, including active members with their
    /// roles and contact info plus any pending invites.
    #[napi]
    pub async fn get_team(&self, id: i64) -> Result<core::admin::GetTeamResponse> {
        self.inner.get_team(id).await.map_err(errors::map_sdk_err)
    }

    /// Deletes a team by id. The team must have no members before it can be
    /// deleted.
    #[napi]
    pub async fn delete_team(&self, id: i64) -> Result<core::admin::DeleteTeamResponse> {
        self.inner
            .delete_team(id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the endpoints accessible to a given team. Each entry includes
    /// the endpoint id, subdomain, chain, and network.
    #[napi]
    pub async fn list_team_endpoints(
        &self,
        id: i64,
    ) -> Result<core::admin::ListTeamEndpointsResponse> {
        self.inner
            .list_team_endpoints(id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Assigns or unassigns endpoints for a team. Pass an array of endpoint ids
    /// to set the team's accessible endpoints; pass an empty array to remove
    /// all associations.
    #[napi]
    pub async fn update_team_endpoints(
        &self,
        id: i64,
        params: core::admin::UpdateTeamEndpointsRequest,
    ) -> Result<core::admin::UpdateTeamEndpointsResponse> {
        self.inner
            .update_team_endpoints(id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Invites a user to a team by email. For new users, `full_name` and
    /// `role` (`admin`, `viewer`, or `billing`) are also required. Returns the
    /// invited user's profile and invitation status.
    #[napi]
    pub async fn invite_team_member(
        &self,
        id: i64,
        params: core::admin::InviteTeamMemberRequest,
    ) -> Result<core::admin::InviteTeamMemberResponse> {
        self.inner
            .invite_team_member(id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a user from a team by team id and user id.
    #[napi]
    pub async fn remove_team_member(
        &self,
        id: i64,
        user_id: i64,
        params: Option<core::admin::RemoveTeamMemberRequest>,
    ) -> Result<core::admin::RemoveTeamMemberResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .remove_team_member(id, user_id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Resends the invitation email to a pending team member, identified by
    /// team id and user id.
    #[napi]
    pub async fn resend_team_invite(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<core::admin::ResendTeamInviteResponse> {
        self.inner
            .resend_team_invite(id, user_id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Pauses or unpauses multiple endpoints in a single call. Accepts an
    /// array of endpoint ids and a target status (`active` or `paused`);
    /// returns per-endpoint success/failure results plus totals.
    #[napi]
    pub async fn bulk_update_endpoint_status(
        &self,
        params: core::admin::BulkUpdateEndpointStatusRequest,
    ) -> Result<core::admin::BulkUpdateEndpointStatusResponse> {
        self.inner
            .bulk_update_endpoint_status(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Applies a single tag label to multiple endpoints in one call. Returns
    /// totals for affected endpoints, successes, and failures, plus the tag
    /// that was applied.
    #[napi]
    pub async fn bulk_add_tag(
        &self,
        params: core::admin::BulkAddTagRequest,
    ) -> Result<core::admin::BulkAddTagResponse> {
        self.inner
            .bulk_add_tag(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a tag from multiple endpoints in one call, identified by an
    /// array of endpoint ids and a tag id.
    #[napi]
    pub async fn bulk_remove_tag(
        &self,
        params: core::admin::BulkRemoveTagRequest,
    ) -> Result<core::admin::BulkRemoveTagResponse> {
        self.inner
            .bulk_remove_tag(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns all account-level tags, including tags with zero associated
    /// endpoints. Each tag includes its id, label, and endpoint usage count.
    #[napi]
    pub async fn list_tags(&self) -> Result<core::admin::ListTagsResponse> {
        self.inner.list_tags().await.map_err(errors::map_sdk_err)
    }

    /// Updates the label of an account tag. Because the tag is shared across
    /// endpoints, all associated endpoints reflect the new label immediately.
    #[napi]
    pub async fn rename_tag(
        &self,
        id: i32,
        params: core::admin::RenameTagRequest,
    ) -> Result<core::admin::RenameTagResponse> {
        self.inner
            .rename_tag(id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Deletes an account-level tag. The tag must first be removed from all
    /// endpoints before it can be deleted.
    #[napi]
    pub async fn delete_account_tag(
        &self,
        id: i32,
    ) -> Result<core::admin::DeleteAccountTagResponse> {
        self.inner
            .delete_account_tag(id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns RPC usage grouped by endpoint tag over an optional time range.
    /// Each entry includes the tag id, label, credits consumed, and request
    /// count.
    #[napi]
    pub async fn get_usage_by_tag(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByTagResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_tag(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the full security configuration for an endpoint in a single
    /// call, without loading the entire endpoint object. The response includes
    /// tokens, JWTs, referrers, domain masks, IPs, and a security options
    /// object describing which features are enabled.
    #[napi]
    pub async fn get_endpoint_security(
        &self,
        id: String,
    ) -> Result<core::admin::GetEndpointSecurityResponse> {
        self.inner
            .get_endpoint_security(&id)
            .await
            .map_err(errors::map_sdk_err)
    }
}

// ── StreamsApiClient ───────────────────────────────────────────────────────

#[derive(Clone)]
#[napi]
pub struct StreamsApiClient {
    inner: core::streams::StreamsApiClient,
}

#[napi]
impl StreamsApiClient {
    /// Creates a new Stream on a given blockchain network and dataset, delivering
    /// batches to the configured destination. Start from a specific block for
    /// backfills or from the tip for real-time streaming, and optionally attach
    /// a base64-encoded JavaScript filter to transform data before delivery.
    /// The stream can be created in an active or paused state and supports
    /// reorg handling, distance-from-tip, elastic batching, notification emails,
    /// and extra destinations for multi-destination delivery.
    #[napi]
    pub async fn create_stream(
        &self,
        params: streams_destination::CreateStreamParamsNode,
    ) -> Result<streams_destination::StreamNode> {
        let core_params = params.into_core()?;
        let stream = self
            .inner
            .create_stream(&core_params)
            .await
            .map_err(errors::map_sdk_err)?;
        streams_destination::StreamNode::from_core(stream)
    }

    /// Returns a paginated list of streams on the account. Each stream includes
    /// its full configuration — identifiers, timestamps, network and dataset,
    /// filter, block range, destination settings, and operational status — and
    /// surfaces advanced features such as elastic batching and extra
    /// destinations, where batches must be delivered to every configured
    /// destination before the stream advances. Supports pagination via
    /// `offset`/`limit` and sorting via `order_by`/`order_direction`, and can
    /// filter by stream type.
    #[napi]
    pub async fn list_streams(
        &self,
        params: Option<core::streams::ListStreamsParams>,
    ) -> Result<streams_destination::ListStreamsResponseNode> {
        let params = params.unwrap_or_default();
        let resp = self
            .inner
            .list_streams(&params)
            .await
            .map_err(errors::map_sdk_err)?;
        streams_destination::ListStreamsResponseNode::from_core(resp)
    }

    /// Removes every stream on the account. Takes no filters and cannot be
    /// undone.
    #[napi]
    pub async fn delete_all_streams(&self) -> Result<()> {
        self.inner
            .delete_all_streams()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns a single stream by ID, including its full configuration and
    /// current status.
    #[napi]
    pub async fn get_stream(&self, id: String) -> Result<streams_destination::StreamNode> {
        let stream = self
            .inner
            .get_stream(&id)
            .await
            .map_err(errors::map_sdk_err)?;
        streams_destination::StreamNode::from_core(stream)
    }

    /// Updates an existing stream's configuration. Only fields present on
    /// `params` are modified; omitted fields are left unchanged.
    #[napi]
    pub async fn update_stream(
        &self,
        id: String,
        params: streams_destination::UpdateStreamParamsNode,
    ) -> Result<streams_destination::StreamNode> {
        let core_params = params.into_core()?;
        let stream = self
            .inner
            .update_stream(&id, &core_params)
            .await
            .map_err(errors::map_sdk_err)?;
        streams_destination::StreamNode::from_core(stream)
    }

    /// Deletes a single stream by ID.
    #[napi]
    pub async fn delete_stream(&self, id: String) -> Result<()> {
        self.inner
            .delete_stream(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Activates a stream by ID, resuming delivery from its current position.
    #[napi]
    pub async fn activate_stream(&self, id: String) -> Result<()> {
        self.inner
            .activate_stream(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Pauses a stream by ID, halting delivery until it is activated again.
    #[napi]
    pub async fn pause_stream(&self, id: String) -> Result<()> {
        self.inner
            .pause_stream(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Runs a filter function against a specified block on a given network and
    /// dataset, returning the filter's output so it can be validated before
    /// being attached to a live stream.
    #[napi]
    pub async fn test_filter(
        &self,
        params: core::streams::TestFilterParams,
    ) -> Result<core::streams::TestFilterResponse> {
        self.inner
            .test_filter(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the total count of currently enabled (active) streams on the
    /// account, optionally filtered by stream type.
    #[napi]
    pub async fn get_enabled_count(
        &self,
        stream_type: Option<String>,
    ) -> Result<core::streams::EnabledCountResponse> {
        self.inner
            .get_enabled_count(stream_type.as_deref())
            .await
            .map_err(errors::map_sdk_err)
    }
}

// ── WebhooksApiClient ───────────────────────────────────────────────────────

#[derive(Clone)]
#[napi]
pub struct WebhooksApiClient {
    inner: core::webhooks::WebhooksApiClient,
}

#[napi]
impl WebhooksApiClient {
    /// Returns a paginated list of webhooks on the account. Each entry includes
    /// the webhook's identifier, creation timestamp, name, network, notification
    /// email, destination configuration (URL, security token, compression),
    /// current status, and any associated template. The response also includes
    /// a `pageInfo` object with the applied limit, offset, and total count.
    #[napi]
    pub async fn list_webhooks(
        &self,
        params: Option<core::webhooks::GetWebhooksParams>,
    ) -> Result<core::webhooks::ListWebhooksResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .list_webhooks(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes every webhook on the account. Destructive and takes no
    /// parameters.
    #[napi]
    pub async fn delete_all_webhooks(&self) -> Result<()> {
        self.inner
            .delete_all_webhooks()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Fetches a single webhook's full configuration and status by ID. Returns
    /// creation timestamp, name, network, notification email, destination
    /// configuration (URL, security token, compression), the sequence number
    /// of the last successfully delivered block, the current status, and the
    /// associated template with its arguments.
    #[napi]
    pub async fn get_webhook(&self, id: String) -> Result<core::webhooks::Webhook> {
        self.inner
            .get_webhook(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Modifies an existing webhook's configuration. Supports updating the
    /// webhook's name, notification email, and destination attributes (URL,
    /// security token, and compression — `none` or `gzip`). All fields are
    /// optional, so partial updates are supported; if the security token is
    /// omitted on update, one is generated automatically. Returns the
    /// webhook's full updated configuration.
    #[napi]
    pub async fn update_webhook(
        &self,
        id: String,
        params: Option<core::webhooks::UpdateWebhookParams>,
    ) -> Result<core::webhooks::Webhook> {
        let params = params.unwrap_or_default();
        self.inner
            .update_webhook(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Permanently removes a single webhook by ID.
    #[napi]
    pub async fn delete_webhook(&self, id: String) -> Result<()> {
        self.inner
            .delete_webhook(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Pauses a webhook by ID so it stops delivering events until reactivated.
    #[napi]
    pub async fn pause_webhook(&self, id: String) -> Result<()> {
        self.inner
            .pause_webhook(&id)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Activates a previously created or paused webhook so it begins (or
    /// resumes) delivering events. `start_from` determines where processing
    /// resumes: `Latest` begins from the newest available block; other values
    /// replay from an earlier point.
    #[napi]
    pub async fn activate_webhook(
        &self,
        id: String,
        params: core::webhooks::ActivateWebhookParams,
    ) -> Result<()> {
        self.inner
            .activate_webhook(&id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the total number of enabled webhooks currently configured on
    /// the account.
    #[napi]
    pub async fn get_enabled_count(&self) -> Result<core::webhooks::WebhookEnabledCountResponse> {
        self.inner
            .get_enabled_count()
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a new webhook from a predefined filter template. Requires a
    /// descriptive name, a target blockchain network, and destination
    /// attributes (URL, compression — `gzip` or `none`, and an optional
    /// security token — auto-generated when omitted). `template_args` carries
    /// template-specific configuration such as wallet addresses or contract
    /// filters. An optional `notification_email` receives alerts if the
    /// webhook terminates.
    #[napi]
    pub async fn create_webhook_from_template(
        &self,
        params: webhooks_template::CreateWebhookFromTemplateParamsNode,
    ) -> Result<core::webhooks::Webhook> {
        let params = params.into_core()?;
        self.inner
            .create_webhook_from_template(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Updates an existing template-backed webhook, modifying its template
    /// arguments and optionally its name, notification email, and destination
    /// attributes (URL, security token, compression — `none` or `gzip`).
    /// All optional fields support partial updates; a security token is
    /// generated automatically if not provided. Templates cover EVM chains,
    /// Solana, Bitcoin, XRPL, Hyperliquid, and Stellar.
    #[napi]
    pub async fn update_webhook_template(
        &self,
        webhook_id: String,
        params: webhooks_template::UpdateWebhookTemplateParamsNode,
    ) -> Result<core::webhooks::Webhook> {
        let params = params.into_core()?;
        self.inner
            .update_webhook_template(&webhook_id, &params)
            .await
            .map_err(errors::map_sdk_err)
    }
}

// ── KvStoreApiClient ───────────────────────────────────────────

#[derive(Clone)]
#[napi]
pub struct KvStoreApiClient {
    inner: core::kvstore::KvStoreApiClient,
}

#[napi]
impl KvStoreApiClient {
    /// Creates a new set, storing a single string value under the given key.
    #[napi]
    pub async fn create_set(&self, params: core::kvstore::CreateSetParams) -> Result<()> {
        self.inner
            .create_set(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns a paginated page of key/value entries from the store. Use the
    /// response `cursor` to fetch subsequent pages.
    #[napi]
    pub async fn get_sets(
        &self,
        params: Option<core::kvstore::GetSetsParams>,
    ) -> Result<core::kvstore::GetSetsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_sets(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns the string value stored for a single set by key.
    #[napi]
    pub async fn get_set(&self, key: String) -> Result<core::kvstore::GetSetResponse> {
        self.inner.get_set(&key).await.map_err(errors::map_sdk_err)
    }

    /// Adds and removes multiple sets in a single request. Either `add_sets`,
    /// `delete_sets`, or both may be supplied.
    #[napi]
    pub async fn bulk_sets(&self, params: core::kvstore::BulkSetsParams) -> Result<()> {
        self.inner
            .bulk_sets(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a single set by key.
    #[napi]
    pub async fn delete_set(&self, key: String) -> Result<()> {
        self.inner
            .delete_set(&key)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Creates a new list under the given key, seeded with the provided items.
    #[napi]
    pub async fn create_list(&self, params: core::kvstore::CreateListParams) -> Result<()> {
        self.inner
            .create_list(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns a paginated page of list keys from the store. Use the response
    /// `cursor` to fetch subsequent pages.
    #[napi]
    pub async fn get_lists(
        &self,
        params: Option<core::kvstore::GetListsParams>,
    ) -> Result<core::kvstore::GetListsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_lists(&params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Returns a paginated page of items from the list identified by `key`.
    /// Use the response `cursor` to fetch subsequent pages.
    #[napi]
    pub async fn get_list(
        &self,
        key: String,
        params: Option<core::kvstore::GetListParams>,
    ) -> Result<core::kvstore::GetListResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_list(&key, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Updates an existing list by adding and/or removing items in a single
    /// operation. Either `add_items`, `remove_items`, or both may be supplied.
    #[napi]
    pub async fn update_list(
        &self,
        key: String,
        params: core::kvstore::UpdateListParams,
    ) -> Result<()> {
        self.inner
            .update_list(&key, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Appends a single item to the list identified by `key`.
    #[napi]
    pub async fn add_list_item(
        &self,
        key: String,
        params: core::kvstore::AddListItemParams,
    ) -> Result<()> {
        self.inner
            .add_list_item(&key, &params)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Checks whether the specified list contains the given item.
    #[napi]
    pub async fn list_contains_item(
        &self,
        key: String,
        item: String,
    ) -> Result<core::kvstore::ListContainsItemResponse> {
        self.inner
            .list_contains_item(&key, &item)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a specific item from the list identified by `key`.
    #[napi]
    pub async fn delete_list_item(&self, key: String, item: String) -> Result<()> {
        self.inner
            .delete_list_item(&key, &item)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Removes a list and all of its items by key.
    #[napi]
    pub async fn delete_list(&self, key: String) -> Result<()> {
        self.inner
            .delete_list(&key)
            .await
            .map_err(errors::map_sdk_err)
    }
}

// ── SqlApiClient ───────────────────────────────────────────────

#[derive(Clone)]
#[napi]
pub struct SqlApiClient {
    inner: core::sql::SqlApiClient,
}

#[napi]
impl SqlApiClient {
    /// Executes a SQL query against the given cluster and returns the result set.
    #[napi]
    pub async fn query(&self, query: String, cluster_id: String) -> Result<sql::QueryResponseNode> {
        self.inner
            .query(&core::sql::QueryParams { query, cluster_id })
            .await
            .map(sql::QueryResponseNode::from)
            .map_err(errors::map_sdk_err)
    }

    /// Fetches the database schema for a cluster, including table names,
    /// columns, types, sort keys, and partition strategies.
    #[napi]
    pub async fn get_schema(&self, cluster_id: String) -> Result<sql::ChainSchemaNode> {
        self.inner
            .get_schema(&cluster_id)
            .await
            .map(sql::ChainSchemaNode::from)
            .map_err(errors::map_sdk_err)
    }
}

// ── RpcApiClient ───────────────────────────────────────────────

#[derive(Clone)]
#[napi]
pub struct RpcApiClient {
    inner: core::rpc::RpcApiClient,
}

#[napi]
impl RpcApiClient {
    /// Makes a JSON-RPC call against the account's Tooling Access endpoint,
    /// authenticated with a short-lived session JWT (minted and refreshed
    /// automatically). `params` accepts an array (positional) or object
    /// (by-name) and defaults to `[]`. `network` selects a chain on the
    /// multichain endpoint (a key in the seeded network map, e.g.
    /// `"solana-mainnet"`); omit for the endpoint's default network. Returns
    /// the JSON-RPC `result`. A JSON-RPC error is thrown as `RpcError`.
    #[napi]
    pub async fn call(
        &self,
        method: String,
        params: Option<serde_json::Value>,
        network: Option<String>,
    ) -> Result<serde_json::Value> {
        self.inner
            .call(&method, params, network)
            .await
            .map_err(errors::map_sdk_err)
    }

    /// Seeds the per-network URL map for multichain routing (network key ->
    /// full http_url), typically built from
    /// `admin.getEndpointUrls(...).multichainUrls`.
    #[napi]
    pub fn set_networks(&self, networks: std::collections::HashMap<String, String>) {
        self.inner.set_networks(networks);
    }

    /// Discards the in-memory cached token, forcing the next call to mint a
    /// fresh one. Use when the cached token is known stale beyond expiry.
    #[napi]
    pub fn clear_cached_token(&self) {
        self.inner.clear_cached_token();
    }

    /// Returns a snapshot of the currently cached session token, or `null` if
    /// no token has been minted or seeded yet. Hosts use this to persist the
    /// token between processes.
    #[napi]
    pub fn current_token(&self) -> Option<core::CachedToken> {
        self.inner.current_token()
    }
}
