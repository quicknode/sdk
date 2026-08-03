const assert = require("node:assert");
const sdk = require("./sdk.js");

async function main() {
  // Payment-lane error classes are exported and form the expected hierarchy.
  assert(sdk.PaymentError.prototype instanceof sdk.QuicknodeError);
  assert(sdk.PaymentUnsupportedError.prototype instanceof sdk.PaymentError);
  assert(sdk.PaymentRejectedError.prototype instanceof sdk.PaymentError);
  assert(sdk.PaymentIndeterminateError.prototype instanceof sdk.PaymentError);

  // A keyless SDK with a payment lane constructs without an API key.
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

  // The payment lane requires a `network`; omitting it is a ConfigError.
  await assert.rejects(
    () => qn.rpc.call("eth_blockNumber", []),
    (e) => e instanceof sdk.ConfigError && /requires `network`/.test(e.message),
  );

  // The whole channel/drawdown surface is reachable from JS.
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

  // Module-level functions are not covered by wrapClient, so their errors must
  // be translated explicitly — a bare napi Error here means that regressed.
  assert.throws(
    () => sdk.generatePaymentWallet("dogecoin"),
    (e) => e instanceof sdk.ConfigError,
  );

  // Base-unit amounts are decimal strings because u128 exceeds a JS number.
  // A non-integer must be refused rather than coerced.
  await assert.rejects(
    () => qn.rpc.mppOpen("12.5"),
    (e) => e instanceof sdk.ConfigError && /decimal base-unit/.test(e.message),
  );

  // A malformed channel object names the field that is wrong.
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
