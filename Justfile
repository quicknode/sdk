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
