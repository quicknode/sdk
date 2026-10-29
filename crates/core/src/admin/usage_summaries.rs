#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

use super::AdminApiClient;
use crate::errors::SdkError;

/// Parameters for `get_usage_credits_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetUsageCreditsSummaryRequest {
    /// Relative window, such as `1h`; use a positive duration in s, m, h, or d.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// Window start as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: Option<String>,
    /// Window end as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: Option<String>,
    /// Opaque cursor returned by the same route and original query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Page size from 1 through 1000; the API defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

/// Full-window totals for `get_usage_credits_summary`. Numbers can be fractional.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCreditsSummaryTotals {
    /// Credits across the complete window.
    pub credits: f64,
}

/// Credits row supported by the response schema. The credits request has no grouping option.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCreditsSummaryRow {
    /// Credits for this group.
    pub credits: f64,
}

/// Summary window and grouped results. Totals cover the full window on every page.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCreditsSummaryData {
    /// Returned window start; the server may align it to billing boundaries.
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: String,
    /// Returned window end; retain the server value without rounding.
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: String,
    /// Totals for the complete window, independent of page size.
    pub totals: UsageCreditsSummaryTotals,
    /// Grouped results, omitted when grouping was not requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<UsageCreditsSummaryRow>>,
    /// Cursor for the next page, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from `get_usage_credits_summary`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageCreditsSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: UsageCreditsSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetUsageCreditsSummaryRequest {
    fn query(&self) -> Vec<(&'static str, String)> {
        summary_window_query(
            &self.last,
            &self.from_time,
            &self.to_time,
            &self.cursor,
            self.limit,
        )
    }
}

impl AdminApiClient {
    /// Returns credit usage summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_usage_credits_summary(
        &self,
        params: &GetUsageCreditsSummaryRequest,
    ) -> Result<GetUsageCreditsSummaryResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/credits/summary")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&params.query())
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

/// Parameters for `get_usage_rpc_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetUsageRpcSummaryRequest {
    /// Relative window, such as `1h`; use a positive duration in s, m, h, or d.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// Window start as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: Option<String>,
    /// Window end as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: Option<String>,
    /// Opaque cursor returned by the same route and original query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Page size from 1 through 1000; the API defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Grouping dimensions. Allowed values: `method`, `endpoint`, `chain`, `network`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Endpoint subdomains to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_names: Option<Vec<String>>,
    /// Chain names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chains: Option<Vec<String>>,
    /// Network names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    /// RPC method names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    /// Numeric account tag IDs represented as strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_ids: Option<Vec<String>>,
    /// Numeric account add-on IDs represented as strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addon_ids: Option<Vec<String>>,
}

/// Full-window totals for `get_usage_rpc_summary`. Numbers can be fractional.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRpcSummaryTotals {
    /// Requests across the complete window.
    pub requests: f64,
    /// Bytes across the complete window.
    pub bytes: f64,
    /// Rpc credits across the complete window.
    pub rpc_credits: f64,
    /// Data credits across the complete window.
    pub data_credits: f64,
    /// Credits across the complete window.
    pub credits: f64,
}

/// One grouped row from `get_usage_rpc_summary`. Dimension fields are absent unless
/// selected; `Some(None)` represents an explicitly null dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRpcSummaryRow {
    /// Requests for this group.
    pub requests: Option<f64>,
    /// Bytes for this group.
    pub bytes: f64,
    /// Rpc credits for this group.
    pub rpc_credits: f64,
    /// Data credits for this group.
    pub data_credits: f64,
    /// Credits for this group.
    pub credits: f64,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub method: Option<Option<String>>,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoint: Option<Option<i64>>,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub chain: Option<Option<String>>,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub network: Option<Option<String>>,
}

