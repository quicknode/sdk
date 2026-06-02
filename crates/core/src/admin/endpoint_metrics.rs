#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Deserializer, Serialize};

// The metrics endpoints return `tag` as either a plain string (single-axis
// series like `"total"` or `"p95"`) or a tuple like `["network", "mainnet"]`
// (multi-axis series). Normalise both to a `Vec<String>` so callers always
// see an array.
fn tag_as_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(s) => Ok(vec![s]),
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => Ok(s),
                other => Err(D::Error::custom(format!(
                    "expected string in tag array, got {other}"
                ))),
            })
            .collect(),
        serde_json::Value::Null => Ok(Vec::new()),
        other => Err(D::Error::custom(format!(
            "expected string or array of strings for tag, got {other}"
        ))),
    }
}

/// Parameters for `get_endpoint_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetEndpointMetricsRequest {
    /// Time period (`hour`, `day`, `week`, or `month`).
    pub period: String,
    /// Metric name (e.g. `method_calls_over_time`, `response_status_breakdown`).
    pub metric: String,
}

/// Parameters for `get_account_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetAccountMetricsRequest {
    /// Time period (`hour`, `day`, `week`, or `month`).
    pub period: String,
    /// Metric name (e.g. `method_calls_over_time`, `credits_over_time`).
    pub metric: String,
    /// Optional percentile for latency metrics (e.g. `p50`, `p95`, `p99`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentile: Option<String>,
}

/// A single metric series, consisting of a descriptive tag and timestamped data points.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetric {
    /// Data points, each as `[timestamp, value]`.
    pub data: Vec<Vec<i64>>,
    /// Tag identifying the series. Single-axis metrics return a one-element
    /// vector (e.g. `["total"]`, `["p95"]`); multi-axis metrics return the
    /// key/value pair (e.g. `["network", "arbitrum-mainnet"]`).
    #[serde(deserialize_with = "tag_as_vec")]
    pub tag: Vec<String>,
}

/// Response from `get_endpoint_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEndpointMetricsResponse {
    /// Metric series returned for the endpoint.
    #[serde(default)]
    pub data: Vec<EndpointMetric>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Response from `get_account_metrics`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountMetricsResponse {
    /// Metric series returned for the account.
    #[serde(default)]
    pub data: Vec<EndpointMetric>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::EndpointMetric;

    #[test]
    fn tag_deserializes_from_string() {
        let m: EndpointMetric =
            serde_json::from_str(r#"{"data": [[1, 2]], "tag": "total"}"#).unwrap();
        assert_eq!(m.tag, vec!["total".to_string()]);
    }

    #[test]
    fn tag_deserializes_from_tuple() {
        let m: EndpointMetric =
            serde_json::from_str(r#"{"data": [[1, 2]], "tag": ["network", "arbitrum-mainnet"]}"#)
                .unwrap();
        assert_eq!(
            m.tag,
            vec!["network".to_string(), "arbitrum-mainnet".to_string()]
        );
    }

    #[test]
    fn tag_deserializes_from_null() {
        let m: EndpointMetric = serde_json::from_str(r#"{"data": [[1, 2]], "tag": null}"#).unwrap();
        assert!(m.tag.is_empty());
    }

    #[test]
    fn tag_rejects_mixed_array() {
        let err =
            serde_json::from_str::<EndpointMetric>(r#"{"data": [], "tag": ["x", 5]}"#).unwrap_err();
        assert!(err.to_string().contains("expected string in tag array"));
    }

    #[test]
    fn tag_rejects_object() {
        let err = serde_json::from_str::<EndpointMetric>(r#"{"data": [], "tag": {"k": "v"}}"#)
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("expected string or array of strings for tag"));
    }
}

// ── Python conveniences (__repr__, to_dict) ───────────────────────────────
// Generated by the `python_repr_dict!` macro (see crates/core/src/python_macros.rs).

#[cfg(feature = "python")]
mod python_repr_impls {
    use super::*;
    crate::python_repr_dict!(GetEndpointMetricsRequest);
    crate::python_repr_dict!(GetAccountMetricsRequest);
    crate::python_repr_dict!(EndpointMetric);
    crate::python_repr_dict!(GetEndpointMetricsResponse);
    crate::python_repr_dict!(GetAccountMetricsResponse);
}
