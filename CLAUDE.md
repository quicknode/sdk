# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

### Rust
```bash
cargo check                                        # Type check all crates
cargo build -p quicknode-sdk                      # Build core crate
cargo test -p quicknode-sdk --lib                  # Run tests (excludes examples)
cargo run --example admin -p quicknode-sdk         # Run example (requires QN_API_KEY env var)
```

### Python
```bash
just python-setup                                 # Create .venv and uv sync (one-time)
just python-build                                 # Compile bindings + generate stubs
cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi # Manually override __init__ so we can overwrite the commands
```
Both recipes are shell-agnostic — they invoke `maturin` via `uvx`, so no venv activation is required and they work in bash, zsh, or fish without per-shell setup.

### Node.js
```bash
just node-build                                   # npm install + build + test
```

### Ruby
```bash
just ruby-build                                   # cargo build + copy .bundle artifact
```
The build compiles `crates/ruby` and copies the resulting `libquicknode_sdk.dylib` to `ruby/lib/quicknode_sdk/quicknode_sdk.bundle` (macOS) or equivalent `.so` on Linux. The native lib lives under `lib/quicknode_sdk/` so `require_relative "quicknode_sdk/quicknode_sdk"` picks the platform extension automatically.

## Verification

When verifying changes, use these commands based on what was modified:

- **Rust only** — `cargo check && just lint`
- **Python crate/bindings** — `just python-setup` (first time only), then `just python-build`
- **Node/npm** — `just node-build`
- **Ruby** — `just ruby-build`
- **Full verification** — `cargo check && just lint && just python-build && just node-build && just ruby-build && just test`

> Note: Do not use `cargo build` directly — Python bindings are compiled via maturin (`just python-build`).

if you can't run a just command, see what it's executing and run it manually

## Architecture

This is a polyglot SDK: one Rust core library with Python, Node.js, and Ruby bindings generated from the same types

### Workspace Layout
- `crates/core` — Pure Rust business logic (HTTP client, request/response types, errors)
- `crates/python` — PyO3 wrapper crate, compiles to `sdk._core` Python extension
- `crates/node` — napi-rs wrapper crate, compiles to native `.node` module
- `crates/ruby` — Magnus wrapper crate, compiles to native `.bundle`/`.so` module
- `crates/python-stubs` — Generates `.pyi` type stub files
- `python/sdk/` — Python package directory (distributed via maturin)
- `npm/` — Node.js package directory
- `ruby/` — Ruby package directory (`lib/quicknode_sdk.rb` entry point, `examples/`)

### Core Pattern
- `QuicknodeSdk` is the root entry point holding sub-clients (e.g., `admin: AdminApiClient`). All clients share a `SdkConfig(Arc<SdkConfigInner>)` wrapping one `reqwest` HTTP client and the API key.
- There are clients per Quicknode product, with functions mapping to API calls
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

- Any update to types in the core crate need to be checked for updates in the language crates (python, node, ruby)

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
- `ruby` feature — enables `magnus` dependency (optional); wrapping is done in the Ruby crate rather than via macros on core types
- `rust` feature — `bon` builder pattern for ergonomic Rust usage

### Error Handling
`SdkError` (`crates/core/src/errors.rs`) uses `thiserror` with five variants:
- `Http` — wraps `reqwest::Error` (further classified via `SdkError::http_kind()` → `HttpKind::{Timeout, Connect, Other}`)
- `Api` — non-2xx response with status code and raw body
- `Decode` — JSON parse failure with raw body for debugging
- `UrlParse` — invalid URL (wraps `url::ParseError`)
- `Config` — invalid configuration (string message)

Each binding exposes a typed exception hierarchy rooted at a shared base class so callers can `rescue` / `catch` / `except` by category. The mapping is:

