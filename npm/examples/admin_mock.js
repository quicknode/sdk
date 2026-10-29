// Offline binding checks. Run through scripts/test_admin_bindings.py.
const assert = require("node:assert/strict");
const { QuicknodeSdk, ApiError } = require("../sdk.js");
const pattern = { method: [{ prefix: "eth_" }], "anything-but": [null, { nested_key: [true, 7.5] }] };

async function main() {
  const { admin } = new QuicknodeSdk({ apiKey: "fixture-key", admin: { baseUrl: process.env.ADMIN_MOCK_URL } });
  const created = await admin.createRequestFilterPattern("12", { pattern });
  assert.deepEqual(created.data.pattern, pattern);
  await admin.updateRequestFilterPattern("12", "filter", { pattern });
  assert.deepEqual((await admin.getRequestFilters("12")).data[0].pattern, pattern);
  const security = (await admin.getEndpointSecurity("12")).data;
  assert.deepEqual(security.requestFilters[0].method, pattern.method);
  assert.deepEqual(security.requestFilters[0].params["anything-but"], pattern["anything-but"]);
  assert.equal(security.options.responseLogging, true);
  assert.equal(security.blockedIps[0].ip, "192.0.2.0/24");
  assert.deepEqual((await admin.showEndpoint("12")).data.security.requestFilters[0].method, pattern.method);
  assert.equal((await admin.createTokenWithResponse("12")).data.id, "token");
  assert.equal((await admin.updateMethodRateLimit("12", "rule", { limitByIp: false })).data.limitByIp, false);
  await admin.updateRateLimits("per-ip", { limitByIp: true });
  assert.equal((await admin.updateRateLimitsWithResponse("endpoint-wide", { limitByIp: false })).data.limits.rps, 10);
  assert.equal((await admin.updateRateLimitsWithResponse("combined", { rateLimits: { rps: 10 }, limitByIp: false })).data.limits.rps, 10);
  await admin.updateRateLimits("limits-only", { rateLimits: { rps: 10 } });
  await assert.rejects(admin.updateRateLimits("empty", {}), error => error instanceof ApiError && error.status === 400 && error.body.includes("at least one property"));
  const summary = (await admin.getUsageRpcSummary({ last: "1d", groupBy: ["method", "endpoint"] })).data;
  assert.equal(summary.totals.rpcCredits, 7.5);
  assert.equal(summary.breakdown[0].requests, null);
  assert.equal(summary.breakdown[0].method, null);
  assert.equal(Object.hasOwn(summary.breakdown[0], "endpoint"), false);
  assert.equal(summary.from, "2026-10-01T00:00:00Z");
  const page = (await admin.getUsageRpcSummary({ cursor: summary.nextCursor })).data;
  assert.equal(page.totals.rpcCredits, 7.5);
  assert.equal(page.nextCursor, undefined);
  const traffic = (await admin.getTrafficSummary({ last: "1d", groupBy: ["method"] })).data;
  assert.equal(traffic.breakdown[0].successfulRequests, null);
  assert.equal(traffic.breakdown[0].method, null);
  assert.equal(Object.hasOwn(traffic.breakdown[0], "endpoint"), false);
  assert.equal((await admin.listTransactions({ last: "1d" })).data.transactions[0].height, "9007199254740993");
  const absent = (await admin.getTransaction("ethereum-mainnet", "hash", { includePayloads: false })).data;
  assert.equal(absent.exists, false);
  assert.equal(absent.data, undefined);
  const detail = (await admin.getTransaction("ethereum-mainnet", "hash", { includePayloads: true })).data.data;
  assert.equal(detail.height, "9007199254740993");
  assert.equal(detail.attempts[0].payloads.request, ' { "method" : "eth_sendRawTransaction" }\n');
  assert.equal(detail.attempts[0].payloads.response, "plain text");
  await assert.rejects(admin.getEndpointSecurity("missing"), error => error instanceof ApiError && error.status === 404 && error.body.includes("fixture missing endpoint"));
}
main().catch(error => { console.error(error); process.exitCode = 1; });
