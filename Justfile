python-setup:
  uv venv && uv sync

python-build:
  uvx maturin develop && cargo run -p sdk-python-stubs && cp python/quicknode_sdk/init_manual_override.pyi python/quicknode_sdk/__init__.pyi

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
# Bump versions across all manifests on a release/vX.Y.Z branch.
# The branch is opened as a PR by release-prepare; the tag is created later
# against the merge commit on main, not here.
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
  current_branch=$(git rev-parse --abbrev-ref HEAD)
  if [[ "$current_branch" != "main" ]]; then
    echo "Error: must be on main to start a release (currently on '$current_branch')." >&2
    exit 1
  fi
  if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "Error: working tree is not clean. Commit or stash changes before bumping." >&2
    exit 1
  fi
  git checkout -b "release/v{{version}}"
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
  echo "Committed bump on branch release/v{{version}}. Next: just release-prepare {{version}}"

# Push the release branch to origin and open a PR for the bump commit.
release-open-pr version:
  #!/usr/bin/env bash
  set -euo pipefail
  git push -u origin "release/v{{version}}"
  gh pr create \
    --base main \
    --head "release/v{{version}}" \
    --title "chore: release v{{version}}" \
    --body "Automated release bump for v{{version}}. Merging this PR triggers the rest of the release flow (tag, GitHub release, CI artifacts, macOS upload)."

# Prompt to merge the release PR via `gh pr merge --squash --delete-branch`,
# then poll until the PR shows MERGED.
release-merge-pr version:
  #!/usr/bin/env bash
  set -euo pipefail
  pr_state=$(gh pr view "release/v{{version}}" --json state -q .state)
  if [[ "$pr_state" == "MERGED" ]]; then
    echo "PR for release/v{{version}} already merged."
    exit 0
  fi
  read -r -p "Merge release PR for v{{version}} now via 'gh pr merge --squash --delete-branch'? [y/N] " response
  if [[ "$response" =~ ^[Yy]$ ]]; then
    gh pr merge "release/v{{version}}" --squash --delete-branch
  else
    echo "Aborted. Merge the PR manually in GitHub, then re-run: just release-prepare {{version}}" >&2
    exit 1
  fi
  for attempt in $(seq 1 20); do
    pr_state=$(gh pr view "release/v{{version}}" --json state -q .state)
    if [[ "$pr_state" == "MERGED" ]]; then
      echo "PR merged."
      exit 0
    fi
    sleep 3
  done
  echo "Error: PR for release/v{{version}} did not reach MERGED state." >&2
  exit 1

# Tag the merge commit on main and push the tag.
release-tag-main version:
  #!/usr/bin/env bash
  set -euo pipefail
  git checkout main
  git pull --ff-only origin main
  git tag "v{{version}}"
  git push origin "v{{version}}"

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

