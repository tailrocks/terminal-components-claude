//! Jackin Preview application shell.
//!
//! This module owns only interaction state and paints through `tui-next`'s
//! public facade.  Domain and simulation state stay in sibling modules.

use std::{mem, time::Duration};

use junie_tui::{
    ActionKey, App as TuiApp, AsItem, Button, Chord, Cx, Dialog, DialogAction, DialogState,
    FrameRead, Id, Intent, Item, ItemKey, KeyCode, KeyMap, KeyModifiers, KeyPhase, List,
    ListAction, ListState, Panel, Picker, PickerAction, PickerState, Rect, Response, SecretPolicy,
    Tabs, TabsState, TextAction, TextInput, TextInputState, TooSmall, Ui, UpdateCause, Variant,
};

use crate::domain::account::{
    Account, CredentialSource, DetectedKind, DuplicateProbe, fingerprint, tail_of,
};
use crate::domain::agent::{Agent, Provider};
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::domain::workspace::{EnvValue, EnvVar, env_key_error, mask};
use crate::rain::{
    HANDOFF_LEN, INTRO_END, IntroPhase, IntroState, OutroPhase, OutroState, P1_LEN, PHRASES,
};
use crate::scenario::{Motion, Scenario};
use crate::screens::{
    accounts::AccountsState,
    capsule::{
        CapsuleFocus, CapsuleInteraction, CapsuleLayer, CapsuleState, ExitDecision, PrefixCommand,
    },
    cockpit::{AccountLine, CockpitState},
    editor::{EditorState, Tab as EditorTab},
    inspect::InspectState,
    manager::{LaunchCandidate, ManagerRowKey, ManagerState},
    prelude::PreludeState,
    settings::SettingsState,
    usage::{Tab as UsageTab, UsageState},
};
use crate::sim::launch::{LaunchEvent, LaunchPlan, LaunchRun, Stage};
use crate::sim::provider;
use crate::sim::pty::{Daemon, SplitDir};
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
pub const MANAGER_LIST: Id = crate::screens::manager::TREE;
/// Accounts list id.
pub const ACCOUNTS_LIST: Id = crate::screens::accounts::LIST;
/// Launch action id.
pub const LAUNCH: Id = crate::screens::manager::LAUNCH;
/// Add-account action id.
pub const ACCOUNT_ADD: Id = APP.sub("account-add");
/// Trust-local-role action id.
pub const SETTINGS_TRUST: Id = crate::screens::settings::TRUST;
/// Capsule tab strip id.
pub const CAPSULE_TABS: Id = crate::screens::capsule::TABS;
/// Capsule pane list id.
pub const CAPSULE_PANES: Id = crate::screens::capsule::PANES;
/// Capsule command input id.
pub const CAPSULE_INPUT: Id = APP.sub("capsule-input");
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
const EDITOR_MOUNT_EDIT: Id = crate::screens::editor::ROOT.sub("mount-edit");
const EDITOR_ROLE_EDIT: Id = crate::screens::editor::ROOT.sub("role-edit");
const EDITOR_ROLE_LOAD: Id = crate::screens::editor::ROOT.sub("role-load");
const EDITOR_ACCOUNTS_LIST: Id = crate::screens::editor::ROOT.sub("accounts-list");
const EDITOR_SAVE_CONFIRM: Id = crate::screens::editor::ROOT.sub("save-confirm");
const SETTINGS_SAVE_CONFIRM: Id = crate::screens::settings::ROOT.sub("save-confirm");

const CMD_QUIT: ActionKey = ActionKey::custom("jackin.quit");
const CMD_MANAGER: ActionKey = ActionKey::custom("jackin.manager");
const CMD_ACCOUNTS: ActionKey = ActionKey::custom("jackin.accounts");
const CMD_USAGE: ActionKey = ActionKey::custom("jackin.usage");
const CMD_SETTINGS: ActionKey = ActionKey::custom("jackin.settings");
const CMD_CAPSULE: ActionKey = ActionKey::custom("jackin.capsule");
const CMD_NEW_WORKSPACE: ActionKey = ActionKey::custom("jackin.new-workspace");
const CMD_EDITOR_NEXT: ActionKey = ActionKey::custom("jackin.editor.next-tab");
const CMD_EDITOR_PREVIOUS: ActionKey = ActionKey::custom("jackin.editor.previous-tab");
const CMD_EDITOR_ENV: ActionKey = ActionKey::custom("jackin.editor.environments");
const CMD_SAVE: ActionKey = ActionKey::custom("jackin.save");
const CMD_MANAGER_EXPAND: ActionKey = ActionKey::custom("jackin.manager.expand");
const CMD_EDITOR_OPEN: ActionKey = ActionKey::custom("jackin.editor.open");
const CMD_EDITOR_ROLES: ActionKey = ActionKey::custom("jackin.editor.roles");
const CMD_EDITOR_ACCOUNTS: ActionKey = ActionKey::custom("jackin.editor.accounts");
const CMD_EDITOR_PREFER: ActionKey = ActionKey::custom("jackin.editor.prefer");
const CMD_SETTINGS_TRUST_KEY: ActionKey = ActionKey::custom("jackin.settings.trust-key");
const CMD_USAGE_NEXT: ActionKey = ActionKey::custom("jackin.usage.next");
const CMD_CAPSULE_PREFIX: ActionKey = ActionKey::custom("jackin.capsule.prefix");
const CMD_CAPSULE_DETACH: ActionKey = ActionKey::custom("jackin.capsule.detach");
const CMD_CAPSULE_SPLIT_RIGHT: ActionKey = ActionKey::custom("jackin.capsule.split-right");
const CMD_CAPSULE_SPLIT_BELOW: ActionKey = ActionKey::custom("jackin.capsule.split-below");
const CMD_CAPSULE_ZOOM: ActionKey = ActionKey::custom("jackin.capsule.zoom");
const CMD_CAPSULE_FOCUS_LEFT: ActionKey = ActionKey::custom("jackin.capsule.focus-left");
const CMD_CAPSULE_PALETTE: ActionKey = ActionKey::custom("jackin.capsule.palette");
const CMD_EXIT_DIALOG: ActionKey = ActionKey::custom("jackin.exit.dialog");
const CMD_EXIT_CONFIRM: ActionKey = ActionKey::custom("jackin.exit.confirm");
const CMD_PRELUDE_BACKSPACE: ActionKey = ActionKey::custom("jackin.prelude.backspace");
const CMD_PRELUDE_DOWN: ActionKey = ActionKey::custom("jackin.prelude.down");
const CMD_PRELUDE_SPACE: ActionKey = ActionKey::custom("jackin.prelude.space");
const CMD_ACCOUNT_DOWN: ActionKey = ActionKey::custom("jackin.account.down");
const CMD_ACCOUNT_REFRESH: ActionKey = ActionKey::custom("jackin.account.refresh");
const CMD_ACCOUNT_VALIDATE: ActionKey = ActionKey::custom("jackin.account.validate");
const CMD_ACCOUNT_REMOVE: ActionKey = ActionKey::custom("jackin.account.remove");
const CMD_ACCOUNT_DEFAULT: ActionKey = ActionKey::custom("jackin.account.default");
const CMD_ACCOUNT_HELP: ActionKey = ActionKey::custom("jackin.account.help");
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
enum PickerMode {
    Launch,
    OnePassword,
    Capsule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CapsuleAction {
    NewTab,
    Split(SplitDir),
}

impl AsItem for AccountOption {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::text(&self.key), &self.label).detail(&self.detail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentOption {
    key: String,
    label: String,
    detail: String,
    agent: Agent,
    account: Option<String>,
    blocked: bool,
}

impl AgentOption {
    fn from_candidate(candidate: LaunchCandidate, world: &World) -> Self {
        let account = candidate.account.clone();
        let account_label = account
            .as_deref()
            .and_then(|id| world.accounts.get(id))
            .map(Account::title);
        let detail = match (&account_label, &candidate.blocked) {
            (_, Some(reason)) => format!("blocked · {reason}"),
            (Some(account), None) => format!("ready · {account}"),
            (None, None) => "ready".to_owned(),
        };
        Self {
            key: candidate.agent.short().to_owned(),
            label: candidate.agent.label().to_owned(),
            detail,
            agent: candidate.agent,
            account,
            blocked: candidate.blocked.is_some(),
        }
    }
}

impl AsItem for AgentOption {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::text(&self.key), &self.label).detail(&self.detail)
    }
}

/// The Jackin Preview application.
#[derive(Debug, Clone)]
pub struct App {
    /// Deterministic services and durable state exposed for focused tests.
    pub world: World,
    /// Manager route state and public list ownership.
    pub manager: ManagerState,
    /// Accounts route state and account-form ownership.
    pub accounts: AccountsState,
    /// Workspace creation route state.
    pub prelude: PreludeState,
    /// Workspace editor route state.
    pub editor: EditorState,
    /// Settings route state.
    pub settings: SettingsState,
    /// Read-only usage route state.
    pub usage: UsageState,
    /// Launch cockpit state.
    pub cockpit: CockpitState,
    /// Capsule interaction state.
    pub capsule: CapsuleState,
    /// Read-only instance inspection state.
    pub inspect: InspectState,
    /// Cached manager projection; rebuilt only when expansion or source data changes.
    manager_rows_cache: Vec<String>,
    manager_rows_revision: u64,
    shell_meta: String,
    manager_header: String,
    manager_header_running: usize,
    route: Route,
    motion: Motion,
    quit: bool,
    keymap: KeyMap,
    tabs_state: TabsState,
    launch_dialog: DialogState,
    role_state: PickerState,
    agent_state: PickerState,
    account_state: PickerState,
    roles: Vec<RoleOption>,
    agent_options: Vec<AgentOption>,
    account_options: Vec<AccountOption>,
    op_options: Vec<AccountOption>,
    op_item_key: String,
    picker_mode: Option<PickerMode>,
    selected_role: usize,
    launch: Option<LaunchRun>,
    status: Option<String>,
    trusted: bool,
    intro: IntroState,
    outro: Option<OutroState>,
    handoff_frame: Option<u64>,
    capsule_prefix: bool,
    capsule_usage: bool,
    exit_choice: Option<u8>,
    capsule_input: String,
    capsule_input_state: TextInputState,
    pending_capsule_action: Option<CapsuleAction>,
    capsule_interaction: CapsuleInteraction,
    editor_accounts: ListState,
    usage_list: ListState,
    active_instance: Option<String>,
    launch_agent: Agent,
    launch_account: Option<String>,
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
        let manager_header_running = world.running_count();
        let manager_header = format!(
            "Current directory · {} · {} running",
            world.home, manager_header_running
        );
        let mut app = Self {
            world,
            manager: ManagerState::default(),
            accounts: AccountsState::default(),
            prelude: PreludeState::default(),
            editor: EditorState::default(),
            settings: SettingsState::default(),
            usage: UsageState::default(),
            cockpit: CockpitState::default(),
            capsule: CapsuleState::default(),
            inspect: InspectState::default(),
            manager_rows_cache: Vec::new(),
            manager_rows_revision: 0,
            shell_meta: format!("scenario · {}", scenario.name()),
            manager_header,
            manager_header_running,
            route,
            motion,
            quit: false,
            keymap: app_keymap(),
            tabs_state: TabsState::default(),
            launch_dialog: DialogState::default(),
            role_state: PickerState::default(),
            agent_state: PickerState::default(),
            account_state: PickerState::default(),
            roles,
            agent_options: Vec::new(),
            account_options,
            op_options: Vec::new(),
            op_item_key: String::new(),
            picker_mode: None,
            selected_role,
            launch,
            status: None,
            trusted: false,
            intro: IntroState::new(motion, frame),
            outro: (scenario == Scenario::OutroLast && frame > 0)
                .then(|| OutroState::new(motion, Some(8_040), frame)),
            handoff_frame: (route == Route::Handoff).then_some(0),
            capsule_prefix: false,
            capsule_usage: false,
            exit_choice: None,
            capsule_input: String::new(),
            capsule_input_state: TextInputState::default(),
            pending_capsule_action: None,
            capsule_interaction: CapsuleInteraction::default(),
            editor_accounts: ListState::default(),
            usage_list: ListState::default(),
            active_instance: None,
            launch_agent: Agent::ClaudeCode,
            launch_account: None,
        };
        if app.launch.as_ref().is_some_and(|run| run.done) {
            app.materialize_launch();
        }
        if app.route == Route::Handoff {
            app.cockpit.handoff.start();
        }
        if app.route == Route::Capsule {
            app.active_instance = app
                .world
                .instances
                .iter()
                .find(|instance| instance.status == InstanceStatus::Running)
                .map(|instance| instance.id.clone());
            app.sync_capsule_projection();
        }
        app
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

