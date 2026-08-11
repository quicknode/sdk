# @quicknode/sdk (Node.js)

Node.js / TypeScript bindings for the Quicknode SDK.

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

`npm install @quicknode/sdk`

## Quick Start

Construct the SDK once, then reach into the five sub-clients (`admin`, `streams`, `webhooks`, `kvstore`, `sql`). Subsequent API Reference snippets assume you have a `qn` handle from one of these blocks.

```typescript
// Node.js
import { QuicknodeSdk } from "quicknode-sdk";

const qn = QuicknodeSdk.fromEnv();
const resp = await qn.admin.getEndpoints();
console.log(`${resp.data.length} endpoints`);
```

## Configuration

There are two ways to configure the SDK.

### Option A — Pass config directly

```typescript
// Node.js
import { QuicknodeSdk } from "quicknode-sdk";
const qn = new QuicknodeSdk({ apiKey: "your-key", http: { timeoutSecs: 30 } });

// apiKey is optional: the crypto-micropayment lane pays per request instead, so
// omitting it builds a usable SDK. Every other client still needs one, and
// fromEnv() always requires QN_SDK__API_KEY.
```

### Option B — Load from environment (`from_env()`)

```typescript
// Node.js
const qn = QuicknodeSdk.fromEnv();
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

```ts
import { QuicknodeSdk } from "@quicknode/sdk";

const qn = new QuicknodeSdk({
  apiKey: "your-key",
  http: {
    headers: {
      "X-Correlation-Id": "abc-123",
      "User-Agent": "my-app/1.0", // overrides SDK default
    },
  },
});
```

## Platform Support

Precompiled native modules are published for:

| Platform | Targets |
|---|---|
| Linux (glibc) | `x86_64`, `aarch64` — glibc **2.17+** (manylinux2014) |
| Linux (musl) | `x86_64`, `aarch64` — Alpine and other musl distros |
| macOS | Apple Silicon (`arm64`) |

Linux glibc binaries are built against glibc 2.17 so they load on any distro from 2014 onward — RHEL 7+, Ubuntu 14.04+, Debian 8+, Amazon Linux 2+, SLES 12+, Fedora 19+. On unsupported platforms, `require('@quicknode/sdk')` throws an error listing the available targets.

**Not supported:** RHEL/CentOS 6 (glibc 2.12), Debian 7 (glibc 2.13), Ubuntu 12.04 (glibc 2.15), SLES 11 (glibc 2.11), Intel macOS, Windows.

## API Reference

Snippets assume `qn` was already constructed via the Quick Start. Optional parameters are skipped unless showing one is needed to illustrate usage.

### Language conventions

- Methods are `async` and take a single options object with camelCase keys.

---

### Admin Client

Accessed as `qn.admin`. Manages endpoints, tags, teams, billing, usage, metrics, security, and rate limits. Backed by `https://api.quicknode.com/v0/`.

#### Endpoints

##### `get_endpoints` / `getEndpoints`

Returns a paginated list of endpoints on the account with optional search, filters (networks, statuses, labels, tags, dedicated, flat-rate), sorting, and pagination.

**Parameters** (all optional): `limit` (i32), `offset` (i32), `search` (string), `sort_by` (string), `sort_direction` (`"asc"` | `"desc"`), `networks` (string[]), `statuses` (string[]), `labels` (string[]), `dedicated` (bool), `is_flat_rate` (bool), `tag_ids` (i32[]), `tag_labels` (string[]).

**Returns**: `GetEndpointsResponse` — `{ data: Endpoint[], pagination?: Pagination }`.

```typescript
// Node.js
const resp = await qn.admin.getEndpoints({
  limit: 20,
  sortBy: "created_at",
  sortDirection: "desc",
});
```

##### `create_endpoint` / `createEndpoint`

Creates a new endpoint for the given blockchain and network.

**Parameters**: `chain` (string, optional), `network` (string, optional).

**Returns**: `CreateEndpointResponse` with `data: SingleEndpoint`.

```typescript
// Node.js
const resp = await qn.admin.createEndpoint({ chain: "ethereum", network: "mainnet" });
```

##### `show_endpoint` / `showEndpoint`

Fetches a single endpoint by id, including its full security configuration and rate limits.

**Parameters**: `id` (string, required).

**Returns**: `ShowEndpointResponse` with `data: SingleEndpoint`.

```typescript
// Node.js
const resp = await qn.admin.showEndpoint("ep-123");
```

##### `update_endpoint` / `updateEndpoint`

Updates editable fields on an endpoint. Currently supports `label`.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.updateEndpoint("ep-123", { label: "my label" });
```

##### `archive_endpoint` / `archiveEndpoint`

Archives an endpoint. The HTTP verb is `DELETE` but the effect is archival, not permanent deletion.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.archiveEndpoint("ep-123");
```

##### `update_endpoint_status` / `updateEndpointStatus`

Pauses or unpauses an endpoint.

**Parameters**: `id` (string, required); body: `status` (string, required — `"active"` or `"paused"`).

**Returns**: `UpdateEndpointStatusResponse`.

```typescript
// Node.js
await qn.admin.updateEndpointStatus("ep-123", { status: "paused" });
```

#### Endpoint Tags

