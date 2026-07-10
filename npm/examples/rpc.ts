import { QuicknodeSdk, RpcError } from "../sdk";

async function main() {
  const qn = QuicknodeSdk.fromEnv();

  // Ensure Tooling Access is provisioned (idempotent; requires admin role).
  const status = await qn.admin.toolingAccessStatus();
  console.log(`tooling access enabled: ${status.enabled}`);
  if (!status.enabled) {
    try {
      const enabled = await qn.admin.enableToolingAccess();
      console.log(`enabled tooling access: ${enabled.enabled}`);
    } catch (e) {
      console.error(`could not enable tooling access: ${e}`);
      return;
    }
  }

  // Make a JSON-RPC call. The SDK mints and refreshes the session JWT.
  const blockNumber = await qn.rpc.call("eth_blockNumber");
  console.log(`eth_blockNumber => ${blockNumber}`);

  // Multichain: seed the per-network URL map (from the endpoint id in status),
  // then route a call to a specific network by its key.
  if (status.endpointId) {
    const urls = await qn.admin.getEndpointUrls(status.endpointId);
    if (urls.data?.multichainUrls) {
      const map = Object.fromEntries(
        Object.entries(urls.data.multichainUrls).map(([k, v]) => [k, v.httpUrl]),
      );
      qn.rpc.setNetworks(map);
      const slot = await qn.rpc.call("getSlot", [], "solana-mainnet");
      console.log(`solana getSlot => ${slot}`);
    }
  }

  // Demonstrate the typed JSON-RPC error path.
  try {
    await qn.rpc.call("eth_getBalance", ["not-an-address"]);
  } catch (e) {
    if (!(e instanceof RpcError)) throw e;
    console.log(`got expected RpcError: code=${e.code} message=${e.message}`);
  }

  // Custom endpoint URL: send a call to a fully-formed HTTP URL, bypassing the
  // Tooling Access endpoint and the session JWT entirely. Set it per-call here
  // (4th arg), or client-wide via `new RpcConfig({ endpointUrl })`.
  const customUrl = process.env.QN_RPC_ENDPOINT_URL;
  if (customUrl) {
    const result = await qn.rpc.call("eth_blockNumber", [], undefined, customUrl);
    console.log(`custom endpoint eth_blockNumber => ${result}`);
  }
}

main();
