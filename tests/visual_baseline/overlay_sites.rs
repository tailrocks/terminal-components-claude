//! Overlay coverage (plan task T15): every place in `src/bin` that opens a
//! popup, menu, dialog or picker, or draws through the overlay layer, is an
//! entry of [`SITES`], keyed `<file under src/bin>::<enclosing fn>`, and
//! names the scenario rows that capture it, or a gap with the reason it has
//! none yet. `overlay_sites` fails when a site appears or disappears in the
//! source without this table following, or when a named row is not in the
//! scenario tables. Gaps are allowed but listed in every run; closing one
//! means adding an `OVERLAYS` row (open + both dismissals) and naming it.
//!
//! A site is a line, outside comments, strings and test modules, with one
//! of [`OPENERS`]. Several openers in one function form one site.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::determinism::{code_only, non_test};

/// What opens or draws an overlay in the four apps.
const OPENERS: [&str; 11] = [
    "req.menu(",
    "ModalTag::new(",
    "Dialog::new(",
    "Dialog::confirm(",
    "Dialog::destructive(",
    "Dialog::facts(",
    "Dialog::prompt(",
    "Picker::new(",
    ".push(Modal::",
    "begin_modal(",
    ".menu.open(",
];

/// Lines that mention an opener without opening anything.
const NOT_OPENING: [&str; 1] = ["unwrap_or(ModalTag::new("];

pub struct Site {
    pub key: &'static str,
    pub rows: &'static [&'static str],
    pub gap: &'static str,
}

