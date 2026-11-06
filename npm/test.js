const assert = require("node:assert");
const errors = require("./errors.js");

async function testErrorMessages() {
  const bodylessKinds = [
    ["Config", errors.ConfigError, errors.QuicknodeError],
    ["Http", errors.HttpError, errors.QuicknodeError],
    ["Timeout", errors.TimeoutError, errors.HttpError],
    ["Connect", errors.ConnectionError, errors.HttpError],
    ["PaymentUnsupported", errors.PaymentUnsupportedError, errors.PaymentError],
    ["PaymentIndeterminate", errors.PaymentIndeterminateError, errors.PaymentError],
  ];
  for (const [kind, ErrorClass, BaseClass] of bodylessKinds) {
    for (const message of ["request failed", "", "café 🙂", "embedded\x1fmarker", "trailing\x1f\x1f"]) {
      const error = errors.fromNapiError(new Error(`[${kind}|-|-]${message}\x1f`));
      assert(error instanceof ErrorClass);
      assert(error instanceof BaseClass);
      assert.strictEqual(error.message, message, `${kind} message`);
    }
    const legacy = errors.fromNapiError(new Error(`[${kind}|-|-]legacy message`));
    assert.strictEqual(legacy.message, "legacy message");
  }

  // Body lengths count UTF-8 bytes, and the separator can also occur in data.
  for (const body of ["", "café 🙂\x1fbody"]) {
    const message = "response\x1fmessage";
    for (const [kind, status, ErrorClass] of [
      ["Api", 400, errors.ApiError],
      ["Decode", "-", errors.DecodeError],
      ["PaymentRejected", 402, errors.PaymentRejectedError],
    ]) {
      const error = errors.fromNapiError(new Error(
        `[${kind}|${status}|${Buffer.byteLength(body)}]${message}\x1f${body}`,
      ));
      assert(error instanceof ErrorClass);
      assert.strictEqual(error.message, message);
      assert.strictEqual(error.body, body);
      if (kind !== "Decode") assert.strictEqual(error.status, status);
    }
  }

  const rpcMessage = "RPC café 🙂\x1fmessage";
  const rpc = errors.fromNapiError(new Error(
    `[Rpc|-32600|${Buffer.byteLength(rpcMessage)}]RPC failed\x1f${rpcMessage}`,
  ));
  assert(rpc instanceof errors.RpcError);
  assert.strictEqual(rpc.message, rpcMessage);
  assert.strictEqual(rpc.code, -32600);

  const untagged = new Error("ordinary error\x1f");
  assert.strictEqual(errors.fromNapiError(untagged), untagged);
  assert.strictEqual(errors.fromNapiError("ordinary rejection"), "ordinary rejection");

  const client = errors.wrapClient({
    sync() { throw new Error("[Config|-|-]invalid config\x1f"); },
    async async() { throw new Error("[Timeout|-|-]request timed out\x1f"); },
  });
  assert.throws(() => client.sync(), (e) => e instanceof errors.ConfigError && e.message === "invalid config");
  await assert.rejects(() => client.async(), (e) => e instanceof errors.TimeoutError && e.message === "request timed out");
  console.log("node error messages OK");
}

async function main() {
  await testErrorMessages();
  const sdk = require("./sdk.js");
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

module.exports = { testErrorMessages };

if (require.main === module) {
  main().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}
