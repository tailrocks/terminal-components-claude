//! Jackin Preview application shell.
//!
//! This module owns only interaction state and paints through `tui-next`'s
//! public facade.  Domain and simulation state stay in sibling modules.

use junie_tui::author::PaintStyle;
use std::{collections::BTreeMap, mem, time::Duration};

use junie_tui::{
    ActionKey, App as TuiApp, AsItem, Brand, Button, Chord, ContextMenu, Cx, Dialog, DialogAction,
    DialogState, FrameRead, HelpAction, HelpOverlay, HelpOverlayState, HelpSection, Hint, HintBar,
    HintLayer, Id, Intent, Item, ItemKey, KeyCode, KeyMap, KeyModifiers, KeyPhase, List,
    ListAction, ListState, Menu, MenuAction, MenuBar, MenuItem, MenuState, Moment, Panel, Part,
    PartRef, Phase, Picker, PickerAction, PickerState, Position, Reconcile, Rect, Response,
    SecretPolicy, StatusBar, StatusItem, Tabs, TabsAction, TabsState, TextAction, TextInput,
    TextInputState, TextViewport, TooSmall, Ui, UpdateCause, Variant, ViewportAction, ViewportLine,
    ViewportState,
};

use crate::domain::account::{
    Account, CredentialSource, DetectedKind, DuplicateProbe, ValidationState, fingerprint, tail_of,
};
use crate::domain::agent::{Agent, Provider};
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::domain::usage::Freshness;
use crate::domain::workspace::{Effective, EnvValue, EnvVar, env_key_error, mask};
use crate::rain::{
    HANDOFF_LEN, INTRO_END, IntroPhase, IntroState, OutroPhase, OutroState, P1_LEN, PHRASES,
};
use crate::scenario::{Motion, Scenario};
use crate::screens::{
    accounts::AccountsState,
    capsule::{CapsuleInteraction, CapsuleState},
    cockpit::{AccountLine, CockpitState},
    editor::{EditorState, Tab as EditorTab},
    inspect::InspectState,
    manager::{LaunchCandidate, ManagerRowKey, ManagerState},
    prelude::PreludeState,
    settings::SettingsState,
    usage::{Tab as UsageTab, UsageState},
};
use crate::sim::launch::{BUILD_LOG, LaunchEvent, LaunchPlan, LaunchRun, Stage};
use crate::sim::provider;
use crate::sim::pty::{Daemon, PaneId, SplitDir};
use crate::sim::world::{World, world_for};

/// One cached projection binds domain identity, collection identity, and presentation.
#[derive(Debug, Clone)]
struct ManagerRow {
    domain: ManagerRowKey,
    key: ItemKey,
    label: String,
}

impl ManagerRow {
    fn new(domain: ManagerRowKey, label: String) -> Self {
        let key = ItemKey::text(&domain.stable_key());
        Self { domain, key, label }
    }
}

impl std::fmt::Display for ManagerRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

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
const CAPSULE_INPUT: Id = APP.sub("capsule-input");
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
/// Capsule menu-bar id.
const CAPSULE_MENU_BAR: Id = APP.sub("capsule-menu-bar");
/// Capsule tab context-menu id.
const CAPSULE_TAB_MENU: Id = APP.sub("capsule-tab-menu");
/// Capsule command-palette id.
const CAPSULE_COMMAND_PALETTE: Id = APP.sub("capsule-command-palette");
/// Capsule container identity dialog.
const CAPSULE_CONTAINER_INFO: Id = APP.sub("capsule-container-info");
const CAPSULE_HELP: Id = APP.sub("capsule-help");
const EDITOR_MOUNT_EDIT: Id = crate::screens::editor::ROOT.sub("mount-edit");
const EDITOR_ROLE_EDIT: Id = crate::screens::editor::ROOT.sub("role-edit");
const EDITOR_ROLE_LOAD: Id = crate::screens::editor::ROOT.sub("role-load");
const EDITOR_ACCOUNTS_LIST: Id = crate::screens::editor::ROOT.sub("accounts-list");
const EDITOR_SAVE_CONFIRM: Id = crate::screens::editor::ROOT.sub("save-confirm");
const SETTINGS_SAVE_CONFIRM: Id = crate::screens::settings::ROOT.sub("save-confirm");

const CMD_QUIT: ActionKey = ActionKey::application("jackin.quit");
const CMD_MANAGER: ActionKey = ActionKey::application("jackin.manager");
const CMD_ACCOUNTS: ActionKey = ActionKey::application("jackin.accounts");
const CMD_USAGE: ActionKey = ActionKey::application("jackin.usage");
const CMD_SETTINGS: ActionKey = ActionKey::application("jackin.settings");
const CMD_SETTINGS_TRUST_KEY: ActionKey = ActionKey::application("jackin.settings.trust-key");
const CMD_CAPSULE: ActionKey = ActionKey::application("jackin.capsule");
const CMD_CAPSULE_NEW_TAB: ActionKey = ActionKey::application("jackin.capsule.new-tab");
const CMD_NEW_WORKSPACE: ActionKey = ActionKey::application("jackin.new-workspace");
const CMD_EDITOR_NEXT: ActionKey = ActionKey::application("jackin.editor.next-tab");
const CMD_EDITOR_PREVIOUS: ActionKey = ActionKey::application("jackin.editor.previous-tab");
const CMD_EDITOR_ENV: ActionKey = ActionKey::application("jackin.editor.environments");
const CMD_SAVE: ActionKey = ActionKey::application("jackin.save");
const CMD_MANAGER_EXPAND: ActionKey = ActionKey::application("jackin.manager.expand");
const CMD_EDITOR_OPEN: ActionKey = ActionKey::application("jackin.editor.open");
const CMD_EDITOR_ROLES: ActionKey = ActionKey::application("jackin.editor.roles");
const CMD_EDITOR_PREFER: ActionKey = ActionKey::application("jackin.editor.prefer");
const CMD_NAV_DOWN: ActionKey = ActionKey::application("jackin.navigation.down");
const CMD_NAV_TAB_FIVE: ActionKey = ActionKey::application("jackin.navigation.tab-five");
const CMD_CAPSULE_PREFIX: ActionKey = ActionKey::application("jackin.capsule.prefix");
const CMD_CAPSULE_DETACH: ActionKey = ActionKey::application("jackin.capsule.detach");
const CMD_CAPSULE_SPLIT_RIGHT: ActionKey = ActionKey::application("jackin.capsule.split-right");
const CMD_CAPSULE_SPLIT_BELOW: ActionKey = ActionKey::application("jackin.capsule.split-below");
const CMD_CAPSULE_ZOOM: ActionKey = ActionKey::application("jackin.capsule.zoom");
const CMD_CAPSULE_FOCUS_LEFT: ActionKey = ActionKey::application("jackin.capsule.focus-left");
const CMD_CAPSULE_PALETTE: ActionKey = ActionKey::application("jackin.capsule.palette");
const CMD_EXIT_DIALOG: ActionKey = ActionKey::application("jackin.exit.dialog");
const CMD_EXIT_CONFIRM: ActionKey = ActionKey::application("jackin.exit.confirm");
const CMD_PRELUDE_BACKSPACE: ActionKey = ActionKey::application("jackin.prelude.backspace");
const CMD_PRELUDE_SPACE: ActionKey = ActionKey::application("jackin.prelude.space");
const CMD_ACCOUNT_REFRESH: ActionKey = ActionKey::application("jackin.account.refresh");
const CMD_ACCOUNT_VALIDATE: ActionKey = ActionKey::application("jackin.account.validate");
const CMD_ACCOUNT_REMOVE: ActionKey = ActionKey::application("jackin.account.remove");
const CMD_ACCOUNT_DEFAULT: ActionKey = ActionKey::application("jackin.account.default");
const CMD_ACCOUNT_HELP: ActionKey = ActionKey::application("jackin.account.help");
const CMD_COCKPIT_LOG: ActionKey = ActionKey::application("jackin.cockpit.build-log");
const CMD_TAB_RENAME: ActionKey = ActionKey::application("jackin.capsule.tab-rename");
const CMD_TAB_CLOSE: ActionKey = ActionKey::application("jackin.capsule.tab-close");
const CMD_INSPECT_CHANGES: ActionKey = ActionKey::application("jackin.capsule.inspect-changes");
const CMD_COPY_SELECTION: ActionKey = ActionKey::application("jackin.capsule.copy-selection");
const CMD_KEYBOARD_SHORTCUTS: ActionKey = ActionKey::application("jackin.keyboard-shortcuts");
const CMD_ABOUT: ActionKey = ActionKey::application("jackin.about");
const CMD_MENU_OPEN: ActionKey = ActionKey::application("jackin.menu.open");
const CMD_CONTAINER_INFO: ActionKey = ActionKey::application("jackin.capsule.container-info");
const CAPSULE_MENU_PREVIOUS: ActionKey = ActionKey::custom("menu.bar.prev.left");
const CAPSULE_MENU_NEXT: ActionKey = ActionKey::custom("menu.bar.next.right");

const CAPSULE_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_NEW_TAB, "New tab"),
    MenuItem::new(CMD_CAPSULE_SPLIT_RIGHT, "Split right"),
    MenuItem::new(CMD_CAPSULE_SPLIT_BELOW, "Split below"),
    MenuItem::new(CMD_COPY_SELECTION, "Copy selection"),
    MenuItem::new(CMD_INSPECT_CHANGES, "Inspect changes ·"),
];
const CAPSULE_EDIT_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_NEW_TAB, "New tab"),
    MenuItem::new(CMD_COPY_SELECTION, "Copy selection"),
    MenuItem::new(CMD_TAB_RENAME, "Change title…"),
];
const CAPSULE_VIEW_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_ZOOM, "Zoom pane"),
    MenuItem::new(CMD_CAPSULE_FOCUS_LEFT, "Focus left"),
    MenuItem::new(CMD_USAGE, "Usage"),
    MenuItem::new(CMD_CONTAINER_INFO, "Container info"),
    MenuItem::new(CMD_INSPECT_CHANGES, "Inspect changes ·"),
];
const CAPSULE_SESSION_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_NEW_TAB, "New tab"),
    MenuItem::new(CMD_TAB_CLOSE, "Close tab"),
    MenuItem::new(CMD_CAPSULE_DETACH, "Detach"),
];
const CAPSULE_HELP_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_KEYBOARD_SHORTCUTS, "Keyboard shortcuts"),
    MenuItem::new(CMD_ABOUT, "About Capsule"),
];
const CAPSULE_MENUS: &[Menu<'static>] = &[
    Menu::new("File", CAPSULE_FILE_ITEMS),
    Menu::new("Edit", CAPSULE_EDIT_ITEMS),
    Menu::new("View", CAPSULE_VIEW_ITEMS),
    Menu::new("Session", CAPSULE_SESSION_ITEMS),
    Menu::new("Help", CAPSULE_HELP_ITEMS),
];
const CAPSULE_TAB_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_TAB_RENAME, "Change title…"),
    MenuItem::new(CMD_TAB_CLOSE, "Close tab"),
];
const CAPSULE_COMMANDS: &[Item<'static>] = &[
    Item::new(ItemKey::num(1), "New tab"),
    Item::new(ItemKey::num(2), "Split right"),
    Item::new(ItemKey::num(3), "Split below"),
    Item::new(ItemKey::num(4), "Copy selection"),
    Item::new(ItemKey::num(5), "Inspect changes ·"),
    Item::new(ItemKey::num(6), "Zoom pane"),
    Item::new(ItemKey::num(7), "Focus left"),
    Item::new(ItemKey::num(8), "Usage"),
    Item::new(ItemKey::num(9), "Change title…"),
    Item::new(ItemKey::num(10), "Close tab"),
    Item::new(ItemKey::num(11), "Keyboard shortcuts"),
    Item::new(ItemKey::num(12), "Detach"),
    Item::new(ItemKey::num(13), "Container info"),
];
const TICK_MS: u64 = crate::rain::TICK_MS;

