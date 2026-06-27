#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// An account-level tag, shared across endpoints.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTag {
    /// Tag identifier.
    pub id: i32,
    /// Tag label.
    pub label: String,
    /// Number of endpoints the tag is applied to.
    pub usage_count: i32,
}

/// Inner data wrapper for `list_tags`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTagsData {
    /// Tags on the account.
    #[serde(default)]
    pub tags: Vec<AccountTag>,
}

/// Response from `list_tags`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTagsResponse {
    /// Account tags payload.
    pub data: Option<ListTagsData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `rename_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct RenameTagRequest {
    /// New label for the tag.
    pub label: String,
}

/// Response from `rename_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameTagResponse {
    /// The renamed tag.
    pub data: Option<AccountTag>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Inner data for `delete_account_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAccountTagData {
    /// `true` when the tag was deleted.
    pub success: bool,
}

/// Response from `delete_account_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteAccountTagResponse {
    /// Deletion result.
    pub data: Option<DeleteAccountTagData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
