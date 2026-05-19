import asyncio
import os
from sdk import (
    QuicknodeSdk,
    SdkFullConfig,
    HttpConfig,
    AdminConfig,
    ApiError,
    TimeoutError,
    QuicknodeError,
)


async def main():
    qn = QuicknodeSdk.from_env()

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

    metrics = await qn.admin.get_account_metrics(
        period="day", metric="credits_over_time"
    )
    first_tag = ":".join(metrics.data[0].tag) if metrics.data else "<none>"
    print(f"get_account_metrics: {len(metrics.data)} series, first tag: {first_tag}")

    if response.data:
        sec = await qn.admin.get_endpoint_security(response.data[0].id)
        print(f"get_endpoint_security: has_data={sec.data is not None}")

    # ── Error handling ──────────────────────────────────────────────────
    # 1) API error path — 404 on a bogus endpoint id.
    try:
        await qn.admin.show_endpoint("does-not-exist")
    except ApiError as e:
        assert isinstance(e, QuicknodeError)
        assert e.status == 404
        print(f"api error {e.status}: {e.body[:80]}")

    # 2) Timeout path — unreachable base URL + 1s timeout forces a timeout.
    blackhole = QuicknodeSdk(
        SdkFullConfig(
            api_key=os.environ["QN_SDK__API_KEY"],
            http=HttpConfig(timeout_secs=1),
            admin=AdminConfig(base_url="http://10.255.255.1/"),
        )
    )
    try:
        await blackhole.admin.get_endpoints()
    except TimeoutError:
        print("timed out as expected")


asyncio.run(main())
