//! Responsive Manager details backed by the live selected row and World.

use termrock::controls::MasterDetail;
use termrock::{
    ActionKey, Binding, Button, Chord, Cx, FrameRead, Id, Intent, KeyCode, MasterDetailCmd,
    MasterDetailLayout, MasterDetailMode, Response, StateFlags, Ui, Variant,
};

use crate::screens::manager::{DETAIL, ManagerRowKey};
use crate::{App, MANAGER, MANAGER_LIST, Route};

const LAYOUT: Id = DETAIL.sub("responsive");
const ACTION: Id = DETAIL.sub("activate-selected");
const OPEN_DETAIL: ActionKey = ActionKey::custom("master_detail.open_detail");
const CLOSE_DETAIL: ActionKey = ActionKey::custom("master_detail.close_detail");
const CLOSE_DETAIL_LEFT: ActionKey = ActionKey::custom("master_detail.close_detail_left");
const CLOSE_DETAIL_BACKTAB: ActionKey = ActionKey::custom("master_detail.close_detail_backtab");
const SHELL_OPEN_DETAIL: ActionKey = ActionKey::custom("manager.detail.open_from_shell");
pub(super) const TREE_PARENT_LEFT: ActionKey =
    ActionKey::custom("manager.tree.parent_or_collapse_left");
pub(super) const TREE_PARENT_H: ActionKey = ActionKey::custom("manager.tree.parent_or_collapse_h");

const SHELL_TAB: &[Binding<MasterDetailCmd>] = &[Binding {
    action: SHELL_OPEN_DETAIL,
    chord: Some(Chord::key(KeyCode::Tab)),
    cmd: MasterDetailCmd::OpenDetail,
    label: "Show details",
    priority: 40,
    visible: true,
}];

const LIST_TAB: &[Binding<MasterDetailCmd>] = &[Binding {
    action: OPEN_DETAIL,
    chord: Some(Chord::key(KeyCode::Tab)),
    cmd: MasterDetailCmd::OpenDetail,
    label: "Show details",
    priority: 40,
    visible: true,
}];

const LIST_TREE_BINDINGS: &[Binding<()>] = &[
    Binding {
        action: TREE_PARENT_LEFT,
        chord: Some(Chord::key(KeyCode::Left)),
        cmd: (),
        label: "Collapse or parent",
        priority: 40,
        visible: false,
    },
    Binding {
        action: TREE_PARENT_H,
        chord: Some(Chord::key(KeyCode::Char('h'))),
        cmd: (),
        label: "Collapse or parent",
        priority: 40,
        visible: false,
    },
];

fn master_detail() -> MasterDetail {
    MasterDetail::new(LAYOUT, MANAGER_LIST, DETAIL)
        .collapse_below(100)
        .summary_rows(6)
        .detail_owns(&[ACTION])
}

pub(super) fn register_shell_tab(app: &App, ui: &mut Ui<'_>) {
    if app.route != Route::Manager || app.manager.detail_open() {
        return;
    }
    let live = ui.state(MANAGER);
    ui.publish_additional_bindings(MANAGER, live, SHELL_TAB);
}

pub(super) fn register_list_tab(ui: &mut Ui<'_>) {
    let live = ui.state(MANAGER_LIST);
    ui.publish_additional_bindings(MANAGER_LIST, live, LIST_TAB);
}

pub(super) fn register_list_tree_bindings(ui: &mut Ui<'_>) {
    let live = ui.state(MANAGER_LIST);
    ui.publish_additional_bindings(MANAGER_LIST, live, LIST_TREE_BINDINGS);
}

