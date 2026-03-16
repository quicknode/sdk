import { QuickNodeSdk } from ".";

async function main() {
  const qn = new QuickNodeSdk({ apiKey: process.env["QN_API_KEY"]! });
  const endpoint_request = await qn.admin.getEndpoints();
  endpoint_request.data.map((ep) => {
    console.log(`Endpoint ${ep.id} on ${ep.network}`);
  });
}

main();