| `SdkError` variant | Python / Ruby class | Node class | Base |
|---|---|---|---|
| `Config`, `UrlParse` | `ConfigError` | `ConfigError` | `QuicknodeError` |
| `Http` + `HttpKind::Timeout` | `TimeoutError` | `TimeoutError` | `HttpError` |
| `Http` + `HttpKind::Connect` | `ConnectionError` | `ConnectionError` | `HttpError` |
| `Http` + `HttpKind::Other` | `HttpError` | `HttpError` | `QuicknodeError` |
| `Api { status, body }` | `ApiError` (with `.status`, `.body`) | `ApiError` (with `.status`, `.body`) | `QuicknodeError` |
| `Decode { body, .. }` | `DecodeError` (with `.body`) | `DecodeError` (with `.body`) | `QuicknodeError` |

Each binding owns its mapping in a dedicated `errors.rs` file:
- **Python** — `crates/python/src/errors.rs` uses `create_exception!` macros; `map_sdk_err` sets `.status` / `.body` attributes via `setattr` on the exception instance. Exceptions are registered on the module in `add_to_module`.
- **Node** — `crates/node/src/errors.rs` encodes the variant, status, and body into a tagged message (`[<kind>|<status>|<body_len>]<msg>\x1f<body>`) because napi-rs only supports plain `napi::Error`. The JS wrapper `npm/errors.js` (`fromNapiError` + `wrapClient` Proxy) parses the prefix and rethrows as the typed subclass. All client methods must be wrapped via `wrapClient` so sync throws and rejected promises are both re-tagged.
- **Ruby** — `crates/ruby/src/errors.rs` uses `module.define_error` to build the class hierarchy under `QuicknodeSdk::Error`; `map_err` instantiates the class and sets `@status` / `@body` ivars (exposed via `attr_reader`-style methods). Classes are captured once in a `OnceLock<Opaque<ExceptionClass>>`.

When adding a new `SdkError` variant:
1. Add the variant to `crates/core/src/errors.rs` and update `http_kind()` if it's transport-level.
2. Update the `match` in each binding's `map_*_err` function — the compiler will flag missing arms in Python and Ruby (Node's match is also exhaustive on the kind string).
3. If the new variant should surface as a new exception class, add it to all three bindings + `npm/errors.js` + the exports in `python/sdk/__init__.py` + `npm/sdk.d.ts` + `npm/sdk.mjs`.
4. Update examples in all four languages to demonstrate the new class if user-facing.

### Python Binding Pattern
`crates/python/src/lib.rs` wraps core async methods using `pyo3_async_runtimes::tokio::future_into_py`. The Python API accepts individual keyword arguments instead of structs.

### Node.js Binding Pattern
`crates/node/src/lib.rs` uses `#[napi(constructor)]` and `#[napi(getter)]` macros. napi handles async conversion automatically.

### Ruby Binding Pattern
`crates/ruby/src/lib.rs` uses the `magnus` crate. All async SDK calls are wrapped via a single shared `tokio::runtime` (static `OnceLock`) using `.block_on()` to produce a synchronous Ruby API. Methods returning data return native Ruby `Hash`/`Array` (via `serde_magnus`); a per-client Ruby delegator in `ruby/lib/quicknode_sdk/clients/` wraps responses in `QuicknodeSdk::IndifferentHash` (a `Hash` subclass with `Hashie::Extensions::IndifferentAccess`) so callers can use either symbol or string keys. All parameters are passed as a single Ruby Hash with symbol keys (e.g. `get_endpoints(limit: 20)`). Required keys throw `ArgumentError` if missing; unknown keys also throw `ArgumentError` (validated via `validate_keys`). The magnus arity limit of 15 is why this pattern is used uniformly — all methods are registered with arity `1` (or `0` for zero-param methods). The native binding classes are registered under `QuicknodeSdk::Native::*` (via `#[magnus::init(name = "quicknode_sdk")]`); user-facing `QuicknodeSdk::SDK`/`Admin`/`Streams`/`Webhooks`/`KvStore` are pure-Ruby wrapper classes defined in `ruby/lib/quicknode_sdk/`.

