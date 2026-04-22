require "json"
require "securerandom"
require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

# ── Read-only globals ─────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.list_chains)
puts "list_chains: #{resp["data"].length} chains"

resp = JSON.parse(qn.admin.get_endpoints(limit: 5))
puts "get_endpoints: #{resp["data"].length} endpoints"

resp = JSON.parse(qn.admin.get_usage({}))
puts "get_usage: #{resp["data"].inspect}"

sleep 0.5

resp = JSON.parse(qn.admin.get_usage_by_endpoint({}))
puts "get_usage_by_endpoint: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.get_usage_by_method({}))
puts "get_usage_by_method: #{resp["data"].inspect}"

sleep 0.5

resp = JSON.parse(qn.admin.get_usage_by_chain({}))
puts "get_usage_by_chain: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.get_usage_by_tag({}))
puts "get_usage_by_tag: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.list_tags)
puts "list_tags: #{resp.dig("data", "tags")&.length || 0} tags"

resp = JSON.parse(qn.admin.get_account_metrics(period: "day", metric: "requests"))
puts "get_account_metrics: #{resp["data"].length} series"

resp = JSON.parse(qn.admin.list_invoices)
puts "list_invoices: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.list_payments)
puts "list_payments: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.list_teams)
puts "list_teams: #{resp["data"].length} teams"

sleep 0.5

# ── Create endpoint ───────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_endpoint(chain: "ethereum", network: "mainnet"))
endpoint_id = resp.dig("data", "id")
unless endpoint_id
  puts "create_endpoint failed: #{resp["error"]}"
  exit 1
end
puts "create_endpoint: #{endpoint_id} (#{resp.dig("data", "http_url")})"

# ── Endpoint CRUD ─────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
puts "show_endpoint: #{resp.dig("data", "id")}"

qn.admin.update_endpoint(id: endpoint_id, label: "sdk-example")
puts "update_endpoint: ok"

sleep 0.5

resp = JSON.parse(qn.admin.update_endpoint_status(id: endpoint_id, status: "paused"))
puts "update_endpoint_status paused: #{resp["data"]}"

resp = JSON.parse(qn.admin.update_endpoint_status(id: endpoint_id, status: "active"))
puts "update_endpoint_status active: #{resp["data"]}"

sleep 1

# ── Tags ──────────────────────────────────────────────────────────────────────

qn.admin.create_tag(id: endpoint_id, label: "example-tag")
puts "create_tag: ok"

sleep 0.5
resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
tag_id = resp.dig("data", "tags", 0, "tag_id")&.to_s
if tag_id
  qn.admin.delete_tag(id: endpoint_id, tag_id: tag_id)
  puts "delete_tag: ok"
end

# ── Logs & metrics ────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.get_endpoint_logs(
  id: endpoint_id,
  from_time: "2025-01-01T00:00:00Z",
  to_time: "2025-01-02T00:00:00Z"
))
puts "get_endpoint_logs: #{resp["data"].length} entries"

sleep 1

resp = JSON.parse(qn.admin.get_endpoint_metrics(id: endpoint_id, period: "day", metric: "credits_over_time"))
puts "get_endpoint_metrics: #{resp["data"].length} series"

# ── Security options ──────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.get_security_options(id: endpoint_id))
puts "get_security_options: #{resp["data"].length} options"

sleep 1

resp = JSON.parse(qn.admin.get_endpoint_security(id: endpoint_id))
puts "get_endpoint_security: has_data=#{!resp["data"].nil?}"

resp = JSON.parse(qn.admin.update_security_options(id: endpoint_id, tokens: "enabled"))
puts "update_security_options: #{resp["data"].length} options"

sleep 0.5


# ── Token ─────────────────────────────────────────────────────────────────────

qn.admin.create_token(id: endpoint_id)
puts "create_token: ok"

resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
token_id = resp.dig("data", "security", "tokens", 0, "id")
if token_id
  resp = JSON.parse(qn.admin.delete_token(id: endpoint_id, token_id: token_id))
  puts "delete_token: #{resp["data"]}"
end

# ── Referrer ──────────────────────────────────────────────────────────────────

qn.admin.create_referrer(id: endpoint_id, referrer: "https://example.com")
puts "create_referrer: ok"

sleep 0.5
resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
referrer_id = resp.dig("data", "security", "referrers", 0, "id")
if referrer_id
  resp = JSON.parse(qn.admin.delete_referrer(id: endpoint_id, referrer_id: referrer_id))
  puts "delete_referrer: #{resp["data"]}"
end

# ── IP allowlist ──────────────────────────────────────────────────────────────

qn.admin.create_ip(id: endpoint_id, ip: "192.0.2.1")
puts "create_ip: ok"

resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
ip_id = resp.dig("data", "security", "ips", 0, "id")

sleep 0.5
if ip_id
  resp = JSON.parse(qn.admin.delete_ip(id: endpoint_id, ip_id: ip_id))
  puts "delete_ip: #{resp["data"]}"
end

# ── Domain mask ───────────────────────────────────────────────────────────────

qn.admin.create_domain_mask(id: endpoint_id, domain_mask: "example.com")
puts "create_domain_mask: ok"

resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
mask_id = resp.dig("data", "security", "domain_masks", 0, "id")
if mask_id
  resp = JSON.parse(qn.admin.delete_domain_mask(id: endpoint_id, domain_mask_id: mask_id))
  puts "delete_domain_mask: #{resp["data"]}"
end

# ── JWT (placeholder key will fail) ──────────────────────────────────────────

