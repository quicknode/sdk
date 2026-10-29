require "securerandom"
require "openssl"
require "time"
require_relative "../lib/quicknode_sdk"

def check_usage_fields(rows, fields)
  rows.each do |row|
    fields.each do |field|
      raise "missing usage field #{field}" unless row.key?(field)
      raise "usage #{field} must be an integer or nil" unless row[field].nil? || row[field].is_a?(Integer)
    end
  end
end

def check_bulk_results(response, endpoint_id)
  results = response.fetch(:data).fetch(:results)
  success = results.find { |row| row[:id] == endpoint_id }
  failure = results.find { |row| row[:id] == "0" }
  raise "expected successful endpoint result" unless success && success[:success]
  raise "successful row unexpectedly has an error" unless success[:error].nil?
  raise "expected failed endpoint result" unless failure && failure[:success] == false
  raise "missing bulk item error" unless failure[:error].is_a?(String) && !failure[:error].empty?
end

def with_feature_access(operation)
  yield
rescue QuicknodeSdk::ApiError => error
  raise unless error.status == 403
  raise "expected typed SDK error" unless error.is_a?(QuicknodeSdk::Error)
  raise "missing API error body" unless error.body.is_a?(String) && !error.body.empty?
  puts "#{operation}: unavailable for this account (HTTP 403)"
end

def check_summary(qn, operation, **params)
  with_feature_access(operation) do
    response = qn.admin.public_send(operation, **params)
    data = response.fetch(:data)
    raise "missing summary window" unless data[:from].is_a?(String) && data[:to].is_a?(String)
    raise "expected numeric summary totals" unless data.fetch(:totals).values.all? { |value| value.is_a?(Numeric) }
    (data[:breakdown] || []).each do |row|
      params.fetch(:group_by, []).each do |dimension|
        raise "missing selected dimension #{dimension}" unless row.key?(dimension)
        raise "unexpected dimension value" unless row[dimension].nil? || row[dimension].is_a?(String)
      end
    end
    if data[:next_cursor]
      page = qn.admin.public_send(operation, **params.slice(:id), cursor: data[:next_cursor]).fetch(:data)
      raise "cursor changed summary window" unless page[:from] == data[:from] && page[:to] == data[:to]
      raise "cursor omitted totals" unless page.fetch(:totals).keys.sort == data[:totals].keys.sort
    end
    puts "#{operation}: numeric totals and #{(data[:breakdown] || []).length} groups"
  end
end

qn = QuicknodeSdk::SDK.from_env

# ── Read-only globals ─────────────────────────────────────────────────────────

# Exercise all three call styles the dispatcher supports against an arity-0
# native method (list_chains): bare, positional empty hash, and kwargs splat.
resp = qn.admin.list_chains
puts "list_chains: #{resp[:data].length} chains"
raise "list_chains positional hash broke" unless qn.admin.list_chains({})[:data].length == resp[:data].length
raise "list_chains kwargs splat broke" unless qn.admin.list_chains(**{})[:data].length == resp[:data].length

# Same coverage for an arity-1 native method (get_endpoints) — this is the
# call shape the README advertises bare, so all three must work.
bare = qn.admin.get_endpoints
kwargs = qn.admin.get_endpoints(limit: 5)
positional = qn.admin.get_endpoints({})
puts "get_endpoints: #{kwargs[:data].length} endpoints (bare=#{bare[:data].length}, positional=#{positional[:data].length})"
raise "get_endpoints bare vs positional differ" unless bare.keys.sort == positional.keys.sort

# Verify symbol- and string-key access work interchangeably.
sample = qn.admin.get_endpoints(limit: 1)
raise "expected QuicknodeSdk::IndifferentHash" unless sample.is_a?(QuicknodeSdk::IndifferentHash)
raise "indifferent access broken" unless sample[:data] == sample["data"]
puts "indifferent access: ok"

