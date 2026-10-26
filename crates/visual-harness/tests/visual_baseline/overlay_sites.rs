#![cfg(test)]
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::determinism::{
    code_only, matching_patterns, non_test, shared_dialog_action_id_projection_ranges,
};

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

const PURE_QUIT_MODEL_PATH: &str = "crates/termrock-shell/src/quit.rs";
const PURE_QUIT_MODEL_FUNCTION: &str = "request_quit";
const PURE_QUIT_MODEL_OPENER: &str = ".request(Leave::";

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
        "crates/jackin-preview-app/src/app.rs::accounts_add_form_dialog",
        &[
            "jackin/accounts/add_form",
            "jackin/accounts/add_form_required",
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
        "crates/jackin-preview-app/src/app/manager_help.rs::draw",
        &["jackin/manager/help_overlay"],
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
        "crates/showcase-demos/src/pages/dialog_kinds.rs::picker",
        &["showcase/overlays/dialogkinds_key_picker"],
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
        "crates/tablepro-ui/src/app.rs::delete_dialog",
        &["tablepro/connections/delete_dialog"],
    ),
    rows(
        "crates/tablepro-ui/src/app.rs::update_delete_dialog",
        &["tablepro/connections/delete_dialog"],
    ),
    rows(
        "crates/tablepro-ui/src/app/query_safety.rs::dialog",
        &["tablepro/ack/gate", "tablepro/ack/delete_gate"],
    ),
    rows(
        "crates/tablepro-ui/src/app/query_safety.rs::update",
        &["tablepro/ack/gate", "tablepro/ack/delete_gate"],
    ),
    rows(
        "crates/tablepro-ui/src/help_keys.rs::draw",
        &["tablepro/overlays/help_overlay"],
    ),
    rows(
        "crates/tablepro-ui/src/help_keys.rs::layer",
        &["tablepro/overlays/help_overlay"],
    ),
    rows(
        "crates/tablepro-ui/src/help_keys.rs::update",
        &["tablepro/overlays/help_overlay"],
    ),
    rows(
        "crates/tablepro-ui/src/quick_switcher.rs::component",
        &["tablepro/overlays/picker_open"],
    ),
    rows(
        "crates/tablepro-ui/src/safe_mode.rs::picker",
        &["tablepro/overlays/safemode_picker"],
    ),
    rows(
        "crates/tablepro-ui/src/tab_list.rs::picker",
        &[
            "tablepro/overlays/tablist_open",
            "tablepro/overlays/tablist_tables",
        ],
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

fn is_pure_quit_model_match(
    path: &str,
    function: &str,
    pattern: &str,
    matched: &std::ops::Range<usize>,
) -> bool {
    path == PURE_QUIT_MODEL_PATH
        && function == PURE_QUIT_MODEL_FUNCTION
        && pattern == PURE_QUIT_MODEL_OPENER
        && matched.start < matched.end
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
        files.extend(crate::determinism::sources(&root));
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
        let non_test = non_test(&text).unwrap_or_else(|error| panic!("scan {rel}: {error}"));
        let code = code_only(&non_test);
        let action_id_projections =
            shared_dialog_action_id_projection_ranges(&code).unwrap_or_default();
        let patterns: Vec<_> = OPENERS.iter().chain(NOT_OPENING.iter()).copied().collect();
        let matches = matching_patterns(&code, &patterns);
        let (opener_matches, not_opening_matches) = matches.split_at(OPENERS.len());
        let mut line_start = 0;
        let mut pure_quit_model_matches = 0;
        for line in code.split_inclusive('\n') {
            let mut source_line = line.strip_suffix('\n').unwrap_or(line);
            source_line = source_line.strip_suffix('\r').unwrap_or(source_line);
            if let Some(name) = enclosing_fn(source_line) {
                current = name.to_string();
            }
            let line_end = line_start + line.len();
            for (pattern_index, pattern_matches) in opener_matches.iter().enumerate() {
                for opener in pattern_matches
                    .iter()
                    .filter(|opener| (line_start..line_end).contains(&opener.start))
                {
                    if is_pure_quit_model_match(&rel, &current, OPENERS[pattern_index], opener) {
                        pure_quit_model_matches += 1;
                    }
                }
            }
            // Keep the established count unit: each opener pattern contributes at most once
            // per physical line, even when that line contains repeated calls.
            let hits = OPENERS
                .iter()
                .enumerate()
                .filter(|(pattern, _)| {
                    opener_matches[*pattern].iter().any(|opener| {
                        (line_start..line_end).contains(&opener.start)
                            && !(OPENERS[*pattern] == "Dialog::new("
                                && action_id_projections
                                    .iter()
                                    .any(|projection| projection.start == opener.start))
                            && !not_opening_matches
                                .iter()
                                .flatten()
                                .any(|excluded| excluded.contains(&opener.start))
                            && !is_pure_quit_model_match(&rel, &current, OPENERS[*pattern], opener)
                    })
                })
                .count();
            if hits > 0 {
                *sites.entry(format!("{rel}::{current}")).or_insert(0) += hits;
            }
            line_start = line_end;
        }
        if rel == PURE_QUIT_MODEL_PATH {
            assert_eq!(
                pure_quit_model_matches, 1,
                "expected one exact {PURE_QUIT_MODEL_OPENER} match in {PURE_QUIT_MODEL_PATH}::{PURE_QUIT_MODEL_FUNCTION}, found {pure_quit_model_matches}"
            );
        }
    }
    sites
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

pub fn reviewed_overlay_registry_changes_are_exact() {
    let expected_mappings: [(&str, &[&str]); 12] = [
        (
            "crates/jackin-preview-app/src/app.rs::accounts_add_form_dialog",
            &[
                "jackin/accounts/add_form",
                "jackin/accounts/add_form_required",
            ],
        ),
        (
            "crates/jackin-preview-app/src/app/manager_help.rs::draw",
            &["jackin/manager/help_overlay"],
        ),
        (
            "crates/showcase-demos/src/pages/dialog_kinds.rs::picker",
            &["showcase/overlays/dialogkinds_key_picker"],
        ),
        (
            "crates/tablepro-ui/src/app.rs::delete_dialog",
            &["tablepro/connections/delete_dialog"],
        ),
        (
            "crates/tablepro-ui/src/app.rs::update_delete_dialog",
            &["tablepro/connections/delete_dialog"],
        ),
        (
            "crates/tablepro-ui/src/app/query_safety.rs::dialog",
            &["tablepro/ack/gate", "tablepro/ack/delete_gate"],
        ),
        (
            "crates/tablepro-ui/src/app/query_safety.rs::update",
            &["tablepro/ack/gate", "tablepro/ack/delete_gate"],
        ),
        (
            "crates/tablepro-ui/src/help_keys.rs::draw",
            &["tablepro/overlays/help_overlay"],
        ),
        (
            "crates/tablepro-ui/src/help_keys.rs::layer",
            &["tablepro/overlays/help_overlay"],
        ),
        (
            "crates/tablepro-ui/src/help_keys.rs::update",
            &["tablepro/overlays/help_overlay"],
        ),
        (
            "crates/tablepro-ui/src/safe_mode.rs::picker",
            &["tablepro/overlays/safemode_picker"],
        ),
        (
            "crates/tablepro-ui/src/tab_list.rs::picker",
            &[
                "tablepro/overlays/tablist_open",
                "tablepro/overlays/tablist_tables",
            ],
        ),
    ];
    for (key, rows) in expected_mappings {
        let site = SITES
            .iter()
            .find(|site| site.key == key)
            .unwrap_or_else(|| panic!("missing reviewed source mapping {key}"));
        assert_eq!(site.rows, rows, "unexpected scenario ownership for {key}");
        assert!(
            site.gap.is_empty(),
            "reviewed source mapping {key} is a gap"
        );
    }

    let expected_gaps = [
        (
            "crates/holla-ui/src/screens/actions.rs::picker",
            "OPEN: parity/custom-actions has no captured open/dismiss sequence for the actions picker",
        ),
        (
            "crates/jackin-preview-app/src/app.rs::draw_layers",
            "OPEN: no capsule-help scenario row; manager/help_overlay exercises a different help content owner",
        ),
        (
            "crates/jackin-preview-app/src/app.rs::editor_exit_dialog",
            "OPEN: editor exit confirmation has no dedicated captured scenario; editor/quit_guard covers application quit guard",
        ),
        (
            "crates/jackin-preview-app/src/app.rs::launch_dialog",
            "OPEN: launch confirmation has no current scenario row",
        ),
        (
            "crates/jackin-preview-app/src/app.rs::open_capsule_help",
            "OPEN: no capsule-help scenario row; manager/help_overlay exercises a different help content owner",
        ),
        (
            "crates/jackin-preview-app/src/app.rs::update_overlays",
            "OPEN: capsule-help update path shares the uncaptured capsule-help owner",
        ),
    ];
    let actual_gaps: Vec<_> = SITES
        .iter()
        .filter(|site| !site.gap.is_empty())
        .map(|site| (site.key, site.gap))
        .collect();
    assert_eq!(
        actual_gaps, expected_gaps,
        "the six reviewed gaps must remain open"
    );

    fn unique_root(id: &str) -> (crate::scenarios::Kind, &'static crate::scenarios::Root) {
        let matches: Vec<_> = crate::scenarios::tables()
            .into_iter()
            .flat_map(|(_, kind, roots)| {
                roots
                    .iter()
                    .filter(move |root| root.root == id)
                    .map(move |root| (kind, root))
            })
            .collect();
        assert_eq!(matches.len(), 1, "expected one scenario root {id}");
        matches[0]
    }

    for (id, steps) in [
        ("jackin/accounts/add_form", &["a", "wait:New account"][..]),
        (
            "jackin/accounts/add_form_required",
            &["a", "wait:New account", "backtab", "enter", "wait:Required"][..],
        ),
    ] {
        let (kind, root) = unique_root(id);
        assert_eq!(
            kind,
            crate::scenarios::Kind::Overlay,
            "{id} must be an overlay"
        );
        assert_eq!(root.bin, crate::scenarios::Bin::Jackin);
        assert_eq!(
            root.args,
            [
                "--scenario",
                "accounts-mixed",
                "--motion",
                "paused",
                "--frame",
                "40"
            ]
        );
        assert_eq!(root.ready, "jackin❯");
        assert_eq!(root.steps, steps);
        assert_eq!(root.status, crate::scenarios::Status::Stored);
        assert_eq!(root.dismiss, Some(crate::scenarios::Status::Pending));
    }

    let (delete_kind, delete_gate) = unique_root("tablepro/ack/delete_gate");
    assert_eq!(delete_kind, crate::scenarios::Kind::Overlay);
    assert_eq!(delete_gate.bin, crate::scenarios::Bin::Tablepro);
    assert_eq!(
        delete_gate.args,
        ["--connect", "Production", "--query-duration-ms", "42"]
    );
    assert_eq!(delete_gate.ready, "Query 1");
    assert_eq!(
        delete_gate.steps,
        [
            "tab",
            "type:DELETE FROM orders",
            "escape",
            "ctrl-r",
            "wait:DELETE without WHERE"
        ]
    );
    assert_eq!(delete_gate.status, crate::scenarios::Status::Stored);
    assert_eq!(delete_gate.dismiss, Some(crate::scenarios::Status::Pending));

    let scenarios = crate::scenarios::all();
    for id in [
        "jackin/accounts/add_form",
        "jackin/accounts/add_form_required",
        "tablepro/ack/delete_gate",
    ] {
        for dismissal in crate::scenarios::Dismiss::ALL {
            assert!(
                scenarios.iter().any(|scenario| {
                    scenario.root.root == id
                        && scenario.dismiss == Some(dismissal)
                        && scenario.pending
                        && scenario.name.starts_with(&format!(
                            "{}{suffix}",
                            id,
                            suffix = dismissal.suffix()
                        ))
                }),
                "missing pending {} case for {id}",
                dismissal.suffix()
            );
        }
    }
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
        concat!(
            "#[cfg(test)]\n",
            "mod hidden { fn test_only() { Dialog::confirm(ID, \"title\", \"body\"); } }\n",
            "#[emit(#[cfg(test)] mod quoted { fn generated() { Dialog::new(ID); } })]\n",
            "fn quoted_marker() {}\n",
            "#[cfg(test)]\n",
            "mod historical_tests;\n",
            "#[cfg(test)] mod shared;\n",
            "#[cfg_attr(all(), path = \"shared.rs\")] mod production_shared;\n",
            "#[cfg(test)] mod conditional;\n",
            "#[cfg_attr(feature = \"production\", path = \"conditional.rs\")] mod production_conditional;\n",
            "#[cfg(test)] mod included;\n",
            "include!(\"included.rs\");\n",
            "#[cfg(test)] mod unresolved;\n",
            "macro_rules! bind_production { () => { mod unresolved; } }\n",
            "bind_production!();\n",
            "mod nested;\n",
            "fn newly_added_owner() { Dialog::confirm(ID, \"title\", \"body\"); ProductDialog::new(intent); }\n",
        ),
    )
    .expect("write overlay source fixture");
    let test_module = source
        .parent()
        .expect("source parent")
        .join("app/historical_tests.rs");
    std::fs::create_dir_all(test_module.parent().expect("test module directory"))
        .expect("create test module directory");
    std::fs::write(
        &test_module,
        "fn test_only() { Dialog::confirm(ID, \"title\", \"body\"); }\n",
    )
    .expect("write out-of-line test source fixture");
    std::fs::write(
        source
            .parent()
            .expect("source parent")
            .join("production_tests_helper.rs"),
        "fn helper_owner() { Dialog::new(ID); }\n",
    )
    .expect("write production helper source fixture");
    let app_source = source.parent().expect("source parent");
    std::fs::create_dir_all(app_source.join("nested")).expect("create nested module directory");
    for (name, body) in [
        (
            "shared.rs",
            "fn shared_open() { Dialog::facts(ID, \"title\", \"body\"); }\n",
        ),
        (
            "conditional.rs",
            "fn conditional_open() { Dialog::prompt(ID, \"title\", \"body\"); }\n",
        ),
        ("included.rs", "fn included_open() { Picker::new(ID); }\n"),
        (
            "unresolved.rs",
            "fn unresolved_open() { ModalTag::new(ID); }\n",
        ),
        (
            "nested/mod.rs",
            "mod child; fn nested_open() { HelpOverlay::new(ID); }\n",
        ),
        (
            "nested/child.rs",
            "fn nested_child_open() { Dialog::destructive(ID, \"title\", \"body\"); }\n",
        ),
    ] {
        std::fs::write(app_source.join(name), body)
            .unwrap_or_else(|error| panic!("write {name}: {error}"));
    }
    let direct_source = workspace.join("crates/tablepro-ui/src");
    std::fs::create_dir_all(&direct_source).expect("create explicit-path source root");
    std::fs::write(
        workspace.join("crates/tablepro-ui/Cargo.toml"),
        "[package]\nname = \"tablepro-ui\"\nversion = \"0.1.0\"\n",
    )
    .expect("write explicit-path package manifest");
    std::fs::write(
        direct_source.join("lib.rs"),
        "#[path = \"direct.rs\"] mod direct;\n",
    )
    .expect("write direct path binding");
    std::fs::write(
        direct_source.join("direct.rs"),
        "fn direct_open() { LayerTag::new(ID); }\n",
    )
    .expect("write direct path source fixture");
    let scanned = scan(&workspace);
    let listed: BTreeMap<&str, &Site> = SITES.iter().map(|site| (site.key, site)).collect();
    std::fs::remove_dir_all(&workspace).expect("remove source fixture");
    let keys = [
        "crates/holla-ui/src/app.rs::generated",
        "crates/holla-ui/src/app.rs::newly_added_owner",
        "crates/holla-ui/src/app/historical_tests.rs::test_only",
        "crates/holla-ui/src/conditional.rs::conditional_open",
        "crates/holla-ui/src/included.rs::included_open",
        "crates/holla-ui/src/nested/child.rs::nested_child_open",
        "crates/holla-ui/src/nested/mod.rs::nested_open",
        "crates/holla-ui/src/production_tests_helper.rs::helper_owner",
        "crates/holla-ui/src/shared.rs::shared_open",
        "crates/holla-ui/src/unresolved.rs::unresolved_open",
        "crates/tablepro-ui/src/direct.rs::direct_open",
    ];
    for key in keys {
        assert_eq!(
            scanned.get(key),
            Some(&1),
            "missing external overlay binding {key}: {scanned:?}"
        );
    }
    assert_eq!(
        unlisted_sites(&scanned, &listed),
        keys.map(str::to_string).to_vec()
    );
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
    assert_eq!(
        matching_patterns("Dialog::confirm(ID)", &["Dialog::confirm("])[0].len(),
        1
    );
    assert!(matching_patterns("ProductDialog::new(ID)", &["Dialog::new("])[0].is_empty());
    site_scanner_detects_unlisted_overlay_and_ignores_prefix_matches();
}

