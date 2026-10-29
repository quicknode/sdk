import {
  QuicknodeSdk,
  ApiError,
  TimeoutError,
  QuicknodeError,
} from "../sdk";

async function optionalRead<T>(label: string, request: Promise<T>): Promise<T | undefined> {
  try {
    return await request;
  } catch (error) {
    if (!(error instanceof ApiError) || error.status !== 403) throw error;
    if (!(error instanceof QuicknodeError) || !error.body) {
      throw new Error(`${label}: expected typed API error with a body`);
    }
    console.log(`${label}: access is unavailable for this account`);
    return undefined;
  }
}

async function expectApiError(request: Promise<unknown>, status: number) {
  try {
    await request;
  } catch (error) {
    if (!(error instanceof ApiError)) throw error;
    if (!(error instanceof QuicknodeError) || error.status !== status || !error.body) {
      throw new Error(`expected ApiError ${status} with a body`);
    }
    return;
  }
  throw new Error(`expected ApiError ${status}`);
}

async function showObservability(qn: QuicknodeSdk) {
  const summaries = [
    qn.admin.getUsageCreditsSummary.bind(qn.admin),
    qn.admin.getUsageRpcSummary.bind(qn.admin),
    qn.admin.getUsageStreamsSummary.bind(qn.admin),
    qn.admin.getUsageWebhooksSummary.bind(qn.admin),
    qn.admin.getTrafficSummary.bind(qn.admin),
    qn.admin.getErrorsSummary.bind(qn.admin),
  ];
  for (const method of summaries) {
    const summary = await optionalRead<Awaited<ReturnType<typeof method>>>("summary", method({ last: "1h", limit: 20 }));
    if (!summary) continue;
    if (!summary.data.from || !summary.data.to) throw new Error("missing summary window");
    if (summary.data.nextCursor) {
      const nextPage = await method({ cursor: summary.data.nextCursor });
      if (nextPage.data.from !== summary.data.from || nextPage.data.to !== summary.data.to) {
        throw new Error("summary cursor changed its window");
      }
    }
    console.log("summary decoded");
  }

  const filters = await optionalRead("activity filters", qn.admin.listActivityLogFilters());
  if (filters) console.log(`activity filter actors: ${filters.data.actors.length}`);
  const activity = await optionalRead("activity logs", qn.admin.listActivityLogs({ limit: 20 }));
  if (activity && activity.data.length > 0) {
    const detail = await qn.admin.getActivityLog(activity.data[0].id);
    if (detail.data?.id !== activity.data[0].id) throw new Error("activity detail ID mismatch");
  }

  const transactions = await optionalRead("transactions", qn.admin.listTransactions({ last: "1h", limit: 20 }));
  if (!transactions) return;
  for (const transaction of transactions.data.transactions) {
    if (transaction.height != null && typeof transaction.height !== "string") {
      throw new Error("transaction height must remain a string");
    }
  }
  const transaction = transactions.data.transactions[0];
  if (transaction) {
    const detail = await qn.admin.getTransaction(
      transaction.network, transaction.transactionHash, { includePayloads: false },
    );
    if (detail.data.exists) {
      if (detail.data.data?.transactionHash !== transaction.transactionHash) {
        throw new Error("transaction detail hash mismatch");
      }
      if (detail.data.data.attempts.some(attempt => attempt.payloads != null)) {
        throw new Error("transaction payloads were not requested");
      }
    }
  }
}

async function main() {
  const qn = QuicknodeSdk.fromEnv();
  await showObservability(qn);

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
    const endpointErrors = await optionalRead(
      "endpoint error summary", qn.admin.getEndpointErrorsSummary(epId, { last: "1h", limit: 20 }),
    );
    if (endpointErrors && (!endpointErrors.data.from || !endpointErrors.data.to)) {
      throw new Error("missing endpoint error summary window");
    }
    const sec = await qn.admin.getEndpointSecurity(epId);
    console.log(`getEndpointSecurity: has_data=${sec.data !== undefined && sec.data !== null}`);

    const filters = await qn.admin.getRequestFilters(epId);
    for (const filter of filters.data) {
      console.log(`filter ${filter.id}: whitelist=${filter.methodWhitelist}, pattern keys=${Object.keys(filter.pattern).length}`);
    }
    const certificates = await optionalRead("client certificates", qn.admin.getClientCerts(epId));
    if (certificates) {
      console.log(`client certificates: ${certificates.data.certs.length}, enabled=${certificates.data.enabled}`);
    }
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

    const effective = await qn.admin.updateRateLimitsWithResponse(epId, { rateLimits: { rps: 3 }, limitByIp: true });
    console.log(`effective RPS override: ${effective.data.limits.rps}`);

    await qn.admin.updateRateLimits(epId, { limitByIp: false });
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

  await expectApiError(qn.admin.showEndpoint("does-not-exist"), 404);
  await expectApiError(
    qn.admin.deleteRateLimitOverride("does-not-exist", "00000000-0000-0000-0000-000000000000"),
    404,
  );

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
  const resp = await headered.admin.getEndpoints({ limit: 1 });
  console.log(`custom-headers smoke: ok (${resp.data.length} endpoints)`);

  // 2) Timeout path — unreachable base URL + 1s timeout forces a timeout.
  const blackhole = new QuicknodeSdk({
    apiKey: process.env.QN_SDK__API_KEY ?? "",
    http: { timeoutSecs: 1 },
    admin: { baseUrl: "http://10.255.255.1/" },
  });
  try {
    await blackhole.admin.getEndpoints();
    throw new Error("expected TimeoutError");
  } catch (e) {
    if (!(e instanceof TimeoutError)) throw e;
    console.log("timed out as expected");
  }
}

main();
