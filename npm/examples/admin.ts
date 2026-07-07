import {
  QuicknodeSdk,
  ApiError,
  TimeoutError,
  QuicknodeError,
} from "../sdk";

async function main() {
  const qn = QuicknodeSdk.fromEnv();

  const account = await qn.admin.accountInfo();
  if (account.data) {
    const a = account.data;
    const plan = a.subscription?.planName ?? "<none>";
    console.log(`account ${a.id} | ${a.name} | billing=${a.billingVersion} | plan=${plan}`);
  }

  const response = await qn.admin.getEndpoints({
    limit: 20,
    sortBy: "created_at",
    sortDirection: "desc",
  });
  if (response.pagination) {
    const p = response.pagination;
    console.log(`${response.data.length} of ${p.total} (offset ${p.offset}, limit ${p.limit})`);
  }
  for (const ep of response.data) {
    console.log(
      `${ep.id} | ${ep.name} | ${ep.status} | ${ep.network} | ` +
        `dedicated=${ep.isDedicated} flat=${ep.isFlatRate} multichain=${ep.isMultichain}`,
    );
  }

  const tags = await qn.admin.listTags();
  if (tags.data) {
    console.log(`account tags: ${tags.data.tags.length}`);
  }

  const metrics = await qn.admin.getAccountMetrics({
    period: "day",
    metric: "credits_over_time",
  });
  const firstTag = metrics.data[0]?.tag.join(":") ?? "<none>";
  console.log(`getAccountMetrics: ${metrics.data.length} series, first tag: ${firstTag}`);

  if (response.data.length > 0) {
    const epId = response.data[0].id;
    const sec = await qn.admin.getEndpointSecurity(epId);
    console.log(`getEndpointSecurity: has_data=${sec.data !== undefined && sec.data !== null}`);

    const urls = await qn.admin.getEndpointUrls(epId);
    if (urls.data) {
      const mc = urls.data.multichainUrls;
      const networks = mc ? Object.keys(mc) : null;
      console.log(`getEndpointUrls: http=${urls.data.httpUrl} multichain_networks=${networks}`);
    }

    const rlBefore = await qn.admin.getRateLimits(epId);
    if (rlBefore.data) {
      for (const row of rlBefore.data.rateLimits) {
        console.log(
          `getRateLimits before PATCH: bucket=${row.bucket} ` +
            `rate_limit=${row.rateLimit} source=${row.source} id=${row.id}`,
        );
      }
    }

    await qn.admin.updateRateLimits(epId, { rateLimits: { rps: 3 } });
    console.log("updateRateLimits: ok");

    const rlAfter = await qn.admin.getRateLimits(epId);
    if (rlAfter.data) {
      for (const row of rlAfter.data.rateLimits) {
        console.log(
          `getRateLimits after PATCH: bucket=${row.bucket} ` +
            `rate_limit=${row.rateLimit} source=${row.source} id=${row.id}`,
        );
      }
    }
  }

  // ── Error handling ──────────────────────────────────────────────────
  // 1) API error path — 404 on a bogus endpoint id.
  try {
    await qn.admin.showEndpoint("does-not-exist");
  } catch (e) {
    if (!(e instanceof ApiError)) throw e;
    console.assert(e instanceof QuicknodeError);
    console.assert(e.status === 404);
    console.log(`api error ${e.status}: ${e.body.slice(0, 80)}`);
  }

  // 1b) Rate-limit override delete with a bogus override id — also a 404.
  try {
    await qn.admin.deleteRateLimitOverride(
      "does-not-exist",
      "00000000-0000-0000-0000-000000000000",
    );
  } catch (e) {
    if (!(e instanceof ApiError)) throw e;
    console.assert(e.status === 404);
    console.log(`deleteRateLimitOverride api error ${e.status}: ${e.body.slice(0, 80)}`);
  }

  // Custom headers smoke test — override User-Agent + add a correlation header.
  const headered = new QuicknodeSdk({
    apiKey: process.env.QN_SDK__API_KEY ?? "",
    http: {
      headers: {
        "User-Agent": "qn-e2e-node/1.0",
        "X-E2E-Correlation": "node-smoke",
      },
    },
  });
  try {
    const resp = await headered.admin.getEndpoints({ limit: 1 });
    console.log(`custom-headers smoke: ok (${resp.data.length} endpoints)`);
  } catch (e) {
    if (!(e instanceof QuicknodeError)) throw e;
    console.log(`custom-headers smoke error: ${e.message}`);
  }

  // 2) Timeout path — unreachable base URL + 1s timeout forces a timeout.
  const blackhole = new QuicknodeSdk({
    apiKey: process.env.QN_SDK__API_KEY ?? "",
    http: { timeoutSecs: 1 },
    admin: { baseUrl: "http://10.255.255.1/" },
  });
  try {
    await blackhole.admin.getEndpoints();
  } catch (e) {
    if (!(e instanceof TimeoutError)) throw e;
    console.log("timed out as expected");
  }
}

main();
