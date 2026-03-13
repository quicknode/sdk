import asyncio
from sdk import QuickNodeSdk, GetEndpointsRequest


async def main():
    qn_sdk = QuickNodeSdk(api_key="test123")
    response = await qn_sdk.admin.get_endpoints(limit=20)
    print(response)


asyncio.run(main())