pub(super) fn update(app: &mut App, cx: &mut Cx<'_>) -> Response<()> {
    let control = master_detail();
    let master_tab = cx
        .intents(MANAGER_LIST)
        .any(|intent| matches!(intent, Intent::Binding(action) if action == OPEN_DETAIL));
    let shell_tab = cx
        .intents(MANAGER)
        .any(|intent| matches!(intent, Intent::Binding(action) if action == SHELL_OPEN_DETAIL));
    let close_requested = [DETAIL, ACTION].into_iter().any(|owner| {
        cx.intents(owner)
            .any(|intent| matches!(intent, Intent::Binding(action) if is_close_action(action)))
    });

    let mut result = Response::ignored();
    if app.manager_detail_state.drawer_open() || app.manager.detail_open() || master_tab {
        result |= control.update(cx, &mut app.manager_detail_state).erase();
    }
    if close_requested {
        app.manager.set_detail_open(false);
    }
    if shell_tab {
        open_from(app, cx, MANAGER_LIST);
        result |= Response::changed();
    }

    if app.manager_detail_state.drawer_open() || app.manager.detail_open() {
        let enabled = action_enabled(app);
        if enabled
            && cx.state(DETAIL).contains(StateFlags::FOCUSED)
            && !cx.state(ACTION).contains(StateFlags::FOCUSED)
        {
            cx.focus(ACTION);
        } else if !enabled && cx.state(ACTION).contains(StateFlags::FOCUSED) {
            cx.focus(DETAIL);
        }

        let back = Button::new(DETAIL, "‹ Workspaces").update(cx);
        let back_chosen = back.activated();
        result |= back.erase();
        if back_chosen {
            control.close_drawer(&mut app.manager_detail_state, cx);
            app.manager.set_detail_open(false);
            result |= Response::changed();
        } else {
            let action_text = action_label(app);
            let action = Button::new(ACTION, &action_text)
                .variant(Variant::PRIMARY)
                .disabled(!enabled)
                .update(cx);
            let chosen = action.activated();
            result |= action.erase();
            if chosen {
                result |= app.activate_selected_manager_row(cx);
            }
        }
    }

    // Do not mirror a wide master-focus reconciliation into the App's active-detail latch.
    if app.manager_detail_state.drawer_open() {
        app.manager.set_detail_open(true);
    }
    result
}

fn is_close_action(action: ActionKey) -> bool {
    action == CLOSE_DETAIL || action == CLOSE_DETAIL_LEFT || action == CLOSE_DETAIL_BACKTAB
}

pub(super) fn close_off_route_state(app: &mut App) {
    if app.route == Route::Manager {
        return;
    }

    if app.manager_detail_state.drawer_open() || app.manager.detail_open() {
        app.manager_detail_state = termrock::MasterDetailState::default();
        app.manager.set_detail_open(false);
    }
}

pub(super) fn settle_off_route_focus(app: &App, cx: &mut Cx<'_>) {
    if app.route == Route::Manager {
        return;
    }
    settle_focus_out(cx, DETAIL, "‹ Workspaces", false);
    let label = action_label(app);
    settle_focus_out(cx, ACTION, &label, !action_enabled(app));
}

fn settle_focus_out(cx: &mut Cx<'_>, owner: Id, label: &str, disabled: bool) {
    let focus_out_targets: Vec<_> = cx
        .intents(owner)
        .filter_map(|intent| match intent {
            Intent::FocusOut { to: Some(target) } => Some(target),
            _ => None,
        })
        .collect();
    let has_focus_out = cx
        .intents(owner)
        .any(|intent| matches!(intent, Intent::FocusOut { .. }));
    if !has_focus_out {
        return;
    }

    let response = Button::new(owner, label).disabled(disabled).update(cx);
    let _ = response.erase();
    for target in focus_out_targets {
        if cx.focus_is_continued(owner, target) {
            let _ = cx.acknowledge_focus_continuation(owner, target);
        }
    }
}

pub(super) fn open_from(app: &mut App, cx: &mut Cx<'_>, return_focus: Id) {
    master_detail().open_drawer_from(&mut app.manager_detail_state, cx, return_focus);
    cx.focus(if action_enabled(app) { ACTION } else { DETAIL });
    app.manager.set_detail_open(true);
}

pub(super) fn draw(app: &App, ui: &mut Ui<'_>, area: termrock::Rect, frame_owns_list: bool) {
    master_detail().draw(
        ui,
        area,
        &app.manager_detail_state,
        |ui, layout| match layout.mode {
            MasterDetailMode::Split => {
                app.draw_manager_master(ui, layout.master, frame_owns_list);
                draw_detail(app, ui, layout.detail);
            }
            MasterDetailMode::Drawer => draw_detail(app, ui, layout.detail),
            MasterDetailMode::Master => {
                app.draw_manager_master(ui, layout.master, frame_owns_list);
                draw_summary(ui, layout);
            }
        },
    );
}

fn draw_summary(ui: &mut Ui<'_>, layout: MasterDetailLayout) {
    if !layout.summary.is_empty() {
        ui.paint_str(
            layout.summary,
            "Tab opens selected workspace details",
            ui.surface_style(),
        );
    }
}

