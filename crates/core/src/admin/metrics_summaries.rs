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

use super::usage_summaries::{deserialize_dimension, push_array, summary_window_query};

/// Parameters for `get_traffic_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetTrafficSummaryRequest {
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
    /// Grouping dimensions. Allowed values: `method`, `endpoint`.
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

/// Full-window totals for `get_traffic_summary`. Numbers can be fractional.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSummaryTotals {
    /// Successful requests across the complete window.
    pub successful_requests: f64,
    /// Bytes across the complete window.
    pub bytes: f64,
    /// Request credits across the complete window.
    pub request_credits: f64,
    /// Bytes credits across the complete window.
    pub bytes_credits: f64,
    /// Credits across the complete window.
    pub credits: f64,
    /// Active methods across the complete window.
    pub active_methods: f64,
}

/// One grouped row from `get_traffic_summary`. Dimension fields are absent unless
/// selected; `Some(None)` represents an explicitly null dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSummaryRow {
    /// Successful requests for this group.
    pub successful_requests: Option<f64>,
    /// Bytes for this group.
    pub bytes: f64,
    /// Request credits for this group.
    pub request_credits: f64,
    /// Bytes credits for this group.
    pub bytes_credits: f64,
    /// Credits for this group.
    pub credits: f64,
    /// Active methods for this group.
    pub active_methods: f64,
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
    pub endpoint: Option<Option<String>>,
}

/// Summary window and grouped results. Totals cover the full window on every page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSummaryData {
    /// Returned window start; the server may align it to billing boundaries.
    #[serde(rename = "from")]
    pub from_time: String,
    /// Returned window end; retain the server value without rounding.
    #[serde(rename = "to")]
    pub to_time: String,
    /// Totals for the complete window, independent of page size.
    pub totals: TrafficSummaryTotals,
    /// Grouped results, omitted when grouping was not requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<TrafficSummaryRow>>,
    /// Cursor for the next page, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from `get_traffic_summary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTrafficSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: TrafficSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetTrafficSummaryRequest {
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
    /// Returns traffic summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_traffic_summary(
        &self,
        params: &GetTrafficSummaryRequest,
    ) -> Result<GetTrafficSummaryResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join("metrics/traffic/summary")?;
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

/// Parameters for `get_errors_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetErrorsSummaryRequest {
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
    /// Grouping dimensions. Allowed values: `code`, `endpoint`, `network`, `method`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Endpoint external UUIDs to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<String>>,
    /// Network names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    /// RPC method names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    /// RPC error codes represented as strings; requires RPC kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codes: Option<Vec<String>>,
    /// HTTP status codes represented as strings; requires HTTP kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_codes: Option<Vec<String>>,
    /// Error kind (`rpc` or `http`); method filters/grouping require RPC kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Full-window totals for `get_errors_summary`. Numbers can be fractional.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorsSummaryTotals {
    /// Rpc errors across the complete window.
    pub rpc_errors: f64,
    /// Http errors across the complete window.
    pub http_errors: f64,
}

/// One grouped row from `get_errors_summary`. Dimension fields are absent unless
/// selected; `Some(None)` represents an explicitly null dimension.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorsSummaryRow {
    /// Kind for this group.
    pub kind: String,
    /// Count for this group.
    pub count: f64,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub code: Option<Option<String>>,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub endpoint: Option<Option<String>>,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub network: Option<Option<String>>,
    /// Group value, preserving omitted versus explicitly null fields.
    #[serde(
        default,
        deserialize_with = "deserialize_dimension",
        skip_serializing_if = "Option::is_none"
    )]
    pub method: Option<Option<String>>,
}

/// Summary window and grouped results. Totals cover the full window on every page.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorsSummaryData {
    /// Returned window start; the server may align it to billing boundaries.
    #[serde(rename = "from")]
    #[cfg_attr(feature = "node", napi(js_name = "from"))]
    pub from_time: String,
    /// Returned window end; retain the server value without rounding.
    #[serde(rename = "to")]
    #[cfg_attr(feature = "node", napi(js_name = "to"))]
    pub to_time: String,
    /// Totals for the complete window, independent of page size.
    pub totals: ErrorsSummaryTotals,
    /// Grouped results, omitted when grouping was not requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<ErrorsSummaryRow>>,
    /// Cursor for the next page, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Response from `get_errors_summary`.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetErrorsSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: ErrorsSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetErrorsSummaryRequest {
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut query = summary_window_query(
            &self.last,
            &self.from_time,
            &self.to_time,
            &self.cursor,
            self.limit,
        );
        push_array(&mut query, "group_by[]", &self.group_by);
        push_array(&mut query, "endpoints[]", &self.endpoints);
        push_array(&mut query, "networks[]", &self.networks);
        push_array(&mut query, "methods[]", &self.methods);
        push_array(&mut query, "codes[]", &self.codes);
        push_array(&mut query, "status_codes[]", &self.status_codes);
        if let Some(value) = &self.kind {
            query.push(("kind", value.clone()));
        }
        query
    }
}

