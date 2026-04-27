require "json"
require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

before = qn.webhooks.list_webhooks({})
puts "webhooks before: #{before.data.length} (total=#{before.pageInfo.total})"

count = qn.webhooks.get_enabled_count
puts "enabled count: #{count.total}"

destination_attributes = JSON.generate({
  url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
  compression: "none"
})

template_args = JSON.generate({
  templateId: "evmWalletFilter",
  templateArgs: { wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"] }
})

webhook = qn.webhooks.create_webhook_from_template(
  name: "E2E Test Webhook",
  network: "ethereum-mainnet",
  destination_attributes_json: destination_attributes,
  template_args_json: template_args
)
id = webhook.id
puts "created: #{id} | #{webhook.status}"

fetched = qn.webhooks.get_webhook(id: id)
puts "fetched: #{fetched.id} | #{fetched.name}"

updated = qn.webhooks.update_webhook(id: id, name: "E2E Test Webhook Updated")
puts "updated name: #{updated.name}"

qn.webhooks.pause_webhook(id: id)
puts "paused"

qn.webhooks.activate_webhook(id: id, start_from: "latest")
puts "activated"

qn.webhooks.delete_webhook(id: id)
puts "deleted: #{id}"
sleep 1

after = qn.webhooks.list_webhooks({})
puts "webhooks after: #{after.data.length} (total=#{after.pageInfo.total})"
