use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core as core;

// ── Top-level SDK ──────────────────────────────────────────────

#[napi]
pub struct QuickNodeSdk {
    admin: AdminApiClient,
    streams: StreamsApiClient,
}

#[napi]
impl QuickNodeSdk {
    #[napi(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(config: core::SdkFullConfig) -> Result<Self> {
        let sdk_config = core::SdkConfig::new(&config)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Self {
            admin: AdminApiClient {
                inner: core::admin::AdminApiClient::new(sdk_config.clone()),
            },
            streams: StreamsApiClient {
                inner: core::streams::StreamsApiClient::new(sdk_config),
            },
        })
    }

    #[napi(getter)]
    pub fn admin(&self) -> AdminApiClient {
        self.admin.clone()
    }

    #[napi(getter)]
    pub fn streams(&self) -> StreamsApiClient {
        self.streams.clone()
    }

    #[napi(factory)]
    pub fn from_env() -> Result<Self> {
        core::QuickNodeSdk::from_env()
            .map(|sdk| Self {
                admin: AdminApiClient { inner: sdk.admin },
                streams: StreamsApiClient { inner: sdk.streams },
            })
            .map_err(|e| Error::from_reason(e.to_string()))
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
    #[napi]
    pub async fn get_endpoints(
        &self,
        params: Option<core::admin::GetEndpointsRequest>,
    ) -> Result<core::admin::GetEndpointsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_endpoints(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_endpoint(
        &self,
        params: Option<core::admin::CreateEndpointRequest>,
    ) -> Result<core::admin::CreateEndpointResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .create_endpoint(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn show_endpoint(&self, id: String) -> Result<core::admin::ShowEndpointResponse> {
        self.inner
            .show_endpoint(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn archive_endpoint(&self, id: String) -> Result<()> {
        self.inner
            .archive_endpoint(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn update_endpoint_status(
        &self,
        id: String,
        params: core::admin::UpdateEndpointStatusRequest,
    ) -> Result<core::admin::UpdateEndpointStatusResponse> {
        self.inner
            .update_endpoint_status(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_tag(&self, id: String, tag_id: String) -> Result<()> {
        self.inner
            .delete_tag(&id, &tag_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_usage(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_usage_by_endpoint(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByEndpointResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_endpoint(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_usage_by_method(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByMethodResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_method(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_usage_by_chain(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByChainResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_chain(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_endpoint_logs(
        &self,
        id: String,
        params: core::admin::GetEndpointLogsRequest,
    ) -> Result<core::admin::GetEndpointLogsResponse> {
        self.inner
            .get_endpoint_logs(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_log_details(
        &self,
        id: String,
        request_id: String,
    ) -> Result<core::admin::GetLogDetailsResponse> {
        self.inner
            .get_log_details(&id, &request_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_security_options(
        &self,
        id: String,
    ) -> Result<core::admin::GetSecurityOptionsResponse> {
        self.inner
            .get_security_options(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn update_security_options(
        &self,
        id: String,
        params: core::admin::UpdateSecurityOptionsRequest,
    ) -> Result<core::admin::UpdateSecurityOptionsResponse> {
        self.inner
            .update_security_options(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_token(&self, id: String) -> Result<()> {
        self.inner
            .create_token(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_token(
        &self,
        id: String,
        token_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_token(&id, &token_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_referrer(
        &self,
        id: String,
        referrer_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_referrer(&id, &referrer_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_ip(
        &self,
        id: String,
        ip_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_ip(&id, &ip_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_domain_mask(
        &self,
        id: String,
        domain_mask_id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_domain_mask(&id, &domain_mask_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_jwt(&self, id: String, jwt_id: String) -> Result<()> {
        self.inner
            .delete_jwt(&id, &jwt_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_request_filter(
        &self,
        id: String,
        request_filter_id: String,
    ) -> Result<()> {
        self.inner
            .delete_request_filter(&id, &request_filter_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn enable_multichain(&self, id: String) -> Result<()> {
        self.inner
            .enable_multichain(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn disable_multichain(&self, id: String) -> Result<()> {
        self.inner
            .disable_multichain(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_or_update_ip_custom_header(
        &self,
        id: String,
        params: core::admin::CreateOrUpdateIpCustomHeaderRequest,
    ) -> Result<core::admin::CreateOrUpdateIpCustomHeaderResponse> {
        self.inner
            .create_or_update_ip_custom_header(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_ip_custom_header(
        &self,
        id: String,
    ) -> Result<core::admin::DeleteBoolResponse> {
        self.inner
            .delete_ip_custom_header(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_method_rate_limits(
        &self,
        id: String,
    ) -> Result<core::admin::GetMethodRateLimitsResponse> {
        self.inner
            .get_method_rate_limits(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_method_rate_limit(
        &self,
        id: String,
        params: core::admin::CreateMethodRateLimitRequest,
    ) -> Result<core::admin::CreateMethodRateLimitResponse> {
        self.inner
            .create_method_rate_limit(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
    ) -> Result<()> {
        self.inner
            .delete_method_rate_limit(&id, &method_rate_limit_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn update_rate_limits(
        &self,
        id: String,
        params: core::admin::UpdateRateLimitsRequest,
    ) -> Result<()> {
        self.inner
            .update_rate_limits(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_endpoint_metrics(
        &self,
        id: String,
        params: core::admin::GetEndpointMetricsRequest,
    ) -> Result<core::admin::GetEndpointMetricsResponse> {
        self.inner
            .get_endpoint_metrics(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_account_metrics(
        &self,
        params: core::admin::GetAccountMetricsRequest,
    ) -> Result<core::admin::GetAccountMetricsResponse> {
        self.inner
            .get_account_metrics(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_chains(&self) -> Result<core::admin::ListChainsResponse> {
        self.inner
            .list_chains()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_invoices(&self) -> Result<core::admin::ListInvoicesResponse> {
        self.inner
            .list_invoices()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_payments(&self) -> Result<core::admin::ListPaymentsResponse> {
        self.inner
            .list_payments()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_teams(&self) -> Result<core::admin::ListTeamsResponse> {
        self.inner
            .list_teams()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_team(
        &self,
        params: core::admin::CreateTeamRequest,
    ) -> Result<core::admin::CreateTeamResponse> {
        self.inner
            .create_team(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_team(&self, id: i64) -> Result<core::admin::GetTeamResponse> {
        self.inner
            .get_team(id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_team(&self, id: i64) -> Result<core::admin::DeleteTeamResponse> {
        self.inner
            .delete_team(id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_team_endpoints(
        &self,
        id: i64,
    ) -> Result<core::admin::ListTeamEndpointsResponse> {
        self.inner
            .list_team_endpoints(id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn update_team_endpoints(
        &self,
        id: i64,
        params: core::admin::UpdateTeamEndpointsRequest,
    ) -> Result<core::admin::UpdateTeamEndpointsResponse> {
        self.inner
            .update_team_endpoints(id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn invite_team_member(
        &self,
        id: i64,
        params: core::admin::InviteTeamMemberRequest,
    ) -> Result<core::admin::InviteTeamMemberResponse> {
        self.inner
            .invite_team_member(id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn resend_team_invite(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<core::admin::ResendTeamInviteResponse> {
        self.inner
            .resend_team_invite(id, user_id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
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
    #[napi]
    pub async fn create_stream(
        &self,
        params: core::streams::CreateStreamParams,
    ) -> Result<core::streams::Stream> {
        self.inner
            .create_stream(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
