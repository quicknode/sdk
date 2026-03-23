#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{exceptions::PyValueError, pyclass, pymethods, PyResult};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Deserializer, Serialize};

use crate::errors::SdkError;

fn deserialize_as_optional_json_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None => Ok(None),
        Some(v) => serde_json::to_string(&v)
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

// ── Enums ──────────────────────────────────────────────────────────────────

#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebhookTemplateId {
    EvmWalletFilter,
    EvmContractEvents,
    EvmAbiFilter,
    SolanaWalletFilter,
    BitcoinWalletFilter,
    XrplWalletFilter,
    HyperliquidWalletEventsFilter,
    StellarWalletTransactionsSourceAccountFilter,
}

impl WebhookTemplateId {
    pub fn as_str(&self) -> &'static str {
        match self {
            WebhookTemplateId::EvmWalletFilter => "evmWalletFilter",
            WebhookTemplateId::EvmContractEvents => "evmContractEvents",
            WebhookTemplateId::EvmAbiFilter => "evmAbiFilter",
            WebhookTemplateId::SolanaWalletFilter => "solanaWalletFilter",
            WebhookTemplateId::BitcoinWalletFilter => "bitcoinWalletFilter",
            WebhookTemplateId::XrplWalletFilter => "xrplWalletFilter",
            WebhookTemplateId::HyperliquidWalletEventsFilter => "hyperliquidWalletEventsFilter",
            WebhookTemplateId::StellarWalletTransactionsSourceAccountFilter => {
                "stellarWalletTransactionsSourceAccountFilter"
            }
        }
    }
}


#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookStartFrom {
    Last,
    Latest,
}

