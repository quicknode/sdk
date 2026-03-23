#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::errors::SdkError;

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct HttpConfig {
    pub timeout_secs: Option<i64>,
    pub pool_max_idle_per_host: Option<i32>,
}

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
#[derive(Debug, Clone, Default, serde::Deserialize)]
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
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct StreamsConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl StreamsConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        StreamsConfig { base_url }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct WebhooksConfig {
    pub base_url: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl WebhooksConfig {
    #[new]
    #[pyo3(signature = (base_url=None))]
    pub fn new(base_url: Option<String>) -> Self {
        WebhooksConfig { base_url }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SdkFullConfig {
    pub api_key: String,
    pub http: Option<HttpConfig>,
    pub admin: Option<AdminConfig>,
    pub streams: Option<StreamsConfig>,
    pub webhooks: Option<WebhooksConfig>,
}

impl SdkFullConfig {
    pub fn from_api_key(api_key: String) -> Self {
        SdkFullConfig { api_key, http: None, admin: None, streams: None, webhooks: None }
    }

    pub fn from_env() -> Result<Self, SdkError> {
        config::Config::builder()
            .add_source(
                config::Environment::with_prefix("QN_SDK")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| SdkError::Config(e.to_string()))
            .and_then(Self::from_config)
    }

    fn from_config(cfg: config::Config) -> Result<Self, SdkError> {
        cfg.try_deserialize::<SdkFullConfig>()
            .map_err(|e| SdkError::Config(e.to_string()))
    }
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl SdkFullConfig {
    #[new]
    #[pyo3(signature = (api_key, http=None, admin=None, streams=None, webhooks=None))]
    pub fn new(api_key: String, http: Option<HttpConfig>, admin: Option<AdminConfig>, streams: Option<StreamsConfig>, webhooks: Option<WebhooksConfig>) -> Self {
        SdkFullConfig {
            api_key,
            http,
            admin,
            streams,
            webhooks,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn build_config(pairs: &[(&str, &str)]) -> config::Config {
        let mut builder = config::Config::builder();
        for (k, v) in pairs {
            builder = builder.set_override(*k, *v).unwrap();
        }
        builder.build().unwrap()
    }

    #[test]
    fn from_env_missing_api_key_returns_error() {
        let cfg = config::Config::builder().build().unwrap();
        assert!(matches!(
            SdkFullConfig::from_config(cfg),
            Err(SdkError::Config(_))
        ));
    }

    #[test]
    fn from_env_only_api_key() {
        let cfg = build_config(&[("api_key", "test-key")]);
        let config = SdkFullConfig::from_config(cfg).unwrap();
        assert_eq!(config.api_key, "test-key");
        assert!(config.http.is_none());
        assert!(config.admin.is_none());
    }

    #[test]
    fn from_env_all_fields() {
        let cfg = build_config(&[
            ("api_key", "my-api-key"),
            ("http.timeout_secs", "30"),
            ("http.pool_max_idle_per_host", "5"),
            ("admin.base_url", "https://example.com/"),
        ]);
        let config = SdkFullConfig::from_config(cfg).unwrap();
        assert_eq!(config.api_key, "my-api-key");
        let http = config.http.unwrap();
        assert_eq!(http.timeout_secs, Some(30));
        assert_eq!(http.pool_max_idle_per_host, Some(5));
        let admin = config.admin.unwrap();
        assert_eq!(admin.base_url, Some("https://example.com/".to_string()));
    }

    #[test]
    fn from_env_invalid_timeout_secs() {
        let cfg = build_config(&[("api_key", "test-key"), ("http.timeout_secs", "abc")]);
        assert!(matches!(
            SdkFullConfig::from_config(cfg),
            Err(SdkError::Config(_))
        ));
    }
}
