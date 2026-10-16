//! Visual-baseline suite: every capturable surface of the four binaries,
//! driven as real processes in PTYs through tuiscotti (`tuiscotti::tui`,
//! pinned to tailrocks/tuiscotti `main` in `Cargo.toml`) and gated against
//! the approved artifacts in `snapshots/`
//! (`<app>/<sub_group>/<cols>x<rows>/<palette>.<ext>`). Which artifacts are
//! committed and gated (`.ansi` cell-exact, `.txt` content, `.png`
//! decoded-pixel, `.html` render-level) is the single constant
//! [`support::COMMITTED`]; scratch (actuals, diffs) lives under
//! `target/tuiscotti/`.
//!
//! The capture matrix and its rationale live in
//! `docs/baseline/tuisnap-coverage.md` and `docs/baseline/snapshots-v2.md`
//! (grouped taxonomy). Capture names are grouped paths
//! (`<app>/<sub_group>/<leaf>`); argv, boot needles, send steps and
//! per-capture timeouts are the original bash runner's, verbatim. The
//! `pointer` module adds the mouse/resize group (hover, drag-select, wheel
//! scroll-fade, resize sequences). The `audit` module generates the
//! 10-fixture × 5 sizes × 5 colours audit matrix data-drivenly; the
//! audit-flow variant matrices live in `showcase.rs` (keyboard) and
//! `pointer.rs` (drag-select).
//!
//! Every capture test is `#[ignore]`d: default `cargo nextest run` compiles
//! the suite and runs only the cheap non-PTY [`store_integrity`] check.
//!
//! Migration status (`docs/visual-testing/tuiscotti-migration.md`): the
//! committed store still holds the retired tuisnap engine's approvals. Only
//! groups in [`support::REAPPROVED_PREFIXES`] gate on tuiscotti; every other
//! capture is skipped unless `VISUAL_FULL=1`:
//!
//! ```sh
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
//! VISUAL_FULL=1 cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(holla_)'
//! # approve reviewed actuals (local only, never in CI):
//! VISUAL_ACCEPT=showcase/pages cargo nextest run --run-ignored only --ignore-default-filter -E 'test(=accept_actuals)'
//! ```
//!
//! Gate policy (fail-closed): only a full match passes; a missing approved
//! artifact fails. Approval is never implicit: [`accept_actuals`] is the only
//! writer of `snapshots/`, and it runs only on explicit request.

#![cfg(any(target_os = "macos", target_os = "linux"))]

mod ansi_cursor;
mod audit;
mod holla;
mod jackin;
mod modes;
mod pointer;
mod showcase;
mod support;
mod tablepro;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use support::{COMMITTED, LEGACY_EXTS};

fn known_ext(ext: &str) -> bool {
    LEGACY_EXTS.contains(&ext) || COMMITTED.iter().any(|a| a.ext() == ext)
}

/// Cheap non-PTY gate: committed `snapshots/` names match the suite, each
/// scenario is `group/sub_group/<cols>x<rows>/<color>` with exactly the
/// [`COMMITTED`] artifacts (re-approved groups) or the legacy tuisnap four
/// (groups pending re-approval), and the legacy `shots/` corpus is gone.
#[test]
fn store_integrity() {
    let approved = support::approved_root();
    let (by_name, extra) = walk_store(&approved);
    assert!(
        extra.is_empty(),
        "snapshots/ has files outside the committed artifact set: {}",
        extra.join(", ")
    );
    let store_names: BTreeSet<String> = by_name.keys().cloned().collect();

    let committed: BTreeSet<String> = COMMITTED.iter().map(|a| a.ext().to_string()).collect();
    let legacy: BTreeSet<String> = LEGACY_EXTS.iter().map(|e| e.to_string()).collect();
    let mut incomplete = Vec::new();
    for (name, exts) in &by_name {
        let want = if support::is_reapproved(name) {
            &committed
        } else {
            &legacy
        };
        if exts != want {
            incomplete.push(format!("{name} has {exts:?}, expected {want:?}"));
        }
    }
    assert!(
        incomplete.is_empty(),
        "scenarios with the wrong artifact set: {}",
        incomplete.join("; ")
    );

    for name in &store_names {
        tuiscotti::grouped::validate_name(name).unwrap_or_else(|e| panic!("{e}"));
        let slashes = name.bytes().filter(|&b| b == b'/').count();
        assert!(
            slashes >= 2,
            "scenario `{name}` must be nested under group/sub_group/… (at least two `/`)"
        );
        let mut parts = name.rsplit('/');
        let color = parts.next().expect("color leaf");
        let size = parts.next().expect("size folder");
        assert!(
            matches!(color, "truecolor" | "256" | "16" | "none" | "nocolor"),
            "scenario `{name}` last component must be a color suffix"
        );
        let size_ok = size.split_once('x').is_some_and(|(cols, rows)| {
            !cols.is_empty()
                && !rows.is_empty()
                && cols.chars().all(|ch| ch.is_ascii_digit())
                && rows.chars().all(|ch| ch.is_ascii_digit())
        });
        assert!(
            size_ok,
            "scenario `{name}` must put terminal size in its own <cols>x<rows> folder"
        );
    }

    let suite = support::suite_capture_names();
    let pending: Vec<_> = suite.difference(&store_names).cloned().collect();
    let orphans: Vec<_> = store_names.difference(&suite).cloned().collect();
    assert!(
        orphans.is_empty(),
        "store contains stale names not in suite inventory ({}): {:?}",
        orphans.len(),
        orphans
    );
    eprintln!(
        "store inventory: {} approved, {} suite captures, {} pending first approval",
        store_names.len(),
        suite.len(),
        pending.len()
    );
    assert!(
        pending.is_empty(),
        "store is missing approved snapshots ({}): {:?}",
        pending.len(),
        pending
    );

    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR")).join("shots").exists(),
        "legacy shots/ corpus must be deleted"
    );
}

