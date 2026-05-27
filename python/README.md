# quicknode-sdk (Python)

Python bindings for the Quicknode SDK.

This is one of four language bindings published from the same Rust core. See the [project README](https://github.com/quiknode-labs/sdk/blob/main/README.md) for the polyglot overview, development setup, and release process.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Platform Support](#platform-support)
- [API Reference](#api-reference)
  - [Admin Client](#admin-client)
    - [Endpoints](#endpoints)
    - [Endpoint Tags](#endpoint-tags)
    - [Teams](#teams)
    - [Usage](#usage)
    - [Logs](#logs)
    - [Endpoint Security](#endpoint-security)
    - [Security Options](#security-options)
    - [Tokens](#tokens)
    - [Referrers](#referrers)
    - [IPs](#ips)
    - [Domain Masks](#domain-masks)
    - [JWTs](#jwts)
    - [Request Filters](#request-filters)
    - [Multichain](#multichain)
    - [IP Custom Headers](#ip-custom-headers)
    - [Method Rate Limits](#method-rate-limits)
    - [Endpoint Rate Limits](#endpoint-rate-limits)
    - [Endpoint URLs](#endpoint-urls)
    - [Metrics](#metrics)
    - [Chains](#chains)
    - [Billing](#billing)
    - [Bulk Operations](#bulk-operations)
    - [Account Tags](#account-tags)
  - [Streams Client](#streams-client)
    - [Datasets, Regions, and Destinations](#datasets-regions-and-destinations)
    - [Streams methods](#streams-methods)
  - [Webhooks Client](#webhooks-client)
    - [Templates and destination](#templates-and-destination)
    - [Webhooks methods](#webhooks-methods)
  - [KV Store Client](#kv-store-client)
    - [Sets](#sets)
    - [Lists](#lists)
- [Error Handling](#error-handling)
- [License](#license)

## Installation

`uv add quicknode-sdk`

## Quick Start

Construct the SDK once, then reach into the four sub-clients (`admin`, `streams`, `webhooks`, `kvstore`). Subsequent API Reference snippets assume you have a `qn` handle from one of these blocks.

```python
# Python
import asyncio
from sdk import QuicknodeSdk

async def main():
    qn = QuicknodeSdk.from_env()
    resp = await qn.admin.get_endpoints()
    print(f"{len(resp.data)} endpoints")

asyncio.run(main())
```

## Configuration

There are two ways to configure the SDK.

### Option A — Pass config directly

```python
# Python
from sdk import QuicknodeSdk, SdkFullConfig, HttpConfig
qn = QuicknodeSdk(SdkFullConfig(api_key="your-key", http=HttpConfig(timeout_secs=30)))
```

### Option B — Load from environment (`from_env()`)

```python
# Python
qn = QuicknodeSdk.from_env()
```

Environment variables (prefix `QN_SDK__`, separator `__`):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `QN_SDK__API_KEY` | yes | — | Your Quicknode API key |
| `QN_SDK__HTTP__TIMEOUT_SECS` | no | 30 | HTTP request timeout in seconds |
| `QN_SDK__HTTP__POOL_MAX_IDLE_PER_HOST` | no | — | Max idle HTTP connections per host |
| `QN_SDK__ADMIN__BASE_URL` | no | `https://api.quicknode.com/v0/` | Override admin API base URL (HTTPS, must end with `/`) |
| `QN_SDK__STREAMS__BASE_URL` | no | `https://api.quicknode.com/streams/rest/v1/` | Override streams base URL |
| `QN_SDK__WEBHOOKS__BASE_URL` | no | `https://api.quicknode.com/webhooks/rest/v1/` | Override webhooks base URL |
| `QN_SDK__KVSTORE__BASE_URL` | no | `https://api.quicknode.com/kv/rest/v1/` | Override KV store base URL |
| `QN_SDK__HTTP__HEADERS__<NAME>` | no | — | Custom HTTP header sent on every request. Overrides SDK-managed headers (see below). |

### Custom headers and `User-Agent`

Every outbound HTTP request includes an auto-generated `User-Agent` of the form:

```
quicknode-sdk-<language>/<sdk-version> (<os>-<arch>; <language>-<runtime-version>)
```

You can attach arbitrary headers via `HttpConfig.headers`. **These headers OVERRIDE any SDK-managed header with the same name**, including `User-Agent`, `x-api-key`, `Accept`, and `Content-Type`. Use this to inject correlation IDs, proxy auth, or to replace the default `User-Agent`. Header names are matched case-insensitively.

```python
from sdk import QuicknodeSdk, SdkFullConfig, HttpConfig

qn = QuicknodeSdk(
    SdkFullConfig(
        api_key="your-key",
        http=HttpConfig(headers={
            "X-Correlation-Id": "abc-123",
            "User-Agent": "my-app/1.0",  # overrides SDK default
        }),
    )
)
```

## Platform Support

Precompiled wheels are published for:

| Platform | Targets |
|---|---|
| Linux (glibc) | `x86_64`, `aarch64` — glibc **2.17+** (manylinux2014) |
| Linux (musl) | `x86_64`, `aarch64` — Alpine and other musl distros |
| macOS | Apple Silicon (`arm64`) |

Linux glibc wheels are built against glibc 2.17 so they load on any distro from 2014 onward — RHEL 7+, Ubuntu 14.04+, Debian 8+, Amazon Linux 2+, SLES 12+, Fedora 19+. If `pip install quicknode-sdk` resolves to a source distribution on your platform, you're on something we don't have a prebuilt wheel for — see the matrix above.

**Not supported:** RHEL/CentOS 6 (glibc 2.12), Debian 7 (glibc 2.13), Ubuntu 12.04 (glibc 2.15), SLES 11 (glibc 2.11), Intel macOS, Windows.

## API Reference

Snippets assume `qn` was already constructed via the Quick Start. Optional parameters are skipped unless showing one is needed to illustrate usage.

### Language conventions

- Methods are `async` — call with `await`. Parameters are kwargs; responses are native `pyclass` objects with attribute access.

---

### Admin Client

Accessed as `qn.admin`. Manages endpoints, tags, teams, billing, usage, metrics, security, and rate limits. Backed by `https://api.quicknode.com/v0/`.

#### Endpoints

##### `get_endpoints` / `getEndpoints`

Returns a paginated list of endpoints on the account with optional search, filters (networks, statuses, labels, tags, dedicated, flat-rate), sorting, and pagination.

**Parameters** (all optional): `limit` (i32), `offset` (i32), `search` (string), `sort_by` (string), `sort_direction` (`"asc"` | `"desc"`), `networks` (string[]), `statuses` (string[]), `labels` (string[]), `dedicated` (bool), `is_flat_rate` (bool), `tag_ids` (i32[]), `tag_labels` (string[]).

**Returns**: `GetEndpointsResponse` — `{ data: Endpoint[], pagination?: Pagination }`.

```python
# Python
resp = await qn.admin.get_endpoints(limit=20, sort_by="created_at", sort_direction="desc")
```

##### `create_endpoint` / `createEndpoint`

Creates a new endpoint for the given blockchain and network.

**Parameters**: `chain` (string, optional), `network` (string, optional).

**Returns**: `CreateEndpointResponse` with `data: SingleEndpoint`.

```python
# Python
resp = await qn.admin.create_endpoint(chain="ethereum", network="mainnet")
```

##### `show_endpoint` / `showEndpoint`

Fetches a single endpoint by id, including its full security configuration and rate limits.

**Parameters**: `id` (string, required).

**Returns**: `ShowEndpointResponse` with `data: SingleEndpoint`.

```python
# Python
resp = await qn.admin.show_endpoint("ep-123")
```

##### `update_endpoint` / `updateEndpoint`

Updates editable fields on an endpoint. Currently supports `label`.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```python
# Python
await qn.admin.update_endpoint("ep-123", label="my label")
```

##### `archive_endpoint` / `archiveEndpoint`

Archives an endpoint. The HTTP verb is `DELETE` but the effect is archival, not permanent deletion.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.archive_endpoint("ep-123")
```

##### `update_endpoint_status` / `updateEndpointStatus`

Pauses or unpauses an endpoint.

**Parameters**: `id` (string, required); body: `status` (string, required — `"active"` or `"paused"`).

**Returns**: `UpdateEndpointStatusResponse`.

```python
# Python
await qn.admin.update_endpoint_status("ep-123", status="paused")
```

#### Endpoint Tags

Per-endpoint tag add/remove. For account-wide tag management see [Account Tags](#account-tags).

##### `create_tag` / `createTag`

Tags an endpoint with the given label. Creates the tag on the account if it does not exist.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```python
# Python
await qn.admin.create_tag("ep-123", label="prod")
```

##### `delete_tag` / `deleteTag`

Removes a tag from a specific endpoint.

**Parameters**: `id` (endpoint id, string, required), `tag_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_tag("ep-123", "42")
```

#### Teams

##### `list_teams` / `listTeams`

Lists all teams on the account.

**Parameters**: none.

**Returns**: `ListTeamsResponse` with `data: TeamSummary[]`.

```python
# Python
resp = await qn.admin.list_teams()
```

##### `create_team` / `createTeam`

Creates a new team.

**Parameters**: `name` (string, required).

**Returns**: `CreateTeamResponse` with `data: CreateTeamData`.

```python
# Python
resp = await qn.admin.create_team(name="Payments")
```

##### `get_team` / `getTeam`

Fetches team detail including pending invites.

**Parameters**: `id` (i64, required).

**Returns**: `GetTeamResponse` with `data: TeamDetail`.

```python
# Python
resp = await qn.admin.get_team(42)
```

##### `delete_team` / `deleteTeam`

Deletes a team.

**Parameters**: `id` (i64, required).

**Returns**: `DeleteTeamResponse`.

```python
# Python
await qn.admin.delete_team(42)
```

##### `list_team_endpoints` / `listTeamEndpoints`

Lists endpoints accessible to a team.

**Parameters**: `id` (i64, required).

**Returns**: `ListTeamEndpointsResponse` with `data: TeamEndpoint[]`.

```python
# Python
resp = await qn.admin.list_team_endpoints(42)
```

##### `update_team_endpoints` / `updateTeamEndpoints`

Replaces the set of endpoints associated with a team. Pass an empty array to remove all.

**Parameters**: `id` (i64, required); body: `endpoint_ids` (string[], required).

**Returns**: `UpdateTeamEndpointsResponse`.

```python
# Python
await qn.admin.update_team_endpoints(42, endpoint_ids=["ep-123", "ep-456"])
```

##### `invite_team_member` / `inviteTeamMember`

Invites a user to a team. Existing users only need `email`; new users require `full_name` and `role`.

**Parameters**: `id` (i64, required); body: `email` (string, required), `full_name` (string, optional), `role` (string, optional — `admin` | `viewer` | `billing`).

**Returns**: `InviteTeamMemberResponse`.

```python
# Python
await qn.admin.invite_team_member(42, email="alice@example.com", role="viewer")
```

##### `remove_team_member` / `removeTeamMember`

Removes a user from a team.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `RemoveTeamMemberResponse`.

```python
# Python
await qn.admin.remove_team_member(42, 7)
```

##### `resend_team_invite` / `resendTeamInvite`

Re-sends a pending team invitation.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `ResendTeamInviteResponse`.

```python
# Python
await qn.admin.resend_team_invite(42, 7)
```

#### Usage

All usage methods accept optional `start_time` and `end_time` Unix timestamps. Omit both for account-to-date totals.

##### `get_usage` / `getUsage`

Aggregate account usage for a time window.

**Returns**: `GetUsageResponse` with `data: UsageData` (`credits_used`, `credits_remaining`, `limit`, `overages`, `start_time`, `end_time`).

```python
# Python
resp = await qn.admin.get_usage()
```

##### `get_usage_by_endpoint` / `getUsageByEndpoint`

Per-endpoint usage breakdown.

**Returns**: `GetUsageByEndpointResponse` with `data.endpoints: EndpointUsage[]`.

```python
# Python
resp = await qn.admin.get_usage_by_endpoint()
```

##### `get_usage_by_method` / `getUsageByMethod`

Per-RPC-method usage breakdown.

**Returns**: `GetUsageByMethodResponse` with `data.methods: MethodUsage[]`.

```python
# Python
resp = await qn.admin.get_usage_by_method()
```

##### `get_usage_by_chain` / `getUsageByChain`

Per-chain usage breakdown.

**Returns**: `GetUsageByChainResponse` with `data.chains: ChainUsage[]`.

```python
# Python
resp = await qn.admin.get_usage_by_chain()
```

##### `get_usage_by_tag` / `getUsageByTag`

Per-tag usage breakdown.

**Returns**: `GetUsageByTagResponse` with `data.tags: TagUsage[]`.

```python
# Python
resp = await qn.admin.get_usage_by_tag()
```

#### Logs

##### `get_endpoint_logs` / `getEndpointLogs`

Fetches a page of request logs for an endpoint. Set `include_details=true` for full request/response payloads (truncated at 2 KB each).

**Parameters**: `id` (endpoint id, required); body: `from` (string timestamp, required), `to` (string timestamp, required), `include_details` (bool, optional), `limit` (i32, optional), `next_at` (string cursor, optional).

**Returns**: `GetEndpointLogsResponse` — `{ data: EndpointLog[], next_at?: string }`.

```python
# Python
resp = await qn.admin.get_endpoint_logs(
    "ep-123",
    from_time="2026-04-01T00:00:00Z",
    to_time="2026-04-02T00:00:00Z",
    limit=100,
)
```

##### `get_log_details` / `getLogDetails`

Returns the full request/response payloads for a single log entry.

**Parameters**: `id` (endpoint id, required), `request_id` (log request uuid, required).

**Returns**: `GetLogDetailsResponse` with `data: LogDetails`.

```python
# Python
resp = await qn.admin.get_log_details("ep-123", "req-abc")
```

#### Endpoint Security

##### `get_endpoint_security` / `getEndpointSecurity`

Returns the full security configuration for an endpoint: tokens, JWTs, referrers, domain masks, IPs, request filters, and their per-feature toggles.

**Parameters**: `id` (string, required).

**Returns**: `GetEndpointSecurityResponse` with `data: EndpointSecurity`.

```python
# Python
resp = await qn.admin.get_endpoint_security("ep-123")
```

#### Security Options

##### `get_security_options` / `getSecurityOptions`

Returns the list of security features and their enabled state for an endpoint.

**Parameters**: `id` (string, required).

**Returns**: `GetSecurityOptionsResponse` with `data: SecurityOption[]`.

```python
# Python
resp = await qn.admin.get_security_options("ep-123")
```

##### `update_security_options` / `updateSecurityOptions`

Enables or disables individual security features. Each field accepts `"enabled"` or `"disabled"`.

**Parameters**: `id` (string, required); `options`: `SecurityOptionsUpdate` (`tokens`, `referrers`, `jwts`, `ips`, `domain_masks`, `hsts`, `cors`, `request_filters`, `ip_custom_header`).

**Returns**: `UpdateSecurityOptionsResponse` with updated `SecurityOption[]`.

```python
# Python
await qn.admin.update_security_options(
    "ep-123",
    tokens="enabled",
    jwts="disabled",
)
```

#### Tokens

##### `create_token` / `createToken`

Generates a new auth token on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```python
# Python
await qn.admin.create_token("ep-123")
```

##### `delete_token` / `deleteToken`

Revokes a token on an endpoint.

**Parameters**: `id` (endpoint id, required), `token_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_token("ep-123", "tok-1")
```

#### Referrers

##### `create_referrer` / `createReferrer`

Whitelists a referrer URL or domain on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `referrer` (string, optional).

**Returns**: nothing.

```python
# Python
await qn.admin.create_referrer("ep-123", referrer="example.com")
```

##### `delete_referrer` / `deleteReferrer`

Removes a referrer from the whitelist.

**Parameters**: `id` (endpoint id, required), `referrer_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_referrer("ep-123", "ref-1")
```

#### IPs

##### `create_ip` / `createIp`

Whitelists an IP address on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `ip` (string, optional).

**Returns**: nothing.

```python
# Python
await qn.admin.create_ip("ep-123", ip="198.51.100.7")
```

##### `delete_ip` / `deleteIp`

Removes an IP from the whitelist.

**Parameters**: `id` (endpoint id, required), `ip_id` (string, required).

**Returns**: `DeleteBoolResponse`.

```python
# Python
await qn.admin.delete_ip("ep-123", "ip-1")
```

#### Domain Masks

##### `create_domain_mask` / `createDomainMask`

Adds a custom domain mask to an endpoint.

**Parameters**: `id` (endpoint id, required); body: `domain_mask` (string, optional).

**Returns**: nothing.

```python
# Python
await qn.admin.create_domain_mask("ep-123", domain_mask="rpc.example.com")
```

##### `delete_domain_mask` / `deleteDomainMask`

Removes a domain mask.

**Parameters**: `id` (endpoint id, required), `domain_mask_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_domain_mask("ep-123", "dm-1")
```

#### JWTs

##### `create_jwt` / `createJwt`

Configures JWT validation on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `public_key` (string, optional), `kid` (string, optional), `name` (string, optional).

**Returns**: nothing.

```python
# Python
await qn.admin.create_jwt(
    "ep-123",
    public_key="-----BEGIN PUBLIC KEY-----\n...",
    kid="key-1",
    name="primary",
)
```

##### `delete_jwt` / `deleteJwt`

Removes a JWT configuration.

**Parameters**: `id` (endpoint id, required), `jwt_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_jwt("ep-123", "jwt-1")
```

#### Request Filters

Whitelist specific RPC methods on an endpoint. Requests for methods not on the list are blocked when the feature is enabled.

##### `create_request_filter` / `createRequestFilter`

**Parameters**: `id` (endpoint id, required); body: `method` (string[], optional). Ruby's Hash key is `methods` (plural).

**Returns**: `CreateRequestFilterResponse` with `data.id`.

```python
# Python
resp = await qn.admin.create_request_filter(
    "ep-123",
    method=["eth_blockNumber", "eth_getBalance"],
)
```

##### `update_request_filter` / `updateRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required); body: `method` (string[], optional). Ruby's Hash keys are `request_filter_id` and `methods` (plural).

**Returns**: nothing.

```python
# Python
await qn.admin.update_request_filter("ep-123", "f-1", method=["eth_call"])
```

##### `delete_request_filter` / `deleteRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_request_filter("ep-123", "f-1")
```

#### Multichain

##### `enable_multichain` / `enableMultichain`

Enables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```python
# Python
await qn.admin.enable_multichain("ep-123")
```

##### `disable_multichain` / `disableMultichain`

Disables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```python
# Python
await qn.admin.disable_multichain("ep-123")
```

#### IP Custom Headers

##### `create_or_update_ip_custom_header` / `createOrUpdateIpCustomHeader`

Sets the custom header used to identify the client IP (e.g. when traffic is proxied).

**Parameters**: `id` (endpoint id, required); body: `header_name` (string, required).

**Returns**: `CreateOrUpdateIpCustomHeaderResponse` with `data.header_name`.

```python
# Python
await qn.admin.create_or_update_ip_custom_header("ep-123", header_name="X-Forwarded-For")
```

##### `delete_ip_custom_header` / `deleteIpCustomHeader`

Removes the custom IP header configuration.

**Parameters**: `id` (endpoint id, required).

**Returns**: `DeleteBoolResponse`.

```python
# Python
await qn.admin.delete_ip_custom_header("ep-123")
```

#### Method Rate Limits

##### `get_method_rate_limits` / `getMethodRateLimits`

Lists method-level rate limiters configured on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetMethodRateLimitsResponse` with `data.rate_limiters: MethodRateLimiter[]`.

```python
# Python
resp = await qn.admin.get_method_rate_limits("ep-123")
```

##### `create_method_rate_limit` / `createMethodRateLimit`

Creates a new method-level rate limiter.

**Parameters**: `id` (endpoint id, required); body: `interval` (string, e.g. `"second"`), `methods` (string[]), `rate` (i32).

**Returns**: `CreateMethodRateLimitResponse` with `data: MethodRateLimiter`.

```python
# Python
resp = await qn.admin.create_method_rate_limit(
    "ep-123",
    interval="second",
    methods=["eth_call"],
    rate=10,
)
```

##### `update_method_rate_limit` / `updateMethodRateLimit`

Updates an existing rate limiter. Only provided fields change.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required); body: `methods` (string[], optional), `status` (`"enabled"` | `"disabled"`, optional), `rate` (i32, optional).

**Returns**: `UpdateMethodRateLimitResponse`.

```python
# Python
await qn.admin.update_method_rate_limit("ep-123", "rl-1", rate=50)
```

##### `delete_method_rate_limit` / `deleteMethodRateLimit`

Deletes a rate limiter.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_method_rate_limit("ep-123", "rl-1")
```

#### Endpoint Rate Limits

##### `update_rate_limits` / `updateRateLimits`

Partial update of the endpoint-level RPS / RPM / RPD caps. Only buckets included in the request are modified — omitted buckets are left unchanged. Values are capped by the account's plan tier. Sends `PATCH`.

**Parameters**: `id` (endpoint id, required); `rate_limits`: `RateLimitSettings` (`rps`, `rpm`, `rpd`, all optional).

**Returns**: nothing.

```python
# Python
await qn.admin.update_rate_limits("ep-123", rps=100, rpm=5000)
```

##### `get_rate_limits` / `getRateLimits`

Returns the rate-limit rows currently enforced on the endpoint, each identifying its `bucket` (`"rps"` / `"rpm"` / `"rpd"`), `rate_limit`, and `source` (`"plan_default"` or `"user_override"`). User-set overrides expose an `id` (camelCased `id` in Node) you can pass to `delete_rate_limit_override`.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetRateLimitsResponse` with `data.rate_limits: RateLimitEntry[]`.

```python
# Python
resp = await qn.admin.get_rate_limits("123")
for row in resp.data.rate_limits:
    print(row.bucket, row.rate_limit, row.source, row.id)
```

##### `delete_rate_limit_override` / `deleteRateLimitOverride`

Deletes a user-set rate-limit override by UUID. Plan defaults are not deletable — passing a UUID that does not match a user-set override on the endpoint returns 404.

**Parameters**: `id` (endpoint id, required); `override_id` / `overrideId` (UUID returned by `get_rate_limits`, required).

**Returns**: nothing.

```python
# Python
await qn.admin.delete_rate_limit_override("123", "ovr-uuid")
```

#### Endpoint URLs

##### `get_endpoint_urls` / `getEndpointUrls`

Returns the HTTP and WebSocket URLs for the endpoint without fetching the full endpoint record. For multichain endpoints, `multichain_urls` / `multichainUrls` is a per-network map of additional URLs; for single-chain endpoints it is `None` / `null`.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetEndpointUrlsResponse` with `data.http_url`, `data.wss_url`, and `data.multichain_urls`.

```python
# Python
resp = await qn.admin.get_endpoint_urls("123")
print(resp.data.http_url)
if resp.data.multichain_urls:
    for network, urls in resp.data.multichain_urls.items():
        print(network, urls.http_url)
```

#### Metrics

##### `get_endpoint_metrics` / `getEndpointMetrics`

Returns metric series for an endpoint over a time period.

**Parameters**: `id` (endpoint id, required); body: `period` (`"hour"` | `"day"` | `"week"` | `"month"`), `metric` (e.g. `"method_calls_over_time"`, `"response_status_breakdown"`).

**Returns**: `GetEndpointMetricsResponse` with `data: list[EndpointMetric]`. Each `EndpointMetric` has a `tag: list[str]` and a `data: list[list[int]]` of `[timestamp, value]` pairs. Single-axis series (e.g. `response_time_over_time` with a percentile) come back as a one-element tag like `["p95"]`; multi-axis series come back as `["network", "arbitrum-mainnet"]`.

```python
# Python
resp = await qn.admin.get_endpoint_metrics(
    "ep-123",
    period="day",
    metric="method_calls_over_time",
)
```

##### `get_account_metrics` / `getAccountMetrics`

Returns account-level metric series. Supports an optional `percentile` (e.g. `"p50"`, `"p95"`, `"p99"`) for latency metrics.

**Parameters**: `period` (required), `metric` (required), `percentile` (string, optional).

**Returns**: `GetAccountMetricsResponse` with `data: list[EndpointMetric]`. See `get_endpoint_metrics` above for the `tag: list[str]` shape.

```python
# Python
resp = await qn.admin.get_account_metrics(period="day", metric="credits_over_time")
```

#### Chains

##### `list_chains` / `listChains`

Lists the blockchains supported by Quicknode along with their networks.

**Parameters**: none.

**Returns**: `ListChainsResponse` with `data: Chain[]`.

```python
# Python
resp = await qn.admin.list_chains()
```

#### Billing

##### `list_invoices` / `listInvoices`

Lists invoices on the account.

**Parameters**: none.

**Returns**: `ListInvoicesResponse` with `data.invoices: Invoice[]`.

```python
# Python
resp = await qn.admin.list_invoices()
```

##### `list_payments` / `listPayments`

Lists payments on the account.

**Parameters**: none.

**Returns**: `ListPaymentsResponse` with `data.payments: Payment[]`.

```python
# Python
resp = await qn.admin.list_payments()
```

#### Bulk Operations

##### `bulk_update_endpoint_status` / `bulkUpdateEndpointStatus`

Activates or pauses many endpoints at once.

**Parameters**: `ids` (string[], required), `status` (`"active"` | `"paused"`, required).

**Returns**: `BulkUpdateEndpointStatusResponse` with per-endpoint `results`.

```python
# Python
resp = await qn.admin.bulk_update_endpoint_status(ids=["ep-1", "ep-2"], status="paused")
```

##### `bulk_add_tag` / `bulkAddTag`

Applies a tag (created if missing) to many endpoints at once.

**Parameters**: `ids` (string[], required), `label` (string, required).

**Returns**: `BulkAddTagResponse`.

```python
# Python
resp = await qn.admin.bulk_add_tag(ids=["ep-1", "ep-2"], label="prod")
```

##### `bulk_remove_tag` / `bulkRemoveTag`

Removes a tag from many endpoints at once.

**Parameters**: `ids` (string[], required), `tag_id` (i32, required).

**Returns**: `BulkRemoveTagResponse`.

```python
# Python
resp = await qn.admin.bulk_remove_tag(ids=["ep-1", "ep-2"], tag_id=42)
```

#### Account Tags

##### `list_tags` / `listTags`

Lists every tag on the account along with usage counts.

**Parameters**: none.

**Returns**: `ListTagsResponse` with `data.tags: AccountTag[]`.

```python
# Python
resp = await qn.admin.list_tags()
```

##### `rename_tag` / `renameTag`

Renames an account-level tag.

**Parameters**: `tag_id` (i32, required); body: `label` (string, required).

**Returns**: `RenameTagResponse` with updated `AccountTag`.

```python
# Python
resp = await qn.admin.rename_tag(42, label="staging")
```

##### `delete_account_tag` / `deleteAccountTag`

Deletes a tag from the account. The tag must first be removed from any endpoints using it.

**Parameters**: `id` (i32, required).

**Returns**: `DeleteAccountTagResponse`.

```python
# Python
await qn.admin.delete_account_tag(42)
```

---

### Streams Client

Accessed as `qn.streams`. Creates and manages blockchain data streams that deliver filtered on-chain events to configured destinations. Backed by `https://api.quicknode.com/streams/rest/v1/`.

#### Datasets, Regions, and Destinations

Enums used across stream methods:

- **`StreamRegion`**: `UsaEast`, `EuropeCentral`, `AsiaEast` (wire values: `usa_east`, `europe_central`, `asia_east`).
- **`StreamDataset`**: `Block`, `BlockWithReceipts`, `Transactions`, `Logs`, `Receipts`, `TraceBlocks`, `DebugTraces`, `BlockWithReceiptsDebugTrace`, `BlockWithReceiptsTraceBlock`, `BlobSidecars`, `ProgramsWithLogs`, `Ledger`, `Events`, `Orders`, `Trades`, `BookUpdates`, `Twap`, `WriterActions`.
- **`StreamStatus`**: `Active`, `Paused`, `Terminated`, `Completed`, `Blocked`.
- **`FilterLanguage`**: `Javascript`, `Go`, `Wasm`.
- **`StreamMetadataLocation`**: `Body`, `Header`, `None`.

Destinations are expressed via `DestinationAttributes`. Each variant wraps an attribute struct:

| Variant | Struct | Key fields |
|---|---|---|
| `Webhook` | `WebhookAttributes` | `url`, `max_retry`, `retry_interval_sec`, `post_timeout_sec`, `compression`, `security_token?` |
| `S3` | `S3Attributes` | `endpoint`, `access_key`, `secret_key`, `bucket`, `object_prefix`, `compression`, `file_type`, `max_retry`, `retry_interval_sec`, `use_ssl?` |
| `Azure` | `AzureAttributes` | `storage_account`, `sas_token`, `container`, `compression`, `file_type`, `max_retry`, `retry_interval_sec`, `blob_prefix?` |
| `Postgres` | `PostgresAttributes` | `host`, `port`, `username`, `password`, `database`, `schema`, `table`, `max_retry`, `retry_interval_sec`, `use_ssl?` |
| `Mysql` | `MysqlAttributes` | `host`, `port`, `username`, `password`, `database`, `table`, `max_retry`, `retry_interval_sec`, `use_ssl?` |
| `Mongo` | `MongoAttributes` | `connection_string`, `database`, `collection`, `max_retry`, `retry_interval_sec` |
| `Clickhouse` | `ClickhouseAttributes` | `host`, `port`, `username`, `password`, `database`, `table`, `max_retry`, `retry_interval_sec`, `use_ssl?` |
| `Snowflake` | `SnowflakeAttributes` | `account`, `warehouse`, `database`, `schema`, `table`, `username`, `private_key`, `max_retry`, `retry_interval_sec` |
| `Kafka` | `KafkaAttributes` | `bootstrap_servers`, `topic`, `compression`, `max_retry`, `retry_interval_sec` |
| `Redis` | `RedisAttributes` | `host`, `port`, `username`, `password`, `key`, `max_retry`, `retry_interval_sec`, `use_ssl?` |

Wrapper naming per language:

- **Rust**: `DestinationAttributes::Webhook(WebhookAttributes { .. })` etc.
- **Python**: `StreamWebhookDestination(WebhookAttributes(...))`, `StreamS3Destination(S3Attributes(...))`, etc.
- **Node.js**: a discriminated object `{ destination: "webhook", attributes: { ... } }` using string discriminators.
- **Ruby**: factory methods on `QuicknodeSdk::DestinationAttributes`, e.g. `QuicknodeSdk::DestinationAttributes.webhook(url: ..., ...)`.

#### Streams methods

##### `create_stream` / `createStream`

Creates a new stream that delivers filtered data to the configured destination. Start from a specific block for backfills or from the tip for real-time streaming. Supports filters, reorg handling, distance-from-tip, elastic batching, notification emails, and extra destinations.

**Parameters**: `CreateStreamParams` — required: `name`, `region`, `network`, `dataset`, `start_range` (i64), `end_range` (i64, `-1` = follow tip), `destination_attributes`, `plan`, `threshold_fetch_buffer`. Common optional fields: `dataset_batch_size`, `include_stream_metadata`, `fix_block_reorgs`, `keep_distance_from_tip`, `elastic_batch_enabled`, `filter_function`, `filter_language`, `status`, `notification_email`, `extra_destinations`.

**Returns**: `Stream`.

```python
# Python
from sdk import WebhookAttributes, StreamWebhookDestination

stream = await qn.streams.create_stream(
    name="My Stream",
    network="ethereum-mainnet",
    dataset="block",
    region="usa_east",
    start_range=24691804,
    end_range=24691904,
    destination_attributes=StreamWebhookDestination(
        WebhookAttributes(
            url="https://webhook.site/...",
            max_retry=3,
            retry_interval_sec=1,
            post_timeout_sec=10,
            compression="none",
        )
    ),
    plan="growth_plan",
    threshold_fetch_buffer=1000,
    status="active",
)
```

##### `list_streams` / `listStreams`

Paginated list of streams on the account.

**Parameters** (all optional): `offset` (i64), `limit` (i64), `order_by` (string), `order_direction` (`"asc"` | `"desc"`), `stream_type` (string).

**Returns**: `ListStreamsResponse` with `data: Stream[]` and `page_info`.

```python
# Python
resp = await qn.streams.list_streams()
```

##### `get_stream` / `getStream`

Fetches one stream by id.

**Parameters**: `id` (string, required).

**Returns**: `Stream`.

```python
# Python
stream = await qn.streams.get_stream("stream-id")
```

##### `update_stream` / `updateStream`

Partially updates a stream. Omitted fields are left unchanged.

**Parameters**: `id` (string, required); body: any field from `CreateStreamParams` (all optional).

**Returns**: updated `Stream`.

```python
# Python
stream = await qn.streams.update_stream("stream-id", name="Renamed")
```

##### `delete_stream` / `deleteStream`

Deletes one stream by id.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.streams.delete_stream("stream-id")
```

##### `delete_all_streams` / `deleteAllStreams`

Deletes every stream on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```python
# Python
await qn.streams.delete_all_streams()
```

##### `activate_stream` / `activateStream`

Resumes delivery on a stream from its current position.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.streams.activate_stream("stream-id")
```

##### `pause_stream` / `pauseStream`

Halts delivery on a stream.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```python
# Python
await qn.streams.pause_stream("stream-id")
```

##### `test_filter` / `testFilter`

Runs a filter function against a block so it can be validated before being attached to a live stream.

**Parameters**: `network` (string, required), `dataset` (`StreamDataset`, required), `block` (string, required), `filter_function` (string, optional), `filter_language` (`FilterLanguage`, optional), `address_book_config` (optional).

**Returns**: `TestFilterResponse` with `result` and `logs`.

```python
# Python
resp = await qn.streams.test_filter(
    network="ethereum-mainnet",
    dataset="block",
    block="17811625",
)
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled (active) streams, optionally filtered by type.

**Parameters**: `stream_type` (string, optional).

**Returns**: `EnabledCountResponse` with `total`.

```python
# Python
resp = await qn.streams.get_enabled_count()
```

---

### Webhooks Client

Accessed as `qn.webhooks`. Creates webhooks from filter templates and manages their lifecycle. Backed by `https://api.quicknode.com/webhooks/rest/v1/`.

#### Templates and destination

`WebhookTemplateId` identifies the filter template:

| Variant | Wire value |
|---|---|
| `EvmWalletFilter` | `evmWalletFilter` |
| `EvmContractEvents` | `evmContractEvents` |
| `EvmAbiFilter` | `evmAbiFilter` |
| `SolanaWalletFilter` | `solanaWalletFilter` |
| `BitcoinWalletFilter` | `bitcoinWalletFilter` |
| `XrplWalletFilter` | `xrplWalletFilter` |
| `HyperliquidWalletEventsFilter` | `hyperliquidWalletEventsFilter` |
| `StellarWalletTransactionsSourceAccountFilter` | `stellarWalletTransactionsSourceAccountFilter` |

`TemplateArgs` carries the arguments; construct one per template via the factory methods:

| Factory | Argument struct | Fields |
|---|---|---|
| `evm_wallet_filter` | `EvmWalletFilterTemplate` | `wallets: string[]` |
| `evm_contract_events` | `EvmContractEventsTemplate` | `contracts: string[]`, `event_hashes?: string[]` |
| `evm_abi_filter` | `EvmAbiFilterTemplate` | `abi: string` (JSON), `contracts: string[]` |
| `solana_wallet_filter` | `SolanaWalletFilterTemplate` | `accounts: string[]` |
| `bitcoin_wallet_filter` | `BitcoinWalletFilterTemplate` | `wallets: string[]` |
| `xrpl_wallet_filter` | `XrplWalletFilterTemplate` | `wallets: string[]` |
| `hyperliquid_wallet_events_filter` | `HyperliquidWalletEventsFilterTemplate` | `wallets: string[]` |
| `stellar_wallet_transactions_filter` | `StellarWalletTransactionsFilterTemplate` | `source_accounts: string[]` |

`WebhookDestinationAttributes`: `url` (required), `security_token` (optional — auto-generated if omitted), `compression` (optional — `"none"` | `"gzip"`).

`WebhookStartFrom`: `Last` (resume from last delivered block) or `Latest` (start from newest).

In Ruby, `template_args` is passed as a JSON string under the key `template_args_json`; destination is passed as a JSON string under `destination_attributes_json`.

#### Webhooks methods

##### `list_webhooks` / `listWebhooks`

Paginated list of webhooks.

**Parameters** (all optional): `limit` (i64), `offset` (i64).

**Returns**: `ListWebhooksResponse` with `data: Webhook[]` and `pageInfo: WebhookPageInfo { limit, offset, total }`.

```python
# Python
resp = await qn.webhooks.list_webhooks()
```

##### `get_webhook` / `getWebhook`

Fetches a webhook by id.

**Parameters**: `id` (string, required).

**Returns**: `Webhook`.

```python
# Python
webhook = await qn.webhooks.get_webhook("wh-1")
```

##### `create_webhook_from_template` / `createWebhookFromTemplate`

Creates a webhook from a predefined filter template.

**Parameters**: `name` (required), `network` (required), `destination_attributes` (`WebhookDestinationAttributes`, required), `template_args` (required — use the `TemplateArgs` enum variant for the chosen template), `notification_email` (optional).

**Returns**: `Webhook`.

```python
# Python
from sdk import EvmWalletFilterArgs, EvmWalletFilterTemplate, WebhookDestinationAttributes

webhook = await qn.webhooks.create_webhook_from_template(
    name="Wallet Webhook",
    network="ethereum-mainnet",
    destination_attributes=WebhookDestinationAttributes(url="https://webhook.site/..."),
    template_args=EvmWalletFilterArgs(
        EvmWalletFilterTemplate(wallets=["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"])
    ),
)
```

##### `update_webhook` / `updateWebhook`

Partially updates a webhook's name, notification email, and/or destination. If `destination_attributes` is supplied without `security_token`, a new token is generated automatically.

**Parameters**: `id` (required); body — all optional: `name`, `notification_email`, `destination_attributes`. In Ruby, `destination_attributes` is passed as a JSON string under the key `destination_attributes_json`.

**Returns**: updated `Webhook`.

```python
# Python
webhook = await qn.webhooks.update_webhook("wh-1", name="Renamed Webhook")
```

##### `update_webhook_template` / `updateWebhookTemplate`

Updates the template args (and optionally name, email, destination) on an existing template-backed webhook.

**Parameters**: `webhook_id` (required), `template_args` (required); optional: `name`, `notification_email`, `destination_attributes`.

**Returns**: updated `Webhook`.

```python
# Python
webhook = await qn.webhooks.update_webhook_template(
    "wh-1",
    template_args=EvmWalletFilterArgs(
        EvmWalletFilterTemplate(wallets=["0xnewwallet"])
    ),
)
```

##### `delete_webhook` / `deleteWebhook`

Deletes a webhook.

**Parameters**: `id` (required).

**Returns**: nothing.

```python
# Python
await qn.webhooks.delete_webhook("wh-1")
```

##### `delete_all_webhooks` / `deleteAllWebhooks`

Deletes every webhook on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```python
# Python
await qn.webhooks.delete_all_webhooks()
```

##### `pause_webhook` / `pauseWebhook`

Pauses a webhook so it stops delivering events.

**Parameters**: `id` (required).

**Returns**: nothing.

```python
# Python
await qn.webhooks.pause_webhook("wh-1")
```

##### `activate_webhook` / `activateWebhook`

Activates a paused or new webhook so it resumes delivering events. `start_from` determines where processing resumes.

**Parameters**: `id` (required), `start_from` (`WebhookStartFrom`, required — `Last` or `Latest`).

**Returns**: nothing.

```python
# Python
await qn.webhooks.activate_webhook("wh-1", start_from="latest")
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled webhooks.

**Parameters**: none.

**Returns**: `WebhookEnabledCountResponse` with `total`.

```python
# Python
resp = await qn.webhooks.get_enabled_count()
```

---

### KV Store Client

Accessed as `qn.kvstore`. Provides two primitives — **sets** (single string values under a key) and **lists** (ordered collections of strings under a key). Backed by `https://api.quicknode.com/kv/rest/v1/`.

#### Sets

##### `create_set` / `createSet`

Stores a single string value under a key.

**Parameters**: `key` (string, required), `value` (string, required).

**Returns**: nothing.

```python
# Python
await qn.kvstore.create_set(key="my-key", value="hello")
```

##### `get_sets` / `getSets`

Paginated page of key/value entries.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetSetsResponse` — `{ data: KvSetEntry[], cursor: string }`.

```python
# Python
resp = await qn.kvstore.get_sets()
```

##### `get_set` / `getSet`

Returns the value stored under a key.

**Parameters**: `key` (string, required).

**Returns**: `GetSetResponse` with `value`.

```python
# Python
resp = await qn.kvstore.get_set("my-key")
```

##### `bulk_sets` / `bulkSets`

Adds and/or deletes multiple sets in a single request.

**Parameters** (at least one required): `add_sets` (map<string,string>, optional), `delete_sets` (string[], optional).

**Returns**: nothing.

```python
# Python
await qn.kvstore.bulk_sets(
    add_sets={"k1": "v1"},
    delete_sets=["old-key"],
)
```

##### `delete_set` / `deleteSet`

Deletes a single set.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```python
# Python
await qn.kvstore.delete_set("my-key")
```

#### Lists

##### `create_list` / `createList`

Creates a list under a key, seeded with the initial items.

**Parameters**: `key` (string, required), `items` (string[], required).

**Returns**: nothing.

```python
# Python
await qn.kvstore.create_list(key="my-list", items=["0xabc", "0xdef"])
```

##### `get_lists` / `getLists`

Paginated page of list keys.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetListsResponse` — `{ data: { keys: string[] }, cursor: string }`.

```python
# Python
resp = await qn.kvstore.get_lists()
```

##### `get_list` / `getList`

Paginated page of items for a specific list.

**Parameters**: `key` (string, required); optional `limit` (i64), `cursor` (string).

**Returns**: `GetListResponse` — `{ data: { items: string[] }, cursor: string }`.

```python
# Python
resp = await qn.kvstore.get_list("my-list")
```

##### `update_list` / `updateList`

Adds and/or removes items in a single operation.

**Parameters**: `key` (string, required); optional: `add_items` (string[]), `remove_items` (string[]).

**Returns**: nothing.

```python
# Python
await qn.kvstore.update_list(
    "my-list",
    add_items=["0x456"],
    remove_items=["0xabc"],
)
```

##### `add_list_item` / `addListItem`

Appends a single item to a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```python
# Python
await qn.kvstore.add_list_item("my-list", "0x123")
```

##### `list_contains_item` / `listContainsItem`

Checks whether a list contains a specific item.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: `ListContainsItemResponse` with `exists: bool`.

```python
# Python
resp = await qn.kvstore.list_contains_item("my-list", "0x123")
```

##### `delete_list_item` / `deleteListItem`

Removes a single item from a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```python
# Python
await qn.kvstore.delete_list_item("my-list", "0x123")
```

##### `delete_list` / `deleteList`

Deletes a list and all of its items.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```python
# Python
await qn.kvstore.delete_list("my-list")
```

## Error Handling

Every binding exposes a typed exception hierarchy derived from the core `SdkError`
enum (`crates/core/src/errors.rs`). Catch the base class (`QuicknodeError`) for any SDK-originated failure, or a specific
subclass to branch on transport vs. API semantics.

| Logical class        | When it fires                                               | Extra fields         |
|----------------------|-------------------------------------------------------------|----------------------|
| `QuicknodeError`     | base class; catches everything below                        | —                    |
| `ConfigError`        | invalid config or URL surfaced at construction time         | —                    |
| `HttpError`          | transport failure that isn't a timeout/connect              | —                    |
| `TimeoutError`       | request timed out (subclass of `HttpError`)                 | —                    |
| `ConnectionError`    | connection refused / DNS / TLS (subclass of `HttpError`)    | —                    |
| `ApiError`           | non-2xx HTTP response                                       | `status`, `body`     |
| `DecodeError`        | 2xx response but JSON parse failed                          | `body`               |

Class names: Importable from `sdk`: `QuicknodeError`, `ConfigError`, `HttpError`, `TimeoutError`, `ConnectionError`, `ApiError`, `DecodeError`.

```python
# Python
from sdk import ApiError, TimeoutError
try:
    await qn.admin.show_endpoint("missing")
except ApiError as e:
    if e.status == 404:
        print(f"not found: {e.body}")
    else:
        raise
except TimeoutError:
    print("timed out")
```

## License

MIT
