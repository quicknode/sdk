import asyncio
from my_sdk import add, get_external_uuid


async def main():
    add(1, 2)
    uuid = await get_external_uuid()
    print(uuid)


asyncio.run(main())