Per-endpoint tag add/remove. For account-wide tag management see [Account Tags](#account-tags).

##### `create_tag` / `createTag`

Tags an endpoint with the given label. Creates the tag on the account if it does not exist.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.createTag("ep-123", { label: "prod" });
```

##### `delete_tag` / `deleteTag`

Removes a tag from a specific endpoint.

**Parameters**: `id` (endpoint id, string, required), `tag_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteTag("ep-123", "42");
```

#### Teams

##### `list_teams` / `listTeams`

Lists all teams on the account.

**Parameters**: none.

**Returns**: `ListTeamsResponse` with `data: TeamSummary[]`.

```typescript
// Node.js
const resp = await qn.admin.listTeams();
```

##### `create_team` / `createTeam`

Creates a new team.

**Parameters**: `name` (string, required).

**Returns**: `CreateTeamResponse` with `data: CreateTeamData`.

```typescript
// Node.js
const resp = await qn.admin.createTeam({ name: "Payments" });
```

##### `get_team` / `getTeam`

Fetches team detail including pending invites.

**Parameters**: `id` (i64, required).

**Returns**: `GetTeamResponse` with `data: TeamDetail`.

```typescript
// Node.js
const resp = await qn.admin.getTeam(42);
```

##### `delete_team` / `deleteTeam`

Deletes a team.

**Parameters**: `id` (i64, required).

**Returns**: `DeleteTeamResponse`.

```typescript
// Node.js
await qn.admin.deleteTeam(42);
```

##### `list_team_endpoints` / `listTeamEndpoints`

Lists endpoints accessible to a team.

**Parameters**: `id` (i64, required).

**Returns**: `ListTeamEndpointsResponse` with `data: TeamEndpoint[]`.

```typescript
// Node.js
const resp = await qn.admin.listTeamEndpoints(42);
```

##### `update_team_endpoints` / `updateTeamEndpoints`

Replaces the set of endpoints associated with a team. Pass an empty array to remove all.

**Parameters**: `id` (i64, required); body: `endpoint_ids` (string[], required).

**Returns**: `UpdateTeamEndpointsResponse`.

```typescript
// Node.js
await qn.admin.updateTeamEndpoints(42, { endpointIds: ["ep-123", "ep-456"] });
```

##### `invite_team_member` / `inviteTeamMember`

Invites a user to a team. Existing users only need `email`; new users require `full_name` and `role`.

**Parameters**: `id` (i64, required); body: `email` (string, required), `full_name` (string, optional), `role` (string, optional — `admin` | `viewer` | `billing`).

**Returns**: `InviteTeamMemberResponse`.

```typescript
// Node.js
await qn.admin.inviteTeamMember(42, { email: "alice@example.com", role: "viewer" });
```

##### `remove_team_member` / `removeTeamMember`

Removes a user from a team.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `RemoveTeamMemberResponse`.

```typescript
// Node.js
await qn.admin.removeTeamMember(42, 7);
```

##### `resend_team_invite` / `resendTeamInvite`

Re-sends a pending team invitation.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `ResendTeamInviteResponse`.

```typescript
// Node.js
await qn.admin.resendTeamInvite(42, 7);
```

#### Usage

All usage methods accept optional `start_time` and `end_time` Unix timestamps. Omit both for account-to-date totals.

##### `get_usage` / `getUsage`

Aggregate account usage for a time window.

**Returns**: `GetUsageResponse` with `data: UsageData` (`credits_used`, `credits_remaining`, `limit`, `overages`, `start_time`, `end_time`).

```typescript
// Node.js
const resp = await qn.admin.getUsage();
```

##### `get_usage_by_endpoint` / `getUsageByEndpoint`

Per-endpoint usage breakdown.

**Returns**: `GetUsageByEndpointResponse` with `data.endpoints: EndpointUsage[]`.

```typescript
// Node.js
const resp = await qn.admin.getUsageByEndpoint();
```

##### `get_usage_by_method` / `getUsageByMethod`

Per-RPC-method usage breakdown.

**Returns**: `GetUsageByMethodResponse` with `data.methods: MethodUsage[]`.

```typescript
// Node.js
const resp = await qn.admin.getUsageByMethod();
```

##### `get_usage_by_chain` / `getUsageByChain`

Per-chain usage breakdown.

**Returns**: `GetUsageByChainResponse` with `data.chains: ChainUsage[]`.

```typescript
// Node.js
const resp = await qn.admin.getUsageByChain();
```

##### `get_usage_by_tag` / `getUsageByTag`

Per-tag usage breakdown.

**Returns**: `GetUsageByTagResponse` with `data.tags: TagUsage[]`.

```typescript
// Node.js
const resp = await qn.admin.getUsageByTag();
```

#### Logs

##### `get_endpoint_logs` / `getEndpointLogs`

Fetches a page of request logs for an endpoint. Set `include_details=true` for full request/response payloads (truncated at 2 KB each).

**Parameters**: `id` (endpoint id, required); body: `from` (string timestamp, required), `to` (string timestamp, required), `include_details` (bool, optional), `limit` (i32, optional), `next_at` (string cursor, optional).

**Returns**: `GetEndpointLogsResponse` — `{ data: EndpointLog[], next_at?: string }`.

```typescript
// Node.js
const resp = await qn.admin.getEndpointLogs("ep-123", {
  from: "2026-04-01T00:00:00Z",
  to: "2026-04-02T00:00:00Z",
  limit: 100,
});
```

##### `get_log_details` / `getLogDetails`

Returns the full request/response payloads for a single log entry.

**Parameters**: `id` (endpoint id, required), `request_id` (log request uuid, required).

**Returns**: `GetLogDetailsResponse` with `data: LogDetails`.

```typescript
// Node.js
const resp = await qn.admin.getLogDetails("ep-123", "req-abc");
```

#### Endpoint Security

##### `get_endpoint_security` / `getEndpointSecurity`

Returns the full security configuration for an endpoint: tokens, JWTs, referrers, domain masks, IPs, request filters, and their per-feature toggles.

**Parameters**: `id` (string, required).

**Returns**: `GetEndpointSecurityResponse` with `data: EndpointSecurity`.

```typescript
// Node.js
const resp = await qn.admin.getEndpointSecurity("ep-123");
```

#### Security Options

##### `get_security_options` / `getSecurityOptions`

Returns the list of security features and their enabled state for an endpoint.

**Parameters**: `id` (string, required).

**Returns**: `GetSecurityOptionsResponse` with `data: SecurityOption[]`.

```typescript
// Node.js
const resp = await qn.admin.getSecurityOptions("ep-123");
```

##### `update_security_options` / `updateSecurityOptions`

Enables or disables individual security features. Each field accepts `"enabled"` or `"disabled"`.

**Parameters**: `id` (string, required); `options`: `SecurityOptionsUpdate` (`tokens`, `referrers`, `jwts`, `ips`, `domain_masks`, `hsts`, `cors`, `request_filters`, `ip_custom_header`).

**Returns**: `UpdateSecurityOptionsResponse` with updated `SecurityOption[]`.

```typescript
// Node.js
await qn.admin.updateSecurityOptions("ep-123", {
  options: { tokens: "enabled", jwts: "disabled" },
});
```

#### Tokens

##### `create_token` / `createToken`

Generates a new auth token on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.createToken("ep-123");
```

##### `delete_token` / `deleteToken`

Revokes a token on an endpoint.

**Parameters**: `id` (endpoint id, required), `token_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteToken("ep-123", "tok-1");
```

#### Referrers

##### `create_referrer` / `createReferrer`

Whitelists a referrer URL or domain on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `referrer` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.createReferrer("ep-123", { referrer: "example.com" });
```

##### `delete_referrer` / `deleteReferrer`

Removes a referrer from the whitelist.

**Parameters**: `id` (endpoint id, required), `referrer_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteReferrer("ep-123", "ref-1");
```

#### IPs

##### `create_ip` / `createIp`

Whitelists an IP address on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `ip` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.createIp("ep-123", { ip: "198.51.100.7" });
```

##### `delete_ip` / `deleteIp`

Removes an IP from the whitelist.

**Parameters**: `id` (endpoint id, required), `ip_id` (string, required).

**Returns**: `DeleteBoolResponse`.

```typescript
// Node.js
await qn.admin.deleteIp("ep-123", "ip-1");
```

#### Domain Masks

##### `create_domain_mask` / `createDomainMask`

Adds a custom domain mask to an endpoint.

**Parameters**: `id` (endpoint id, required); body: `domain_mask` (string, optional).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.createDomainMask("ep-123", { domainMask: "rpc.example.com" });
```

##### `delete_domain_mask` / `deleteDomainMask`

Removes a domain mask.

**Parameters**: `id` (endpoint id, required), `domain_mask_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteDomainMask("ep-123", "dm-1");
```

#### JWTs

##### `create_jwt` / `createJwt`

Configures JWT validation on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `public_key` (string, required), `kid` (string, required), `name` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.createJwt("ep-123", {
  publicKey: "-----BEGIN PUBLIC KEY-----\n...",
  kid: "key-1",
  name: "primary",
});
```

##### `delete_jwt` / `deleteJwt`

Removes a JWT configuration.

**Parameters**: `id` (endpoint id, required), `jwt_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteJwt("ep-123", "jwt-1");
```

#### Request Filters

Whitelist specific RPC methods on an endpoint. Requests for methods not on the list are blocked when the feature is enabled.

##### `create_request_filter` / `createRequestFilter`

**Parameters**: `id` (endpoint id, required); body: `method` (string[], required). Ruby's Hash key is `methods` (plural).

**Returns**: `CreateRequestFilterResponse` with `data.id`.

```typescript
// Node.js
const resp = await qn.admin.createRequestFilter("ep-123", {
  method: ["eth_blockNumber", "eth_getBalance"],
});
```

##### `update_request_filter` / `updateRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required); body: `method` (string[], optional). Ruby's Hash keys are `request_filter_id` and `methods` (plural).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.updateRequestFilter("ep-123", "f-1", { method: ["eth_call"] });
```

##### `delete_request_filter` / `deleteRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteRequestFilter("ep-123", "f-1");
```

#### Multichain

##### `enable_multichain` / `enableMultichain`

Enables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.enableMultichain("ep-123");
```

##### `disable_multichain` / `disableMultichain`

Disables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.disableMultichain("ep-123");
```

#### IP Custom Headers

##### `create_or_update_ip_custom_header` / `createOrUpdateIpCustomHeader`

Sets the custom header used to identify the client IP (e.g. when traffic is proxied).

**Parameters**: `id` (endpoint id, required); body: `header_name` (string, required).

**Returns**: `CreateOrUpdateIpCustomHeaderResponse` with `data.header_name`.

```typescript
// Node.js
await qn.admin.createOrUpdateIpCustomHeader("ep-123", { headerName: "X-Forwarded-For" });
```

##### `delete_ip_custom_header` / `deleteIpCustomHeader`

Removes the custom IP header configuration.

**Parameters**: `id` (endpoint id, required).

**Returns**: `DeleteBoolResponse`.

```typescript
// Node.js
await qn.admin.deleteIpCustomHeader("ep-123");
```

#### Method Rate Limits

##### `get_method_rate_limits` / `getMethodRateLimits`

Lists method-level rate limiters configured on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetMethodRateLimitsResponse` with `data.rate_limiters: MethodRateLimiter[]`.

```typescript
// Node.js
const resp = await qn.admin.getMethodRateLimits("ep-123");
```

##### `create_method_rate_limit` / `createMethodRateLimit`

Creates a new method-level rate limiter.

**Parameters**: `id` (endpoint id, required); body: `interval` (string, e.g. `"second"`), `methods` (string[]), `rate` (i32).

**Returns**: `CreateMethodRateLimitResponse` with `data: MethodRateLimiter`.

```typescript
// Node.js
const resp = await qn.admin.createMethodRateLimit("ep-123", {
  interval: "second",
  methods: ["eth_call"],
  rate: 10,
});
```

##### `update_method_rate_limit` / `updateMethodRateLimit`

Updates an existing rate limiter. Only provided fields change.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required); body: `methods` (string[], optional), `status` (`"enabled"` | `"disabled"`, optional), `rate` (i32, optional).

