#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// Parameters for `bulk_update_endpoint_status`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkUpdateEndpointStatusRequest {
    /// Endpoint ids to update.
    pub ids: Vec<String>,
    /// Target status (`active` or `paused`).
    pub status: String,
}

/// Per-endpoint result within a bulk response.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    /// Endpoint id the result refers to.
    pub id: String,
    /// Whether the operation succeeded for this endpoint.
    pub success: bool,
}

/// Summary data for a `bulk_update_endpoint_status` response.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateEndpointStatusData {
    /// Total number of endpoints processed.
    pub total: i32,
    /// Number successfully updated.
    pub updated_count: i32,
    /// Number that failed.
    pub failed_count: i32,
    /// Per-endpoint outcomes.
    #[serde(default)]
    pub results: Vec<BulkOperationResult>,
}

/// Response from `bulk_update_endpoint_status`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkUpdateEndpointStatusResponse {
    /// Bulk update summary.
    pub data: Option<BulkUpdateEndpointStatusData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `bulk_add_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkAddTagRequest {
    /// Endpoint ids to tag.
    pub ids: Vec<String>,
    /// Label of the tag to apply (created if it doesn't exist). Maximum 25 characters.
    pub label: String,
}

/// Tag reference returned on bulk tag operations.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTag {
    /// Tag identifier.
    pub tag_id: i32,
    /// Tag label.
    pub label: String,
}

/// Summary data for a `bulk_add_tag` response.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAddTagData {
    /// Total number of endpoints processed.
    pub total: i32,
    /// Number successfully tagged.
    pub updated_count: i32,
    /// Number that failed.
    pub failed_count: i32,
    /// Per-endpoint outcomes.
    #[serde(default)]
    pub results: Vec<BulkOperationResult>,
    /// The tag that was applied.
    pub tag: BulkTag,
}

/// Response from `bulk_add_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAddTagResponse {
    /// Bulk add-tag summary.
    pub data: Option<BulkAddTagData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Parameters for `bulk_remove_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, Serialize)]
pub struct BulkRemoveTagRequest {
    /// Endpoint ids to untag.
    pub ids: Vec<String>,
    /// Tag to remove.
    pub tag_id: i32,
}

/// Summary data for a `bulk_remove_tag` response.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRemoveTagData {
    /// Total number of endpoints processed.
    pub total: i32,
    /// Number successfully updated.
    pub updated_count: i32,
    /// Number that failed.
    pub failed_count: i32,
    /// Per-endpoint outcomes.
    #[serde(default)]
    pub results: Vec<BulkOperationResult>,
}

/// Response from `bulk_remove_tag`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRemoveTagResponse {
    /// Bulk remove-tag summary.
    pub data: Option<BulkRemoveTagData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}
