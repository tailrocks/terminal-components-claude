//! Jackin Preview application shell.
//!
//! This module owns only interaction state and paints through `junie-tui`'s
//! public facade.  Domain and simulation state stay in sibling modules.

use std::time::Duration;

use junie_tui::{
    ActionKey, App as TuiApp, AsItem, Button, Chord, Cx, Dialog, DialogAction, DialogState, Id,
    Item, ItemKey, KeyCode, KeyMap, KeyPhase, List, ListAction, ListState, Panel, Picker,
    PickerAction, PickerState, Rect, Response, StepState, Steps, StepsState, Tabs, TabsState, Ui,
    UpdateCause, Variant,
};

use crate::domain::account::Account;
use crate::domain::agent::Agent;
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::rain::{
    HANDOFF_LEN, INTRO_END, IntroPhase, IntroState, OutroPhase, OutroState, P1_LEN, PHRASES,
};
use crate::scenario::{Motion, Scenario};
use crate::sim::launch::{LaunchEvent, LaunchPlan, LaunchRun, Stage};
use crate::sim::world::{World, world_for};

/// Root id for the Jackin Preview component tree.
pub const APP: Id = Id::root("jackin.preview");
/// Intro entry action id.
pub const ENTER: Id = APP.sub("enter");
/// Manager route button id.
pub const MANAGER: Id = APP.sub("manager");
/// Accounts route button id.
pub const ACCOUNTS: Id = APP.sub("accounts");
/// Usage route button id.
pub const USAGE: Id = APP.sub("usage");
/// Settings route button id.
pub const SETTINGS: Id = APP.sub("settings");
/// Capsule route button id.
pub const CAPSULE: Id = APP.sub("capsule");
/// Manager instance list id.
pub const MANAGER_LIST: Id = APP.sub("manager-list");
/// Accounts list id.
pub const ACCOUNTS_LIST: Id = APP.sub("accounts-list");
/// Launch action id.
pub const LAUNCH: Id = APP.sub("launch");
/// Add-account action id.
pub const ACCOUNT_ADD: Id = APP.sub("account-add");
/// Trust-local-role action id.
pub const SETTINGS_TRUST: Id = APP.sub("settings-trust");
/// Capsule tab strip id.
pub const CAPSULE_TABS: Id = APP.sub("capsule-tabs");
/// Capsule pane list id.
pub const CAPSULE_PANES: Id = APP.sub("capsule-panes");
/// Launch confirmation dialog id.
pub const LAUNCH_DIALOG: Id = APP.sub("launch-dialog");
/// Role control inside the launch dialog.
pub const ROLE_CHOOSE: Id = LAUNCH_DIALOG.sub("role");
/// Role picker overlay id.
pub const ROLE_PICKER: Id = APP.sub("role-picker");
/// Account picker overlay id.
pub const ACCOUNT_PICKER: Id = APP.sub("account-picker");
/// Launch cancellation action id.
pub const LAUNCH_CANCEL: Id = APP.sub("launch-cancel");
/// Launch retry action id.
pub const LAUNCH_RETRY: Id = APP.sub("launch-retry");
/// Launch lifecycle rail id.
pub const LAUNCH_STEPS: Id = APP.sub("launch-steps");

const CMD_QUIT: ActionKey = ActionKey::custom("jackin.quit");
const CMD_MANAGER: ActionKey = ActionKey::custom("jackin.manager");
const CMD_ACCOUNTS: ActionKey = ActionKey::custom("jackin.accounts");
const CMD_USAGE: ActionKey = ActionKey::custom("jackin.usage");
const CMD_SETTINGS: ActionKey = ActionKey::custom("jackin.settings");
const CMD_CAPSULE: ActionKey = ActionKey::custom("jackin.capsule");
const TICK_MS: u64 = crate::rain::TICK_MS;

