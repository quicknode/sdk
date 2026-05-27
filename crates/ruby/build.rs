// Build scripts have no way to signal failure to cargo other than panicking,
// so the workspace-wide bans on `panic!` / `expect` do not apply here.
#![allow(clippy::panic, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn main() {
    rb_sys_build::rb_config().print_cargo_args();

    // Extract the gem version from the gemspec so the User-Agent reflects
    // the published gem version. Parsed line-by-line (rather than executing
    // the gemspec) so the build does not depend on a Ruby toolchain.
    let gemspec_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("ruby")
        .join("quicknode_sdk.gemspec");
    println!("cargo:rerun-if-changed={}", gemspec_path.display());

    let contents = fs::read_to_string(&gemspec_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", gemspec_path.display()));
    let version = contents
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            // Match lines like: s.version = "0.1.0-alpha.27"
            let after_eq = trimmed.strip_prefix("s.version")?.trim_start();
            let after_eq = after_eq.strip_prefix('=')?.trim();
            let after_quote = after_eq.strip_prefix('"')?;
            let end = after_quote.find('"')?;
            Some(after_quote[..end].to_string())
        })
        .expect("could not parse s.version from quicknode_sdk.gemspec");

    println!("cargo:rustc-env=GEM_VERSION={version}");
}
