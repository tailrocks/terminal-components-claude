//! Jackin Preview application shell.
//!
//! This module owns only interaction state and paints through `tui-next`'s
//! public facade.  Domain and simulation state stay in sibling modules.

use std::time::Duration;

use tui_next::{
    ActionKey, App as TuiApp, AsItem, Button, Chord, Cx, Dialog, DialogAction, DialogState, Id,
    Item, ItemKey, KeyCode, KeyMap, KeyPhase, List, ListAction, ListState, Panel, Picker,
    PickerAction, PickerState, Rect, Response, Tabs, TabsState, Ui, UpdateCause, Variant,
};

use crate::domain::account::Account;
use crate::domain::agent::Agent;
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::scenario::{Motion, Scenario};
use crate::sim::launch::{LaunchEvent, LaunchPlan, LaunchRun, Stage};
use crate::sim::world::{World, world_for};

pub const APP: Id = Id::root("jackin.preview");
pub const ENTER: Id = APP.sub("enter");
pub const MANAGER: Id = APP.sub("manager");
pub const ACCOUNTS: Id = APP.sub("accounts");
pub const USAGE: Id = APP.sub("usage");
pub const SETTINGS: Id = APP.sub("settings");
pub const CAPSULE: Id = APP.sub("capsule");
pub const MANAGER_LIST: Id = APP.sub("manager-list");
pub const ACCOUNTS_LIST: Id = APP.sub("accounts-list");
pub const LAUNCH: Id = APP.sub("launch");
pub const ACCOUNT_ADD: Id = APP.sub("account-add");
pub const SETTINGS_TRUST: Id = APP.sub("settings-trust");
pub const CAPSULE_TABS: Id = APP.sub("capsule-tabs");
pub const CAPSULE_PANES: Id = APP.sub("capsule-panes");
pub const LAUNCH_DIALOG: Id = APP.sub("launch-dialog");
pub const ROLE_CHOOSE: Id = LAUNCH_DIALOG.sub("role");
pub const ROLE_PICKER: Id = APP.sub("role-picker");
pub const ACCOUNT_PICKER: Id = APP.sub("account-picker");
pub const LAUNCH_CANCEL: Id = APP.sub("launch-cancel");
pub const LAUNCH_RETRY: Id = APP.sub("launch-retry");

const CMD_QUIT: ActionKey = ActionKey::custom("jackin.quit");
const CMD_MANAGER: ActionKey = ActionKey::custom("jackin.manager");
const CMD_ACCOUNTS: ActionKey = ActionKey::custom("jackin.accounts");
const CMD_USAGE: ActionKey = ActionKey::custom("jackin.usage");
const CMD_SETTINGS: ActionKey = ActionKey::custom("jackin.settings");
const CMD_CAPSULE: ActionKey = ActionKey::custom("jackin.capsule");
const TICK_MS: u64 = 33;

/// The visible product route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Intro,
    Manager,
    Accounts,
    Usage,
    Settings,
    Launch,
    Capsule,
}