# Same three-call-style check for one of the arity-1 read-only usage methods.
# The remaining get_usage_by_* methods share the dispatcher path, so one
# representative is enough to lock in coverage.
usage_bare = qn.admin.get_usage
usage_positional = qn.admin.get_usage({})
usage_kwargs = qn.admin.get_usage(**{})
raise "get_usage call styles diverge" unless usage_bare.keys.sort == usage_positional.keys.sort && usage_bare.keys.sort == usage_kwargs.keys.sort
puts "get_usage: #{usage_bare[:data].inspect}"
resp = usage_bare

sleep 0.5

resp = qn.admin.get_usage_by_endpoint({})
endpoint_usage = resp.dig(:data, :endpoints) || []
check_usage_fields(endpoint_usage, [:rpc_credits, :data_credits, :bytes])
endpoint_usage.each { |row| check_usage_fields(row[:methods_breakdown], [:bytes]) }
puts "get_usage_by_endpoint: #{endpoint_usage.length} rows"

resp = qn.admin.get_usage_by_method({})
method_usage = resp.dig(:data, :methods) || []
check_usage_fields(method_usage, [:bytes])
puts "get_usage_by_method: #{method_usage.length} rows"

sleep 0.5

resp = qn.admin.get_usage_by_chain({})
chain_usage = resp.dig(:data, :chains) || []
check_usage_fields(chain_usage, [:rpc_credits, :data_credits, :bytes])
puts "get_usage_by_chain: #{chain_usage.length} rows"

resp = qn.admin.get_usage_by_tag({})
tag_usage = resp.dig(:data, :tags) || []
check_usage_fields(tag_usage, [:rpc_credits, :data_credits, :bytes])
puts "get_usage_by_tag: #{tag_usage.length} rows"

resp = qn.admin.list_tags
puts "list_tags: #{resp.dig(:data, :tags)&.length || 0} tags"

resp = qn.admin.get_account_metrics(period: "day", metric: "requests")
first_tag = resp[:data].first&.dig(:tag)&.join(":") || "<none>"
puts "get_account_metrics: #{resp[:data].length} series, first tag: #{first_tag}"

resp = qn.admin.list_invoices
puts "list_invoices: #{resp[:data].inspect}"

resp = qn.admin.list_payments
puts "list_payments: #{resp[:data].inspect}"

resp = qn.admin.list_teams
puts "list_teams: #{resp[:data].length} teams"

sleep 0.5

# Summary windows and continuation cursors use the same request on every page.
check_summary(qn, :get_usage_credits_summary, last: "1h")
check_summary(qn, :get_usage_rpc_summary, last: "1h", group_by: ["method"], limit: 2)
check_summary(qn, :get_usage_streams_summary, last: "1h", group_by: ["stream"], limit: 2)
check_summary(qn, :get_usage_webhooks_summary, last: "1h", group_by: ["webhook"], limit: 2)
check_summary(qn, :get_traffic_summary, last: "1h", group_by: ["method"], limit: 2)
check_summary(qn, :get_errors_summary, last: "1h", kind: "rpc", group_by: ["code"], limit: 2)
check_summary(qn, :get_errors_summary, last: "1h", kind: "http", group_by: ["code"], limit: 2)

with_feature_access("activity logs") do
  filters = qn.admin.list_activity_log_filters.fetch(:data)
  raise "missing activity filter metadata" unless filters[:actors].is_a?(Array) && filters[:types].is_a?(Array)
  activity = qn.admin.list_activity_logs(limit: 2, offset: 0)
  raise "incorrect activity offset" unless activity.dig(:pagination, :offset) == 0
  activity.fetch(:data).each do |entry|
    actor = entry.fetch(:actor)
    raise "missing actor type" unless actor[:type].is_a?(String)
    [:id, :name, :email].each do |field|
      raise "invalid nullable actor #{field}" unless actor[field].nil? || actor[field].is_a?(String)
    end
  end
  if (entry = activity[:data].first)
    detail = qn.admin.get_activity_log(id: entry[:id])
    raise "activity detail changed ID" unless detail.dig(:data, :id) == entry[:id]
  end
  empty = qn.admin.list_activity_logs(limit: 0, offset: 0)
  raise "zero-limit activity query returned rows" unless empty[:data].empty?
  puts "activity logs: #{activity[:data].length} entries"
