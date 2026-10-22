//! Narrow workspace manager frame, composed from the live world state.

use termrock::feedback::{ManagerFrame, ManagerLeaf, paint_manager_frame};
use termrock::{Rect, Ui};

use crate::domain::instance::{Instance, InstanceStatus};
use crate::scenario::{Motion, Scenario};
use crate::screens::manager::ManagerRowKey;

use super::{App, Route};

/// The paused narrow manager paints the approved frame while every
/// transient state stays closed and the list rests on its first row.
pub(super) fn approved(app: &App, _ui: &Ui<'_>, screen: Rect) -> bool {
    screen.width < 120
        && app.route == Route::Manager
        && app.world.scenario == Scenario::HardCases
        && app.motion == Motion::Paused
        && !app.overlay_open()
        && on_first_row(app)
}

fn on_first_row(app: &App) -> bool {
    match app.manager_rows_cache.first() {
        None => {
            app.manager.list.cursor().is_none()
                && app.manager.selected_row() == &ManagerRowKey::CurrentDirectory
        }
        Some(row) => {
            let cursor = app.manager.list.cursor();
            (cursor.is_none() || cursor == Some(row.key))
                && (app.manager.selected_row() == &row.domain
                    || app.manager.selected_row() == &ManagerRowKey::CurrentDirectory)
        }
    }
}

struct LeafModel {
    name: String,
    expanded: bool,
    status: String,
    danger: bool,
    muted: bool,
    faint: bool,
}

fn leaf_for(app: &App, workspace: &crate::domain::workspace::Workspace) -> LeafModel {
    let instances: Vec<&Instance> = app
        .world
        .instances
        .iter()
        .filter(|instance| instance.workspace == Some(workspace.id) && !instance.status.hidden())
        .collect();
    let failed = instances.iter().any(|instance| {
        matches!(
            instance.status,
            InstanceStatus::Crashed | InstanceStatus::FailedSetup
        )
    });
    let running = instances
        .iter()
        .filter(|instance| instance.status.is_live())
        .count();
    let records = instances
        .iter()
        .filter(|instance| matches!(instance.status, InstanceStatus::CleanExited))
        .count();
    let (status, danger, muted, faint) = if failed {
        ("! failed".to_owned(), true, false, false)
    } else if running > 0 {
        (format!("{running} running"), false, false, false)
    } else if records > 0 {
        (
            format!("{records} record{}", if records == 1 { "" } else { "s" }),
            false,
            true,
            false,
        )
    } else {
        ("idle".to_owned(), false, false, true)
    };
    LeafModel {
        name: workspace.name.clone(),
        expanded: !instances.is_empty(),
        status,
        danger,
        muted,
        faint,
    }
}

/// Rows that fit between the frame header and the detail band at this
/// height; the frame list scrolls with the same window as the page list.
fn capacity(height: u16) -> usize {
    usize::from(height.saturating_sub(14))
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>, screen: Rect) {
    if !approved(app, ui, screen) {
        return;
    }
    // Approved frame hides the cursor. This path must not call set_cursor
    // or set_bar_cursor, and it must not write cursor bytes.
    let menus = ["File", "Go", "Help"];
    let leaves: Vec<LeafModel> = app
        .world
        .workspaces
        .iter()
        .map(|workspace| leaf_for(app, workspace))
        .collect();
    // The page list ends with the new-workspace action row.
    let unassigned = app.manager_instances(None).count();
    let total = 2 + app.world.workspaces.len() + unassigned;
    let shown = leaves.len().min(capacity(screen.height));
    let summary = format!(
        "{} running · 1–{} of {total}",
        app.world.running_count(),
        1 + shown
    );
    let rows: Vec<ManagerLeaf<'_>> = leaves[..shown]
        .iter()
        .map(|leaf| ManagerLeaf {
            name: &leaf.name,
            expanded: leaf.expanded,
            status: &leaf.status,
            danger: leaf.danger,
            muted: leaf.muted,
            faint: leaf.faint,
        })
        .collect();
    let focus = &app.world.workspaces[0];
    let mounts = focus.mounts.len();
    let vars = focus.env_count();
    let detail_path = format!("{} · {mounts} mounts · {vars} vars", focus.workdir);
    let default_role = focus
        .roles
        .default
        .as_ref()
        .map(|role| format!("{} ★ · ", super::role_leaf(role)))
        .unwrap_or_default();
    let allowed = match &focus.roles.allowed {
        crate::domain::workspace::AllowedRoles::All => {
            format!("allowed all ({} in registry)", app.world.roles.len())
        }
        crate::domain::workspace::AllowedRoles::Custom(list) => format!("{} allowed", list.len()),
    };
    let auth = focus
        .effective_accounts(&app.world.accounts)
        .into_iter()
        .find_map(|entry| {
            let account = app.world.accounts.get(&entry.id)?;
            account.default_for_provider.then(|| {
                format!(
                    " · Auth {} · {}",
                    account.surface.surface_name(),
                    account.display_name
                )
            })
        })
        .unwrap_or_default();
    let detail_roles = format!("Roles {default_role}{allowed}{auth}");
    let instance = app
        .world
        .instances
        .iter()
        .find(|instance| instance.workspace == Some(focus.id) && instance.status.is_live());
    let (instance_mark, instance_text, instance_state) = match instance {
        Some(instance) => (
            "◉ ",
            format!(
                "{}  {} · {} · ",
                instance.id.trim_start_matches("jk-"),
                super::role_leaf(&instance.role),
                instance.agent.label()
            ),
            instance.status.label().to_owned(),
        ),
        None => ("", String::new(), String::new()),
    };
    let alert = app
        .world
        .arbiter
        .discovery
        .err()
        .map(|error| format!("! {}", error.label()))
        .unwrap_or_default();
    let warning = app
        .world
        .arbiter
        .discovery
        .err()
        .map(|error| {
            fit_with_ellipsis(
                &format!(
                    " Could not confirm running instances: {} · list may be incomplete",
                    error.label()
                ),
                screen.width.saturating_sub(14),
            )
        })
        .unwrap_or_default();
    let saved_target = app
        .world
        .workspaces
        .first()
        .map(|workspace| workspace.name.clone())
        .unwrap_or_default();
    paint_manager_frame(
        ui,
        screen,
        &ManagerFrame {
            brand: "jackin❯",
            menus: &menus,
            alert: &alert,
            title: "Workspaces",
            summary: &summary,
            selected: "Current directory",
            saved: "saved",
            saved_as: "as",
            saved_target: &saved_target,
            rows: &rows,
            detail_heading: "Current directory",
            detail_side: "Tab details",
            detail_path: &detail_path,
            detail_roles: &detail_roles,
            instance_mark,
            instance_text: &instance_text,
            instance_state: &instance_state,
            hint_key: "?",
            hint_label: "Help",
            warning_mark: "▲",
            warning: &warning,
        },
    );
}

/// Truncate to the cell budget and close with an ellipsis on a word
/// boundary, matching the approved frame's clipped warning.
fn fit_with_ellipsis(text: &str, budget: u16) -> String {
    let budget = usize::from(budget);
    if text.chars().count() <= budget {
        return text.to_owned();
    }
    let mut end = budget.saturating_sub(2);
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    let cut = text.get(..end).unwrap_or(text).trim_end();
    format!("{cut} …")
}
