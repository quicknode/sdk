pub mod webhook;

pub use webhook::{
    ActivateWebhookParams, BitcoinWalletFilterByListTemplate, BitcoinWalletFilterInput,
    BitcoinWalletFilterTemplate, CreateWebhookFromTemplateParams, EvmAbiFilterByListTemplate,
    EvmAbiFilterInput, EvmAbiFilterTemplate, EvmContractEventsByListTemplate,
    EvmContractEventsInput, EvmContractEventsTemplate, EvmWalletFilterByListTemplate,
    EvmWalletFilterInput, EvmWalletFilterTemplate, GetWebhooksParams,
    HyperliquidWalletEventsFilterByListTemplate, HyperliquidWalletEventsFilterInput,
    HyperliquidWalletEventsFilterTemplate, ListWebhooksResponse, SolanaWalletFilterByListTemplate,
    SolanaWalletFilterInput, SolanaWalletFilterTemplate,
    StellarWalletTransactionsFilterByListTemplate, StellarWalletTransactionsFilterInput,
    StellarWalletTransactionsFilterTemplate, TemplateArgs, UpdateWebhookParams,
    UpdateWebhookTemplateParams, Webhook, WebhookDestinationAttributes,
    WebhookEnabledCountResponse, WebhookPageInfo, WebhookStartFrom, WebhookTemplateId,
    XrplWalletFilterByListTemplate, XrplWalletFilterInput, XrplWalletFilterTemplate,
};

use crate::{config::WebhooksConfig, errors::SdkError, SdkConfig};

const WEBHOOKS_BASE_URL: &str = "https://api.quicknode.com/webhooks/rest/v1/";

pub(crate) struct ResolvedWebhooksConfig {
    pub(crate) base_url: reqwest::Url,
}

impl ResolvedWebhooksConfig {
    pub(crate) fn from_config(config: Option<&WebhooksConfig>) -> Result<Self, SdkError> {
        let url_str = config
            .and_then(|s| s.base_url.as_deref())
            .unwrap_or(WEBHOOKS_BASE_URL);
        let mut base_url = reqwest::Url::parse(url_str)?;
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Ok(Self { base_url })
    }
}

// ── Client ─────────────────────────────────────────────────────────────────

/// Client for the Quicknode Webhooks REST API. Create webhooks from filter
/// templates, manage their lifecycle, and update their destinations.
#[derive(Debug, Clone)]
pub struct WebhooksApiClient {
    config: SdkConfig,
}

impl WebhooksApiClient {
    pub fn new(config: SdkConfig) -> Self {
        Self { config }
    }

