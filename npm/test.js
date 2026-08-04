const assert = require("node:assert");
const sdk = require("./sdk.js");

async function main() {
  // Payment errors preserve the expected hierarchy.
  assert(sdk.PaymentError.prototype instanceof sdk.QuicknodeError);
  assert(sdk.PaymentUnsupportedError.prototype instanceof sdk.PaymentError);
  assert(sdk.PaymentRejectedError.prototype instanceof sdk.PaymentError);
  assert(sdk.PaymentIndeterminateError.prototype instanceof sdk.PaymentError);

  // Payment configuration does not require an API key.
  const qn = new sdk.QuicknodeSdk({
    rpc: {
      payment: {
        scheme: "x402",
        key: "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        payNetwork: "eip155:84532",
        asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        maxAmount: "10000",
      },
    },
  });
  assert(typeof qn.rpc.callWithReceipt === "function");

  // network is required for payment calls.
  await assert.rejects(
    () => qn.rpc.call("eth_blockNumber", []),
    (e) => e instanceof sdk.ConfigError && /requires `network`/.test(e.message),
  );

  // Verify the payment methods are exposed.
  for (const m of [
    "paymentAddress", "gatewayAuthenticate", "gatewayCredits", "gatewayBuyCredits",
    "gatewayDrip", "gatewayDrawdownCall", "mppOpen", "mppTopUp", "mppClose",
    "mppStatus", "mppSessionCall",
  ]) {
    assert(typeof qn.rpc[m] === "function", `rpc.${m} missing`);
  }

  // Wallet generation is offline and returns the key exactly once.
  const wallet = sdk.generatePaymentWallet("evm");
  assert(wallet.address.startsWith("0x") && wallet.address.length === 42);
  assert.equal(wallet.chain, "evm");
  assert.equal(typeof wallet.key, "string");

  // Module-level errors must be mapped to typed errors.
  assert.throws(
    () => sdk.generatePaymentWallet("dogecoin"),
    (e) => e instanceof sdk.ConfigError,
  );

  // Reject non-integer base-unit amounts.
  await assert.rejects(
    () => qn.rpc.mppOpen("12.5"),
    (e) => e instanceof sdk.ConfigError && /decimal base-unit/.test(e.message),
  );

  // Malformed channel objects report the missing field.
  await assert.rejects(
    () => qn.rpc.mppStatus({ channelId: "0xabc" }),
    (e) => e instanceof sdk.ConfigError && /missing token/.test(e.message),
  );

  console.log("node payment surface OK");
  return true;
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
