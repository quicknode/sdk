#![allow(clippy::expect_used)]
use magnus::{
    function, method, prelude::*, r_hash::ForEach, symbol::Symbol, Error, RArray, RHash, Ruby,
};
use quicknode_sdk as core;

mod errors;

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
    errors::map_err(e)
}

#[allow(clippy::needless_pass_by_value)]
fn parse_err(e: serde_json::Error) -> Error {
    Error::new(ruby().exception_arg_error(), e.to_string())
}

fn to_ruby<T: serde::Serialize>(v: T) -> Result<magnus::Value, Error> {
    serde_magnus::serialize(&ruby(), &v).map_err(|e| {
        Error::new(
            ruby().exception_runtime_error(),
            format!("response serialization failed: {e}"),
        )
    })
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

fn hash_require_bool(h: &RHash, key: &str) -> Result<bool, Error> {
    hash_get_bool(h, key)?.ok_or_else(|| {
        Error::new(
            ruby().exception_arg_error(),
            format!("missing required key: {key}"),
        )
    })
}

fn hash_require_i32(h: &RHash, key: &str) -> Result<i32, Error> {
    hash_get_i32(h, key)?.ok_or_else(|| {
        Error::new(
            ruby().exception_arg_error(),
            format!("missing required key: {key}"),
        )
    })
}

fn hash_get_vec_string(h: &RHash, key: &str) -> Result<Option<Vec<String>>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => Vec::<String>::try_convert(v)
            .map(Some)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}"))),
        _ => Ok(None),
    }
}

fn hash_get_vec_i32(h: &RHash, key: &str) -> Result<Option<Vec<i32>>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => Vec::<i32>::try_convert(v)
            .map(Some)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}"))),
        _ => Ok(None),
    }
}

fn hash_get_map_string_string(
    h: &RHash,
    key: &str,
) -> Result<Option<std::collections::HashMap<String, String>>, Error> {
    let r = ruby();
    let Some(v) = h.get(r.to_symbol(key)) else {
        return Ok(None);
    };
    if v.is_nil() {
        return Ok(None);
    }
    let inner: RHash = magnus::TryConvert::try_convert(v)
        .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}")))?;
    let mut out = std::collections::HashMap::with_capacity(inner.len());
    inner.foreach(|k: magnus::Value, val: magnus::Value| {
        let k_str = String::try_convert(k)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key} key: {e}")))?;
        let v_str = String::try_convert(val)
            .map_err(|e| Error::new(r.exception_type_error(), format!("{key} value: {e}")))?;
        out.insert(k_str, v_str);
        Ok(ForEach::Continue)
    })?;
    Ok(Some(out))
}

fn hash_require_vec_string(h: &RHash, key: &str) -> Result<Vec<String>, Error> {
    hash_get_vec_string(h, key)?.ok_or_else(|| {
        Error::new(
            ruby().exception_arg_error(),
            format!("missing required key: {key}"),
        )
    })
}

