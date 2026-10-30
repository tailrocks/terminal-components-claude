use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::determinism::{code_only, non_test};

const OPENERS: [&str; 16] = [
    "req.menu(",
    "ModalTag::new(",
    "LayerTag::new(",
    "Dialog::new(",
    "Dialog::confirm(",
    "Dialog::destructive(",
    "Dialog::facts(",
    "Dialog::prompt(",
    "Picker::new(",
    ".push(Modal::",
    "begin_modal(",
    ".menu.open(",
    ".request_quit(&",
    ".request(Leave::",
    "HelpOverlay::new(",
    "HelpOverlay::from_keymaps(",
];

const NOT_OPENING: [&str; 2] = [
    "unwrap_or(ModalTag::new(",
    "let p = self.style(Picker::new(ID.sub(",
];

pub struct Site {
    pub key: &'static str,
    pub rows: &'static [&'static str],
    pub gap: &'static str,
}

const fn rows(key: &'static str, rows: &'static [&'static str]) -> Site {
    Site { key, rows, gap: "" }
}

#[allow(dead_code)]
const fn gap(key: &'static str, gap: &'static str) -> Site {
    Site {
        key,
        rows: &[],
        gap,
    }
}

#[rustfmt::skip]
pub const HISTORICAL_SITES: &[Site] = &[

    rows("holla/app.rs::confirm_plan", &["holla/flows/upgrade/confirm"]),
    rows("holla/app.rs::draw", &["holla/flows/help/overlay", "holla/flows/quit/confirm"]),
    rows("holla/app.rs::menu_action", &["holla/overlays/alias_prompt"]),
    gap(
        "holla/app.rs::navigate",
        "the leave guard's question (S14): no holla page holds unsaved edits yet, so no scenario reaches it; app_tests_nav drives it with a test page",
    ),
    gap(
        "holla/app.rs::request_quit",
        "the leave guard's unsaved-edits question on quit (S14; running work keeps open_quit_confirm): no holla page holds unsaved edits yet; app_tests_nav drives it with a test page",
    ),
    rows("holla/app.rs::open_about", &["holla/overlays/about"]),
    rows("holla/app.rs::open_activities", &["holla/flows/activities/overlay", "holla/flows/activities/empty", "holla/flows/git/batch_picker"]),
    rows("holla/app.rs::open_alternatives", &["holla/flows/alternatives"]),
    rows("holla/app.rs::open_confirm_one", &["holla/flows/cargo/clean_confirm"]),
    rows("holla/app.rs::open_gate2", &["holla/flows/remote/gate2"]),
    rows("holla/app.rs::open_help", &["holla/flows/help/overlay"]),
    rows("holla/app.rs::open_quit_confirm", &["holla/flows/quit/confirm", "holla/flows/quit/confirm_ctrl_c", "holla/flows/quit/confirm_menu"]),
    rows("holla/app.rs::open_why", &["holla/overlays/why"]),
    rows("holla/app.rs::request_close_tab", &["holla/overlays/close_tab"]),
    rows("holla/screens/activity.rs::chip_menu", &["holla/overlays/activity_chip_menu"]),
    rows("holla/screens/activity.rs::output_menu", &["holla/overlays/activity_output_menu"]),
    rows("holla/screens/cleanup.rs::row_menu", &["holla/overlays/cleanup_row_menu"]),
    rows("holla/screens/disk.rs::row_menu", &["holla/overlays/disk_row_menu"]),
    rows("holla/screens/files.rs::open_actions_at", &["holla/overlays/files_actions"]),
    rows("holla/screens/files.rs::open_jump", &["holla/flows/files/jump"]),
    rows("holla/screens/plan.rs::on_interrupt", &["holla/overlays/plan_cancel"]),
    rows("holla/screens/plan.rs::step_menu", &["holla/overlays/plan_step_menu"]),
    rows("holla/screens/review.rs::sequence_menu", &["holla/overlays/review_sequence_menu"]),
    rows("holla/screens/snapshot.rs::body_menu", &["holla/overlays/snapshot_body_menu"]),

    rows("jackin_preview/app.rs::open_about", &["jackin/overlays/about"]),
    rows("jackin_preview/app.rs::open_help", &["jackin/manager/help_overlay"]),
    rows("jackin_preview/screens/accounts.rs::on_key", &["jackin/overlays/accounts_filter"]),
    rows("jackin_preview/screens/accounts.rs::on_layer_result", &["jackin/overlays/accounts_op_picker"]),
    rows("jackin_preview/screens/accounts.rs::open_form", &["jackin/accounts/add_form", "jackin/accounts/add_form_required"]),
    rows("jackin_preview/screens/accounts.rs::open_menu", &["jackin/overlays/accounts_row_menu"]),
    rows("jackin_preview/screens/accounts.rs::remove", &["jackin/overlays/accounts_remove"]),
    rows("jackin_preview/screens/capsule.rs::confirm_close", &["jackin/overlays/capsule_close"]),
    rows("jackin_preview/screens/capsule.rs::continue_spawn", &["jackin/overlays/capsule_provider"]),
    rows("jackin_preview/screens/capsule.rs::on_layer_result", &["jackin/overlays/capsule_discard"]),
    rows("jackin_preview/screens/capsule.rs::open_about", &["jackin/overlays/capsule_about"]),
    rows("jackin_preview/screens/capsule.rs::open_container_info", &["jackin/overlays/capsule_info"]),
    rows("jackin_preview/screens/capsule.rs::open_github", &["jackin/overlays/capsule_github"]),
    rows("jackin_preview/screens/capsule.rs::open_inspect", &["jackin/overlays/capsule_inspect"]),
    rows("jackin_preview/screens/capsule.rs::open_palette", &["jackin/capsule/palette"]),
    rows("jackin_preview/screens/capsule.rs::open_rename", &["jackin/overlays/capsule_rename"]),
    rows("jackin_preview/screens/capsule.rs::open_usage", &["jackin/overlays/capsule_usage"]),
    rows("jackin_preview/screens/capsule.rs::request_exit", &["jackin/overlays/capsule_quit", "jackin/overlays/capsule_discard"]),
    rows("jackin_preview/screens/capsule.rs::run_palette", &["jackin/overlays/capsule_split_picker"]),
    rows("jackin_preview/screens/capsule.rs::spawn_flow", &["jackin/capsule/new_tab"]),
    rows("jackin_preview/screens/cockpit.rs::ask_cancel", &["jackin/cockpit/cancel_confirm"]),
    rows("jackin_preview/screens/cockpit.rs::open_failure", &["jackin/overlays/cockpit_failure"]),
    rows("jackin_preview/screens/cockpit.rs::open_info", &["jackin/cockpit/info"]),
    rows("jackin_preview/screens/cockpit.rs::open_quit", &["jackin/overlays/cockpit_quit"]),
    rows("jackin_preview/screens/cockpit.rs::step", &["jackin/overlays/cockpit_credentials"]),
    rows("jackin_preview/screens/config.rs::on_key", &["jackin/overlays/config_env_remove", "jackin/overlays/config_env_op"]),
    rows("jackin_preview/screens/config.rs::on_modal", &["jackin/overlays/config_mount_browse"]),
    rows("jackin_preview/screens/config.rs::open_env_form", &["jackin/overlays/config_env_form"]),
    rows("jackin_preview/screens/config.rs::open_mount_form", &["jackin/overlays/config_mount_form"]),
    rows("jackin_preview/screens/config.rs::open_role_picker", &["jackin/overlays/config_role_picker"]),
    rows("jackin_preview/screens/config.rs::open_scope_picker", &["jackin/overlays/config_scope_picker"]),
    rows("jackin_preview/screens/editor.rs::accounts_key", &["jackin/overlays/editor_accounts_filter"]),
    rows("jackin_preview/screens/editor.rs::finish_role_load", &["jackin/overlays/editor_role_load_error"]),
    rows("jackin_preview/screens/editor.rs::finish_save", &["jackin/overlays/editor_save_failed"]),
    rows("jackin_preview/screens/editor.rs::leave", &["jackin/editor/quit_guard"]),
    rows("jackin_preview/screens/editor.rs::open_preview", &["jackin/editor/save_preview"]),
    rows("jackin_preview/screens/editor.rs::open_role_picker", &["jackin/overlays/editor_load_role"]),
    rows("jackin_preview/screens/editor.rs::open_trust_dialog", &["jackin/overlays/editor_trust"]),
    rows("jackin_preview/screens/editor.rs::open_workdir_picker", &["jackin/overlays/editor_workdir"]),
    rows("jackin_preview/screens/editor.rs::roles_key", &["jackin/overlays/editor_roles_filter"]),
    rows("jackin_preview/screens/manager.rs::delete", &["jackin/overlays/manager_delete"]),
    rows("jackin_preview/screens/manager.rs::inspect", &["jackin/manager/inspect"]),
    rows("jackin_preview/screens/manager.rs::open_launch_picker", &["jackin/manager/launch_picker", "jackin/manager/hard_launch_picker"]),
    rows("jackin_preview/screens/manager.rs::open_menu", &["jackin/overlays/manager_row_menu"]),
    rows("jackin_preview/screens/manager.rs::open_session_picker", &["jackin/overlays/manager_session_picker"]),
    rows("jackin_preview/screens/manager.rs::purge", &["jackin/overlays/manager_purge"]),
    rows("jackin_preview/screens/manager.rs::stop", &["jackin/overlays/manager_stop"]),
    rows("jackin_preview/screens/modals.rs::new", &["jackin/overlays/accounts_op_picker", "jackin/overlays/config_env_op"]),
    rows("jackin_preview/screens/prelude.rs::open", &["jackin/manager/new_workspace_prelude"]),
    rows("jackin_preview/screens/settings.rs::finish_save", &["jackin/overlays/settings_save_failed"]),
    rows("jackin_preview/screens/settings.rs::leave", &["jackin/settings/quit_guard"]),
    rows("jackin_preview/screens/settings.rs::open_preview", &["jackin/settings/save_preview"]),

    rows("showcase/app.rs::open_dialog", &["showcase/flows/dialogs/open"]),
    rows("showcase/app.rs::open_help", &["showcase/flows/help/overlay"]),
    rows("showcase/app.rs::open_quit", &["showcase/overlays/quit_confirm"]),
    rows("showcase/app.rs::show_help", &["showcase/flows/help/overlay"]),
    rows("showcase/app.rs::request_quit", &["showcase/overlays/settings_quit_guard"]),
    rows("showcase/pages/chrome.rs::open_context", &["showcase/flows/chrome/context", "showcase/overlays/chrome_context_row"]),
    rows("showcase/pages/dialogs.rs::variant", &["showcase/flows/dialogs/open", "showcase/flows/dialogs/prompt", "showcase/flows/dialogs/destructive", "showcase/overlays/dialogs_viewer", "showcase/overlays/dialogs_gate", "showcase/overlays/dialogs_push", "showcase/overlays/dialogs_notes", "showcase/overlays/dialogs_anchored"]),
    rows("showcase/pages/dialogs.rs::gate2", &["showcase/overlays/dialogs_gate2"]),
    rows("showcase/pages/dialogs.rs::states_grid", &["showcase/sections/dialogs-states"]),
    rows("showcase/pages/launch_cockpit.rs::advance", &["showcase/overlays/launchcockpit_credentials", "showcase/overlays/launchcockpit_build_failed"]),
    rows("showcase/pages/launch_cockpit.rs::ask_cancel", &["showcase/overlays/launchcockpit_cancel"]),
    rows("showcase/pages/disk_cleanup.rs::gate1", &["showcase/overlays/diskcleanup_gate1"]),
    rows("showcase/pages/disk_cleanup.rs::gate2", &["showcase/overlays/diskcleanup_gate2"]),
    rows("showcase/pages/workspaces.rs::purge_dialog", &["showcase/overlays/workspaces_purge"]),
    rows("showcase/pages/grid.rs::changes_dialog", &["showcase/flows/datagrid/preview", "showcase/flows/datagrid/save-review"]),
    rows("showcase/pages/empty_error.rs::open_menu", &["showcase/overlays/emptyerror_row_actions"]),
    rows("showcase/pages/help_keys.rs::build_help", &["showcase/overlays/helpkeys_overlay", "showcase/overlays/helpkeys_overlay_small"]),
    rows("showcase/pages/lists.rs::open_menu", &["showcase/overlays/lists_context"]),
    rows("showcase/pages/quit_leave.rs::handle", &["showcase/overlays/quitleave_unsaved", "showcase/overlays/quitleave_running"]),
    rows("showcase/pages/mod.rs::open_dialog", &["showcase/flows/dialogs/open"]),
    rows("showcase/pages/mod.rs::open_help", &["showcase/flows/help/overlay"]),
    rows("showcase/pages/menus.rs::open_row_menu", &["showcase/overlays/menus_row_keyboard", "showcase/overlays/menus_row_pointer"]),
    rows("showcase/pages/pickers.rs::open", &["showcase/flows/pickers/open", "showcase/overlays/pickers_commands", "showcase/overlays/pickers_remote"]),
    rows("showcase/pages/pickers.rs::states_grid", &["showcase/sections/pickers-states"]),
    rows("showcase/pages/connections.rs::ask_delete", &["showcase/overlays/connections_delete"]),
    rows("showcase/pages/connections.rs::form", &["showcase/overlays/connections_form"]),
    rows("showcase/pages/connections.rs::open_form", &["showcase/overlays/connections_form"]),
    rows("showcase/pages/dialog_kinds.rs::open", &["showcase/overlays/dialogkinds_form", "showcase/overlays/dialogkinds_info", "showcase/overlays/dialogkinds_choice", "showcase/overlays/dialogkinds_help"]),
    rows("showcase/pages/dialog_kinds.rs::help", &["showcase/overlays/dialogkinds_help"]),
    rows("showcase/pages/dialog_kinds.rs::form", &["showcase/overlays/dialogkinds_form"]),
    rows("showcase/pages/dialog_kinds.rs::info", &["showcase/overlays/dialogkinds_info"]),
    rows("showcase/pages/dialog_kinds.rs::choice", &["showcase/overlays/dialogkinds_choice"]),
    rows("showcase/pages/dialog_kinds.rs::states_grid", &["showcase/sections/dialogkinds-states"]),
    rows("showcase/pages/dialog_kinds.rs::open_key_picker", &["showcase/overlays/dialogkinds_key_picker"]),
    rows("showcase/pages/finder.rs::open_alternatives", &["showcase/overlays/finder_alternatives"]),
    rows("showcase/pages/finder.rs::open_why", &["showcase/overlays/finder_why"]),
    rows("showcase/pages/file_browser.rs::open", &["showcase/overlays/filebrowser", "showcase/overlays/filebrowser_url"]),
    rows("showcase/pages/selectable.rs::ask_delete", &["showcase/flows/selectable/delete_confirm"]),
    rows("showcase/pages/selectable.rs::open_menu", &["showcase/overlays/selectable_context"]),
    rows("showcase/pages/selectable.rs::table_input", &["showcase/overlays/selectable_discard"]),
    rows("showcase/pages/settings.rs::save", &["showcase/overlays/settings_save_keyboard", "showcase/overlays/settings_save_pointer"]),
    rows("showcase/pages/settings.rs::open_add", &["showcase/overlays/settings_add_variable"]),
    rows("showcase/pages/settings.rs::open_remove", &["showcase/overlays/settings_remove_member"]),
    rows("showcase/pages/taskrunner.rs::handle", &["showcase/overlays/taskrunner_cancel"]),
    rows("showcase/pages/tables.rs::open_menu", &["showcase/overlays/tables_context"]),
    rows("showcase/pages/trees.rs::open_menu", &["showcase/overlays/trees_context"]),

    rows("tablepro/app.rs::act", &["tablepro/query/close_confirm"]),
    rows("tablepro/app.rs::begin_commit", &["tablepro/workbench/commit_dialog"]),
    rows("tablepro/app.rs::open_context", &["tablepro/overlays/explorer_context", "tablepro/overlays/connections_context"]),
    rows("tablepro/app.rs::open_dialog", &["tablepro/workbench/quit_confirm"]),
    rows("tablepro/app.rs::open_filter_editor", &["tablepro/table/filter_editor"]),
    rows("tablepro/app.rs::open_help", &["tablepro/overlays/help_overlay"]),
    rows("tablepro/app.rs::open_preview", &["tablepro/overlays/sql_preview"]),
    rows("tablepro/app.rs::open_safe_mode_picker", &["tablepro/overlays/safemode_picker"]),
    rows("tablepro/app.rs::open_switcher", &["tablepro/overlays/picker_open"]),
    rows("tablepro/app.rs::open_tab_list", &["tablepro/overlays/tablist_open", "tablepro/overlays/tablist_tables"]),
    rows("tablepro/app.rs::request_quit", &["tablepro/workbench/quit_confirm", "tablepro/connections/form_quit_guard"]),
    rows("tablepro/app.rs::run_active", &["tablepro/ack/gate"]),
    rows("tablepro/context.rs::finish", &["tablepro/overlays/explorer_context", "tablepro/overlays/connections_context"]),
    rows("tablepro/connections.rs::action", &["tablepro/connections/delete_dialog"]),
    rows("tablepro/connections.rs::leave_form", &["tablepro/connections/form_leave_esc", "tablepro/connections/form_leave_edit", "tablepro/connections/form_leave_new"]),
];

pub const SITES: &[Site] = &[
    rows(
        "crates/holla-ui/src/app.rs::gate_dialog",
        &["holla/flows/upgrade/confirm"],
    ),
    gap(
        "crates/holla-ui/src/screens/actions.rs::picker",
        "OPEN: parity/custom-actions has no captured open/dismiss sequence for the actions picker",
    ),
    rows(
        "crates/holla-ui/src/screens/clone.rs::picker",
        &["holla/flows/args/clone"],
    ),
    rows(
        "crates/holla-ui/src/screens/dialogs.rs::props",
        &["holla/flows/upgrade/confirm", "holla/overlays/alias_prompt"],
    ),
    rows(
        "crates/jackin-preview-app/src/app.rs::account_picker",
        &[
            "jackin/overlays/accounts_op_picker",
            "jackin/overlays/config_env_op",
        ],
    ),
    rows(
        "crates/jackin-preview-app/src/app.rs::capsule_command_palette",
        &[
            "jackin/capsule/palette",
            "jackin/overlays/capsule_split_picker",
        ],
    ),
    rows(
        "crates/jackin-preview-app/src/app.rs::cockpit_cancel_dialog",
        &["jackin/cockpit/cancel_confirm"],
    ),
    gap(
        "crates/jackin-preview-app/src/app.rs::draw_layers",
        "OPEN: no capsule-help scenario row; manager/help_overlay exercises a different help content owner",
    ),
    gap(
        "crates/jackin-preview-app/src/app.rs::editor_exit_dialog",
        "OPEN: editor exit confirmation has no dedicated captured scenario; editor/quit_guard covers application quit guard",
    ),
    rows(
        "crates/jackin-preview-app/src/app.rs::launch_agent_picker",
        &[
            "jackin/manager/launch_picker",
            "jackin/manager/hard_launch_picker",
        ],
    ),
    gap(
        "crates/jackin-preview-app/src/app.rs::launch_dialog",
        "OPEN: launch confirmation has no current scenario row",
    ),
    gap(
        "crates/jackin-preview-app/src/app.rs::open_capsule_help",
        "OPEN: no capsule-help scenario row; manager/help_overlay exercises a different help content owner",
    ),
    rows(
        "crates/jackin-preview-app/src/app.rs::quit_dialog",
        &["jackin/overlays/capsule_quit"],
    ),
    rows(
        "crates/jackin-preview-app/src/app.rs::role_picker",
        &["jackin/overlays/config_role_picker"],
    ),
    gap(
        "crates/jackin-preview-app/src/app.rs::update_overlays",
        "OPEN: capsule-help update path shares the uncaptured capsule-help owner",
    ),
    rows(
        "crates/showcase-demos/src/pages/dialogs.rs::confirm",
        &[
            "showcase/flows/dialogs/open",
            "showcase/flows/dialogs/destructive",
        ],
    ),
    rows(
        "crates/showcase-demos/src/pages/dialogs.rs::prompt",
        &["showcase/flows/dialogs/prompt"],
    ),
    rows(
        "crates/showcase-demos/src/pages/overview.rs::update",
        &["showcase/flows/help/overlay"],
    ),
    rows(
        "crates/showcase-demos/src/pages/pickers.rs::picker",
        &["showcase/flows/pickers/open"],
    ),
    rows(
        "crates/showcase-demos/src/pages/selectable.rs::with_dialog",
        &["showcase/flows/selectable/delete_confirm"],
    ),
    rows(
        "crates/showcase-demos/src/pages/settings.rs::remove_dialog",
        &["showcase/overlays/settings_remove_member"],
    ),
    rows(
        "crates/showcase-demos/src/pages/taskrunner.rs::cancel_dialog",
        &["showcase/overlays/taskrunner_cancel"],
    ),
    rows(
        "crates/tablepro-ui/src/app.rs::dialog",
        &["tablepro/workbench/quit_confirm"],
    ),
    rows(
        "crates/tablepro-ui/src/quick_switcher.rs::component",
        &["tablepro/overlays/picker_open"],
    ),
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

pub fn scan(workspace: &Path) -> BTreeMap<String, usize> {
    let roots = crate::determinism::source_roots(workspace);
    assert!(
        !roots.is_empty(),
        "no application or termrock source roots found under {}",
        workspace.display()
    );
    let mut files = Vec::new();
    for root in roots {
        rs_files(&root, &mut files);
    }
    files.sort();
    let mut sites = BTreeMap::new();
    for file in files {
        let rel = file
            .strip_prefix(workspace)
            .expect("under workspace")
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
            let hits = OPENERS.iter().filter(|p| contains_call(line, p)).count();
            if hits > 0 {
                *sites.entry(format!("{rel}::{current}")).or_insert(0) += hits;
            }
        }
    }
    sites
}

fn contains_call(line: &str, pattern: &str) -> bool {
    line.match_indices(pattern).any(|(at, _)| {
        pattern.starts_with('.')
            || at == 0
            || !line[..at]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
    })
}

fn unlisted_sites(
    scanned: &BTreeMap<String, usize>,
    listed: &BTreeMap<&str, &Site>,
) -> Vec<String> {
    scanned
        .keys()
        .filter(|key| !listed.contains_key(key.as_str()))
        .cloned()
        .collect()
}

pub fn overlay_sites() {
    let workspace = crate::support::manifest_path("");
    let scanned = scan(&workspace);
    let listed: BTreeMap<&str, &Site> = SITES.iter().map(|s| (s.key, s)).collect();
    let mut problems = Vec::new();
    if listed.len() != SITES.len() {
        problems.push("duplicate keys in SITES".to_string());
    }
    for key in unlisted_sites(&scanned, &listed) {
        problems.push(format!(
            "new overlay site `{key}`: add an OVERLAYS row (open + both dismissals) and list it in overlay_sites::SITES"
        ));
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
    for site in SITES.iter().chain(HISTORICAL_SITES) {
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
    let gaps = SITES.len() - covered;
    eprintln!(
        "overlay sites: {covered} of {} have scenario rows; {gaps} gaps{}",
        SITES.len(),
        if gaps == 0 { "" } else { ":" }
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

pub fn site_scanner_detects_unlisted_overlay_and_ignores_prefix_matches() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-overlay-sites-{}",
        std::process::id()
    ));
    let source = workspace.join("crates/holla-ui/src/app.rs");
    let package = source
        .parent()
        .and_then(Path::parent)
        .expect("package root");
    std::fs::create_dir_all(source.parent().expect("source parent")).expect("create source root");
    std::fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"holla-ui\"\nversion = \"0.1.0\"\n",
    )
    .expect("write package manifest");
    std::fs::write(
        &source,
        "fn newly_added_owner() { Dialog::confirm(ID, \"title\", \"body\"); ProductDialog::new(intent); }\n",
    )
    .expect("write overlay source fixture");
    let scanned = scan(&workspace);
    let listed: BTreeMap<&str, &Site> = SITES.iter().map(|site| (site.key, site)).collect();
    std::fs::remove_dir_all(&workspace).expect("remove source fixture");
    let key = "crates/holla-ui/src/app.rs::newly_added_owner";
    assert_eq!(scanned.get(key), Some(&1));
    assert_eq!(unlisted_sites(&scanned, &listed), vec![key.to_string()]);
    assert!(!scanned.keys().any(|key| key.contains("ProductDialog")));
}

pub fn site_scanner_reads_functions() {
    assert_eq!(
        enclosing_fn("    pub fn open_help(&mut self) {"),
        Some("open_help")
    );
    assert_eq!(enclosing_fn("fn a<'b>() {}"), Some("a"));
    assert_eq!(enclosing_fn("let f = Fn::new();"), None);
    assert_eq!(enclosing_fn("    let defn = 1;"), None);
    assert!(contains_call("Dialog::confirm(ID)", "Dialog::confirm("));
    assert!(!contains_call("ProductDialog::new(ID)", "Dialog::new("));
    site_scanner_detects_unlisted_overlay_and_ignores_prefix_matches();
}
