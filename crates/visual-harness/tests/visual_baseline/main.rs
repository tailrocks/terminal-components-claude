#![cfg(any(target_os = "macos", target_os = "linux"))]

mod determinism;
mod isolation;
mod modes;
mod oracle;
mod overlay_sites;
mod scenarios;
mod startup_input;
mod startup_resize;
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
        check("store_ansi_round_trips", store_ansi_round_trips),
        check("oracle_integrity", oracle::integrity),
        check(
            "oracle::manifest_and_hash_are_stable",
            oracle::manifest_and_hash_are_stable,
        ),
        check(
            "oracle::store_check_accepts_and_rejects",
            oracle::store_check_accepts_and_rejects,
        ),
        check("overlay_sites", overlay_sites::overlay_sites),
        check(
            "overlay_sites::site_scanner_reads_functions",
            overlay_sites::site_scanner_reads_functions,
        ),
        check("isolation::env_isolation", isolation::env_isolation),
        check(
            "support::clear_actual_leaves_nothing",
            support::clear_actual_leaves_nothing,
        ),
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
        check(
            "startup_resize::a_resize_right_after_the_first_frame_repaints",
            startup_resize::a_resize_right_after_the_first_frame_repaints,
        ),
        check(
            "startup_resize::a_resize_queued_with_a_key_still_reaches_the_app",
            startup_resize::a_resize_queued_with_a_key_still_reaches_the_app,
        ),
        check(
            "startup_input::showcase_discards_keys_typed_before_start",
            startup_input::showcase_discards_keys_typed_before_start,
        ),
        check(
            "startup_input::tablepro_discards_keys_typed_before_start",
            startup_input::tablepro_discards_keys_typed_before_start,
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
    LEGACY_EXTS.contains(&ext) || COMMITTED.contains(&ext)
}

fn store_ansi_round_trips() {
    let root = support::approved_root();
    if !root.exists() {
        return;
    }
    let provenance =
        tuiscotti::render::frame_from_screen(&tuiscotti::Screen::blank(1, 1), "store").provenance;
    let (by_name, _) = walk_store(&root);
    let mut checked = 0usize;
    let mut bad = Vec::new();
    for name in by_name.keys().filter(|n| support::is_reapproved(n)) {
        let path = root.join(format!("{name}.ansi"));
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        match tuiscotti::formats::ansi::frame_from_ansi(&text, provenance.clone()) {
            Ok(frame) if tuiscotti::render::ansi_dump_with_cursor(&frame) == text => {}
            Ok(_) => bad.push(format!("{name}: does not re-encode byte-identical")),
            Err(e) => bad.push(format!("{name}: {e}")),
        }
        checked += 1;
    }
    assert!(
        bad.is_empty(),
        "approved .ansi files that do not round-trip ({}): {}",
        bad.len(),
        bad.join("; ")
    );
    eprintln!("{checked} re-approved .ansi file(s) round-trip");
}

fn store_integrity() {
    let approved = support::snapshots_root();
    if !approved.exists() && oracle::active() {
        eprintln!("snapshots/ is gone; the oracle is the store (oracle_integrity)");
        return;
    }
    let (by_name, extra) = walk_store(&approved);
    assert!(
        extra.is_empty(),
        "snapshots/ has files outside the committed artifact set: {}",
        extra.join(", ")
    );
    let store_names: BTreeSet<String> = by_name.keys().cloned().collect();

    let committed: BTreeSet<String> = COMMITTED.iter().map(|e| e.to_string()).collect();
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
