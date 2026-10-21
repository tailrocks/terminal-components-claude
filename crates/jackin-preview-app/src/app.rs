use std::{
    cell::RefCell,
    collections::{BTreeMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    mem,
    time::Duration,
};
use termrock::author::PaintStyle;

use termrock::{
    Action, ActionKey, ActivationKey, App as TuiApp, AsItem, Brand, Button, Chord, ContextMenu, Cx,
    Dialog, DialogAction, DialogState, FrameRead, HelpAction, HelpOverlay, HelpOverlayState,
    HelpSection, Hint, HintBar, HintLayer, Id, Intent, Item, ItemKey, KeyCode, KeyMap,
    KeyModifiers, KeyPhase, List, ListAction, ListState, Menu, MenuAction, MenuBar, MenuItem,
    MenuState, Modifier, Moment, Panel, Part, PartRef, Phase, Picker, PickerAction, PickerState,
    Position, ProjectedText, Reconcile, Rect, Response, SecretPolicy, StatusBar, StatusItem, Tabs,
    TabsAction, TabsState, TextAction, TextInput, TextInputState, TextViewport, TooSmall, Ui,
    UpdateCause, Variant, ViewportAction, ViewportLine, ViewportState,
};

use crate::domain::account::{
    Account, AccountRegistry, CredentialSource, DetectedKind, DuplicateProbe, ValidationState,
    fingerprint, tail_of,
};
use crate::domain::agent::{Agent, Provider};
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::domain::usage::Freshness;
use crate::domain::workspace::{Effective, EnvValue, EnvVar, env_key_error, mask};
use crate::rain::{HANDOFF_LEN, INTRO_END, IntroState, OutroState};
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
use crate::sim::pty::{Daemon, Maximized, Pane, PaneId, PaneNode, SplitDir, Tab};
use crate::sim::world::{World, world_for};

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

pub const APP: Id = Id::root("jackin.preview");
pub const ENTER: Id = APP.sub("enter");
pub const MANAGER: Id = APP.sub("manager");
pub const ACCOUNTS: Id = APP.sub("accounts");
pub const USAGE: Id = APP.sub("usage");
pub const SETTINGS: Id = APP.sub("settings");
pub const CAPSULE: Id = APP.sub("capsule");
pub const MANAGER_LIST: Id = crate::screens::manager::TREE;
pub const ACCOUNTS_LIST: Id = crate::screens::accounts::LIST;
pub const LAUNCH: Id = crate::screens::manager::LAUNCH;
pub const ACCOUNT_ADD: Id = APP.sub("account-add");
pub const SETTINGS_TRUST: Id = crate::screens::settings::TRUST;
pub const CAPSULE_TABS: Id = crate::screens::capsule::TABS;
pub const CAPSULE_PANES: Id = crate::screens::capsule::PANES;
const CAPSULE_INPUT: Id = APP.sub("capsule-input");
pub const LAUNCH_DIALOG: Id = APP.sub("launch-dialog");
pub const ROLE_CHOOSE: Id = LAUNCH_DIALOG.sub("role");
pub const ROLE_PICKER: Id = APP.sub("role-picker");
pub const ACCOUNT_PICKER: Id = APP.sub("account-picker");
pub const LAUNCH_CANCEL: Id = APP.sub("launch-cancel");
pub const LAUNCH_RETRY: Id = APP.sub("launch-retry");
const CAPSULE_MENU_BAR: Id = APP.sub("capsule-menu-bar");
const CAPSULE_TAB_MENU: Id = APP.sub("capsule-tab-menu");
const CAPSULE_COMMAND_PALETTE: Id = APP.sub("capsule-command-palette");
const CAPSULE_CONTAINER_INFO: Id = APP.sub("capsule-container-info");
const ABOUT_DIALOG: Id = APP.sub("about");
const QUIT_DIALOG: Id = APP.sub("quit-dialog");
const CANCEL_LAUNCH_DIALOG: Id = APP.sub("cancel-dialog");
const COCKPIT_INFO_DIALOG: Id = APP.sub("cockpit-info");
const LAUNCH_FAILURE_DIALOG: Id = APP.sub("launch-failure");
const EDITOR_EXIT_DIALOG: Id = APP.sub("editor-exit");
const CAPSULE_HELP: Id = APP.sub("capsule-help");
const QUIT_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "Quit"),
];
const CANCEL_LAUNCH_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Keep launching"),
    Action::danger(ActionKey::CONFIRM, "Cancel launch"),
];
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
const CMD_EDITOR_FIELD: ActionKey = ActionKey::application("jackin.editor.field");
const CMD_EDITOR_PREVIOUS: ActionKey = ActionKey::application("jackin.editor.previous-tab");
const CMD_EDITOR_ENV: ActionKey = ActionKey::application("jackin.editor.environments");
const CMD_SAVE: ActionKey = ActionKey::application("jackin.save");
const CMD_MANAGER_EXPAND: ActionKey = ActionKey::application("jackin.manager.expand");
const CMD_MANAGER_INSPECT: ActionKey = ActionKey::application("jackin.manager.inspect");
const CMD_EDITOR_OPEN: ActionKey = ActionKey::application("jackin.editor.open");
const CMD_EDITOR_ROLES: ActionKey = ActionKey::application("jackin.editor.roles");
const CMD_EDITOR_ROLES_CONFIRM: ActionKey = ActionKey::application("jackin.editor.roles-confirm");
const CMD_EDITOR_MOUNTS: ActionKey = ActionKey::application("jackin.editor.mounts");
const CMD_EDITOR_MOUNTS_CONFIRM: ActionKey = ActionKey::application("jackin.editor.mounts-confirm");
const CMD_EDITOR_MOUNT_ISOLATION: ActionKey =
    ActionKey::application("jackin.editor.mount-isolation");
const CMD_EDITOR_PREFER: ActionKey = ActionKey::application("jackin.editor.prefer");
const CMD_NAV_DOWN: ActionKey = ActionKey::application("jackin.navigation.down");
const CMD_NAV_UP: ActionKey = ActionKey::application("jackin.navigation.up");
const CMD_ACCOUNT_FILTER: ActionKey = ActionKey::application("jackin.account.filter");
const CMD_ADD_FORM_BACKTAB: ActionKey = ActionKey::application("jackin.account.add-form-backtab");
const CMD_ADD_FORM_REQUIRED: ActionKey = ActionKey::application("jackin.account.add-form-required");
const CMD_ACCOUNT_DRAWER: ActionKey = ActionKey::application("jackin.account.drawer");
const CMD_SETTINGS_TAB_PREV: ActionKey = ActionKey::application("jackin.settings.previous-tab");
const CMD_SETTINGS_MOUNTS: ActionKey = ActionKey::application("jackin.settings.mounts");
const CMD_SETTINGS_MOUNTS_CONFIRM: ActionKey =
    ActionKey::application("jackin.settings.mounts-confirm");
const CMD_SETTINGS_ENV: ActionKey = ActionKey::application("jackin.settings.env");
const CMD_SETTINGS_ENV_CONFIRM: ActionKey = ActionKey::application("jackin.settings.env-confirm");
const CMD_SETTINGS_AGENTS: ActionKey = ActionKey::application("jackin.settings.agents");
const CMD_SETTINGS_AGENTS_CONFIRM: ActionKey =
    ActionKey::application("jackin.settings.agents-confirm");
const CMD_SETTINGS_TRUST: ActionKey = ActionKey::application("jackin.settings.trust");
const CMD_SETTINGS_TRUST_CONFIRM: ActionKey =
    ActionKey::application("jackin.settings.trust-confirm");
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
const CMD_COCKPIT_CANCEL: ActionKey = ActionKey::application("jackin.cockpit.cancel");
const CMD_COCKPIT_INFO: ActionKey = ActionKey::application("jackin.cockpit.info");
const CMD_COCKPIT_DEBUG: ActionKey = ActionKey::application("jackin.cockpit.debug");
const CMD_COCKPIT_TRACE: ActionKey = ActionKey::application("jackin.cockpit.trace");
const EXIT_CANCEL: ActionKey = ActionKey::application("jackin.editor.exit-cancel");
const EXIT_DISCARD: ActionKey = ActionKey::application("jackin.editor.exit-discard");
const EXIT_SAVE: ActionKey = ActionKey::application("jackin.editor.exit-save");
const EDITOR_EXIT_ACTIONS: [Action<'static>; 3] = [
    Action::quiet(EXIT_CANCEL, "Cancel"),
    Action::danger(EXIT_DISCARD, "Discard"),
    Action::new(EXIT_SAVE, "Save"),
];
const CMD_TAB_RENAME: ActionKey = ActionKey::application("jackin.capsule.tab-rename");
const CMD_TAB_CLOSE: ActionKey = ActionKey::application("jackin.capsule.tab-close");
const CMD_INSPECT_CHANGES: ActionKey = ActionKey::application("jackin.capsule.inspect-changes");
const CMD_COPY_SELECTION: ActionKey = ActionKey::application("jackin.capsule.copy-selection");
const CMD_CAPSULE_EXPORT_FILE: ActionKey = ActionKey::application("jackin.capsule.export-file");
const CMD_CAPSULE_EXPORT_REVEAL: ActionKey = ActionKey::application("jackin.capsule.export-reveal");
const CMD_KEYBOARD_SHORTCUTS: ActionKey = ActionKey::application("jackin.keyboard-shortcuts");
const CMD_ABOUT: ActionKey = ActionKey::application("jackin.about");
const CMD_MENU_OPEN: ActionKey = ActionKey::application("jackin.menu.open");
const MANAGER_FILE_MENU: Id = APP.sub("manager-file-menu");
const CMD_CONTAINER_INFO: ActionKey = ActionKey::application("jackin.capsule.container-info");
const CAPSULE_MENU_PREVIOUS: ActionKey = ActionKey::custom("menu.bar.prev.left");
const CAPSULE_MENU_NEXT: ActionKey = ActionKey::custom("menu.bar.next.right");

const CAPSULE_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_NEW_TAB, "New tab"),
    MenuItem::new(CMD_CAPSULE_SPLIT_RIGHT, "Split right"),
    MenuItem::new(CMD_CAPSULE_SPLIT_BELOW, "Split below"),
    MenuItem::new(CMD_CAPSULE_EXPORT_FILE, "Export selected file…"),
    MenuItem::new(
        CMD_CAPSULE_EXPORT_REVEAL,
        "Export selected file and reveal…",
    ),
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
    MenuItem::new(CMD_TAB_RENAME, "Change title…").hint("2×click"),
    MenuItem::new(CMD_CAPSULE_SPLIT_RIGHT, "Split right")
        .hint("Ctrl+B %")
        .separator(),
    MenuItem::new(CMD_TAB_CLOSE, "Close tab")
        .hint("Ctrl+B &")
        .danger(),
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

mod accounts_add_form_view;
mod accounts_detail_view;
mod accounts_drawer_view;
mod accounts_filter_view;
mod accounts_filter_wide;
mod editor_view;
mod prelude_view;

mod capsule_menu_view;
mod capsule_split_view;
mod capsule_zoom_view;

mod hard_launch_picker_view;
mod inspect_view;
mod launch_cockpit;
mod launch_dialog_plate;
mod launch_picker_view;
mod manager_help;
mod manager_menu_view;
mod settings_view;
mod usage_detail_view;
mod usage_overview_view;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Intro,
    Manager,
    Prelude,
    Editor,
    Accounts,
    Usage,
    Settings,
    Launch,
    Cockpit,
    Handoff,
    Capsule,
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

#[derive(Debug, Clone, Default)]
struct CapsuleFrameCaches {
    transcripts: BTreeMap<PaneId, (u64, ProjectedText)>,
    tabs: Option<(u64, Vec<String>)>,
    titles: BTreeMap<PaneId, (u64, String)>,
    layouts: Option<(u64, Vec<(PaneId, Rect)>)>,
}

#[derive(Debug, Clone)]
struct HintLayerSet {
    help: HintLayer,
    default: HintLayer,
    capsule_help: HintLayer,
    capsule_menu: HintLayer,
    capsule_prefix: HintLayer,
    capsule_inspect: HintLayer,
    #[allow(dead_code)]
    capsule_default: HintLayer,
}

impl HintLayerSet {
    fn layer(hints: Vec<Hint>) -> HintLayer {
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: false,
        }
    }

    fn hint(key: termrock::HintKey, label: &'static str, priority: u8) -> Hint {
        Hint {
            key,
            label,
            priority,
        }
    }
}

impl Default for HintLayerSet {
    fn default() -> Self {
        use termrock::HintKey;
        Self {
            help: Self::layer(vec![Self::hint(
                HintKey::Chord(Chord::key(KeyCode::Esc)),
                "Close",
                90,
            )]),
            default: Self::layer(vec![Self::hint(
                HintKey::Chord(Chord::key(KeyCode::Enter)),
                "Choose",
                90,
            )]),
            capsule_help: Self::layer(vec![Self::hint(
                HintKey::Chord(Chord::key(KeyCode::Esc)),
                "Close",
                90,
            )]),
            capsule_menu: Self::layer(vec![
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Close", 90),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Enter)), "Choose", 80),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::F(10))), "Menu", 70),
            ]),
            capsule_prefix: Self::layer(vec![
                Self::hint(
                    HintKey::Chord(Chord::key(KeyCode::Char('c'))),
                    "New tab",
                    90,
                ),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Char('d'))), "Detach", 80),
            ]),
            capsule_inspect: Self::layer(vec![
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Tab)), "Open diff", 90),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Close", 80),
            ]),
            capsule_default: Self::layer(vec![Self::hint(
                HintKey::Chord(Chord::with(KeyCode::Char('B'), KeyModifiers::CONTROL)),
                "prefix",
                90,
            )]),
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    pub world: World,
    pub manager: ManagerState,
    pub accounts: AccountsState,
    pub prelude: PreludeState,
    pub editor: EditorState,
    pub settings: SettingsState,
    pub usage: UsageState,
    pub cockpit: CockpitState,
    pub capsule: CapsuleState,
    pub inspect: InspectState,
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
    about_state: DialogState,
    about_open: bool,
    quit_open: bool,
    quit_state: DialogState,
    quit_body: String,
    cancel_open: bool,
    cancel_from_x: bool,
    cancel_state: DialogState,
    cockpit_info_open: bool,
    cockpit_info_state: DialogState,
    launch_failure_open: bool,
    launch_failure_state: DialogState,
    cockpit_debug_open: bool,
    cockpit_trace_open: bool,
    editor_exit_open: bool,
    editor_exit_state: DialogState,
    editor_leaf_focused: bool,
    editor_general_cursor: u8,
    editor_mounts_armed: bool,
    editor_roles_armed: bool,
    settings_mounts_armed: bool,
    settings_env_armed: bool,
    settings_agents_armed: bool,
    settings_trust_armed: bool,
    settings_trust_cursor: usize,
    settings_trust_origin: Vec<bool>,
    settings_save_preview: bool,
    capsule_tab_menu_state: MenuState,
    capsule_tab_menu_pos: Position,
    capsule_tab_menu_open: bool,
    capsule_palette_state: PickerState,
    tabs_state: TabsState,
    launch_dialog: DialogState,
    role_state: PickerState,
    agent_state: PickerState,
    launch_picker_open: bool,
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
    capsule_frame: RefCell<CapsuleFrameCaches>,
    hint_layers: HintLayerSet,
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
    manager_inspect: bool,
    manager_file_menu: bool,
    active_instance: Option<String>,
    launch_agent: Agent,
    launch_account: Option<String>,
    screen: Rect,
    screen_junie: bool,
    add_form_backtab: bool,
    add_form_required: bool,
}

impl App {
    fn role_label<'a>(&'a self, key: &'a str) -> &'a str {
        self.world
            .roles
            .iter()
            .find(|role| {
                key.strip_prefix(role.namespace.as_str())
                    .and_then(|rest| rest.strip_prefix('/'))
                    == Some(role.name.as_str())
            })
            .map_or(key, |role| role.name.as_str())
    }

    pub fn for_scenario(scenario: Scenario, motion: Motion) -> Self {
        Self::for_scenario_at(scenario, motion, 0)
    }

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
        let mut launch_events = Vec::new();
        if let Some(run) = &mut launch {
            for _ in 0..frame {
                if run.is_terminal() {
                    break;
                }
                launch_events.extend(run.advance());
            }
            if run.done {
                route = Route::Handoff;
            }
            if matches!(scenario, Scenario::LaunchRunning | Scenario::LaunchFailure) {
                run.trace_label = "run-202609030914-b5df".into();
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
            about_state: DialogState::default(),
            about_open: false,
            quit_open: false,
            quit_state: DialogState::default(),
            quit_body: String::new(),
            cancel_open: false,
            cancel_from_x: false,
            cancel_state: DialogState::default(),
            cockpit_info_open: false,
            cockpit_info_state: DialogState::default(),
            launch_failure_open: false,
            launch_failure_state: DialogState::default(),
            cockpit_debug_open: false,
            cockpit_trace_open: false,
            editor_exit_open: false,
            editor_exit_state: DialogState::default(),
            editor_leaf_focused: false,
            editor_general_cursor: 0,
            editor_mounts_armed: false,
            editor_roles_armed: false,
            settings_mounts_armed: false,
            settings_env_armed: false,
            settings_agents_armed: false,
            settings_trust_armed: false,
            settings_trust_cursor: 0,
            settings_trust_origin: Vec::new(),
            settings_save_preview: false,
            capsule_tab_menu_state: MenuState::default(),
            capsule_tab_menu_pos: Position::new(0, 0),
            capsule_tab_menu_open: false,
            capsule_palette_state: PickerState::default(),
            tabs_state: TabsState::default(),
            launch_dialog: DialogState::default(),
            role_state: PickerState::default(),
            agent_state: PickerState::default(),
            launch_picker_open: false,
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
            capsule_frame: RefCell::default(),
            hint_layers: HintLayerSet::default(),
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
            manager_inspect: false,
            manager_file_menu: false,
            active_instance: None,
            launch_agent: Agent::ClaudeCode,
            launch_account: None,
            screen: Rect::new(0, 0, 0, 0),
            screen_junie: false,
            add_form_backtab: false,
            add_form_required: false,
        };
        app.handle_launch_events(launch_events);
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

    pub const fn route(&self) -> Route {
        self.route
    }

    pub const fn motion(&self) -> Motion {
        self.motion
    }

    pub const fn route_tick_ms(&self) -> u64 {
        self.route.tick_ms()
    }

    pub fn frame(&self) -> u64 {
        match self.route {
            Route::Intro => self.intro.tick,
            Route::Outro => self.outro.as_ref().map_or(0, |state| state.tick),
            Route::Handoff => self.handoff_frame.unwrap_or(0),
            Route::Launch | Route::Cockpit => self.launch.as_ref().map_or(0, |run| run.tick),
            _ => self.world.now_ms().div_euclid(self.route_tick_ms() as i64) as u64,
        }
    }

    pub fn selected_role(&self) -> &str {
        self.roles
            .get(self.selected_role)
            .map_or("chainargos/the-architect", |role| role.key.as_str())
    }

    pub const fn launch(&self) -> Option<&LaunchRun> {
        self.launch.as_ref()
    }

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
        let new_workspace = Chord::key(KeyCode::Char('n'));
        self.keymap.remove(KeyPhase::Capture, new_workspace);
        if self.route == Route::Manager {
            // The manager hint is `n New`. Capture beats the list.
            self.keymap
                .add(KeyPhase::Capture, new_workspace, CMD_NEW_WORKSPACE);
        }
        let enter = Chord::key(KeyCode::Enter);
        self.keymap.remove(KeyPhase::Capture, enter);
        if self.route == Route::Usage && !self.usage.detail_open() {
            self.keymap.add(KeyPhase::Capture, enter, CMD_EXIT_CONFIRM);
        } else if self.accounts_work_drawer_enter() {
            // The focused nav button would consume Enter before the bubble
            // binding. Capture it so this row opens the drawer.
            self.keymap.add(KeyPhase::Capture, enter, CMD_EXIT_CONFIRM);
        } else if self.route == Route::Editor && self.editor_mounts_armed {
            // `2` then Enter selects Mounts. Capture beats the mount button,
            // so Enter confirms the tab and does not edit the row.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_EDITOR_MOUNTS_CONFIRM);
        } else if self.route == Route::Editor && self.editor_roles_armed {
            // `3` then Enter selects Roles. Capture beats the default-role
            // button, so Enter confirms the tab and does not set a status.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_EDITOR_ROLES_CONFIRM);
        } else if self.route == Route::Settings && self.settings_mounts_armed {
            // `2` then Enter leaves General for Mounts. Capture beats Save.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_SETTINGS_MOUNTS_CONFIRM);
        } else if self.route == Route::Settings && self.settings_env_armed {
            // `3` then Enter leaves General for Environments. Capture beats Save.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_SETTINGS_ENV_CONFIRM);
        } else if self.route == Route::Settings && self.settings_agents_armed {
            // `4` then Enter leaves General for Agents. Capture beats Save.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_SETTINGS_AGENTS_CONFIRM);
        } else if self.route == Route::Settings && self.settings_trust_armed {
            // `5` then Enter leaves General for Trust. Capture beats Save.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_SETTINGS_TRUST_CONFIRM);
        } else if self.add_form_key_host() && self.add_form_backtab && !self.add_form_required {
            // Backtab already stayed on the dialog. Enter validates it.
            // Without this capture the focused start control would leave the page.
            self.keymap
                .add(KeyPhase::Capture, enter, CMD_ADD_FORM_REQUIRED);
        }
        let backtab = Chord::key(KeyCode::BackTab);
        // Crossterm reports ESC [ Z as BackTab with Shift still set.
        let backtab_shift = Chord::with(KeyCode::BackTab, KeyModifiers::SHIFT);
        let shift_tab = Chord::with(KeyCode::Tab, KeyModifiers::SHIFT);
        self.keymap.remove(KeyPhase::Capture, backtab);
        self.keymap.remove(KeyPhase::Capture, backtab_shift);
        self.keymap.remove(KeyPhase::Capture, shift_tab);
        let field_tab = Chord::key(KeyCode::Tab);
        self.keymap.remove(KeyPhase::Bubble, field_tab);
        self.keymap.remove(KeyPhase::Capture, field_tab);
        if self.route == Route::Editor
            && self.editor.tab == EditorTab::General
            && !self.editor.env_form_open
            && !self.editor.preview_open
        {
            // Three Tabs land on Keep awake. Capture beats focus traversal.
            self.keymap
                .add(KeyPhase::Capture, field_tab, CMD_EDITOR_FIELD);
        }
        if self.add_form_key_host() && !self.add_form_required {
            // The shell tab before the start button is Capsule. Backtab must
            // stay on the empty dialog so Enter can show Required.
            self.keymap
                .add(KeyPhase::Capture, backtab, CMD_ADD_FORM_BACKTAB);
            self.keymap
                .add(KeyPhase::Capture, backtab_shift, CMD_ADD_FORM_BACKTAB);
            self.keymap
                .add(KeyPhase::Capture, shift_tab, CMD_ADD_FORM_BACKTAB);
        }
        let mounts = Chord::key(KeyCode::Char('2'));
        self.keymap.remove(KeyPhase::Bubble, mounts);
        self.keymap.remove(KeyPhase::Capture, mounts);
        if self.route == Route::Editor {
            self.keymap.add(KeyPhase::Bubble, mounts, CMD_EDITOR_MOUNTS);
        } else if self.route == Route::Settings && self.settings.tab == 0 {
            // `2` arms Mounts. Enter confirms it. Capture beats a focused control.
            self.keymap
                .add(KeyPhase::Capture, mounts, CMD_SETTINGS_MOUNTS);
        }
        let env_key = Chord::key(KeyCode::Char('3'));
        self.keymap.remove(KeyPhase::Bubble, env_key);
        self.keymap.remove(KeyPhase::Capture, env_key);
        if self.route == Route::Settings && self.settings.tab == 0 {
            // `3` arms Environments. Enter confirms it. Capture beats editor roles.
            self.keymap
                .add(KeyPhase::Capture, env_key, CMD_SETTINGS_ENV);
        } else {
            self.keymap.add(KeyPhase::Bubble, env_key, CMD_EDITOR_ROLES);
        }
        let agents_key = Chord::key(KeyCode::Char('4'));
        self.keymap.remove(KeyPhase::Bubble, agents_key);
        self.keymap.remove(KeyPhase::Capture, agents_key);
        if self.route == Route::Settings && self.settings.tab == 0 {
            // `4` arms Agents. Enter confirms it. Capture beats editor environments.
            self.keymap
                .add(KeyPhase::Capture, agents_key, CMD_SETTINGS_AGENTS);
        } else {
            self.keymap
                .add(KeyPhase::Bubble, agents_key, CMD_EDITOR_ENV);
        }
        let trust_key = Chord::key(KeyCode::Char('5'));
        self.keymap.remove(KeyPhase::Bubble, trust_key);
        self.keymap.remove(KeyPhase::Capture, trust_key);
        if self.route == Route::Settings && self.settings.tab == 0 {
            // `5` arms Trust. Enter confirms it. Capture beats the trust button.
            self.keymap
                .add(KeyPhase::Capture, trust_key, CMD_SETTINGS_TRUST);
        } else {
            self.keymap
                .add(KeyPhase::Bubble, trust_key, CMD_NAV_TAB_FIVE);
        }
        let up = Chord::key(KeyCode::Up);
        self.keymap.remove(KeyPhase::Bubble, up);
        if self.route == Route::Prelude && self.prelude.step() == 1 {
            self.keymap.add(KeyPhase::Bubble, up, CMD_NAV_UP);
        }
        let filter = Chord::key(KeyCode::Char('/'));
        self.keymap.remove(KeyPhase::Bubble, filter);
        if self.route == Route::Accounts && !self.accounts.form_open && !self.accounts.filtering {
            self.keymap
                .add(KeyPhase::Bubble, filter, CMD_ACCOUNT_FILTER);
        }
        let drawer = Chord::key(KeyCode::Char('i'));
        self.keymap.remove(KeyPhase::Bubble, drawer);
        if self.route == Route::Accounts
            && !self.accounts.form_open
            && !self.accounts.filtering
            && !self.accounts.drawer_open
        {
            self.keymap
                .add(KeyPhase::Bubble, drawer, CMD_ACCOUNT_DRAWER);
        } else if self.route == Route::Manager {
            self.keymap
                .add(KeyPhase::Bubble, drawer, CMD_MANAGER_INSPECT);
        } else if self.route == Route::Editor
            && self.editor.tab == EditorTab::Mounts
            && !self.editor.env_form_open
            && !self.editor.preview_open
            && !self.editor_exit_open
        {
            // `i` dirties the Mounts row. Accounts and the manager keep this key.
            self.keymap
                .add(KeyPhase::Bubble, drawer, CMD_EDITOR_MOUNT_ISOLATION);
        }
        let left = Chord::key(KeyCode::Left);
        self.keymap.remove(KeyPhase::Bubble, left);
        let save_confirm = self
            .status
            .as_deref()
            .is_some_and(|status| status.starts_with("Save settings"));
        if self.route == Route::Settings && !self.settings_save_preview && !save_confirm {
            self.keymap
                .add(KeyPhase::Bubble, left, CMD_SETTINGS_TAB_PREV);
        }
        let info = Chord::key(KeyCode::Char('i'));
        let debug = Chord::key(KeyCode::Char('d'));
        let trace = Chord::key(KeyCode::Char('t'));
        let cancel = Chord::key(KeyCode::Char('x'));
        self.keymap.remove(KeyPhase::Capture, info);
        self.keymap.remove(KeyPhase::Capture, debug);
        self.keymap.remove(KeyPhase::Capture, trace);
        self.keymap.remove(KeyPhase::Capture, cancel);
        let cockpit_keys = matches!(self.route, Route::Cockpit | Route::Launch)
            && self.launch.as_ref().is_none_or(|run| run.failure.is_none());
        if cockpit_keys {
            self.keymap.add(KeyPhase::Capture, info, CMD_COCKPIT_INFO);
            self.keymap.add(KeyPhase::Capture, debug, CMD_COCKPIT_DEBUG);
            // `t` reveals the approved run id. `d` still toggles debug telemetry.
            self.keymap.add(KeyPhase::Capture, trace, CMD_COCKPIT_TRACE);
            // `x` opens cancel confirm. Accounts keep `x` for remove.
            self.keymap
                .add(KeyPhase::Capture, cancel, CMD_COCKPIT_CANCEL);
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

    fn launch_available(&self) -> bool {
        let id = self
            .manager
            .selected()
            .or_else(|| self.world.workspaces.first().map(|workspace| workspace.id));
        let workspace = id.and_then(|id| self.world.workspace(id));
        self.world.has_offered_agents(workspace)
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
        Self::active_running_instance_id_ref(&self.active_instance, &self.world).map(str::to_owned)
    }

    fn active_running_instance_id_ref<'a>(
        active_instance: &'a Option<String>,
        world: &'a World,
    ) -> Option<&'a str> {
        active_instance.as_ref().and_then(|id| {
            world
                .instance(id)
                .filter(|instance| instance.status == InstanceStatus::Running)
                .map(|instance| instance.id.as_str())
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

    fn manager_list() -> List<'static, ManagerRow, impl Fn(&ManagerRow) -> ItemKey> {
        List::new(MANAGER_LIST).key(|row: &ManagerRow| row.key)
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

    fn account_filter_input() -> TextInput<'static> {
        TextInput::new(crate::screens::accounts::FILTER)
            .placeholder("Filter accounts")
            .placeholder_while_editing(true)
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
                let (title, placeholder) = match self.accounts.op_stage {
                    1 => ("Choose vault", "Search vaults…"),
                    2 => ("Choose item", "Search items…"),
                    3 => ("Choose field", "Search fields…"),
                    _ => ("Choose 1Password account", "Search accounts…"),
                };
                Self::account_picker().title(title).placeholder(placeholder)
            }
            Some(PickerMode::Capsule) => {
                let title = match self.pending_capsule_action {
                    Some(CapsuleAction::NewTab) => "New tab · Account for Claude Code",
                    Some(CapsuleAction::Split(SplitDir::Horizontal)) => {
                        "Split right · Account for Claude Code"
                    }
                    Some(CapsuleAction::Split(SplitDir::Vertical)) => {
                        "Split below · Account for Claude Code"
                    }
                    None => "Choose a configured account",
                };
                Self::account_picker().title(title)
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

    fn capsule_tab_menu_title(&self) -> String {
        self.active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .and_then(|daemon| {
                daemon
                    .tabs
                    .get(self.capsule_tab_title_index)
                    .map(|tab| daemon.tab_label(tab, &|_| None))
            })
            .unwrap_or_default()
    }

    fn capsule_tab_context<'a>(&self, position: Position, title: &'a str) -> ContextMenu<'a> {
        let menu = ContextMenu::at(CAPSULE_TAB_MENU, CAPSULE_TAB_ITEMS, position)
            .title(title)
            .keep_underlay();
        let Some(pane) = self
            .active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .and_then(|daemon| daemon.focused_pane())
        else {
            return menu;
        };
        // The tab menu takes the keys. The pane caret stays the steady bar.
        menu.retain_cursor(Self::capsule_viewport_id(pane))
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

    fn about_dialog() -> Dialog<'static> {
        Dialog::info(ABOUT_DIALOG, "About").body_rows(4)
    }

    fn about_lines(&self) -> Vec<String> {
        vec![
            "Product       jackin-preview · deterministic redesign preview".into(),
            format!("Scenario      {}", self.world.scenario.name()),
            "Construct     simulated · nothing here touches a real container".into(),
            "Design system Junie-inspired Ratatui components (termrock)".into(),
        ]
    }

    fn open_usage_detail(&mut self) {
        if self.usage.selected().is_none() {
            let id = self
                .world
                .accounts
                .sorted()
                .into_iter()
                .next()
                .map(|account| account.id.clone());
            self.usage.select(id);
        }
        let _ = self.usage.open_detail();
    }

    fn open_about(&mut self, cx: &mut Cx<'_>) {
        self.about_state = DialogState::default();
        self.about_open = true;
        cx.open_layer(ABOUT_DIALOG, Self::about_dialog().layer(cx));
        self.status = None;
    }

    fn prepare_quit_body(&mut self) {
        let running = self.world.running_count();
        self.quit_body = match running {
            0 => "No instances are running. The pending Construct entry is released.".into(),
            1 => "1 instance keeps running in the Construct. The host console closes; reconnect from a new terminal.".into(),
            2 => "2 instances keep running in the Construct. The host console closes; reconnect from a new terminal.".into(),
            count => format!(
                "{count} instances keep running in the Construct. The host console closes; reconnect from a new terminal."
            ),
        };
    }

    fn quit_dialog(body: &str) -> Dialog<'_> {
        Dialog::confirm(QUIT_DIALOG, "Exit jackin?", body).actions(&QUIT_ACTIONS)
    }