**Returns**: `UpdateMethodRateLimitResponse`.

```typescript
// Node.js
await qn.admin.updateMethodRateLimit("ep-123", "rl-1", { rate: 50 });
```

##### `delete_method_rate_limit` / `deleteMethodRateLimit`

Deletes a rate limiter.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteMethodRateLimit("ep-123", "rl-1");
```

#### Endpoint Rate Limits

##### `update_rate_limits` / `updateRateLimits`

Partial update of the endpoint-level RPS / RPM / RPD caps. Only buckets included in the request are modified — omitted buckets are left unchanged. Values are capped by the account's plan tier. Sends `PATCH`.

**Parameters**: `id` (endpoint id, required); `rate_limits`: `RateLimitSettings` (`rps`, `rpm`, `rpd`, all optional).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.updateRateLimits("ep-123", { rateLimits: { rps: 100, rpm: 5000 } });
```

##### `get_rate_limits` / `getRateLimits`

Returns the rate-limit rows currently enforced on the endpoint, each identifying its `bucket` (`"rps"` / `"rpm"` / `"rpd"`), `rateLimit`, and `source` (`"plan_default"` or `"user_override"`). User-set overrides expose an `id` you can pass to `deleteRateLimitOverride`.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetRateLimitsResponse` with `data.rateLimits: RateLimitEntry[]`.

```typescript
// Node.js
const resp = await qn.admin.getRateLimits("123");
for (const row of resp.data.rateLimits) {
  console.log(row.bucket, row.rateLimit, row.source, row.id);
}
```

##### `delete_rate_limit_override` / `deleteRateLimitOverride`

Deletes a user-set rate-limit override by UUID. Plan defaults are not deletable — passing a UUID that does not match a user-set override on the endpoint returns 404.

**Parameters**: `id` (endpoint id, required); `override_id` / `overrideId` (UUID returned by `getRateLimits`, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.admin.deleteRateLimitOverride("123", "ovr-uuid");
```

