require "json"
require_relative "../lib/quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env

# ── Read-only globals ─────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.list_chains)
puts "list_chains: #{resp["data"].length} chains"

resp = JSON.parse(qn.admin.get_endpoints(5, nil, nil, nil))
puts "get_endpoints: #{resp["data"].length} endpoints"

resp = JSON.parse(qn.admin.get_usage(nil, nil))
puts "get_usage: #{resp["data"].inspect}"

sleep 0.5

resp = JSON.parse(qn.admin.get_usage_by_endpoint(nil, nil))
puts "get_usage_by_endpoint: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.get_usage_by_method(nil, nil))
puts "get_usage_by_method: #{resp["data"].inspect}"

sleep 0.5

resp = JSON.parse(qn.admin.get_usage_by_chain(nil, nil))
puts "get_usage_by_chain: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.get_account_metrics("day", "requests", nil))
puts "get_account_metrics: #{resp["data"].length} series"

resp = JSON.parse(qn.admin.list_invoices)
puts "list_invoices: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.list_payments)
puts "list_payments: #{resp["data"].inspect}"

resp = JSON.parse(qn.admin.list_teams)
puts "list_teams: #{resp["data"].length} teams"

sleep 0.5

# ── Create endpoint ───────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_endpoint("ethereum", "mainnet"))
endpoint_id = resp.dig("data", "id")
unless endpoint_id
  puts "create_endpoint failed: #{resp["error"]}"
  exit 1
end
puts "create_endpoint: #{endpoint_id} (#{resp.dig("data", "http_url")})"

# ── Endpoint CRUD ─────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
puts "show_endpoint: #{resp.dig("data", "id")}"

qn.admin.update_endpoint(endpoint_id, "sdk-example")
puts "update_endpoint: ok"

sleep 0.5

resp = JSON.parse(qn.admin.update_endpoint_status(endpoint_id, "paused"))
puts "update_endpoint_status paused: #{resp["data"]}"

resp = JSON.parse(qn.admin.update_endpoint_status(endpoint_id, "active"))
puts "update_endpoint_status active: #{resp["data"]}"

# ── Tags ──────────────────────────────────────────────────────────────────────

qn.admin.create_tag(endpoint_id, "example-tag")
puts "create_tag: ok"

sleep 0.5
resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
tag_id = resp.dig("data", "tags", 0, "tag_id")&.to_s
if tag_id
  qn.admin.delete_tag(endpoint_id, tag_id)
  puts "delete_tag: ok"
end

# ── Logs & metrics ────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.get_endpoint_logs(
  endpoint_id, "2025-01-01T00:00:00Z", "2025-01-02T00:00:00Z", nil, nil, nil
))
puts "get_endpoint_logs: #{resp["data"].length} entries"

resp = JSON.parse(qn.admin.get_endpoint_metrics(endpoint_id, "day", "requests"))
puts "get_endpoint_metrics: #{resp["data"].length} series"

# ── Security options ──────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.get_security_options(endpoint_id))
puts "get_security_options: #{resp["data"].length} options"

resp = JSON.parse(qn.admin.update_security_options(
  endpoint_id, "enabled", nil, nil, nil, nil, nil, nil, nil, nil
))
puts "update_security_options: #{resp["data"].length} options"

# ── Token ─────────────────────────────────────────────────────────────────────

qn.admin.create_token(endpoint_id)
puts "create_token: ok"

resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
token_id = resp.dig("data", "security", "tokens", 0, "id")
if token_id
  resp = JSON.parse(qn.admin.delete_token(endpoint_id, token_id))
  puts "delete_token: #{resp["data"]}"
end

# ── Referrer ──────────────────────────────────────────────────────────────────

qn.admin.create_referrer(endpoint_id, "https://example.com")
puts "create_referrer: ok"

sleep 0.5
resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
referrer_id = resp.dig("data", "security", "referrers", 0, "id")
if referrer_id
  resp = JSON.parse(qn.admin.delete_referrer(endpoint_id, referrer_id))
  puts "delete_referrer: #{resp["data"]}"
end

# ── IP allowlist ──────────────────────────────────────────────────────────────

qn.admin.create_ip(endpoint_id, "192.0.2.1")
puts "create_ip: ok"

resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
ip_id = resp.dig("data", "security", "ips", 0, "id")

sleep 0.5
if ip_id
  resp = JSON.parse(qn.admin.delete_ip(endpoint_id, ip_id))
  puts "delete_ip: #{resp["data"]}"