/// Summary window and grouped results. Totals cover the full window on every page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRpcSummaryData {
    /// Returned window start; the server may align it to billing boundaries.
    #[serde(rename = "from")]
    pub from_time: String,
    /// Returned window end; retain the server value without rounding.
    #[serde(rename = "to")]
    pub to_time: String,
    /// Totals for the complete window, independent of page size.
    pub totals: UsageRpcSummaryTotals,
    /// Grouped results, omitted when grouping was not requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<UsageRpcSummaryRow>>,
    /// Cursor for the next page, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from `get_usage_rpc_summary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageRpcSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: UsageRpcSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetUsageRpcSummaryRequest {
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut query = summary_window_query(
            &self.last,
            &self.from_time,
            &self.to_time,
            &self.cursor,
            self.limit,
        );
        push_array(&mut query, "group_by[]", &self.group_by);
        push_array(&mut query, "endpoint_names[]", &self.endpoint_names);
        push_array(&mut query, "chains[]", &self.chains);
        push_array(&mut query, "networks[]", &self.networks);
        push_array(&mut query, "methods[]", &self.methods);
        push_array(&mut query, "tag_ids[]", &self.tag_ids);
        push_array(&mut query, "addon_ids[]", &self.addon_ids);
        query
    }
}

impl AdminApiClient {
    /// Returns RPC usage summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_usage_rpc_summary(
        &self,
        params: &GetUsageRpcSummaryRequest,
    ) -> Result<GetUsageRpcSummaryResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/rpc/summary")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&params.query())
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

/// Parameters for `get_usage_streams_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetUsageStreamsSummaryRequest {
    /// Relative window, such as `1h`; use a positive duration in s, m, h, or d.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// Window start as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: Option<String>,
    /// Window end as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: Option<String>,
    /// Opaque cursor returned by the same route and original query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Page size from 1 through 1000; the API defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Grouping dimensions. Allowed values: `stream`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Stream identifier to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_id: Option<String>,
}

/// Full-window totals for `get_usage_streams_summary`. Numbers can be fractional.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStreamsSummaryTotals {
    /// Blocks across the complete window.
    pub blocks: f64,
    /// Bytes across the complete window.
    pub bytes: f64,
    /// Block credits across the complete window.
    pub block_credits: f64,
}

/// One grouped row from `get_usage_streams_summary`. Dimension fields are absent unless
/// selected; `Some(None)` represents an explicitly null dimension.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStreamsSummaryRow {
    /// Blocks for this group.
    pub blocks: f64,
    /// Bytes for this group.
    pub bytes: f64,
    /// Block credits for this group.
    pub block_credits: f64,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub stream: Option<Option<String>>,
}

/// Summary window and grouped results. Totals cover the full window on every page.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStreamsSummaryData {
    /// Returned window start; the server may align it to billing boundaries.
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: String,
    /// Returned window end; retain the server value without rounding.
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: String,
    /// Totals for the complete window, independent of page size.
    pub totals: UsageStreamsSummaryTotals,
    /// Grouped results, omitted when grouping was not requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<UsageStreamsSummaryRow>>,
    /// Cursor for the next page, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from `get_usage_streams_summary`.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageStreamsSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: UsageStreamsSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetUsageStreamsSummaryRequest {
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut query = summary_window_query(
            &self.last,
            &self.from_time,
            &self.to_time,
            &self.cursor,
            self.limit,
        );
        push_array(&mut query, "group_by[]", &self.group_by);
        if let Some(value) = &self.stream_id {
            query.push(("stream_id", value.clone()));
        }
        query
    }
}

impl AdminApiClient {
    /// Returns Streams usage summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_usage_streams_summary(
        &self,
        params: &GetUsageStreamsSummaryRequest,
    ) -> Result<GetUsageStreamsSummaryResponse, SdkError> {
        let url = self.config.admin().base_url.join("usage/streams/summary")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&params.query())
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

/// Parameters for `get_usage_webhooks_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetUsageWebhooksSummaryRequest {
    /// Relative window, such as `1h`; use a positive duration in s, m, h, or d.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
    /// Window start as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: Option<String>,
    /// Window end as an ISO 8601 timestamp with timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: Option<String>,
    /// Opaque cursor returned by the same route and original query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Page size from 1 through 1000; the API defaults to 100.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Grouping dimensions. Allowed values: `webhook`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Webhook identifier to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_id: Option<String>,
}

/// Full-window totals for `get_usage_webhooks_summary`. Numbers can be fractional.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWebhooksSummaryTotals {
    /// Deliveries across the complete window.
    pub deliveries: f64,
    /// Credits across the complete window.
    pub credits: f64,
}

/// One grouped row from `get_usage_webhooks_summary`. Dimension fields are absent unless
/// selected; `Some(None)` represents an explicitly null dimension.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWebhooksSummaryRow {
    /// Deliveries for this group.
    pub deliveries: f64,
    /// Credits for this group.
    pub credits: f64,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub webhook: Option<Option<String>>,
}

