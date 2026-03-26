require "json"
require_relative "../lib/quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env

before = JSON.parse(qn.webhooks.list_webhooks(nil, nil))
puts "webhooks before: #{before["data"].length}"

count = JSON.parse(qn.webhooks.get_enabled_count)
puts "enabled count: #{count["total"]}"

destination_attributes = JSON.generate({
  url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef"
})

template_args = JSON.generate({
  template_id: "evmWalletFilter",
  wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"]
})

webhook = JSON.parse(qn.webhooks.create_webhook_from_template(
  "E2E Test Webhook",
  "ethereum-mainnet",
  destination_attributes,
  template_args,
  nil
))
id = webhook["id"]
puts "created: #{id} | #{webhook["status"]}"

fetched = JSON.parse(qn.webhooks.get_webhook(id))
puts "fetched: #{fetched["id"]} | #{fetched["name"]}"

updated = JSON.parse(qn.webhooks.update_webhook(id, "E2E Test Webhook Updated", nil))
puts "updated name: #{updated["name"]}"

qn.webhooks.pause_webhook(id)
puts "paused"

qn.webhooks.activate_webhook(id, "latest")
puts "activated"

qn.webhooks.delete_webhook(id)
puts "deleted: #{id}"
sleep 1

after = JSON.parse(qn.webhooks.list_webhooks(nil, nil))
puts "webhooks after: #{after["data"].length}"
