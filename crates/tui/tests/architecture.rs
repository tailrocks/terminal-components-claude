//! Architecture checks (`COMPONENT_ARCHITECTURE.md` §16.5, §22.7), each
//! driven through `xtask` so the checks that need the whole workspace read
//! it once. `architecture::<name>` is the test name; the xtask check of the
//! same name is the mechanism.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::print_stdout
    )
)]

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .expect("workspace root")
}

fn xtask(args: &[&str]) -> (bool, String) {
    let out = Command::new("cargo")
        .args(["run", "-q", "-p", "xtask", "--"])
        .args(args)
        .current_dir(workspace_root())
        .output()
        .expect("run xtask");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

fn check(name: &str) {
    let (ok, text) = xtask(&["boundary", "--check", name]);
    assert!(ok, "{name} failed:\n{text}");
    println!("{text}");
}

mod architecture {
    use super::*;

    #[test]
    fn no_deprecated_or_legacy_api_usage() {
        check("no_deprecated_or_legacy_api_usage");
    }

    #[test]
    fn dependency_graph_is_exactly_the_declared_set() {
        check("dependency_graph_is_exactly_the_declared_set");
    }

    #[test]
    fn library_has_no_application_dependency() {
        check("library_has_no_application_dependency");
    }

    #[test]
    fn no_domain_vocabulary_in_the_library() {
        check("no_domain_vocabulary_in_the_library");
    }

    #[test]
    fn palette_literals_are_confined_to_theme_builtins() {
        check("palette_literals_are_confined_to_theme_builtins");
    }

    #[test]
    fn no_raw_background_parameter() {
        check("no_raw_background_parameter");
    }

    #[test]
    fn no_public_geometry_or_cache() {
        check("no_public_geometry_or_cache");
    }

    #[test]
    fn no_fn_pointer_extension_points() {
        check("no_fn_pointer_extension_points");
    }

    #[test]
    fn no_todo_or_unimplemented() {
        check("no_todo_or_unimplemented");
    }

    #[test]
    fn no_unsafe() {
        check("no_unsafe");
    }

    #[test]
    fn no_static_bound_in_component_surface() {
        check("no_static_bound_in_component_surface");
    }

    #[test]
    fn draw_takes_shared_self() {
        check("draw_takes_shared_self");
    }

    #[test]
    fn cache_types_are_derived_only() {
        check("cache_types_are_derived_only");
    }

    #[test]
    fn capability_has_no_unicode_field() {
        check("capability_has_no_unicode_field");
    }

    #[test]
    fn no_boolean_capability_parameter_on_grid() {
        check("no_boolean_capability_parameter_on_grid");
    }

    #[test]
    fn core_is_backend_free() {
        check("core_is_backend_free");
    }

    #[test]
    fn msrv_and_edition_are_unchanged() {
        check("msrv_and_edition_are_unchanged");
    }

    #[test]
    fn public_items_are_documented() {
        // `#![deny(missing_docs)]` at the crate root is the mechanism; this
        // pins the attribute so it cannot be removed silently
        let lib = std::fs::read_to_string(workspace_root().join("crates/tui/src/lib.rs"))
            .expect("lib.rs");
        assert!(lib.contains("#![deny(missing_docs)]"));
        assert!(lib.contains("#![doc = include_str!(\"../README.md\")]"));
    }

    #[test]
    fn doc_check_resolves_every_reference() {
        let (ok, text) = xtask(&["doc-check"]);
        assert!(ok, "doc-check failed:\n{text}");
        println!("{text}");
    }

    #[test]
    fn every_foreign_type_in_the_public_surface_is_re_exported() {
        // Adjudication M1: every ratatui type named by a `pub` signature has
        // a facade line; the facade names them explicitly and this pins it
        let lib = std::fs::read_to_string(workspace_root().join("crates/tui/src/lib.rs"))
            .expect("lib.rs");
        for ty in [
            "Buffer", "Cell", "Position", "Rect", "Color", "Style", "Frame", "Modifier",
        ] {
            assert!(lib.contains(ty), "{ty} is not re-exported at the root");
        }
        assert!(
            !lib.contains("pub use ratatui_core::text::"),
            "M1: ratatui text types are never flat re-exports"
        );
        let author = std::fs::read_to_string(workspace_root().join("crates/tui/src/author.rs"))
            .expect("author.rs");
        let raw_at = author
            .find("pub mod raw")
            .expect("M1: author::raw is the qualified escape module");
        let before_raw = &author[..raw_at];
        assert!(
            !before_raw.contains("pub use ratatui_core::text::"),
            "M1: text types only inside author::raw"
        );
    }
}
