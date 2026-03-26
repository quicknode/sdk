#![allow(clippy::expect_used)]
use magnus::{function, method, prelude::*, Error, RHash, Ruby};
use sdk_core as core;

// ── Tokio runtime ───────────────────────────────────────────────────────────
//
// A single shared runtime for all blocking HTTP calls. GVL is held during
// network I/O for now; when magnus adds a without_gvl API this can be updated
// to release it. Can also look at lucchetto

static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("tokio runtime init failed"))
}

fn ruby() -> Ruby {
    Ruby::get().expect("called outside of a Ruby thread")
}

#[allow(clippy::needless_pass_by_value)]
fn map_err(e: core::errors::SdkError) -> Error {
    Error::new(ruby().exception_runtime_error(), e.to_string())
}

#[allow(clippy::needless_pass_by_value)]
fn parse_err(e: serde_json::Error) -> Error {
    Error::new(ruby().exception_arg_error(), e.to_string())
}

fn to_json<T: serde::Serialize>(v: T) -> Result<String, Error> {
    serde_json::to_string(&v).map_err(parse_err)
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: String) -> Result<T, Error> {
    serde_json::from_value(serde_json::Value::String(s)).map_err(parse_err)
}

fn parse_enum_opt<T: serde::de::DeserializeOwned>(s: Option<String>) -> Result<Option<T>, Error> {
    s.map(parse_enum).transpose()
}

fn hash_get_string(h: &RHash, key: &str) -> Result<Option<String>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => String::try_convert(v)
            .map(Some)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}"))),
        _ => Ok(None),
    }
}

fn hash_require_string(h: &RHash, key: &str) -> Result<String, Error> {
    hash_get_string(h, key)?.ok_or_else(|| {
        Error::new(
            ruby().exception_arg_error(),
            format!("missing required key: {key}"),
        )
    })
}

fn hash_get_i64(h: &RHash, key: &str) -> Result<Option<i64>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => i64::try_convert(v)
            .map(Some)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}"))),
        _ => Ok(None),
    }
}

fn hash_require_i64(h: &RHash, key: &str) -> Result<i64, Error> {
    hash_get_i64(h, key)?.ok_or_else(|| {
        Error::new(
            ruby().exception_arg_error(),
            format!("missing required key: {key}"),
        )
    })
}

fn hash_get_i32(h: &RHash, key: &str) -> Result<Option<i32>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => i32::try_convert(v)
            .map(Some)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}"))),
        _ => Ok(None),
    }
}

fn hash_get_bool(h: &RHash, key: &str) -> Result<Option<bool>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => bool::try_convert(v)
            .map(Some)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}"))),
        _ => Ok(None),
    }
}

fn hash_get_dest_attrs(
    h: &RHash,
    key: &str,
) -> Result<Option<core::streams::DestinationAttributes>, Error> {
    match h.get(ruby().to_symbol(key)) {
        Some(v) if !v.is_nil() => {
            let wrapped: &DestinationAttributes = magnus::TryConvert::try_convert(v)?;
            Ok(Some(wrapped.inner.clone()))
        }
        _ => Ok(None),
    }
}

// ── QuickNodeSdk ────────────────────────────────────────────────────────────

#[magnus::wrap(class = "QuickNodeSdk::SDK", free_immediately, size)]
pub struct QuickNodeSdk {
    inner: core::QuickNodeSdk,
}

