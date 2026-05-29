import asyncio

from quicknode_sdk import (
    EvmWalletFilterArgs,
    EvmWalletFilterTemplate,
    QuicknodeSdk,
    WebhookDestinationAttributes,
)


async def main():
    qn = QuicknodeSdk.from_env()

    before = await qn.webhooks.list_webhooks()
    print(f"webhooks before: {len(before.data)} (total={before.page_info.total})")

    count = await qn.webhooks.get_enabled_count()
    print(f"enabled count: {count.total}")

    webhook = await qn.webhooks.create_webhook_from_template(
        name="E2E Test Webhook",
        network="ethereum-mainnet",
        destination_attributes=WebhookDestinationAttributes(
            url="https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
        ),
        template_args=EvmWalletFilterArgs(
            EvmWalletFilterTemplate(wallets=["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"])
        ),
    )
    webhook_id = webhook.id
    print(f"created: {webhook_id} | {webhook.status}")

    fetched = await qn.webhooks.get_webhook(webhook_id)
    print(f"fetched: {fetched.id} | {fetched.name}")

    updated = await qn.webhooks.update_webhook(webhook_id, name="E2E Test Webhook Updated")
    print(f"updated name: {updated.name}")

    await qn.webhooks.pause_webhook(webhook_id)
    print("paused")

    await qn.webhooks.activate_webhook(webhook_id, start_from="latest")
    print("activated")

    await qn.webhooks.delete_webhook(webhook_id)
    print(f"deleted: {webhook_id}")

    after = await qn.webhooks.list_webhooks()
    print(f"webhooks after: {len(after.data)} (total={after.page_info.total})")


asyncio.run(main())