fn validate_keys(h: &RHash, allowed: &[&str]) -> Result<(), Error> {
    let r = ruby();
    h.foreach(|key: Symbol, _val: magnus::Value| {
        let key_str = key
            .name()
            .map_err(|e| Error::new(r.exception_arg_error(), e.to_string()))?;
        if !allowed.contains(&key_str.as_ref()) {
            return Err(Error::new(
                r.exception_arg_error(),
                format!("unknown key: {key_str} (allowed: {})", allowed.join(", ")),
            ));
        }
        Ok(ForEach::Continue)
    })
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

fn hash_get_extra_destinations(
    h: &RHash,
    key: &str,
) -> Result<Option<Vec<core::streams::DestinationAttributes>>, Error> {
    let r = ruby();
    match h.get(r.to_symbol(key)) {
        Some(v) if !v.is_nil() => {
            let arr: RArray = magnus::TryConvert::try_convert(v)
                .map_err(|e| Error::new(r.exception_type_error(), format!("{key}: {e}")))?;
            let mut out = Vec::with_capacity(arr.len());
            for item in arr.into_iter() {
                let wrapped: &DestinationAttributes = magnus::TryConvert::try_convert(item)
                    .map_err(|e| {
                        Error::new(
                            r.exception_type_error(),
                            format!("{key}: element must be a DestinationAttributes: {e}"),
                        )
                    })?;
                out.push(wrapped.inner.clone());
            }
            Ok(Some(out))
        }
        _ => Ok(None),
    }
}

// ── QuicknodeSdk ────────────────────────────────────────────────────────────

#[magnus::wrap(class = "QuicknodeSdk::Native::SDK", free_immediately, size)]
pub struct QuicknodeSdk {
    inner: core::QuicknodeSdk,
}

impl QuicknodeSdk {
    fn from_env() -> Result<Self, Error> {
        core::QuicknodeSdk::from_env()
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
// Methods returning data return native Ruby Hash/Array via serde_magnus. The
// Ruby package wraps these in QuicknodeSdk::IndifferentHash (a Hash subclass
// with Hashie::Extensions::IndifferentAccess) before returning to the user.

#[magnus::wrap(class = "QuicknodeSdk::Native::Admin", free_immediately, size)]
#[derive(Clone)]
pub struct AdminApiClient {
    inner: core::admin::AdminApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl AdminApiClient {
    fn get_endpoints(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "limit",
                "offset",
                "search",
                "sort_by",
                "sort_direction",
                "networks",
                "statuses",
                "labels",
                "dedicated",
                "is_flat_rate",
                "tag_ids",
                "tag_labels",
            ],
        )?;
        let client = self.inner.clone();
        let params = core::admin::GetEndpointsRequest {
            limit: hash_get_i32(&opts, "limit")?,
            offset: hash_get_i32(&opts, "offset")?,
            search: hash_get_string(&opts, "search")?,
            sort_by: hash_get_string(&opts, "sort_by")?,
            sort_direction: hash_get_string(&opts, "sort_direction")?,
            networks: hash_get_vec_string(&opts, "networks")?,
            statuses: hash_get_vec_string(&opts, "statuses")?,
            labels: hash_get_vec_string(&opts, "labels")?,
            dedicated: hash_get_bool(&opts, "dedicated")?,
            is_flat_rate: hash_get_bool(&opts, "is_flat_rate")?,
            tag_ids: hash_get_vec_i32(&opts, "tag_ids")?,
            tag_labels: hash_get_vec_string(&opts, "tag_labels")?,
        };
        runtime()
            .block_on(client.get_endpoints(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_endpoint(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["chain", "network"])?;
        let client = self.inner.clone();
        let params = core::admin::CreateEndpointRequest {
            chain: hash_get_string(&opts, "chain")?,
            network: hash_get_string(&opts, "network")?,
        };
        runtime()
            .block_on(client.create_endpoint(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn show_endpoint(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.show_endpoint(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_endpoint(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "label"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::UpdateEndpointRequest {
            label: hash_get_string(&opts, "label")?,
        };
        runtime()
            .block_on(client.update_endpoint(&id, &params))
            .map_err(map_err)
    }

    fn archive_endpoint(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.archive_endpoint(&id))
            .map_err(map_err)
    }

    fn update_endpoint_status(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "status"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::UpdateEndpointStatusRequest {
            status: hash_require_string(&opts, "status")?,
        };
        runtime()
            .block_on(client.update_endpoint_status(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_tag(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "label"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateTagRequest {
            label: hash_get_string(&opts, "label")?,
        };
        runtime()
            .block_on(client.create_tag(&id, &params))
            .map_err(map_err)
    }

    fn delete_tag(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "tag_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let tag_id = hash_require_string(&opts, "tag_id")?;
        runtime()
            .block_on(client.delete_tag(&id, &tag_id))
            .map_err(map_err)
    }

    fn get_usage(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["start_time", "end_time"])?;
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time: hash_get_i64(&opts, "start_time")?,
            end_time: hash_get_i64(&opts, "end_time")?,
        };
        runtime()
            .block_on(client.get_usage(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_usage_by_endpoint(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["start_time", "end_time"])?;
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time: hash_get_i64(&opts, "start_time")?,
            end_time: hash_get_i64(&opts, "end_time")?,
        };
        runtime()
            .block_on(client.get_usage_by_endpoint(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_usage_by_method(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["start_time", "end_time"])?;
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time: hash_get_i64(&opts, "start_time")?,
            end_time: hash_get_i64(&opts, "end_time")?,
        };
        runtime()
            .block_on(client.get_usage_by_method(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_usage_by_chain(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["start_time", "end_time"])?;
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time: hash_get_i64(&opts, "start_time")?,
            end_time: hash_get_i64(&opts, "end_time")?,
        };
        runtime()
            .block_on(client.get_usage_by_chain(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_endpoint_logs(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "id",
                "from_time",
                "to_time",
                "include_details",
                "limit",
                "next_at",
            ],
        )?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::GetEndpointLogsRequest {
            from: hash_require_string(&opts, "from_time")?,
            to: hash_require_string(&opts, "to_time")?,
            include_details: hash_get_bool(&opts, "include_details")?,
            limit: hash_get_i32(&opts, "limit")?,
            next_at: hash_get_string(&opts, "next_at")?,
        };
        runtime()
            .block_on(client.get_endpoint_logs(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_log_details(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "request_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let request_id = hash_require_string(&opts, "request_id")?;
        runtime()
            .block_on(client.get_log_details(&id, &request_id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_security_options(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_security_options(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_security_options(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "id",
                "tokens",
                "referrers",
                "jwts",
                "ips",
                "domain_masks",
                "hsts",
                "cors",
                "request_filters",
                "ip_custom_header",
            ],
        )?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::UpdateSecurityOptionsRequest {
            options: core::admin::SecurityOptionsUpdate {
                tokens: hash_get_string(&opts, "tokens")?,
                referrers: hash_get_string(&opts, "referrers")?,
                jwts: hash_get_string(&opts, "jwts")?,
                ips: hash_get_string(&opts, "ips")?,
                domain_masks: hash_get_string(&opts, "domain_masks")?,
                hsts: hash_get_string(&opts, "hsts")?,
                cors: hash_get_string(&opts, "cors")?,
                request_filters: hash_get_string(&opts, "request_filters")?,
                ip_custom_header: hash_get_string(&opts, "ip_custom_header")?,
            },
        };
        runtime()
            .block_on(client.update_security_options(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_token(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.create_token(&id))
            .map_err(map_err)
    }

    fn delete_token(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "token_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let token_id = hash_require_string(&opts, "token_id")?;
        runtime()
            .block_on(client.delete_token(&id, &token_id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_referrer(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "referrer"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateReferrerRequest {
            referrer: hash_get_string(&opts, "referrer")?,
        };
        runtime()
            .block_on(client.create_referrer(&id, &params))
            .map_err(map_err)
    }

    fn delete_referrer(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "referrer_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let referrer_id = hash_require_string(&opts, "referrer_id")?;
        runtime()
            .block_on(client.delete_referrer(&id, &referrer_id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_ip(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "ip"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateIpRequest {
            ip: hash_get_string(&opts, "ip")?,
        };
        runtime()
            .block_on(client.create_ip(&id, &params))
            .map_err(map_err)
    }

    fn delete_ip(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "ip_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let ip_id = hash_require_string(&opts, "ip_id")?;
        runtime()
            .block_on(client.delete_ip(&id, &ip_id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_domain_mask(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "domain_mask"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateDomainMaskRequest {
            domain_mask: hash_get_string(&opts, "domain_mask")?,
        };
        runtime()
            .block_on(client.create_domain_mask(&id, &params))
            .map_err(map_err)
    }

    fn delete_domain_mask(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "domain_mask_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let domain_mask_id = hash_require_string(&opts, "domain_mask_id")?;
        runtime()
            .block_on(client.delete_domain_mask(&id, &domain_mask_id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_jwt(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "public_key", "kid", "name"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateJwtRequest {
            public_key: hash_get_string(&opts, "public_key")?,
            kid: hash_get_string(&opts, "kid")?,
            name: hash_get_string(&opts, "name")?,
        };
        runtime()
            .block_on(client.create_jwt(&id, &params))
            .map_err(map_err)
    }

    fn delete_jwt(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "jwt_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let jwt_id = hash_require_string(&opts, "jwt_id")?;
        runtime()
            .block_on(client.delete_jwt(&id, &jwt_id))
            .map_err(map_err)
    }

    fn create_request_filter(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "methods"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateRequestFilterRequest {
            method: hash_get_vec_string(&opts, "methods")?,
        };
        runtime()
            .block_on(client.create_request_filter(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_request_filter(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "request_filter_id", "methods"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let request_filter_id = hash_require_string(&opts, "request_filter_id")?;
        let params = core::admin::UpdateRequestFilterRequest {
            method: hash_get_vec_string(&opts, "methods")?,
        };
        runtime()
            .block_on(client.update_request_filter(&id, &request_filter_id, &params))
            .map_err(map_err)
    }

    fn delete_request_filter(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "request_filter_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let request_filter_id = hash_require_string(&opts, "request_filter_id")?;
        runtime()
            .block_on(client.delete_request_filter(&id, &request_filter_id))
            .map_err(map_err)
    }

    fn enable_multichain(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.enable_multichain(&id))
            .map_err(map_err)
    }

    fn disable_multichain(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.disable_multichain(&id))
            .map_err(map_err)
    }

    fn create_or_update_ip_custom_header(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "header_name"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateOrUpdateIpCustomHeaderRequest {
            header_name: hash_require_string(&opts, "header_name")?,
        };
        runtime()
            .block_on(client.create_or_update_ip_custom_header(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_ip_custom_header(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.delete_ip_custom_header(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_method_rate_limits(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_method_rate_limits(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_method_rate_limit(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "interval", "methods", "rate"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::CreateMethodRateLimitRequest {
            interval: hash_require_string(&opts, "interval")?,
            methods: hash_require_vec_string(&opts, "methods")?,
            rate: hash_require_i32(&opts, "rate")?,
        };
        runtime()
            .block_on(client.create_method_rate_limit(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_method_rate_limit(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &["id", "method_rate_limit_id", "methods", "status", "rate"],
        )?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let method_rate_limit_id = hash_require_string(&opts, "method_rate_limit_id")?;
        let params = core::admin::UpdateMethodRateLimitRequest {
            methods: hash_get_vec_string(&opts, "methods")?,
            status: hash_get_string(&opts, "status")?,
            rate: hash_get_i32(&opts, "rate")?,
        };
        runtime()
            .block_on(client.update_method_rate_limit(&id, &method_rate_limit_id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_method_rate_limit(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "method_rate_limit_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let method_rate_limit_id = hash_require_string(&opts, "method_rate_limit_id")?;
        runtime()
            .block_on(client.delete_method_rate_limit(&id, &method_rate_limit_id))
            .map_err(map_err)
    }

    fn update_rate_limits(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "rps", "rpm", "rpd"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::UpdateRateLimitsRequest {
            rate_limits: core::admin::RateLimitSettings {
                rps: hash_get_i32(&opts, "rps")?,
                rpm: hash_get_i32(&opts, "rpm")?,
                rpd: hash_get_i32(&opts, "rpd")?,
            },
        };
        runtime()
            .block_on(client.update_rate_limits(&id, &params))
            .map_err(map_err)
    }

    fn get_rate_limits(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_rate_limits(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_rate_limit_override(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "override_id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let override_id = hash_require_string(&opts, "override_id")?;
        runtime()
            .block_on(client.delete_rate_limit_override(&id, &override_id))
            .map_err(map_err)
    }

    fn get_endpoint_urls(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_endpoint_urls(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_endpoint_metrics(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "period", "metric"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let params = core::admin::GetEndpointMetricsRequest {
            period: hash_require_string(&opts, "period")?,
            metric: hash_require_string(&opts, "metric")?,
        };
        runtime()
            .block_on(client.get_endpoint_metrics(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_account_metrics(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["period", "metric", "percentile"])?;
        let client = self.inner.clone();
        let params = core::admin::GetAccountMetricsRequest {
            period: hash_require_string(&opts, "period")?,
            metric: hash_require_string(&opts, "metric")?,
            percentile: hash_get_string(&opts, "percentile")?,
        };
        runtime()
            .block_on(client.get_account_metrics(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_chains(&self) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_chains())
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_invoices(&self) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_invoices())
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_payments(&self) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_payments())
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_teams(&self) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_teams())
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_team(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["name"])?;
        let client = self.inner.clone();
        let params = core::admin::CreateTeamRequest {
            name: hash_require_string(&opts, "name")?,
        };
        runtime()
            .block_on(client.create_team(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_team(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        runtime()
            .block_on(client.get_team(id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_team(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        runtime()
            .block_on(client.delete_team(id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_team_endpoints(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        runtime()
            .block_on(client.list_team_endpoints(id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_team_endpoints(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "endpoint_ids"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        let params = core::admin::UpdateTeamEndpointsRequest {
            endpoint_ids: hash_require_vec_string(&opts, "endpoint_ids")?,
        };
        runtime()
            .block_on(client.update_team_endpoints(id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn invite_team_member(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "email", "full_name", "role"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        let params = core::admin::InviteTeamMemberRequest {
            email: hash_require_string(&opts, "email")?,
            full_name: hash_get_string(&opts, "full_name")?,
            role: hash_get_string(&opts, "role")?,
        };
        runtime()
            .block_on(client.invite_team_member(id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn remove_team_member(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "user_id", "destroy_user"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        let user_id = hash_require_i64(&opts, "user_id")?;
        let params = core::admin::RemoveTeamMemberRequest {
            destroy_user: hash_get_bool(&opts, "destroy_user")?,
        };
        runtime()
            .block_on(client.remove_team_member(id, user_id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn resend_team_invite(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "user_id"])?;
        let client = self.inner.clone();
        let id = hash_require_i64(&opts, "id")?;
        let user_id = hash_require_i64(&opts, "user_id")?;
        runtime()
            .block_on(client.resend_team_invite(id, user_id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn bulk_update_endpoint_status(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["ids", "status"])?;
        let client = self.inner.clone();
        let params = core::admin::BulkUpdateEndpointStatusRequest {
            ids: hash_require_vec_string(&opts, "ids")?,
            status: hash_require_string(&opts, "status")?,
        };
        runtime()
            .block_on(client.bulk_update_endpoint_status(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn bulk_add_tag(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["ids", "label"])?;
        let client = self.inner.clone();
        let params = core::admin::BulkAddTagRequest {
            ids: hash_require_vec_string(&opts, "ids")?,
            label: hash_require_string(&opts, "label")?,
        };
        runtime()
            .block_on(client.bulk_add_tag(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn bulk_remove_tag(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["ids", "tag_id"])?;
        let client = self.inner.clone();
        let params = core::admin::BulkRemoveTagRequest {
            ids: hash_require_vec_string(&opts, "ids")?,
            tag_id: hash_require_i32(&opts, "tag_id")?,
        };
        runtime()
            .block_on(client.bulk_remove_tag(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_tags(&self) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.list_tags())
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn rename_tag(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id", "label"])?;
        let client = self.inner.clone();
        let id = hash_require_i32(&opts, "id")?;
        let params = core::admin::RenameTagRequest {
            label: hash_require_string(&opts, "label")?,
        };
        runtime()
            .block_on(client.rename_tag(id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_account_tag(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_i32(&opts, "id")?;
        runtime()
            .block_on(client.delete_account_tag(id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_usage_by_tag(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["start_time", "end_time"])?;
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time: hash_get_i64(&opts, "start_time")?,
            end_time: hash_get_i64(&opts, "end_time")?,
        };
        runtime()
            .block_on(client.get_usage_by_tag(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_endpoint_security(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_endpoint_security(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }
}

// ── DestinationAttributes ───────────────────────────────────────────────────

#[magnus::wrap(class = "QuicknodeSdk::DestinationAttributes", free_immediately, size)]
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
            compression: hash_get_string(&opts, "compression")?,
        };
        Ok(Self {
            inner: core::streams::DestinationAttributes::Webhook(attrs),
        })
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
        Ok(Self {
            inner: core::streams::DestinationAttributes::S3(attrs),
        })
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
        Ok(Self {
            inner: core::streams::DestinationAttributes::Azure(attrs),
        })
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
        Ok(Self {
            inner: core::streams::DestinationAttributes::Postgres(attrs),
        })
    }

    fn kafka(opts: RHash) -> Result<Self, Error> {
        let attrs = core::streams::KafkaAttributes {
            bootstrap_servers: hash_require_string(&opts, "bootstrap_servers")?,
            topic_name: hash_require_string(&opts, "topic_name")?,
            compression_type: hash_require_string(&opts, "compression_type")?,
            batch_size: hash_get_i32(&opts, "batch_size")?.unwrap_or(0),
            linger_ms: hash_get_i32(&opts, "linger_ms")?.unwrap_or(0),
            max_message_bytes: hash_get_i32(&opts, "max_message_bytes")?.unwrap_or(0),
            timeout_sec: hash_get_i32(&opts, "timeout_sec")?.unwrap_or(0),
            max_retry: hash_get_i32(&opts, "max_retry")?.unwrap_or(0),
            retry_interval_sec: hash_get_i32(&opts, "retry_interval_sec")?.unwrap_or(0),
            username: hash_get_string(&opts, "username")?,
            password: hash_get_string(&opts, "password")?,
            protocol: hash_get_string(&opts, "protocol")?,
            mechanisms: hash_get_string(&opts, "mechanisms")?,
        };
        Ok(Self {
            inner: core::streams::DestinationAttributes::Kafka(attrs),
        })
    }
}

// ── StreamsApiClient ────────────────────────────────────────────────────────

#[magnus::wrap(class = "QuicknodeSdk::Native::Streams", free_immediately, size)]
#[derive(Clone)]
pub struct StreamsApiClient {
    inner: core::streams::StreamsApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl StreamsApiClient {
    // create_stream accepts a Ruby Hash because the param count exceeds magnus arity limit of 15.
    // Required keys: name, network, dataset, region, start_range, end_range,
    // destination_attributes, plan, threshold_fetch_buffer
    fn create_stream(&self, opts: RHash) -> Result<magnus::Value, Error> {
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
        let plan = hash_get_string(&opts, "plan")?;
        let threshold_fetch_buffer = hash_get_i64(&opts, "threshold_fetch_buffer")?;
        let dataset_batch_size = hash_require_i64(&opts, "dataset_batch_size")?;
        let elastic_batch_enabled = hash_require_bool(&opts, "elastic_batch_enabled")?;
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
            dataset_batch_size,
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
            elastic_batch_enabled,
            extra_destinations: hash_get_extra_destinations(&opts, "extra_destinations")?,
        };
        runtime()
            .block_on(client.create_stream(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn list_streams(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "stream_type",
                "offset",
                "limit",
                "order_by",
                "order_direction",
            ],
        )?;
        let client = self.inner.clone();
        let params = core::streams::ListStreamsParams {
            stream_type: hash_get_string(&opts, "stream_type")?,
            offset: hash_get_i64(&opts, "offset")?,
            limit: hash_get_i64(&opts, "limit")?,
            order_by: hash_get_string(&opts, "order_by")?,
            order_direction: hash_get_string(&opts, "order_direction")?,
        };
        runtime()
            .block_on(client.list_streams(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_all_streams(&self) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_all_streams())
            .map_err(map_err)
    }

    fn get_stream(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_stream(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    // update_stream accepts id + a Ruby Hash (opts) because the param count exceeds 15.
    fn update_stream(&self, opts: RHash) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
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
            extra_destinations: hash_get_extra_destinations(&opts, "extra_destinations")?,
        };
        runtime()
            .block_on(client.update_stream(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_stream(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.delete_stream(&id))
            .map_err(map_err)
    }

    fn activate_stream(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.activate_stream(&id))
            .map_err(map_err)
    }

    fn pause_stream(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.pause_stream(&id))
            .map_err(map_err)
    }

    fn test_filter(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "network",
                "dataset",
                "block",
                "filter_function",
                "filter_language",
            ],
        )?;
        let client = self.inner.clone();
        let dataset =
            parse_enum::<core::streams::StreamDataset>(hash_require_string(&opts, "dataset")?)?;
        let filter_language = parse_enum_opt::<core::streams::FilterLanguage>(hash_get_string(
            &opts,
            "filter_language",
        )?)?;
        let params = core::streams::TestFilterParams {
            network: hash_require_string(&opts, "network")?,
            dataset,
            block: hash_require_string(&opts, "block")?,
            filter_function: hash_require_string(&opts, "filter_function")?,
            filter_language,
            address_book_config: None,
        };
        runtime()
            .block_on(client.test_filter(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_enabled_count(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["stream_type"])?;
        let client = self.inner.clone();
        let stream_type = hash_get_string(&opts, "stream_type")?;
        runtime()
            .block_on(client.get_enabled_count(stream_type.as_deref()))
            .map_err(map_err)
            .and_then(to_ruby)
    }
}

// ── WebhooksApiClient ───────────────────────────────────────────────────────

#[magnus::wrap(class = "QuicknodeSdk::Native::Webhooks", free_immediately, size)]
#[derive(Clone)]
pub struct WebhooksApiClient {
    inner: core::webhooks::WebhooksApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl WebhooksApiClient {
    fn list_webhooks(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["limit", "offset"])?;
        let client = self.inner.clone();
        let params = core::webhooks::GetWebhooksParams {
            limit: hash_get_i64(&opts, "limit")?,
            offset: hash_get_i64(&opts, "offset")?,
        };
        runtime()
            .block_on(client.list_webhooks(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_all_webhooks(&self) -> Result<(), Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.delete_all_webhooks())
            .map_err(map_err)
    }

    fn get_webhook(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.get_webhook(&id))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_webhook(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "id",
                "name",
                "notification_email",
                "destination_attributes_json",
            ],
        )?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let destination_attributes = match hash_get_string(&opts, "destination_attributes_json")? {
            Some(json) => Some(
                serde_json::from_str::<core::webhooks::WebhookDestinationAttributes>(&json)
                    .map_err(parse_err)?,
            ),
            None => None,
        };
        let params = core::webhooks::UpdateWebhookParams {
            name: hash_get_string(&opts, "name")?,
            notification_email: hash_get_string(&opts, "notification_email")?,
            destination_attributes,
        };
        runtime()
            .block_on(client.update_webhook(&id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_webhook(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.delete_webhook(&id))
            .map_err(map_err)
    }

    fn pause_webhook(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        runtime()
            .block_on(client.pause_webhook(&id))
            .map_err(map_err)
    }

    fn activate_webhook(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["id", "start_from"])?;
        let client = self.inner.clone();
        let id = hash_require_string(&opts, "id")?;
        let start_from = parse_enum::<core::webhooks::WebhookStartFrom>(hash_require_string(
            &opts,
            "start_from",
        )?)?;
        let params = core::webhooks::ActivateWebhookParams { start_from };
        runtime()
            .block_on(client.activate_webhook(&id, &params))
            .map_err(map_err)
    }

    fn get_enabled_count(&self) -> Result<magnus::Value, Error> {
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_enabled_count())
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn create_webhook_from_template(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "name",
                "network",
                "destination_attributes_json",
                "template_args_json",
                "notification_email",
            ],
        )?;
        let client = self.inner.clone();
        let destination_attributes_json =
            hash_require_string(&opts, "destination_attributes_json")?;
        let template_args_json = hash_require_string(&opts, "template_args_json")?;
        let destination_attributes: core::webhooks::WebhookDestinationAttributes =
            serde_json::from_str(&destination_attributes_json).map_err(parse_err)?;
        let template_args: core::webhooks::TemplateArgs =
            serde_json::from_str(&template_args_json).map_err(parse_err)?;
        let params = core::webhooks::CreateWebhookFromTemplateParams {
            name: hash_require_string(&opts, "name")?,
            network: hash_require_string(&opts, "network")?,
            notification_email: hash_get_string(&opts, "notification_email")?,
            destination_attributes,
            template_args,
        };
        runtime()
            .block_on(client.create_webhook_from_template(&params))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_webhook_template(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(
            &opts,
            &[
                "webhook_id",
                "template_args_json",
                "name",
                "notification_email",
            ],
        )?;
        let client = self.inner.clone();
        let webhook_id = hash_require_string(&opts, "webhook_id")?;
        let template_args_json = hash_require_string(&opts, "template_args_json")?;
        let template_args: core::webhooks::TemplateArgs =
            serde_json::from_str(&template_args_json).map_err(parse_err)?;
        let params = core::webhooks::UpdateWebhookTemplateParams {
            name: hash_get_string(&opts, "name")?,
            notification_email: hash_get_string(&opts, "notification_email")?,
            destination_attributes: None,
            template_args,
        };
        runtime()
            .block_on(client.update_webhook_template(&webhook_id, &params))
            .map_err(map_err)
            .and_then(to_ruby)
    }
}

// ── KvStoreApiClient ────────────────────────────────────────────────────────

#[magnus::wrap(class = "QuicknodeSdk::Native::KvStore", free_immediately, size)]
#[derive(Clone)]
pub struct KvStoreApiClient {
    inner: core::kvstore::KvStoreApiClient,
}

#[allow(clippy::needless_pass_by_value)]
impl KvStoreApiClient {
    fn create_set(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key", "value"])?;
        let client = self.inner.clone();
        runtime()
            .block_on(client.create_set(&core::kvstore::CreateSetParams {
                key: hash_require_string(&opts, "key")?,
                value: hash_require_string(&opts, "value")?,
            }))
            .map_err(map_err)
    }

    fn get_sets(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["limit", "cursor"])?;
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_sets(&core::kvstore::GetSetsParams {
                limit: hash_get_i64(&opts, "limit")?,
                cursor: hash_get_string(&opts, "cursor")?,
            }))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_set(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["key"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        runtime()
            .block_on(client.get_set(&key))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn bulk_sets(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["add_sets", "delete_sets"])?;
        let client = self.inner.clone();
        runtime()
            .block_on(client.bulk_sets(&core::kvstore::BulkSetsParams {
                add_sets: hash_get_map_string_string(&opts, "add_sets")?,
                delete_sets: hash_get_vec_string(&opts, "delete_sets")?,
            }))
            .map_err(map_err)
    }

    fn delete_set(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        runtime().block_on(client.delete_set(&key)).map_err(map_err)
    }

    fn create_list(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key", "items"])?;
        let client = self.inner.clone();
        runtime()
            .block_on(client.create_list(&core::kvstore::CreateListParams {
                key: hash_require_string(&opts, "key")?,
                items: hash_require_vec_string(&opts, "items")?,
            }))
            .map_err(map_err)
    }

    fn get_lists(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["limit", "cursor"])?;
        let client = self.inner.clone();
        runtime()
            .block_on(client.get_lists(&core::kvstore::GetListsParams {
                limit: hash_get_i64(&opts, "limit")?,
                cursor: hash_get_string(&opts, "cursor")?,
            }))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn get_list(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["key", "limit", "cursor"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        runtime()
            .block_on(client.get_list(
                &key,
                &core::kvstore::GetListParams {
                    limit: hash_get_i64(&opts, "limit")?,
                    cursor: hash_get_string(&opts, "cursor")?,
                },
            ))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn update_list(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key", "add_items", "remove_items"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        runtime()
            .block_on(client.update_list(
                &key,
                &core::kvstore::UpdateListParams {
                    add_items: hash_get_vec_string(&opts, "add_items")?,
                    remove_items: hash_get_vec_string(&opts, "remove_items")?,
                },
            ))
            .map_err(map_err)
    }

    fn add_list_item(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key", "item"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        runtime()
            .block_on(client.add_list_item(
                &key,
                &core::kvstore::AddListItemParams {
                    item: hash_require_string(&opts, "item")?,
                },
            ))
            .map_err(map_err)
    }

    fn list_contains_item(&self, opts: RHash) -> Result<magnus::Value, Error> {
        validate_keys(&opts, &["key", "item"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        let item = hash_require_string(&opts, "item")?;
        runtime()
            .block_on(client.list_contains_item(&key, &item))
            .map_err(map_err)
            .and_then(to_ruby)
    }

    fn delete_list_item(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key", "item"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        let item = hash_require_string(&opts, "item")?;
        runtime()
            .block_on(client.delete_list_item(&key, &item))
            .map_err(map_err)
    }

    fn delete_list(&self, opts: RHash) -> Result<(), Error> {
        validate_keys(&opts, &["key"])?;
        let client = self.inner.clone();
        let key = hash_require_string(&opts, "key")?;
        runtime()
            .block_on(client.delete_list(&key))
            .map_err(map_err)
    }
}

// ── Extension init ──────────────────────────────────────────────────────────

#[magnus::init(name = "quicknode_sdk")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("QuicknodeSdk")?;

    // Typed exception hierarchy — register before client classes so map_err
    // can use them from the first call onward.
    errors::init(ruby, &module)?;

    let native = module.define_module("Native")?;

    // ── SDK root ──────────────────────────────────────────────
    let sdk = native.define_class("SDK", ruby.class_object())?;
    sdk.define_singleton_method("from_env", function!(QuicknodeSdk::from_env, 0))?;
    sdk.define_method("admin", method!(QuicknodeSdk::admin, 0))?;
    sdk.define_method("streams", method!(QuicknodeSdk::streams, 0))?;
    sdk.define_method("webhooks", method!(QuicknodeSdk::webhooks, 0))?;
    sdk.define_method("kvstore", method!(QuicknodeSdk::kvstore, 0))?;

    // ── Admin ─────────────────────────────────────────────────
    let admin = native.define_class("Admin", ruby.class_object())?;
    admin.define_method("get_endpoints", method!(AdminApiClient::get_endpoints, 1))?;
    admin.define_method(
        "create_endpoint",
        method!(AdminApiClient::create_endpoint, 1),
    )?;
    admin.define_method("show_endpoint", method!(AdminApiClient::show_endpoint, 1))?;
    admin.define_method(
        "update_endpoint",
        method!(AdminApiClient::update_endpoint, 1),
    )?;
    admin.define_method(
        "archive_endpoint",
        method!(AdminApiClient::archive_endpoint, 1),
    )?;
    admin.define_method(
        "update_endpoint_status",
        method!(AdminApiClient::update_endpoint_status, 1),
    )?;
    admin.define_method("create_tag", method!(AdminApiClient::create_tag, 1))?;
    admin.define_method("delete_tag", method!(AdminApiClient::delete_tag, 1))?;
    admin.define_method("get_usage", method!(AdminApiClient::get_usage, 1))?;
    admin.define_method(
        "get_usage_by_endpoint",
        method!(AdminApiClient::get_usage_by_endpoint, 1),
    )?;
    admin.define_method(
        "get_usage_by_method",
        method!(AdminApiClient::get_usage_by_method, 1),
    )?;
    admin.define_method(
        "get_usage_by_chain",
        method!(AdminApiClient::get_usage_by_chain, 1),
    )?;
    admin.define_method(
        "get_endpoint_logs",
        method!(AdminApiClient::get_endpoint_logs, 1),
    )?;
    admin.define_method(
        "get_log_details",
        method!(AdminApiClient::get_log_details, 1),
    )?;
    admin.define_method(
        "get_security_options",
        method!(AdminApiClient::get_security_options, 1),
    )?;
    admin.define_method(
        "update_security_options",
        method!(AdminApiClient::update_security_options, 1),
    )?;
    admin.define_method("create_token", method!(AdminApiClient::create_token, 1))?;
    admin.define_method("delete_token", method!(AdminApiClient::delete_token, 1))?;
    admin.define_method(
        "create_referrer",
        method!(AdminApiClient::create_referrer, 1),
    )?;
    admin.define_method(
        "delete_referrer",
        method!(AdminApiClient::delete_referrer, 1),
    )?;
    admin.define_method("create_ip", method!(AdminApiClient::create_ip, 1))?;
    admin.define_method("delete_ip", method!(AdminApiClient::delete_ip, 1))?;
    admin.define_method(
        "create_domain_mask",
        method!(AdminApiClient::create_domain_mask, 1),
    )?;
    admin.define_method(
        "delete_domain_mask",
        method!(AdminApiClient::delete_domain_mask, 1),
    )?;
    admin.define_method("create_jwt", method!(AdminApiClient::create_jwt, 1))?;
    admin.define_method("delete_jwt", method!(AdminApiClient::delete_jwt, 1))?;
    admin.define_method(
        "create_request_filter",
        method!(AdminApiClient::create_request_filter, 1),
    )?;
    admin.define_method(
        "update_request_filter",
        method!(AdminApiClient::update_request_filter, 1),
    )?;
    admin.define_method(
        "delete_request_filter",
        method!(AdminApiClient::delete_request_filter, 1),
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
        method!(AdminApiClient::create_or_update_ip_custom_header, 1),
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
        method!(AdminApiClient::create_method_rate_limit, 1),
    )?;
    admin.define_method(
        "update_method_rate_limit",
        method!(AdminApiClient::update_method_rate_limit, 1),
    )?;
    admin.define_method(
        "delete_method_rate_limit",
        method!(AdminApiClient::delete_method_rate_limit, 1),
    )?;
    admin.define_method(
        "update_rate_limits",
        method!(AdminApiClient::update_rate_limits, 1),
    )?;
    admin.define_method(
        "get_rate_limits",
        method!(AdminApiClient::get_rate_limits, 1),
    )?;
    admin.define_method(
        "delete_rate_limit_override",
        method!(AdminApiClient::delete_rate_limit_override, 1),
    )?;
    admin.define_method(
        "get_endpoint_urls",
        method!(AdminApiClient::get_endpoint_urls, 1),
    )?;
    admin.define_method(
        "get_endpoint_metrics",
        method!(AdminApiClient::get_endpoint_metrics, 1),
    )?;
    admin.define_method(
        "get_account_metrics",
        method!(AdminApiClient::get_account_metrics, 1),
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
        method!(AdminApiClient::update_team_endpoints, 1),
    )?;
    admin.define_method(
        "invite_team_member",
        method!(AdminApiClient::invite_team_member, 1),
    )?;
    admin.define_method(
        "remove_team_member",
        method!(AdminApiClient::remove_team_member, 1),
    )?;
    admin.define_method(
        "resend_team_invite",
        method!(AdminApiClient::resend_team_invite, 1),
    )?;
    admin.define_method(
        "bulk_update_endpoint_status",
        method!(AdminApiClient::bulk_update_endpoint_status, 1),
    )?;
    admin.define_method("bulk_add_tag", method!(AdminApiClient::bulk_add_tag, 1))?;
    admin.define_method(
        "bulk_remove_tag",
        method!(AdminApiClient::bulk_remove_tag, 1),
    )?;
    admin.define_method("list_tags", method!(AdminApiClient::list_tags, 0))?;
    admin.define_method("rename_tag", method!(AdminApiClient::rename_tag, 1))?;
    admin.define_method(
        "delete_account_tag",
        method!(AdminApiClient::delete_account_tag, 1),
    )?;
    admin.define_method(
        "get_usage_by_tag",
        method!(AdminApiClient::get_usage_by_tag, 1),
    )?;
    admin.define_method(
        "get_endpoint_security",
        method!(AdminApiClient::get_endpoint_security, 1),
    )?;

    // ── DestinationAttributes ─────────────────────────────────
    let dest_attrs = module.define_class("DestinationAttributes", ruby.class_object())?;
    dest_attrs.define_singleton_method("webhook", function!(DestinationAttributes::webhook, 1))?;
    dest_attrs.define_singleton_method("s3", function!(DestinationAttributes::s3, 1))?;
    dest_attrs.define_singleton_method("azure", function!(DestinationAttributes::azure, 1))?;
    dest_attrs
        .define_singleton_method("postgres", function!(DestinationAttributes::postgres, 1))?;
    dest_attrs.define_singleton_method("kafka", function!(DestinationAttributes::kafka, 1))?;

    // ── Streams ───────────────────────────────────────────────
    let streams = native.define_class("Streams", ruby.class_object())?;
    streams.define_method("create_stream", method!(StreamsApiClient::create_stream, 1))?;
    streams.define_method("list_streams", method!(StreamsApiClient::list_streams, 1))?;
    streams.define_method(
        "delete_all_streams",
        method!(StreamsApiClient::delete_all_streams, 0),
    )?;
    streams.define_method("get_stream", method!(StreamsApiClient::get_stream, 1))?;
    streams.define_method("update_stream", method!(StreamsApiClient::update_stream, 1))?;
    streams.define_method("delete_stream", method!(StreamsApiClient::delete_stream, 1))?;
    streams.define_method(
        "activate_stream",
        method!(StreamsApiClient::activate_stream, 1),
    )?;
    streams.define_method("pause_stream", method!(StreamsApiClient::pause_stream, 1))?;
    streams.define_method("test_filter", method!(StreamsApiClient::test_filter, 1))?;
    streams.define_method(
        "get_enabled_count",
        method!(StreamsApiClient::get_enabled_count, 1),
    )?;

    // ── Webhooks ──────────────────────────────────────────────
    let webhooks = native.define_class("Webhooks", ruby.class_object())?;
    webhooks.define_method(
        "list_webhooks",
        method!(WebhooksApiClient::list_webhooks, 1),
    )?;
    webhooks.define_method(
        "delete_all_webhooks",
        method!(WebhooksApiClient::delete_all_webhooks, 0),
    )?;
    webhooks.define_method("get_webhook", method!(WebhooksApiClient::get_webhook, 1))?;
    webhooks.define_method(
        "update_webhook",
        method!(WebhooksApiClient::update_webhook, 1),
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
        method!(WebhooksApiClient::activate_webhook, 1),
    )?;
    webhooks.define_method(
        "get_enabled_count",
        method!(WebhooksApiClient::get_enabled_count, 0),
    )?;
    webhooks.define_method(
        "create_webhook_from_template",
        method!(WebhooksApiClient::create_webhook_from_template, 1),
    )?;
    webhooks.define_method(
        "update_webhook_template",
        method!(WebhooksApiClient::update_webhook_template, 1),
    )?;

    // ── KvStore ───────────────────────────────────────────────
    let kvstore = native.define_class("KvStore", ruby.class_object())?;
    kvstore.define_method("create_set", method!(KvStoreApiClient::create_set, 1))?;
    kvstore.define_method("get_sets", method!(KvStoreApiClient::get_sets, 1))?;
    kvstore.define_method("get_set", method!(KvStoreApiClient::get_set, 1))?;
    kvstore.define_method("bulk_sets", method!(KvStoreApiClient::bulk_sets, 1))?;
    kvstore.define_method("delete_set", method!(KvStoreApiClient::delete_set, 1))?;
    kvstore.define_method("create_list", method!(KvStoreApiClient::create_list, 1))?;
    kvstore.define_method("get_lists", method!(KvStoreApiClient::get_lists, 1))?;
    kvstore.define_method("get_list", method!(KvStoreApiClient::get_list, 1))?;
    kvstore.define_method("update_list", method!(KvStoreApiClient::update_list, 1))?;
    kvstore.define_method("add_list_item", method!(KvStoreApiClient::add_list_item, 1))?;
    kvstore.define_method(
        "list_contains_item",
        method!(KvStoreApiClient::list_contains_item, 1),
    )?;
    kvstore.define_method(
        "delete_list_item",
        method!(KvStoreApiClient::delete_list_item, 1),
    )?;
    kvstore.define_method("delete_list", method!(KvStoreApiClient::delete_list, 1))?;

    Ok(())
}
