# QuickNode SDK Notes

## Overview

The QuickNode SDK wraps QuickNode's platform APIs across four product areas:

- **Admin** — Manage RPC endpoints, security, rate limits, teams, usage, billing
- **Streams** — Real-time blockchain data streams to various destinations
- **Webhooks** — Event-driven webhook subscriptions via templates
- **KV Store** — Distributed key-value store with set and list data structures

The SDK is polyglot: one Rust core with Python (PyO3/maturin) and Node.js (napi-rs) bindings generated from the same types.

3 Languages will be supported at first: Rust, Typescript, and Python

---

## Goals

- **Easy access to QuickNode products** — a single, unified entry point for Admin, Streams, Webhooks, and KV Store instead of hand-rolling HTTP calls
- **Standardized offerings** — consistent API shape, naming conventions, and behavior across all four product clients
- **Type safety** — fully typed request/response structs give immediate feedback and guard against mistakes for both humans and AI agents
- **Cross-language consistency** — same types, same API surface, same behavior in Rust, TypeScript, and Python; one Rust source of truth drives all three
- **Reduced boilerplate** — builder patterns and sensible defaults minimize setup; auth and transport are handled automatically
- **Maintainability** — adding or updating a product client means touching one place (Rust core); bindings are generated, not duplicated
- **Reliability** — unified error handling (`SdkError`) and a mocked HTTP test layer mean consumers can trust the SDK behaves correctly without hitting real APIs

---

## Examples

### Initialization

**Rust**
```rust
let config = SdkFullConfig::from_env()?;
let qn = QuickNodeSdk::new(&config)?;
```

**TypeScript**
```typescript
const qn = QuickNodeSdk.fromEnv();
```

**Python**
```python
qn = QuickNodeSdk.from_env()
```

All three read the API key from `QN_SDK_API_KEY`.

---

### Admin — Endpoints

**Rust**
```rust
// List endpoints
let resp = qn.admin.get_endpoints(&GetEndpointsRequest::builder().limit(20).build()).await?;

// Create an endpoint
let endpoint = qn.admin.create_endpoint(
    &CreateEndpointRequest::builder()
        .chain("ethereum".to_string())
        .network("mainnet".to_string())
        .build(),
).await?;
let id = endpoint.data.id;

// Update, secure, rate-limit, then archive
qn.admin.update_endpoint(&id, &UpdateEndpointRequest { label: Some("my-node".to_string()) }).await?;
qn.admin.create_ip(&id, &CreateIpRequest { ip: Some("192.0.2.1".to_string()) }).await?;
qn.admin.create_method_rate_limit(&id, &CreateMethodRateLimitRequest {
    interval: "second".to_string(),
    methods: vec!["eth_call".to_string()],
    rate: 5,
}).await?;
qn.admin.archive_endpoint(&id).await?;
```

**TypeScript**
```typescript
const resp = await qn.admin.getEndpoints({ limit: 20 });
for (const ep of resp.data) {
  console.log(`${ep.id} | ${ep.network}`);
}
```

**Python**
```python
response = await qn.admin.get_endpoints(limit=20)
for ep in response.data:
    print(f"{ep.id} | {ep.network}")
```

---

### Streams — Create a block stream

**Rust**
```rust
let stream = qn.streams.create_stream(
    &CreateStreamParams::builder()
        .name("My Block Stream".to_string())
        .network("ethereum-mainnet".to_string())
        .dataset(StreamDataset::Block)
        .region(StreamRegion::UsaEast)
        .start_range(24691804)
        .destination_attributes(
            DestinationAttributes::webhook(&WebhookAttributes {
                url: "https://...".to_string(),
                compression: "none".to_string(),
                max_retry: 3,
                retry_interval_sec: 1,
                post_timeout_sec: 10,
                security_token: None,
            }).expect("valid"),
        )
        .status(StreamStatus::Active)
        .build(),
).await?;

qn.streams.pause_stream(&stream.id).await?;
qn.streams.delete_stream(&stream.id).await?;
```

**TypeScript**
```typescript
const stream = await qn.streams.createStream({
  name: "My Block Stream",
  network: "ethereum-mainnet",
  dataset: StreamDataset.Block,
  region: StreamRegion.UsaEast,
  startRange: 24691804,
  destinationAttributes: DestinationAttributes.webhook({
    url: "https://...",
    maxRetry: 3,
    retryIntervalSec: 1,
    postTimeoutSec: 10,
    compression: "none",
  }),
  status: StreamStatus.Active,
});

await qn.streams.pauseStream(stream.id);
await qn.streams.deleteStream(stream.id);
```