begin
  qn.admin.create_jwt(
    id: endpoint_id,
    public_key: "-----BEGIN PUBLIC KEY-----\nPLACEHOLDER\n-----END PUBLIC KEY-----",
    kid: "kid1",
    name: "example-jwt"
  )
  puts "create_jwt: ok"
rescue => e
  puts "create_jwt error (expected with placeholder key): #{e}"
end

resp = JSON.parse(qn.admin.show_endpoint(id: endpoint_id))
jwt_id = resp.dig("data", "security", "jwts", 0, "id")
if jwt_id
  qn.admin.delete_jwt(id: endpoint_id, jwt_id: jwt_id)
  puts "delete_jwt: ok"
end

# ── Request filter ────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_request_filter(id: endpoint_id, methods: ["eth_getBalance"]))
rf_id = resp.dig("data", "id")
puts "create_request_filter: #{resp["data"]}"

if rf_id
  qn.admin.update_request_filter(id: endpoint_id, request_filter_id: rf_id, methods: ["eth_call"])
  puts "update_request_filter: ok"

  qn.admin.delete_request_filter(id: endpoint_id, request_filter_id: rf_id)
  puts "delete_request_filter: ok"
end

# ── IP custom header ──────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_or_update_ip_custom_header(id: endpoint_id, header_name: "X-Custom-Header"))
puts "create_or_update_ip_custom_header: #{resp["data"]}"

resp = JSON.parse(qn.admin.delete_ip_custom_header(id: endpoint_id))
puts "delete_ip_custom_header: #{resp["data"]}"

sleep 0.5

# ── Rate limits ───────────────────────────────────────────────────────────────

#qn.admin.update_rate_limits(id: endpoint_id, rps: 3)
#puts "update_rate_limits: ok"

resp = JSON.parse(qn.admin.get_method_rate_limits(id: endpoint_id))
puts "get_method_rate_limits: #{resp["data"]}"

resp = JSON.parse(qn.admin.create_method_rate_limit(id: endpoint_id, interval: "second", methods: ["eth_call"], rate: 5))
mrl_id = resp.dig("data", "id")
puts "create_method_rate_limit: #{resp["data"]}"

if mrl_id
  resp = JSON.parse(qn.admin.update_method_rate_limit(id: endpoint_id, method_rate_limit_id: mrl_id, rate: 10))
  puts "update_method_rate_limit: #{resp["data"]}"

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

# ── Bulk endpoint ops (single-endpoint batch) ────────────────────────────────

resp = JSON.parse(qn.admin.bulk_update_endpoint_status(ids: [endpoint_id], status: "paused"))
puts "bulk_update_endpoint_status paused: #{resp["data"]}"

resp = JSON.parse(qn.admin.bulk_update_endpoint_status(ids: [endpoint_id], status: "active"))
puts "bulk_update_endpoint_status active: #{resp["data"]}"

# ── Account-level tags (bulk_add/remove + rename/delete) ─────────────────────

tag_suffix = SecureRandom.hex(4)
resp = JSON.parse(qn.admin.bulk_add_tag(ids: [endpoint_id], label: "sdk-bulk-#{tag_suffix}"))
puts "bulk_add_tag: #{resp["data"]}"
bulk_tag_id = resp.dig("data", "tag", "tag_id")

sleep 0.5

if bulk_tag_id
  resp = JSON.parse(qn.admin.rename_tag(id: bulk_tag_id, label: "sdk-renamed-#{tag_suffix}"))
  puts "rename_tag: #{resp["data"]}"

  resp = JSON.parse(qn.admin.bulk_remove_tag(ids: [endpoint_id], tag_id: bulk_tag_id))
  puts "bulk_remove_tag: #{resp["data"]}"

  resp = JSON.parse(qn.admin.delete_account_tag(id: bulk_tag_id))
  puts "delete_account_tag: #{resp["data"]}"
end

sleep 0.5

# ── Teams ─────────────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_team(name: "sdk-example-team"))
team_id = resp.dig("data", "id")
puts "create_team: #{resp["data"]}"

sleep 0.5
if team_id
  resp = JSON.parse(qn.admin.get_team(id: team_id))
  puts "get_team: #{resp.dig("data", "name")}"

  resp = JSON.parse(qn.admin.list_team_endpoints(id: team_id))
  puts "list_team_endpoints: #{resp["data"].length} endpoints"

  sleep 0.5

  resp = JSON.parse(qn.admin.update_team_endpoints(id: team_id, endpoint_ids: [endpoint_id]))
  puts "update_team_endpoints: #{resp["data"]}"

  sleep 0.5

  begin
    resp = JSON.parse(qn.admin.invite_team_member(id: team_id, email: "placeholder@example.com"))
    puts "invite_team_member: #{resp["data"]}"
  rescue => e
    puts "invite_team_member error (expected with placeholder email): #{e}"
  end

  sleep 0.5

  resp = JSON.parse(qn.admin.delete_team(id: team_id))
  puts "delete_team: #{resp["data"]}"
end

sleep 0.5

# ── Cleanup endpoint ──────────────────────────────────────────────────────────

qn.admin.archive_endpoint(id: endpoint_id)
puts "archive_endpoint: ok"

# ── Error handling ───────────────────────────────────────────────────────────

# 1) API error path — 404 on a bogus endpoint id.
begin
  qn.admin.show_endpoint(id: "does-not-exist")
  raise "expected 404"
rescue QuicknodeSdk::ApiError => e
  raise "expected QuicknodeSdk::Error subclass" unless e.is_a?(QuicknodeSdk::Error)
  raise "expected 404, got #{e.status}" unless e.status == 404
  puts "api error #{e.status}: #{e.body[0, 80]}"
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
