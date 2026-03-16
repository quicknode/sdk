# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Rust
```bash
cargo check                                        # Type check all crates
cargo build -p sdk-core                           # Build core crate
cargo test -p sdk-core                            # Run tests
cargo run --example admin -p sdk-core             # Run example (requires QN_API_KEY env var)
```

### Python
```bash
just python-setup-env                             # Create venv, install maturin (one-time)
just python-build                                 # Compile bindings + generate stubs
uv run example.py                                 # Run Python example (requires QN_API_KEY)
```

### Node.js
```bash
just node-build                                   # npm install + build + test
cd npm && QN_API_KEY=xxx npx tsx example.ts       # Run example
```

## Verification

When verifying changes, use these commands based on what was modified:

- **Rust only** — `cargo check`
- **Python crate/bindings** — `just python-setup-env` (first time only), then `just python-build`
- **Node/npm** — `just node-build`
- **Full verification** — `cargo check && just python-build && just node-build`

> Note: Do not use `cargo build` directly — Python bindings are compiled via maturin (`just python-build`).

## Architecture

This is a polyglot SDK: one Rust core library with Python and Node.js bindings generated from the same types.

### Workspace Layout
- `crates/core` — Pure Rust business logic (HTTP client, request/response types, errors)
- `crates/python` — PyO3 wrapper crate, compiles to `sdk._core` Python extension
- `crates/node` — napi-rs wrapper crate, compiles to native `.node` module
- `crates/python-stubs` — Generates `.pyi` type stub files
- `python/sdk/` — Python package directory (distributed via maturin)
- `npm/` — Node.js package directory

### Core Pattern
`QuickNodeSdk` is the root entry point holding sub-clients (e.g., `admin: AdminApiClient`). All clients share a `SdkConfig(Arc<SdkConfigInner>)` wrapping one `reqwest` HTTP client and the API key.

### Multi-Language Type Annotations
Data types are defined once in `crates/core/src/` with feature-gated attribute macros:
```rust
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "rust", derive(Builder))]
pub struct SomeRequest { ... }
```
- `python` feature — PyO3 class macros and stub generation via `pyo3-stub-gen`
- `node` feature — napi-rs object macros and auto-generated TypeScript types in `npm/index.d.ts`
- `rust` feature — `bon` builder pattern for ergonomic Rust usage

### Error Handling
`SdkError` (`crates/core/src/errors.rs`) uses `thiserror` with three variants:
- `Http` — wraps `reqwest::Error`
- `Api` — non-2xx response with status code and raw body
- `Decode` — JSON parse failure with raw body for debugging

Language bindings convert `SdkError` to native exceptions: `PyValueError` (Python), `napi::Error` (Node.js).

### Python Binding Pattern
`crates/python/src/lib.rs` wraps core async methods using `pyo3_async_runtimes::tokio::future_into_py`. The Python API accepts individual keyword arguments instead of structs.

### Node.js Binding Pattern
`crates/node/src/lib.rs` uses `#[napi(constructor)]` and `#[napi(getter)]` macros. napi handles async conversion automatically.
