import asyncio
import os
from quicknode_sdk import (
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
            f"dedicated={ep.is_dedicated} flat={ep.is_flat_rate} multichain={ep.is_multichain}"
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
        ep_id = response.data[0].id
        sec = await qn.admin.get_endpoint_security(ep_id)
        print(f"get_endpoint_security: has_data={sec.data is not None}")

        urls = await qn.admin.get_endpoint_urls(ep_id)
        if urls.data is not None:
            mc = urls.data.multichain_urls
            print(
                f"get_endpoint_urls: http={urls.data.http_url} "
                f"multichain_networks={list(mc.keys()) if mc is not None else None}"
            )

        rl_before = await qn.admin.get_rate_limits(ep_id)
        if rl_before.data is not None:
            for row in rl_before.data.rate_limits:
                print(
                    f"get_rate_limits before PATCH: bucket={row.bucket} "
                    f"rate_limit={row.rate_limit} source={row.source} id={row.id}"
                )

        await qn.admin.update_rate_limits(ep_id, rps=3)
        print("update_rate_limits: ok")

        rl_after = await qn.admin.get_rate_limits(ep_id)
        if rl_after.data is not None:
            for row in rl_after.data.rate_limits:
                print(
                    f"get_rate_limits after PATCH: bucket={row.bucket} "
                    f"rate_limit={row.rate_limit} source={row.source} id={row.id}"
                )

    # ── Error handling ──────────────────────────────────────────────────
    # 1) API error path — 404 on a bogus endpoint id.
    try:
        await qn.admin.show_endpoint("does-not-exist")
    except ApiError as e:
        assert isinstance(e, QuicknodeError)
        assert e.status == 404
        print(f"api error {e.status}: {e.body[:80]}")

    # 1b) Rate-limit override delete with a bogus override id — also a 404.
    try:
        await qn.admin.delete_rate_limit_override(
            "does-not-exist", "00000000-0000-0000-0000-000000000000"
        )
    except ApiError as e:
        assert e.status == 404
        print(f"delete_rate_limit_override api error {e.status}: {e.body[:80]}")

    # Custom headers smoke test — override User-Agent + add a correlation header.
    headered = QuicknodeSdk(
        SdkFullConfig(
            api_key=os.environ["QN_SDK__API_KEY"],
            http=HttpConfig(
                headers={
                    "User-Agent": "qn-e2e-python/1.0",
                    "X-E2E-Correlation": "python-smoke",
                }
            ),
        )
    )
    try:
        resp = await headered.admin.get_endpoints(limit=1)
        print(f"custom-headers smoke: ok ({len(resp.data)} endpoints)")
    except QuicknodeError as e:
        print(f"custom-headers smoke error: {e}")

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
