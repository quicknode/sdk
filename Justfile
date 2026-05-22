python-setup:
  uv venv && uv sync

python-build:
  uvx maturin develop && cargo run -p sdk-python-stubs && cp python/sdk/init_manual_override.pyi python/sdk/__init__.pyi

node-build:
  cd ./npm && npm install && npm run build && npm run test && cd ..

ruby-build:
  cargo build -p sdk-ruby --release
  cp target/release/libquicknode_sdk.dylib ruby/lib/quicknode_sdk/quicknode_sdk.bundle

macos-dist-python:
  uv python install 3.11 3.12 3.13 3.14
  mkdir -p dist
  uvx maturin build --release \
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
  cd npm && npx napi build --release --platform --target aarch64-apple-darwin --manifest-path ../crates/node/Cargo.toml --output-dir .
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

# Build macOS arm64 artifacts locally and upload to an existing GitHub release.
# Usage: just macos-build-and-publish 0.2.0
# Precondition: tag vX.Y.Z has been pushed and CI has published the release.
macos-build-and-publish version:
  #!/usr/bin/env bash
  set -euo pipefail
  if ! gh release view "v{{version}}" >/dev/null 2>&1; then
    echo "Error: release v{{version}} not found. Push the tag and let CI publish it first." >&2
    exit 1
  fi
  # Clean dist/ so stale artifacts from a previous version's build don't get uploaded.
  rm -rf dist
  just macos-dist-python
  just macos-dist-node
  just macos-dist-ruby
  # Stage the compiled bundle under ruby/lib/quicknode_sdk so the platform gem picks it up,
  # then build the arm64-darwin gem.
  mkdir -p ruby/lib/quicknode_sdk
  cp dist/quicknode_sdk.bundle ruby/lib/quicknode_sdk/quicknode_sdk.bundle
  cd ruby && ruby ../scripts/build-platform-gem.rb arm64-darwin lib/quicknode_sdk/quicknode_sdk.bundle && gem build quicknode_sdk_platform.gemspec && rm quicknode_sdk_platform.gemspec && cd ..
  mv ruby/*.gem dist/
  gh release upload "v{{version}}" dist/*.whl dist/index.darwin-arm64.node dist/*.gem --clobber
  echo "Uploaded macOS arm64 artifacts to v{{version}}"

test:
  cargo test -p quicknode-sdk --lib

lint:
  cargo clippy --workspace --lib --tests -- -D warnings

# Bumps lockstep across Cargo (workspace + core crate), pyproject (PEP 440),
# Ruby gemspec, and npm/package.json. The npm version is auto-translated from
# 0.x.y... to 3.x.y... because @quicknode/sdk 2.x already exists on npm.
# Usage: just release-bump 0.2.0
# Bump versions across all manifests, commit + tag locally for release.
release-bump version:
  #!/usr/bin/env bash
  set -euo pipefail
  raw_version="{{version}}"
  if [[ "$raw_version" =~ ^v ]]; then
    echo "Error: version '$raw_version' must not start with 'v'. The 'v' prefix is added automatically when tagging. Try: just release-bump ${raw_version#v}" >&2
    exit 1
  fi
  if [[ ! "{{version}}" =~ ^0\. ]]; then
    echo "Error: version '{{version}}' must start with '0.' (npm auto-translate assumes 0.x → 3.x). Update release-bump when 0.x graduates." >&2
    exit 1
  fi
  npm_version="3.${raw_version#0.}"
  py_version=$(echo "$raw_version" | sed -E 's/-alpha\.([0-9]+)$/a\1/; s/-beta\.([0-9]+)$/b\1/; s/-rc\.([0-9]+)$/rc\1/')
  sed -i.bak 's/^version = ".*"/version = "{{version}}"/' Cargo.toml && rm Cargo.toml.bak
  sed -i.bak 's/^version = ".*"/version = "{{version}}"/' crates/core/Cargo.toml && rm crates/core/Cargo.toml.bak
  sed -i.bak "s/^version = \".*\"/version = \"$py_version\"/" pyproject.toml && rm pyproject.toml.bak
  uv lock
  sed -i.bak "s/\"version\": \".*\"/\"version\": \"$npm_version\"/" npm/package.json && rm npm/package.json.bak
  cd npm && npm install --package-lock-only && cd ..
  # Regenerate the napi platform loader so the version literals it embeds (~26 sites
  # used by NAPI_RS_ENFORCE_VERSION_CHECK) match the bumped npm version. This runs
  # a full Rust release build of crates/node — slow but the only way napi-cli emits
  # an accurate loader.
  cd npm && npm install && npm run build && cd ..
  sed -i.bak 's/s\.version *= *".*"/s.version = "{{version}}"/' ruby/quicknode_sdk.gemspec && rm ruby/quicknode_sdk.gemspec.bak
  git add Cargo.toml crates/core/Cargo.toml pyproject.toml uv.lock npm/package.json npm/package-lock.json npm/index.js ruby/quicknode_sdk.gemspec
  git commit -m "chore: release v{{version}}"
  git tag v{{version}}
  echo "Tagged v{{version}}. Next: just release-prepare {{version}}  (or push manually with: just release-push {{version}})"

# Push the release commit + tag to origin.
release-push version:
  git push
  git push origin v{{version}}

# Triggers .github/workflows/release.yml which builds Linux artifacts.
# Create the GitHub release for the pushed tag, generating notes from commits.
release-create-tag version:
  gh release create v{{version}} --generate-notes --target main --title "v{{version}}"

# Wait for release.yml (triggered by the release publish event) to finish for this tag.
release-wait-ci version:
  #!/usr/bin/env bash
  set -euo pipefail
  echo "Waiting for release.yml run for tag v{{version}}..."
  for attempt in $(seq 1 30); do
    run_id=$(gh run list --workflow=release.yml --event=release --limit 20 --json databaseId,headBranch \
      --jq '.[] | select(.headBranch == "v{{version}}") | .databaseId' | head -n1)
    if [[ -n "${run_id:-}" ]]; then
      echo "Found release.yml run $run_id for v{{version}}"
      gh run watch "$run_id" --exit-status
      exit 0
    fi
    echo "  attempt $attempt/30: run not visible yet, sleeping 5s..."
    sleep 5
  done
  echo "Error: timed out waiting for release.yml run for v{{version}} to appear." >&2
  exit 1

# Validate the Rust crate tarball without uploading.
release-cargo-publish-check:
  cargo publish -p quicknode-sdk --dry-run

# Publish the Rust crate to crates.io. Requires `cargo login` first.
release-cargo-publish:
  cargo publish -p quicknode-sdk

# Trigger the PyPI publish workflow for an existing release tag.
release-trigger-pypi version:
  gh workflow run publish-pypi.yml -f tag=v{{version}}

# Trigger the npm publish workflow for an existing release tag.
release-trigger-npm version npm_tag="next":
  gh workflow run publish-npm.yml -f tag=v{{version}} -f npm_tag={{npm_tag}}

# Trigger the RubyGems publish workflow for an existing release tag.
release-trigger-rubygems version:
  gh workflow run publish-rubygems.yml -f tag=v{{version}}

# Trigger all three binding publish workflows.
release-trigger-all version npm_tag="next":
  just release-trigger-pypi {{version}}
  just release-trigger-npm {{version}} {{npm_tag}}
  just release-trigger-rubygems {{version}}

# After this finishes, the GitHub release exists with all Linux + macOS
# artifacts attached, but nothing has been published to a registry yet.
# Pass yes=1 to skip the confirmation prompt (for automation).
# Phase 1: bump → push → tag → wait for CI → build + upload macOS artifacts.
release-prepare version yes="0":
  #!/usr/bin/env bash
  set -euo pipefail
  if [[ "{{yes}}" != "1" ]]; then
    echo "About to release v{{version}}:"
    echo "  1. Bump versions across Cargo (core+workspace), pyproject, npm, gemspec"
    echo "  2. Commit and tag locally"
    echo "  --- review diff and confirm before push ---"
    echo "  3. Push commit + tag to origin"
    echo "  4. Create GitHub release v{{version}}"
    echo "  5. Wait for release.yml CI to attach Linux artifacts"
    echo "  6. Build macOS arm64 artifacts locally and upload them to the release"
    echo
    read -r -p "Continue? [y/N] " response
    [[ "$response" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
  fi
  just release-bump {{version}}
  echo
  echo "=== Bump commit (HEAD) ==="
  git --no-pager show --stat HEAD
  echo
  echo "=== Diff vs previous commit ==="
  git --no-pager diff HEAD~1 HEAD -- Cargo.toml crates/core/Cargo.toml pyproject.toml npm/package.json ruby/quicknode_sdk.gemspec
  echo
  if [[ "{{yes}}" != "1" ]]; then
    echo "Review the bump above. Pushing will trigger CI builds against the tag."
    read -r -p "Push commit + tag v{{version}} to origin? [y/N] " response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
      echo "Aborted before push. The bump commit and tag exist locally — undo with:"
      echo "  git tag -d v{{version}} && git reset --hard HEAD~1"
      exit 1
    fi
  fi
  just release-push {{version}}
  just release-create-tag {{version}}
  just release-wait-ci {{version}}
  just macos-build-and-publish {{version}}
  echo
  echo "Phase 1 complete. Inspect https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/v{{version}}"
  echo "When ready: just release-publish {{version}}"

# Pass yes=1 to skip the confirmation prompt (for automation).
# Phase 2: publish to crates.io + trigger PyPI/npm/RubyGems publish workflows.
release-publish version npm_tag="next" yes="0":
  #!/usr/bin/env bash
  set -euo pipefail
  if [[ "{{yes}}" != "1" ]]; then
    echo "About to publish v{{version}} to:"
    echo "  - crates.io (quicknode-sdk)"
    echo "  - PyPI (quicknode-sdk)"
    echo "  - npm (@quicknode/sdk, dist-tag: {{npm_tag}})"
    echo "  - RubyGems (quicknode_sdk)"
    echo
    read -r -p "Continue? [y/N] " response
    [[ "$response" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
  fi
  just release-cargo-publish
  just release-trigger-all {{version}} {{npm_tag}}
  echo
  echo "Phase 2 dispatched. Crates.io is published; PyPI/npm/RubyGems workflows are running asynchronously."
  echo "Watch: gh run list --limit 5"