impl Route {
    const fn title(self) -> &'static str {
        match self {
            Self::Intro => "Welcome to Jackin",
            Self::Manager => "Workspaces & instances",
            Self::Accounts => "Account & Usage Center",
            Self::Usage => "Usage overview",
            Self::Settings => "Settings",
            Self::Launch => "Launch cockpit",
            Self::Capsule => "Capsule",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoleOption {
    key: String,
    label: String,
    detail: String,
}

impl AsItem for RoleOption {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::text(&self.key), &self.label).detail(&self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AccountOption {
    key: String,
    label: String,
    detail: String,
}

impl AsItem for AccountOption {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::text(&self.key), &self.label).detail(&self.detail)
    }
}

/// The Jackin Preview application.
#[derive(Debug, Clone)]
pub struct App {
    /// Deterministic services and durable state exposed for focused tests.
    pub world: World,
    route: Route,
    motion: Motion,
    quit: bool,
    keymap: KeyMap,
    manager_state: ListState,
    accounts_state: ListState,
    tabs_state: TabsState,
    launch_dialog: DialogState,
    role_state: PickerState,
    account_state: PickerState,
    roles: Vec<RoleOption>,
    account_options: Vec<AccountOption>,
    selected_role: usize,
    launch: Option<LaunchRun>,
    status: Option<String>,
    trusted: bool,
}

impl App {
    /// Build one deterministic app scenario.
    pub fn for_scenario(scenario: Scenario, motion: Motion) -> Self {
        let world = world_for(scenario);
        let roles = world
            .roles
            .iter()
            .map(|role| RoleOption {
                key: role.full_name(),
                label: role.full_name(),
                detail: format!(
                    "{} · {}",
                    if role.trusted { "trusted" } else { "untrusted" },
                    role.description
                ),
            })
            .collect::<Vec<_>>();
        let account_options = world
            .accounts
            .sorted()
            .into_iter()
            .map(|account| AccountOption {
                key: account.id.clone(),
                label: account.title(),
                detail: format!(
                    "{} · {}",
                    account.status_word(),
                    account.source.safe_detail()
                ),
            })
            .collect::<Vec<_>>();
        let selected_role = roles
            .iter()
            .position(|role| role.key == "chainargos/the-architect")
            .unwrap_or(0);
        let route = match scenario {
            Scenario::FirstUse => Route::Intro,
            Scenario::AccountsMixed => Route::Accounts,
            Scenario::LaunchRunning | Scenario::LaunchFailure => Route::Launch,
            Scenario::CapsuleMulti | Scenario::OutroLast => Route::Capsule,
            Scenario::Returning | Scenario::HardCases => Route::Manager,
        };
        let launch = matches!(route, Route::Launch).then(|| {
            LaunchRun::new(
                if scenario == Scenario::LaunchFailure {
                    LaunchPlan::FailNetwork
                } else {
                    LaunchPlan::Clean
                },
                Agent::ClaudeCode,
                "jackin-payments-platform",
                crate::RunId::new(0x9c41_e2f0),
            )
        });
        Self {
            world,
            route,
            motion,
            quit: false,
            keymap: app_keymap(),
            manager_state: ListState::default(),
            accounts_state: ListState::default(),
            tabs_state: TabsState::default(),
            launch_dialog: DialogState::default(),
            role_state: PickerState::default(),
            account_state: PickerState::default(),
            roles,
            account_options,
            selected_role,
            launch,
            status: None,
            trusted: false,
        }
    }

    /// The current route.
    pub const fn route(&self) -> Route {
        self.route
    }

    /// The configured motion mode.
    pub const fn motion(&self) -> Motion {
        self.motion
    }

    /// The current selected role label.
    pub fn selected_role(&self) -> &str {
        self.roles
            .get(self.selected_role)
            .map_or("chainargos/the-architect", |role| role.key.as_str())
    }

    /// The active launch run, if the app is in the cockpit.
    pub const fn launch(&self) -> Option<&LaunchRun> {
        self.launch.as_ref()
    }

    fn manager_rows(&self) -> Vec<String> {
        if self.world.instances.is_empty() {
            return Vec::new();
        }
        self.world
            .instances
            .iter()
            .filter(|instance| !instance.status.hidden())
            .map(|instance| {
                format!(
                    "{} · {} · run {} · {}",
                    instance.id,
                    instance.status.label(),
                    instance.run_id.short(),
                    instance.dirty_summary()
                )
            })
            .collect()
    }

    fn account_rows(&self) -> Vec<String> {
        self.world
            .accounts
            .sorted()
            .into_iter()
            .map(|account| {
                format!(
                    "{} · {} · {}",
                    account.title(),
                    account.status_word(),
                    account.source.safe_detail()
                )
            })
            .collect()
    }

