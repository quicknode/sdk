pub mod billing;
pub mod chains;
pub mod endpoint_metrics;
pub mod endpoint_rate_limits;
pub mod endpoint_security;
pub mod endpoints;
pub mod logs;
pub mod teams;
pub mod usage;

pub use billing::{
    Invoice, InvoiceLine, ListInvoicesData, ListInvoicesResponse, ListPaymentsData,
    ListPaymentsResponse, Payment,
};
pub use chains::{Chain, ChainNetwork, ListChainsResponse};
pub use endpoint_metrics::{
    EndpointMetric, GetAccountMetricsRequest, GetAccountMetricsResponse, GetEndpointMetricsRequest,
    GetEndpointMetricsResponse,
};
pub use endpoint_rate_limits::{
    CreateMethodRateLimitRequest, CreateMethodRateLimitResponse, GetMethodRateLimitsData,
    GetMethodRateLimitsResponse, MethodRateLimiter, RateLimitSettings,
    UpdateMethodRateLimitRequest, UpdateMethodRateLimitResponse, UpdateRateLimitsRequest,
};
pub use endpoint_security::{
    CreateDomainMaskRequest, CreateIpRequest, CreateJwtRequest,
    CreateOrUpdateIpCustomHeaderRequest, CreateOrUpdateIpCustomHeaderResponse,
    CreateReferrerRequest, CreateRequestFilterData, CreateRequestFilterRequest,
    CreateRequestFilterResponse, DeleteBoolResponse, GetSecurityOptionsResponse,
    IpCustomHeaderData, SecurityOption, SecurityOptionsUpdate, UpdateRequestFilterRequest,
    UpdateSecurityOptionsRequest, UpdateSecurityOptionsResponse,
};
pub use endpoints::{
    CreateEndpointRequest, CreateEndpointResponse, CreateTagRequest, Endpoint, EndpointDomainMask,
    EndpointIp, EndpointIpCustomHeaderOption, EndpointJwt, EndpointRateLimits, EndpointReferrer,
    EndpointRequestFilter, EndpointSecurity, EndpointSecurityOptions, EndpointTag, EndpointToken,
    GetEndpointsRequest, GetEndpointsResponse, ShowEndpointResponse, SingleEndpoint,
    UpdateEndpointRequest, UpdateEndpointStatusRequest, UpdateEndpointStatusResponse,
};
pub use logs::{
    EndpointLog, GetEndpointLogsRequest, GetEndpointLogsResponse, GetLogDetailsResponse, LogDetails,
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
    GetUsageByMethodResponse, GetUsageRequest, GetUsageResponse, MethodUsage, UsageByChainData,
    UsageByEndpointData, UsageByMethodData, UsageData,
};

use crate::{
    config::AdminConfig,
    errors::SdkError,
    SdkConfig,
};

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

#[derive(Debug, Clone)]
pub struct AdminApiClient {
    config: SdkConfig,
}

