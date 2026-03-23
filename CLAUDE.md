# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Rust
```bash
cargo check                                        # Type check all crates
cargo build -p sdk-core                           # Build core crate
cargo test -p sdk-core --lib                      # Run tests (excludes examples)
cargo run --example admin -p sdk-core             # Run example (requires QN_API_KEY env var)
```

### Python
```bash
just python-setup-env                             # Create venv, install maturin (one-time)
just python-build                                 # Compile bindings + generate stubs
cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi # Manually override __init__ so we can overwrite the commands
```
If you are in a fish shell, run the python-setup-env manually:
```
uv venv
source .venv/bin/activate.fish
uv pip install maturin
```

### Node.js
```bash
just node-build                                   # npm install + build + test
```

## Verification

When verifying changes, use these commands based on what was modified:

- **Rust only** — `cargo check && just lint`
- **Python crate/bindings** — `just python-setup-env` (first time only), then `just python-build`
- **Node/npm** — `just node-build`
- **Full verification** — `cargo check && just lint && just python-build && just node-build && just test`

> Note: Do not use `cargo build` directly — Python bindings are compiled via maturin (`just python-build`).

if you can't run a just command, see what it's executing and run it manually

## Architecture

This is a polyglot SDK: one Rust core library with Python and Node.js bindings generated from the same types

### Workspace Layout
- `crates/core` — Pure Rust business logic (HTTP client, request/response types, errors)
- `crates/python` — PyO3 wrapper crate, compiles to `sdk._core` Python extension
- `crates/node` — napi-rs wrapper crate, compiles to native `.node` module
- `crates/python-stubs` — Generates `.pyi` type stub files
- `python/sdk/` — Python package directory (distributed via maturin)
- `npm/` — Node.js package directory

### Core Pattern
- `QuickNodeSdk` is the root entry point holding sub-clients (e.g., `admin: AdminApiClient`). All clients share a `SdkConfig(Arc<SdkConfigInner>)` wrapping one `reqwest` HTTP client and the API key.
- There are clients per QuickNode product, with functions mapping to API calls
- Request params and Responses should be fully typed structs

### Per-Sub-Client Config Pattern
Each sub-client module defines its own resolved config struct that holds the parsed, validated state derived from its public config type:
```rust
// In crates/core/src/<client>/mod.rs
pub(crate) struct Resolved<Name>Config {
    pub(crate) base_url: reqwest::Url,
    // other resolved fields...
}

impl Resolved<Name>Config {
    pub(crate) fn from_config(config: Option<&<Name>Config>) -> Result<Self, SdkError> {
        // parse and validate here
    }
}
```
`SdkConfigInner` holds one field per sub-client (e.g., `admin: admin::ResolvedAdminConfig`), and `SdkConfig` exposes a matching accessor (e.g., `fn admin(&self) -> &admin::ResolvedAdminConfig`). Call sites use `self.config.admin().base_url` instead of a flat `admin_base_url` field. Resolved config structs should be cheaply cloneable — prefer types like `reqwest::Url` (which implements `Clone`) and avoid heap allocations that would make cloning expensive; `SdkConfig` itself is a cheap clone via `Arc<SdkConfigInner>`.

- Any update to types in the core crate need to be checked for updates in the language crates (python, node)

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

### Testing
Core clients are tested using mocked API calls with wiremock. All functions making external http calls should be tested this way and test the happy path, errors, with params, and with bad params. Keep testing focused and flexible, avoid overtesting

## SDK-Specific Guidelines

### Polyglot consistency
- When adding a new public type to `crates/core`, export it across all three layers: Rust re-exports in `lib.rs`, Python `__init__.py` + `init_manual_override.pyi`, and TypeScript `sdk.d.ts`
- `python/sdk/__init__.py` is **manually maintained** — it is NOT auto-generated. Every new public struct/type must be added to both the `from sdk._core import (...)` block and the `__all__` list in this file
- When adding a new type with `#[cfg_attr(feature = "node", napi(object))]`, also add it to the named `export type { ... }` block in `npm/sdk.d.ts` — this is the user-facing type file and is not auto-updated by napi-rs
- When adding a new `#[napi(string_enum)]` Rust enum, it generates a TypeScript `const enum` in `npm/index.d.ts`. In `npm/sdk.d.ts`, these must be re-exported using a regular `export { ... }` (not `export type { ... }`), otherwise TypeScript consumers cannot use them as values (e.g., `StreamDataset.Block`)
- When updating `sdk.js` wrapper methods, verify the argument types match the underlying napi-rs constructor/method signature (object vs primitive)
- `python/sdk/__init__.pyi` is overwritten by `just python-build` — edit `init_manual_override.pyi` instead

### Security
- Never derive `Debug` on types containing sensitive values (API keys, tokens) without redaction — use `secrecy::SecretString` in internal structs, or a manual `Debug` impl that prints `[redacted]`
- Configurable URL overrides must be validated: normalize trailing slash before calling `.join()`

### Error handling
- Library constructors should return `Result`, not panic — use `.unwrap()` or `.expect()` only in examples and tests, never in library code
- Validate numeric config values before casting between signed/unsigned types (e.g., check `>= 0` before `i64 as u64`)

## Code style

### Imports
- Use direct imports instead of glob imports
- Keep modules at the top of the files

### Comments
- When doing anything out of the ordinary or breaking conventions or patterns, add a comment explaining the "why" behind it