pub fn scanner_matches_calls_across_comments_without_losing_owners() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-overlay-token-matches-{}",
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
        concat!(
            "#[cfg(test)] mod hidden { fn test_only() { Dialog::new(ID); } }\n",
            "#[emit(#[cfg(test)] mod quoted { fn generated() { Dialog::new(ID); } })]\n",
            "fn attribute_marker() {}\n",
            "fn inline_open() { Dialog/**/::new(ID); }\n",
            "fn after_inline() { Dialog::confirm(ID, \"title\", \"body\"); }\n",
            "fn multiline_open() {\n",
            "    Dialog /* first */\n",
            "        :: /* second */ new(ID);\n",
            "}\n",
            "fn after_multiline() { Picker::new(ID); }\n",
            "fn fallback_only() { let _value = value.unwrap_or(ModalTag::new(ID)); }\n",
            "fn fallback_with_real() { let _value = value.unwrap_or(ModalTag/**/::new(ID)); Dialog::new(ID); }\n",
            "fn fallback_multiline() {\n",
            "    let _value = value.unwrap_or(\n",
            "        ModalTag /* fallback */\n",
            "            :: /* path */ new(ID)\n",
            "    );\n",
            "    Dialog\n",
            "        :: new(ID);\n",
            "}\n",
            "fn preview_only() { let p = self.style(Picker::new(ID.sub(\"preview\", \"item\"))); }\n",
            "fn preview_with_real() {\n",
            "    let p = self.style(\n",
            "        Picker /* preview */\n",
            "            :: /* path */ new(ID.sub(\"preview\", \"item\"))\n",
            "    );\n",
            "    Dialog::new(ID);\n",
            "}\n",
            "fn boundaries() { unrelatedreq.menu(ID); ProductDialog/**/::new(ID); Dialog::newer(ID); this.req /* receiver */ . menu(ID); Dialog::new(ID); }\n",
            "fn method_openers() {\n",
            "    self /* receiver */ . menu /* method */ . open(ID);\n",
            "    self\n",
            "        . /* method */ push(Modal::confirm(ID));\n",
            "}\n",
            "fn repeated() { Dialog::new(ID); Dialog::new(ID); }\n",
            "fn repeated_lines() {\n",
            "    Dialog::new(ID);\n",
            "    Dialog::new(ID);\n",
            "}\n",
            "fn lexical_noise() { let _text = \"Dialog::new(ID)\"; }\n",
            "// Dialog::new(ID)\n",
            "fn production_after_test() { Dialog::facts(ID, \"title\", \"body\"); }\n",
        ),
    )
    .expect("write overlay token fixture");

    let scanned = scan(&workspace);
    std::fs::remove_dir_all(&workspace).expect("remove source fixture");
    let expected = BTreeMap::from([
        ("crates/holla-ui/src/app.rs::after_inline".to_string(), 1),
        ("crates/holla-ui/src/app.rs::after_multiline".to_string(), 1),
        ("crates/holla-ui/src/app.rs::generated".to_string(), 1),
        ("crates/holla-ui/src/app.rs::boundaries".to_string(), 2),
        (
            "crates/holla-ui/src/app.rs::fallback_multiline".to_string(),
            1,
        ),
        (
            "crates/holla-ui/src/app.rs::fallback_with_real".to_string(),
            1,
        ),
        ("crates/holla-ui/src/app.rs::inline_open".to_string(), 1),
        ("crates/holla-ui/src/app.rs::method_openers".to_string(), 2),
        ("crates/holla-ui/src/app.rs::multiline_open".to_string(), 1),
        (
            "crates/holla-ui/src/app.rs::preview_with_real".to_string(),
            1,
        ),
        (
            "crates/holla-ui/src/app.rs::production_after_test".to_string(),
            1,
        ),
        ("crates/holla-ui/src/app.rs::repeated".to_string(), 1),
        ("crates/holla-ui/src/app.rs::repeated_lines".to_string(), 2),
    ]);
    assert_eq!(
        scanned, expected,
        "match exact token sequences at their first-token line, preserve one hit per opener type and physical line, and bound preview/fallback exclusions"
    );
    action_id_projection_first_red_is_a_production_root_case();
    action_id_projection_controls_are_conservative();
    root_cfg_test_projection_is_exact_and_byte_stable();
    root_cfg_test_scanner_controls_are_conservative();
}