# Promote an already-published npm version from the `next` dist-tag to `latest`.
# Updates the main @quicknode/sdk package and every platform sub-package
# (auto-discovered from npm/package.json's napi.targets).
# Precondition: `npm login` with publish rights on @quicknode/sdk.
# Usage: just release-promote-npm 3.1.1
release-promote-npm version:
  #!/usr/bin/env bash
  set -euo pipefail
  npm_version="{{version}}"
  if [[ "$npm_version" =~ ^v ]]; then
    echo "Error: version '$npm_version' must not start with 'v'." >&2
    exit 1
  fi
  pkg_names=$(cd npm && node -e '
    const fs = require("fs");
    const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
    const targetMap = {
      "x86_64-unknown-linux-gnu":   "linux-x64-gnu",
      "aarch64-unknown-linux-gnu":  "linux-arm64-gnu",
      "x86_64-unknown-linux-musl":  "linux-x64-musl",
      "aarch64-unknown-linux-musl": "linux-arm64-musl",
      "aarch64-apple-darwin":       "darwin-arm64",
    };
    const names = pkg.napi.targets.map(t => {
      if (!targetMap[t]) throw new Error("No abi for " + t);
      return pkg.name + "-" + targetMap[t];
    });
    names.push(pkg.name);
    console.log(names.join("\n"));
  ')
  echo "Pre-flight: verifying ${npm_version} is published for all packages..."
  missing=()
  while IFS= read -r pkg_name; do
    if ! npm view "${pkg_name}@${npm_version}" version >/dev/null 2>&1; then
      missing+=("${pkg_name}@${npm_version}")
    fi
  done <<< "$pkg_names"
  if (( ${#missing[@]} > 0 )); then
    echo "Error: the following packages are not published at ${npm_version}:" >&2
    printf '  %s\n' "${missing[@]}" >&2
    exit 1
  fi
  echo "Pre-flight OK."
  # npm 2FA: dist-tag add requires --otp on each call. Prompt once and reuse;
  # all calls run within seconds so a single 30s code covers the batch.
  read -r -p "Enter npm OTP (or leave blank if 2FA is disabled): " otp
  otp_flag=()
  if [[ -n "$otp" ]]; then
    otp_flag=(--otp="$otp")
  fi
  echo "Promoting..."
  while IFS= read -r pkg_name; do
    echo "+ npm dist-tag add ${pkg_name}@${npm_version} latest"
    npm dist-tag add "${pkg_name}@${npm_version}" latest ${otp_flag[@]+"${otp_flag[@]}"}
  done <<< "$pkg_names"
  count=$(echo "$pkg_names" | wc -l | tr -d ' ')
  echo "Promoted ${npm_version} → latest for ${count} packages (main + $((count - 1)) platform sub-packages)."
  just release-remove-npm-next "${npm_version}" "${otp}"

# Remove the `next` dist-tag from each npm package, but only on packages where
# `next` currently points at the given version. Skips packages where `next`
# points elsewhere (e.g. a newer pre-release) so in-flight betas aren't clobbered.
# Auto-discovers sub-packages from npm/package.json's napi.targets.
# Called automatically by release-promote-npm; can also be run standalone.
# Usage: just release-remove-npm-next 3.1.1 [otp]
release-remove-npm-next version otp="":
  #!/usr/bin/env bash
  set -euo pipefail
  npm_version="{{version}}"
  otp="{{otp}}"
  otp_flag=()
  if [[ -n "$otp" ]]; then
    otp_flag=(--otp="$otp")
  fi
  pkg_names=$(cd npm && node -e '
    const fs = require("fs");
    const pkg = JSON.parse(fs.readFileSync("package.json", "utf8"));
    const targetMap = {
      "x86_64-unknown-linux-gnu":   "linux-x64-gnu",
      "aarch64-unknown-linux-gnu":  "linux-arm64-gnu",
      "x86_64-unknown-linux-musl":  "linux-x64-musl",
      "aarch64-unknown-linux-musl": "linux-arm64-musl",
      "aarch64-apple-darwin":       "darwin-arm64",
    };
    const names = pkg.napi.targets.map(t => {
      if (!targetMap[t]) throw new Error("No abi for " + t);
      return pkg.name + "-" + targetMap[t];
    });
    names.push(pkg.name);
    console.log(names.join("\n"));
  ')
  echo "Removing 'next' tag where it points at ${npm_version}..."
  while IFS= read -r pkg_name; do
    current=$(npm view "${pkg_name}" dist-tags.next 2>/dev/null || true)
    if [[ -z "$current" ]]; then
      echo "  - ${pkg_name}: no 'next' tag, skipping"
    elif [[ "$current" != "$npm_version" ]]; then
      echo "  - ${pkg_name}: 'next' points at ${current}, skipping"
    else
      echo "+ npm dist-tag rm ${pkg_name} next"
      npm dist-tag rm "${pkg_name}" next ${otp_flag[@]+"${otp_flag[@]}"}
    fi
  done <<< "$pkg_names"

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
# Phase 1: bump on a release branch → open PR → merge → tag merge commit →
# create GitHub release → wait for CI → build + upload macOS artifacts.
release-prepare version yes="0":
  #!/usr/bin/env bash
  set -euo pipefail
  if [[ "{{yes}}" != "1" ]]; then
    echo "About to release v{{version}}:"
    echo "  1. Bump versions across Cargo (core+workspace), pyproject, npm, gemspec"
    echo "  2. Commit on branch release/v{{version}}"
    echo "  --- review diff and confirm before push ---"
    echo "  3. Push branch + open PR (review checkpoint)"
    echo "  4. Merge PR via 'gh pr merge --squash --delete-branch'"
    echo "  5. Tag the merge commit on main and push the tag"
    echo "  6. Create GitHub release v{{version}}"
    echo "  7. Wait for release.yml CI to attach Linux artifacts"
    echo "  8. Build macOS arm64 artifacts locally and upload them to the release"
    echo
    read -r -p "Continue? [y/N] " response
    [[ "$response" =~ ^[Yy]$ ]] || { echo "Aborted."; exit 1; }
  fi
  just release-bump {{version}}
  echo
  echo "=== Bump commit (HEAD) ==="
  git --no-pager show --stat HEAD
  echo
  echo "=== Diff vs main ==="
  git --no-pager diff main...HEAD -- Cargo.toml crates/core/Cargo.toml pyproject.toml npm/package.json ruby/quicknode_sdk.gemspec
  echo
  if [[ "{{yes}}" != "1" ]]; then
    echo "Review the bump above. Pushing will open a PR for review."
    read -r -p "Push branch release/v{{version}} and open PR? [y/N] " response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
      echo "Aborted before push. The bump commit exists locally on release/v{{version}} — undo with:"
      echo "  git checkout main && git branch -D release/v{{version}}"
      exit 1
    fi
  fi
  just release-open-pr {{version}}
  just release-merge-pr {{version}}
  just release-tag-main {{version}}
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
