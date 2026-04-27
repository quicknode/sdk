#[cfg(feature = "rust")]
use bon::Builder;
#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::{pyclass, pymethods};
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use serde::{Deserialize, Deserializer, Serialize};

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

/// Identifier of a predefined webhook filter template.
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

/// Position a webhook begins (or resumes) delivering from when activated.
#[cfg_attr(feature = "node", napi(string_enum))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookStartFrom {
    /// Resume from the last-delivered block.
    Last,
    /// Start from the newest available block.
    Latest,
}

// ── Template Arg Structs ───────────────────────────────────────────────────

/// Template arguments for an EVM wallet filter: matches activity for a list of
/// wallet addresses.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmWalletFilterTemplate {
    /// Wallet addresses to match against.
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

/// Template arguments for filtering EVM contract events, optionally scoped to
/// a specific set of event topic hashes.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmContractEventsTemplate {
    /// Contract addresses to watch for events.
    pub contracts: Vec<String>,
    /// Optional list of event topic hashes to restrict the filter to specific events.
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

/// Template arguments for an EVM ABI filter: decodes and filters events for a
/// set of contracts using a provided ABI.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmAbiFilterTemplate {
    /// JSON-encoded contract ABI used to decode event data.
    pub abi: String,
    /// Contract addresses to watch for events.
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

/// Template arguments for a Solana wallet filter: matches activity for a list
/// of Solana account addresses.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaWalletFilterTemplate {
    /// Solana account addresses to match against.
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

/// Template arguments for a Bitcoin wallet filter.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinWalletFilterTemplate {
    /// Bitcoin wallet addresses to match against.
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

/// Template arguments for an XRPL wallet filter.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrplWalletFilterTemplate {
    /// XRPL wallet addresses to match against.
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

/// Template arguments for a Hyperliquid wallet-events filter.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperliquidWalletEventsFilterTemplate {
    /// Hyperliquid wallet addresses to match against.
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

