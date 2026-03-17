import { QuickNodeSdk } from ".";

async function main() {
  const qn = new QuickNodeSdk({ apiKey: process.env["QN_API_KEY"] || "" });
  const endpoint_request = await qn.admin.getEndpoints({ limit: 5 });
  endpoint_request.data.map((ep) => {
    console.log(`Endpoint ${ep.id} on ${ep.network}`);
    qn.admin.showEndpoint(ep.id);
  });
}

main();
