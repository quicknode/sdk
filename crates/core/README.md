# quicknode-sdk (Rust)

The core Rust crate for the Quicknode SDK.

This is one of four language bindings published from the same Rust core. See the [project README](https://github.com/quicknode/sdk/blob/main/README.md) for the polyglot overview, development setup, and release process.

> **Pre-1.0**: While on `0.x`, releases may contain breaking changes. Check the [release notes](https://github.com/quicknode/sdk/releases) before upgrading.

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
- [Error Handling](#error-handling)
- [License](#license)

## Installation

`cargo add quicknode-sdk`

### Optional features — the crypto-micropayment lane

The pay-per-request `rpc.call` lane is feature-gated so you only pay its
dependency/build cost when you use it:

- `payments` — x402/EVM (EIP-712).
- `payments-svm` — adds x402/Solana (ed25519 + hand-rolled SPL).
- `payments-tempo` — adds MPP/Tempo (native Tempo tx). **Requires Rust ≥ 1.93**
  (pulls `tempo-primitives`).

```toml
quicknode-sdk = { version = "0.7", features = ["payments", "payments-svm", "payments-tempo"] }
```

The Python, Node, and Ruby packages ship precompiled with all payment features
on, so those consumers get the lane out of the box (and pay its cost regardless).

## Quick Start

Construct the SDK once, then reach into the five sub-clients (`admin`, `streams`, `webhooks`, `kvstore`, `sql`). Subsequent API Reference snippets assume you have a `qn` handle from one of these blocks.

```rust
// Rust
use quicknode_sdk::{QuicknodeSdk, SdkFullConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let qn = QuicknodeSdk::from_env()?;
    let resp = qn.admin.get_endpoints(&Default::default()).await?;
    println!("{} endpoints", resp.data.len());
    Ok(())
}
```

## Configuration

There are two ways to configure the SDK.

### Option A — Pass config directly

```rust
// Rust
let qn = QuicknodeSdk::new(&SdkFullConfig::builder().api_key("your-key").build())?;
```

`api_key` is optional here: the [crypto-micropayment lane](#crypto-micropayment-lane-rpccall)
pays per request instead, so `SdkFullConfig::keyless()` builds a usable SDK with no key.
Every other client still needs one. `from_env()` always requires `QN_SDK__API_KEY`.

### Option B — Load from environment (`from_env()`)

```rust
// Rust
let qn = QuicknodeSdk::from_env()?;
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

```rust
use std::collections::HashMap;
use quicknode_sdk::{HttpConfig, QuicknodeSdk, SdkFullConfig};

let mut headers = HashMap::new();
headers.insert("X-Correlation-Id".to_string(), "abc-123".to_string());
headers.insert("User-Agent".to_string(), "my-app/1.0".to_string()); // overrides SDK default

let qn = QuicknodeSdk::new(
    &SdkFullConfig::builder()
        .api_key("your-key")
        .http(HttpConfig { headers: Some(headers), ..Default::default() })
        .build(),
)?;
```

## Platform Support

`quicknode-sdk` is a pure-Rust source crate — it builds wherever `rustc` and `reqwest` are supported. It is regularly tested on Linux (glibc) and macOS (Apple Silicon). Windows is not tested.

If you are using one of the language bindings (`quicknode-sdk` on PyPI, `@quicknode/sdk` on npm, `quicknode_sdk` on RubyGems), see that package's README for the precompiled-binary platform matrix.

## API Reference

Snippets assume `qn` was already constructed via the Quick Start. Optional parameters are skipped unless showing one is needed to illustrate usage.

### Language conventions

- Methods are `async` and return `Result<T, SdkError>`. Request structs use the [`bon`](https://docs.rs/bon) builder pattern via `::builder()`.

---

### Admin Client

Accessed as `qn.admin`. Manages endpoints, tags, teams, billing, usage, metrics, security, and rate limits. Backed by `https://api.quicknode.com/v0/`.

#### Endpoints

##### `get_endpoints` / `getEndpoints`

Returns a paginated list of endpoints on the account with optional search, filters (networks, statuses, labels, tags, dedicated, flat-rate), sorting, and pagination.

**Parameters** (all optional): `limit` (i32), `offset` (i32), `search` (string), `sort_by` (string), `sort_direction` (`"asc"` | `"desc"`), `networks` (string[]), `statuses` (string[]), `labels` (string[]), `dedicated` (bool), `is_flat_rate` (bool), `tag_ids` (i32[]), `tag_labels` (string[]).

**Returns**: `GetEndpointsResponse` — `{ data: Endpoint[], pagination?: Pagination }`.

```rust
// Rust
let params = GetEndpointsRequest::builder()
    .limit(20)
    .sort_by("created_at".to_string())
    .sort_direction("desc".to_string())
    .build();
let resp = qn.admin.get_endpoints(&params).await?;
```

##### `create_endpoint` / `createEndpoint`

Creates a new endpoint for the given blockchain and network.

**Parameters**: `chain` (string, optional), `network` (string, optional).

**Returns**: `CreateEndpointResponse` with `data: SingleEndpoint`.

```rust
// Rust
let params = CreateEndpointRequest::builder()
    .chain("ethereum".to_string())
    .network("mainnet".to_string())
    .build();
let resp = qn.admin.create_endpoint(&params).await?;
```

##### `show_endpoint` / `showEndpoint`

Fetches a single endpoint by id, including its full security configuration and rate limits.

**Parameters**: `id` (string, required).

**Returns**: `ShowEndpointResponse` with `data: SingleEndpoint`.

```rust
// Rust
let resp = qn.admin.show_endpoint("ep-123").await?;
```

##### `update_endpoint` / `updateEndpoint`

Updates editable fields on an endpoint. Currently supports `label`.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```rust
// Rust
let params = UpdateEndpointRequest::builder().label("my label".to_string()).build();
qn.admin.update_endpoint("ep-123", &params).await?;
```

##### `archive_endpoint` / `archiveEndpoint`

Archives an endpoint. The HTTP verb is `DELETE` but the effect is archival, not permanent deletion.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.archive_endpoint("ep-123").await?;
```

##### `update_endpoint_status` / `updateEndpointStatus`

Pauses or unpauses an endpoint.

**Parameters**: `id` (string, required); body: `status` (string, required — `"active"` or `"paused"`).

**Returns**: `UpdateEndpointStatusResponse`.

```rust
// Rust
let params = UpdateEndpointStatusRequest::builder().status("paused".to_string()).build();
qn.admin.update_endpoint_status("ep-123", &params).await?;
```

#### Endpoint Tags

Per-endpoint tag add/remove. For account-wide tag management see [Account Tags](#account-tags).

##### `create_tag` / `createTag`

Tags an endpoint with the given label. Creates the tag on the account if it does not exist.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```rust
// Rust
let params = CreateTagRequest::builder().label("prod".to_string()).build();
qn.admin.create_tag("ep-123", &params).await?;
```

##### `delete_tag` / `deleteTag`

Removes a tag from a specific endpoint.

**Parameters**: `id` (endpoint id, string, required), `tag_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_tag("ep-123", "42").await?;
```

#### Teams

##### `list_teams` / `listTeams`

Lists all teams on the account.

**Parameters**: none.

**Returns**: `ListTeamsResponse` with `data: TeamSummary[]`.

```rust
// Rust
let resp = qn.admin.list_teams().await?;
```

##### `create_team` / `createTeam`

Creates a new team.

**Parameters**: `name` (string, required).

**Returns**: `CreateTeamResponse` with `data: CreateTeamData`.

```rust
// Rust
let params = CreateTeamRequest::builder().name("Payments".to_string()).build();
let resp = qn.admin.create_team(&params).await?;
```

##### `get_team` / `getTeam`

Fetches team detail including pending invites.

**Parameters**: `id` (i64, required).

**Returns**: `GetTeamResponse` with `data: TeamDetail`.

```rust
// Rust
let resp = qn.admin.get_team(42).await?;
```

##### `delete_team` / `deleteTeam`

Deletes a team.

**Parameters**: `id` (i64, required).

**Returns**: `DeleteTeamResponse`.

```rust
// Rust
qn.admin.delete_team(42).await?;
```

##### `list_team_endpoints` / `listTeamEndpoints`

Lists endpoints accessible to a team.

**Parameters**: `id` (i64, required).

**Returns**: `ListTeamEndpointsResponse` with `data: TeamEndpoint[]`.

```rust
// Rust
let resp = qn.admin.list_team_endpoints(42).await?;
```

##### `update_team_endpoints` / `updateTeamEndpoints`

Replaces the set of endpoints associated with a team. Pass an empty array to remove all.

**Parameters**: `id` (i64, required); body: `endpoint_ids` (string[], required).

**Returns**: `UpdateTeamEndpointsResponse`.

```rust
// Rust
let params = UpdateTeamEndpointsRequest::builder()
    .endpoint_ids(vec!["ep-123".to_string(), "ep-456".to_string()])
    .build();
qn.admin.update_team_endpoints(42, &params).await?;
```

##### `invite_team_member` / `inviteTeamMember`

Invites a user to a team. Existing users only need `email`; new users require `full_name` and `role`.

**Parameters**: `id` (i64, required); body: `email` (string, required), `full_name` (string, optional), `role` (string, optional — `admin` | `viewer` | `billing`).

**Returns**: `InviteTeamMemberResponse`.

```rust
// Rust
let params = InviteTeamMemberRequest::builder()
    .email("alice@example.com".to_string())
    .role("viewer".to_string())
    .build();
qn.admin.invite_team_member(42, &params).await?;
```

##### `remove_team_member` / `removeTeamMember`

Removes a user from a team.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `RemoveTeamMemberResponse`.

```rust
// Rust
qn.admin.remove_team_member(42, 7).await?;
```

##### `resend_team_invite` / `resendTeamInvite`

Re-sends a pending team invitation.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `ResendTeamInviteResponse`.

```rust
// Rust
qn.admin.resend_team_invite(42, 7).await?;
```

#### Usage

All usage methods accept optional `start_time` and `end_time` Unix timestamps. Omit both for account-to-date totals.

##### `get_usage` / `getUsage`

Aggregate account usage for a time window.

**Returns**: `GetUsageResponse` with `data: UsageData` (`credits_used`, `credits_remaining`, `limit`, `overages`, `start_time`, `end_time`).

```rust
// Rust
let resp = qn.admin.get_usage(&GetUsageRequest::default()).await?;
```

##### `get_usage_by_endpoint` / `getUsageByEndpoint`

Per-endpoint usage breakdown.

**Returns**: `GetUsageByEndpointResponse` with `data.endpoints: EndpointUsage[]`.

```rust
// Rust
let resp = qn.admin.get_usage_by_endpoint(&GetUsageRequest::default()).await?;
```

##### `get_usage_by_method` / `getUsageByMethod`

Per-RPC-method usage breakdown.

**Returns**: `GetUsageByMethodResponse` with `data.methods: MethodUsage[]`.

```rust
// Rust
let resp = qn.admin.get_usage_by_method(&GetUsageRequest::default()).await?;
```

##### `get_usage_by_chain` / `getUsageByChain`

Per-chain usage breakdown.

**Returns**: `GetUsageByChainResponse` with `data.chains: ChainUsage[]`.

```rust
// Rust
let resp = qn.admin.get_usage_by_chain(&GetUsageRequest::default()).await?;
```

##### `get_usage_by_tag` / `getUsageByTag`

Per-tag usage breakdown.

**Returns**: `GetUsageByTagResponse` with `data.tags: TagUsage[]`.

```rust
// Rust
let resp = qn.admin.get_usage_by_tag(&GetUsageRequest::default()).await?;
```

#### Logs

##### `get_endpoint_logs` / `getEndpointLogs`

Fetches a page of request logs for an endpoint. Set `include_details=true` for full request/response payloads (truncated at 2 KB each).

**Parameters**: `id` (endpoint id, required); body: `from` (string timestamp, required), `to` (string timestamp, required), `include_details` (bool, optional), `limit` (i32, optional), `next_at` (string cursor, optional).

**Returns**: `GetEndpointLogsResponse` — `{ data: EndpointLog[], next_at?: string }`.

```rust
// Rust
let params = GetEndpointLogsRequest::builder()
    .from("2026-04-01T00:00:00Z".to_string())
    .to("2026-04-02T00:00:00Z".to_string())
    .limit(100)
    .build();
let resp = qn.admin.get_endpoint_logs("ep-123", &params).await?;
```

##### `get_log_details` / `getLogDetails`

Returns the full request/response payloads for a single log entry.

**Parameters**: `id` (endpoint id, required), `request_id` (log request uuid, required).

**Returns**: `GetLogDetailsResponse` with `data: LogDetails`.

```rust
// Rust
let resp = qn.admin.get_log_details("ep-123", "req-abc").await?;
```

#### Endpoint Security

##### `get_endpoint_security` / `getEndpointSecurity`

Returns the full security configuration for an endpoint: tokens, JWTs, referrers, domain masks, IPs, request filters, and their per-feature toggles.

**Parameters**: `id` (string, required).

**Returns**: `GetEndpointSecurityResponse` with `data: EndpointSecurity`.

```rust
// Rust
let resp = qn.admin.get_endpoint_security("ep-123").await?;
```

#### Security Options

##### `get_security_options` / `getSecurityOptions`

Returns the list of security features and their enabled state for an endpoint.

**Parameters**: `id` (string, required).

**Returns**: `GetSecurityOptionsResponse` with `data: SecurityOption[]`.

```rust
// Rust
let resp = qn.admin.get_security_options("ep-123").await?;
```

##### `update_security_options` / `updateSecurityOptions`

Enables or disables individual security features. Each field accepts `"enabled"` or `"disabled"`.

**Parameters**: `id` (string, required); `options`: `SecurityOptionsUpdate` (`tokens`, `referrers`, `jwts`, `ips`, `domain_masks`, `hsts`, `cors`, `request_filters`, `ip_custom_header`).

**Returns**: `UpdateSecurityOptionsResponse` with updated `SecurityOption[]`.

```rust
// Rust
let options = SecurityOptionsUpdate::builder()
    .tokens("enabled".to_string())
    .jwts("disabled".to_string())
    .build();
let params = UpdateSecurityOptionsRequest { options };
qn.admin.update_security_options("ep-123", &params).await?;
```

#### Tokens

##### `create_token` / `createToken`

Generates a new auth token on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.create_token("ep-123").await?;
```

##### `delete_token` / `deleteToken`

Revokes a token on an endpoint.

**Parameters**: `id` (endpoint id, required), `token_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_token("ep-123", "tok-1").await?;
```

#### Referrers

##### `create_referrer` / `createReferrer`

Whitelists a referrer URL or domain on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `referrer` (string, required).

**Returns**: nothing.

```rust
// Rust
let params = CreateReferrerRequest::builder().referrer("example.com".to_string()).build();
qn.admin.create_referrer("ep-123", &params).await?;
```

##### `delete_referrer` / `deleteReferrer`

Removes a referrer from the whitelist.

**Parameters**: `id` (endpoint id, required), `referrer_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_referrer("ep-123", "ref-1").await?;
```

#### IPs

##### `create_ip` / `createIp`

Whitelists an IP address on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `ip` (string, required).

**Returns**: nothing.

```rust
// Rust
let params = CreateIpRequest::builder().ip("198.51.100.7".to_string()).build();
qn.admin.create_ip("ep-123", &params).await?;
```

##### `delete_ip` / `deleteIp`

Removes an IP from the whitelist.

**Parameters**: `id` (endpoint id, required), `ip_id` (string, required).

**Returns**: `DeleteBoolResponse`.

```rust
// Rust
qn.admin.delete_ip("ep-123", "ip-1").await?;
```

#### Domain Masks

##### `create_domain_mask` / `createDomainMask`

Adds a custom domain mask to an endpoint.

**Parameters**: `id` (endpoint id, required); body: `domain_mask` (string, optional).

**Returns**: nothing.

```rust
// Rust
let params = CreateDomainMaskRequest::builder()
    .domain_mask("rpc.example.com".to_string())
    .build();
qn.admin.create_domain_mask("ep-123", &params).await?;
```

##### `delete_domain_mask` / `deleteDomainMask`

Removes a domain mask.

**Parameters**: `id` (endpoint id, required), `domain_mask_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_domain_mask("ep-123", "dm-1").await?;
```

#### JWTs

##### `create_jwt` / `createJwt`

Configures JWT validation on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `public_key` (string, required), `kid` (string, required), `name` (string, required).

**Returns**: nothing.

```rust
// Rust
let params = CreateJwtRequest::builder()
    .public_key("-----BEGIN PUBLIC KEY-----\n...".to_string())
    .kid("key-1".to_string())
    .name("primary".to_string())
    .build();
qn.admin.create_jwt("ep-123", &params).await?;
```

##### `delete_jwt` / `deleteJwt`

Removes a JWT configuration.

**Parameters**: `id` (endpoint id, required), `jwt_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_jwt("ep-123", "jwt-1").await?;
```

#### Request Filters

Whitelist specific RPC methods on an endpoint. Requests for methods not on the list are blocked when the feature is enabled.

##### `create_request_filter` / `createRequestFilter`

**Parameters**: `id` (endpoint id, required); body: `method` (string[], required). Ruby's Hash key is `methods` (plural).

**Returns**: `CreateRequestFilterResponse` with `data.id`.

```rust
// Rust
let params = CreateRequestFilterRequest::builder()
    .method(vec!["eth_blockNumber".to_string(), "eth_getBalance".to_string()])
    .build();
let resp = qn.admin.create_request_filter("ep-123", &params).await?;
```

##### `update_request_filter` / `updateRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required); body: `method` (string[], optional). Ruby's Hash keys are `request_filter_id` and `methods` (plural).

**Returns**: nothing.

```rust
// Rust
let params = UpdateRequestFilterRequest::builder()
    .method(vec!["eth_call".to_string()])
    .build();
qn.admin.update_request_filter("ep-123", "f-1", &params).await?;
```

##### `delete_request_filter` / `deleteRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_request_filter("ep-123", "f-1").await?;
```

#### Multichain

##### `enable_multichain` / `enableMultichain`

Enables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.enable_multichain("ep-123").await?;
```

##### `disable_multichain` / `disableMultichain`

Disables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.disable_multichain("ep-123").await?;
```

#### IP Custom Headers

##### `create_or_update_ip_custom_header` / `createOrUpdateIpCustomHeader`

Sets the custom header used to identify the client IP (e.g. when traffic is proxied).

**Parameters**: `id` (endpoint id, required); body: `header_name` (string, required).

**Returns**: `CreateOrUpdateIpCustomHeaderResponse` with `data.header_name`.

```rust
// Rust
let params = CreateOrUpdateIpCustomHeaderRequest::builder()
    .header_name("X-Forwarded-For".to_string())
    .build();
qn.admin.create_or_update_ip_custom_header("ep-123", &params).await?;
```

##### `delete_ip_custom_header` / `deleteIpCustomHeader`

Removes the custom IP header configuration.

**Parameters**: `id` (endpoint id, required).

**Returns**: `DeleteBoolResponse`.

```rust
// Rust
qn.admin.delete_ip_custom_header("ep-123").await?;
```

#### Method Rate Limits

##### `get_method_rate_limits` / `getMethodRateLimits`

Lists method-level rate limiters configured on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetMethodRateLimitsResponse` with `data.rate_limiters: MethodRateLimiter[]`.

```rust
// Rust
let resp = qn.admin.get_method_rate_limits("ep-123").await?;
```

##### `create_method_rate_limit` / `createMethodRateLimit`

Creates a new method-level rate limiter.

**Parameters**: `id` (endpoint id, required); body: `interval` (string, e.g. `"second"`), `methods` (string[]), `rate` (i32).

**Returns**: `CreateMethodRateLimitResponse` with `data: MethodRateLimiter`.

```rust
// Rust
let params = CreateMethodRateLimitRequest::builder()
    .interval("second".to_string())
    .methods(vec!["eth_call".to_string()])
    .rate(10)
    .build();
let resp = qn.admin.create_method_rate_limit("ep-123", &params).await?;
```

##### `update_method_rate_limit` / `updateMethodRateLimit`

Updates an existing rate limiter. Only provided fields change.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required); body: `methods` (string[], optional), `status` (`"enabled"` | `"disabled"`, optional), `rate` (i32, optional).

**Returns**: `UpdateMethodRateLimitResponse`.

```rust
// Rust
let params = UpdateMethodRateLimitRequest::builder().rate(50).build();
qn.admin.update_method_rate_limit("ep-123", "rl-1", &params).await?;
```

##### `delete_method_rate_limit` / `deleteMethodRateLimit`

Deletes a rate limiter.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_method_rate_limit("ep-123", "rl-1").await?;
```

#### Endpoint Rate Limits

##### `update_rate_limits` / `updateRateLimits`

Partial update of the endpoint-level RPS / RPM / RPD caps. Only buckets included in the request are modified — omitted buckets are left unchanged. Values are capped by the account's plan tier. Sends `PATCH`.

**Parameters**: `id` (endpoint id, required); `rate_limits`: `RateLimitSettings` (`rps`, `rpm`, `rpd`, all optional).

**Returns**: nothing.

```rust
// Rust
let rate_limits = RateLimitSettings::builder().rps(100).rpm(5000).build();
let params = UpdateRateLimitsRequest { rate_limits };
qn.admin.update_rate_limits("ep-123", &params).await?;
```

##### `get_rate_limits` / `getRateLimits`

Returns the rate-limit rows currently enforced on the endpoint, each identifying its `bucket` (`"rps"` / `"rpm"` / `"rpd"`), `rate_limit`, and `source` (`"plan_default"` or `"user_override"`). User-set overrides expose an `id` you can pass to `delete_rate_limit_override`.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetRateLimitsResponse` with `data.rate_limits: Vec<RateLimitEntry>`.

```rust
// Rust
let resp = qn.admin.get_rate_limits("123").await?;
for row in resp.data.unwrap().rate_limits {
    println!("{} {} {} {:?}", row.bucket, row.rate_limit, row.source, row.id);
}
```

##### `delete_rate_limit_override` / `deleteRateLimitOverride`

Deletes a user-set rate-limit override by UUID. Plan defaults are not deletable — passing a UUID that does not match a user-set override on the endpoint returns 404.

**Parameters**: `id` (endpoint id, required); `override_id` (UUID returned by `get_rate_limits`, required).

**Returns**: nothing.

```rust
// Rust
qn.admin.delete_rate_limit_override("123", "ovr-uuid").await?;
```

#### Endpoint URLs

##### `get_endpoint_urls` / `getEndpointUrls`

Returns the HTTP and WebSocket URLs for the endpoint without fetching the full endpoint record. For multichain endpoints, `multichain_urls` is a per-network map of additional URLs; for single-chain endpoints it is `None`.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetEndpointUrlsResponse` with `data.http_url`, `data.wss_url`, and `data.multichain_urls`.

```rust
// Rust
let resp = qn.admin.get_endpoint_urls("123").await?;
if let Some(data) = resp.data {
    println!("{}", data.http_url);
    if let Some(mc) = data.multichain_urls {
        for (network, urls) in mc {
            println!("{network} {}", urls.http_url);
        }
    }
}
```

#### Metrics

##### `get_endpoint_metrics` / `getEndpointMetrics`

Returns metric series for an endpoint over a time period.

**Parameters**: `id` (endpoint id, required); body: `period` (`"hour"` | `"day"` | `"week"` | `"month"`), `metric` (e.g. `"method_calls_over_time"`, `"response_status_breakdown"`).

**Returns**: `GetEndpointMetricsResponse` with `data: Vec<EndpointMetric>`. Each `EndpointMetric` has `tag: Vec<String>` and `data: Vec<Vec<i64>>` of `[timestamp, value]` pairs. Single-axis series (e.g. `response_time_over_time` with a percentile) come back as a one-element tag like `vec!["p95"]`; multi-axis series come back as `vec!["network", "arbitrum-mainnet"]`.

```rust
// Rust
let params = GetEndpointMetricsRequest {
    period: "day".to_string(),
    metric: "method_calls_over_time".to_string(),
};
let resp = qn.admin.get_endpoint_metrics("ep-123", &params).await?;
```

##### `get_account_metrics` / `getAccountMetrics`

Returns account-level metric series. Supports an optional `percentile` (e.g. `"p50"`, `"p95"`, `"p99"`) for latency metrics.

**Parameters**: `period` (required), `metric` (required), `percentile` (string, optional).

**Returns**: `GetAccountMetricsResponse` with `data: Vec<EndpointMetric>`. See `get_endpoint_metrics` above for the `tag: Vec<String>` shape.

```rust
// Rust
let params = GetAccountMetricsRequest {
    period: "day".to_string(),
    metric: "credits_over_time".to_string(),
    percentile: None,
};
let resp = qn.admin.get_account_metrics(&params).await?;
```

#### Chains

##### `list_chains` / `listChains`

Lists the blockchains supported by Quicknode along with their networks.

**Parameters**: none.

**Returns**: `ListChainsResponse` with `data: Chain[]`.

```rust
// Rust
let resp = qn.admin.list_chains().await?;
```

#### Account

##### `account_info` / `accountInfo`

Returns details about the account, including its id, name, creation timestamp, billing version, and current subscription.

**Parameters**: none.

**Returns**: `AccountInfoResponse` with `data: AccountInfo` (including a nested `subscription: AccountSubscription`).

```rust
// Rust
let resp = qn.admin.account_info().await?;
```

##### `get_api_credits` / `getApiCredits`

Returns the per-method API credit costs for a chain, identified by its slug (the same slugs returned by `list_chains`, e.g. `ethereum`). An unknown chain slug returns a 404 (surfaced as `ApiError`).

**Parameters**: `chain` (string, required) — the chain slug.

**Returns**: `GetApiCreditsResponse` with `data: Vec<ApiCredit>`, where each `ApiCredit` has `method` and `credits`.

```rust
// Rust
let resp = qn.admin.get_api_credits("ethereum").await?;
```

#### Billing

##### `list_invoices` / `listInvoices`

Lists invoices on the account.

**Parameters**: none.

**Returns**: `ListInvoicesResponse` with `data.invoices: Invoice[]`.

```rust
// Rust
let resp = qn.admin.list_invoices().await?;
```

##### `list_payments` / `listPayments`

Lists payments on the account.

**Parameters**: none.

**Returns**: `ListPaymentsResponse` with `data.payments: Payment[]`.

```rust
// Rust
let resp = qn.admin.list_payments().await?;
```

#### Bulk Operations

##### `bulk_update_endpoint_status` / `bulkUpdateEndpointStatus`

Activates or pauses many endpoints at once.

**Parameters**: `ids` (string[], required), `status` (`"active"` | `"paused"`, required).

**Returns**: `BulkUpdateEndpointStatusResponse` with per-endpoint `results`.

```rust
// Rust
let params = BulkUpdateEndpointStatusRequest::builder()
    .ids(vec!["ep-1".to_string(), "ep-2".to_string()])
    .status("paused".to_string())
    .build();
let resp = qn.admin.bulk_update_endpoint_status(&params).await?;
```

##### `bulk_add_tag` / `bulkAddTag`

Applies a tag (created if missing) to many endpoints at once.

**Parameters**: `ids` (string[], required), `label` (string, required).

**Returns**: `BulkAddTagResponse`.

```rust
// Rust
let params = BulkAddTagRequest::builder()
    .ids(vec!["ep-1".to_string(), "ep-2".to_string()])
    .label("prod".to_string())
    .build();
let resp = qn.admin.bulk_add_tag(&params).await?;
```

##### `bulk_remove_tag` / `bulkRemoveTag`

Removes a tag from many endpoints at once.

**Parameters**: `ids` (string[], required), `tag_id` (i32, required).

**Returns**: `BulkRemoveTagResponse`.

```rust
// Rust
let params = BulkRemoveTagRequest::builder()
    .ids(vec!["ep-1".to_string(), "ep-2".to_string()])
    .tag_id(42)
    .build();
let resp = qn.admin.bulk_remove_tag(&params).await?;
```

#### Account Tags

##### `list_tags` / `listTags`

Lists every tag on the account along with usage counts.

**Parameters**: none.

**Returns**: `ListTagsResponse` with `data.tags: AccountTag[]`.

```rust
// Rust
let resp = qn.admin.list_tags().await?;
```

##### `rename_tag` / `renameTag`

Renames an account-level tag.

**Parameters**: `tag_id` (i32, required); body: `label` (string, required).

**Returns**: `RenameTagResponse` with updated `AccountTag`.

```rust
// Rust
let params = RenameTagRequest::builder().label("staging".to_string()).build();
let resp = qn.admin.rename_tag(42, &params).await?;
```

##### `delete_account_tag` / `deleteAccountTag`

Deletes a tag from the account. The tag must first be removed from any endpoints using it.

**Parameters**: `id` (i32, required).

**Returns**: `DeleteAccountTagResponse`.

```rust
// Rust
qn.admin.delete_account_tag(42).await?;
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

```rust
// Rust
let params = CreateStreamParams::builder()
    .name("My Stream".to_string())
    .region(StreamRegion::UsaEast)
    .network("ethereum-mainnet".to_string())
    .dataset(StreamDataset::Block)
    .start_range(24691804)
    .end_range(24691904)
    .destination_attributes(DestinationAttributes::Webhook(WebhookAttributes {
        url: "https://webhook.site/...".to_string(),
        max_retry: 3,
        retry_interval_sec: 1,
        post_timeout_sec: 10,
        compression: "none".to_string(),
        security_token: None,
    }))
    .plan("growth_plan".to_string())
    .threshold_fetch_buffer(1000)
    .status(StreamStatus::Active)
    .build();
let stream = qn.streams.create_stream(&params).await?;
```

##### `list_streams` / `listStreams`

Paginated list of streams on the account.

**Parameters** (all optional): `offset` (i64), `limit` (i64), `order_by` (string), `order_direction` (`"asc"` | `"desc"`), `stream_type` (string).

**Returns**: `ListStreamsResponse` with `data: Stream[]` and `page_info`.

```rust
// Rust
let resp = qn.streams.list_streams(&ListStreamsParams::default()).await?;
```

##### `get_stream` / `getStream`

Fetches one stream by id.

**Parameters**: `id` (string, required).

**Returns**: `Stream`.

```rust
// Rust
let stream = qn.streams.get_stream("stream-id").await?;
```

##### `update_stream` / `updateStream`

Partially updates a stream. Omitted fields are left unchanged.

**Parameters**: `id` (string, required); body: any field from `CreateStreamParams` (all optional).

**Returns**: updated `Stream`.

```rust
// Rust
let params = UpdateStreamParams {
    name: Some("Renamed".to_string()),
    ..Default::default()
};
let stream = qn.streams.update_stream("stream-id", &params).await?;
```

##### `delete_stream` / `deleteStream`

Deletes one stream by id.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.streams.delete_stream("stream-id").await?;
```

##### `delete_all_streams` / `deleteAllStreams`

Deletes every stream on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```rust
// Rust
qn.streams.delete_all_streams().await?;
```

##### `activate_stream` / `activateStream`

Resumes delivery on a stream from its current position.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.streams.activate_stream("stream-id").await?;
```

##### `pause_stream` / `pauseStream`

Halts delivery on a stream.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.streams.pause_stream("stream-id").await?;
```

##### `test_filter` / `testFilter`

Runs a filter function against a block so it can be validated before being attached to a live stream.

**Parameters**: `network` (string, required), `dataset` (`StreamDataset`, required), `block` (string, required), `filter_function` (string, optional), `filter_language` (`FilterLanguage`, optional), `address_book_config` (optional).

**Returns**: `TestFilterResponse` with `result` and `logs`.

```rust
// Rust
let params = TestFilterParams {
    network: "ethereum-mainnet".to_string(),
    dataset: StreamDataset::Block,
    block: "17811625".to_string(),
    filter_function: None,
    filter_language: None,
    address_book_config: None,
};
let resp = qn.streams.test_filter(&params).await?;
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled (active) streams, optionally filtered by type.

**Parameters**: `stream_type` (string, optional).

**Returns**: `EnabledCountResponse` with `total`.

```rust
// Rust
let resp = qn.streams.get_enabled_count(None).await?;
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

`TemplateArgs` carries the arguments. Each template supports two input forms — inline values or a reference to a pre-created list by name. Construct one per template via the variant + the appropriate input enum (`<Template>Input::Inline | ByList`):

| Variant | Inline struct (fields) | ByList struct (fields) |
|---|---|---|
| `EvmWalletFilter` | `EvmWalletFilterTemplate { wallets: string[] }` | `EvmWalletFilterByListTemplate { wallets_list_name: string }` |
| `EvmContractEvents` | `EvmContractEventsTemplate { contracts: string[], event_hashes: string[] }` | `EvmContractEventsByListTemplate { contracts_list_name: string, event_hashes_list_name?: string }` |
| `EvmAbiFilter` | `EvmAbiFilterTemplate { abi: string, contracts: string[] }` | `EvmAbiFilterByListTemplate { abi_json: string, contracts_list_name?: string }` |
| `SolanaWalletFilter` | `SolanaWalletFilterTemplate { accounts: string[] }` | `SolanaWalletFilterByListTemplate { accounts_list_name: string }` |
| `BitcoinWalletFilter` | `BitcoinWalletFilterTemplate { wallets: string[] }` | `BitcoinWalletFilterByListTemplate { wallets_list_name: string }` |
| `XrplWalletFilter` | `XrplWalletFilterTemplate { wallets: string[] }` | `XrplWalletFilterByListTemplate { wallets_list_name: string }` |
| `HyperliquidWalletEventsFilter` | `HyperliquidWalletEventsFilterTemplate { wallets: string[] }` | `HyperliquidWalletEventsFilterByListTemplate { wallets_list_name: string }` |
| `StellarWalletTransactionsSourceAccountFilter` | `StellarWalletTransactionsFilterTemplate { wallets: string[] }` | `StellarWalletTransactionsFilterByListTemplate { wallets_list_name: string }` |

`WebhookDestinationAttributes`: `url` (required), `compression` (required — `"none"` | `"gzip"`), `security_token` (optional — auto-generated if omitted).

`WebhookStartFrom`: `Last` (resume from last delivered block) or `Latest` (start from newest).

In Ruby, `template_args` is passed as a JSON string under the key `template_args_json`; destination is passed as a JSON string under `destination_attributes_json`.

#### Webhooks methods

##### `list_webhooks` / `listWebhooks`

Paginated list of webhooks.

**Parameters** (all optional): `limit` (i64), `offset` (i64).

**Returns**: `ListWebhooksResponse` with `data: Webhook[]` and `pageInfo: WebhookPageInfo { limit, offset, total }`.

```rust
// Rust
let resp = qn.webhooks.list_webhooks(&GetWebhooksParams::default()).await?;
```

##### `get_webhook` / `getWebhook`

Fetches a webhook by id.

**Parameters**: `id` (string, required).

**Returns**: `Webhook`.

```rust
// Rust
let webhook = qn.webhooks.get_webhook("wh-1").await?;
```

##### `create_webhook_from_template` / `createWebhookFromTemplate`

Creates a webhook from a predefined filter template.

**Parameters**: `name` (required), `network` (required), `destination_attributes` (`WebhookDestinationAttributes`, required), `template_args` (required — use the `TemplateArgs` enum variant for the chosen template), `notification_email` (optional).

**Returns**: `Webhook`.

```rust
// Rust
let template_args = TemplateArgs::EvmWalletFilter(EvmWalletFilterTemplate {
    wallets: vec!["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
});
let params = CreateWebhookFromTemplateParams {
    name: "Wallet Webhook".to_string(),
    network: "ethereum-mainnet".to_string(),
    notification_email: None,
    destination_attributes: WebhookDestinationAttributes {
        url: "https://webhook.site/...".to_string(),
        security_token: None,
        compression: "none".to_string(),
    },
    template_args,
};
let webhook = qn.webhooks.create_webhook_from_template(&params).await?;
```

##### `update_webhook` / `updateWebhook`

Partially updates a webhook's name, notification email, and/or destination. If `destination_attributes` is supplied without `security_token`, a new token is generated automatically.

**Parameters**: `id` (required); body — all optional: `name`, `notification_email`, `destination_attributes`. In Ruby, `destination_attributes` is passed as a JSON string under the key `destination_attributes_json`.

**Returns**: updated `Webhook`.

```rust
// Rust
let params = UpdateWebhookParams {
    name: Some("Renamed Webhook".to_string()),
    ..Default::default()
};
let webhook = qn.webhooks.update_webhook("wh-1", &params).await?;
```

##### `update_webhook_template` / `updateWebhookTemplate`

Updates the template args (and optionally name, email, destination) on an existing template-backed webhook.

**Parameters**: `webhook_id` (required), `template_args` (required); optional: `name`, `notification_email`, `destination_attributes`.

**Returns**: updated `Webhook`.

```rust
// Rust
let template_args = TemplateArgs::EvmWalletFilter(EvmWalletFilterTemplate {
    wallets: vec!["0xnewwallet".to_string()],
});
let params = UpdateWebhookTemplateParams {
    name: None,
    notification_email: None,
    destination_attributes: None,
    template_args,
};
let webhook = qn.webhooks.update_webhook_template("wh-1", &params).await?;
```

##### `delete_webhook` / `deleteWebhook`

Deletes a webhook.

**Parameters**: `id` (required).

**Returns**: nothing.

```rust
// Rust
qn.webhooks.delete_webhook("wh-1").await?;
```

##### `delete_all_webhooks` / `deleteAllWebhooks`

Deletes every webhook on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```rust
// Rust
qn.webhooks.delete_all_webhooks().await?;
```

##### `pause_webhook` / `pauseWebhook`

Pauses a webhook so it stops delivering events.

**Parameters**: `id` (required).

**Returns**: nothing.

```rust
// Rust
qn.webhooks.pause_webhook("wh-1").await?;
```

##### `activate_webhook` / `activateWebhook`

Activates a paused or new webhook so it resumes delivering events. `start_from` determines where processing resumes.

**Parameters**: `id` (required), `start_from` (`WebhookStartFrom`, required — `Last` or `Latest`).

**Returns**: nothing.

```rust
// Rust
let params = ActivateWebhookParams { start_from: WebhookStartFrom::Latest };
qn.webhooks.activate_webhook("wh-1", &params).await?;
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled webhooks.

**Parameters**: none.

**Returns**: `WebhookEnabledCountResponse` with `total`.

```rust
// Rust
let resp = qn.webhooks.get_enabled_count().await?;
```

---

### KV Store Client

Accessed as `qn.kvstore`. Provides two primitives — **sets** (single string values under a key) and **lists** (ordered collections of strings under a key). Backed by `https://api.quicknode.com/kv/rest/v1/`.

#### Sets

##### `create_set` / `createSet`

Stores a single string value under a key.

**Parameters**: `key` (string, required), `value` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.create_set(&CreateSetParams {
    key: "my-key".to_string(),
    value: "hello".to_string(),
}).await?;
```

##### `get_sets` / `getSets`

Paginated page of key/value entries.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetSetsResponse` — `{ data: KvSetEntry[], cursor: string }`.

```rust
// Rust
let resp = qn.kvstore.get_sets(&Default::default()).await?;
```

##### `get_set` / `getSet`

Returns the value stored under a key.

**Parameters**: `key` (string, required).

**Returns**: `GetSetResponse` with `value`.

```rust
// Rust
let resp = qn.kvstore.get_set("my-key").await?;
```

##### `bulk_sets` / `bulkSets`

Adds and/or deletes multiple sets in a single request.

**Parameters** (at least one required): `add_sets` (map<string,string>, optional), `delete_sets` (string[], optional).

**Returns**: nothing.

```rust
// Rust
use std::collections::HashMap;

let mut add_sets = HashMap::new();
add_sets.insert("k1".to_string(), "v1".to_string());
qn.kvstore.bulk_sets(&BulkSetsParams {
    add_sets: Some(add_sets),
    delete_sets: Some(vec!["old-key".to_string()]),
}).await?;
```

##### `delete_set` / `deleteSet`

Deletes a single set.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.delete_set("my-key").await?;
```

#### Lists

##### `create_list` / `createList`

Creates a list under a key, seeded with the initial items.

**Parameters**: `key` (string, required), `items` (string[], required).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.create_list(&CreateListParams {
    key: "my-list".to_string(),
    items: vec!["0xabc".to_string(), "0xdef".to_string()],
}).await?;
```

##### `get_lists` / `getLists`

Paginated page of list keys.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetListsResponse` — `{ data: { keys: string[] }, cursor: string }`.

```rust
// Rust
let resp = qn.kvstore.get_lists(&Default::default()).await?;
```

##### `get_list` / `getList`

Paginated page of items for a specific list.

**Parameters**: `key` (string, required); optional `limit` (i64), `cursor` (string).

**Returns**: `GetListResponse` — `{ data: { items: string[] }, cursor: string }`.

```rust
// Rust
let resp = qn.kvstore.get_list("my-list", &Default::default()).await?;
```

##### `update_list` / `updateList`

Adds and/or removes items in a single operation.

**Parameters**: `key` (string, required); optional: `add_items` (string[]), `remove_items` (string[]).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.update_list(
    "my-list",
    &UpdateListParams {
        add_items: Some(vec!["0x456".to_string()]),
        remove_items: Some(vec!["0xabc".to_string()]),
    },
).await?;
```

##### `add_list_item` / `addListItem`

Appends a single item to a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.add_list_item(
    "my-list",
    &AddListItemParams { item: "0x123".to_string() },
).await?;
```

##### `list_contains_item` / `listContainsItem`

Checks whether a list contains a specific item.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: `ListContainsItemResponse` with `exists: bool`.

```rust
// Rust
let resp = qn.kvstore.list_contains_item("my-list", "0x123").await?;
```

##### `delete_list_item` / `deleteListItem`

Removes a single item from a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.delete_list_item("my-list", "0x123").await?;
```

##### `delete_list` / `deleteList`

Deletes a list and all of its items.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```rust
// Rust
qn.kvstore.delete_list("my-list").await?;
```

---

### SQL Client

Accessed as `qn.sql`. Runs SQL queries against indexed blockchain data and fetches the database schema. Backed by `https://api.quicknode.com/sql/rest/v1/`.

##### `query`

Executes a SQL query against a cluster and returns the result set. Paginate by writing `LIMIT`/`OFFSET` into the SQL.

**Parameters**: `QueryParams` with `query` (String, required) and `cluster_id` (String, required).

**Returns**: `QueryResponse` — `meta` (`Vec<ColumnMeta>`, each with `name` and `column_type`), `data` (`Vec<serde_json::Value>`, rows as JSON objects keyed by column name), `rows`, `rows_before_limit_at_least`, `statistics` (`QueryStatistics` with `elapsed`, `rows_read`, `bytes_read`), and `credits`.

```rust
// Rust
let resp = qn
    .sql
    .query(&QueryParams {
        query: "SELECT action_type, user FROM hyperliquid_system_actions ORDER BY block_time DESC LIMIT 100".to_string(),
        cluster_id: "hyperliquid-core-mainnet".to_string(),
    })
    .await?;
println!("{} rows, {:?}", resp.rows, resp.data.first());
```

##### `get_schema`

Fetches the database schema for a cluster: table names, columns, types, sort keys, and partition strategies.

**Parameters**: `cluster_id` (`&str`, required).

**Returns**: `ChainSchema` — `chain`, `cluster_id`, and `tables` (`Vec<TableSchema>`, each with `name`, `engine`, `total_rows`, `partition_key`, `sorting_key`, and `columns` of `ColumnSchema { name, column_type }`).

```rust
// Rust
let schema = qn.sql.get_schema("hyperliquid-core-mainnet").await?;
println!("{} tables", schema.tables.len());
```

---

### RPC & Tooling Access

Tooling Access provisions a single multichain, read-only endpoint per account and
mints short-lived session JWTs. `qn.rpc` makes JSON-RPC calls directly against that
endpoint, minting and refreshing the JWT automatically — no endpoint URL or token to
manage.

Tooling Access must be enabled once (admin role + eligible plan). The control-plane
methods live on `qn.admin`:

```rust
// Rust
let status = qn.admin.tooling_access_status().await?;
if !status.enabled {
    qn.admin.enable_tooling_access().await?; // idempotent; admin role required
}

// call(method, params, network, endpoint_url). params is Option<serde_json::Value>;
// None defaults to []. Both trailing args are Option and independently omittable.
let block_number = qn.rpc.call("eth_blockNumber", None, None, None).await?;
let balance = qn
    .rpc
    .call(
        "eth_getBalance",
        Some(serde_json::json!(["0xabc...", "latest"])),
        None,
        None,
    )
    .await?;

// Multichain: seed the per-network URL map (from get_endpoint_urls), then pass
// the network key as the third arg.
let urls = qn.admin.get_endpoint_urls(endpoint_id).await?;
if let Some(data) = urls.data {
    if let Some(mc) = data.multichain_urls {
        qn.rpc
            .set_networks(mc.into_iter().map(|(k, v)| (k, v.http_url)).collect());
    }
}
let slot = qn.rpc.call("getSlot", None, Some("solana-mainnet".into()), None).await?;

// Custom endpoint URL: send to a fully-formed HTTP URL, bypassing Tooling Access
// and the JWT (no Authorization header). Per-call via the 4th arg, or client-wide
// via RpcConfig { endpoint_url, .. }. endpoint_url and network are mutually
// exclusive (a custom URL is not multichain-routed).
let block = qn
    .rpc
    .call("eth_blockNumber", None, None, Some("https://my-endpoint.example/rpc".into()))
    .await?;

// A JSON-RPC error member is returned as SdkError::Rpc { code, message }.
```

A host that persists across processes can snapshot the cached token with
`qn.rpc.current_token()` and re-seed it via `RpcConfig { seed, .. }`;
`refresh_margin_secs` (default 60) tunes how early the token is refreshed. Set
`RpcConfig { endpoint_url, .. }` to route every call to a custom HTTP URL by
default (no JWT minted); a per-call `endpoint_url` overrides it.

## Crypto-micropayment lane (`rpc.call`)

Pay per RPC request with a stablecoin instead of a provisioned account + API key,
against Quicknode's `x402.quicknode.com` and `mpp.quicknode.com` gateways. Configure
it by setting `payment` on the RPC config; the SDK runs the `402` → sign → resend
handshake for you. An API key is **not** required for this lane — build a keyless SDK.

Confirmed paths: **x402/EVM** (EIP-712 `TransferWithAuthorization`), **x402/Solana**
(SPL `TransferChecked` in a v0 tx, gateway sponsors gas), and **MPP/Tempo** (native Tempo tx).

`PaymentConfig` fields:

| Field | Meaning |
|---|---|
| `scheme` | `"x402"` (pay-per-request) or `"mpp"` (MPP charge) |
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
  never prints it in its own errors/`Debug`, but a plain `{:?}`/`dbg!(config)` will show it.
- **`max_amount` is integer base units of the selected asset.** The SDK skips any offered
  entry above it and refuses to sign one — a guard against an overcharging gateway.
- **`PaymentIndeterminateError` means the paid request was sent but the response was lost.**
  You MAY have been charged — do **not** blindly retry.
- **x402/Solana: one payment per call.** Building a payment reads the mint and a recent
  blockhash from a Solana RPC. The default is a public RPC that **rate-limits
  aggressively** — set `svm_rpc_url` to your own endpoint at any volume.

```rust
use quicknode_sdk::{PaymentConfig, QuicknodeSdk, RpcConfig, SdkFullConfig};

let mut config = SdkFullConfig::keyless();
config.rpc = Some(RpcConfig {
    payment: Some(PaymentConfig {
        scheme: "x402".into(),
        key: std::env::var("QN_PAYMENT_KEY").unwrap(),
        pay_network: "eip155:84532".into(),
        asset: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".into(),
        max_amount: "10000".into(),
        svm_rpc_url: None,
        base_url_override: None,
    }),
    ..Default::default()
});
let qn = QuicknodeSdk::new(&config)?;
let resp = qn.rpc.call_with_receipt("eth_blockNumber", None, Some("base-sepolia".into()), None).await?;
println!("{}", resp.result);
```

### Wallet generation

`generate_payment_wallet(chain)` creates a fresh keypair offline — no network call, no
funds — for `ChainKind::Evm`, `Svm`, or `Tempo`. The private key is returned **exactly
once**, at generation; nothing in the SDK stores or re-derives it, so persist it before
dropping the value.

```rust
use quicknode_sdk::{generate_payment_wallet, ChainKind};

let wallet = generate_payment_wallet(ChainKind::Evm)?;
println!("fund this address: {}", wallet.address);
std::fs::write("payment.key", wallet.into_key())?;  // consuming: a deliberate, one-shot read
```

### Drawdown lane (buy credits, then draw one per call)

Cheaper per call than paying per request: one signature buys a block of credits, then
each call draws a single credit. The session JWT is free to mint, so a host can
re-authenticate transparently. Persist the session between processes.

| Method | Cost | Returns |
|---|---|---|
| `payment_address()` | free, offline | the wallet address derived from the key |
| `gateway_authenticate()` | free | `GatewaySession { token, exp_unix, account_id }` |
| `gateway_credits(session)` | free | `CreditBalance { account_id, credits }` |
| `gateway_buy_credits(session, network)` | **moves funds** | the post-purchase `CreditBalance` |
| `gateway_drip(session)` | free (testnet) | `DripReceipt { account_id, transaction_hash }` |
| `gateway_drawdown_call(method, params, network, session)` | 1 credit | the JSON-RPC `result` |

```rust
let session = qn.rpc.gateway_authenticate().await?;
let balance = qn.rpc.gateway_credits(&session).await?;
println!("credits: {}", balance.credits);
let result = qn.rpc.gateway_drawdown_call("eth_blockNumber", None, "base-sepolia", &session).await?;
```

`gateway_drip` returns the **funding transaction, not a balance** — call
`gateway_credits` afterwards to read the new balance. A `token_expired` surfaces as
`SdkError::Api` with status 401/403; re-authenticate and retry that call.

### MPP channel lane (deposit once, then vouchers)

Open a payment channel by depositing into the escrow, then authorize each call with a
cumulative voucher — one `ecrecover` server-side, no on-chain transaction per call.
Requires the `payments-tempo` feature.

| Method | Cost | Returns |
|---|---|---|
| `mpp_open(deposit)` | **moves funds** | `ChannelState` — persist it |
| `mpp_top_up(channel, additional_deposit)` | **moves funds** | the updated `ChannelState` |
| `mpp_status(channel)` | **1 request unit** | `ChannelStatus { channel_id, accepted_cumulative, spent }` |
| `mpp_session_call(method, params, network, channel, new_cumulative)` | 1 request unit | the JSON-RPC `result` |
| `mpp_close(channel)` | settles on-chain | `()` — refunds the unused deposit |

```rust
let channel = qn.rpc.mpp_open(1_000_000).await?;         // persist this
let result = qn.rpc
    .mpp_session_call("eth_blockNumber", None, "base-sepolia", &channel, channel.cumulative_spent + channel.per_call)
    .await?;
// On success, advance and re-persist cumulative_spent by per_call.
```

**Things to know:**

- **Persist `ChannelState`.** The gateway exposes no read-only channel endpoint, so a
  lost local record means opening (and funding) a new channel.
- **`mpp_status` is not free.** The gateway prices every session POST as a chargeable
  request and computes the balance from the *new* spend a voucher authorizes, so the
  probe advances `cumulative_spent` by `per_call` exactly like a call. Re-persist the
  advanced total. It returns `PaymentUnsupported` before any network I/O when the
  channel has no room left for the probe.
- **The lifecycle takes no query network.** A channel is scoped by the configured pay
  network and asset, so one channel funds calls to every supported network. Only
  `mpp_session_call` takes a `network`, because it routes an RPC method.
- **Advance `cumulative_spent` only after a success.** A voucher authorizes the running
  total *after* the call; re-presenting the current high-water mark authorizes zero and
  is always refused with `insufficient-balance`.


## Error Handling

Every binding exposes a typed exception hierarchy derived from the core `SdkError`
enum (`crates/core/src/errors.rs`). Catch the base class (`SdkError`) for any SDK-originated failure, or a specific
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

Variants: pattern-match on `SdkError { Http, Api, Decode, UrlParse, Config, Rpc, PaymentUnsupported, PaymentRejected, PaymentIndeterminate }`; use `err.http_kind()` to classify `Http` into `Timeout`, `Connect`, or `Other`. The `Payment*` variants require a `payments*` feature.

```rust
// Rust
match qn.admin.show_endpoint("missing").await {
    Ok(resp) => println!("{:?}", resp.data),
    Err(SdkError::Api { status, body }) if status.as_u16() == 404 => {
        eprintln!("not found: {body}")
    }
    Err(e) if matches!(e.http_kind(), Some(HttpKind::Timeout)) => eprintln!("timed out"),
    Err(e) => eprintln!("other: {e}"),
}
```

## License

MIT