fn root_cfg_test_projection_is_exact_and_byte_stable() {
    let exact = concat!(
        "/* fake marker #![cfg(test)] */\r\n",
        "#![cfg(test)]\r\n",
        "fn test_only() { Dialog::new(ID); }\r\n",
        "const UTF8: &str = \"é🧪\";\r\n",
    );
    let projection = non_test(exact).expect("project exact root cfg(test) source");
    assert_eq!(projection.len(), exact.len());
    let line_breaks = |source: &str| {
        source
            .bytes()
            .enumerate()
            .filter_map(|(index, byte)| matches!(byte, b'\n' | b'\r').then_some((index, byte)))
            .collect::<Vec<_>>()
    };
    assert_eq!(line_breaks(&projection), line_breaks(exact));
    assert!(
        projection
            .bytes()
            .all(|byte| matches!(byte, b' ' | b'\n' | b'\r'))
    );

    for source in [
        "// #![cfg(test)]\nfn production() { Dialog::new(ID); }\n",
        "const FAKE: &str = r##\"#![cfg(test)]\"##;\nfn production() { Dialog::new(ID); }\n",
        "fn production() { quote! { #![cfg(test)] }; Dialog::new(ID); }\n",
        "#![cfg(any(test, feature = \"production\"))]\nfn production() { Dialog::new(ID); }\n",
        "#![cfg(all(test, feature = \"nightly\"))]\nfn production() { Dialog::new(ID); }\n",
        "#![allow(dead_code)]\n#![cfg(test)]\nfn production() { Dialog::new(ID); }\n",
    ] {
        let projection = non_test(source).expect("preserve non-exact root cfg source");
        assert_eq!(
            projection,
            source.to_owned(),
            "only the exact first-root cfg(test) token sequence proves a test-only file",
        );
    }

    assert!(non_test("#![cfg(test]\nfn malformed() { Dialog::new(ID); }").is_err());
}

