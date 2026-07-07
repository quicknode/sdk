pub mod account;
pub mod billing;
pub mod bulk;
pub mod chains;
pub mod endpoint_metrics;
pub mod endpoint_rate_limits;
pub mod endpoint_security;
pub mod endpoint_urls;
pub mod endpoints;
pub mod logs;
pub mod tags;
pub mod teams;
pub mod usage;

pub use account::{AccountInfo, AccountInfoResponse, AccountSubscription};
pub use billing::{
    Invoice, InvoiceLine, ListInvoicesData, ListInvoicesResponse, ListPaymentsData,
    ListPaymentsResponse, Payment,
};
pub use bulk::{
    BulkAddTagData, BulkAddTagRequest, BulkAddTagResponse, BulkOperationResult, BulkRemoveTagData,
    BulkRemoveTagRequest, BulkRemoveTagResponse, BulkTag, BulkUpdateEndpointStatusData,
    BulkUpdateEndpointStatusRequest, BulkUpdateEndpointStatusResponse,
};
pub use chains::{Chain, ChainNetwork, ListChainsResponse};
pub use endpoint_metrics::{
    EndpointMetric, GetAccountMetricsRequest, GetAccountMetricsResponse, GetEndpointMetricsRequest,
    GetEndpointMetricsResponse,
};
pub use endpoint_rate_limits::{
    CreateMethodRateLimitRequest, CreateMethodRateLimitResponse, GetMethodRateLimitsData,
    GetMethodRateLimitsResponse, GetRateLimitsData, GetRateLimitsResponse, MethodRateLimiter,
    RateLimitEntry, RateLimitSettings, UpdateMethodRateLimitRequest, UpdateMethodRateLimitResponse,
    UpdateRateLimitsRequest,
};
pub use endpoint_security::{
    CreateDomainMaskRequest, CreateIpRequest, CreateJwtRequest,
    CreateOrUpdateIpCustomHeaderRequest, CreateOrUpdateIpCustomHeaderResponse,
    CreateReferrerRequest, CreateRequestFilterData, CreateRequestFilterRequest,
    CreateRequestFilterResponse, DeleteBoolResponse, GetSecurityOptionsResponse,
    IpCustomHeaderData, SecurityOption, SecurityOptionsUpdate, UpdateRequestFilterRequest,
    UpdateSecurityOptionsRequest, UpdateSecurityOptionsResponse,
};
pub use endpoint_urls::{EndpointUrl, GetEndpointUrlsData, GetEndpointUrlsResponse};
pub use endpoints::{
    CreateEndpointRequest, CreateEndpointResponse, CreateTagRequest, Endpoint, EndpointDomainMask,
    EndpointIp, EndpointIpCustomHeaderOption, EndpointJwt, EndpointRateLimits, EndpointReferrer,
    EndpointRequestFilter, EndpointSecurity, EndpointSecurityOptions, EndpointTag, EndpointToken,
    GetEndpointSecurityResponse, GetEndpointsRequest, GetEndpointsResponse, Pagination,
    ShowEndpointResponse, SingleEndpoint, UpdateEndpointRequest, UpdateEndpointStatusRequest,
    UpdateEndpointStatusResponse,
};
pub use logs::{
    EndpointLog, GetEndpointLogsRequest, GetEndpointLogsResponse, GetLogDetailsResponse, LogDetails,
};
pub use tags::{
    AccountTag, DeleteAccountTagData, DeleteAccountTagResponse, ListTagsData, ListTagsResponse,
    RenameTagRequest, RenameTagResponse,
};
pub use teams::{
    CreateTeamData, CreateTeamRequest, CreateTeamResponse, DeleteTeamData, DeleteTeamResponse,
    GetTeamResponse, InviteTeamMemberRequest, InviteTeamMemberResponse, ListTeamEndpointsResponse,
    ListTeamsResponse, RemoveTeamMemberRequest, RemoveTeamMemberResponse, ResendTeamInviteResponse,
    TeamDetail, TeamEndpoint, TeamMessageData, TeamSummary, TeamUser, UpdateTeamEndpointsData,
    UpdateTeamEndpointsRequest, UpdateTeamEndpointsResponse,
};

pub use usage::{
    ChainUsage, EndpointUsage, GetUsageByChainResponse, GetUsageByEndpointResponse,
    GetUsageByMethodResponse, GetUsageByTagResponse, GetUsageRequest, GetUsageResponse,
    MethodUsage, TagUsage, UsageByChainData, UsageByEndpointData, UsageByMethodData,
    UsageByTagData, UsageData,
};

use crate::{config::AdminConfig, errors::SdkError, SdkConfig};

const ADMIN_BASE_URL: &str = "https://api.quicknode.com/v0/";

pub(crate) struct ResolvedAdminConfig {
    pub(crate) base_url: reqwest::Url,
}

impl ResolvedAdminConfig {
    pub(crate) fn from_config(config: Option<&AdminConfig>) -> Result<Self, SdkError> {
        let url_str = config
            .and_then(|a| a.base_url.as_deref())
            .unwrap_or(ADMIN_BASE_URL);
        let mut base_url = reqwest::Url::parse(url_str)?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Ok(Self { base_url })
    }
}

/// Client for the Quicknode Admin API. Manage endpoints, tags, teams, billing,
/// usage/metrics, security, and rate limits on the account.
#[derive(Debug, Clone)]
pub struct AdminApiClient {
    config: SdkConfig,
}

