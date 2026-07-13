# frozen_string_literal: true

# Crypto-micropayment lane for rpc.call: pay per RPC request with a stablecoin
# instead of an account API key, against Quicknode's x402/MPP gateways.
#
# ⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded wallet.
# Reads the key from QN_PAYMENT_KEY — never hard-code it.
#
# Run (x402/EVM on Base Sepolia testnet):
#   QN_PAYMENT_KEY=0x<throwaway-key> ruby -Ilib examples/rpc_payment.rb

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

  puts "selfcheck OK: payment error classes + network-required ConfigError"
end

key = ENV["QN_PAYMENT_KEY"]
unless key
  puts "set QN_PAYMENT_KEY to a throwaway key to run the live payment call"
  exit 0
end

# A keyless SDK: the payment lane needs no account API key. Do NOT log the
# config hash — the `key` field is readable (like ethers' .privateKey).
sdk = QuicknodeSdk::SDK.from_config(
  api_key: nil,
  rpc: {
    payment: {
      scheme: "x402",
      key: key,
      # Base Sepolia testnet USDC (x402/EVM).
      pay_network: "eip155:84532",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      # Spend ceiling in base units of the asset (required).
      max_amount: "10000"
      # For x402/Solana at any volume, set svm_rpc_url: to your own Solana RPC —
      # the public default rate-limits aggressively.
    }
  }
)

begin
  # `network` is the QUERY chain (gateway path slug), independent of the pay
  # network. The SDK runs the 402 -> sign -> resend handshake.
  resp = sdk.rpc.call_with_receipt(method: "eth_blockNumber", params: [], network: "base-sepolia")
  puts "paid eth_blockNumber => #{resp["result"]}"
  # payment_receipt is set on the MPP lane (reference = settlement tx hash),
  # nil for x402.
  puts "settlement reference: #{resp.dig("payment_receipt", "reference")}" if resp["payment_receipt"]
rescue QuicknodeSdk::PaymentIndeterminateError => e
  # The paid request was sent but the response was lost — you may already have
  # been charged. Do NOT blindly retry.
  warn "payment indeterminate — do not retry: #{e.message}"
rescue QuicknodeSdk::PaymentRejectedError => e
  warn "payment rejected (#{e.status}): #{e.body}"
end