    /// Whether the exit ritual has completed.
    pub const fn should_quit(&self) -> bool {
        self.quit
    }

    fn launch_candidates(&self) -> Vec<AgentOption> {
        let workspace = self
            .manager
            .selected()
            .or_else(|| self.world.workspaces.first().map(|workspace| workspace.id));
        ManagerState::launch_candidates(&self.world, workspace, Some(self.selected_role()))
            .into_iter()
            .map(|candidate| AgentOption::from_candidate(candidate, &self.world))
            .collect()
    }

    fn selected_instance_id(&self) -> Option<String> {
        match self.manager.selected_row() {
            ManagerRowKey::Instance(id) => Some(id.clone()),
            ManagerRowKey::Workspace(workspace) => self
                .world
                .instances_of(Some(*workspace))
                .into_iter()
                .find(|instance| instance.status.reconnectable())
                .map(|instance| instance.id.clone()),
            ManagerRowKey::CurrentDirectory | ManagerRowKey::NewWorkspace => self
                .world
                .running()
                .first()
                .map(|instance| instance.id.clone()),
        }
    }

    fn sync_capsule_projection(&mut self) {
        let Some(instance_id) = self.active_instance.clone().or_else(|| {
            self.world
                .running()
                .first()
                .map(|instance| instance.id.clone())
        }) else {
            return;
        };
        self.active_instance = Some(instance_id.clone());
        let Some(daemon) = self.world.daemons.get(&instance_id) else {
            return;
        };
        self.capsule.tab = u8::try_from(daemon.active).unwrap_or(u8::MAX);
        self.capsule.selected_pane = daemon.focused_pane().unwrap_or_default();
        self.capsule.zoomed = daemon.active_tab().is_some_and(|tab| tab.zoomed.is_some());
    }

    fn refresh_cockpit_account_line(&mut self, agent: Agent) {
        let workspace = self.world.workspaces.first().or_else(|| {
            self.manager
                .selected()
                .and_then(|id| self.world.workspace(id))
        });
        let labels = self
            .world
            .offer_for(agent, workspace, Some(self.selected_role()))
            .accounts
            .into_iter()
            .filter_map(|id| self.world.accounts.get(&id).map(Account::title));
        self.cockpit.account_line = AccountLine::from_labels(labels);
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

    fn new_workspace_button() -> Button<'static> {
        Button::new(crate::screens::manager::NEW_WORKSPACE, "+ New workspace")
            .variant(Variant::PRIMARY)
    }