#### Endpoint URLs

##### `get_endpoint_urls` / `getEndpointUrls`

Returns the HTTP and WebSocket URLs for the endpoint without fetching the full endpoint record. For multichain endpoints, `multichain_urls` / `multichainUrls` is a per-network map of additional URLs; for single-chain endpoints it is `null`.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetEndpointUrlsResponse` with `data.httpUrl`, `data.wssUrl`, and `data.multichainUrls`.

```typescript
// Node.js
const resp = await qn.admin.getEndpointUrls("123");
console.log(resp.data.httpUrl);
if (resp.data.multichainUrls) {
  for (const [network, urls] of Object.entries(resp.data.multichainUrls)) {
    console.log(network, urls.httpUrl);
  }
}
```

#### Metrics

##### `get_endpoint_metrics` / `getEndpointMetrics`

Returns metric series for an endpoint over a time period.

**Parameters**: `id` (endpoint id, required); body: `period` (`"hour"` | `"day"` | `"week"` | `"month"`), `metric` (e.g. `"method_calls_over_time"`, `"response_status_breakdown"`).

**Returns**: `GetEndpointMetricsResponse` with `data: EndpointMetric[]`. Each `EndpointMetric` has a `tag: string[]` and a `data: [timestamp, value][]`. Single-axis series (e.g. `response_time_over_time` with a percentile) come back as a one-element tag like `["p95"]`; multi-axis series come back as `["network", "arbitrum-mainnet"]`.

```typescript
// Node.js
const resp = await qn.admin.getEndpointMetrics("ep-123", {
  period: "day",
  metric: "method_calls_over_time",
});
```

##### `get_account_metrics` / `getAccountMetrics`

Returns account-level metric series. Supports an optional `percentile` (e.g. `"p50"`, `"p95"`, `"p99"`) for latency metrics.

**Parameters**: `period` (required), `metric` (required), `percentile` (string, optional).

**Returns**: `GetAccountMetricsResponse` with `data: EndpointMetric[]`. See `getEndpointMetrics` above for the `tag: string[]` shape.

```typescript
// Node.js
const resp = await qn.admin.getAccountMetrics({
  period: "day",
  metric: "credits_over_time",
});
```

#### Chains

##### `list_chains` / `listChains`

Lists the blockchains supported by Quicknode along with their networks.

**Parameters**: none.

**Returns**: `ListChainsResponse` with `data: Chain[]`.

```typescript
// Node.js
const resp = await qn.admin.listChains();
```

#### Account

##### `account_info` / `accountInfo`

Returns details about the account, including its id, name, creation timestamp, billing version, and current subscription.

**Parameters**: none.

**Returns**: `AccountInfoResponse` with `data: AccountInfo` (including a nested `subscription: AccountSubscription`).

```typescript
// Node.js
const resp = await qn.admin.accountInfo();
```

##### `get_api_credits` / `getApiCredits`

Returns the per-method API credit costs for a chain, identified by its slug (the same slugs returned by `list_chains`, e.g. `ethereum`). An unknown chain slug rejects with `ApiError` (status 404).

**Parameters**: `chain` (string, required) — the chain slug.

**Returns**: `GetApiCreditsResponse` with `data: ApiCredit[]`, where each `ApiCredit` has `method` and `credits`.

```typescript
// Node.js
const resp = await qn.admin.getApiCredits("ethereum");
```

#### Billing

##### `list_invoices` / `listInvoices`

Lists invoices on the account.

**Parameters**: none.

**Returns**: `ListInvoicesResponse` with `data.invoices: Invoice[]`.

```typescript
// Node.js
const resp = await qn.admin.listInvoices();
```

##### `list_payments` / `listPayments`

Lists payments on the account.

**Parameters**: none.

**Returns**: `ListPaymentsResponse` with `data.payments: Payment[]`.

```typescript
// Node.js
const resp = await qn.admin.listPayments();
```

#### Bulk Operations

##### `bulk_update_endpoint_status` / `bulkUpdateEndpointStatus`

Activates or pauses many endpoints at once.

**Parameters**: `ids` (string[], required), `status` (`"active"` | `"paused"`, required).

**Returns**: `BulkUpdateEndpointStatusResponse` with per-endpoint `results`.

```typescript
// Node.js
const resp = await qn.admin.bulkUpdateEndpointStatus({
  ids: ["ep-1", "ep-2"],
  status: "paused",
});
```

##### `bulk_add_tag` / `bulkAddTag`

Applies a tag (created if missing) to many endpoints at once.

**Parameters**: `ids` (string[], required), `label` (string, required).

**Returns**: `BulkAddTagResponse`.

```typescript
// Node.js
const resp = await qn.admin.bulkAddTag({ ids: ["ep-1", "ep-2"], label: "prod" });
```

##### `bulk_remove_tag` / `bulkRemoveTag`

Removes a tag from many endpoints at once.

**Parameters**: `ids` (string[], required), `tag_id` (i32, required).

**Returns**: `BulkRemoveTagResponse`.

```typescript
// Node.js
const resp = await qn.admin.bulkRemoveTag({ ids: ["ep-1", "ep-2"], tagId: 42 });
```

#### Account Tags

##### `list_tags` / `listTags`

Lists every tag on the account along with usage counts.

**Parameters**: none.

**Returns**: `ListTagsResponse` with `data.tags: AccountTag[]`.

```typescript
// Node.js
const resp = await qn.admin.listTags();
```

##### `rename_tag` / `renameTag`

Renames an account-level tag.

**Parameters**: `tag_id` (i32, required); body: `label` (string, required).

**Returns**: `RenameTagResponse` with updated `AccountTag`.

```typescript
// Node.js
const resp = await qn.admin.renameTag(42, { label: "staging" });
```

##### `delete_account_tag` / `deleteAccountTag`

Deletes a tag from the account. The tag must first be removed from any endpoints using it.

**Parameters**: `id` (i32, required).

**Returns**: `DeleteAccountTagResponse`.

```typescript
// Node.js
await qn.admin.deleteAccountTag(42);
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

