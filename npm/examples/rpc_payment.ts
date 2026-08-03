// Crypto-micropayment lane for rpc.call: pay per RPC request with a stablecoin
// instead of an account API key, against Quicknode's x402/MPP gateways.
//
// ⚠️ MOVES REAL FUNDS when it settles. Use a throwaway, minimally-funded
// wallet. Reads the key from QN_PAYMENT_KEY — never hard-code it.
//
// Run (x402/EVM on Base Sepolia testnet):
//   QN_PAYMENT_KEY=0x<throwaway-key> npx tsx examples/rpc_payment.ts
//
// Run the x402 drawdown lane (authenticate once, then 1 credit per call):
//   QN_PAYMENT_KEY=0x<key> QN_PAYMENT_LANE=drawdown npx tsx examples/rpc_payment.ts

import {
  QuicknodeSdk,
  PaymentIndeterminateError,
  PaymentRejectedError,
  generatePaymentWallet,
} from "@quicknode/sdk";

const key = process.env.QN_PAYMENT_KEY;
if (!key) {
  // Wallet generation is offline: no gateway, no funds. The key is returned
  // exactly once — persist it here or it is gone.
  const wallet = generatePaymentWallet("evm");
  console.log("generated a throwaway wallet:", wallet.address);
  console.log("fund it, then re-run with QN_PAYMENT_KEY set to its key");
  process.exit(0);
}

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

// The x402 drawdown lane: authenticate once, then draw 1 credit per call.
// Cheaper per call than the per-request lane, and the session JWT is free to
// mint. Persist the session object between runs.
async function drawdownDemo() {
  // Derived offline from the key — no network round trip. Use it to key a
  // per-wallet session cache.
  console.log("payment wallet:", qn.rpc.paymentAddress());

  const session = await qn.rpc.gatewayAuthenticate();
  console.log("session account:", session.accountId, "expires:", session.expUnix);

  const balance = await qn.rpc.gatewayCredits(session);
  console.log("credits:", balance.credits);

  if (balance.credits === 0) {
    // Testnet faucet: allowed once per account, and it returns the funding
    // transaction — NOT a balance. Read the balance separately afterwards.
    try {
      const drip = await qn.rpc.gatewayDrip(session);
      console.log("faucet tx:", drip.transactionHash);
    } catch (e) {
      if (e instanceof PaymentRejectedError) {
        console.error(`faucet refused (${e.status}):`, e.body);
      } else throw e;
    }
  }

  const result = await qn.rpc.gatewayDrawdownCall(
    "eth_blockNumber",
    session,
    "base-sepolia",
  );
  console.log("drawdown eth_blockNumber =>", result);
}

async function main() {
  if (process.env.QN_PAYMENT_LANE === "drawdown") {
    await drawdownDemo();
    return;
  }
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