impl AdminApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    /// Returns a paginated list of endpoints on the account. Supports searching
    /// by subdomain or label, filtering by networks, statuses, labels, and
    /// tags, and sorting. The response includes endpoint metadata (id, label,
    /// status, chain/network, HTTP and WebSocket URLs, tags) plus
    /// total/limit/offset pagination info.
    pub async fn get_endpoints(
        &self,
        params: &GetEndpointsRequest,
    ) -> Result<GetEndpointsResponse, SdkError> {
        let url = self.config.admin().base_url.join("endpoints")?;
        // Build query manually: serde_urlencoded (used by reqwest's .query())
        // rejects Vec<T> fields, but the API expects array params like
        // networks[]=mainnet for the filter/list query string.
        let query = endpoints_query(params);
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&query)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Creates a new endpoint for a given blockchain and network. Requires
    /// `chain` and `network`; returns the new endpoint with its HTTP and
    /// WebSocket URLs, default security configuration (tokens, JWTs, IPs,
    /// domain masks, CORS), and rate limits.
    pub async fn create_endpoint(
        &self,
        params: &CreateEndpointRequest,
    ) -> Result<CreateEndpointResponse, SdkError> {
        let url = self.config.admin().base_url.join("endpoints")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns details for a specific endpoint by ID.
    pub async fn show_endpoint(&self, id: &str) -> Result<ShowEndpointResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Updates editable fields on an endpoint (e.g. its label). Returns a
    /// boolean indicating whether the update succeeded.
    pub async fn update_endpoint(
        &self,
        id: &str,
        params: &UpdateEndpointRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Archives an endpoint. The API uses `DELETE` but the effect is archival
    /// rather than permanent deletion.
    pub async fn archive_endpoint(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}", id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Pauses or unpauses an endpoint by setting its status to `active` or
    /// `paused`.
    pub async fn update_endpoint_status(
        &self,
        id: &str,
        params: &UpdateEndpointStatusRequest,
    ) -> Result<UpdateEndpointStatusResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/status", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Creates a new tag on a specific endpoint from a label. Returns the new
    /// tag with its id, account info, and timestamps.
    pub async fn create_tag(&self, id: &str, params: &CreateTagRequest) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/tags", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a tag from a specific endpoint by tag id.
    pub async fn delete_tag(&self, id: &str, tag_id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/tags/{}", id, tag_id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Returns all teams on the account. Each team includes its id, name,
    /// member count, and member details (roles, contact info, account status).
    pub async fn list_teams(&self) -> Result<ListTeamsResponse, SdkError> {
        let url = self.config.admin().base_url.join("teams")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Creates a new team. Requires a `name`; returns the new team with its
    /// id, name, default role, and member count.
    pub async fn create_team(
        &self,
        params: &CreateTeamRequest,
    ) -> Result<CreateTeamResponse, SdkError> {
        let url = self.config.admin().base_url.join("teams")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns a specific team by id, including active members with their
    /// roles and contact info plus any pending invites.
    pub async fn get_team(&self, id: i64) -> Result<GetTeamResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Deletes a team by id. The team must have no members before it can be
    /// deleted.
    pub async fn delete_team(&self, id: i64) -> Result<DeleteTeamResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}", id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns the endpoints accessible to a given team. Each entry includes
    /// the endpoint id, subdomain, chain, and network.
    pub async fn list_team_endpoints(
        &self,
        id: i64,
    ) -> Result<ListTeamEndpointsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}/endpoints", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Assigns or unassigns endpoints for a team. Pass an array of endpoint ids
    /// to set the team's accessible endpoints; pass an empty array to remove
    /// all associations.
    pub async fn update_team_endpoints(
        &self,
        id: i64,
        params: &UpdateTeamEndpointsRequest,
    ) -> Result<UpdateTeamEndpointsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}/endpoints", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Invites a user to a team by email. For new users, `full_name` and
    /// `role` (`admin`, `viewer`, or `billing`) are also required. Returns the
    /// invited user's profile and invitation status.
    pub async fn invite_team_member(
        &self,
        id: i64,
        params: &InviteTeamMemberRequest,
    ) -> Result<InviteTeamMemberResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}/members", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Removes a user from a team by team id and user id.
    pub async fn remove_team_member(
        &self,
        id: i64,
        user_id: i64,
        params: &RemoveTeamMemberRequest,
    ) -> Result<RemoveTeamMemberResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}/members/{}", id, user_id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Resends the invitation email to a pending team member, identified by
    /// team id and user id.
    pub async fn resend_team_invite(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<ResendTeamInviteResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("teams/{}/members/{}/resend_invite", id, user_id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns account RPC usage totals for an optional time range. The
    /// response includes `credits_used`, `credits_remaining`, the account
    /// `limit`, any `overages`, and the queried time window.
    pub async fn get_usage(&self, params: &GetUsageRequest) -> Result<GetUsageResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/rpc")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns RPC usage broken down per endpoint over an optional time range.
    /// Each entry includes endpoint metadata, aggregate `credits_used` and
    /// `requests`, and a per-method credit breakdown.
    pub async fn get_usage_by_endpoint(
        &self,
        params: &GetUsageRequest,
    ) -> Result<GetUsageByEndpointResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/rpc/by-endpoint")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns RPC usage grouped by method over an optional time range. Each
    /// entry includes the method name, credits consumed, and archival status.
    /// Ranges longer than one week are rounded to midnight UTC.
    pub async fn get_usage_by_method(
        &self,
        params: &GetUsageRequest,
    ) -> Result<GetUsageByMethodResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/rpc/by-method")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns RPC usage grouped by chain over an optional time range. Each
    /// entry includes the chain and its credit consumption.
    pub async fn get_usage_by_chain(
        &self,
        params: &GetUsageRequest,
    ) -> Result<GetUsageByChainResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/rpc/by-chain")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns activity logs for a specific endpoint. Supports filtering by
    /// timestamp range and pagination. Each log entry includes timestamp,
    /// HTTP method, network, status code, and error data; full request/response
    /// bodies can be included when requested.
    pub async fn get_endpoint_logs(
        &self,
        id: &str,
        params: &GetEndpointLogsRequest,
    ) -> Result<GetEndpointLogsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/logs", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns the raw request and response payloads for a specific log entry
    /// on an endpoint, identified by request UUID. Both payloads are
    /// JSON-encoded strings and are truncated at 2KB.
    pub async fn get_log_details(
        &self,
        id: &str,
        request_id: &str,
    ) -> Result<GetLogDetailsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/log_details", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&[("request_id", request_id)])
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns the security options for an endpoint — an object of security
    /// feature toggles with their current enabled/disabled status.
    pub async fn get_security_options(
        &self,
        id: &str,
    ) -> Result<GetSecurityOptionsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security_options", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Updates which security features are enabled on an endpoint. Each option
    /// in the submitted object can be toggled `enabled` or `disabled` —
    /// examples include token auth, JWT validation, IP restrictions, CORS,
    /// HSTS, referrer validation, and domain masking.
    pub async fn update_security_options(
        &self,
        id: &str,
        params: &UpdateSecurityOptionsRequest,
    ) -> Result<UpdateSecurityOptionsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security_options", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Generates a new authentication token for an endpoint.
    pub async fn create_token(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/tokens", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Revokes a token on an endpoint by token id.
    pub async fn delete_token(
        &self,
        id: &str,
        token_id: &str,
    ) -> Result<DeleteBoolResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/tokens/{}", id, token_id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Adds a referrer to an endpoint's security settings, specifying which
    /// external URL or domain is permitted to call the endpoint.
    pub async fn create_referrer(
        &self,
        id: &str,
        params: &CreateReferrerRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/referrers", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a referrer from an endpoint's security settings by referrer id.
    pub async fn delete_referrer(
        &self,
        id: &str,
        referrer_id: &str,
    ) -> Result<DeleteBoolResponse, SdkError> {
        let url = self.config.admin().base_url.join(&format!(
            "endpoints/{}/security/referrers/{}",
            id, referrer_id
        ))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Adds an IP address to an endpoint's security whitelist.
    pub async fn create_ip(&self, id: &str, params: &CreateIpRequest) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/ips", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes an IP address from an endpoint's security whitelist by ip id.
    pub async fn delete_ip(&self, id: &str, ip_id: &str) -> Result<DeleteBoolResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/ips/{}", id, ip_id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Adds a domain mask to an endpoint — a custom domain used to hide the
    /// endpoint's Quicknode URL so requests can be routed through your own
    /// domain.
    pub async fn create_domain_mask(
        &self,
        id: &str,
        params: &CreateDomainMaskRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/domain_masks", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a domain mask from an endpoint by domain mask id.
    pub async fn delete_domain_mask(
        &self,
        id: &str,
        domain_mask_id: &str,
    ) -> Result<DeleteBoolResponse, SdkError> {
        let url = self.config.admin().base_url.join(&format!(
            "endpoints/{}/security/domain_masks/{}",
            id, domain_mask_id
        ))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Creates a new JWT for endpoint authentication. Accepts a public key,
    /// key id (`kid`), and token name.
    pub async fn create_jwt(&self, id: &str, params: &CreateJwtRequest) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/jwts", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a JWT from an endpoint's security configuration by jwt id,
    /// revoking its access.
    pub async fn delete_jwt(&self, id: &str, jwt_id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/jwts/{}", id, jwt_id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Creates a request filter on an endpoint — a method whitelist that
    /// restricts which RPC methods may be called. Accepts an array of method
    /// names; other methods are blocked.
    pub async fn create_request_filter(
        &self,
        id: &str,
        params: &CreateRequestFilterRequest,
    ) -> Result<CreateRequestFilterResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security/request_filters", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Updates an existing request filter on an endpoint, replacing the
    /// whitelisted method list.
    pub async fn update_request_filter(
        &self,
        id: &str,
        request_filter_id: &str,
        params: &UpdateRequestFilterRequest,
    ) -> Result<(), SdkError> {
        let url = self.config.admin().base_url.join(&format!(
            "endpoints/{}/security/request_filters/{}",
            id, request_filter_id
        ))?;
        let resp = self
            .config
            .http_client()
            .put(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a request filter from an endpoint's security configuration by
    /// request filter id.
    pub async fn delete_request_filter(
        &self,
        id: &str,
        request_filter_id: &str,
    ) -> Result<(), SdkError> {
        let url = self.config.admin().base_url.join(&format!(
            "endpoints/{}/security/request_filters/{}",
            id, request_filter_id
        ))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Enables multichain functionality on an endpoint, allowing a single
    /// endpoint to serve multiple chains.
    pub async fn enable_multichain(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/enable_multichain", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Disables multichain functionality on an endpoint.
    pub async fn disable_multichain(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/disable_multichain", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Sets the custom HTTP header used to identify the client IP for an
    /// endpoint (for example, `X-Forwarded-For`). This header is used by
    /// IP-based security features to resolve the real client address when
    /// requests are proxied.
    pub async fn create_or_update_ip_custom_header(
        &self,
        id: &str,
        params: &CreateOrUpdateIpCustomHeaderRequest,
    ) -> Result<CreateOrUpdateIpCustomHeaderResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/ip_custom_header", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Removes the custom IP header configuration from an endpoint.
    pub async fn delete_ip_custom_header(&self, id: &str) -> Result<DeleteBoolResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/ip_custom_header", id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns the method rate limits configured on an endpoint, including
    /// each limiter's interval, methods, rate, and status.
    pub async fn get_method_rate_limits(
        &self,
        id: &str,
    ) -> Result<GetMethodRateLimitsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/method-rate-limits", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Creates a per-method rate limit on an endpoint. A method rate limit
    /// caps specific RPC methods rather than the endpoint as a whole, defined
    /// by an `interval` (e.g. `second`), the target `methods`, and a `rate`.
    pub async fn create_method_rate_limit(
        &self,
        id: &str,
        params: &CreateMethodRateLimitRequest,
    ) -> Result<CreateMethodRateLimitResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/method-rate-limits", id))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Updates an existing method rate limit on an endpoint. Accepts the
    /// methods to apply the limit to, the desired `status`, and the `rate`.
    pub async fn update_method_rate_limit(
        &self,
        id: &str,
        method_rate_limit_id: &str,
        params: &UpdateMethodRateLimitRequest,
    ) -> Result<UpdateMethodRateLimitResponse, SdkError> {
        let url = self.config.admin().base_url.join(&format!(
            "endpoints/{}/method-rate-limits/{}",
            id, method_rate_limit_id
        ))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Removes a method rate limit from an endpoint by method rate limit id.
    pub async fn delete_method_rate_limit(
        &self,
        id: &str,
        method_rate_limit_id: &str,
    ) -> Result<(), SdkError> {
        let url = self.config.admin().base_url.join(&format!(
            "endpoints/{}/method-rate-limits/{}",
            id, method_rate_limit_id
        ))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Partial update of the endpoint-level rate-limit overrides. Accepts
    /// `rps` (requests per second), `rpm` (requests per minute), and `rpd`
    /// (requests per day). Only buckets included in the request body are
    /// modified — omitted buckets are left unchanged. Values are capped by the
    /// account's plan tier.
    pub async fn update_rate_limits(
        &self,
        id: &str,
        params: &UpdateRateLimitsRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/rate-limits", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Returns the endpoint-level rate limits currently enforced, with each
    /// row identifying its bucket (`rps`/`rpm`/`rpd`), value, and source
    /// (`plan_default` or `user_override`). User-set overrides expose an
    /// `override_id` that can be passed to `delete_rate_limit_override`.
    pub async fn get_rate_limits(&self, id: &str) -> Result<GetRateLimitsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/rate-limits", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Deletes a user-set rate-limit override by its UUID. Plan defaults are
    /// not deletable — passing a UUID that does not match a user-set override
    /// on the endpoint returns 404.
    pub async fn delete_rate_limit_override(
        &self,
        id: &str,
        override_id: &str,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/rate-limits/{}", id, override_id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Returns the HTTP and WebSocket URLs for the endpoint without fetching
    /// the full endpoint record. For multichain endpoints, `multichain_urls`
    /// is a per-network map of additional URLs; for single-chain endpoints it
    /// is `None`.
    pub async fn get_endpoint_urls(&self, id: &str) -> Result<GetEndpointUrlsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/urls", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns time-series metrics for a specific endpoint. Requires a
    /// `period` (`hour`, `day`, `week`, or `month`) and a metric type such as
    /// `method_calls_over_time` or `response_status_breakdown`.
    pub async fn get_endpoint_metrics(
        &self,
        id: &str,
        params: &GetEndpointMetricsRequest,
    ) -> Result<GetEndpointMetricsResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/metrics", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns aggregated metrics across all endpoints on the account. Accepts
    /// a `period` (`hour`, `day`, `week`, or `month`) and a metric type such
    /// as `method_calls_over_time` or `credits_over_time`.
    pub async fn get_account_metrics(
        &self,
        params: &GetAccountMetricsRequest,
    ) -> Result<GetAccountMetricsResponse, SdkError> {
        let url = self.config.admin().base_url.join("metrics")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns all chains supported by Quicknode along with their networks.
    /// Each entry includes the chain slug and its network slugs and names.
    pub async fn list_chains(&self) -> Result<ListChainsResponse, SdkError> {
        let url = self.config.admin().base_url.join("chains")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns details about the account, including its id, name, creation
    /// timestamp, billing version, and current subscription.
    pub async fn account_info(&self) -> Result<AccountInfoResponse, SdkError> {
        let url = self.config.admin().base_url.join("account/info")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns the account's invoices, including id, status, billing reason,
    /// amounts due and paid, line items with descriptions and billing periods,
    /// and creation timestamps.
    pub async fn list_invoices(&self) -> Result<ListInvoicesResponse, SdkError> {
        let url = self.config.admin().base_url.join("billing/invoices")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns all payments on the account, including amount, status, card
    /// last-four, timestamp, currency, and marketplace spending.
    pub async fn list_payments(&self) -> Result<ListPaymentsResponse, SdkError> {
        let url = self.config.admin().base_url.join("billing/payments")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Pauses or unpauses multiple endpoints in a single call. Accepts an
    /// array of endpoint ids and a target status (`active` or `paused`);
    /// returns per-endpoint success/failure results plus totals.
    pub async fn bulk_update_endpoint_status(
        &self,
        params: &BulkUpdateEndpointStatusRequest,
    ) -> Result<BulkUpdateEndpointStatusResponse, SdkError> {
        if params.ids.is_empty() {
            return Err(SdkError::Config(
                "bulk_update_endpoint_status requires at least one id".into(),
            ));
        }
        let url = self.config.admin().base_url.join("endpoints/bulk/status")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Applies a single tag label to multiple endpoints in one call. Returns
    /// totals for affected endpoints, successes, and failures, plus the tag
    /// that was applied.
    pub async fn bulk_add_tag(
        &self,
        params: &BulkAddTagRequest,
    ) -> Result<BulkAddTagResponse, SdkError> {
        if params.ids.is_empty() {
            return Err(SdkError::Config(
                "bulk_add_tag requires at least one id".into(),
            ));
        }
        let url = self.config.admin().base_url.join("endpoints/bulk/tags")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Removes a tag from multiple endpoints in one call, identified by an
    /// array of endpoint ids and a tag id.
    pub async fn bulk_remove_tag(
        &self,
        params: &BulkRemoveTagRequest,
    ) -> Result<BulkRemoveTagResponse, SdkError> {
        // Empty ids on a DELETE-with-body is high blast radius: some proxies
        // strip DELETE bodies, and an empty batch could be misinterpreted by
        // the server. Fail fast client-side before firing the request.
        if params.ids.is_empty() {
            return Err(SdkError::Config(
                "bulk_remove_tag requires at least one id".into(),
            ));
        }
        let url = self.config.admin().base_url.join("endpoints/bulk/tags")?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns all account-level tags, including tags with zero associated
    /// endpoints. Each tag includes its id, label, and endpoint usage count.
    pub async fn list_tags(&self) -> Result<ListTagsResponse, SdkError> {
        let url = self.config.admin().base_url.join("endpoints/tags")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Updates the label of an account tag. Because the tag is shared across
    /// endpoints, all associated endpoints reflect the new label immediately.
    pub async fn rename_tag(
        &self,
        id: i32,
        params: &RenameTagRequest,
    ) -> Result<RenameTagResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/tags/{}", id))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    // Named delete_account_tag to avoid collision with the existing per-endpoint
    // delete_tag(id, tag_id). OpenAPI reuses the deleteTag operationId for both.
    /// Deletes an account-level tag. The tag must first be removed from all
    /// endpoints before it can be deleted.
    pub async fn delete_account_tag(&self, id: i32) -> Result<DeleteAccountTagResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/tags/{}", id))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns RPC usage grouped by endpoint tag over an optional time range.
    /// Each entry includes the tag id, label, credits consumed, and request
    /// count.
    pub async fn get_usage_by_tag(
        &self,
        params: &GetUsageRequest,
    ) -> Result<GetUsageByTagResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/rpc/by-tag")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(params)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }

    /// Returns the full security configuration for an endpoint in a single
    /// call, without loading the entire endpoint object. The response includes
    /// tokens, JWTs, referrers, domain masks, IPs, and a security options
    /// object describing which features are enabled.
    pub async fn get_endpoint_security(
        &self,
        id: &str,
    ) -> Result<GetEndpointSecurityResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join(&format!("endpoints/{}/security", id))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;

        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;

        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })
    }
}

fn endpoints_query(params: &GetEndpointsRequest) -> Vec<(&'static str, String)> {
    let mut q: Vec<(&'static str, String)> = Vec::new();
    if let Some(v) = params.limit {
        q.push(("limit", v.to_string()));
    }
    if let Some(v) = params.offset {
        q.push(("offset", v.to_string()));
    }
    if let Some(ref v) = params.search {
        q.push(("search", v.clone()));
    }
    if let Some(ref v) = params.sort_by {
        q.push(("sort_by", v.clone()));
    }
    if let Some(ref v) = params.sort_direction {
        q.push(("sort_direction", v.clone()));
    }
    if let Some(ref list) = params.networks {
        for item in list {
            q.push(("networks[]", item.clone()));
        }
    }
    if let Some(ref list) = params.statuses {
        for item in list {
            q.push(("statuses[]", item.clone()));
        }
    }
    if let Some(ref list) = params.labels {
        for item in list {
            q.push(("labels[]", item.clone()));
        }
    }
    if let Some(v) = params.dedicated {
        q.push(("dedicated", v.to_string()));
    }
    if let Some(v) = params.is_flat_rate {
        q.push(("is_flat_rate", v.to_string()));
    }
    if let Some(ref list) = params.tag_ids {
        for item in list {
            q.push(("tag_ids[]", item.to_string()));
        }
    }
    if let Some(ref list) = params.tag_labels {
        for item in list {
            q.push(("tag_labels[]", item.clone()));
        }
    }
    q
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{AdminConfig, QuicknodeSdk, SdkFullConfig};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuicknodeSdk {
        QuicknodeSdk::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: None,
            admin: Some(AdminConfig {
                base_url: Some(base_url),
            }),
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn get_endpoints_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "id": "abc123",
                        "name": "aged-intensive-patron",
                        "label": "My Endpoint",
                        "status": "active",
                        "chain": "ethereum",
                        "network": "mainnet",
                        "is_dedicated": false,
                        "is_flat_rate": true,
                        "http_url": "https://example.quicknode.pro/abc123",
                        "wss_url": null,
                        "tags": []
                    }
                ],
                "pagination": {
                    "total": 1,
                    "limit": 20,
                    "offset": 0
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .get_endpoints(&GetEndpointsRequest::default())
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, "abc123");
        assert_eq!(resp.data[0].name, "aged-intensive-patron");
        assert_eq!(resp.data[0].status, "active");
        assert_eq!(resp.data[0].chain, "ethereum");
        assert!(!resp.data[0].is_dedicated);
        assert!(resp.data[0].is_flat_rate);
        let pagination = resp.pagination.expect("pagination present");
        assert_eq!(pagination.total, 1);
        assert_eq!(pagination.limit, 20);
        assert_eq!(pagination.offset, 0);
    }

    #[tokio::test]
    async fn get_endpoints_sends_search_and_filter_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .and(query_param("search", "intensive"))
            .and(query_param("networks[]", "mainnet"))
            .and(query_param("statuses[]", "active"))
            .and(query_param("dedicated", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetEndpointsRequest {
            search: Some("intensive".to_string()),
            networks: Some(vec!["mainnet".to_string()]),
            statuses: Some(vec!["active".to_string()]),
            dedicated: Some(true),
            ..Default::default()
        };
        let resp = sdk.admin.get_endpoints(&params).await.unwrap();

        assert_eq!(resp.data.len(), 0);
    }

    #[tokio::test]
    async fn get_endpoints_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .admin
            .get_endpoints(&GetEndpointsRequest::default())
            .await
            .unwrap_err();

        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 401),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_endpoints_sends_query_params() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .and(query_param("limit", "10"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetEndpointsRequest {
            limit: Some(10),
            ..Default::default()
        };
        let resp = sdk.admin.get_endpoints(&params).await.unwrap();

        assert_eq!(resp.data.len(), 0);
    }

    #[tokio::test]
    async fn get_endpoints_base_url_without_trailing_slash() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [],
                "error": null
            })))
            .mount(&server)
            .await;

        let base_url_no_slash = server.uri();
        let sdk = make_sdk(base_url_no_slash);
        let resp = sdk
            .admin
            .get_endpoints(&GetEndpointsRequest::default())
            .await
            .unwrap();

        assert_eq!(resp.data.len(), 0);
    }

    #[tokio::test]
    async fn create_endpoint_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": "ep123",
                    "label": null,
                    "status": "active",
                    "chain": "ethereum",
                    "network": "mainnet",
                    "http_url": "https://example.quicknode.pro/ep123",
                    "wss_url": null,
                    "security": {
                        "options": { "tokens": true, "jwts": false, "domainMasks": false, "ips": false, "referrers": false, "requestFilters": false },
                        "tokens": [{"id": "tok1", "token": "abc123"}],
                        "jwts": null,
                        "referrers": null,
                        "domain_masks": null,
                        "ips": null,
                        "request_filters": null
                    },
                    "rate_limits": null,
                    "tags": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .create_endpoint(&CreateEndpointRequest::default())
            .await
            .unwrap();

        assert_eq!(resp.data.id, "ep123");
        assert_eq!(resp.data.chain, "ethereum");
        assert_eq!(resp.data.network, "mainnet");
        let security = resp.data.security.unwrap();
        assert!(security.tokens.unwrap().len() == 1);
        assert!(security.jwts.is_none());
    }

    #[tokio::test]
    async fn create_endpoint_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .admin
            .create_endpoint(&CreateEndpointRequest::default())
            .await
            .unwrap_err();

        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 400),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn create_endpoint_sends_body() {
        use wiremock::matchers::body_json;

        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints"))
            .and(body_json(serde_json::json!({
                "chain": "solana",
                "network": "mainnet-beta"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": "ep456",
                    "label": null,
                    "status": "active",
                    "chain": "solana",
                    "network": "mainnet-beta",
                    "http_url": "https://example.quicknode.pro/ep456",
                    "wss_url": null,
                    "security": null,
                    "rate_limits": null,
                    "tags": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateEndpointRequest {
            chain: Some("solana".to_string()),
            network: Some("mainnet-beta".to_string()),
        };
        let resp = sdk.admin.create_endpoint(&params).await.unwrap();

        assert_eq!(resp.data.id, "ep456");
        assert_eq!(resp.data.chain, "solana");
    }

    #[tokio::test]
    async fn show_endpoint_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": "ep123",
                    "label": null,
                    "status": "active",
                    "chain": "ethereum",
                    "network": "mainnet",
                    "http_url": "https://example.quicknode.pro/ep123",
                    "wss_url": null,
                    "security": null,
                    "rate_limits": null,
                    "tags": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.show_endpoint("ep123").await.unwrap();
        assert_eq!(resp.data.unwrap().id, "ep123");
    }

    #[tokio::test]
    async fn show_endpoint_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.show_endpoint("ep123").await.unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 404),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn update_endpoint_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/endpoints/ep123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin
            .update_endpoint(
                "ep123",
                &UpdateEndpointRequest {
                    label: Some("New Name".to_string()),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn archive_endpoint_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/endpoints/ep123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin.archive_endpoint("ep123").await.unwrap();
    }

    #[tokio::test]
    async fn update_endpoint_status_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/endpoints/ep123/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": "paused",
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .update_endpoint_status(
                "ep123",
                &UpdateEndpointStatusRequest {
                    status: "paused".to_string(),
                },
            )
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap(), "paused");
    }

    #[tokio::test]
    async fn create_tag_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints/ep123/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin
            .create_tag(
                "ep123",
                &CreateTagRequest {
                    label: Some("my-tag".to_string()),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_tag_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/endpoints/ep123/tags/tag456"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin.delete_tag("ep123", "tag456").await.unwrap();
    }

    #[tokio::test]
    async fn get_usage_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/usage/rpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "credits_used": 5000,
                    "credits_remaining": 95000,
                    "limit": 100000,
                    "overages": null,
                    "start_time": 1700000000,
                    "end_time": 1702592000
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .get_usage(&GetUsageRequest::default())
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap().credits_used, 5000);
    }

    #[tokio::test]
    async fn get_usage_by_endpoint_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/usage/rpc/by-endpoint"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "endpoints": [{"name": "ep1", "chain": "eth", "network": "mainnet", "status": "active", "credits_used": 100, "label": null, "methods_breakdown": [], "requests": 50}],
                    "start_time": 1700000000,
                    "end_time": 1702592000
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .get_usage_by_endpoint(&GetUsageRequest::default())
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap().endpoints.len(), 1);
    }

    #[tokio::test]
    async fn get_usage_by_chain_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/usage/rpc/by-chain"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "chains": [{"name": "ethereum", "credits_used": 1000}],
                    "start_time": 1700000000,
                    "end_time": 1702592000
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .admin
            .get_usage_by_chain(&GetUsageRequest::default())
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap().chains[0].name, "ethereum");
    }

    #[tokio::test]
    async fn get_endpoint_logs_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/logs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "timestamp": "2025-04-29T12:39:25.543Z",
                        "method": "eth_call",
                        "network": "mainnet",
                        "http_method": "POST",
                        "status": 200,
                        "error_code": null,
                        "url": "/",
                        "request_id": "abc-123",
                        "details": null
                    }
                ],
                "next_at": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetEndpointLogsRequest {
            from: "2025-04-29T00:00:00Z".to_string(),
            to: "2025-04-29T23:59:59Z".to_string(),
            ..Default::default()
        };
        let resp = sdk.admin.get_endpoint_logs("ep123", &params).await.unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].method.as_deref(), Some("eth_call"));
    }

    #[tokio::test]
    async fn get_log_details_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/log_details"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "request": "{\"method\":\"eth_call\"}",
                    "response": "{\"result\":\"0x1\"}"
                }
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_log_details("ep123", "abc-123").await.unwrap();
        assert!(resp.data.is_some());
    }

    #[tokio::test]
    async fn get_security_options_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/security_options"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"option": "tokens", "status": "enabled", "value": null}],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_security_options("ep123").await.unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].option, "tokens");
    }

    #[tokio::test]
    async fn update_security_options_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/endpoints/ep123/security_options"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"option": "tokens", "status": "disabled", "value": null}],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateSecurityOptionsRequest {
            options: SecurityOptionsUpdate {
                tokens: Some("disabled".to_string()),
                ..Default::default()
            },
        };
        let resp = sdk
            .admin
            .update_security_options("ep123", &params)
            .await
            .unwrap();
        assert_eq!(resp.data[0].status, "disabled");
    }

    #[tokio::test]
    async fn create_token_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints/ep123/security/tokens"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin.create_token("ep123").await.unwrap();
    }

    #[tokio::test]
    async fn delete_token_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/endpoints/ep123/security/tokens/tok1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": true, "error": null})),
            )
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.delete_token("ep123", "tok1").await.unwrap();
        assert_eq!(resp.data, Some(true));
    }

    #[tokio::test]
    async fn create_referrer_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints/ep123/security/referrers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin
            .create_referrer(
                "ep123",
                &CreateReferrerRequest {
                    referrer: "example.com".to_string(),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn enable_disable_multichain_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints/ep123/enable_multichain"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/endpoints/ep123/disable_multichain"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin.enable_multichain("ep123").await.unwrap();
        sdk.admin.disable_multichain("ep123").await.unwrap();
    }

    #[tokio::test]
    async fn create_or_update_ip_custom_header_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/endpoints/ep123/ip_custom_header"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"header_name": "CF-Connecting-IP"},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateOrUpdateIpCustomHeaderRequest {
            header_name: "CF-Connecting-IP".to_string(),
        };
        let resp = sdk
            .admin
            .create_or_update_ip_custom_header("ep123", &params)
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap().header_name, "CF-Connecting-IP");
    }

    #[tokio::test]
    async fn get_method_rate_limits_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/method-rate-limits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "rate_limiters": [
                        {"id": "rl1", "interval": "second", "methods": ["eth_call"], "rate": 10, "status": "enabled", "created": "2024-01-01T00:00:00Z"}
                    ]
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_method_rate_limits("ep123").await.unwrap();
        assert_eq!(resp.data.unwrap().rate_limiters.len(), 1);
    }

    #[tokio::test]
    async fn create_method_rate_limit_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/endpoints/ep123/method-rate-limits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "rl1", "interval": "second", "methods": ["eth_call"], "rate": 10, "status": "enabled", "created": "2024-01-01T00:00:00Z"},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateMethodRateLimitRequest {
            interval: "second".to_string(),
            methods: vec!["eth_call".to_string()],
            rate: 10,
        };
        let resp = sdk
            .admin
            .create_method_rate_limit("ep123", &params)
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap().id, "rl1");
    }

    #[tokio::test]
    async fn update_method_rate_limit_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/endpoints/ep123/method-rate-limits/rl1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": "rl1", "interval": "day", "methods": ["eth_call"], "rate": 30, "status": "enabled", "created": "2024-01-01T00:00:00Z"},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateMethodRateLimitRequest {
            rate: Some(30),
            ..Default::default()
        };
        let resp = sdk
            .admin
            .update_method_rate_limit("ep123", "rl1", &params)
            .await
            .unwrap();
        assert_eq!(resp.data.unwrap().rate, 30);
    }

    #[tokio::test]
    async fn delete_method_rate_limit_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/endpoints/ep123/method-rate-limits/rl1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": "deleted", "error": null})),
            )
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin
            .delete_method_rate_limit("ep123", "rl1")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn update_rate_limits_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/endpoints/ep123/rate-limits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateRateLimitsRequest {
            rate_limits: RateLimitSettings {
                rps: Some(100),
                rpm: None,
                rpd: None,
            },
        };
        sdk.admin
            .update_rate_limits("ep123", &params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn get_rate_limits_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/rate-limits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "rate_limits": [
                        {"bucket": "rps", "rate_limit": 100, "source": "plan_default"},
                        {"bucket": "rpm", "rate_limit": 6000, "source": "user_override", "id": "ovr-1"}
                    ]
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_rate_limits("ep123").await.unwrap();
        let rows = resp.data.unwrap().rate_limits;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].source, "plan_default");
        assert!(rows[0].id.is_none());
        assert_eq!(rows[1].source, "user_override");
        assert_eq!(rows[1].rate_limit, 6000);
        assert_eq!(rows[1].id.as_deref(), Some("ovr-1"));
    }

    #[tokio::test]
    async fn get_rate_limits_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/missing/rate-limits"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.get_rate_limits("missing").await.unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 404),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn delete_rate_limit_override_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/endpoints/ep123/rate-limits/ovr-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(""))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.admin
            .delete_rate_limit_override("ep123", "ovr-1")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_rate_limit_override_not_found() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/endpoints/ep123/rate-limits/bogus"))
            .respond_with(ResponseTemplate::new(404).set_body_string("override not found"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .admin
            .delete_rate_limit_override("ep123", "bogus")
            .await
            .unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 404),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_endpoint_urls_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/urls"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "http_url": "https://example.quiknode.pro/abc/",
                    "wss_url": "wss://example.quiknode.pro/abc/",
                    "multichain_urls": null
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_endpoint_urls("ep123").await.unwrap();
        let data = resp.data.unwrap();
        assert_eq!(data.http_url, "https://example.quiknode.pro/abc/");
        assert!(data.multichain_urls.is_none());
    }

    #[tokio::test]
    async fn get_endpoint_urls_multichain() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/urls"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "http_url": "https://example.quiknode.pro/abc/",
                    "wss_url": null,
                    "multichain_urls": {
                        "ethereum-mainnet": {
                            "http_url": "https://example.quiknode.pro/abc/eth/",
                            "wss_url": "wss://example.quiknode.pro/abc/eth/"
                        }
                    }
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_endpoint_urls("ep123").await.unwrap();
        let data = resp.data.unwrap();
        let mc = data.multichain_urls.unwrap();
        assert_eq!(mc.len(), 1);
        assert_eq!(
            mc.get("ethereum-mainnet").unwrap().http_url,
            "https://example.quiknode.pro/abc/eth/"
        );
    }

    #[tokio::test]
    async fn get_endpoint_metrics_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"data": [[1700000000, 42]], "tag": ["network", "mainnet"]}],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetEndpointMetricsRequest {
            period: "day".to_string(),
            metric: "credits_over_time".to_string(),
        };
        let resp = sdk
            .admin
            .get_endpoint_metrics("ep123", &params)
            .await
            .unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(
            resp.data[0].tag,
            vec!["network".to_string(), "mainnet".to_string()]
        );
    }

    #[tokio::test]
    async fn get_account_metrics_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"data": [[1700000000, 100]], "tag": "total"}],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetAccountMetricsRequest {
            period: "week".to_string(),
            metric: "credits_over_time".to_string(),
            percentile: None,
        };
        let resp = sdk.admin.get_account_metrics(&params).await.unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].tag, vec!["total".to_string()]);
    }

    // Regression: the metrics endpoints return `tag` as either a plain string
    // (single-axis series) or a `[key, value]` tuple (multi-axis series).
    // Exercise both shapes so any future serde change that breaks either
    // branch fails loudly.
    #[tokio::test]
    async fn get_account_metrics_decodes_tuple_tag() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"tag": ["network", "arbitrum-mainnet"], "data": [[1779109200, 40]]},
                    {"tag": ["network", "mainnet"], "data": [[1779116400, 40]]},
                    {"tag": "p95", "data": [[1779116400, 12]]}
                ],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetAccountMetricsRequest {
            period: "day".to_string(),
            metric: "credits_over_time".to_string(),
            percentile: None,
        };
        let resp = sdk.admin.get_account_metrics(&params).await.unwrap();
        assert_eq!(resp.data.len(), 3);
        assert_eq!(
            resp.data[0].tag,
            vec!["network".to_string(), "arbitrum-mainnet".to_string()]
        );
        assert_eq!(
            resp.data[1].tag,
            vec!["network".to_string(), "mainnet".to_string()]
        );
        assert_eq!(resp.data[2].tag, vec!["p95".to_string()]);
    }

    #[tokio::test]
    async fn list_chains_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/chains"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "slug": "eth",
                        "networks": [{"slug": "mainnet", "name": "Ethereum Mainnet", "chain_id": 1}],
                        "is_select_chain": true
                    }
                ],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.list_chains().await.unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].slug, "eth");
        assert_eq!(resp.data[0].networks[0].chain_id, Some(1));
    }

    #[tokio::test]
    async fn account_info_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/account/info"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": 794770,
                    "name": "MCP Test Account",
                    "created_at": "2026-03-27T20:22:32.536Z",
                    "billing_version": "v6",
                    "subscription": {
                        "plan_name": "Accelerate",
                        "status": "active",
                        "interval": "monthly"
                    }
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.account_info().await.unwrap();
        let data = resp.data.expect("expected account data");
        assert_eq!(data.id, 794770);
        assert_eq!(data.name, "MCP Test Account");
        assert_eq!(data.billing_version.as_deref(), Some("v6"));
        let subscription = data.subscription.expect("expected subscription");
        assert_eq!(subscription.plan_name.as_deref(), Some("Accelerate"));
        assert_eq!(subscription.status.as_deref(), Some("active"));
        assert_eq!(subscription.interval.as_deref(), Some("monthly"));
    }

    #[tokio::test]
    async fn account_info_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/account/info"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.account_info().await.unwrap_err();
        let SdkError::Api { status, .. } = err else {
            unreachable!("expected SdkError::Api, got {err:?}");
        };
        assert_eq!(status.as_u16(), 401);
    }

    #[tokio::test]
    async fn list_invoices_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/billing/invoices"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "invoices": [
                        {
                            "id": "inv123",
                            "status": "paid",
                            "billing_reason": "subscription",
                            "lines": [{"description": "Pro plan", "amount": 4900}],
                            "amount_due": 4900,
                            "amount_paid": 4900,
                            "period_start": 1700000000,
                            "period_end": 1702592000,
                            "created": 1700000000,
                            "subtotal": 4900
                        }
                    ]
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.list_invoices().await.unwrap();
        let data = resp.data.unwrap();
        assert_eq!(data.invoices.len(), 1);
        assert_eq!(data.invoices[0].id, "inv123");
    }

    #[tokio::test]
    async fn list_invoices_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/billing/invoices"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.list_invoices().await.unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 401),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn list_payments_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/billing/payments"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "payments": [
                        {
                            "amount": "49.00",
                            "card_last_4": "4242",
                            "created_at": "2024-01-01T00:00:00Z",
                            "currency": "usd",
                            "status": "succeeded",
                            "marketplace_amount": "9.0"
                        }
                    ]
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.list_payments().await.unwrap();
        let data = resp.data.unwrap();
        assert_eq!(data.payments.len(), 1);
        assert_eq!(data.payments[0].currency, "usd");
    }

    #[tokio::test]
    async fn list_teams_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/teams"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"id": 1, "name": "Engineering", "members_count": 5, "users": []}],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.list_teams().await.unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].name, "Engineering");
    }

    #[tokio::test]
    async fn list_teams_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/teams"))
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.list_teams().await.unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 401),
            other => panic!("expected SdkError::Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn create_team_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/teams"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"id": 42, "name": "New Team", "default_role": null, "members_count": 0},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = CreateTeamRequest {
            name: "New Team".to_string(),
        };
        let resp = sdk.admin.create_team(&params).await.unwrap();
        assert_eq!(resp.data.unwrap().id, 42);
    }

    #[tokio::test]
    async fn get_team_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/teams/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": 1,
                    "name": "Engineering",
                    "default_role": "member",
                    "members_count": 3,
                    "users": [],
                    "pending_invites": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_team(1).await.unwrap();
        assert_eq!(resp.data.unwrap().name, "Engineering");
    }

    #[tokio::test]
    async fn delete_team_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/teams/1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"message": "Team deleted"},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.delete_team(1).await.unwrap();
        assert!(resp.data.is_some());
    }

    #[tokio::test]
    async fn list_team_endpoints_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/teams/1/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"id": 10, "subdomain": "abc123", "chain": "ethereum", "network": "mainnet"}],
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.list_team_endpoints(1).await.unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].subdomain, "abc123");
    }

    #[tokio::test]
    async fn update_team_endpoints_success() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/teams/1/endpoints"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"success": true},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateTeamEndpointsRequest {
            endpoint_ids: vec!["ep1".to_string()],
        };
        let resp = sdk.admin.update_team_endpoints(1, &params).await.unwrap();
        assert!(resp.data.unwrap().success.unwrap());
    }

    // Wire-inspection regression: confirm an empty endpoint_ids array reaches
    // the wire as `[]` (not omitted), so any future `skip_serializing_if`
    // change that drops the empty case fails loudly.
    #[tokio::test]
    async fn update_team_endpoints_empty_array_wire_body() {
        use wiremock::matchers::body_json;
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/teams/1/endpoints"))
            .and(body_json(serde_json::json!({ "endpoint_ids": [] })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"success": true},
                "error": null
            })))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateTeamEndpointsRequest {
            endpoint_ids: vec![],
        };
        sdk.admin.update_team_endpoints(1, &params).await.unwrap();
    }

    #[tokio::test]
    async fn invite_team_member_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/teams/1/members"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "id": 99,
                    "email": "user@example.com",
                    "full_name": null,
                    "role": "member",
                    "status": "pending",
                    "created_at": null,
                    "photo_url": null,
                    "account_primary_user": null
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = InviteTeamMemberRequest {
            email: "user@example.com".to_string(),
            full_name: None,
            role: None,
        };
        let resp = sdk.admin.invite_team_member(1, &params).await.unwrap();
        assert_eq!(resp.data.unwrap().email, "user@example.com");
    }

    #[tokio::test]
    async fn remove_team_member_success() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/teams/1/members/99"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"message": "Member removed"},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = RemoveTeamMemberRequest { destroy_user: None };
        let resp = sdk.admin.remove_team_member(1, 99, &params).await.unwrap();
        assert!(resp.data.is_some());
    }

    #[tokio::test]
    async fn resend_team_invite_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/teams/1/members/99/resend_invite"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"message": "Invite resent"},
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.resend_team_invite(1, 99).await.unwrap();
        assert!(resp.data.is_some());
    }

    #[tokio::test]
    async fn bulk_update_endpoint_status_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/endpoints/bulk/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "total": 2,
                    "updated_count": 2,
                    "failed_count": 0,
                    "results": [
                        { "id": "a", "success": true },
                        { "id": "b", "success": true }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = BulkUpdateEndpointStatusRequest {
            ids: vec!["a".to_string(), "b".to_string()],
            status: "paused".to_string(),
        };
        let resp = sdk
            .admin
            .bulk_update_endpoint_status(&params)
            .await
            .unwrap();
        let data = resp.data.expect("data present");
        assert_eq!(data.total, 2);
        assert_eq!(data.updated_count, 2);
        assert_eq!(data.results.len(), 2);
    }

    #[tokio::test]
    async fn bulk_update_endpoint_status_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/endpoints/bulk/status"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = BulkUpdateEndpointStatusRequest {
            ids: vec!["a".to_string()],
            status: "paused".to_string(),
        };
        let err = sdk
            .admin
            .bulk_update_endpoint_status(&params)
            .await
            .unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 400),
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn bulk_add_tag_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/endpoints/bulk/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "total": 1,
                    "updated_count": 1,
                    "failed_count": 0,
                    "results": [{ "id": "a", "success": true }],
                    "tag": { "tag_id": 7, "label": "prod" }
                }
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = BulkAddTagRequest {
            ids: vec!["a".to_string()],
            label: "prod".to_string(),
        };
        let resp = sdk.admin.bulk_add_tag(&params).await.unwrap();
        let data = resp.data.expect("data present");
        assert_eq!(data.tag.tag_id, 7);
        assert_eq!(data.tag.label, "prod");
    }

    #[tokio::test]
    async fn bulk_remove_tag_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/endpoints/bulk/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "total": 2,
                    "updated_count": 2,
                    "failed_count": 0,
                    "results": [
                        { "id": "a", "success": true },
                        { "id": "b", "success": true }
                    ]
                }
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = BulkRemoveTagRequest {
            ids: vec!["a".to_string(), "b".to_string()],
            tag_id: 42,
        };
        let resp = sdk.admin.bulk_remove_tag(&params).await.unwrap();
        assert_eq!(resp.data.expect("data").updated_count, 2);
    }

    #[test]
    fn endpoint_token_debug_is_redacted() {
        let t = EndpointToken {
            id: "tok_1".to_string(),
            token: "super-secret".to_string(),
        };
        let dbg = format!("{t:?}");
        assert!(dbg.contains("tok_1"));
        assert!(!dbg.contains("super-secret"));
        assert!(dbg.contains("[redacted]"));
    }

    #[test]
    fn endpoint_jwt_debug_redacts_public_key() {
        let j = EndpointJwt {
            id: "jwt_1".to_string(),
            public_key: "-----BEGIN PUBLIC KEY-----\nAAAA\n-----END PUBLIC KEY-----".to_string(),
            kid: "kid1".to_string(),
            name: "myjwt".to_string(),
        };
        let dbg = format!("{j:?}");
        assert!(dbg.contains("jwt_1"));
        assert!(dbg.contains("kid1"));
        assert!(!dbg.contains("BEGIN PUBLIC KEY"));
        assert!(dbg.contains("[redacted]"));
    }

    #[tokio::test]
    async fn bulk_methods_reject_empty_ids() {
        // No MockServer: the guards must fail before any HTTP request fires.
        let sdk = make_sdk("http://127.0.0.1:1/".to_string());

        let err = sdk
            .admin
            .bulk_update_endpoint_status(&BulkUpdateEndpointStatusRequest {
                ids: vec![],
                status: "paused".to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Config(_)));

        let err = sdk
            .admin
            .bulk_add_tag(&BulkAddTagRequest {
                ids: vec![],
                label: "x".to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Config(_)));

        let err = sdk
            .admin
            .bulk_remove_tag(&BulkRemoveTagRequest {
                ids: vec![],
                tag_id: 1,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Config(_)));
    }

    #[tokio::test]
    async fn list_tags_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/endpoints/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "tags": [
                        { "id": 1, "label": "prod", "usage_count": 3 },
                        { "id": 2, "label": "staging", "usage_count": 0 }
                    ]
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.list_tags().await.unwrap();
        let data = resp.data.expect("data present");
        assert_eq!(data.tags.len(), 2);
        assert_eq!(data.tags[0].label, "prod");
        assert_eq!(data.tags[1].usage_count, 0);
    }

    #[tokio::test]
    async fn list_tags_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/endpoints/tags"))
            .respond_with(ResponseTemplate::new(500).set_body_string("oops"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.list_tags().await.unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 500),
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn rename_tag_success() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/endpoints/tags/7"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "id": 7, "label": "prod-v2", "usage_count": 3 },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = RenameTagRequest {
            label: "prod-v2".to_string(),
        };
        let resp = sdk.admin.rename_tag(7, &params).await.unwrap();
        let tag = resp.data.expect("tag present");
        assert_eq!(tag.id, 7);
        assert_eq!(tag.label, "prod-v2");
    }

    #[tokio::test]
    async fn delete_account_tag_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/endpoints/tags/7"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "success": true },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.delete_account_tag(7).await.unwrap();
        assert!(resp.data.expect("data").success);
    }

    #[tokio::test]
    async fn delete_account_tag_still_in_use() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/endpoints/tags/7"))
            .respond_with(ResponseTemplate::new(400).set_body_string("tag still in use"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.admin.delete_account_tag(7).await.unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 400),
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn get_usage_by_tag_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/usage/rpc/by-tag"))
            .and(query_param("start_time", "1700000000"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "tags": [
                        { "tag_id": 1, "label": "prod", "credits_used": 1234, "requests": 10 },
                        { "tag_id": null, "label": "untagged", "credits_used": 50, "requests": 2 }
                    ],
                    "start_time": 1700000000,
                    "end_time": 1700003600
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = GetUsageRequest {
            start_time: Some(1_700_000_000),
            ..Default::default()
        };
        let resp = sdk.admin.get_usage_by_tag(&params).await.unwrap();
        let data = resp.data.expect("data present");
        assert_eq!(data.tags.len(), 2);
        assert_eq!(data.tags[0].tag_id, Some(1));
        assert_eq!(data.tags[1].tag_id, None);
        assert_eq!(data.tags[1].label, "untagged");
    }

    #[tokio::test]
    async fn get_endpoint_security_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/endpoints/abc123/security"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "options": { "tokens": true, "ips": false },
                    "tokens": [{ "id": "tok_1", "token": "secret" }],
                    "jwts": [],
                    "referrers": [],
                    "domain_masks": [],
                    "ips": [],
                    "request_filters": []
                },
                "error": null
            })))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.admin.get_endpoint_security("abc123").await.unwrap();
        let data = resp.data.expect("data present");
        let tokens = data.tokens.expect("tokens present");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].id, "tok_1");
    }

    #[tokio::test]
    async fn get_endpoint_security_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/endpoints/missing/security"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;

        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .admin
            .get_endpoint_security("missing")
            .await
            .unwrap_err();
        match err {
            SdkError::Api { status, .. } => assert_eq!(status.as_u16(), 404),
            other => panic!("expected Api, got {:?}", other),
        }
    }

    #[test]
    fn negative_timeout_secs_returns_error() {
        use crate::{HttpConfig, SdkConfig, SdkFullConfig};
        let result = SdkConfig::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: Some(HttpConfig {
                timeout_secs: Some(-1),
                pool_max_idle_per_host: None,
                headers: None,
            }),
            admin: None,
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
        });
        assert!(matches!(result, Err(crate::errors::SdkError::Config(_))));
    }
}