**Python**
```python
stream = await qn.streams.create_stream(
    name="My Block Stream",
    network="ethereum-mainnet",
    dataset="block",
    region="usa_east",
    start_range=24691804,
    destination_attributes=DestinationAttributes.webhook(
        WebhookAttributes(url="https://...", max_retry=3, retry_interval_sec=1,
                          post_timeout_sec=10, compression="none")
    ),
    status="active",
)

await qn.streams.pause_stream(stream.id)
await qn.streams.delete_stream(stream.id)
```

---

### Webhooks — Create from template

**Rust**
```rust
let webhook = qn.webhooks.create_webhook_from_template(&CreateWebhookFromTemplateParams {
    name: "Wallet Watcher".to_string(),
    network: "ethereum-mainnet".to_string(),
    destination_attributes: WebhookDestinationAttributes {
        url: "https://...".to_string(),
        security_token: None,
        compression: None,
    },
    template_args: TemplateArgs::evm_wallet_filter(&EvmWalletFilterTemplate {
        wallets: vec!["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_string()],
    }).expect("valid"),
    notification_email: None,
}).await?;

qn.webhooks.delete_webhook(&webhook.id).await?;
```

**TypeScript**
```typescript
const webhook = await qn.webhooks.createWebhookFromTemplate({
  name: "Wallet Watcher",
  network: "ethereum-mainnet",
  destinationAttributes: { url: "https://..." },
  templateArgs: TemplateArgs.evmWalletFilter({
    wallets: ["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"],
  }),
});

await qn.webhooks.deleteWebhook(webhook.id);
```

**Python**
```python
webhook = await qn.webhooks.create_webhook_from_template(
    name="Wallet Watcher",
    network="ethereum-mainnet",
    destination_attributes=WebhookDestinationAttributes(url="https://..."),
    template_args=TemplateArgs.evm_wallet_filter(
        EvmWalletFilterTemplate(wallets=["0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"])
    ),
)

await qn.webhooks.delete_webhook(webhook.id)
```

---

### KV Store — Sets and lists

**Rust**
```rust
qn.kvstore.create_set(&CreateSetParams { key: "k".to_string(), value: "v".to_string() }).await?;
let entry = qn.kvstore.get_set("k").await?;

qn.kvstore.create_list(&CreateListParams {
    key: "wallets".to_string(),
    items: vec!["0xabc".to_string()],
}).await?;
let exists = qn.kvstore.list_contains_item("wallets", "0xabc").await?;
```

**TypeScript**
```typescript
await qn.kvstore.createSet({ key: "k", value: "v" });
const entry = await qn.kvstore.getSet("k");

await qn.kvstore.createList({ key: "wallets", items: ["0xabc"] });
const exists = await qn.kvstore.listContainsItem("wallets", "0xabc");
```

**Python**
```python
await qn.kvstore.create_set(key="k", value="v")
entry = await qn.kvstore.get_set("k")

await qn.kvstore.create_list(key="wallets", items=["0xabc"])
exists = await qn.kvstore.list_contains_item("wallets", "0xabc")
```
---

## Admin Client

**Base URL:** `https://api.quicknode.com/v0/`

### Endpoints

| Function | Method | Path |
|----------|--------|------|
| `get_endpoints(params)` | GET | `endpoints` |
| `create_endpoint(params)` | POST | `endpoints` |
| `show_endpoint(id)` | GET | `endpoints/{id}` |
| `update_endpoint(id, params)` | PATCH | `endpoints/{id}` |
| `archive_endpoint(id)` | DELETE | `endpoints/{id}` |
| `update_endpoint_status(id, params)` | PATCH | `endpoints/{id}/status` |
| `enable_multichain(id)` | POST | `endpoints/{id}/enable_multichain` |
| `disable_multichain(id)` | POST | `endpoints/{id}/disable_multichain` |

### Tags

| Function | Method | Path |
|----------|--------|------|
| `create_tag(id, params)` | POST | `endpoints/{id}/tags` |
| `delete_tag(id, tag_id)` | DELETE | `endpoints/{id}/tags/{tag_id}` |

### Logs

| Function | Method | Path |
|----------|--------|------|
| `get_endpoint_logs(id, params)` | GET | `endpoints/{id}/logs` |
| `get_log_details(id, request_id)` | GET | `endpoints/{id}/log_details?request_id=...` |

### Security Options