/// Summary window and grouped results. Totals cover the full window on every page.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWebhooksSummaryData {
    /// Returned window start; the server may align it to billing boundaries.
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: String,
    /// Returned window end; retain the server value without rounding.
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: String,
    /// Totals for the complete window, independent of page size.
    pub totals: UsageWebhooksSummaryTotals,
    /// Grouped results, omitted when grouping was not requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<UsageWebhooksSummaryRow>>,
    /// Cursor for the next page, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from `get_usage_webhooks_summary`.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUsageWebhooksSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: UsageWebhooksSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetUsageWebhooksSummaryRequest {
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut query = summary_window_query(
            &self.last,
            &self.from_time,
            &self.to_time,
            &self.cursor,
            self.limit,
        );
        push_array(&mut query, "group_by[]", &self.group_by);
        if let Some(value) = &self.webhook_id {
            query.push(("webhook_id", value.clone()));
        }
        query
    }
}

impl AdminApiClient {
    /// Returns Webhooks usage summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_usage_webhooks_summary(
        &self,
        params: &GetUsageWebhooksSummaryRequest,
    ) -> Result<GetUsageWebhooksSummaryResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join("usage/webhooks/summary")?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .query(&params.query())
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

pub(super) fn summary_window_query(
    last: &Option<String>,
    from: &Option<String>,
    to: &Option<String>,
    cursor: &Option<String>,
    limit: Option<i32>,
) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    for (key, value) in [
        ("last", last),
        ("from", from),
        ("to", to),
        ("cursor", cursor),
    ] {
        if let Some(value) = value {
            query.push((key, value.clone()));
        }
    }
    if let Some(limit) = limit {
        query.push(("limit", limit.to_string()));
    }
    query
}

