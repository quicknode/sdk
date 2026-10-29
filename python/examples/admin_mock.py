"""Offline binding checks. Run through scripts/test_admin_bindings.py."""
import asyncio
import os
from quicknode_sdk import QuicknodeSdk, SdkFullConfig, AdminConfig, ApiError

PATTERN = {"method": [{"prefix": "eth_"}], "anything-but": [None, {"nested_key": [True, 7.5]}]}


async def main():
    admin = QuicknodeSdk(SdkFullConfig(api_key="fixture-key", admin=AdminConfig(base_url=os.environ["ADMIN_MOCK_URL"]))).admin
    created = await admin.create_request_filter_pattern("12", PATTERN)
    assert created.data.pattern == PATTERN
    data = created.data
    changed = {"method": ["eth_chainId"], "anything-but": {"nested_key": None}}
    data.pattern = changed
    assert data.pattern == changed
    created.data = data
    assert created.data.pattern == changed
    await admin.update_request_filter_pattern("12", "filter", PATTERN)
    listed = await admin.get_request_filters("12")
    assert listed.data[0].pattern == PATTERN
    security = (await admin.get_endpoint_security("12")).data
    item = security.request_filters[0]
    assert item.method == PATTERN["method"]
    assert item.params["anything-but"] == PATTERN["anything-but"]
    item.method = ["eth_chainId", {"anything-but": None}]
    assert item.method == ["eth_chainId", {"anything-but": None}]
    security.request_filters = [item]
    assert security.request_filters[0].method == item.method
    assert security.options.response_logging is True
    assert security.blocked_ips[0].ip == "192.0.2.0/24"
    endpoint = (await admin.show_endpoint("12")).data
    assert endpoint.security.request_filters[0].method == PATTERN["method"]
    assert (await admin.create_token_with_response("12")).data.id == "token"
    assert (await admin.update_method_rate_limit("12", "rule", limit_by_ip=False)).data.limit_by_ip is False
    await admin.update_rate_limits("per-ip", limit_by_ip=True)
    assert (await admin.update_rate_limits_with_response("endpoint-wide", limit_by_ip=False)).data.limits.rps == 10
    assert (await admin.update_rate_limits_with_response("combined", rps=10, limit_by_ip=False)).data.limits.rps == 10
    await admin.update_rate_limits("limits-only", rps=10)
    try:
        await admin.update_rate_limits("empty")
    except ApiError as error:
        assert error.status == 400 and "at least one property" in error.body
    else:
        raise AssertionError("expected ApiError for empty update")
    summary = (await admin.get_usage_rpc_summary(last="1d", group_by=["method", "endpoint"])).data
    assert summary.totals.rpc_credits == 7.5
    assert summary.breakdown[0].requests is None
    assert summary.breakdown[0].method is None
    assert summary.from_time == "2026-10-01T00:00:00Z"
    page = (await admin.get_usage_rpc_summary(cursor=summary.next_cursor)).data
    assert page.totals.rpc_credits == 7.5 and page.next_cursor is None
    traffic = (await admin.get_traffic_summary(last="1d", group_by=["method", "endpoint"])).data
    assert traffic.breakdown[0].successful_requests is None
    assert traffic.breakdown[0].method is None
    errors = (await admin.get_errors_summary(last="1d", group_by=["endpoint"], endpoints=[4123, 4124])).data
    for response in (summary, traffic, errors):
        assert response.breakdown[0].endpoint == 4123
        assert response.breakdown[1].endpoint is None
        assert not hasattr(response.breakdown[2], "endpoint")
    rows = (await admin.list_transactions(last="1d", endpoints=[4123, 4124])).data
    assert rows.transactions[0].height == "9007199254740993"
    assert rows.transactions[0].endpoint_ids == [4123, None, 4124]
    absent = (await admin.get_transaction("ethereum-mainnet", "hash", include_payloads=False)).data
    assert absent.exists is False and absent.data is None
    detail = (await admin.get_transaction("ethereum-mainnet", "hash", include_payloads=True)).data.data
    assert detail.height == "9007199254740993"
    assert detail.endpoint_ids == [4123, None, 4124]
    assert detail.attempts[0].endpoint_id == 4123
    assert detail.attempts[0].payloads.request == ' { "method" : "eth_sendRawTransaction" }\n'
    assert detail.attempts[0].payloads.response == "plain text"
    try:
        await admin.get_endpoint_security("missing")
    except ApiError as error:
        assert error.status == 404 and "fixture missing endpoint" in error.body
    else:
        raise AssertionError("expected ApiError")


asyncio.run(main())