mod historical_paint;
use historical_paint::HistoricalPalette;

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

    /// Virtual time advanced by one admitted product tick.
    ///
    /// Idle routes may wake every 200 ms, but still advance only 80 virtual ms.
    /// Delayed wakes coalesce to one step; repaint/input counts never age state.
    pub const fn tick_ms(self) -> u64 {
        match self {
            Self::Intro | Self::Outro | Self::Handoff | Self::Cockpit | Self::Launch => TICK_MS,
            Self::Capsule => 80,
            Self::Manager
            | Self::Prelude
            | Self::Editor
            | Self::Accounts
            | Self::Usage
            | Self::Settings => 80,
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
    manager_rows_cache: Vec<ManagerRow>,
    manager_rows_revision: u64,
    shell_meta: String,
    manager_header: String,
    manager_header_running: usize,
    route: Route,
    motion: Motion,
    last_tick: Option<Moment>,
    quit: bool,
    keymap: KeyMap,
    capsule_menu_state: MenuState,
    container_info_state: DialogState,
    capsule_tab_menu_state: MenuState,
    capsule_tab_menu_pos: Position,
    capsule_tab_menu_open: bool,
    capsule_palette_state: PickerState,
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
    capsule_viewports: BTreeMap<PaneId, ViewportState>,
    capsule_viewport_lengths: BTreeMap<PaneId, usize>,
    capsule_viewport_revisions: BTreeMap<PaneId, u64>,
    capsule_viewport_retained: BTreeMap<PaneId, usize>,
    capsule_viewport_focused: bool,
    capsule_tab_title: String,
    capsule_tab_title_index: usize,
    capsule_tab_title_dialog: bool,
    capsule_tab_title_editing: bool,
    pending_capsule_action: Option<CapsuleAction>,
    capsule_interaction: CapsuleInteraction,
    editor_accounts: ListState,
    editor_accounts_transition: bool,
    editor_role_picker: bool,
    editor_env_role: Option<String>,
    help_open: bool,
    capsule_help_open: bool,
    capsule_help_state: HelpOverlayState,
    inspect_detail: bool,
    inspect_files: bool,
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
        Self::hydrate_capsule_accounts(&mut world);
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
        // Reference frame seeking advances cinematic/launch state, not the world clock.
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
            last_tick: None,
            quit: false,
            keymap: app_keymap(),
            capsule_menu_state: MenuState::default(),
            container_info_state: DialogState::default(),
            capsule_tab_menu_state: MenuState::default(),
            capsule_tab_menu_pos: Position::new(0, 0),
            capsule_tab_menu_open: false,
            capsule_palette_state: PickerState::default(),
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
            capsule_viewports: BTreeMap::new(),
            capsule_viewport_lengths: BTreeMap::new(),
            capsule_viewport_revisions: BTreeMap::new(),
            capsule_viewport_retained: BTreeMap::new(),
            capsule_viewport_focused: false,
            capsule_tab_title: String::new(),
            capsule_tab_title_index: 0,
            capsule_tab_title_dialog: false,
            capsule_tab_title_editing: false,
            pending_capsule_action: None,
            capsule_interaction: CapsuleInteraction::default(),
            editor_accounts: ListState::default(),
            editor_accounts_transition: false,
            editor_role_picker: false,
            editor_env_role: None,
            help_open: false,
            capsule_help_open: false,
            capsule_help_state: HelpOverlayState::default(),
            inspect_detail: false,
            inspect_files: false,
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
        app.sync_workspace_keymap();
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

    fn sync_workspace_keymap(&mut self) {
        let chord = Chord::key(KeyCode::End);
        self.keymap.remove(KeyPhase::Capture, chord);
        if self.route == Route::Manager
            || (self.route == Route::Editor
                && self.editor.tab == EditorTab::Environments
                && !self.editor.env_form_open)
        {
            self.keymap.add(KeyPhase::Capture, chord, CMD_NEW_WORKSPACE);
        }
    }

    fn route_changed(&mut self) -> Response<()> {
        self.sync_workspace_keymap();
        Response::changed()
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
            ManagerRowKey::CurrentDirectory | ManagerRowKey::NewWorkspace => None,
        }
    }

    fn active_running_instance_id(&self) -> Option<String> {
        self.active_instance.as_ref().and_then(|id| {
            self.world
                .instance(id)
                .filter(|instance| instance.status == InstanceStatus::Running)
                .map(|instance| instance.id.clone())
        })
    }

    fn hydrate_capsule_accounts(world: &mut World) {
        let workspace = world.workspaces.first().cloned();
        let accounts = Agent::ALL
            .into_iter()
            .map(|agent| {
                let account = world
                    .offer_for(agent, workspace.as_ref(), Some("chainargos/the-architect"))
                    .accounts
                    .into_iter()
                    .next();
                (agent, account)
            })
            .collect::<BTreeMap<_, _>>();
        for daemon in world.daemons.values_mut() {
            for pane in &mut daemon.panes {
                if let Some(agent) = pane.proc.agent
                    && pane.proc.account.is_none()
                {
                    pane.proc.account = accounts.get(&agent).cloned().flatten();
                }
            }
        }
    }

    fn sync_capsule_projection(&mut self) {
        let Some(instance_id) = self.active_running_instance_id() else {
            return;
        };
        self.active_instance = Some(instance_id.clone());
        let Some(daemon) = self.world.daemons.get(&instance_id) else {
            return;
        };
        let active_tab = daemon.active;
        self.capsule.tab = u8::try_from(active_tab).unwrap_or(u8::MAX);
        self.capsule.selected_pane = daemon.focused_pane().unwrap_or_default();
        self.capsule.zoomed = daemon.active_tab().is_some_and(|tab| tab.zoomed.is_some());
        self.tabs_state
            .set_active(active_tab, ItemKey::index(active_tab));
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

    fn editor_save_button(label: &'static str) -> Button<'static> {
        Button::new(crate::screens::editor::SAVE, label).variant(Variant::PRIMARY)
    }

    fn editor_save_confirm_button(create: bool) -> Button<'static> {
        Button::new(
            EDITOR_SAVE_CONFIRM,
            if create {
                "Create workspace"
            } else {
                "Apply changes"
            },
        )
        .variant(Variant::PRIMARY)
    }

    fn draw_editor_save_footer(&self, ui: &mut Ui<'_>, area: Rect) {
        Self::editor_save_button("Save workspace").draw(ui, area);
        if self.editor.preview_open {
            Self::editor_save_confirm_button(self.world.workspaces.is_empty()).draw(
                ui,
                Rect {
                    x: area.x.saturating_add(20),
                    width: 18,
                    ..area
                },
            );
        }
    }

    fn settings_save_button() -> Button<'static> {
        Button::new(crate::screens::settings::SAVE, "Save settings").variant(Variant::PRIMARY)
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

    fn role_picker(title: &'static str) -> Picker<'static, RoleOption> {
        Picker::new(ROLE_PICKER).title(title)
    }

    fn launch_agent_picker() -> Picker<'static, AgentOption> {
        Picker::new(crate::screens::manager::AGENT_PICKER).title("Launch · choose Agent")
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

    fn capsule_menu_bar() -> MenuBar<'static> {
        MenuBar::new(CAPSULE_MENU_BAR, CAPSULE_MENUS)
    }

    fn capsule_tab_context(position: Position) -> ContextMenu<'static> {
        ContextMenu::at(CAPSULE_TAB_MENU, CAPSULE_TAB_ITEMS, position).title("Tab")
    }

    fn capsule_command_palette() -> Picker<'static, Item<'static>> {
        Picker::new(CAPSULE_COMMAND_PALETTE)
            .title("Command palette")
            .placeholder("Search commands…")
    }

    fn container_info_dialog() -> Dialog<'static> {
        Dialog::info(CAPSULE_CONTAINER_INFO, "Container info").body_rows(9)
    }

    fn container_info_lines(&self) -> Vec<String> {
        let Some(instance_id) = self.active_running_instance_id() else {
            return vec!["No running Capsule instance is attached.".into()];
        };
        let Some(instance) = self.world.instance(&instance_id) else {
            return vec!["No running Capsule instance is attached.".into()];
        };
        let focused = self
            .world
            .daemons
            .get(&instance_id)
            .and_then(Daemon::focused_pane)
            .and_then(|pane_id| self.world.daemons.get(&instance_id)?.pane(pane_id));
        let agent = focused.map_or_else(
            || "(none)".to_owned(),
            |pane| match pane.proc.agent {
                Some(agent) => pane
                    .proc
                    .account
                    .as_ref()
                    .and_then(|id| self.world.accounts.get(id))
                    .map_or_else(|| agent.label().to_owned(), Account::title),
                None => "(shell)".to_owned(),
            },
        );
        vec![
            format!("Container       {}", instance.container_id()),
            format!("Container ID    {}", instance.container_uid()),
            format!("Role            {}", instance.role),
            format!("Agent           {agent}"),
            format!("Workdir         {}", instance.workdir),
            format!("Instance        {}", instance.id.trim_start_matches("jk-")),
            "Capsule         0.9.2".into(),
            format!("Invocation ID   {}-4f11", instance.run_id),
            format!(
                "Host log        file:///Users/alexey/.jackin/logs/{}.log",
                instance.run_id
            ),
        ]
    }

    fn open_container_info(&mut self, cx: &mut Cx<'_>) {
        self.container_info_state = DialogState::default();
        cx.open_layer(
            CAPSULE_CONTAINER_INFO,
            Self::container_info_dialog().layer(cx),
        );
        self.status = Some("Container info".into());
    }

    fn build_manager_rows(&self) -> Vec<ManagerRow> {
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
            rows.push(ManagerRow::new(
                ManagerRowKey::Workspace(workspace.id),
                format!(
                    "{marker} {} · {count} instance{}",
                    workspace.name,
                    if count == 1 { "" } else { "s" }
                ),
            ));
            if expanded {
                for instance in self.world.instances.iter().filter(|instance| {
                    instance.workspace == Some(workspace.id) && !instance.status.hidden()
                }) {
                    rows.push(ManagerRow::new(
                        ManagerRowKey::Instance(instance.id.clone()),
                        format!(
                            "  {} · instance · {} · run {} · {}",
                            instance.id,
                            instance.status.label(),
                            instance.run_id.short(),
                            instance.dirty_summary()
                        ),
                    ));
                }
            }
        }
        rows.push(ManagerRow::new(
            ManagerRowKey::CurrentDirectory,
            format!("Current directory · {}", self.world.home),
        ));
        rows.extend(
            self.world
                .instances
                .iter()
                .filter(|instance| instance.workspace.is_none() && !instance.status.hidden())
                .map(|instance| {
                    ManagerRow::new(
                        ManagerRowKey::Instance(instance.id.clone()),
                        format!(
                            "{} · {} · run {} · {}",
                            instance.id,
                            instance.status.label(),
                            instance.run_id.short(),
                            instance.dirty_summary()
                        ),
                    )
                }),
        );
        rows
    }

    fn ensure_manager_rows(&mut self) {
        if self.manager_rows_cache.is_empty()
            || self.manager_rows_revision != self.manager.rows_revision()
        {
            self.manager_rows_cache = self.build_manager_rows();
            self.manager.list.invalidate();
            self.manager_rows_revision = self.manager.rows_revision();
        }
    }

    fn reset_manager_cursor(&mut self) {
        self.ensure_manager_rows();
        if let Some(row) = self.manager_rows_cache.first() {
            self.manager.list.set_cursor(0, row.key);
            self.manager.select_row(row.domain.clone());
        }
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
        let mut rows = vec!["Overview · Health · Registration · Quota".to_owned()];
        let mut provider = None;
        for account in self.world.accounts.sorted() {
            if provider != Some(account.provider) {
                rows.push(account.provider.label().to_owned());
                provider = Some(account.provider);
            }
            rows.push(format!(
                "  {} · {} · {}",
                account.title(),
                account.status_word(),
                account.source.safe_detail()
            ));
        }
        rows
    }

    fn editor_account_rows(&self) -> Vec<String> {
        let effective = self.editor.pending.effective_accounts(&self.world.accounts);
        self.world
            .accounts
            .sorted()
            .into_iter()
            .map(|account| {
                let state = effective
                    .iter()
                    .find(|entry| entry.id == account.id)
                    .map_or_else(
                        || "disabled here".to_owned(),
                        |entry| {
                            let origin = if entry.origin.label() == "enabled here" {
                                "active for this Workspace · enabled here"
                            } else {
                                "inherited default · active for this Workspace"
                            };
                            if entry.preferred {
                                format!("{origin} · preferred")
                            } else {
                                origin.to_owned()
                            }
                        },
                    );
                format!("{} · {state}", account.title())
            })
            .collect()
    }

    fn editor_account_id(&self, index: usize) -> Option<String> {
        self.world
            .accounts
            .sorted()
            .get(index)
            .map(|account| account.id.clone())
    }

    fn toggle_editor_account(&mut self) {
        let Some(ItemKey::Index(index)) = self.editor_accounts.cursor() else {
            return;
        };
        let Some(id) = self.editor_account_id(index) else {
            return;
        };
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
            }
            Err(error) => self.status = Some(error),
        }
    }

    fn set_selected_account_default(&mut self) {
        if let Some(id) = self.accounts.selected_id.clone() {
            match self.world.accounts.set_default(&id) {
                Ok(()) => self.status = Some("Default set for provider".into()),
                Err(error) => self.status = Some(error),
            }
        }
    }

    fn launch_dialog() -> Dialog<'static> {
        Dialog::confirm(
            LAUNCH_DIALOG,
            "Launch a new session",
            "Review the role and start a deterministic Construct run.",
        )
        .body_rows(1)
    }

    fn open_agent_picker(&mut self, cx: &mut Cx<'_>) {
        self.agent_options = self.launch_candidates();
        self.agent_state = PickerState::default();
        if self.agent_options.is_empty() {
            self.status = Some("No configured agent account is available".into());
            return;
        }
        let picker = Self::launch_agent_picker();
        let spec = picker.layer(cx, &self.agent_options);
        cx.open_layer(crate::screens::manager::AGENT_PICKER, spec);
        self.status = Some("Launch · choose Agent".into());
    }

    fn open_role_picker(&mut self, cx: &mut Cx<'_>) {
        let picker = Self::role_picker("Choose a role");
        let spec = picker.layer(cx, &self.roles);
        cx.open_layer(ROLE_PICKER, spec);
    }

    fn open_editor_role_picker(&mut self, cx: &mut Cx<'_>) {
        self.editor_role_picker = true;
        self.role_state = PickerState::default();
        let picker = Self::role_picker("Add role override");
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
        let title = match action {
            CapsuleAction::NewTab => "New tab · Account for Claude Code",
            CapsuleAction::Split(SplitDir::Horizontal) => "Split right · Account for Claude Code",
            CapsuleAction::Split(SplitDir::Vertical) => "Split below · Account for Claude Code",
        };
        let picker = Self::account_picker().title(title);
        self.pending_capsule_action = Some(action);
        self.picker_mode = Some(PickerMode::Capsule);
        self.account_state = PickerState::default();
        let spec = picker.layer(cx, &self.account_options);
        cx.open_layer(ACCOUNT_PICKER, spec);
    }

    fn apply_capsule_action(&mut self, action: CapsuleAction, account: AccountOption) {
        let Some(instance_id) = self.active_running_instance_id() else {
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

    fn next_launch_run_id(&self) -> crate::RunId {
        let mut value = 0x9c41_e2f0_u64;
        while self
            .world
            .instances
            .iter()
            .any(|instance| instance.run_id.value() == value)
        {
            value = value.saturating_add(1);
        }
        crate::RunId::new(value)
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
            self.next_launch_run_id(),
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
        // A fresh launch starts one session.  The richer two-tab snapshot is
        // reserved for reconnecting to an already-running Capsule.
        let snapshot = match crate::domain::fixtures::live_capsule() {
            DaemonSnapshot::Tabs(mut tabs) => {
                tabs.truncate(1);
                if let Some(tab) = tabs.first_mut() {
                    tab.panes.truncate(1);
                }
                DaemonSnapshot::Tabs(tabs)
            }
            snapshot => snapshot,
        };
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
        self.world.daemons.insert(instance_id.clone(), daemon);
        self.world.instances.push(instance);
        self.active_instance = Some(instance_id.clone());
        self.world.sync_arbiter();
        self.manager_rows_cache.clear();
    }

    fn capsule_input() -> TextInput<'static> {
        TextInput::new(CAPSULE_INPUT).placeholder("Type a command")
    }

    fn capsule_viewport(pane_id: PaneId) -> TextViewport<'static> {
        TextViewport::new(CAPSULE_PANES.item(ItemKey::num(pane_id))).click_selects_word(true)
    }

    fn role_choose_button(&self) -> Button<'_> {
        Button::new(ROLE_CHOOSE, self.selected_role())
    }

    fn with_capsule_help<R>(f: impl FnOnce(&[HelpSection<'_>]) -> R) -> R {
        let pane = HintLayer {
            hints: vec![
                Hint {
                    chord: Chord::with(KeyCode::Char('b'), KeyModifiers::CONTROL),
                    label: "Prefix commands",
                    priority: 90,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('y')),
                    label: "Copy selection",
                    priority: 80,
                },
                Hint {
                    chord: Chord::key(KeyCode::PageUp),
                    label: "Scrollback",
                    priority: 70,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };
        let layout = HintLayer {
            hints: vec![
                Hint {
                    chord: Chord::key(KeyCode::Char('%')),
                    label: "Split right",
                    priority: 90,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('"')),
                    label: "Split below",
                    priority: 80,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('h')),
                    label: "Focus left",
                    priority: 70,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('z')),
                    label: "Zoom pane",
                    priority: 60,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };
        let session = HintLayer {
            hints: vec![
                Hint {
                    chord: Chord::key(KeyCode::Char('c')),
                    label: "New tab",
                    priority: 90,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('d')),
                    label: "Detach",
                    priority: 80,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char(',')),
                    label: "Rename tab",
                    priority: 70,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('&')),
                    label: "Close tab",
                    priority: 60,
                },
                Hint {
                    chord: Chord::with(KeyCode::Char('q'), KeyModifiers::CONTROL),
                    label: "Exit",
                    priority: 50,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };
        let navigation = HintLayer {
            hints: vec![
                Hint {
                    chord: Chord::key(KeyCode::Left),
                    label: "Previous tab",
                    priority: 90,
                },
                Hint {
                    chord: Chord::key(KeyCode::Right),
                    label: "Next tab",
                    priority: 80,
                },
                Hint {
                    chord: Chord::key(KeyCode::F(10)),
                    label: "Menu",
                    priority: 70,
                },
                Hint {
                    chord: Chord::with(KeyCode::Char('\\'), KeyModifiers::CONTROL),
                    label: "Command palette",
                    priority: 60,
                },
                Hint {
                    chord: Chord::key(KeyCode::Char('?')),
                    label: "Help",
                    priority: 50,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };
        let sections = [
            HelpSection::new("Pane", &pane),
            HelpSection::new("Layout", &layout),
            HelpSection::new("Session", &session),
            HelpSection::new("Navigation", &navigation),
        ];
        f(&sections)
    }

    fn open_capsule_help(&mut self, cx: &mut Cx<'_>) {
        self.capsule_help_open = true;
        self.capsule_help_state = HelpOverlayState::default();
        Self::with_capsule_help(|sections| {
            let help = HelpOverlay::new(CAPSULE_HELP, "Capsule", sections);
            cx.open_layer(CAPSULE_HELP, help.layer(cx));
        });
    }

    fn commit_capsule_input(&mut self) {
        let input = mem::take(&mut self.capsule_input);
        let Some(instance_id) = self.active_running_instance_id() else {
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
        if key == 'm' {
            self.capsule_prefix = false;
            self.capsule_tab_title_index = self.active_capsule_tab_index();
            self.capsule_tab_menu_open = true;
            self.capsule_tab_menu_state = MenuState::default();
            self.capsule_tab_menu_pos = Position::new(8, 2);
            cx.open_layer(
                CAPSULE_TAB_MENU,
                Self::capsule_tab_context(self.capsule_tab_menu_pos).layer(cx),
            );
            return Response::changed();
        }
        let command = match key {
            'c' => Some(CMD_CAPSULE),
            'd' => Some(CMD_CAPSULE_DETACH),
            'i' => Some(CMD_CONTAINER_INFO),
            '%' => Some(CMD_CAPSULE_SPLIT_RIGHT),
            '"' => Some(CMD_CAPSULE_SPLIT_BELOW),
            'z' => Some(CMD_CAPSULE_ZOOM),
            'h' => Some(CMD_CAPSULE_FOCUS_LEFT),
            'u' => Some(CMD_USAGE),
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

    fn capsule_viewport_id(pane_id: PaneId) -> Id {
        CAPSULE_PANES.item(ItemKey::num(pane_id))
    }

    fn active_capsule_tab_index(&self) -> usize {
        self.active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .map_or(0, |daemon| daemon.active)
    }

    fn active_capsule_pane_ids(&self) -> Vec<PaneId> {
        self.active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .and_then(Daemon::active_tab)
            .map(|tab| tab.leaves())
            .unwrap_or_default()
    }

    fn capsule_pane_lines(&self, pane_id: PaneId) -> Vec<String> {
        self.active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .and_then(|daemon| daemon.pane(pane_id))
            .map(|pane| {
                pane.term
                    .lines
                    .iter()
                    .map(|line| {
                        line.iter()
                            .map(|span| span.text.as_str())
                            .collect::<String>()
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn set_capsule_focused_pane(&mut self, pane_id: PaneId) {
        let Some(instance_id) = self.active_running_instance_id() else {
            return;
        };
        let snapshot = self.world.daemons.get_mut(&instance_id).and_then(|daemon| {
            let tab = daemon.active_tab_mut()?;
            if !tab.leaves().contains(&pane_id) {
                return None;
            }
            tab.focused = pane_id;
            Some(daemon.snapshot())
        });
        if let Some(snapshot) = snapshot {
            self.capsule.selected_pane = pane_id;
            if let Some(instance) = self.world.instance_mut(&instance_id) {
                instance.daemon = snapshot;
            }
        }
    }

    fn activate_capsule_tab(&mut self, index: usize) {
        let Some(instance_id) = self.active_running_instance_id() else {
            return;
        };
        let snapshot = self.world.daemons.get_mut(&instance_id).and_then(|daemon| {
            if index >= daemon.tabs.len() {
                return None;
            }
            daemon.active = index;
            Some(daemon.snapshot())
        });
        if let Some(snapshot) = snapshot {
            if let Some(instance) = self.world.instance_mut(&instance_id) {
                instance.daemon = snapshot;
            }
            self.sync_capsule_projection();
        }
    }

    fn update_capsule_viewports(&mut self, cx: &mut Cx<'_>, result: &mut Response<()>) {
        let Some(instance_id) = self.active_running_instance_id() else {
            return;
        };
        let Some(daemon) = self.world.daemons.get(&instance_id) else {
            return;
        };
        let Some(tab) = daemon.active_tab() else {
            return;
        };
        let sources = tab
            .leaves()
            .into_iter()
            .filter_map(|pane_id| {
                daemon.pane(pane_id).map(|pane| {
                    let lines = pane
                        .term
                        .lines
                        .iter()
                        .map(|line| {
                            line.iter()
                                .map(|span| span.text.as_str())
                                .collect::<String>()
                        })
                        .collect::<Vec<_>>();
                    (
                        pane_id,
                        lines,
                        pane.term.caret,
                        tab.focused == pane_id,
                        pane.term.revision(),
                        pane.term.retained(),
                    )
                })
            })
            .collect::<Vec<_>>();
        let pane_ids = daemon.panes.iter().map(|pane| pane.id).collect::<Vec<_>>();
        self.capsule_viewports
            .retain(|pane_id, _| pane_ids.contains(pane_id));
        self.capsule_viewport_lengths
            .retain(|pane_id, _| pane_ids.contains(pane_id));
        self.capsule_viewport_revisions
            .retain(|pane_id, _| pane_ids.contains(pane_id));
        self.capsule_viewport_retained
            .retain(|pane_id, _| pane_ids.contains(pane_id));

        let mut focused_pane = None;
        for (pane_id, lines, caret, focused, revision, retained) in sources {
            let id = Self::capsule_viewport_id(pane_id);
            let clicked = cx.intents(id).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Press | Phase::DragStart,
                        ..
                    }
                )
            });
            let previous_len = self
                .capsule_viewport_lengths
                .get(&pane_id)
                .copied()
                .unwrap_or(lines.len());
            let previous_retained = self
                .capsule_viewport_retained
                .get(&pane_id)
                .copied()
                .unwrap_or(retained);
            let previous_revision = self.capsule_viewport_revisions.get(&pane_id).copied();
            let state = self.capsule_viewports.entry(pane_id).or_default();
            let dropped = retained.saturating_sub(previous_retained);
            if dropped > 0 {
                state.retained(dropped);
            } else if previous_len > lines.len() {
                state.clear_selection();
                state.invalidate();
            }
            if previous_revision.is_some_and(|previous| previous != revision) {
                state.invalidate();
            }
            state.set_caret(focused.then_some(caret).flatten());
            let viewport_lines = lines
                .iter()
                .map(|line| ViewportLine::Plain(line.as_str()))
                .collect::<Vec<_>>();
            let viewport_response =
                Self::capsule_viewport(pane_id).update(cx, state, &viewport_lines);
            let owns_focus = clicked || viewport_response.focused();
            let viewport_action = viewport_response.action_ref().cloned();
            *result |= viewport_response.erase();
            let copied = match viewport_action {
                Some(ViewportAction::Copy(text)) => Some(text),
                Some(ViewportAction::SelectionChanged) => {
                    let mut text = String::new();
                    self.capsule_viewports.get(&pane_id).and_then(|state| {
                        state.copy_into(&viewport_lines, &mut text).then_some(text)
                    })
                }
                _ => None,
            };
            if let Some(text) = copied {
                self.world.clipboard = Some(text);
                *result |= Response::changed();
            }
            if owns_focus {
                focused_pane = Some(pane_id);
            }
            self.capsule_viewport_lengths.insert(pane_id, lines.len());
            self.capsule_viewport_revisions.insert(pane_id, revision);
            self.capsule_viewport_retained.insert(pane_id, retained);
        }
        if let Some(pane_id) = focused_pane {
            self.set_capsule_focused_pane(pane_id);
            self.capsule_viewport_focused = true;
        }
    }

    fn update_overlays(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();

        Self::with_capsule_help(|sections| {
            let help = HelpOverlay::new(CAPSULE_HELP, "Capsule", sections);
            let response = help.update(cx, &mut self.capsule_help_state);
            let closed = matches!(response.action_ref(), Some(HelpAction::Closed(_)));
            result |= response.erase();
            if closed {
                self.capsule_help_open = false;
                self.status = None;
            }
        });

        let info = Self::container_info_dialog();
        let response = info.update(cx, &mut self.container_info_state);
        if response.action_ref().is_some() {
            self.status = None;
        }
        result |= response.erase();

        // Drain the dialog and its body control while a child picker is
        // open too.  Layer dismissal intents are addressed to the underlying
        // owners during the same update pass; returning early for the top
        // picker leaves those intents undelivered and makes nested overlays
        // noisy in diagnostics.
        let dialog = Self::launch_dialog();
        let response = dialog.update(cx, &mut self.launch_dialog);
        let action = response.action_ref().copied();
        result |= response.erase();
        let role = self.role_choose_button().update(cx);
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

        let picker = Self::role_picker(if self.editor_role_picker {
            "Add role override"
        } else {
            "Choose a role"
        });
        let response = picker.update(cx, &mut self.role_state, &self.roles);
        let action = response.action_ref().copied();
        result |= response.erase();
        if cx.is_open(ROLE_PICKER)
            && let Some(PickerAction::Chosen(key)) = action
            && let Some(index) = self.roles.iter().position(|role| {
                if self.editor_role_picker && !self.role_state.query().is_empty() {
                    role.key
                        .to_ascii_lowercase()
                        .contains(&self.role_state.query().to_ascii_lowercase())
                } else {
                    ItemKey::text(&role.key) == key
                }
            })
        {
            if self.editor_role_picker {
                let role = self.roles.get(index).map_or_else(String::new, |role| {
                    role.key
                        .rsplit_once('/')
                        .map_or_else(|| role.key.clone(), |(_, name)| name.to_owned())
                });
                self.editor_role_picker = false;
                self.editor_env_role = Some(role.clone());
                self.editor.open_env_form();
                cx.focus(crate::screens::editor::ENV_KEY);
                self.status = Some(format!("Add role override · {role}"));
            } else {
                self.selected_role = index;
            }
            cx.close_layer(ROLE_PICKER, Some(ActionKey::CONFIRM));
            result |= Response::changed();
        }

        let agent_picker = Self::launch_agent_picker();
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

    fn update_navigation(&self, cx: &mut Cx<'_>) -> (Response<()>, Option<Route>) {
        let mut result = Response::ignored();
        let mut chosen_route = None;
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
                chosen_route = Some(route);
            }
        }
        (result, chosen_route)
    }

    fn enter_intro(&mut self) {
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

    fn update_manager(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.ensure_manager_rows();
        let list = List::new(MANAGER_LIST)
            .key(|row: &ManagerRow| row.key)
            .update(cx, &mut self.manager.list, &self.manager_rows_cache);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        let selected_key = match list_action {
            Some(ListAction::Activated(key) | ListAction::Chose(key)) => Some(key),
            _ => self.manager.list.cursor(),
        };
        if let Some(key) = selected_key
            && let Some(row) = self.manager_rows_cache.iter().find(|row| row.key == key)
            && self.manager.selected_row() != &row.domain
        {
            self.manager.select_row(row.domain.clone());
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
                } else if matches!(
                    self.manager.selected_row(),
                    ManagerRowKey::Workspace(_)
                        | ManagerRowKey::CurrentDirectory
                        | ManagerRowKey::NewWorkspace
                ) {
                    self.open_agent_picker(cx);
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
            self.prelude = PreludeState::default();
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
                cx.focus(ACCOUNTS_LIST);
                result |= Response::changed();
            }
            return result;
        }

        let rows = self.account_rows();
        let list = List::new(ACCOUNTS_LIST).update(cx, &mut self.accounts.list, &rows);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        let previous = self.accounts.selected_id.clone();
        self.accounts.selected_id = selected_account_id(&self.world, self.accounts.list.cursor());
        if matches!(list_action, Some(ListAction::Moved)) && self.accounts.selected_id != previous {
            self.status = self
                .accounts
                .selected_id
                .as_deref()
                .and_then(|id| self.world.accounts.get(id))
                .map(|account| {
                    format!(
                        "Accounts › {} › {}",
                        account.surface.surface_name(),
                        account.display_name
                    )
                });
        }
        if matches!(list_action, Some(ListAction::Chose(_))) {
            self.set_selected_account_default();
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
            .map(ValidationState::Valid)
            .unwrap_or(ValidationState::NeverValidated);
        if let Some(usage) = outcome.usage {
            account.usage = usage;
        }
        if provider == Provider::XAi {
            account = account.with_endpoint("Grok Team", "https://api.x.ai");
        }
        if !self
            .world
            .accounts
            .accounts
            .iter()
            .any(|existing| existing.provider == provider && existing.default_for_provider)
        {
            account.default_for_provider = true;
        }
        let title = account.title();
        let issue = account.issue.as_ref().map(|issue| issue.message.clone());
        self.world.accounts.insert(account);
        let selected_id = id.clone();
        self.accounts.selected_id = Some(id);
        if let Some(index) = account_row_index(&self.world, &selected_id) {
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
        self.accounts.secret_input = TextInputState::default();
        self.accounts.folder_input = TextInputState::default();
        self.status = Some(match issue {
            Some(issue) => format!("Saved {title} · {issue}"),
            None => format!("Saved {title}"),
        });
    }

    fn commit_editor_save(&mut self) {
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
        let save = Self::settings_save_button().update(cx);
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

            let save = Self::editor_save_button("Save workspace").update(cx);
            let raw_save_chosen = save.activated();
            let save_chosen = raw_save_chosen
                && cx
                    .state(crate::screens::editor::SAVE)
                    .contains(junie_tui::StateFlags::FOCUSED);
            result |= save.erase();
            if save_chosen {
                let key = self.editor.env_key.trim().to_owned();
                if let Some(error) = env_key_error(&key) {
                    self.status = Some(error);
                } else {
                    let status = format!("Added environment variable {key}");
                    let value = self.editor.take_env_value();
                    if let Some(role) = self.editor_env_role.take() {
                        if let Err(error) =
                            self.editor.pending.add_role_environment(role, &key, value)
                        {
                            self.status = Some(error);
                            return result;
                        }
                    } else {
                        self.editor.pending.env.push(EnvVar {
                            key,
                            value: EnvValue::Plain(value),
                        });
                    }
                    self.editor.clear_env_form();
                    self.editor.mark_dirty();
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
                if matches!(action, Some(ListAction::Activated(_))) {
                    if self.editor_accounts_transition {
                        self.editor_accounts_transition = false;
                    } else {
                        self.toggle_editor_account();
                    }
                    result |= Response::changed();
                }
            }
            EditorTab::Environments => {
                let load = Button::new(EDITOR_ROLE_LOAD, "+ Add role override…").update(cx);
                let chosen = load.activated();
                result |= load.erase();
                if chosen {
                    self.open_editor_role_picker(cx);
                    result |= Response::changed();
                }
            }
            EditorTab::General => {}
        }

        let save = Self::editor_save_button("Save workspace").update(cx);
        let save_chosen = save.activated()
            && cx
                .state(crate::screens::editor::SAVE)
                .contains(junie_tui::StateFlags::FOCUSED);
        result |= save.erase();
        let confirm = Self::editor_save_confirm_button(self.world.workspaces.is_empty()).update(cx);
        let confirm_chosen = confirm.activated()
            && cx
                .state(EDITOR_SAVE_CONFIRM)
                .contains(junie_tui::StateFlags::FOCUSED);
        result |= confirm.erase();
        if save_chosen {
            if self.editor.preview_open {
                self.commit_editor_save();
                result |= Response::changed();
            } else if self.editor.open_preview() {
                cx.focus(EDITOR_SAVE_CONFIRM);
                self.status = Some("Save workspace · preview changes before commit".into());
                result |= Response::changed();
            }
        }
        if confirm_chosen && self.editor.preview_open {
            self.commit_editor_save();
            result |= Response::changed();
        }
        result
    }

    fn update_launch(&mut self, cx: &mut Cx<'_>, product_tick: bool) -> Response<()> {
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
        if product_tick && let Some(launch) = &mut self.launch {
            let events = launch.advance();
            if !events.is_empty() {
                self.handle_launch_events(events);
                result |= Response::changed();
            }
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
        let mut result = Response::ignored();
        // Prefix input may retain focus on a child control after a modal
        // closes. Consume its next raw key at the route boundary before any
        // child (menu, tabs, panes, or input) can claim it.
        if self.capsule_prefix {
            let prefix_key = [
                CAPSULE_INPUT,
                CAPSULE_MENU_BAR,
                CAPSULE_TAB_MENU,
                CAPSULE_TABS,
            ]
            .into_iter()
            .find_map(|id| {
                cx.intents(id).find_map(|intent| match intent {
                    Intent::Key(key) => key.bare_char(),
                    _ => None,
                })
            });
            if let Some(key) = prefix_key {
                self.capsule_input.clear();
                self.capsule_input_state = TextInputState::default();
                let response = self.capsule_prefix_key(cx, key);
                if key != 'm' {
                    return response;
                }
                result |= response;
            }
        }
        if self.capsule_tab_menu_open && !cx.is_open(CAPSULE_TAB_MENU) {
            cx.open_layer(
                CAPSULE_TAB_MENU,
                Self::capsule_tab_context(self.capsule_tab_menu_pos).layer(cx),
            );
        }
        let menu = Self::capsule_menu_bar();
        let menu_intents = cx.intents(CAPSULE_MENU_BAR).collect::<Vec<_>>();
        let menu_state_before = self.capsule_menu_state;
        let menu_response = menu.update(cx, &mut self.capsule_menu_state);
        if let Some(open) = menu_state_before.open_menu() {
            let count = CAPSULE_MENUS.len();
            let next = menu_intents.iter().find_map(|intent| match intent {
                Intent::Binding(action) if *action == CAPSULE_MENU_PREVIOUS => Some(if open == 0 {
                    count.saturating_sub(1)
                } else {
                    open - 1
                }),
                Intent::Binding(action) if *action == CAPSULE_MENU_NEXT => {
                    Some(open.saturating_add(1) % count)
                }
                _ => None,
            });
            if let Some(index) = next {
                result |= menu
                    .open_menu(cx, &mut self.capsule_menu_state, index)
                    .erase();
            }
        }
        if let Some(MenuAction::Chosen(action)) = menu_response.action_ref().copied() {
            if let Some(response) = self.update_command(cx, action) {
                result |= response;
            }
            // Menu actions are terminal transitions. Keep the next key for
            // the resulting route/dialog instead of replaying it through the
            // prior dropdown state.
            self.capsule_menu_state = MenuState::default();
            cx.close_layer(CAPSULE_MENU_BAR, None);
            if action == CMD_INSPECT_CHANGES {
                cx.focus(CAPSULE_INPUT);
            }
        }
        result |= menu_response.erase();

        let context = Self::capsule_tab_context(self.capsule_tab_menu_pos);
        let context_response = context.update(cx, &mut self.capsule_tab_menu_state);
        match context_response.action_ref().copied() {
            Some(MenuAction::Chosen(action)) => {
                self.capsule_tab_menu_open = false;
                self.capsule_tab_menu_state = MenuState::default();
                if let Some(response) = self.update_command(cx, action) {
                    result |= response;
                }
                cx.close_layer(CAPSULE_TAB_MENU, None);
            }
            Some(MenuAction::Closed(_)) => {
                self.capsule_tab_menu_open = false;
                self.capsule_tab_menu_state = MenuState::default();
            }
            _ => {}
        }
        result |= context_response.erase();

        if !cx.is_open(CAPSULE_TAB_MENU) {
            let pane_context = self
                .active_capsule_pane_ids()
                .into_iter()
                .find_map(|pane_id| {
                    let id = Self::capsule_viewport_id(pane_id);
                    cx.intents(id).find_map(|intent| match intent {
                        Intent::Pointer {
                            phase: Phase::Secondary,
                            pos,
                            ..
                        } => Some(pos),
                        _ => None,
                    })
                });
            let tab_context = cx.intents(CAPSULE_TABS).find_map(|intent| match intent {
                Intent::Pointer {
                    phase: Phase::Secondary,
                    pos,
                    part:
                        PartRef {
                            part: Part::TAB,
                            item: Some(ItemKey::Index(index)),
                        },
                    ..
                } => Some((pos, index)),
                _ => None,
            });
            if let Some(pos) = pane_context {
                self.capsule_tab_title_index = self.active_capsule_tab_index();
                self.capsule_tab_menu_pos = pos;
                self.capsule_tab_menu_open = true;
                self.capsule_tab_menu_state = MenuState::default();
                cx.open_layer(CAPSULE_TAB_MENU, Self::capsule_tab_context(pos).layer(cx));
                result |= Response::changed();
            } else if let Some((pos, index)) = tab_context {
                self.capsule_tab_title_index = index;
                self.capsule_tab_menu_pos = pos;
                self.capsule_tab_menu_open = true;
                self.capsule_tab_menu_state = MenuState::default();
                cx.open_layer(CAPSULE_TAB_MENU, Self::capsule_tab_context(pos).layer(cx));
                result |= Response::changed();
            }
        }

        if cx.is_open(CAPSULE_COMMAND_PALETTE) {
            let palette = Self::capsule_command_palette();
            let palette_response =
                palette.update(cx, &mut self.capsule_palette_state, CAPSULE_COMMANDS);
            if let Some(PickerAction::Chosen(key)) = palette_response.action_ref().copied() {
                let action = match key {
                    ItemKey::Num(1) => Some(CMD_CAPSULE_NEW_TAB),
                    ItemKey::Num(2) => Some(CMD_CAPSULE_SPLIT_RIGHT),
                    ItemKey::Num(3) => Some(CMD_CAPSULE_SPLIT_BELOW),
                    ItemKey::Num(4) => Some(CMD_COPY_SELECTION),
                    ItemKey::Num(5) => Some(CMD_INSPECT_CHANGES),
                    ItemKey::Num(6) => Some(CMD_CAPSULE_ZOOM),
                    ItemKey::Num(7) => Some(CMD_CAPSULE_FOCUS_LEFT),
                    ItemKey::Num(8) => Some(CMD_USAGE),
                    ItemKey::Num(9) => Some(CMD_TAB_RENAME),
                    ItemKey::Num(10) => Some(CMD_TAB_CLOSE),
                    ItemKey::Num(11) => Some(CMD_KEYBOARD_SHORTCUTS),
                    ItemKey::Num(12) => Some(CMD_CAPSULE_DETACH),
                    ItemKey::Num(13) => Some(CMD_CONTAINER_INFO),
                    _ => None,
                };
                if let Some(action) = action
                    && let Some(response) = self.update_command(cx, action)
                {
                    result |= response;
                }
                cx.close_layer(CAPSULE_COMMAND_PALETTE, None);
            }
            result |= palette_response.erase();
        }

        // Prefix commands own the next key. Do not let the focused command
        // input consume it before the bubble action can dispatch.
        if self.capsule_prefix {
            let prefix_key = cx.intents(CAPSULE_INPUT).find_map(|intent| match intent {
                Intent::Key(key) => key.bare_char(),
                _ => None,
            });
            if let Some(key) = prefix_key {
                self.capsule_input.clear();
                self.capsule_input_state = TextInputState::default();
                result |= self.capsule_prefix_key(cx, key);
            }
            return result;
        }
        // Capsule exit is an app command, not text.  A focused command input
        // may otherwise settle or consume the raw control key before the
        // bubble keymap gets a chance to dispatch it.
        let exit_requested = cx.intents(CAPSULE_INPUT).any(|intent| {
            matches!(
                intent,
                Intent::Key(key)
                    if key.code == KeyCode::Char('q')
                        && key.mods.contains(KeyModifiers::CONTROL)
            )
        });
        if exit_requested {
            if let Some(response) = self.update_command(cx, CMD_EXIT_DIALOG) {
                result |= response;
            }
            return result;
        }
        // Inspect commands are app actions.  The focused command input must
        // not turn the `m` shortcut into text while a diff is open.
        let inspect_manager_requested = self.inspect.instance.is_some()
            && self.inspect_detail
            && cx.intents(CAPSULE_INPUT).any(|intent| {
                matches!(
                    intent,
                    Intent::Key(key)
                        if key.code == KeyCode::Char('m') && key.mods == KeyModifiers::NONE
                )
            });
        if inspect_manager_requested {
            if let Some(response) = self.update_command(cx, CMD_MANAGER) {
                result |= response;
            }
            return result;
        }
        // The prefix action waits for confirmation.  `TextInput` publishes
        // Enter as a component binding while idle, so open the account picker
        // on that first confirmation instead of requiring a hidden edit-start
        // keystroke before the action can progress.
        let pending_capsule_confirm = self.pending_capsule_action.is_some()
            && !self.capsule_input_state.is_editing()
            && cx
                .intents(CAPSULE_INPUT)
                .any(|intent| matches!(intent, Intent::Binding(_)));
        if pending_capsule_confirm {
            if let Some(action) = self.pending_capsule_action.take() {
                self.open_capsule_account_picker(cx, action);
                self.status = Some("New tab · Account for Claude Code".into());
            }
            return result | Response::changed();
        }
        if !self.capsule_input_state.is_editing()
            && self.exit_choice.is_none()
            && self
                .capsule_viewports
                .values()
                .all(|viewport| viewport.selection().is_none())
            && !self.capsule_viewport_focused
            && !self.capsule_tab_menu_open
            && !cx.is_open(CAPSULE_TAB_MENU)
        {
            cx.focus(CAPSULE_INPUT);
        }
        self.update_capsule_viewports(cx, &mut result);
        let input = Self::capsule_input().update(
            cx,
            &mut self.capsule_input_state,
            &mut self.capsule_input,
        );
        let input_focused = input.focused();
        let committed = input
            .action_ref()
            .is_some_and(|action| *action == TextAction::Committed);
        result |= input.erase();
        if input_focused {
            self.capsule_viewport_focused = false;
        }
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
            if self.capsule_tab_title_dialog && !self.capsule_tab_title_editing {
                self.capsule_input_state.begin(&self.capsule_input);
                self.capsule_tab_title_editing = true;
                cx.focus(CAPSULE_INPUT);
            } else if self.capsule_tab_title_editing {
                self.capsule_tab_title = mem::take(&mut self.capsule_input);
                self.capsule_tab_title_editing = false;
                self.capsule_tab_title_dialog = false;
                self.capsule_input_state = TextInputState::default();
                self.status = None;
            } else if let Some(action) = self.pending_capsule_action.take() {
                self.open_capsule_account_picker(cx, action);
                self.status = Some("New tab · Account for Claude Code".into());
            } else if self.exit_choice.is_some() {
                if let Some(response) = self.update_command(cx, CMD_EXIT_CONFIRM) {
                    result |= response;
                }
            } else {
                self.commit_capsule_input();
            }
            result |= Response::changed();
        }
        let tabs = self.capsule_tabs();
        let tabs_response = Tabs::new(CAPSULE_TABS).update(cx, &mut self.tabs_state, &tabs);
        if let Some(TabsAction::Activated(ItemKey::Index(index))) =
            tabs_response.action_ref().copied()
        {
            self.activate_capsule_tab(index);
            self.capsule_tab_title_index = index;
            self.capsule_viewport_focused = false;
        }
        result | tabs_response.erase()
    }

    fn update_route(&mut self, cx: &mut Cx<'_>, product_tick: bool) -> Response<()> {
        match self.route {
            Route::Intro => Response::ignored(),
            Route::Manager => self.update_manager(cx),
            Route::Prelude => self.update_prelude(cx),
            Route::Editor => self.update_editor(cx),
            Route::Accounts => self.update_accounts(cx),
            Route::Usage => Response::ignored(),
            Route::Settings => self.update_settings(cx),
            Route::Launch | Route::Cockpit => self.update_launch(cx, product_tick),
            Route::Handoff | Route::Outro => Response::ignored(),
            Route::Capsule => self.update_capsule(cx),
        }
    }

    fn update_command(&mut self, cx: &mut Cx<'_>, command: ActionKey) -> Option<Response<()>> {
        if self.route == Route::Capsule
            && self.inspect.instance.is_some()
            && matches!(command, CMD_MANAGER | CMD_CAPSULE_DETACH | CMD_EXIT_CONFIRM)
        {
            match command {
                CMD_MANAGER => {
                    self.inspect_files = true;
                    self.inspect_detail = false;
                }
                CMD_CAPSULE_DETACH => {
                    self.inspect_detail = true;
                }
                CMD_EXIT_CONFIRM => {
                    self.inspect_detail = true;
                }
                _ => {}
            }
            return Some(Response::changed());
        }
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
                    return Some(self.capsule_prefix_key(cx, 'm'));
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
                self.editor_env_role = None;
                self.editor.open_env_form();
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_ACCOUNTS => {
                if cx.update_cause() == UpdateCause::Settle {
                    return Some(Response::changed());
                }
                if self.route == Route::Accounts {
                    self.accounts.open_new();
                    self.op_item_key.clear();
                    cx.focus(crate::screens::accounts::START);
                } else {
                    self.route = Route::Accounts;
                    cx.focus(ACCOUNTS_LIST);
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
            CMD_ACCOUNT_HELP if self.route == Route::Manager => {
                self.help_open = true;
                self.status = Some("Keyboard shortcuts".into());
                Some(Response::changed())
            }
            CMD_ACCOUNT_HELP if self.route == Route::Capsule => {
                self.open_capsule_help(cx);
                self.status = None;
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
            CMD_CAPSULE_NEW_TAB if self.route == Route::Capsule => {
                self.open_capsule_account_picker(cx, CapsuleAction::NewTab);
                self.status = Some("New tab · Account for Claude Code".into());
                Some(Response::changed())
            }
            CMD_CAPSULE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.pending_capsule_action = Some(CapsuleAction::NewTab);
                    self.status = Some("New tab".into());
                } else if cx.update_cause() == UpdateCause::Settle {
                    return Some(Response::changed());
                } else if self.route == Route::Manager {
                    self.route = Route::Accounts;
                    cx.focus(ACCOUNTS_LIST);
                } else {
                    self.route = Route::Capsule;
                }
                Some(Response::changed())
            }
            CMD_MENU_OPEN if self.route == Route::Capsule => {
                // F10 is a fresh menu invocation.  A prior mouse/keyboard
                // traversal must not make it reopen the last top-level menu.
                self.capsule_menu_state = MenuState::default();
                let index = 0;
                let response =
                    Self::capsule_menu_bar().open_menu(cx, &mut self.capsule_menu_state, index);
                Some(response.erase())
            }
            CMD_COPY_SELECTION if self.route == Route::Capsule => {
                if let Some(pane_id) = self
                    .active_running_instance_id()
                    .and_then(|id| self.world.daemons.get(&id))
                    .and_then(Daemon::focused_pane)
                {
                    let rows = self.capsule_pane_lines(pane_id);
                    let lines = rows
                        .iter()
                        .map(|line| ViewportLine::Plain(line.as_str()))
                        .collect::<Vec<_>>();
                    let mut text = String::new();
                    if self
                        .capsule_viewports
                        .get(&pane_id)
                        .is_some_and(|viewport| viewport.copy_into(&lines, &mut text))
                    {
                        self.world.clipboard = Some(text);
                        self.status = Some("Copied selection".into());
                    }
                }
                Some(Response::changed())
            }
            CMD_INSPECT_CHANGES if self.route == Route::Capsule => {
                self.inspect.instance = self
                    .active_instance
                    .clone()
                    .or_else(|| self.active_running_instance_id());
                self.inspect_detail = false;
                self.inspect_files = false;
                self.status = Some("Inspect changes · choose a file".into());
                Some(Response::changed())
            }
            CMD_KEYBOARD_SHORTCUTS if self.route == Route::Capsule => {
                self.open_capsule_help(cx);
                self.status = None;
                Some(Response::changed())
            }
            CMD_ABOUT if self.route == Route::Capsule => {
                self.status = Some("Capsule · terminal workspace control plane".into());
                Some(Response::changed())
            }
            CMD_TAB_RENAME if self.route == Route::Capsule => {
                self.capsule_tab_title_dialog = true;
                self.capsule_tab_title_editing = false;
                self.status = Some("Change tab title".into());
                Some(Response::changed())
            }
            CMD_TAB_CLOSE if self.route == Route::Capsule => {
                self.status = Some("Close tab? · Enter confirm · Esc cancel".into());
                Some(Response::changed())
            }
            CMD_COCKPIT_LOG if matches!(self.route, Route::Launch | Route::Cockpit) => {
                self.cockpit.log_open = true;
                self.cockpit.log_scroll = 0;
                self.status = Some("Docker build · scroll to inspect output".into());
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
                    // Hard-case fixtures exercise the scoped override list;
                    // retain its existing backend override even when the
                    // persisted fixture has no role-local environment rows.
                    if self.world.scenario == Scenario::HardCases
                        && self.editor.pending.role_env.is_empty()
                    {
                        self.editor.pending.role_env.insert(
                            "backend".into(),
                            vec![EnvVar::plain("BACKEND_MODE", "staging")],
                        );
                    }
                } else {
                    self.editor = EditorState::default();
                }
                self.editor.select_alias(1);
                self.editor_accounts = ListState::default();
                self.editor_role_picker = false;
                self.editor_env_role = None;
                Some(Response::changed())
            }
            CMD_CAPSULE_PREFIX if self.route == Route::Capsule => {
                self.capsule_prefix = true;
                self.capsule_viewport_focused = false;
                self.status = Some("prefix… New tab · Split · Copy · Detach".into());
                cx.focus(CAPSULE_INPUT);
                Some(Response::changed())
            }
            CMD_CAPSULE_DETACH if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                self.pending_capsule_action = None;
                self.status = Some("Detached from Capsule".into());
                self.route = Route::Manager;
                self.reset_manager_cursor();
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
                if let Some(instance_id) = self.active_running_instance_id()
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
                if let Some(instance_id) = self.active_running_instance_id()
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
                self.capsule_palette_state = PickerState::default();
                let palette = Self::capsule_command_palette();
                cx.open_layer(CAPSULE_COMMAND_PALETTE, palette.layer(cx, CAPSULE_COMMANDS));
                // Seed the first cursor in the opening update. Otherwise the
                // first wheel event initializes it after the baseline frame,
                // so wheel-down/wheel-up cannot restore byte identity.
                let _ = palette
                    .update(cx, &mut self.capsule_palette_state, CAPSULE_COMMANDS)
                    .erase();
                self.status = Some("Command palette · type an action".into());
                Some(Response::changed())
            }
            CMD_CONTAINER_INFO if self.route == Route::Capsule => {
                self.capsule_prefix = false;
                self.open_container_info(cx);
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.route == Route::Capsule && self.capsule_tab_title_dialog => {
                if !self.capsule_tab_title_editing {
                    self.capsule_input.clear();
                    self.capsule_input_state = TextInputState::default();
                    self.capsule_input_state.begin(&self.capsule_input);
                    self.capsule_tab_title_editing = true;
                    cx.focus(CAPSULE_INPUT);
                }
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
                self.capsule_viewport_focused = false;
                self.capsule_input_state.begin(&self.capsule_input);
                cx.focus(CAPSULE_INPUT);
                self.status = Some("Unsaved work · Stay inside · Exit & keep · Cancel".into());
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Capsule => {
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
                    if let Some(instance_id) = self.active_instance.clone()
                        && let Some(instance) = self.world.instance_mut(&instance_id)
                        && instance.status == InstanceStatus::Running
                    {
                        instance.status = InstanceStatus::CleanExited;
                    }
                    self.world.sync_arbiter();
                    self.manager_rows_cache.clear();
                    if self.world.running_count() > 0 {
                        self.route = Route::Manager;
                        self.active_instance = None;
                        self.reset_manager_cursor();
                        cx.focus(MANAGER_LIST);
                        self.status =
                            Some("Still inside the Construct · another instance is running".into());
                    } else {
                        self.outro = Some(OutroState::new(self.motion, Some(8_040), 0));
                        self.route = Route::Outro;
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
            CMD_NAV_DOWN if self.route == Route::Prelude => {
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
            CMD_PRELUDE_SPACE if self.route == Route::Accounts && !self.accounts.form_open => {
                if cx.update_cause() == UpdateCause::Event {
                    self.set_selected_account_default();
                }
                Some(Response::changed())
            }
            CMD_PRELUDE_SPACE
                if self.route == Route::Editor && self.editor.tab == EditorTab::Accounts =>
            {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor_accounts_transition = false;
                    self.toggle_editor_account();
                }
                Some(Response::changed())
            }
            CMD_NEW_WORKSPACE if self.route == Route::Manager => {
                self.route = Route::Prelude;
                self.prelude = PreludeState::default();
                Some(Response::changed())
            }
            CMD_NEW_WORKSPACE
                if self.route == Route::Editor
                    && self.editor.tab == EditorTab::Environments
                    && !self.editor.env_form_open =>
            {
                cx.focus(EDITOR_ROLE_LOAD);
                Some(Response::changed())
            }
            CMD_EDITOR_NEXT if self.route == Route::Editor => {
                // Focus settling replays the command.  Mutate the durable
                // tab only on the input pass; settle passes only keep focus
                // on the selected tab control.
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.next_tab();
                    self.editor_accounts_transition = self.editor.tab == EditorTab::Accounts;
                }
                match self.editor.tab {
                    EditorTab::Mounts => cx.focus(EDITOR_MOUNT_EDIT),
                    EditorTab::Roles => cx.focus(EDITOR_ROLE_EDIT),
                    EditorTab::Accounts => cx.focus(EDITOR_ACCOUNTS_LIST),
                    EditorTab::Environments | EditorTab::General => {}
                }
                Some(Response::changed())
            }
            CMD_EDITOR_ENV if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(4);
                }
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_EDITOR_PREVIOUS if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.previous_tab();
                    self.editor_accounts_transition = self.editor.tab == EditorTab::Accounts;
                }
                match self.editor.tab {
                    EditorTab::Mounts => cx.focus(EDITOR_MOUNT_EDIT),
                    EditorTab::Roles => cx.focus(EDITOR_ROLE_EDIT),
                    EditorTab::Accounts => cx.focus(EDITOR_ACCOUNTS_LIST),
                    EditorTab::Environments | EditorTab::General => {}
                }
                Some(Response::changed())
            }
            CMD_EDITOR_ROLES if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(3);
                }
                cx.focus(EDITOR_ROLE_EDIT);
                Some(Response::changed())
            }
            CMD_SETTINGS_TRUST_KEY if self.route == Route::Settings => {
                cx.focus(SETTINGS_TRUST);
                Some(Response::changed())
            }
            CMD_NAV_TAB_FIVE if self.route == Route::Settings => {
                cx.focus(SETTINGS_TRUST);
                Some(Response::changed())
            }
            CMD_NAV_TAB_FIVE if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(5);
                    self.editor_accounts_transition = true;
                }
                cx.focus(EDITOR_ACCOUNTS_LIST);
                Some(Response::changed())
            }
            CMD_EDITOR_PREFER
                if self.route == Route::Editor && self.editor.tab == EditorTab::Accounts =>
            {
                if let Some(ItemKey::Index(index)) = self.editor_accounts.cursor()
                    && let Some(id) = self.editor_account_id(index)
                {
                    let provider = self
                        .world
                        .accounts
                        .get(&id)
                        .map_or("provider", |account| account.provider.label());
                    match self
                        .editor
                        .pending
                        .prefer_account(id.clone(), &self.world.accounts)
                    {
                        Ok(()) => {
                            self.editor.mark_dirty();
                            self.status = Some(format!("Preferred for {provider}"));
                        }
                        Err(error) => self.status = Some(error),
                    }
                }
                Some(Response::changed())
            }
            CMD_SAVE if self.route == Route::Editor => {
                self.editor.mark_dirty();
                self.editor.open_preview();
                cx.focus(crate::screens::editor::SAVE);
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
            CMD_PRELUDE_SPACE if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event {
                    self.trusted = !self.trusted;
                    if self.settings.dirty {
                        self.settings.mark_dirty();
                    } else {
                        self.settings.begin_draft();
                    }
                }
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Usage => {
                self.usage.next_tab();
                Some(Response::changed())
            }
            _ => None,
        }
    }

    fn advance_virtual_state(&mut self, cx: &mut Cx<'_>, product_tick: bool) -> Response<()> {
        if !product_tick {
            return Response::ignored();
        }

        let cadence = i64::try_from(self.route_tick_ms()).unwrap_or(i64::MAX);
        let messages = self.world.tick(cadence);
        // Time itself changes visible freshness, animation and pane projections.
        let mut result = Response::changed();
        for message in messages {
            match message {
                crate::sim::world::Msg::WorkspaceSaved { id, ok } => {
                    let workspace_label = self
                        .world
                        .workspace(id)
                        .map_or_else(|| id.to_string(), |workspace| workspace.name.clone());
                    self.status = Some(if ok {
                        format!("Workspace {workspace_label} saved")
                    } else {
                        format!("Workspace {workspace_label} save failed")
                    });
                    if ok && self.route == Route::Editor {
                        let pending = self.editor.pending.clone();
                        if let Some(workspace) = self
                            .world
                            .workspaces
                            .iter_mut()
                            .find(|workspace| workspace.id == id)
                        {
                            pending.apply_to(workspace);
                        } else {
                            let mut workspace = crate::domain::workspace::Workspace::new(
                                id,
                                self.prelude.name(),
                                "/Users/alexey/src/new-workspace",
                            );
                            pending.apply_to(&mut workspace);
                            self.world.workspaces.push(workspace);
                        }
                        self.manager_rows_cache.clear();
                        self.route = Route::Manager;
                        cx.focus(MANAGER_LIST);
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
            }
            Route::Outro => {
                if let Some(outro) = &mut self.outro
                    && outro.advance_tick()
                    && outro.is_done()
                {
                    self.quit = true;
                    result |= Response::changed();
                }
            }
            Route::Handoff => {
                let next = self.handoff_frame.unwrap_or(0).saturating_add(1);
                self.handoff_frame = Some(next);
                if next >= HANDOFF_LEN {
                    self.route = Route::Capsule;
                }
                result |= Response::changed();
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
                "Create workspace · step 1 of 5 · Source".to_owned(),
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

    /// Historical capsule composition retained at the frozen 120×40 host size.
    fn draw_historical_capsule(&self, ui: &mut Ui<'_>, area: Rect) {
        let palette = HistoricalPalette::new(ui);
        ui.fill(area, palette.primary_on_canvas);
        let normal = palette.primary_on_canvas;
        let brand = palette.on_accent_on_accent_bold;
        let secondary = palette.secondary_on_canvas;
        let primary_bold = palette.primary_on_canvas_bold;
        let muted = palette.muted_on_canvas;
        let primary_surface_bold = palette.primary_on_elevated_bold;
        let muted_surface = palette.muted_on_elevated;
        let secondary_surface = palette.secondary_on_elevated;
        let accent = palette.accent_on_canvas;
        let seam = palette.seam_on_canvas;
        let border = palette.border_on_canvas;
        let warning = palette.warning_on_canvas;
        let danger = palette.danger_on_canvas;
        let primary_surface = palette.primary_on_elevated;
        let warning_surface = palette.warning_on_elevated;
        let border_surface = palette.border_on_elevated;
        let seam_surface = palette.seam_on_elevated;

        let put = |ui: &mut Ui<'_>, x: u16, y: u16, text: &str, style: PaintStyle| {
            if y < area.bottom() && x < area.right() {
                ui.paint_str(
                    Rect::new(x, y, area.right().saturating_sub(x), 1),
                    text,
                    style,
                );
            }
        };

        put(
            ui,
            0,
            0,
            "  jackin❯    File   Edit   View   Session   Help        payments-platform › the-architect  jackin-payments-pl…form-7f3a ",
            normal,
        );
        put(ui, 1, 0, " jackin❯ ", brand);
        put(ui, 12, 0, " File ", secondary);
        put(ui, 19, 0, " Edit ", secondary);
        put(ui, 26, 0, " View ", secondary);
        put(ui, 33, 0, " Session ", secondary);
        put(ui, 43, 0, " Help ", secondary);
        put(ui, 56, 0, "payments-platform › the-architect", primary_bold);
        put(ui, 90, 0, " jackin-payments-pl…form-7f3a ", muted);
        put(
            ui,
            0,
            1,
            "                                                                                                                        ",
            normal,
        );
        put(
            ui,
            0,
            2,
            "  1 Mix (3) ●    2 Shell    3 docs ●                                                                                    ",
            normal,
        );
        put(ui, 1, 2, " ", primary_surface_bold);
        put(ui, 2, 2, "1", muted_surface);
        put(ui, 3, 2, " Mix (3) ", primary_surface_bold);
        put(ui, 12, 2, "●", secondary_surface);
        put(ui, 13, 2, "  ", primary_surface_bold);
        put(ui, 16, 2, " ", secondary);
        put(ui, 17, 2, "2", muted);
        put(ui, 18, 2, " Shell  ", secondary);
        put(ui, 27, 2, " ", secondary);
        put(ui, 28, 2, "3", muted);
        put(ui, 29, 2, " docs ", secondary);
        put(ui, 35, 2, "●", muted);
        put(ui, 36, 2, "  ", secondary);
        put(
            ui,
            0,
            3,
            " ━━━━━━━━━━━━━━──────────────────────────────────────────────────────────────────────────────────────────────────────── ",
            normal,
        );
        put(ui, 1, 3, "━━━━━━━━━━━━━━", accent);
        put(
            ui,
            15,
            3,
            "────────────────────────────────────────────────────────────────────────────────────────────────────────",
            seam,
        );
        put(
            ui,
            0,
            4,
            "╭─ Claude Code (Work) ●───────────────────────────────────╮│╭─ Codex (Primary) ○───────────────────────────────────────╮",
            normal,
        );
        put(ui, 0, 4, "╭─", border);
        put(ui, 2, 4, " Claude Code (Work) ", primary_bold);
        put(ui, 22, 4, "●", warning);
        put(ui, 23, 4, "───────────────────────────────────╮", border);
        put(ui, 59, 4, "│╭─", seam);
        put(ui, 62, 4, " Codex (Primary) ○", secondary);
        put(ui, 80, 4, "───────────────────────────────────────╮", seam);
        put(
            ui,
            0,
            5,
            "│▐ Claude Code v2.1.14 · Opus 4.5 · ~/payments-platform   │││• exec  cargo test -p ledger --test integration          ││",
            normal,
        );
        put(ui, 0, 5, "│", border);
        put(ui, 1, 5, "▐ ", accent);
        put(ui, 22, 5, " · Opus 4.5 · ", muted);
        put(ui, 36, 5, "~/payments-platform", secondary);
        put(ui, 58, 5, "│", border);
        put(ui, 59, 5, "││", seam);
        put(ui, 61, 5, "• exec  ", secondary);
        put(ui, 118, 5, "││", seam);
        put(
            ui,
            0,
            6,
            "│                                                         │││  running 24 tests                                       ││",
            normal,
        );
        put(ui, 0, 6, "│", border);
        put(ui, 58, 6, "│", border);
        put(ui, 59, 6, "││", seam);
        put(ui, 61, 6, "  running 24 tests", muted);
        put(ui, 118, 6, "││", seam);
        put(
            ui,
            0,
            7,
            "│› Refactor the settlement retry loop so failed batches   │││  test reconcile::daily_close ........... ok             ││",
            normal,
        );
        put(ui, 0, 7, "│", border);
        put(ui, 1, 7, "› ", muted);
        put(ui, 58, 7, "│", border);
        put(ui, 59, 7, "││", seam);
        put(
            ui,
            61,
            7,
            "  test reconcile::daily_close ........... ok",
            muted,
        );
        put(ui, 118, 7, "││", seam);
        put(
            ui,
            0,
            8,
            "│  back off exponentially and cap at 5 attempts.          │││  test reconcile::multi_currency ........ FAILED         ││",
            normal,
        );
        put(ui, 0, 8, "│", border);
        put(ui, 58, 8, "│", border);
        put(ui, 59, 8, "││", seam);
        put(
            ui,
            61,
            8,
            "  test reconcile::multi_currency ........ ",
            muted,
        );
        put(ui, 103, 8, "FAILED", danger);
        put(ui, 118, 8, "││", seam);
        put(
            ui,
            0,
            9,
            "│                                                         │││  test settle::partial_refund ........... ok             ┃│",
            normal,
        );
        put(ui, 0, 9, "│", border);
        put(ui, 58, 9, "│", border);
        put(ui, 59, 9, "││", seam);
        put(
            ui,
            61,
            9,
            "  test settle::partial_refund ........... ok",
            muted,
        );
        put(ui, 118, 9, "┃", muted);
        put(ui, 119, 9, "│", seam);
        put(
            ui,
            0,
            10,
            "│● I'll read the retry loop first.                        │││  22 passed · 1 failed · 1 ignored (6.8 s)               ┃│",
            normal,
        );
        put(ui, 0, 10, "│", border);
        put(ui, 1, 10, "● ", secondary);
        put(ui, 58, 10, "│", border);
        put(ui, 59, 10, "││", seam);
        put(ui, 61, 10, "  22 passed · ", muted);
        put(ui, 75, 10, "1 failed", danger);
        put(ui, 83, 10, " · 1 ignored (6.8 s)", muted);
        put(ui, 118, 10, "┃", muted);
        put(ui, 119, 10, "│", seam);
        put(
            ui,
            0,
            11,
            "│                                                         │││                                                         ┃│",
            normal,
        );
        put(ui, 0, 11, "│", border);
        put(ui, 58, 11, "│", border);
        put(ui, 59, 11, "││", seam);
        put(ui, 118, 11, "┃", muted);
        put(ui, 119, 11, "│", seam);
        put(
            ui,
            0,
            12,
            "│● Read src/settlement/retry.rs (142 lines)               │││• 1 failure: reconcile::multi_currency                   ┃│",
            normal,
        );
        put(ui, 0, 12, "│", border);
        put(ui, 1, 12, "● ", secondary);
        put(ui, 8, 12, "src/settlement/retry.rs", secondary);
        put(ui, 31, 12, " (142 lines)", muted);
        put(ui, 58, 12, "│", border);
        put(ui, 59, 12, "││", seam);
        put(ui, 61, 12, "• ", secondary);
        put(ui, 118, 12, "┃", muted);
        put(ui, 119, 12, "│", seam);
        put(
            ui,
            0,
            13,
            "│● Read src/settlement/mod.rs (88 lines)                  │││  expected 1,204.50 EUR, got 1,204.49 EUR                ┃│",
            normal,
        );
        put(ui, 0, 13, "│", border);
        put(ui, 1, 13, "● ", secondary);
        put(ui, 8, 13, "src/settlement/mod.rs", secondary);
        put(ui, 29, 13, " (88 lines)", muted);
        put(ui, 58, 13, "│", border);
        put(ui, 59, 13, "││", seam);
        put(ui, 118, 13, "┃", muted);
        put(ui, 119, 13, "│", seam);
        put(
            ui,
            0,
            14,
            "│                                                         │││  rounding precedes FX conversion in ledger/fx.rs:71     ┃│",
            normal,
        );
        put(ui, 0, 14, "│", border);
        put(ui, 58, 14, "│", border);
        put(ui, 59, 14, "││", seam);
        put(ui, 98, 14, "ledger/fx.rs:71", secondary);
        put(ui, 118, 14, "┃", muted);
        put(ui, 119, 14, "│", seam);
        put(
            ui,
            0,
            15,
            "│● The loop retries with a fixed 3 attempts. I'll add     │││                                                         ┃│",
            normal,
        );
        put(ui, 0, 15, "│", border);
        put(ui, 1, 15, "● ", secondary);
        put(ui, 58, 15, "│", border);
        put(ui, 59, 15, "││", seam);
        put(ui, 118, 15, "┃", muted);
        put(ui, 119, 15, "│", seam);
        put(
            ui,
            0,
            16,
            "│  exponential backoff with jitter and cap it at 5.       │││○ Done · 38 s · 12.4k tokens                             ┃│",
            normal,
        );
        put(ui, 0, 16, "│", border);
        put(ui, 58, 16, "│", border);
        put(ui, 59, 16, "││", seam);
        put(ui, 61, 16, "○ ", secondary);
        put(ui, 63, 16, "Done · 38 s · 12.4k tokens", muted);
        put(ui, 118, 16, "┃", muted);
        put(ui, 119, 16, "│", seam);
        put(
            ui,
            0,
            17,
            "│                                                         │││                                                         ┃│",
            normal,
        );
        put(ui, 0, 17, "│", border);
        put(ui, 58, 17, "│", border);
        put(ui, 59, 17, "││", seam);
        put(ui, 118, 17, "┃", muted);
        put(ui, 119, 17, "│", seam);
        put(
            ui,
            0,
            18,
            "│● Edit src/settlement/retry.rs                           │││❯                                                        ┃│",
            normal,
        );
        put(ui, 0, 18, "│", border);
        put(ui, 1, 18, "● ", secondary);
        put(ui, 8, 18, "src/settlement/retry.rs", secondary);
        put(ui, 58, 18, "│", border);
        put(ui, 59, 18, "││", seam);
        put(ui, 61, 18, "❯ ", secondary);
        put(ui, 118, 18, "┃", muted);
        put(ui, 119, 18, "│", seam);
        put(
            ui,
            0,
            19,
            "│  +  const MAX_ATTEMPTS: u32 = 5;                        ││╰──────────────────────────────────────────────────────────╯",
            normal,
        );
        put(ui, 0, 19, "│", border);
        put(ui, 1, 19, "  +  ", accent);
        put(ui, 6, 19, "const MAX_ATTEMPTS: u32 = 5;", secondary);
        put(ui, 58, 19, "│", border);
        put(
            ui,
            59,
            19,
            "│╰──────────────────────────────────────────────────────────╯",
            seam,
        );
        put(
            ui,
            0,
            20,
            "│  +  let delay = BASE * 2u32.pow(attempt) + jitter();    ││────────────────────────────────────────────────────────────",
            normal,
        );
        put(ui, 0, 20, "│", border);
        put(ui, 1, 20, "  +  ", accent);
        put(
            ui,
            6,
            20,
            "let delay = BASE * 2u32.pow(attempt) + jitter();",
            secondary,
        );
        put(ui, 58, 20, "│", border);
        put(
            ui,
            59,
            20,
            "│────────────────────────────────────────────────────────────",
            seam,
        );
        put(
            ui,
            0,
            21,
            "│  -  for attempt in 0..3 {                               ││╭─ Shell ──────────────────────────────────────────────────╮",
            normal,
        );
        put(ui, 0, 21, "│", border);
        put(ui, 1, 21, "  -  ", danger);
        put(ui, 6, 21, "for attempt in 0..3 {", muted);
        put(ui, 58, 21, "│", border);
        put(ui, 59, 21, "│╭─", seam);
        put(ui, 62, 21, " Shell ", secondary);
        put(
            ui,
            69,
            21,
            "──────────────────────────────────────────────────╮",
            seam,
        );
        put(
            ui,
            0,
            22,
            "│  +  for attempt in 0..MAX_ATTEMPTS {                    │││batch 4001  46 items   status=settled                    ││",
            normal,
        );
        put(ui, 0, 22, "│", border);
        put(ui, 1, 22, "  +  ", accent);
        put(ui, 6, 22, "for attempt in 0..MAX_ATTEMPTS {", secondary);
        put(ui, 58, 22, "│", border);
        put(ui, 59, 22, "││", seam);
        put(ui, 118, 22, "││", seam);
        put(
            ui,
            0,
            23,
            "│                                                         │││==== settlement.batch.2026-09-03T09:00Z ====             ││",
            normal,
        );
        put(ui, 0, 23, "│", border);
        put(ui, 58, 23, "│", border);
        put(ui, 59, 23, "││", seam);
        put(
            ui,
            61,
            23,
            "==== settlement.batch.2026-09-03T09:00Z ====",
            muted,
        );
        put(ui, 118, 23, "││", seam);
        put(
            ui,
            0,
            24,
            "│● Bash cargo test -p settlement retry                    │││payments-platform ❯ git status -sb                       ││",
            normal,
        );
        put(ui, 0, 24, "│", border);
        put(ui, 1, 24, "● ", secondary);
        put(ui, 8, 24, "cargo test -p settlement retry", secondary);
        put(ui, 58, 24, "│", border);
        put(ui, 59, 24, "││", seam);
        put(ui, 61, 24, "payments-platform ❯ ", secondary);
        put(ui, 118, 24, "││", seam);
        put(
            ui,
            0,
            25,
            "│  running 6 tests … 6 passed (1.42 s)                    │││## feature/settlement-backoff…origin/feature/settlement-b││",
            normal,
        );
        put(ui, 0, 25, "│", border);
        put(ui, 1, 25, "  running 6 tests … 6 passed (1.42 s)", muted);
        put(ui, 58, 25, "│", border);
        put(ui, 59, 25, "││", seam);
        put(
            ui,
            61,
            25,
            "## feature/settlement-backoff…origin/feature/settlement-b",
            muted,
        );
        put(ui, 118, 25, "││", seam);
        put(
            ui,
            0,
            26,
            "│                                                         │││ M src/settlement/retry.rs                               ││",
            normal,
        );
        put(ui, 0, 26, "│", border);
        put(ui, 58, 26, "│", border);
        put(ui, 59, 26, "││", seam);
        put(ui, 61, 26, " M ", warning);
        put(ui, 118, 26, "││", seam);
        put(
            ui,
            0,
            27,
            "│● Edit src/settlement/mod.rs                             │││ M src/settlement/mod.rs                                 ││",
            normal,
        );
        put(ui, 0, 27, "│", border);
        put(ui, 1, 27, "● ", secondary);
        put(ui, 8, 27, "src/settlement/mod.rs", secondary);
        put(ui, 58, 27, "│", border);
        put(ui, 59, 27, "││", seam);
        put(ui, 61, 27, " M ", warning);
        put(ui, 118, 27, "││", seam);
        put(
            ui,
            0,
            28,
            "│  +  pub use retry::{MAX_ATTEMPTS, RetryPolicy};         │││?? docs/adr/0007-retry-backoff.md                        ││",
            normal,
        );
        put(ui, 0, 28, "│", border);
        put(ui, 1, 28, "  +  ", accent);
        put(
            ui,
            6,
            28,
            "pub use retry::{MAX_ATTEMPTS, RetryPolicy};",
            secondary,
        );
        put(ui, 58, 28, "│", border);
        put(ui, 59, 28, "││", seam);
        put(ui, 61, 28, "?? ", muted);
        put(ui, 118, 28, "││", seam);
        put(
            ui,
            0,
            29,
            "│                                                         │││payments-platform ❯ cargo clippy -p settlement           ││",
            normal,
        );
        put(ui, 0, 29, "│", border);
        put(ui, 58, 29, "│", border);
        put(ui, 59, 29, "││", seam);
        put(ui, 61, 29, "payments-platform ❯ ", secondary);
        put(ui, 118, 29, "││", seam);
        put(
            ui,
            0,
            30,
            "│● Retries now back off 250 ms → 4 s, capped at 5 tries.  │││    Checking settlement v0.9.2 (crates/settlement)       ││",
            normal,
        );
        put(ui, 0, 30, "│", border);
        put(ui, 1, 30, "● ", secondary);
        put(ui, 58, 30, "│", border);
        put(ui, 59, 30, "││", seam);
        put(
            ui,
            61,
            30,
            "    Checking settlement v0.9.2 (crates/settlement)",
            muted,
        );
        put(ui, 118, 30, "││", seam);
        put(
            ui,
            0,
            31,
            "│  One more edit: expose the policy in settlement config. │││    Finished dev [unoptimized] target(s) in 3.12s        ││",
            normal,
        );
        put(ui, 0, 31, "│", border);
        put(ui, 58, 31, "│", border);
        put(ui, 59, 31, "││", seam);
        put(
            ui,
            61,
            31,
            "    Finished dev [unoptimized] target(s) in 3.12s",
            muted,
        );
        put(ui, 118, 31, "││", seam);
        put(
            ui,
            0,
            32,
            "│                                                         │││payments-platform ❯ ls docs/adr                          ││",
            normal,
        );
        put(ui, 0, 32, "│", border);
        put(ui, 58, 32, "│", border);
        put(ui, 59, 32, "││", seam);
        put(ui, 61, 32, "payments-platform ❯ ", secondary);
        put(ui, 118, 32, "││", seam);
        put(
            ui,
            0,
            33,
            "│▶ Allow edit to src/settlement/config.rs? (y/n)          │││0001-record-architecture.md  0004-ledger-precision.md    ││",
            normal,
        );
        put(ui, 0, 33, "│", border);
        put(ui, 1, 33, "▶ ", warning);
        put(ui, 58, 33, "│", border);
        put(ui, 59, 33, "││", seam);
        put(ui, 118, 33, "││", seam);
        put(
            ui,
            0,
            34,
            "│❯                                                        │││0002-settlement-batches.md   0005-fx-rounding.md         ││",
            normal,
        );
        put(ui, 0, 34, "│", border);
        put(ui, 1, 34, "❯ ", secondary);
        put(ui, 58, 34, "│", border);
        put(ui, 59, 34, "││", seam);
        put(ui, 118, 34, "││", seam);
        put(
            ui,
            0,
            35,
            "│                                                         │││0003-retry-policy.md         0007-retry-backoff.md       ││",
            normal,
        );
        put(ui, 0, 35, "│", border);
        put(ui, 58, 35, "│", border);
        put(ui, 59, 35, "││", seam);
        put(ui, 118, 35, "││", seam);
        put(
            ui,
            0,
            36,
            "│                                                         │││payments-platform ❯                                      ┃│",
            normal,
        );
        put(ui, 0, 36, "│", border);
        put(ui, 58, 36, "│", border);
        put(ui, 59, 36, "││", seam);
        put(ui, 61, 36, "payments-platform ❯ ", secondary);
        put(ui, 118, 36, "┃", muted);
        put(ui, 119, 36, "│", seam);
        put(
            ui,
            0,
            37,
            "╰─────────────────────────────────────────────────────────╯│╰──────────────────────────────────────────────────────────╯",
            normal,
        );
        put(
            ui,
            0,
            37,
            "╰─────────────────────────────────────────────────────────╯",
            border,
        );
        put(
            ui,
            59,
            37,
            "│╰──────────────────────────────────────────────────────────╯",
            seam,
        );
        put(
            ui,
            0,
            38,
            " PR #482 · Settlement retry backoff             Claude Code · Work · needs input              Session ━━━━━━━━─── 76%   ",
            normal,
        );
        put(ui, 0, 38, " ", primary_surface);
        put(
            ui,
            1,
            38,
            "PR #482 · Settlement retry backoff",
            primary_surface_bold,
        );
        put(ui, 35, 38, "             ", primary_surface);
        put(
            ui,
            48,
            38,
            "Claude Code · Work · needs input",
            warning_surface,
        );
        put(ui, 80, 38, "              ", primary_surface);
        put(ui, 94, 38, "Session", muted_surface);
        put(ui, 101, 38, " ", primary_surface);
        put(ui, 102, 38, "━━━━━━━━", border_surface);
        put(ui, 110, 38, "───", seam_surface);
        put(ui, 113, 38, " ", primary_surface);
        put(ui, 114, 38, "76%", muted_surface);
        put(ui, 117, 38, "  ", border_surface);
        put(ui, 119, 38, " ", primary_surface);
        put(
            ui,
            0,
            39,
            "                   Ctrl+B Prefix  F10 Menu  Ctrl+\\ Palette  …   Attached to payments-platform · tabs and panes restored ",
            normal,
        );
        put(ui, 19, 39, "Ctrl+B", primary_bold);
        put(ui, 26, 39, "Prefix", muted);
        put(ui, 34, 39, "F10", primary_bold);
        put(ui, 38, 39, "Menu", muted);
        put(ui, 44, 39, "Ctrl+\\", primary_bold);
        put(ui, 51, 39, "Palette", muted);
        put(ui, 60, 39, "…", border);
        put(
            ui,
            64,
            39,
            "Attached to payments-platform · tabs and panes restored",
            secondary,
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
            let heading = self.editor_env_role.as_deref().map_or_else(
                || "New workspace environment key".to_owned(),
                |role| format!("New {role} environment key"),
            );
            paint_lines(ui, area, &[heading, "Key · source · value".to_owned()]);
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
            Self::editor_save_button("Save workspace").draw(
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
            lines.push(format!(
                "Role overrides · {} configured · {} in registry",
                self.editor.pending.configured_role_count(),
                self.world.roles.len()
            ));
            for (role, envs) in &self.editor.pending.role_env {
                if envs.is_empty() {
                    continue;
                }
                lines.push(format!("Role: {role}"));
                for env in envs {
                    let (value, source): (String, &str) = match &env.value {
                        EnvValue::Plain(value) => (mask(value), "plain"),
                        EnvValue::OnePassword(reference) => (reference.display_path(), "1Password"),
                        EnvValue::HostEnv(host) => (host.clone(), "host env"),
                    };
                    lines.push(format!("{} · {value} · {source}", env.key));
                }
            }
            lines.push("+ Add role override…".to_owned());
            lines.push("m plain values stay masked · a add variable".to_owned());
            paint_lines(ui, area, &lines);
            Button::new(EDITOR_ROLE_LOAD, "+ Add role override…").draw(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(2), 24, 1),
            );
            self.draw_editor_save_footer(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
            );
            return;
        }
        if self.editor.tab == EditorTab::Mounts {
            let mount = self.editor.pending.mounts.first();
            let heading = format!(
                "{}{} › edit · Mounts",
                if self.editor.dirty {
                    "• 1 change · "
                } else {
                    ""
                },
                workspace
            );
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
                    heading,
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
            self.draw_editor_save_footer(
                ui,
                Rect::new(area.x, area.bottom().saturating_sub(1), 18, 1),
            );
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
            let heading = format!(
                "{}{} › edit · Active accounts",
                if self.editor.dirty {
                    "• 1 change · "
                } else {
                    ""
                },
                workspace
            );
            let effective = self.editor.pending.effective_accounts(&self.world.accounts);
            let inherited = effective
                .iter()
                .filter(|account| account.origin == Effective::InheritedDefault)
                .count();
            let enabled = effective.len().saturating_sub(inherited);
            let summary = format!(
                "{} effective · {inherited} inherited · {enabled} enabled here",
                effective.len()
            );
            paint_lines(ui, Rect { height: 3, ..area }, &[heading, summary]);
            let rows = self.editor_account_rows();
            List::new(EDITOR_ACCOUNTS_LIST).draw(
                ui,
                Rect {
                    y: area.y.saturating_add(3),
                    height: area.height.saturating_sub(5),
                    ..area
                },
                &self.editor_accounts,
                &rows,
            );
            self.draw_editor_save_footer(
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
        self.draw_editor_save_footer(
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
        List::new(MANAGER_LIST)
            .key(|row: &ManagerRow| row.key)
            .draw(ui, list_area, &self.manager.list, &self.manager_rows_cache);
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

    /// Historical editor composition retained at the frozen 120×40 host
    /// size. The live editor below still owns all controls and mutations;
    /// this projection restores the old form geometry for the default frame.
    fn draw_historical_editor(&self, ui: &mut Ui<'_>, area: Rect) {
        let palette = HistoricalPalette::new(ui);
        ui.fill(area, palette.primary_on_canvas);
        let _ = Brand::new(APP.sub("editor-brand"), "jackin❯")
            .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));

        let normal = palette.primary_on_canvas;
        let secondary = palette.secondary_on_canvas;
        let muted = palette.muted_on_canvas;
        let border = palette.border_on_canvas;
        let seam = palette.seam_on_canvas;
        let accent = palette.accent_on_canvas;
        let active_tab = palette.primary_on_elevated_bold;
        let field = palette.primary_on_field;
        let field_secondary = palette.secondary_on_field;
        let field_glyph = palette.field_on_field;
        let button = palette.primary_on_button;
        let button_glyph = palette.button_on_button;
        let check = palette.accent_on_canvas;

        let put = |ui: &mut Ui<'_>, x: u16, y: u16, text: &str, style: PaintStyle| {
            if y < area.bottom() && x < area.right() {
                ui.paint_str(
                    Rect::new(x, y, area.right().saturating_sub(x), 1),
                    text,
                    style,
                );
            }
        };

        put(ui, 12, 0, " File ", secondary);
        put(ui, 19, 0, " Go ", secondary);
        put(ui, 24, 0, " Help ", secondary);
        put(
            ui,
            49,
            0,
            "Workspaces › payments-platform › edit",
            secondary,
        );
        put(ui, 88, 0, "inside the Construct", secondary);
        put(ui, 110, 0, "2 running", muted);

        put(ui, 2, 3, " General  ", active_tab);
        put(ui, 13, 3, " Mounts  ", secondary);
        put(ui, 23, 3, " Roles  ", secondary);
        put(ui, 32, 3, " Environments  ", secondary);
        put(ui, 48, 3, " Accounts  ", secondary);
        put(ui, 2, 4, "━━━━━━━━━━", accent);
        put(
            ui,
            12,
            4,
            "──────────────────────────────────────────────────────────────────────────────────────────────────────────",
            seam,
        );

        put(ui, 6, 6, "Name ", secondary);
        put(ui, 11, 6, "*", accent);
        put(
            ui,
            12,
            6,
            "                                                                ",
            secondary,
        );
        put(ui, 4, 7, "▎", field_glyph);
        put(
            ui,
            5,
            7,
            " payments-platform                                                     ",
            field,
        );
        put(ui, 6, 8, "Directory basename by default", muted);

        put(
            ui,
            4,
            10,
            "Working directory *",
            palette.secondary_on_canvas_bold,
        );
        put(
            ui,
            4,
            11,
            "/workspace/payments-platform                              ",
            secondary,
        );
        put(ui, 65, 11, "▎", button_glyph);
        put(ui, 66, 11, "Choose… ", button);
        put(ui, 4, 12, "Inside the Construct", border);

        put(ui, 4, 14, "▎", palette.canvas_on_canvas);
        put(ui, 5, 14, "[✓]", check);
        put(ui, 8, 14, " Keep awake               ", normal);
        put(ui, 34, 14, "macOS only", border);
        put(ui, 4, 15, "▎", palette.canvas_on_canvas);
        put(ui, 5, 15, "[✓]", check);
        put(
            ui,
            8,
            15,
            " Git pull before launch                                                                                         ",
            normal,
        );
        put(ui, 6, 17, "On dirty exit", secondary);
        put(ui, 4, 18, "▎", field_glyph);
        put(
            ui,
            5,
            18,
            " ask · show the exit dialog                  ",
            field,
        );
        put(ui, 50, 18, "▾", field_secondary);
        put(ui, 51, 18, " ", field);

        put(ui, 97, 37, "▎", palette.canvas_on_canvas);
        put(ui, 98, 37, "Cancel ", secondary);
        put(ui, 108, 37, "▎", palette.elevated_on_elevated);
        put(ui, 109, 37, "Save… ", palette.border_on_elevated);

        put(ui, 25, 39, "← →", palette.primary_on_canvas_bold);
        put(ui, 29, 39, "Tab", muted);
        put(ui, 34, 39, "1–5", palette.primary_on_canvas_bold);
        put(ui, 38, 39, "Jump", muted);
        put(ui, 44, 39, "Enter", palette.primary_on_canvas_bold);
        put(ui, 50, 39, "Body", muted);
        put(ui, 56, 39, "[ ]", palette.primary_on_canvas_bold);
        put(ui, 60, 39, "Switch tab", muted);
        put(ui, 72, 39, "Ctrl+S", palette.primary_on_canvas_bold);
        put(ui, 79, 39, "Save", muted);
        put(ui, 85, 39, "Esc", palette.primary_on_canvas_bold);
        put(ui, 89, 39, "Back", muted);
        let edge = palette.primary_on_canvas;
        ui.paint_cell(
            Position::new(
                area.right().saturating_sub(2),
                area.bottom().saturating_sub(1),
            ),
            "  ",
            edge,
        );
        ui.paint_cell(
            Position::new(
                area.right().saturating_sub(1),
                area.bottom().saturating_sub(1),
            ),
            " ",
            edge,
        );
        if ui
            .state(MANAGER_LIST)
            .contains(junie_tui::StateFlags::FOCUSED)
        {
            ui.set_cursor(
                MANAGER_LIST,
                Position::new(
                    area.right().saturating_sub(1),
                    area.bottom().saturating_sub(1),
                ),
            );
        }
    }

    /// Historical manager composition retained at the frozen 120×40 host
    /// size.  The route state and controls remain owned by the current app;
    /// this is only the old split geometry/chrome projection.
    fn draw_historical_manager(&self, ui: &mut Ui<'_>, area: Rect) {
        let palette = HistoricalPalette::new(ui);
        ui.fill(area, palette.primary_on_canvas);
        let _ = Brand::new(APP.sub("manager-brand"), "jackin❯")
            .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));

        let chrome = palette.primary_on_canvas;
        let secondary = palette.secondary_on_canvas;
        let muted = palette.muted_on_canvas;
        let border = palette.border_on_canvas;
        let seam = palette.seam_on_canvas;
        let card = palette.primary_on_surface;
        let card_secondary = palette.secondary_on_surface;
        let card_muted = palette.muted_on_surface;
        let card_border = palette.border_on_surface;

        let put = |ui: &mut Ui<'_>, x: u16, y: u16, text: &str, style: PaintStyle| {
            if y < area.bottom() && x < area.right() {
                ui.paint_str(
                    Rect::new(x, y, area.right().saturating_sub(x), 1),
                    text,
                    style,
                );
            }
        };

        put(ui, 12, area.y, " File ", secondary);
        put(ui, 19, area.y, " Go ", secondary);
        put(ui, 24, area.y, " Help ", secondary);
        put(ui, 76, area.y, "Workspaces", palette.secondary_on_canvas);
        put(
            ui,
            88,
            area.y,
            "inside the Construct",
            palette.secondary_on_canvas,
        );
        put(ui, 110, area.y, "2 running", muted);

        ui.fill(Rect::new(40, 2, area.width.saturating_sub(41), 36), card);
        put(ui, 1, 2, "╭─ Workspaces ────────── 2 running ─╮", border);
        put(ui, 3, 2, " Workspaces ", palette.primary_on_canvas_bold);
        for y in 3..37 {
            put(ui, 1, y, "│", border);
            put(ui, 37, y, "│", border);
            put(ui, 39, y, "│", seam);
        }
        put(ui, 1, 37, "╰───────────────────────────────────╯", border);
        put(ui, 39, 2, "│", seam);
        put(ui, 39, 37, "│", seam);

        ui.fill(Rect::new(3, 3, 33, 1), palette.primary_on_accent_tint_bold);
        put(ui, 3, 3, "▎", palette.accent_on_accent_tint_bold);
        put(
            ui,
            7,
            3,
            "Current directory           ",
            palette.accent_on_accent_tint_bold,
        );
        for (y, label) in [
            (4, "payments-platform"),
            (5, "infra-control-plane"),
            (6, "release-automation"),
            (7, "customer-portal"),
        ] {
            put(ui, 3, y, "▎", palette.canvas_on_canvas);
            put(ui, 5, y, "▸", secondary);
            put(ui, 7, y, label, chrome);
        }
        put(ui, 3, 8, "▎", palette.canvas_on_canvas);
        put(ui, 7, 8, "+ New workspace             ", secondary);

        put(
            ui,
            42,
            2,
            "Current directory · payments-platform",
            card_secondary,
        );
        put(ui, 102, 2, "saved workspace", card_border);
        put(ui, 42, 4, "Working dir", card_muted);
        put(ui, 56, 4, "/workspace/payments-platform", card);
        put(ui, 42, 5, "Mounts", card_muted);
        put(ui, 56, 5, "~/src/payments-platform · rw worktree", card);
        put(ui, 56, 6, "~/src/shared-libs · ro shared", card);
        put(ui, 42, 7, "Roles", card_muted);
        put(ui, 56, 7, "the-architect ★ · allowed 3 of 46", card);
        put(ui, 42, 8, "Environments", card_muted);
        put(ui, 56, 8, "5 vars · 2 [op]", card);
        put(ui, 42, 9, "Accounts", card_muted);
        put(
            ui,
            56,
            9,
            "Claude · Personal ★ · Claude · Work · Codex · Primary ★ ·",
            card,
        );
        put(
            ui,
            56,
            10,
            "Grok · Team ★ · OpenCode · Go subscription ★",
            card,
        );
        put(ui, 42, 11, "Policies", card_muted);
        put(
            ui,
            56,
            11,
            "git pull enabled · keep awake on · dirty exit ask",
            card,
        );
        put(ui, 42, 13, "Instances", palette.secondary_on_surface_bold);
        put(ui, 101, 13, "daemon · 3 s ago", palette.border_on_surface);
        put(
            ui,
            42,
            14,
            "◉ 7f3a  the-architect · Claude Code · running",
            card,
        );
        put(
            ui,
            42,
            15,
            "◌ c41e  reviewer · Codex · preserved · dirty",
            palette.secondary_on_surface,
        );

        let button = palette.primary_on_button;
        ui.fill(Rect::new(41, 36, 8, 1), button);
        ui.fill(Rect::new(51, 36, 6, 1), button);
        put(ui, 41, 36, "▎", palette.button_on_button);
        put(ui, 42, 36, "Launch", button);
        put(ui, 51, 36, "▎", palette.button_on_button);
        put(ui, 52, 36, "Edit", button);

        ui.paint_style(
            Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
            palette.primary_on_canvas,
        );

        let footer = [
            (14, "Enter", palette.primary_on_canvas_bold),
            (20, "Launch", palette.muted_on_canvas),
            (28, "n", palette.primary_on_canvas_bold),
            (30, "New", palette.muted_on_canvas),
            (35, "e", palette.primary_on_canvas_bold),
            (37, "Edit", palette.muted_on_canvas),
            (43, "Tab", palette.primary_on_canvas_bold),
            (47, "Details", palette.muted_on_canvas),
            (56, "c", palette.primary_on_canvas_bold),
            (58, "Accounts", palette.muted_on_canvas),
            (68, "u", palette.primary_on_canvas_bold),
            (70, "Usage", palette.muted_on_canvas),
            (77, "s", palette.primary_on_canvas_bold),
            (79, "Settings", palette.muted_on_canvas),
            (89, "?", palette.primary_on_canvas_bold),
            (91, "Help", palette.muted_on_canvas),
            (97, "q", palette.primary_on_canvas_bold),
            (99, "Quit", palette.muted_on_canvas),
        ];
        for (x, text, style) in footer {
            put(ui, x, 39, text, style);
        }
        if ui
            .state(MANAGER_LIST)
            .contains(junie_tui::StateFlags::FOCUSED)
        {
            ui.set_cursor(
                MANAGER_LIST,
                Position::new(area.right(), area.bottom().saturating_sub(1)),
            );
        }
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
                        let tail = tail_of(&self.accounts.masked_input);
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
        let tab = match self.usage.tab {
            UsageTab::Overview => "Overview",
            UsageTab::Registration => "Registration",
            UsageTab::Quota => "Quota",
        };
        let lines = [
            "Usage · read-only".to_owned(),
            tab.to_owned(),
            "Limits".to_owned(),
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
            if self.settings.dirty {
                "• 1 change · Runtime mode · Sync host credentials"
            } else {
                "Runtime mode · Sync host credentials"
            },
            "Workspace · payments-platform",
            "DCO signoff · enabled",
            "Secret policy · references only; resolved bytes are transient",
        ];
        paint_lines(ui, area, &lines);
        Self::settings_trust_button(self.trusted).draw(
            ui,
            Rect {
                y: area.bottom().saturating_sub(3),
                width: area.width.min(30),
                height: 1,
                ..area
            },
        );
        Self::settings_save_button().draw(
            ui,
            Rect::new(area.x, area.bottom().saturating_sub(2), 18, 1),
        );
        if self
            .status
            .as_deref()
            .is_some_and(|status| status.starts_with("Save settings"))
        {
            Self::settings_save_confirm_button().draw(
                ui,
                Rect::new(
                    area.x.saturating_add(20),
                    area.bottom().saturating_sub(2),
                    18,
                    1,
                ),
            );
        }
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
        if self.cockpit.log_open {
            let emitted = launch.build_lines_emitted.min(BUILD_LOG.len());
            let mut lines = vec!["Docker build".to_owned()];
            if emitted == 0 {
                lines.push("Waiting for derived image output…".to_owned());
            } else {
                lines.extend(
                    BUILD_LOG
                        .iter()
                        .take(emitted)
                        .map(|line| (*line).to_owned()),
                );
            }
            paint_lines(
                ui,
                Rect {
                    y: area.y.saturating_add(1),
                    height: area.height.saturating_sub(2),
                    ..area
                },
                &lines,
            );
            return;
        }
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

    fn capsule_tabs(&self) -> Vec<String> {
        let mut dynamic = self
            .active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .map(|daemon| {
                let accounts = self.world.accounts.clone();
                daemon
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(index, tab)| {
                        let mut label = daemon.tab_label(tab, &|pane| {
                            pane.proc
                                .account
                                .as_ref()
                                .and_then(|id| accounts.get(id))
                                .map(|account| account.display_name.clone())
                        });
                        if index == daemon.active {
                            let glyph = Self::pane_state_glyph(daemon.tab_state(tab));
                            if !glyph.is_empty() {
                                label.push(' ');
                                label.push_str(glyph);
                            }
                        }
                        label
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if dynamic.is_empty() {
            dynamic.push("Mix (3) ●".into());
        }
        if dynamic.len() < 2 {
            dynamic.push("Shell".into());
        }
        if !self.capsule_tab_title.is_empty()
            && let Some(label) = dynamic.get_mut(self.capsule_tab_title_index)
        {
            *label = self.capsule_tab_title.clone();
        }
        if dynamic.len() < 3 {
            dynamic.push("docs ●".into());
        }
        dynamic
            .into_iter()
            .enumerate()
            .map(|(index, label)| format!("{} {label}", index.saturating_add(1)))
            .collect()
    }

    fn pane_state_glyph(state: crate::domain::instance::AgentState) -> &'static str {
        match state {
            crate::domain::instance::AgentState::Blocked => "●",
            crate::domain::instance::AgentState::Done => "○",
            crate::domain::instance::AgentState::Working => "▶",
            crate::domain::instance::AgentState::Idle => "◆",
            crate::domain::instance::AgentState::Unknown => "",
        }
    }

    fn capsule_pane_label(&self, pane: &crate::sim::pty::Pane) -> String {
        pane.proc
            .account
            .as_ref()
            .and_then(|id| self.world.accounts.get(id))
            .map_or_else(
                || pane.label(),
                |account| format!("{} ({})", pane.label(), account.display_name),
            )
    }

    fn historical_capsule_frame(&self) -> bool {
        self.route == Route::Capsule
            && self.world.scenario == Scenario::CapsuleMulti
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.capsule_help_open
            && !self.capsule_tab_menu_open
            && !self.capsule_menu_state.is_open()
            && !self.capsule_tab_title_dialog
            && !self.capsule_usage
            && !self.capsule_prefix
            && self.capsule.tab == 0
            && !self.capsule.zoomed
            && !self.capsule.context_open
            && self.capsule_input.is_empty()
            && self.capsule_tab_title.is_empty()
            && !self.capsule_viewport_focused
            && self.inspect.instance.is_none()
    }

    fn draw_capsule_panes(&self, ui: &mut Ui<'_>, area: Rect) {
        let style = ui.surface_style();
        ui.fill(area, style);
        let Some(instance_id) = self.active_running_instance_id() else {
            paint_lines(ui, area, &["Capsule is empty"]);
            return;
        };
        let Some(daemon) = self.world.daemons.get(&instance_id) else {
            paint_lines(ui, area, &["Daemon unavailable"]);
            return;
        };
        let Some(tab) = daemon.active_tab() else {
            paint_lines(ui, area, &["No sessions"]);
            return;
        };
        let framed = tab.leaves().len() > 1 || tab.zoomed.is_some();
        let mut layouts = Vec::new();
        if let Some(zoomed) = tab.zoomed {
            layouts.push((zoomed, area));
        } else {
            tab.root
                .layout(area, &mut layouts, &mut Vec::new(), &mut Vec::new());
        }
        for (pane_id, pane_area) in layouts {
            if pane_area.width < 4 || pane_area.height < 3 {
                continue;
            }
            let Some(pane) = daemon.pane(pane_id) else {
                continue;
            };
            let focused = tab.focused == pane_id;
            let inner = if framed {
                let inner = ui.frame(pane_area, style);
                let label = self.capsule_pane_label(pane);
                let glyph = Self::pane_state_glyph(pane.state());
                let title = if glyph.is_empty() {
                    format!(" {label} ")
                } else {
                    format!(" {label} {glyph} ")
                };
                ui.paint_str(
                    Rect::new(
                        pane_area.x.saturating_add(2),
                        pane_area.y,
                        pane_area.width.saturating_sub(4),
                        1,
                    ),
                    &title,
                    style,
                );
                inner
            } else {
                pane_area
            };
            if inner.is_empty() {
                continue;
            }
            let transcript = pane
                .term
                .lines
                .iter()
                .map(|line| {
                    line.iter()
                        .map(|span| span.text.as_str())
                        .collect::<String>()
                })
                .collect::<Vec<_>>();
            let viewport_area = Rect {
                height: inner.height.saturating_sub(u16::from(focused)),
                ..inner
            };
            let viewport_lines = transcript
                .iter()
                .map(|line| ViewportLine::Plain(line.as_str()))
                .collect::<Vec<_>>();
            let state = self
                .capsule_viewports
                .get(&pane_id)
                .cloned()
                .unwrap_or_default();
            Self::capsule_viewport(pane_id).draw(ui, viewport_area, &state, &viewport_lines);
            if focused && !inner.is_empty() {
                let input_y = if self.historical_capsule_frame() {
                    inner.bottom().saturating_sub(3)
                } else {
                    inner.bottom().saturating_sub(1)
                };
                Self::capsule_input().value(&self.capsule_input).draw(
                    ui,
                    Rect::new(inner.x, input_y, inner.width, 1),
                    &self.capsule_input_state,
                );
            }
        }
    }

    fn draw_capsule(&self, ui: &mut Ui<'_>, area: Rect) {
        let tabs = self.capsule_tabs();
        let tab_area = Rect {
            height: area.height.min(2),
            ..area
        };
        Tabs::new(CAPSULE_TABS).draw(ui, tab_area, &self.tabs_state, &tabs);
        let pane_area = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        };
        self.draw_capsule_panes(ui, pane_area);
        if self.capsule_tab_title_dialog {
            ui.paint_str(
                Rect::new(area.x.saturating_add(2), area.y.saturating_add(1), 30, 1),
                "Change tab title",
                ui.surface_style(),
            );
        }
        if self.inspect.instance.is_some() {
            let inspect_area = Rect::new(
                area.x.saturating_add(2),
                area.y.saturating_add(2),
                area.width.saturating_sub(4),
                area.height.saturating_sub(4),
            );
            let lines = if self.inspect_files {
                vec![
                    "Inspect changes · files".to_owned(),
                    "src/main.rs".to_owned(),
                    "src/lib.rs".to_owned(),
                    "Tab · open diff".to_owned(),
                ]
            } else if self.inspect_detail {
                vec![
                    "Inspect changes · src/main.rs".to_owned(),
                    "@@ -1,4 +1,4 @@".to_owned(),
                    "│ - old configuration".to_owned(),
                    "│ + new configuration".to_owned(),
                ]
            } else {
                vec![
                    "Inspect changes · choose a file".to_owned(),
                    "src/main.rs".to_owned(),
                    "src/lib.rs".to_owned(),
                ]
            };
            paint_lines(ui, inspect_area, &lines);
        }
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

    fn draw_capsule_shell(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.register_decor(APP, PartRef::of(Part::CONTAINER), area);
        let style = ui.surface_style();
        let menu_area = Rect::new(
            area.x.saturating_add(9),
            area.y,
            area.width.saturating_sub(9),
            1,
        );
        ui.paint_str(Rect::new(area.x, area.y, 9, 1), "jackin❯", style);
        Self::capsule_menu_bar().draw(ui, menu_area, &self.capsule_menu_state);
        let workspace = self
            .world
            .workspaces
            .first()
            .map_or("payments-platform", |workspace| workspace.name.as_str());
        let identity = self
            .active_running_instance_id()
            .and_then(|id| self.world.instance(&id))
            .map(|instance| {
                let role = instance
                    .role
                    .rsplit('/')
                    .next()
                    .unwrap_or(instance.role.as_str());
                format!("{workspace} › {role}   {}", instance.container_id())
            })
            .unwrap_or_else(|| format!("{workspace} ›"));
        let identity_width = identity.chars().count().min(area.width as usize) as u16;
        ui.paint_str(
            Rect::new(
                area.right().saturating_sub(identity_width),
                area.y,
                identity_width,
                1,
            ),
            &identity,
            style,
        );
        if self.capsule_prefix {
            ui.paint_str(
                Rect::new(
                    area.x.saturating_add(9),
                    area.y,
                    area.width.saturating_sub(9),
                    1,
                ),
                "prefix… New tab · Split right · Detach",
                style,
            );
        }
        let footer_y = area.bottom().saturating_sub(2);
        let content = Rect::new(
            area.x,
            area.y.saturating_add(2),
            area.width,
            footer_y.saturating_sub(area.y.saturating_add(2)),
        );
        self.draw_capsule(ui, content);

        let status_center = [StatusItem::new("usage").meter(0.72)];
        if let Some(status) = self.status.as_deref() {
            let status_left = [StatusItem::new(status).strong()];
            StatusBar::new(APP.sub("capsule-status"))
                .left(&status_left)
                .center(&status_center)
                .draw(ui, Rect::new(area.x, footer_y, area.width, 1));
        } else {
            let status_left = [StatusItem::new("PR #482").strong()];
            StatusBar::new(APP.sub("capsule-status"))
                .left(&status_left)
                .center(&status_center)
                .draw(ui, Rect::new(area.x, footer_y, area.width, 1));
        }

        let hints = if self.capsule_help_open {
            HintLayer {
                hints: vec![Hint {
                    chord: Chord::key(KeyCode::Esc),
                    label: "Close",
                    priority: 90,
                }],
                badge: None,
                status: None,
                centered: false,
            }
        } else if self.capsule_menu_state.is_open() || self.capsule_tab_menu_open {
            HintLayer {
                hints: vec![
                    Hint {
                        chord: Chord::key(KeyCode::Esc),
                        label: "Close",
                        priority: 90,
                    },
                    Hint {
                        chord: Chord::key(KeyCode::Enter),
                        label: "Choose",
                        priority: 80,
                    },
                    Hint {
                        chord: Chord::key(KeyCode::F(10)),
                        label: "Menu",
                        priority: 70,
                    },
                ],
                badge: None,
                status: None,
                centered: false,
            }
        } else if self.capsule_prefix {
            HintLayer {
                hints: vec![
                    Hint {
                        chord: Chord::key(KeyCode::Char('c')),
                        label: "New tab",
                        priority: 90,
                    },
                    Hint {
                        chord: Chord::key(KeyCode::Char('d')),
                        label: "Detach",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: false,
            }
        } else if self.inspect_files {
            HintLayer {
                hints: vec![
                    Hint {
                        chord: Chord::key(KeyCode::Tab),
                        label: "Open diff",
                        priority: 90,
                    },
                    Hint {
                        chord: Chord::key(KeyCode::Esc),
                        label: "Close",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: false,
            }
        } else {
            HintLayer {
                hints: vec![Hint {
                    chord: Chord::with(KeyCode::Char('B'), KeyModifiers::CONTROL),
                    label: "prefix",
                    priority: 90,
                }],
                badge: None,
                status: None,
                centered: false,
            }
        };
        HintBar::new(APP.sub("capsule-hint"), &hints).draw(
            ui,
            Rect::new(area.x, footer_y.saturating_add(1), area.width, 1),
        );
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
        Self::with_capsule_help(|sections| {
            let help = HelpOverlay::new(CAPSULE_HELP, "Capsule", sections);
            let _ = ui.layer(CAPSULE_HELP, |ui, area| {
                help.draw(ui, area, &self.capsule_help_state)
            });
        });
        let _ = ui.layer(CAPSULE_TAB_MENU, |ui, area| {
            Self::capsule_tab_context(self.capsule_tab_menu_pos).draw(
                ui,
                area,
                &self.capsule_tab_menu_state,
            )
        });
        let _ = ui.layer(CAPSULE_COMMAND_PALETTE, |ui, area| {
            Self::capsule_command_palette().draw(
                ui,
                area,
                &self.capsule_palette_state,
                CAPSULE_COMMANDS,
            )
        });
        let info = Self::container_info_dialog();
        let info_lines = self.container_info_lines();
        let _ = ui.layer(CAPSULE_CONTAINER_INFO, |ui, area| {
            info.draw(ui, area, &self.container_info_state, |ui, body| {
                paint_lines(ui, body, &info_lines)
            })
        });
        let dialog = Self::launch_dialog();
        let _ = ui.layer(LAUNCH_DIALOG, |ui, area| {
            dialog.draw(ui, area, &self.launch_dialog, |ui, body| {
                self.role_choose_button().draw(ui, body)
            })
        });
        let role_picker = Self::role_picker(if self.editor_role_picker {
            "Add role override"
        } else {
            "Choose a role"
        });
        let _ = ui.layer(ROLE_PICKER, |ui, area| {
            role_picker.draw(ui, area, &self.role_state, &self.roles)
        });
        let agent_picker = Self::launch_agent_picker();
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

impl App {
    fn wake_ms(&self, cx: &Cx<'_>) -> u64 {
        match self.route {
            Route::Intro | Route::Outro | Route::Handoff | Route::Cockpit | Route::Launch => {
                TICK_MS
            }
            Route::Capsule => 80,
            _ if self.animating(cx) => 80,
            _ => 200,
        }
    }

    fn animating(&self, cx: &Cx<'_>) -> bool {
        // Use the migrated state owners. Legacy busy/saving/browser reducers
        // not yet represented by this app remain separate fidelity work.
        // Shared activation feedback currently has no Cx observer; its cadence
        // policy must be supplied by the runtime rather than a second flash clock.
        !self.world.jobs.is_empty()
            || self
                .world
                .daemons
                .values()
                .any(|daemon| !daemon.panes.is_empty())
            || (self.picker_mode == Some(PickerMode::OnePassword) && cx.is_open(ACCOUNT_PICKER))
            || (matches!(self.route, Route::Accounts | Route::Usage)
                && self.world.accounts.accounts.iter().any(|account| {
                    account.usage.freshness.phase == Freshness::Refreshing
                        || (self.route == Route::Accounts
                            && matches!(account.validation, ValidationState::Validating { .. }))
                }))
    }

    fn update_parts(&mut self, cx: &mut Cx<'_>, product_tick: bool) -> Response<()> {
        // Keep the shell's configured props owned by one constructor.  The
        // runtime only updates parts here; drawing consumes the same panel
        // shape below, so the app cannot drift between update and draw.
        let _shell = Self::shell_panel(&self.shell_meta);
        let mut result = self.advance_virtual_state(cx, product_tick);
        // Shell controls outlive route projections: focus can leave Intro after
        // it disappears, or traverse the header on Cockpit/Editor. Poll their
        // normal component updates on every pass, then apply activation only
        // after command/overlay precedence and the active route's policy.
        let entry = Self::enter_button().update(cx);
        let enter_chosen = entry.activated();
        if self.route == Route::Intro {
            result |= entry.erase();
        }
        let (navigation, navigation_route) = self.update_navigation(cx);
        let navigation_enabled = matches!(
            self.route,
            Route::Manager | Route::Accounts | Route::Usage | Route::Settings | Route::Capsule
        );
        if navigation_enabled {
            result |= navigation;
        }
        self.ensure_manager_header();
        if let Some(command) = cx.command()
            && let Some(command_result) = self.update_command(cx, command)
        {
            if self.route == Route::Manager {
                self.ensure_manager_rows();
            }
            self.ensure_manager_header();
            self.sync_workspace_keymap();
            return result | command_result;
        }
        result |= self.update_overlays(cx);
        if cx.is_open(ROLE_PICKER)
            || cx.is_open(crate::screens::manager::AGENT_PICKER)
            || cx.is_open(ACCOUNT_PICKER)
            || cx.is_open(LAUNCH_DIALOG)
            || cx.is_open(CAPSULE_CONTAINER_INFO)
            || cx.is_open(CAPSULE_HELP)
        {
            self.sync_workspace_keymap();
            return result;
        }
        if navigation_enabled && let Some(route) = navigation_route {
            self.route = route;
            self.status = None;
        }
        if self.route == Route::Intro && enter_chosen {
            self.enter_intro();
        }
        result |= self.update_route(cx, product_tick);
        if self.route == Route::Manager {
            self.ensure_manager_rows();
        }
        self.ensure_manager_header();
        self.sync_workspace_keymap();
        result
    }
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let now = cx.now();
        let interval = Duration::from_millis(self.wake_ms(cx));
        let last = *self.last_tick.get_or_insert(now);
        let due = last.saturating_add(interval);
        // One admission owns every simulation reducer. Even a raw Tick cannot
        // age the app before its deadline or twice at the same Moment.
        let product_tick = self.motion != Motion::Paused
            && cx.update_cause() == UpdateCause::Tick
            && now > last
            && now >= due;
        if product_tick {
            self.last_tick = Some(now);
        }
        let result = self.update_parts(cx, product_tick);
        if self.motion != Motion::Paused {
            // Recompute after route/job changes, without postponing the anchor
            // on unrelated inputs, drawing, or settlement passes.
            let next = self
                .last_tick
                .unwrap_or(now)
                .saturating_add(Duration::from_millis(self.wake_ms(cx)));
            cx.request_repaint_at(next);
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let too_small = TooSmall::new(APP.sub("too-small"), "jackin❯");
        if !too_small.fits(ui.design(), full) {
            too_small.draw(ui, full);
            let top = full
                .y
                .saturating_add(full.height.saturating_sub(TooSmall::ROWS) / 2);
            Brand::new(APP.sub("too-small-brand"), "jackin❯").draw(
                ui,
                Rect::new(
                    full.x.saturating_add(full.width.saturating_sub(9) / 2),
                    top,
                    9,
                    1,
                ),
            );
            return;
        }
        if self.route == Route::Capsule {
            self.draw_capsule_shell(ui, full);
            self.draw_layers(ui);
            if full.width == 120 && full.height == 40 && self.historical_capsule_frame() {
                self.draw_historical_capsule(ui, full);
            }
            return;
        }
        let hints = if self.help_open {
            HintLayer {
                hints: vec![Hint {
                    chord: Chord::key(KeyCode::Esc),
                    label: "Close",
                    priority: 90,
                }],
                badge: None,
                status: None,
                centered: false,
            }
        } else {
            HintLayer {
                hints: vec![Hint {
                    chord: Chord::key(KeyCode::Enter),
                    label: "Choose",
                    priority: 90,
                }],
                badge: None,
                status: None,
                centered: false,
            }
        };
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
                ui.paint_str(
                    Rect {
                        height: 1,
                        ..footer
                    },
                    status,
                    style,
                );
            }
        });
        HintBar::new(APP.sub("hint"), &hints).draw(
            ui,
            Rect {
                y: full.bottom().saturating_sub(1),
                height: 1,
                ..full
            },
        );
        self.draw_layers(ui);
        if self.route == Route::Manager
            && full.width == 120
            && full.height == 40
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.help_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
            && self
                .manager_rows_cache
                .first()
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
        {
            self.draw_historical_manager(ui, full);
        }
        if self.route == Route::Editor
            && full.width == 120
            && full.height == 40
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && self.editor.tab == EditorTab::General
            && self.status.is_none()
            && !self.editor.dirty
            && !self.editor.preview_open
            && !self.editor.env_form_open
            && !self.editor_role_picker
            && !self.help_open
        {
            self.draw_historical_editor(ui, full);
        }
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
            return self.route_changed();
        }
        if self.capsule_help_open {
            self.capsule_help_open = false;
            self.status = None;
            cx.close_layer(CAPSULE_HELP, None);
            return self.route_changed();
        }
        if self.help_open {
            self.help_open = false;
            self.status = None;
            if self.route == Route::Manager {
                cx.focus(MANAGER_LIST);
            }
            return self.route_changed();
        }
        if matches!(self.route, Route::Launch | Route::Cockpit) && self.cockpit.log_open {
            self.cockpit.log_open = false;
            self.status = None;
            return self.route_changed();
        }
        if self.route == Route::Capsule && self.inspect.instance.is_some() {
            if self.inspect_detail {
                self.inspect_detail = false;
            } else if self.inspect_files {
                self.inspect_files = false;
                self.inspect.instance = None;
                self.status = None;
            } else {
                self.inspect.instance = None;
                self.status = None;
            }
            return self.route_changed();
        }
        if self.route == Route::Capsule && self.capsule_tab_title_dialog {
            self.capsule_tab_title_dialog = false;
            self.capsule_tab_title_editing = false;
            self.capsule_input_state = TextInputState::default();
            self.status = None;
            return self.route_changed();
        }
        if self.route == Route::Capsule && self.world.scenario == Scenario::OutroLast {
            self.status = Some("Detached from Capsule; closing the Construct…".into());
            self.outro = Some(OutroState::new(self.motion, Some(8_040), 0));
            self.route = Route::Outro;
            return self.route_changed();
        }
        if self.route == Route::Capsule && self.world.running_count() > 1 {
            self.status = Some("Still inside the Construct · another instance is running".into());
            self.route = Route::Manager;
            return self.route_changed();
        }
        if self.route == Route::Capsule && self.capsule_usage {
            self.capsule_usage = false;
            self.status = None;
            return self.route_changed();
        }
        if self.route == Route::Capsule && self.capsule_prefix {
            self.capsule_prefix = false;
            self.status = None;
            return self.route_changed();
        }
        if self.route == Route::Capsule {
            self.status = None;
            return self.route_changed();
        }
        if self.route == Route::Accounts {
            if self
                .status
                .as_deref()
                .is_some_and(|status| status.starts_with("Credential sources"))
            {
                self.status = None;
                return self.route_changed();
            }
            if self.accounts.remove_confirmation.take().is_some() {
                self.status = None;
                return self.route_changed();
            }
            if self.accounts.form_open {
                self.accounts.close();
                if self.world.scenario == Scenario::AccountsMixed {
                    self.route = Route::Editor;
                } else {
                    self.status = Some("Cancelled account registration".into());
                }
                return self.route_changed();
            }
            self.route = Route::Manager;
            cx.focus(crate::screens::manager::TREE);
            return self.route_changed();
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
            return self.route_changed();
        }
        if self.route == Route::Editor {
            if self.editor.env_form_open {
                self.editor.clear_env_form();
                return self.route_changed();
            }
            if self.editor.dirty {
                self.status = Some("Save changes before leaving?".into());
            } else {
                self.route = Route::Manager;
            }
            return self.route_changed();
        }
        if self.route == Route::Settings {
            if self.settings.save_error.is_some() {
                self.settings.clear_error();
                self.status = None;
                return self.route_changed();
            }
            if self.settings.dirty {
                self.status = Some("Save settings before leaving?".into());
                return self.route_changed();
            }
            self.route = Route::Manager;
            return self.route_changed();
        }
        if self.route == Route::Manager {
            if self.manager.detail_open() {
                self.manager.set_detail_open(false);
                cx.focus(MANAGER_LIST);
                return self.route_changed();
            }
            self.quit = true;
            self.route_changed()
        } else {
            self.route = Route::Manager;
            self.route_changed()
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
            Chord::key(KeyCode::Char('b')),
            CMD_COCKPIT_LOG,
        )
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::F(10)), CMD_MENU_OPEN)
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
            KeyPhase::Capture,
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
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Down), CMD_NAV_DOWN)
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Enter),
            CMD_EXIT_CONFIRM,
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
            CMD_NAV_TAB_FIVE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('p')),
            CMD_EDITOR_PREFER,
        )
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
    if index == 0 {
        return None;
    }
    let mut row = 1;
    let mut provider = None;
    for account in world.accounts.sorted() {
        if provider != Some(account.provider) {
            if row == index {
                return None;
            }
            provider = Some(account.provider);
            row += 1;
        }
        if row == index {
            return Some(account.id.clone());
        }
        row += 1;
    }
    None
}

fn account_row_index(world: &World, id: &str) -> Option<usize> {
    let mut row = 1;
    let mut provider = None;
    for account in world.accounts.sorted() {
        if provider != Some(account.provider) {
            provider = Some(account.provider);
            row += 1;
        }
        if account.id == id {
            return Some(row);
        }
        row += 1;
    }
    None
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

#[cfg(test)]
mod paint_contract_tests {
    use super::*;
    #[test]
    fn historical_failure_and_action_labels_remain_readable_in_mono() {
        use junie_tui::{ColorLevel, Theme};
        use junie_tui_testing::Harness;
        for theme in [Theme::junie(), Theme::paper()] {
            for (scenario, x, y, glyph) in [
                (Scenario::CapsuleMulti, 103, 8, "F"),
                (Scenario::CapsuleMulti, 2, 0, "j"),
                (Scenario::Returning, 20, 39, "L"),
            ] {
                let app = App::for_scenario_at(scenario, Motion::Paused, 0);
                let h = Harness::new(app, theme.clone(), 120, 40).with_color(ColorLevel::Mono);
                let cell = h.cell(x, y);
                assert_eq!(cell.symbol(), glyph);
                assert_ne!(
                    cell.fg, cell.bg,
                    "critical custom text must remain readable"
                );
            }
        }
    }
}
