//! Visual-baseline suite: every capturable surface of the four binaries,
//! driven as real processes in PTYs via the tuisnap library and gated
//! cell-exact (`.ansi`), content (`.txt`), render-level (`.html`) and
//! pixel-exact (`.png`) against the frozen grouped store at
//! `VISUAL_BASELINE_STORE` (scratch under `VISUAL_BASELINE_ACTUAL` or
//! `$RUN_DIR/tuisnap`; never in-tree `snapshots/`).
//!
//! This suite replaces the retired `tools/tuisnap_baseline.sh`; the capture
//! matrix and its rationale live in `docs/baseline/tuisnap-coverage.md` and
//! `docs/baseline/snapshots-v2.md` (grouped taxonomy). Capture names are
//! grouped paths (`<app>/<sub_group>/<leaf>`); argv, boot needles, send steps
//! and per-capture timeouts are the bash runner's, verbatim (via
//! [`tuisnap::pty::run_once`], the same runner the `tuisnap run` CLI used);
//! the `pointer` module adds the mouse/resize group the CLI could not
//! express (hover, drag-select, wheel scroll-fade, resize sequences). The
//! `audit` module generates the 10-fixture × 5 sizes × 5 colours audit
//! matrix data-drivenly; the audit-flow variant matrices live in
//! `showcase.rs` (keyboard) and `pointer.rs` (drag-select).
//!
//! Every capture test is `#[ignore]`d: default `cargo nextest run` compiles
//! the suite and runs only the cheap non-PTY [`store_integrity`] check. Run
//! the PTY baseline explicitly:
//!
//! ```sh
//! VISUAL_BASELINE_STORE=.../snapshots cargo nextest run \
//!   -E 'binary(visual_baseline) & test(store_integrity)'
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(holla_)'
//! cargo nextest run --run-ignored only --ignore-default-filter -E 'test(rebuild_review_html)'
//! ```
//!
//! Gate policy (fail-closed): only `matched` passes. This port never writes
//! the oracle, never runs `tuisnap accept`, and never blesses candidate
//! output. Missing oracle data is a failed gate.

#![cfg(any(target_os = "macos", target_os = "linux"))]
#![allow(
    missing_docs,
    unreachable_pub,
    missing_debug_implementations,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::arithmetic_side_effects,
    reason = "PTY visual harness fails closed; case tables are copied verbatim from the frozen tag"
)]

mod audit;
mod holla;
mod jackin;
mod pointer;
mod showcase;
mod support;
mod tablepro;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tuisnap::grouped::{self, GroupedStore};
use tuisnap::{Profile, VENDORED_FACES};

const STORE_EXTS: [&str; 4] = ["ansi", "txt", "png", "html"];

/// Cheap non-PTY gate: the external oracle names match the suite, each
/// scenario is `group/sub_group/leaf` with exactly four artifacts, and the
/// store holds the frozen 7,550-key matrix. In-tree `snapshots/` is not
/// required and must not be used as the oracle.
#[test]
fn store_integrity() {
    let approved = support::oracle_store_path();
    let store = GroupedStore::new(&approved);
    let store_names: BTreeSet<String> = store
        .approved_names()
        .unwrap_or_else(|error| panic!("list approved names: {error}"))
        .into_iter()
        .collect();

    let (by_name, extra) = walk_store(&approved);
    assert!(
        extra.is_empty(),
        "oracle store has files that are not .ansi/.txt/.png/.html: {}",
        extra.join(", ")
    );

    let mut incomplete = Vec::new();
    for (name, exts) in &by_name {
        let missing: Vec<_> = STORE_EXTS
            .iter()
            .copied()
            .filter(|ext| !exts.contains(*ext))
            .collect();
        if !missing.is_empty() || exts.len() != STORE_EXTS.len() {
            incomplete.push(format!("{name} has {exts:?} (missing {missing:?})"));
        }
    }
    assert!(
        incomplete.is_empty(),
        "every scenario needs exactly four artifacts (.ansi/.txt/.png/.html): {}",
        incomplete.join("; ")
    );

    assert_eq!(
        store_names.len(),
        support::EXPECTED_STORE_KEYS,
        "oracle GroupedStore name count"
    );
    assert_eq!(
        by_name.len(),
        support::EXPECTED_STORE_KEYS,
        "walked oracle key count"
    );
    assert_eq!(
        by_name.len().saturating_mul(STORE_EXTS.len()),
        support::EXPECTED_STORE_KEYS.saturating_mul(STORE_EXTS.len()),
        "oracle artifact count"
    );

    for name in &store_names {
        grouped::validate_name(name).unwrap_or_else(|error| panic!("{error}"));
        let slashes = name.bytes().filter(|&b| b == b'/').count();
        assert!(
            slashes >= 2,
            "scenario `{name}` must be nested under group/sub_group/… (at least two `/`)"
        );
        let mut parts = name.rsplit('/');
        let color = parts.next().unwrap_or_else(|| panic!("color leaf"));
        let size = parts.next().unwrap_or_else(|| panic!("size folder"));
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
        if STORE_EXTS.contains(&ext) {
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

/// Write a fast file-link index at the scratch report path. Does not
/// re-render PNGs and does not embed them. Per-capture `#[ignore]` tests are
/// the gate; this only indexes on-disk actual vs approved bytes.
#[test]
#[ignore = "rebuilds scratch report.html; run after a capture; skip with --skip rebuild_review_html"]
fn rebuild_review_html() {
    let store = support::store();
    let mut renderer = Profile::default_profile()
        .renderer(&VENDORED_FACES)
        .unwrap_or_else(|error| panic!("vendored faces parse: {error}"));
    let report = store
        .report_with(&mut renderer, 1.0, "tuisnap visual report")
        .unwrap_or_else(|error| panic!("report generation: {error}"));
    eprintln!(
        "report: {} ({} captures, {} failed)",
        report.path.display(),
        report.outcomes.len(),
        report.failed()
    );
    assert_eq!(report.failed(), 0, "unmatched gates — review report.html");
}