fn walk_store(root: &Path) -> (BTreeMap<String, BTreeSet<String>>, Vec<String>) {
    let mut by_name: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut extra = Vec::new();
    walk_store_dir(root, root, &mut by_name, &mut extra);
    (by_name, extra)
}

fn walk_store_dir(
    root: &Path,
    dir: &Path,
    by_name: &mut BTreeMap<String, BTreeSet<String>>,
    extra: &mut Vec<String>,
) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("list {}: {e}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("list {}: {e}", dir.display()))
            .path();
        if path.is_dir() {
            walk_store_dir(root, &path, by_name, extra);
            continue;
        }
        if support::is_macos_platform_metadata(&path) {
            continue;
        }
        let rel = posix_rel(root, &path);
        let Some((name, ext)) = rel.rsplit_once('.') else {
            extra.push(rel);
            continue;
        };
        if known_ext(ext) {
            by_name
                .entry(name.to_string())
                .or_default()
                .insert(ext.to_string());
        } else {
            extra.push(rel);
        }
    }
}

fn posix_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => s.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Explicit, local-only approval: copy the reviewed actuals under
/// `target/tuiscotti/actual/<VISUAL_ACCEPT prefix>` into `snapshots/`,
/// exactly the [`COMMITTED`] artifacts, removing any other legacy artifact
/// of an accepted scenario so the store follows the format set. Requires
/// `VISUAL_ACCEPT` (a capture-name prefix, `*` for everything); never run
/// in CI. tuiscotti's `GroupedStore::accept_all` always copies all four
/// artifacts, so it cannot follow a smaller format set.
#[test]
#[ignore = "approves reviewed actuals into snapshots/; set VISUAL_ACCEPT=<prefix> and run explicitly"]
fn accept_actuals() {
    let prefix = std::env::var("VISUAL_ACCEPT")
        .expect("set VISUAL_ACCEPT=<capture-name prefix> (or `*`) to approve actuals");
    let prefix = if prefix == "*" { String::new() } else { prefix };
    let actual = support::actual_root();
    let approved = support::approved_root();
    let (actuals, _) = walk_store(&actual);
    let mut accepted = 0usize;
    for name in actuals.keys().filter(|n| n.starts_with(&prefix)) {
        for ext in LEGACY_EXTS {
            let target = approved.join(format!("{name}.{ext}"));
            if !COMMITTED.iter().any(|a| a.ext() == ext) && target.exists() {
                std::fs::remove_file(&target)
                    .unwrap_or_else(|e| panic!("remove {}: {e}", target.display()));
            }
        }
        for artifact in COMMITTED {
            let source = actual.join(format!("{name}.{}", artifact.ext()));
            let target = approved.join(format!("{name}.{}", artifact.ext()));
            std::fs::create_dir_all(target.parent().expect("nested name"))
                .unwrap_or_else(|e| panic!("mkdir for {}: {e}", target.display()));
            std::fs::copy(&source, &target).unwrap_or_else(|e| {
                panic!("copy {} -> {}: {e}", source.display(), target.display())
            });
        }
        accepted += 1;
    }
    eprintln!("accepted {accepted} scenario(s) under `{prefix}`");
    assert!(
        accepted > 0,
        "no actuals under target/tuiscotti/actual/{prefix}"
    );
}
