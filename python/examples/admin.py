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


async def show_observability(qn):
    summary = await qn.admin.get_usage_rpc_summary(last="1h", group_by=["method"], limit=20)
    print(f"RPC credits in window: {summary.data.totals.credits}")
    if summary.data.next_cursor is not None:
        next_page = await qn.admin.get_usage_rpc_summary(cursor=summary.data.next_cursor)
        print(f"next summary page: {len(next_page.data.breakdown or [])} groups")

    filters = await qn.admin.list_activity_log_filters()
    print(f"activity filter actors: {len(filters.data.actors)}")
    activity = await qn.admin.list_activity_logs(limit=20)
    print(f"activity entries: {len(activity.data)}")
    if activity.data:
        detail = await qn.admin.get_activity_log(activity.data[0].id)
        print(f"activity detail present: {detail.data is not None}")

    transactions = await qn.admin.list_transactions(last="1h", limit=20)
    for transaction in transactions.data.transactions:
        print(f"transaction state={transaction.state}, height={transaction.height}")
    if transactions.data.transactions:
        transaction = transactions.data.transactions[0]
        detail = await qn.admin.get_transaction(
            transaction.network, transaction.transaction_hash, include_payloads=False,
        )
        print(f"retained transaction exists: {detail.data.exists}")


async def main():
    qn = QuicknodeSdk.from_env()
    try:
        await show_observability(qn)
    except ApiError as error:
        if error.status != 403:
            raise
        print("Observability access is unavailable for this account.")

    account = await qn.admin.account_info()
    if account.data is not None:
        a = account.data
        plan = a.subscription.plan_name if a.subscription is not None else "<none>"
        print(f"account {a.id} | {a.name} | billing={a.billing_version} | plan={plan}")

    credits = await qn.admin.get_api_credits("ethereum")
    if credits.data is not None:
        print(f"ethereum api credits: {len(credits.data)} methods")
        for c in credits.data[:3]:
            print(f"  {c.method} = {c.credits}")

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
        filters = await qn.admin.get_request_filters(ep_id)
        for request_filter in filters.data:
            print(f"filter {request_filter.id}: whitelist={request_filter.method_whitelist}, "
                  f"pattern keys={len(request_filter.pattern)}")
        certificates = await qn.admin.get_client_certs(ep_id)
        print(f"client certificates: {len(certificates.data.certs)}, enabled={certificates.data.enabled}")
        blacklist = await qn.admin.get_method_blacklist(ep_id)
        print(f"blocked methods: {len(blacklist.data.methods)}")
        ranges = await qn.admin.get_range_limits(ep_id)
        print(f"range limits: {len(ranges.data.limits)}, supported methods={len(ranges.data.supported_methods)}")
        blocked = await qn.admin.get_blocked_ips(ep_id)
        print(f"blocked IP records: {len(blocked.data)}")
        latency = await qn.admin.get_endpoint_metrics(
            ep_id, period="day", metric="response_time_over_time", percentile=95,
        )
        print(f"p95 latency series: {len(latency.data)}")

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

        effective = await qn.admin.update_rate_limits_with_response(ep_id, rps=3)
        print(f"effective RPS override: {effective.data.limits.rps}")

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
