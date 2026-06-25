require_relative "../lib/quicknode_sdk"

CLUSTER_ID = "hyperliquid-core-mainnet"

qn = QuicknodeSdk::SDK.from_env

# Query
resp = qn.sql.query(
  query: "SELECT toDateTime(block_time) AS time, action_type, user " \
         "FROM hyperliquid_system_actions " \
         "ORDER BY block_time DESC LIMIT 3",
  cluster_id: CLUSTER_ID
)
stats = resp[:statistics]
puts "query: #{resp[:rows]} rows (#{resp[:rows_before_limit_at_least]} before limit), " \
     "#{resp[:credits]} credits, #{stats[:elapsed].round(4)}s"
puts "columns: #{resp[:meta].map { |c| c[:name] }.join(', ')}"
first_row = resp[:data].first
puts "first row action_type: #{first_row[:action_type]}" if first_row

# Schema
schema = qn.sql.get_schema(cluster_id: CLUSTER_ID)
puts "schema: #{schema[:chain]} (#{schema[:tables].length} tables)"
table = schema[:tables].first
if table
  puts "first table: #{table[:name]} (#{table[:columns].length} columns, #{table[:total_rows]} rows)"
end

# Error handling: an empty query is rejected with a 403.
begin
  qn.sql.query(query: "", cluster_id: CLUSTER_ID)
rescue QuicknodeSdk::ApiError => e
  puts "api error #{e.status}: #{e.body[0, 120]}"
end