impl AdminApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    pub async fn get_endpoints(
        &self,
        params: &GetEndpointsRequest,
    ) -> Result<GetEndpointsResponse, SdkError> {
        let url = self.config.admin().base_url.join("endpoints")?;
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

    pub async fn show_endpoint(&self, id: &str) -> Result<ShowEndpointResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn update_endpoint(
        &self,
        id: &str,
        params: &UpdateEndpointRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn archive_endpoint(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn update_endpoint_status(
        &self,
        id: &str,
        params: &UpdateEndpointStatusRequest,
    ) -> Result<UpdateEndpointStatusResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_tag(&self, id: &str, params: &CreateTagRequest) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn delete_tag(&self, id: &str, tag_id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn get_team(&self, id: i64) -> Result<GetTeamResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn delete_team(&self, id: i64) -> Result<DeleteTeamResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn list_team_endpoints(
        &self,
        id: i64,
    ) -> Result<ListTeamEndpointsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn update_team_endpoints(
        &self,
        id: i64,
        params: &UpdateTeamEndpointsRequest,
    ) -> Result<UpdateTeamEndpointsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn invite_team_member(
        &self,
        id: i64,
        params: &InviteTeamMemberRequest,
    ) -> Result<InviteTeamMemberResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn remove_team_member(
        &self,
        id: i64,
        user_id: i64,
        params: &RemoveTeamMemberRequest,
    ) -> Result<RemoveTeamMemberResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn resend_team_invite(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<ResendTeamInviteResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn get_endpoint_logs(
        &self,
        id: &str,
        params: &GetEndpointLogsRequest,
    ) -> Result<GetEndpointLogsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn get_log_details(
        &self,
        id: &str,
        request_id: &str,
    ) -> Result<GetLogDetailsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn get_security_options(
        &self,
        id: &str,
    ) -> Result<GetSecurityOptionsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn update_security_options(
        &self,
        id: &str,
        params: &UpdateSecurityOptionsRequest,
    ) -> Result<UpdateSecurityOptionsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_token(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn delete_token(
        &self,
        id: &str,
        token_id: &str,
    ) -> Result<DeleteBoolResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_referrer(
        &self,
        id: &str,
        params: &CreateReferrerRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_ip(&self, id: &str, params: &CreateIpRequest) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn delete_ip(&self, id: &str, ip_id: &str) -> Result<DeleteBoolResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_domain_mask(
        &self,
        id: &str,
        params: &CreateDomainMaskRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_jwt(&self, id: &str, params: &CreateJwtRequest) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn delete_jwt(&self, id: &str, jwt_id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_request_filter(
        &self,
        id: &str,
        params: &CreateRequestFilterRequest,
    ) -> Result<CreateRequestFilterResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn enable_multichain(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn disable_multichain(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_or_update_ip_custom_header(
        &self,
        id: &str,
        params: &CreateOrUpdateIpCustomHeaderRequest,
    ) -> Result<CreateOrUpdateIpCustomHeaderResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn delete_ip_custom_header(&self, id: &str) -> Result<DeleteBoolResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn get_method_rate_limits(
        &self,
        id: &str,
    ) -> Result<GetMethodRateLimitsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn create_method_rate_limit(
        &self,
        id: &str,
        params: &CreateMethodRateLimitRequest,
    ) -> Result<CreateMethodRateLimitResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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

    pub async fn update_rate_limits(
        &self,
        id: &str,
        params: &UpdateRateLimitsRequest,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .admin().base_url
            .join(&format!("endpoints/{}/rate-limits", id))?;
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

    pub async fn get_endpoint_metrics(
        &self,
        id: &str,
        params: &GetEndpointMetricsRequest,
    ) -> Result<GetEndpointMetricsResponse, SdkError> {
        let url = self
            .config
            .admin().base_url
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{AdminConfig, QuickNodeSdk, SdkFullConfig};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuickNodeSdk {
        QuickNodeSdk::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: None,
            admin: Some(AdminConfig {
                base_url: Some(base_url),
            }),
            streams: None,
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
                        "label": "My Endpoint",
                        "chain": "ethereum",
                        "network": "mainnet",
                        "http_url": "https://example.quicknode.pro/abc123",
                        "wss_url": null,
                        "tags": []
                    }
                ],
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
        assert_eq!(resp.data[0].chain, "ethereum");
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
                    referrer: Some("example.com".to_string()),
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

        Mock::given(method("PUT"))
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
    async fn get_endpoint_metrics_success() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/endpoints/ep123/metrics"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"data": [[1700000000, 42]], "tag": "mainnet"}],
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
        assert_eq!(resp.data[0].tag, "mainnet");
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

    #[test]
    fn negative_timeout_secs_returns_error() {
        use crate::{HttpConfig, SdkConfig, SdkFullConfig};
        let result = SdkConfig::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: Some(HttpConfig {
                timeout_secs: Some(-1),
                pool_max_idle_per_host: None,
            }),
            admin: None,
            streams: None,
        });
        assert!(matches!(result, Err(crate::errors::SdkError::Config(_))));
    }
}
