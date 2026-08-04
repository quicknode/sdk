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
// Solana drawdown uses a base58 key and a solana:<genesis-hash> pay network;
// the SDK authenticates with SIWS and signs the credit offer with x402/Solana.

import {
  QuicknodeSdk,
  PaymentIndeterminateError,
  PaymentRejectedError,
  generatePaymentWallet,
} from "@quicknode/sdk";

const key = process.env.QN_PAYMENT_KEY;
if (!key) {
  // Wallet generation is offline; persist the key returned here.
  const wallet = generatePaymentWallet("evm");
  console.log("generated a throwaway wallet:", wallet.address);
  console.log("fund it, then re-run with QN_PAYMENT_KEY set to its key");
  process.exit(0);
}

// Keyless SDK. Do not log this config; it contains the private key.
const qn = new QuicknodeSdk({
  rpc: {
    payment: {
      scheme: "x402",
      key,
      // Base Sepolia testnet USDC (x402/EVM).
      payNetwork: "eip155:84532",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      // Spend ceiling in asset base units.
      maxAmount: "10000",
      // Set svmRpcUrl for x402/Solana at volume.
    },
  },
});

// Drawdown lane: authenticate once, then spend one credit per call.
async function drawdownDemo() {
  // Derived locally; use it to key a session cache.
  console.log("payment wallet:", qn.rpc.paymentAddress());

  const session = await qn.rpc.gatewayAuthenticate();
  console.log("session account:", session.accountId, "expires:", session.expUnix);

  const balance = await qn.rpc.gatewayCredits(session);
  console.log("credits:", balance.credits);

  if (balance.credits === 0) {
    // The faucet returns a funding transaction, not a balance.
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
    // Query network is independent of the payment network.
    const { result, paymentReceipt } = await qn.rpc.callWithReceipt(
      "eth_blockNumber",
      [],
      "base-sepolia",
    );
    console.log("paid eth_blockNumber =>", result);
    // x402 does not return a settlement receipt.
    if (paymentReceipt) console.log("settlement reference:", paymentReceipt.reference);
  } catch (e) {
    if (e instanceof PaymentIndeterminateError) {
      // The request may have settled. Do not retry blindly.
      console.error("payment indeterminate — do not retry:", e.message);
    } else if (e instanceof PaymentRejectedError) {
      console.error(`payment rejected (${e.status}):`, e.body);
    } else {
      throw e;
    }
  }
}

main();