/// Template arguments for a Stellar wallet-transactions filter, matching
/// transactions where the given wallets are the source account.
#[cfg_attr(feature = "rust", derive(Builder))]
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarWalletTransactionsFilterTemplate {
    /// Stellar wallet addresses to match against.
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

/// Template identifier paired with its arguments. Exactly one variant selects
/// which filter is applied. Consumed by `create_webhook_from_template` and
/// `update_webhook_template`.
// Pure-Rust discriminated union; no #[pyclass] / #[napi(object)] because PyO3
// and napi-rs cannot represent enum-with-data. Each language binding crate
// wraps this type for its own FFI surface.
// The serde tag/content pair matches the API wire format when flattened into
// a request struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "templateId", content = "templateArgs", rename_all = "camelCase")]
pub enum TemplateArgs {
    /// EVM wallet filter: matches activity for a list of wallet addresses.
    EvmWalletFilter(EvmWalletFilterTemplate),
    /// EVM contract events filter, optionally scoped to specific event topic hashes.
    EvmContractEvents(EvmContractEventsTemplate),
    /// EVM ABI filter: decodes and filters events using a provided ABI.
    EvmAbiFilter(EvmAbiFilterTemplate),
    /// Solana wallet filter.
    SolanaWalletFilter(SolanaWalletFilterTemplate),
    /// Bitcoin wallet filter.
    BitcoinWalletFilter(BitcoinWalletFilterTemplate),
    /// XRPL wallet filter.
    XrplWalletFilter(XrplWalletFilterTemplate),
    /// Hyperliquid wallet-events filter.
    HyperliquidWalletEventsFilter(HyperliquidWalletEventsFilterTemplate),
    /// Stellar wallet-transactions filter (source-account match).
    StellarWalletTransactionsSourceAccountFilter(StellarWalletTransactionsFilterTemplate),
}

impl TemplateArgs {
    pub fn tag(&self) -> WebhookTemplateId {
        match self {
            Self::EvmWalletFilter(_) => WebhookTemplateId::EvmWalletFilter,
            Self::EvmContractEvents(_) => WebhookTemplateId::EvmContractEvents,
            Self::EvmAbiFilter(_) => WebhookTemplateId::EvmAbiFilter,
            Self::SolanaWalletFilter(_) => WebhookTemplateId::SolanaWalletFilter,
            Self::BitcoinWalletFilter(_) => WebhookTemplateId::BitcoinWalletFilter,
            Self::XrplWalletFilter(_) => WebhookTemplateId::XrplWalletFilter,
            Self::HyperliquidWalletEventsFilter(_) => {
                WebhookTemplateId::HyperliquidWalletEventsFilter
            }
            Self::StellarWalletTransactionsSourceAccountFilter(_) => {
                WebhookTemplateId::StellarWalletTransactionsSourceAccountFilter
            }
        }
    }
}

// ── Webhook Destination Attributes ─────────────────────────────────────────

/// Destination configuration for a webhook.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDestinationAttributes {
    /// Target URL that receives webhook payloads.
    pub url: String,
    /// Optional token sent with each payload so the receiver can verify authenticity; generated automatically when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_token: Option<String>,
    /// Optional payload compression (`gzip` or `none`).
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

/// Parameters for `list_webhooks`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GetWebhooksParams {
    /// Maximum number of webhooks returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Starting index into the result set.
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

/// Parameters for `update_webhook`. All fields are optional; only set fields
/// are modified.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateWebhookParams {
    /// New human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New notification email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// New destination configuration.
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

/// Parameters for `activate_webhook`.
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(not(feature = "node"), derive(Clone))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivateWebhookParams {
    /// Position to begin (or resume) delivery from.
    pub start_from: WebhookStartFrom,
}

/// Parameters for `create_webhook_from_template`.
#[cfg_attr(feature = "rust", derive(Builder))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebhookFromTemplateParams {
    /// Human-readable label for the webhook.
    pub name: String,
    /// Blockchain network to watch (e.g. `ethereum-mainnet`).
    pub network: String,
    /// Optional email that receives alerts if the webhook terminates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// Destination configuration for delivered payloads.
    pub destination_attributes: WebhookDestinationAttributes,
    /// Filter template identifier and its arguments.
    // Flattening the enum's tag/content produces { templateId, templateArgs }.
    #[serde(flatten)]
    pub template_args: TemplateArgs,
}

/// Parameters for `update_webhook_template`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWebhookTemplateParams {
    /// New human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New notification email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_email: Option<String>,
    /// New destination configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_attributes: Option<WebhookDestinationAttributes>,
    /// New template identifier and arguments.
    // Flattening the enum's tag/content produces { templateId, templateArgs }.
    #[serde(flatten)]
    pub template_args: TemplateArgs,
}

// ── Response Types ─────────────────────────────────────────────────────────

/// A webhook's full configuration and current state.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Unique webhook identifier.
    pub id: String,
    /// Human-readable webhook name.
    pub name: String,
    /// Current operational state (e.g. `active`, `paused`).
    pub status: String,
    /// Blockchain network the webhook is watching.
    pub network: String,
    /// Timestamp when the webhook was created.
    pub created_at: String,
    /// Timestamp of the most recent modification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Template identifier used to create the webhook, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    /// Email address notified of webhook terminations or failures.
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

/// Pagination metadata returned alongside a paginated webhooks list.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPageInfo {
    /// Page size used for this response.
    pub limit: i64,
    /// Starting index of this page within the full result set.
    pub offset: i64,
    /// Total number of webhooks matching the query across all pages.
    pub total: i64,
}

/// Response from `list_webhooks`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWebhooksResponse {
    /// Webhooks on the current page.
    pub data: Vec<Webhook>,
    /// Pagination metadata for the response.
    #[serde(rename = "pageInfo")]
    pub page_info: WebhookPageInfo,
}

