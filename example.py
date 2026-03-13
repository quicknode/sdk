import asyncio
import os
from sdk import QuickNodeSdk


async def main():
    api_key = os.environ.get("QN_API_KEY") or ""
    qn_sdk = QuickNodeSdk(api_key=api_key)
    response = await qn_sdk.admin.get_endpoints(limit=20)
    for endpoint in response.data:
        name = endpoint.id
        print(f"Endpoint {name} on {endpoint.network}")


asyncio.run(main())