const fn rows(key: &'static str, rows: &'static [&'static str]) -> Site {
    Site { key, rows, gap: "" }
}

const fn gap(key: &'static str, gap: &'static str) -> Site {
    Site {
        key,
        rows: &[],
        gap,
    }
}

const NO_ROW: &str = "no scenario row yet";

#[rustfmt::skip]
pub const SITES: &[Site] = &[
    // ------------------------------------------------------------- holla --
    rows("holla/app.rs::confirm_plan", &["holla/flows/upgrade/confirm"]),
    rows("holla/app.rs::draw", &["holla/flows/help/overlay", "holla/flows/quit/confirm"]),
    gap("holla/app.rs::menu_action", NO_ROW),
    gap("holla/app.rs::open_about", NO_ROW),
    rows("holla/app.rs::open_activities", &["holla/flows/activities/overlay", "holla/flows/activities/empty", "holla/flows/git/batch_picker"]),
    gap("holla/app.rs::open_alternatives", NO_ROW),
    rows("holla/app.rs::open_confirm_one", &["holla/flows/cargo/clean_confirm"]),
    gap("holla/app.rs::open_gate2", NO_ROW),
    rows("holla/app.rs::open_help", &["holla/flows/help/overlay"]),
    rows("holla/app.rs::open_quit_confirm", &["holla/flows/quit/confirm"]),
    gap("holla/app.rs::open_why", NO_ROW),
    gap("holla/app.rs::request_close_tab", NO_ROW),
    gap("holla/screens/activity.rs::chip_menu", NO_ROW),
    gap("holla/screens/activity.rs::output_menu", NO_ROW),
    gap("holla/screens/cleanup.rs::row_menu", NO_ROW),
    gap("holla/screens/disk.rs::row_menu", NO_ROW),
    gap("holla/screens/files.rs::open_actions_at", NO_ROW),
    gap("holla/screens/files.rs::open_jump", NO_ROW),
    gap("holla/screens/plan.rs::on_key", NO_ROW),
    gap("holla/screens/plan.rs::step_menu", NO_ROW),
    gap("holla/screens/review.rs::sequence_menu", NO_ROW),
    gap("holla/screens/snapshot.rs::body_menu", NO_ROW),
    // ------------------------------------------------------------ jackin --
    gap("jackin_preview/app.rs::form_changed", NO_ROW),
    gap("jackin_preview/app.rs::open_about", NO_ROW),
    rows("jackin_preview/app.rs::open_help", &["jackin/manager/help_overlay"]),
    rows("jackin_preview/app.rs::open_quit_confirm", &["jackin/manager/quit_confirm"]),
    gap("jackin_preview/screens/accounts.rs::on_key", NO_ROW),
    gap("jackin_preview/screens/accounts.rs::on_modal", NO_ROW),
    rows("jackin_preview/screens/accounts.rs::open_form", &["jackin/accounts/add_form", "jackin/accounts/add_form_required"]),
    gap("jackin_preview/screens/accounts.rs::open_menu", NO_ROW),
    gap("jackin_preview/screens/accounts.rs::remove", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::confirm_close", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::continue_spawn", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::on_modal", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::open_about", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::open_container_info", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::open_github", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::open_inspect", NO_ROW),
    rows("jackin_preview/screens/capsule.rs::open_palette", &["jackin/capsule/palette"]),
    gap("jackin_preview/screens/capsule.rs::open_rename", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::open_usage", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::request_exit", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::run_palette", NO_ROW),
    gap("jackin_preview/screens/capsule.rs::spawn_flow", NO_ROW),
    rows("jackin_preview/screens/cockpit.rs::on_key", &["jackin/cockpit/cancel_confirm"]),
    gap("jackin_preview/screens/cockpit.rs::open_failure", NO_ROW),
    rows("jackin_preview/screens/cockpit.rs::open_info", &["jackin/cockpit/info"]),
    gap("jackin_preview/screens/cockpit.rs::open_quit", NO_ROW),
    gap("jackin_preview/screens/cockpit.rs::step", NO_ROW),
    gap("jackin_preview/screens/config.rs::on_key", NO_ROW),
    gap("jackin_preview/screens/config.rs::on_modal", NO_ROW),
    gap("jackin_preview/screens/config.rs::open_env_form", NO_ROW),
    gap("jackin_preview/screens/config.rs::open_mount_form", NO_ROW),
    gap("jackin_preview/screens/config.rs::open_role_picker", NO_ROW),
    gap("jackin_preview/screens/config.rs::open_scope_picker", NO_ROW),
    gap("jackin_preview/screens/editor.rs::accounts_key", NO_ROW),
    gap("jackin_preview/screens/editor.rs::finish_role_load", NO_ROW),
    gap("jackin_preview/screens/editor.rs::finish_save", NO_ROW),
    gap("jackin_preview/screens/editor.rs::leave", NO_ROW),
    rows("jackin_preview/screens/editor.rs::open_preview", &["jackin/editor/save_preview"]),
    gap("jackin_preview/screens/editor.rs::open_role_picker", NO_ROW),
    gap("jackin_preview/screens/editor.rs::open_trust_dialog", NO_ROW),
    gap("jackin_preview/screens/editor.rs::open_workdir_picker", NO_ROW),
    gap("jackin_preview/screens/editor.rs::roles_key", NO_ROW),
    gap("jackin_preview/screens/manager.rs::delete", NO_ROW),
    rows("jackin_preview/screens/manager.rs::inspect", &["jackin/manager/inspect"]),
    rows("jackin_preview/screens/manager.rs::open_launch_picker", &["jackin/manager/launch_picker", "jackin/manager/hard_launch_picker"]),
    gap("jackin_preview/screens/manager.rs::open_menu", NO_ROW),
    gap("jackin_preview/screens/manager.rs::open_session_picker", NO_ROW),
    gap("jackin_preview/screens/manager.rs::purge", NO_ROW),
    gap("jackin_preview/screens/manager.rs::stop", NO_ROW),
    gap("jackin_preview/screens/modals.rs::new", NO_ROW),
    gap("jackin_preview/screens/prelude.rs::open", NO_ROW),
    gap("jackin_preview/screens/settings.rs::finish_save", NO_ROW),
    gap("jackin_preview/screens/settings.rs::leave", NO_ROW),
    rows("jackin_preview/screens/settings.rs::open_preview", &["jackin/settings/save_preview"]),
    // ---------------------------------------------------------- showcase --
    rows("showcase/app.rs::open_help", &["showcase/flows/help/overlay"]),
    rows("showcase/pages/chrome.rs::open_context", &["showcase/flows/chrome/context", "showcase/overlays/chrome_context_row"]),
    rows("showcase/pages/dialogs.rs::open", &["showcase/flows/dialogs/open", "showcase/flows/dialogs/prompt", "showcase/flows/dialogs/destructive"]),
    rows("showcase/pages/grid.rs::on_event", &["showcase/flows/datagrid/preview"]),
    rows("showcase/pages/lists.rs::open_menu", &["showcase/overlays/lists_context"]),
    rows("showcase/pages/pickers.rs::open", &["showcase/flows/pickers/open"]),
    rows("showcase/pages/pickers.rs::render", &["showcase/flows/pickers/open"]),
    rows("showcase/pages/selectable.rs::ask_delete", &["showcase/flows/selectable/delete_confirm"]),
    gap("showcase/pages/selectable.rs::table_input", NO_ROW),
    gap("showcase/pages/settings.rs::handle", NO_ROW),
    gap("showcase/pages/settings.rs::open_add", NO_ROW),
    gap("showcase/pages/settings.rs::open_remove", NO_ROW),
    gap("showcase/pages/taskrunner.rs::handle", NO_ROW),
    gap("showcase/pages/tables.rs::open_menu", NO_ROW),
    gap("showcase/pages/trees.rs::open_menu", NO_ROW),
    // ---------------------------------------------------------- tablepro --
    rows("tablepro/app.rs::apply_requests", &["tablepro/query/close_confirm"]),
    rows("tablepro/app.rs::begin_commit", &["tablepro/workbench/commit_dialog"]),
    rows("tablepro/app.rs::open_context", &["tablepro/overlays/explorer_context", "tablepro/overlays/connections_context"]),
    rows("tablepro/app.rs::open_dialog", &["tablepro/workbench/quit_confirm"]),
    rows("tablepro/app.rs::open_filter_editor", &["tablepro/table/filter_editor"]),
    rows("tablepro/app.rs::open_help", &["tablepro/overlays/help_overlay"]),
    gap("tablepro/app.rs::open_preview", NO_ROW),
    rows("tablepro/app.rs::open_safe_mode_picker", &["tablepro/overlays/safemode_picker"]),
    rows("tablepro/app.rs::open_switcher", &["tablepro/overlays/picker_open"]),
    rows("tablepro/app.rs::open_tab_list", &["tablepro/overlays/tablist_open", "tablepro/overlays/tablist_tables"]),
    rows("tablepro/app.rs::request_quit", &["tablepro/workbench/quit_confirm"]),
    rows("tablepro/app.rs::run_active", &["tablepro/ack/gate"]),
    rows("tablepro/context.rs::finish", &["tablepro/overlays/explorer_context", "tablepro/overlays/connections_context"]),
    rows("tablepro/connections.rs::action", &["tablepro/connections/delete_dialog"]),
];

fn enclosing_fn(line: &str) -> Option<&str> {
    let at = line
        .match_indices("fn ")
        .find(|(i, _)| *i == 0 || !line[..*i].ends_with(|c: char| c.is_alphanumeric() || c == '_'))?
        .0;
    let name: &str = line[at + 3..]
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()?;
    (!name.is_empty()).then_some(name)
}

fn rs_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("list {}: {e}", dir.display())) {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rs_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs")
            && !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.contains("tests"))
        {
            out.push(path);
        }
    }
}