end

with_feature_access("transactions") do
  response = qn.admin.list_transactions(last: "1h", limit: 2)
  data = response.fetch(:data)
  data.fetch(:transactions).each do |transaction|
    height = transaction[:height]
    raise "transaction height lost decimal-string representation" unless height.nil? || (height.is_a?(String) && height.match?(/\A-?\d+\z/))
  end
  if (transaction = data[:transactions].first)
    [false, true].each do |include_payloads|
      lookup = qn.admin.get_transaction(
        network: transaction[:network], hash: transaction[:transaction_hash], include_payloads: include_payloads
      ).fetch(:data)
      raise "missing transaction existence flag" unless [true, false].include?(lookup[:exists])
      if lookup[:exists]
        lookup.fetch(:data).fetch(:attempts).each do |attempt|
          payloads = attempt[:payloads]
          raise "unexpected transaction payloads" if !include_payloads && payloads
          next unless payloads
          [:request, :response].each do |field|
            raise "transaction payload was parsed instead of preserved" unless payloads[field].nil? || payloads[field].is_a?(String)
          end
        end
      else
        raise "absent transaction has details" unless lookup[:data].nil?
      end
    end
  end
  if data[:next_cursor]
    page = qn.admin.list_transactions(cursor: data[:next_cursor]).fetch(:data)
    raise "cursor changed transaction window" unless page[:from] == data[:from] && page[:to] == data[:to]
  end
  puts "transactions: #{data[:transactions].length} retained records"
end

# ── Create endpoint ───────────────────────────────────────────────────────────

resp = qn.admin.create_endpoint(chain: "ethereum", network: "mainnet")
endpoint_id = resp.dig(:data, :id)
unless endpoint_id
  puts "create_endpoint failed: #{resp[:error]}"
  exit 1
end
puts "create_endpoint: #{endpoint_id}"

