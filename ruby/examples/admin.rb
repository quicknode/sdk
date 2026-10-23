require_relative "../lib/quicknode_sdk"

def show_observability(qn)
  summary = qn.admin.get_usage_rpc_summary(last: "1h", group_by: ["method"], limit: 20)
  puts "RPC credits in window: #{summary[:data][:totals][:credits]}"
  if summary[:data][:next_cursor]
    next_page = qn.admin.get_usage_rpc_summary(cursor: summary[:data][:next_cursor])
    puts "next summary page: #{next_page[:data][:breakdown]&.length || 0} groups"
  end

  filters = qn.admin.list_activity_log_filters
  puts "activity filter actors: #{filters[:data][:actors].length}"
  activity = qn.admin.list_activity_logs(limit: 20)
  puts "activity entries: #{activity[:data].length}"
  if (entry = activity[:data].first)
    detail = qn.admin.get_activity_log(id: entry[:id])
    puts "activity detail present: #{!detail[:data].nil?}"
  end

  transactions = qn.admin.list_transactions(last: "1h", limit: 20)
  transactions[:data][:transactions].each do |transaction|
    puts "transaction state=#{transaction[:state]}, height=#{transaction[:height]}"
  end
  if (transaction = transactions[:data][:transactions].first)
    detail = qn.admin.get_transaction(
      network: transaction[:network], hash: transaction[:transaction_hash], include_payloads: false
    )
    puts "retained transaction exists: #{detail[:data][:exists]}"
  end
end

qn = QuicknodeSdk::SDK.from_env
begin
  show_observability(qn)
rescue QuicknodeSdk::ApiError => error
  raise unless error.status == 403
  puts "Observability access is unavailable for this account."
end

account = qn.admin.account_info
if account[:data]
  a = account[:data]
  plan = a.dig(:subscription, :plan_name) || "<none>"
  puts "account #{a[:id]} | #{a[:name]} | billing=#{a[:billing_version]} | plan=#{plan}"
end

credits = qn.admin.get_api_credits(chain: "ethereum")
if credits[:data]
  puts "ethereum api credits: #{credits[:data].length} methods"
  credits[:data].first(3).each do |c|
    puts "  #{c[:method]} = #{c[:credits]}"
  end
end

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

  filters = qn.admin.get_request_filters(id: first[:id])
  filters[:data].each do |filter|
    puts "filter #{filter[:id]}: whitelist=#{filter[:method_whitelist]}, pattern keys=#{filter[:pattern].length}"
  end
  certificates = qn.admin.get_client_certs(id: first[:id])
  puts "client certificates: #{certificates[:data][:certs].length}, enabled=#{certificates[:data][:enabled]}"
  blacklist = qn.admin.get_method_blacklist(id: first[:id])
  puts "blocked methods: #{blacklist[:data][:methods].length}"
  ranges = qn.admin.get_range_limits(id: first[:id])
  puts "range limits: #{ranges[:data][:limits].length}, supported methods=#{ranges[:data][:supported_methods].length}"
  blocked = qn.admin.get_blocked_ips(id: first[:id])
  puts "blocked IP records: #{blocked[:data].length}"
  latency = qn.admin.get_endpoint_metrics(
    id: first[:id], period: "day", metric: "response_time_over_time", percentile: 95
  )
  puts "p95 latency series: #{latency[:data].length}"

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
