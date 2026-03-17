# Quicknode SDK

A unified SDK for building on QuickNode.

Rust SDK with Python and Node.js bindings.

## Project Structure

```
sdk/
├── crates/
│   ├── core/          # Pure Rust business logic
│   ├── python/        # PyO3 bindings
│   └── node/          # napi-rs bindings
├── python/sdk/        # Python package with type hints
├── npm/               # Node.js package with TypeScript types
└── pyproject.toml     # maturin build config
```

## Installation

**Python:** `uv add quicknode-sdk`

**Node.js:** `npm install quicknode-sdk`

## Development

### Prerequisites

- Rust (stable)
- Python 3.8+ with [uv](https://docs.astral.sh/uv/)
- Node.js 18+
- [just](https://github.com/casey/just)

### Build Commands

Use the commands in the `Justfile` for the setup and build commands
```bash
# Core library
cargo check
cargo test -p sdk-core

# Python (from project root)
just python-setup-env
just python-build

# Node.js (from npm/)
just node-build

# Rust
cargo build -p sdk-core
```

### Testing

```bash
just test
```

Runs the Rust unit tests for `sdk-core` using [wiremock](https://github.com/LukeMathWalker/wiremock-rs) to mock HTTP responses — no API key required.

Examples
```bash
QN_API_KEY=replaceme cargo run --example admin -p sdk-core --features rust
QN_API_KEY=replaceme uv run example.py
cd npm && QN_API_KEY=replaceme npx tsx example.ts
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
import { QuickNodeSdk } from ".";
const qn = new QuickNodeSdk({ apiKey: "your-key", http: { timeoutSecs: 30 } });
```

```rust
// Rust
let qn = QuickNodeSdk::new(SdkFullConfig::builder().api_key("your-key").build())?;
```

### Option B — Load from environment (`from_env()`)

```python
qn = QuickNodeSdk.from_env()
```
```typescript
const qn = QuickNodeSdk.fromEnv();
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
| `QN_SDK__ADMIN__BASE_URL` | no | `https://api.quicknode.com/v0/` | Override admin API base URL (must be HTTPS, must end with `/`) |


## License

MIT
