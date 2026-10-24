//! Guards the external-consumer contract at the manifest and source level.
//!
//! The acceptance gate for this crate is that it is an *external* consumer:
//! it compiles against the public `termrock` facade only. It declares no
//! internal `termrock-*` crate as a dependency of its own, and its source
//! imports nothing but the facade — every widget, runtime and theme item
//! reaches this crate through the facade's re-exports.

use std::fs;
use std::path::Path;

/// The crate's own manifest, read straight from disk.
fn manifest() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("the consumer reads its own manifest")
}

/// The manifest with comment lines removed, so prose about the contract does
/// not count as a dependency.
fn active_manifest_lines() -> Vec<String> {
    manifest()
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Every `.rs` file in the crate, as (relative path, contents).
fn rust_sources() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    for dir in ["src", "tests"] {
        let entries =
            fs::read_dir(root.join(dir)).unwrap_or_else(|_| panic!("the crate has a {dir}/ dir"));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "rs") {
                let contents = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("{} is readable", path.display()));
                sources.push((
                    format!("{dir}/{}", path.file_name().unwrap().to_string_lossy()),
                    contents,
                ));
            }
        }
    }
    sources
}

#[test]
fn manifest_depends_on_the_public_facade_only() {
    let lines = active_manifest_lines();
    let joined = lines.join("\n");
    assert!(
        lines
            .iter()
            .any(|line| line.starts_with("termrock = { path = \"../crates/termrock\" }")),
        "the consumer depends on the public `termrock` facade by path"
    );
    let offenders: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .filter(|line| line.contains("termrock-"))
        .collect();
    assert!(
        offenders.is_empty(),
        "an external consumer must not name an internal `termrock-*` crate as its own \
         dependency; found: {offenders:?}"
    );
    assert!(
        joined.contains("[dependencies]") && joined.contains("[dev-dependencies]"),
        "the manifest keeps its dependency sections inspectable"
    );
}

#[test]
fn sources_import_only_the_public_facade() {
    for (path, contents) in rust_sources() {
        let offenders: Vec<&str> = contents
            .lines()
            .filter(|line| {
                let code = line.trim();
                (code.starts_with("use ") || code.starts_with("extern crate "))
                    && (code.contains("termrock_") || code.contains("termrock-"))
            })
            .collect();
        assert!(
            offenders.is_empty(),
            "{path} must import the `termrock` facade only, never an internal `termrock_*` \
             crate; found: {offenders:?}"
        );
    }
}