/// `key -> opener count` for every site under `src/bin`.
pub fn scan(bin_dir: &Path) -> BTreeMap<String, usize> {
    let mut files = Vec::new();
    rs_files(bin_dir, &mut files);
    files.sort();
    let mut sites = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(bin_dir)
            .expect("under src/bin")
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        let mut current = String::from("<top>");
        for line in code_only(non_test(&text)).lines() {
            if let Some(name) = enclosing_fn(line) {
                current = name.to_string();
            }
            if NOT_OPENING.iter().any(|p| line.contains(p)) {
                continue;
            }
            let hits = OPENERS.iter().filter(|p| line.contains(*p)).count();
            if hits > 0 {
                *sites.entry(format!("{rel}::{current}")).or_insert(0) += hits;
            }
        }
    }
    sites
}

/// The default-run check.
pub fn overlay_sites() {
    let scanned = scan(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/bin"));
    let listed: BTreeMap<&str, &Site> = SITES.iter().map(|s| (s.key, s)).collect();
    let mut problems = Vec::new();
    if listed.len() != SITES.len() {
        problems.push("duplicate keys in SITES".to_string());
    }
    for key in scanned.keys() {
        if !listed.contains_key(key.as_str()) {
            problems.push(format!(
                "new overlay site `{key}`: add an OVERLAYS row (open + both dismissals) and list it in overlay_sites::SITES"
            ));
        }
    }
    for key in listed.keys() {
        if !scanned.contains_key(*key) {
            problems.push(format!(
                "`{key}` no longer opens an overlay: remove it from SITES"
            ));
        }
    }
    let roots: BTreeSet<&str> = crate::scenarios::tables()
        .into_iter()
        .flat_map(|(_, _, table)| table.iter().map(|r| r.root))
        .collect();
    for site in SITES {
        if site.rows.is_empty() == site.gap.is_empty() {
            problems.push(format!(
                "`{}` needs rows or a gap reason, not both",
                site.key
            ));
        }
        for row in site.rows {
            if !roots.contains(row) {
                problems.push(format!(
                    "`{}` names `{row}`, which is not a scenario row",
                    site.key
                ));
            }
        }
    }
    let covered = SITES.iter().filter(|s| !s.rows.is_empty()).count();
    eprintln!(
        "overlay sites: {covered} of {} have scenario rows; gaps:",
        SITES.len()
    );
    for site in SITES.iter().filter(|s| s.rows.is_empty()) {
        eprintln!("  {}  ({})", site.key, site.gap);
    }
    assert!(
        problems.is_empty(),
        "overlay inventory:\n  {}",
        problems.join("\n  ")
    );
}

pub fn site_scanner_reads_functions() {
    assert_eq!(
        enclosing_fn("    pub fn open_help(&mut self) {"),
        Some("open_help")
    );
    assert_eq!(enclosing_fn("fn a<'b>() {}"), Some("a"));
    assert_eq!(enclosing_fn("let f = Fn::new();"), None);
    assert_eq!(enclosing_fn("    let defn = 1;"), None);
}