// ── Template Arg Structs ───────────────────────────────────────────────────

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmWalletFilterTemplate {
    pub wallets: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl EvmWalletFilterTemplate {
    #[new]
    pub fn new(wallets: Vec<String>) -> Self {
        Self { wallets }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmContractEventsTemplate {
    pub contracts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_hashes: Option<Vec<String>>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl EvmContractEventsTemplate {
    #[new]
    #[pyo3(signature = (contracts, event_hashes=None))]
    pub fn new(contracts: Vec<String>, event_hashes: Option<Vec<String>>) -> Self {
        Self {
            contracts,
            event_hashes,
        }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmAbiFilterTemplate {
    pub abi: String,
    pub contracts: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl EvmAbiFilterTemplate {
    #[new]
    pub fn new(abi: String, contracts: Vec<String>) -> Self {
        Self { abi, contracts }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaWalletFilterTemplate {
    pub accounts: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl SolanaWalletFilterTemplate {
    #[new]
    pub fn new(accounts: Vec<String>) -> Self {
        Self { accounts }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinWalletFilterTemplate {
    pub wallets: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl BitcoinWalletFilterTemplate {
    #[new]
    pub fn new(wallets: Vec<String>) -> Self {
        Self { wallets }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrplWalletFilterTemplate {
    pub wallets: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl XrplWalletFilterTemplate {
    #[new]
    pub fn new(wallets: Vec<String>) -> Self {
        Self { wallets }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidWalletEventsFilterTemplate {
    pub wallets: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl HyperliquidWalletEventsFilterTemplate {
    #[new]
    pub fn new(wallets: Vec<String>) -> Self {
        Self { wallets }
    }
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarWalletTransactionsFilterTemplate {
    pub wallets: Vec<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl StellarWalletTransactionsFilterTemplate {
    #[new]
    pub fn new(wallets: Vec<String>) -> Self {
        Self { wallets }
    }
}

// ── Template Args ──────────────────────────────────────────────────────────

// The API expects a `template_id` string and a `template_args` object whose
// shape depends on which template is selected. The natural Rust model would be
// an enum with per-variant data, but napi-rs and PyO3 cannot represent Rust
// discriminated unions at the FFI boundary — they require flat structs.
// Instead, `TemplateArgs` is a flat wrapper struct that bundles the template
// variant with its pre-serialized JSON value. Callers construct it via typed
// static factory methods (one per template), so they never interact with raw
// JSON.

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass)]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateArgs {
    // pub fields required for napi(object) to expose them in TypeScript.
    // Callers should use the typed factory methods rather than setting fields
    // directly — the value field is a pre-serialized JSON string.
    pub template_id: WebhookTemplateId,
    // Stored as a JSON string so napi(object) can represent it (serde_json::Value
    // is not supported by napi-rs). Parsed back to Value in the client.
    pub value: String,
}

// napi(object) on params structs requires all fields to implement Default so
// napi can handle cases where the field is absent in JS. In practice,
// template_args is always required — the default is never used.
impl Default for TemplateArgs {
    fn default() -> Self {
        Self {
            template_id: WebhookTemplateId::EvmWalletFilter,
            value: "null".to_string(),
        }
    }
}

impl TemplateArgs {
    pub fn evm_wallet_filter(attrs: &EvmWalletFilterTemplate) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::EvmWalletFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn evm_contract_events(attrs: &EvmContractEventsTemplate) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::EvmContractEvents,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn evm_abi_filter(attrs: &EvmAbiFilterTemplate) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::EvmAbiFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn solana_wallet_filter(attrs: &SolanaWalletFilterTemplate) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::SolanaWalletFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn bitcoin_wallet_filter(attrs: &BitcoinWalletFilterTemplate) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::BitcoinWalletFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn xrpl_wallet_filter(attrs: &XrplWalletFilterTemplate) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::XrplWalletFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn hyperliquid_wallet_events_filter(
        attrs: &HyperliquidWalletEventsFilterTemplate,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::HyperliquidWalletEventsFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }

    pub fn stellar_wallet_transactions_filter(
        attrs: &StellarWalletTransactionsFilterTemplate,
    ) -> Result<Self, SdkError> {
        Ok(Self {
            template_id: WebhookTemplateId::StellarWalletTransactionsSourceAccountFilter,
            value: serde_json::to_string(attrs).map_err(|e| SdkError::Config(e.to_string()))?,
        })
    }
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl TemplateArgs {
    #[staticmethod]
    #[pyo3(name = "evm_wallet_filter")]
    fn py_evm_wallet_filter(attrs: &EvmWalletFilterTemplate) -> PyResult<Self> {
        Self::evm_wallet_filter(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "evm_contract_events")]
    fn py_evm_contract_events(attrs: &EvmContractEventsTemplate) -> PyResult<Self> {
        Self::evm_contract_events(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "evm_abi_filter")]
    fn py_evm_abi_filter(attrs: &EvmAbiFilterTemplate) -> PyResult<Self> {
        Self::evm_abi_filter(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "solana_wallet_filter")]
    fn py_solana_wallet_filter(attrs: &SolanaWalletFilterTemplate) -> PyResult<Self> {
        Self::solana_wallet_filter(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "bitcoin_wallet_filter")]
    fn py_bitcoin_wallet_filter(attrs: &BitcoinWalletFilterTemplate) -> PyResult<Self> {
        Self::bitcoin_wallet_filter(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "xrpl_wallet_filter")]
    fn py_xrpl_wallet_filter(attrs: &XrplWalletFilterTemplate) -> PyResult<Self> {
        Self::xrpl_wallet_filter(attrs).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "hyperliquid_wallet_events_filter")]
    fn py_hyperliquid_wallet_events_filter(
        attrs: &HyperliquidWalletEventsFilterTemplate,
    ) -> PyResult<Self> {
        Self::hyperliquid_wallet_events_filter(attrs)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[staticmethod]
    #[pyo3(name = "stellar_wallet_transactions_filter")]
    fn py_stellar_wallet_transactions_filter(
        attrs: &StellarWalletTransactionsFilterTemplate,
    ) -> PyResult<Self> {
        Self::stellar_wallet_transactions_filter(attrs)
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

// ── Webhook Destination Attributes ─────────────────────────────────────────

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDestinationAttributes {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl WebhookDestinationAttributes {
    #[new]
    #[pyo3(signature = (url, security_token=None, compression=None))]
    pub fn new(url: String, security_token: Option<String>, compression: Option<String>) -> Self {
        Self {
            url,
            security_token,
            compression,
        }
    }
}

// ── Request Types ──────────────────────────────────────────────────────────

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetWebhooksParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl GetWebhooksParams {
    #[new]
    #[pyo3(signature = (limit=None, offset=None))]
    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        Self { limit, offset }
    }
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWebhookParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<WebhookDestinationAttributes>,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl UpdateWebhookParams {
    #[new]
    #[pyo3(signature = (name=None, notification_email=None, destination_attributes=None))]
    pub fn new(
        name: Option<String>,
        notification_email: Option<String>,
        destination_attributes: Option<WebhookDestinationAttributes>,
    ) -> Self {
        Self {
            name,
            notification_email,
            destination_attributes,
        }
    }
}

#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivateWebhookParams {
    pub start_from: WebhookStartFrom,
}

#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWebhookFromTemplateParams {
    pub name: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    pub destination_attributes: WebhookDestinationAttributes,
    // template_args is skipped here and inserted manually into the request body
    // in the client, so serde doesn't try to serialize it as a field of this struct.
    #[serde(skip)]
    pub template_args: TemplateArgs,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWebhookTemplateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<WebhookDestinationAttributes>,
    // template_id and template_args are skipped here and inserted manually into
    // the request body in the client.
    #[serde(skip)]
    pub template_args: TemplateArgs,
}

#[cfg(feature = "python")]
#[gen_stub_pymethods]
#[pymethods]
impl UpdateWebhookTemplateParams {
    #[new]
    #[pyo3(signature = (template_args, name=None, notification_email=None, destination_attributes=None))]
    pub fn new(
        template_args: TemplateArgs,
        name: Option<String>,
        notification_email: Option<String>,
        destination_attributes: Option<WebhookDestinationAttributes>,
    ) -> Self {
        Self {
            name,
            notification_email,
            destination_attributes,
            template_args,
        }
    }
}

// ── Response Types ─────────────────────────────────────────────────────────

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    pub id: String,
    pub name: String,
    pub status: String,
    pub network: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// Destination-specific configuration as a JSON string.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_as_optional_json_string"
    )]
    pub destination_attributes: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWebhooksResponse {
    pub data: Vec<Webhook>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEnabledCountResponse {
    pub total: i64,
}
