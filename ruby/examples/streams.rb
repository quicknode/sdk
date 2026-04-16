require "json"
require_relative "../lib/quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env

dest = QuickNodeSdk::DestinationAttributes.webhook(
  url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
  max_retry: 3,
  retry_interval_sec: 1,
  post_timeout_sec: 10,
  compression: "none"
)

stream_json = qn.streams.create_stream({
  name: "My Stream",
  network: "ethereum-mainnet",
  dataset: "block",
  region: "usa_east",
  start_range: 24691804,
  end_range: 24691904,
  destination_attributes: dest,
  plan: "growth_plan",
  threshold_fetch_buffer: 1000,
  dataset_batch_size: 1,
  include_stream_metadata: "body",
  fix_block_reorgs: 0,
  keep_distance_from_tip: 0,
  elastic_batch_enabled: true,
  status: "active"
})

stream = JSON.parse(stream_json)
puts "#{stream["id"]} | #{stream["name"]} | #{stream["status"]}"
