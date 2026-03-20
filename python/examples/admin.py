import asyncio
from sdk import QuickNodeSdk


async def main():
    qn = QuickNodeSdk.from_env()
    response = await qn.admin.get_endpoints(limit=20)
    for ep in response.data:
        print(f"{ep.id} | {ep.network}")


asyncio.run(main())
