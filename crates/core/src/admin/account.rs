#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// The account's current subscription.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountSubscription {
    /// Plan name (e.g. `Accelerate`).
    pub plan_name: Option<String>,
    /// Subscription status (e.g. `active`).
    pub status: Option<String>,
    /// Billing interval (e.g. `monthly`).
    pub interval: Option<String>,
}

/// Details about a Quicknode account.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Numeric account id.
    pub id: i64,
    /// Account name.
    pub name: String,
    /// ISO 8601 timestamp of when the account was created.
    pub created_at: String,
    /// Billing version (e.g. `v6`).
    pub billing_version: Option<String>,
    /// The account's current subscription, when present.
    pub subscription: Option<AccountSubscription>,
}

/// Response from `account_info`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfoResponse {
    /// Account details, when the request succeeded.
    pub data: Option<AccountInfo>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