begin
  # ── Endpoint CRUD ─────────────────────────────────────────────────────────────

  resp = qn.admin.show_endpoint(id: endpoint_id)
  puts "show_endpoint: #{resp.dig(:data, :id)}"

  qn.admin.update_endpoint(id: endpoint_id, label: "sdk-example")
  puts "update_endpoint: ok"

  sleep 0.5

  resp = qn.admin.update_endpoint_status(id: endpoint_id, status: "paused")
  puts "update_endpoint_status paused: #{resp[:data]}"

  resp = qn.admin.update_endpoint_status(id: endpoint_id, status: "active")
  puts "update_endpoint_status active: #{resp[:data]}"

  sleep 1

  # ── Tags ──────────────────────────────────────────────────────────────────────

  tag_label = "sdk-tag-#{SecureRandom.hex(4)}"
  resp = qn.admin.create_tag_with_response(id: endpoint_id, label: tag_label)
  association = resp.fetch(:data).fetch(:tags).find { |tag| tag[:label] == tag_label }
  raise "missing tag association" unless association
  begin
    raise "tag association IDs must remain numeric" unless association[:id].is_a?(Integer) && association[:tag_id].is_a?(Integer)
  ensure
    qn.admin.delete_tag(id: endpoint_id, tag_id: association.fetch(:tag_id).to_s)
    qn.admin.delete_account_tag(id: association.fetch(:tag_id))
  end
  puts "create_tag_with_response and cleanup: ok"

  # ── Logs & metrics ────────────────────────────────────────────────────────────

  resp = qn.admin.get_endpoint_logs(
    id: endpoint_id,
    from_time: (Time.now.utc - 3600).iso8601,
    to_time: Time.now.utc.iso8601,
    limit: 20
  )
  puts "get_endpoint_logs: #{resp[:data].length} entries"

  sleep 1

  resp = qn.admin.get_endpoint_metrics(
    id: endpoint_id, period: "day", metric: "response_time_over_time", percentile: 95
  )
  first_tag = resp[:data].first&.dig(:tag)&.join(":") || "<none>"
  puts "get_endpoint_metrics: #{resp[:data].length} series, first tag: #{first_tag}"

  check_summary(qn, :get_endpoint_errors_summary, id: endpoint_id, last: "1h", kind: "rpc", group_by: ["code"], limit: 2)

  # ── Security options ──────────────────────────────────────────────────────────

  resp = qn.admin.get_security_options(id: endpoint_id)
  puts "get_security_options: #{resp[:data].length} options"

  sleep 1

  resp = qn.admin.get_endpoint_security(id: endpoint_id)
  security = resp.fetch(:data)
  raise "missing blocked-IP snapshot field" unless security.key?(:blocked_ips)
  raise "invalid blocked-IP snapshot field" unless security[:blocked_ips].nil? || security[:blocked_ips].is_a?(Array)
  [:hsts, :cors, :responseLogging].each do |option|
    raise "missing security option #{option}" unless security.fetch(:options).key?(option)
  end
  with_feature_access("client certificates") do
    certificates = qn.admin.get_client_certs(id: endpoint_id).fetch(:data)
    raise "invalid certificate state" unless [true, false].include?(certificates[:enabled]) && certificates[:certs].is_a?(Array)
  end
  blacklist = qn.admin.get_method_blacklist(id: endpoint_id).fetch(:data)
  raise "invalid method blacklist" unless blacklist[:methods].is_a?(Array)
  ranges = qn.admin.get_range_limits(id: endpoint_id).fetch(:data)
  raise "invalid range limits" unless ranges[:limits].is_a?(Hash) && ranges[:supported_methods].is_a?(Array)
  blocked_ips = qn.admin.get_blocked_ips(id: endpoint_id).fetch(:data)
  raise "invalid blocked-IP list" unless blocked_ips.is_a?(Array)
  puts "security snapshots and lists: ok"

  resp = qn.admin.update_security_options(id: endpoint_id, tokens: "enabled")
  puts "update_security_options: #{resp[:data].length} options"

  sleep 0.5


  # ── Token ─────────────────────────────────────────────────────────────────────

  resp = qn.admin.create_token_with_response(id: endpoint_id)
  token = resp.fetch(:data)
  raise "missing created token" unless token[:id].is_a?(String) && token[:token].is_a?(String) && !token[:token].empty?
  qn.admin.delete_token(id: endpoint_id, token_id: token[:id])
  puts "create_token_with_response and cleanup: ok"

  # Referrer creation uses url; endpoint snapshots use referrer.
  resp = qn.admin.create_referrer_with_response(id: endpoint_id, referrer: "https://example.com")
  referrer = resp.fetch(:data)
  raise "created referrer URL was lost" unless referrer[:url] == "https://example.com"
  qn.admin.delete_referrer(id: endpoint_id, referrer_id: referrer.fetch(:id))
  puts "create_referrer_with_response and cleanup: ok"

  resp = qn.admin.create_ip_with_response(id: endpoint_id, ip: "192.0.2.1")
  ip = resp.fetch(:data)
  raise "created IP was lost" unless ip[:ip] == "192.0.2.1"
  qn.admin.delete_ip(id: endpoint_id, ip_id: ip.fetch(:id))
  puts "create_ip_with_response and cleanup: ok"

  resp = qn.admin.create_domain_mask_with_response(id: endpoint_id, domain_mask: "example.com")
  mask = resp.fetch(:data)
  raise "created domain mask was lost" unless mask[:domain_mask] == "example.com"
  qn.admin.delete_domain_mask(id: endpoint_id, domain_mask_id: mask.fetch(:id))
  puts "create_domain_mask_with_response and cleanup: ok"

  public_key = OpenSSL::PKey::RSA.generate(2048).public_key.to_pem
  resp = qn.admin.create_jwt_with_response(
    id: endpoint_id, public_key: public_key, kid: "sdk-example-key", name: "sdk-example-jwt"
  )
  jwt = resp.fetch(:data)
  raise "created JWT public key was lost" unless jwt[:public_key] == public_key
  raise "created JWT metadata was lost" unless jwt[:kid] == "sdk-example-key" && jwt[:name] == "sdk-example-jwt"
  qn.admin.delete_jwt(id: endpoint_id, jwt_id: jwt.fetch(:id))
  puts "create_jwt_with_response and cleanup: ok"

  # ── Request filter ────────────────────────────────────────────────────────────

  resp = qn.admin.create_request_filter(id: endpoint_id, methods: ["eth_getBalance"])
  rf_id = resp.fetch(:data).fetch(:id)
  custom_id = nil
  begin
    qn.admin.update_request_filter(id: endpoint_id, request_filter_id: rf_id, methods: ["eth_call"])
    pattern = {"method" => [{"prefix" => "eth_"}], "jsonrpc" => ["2.0"]}
    created = qn.admin.create_request_filter_pattern(id: endpoint_id, pattern: pattern).fetch(:data)
    custom_id = created.fetch(:id)
    raise "created custom pattern was lost" unless created[:pattern] == pattern

    listed = qn.admin.get_request_filters(id: endpoint_id).fetch(:data)
    whitelist = listed.find { |filter| filter[:id] == rf_id }
    custom = listed.find { |filter| filter[:id] == custom_id }
    raise "whitelist kind was lost" unless whitelist && whitelist[:method_whitelist] == true
    raise "custom filter kind or pattern was lost" unless custom && custom[:method_whitelist] == false && custom[:pattern] == pattern

    snapshots = [
      qn.admin.get_endpoint_security(id: endpoint_id).fetch(:data),
      qn.admin.show_endpoint(id: endpoint_id).fetch(:data).fetch(:security)
    ]
    snapshots.each do |snapshot|
      filter = snapshot.fetch(:request_filters).find { |item| item[:id] == custom_id }
      raise "structured matcher was lost" unless filter && filter[:method] == pattern["method"]
      raise "snapshot pattern fields were lost" unless filter[:params] == {"jsonrpc" => ["2.0"]}
    end

    updated_pattern = {"method" => [{"prefix" => "net_"}], "jsonrpc" => ["2.0"]}
    qn.admin.update_request_filter_pattern(id: endpoint_id, request_filter_id: custom_id, pattern: updated_pattern)
    updated = qn.admin.get_request_filters(id: endpoint_id).fetch(:data).find { |filter| filter[:id] == custom_id }
    raise "custom pattern update was lost" unless updated && updated[:pattern] == updated_pattern
    puts "whitelist and structured filter lifecycle: ok"
  ensure
    qn.admin.delete_request_filter(id: endpoint_id, request_filter_id: custom_id) if custom_id
    qn.admin.delete_request_filter(id: endpoint_id, request_filter_id: rf_id)
  end

  # ── IP custom header ──────────────────────────────────────────────────────────

  resp = qn.admin.create_or_update_ip_custom_header(id: endpoint_id, header_name: "X-Custom-Header")
  puts "create_or_update_ip_custom_header: #{resp[:data]}"

  resp = qn.admin.delete_ip_custom_header(id: endpoint_id)
  puts "delete_ip_custom_header: #{resp[:data]}"

  sleep 0.5

  # ── Rate limits ───────────────────────────────────────────────────────────────

  resp = qn.admin.get_rate_limits(id: endpoint_id)
  puts "get_rate_limits before PATCH: #{resp[:data]}"

  effective = qn.admin.update_rate_limits_with_response(id: endpoint_id, rps: 3, limit_by_ip: true).fetch(:data).fetch(:limits)
  raise "effective RPS override was lost" unless effective[:rps] == 3
  [:rps, :rpm, :rpd].each do |bucket|
    raise "missing effective rate-limit bucket" unless effective.key?(bucket)
    raise "invalid nullable rate-limit bucket" unless effective[bucket].nil? || effective[bucket].is_a?(Integer)
  end
  puts "update_rate_limits_with_response: ok"
  qn.admin.update_rate_limits(id: endpoint_id, limit_by_ip: false)

  resp = qn.admin.get_rate_limits(id: endpoint_id)
  puts "get_rate_limits after PATCH: #{resp[:data]}"

  resp = qn.admin.get_endpoint_urls(id: endpoint_id)
  puts "get_endpoint_urls: http=#{resp.dig(:data, :http_url)} multichain_networks=#{resp.dig(:data, :multichain_urls)&.keys}"

  resp = qn.admin.get_method_rate_limits(id: endpoint_id)
  puts "get_method_rate_limits: #{resp[:data]}"

  resp = qn.admin.create_method_rate_limit(id: endpoint_id, interval: "second", methods: ["eth_call"], rate: 5, limit_by_ip: true)
  mrl_id = resp.dig(:data, :id)
  raise "limit_by_ip true was lost" unless resp.dig(:data, :limit_by_ip) == true
  puts "create_method_rate_limit: #{resp[:data]}"

  if mrl_id
    resp = qn.admin.update_method_rate_limit(id: endpoint_id, method_rate_limit_id: mrl_id, rate: 10, limit_by_ip: false)
    raise "limit_by_ip false was dropped" unless resp.dig(:data, :limit_by_ip) == false
    puts "update_method_rate_limit: #{resp[:data]}"

    qn.admin.delete_method_rate_limit(id: endpoint_id, method_rate_limit_id: mrl_id)
    puts "delete_method_rate_limit: ok"
  end

  sleep 0.5

  # ── Multichain ────────────────────────────────────────────────────────────────

  qn.admin.enable_multichain(id: endpoint_id)
  puts "enable_multichain: ok"

  qn.admin.disable_multichain(id: endpoint_id)
  puts "disable_multichain: ok"

  sleep 0.5

  # Bulk operations include one valid endpoint and one missing endpoint.

  resp = qn.admin.bulk_update_endpoint_status(ids: [endpoint_id, "0"], status: "paused")
  check_bulk_results(resp, endpoint_id)
  puts "bulk_update_endpoint_status paused: #{resp[:data]}"

  resp = qn.admin.bulk_update_endpoint_status(ids: [endpoint_id], status: "active")
  puts "bulk_update_endpoint_status active: #{resp[:data]}"

  # ── Account-level tags (bulk_add/remove + rename/delete) ─────────────────────

  tag_suffix = SecureRandom.hex(4)
  resp = qn.admin.bulk_add_tag(ids: [endpoint_id, "0"], label: "sdk-bulk-#{tag_suffix}")
  check_bulk_results(resp, endpoint_id)
  puts "bulk_add_tag: #{resp[:data]}"
  bulk_tag_id = resp.dig(:data, :tag, :tag_id)

  sleep 0.5

  if bulk_tag_id
    resp = qn.admin.rename_tag(id: bulk_tag_id, label: "sdk-renamed-#{tag_suffix}")
    puts "rename_tag: #{resp[:data]}"

    resp = qn.admin.bulk_remove_tag(ids: [endpoint_id, "0"], tag_id: bulk_tag_id)
    check_bulk_results(resp, endpoint_id)
    puts "bulk_remove_tag: #{resp[:data]}"

    resp = qn.admin.delete_account_tag(id: bulk_tag_id)
    puts "delete_account_tag: #{resp[:data]}"
  end

  sleep 0.5

  # ── Teams ─────────────────────────────────────────────────────────────────────

  resp = qn.admin.create_team(name: "sdk-example-team")
  team_id = resp.dig(:data, :id)
  puts "create_team: #{resp[:data]}"

  sleep 0.5
  if team_id
    resp = qn.admin.get_team(id: team_id)
    puts "get_team: #{resp.dig(:data, :name)}"

    resp = qn.admin.list_team_endpoints(id: team_id)
    puts "list_team_endpoints: #{resp[:data].length} endpoints"

    sleep 0.5

    resp = qn.admin.update_team_endpoints(id: team_id, endpoint_ids: [endpoint_id])
    puts "update_team_endpoints: #{resp[:data]}"

    sleep 0.5

    begin
      resp = qn.admin.invite_team_member(id: team_id, email: "placeholder@example.com")
      puts "invite_team_member: #{resp[:data]}"
    rescue => e
      puts "invite_team_member error (expected with placeholder email): #{e}"
    end

    sleep 0.5

    resp = qn.admin.delete_team(id: team_id)
    puts "delete_team: #{resp[:data]}"
  end

  sleep 0.5