When adding a new Ruby method:
1. Accept `opts: RHash` as the single parameter
2. Call `validate_keys(&opts, &["key1", "key2", ...])?;` as the first line
3. Use `hash_require_string/i64/i32/bool/vec_string` for required fields and `hash_get_*` for optional fields
4. Register with `method!(ClassName::method_name, 1)` in the `init` function on the `QuicknodeSdk::Native::*` class (the user-facing Ruby wrapper in `ruby/lib/quicknode_sdk/clients/` picks up new methods automatically via `method_missing`)
5. For methods returning data, the return type is `Result<magnus::Value, Error>` and the call ends with `.and_then(to_ruby)`. The Ruby delegator wraps the result in `QuicknodeSdk::IndifferentHash` automatically — no per-method code is needed on the Ruby side.
6. Add a corresponding RBS signature to `ruby/sig/quicknode_sdk.rbs` under the matching client class. Method name must match step 4; keyword args must match the `validate_keys` list from step 2 with types derived from the `hash_require_*` / `hash_get_*` accessors (`hash_require_string` → `String`, `hash_get_string` → optional `?key: String`, `hash_*_i32`/`i64` → `Integer`, `hash_*_bool` → `bool`, `hash_*_vec_string` → `Array[String]`, `hash_get_map_string_string` → `Hash[String, String]`). Use `untyped` as the return type for methods that return `Result<magnus::Value, Error>` (response is wrapped as a `Hashie::Mash` at the Ruby boundary) and `void` for methods returning `Result<(), Error>`. If a new exception class is added, also add it to the error hierarchy section at the top of the same file.

### Testing
Core clients are tested using mocked API calls with wiremock. All functions making external http calls should be tested this way and test the happy path, errors, with params, and with bad params. Keep testing focused and flexible, avoid overtesting

## SDK-Specific Guidelines

### Polyglot consistency
- When adding a new public type to `crates/core`, export it across all four layers: Rust re-exports in `lib.rs`, Python `__init__.py` + `init_manual_override.pyi`, TypeScript `sdk.d.ts`, and the Ruby binding in `crates/ruby/src/lib.rs`
- **Discriminated unions (Rust enum-with-data)** cannot be annotated with `#[pyclass]` or `#[napi(object)]`. Pattern: keep the enum pure-Rust in `crates/core` with `#[serde(tag = "...", content = "...")]`, then in each binding crate provide a typed surface that converts to/from the enum at the FFI boundary. Any core struct that flattens such an enum (e.g. via `#[serde(flatten)]`) also loses its `#[pyclass]` / `#[napi(object)]` and needs a wrapper in each binding. Reference implementations:
  - `crates/core/src/streams/stream.rs::DestinationAttributes` — stream destinations, wrapped by `crates/{python,node}/src/streams_destination.rs`.
  - `crates/core/src/webhooks/webhook.rs::TemplateArgs` — webhook filter templates, wrapped by `crates/{python,node}/src/webhooks_template.rs`. Ruby accepts the wire JSON directly (`{"templateId":..., "templateArgs":...}`) and relies on serde to deserialize into the enum.