/// The visible product route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// First-use entry ritual.
    Intro,
    /// Workspace and running-instance manager.
    Manager,
    /// New-workspace prelude.
    Prelude,
    /// Workspace configuration editor.
    Editor,
    /// Account and usage center.
    Accounts,
    /// Usage summary.
    Usage,
    /// Application settings.
    Settings,
    /// Compatibility launch route.
    Launch,
    /// Active launch cockpit.
    Cockpit,
    /// Cockpit-to-Capsule handoff.
    Handoff,
    /// Running Capsule view.
    Capsule,
    /// Exit ritual.
    Outro,
}

impl Route {
    const fn title(self) -> &'static str {
        match self {
            Self::Intro => "Welcome to Jackin",
            Self::Manager => "Workspaces & instances",
            Self::Prelude => "Create workspace",
            Self::Editor => "Workspace editor",
            Self::Accounts => "Account & Usage Center",
            Self::Usage => "Usage overview",
            Self::Settings => "Settings",
            Self::Launch => "Launch cockpit",
            Self::Cockpit => "Launch cockpit",
            Self::Handoff => "Opening Capsule",
            Self::Capsule => "Capsule",
            Self::Outro => "Leaving the Construct",
        }
    }

    /// Virtual time cadence for one application tick.
    ///
    /// This is intentionally separate from runtime repaint scheduling.  The
    /// fixture clock and every deterministic state machine advance by this
    /// product-owned cadence only.
    pub const fn tick_ms(self) -> u64 {
        match self {
            Self::Intro | Self::Outro | Self::Handoff | Self::Cockpit | Self::Launch => TICK_MS,
            Self::Capsule => 80,
            Self::Manager
            | Self::Prelude
            | Self::Editor
            | Self::Accounts
            | Self::Usage
            | Self::Settings => 200,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LaunchStep {
    stage: Stage,
    state: StepState,
}

impl std::fmt::Display for LaunchStep {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.stage.label().fmt(formatter)
    }
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
    /// Stable manager row projections; rebuilt only when the fixture is built.
    ///
    /// The manager key path must not format or allocate merely to reconcile a
    /// list whose durable instance set is unchanged.
    manager_rows: Vec<String>,
    route: Route,
    motion: Motion,
    quit: bool,
    keymap: KeyMap,
    manager_state: ListState,
    accounts_state: ListState,
    tabs_state: TabsState,
    launch_steps_state: StepsState,
    launch_dialog: DialogState,
    role_state: PickerState,
    account_state: PickerState,
    roles: Vec<RoleOption>,
    account_options: Vec<AccountOption>,
    selected_role: usize,
    launch: Option<LaunchRun>,
    status: Option<String>,
    trusted: bool,
    intro: IntroState,
    outro: Option<OutroState>,
    handoff_frame: Option<u64>,
}

impl App {
    /// Build one deterministic app scenario.
    pub fn for_scenario(scenario: Scenario, motion: Motion) -> Self {
        Self::for_scenario_at(scenario, motion, 0)
    }

    /// Build one deterministic app scenario at a pinned virtual frame.
    ///
    /// Captures use this constructor instead of wall-clock time.  Paused
    /// mode remains frozen after construction, while full/reduced modes can
    /// continue from the same reproducible boundary on the next tick.
    pub fn for_scenario_at(scenario: Scenario, motion: Motion, frame: u64) -> Self {
        let mut world = world_for(scenario);
        let manager_rows = Self::build_manager_rows(&world);
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
        let mut route = match scenario {
            Scenario::FirstUse if frame >= INTRO_END => Route::Manager,
            Scenario::FirstUse => Route::Intro,
            Scenario::AccountsMixed => Route::Accounts,
            Scenario::LaunchRunning | Scenario::LaunchFailure => Route::Cockpit,
            Scenario::CapsuleMulti => Route::Capsule,
            Scenario::OutroLast if frame > 0 => Route::Outro,
            Scenario::OutroLast => Route::Capsule,
            Scenario::Returning | Scenario::HardCases => Route::Manager,
        };
        world.clock.running = motion != Motion::Paused;
        let frame_i64 = i64::try_from(frame).unwrap_or(i64::MAX);
        let cadence_i64 = i64::try_from(route.tick_ms()).unwrap_or(i64::MAX);
        world.clock.now_ms = frame_i64.saturating_mul(cadence_i64);
        world.last_refresh_secs = world.now_secs();
        let mut launch = matches!(route, Route::Launch | Route::Cockpit).then(|| {
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
        if let Some(run) = &mut launch {
            run.seek(frame);
            if run.done {
                route = Route::Handoff;
            }
        }
        Self {
            world,
            manager_rows,
            route,
            motion,
            quit: false,
            keymap: app_keymap(),
            manager_state: ListState::default(),
            accounts_state: ListState::default(),
            tabs_state: TabsState::default(),
            launch_steps_state: StepsState::default(),
            launch_dialog: DialogState::default(),
            role_state: PickerState::default(),
            account_state: PickerState::default(),
            roles,
            account_options,
            selected_role,
            launch,
            status: None,
            trusted: false,
            intro: IntroState::new(motion, frame),
            outro: (scenario == Scenario::OutroLast && frame > 0)
                .then(|| OutroState::new(motion, Some(8_040), frame)),
            handoff_frame: (route == Route::Handoff).then_some(0),
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

    /// The app's deterministic route cadence in milliseconds.
    pub const fn route_tick_ms(&self) -> u64 {
        self.route.tick_ms()
    }

    /// Current pinned virtual frame for ritual/cross-fade state.
    pub fn frame(&self) -> u64 {
        match self.route {
            Route::Intro => self.intro.tick,
            Route::Outro => self.outro.as_ref().map_or(0, |state| state.tick),
            Route::Handoff => self.handoff_frame.unwrap_or(0),
            Route::Launch | Route::Cockpit => self.launch.as_ref().map_or(0, |run| run.tick),
            _ => self.world.now_ms().div_euclid(self.route_tick_ms() as i64) as u64,
        }
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

    fn enter_button() -> Button<'static> {
        Button::new(ENTER, "Enter Construct").variant(Variant::PRIMARY)
    }

    fn account_add_button() -> Button<'static> {
        Button::new(ACCOUNT_ADD, "Choose 1Password reference…").variant(Variant::PRIMARY)
    }

    fn launch_button(disabled: bool) -> Button<'static> {
        Button::new(LAUNCH, "Launch session")
            .variant(Variant::PRIMARY)
            .disabled(disabled)
    }

    fn settings_trust_button(checked: bool) -> Button<'static> {
        Button::new(SETTINGS_TRUST, "Trust local incident role").checked(checked)
    }

    fn launch_retry_button() -> Button<'static> {
        Button::new(LAUNCH_RETRY, "Retry").variant(Variant::PRIMARY)
    }

    fn role_picker() -> Picker<'static, RoleOption> {
        Picker::new(ROLE_PICKER).title("Choose a role")
    }

    fn account_picker() -> Picker<'static, AccountOption> {
        Picker::new(ACCOUNT_PICKER).title("Choose a configured account")
    }

    fn shell_panel<'a>(meta: &'a str) -> Panel<'a> {
        Panel::new(APP).title("Jackin Preview").meta(meta)
    }

    fn build_manager_rows(world: &World) -> Vec<String> {
        world
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

    fn launch_steps(&self) -> Vec<LaunchStep> {
        let states = self
            .launch
            .as_ref()
            .map(|launch| launch.states)
            .unwrap_or([StepState::Queued; Stage::ALL.len()]);
        Stage::ALL
            .into_iter()
            .zip(states)
            .map(|(stage, state)| LaunchStep { stage, state })
            .collect()
    }

    fn launch_steps_component<'a>(
        state_of: &'a dyn Fn(&LaunchStep) -> StepState,
    ) -> Steps<'a, LaunchStep> {
        Steps::new(LAUNCH_STEPS).step(state_of)
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

    fn launch_dialog() -> Dialog<'static> {
        Dialog::confirm(
            LAUNCH_DIALOG,
            "Launch a new session",
            "Review the role and start a deterministic Construct run.",
        )
        .body_rows(1)
    }

    fn open_launch_dialog(&mut self, cx: &mut Cx<'_>) {
        let dialog = Self::launch_dialog();
        let spec = dialog.layer(cx);
        self.launch_dialog = DialogState::default();
        cx.open_layer(LAUNCH_DIALOG, spec);
    }

    fn open_role_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Self::role_picker();
        let spec = picker.layer(cx, &self.roles);
        cx.open_layer(ROLE_PICKER, spec);
    }

    fn open_account_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Self::account_picker();
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
        self.route = Route::Cockpit;
        self.handoff_frame = None;
        self.status = Some(format!(
            "Queued {} · {}",
            self.selected_role(),
            plan_label(plan)
        ));
    }

    fn update_overlays(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();

        // Drain the dialog and its body control while a child picker is
        // open too.  Layer dismissal intents are addressed to the underlying
        // owners during the same update pass; returning early for the top
        // picker leaves those intents undelivered and makes nested overlays
        // noisy in diagnostics.
        let dialog = Self::launch_dialog();
        let response = dialog.update(cx, &mut self.launch_dialog);
        let action = response.action_ref().copied();
        result |= response.erase();
        let role = Button::new(ROLE_CHOOSE, self.selected_role()).update(cx);
        let role_chosen = role.activated();
        result |= role.erase();
        if role_chosen && cx.is_open(LAUNCH_DIALOG) && !cx.is_open(ROLE_PICKER) {
            self.open_role_picker(cx);
        }
        if let Some(action) = action {
            match action {
                DialogAction::Action(ActionKey::CONFIRM) if cx.is_open(LAUNCH_DIALOG) => {
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

        let picker = Self::role_picker();
        let response = picker.update(cx, &mut self.role_state, &self.roles);
        let action = response.action_ref().copied();
        result |= response.erase();
        if cx.is_open(ROLE_PICKER)
            && let Some(PickerAction::Chosen(key)) = action
            && let Some(index) = self
                .roles
                .iter()
                .position(|role| ItemKey::text(&role.key) == key)
        {
            self.selected_role = index;
            cx.close_layer(ROLE_PICKER, Some(ActionKey::CONFIRM));
            result |= Response::changed();
        }

        let picker = Self::account_picker();
        let response = picker.update(cx, &mut self.account_state, &self.account_options);
        let action = response.action_ref().copied();
        result |= response.erase();
        if cx.is_open(ACCOUNT_PICKER)
            && let Some(PickerAction::Chosen(key)) = action
            && let Some(account) = self
                .account_options
                .iter()
                .find(|account| ItemKey::text(&account.key) == key)
        {
            self.status = Some(format!("Selected reference · {}", account.detail));
            cx.close_layer(ACCOUNT_PICKER, Some(ActionKey::CONFIRM));
            result |= Response::changed();
        }
        result
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
        let button = Self::enter_button().update(cx);
        let chosen = button.activated();
        let result = button.erase();
        if chosen {
            self.route = Route::Manager;
            self.world.arbiter.complete_entry(self.world.now_ms());
        }
        result
    }

    fn update_manager(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let list = List::new(MANAGER_LIST).update(cx, &mut self.manager_state, &self.manager_rows);
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
        let button = Self::launch_button(self.world.workspaces.is_empty()).update(cx);
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
        let add = Self::account_add_button().update(cx);
        let chosen = add.activated();
        result |= add.erase();
        if chosen {
            self.open_account_picker(cx);
        }
        result
    }

    fn update_settings(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let button = Self::settings_trust_button(self.trusted).update(cx);
        let chosen = button.activated();
        let result = button.erase();
        if chosen {
            self.trusted = !self.trusted;
        }
        result
    }

    fn update_launch(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let steps = self.launch_steps();
        let state_of = |step: &LaunchStep| step.state;
        let rail = Self::launch_steps_component(&state_of);
        result |= rail
            .update(cx, &mut self.launch_steps_state, &steps)
            .erase();
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
            let retry = Self::launch_retry_button().update(cx);
            let retry_chosen = retry.activated();
            result |= retry.erase();
            if retry_chosen {
                self.begin_launch();
                return result;
            }
        }
        if cx.update_cause() == UpdateCause::Tick
            && self.motion != Motion::Paused
            && let Some(launch) = &mut self.launch
        {
            let events = launch.advance();
            if !events.is_empty() {
                self.handle_launch_events(events);
                result |= Response::changed();
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
                    if self.world.running_count() > 0 {
                        self.route = Route::Manager;
                        self.status = Some(format!(
                            "Launch failed · {} · another instance is still running",
                            failure.summary
                        ));
                    }
                }
                LaunchEvent::Ready => {
                    self.status = Some("Capsule ready".into());
                    self.route = Route::Handoff;
                    self.handoff_frame = Some(0);
                }
                LaunchEvent::StageChanged(stage_kind, step_state) => {
                    self.status = Some(format!("{} · {}", stage_kind.label(), step_state.label()));
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
            Route::Prelude | Route::Editor => Response::ignored(),
            Route::Accounts => self.update_accounts(cx),
            Route::Usage => Response::ignored(),
            Route::Settings => self.update_settings(cx),
            Route::Launch | Route::Cockpit => self.update_launch(cx),
            Route::Handoff | Route::Outro => Response::ignored(),
            Route::Capsule => self.update_capsule(cx),
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

    fn advance_virtual_state(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() != UpdateCause::Tick || self.motion == Motion::Paused {
            return Response::ignored();
        }

        let cadence = i64::try_from(self.route_tick_ms()).unwrap_or(i64::MAX);
        let messages = self.world.tick(cadence);
        let mut result = if messages.is_empty() {
            Response::ignored()
        } else {
            Response::changed()
        };
        for message in messages {
            match message {
                crate::sim::world::Msg::WorkspaceSaved { id, ok } => {
                    self.status = Some(if ok {
                        format!("Workspace {id} saved")
                    } else {
                        format!("Workspace {id} save failed")
                    });
                }
                crate::sim::world::Msg::Refreshed { ok } => {
                    self.status = Some(if ok {
                        "Refresh complete".into()
                    } else {
                        "Refresh failed; last good data retained".into()
                    });
                }
                crate::sim::world::Msg::AccountRefreshed { account } => {
                    self.status = Some(format!("Account {account} refreshed"));
                }
            }
        }

        match self.route {
            Route::Intro => {
                if self.intro.advance() && self.intro.is_done() {
                    self.route = Route::Manager;
                    self.world.arbiter.complete_entry(self.world.now_ms());
                    result |= Response::changed();
                }
                cx.request_repaint_after(Duration::from_millis(TICK_MS));
            }
            Route::Outro => {
                if let Some(outro) = &mut self.outro
                    && outro.advance()
                    && outro.is_done()
                {
                    self.quit = true;
                    result |= Response::changed();
                }
                cx.request_repaint_after(Duration::from_millis(TICK_MS));
            }
            Route::Handoff => {
                let next = self.handoff_frame.unwrap_or(0).saturating_add(1);
                self.handoff_frame = Some(next);
                if next >= HANDOFF_LEN {
                    self.route = Route::Capsule;
                }
                result |= Response::changed();
                cx.request_repaint_after(Duration::from_millis(TICK_MS));
            }
            Route::Cockpit | Route::Launch => {
                cx.request_repaint_after(Duration::from_millis(TICK_MS));
            }
            _ => {}
        }
        result
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
        let message = match self.intro.phase() {
            IntroPhase::Phrases => {
                let index = self.intro.tick / P1_LEN;
                PHRASES
                    .get(usize::try_from(index).unwrap_or(0))
                    .map_or("Stand up, operator…", |(text, _, _)| *text)
            }
            IntroPhase::Warp => "Knock, knock, operator. · opening the Construct",
            IntroPhase::Done => "Construct ready. Choose a workspace to continue.",
        };
        ui.paint_str(
            Rect {
                height: area.height.min(1),
                ..area
            },
            message,
            style,
        );
        if self.intro.phase() == IntroPhase::Phrases {
            ui.paint_str(
                Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
                "No running instances found. The first launch owns the Construct entry ritual.",
                style,
            );
            Self::enter_button().draw(
                ui,
                Rect {
                    y: area.y.saturating_add(3),
                    width: area.width.min(24),
                    height: 1,
                    ..area
                },
            );
        }
    }

    fn draw_prelude(&self, ui: &mut Ui<'_>, area: Rect) {
        let workspace = self
            .world
            .workspaces
            .first()
            .map(|workspace| workspace.name.as_str())
            .unwrap_or("new workspace");
        let lines = [
            "Create workspace · step 1 of 5",
            "Source · choose a local repository",
            "Destination · choose a Workspace name",
            "Mounts and environment · review before save",
            workspace,
        ];
        paint_lines(ui, area, &lines);
    }

    fn draw_editor(&self, ui: &mut Ui<'_>, area: Rect) {
        let workspace = self
            .world
            .workspaces
            .first()
            .map(|workspace| workspace.name.as_str())
            .unwrap_or("new workspace");
        let lines = [
            format!("{workspace} · edit"),
            "Mounts · inherited defaults".to_owned(),
            "Environments · references only; values stay masked".to_owned(),
            format!("Roles · {} configured", self.world.roles.len()),
            "Save workspace · Ctrl+S".to_owned(),
        ];
        paint_lines(ui, area, &lines);
    }

    fn draw_handoff(&self, ui: &mut Ui<'_>, area: Rect) {
        let stage = crate::rain::handoff_stage(self.handoff_frame.unwrap_or(0));
        let label = match stage {
            crate::rain::HandoffStage::CockpitDim(step) => {
                format!("Opening Capsule · fading launch cockpit ({step}/4)")
            }
            crate::rain::HandoffStage::Canvas => "Opening Capsule · settling canvas".into(),
            crate::rain::HandoffStage::CapsuleDim(step) => {
                format!("Opening Capsule · revealing panes ({step}/4)")
            }
            crate::rain::HandoffStage::Capsule => "Capsule ready".into(),
        };
        paint_lines(
            ui,
            area,
            &[
                label,
                "The daemon owns pane state; the shell owns the handoff.".to_owned(),
            ],
        );
    }

    fn draw_outro(&self, ui: &mut Ui<'_>, area: Rect) {
        let Some(outro) = &self.outro else {
            paint_lines(ui, area, &["Detached from the Construct."]);
            return;
        };
        let line = match outro.phase() {
            OutroPhase::Warp => "Leaving the Construct · closing Capsule".to_owned(),
            OutroPhase::Caption => outro
                .caption()
                .unwrap_or_else(|| "Leaving the Construct · goodbye, operator.".to_owned()),
            OutroPhase::Done => "Detached from the Construct.".to_owned(),
        };
        paint_lines(
            ui,
            area,
            &[
                line,
                "No host process or wall-clock state is consulted.".to_owned(),
            ],
        );
    }

    fn draw_manager(&self, ui: &mut Ui<'_>, area: Rect) {
        let list_area = Rect {
            height: area.height.saturating_sub(3),
            ..area
        };
        List::new(MANAGER_LIST).draw(ui, list_area, &self.manager_state, &self.manager_rows);
        Self::launch_button(self.world.workspaces.is_empty()).draw(
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
        Self::account_add_button().draw(
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
        Self::settings_trust_button(self.trusted).draw(
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
        let steps = self.launch_steps();
        let state_of = |step: &LaunchStep| step.state;
        let rail = Self::launch_steps_component(&state_of);
        let rail_area = Rect {
            width: area.width.min(42),
            height: area.height.saturating_sub(3),
            ..area
        };
        rail.draw(ui, rail_area, &self.launch_steps_state, &steps);
        let log_x = area.x.saturating_add(44).min(area.right());
        let log_width = area.right().saturating_sub(log_x);
        if log_width > 0 {
            let log_lines: Vec<String> = if launch.build_lines_emitted == 0 {
                vec!["Docker build · waiting for Derived Image".into()]
            } else {
                crate::sim::launch::BUILD_LOG
                    .iter()
                    .take(launch.build_lines_emitted)
                    .rev()
                    .take(5)
                    .map(|line| (*line).to_owned())
                    .collect()
            };
            paint_lines(
                ui,
                Rect::new(log_x, area.y, log_width, area.height),
                &log_lines,
            );
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
            Self::launch_retry_button().draw(
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
            .map_or_else(
                || vec!["Capsule is empty".into()],
                |instance| match &instance.daemon {
                    DaemonSnapshot::Tabs(tabs) => tabs
                        .iter()
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
                    DaemonSnapshot::Unavailable => vec!["Daemon unavailable".into()],
                    DaemonSnapshot::NoTabs => vec!["No tabs reported".into()],
                },
            );
        List::new(CAPSULE_PANES).draw(ui, pane_area, &ListState::default(), &rows);
    }

    fn draw_content(&self, ui: &mut Ui<'_>, area: Rect) {
        Panel::new(APP.sub("content"))
            .title(self.route.title())
            .draw(ui, area, |ui, inner| match self.route {
                Route::Intro => self.draw_intro(ui, inner),
                Route::Manager => self.draw_manager(ui, inner),
                Route::Prelude => self.draw_prelude(ui, inner),
                Route::Editor => self.draw_editor(ui, inner),
                Route::Accounts => self.draw_accounts(ui, inner),
                Route::Usage => self.draw_usage(ui, inner),
                Route::Settings => self.draw_settings(ui, inner),
                Route::Launch | Route::Cockpit => self.draw_launch(ui, inner),
                Route::Handoff => self.draw_handoff(ui, inner),
                Route::Capsule => self.draw_capsule(ui, inner),
                Route::Outro => self.draw_outro(ui, inner),
            });
    }

    fn draw_layers(&self, ui: &mut Ui<'_>) {
        let role_label = self.selected_role().to_owned();
        let dialog = Self::launch_dialog();
        let _ = ui.layer(LAUNCH_DIALOG, |ui, area| {
            dialog.draw(ui, area, &self.launch_dialog, |ui, body| {
                Button::new(ROLE_CHOOSE, &role_label).draw(ui, body)
            })
        });
        let role_picker = Self::role_picker();
        let _ = ui.layer(ROLE_PICKER, |ui, area| {
            role_picker.draw(ui, area, &self.role_state, &self.roles)
        });
        let account_picker = Self::account_picker();
        let _ = ui.layer(ACCOUNT_PICKER, |ui, area| {
            account_picker.draw(ui, area, &self.account_state, &self.account_options)
        });
    }
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = self.advance_virtual_state(cx);
        if let Some(command) = cx.command()
            && let Some(result) = self.update_command(cx, command)
        {
            return result;
        }
        result |= self.update_overlays(cx);
        if cx.is_open(ROLE_PICKER) || cx.is_open(ACCOUNT_PICKER) || cx.is_open(LAUNCH_DIALOG) {
            return result;
        }
        if self.route != Route::Intro {
            result |= self.update_navigation(cx);
        }
        result |= self.update_route(cx);
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let meta = format!("scenario · {}", self.world.scenario.name());
        Self::shell_panel(&meta).draw(ui, full, |ui, inner| {
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

    fn min_size(&self) -> junie_tui::Size {
        junie_tui::Size {
            min: (72, 20),
            preferred: (120, 40),
        }
    }

    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        if self.route == Route::Outro {
            self.quit = true;
            return Response::changed();
        }
        if self.route == Route::Capsule && self.world.scenario == Scenario::OutroLast {
            self.status = Some("Detached from Capsule; closing the Construct…".into());
            self.outro = Some(OutroState::new(
                self.motion,
                Some((self.world.now_ms().max(0) as u64) / 1000),
                0,
            ));
            self.route = Route::Outro;
            return Response::changed();
        }
        if self.route == Route::Capsule && self.world.running_count() > 1 {
            self.status = Some("Still inside the Construct · another instance is running".into());
            self.route = Route::Manager;
            return Response::changed();
        }
        if self.route == Route::Manager {
            self.quit = true;
            Response::changed()
        } else {
            self.route = Route::Manager;
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
