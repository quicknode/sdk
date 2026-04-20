use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core as core;

mod streams_destination;

// ── Top-level SDK ──────────────────────────────────────────────

#[napi]
pub struct QuickNodeSdk {
    admin: AdminApiClient,
    streams: StreamsApiClient,
    webhooks: WebhooksApiClient,
    kvstore: KvStoreApiClient,
}

#[napi]
impl QuickNodeSdk {
    #[napi(constructor)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(config: core::SdkFullConfig) -> Result<Self> {
        let sdk_config =
            core::SdkConfig::new(&config).map_err(|e| Error::from_reason(e.to_string()))?;
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
                inner: core::kvstore::KvStoreApiClient::new(sdk_config),
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

    #[napi(getter)]
    pub fn webhooks(&self) -> WebhooksApiClient {
        self.webhooks.clone()
    }

    #[napi(getter)]
    pub fn kvstore(&self) -> KvStoreApiClient {
        self.kvstore.clone()
    }

    #[napi(factory)]
    pub fn from_env() -> Result<Self> {
        core::QuickNodeSdk::from_env()
            .map(|sdk| Self {
                admin: AdminApiClient { inner: sdk.admin },
                streams: StreamsApiClient { inner: sdk.streams },
                webhooks: WebhooksApiClient {
                    inner: sdk.webhooks,
                },
                kvstore: KvStoreApiClient { inner: sdk.kvstore },
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
    pub async fn delete_request_filter(&self, id: String, request_filter_id: String) -> Result<()> {
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

    #[napi]
    pub async fn bulk_update_endpoint_status(
        &self,
        params: core::admin::BulkUpdateEndpointStatusRequest,
    ) -> Result<core::admin::BulkUpdateEndpointStatusResponse> {
        self.inner
            .bulk_update_endpoint_status(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn bulk_add_tag(
        &self,
        params: core::admin::BulkAddTagRequest,
    ) -> Result<core::admin::BulkAddTagResponse> {
        self.inner
            .bulk_add_tag(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn bulk_remove_tag(
        &self,
        params: core::admin::BulkRemoveTagRequest,
    ) -> Result<core::admin::BulkRemoveTagResponse> {
        self.inner
            .bulk_remove_tag(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_tags(&self) -> Result<core::admin::ListTagsResponse> {
        self.inner
            .list_tags()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn rename_tag(
        &self,
        id: i32,
        params: core::admin::RenameTagRequest,
    ) -> Result<core::admin::RenameTagResponse> {
        self.inner
            .rename_tag(id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_account_tag(
        &self,
        id: i32,
    ) -> Result<core::admin::DeleteAccountTagResponse> {
        self.inner
            .delete_account_tag(id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_usage_by_tag(
        &self,
        params: Option<core::admin::GetUsageRequest>,
    ) -> Result<core::admin::GetUsageByTagResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_usage_by_tag(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_endpoint_security(
        &self,
        id: String,
    ) -> Result<core::admin::GetEndpointSecurityResponse> {
        self.inner
            .get_endpoint_security(&id)
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
        params: streams_destination::CreateStreamParamsNode,
    ) -> Result<streams_destination::StreamNode> {
        let core_params = params.into_core()?;
        let stream = self
            .inner
            .create_stream(&core_params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        streams_destination::StreamNode::from_core(stream)
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))?;
        streams_destination::ListStreamsResponseNode::from_core(resp)
    }

    #[napi]
    pub async fn delete_all_streams(&self) -> Result<()> {
        self.inner
            .delete_all_streams()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_stream(&self, id: String) -> Result<streams_destination::StreamNode> {
        let stream = self
            .inner
            .get_stream(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))?;
        streams_destination::StreamNode::from_core(stream)
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))?;
        streams_destination::StreamNode::from_core(stream)
    }

    #[napi]
    pub async fn delete_stream(&self, id: String) -> Result<()> {
        self.inner
            .delete_stream(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn activate_stream(&self, id: String) -> Result<()> {
        self.inner
            .activate_stream(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn pause_stream(&self, id: String) -> Result<()> {
        self.inner
            .pause_stream(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn test_filter(
        &self,
        params: core::streams::TestFilterParams,
    ) -> Result<core::streams::TestFilterResponse> {
        self.inner
            .test_filter(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_enabled_count(
        &self,
        stream_type: Option<String>,
    ) -> Result<core::streams::EnabledCountResponse> {
        self.inner
            .get_enabled_count(stream_type.as_deref())
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
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
    #[napi]
    pub async fn list_webhooks(
        &self,
        params: Option<core::webhooks::GetWebhooksParams>,
    ) -> Result<core::webhooks::ListWebhooksResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .list_webhooks(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_all_webhooks(&self) -> Result<()> {
        self.inner
            .delete_all_webhooks()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_webhook(&self, id: String) -> Result<core::webhooks::Webhook> {
        self.inner
            .get_webhook(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_webhook(&self, id: String) -> Result<()> {
        self.inner
            .delete_webhook(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn pause_webhook(&self, id: String) -> Result<()> {
        self.inner
            .pause_webhook(&id)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn activate_webhook(
        &self,
        id: String,
        params: core::webhooks::ActivateWebhookParams,
    ) -> Result<()> {
        self.inner
            .activate_webhook(&id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_enabled_count(&self) -> Result<core::webhooks::WebhookEnabledCountResponse> {
        self.inner
            .get_enabled_count()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_webhook_from_template(
        &self,
        params: core::webhooks::CreateWebhookFromTemplateParams,
    ) -> Result<core::webhooks::Webhook> {
        self.inner
            .create_webhook_from_template(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn update_webhook_template(
        &self,
        webhook_id: String,
        params: core::webhooks::UpdateWebhookTemplateParams,
    ) -> Result<core::webhooks::Webhook> {
        self.inner
            .update_webhook_template(&webhook_id, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
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
    #[napi]
    pub async fn create_set(&self, params: core::kvstore::CreateSetParams) -> Result<()> {
        self.inner
            .create_set(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_sets(
        &self,
        params: Option<core::kvstore::GetSetsParams>,
    ) -> Result<core::kvstore::GetSetsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_sets(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_set(&self, key: String) -> Result<core::kvstore::GetSetResponse> {
        self.inner
            .get_set(&key)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn bulk_sets(&self, params: core::kvstore::BulkSetsParams) -> Result<()> {
        self.inner
            .bulk_sets(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_set(&self, key: String) -> Result<()> {
        self.inner
            .delete_set(&key)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn create_list(&self, params: core::kvstore::CreateListParams) -> Result<()> {
        self.inner
            .create_list(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn get_lists(
        &self,
        params: Option<core::kvstore::GetListsParams>,
    ) -> Result<core::kvstore::GetListsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_lists(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

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
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn update_list(
        &self,
        key: String,
        params: core::kvstore::UpdateListParams,
    ) -> Result<()> {
        self.inner
            .update_list(&key, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn add_list_item(
        &self,
        key: String,
        params: core::kvstore::AddListItemParams,
    ) -> Result<()> {
        self.inner
            .add_list_item(&key, &params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn list_contains_item(
        &self,
        key: String,
        item: String,
    ) -> Result<core::kvstore::ListContainsItemResponse> {
        self.inner
            .list_contains_item(&key, &item)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_list_item(&self, key: String, item: String) -> Result<()> {
        self.inner
            .delete_list_item(&key, &item)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn delete_list(&self, key: String) -> Result<()> {
        self.inner
            .delete_list(&key)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
