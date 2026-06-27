# Go SDK Binding — Implementation Plan

Build a Go binding for the Quicknode SDK as a fifth language layer, reusing the Rust core through a UniFFI facade (`uniffi-bindgen-go`). Scope: full-surface facade, thin/derive style. This phase is the **Go SDK only** (no Terraform provider yet); the binding is internal scaffolding whose only intended consumer is a future provider, but the facade covers the full public surface so public distribution later is additive.

Toolchain pinned: `uniffi-bindgen-go v0.7.1+v0.31.0` ⇒ `uniffi = "0.31.0"`. Installed and verified.

## Stage 1: Prove the FFI path on one method — COMPLETE
**Goal**: facade exposing a single admin read method (`get_endpoints`) through UniFFI, generated Go under `go/`, and a Go test that **statically links** the `.a` and calls it.
**Success criteria**: `just go-build` produces the staticlib + regenerates Go; a Go test links the `.a` via cgo and asserts a real (mocked) `get_endpoints` round-trip plus a typed API-error mapping. Validated the highest-risk unknowns (static link, async→sync via block_on, codegen, error mapping).
**Status**: Complete. Both Go tests pass (`TestGetEndpointsRoundTrip`, `TestGetEndpointsApiError`); clippy clean with and without `go`; no regressions (217 core tests pass).

### Key finding that changed the architecture
The planned two-crate layout (plain `crates/go` facade + `#[derive(uniffi::Record)]` on core types) is **not viable** in uniffi 0.31: the derives require `crate::UniFfiTag`, which only `setup_scaffolding!` provides, but calling `setup_scaffolding!` in both core and the facade double-emits the runtime FFI symbols → 54 duplicate symbols at static-link. The documented escape (`#[uniffi::remote]` / `use_remote_type!`) forces hand-mirroring every core type's field list in the facade.

**Resolution (single-crate):** the UniFFI facade lives in `crates/core/src/go.rs`, gated on the `go` feature — `setup_scaffolding!`, the `QuicknodeError` enum, and the `#[uniffi::export]` `QuicknodeSdkClient`. The `#[cfg_attr(feature = "go", derive(uniffi::Record))]` annotations sit one-per-type on the core data types (true thin/derive, no mirroring). `crates/go` is now a **trivial cdylib/staticlib shim** (`pub use quicknode_sdk::go::*;`) that exists only to produce the linkable native artifact; it is the sole place `setup_scaffolding!` is reachable from, so the FFI symbols are emitted exactly once. tokio is a `go`-gated optional dep of core.

## Stage 2: Fan out to the full public surface
**Goal**: `go`-feature-gated UniFFI attributes on all public types/methods across admin, streams, webhooks, kvstore, sql. Explicitly validate the `DestinationAttributes` discriminated union (enum-with-data + `#[serde(flatten)]`) through the Go codegen.
**Success criteria**: full surface generates and compiles; `go/examples/` exercises representative methods per sub-client; no regressions to python/node/ruby builds (default build byte-identical).
**Status**: Not Started

## Stage 3: Release pipeline
**Goal**: `crates/go` added to workspace members; `build-go` job in `release.yml` mirroring `build-ruby` (cross for linux-gnu x64/arm via zigbuild; macOS-arm64 via a `macos-dist-go` recipe locally). The `.a` per target is uploaded to the GitHub release (lives there / in CI; not committed).
**Success criteria**: a tagged release builds the Go `.a` artifacts for all targets without destabilizing existing jobs; `release-bump` handles any Go version manifest.
**Status**: Not Started

## Notes / decided constraints
- Facade is a passthrough: no HTTP, no logic. Drift caught at compile time.
- Sync via shared `tokio` `OnceLock` runtime + `block_on` (mirrors Ruby binding). UniFFI async surfaces as blocking Go calls anyway.
- `SdkError` mapped to a facade-local UniFFI error enum at the boundary (it wraps `reqwest::Error`/`serde_json::Error`, which can't cross UniFFI).
- Generated Go committed; native `.a`/`.so`/`.dylib` gitignored, built fresh.
- `crate-type = ["staticlib", "cdylib"]` — cdylib for bindgen to inspect, staticlib for the link.
- Public Go-module distribution (committed-`.a` vs separate mirrored repo, `go/vX.Y.Z` tag scheme) is deliberately deferred — no external consumer yet.
