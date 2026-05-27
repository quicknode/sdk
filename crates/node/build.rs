// Build scripts have no way to signal failure to cargo other than panicking,
// so the workspace-wide bans on `panic!` / `expect` do not apply here.
#![allow(clippy::panic, clippy::expect_used)]

extern crate napi_build;

use std::fs;
use std::path::PathBuf;

fn main() {
    napi_build::setup();

    // Read the npm package version from npm/package.json and expose it as
    // an env var the source can read with env!("NPM_PACKAGE_VERSION").
    // The npm package version (3.x.y) differs from the workspace version
    // (0.x.y) because @quicknode/sdk 2.x already shipped on npm.
    let pkg_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("npm")
        .join("package.json");
    println!("cargo:rerun-if-changed={}", pkg_path.display());

    let contents = fs::read_to_string(&pkg_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", pkg_path.display()));
    let v: serde_json::Value =
        serde_json::from_str(&contents).expect("npm/package.json is not valid JSON");
    let version = v
        .get("version")
        .and_then(|x| x.as_str())
        .expect("npm/package.json missing string \"version\"")
        .to_string();

    println!("cargo:rustc-env=NPM_PACKAGE_VERSION={version}");
}
