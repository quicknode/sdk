require "json"
require_relative "../lib/quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env

before = JSON.parse(qn.streams.list_streams(nil, nil, nil, nil, nil))
puts "streams before: #{before["pageInfo"]["total"]}"

count = JSON.parse(qn.streams.get_enabled_count(nil))
puts "enabled count: #{count["total"]}"

filter_result = JSON.parse(qn.streams.test_filter(
  "ethereum-mainnet",
  "block",
  "17811625",
  "ZnVuY3Rpb24gbWFpbihkYXRhKSB7IHJldHVybiBkYXRhOyB9",
  nil
))
puts "filter logs: #{filter_result["logs"]}"
sleep 1

dest = QuickNodeSdk::DestinationAttributes.webhook(
  url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
  max_retry: 3,
  retry_interval_sec: 1,
  post_timeout_sec: 10,
  compression: "none"
)

stream = JSON.parse(qn.streams.create_stream({
  name: "E2E Test Stream",
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
}))
stream_id = stream["id"]
puts "created: #{stream_id} | #{stream["status"]}"

fetched = JSON.parse(qn.streams.get_stream(stream_id))
puts "fetched: #{fetched["id"]} | #{fetched["name"]}"

updated = JSON.parse(qn.streams.update_stream(stream_id, { name: "E2E Test Stream Updated" }))
puts "updated name: #{updated["name"]}"
sleep 1

qn.streams.pause_stream(stream_id)
puts "paused"

qn.streams.activate_stream(stream_id)
puts "activated"

qn.streams.delete_stream(stream_id)
puts "deleted: #{stream_id}"
sleep 1

after = JSON.parse(qn.streams.list_streams(nil, nil, nil, nil, nil))
puts "streams after: #{after["pageInfo"]["total"]}"
