require_relative "../lib/quicknode_sdk"

qn = QuicknodeSdk::SDK.from_env

# Ensure Tooling Access is provisioned (idempotent; requires admin role).
status = qn.admin.tooling_access_status
puts "tooling access enabled: #{status["enabled"]}"
unless status["enabled"]
  begin
    enabled = qn.admin.enable_tooling_access
    puts "enabled tooling access: #{enabled["enabled"]}"
  rescue QuicknodeSdk::Error => e
    warn "could not enable tooling access: #{e.message}"
    exit
  end
end

# Make a JSON-RPC call. The SDK mints and refreshes the session JWT.
block_number = qn.rpc.call(method: "eth_blockNumber")
puts "eth_blockNumber => #{block_number}"

# Multichain: seed the per-network URL map (from the endpoint id in status),
# then route a call to a specific network by its key.
if status["endpoint_id"]
  urls = qn.admin.get_endpoint_urls(id: status["endpoint_id"])
  mc = urls.dig("data", "multichain_urls")
  if mc
    map = mc.transform_values { |v| v["http_url"] }
    qn.rpc.set_networks(networks: map)
    slot = qn.rpc.call(method: "getSlot", network: "solana-mainnet")
    puts "solana getSlot => #{slot}"
  end
end

# Demonstrate the typed JSON-RPC error path.
begin
  qn.rpc.call(method: "eth_getBalance", params: ["not-an-address"])
rescue QuicknodeSdk::RpcError => e
  puts "got expected RpcError: code=#{e.code} message=#{e.message}"
end

# Custom endpoint URL: send a call to a fully-formed HTTP URL, bypassing the
# Tooling Access endpoint and the session JWT entirely. Set it per-call here,
# or client-wide via RpcConfig(endpoint_url:).
custom_url = ENV.fetch("QN_RPC_ENDPOINT_URL", nil)
if custom_url
  result = qn.rpc.call(method: "eth_blockNumber", endpoint_url: custom_url)
  puts "custom endpoint eth_blockNumber => #{result}"
end
