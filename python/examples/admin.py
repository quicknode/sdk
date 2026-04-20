import asyncio
from sdk import QuickNodeSdk


async def main():
    qn = QuickNodeSdk.from_env()

    response = await qn.admin.get_endpoints(
        limit=20,
        sort_by="created_at",
        sort_direction="desc",
    )
    if response.pagination is not None:
        p = response.pagination
        print(f"{len(response.data)} of {p.total} (offset {p.offset}, limit {p.limit})")
    for ep in response.data:
        print(
            f"{ep.id} | {ep.name} | {ep.status} | {ep.network} | "
            f"dedicated={ep.is_dedicated} flat={ep.is_flat_rate}"
        )

    tags = await qn.admin.list_tags()
    if tags.data is not None:
        print(f"account tags: {len(tags.data.tags)}")

    if response.data:
        sec = await qn.admin.get_endpoint_security(response.data[0].id)
        print(f"get_endpoint_security: has_data={sec.data is not None}")


asyncio.run(main())
