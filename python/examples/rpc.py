import asyncio
import os
from quicknode_sdk import QuicknodeSdk, RpcError


async def main():
    qn = QuicknodeSdk.from_env()

    # Ensure Tooling Access is provisioned (idempotent; requires admin role).
    status = await qn.admin.tooling_access_status()
    print(f"tooling access enabled: {status.enabled}")
    if not status.enabled:
        try:
            enabled = await qn.admin.enable_tooling_access()
            print(f"enabled tooling access: {enabled.enabled}")
        except Exception as e:
            print(f"could not enable tooling access: {e}")
            return

    # Make a JSON-RPC call. The SDK mints and refreshes the session JWT.
    block_number = await qn.rpc.call("eth_blockNumber")
    print(f"eth_blockNumber => {block_number}")

    # Multichain: seed the per-network URL map (from the endpoint id in status),
    # then route a call to a specific network by its key.
    if status.endpoint_id:
        urls = await qn.admin.get_endpoint_urls(status.endpoint_id)
        if urls.data and urls.data.multichain_urls:
            qn.rpc.set_networks(
                {k: v.http_url for k, v in urls.data.multichain_urls.items()}
            )
            slot = await qn.rpc.call("getSlot", network="solana-mainnet")
            print(f"solana getSlot => {slot}")

    # Demonstrate the typed JSON-RPC error path.
    try:
        await qn.rpc.call("eth_getBalance", ["not-an-address"])
    except RpcError as e:
        print(f"got expected RpcError: code={e.code} message={e.message}")

    # Custom endpoint URL: send a call to a fully-formed HTTP URL, bypassing the
    # Tooling Access endpoint and the session JWT entirely. Set it per-call here,
    # or client-wide via RpcConfig(endpoint_url=...).
    custom_url = os.environ.get("QN_RPC_ENDPOINT_URL")
    if custom_url:
        result = await qn.rpc.call("eth_blockNumber", endpoint_url=custom_url)
        print(f"custom endpoint eth_blockNumber => {result}")


asyncio.run(main())