impl AdminApiClient {
    /// Returns error summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_errors_summary(
        &self,
        params: &GetErrorsSummaryRequest,
    ) -> Result<GetErrorsSummaryResponse, SdkError> {
        let url = self
            .config
            .admin()
            .base_url
            .join("metrics/errors/summary")?;
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

/// Parameters for `get_endpoint_errors_summary`. A first page requires `last`, or both
/// `from` and `to`; a continuation can contain only `cursor`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEndpointErrorsSummaryRequest {
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
    /// Grouping dimensions. Allowed values: `code`, `network`, `method`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<Vec<String>>,
    /// Network names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    /// RPC method names to include.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub methods: Option<Vec<String>>,
    /// RPC error codes represented as strings; requires RPC kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codes: Option<Vec<String>>,
    /// HTTP status codes represented as strings; requires HTTP kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_codes: Option<Vec<String>>,
    /// Error kind (`rpc` or `http`); method filters/grouping require RPC kind.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Response from `get_endpoint_errors_summary`.
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEndpointErrorsSummaryResponse {
    /// Summary window, totals, and optional grouped results.
    pub data: ErrorsSummaryData,
    /// Error returned by the API, when present.
    pub error: Option<String>,
}

impl GetEndpointErrorsSummaryRequest {
    fn query(&self) -> Vec<(&'static str, String)> {
        let mut query = summary_window_query(
            &self.last,
            &self.from_time,
            &self.to_time,
            &self.cursor,
            self.limit,
        );
        push_array(&mut query, "group_by[]", &self.group_by);
        push_array(&mut query, "networks[]", &self.networks);
        push_array(&mut query, "methods[]", &self.methods);
        push_array(&mut query, "codes[]", &self.codes);
        push_array(&mut query, "status_codes[]", &self.status_codes);
        if let Some(value) = &self.kind {
            query.push(("kind", value.clone()));
        }
        query
    }
}

impl AdminApiClient {
    /// Returns endpoint error summary totals and optional groups for a time window.
    /// Call again with `cursor` for continuation; totals still cover the full window.
    pub async fn get_endpoint_errors_summary(
        &self,
        id: &str,
        params: &GetEndpointErrorsSummaryRequest,
    ) -> Result<GetEndpointErrorsSummaryResponse, SdkError> {
        let mut url = self.config.admin().base_url.clone();
        url.path_segments_mut()
            .map_err(|_| {
                SdkError::Config("Admin base URL does not support path segments".to_string())
            })?
            .pop_if_empty()
            .extend(["endpoints", id, "metrics", "errors", "summary"]);
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::{
        GetEndpointErrorsSummaryRequest, GetErrorsSummaryRequest, GetTrafficSummaryRequest,
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
    async fn get_traffic_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "successful_requests": 7.5,
                    "bytes": 7.5,
                    "request_credits": 7.5,
                    "bytes_credits": 7.5,
                    "credits": 7.5,
                    "active_methods": 7.5
                },
                "breakdown": [
                    {
                        "successful_requests": null,
                        "bytes": 7.5,
                        "request_credits": 7.5,
                        "bytes_credits": 7.5,
                        "credits": 7.5,
                        "active_methods": 7.5,
                        "method": null,
                        "endpoint": "fixture-group"
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/metrics/traffic/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_traffic_summary(&GetTrafficSummaryRequest {
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
            .and(path("/metrics/traffic/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_traffic_summary(&GetTrafficSummaryRequest {
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
    async fn get_traffic_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/metrics/traffic/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_traffic_summary(&GetTrafficSummaryRequest::default())
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
    async fn get_errors_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "rpc_errors": 7.5,
                    "http_errors": 7.5
                },
                "breakdown": [
                    {
                        "kind": "rpc",
                        "count": 7.5,
                        "code": null,
                        "endpoint": "fixture-group"
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/metrics/errors/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_errors_summary(&GetErrorsSummaryRequest {
                last: Some("1h".to_string()),
                limit: Some(2),
                group_by: Some(vec!["code".to_string(), "endpoint".to_string()]),
                endpoints: Some(vec![
                    "fixture-uuid-one".to_string(),
                    "fixture-uuid-two".to_string(),
                ]),
                networks: Some(vec!["mainnet".to_string(), "sepolia".to_string()]),
                methods: Some(vec!["eth_blockNumber".to_string(), "eth_call".to_string()]),
                codes: Some(vec!["-32000".to_string(), "-32601".to_string()]),
                kind: Some("rpc".to_string()),
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
            ("group_by[]".to_string(), "code".to_string()),
            ("group_by[]".to_string(), "endpoint".to_string()),
            ("endpoints[]".to_string(), "fixture-uuid-one".to_string()),
            ("endpoints[]".to_string(), "fixture-uuid-two".to_string()),
            ("networks[]".to_string(), "mainnet".to_string()),
            ("networks[]".to_string(), "sepolia".to_string()),
            ("methods[]".to_string(), "eth_blockNumber".to_string()),
            ("methods[]".to_string(), "eth_call".to_string()),
            ("codes[]".to_string(), "-32000".to_string()),
            ("codes[]".to_string(), "-32601".to_string()),
            ("kind".to_string(), "rpc".to_string()),
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
            .and(path("/metrics/errors/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_errors_summary(&GetErrorsSummaryRequest {
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
    async fn get_errors_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/metrics/errors/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_errors_summary(&GetErrorsSummaryRequest::default())
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
    async fn get_endpoint_errors_summary_preserves_fields_and_cursor_queries() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let fixture = json!({
            "data": {
                "from": "2026-10-01T00:00:00Z",
                "to": "2026-11-01T00:00:00Z",
                "totals": {
                    "rpc_errors": 7.5,
                    "http_errors": 7.5
                },
                "breakdown": [
                    {
                        "kind": "rpc",
                        "count": 7.5,
                        "code": null,
                        "network": "fixture-group"
                    }
                ],
                "next_cursor": "opaque+/cursor=="
            },
            "error": null
        });
        Mock::given(method("GET"))
            .and(path("/endpoints/123/metrics/errors/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&fixture))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_endpoint_errors_summary(
                "123",
                &GetEndpointErrorsSummaryRequest {
                    last: Some("1h".to_string()),
                    limit: Some(2),
                    group_by: Some(vec!["code".to_string(), "network".to_string()]),
                    networks: Some(vec!["mainnet".to_string(), "sepolia".to_string()]),
                    methods: Some(vec!["eth_blockNumber".to_string(), "eth_call".to_string()]),
                    codes: Some(vec!["-32000".to_string(), "-32601".to_string()]),
                    kind: Some("rpc".to_string()),
                    ..Default::default()
                },
            )
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
            ("group_by[]".to_string(), "code".to_string()),
            ("group_by[]".to_string(), "network".to_string()),
            ("networks[]".to_string(), "mainnet".to_string()),
            ("networks[]".to_string(), "sepolia".to_string()),
            ("methods[]".to_string(), "eth_blockNumber".to_string()),
            ("methods[]".to_string(), "eth_call".to_string()),
            ("codes[]".to_string(), "-32000".to_string()),
            ("codes[]".to_string(), "-32601".to_string()),
            ("kind".to_string(), "rpc".to_string()),
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
            .and(path("/endpoints/123/metrics/errors/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&next_page))
            .expect(1)
            .mount(&server)
            .await;
        let next = sdk
            .admin
            .get_endpoint_errors_summary(
                "123",
                &GetEndpointErrorsSummaryRequest {
                    cursor: response.data.next_cursor,
                    ..Default::default()
                },
            )
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
    async fn get_endpoint_errors_summary_retains_api_errors() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for status in [400, 403] {
            Mock::given(method("GET"))
                .and(path("/endpoints/123/metrics/errors/summary"))
                .respond_with(ResponseTemplate::new(status).set_body_string("fixture API error"))
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_endpoint_errors_summary("123", &GetEndpointErrorsSummaryRequest::default())
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
    async fn error_kinds_and_incompatible_filters_reach_the_api() {
        let server = MockServer::start().await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        for (kind, filters) in [("rpc", "status_codes[]"), ("http", "codes[]")] {
            Mock::given(method("GET"))
                .and(path("/metrics/errors/summary"))
                .respond_with(
                    ResponseTemplate::new(400).set_body_string("incompatible error filters"),
                )
                .expect(1)
                .mount(&server)
                .await;
            let error = sdk
                .admin
                .get_errors_summary(&GetErrorsSummaryRequest {
                    last: Some("1h".to_string()),
                    kind: Some(kind.to_string()),
                    codes: (kind == "http").then(|| vec!["-32000".to_string()]),
                    status_codes: (kind == "rpc").then(|| vec!["429".to_string()]),
                    ..Default::default()
                })
                .await
                .unwrap_err();
            assert!(matches!(error, SdkError::Api { status, .. } if status.as_u16() == 400));
            let requests = server.received_requests().await.unwrap();
            let query = requests[0].url.query_pairs().collect::<Vec<_>>();
            assert!(query
                .iter()
                .any(|(key, value)| key == "kind" && value == kind));
            assert!(query.iter().any(|(key, _)| key == filters));
            server.verify().await;
            server.reset().await;
        }
        Mock::given(method("GET"))
            .and(path("/metrics/errors/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": {
                    "from": "2026-10-01T00:00:00Z", "to": "2026-11-01T00:00:00Z",
                    "totals": {"rpc_errors": 0.0, "http_errors": 7.5},
                    "breakdown": [{"kind": "http", "count": 7.5, "code": "429"}]
                }, "error": null
            })))
            .expect(1)
            .mount(&server)
            .await;
        let response = sdk
            .admin
            .get_errors_summary(&GetErrorsSummaryRequest {
                last: Some("1h".to_string()),
                kind: Some("http".to_string()),
                group_by: Some(vec!["code".to_string()]),
                status_codes: Some(vec!["429".to_string()]),
                ..Default::default()
            })
            .await
            .unwrap();
        let row = &response.data.breakdown.unwrap()[0];
        assert_eq!(row.kind, "http");
        assert_eq!(row.code, Some(Some("429".to_string())));
        assert!(row.method.is_none());
    }
}
