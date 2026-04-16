import { QuickNodeSdk } from "..";

async function main() {
  const qn = QuickNodeSdk.fromEnv();
  const response = await qn.admin.getEndpoints({ limit: 20 });
  for (const ep of response.data) {
    console.log(`${ep.id} | ${ep.network}`);
  }
}

main();
