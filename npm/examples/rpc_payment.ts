// Crypto-micropayment lane for rpc.call: pay per RPC request with a stablecoin
// instead of an account API key, against Quicknode's x402/MPP gateways.
//
// ⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded
// wallet. Reads the key from QN_PAYMENT_KEY — never hard-code it.
//
// Run (x402/EVM on Base Sepolia testnet):
//   QN_PAYMENT_KEY=0x<throwaway-key> npx tsx examples/rpc_payment.ts

import {
  QuicknodeSdk,
  PaymentIndeterminateError,
  PaymentRejectedError,
} from "@quicknode/sdk";

const key = process.env.QN_PAYMENT_KEY;
if (!key) throw new Error("set QN_PAYMENT_KEY to a throwaway key");

// A keyless SDK: the payment lane needs no account API key. Do NOT log the
// config object — the `key` field is readable.
const qn = new QuicknodeSdk({
  rpc: {
    payment: {
      scheme: "x402",
      key,
      // Base Sepolia testnet USDC (x402/EVM).
      payNetwork: "eip155:84532",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      // Spend ceiling in base units of the asset (required).
      maxAmount: "10000",
      // For x402/Solana at any volume, set svmRpcUrl to your own Solana RPC —
      // the public default rate-limits aggressively.
    },
  },
});

async function main() {
  try {
    // `network` is the QUERY chain (gateway path slug), independent of the pay
    // network. The SDK runs the 402 -> sign -> resend handshake.
    const { result, paymentReceipt } = await qn.rpc.callWithReceipt(
      "eth_blockNumber",
      [],
      "base-sepolia",
    );
    console.log("paid eth_blockNumber =>", result);
    // paymentReceipt is set on the MPP lane (reference = settlement tx hash),
    // null for x402.
    if (paymentReceipt) console.log("settlement reference:", paymentReceipt.reference);
  } catch (e) {
    if (e instanceof PaymentIndeterminateError) {
      // The paid request was sent but the response was lost — you may already
      // have been charged. Do NOT blindly retry.
      console.error("payment indeterminate — do not retry:", e.message);
    } else if (e instanceof PaymentRejectedError) {
      console.error(`payment rejected (${e.status}):`, e.body);
    } else {
      throw e;
    }
  }
}

main();
