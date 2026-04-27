require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

before = qn.streams.list_streams({})
puts "streams before: #{before.dig(:pageInfo, :total)}"

count = qn.streams.get_enabled_count({})
puts "enabled count: #{count[:total]}"

filter_result = qn.streams.test_filter(
  network: "ethereum-mainnet",
  dataset: "block",
  block: "17811625",
  filter_function: "ZnVuY3Rpb24gbWFpbihkYXRhKSB7IHJldHVybiBkYXRhOyB9"
)
puts "filter logs: #{filter_result[:logs]}"
sleep 1

dest = QuicknodeSdk::DestinationAttributes.webhook(
  url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
  max_retry: 3,
  retry_interval_sec: 1,
  post_timeout_sec: 10,
  compression: "none"
)

extra_dest = QuicknodeSdk::DestinationAttributes.webhook(
  url: "https://webhook.site/ae19071a-2dcc-4035-9cdf-406dcb4719ef",
  max_retry: 3,
  retry_interval_sec: 1,
  post_timeout_sec: 10,
  compression: "none"
)

stream = qn.streams.create_stream(
  name: "E2E Test Stream",
  network: "ethereum-mainnet",
  dataset: "block",
  region: "usa_east",
  start_range: 24691804,
  end_range: 24691904,
  destination_attributes: dest,
  extra_destinations: [extra_dest],
  plan: "growth_plan",
  threshold_fetch_buffer: 1000,
  dataset_batch_size: 1,
  include_stream_metadata: "body",
  fix_block_reorgs: 0,
  keep_distance_from_tip: 0,
  elastic_batch_enabled: true,
  status: "active"
)
stream_id = stream[:id]
puts "created: #{stream_id} | #{stream[:status]}"

fetched = qn.streams.get_stream(id: stream_id)
puts "fetched: #{fetched[:id]} | #{fetched[:name]}"

updated = qn.streams.update_stream(id: stream_id, name: "E2E Test Stream Updated")
puts "updated name: #{updated[:name]}"
sleep 1

qn.streams.pause_stream(id: stream_id)
puts "paused"

qn.streams.activate_stream(id: stream_id)
puts "activated"

qn.streams.delete_stream(id: stream_id)
puts "deleted: #{stream_id}"
sleep 1

after = qn.streams.list_streams({})
puts "streams after: #{after.dig(:pageInfo, :total)}"
