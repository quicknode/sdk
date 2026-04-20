# Quicknode SDK

A unified SDK for building on QuickNode.

Rust SDK with Python, Node.js, and Ruby bindings.

## Table of Contents

- [Project Structure](#project-structure)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
  - [Option A — Pass config directly](#option-a--pass-config-directly)
  - [Option B — Load from environment (`from_env()`)](#option-b--load-from-environment-from_env)
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
- [Development](#development)
- [License](#license)

## Project Structure

```
sdk/
├── crates/
│   ├── core/          # Pure Rust business logic
│   ├── python/        # PyO3 bindings
│   ├── node/          # napi-rs bindings
│   └── ruby/          # magnus bindings
├── python/sdk/        # Python package with type hints
├── npm/               # Node.js package with TypeScript types
├── ruby/              # Ruby package
└── pyproject.toml     # maturin build config
```

## Installation

**Python:** `uv add quicknode-sdk`

**Node.js:** `npm install quicknode-sdk`

**Ruby:** `gem install quicknode-sdk` _(not yet published — see Development below)_

## Quick Start

Construct the SDK once, then reach into the four sub-clients (`admin`, `streams`, `webhooks`, `kvstore`). Subsequent API Reference snippets assume you have a `qn` handle from one of these blocks.

```rust
use sdk_core::{QuickNodeSdk, SdkFullConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let qn = QuickNodeSdk::from_env()?;
    let resp = qn.admin.get_endpoints(&Default::default()).await?;
    println!("{} endpoints", resp.data.len());
    Ok(())
}
```

```python
import asyncio
from sdk import QuickNodeSdk

async def main():
    qn = QuickNodeSdk.from_env()
    resp = await qn.admin.get_endpoints()
    print(f"{len(resp.data)} endpoints")

asyncio.run(main())
```

```typescript
import { QuickNodeSdk } from "quicknode-sdk";

const qn = QuickNodeSdk.fromEnv();
const resp = await qn.admin.getEndpoints();
console.log(`${resp.data.length} endpoints`);
```

```ruby
require "json"
require "quicknode_sdk"

qn = QuickNodeSdk::SDK.from_env
resp = JSON.parse(qn.admin.get_endpoints({}))
puts "#{resp["data"].length} endpoints"
```

## Configuration

There are two ways to configure the SDK.

### Option A — Pass config directly

```python
# Python
from sdk import QuickNodeSdk, SdkFullConfig, HttpConfig
qn = QuickNodeSdk(SdkFullConfig(api_key="your-key", http=HttpConfig(timeout_secs=30)))
```

```typescript
// Node.js
import { QuickNodeSdk } from "quicknode-sdk";
const qn = new QuickNodeSdk({ apiKey: "your-key", http: { timeoutSecs: 30 } });
```

```rust
// Rust
let qn = QuickNodeSdk::new(&SdkFullConfig::builder().api_key("your-key").build())?;
```

### Option B — Load from environment (`from_env()`)

```python
qn = QuickNodeSdk.from_env()
```
```typescript
const qn = QuickNodeSdk.fromEnv();
```
```ruby
qn = QuickNodeSdk::SDK.from_env
```
```rust
let qn = QuickNodeSdk::from_env()?;
```

Environment variables (prefix `QN_SDK__`, separator `__`):

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `QN_SDK__API_KEY` | yes | — | Your QuickNode API key |
| `QN_SDK__HTTP__TIMEOUT_SECS` | no | 30 | HTTP request timeout in seconds |
| `QN_SDK__HTTP__POOL_MAX_IDLE_PER_HOST` | no | — | Max idle HTTP connections per host |
| `QN_SDK__ADMIN__BASE_URL` | no | `https://api.quicknode.com/v0/` | Override admin API base URL (HTTPS, must end with `/`) |
| `QN_SDK__STREAMS__BASE_URL` | no | `https://api.quicknode.com/streams/rest/v1/` | Override streams base URL |
| `QN_SDK__WEBHOOKS__BASE_URL` | no | `https://api.quicknode.com/webhooks/rest/v1/` | Override webhooks base URL |
| `QN_SDK__KVSTORE__BASE_URL` | no | `https://api.quicknode.com/kv/rest/v1/` | Override KV store base URL |

## API Reference

Each method below shows the call pattern in Rust, Python, Node.js, and Ruby in that order. Snippets assume `qn` was already constructed via the Quick Start. Optional parameters are skipped unless showing one is needed to illustrate usage.

### Language conventions

- **Rust**: methods are `async` and return `Result<T, SdkError>`. Request structs use the [`bon`](https://docs.rs/bon) builder pattern via `::builder()`.
- **Python**: methods are `async` — call with `await`. Parameters are kwargs; responses are native `pyclass` objects with attribute access.
- **Node.js**: methods are `async` and take a single options object with camelCase keys.
- **Ruby**: methods are **blocking** (not async). Parameters are a single Hash with symbol keys. Responses that carry data are returned as **JSON strings** — wrap calls with `JSON.parse`. Unknown keys raise `ArgumentError`.

---

### Admin Client

Accessed as `qn.admin`. Manages endpoints, tags, teams, billing, usage, metrics, security, and rate limits. Backed by `https://api.quicknode.com/v0/`.

#### Endpoints

##### `get_endpoints` / `getEndpoints`

Returns a paginated list of endpoints on the account with optional search, filters (networks, statuses, labels, tags, dedicated, flat-rate), sorting, and pagination.

**Parameters** (all optional): `limit` (i32), `offset` (i32), `search` (string), `sort_by` (string), `sort_direction` (`"asc"` | `"desc"`), `networks` (string[]), `statuses` (string[]), `labels` (string[]), `dedicated` (bool), `is_flat_rate` (bool), `tag_ids` (i32[]), `tag_labels` (string[]).

**Returns**: `GetEndpointsResponse` — `{ data: Endpoint[], pagination?: Pagination }`.

```rust
let params = GetEndpointsRequest::builder()
    .limit(20)
    .sort_by("created_at".to_string())
    .sort_direction("desc".to_string())
    .build();
let resp = qn.admin.get_endpoints(&params).await?;
```

```python
resp = await qn.admin.get_endpoints(limit=20, sort_by="created_at", sort_direction="desc")
```

```typescript
const resp = await qn.admin.getEndpoints({
  limit: 20,
  sortBy: "created_at",
  sortDirection: "desc",
});
```

```ruby
resp = JSON.parse(qn.admin.get_endpoints(limit: 20, sort_by: "created_at", sort_direction: "desc"))
```

##### `create_endpoint` / `createEndpoint`

Creates a new endpoint for the given blockchain and network.

**Parameters**: `chain` (string, optional), `network` (string, optional).

**Returns**: `CreateEndpointResponse` with `data: SingleEndpoint`.

```rust
let params = CreateEndpointRequest::builder()
    .chain("ethereum".to_string())
    .network("mainnet".to_string())
    .build();
let resp = qn.admin.create_endpoint(&params).await?;
```

```python
resp = await qn.admin.create_endpoint(chain="ethereum", network="mainnet")
```

```typescript
const resp = await qn.admin.createEndpoint({ chain: "ethereum", network: "mainnet" });
```

```ruby
resp = JSON.parse(qn.admin.create_endpoint(chain: "ethereum", network: "mainnet"))
```

##### `show_endpoint` / `showEndpoint`

Fetches a single endpoint by id, including its full security configuration and rate limits.

**Parameters**: `id` (string, required).

**Returns**: `ShowEndpointResponse` with `data: SingleEndpoint`.

```rust
let resp = qn.admin.show_endpoint("ep-123").await?;
```

```python
resp = await qn.admin.show_endpoint("ep-123")
```

```typescript
const resp = await qn.admin.showEndpoint("ep-123");
```

```ruby
resp = JSON.parse(qn.admin.show_endpoint(id: "ep-123"))
```

##### `update_endpoint` / `updateEndpoint`

Updates editable fields on an endpoint. Currently supports `label`.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```rust
let params = UpdateEndpointRequest::builder().label("my label".to_string()).build();
qn.admin.update_endpoint("ep-123", &params).await?;
```

```python
await qn.admin.update_endpoint("ep-123", label="my label")
```

```typescript
await qn.admin.updateEndpoint("ep-123", { label: "my label" });
```

```ruby
qn.admin.update_endpoint(id: "ep-123", label: "my label")
```

##### `archive_endpoint` / `archiveEndpoint`

Archives an endpoint. The HTTP verb is `DELETE` but the effect is archival, not permanent deletion.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
qn.admin.archive_endpoint("ep-123").await?;
```

```python
await qn.admin.archive_endpoint("ep-123")
```

```typescript
await qn.admin.archiveEndpoint("ep-123");
```

```ruby
qn.admin.archive_endpoint(id: "ep-123")
```

##### `update_endpoint_status` / `updateEndpointStatus`

Pauses or unpauses an endpoint.

**Parameters**: `id` (string, required); body: `status` (string, required — `"active"` or `"paused"`).

**Returns**: `UpdateEndpointStatusResponse`.

```rust
let params = UpdateEndpointStatusRequest::builder().status("paused".to_string()).build();
qn.admin.update_endpoint_status("ep-123", &params).await?;
```

```python
await qn.admin.update_endpoint_status("ep-123", status="paused")
```

```typescript
await qn.admin.updateEndpointStatus("ep-123", { status: "paused" });
```

```ruby
JSON.parse(qn.admin.update_endpoint_status(id: "ep-123", status: "paused"))
```

#### Endpoint Tags

Per-endpoint tag add/remove. For account-wide tag management see [Account Tags](#account-tags).

##### `create_tag` / `createTag`

Tags an endpoint with the given label. Creates the tag on the account if it does not exist.

**Parameters**: `id` (string, required); body: `label` (string, optional).

**Returns**: nothing.

```rust
let params = CreateTagRequest::builder().label("prod".to_string()).build();
qn.admin.create_tag("ep-123", &params).await?;
```

```python
await qn.admin.create_tag("ep-123", label="prod")
```

```typescript
await qn.admin.createTag("ep-123", { label: "prod" });
```

```ruby
qn.admin.create_tag(id: "ep-123", label: "prod")
```

##### `delete_tag` / `deleteTag`

Removes a tag from a specific endpoint.

**Parameters**: `id` (endpoint id, string, required), `tag_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_tag("ep-123", "42").await?;
```

```python
await qn.admin.delete_tag("ep-123", "42")
```

```typescript
await qn.admin.deleteTag("ep-123", "42");
```

```ruby
qn.admin.delete_tag(id: "ep-123", tag_id: "42")
```

#### Teams

##### `list_teams` / `listTeams`

Lists all teams on the account.

**Parameters**: none.

**Returns**: `ListTeamsResponse` with `data: TeamSummary[]`.

```rust
let resp = qn.admin.list_teams().await?;
```

```python
resp = await qn.admin.list_teams()
```

```typescript
const resp = await qn.admin.listTeams();
```

```ruby
resp = JSON.parse(qn.admin.list_teams)
```

##### `create_team` / `createTeam`

Creates a new team.

**Parameters**: `name` (string, required).

**Returns**: `CreateTeamResponse` with `data: CreateTeamData`.

```rust
let params = CreateTeamRequest::builder().name("Payments".to_string()).build();
let resp = qn.admin.create_team(&params).await?;
```

```python
resp = await qn.admin.create_team(name="Payments")
```

```typescript
const resp = await qn.admin.createTeam({ name: "Payments" });
```

```ruby
resp = JSON.parse(qn.admin.create_team(name: "Payments"))
```

##### `get_team` / `getTeam`

Fetches team detail including pending invites.

**Parameters**: `id` (i64, required).

**Returns**: `GetTeamResponse` with `data: TeamDetail`.

```rust
let resp = qn.admin.get_team(42).await?;
```

```python
resp = await qn.admin.get_team(42)
```

```typescript
const resp = await qn.admin.getTeam(42);
```

```ruby
resp = JSON.parse(qn.admin.get_team(id: 42))
```

##### `delete_team` / `deleteTeam`

Deletes a team.

**Parameters**: `id` (i64, required).

**Returns**: `DeleteTeamResponse`.

```rust
qn.admin.delete_team(42).await?;
```

```python
await qn.admin.delete_team(42)
```

```typescript
await qn.admin.deleteTeam(42);
```

```ruby
qn.admin.delete_team(id: 42)
```

##### `list_team_endpoints` / `listTeamEndpoints`

Lists endpoints accessible to a team.

**Parameters**: `id` (i64, required).

**Returns**: `ListTeamEndpointsResponse` with `data: TeamEndpoint[]`.

```rust
let resp = qn.admin.list_team_endpoints(42).await?;
```

```python
resp = await qn.admin.list_team_endpoints(42)
```

```typescript
const resp = await qn.admin.listTeamEndpoints(42);
```

```ruby
resp = JSON.parse(qn.admin.list_team_endpoints(id: 42))
```

##### `update_team_endpoints` / `updateTeamEndpoints`

Replaces the set of endpoints associated with a team. Pass an empty array to remove all.

**Parameters**: `id` (i64, required); body: `endpoint_ids` (string[], required).

**Returns**: `UpdateTeamEndpointsResponse`.

```rust
let params = UpdateTeamEndpointsRequest::builder()
    .endpoint_ids(vec!["ep-123".to_string(), "ep-456".to_string()])
    .build();
qn.admin.update_team_endpoints(42, &params).await?;
```

```python
await qn.admin.update_team_endpoints(42, endpoint_ids=["ep-123", "ep-456"])
```

```typescript
await qn.admin.updateTeamEndpoints(42, { endpointIds: ["ep-123", "ep-456"] });
```

```ruby
qn.admin.update_team_endpoints(id: 42, endpoint_ids: ["ep-123", "ep-456"])
```

##### `invite_team_member` / `inviteTeamMember`

Invites a user to a team. Existing users only need `email`; new users require `full_name` and `role`.

**Parameters**: `id` (i64, required); body: `email` (string, required), `full_name` (string, optional), `role` (string, optional — `admin` | `viewer` | `billing`).

**Returns**: `InviteTeamMemberResponse`.

```rust
let params = InviteTeamMemberRequest::builder()
    .email("alice@example.com".to_string())
    .role("viewer".to_string())
    .build();
qn.admin.invite_team_member(42, &params).await?;
```

```python
await qn.admin.invite_team_member(42, email="alice@example.com", role="viewer")
```

```typescript
await qn.admin.inviteTeamMember(42, { email: "alice@example.com", role: "viewer" });
```

```ruby
qn.admin.invite_team_member(id: 42, email: "alice@example.com", role: "viewer")
```

##### `remove_team_member` / `removeTeamMember`

Removes a user from a team.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `RemoveTeamMemberResponse`.

```rust
qn.admin.remove_team_member(42, 7).await?;
```

```python
await qn.admin.remove_team_member(42, 7)
```

```typescript
await qn.admin.removeTeamMember(42, 7);
```

```ruby
qn.admin.remove_team_member(id: 42, user_id: 7)
```

##### `resend_team_invite` / `resendTeamInvite`

Re-sends a pending team invitation.

**Parameters**: `id` (team id, i64, required), `user_id` (i64, required).

**Returns**: `ResendTeamInviteResponse`.

```rust
qn.admin.resend_team_invite(42, 7).await?;
```

```python
await qn.admin.resend_team_invite(42, 7)
```

```typescript
await qn.admin.resendTeamInvite(42, 7);
```

```ruby
qn.admin.resend_team_invite(id: 42, user_id: 7)
```

#### Usage

All usage methods accept optional `start_time` and `end_time` Unix timestamps. Omit both for account-to-date totals.

##### `get_usage` / `getUsage`

Aggregate account usage for a time window.

**Returns**: `GetUsageResponse` with `data: UsageData` (`credits_used`, `credits_remaining`, `limit`, `overages`, `start_time`, `end_time`).

```rust
let resp = qn.admin.get_usage(&GetUsageRequest::default()).await?;
```

```python
resp = await qn.admin.get_usage()
```

```typescript
const resp = await qn.admin.getUsage();
```

```ruby
resp = JSON.parse(qn.admin.get_usage({}))
```

##### `get_usage_by_endpoint` / `getUsageByEndpoint`

Per-endpoint usage breakdown.

**Returns**: `GetUsageByEndpointResponse` with `data.endpoints: EndpointUsage[]`.

```rust
let resp = qn.admin.get_usage_by_endpoint(&GetUsageRequest::default()).await?;
```

```python
resp = await qn.admin.get_usage_by_endpoint()
```

```typescript
const resp = await qn.admin.getUsageByEndpoint();
```

```ruby
resp = JSON.parse(qn.admin.get_usage_by_endpoint({}))
```

##### `get_usage_by_method` / `getUsageByMethod`

Per-RPC-method usage breakdown.

**Returns**: `GetUsageByMethodResponse` with `data.methods: MethodUsage[]`.

```rust
let resp = qn.admin.get_usage_by_method(&GetUsageRequest::default()).await?;
```

```python
resp = await qn.admin.get_usage_by_method()
```

```typescript
const resp = await qn.admin.getUsageByMethod();
```

```ruby
resp = JSON.parse(qn.admin.get_usage_by_method({}))
```

##### `get_usage_by_chain` / `getUsageByChain`

Per-chain usage breakdown.

**Returns**: `GetUsageByChainResponse` with `data.chains: ChainUsage[]`.

```rust
let resp = qn.admin.get_usage_by_chain(&GetUsageRequest::default()).await?;
```

```python
resp = await qn.admin.get_usage_by_chain()
```

```typescript
const resp = await qn.admin.getUsageByChain();
```

```ruby
resp = JSON.parse(qn.admin.get_usage_by_chain({}))
```

##### `get_usage_by_tag` / `getUsageByTag`

Per-tag usage breakdown.

**Returns**: `GetUsageByTagResponse` with `data.tags: TagUsage[]`.

```rust
let resp = qn.admin.get_usage_by_tag(&GetUsageRequest::default()).await?;
```

```python
resp = await qn.admin.get_usage_by_tag()
```

```typescript
const resp = await qn.admin.getUsageByTag();
```

```ruby
resp = JSON.parse(qn.admin.get_usage_by_tag({}))
```

#### Logs

##### `get_endpoint_logs` / `getEndpointLogs`

Fetches a page of request logs for an endpoint. Set `include_details=true` for full request/response payloads (truncated at 2 KB each).

**Parameters**: `id` (endpoint id, required); body: `from` (string timestamp, required), `to` (string timestamp, required), `include_details` (bool, optional), `limit` (i32, optional), `next_at` (string cursor, optional).

**Returns**: `GetEndpointLogsResponse` — `{ data: EndpointLog[], next_at?: string }`.

```rust
let params = GetEndpointLogsRequest::builder()
    .from("2026-04-01T00:00:00Z".to_string())
    .to("2026-04-02T00:00:00Z".to_string())
    .limit(100)
    .build();
let resp = qn.admin.get_endpoint_logs("ep-123", &params).await?;
```

```python
resp = await qn.admin.get_endpoint_logs(
    "ep-123",
    from_time="2026-04-01T00:00:00Z",
    to_time="2026-04-02T00:00:00Z",
    limit=100,
)
```

```typescript
const resp = await qn.admin.getEndpointLogs("ep-123", {
  from: "2026-04-01T00:00:00Z",
  to: "2026-04-02T00:00:00Z",
  limit: 100,
});
```

```ruby
resp = JSON.parse(qn.admin.get_endpoint_logs(
  id: "ep-123",
  from_time: "2026-04-01T00:00:00Z",
  to_time: "2026-04-02T00:00:00Z",
  limit: 100
))
```

##### `get_log_details` / `getLogDetails`

Returns the full request/response payloads for a single log entry.

**Parameters**: `id` (endpoint id, required), `request_id` (log request uuid, required).

**Returns**: `GetLogDetailsResponse` with `data: LogDetails`.

```rust
let resp = qn.admin.get_log_details("ep-123", "req-abc").await?;
```

```python
resp = await qn.admin.get_log_details("ep-123", "req-abc")
```

```typescript
const resp = await qn.admin.getLogDetails("ep-123", "req-abc");
```

```ruby
resp = JSON.parse(qn.admin.get_log_details(id: "ep-123", request_id: "req-abc"))
```

#### Endpoint Security

##### `get_endpoint_security` / `getEndpointSecurity`

Returns the full security configuration for an endpoint: tokens, JWTs, referrers, domain masks, IPs, request filters, and their per-feature toggles.

**Parameters**: `id` (string, required).

**Returns**: `GetEndpointSecurityResponse` with `data: EndpointSecurity`.

```rust
let resp = qn.admin.get_endpoint_security("ep-123").await?;
```

```python
resp = await qn.admin.get_endpoint_security("ep-123")
```

```typescript
const resp = await qn.admin.getEndpointSecurity("ep-123");
```

```ruby
resp = JSON.parse(qn.admin.get_endpoint_security(id: "ep-123"))
```

#### Security Options

##### `get_security_options` / `getSecurityOptions`

Returns the list of security features and their enabled state for an endpoint.

**Parameters**: `id` (string, required).

**Returns**: `GetSecurityOptionsResponse` with `data: SecurityOption[]`.

```rust
let resp = qn.admin.get_security_options("ep-123").await?;
```

```python
resp = await qn.admin.get_security_options("ep-123")
```

```typescript
const resp = await qn.admin.getSecurityOptions("ep-123");
```

```ruby
resp = JSON.parse(qn.admin.get_security_options(id: "ep-123"))
```

##### `update_security_options` / `updateSecurityOptions`

Enables or disables individual security features. Each field accepts `"enabled"` or `"disabled"`.

**Parameters**: `id` (string, required); `options`: `SecurityOptionsUpdate` (`tokens`, `referrers`, `jwts`, `ips`, `domain_masks`, `hsts`, `cors`, `request_filters`, `ip_custom_header`).

**Returns**: `UpdateSecurityOptionsResponse` with updated `SecurityOption[]`.

```rust
let options = SecurityOptionsUpdate::builder()
    .tokens("enabled".to_string())
    .jwts("disabled".to_string())
    .build();
let params = UpdateSecurityOptionsRequest { options };
qn.admin.update_security_options("ep-123", &params).await?;
```

```python
await qn.admin.update_security_options(
    "ep-123",
    tokens="enabled",
    jwts="disabled",
)
```

```typescript
await qn.admin.updateSecurityOptions("ep-123", {
  options: { tokens: "enabled", jwts: "disabled" },
});
```

```ruby
qn.admin.update_security_options(id: "ep-123", tokens: "enabled", jwts: "disabled")
```

#### Tokens

##### `create_token` / `createToken`

Generates a new auth token on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```rust
qn.admin.create_token("ep-123").await?;
```

```python
await qn.admin.create_token("ep-123")
```

```typescript
await qn.admin.createToken("ep-123");
```

```ruby
qn.admin.create_token(id: "ep-123")
```

##### `delete_token` / `deleteToken`

Revokes a token on an endpoint.

**Parameters**: `id` (endpoint id, required), `token_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_token("ep-123", "tok-1").await?;
```

```python
await qn.admin.delete_token("ep-123", "tok-1")
```

```typescript
await qn.admin.deleteToken("ep-123", "tok-1");
```

```ruby
qn.admin.delete_token(id: "ep-123", token_id: "tok-1")
```

#### Referrers

##### `create_referrer` / `createReferrer`

Whitelists a referrer URL or domain on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `referrer` (string, optional).

**Returns**: nothing.

```rust
let params = CreateReferrerRequest::builder().referrer("example.com".to_string()).build();
qn.admin.create_referrer("ep-123", &params).await?;
```

```python
await qn.admin.create_referrer("ep-123", referrer="example.com")
```

```typescript
await qn.admin.createReferrer("ep-123", { referrer: "example.com" });
```

```ruby
qn.admin.create_referrer(id: "ep-123", referrer: "example.com")
```

##### `delete_referrer` / `deleteReferrer`

Removes a referrer from the whitelist.

**Parameters**: `id` (endpoint id, required), `referrer_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_referrer("ep-123", "ref-1").await?;
```

```python
await qn.admin.delete_referrer("ep-123", "ref-1")
```

```typescript
await qn.admin.deleteReferrer("ep-123", "ref-1");
```

```ruby
qn.admin.delete_referrer(id: "ep-123", referrer_id: "ref-1")
```

#### IPs

##### `create_ip` / `createIp`

Whitelists an IP address on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `ip` (string, optional).

**Returns**: nothing.

```rust
let params = CreateIpRequest::builder().ip("198.51.100.7".to_string()).build();
qn.admin.create_ip("ep-123", &params).await?;
```

```python
await qn.admin.create_ip("ep-123", ip="198.51.100.7")
```

```typescript
await qn.admin.createIp("ep-123", { ip: "198.51.100.7" });
```

```ruby
qn.admin.create_ip(id: "ep-123", ip: "198.51.100.7")
```

##### `delete_ip` / `deleteIp`

Removes an IP from the whitelist.

**Parameters**: `id` (endpoint id, required), `ip_id` (string, required).

**Returns**: `DeleteBoolResponse`.

```rust
qn.admin.delete_ip("ep-123", "ip-1").await?;
```

```python
await qn.admin.delete_ip("ep-123", "ip-1")
```

```typescript
await qn.admin.deleteIp("ep-123", "ip-1");
```

```ruby
resp = JSON.parse(qn.admin.delete_ip(id: "ep-123", ip_id: "ip-1"))
```

#### Domain Masks

##### `create_domain_mask` / `createDomainMask`

Adds a custom domain mask to an endpoint.

**Parameters**: `id` (endpoint id, required); body: `domain_mask` (string, optional).

**Returns**: nothing.

```rust
let params = CreateDomainMaskRequest::builder()
    .domain_mask("rpc.example.com".to_string())
    .build();
qn.admin.create_domain_mask("ep-123", &params).await?;
```

```python
await qn.admin.create_domain_mask("ep-123", domain_mask="rpc.example.com")
```

```typescript
await qn.admin.createDomainMask("ep-123", { domainMask: "rpc.example.com" });
```

```ruby
qn.admin.create_domain_mask(id: "ep-123", domain_mask: "rpc.example.com")
```

##### `delete_domain_mask` / `deleteDomainMask`

Removes a domain mask.

**Parameters**: `id` (endpoint id, required), `domain_mask_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_domain_mask("ep-123", "dm-1").await?;
```

```python
await qn.admin.delete_domain_mask("ep-123", "dm-1")
```

```typescript
await qn.admin.deleteDomainMask("ep-123", "dm-1");
```

```ruby
qn.admin.delete_domain_mask(id: "ep-123", domain_mask_id: "dm-1")
```

#### JWTs

##### `create_jwt` / `createJwt`

Configures JWT validation on an endpoint.

**Parameters**: `id` (endpoint id, required); body: `public_key` (string, optional), `kid` (string, optional), `name` (string, optional).

**Returns**: nothing.

```rust
let params = CreateJwtRequest::builder()
    .public_key("-----BEGIN PUBLIC KEY-----\n...".to_string())
    .kid("key-1".to_string())
    .name("primary".to_string())
    .build();
qn.admin.create_jwt("ep-123", &params).await?;
```

```python
await qn.admin.create_jwt(
    "ep-123",
    public_key="-----BEGIN PUBLIC KEY-----\n...",
    kid="key-1",
    name="primary",
)
```

```typescript
await qn.admin.createJwt("ep-123", {
  publicKey: "-----BEGIN PUBLIC KEY-----\n...",
  kid: "key-1",
  name: "primary",
});
```

```ruby
qn.admin.create_jwt(
  id: "ep-123",
  public_key: "-----BEGIN PUBLIC KEY-----\n...",
  kid: "key-1",
  name: "primary"
)
```

##### `delete_jwt` / `deleteJwt`

Removes a JWT configuration.

**Parameters**: `id` (endpoint id, required), `jwt_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_jwt("ep-123", "jwt-1").await?;
```

```python
await qn.admin.delete_jwt("ep-123", "jwt-1")
```

```typescript
await qn.admin.deleteJwt("ep-123", "jwt-1");
```

```ruby
qn.admin.delete_jwt(id: "ep-123", jwt_id: "jwt-1")
```

#### Request Filters

Whitelist specific RPC methods on an endpoint. Requests for methods not on the list are blocked when the feature is enabled.

##### `create_request_filter` / `createRequestFilter`

**Parameters**: `id` (endpoint id, required); body: `method` (string[], optional). Ruby's Hash key is `methods` (plural).

**Returns**: `CreateRequestFilterResponse` with `data.id`.

```rust
let params = CreateRequestFilterRequest::builder()
    .method(vec!["eth_blockNumber".to_string(), "eth_getBalance".to_string()])
    .build();
let resp = qn.admin.create_request_filter("ep-123", &params).await?;
```

```python
resp = await qn.admin.create_request_filter(
    "ep-123",
    method=["eth_blockNumber", "eth_getBalance"],
)
```

```typescript
const resp = await qn.admin.createRequestFilter("ep-123", {
  method: ["eth_blockNumber", "eth_getBalance"],
});
```

```ruby
resp = JSON.parse(qn.admin.create_request_filter(
  id: "ep-123",
  methods: ["eth_blockNumber", "eth_getBalance"]
))
```

##### `update_request_filter` / `updateRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required); body: `method` (string[], optional). Ruby's Hash keys are `request_filter_id` and `methods` (plural).

**Returns**: nothing.

```rust
let params = UpdateRequestFilterRequest::builder()
    .method(vec!["eth_call".to_string()])
    .build();
qn.admin.update_request_filter("ep-123", "f-1", &params).await?;
```

```python
await qn.admin.update_request_filter("ep-123", "f-1", method=["eth_call"])
```

```typescript
await qn.admin.updateRequestFilter("ep-123", "f-1", { method: ["eth_call"] });
```

```ruby
qn.admin.update_request_filter(id: "ep-123", request_filter_id: "f-1", methods: ["eth_call"])
```

##### `delete_request_filter` / `deleteRequestFilter`

**Parameters**: `id` (endpoint id, required), `request_filter_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_request_filter("ep-123", "f-1").await?;
```

```python
await qn.admin.delete_request_filter("ep-123", "f-1")
```

```typescript
await qn.admin.deleteRequestFilter("ep-123", "f-1");
```

```ruby
qn.admin.delete_request_filter(id: "ep-123", request_filter_id: "f-1")
```

#### Multichain

##### `enable_multichain` / `enableMultichain`

Enables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```rust
qn.admin.enable_multichain("ep-123").await?;
```

```python
await qn.admin.enable_multichain("ep-123")
```

```typescript
await qn.admin.enableMultichain("ep-123");
```

```ruby
qn.admin.enable_multichain(id: "ep-123")
```

##### `disable_multichain` / `disableMultichain`

Disables multichain on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: nothing.

```rust
qn.admin.disable_multichain("ep-123").await?;
```

```python
await qn.admin.disable_multichain("ep-123")
```

```typescript
await qn.admin.disableMultichain("ep-123");
```

```ruby
qn.admin.disable_multichain(id: "ep-123")
```

#### IP Custom Headers

##### `create_or_update_ip_custom_header` / `createOrUpdateIpCustomHeader`

Sets the custom header used to identify the client IP (e.g. when traffic is proxied).

**Parameters**: `id` (endpoint id, required); body: `header_name` (string, required).

**Returns**: `CreateOrUpdateIpCustomHeaderResponse` with `data.header_name`.

```rust
let params = CreateOrUpdateIpCustomHeaderRequest::builder()
    .header_name("X-Forwarded-For".to_string())
    .build();
qn.admin.create_or_update_ip_custom_header("ep-123", &params).await?;
```

```python
await qn.admin.create_or_update_ip_custom_header("ep-123", header_name="X-Forwarded-For")
```

```typescript
await qn.admin.createOrUpdateIpCustomHeader("ep-123", { headerName: "X-Forwarded-For" });
```

```ruby
JSON.parse(qn.admin.create_or_update_ip_custom_header(
  id: "ep-123",
  header_name: "X-Forwarded-For"
))
```

##### `delete_ip_custom_header` / `deleteIpCustomHeader`

Removes the custom IP header configuration.

**Parameters**: `id` (endpoint id, required).

**Returns**: `DeleteBoolResponse`.

```rust
qn.admin.delete_ip_custom_header("ep-123").await?;
```

```python
await qn.admin.delete_ip_custom_header("ep-123")
```

```typescript
await qn.admin.deleteIpCustomHeader("ep-123");
```

```ruby
JSON.parse(qn.admin.delete_ip_custom_header(id: "ep-123"))
```

#### Method Rate Limits

##### `get_method_rate_limits` / `getMethodRateLimits`

Lists method-level rate limiters configured on an endpoint.

**Parameters**: `id` (endpoint id, required).

**Returns**: `GetMethodRateLimitsResponse` with `data.rate_limiters: MethodRateLimiter[]`.

```rust
let resp = qn.admin.get_method_rate_limits("ep-123").await?;
```

```python
resp = await qn.admin.get_method_rate_limits("ep-123")
```

```typescript
const resp = await qn.admin.getMethodRateLimits("ep-123");
```

```ruby
resp = JSON.parse(qn.admin.get_method_rate_limits(id: "ep-123"))
```

##### `create_method_rate_limit` / `createMethodRateLimit`

Creates a new method-level rate limiter.

**Parameters**: `id` (endpoint id, required); body: `interval` (string, e.g. `"second"`), `methods` (string[]), `rate` (i32).

**Returns**: `CreateMethodRateLimitResponse` with `data: MethodRateLimiter`.

```rust
let params = CreateMethodRateLimitRequest::builder()
    .interval("second".to_string())
    .methods(vec!["eth_call".to_string()])
    .rate(10)
    .build();
let resp = qn.admin.create_method_rate_limit("ep-123", &params).await?;
```

```python
resp = await qn.admin.create_method_rate_limit(
    "ep-123",
    interval="second",
    methods=["eth_call"],
    rate=10,
)
```

```typescript
const resp = await qn.admin.createMethodRateLimit("ep-123", {
  interval: "second",
  methods: ["eth_call"],
  rate: 10,
});
```

```ruby
resp = JSON.parse(qn.admin.create_method_rate_limit(
  id: "ep-123",
  interval: "second",
  methods: ["eth_call"],
  rate: 10
))
```

##### `update_method_rate_limit` / `updateMethodRateLimit`

Updates an existing rate limiter. Only provided fields change.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required); body: `methods` (string[], optional), `status` (`"enabled"` | `"disabled"`, optional), `rate` (i32, optional).

**Returns**: `UpdateMethodRateLimitResponse`.

```rust
let params = UpdateMethodRateLimitRequest::builder().rate(50).build();
qn.admin.update_method_rate_limit("ep-123", "rl-1", &params).await?;
```

```python
await qn.admin.update_method_rate_limit("ep-123", "rl-1", rate=50)
```

```typescript
await qn.admin.updateMethodRateLimit("ep-123", "rl-1", { rate: 50 });
```

```ruby
JSON.parse(qn.admin.update_method_rate_limit(id: "ep-123", method_rate_limit_id: "rl-1", rate: 50))
```

##### `delete_method_rate_limit` / `deleteMethodRateLimit`

Deletes a rate limiter.

**Parameters**: `id` (endpoint id, required), `method_rate_limit_id` (string, required).

**Returns**: nothing.

```rust
qn.admin.delete_method_rate_limit("ep-123", "rl-1").await?;
```

```python
await qn.admin.delete_method_rate_limit("ep-123", "rl-1")
```

```typescript
await qn.admin.deleteMethodRateLimit("ep-123", "rl-1");
```

```ruby
qn.admin.delete_method_rate_limit(id: "ep-123", method_rate_limit_id: "rl-1")
```

#### Endpoint Rate Limits

##### `update_rate_limits` / `updateRateLimits`

Updates the endpoint-level RPS / RPM / RPD caps.

**Parameters**: `id` (endpoint id, required); `rate_limits`: `RateLimitSettings` (`rps`, `rpm`, `rpd`, all optional).

**Returns**: nothing.

```rust
let rate_limits = RateLimitSettings::builder().rps(100).rpm(5000).build();
let params = UpdateRateLimitsRequest { rate_limits };
qn.admin.update_rate_limits("ep-123", &params).await?;
```

```python
await qn.admin.update_rate_limits("ep-123", rps=100, rpm=5000)
```

```typescript
await qn.admin.updateRateLimits("ep-123", { rateLimits: { rps: 100, rpm: 5000 } });
```

```ruby
qn.admin.update_rate_limits(id: "ep-123", rps: 100, rpm: 5000)
```

#### Metrics

##### `get_endpoint_metrics` / `getEndpointMetrics`

Returns metric series for an endpoint over a time period.

**Parameters**: `id` (endpoint id, required); body: `period` (`"hour"` | `"day"` | `"week"` | `"month"`), `metric` (e.g. `"method_calls_over_time"`, `"response_status_breakdown"`).

**Returns**: `GetEndpointMetricsResponse` with `data: EndpointMetric[]`.

```rust
let params = GetEndpointMetricsRequest {
    period: "day".to_string(),
    metric: "method_calls_over_time".to_string(),
};
let resp = qn.admin.get_endpoint_metrics("ep-123", &params).await?;
```

```python
resp = await qn.admin.get_endpoint_metrics(
    "ep-123",
    period="day",
    metric="method_calls_over_time",
)
```

```typescript
const resp = await qn.admin.getEndpointMetrics("ep-123", {
  period: "day",
  metric: "method_calls_over_time",
});
```

```ruby
resp = JSON.parse(qn.admin.get_endpoint_metrics(
  id: "ep-123",
  period: "day",
  metric: "method_calls_over_time"
))
```

##### `get_account_metrics` / `getAccountMetrics`

Returns account-level metric series. Supports an optional `percentile` (e.g. `"p50"`, `"p95"`, `"p99"`) for latency metrics.

**Parameters**: `period` (required), `metric` (required), `percentile` (string, optional).

**Returns**: `GetAccountMetricsResponse` with `data: EndpointMetric[]`.

```rust
let params = GetAccountMetricsRequest {
    period: "day".to_string(),
    metric: "credits_over_time".to_string(),
    percentile: None,
};
let resp = qn.admin.get_account_metrics(&params).await?;
```

```python
resp = await qn.admin.get_account_metrics(period="day", metric="credits_over_time")
```

```typescript
const resp = await qn.admin.getAccountMetrics({
  period: "day",
  metric: "credits_over_time",
});
```

```ruby
resp = JSON.parse(qn.admin.get_account_metrics(period: "day", metric: "credits_over_time"))
```

#### Chains

##### `list_chains` / `listChains`

Lists the blockchains supported by QuickNode along with their networks.

**Parameters**: none.

**Returns**: `ListChainsResponse` with `data: Chain[]`.

```rust
let resp = qn.admin.list_chains().await?;
```

```python
resp = await qn.admin.list_chains()
```

```typescript
const resp = await qn.admin.listChains();
```

```ruby
resp = JSON.parse(qn.admin.list_chains)
```

#### Billing

##### `list_invoices` / `listInvoices`

Lists invoices on the account.

**Parameters**: none.

**Returns**: `ListInvoicesResponse` with `data.invoices: Invoice[]`.

```rust
let resp = qn.admin.list_invoices().await?;
```

```python
resp = await qn.admin.list_invoices()
```

```typescript
const resp = await qn.admin.listInvoices();
```

```ruby
resp = JSON.parse(qn.admin.list_invoices)
```

##### `list_payments` / `listPayments`

Lists payments on the account.

**Parameters**: none.

**Returns**: `ListPaymentsResponse` with `data.payments: Payment[]`.

```rust
let resp = qn.admin.list_payments().await?;
```

```python
resp = await qn.admin.list_payments()
```

```typescript
const resp = await qn.admin.listPayments();
```

```ruby
resp = JSON.parse(qn.admin.list_payments)
```

#### Bulk Operations

##### `bulk_update_endpoint_status` / `bulkUpdateEndpointStatus`

Activates or pauses many endpoints at once.

**Parameters**: `ids` (string[], required), `status` (`"active"` | `"paused"`, required).

**Returns**: `BulkUpdateEndpointStatusResponse` with per-endpoint `results`.

```rust
let params = BulkUpdateEndpointStatusRequest::builder()
    .ids(vec!["ep-1".to_string(), "ep-2".to_string()])
    .status("paused".to_string())
    .build();
let resp = qn.admin.bulk_update_endpoint_status(&params).await?;
```

```python
resp = await qn.admin.bulk_update_endpoint_status(ids=["ep-1", "ep-2"], status="paused")
```

```typescript
const resp = await qn.admin.bulkUpdateEndpointStatus({
  ids: ["ep-1", "ep-2"],
  status: "paused",
});
```

```ruby
resp = JSON.parse(qn.admin.bulk_update_endpoint_status(ids: ["ep-1", "ep-2"], status: "paused"))
```

##### `bulk_add_tag` / `bulkAddTag`

Applies a tag (created if missing) to many endpoints at once.

**Parameters**: `ids` (string[], required), `label` (string, required).

**Returns**: `BulkAddTagResponse`.

```rust
let params = BulkAddTagRequest::builder()
    .ids(vec!["ep-1".to_string(), "ep-2".to_string()])
    .label("prod".to_string())
    .build();
let resp = qn.admin.bulk_add_tag(&params).await?;
```

```python
resp = await qn.admin.bulk_add_tag(ids=["ep-1", "ep-2"], label="prod")
```

```typescript
const resp = await qn.admin.bulkAddTag({ ids: ["ep-1", "ep-2"], label: "prod" });
```

```ruby
resp = JSON.parse(qn.admin.bulk_add_tag(ids: ["ep-1", "ep-2"], label: "prod"))
```

##### `bulk_remove_tag` / `bulkRemoveTag`

Removes a tag from many endpoints at once.

**Parameters**: `ids` (string[], required), `tag_id` (i32, required).

**Returns**: `BulkRemoveTagResponse`.

```rust
let params = BulkRemoveTagRequest::builder()
    .ids(vec!["ep-1".to_string(), "ep-2".to_string()])
    .tag_id(42)
    .build();
let resp = qn.admin.bulk_remove_tag(&params).await?;
```

```python
resp = await qn.admin.bulk_remove_tag(ids=["ep-1", "ep-2"], tag_id=42)
```

```typescript
const resp = await qn.admin.bulkRemoveTag({ ids: ["ep-1", "ep-2"], tagId: 42 });
```

```ruby
resp = JSON.parse(qn.admin.bulk_remove_tag(ids: ["ep-1", "ep-2"], tag_id: 42))
```

#### Account Tags

##### `list_tags` / `listTags`

Lists every tag on the account along with usage counts.

**Parameters**: none.

**Returns**: `ListTagsResponse` with `data.tags: AccountTag[]`.

```rust
let resp = qn.admin.list_tags().await?;
```

```python
resp = await qn.admin.list_tags()
```

```typescript
const resp = await qn.admin.listTags();
```

```ruby
resp = JSON.parse(qn.admin.list_tags)
```

##### `rename_tag` / `renameTag`

Renames an account-level tag.

**Parameters**: `tag_id` (i32, required); body: `label` (string, required).

**Returns**: `RenameTagResponse` with updated `AccountTag`.

```rust
let params = RenameTagRequest::builder().label("staging".to_string()).build();
let resp = qn.admin.rename_tag(42, &params).await?;
```

```python
resp = await qn.admin.rename_tag(42, label="staging")
```

```typescript
const resp = await qn.admin.renameTag(42, { label: "staging" });
```

```ruby
resp = JSON.parse(qn.admin.rename_tag(tag_id: 42, label: "staging"))
```

##### `delete_account_tag` / `deleteAccountTag`

Deletes a tag from the account. The tag must first be removed from any endpoints using it.

**Parameters**: `id` (i32, required).

**Returns**: `DeleteAccountTagResponse`.

```rust
qn.admin.delete_account_tag(42).await?;
```

```python
await qn.admin.delete_account_tag(42)
```

```typescript
await qn.admin.deleteAccountTag(42);
```

```ruby
JSON.parse(qn.admin.delete_account_tag(id: 42))
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
- **Ruby**: factory methods on `QuickNodeSdk::DestinationAttributes`, e.g. `QuickNodeSdk::DestinationAttributes.webhook(url: ..., ...)`.

#### Streams methods

##### `create_stream` / `createStream`

Creates a new stream that delivers filtered data to the configured destination. Start from a specific block for backfills or from the tip for real-time streaming. Supports filters, reorg handling, distance-from-tip, elastic batching, notification emails, and extra destinations.

**Parameters**: `CreateStreamParams` — required: `name`, `region`, `network`, `dataset`, `start_range` (i64), `end_range` (i64, `-1` = follow tip), `destination_attributes`, `plan`, `threshold_fetch_buffer`. Common optional fields: `dataset_batch_size`, `include_stream_metadata`, `fix_block_reorgs`, `keep_distance_from_tip`, `elastic_batch_enabled`, `filter_function`, `filter_language`, `status`, `notification_email`, `extra_destinations`.

**Returns**: `Stream`.

```rust
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

```python
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

```typescript
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

```ruby
dest = QuickNodeSdk::DestinationAttributes.webhook(
  url: "https://webhook.site/...",
  max_retry: 3,
  retry_interval_sec: 1,
  post_timeout_sec: 10,
  compression: "none"
)
stream = JSON.parse(qn.streams.create_stream(
  name: "My Stream",
  network: "ethereum-mainnet",
  dataset: "block",
  region: "usa_east",
  start_range: 24691804,
  end_range: 24691904,
  destination_attributes: dest,
  plan: "growth_plan",
  threshold_fetch_buffer: 1000,
  status: "active"
))
```

##### `list_streams` / `listStreams`

Paginated list of streams on the account.

**Parameters** (all optional): `offset` (i64), `limit` (i64), `order_by` (string), `order_direction` (`"asc"` | `"desc"`), `stream_type` (string).

**Returns**: `ListStreamsResponse` with `data: Stream[]` and `page_info`.

```rust
let resp = qn.streams.list_streams(&ListStreamsParams::default()).await?;
```

```python
resp = await qn.streams.list_streams()
```

```typescript
const resp = await qn.streams.listStreams();
```

```ruby
resp = JSON.parse(qn.streams.list_streams({}))
```

##### `get_stream` / `getStream`

Fetches one stream by id.

**Parameters**: `id` (string, required).

**Returns**: `Stream`.

```rust
let stream = qn.streams.get_stream("stream-id").await?;
```

```python
stream = await qn.streams.get_stream("stream-id")
```

```typescript
const stream = await qn.streams.getStream("stream-id");
```

```ruby
stream = JSON.parse(qn.streams.get_stream(id: "stream-id"))
```

##### `update_stream` / `updateStream`

Partially updates a stream. Omitted fields are left unchanged.

**Parameters**: `id` (string, required); body: any field from `CreateStreamParams` (all optional).

**Returns**: updated `Stream`.

```rust
let params = UpdateStreamParams {
    name: Some("Renamed".to_string()),
    ..Default::default()
};
let stream = qn.streams.update_stream("stream-id", &params).await?;
```

```python
stream = await qn.streams.update_stream("stream-id", name="Renamed")
```

```typescript
const stream = await qn.streams.updateStream("stream-id", { name: "Renamed" });
```

```ruby
stream = JSON.parse(qn.streams.update_stream(id: "stream-id", name: "Renamed"))
```

##### `delete_stream` / `deleteStream`

Deletes one stream by id.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
qn.streams.delete_stream("stream-id").await?;
```

```python
await qn.streams.delete_stream("stream-id")
```

```typescript
await qn.streams.deleteStream("stream-id");
```

```ruby
qn.streams.delete_stream(id: "stream-id")
```

##### `delete_all_streams` / `deleteAllStreams`

Deletes every stream on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```rust
qn.streams.delete_all_streams().await?;
```

```python
await qn.streams.delete_all_streams()
```

```typescript
await qn.streams.deleteAllStreams();
```

```ruby
qn.streams.delete_all_streams
```

##### `activate_stream` / `activateStream`

Resumes delivery on a stream from its current position.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
qn.streams.activate_stream("stream-id").await?;
```

```python
await qn.streams.activate_stream("stream-id")
```

```typescript
await qn.streams.activateStream("stream-id");
```

```ruby
qn.streams.activate_stream(id: "stream-id")
```

##### `pause_stream` / `pauseStream`

Halts delivery on a stream.

**Parameters**: `id` (string, required).

**Returns**: nothing.

```rust
qn.streams.pause_stream("stream-id").await?;
```

```python
await qn.streams.pause_stream("stream-id")
```

```typescript
await qn.streams.pauseStream("stream-id");
```

```ruby
qn.streams.pause_stream(id: "stream-id")
```

##### `test_filter` / `testFilter`

Runs a filter function against a block so it can be validated before being attached to a live stream.

**Parameters**: `network` (string, required), `dataset` (`StreamDataset`, required), `block` (string, required), `filter_function` (string, optional), `filter_language` (`FilterLanguage`, optional), `address_book_config` (optional).

**Returns**: `TestFilterResponse` with `result` and `logs`.

```rust
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

```python
resp = await qn.streams.test_filter(
    network="ethereum-mainnet",
    dataset="block",
    block="17811625",
)
```

```typescript
import { StreamDataset } from "quicknode-sdk";

const resp = await qn.streams.testFilter({
  network: "ethereum-mainnet",
  dataset: StreamDataset.Block,
  block: "17811625",
});
```

```ruby
resp = JSON.parse(qn.streams.test_filter(
  network: "ethereum-mainnet",
  dataset: "block",
  block: "17811625"
))
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled (active) streams, optionally filtered by type.

**Parameters**: `stream_type` (string, optional).

**Returns**: `EnabledCountResponse` with `total`.

```rust
let resp = qn.streams.get_enabled_count(None).await?;
```

```python
resp = await qn.streams.get_enabled_count()
```

```typescript
const resp = await qn.streams.getEnabledCount();
```

```ruby
resp = JSON.parse(qn.streams.get_enabled_count({}))
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

**Returns**: `ListWebhooksResponse` with `data: Webhook[]` and `pageInfo`.

```rust
let resp = qn.webhooks.list_webhooks(&GetWebhooksParams::default()).await?;
```

```python
resp = await qn.webhooks.list_webhooks()
```

```typescript
const resp = await qn.webhooks.listWebhooks();
```

```ruby
resp = JSON.parse(qn.webhooks.list_webhooks({}))
```

##### `get_webhook` / `getWebhook`

Fetches a webhook by id.

**Parameters**: `id` (string, required).

**Returns**: `Webhook`.

```rust
let webhook = qn.webhooks.get_webhook("wh-1").await?;
```

```python
webhook = await qn.webhooks.get_webhook("wh-1")
```

```typescript
const webhook = await qn.webhooks.getWebhook("wh-1");
```

```ruby
webhook = JSON.parse(qn.webhooks.get_webhook(id: "wh-1"))
```

##### `create_webhook_from_template` / `createWebhookFromTemplate`

Creates a webhook from a predefined filter template.

**Parameters**: `name` (required), `network` (required), `destination_attributes` (`WebhookDestinationAttributes`, required), `template_args` (`TemplateArgs`, required), `notification_email` (optional).

**Returns**: `Webhook`.

```rust
let template_args = TemplateArgs::evm_wallet_filter(&EvmWalletFilterTemplate {
    wallets: vec!["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
})?;
let params = CreateWebhookFromTemplateParams {
    name: "Wallet Webhook".to_string(),
    network: "ethereum-mainnet".to_string(),
    notification_email: None,
    destination_attributes: WebhookDestinationAttributes {
        url: "https://webhook.site/...".to_string(),
        security_token: None,
        compression: None,
    },
    template_args,
};
let webhook = qn.webhooks.create_webhook_from_template(&params).await?;
```

```python
from sdk import EvmWalletFilterTemplate, TemplateArgs, WebhookDestinationAttributes

webhook = await qn.webhooks.create_webhook_from_template(
    name="Wallet Webhook",
    network="ethereum-mainnet",
    destination_attributes=WebhookDestinationAttributes(url="https://webhook.site/..."),
    template_args=TemplateArgs.evm_wallet_filter(
        EvmWalletFilterTemplate(wallets=["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"])
    ),
)
```

```typescript
import { TemplateArgs } from "quicknode-sdk";

const webhook = await qn.webhooks.createWebhookFromTemplate({
  name: "Wallet Webhook",
  network: "ethereum-mainnet",
  destinationAttributes: { url: "https://webhook.site/..." },
  templateArgs: TemplateArgs.evmWalletFilter({
    wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
  }),
});
```

```ruby
destination_attributes = JSON.generate({
  url: "https://webhook.site/...",
  compression: "none"
})
template_args = JSON.generate({
  template_id: "evmWalletFilter",
  value: JSON.generate({ wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"] })
})
webhook = JSON.parse(qn.webhooks.create_webhook_from_template(
  name: "Wallet Webhook",
  network: "ethereum-mainnet",
  destination_attributes_json: destination_attributes,
  template_args_json: template_args
))
```

##### `update_webhook` / `updateWebhook`

Partially updates a webhook's name, notification email, and/or destination. If `destination_attributes` is supplied without `security_token`, a new token is generated automatically.

**Parameters**: `id` (required); body — all optional: `name`, `notification_email`, `destination_attributes`.

**Returns**: updated `Webhook`.

```rust
let params = UpdateWebhookParams {
    name: Some("Renamed Webhook".to_string()),
    ..Default::default()
};
let webhook = qn.webhooks.update_webhook("wh-1", &params).await?;
```

```python
webhook = await qn.webhooks.update_webhook("wh-1", name="Renamed Webhook")
```

```typescript
const webhook = await qn.webhooks.updateWebhook("wh-1", { name: "Renamed Webhook" });
```

```ruby
webhook = JSON.parse(qn.webhooks.update_webhook(id: "wh-1", name: "Renamed Webhook"))
```

##### `update_webhook_template` / `updateWebhookTemplate`

Updates the template args (and optionally name, email, destination) on an existing template-backed webhook.

**Parameters**: `webhook_id` (required), `template_args` (required); optional: `name`, `notification_email`, `destination_attributes`.

**Returns**: updated `Webhook`.

```rust
let template_args = TemplateArgs::evm_wallet_filter(&EvmWalletFilterTemplate {
    wallets: vec!["0xnewwallet".to_string()],
})?;
let params = UpdateWebhookTemplateParams {
    name: None,
    notification_email: None,
    destination_attributes: None,
    template_args,
};
let webhook = qn.webhooks.update_webhook_template("wh-1", &params).await?;
```

```python
webhook = await qn.webhooks.update_webhook_template(
    "wh-1",
    template_args=TemplateArgs.evm_wallet_filter(
        EvmWalletFilterTemplate(wallets=["0xnewwallet"])
    ),
)
```

```typescript
const webhook = await qn.webhooks.updateWebhookTemplate("wh-1", {
  templateArgs: TemplateArgs.evmWalletFilter({ wallets: ["0xnewwallet"] }),
});
```

```ruby
template_args = JSON.generate({
  template_id: "evmWalletFilter",
  value: JSON.generate({ wallets: ["0xnewwallet"] })
})
webhook = JSON.parse(qn.webhooks.update_webhook_template(
  webhook_id: "wh-1",
  template_args_json: template_args
))
```

##### `delete_webhook` / `deleteWebhook`

Deletes a webhook.

**Parameters**: `id` (required).

**Returns**: nothing.

```rust
qn.webhooks.delete_webhook("wh-1").await?;
```

```python
await qn.webhooks.delete_webhook("wh-1")
```

```typescript
await qn.webhooks.deleteWebhook("wh-1");
```

```ruby
qn.webhooks.delete_webhook(id: "wh-1")
```

##### `delete_all_webhooks` / `deleteAllWebhooks`

Deletes every webhook on the account. Destructive and takes no arguments.

**Parameters**: none.

**Returns**: nothing.

```rust
qn.webhooks.delete_all_webhooks().await?;
```

```python
await qn.webhooks.delete_all_webhooks()
```

```typescript
await qn.webhooks.deleteAllWebhooks();
```

```ruby
qn.webhooks.delete_all_webhooks
```

##### `pause_webhook` / `pauseWebhook`

Pauses a webhook so it stops delivering events.

**Parameters**: `id` (required).

**Returns**: nothing.

```rust
qn.webhooks.pause_webhook("wh-1").await?;
```

```python
await qn.webhooks.pause_webhook("wh-1")
```

```typescript
await qn.webhooks.pauseWebhook("wh-1");
```

```ruby
qn.webhooks.pause_webhook(id: "wh-1")
```

##### `activate_webhook` / `activateWebhook`

Activates a paused or new webhook so it resumes delivering events. `start_from` determines where processing resumes.

**Parameters**: `id` (required), `start_from` (`WebhookStartFrom`, required — `Last` or `Latest`).

**Returns**: nothing.

```rust
let params = ActivateWebhookParams { start_from: WebhookStartFrom::Latest };
qn.webhooks.activate_webhook("wh-1", &params).await?;
```

```python
await qn.webhooks.activate_webhook("wh-1", start_from="latest")
```

```typescript
import { WebhookStartFrom } from "quicknode-sdk";

await qn.webhooks.activateWebhook("wh-1", { startFrom: WebhookStartFrom.Latest });
```

```ruby
qn.webhooks.activate_webhook(id: "wh-1", start_from: "latest")
```

##### `get_enabled_count` / `getEnabledCount`

Counts currently enabled webhooks.

**Parameters**: none.

**Returns**: `WebhookEnabledCountResponse` with `total`.

```rust
let resp = qn.webhooks.get_enabled_count().await?;
```

```python
resp = await qn.webhooks.get_enabled_count()
```

```typescript
const resp = await qn.webhooks.getEnabledCount();
```

```ruby
resp = JSON.parse(qn.webhooks.get_enabled_count)
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
qn.kvstore.create_set(&CreateSetParams {
    key: "my-key".to_string(),
    value: "hello".to_string(),
}).await?;
```

```python
await qn.kvstore.create_set(key="my-key", value="hello")
```

```typescript
await qn.kvstore.createSet({ key: "my-key", value: "hello" });
```

```ruby
qn.kvstore.create_set(key: "my-key", value: "hello")
```

##### `get_sets` / `getSets`

Paginated page of key/value entries.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetSetsResponse` — `{ data: KvSetEntry[], cursor: string }`.

```rust
let resp = qn.kvstore.get_sets(&Default::default()).await?;
```

```python
resp = await qn.kvstore.get_sets()
```

```typescript
const resp = await qn.kvstore.getSets();
```

```ruby
resp = JSON.parse(qn.kvstore.get_sets({}))
```

##### `get_set` / `getSet`

Returns the value stored under a key.

**Parameters**: `key` (string, required).

**Returns**: `GetSetResponse` with `value`.

```rust
let resp = qn.kvstore.get_set("my-key").await?;
```

```python
resp = await qn.kvstore.get_set("my-key")
```

```typescript
const resp = await qn.kvstore.getSet("my-key");
```

```ruby
resp = JSON.parse(qn.kvstore.get_set(key: "my-key"))
```

##### `bulk_sets` / `bulkSets`

Adds and/or deletes multiple sets in a single request.

**Parameters** (at least one required): `add_sets` (map<string,string>, optional), `delete_sets` (string[], optional). The Ruby binding currently only forwards `delete_sets`; use `create_set` / `bulk_sets` individually from Ruby to add.

**Returns**: nothing.

```rust
use std::collections::HashMap;

let mut add_sets = HashMap::new();
add_sets.insert("k1".to_string(), "v1".to_string());
qn.kvstore.bulk_sets(&BulkSetsParams {
    add_sets: Some(add_sets),
    delete_sets: Some(vec!["old-key".to_string()]),
}).await?;
```

```python
await qn.kvstore.bulk_sets(
    add_sets={"k1": "v1"},
    delete_sets=["old-key"],
)
```

```typescript
await qn.kvstore.bulkSets({
  addSets: { k1: "v1" },
  deleteSets: ["old-key"],
});
```

```ruby
qn.kvstore.bulk_sets(delete_sets: ["old-key"])
```

##### `delete_set` / `deleteSet`

Deletes a single set.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```rust
qn.kvstore.delete_set("my-key").await?;
```

```python
await qn.kvstore.delete_set("my-key")
```

```typescript
await qn.kvstore.deleteSet("my-key");
```

```ruby
qn.kvstore.delete_set(key: "my-key")
```

#### Lists

##### `create_list` / `createList`

Creates a list under a key, seeded with the initial items.

**Parameters**: `key` (string, required), `items` (string[], required).

**Returns**: nothing.

```rust
qn.kvstore.create_list(&CreateListParams {
    key: "my-list".to_string(),
    items: vec!["0xabc".to_string(), "0xdef".to_string()],
}).await?;
```

```python
await qn.kvstore.create_list(key="my-list", items=["0xabc", "0xdef"])
```

```typescript
await qn.kvstore.createList({ key: "my-list", items: ["0xabc", "0xdef"] });
```

```ruby
qn.kvstore.create_list(key: "my-list", items: ["0xabc", "0xdef"])
```

##### `get_lists` / `getLists`

Paginated page of list keys.

**Parameters** (all optional): `limit` (i64), `cursor` (string).

**Returns**: `GetListsResponse` — `{ data: { keys: string[] }, cursor: string }`.

```rust
let resp = qn.kvstore.get_lists(&Default::default()).await?;
```

```python
resp = await qn.kvstore.get_lists()
```

```typescript
const resp = await qn.kvstore.getLists();
```

```ruby
resp = JSON.parse(qn.kvstore.get_lists({}))
```

##### `get_list` / `getList`

Paginated page of items for a specific list.

**Parameters**: `key` (string, required); optional `limit` (i64), `cursor` (string).

**Returns**: `GetListResponse` — `{ data: { items: string[] }, cursor: string }`.

```rust
let resp = qn.kvstore.get_list("my-list", &Default::default()).await?;
```

```python
resp = await qn.kvstore.get_list("my-list")
```

```typescript
const resp = await qn.kvstore.getList("my-list");
```

```ruby
resp = JSON.parse(qn.kvstore.get_list(key: "my-list"))
```

##### `update_list` / `updateList`

Adds and/or removes items in a single operation.

**Parameters**: `key` (string, required); optional: `add_items` (string[]), `remove_items` (string[]).

**Returns**: nothing.

```rust
qn.kvstore.update_list(
    "my-list",
    &UpdateListParams {
        add_items: Some(vec!["0x456".to_string()]),
        remove_items: Some(vec!["0xabc".to_string()]),
    },
).await?;
```

```python
await qn.kvstore.update_list(
    "my-list",
    add_items=["0x456"],
    remove_items=["0xabc"],
)
```

```typescript
await qn.kvstore.updateList("my-list", {
  addItems: ["0x456"],
  removeItems: ["0xabc"],
});
```

```ruby
qn.kvstore.update_list(key: "my-list", add_items: ["0x456"], remove_items: ["0xabc"])
```

##### `add_list_item` / `addListItem`

Appends a single item to a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```rust
qn.kvstore.add_list_item(
    "my-list",
    &AddListItemParams { item: "0x123".to_string() },
).await?;
```

```python
await qn.kvstore.add_list_item("my-list", "0x123")
```

```typescript
await qn.kvstore.addListItem("my-list", { item: "0x123" });
```

```ruby
qn.kvstore.add_list_item(key: "my-list", item: "0x123")
```

##### `list_contains_item` / `listContainsItem`

Checks whether a list contains a specific item.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: `ListContainsItemResponse` with `exists: bool`.

```rust
let resp = qn.kvstore.list_contains_item("my-list", "0x123").await?;
```

```python
resp = await qn.kvstore.list_contains_item("my-list", "0x123")
```

```typescript
const resp = await qn.kvstore.listContainsItem("my-list", "0x123");
```

```ruby
resp = JSON.parse(qn.kvstore.list_contains_item(key: "my-list", item: "0x123"))
```

##### `delete_list_item` / `deleteListItem`

Removes a single item from a list.

**Parameters**: `key` (string, required), `item` (string, required).

**Returns**: nothing.

```rust
qn.kvstore.delete_list_item("my-list", "0x123").await?;
```

```python
await qn.kvstore.delete_list_item("my-list", "0x123")
```

```typescript
await qn.kvstore.deleteListItem("my-list", "0x123");
```

```ruby
qn.kvstore.delete_list_item(key: "my-list", item: "0x123")
```

##### `delete_list` / `deleteList`

Deletes a list and all of its items.

**Parameters**: `key` (string, required).

**Returns**: nothing.

```rust
qn.kvstore.delete_list("my-list").await?;
```

```python
await qn.kvstore.delete_list("my-list")
```

```typescript
await qn.kvstore.deleteList("my-list");
```

```ruby
qn.kvstore.delete_list(key: "my-list")
```

## Error Handling

The core SDK defines `SdkError` (`crates/core/src/errors.rs`) with these variants:

- `Http` — transport failure (wraps `reqwest::Error`).
- `Api { status, body }` — non-2xx HTTP response, carrying the status code and raw response body.
- `Decode { source, body }` — response was 2xx but JSON parsing failed; `body` holds the raw payload for debugging.
- `Config` — misconfiguration surfaced at construction time.

Each language binding maps these to its native exception type:

- **Rust**: `Result<T, SdkError>` — pattern-match on the variants.
- **Python**: raises `ValueError` (`PyValueError`) with the error message.
- **Node.js**: rejects with a napi-wrapped `Error` carrying the message.
- **Ruby**: raises `RuntimeError` for SDK errors and `ArgumentError` for missing/unknown Hash keys or bad types.

```rust
match qn.streams.get_stream("missing").await {
    Ok(stream) => println!("{}", stream.name),
    Err(SdkError::Api { status, body }) => eprintln!("api {status}: {body}"),
    Err(e) => eprintln!("other error: {e}"),
}
```

```python
try:
    await qn.streams.get_stream("missing")
except ValueError as e:
    print(f"sdk error: {e}")
```

```typescript
try {
  await qn.streams.getStream("missing");
} catch (e) {
  console.error("sdk error:", e);
}
```

```ruby
begin
  qn.streams.get_stream(id: "missing")
rescue => e
  warn "sdk error: #{e.message}"
end
```

## Development

### Prerequisites

- Rust (stable)
- Python 3.8+ with [uv](https://docs.astral.sh/uv/)
- Node.js 18+
- Ruby 3.0+
- [just](https://github.com/casey/just)

### Build Commands

Use the commands in the `Justfile` for the setup and build commands.

```bash
# Core library
cargo check
cargo test -p sdk-core

# Python (from project root)
just python-setup-env
just python-build

# Node.js (from npm/)
just node-build

# Ruby
just ruby-build

# Rust
cargo build -p sdk-core
```

### Testing

```bash
just test
```

Runs the Rust unit tests for `sdk-core` using [wiremock](https://github.com/LukeMathWalker/wiremock-rs) to mock HTTP responses — no API key required.

### Examples

```bash
# Rust
QN_SDK__API_KEY=replaceme cargo run --example admin -p sdk-core --features rust

# Python
QN_SDK__API_KEY=replaceme uv run python/examples/admin.py
QN_SDK__API_KEY=replaceme uv run python/examples/streams.py

# Node.js
cd npm && QN_SDK__API_KEY=replaceme npx tsx examples/admin.ts
cd npm && QN_SDK__API_KEY=replaceme npx tsx examples/streams.ts

# Ruby (build first, then run)
just ruby-build
QN_SDK__API_KEY=replaceme ruby ruby/examples/admin.rb
QN_SDK__API_KEY=replaceme ruby ruby/examples/admin_e2e.rb
QN_SDK__API_KEY=replaceme ruby ruby/examples/streams.rb
```

## License

MIT