| Function | Method | Path |
|----------|--------|------|
| `get_security_options(id)` | GET | `endpoints/{id}/security_options` |
| `update_security_options(id, params)` | PATCH | `endpoints/{id}/security_options` |
| `create_token(id)` | POST | `endpoints/{id}/security/tokens` |
| `delete_token(id, token_id)` | DELETE | `endpoints/{id}/security/tokens/{token_id}` |
| `create_referrer(id, params)` | POST | `endpoints/{id}/security/referrers` |
| `delete_referrer(id, referrer_id)` | DELETE | `endpoints/{id}/security/referrers/{referrer_id}` |
| `create_ip(id, params)` | POST | `endpoints/{id}/security/ips` |
| `delete_ip(id, ip_id)` | DELETE | `endpoints/{id}/security/ips/{ip_id}` |
| `create_domain_mask(id, params)` | POST | `endpoints/{id}/security/domain_masks` |
| `delete_domain_mask(id, mask_id)` | DELETE | `endpoints/{id}/security/domain_masks/{mask_id}` |
| `create_jwt(id, params)` | POST | `endpoints/{id}/security/jwts` |
| `delete_jwt(id, jwt_id)` | DELETE | `endpoints/{id}/security/jwts/{jwt_id}` |
| `create_request_filter(id, params)` | POST | `endpoints/{id}/security/request_filters` |
| `update_request_filter(id, filter_id, params)` | PUT | `endpoints/{id}/security/request_filters/{filter_id}` |
| `delete_request_filter(id, filter_id)` | DELETE | `endpoints/{id}/security/request_filters/{filter_id}` |
| `create_or_update_ip_custom_header(id, params)` | PATCH | `endpoints/{id}/ip_custom_header` |
| `delete_ip_custom_header(id)` | DELETE | `endpoints/{id}/ip_custom_header` |

### Rate Limits

| Function | Method | Path |
|----------|--------|------|
| `get_method_rate_limits(id)` | GET | `endpoints/{id}/method-rate-limits` |
| `create_method_rate_limit(id, params)` | POST | `endpoints/{id}/method-rate-limits` |
| `update_method_rate_limit(id, limiter_id, params)` | PATCH | `endpoints/{id}/method-rate-limits/{limiter_id}` |
| `delete_method_rate_limit(id, limiter_id)` | DELETE | `endpoints/{id}/method-rate-limits/{limiter_id}` |
| `update_rate_limits(id, params)` | PUT | `endpoints/{id}/rate-limits` |

### Metrics

| Function | Method | Path |
|----------|--------|------|
| `get_endpoint_metrics(id, params)` | GET | `endpoints/{id}/metrics` |
| `get_account_metrics(params)` | GET | `metrics` |

### Teams

| Function | Method | Path |
|----------|--------|------|
| `list_teams()` | GET | `teams` |
| `create_team(params)` | POST | `teams` |
| `get_team(id)` | GET | `teams/{id}` |
| `delete_team(id)` | DELETE | `teams/{id}` |
| `list_team_endpoints(id)` | GET | `teams/{id}/endpoints` |
| `update_team_endpoints(id, params)` | PATCH | `teams/{id}/endpoints` |
| `invite_team_member(id, params)` | POST | `teams/{id}/members` |
| `remove_team_member(id, user_id, params)` | DELETE | `teams/{id}/members/{user_id}` |
| `resend_team_invite(id, user_id)` | POST | `teams/{id}/members/{user_id}/resend_invite` |

### Usage

| Function | Method | Path |
|----------|--------|------|
| `get_usage(params)` | GET | `usage/rpc` |
| `get_usage_by_endpoint(params)` | GET | `usage/rpc/by-endpoint` |
| `get_usage_by_method(params)` | GET | `usage/rpc/by-method` |
| `get_usage_by_chain(params)` | GET | `usage/rpc/by-chain` |

### Billing & Chains

| Function | Method | Path |
|----------|--------|------|
| `list_chains()` | GET | `chains` |
| `list_invoices()` | GET | `billing/invoices` |
| `list_payments()` | GET | `billing/payments` |

---

## Streams Client

**Base URL:** `https://api.quicknode.com/streams/rest/v1/`

| Function | Method | Path |
|----------|--------|------|
| `create_stream(params)` | POST | `streams` |
| `list_streams(params)` | GET | `streams` |
| `get_stream(id)` | GET | `streams/{id}` |
| `update_stream(id, params)` | PATCH | `streams/{id}` |
| `delete_stream(id)` | DELETE | `streams/{id}` |
| `activate_stream(id)` | POST | `streams/{id}/activate` |
| `pause_stream(id)` | POST | `streams/{id}/pause` |
| `delete_all_streams()` | DELETE | `streams` |
| `test_filter(params)` | POST | `streams/test_filter` |
| `get_enabled_count(stream_type?)` | GET | `streams/enabled_count` |