    fn launch_dialog(&self) -> Dialog<'static> {
        Dialog::confirm(
            LAUNCH_DIALOG,
            "Launch a new session",
            "Review the role and start a deterministic Construct run.",
        )
        .body_rows(1)
    }

    fn open_launch_dialog(&mut self, cx: &mut Cx<'_>) {
        let dialog = self.launch_dialog();
        let spec = dialog.layer(cx);
        self.launch_dialog = DialogState::default();
        cx.open_layer(LAUNCH_DIALOG, spec);
    }

    fn open_role_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Picker::new(ROLE_PICKER).title("Choose a role");
        let spec = picker.layer(cx, &self.roles);
        cx.open_layer(ROLE_PICKER, spec);
    }

    fn open_account_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Picker::new(ACCOUNT_PICKER).title("Choose a configured account");
        let spec = picker.layer(cx, &self.account_options);
        cx.open_layer(ACCOUNT_PICKER, spec);
    }

    fn begin_launch(&mut self) {
        let plan = if self.world.scenario == Scenario::LaunchFailure {
            LaunchPlan::FailNetwork
        } else {
            LaunchPlan::Clean
        };
        self.launch = Some(LaunchRun::new(
            plan,
            Agent::ClaudeCode,
            "jackin-payments-platform",
            crate::RunId::new(0x9c41_e2f0),
        ));
        self.route = Route::Launch;
        self.status = Some(format!(
            "Queued {} · {}",
            self.selected_role(),
            plan_label(plan)
        ));
    }

    fn update_overlays(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.is_open(ROLE_PICKER) {
            let picker = Picker::new(ROLE_PICKER).title("Choose a role");
            let response = picker.update(cx, &mut self.role_state, &self.roles);
            let action = response.action_ref().copied();
            let mut result = response.erase();
            if let Some(PickerAction::Chosen(key)) = action
                && let Some(index) = self
                    .roles
                    .iter()
                    .position(|role| ItemKey::text(&role.key) == key)
            {
                self.selected_role = index;
                cx.close_layer(ROLE_PICKER, Some(ActionKey::CONFIRM));
                result |= Response::changed();
            }
            return result;
        }
        if cx.is_open(ACCOUNT_PICKER) {
            let picker = Picker::new(ACCOUNT_PICKER).title("Choose a configured account");
            let response = picker.update(cx, &mut self.account_state, &self.account_options);
            let action = response.action_ref().copied();
            let mut result = response.erase();
            if let Some(PickerAction::Chosen(key)) = action
                && let Some(account) = self
                    .account_options
                    .iter()
                    .find(|account| ItemKey::text(&account.key) == key)
            {
                self.status = Some(format!("Selected reference · {}", account.detail));
                cx.close_layer(ACCOUNT_PICKER, Some(ActionKey::CONFIRM));
                result |= Response::changed();
            }
            return result;
        }
        if cx.is_open(LAUNCH_DIALOG) {
            let dialog = self.launch_dialog();
            let response = dialog.update(cx, &mut self.launch_dialog);
            let action = response.action_ref().copied();
            let mut result = response.erase();
            let role_label = self.selected_role().to_owned();
            let role = Button::new(ROLE_CHOOSE, &role_label).update(cx);
            let role_chosen = role.activated();
            result |= role.erase();
            if role_chosen {
                self.open_role_picker(cx);
            }
            if let Some(action) = action {
                match action {
                    DialogAction::Action(ActionKey::CONFIRM) => {
                        cx.close_layer(LAUNCH_DIALOG, Some(ActionKey::CONFIRM));
                        self.begin_launch();
                    }
                    DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_) => {
                        if cx.is_open(LAUNCH_DIALOG) {
                            cx.close_layer(LAUNCH_DIALOG, Some(ActionKey::CANCEL));
                        }
                    }
                    DialogAction::Action(_) => {}
                }
                result |= Response::changed();
            }
            return result;
        }
        Response::ignored()
    }

    fn update_navigation(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let nav = [
            (MANAGER, "Manager", Route::Manager),
            (ACCOUNTS, "Accounts", Route::Accounts),
            (USAGE, "Usage", Route::Usage),
            (SETTINGS, "Settings", Route::Settings),
            (CAPSULE, "Capsule", Route::Capsule),
        ];
        for (id, label, route) in nav {
            let button = Button::new(id, label)
                .checked(self.route == route)
                .update(cx);
            let chosen = button.activated();
            result |= button.erase();
            if chosen {
                self.route = route;
                self.status = None;
            }
        }
        result
    }

    fn update_intro(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let button = Button::new(ENTER, "Enter Construct")
            .variant(Variant::PRIMARY)
            .update(cx);
        let chosen = button.activated();
        let result = button.erase();
        if chosen {
            self.route = Route::Manager;
            self.world.arbiter.complete_entry(self.world.now_ms());
        }
        result
    }

    fn update_manager(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let rows = self.manager_rows();
        let list = List::new(MANAGER_LIST).update(cx, &mut self.manager_state, &rows);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        if matches!(
            list_action,
            Some(ListAction::Activated(_) | ListAction::Chose(_))
        ) {
            if self.world.running_count() == 1 {
                self.route = Route::Capsule;
            }
            result |= Response::changed();
        }
        let button = Button::new(LAUNCH, "Launch session")
            .variant(Variant::PRIMARY)
            .disabled(self.world.workspaces.is_empty())
            .update(cx);
        let chosen = button.activated();
        result |= button.erase();
        if chosen {
            self.open_launch_dialog(cx);
        }
        result
    }

    fn update_accounts(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let rows = self.account_rows();
        let list = List::new(ACCOUNTS_LIST).update(cx, &mut self.accounts_state, &rows);
        let mut result = list.erase();
        let add = Button::new(ACCOUNT_ADD, "Choose 1Password reference…")
            .variant(Variant::PRIMARY)
            .update(cx);
        let chosen = add.activated();
        result |= add.erase();
        if chosen {
            self.open_account_picker(cx);
        }
        result
    }

    fn update_settings(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let button = Button::new(SETTINGS_TRUST, "Trust local incident role")
            .checked(self.trusted)
            .update(cx);
        let chosen = button.activated();
        let result = button.erase();
        if chosen {
            self.trusted = !self.trusted;
        }
        result
    }

    fn update_launch(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let failed = self
            .launch
            .as_ref()
            .is_some_and(|launch| launch.failure.is_some());
        let cancel = Button::new(LAUNCH_CANCEL, "Cancel").update(cx);
        let cancel_chosen = cancel.activated();
        result |= cancel.erase();
        if cancel_chosen {
            self.launch = None;
            self.route = Route::Manager;
            return result;
        }
        if failed {
            let retry = Button::new(LAUNCH_RETRY, "Retry")
                .variant(Variant::PRIMARY)
                .update(cx);
            let retry_chosen = retry.activated();
            result |= retry.erase();
            if retry_chosen {
                self.begin_launch();
                return result;
            }
        }
        if cx.update_cause() == UpdateCause::Tick && self.motion != Motion::Paused {
            if let Some(launch) = &mut self.launch {
                let events = launch.advance();
                if !events.is_empty() {
                    self.handle_launch_events(events);
                    result |= Response::changed();
                }
            }
        }
        if self
            .launch
            .as_ref()
            .is_some_and(|launch| !launch.is_terminal())
        {
            cx.request_repaint_after(Duration::from_millis(TICK_MS));
        }
        result
    }

    fn handle_launch_events(&mut self, events: Vec<LaunchEvent>) {
        for event in events {
            match event {
                LaunchEvent::Activity(activity) => self.status = Some(activity),
                LaunchEvent::BuildLine(_) => {
                    if let Some(launch) = &self.launch {
                        self.status = Some(format!(
                            "Building derived image · {} log lines · run {}",
                            launch.build_lines_emitted,
                            launch.run_id.short()
                        ));
                    }
                }
                LaunchEvent::ContainerReady(container) => {
                    self.status = Some(format!("Container ready · {container}"));
                }
                LaunchEvent::CredentialsResolved { .. } => {
                    self.status = Some("Credentials resolved in memory and discarded".into());
                }
                LaunchEvent::CredentialError { message } => self.status = Some(message),
                LaunchEvent::Failed(failure) => {
                    self.status = Some(format!("{} · {}", failure.stage.label(), failure.summary));
                }
                LaunchEvent::Ready => {
                    self.status = Some("Capsule ready".into());
                    self.route = Route::Capsule;
                }
                LaunchEvent::StageChanged(stage, state) => {
                    self.status = Some(format!("{} · {}", stage.label(), state.label()));
                }
            }
        }
    }

    fn update_capsule(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tabs = capsule_tabs();
        Tabs::new(CAPSULE_TABS)
            .update(cx, &mut self.tabs_state, &tabs)
            .erase()
    }

    fn update_route(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        match self.route {
            Route::Intro => self.update_intro(cx),
            Route::Manager => self.update_manager(cx),
            Route::Accounts => self.update_accounts(cx),
            Route::Usage | Route::Capsule => {
                if self.route == Route::Capsule {
                    self.update_capsule(cx)
                } else {
                    Response::ignored()
                }
            }
            Route::Settings => self.update_settings(cx),
            Route::Launch => self.update_launch(cx),
        }
    }

    fn update_command(&mut self, cx: &mut Cx<'_>, command: ActionKey) -> Option<Response<()>> {
        match command {
            CMD_QUIT => {
                self.quit = true;
                cx.quit();
                Some(Response::changed())
            }
            CMD_MANAGER => {
                self.route = Route::Manager;
                Some(Response::changed())
            }
            CMD_ACCOUNTS => {
                self.route = Route::Accounts;
                Some(Response::changed())
            }
            CMD_USAGE => {
                self.route = Route::Usage;
                Some(Response::changed())
            }
            CMD_SETTINGS => {
                self.route = Route::Settings;
                Some(Response::changed())
            }
            CMD_CAPSULE => {
                self.route = Route::Capsule;
                Some(Response::changed())
            }
            _ => None,
        }
    }

    fn draw_header(&self, ui: &mut Ui<'_>, area: Rect) {
        let heading = Rect {
            height: area.height.min(1),
            ..area
        };
        let style = ui.surface_style();
        ui.paint_str(heading, self.route.title(), style);
        if area.height < 2 {
            return;
        }
        let y = area.y.saturating_add(1);
        let labels = [
            (MANAGER, "Manager"),
            (ACCOUNTS, "Accounts"),
            (USAGE, "Usage"),
            (SETTINGS, "Settings"),
            (CAPSULE, "Capsule"),
        ];
        let mut x = area.x;
        for (id, label) in labels {
            let width = 12;
            Button::new(id, label)
                .checked(match id {
                    MANAGER => self.route == Route::Manager,
                    ACCOUNTS => self.route == Route::Accounts,
                    USAGE => self.route == Route::Usage,
                    SETTINGS => self.route == Route::Settings,
                    CAPSULE => self.route == Route::Capsule,
                    _ => false,
                })
                .draw(ui, Rect::new(x, y, width, 1));
            x = x.saturating_add(width);
        }
    }

    fn draw_intro(&self, ui: &mut Ui<'_>, area: Rect) {
        let style = ui.surface_style();
        ui.paint_str(
            Rect {
                height: area.height.min(1),
                ..area
            },
            "No running instances found. The first launch owns the Construct entry ritual.",
            style,
        );
        Button::new(ENTER, "Enter Construct")
            .variant(Variant::PRIMARY)
            .draw(
                ui,
                Rect {
                    y: area.y.saturating_add(2),
                    width: area.width.min(24),
                    height: 1,
                    ..area
                },
            );
    }

    fn draw_manager(&self, ui: &mut Ui<'_>, area: Rect) {
        let rows = self.manager_rows();
        let list_area = Rect {
            height: area.height.saturating_sub(3),
            ..area
        };
        List::new(MANAGER_LIST).draw(ui, list_area, &self.manager_state, &rows);
        Button::new(LAUNCH, "Launch session")
            .variant(Variant::PRIMARY)
            .disabled(self.world.workspaces.is_empty())
            .draw(
                ui,
                Rect {
                    y: area.bottom().saturating_sub(1),
                    width: area.width.min(22),
                    height: 1,
                    ..area
                },
            );
    }

    fn draw_accounts(&self, ui: &mut Ui<'_>, area: Rect) {
        let rows = self.account_rows();
        let list_area = Rect {
            height: area.height.saturating_sub(3),
            ..area
        };
        List::new(ACCOUNTS_LIST).draw(ui, list_area, &self.accounts_state, &rows);
        Button::new(ACCOUNT_ADD, "Choose 1Password reference…")
            .variant(Variant::PRIMARY)
            .draw(
                ui,
                Rect {
                    y: area.bottom().saturating_sub(1),
                    width: area.width.min(34),
                    height: 1,
                    ..area
                },
            );
    }

    fn draw_usage(&self, ui: &mut Ui<'_>, area: Rect) {
        let summary = crate::domain::usage::OverallSummary::compute(&self.world.accounts.accounts);
        let lines = [
            format!("Health · {}", summary.health.label()),
            format!(
                "Accounts · {} total · {} enabled · {} disabled",
                summary.counts.accounts, summary.counts.enabled, summary.counts.disabled
            ),
            format!(
                "Providers · {} · warnings {} · exhausted {}",
                summary.counts.providers, summary.counts.warnings, summary.counts.exhausted
            ),
            format!(
                "Freshness · stale {} · failed {} · unresolved identities {}",
                summary.counts.stale, summary.counts.failed, summary.counts.unresolved_identity
            ),
        ];
        paint_lines(ui, area, &lines);
    }

    fn draw_settings(&self, ui: &mut Ui<'_>, area: Rect) {
        let lines = [
            "Runtime mode · Sync host credentials",
            "Workspace · payments-platform",
            "DCO signoff · enabled",
            "Secret policy · references only; resolved bytes are transient",
        ];
        paint_lines(ui, area, &lines);
        Button::new(SETTINGS_TRUST, "Trust local incident role")
            .checked(self.trusted)
            .draw(
                ui,
                Rect {
                    y: area.bottom().saturating_sub(1),
                    width: area.width.min(30),
                    height: 1,
                    ..area
                },
            );
    }

    fn draw_launch(&self, ui: &mut Ui<'_>, area: Rect) {
        let Some(launch) = &self.launch else {
            paint_lines(ui, area, &["No launch run is active."]);
            return;
        };
        let header = format!(
            "{} · run {} · role {}",
            launch.agent.label(),
            launch.run_id.short(),
            self.selected_role()
        );
        let style = ui.surface_style();
        ui.paint_str(
            Rect {
                height: area.height.min(1),
                ..area
            },
            &header,
            style,
        );
        let mut y = area.y.saturating_add(1);
        for (index, stage) in Stage::ALL.iter().enumerate() {
            if y >= area.bottom().saturating_sub(2) {
                break;
            }
            let state = launch.states.get(index).copied().unwrap_or_default();
            let line = format!(
                "{:>2}. {:<16} {}",
                index.saturating_add(1),
                stage.label(),
                state.label()
            );
            ui.paint_str(Rect::new(area.x, y, area.width, 1), &line, style);
            y = y.saturating_add(1);
        }
        if let Some(status) = &self.status {
            ui.paint_str(
                Rect {
                    y: area.bottom().saturating_sub(2),
                    height: 1,
                    ..area
                },
                status,
                style,
            );
        }
        if launch.failure.is_some() {
            Button::new(LAUNCH_RETRY, "Retry")
                .variant(Variant::PRIMARY)
                .draw(
                    ui,
                    Rect::new(area.x, area.bottom().saturating_sub(1), 12, 1),
                );
        } else {
            Button::new(LAUNCH_CANCEL, "Cancel").draw(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 12, 1),
            );
        }
    }

    fn draw_capsule(&self, ui: &mut Ui<'_>, area: Rect) {
        let tabs = capsule_tabs();
        Tabs::new(CAPSULE_TABS).draw(ui, area, &self.tabs_state, &tabs);
        let pane_area = Rect {
            y: area.y.saturating_add(3),
            height: area.height.saturating_sub(3),
            ..area
        };
        let rows = self
            .world
            .instances
            .iter()
            .find(|instance| instance.status == InstanceStatus::Running)
            .and_then(|instance| match &instance.daemon {
                DaemonSnapshot::Tabs(tabs) => Some(
                    tabs.iter()
                        .flat_map(|tab| tab.panes.iter())
                        .map(|pane| {
                            format!(
                                "{} · {} · {}",
                                pane.label,
                                pane.state.label(),
                                if pane.focused {
                                    "focused"
                                } else {
                                    "idle focus"
                                }
                            )
                        })
                        .collect::<Vec<_>>(),
                ),
                DaemonSnapshot::Unavailable => Some(vec!["Daemon unavailable".into()]),
                DaemonSnapshot::NoTabs => Some(vec!["No tabs reported".into()]),
            })
            .unwrap_or_else(|| vec!["Capsule is empty".into()]);
        List::new(CAPSULE_PANES).draw(ui, pane_area, &ListState::default(), &rows);
    }

    fn draw_content(&self, ui: &mut Ui<'_>, area: Rect) {
        Panel::new(APP.sub("content"))
            .title(self.route.title())
            .draw(ui, area, |ui, inner| match self.route {
                Route::Intro => self.draw_intro(ui, inner),
                Route::Manager => self.draw_manager(ui, inner),
                Route::Accounts => self.draw_accounts(ui, inner),
                Route::Usage => self.draw_usage(ui, inner),
                Route::Settings => self.draw_settings(ui, inner),
                Route::Launch => self.draw_launch(ui, inner),
                Route::Capsule => self.draw_capsule(ui, inner),
            });
    }

    fn draw_layers(&self, ui: &mut Ui<'_>) {
        let role_label = self.selected_role().to_owned();
        let dialog = self.launch_dialog();
        let _ = ui.layer(LAUNCH_DIALOG, |ui, area| {
            dialog.draw(ui, area, &self.launch_dialog, |ui, body| {
                Button::new(ROLE_CHOOSE, &role_label).draw(ui, body)
            })
        });
        let role_picker = Picker::new(ROLE_PICKER).title("Choose a role");
        let _ = ui.layer(ROLE_PICKER, |ui, area| {
            role_picker.draw(ui, area, &self.role_state, &self.roles)
        });
        let account_picker = Picker::new(ACCOUNT_PICKER).title("Choose a configured account");
        let _ = ui.layer(ACCOUNT_PICKER, |ui, area| {
            account_picker.draw(ui, area, &self.account_state, &self.account_options)
        });
    }
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(command) = cx.command()
            && let Some(result) = self.update_command(cx, command)
        {
            return result;
        }
        if cx.is_open(ROLE_PICKER) || cx.is_open(ACCOUNT_PICKER) || cx.is_open(LAUNCH_DIALOG) {
            return self.update_overlays(cx);
        }
        let mut result = Response::ignored();
        if self.route != Route::Intro {
            result |= self.update_navigation(cx);
        }
        result |= self.update_route(cx);
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let meta = format!("scenario · {}", self.world.scenario.name());
        Panel::new(APP)
            .title("Jackin Preview")
            .meta(&meta)
            .draw(ui, full, |ui, inner| {
                let header_height = inner.height.min(3);
                let footer_height = inner.height.saturating_sub(header_height).min(2);
                let header = Rect {
                    height: header_height,
                    ..inner
                };
                let footer_y = inner.bottom().saturating_sub(footer_height);
                let content = Rect {
                    y: header.bottom(),
                    height: footer_y.saturating_sub(header.bottom()),
                    ..inner
                };
                let footer = Rect {
                    y: footer_y,
                    height: footer_height,
                    ..inner
                };
                if self.route == Route::Intro {
                    self.draw_intro(ui, content);
                } else {
                    self.draw_header(ui, header);
                    self.draw_content(ui, content);
                }
                let style = ui.surface_style();
                if let Some(status) = &self.status {
                    ui.paint_str(footer, status, style);
                } else {
                    ui.paint_str(
                        footer,
                        "q quit · m manager · a accounts · u usage · s settings",
                        style,
                    );
                }
            });
        self.draw_layers(ui);
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }

    fn min_size(&self) -> tui_next::Size {
        tui_next::Size {
            min: (72, 20),
            preferred: (120, 40),
        }
    }

    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        if self.route != Route::Manager {
            self.route = Route::Manager;
            Response::changed()
        } else {
            self.quit = true;
            Response::changed()
        }
    }
}