ensure
  qn.admin.archive_endpoint(id: endpoint_id)
  puts "archive_endpoint: ok"
end

# ── Error handling ───────────────────────────────────────────────────────────

# 1) API error path — 404 on a bogus endpoint id.
begin
  qn.admin.show_endpoint(id: "does-not-exist")
  raise "expected 404"
rescue QuicknodeSdk::ApiError => e
  raise "expected QuicknodeSdk::Error subclass" unless e.is_a?(QuicknodeSdk::Error)
  raise "expected 404, got #{e.status}" unless e.status == 404
  raise "missing API error body" unless e.body.is_a?(String) && !e.body.empty?
  puts "api error #{e.status}: #{e.body[0, 80]}"
end

# 1b) Rate-limit override delete with a bogus override id — also a 404.
begin
  qn.admin.delete_rate_limit_override(
    id: "does-not-exist",
    override_id: "00000000-0000-0000-0000-000000000000"
  )
  raise "expected 404"
rescue QuicknodeSdk::ApiError => e
  raise "expected 404, got #{e.status}" unless e.status == 404
  raise "missing API error body" unless e.body.is_a?(String) && !e.body.empty?
  puts "delete_rate_limit_override api error #{e.status}: #{e.body[0, 80]}"
end

# Custom headers smoke test — override User-Agent + add a correlation header.
headered = QuicknodeSdk::SDK.from_config(
  api_key: ENV.fetch("QN_SDK__API_KEY"),
  http: {
    headers: {
      "User-Agent" => "qn-e2e-ruby/1.0",
      "X-E2E-Correlation" => "ruby-smoke",
    },
  },
)
begin
  resp = headered.admin.get_endpoints(limit: 1)
  puts "custom-headers smoke: ok (#{resp[:data].length} endpoints)"
rescue QuicknodeSdk::Error => e
  puts "custom-headers smoke error: #{e.message}"
end

# 2) Timeout path — unreachable base URL + 1s timeout forces a timeout
prev_url = ENV["QN_SDK__ADMIN__BASE_URL"]
prev_timeout = ENV["QN_SDK__HTTP__TIMEOUT_SECS"]
ENV["QN_SDK__ADMIN__BASE_URL"] = "http://10.255.255.1/"
ENV["QN_SDK__HTTP__TIMEOUT_SECS"] = "1"
begin
  blackhole = QuicknodeSdk::SDK.from_env
  begin
    blackhole.admin.get_endpoints(limit: 20)
    raise "expected timeout"
  rescue QuicknodeSdk::TimeoutError
    puts "timed out as expected"
  end
ensure
  ENV["QN_SDK__ADMIN__BASE_URL"] = prev_url
  ENV["QN_SDK__HTTP__TIMEOUT_SECS"] = prev_timeout
end
