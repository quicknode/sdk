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

**Python:** `uv add my-sdk`

**Node.js:** `npm install my-sdk`

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

## License

MIT