end

# ── Domain mask ───────────────────────────────────────────────────────────────

qn.admin.create_domain_mask(endpoint_id, "example.com")
puts "create_domain_mask: ok"

resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
mask_id = resp.dig("data", "security", "domain_masks", 0, "id")
if mask_id
  resp = JSON.parse(qn.admin.delete_domain_mask(endpoint_id, mask_id))
  puts "delete_domain_mask: #{resp["data"]}"
end

# ── JWT (placeholder key will fail) ──────────────────────────────────────────

begin
  qn.admin.create_jwt(
    endpoint_id,
    "-----BEGIN PUBLIC KEY-----\nPLACEHOLDER\n-----END PUBLIC KEY-----",
    "kid1",
    "example-jwt"
  )
  puts "create_jwt: ok"
rescue => e
  puts "create_jwt error (expected with placeholder key): #{e}"
end

resp = JSON.parse(qn.admin.show_endpoint(endpoint_id))
jwt_id = resp.dig("data", "security", "jwts", 0, "id")
if jwt_id
  qn.admin.delete_jwt(endpoint_id, jwt_id)
  puts "delete_jwt: ok"
end

# ── Request filter ────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_request_filter(endpoint_id, ["eth_getBalance"]))
rf_id = resp.dig("data", "id")
puts "create_request_filter: #{resp["data"]}"

if rf_id
  qn.admin.update_request_filter(endpoint_id, rf_id, ["eth_call"])
  puts "update_request_filter: ok"

  qn.admin.delete_request_filter(endpoint_id, rf_id)
  puts "delete_request_filter: ok"
end

# ── IP custom header ──────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_or_update_ip_custom_header(endpoint_id, "X-Custom-Header"))
puts "create_or_update_ip_custom_header: #{resp["data"]}"

resp = JSON.parse(qn.admin.delete_ip_custom_header(endpoint_id))
puts "delete_ip_custom_header: #{resp["data"]}"

sleep 0.5

# ── Rate limits ───────────────────────────────────────────────────────────────

qn.admin.update_rate_limits(endpoint_id, 10, nil, nil)
puts "update_rate_limits: ok"

resp = JSON.parse(qn.admin.get_method_rate_limits(endpoint_id))
puts "get_method_rate_limits: #{resp["data"]}"

resp = JSON.parse(qn.admin.create_method_rate_limit(endpoint_id, "second", ["eth_call"], 5))
mrl_id = resp.dig("data", "id")
puts "create_method_rate_limit: #{resp["data"]}"

if mrl_id
  resp = JSON.parse(qn.admin.update_method_rate_limit(endpoint_id, mrl_id, nil, nil, 10))
  puts "update_method_rate_limit: #{resp["data"]}"

  qn.admin.delete_method_rate_limit(endpoint_id, mrl_id)
  puts "delete_method_rate_limit: ok"
end

sleep 0.5

# ── Multichain ────────────────────────────────────────────────────────────────

qn.admin.enable_multichain(endpoint_id)
puts "enable_multichain: ok"

qn.admin.disable_multichain(endpoint_id)
puts "disable_multichain: ok"

sleep 0.5

# ── Teams ─────────────────────────────────────────────────────────────────────

resp = JSON.parse(qn.admin.create_team("sdk-example-team"))
team_id = resp.dig("data", "id")
puts "create_team: #{resp["data"]}"

if team_id
  resp = JSON.parse(qn.admin.get_team(team_id))
  puts "get_team: #{resp.dig("data", "name")}"

  resp = JSON.parse(qn.admin.list_team_endpoints(team_id))
  puts "list_team_endpoints: #{resp["data"].length} endpoints"

  resp = JSON.parse(qn.admin.update_team_endpoints(team_id, [endpoint_id]))
  puts "update_team_endpoints: #{resp["data"]}"

  begin
    resp = JSON.parse(qn.admin.invite_team_member(team_id, "placeholder@example.com", nil, nil))
    puts "invite_team_member: #{resp["data"]}"
  rescue => e
    puts "invite_team_member error (expected with placeholder email): #{e}"
  end

  sleep 0.5

  resp = JSON.parse(qn.admin.delete_team(team_id))
  puts "delete_team: #{resp["data"]}"
end

sleep 0.5

# ── Cleanup endpoint ──────────────────────────────────────────────────────────

qn.admin.archive_endpoint(endpoint_id)
puts "archive_endpoint: ok"
