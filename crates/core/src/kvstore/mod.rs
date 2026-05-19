use std::collections::HashMap;

#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{config::KvStoreConfig, errors::SdkError, SdkConfig};

// The KV index endpoints return `data: null` when the store has no entries.
// Map that to the default value so consumers get `[]` (or an empty struct) instead of a decode error.
fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
}

const KV_STORE_BASE_URL: &str = "https://api.quicknode.com/kv/rest/v1/";

// ── Resolved config ────────────────────────────────────────────────────────

pub(crate) struct ResolvedKvStoreConfig {
    pub(crate) base_url: reqwest::Url,
}

impl ResolvedKvStoreConfig {
    pub(crate) fn from_config(config: Option<&KvStoreConfig>) -> Result<Self, SdkError> {
        let url_str = config
            .and_then(|s| s.base_url.as_deref())
            .unwrap_or(KV_STORE_BASE_URL);
        let mut base_url =
            reqwest::Url::parse(url_str).map_err(|e| SdkError::Config(e.to_string()))?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Ok(Self { base_url })
    }
}

// ── Request types ──────────────────────────────────────────────────────────

/// Parameters for `create_set`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSetParams {
    /// Unique key identifying the set.
    pub key: String,
    /// String value stored under the key.
    pub value: String,
}

/// Parameters for `get_sets`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetSetsParams {
    /// Maximum number of entries returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor returned by a previous page; pass to fetch the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameters for `bulk_sets`. Either or both fields may be supplied.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSetsParams {
    /// Key/value pairs to add.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_sets: Option<HashMap<String, String>>,
    /// Keys to delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_sets: Option<Vec<String>>,
}

/// Parameters for `create_list`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateListParams {
    /// Unique key identifying the list.
    pub key: String,
    /// Initial items inserted into the list.
    pub items: Vec<String>,
}

/// Parameters for `get_lists`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetListsParams {
    /// Maximum number of list keys returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor returned by a previous page; pass to fetch the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameters for `get_list`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetListParams {
    /// Maximum number of items returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Cursor returned by a previous page; pass to fetch the next page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Parameters for `update_list`. Either or both fields may be supplied.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateListParams {
    /// Items to add to the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_items: Option<Vec<String>>,
    /// Items to remove from the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_items: Option<Vec<String>>,
}

/// Parameters for `add_list_item`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct AddListItemParams {
    /// Item to append to the list.
    pub item: String,
}

// ── Response types ─────────────────────────────────────────────────────────

/// A single key/value entry returned by `get_sets`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvSetEntry {
    /// Key identifying the set.
    pub key: String,
    /// Stored string value.
    pub value: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl KvSetEntry {
    #[new]
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }
}

/// Response from `get_sets`.
// GET /sets → {"data": [{key, value}], "cursor": ""}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSetsResponse {
    /// Key/value entries on the current page.
    #[serde(default, deserialize_with = "null_as_default")]
    pub data: Vec<KvSetEntry>,
    /// Cursor for the next page; empty string when there are no more pages.
    pub cursor: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetSetsResponse {
    #[new]
    pub fn new(data: Vec<KvSetEntry>, cursor: String) -> Self {
        Self { data, cursor }
    }
}