fn root_cfg_test_scanner_controls_are_conservative() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-root-cfg-controls-{}",
        std::process::id()
    ));
    let source_root = workspace.join("crates/holla-ui/src");
    std::fs::create_dir_all(&source_root).expect("create source root");
    std::fs::write(
        workspace.join("crates/holla-ui/Cargo.toml"),
        "[package]\nname = \"holla-ui\"\nversion = \"0.1.0\"\n",
    )
    .expect("write package manifest");
    std::fs::write(
        source_root.join("lib.rs"),
        concat!(
            "use termrock::Dialog;\n",
            "#[cfg(test)] mod root_test;\n",
            "#[cfg(test)] mod external_tests;\n",
            "mod shared;\n",
            "fn production_owner() { Dialog::confirm(ID, \"title\", \"body\"); }\n",
        ),
    )
    .expect("write root source");
    std::fs::write(
        source_root.join("root_test.rs"),
        concat!(
            "/* fake marker */\r\n",
            "#![cfg(test)]\r\n",
            "fn test_only() { Dialog::new(ID); }\r\n",
        ),
    )
    .expect("write root-marked test source");
    std::fs::write(
        source_root.join("external_tests.rs"),
        "#[test] fn external_real_owner() { Dialog::confirm(ID, \"title\", \"body\"); }\n",
    )
    .expect("write unmarked external test source");
    std::fs::write(
        source_root.join("production_tests_helper.rs"),
        "fn helper_owner() { Dialog::prompt(ID, \"title\", \"body\"); }\n",
    )
    .expect("write unmarked shared production helper");
    std::fs::write(
        source_root.join("shared.rs"),
        "fn shared_open() { Dialog::facts(ID, \"title\", \"body\"); }\n",
    )
    .expect("write shared production module");
    std::fs::write(
        source_root.join("mixed.rs"),
        concat!(
            "#![cfg(any(test, feature = \"production\"))]\n",
            "fn mixed_owner() { Dialog::new(ID); }\n",
        ),
    )
    .expect("write mixed cfg source");
    std::fs::write(
        source_root.join("unknown_cfg.rs"),
        concat!(
            "#![cfg(all(test, feature = \"nightly\"))]\n",
            "fn unknown_cfg_owner() { Dialog::new(ID); }\n",
        ),
    )
    .expect("write unknown cfg source");
    std::fs::write(
        source_root.join("later_marker.rs"),
        concat!(
            "#![allow(dead_code)]\n",
            "#![cfg(test)]\n",
            "fn later_marker_owner() { Dialog::new(ID); }\n",
        ),
    )
    .expect("write non-first root marker source");
    std::fs::write(
        source_root.join("fake_markers.rs"),
        concat!(
            "fn fake_marker_owner() {\n",
            "    // #![cfg(test)]\n",
            "    let literal = r##\"#![cfg(test)]\"##;\n",
            "    let _ = literal;\n",
            "    quote! { #![cfg(test)] };\n",
            "    Dialog::new(ID);\n",
            "    unknown! { Dialog::confirm(ID, \"title\", \"body\"); }\n",
            "}\n",
        ),
    )
    .expect("write fake marker source");

    let scanned = scan(&workspace);
    std::fs::remove_dir_all(&workspace).expect("remove root cfg test fixture");
    let expected = BTreeMap::from([
        (
            "crates/holla-ui/src/external_tests.rs::external_real_owner".to_string(),
            1,
        ),
        (
            "crates/holla-ui/src/fake_markers.rs::fake_marker_owner".to_string(),
            2,
        ),
        (
            "crates/holla-ui/src/later_marker.rs::later_marker_owner".to_string(),
            1,
        ),
        (
            "crates/holla-ui/src/lib.rs::production_owner".to_string(),
            1,
        ),
        ("crates/holla-ui/src/mixed.rs::mixed_owner".to_string(), 1),
        (
            "crates/holla-ui/src/production_tests_helper.rs::helper_owner".to_string(),
            1,
        ),
        ("crates/holla-ui/src/shared.rs::shared_open".to_string(), 1),
        (
            "crates/holla-ui/src/unknown_cfg.rs::unknown_cfg_owner".to_string(),
            1,
        ),
    ]);
    assert_eq!(
        scanned, expected,
        "mask only exact root test files while preserving unmarked, mixed cfg, and unknown macro sources",
    );
}

