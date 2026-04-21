import {
  QuickNodeSdk,
  ApiError,
  TimeoutError,
  QuickNodeError,
} from "..";

async function main() {
  const qn = QuickNodeSdk.fromEnv();

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
        `dedicated=${ep.isDedicated} flat=${ep.isFlatRate}`,
    );
  }

  const tags = await qn.admin.listTags();
  if (tags.data) {
    console.log(`account tags: ${tags.data.tags.length}`);
  }

  if (response.data.length > 0) {
    const sec = await qn.admin.getEndpointSecurity(response.data[0].id);
    console.log(`getEndpointSecurity: has_data=${sec.data !== undefined && sec.data !== null}`);
  }

  // ── Error handling ──────────────────────────────────────────────────
  // 1) API error path — 404 on a bogus endpoint id.
  try {
    await qn.admin.showEndpoint("does-not-exist");
  } catch (e) {
    if (!(e instanceof ApiError)) throw e;
    console.assert(e instanceof QuickNodeError);
    console.assert(e.status === 404);
    console.log(`api error ${e.status}: ${e.body.slice(0, 80)}`);
  }

  // 2) Timeout path — unreachable base URL + 1s timeout forces a timeout.
  const blackhole = new QuickNodeSdk({
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