impl QuickNodeSdk {
    fn from_env() -> Result<Self, Error> {
        core::QuickNodeSdk::from_env()
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn admin(&self) -> AdminApiClient {
        AdminApiClient {
            inner: self.inner.admin.clone(),
        }
    }

    fn streams(&self) -> StreamsApiClient {
        StreamsApiClient {
            inner: self.inner.streams.clone(),
        }
    }

    fn webhooks(&self) -> WebhooksApiClient {
        WebhooksApiClient {
            inner: self.inner.webhooks.clone(),
        }
    }

    fn kvstore(&self) -> KvStoreApiClient {
        KvStoreApiClient {
            inner: self.inner.kvstore.clone(),
        }
    }
}

// ── AdminApiClient ──────────────────────────────────────────────────────────
//
// All methods return JSON strings. Call JSON.parse on the result in Ruby.

#[magnus::wrap(class = "QuickNodeSdk::Admin", free_immediately, size)]
#[derive(Clone)]
pub struct AdminApiClient {
    inner: core::admin::AdminApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl AdminApiClient {
    fn get_endpoints(
        &self,
        limit: Option<i32>,
        offset: Option<i32>,
        tag_ids: Option<Vec<i32>>,
        tag_labels: Option<Vec<String>>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointsRequest {
            limit,
            offset,
            tag_ids,
            tag_labels,
        };
        runtime()
            .block_on(client.get_endpoints(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_endpoint(
        &self,
        chain: Option<String>,
        network: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateEndpointRequest { chain, network };
        runtime()
            .block_on(client.create_endpoint(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn show_endpoint(&self, id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.show_endpoint(&id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_endpoint(&self, id: String, label: Option<String>) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateEndpointRequest { label };
        runtime()
            .block_on(client.update_endpoint(&id, &params))
            .map_err(map_err)
    }

    fn archive_endpoint(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.archive_endpoint(&id))
            .map_err(map_err)
    }

    fn update_endpoint_status(&self, id: String, status: String) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateEndpointStatusRequest { status };
        runtime()
            .block_on(client.update_endpoint_status(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_tag(&self, id: String, label: Option<String>) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateTagRequest { label };
        runtime()
            .block_on(client.create_tag(&id, &params))
            .map_err(map_err)
    }

    fn delete_tag(&self, id: String, tag_id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_tag(&id, &tag_id))
            .map_err(map_err)
    }

    fn get_usage(&self, start_time: Option<i64>, end_time: Option<i64>) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
        runtime()
            .block_on(client.get_usage(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_usage_by_endpoint(
        &self,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
        runtime()
            .block_on(client.get_usage_by_endpoint(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_usage_by_method(
        &self,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
        runtime()
            .block_on(client.get_usage_by_method(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_usage_by_chain(
        &self,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
        runtime()
            .block_on(client.get_usage_by_chain(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    #[allow(clippy::too_many_arguments)]
    fn get_endpoint_logs(
        &self,
        id: String,
        from_time: String,
        to_time: String,
        include_details: Option<bool>,
        limit: Option<i32>,
        next_at: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointLogsRequest {
            from: from_time,
            to: to_time,
            include_details,
            limit,
            next_at,
        };
        runtime()
            .block_on(client.get_endpoint_logs(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_log_details(&self, id: String, request_id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_log_details(&id, &request_id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_security_options(&self, id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_security_options(&id))
            .map_err(map_err)
            .and_then(to_json)
    }

    #[allow(clippy::too_many_arguments)]
    fn update_security_options(
        &self,
        id: String,
        tokens: Option<String>,
        referrers: Option<String>,
        jwts: Option<String>,
        ips: Option<String>,
        domain_masks: Option<String>,
        hsts: Option<String>,
        cors: Option<String>,
        request_filters: Option<String>,
        ip_custom_header: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateSecurityOptionsRequest {
            options: core::admin::SecurityOptionsUpdate {
                tokens,
                referrers,
                jwts,
                ips,
                domain_masks,
                hsts,
                cors,
                request_filters,
                ip_custom_header,
            },
        };
        runtime()
            .block_on(client.update_security_options(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_token(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.create_token(&id))
            .map_err(map_err)
    }

    fn delete_token(&self, id: String, token_id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_token(&id, &token_id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_referrer(&self, id: String, referrer: Option<String>) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateReferrerRequest { referrer };
        runtime()
            .block_on(client.create_referrer(&id, &params))
            .map_err(map_err)
    }

    fn delete_referrer(&self, id: String, referrer_id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_referrer(&id, &referrer_id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_ip(&self, id: String, ip: Option<String>) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateIpRequest { ip };
        runtime()
            .block_on(client.create_ip(&id, &params))
            .map_err(map_err)
    }

    fn delete_ip(&self, id: String, ip_id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_ip(&id, &ip_id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_domain_mask(&self, id: String, domain_mask: Option<String>) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateDomainMaskRequest { domain_mask };
        runtime()
            .block_on(client.create_domain_mask(&id, &params))
            .map_err(map_err)
    }

    fn delete_domain_mask(&self, id: String, domain_mask_id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_domain_mask(&id, &domain_mask_id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_jwt(
        &self,
        id: String,
        public_key: Option<String>,
        kid: Option<String>,
        name: Option<String>,
    ) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateJwtRequest {
            public_key,
            kid,
            name,
        };
        runtime()
            .block_on(client.create_jwt(&id, &params))
            .map_err(map_err)
    }

    fn delete_jwt(&self, id: String, jwt_id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_jwt(&id, &jwt_id))
            .map_err(map_err)
    }

    fn create_request_filter(
        &self,
        id: String,
        methods: Option<Vec<String>>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateRequestFilterRequest { method: methods };
        runtime()
            .block_on(client.create_request_filter(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_request_filter(
        &self,
        id: String,
        request_filter_id: String,
        methods: Option<Vec<String>>,
    ) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateRequestFilterRequest { method: methods };
        runtime()
            .block_on(client.update_request_filter(&id, &request_filter_id, &params))
            .map_err(map_err)
    }

    fn delete_request_filter(&self, id: String, request_filter_id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_request_filter(&id, &request_filter_id))
            .map_err(map_err)
    }

    fn enable_multichain(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.enable_multichain(&id))
            .map_err(map_err)
    }

    fn disable_multichain(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.disable_multichain(&id))
            .map_err(map_err)
    }

    fn create_or_update_ip_custom_header(
        &self,
        id: String,
        header_name: String,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateOrUpdateIpCustomHeaderRequest { header_name };
        runtime()
            .block_on(client.create_or_update_ip_custom_header(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_ip_custom_header(&self, id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_ip_custom_header(&id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_method_rate_limits(&self, id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_method_rate_limits(&id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_method_rate_limit(
        &self,
        id: String,
        interval: String,
        methods: Vec<String>,
        rate: i32,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateMethodRateLimitRequest {
            interval,
            methods,
            rate,
        };
        runtime()
            .block_on(client.create_method_rate_limit(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
        methods: Option<Vec<String>>,
        status: Option<String>,
        rate: Option<i32>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateMethodRateLimitRequest {
            methods,
            status,
            rate,
        };
        runtime()
            .block_on(client.update_method_rate_limit(&id, &method_rate_limit_id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_method_rate_limit(
        &self,
        id: String,
        method_rate_limit_id: String,
    ) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_method_rate_limit(&id, &method_rate_limit_id))
            .map_err(map_err)
    }

    fn update_rate_limits(
        &self,
        id: String,
        rps: Option<i32>,
        rpm: Option<i32>,
        rpd: Option<i32>,
    ) -> Result<(), Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateRateLimitsRequest {
            rate_limits: core::admin::RateLimitSettings { rps, rpm, rpd },
        };
        runtime()
            .block_on(client.update_rate_limits(&id, &params))
            .map_err(map_err)
    }

    fn get_endpoint_metrics(
        &self,
        id: String,
        period: String,
        metric: String,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointMetricsRequest { period, metric };
        runtime()
            .block_on(client.get_endpoint_metrics(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_account_metrics(
        &self,
        period: String,
        metric: String,
        percentile: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::GetAccountMetricsRequest {
            period,
            metric,
            percentile,
        };
        runtime()
            .block_on(client.get_account_metrics(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn list_chains(&self) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_chains())
            .map_err(map_err)
            .and_then(to_json)
    }

    fn list_invoices(&self) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_invoices())
            .map_err(map_err)
            .and_then(to_json)
    }

    fn list_payments(&self) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_payments())
            .map_err(map_err)
            .and_then(to_json)
    }

    fn list_teams(&self) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_teams())
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_team(&self, name: String) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::CreateTeamRequest { name };
        runtime()
            .block_on(client.create_team(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_team(&self, id: i64) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_team(id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_team(&self, id: i64) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_team(id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn list_team_endpoints(&self, id: i64) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_team_endpoints(id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_team_endpoints(&self, id: i64, endpoint_ids: Vec<String>) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::UpdateTeamEndpointsRequest { endpoint_ids };
        runtime()
            .block_on(client.update_team_endpoints(id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn invite_team_member(
        &self,
        id: i64,
        email: String,
        full_name: Option<String>,
        role: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::InviteTeamMemberRequest {
            email,
            full_name,
            role,
        };
        runtime()
            .block_on(client.invite_team_member(id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn remove_team_member(
        &self,
        id: i64,
        user_id: i64,
        destroy_user: Option<bool>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::admin::RemoveTeamMemberRequest { destroy_user };
        runtime()
            .block_on(client.remove_team_member(id, user_id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn resend_team_invite(&self, id: i64, user_id: i64) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.resend_team_invite(id, user_id))
            .map_err(map_err)
            .and_then(to_json)
    }
}

// ── DestinationAttributes ───────────────────────────────────────────────────

#[magnus::wrap(class = "QuickNodeSdk::DestinationAttributes", free_immediately, size)]
pub struct DestinationAttributes {
    pub inner: core::streams::DestinationAttributes,
}

#[allow(clippy::needless_pass_by_value)]
impl DestinationAttributes {
    fn webhook(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::WebhookAttributes {
            url: hash_require_string(&opts, "url")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            post_timeout_sec: hash_get_i32(&opts, "post_timeout_sec")?.unwrap_or(0),
            security_token: hash_get_string(&opts, "security_token")?,
            compression: hash_require_string(&opts, "compression")?,
        };
        core::streams::DestinationAttributes::webhook(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn s3(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::S3Attributes {
            endpoint: hash_require_string(&opts, "endpoint")?,
            access_key: hash_require_string(&opts, "access_key")?,
            secret_key: hash_require_string(&opts, "secret_key")?,
            bucket: hash_require_string(&opts, "bucket")?,
            object_prefix: hash_require_string(&opts, "object_prefix")?,
            compression: hash_require_string(&opts, "compression")?,
            file_type: hash_require_string(&opts, "file_type")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            use_ssl: hash_get_bool(&opts, "use_ssl")?,
        };
        core::streams::DestinationAttributes::s3(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn azure(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::AzureAttributes {
            storage_account: hash_require_string(&opts, "storage_account")?,
            sas_token: hash_require_string(&opts, "sas_token")?,
            container: hash_require_string(&opts, "container")?,
            compression: hash_require_string(&opts, "compression")?,
            file_type: hash_require_string(&opts, "file_type")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            blob_prefix: hash_get_string(&opts, "blob_prefix")?,
        };
        core::streams::DestinationAttributes::azure(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn postgres(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::PostgresAttributes {
            host: hash_require_string(&opts, "host")?,
            port: hash_get_i32(&opts, "port")?.unwrap_or(5432),
            database: hash_require_string(&opts, "database")?,
            username: hash_require_string(&opts, "username")?,
            password: hash_require_string(&opts, "password")?,
            table_name: hash_require_string(&opts, "table_name")?,
            sslmode: hash_require_string(&opts, "sslmode")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
        };
        core::streams::DestinationAttributes::postgres(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn mysql(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::MysqlAttributes {
            host: hash_require_string(&opts, "host")?,
            port: hash_get_i32(&opts, "port")?.unwrap_or(3306),
            database: hash_require_string(&opts, "database")?,
            username: hash_require_string(&opts, "username")?,
            password: hash_require_string(&opts, "password")?,
            table_name: hash_require_string(&opts, "table_name")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
        };
        core::streams::DestinationAttributes::mysql(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn mongo(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::MongoAttributes {
            host: hash_require_string(&opts, "host")?,
            database: hash_require_string(&opts, "database")?,
            username: hash_require_string(&opts, "username")?,
            password: hash_require_string(&opts, "password")?,
            collection_name: hash_require_string(&opts, "collection_name")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
        };
        core::streams::DestinationAttributes::mongo(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn clickhouse(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::ClickhouseAttributes {
            hosts: hash_require_string(&opts, "hosts")?,
            database: hash_require_string(&opts, "database")?,
            username: hash_require_string(&opts, "username")?,
            password: hash_require_string(&opts, "password")?,
            table_name: hash_require_string(&opts, "table_name")?,
            default_table_engine_opts: hash_require_string(&opts, "default_table_engine_opts")?,
            default_granularity: hash_get_i32(&opts, "default_granularity")?.unwrap_or(0),
            default_compression: hash_require_string(&opts, "default_compression")?,
            default_index_type: hash_require_string(&opts, "default_index_type")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            disable_datetime_precision: hash_get_bool(&opts, "disable_datetime_precision")?,
            dont_support_rename_column: hash_get_bool(&opts, "dont_support_rename_column")?,
            dont_support_empty_default_value: hash_get_bool(
                &opts,
                "dont_support_empty_default_value",
            )?,
            skip_initialize_with_version: hash_get_bool(&opts, "skip_initialize_with_version")?,
        };
        core::streams::DestinationAttributes::clickhouse(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn snowflake(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::SnowflakeAttributes {
            account: hash_require_string(&opts, "account")?,
            host: hash_require_string(&opts, "host")?,
            port: hash_get_i32(&opts, "port")?.unwrap_or(443),
            protocol: hash_require_string(&opts, "protocol")?,
            database: hash_require_string(&opts, "database")?,
            schema: hash_require_string(&opts, "schema")?,
            warehouse: hash_require_string(&opts, "warehouse")?,
            username: hash_require_string(&opts, "username")?,
            password: hash_require_string(&opts, "password")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            table_name: hash_get_string(&opts, "table_name")?,
        };
        core::streams::DestinationAttributes::snowflake(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn kafka(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::KafkaAttributes {
            bootstrap_servers: hash_require_string(&opts, "bootstrap_servers")?,
            topic_name: hash_require_string(&opts, "topic_name")?,
            compression_type: hash_require_string(&opts, "compression_type")?,
            batch_size: hash_get_i32(&opts, "batch_size")?.unwrap_or(0),
            linger_ms: hash_get_i32(&opts, "linger_ms")?.unwrap_or(0),
            max_request_size: hash_get_i32(&opts, "max_request_size")?.unwrap_or(0),
            timeout_sec: hash_get_i32(&opts, "timeout_sec")?.unwrap_or(0),
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            username: hash_get_string(&opts, "username")?,
            password: hash_get_string(&opts, "password")?,
            protocol: hash_get_string(&opts, "protocol")?,
            mechanisms: hash_get_string(&opts, "mechanisms")?,
        };
        core::streams::DestinationAttributes::kafka(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }

    fn redis(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::RedisAttributes {
            host: hash_require_string(&opts, "host")?,
            port: hash_get_i32(&opts, "port")?.unwrap_or(6379),
            database: hash_get_i32(&opts, "database")?.unwrap_or(0),
            username: hash_require_string(&opts, "username")?,
            password: hash_require_string(&opts, "password")?,
            key_name: hash_require_string(&opts, "key_name")?,
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            tls: hash_get_bool(&opts, "tls")?,
        };
        core::streams::DestinationAttributes::redis(&attrs)
            .map(|inner| Self { inner })
            .map_err(map_err)
    }
}

// ── StreamsApiClient ────────────────────────────────────────────────────────

#[magnus::wrap(class = "QuickNodeSdk::Streams", free_immediately, size)]
#[derive(Clone)]
pub struct StreamsApiClient {
    inner: core::streams::StreamsApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl StreamsApiClient {
    // create_stream accepts a Ruby Hash because the param count exceeds magnus arity limit of 15.
    // Required keys: name, network, dataset, region, start_range, end_range,
    // destination_attributes, plan, threshold_fetch_buffer
    fn create_stream(&self, opts: RHash) -> Result<String, Error> {
        let client = self.inner.clone();
        let name = hash_require_string(&opts, "name")?;
        let network = hash_require_string(&opts, "network")?;
        let dataset_s = hash_require_string(&opts, "dataset")?;
        let region_s = hash_require_string(&opts, "region")?;
        let start_range = hash_require_i64(&opts, "start_range")?;
        let end_range = hash_require_i64(&opts, "end_range")?;
        let destination_attributes = hash_get_dest_attrs(&opts, "destination_attributes")?
            .ok_or_else(|| {
                Error::new(
                    ruby().exception_arg_error(),
                    "missing required key: destination_attributes",
                )
            })?;
        let plan = hash_require_string(&opts, "plan")?;
        let threshold_fetch_buffer = hash_require_i64(&opts, "threshold_fetch_buffer")?;
        let dataset = parse_enum::<core::streams::StreamDataset>(dataset_s)?;
        let region = parse_enum::<core::streams::StreamRegion>(region_s)?;
        let filter_language = parse_enum_opt::<core::streams::FilterLanguage>(hash_get_string(
            &opts,
            "filter_language",
        )?)?;
        let include_stream_metadata = parse_enum_opt::<core::streams::StreamMetadataLocation>(
            hash_get_string(&opts, "include_stream_metadata")?,
        )?;
        let product_type =
            parse_enum_opt::<core::streams::ProductType>(hash_get_string(&opts, "product_type")?)?;
        let status =
            parse_enum_opt::<core::streams::StreamStatus>(hash_get_string(&opts, "status")?)?;
        let params = core::streams::CreateStreamParams {
            name,
            network,
            dataset,
            region,
            start_range,
            end_range,
            destination_attributes,
            plan,
            threshold_fetch_buffer,
            dataset_batch_size: hash_get_i64(&opts, "dataset_batch_size")?,
            max_batch_size: hash_get_i64(&opts, "max_batch_size")?,
            max_buffer_range_size: hash_get_i64(&opts, "max_buffer_range_size")?,
            max_buffer_processing_workers: hash_get_i64(&opts, "max_buffer_processing_workers")?,
            keep_distance_from_tip: hash_get_i64(&opts, "keep_distance_from_tip")?,
            filter_function: hash_get_string(&opts, "filter_function")?,
            filter_language,
            address_book_config: None,
            include_stream_metadata,
            product_type,
            status,
            notification_email: hash_get_string(&opts, "notification_email")?,
            charge_min_cap: hash_get_i32(&opts, "charge_min_cap")?,
            fix_block_reorgs: hash_get_i32(&opts, "fix_block_reorgs")?,
            elastic_batch_enabled: hash_get_bool(&opts, "elastic_batch_enabled")?,
        };
        runtime()
            .block_on(client.create_stream(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn list_streams(
        &self,
        stream_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
        order_by: Option<String>,
        order_direction: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::streams::ListStreamsParams {
            stream_type,
            offset,
            limit,
            order_by,
            order_direction,
        };
        runtime()
            .block_on(client.list_streams(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_all_streams(&self) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_all_streams())
            .map_err(map_err)
    }

    fn get_stream(&self, id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_stream(&id))
            .map_err(map_err)
            .and_then(to_json)
    }

    // update_stream accepts id + a Ruby Hash (opts) because the param count exceeds 15.
    fn update_stream(&self, id: String, opts: RHash) -> Result<String, Error> {
        let client = self.inner.clone();
        let dataset =
            parse_enum_opt::<core::streams::StreamDataset>(hash_get_string(&opts, "dataset")?)?;
        let region =
            parse_enum_opt::<core::streams::StreamRegion>(hash_get_string(&opts, "region")?)?;
        let filter_language = parse_enum_opt::<core::streams::FilterLanguage>(hash_get_string(
            &opts,
            "filter_language",
        )?)?;
        let include_stream_metadata = parse_enum_opt::<core::streams::StreamMetadataLocation>(
            hash_get_string(&opts, "include_stream_metadata")?,
        )?;
        let status =
            parse_enum_opt::<core::streams::StreamStatus>(hash_get_string(&opts, "status")?)?;
        let destination_attributes = hash_get_dest_attrs(&opts, "destination_attributes")?;
        let params = core::streams::UpdateStreamParams {
            name: hash_get_string(&opts, "name")?,
            network: hash_get_string(&opts, "network")?,
            dataset,
            region,
            start_range: hash_get_i64(&opts, "start_range")?,
            end_range: hash_get_i64(&opts, "end_range")?,
            destination_attributes,
            plan: hash_get_string(&opts, "plan")?,
            threshold_fetch_buffer: hash_get_i64(&opts, "threshold_fetch_buffer")?,
            dataset_batch_size: hash_get_i64(&opts, "dataset_batch_size")?,
            max_batch_size: hash_get_i64(&opts, "max_batch_size")?,
            max_buffer_range_size: hash_get_i64(&opts, "max_buffer_range_size")?,
            max_buffer_processing_workers: hash_get_i64(&opts, "max_buffer_processing_workers")?,
            keep_distance_from_tip: hash_get_i64(&opts, "keep_distance_from_tip")?,
            filter_function: hash_get_string(&opts, "filter_function")?,
            filter_language,
            address_book_config: None,
            include_stream_metadata,
            notification_email: hash_get_string(&opts, "notification_email")?,
            charge_min_cap: hash_get_i32(&opts, "charge_min_cap")?,
            fix_block_reorgs: hash_get_i32(&opts, "fix_block_reorgs")?,
            elastic_batch_enabled: hash_get_bool(&opts, "elastic_batch_enabled")?,
            status,
            memo: hash_get_string(&opts, "memo")?,
        };
        runtime()
            .block_on(client.update_stream(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_stream(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_stream(&id))
            .map_err(map_err)
    }

    fn activate_stream(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.activate_stream(&id))
            .map_err(map_err)
    }

    fn pause_stream(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.pause_stream(&id))
            .map_err(map_err)
    }

    fn test_filter(
        &self,
        network: String,
        dataset: String,
        block: String,
        filter_function: Option<String>,
        filter_language: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let dataset = parse_enum::<core::streams::StreamDataset>(dataset)?;
        let filter_language = parse_enum_opt::<core::streams::FilterLanguage>(filter_language)?;
        let params = core::streams::TestFilterParams {
            network,
            dataset,
            block,
            filter_function,
            filter_language,
            address_book_config: None,
        };
        runtime()
            .block_on(client.test_filter(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_enabled_count(&self, stream_type: Option<String>) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_enabled_count(stream_type.as_deref()))
            .map_err(map_err)
            .and_then(to_json)
    }
}

// ── WebhooksApiClient ───────────────────────────────────────────────────────

#[magnus::wrap(class = "QuickNodeSdk::Webhooks", free_immediately, size)]
#[derive(Clone)]
pub struct WebhooksApiClient {
    inner: core::webhooks::WebhooksApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl WebhooksApiClient {
    fn list_webhooks(&self, limit: Option<i64>, offset: Option<i64>) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::webhooks::GetWebhooksParams { limit, offset };
        runtime()
            .block_on(client.list_webhooks(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_all_webhooks(&self) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_all_webhooks())
            .map_err(map_err)
    }

    fn get_webhook(&self, id: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_webhook(&id))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_webhook(
        &self,
        id: String,
        name: Option<String>,
        notification_email: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let params = core::webhooks::UpdateWebhookParams {
            name,
            notification_email,
            destination_attributes: None,
        };
        runtime()
            .block_on(client.update_webhook(&id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_webhook(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_webhook(&id))
            .map_err(map_err)
    }

    fn pause_webhook(&self, id: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.pause_webhook(&id))
            .map_err(map_err)
    }

    fn activate_webhook(&self, id: String, start_from: String) -> Result<(), Error> {
        let client = self.inner.clone();
        let start_from = parse_enum::<core::webhooks::WebhookStartFrom>(start_from)?;
        let params = core::webhooks::ActivateWebhookParams { start_from };
        runtime()
            .block_on(client.activate_webhook(&id, &params))
            .map_err(map_err)
    }

    fn get_enabled_count(&self) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_enabled_count())
            .map_err(map_err)
            .and_then(to_json)
    }

    fn create_webhook_from_template(
        &self,
        name: String,
        network: String,
        destination_attributes_json: String,
        template_args_json: String,
        notification_email: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let destination_attributes: core::webhooks::WebhookDestinationAttributes =
            serde_json::from_str(&destination_attributes_json).map_err(parse_err)?;
        let template_args: core::webhooks::TemplateArgs =
            serde_json::from_str(&template_args_json).map_err(parse_err)?;
        let params = core::webhooks::CreateWebhookFromTemplateParams {
            name,
            network,
            notification_email,
            destination_attributes,
            template_args,
        };
        runtime()
            .block_on(client.create_webhook_from_template(&params))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_webhook_template(
        &self,
        webhook_id: String,
        template_args_json: String,
        name: Option<String>,
        notification_email: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        let template_args: core::webhooks::TemplateArgs =
            serde_json::from_str(&template_args_json).map_err(parse_err)?;
        let params = core::webhooks::UpdateWebhookTemplateParams {
            name,
            notification_email,
            destination_attributes: None,
            template_args,
        };
        runtime()
            .block_on(client.update_webhook_template(&webhook_id, &params))
            .map_err(map_err)
            .and_then(to_json)
    }
}

// ── KvStoreApiClient ────────────────────────────────────────────────────────

#[magnus::wrap(class = "QuickNodeSdk::KvStore", free_immediately, size)]
#[derive(Clone)]
pub struct KvStoreApiClient {
    inner: core::kvstore::KvStoreApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl KvStoreApiClient {
    fn create_set(&self, key: String, value: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.create_set(&core::kvstore::CreateSetParams { key, value }))
            .map_err(map_err)
    }

    fn get_sets(&self, limit: Option<i64>, cursor: Option<String>) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_sets(&core::kvstore::GetSetsParams { limit, cursor }))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_set(&self, key: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_set(&key))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn bulk_sets(
        &self,
        add_sets: Option<std::collections::HashMap<String, String>>,
        delete_sets: Option<Vec<String>>,
    ) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.bulk_sets(&core::kvstore::BulkSetsParams {
                add_sets,
                delete_sets,
            }))
            .map_err(map_err)
    }

    fn delete_set(&self, key: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime().block_on(client.delete_set(&key)).map_err(map_err)
    }

    fn create_list(&self, key: String, items: Vec<String>) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.create_list(&core::kvstore::CreateListParams { key, items }))
            .map_err(map_err)
    }

    fn get_lists(&self, limit: Option<i64>, cursor: Option<String>) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_lists(&core::kvstore::GetListsParams { limit, cursor }))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn get_list(
        &self,
        key: String,
        limit: Option<i64>,
        cursor: Option<String>,
    ) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_list(&key, &core::kvstore::GetListParams { limit, cursor }))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn update_list(
        &self,
        key: String,
        add_items: Option<Vec<String>>,
        remove_items: Option<Vec<String>>,
    ) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.update_list(
                &key,
                &core::kvstore::UpdateListParams {
                    add_items,
                    remove_items,
                },
            ))
            .map_err(map_err)
    }

    fn add_list_item(&self, key: String, item: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.add_list_item(&key, &core::kvstore::AddListItemParams { item }))
            .map_err(map_err)
    }

    fn list_contains_item(&self, key: String, item: String) -> Result<String, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_contains_item(&key, &item))
            .map_err(map_err)
            .and_then(to_json)
    }

    fn delete_list_item(&self, key: String, item: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_list_item(&key, &item))
            .map_err(map_err)
    }

    fn delete_list(&self, key: String) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_list(&key))
            .map_err(map_err)
    }
}

// ── Extension init ──────────────────────────────────────────────────────────

#[magnus::init(name = "quicknode_sdk")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("QuickNodeSdk")?;

    // ── SDK root ──────────────────────────────────────────────
    let sdk = module.define_class("SDK", ruby.class_object())?;
    sdk.define_singleton_method("from_env", function!(QuickNodeSdk::from_env, 0))?;
    sdk.define_method("admin", method!(QuickNodeSdk::admin, 0))?;
    sdk.define_method("streams", method!(QuickNodeSdk::streams, 0))?;
    sdk.define_method("webhooks", method!(QuickNodeSdk::webhooks, 0))?;
    sdk.define_method("kvstore", method!(QuickNodeSdk::kvstore, 0))?;

    // ── Admin ─────────────────────────────────────────────────
    let admin = module.define_class("Admin", ruby.class_object())?;
    admin.define_method("get_endpoints", method!(AdminApiClient::get_endpoints, 4))?;
    admin.define_method(
        "create_endpoint",
        method!(AdminApiClient::create_endpoint, 2),
    )?;
    admin.define_method("show_endpoint", method!(AdminApiClient::show_endpoint, 1))?;
    admin.define_method(
        "update_endpoint",
        method!(AdminApiClient::update_endpoint, 2),
    )?;
    admin.define_method(
        "archive_endpoint",
        method!(AdminApiClient::archive_endpoint, 1),
    )?;
    admin.define_method(
        "update_endpoint_status",
        method!(AdminApiClient::update_endpoint_status, 2),
    )?;
    admin.define_method("create_tag", method!(AdminApiClient::create_tag, 2))?;
    admin.define_method("delete_tag", method!(AdminApiClient::delete_tag, 2))?;
    admin.define_method("get_usage", method!(AdminApiClient::get_usage, 2))?;
    admin.define_method(
        "get_usage_by_endpoint",
        method!(AdminApiClient::get_usage_by_endpoint, 2),
    )?;
    admin.define_method(
        "get_usage_by_method",
        method!(AdminApiClient::get_usage_by_method, 2),
    )?;
    admin.define_method(
        "get_usage_by_chain",
        method!(AdminApiClient::get_usage_by_chain, 2),
    )?;
    admin.define_method(
        "get_endpoint_logs",
        method!(AdminApiClient::get_endpoint_logs, 6),
    )?;
    admin.define_method(
        "get_log_details",
        method!(AdminApiClient::get_log_details, 2),
    )?;
    admin.define_method(
        "get_security_options",
        method!(AdminApiClient::get_security_options, 1),
    )?;
    admin.define_method(
        "update_security_options",
        method!(AdminApiClient::update_security_options, 10),
    )?;
    admin.define_method("create_token", method!(AdminApiClient::create_token, 1))?;
    admin.define_method("delete_token", method!(AdminApiClient::delete_token, 2))?;
    admin.define_method(
        "create_referrer",
        method!(AdminApiClient::create_referrer, 2),
    )?;
    admin.define_method(
        "delete_referrer",
        method!(AdminApiClient::delete_referrer, 2),
    )?;
    admin.define_method("create_ip", method!(AdminApiClient::create_ip, 2))?;
    admin.define_method("delete_ip", method!(AdminApiClient::delete_ip, 2))?;
    admin.define_method(
        "create_domain_mask",
        method!(AdminApiClient::create_domain_mask, 2),
    )?;
    admin.define_method(
        "delete_domain_mask",
        method!(AdminApiClient::delete_domain_mask, 2),
    )?;
    admin.define_method("create_jwt", method!(AdminApiClient::create_jwt, 4))?;
    admin.define_method("delete_jwt", method!(AdminApiClient::delete_jwt, 2))?;
    admin.define_method(
        "create_request_filter",
        method!(AdminApiClient::create_request_filter, 2),
    )?;
    admin.define_method(
        "update_request_filter",
        method!(AdminApiClient::update_request_filter, 3),
    )?;
    admin.define_method(
        "delete_request_filter",
        method!(AdminApiClient::delete_request_filter, 2),
    )?;
    admin.define_method(
        "enable_multichain",
        method!(AdminApiClient::enable_multichain, 1),
    )?;
    admin.define_method(
        "disable_multichain",
        method!(AdminApiClient::disable_multichain, 1),
    )?;
    admin.define_method(
        "create_or_update_ip_custom_header",
        method!(AdminApiClient::create_or_update_ip_custom_header, 2),
    )?;
    admin.define_method(
        "delete_ip_custom_header",
        method!(AdminApiClient::delete_ip_custom_header, 1),
    )?;
    admin.define_method(
        "get_method_rate_limits",
        method!(AdminApiClient::get_method_rate_limits, 1),
    )?;
    admin.define_method(
        "create_method_rate_limit",
        method!(AdminApiClient::create_method_rate_limit, 4),
    )?;
    admin.define_method(
        "update_method_rate_limit",
        method!(AdminApiClient::update_method_rate_limit, 5),
    )?;
    admin.define_method(
        "delete_method_rate_limit",
        method!(AdminApiClient::delete_method_rate_limit, 2),
    )?;
    admin.define_method(
        "update_rate_limits",
        method!(AdminApiClient::update_rate_limits, 4),
    )?;
    admin.define_method(
        "get_endpoint_metrics",
        method!(AdminApiClient::get_endpoint_metrics, 3),
    )?;
    admin.define_method(
        "get_account_metrics",
        method!(AdminApiClient::get_account_metrics, 3),
    )?;
    admin.define_method("list_chains", method!(AdminApiClient::list_chains, 0))?;
    admin.define_method("list_invoices", method!(AdminApiClient::list_invoices, 0))?;
    admin.define_method("list_payments", method!(AdminApiClient::list_payments, 0))?;
    admin.define_method("list_teams", method!(AdminApiClient::list_teams, 0))?;
    admin.define_method("create_team", method!(AdminApiClient::create_team, 1))?;
    admin.define_method("get_team", method!(AdminApiClient::get_team, 1))?;
    admin.define_method("delete_team", method!(AdminApiClient::delete_team, 1))?;
    admin.define_method(
        "list_team_endpoints",
        method!(AdminApiClient::list_team_endpoints, 1),
    )?;
    admin.define_method(
        "update_team_endpoints",
        method!(AdminApiClient::update_team_endpoints, 2),
    )?;
    admin.define_method(
        "invite_team_member",
        method!(AdminApiClient::invite_team_member, 4),
    )?;
    admin.define_method(
        "remove_team_member",
        method!(AdminApiClient::remove_team_member, 3),
    )?;
    admin.define_method(
        "resend_team_invite",
        method!(AdminApiClient::resend_team_invite, 2),
    )?;

    // ── DestinationAttributes ─────────────────────────────────
    let dest_attrs = module.define_class("DestinationAttributes", ruby.class_object())?;
    dest_attrs.define_singleton_method("webhook", function!(DestinationAttributes::webhook, 1))?;
    dest_attrs.define_singleton_method("s3", function!(DestinationAttributes::s3, 1))?;
    dest_attrs.define_singleton_method("azure", function!(DestinationAttributes::azure, 1))?;
    dest_attrs
        .define_singleton_method("postgres", function!(DestinationAttributes::postgres, 1))?;
    dest_attrs.define_singleton_method("mysql", function!(DestinationAttributes::mysql, 1))?;
    dest_attrs.define_singleton_method("mongo", function!(DestinationAttributes::mongo, 1))?;
    dest_attrs.define_singleton_method(
        "clickhouse",
        function!(DestinationAttributes::clickhouse, 1),
    )?;
    dest_attrs.define_singleton_method(
        "snowflake",
        function!(DestinationAttributes::snowflake, 1),
    )?;
    dest_attrs.define_singleton_method("kafka", function!(DestinationAttributes::kafka, 1))?;
    dest_attrs.define_singleton_method("redis", function!(DestinationAttributes::redis, 1))?;

    // ── Streams ───────────────────────────────────────────────
    let streams = module.define_class("Streams", ruby.class_object())?;
    // create_stream takes a Hash (opts) because the param count exceeds magnus arity limit of 15
    streams.define_method("create_stream", method!(StreamsApiClient::create_stream, 1))?;
    streams.define_method("list_streams", method!(StreamsApiClient::list_streams, 5))?;
    streams.define_method(
        "delete_all_streams",
        method!(StreamsApiClient::delete_all_streams, 0),
    )?;
    streams.define_method("get_stream", method!(StreamsApiClient::get_stream, 1))?;
    // update_stream takes id + a Hash (opts)
    streams.define_method("update_stream", method!(StreamsApiClient::update_stream, 2))?;
    streams.define_method("delete_stream", method!(StreamsApiClient::delete_stream, 1))?;
    streams.define_method(
        "activate_stream",
        method!(StreamsApiClient::activate_stream, 1),
    )?;
    streams.define_method("pause_stream", method!(StreamsApiClient::pause_stream, 1))?;
    streams.define_method("test_filter", method!(StreamsApiClient::test_filter, 5))?;
    streams.define_method(
        "get_enabled_count",
        method!(StreamsApiClient::get_enabled_count, 1),
    )?;

    // ── Webhooks ──────────────────────────────────────────────
    let webhooks = module.define_class("Webhooks", ruby.class_object())?;
    webhooks.define_method(
        "list_webhooks",
        method!(WebhooksApiClient::list_webhooks, 2),
    )?;
    webhooks.define_method(
        "delete_all_webhooks",
        method!(WebhooksApiClient::delete_all_webhooks, 0),
    )?;
    webhooks.define_method("get_webhook", method!(WebhooksApiClient::get_webhook, 1))?;
    webhooks.define_method(
        "update_webhook",
        method!(WebhooksApiClient::update_webhook, 3),
    )?;
    webhooks.define_method(
        "delete_webhook",
        method!(WebhooksApiClient::delete_webhook, 1),
    )?;
    webhooks.define_method(
        "pause_webhook",
        method!(WebhooksApiClient::pause_webhook, 1),
    )?;
    webhooks.define_method(
        "activate_webhook",
        method!(WebhooksApiClient::activate_webhook, 2),
    )?;
    webhooks.define_method(
        "get_enabled_count",
        method!(WebhooksApiClient::get_enabled_count, 0),
    )?;
    webhooks.define_method(
        "create_webhook_from_template",
        method!(WebhooksApiClient::create_webhook_from_template, 5),
    )?;
    webhooks.define_method(
        "update_webhook_template",
        method!(WebhooksApiClient::update_webhook_template, 4),
    )?;

    // ── KvStore ───────────────────────────────────────────────
    let kvstore = module.define_class("KvStore", ruby.class_object())?;
    kvstore.define_method("create_set", method!(KvStoreApiClient::create_set, 2))?;
    kvstore.define_method("get_sets", method!(KvStoreApiClient::get_sets, 2))?;
    kvstore.define_method("get_set", method!(KvStoreApiClient::get_set, 1))?;
    kvstore.define_method("bulk_sets", method!(KvStoreApiClient::bulk_sets, 2))?;
    kvstore.define_method("delete_set", method!(KvStoreApiClient::delete_set, 1))?;
    kvstore.define_method("create_list", method!(KvStoreApiClient::create_list, 2))?;
    kvstore.define_method("get_lists", method!(KvStoreApiClient::get_lists, 2))?;
    kvstore.define_method("get_list", method!(KvStoreApiClient::get_list, 3))?;
    kvstore.define_method("update_list", method!(KvStoreApiClient::update_list, 3))?;
    kvstore.define_method("add_list_item", method!(KvStoreApiClient::add_list_item, 2))?;
    kvstore.define_method(
        "list_contains_item",
        method!(KvStoreApiClient::list_contains_item, 2),
    )?;
    kvstore.define_method(
        "delete_list_item",
        method!(KvStoreApiClient::delete_list_item, 2),
    )?;
    kvstore.define_method("delete_list", method!(KvStoreApiClient::delete_list, 1))?;

    Ok(())
}