fn action_id_projection_first_red_is_a_production_root_case() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-action-id-first-red-{}",
        std::process::id()
    ));
    let source = workspace.join("crates/holla-ui/src/lib.rs");
    let package = source
        .parent()
        .and_then(Path::parent)
        .expect("package root");
    std::fs::create_dir_all(source.parent().expect("source parent"))
        .expect("create production source root");
    std::fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"holla-ui\"\nversion = \"0.1.0\"\n",
    )
    .expect("write package manifest");
    std::fs::write(
        &source,
        concat!(
            "use termrock::{Dialog, Id};\n",
            "const DIALOG: Id = Id::root(\"holla.action\");\n",
            "fn action_id_owners() {\n",
            "    let cancel = Dialog::new(DIALOG).action_id(0);\n",
            "    let review = Dialog::new(DIALOG).action_id(1);\n",
            "    let actions = [\n",
            "        Dialog::new(DIALOG).action_id(0),\n",
            "        Dialog::new(DIALOG).action_id(1),\n",
            "    ];\n",
            "    let _ = (cancel, review, actions);\n",
            "}\n",
        ),
    )
    .expect("write production first-red source");

    let raw = std::fs::read_to_string(&source).expect("read first-red source");
    let production = non_test(&raw).expect("project first-red source");
    let code = code_only(&production);
    let legacy_matches = matching_patterns(&code, &["Dialog::new("]);
    assert_eq!(
        legacy_matches[0].len(),
        4,
        "the old rule reports four production-root openers on four lines"
    );

    let scanned = scan(&workspace);
    std::fs::remove_dir_all(&workspace).expect("remove first-red source root");
    assert_eq!(
        scanned,
        BTreeMap::new(),
        "pure action-ID projections have no overlay owner"
    );
}

