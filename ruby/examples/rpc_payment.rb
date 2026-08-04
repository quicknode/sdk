# frozen_string_literal: true

# Crypto-micropayment lane for rpc.call: pay per RPC request with a stablecoin
# instead of an account API key, against Quicknode's x402/MPP gateways.
#
# ⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded wallet.
# Reads the key from QN_PAYMENT_KEY — never hard-code it.
#
# Run (x402/EVM on Base Sepolia testnet):
#   QN_PAYMENT_KEY=0x<throwaway-key> ruby -Ilib examples/rpc_payment.rb
#
# Run the x402 drawdown lane (authenticate once, then 1 credit per call):
#   QN_PAYMENT_KEY=0x<key> QN_PAYMENT_LANE=drawdown ruby -Ilib examples/rpc_payment.rb
# Solana drawdown uses a base58 key and a solana:<genesis-hash> pay network;
# the SDK authenticates with SIWS and signs the credit offer with x402/Solana.

require "quicknode_sdk"

# No-funds selfcheck that always runs: error hierarchy + the network-required
# ConfigError. Asserts the payment surface is wired without moving money.
raise "hierarchy" unless QuicknodeSdk::PaymentIndeterminateError < QuicknodeSdk::PaymentError

check_sdk = QuicknodeSdk::SDK.from_config(
  api_key: nil,
  rpc: { payment: { scheme: "x402", key: "0xabc", pay_network: "eip155:84532",
                    asset: "0xUSDC", max_amount: "10000" } }
)
begin
  check_sdk.rpc.call(method: "eth_blockNumber")
  raise "expected a ConfigError (payment lane requires network)"
rescue QuicknodeSdk::ConfigError => e
  raise "wrong message: #{e.message}" unless e.message.include?("requires")
end

# Wallet generation is offline; persist the returned key.
wallet = QuicknodeSdk.generate_payment_wallet(chain: "evm")
raise "address" unless wallet[:address].start_with?("0x") && wallet[:address].length == 42
raise "chain" unless wallet[:chain] == "evm"
raise "key" unless wallet[:key].is_a?(String)
begin
  QuicknodeSdk.generate_payment_wallet(chain: "dogecoin")
  raise "expected an ArgumentError for an unknown chain"
rescue ArgumentError
  # expected
end

# Amounts are decimal strings; reject non-integers.
begin
  check_sdk.rpc.mpp_open(deposit: "12.5")
  raise "expected an ArgumentError for a non-integer deposit"
rescue ArgumentError => e
  raise "wrong message: #{e.message}" unless e.message.include?("decimal base-unit")
end

puts "selfcheck OK: error classes, wallet generation, u128 String amounts"

key = ENV["QN_PAYMENT_KEY"]
unless key
  puts "set QN_PAYMENT_KEY to a throwaway key to run the live payment call"
  exit 0
end

# Keyless SDK. Do not log this config; it contains the private key.
sdk = QuicknodeSdk::SDK.from_config(
  api_key: nil,
  rpc: {
    payment: {
      scheme: "x402",
      key: key,
      # Base Sepolia testnet USDC (x402/EVM).
      pay_network: "eip155:84532",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      # Spend ceiling in asset base units.
      max_amount: "10000"
      # Set svm_rpc_url: for x402/Solana at volume.
    }
  }
)

begin
  # Query network is independent of the payment network.
  resp = sdk.rpc.call_with_receipt(method: "eth_blockNumber", params: [], network: "base-sepolia")
  puts "paid eth_blockNumber => #{resp["result"]}"
  # x402 does not return a settlement receipt.
  puts "settlement reference: #{resp.dig("payment_receipt", "reference")}" if resp["payment_receipt"]
rescue QuicknodeSdk::PaymentIndeterminateError => e
  # The request may have settled. Do not retry blindly.
  warn "payment indeterminate — do not retry: #{e.message}"
rescue QuicknodeSdk::PaymentRejectedError => e
  warn "payment rejected (#{e.status}): #{e.body}"
end

# Drawdown lane: authenticate once, then spend one credit per call.
if ENV["QN_PAYMENT_LANE"] == "drawdown"
  # Derived locally; use it to key a session cache.
  puts "payment wallet: #{sdk.rpc.payment_address}"

  session = sdk.rpc.gateway_authenticate
  puts "session account: #{session[:account_id]} expires: #{session[:exp_unix]}"

  balance = sdk.rpc.gateway_credits(session: session)
  puts "credits: #{balance[:credits]}"

  if balance[:credits].zero?
    # The faucet returns a funding transaction, not a balance.
    begin
      drip = sdk.rpc.gateway_drip(session: session)
      puts "faucet tx: #{drip[:transaction_hash]}"
    rescue QuicknodeSdk::PaymentRejectedError => e
      warn "faucet refused (#{e.status}): #{e.body}"
    end
  end

  result = sdk.rpc.gateway_drawdown_call(
    method: "eth_blockNumber", session: session, network: "base-sepolia"
  )
  puts "drawdown eth_blockNumber => #{result}"
end
