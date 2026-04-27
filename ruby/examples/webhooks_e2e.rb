require "json"
require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

# list_webhooks is arity-1 native: exercise bare, kwargs, and positional hash.
before = qn.webhooks.list_webhooks({})
puts "webhooks before: #{before[:data].length} (total=#{before.dig(:pageInfo, :total)})"
raise "list_webhooks bare broke" unless qn.webhooks.list_webhooks.keys.sort == before.keys.sort
raise "list_webhooks kwargs splat broke" unless qn.webhooks.list_webhooks(**{}).keys.sort == before.keys.sort

# get_enabled_count is arity-0 native: exercise bare, positional empty hash,
# and kwargs splat — all three must reach the no-args branch of the
# dispatcher cleanly.
count = qn.webhooks.get_enabled_count
puts "enabled count: #{count[:total]}"
raise "get_enabled_count positional hash broke" unless qn.webhooks.get_enabled_count({}).keys.sort == count.keys.sort
raise "get_enabled_count kwargs splat broke" unless qn.webhooks.get_enabled_count(**{}).keys.sort == count.keys.sort

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
id = webhook[:id]
puts "created: #{id} | #{webhook[:status]}"

fetched = qn.webhooks.get_webhook(id: id)
puts "fetched: #{fetched[:id]} | #{fetched[:name]}"

updated = qn.webhooks.update_webhook(id: id, name: "E2E Test Webhook Updated")
puts "updated name: #{updated[:name]}"

qn.webhooks.pause_webhook(id: id)
puts "paused"

qn.webhooks.activate_webhook(id: id, start_from: "latest")
puts "activated"

qn.webhooks.delete_webhook(id: id)
puts "deleted: #{id}"
sleep 1

after = qn.webhooks.list_webhooks({})
puts "webhooks after: #{after[:data].length} (total=#{after.dig(:pageInfo, :total)})"
