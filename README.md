# Quicknode SDK

A unified SDK for building on Quicknode.

Rust SDK with Python, Node.js, and Ruby bindings.

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
    - [Rust crate only (crates.io)](#rust-crate-only-cratesio)
    - [All bindings together (Python / Node / Ruby)](#all-bindings-together-python--node--ruby)
    - [npm publish (`@quicknode/sdk`)](#npm-publish-quicknodesdk)
    - [PyPI publish (`quicknode-sdk`)](#pypi-publish-quicknode-sdk)
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
├── python/sdk/        # Python package with type hints
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
just python-setup-env
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

The Rust crate (`quicknode-sdk` on crates.io) versions independently from the Python, Node, and Ruby bindings. Its version lives in `crates/core/Cargo.toml`; the bindings share the workspace version in the root `Cargo.toml`.

#### Rust crate only (crates.io)

```bash
# 1. Bump the version in crates/core/Cargo.toml (e.g. 0.1.0 → 0.1.0-alpha.5)
#    Pre-release identifiers use SemVer 2.0 syntax: MAJOR.MINOR.PATCH-<id>.<N>
#    Examples: 0.1.0-alpha.4, 0.2.0-beta.1, 0.2.0-rc.1

# 2. Commit and push
git commit -am "chore: release quicknode-sdk 0.1.0-alpha.5"
git push

# 3. Validate the tarball (no upload)
cargo publish -p quicknode-sdk --dry-run

# 4. Publish (requires `cargo login` with a crates.io token)
cargo publish -p quicknode-sdk
```

The first publish claims the `quicknode-sdk` name permanently. Published versions are immutable — you cannot overwrite or delete them (only `cargo yank`, which hides but doesn't remove).

#### All bindings together (Python / Node / Ruby)

macOS (Apple Silicon) artifacts are built locally rather than on GitHub Actions to avoid the ~10× runner cost. Linux artifacts are built by CI when a GitHub release is published.

1. **Bump versions and commit:**
   ```bash
   just release 0.2.0
   git push
   ```

2. **Create the GitHub release** via the GitHub UI:
   - **Releases → Draft a new release**.
   - **Choose a tag** → type `v0.2.0` → **Create new tag on publish**.
   - Target branch: `main`.
   - Fill in title and notes (or click **Generate release notes**).
   - Click **Publish release**.

   This creates + pushes the tag and triggers `.github/workflows/release.yml`, which builds Linux artifacts and attaches them to the release.

3. **Build macOS arm64 artifacts locally and append them to the release:**
   ```bash
   just macos-build-and-publish 0.2.0
   ```

Step 3 requires the [`gh` CLI](https://cli.github.com/) authenticated to the repo. Intel macOS (`x86_64-apple-darwin`) is not shipped — users on Intel Macs install from source.

`just release` does **not** bump the Rust crate version (that's managed separately in `crates/core/Cargo.toml`). If you want the Rust crate to move in lockstep with a binding release, bump it manually in the same commit.

#### npm publish (`@quicknode/sdk`)

The Node package is published to npm as `@quicknode/sdk`. During the 3.x pre-release period, publishes use the `next` dist-tag so `npm install @quicknode/sdk` continues to resolve to the legacy 2.x release while `npm install @quicknode/sdk@next` pulls the rewrite.

The npm publish uses [napi-rs's multi-package layout](https://napi.rs/docs/deep-dive/release): one main package plus per-platform sub-packages (`@quicknode/sdk-linux-x64-gnu`, `@quicknode/sdk-darwin-arm64`, etc.) declared as `optionalDependencies`. Publishing is triggered manually via a GitHub Actions workflow so the macOS binary (built locally) can be uploaded to the GitHub release before publish.

Anyone with permission to run the `Publish npm` workflow in this repo can cut a release.

**Note on versions:** the git tag tracks the overall project version (e.g. `v0.1.0-alpha.5`) and is set in `crates/core/Cargo.toml` / the root `Cargo.toml`. The npm package version is set independently in `npm/package.json` (e.g. `3.0.0-alpha.5`) to stay compatible with the pre-existing `@quicknode/sdk` 2.x series on npm. The two versions do not need to match.

**Per-release flow:**

1. **Bump the npm version** in `npm/package.json` (e.g. `3.0.0-alpha.4` → `3.0.0-alpha.5`), commit, and push to `main`. (Bump the overall project version in `just release <version>` as part of the normal release flow above — this sets the git tag.)

2. **Create the GitHub release** via the GitHub UI:
   - Go to **Releases → Draft a new release**.
   - Click **Choose a tag**, type the new tag (e.g. `v0.1.0-alpha.5`), and select **Create new tag on publish**.
   - Target branch: `main`.
   - Fill in the title and release notes (or click **Generate release notes**).
   - Click **Publish release**.

   Publishing the release creates + pushes the tag, which triggers `.github/workflows/release.yml`. CI builds the Linux `.node` artifacts and attaches them to the release you just created.

3. **Wait for `release.yml` to finish.** Confirm the four Linux `index.*.node` artifacts are attached to the release.

4. **Build and upload the macOS arm64 binary** locally (Apple Silicon Mac required):
   ```bash
   just node-build
   gh release upload v0.1.0-alpha.5 npm/index.darwin-arm64.node
   ```

5. **Trigger the publish workflow.** From the GitHub UI: **Actions → Publish npm → Run workflow**, then enter the git tag (`v0.1.0-alpha.5`) and npm dist-tag (`next`). Or via CLI:
   ```bash
   gh workflow run publish-npm.yml -f tag=v0.1.0-alpha.5 -f npm_tag=next
   ```

6. **Verify.**
   ```bash
   npm view @quicknode/sdk dist-tags
   # Expected: next: 3.0.0-alpha.5, latest: 2.6.0 (unchanged)
   ```

Users can install the pre-release with `npm install @quicknode/sdk@next`.

#### PyPI publish (`quicknode-sdk`)

The Python package is published to PyPI as `quicknode-sdk`. Wheels and the sdist are built by `release.yml` on every GitHub release and attached as artifacts; the `Publish PyPI` workflow downloads them from a release tag and uploads to PyPI via `twine`.

**Version format:** PyPI uses PEP 440, so pre-releases are written without a hyphen. `0.1.0-alpha.6` → `0.1.0a6` in `pyproject.toml`.

**Per-release flow:**

1. **Bump the version** in `pyproject.toml` (e.g. `0.1.0a6` → `0.1.0a7`) and run `uv lock` to refresh `uv.lock`. Commit and push.

2. **Create the GitHub release** as described in the main Releasing section — this triggers `release.yml`, which builds the Linux wheels and sdist and attaches them to the release as `quicknode_sdk-*.whl` and `quicknode_sdk-*.tar.gz`.

3. **Wait for `release.yml` to finish.** Confirm 16 wheels (4 Python versions × 4 Linux targets) and 1 sdist are attached to the release.

4. **Trigger the publish workflow.** From the GitHub UI: **Actions → Publish PyPI → Run workflow**, then enter the git tag. Or via CLI:
   ```bash
   gh workflow run publish-pypi.yml -f tag=v0.1.0-alpha.6
   ```

5. **Verify.**
   ```bash
   pip install quicknode-sdk==0.1.0a6
   python -c "import sdk; print(sdk.QuicknodeSdk)"
   ```

**First publish:** the repo secret `PYPI_API_TOKEN` must be set. Project-scoped tokens only work after the project exists on PyPI, so the first upload needs a user-scoped token; rotate to a project-scoped token after.

## License

MIT
