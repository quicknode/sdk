pub mod admin;
pub mod errors;

use reqwest::Client as ReqwestClient;
use std::sync::Arc;

#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

const ADMIN_BASE_URL: &str = "https://api.quicknode.com/v0/";

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default)]
pub struct HttpConfig {
    pub timeout_secs: Option<i64>,
    pub pool_max_idle_per_host: Option<i32>,
}

// Only for python to keep typings in arguments rather than building a class as an argument
#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl HttpConfig {
    #[new]
    #[pyo3(signature = (timeout_secs=None, pool_max_idle_per_host=None))]
    pub fn new(timeout_secs: Option<i64>, pool_max_idle_per_host: Option<i32>) -> Self {
        HttpConfig {
            timeout_secs,
            pool_max_idle_per_host,
        }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default)]
pub struct AdminConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl AdminConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        AdminConfig { base_url }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone)]
pub struct SdkFullConfig {
    pub api_key: String,
    pub http: Option<HttpConfig>,
    pub admin: Option<AdminConfig>,
}

impl SdkFullConfig {
    pub fn from_api_key(api_key: String) -> Self {
        SdkFullConfig { api_key, http: None, admin: None }
    }
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl SdkFullConfig {
    #[new]
    #[pyo3(signature = (api_key, http=None, admin=None))]
    pub fn new(api_key: String, http: Option<HttpConfig>, admin: Option<AdminConfig>) -> Self {
        SdkFullConfig {
            api_key,
            http,
            admin,
        }
    }
}

// Using Arc for the inner config to keep as a cheap clone
#[derive(Debug, Clone)]
pub struct SdkConfig(Arc<SdkConfigInner>);

#[derive(Debug)]
struct SdkConfigInner {
    http_client: ReqwestClient,
    api_key: String,
    admin_base_url: reqwest::Url,
}

impl SdkConfig {
    pub fn new(config: SdkFullConfig) -> Self {
        let mut builder = ReqwestClient::builder();
        if let Some(http) = &config.http {
            if let Some(secs) = http.timeout_secs {
                builder = builder.timeout(std::time::Duration::from_secs(secs as u64));
            }
            if let Some(max_idle) = http.pool_max_idle_per_host {
                builder = builder.pool_max_idle_per_host(max_idle as usize);
            }
        }
        let http_client = builder.build().expect("failed to build HTTP client");

        let admin_base_url_str = config
            .admin
            .as_ref()
            .and_then(|a| a.base_url.as_deref())
            .unwrap_or(ADMIN_BASE_URL);
        let admin_base_url =
            reqwest::Url::parse(admin_base_url_str).expect("invalid admin base URL");

        Self(Arc::new(SdkConfigInner {
            http_client,
            api_key: config.api_key,
            admin_base_url,
        }))
    }

    pub(crate) fn http_client(&self) -> &ReqwestClient {
        &self.0.http_client
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.0.api_key
    }

    pub(crate) fn admin_base_url(&self) -> &reqwest::Url {
        &self.0.admin_base_url
    }
}

pub struct QuickNodeSdk {
    pub admin: admin::AdminApiClient,
}

impl QuickNodeSdk {
    pub fn new(config: SdkFullConfig) -> Self {
        let sdk_config = SdkConfig::new(config);
        Self {
            admin: admin::AdminApiClient::new(sdk_config),
        }
    }
}