fn action_id_projection_controls_are_conservative() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-action-id-controls-{}",
        std::process::id()
    ));
    let source_root = workspace.join("crates/holla-ui/src");
    std::fs::create_dir_all(&source_root).expect("create production source root");
    std::fs::write(
        workspace.join("crates/holla-ui/Cargo.toml"),
        "[package]\nname = \"holla-ui\"\nversion = \"0.1.0\"\n",
    )
    .expect("write package manifest");
    let source = concat!(
        "use super::*;\n",
        "use termrock::{Dialog, Id};\n",
        "const DIALOG: Id = Id::root(\"holla.action\");\n",
        "#[cfg(test)] mod external_tests;\n",
        "fn id_only_owners() {\n",
        "    let cancel = Dialog::new(DIALOG).action_id(0);\n",
        "    let actions = [\n",
        "        Dialog::new(DIALOG).action_id(0),\n",
        "        Dialog::new(DIALOG).action_id(1),\n",
        "    ];\n",
        "    let _ = (cancel, actions);\n",
        "}\n",
        "fn comments_and_multiline_tokens() {\n",
        "    let _owner = Dialog /* gap */\n",
        "        :: /* gap */ new\n",
        "        (DIALOG)\n",
        "        . /* gap */ action_id\n",
        "        (1);\n",
        "}\n",
        "fn same_line_mixture() { let _id = Dialog::new(DIALOG).action_id(1); let _props = Dialog::new(DIALOG).title(\"real\"); }\n",
        "fn props_builder() -> Dialog<'static> { Dialog::new(DIALOG).title(\"title\").actions(ACTIONS).layer(LAYER) }\n",
        "fn returned_props() -> Dialog<'static> { Dialog::new(DIALOG) }\n",
        "fn update_path() { let props = Dialog::new(DIALOG).title(\"title\").update(cx, state); props.draw(cx, area); }\n",
        "fn nested_real_constructor() {\n",
        "    Dialog::new(\n",
        "        Dialog::new(DIALOG).title(\"inner\")\n",
        "    ).action_id(1);\n",
        "}\n",
        "fn action_ids_prefix() { Dialog::new(DIALOG).action_ids(1); }\n",
        "fn unsupported_argument() { Dialog::new(make_id()).action_id(1); }\n",
        "fn braced_const_argument() { Dialog::new(const { DIALOG }).action_id(1); }\n",
        "fn array_argument() { Dialog::new([DIALOG]).action_id(1); }\n",
        "fn macro_argument() { Dialog::new(dialog_id!()).action_id(1); }\n",
        "fn unsupported_extra_chain() { let _id = Dialog::new(DIALOG).action_id(1).title(\"builder\"); }\n",
        "fn unsupported_operator_context() { let _same = Dialog::new(DIALOG).action_id(1) == other; }\n",
        "fn where_clause<T>() where T: Bound { let _id = Dialog::new(DIALOG).action_id(1); }\n",
        "fn custom_local_import() { use custom::Dialog; let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn local_extern_crate_shadow() { extern crate custom as Dialog; let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn wildcard_local_import() { use custom::*; let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn unknown_local_attribute() { #[inject_binding] struct Inject; let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn local_type_shadow() { struct Dialog; let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn generic_type_shadow<Dialog>() { let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn parameter_shadow(Dialog: ()) { let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn unknown_macro_before_projection() { inject_binding!(); let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn macro_body_projection() { macro_rules! project { () => { Dialog::new(DIALOG).action_id(0) }; } project!(); }\n",
        "fn raw_macro_body_projection() { macro_rules! r#project { () => { Dialog::new(DIALOG).action_id(0) }; } }\n",
        "fn fully_qualified_path() { termrock::Dialog::new(DIALOG).action_id(0); }\n",
        "struct Wrapper<T>(T); impl<Dialog> Wrapper<Dialog> { fn impl_generic_shadow() { let _id = Dialog::new(DIALOG).action_id(0); } }\n",
        "mod nested { fn nested_module_owner() { Dialog::new(DIALOG).action_id(0); } }\n",
        "fn standard_macro_before_projection() { assert!(true); let _id = Dialog::new(DIALOG).action_id(0); }\n",
        "fn outer_macro_scope() {\n",
        "    inject_binding!();\n",
        "    fn nested_macro_owner() {\n",
        "        let _owner = Dialog::new(DIALOG).action_id(0);\n",
        "    }\n",
        "}\n",
        "mod macro_parent;\n",
        "#[cfg(test)] mod inherited_derive;\n",
    );
    let lib = source_root.join("lib.rs");
    std::fs::write(&lib, source).expect("write production source fixture");
    std::fs::write(
        source_root.join("production_tests_helper.rs"),
        concat!(
            "use termrock::{Dialog, Id};\n",
            "const DIALOG: Id = Id::root(\"holla.disk.selection\");\n",
            "fn helper_owner() { let _owner = Dialog::new(DIALOG).action_id(1); }\n",
            "fn disk_props() -> Dialog<'static> { Dialog::new(DIALOG).title(\"Select disk candidates\").actions(ACTIONS).body_rows(2) }\n",
        ),
    )
    .expect("write production helper fixture");
    std::fs::write(
        source_root.join("external_tests.rs"),
        concat!(
            "use termrock::{Dialog, Id};\n",
            "const DIALOG: Id = Id::root(\"holla.external\");\n",
            "#[test] fn external_id_only() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
            "#[test] fn external_real_owner() { Dialog::new(DIALOG).title(\"real props\"); Dialog::confirm(DIALOG, \"title\", \"body\"); }\n",
        ),
    )
    .expect("write external cfg(test) source fixture");
    let macro_parent = source_root.join("macro_parent.rs");
    std::fs::create_dir_all(source_root.join("macro_parent"))
        .expect("create out-of-line macro child root");
    std::fs::write(
        &macro_parent,
        concat!(
            "pub mod custom {\n",
            "    pub struct Dialog;\n",
            "    impl Dialog {\n",
            "        pub fn new<T>(_: T) -> Self { Self }\n",
            "        pub fn action_id(self, _: usize) -> usize { 0 }\n",
            "    }\n",
            "}\n",
            "macro_rules! assert { ($name:ident) => { use crate::macro_parent::custom::$name; }; }\n",
            "#[cfg(test)] mod inherited_macro;\n",
        ),
    )
    .expect("write parent macro shadow fixture");
    std::fs::write(
        source_root.join("macro_parent/inherited_macro.rs"),
        concat!(
            "use termrock::{Dialog, Id};\n",
            "const DIALOG: Id = Id::root(\"holla.parent-macro\");\n",
            "fn inherited_parent_macro_without_glob() {\n",
            "    assert!(Dialog);\n",
            "    let _owner = Dialog::new(DIALOG).action_id(0);\n",
            "}\n",
        ),
    )
    .expect("write out-of-line child without super glob");
    std::fs::write(
        source_root.join("inherited_derive.rs"),
        concat!(
            "#[derive(Clone)] struct Marker;\n",
            "use termrock::{Dialog, Id};\n",
            "const DIALOG: Id = Id::root(\"holla.unproven-derive\");\n",
            "fn inherited_derive_scope() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write unproven external derive fixture");
    std::fs::write(
        source_root.join("alias.rs"),
        concat!(
            "use termrock::Dialog as SharedDialog;\n",
            "fn alias_only() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write alias binding fixture");
    std::fs::write(
        source_root.join("ambiguous.rs"),
        concat!(
            "use termrock::Dialog;\n",
            "use custom::Dialog;\n",
            "fn ambiguous_import() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write ambiguous import fixture");
    std::fs::write(
        source_root.join("macro_binding.rs"),
        concat!(
            "use termrock::Dialog;\n",
            "macro_rules! Dialog { () => {}; }\n",
            "fn macro_binding() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write macro binding fixture");
    std::fs::write(
        source_root.join("wildcard_import.rs"),
        concat!(
            "use custom::*;\n",
            "use termrock::Dialog;\n",
            "fn wildcard_import_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write wildcard import fixture");
    std::fs::write(
        source_root.join("extern_shadow.rs"),
        concat!(
            "extern crate Dialog;\n",
            "use termrock::Dialog;\n",
            "fn extern_binding_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write external-crate binding fixture");
    std::fs::write(
        source_root.join("root_type_shadow.rs"),
        concat!(
            "use termrock::Dialog;\n",
            "struct Dialog;\n",
            "fn root_type_shadow_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write root type binding fixture");
    std::fs::write(
        source_root.join("root_attribute.rs"),
        concat!(
            "#[inject_binding] struct Inject;\n",
            "use termrock::Dialog;\n",
            "fn root_attribute_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write root attribute fixture");
    std::fs::write(
        source_root.join("spaced_separator.rs"),
        concat!(
            "use termrock: /* gap */ : Dialog;\n",
            "fn spaced_separator_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write malformed spaced-separator fixture");
    std::fs::write(
        source_root.join("unknown.rs"),
        "use custom::Dialog;\nfn unknown_binding() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
    )
    .expect("write unknown binding fixture");
    std::fs::write(
        source_root.join("crate_shadow.rs"),
        concat!(
            "mod termrock { pub struct Dialog; }\n",
            "use termrock::Dialog;\n",
            "fn local_crate_binding() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write shadowed shared-crate path fixture");
    std::fs::write(
        source_root.join("conditional_import.rs"),
        concat!(
            "#[cfg(feature = \"shared-dialog\")] use termrock::Dialog;\n",
            "fn conditional_import_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write conditional import fixture");
    std::fs::write(
        source_root.join("attribute_macro.rs"),
        concat!(
            "use termrock::Dialog;\n",
            "#[inject_binding] fn attribute_macro_owner() { let _owner = Dialog::new(DIALOG).action_id(0); }\n",
        ),
    )
    .expect("write unknown attribute macro fixture");

    let lib_non_test = non_test(source).expect("tokenize fixture source");
    let lib_code = code_only(&lib_non_test);
    let projections = shared_dialog_action_id_projection_ranges(&lib_code)
        .expect("find proven action-ID projections");
    assert_eq!(
        projections.len(),
        5,
        "only the let/array, comment-separated, and same-line mixed ID projections qualify"
    );
    let projected_lines: Vec<_> = projections
        .iter()
        .map(|projection| {
            lib_code.as_bytes()[..projection.start]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count()
                + 1
        })
        .collect();
    assert_eq!(projected_lines, [6, 8, 9, 14, 20]);
    assert!(
        shared_dialog_action_id_projection_ranges(
            "use termrock::Dialog; fn malformed() { Dialog::new(ID).action_id(0;"
        )
        .is_err()
    );

    let scanned = scan(&workspace);
    std::fs::remove_dir_all(&workspace).expect("remove action-ID source fixture");
    let expected = BTreeMap::from([
        ("crates/holla-ui/src/alias.rs::alias_only".to_string(), 1),
        ("crates/holla-ui/src/ambiguous.rs::ambiguous_import".to_string(), 1),
        ("crates/holla-ui/src/attribute_macro.rs::attribute_macro_owner".to_string(), 1),
        ("crates/holla-ui/src/conditional_import.rs::conditional_import_owner".to_string(), 1),
        ("crates/holla-ui/src/crate_shadow.rs::local_crate_binding".to_string(), 1),
        ("crates/holla-ui/src/external_tests.rs::external_real_owner".to_string(), 2),
        ("crates/holla-ui/src/inherited_derive.rs::inherited_derive_scope".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::action_ids_prefix".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::array_argument".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::braced_const_argument".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::custom_local_import".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::local_extern_crate_shadow".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::wildcard_local_import".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::unknown_local_attribute".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::fully_qualified_path".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::generic_type_shadow".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::impl_generic_shadow".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::local_type_shadow".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::macro_argument".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::macro_body_projection".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::raw_macro_body_projection".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::nested_module_owner".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::nested_real_constructor".to_string(), 2),
        ("crates/holla-ui/src/lib.rs::parameter_shadow".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::props_builder".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::returned_props".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::same_line_mixture".to_string(), 1),
        (
            "crates/holla-ui/src/lib.rs::standard_macro_before_projection".to_string(),
            1,
        ),
        ("crates/holla-ui/src/lib.rs::nested_macro_owner".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::unknown_macro_before_projection".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::update_path".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::unsupported_argument".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::unsupported_extra_chain".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::unsupported_operator_context".to_string(), 1),
        ("crates/holla-ui/src/lib.rs::where_clause".to_string(), 1),
        ("crates/holla-ui/src/production_tests_helper.rs::disk_props".to_string(), 1),
        ("crates/holla-ui/src/unknown.rs::unknown_binding".to_string(), 1),
        ("crates/holla-ui/src/macro_binding.rs::macro_binding".to_string(), 1),
        (
            "crates/holla-ui/src/macro_parent/inherited_macro.rs::inherited_parent_macro_without_glob"
                .to_string(),
            1,
        ),
        ("crates/holla-ui/src/wildcard_import.rs::wildcard_import_owner".to_string(), 1),
        ("crates/holla-ui/src/extern_shadow.rs::extern_binding_owner".to_string(), 1),
        ("crates/holla-ui/src/root_type_shadow.rs::root_type_shadow_owner".to_string(), 1),
        ("crates/holla-ui/src/root_attribute.rs::root_attribute_owner".to_string(), 1),
        ("crates/holla-ui/src/spaced_separator.rs::spaced_separator_owner".to_string(), 1),
    ]);
    assert_eq!(
        scanned, expected,
        "retain all real, shadowed, unsupported, macro, and uncertain candidates; exclude only proven terminal Id projections"
    );
    assert!(!scanned.contains_key("crates/holla-ui/src/production_tests_helper.rs::helper_owner"));
    assert!(!scanned.contains_key("crates/holla-ui/src/external_tests.rs::external_id_only"));
}

pub fn quit_model_exclusion_is_source_and_span_qualified() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-overlay-quit-model-{}",
        std::process::id()
    ));
    let shell_source = workspace.join("crates/termrock-shell/src/quit.rs");
    let shell_manifest = shell_source
        .parent()
        .and_then(Path::parent)
        .expect("termrock-shell package root")
        .join("Cargo.toml");
    std::fs::create_dir_all(shell_source.parent().expect("shell source parent"))
        .expect("create shell source root");
    std::fs::write(
        shell_manifest,
        "[package]\nname = \"termrock-shell\"\nversion = \"0.1.0\"\n",
    )
    .expect("write shell package manifest");
    std::fs::write(
        &shell_source,
        concat!(
            "fn request_quit() {\n",
            "    self.request(Leave::Quit, stakes); Dialog::confirm(ID, \"title\", \"body\");\n",
            "    self.requesting(Leave::Quit, stakes);\n",
            "}\n",
            "fn false_identifier() { self.requesting(Leave::Quit, stakes); }\n",
            "fn another_owner() {\n",
            "    self /* keep the method chain */\n",
            "        . /* comments and newlines remain token-transparent */ request\n",
            "        (Leave :: Quit, stakes);\n",
            "}\n",
        ),
    )
    .expect("write shell source fixture");

    let other_source = workspace.join("crates/holla-ui/src/quit.rs");
    let other_manifest = other_source
        .parent()
        .and_then(Path::parent)
        .expect("holla package root")
        .join("Cargo.toml");
    std::fs::create_dir_all(other_source.parent().expect("holla source parent"))
        .expect("create holla source root");
    std::fs::write(
        other_manifest,
        "[package]\nname = \"holla-ui\"\nversion = \"0.1.0\"\n",
    )
    .expect("write holla package manifest");
    std::fs::write(
        &other_source,
        "fn request_quit() { self.request(Leave::Quit, stakes); }\n",
    )
    .expect("write holla source fixture");

    let scanned = scan(&workspace);
    let listed: BTreeMap<&str, &Site> = SITES.iter().map(|site| (site.key, site)).collect();
    std::fs::remove_dir_all(&workspace).expect("remove source fixture");

    let exact_owner = "crates/termrock-shell/src/quit.rs::request_quit";
    assert_eq!(
        scanned.get(exact_owner),
        Some(&1),
        "the adjacent Dialog::confirm remains detected after excluding only the pure delegate"
    );
    assert!(
        unlisted_sites(&scanned, &listed)
            .iter()
            .any(|key| key == exact_owner),
        "a genuine opener in the qualified function must remain an unlisted site"
    );
    assert_eq!(
        scanned.get("crates/termrock-shell/src/quit.rs::another_owner"),
        Some(&1),
        "the same opener in another owner, including comment/newline-separated tokens, remains detected"
    );
    assert_eq!(
        scanned.get("crates/holla-ui/src/quit.rs::request_quit"),
        Some(&1),
        "the same function name and body in another source path remains detected"
    );
    assert!(
        !scanned.contains_key("crates/termrock-shell/src/quit.rs::false_identifier"),
        "a longer request method name does not match the exact opener"
    );
}

pub fn quit_model_exclusion_rejects_an_extra_qualified_match() {
    let workspace = std::env::temp_dir().join(format!(
        "visual-harness-overlay-quit-model-count-{}",
        std::process::id()
    ));
    let source = workspace.join("crates/termrock-shell/src/quit.rs");
    let manifest = source
        .parent()
        .and_then(Path::parent)
        .expect("termrock-shell package root")
        .join("Cargo.toml");
    std::fs::create_dir_all(source.parent().expect("shell source parent"))
        .expect("create shell source root");
    std::fs::write(
        manifest,
        "[package]\nname = \"termrock-shell\"\nversion = \"0.1.0\"\n",
    )
    .expect("write shell package manifest");
    std::fs::write(
        &source,
        concat!(
            "fn request_quit() {\n",
            "    self.request(Leave::Quit, stakes);\n",
            "    self.request(Leave::Quit, stakes);\n",
            "}\n",
        ),
    )
    .expect("write duplicate delegate fixture");

    let result = std::panic::catch_unwind(|| scan(&workspace));
    std::fs::remove_dir_all(&workspace).expect("remove source fixture");
    assert!(
        result.is_err(),
        "a second exact delegate in the qualified owner must fail the count-one assertion"
    );
}