    fn account_start_button() -> Button<'static> {
        Button::new(crate::screens::accounts::START, "New account").variant(Variant::PRIMARY)
    }

    fn account_agent_button() -> Button<'static> {
        Button::new(crate::screens::accounts::AGENT, "Claude Code").checked(true)
    }

    fn account_save_button() -> Button<'static> {
        Button::new(crate::screens::accounts::SAVE, "Save account").variant(Variant::PRIMARY)
    }

    fn editor_save_button() -> Button<'static> {
        Button::new(crate::screens::editor::SAVE, "Save workspace").variant(Variant::PRIMARY)
    }

    fn editor_save_confirm_button() -> Button<'static> {
        Button::new(EDITOR_SAVE_CONFIRM, "Apply changes").variant(Variant::PRIMARY)
    }

    fn settings_save_confirm_button() -> Button<'static> {
        Button::new(SETTINGS_SAVE_CONFIRM, "Apply settings").variant(Variant::PRIMARY)
    }

    fn editor_mount_button() -> Button<'static> {
        Button::new(EDITOR_MOUNT_EDIT, "Edit mount")
    }

    fn editor_role_button() -> Button<'static> {
        Button::new(EDITOR_ROLE_EDIT, "Default role")
    }

    fn editor_role_load_button() -> Button<'static> {
        Button::new(EDITOR_ROLE_LOAD, "+ Load role…")
    }

    fn editor_env_source_button() -> Button<'static> {
        Button::new(crate::screens::editor::ENV_SOURCE, "Plain text")
    }

    fn editor_env_key_input() -> TextInput<'static> {
        TextInput::new(crate::screens::editor::ENV_KEY).placeholder("Variable name")
    }

    fn editor_env_value_input() -> TextInput<'static> {
        TextInput::new(crate::screens::editor::ENV_VALUE)
            .placeholder("Value")
            .secret(SecretPolicy::default())
    }

    fn prelude_continue_button() -> Button<'static> {
        Button::new(crate::screens::prelude::CONTINUE, "Continue").variant(Variant::PRIMARY)
    }

    fn account_name_input() -> TextInput<'static> {
        TextInput::new(crate::screens::accounts::NAME).placeholder("Display name")
    }

    fn account_folder_input() -> TextInput<'static> {
        TextInput::new(crate::screens::accounts::FOLDER).placeholder("Local agent folder")
    }

    fn account_secret_input() -> TextInput<'static> {
        TextInput::new(crate::screens::accounts::SECRET)
            .placeholder("API key")
            .secret(SecretPolicy::default())
    }

    fn text_input_empty_commit(cx: &Cx<'_>, id: Id, state: &TextInputState) -> bool {
        state.is_editing()
            && cx
                .intents(id)
                .any(|intent| matches!(intent, Intent::Binding(_)))
    }

    fn rearm_text_input(should_rearm: bool, state: &mut TextInputState, value: &str) -> bool {
        if should_rearm && !state.is_editing() && value.is_empty() {
            state.begin(value);
            true
        } else {
            false
        }
    }

    fn role_picker() -> Picker<'static, RoleOption> {
        Picker::new(ROLE_PICKER).title("Choose a role")
    }

    fn account_picker() -> Picker<'static, AccountOption> {
        Picker::new(ACCOUNT_PICKER).title("Choose a configured account")
    }

    fn active_account_picker(&self) -> Picker<'static, AccountOption> {
        match self.picker_mode {
            Some(PickerMode::OnePassword) => {
                Self::account_picker().title("Choose 1Password account")
            }
            _ => Self::account_picker(),
        }
    }

    fn shell_panel<'a>(meta: &'a str) -> Panel<'a> {
        Panel::new(APP).title("Jackin Preview").meta(meta)
    }

    fn build_manager_rows(&self) -> Vec<String> {
        let mut rows = Vec::new();
        for workspace in &self.world.workspaces {
            let expanded = self.manager.is_expanded(workspace.id);
            let marker = if expanded { "▾" } else { "▸" };
            let count = self
                .world
                .instances
                .iter()
                .filter(|instance| {
                    instance.workspace == Some(workspace.id) && !instance.status.hidden()
                })
                .count();
            rows.push(format!(
                "{marker} {} · {count} instance{}",
                workspace.name,
                if count == 1 { "" } else { "s" }
            ));
            if expanded {
                for instance in self.world.instances.iter().filter(|instance| {
                    instance.workspace == Some(workspace.id) && !instance.status.hidden()
                }) {
                    rows.push(format!(
                        "  {} · instance · {} · run {} · {}",
                        instance.id,
                        instance.status.label(),
                        instance.run_id.short(),
                        instance.dirty_summary()
                    ));
                }
            }
        }
        if rows.is_empty() && !self.world.instances.is_empty() {
            rows.extend(
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
                    }),
            );
        }
        rows
    }

    fn ensure_manager_rows(&mut self) {
        if self.manager_rows_cache.is_empty()
            || self.manager_rows_revision != self.manager.rows_revision()
        {
            self.manager_rows_cache = self.build_manager_rows();
            self.manager_rows_revision = self.manager.rows_revision();
        }
    }

    fn manager_row_at(&self, index: usize) -> Option<ManagerRowKey> {
        let mut cursor = 0usize;
        for workspace in &self.world.workspaces {
            if cursor == index {
                return Some(ManagerRowKey::Workspace(workspace.id));
            }
            cursor = cursor.saturating_add(1);
            if self.manager.is_expanded(workspace.id) {
                for instance in self.world.instances.iter().filter(|instance| {
                    instance.workspace == Some(workspace.id) && !instance.status.hidden()
                }) {
                    if cursor == index {
                        return Some(ManagerRowKey::Instance(instance.id.clone()));
                    }
                    cursor = cursor.saturating_add(1);
                }
            }
        }
        self.world
            .instances
            .iter()
            .filter(|instance| instance.workspace.is_none() && !instance.status.hidden())
            .nth(index.saturating_sub(cursor))
            .map(|instance| ManagerRowKey::Instance(instance.id.clone()))
    }

    fn ensure_manager_header(&mut self) {
        let running = self.world.running_count();
        if running != self.manager_header_running {
            self.manager_header = format!(
                "Current directory · {} · {} running",
                self.world.home, running
            );
            self.manager_header_running = running;
        }
    }

    fn account_rows(&self) -> Vec<String> {
        let mut rows = vec!["Overview".to_owned()];
        rows.extend(
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
                .collect::<Vec<_>>(),
        );
        rows
    }

    fn editor_account_rows(&self) -> Vec<String> {
        self.editor
            .pending
            .effective_accounts(&self.world.accounts)
            .into_iter()
            .map(|account| {
                format!(
                    "{} · {}",
                    self.world
                        .accounts
                        .get(&account.id)
                        .map_or_else(|| account.id.clone(), Account::title),
                    if account.preferred {
                        "preferred"
                    } else {
                        "active for this Workspace"
                    }
                )
            })
            .collect()
    }

    fn editor_account_id(&self, index: usize) -> Option<String> {
        self.editor
            .pending
            .effective_accounts(&self.world.accounts)
            .get(index)
            .map(|account| account.id.clone())
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

    fn open_agent_picker(&mut self, cx: &mut Cx<'_>) {
        self.agent_options = self.launch_candidates();
        self.agent_state = PickerState::default();
        if self.agent_options.is_empty() {
            self.status = Some("No configured agent account is available".into());
            return;
        }
        let picker =
            Picker::new(crate::screens::manager::AGENT_PICKER).title("Launch · choose Agent");
        let spec = picker.layer(cx, &self.agent_options);
        cx.open_layer(crate::screens::manager::AGENT_PICKER, spec);
        self.status = Some("Launch · choose Agent".into());
    }

    fn open_role_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Self::role_picker();
        let spec = picker.layer(cx, &self.roles);
        cx.open_layer(ROLE_PICKER, spec);
    }

    fn open_account_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Self::account_picker();
        self.picker_mode = Some(PickerMode::Launch);
        self.account_state = PickerState::default();
        let spec = picker.layer(cx, &self.account_options);
        cx.open_layer(ACCOUNT_PICKER, spec);
    }

    fn open_capsule_account_picker(&mut self, cx: &mut Cx<'_>, action: CapsuleAction) {
        let picker = Self::account_picker().title("Account for Claude Code");
        self.pending_capsule_action = Some(action);
        self.picker_mode = Some(PickerMode::Capsule);
        self.account_state = PickerState::default();
        let spec = picker.layer(cx, &self.account_options);
        cx.open_layer(ACCOUNT_PICKER, spec);
    }

    fn apply_capsule_action(&mut self, action: CapsuleAction, account: AccountOption) {
        let Some(instance_id) = self
            .world
            .instances
            .iter()
            .find(|instance| instance.status == InstanceStatus::Running)
            .map(|instance| instance.id.clone())
        else {
            self.status = Some("Capsule unavailable · no running instance".into());
            return;
        };
        let now_ms = self.world.now_ms();
        let observed_secs = self.world.now_secs();
        let (snapshot, status) = {
            let Some(daemon) = self.world.daemons.get_mut(&instance_id) else {
                self.status = Some("Capsule unavailable · daemon not connected".into());
                return;
            };
            let result = match action {
                CapsuleAction::NewTab => {
                    daemon.new_tab(
                        Some(Agent::ClaudeCode),
                        Some(account.key.clone()),
                        now_ms,
                        true,
                    );
                    format!("New tab · Account for Claude Code · {}", account.label)
                }
                CapsuleAction::Split(direction) => {
                    let _ = daemon.split(
                        direction,
                        false,
                        Some(Agent::ClaudeCode),
                        Some(account.key.clone()),
                        now_ms,
                        true,
                    );
                    let label = match direction {
                        SplitDir::Horizontal => "Split right",
                        SplitDir::Vertical => "Split below",
                    };
                    format!("{label} · Account for Claude Code · {}", account.label)
                }
            };
            (daemon.snapshot(), result)
        };
        if let Some(instance) = self.world.instance_mut(&instance_id) {
            instance.daemon = snapshot;
            instance.last_seen_secs = observed_secs;
        }
        self.status = Some(status);
    }

    fn open_op_picker(&mut self, cx: &mut Cx<'_>) {
        self.accounts.op_stage = 0;
        self.accounts.op_item.clear();
        self.op_item_key.clear();
        self.accounts.selected_op = None;
        self.picker_mode = Some(PickerMode::OnePassword);
        self.account_state = PickerState::default();
        self.op_options = vec![AccountOption {
            key: "chainargos".into(),
            label: "chainargos.1password.com".into(),
            detail: "signed in · 1Password account".into(),
        }];
        let picker = Self::account_picker().title("Choose 1Password account");
        let spec = picker.layer(cx, &self.op_options);
        cx.open_layer(ACCOUNT_PICKER, spec);
    }

    fn set_op_stage(&mut self, stage: u8) {
        self.accounts.op_stage = stage;
        self.account_state = PickerState::default();
        self.op_options = match stage {
            1 => vec![AccountOption {
                key: "engineering".into(),
                label: "Engineering".into(),
                detail: "team vault".into(),
            }],
            2 => vec![
                AccountOption {
                    key: "it_ant01".into(),
                    label: "Anthropic · Work".into(),
                    detail: "Claude credential".into(),
                },
                AccountOption {
                    key: "it_cdx01".into(),
                    label: "OpenAI · Codex Primary".into(),
                    detail: "Codex credential".into(),
                },
                AccountOption {
                    key: "it_thr01".into(),
                    label: "OpenAI · Throttled sandbox".into(),
                    detail: "Codex credential · rate limited".into(),
                },
                AccountOption {
                    key: "it_grk01".into(),
                    label: "xAI · Grok Team".into(),
                    detail: "Grok credential".into(),
                },
                AccountOption {
                    key: "it_ocg01".into(),
                    label: "OpenCode Go".into(),
                    detail: "OpenCode credential".into(),
                },
            ],
            3 => vec![AccountOption {
                key: "credential".into(),
                label: "credential".into(),
                detail: "concealed field".into(),
            }],
            _ => vec![],
        };
    }

    fn begin_launch(&mut self) {
        self.begin_launch_with(self.launch_agent, self.launch_account.clone());
    }

    fn begin_launch_with(&mut self, agent: Agent, account: Option<String>) {
        let plan = if self.world.scenario == Scenario::LaunchFailure {
            LaunchPlan::FailNetwork
        } else {
            LaunchPlan::Clean
        };
        self.launch = Some(LaunchRun::new(
            plan,
            agent,
            "jackin-payments-platform",
            crate::RunId::new(0x9c41_e2f0),
        ));
        self.launch_agent = agent;
        self.launch_account = account;
        self.refresh_cockpit_account_line(agent);
        self.cockpit.handoff = Default::default();
        self.route = Route::Cockpit;
        self.handoff_frame = None;
        self.status = Some(format!(
            "Queued {} · {}",
            self.selected_role(),
            plan_label(plan)
        ));
    }

    fn materialize_launch(&mut self) {
        let Some(run) = self.launch.as_ref() else {
            return;
        };
        let run_id = run.run_id;
        if self
            .world
            .instances
            .iter()
            .any(|instance| instance.run_id == run_id)
        {
            return;
        }

        let agent = run.agent;
        let container = run.container.clone();
        let role = self.selected_role().to_owned();
        let workspace = self.world.workspaces.first().cloned();
        let mut accounts = self
            .world
            .offer_for(agent, workspace.as_ref(), Some(&role))
            .accounts;
        if let Some(account) = self.launch_account.clone()
            && self.world.accounts.get(&account).is_some()
        {
            accounts.retain(|id| id != &account);
            accounts.insert(0, account);
        }
        let now_ms = self.world.now_ms();
        let now_secs = self.world.now_secs();
        let snapshot = crate::domain::fixtures::live_capsule();
        let mut daemon = Daemon::from_snapshot(&snapshot, &container, now_ms);
        if let Some(account) = accounts.first().cloned()
            && let Some(pane) = daemon
                .panes
                .iter_mut()
                .find(|pane| pane.proc.agent == Some(agent))
        {
            pane.proc.account = Some(account);
        }
        for pane in &mut daemon.panes {
            pane.boot_all();
        }

        let mut instance = crate::domain::fixtures::fixture_instance(
            InstanceStatus::Running,
            run_id,
            now_secs,
            snapshot,
        );
        instance.id = self.world.new_instance_id();
        instance.container = container;
        instance.workspace = workspace.as_ref().map(|workspace| workspace.id);
        instance.workdir = workspace
            .as_ref()
            .map_or_else(String::new, |workspace| workspace.workdir.clone());
        instance.role = role;
        instance.agent = agent;
        instance.created_secs = now_secs;
        instance.last_seen_secs = now_secs;
        instance.accounts = accounts;
        instance.daemon = daemon.snapshot();
        let instance_id = instance.id.clone();
        self.world.daemons.insert(instance_id, daemon);
        self.world.instances.push(instance);
        self.world.sync_arbiter();
        self.manager_rows_cache.clear();
    }

    fn capsule_input() -> TextInput<'static> {
        TextInput::new(CAPSULE_INPUT).placeholder("Type a command")
    }

    fn commit_capsule_input(&mut self) {
        let input = mem::take(&mut self.capsule_input);
        let Some(instance_id) = self
            .world
            .instances
            .iter()
            .find(|instance| instance.status == InstanceStatus::Running)
            .map(|instance| instance.id.clone())
        else {
            return;
        };
        let now_ms = self.world.now_ms();
        let observed_secs = self.world.now_secs();
        let (snapshot, last_seen_secs) = {
            let Some(daemon) = self.world.daemons.get_mut(&instance_id) else {
                return;
            };
            let workspace = daemon.workspace.clone();
            if let Some(pane_id) = daemon.focused_pane()
                && let Some(pane) = daemon.pane_mut(pane_id)
            {
                for character in input.chars() {
                    pane.type_char(character, now_ms, &workspace);
                }
                pane.commit(now_ms, &workspace);
            }
            (daemon.snapshot(), observed_secs)
        };
        if let Some(instance) = self.world.instance_mut(&instance_id) {
            instance.daemon = snapshot;
            instance.last_seen_secs = last_seen_secs;
        }
    }

    fn capsule_prefix_key(&mut self, cx: &mut Cx<'_>, key: char) -> Response<()> {
        let command = match key {
            'c' => Some(CMD_CAPSULE),
            'd' => Some(CMD_CAPSULE_DETACH),
            '%' => Some(CMD_CAPSULE_SPLIT_RIGHT),
            '"' => Some(CMD_CAPSULE_SPLIT_BELOW),
            'z' => Some(CMD_CAPSULE_ZOOM),
            'h' => Some(CMD_CAPSULE_FOCUS_LEFT),
            'u' => Some(CMD_USAGE),
            'm' => Some(CMD_MANAGER),
            _ => None,
        };
        if let Some(command) = command {
            return self
                .update_command(cx, command)
                .unwrap_or_else(Response::changed);
        }
        self.capsule_prefix = false;
        self.status = Some(format!("Unknown Capsule prefix · {key}"));
        Response::changed()
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

        let agent_picker =
            Picker::new(crate::screens::manager::AGENT_PICKER).title("Launch · choose Agent");
        let response = agent_picker.update(cx, &mut self.agent_state, &self.agent_options);
        let action = response.action_ref().copied();
        result |= response.erase();
        if cx.is_open(crate::screens::manager::AGENT_PICKER)
            && let Some(PickerAction::Chosen(key)) = action
            && let Some(option) = self
                .agent_options
                .iter()
                .find(|option| ItemKey::text(&option.key) == key)
                .cloned()
        {
            cx.close_layer(
                crate::screens::manager::AGENT_PICKER,
                Some(ActionKey::CONFIRM),
            );
            if option.blocked {
                self.status = Some(format!("{} unavailable · {}", option.label, option.detail));
            } else {
                self.begin_launch_with(option.agent, option.account);
            }
            result |= Response::changed();
        }

        let picker = self.active_account_picker();
        let response = match self.picker_mode {
            Some(PickerMode::OnePassword) => {
                picker.update(cx, &mut self.account_state, &self.op_options)
            }
            _ => picker.update(cx, &mut self.account_state, &self.account_options),
        };
        let action = response.action_ref().copied();
        result |= response.erase();
        let picker_items = match self.picker_mode {
            Some(PickerMode::OnePassword) => &self.op_options,
            _ => &self.account_options,
        };
        if cx.is_open(ACCOUNT_PICKER)
            && let Some(PickerAction::Chosen(key)) = action
            && let Some(account) = picker_items
                .iter()
                .find(|account| ItemKey::text(&account.key) == key)
                .cloned()
        {
            match self.picker_mode {
                Some(PickerMode::OnePassword) => match self.accounts.op_stage {
                    0 => {
                        self.status = Some("chainargos.1password.com".into());
                        self.set_op_stage(1);
                    }
                    1 => {
                        self.status = Some("Engineering".into());
                        self.set_op_stage(2);
                    }
                    2 => {
                        self.accounts.op_item = account.label.clone();
                        self.op_item_key = account.key.clone();
                        self.set_op_stage(3);
                        self.status = Some(account.label.clone());
                    }
                    _ => {
                        let item = self.accounts.op_item.clone();
                        let item_key = self.op_item_key.clone();
                        if let Ok(reference) = self.world.op.reference(
                            "chainargos.1password.com",
                            "Engineering",
                            &item_key,
                            "credential",
                        ) {
                            self.accounts.selected_op = Some(reference.clone());
                            self.status = Some(reference.display_path());
                        } else {
                            self.status = Some(format!("{item} · Work › credential"));
                        }
                        self.picker_mode = None;
                        cx.close_layer(ACCOUNT_PICKER, Some(ActionKey::CONFIRM));
                    }
                },
                Some(PickerMode::Capsule) => {
                    if let Some(action) = self.pending_capsule_action.take() {
                        self.apply_capsule_action(action, account);
                    }
                    self.picker_mode = None;
                    cx.close_layer(ACCOUNT_PICKER, Some(ActionKey::CONFIRM));
                }
                _ => {
                    self.status = Some(format!("Selected reference · {}", account.detail));
                    self.picker_mode = None;
                    cx.close_layer(ACCOUNT_PICKER, Some(ActionKey::CONFIRM));
                }
            }
            result |= Response::changed();
        }
        if !cx.is_open(ACCOUNT_PICKER) {
            self.picker_mode = None;
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
            if self.intro.is_done() {
                self.route = Route::Manager;
                self.world.arbiter.complete_entry(self.world.now_ms());
            } else {
                self.intro.skip();
                if self.intro.is_done() {
                    self.route = Route::Manager;
                    self.world.arbiter.complete_entry(self.world.now_ms());
                }
            }
        }
        result
    }

    fn update_manager(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.ensure_manager_rows();
        let list =
            List::new(MANAGER_LIST).update(cx, &mut self.manager.list, &self.manager_rows_cache);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        if let Some(ItemKey::Index(index)) = self.manager.list.cursor()
            && let Some(row) = self.manager_row_at(index)
        {
            self.manager.select_row(row);
        }
        match list_action {
            Some(ListAction::Activated(_)) => {
                if let Some(instance_id) = self.selected_instance_id()
                    && self
                        .world
                        .instance(&instance_id)
                        .is_some_and(|instance| instance.status.reconnectable())
                {
                    self.active_instance = Some(instance_id);
                    self.route = Route::Capsule;
                    self.capsule_interaction.focus_pane();
                    self.sync_capsule_projection();
                }
                result |= Response::changed();
            }
            Some(ListAction::Chose(_)) => {
                self.manager.set_detail_open(true);
                self.status = match self.manager.selected_row() {
                    ManagerRowKey::Workspace(id) => self
                        .world
                        .workspace(*id)
                        .map(|workspace| format!("Workspaces › {}", workspace.name)),
                    ManagerRowKey::Instance(id) => Some(format!("Instance › {id}")),
                    ManagerRowKey::CurrentDirectory | ManagerRowKey::NewWorkspace => None,
                };
                result |= Response::changed();
            }
            _ => {}
        }
        if self.manager.detail_open() {
            let detail = Button::new(crate::screens::manager::DETAIL, "Live topology").update(cx);
            result |= detail.erase();
        }
        let new_workspace = Self::new_workspace_button().update(cx);
        let new_workspace_chosen = new_workspace.activated();
        result |= new_workspace.erase();
        if new_workspace_chosen {
            self.route = Route::Prelude;
            self.prelude = crate::screens::prelude::PreludeState::default();
            result |= Response::changed();
        }
        let launch_disabled = self.launch_candidates().is_empty();
        let button = Self::launch_button(launch_disabled).update(cx);
        let chosen = button.activated();
        result |= button.erase();
        if chosen {
            self.open_agent_picker(cx);
        }
        result
    }

    fn update_accounts(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.accounts.form_open {
            if !self.accounts.started {
                let start = Self::account_start_button().update(cx);
                let chosen = start.activated();
                let mut result = start.erase();
                if chosen {
                    self.accounts.started = true;
                    cx.focus(crate::screens::accounts::NAME);
                    result |= Response::changed();
                }
                return result;
            }

            let name = Self::account_name_input().update(
                cx,
                &mut self.accounts.name_input,
                &mut self.accounts.draft_name,
            );
            let mut result = name.erase();

            // Keep the agent choice explicit in the tab order.  The current
            // account flow registers the provider for a selected agent, so a
            // public button is preferable to an implicit/raw form field.
            let agent = Self::account_agent_button().update(cx);
            result |= agent.erase();

            let provider_rows = vec![
                provider_label(Provider::Anthropic).to_owned(),
                provider_label(Provider::OpenAi).to_owned(),
                provider_label(Provider::XAi).to_owned(),
                provider_label(Provider::OpenCode).to_owned(),
            ];
            let provider = List::new(crate::screens::accounts::PROVIDER).update(
                cx,
                &mut self.accounts.provider_list,
                &provider_rows,
            );
            if let Some(ItemKey::Index(index)) = self.accounts.provider_list.cursor() {
                self.accounts.provider_index = u8::try_from(index).unwrap_or(0).min(3);
            }
            result |= provider.erase();

            let source_rows = vec![
                source_label(0).to_owned(),
                source_label(1).to_owned(),
                source_label(2).to_owned(),
            ];
            let source = List::new(crate::screens::accounts::SOURCE).update(
                cx,
                &mut self.accounts.source_list,
                &source_rows,
            );
            let source_action = source.action_ref().copied();
            if let Some(ItemKey::Index(index)) = self.accounts.source_list.cursor() {
                self.accounts.source_index = u8::try_from(index).unwrap_or(0).min(2);
            }
            result |= source.erase();
            if matches!(source_action, Some(ListAction::Activated(_))) {
                result |= Response::changed();
            }
            if self.accounts.source_index == 0 {
                let label = self.accounts.selected_op.as_ref().map_or(
                    "Choose 1Password reference…",
                    |reference| {
                        // Keep the value in a local owned string below; this
                        // label is a non-secret reference path only.
                        let _ = reference;
                        "Selected 1Password reference"
                    },
                );
                let op = Button::new(crate::screens::accounts::OP, label).update(cx);
                let chosen = op.activated();
                result |= op.erase();
                if chosen {
                    self.open_op_picker(cx);
                }
            }

            match self.accounts.source_index {
                1 => {
                    let rearm = Self::text_input_empty_commit(
                        cx,
                        crate::screens::accounts::FOLDER,
                        &self.accounts.folder_input,
                    );
                    let folder = Self::account_folder_input().update(
                        cx,
                        &mut self.accounts.folder_input,
                        &mut self.accounts.masked_input,
                    );
                    result |= folder.erase();
                    if Self::rearm_text_input(
                        rearm,
                        &mut self.accounts.folder_input,
                        &self.accounts.masked_input,
                    ) {
                        result |= Response::changed();
                    }
                }
                2 => {
                    let rearm = Self::text_input_empty_commit(
                        cx,
                        crate::screens::accounts::SECRET,
                        &self.accounts.secret_input,
                    );
                    let secret = Self::account_secret_input().update(
                        cx,
                        &mut self.accounts.secret_input,
                        &mut self.accounts.masked_input,
                    );
                    result |= secret.erase();
                    if Self::rearm_text_input(
                        rearm,
                        &mut self.accounts.secret_input,
                        &self.accounts.masked_input,
                    ) {
                        result |= Response::changed();
                    }
                }
                _ => {}
            }

            let save = Self::account_save_button().update(cx);
            let save_chosen = save.activated();
            result |= save.erase();
            if save_chosen {
                self.save_account();
                result |= Response::changed();
            }
            return result;
        }

        let rows = self.account_rows();
        let list = List::new(ACCOUNTS_LIST).update(cx, &mut self.accounts.list, &rows);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        self.accounts.selected_id = selected_account_id(&self.world, self.accounts.list.cursor());
        if matches!(list_action, Some(ListAction::Chose(_))) {
            if let Some(id) = self.accounts.selected_id.clone() {
                match self.world.accounts.set_default(&id) {
                    Ok(()) => self.status = Some("Default set for provider".into()),
                    Err(error) => self.status = Some(error),
                }
            }
            result |= Response::changed();
        }
        let add = Self::account_add_button().update(cx);
        let chosen = add.activated();
        result |= add.erase();
        if chosen {
            self.open_account_picker(cx);
        }
        result
    }

    fn save_account(&mut self) {
        let provider = register_provider(self.accounts.provider_index);
        let name = if self.accounts.draft_name.trim().is_empty() {
            "Unnamed"
        } else {
            self.accounts.draft_name.trim()
        };
        let source = match self.accounts.source_index {
            0 => match self.accounts.selected_op.clone() {
                Some(reference) => CredentialSource::OnePassword(reference),
                None => {
                    self.status = Some("Choose a 1Password reference first".into());
                    return;
                }
            },
            1 => {
                let path = self.accounts.masked_input.trim().to_owned();
                if path.is_empty() {
                    self.status = Some("Local agent folder is required".into());
                    return;
                }
                let detected = match provider::probe_folder(&path) {
                    provider::FolderProbe::Found(kind) => kind,
                    _ => DetectedKind::Unknown,
                };
                CredentialSource::LocalFolder { path, detected }
            }
            _ => {
                let value = self.accounts.masked_input.clone();
                if value.is_empty() {
                    self.status = Some("API key is required".into());
                    return;
                }
                CredentialSource::PlainApiKey {
                    fingerprint: fingerprint(&value),
                    tail: tail_of(&value),
                }
            }
        };

        let duplicate = match &source {
            CredentialSource::OnePassword(reference) => {
                self.world
                    .accounts
                    .find_duplicate(&DuplicateProbe::OpReference {
                        canonical: reference.canonical(),
                        account: reference.account.clone(),
                    })
            }
            CredentialSource::LocalFolder { path, .. } => {
                self.world.accounts.find_duplicate(&DuplicateProbe::Folder {
                    provider,
                    path: path.clone(),
                })
            }
            CredentialSource::PlainApiKey { fingerprint, .. } => self
                .world
                .accounts
                .find_duplicate(&DuplicateProbe::KeyFingerprint {
                    provider,
                    fingerprint: fingerprint.clone(),
                }),
            CredentialSource::HostEnv { .. } => None,
        };
        if let Some(account) = duplicate {
            self.status = Some(format!(
                "Already registered: this source is used by {}",
                account.title()
            ));
            return;
        }
        if self.world.accounts.name_taken(provider, name, None) {
            self.status = Some(format!("Name already used for {}", provider.short()));
            return;
        }

        let slug = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_owned();
        let id = format!("acct-{}-{}", provider_slug(provider), slug);
        let plain =
            (self.accounts.source_index == 2).then_some(self.accounts.masked_input.as_str());
        let outcome = provider::validate(
            provider,
            &source,
            plain,
            &self.world.op,
            self.world.now_secs(),
        );
        let mut account = Account::registered(&id, name, provider, source);
        account.identity = outcome.identity;
        account.confidence = outcome.confidence;
        account.lifecycle = outcome.lifecycle;
        account.issue = outcome.issue;
        account.validation = outcome
            .level
            .map(crate::domain::account::ValidationState::Valid)
            .unwrap_or(crate::domain::account::ValidationState::NeverValidated);
        if let Some(usage) = outcome.usage {
            account.usage = usage;
        }
        if provider == Provider::XAi {
            account = account.with_endpoint("Grok Team", "https://api.x.ai");
        }
        let title = account.title();
        let issue = account.issue.as_ref().map(|issue| issue.message.clone());
        self.world.accounts.insert(account);
        let selected_id = id.clone();
        self.accounts.selected_id = Some(id);
        if let Some(index) = self
            .world
            .accounts
            .sorted()
            .iter()
            .position(|account| account.id == selected_id)
            .map(|index| index.saturating_add(1))
        {
            self.accounts.list.set_cursor(index, ItemKey::index(index));
        }
        self.account_options = self
            .world
            .accounts
            .sorted()
            .into_iter()
            .map(AccountOption::from)
            .collect();
        self.accounts.form_open = false;
        self.accounts.started = false;
        self.accounts.masked_input.clear();
        self.accounts.secret_input = junie_tui::TextInputState::default();
        self.accounts.folder_input = junie_tui::TextInputState::default();
        self.status = Some(match issue {
            Some(issue) => format!("Saved {title} · {issue}"),
            None => format!("Saved {title}"),
        });
    }

    fn update_settings(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let button = Self::settings_trust_button(self.trusted).update(cx);
        let chosen = button.activated();
        let mut result = button.erase();
        if chosen {
            self.trusted = !self.trusted;
            if self.settings.dirty {
                self.settings.mark_dirty();
            } else {
                self.settings.begin_draft();
            }
            result |= Response::changed();
        }
        let save = Button::new(crate::screens::settings::SAVE, "Save settings")
            .variant(Variant::PRIMARY)
            .update(cx);
        let save_chosen = save.activated();
        result |= save.erase();
        let confirm = Self::settings_save_confirm_button().update(cx);
        let confirm_chosen = confirm.activated();
        result |= confirm.erase();
        if save_chosen {
            if self.settings.dirty {
                cx.focus(SETTINGS_SAVE_CONFIRM);
                self.status = Some("Save settings · choose a confirmation action".into());
                result |= Response::changed();
            } else {
                self.status = Some("No settings changes".into());
            }
        }
        if confirm_chosen && self.settings.dirty {
            let keep = self.settings.attempt_save(self.world.refresh_fails);
            if keep {
                self.status = self.settings.save_error.clone();
            } else {
                if let Some(trust) = self.world.global.trust.first_mut() {
                    trust.trusted = self.trusted;
                }
                self.status = Some("Settings saved".into());
                self.route = Route::Manager;
            }
            result |= Response::changed();
        }
        result
    }

    fn update_prelude(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let continue_button = Self::prelude_continue_button().update(cx);
        let chosen = continue_button.activated();
        let mut result = continue_button.erase();
        if chosen {
            let previous_step = self.prelude.step();
            self.prelude.advance_flow();
            if previous_step >= 5 {
                if self.prelude.duplicate() {
                    self.status = Some(format!(
                        "A workspace named {} already exists",
                        self.prelude.name()
                    ));
                } else {
                    self.route = Route::Editor;
                    self.editor = EditorState::default();
                    self.editor.pending.name = self.prelude.name().into();
                    self.editor.pending.workdir =
                        self.prelude.source().replace("~/", "/Users/alexey/");
                    self.editor.pending.mounts = vec![crate::domain::workspace::Mount::host(
                        &self.editor.pending.workdir,
                        &self.editor.pending.workdir,
                    )];
                }
            }
            result |= Response::changed();
        }
        result
    }

    fn update_editor(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.editor.env_form_open {
            let key_rearm = Self::text_input_empty_commit(
                cx,
                crate::screens::editor::ENV_KEY,
                &self.editor.env_key_input,
            );
            let key = Self::editor_env_key_input().update(
                cx,
                &mut self.editor.env_key_input,
                &mut self.editor.env_key,
            );
            let key_cancelled = key
                .action_ref()
                .is_some_and(|action| *action == TextAction::Cancelled);
            let mut result = key.erase();
            if Self::rearm_text_input(
                key_rearm,
                &mut self.editor.env_key_input,
                &self.editor.env_key,
            ) {
                result |= Response::changed();
            }

            let source = Self::editor_env_source_button().update(cx);
            result |= source.erase();

            let value_rearm = Self::text_input_empty_commit(
                cx,
                crate::screens::editor::ENV_VALUE,
                &self.editor.env_value_input,
            );
            let value = Self::editor_env_value_input().update(
                cx,
                &mut self.editor.env_value_input,
                &mut self.editor.env_value,
            );
            let value_cancelled = value
                .action_ref()
                .is_some_and(|action| *action == TextAction::Cancelled);
            result |= value.erase();
            if key_cancelled || value_cancelled {
                self.editor.discard_env_value();
            }
            if Self::rearm_text_input(
                value_rearm,
                &mut self.editor.env_value_input,
                &self.editor.env_value,
            ) {
                result |= Response::changed();
            }

            let save = Self::editor_save_button().update(cx);
            let save_chosen = save.activated();
            result |= save.erase();
            if save_chosen {
                let key = self.editor.env_key.trim().to_owned();
                if let Some(error) = env_key_error(&key) {
                    self.status = Some(error);
                } else {
                    let status = format!("Added environment variable {key}");
                    let value = self.editor.take_env_value();
                    self.editor.pending.env.push(EnvVar {
                        key,
                        value: EnvValue::Plain(value),
                    });
                    self.editor.clear_env_form();
                    self.editor.dirty = true;
                    self.status = Some(status);
                    result |= Response::changed();
                }
            }
            return result;
        }
        let mut result = Response::ignored();
        match self.editor.tab {
            EditorTab::Mounts => {
                let mount = Self::editor_mount_button().update(cx);
                let chosen = mount.activated();
                result |= mount.erase();
                if chosen {
                    if let Some(mount) = self.editor.pending.mounts.first_mut() {
                        mount.readonly = true;
                        mount.isolation = crate::domain::workspace::Isolation::Clone;
                    }
                    self.editor.mark_dirty();
                    self.status = Some("Mounts · 1 modified · readonly · worktree".into());
                    result |= Response::changed();
                }
            }
            EditorTab::Roles => {
                let role = Self::editor_role_button().update(cx);
                let chosen = role.activated();
                result |= role.erase();
                if chosen {
                    self.editor.pending.roles.default = Some("chainargos/the-architect".into());
                    self.editor.mark_dirty();
                    self.status = Some("Default role ★ chainargos/the-architect".into());
                    result |= Response::changed();
                }
                let load = Self::editor_role_load_button().update(cx);
                let load_chosen = load.activated();
                result |= load.erase();
                if load_chosen {
                    self.status = Some("Add role override · type a role name".into());
                    result |= Response::changed();
                }
            }
            EditorTab::Accounts => {
                let rows = self.editor_account_rows();
                let list =
                    List::new(EDITOR_ACCOUNTS_LIST).update(cx, &mut self.editor_accounts, &rows);
                let action = list.action_ref().copied();
                result |= list.erase();
                if let Some(ItemKey::Index(index)) = self.editor_accounts.cursor()
                    && let Some(id) = self.editor_account_id(index)
                    && matches!(action, Some(ListAction::Activated(_)))
                {
                    match self
                        .editor
                        .pending
                        .toggle_account(id.clone(), &self.world.accounts)
                    {
                        Ok(active) => {
                            self.editor.mark_dirty();
                            self.status = Some(if active {
                                format!("{id} · active for this Workspace")
                            } else {
                                format!("{id} · off for this Workspace")
                            });
                            result |= Response::changed();
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
                if matches!(action, Some(ListAction::Chose(_)))
                    && let Some(ItemKey::Index(index)) = self.editor_accounts.cursor()
                    && let Some(id) = self.editor_account_id(index)
                {
                    match self
                        .editor
                        .pending
                        .prefer_account(id.clone(), &self.world.accounts)
                    {
                        Ok(()) => {
                            self.editor.mark_dirty();
                            self.status = Some(format!("Preferred for {}", id));
                            result |= Response::changed();
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
            }
            EditorTab::Environments | EditorTab::General => {}
        }

        let save = Self::editor_save_button().update(cx);
        let save_chosen = save.activated();
        result |= save.erase();
        let confirm = Self::editor_save_confirm_button().update(cx);
        let confirm_chosen = confirm.activated();
        result |= confirm.erase();
        if save_chosen {
            if self.editor.open_preview() {
                cx.focus(EDITOR_SAVE_CONFIRM);
                self.status = Some("Save workspace · preview changes before commit".into());
                result |= Response::changed();
            }
        }
        if confirm_chosen && self.editor.preview_open {
            self.editor.close_preview();
            self.editor.mark_saved();
            self.world.saved = true;
            let id = self
                .world
                .workspaces
                .first()
                .map_or(1, |workspace| workspace.id);
            self.world
                .schedule(200, crate::sim::world::Msg::WorkspaceSaved { id, ok: true });
            self.status = Some("Saving workspace…".into());
            result |= Response::changed();
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
                            "Launch failed · {} · {} · another instance is still running",
                            failure.stage.label(),
                            failure.summary
                        ));
                    }
                }
                LaunchEvent::Ready => {
                    self.materialize_launch();
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
        if !self.capsule_input_state.is_editing() {
            cx.focus(CAPSULE_INPUT);
        }
        let input = Self::capsule_input().update(
            cx,
            &mut self.capsule_input_state,
            &mut self.capsule_input,
        );
        let committed = input
            .action_ref()
            .is_some_and(|action| *action == TextAction::Committed);
        let mut result = input.erase();
        let prefix_key = self
            .capsule_prefix
            .then(|| self.capsule_input_state.draft_text())
            .flatten();
        if let Some(key) = prefix_key.and_then(|draft| draft.chars().next()) {
            self.capsule_input.clear();
            self.capsule_input_state = TextInputState::default();
            result |= self.capsule_prefix_key(cx, key);
        }
        if committed {
            if self.exit_choice.is_some() {
                if let Some(response) = self.update_command(cx, CMD_EXIT_CONFIRM) {
                    result |= response;
                }
            } else {
                self.commit_capsule_input();
            }
            result |= Response::changed();
        }
        let tabs = capsule_tabs();
        result
            | Tabs::new(CAPSULE_TABS)
                .update(cx, &mut self.tabs_state, &tabs)
                .erase()
    }

    fn update_route(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        match self.route {
            Route::Intro => self.update_intro(cx),
            Route::Manager => self.update_manager(cx),
            Route::Prelude => self.update_prelude(cx),
            Route::Editor => self.update_editor(cx),
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
            CMD_MANAGER
                if self.route == Route::Editor
                    && self.editor.tab == crate::screens::editor::Tab::Environments =>
            {
                self.status = Some("Plain values stay masked · a add variable".into());
                Some(Response::changed())
            }
            CMD_MANAGER => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.status = Some("Detached from Capsule".into());
                    self.route = Route::Manager;
                } else {
                    if self.route == Route::Usage {
                        self.accounts.selected_id = self.usage.manage_target().map(str::to_owned);
                        self.route = Route::Accounts;
                        if let Some(id) = self.accounts.selected_id.as_deref()
                            && let Some(account) = self.world.accounts.get(id)
                        {
                            self.status = Some(format!("Accounts › {}", account.title()));
                        }
                    } else {
                        self.route = Route::Manager;
                    }
                }
                Some(Response::changed())
            }
            CMD_ACCOUNTS
                if self.route == Route::Editor
                    && self.editor.tab == crate::screens::editor::Tab::Environments =>
            {
                self.editor.open_env_form();
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_ACCOUNTS => {
                if self.route == Route::Accounts {
                    self.accounts.open_new();
                    self.op_item_key.clear();
                    cx.focus(crate::screens::accounts::START);
                } else {
                    self.route = Route::Accounts;
                }
                Some(Response::changed())
            }
            CMD_ACCOUNT_REFRESH if self.route == Route::Accounts && !self.accounts.form_open => {
                if let Some(id) = self.accounts.selected_id.clone() {
                    self.accounts.pending_refresh = Some(id.clone());
                    self.status = Some("Refreshing account…".into());
                    self.world.schedule(
                        1_000,
                        crate::sim::world::Msg::AccountRefreshed { account: id },
                    );
                }
                Some(Response::changed())
            }
            CMD_ACCOUNT_VALIDATE if self.route == Route::Accounts && !self.accounts.form_open => {
                if let Some(id) = self.accounts.selected_id.clone()
                    && let Some(account) = self.world.accounts.get(&id).cloned()
                {
                    let outcome = provider::validate(
                        account.provider,
                        &account.source,
                        None,
                        &self.world.op,
                        self.world.now_secs(),
                    );
                    if let Some(account) = self.world.accounts.get_mut(&id) {
                        account.identity = outcome.identity;
                        account.confidence = outcome.confidence;
                        account.lifecycle = outcome.lifecycle;
                        account.issue = outcome.issue;
                        account.usage = outcome.usage.unwrap_or_else(|| account.usage.clone());
                    }
                    self.status = Some("Validation fingerprint matches configured source".into());
                }
                Some(Response::changed())
            }
            CMD_ACCOUNT_REMOVE if self.route == Route::Accounts && !self.accounts.form_open => {
                if let Some(id) = self.accounts.selected_id.clone()
                    && let Some(account) = self.world.accounts.get(&id)
                {
                    self.accounts.remove_confirmation = Some(id);
                    self.status = Some(format!("Remove account {}?", account.display_name));
                }
                Some(Response::changed())
            }
            CMD_ACCOUNT_DEFAULT if self.route == Route::Accounts && !self.accounts.form_open => {
                if let Some(id) = self.accounts.selected_id.clone() {
                    match self.world.accounts.set_default(&id) {
                        Ok(()) => self.status = Some("Default set for provider".into()),
                        Err(error) => self.status = Some(error),
                    }
                }
                Some(Response::changed())
            }
            CMD_ACCOUNT_HELP if self.route == Route::Accounts => {
                self.status =
                    Some("Credential sources · 1Password · Local agent folder · API key".into());
                Some(Response::changed())
            }
            CMD_ACCOUNT_HELP if self.route == Route::Usage => {
                self.status = Some("Reading meters · usage is read-only".into());
                Some(Response::changed())
            }
            CMD_USAGE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.capsule_usage = true;
                    self.status = Some("Usage".into());
                } else {
                    if self.usage.selected().is_none() {
                        let selected = self
                            .world
                            .accounts
                            .sorted()
                            .first()
                            .map(|account| account.id.clone());
                        self.usage.select(selected);
                    }
                    self.route = Route::Usage;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS => {
                self.route = Route::Settings;
                self.settings.clear_error();
                Some(Response::changed())
            }
            CMD_SETTINGS_TRUST_KEY if self.route == Route::Settings => {
                cx.focus(SETTINGS_TRUST);
                Some(Response::changed())
            }
            CMD_CAPSULE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.status = Some("New tab · Account for Claude Code".into());
                    self.open_capsule_account_picker(cx, CapsuleAction::NewTab);
                } else if self.route == Route::Manager {
                    self.route = Route::Accounts;
                } else {
                    self.route = Route::Capsule;
                }
                Some(Response::changed())
            }
            CMD_MANAGER_EXPAND if self.route == Route::Manager => {
                if let Some(workspace) = self.world.workspaces.first() {
                    self.manager.toggle(workspace.id);
                    self.manager.set_detail_open(true);
                    self.ensure_manager_rows();
                }
                Some(Response::changed())
            }
            CMD_EDITOR_OPEN if self.route == Route::Manager => {
                self.route = Route::Editor;
                if let Some(workspace) = self.world.workspaces.first() {
                    self.editor.load_workspace(workspace);
                } else {
                    self.editor = EditorState::default();
                }
                self.editor.select_alias(1);
                self.editor_accounts = ListState::default();
                Some(Response::changed())
            }
            CMD_CAPSULE_PREFIX if self.route == Route::Capsule => {
                self.capsule_prefix = true;
                self.status = Some("prefix… New tab · Split · Copy · Detach".into());
                Some(Response::changed())
            }
            CMD_CAPSULE_DETACH if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                self.pending_capsule_action = None;
                self.status = Some("Detached from Capsule".into());
                self.route = Route::Manager;
                cx.focus(MANAGER_LIST);
                Some(Response::changed())
            }
            CMD_CAPSULE_SPLIT_RIGHT if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                self.status = Some("Split right · Account for Claude Code".into());
                self.open_capsule_account_picker(cx, CapsuleAction::Split(SplitDir::Horizontal));
                Some(Response::changed())
            }
            CMD_CAPSULE_SPLIT_BELOW if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                self.status = Some("Split below · Account for Claude Code".into());
                self.open_capsule_account_picker(cx, CapsuleAction::Split(SplitDir::Vertical));
                Some(Response::changed())
            }
            CMD_CAPSULE_ZOOM if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                if let Some(instance_id) = self
                    .world
                    .instances
                    .iter()
                    .find(|instance| instance.status == InstanceStatus::Running)
                    .map(|instance| instance.id.clone())
                    && let Some(daemon) = self.world.daemons.get_mut(&instance_id)
                    && let Some(tab) = daemon.active_tab_mut()
                {
                    tab.zoomed = if tab.zoomed == Some(tab.focused) {
                        None
                    } else {
                        Some(tab.focused)
                    };
                    self.capsule.zoomed = tab.zoomed.is_some();
                    self.status = Some(if tab.zoomed.is_some() {
                        "zoom · focused pane".into()
                    } else {
                        "zoom off".into()
                    });
                    let snapshot = daemon.snapshot();
                    if let Some(instance) = self.world.instance_mut(&instance_id) {
                        instance.daemon = snapshot;
                    }
                }
                Some(Response::changed())
            }
            CMD_CAPSULE_FOCUS_LEFT if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                if let Some(instance_id) = self
                    .world
                    .instances
                    .iter()
                    .find(|instance| instance.status == InstanceStatus::Running)
                    .map(|instance| instance.id.clone())
                    && let Some(daemon) = self.world.daemons.get_mut(&instance_id)
                    && let Some(tab) = daemon.active_tab_mut()
                {
                    let leaves = tab.leaves();
                    if let Some(position) = leaves.iter().position(|id| *id == tab.focused) {
                        tab.focused = leaves
                            .get(position.saturating_sub(1))
                            .copied()
                            .unwrap_or(tab.focused);
                        self.capsule.selected_pane = tab.focused;
                        self.status = Some("focus left".into());
                        let snapshot = daemon.snapshot();
                        if let Some(instance) = self.world.instance_mut(&instance_id) {
                            instance.daemon = snapshot;
                        }
                    }
                }
                Some(Response::changed())
            }
            CMD_CAPSULE_PALETTE if self.route == Route::Capsule => {
                self.capsule_prefix = false;
                self.status = Some("Command palette · type an action".into());
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.route == Route::Outro => {
                if let Some(outro) = &mut self.outro {
                    outro.skip();
                    if outro.is_done() {
                        self.quit = true;
                        cx.quit();
                    }
                    Some(Response::changed())
                } else {
                    None
                }
            }
            CMD_EXIT_DIALOG if self.route == Route::Capsule => {
                self.exit_choice = Some(0);
                self.status = Some("Unsaved work · Stay inside · Exit & keep · Cancel".into());
                Some(Response::changed())
            }
            CMD_PRELUDE_DOWN if self.route == Route::Capsule => {
                if let Some(choice) = &mut self.exit_choice {
                    *choice = (*choice + 1).min(2);
                    self.status = Some(match *choice {
                        0 => "Unsaved work · Stay inside · Exit & keep · Cancel".into(),
                        1 => "Unsaved work · Stay inside · Exit & keep · Cancel [Exit]".into(),
                        _ => "Unsaved work · Stay inside · Exit & keep · Cancel [Cancel]".into(),
                    });
                    Some(Response::changed())
                } else {
                    None
                }
            }
            CMD_EXIT_CONFIRM if self.route == Route::Capsule => {
                if self.exit_choice.is_some_and(|choice| choice >= 2) {
                    self.exit_choice = None;
                    self.status = None;
                    if self.world.running_count() > 1 {
                        if let Some(instance) = self
                            .world
                            .instances
                            .iter_mut()
                            .find(|instance| instance.status == InstanceStatus::Running)
                        {
                            instance.status = InstanceStatus::CleanExited;
                        }
                        self.manager_rows_cache.clear();
                        self.route = Route::Manager;
                        self.status =
                            Some("Still inside the Construct · another instance is running".into());
                    } else if self.world.scenario == Scenario::OutroLast {
                        self.outro = Some(OutroState::new(self.motion, Some(8_040), 0));
                        self.route = Route::Outro;
                    } else {
                        self.route = Route::Manager;
                    }
                    Some(Response::changed())
                } else {
                    None
                }
            }
            CMD_EXIT_CONFIRM if self.route == Route::Intro => {
                if self.intro.is_done() {
                    self.route = Route::Manager;
                    self.world.arbiter.complete_entry(self.world.now_ms());
                } else {
                    self.intro.skip();
                    if self.intro.is_done() {
                        self.route = Route::Manager;
                        self.world.arbiter.complete_entry(self.world.now_ms());
                    }
                }
                Some(Response::changed())
            }
            CMD_PRELUDE_BACKSPACE if self.route == Route::Prelude => {
                self.prelude.source_back();
                Some(Response::changed())
            }
            CMD_PRELUDE_DOWN if self.route == Route::Prelude => {
                self.prelude.move_selection(true);
                Some(Response::changed())
            }
            CMD_PRELUDE_SPACE if self.route == Route::Prelude => {
                if self.prelude.step() == 1 {
                    self.prelude.choose_source();
                    cx.focus(crate::screens::prelude::CONTINUE);
                }
                Some(Response::changed())
            }
            CMD_NEW_WORKSPACE if self.route == Route::Manager => {
                self.route = Route::Prelude;
                self.prelude = PreludeState::default();
                Some(Response::changed())
            }
            CMD_EDITOR_NEXT if self.route == Route::Editor => {
                let before = self.editor.tab;
                self.editor.next_tab();
                self.status = Some(format!("editor-tab {before:?}->{:?}", self.editor.tab));
                match self.editor.tab {
                    EditorTab::Mounts => cx.focus(EDITOR_MOUNT_EDIT),
                    EditorTab::Roles => cx.focus(EDITOR_ROLE_EDIT),
                    EditorTab::Accounts => cx.focus(EDITOR_ACCOUNTS_LIST),
                    EditorTab::Environments | EditorTab::General => {}
                }
                Some(Response::changed())
            }
            CMD_EDITOR_ENV if self.route == Route::Editor => {
                self.editor.select_alias(4);
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_EDITOR_PREVIOUS if self.route == Route::Editor => {
                self.editor.previous_tab();
                match self.editor.tab {
                    EditorTab::Mounts => cx.focus(EDITOR_MOUNT_EDIT),
                    EditorTab::Roles => cx.focus(EDITOR_ROLE_EDIT),
                    EditorTab::Accounts => cx.focus(EDITOR_ACCOUNTS_LIST),
                    EditorTab::Environments | EditorTab::General => {}
                }
                Some(Response::changed())
            }
            CMD_EDITOR_ROLES if self.route == Route::Editor => {
                self.editor.select_alias(3);
                cx.focus(EDITOR_ROLE_EDIT);
                Some(Response::changed())
            }
            CMD_EDITOR_ACCOUNTS if self.route == Route::Editor => {
                self.editor.select_alias(5);
                cx.focus(EDITOR_ACCOUNTS_LIST);
                Some(Response::changed())
            }
            CMD_EDITOR_PREFER
                if self.route == Route::Editor && self.editor.tab == EditorTab::Accounts =>
            {
                if let Some(ItemKey::Index(index)) = self.editor_accounts.cursor()
                    && let Some(id) = self.editor_account_id(index)
                {
                    match self
                        .editor
                        .pending
                        .prefer_account(id.clone(), &self.world.accounts)
                    {
                        Ok(()) => {
                            self.editor.mark_dirty();
                            self.status = Some(format!("Preferred for {id}"));
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
                Some(Response::changed())
            }
            CMD_SAVE if self.route == Route::Editor => {
                self.editor.mark_dirty();
                self.editor.open_preview();
                cx.focus(EDITOR_SAVE_CONFIRM);
                self.status = Some("Save workspace · preview changes before commit".into());
                Some(Response::changed())
            }
            CMD_SAVE if self.route == Route::Settings => {
                if !self.settings.dirty {
                    self.settings.begin_draft();
                }
                cx.focus(SETTINGS_SAVE_CONFIRM);
                self.status = Some("Save settings · choose a confirmation action".into());
                Some(Response::changed())
            }
            CMD_USAGE_NEXT if self.route == Route::Usage => {
                self.usage.next_tab();
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
                    if ok && self.route == Route::Editor {
                        if let Some(workspace) = self
                            .world
                            .workspaces
                            .iter_mut()
                            .find(|workspace| workspace.id == id)
                        {
                            workspace.env = mem::take(&mut self.editor.pending.env);
                        } else {
                            let mut workspace = crate::domain::workspace::Workspace::new(
                                id,
                                self.prelude.name(),
                                "/Users/alexey/src/new-workspace",
                            );
                            workspace.env = mem::take(&mut self.editor.pending.env);
                            self.world.workspaces.push(workspace);
                        }
                        self.manager_rows_cache.clear();
                        self.route = Route::Manager;
                    }
                }
                crate::sim::world::Msg::Refreshed { ok } => {
                    self.status = Some(if ok {
                        "Refresh complete".into()
                    } else {
                        "Refresh failed; last good data retained".into()
                    });
                }
                crate::sim::world::Msg::AccountRefreshed { account } => {
                    self.accounts.pending_refresh = None;
                    if self.world.refresh_fails {
                        self.status = Some(
                            "Refresh failed · broker unreachable · last good data retained".into(),
                        );
                    } else if let Some(entry) = self.world.accounts.get(&account) {
                        self.status =
                            Some(format!("Refreshed {} · still rate limited", entry.title()));
                    } else {
                        self.status = Some(format!("Account {account} refreshed"));
                    }
                }
            }
        }

        match self.route {
            Route::Intro => {
                if self.intro.advance_tick() && self.intro.is_done() {
                    self.route = Route::Manager;
                    self.world.arbiter.complete_entry(self.world.now_ms());
                    result |= Response::changed();
                }
                cx.request_repaint_after(Duration::from_millis(TICK_MS));
            }
            Route::Outro => {
                if let Some(outro) = &mut self.outro
                    && outro.advance_tick()
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
        let brand = format!("jackin❯  {message}");
        ui.paint_str(
            Rect {
                height: area.height.min(1),
                ..area
            },
            &brand,
            style,
        );
        if self.motion == Motion::Reduced {
            ui.paint_str(
                Rect::new(area.x, area.y.saturating_add(2), area.width, 1),
                "Enter Continue",
                style,
            );
        }
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
        let step = self.prelude.step();
        let source = self.prelude.source();
        let name = self.prelude.name();
        let lines = match step {
            1 => vec![
                format!("Create workspace · step 1 of 5 · Source"),
                format!("Source · {source}"),
                "customer-portal/".to_owned(),
                "data-pipeline/".to_owned(),
                "payments-platform/".to_owned(),
                format!("Selected source · {}", self.prelude.selection()),
            ],
            2 => vec![
                "Create workspace · step 2 of 5 · Destination".to_owned(),
                format!("Same path   {}", source.replace("~/", "/Users/alexey/")),
                "✓ Source".to_owned(),
                "Destination · choose a Workspace name".to_owned(),
            ],
            4 => vec![
                "Create workspace · step 4 of 5 · Working directory".to_owned(),
                format!("Source · {source}"),
                "destination · inherited from source".to_owned(),
                "Mounts and environment · review before save".to_owned(),
            ],
            5 => vec![
                "Create workspace · step 5 of 5 · Name".to_owned(),
                format!("Workspace · {name}"),
                format!("Source · {source}"),
                "Review and save when ready".to_owned(),
            ],
            _ => vec![
                format!("Create workspace · step {step} of 5"),
                format!("Source · {source}"),
                "Destination · choose a Workspace name".to_owned(),
                "Mounts and environment · review before save".to_owned(),
                name.to_owned(),
            ],
        };
        paint_lines(ui, area, &lines);
        Self::prelude_continue_button().draw(
            ui,
            Rect::new(area.x, area.bottom().saturating_sub(1), 14, 1),
        );
    }

    fn draw_editor(&self, ui: &mut Ui<'_>, area: Rect) {
        let workspace = self
            .world
            .workspaces
            .first()
            .map(|workspace| workspace.name.as_str())
            .unwrap_or("new workspace");
        let tab = match self.editor.tab {
            crate::screens::editor::Tab::General => "General",
            crate::screens::editor::Tab::Mounts => "Mounts",
            crate::screens::editor::Tab::Roles => "Roles",
            crate::screens::editor::Tab::Environments => "Environments",
            crate::screens::editor::Tab::Accounts => "Accounts",
        };
        if self.editor.env_form_open {
            paint_lines(
                ui,
                area,
                &["New workspace environment key", "Key · source · value"],
            );
            Self::editor_env_key_input()
                .value(&self.editor.env_key)
                .draw(
                    ui,
                    Rect::new(area.x, area.y.saturating_add(3), area.width, 1),
                    &self.editor.env_key_input,
                );
            Self::editor_env_source_button().draw(
                ui,
                Rect::new(area.x, area.y.saturating_add(4), area.width.min(20), 1),
            );
            Self::editor_env_value_input()
                .value(&self.editor.env_value)
                .draw(
                    ui,
                    Rect::new(area.x, area.y.saturating_add(5), area.width, 1),
                    &self.editor.env_value_input,
                );
            Self::editor_save_button().draw(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
            );
            return;
        }
        if self.editor.tab == crate::screens::editor::Tab::Environments {
            let mut lines = vec![format!(
                "{}{} · edit · {tab}",
                if self.editor.dirty {
                    "• 1 change · "
                } else {
                    ""
                },
                workspace
            )];
            for env in &self.editor.pending.env {
                let (value, source): (String, &str) = match &env.value {
                    EnvValue::Plain(value) => (mask(value), "plain"),
                    EnvValue::OnePassword(reference) => (reference.display_path(), "1Password"),
                    EnvValue::HostEnv(host) => (host.clone(), "host env"),
                };
                lines.push(format!("{} · {value} · {source}", env.key));
            }
            lines.push("m plain values stay masked · a add variable".to_owned());
            paint_lines(ui, area, &lines);
            Self::editor_save_button().draw(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
            );
            return;
        }
        if self.editor.tab == EditorTab::Mounts {
            let mount = self.editor.pending.mounts.first();
            let mount_line = mount.map_or_else(
                || "Mounts · none".to_owned(),
                |mount| {
                    format!(
                        "Mounts {} · {} · {}",
                        if mount.readonly { "•" } else { "" },
                        mount.mode_label(),
                        if matches!(mount.isolation, crate::domain::workspace::Isolation::Clone) {
                            "worktree"
                        } else {
                            "shared"
                        }
                    )
                },
            );
            paint_lines(
                ui,
                area,
                &[
                    workspace.to_owned(),
                    mount_line,
                    "Mount source · workspace".into(),
                    if self.editor.dirty {
                        "1 modified".into()
                    } else {
                        String::new()
                    },
                ],
            );
            Self::editor_mount_button()
                .draw(ui, Rect::new(area.x, area.y.saturating_add(4), 18, 1));
            if self.editor.preview_open {
                Self::editor_save_confirm_button().draw(
                    ui,
                    Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
                );
            } else {
                Self::editor_save_button().draw(
                    ui,
                    Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
                );
            }
            return;
        }
        if self.editor.tab == EditorTab::Roles {
            let default = self
                .editor
                .pending
                .roles
                .default
                .as_deref()
                .unwrap_or("none");
            paint_lines(
                ui,
                area,
                &[
                    format!("{workspace} › edit · Roles"),
                    format!("Default role ★ {default}"),
                    format!(
                        "Role overrides · {} configured · {} in registry",
                        self.editor.pending.configured_role_count(),
                        self.world.roles.len()
                    ),
                ],
            );
            Self::editor_role_button().draw(ui, Rect::new(area.x, area.y.saturating_add(4), 20, 1));
            Self::editor_role_load_button()
                .draw(ui, Rect::new(area.x, area.y.saturating_add(5), 18, 1));
            return;
        }
        if self.editor.tab == EditorTab::Accounts {
            paint_lines(
                ui,
                Rect { height: 2, ..area },
                &[format!("{workspace} › edit · Active accounts")],
            );
            let rows = self.editor_account_rows();
            List::new(EDITOR_ACCOUNTS_LIST).draw(
                ui,
                Rect {
                    y: area.y.saturating_add(2),
                    height: area.height.saturating_sub(4),
                    ..area
                },
                &self.editor_accounts,
                &rows,
            );
            Self::editor_save_button().draw(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
            );
            return;
        }
        let lines = [
            format!("{workspace} › edit · {tab}"),
            "Mounts · inherited defaults".to_owned(),
            "Environments · references only; values stay masked".to_owned(),
            format!("Roles · {} configured", self.world.roles.len()),
            format!(
                "{}Save workspace · Ctrl+S",
                if self.editor.dirty {
                    "• 1 change · "
                } else {
                    ""
                }
            ),
        ];
        paint_lines(ui, area, &lines);
        Self::editor_save_button().draw(
            ui,
            Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
        );
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
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(5),
            ..area
        };
        paint_lines(
            ui,
            Rect { height: 2, ..area },
            std::slice::from_ref(&self.manager_header),
        );
        List::new(MANAGER_LIST).draw(ui, list_area, &self.manager.list, &self.manager_rows_cache);
        if self.manager.detail_open() {
            ui.paint_str(
                Rect::new(area.x, area.y.saturating_add(1), area.width, 1),
                "Live topology · Workspaces › infra-control-plane",
                ui.surface_style(),
            );
        }
        Self::new_workspace_button().draw(
            ui,
            Rect {
                y: area.bottom().saturating_sub(2),
                width: area.width.min(24),
                height: 1,
                ..area
            },
        );
        Self::launch_button(self.launch_candidates().is_empty()).draw(
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
        if self.accounts.form_open {
            if !self.accounts.started {
                paint_lines(
                    ui,
                    area,
                    &[
                        "New account",
                        "Register a provider account without storing secret material.",
                    ],
                );
                Self::account_start_button()
                    .draw(ui, Rect::new(area.x, area.y.saturating_add(3), 18, 1));
                return;
            }
            paint_lines(
                ui,
                area,
                &[
                    "New account · register",
                    "Name · provider · credential source",
                ],
            );
            Self::account_name_input()
                .value(&self.accounts.draft_name)
                .draw(
                    ui,
                    Rect::new(area.x, area.y.saturating_add(3), area.width, 1),
                    &self.accounts.name_input,
                );
            ui.paint_str(
                Rect::new(area.x, area.y.saturating_add(4), area.width, 1),
                "Agent · Claude Code",
                ui.surface_style(),
            );
            Self::account_agent_button().draw(
                ui,
                Rect::new(area.x, area.y.saturating_add(5), area.width.min(28), 1),
            );
            List::new(crate::screens::accounts::PROVIDER).draw(
                ui,
                Rect::new(area.x, area.y.saturating_add(6), area.width.min(34), 4),
                &self.accounts.provider_list,
                &[
                    provider_label(Provider::Anthropic),
                    provider_label(Provider::OpenAi),
                    provider_label(Provider::XAi),
                    provider_label(Provider::OpenCode),
                ],
            );
            let source_y = area.y.saturating_add(11);
            List::new(crate::screens::accounts::SOURCE).draw(
                ui,
                Rect::new(area.x, source_y, area.width.min(34), 3),
                &self.accounts.source_list,
                &[source_label(0), source_label(1), source_label(2)],
            );
            let input_y = source_y.saturating_add(4);
            match self.accounts.source_index {
                0 => {
                    Button::new(
                        crate::screens::accounts::OP,
                        self.accounts.selected_op.as_ref().map_or(
                            "Choose 1Password reference…",
                            |_| "Selected 1Password reference",
                        ),
                    )
                    .draw(ui, Rect::new(area.x, input_y, area.width.min(38), 1));
                }
                1 => {
                    Self::account_folder_input()
                        .value(&self.accounts.masked_input)
                        .draw(
                            ui,
                            Rect::new(area.x, input_y, area.width, 1),
                            &self.accounts.folder_input,
                        );
                }
                2 => {
                    Self::account_secret_input()
                        .value(&self.accounts.masked_input)
                        .draw(
                            ui,
                            Rect::new(area.x, input_y, area.width, 1),
                            &self.accounts.secret_input,
                        );
                    if !self.accounts.masked_input.is_empty() {
                        let tail = crate::domain::account::tail_of(&self.accounts.masked_input);
                        ui.paint_str(
                            Rect::new(area.x, input_y.saturating_add(1), area.width, 1),
                            &format!("Last four · {tail}"),
                            ui.surface_style(),
                        );
                    }
                }
                _ => {
                    if let Some(reference) = self.accounts.selected_op.as_ref() {
                        let display = reference.display_path();
                        ui.paint_str(
                            Rect::new(area.x, input_y, area.width, 1),
                            &display,
                            ui.surface_style(),
                        );
                    } else {
                        ui.paint_str(
                            Rect::new(area.x, input_y, area.width, 1),
                            "Choose 1Password reference…",
                            ui.surface_style(),
                        );
                    }
                }
            }
            Self::account_save_button().draw(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
            );
            return;
        }
        let rows = self.account_rows();
        let list_area = Rect {
            height: area.height.saturating_sub(3),
            ..area
        };
        List::new(ACCOUNTS_LIST).draw(ui, list_area, &self.accounts.list, &rows);
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
        let mut y = area.y.saturating_add(1);
        let workspace = self.world.workspaces.first();
        let account_labels = self
            .world
            .offer_for(launch.agent, workspace, Some(self.selected_role()))
            .accounts
            .iter()
            .filter_map(|id| self.world.accounts.get(id).map(Account::title))
            .collect::<Vec<_>>();
        let accounts = if account_labels.is_empty() {
            "Accounts · none".to_owned()
        } else {
            format!(
                "{} accounts · {} · {}",
                account_labels.len(),
                launch.agent.provider().usage_surface().surface_name(),
                account_labels.join(" · ")
            )
        };
        ui.paint_str(Rect::new(area.x, y, area.width, 1), &accounts, style);
        y = y.saturating_add(1);
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
        let style = ui.surface_style();
        ui.paint_str(
            Rect::new(area.x, area.y.saturating_add(3), area.width, 1),
            "jackin❯",
            style,
        );
        Self::capsule_input().value(&self.capsule_input).draw(
            ui,
            Rect::new(
                area.x.saturating_add(8),
                area.y.saturating_add(3),
                area.width.saturating_sub(8),
                1,
            ),
            &self.capsule_input_state,
        );
        let pane_area = Rect {
            y: area.y.saturating_add(4),
            height: area.height.saturating_sub(4),
            ..area
        };
        let rows = self
            .world
            .instances
            .iter()
            .find(|instance| instance.status == InstanceStatus::Running)
            .map_or_else(
                || vec!["Capsule is empty".into()],
                |instance| {
                    let live = self.world.daemons.get(&instance.id);
                    match live {
                        Some(daemon) if !daemon.panes.is_empty() => daemon
                            .tabs
                            .iter()
                            .enumerate()
                            .flat_map(|(tab_index, tab)| {
                                let account_registry = self.world.accounts.clone();
                                let label = daemon.tab_label(tab, &|pane| {
                                    pane.proc
                                        .account
                                        .as_ref()
                                        .and_then(|id| account_registry.get(id))
                                        .map(|account| account.display_name.clone())
                                });
                                let tab_line = format!(
                                    "{}{} · {}",
                                    if tab_index == daemon.active {
                                        "▸ "
                                    } else {
                                        "  "
                                    },
                                    label,
                                    daemon.tab_state(tab).label()
                                );
                                let panes = tab.leaves().into_iter().filter_map(|pane_id| {
                                    daemon.pane(pane_id).map(|pane| {
                                        let heading = format!(
                                            "{} · {} · {}",
                                            pane.label(),
                                            pane.state().label(),
                                            if tab.focused == pane.id {
                                                "focused"
                                            } else {
                                                "idle focus"
                                            }
                                        );
                                        let transcript =
                                            pane.term.lines.iter().filter_map(|line| {
                                                let text = line
                                                    .iter()
                                                    .map(|span| span.text.as_str())
                                                    .collect::<String>();
                                                (!text.is_empty()).then_some(text)
                                            });
                                        std::iter::once(heading)
                                            .chain(transcript)
                                            .collect::<Vec<_>>()
                                    })
                                });
                                std::iter::once(tab_line).chain(panes.flatten())
                            })
                            .collect::<Vec<_>>(),
                        _ => match &instance.daemon {
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
                    }
                },
            );
        List::new(CAPSULE_PANES).draw(ui, pane_area, &ListState::default(), &rows);
        if self.capsule_usage {
            paint_lines(
                ui,
                Rect::new(
                    area.x.saturating_add(2),
                    area.y.saturating_add(4),
                    area.width.saturating_sub(4),
                    6,
                ),
                &[
                    "Usage · read-only",
                    "Overview",
                    "Limits",
                    "No credentials are displayed",
                ],
            );
        }
        if self.capsule_prefix {
            ui.paint_str(
                Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
                "prefix… New tab · Split right · Copy selection · Detach",
                ui.surface_style(),
            );
        }
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
        let dialog = Self::launch_dialog();
        let _ = ui.layer(LAUNCH_DIALOG, |ui, area| {
            dialog.draw(ui, area, &self.launch_dialog, |ui, body| {
                Button::new(ROLE_CHOOSE, self.selected_role()).draw(ui, body)
            })
        });
        let role_picker = Self::role_picker();
        let _ = ui.layer(ROLE_PICKER, |ui, area| {
            role_picker.draw(ui, area, &self.role_state, &self.roles)
        });
        let agent_picker =
            Picker::new(crate::screens::manager::AGENT_PICKER).title("Launch · choose Agent");
        let _ = ui.layer(crate::screens::manager::AGENT_PICKER, |ui, area| {
            agent_picker.draw(ui, area, &self.agent_state, &self.agent_options)
        });
        let account_picker = self.active_account_picker();
        let account_items = match self.picker_mode {
            Some(PickerMode::OnePassword) => &self.op_options,
            _ => &self.account_options,
        };
        let _ = ui.layer(ACCOUNT_PICKER, |ui, area| {
            account_picker.draw(ui, area, &self.account_state, account_items)
        });
    }
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        // Keep the shell's configured props owned by one constructor.  The
        // runtime only updates parts here; drawing consumes the same panel
        // shape below, so the app cannot drift between update and draw.
        let _shell = Self::shell_panel(&self.shell_meta);
        let mut result = self.advance_virtual_state(cx);
        self.ensure_manager_header();
        if let Some(command) = cx.command()
            && let Some(result) = self.update_command(cx, command)
        {
            if self.route == Route::Manager {
                self.ensure_manager_rows();
            }
            self.ensure_manager_header();
            return result;
        }
        result |= self.update_overlays(cx);
        if cx.is_open(ROLE_PICKER)
            || cx.is_open(crate::screens::manager::AGENT_PICKER)
            || cx.is_open(ACCOUNT_PICKER)
            || cx.is_open(LAUNCH_DIALOG)
        {
            return result;
        }
        if matches!(
            self.route,
            Route::Manager | Route::Accounts | Route::Usage | Route::Settings | Route::Capsule
        ) {
            result |= self.update_navigation(cx);
        }
        result |= self.update_route(cx);
        if self.route == Route::Manager {
            self.ensure_manager_rows();
        }
        self.ensure_manager_header();
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let too_small = TooSmall::new(APP.sub("too-small"), "Jackin Preview");
        if !too_small.fits(ui.design(), full) {
            too_small.draw(ui, full);
            return;
        }
        Self::shell_panel(&self.shell_meta).draw(ui, full, |ui, inner| {
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

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.route == Route::Outro {
            self.quit = true;
            return Response::changed();
        }
        if self.route == Route::Capsule && self.world.scenario == Scenario::OutroLast {
            self.status = Some("Detached from Capsule; closing the Construct…".into());
            self.outro = Some(OutroState::new(self.motion, Some(8_040), 0));
            self.route = Route::Outro;
            return Response::changed();
        }
        if self.route == Route::Capsule && self.world.running_count() > 1 {
            self.status = Some("Still inside the Construct · another instance is running".into());
            self.route = Route::Manager;
            return Response::changed();
        }
        if self.route == Route::Capsule && self.capsule_usage {
            self.capsule_usage = false;
            self.status = None;
            return Response::changed();
        }
        if self.route == Route::Capsule && self.capsule_prefix {
            self.capsule_prefix = false;
            self.status = None;
            return Response::changed();
        }
        if self.route == Route::Accounts {
            if self.accounts.remove_confirmation.take().is_some() {
                self.status = None;
                return Response::changed();
            }
            if self.accounts.form_open {
                self.accounts.close();
                if self.world.scenario == Scenario::AccountsMixed {
                    self.route = Route::Editor;
                } else {
                    self.status = Some("Cancelled account registration".into());
                }
                return Response::changed();
            }
        }
        if self.route == Route::Prelude {
            if self.prelude.step() == 2 {
                self.prelude.source_back();
            } else if self.prelude.step() > 1 {
                self.prelude.back();
            } else {
                self.status = Some("Cancelled · nothing created".into());
                self.route = Route::Manager;
            }
            return Response::changed();
        }
        if self.route == Route::Editor {
            if self.editor.env_form_open {
                self.editor.clear_env_form();
                return Response::changed();
            }
            if self.editor.dirty {
                self.status = Some("Save changes before leaving?".into());
            } else {
                self.route = Route::Manager;
            }
            return Response::changed();
        }
        if self.route == Route::Manager {
            if self.manager.detail_open() {
                self.manager.set_detail_open(false);
                cx.focus(MANAGER_LIST);
                return Response::changed();
            }
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
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('r')),
            CMD_ACCOUNT_REFRESH,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('v')),
            CMD_ACCOUNT_VALIDATE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('x')),
            CMD_ACCOUNT_REMOVE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('?')),
            CMD_ACCOUNT_HELP,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('e')),
            CMD_EDITOR_OPEN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Right),
            CMD_MANAGER_EXPAND,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('b'), KeyModifiers::CONTROL),
            CMD_CAPSULE_PREFIX,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('d')),
            CMD_CAPSULE_DETACH,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('%')),
            CMD_CAPSULE_SPLIT_RIGHT,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('"')),
            CMD_CAPSULE_SPLIT_BELOW,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('z')),
            CMD_CAPSULE_ZOOM,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('h')),
            CMD_CAPSULE_FOCUS_LEFT,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('\\'), KeyModifiers::CONTROL),
            CMD_CAPSULE_PALETTE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('q'), KeyModifiers::CONTROL),
            CMD_EXIT_DIALOG,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Backspace),
            CMD_PRELUDE_BACKSPACE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char(' ')),
            CMD_PRELUDE_SPACE,
        )
        .bind(
            KeyPhase::Capture,
            Chord::key(KeyCode::Char(' ')),
            CMD_PRELUDE_SPACE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Down),
            CMD_PRELUDE_DOWN,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Enter),
            CMD_EXIT_CONFIRM,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::End),
            CMD_NEW_WORKSPACE,
        )
        .bind(
            KeyPhase::Capture,
            Chord::key(KeyCode::End),
            CMD_NEW_WORKSPACE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char(']')),
            CMD_EDITOR_NEXT,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('[')),
            CMD_EDITOR_PREVIOUS,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('4')),
            CMD_EDITOR_ENV,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('3')),
            CMD_EDITOR_ROLES,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('5')),
            CMD_EDITOR_ACCOUNTS,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('p')),
            CMD_EDITOR_PREFER,
        )
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Down), CMD_USAGE_NEXT)
        .bind(
            KeyPhase::Bubble,
            Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL),
            CMD_SAVE,
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

