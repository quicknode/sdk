use std::{thread, time::Duration};

use quicknode_sdk::{
    webhooks::{
        ActivateWebhookParams, CreateWebhookFromTemplateParams, EvmWalletFilterTemplate,
        GetWebhooksParams, TemplateArgs, UpdateWebhookParams, WebhookDestinationAttributes,
        WebhookStartFrom,
    },
    QuickNodeSdk, SdkFullConfig,
};

#[tokio::main]
#[allow(clippy::unwrap_used, clippy::expect_used)]
async fn main() {
    let config = SdkFullConfig::from_env().expect("Config from env failed");
    let qn = QuickNodeSdk::new(&config).expect("sdk failed to initialize");

    let before = qn
        .webhooks
        .list_webhooks(&GetWebhooksParams::default())
        .await
        .expect("list_webhooks failed");
    println!("webhooks before: {}", before.data.len());

    let count = qn
        .webhooks
        .get_enabled_count()
        .await
        .expect("get_enabled_count failed");
    println!("enabled count: {}", count.total);

    let template_args = TemplateArgs::evm_wallet_filter(&EvmWalletFilterTemplate {
        wallets: vec!["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
    })
    .expect("template args are valid");

    let create_params = CreateWebhookFromTemplateParams {
        name: "E2E Test Webhook".to_string(),
        network: "ethereum-mainnet".to_string(),
        notification_email: None,
        destination_attributes: WebhookDestinationAttributes {
            url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef".to_string(),
            security_token: None,
            compression: None,
        },
        template_args,
    };
    let webhook = qn
        .webhooks
        .create_webhook_from_template(&create_params)
        .await
        .expect("create_webhook_from_template failed");
    let id = webhook.id.clone();
    println!("created: {} | {}", id, webhook.status);
    thread::sleep(Duration::from_secs(1));

    let fetched = qn
        .webhooks
        .get_webhook(&id)
        .await
        .expect("get_webhook failed");
    println!("fetched: {} | {}", fetched.id, fetched.name);

    let update_params = UpdateWebhookParams {
        name: Some("E2E Test Webhook Updated".to_string()),
        ..Default::default()
    };
    let updated = qn
        .webhooks
        .update_webhook(&id, &update_params)
        .await
        .expect("update_webhook failed");
    println!("updated name: {}", updated.name);
    thread::sleep(Duration::from_secs(1));

    qn.webhooks
        .pause_webhook(&id)
        .await
        .expect("pause_webhook failed");
    println!("paused");

    let activate_params = ActivateWebhookParams {
        start_from: WebhookStartFrom::Latest,
    };
    qn.webhooks
        .activate_webhook(&id, &activate_params)
        .await
        .expect("activate_webhook failed");
    println!("activated");

    qn.webhooks
        .delete_webhook(&id)
        .await
        .expect("delete_webhook failed");
    println!("deleted: {id}");
    thread::sleep(Duration::from_secs(1));

    let after = qn
        .webhooks
        .list_webhooks(&GetWebhooksParams::default())
        .await
        .expect("list_webhooks failed");
    println!("webhooks after: {}", after.data.len());
}