    /// Returns a paginated list of webhooks on the account. Each entry includes
    /// the webhook's identifier, creation timestamp, name, network, notification
    /// email, destination configuration (URL, security token, compression),
    /// current status, and any associated template. The response also includes
    /// a `pageInfo` object with the applied limit, offset, and total count.
    pub async fn list_webhooks(
        &self,
        params: &GetWebhooksParams,
    ) -> Result<ListWebhooksResponse, SdkError> {
        let mut url = self.config.webhooks().base_url.join("webhooks")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(v) = params.limit {
                pairs.append_pair("limit", &v.to_string());
            }
            if let Some(v) = params.offset {
                pairs.append_pair("offset", &v.to_string());
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

    /// Removes every webhook on the account. Destructive and takes no
    /// parameters.
    pub async fn delete_all_webhooks(&self) -> Result<(), SdkError> {
        let url = self.config.webhooks().base_url.join("webhooks")?;
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

    /// Fetches a single webhook's full configuration and status by ID. Returns
    /// creation timestamp, name, network, notification email, destination
    /// configuration (URL, security token, compression), the sequence number
    /// of the last successfully delivered block, the current status, and the
    /// associated template with its arguments.
    pub async fn get_webhook(&self, id: &str) -> Result<Webhook, SdkError> {
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/{id}"))?;
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

    /// Modifies an existing webhook's configuration. Supports updating the
    /// webhook's name, notification email, and destination attributes (URL,
    /// security token, and compression — `none` or `gzip`). All fields are
    /// optional, so partial updates are supported; if the security token is
    /// omitted on update, one is generated automatically. Returns the
    /// webhook's full updated configuration.
    pub async fn update_webhook(
        &self,
        id: &str,
        params: &UpdateWebhookParams,
    ) -> Result<Webhook, SdkError> {
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/{id}"))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
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

    /// Permanently removes a single webhook by ID.
    pub async fn delete_webhook(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/{id}"))?;
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

    /// Pauses a webhook by ID so it stops delivering events until reactivated.
    pub async fn pause_webhook(&self, id: &str) -> Result<(), SdkError> {
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/{id}/pause"))?;
        let resp = self
            .config
            .http_client()
            .post(url)
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

    /// Activates a previously created or paused webhook so it begins (or
    /// resumes) delivering events. `start_from` determines where processing
    /// resumes: `Latest` begins from the newest available block; other values
    /// replay from an earlier point.
    pub async fn activate_webhook(
        &self,
        id: &str,
        params: &ActivateWebhookParams,
    ) -> Result<(), SdkError> {
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/{id}/activate"))?;
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

    /// Returns the total number of enabled webhooks currently configured on
    /// the account.
    pub async fn get_enabled_count(&self) -> Result<WebhookEnabledCountResponse, SdkError> {
        let url = self
            .config
            .webhooks()
            .base_url
            .join("webhooks/enabled_count")?;
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

    /// Creates a new webhook from a predefined filter template. Requires a
    /// descriptive name, a target blockchain network, and destination
    /// attributes (URL, compression — `gzip` or `none`, and an optional
    /// security token — auto-generated when omitted). `template_args` carries
    /// template-specific configuration such as wallet addresses or contract
    /// filters. An optional `notification_email` receives alerts if the
    /// webhook terminates.
    pub async fn create_webhook_from_template(
        &self,
        params: &CreateWebhookFromTemplateParams,
    ) -> Result<Webhook, SdkError> {
        let template_id = params.template_args.tag().as_str();
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/template/{template_id}"))?;
        let resp = self
            .config
            .http_client()
            .post(url)
            .json(params)
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

    /// Updates an existing template-backed webhook, modifying its template
    /// arguments and optionally its name, notification email, and destination
    /// attributes (URL, security token, compression — `none` or `gzip`).
    /// All optional fields support partial updates; a security token is
    /// generated automatically if not provided. Templates cover EVM chains,
    /// Solana, Bitcoin, XRPL, Hyperliquid, and Stellar.
    pub async fn update_webhook_template(
        &self,
        webhook_id: &str,
        params: &UpdateWebhookTemplateParams,
    ) -> Result<Webhook, SdkError> {
        let template_id = params.template_args.tag().as_str();
        let url = self
            .config
            .webhooks()
            .base_url
            .join(&format!("webhooks/{webhook_id}/template/{template_id}"))?;
        let resp = self
            .config
            .http_client()
            .patch(url)
            .json(params)
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
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::{QuicknodeSdk, SdkFullConfig, WebhooksConfig};
    use wiremock::matchers::{method, path, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_sdk(base_url: String) -> QuicknodeSdk {
        QuicknodeSdk::new(&SdkFullConfig {
            api_key: "test-key".to_string(),
            http: None,
            admin: None,
            streams: None,
            webhooks: Some(WebhooksConfig {
                base_url: Some(base_url),
            }),
            kvstore: None,
            sql: None,
        })
        .unwrap()
    }

    fn webhook_response_json() -> serde_json::Value {
        serde_json::json!({
            "id": "wh-1234-5678",
            "name": "test-webhook",
            "status": "active",
            "network": "ethereum-mainnet",
            "created_at": "2026-03-19T12:00:00Z",
            "updated_at": "2026-03-19T12:00:00Z"
        })
    }

    #[tokio::test]
    async fn list_webhooks_success() {
        let server = MockServer::start().await;
        let response = serde_json::json!({
            "data": [webhook_response_json()],
            "pageInfo": {
                "limit": 20,
                "offset": 0,
                "total": 1,
            }
        });
        Mock::given(method("GET"))
            .and(path("/webhooks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk
            .webhooks
            .list_webhooks(&GetWebhooksParams::default())
            .await
            .unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].id, "wh-1234-5678");
        assert_eq!(resp.page_info.limit, 20);
        assert_eq!(resp.page_info.offset, 0);
        assert_eq!(resp.page_info.total, 1);
    }

    #[tokio::test]
    async fn list_webhooks_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/webhooks"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .webhooks
            .list_webhooks(&GetWebhooksParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn list_webhooks_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/webhooks"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk
            .webhooks
            .list_webhooks(&GetWebhooksParams::default())
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_webhook_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/webhooks/test-id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(webhook_response_json()))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.webhooks.get_webhook("test-id").await.unwrap();
        assert_eq!(resp.id, "wh-1234-5678");
    }

    #[tokio::test]
    async fn get_webhook_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/webhooks/test-id"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.webhooks.get_webhook("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn update_webhook_success() {
        let server = MockServer::start().await;
        let mut updated = webhook_response_json();
        updated["name"] = serde_json::json!("updated-name");
        Mock::given(method("PATCH"))
            .and(path("/webhooks/test-id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(updated))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateWebhookParams {
            name: Some("updated-name".to_string()),
            ..Default::default()
        };
        let resp = sdk
            .webhooks
            .update_webhook("test-id", &params)
            .await
            .unwrap();
        assert_eq!(resp.name, "updated-name");
    }

    #[tokio::test]
    async fn update_webhook_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/webhooks/test-id"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = UpdateWebhookParams::default();
        let err = sdk
            .webhooks
            .update_webhook("test-id", &params)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_webhook_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/webhooks/test-id"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.webhooks.delete_webhook("test-id").await.unwrap();
    }

    #[tokio::test]
    async fn delete_webhook_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/webhooks/test-id"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.webhooks.delete_webhook("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn delete_all_webhooks_success() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/webhooks"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.webhooks.delete_all_webhooks().await.unwrap();
    }

    #[tokio::test]
    async fn delete_all_webhooks_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/webhooks"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.webhooks.delete_all_webhooks().await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn pause_webhook_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/webhooks/test-id/pause"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        sdk.webhooks.pause_webhook("test-id").await.unwrap();
    }

    #[tokio::test]
    async fn pause_webhook_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/webhooks/test-id/pause"))
            .respond_with(ResponseTemplate::new(404).set_body_string("Not Found"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.webhooks.pause_webhook("test-id").await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn activate_webhook_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/webhooks/test-id/activate"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = ActivateWebhookParams {
            start_from: WebhookStartFrom::Latest,
        };
        sdk.webhooks
            .activate_webhook("test-id", &params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn activate_webhook_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/webhooks/test-id/activate"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let params = ActivateWebhookParams {
            start_from: WebhookStartFrom::Latest,
        };
        let err = sdk
            .webhooks
            .activate_webhook("test-id", &params)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn get_enabled_count_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/webhooks/enabled_count"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"total": 5})))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let resp = sdk.webhooks.get_enabled_count().await.unwrap();
        assert_eq!(resp.total, 5);
    }

    #[tokio::test]
    async fn get_enabled_count_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/webhooks/enabled_count"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let err = sdk.webhooks.get_enabled_count().await.unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn create_webhook_from_template_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex("/webhooks/template/evmWalletFilter"))
            .respond_with(ResponseTemplate::new(201).set_body_json(webhook_response_json()))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let template_args =
            TemplateArgs::EvmWalletFilter(EvmWalletFilterInput::Inline(EvmWalletFilterTemplate {
                wallets: vec!["0xabc".to_string()],
            }));
        let params = CreateWebhookFromTemplateParams {
            name: "test-webhook".to_string(),
            network: "ethereum-mainnet".to_string(),
            notification_email: None,
            destination_attributes: WebhookDestinationAttributes {
                url: "https://example.com/hook".to_string(),
                security_token: None,
                compression: "none".to_string(),
            },
            template_args,
        };
        let resp = sdk
            .webhooks
            .create_webhook_from_template(&params)
            .await
            .unwrap();
        assert_eq!(resp.id, "wh-1234-5678");
    }

    #[tokio::test]
    async fn create_webhook_from_template_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex("/webhooks/template/evmWalletFilter"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let template_args =
            TemplateArgs::EvmWalletFilter(EvmWalletFilterInput::Inline(EvmWalletFilterTemplate {
                wallets: vec!["0xabc".to_string()],
            }));
        let params = CreateWebhookFromTemplateParams {
            name: "test-webhook".to_string(),
            network: "ethereum-mainnet".to_string(),
            notification_email: None,
            destination_attributes: WebhookDestinationAttributes {
                url: "https://example.com/hook".to_string(),
                security_token: None,
                compression: "none".to_string(),
            },
            template_args,
        };
        let err = sdk
            .webhooks
            .create_webhook_from_template(&params)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[tokio::test]
    async fn update_webhook_template_success() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path_regex("/webhooks/test-id/template/evmWalletFilter"))
            .respond_with(ResponseTemplate::new(200).set_body_json(webhook_response_json()))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let template_args =
            TemplateArgs::EvmWalletFilter(EvmWalletFilterInput::Inline(EvmWalletFilterTemplate {
                wallets: vec!["0xabc".to_string()],
            }));
        let params = UpdateWebhookTemplateParams {
            name: None,
            notification_email: None,
            destination_attributes: None,
            template_args,
        };
        let resp = sdk
            .webhooks
            .update_webhook_template("test-id", &params)
            .await
            .unwrap();
        assert_eq!(resp.id, "wh-1234-5678");
    }

    // Wire-inspection regression: confirm that `name` reaches the wire when
    // supplied so any future serde rename/drop of the field fails loudly.
    #[tokio::test]
    async fn update_webhook_template_wire_body_includes_name() {
        use wiremock::matchers::body_partial_json;
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path_regex("/webhooks/test-id/template/evmWalletFilter"))
            .and(body_partial_json(serde_json::json!({"name": "new-name"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(webhook_response_json()))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let template_args =
            TemplateArgs::EvmWalletFilter(EvmWalletFilterInput::Inline(EvmWalletFilterTemplate {
                wallets: vec!["0xabc".to_string()],
            }));
        let params = UpdateWebhookTemplateParams {
            name: Some("new-name".to_string()),
            notification_email: None,
            destination_attributes: None,
            template_args,
        };
        sdk.webhooks
            .update_webhook_template("test-id", &params)
            .await
            .unwrap();
    }

    // Wire-inspection regression: the API expects `eventHashes` (camelCase)
    // and returns 500 if it sees `event_hashes`. Confirm the field reaches the
    // wire under the camelCase key.
    #[tokio::test]
    async fn create_webhook_from_template_wire_body_uses_camelcase_event_hashes() {
        use wiremock::matchers::body_partial_json;
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path_regex("/webhooks/template/evmContractEvents"))
            .and(body_partial_json(serde_json::json!({
                "templateArgs": {
                    "eventHashes": ["0xabcd"],
                }
            })))
            .respond_with(ResponseTemplate::new(201).set_body_json(webhook_response_json()))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let template_args = TemplateArgs::EvmContractEvents(EvmContractEventsInput::Inline(
            EvmContractEventsTemplate {
                contracts: vec!["0xa0b8".to_string()],
                event_hashes: vec!["0xabcd".to_string()],
            },
        ));
        let params = CreateWebhookFromTemplateParams {
            name: "test-webhook".to_string(),
            network: "ethereum-mainnet".to_string(),
            notification_email: None,
            destination_attributes: WebhookDestinationAttributes {
                url: "https://example.com/hook".to_string(),
                security_token: None,
                compression: "none".to_string(),
            },
            template_args,
        };
        sdk.webhooks
            .create_webhook_from_template(&params)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn update_webhook_template_api_error() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path_regex("/webhooks/test-id/template/evmWalletFilter"))
            .respond_with(ResponseTemplate::new(400).set_body_string("Bad Request"))
            .mount(&server)
            .await;
        let sdk = make_sdk(format!("{}/", server.uri()));
        let template_args =
            TemplateArgs::EvmWalletFilter(EvmWalletFilterInput::Inline(EvmWalletFilterTemplate {
                wallets: vec!["0xabc".to_string()],
            }));
        let params = UpdateWebhookTemplateParams {
            name: None,
            notification_email: None,
            destination_attributes: None,
            template_args,
        };
        let err = sdk
            .webhooks
            .update_webhook_template("test-id", &params)
            .await
            .unwrap_err();
        assert!(matches!(err, SdkError::Api { .. }));
    }
}