**Datasets** (`StreamDataset`): `Block`, `Transaction`, `Log`, `TraceCall`, `NftTransfer`

**Regions** (`StreamRegion`): `UsaEast`, `UsaWest`, `EuropeWest`, `AsiaEast`

**Destinations**: Webhook, S3, Azure Blob, PostgreSQL, MySQL, MongoDB, Clickhouse, Snowflake, Kafka, Redis

---

## Webhooks Client

**Base URL:** `https://api.quicknode.com/webhooks/rest/v1/`

| Function | Method | Path |
|----------|--------|------|
| `list_webhooks(params)` | GET | `webhooks` |
| `get_webhook(id)` | GET | `webhooks/{id}` |
| `update_webhook(id, params)` | PATCH | `webhooks/{id}` |
| `delete_webhook(id)` | DELETE | `webhooks/{id}` |
| `pause_webhook(id)` | POST | `webhooks/{id}/pause` |
| `activate_webhook(id, params)` | POST | `webhooks/{id}/activate` |
| `delete_all_webhooks()` | DELETE | `webhooks` |
| `get_enabled_count()` | GET | `webhooks/enabled_count` |
| `create_webhook_from_template(params)` | POST | `webhooks/template/{template_id}` |
| `update_webhook_template(webhook_id, params)` | PATCH | `webhooks/{webhook_id}/template/{template_id}` |

**Templates** (`WebhookTemplateId`): `EvmWalletFilter`, `EvmContractEvents`, `EvmAbiFilter`, `SolanaWalletFilter`, `BitcoinWalletFilter`, `XrplWalletFilter`, `HyperliquidWalletEventsFilter`, `StellarWalletTransactionsFilter`

---

## KV Store Client

**Base URL:** `https://api.quicknode.com/kv/rest/v1/`

### Sets

| Function | Method | Path |
|----------|--------|------|
| `create_set(params)` | POST | `sets` |
| `get_sets(params)` | GET | `sets` |
| `get_set(key)` | GET | `sets/{key}` |
| `bulk_sets(params)` | POST | `sets/bulk` |
| `delete_set(key)` | DELETE | `sets/{key}` |

### Lists

| Function | Method | Path |
|----------|--------|------|
| `create_list(params)` | POST | `lists` |
| `get_lists(params)` | GET | `lists` |
| `get_list(key, params)` | GET | `lists/{key}` |
| `update_list(key, params)` | PATCH | `lists/{key}` |
| `add_list_item(key, params)` | POST | `lists/{key}/items` |
| `list_contains_item(key, item)` | GET | `lists/{key}/contains/{item}` |
| `delete_list_item(key, item)` | DELETE | `lists/{key}/items/{item}` |
| `delete_list(key)` | DELETE | `lists/{key}` |

---

## Technical Details

### How it works: FFI via C ABI

Both Python and Node.js load Rust code through a **C-compatible shared library** — not through a runtime VM or interpreter bridge. The two binding crates (`crates/python`, `crates/node`) are compiled with `crate-type = ["cdylib"]`, which tells the Rust compiler to produce a shared library with a stable C ABI (`.so` on Linux/macOS, `.pyd`/`.dll` on Windows). The host runtime then `dlopen`s that binary and calls into it through a well-defined C interface.

**Python — CPython C API**
When Python executes `import sdk._core`, it looks for a shared library with an entry point named `PyInit__core`. PyO3's `#[pymodule]` macro auto-generates that C function, which registers all `#[pyclass]` structs as Python types and returns a module object to the interpreter. From that point the Python runtime treats `_core` like any other module — the fact that it's compiled Rust is invisible. PyO3 targets the stable CPython ABI (using the `abi3` feature), meaning the same `.so` can run across multiple Python versions without recompilation. The `extension-module` feature flag ensures PyO3 doesn't try to initialize CPython itself (the interpreter already owns the runtime).

**Node.js — N-API (Node-API)**
Node.js has a stable native module interface called N-API, deliberately designed to be ABI-stable across Node.js versions. When Node loads a `.node` file (via `require`), it calls the `napi_register_module_v1` entry point, which napi-rs auto-generates from `#[napi]` annotations. Unlike the V8 C++ API (which breaks between Node versions), N-API is a pure C interface maintained as part of Node.js core, so a `.node` binary compiled against N-API v8 works on any Node.js version that supports N-API v8+. `napi_build::setup()` in `build.rs` configures the linker flags needed to target this interface.

