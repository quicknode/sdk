import { QuickNodeSdk } from "..";

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
}

main();