fn draw_detail(app: &App, ui: &mut Ui<'_>, area: termrock::Rect) {
    if area.is_empty() {
        return;
    }
    let style = ui.surface_style();
    Button::new(DETAIL, "‹ Workspaces").draw(
        ui,
        termrock::Rect::new(area.x, area.y, area.width.min(22), 1),
    );
    let heading = selected_heading(app);
    ui.paint_str(
        termrock::Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
        &heading,
        style,
    );
    for (index, line) in selected_facts(app).iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = area.y.saturating_add(3).saturating_add(offset);
        if y >= area.bottom().saturating_sub(2) {
            break;
        }
        ui.paint_str(termrock::Rect::new(area.x, y, area.width, 1), line, style);
    }
    let action_text = action_label(app);
    Button::new(ACTION, &action_text)
        .variant(Variant::PRIMARY)
        .disabled(!action_enabled(app))
        .draw(
            ui,
            termrock::Rect::new(
                area.x,
                area.bottom().saturating_sub(1),
                area.width.min(32),
                1,
            ),
        );
}

fn selected_heading(app: &App) -> String {
    match app.manager.selected_row() {
        ManagerRowKey::Workspace(id) => app
            .world
            .workspace(*id)
            .map(|workspace| format!("Workspaces › {}", workspace.name))
            .unwrap_or_else(|| format!("Workspace {id} · unavailable")),
        ManagerRowKey::Instance(id) => format!("Instance › {id}"),
        ManagerRowKey::CurrentDirectory => "Current directory".to_owned(),
        ManagerRowKey::NewWorkspace => "New workspace".to_owned(),
    }
}

fn selected_facts(app: &App) -> Vec<String> {
    let mut facts = vec![format!(
        "Selected key · {}",
        app.manager.selected_row().stable_key()
    )];
    match app.manager.selected_row() {
        ManagerRowKey::Workspace(id) => {
            if let Some(workspace) = app.world.workspace(*id) {
                facts.push(format!("Workdir · {}", workspace.workdir));
                facts.push(format!(
                    "{} mounts · {} environment variables",
                    workspace.mounts.len(),
                    workspace.env_count()
                ));
                let count = app.manager_instances(Some(*id)).count();
                facts.push(format!("{} visible instances", count));
            } else {
                facts.push("This workspace is no longer in the current World".to_owned());
            }
        }
        ManagerRowKey::Instance(id) => {
            if let Some(instance) = app.world.instance(id) {
                let workspace = instance
                    .workspace
                    .and_then(|workspace| app.world.workspace(workspace))
                    .map_or("Current directory", |workspace| workspace.name.as_str());
                facts.push(format!("Workspace · {workspace}"));
                facts.push(format!(
                    "Agent · {} · role {}",
                    instance.agent.label(),
                    instance.role
                ));
                facts.push(format!(
                    "Status · {} · {}",
                    instance.status.label(),
                    instance.dirty_summary()
                ));
                facts.push(format!("Workdir · {}", instance.workdir));
            } else {
                facts.push("This instance is no longer in the current World".to_owned());
            }
        }
        ManagerRowKey::CurrentDirectory => {
            facts.push(format!("Path · {}", app.world.cwd));
            facts.push(format!("Home · {}", app.world.home));
        }
        ManagerRowKey::NewWorkspace => {
            facts.push("Create a workspace from the current directory".to_owned());
        }
    }
    facts
}

fn reconnectable_selected_id(app: &App) -> Option<String> {
    let id = app.selected_instance_id()?;
    app.world
        .instance(&id)
        .filter(|instance| instance.status.reconnectable())
        .map(|_| id)
}

fn action_enabled(app: &App) -> bool {
    reconnectable_selected_id(app).is_some()
        || matches!(
            app.manager.selected_row(),
            ManagerRowKey::Workspace(_)
                | ManagerRowKey::CurrentDirectory
                | ManagerRowKey::NewWorkspace
        )
}

fn action_label(app: &App) -> String {
    if let Some(id) = reconnectable_selected_id(app) {
        format!("Reconnect {id}")
    } else if action_enabled(app) {
        "Choose an agent".to_owned()
    } else {
        "Instance unavailable".to_owned()
    }
}