- `python/sdk/__init__.py` is **manually maintained** — it is NOT auto-generated. Every new public struct/type must be added to both the `from sdk._core import (...)` block and the `__all__` list in this file
- When adding a new type with `#[cfg_attr(feature = "node", napi(object))]`, also add it to the named `export type { ... }` block in `npm/sdk.d.ts` — this is the user-facing type file and is not auto-updated by napi-rs
- When adding a new `#[napi(string_enum)]` Rust enum, it generates a TypeScript `const enum` in `npm/index.d.ts`. In `npm/sdk.d.ts`, these must be re-exported using a regular `export { ... }` (not `export type { ... }`), otherwise TypeScript consumers cannot use them as values (e.g., `StreamDataset.Block`)
- When updating `sdk.js` wrapper methods, verify the argument types match the underlying napi-rs constructor/method signature (object vs primitive)
- When adding a new export to `sdk.js`, also add it to the named exports in `npm/sdk.mjs` — ESM named exports cannot be spread dynamically and must be listed explicitly
- `python/sdk/__init__.pyi` is overwritten by `just python-build` — edit `init_manual_override.pyi` instead
- `ruby/sig/quicknode_sdk.rbs` is **manually maintained** — it is NOT auto-generated. It provides RBS type signatures so editor LSPs (VSCode Ruby LSP, Solargraph, RubyMine, Steep) autocomplete method names and keyword argument keys for `QuicknodeSdk::Admin/Streams/Webhooks/KvStore/DestinationAttributes` and the exception classes. Every change to method registration in `crates/ruby/src/lib.rs` (new method, renamed key, new arg, removed arg, type change) must be mirrored here in the same PR. Responses are typed as `untyped` because they're wrapped in `Hashie::Mash` at the Ruby boundary — that's intentional, do not try to type response shapes.
- Always update examples alongside the code changes

### Security
- Never derive `Debug` on types containing sensitive values (API keys, tokens) without redaction — use `secrecy::SecretString` in internal structs, or a manual `Debug` impl that prints `[redacted]`
- Configurable URL overrides must be validated: normalize trailing slash before calling `.join()`

### Error handling
- Library constructors should return `Result`, not panic — use `.unwrap()` or `.expect()` only in examples and tests, never in library code
- Validate numeric config values before casting between signed/unsigned types (e.g., check `>= 0` before `i64 as u64`)
- Map `SdkError` at the binding boundary only — keep core code returning `Result<_, SdkError>`, never a language-specific exception type. See the Error Handling section above for the typed exception hierarchy and how to add a new variant.
- When a binding needs new error metadata (status, body, retry info, etc.), add it to the `SdkError` variant first, then surface it on the exception class in each binding (PyO3 `setattr`, Ruby `ivar_set`, Node tagged-message prefix).
- Exception-raising tests belong in each language's example script (`crates/core/examples/admin_e2e.rs`, `python/examples/admin.py`, `npm/examples/admin.ts`, `ruby/examples/admin_e2e.rb`) — assert on the typed class, `status`, and `body` so regressions in the mapping layer fail loudly.

### Backwards Compatability
- If the release is still an 0.1.z release, we don't need to worry about backwards compatability as this is a greenfield project

### Documentation
- Keep inline documentation comments updated with each change
- The repo has **five READMEs**, each with a defined scope:
  - Root `README.md` — project landing page: structure, install index, development, releasing. **Do not** add per-language API examples here.
  - `crates/core/README.md` — Rust-only docs (renders on crates.io)
  - `python/README.md` — Python-only docs (renders on PyPI)
  - `npm/README.md` — Node.js-only docs (renders on npmjs)
  - `ruby/README.md` — Ruby-only docs (renders on RubyGems)
- Any user-facing change to a method, parameter, return type, error class, or environment variable must be reflected in **all four** per-language READMEs in the same PR. This matches the polyglot consistency rule for `__init__.py`, `sdk.d.ts`, and `quicknode_sdk.rbs` documented in §SDK-Specific Guidelines → Polyglot consistency.
- The Configuration env-var table and the Error Handling class table are duplicated verbatim across all four per-language READMEs. When one changes, update all four — keep them byte-identical.
- Per-language READMEs are wired into package metadata (`crates/core/Cargo.toml` `readme`, `pyproject.toml` `readme`, `npm/package.json` `files`, `ruby/quicknode_sdk.gemspec` `s.files`). When adding a new language or moving a README, update the corresponding manifest.

## Code style

### Imports
- Use direct imports instead of glob imports
- Keep modules at the top of the files

### Comments
- When doing anything out of the ordinary or breaking conventions or patterns, add a comment explaining the "why" behind it