```typescript
// Node.js
import { StreamDataset, StreamRegion, StreamStatus } from "quicknode-sdk";

const stream = await qn.streams.createStream({
  name: "My Stream",
  network: "ethereum-mainnet",
  dataset: StreamDataset.Block,
  region: StreamRegion.UsaEast,
  startRange: 24691804,
  endRange: 24691904,
  destinationAttributes: {
    destination: "webhook",
    attributes: {
      url: "https://webhook.site/...",
      maxRetry: 3,
      retryIntervalSec: 1,
      postTimeoutSec: 10,
      compression: "none",
    },
  },
  plan: "growth_plan",
  thresholdFetchBuffer: 1000,
  status: StreamStatus.Active,
});
```

##### `list_streams` / `listStreams`

Paginated list of streams on the account.

**Parameters** (all optional): `offset` (i64), `limit` (i64), `order_by` (string), `order_direction` (`"asc"` | `"desc"`), `stream_type` (string).

**Returns**: `ListStreamsResponse` with `data: Stream[]` and `page_info`.

```typescript
// Node.js
const resp = await qn.streams.listStreams();
```

##### `get_stream` / `getStream`

Fetches one stream by id.

**Parameters**: `id` (string, required).

**Returns**: `Stream`.

```typescript
// Node.js
const stream = await qn.streams.getStream("stream-id");
```

##### `update_stream` / `updateStream`

Partially updates a stream. Omitted fields are left unchanged.

**Parameters**: `id` (string, required); body: any field from `CreateStreamParams` (all optional).

**Returns**: updated `Stream`.

```typescript
// Node.js
const stream = await qn.streams.updateStream("stream-id", { name: "Renamed" });
```

##### `delete_stream` / `deleteStream`

Deletes one stream by id.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.streams.deleteStream("stream-id");
```

##### `delete_all_streams` / `deleteAllStreams`

Deletes every stream on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```typescript
// Node.js
await qn.streams.deleteAllStreams();
```

##### `activate_stream` / `activateStream`

Resumes delivery on a stream from its current position.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.streams.activateStream("stream-id");
```

##### `pause_stream` / `pauseStream`

Halts delivery on a stream.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.streams.pauseStream("stream-id");
```

##### `test_filter` / `testFilter`

Runs a filter function against a block so it can be validated before being attached to a live stream.

**Parameters**: `network` (string, required), `dataset` (`StreamDataset`, required), `block` (string, required), `filter_function` (string, optional), `filter_language` (`FilterLanguage`, optional), `address_book_config` (optional).

**Returns**: `TestFilterResponse` with `result` and `logs`.

```typescript
// Node.js
import { StreamDataset } from "quicknode-sdk";

const resp = await qn.streams.testFilter({
  network: "ethereum-mainnet",
  dataset: StreamDataset.Block,
  block: "17811625",
});
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled (active) streams, optionally filtered by type.

**Parameters**: `stream_type` (string, optional).

**Returns**: `EnabledCountResponse` with `total`.

```typescript
// Node.js
const resp = await qn.streams.getEnabledCount();
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

`TemplateArgs` carries the arguments. Each factory method accepts either the inline template (with values) or the `*ByListTemplate` (with a pre-created list name):

| Factory | Inline template (fields) | ByList template (fields) |
|---|---|---|
| `evmWalletFilter` | `EvmWalletFilterTemplate { wallets: string[] }` | `EvmWalletFilterByListTemplate { walletsListName: string }` |
| `evmContractEvents` | `EvmContractEventsTemplate { contracts: string[], eventHashes: string[] }` | `EvmContractEventsByListTemplate { contractsListName: string, eventHashesListName?: string }` |
| `evmAbiFilter` | `EvmAbiFilterTemplate { abi: string, contracts: string[] }` | `EvmAbiFilterByListTemplate { abiJson: string, contractsListName?: string }` |
| `solanaWalletFilter` | `SolanaWalletFilterTemplate { accounts: string[] }` | `SolanaWalletFilterByListTemplate { accountsListName: string }` |
| `bitcoinWalletFilter` | `BitcoinWalletFilterTemplate { wallets: string[] }` | `BitcoinWalletFilterByListTemplate { walletsListName: string }` |
| `xrplWalletFilter` | `XrplWalletFilterTemplate { wallets: string[] }` | `XrplWalletFilterByListTemplate { walletsListName: string }` |
| `hyperliquidWalletEventsFilter` | `HyperliquidWalletEventsFilterTemplate { wallets: string[] }` | `HyperliquidWalletEventsFilterByListTemplate { walletsListName: string }` |
| `stellarWalletTransactionsFilter` | `StellarWalletTransactionsFilterTemplate { wallets: string[] }` | `StellarWalletTransactionsFilterByListTemplate { walletsListName: string }` |

`WebhookDestinationAttributes`: `url` (required), `compression` (required — `"none"` | `"gzip"`), `security_token` (optional — auto-generated if omitted).

`WebhookStartFrom`: `Last` (resume from last delivered block) or `Latest` (start from newest).

In Ruby, `template_args` is passed as a JSON string under the key `template_args_json`; destination is passed as a JSON string under `destination_attributes_json`.

#### Webhooks methods

##### `list_webhooks` / `listWebhooks`

Paginated list of webhooks.

**Parameters** (all optional): `limit` (i64), `offset` (i64).

**Returns**: `ListWebhooksResponse` with `data: Webhook[]` and `pageInfo: WebhookPageInfo { limit, offset, total }`.

```typescript
// Node.js
const resp = await qn.webhooks.listWebhooks();
```

##### `get_webhook` / `getWebhook`

Fetches a webhook by id.

**Parameters**: `id` (string, required).

**Returns**: `Webhook`.

```typescript
// Node.js
const webhook = await qn.webhooks.getWebhook("wh-1");
```

##### `create_webhook_from_template` / `createWebhookFromTemplate`

Creates a webhook from a predefined filter template.

**Parameters**: `name` (required), `network` (required), `destination_attributes` (`WebhookDestinationAttributes`, required), `template_args` (required — use the `TemplateArgs` enum variant for the chosen template), `notification_email` (optional).

**Returns**: `Webhook`.

```typescript
// Node.js
import { TemplateArgs } from "quicknode-sdk";