fn register_provider(index: u8) -> Provider {
    match index {
        1 => Provider::OpenAi,
        2 => Provider::XAi,
        3 => Provider::OpenCode,
        _ => Provider::Anthropic,
    }
}

fn provider_slug(provider: Provider) -> &'static str {
    match provider {
        Provider::Anthropic => "anthropic",
        Provider::OpenAi => "openai",
        Provider::XAi => "xai",
        Provider::OpenCode => "opencode",
        _ => "provider",
    }
}

fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::Anthropic => "Claude · Anthropic",
        Provider::OpenAi => "Codex · OpenAI",
        Provider::XAi => "Grok Build · xAI",
        Provider::OpenCode => "OpenCode",
        _ => "Provider",
    }
}

fn source_label(index: u8) -> &'static str {
    match index {
        1 => "Local agent folder",
        2 => "API key",
        _ => "1Password reference",
    }
}

fn selected_account_id(world: &World, key: Option<ItemKey>) -> Option<String> {
    let Some(ItemKey::Index(index)) = key else {
        return None;
    };
    index
        .checked_sub(1)
        .and_then(|index| world.accounts.sorted().get(index).map(|a| a.id.clone()))
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

    #[test]
    fn end_is_a_capture_command_for_workspace_creation() {
        let key = junie_tui::Key {
            code: KeyCode::End,
            mods: KeyModifiers::NONE,
        };
        assert_eq!(
            app_keymap().lookup(KeyPhase::Capture, &key, false),
            Some(CMD_NEW_WORKSPACE)
        );
    }
}