/// Response from `get_enabled_count` for webhooks.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEnabledCountResponse {
    /// Total count of enabled webhooks on the account.
    pub total: i64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod template_args_tests {
    use super::*;

    #[test]
    fn evm_wallet_filter_roundtrip() {
        let args = TemplateArgs::EvmWalletFilter(EvmWalletFilterTemplate {
            wallets: vec!["0xabc".to_string()],
        });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"evmWalletFilter""#));
        assert!(json.contains(r#""wallets":["0xabc"]"#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TemplateArgs::EvmWalletFilter(_)));
        assert!(matches!(parsed.tag(), WebhookTemplateId::EvmWalletFilter));
    }

    #[test]
    fn evm_contract_events_roundtrip() {
        let args = TemplateArgs::EvmContractEvents(EvmContractEventsTemplate {
            contracts: vec!["0xdef".to_string()],
            event_hashes: Some(vec!["0x1234".to_string()]),
        });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"evmContractEvents""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TemplateArgs::EvmContractEvents(_)));
    }

    #[test]
    fn evm_abi_filter_roundtrip() {
        let args = TemplateArgs::EvmAbiFilter(EvmAbiFilterTemplate {
            abi: "[]".to_string(),
            contracts: vec!["0xdef".to_string()],
        });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"evmAbiFilter""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TemplateArgs::EvmAbiFilter(_)));
    }

    #[test]
    fn solana_wallet_filter_roundtrip() {
        let args = TemplateArgs::SolanaWalletFilter(SolanaWalletFilterTemplate {
            accounts: vec!["acc".to_string()],
        });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"solanaWalletFilter""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TemplateArgs::SolanaWalletFilter(_)));
    }

    #[test]
    fn bitcoin_wallet_filter_roundtrip() {
        let args = TemplateArgs::BitcoinWalletFilter(BitcoinWalletFilterTemplate {
            wallets: vec!["bc1".to_string()],
        });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"bitcoinWalletFilter""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TemplateArgs::BitcoinWalletFilter(_)));
    }

    #[test]
    fn xrpl_wallet_filter_roundtrip() {
        let args = TemplateArgs::XrplWalletFilter(XrplWalletFilterTemplate {
            wallets: vec!["r1".to_string()],
        });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"xrplWalletFilter""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, TemplateArgs::XrplWalletFilter(_)));
    }

    #[test]
    fn hyperliquid_wallet_events_filter_roundtrip() {
        let args =
            TemplateArgs::HyperliquidWalletEventsFilter(HyperliquidWalletEventsFilterTemplate {
                wallets: vec!["0xhl".to_string()],
            });
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"hyperliquidWalletEventsFilter""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            parsed,
            TemplateArgs::HyperliquidWalletEventsFilter(_)
        ));
    }

    #[test]
    fn stellar_wallet_transactions_filter_roundtrip() {
        let args = TemplateArgs::StellarWalletTransactionsSourceAccountFilter(
            StellarWalletTransactionsFilterTemplate {
                wallets: vec!["G...".to_string()],
            },
        );
        let json = serde_json::to_string(&args).unwrap();
        assert!(json.contains(r#""templateId":"stellarWalletTransactionsSourceAccountFilter""#));
        let parsed: TemplateArgs = serde_json::from_str(&json).unwrap();
        assert!(matches!(
            parsed,
            TemplateArgs::StellarWalletTransactionsSourceAccountFilter(_)
        ));
    }

    #[test]
    fn create_params_flattens_template_args() {
        let params = CreateWebhookFromTemplateParams {
            name: "n".to_string(),
            network: "ethereum-mainnet".to_string(),
            notification_email: None,
            destination_attributes: WebhookDestinationAttributes {
                url: "https://x".to_string(),
                security_token: None,
                compression: None,
            },
            template_args: TemplateArgs::EvmWalletFilter(EvmWalletFilterTemplate {
                wallets: vec!["0xabc".to_string()],
            }),
        };
        let json = serde_json::to_value(&params).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(
            obj.get("templateId").and_then(|v| v.as_str()),
            Some("evmWalletFilter")
        );
        assert!(obj.get("templateArgs").unwrap().is_object());
        assert_eq!(obj["templateArgs"]["wallets"][0].as_str(), Some("0xabc"));
    }
}