/// Response from `get_set`.
// GET /sets/{key} → {"data": {"key": "...", "value": "..."}}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSetResponse {
    /// Stored string value.
    pub value: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetSetResponse {
    #[new]
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

/// Inner data for `get_lists` responses.
// Inner data for GET /lists → data.keys
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GetListsData {
    /// List keys on the current page.
    pub keys: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetListsData {
    #[new]
    pub fn new(keys: Vec<String>) -> Self {
        Self { keys }
    }
}

/// Response from `get_lists`.
// GET /lists → {"data": {"keys": [...]}, "cursor": ""}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListsResponse {
    /// List keys on the current page.
    #[serde(default, deserialize_with = "null_as_default")]
    pub data: GetListsData,
    /// Cursor for the next page; empty string when there are no more pages.
    pub cursor: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetListsResponse {
    #[new]
    pub fn new(data: GetListsData, cursor: String) -> Self {
        Self { data, cursor }
    }
}

/// Inner data for `get_list` responses.
// Inner data for GET /lists/{key} → data.items
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListData {
    /// Items in the list on the current page.
    pub items: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetListData {
    #[new]
    pub fn new(items: Vec<String>) -> Self {
        Self { items }
    }
}

/// Response from `get_list`.
// GET /lists/{key} → {"data": {"items": [...]}, "cursor": ""}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListResponse {
    /// Items for the list on the current page.
    pub data: GetListData,
    /// Cursor for the next page; empty string when there are no more pages.
    pub cursor: String,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetListResponse {
    #[new]
    pub fn new(data: GetListData, cursor: String) -> Self {
        Self { data, cursor }
    }
}

/// Response from `list_contains_item`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContainsItemResponse {
    /// `true` when the item is present in the list.
    pub exists: bool,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl ListContainsItemResponse {
    #[new]
    pub fn new(exists: bool) -> Self {
        Self { exists }
    }
}

// ── API response wrapper ───────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    data: T,
}

// ── Client ─────────────────────────────────────────────────────────────────

/// Client for the Quicknode Key-Value Store. Supports two primitives: *sets*
/// (single string values under a key) and *lists* (ordered collections of
/// strings under a key).
#[derive(Debug, Clone)]
pub struct KvStoreApiClient {
    config: SdkConfig,
}

impl KvStoreApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    // ── Sets ────────────────────────────────────────────────────────────────

    /// Creates a new set, storing a single string value under the given key.
    pub async fn create_set(&self, params: &CreateSetParams) -> Result<(), SdkError> {
        let url = self.config.kvstore().base_url.join("sets")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Returns a paginated page of key/value entries from the store. Use the
    /// response `cursor` to fetch subsequent pages.
    pub async fn get_sets(&self, params: &GetSetsParams) -> Result<GetSetsResponse, SdkError> {
        let mut url = self.config.kvstore().base_url.join("sets")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = params.limit {
                pairs.append_pair("limit", &v.to_string());
            }
            if let Some(v) = &params.cursor {
                pairs.append_pair("cursor", v);
            }
        }
        let resp = self
            .config
            .http_client()
            .get(url)
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

    /// Returns the string value stored for a single set by key.
    pub async fn get_set(&self, key: &str) -> Result<GetSetResponse, SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("sets/{key}"))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;
        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        let wrapper: ApiResponse<GetSetResponse> =
            serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
        Ok(wrapper.data)
    }

    /// Adds and removes multiple sets in a single request. Either `add_sets`,
    /// `delete_sets`, or both may be supplied.
    pub async fn bulk_sets(&self, params: &BulkSetsParams) -> Result<(), SdkError> {
        let url = self.config.kvstore().base_url.join("sets/bulk")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a single set by key.
    pub async fn delete_set(&self, key: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("sets/{key}"))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    // ── Lists ───────────────────────────────────────────────────────────────

    /// Creates a new list under the given key, seeded with the provided items.
    pub async fn create_list(&self, params: &CreateListParams) -> Result<(), SdkError> {
        let url = self.config.kvstore().base_url.join("lists")?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Returns a paginated page of list keys from the store. Use the response
    /// `cursor` to fetch subsequent pages.
    pub async fn get_lists(&self, params: &GetListsParams) -> Result<GetListsResponse, SdkError> {
        let mut url = self.config.kvstore().base_url.join("lists")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = params.limit {
                pairs.append_pair("limit", &v.to_string());
            }
            if let Some(v) = &params.cursor {
                pairs.append_pair("cursor", v);
            }
        }
        let resp = self
            .config
            .http_client()
            .get(url)
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

    /// Returns a paginated page of items from the list identified by `key`.
    /// Use the response `cursor` to fetch subsequent pages.
    pub async fn get_list(
        &self,
        key: &str,
        params: &GetListParams,
    ) -> Result<GetListResponse, SdkError> {
        let mut url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("lists/{key}"))?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = params.limit {
                pairs.append_pair("limit", &v.to_string());
            }
            if let Some(v) = &params.cursor {
                pairs.append_pair("cursor", v);
            }
        }
        let resp = self
            .config
            .http_client()
            .get(url)
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

    /// Updates an existing list by adding and/or removing items in a single
    /// operation. Either `add_items`, `remove_items`, or both may be supplied.
    pub async fn update_list(&self, key: &str, params: &UpdateListParams) -> Result<(), SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("lists/{key}"))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Appends a single item to the list identified by `key`.
    pub async fn add_list_item(
        &self,
        key: &str,
        params: &AddListItemParams,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("lists/{key}/items"))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Checks whether the specified list contains the given item.
    pub async fn list_contains_item(
        &self,
        key: &str,
        item: &str,
    ) -> Result<ListContainsItemResponse, SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("lists/{key}/contains/{item}"))?;
        let resp = self
            .config
            .http_client()
            .get(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        let body = resp.text().await.map_err(SdkError::Http)?;
        if !status.is_success() {
            return Err(SdkError::Api { status, body });
        }
        let wrapper: ApiResponse<ListContainsItemResponse> =
            serde_json::from_str(&body).map_err(|source| SdkError::Decode { source, body })?;
        Ok(wrapper.data)
    }

    /// Removes a specific item from the list identified by `key`.
    pub async fn delete_list_item(&self, key: &str, item: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("lists/{key}/items/{item}"))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }

    /// Removes a list and all of its items by key.
    pub async fn delete_list(&self, key: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .kvstore()
            .base_url
            .join(&format!("lists/{key}"))?;
        let resp = self
            .config
            .http_client()
            .delete(url)
            .send()
            .await
            .map_err(SdkError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.map_err(SdkError::Http)?;
            return Err(SdkError::Api { status, body });
        }
        Ok(())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{KvStoreConfig, QuicknodeSdk, SdkFullConfig};
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuicknodeSdk {
        QuicknodeSdk::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: None,
            admin: None,
            streams: None,
            webhooks: None,
            kvstore: Some(KvStoreConfig {
                base_url: Some(base_url),
            }),
        })
        .unwrap()
    }

    // ── Sets ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_set_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .create_set(&CreateSetParams {
                key: "k".to_string(),
                value: "v".to_string(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_set_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .create_set(&CreateSetParams {
                key: "k".to_string(),
                value: "v".to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn create_set_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .create_set(&CreateSetParams {
                key: "k".to_string(),
                value: "v".to_string(),
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_sets_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": [{"key": "k1", "value": "v1"}, {"key": "k2", "value": "v2"}], "cursor": ""})))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .kvstore
            .get_sets(&GetSetsParams::default())
            .await
            .unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].key, "k1");
    }

    #[tokio::test]
    async fn get_sets_null_data_empty_store() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"code": 200, "msg": "", "data": null, "cursor": ""}),
            ))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .kvstore
            .get_sets(&GetSetsParams::default())
            .await
            .unwrap();
        assert!(resp.data.is_empty());
        assert_eq!(resp.cursor, "");
    }

    #[tokio::test]
    async fn get_sets_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .get_sets(&GetSetsParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_sets_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .get_sets(&GetSetsParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_set_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets/my-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"value": "my-value"}})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.kvstore.get_set("my-key").await.unwrap();
        assert_eq!(resp.value, "my-value");
    }

    #[tokio::test]
    async fn get_set_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets/my-key"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.kvstore.get_set("my-key").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_set_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/sets/my-key"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.kvstore.get_set("my-key").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn bulk_sets_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets/bulk"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let mut add = HashMap::new();
        add.insert("k1".to_string(), "v1".to_string());
        sdk.kvstore
            .bulk_sets(&BulkSetsParams {
                add_sets: Some(add),
                delete_sets: None,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn bulk_sets_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets/bulk"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .bulk_sets(&BulkSetsParams {
                add_sets: None,
                delete_sets: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn bulk_sets_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets/bulk"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .bulk_sets(&BulkSetsParams {
                add_sets: None,
                delete_sets: None,
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    // Wire-inspection regressions for DX-5341 / DX-5342: confirm that addSets
    // and deleteSets reach the wire under the names the API expects. If these
    // pass, the silent no-op is server-side.
    #[tokio::test]
    async fn bulk_sets_wire_body_add_sets() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets/bulk"))
            .and(body_json(serde_json::json!({
                "add_sets": {"k1": "v1"}
            })))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let mut add = HashMap::new();
        add.insert("k1".to_string(), "v1".to_string());
        sdk.kvstore
            .bulk_sets(&BulkSetsParams {
                add_sets: Some(add),
                delete_sets: None,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn bulk_sets_wire_body_delete_sets() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sets/bulk"))
            .and(body_json(serde_json::json!({
                "delete_sets": ["k1", "k2"]
            })))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .bulk_sets(&BulkSetsParams {
                add_sets: None,
                delete_sets: Some(vec!["k1".to_string(), "k2".to_string()]),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_set_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/sets/my-key"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore.delete_set("my-key").await.unwrap();
    }

    #[tokio::test]
    async fn delete_set_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/sets/my-key"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.kvstore.delete_set("my-key").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_set_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/sets/my-key"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.kvstore.delete_set("my-key").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    // ── Lists ─────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_list_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/lists"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .create_list(&CreateListParams {
                key: "my-list".to_string(),
                items: vec!["item1".to_string()],
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn create_list_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/lists"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .create_list(&CreateListParams {
                key: "my-list".to_string(),
                items: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn create_list_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/lists"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .create_list(&CreateListParams {
                key: "my-list".to_string(),
                items: vec![],
            })
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_lists_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"keys": ["list1", "list2"]}, "cursor": ""}),
            ))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .kvstore
            .get_lists(&GetListsParams::default())
            .await
            .unwrap();
        assert_eq!(resp.data.keys, vec!["list1", "list2"]);
    }

    #[tokio::test]
    async fn get_lists_null_data_empty_store() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"code": 200, "msg": "", "data": null, "cursor": ""}),
            ))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .kvstore
            .get_lists(&GetListsParams::default())
            .await
            .unwrap();
        assert!(resp.data.keys.is_empty());
        assert_eq!(resp.cursor, "");
    }

    #[tokio::test]
    async fn get_lists_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .get_lists(&GetListsParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_lists_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .get_lists(&GetListsParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_list_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"items": ["item1", "item2"]}, "cursor": ""}),
            ))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .kvstore
            .get_list("my-list", &GetListParams::default())
            .await
            .unwrap();
        assert_eq!(resp.data.items, vec!["item1", "item2"]);
    }

    #[tokio::test]
    async fn get_list_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .get_list("my-list", &GetListParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_list_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .get_list("my-list", &GetListParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn update_list_success() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/lists/my-list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .update_list(
                "my-list",
                &UpdateListParams {
                    add_items: Some(vec!["item3".to_string()]),
                    remove_items: None,
                },
            )
            .await
            .unwrap();
    }

    // Wire-inspection regression for DX-5343: confirm addItems/removeItems
    // reach the wire under the names the API expects. If this passes, the
    // silent no-op is server-side.
    #[tokio::test]
    async fn update_list_wire_body() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/lists/my-list"))
            .and(body_json(serde_json::json!({
                "add_items": ["c"],
                "remove_items": ["a"]
            })))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .update_list(
                "my-list",
                &UpdateListParams {
                    add_items: Some(vec!["c".to_string()]),
                    remove_items: Some(vec!["a".to_string()]),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn update_list_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .update_list("my-list", &UpdateListParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn update_list_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .update_list("my-list", &UpdateListParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn add_list_item_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/lists/my-list/items"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .add_list_item(
                "my-list",
                &AddListItemParams {
                    item: "item1".to_string(),
                },
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn add_list_item_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/lists/my-list/items"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .add_list_item(
                "my-list",
                &AddListItemParams {
                    item: "item1".to_string(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn add_list_item_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/lists/my-list/items"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .add_list_item(
                "my-list",
                &AddListItemParams {
                    item: "item1".to_string(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn list_contains_item_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists/my-list/contains/item1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": {"exists": true}})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .kvstore
            .list_contains_item("my-list", "item1")
            .await
            .unwrap();
        assert!(resp.exists);
    }

    #[tokio::test]
    async fn list_contains_item_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists/my-list/contains/item1"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .list_contains_item("my-list", "item1")
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn list_contains_item_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/lists/my-list/contains/item1"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .list_contains_item("my-list", "item1")
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_list_item_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/lists/my-list/items/item1"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore
            .delete_list_item("my-list", "item1")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn delete_list_item_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/lists/my-list/items/item1"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .delete_list_item("my-list", "item1")
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_list_item_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/lists/my-list/items/item1"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .kvstore
            .delete_list_item("my-list", "item1")
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_list_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/lists/my-list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"code": 0, "msg": "ok", "data": null})),
            )
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.kvstore.delete_list("my-list").await.unwrap();
    }

    #[tokio::test]
    async fn delete_list_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.kvstore.delete_list("my-list").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_list_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/lists/my-list"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.kvstore.delete_list("my-list").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }
}
