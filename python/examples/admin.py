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


async def optional_read(label, request):
    try:
        return await request
    except ApiError as error:
        if error.status != 403:
            raise
        assert isinstance(error, QuicknodeError)
        assert isinstance(error.body, str) and error.body
        print(f"{label}: access is unavailable for this account")
        return None


async def expect_api_error(request, status):
    try:
        await request
    except ApiError as error:
        assert isinstance(error, QuicknodeError)
        assert error.status == status
        assert isinstance(error.body, str) and error.body
    else:
        raise AssertionError(f"expected ApiError {status}")


async def show_observability(qn):
    for name in (
        "get_usage_credits_summary", "get_usage_rpc_summary",
        "get_usage_streams_summary", "get_usage_webhooks_summary",
        "get_traffic_summary", "get_errors_summary",
    ):
        method = getattr(qn.admin, name)
        endpoint_group = name in {"get_usage_rpc_summary", "get_traffic_summary", "get_errors_summary"}
        groups = {"group_by": ["endpoint"]} if endpoint_group else {}
        summary = await optional_read(name, method(last="1h", limit=20, **groups))
        if summary is None:
            continue
        assert summary.data.from_time and summary.data.to_time
        if endpoint_group:
            for row in summary.data.breakdown or []:
                assert hasattr(row, "endpoint")
                assert row.endpoint is None or type(row.endpoint) is int
        if summary.data.next_cursor is not None:
            next_page = await method(cursor=summary.data.next_cursor)
            assert next_page.data.from_time == summary.data.from_time
            assert next_page.data.to_time == summary.data.to_time
        print(f"{name}: summary decoded")

    filters = await optional_read("activity filters", qn.admin.list_activity_log_filters())
    if filters is not None:
        print(f"activity filter actors: {len(filters.data.actors)}")
    activity = await optional_read("activity logs", qn.admin.list_activity_logs(limit=20))
    if activity is not None and activity.data:
        detail = await qn.admin.get_activity_log(activity.data[0].id)
        assert detail.data is not None
        assert detail.data.id == activity.data[0].id

    transactions = await optional_read(
        "transactions", qn.admin.list_transactions(last="1h", limit=20),
    )
    if transactions is None:
        return
    for transaction in transactions.data.transactions:
        assert transaction.height is None or isinstance(transaction.height, str)
        assert all(endpoint_id is None or type(endpoint_id) is int for endpoint_id in transaction.endpoint_ids)
    if transactions.data.transactions:
        transaction = transactions.data.transactions[0]
        detail = await qn.admin.get_transaction(
            transaction.network, transaction.transaction_hash, include_payloads=False,
        )
        if detail.data.exists:
            assert detail.data.data is not None
            assert detail.data.data.transaction_hash == transaction.transaction_hash
            assert all(attempt.payloads is None for attempt in detail.data.data.attempts)
            if detail.data.data.endpoint_ids is not None:
                assert all(endpoint_id is None or type(endpoint_id) is int for endpoint_id in detail.data.data.endpoint_ids)
            assert all(type(attempt.endpoint_id) is int for attempt in detail.data.data.attempts)


async def main():
    qn = QuicknodeSdk.from_env()
    await show_observability(qn)

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
        endpoint_errors = await optional_read(
            "endpoint error summary",
            qn.admin.get_endpoint_errors_summary(ep_id, last="1h", limit=20),
        )
        if endpoint_errors is not None:
            assert endpoint_errors.data.from_time and endpoint_errors.data.to_time
        sec = await qn.admin.get_endpoint_security(ep_id)
        print(f"get_endpoint_security: has_data={sec.data is not None}")
        filters = await qn.admin.get_request_filters(ep_id)
        for request_filter in filters.data:
            print(f"filter {request_filter.id}: whitelist={request_filter.method_whitelist}, "
                  f"pattern keys={len(request_filter.pattern)}")
        certificates = await optional_read("client certificates", qn.admin.get_client_certs(ep_id))
        if certificates is not None:
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

        effective = await qn.admin.update_rate_limits_with_response(ep_id, rps=3, limit_by_ip=True)
        print(f"effective RPS override: {effective.data.limits.rps}")

        await qn.admin.update_rate_limits(ep_id, limit_by_ip=False)
        rl_after = await qn.admin.get_rate_limits(ep_id)
        if rl_after.data is not None:
            for row in rl_after.data.rate_limits:
                print(
                    f"get_rate_limits after PATCH: bucket={row.bucket} "
                    f"rate_limit={row.rate_limit} source={row.source} id={row.id}"
                )

    await expect_api_error(qn.admin.show_endpoint("does-not-exist"), 404)
    await expect_api_error(
        qn.admin.delete_rate_limit_override(
            "does-not-exist", "00000000-0000-0000-0000-000000000000",
        ),
        404,
    )

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
    resp = await headered.admin.get_endpoints(limit=1)
    print(f"custom-headers smoke: ok ({len(resp.data)} endpoints)")

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
    else:
        raise AssertionError("expected TimeoutError")


asyncio.run(main())