pub(super) fn push_array<T: ToString>(
    query: &mut Vec<(&'static str, String)>,
    key: &'static str,
    values: &Option<Vec<T>>,
) {
    if let Some(values) = values {
        query.extend(values.iter().map(|value| (key, value.to_string())));
    }
}

// Serde normally maps both missing and null fields to None. The outer option
// records presence so selected null groups survive serialization and bindings.
pub(super) fn deserialize_dimension<'de, D, T>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{
        GetUsageCreditsSummaryRequest, GetUsageRpcSummaryRequest, GetUsageStreamsSummaryRequest,
        GetUsageWebhooksSummaryRequest,
    };
    use crate::errors::SdkError;
    use crate::{AdminConfig, QuicknodeSdk, SdkFullConfig};
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuicknodeSdk {
        QuicknodeSdk::new(&SdkFullConfig {
            api_key: Some("fixture-key".to_string()),
            admin: Some(AdminConfig {
                base_url: Some(base_url),
            }),
            http: None,
            streams: None,
            webhooks: None,
            kvstore: None,
            sql: None,
            rpc: None,
        })
        .unwrap()
    }

    #[tokio::test]
    async fn get_usage_credits_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "credits": 7.5
                },
                "breakdown": [
                    {
                        "credits": 7.5
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/usage/credits/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_usage_credits_summary(&GetUsageCreditsSummaryRequest {
                last: Some("1h".to_string()),
                limit: Some(2),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&response).unwrap(), fixture);
        let requests = server.received_requests().await.unwrap();
        let mut actual = requests[0]
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        let mut expected = vec![
            ("last".to_string(), "1h".to_string()),
            ("limit".to_string(), "2".to_string()),
        ];
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
        assert!(requests[0].body.is_empty());
        server.verify().await;
        server.reset().await;

        let mut next_page = fixture.clone();
        next_page["data"]
            .as_object_mut()
            .unwrap()
            .remove("next_cursor");
        Mock::given(method("GET"))
            .and(path("/usage/credits/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_usage_credits_summary(&GetUsageCreditsSummaryRequest {
                cursor: response.data.next_cursor,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&next).unwrap(), next_page);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query_pairs().collect::<Vec<_>>(),
            vec![("cursor".into(), "opaque+/cursor==".into())]
        );
    }

    #[tokio::test]
    async fn get_usage_credits_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/usage/credits/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_usage_credits_summary(&GetUsageCreditsSummaryRequest::default())
                .await
                .unwrap_err();
            assert!(
                matches!(error, SdkError::Api { status: actual, ref body } if actual.as_u16() == status && body == "fixture API error")
            );
            server.verify().await;
            server.reset().await;
        }
    }

    #[tokio::test]
    async fn get_usage_rpc_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "requests": 7.5,
                    "bytes": 7.5,
                    "rpc_credits": 7.5,
                    "data_credits": 7.5,
                    "credits": 7.5
                },
                "breakdown": [
                    {
                        "requests": null,
                        "bytes": 7.5,
                        "rpc_credits": 7.5,
                        "data_credits": 7.5,
                        "credits": 7.5,
                        "method": null,
                        "endpoint": 4123
                    },
                    {
                        "requests": 1.5,
                        "bytes": 0.0,
                        "rpc_credits": 0.0,
                        "data_credits": 0.0,
                        "credits": 0.0,
                        "method": "eth_call",
                        "endpoint": null
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/usage/rpc/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_usage_rpc_summary(&GetUsageRpcSummaryRequest {
                last: Some("1h".to_string()),
                limit: Some(2),
                group_by: Some(vec!["method".to_string(), "endpoint".to_string()]),
                endpoint_names: Some(vec!["fixture-one".to_string(), "fixture-two".to_string()]),
                chains: Some(vec!["ethereum".to_string(), "base".to_string()]),
                networks: Some(vec!["mainnet".to_string(), "sepolia".to_string()]),
                methods: Some(vec!["eth_blockNumber".to_string(), "eth_call".to_string()]),
                tag_ids: Some(vec!["41".to_string(), "42".to_string()]),
                addon_ids: Some(vec!["51".to_string(), "52".to_string()]),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&response).unwrap(), fixture);
        let requests = server.received_requests().await.unwrap();
        let mut actual = requests[0]
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        let mut expected = vec![
            ("last".to_string(), "1h".to_string()),
            ("limit".to_string(), "2".to_string()),
            ("group_by[]".to_string(), "method".to_string()),
            ("group_by[]".to_string(), "endpoint".to_string()),
            ("endpoint_names[]".to_string(), "fixture-one".to_string()),
            ("endpoint_names[]".to_string(), "fixture-two".to_string()),
            ("chains[]".to_string(), "ethereum".to_string()),
            ("chains[]".to_string(), "base".to_string()),
            ("networks[]".to_string(), "mainnet".to_string()),
            ("networks[]".to_string(), "sepolia".to_string()),
            ("methods[]".to_string(), "eth_blockNumber".to_string()),
            ("methods[]".to_string(), "eth_call".to_string()),
            ("tag_ids[]".to_string(), "41".to_string()),
            ("tag_ids[]".to_string(), "42".to_string()),
            ("addon_ids[]".to_string(), "51".to_string()),
            ("addon_ids[]".to_string(), "52".to_string()),
        ];
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
        assert!(requests[0].body.is_empty());
        server.verify().await;
        server.reset().await;

        let mut next_page = fixture.clone();
        next_page["data"]
            .as_object_mut()
            .unwrap()
            .remove("next_cursor");
        Mock::given(method("GET"))
            .and(path("/usage/rpc/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_usage_rpc_summary(&GetUsageRpcSummaryRequest {
                cursor: response.data.next_cursor,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&next).unwrap(), next_page);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query_pairs().collect::<Vec<_>>(),
            vec![("cursor".into(), "opaque+/cursor==".into())]
        );
    }

    #[tokio::test]
    async fn get_usage_rpc_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/usage/rpc/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_usage_rpc_summary(&GetUsageRpcSummaryRequest::default())
                .await
                .unwrap_err();
            assert!(
                matches!(error, SdkError::Api { status: actual, ref body } if actual.as_u16() == status && body == "fixture API error")
            );
            server.verify().await;
            server.reset().await;
        }
    }

    #[tokio::test]
    async fn get_usage_streams_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "blocks": 7.5,
                    "bytes": 7.5,
                    "block_credits": 7.5
                },
                "breakdown": [
                    {
                        "blocks": 7.5,
                        "bytes": 7.5,
                        "block_credits": 7.5,
                        "stream": null
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/usage/streams/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_usage_streams_summary(&GetUsageStreamsSummaryRequest {
                last: Some("1h".to_string()),
                limit: Some(2),
                group_by: Some(vec!["stream".to_string()]),
                stream_id: Some("fixture-stream_id".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&response).unwrap(), fixture);
        let requests = server.received_requests().await.unwrap();
        let mut actual = requests[0]
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        let mut expected = vec![
            ("last".to_string(), "1h".to_string()),
            ("limit".to_string(), "2".to_string()),
            ("group_by[]".to_string(), "stream".to_string()),
            ("stream_id".to_string(), "fixture-stream_id".to_string()),
        ];
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
        assert!(requests[0].body.is_empty());
        server.verify().await;
        server.reset().await;

        let mut next_page = fixture.clone();
        next_page["data"]
            .as_object_mut()
            .unwrap()
            .remove("next_cursor");
        Mock::given(method("GET"))
            .and(path("/usage/streams/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_usage_streams_summary(&GetUsageStreamsSummaryRequest {
                cursor: response.data.next_cursor,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&next).unwrap(), next_page);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query_pairs().collect::<Vec<_>>(),
            vec![("cursor".into(), "opaque+/cursor==".into())]
        );
    }

    #[tokio::test]
    async fn get_usage_streams_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/usage/streams/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_usage_streams_summary(&GetUsageStreamsSummaryRequest::default())
                .await
                .unwrap_err();
            assert!(
                matches!(error, SdkError::Api { status: actual, ref body } if actual.as_u16() == status && body == "fixture API error")
            );
            server.verify().await;
            server.reset().await;
        }
    }

    #[tokio::test]
    async fn get_usage_webhooks_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "deliveries": 7.5,
                    "credits": 7.5
                },
                "breakdown": [
                    {
                        "deliveries": 7.5,
                        "credits": 7.5,
                        "webhook": null
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/usage/webhooks/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_usage_webhooks_summary(&GetUsageWebhooksSummaryRequest {
                last: Some("1h".to_string()),
                limit: Some(2),
                group_by: Some(vec!["webhook".to_string()]),
                webhook_id: Some("fixture-webhook_id".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&response).unwrap(), fixture);
        let requests = server.received_requests().await.unwrap();
        let mut actual = requests[0]
            .url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<Vec<_>>();
        let mut expected = vec![
            ("last".to_string(), "1h".to_string()),
            ("limit".to_string(), "2".to_string()),
            ("group_by[]".to_string(), "webhook".to_string()),
            ("webhook_id".to_string(), "fixture-webhook_id".to_string()),
        ];
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected);
        assert!(requests[0].body.is_empty());
        server.verify().await;
        server.reset().await;

        let mut next_page = fixture.clone();
        next_page["data"]
            .as_object_mut()
            .unwrap()
            .remove("next_cursor");
        Mock::given(method("GET"))
            .and(path("/usage/webhooks/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_usage_webhooks_summary(&GetUsageWebhooksSummaryRequest {
                cursor: response.data.next_cursor,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(serde_json::to_value(&next).unwrap(), next_page);
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query_pairs().collect::<Vec<_>>(),
            vec![("cursor".into(), "opaque+/cursor==".into())]
        );
    }

    #[tokio::test]
    async fn get_usage_webhooks_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/usage/webhooks/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_usage_webhooks_summary(&GetUsageWebhooksSummaryRequest::default())
                .await
                .unwrap_err();
            assert!(
                matches!(error, SdkError::Api { status: actual, ref body } if actual.as_u16() == status && body == "fixture API error")
            );
            server.verify().await;
            server.reset().await;
        }
    }

    #[tokio::test]
    async fn explicit_dates_do_not_add_a_window_or_grouping() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        Mock::given(method("GET"))
            .and(path("/usage/credits/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {"from": "2026-10-01T00:00:00Z", "to": "2026-11-01T00:00:00Z", "totals": {"credits": 7.5}},
                "error": null
            })))
            .expect(1).mount(&server).await;
        let response = sdk
            .admin
            .get_usage_credits_summary(&GetUsageCreditsSummaryRequest {
                from_time: Some("2026-10-05T12:00:00+01:00".to_string()),
                to_time: Some("2026-10-06T12:00:00+01:00".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(response.data.breakdown.is_none());
        assert!(response.data.next_cursor.is_none());
        assert_eq!(response.data.to_time, "2026-11-01T00:00:00Z");
        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query_pairs().collect::<Vec<_>>(),
            vec![
                ("from".into(), "2026-10-05T12:00:00+01:00".into()),
                ("to".into(), "2026-10-06T12:00:00+01:00".into()),
            ]
        );
    }
}
