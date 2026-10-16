//! Visual-baseline suite: every capturable surface of the four binaries,
//! driven as real processes in PTYs through tuiscotti (`tuiscotti::tui`,
//! pinned to tailrocks/tuiscotti `main` in `Cargo.toml`) and gated against
//! the approved artifacts in `snapshots/`
//! (`<app>/<sub_group>/<cols>x<rows>/<palette>.<ext>`). Which artifacts are
//! committed and gated is the single constant [`support::COMMITTED`]
//! (one `.ansi` with a cursor trailer); scratch (actuals, review PNG/HTML,
//! diffs) lives under `target/tuiscotti/`.
//!
//! `harness = false` (libtest-mimic): scenarios come from the typed state
//! tables in [`scenarios`] and each one is its own test, named
//! `<app>::<group>::<leaf>…::<cols>x<rows>::<color>`, so one drift fails one
//! test and nextest filters by app, group, size or palette:
//!
//! ```sh
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline)'
//! cargo nextest run --run-ignored only -E 'binary(visual_baseline) & test(/^tablepro::/)'
//! cargo nextest run --profile smoke --run-ignored only    # 120x40/truecolor + 72x20/nocolor
//! VISUAL_FULL=1 cargo nextest run --run-ignored only -E 'test(/^holla::parity::/)'
//! ```
//!
//! Every capture test is ignored: the default `cargo nextest run` runs only
//! the cheap checks (`store_integrity`, `scenario_tables`, the cursor-trailer
//! round trip) and the four boot-mode tests.
//!
//! Migration status (`docs/visual-testing/tuiscotti-migration.md`): only
//! scenarios already re-approved on tuiscotti ([`support::is_reapproved`])
//! gate; every other capture is skipped unless `VISUAL_FULL=1`. Gate policy
//! is fail-closed: only a full match passes, and approval is never implicit
//! (`cargo xtask visual accept`, local only).

#![cfg(any(target_os = "macos", target_os = "linux"))]

mod ansi_cursor;
mod determinism;
mod isolation;
mod modes;
mod scenarios;
mod support;
mod tool;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use libtest_mimic::{Arguments, Failed, Trial};
use support::{COMMITTED, LEGACY_EXTS};

fn main() {
    if let Ok(tool) = std::env::var("VISUAL_TOOL") {
        if let Err(e) = tool::run(&tool) {
            eprintln!("{e}");
            std::process::exit(1);
        }
        return;
    }
    let args = Arguments::from_args();
    let mut trials = vec![
        check("store_integrity", store_integrity),
        check("scenario_tables", || {
            if let Err(e) = scenarios::check_tables() {
                panic!("scenario tables are inconsistent:\n{e}");
            }
        }),
        check(
            "ansi_cursor::ansi_cursor_round_trips_cells_and_visible_cursor",
            ansi_cursor::ansi_cursor_round_trips_cells_and_visible_cursor,
        ),
        check("isolation::env_isolation", isolation::env_isolation),
        check(
            "determinism::sources_are_deterministic",
            determinism::sources_are_deterministic,
        ),
        check(
            "determinism::scanner_ignores_strings_comments_and_tests",
            determinism::scanner_ignores_strings_comments_and_tests,
        ),
        check(
            "modes::showcase_boot_sets_terminal_modes",
            modes::showcase_boot_sets_terminal_modes,
        ),
        check(
            "modes::tablepro_boot_sets_terminal_modes",
            modes::tablepro_boot_sets_terminal_modes,
        ),
        check(
            "modes::holla_boot_sets_terminal_modes",
            modes::holla_boot_sets_terminal_modes,
        ),
        check(
            "modes::jackin_boot_sets_terminal_modes",
            modes::jackin_boot_sets_terminal_modes,
        ),
    ];
    for scenario in scenarios::all() {
        trials.push(
            Trial::test(scenario.test_name(), move || catch(|| scenario.run()))
                .with_ignored_flag(true),
        );
    }
    libtest_mimic::run(&args, trials).exit();
}

fn check(name: &str, body: fn()) -> Trial {
    Trial::test(name, move || catch(body))
}

/// Turn a panic (the suite's assertion style) into a test failure with the
/// panic message.
fn catch(body: impl FnOnce()) -> Result<(), Failed> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)).map_err(|e| {
        e.downcast_ref::<String>()
            .cloned()
            .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "test panicked".into())
            .into()
    })
}

fn known_ext(ext: &str) -> bool {
    LEGACY_EXTS.contains(&ext) || COMMITTED.iter().any(|a| a.ext() == ext)
}

/// Cheap non-PTY gate: the committed `snapshots/` names match the scenario
/// tables, each scenario is `group/sub_group/<cols>x<rows>/<color>` with
/// exactly the [`COMMITTED`] artifacts (re-approved) or the legacy tuisnap
/// four (pending re-approval), and the legacy `shots/` corpus is gone.
/// Pending rows and the narrow tier may be absent; nothing in the store may
/// be outside the tables.
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
        if exts != &committed && exts != &legacy {
            incomplete.push(format!(
                "{name} has {exts:?}, expected {committed:?} (re-approved) or {legacy:?} (pending)"
            ));
        }
    }
    assert!(
        incomplete.is_empty(),
        "scenarios with the wrong artifact set: {}",
        incomplete.join("; ")
    );
    let reapproved = by_name.values().filter(|exts| *exts == &committed).count();
    eprintln!(
        "re-approved on tuiscotti: {reapproved} of {} stored scenarios",
        by_name.len()
    );

    let all = scenarios::all();
    let suite: BTreeSet<String> = all.iter().map(|s| s.name.clone()).collect();
    let required: BTreeSet<String> = all
        .iter()
        .filter(|s| !s.pending)
        .map(|s| s.name.clone())
        .collect();
    let missing: Vec<_> = required.difference(&store_names).cloned().collect();
    let orphans: Vec<_> = store_names.difference(&suite).cloned().collect();
    assert!(
        orphans.is_empty(),
        "store contains names not in the scenario tables ({}): {:?}",
        orphans.len(),
        orphans
    );
    eprintln!(
        "store inventory: {} stored, {} scenarios ({} pending first approval)",
        store_names.len(),
        suite.len(),
        suite.len() - required.len(),
    );
    assert!(
        missing.is_empty(),
        "store is missing approved snapshots ({}): {:?}",
        missing.len(),
        missing
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