const webhook = await qn.webhooks.createWebhookFromTemplate({
  name: "Wallet Webhook",
  network: "ethereum-mainnet",
  destinationAttributes: { url: "https://webhook.site/...", compression: "none" },
  templateArgs: TemplateArgs.evmWalletFilter({
    wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
  }),
});
```

##### `update_webhook` / `updateWebhook`

Partially updates a webhook's name, notification email, and/or destination. If `destination_attributes` is supplied without `security_token`, a new token is generated automatically.

**Parameters**: `id` (required); body — all optional: `name`, `notification_email`, `destination_attributes`. In Ruby, `destination_attributes` is passed as a JSON string under the key `destination_attributes_json`.

**Returns**: updated `Webhook`.

```typescript
// Node.js
const webhook = await qn.webhooks.updateWebhook("wh-1", { name: "Renamed Webhook" });
```

##### `update_webhook_template` / `updateWebhookTemplate`

Updates the template args (and optionally name, email, destination) on an existing template-backed webhook.

**Parameters**: `webhook_id` (required), `template_args` (required); optional: `name`, `notification_email`, `destination_attributes`.

**Returns**: updated `Webhook`.

```typescript
// Node.js
const webhook = await qn.webhooks.updateWebhookTemplate("wh-1", {
  templateArgs: TemplateArgs.evmWalletFilter({ wallets: ["0xnewwallet"] }),
});
```

##### `delete_webhook` / `deleteWebhook`

Deletes a webhook.

**Parameters**: `id` (required).

**Returns**: nothing.

```typescript
// Node.js
await qn.webhooks.deleteWebhook("wh-1");
```

##### `delete_all_webhooks` / `deleteAllWebhooks`

Deletes every webhook on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```typescript
// Node.js
await qn.webhooks.deleteAllWebhooks();
```

##### `pause_webhook` / `pauseWebhook`

Pauses a webhook so it stops delivering events.

**Parameters**: `id` (required).

**Returns**: nothing.

```typescript
// Node.js
await qn.webhooks.pauseWebhook("wh-1");
```

##### `activate_webhook` / `activateWebhook`

Activates a paused or new webhook so it resumes delivering events. `start_from` determines where processing resumes.

**Parameters**: `id` (required), `start_from` (`WebhookStartFrom`, required — `Last` or `Latest`).

**Returns**: nothing.

```typescript
// Node.js
import { WebhookStartFrom } from "quicknode-sdk";

await qn.webhooks.activateWebhook("wh-1", { startFrom: WebhookStartFrom.Latest });
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled webhooks.

**Parameters**: none.

**Returns**: `WebhookEnabledCountResponse` with `total`.

```typescript
// Node.js
const resp = await qn.webhooks.getEnabledCount();
```

---

### KV Store Client

Accessed as `qn.kvstore`. Provides two primitives — **sets** (single string values under a key) and **lists** (ordered collections of strings under a key). Backed by `https://api.quicknode.com/kv/rest/v1/`.

#### Sets

##### `create_set` / `createSet`

Stores a single string value under a key.

**Parameters**: `key` (string, required), `value` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.createSet({ key: "my-key", value: "hello" });
```

##### `get_sets` / `getSets`

Paginated page of key/value entries.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetSetsResponse` — `{ data: KvSetEntry[], cursor: string }`.

```typescript
// Node.js
const resp = await qn.kvstore.getSets();
```

##### `get_set` / `getSet`

Returns the value stored under a key.

**Parameters**: `key` (string, required).

**Returns**: `GetSetResponse` with `value`.

```typescript
// Node.js
const resp = await qn.kvstore.getSet("my-key");
```

##### `bulk_sets` / `bulkSets`

Adds and/or deletes multiple sets in a single request.

**Parameters** (at least one required): `add_sets` (map<string,string>, optional), `delete_sets` (string[], optional).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.bulkSets({
  addSets: { k1: "v1" },
  deleteSets: ["old-key"],
});
```

##### `delete_set` / `deleteSet`

Deletes a single set.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.deleteSet("my-key");
```

#### Lists

##### `create_list` / `createList`

Creates a list under a key, seeded with the initial items.

**Parameters**: `key` (string, required), `items` (string[], required).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.createList({ key: "my-list", items: ["0xabc", "0xdef"] });
```

##### `get_lists` / `getLists`

Paginated page of list keys.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetListsResponse` — `{ data: { keys: string[] }, cursor: string }`.

```typescript
// Node.js
const resp = await qn.kvstore.getLists();
```

##### `get_list` / `getList`

Paginated page of items for a specific list.

**Parameters**: `key` (string, required); optional `limit` (i64), `cursor` (string).

**Returns**: `GetListResponse` — `{ data: { items: string[] }, cursor: string }`.

```typescript
// Node.js
const resp = await qn.kvstore.getList("my-list");
```

##### `update_list` / `updateList`

Adds and/or removes items in a single operation.

**Parameters**: `key` (string, required); optional: `add_items` (string[]), `remove_items` (string[]).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.updateList("my-list", {
  addItems: ["0x456"],
  removeItems: ["0xabc"],
});
```

##### `add_list_item` / `addListItem`

Appends a single item to a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.addListItem("my-list", { item: "0x123" });
```

##### `list_contains_item` / `listContainsItem`

Checks whether a list contains a specific item.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: `ListContainsItemResponse` with `exists: bool`.

```typescript
// Node.js
const resp = await qn.kvstore.listContainsItem("my-list", "0x123");
```

##### `delete_list_item` / `deleteListItem`

Removes a single item from a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.deleteListItem("my-list", "0x123");
```

##### `delete_list` / `deleteList`

Deletes a list and all of its items.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```typescript
// Node.js
await qn.kvstore.deleteList("my-list");
```

---

### SQL Client

Accessed as `qn.sql`. Runs SQL queries against indexed blockchain data and fetches the database schema. Backed by `https://api.quicknode.com/sql/rest/v1/`.

##### `query`

