#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// The API credit cost of a single RPC method on a chain.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCredit {
    /// RPC method name (e.g. `eth_chainId`).
    pub method: String,
    /// Number of API credits the method costs.
    pub credits: i64,
}

/// Response from `get_api_credits`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetApiCreditsResponse {
    /// Per-method API credit costs for the chain, when the request succeeded.
    /// `None` for an unknown chain slug.
    pub data: Option<Vec<ApiCredit>>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
