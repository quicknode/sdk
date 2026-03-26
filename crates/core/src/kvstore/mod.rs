use std::collections::HashMap;

#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Serialize};

use crate::{config::KvStoreConfig, errors::SdkError, SdkConfig};

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

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSetParams {
    pub key: String,
    pub value: String,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetSetsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct BulkSetsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_sets: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_sets: Option<Vec<String>>,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateListParams {
    pub key: String,
    pub items: Vec<String>,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetListsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateListParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_items: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_items: Option<Vec<String>>,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct AddListItemParams {
    pub item: String,
}

// ── Response types ─────────────────────────────────────────────────────────

// A single entry returned in the GET /sets listing
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KvSetEntry {
    pub key: String,
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

// GET /sets → {"data": [{key, value}], "cursor": ""}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSetsResponse {
    pub data: Vec<KvSetEntry>,
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

// GET /sets/{key} → {"data": {"key": "...", "value": "..."}}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSetResponse {
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

// Inner data for GET /lists → data.keys
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListsData {
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

// GET /lists → {"data": {"keys": [...]}, "cursor": ""}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListsResponse {
    pub data: GetListsData,
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

// Inner data for GET /lists/{key} → data.items
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListData {
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

// GET /lists/{key} → {"data": {"items": [...]}, "cursor": ""}
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetListResponse {
    pub data: GetListData,
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

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListContainsItemResponse {
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

#[derive(Debug, Clone)]
pub struct KvStoreApiClient {
    config: SdkConfig,
}

impl KvStoreApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    // ── Sets ────────────────────────────────────────────────────────────────

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
    use crate::{KvStoreConfig, QuickNodeSdk, SdkFullConfig};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuickNodeSdk {
        QuickNodeSdk::new(&SdkFullConfig {
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
