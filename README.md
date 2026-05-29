# Quicknode SDK

A unified SDK for building on Quicknode.

Rust SDK with Python, Node.js, and Ruby bindings.

> **Pre-1.0**: While on `0.x`, releases may contain breaking changes. Check the [release notes](https://github.com/quicknode/sdk/releases) before upgrading.

## Table of Contents

- [Per-language docs](#per-language-docs)
- [Project Structure](#project-structure)
- [Installation](#installation)
- [Development](#development)
  - [Prerequisites](#prerequisites)
  - [Build Commands](#build-commands)
  - [Testing](#testing)
  - [Examples](#examples)
  - [Releasing](#releasing)
    - [Two-phase flow](#two-phase-flow)
    - [Building blocks](#building-blocks)
    - [Version conventions](#version-conventions)
    - [First-time setup](#first-time-setup)
- [License](#license)

## Per-language docs

API reference, configuration, and error handling for each language live next to the package — those are also the docs that render on each package listing.

- **Rust** — [`crates/core/README.md`](crates/core/README.md) (`quicknode-sdk` on crates.io)
- **Python** — [`python/README.md`](python/README.md) (`quicknode-sdk` on PyPI)
- **Node.js** — [`npm/README.md`](npm/README.md) (`@quicknode/sdk` on npm)
- **Ruby** — [`ruby/README.md`](ruby/README.md) (`quicknode_sdk` on RubyGems)

This file covers project structure, install index, and how to develop and release the SDK.

## Project Structure

```
sdk/
├── crates/
│   ├── core/          # Pure Rust business logic
│   ├── python/        # PyO3 bindings
│   ├── node/          # napi-rs bindings
│   └── ruby/          # magnus bindings
├── python/quicknode_sdk/  # Python package with type hints
├── npm/               # Node.js package with TypeScript types
├── ruby/              # Ruby package
└── pyproject.toml     # maturin build config
```

## Installation

| Language | Install |
|---|---|
| Rust    | `cargo add quicknode-sdk` — see [`crates/core/README.md`](crates/core/README.md) |
| Python  | `uv add quicknode-sdk` — see [`python/README.md`](python/README.md) |
| Node.js | `npm install @quicknode/sdk` — see [`npm/README.md`](npm/README.md) |
| Ruby    | `gem install quicknode_sdk` — see [`ruby/README.md`](ruby/README.md) |

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
cargo test -p quicknode-sdk

# Python (from project root)
just python-setup
just python-build

# Node.js (from npm/)
just node-build

# Ruby
just ruby-build

# Rust
cargo build -p quicknode-sdk
```

### Testing

```bash
just test
```

Runs the Rust unit tests for `quicknode-sdk` using [wiremock](https://github.com/LukeMathWalker/wiremock-rs) to mock HTTP responses — no API key required.

### Examples

```bash
# Rust
QN_SDK__API_KEY=replaceme cargo run --example admin -p quicknode-sdk --features rust

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

### Releasing

All four packages (Rust crate, Python wheels, Node `.node` module, Ruby gem) ship together from a single project version. The release flow is split into two `just` commands:

- **`just release-prepare <version>`** — bump versions, push, tag, build all Linux + macOS artifacts, attach them to a GitHub release. **No registry publish yet.**
- **`just release-publish <version>`** — push to crates.io and trigger the PyPI / npm / RubyGems publish workflows.

The pause between the two commands is the natural review point: inspect the GitHub release page and confirm all artifacts are present before anything goes public.

Requires the [`gh` CLI](https://cli.github.com/) authenticated to the repo, an Apple Silicon Mac (macOS arm64 artifacts are built locally to avoid the ~10× CI runner cost), and `cargo login` with a crates.io token.

#### Two-phase flow

```bash
# Phase 1: build a fully-staged GitHub release
just release-prepare 0.2.0

# Inspect https://github.com/<org>/<repo>/releases/tag/v0.2.0
# Confirm: Linux Python wheels, Linux .node files, Linux Ruby gem,
#          macOS arm64 wheels, index.darwin-arm64.node, arm64-darwin .gem.

# Phase 2: publish to crates.io + trigger registry publish workflows
just release-publish 0.2.0
```

`release-prepare` prompts twice — once up front before bumping, and once after the bump (showing the diff) before pushing the tag. `release-publish` prompts once before any registry write. Pass `yes=1` to skip prompts (e.g. for automation): `just release-prepare 0.2.0 yes=1`.

#### Building blocks

If a phase fails partway, resume with the individual recipes — they are also useful for reruns:

| Recipe | Purpose |
|---|---|
| `release-bump <version>` | Bump versions in all manifests, commit + tag locally |
| `release-push <version>` | `git push` + push tag |
| `release-create-tag <version>` | `gh release create` (triggers Linux CI build) |
| `release-wait-ci <version>` | Block until `release.yml` finishes attaching Linux artifacts |
| `macos-build-and-publish <version>` | Build macOS arm64 wheels / `.node` / `.gem` and `gh release upload` |
| `release-cargo-publish-check` | `cargo publish -p quicknode-sdk --dry-run` |
| `release-cargo-publish` | `cargo publish -p quicknode-sdk` |
| `release-trigger-pypi <version>` | Dispatch `publish-pypi.yml` |
| `release-trigger-npm <version> [npm_tag]` | Dispatch `publish-npm.yml` (default tag: `next`) |
| `release-trigger-rubygems <version>` | Dispatch `publish-rubygems.yml` |
| `release-trigger-all <version> [npm_tag]` | All three trigger recipes in sequence |

#### Version conventions

A single version argument drives every manifest, with two automatic translations applied by `release-bump`:

| Manifest | Format example for `0.1.0-alpha.6` |
|---|---|
| `Cargo.toml` (workspace) | `0.1.0-alpha.6` |
| `crates/core/Cargo.toml` | `0.1.0-alpha.6` |
| `pyproject.toml` (PEP 440) | `0.1.0a6` |
| `ruby/quicknode_sdk.gemspec` | `0.1.0-alpha.6` |
| `npm/package.json` | `3.0.0-alpha.6` (`0.x` → `3.x` because `@quicknode/sdk` 2.x already exists on npm) |

Pre-release identifiers use SemVer 2.0 syntax: `MAJOR.MINOR.PATCH-<id>.<N>` — e.g. `0.1.0-alpha.4`, `0.2.0-beta.1`, `0.2.0-rc.1`. The npm pre-release period uses the `next` dist-tag so `npm install @quicknode/sdk` continues to resolve to legacy 2.x while `npm install @quicknode/sdk@next` pulls the rewrite.

`release-bump` only accepts versions starting with `0.`. The `0.x → 3.x` mapping (and the recipe's guard) needs to be revisited when the project graduates to `1.0`.

#### First-time setup

- The first crates.io publish claims the `quicknode-sdk` name permanently. Published versions are immutable (only `cargo yank` is reversible, and it only hides).
- The first PyPI upload needs a user-scoped `PYPI_API_TOKEN`; rotate to a project-scoped token after.
- Verify a release reached each registry:
  ```bash
  cargo search quicknode-sdk
  pip install quicknode-sdk==0.1.0a6
  npm view @quicknode/sdk dist-tags
  gem search -e quicknode_sdk
  ```

## License

MIT
