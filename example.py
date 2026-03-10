import asyncio
from sdk import init


async def main():
    qn_sdk = init(api_key="test123")
    uuid = await qn_sdk.httpbin.get_uuid()
    print(uuid)


asyncio.run(main())
