require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

response = qn.admin.get_endpoints(
  limit: 20,
  sort_by: "created_at",
  sort_direction: "desc"
)

if response[:pagination]
  p = response[:pagination]
  puts "#{response[:data].length} of #{p[:total]} (offset #{p[:offset]}, limit #{p[:limit]})"
end

response[:data].each do |ep|
  puts "#{ep[:id]} | #{ep[:name]} | #{ep[:status]} | #{ep[:network]} | " \
       "dedicated=#{ep[:is_dedicated]} flat=#{ep[:is_flat_rate]} multichain=#{ep[:is_multichain]}"
end

tags = qn.admin.list_tags
puts "account tags: #{tags.dig(:data, :tags)&.length || 0}"

first = response[:data].first
if first
  sec = qn.admin.get_endpoint_security(id: first[:id])
  puts "get_endpoint_security: has_data=#{!sec[:data].nil?}"

  urls = qn.admin.get_endpoint_urls(id: first[:id])
  if urls[:data]
    mc = urls.dig(:data, :multichain_urls)
    puts "get_endpoint_urls: http=#{urls.dig(:data, :http_url)} multichain_networks=#{mc&.keys}"
  end

  rl = qn.admin.get_rate_limits(id: first[:id])
  rl.dig(:data, :rate_limits)&.each do |row|
    puts "get_rate_limits: bucket=#{row[:bucket]} rate_limit=#{row[:rate_limit]} source=#{row[:source]} id=#{row[:id]}"
  end
end