fn app_keymap() -> KeyMap {
    KeyMap::new()
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('q')), CMD_QUIT)
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('m')),
            CMD_MANAGER,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('a')),
            CMD_ACCOUNTS,
        )
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Char('u')), CMD_USAGE)
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('s')),
            CMD_SETTINGS,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('c')),
            CMD_CAPSULE,
        )
}

fn plan_label(plan: LaunchPlan) -> &'static str {
    match plan {
        LaunchPlan::Clean => "clean",
        LaunchPlan::FailNetwork => "network failure",
        LaunchPlan::CredentialsLocked => "credentials locked",
        LaunchPlan::BlockedSidecar => "sidecar blocked",
    }
}

fn capsule_tabs() -> Vec<String> {
    vec!["Overview".into(), "Logs".into(), "Environment".into()]
}

fn paint_lines(ui: &mut Ui<'_>, area: Rect, lines: &[impl AsRef<str>]) {
    let style = ui.surface_style();
    for (index, line) in lines.iter().enumerate() {
        let Ok(offset) = u16::try_from(index) else {
            break;
        };
        let y = area.y.saturating_add(offset);
        if y >= area.bottom() {
            break;
        }
        ui.paint_str(Rect::new(area.x, y, area.width, 1), line.as_ref(), style);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_starts_in_returning_manager() {
        let app = App::default();
        assert_eq!(app.route(), Route::Manager);
        assert_eq!(app.world.running_count(), 1);
        assert_eq!(
            app.world.instances[0].run_id,
            crate::RunId::new(0x9c41_e2f0)
        );
    }

    #[test]
    fn route_construction_is_deterministic() {
        let a = App::for_scenario(Scenario::AccountsMixed, Motion::Paused);
        let b = App::for_scenario(Scenario::AccountsMixed, Motion::Paused);
        assert_eq!(format!("{a:?}"), format!("{b:?}"));
        assert_eq!(a.route(), Route::Accounts);
        assert_eq!(a.motion(), Motion::Paused);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::for_scenario(Scenario::Returning, Motion::Full)
    }
}

impl From<&Account> for AccountOption {
    fn from(account: &Account) -> Self {
        Self {
            key: account.id.clone(),
            label: account.title(),
            detail: account.source.safe_detail(),
        }
    }
}
