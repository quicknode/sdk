# quicknode-sdk (Python)

Python bindings for the Quicknode SDK.

This is one of four language bindings published from the same Rust core. See the [project README](https://github.com/quicknode/sdk/blob/main/README.md) for the polyglot overview, development setup, and release process.

> **Pre-1.0**: While on `0.x`, releases may contain breaking changes. Check the [release notes](https://github.com/quicknode/sdk/releases) before upgrading.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
  - [Option A — Pass config directly](#option-a--pass-config-directly)
  - [Option B — Load from environment (`from_env()`)](#option-b--load-from-environment-from_env)
  - [Custom headers and `User-Agent`](#custom-headers-and-user-agent)
- [Platform Support](#platform-support)
- [API Reference](#api-reference)
  - [Language conventions](#language-conventions)
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
    - [Account](#account)
    - [Billing](#billing)
    - [Bulk Operations](#bulk-operations)
    - [Account Tags](#account-tags)
    - [Additional security operations](#additional-security-operations)
    - [Mutation responses](#mutation-responses)
    - [Usage and metrics summaries](#usage-and-metrics-summaries)
    - [Activity logs](#activity-logs)
    - [Transactions](#transactions)
    - [Admin compatibility notes](#admin-compatibility-notes)
  - [Streams Client](#streams-client)
    - [Datasets, Regions, and Destinations](#datasets-regions-and-destinations)
    - [Streams methods](#streams-methods)
  - [Webhooks Client](#webhooks-client)
    - [Templates and destination](#templates-and-destination)
    - [Webhooks methods](#webhooks-methods)
  - [KV Store Client](#kv-store-client)
    - [Sets](#sets)
    - [Lists](#lists)
  - [SQL Client](#sql-client)
  - [RPC & Tooling Access](#rpc--tooling-access)
- [Crypto-micropayment lane (`rpc.call`)](#crypto-micropayment-lane-rpccall)
  - [Wallet generation](#wallet-generation)
  - [x402 credit drawdown (authenticate once, then draw one credit per call)](#x402-credit-drawdown-authenticate-once-then-draw-one-credit-per-call)
    - [Testnet faucet](#testnet-faucet)
  - [MPP payment channel (deposit once, then vouchers)](#mpp-payment-channel-deposit-once-then-vouchers)
- [Error Handling](#error-handling)
- [License](#license)

## Installation

`uv add quicknode-sdk`

## Quick Start

Construct the SDK once, then reach into the five sub-clients (`admin`, `streams`, `webhooks`, `kvstore`, `sql`). Subsequent API Reference snippets assume you have a `qn` handle from one of these blocks.

```python
# Python
import asyncio
from quicknode_sdk import QuicknodeSdk

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
from quicknode_sdk import QuicknodeSdk, SdkFullConfig, HttpConfig
qn = QuicknodeSdk(SdkFullConfig(api_key="your-key", http=HttpConfig(timeout_secs=30)))

# api_key is optional: the crypto-micropayment lane pays per request instead, so
# api_key=None builds a usable SDK. Every other client still needs one, and
# from_env() always requires QN_SDK__API_KEY.
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
| `QN_SDK__SQL__BASE_URL` | no | `https://api.quicknode.com/sql/rest/v1/` | Override SQL Explorer base URL |
| `QN_SDK__HTTP__HEADERS__<NAME>` | no | — | Custom HTTP header sent on every request. Overrides SDK-managed headers (see below). |

### Custom headers and `User-Agent`

Every outbound HTTP request includes an auto-generated `User-Agent` of the form:

```
quicknode-sdk-<language>/<sdk-version> (<os>-<arch>; <language>-<runtime-version>)
```

You can attach arbitrary headers via `HttpConfig.headers`. **These headers OVERRIDE any SDK-managed header with the same name**, including `User-Agent`, `x-api-key`, `Accept`, and `Content-Type`. Use this to inject correlation IDs, proxy auth, or to replace the default `User-Agent`. Header names are matched case-insensitively.

```python
from quicknode_sdk import QuicknodeSdk, SdkFullConfig, HttpConfig

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

Returns a paginated list of endpoints on the account with optional search, filters (networks, statuses, labels, tags, dedicated, flat-rate), sorting, and pagination. The API caps `limit` at 100. Nonempty `tag_ids` and `tag_labels` filters are mutually exclusive.

**Parameters** (all optional): `limit` (i32), `offset` (i32), `search` (string), `sort_by` (string), `sort_direction` (`"asc"` | `"desc"`), `networks` (string[]), `statuses` (string[]), `labels` (string[]), `dedicated` (bool), `is_flat_rate` (bool), `tag_ids` (i32[]), `tag_labels` (string[]).

**Returns**: `GetEndpointsResponse` — `{ data: Endpoint[], pagination?: Pagination }`.

```python
# Python
resp = await qn.admin.get_endpoints(limit=20, sort_by="created_at", sort_direction="desc")
```

##### `create_endpoint` / `createEndpoint`

Creates a new endpoint for the given blockchain and network.

**Parameters**: `chain` and `network` are strings required by the API. The SDK retains its existing optional arguments for compatibility; supply both for a valid request.

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

**Parameters**: `id` (string, required); `label` (string, required by the API). The SDK retains the optional label argument; omitting it produces an API error.

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

**Parameters**: `id` (string, required); `label` (string, required by the API). Labels must be nonblank and contain 3 through 25 characters after trimming. The SDK retains its optional argument and does not trim values.

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

Creates a new team. The name must contain at least four characters.

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

**Parameters**: `id` (i64, required); body: `email` (string, required), `full_name` (string, optional), `role` (string, optional). Standard roles and account custom-role keys are accepted.

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

Endpoint, chain, and tag usage rows include integer `rpc_credits`, `data_credits`, and `bytes`. Shared method rows, including methods nested under endpoints, include integer `bytes`. Existing nullable method metadata and null tag IDs for untagged usage remain nullable.

All usage methods accept optional `start_time` and `end_time` Unix timestamps. Omit both to use the current billing-period default.

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

Fetches a page of request logs for an endpoint. Use ISO 8601 timestamps for explicit bounds. The limit is 1 through 100 and defaults to 20. Set `include_details=true` for full request/response payloads (truncated at 2 KB each).

**Parameters**: `id` (endpoint id, required); query: `from` (string timestamp, required), `to` (string timestamp, required), `include_details` (bool, optional), `limit` (i32, optional), `next_at` (string cursor, optional).

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

Returns the full security configuration for an endpoint: tokens, JWTs, referrers, domain masks, IPs, blocked IPs (`blocked_ips`), request filters, and their per-feature toggles. Snapshot options include `hsts`, `cors`, and `response_logging` (wire key `responseLogging`). Structured filter matchers and `params` are preserved.

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

**Parameters**: `id` (string, required); `options`: `SecurityOptionsUpdate` (`tokens`, `referrers`, `jwts`, `ips`, `domain_masks`, `hsts`, `cors`, `request_filters`, `ip_custom_header`, `response_logging`). The `response_logging` option is sent as `responseLogging` and accepts `"enabled"` or `"disabled"`.

**Returns**: `UpdateSecurityOptionsResponse` with updated `SecurityOption[]`.

```python
# Python
await qn.admin.update_security_options(
    "ep-123",
    tokens="enabled",
    jwts="disabled",
    response_logging="enabled",
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

**Parameters**: `id` (endpoint id, required); body: `referrer` (string, required).

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

**Parameters**: `id` (endpoint id, required); body: `ip` (string, required).

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

**Parameters**: `id` (endpoint id, required); body: `domain_mask` (string, required by the API). The SDK retains its optional argument, but supplies no default value.

**Returns**: nothing.

```python
# Python
await qn.admin.create_domain_mask("ep-123", domain_mask="rpc.example.com")
```

##### `delete_domain_mask` / `deleteDomainMask`

Removes a domain mask.

**Parameters**: `id` (endpoint id, required), `domain_mask_id` (string, required).

**Returns**: `DeleteBoolResponse`.

```python
# Python
await qn.admin.delete_domain_mask("ep-123", "dm-1")
```

#### JWTs

##### `create_jwt` / `createJwt`

Configures JWT validation on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `public_key` (string, required), `kid` (string, required), `name` (string, required).

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

**Parameters**: `id` (endpoint id, required); body: `method` (string[], required). Ruby's Hash key is `methods` (plural).

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

**Returns**: nothing. Supply a method list for an update; an empty request produces an API error.

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

##### Structured patterns and filter listing

`get_request_filters` returns list records with `id`, `pattern`, and `method_whitelist`. This differs from endpoint/security snapshots, where each filter has `id`, a `method` matcher array, and `params`. Snapshot `params` holds every top-level pattern key except `method`, not only a nested `params` matcher.

Use `create_request_filter_pattern` and `update_request_filter_pattern` for a custom JSON object. The existing whitelist methods keep their current inputs, including Ruby's `methods:` keyword. Do not combine a whitelist and a custom pattern in one request or use an update to switch filter kinds. One whitelist and multiple custom filters can coexist. Updates and deletions continue to accept successful empty HTTP 204 responses.

Nested objects, arrays, nulls, and keys such as `anything-but` remain intact. Pass a native object rather than a JSON string. Creation returns `data.id` and, when supplied by the API, `data.pattern`.

```python
pattern = {"method": [{"prefix": "eth_"}], "jsonrpc": ["2.0"]}
created = await qn.admin.create_request_filter_pattern("123", pattern=pattern)
if created.data is not None:
    await qn.admin.update_request_filter_pattern("123", created.data.id, pattern=pattern)
filters = await qn.admin.get_request_filters("123")
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

Creates a new method-level rate limiter. `rate` must be positive; zero produces an API error. Optional `limit_by_ip` applies the limit per client IP and is returned on `MethodRateLimiter`.

**Parameters**: `id` (endpoint id, required); body: `interval` (string, e.g. `"second"`), `methods` (string[]), `rate` (i32), `limit_by_ip` (bool, optional).

**Returns**: `CreateMethodRateLimitResponse` with `data: MethodRateLimiter`.

```python
# Python
resp = await qn.admin.create_method_rate_limit(
    "ep-123",
    interval="second",
    methods=["eth_call"],
    rate=10,
    limit_by_ip=True,
)
```

##### `update_method_rate_limit` / `updateMethodRateLimit`

Updates an existing rate limiter. Only provided fields change. Supply at least one supported field; an empty PATCH produces an API error. `limit_by_ip=false` is an explicit update, not an omitted value.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required); body: `methods` (string[], optional), `status` (`"enabled"` | `"disabled"`, optional), `rate` (i32, optional), `limit_by_ip` (bool, optional).

**Returns**: `UpdateMethodRateLimitResponse`.

```python
# Python
await qn.admin.update_method_rate_limit("ep-123", "rl-1", rate=50, limit_by_ip=False)
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

Partial update of the endpoint-level RPS / RPM / RPD caps. Only supplied fields change. Values are capped by the account's plan tier. Sends `PATCH`. Set `limit_by_ip` (`limitByIp` in Node) to `true` to apply the caps per client IP or `false` to apply them to the whole endpoint. You can update this flag without sending caps. Supply at least one cap or the flag; an empty update returns an API error. Both update methods accept these options.

**Parameters**: `id` (endpoint id, required); `rps`, `rpm`, `rpd`: optional integers; `limit_by_ip`: optional boolean.

**Returns**: nothing.

```python
# Python
await qn.admin.update_rate_limits("ep-123", rps=100, rpm=5000, limit_by_ip=True)
await qn.admin.update_rate_limits("ep-123", limit_by_ip=False)
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

Returns metric series for an endpoint over a time period. Optional integer `percentile` ranges from 1 through 100 and is required for `response_time_over_time` and `method_response_time_max`. For example, use `95`. Metric and period strings remain open.

**Parameters**: `id` (endpoint id, required); query: `period` (`"hour"` | `"day"` | `"week"` | `"month"`), `metric` (e.g. `"method_calls_over_time"`, `"response_status_breakdown"`), `percentile` (integer, optional).

**Returns**: `GetEndpointMetricsResponse` with `data: list[EndpointMetric]`. Each `EndpointMetric` has a `tag: list[str]` and a `data: list[list[int]]` of `[timestamp, value]` pairs. Single-axis series (e.g. `response_time_over_time` with a percentile) come back as a one-element tag like `["p95"]`; multi-axis series come back as `["network", "arbitrum-mainnet"]`.

```python
# Python
resp = await qn.admin.get_endpoint_metrics(
    "ep-123",
    period="day",
    metric="response_time_over_time",
    percentile=95,
)
```

##### `get_account_metrics` / `getAccountMetrics`

Returns account-level metric series. The existing optional `percentile` argument remains for compatibility; the API currently ignores it. Endpoint latency percentile support belongs to `get_endpoint_metrics`.

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

#### Account

##### `account_info` / `accountInfo`

Returns details about the account, including its id, name, creation timestamp, billing version, and current subscription.

**Parameters**: none.

**Returns**: `AccountInfoResponse` with `data: AccountInfo` (including a nested `subscription: AccountSubscription`).

```python
# Python
resp = await qn.admin.account_info()
```

##### `get_api_credits` / `getApiCredits`

Returns the per-method API credit costs for a chain, identified by its slug (the same slugs returned by `list_chains`, e.g. `ethereum`). An unknown chain slug raises `ApiError` (status 404).

**Parameters**: `chain` (string, required) — the chain slug.

**Returns**: `GetApiCreditsResponse` with `data: list[ApiCredit]`, where each `ApiCredit` has `method` and `credits`.

```python
# Python
resp = await qn.admin.get_api_credits("ethereum")
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

Each result can include an `error` string. Check individual results: HTTP 200 can contain both successes and failures. Successful rows can omit `error`.

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

Renames an account-level tag. Labels must be nonblank and contain 3 through 25 characters after trimming. The SDK does not trim values.

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

#### Additional security operations

All methods below take the numeric endpoint `id` as a string. Reads do not enable a security feature.

| Method | Additional input | Returned `data` |
| --- | --- | --- |
| `get_client_certs` | None | `enabled`, `certs` |
| `create_client_cert` | `certificate`, a PEM string | Full certificate state |
| `update_client_certs` | `enabled`, a boolean | Full certificate state |
| `delete_client_cert` | `fingerprint` | Full state after deletion |
| `get_method_blacklist` | None | `methods` |
| `update_method_blacklist` | `methods`, a string array | Replaced method list |
| `get_range_limits` | None | `limits`, `supported_methods` |
| `update_range_limits` | `limits`, a method-to-integer map | Replaced limits and supported methods |
| `get_blocked_ips` | None | Array of string `id` and `ip` records |
| `create_blocked_ip` | `ip`, an address or CIDR string | Created string `id` and `ip` |
| `delete_blocked_ip` | `ip_id` | Boolean confirmation |

Certificate records contain `fingerprint`, `subject`, `issuer`, and `expires_at`. Deleting the last certificate returns disabled state with an empty list; DELETE still has a response body. Invalid, expired, duplicate, or unknown certificates can return HTTP 422.

Blacklist and range-limit updates replace the complete configuration. An empty array clears the blacklist; an empty object clears range limits. Range limits must be positive integers. Read `supported_methods` for the endpoint's supported methods instead of assuming one global list. Blocked-IP calls do not toggle IP security; visibility can depend on the endpoint's IP security setting.

```python
certificates = await qn.admin.get_client_certs("123")
ranges = await qn.admin.get_range_limits("123")
blocked = await qn.admin.get_blocked_ips("123")
blacklist = await qn.admin.update_method_blacklist("123", methods=["eth_sendRawTransaction"])
```

#### Mutation responses

Existing mutation methods keep their original return values. Use these response-returning variants when you need the created record or effective limits. Each takes the same arguments as its existing counterpart.

| Method | Returned `data` |
| --- | --- |
| `create_token_with_response` | Token `id` and `token` |
| `create_referrer_with_response` | Referrer `id` and `url` |
| `create_ip_with_response` | IP record `id` and `ip` |
| `create_domain_mask_with_response` | Record `id` and `domain_mask` |
| `create_jwt_with_response` | JWT `id`, `public_key`, `kid`, and `name` |
| `create_tag_with_response` | `tags` containing association `id`, `tag_id`, `account_id`, `endpoint_id`, label, and timestamps |
| `update_rate_limits_with_response` | `limits` with nullable `rps`, `rpm`, and `rpd` |

Creation and snapshot fields differ: a created referrer uses `url`, while endpoint snapshots use `referrer`; a created domain mask uses `domain_mask`, while snapshots use `domain`. Tag association `id` differs from `tag_id`; these IDs are numeric. Rust Debug output redacts token and JWT credential values. Store returned credentials securely and do not log them.

```python
created = await qn.admin.create_referrer_with_response("123", referrer="https://example.com")
print(created.data.id)
```

#### Usage and metrics summaries

These methods return a resolved time window, typed totals, optional grouped rows, and an optional continuation cursor.

| Method | Grouping | Additional filters |
| --- | --- | --- |
| `get_usage_credits_summary` | None | None |
| `get_usage_rpc_summary` | `method`, `endpoint`, `chain`, `network` | `endpoint_names`, `chains`, `networks`, `methods`, `tag_ids`, `addon_ids` |
| `get_usage_streams_summary` | `stream` | `stream_id` |
| `get_usage_webhooks_summary` | `webhook` | `webhook_id` |
| `get_traffic_summary` | `method`, `endpoint` | Same filters as the RPC summary |
| `get_errors_summary` | `code`, `endpoint`, `network`, `method` | `endpoints`, `networks`, `methods`, `codes`, `status_codes`, `kind` |
| `get_endpoint_errors_summary` | `code`, `network`, `method` | Required endpoint `id`; error-summary filters except `endpoints` |

| Summary | Totals fields |
| --- | --- |
| Credits | `credits` |
| RPC | `requests`, `bytes`, `rpc_credits`, `data_credits`, `credits` |
| Streams | `blocks`, `bytes`, `block_credits` |
| Webhooks | `deliveries`, `credits` |
| Traffic | `successful_requests`, `bytes`, `request_credits`, `bytes_credits`, `credits`, `active_methods` |
| Account and endpoint errors | `rpc_errors`, `http_errors` |

Error breakdown rows contain `kind` and `count`, plus the selected grouping dimensions. Other breakdown rows carry their summary metrics and selected dimensions.

For the first page, supply `last` as a positive integer duration ending now (`s`, `m`, `h`, or `d`), or both `from_time` and `to_time` as ISO 8601 timestamps with timezones. Bare dates are not summary timestamps. Use `group_by` for grouping. Credits summaries have no grouping parameter. The limit is 1 through 1000 and defaults to 100.

Pass native arrays for groups and array filters. The SDK encodes repeated bracketed query keys such as `group_by[]=method&group_by[]=endpoint`. Error codes and HTTP status-code filters remain strings. `endpoint_names` contains endpoint subdomains; RPC, traffic, and account-error result `endpoint` values are integer endpoint IDs. Account-error `endpoints` filters accept integer arrays, such as `[123, 456]`. `tag_ids` and `addon_ids` contain numeric IDs represented as strings. Endpoint error-summary `id` uses the numeric endpoint ID represented as a string.

For error summaries, `kind` is `rpc` or `http`. Method filtering/grouping requires `rpc`; RPC kind rejects `status_codes`, and HTTP kind rejects `codes`. Feature access and retention can produce API errors, including HTTP 403.

Continue with `cursor` alone. A cursor belongs to the original route, account, and query; any repeated parameters, including `limit`, must match. Keep the returned window values: the server can align summary windows to billing boundaries. Totals cover the full window on every page, not only the current rows. The SDK does not round windows or paginate automatically.

Summary metrics use floating-point numbers, including count-like fields, and can contain fractions such as `7.5`. They do not promise arbitrary decimal precision. Legacy usage fields remain integers. RPC breakdown `requests` and traffic breakdown `successful_requests` can be null even though totals are non-null. A missing `breakdown` is not an empty grouped result.

Endpoint grouping values are `int` or `None`. For a grouping dimension, an omitted attribute raises `AttributeError`; a selected null dimension returns `None`. Use `hasattr(row, "method")` to check presence. An omitted breakdown returns `None`. Summary window attributes are `from_time` and `to_time`.

```python
page = await qn.admin.get_usage_rpc_summary(
    last="1h", group_by=["method", "endpoint"], limit=20,
)
print(page.data.totals.credits)
if page.data.next_cursor is not None:
    next_page = await qn.admin.get_usage_rpc_summary(cursor=page.data.next_cursor)
```

#### Activity logs

Lists account changes, newest first. `list_activity_log_filters` returns the actor IDs and target types accepted by `list_activity_logs`; `get_activity_log` returns one entry by UUID.

List filters are `q`, `actor`, `endpoint_id`, `target_type`, `from_time`, `to_time`, `limit`, and `offset`. `endpoint_id` is a numeric endpoint ID represented as a string. `target_type` is sent as `type`. The limit is 0 through 100 and defaults to 20. Offset pagination returns `total`, `limit`, and `offset`; pin `to_time` when exporting because new entries shift offsets. Bare dates are accepted: `from_time` starts at midnight UTC, and `to_time` includes the entire day.

Responses keep actor and target metadata typed. Actor IDs, names, and emails can be null, including for deleted users. Feature access can return HTTP 403; a missing detail entry returns HTTP 404.

```python
filters = await qn.admin.list_activity_log_filters()
page = await qn.admin.list_activity_logs(limit=20)
if page.data:
    detail = await qn.admin.get_activity_log(page.data[0].id)
```

#### Transactions

`list_transactions` returns retained transactions and an optional continuation cursor. A first page needs `last` (for example, `1h`), or both `from_time` and `to_time` as ISO 8601 timestamps with timezones. A cursor alone continues the same resolved window.

Optional filters are `network`, `method`, `state`, `transaction_hash`, and `endpoints`. States are `submitted`, `succeeded`, and `failed`. Endpoint filters accept integer endpoint IDs, such as `[123, 456]`, and use repeated `endpoints[]` query keys. The limit is 1 through 1000 and defaults to 100.

`get_transaction` takes a network, a transaction hash, and optional `include_payloads`. It returns an outer `data` containing `exists` and a second `data` value. An absent or expired transaction can return `exists: false` and `data: null`. Payloads default to excluded; set `include_payloads` to true to request retained request and response text. The SDK preserves that text without parsing it. Feature access can return HTTP 403.

Transaction `endpoint_ids` arrays contain integer endpoint IDs or null elements; the detail array itself can be absent or null. Each attempt has a required integer `endpoint_id`. Block heights remain decimal strings, including values such as `9007199254740993`. Other detail metadata can be absent or null. Attempts retain the order returned by the API.

```python
page = await qn.admin.list_transactions(last="1h", endpoints=[123, 456], limit=20)
if page.data.transactions:
    transaction = page.data.transactions[0]
    result = await qn.admin.get_transaction(
        transaction.network, transaction.transaction_hash, include_payloads=False,
    )
    if result.data.exists and result.data.data is not None:
        print(result.data.data.height)  # str or None
if page.data.next_cursor is not None:
    next_page = await qn.admin.list_transactions(cursor=page.data.next_cursor)
```

#### Admin compatibility notes

Summary endpoint dimensions and transaction endpoint IDs use `int` values. Pass integer lists to account-error and transaction `endpoints` filters. Transaction endpoint-ID lists can contain `None`; attempt `endpoint_id` is required. Update callers that used UUID strings.

Existing methods retain their return values and optional argument requiredness. Required API inputs are documented above; omitted or invalid values still produce API errors. Use the response-returning methods when you need created records.

Rust callers must update affected full struct literals for the added fields, even when those fields are optional. This includes security snapshots/options, filter responses, usage rows, bulk results, method-rate-limit requests/responses, and endpoint metric requests. Request structs with `Default` can use `..Default::default()` for omitted fields; builders avoid full-literal updates where available.

`EndpointRequestFilter.method` changed from `Vec<String>` to `Vec<serde_json::Value>` so it can preserve structured matchers. Convert strings with `.into()` or `serde_json::json!`, and use `as_str()` when reading a string matcher. Match object values as JSON rather than assuming every matcher is a string. Snapshot `params` and custom `pattern` are object maps containing JSON values.

Python endpoint/security response wrappers keep their public class names, getters, and setters. Filter values are native lists and dictionaries; assign through the setter after editing a returned value. Node uses `unknown[]` for matcher arrays and `Record<string, unknown>` for open pattern objects. Ruby continues to return `IndifferentHash` values. JSON object keys are preserved without recursive case conversion.

New summary, activity, and transaction window fields use `from_time` and `to_time` in Rust/Python. Node fields and serialized response keys remain `from` and `to`; Python/Ruby client arguments use `from_time` and `to_time`. Existing endpoint-log arguments do not change.


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
| `Postgres` | `PostgresAttributes` | `host`, `port`, `username`, `password`, `database`, `table_name`, `sslmode`, `max_retry`, `retry_interval_sec` |
| `Kafka` | `KafkaAttributes` | `bootstrap_servers`, `topic_name`, `compression_type`, `batch_size`, `linger_ms`, `max_message_bytes`, `timeout_sec`, `max_retry`, `retry_interval_sec`, `username?`, `password?`, `protocol?`, `mechanisms?` |

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
from quicknode_sdk import WebhookAttributes, StreamWebhookDestination

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

`TemplateArgs` carries the arguments. Each template supports two input forms — inline values (`*Args(*Template(...))`) or a reference to a pre-created list by name (`*ByListArgs(*ByListTemplate(...))`):

| Template | Inline class (fields) | ByList class (fields) |
|---|---|---|
| EVM wallet filter | `EvmWalletFilterArgs(EvmWalletFilterTemplate(wallets=[...]))` | `EvmWalletFilterByListArgs(EvmWalletFilterByListTemplate(wallets_list_name=...))` |
| EVM contract events | `EvmContractEventsArgs(EvmContractEventsTemplate(contracts=[...], event_hashes=[...]))` | `EvmContractEventsByListArgs(EvmContractEventsByListTemplate(contracts_list_name=..., event_hashes_list_name=...))` |
| EVM ABI filter | `EvmAbiFilterArgs(EvmAbiFilterTemplate(abi="...", contracts=[...]))` | `EvmAbiFilterByListArgs(EvmAbiFilterByListTemplate(abi_json="...", contracts_list_name=...))` |
| Solana wallet filter | `SolanaWalletFilterArgs(SolanaWalletFilterTemplate(accounts=[...]))` | `SolanaWalletFilterByListArgs(SolanaWalletFilterByListTemplate(accounts_list_name=...))` |
| Bitcoin wallet filter | `BitcoinWalletFilterArgs(BitcoinWalletFilterTemplate(wallets=[...]))` | `BitcoinWalletFilterByListArgs(BitcoinWalletFilterByListTemplate(wallets_list_name=...))` |
| XRPL wallet filter | `XrplWalletFilterArgs(XrplWalletFilterTemplate(wallets=[...]))` | `XrplWalletFilterByListArgs(XrplWalletFilterByListTemplate(wallets_list_name=...))` |
| Hyperliquid wallet events | `HyperliquidWalletEventsFilterArgs(HyperliquidWalletEventsFilterTemplate(wallets=[...]))` | `HyperliquidWalletEventsFilterByListArgs(HyperliquidWalletEventsFilterByListTemplate(wallets_list_name=...))` |
| Stellar wallet transactions | `StellarWalletTransactionsFilterArgs(StellarWalletTransactionsFilterTemplate(wallets=[...]))` | `StellarWalletTransactionsFilterByListArgs(StellarWalletTransactionsFilterByListTemplate(wallets_list_name=...))` |

`WebhookDestinationAttributes`: `url` (required), `compression` (required — `"none"` | `"gzip"`), `security_token` (optional — auto-generated if omitted).

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
from quicknode_sdk import EvmWalletFilterArgs, EvmWalletFilterTemplate, WebhookDestinationAttributes

webhook = await qn.webhooks.create_webhook_from_template(
    name="Wallet Webhook",
    network="ethereum-mainnet",
    destination_attributes=WebhookDestinationAttributes(
        url="https://webhook.site/...",
        compression="none",
    ),
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

---

### SQL Client

Accessed as `qn.sql`. Runs SQL queries against indexed blockchain data and fetches the database schema. Backed by `https://api.quicknode.com/sql/rest/v1/`.

##### `query`

Executes a SQL query against a cluster and returns the result set. Paginate by writing `LIMIT`/`OFFSET` into the SQL.

**Parameters**: `query` (str, required), `cluster_id` (str, required).

**Returns**: `QueryResponse` — `meta` (list of `ColumnMeta`, each with `name` and `column_type`), `data` (`list[dict]`, rows keyed by column name), `rows`, `rows_before_limit_at_least`, `statistics` (`QueryStatistics` with `elapsed`, `rows_read`, `bytes_read`), and `credits`.

```python
# Python
resp = await qn.sql.query(
    query="SELECT action_type, user FROM hyperliquid_system_actions ORDER BY block_time DESC LIMIT 100",
    cluster_id="hyperliquid-core-mainnet",
)
print(resp.rows, resp.data[0])
```

##### `get_schema`

Fetches the database schema for a cluster: table names, columns, types, sort keys, and partition strategies.

**Parameters**: `cluster_id` (str, required).

**Returns**: `ChainSchema` — `chain`, `cluster_id`, and `tables` (list of `TableSchema`, each with `name`, `engine`, `total_rows`, `partition_key`, `sorting_key`, and `columns` of `ColumnSchema` with `name` and `column_type`).

```python
# Python
schema = await qn.sql.get_schema("hyperliquid-core-mainnet")
print(len(schema.tables))
```

---

### RPC & Tooling Access

Tooling Access provisions a single multichain, read-only endpoint per account and
mints short-lived session JWTs. `qn.rpc` makes JSON-RPC calls directly against that
endpoint, minting and refreshing the JWT automatically — no endpoint URL or token to
manage.

Tooling Access must be enabled once (admin role + eligible plan). Status reads can
return HTTP 403 when feature access is unavailable; successful authentication alone
does not guarantee access. The control-plane methods live on `qn.admin`:

```python
# Python
status = await qn.admin.tooling_access_status()
if not status.enabled:
    await qn.admin.enable_tooling_access()  # idempotent; admin role required

# Make on-chain calls. params defaults to []; pass a list (positional) or dict.
block_number = await qn.rpc.call("eth_blockNumber")
balance = await qn.rpc.call("eth_getBalance", ["0xabc...", "latest"])

# Multichain: select a network by its multichain_urls key. Seed the map first
# (from admin.get_endpoint_urls), then pass network=.
urls = await qn.admin.get_endpoint_urls(endpoint_id)
mc = urls.data.multichain_urls if urls.data else {}
qn.rpc.set_networks({k: v.http_url for k, v in (mc or {}).items()})
slot = await qn.rpc.call("getSlot", network="solana-mainnet")

# Custom endpoint URL: send to a fully-formed HTTP URL, bypassing Tooling Access
# and the JWT (no Authorization header). Per-call via endpoint_url=, or
# client-wide via RpcConfig(endpoint_url=...). endpoint_url and network are
# mutually exclusive (a custom URL is not multichain-routed).
block = await qn.rpc.call("eth_blockNumber", endpoint_url="https://my-endpoint.example/rpc")

# A JSON-RPC error member is raised as RpcError (with .code and .message).
from quicknode_sdk import RpcError
try:
    await qn.rpc.call("eth_getBalance", ["bad"])
except RpcError as e:
    print(e.code, e.message)
```

A host that persists across processes can snapshot the cached token with
`qn.rpc.current_token()` and re-seed it via `RpcConfig(seed=...)` on the next
construction; `refresh_margin_secs` (default 60) tunes how early the token is
refreshed. Set `RpcConfig(endpoint_url=...)` to route every call to a custom
HTTP URL by default (no JWT minted); a per-call `endpoint_url` overrides it.

## Crypto-micropayment lane (`rpc.call`)

Pay per RPC request with a stablecoin instead of a provisioned account + API key,
against Quicknode's `x402.quicknode.com` and `mpp.quicknode.com` gateways. Configure
it by setting `payment` on the RPC config; the SDK runs the `402` → sign → resend
handshake for you. An API key is **not** required for this lane — build a keyless SDK.

There are four payment paths. Two pay per request; two amortize one signature over many
calls.

| Path | Entry point | Gateway | Signs |
|---|---|---|---|
| Per-request x402 | `call` / `call_with_receipt` with `scheme="x402"` | x402 | once per call |
| Per-request MPP charge | `call` / `call_with_receipt` with `scheme="mpp"` | mpp | once per call |
| [x402 credit drawdown](#x402-credit-drawdown-authenticate-once-then-draw-one-credit-per-call) | `gateway_authenticate` → `gateway_drawdown_call` | x402 | once per session |
| [MPP payment channel](#mpp-payment-channel-deposit-once-then-vouchers) | `mpp_open` → `mpp_session_call` | mpp | once per channel |

The signer construction is derived from the scheme and pay network, never stated directly:
**x402/EVM** signs an EIP-712 `TransferWithAuthorization`, **x402/Solana** an SPL
`TransferChecked` in a v0 tx (the gateway sponsors gas), and **MPP/Tempo** a native Tempo
transaction.

`scheme` selects the gateway for `call` only. The `gateway_*` drawdown methods always use
the x402 gateway and the `mpp_*` channel methods always use the MPP gateway, whatever
`scheme` is set to.

`PaymentConfig` fields:

| Field | Meaning |
|---|---|
| `scheme` | `"x402"` (pay-per-request) or `"mpp"` (MPP charge; `"mpp-charge"` is accepted too) |
| `key` | raw private key — EVM/Tempo: hex; Solana: base58 64-byte secret |
| `pay_network` | CAIP-2 pay network, e.g. `eip155:84532`, `solana:5eykt4…` |
| `asset` | token address/mint to pay in (matches the offered menu entry) |
| `max_amount` | **required** spend ceiling in integer base units of `asset` |
| `svm_rpc_url` | optional Solana RPC for x402/Solana payment-build reads (mint + blockhash) |
| `base_url_override` | optional gateway base (testing) |

`network` on the call is the **query** chain (gateway path slug), independent of the
pay network. Use `call_with_receipt` to also get the settlement receipt (`reference` =
settlement tx hash) — populated on the MPP lane, `null`/`None`/`nil` for x402.

**Things to know:**

- **Do not log your own `PaymentConfig`** — the `key` field is readable. The SDK
  never prints it in its own errors/`Debug`, but a plain `print(config)` will show it.
- **`max_amount` is integer base units of the selected asset.** The SDK skips any offered
  entry above it and refuses to sign one — a guard against an overcharging gateway.
- **`PaymentIndeterminateError` means the paid request was sent but the response was lost.**
  You MAY have been charged — do **not** blindly retry.
- **x402/Solana: one payment per call.** Building a payment reads the mint and a recent
  blockhash from a Solana RPC. The default is a public RPC that **rate-limits
  aggressively** — set `svm_rpc_url` to your own endpoint at any volume.

```python
import os
from quicknode_sdk import QuicknodeSdk, SdkFullConfig, RpcConfig, PaymentConfig

qn = QuicknodeSdk(SdkFullConfig(api_key=None, rpc=RpcConfig(payment=PaymentConfig(
    scheme="x402",
    key=os.environ["QN_PAYMENT_KEY"],
    pay_network="eip155:84532",
    asset="0x036CbD53842c5426634e7929541eC2318f3dCF7e",
    max_amount="10000",
))))
resp = await qn.rpc.call_with_receipt("eth_blockNumber", [], "base-sepolia")
print(resp["result"], resp["payment_receipt"])
```

### Wallet generation

`generate_payment_wallet("evm")` creates a fresh keypair offline — no network call, no funds — for
`"evm"`, `"svm"`, or `"tempo"`. The private key is returned **exactly once**, at
generation; nothing in the SDK stores or re-derives it, so persist it immediately.

```python
from quicknode_sdk import generate_payment_wallet

wallet = generate_payment_wallet("evm")
print("fund this address:", wallet["address"])
open("payment.key", "w").write(wallet["key"])  # returned exactly once
```

### x402 credit drawdown (authenticate once, then draw one credit per call)

Cheaper per call than paying per request: one SIWE or SIWS signature mints a session JWT, then
each call draws a single credit from the account balance instead of signing a fresh
settlement. Minting the session is free and moves no funds, so a host can re-authenticate
transparently. Persist it between processes.

Fund the payment wallet out of band — the testnet faucet below, or by sending funds to
`payment_address()` directly. Credits are provisioned against the account gateway-side.

EVM payment networks use SIWE. Solana payment networks use SIWS with an Ed25519
signature encoded as Base58.

| Method | Cost | Returns |
|---|---|---|
| `payment_address()` | free, offline | the wallet address derived from the key |
| `gateway_authenticate()` | free | a dict `{token, exp_unix, account_id}` |
| `gateway_credits(session)` | free | a dict `{account_id, credits}` |
| `gateway_drip(session)` | free (testnet) | a dict with `transfer_id` or `transaction_hash` |
| `gateway_drawdown_call(method, session, network, params=None)` | 1 credit | the JSON-RPC `result` |

```python
session = await qn.rpc.gateway_authenticate()
balance = await qn.rpc.gateway_credits(session)
print("credits:", balance["credits"])
result = await qn.rpc.gateway_drawdown_call("eth_blockNumber", session, "base-sepolia")
```

A `token_expired` surfaces as an `ApiError` with status 401/403; re-authenticate and retry
that call.

#### Testnet faucet

`gateway_drip` requests testnet tokens for the payment **wallet**. Circle Gateway-backed
networks return `transfer_id` because settlement is asynchronous. Direct-transfer
networks such as Arc Testnet return `transaction_hash`. The response is not a credit
balance; call `gateway_credits` separately.

### MPP payment channel (deposit once, then vouchers)

Open a payment channel by depositing into the escrow, then authorize each call with a
cumulative voucher — one `ecrecover` server-side, no on-chain transaction per call.

| Method | Cost | Returns |
|---|---|---|
| `mpp_open(deposit)` | **moves funds** | the channel state dict |
| `mpp_top_up(channel, additional_deposit)` | **moves funds** | the updated channel state |
| `mpp_status(channel)` | **1 request unit** | a dict `{channel_id, accepted_cumulative, spent}` |
| `mpp_session_call(method, network, channel, new_cumulative, params=None)` | 1 request unit | the JSON-RPC `result` |
| `mpp_close(channel)` | settles on-chain | nothing — refunds the unused deposit |

```python
channel = await qn.rpc.mpp_open("1000000")        # persist this dict
new_total = str(int(channel["cumulative_spent"]) + int(channel["per_call"]))
result = await qn.rpc.mpp_session_call(
    "eth_blockNumber", "base-sepolia", channel, new_total
)
# On success, store new_total as the channel's cumulative_spent.
```

**Things to know:**

- **Persist the channel state.** The gateway exposes no read-only channel endpoint, so a
  lost local record means opening (and funding) a new channel.
- **`mpp_status` is not free.** The gateway prices every session POST as a chargeable
  request and computes the balance from the *new* spend a voucher authorizes, so the
  probe advances `cumulative_spent` by `per_call` exactly like a call. Re-persist the
  advanced total. It raises `PaymentUnsupportedError` before any network I/O when the
  channel has no room left for the probe.
- **The lifecycle takes no query network.** A channel is scoped by the configured pay
  network and asset, so one channel funds calls to every supported network. Only
  `mpp_session_call` takes a network, because it routes an RPC method.
- **Amounts are decimal strings, not numbers.** They are `u128` in the core; a Python int has no lossless `u128` conversion, so pass and store them as strings.
- **Advance `cumulative_spent` only after a success.** A voucher authorizes the running total
  *after* the call; re-presenting the current high-water mark authorizes zero and is
  always refused with `insufficient-balance`.



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
| `RpcError`           | JSON-RPC call returned an `error` member                    | `code`, `message`    |
| `PaymentError`       | base class for the crypto-micropayment lane                 | —                    |
| `PaymentUnsupportedError` | no offered payment option matched your selector (or all were over `max_amount`/unsupported) | — |
| `PaymentRejectedError` | the gateway rejected a signed payment (terminal, one resend only) | `status`, `body` |
| `PaymentIndeterminateError` | paid request sent but response lost — MAY have been charged; do NOT blindly retry | — |

Class names: Importable from `quicknode_sdk`: `QuicknodeError`, `ConfigError`, `HttpError`, `TimeoutError`, `ConnectionError`, `ApiError`, `DecodeError`, `RpcError`, `PaymentError`, `PaymentUnsupportedError`, `PaymentRejectedError`, `PaymentIndeterminateError`.

```python
# Python
from quicknode_sdk import ApiError, TimeoutError
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
