import asyncio
import os
from sdk import QuickNodeSdk
from sdk._core import SdkFullConfig


async def main():
    api_key = os.environ.get("QN_API_KEY") or ""
    qn_sdk = QuickNodeSdk(SdkFullConfig(api_key=api_key))
    response = await qn_sdk.admin.get_endpoints(limit=20)
    for endpoint in response.data:
        name = endpoint.id
        print(f"Endpoint {name} on {endpoint.network}")


asyncio.run(main())
