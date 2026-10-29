# Offline binding checks. Run through scripts/test_admin_bindings.py.
require "quicknode_sdk"

def check(value)
  raise "Admin binding assertion failed" unless value
end

pattern = {"method" => [{"prefix" => "eth_"}], "anything-but" => [nil, {"nested_key" => [true, 7.5]}]}
admin = QuicknodeSdk::SDK.from_config(api_key: "fixture-key", admin: {base_url: ENV.fetch("ADMIN_MOCK_URL")}).admin
created = admin.create_request_filter_pattern(id: "12", pattern: pattern)
check(created[:data][:pattern] == pattern)
admin.update_request_filter_pattern(id: "12", request_filter_id: "filter", pattern: pattern)
check(admin.get_request_filters(id: "12")[:data][0][:pattern] == pattern)
security = admin.get_endpoint_security(id: "12")[:data]
check(security[:request_filters][0][:method] == pattern["method"])
check(security[:request_filters][0][:params]["anything-but"] == pattern["anything-but"])
check(security[:options][:responseLogging] == true)
check(security[:blocked_ips][0][:ip] == "192.0.2.0/24")
check(admin.show_endpoint(id: "12")[:data][:security][:request_filters][0][:method] == pattern["method"])
check(admin.create_token_with_response(id: "12")[:data][:id] == "token")
check(admin.update_method_rate_limit(id: "12", method_rate_limit_id: "rule", limit_by_ip: false)[:data][:limit_by_ip] == false)
admin.update_rate_limits(id: "per-ip", limit_by_ip: true)
check(admin.update_rate_limits_with_response(id: "endpoint-wide", limit_by_ip: false)[:data][:limits][:rps] == 10)
check(admin.update_rate_limits_with_response(id: "combined", rps: 10, limit_by_ip: false)[:data][:limits][:rps] == 10)
admin.update_rate_limits(id: "limits-only", rps: 10)
begin
  admin.update_rate_limits(id: "empty")
  raise "Expected ApiError for empty update"
rescue QuicknodeSdk::ApiError => error
  check(error.status == 400 && error.body.include?("at least one property"))
end
summary = admin.get_usage_rpc_summary(last: "1d", group_by: ["method", "endpoint"])[:data]
check(summary[:totals][:rpc_credits] == 7.5)
row = summary[:breakdown][0]
check(row.key?(:requests) && row[:requests].nil?)
check(row.key?(:method) && row[:method].nil?)
check(summary[:from] == "2026-10-01T00:00:00Z")
page = admin.get_usage_rpc_summary(cursor: summary[:next_cursor])[:data]
check(page[:totals][:rpc_credits] == 7.5 && page[:next_cursor].nil?)
traffic_data = admin.get_traffic_summary(last: "1d", group_by: ["method", "endpoint"])[:data]
traffic = traffic_data[:breakdown][0]
check(traffic.key?(:successful_requests) && traffic[:successful_requests].nil?)
check(traffic.key?(:method) && traffic[:method].nil?)
errors = admin.get_errors_summary(last: "1d", group_by: ["endpoint"], endpoints: [4123, 4124])[:data]
[summary, traffic_data, errors].each do |response|
  rows = response[:breakdown]
  check(rows[0][:endpoint] == 4123)
  check(rows[1].key?(:endpoint) && rows[1][:endpoint].nil?)
  check(!rows[2].key?(:endpoint))
end
rows = admin.list_transactions(last: "1d", endpoints: [4123, 4124])[:data][:transactions]
check(rows[0][:height] == "9007199254740993")
check(rows[0][:endpoint_ids] == [4123, nil, 4124])
absent = admin.get_transaction(network: "ethereum-mainnet", hash: "hash", include_payloads: false)[:data]
check(absent[:exists] == false && absent[:data].nil?)
detail = admin.get_transaction(network: "ethereum-mainnet", hash: "hash", include_payloads: true)[:data][:data]
check(detail[:height] == "9007199254740993")
check(detail[:endpoint_ids] == [4123, nil, 4124])
check(detail[:attempts][0][:endpoint_id] == 4123)
check(detail[:attempts][0][:payloads][:request] == " { \"method\" : \"eth_sendRawTransaction\" }\n")
check(detail[:attempts][0][:payloads][:response] == "plain text")
begin
  admin.get_endpoint_security(id: "missing")
  raise "Expected ApiError"
rescue QuicknodeSdk::ApiError => error
  check(error.status == 404 && error.body.include?("fixture missing endpoint"))
end