**Summary**

| | Python | Node.js |
|---|---|---|
| Interface | CPython C API | Node-API (N-API) |
| Entry point | `PyInit__core()` | `napi_register_module_v1()` |
| Binary | `.so` / `.pyd` | `.node` |
| ABI stability | CPython `abi3` | N-API versioned |
| Async bridge | `future_into_py` → `asyncio` coroutine | Auto-wrapped → `Promise` |

When you write `from sdk._core import QuickNodeSdk` or `require('./sdk.node')`, you're calling directly into compiled Rust machine code through a standardized C interface.

---

### Architecture: One Core, Three Languages

All types and business logic live in `crates/core`. The two binding crates (`crates/python`, `crates/node`) are thin wrappers — they handle async bridging and argument marshalling, nothing else. Types in core use feature-gated attribute macros to opt into each language's binding system:

```rust
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
pub struct CreateStreamParams { ... }
```

The three features (`python`, `node`, `rust`) are mutually exclusive at compile time — each binding crate enables only its own feature when building.

---

### Python: PyO3 + Maturin

**Build pipeline** (`just python-build`):
1. `maturin develop` — compiles `crates/python` with `feature="python"` into `sdk._core`, a native `.so`/`.pyd` extension installed directly into the active venv
2. `cargo run -p sdk-python-stubs` — runs `pyo3-stub-gen` to read `#[gen_stub_*]` metadata from all annotated types and emit `.pyi` stub files
3. `cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi` — restores the manually-maintained stub (maturin overwrites `__init__.pyi` on each build)

**Async bridging** — Python is synchronous at the C level; PyO3 can't natively return a Rust future. Each async method in `crates/python/src/lib.rs` wraps the core call in `pyo3_async_runtimes::tokio::future_into_py`, which runs the future on a shared Tokio runtime and returns a Python coroutine:

```rust
#[gen_stub_pymethod]
#[pyo3(signature = (id))]
fn get_stream<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
    let client = self.inner.clone();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        client.get_stream(&id).await.map_err(|e| PyValueError::new_err(e.to_string()))
    })
}
```

**Method signatures** — Python methods accept flat keyword arguments instead of structs (e.g., `create_stream(name=..., network=...)`) so callers don't have to instantiate intermediate classes.

**Type stubs** — `python/sdk/__init__.pyi` is a build artifact; edit `init_manual_override.pyi` instead. `python/sdk/__init__.py` is manually maintained and must be updated when adding new public types.

---

### Node.js: napi-rs

**Build pipeline** (`just node-build`):
1. `npm install` in `npm/`
2. `napi build --platform --release --cargo-cwd ../crates/node` — compiles `crates/node` with `feature="node"` to a platform-specific `.node` native binary
3. napi-rs auto-generates `npm/index.d.ts` with TypeScript interfaces for all annotated types
4. `npm run test` runs the test suite against the compiled binary

**Async bridging** — napi-rs detects `async fn` automatically and handles the Tokio runtime internally. No manual wrapping is needed:

```rust
#[napi]
impl StreamsApiClient {
    #[napi]
    pub async fn get_stream(&self, id: String) -> napi::Result<Stream> {
        self.inner.get_stream(&id).await
            .map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}
```

**Naming conventions** — napi-rs automatically converts Rust `snake_case` to JavaScript `camelCase` for field names and method names. `get_stream` becomes `getStream`, `start_range` becomes `startRange`.

**Generated vs. manual files:**
- `npm/index.d.ts` — auto-generated by napi-rs; contains raw interfaces and `const enum` types; do not edit
- `npm/sdk.d.ts` — manually maintained; re-exports types from `index.d.ts` using `export type { ... }`; string enums must use `export { ... }` (not `export type`) so they work as runtime values
- `npm/sdk.js` — manually maintained; provides factory helpers like `DestinationAttributes.webhook(...)` and `TemplateArgs.evmWalletFilter(...)` that serialize complex union types to JSON strings for the napi layer

---

### Build Commands

```bash
# Python (one-time setup)
just python-setup-env       # uv venv + install maturin

# Python (after any core change)
just python-build           # maturin develop + stubs + copy override

# Node.js
just node-build             # npm install + napi build + tests

# Rust only
cargo check
cargo test -p sdk-core --lib

# Full
cargo check && just lint && just python-build && just node-build && just test
```