Executes a SQL query against a cluster and returns the result set. Paginate by writing `LIMIT`/`OFFSET` into the SQL.

**Parameters**: `query` (string, required), `cluster_id` / `clusterId` (string, required).

**Returns**: a query result — `meta` (column metadata, each with `name` and `type`), `data` (rows as objects keyed by column name), `rows`, `rows_before_limit_at_least` / `rowsBeforeLimitAtLeast`, `statistics` (`elapsed`, `rows_read`/`rowsRead`, `bytes_read`/`bytesRead`), and `credits`.

```typescript
// Node.js
const resp = await qn.sql.query(
  "SELECT action_type, user FROM hyperliquid_system_actions ORDER BY block_time DESC LIMIT 100",
  "hyperliquid-core-mainnet",
);
console.log(resp.rows, resp.data[0]);
```

##### `get_schema` / `getSchema`

Fetches the database schema for a cluster: table names, columns, types, sort keys, and partition strategies.

**Parameters**: `cluster_id` / `clusterId` (string, required).

**Returns**: a chain schema — `chain`, `cluster_id` / `clusterId`, and `tables` (each with `name`, `engine`, `total_rows` / `totalRows`, `partition_key` / `partitionKey`, `sorting_key` / `sortingKey`, and `columns` of `{ name, type }`).

```typescript
// Node.js
const schema = await qn.sql.getSchema("hyperliquid-core-mainnet");
console.log(schema.tables.length);
```

---

### RPC & Tooling Access

Tooling Access provisions a single multichain, read-only endpoint per account and
mints short-lived session JWTs. `qn.rpc` makes JSON-RPC calls directly against that
endpoint, minting and refreshing the JWT automatically — no endpoint URL or token to
manage.

Tooling Access must be enabled once (admin role + eligible plan). The control-plane
methods live on `qn.admin`:

```typescript
// Node.js
const status = await qn.admin.toolingAccessStatus();
if (!status.enabled) {
  await qn.admin.enableToolingAccess(); // idempotent; admin role required
}

// Make on-chain calls. params defaults to []; pass an array (positional) or object.
const blockNumber = await qn.rpc.call("eth_blockNumber");
const balance = await qn.rpc.call("eth_getBalance", ["0xabc...", "latest"]);

// Multichain: select a network by its multichain_urls key. Seed the map first
// (from admin.getEndpointUrls), then pass the network as the 3rd arg.
const urls = await qn.admin.getEndpointUrls(endpointId);
const map = Object.fromEntries(
  Object.entries(urls.multichainUrls ?? {}).map(([k, v]) => [k, v.httpUrl]),
);
qn.rpc.setNetworks(map);
const slot = await qn.rpc.call("getSlot", [], "solana-mainnet");

// Custom endpoint URL: send to a fully-formed HTTP URL, bypassing Tooling Access
// and the JWT (no Authorization header). Per-call via the 4th arg, or client-wide
// via `new RpcConfig({ endpointUrl })`. endpointUrl and network are mutually
// exclusive (a custom URL is not multichain-routed).
const block = await qn.rpc.call("eth_blockNumber", [], undefined, "https://my-endpoint.example/rpc");

// A JSON-RPC error member is thrown as RpcError (with .code and .message).
import { RpcError } from "@quicknode/sdk";
try {
  await qn.rpc.call("eth_getBalance", ["bad"]);
} catch (e) {
  if (e instanceof RpcError) console.error(e.code, e.message);
}
```

A host that persists across processes can snapshot the cached token with
`qn.rpc.currentToken()` and re-seed it via `RpcConfig.seed` on the next construction;
`refreshMarginSecs` (default 60) tunes how early the token is refreshed. Set
`RpcConfig.endpointUrl` to route every call to a custom HTTP URL by default (no
JWT minted); a per-call `endpointUrl` overrides it.

## Crypto-micropayment lane (`rpc.call`)

Pay per RPC request with a stablecoin instead of a provisioned account + API key,
against Quicknode's `x402.quicknode.com` and `mpp.quicknode.com` gateways. Configure
it by setting `payment` on the RPC config; the SDK runs the `402` → sign → resend
handshake for you. An API key is **not** required for this lane — build a keyless SDK.

There are four payment paths. Two pay per request; two amortize one signature over many
calls.

