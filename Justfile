python-setup-env:
  uv venv && source .venv/bin/activate && uv pip install maturin

python-setup-env-fish:
  #!/usr/bin/env fish
  uv venv
  source .venv/bin/activate.fish
  uv pip install maturin

python-build:
  maturin develop && cargo run -p sdk-python-stubs && cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi

node-build:
  cd ./npm && npm install && npm run build && npm run test && cd ..

ruby-build:
  cargo build -p sdk-ruby --release
  cp target/release/libquicknode_sdk.dylib ruby/lib/quicknode_sdk.bundle

# Requires: maturin on PATH (e.g. pipx install maturin or brew install maturin)
macos-dist-python:
  uv python install 3.11 3.12 3.13 3.14
  mkdir -p dist
  maturin build --release \
    --target aarch64-apple-darwin \
    --interpreter $(uv python find 3.11) \
    --interpreter $(uv python find 3.12) \
    --interpreter $(uv python find 3.13) \
    --interpreter $(uv python find 3.14) \
    --out dist/
  @echo "Built wheels:"
  @ls dist/*macosx*arm64*.whl

macos-dist-node:
  cd npm && npm install
  cd npm && npx napi build --release --platform --target aarch64-apple-darwin --cargo-cwd ../crates/node
  mkdir -p dist
  cp npm/index.darwin-arm64.node dist/
  @echo "Built Node module:"
  @file dist/index.darwin-arm64.node

macos-dist-ruby:
  cargo build -p sdk-ruby --release --target aarch64-apple-darwin
  mkdir -p dist
  cp target/aarch64-apple-darwin/release/libquicknode_sdk.dylib dist/quicknode_sdk.bundle
  @echo "Built Ruby bundle:"
  @file dist/quicknode_sdk.bundle

test:
  cargo test -p sdk-core --lib

lint:
  cargo clippy --workspace --lib --tests -- -D warnings

# Bump version across all manifests, commit, and tag for release.
# Usage: just release 0.2.0
release version:
  sed -i.bak 's/^version = ".*"/version = "{{version}}"/' Cargo.toml && rm Cargo.toml.bak
  sed -i.bak 's/^version = ".*"/version = "{{version}}"/' pyproject.toml && rm pyproject.toml.bak
  uv lock
  sed -i.bak 's/"version": ".*"/"version": "{{version}}"/' npm/package.json && rm npm/package.json.bak
  cd npm && npm install --package-lock-only && cd ..
  sed -i.bak 's/s\.version *= *".*"/s.version = "{{version}}"/' ruby/quicknode_sdk.gemspec && rm ruby/quicknode_sdk.gemspec.bak
  git add Cargo.toml pyproject.toml uv.lock npm/package.json npm/package-lock.json ruby/quicknode_sdk.gemspec
  git commit -m "chore: release v{{version}}"
  git tag v{{version}}
  @echo "Tagged v{{version}}. Push with: git push && git push origin v{{version}}"
