import {
  QuicknodeSdk,
  ApiError,
  TimeoutError,
  QuicknodeError,
} from "../sdk";

async function showObservability(qn: QuicknodeSdk) {
  const summary = await qn.admin.getUsageRpcSummary({ last: "1h", groupBy: ["method"], limit: 20 });
  console.log(`RPC credits in window: ${summary.data.totals.credits}`);
  if (summary.data.nextCursor) {
    const nextPage = await qn.admin.getUsageRpcSummary({ cursor: summary.data.nextCursor });
    console.log(`next summary page: ${nextPage.data.breakdown?.length ?? 0} groups`);
  }

  const filters = await qn.admin.listActivityLogFilters();
  console.log(`activity filter actors: ${filters.data.actors.length}`);
  const activity = await qn.admin.listActivityLogs({ limit: 20 });
  console.log(`activity entries: ${activity.data.length}`);
  if (activity.data.length > 0) {
    const detail = await qn.admin.getActivityLog(activity.data[0].id);
    console.log(`activity detail present: ${detail.data != null}`);
  }

  const transactions = await qn.admin.listTransactions({ last: "1h", limit: 20 });
  for (const transaction of transactions.data.transactions) {
    console.log(`transaction state=${transaction.state}, height=${transaction.height}`);
  }
  const transaction = transactions.data.transactions[0];
  if (transaction) {
    const detail = await qn.admin.getTransaction(
      transaction.network, transaction.transactionHash, { includePayloads: false },
    );
    console.log(`retained transaction exists: ${detail.data.exists}`);
  }
}

async function main() {
  const qn = QuicknodeSdk.fromEnv();
  try {
    await showObservability(qn);
  } catch (error) {
    if (!(error instanceof ApiError) || error.status !== 403) throw error;
    console.log("Observability access is unavailable for this account.");
  }

  const account = await qn.admin.accountInfo();
  if (account.data) {
    const a = account.data;
    const plan = a.subscription?.planName ?? "<none>";
    console.log(`account ${a.id} | ${a.name} | billing=${a.billingVersion} | plan=${plan}`);
  }

  const credits = await qn.admin.getApiCredits("ethereum");
  if (credits.data) {
    console.log(`ethereum api credits: ${credits.data.length} methods`);
    for (const c of credits.data.slice(0, 3)) {
      console.log(`  ${c.method} = ${c.credits}`);
    }
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

    const filters = await qn.admin.getRequestFilters(epId);
    for (const filter of filters.data) {
      console.log(`filter ${filter.id}: whitelist=${filter.methodWhitelist}, pattern keys=${Object.keys(filter.pattern).length}`);
    }
    const certificates = await qn.admin.getClientCerts(epId);
    console.log(`client certificates: ${certificates.data.certs.length}, enabled=${certificates.data.enabled}`);
    const blacklist = await qn.admin.getMethodBlacklist(epId);
    console.log(`blocked methods: ${blacklist.data.methods.length}`);
    const ranges = await qn.admin.getRangeLimits(epId);
    console.log(`range limits: ${Object.keys(ranges.data.limits).length}, supported methods=${ranges.data.supportedMethods.length}`);
    const blocked = await qn.admin.getBlockedIps(epId);
    console.log(`blocked IP records: ${blocked.data.length}`);
    const latency = await qn.admin.getEndpointMetrics(epId, {
      period: "day", metric: "response_time_over_time", percentile: 95,
    });
    console.log(`p95 latency series: ${latency.data.length}`);

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

    const effective = await qn.admin.updateRateLimitsWithResponse(epId, { rateLimits: { rps: 3 } });
    console.log(`effective RPS override: ${effective.data.limits.rps}`);

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