| Path | Entry point | Gateway | Signs |
|---|---|---|---|
| Per-request x402 | `call` / `callWithReceipt` with `scheme: "x402"` | x402 | once per call |
| Per-request MPP charge | `call` / `callWithReceipt` with `scheme: "mpp"` | mpp | once per call |
| [x402 credit drawdown](#x402-credit-drawdown-authenticate-once-then-draw-one-credit-per-call) | `gatewayAuthenticate` → `gatewayDrawdownCall` | x402 | once per session |
| [MPP payment channel](#mpp-payment-channel-deposit-once-then-vouchers) | `mppOpen` → `mppSessionCall` | mpp | once per channel |

The signer construction is derived from the scheme and pay network, never stated directly:
**x402/EVM** signs an EIP-712 `TransferWithAuthorization`, **x402/Solana** an SPL
`TransferChecked` in a v0 tx (the gateway sponsors gas), and **MPP/Tempo** a native Tempo
transaction.

`scheme` selects the gateway for `call` only. The `gateway*` drawdown methods always use
the x402 gateway and the `mpp*` channel methods always use the MPP gateway, whatever
`scheme` is set to.

`PaymentConfig` fields:

| Field | Meaning |
|---|---|
| `scheme` | `"x402"` (pay-per-request) or `"mpp"` (MPP charge; `"mpp-charge"` is accepted too) |
| `key` | raw private key — EVM/Tempo: hex; Solana: base58 64-byte secret |
| `payNetwork` | CAIP-2 pay network, e.g. `eip155:84532`, `solana:5eykt4…` |
| `asset` | token address/mint to pay in (matches the offered menu entry) |
| `maxAmount` | **required** spend ceiling in integer base units of `asset` |
| `svmRpcUrl` | optional Solana RPC for x402/Solana payment-build reads (mint + blockhash) |
| `baseUrlOverride` | optional gateway base (testing) |

`network` on the call is the **query** chain (gateway path slug), independent of the
pay network. Use `callWithReceipt` to also get the settlement receipt (`reference` =
settlement tx hash) — populated on the MPP lane, `null` for x402.

**Things to know:**

- **Do not log your own `PaymentConfig`** — the `key` field is readable. The SDK
  never prints it in its own errors, but `console.log(config)` will show it.
- **`maxAmount` is integer base units of the selected asset.** The SDK skips any offered
  entry above it and refuses to sign one — a guard against an overcharging gateway.
- **`PaymentIndeterminateError` means the paid request was sent but the response was lost.**
  You MAY have been charged — do **not** blindly retry.
- **x402/Solana: one payment per call.** Building a payment reads the mint and a recent
  blockhash from a Solana RPC. The default is a public RPC that **rate-limits
  aggressively** — set `svmRpcUrl` to your own endpoint at any volume.

```typescript
import { QuicknodeSdk } from "@quicknode/sdk";

const qn = new QuicknodeSdk({
  rpc: {
    payment: {
      scheme: "x402",
      key: process.env.QN_PAYMENT_KEY!,
      payNetwork: "eip155:84532",
      asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      maxAmount: "10000",
    },
  },
});
const { result, paymentReceipt } = await qn.rpc.callWithReceipt("eth_blockNumber", [], "base-sepolia");
console.log(result, paymentReceipt);
```

### Wallet generation

`generatePaymentWallet("evm")` creates a fresh keypair offline — no network call, no funds — for
`"evm"`, `"svm"`, or `"tempo"`. The private key is returned **exactly once**, at
generation; nothing in the SDK stores or re-derives it, so persist it immediately.

```typescript
import { generatePaymentWallet } from "@quicknode/sdk";

const wallet = generatePaymentWallet("evm");
console.log("fund this address:", wallet.address);
// wallet.key is returned exactly once — persist it now.
```

### x402 credit drawdown (authenticate once, then draw one credit per call)

Cheaper per call than paying per request: one SIWE or SIWS signature mints a session JWT, then
each call draws a single credit from the account balance instead of signing a fresh
settlement. Minting the session is free and moves no funds, so a host can re-authenticate
transparently. Persist it between processes.

Fund the payment wallet out of band — the testnet faucet below, or by sending funds to
`paymentAddress()` directly. Credits are provisioned against the account gateway-side.

EVM payment networks use SIWE. Solana payment networks use SIWS with an Ed25519
signature encoded as Base58.

| Method | Cost | Returns |
|---|---|---|
| `paymentAddress()` | free, offline | the wallet address derived from the key |
| `gatewayAuthenticate()` | free | `GatewaySession { token, expUnix, accountId }` |
| `gatewayCredits(session)` | free | `CreditBalance { accountId, credits }` |
| `gatewayDrip(session)` | free (testnet) | `DripReceipt` with `transferId` or `transactionHash` |
| `gatewayDrawdownCall(method, session, network, params?)` | 1 credit | the JSON-RPC `result` |

```typescript
const session = await qn.rpc.gatewayAuthenticate();
const balance = await qn.rpc.gatewayCredits(session);
console.log("credits:", balance.credits);
const result = await qn.rpc.gatewayDrawdownCall("eth_blockNumber", session, "base-sepolia");
```

A `token_expired` surfaces as an `ApiError` with status 401/403; re-authenticate and retry
that call.

#### Testnet faucet

`gatewayDrip` requests testnet tokens for the payment **wallet**. Circle Gateway-backed
networks return `transferId` because settlement is asynchronous. Direct-transfer
networks such as Arc Testnet return `transactionHash`. The response is not a credit
balance; call `gatewayCredits` separately.

### MPP payment channel (deposit once, then vouchers)

Open a payment channel by depositing into the escrow, then authorize each call with a
cumulative voucher — one `ecrecover` server-side, no on-chain transaction per call.

| Method | Cost | Returns |
|---|---|---|
| `mppOpen(deposit)` | **moves funds** | `ChannelState` — persist it |
| `mppTopUp(channel, additionalDeposit)` | **moves funds** | the updated channel state |
| `mppStatus(channel)` | **1 request unit** | `ChannelStatus { channelId, acceptedCumulative, spent }` |
| `mppSessionCall(method, network, channel, newCumulative, params?)` | 1 request unit | the JSON-RPC `result` |
| `mppClose(channel)` | settles on-chain | nothing — refunds the unused deposit |

```typescript
const channel = await qn.rpc.mppOpen("1000000");   // persist this object
const newTotal = (BigInt(channel.cumulativeSpent) + BigInt(channel.perCall)).toString();
const result = await qn.rpc.mppSessionCall(
  "eth_blockNumber", "base-sepolia", channel, newTotal,
);
// On success, store newTotal as the channel's cumulativeSpent.
```

**Things to know:**

- **Persist the channel state.** The gateway exposes no read-only channel endpoint, so a
  lost local record means opening (and funding) a new channel.
- **`mppStatus` is not free.** The gateway prices every session POST as a chargeable
  request and computes the balance from the *new* spend a voucher authorizes, so the
  probe advances `cumulativeSpent` by `perCall` exactly like a call. Re-persist the
  advanced total. It raises `PaymentUnsupportedError` before any network I/O when the
  channel has no room left for the probe.
- **The lifecycle takes no query network.** A channel is scoped by the configured pay
  network and asset, so one channel funds calls to every supported network. Only
  `mppSessionCall` takes a network, because it routes an RPC method.
- **Amounts are decimal strings, not numbers.** They are `u128` in the core; a JS `number` is an f64 that loses precision above 2^53, so pass and store them as strings.
- **Advance `cumulativeSpent` only after a success.** A voucher authorizes the running total
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

Class names: Importable from `@quicknode/sdk`: `QuicknodeError`, `ConfigError`, `HttpError`, `TimeoutError`, `ConnectionError`, `ApiError`, `DecodeError`, `RpcError`, `PaymentError`, `PaymentUnsupportedError`, `PaymentRejectedError`, `PaymentIndeterminateError`. All extend `Error`.

```typescript
// Node.js
import { ApiError, TimeoutError } from "@quicknode/sdk";
try {
  await qn.admin.showEndpoint("missing");
} catch (e) {
  if (e instanceof ApiError && e.status === 404) console.error("not found:", e.body);
  else if (e instanceof TimeoutError) console.error("timed out");
  else throw e;
}
```

## License

MIT
