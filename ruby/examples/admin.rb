require "json"
require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

response = JSON.parse(qn.admin.get_endpoints(
  limit: 20,
  sort_by: "created_at",
  sort_direction: "desc"
))

if response["pagination"]
  p = response["pagination"]
  puts "#{response["data"].length} of #{p["total"]} (offset #{p["offset"]}, limit #{p["limit"]})"
end

response["data"].each do |ep|
  puts "#{ep["id"]} | #{ep["name"]} | #{ep["status"]} | #{ep["network"]} | " \
       "dedicated=#{ep["is_dedicated"]} flat=#{ep["is_flat_rate"]}"
end

tags = JSON.parse(qn.admin.list_tags)
puts "account tags: #{tags.dig("data", "tags")&.length || 0}"

first = response["data"].first
if first
  sec = JSON.parse(qn.admin.get_endpoint_security(id: first["id"]))
  puts "get_endpoint_security: has_data=#{!sec["data"].nil?}"
end