    fn open_quit(&mut self, cx: &mut Cx<'_>) {
        self.prepare_quit_body();
        self.quit_state = DialogState::default();
        self.quit_open = true;
        let body = self.quit_body.clone();
        let spec = Self::quit_dialog(&body).layer(cx);
        cx.open_layer(QUIT_DIALOG, spec);
    }

    fn cockpit_cancel_dialog() -> Dialog<'static> {
        Dialog::destructive(
            CANCEL_LAUNCH_DIALOG,
            "Cancel the launch?",
            "The pipeline stops at its current stage and the partially prepared instance is marked failed setup. Nothing is attached.",
        )
        .actions(&CANCEL_LAUNCH_ACTIONS)
        // The frozen app centred the card with ratatui Flex::Center: the odd
        // leftover row goes above, not to the floor split.
        .align(termrock::ScreenAlign::Balanced)
    }

    fn cockpit_info_dialog() -> Dialog<'static> {
        Dialog::info(COCKPIT_INFO_DIALOG, "Debug info")
            .width(66)
            .body_rows(8)
            // The frozen info card carried its title and meta in the border.
            .meta("read-only")
            .head_in_border()
            // The frozen app centred the card with a floor split of the free
            // rows (popup Center), not the Flex::Center split.
            .align(termrock::ScreenAlign::GeometricCenter)
    }

    fn info_facts(&self) -> Vec<(String, String)> {
        let role_full = self.selected_role();
        let role = role_full.rsplit('/').next().unwrap_or(role_full);
        let workspace = self
            .world
            .workspaces
            .first()
            .map_or("payments-platform", |workspace| workspace.name.as_str());
        let mut facts = Vec::new();
        if let Some(launch) = &self.launch
            && launch.container_ready()
        {
            facts.push(("Container".to_owned(), launch.container.clone()));
        }
        facts.push((
            "Target".to_owned(),
            format!("{role} into workspace {workspace}"),
        ));
        facts.push(("Role".to_owned(), role.to_owned()));
        facts.push((
            "Agent".to_owned(),
            "Claude Code · account Claude · Work".to_owned(),
        ));
        let run_id = self
            .launch
            .as_ref()
            .map(|launch| {
                if launch.trace_label.is_empty() {
                    launch.run_id.short()
                } else {
                    launch.trace_label.clone()
                }
            })
            .unwrap_or_else(|| "none".to_owned());
        facts.push(("Run id".to_owned(), run_id));
        facts.push(("jackin".to_owned(), "0.6.4 · preview".to_owned()));
        if self.cockpit_debug_open {
            facts.push((
                "Telemetry".to_owned(),
                "otlp://collector.internal:4317".to_owned(),
            ));
        }
        facts
    }

    fn open_cockpit_info(&mut self, cx: &mut Cx<'_>) {
        self.cockpit_info_state = DialogState::default();
        self.cockpit_info_open = true;
        cx.open_layer(COCKPIT_INFO_DIALOG, Self::cockpit_info_dialog().layer(cx));
        self.status = Some("Debug info".into());
    }

    fn close_cockpit_info(&mut self, cx: &mut Cx<'_>) {
        if cx.is_open(COCKPIT_INFO_DIALOG) {
            cx.close_layer(COCKPIT_INFO_DIALOG, Some(ActionKey::CLOSE));
        }
        self.cockpit_info_open = false;
        self.status = None;
    }

    fn editor_exit_dialog() -> Dialog<'static> {
        Dialog::confirm(
            EDITOR_EXIT_DIALOG,
            "Leave the editor?",
            "Save changes before leaving?",
        )
        .actions(&EDITOR_EXIT_ACTIONS)
    }

    fn open_editor_exit(&mut self, cx: &mut Cx<'_>) {
        self.editor_exit_state = DialogState::default();
        self.editor_exit_open = true;
        self.editor_leaf_focused = false;
        let dialog = Self::editor_exit_dialog();
        let focus = dialog.action_id(0);
        cx.open_layer(EDITOR_EXIT_DIALOG, dialog.layer(cx).initial_focus(focus));
        self.status = Some("Save changes before leaving?".into());
    }

    fn close_editor_exit(&mut self, cx: &mut Cx<'_>) {
        if cx.is_open(EDITOR_EXIT_DIALOG) {
            cx.close_layer(EDITOR_EXIT_DIALOG, Some(EXIT_CANCEL));
        }
        self.editor_exit_open = false;
        if self.status.as_deref() == Some("Save changes before leaving?") {
            self.status = None;
        }
    }

    fn dismiss_editor_env_form(&mut self, cx: &mut Cx<'_>) {
        self.editor.clear_env_form();
        self.editor_leaf_focused = false;
        cx.focus(crate::screens::editor::SAVE);
    }

    fn mounts_dirty_key_applies(&self) -> bool {
        self.route == Route::Editor
            && self.editor.tab == EditorTab::Mounts
            && !self.editor.dirty
            && self.status.is_none()
            && !self.editor.preview_open
            && !self.editor.env_form_open
            && !self.editor_role_picker
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.editor_exit_open
    }

    fn dirty_mounts_tab(&mut self) {
        // One edit: the selected host mount becomes read-only and cloned.
        // `r` and `i` both land here. A second press does not undo it.
        if let Some(mount) = self.editor.pending.mounts.first_mut() {
            mount.readonly = true;
            mount.isolation = crate::domain::workspace::Isolation::Clone;
        }
        self.editor.mark_dirty();
    }

    fn focus_editor_tab(&mut self, cx: &mut Cx<'_>) {
        self.editor_leaf_focused = match self.editor.tab {
            EditorTab::Mounts => {
                cx.focus(EDITOR_MOUNT_EDIT);
                true
            }
            EditorTab::Roles => {
                cx.focus(EDITOR_ROLE_EDIT);
                true
            }
            EditorTab::Accounts => {
                cx.focus(EDITOR_ACCOUNTS_LIST);
                true
            }
            EditorTab::Environments | EditorTab::General => false,
        };
    }

    fn open_cancel_launch(&mut self, cx: &mut Cx<'_>) {
        self.cancel_state = DialogState::default();
        self.cancel_open = true;
        self.cancel_from_x = false;
        cx.open_layer(
            CANCEL_LAUNCH_DIALOG,
            Self::cockpit_cancel_dialog().layer(cx),
        );
    }

    fn acknowledge_launch_failure(&mut self) {
        let run_id = self.launch.as_ref().map(|run| run.run_id);
        self.launch = None;
        self.route = Route::Manager;
        self.ensure_manager_rows();
        if let Some(run_id) = run_id {
            let failed_id = self
                .world
                .instances
                .iter()
                .find(|instance| instance.run_id == run_id)
                .map(|instance| instance.id.clone());
            if let Some(id) = failed_id
                && let Some((index, row)) = self.manager_rows_cache.iter().enumerate().find(|(_, row)| {
                    matches!(&row.domain, ManagerRowKey::Instance(existing) if existing == &id)
                })
            {
                let key = row.key;
                let domain = row.domain.clone();
                self.manager.list.set_cursor(index, key);
                self.manager.select_row(domain);
            }
        }
        let running = self.world.running_count();
        self.status = Some(if running == 0 {
            "Launch failed · the Construct is empty".into()
        } else {
            let noun = if running == 1 {
                "instance"
            } else {
                "instances"
            };
            format!("Launch failed · {running} {noun} still running in the Construct")
        });
    }

    fn manager_instances(
        &self,
        workspace: Option<crate::domain::workspace::WorkspaceId>,
    ) -> impl Iterator<Item = &crate::domain::instance::Instance> {
        self.world
            .instances
            .iter()
            .filter(move |instance| instance.workspace == workspace && !instance.status.hidden())
    }

    fn build_manager_rows(&self) -> Vec<ManagerRow> {
        let mut rows = Vec::new();
        for workspace in &self.world.workspaces {
            let expanded = self.manager.is_expanded(workspace.id);
            let marker = if expanded { "▾" } else { "▸" };
            let count = self.manager_instances(Some(workspace.id)).count();
            rows.push(ManagerRow::new(
                ManagerRowKey::Workspace(workspace.id),
                format!(
                    "{marker} {} · {count} instance{}",
                    workspace.name,
                    if count == 1 { "" } else { "s" }
                ),
            ));
            if expanded {
                for instance in self.manager_instances(Some(workspace.id)) {
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
        rows.extend(self.manager_instances(None).map(|instance| {
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
        }));
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

    fn move_manager_cursor(&mut self, forward: bool) {
        self.ensure_manager_rows();
        let len = self.manager_rows_cache.len();
        if len == 0 {
            return;
        }
        let current = self
            .manager_rows_cache
            .iter()
            .position(|row| self.manager.list.cursor() == Some(row.key))
            .unwrap_or(0);
        let next = if forward {
            current.saturating_add(1).min(len.saturating_sub(1))
        } else {
            current.saturating_sub(1)
        };
        let Some(row) = self.manager_rows_cache.get(next) else {
            return;
        };
        let key = row.key;
        let domain = row.domain.clone();
        self.manager.list.set_cursor(next, key);
        self.manager.select_row(domain);
    }

    fn move_accounts_cursor(&mut self, forward: bool) {
        let len = self.account_rows().len();
        if len == 0 {
            return;
        }
        let current = self
            .accounts
            .list
            .cursor()
            .and_then(|key| match key {
                ItemKey::Index(index) => Some(index),
                _ => None,
            })
            .unwrap_or(0);
        let next = if forward {
            current.saturating_add(1).min(len.saturating_sub(1))
        } else {
            current.saturating_sub(1)
        };
        self.accounts.list.set_cursor(next, ItemKey::index(next));
        let previous = self.accounts.selected_id.clone();
        self.accounts.selected_id = selected_account_id(&self.world, self.accounts.list.cursor());
        if self.accounts.selected_id != previous {
            self.note_selected_account();
        }
    }

    fn note_selected_account(&mut self) {
        self.status = self
            .accounts
            .selected_id
            .as_deref()
            .and_then(|id| self.world.accounts.get(id))
            .map(account_selection_status);
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

    fn account_matches_filter(&self, account: &Account) -> bool {
        let query = self.accounts.filter_query.trim();
        if !self.accounts.filtering || query.is_empty() {
            return true;
        }
        let needle = query.to_lowercase();
        let haystack = format!(
            "{} {} {} {}",
            account.provider.label(),
            account.title(),
            account.status_word(),
            account.source.safe_detail()
        )
        .to_lowercase();
        haystack.contains(&needle)
    }

    fn reconcile_filtered_account(&mut self) {
        if !self.accounts.filtering {
            return;
        }
        let visible: Vec<String> = self
            .world
            .accounts
            .sorted()
            .into_iter()
            .filter(|account| self.account_matches_filter(account))
            .map(|account| account.id.clone())
            .collect();
        let selected_visible = self
            .accounts
            .selected_id
            .as_ref()
            .is_some_and(|id| visible.iter().any(|visible_id| visible_id == id));
        if !selected_visible {
            self.accounts.selected_id = visible.into_iter().next();
        }
    }

    fn account_rows(&self) -> Vec<String> {
        let mut rows = vec!["Overview · Health · Registration · Quota".to_owned()];
        let mut provider = None;
        for account in self.world.accounts.sorted() {
            if !self.account_matches_filter(account) {
                continue;
            }
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
        self.launch_picker_open = true;
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

        let snapshot = match crate::sim::fixtures::live_capsule() {
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

        let mut instance = crate::sim::fixtures::fixture_instance(
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

    fn capsule_viewport(pane_id: PaneId, focused: bool) -> TextViewport<'static> {
        static PARTS: std::sync::OnceLock<[(Part, termrock::StylePatch); 2]> =
            std::sync::OnceLock::new();
        let parts = PARTS.get_or_init(|| {
            [
                (
                    Part::TRACK,
                    termrock::StylePatch::new().set_fg(termrock::Role::Custom(
                        termrock::Color::from_u32(0x0047_4747),
                    )),
                ),
                (
                    Part::THUMB,
                    termrock::StylePatch::new().set_fg(termrock::Role::Custom(
                        termrock::Color::from_u32(0x0080_8080),
                    )),
                ),
            ]
        });
        TextViewport::new(CAPSULE_PANES.item(ItemKey::num(pane_id)))
            .click_selects_word(true)
            .reserve_gutter(false)
            .scroll_gap(1)
            .caret_when_unfocused(focused)
            .bar_cursor(true)
            .patch_part(parts)
    }

    fn role_choose_button(&self) -> Button<'_> {
        Button::new(ROLE_CHOOSE, self.selected_role())
    }

    fn with_capsule_help<R>(f: impl FnOnce(&[HelpSection<'_>]) -> R) -> R {
        let pane = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Chord(Chord::with(
                        KeyCode::Char('b'),
                        KeyModifiers::CONTROL,
                    )),
                    label: "Prefix commands",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('y'))),
                    label: "Copy selection",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::PageUp)),
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
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('%'))),
                    label: "Split right",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('"'))),
                    label: "Split below",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('h'))),
                    label: "Focus left",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('z'))),
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
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('c'))),
                    label: "New tab",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('d'))),
                    label: "Detach",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char(','))),
                    label: "Rename tab",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('&'))),
                    label: "Close tab",
                    priority: 60,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::with(
                        KeyCode::Char('q'),
                        KeyModifiers::CONTROL,
                    )),
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
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Left)),
                    label: "Previous tab",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Right)),
                    label: "Next tab",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::F(10))),
                    label: "Menu",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::with(
                        KeyCode::Char('\\'),
                        KeyModifiers::CONTROL,
                    )),
                    label: "Command palette",
                    priority: 60,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('?'))),
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
        if input.is_empty() {
            return;
        }
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
            // The prefix line is transient. The open tab menu keeps the session
            // status row, the same row the idle capsule paints.
            self.status = None;
            self.capsule_tab_title_index = self.active_capsule_tab_index();
            self.capsule_tab_menu_open = true;
            self.capsule_tab_menu_state = MenuState::default();
            self.capsule_tab_menu_pos = Position::new(1, 2);
            let title = self.capsule_tab_menu_title();
            cx.open_layer(
                CAPSULE_TAB_MENU,
                self.capsule_tab_context(self.capsule_tab_menu_pos, &title)
                    .layer(cx),
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
            ' ' => Some(CMD_CAPSULE_PALETTE),
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

    fn project_term(term: &crate::sim::pty::TextViewport, out: &mut ProjectedText) {
        out.clear();
        for line in &term.lines {
            if line.is_empty() {
                out.push_line([("", None, Modifier::empty())]);
                continue;
            }
            out.push_line(line.iter().map(|span| {
                let modifier = if span.bold {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                };
                (
                    span.text.as_str(),
                    Some(capsule_span_role(span.tone)),
                    modifier,
                )
            }));
        }
    }

    fn ensure_capsule_transcript(caches: &mut CapsuleFrameCaches, pane: &Pane) {
        let revision = pane.term.revision();
        let fresh = caches
            .transcripts
            .get(&pane.id)
            .is_some_and(|(cached, _)| *cached == revision);
        if !fresh {
            let mut text = ProjectedText::default();
            Self::project_term(&pane.term, &mut text);
            caches.transcripts.insert(pane.id, (revision, text));
        }
    }

    fn hash_topology(hasher: &mut DefaultHasher, node: &PaneNode) {
        match node {
            PaneNode::Leaf(id) => {
                hasher.write_u8(0);
                hasher.write_u64(*id);
            }
            PaneNode::Split {
                dir,
                split,
                first,
                second,
            } => {
                hasher.write_u8(1);
                hasher.write_u8(match dir {
                    SplitDir::Horizontal => 0,
                    SplitDir::Vertical => 1,
                });
                hasher.write_u16(split.percent);
                hasher.write_u16(split.min_first);
                hasher.write_u16(split.min_second);
                hasher.write_u8(match split.maximized {
                    Maximized::None => 0,
                    Maximized::First => 1,
                    Maximized::Second => 2,
                });
                Self::hash_topology(hasher, first);
                Self::hash_topology(hasher, second);
            }
        }
    }

    fn hash_tab_pane(
        hasher: &mut DefaultHasher,
        daemon: &Daemon,
        accounts: &AccountRegistry,
        pane_id: PaneId,
    ) {
        hasher.write_u64(pane_id);
        let Some(pane) = daemon.pane(pane_id) else {
            hasher.write_u8(0);
            return;
        };
        hasher.write_u8(1);
        match pane.proc.agent {
            None => hasher.write_u8(0),
            Some(agent) => {
                hasher.write_u8(1);
                hasher.write(agent.label().as_bytes());
            }
        }
        match pane.proc.account.as_ref().and_then(|id| accounts.get(id)) {
            None => hasher.write_u8(0),
            Some(account) => {
                hasher.write_u8(1);
                hasher.write(account.display_name.as_bytes());
            }
        }
        pane.state().hash(hasher);
    }

    fn hash_tab_labels(
        hasher: &mut DefaultHasher,
        daemon: &Daemon,
        accounts: &AccountRegistry,
        node: &PaneNode,
    ) {
        match node {
            PaneNode::Leaf(id) => Self::hash_tab_pane(hasher, daemon, accounts, *id),
            PaneNode::Split { first, second, .. } => {
                Self::hash_tab_labels(hasher, daemon, accounts, first);
                Self::hash_tab_labels(hasher, daemon, accounts, second);
            }
        }
    }

    fn capsule_tabs_fingerprint(
        world: &World,
        instance_id: Option<&str>,
        tab_title: &str,
        tab_title_index: usize,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        match instance_id.and_then(|id| world.daemons.get(id)) {
            None => hasher.write_u8(0),
            Some(daemon) => {
                hasher.write_u8(1);
                hasher.write_usize(daemon.tabs.len());
                hasher.write_usize(daemon.active);
                for tab in &daemon.tabs {
                    match &tab.custom_label {
                        None => hasher.write_u8(0),
                        Some(label) => {
                            hasher.write_u8(1);
                            hasher.write(label.as_bytes());
                        }
                    }
                    Self::hash_tab_labels(&mut hasher, daemon, &world.accounts, &tab.root);
                }
            }
        }
        hasher.write(tab_title.as_bytes());
        hasher.write_usize(tab_title_index);
        hasher.finish()
    }

    fn build_capsule_tabs(
        world: &World,
        instance_id: Option<&str>,
        tab_title: &str,
        tab_title_index: usize,
    ) -> Vec<String> {
        let mut dynamic = instance_id
            .and_then(|id| world.daemons.get(id))
            .map(|daemon| {
                daemon
                    .tabs
                    .iter()
                    .map(|tab| {
                        let mut label = daemon.tab_label(tab, &|pane| {
                            pane.proc
                                .account
                                .as_ref()
                                .and_then(|id| world.accounts.get(id))
                                .map(|account| account.display_name.clone())
                        });
                        let glyph = match daemon.tab_state(tab) {
                            crate::domain::instance::AgentState::Blocked => "●",
                            crate::domain::instance::AgentState::Working => "▶",
                            crate::domain::instance::AgentState::Idle => "◆",
                            crate::domain::instance::AgentState::Done
                            | crate::domain::instance::AgentState::Unknown => "",
                        };
                        if !glyph.is_empty() {
                            label.push(' ');
                            label.push_str(glyph);
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
        if !tab_title.is_empty()
            && let Some(label) = dynamic.get_mut(tab_title_index)
        {
            *label = tab_title.to_owned();
        }
        if dynamic.len() < 3 {
            dynamic.push("docs ●".into());
        }
        dynamic
            .into_iter()
            .enumerate()
            .map(|(index, label)| format!("{} {label} ", index.saturating_add(1)))
            .collect()
    }

    fn ensure_capsule_tabs(
        caches: &mut CapsuleFrameCaches,
        world: &World,
        instance_id: Option<&str>,
        tab_title: &str,
        tab_title_index: usize,
    ) {
        let fingerprint =
            Self::capsule_tabs_fingerprint(world, instance_id, tab_title, tab_title_index);
        let fresh = caches
            .tabs
            .as_ref()
            .is_some_and(|(cached, _)| *cached == fingerprint);
        if !fresh {
            caches.tabs = Some((
                fingerprint,
                Self::build_capsule_tabs(world, instance_id, tab_title, tab_title_index),
            ));
        }
    }

    fn pane_title_fingerprint(accounts: &AccountRegistry, pane: &Pane) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(pane.id);
        match pane.proc.agent {
            None => hasher.write_u8(0),
            Some(agent) => {
                hasher.write_u8(1);
                hasher.write(agent.label().as_bytes());
            }
        }
        match pane.proc.account.as_ref().and_then(|id| accounts.get(id)) {
            None => hasher.write_u8(0),
            Some(account) => {
                hasher.write_u8(1);
                hasher.write(account.display_name.as_bytes());
            }
        }
        pane.state().hash(&mut hasher);
        hasher.finish()
    }

    fn build_pane_title(accounts: &AccountRegistry, pane: &Pane) -> String {
        let label = pane
            .proc
            .account
            .as_ref()
            .and_then(|id| accounts.get(id))
            .map_or_else(
                || pane.label(),
                |account| format!("{} ({})", pane.label(), account.display_name),
            );
        format!(" {label} ")
    }

    fn ensure_pane_title(caches: &mut CapsuleFrameCaches, accounts: &AccountRegistry, pane: &Pane) {
        let fingerprint = Self::pane_title_fingerprint(accounts, pane);
        let fresh = caches
            .titles
            .get(&pane.id)
            .is_some_and(|(cached, _)| *cached == fingerprint);
        if !fresh {
            caches.titles.insert(
                pane.id,
                (fingerprint, Self::build_pane_title(accounts, pane)),
            );
        }
    }

    fn pane_layouts_fingerprint(tab: &Tab, area: Rect) -> u64 {
        let mut hasher = DefaultHasher::new();
        Self::hash_topology(&mut hasher, &tab.root);
        match tab.zoomed {
            None => hasher.write_u8(0),
            Some(id) => {
                hasher.write_u8(1);
                hasher.write_u64(id);
            }
        }
        hasher.write_u16(area.x);
        hasher.write_u16(area.y);
        hasher.write_u16(area.width);
        hasher.write_u16(area.height);
        hasher.finish()
    }

    fn build_pane_layouts(tab: &Tab, area: Rect) -> Vec<(PaneId, Rect)> {
        let mut layouts = Vec::new();
        if let Some(zoomed) = tab.zoomed {
            layouts.push((zoomed, area));
        } else {
            tab.root
                .layout(area, &mut layouts, &mut Vec::new(), &mut Vec::new());
        }
        layouts
    }

    fn ensure_pane_layouts(caches: &mut CapsuleFrameCaches, tab: &Tab, area: Rect) {
        let fingerprint = Self::pane_layouts_fingerprint(tab, area);
        let fresh = caches
            .layouts
            .as_ref()
            .is_some_and(|(cached, _)| *cached == fingerprint);
        if !fresh {
            caches.layouts = Some((fingerprint, Self::build_pane_layouts(tab, area)));
        }
    }

    fn ensure_tab_projections(
        caches: &mut CapsuleFrameCaches,
        daemon: &Daemon,
        accounts: &AccountRegistry,
        node: &PaneNode,
    ) {
        match node {
            PaneNode::Leaf(id) => {
                if let Some(pane) = daemon.pane(*id) {
                    Self::ensure_capsule_transcript(caches, pane);
                    Self::ensure_pane_title(caches, accounts, pane);
                }
            }
            PaneNode::Split { first, second, .. } => {
                Self::ensure_tab_projections(caches, daemon, accounts, first);
                Self::ensure_tab_projections(caches, daemon, accounts, second);
            }
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
        Self::ensure_tab_projections(
            self.capsule_frame.get_mut(),
            daemon,
            &self.world.accounts,
            &tab.root,
        );
        let sources = tab
            .leaves()
            .into_iter()
            .filter_map(|pane_id| {
                daemon.pane(pane_id).map(|pane| {
                    (
                        pane_id,
                        pane.term.lines.len(),
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
        self.capsule_frame
            .get_mut()
            .transcripts
            .retain(|pane_id, _| pane_ids.contains(pane_id));
        self.capsule_frame
            .get_mut()
            .titles
            .retain(|pane_id, _| pane_ids.contains(pane_id));

        let mut focused_pane = None;
        for (pane_id, line_count, caret, focused, revision, retained) in sources {
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
                .unwrap_or(line_count);
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
            } else if previous_len > line_count {
                state.clear_selection();
                state.invalidate();
            }
            if previous_revision.is_some_and(|previous| previous != revision) {
                state.invalidate();
            }
            state.set_caret(focused.then_some(caret).flatten());
            let frame = self.capsule_frame.borrow();
            let Some((_, projected)) = frame.transcripts.get(&pane_id) else {
                continue;
            };
            let viewport_response =
                Self::capsule_viewport(pane_id, focused).update_projected(cx, state, projected);
            let owns_focus = clicked || viewport_response.focused();
            let viewport_action = viewport_response.action_ref().cloned();
            *result |= viewport_response.erase();
            let copied = match viewport_action {
                Some(ViewportAction::Copy(text)) => Some(text),
                Some(ViewportAction::SelectionChanged) => {
                    let mut text = String::new();
                    self.capsule_viewports.get(&pane_id).and_then(|state| {
                        state
                            .copy_from_projected(projected, &mut text)
                            .then_some(text)
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
            self.capsule_viewport_lengths.insert(pane_id, line_count);
            self.capsule_viewport_revisions.insert(pane_id, revision);
            self.capsule_viewport_retained.insert(pane_id, retained);
        }
        if let Some(pane_id) = focused_pane {
            self.set_capsule_focused_pane(pane_id);
            self.capsule_viewport_focused = true;
        }
    }

    fn launch_failure_dialog() -> Dialog<'static> {
        Dialog::info(LAUNCH_FAILURE_DIALOG, "Launch failed")
            .width(70)
            .body_rows(12)
    }

    fn launch_failure_lines(&self) -> Vec<String> {
        let Some(failure) = self
            .launch
            .as_ref()
            .and_then(|launch| launch.failure.clone())
        else {
            return Vec::new();
        };
        let mut lines = vec![
            failure.summary,
            format!("Stage · {}", failure.stage.label()),
            format!("Next step · {}", failure.next_step),
        ];
        lines.extend(failure.detail);
        lines
    }

    fn settle_launch_failure(&mut self, cx: &mut Cx<'_>) {
        if cx.is_open(LAUNCH_FAILURE_DIALOG) {
            cx.close_layer(LAUNCH_FAILURE_DIALOG, Some(ActionKey::CLOSE));
        }
        self.launch_failure_open = false;
        self.acknowledge_launch_failure();
    }

    fn update_overlays(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();

        if self
            .launch
            .as_ref()
            .is_some_and(|launch| launch.failure.is_some())
            && !self.launch_failure_open
        {
            self.launch_failure_open = true;
        }
        if self.launch_failure_open && !cx.is_open(LAUNCH_FAILURE_DIALOG) {
            self.launch_failure_state = DialogState::default();
            cx.open_layer(
                LAUNCH_FAILURE_DIALOG,
                Self::launch_failure_dialog().layer(cx),
            );
            result |= Response::changed();
        }
        if self.launch_failure_open {
            let dialog = Self::launch_failure_dialog();
            let response = dialog.update(cx, &mut self.launch_failure_state);
            let action = response.action_ref().copied();
            result |= response.erase();
            match action {
                Some(DialogAction::Dismissed(_) | DialogAction::Action(ActionKey::CLOSE)) => {
                    self.settle_launch_failure(cx);
                    result |= Response::changed();
                }
                None if !cx.is_open(LAUNCH_FAILURE_DIALOG) => {
                    self.settle_launch_failure(cx);
                    result |= Response::changed();
                }
                Some(DialogAction::Action(_)) | None => {}
            }
        }

        if self.capsule_help_open {
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
        }

        let info = Self::container_info_dialog();
        let response = info.update(cx, &mut self.container_info_state);
        if response.action_ref().is_some() {
            self.status = None;
        }
        result |= response.erase();

        if self.about_open {
            let about = Self::about_dialog();
            let response = about.update(cx, &mut self.about_state);
            let action = response.action_ref().copied();
            result |= response.erase();
            match action {
                Some(DialogAction::Dismissed(_)) => {
                    self.about_open = false;
                    self.status = None;
                    result |= Response::changed();
                }
                Some(DialogAction::Action(key)) => {
                    if cx.is_open(ABOUT_DIALOG) {
                        cx.close_layer(ABOUT_DIALOG, Some(key));
                    }
                    result |= Response::changed();
                }
                None if !cx.is_open(ABOUT_DIALOG) => {
                    self.about_open = false;
                    self.status = None;
                    result |= Response::changed();
                }
                None => {}
            }
        }

        if self.quit_open {
            let body = self.quit_body.clone();
            let quit = Self::quit_dialog(&body);
            let response = quit.update(cx, &mut self.quit_state);
            let action = response.action_ref().copied();
            result |= response.erase();
            match action {
                Some(DialogAction::Action(ActionKey::CONFIRM)) => {
                    if cx.is_open(QUIT_DIALOG) {
                        cx.close_layer(QUIT_DIALOG, Some(ActionKey::CONFIRM));
                    }
                    self.quit_open = false;
                    self.quit = true;
                    cx.quit();
                    result |= Response::changed();
                }
                Some(DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_)) => {
                    if cx.is_open(QUIT_DIALOG) {
                        cx.close_layer(QUIT_DIALOG, Some(ActionKey::CANCEL));
                    }
                    self.quit_open = false;
                    result |= Response::changed();
                }
                None if !cx.is_open(QUIT_DIALOG) => {
                    self.quit_open = false;
                    result |= Response::changed();
                }
                Some(DialogAction::Action(_)) | None => {}
            }
        }

        if self.cancel_open {
            let dialog = Self::cockpit_cancel_dialog();
            let response = dialog.update(cx, &mut self.cancel_state);
            let action = response.action_ref().copied();
            result |= response.erase();
            match action {
                Some(DialogAction::Action(ActionKey::CONFIRM)) => {
                    if cx.is_open(CANCEL_LAUNCH_DIALOG) {
                        cx.close_layer(CANCEL_LAUNCH_DIALOG, Some(ActionKey::CONFIRM));
                    }
                    self.cancel_open = false;
                    self.cancel_from_x = false;
                    self.launch = None;
                    self.route = Route::Manager;
                    self.status = Some("Launch cancelled · nothing was attached".into());
                    cx.focus(MANAGER_LIST);
                    result |= Response::changed();
                }
                Some(DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_)) => {
                    if cx.is_open(CANCEL_LAUNCH_DIALOG) {
                        cx.close_layer(CANCEL_LAUNCH_DIALOG, Some(ActionKey::CANCEL));
                    }
                    self.cancel_open = false;
                    self.cancel_from_x = false;
                    result |= Response::changed();
                }
                None if !cx.is_open(CANCEL_LAUNCH_DIALOG) => {
                    self.cancel_open = false;
                    self.cancel_from_x = false;
                    result |= Response::changed();
                }
                Some(DialogAction::Action(_)) | None => {}
            }
        }

        if self.cockpit_info_open {
            let info = Self::cockpit_info_dialog();
            let response = info.update(cx, &mut self.cockpit_info_state);
            let action = response.action_ref().copied();
            result |= response.erase();
            match action {
                Some(DialogAction::Dismissed(_)) => {
                    self.cockpit_info_open = false;
                    self.status = None;
                    result |= Response::changed();
                }
                Some(DialogAction::Action(key)) => {
                    if cx.is_open(COCKPIT_INFO_DIALOG) {
                        cx.close_layer(COCKPIT_INFO_DIALOG, Some(key));
                    }
                    result |= Response::changed();
                }
                None if !cx.is_open(COCKPIT_INFO_DIALOG) => {
                    self.cockpit_info_open = false;
                    self.status = None;
                    result |= Response::changed();
                }
                None => {}
            }
        }

        if self.editor_exit_open {
            let dialog = Self::editor_exit_dialog();
            let response = dialog.update(cx, &mut self.editor_exit_state);
            let action = response.action_ref().copied();
            result |= response.erase();
            match action {
                Some(DialogAction::Action(EXIT_SAVE)) => {
                    if cx.is_open(EDITOR_EXIT_DIALOG) {
                        cx.close_layer(EDITOR_EXIT_DIALOG, Some(EXIT_SAVE));
                    }
                    self.editor_exit_open = false;
                    self.editor.mark_dirty();
                    self.editor.open_preview();
                    cx.focus(crate::screens::editor::SAVE);
                    self.status = Some("Save workspace · preview changes before commit".into());
                    result |= Response::changed();
                }
                Some(DialogAction::Action(EXIT_DISCARD)) => {
                    if cx.is_open(EDITOR_EXIT_DIALOG) {
                        cx.close_layer(EDITOR_EXIT_DIALOG, Some(EXIT_DISCARD));
                    }
                    self.editor_exit_open = false;
                    self.editor.discard_draft();
                    self.route = Route::Manager;
                    cx.focus(MANAGER_LIST);
                    self.status = Some("Discarded 1 change".into());
                    result |= Response::changed();
                }
                Some(DialogAction::Action(EXIT_CANCEL) | DialogAction::Dismissed(_)) => {
                    self.close_editor_exit(cx);
                    result |= Response::changed();
                }
                None if !cx.is_open(EDITOR_EXIT_DIALOG) => {
                    self.editor_exit_open = false;
                    if self.status.as_deref() == Some("Save changes before leaving?") {
                        self.status = None;
                    }
                    result |= Response::changed();
                }
                Some(DialogAction::Action(_)) | None => {}
            }
        }

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
                let role = self
                    .roles
                    .get(index)
                    .map_or_else(String::new, |role| role.key.clone());
                self.editor_role_picker = false;
                self.editor_env_role = Some(role.clone());
                self.editor.open_env_form();
                self.editor_leaf_focused = true;
                cx.focus(crate::screens::editor::ENV_KEY);
                self.status = Some(format!("Add role override · {}", self.role_label(&role)));
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
        if !cx.is_open(crate::screens::manager::AGENT_PICKER) {
            self.launch_picker_open = false;
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
        if cx.activation_key() == Some(ActivationKey::Enter)
            && (self.launch_picker_enter(cx) || self.hard_launch_picker_enter(cx))
        {
            // Focus sits on the manager shell, so the list never sees Enter.
            self.open_agent_picker(cx);
            return Response::changed();
        }
        let list =
            Self::manager_list().update(cx, &mut self.manager.list, &self.manager_rows_cache);
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
        let launch_disabled = !self.launch_available();
        let button = Self::launch_button(launch_disabled).update(cx);
        let chosen = button.activated();
        result |= button.erase();
        if chosen {
            self.open_agent_picker(cx);
        }
        result
    }

    fn update_accounts(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.accounts.filtering && !self.accounts.form_open {
            let filter = Self::account_filter_input().update(
                cx,
                &mut self.accounts.filter_input,
                &mut self.accounts.filter_query,
            );
            let cancelled = filter
                .action_ref()
                .is_some_and(|action| *action == TextAction::Cancelled);
            let committed = filter
                .action_ref()
                .is_some_and(|action| *action == TextAction::Committed);
            let mut result = filter.erase();

            if let Some(draft) = self.accounts.filter_input.draft_text()
                && self.accounts.filter_query != draft
            {
                self.accounts.filter_query = draft.to_owned();
                self.reconcile_filtered_account();
                result |= Response::changed();
            }
            if committed {
                // Enter leaves the query. The approved filter frame has no caret,
                // so focus must leave the field before draw.
                cx.focus(ACCOUNTS_LIST);
                result |= Response::changed();
            }
            if cancelled {
                self.accounts.filtering = false;
                self.accounts.filter_query.clear();
                self.accounts.filter_input = TextInputState::default();
                result |= Response::changed();
            }
            return result;
        }
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
            self.note_selected_account();
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
        self.status = Some(match self.editor.begin_save(&mut self.world) {
            Ok(_) => format!("Saving {}…", self.editor.pending.name),
            Err(error) => error.to_string(),
        });
    }

    fn toggle_settings_trust_row(&mut self) {
        if self.settings_trust_origin.is_empty() {
            self.settings_trust_origin = self
                .world
                .global
                .trust
                .iter()
                .map(|row| row.trusted)
                .collect();
        }
        if let Some(row) = self.world.global.trust.get_mut(self.settings_trust_cursor) {
            row.trusted = !row.trusted;
        }
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
            self.settings_save_preview = false;
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
            if key_cancelled {
                self.dismiss_editor_env_form(cx);
                return result;
            }
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
            if value_cancelled {
                self.dismiss_editor_env_form(cx);
                return result;
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
                    .contains(termrock::StateFlags::FOCUSED);
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
                .contains(termrock::StateFlags::FOCUSED);
        result |= save.erase();
        let confirm = Self::editor_save_confirm_button(self.world.workspaces.is_empty()).update(cx);
        let confirm_chosen = confirm.activated()
            && cx
                .state(EDITOR_SAVE_CONFIRM)
                .contains(termrock::StateFlags::FOCUSED);
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
            self.open_cancel_launch(cx);
            return result | Response::changed();
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
                    self.status = Some(format!("{} failed", failure.stage.label()));
                    self.launch_failure_open = true;
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
            let title = self.capsule_tab_menu_title();
            cx.open_layer(
                CAPSULE_TAB_MENU,
                self.capsule_tab_context(self.capsule_tab_menu_pos, &title)
                    .layer(cx),
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

            self.capsule_menu_state = MenuState::default();
            cx.close_layer(CAPSULE_MENU_BAR, None);
            if action == CMD_INSPECT_CHANGES {
                cx.focus(CAPSULE_INPUT);
            }
        }
        result |= menu_response.erase();

        let title = self.capsule_tab_menu_title();
        let context = self.capsule_tab_context(self.capsule_tab_menu_pos, &title);
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
                let title = self.capsule_tab_menu_title();
                cx.open_layer(
                    CAPSULE_TAB_MENU,
                    self.capsule_tab_context(pos, &title).layer(cx),
                );
                result |= Response::changed();
            } else if let Some((pos, index)) = tab_context {
                self.capsule_tab_title_index = index;
                self.capsule_tab_menu_pos = pos;
                self.capsule_tab_menu_open = true;
                self.capsule_tab_menu_state = MenuState::default();
                let title = self.capsule_tab_menu_title();
                cx.open_layer(
                    CAPSULE_TAB_MENU,
                    self.capsule_tab_context(pos, &title).layer(cx),
                );
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

        let pending_capsule_confirm = self.pending_capsule_action.is_some()
            && !self.capsule_input_state.is_editing()
            && cx
                .intents(CAPSULE_INPUT)
                .any(|intent| matches!(intent, Intent::Binding(_)));
        if pending_capsule_confirm {
            // The 120×40 vertical split keeps the agent picker. A leftover
            // binding from `"` must not replace it with the account list.
            if self.capsule_split_vertical_frame(cx.theme(), cx.viewport()) {
                cx.focus(CAPSULE_INPUT);
                return result;
            }
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
            && !self.capsule_menu_state.is_open()
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
                if self.capsule_split_vertical_host(cx.theme(), cx.viewport())
                    && matches!(action, CapsuleAction::Split(SplitDir::Vertical))
                {
                    self.pending_capsule_action = Some(action);
                } else {
                    self.open_capsule_account_picker(cx, action);
                    self.status = Some("New tab · Account for Claude Code".into());
                }
            } else if self.exit_choice.is_some() {
                if let Some(response) = self.update_command(cx, CMD_EXIT_CONFIRM) {
                    result |= response;
                }
            } else {
                self.commit_capsule_input();
            }
            result |= Response::changed();
        }
        Self::ensure_capsule_tabs(
            self.capsule_frame.get_mut(),
            &self.world,
            Self::active_running_instance_id_ref(&self.active_instance, &self.world),
            &self.capsule_tab_title,
            self.capsule_tab_title_index,
        );
        let tabs_response = {
            let frame = self.capsule_frame.borrow();
            let tabs = frame
                .tabs
                .as_ref()
                .map_or(&[] as &[String], |(_, tabs)| tabs.as_slice());
            Tabs::new(CAPSULE_TABS).update(cx, &mut self.tabs_state, tabs)
        };
        if let Some(TabsAction::Activated(ItemKey::Index(index))) =
            tabs_response.action_ref().copied()
        {
            self.activate_capsule_tab(index);
            self.capsule_tab_title_index = index;
            self.capsule_viewport_focused = false;
        }
        if self.historical_capsule_frame()
            && let Some(pane) = self
                .active_running_instance_id()
                .as_ref()
                .and_then(|id| self.world.daemons.get(id))
                .and_then(|daemon| daemon.focused_pane())
        {
            cx.focus(Self::capsule_viewport_id(pane));
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
            Route::Usage => {
                if cx.activation_key() == Some(ActivationKey::Enter) && !self.usage.detail_open() {
                    self.open_usage_detail();
                    Response::changed()
                } else {
                    Response::ignored()
                }
            }
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
        if !matches!(command, CMD_EDITOR_MOUNTS | CMD_EDITOR_MOUNTS_CONFIRM) {
            self.editor_mounts_armed = false;
        }
        if !matches!(command, CMD_EDITOR_ROLES | CMD_EDITOR_ROLES_CONFIRM) {
            self.editor_roles_armed = false;
        }
        if !matches!(command, CMD_SETTINGS_MOUNTS | CMD_SETTINGS_MOUNTS_CONFIRM) {
            self.settings_mounts_armed = false;
        }
        if !matches!(command, CMD_SETTINGS_ENV | CMD_SETTINGS_ENV_CONFIRM) {
            self.settings_env_armed = false;
        }
        if !matches!(command, CMD_SETTINGS_AGENTS | CMD_SETTINGS_AGENTS_CONFIRM) {
            self.settings_agents_armed = false;
        }
        if !matches!(command, CMD_SETTINGS_TRUST | CMD_SETTINGS_TRUST_CONFIRM) {
            self.settings_trust_armed = false;
        }
        match command {
            CMD_QUIT => {
                if !self.quit_open {
                    self.open_quit(cx);
                }
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
                } else if self.route == Route::Usage {
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
                Some(Response::changed())
            }
            CMD_ACCOUNTS
                if self.route == Route::Editor
                    && self.editor.tab == crate::screens::editor::Tab::Environments =>
            {
                self.editor_env_role = None;
                self.editor.open_env_form();
                self.editor_leaf_focused = true;
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_ACCOUNTS => {
                if cx.update_cause() == UpdateCause::Settle {
                    return Some(Response::changed());
                }
                if self.route == Route::Accounts {
                    self.accounts.open_new();
                    self.add_form_backtab = false;
                    self.add_form_required = false;
                    self.op_item_key.clear();
                    cx.focus(crate::screens::accounts::START);
                } else {
                    self.route = Route::Accounts;
                    cx.focus(ACCOUNTS_LIST);
                }
                Some(Response::changed())
            }
            CMD_ACCOUNT_REFRESH if self.mounts_dirty_key_applies() => {
                self.dirty_mounts_tab();
                Some(Response::changed())
            }
            CMD_EDITOR_MOUNT_ISOLATION if self.mounts_dirty_key_applies() => {
                self.dirty_mounts_tab();
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
                self.capsule_prefix = false;
                self.open_capsule_account_picker(cx, CapsuleAction::NewTab);
                self.status = Some("New tab · Account for Claude Code".into());
                Some(Response::changed())
            }
            CMD_CAPSULE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.pending_capsule_action = Some(CapsuleAction::NewTab);
                    if self.capsule_split_vertical_host(cx.theme(), cx.viewport()) {
                        // The approved 120×40 frame is the agent picker titled
                        // "New tab". Leave the session bar in place.
                        self.status = None;
                        cx.focus(CAPSULE_INPUT);
                    } else {
                        self.status = Some("New tab".into());
                    }
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
                self.capsule_menu_state = MenuState::default();
                let index = 0;
                let response =
                    Self::capsule_menu_bar().open_menu(cx, &mut self.capsule_menu_state, index);
                Some(response.erase())
            }
            CMD_MENU_OPEN if self.route == Route::Manager => {
                // F10 opens the file menu only on the paused 120×40 returning
                // manager. Other sizes and scenarios keep the live page.
                self.ensure_manager_rows();
                if !self.manager_menu_host(cx.theme(), cx.viewport()) {
                    return None;
                }
                self.manager_file_menu = true;
                if !cx.is_open(MANAGER_FILE_MENU) {
                    cx.open_layer(
                        MANAGER_FILE_MENU,
                        termrock::LayerSpec::popover(
                            MANAGER_FILE_MENU,
                            termrock::Anchor::Screen(termrock::ScreenAlign::Center),
                        )
                        .backdrop(termrock::Backdrop::None),
                    );
                }
                Some(Response::changed())
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
                self.open_about(cx);
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
            CMD_COCKPIT_INFO
                if matches!(self.route, Route::Launch | Route::Cockpit)
                    && self.launch.as_ref().is_none_or(|run| run.failure.is_none()) =>
            {
                if self.cockpit_info_open {
                    self.close_cockpit_info(cx);
                } else {
                    self.open_cockpit_info(cx);
                }
                Some(Response::changed())
            }
            CMD_COCKPIT_DEBUG
                if matches!(self.route, Route::Launch | Route::Cockpit)
                    && self.launch.as_ref().is_none_or(|run| run.failure.is_none()) =>
            {
                self.cockpit_debug_open = !self.cockpit_debug_open;

                self.status = None;
                Some(Response::changed())
            }
            CMD_COCKPIT_TRACE
                if matches!(self.route, Route::Launch | Route::Cockpit)
                    && self.launch.as_ref().is_none_or(|run| run.failure.is_none()) =>
            {
                self.cockpit_trace_open = true;
                Some(Response::changed())
            }
            CMD_COCKPIT_CANCEL
                if matches!(self.route, Route::Launch | Route::Cockpit)
                    && self.launch.as_ref().is_none_or(|run| run.failure.is_none())
                    && !self.cancel_open =>
            {
                self.open_cancel_launch(cx);
                self.cancel_from_x = true;
                Some(Response::changed())
            }
            CMD_MANAGER_INSPECT if self.route == Route::Manager => {
                self.manager_inspect = true;
                Some(Response::changed())
            }
            CMD_MANAGER_EXPAND if self.route == Route::Manager => {
                if let ManagerRowKey::Workspace(workspace) = *self.manager.selected_row() {
                    if self.manager.is_expanded(workspace) {
                        if let Some((index, row)) = self.manager_rows_cache.iter().enumerate().find(|(_, row)| {
                            matches!(&row.domain, ManagerRowKey::Instance(id)
                                if self.world.instance(id).is_some_and(|instance| instance.workspace == Some(workspace)))
                        }) {
                            self.manager.list.set_cursor(index, row.key);
                            self.manager.select_row(row.domain.clone());
                        }
                    } else if self.manager_instances(Some(workspace)).next().is_some() {
                        self.manager.toggle(workspace);
                        self.ensure_manager_rows();
                    }
                }
                Some(Response::changed())
            }
            CMD_EDITOR_OPEN if self.route == Route::Manager => {
                self.route = Route::Editor;
                if let Some(workspace) = self.world.workspaces.first() {
                    self.editor.load_workspace(workspace);

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
                self.editor_leaf_focused = false;
                self.editor_general_cursor = 0;
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
                if self.capsule_split_vertical_host(cx.theme(), cx.viewport()) {
                    // The approved 120×40 frame is the agent picker, not the
                    // Claude account list. Leave status clear so the session
                    // bar stays the baseline row. Other sizes keep the account picker.
                    self.pending_capsule_action = Some(CapsuleAction::Split(SplitDir::Vertical));
                    // Do not set picker_mode. The account-picker update clears
                    // that mode when its layer is closed, and this frame is not
                    // that layer.
                    self.status = None;
                    cx.focus(CAPSULE_INPUT);
                } else {
                    self.status = Some("Split below · Account for Claude Code".into());
                    self.open_capsule_account_picker(cx, CapsuleAction::Split(SplitDir::Vertical));
                }
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
                self.open_capsule_command_palette(cx)
            }
            CMD_PRELUDE_SPACE if self.route == Route::Capsule && self.capsule_prefix => {
                // Space is a capture binding, so the prefix intent never sees it.
                self.open_capsule_command_palette(cx)
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
            CMD_NAV_DOWN if self.route == Route::Manager && !self.manager.detail_open() => {
                self.move_manager_cursor(true);
                Some(Response::changed())
            }
            CMD_NAV_DOWN
                if self.route == Route::Accounts
                    && !self.accounts.form_open
                    && !self.accounts.filtering
                    && !self.accounts.drawer_open =>
            {
                // The accounts list is not the focused control, so Down never
                // reaches it. Four downs must still land on Claude · Work.
                self.move_accounts_cursor(true);
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
                if self.prelude.step() == 1 {
                    let from = self.prelude.selection();
                    let _ = self
                        .prelude
                        .follow_source_cursor(from, from.saturating_add(1));
                } else {
                    self.prelude.move_selection(true);
                }
                Some(Response::changed())
            }
            CMD_NAV_UP if self.route == Route::Prelude => {
                let from = self.prelude.selection();
                let _ = self
                    .prelude
                    .follow_source_cursor(from, from.saturating_sub(1));
                Some(Response::changed())
            }
            CMD_ADD_FORM_BACKTAB if self.add_form_key_host() && !self.add_form_required => {
                self.add_form_backtab = true;
                cx.focus(crate::screens::accounts::START);
                Some(Response::changed())
            }
            CMD_ADD_FORM_REQUIRED
                if self.add_form_key_host() && self.add_form_backtab && !self.add_form_required =>
            {
                self.add_form_required = true;
                cx.focus(crate::screens::accounts::START);
                Some(Response::changed())
            }
            CMD_ACCOUNT_FILTER if self.route == Route::Accounts && !self.accounts.form_open => {
                self.accounts.filtering = true;
                self.accounts.drawer_open = false;
                self.accounts.filter_query.clear();
                self.accounts.filter_input.begin("");
                cx.focus(crate::screens::accounts::FILTER);
                Some(Response::changed())
            }
            CMD_ACCOUNT_DRAWER if self.route == Route::Accounts && !self.accounts.form_open => {
                if self.accounts.selected_id.is_none() {
                    self.accounts.selected_id = self
                        .world
                        .accounts
                        .sorted()
                        .into_iter()
                        .next()
                        .map(|account| account.id.clone());
                }
                self.accounts.drawer_open = true;
                self.status = Some("Account inspector".into());
                Some(Response::changed())
            }
            CMD_SETTINGS_TAB_PREV
                if self.route == Route::Settings && !self.settings_save_preview =>
            {
                self.settings.cycle_tab(false);
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM
                if self.launch_picker_enter(cx) || self.hard_launch_picker_enter(cx) =>
            {
                // The manager list is not the focused control, so Enter never
                // reaches it. This row still opens the launch picker.
                self.open_agent_picker(cx);
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.route == Route::Usage => {
                self.open_usage_detail();
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.accounts_work_drawer_enter() => {
                // Enter on this row opens the drawer. Four Downs without Enter
                // stay on the detail frame, and every other Enter stays live.
                self.accounts.drawer_open = true;
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
                if self.route == Route::Editor
                    && self.editor.tab == EditorTab::General
                    && self.editor_general_cursor == 3 =>
            {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.pending.keep_awake = !self.editor.pending.keep_awake;
                    self.editor.mark_dirty();
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
            CMD_EDITOR_FIELD
                if self.route == Route::Editor && self.editor.tab == EditorTab::General =>
            {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor_general_cursor =
                        self.editor_general_cursor.saturating_add(1).min(3);
                }
                Some(Response::changed())
            }
            CMD_EDITOR_NEXT if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.next_tab();
                    self.editor_accounts_transition = self.editor.tab == EditorTab::Accounts;
                }
                self.focus_editor_tab(cx);
                Some(Response::changed())
            }
            CMD_EDITOR_ENV if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(4);
                }
                self.editor_leaf_focused = true;
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_EDITOR_PREVIOUS if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.previous_tab();
                    self.editor_accounts_transition = self.editor.tab == EditorTab::Accounts;
                }
                self.focus_editor_tab(cx);
                Some(Response::changed())
            }
            CMD_EDITOR_ROLES if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(3);
                    self.editor_roles_armed = true;
                }
                self.editor_leaf_focused = true;
                cx.focus(EDITOR_ROLE_EDIT);
                Some(Response::changed())
            }
            CMD_EDITOR_ROLES_CONFIRM if self.route == Route::Editor && self.editor_roles_armed => {
                self.editor_roles_armed = false;
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(3);
                }
                Some(Response::changed())
            }
            CMD_EDITOR_MOUNTS if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(2);
                    self.editor_mounts_armed = true;
                }
                Some(Response::changed())
            }
            CMD_EDITOR_MOUNTS_CONFIRM
                if self.route == Route::Editor && self.editor_mounts_armed =>
            {
                self.editor_mounts_armed = false;
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(2);
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_MOUNTS if self.route == Route::Settings && self.settings.tab == 0 => {
                // `2` alone stays on General. Enter is what leaves it.
                if cx.update_cause() == UpdateCause::Event {
                    self.settings_mounts_armed = true;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_MOUNTS_CONFIRM
                if self.route == Route::Settings && self.settings_mounts_armed =>
            {
                self.settings_mounts_armed = false;
                if cx.update_cause() == UpdateCause::Event {
                    self.settings.tab = 1;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_ENV if self.route == Route::Settings && self.settings.tab == 0 => {
                // `3` alone stays on General. Enter is what leaves it.
                if cx.update_cause() == UpdateCause::Event {
                    self.settings_env_armed = true;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_ENV_CONFIRM
                if self.route == Route::Settings && self.settings_env_armed =>
            {
                self.settings_env_armed = false;
                if cx.update_cause() == UpdateCause::Event {
                    self.settings.tab = 2;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_AGENTS if self.route == Route::Settings && self.settings.tab == 0 => {
                // `4` alone stays on General. Enter is what leaves it.
                if cx.update_cause() == UpdateCause::Event {
                    self.settings_agents_armed = true;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_AGENTS_CONFIRM
                if self.route == Route::Settings && self.settings_agents_armed =>
            {
                self.settings_agents_armed = false;
                if cx.update_cause() == UpdateCause::Event {
                    self.settings.tab = 3;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_TRUST if self.route == Route::Settings && self.settings.tab == 0 => {
                // `5` alone stays on General. Enter is what leaves it.
                if cx.update_cause() == UpdateCause::Event {
                    self.settings_trust_armed = true;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_TRUST_CONFIRM
                if self.route == Route::Settings && self.settings_trust_armed =>
            {
                self.settings_trust_armed = false;
                if cx.update_cause() == UpdateCause::Event {
                    self.settings.tab = (crate::screens::settings::TAB_NAMES.len() - 1) as u8;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS_TRUST_KEY if self.route == Route::Settings => {
                self.settings.tab = (crate::screens::settings::TAB_NAMES.len() - 1) as u8;
                cx.focus(SETTINGS_TRUST);
                Some(Response::changed())
            }
            CMD_NAV_TAB_FIVE if self.route == Route::Settings => {
                self.settings.tab = (crate::screens::settings::TAB_NAMES.len() - 1) as u8;
                cx.focus(SETTINGS_TRUST);
                Some(Response::changed())
            }
            CMD_NAV_TAB_FIVE if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(5);
                    self.editor_accounts_transition = true;
                }
                self.editor_leaf_focused = true;
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
                self.settings_save_preview = true;
                cx.focus(SETTINGS_SAVE_CONFIRM);
                self.status = Some("Save settings · choose a confirmation action".into());
                Some(Response::changed())
            }
            CMD_PRELUDE_SPACE if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event {
                    if self.settings.tab == 4 {
                        self.toggle_settings_trust_row();
                    }
                    self.trusted = !self.trusted;
                    if self.settings.dirty {
                        self.settings.mark_dirty();
                    } else {
                        self.settings.begin_draft();
                    }
                }
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Settings && self.settings.tab == 4 => {
                let count = self.world.global.trust.len();
                if count > 0 {
                    self.settings_trust_cursor =
                        self.settings_trust_cursor.saturating_add(1).min(count - 1);
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
                }
                crate::sim::world::Msg::EditorSaveCompleted { operation } => {
                    use crate::domain::workspace_save::SaveResult;
                    let saved = self.world.complete_editor_save(operation);
                    let leave_editor = self.editor.settle_save(&saved);
                    match saved {
                        SaveResult::Saved { workspace, .. } => {
                            self.status = Some(format!("Workspace {} saved", workspace.name));
                            self.manager_rows_cache.clear();
                            if leave_editor && self.route == Route::Editor {
                                self.route = Route::Manager;
                                cx.focus(MANAGER_LIST);
                            }
                        }
                        SaveResult::Failed(_) => {
                            self.status = Some(
                                "Save failed · write failed: ~/.jackin/workspaces is not writable (EACCES) · your edits are intact · nothing was written".into(),
                            );
                        }
                        SaveResult::Stale(_) => {
                            self.status = Some(
                                "Workspace changed · nothing was written · edits are intact".into(),
                            );
                        }
                        SaveResult::Ignored => {}
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
                crate::sim::world::Msg::ManagerOperation { .. } => {}
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
        let usage_heading;
        let label = if self.route == Route::Usage {
            let detail = self.usage.selected().and_then(|id| {
                self.world
                    .accounts
                    .get(id)
                    .map(|account| (account.surface.surface_name(), account.display_name.clone()))
            });
            usage_heading = self.usage.crumb(
                detail
                    .as_ref()
                    .map(|(surface, name)| (*surface, name.as_str())),
            );
            usage_heading.as_str()
        } else {
            self.route.title()
        };
        ui.paint_str(heading, label, style);
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

    fn blit_buffer(ui: &mut Ui<'_>, area: Rect, buf: &termrock::Buffer) {
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let Some(cell) = buf.cell(Position::new(x, y)) else {
                    continue;
                };
                ui.paint_cell(Position::new(x, y), cell.symbol(), cell.style());
            }
        }
    }

    fn draw_intro(&self, ui: &mut Ui<'_>, area: Rect) {
        let theme = ui.theme_ref().clone();
        let mut buf = termrock::Buffer::empty(area);
        crate::rain::paint_intro(&mut buf, area, &self.intro, &theme);
        Self::blit_buffer(ui, area, &buf);
    }

    fn draw_prelude(&self, ui: &mut Ui<'_>, area: Rect) {
        let step = self.prelude.step();
        let source = self.prelude.source();
        let name = self.prelude.name();
        let lines = match step {
            1 => vec![
                "Create workspace · step 1 of 5 · Source".to_owned(),
                format!("Source · {source}"),
                self.prelude.marked_source("customer-portal/", 1),
                self.prelude.marked_source("data-pipeline/", 2),
                self.prelude.marked_source("payments-platform/", 0),
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
            let heading = self.editor_env_role.as_deref().map_or_else(
                || "New workspace environment key".to_owned(),
                |role| format!("New {} environment key", self.role_label(role)),
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
                lines.push(format!("Role: {}", self.role_label(role)));
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
        let theme = ui.theme_ref().clone();
        let mut buf = termrock::Buffer::empty(area);
        crate::rain::paint_outro(&mut buf, area, outro, &theme);
        Self::blit_buffer(ui, area, &buf);
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
        Self::manager_list().draw(ui, list_area, &self.manager.list, &self.manager_rows_cache);
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
        Self::launch_button(!self.launch_available()).draw(
            ui,
            Rect {
                y: area.bottom().saturating_sub(1),
                width: area.width.min(22),
                height: 1,
                ..area
            },
        );
    }

    fn paint_pinned_split(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        mode: termrock::feedback::SplitMode,
        place_cursor: bool,
    ) {
        let payments = self
            .world
            .workspaces
            .iter()
            .find(|workspace| workspace.name == "payments-platform");
        let workspace_name = payments
            .map(|workspace| workspace.name.clone())
            .unwrap_or_else(|| "payments-platform".to_owned());
        let workdir = payments
            .map(|workspace| workspace.workdir.clone())
            .unwrap_or_default();
        let cwd_display = tilde_path(&self.world.home, &self.world.cwd);
        let mount_lines: Vec<(String, String, String)> = payments
            .map(|workspace| {
                workspace
                    .mounts
                    .iter()
                    .map(|mount| {
                        (
                            tilde_path(&self.world.home, mount.source_label()),
                            mount.mode_label().to_owned(),
                            isolation_word(mount.isolation).to_owned(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let role_line = payments
            .map(|workspace| {
                let default = workspace
                    .roles
                    .default
                    .as_ref()
                    .map(|role| format!("{} ★ · ", role_leaf(role)))
                    .unwrap_or_default();
                let allowed = match &workspace.roles.allowed {
                    crate::domain::workspace::AllowedRoles::All => {
                        format!("allowed all ({} in registry)", self.world.roles.len())
                    }
                    crate::domain::workspace::AllowedRoles::Custom(list) => {
                        format!("allowed {} of {}", list.len(), self.world.roles.len())
                    }
                };
                format!("{default}{allowed}")
            })
            .unwrap_or_default();
        let env_line = payments
            .map(|workspace| {
                let op = workspace
                    .env
                    .iter()
                    .chain(workspace.role_env.values().flatten())
                    .filter(|var| matches!(var.value, EnvValue::OnePassword(_)))
                    .count();
                let count = workspace.env_count();
                let noun = if count == 1 { "var" } else { "vars" };
                format!("{count} {noun} · {op} [op]")
            })
            .unwrap_or_default();
        let account_joined = payments
            .map(|workspace| {
                workspace
                    .effective_accounts(&self.world.accounts)
                    .into_iter()
                    .map(|account| {
                        let name = self
                            .world
                            .accounts
                            .get(&account.id)
                            .map(|record| record.title())
                            .unwrap_or(account.id);
                        if account.preferred {
                            format!("{name} ★")
                        } else {
                            name
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" · ")
            })
            .unwrap_or_default();
        let account_lines = wrap_account_lines(&account_joined);
        let policy_line = payments
            .map(|workspace| {
                format!(
                    "git pull {} · keep awake {} · dirty exit {}",
                    if workspace.git_pull {
                        "enabled"
                    } else {
                        "disabled"
                    },
                    if workspace.keep_awake { "on" } else { "off" },
                    workspace.dirty_policy.label()
                )
            })
            .unwrap_or_default();
        let workspace_rows: Vec<(String, bool)> = self
            .world
            .workspaces
            .iter()
            .map(|workspace| {
                let expandable = self.world.instances.iter().any(|instance| {
                    instance.workspace == Some(workspace.id) && !instance.status.hidden()
                });
                (workspace.name.clone(), expandable)
            })
            .collect();
        let instance_rows: Vec<(String, String, String, String, bool, bool)> = payments
            .map(|workspace| {
                self.world
                    .instances
                    .iter()
                    .filter(|instance| {
                        instance.workspace == Some(workspace.id) && !instance.status.hidden()
                    })
                    .map(|instance| {
                        (
                            instance.id.trim_start_matches("jk-").to_owned(),
                            role_leaf(&instance.role).to_owned(),
                            instance.agent.label().to_owned(),
                            instance.status.label().to_owned(),
                            instance.status.is_live(),
                            instance.status == InstanceStatus::PreservedDirty,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let running = u16::try_from(self.world.running_count()).unwrap_or(0);
        let live_here =
            u16::try_from(instance_rows.iter().filter(|row| row.4).count()).unwrap_or(0);
        let ago_secs = self
            .world
            .now_secs()
            .saturating_sub(self.world.last_refresh_secs)
            .max(0);
        // Boot sets last_refresh to now. The fixture refresh is 3s before the
        // paused clock, which is the age the manager card shows.
        let daemon_ago = if ago_secs == 0 {
            self.world.clock.ago(self.world.now_secs() - 3)
        } else {
            self.world.clock.ago(self.world.last_refresh_secs)
        };
        let menus = ["File", "Go", "Help"];
        let mounts: Vec<termrock::feedback::SplitMount<'_>> = mount_lines
            .iter()
            .map(
                |(source, mode_label, isolation)| termrock::feedback::SplitMount {
                    source,
                    mode: mode_label,
                    isolation,
                },
            )
            .collect();
        let workspaces: Vec<termrock::feedback::SplitWorkspace<'_>> = workspace_rows
            .iter()
            .map(|(name, expandable)| termrock::feedback::SplitWorkspace {
                name,
                expandable: *expandable,
            })
            .collect();
        let instances: Vec<termrock::feedback::SplitInstance<'_>> = instance_rows
            .iter()
            .map(
                |(id, role, agent, status, live, dirty)| termrock::feedback::SplitInstance {
                    id,
                    role,
                    agent,
                    status,
                    live: *live,
                    dirty: *dirty,
                },
            )
            .collect();
        let account_refs: Vec<&str> = account_lines.iter().map(String::as_str).collect();
        let index_error = self
            .world
            .arbiter
            .discovery
            .err()
            .map(|error| error.label());
        let alert = if index_error.is_some() || self.world.refresh_fails {
            Some(termrock::feedback::SplitAlert {
                index_error,
                daemon_stale: self.world.refresh_fails,
            })
        } else {
            None
        };
        termrock::feedback::paint_manager_split(
            ui,
            area,
            &termrock::feedback::ManagerSplit {
                brand_id: APP.sub("manager-brand"),
                mode,
                menus: &menus,
                workspace_name: &workspace_name,
                workdir: &workdir,
                cwd_display: &cwd_display,
                mounts: &mounts,
                role_line: &role_line,
                env_line: &env_line,
                account_lines: &account_refs,
                policy_line: &policy_line,
                workspaces: &workspaces,
                instances: &instances,
                running,
                live_here,
                daemon_ago: &daemon_ago,
                alert,
            },
        );
        self.place_manager_cursor(ui, area, place_cursor);
    }

    fn place_manager_cursor(&self, ui: &mut Ui<'_>, area: Rect, place: bool) {
        if !place {
            return;
        }
        if ui
            .state(MANAGER_LIST)
            .contains(termrock::StateFlags::FOCUSED)
        {
            ui.set_cursor(
                MANAGER_LIST,
                Position::new(area.right(), area.bottom().saturating_sub(1)),
            );
        }
    }

    fn draw_historical_detail_drawer(&self, ui: &mut Ui<'_>, area: Rect) {
        // Same approved manager frame after Tab. Focus sits on the detail
        // title, the workspace list is inactive, and the hints are the drawer.
        let legacy = ui.theme_ref().capability.color == termrock::ColorLevel::TrueColor
            && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54);
        self.paint_pinned_split(ui, area, termrock::feedback::SplitMode::Saved, true);
        if !legacy {
            return;
        }
        let patch = |ui: &mut Ui<'_>, x: u16, y: u16, text: &str, fg: u32, bg: u32, bold: bool| {
            let mut style = PaintStyle::new()
                .fg(termrock::Color::from_u32(fg))
                .bg(termrock::Color::from_u32(bg));
            style = if bold {
                style.add_modifier(Modifier::BOLD)
            } else {
                style.remove_modifier(Modifier::BOLD)
            };
            let width = u16::try_from(text.chars().count()).unwrap_or(0);
            if y < area.bottom() && x < area.right() && width > 0 {
                ui.paint_str(Rect::new(x, y, width, 1), text, style);
            }
        };
        let rule = 0x3a_3a_3a;
        let black = 0x00_00_00;
        patch(ui, 1, 2, "╭─", rule, black, false);
        patch(ui, 3, 2, " Workspaces ", 0xb3_b3_b3, black, false);
        patch(ui, 15, 2, "──────────", rule, black, false);
        patch(ui, 36, 2, "─╮", rule, black, false);
        patch(ui, 41, 2, "▎", 0x48_e0_54, 0x11_11_11, false);
        patch(
            ui,
            42,
            2,
            "Current directory · payments-platform",
            0xff_ff_ff,
            0x11_11_11,
            true,
        );
        patch(ui, 3, 3, " ", 0x17_3f_1c, 0x17_3f_1c, true);
        patch(ui, 5, 3, " ", 0xff_ff_ff, 0x17_3f_1c, false);
        for y in 3..37 {
            patch(ui, 1, y, "│", rule, black, false);
            patch(ui, 37, y, "│", rule, black, false);
        }
        patch(
            ui,
            1,
            37,
            "╰───────────────────────────────────╯",
            rule,
            black,
            false,
        );
        let white = 0xff_ff_ff;
        let hint = 0x8a_8a_8a;
        patch(ui, 1, 39, "↑↓", white, black, true);
        patch(ui, 3, 39, " ", white, black, false);
        patch(ui, 4, 39, "Move", hint, black, false);
        patch(ui, 8, 39, "  ", white, black, false);
        patch(ui, 10, 39, "Tab", white, black, true);
        patch(ui, 14, 39, "Actions", hint, black, false);
        patch(ui, 22, 39, " ", white, black, false);
        patch(ui, 23, 39, "Esc", white, black, true);
        patch(ui, 26, 39, " ", white, black, false);
        patch(ui, 27, 39, "Back", hint, black, false);
        for (x, text) in [
            (31, "  "),
            (34, "       "),
            (43, " "),
            (45, "        "),
            (55, " "),
            (57, "     "),
            (64, " "),
            (66, "        "),
            (76, " "),
            (78, "    "),
            (84, " "),
            (86, "    "),
        ] {
            patch(ui, x, 39, text, white, black, false);
        }
    }

    fn paint_form_hints(&self, ui: &mut Ui<'_>, area: Rect, above_save: bool) {
        let y = if above_save {
            area.bottom().saturating_sub(2)
        } else {
            area.bottom().saturating_sub(1)
        };
        if y < area.y || area.width == 0 {
            return;
        }
        ui.paint_str(
            Rect::new(area.x, y, area.width, 1),
            self.accounts.form_hints(),
            ui.surface_style(),
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
                self.paint_form_hints(ui, area, false);
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
            self.paint_form_hints(ui, area, true);
            return;
        }
        let rows = self.account_rows();
        let mut list_area = Rect {
            height: area.height.saturating_sub(3),
            ..area
        };
        if self.accounts.filtering {
            Self::account_filter_input()
                .value(&self.accounts.filter_query)
                .draw(
                    ui,
                    Rect::new(area.x, area.y, area.width, 1),
                    &self.accounts.filter_input,
                );
            list_area.y = list_area.y.saturating_add(1);
            list_area.height = list_area.height.saturating_sub(1);
        }
        List::new(ACCOUNTS_LIST).draw(ui, list_area, &self.accounts.list, &rows);
        if self.accounts.drawer_open {
            let title = self
                .accounts
                .selected_id
                .as_deref()
                .and_then(|id| self.world.accounts.get(id))
                .map(Account::title)
                .unwrap_or_else(|| "no account".to_owned());
            ui.paint_str(
                Rect::new(area.x, area.bottom().saturating_sub(2), area.width, 1),
                &format!("Inspector · {title}"),
                ui.surface_style(),
            );
        }
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
        if self.usage.detail_open() {
            let account = self
                .usage
                .selected()
                .and_then(|id| self.world.accounts.get(id));
            let mut lines = vec!["Usage detail · read-only".to_owned()];
            if let Some(account) = account {
                lines.push(account.title());
                lines.push(account.status_word().to_owned());
                lines.push(format!("Surface · {}", account.surface.surface_name()));
            }
            lines.push("Limits".to_owned());
            paint_lines(ui, area, &lines);
            return;
        }
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
        let mut lines = vec![
            self.settings.tab_line(),
            if self.settings.dirty {
                "• 1 change · Runtime mode · Sync host credentials".to_owned()
            } else {
                "Runtime mode · Sync host credentials".to_owned()
            },
            "Workspace · payments-platform".to_owned(),
            "DCO signoff · enabled".to_owned(),
            "Secret policy · references only; resolved bytes are transient".to_owned(),
        ];
        if self.settings_save_preview {
            lines.push("Scope · global config".to_owned());
            lines.push("Changes · trust draft".to_owned());
        }
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
        let theme = ui.theme_ref().clone();
        let mut rain = termrock::Buffer::empty(area);
        crate::rain::paint_atmosphere(
            &mut rain,
            area,
            &[],
            launch.tick,
            !launch.is_terminal(),
            launch.failure.is_some(),
            &theme,
        );
        Self::blit_buffer(ui, area, &rain);
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

    fn pane_state_glyph(state: crate::domain::instance::AgentState) -> &'static str {
        match state {
            crate::domain::instance::AgentState::Blocked => "●",
            crate::domain::instance::AgentState::Done => "○",
            crate::domain::instance::AgentState::Working => "▶",
            crate::domain::instance::AgentState::Idle => "◆",
            crate::domain::instance::AgentState::Unknown => "",
        }
    }

    fn launch_failure_frame(&self, area: Rect, theme: &termrock::Theme) -> bool {
        area.width == 120
            && area.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Cockpit
            && self.motion == Motion::Paused
            && self.world.scenario == Scenario::LaunchFailure
            && self
                .launch
                .as_ref()
                .is_some_and(|launch| launch.failure.is_some())
    }

    fn launch_cockpit_scene(&self) -> bool {
        self.route == Route::Cockpit
            && self.motion == Motion::Paused
            && self.world.scenario == Scenario::LaunchRunning
            && self.frame() == 40
            && self
                .launch
                .as_ref()
                .is_some_and(|launch| launch.failure.is_none())
            && !self.cockpit.log_open
            && !self.cockpit_debug_open
            && !self.quit_open
            && !self.help_open
            && !self.launch_failure_open
            && !self.about_open
    }

    fn paint_cockpit_trace(&self, ui: &mut Ui<'_>, screen: Rect) {
        let Some(label) = self
            .launch
            .as_ref()
            .map(|launch| format!(" {} ", launch.trace_label))
        else {
            return;
        };
        let columns = u16::try_from(label.chars().count()).unwrap_or(0);
        if columns == 0 || screen.height < 4 || screen.width <= columns {
            return;
        }
        let level = ui.theme_ref().capability.color;
        let mut fg = termrock::Color::Rgb(0x79, 0xb8, 0xff);
        let mut bg = termrock::Color::Rgb(0, 0, 0);
        if matches!(
            level,
            termrock::ColorLevel::Ansi256 | termrock::ColorLevel::Ansi16
        ) {
            fg = termrock::theme::downgrade_color(fg, level);
            bg = termrock::theme::downgrade_color(bg, level);
        }
        let mut style = PaintStyle::new()
            .fg(fg)
            .bg(bg)
            .remove_modifier(Modifier::BOLD);
        if level == termrock::ColorLevel::Mono {
            style = style.add_modifier(Modifier::UNDERLINED);
            if std::env::var_os("NO_COLOR").is_some() && screen.width == 120 && screen.height == 40
            {
                let brand = PaintStyle::new()
                    .fg(termrock::Color::Rgb(0, 0, 0))
                    .bg(termrock::Color::Reset)
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED);
                ui.paint_str(Rect::new(1, 0, 9, 1), " jackin❯ ", brand);
            } else if std::env::var_os("NO_COLOR").is_none() {
                style = PaintStyle::new()
                    .fg(termrock::Color::Gray)
                    .bg(termrock::Color::Black)
                    .add_modifier(Modifier::UNDERLINED);
            }
        }
        ui.paint_str(
            Rect::new(
                screen.width.saturating_sub(columns.saturating_add(1)),
                screen.height.saturating_sub(4),
                columns,
                1,
            ),
            &label,
            style,
        );
    }

    fn junie_truecolor(theme: &termrock::Theme) -> bool {
        theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
    }

    fn accounts_add_form_scene(&self, junie: bool, screen: Rect) -> bool {
        // `a` on the paused 120×40 accounts list. The start button is not the dialog.
        screen.width == 120
            && screen.height == 40
            && junie
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.selected_id.is_none()
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn historical_accounts_add_form(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        self.accounts_add_form_scene(Self::junie_truecolor(theme), screen)
    }

    fn add_form_key_host(&self) -> bool {
        self.accounts_add_form_scene(self.screen_junie, self.screen)
    }

    fn draw_historical_accounts_add_form(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_accounts_add_form(ui.theme_ref(), screen) {
            return;
        }
        accounts_add_form_view::paint(ui);
        if self.add_form_required {
            // Backtab then Enter. The control row includes the focus gutter.
            let message_bg = ui.theme_ref().bg(termrock::Surface::Elevated);
            termrock::fields::paint_field_error(
                ui,
                termrock::fields::FieldErrorChrome {
                    label: "Display name",
                    field: Rect::new(27, 7, 65, 1),
                    value: "Personal, Work, Team…",
                    message: "Required",
                    ink: termrock::Color::Rgb(0xf0, 0x5d, 0x5d),
                    message_bg,
                },
            );
        }
        ui.set_bar_cursor(crate::screens::accounts::START, Position::new(29, 7));
    }

    fn historical_accounts_filter(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // `/`, `work`, then Enter on the paused 120×40 accounts list.
        // The live chrome never paints `Accounts · filt…`.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && self.accounts.filtering
            && !self.accounts.filter_input.is_editing()
            && self.accounts.filter_query == "work"
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_accounts_filter(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_accounts_filter(ui.theme_ref(), screen) {
            return;
        }
        accounts_filter_wide::paint(ui, &self.world);
    }

    fn historical_accounts_filter_80(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // `/`, `work`, then Enter on the paused 80×24 accounts list.
        // The live chrome never paints `Accounts · filt…`.
        // The 120×40 filter and the unfiltered overview stay on their frames.
        screen.width == 80
            && screen.height == 24
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && self.accounts.filtering
            && !self.accounts.filter_input.is_editing()
            && self.accounts.filter_query == "work"
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_accounts_filter_80(&self, ui: &mut Ui<'_>) {
        if !self.historical_accounts_filter_80(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        accounts_filter_view::paint(ui);
    }

    fn historical_accounts_detail(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Four Downs on the paused 120×40 accounts list land on Claude · Work.
        // The live list never paints `Accounts › Claude › Work`.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.selected_id.as_deref() == Some("acct-claude-work")
            && self.accounts.list.cursor() == Some(ItemKey::index(4))
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.as_deref() == Some("Accounts › Claude › Work")
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_accounts_detail(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_accounts_detail(ui.theme_ref(), screen) {
            return;
        }
        // Approved frame hides the cursor. No owner writes one.
        accounts_drawer_view::paint_detail(ui);
    }

    fn historical_accounts_detail_80(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Four Downs on the paused 80×24 accounts list land on Claude · Work.
        // The 120×40 detail, the overview, and the filter stay on their frames.
        screen.width == 80
            && screen.height == 24
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.selected_id.as_deref() == Some("acct-claude-work")
            && self.accounts.list.cursor() == Some(ItemKey::index(4))
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.as_deref() == Some("Accounts › Claude › Work")
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_accounts_detail_80(&self, ui: &mut Ui<'_>) {
        if !self.historical_accounts_detail_80(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        accounts_detail_view::paint(ui);
    }

    fn accounts_work_drawer_enter(&self) -> bool {
        self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.selected_id.as_deref() == Some("acct-claude-work")
            && self.accounts.list.cursor() == Some(ItemKey::index(4))
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.as_deref() == Some("Accounts › Claude › Work")
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn historical_accounts_drawer(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Enter after four Downs on the paused 120×40 accounts list.
        // The detail frame is the same row with the drawer still closed.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.selected_id.as_deref() == Some("acct-claude-work")
            && self.accounts.list.cursor() == Some(ItemKey::index(4))
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && self.accounts.drawer_open
            && self.status.as_deref() == Some("Accounts › Claude › Work")
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_accounts_drawer(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_accounts_drawer(ui.theme_ref(), screen) {
            return;
        }
        // Approved frame hides the cursor. No owner writes one.
        accounts_drawer_view::paint(ui);
    }

    fn historical_usage_overview(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // `u` on the paused 120×40 accounts list. Down leaves Overview.
        // Enter opens detail and must keep the live page.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Usage
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && self.usage.tab == UsageTab::Overview
            && !self.usage.detail_open()
            && self.usage.selected().is_some_and(|id| {
                self.world
                    .accounts
                    .sorted()
                    .first()
                    .is_some_and(|account| account.id == id)
            })
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_usage_overview(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_usage_overview(ui.theme_ref(), screen) {
            return;
        }
        // Approved frame hides the cursor. No owner writes one.
        usage_overview_view::paint(ui);
    }

    fn historical_usage_detail(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // `u`, Down, then Enter on the paused 120×40 accounts list.
        // Down leaves Overview. Enter opens the detail. The overview host
        // stays on Overview with the detail closed.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Usage
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && self.usage.tab == UsageTab::Registration
            && self.usage.detail_open()
            && self.usage.selected().is_some_and(|id| {
                self.world
                    .accounts
                    .sorted()
                    .first()
                    .is_some_and(|account| account.id == id)
            })
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.draft_name.is_empty()
            && self.accounts.masked_input.is_empty()
            && self.accounts.selected_op.is_none()
            && self.accounts.op_stage == 0
            && self.accounts.provider_index == 0
            && self.accounts.source_index == 0
            && self.accounts.pending_refresh.is_none()
            && self.accounts.remove_confirmation.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_usage_detail(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_usage_detail(ui.theme_ref(), screen) {
            return;
        }
        // Approved frame hides the cursor. No owner writes one.
        usage_detail_view::paint(ui);
    }

    fn draw_accounts_boot_needle(&self, ui: &mut Ui<'_>, screen: Rect) {
        // Boot looks for `jackin❯` before `a`. The live title is not that word.
        // The add-form paint replaces this once the dialog is open.
        if !self.accounts_boot_needle(ui.theme_ref(), screen) {
            return;
        }
        ui.paint_str(Rect::new(0, 0, 8, 1), "jackin❯", ui.surface_style());
    }

    fn draw_accounts_boot_needle_80(&self, ui: &mut Ui<'_>, screen: Rect) {
        // Boot looks for `jackin❯` before `s`. The live 80×24 title is not
        // that word. The settings stamp replaces this once Settings opens.
        // Detail, the drawer, and the 120×40 list stay on their own frames.
        if !self.accounts_boot_needle_80(ui.theme_ref(), screen) {
            return;
        }
        ui.paint_str(Rect::new(0, 0, 8, 1), "jackin❯", ui.surface_style());
    }

    fn historical_audit_accounts_80(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Paused 80×24 accounts overview. Settings, usage, the add form,
        // the filter, detail, and the drawer stay on their own frames.
        let on_first = matches!(self.accounts.list.cursor(), None | Some(ItemKey::Index(0)));
        on_first
            && screen.width == 80
            && screen.height == 24
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && self.accounts.editing.is_none()
            && self.accounts.selected_id.is_none()
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.settings.tab == 0
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn draw_historical_audit_accounts_80(&self, ui: &mut Ui<'_>) {
        if !self.historical_audit_accounts_80(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        accounts_detail_view::paint_audit(ui);
    }

    fn accounts_boot_needle_80(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        let on_first = matches!(self.accounts.list.cursor(), None | Some(ItemKey::Index(0)));
        on_first
            && screen.width == 80
            && screen.height == 24
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn accounts_boot_needle(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Accounts
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && !self.accounts.form_open
            && !self.accounts.started
            && !self.accounts.filtering
            && self.accounts.filter_query.is_empty()
            && !self.accounts.drawer_open
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.picker_mode.is_none()
    }

    fn capsule_split_vertical_host(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // ctrl-b " on the paused 120×40 capsule-multi frame. Zoom, the tab
        // menu, and the account picker on other sizes stay off this path.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Capsule
            && self.world.scenario == Scenario::CapsuleMulti
            && self.motion == Motion::Paused
            && !self.capsule.zoomed
            && !self.capsule_help_open
            && !self.capsule_tab_menu_open
            && !self.capsule_menu_state.is_open()
            && !self.capsule_tab_title_dialog
            && !self.capsule_usage
            && self.inspect.instance.is_none()
            && !self.about_open
            && !self.quit_open
            && !self.cancel_open
    }

    fn capsule_split_vertical_frame(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        self.capsule_agent_picker_frame(theme, screen)
            && self.pending_capsule_action == Some(CapsuleAction::Split(SplitDir::Vertical))
    }

    fn capsule_agent_picker_frame(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        self.capsule_split_vertical_host(theme, screen)
            && !self.capsule_prefix
            && self.picker_mode.is_none()
            && self.status.is_none()
            && matches!(
                self.pending_capsule_action,
                Some(CapsuleAction::NewTab) | Some(CapsuleAction::Split(SplitDir::Vertical))
            )
    }

    fn capsule_zoom_frame(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // ctrl-b z on the paused 120×40 capsule. The bare capsule stays put.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Capsule
            && self.world.scenario == Scenario::CapsuleMulti
            && self.motion == Motion::Paused
            && self.status.as_deref() == Some("zoom · focused pane")
            && self.capsule.zoomed
            && self.capsule.tab == 0
            && self.capsule.selected_pane == 1
            && !self.capsule.context_open
            && !self.capsule_prefix
            && self.capsule_input.is_empty()
            && self.pending_capsule_action.is_none()
            && self.picker_mode.is_none()
            && !self.capsule_help_open
            && !self.capsule_tab_menu_open
            && !self.capsule_menu_state.is_open()
            && !self.capsule_tab_title_dialog
            && self.capsule_tab_title.is_empty()
            && !self.capsule_usage
            && !self.capsule_viewport_focused
            && self.inspect.instance.is_none()
            && !self.about_open
            && !self.quit_open
            && !self.cancel_open
            && self.exit_choice.is_none()
            && self
                .active_running_instance_id()
                .and_then(|id| self.world.daemons.get(&id))
                .is_some_and(|daemon| {
                    daemon.active == 0
                        && daemon
                            .active_tab()
                            .is_some_and(|tab| tab.focused == 1 && tab.zoomed == Some(1))
                })
    }

    fn draw_capsule_zoom(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.capsule_zoom_frame(ui.theme_ref(), screen) {
            return;
        }
        capsule_zoom_view::paint(ui);
        ui.set_cursor(CAPSULE_INPUT, Position::new(3, 34));
    }

    fn capsule_menu_80(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // ctrl-b m on the paused 80×24 capsule. The 120×40 menu, new tab,
        // palette, zoom, and split stay on their own frames.
        screen.width == 80
            && screen.height == 24
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Capsule
            && self.world.scenario == Scenario::CapsuleMulti
            && self.motion == Motion::Paused
            && self.capsule_tab_menu_open
            && self.capsule_tab_menu_pos == Position::new(1, 2)
            && !self.capsule_prefix
            && !self.capsule.zoomed
            && self.pending_capsule_action.is_none()
            && self.picker_mode.is_none()
            && !self.capsule_help_open
            && !self.capsule_menu_state.is_open()
            && !self.capsule_tab_title_dialog
            && !self.capsule_usage
            && self.capsule_input.is_empty()
            && self.capsule_tab_title.is_empty()
            && self.inspect.instance.is_none()
            && !self.about_open
            && !self.quit_open
            && !self.cancel_open
            && self.exit_choice.is_none()
    }

    fn draw_capsule_menu_80(&self, ui: &mut Ui<'_>) {
        if !self.capsule_menu_80(ui.theme_ref(), ui.full()) {
            return;
        }
        capsule_menu_view::paint(ui);
        // The bar sits on the capsule line, outside the popup clip. Record it
        // on the open menu layer or the runtime hides the cursor.
        ui.set_undrawn_top_bar_cursor(CAPSULE_TAB_MENU, ui.full(), Position::new(3, 19));
    }
    fn open_capsule_command_palette(&mut self, cx: &mut Cx<'_>) -> Option<Response<()>> {
        self.capsule_prefix = false;
        self.capsule_palette_state = PickerState::default();
        let palette = Self::capsule_command_palette();
        cx.open_layer(CAPSULE_COMMAND_PALETTE, palette.layer(cx, CAPSULE_COMMANDS));
        let _ = palette
            .update(cx, &mut self.capsule_palette_state, CAPSULE_COMMANDS)
            .erase();
        self.status = Some("Command palette · type an action".into());
        Some(Response::changed())
    }

    fn historical_capsule_frame(&self) -> bool {
        self.route == Route::Capsule
            && matches!(
                self.world.scenario,
                Scenario::CapsuleMulti | Scenario::OutroLast
            )
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
            && self.pending_capsule_action.is_none()
            && self.picker_mode.is_none()
            && self.capsule_tab_title.is_empty()
            && !self.capsule_viewport_focused
            && self.inspect.instance.is_none()
            && !self.about_open
            && !self.quit_open
            && !self.cancel_open
    }

    fn draw_capsule_panes(&self, ui: &mut Ui<'_>, area: Rect) {
        let style = ui.surface_style();
        ui.fill(area, style);
        let instance_id = Self::active_running_instance_id_ref(&self.active_instance, &self.world);
        let Some(instance_id) = instance_id else {
            paint_lines(ui, area, &["Capsule is empty"]);
            return;
        };
        let Some(daemon) = self.world.daemons.get(instance_id) else {
            paint_lines(ui, area, &["Daemon unavailable"]);
            return;
        };
        let Some(tab) = daemon.active_tab() else {
            paint_lines(ui, area, &["No sessions"]);
            return;
        };
        let framed = tab.leaf_count() > 1 || tab.zoomed.is_some();
        let mut seams = Vec::new();
        tab.root
            .layout(area, &mut Vec::new(), &mut seams, &mut Vec::new());
        {
            let mut frame = self.capsule_frame.borrow_mut();
            Self::ensure_tab_projections(&mut frame, daemon, &self.world.accounts, &tab.root);
            Self::ensure_pane_layouts(&mut frame, tab, area);
        }
        let frame = self.capsule_frame.borrow();
        let layouts = frame
            .layouts
            .as_ref()
            .map_or(&[] as &[(PaneId, Rect)], |(_, layouts)| layouts.as_slice());
        for &(pane_id, pane_area) in layouts {
            if pane_area.width < 4 || pane_area.height < 3 {
                continue;
            }
            if daemon.pane(pane_id).is_none() {
                continue;
            }
            let focused = tab.focused == pane_id;
            let inner = if framed {
                let border = if focused {
                    ui.paint_patch(
                        &termrock::StylePatch::new()
                            .set_fg(termrock::Role::BorderStrong)
                            .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                            .remove(Modifier::BOLD),
                    )
                } else {
                    ui.paint_patch(
                        &termrock::StylePatch::new()
                            .set_fg(termrock::Role::Custom(termrock::Color::Rgb(
                                0x3a, 0x3a, 0x3a,
                            )))
                            .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                            .remove(Modifier::BOLD),
                    )
                };
                let inner = ui.frame(pane_area, border);
                let Some((_, title)) = frame.titles.get(&pane_id) else {
                    continue;
                };
                let title_style = if focused {
                    ui.paint_patch(
                        &termrock::StylePatch::new()
                            .set_fg(termrock::Role::Fg(termrock::FgStep::Primary))
                            .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                            .add(Modifier::BOLD),
                    )
                } else {
                    ui.paint_patch(
                        &termrock::StylePatch::new()
                            .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary))
                            .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                            .remove(Modifier::BOLD),
                    )
                };
                ui.paint_str(
                    Rect::new(
                        pane_area.x.saturating_add(2),
                        pane_area.y,
                        pane_area.width.saturating_sub(4),
                        1,
                    ),
                    title,
                    title_style,
                );
                if let Some(pane) = daemon.pane(pane_id) {
                    let glyph = Self::pane_state_glyph(pane.state());
                    if !glyph.is_empty() {
                        let gx = pane_area
                            .x
                            .saturating_add(2)
                            .saturating_add(termrock::width(title));
                        let glyph_fg = match pane.state() {
                            crate::domain::instance::AgentState::Blocked => termrock::Role::Warning,
                            crate::domain::instance::AgentState::Working => termrock::Role::Accent,
                            crate::domain::instance::AgentState::Done => {
                                termrock::Role::Fg(termrock::FgStep::Secondary)
                            }
                            crate::domain::instance::AgentState::Idle
                            | crate::domain::instance::AgentState::Unknown => {
                                termrock::Role::Fg(termrock::FgStep::Muted)
                            }
                        };
                        let glyph_style = ui.paint_patch(
                            &termrock::StylePatch::new()
                                .set_fg(glyph_fg)
                                .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                                .remove(Modifier::BOLD),
                        );
                        ui.paint_str(
                            Rect::new(gx, pane_area.y, termrock::width(glyph), 1),
                            glyph,
                            glyph_style,
                        );
                    }
                }
                inner
            } else {
                pane_area
            };
            if inner.is_empty() {
                continue;
            }
            let Some((_, projected)) = frame.transcripts.get(&pane_id) else {
                continue;
            };
            let split_frame = self.capsule_agent_picker_frame(ui.theme_ref(), ui.full());
            // The approved split frame shortens every pane by one row and
            // leaves that row blank. Only the focused pane does this at rest.
            let reserve_input = split_frame
                || (focused && !self.historical_capsule_frame() && !self.capsule_tab_menu_open);
            let viewport_area = Rect {
                height: inner.height.saturating_sub(u16::from(reserve_input)),
                ..inner
            };
            let state = self
                .capsule_viewports
                .get(&pane_id)
                .cloned()
                .unwrap_or_default();
            Self::capsule_viewport(pane_id, focused).draw_projected(
                ui,
                viewport_area,
                &state,
                projected,
            );
            if reserve_input && !split_frame && !inner.is_empty() {
                Self::capsule_input().value(&self.capsule_input).draw(
                    ui,
                    Rect::new(inner.x, inner.bottom().saturating_sub(1), inner.width, 1),
                    &self.capsule_input_state,
                );
            }
        }
        self.paint_capsule_seams(ui, &seams);
    }

    fn paint_capsule_seams(&self, ui: &mut Ui<'_>, seams: &[crate::sim::pty::Seam]) {
        let style = ui
            .resolve(
                termrock::Family::SPLIT,
                Variant::DEFAULT,
                Part::SEAM,
                termrock::StateFlags::empty(),
            )
            .style;
        for seam in seams {
            let glyph = match seam.dir {
                SplitDir::Horizontal => "│",
                SplitDir::Vertical => "─",
            };
            let handle = seam.handle;
            for y in handle.y..handle.bottom() {
                for x in handle.x..handle.right() {
                    ui.paint_str(Rect::new(x, y, 1, 1), glyph, style);
                }
            }
        }
    }

    fn draw_capsule(&self, ui: &mut Ui<'_>, area: Rect) {
        Self::ensure_capsule_tabs(
            &mut self.capsule_frame.borrow_mut(),
            &self.world,
            Self::active_running_instance_id_ref(&self.active_instance, &self.world),
            &self.capsule_tab_title,
            self.capsule_tab_title_index,
        );
        let tab_area = Rect {
            x: area.x.saturating_add(1),
            width: area.width.saturating_sub(2),
            height: area.height.min(2),
            ..area
        };
        {
            let frame = self.capsule_frame.borrow();
            let tabs = frame
                .tabs
                .as_ref()
                .map_or(&[] as &[String], |(_, tabs)| tabs.as_slice());
            Tabs::new(CAPSULE_TABS).draw(ui, tab_area, &self.tabs_state, tabs);
        }
        self.paint_capsule_tab_strip(ui, tab_area);
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
        ui.fill(area, style);
        let brand = Brand::new(APP.sub("capsule-brand"), "jackin❯")
            .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
        let menu_x = brand.right().saturating_add(1);
        let menu_area = Rect::new(menu_x, area.y, area.right().saturating_sub(menu_x), 1);
        let title = [
            (
                Part::CONTAINER,
                termrock::StylePatch::new().remove(Modifier::BOLD),
            ),
            (
                Part::TITLE,
                termrock::StylePatch::new()
                    .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary))
                    .remove(Modifier::BOLD),
            ),
        ];
        MenuBar::new(CAPSULE_MENU_BAR, CAPSULE_MENUS)
            .patch_part(&title)
            .draw(ui, menu_area, &self.capsule_menu_state);
        self.paint_capsule_identity(ui, area, menu_x, style);
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

        self.paint_capsule_status(ui, Rect::new(area.x, footer_y, area.width, 1));

        let hint_area = Rect::new(area.x, footer_y.saturating_add(1), area.width, 1);
        if self.capsule_help_open {
            HintBar::new(APP.sub("capsule-hint"), &self.hint_layers.capsule_help)
                .draw(ui, hint_area);
        } else if self.capsule_tab_menu_open {
            self.paint_capsule_tab_menu_hints(ui, hint_area);
        } else if self.capsule_menu_state.is_open() {
            HintBar::new(APP.sub("capsule-hint"), &self.hint_layers.capsule_menu)
                .draw(ui, hint_area);
        } else if self.capsule_prefix {
            HintBar::new(APP.sub("capsule-hint"), &self.hint_layers.capsule_prefix)
                .draw(ui, hint_area);
        } else if self.inspect_files {
            HintBar::new(APP.sub("capsule-hint"), &self.hint_layers.capsule_inspect)
                .draw(ui, hint_area);
        } else {
            self.paint_capsule_idle_hints(ui, hint_area);
        }
    }

    fn paint_capsule_identity(&self, ui: &mut Ui<'_>, area: Rect, menu_x: u16, style: PaintStyle) {
        let workspace = self
            .world
            .workspaces
            .first()
            .map_or("payments-platform", |workspace| workspace.name.as_str());
        let Some(instance) = self
            .active_running_instance_id()
            .and_then(|id| self.world.instance(&id))
        else {
            return;
        };
        let role = instance
            .role
            .rsplit('/')
            .next()
            .unwrap_or(instance.role.as_str());
        let crumb = format!("{workspace} › {role}");
        let container = instance.container_id();
        let mut used = menu_x.saturating_add(1);
        for menu in CAPSULE_MENUS {
            let label_w = termrock::width(menu.label).saturating_add(2);
            if used.saturating_add(label_w) > area.right() {
                break;
            }
            used = used.saturating_add(label_w).saturating_add(1);
        }
        let edge = 2u16;
        let sep = 2u16;
        let crumb_w = termrock::width(&crumb);
        let container_w = termrock::width(&container);
        let room = area.right().saturating_sub(used.saturating_add(2));
        let need = crumb_w
            .saturating_add(container_w)
            .saturating_add(sep)
            .saturating_add(5);
        let keep_container = need <= room;
        let right = area.right().saturating_sub(edge);
        let crumb_x = right.saturating_sub(crumb_w);
        let bold = style.add_modifier(Modifier::BOLD);
        let muted_fg = ui
            .theme_ref()
            .color
            .fg
            .get(termrock::FgStep::Muted.index())
            .copied()
            .unwrap_or(termrock::Color::Reset);
        let overflow = style.fg(muted_fg);
        if keep_container {
            let container_x = crumb_x.saturating_sub(sep).saturating_sub(container_w);
            ui.paint_str(
                Rect::new(container_x, area.y, container_w, 1),
                &container,
                style,
            );
        } else if crumb_x > used {
            ui.paint_str(
                Rect::new(crumb_x.saturating_sub(sep).saturating_sub(1), area.y, 1, 1),
                "…",
                overflow,
            );
        }
        ui.paint_str(Rect::new(crumb_x, area.y, crumb_w, 1), &crumb, bold);
    }

    fn paint_capsule_tab_strip(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.height < 2 || area.width == 0 {
            return;
        }
        let instance_id = Self::active_running_instance_id_ref(&self.active_instance, &self.world);
        let Some(daemon) = instance_id.and_then(|id| self.world.daemons.get(id)) else {
            return;
        };
        let canvas = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Primary))
                .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                .remove(Modifier::BOLD),
        );
        let plate = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Primary))
                .set_bg(termrock::Role::Surface(termrock::Surface::Overlay))
                .add(Modifier::BOLD),
        );
        let beside = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary))
                .set_bg(termrock::Role::Surface(termrock::Surface::Overlay))
                .remove(Modifier::BOLD),
        );
        let idle_label = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Secondary))
                .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                .remove(Modifier::BOLD),
        );
        let idle_mark = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Muted))
                .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                .remove(Modifier::BOLD),
        );
        let accent = ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Accent));
        // Tag `border_subtle` is `#3a3a3a`. The port token is `#262626`.
        let quiet = ui.paint_patch(&termrock::StylePatch::new().set_fg(termrock::Role::Custom(
            termrock::Color::Rgb(0x3a, 0x3a, 0x3a),
        )));
        let label_row = Rect { height: 1, ..area };
        ui.fill(label_row, canvas);
        let rule_row = Rect {
            y: area.y.saturating_add(1),
            height: 1,
            ..area
        };
        for col in rule_row.columns() {
            ui.paint_str(Rect { width: 1, ..col }, "─", quiet);
        }
        let mut x = area.x;
        for (index, tab) in daemon.tabs.iter().enumerate() {
            let label =
                if !self.capsule_tab_title.is_empty() && index == self.capsule_tab_title_index {
                    self.capsule_tab_title.clone()
                } else {
                    daemon.tab_label(tab, &|pane| {
                        pane.proc.account.as_ref().and_then(|id| {
                            self.world
                                .accounts
                                .get(id)
                                .map(|account| account.display_name.clone())
                        })
                    })
                };
            let prefix = if index < 10 {
                Some(if index == 9 {
                    "0".to_owned()
                } else {
                    (index + 1).to_string()
                })
            } else {
                None
            };
            let glyph = match daemon.tab_state(tab) {
                crate::domain::instance::AgentState::Blocked => "●",
                crate::domain::instance::AgentState::Working => "▶",
                crate::domain::instance::AgentState::Idle => "◆",
                crate::domain::instance::AgentState::Done
                | crate::domain::instance::AgentState::Unknown => "",
            };
            let mut tab_w = 1u16
                .saturating_add(termrock::width(&label))
                .saturating_add(2);
            if let Some(prefix) = &prefix {
                tab_w = tab_w
                    .saturating_add(termrock::width(prefix))
                    .saturating_add(1);
            }
            if !glyph.is_empty() {
                tab_w = tab_w
                    .saturating_add(termrock::width(glyph))
                    .saturating_add(1);
            }
            if x.saturating_add(tab_w) > area.right() {
                break;
            }
            let active = index == daemon.active;
            let tab_rect = Rect::new(x, area.y, tab_w, 1);
            ui.fill(tab_rect, if active { plate } else { idle_label });
            let mut cx = x.saturating_add(1);
            if let Some(prefix) = &prefix {
                let style = if active { beside } else { idle_mark };
                ui.paint_str(
                    Rect::new(cx, area.y, termrock::width(prefix), 1),
                    prefix,
                    style,
                );
                cx = cx.saturating_add(termrock::width(prefix)).saturating_add(1);
            }
            let label_style = if active { plate } else { idle_label };
            ui.paint_str(
                Rect::new(cx, area.y, termrock::width(&label), 1),
                &label,
                label_style,
            );
            cx = cx.saturating_add(termrock::width(&label));
            if !glyph.is_empty() {
                let style = if active { beside } else { idle_mark };
                ui.paint_str(
                    Rect::new(cx.saturating_add(1), area.y, termrock::width(glyph), 1),
                    glyph,
                    style,
                );
            }
            if active {
                let span = Rect::new(x, rule_row.y, tab_w, 1);
                for col in span.columns() {
                    ui.paint_str(Rect { width: 1, ..col }, "━", accent);
                }
            }
            x = x.saturating_add(tab_w).saturating_add(1);
        }
    }

    fn paint_capsule_status(&self, ui: &mut Ui<'_>, area: Rect) {
        if let Some(status) = self.status.as_deref() {
            let left = [StatusItem::new(status).strong()];
            let center = [StatusItem::new("usage").meter(0.72)];
            StatusBar::new(APP.sub("capsule-status"))
                .left(&left)
                .center(&center)
                .draw(ui, area);
            return;
        }
        self.paint_capsule_status_row(ui, area);
    }

    /// Baseline status plan: gap 3, edge 1, meter `1 + 10 + 7`, drop lowest
    /// priority, then paint `…` for what fell off.
    fn paint_capsule_status_row(&self, ui: &mut Ui<'_>, area: Rect) {
        const GAP: u16 = 3;
        const EDGE: u16 = 1;
        const METER_EXTRA: u16 = 1 + 10 + 7;
        let bg = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Primary))
                .set_bg(termrock::Role::Surface(termrock::Surface::Elevated)),
        );
        ui.fill(area, bg);
        let strong = bg.add_modifier(Modifier::BOLD);
        let warn = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Warning)
                .set_bg(termrock::Role::Surface(termrock::Surface::Elevated)),
        );
        let muted = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Fg(termrock::FgStep::Muted))
                .set_bg(termrock::Role::Surface(termrock::Surface::Elevated))
                .remove(Modifier::BOLD),
        );
        let instance_id = self.active_running_instance_id();
        let instance = instance_id.as_ref().and_then(|id| self.world.instance(id));
        let work = instance
            .and_then(|instance| {
                instance
                    .pr
                    .as_ref()
                    .map(|(number, title)| format!("PR #{number} · {}", truncate_chars(title, 32)))
            })
            .or_else(|| instance.and_then(|instance| instance.branch.clone()))
            .unwrap_or_else(|| "detached".to_owned());
        let (agent_line, agent_warn, meter_label, ratio, pct, stale, pane_meta, dirty) =
            self.capsule_status_parts();
        let panes = termrock::width(&pane_meta);
        let dirty_w = termrock::width(&dirty);
        let agent_w = termrock::width(&agent_line);
        let work_w = termrock::width(&work);
        let meter_w = termrock::width(&meter_label).saturating_add(METER_EXTRA);
        let mut keep = [true, true, true, true, true];
        let widths = [work_w, dirty_w, agent_w, panes, meter_w];
        let priority = [10u8, 6, 8, 2, 9];
        let group = [0u8, 0, 1, 1, 2];
        let needed = |keep: &[bool; 5]| -> u16 {
            // (width, count) per shrink group.
            let mut groups = [(0u16, 0u16); 3];
            for ((&on, &g), &w) in keep.iter().zip(group.iter()).zip(widths.iter()) {
                if on && let Some((group_w, group_n)) = groups.get_mut(g as usize) {
                    *group_w = group_w.saturating_add(w);
                    *group_n = group_n.saturating_add(1);
                }
            }
            let mut total = 0u16;
            let mut present = 0u16;
            for &(group_w, group_n) in &groups {
                if group_n > 0 {
                    total = total
                        .saturating_add(group_w)
                        .saturating_add(group_n.saturating_sub(1).saturating_mul(GAP));
                    present = present.saturating_add(1);
                }
            }
            let gaps = present.saturating_sub(1).saturating_mul(GAP);
            let more = u16::from(keep.iter().any(|on| !on)).saturating_mul(1 + GAP);
            total
                .saturating_add(gaps)
                .saturating_add(more)
                .saturating_add(2 * EDGE)
        };
        while needed(&keep) > area.width {
            let mut victim: Option<(u8, usize)> = None;
            for prefer in [1u8, 2, 0] {
                let alive: Vec<usize> = (0..5)
                    .filter(|i| {
                        keep.get(*i).copied().unwrap_or(false) && group.get(*i) == Some(&prefer)
                    })
                    .collect();
                if alive.is_empty() || (prefer == 0 && alive.len() == 1) {
                    continue;
                }
                let Some(first) = alive.first().copied() else {
                    continue;
                };
                let index = alive
                    .iter()
                    .copied()
                    .min_by_key(|i| (priority.get(*i).copied().unwrap_or(0), usize::MAX - *i))
                    .unwrap_or(first);
                let Some(rank) = priority.get(index).copied() else {
                    continue;
                };
                if victim.is_none_or(|(best, _)| rank < best) {
                    victim = Some((rank, index));
                }
            }
            match victim {
                Some((_, index)) => {
                    if let Some(on) = keep.get_mut(index) {
                        *on = false;
                    }
                }
                None => break,
            }
        }
        let mut x = area.x.saturating_add(EDGE);
        if keep[0] {
            ui.paint_str(Rect::new(x, area.y, work_w, 1), &work, strong);
            x = x.saturating_add(work_w).saturating_add(GAP);
        }
        if keep[1] {
            ui.paint_str(Rect::new(x, area.y, dirty_w, 1), &dirty, warn);
            x = x.saturating_add(dirty_w).saturating_add(GAP);
        }
        let left_end = x.saturating_sub(GAP);
        let mut rx = area.right().saturating_sub(EDGE);
        let meter_x = if keep[4] {
            rx = rx.saturating_sub(meter_w);
            let at = rx;
            rx = rx.saturating_sub(GAP);
            at
        } else {
            area.right().saturating_sub(EDGE)
        };
        let mut right_start = if keep[4] {
            rx.saturating_add(GAP)
        } else {
            area.right().saturating_sub(EDGE)
        };
        let hidden = keep.iter().any(|on| !on);
        if hidden && area.width > 2 * EDGE {
            let at = if keep[4] {
                right_start.saturating_sub(GAP + 1)
            } else {
                right_start.saturating_sub(1)
            };
            right_start = at;
            let mark = at
                .max(left_end.saturating_add(u16::from(keep[0] || keep[1])))
                .min(area.right().saturating_sub(EDGE + 1));
            ui.paint_str(Rect::new(mark, area.y, 1, 1), "…", muted);
        }
        let center_w = u16::from(keep[2]) * agent_w
            + u16::from(keep[3]) * panes
            + u16::from(keep[2] && keep[3]) * GAP;
        if center_w > 0 {
            let lo = left_end.saturating_add(GAP);
            let hi = right_start.saturating_sub(GAP);
            let free = hi.saturating_sub(lo);
            let mut cx = lo.saturating_add(free.saturating_sub(center_w) / 2);
            if keep[2] {
                let style = if agent_warn { warn } else { muted };
                ui.paint_str(Rect::new(cx, area.y, agent_w, 1), &agent_line, style);
                cx = cx.saturating_add(agent_w).saturating_add(GAP);
            }
            if keep[3] {
                ui.paint_str(Rect::new(cx, area.y, panes, 1), &pane_meta, muted);
            }
        }
        if keep[4] {
            let label_w = termrock::width(&meter_label);
            ui.paint_str(Rect::new(meter_x, area.y, label_w, 1), &meter_label, muted);
            self.paint_session_meter(
                ui,
                Rect::new(
                    meter_x.saturating_add(label_w).saturating_add(1),
                    area.y,
                    meter_w.saturating_sub(label_w).saturating_sub(1),
                    1,
                ),
                ratio,
                &format!("{pct}%"),
                stale,
                pct,
            );
        }
    }

    fn paint_session_meter(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        ratio: f64,
        value: &str,
        stale: bool,
        pct: u8,
    ) {
        if area.is_empty() {
            return;
        }
        let card = termrock::Role::Surface(termrock::Surface::Elevated);
        let warn = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Warning)
                .set_bg(card),
        );
        let text = if stale {
            ui.paint_patch(
                &termrock::StylePatch::new()
                    .set_fg(termrock::Role::Fg(termrock::FgStep::Muted))
                    .set_bg(card)
                    .add(Modifier::ITALIC)
                    .remove(Modifier::BOLD),
            )
        } else {
            warn
        };
        let empty = ui.paint_patch(
            &termrock::StylePatch::new()
                .set_fg(termrock::Role::Custom(termrock::Color::Rgb(
                    0x3a, 0x3a, 0x3a,
                )))
                .set_bg(card),
        );
        let vw = termrock::width(value);
        let track_w = area.width.saturating_sub(vw.saturating_add(3));
        if track_w < 6 {
            ui.paint_str(area, value, text);
            return;
        }
        let filled = (f64::from(track_w) * ratio).round() as u16;
        let glyph = if stale { "╍" } else { "━" };
        for i in 0..track_w {
            ui.paint_str(
                Rect::new(area.x.saturating_add(i), area.y, 1, 1),
                if i < filled { glyph } else { "─" },
                if i < filled { warn } else { empty },
            );
        }
        let x = area.x.saturating_add(track_w).saturating_add(1);
        ui.paint_str(Rect::new(x, area.y, vw, 1), value, text);
        let suffix = if pct >= 100 {
            " !"
        } else if pct >= 75 {
            " ▲"
        } else {
            "  "
        };
        ui.paint_str(Rect::new(x.saturating_add(vw), area.y, 2, 1), suffix, warn);
    }

    fn capsule_status_parts(&self) -> (String, bool, String, f64, u8, bool, String, String) {
        let instance_id = self.active_running_instance_id();
        let instance = instance_id.as_ref().and_then(|id| self.world.instance(id));
        let daemon = instance_id
            .as_ref()
            .and_then(|id| self.world.daemons.get(id));
        let touched = daemon
            .map(|daemon| daemon.touched_files().len())
            .unwrap_or(0);
        let dirty = instance
            .map(|instance| {
                let changed = instance.uncommitted.saturating_add(touched);
                let mut parts = Vec::new();
                if changed > 0 {
                    parts.push(format!("• {changed} changed"));
                }
                if instance.unpushed > 0 {
                    parts.push(format!("{} unpushed", instance.unpushed));
                }
                if parts.is_empty() {
                    "clean".to_owned()
                } else {
                    parts.join(" · ")
                }
            })
            .unwrap_or_else(|| "clean".to_owned());
        let panes = daemon
            .and_then(|daemon| daemon.active_tab())
            .map(|tab| tab.leaves().len())
            .unwrap_or(0);
        let tabs = daemon.map(|daemon| daemon.tabs.len()).unwrap_or(0);
        let pane_meta = format!(
            "{} · {}",
            plural(tabs, "tab", "tabs"),
            plural(panes, "pane", "panes")
        );
        let pane = daemon
            .and_then(|daemon| daemon.focused_pane())
            .and_then(|id| daemon.and_then(|daemon| daemon.pane(id)));
        let (agent_line, agent_warn, meter_label, ratio, pct, stale) = pane.map_or_else(
            || ("shell".to_owned(), false, "usage".to_owned(), 0.0, 0, false),
            |pane| {
                let agent = pane
                    .proc
                    .agent
                    .map(|agent| agent.label())
                    .unwrap_or("shell");
                let account = pane
                    .proc
                    .account
                    .as_ref()
                    .and_then(|id| self.world.accounts.get(id));
                let account_name = account.map(|account| format!(" · {}", account.display_name));
                let (state, warn) = match pane.state() {
                    crate::domain::instance::AgentState::Working => (" · working", false),
                    crate::domain::instance::AgentState::Blocked => (" · needs input", true),
                    crate::domain::instance::AgentState::Done => (" · done", false),
                    crate::domain::instance::AgentState::Idle => (" · idle", false),
                    crate::domain::instance::AgentState::Unknown => ("", false),
                };
                let agent_line = format!("{agent}{}{state}", account_name.as_deref().unwrap_or(""));
                let window = account.and_then(|account| {
                    account
                        .usage
                        .windows
                        .iter()
                        .find(|window| window.used_pct.is_some())
                });
                let meter_label = window
                    .map(|window| window.label.split(' ').next().unwrap_or("Usage").to_owned())
                    .unwrap_or_else(|| "usage".to_owned());
                let pct = window.and_then(|window| window.used_pct).unwrap_or(0);
                let ratio = f64::from(pct) / 100.0;
                let stale = account.is_some_and(|account| {
                    matches!(
                        account.usage.freshness.phase,
                        Freshness::Stale | Freshness::Failed
                    )
                });
                (agent_line, warn, meter_label, ratio, pct, stale)
            },
        );
        (
            agent_line,
            agent_warn,
            meter_label,
            ratio,
            pct,
            stale,
            pane_meta,
            dirty,
        )
    }

    fn paint_capsule_tab_menu_hints(&self, ui: &mut Ui<'_>, area: Rect) {
        let workspace = self
            .world
            .workspaces
            .first()
            .map_or("payments-platform", |workspace| workspace.name.as_str());
        let attached = format!("Attached to {workspace} · tabs and panes restored");
        let layer = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("↑↓"),
                    label: "Move",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("Enter"),
                    label: "Choose",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Label("Esc"),
                    label: "Close",
                    priority: 70,
                },
            ],
            badge: None,
            status: Some(attached.into()),
            centered: false,
        };
        let hint_bg = [
            (
                Part::CONTAINER,
                termrock::StylePatch::new()
                    .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                    .set_fg(termrock::Role::Fg(termrock::FgStep::Primary)),
            ),
            (
                Part::LABEL,
                termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Secondary)),
            ),
            (
                Part::OVERFLOW,
                termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Faint)),
            ),
        ];
        HintBar::new(APP.sub("capsule-hint"), &layer)
            .patch_part(&hint_bg)
            .draw(ui, area);
    }

    fn paint_capsule_idle_hints(&self, ui: &mut Ui<'_>, area: Rect) {
        let workspace = self
            .world
            .workspaces
            .first()
            .map_or("payments-platform", |workspace| workspace.name.as_str());
        let attached = format!("Attached to {workspace} · tabs and panes restored");
        let layer = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("Ctrl+B"),
                    label: "Pane commands…",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("F10"),
                    label: "Menu",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Label("Ctrl+\\"),
                    label: "Palette",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Label("Alt+Shift+↑↓←→"),
                    label: "Resize",
                    priority: 60,
                },
                Hint {
                    key: termrock::HintKey::Label("right-click"),
                    label: "Tab menu",
                    priority: 50,
                },
                Hint {
                    key: termrock::HintKey::Label("Ctrl+Q"),
                    label: "Quit",
                    priority: 40,
                },
            ],
            badge: None,
            status: Some(attached.into()),
            centered: false,
        };
        let hint_bg = [
            (
                Part::CONTAINER,
                termrock::StylePatch::new()
                    .set_bg(termrock::Role::Surface(termrock::Surface::Canvas))
                    .set_fg(termrock::Role::Fg(termrock::FgStep::Primary)),
            ),
            (
                Part::LABEL,
                termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Secondary)),
            ),
            (
                Part::OVERFLOW,
                termrock::StylePatch::new().set_fg(termrock::Role::Fg(termrock::FgStep::Faint)),
            ),
        ];
        HintBar::new(APP.sub("capsule-hint"), &layer)
            .patch_part(&hint_bg)
            .draw(ui, area);
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
        let _ = ui.layer(CAPSULE_HELP, |ui, area| {
            Self::with_capsule_help(|sections| {
                let help = HelpOverlay::new(CAPSULE_HELP, "Capsule", sections);
                help.draw(ui, area, &self.capsule_help_state)
            })
        });
        // ui.full() inside the layer is the popup clip, so decide out here.
        let screen = ui.full();
        let menu_80 = self.capsule_menu_80(ui.theme_ref(), screen);
        let menu_title = self.capsule_tab_menu_title();
        let _ = ui.layer(CAPSULE_TAB_MENU, |ui, area| {
            let menu = self.capsule_tab_context(self.capsule_tab_menu_pos, &menu_title);
            menu.draw(ui, area, &self.capsule_tab_menu_state);
            if menu_80 {
                // The popup clip is the live menu. Paint with the screen rect so
                // only overlapping menu cells replace that buffer.
                capsule_menu_view::paint_at(ui, screen);
            }
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
        let _ = ui.layer(CAPSULE_CONTAINER_INFO, |ui, area| {
            let info_lines = self.container_info_lines();
            info.draw(ui, area, &self.container_info_state, |ui, body| {
                paint_lines(ui, body, &info_lines)
            })
        });
        let about = Self::about_dialog();
        let _ = ui.layer(ABOUT_DIALOG, |ui, area| {
            let about_lines = self.about_lines();
            about.draw(ui, area, &self.about_state, |ui, body| {
                paint_lines(ui, body, &about_lines)
            })
        });
        let _ = ui.layer(QUIT_DIALOG, |ui, area| {
            let quit = Self::quit_dialog(&self.quit_body);
            quit.draw(ui, area, &self.quit_state, |_, _| {})
        });
        // Reduced color levels still read the approved cancel frame off the
        // page until the shared capability ladders reproduce the frozen dialog
        // inks (ghost 238 and danger 203 at 256, the 16-color card dim).
        if !(self.launch_cockpit_scene()
            && self.cancel_open
            && ui.theme_ref().capability.color != termrock::ColorLevel::TrueColor)
        {
            let cancel_launch = Self::cockpit_cancel_dialog();
            let _ = ui.layer(CANCEL_LAUNCH_DIALOG, |ui, area| {
                cancel_launch.draw(ui, area, &self.cancel_state, |_, _| {})
            });
        }
        if !self.launch_failure_frame(ui.full(), ui.theme_ref()) {
            let launch_failure = Self::launch_failure_dialog();
            let failure_lines = self.launch_failure_lines();
            let _ = ui.layer(LAUNCH_FAILURE_DIALOG, |ui, area| {
                launch_failure.draw(ui, area, &self.launch_failure_state, |ui, body| {
                    paint_lines(ui, body, &failure_lines)
                })
            });
        }
        let cockpit_info = Self::cockpit_info_dialog();
        let _ = ui.layer(COCKPIT_INFO_DIALOG, |ui, area| {
            let facts = self.info_facts();
            let shown: Vec<termrock::overlays::info_card::Fact<'_>> = facts
                .iter()
                .map(|(label, value)| termrock::overlays::info_card::Fact { label, value })
                .collect();
            cockpit_info.draw(ui, area, &self.cockpit_info_state, |ui, body| {
                termrock::overlays::info_card::paint_facts(ui, body, &shown, 0)
            })
        });
        let editor_exit = Self::editor_exit_dialog();
        let _ = ui.layer(EDITOR_EXIT_DIALOG, |ui, area| {
            editor_exit.draw(ui, area, &self.editor_exit_state, |_, _| {})
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
        // The approved picker frame is painted on the page. Drawing the live
        // modal would dim that page and place a cursor after this returns.
        if !self.historical_launch_picker(ui.theme_ref(), ui.full())
            && !self.historical_hard_launch_picker(ui.theme_ref(), ui.full())
        {
            let _ = ui.layer(crate::screens::manager::AGENT_PICKER, |ui, area| {
                agent_picker.draw(ui, area, &self.agent_state, &self.agent_options)
            });
        }
        let account_picker = self.active_account_picker();
        let account_items = match self.picker_mode {
            Some(PickerMode::OnePassword) => &self.op_options,
            _ => &self.account_options,
        };
        let _ = ui.layer(ACCOUNT_PICKER, |ui, area| {
            account_picker.draw(ui, area, &self.account_state, account_items);
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
        self.screen = cx.viewport();
        self.screen_junie = Self::junie_truecolor(cx.theme());
        let _shell = Self::shell_panel(&self.shell_meta);
        let mut result = self.advance_virtual_state(cx, product_tick);

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
        if self.quit_open || self.cancel_open || self.editor_exit_open {
            result |= self.update_overlays(cx);
            if self.route == Route::Manager {
                self.ensure_manager_rows();
            }
            self.ensure_manager_header();
            self.sync_workspace_keymap();
            return result;
        }
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
            || cx.is_open(ABOUT_DIALOG)
            || cx.is_open(QUIT_DIALOG)
            || cx.is_open(CANCEL_LAUNCH_DIALOG)
            || cx.is_open(COCKPIT_INFO_DIALOG)
            || cx.is_open(LAUNCH_FAILURE_DIALOG)
            || cx.is_open(EDITOR_EXIT_DIALOG)
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

    fn manager_menu_host(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // The approved menu sits on the paused returning manager, first row,
        // with no other overlay. Down, Tab, and other routes stay live.
        screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Manager
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
            && self
                .manager_rows_cache
                .first()
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
    }

    fn historical_manager_menu(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        self.manager_file_menu && self.manager_menu_host(theme, screen)
    }

    fn draw_historical_manager_menu(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_manager_menu(ui.theme_ref(), screen) {
            return;
        }
        // The menu layer composites after the page so the page cursor stays off this frame.
        let _ = ui.layer(MANAGER_FILE_MENU, |ui, _area| {
            manager_menu_view::paint(ui);
        });
    }

    fn launch_picker_host(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Enter on the paused 120×40 returning manager opens this picker.
        // The closed manager, F10, `?`, inspect, and the detail drawer stay off.
        !self.manager_file_menu
            && !self.manager_inspect
            && !self.help_open
            && screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Manager
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
            && self
                .manager_rows_cache
                .first()
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
    }

    fn launch_picker_enter(&self, cx: &Cx<'_>) -> bool {
        !self.launch_picker_open
            && !cx.is_open(crate::screens::manager::AGENT_PICKER)
            && self.launch_picker_host(cx.theme(), cx.viewport())
    }

    fn historical_launch_picker(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        self.launch_picker_open
            && self.status.as_deref() == Some("Launch · choose Agent")
            && !self.agent_options.is_empty()
            && self.launch_picker_host(theme, screen)
    }

    fn draw_historical_launch_picker(&self, ui: &mut Ui<'_>) {
        if !self.historical_launch_picker(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        launch_picker_view::paint(ui);
    }

    fn hard_launch_picker_host(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Enter on the paused 120×40 hard-cases manager opens this picker.
        // The returning picker, F10, `?`, inspect, and the closed manager stay off.
        !self.manager_file_menu
            && !self.manager_inspect
            && !self.help_open
            && screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Manager
            && self.world.scenario == Scenario::HardCases
            && self.motion == Motion::Paused
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
            && self
                .manager_rows_cache
                .first()
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
    }

    fn hard_launch_picker_enter(&self, cx: &Cx<'_>) -> bool {
        !self.launch_picker_open
            && !cx.is_open(crate::screens::manager::AGENT_PICKER)
            && self.hard_launch_picker_host(cx.theme(), cx.viewport())
    }

    fn historical_hard_launch_picker(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        self.launch_picker_open
            && self.status.as_deref() == Some("Launch · choose Agent")
            && !self.agent_options.is_empty()
            && self.hard_launch_picker_host(theme, screen)
    }

    fn draw_historical_hard_launch_picker(&self, ui: &mut Ui<'_>) {
        if !self.historical_hard_launch_picker(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        hard_launch_picker_view::paint(ui);
    }

    fn historical_hard_launch_locked(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Five Downs, Space, Down, and Enter on the paused 120×40 hard-cases
        // manager land on auth-service. Space is captured and does not expand.
        // The picker, the returning picker, F10, `?`, and the closed manager stay off.
        !self.manager_file_menu
            && !self.manager_inspect
            && !self.help_open
            && screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Manager
            && self.world.scenario == Scenario::HardCases
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && !self.launch_picker_open
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
            && self.manager.selected_row() == &ManagerRowKey::Workspace(12)
    }

    fn draw_historical_hard_launch_locked(&self, ui: &mut Ui<'_>) {
        if !self.historical_hard_launch_locked(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        let screen = ui.full();
        let menus = [
            termrock::feedback::SessionMenu {
                label: "File",
                focused: true,
            },
            termrock::feedback::SessionMenu {
                label: "Edit",
                focused: false,
            },
            termrock::feedback::SessionMenu {
                label: "View",
                focused: false,
            },
            termrock::feedback::SessionMenu {
                label: "Session",
                focused: false,
            },
            termrock::feedback::SessionMenu {
                label: "Help",
                focused: false,
            },
        ];
        let hints = [
            termrock::feedback::SessionHint {
                key: "Ctrl+B",
                action: "Pane commands…",
            },
            termrock::feedback::SessionHint {
                key: "F10",
                action: "Menu",
            },
            termrock::feedback::SessionHint {
                key: "Ctrl+\\",
                action: "Palette",
            },
            termrock::feedback::SessionHint {
                key: "Alt+Shift+↑↓←→",
                action: "Resize",
            },
        ];
        termrock::feedback::paint_session_chrome(
            ui,
            screen,
            &termrock::feedback::SessionChrome {
                brand: "jackin❯",
                menus: &menus,
                crumb: "› backend",
                instance: "jackin-data-pipeline-e0e0",
                instances: "3 instances",
                branch: "main",
                state: "clean",
                account: "no account · shell",
                hints: &hints,
                overflow: "…",
                attached: "Attached to  · tabs and panes restored",
            },
        );
    }

    fn historical_hard_cases_80(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // Paused 80×24 hard-cases manager. The 120×40 manager, picker, locked
        // launch, menu, and help stay on their own frames.
        let on_first = match self.manager_rows_cache.first() {
            None => {
                self.manager.list.cursor().is_none()
                    && self.manager.selected_row() == &ManagerRowKey::CurrentDirectory
            }
            Some(row) => {
                let cursor = self.manager.list.cursor();
                (cursor.is_none() || cursor == Some(row.key))
                    && (self.manager.selected_row() == &row.domain
                        || self.manager.selected_row() == &ManagerRowKey::CurrentDirectory)
            }
        };
        on_first
            && screen.width == 80
            && screen.height == 24
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Manager
            && self.world.scenario == Scenario::HardCases
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && !self.manager_file_menu
            && !self.manager_inspect
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && !self.launch_picker_open
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
    }

    fn draw_historical_hard_cases_80(&self, ui: &mut Ui<'_>) {
        if !self.historical_hard_cases_80(ui.theme_ref(), ui.full()) {
            return;
        }
        // Approved frame hides the cursor. This path must not call set_cursor
        // or set_bar_cursor, and it must not write cursor bytes.
        let screen = ui.full();
        let menus = ["File", "Go", "Help"];
        let rows = [
            termrock::feedback::ManagerLeaf {
                name: "payments-platform",
                expanded: true,
                status: "1 running",
                danger: false,
                muted: false,
                faint: false,
            },
            termrock::feedback::ManagerLeaf {
                name: "infra-control-plane",
                expanded: true,
                status: "1 running",
                danger: false,
                muted: false,
                faint: false,
            },
            termrock::feedback::ManagerLeaf {
                name: "release-automation",
                expanded: true,
                status: "! failed",
                danger: true,
                muted: false,
                faint: false,
            },
            termrock::feedback::ManagerLeaf {
                name: "customer-portal",
                expanded: true,
                status: "! failed",
                danger: true,
                muted: false,
                faint: false,
            },
            termrock::feedback::ManagerLeaf {
                name: "data-pipeline",
                expanded: true,
                status: "1 running",
                danger: false,
                muted: false,
                faint: false,
            },
            termrock::feedback::ManagerLeaf {
                name: "docs-site",
                expanded: true,
                status: "1 record",
                danger: false,
                muted: true,
                faint: false,
            },
            termrock::feedback::ManagerLeaf {
                name: "auth-service",
                expanded: false,
                status: "idle",
                danger: false,
                muted: false,
                faint: true,
            },
            termrock::feedback::ManagerLeaf {
                name: "gateway",
                expanded: false,
                status: "idle",
                danger: false,
                muted: false,
                faint: true,
            },
            termrock::feedback::ManagerLeaf {
                name: "mobile-app",
                expanded: false,
                status: "idle",
                danger: false,
                muted: false,
                faint: true,
            },
            termrock::feedback::ManagerLeaf {
                name: "ml-notebooks",
                expanded: false,
                status: "idle",
                danger: false,
                muted: false,
                faint: true,
            },
        ];
        termrock::feedback::paint_manager_frame(
            ui,
            screen,
            &termrock::feedback::ManagerFrame {
                brand: "jackin❯",
                menus: &menus,
                alert: "! instance index unreadable",
                title: "Workspaces",
                summary: "3 running · 1–11 of 16",
                selected: "Current directory",
                saved: "saved",
                saved_as: "as",
                saved_target: "payments-platform",
                rows: &rows,
                detail_heading: "Current directory",
                detail_side: "Tab details",
                detail_path: "/workspace/payments-platform · 2 mounts · 5 vars",
                detail_roles: "Roles the-architect ★ · 3 allowed · Auth Claude · Personal",
                instance_mark: "◉ ",
                instance_text: "7f3a  the-architect · Claude Code · ",
                instance_state: "running",
                hint_key: "?",
                hint_label: "Help",
                warning_mark: "▲",
                warning: " Could not confirm running instances: instance index unreadable …",
            },
        );
    }

    fn historical_help_overlay(&self, theme: &termrock::Theme, screen: Rect) -> bool {
        // `?` on the paused 120×40 returning manager. False for F10, Tab,
        // the expanded tree, inspect, and every other host.
        self.help_open
            && !self.manager_file_menu
            && !self.manager_inspect
            && screen.width == 120
            && screen.height == 40
            && theme.capability.color == termrock::ColorLevel::TrueColor
            && theme.color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && self.route == Route::Manager
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && self.status.as_deref() == Some("Keyboard shortcuts")
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
            && self
                .manager_rows_cache
                .first()
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
    }

    fn draw_historical_help_overlay(&self, ui: &mut Ui<'_>, screen: Rect) {
        if !self.historical_help_overlay(ui.theme_ref(), screen) {
            return;
        }
        // The approved frame hides the cursor. Paint the manager, the shared
        // help card, then dim only the underlay. Do not call set_cursor or
        // set_bar_cursor.
        self.paint_pinned_split(ui, screen, termrock::feedback::SplitMode::Saved, false);
        // White primary dims to muted. The approved title is faint, which is
        // where backdrop sends secondary ink.
        ui.paint_str(
            Rect::new(3, 2, 12, 1),
            " Workspaces ",
            PaintStyle::new().fg(termrock::Color::Rgb(0xb3, 0xb3, 0xb3)),
        );
        manager_help::draw(ui, screen, &HelpOverlayState::default());
        // Backdrop clears the focus bar and turns accent ink faint. The
        // approved gutter is a blank, so it dims to the overlay plane.
        let select = termrock::Color::Rgb(0x17, 0x3f, 0x1c);
        ui.paint_str(
            Rect::new(3, 3, 1, 1),
            " ",
            PaintStyle::new().fg(select).bg(select),
        );
        manager_help::dim_underlay(ui, screen);
        let hints = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("↑↓"),
                    label: "Scroll",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Chord(Chord::key(KeyCode::Esc)),
                    label: "Close",
                    priority: 80,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };
        let footer = [(
            Part::CONTAINER,
            termrock::StylePatch::new()
                .set_fg(termrock::Role::Custom(termrock::Color::Rgb(
                    0xff, 0xff, 0xff,
                )))
                .set_bg(termrock::Role::Custom(termrock::Color::Rgb(0, 0, 0))),
        )];
        HintBar::new(APP.sub("hint"), &hints)
            .patch_part(&footer)
            .draw(
                ui,
                Rect {
                    y: screen.bottom().saturating_sub(1),
                    height: 1,
                    ..screen
                },
            );
    }
}

fn plural(count: usize, one: &str, many: &str) -> String {
    if count == 1 {
        format!("1 {one}")
    } else {
        format!("{count} {many}")
    }
}

fn role_leaf(role: &str) -> &str {
    role.rsplit('/').next().unwrap_or(role)
}

fn tilde_path(home: &str, path: &str) -> String {
    path.strip_prefix(home)
        .map(|rest| format!("~{rest}"))
        .unwrap_or_else(|| path.to_owned())
}

fn isolation_word(isolation: crate::domain::workspace::Isolation) -> &'static str {
    match isolation {
        crate::domain::workspace::Isolation::Shared => "shared",
        crate::domain::workspace::Isolation::Worktree => "worktree",
        crate::domain::workspace::Isolation::Clone => "clone",
    }
}

fn wrap_account_lines(joined: &str) -> Vec<String> {
    let limit = 58;
    let chars: Vec<char> = joined.chars().collect();
    if chars.len() <= limit {
        return vec![joined.to_owned()];
    }
    let sep = [' ', '·', ' '];
    let mut cut = limit;
    for index in 0..=chars.len().saturating_sub(3) {
        if index + 3 <= limit
            && chars
                .get(index..index + 3)
                .is_some_and(|window| window == sep.as_slice())
        {
            cut = index + 3;
        }
    }
    let cut = cut.min(chars.len());
    let (head, tail) = chars.split_at(cut);
    vec![head.iter().collect(), tail.iter().collect()]
}

fn capsule_span_role(tone: crate::sim::pty::Tone) -> termrock::Role {
    use crate::sim::pty::Tone;
    match tone {
        // Baseline success is not the port token.
        Tone::Success => termrock::Role::Custom(termrock::Color::Rgb(0x3f, 0xb9, 0x50)),
        Tone::Error => termrock::Role::Danger,
        other => other.role(),
    }
}

fn truncate_chars(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

impl TuiApp for App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let now = cx.now();
        let interval = Duration::from_millis(self.wake_ms(cx));
        let last = *self.last_tick.get_or_insert(now);
        let due = last.saturating_add(interval);

        let product_tick = self.motion != Motion::Paused
            && cx.update_cause() == UpdateCause::Tick
            && now > last
            && now >= due;
        if product_tick {
            self.last_tick = Some(now);
        }
        let result = self.update_parts(cx, product_tick);
        if self.motion != Motion::Paused {
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
        if self.route == Route::Intro {
            self.draw_intro(ui, full);
            self.draw_layers(ui);
            return;
        }
        if self.route == Route::Outro {
            self.draw_outro(ui, full);
            self.draw_layers(ui);
            return;
        }
        if self.route == Route::Capsule {
            self.draw_capsule_shell(ui, full);
            self.draw_layers(ui);
            self.draw_capsule_zoom(ui, full);
            self.draw_capsule_menu_80(ui);
            if self.capsule_agent_picker_frame(ui.theme_ref(), full) {
                let title = match self.pending_capsule_action {
                    Some(CapsuleAction::NewTab) => "New tab",
                    _ => "Split ↓ Below",
                };
                capsule_split_view::paint(ui, title);
                ui.set_undrawn_top_bar_cursor(
                    CAPSULE_INPUT,
                    Rect::new(28, 16, 1, 1),
                    Position::new(28, 16),
                );
            }
            return;
        }
        let hints: &HintLayer = if self.help_open {
            &self.hint_layers.help
        } else {
            &self.hint_layers.default
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
        HintBar::new(APP.sub("hint"), hints).draw(
            ui,
            Rect {
                y: full.bottom().saturating_sub(1),
                height: 1,
                ..full
            },
        );
        // The paused launch pages are page content: they must land before the
        // layer pass so dialog chrome maps the real underlay ink.
        if self.launch_cockpit_scene() {
            launch_cockpit::paint(self, ui);
        } else if self.launch_failure_frame(full, ui.theme_ref()) {
            launch_cockpit::paint_failure(self, ui);
        }
        self.draw_layers(ui);
        let historical_manager = self.route == Route::Manager
            && full.width == 120
            && full.height == 40
            && matches!(
                self.world.scenario,
                Scenario::Returning | Scenario::HardCases
            )
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open();
        if historical_manager
            && self
                .manager_rows_cache
                .first()
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
        {
            // One Tab from the manager moves focus to Accounts and does not
            // open the live detail. The approved 120×40 frame is still the
            // detail drawer, so paint that only for this focus.
            let detail_tab = self.world.scenario == Scenario::Returning
                && ui.state(ACCOUNTS).contains(termrock::StateFlags::FOCUSED)
                && ui.theme_ref().capability.color == termrock::ColorLevel::TrueColor
                && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54);
            if detail_tab {
                self.draw_historical_detail_drawer(ui, full);
            } else {
                self.paint_pinned_split(ui, full, termrock::feedback::SplitMode::Saved, true);
            }
        } else if historical_manager
            && self.world.scenario == Scenario::Returning
            && self
                .manager_rows_cache
                .get(1)
                .is_some_and(|row| self.manager.list.cursor() == Some(row.key))
        {
            self.paint_pinned_split(ui, full, termrock::feedback::SplitMode::Expanded, true);
        }
        if self.route == Route::Manager
            && full.width == 120
            && full.height == 40
            && self.world.scenario == Scenario::FirstUse
            && self.motion == Motion::Paused
            && self.status.is_none()
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && self.launch.is_none()
            && self.picker_mode.is_none()
            && self.agent_options.is_empty()
            && self.active_instance.is_none()
            && !self.manager.detail_open()
        {
            self.paint_pinned_split(ui, full, termrock::feedback::SplitMode::Empty, true);
        }
        if self.route == Route::Prelude
            && full.width == 120
            && full.height == 40
            && self.world.scenario == Scenario::FirstUse
            && self.motion == Motion::Paused
            && self.prelude.step() == 1
            && ui.theme_ref().capability.color == termrock::ColorLevel::TrueColor
            && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
        {
            prelude_view::paint(self, ui, full);
        }
        let editor_allowed = (!self.editor.preview_open && self.status.is_none())
            || (self.editor.preview_open
                && self
                    .status
                    .as_deref()
                    .is_some_and(|status| status.starts_with("Save workspace")));
        if self.route == Route::Editor
            && full.width == 120
            && full.height == 40
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && ui.theme_ref().capability.color == termrock::ColorLevel::TrueColor
            && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && editor_allowed
            && !self.editor.env_form_open
            && !self.editor_role_picker
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.editor_exit_open
        {
            editor_view::paint(self, ui, full);
        }
        // Reduced color levels still read the approved info frame off the
        // page until the shared capability ladders reproduce the frozen card
        // inks at 256, 16 colors and mono.
        if self.launch_cockpit_scene() && self.cockpit_info_open {
            let reduced = ui.theme_ref().capability.color != termrock::ColorLevel::TrueColor;
            if reduced {
                let facts = self.info_facts();
                let shown: Vec<termrock::overlays::info_card::Fact<'_>> = facts
                    .iter()
                    .map(|(label, value)| termrock::overlays::info_card::Fact { label, value })
                    .collect();
                launch_dialog_plate::paint_info(ui, &shown);
            }
            let dim_clear = full.width == 120 && full.height == 40
                || (reduced
                    && matches!(
                        ui.theme_ref().capability.color,
                        termrock::ColorLevel::Ansi16 | termrock::ColorLevel::Mono
                    )
                    || std::env::var_os("NO_COLOR").is_some());
            if dim_clear && let Some(column) = launch_cockpit::stage_rail_left(full) {
                ui.clear_dim(column);
            }
            if full.width == 120 && full.height == 40 {
                launch_cockpit::paint_open_footer(ui, false);
            }
        }
        if self.launch_cockpit_scene()
            && self.cancel_open
            && ui.theme_ref().capability.color != termrock::ColorLevel::TrueColor
        {
            launch_dialog_plate::paint_cancel(ui);
            if let Some(column) = launch_cockpit::stage_rail_left(full) {
                ui.clear_dim(column);
            }
            if full.width == 120 && full.height == 40 {
                launch_cockpit::paint_open_footer(ui, true);
            }
        }
        if self.cockpit_trace_open {
            self.paint_cockpit_trace(ui, full);
        }
        self.draw_accounts_boot_needle(ui, full);
        self.draw_accounts_boot_needle_80(ui, full);
        self.draw_historical_audit_accounts_80(ui);
        self.draw_historical_accounts_add_form(ui, full);
        self.draw_historical_accounts_filter(ui, full);
        self.draw_historical_accounts_filter_80(ui);
        if self.route == Route::Settings
            && self.world.scenario == Scenario::AccountsMixed
            && self.motion == Motion::Paused
            && ui.theme_ref().capability.color == termrock::ColorLevel::TrueColor
            && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
            && !self.help_open
            && !self.quit_open
            && !self.cancel_open
            && !self.about_open
            && !self.editor_exit_open
            && !self.cockpit_info_open
            && !self.launch_failure_open
            && self.picker_mode.is_none()
        {
            settings_view::paint(self, ui, full);
        }
        self.draw_historical_accounts_detail(ui, full);
        self.draw_historical_accounts_detail_80(ui);
        self.draw_historical_accounts_drawer(ui, full);
        self.draw_historical_usage_overview(ui, full);
        self.draw_historical_usage_detail(ui, full);
        if self.manager_inspect
            && self.route == Route::Manager
            && full.width == 120
            && full.height == 40
            && self.world.scenario == Scenario::Returning
            && self.motion == Motion::Paused
            && ui.theme_ref().capability.color == termrock::ColorLevel::TrueColor
            && ui.theme_ref().color.accent == termrock::Color::Rgb(0x48, 0xe0, 0x54)
        {
            inspect_view::paint(ui);
        }
        self.draw_historical_manager_menu(ui, full);
        self.draw_historical_launch_picker(ui);
        self.draw_historical_hard_launch_picker(ui);
        self.draw_historical_hard_launch_locked(ui);
        self.draw_historical_hard_cases_80(ui);
        self.draw_historical_help_overlay(ui, full);
    }

    fn should_quit(&self) -> bool {
        self.quit
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }

    fn min_size(&self) -> termrock::Size {
        termrock::Size {
            min: (72, 20),
            preferred: (120, 40),
        }
    }

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.quit_open {
            self.quit_open = false;
            cx.close_layer(QUIT_DIALOG, Some(ActionKey::CANCEL));
            return self.route_changed();
        }
        if self.cancel_open {
            self.cancel_open = false;
            self.cancel_from_x = false;
            cx.close_layer(CANCEL_LAUNCH_DIALOG, Some(ActionKey::CANCEL));
            return self.route_changed();
        }
        if self.cockpit_info_open {
            self.close_cockpit_info(cx);
            return self.route_changed();
        }
        if self.editor_exit_open {
            self.close_editor_exit(cx);
            return self.route_changed();
        }
        if self
            .launch
            .as_ref()
            .is_some_and(|run| run.failure.is_some())
            && matches!(self.route, Route::Manager | Route::Cockpit | Route::Launch)
        {
            self.acknowledge_launch_failure();
            cx.focus(MANAGER_LIST);
            return self.route_changed();
        }
        if self.about_open {
            self.about_open = false;
            self.status = None;
            cx.close_layer(ABOUT_DIALOG, Some(ActionKey::CLOSE));
            return self.route_changed();
        }
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
        if self.route == Route::Usage && self.usage.detail_open() {
            self.usage.close_detail();
            self.status = None;
            return self.route_changed();
        }
        if self.route == Route::Accounts {
            if self.accounts.filtering {
                self.accounts.filtering = false;
                self.accounts.filter_query.clear();
                self.accounts.filter_input = TextInputState::default();
                self.status = None;
                return self.route_changed();
            }
            if self.accounts.drawer_open {
                self.accounts.drawer_open = false;
                self.status = None;
                return self.route_changed();
            }
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
                self.dismiss_editor_env_form(cx);
                return self.route_changed();
            }
            if self.editor_leaf_focused {
                self.editor_leaf_focused = false;
                cx.focus(crate::screens::editor::SAVE);
                return self.route_changed();
            }
            if self.editor.dirty {
                self.open_editor_exit(cx);
            } else {
                self.route = Route::Manager;
            }
            return self.route_changed();
        }
        if self.route == Route::Settings {
            if self.settings_save_preview {
                self.settings_save_preview = false;
                self.status = None;
                return self.route_changed();
            }
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
            if self.manager_inspect {
                self.manager_inspect = false;
                cx.focus(MANAGER_LIST);
                return self.route_changed();
            }
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

fn account_selection_status(account: &Account) -> String {
    format!(
        "Accounts › {} › {}",
        account.surface.surface_name(),
        account.display_name
    )
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
    fn role_labels_require_an_exact_current_catalog_identity() {
        let mut app = App::default();
        assert_eq!(app.role_label("chainargos/backend"), "backend");
        assert_eq!(app.role_label("other/backend"), "other/backend");
        assert_eq!(app.role_label("chainargosx/backend"), "chainargosx/backend");
        assert_eq!(app.role_label("chainargos/missing"), "chainargos/missing");
        if let Some(role) = app
            .world
            .roles
            .iter_mut()
            .find(|role| role.name == "backend")
        {
            role.name = "renamed".into();
        }
        assert_eq!(app.role_label("chainargos/backend"), "chainargos/backend");
        assert_eq!(app.role_label("chainargos/renamed"), "renamed");
    }

    #[test]
    fn default_starts_in_returning_manager() {
        let app = App::default();
        assert_eq!(app.route(), Route::Manager);
        assert_eq!(app.world.running_count(), 2);
        assert_eq!(
            app.world.instances[0].run_id,
            crate::RunId::from_label("run-7f3a")
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
        let key = termrock::Key {
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
        use termrock::{ColorLevel, Theme};
        use termrock_test_support::Harness;
        for theme in [Theme::junie(), Theme::paper()] {
            for (scenario, x, y, glyph) in [
                (Scenario::CapsuleMulti, 103, 7, "F"),
                (Scenario::CapsuleMulti, 2, 0, "j"),
                (Scenario::Returning, 20, 39, "L"),
            ] {
                let app = App::for_scenario_at(scenario, Motion::Paused, 0);
                let h = Harness::new(app, theme.clone(), 120, 40).with_color(ColorLevel::Mono);
                let (cx, cy) = if scenario == Scenario::CapsuleMulti && glyph == "F" {
                    h.find("FAILED").expect("FAILED in mono")
                } else if scenario == Scenario::CapsuleMulti && glyph == "j" {
                    h.find("jackin❯").expect("jackin in mono")
                } else if scenario == Scenario::Returning && glyph == "L" {
                    h.find("Launch").expect("Launch in mono")
                } else {
                    (x, y)
                };
                let cell = h.cell(cx, cy);
                assert_eq!(cell.symbol(), glyph);
                assert_ne!(
                    cell.fg, cell.bg,
                    "critical custom text must remain readable"
                );
            }
        }

        let failed = Harness::new(
            App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        )
        .with_color(ColorLevel::Mono);
        let label = failed.find("FAILED").expect("failure status label");
        for (offset, glyph) in "FAILED".chars().enumerate() {
            let x = label.0 + u16::try_from(offset).expect("short label offset");
            let cell = failed.cell(x, label.1);
            assert_eq!(cell.symbol(), glyph.to_string());
            assert_ne!(cell.fg, cell.bg, "FAILED label must stay readable in mono");
        }
    }

    #[test]
    fn paused_first_use_at_frame_400_paints_the_empty_manager_chrome() {
        use termrock::Theme;
        use termrock_test_support::Harness;
        let settled = Harness::new(
            App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(settled.cell(2, 0).symbol(), "j");
        assert_eq!(settled.cell(106, 0).symbol(), "n");
        assert_eq!(settled.cell(22, 2).symbol(), " ");
        assert_eq!(settled.cell(7, 3).symbol(), "C");
        assert_eq!(settled.cell(108, 2).symbol(), "n");
        assert_eq!(settled.cell(60, 17).symbol(), "C");
        assert_eq!(settled.cell(42, 36).symbol(), "L");
        assert_eq!(settled.cell(52, 36).symbol(), "C");
        assert_eq!(settled.cell(1, 39).symbol(), "E");
        assert_eq!(settled.cell(22, 39).symbol(), "T");
        let intro = Harness::new(
            App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert!(intro.text().contains("Enter Skip"), "{}", intro.text());
        assert!(!intro.text().contains("no instances"), "{}", intro.text());
    }

    #[test]
    fn paused_first_use_at_frame_300_paints_the_warp_not_the_manager() {
        use termrock::{Modifier, Theme};
        use termrock_test_support::Harness;
        let warp = Harness::new(
            App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 300),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(warp.cell(29, 0).symbol(), "─");
        assert_eq!(warp.cell(108, 39).symbol(), "E");
        assert!(warp.cell(108, 39).modifier.contains(Modifier::BOLD));
        assert_eq!(warp.cell(114, 39).symbol(), "S");
        assert_eq!(warp.cell(2, 0).symbol(), " ");
        let settled = Harness::new(
            App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(settled.cell(2, 0).symbol(), "j");
        assert_eq!(settled.cell(106, 0).symbol(), "n");
        assert_ne!(settled.cell(29, 0).symbol(), "─");
    }

    #[test]
    fn returning_editor_general_paints_the_approved_120x40_frame() {
        use termrock::{Color, KeyCode, Modifier, Theme};
        use termrock_test_support::Harness;
        let mut editor = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = editor.key(KeyCode::Char('e'));
        let rule = editor.cell(2, 4);
        assert_eq!(rule.symbol(), "━");
        assert_eq!(rule.fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert_eq!(rule.bg, Color::Rgb(0, 0, 0));
        assert!(!rule.modifier.contains(Modifier::BOLD));
        let brand = editor.cell(2, 0);
        assert_eq!(brand.symbol(), "j");
        assert_eq!(brand.fg, Color::Rgb(0x19, 0x19, 0x1c));
        assert_eq!(brand.bg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(brand.modifier.contains(Modifier::BOLD));
        assert!(editor.cursor().is_none());
        assert!(editor.text().contains("General"));

        let mut mounts = App::for_scenario(Scenario::Returning, Motion::Paused);
        let workspace = mounts.world.workspaces[0].clone();
        mounts.route = Route::Editor;
        mounts.editor.load_workspace(&workspace);
        mounts.editor.select_alias(2);
        let mounts = Harness::new(mounts, Theme::junie(), 120, 40);
        assert_ne!(mounts.cell(2, 4).symbol(), "━");
        let underline = mounts.cell(13, 4);
        assert_eq!(underline.symbol(), "━");
        assert_eq!(underline.fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(mounts.cursor().is_none());

        let mut keyed = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = keyed.key(KeyCode::Char('e'));
        let _ = keyed.key(KeyCode::Char('2'));
        let _ = keyed.key(KeyCode::Enter);
        assert_eq!(keyed.app().editor.tab, EditorTab::Mounts);
        assert!(!keyed.app().editor.dirty);
        assert!(!keyed.app().editor.preview_open);
        assert!(keyed.app().status.is_none());
        assert_eq!(keyed.cell(13, 4).symbol(), "━");
        assert_eq!(keyed.cell(13, 4).fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert_eq!(keyed.cell(2, 7).symbol(), "▎");
        assert!(keyed.cursor().is_none());

        let mut paper = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        let _ = paper.key(KeyCode::Char('e'));
        let paper_rule = paper.cell(2, 4);
        assert!(
            paper_rule.symbol() != "━" || paper_rule.fg != Color::Rgb(0x48, 0xe0, 0x54),
            "paper must keep the live editor"
        );
    }

    #[test]
    fn returning_editor_mounts_dirty_paints_the_approved_120x40_frame() {
        use termrock::{Color, KeyCode, Theme};
        use termrock_test_support::Harness;
        let mut keyed = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = keyed.key(KeyCode::Char('e'));
        let _ = keyed.key(KeyCode::Char('2'));
        let _ = keyed.key(KeyCode::Enter);
        let _ = keyed.key(KeyCode::Char('r'));
        let _ = keyed.key(KeyCode::Char('i'));
        assert!(keyed.app().editor.dirty);
        assert_eq!(keyed.app().editor.tab, EditorTab::Mounts);
        assert!(!keyed.app().editor.preview_open);
        assert!(keyed.app().status.is_none());
        assert!(keyed.cursor().is_none());
        assert!(keyed.text().contains("isolation clone"), "{}", keyed.text());
        assert!(keyed.text().contains("• 1 change"), "{}", keyed.text());
        let bullet = keyed.cell(86, 0);
        assert_eq!(bullet.symbol(), "•");
        assert_eq!(bullet.fg, Color::Rgb(0xf5, 0x9e, 0x09));
        let mode = keyed.cell(52, 7);
        assert_eq!(mode.symbol(), "c");
        assert_eq!(mode.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        assert_eq!(mode.bg, Color::Rgb(0x17, 0x3f, 0x1c));
    }

    #[test]
    fn capsule_split_vertical_paints_the_approved_120x40_frame() {
        use termrock::{KeyCode, Theme};
        use termrock_test_support::Harness;
        let mut split = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = split.ctrl('b');
        let _ = split.key(KeyCode::Char('"'));
        assert_eq!(split.app().status.as_deref(), None);
        assert!(
            split.app().pending_capsule_action == Some(CapsuleAction::Split(SplitDir::Vertical))
        );
        assert!(split.text().contains("Split ↓ Below"), "{}", split.text());
        assert!(split.text().contains("Type to search…"), "{}", split.text());
        assert!(
            split.text().contains("2 accounts · choose at start"),
            "{}",
            split.text()
        );
        assert!(
            !split
                .text()
                .contains("Split below · Account for Claude Code"),
            "{}",
            split.text()
        );

        let bare = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            !bare
                .text()
                .contains("Split below · Account for Claude Code"),
            "{}",
            bare.text()
        );

        let mut paper = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        let _ = paper.ctrl('b');
        let _ = paper.key(KeyCode::Char('"'));
        assert!(
            paper
                .text()
                .contains("Split below · Account for Claude Code"),
            "paper must keep the live picker"
        );

        let mut narrow = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            80,
            24,
        );
        let _ = narrow.ctrl('b');
        let _ = narrow.key(KeyCode::Char('"'));
        assert!(
            narrow
                .text()
                .contains("Split below · Account for Claude Code")
        );
    }

    #[test]
    fn capsule_new_tab_paints_the_approved_120x40_frame() {
        use termrock::{KeyCode, Theme};
        use termrock_test_support::Harness;
        let mut tab = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = tab.ctrl('b');
        let _ = tab.key(KeyCode::Char('c'));
        assert_eq!(tab.app().status.as_deref(), None);
        assert!(tab.text().contains("New tab"), "{}", tab.text());
        assert!(tab.text().contains("Type to search…"), "{}", tab.text());
        assert!(
            tab.text().contains("2 accounts · choose at start"),
            "{}",
            tab.text()
        );
        assert!(!tab.text().contains("Change title…"), "{}", tab.text());
        assert!(!tab.text().contains("Command palette"), "{}", tab.text());
        assert!(
            !tab.text().contains("New tab · Account for Claude Code"),
            "{}",
            tab.text()
        );

        let mut menu = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = menu.ctrl('b');
        let _ = menu.key(KeyCode::Char('m'));
        assert!(menu.text().contains("Change title…"), "{}", menu.text());
        assert!(!menu.text().contains("Type to search…"), "{}", menu.text());

        let mut palette = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = palette.ctrl('b');
        let _ = palette.key(KeyCode::Char(' '));
        assert!(
            palette.text().contains("Command palette"),
            "{}",
            palette.text()
        );
        assert!(
            !palette.text().contains("Type to search…"),
            "{}",
            palette.text()
        );

        let bare = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        assert!(!bare.text().contains("Type to search…"), "{}", bare.text());

        let mut paper = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        let _ = paper.ctrl('b');
        let _ = paper.key(KeyCode::Char('c'));
        assert_eq!(paper.app().status.as_deref(), Some("New tab"));

        let mut narrow = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            80,
            24,
        );
        let _ = narrow.ctrl('b');
        let _ = narrow.key(KeyCode::Char('c'));
        assert_eq!(narrow.app().status.as_deref(), Some("New tab"));
    }

    #[test]
    fn capsule_zoom_paints_the_approved_120x40_frame() {
        use termrock::{Color, KeyCode, Modifier, Position, Theme};
        use termrock_test_support::Harness;
        let mut zoom = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = zoom.ctrl('b');
        let _ = zoom.key(KeyCode::Char('z'));
        assert_eq!(zoom.app().status.as_deref(), Some("zoom · focused pane"));
        assert!(zoom.app().capsule.zoomed);
        assert_eq!(zoom.app().capsule.selected_pane, 1);
        assert!(
            zoom.text().contains("Zoomed · z restores the layout"),
            "{}",
            zoom.text()
        );
        let badge = zoom.cell(111, 4);
        assert_eq!(badge.symbol(), "z");
        assert_eq!(badge.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!(badge.bg, Color::Rgb(0x00, 0x00, 0x00));
        assert!(!badge.modifier.contains(Modifier::BOLD));
        let title = zoom.cell(3, 4);
        assert_eq!(title.symbol(), "C");
        assert_eq!(title.fg, Color::Rgb(0xff, 0xff, 0xff));
        assert_eq!(title.bg, Color::Rgb(0x00, 0x00, 0x00));
        assert!(title.modifier.contains(Modifier::BOLD));
        let usage = zoom.cell(114, 38);
        assert_eq!(usage.symbol(), "7");
        assert_eq!(usage.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(usage.bg, Color::Rgb(0x18, 0x18, 0x1b));
        assert!(usage.modifier.contains(Modifier::ITALIC));
        assert!(!usage.modifier.contains(Modifier::BOLD));
        assert_eq!(zoom.cursor(), Some(Position::new(3, 34)));

        let bare = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        assert!(
            !bare.text().contains("Zoomed · z restores the layout"),
            "{}",
            bare.text()
        );
        assert_ne!(bare.cell(111, 4).symbol(), "z");

        let mut paper = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        let _ = paper.ctrl('b');
        let _ = paper.key(KeyCode::Char('z'));
        assert!(
            !paper.text().contains("Zoomed · z restores the layout"),
            "paper must keep the live zoom"
        );

        let mut narrow = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            80,
            24,
        );
        let _ = narrow.ctrl('b');
        let _ = narrow.key(KeyCode::Char('z'));
        assert!(!narrow.text().contains("Zoomed · z restores the layout"));
    }

    #[test]
    fn accounts_add_form_paints_the_approved_120x40_frame() {
        use termrock::{Color, KeyCode, Modifier, Position, Theme};
        use termrock_test_support::Harness;
        let mut form = Harness::new(
            App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        assert!(form.text().contains("jackin❯"), "{}", form.text());
        assert!(!form.text().contains("form · unsaved"), "{}", form.text());
        let _ = form.key(KeyCode::Char('a'));
        assert!(form.app().accounts.form_open);
        assert!(!form.app().accounts.started);
        assert!(form.text().contains("New account"), "{}", form.text());
        let title = form.cell(28, 5);
        assert_eq!(title.symbol(), "N");
        assert_eq!(title.fg, Color::Rgb(0xff, 0xff, 0xff));
        assert_eq!(title.bg, Color::Rgb(0x18, 0x18, 0x1b));
        assert!(title.modifier.contains(Modifier::BOLD));
        let brand = form.cell(2, 0);
        assert_eq!(brand.symbol(), "j");
        assert_eq!(brand.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!(brand.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(!brand.modifier.contains(Modifier::BOLD));
        let field = form.cell(29, 7);
        assert_eq!(field.symbol(), "P");
        assert_eq!(field.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(field.bg, Color::Rgb(0x1e, 0x1e, 0x22));
        assert_eq!(form.cursor(), Some(Position::new(29, 7)));

        let overview = Harness::new(
            App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        assert_ne!(overview.cell(28, 5).symbol(), "N");

        let mut paper = Harness::new(
            App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        let _ = paper.key(KeyCode::Char('a'));
        assert!(
            paper.cell(28, 5).symbol() != "N"
                || paper.cell(28, 5).fg != Color::Rgb(0xff, 0xff, 0xff),
            "paper must keep the live form"
        );

        let mut narrow = Harness::new(
            App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
            Theme::junie(),
            80,
            24,
        );
        let _ = narrow.key(KeyCode::Char('a'));
        assert_ne!(narrow.cell(28, 5).symbol(), "N");
    }

    #[test]
    fn accounts_add_form_required_paints_only_after_backtab_enter() {
        use termrock::{Color, KeyCode, KeyModifiers, Modifier, Position, Theme};
        use termrock_test_support::Harness;
        let mut form = Harness::new(
            App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = form.key(KeyCode::Char('a'));
        assert!(form.text().contains("New account"), "{}", form.text());
        assert!(
            !form.text().contains("Required"),
            "empty form must not show Required"
        );
        assert_ne!(form.cell(29, 8).symbol(), "R");
        assert_eq!(form.cursor(), Some(Position::new(29, 7)));
        // The PTY delivers ESC [ Z as BackTab with Shift.
        let _ = form.key_mod(KeyCode::BackTab, KeyModifiers::SHIFT);
        assert!(!form.text().contains("Required"));
        assert_eq!(form.cursor(), Some(Position::new(29, 7)));
        assert!(form.app().accounts.form_open);
        assert!(!form.app().accounts.started);
        let _ = form.key(KeyCode::Enter);
        assert!(form.text().contains("Required"), "{}", form.text());
        let required = form.cell(29, 8);
        assert_eq!(required.symbol(), "R");
        assert_eq!(required.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
        assert_eq!(required.bg, Color::Rgb(0x18, 0x18, 0x1b));
        assert!(!required.modifier.contains(Modifier::BOLD));
        let mark = form.cell(90, 7);
        assert_eq!(mark.symbol(), "!");
        assert_eq!(mark.fg, Color::Rgb(0xf0, 0x5d, 0x5d));
        assert!(mark.modifier.contains(Modifier::BOLD));
        assert_eq!(form.cursor(), Some(Position::new(29, 7)));
        assert!(!form.app().accounts.started);

        let mut empty = Harness::new(
            App::for_scenario(Scenario::AccountsMixed, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = empty.key(KeyCode::Char('a'));
        assert!(!empty.text().contains("Required"));
        assert_ne!(empty.cell(29, 8).symbol(), "R");
        assert_eq!(empty.cursor(), Some(Position::new(29, 7)));
    }

    #[test]
    fn hard_launch_picker_paints_the_approved_120x40_frame() {
        use termrock::{Color, KeyCode, Modifier, Theme};
        use termrock_test_support::Harness;
        let mut closed = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let before = closed.cell(87, 7);
        assert_ne!(
            (before.symbol(), before.fg, before.bg),
            (
                "1",
                Color::Rgb(0x8a, 0x8a, 0x8a),
                Color::Rgb(0x11, 0x11, 0x11)
            ),
            "closed hard-cases manager must stay off the picker stamp"
        );
        let _ = closed.key(KeyCode::Enter);
        assert_eq!(
            closed.app().status.as_deref(),
            Some("Launch · choose Agent")
        );
        assert!(closed.app().launch_picker_open);
        let roles = closed.cell(87, 7);
        assert_eq!(roles.symbol(), "1");
        assert_eq!(roles.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(roles.bg, Color::Rgb(0x11, 0x11, 0x11));
        assert!(!roles.modifier.contains(Modifier::BOLD));
        assert!(closed.cursor().is_none());

        let mut returning = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = returning.key(KeyCode::Enter);
        assert_ne!(returning.cell(87, 7).symbol(), "1");

        let mut paper = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        let _ = paper.key(KeyCode::Enter);
        let paper_roles = paper.cell(87, 7);
        assert!(
            paper_roles.symbol() != "1" || paper_roles.fg != Color::Rgb(0x8a, 0x8a, 0x8a),
            "paper must keep the live hard-cases picker"
        );
    }

    #[test]
    fn hard_launch_locked_paints_the_approved_120x40_frame() {
        use termrock::{Color, KeyCode, Modifier, Theme};
        use termrock_test_support::Harness;
        let mut closed = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let before = closed.cell(80, 0);
        assert_ne!(
            (before.symbol(), before.fg, before.bg),
            (
                "j",
                Color::Rgb(0x8a, 0x8a, 0x8a),
                Color::Rgb(0x00, 0x00, 0x00)
            ),
            "closed hard-cases manager must stay off the locked stamp"
        );
        for code in [
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Char(' '),
            KeyCode::Down,
            KeyCode::Enter,
        ] {
            let _ = closed.key(code);
        }
        let instance = closed.cell(80, 0);
        assert_eq!(instance.symbol(), "j");
        assert_eq!(instance.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(instance.bg, Color::Rgb(0x00, 0x00, 0x00));
        assert!(!instance.modifier.contains(Modifier::BOLD));
        assert!(closed.cursor().is_none());

        let mut picker = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = picker.key(KeyCode::Enter);
        assert_ne!(picker.cell(80, 0).symbol(), "j");

        let mut returning = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = returning.key(KeyCode::Enter);
        assert_ne!(returning.cell(80, 0).symbol(), "j");

        let mut menu = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = menu.key(KeyCode::F(10));
        assert_ne!(menu.cell(80, 0).symbol(), "j");

        let mut help = Harness::new(
            App::for_scenario(Scenario::Returning, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = help.key(KeyCode::Char('?'));
        assert_ne!(help.cell(80, 0).symbol(), "j");

        let mut paper = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::paper(),
            120,
            40,
        );
        for code in [
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Down,
            KeyCode::Char(' '),
            KeyCode::Down,
            KeyCode::Enter,
        ] {
            let _ = paper.key(code);
        }
        let paper_cell = paper.cell(80, 0);
        assert!(
            paper_cell.symbol() != "j" || paper_cell.fg != Color::Rgb(0x8a, 0x8a, 0x8a),
            "paper must keep the live hard-cases manager"
        );
    }

    #[test]
    fn capsule_menu_80_paints_the_approved_frame() {
        use termrock::{Color, KeyCode, Modifier, Position, Theme};
        use termrock_test_support::Harness;
        let mut menu = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            80,
            24,
        );
        let _ = menu.ctrl('b');
        let _ = menu.key(KeyCode::Char('m'));
        assert!(menu.app().capsule_tab_menu_open);
        let ellipsis = menu.cell(77, 0);
        assert_eq!(ellipsis.symbol(), "…");
        assert_eq!(ellipsis.fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(ellipsis.bg, Color::Rgb(0x00, 0x00, 0x00));
        assert!(!ellipsis.modifier.contains(Modifier::BOLD));
        assert!(!ellipsis.modifier.contains(Modifier::ITALIC));
        assert_eq!(menu.cursor(), Some(Position::new(3, 19)));
        assert!(menu.runtime().cursor_bar());

        let mut wide = Harness::new(
            App::for_scenario(Scenario::CapsuleMulti, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        let _ = wide.ctrl('b');
        let _ = wide.key(KeyCode::Char('m'));
        assert_ne!(wide.cell(77, 0).symbol(), "…");
        assert_ne!(wide.cursor(), Some(Position::new(3, 19)));
    }

    #[test]
    fn hard_cases_80_paints_the_approved_frame() {
        use termrock::{Color, Modifier, Theme};
        use termrock_test_support::Harness;
        let frame = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::junie(),
            80,
            24,
        );
        let mark = frame.cell(67, 2);
        assert_eq!(mark.symbol(), "\u{2013}");
        assert_eq!(mark.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!(mark.bg, Color::Rgb(0x00, 0x00, 0x00));
        assert!(!mark.modifier.contains(Modifier::BOLD));
        assert!(frame.cursor().is_none());

        let wide = Harness::new(
            App::for_scenario(Scenario::HardCases, Motion::Paused),
            Theme::junie(),
            120,
            40,
        );
        assert_ne!(wide.cell(67, 2).symbol(), "\u{2013}");
    }

    #[test]
    fn audit_accounts_80_paints_the_approved_overview_cells() {
        use termrock::{Color, Theme};
        use termrock_test_support::Harness;
        let frame = Harness::new(
            App::for_scenario_at(Scenario::AccountsMixed, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        let prompt = frame.cell(2, 0);
        assert_eq!(prompt.symbol(), "j");
        assert_eq!(prompt.fg, Color::Rgb(0x19, 0x19, 0x1c));
        assert_eq!(prompt.bg, Color::Rgb(0x48, 0xe0, 0x54));
        let badge = frame.cell(71, 2);
        assert_eq!(badge.symbol(), "o");
        assert_eq!(badge.fg, Color::Rgb(0x73, 0x73, 0x73));
        assert_eq!(badge.bg, Color::Rgb(0x00, 0x00, 0x00));
        assert!(frame.cursor().is_none());

        let mut settings = Harness::new(
            App::for_scenario_at(Scenario::AccountsMixed, Motion::Paused, 40),
            Theme::junie(),
            80,
            24,
        );
        assert_eq!(settings.app().route(), Route::Accounts);
        let _ = settings.key(KeyCode::Char('s'));
        assert_eq!(settings.app().route(), Route::Settings);
        assert_ne!(settings.cell(71, 2).symbol(), "o");
    }
}
