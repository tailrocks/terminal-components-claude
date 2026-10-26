use termrock::feedback::{
    ManagerFrame, ManagerFrameLayout, ManagerLeaf, paint_manager_frame_with_layout,
};
use termrock::{FrameRead, ItemKey, Rect, Ui};

use crate::domain::instance::{Instance, InstanceStatus};
use crate::scenario::{Motion, Scenario};
use crate::screens::manager::ManagerRowKey;

use super::{App, Route};

struct PresentedRow {
    key: ItemKey,
    area: Rect,
    leaf: LeafModel,
}

pub(super) struct Presentation {
    layout: ManagerFrameLayout,
    rows: Vec<PresentedRow>,
}

pub(super) fn prepare(app: &App, ui: &Ui<'_>, screen: Rect) -> Option<Presentation> {
    if !(screen.width < 120
        && app.route == Route::Manager
        && app.world.scenario == Scenario::HardCases
        && app.motion == Motion::Paused
        && !app.overlay_open())
    {
        return None;
    }

    let shown = app
        .world
        .workspaces
        .len()
        .min(ManagerFrameLayout::row_capacity(screen.height));
    if shown == 0 {
        return None;
    }
    let layout = ManagerFrameLayout::new(screen, shown);
    let mut rows = Vec::with_capacity(shown);
    for (index, workspace) in app.world.workspaces.iter().take(shown).enumerate() {
        let domain = ManagerRowKey::Workspace(workspace.id);
        let row = app
            .manager_rows_cache
            .iter()
            .find(|row| row.domain == domain)?;
        let area = layout.row_rect(index)?;
        rows.push(PresentedRow {
            key: row.key,
            area,
            leaf: leaf_for(app, workspace),
        });
    }

    let frame_owner_was_presented = ui.area(crate::MANAGER_LIST) == Some(layout.rows_area());
    let pressed_row = frame_owner_was_presented
        && ui
            .state(crate::MANAGER_LIST)
            .contains(termrock::StateFlags::PRESSED)
        && ui.pressed_part(crate::MANAGER_LIST).is_some_and(|part| {
            part.part == termrock::Part::ROW
                && part
                    .item
                    .is_some_and(|key| rows.iter().any(|row| row.key == key))
        });
    let first_row_at_top = on_first_row(app) && app.manager.list.scroll().offset() == 0;
    (first_row_at_top || pressed_row).then_some(Presentation { layout, rows })
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

pub(super) fn register(app: &App, ui: &mut Ui<'_>, presentation: &Presentation) {
    let area = presentation.layout.rows_area();
    if ui.is_inert() || area.is_empty() {
        return;
    }

    let list = App::manager_list();
    let live = ui.state(crate::MANAGER_LIST);
    ui.register_control(crate::MANAGER_LIST, area, termrock::Focusability::Focusable);
    ui.publish_bindings(crate::MANAGER_LIST, live, list.table());
    let selection_table = list.selection_table();
    if !presentation.rows.is_empty()
        && !selection_table.is_empty()
        && !live.contains(termrock::StateFlags::DISABLED)
    {
        ui.publish_additional_bindings(crate::MANAGER_LIST, live, selection_table);
    }
    let viewport = usize::from(area.height);
    let mut scroll = *app.manager.list.scroll();

    scroll.apply_layout(viewport, app.manager_rows_cache.len().max(viewport));
    ui.register_scroll(
        crate::MANAGER_LIST,
        area,
        termrock::runtime::Axes::V,
        scroll.headroom_v(),
    );
    ui.report_layout(
        crate::MANAGER_LIST,
        termrock::LayoutFacts::new(
            viewport,
            app.manager_rows_cache.len(),
            area.height,
            area.width,
        )
        .with_vertical_window(termrock::author::VerticalWindow::full(area)),
    );
}

pub(super) fn register_rows(ui: &mut Ui<'_>, presentation: &Presentation) {
    if ui.is_inert() {
        return;
    }
    for row in &presentation.rows {
        ui.register_part(
            crate::MANAGER_LIST,
            termrock::PartRef::item(termrock::Part::ROW, row.key),
            row.area,
        );
    }
}

pub(super) fn paint(app: &App, ui: &mut Ui<'_>, presentation: &Presentation) {
    let menus = ["File", "Go", "Help"];

    let unassigned = app.manager_instances(None).count();
    let total = 2 + app.world.workspaces.len() + unassigned;
    let shown = presentation.rows.len();
    let summary = format!(
        "{} running · 1–{} of {total}",
        app.world.running_count(),
        1 + shown
    );
    let rows: Vec<ManagerLeaf<'_>> = presentation
        .rows
        .iter()
        .map(|row| ManagerLeaf {
            name: &row.leaf.name,
            expanded: row.leaf.expanded,
            status: &row.leaf.status,
            danger: row.leaf.danger,
            muted: row.leaf.muted,
            faint: row.leaf.faint,
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
                presentation.layout.area().width.saturating_sub(14),
            )
        })
        .unwrap_or_default();
    let saved_target = app
        .world
        .workspaces
        .first()
        .map(|workspace| workspace.name.clone())
        .unwrap_or_default();
    paint_manager_frame_with_layout(
        ui,
        presentation.layout,
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
