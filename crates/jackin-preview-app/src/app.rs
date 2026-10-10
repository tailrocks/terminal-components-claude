//! Jackin Preview application shell.
//!
//! This module owns only interaction state and paints through `tui-next`'s
//! public facade.  Domain and simulation state stay in sibling modules.

use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{BTreeMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    mem,
    time::Duration,
};
use termrock::author::PaintStyle;

use termrock::{
    Action, ActionKey, Anchor, App as TuiApp, AsItem, Brand, Button, Checkbox, Chord, ContextMenu,
    CrossAlign, Cx, Dialog, DialogAction, DialogState, Empty, EmptyState, FgStep, Field, FrameRead,
    HelpAction, HelpOverlay, HelpOverlayState, HelpSection, Hint, HintBar, HintKey, HintLayer, Id,
    Insets, Intent, Item, ItemKey, ItemRowLayout, KeyCode, KeyMap, KeyModifiers, KeyPhase,
    LayerEvent, LayerSize, LayerSpec, List, ListAction, ListState, Menu, MenuAction, MenuBar,
    MenuItem, MenuState, MeterTone, Modifier, Moment, Panel, PanelKind, Part, PartRef, Phase,
    Picker, PickerAction, PickerState, Position, ProjectedText, Props, PropsAction, PropsList,
    PropsRow, PropsState, RadioGroup, RadioGroupState, Reconcile, Rect, Response, Role, RowUi,
    ScrollRegion, ScrollState, SecretPolicy, Select, SelectField, SelectState, Side,
    SimulationMoment, Slot, SplitAxis, SplitPane, SplitPaneState, StateFlags, Status, StatusBar,
    StatusItem, StylePatch, Surface, Tabs, TabsAction, TabsState, TextAction, TextInput,
    TextInputState, TextViewport, TooSmall, Ui, UpdateCause, Variant, ViewportAction, ViewportLine,
    ViewportState,
};

use crate::domain::account::{
    Account, AccountRegistry, CredentialSource, DetectedKind, DuplicateProbe, ValidationState,
    fingerprint, tail_of,
};
use crate::domain::agent::{Agent, Provider};
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::domain::usage::{Freshness, QuotaStatus};
use crate::domain::workspace::{EnvValue, EnvVar, Workspace, env_key_error};
use crate::rain::{HANDOFF_LEN, INTRO_END, IntroState, OutroState};
use crate::scenario::{Motion, Scenario};
use crate::screens::{
    accounts::{AccountSel, AccountsState},
    capsule::{CapsuleInteraction, CapsuleState},
    cockpit::{AccountLine, CockpitState},
    editor::{EditorState, Tab as EditorTab},
    inspect::InspectState,
    manager::{LaunchCandidate, ManagerRowKey, ManagerState},
    prelude::{PreludeState, PreludeUiState},
    settings::{SettingsFocus, SettingsScreen, SettingsState},
    usage::UsageState,
};
use crate::sim::launch::{LaunchEvent, LaunchPlan, LaunchRun};
use crate::sim::provider;
use crate::sim::pty::{Daemon, Maximized, Pane, PaneId, PaneNode, SplitDir, Tab};
use crate::sim::world::{DaemonHealth, World, world_for};

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
const ACCOUNTS_FILTER_INPUT: Id = crate::screens::accounts::TREE.sub("filter");
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
/// Capsule status strip id.
const CAPSULE_STATUS: Id = APP.sub("capsule-status");
/// Capsule empty-state surface id.
const CAPSULE_PANES_EMPTY: Id = CAPSULE_PANES.sub("empty");
/// Capsule split-seam component namespace.
const CAPSULE_SPLIT: Id = CAPSULE_PANES.sub("split");
/// Exit confirmation dialog id.
pub const QUIT_DIALOG: Id = APP.sub("quit-dialog");
/// Launch confirmation dialog id.
pub const LAUNCH_DIALOG: Id = APP.sub("launch-dialog");
/// Launch-cancel confirmation dialog id.
pub const CANCEL_DIALOG: Id = APP.sub("cancel-dialog");
/// About dialog layer id.
pub const ABOUT_DIALOG: Id = APP.sub("about-dialog");
/// About dialog props list id.
pub const ABOUT_PROPS: Id = ABOUT_DIALOG.sub("props");
/// About dialog close action id.
pub const ABOUT_CLOSE: Id = ABOUT_DIALOG.sub("close");
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
/// Property list inside the container identity dialog.
const CONTAINER_INFO_PROPS: Id = CAPSULE_CONTAINER_INFO.sub("props");
/// Stage lines on the cockpit-to-capsule handoff body.
const HANDOFF_LINES: Id = APP.sub("handoff-lines");
/// Right-aligned facts on a host screen header.
const HOST_HEADER: Id = APP.sub("host-header");
/// Cockpit and handoff breadcrumb strip.
const ROUTE_STRIP: Id = APP.sub("route-strip");
const CAPSULE_HELP: Id = APP.sub("capsule-help");
const MANAGER_HELP: Id = APP.sub("manager-help");
pub const MANAGER_INSPECT: Id = crate::screens::manager::INSPECT;
const EDITOR_MOUNT_EDIT: Id = crate::screens::editor::ROOT.sub("mount-edit");
const EDITOR_ROLE_EDIT: Id = crate::screens::editor::ROOT.sub("role-edit");
const EDITOR_ROLE_LOAD: Id = crate::screens::editor::ROOT.sub("role-load");
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
const CMD_EDITOR_MOUNTS: ActionKey = ActionKey::application("jackin.editor.mounts");
const CMD_MOUNT_TOGGLE_RO: ActionKey = ActionKey::application("jackin.mount.toggle-ro");
const CMD_MOUNT_CYCLE_ISOLATION: ActionKey = ActionKey::application("jackin.mount.cycle-isolation");
const CMD_EDITOR_ENV: ActionKey = ActionKey::application("jackin.editor.environments");
const CMD_SAVE: ActionKey = ActionKey::application("jackin.save");
const CMD_MANAGER_EXPAND: ActionKey = ActionKey::application("jackin.manager.expand");
const CMD_MANAGER_TOGGLE: ActionKey = ActionKey::application("jackin.manager.toggle");
const CMD_MANAGER_DETAIL: ActionKey = ActionKey::application("jackin.manager.detail");
const CMD_EDITOR_OPEN: ActionKey = ActionKey::application("jackin.editor.open");
const CMD_EDITOR_ROLES: ActionKey = ActionKey::application("jackin.editor.roles");
const CMD_EDITOR_PREFER: ActionKey = ActionKey::application("jackin.editor.prefer");
const CMD_NAV_UP: ActionKey = ActionKey::application("jackin.navigation.up");
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
const CMD_ACCOUNTS_FILTER: ActionKey = ActionKey::application("jackin.accounts.filter");
const CMD_ACCOUNT_HELP: ActionKey = ActionKey::application("jackin.account.help");
const CMD_COCKPIT_LOG: ActionKey = ActionKey::application("jackin.cockpit.build-log");
const CMD_COCKPIT_INFO: ActionKey = ActionKey::application("jackin.cockpit.info");
const CMD_COCKPIT_CANCEL: ActionKey = ActionKey::application("jackin.cockpit.cancel");
const CMD_COCKPIT_DEBUG: ActionKey = ActionKey::application("jackin.cockpit.debug");
const CMD_TAB_RENAME: ActionKey = ActionKey::application("jackin.capsule.tab-rename");
const CMD_TAB_CLOSE: ActionKey = ActionKey::application("jackin.capsule.tab-close");
const CMD_INSPECT_CHANGES: ActionKey = ActionKey::application("jackin.capsule.inspect-changes");
const CMD_COPY_SELECTION: ActionKey = ActionKey::application("jackin.capsule.copy-selection");
const CMD_CAPSULE_PASTE: ActionKey = ActionKey::application("jackin.capsule.paste");
const CMD_CAPSULE_CLEAR_SELECTION: ActionKey =
    ActionKey::application("jackin.capsule.clear-selection");
const CMD_CAPSULE_CLOSE_PANE: ActionKey = ActionKey::application("jackin.capsule.close-pane");
const CMD_CAPSULE_EXPORT_FILE: ActionKey = ActionKey::application("jackin.capsule.export-file");
const CMD_CAPSULE_EXPORT_REVEAL: ActionKey = ActionKey::application("jackin.capsule.export-reveal");
const CMD_CAPSULE_CLEAR_PANE: ActionKey = ActionKey::application("jackin.capsule.clear-pane");
const CMD_CAPSULE_REDRAW: ActionKey = ActionKey::application("jackin.capsule.redraw");
const CMD_CAPSULE_GITHUB: ActionKey = ActionKey::application("jackin.capsule.github");
const CMD_CAPSULE_NEW_TAB_ACCOUNT: ActionKey =
    ActionKey::application("jackin.capsule.new-tab-account");
const CMD_KEYBOARD_SHORTCUTS: ActionKey = ActionKey::application("jackin.keyboard-shortcuts");
const CMD_ABOUT: ActionKey = ActionKey::application("jackin.about");
const CMD_MENU_OPEN: ActionKey = ActionKey::application("jackin.menu.open");
const CMD_CONTAINER_INFO: ActionKey = ActionKey::application("jackin.capsule.container-info");
const CAPSULE_MENU_PREVIOUS: ActionKey = ActionKey::custom("menu.bar.prev.left");
const CAPSULE_MENU_NEXT: ActionKey = ActionKey::custom("menu.bar.next.right");

const CMD_EDIT_WORKSPACE: ActionKey = ActionKey::application("jackin.edit-workspace");
const CMD_LAUNCH: ActionKey = ActionKey::application("jackin.launch");
const CMD_PREWARM: ActionKey = ActionKey::application("jackin.prewarm");
const CMD_DELETE_WORKSPACE: ActionKey = ActionKey::application("jackin.delete-workspace");
const CMD_REFRESH: ActionKey = ActionKey::application("jackin.refresh");
const MANAGER_MENU_BAR: Id = APP.sub("manager-menu-bar");

const MANAGER_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_NEW_WORKSPACE, "New workspace…").chord(Chord::key(KeyCode::Char('n'))),
    MenuItem::new(CMD_EDIT_WORKSPACE, "Edit workspace…").chord(Chord::key(KeyCode::Char('e'))),
    MenuItem::new(CMD_LAUNCH, "Launch…").chord(Chord::key(KeyCode::Enter)),
    MenuItem::new(CMD_PREWARM, "Prewarm").chord(Chord::key(KeyCode::Char('w'))),
    MenuItem::new(CMD_DELETE_WORKSPACE, "Delete workspace…")
        .chord(Chord::key(KeyCode::Char('d')))
        .danger()
        .separator(),
    MenuItem::new(CMD_REFRESH, "Refresh")
        .chord(Chord::key(KeyCode::F(5)))
        .separator(),
    MenuItem::new(CMD_QUIT, "Quit").chord(Chord::with(KeyCode::Char('Q'), KeyModifiers::CONTROL)),
];

const MANAGER_GO_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_MANAGER, "Workspace manager").chord(Chord::key(KeyCode::Esc)),
    MenuItem::new(CMD_ACCOUNTS, "Account & Usage Center").chord(Chord::key(KeyCode::Char('c'))),
    MenuItem::new(CMD_USAGE, "Usage").chord(Chord::key(KeyCode::Char('u'))),
    MenuItem::new(CMD_SETTINGS, "Global settings").chord(Chord::key(KeyCode::Char('s'))),
];

const MANAGER_HELP_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_KEYBOARD_SHORTCUTS, "Key reference").chord(Chord::key(KeyCode::Char('?'))),
    MenuItem::new(CMD_ABOUT, "About jackin-preview"),
];

const MANAGER_MENUS: &[Menu<'static>] = &[
    Menu::new("File", MANAGER_FILE_ITEMS),
    Menu::new("Go", MANAGER_GO_ITEMS),
    Menu::new("Help", MANAGER_HELP_ITEMS),
];

const CMD_ACCOUNT_EDIT: ActionKey = ActionKey::application("jackin.account-edit");
const CMD_ACCOUNT_REFRESH_ALL: ActionKey = ActionKey::application("jackin.account-refresh-all");
const CMD_HOST_SAVE: ActionKey = ActionKey::application("jackin.host-save");
const CMD_HOST_CANCEL: ActionKey = ActionKey::application("jackin.host-cancel");

const ACCOUNTS_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_ACCOUNTS, "Add account…").chord(Chord::key(KeyCode::Char('a'))),
    MenuItem::new(CMD_ACCOUNT_EDIT, "Edit account…").chord(Chord::key(KeyCode::Char('e'))),
    MenuItem::new(CMD_ACCOUNT_VALIDATE, "Validate")
        .chord(Chord::key(KeyCode::Char('v')))
        .separator(),
    MenuItem::new(CMD_ACCOUNT_REFRESH, "Refresh selection").chord(Chord::key(KeyCode::Char('r'))),
    MenuItem::new(CMD_ACCOUNT_REFRESH_ALL, "Refresh all").chord(Chord::key(KeyCode::F(5))),
    MenuItem::new(CMD_ACCOUNTS_FILTER, "Filter…").chord(Chord::key(KeyCode::Char('/'))),
];

const EDITOR_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_HOST_SAVE, "Save…")
        .chord(Chord::with(KeyCode::Char('S'), KeyModifiers::CONTROL))
        .separator(),
    MenuItem::new(CMD_HOST_CANCEL, "Cancel").chord(Chord::key(KeyCode::Esc)),
];

const USAGE_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_REFRESH, "Refresh").chord(Chord::key(KeyCode::Char('r'))),
    MenuItem::new(CMD_ACCOUNTS, "Manage in Accounts").chord(Chord::key(KeyCode::Char('m'))),
];

const FALLBACK_FILE_ITEMS: &[MenuItem<'static>] =
    &[MenuItem::new(CMD_HOST_CANCEL, "Cancel").chord(Chord::key(KeyCode::Esc))];

const ACCOUNTS_MENUS: &[Menu<'static>] = &[
    Menu::new("File", ACCOUNTS_FILE_ITEMS),
    Menu::new("Go", MANAGER_GO_ITEMS),
    Menu::new("Help", MANAGER_HELP_ITEMS),
];

const EDITOR_MENUS: &[Menu<'static>] = &[
    Menu::new("File", EDITOR_FILE_ITEMS),
    Menu::new("Go", MANAGER_GO_ITEMS),
    Menu::new("Help", MANAGER_HELP_ITEMS),
];

const USAGE_MENUS: &[Menu<'static>] = &[
    Menu::new("File", USAGE_FILE_ITEMS),
    Menu::new("Go", MANAGER_GO_ITEMS),
    Menu::new("Help", MANAGER_HELP_ITEMS),
];

const FALLBACK_MENUS: &[Menu<'static>] = &[
    Menu::new("File", FALLBACK_FILE_ITEMS),
    Menu::new("Go", MANAGER_GO_ITEMS),
    Menu::new("Help", MANAGER_HELP_ITEMS),
];

const CAPSULE_FILE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_NEW_TAB, "New tab").shortcut_text("Ctrl+B c"),
    MenuItem::new(CMD_CAPSULE_SPLIT_RIGHT, "Split right").shortcut_text("Ctrl+B %"),
    MenuItem::new(CMD_CAPSULE_SPLIT_BELOW, "Split below")
        .shortcut_text("Ctrl+B \"")
        .separator(),
    MenuItem::new(CMD_CAPSULE_EXPORT_FILE, "Export selected file…"),
    MenuItem::new(
        CMD_CAPSULE_EXPORT_REVEAL,
        "Export selected file and reveal…",
    )
    .separator(),
    MenuItem::new(CMD_CAPSULE_CLOSE_PANE, "Close pane").shortcut_text("Ctrl+B x"),
];
const CAPSULE_EDIT_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_COPY_SELECTION, "Copy selection").shortcut_text("y"),
    MenuItem::new(CMD_CAPSULE_PASTE, "Paste clipboard"),
    MenuItem::new(CMD_CAPSULE_CLEAR_SELECTION, "Clear selection")
        .shortcut_text("Esc")
        .separator(),
    MenuItem::new(CMD_CAPSULE_CLEAR_PANE, "Clear pane").shortcut_text("Ctrl+B Ctrl+L"),
];
const CAPSULE_VIEW_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_ZOOM, "Zoom pane").shortcut_text("Ctrl+B z"),
    MenuItem::new(CMD_CAPSULE_REDRAW, "Redraw")
        .shortcut_text("Ctrl+B r")
        .separator(),
    MenuItem::new(CMD_USAGE, "Usage").shortcut_text("Ctrl+B u"),
    MenuItem::new(CMD_CONTAINER_INFO, "Container info").shortcut_text("Ctrl+B i"),
    MenuItem::new(CMD_CAPSULE_GITHUB, "GitHub context"),
    MenuItem::new(CMD_INSPECT_CHANGES, "Inspect changes"),
];
const CAPSULE_SESSION_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_CAPSULE_NEW_TAB_ACCOUNT, "New tab with account…"),
    MenuItem::new(CMD_TAB_RENAME, "Change tab title…")
        .shortcut_text("Ctrl+B ,")
        .separator(),
    MenuItem::new(CMD_CAPSULE_DETACH, "Detach").shortcut_text("Ctrl+B d"),
    MenuItem::new(CMD_TAB_CLOSE, "Close tab")
        .shortcut_text("Ctrl+B &")
        .danger()
        .separator(),
    MenuItem::new(CMD_EXIT_DIALOG, "Exit")
        .shortcut_text("Ctrl+Q")
        .danger(),
];
const CAPSULE_HELP_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_KEYBOARD_SHORTCUTS, "Key reference").shortcut_text("?"),
    MenuItem::new(CMD_CAPSULE_PALETTE, "Command palette").shortcut_text("Ctrl+\\"),
];
const CAPSULE_MENUS: &[Menu<'static>] = &[
    Menu::new("File", CAPSULE_FILE_ITEMS),
    Menu::new("Edit", CAPSULE_EDIT_ITEMS),
    Menu::new("View", CAPSULE_VIEW_ITEMS),
    Menu::new("Session", CAPSULE_SESSION_ITEMS),
    Menu::new("Help", CAPSULE_HELP_ITEMS),
];
const CAPSULE_TAB_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_TAB_RENAME, "Change title…").shortcut_text("2×click"),
    MenuItem::new(CMD_CAPSULE_SPLIT_RIGHT, "Split right")
        .shortcut_text("Ctrl+B %")
        .separator(),
    MenuItem::new(CMD_TAB_CLOSE, "Close tab")
        .shortcut_text("Ctrl+B &")
        .danger(),
];
const CAPSULE_PANE_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_COPY_SELECTION, "Copy selection").shortcut_text("y"),
    MenuItem::new(CMD_CAPSULE_PASTE, "Paste clipboard"),
    MenuItem::new(CMD_CAPSULE_CLEAR_SELECTION, "Clear selection").separator(),
    MenuItem::new(CMD_CAPSULE_SPLIT_RIGHT, "Split right").shortcut_text("Ctrl+B %"),
    MenuItem::new(CMD_CAPSULE_SPLIT_BELOW, "Split below").shortcut_text("Ctrl+B \""),
    MenuItem::new(CMD_CAPSULE_ZOOM, "Zoom pane")
        .shortcut_text("Ctrl+B z")
        .separator(),
    MenuItem::new(CMD_CAPSULE_CLOSE_PANE, "Close pane")
        .shortcut_text("Ctrl+B x")
        .danger(),
];
const CAPSULE_BRAND_ITEMS: &[MenuItem<'static>] = &[
    MenuItem::new(CMD_ABOUT, "About jackin-preview"),
    MenuItem::new(CMD_USAGE, "Usage")
        .shortcut_text("Ctrl+B u")
        .separator(),
    MenuItem::new(CMD_CAPSULE_DETACH, "Workspace manager").shortcut_text("Ctrl+B d"),
];
const CAPSULE_COMMANDS: &[Item<'static>] = &[
    Item::new(ItemKey::num(1), "New tab")
        .glyph(" ")
        .detail("Ctrl+B c"),
    Item::new(ItemKey::num(2), "Split pane")
        .glyph(" ")
        .detail("Ctrl+B \" %"),
    Item::new(ItemKey::num(3), "Zoom / unzoom pane")
        .glyph(" ")
        .detail("Ctrl+B z"),
    Item::new(ItemKey::num(4), "Export file")
        .glyph(" ")
        .detail("host"),
    Item::new(ItemKey::num(5), "Export file and reveal")
        .glyph(" ")
        .detail("host"),
    Item::new(ItemKey::num(6), "Export file and open")
        .glyph(" ")
        .detail("host"),
    Item::new(ItemKey::num(7), "Export file under cursor")
        .glyph(" ")
        .detail("no file under cursor")
        .disabled(true),
    Item::new(ItemKey::num(8), "Export file under cursor and reveal")
        .glyph(" ")
        .detail("no file under cursor")
        .disabled(true),
    Item::new(ItemKey::num(9), "Export file under cursor and open")
        .glyph(" ")
        .detail("no file under cursor")
        .disabled(true),
    Item::new(ItemKey::num(10), "Export selected file")
        .glyph(" ")
        .detail("no selection")
        .disabled(true),
    Item::new(ItemKey::num(11), "Export selected file and reveal")
        .glyph(" ")
        .detail("no selection")
        .disabled(true),
    Item::new(ItemKey::num(12), "Export selected file and open")
        .glyph(" ")
        .detail("no selection")
        .disabled(true),
    Item::new(ItemKey::num(13), "Stage image from clipboard path")
        .glyph(" ")
        .detail("host"),
    Item::new(ItemKey::num(14), "Paste image from host clipboard")
        .glyph(" ")
        .detail("host"),
    Item::new(ItemKey::num(15), "Stage image without pasting")
        .glyph(" ")
        .detail("host"),
    Item::new(ItemKey::num(16), "Open link under cursor")
        .glyph(" ")
        .detail("no link under cursor")
        .disabled(true),
    Item::new(ItemKey::num(17), "Clear pane")
        .glyph(" ")
        .detail("Ctrl+B Ctrl+L"),
    Item::new(ItemKey::num(18), "Usage")
        .glyph(" ")
        .detail("Ctrl+B u"),
    Item::new(ItemKey::num(19), "Close")
        .glyph(" ")
        .detail("Ctrl+B x"),
    Item::new(ItemKey::num(20), "Exit")
        .glyph(" ")
        .detail("Ctrl+Q"),
];
const CAPSULE_AGENT_OPTIONS: &[Item<'static>] = &[
    Item::new(ItemKey::num(1), "Claude Code")
        .glyph("▪")
        .detail("2 accounts · choose at start")
        .group("agents"),
    Item::new(ItemKey::num(2), "Codex")
        .glyph("▪")
        .detail("Primary"),
    Item::new(ItemKey::num(3), "OpenCode")
        .glyph("▪")
        .detail("Go subscription"),
    Item::new(ItemKey::num(4), "Grok Build")
        .glyph("▪")
        .detail("Team"),
    Item::new(ItemKey::num(5), "Shell")
        .glyph("$")
        .detail("zsh")
        .group("shells"),
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
    #[allow(dead_code)]
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
    OnePassword,
    Capsule,
}

/// Which context the shared capsule context-menu layer currently voices:
/// the titled tab menu, the untitled pane Edit menu, or the untitled
/// brand menu (tag `capsule.rs:2073,288,303`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum CapsuleCtxKind {
    #[default]
    Tab,
    Pane,
    Brand,
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
    tag: String,
    agent: Agent,
    account: Option<String>,
    blocked: bool,
}

impl AgentOption {
    fn from_candidate(
        candidate: LaunchCandidate,
        world: &World,
        workspace: Option<&Workspace>,
        role: Option<&str>,
    ) -> Self {
        let offer = world.offer_for(candidate.agent, workspace, role);
        let tag = if candidate.blocked.is_some() || offer.blocked.is_some() {
            "blocked".to_owned()
        } else if offer.accounts.len() > 1 {
            "choose at start".to_owned()
        } else {
            "ready".to_owned()
        };
        let resolved = world.account_for(candidate.agent.provider(), workspace, role, None);
        let account_title = resolved.label(&world.accounts);
        let level_label = resolved.level_label();
        let detail = if let Some(reason) = &candidate.blocked {
            format!("blocked · {reason}")
        } else {
            format!("account {account_title} · {level_label}")
        };
        Self {
            key: candidate.agent.short().to_owned(),
            label: candidate.agent.label().to_owned(),
            detail,
            tag,
            agent: candidate.agent,
            account: candidate.account,
            blocked: candidate.blocked.is_some(),
        }
    }
}

impl AsItem for AgentOption {
    fn as_item(&self) -> Item<'_> {
        Item::new(ItemKey::text(&self.key), &self.label)
            .glyph("▪")
            .detail(&self.detail)
            .tag(&self.tag)
            .group("agents")
    }
}

/// One capsule tab strip item: the frozen strip styles the number prefix,
/// the label, and the state glyph independently, so the row painter needs
/// the three runs separately instead of one joined string.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CapsuleTab {
    /// 1-based tab number.
    number: String,
    /// Tab label without number or state glyph.
    label: String,
    /// State glyph, or empty when the tab carries none.
    glyph: &'static str,
}

/// Muted, never bold: tab number prefixes and inactive state glyphs.
const CAPSULE_TAB_MUTED_PATCH: StylePatch = StylePatch::new()
    .set_fg(Role::Fg(FgStep::Muted))
    .remove(Modifier::BOLD);

/// Secondary, never bold: the active tab's state glyph.
const CAPSULE_TAB_GLYPH_PATCH: StylePatch = StylePatch::new()
    .set_fg(Role::Fg(FgStep::Secondary))
    .remove(Modifier::BOLD);

/// Paint one capsule tab through the Tabs content slot: muted number, label
/// in the strip's label style, state glyph muted (secondary while active).
fn paint_capsule_tab(tab: &CapsuleTab, row: &mut RowUi<'_>) {
    row.label_patched(tab.number.as_str(), &CAPSULE_TAB_MUTED_PATCH);
    row.label(" ");
    row.label(tab.label.as_str());
    if !tab.glyph.is_empty() {
        row.label(" ");
        let patch = if row.flags().contains(StateFlags::ACTIVE) {
            &CAPSULE_TAB_GLYPH_PATCH
        } else {
            &CAPSULE_TAB_MUTED_PATCH
        };
        row.label_patched(tab.glyph, patch);
    }
}

/// Shared draw context for one pane tree: the daemon and tab stay fixed
/// while the node recursion walks splits and leaves.
struct CapsulePaneCtx<'a> {
    daemon: &'a Daemon,
    tab: &'a Tab,
    framed: bool,
}

/// Cached capsule per-frame projections: scrollback text, tab labels, pane
/// titles and pane geometry. Rebuilt only when the fingerprinted inputs
/// change, so steady-state draws borrow instead of allocating.
///
/// The cache lives behind [`RefCell`] because draws are `&self`: both update
/// and draw phases ensure entries on demand. Fingerprints hash every byte the
/// corresponding builder reads, so a hit always matches a fresh build (up to
/// a 64-bit hash collision, which self-heals on the next input change).
#[derive(Debug, Clone, Default)]
struct CapsuleFrameCaches {
    /// Scrollback projection per pane, valid while the term revision matches.
    transcripts: BTreeMap<PaneId, (u64, ProjectedText)>,
    /// Tab strip items with the fingerprint of everything they read.
    tabs: Option<(u64, Vec<CapsuleTab>)>,
    /// Framed pane titles with per-pane fingerprints: the label run and the
    /// state glyph paint in different styles, so both are cached separately.
    titles: BTreeMap<PaneId, (u64, (String, &'static str))>,
}

/// Static hint-bar content, built once per [`App`] so draws borrow instead of
/// allocating. Process-global statics are banned (rule 18); ownership here
/// keeps every hint layer an ordinary App field.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HintLayerSet {
    help: HintLayer,
    default: HintLayer,
    dialog: HintLayer,
    capsule_help: HintLayer,
    capsule_menu: HintLayer,
    capsule_prefix: HintLayer,
    capsule_inspect: HintLayer,
    capsule_default: HintLayer,
    prelude: HintLayer,
    manager_menu: HintLayer,
    manager_inspect: HintLayer,
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

    fn layer_centered(hints: Vec<Hint>) -> HintLayer {
        HintLayer {
            hints,
            badge: None,
            status: None,
            centered: true,
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
            prelude: Self::layer_centered(vec![
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Enter)), "Open", 100),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Char(' '))), "Choose", 90),
                Self::hint(
                    HintKey::Chord(Chord::key(KeyCode::Char('g'))),
                    "Git URL",
                    80,
                ),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Tab)), "Next", 70),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Cancel", 60),
            ]),
            help: Self::layer_centered(vec![
                Self::hint(HintKey::Label("↑↓"), "Scroll", 100),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Close", 90),
            ]),
            default: Self::layer(vec![Self::hint(
                HintKey::Chord(Chord::key(KeyCode::Enter)),
                "Choose",
                90,
            )]),
            dialog: Self::layer_centered(vec![
                Self::hint(HintKey::Label("← →"), "Choose", 100),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Enter)), "Confirm", 90),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Cancel", 80),
                Self::hint(HintKey::Label("y / n"), "Quick answer", 70),
            ]),
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
            manager_menu: Self::layer_centered(vec![
                Self::hint(HintKey::Label("← →"), "Menu", 100),
                Self::hint(HintKey::Label("↑↓"), "Move", 90),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Enter)), "Choose", 80),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Close", 70),
            ]),
            manager_inspect: Self::layer_centered(vec![
                Self::hint(HintKey::Label("↑↓"), "Move", 100),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Char('y'))), "Copy", 90),
                Self::hint(HintKey::Chord(Chord::key(KeyCode::Esc)), "Close", 80),
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
            capsule_default: Self::layer(vec![
                Self::hint(HintKey::Label("Ctrl+B"), "Prefix", 100),
                Self::hint(HintKey::Label("F10"), "Menu", 90),
                Self::hint(HintKey::Label("Ctrl+\\"), "Palette", 80),
                Self::hint(HintKey::Label("Alt+Shift+↑↓←→"), "Resize", 70),
                Self::hint(HintKey::Label("right-click"), "Tab menu", 60),
                Self::hint(HintKey::Label("Ctrl+Q"), "Quit", 50),
            ]),
        }
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
    /// Workspace creation UI state (mount read-only flag).
    pub prelude_ui: PreludeUiState,
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
    manager_menu_state: MenuState,
    container_info_state: DialogState,
    container_info_props: PropsState,
    capsule_tab_menu_state: MenuState,
    capsule_tab_menu_pos: Position,
    capsule_tab_menu_open: bool,
    capsule_tab_menu_kind: CapsuleCtxKind,
    capsule_new_tab_open: bool,
    capsule_split_vertical_open: bool,
    capsule_palette_open: bool,
    capsule_palette_state: PickerState,
    capsule_agent_state: PickerState,
    tabs_state: TabsState,
    quit_dialog: DialogState,
    launch_dialog: DialogState,
    preview_dialog: DialogState,
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
    manager_help_state: HelpOverlayState,
    capsule_help_open: bool,
    capsule_help_state: HelpOverlayState,
    inspect_detail: bool,
    inspect_files: bool,
    active_instance: Option<String>,
    launch_agent: Agent,
    launch_account: Option<String>,
    manager_menu_open: bool,
    manager_inspect_open: bool,
    manager_quit_confirm: bool,
    cockpit_info_open: bool,
    cockpit_info_props: PropsState,
    cockpit_info_focus_pending: bool,
    about_open: bool,
    about_props: PropsState,
    cockpit_cancel_confirm: bool,
    cancel_dialog: DialogState,
    cockpit_debug_open: bool,
    cockpit_failure_open: bool,
    accounts_form_stage: u8,
    accounts_form_enters: u8,
    accounts_filtering: bool,
    accounts_filter_enters: u8,
    accounts_filtered: bool,
    accounts_drawer_open: bool,
    accounts_down_count: usize,
    settings_tab: usize,
    settings_save_preview: bool,
    settings_save_dialog: DialogState,
    usage_detail: bool,
}

impl App {
    // Role identity stays qualified; display comes from the matching catalog entry.
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
            Scenario::FirstUse if frame >= INTRO_END => {
                world.arbiter.complete_entry(world.now_ms());
                Route::Manager
            }
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
            prelude_ui: PreludeUiState::default(),
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
            manager_menu_state: MenuState::default(),
            container_info_state: DialogState::default(),
            container_info_props: PropsState::default(),
            capsule_tab_menu_state: MenuState::default(),
            capsule_tab_menu_pos: Position::new(0, 0),
            capsule_tab_menu_open: false,
            capsule_tab_menu_kind: CapsuleCtxKind::Tab,
            capsule_new_tab_open: false,
            capsule_split_vertical_open: false,
            capsule_palette_open: false,
            capsule_palette_state: PickerState::default(),
            capsule_agent_state: PickerState::default(),
            tabs_state: TabsState::default(),
            quit_dialog: DialogState::default(),
            launch_dialog: DialogState::default(),
            preview_dialog: DialogState::default(),
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
            manager_help_state: HelpOverlayState::default(),
            capsule_help_open: false,
            capsule_help_state: HelpOverlayState::default(),
            inspect_detail: false,
            inspect_files: false,
            active_instance: None,
            launch_agent: Agent::ClaudeCode,
            launch_account: None,
            manager_menu_open: false,
            manager_inspect_open: false,
            manager_quit_confirm: false,
            cockpit_info_open: false,
            cockpit_info_props: PropsState::default(),
            cockpit_info_focus_pending: false,
            about_open: false,
            about_props: PropsState::default(),
            cockpit_cancel_confirm: false,
            cancel_dialog: DialogState::default(),
            cockpit_debug_open: false,
            cockpit_failure_open: false,
            accounts_form_stage: 0,
            accounts_form_enters: 0,
            accounts_filtering: false,
            accounts_filter_enters: 0,
            accounts_filtered: false,
            accounts_drawer_open: false,
            accounts_down_count: 0,
            settings_tab: 1,
            settings_save_preview: false,
            settings_save_dialog: DialogState::default(),
            usage_detail: false,
        };
        if app.launch.as_ref().is_some_and(|run| run.done) {
            app.materialize_launch();
        }
        if app.launch.as_ref().is_some_and(|run| run.failure.is_some()) {
            app.present_boot_failure();
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
            let name = app
                .active_instance
                .as_ref()
                .and_then(|id| app.world.daemons.get(id))
                .map(|d| d.workspace.clone())
                .unwrap_or_default();
            app.status = Some(format!("Attached to {name} · tabs and panes restored"));
            app.sync_capsule_projection();
        }
        if app.route == Route::Manager {
            app.reset_manager_cursor();
            // Tag boot: an unreadable instance index enters the manager
            // with the ritual warning (arbiter `Unknown` entry decision).
            if let Err(err) = app.world.arbiter.running() {
                app.status = Some(format!(
                    "Could not confirm running instances: {} · entered without the ritual",
                    err.label()
                ));
            }
        }
        app.sync_workspace_keymap();
        app
    }

    /// Whether this route is one of the host management screens.
    pub const fn is_host(route: Route) -> bool {
        matches!(
            route,
            Route::Manager
                | Route::Accounts
                | Route::Usage
                | Route::Settings
                | Route::Editor
                | Route::Prelude
        )
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

    /// The current launch-picker scope label: the selected workspace, the
    /// current directory's workspace, or the directory itself, plus the
    /// short current-role name (tag `open_launch_picker` scope).
    fn manager_picker_meta(&self) -> String {
        let ws_name = self
            .manager
            .selected()
            .and_then(|id| self.world.workspace(id))
            .map(|ws| ws.name.clone())
            .or_else(|| self.world.cwd_workspace().map(|ws| ws.name.clone()))
            .unwrap_or_else(|| self.world.tilde(&self.world.cwd));
        let role = self.selected_role();
        let short = role.rsplit('/').next().unwrap_or(role);
        let role_label = match short {
            "developer" => "Developer",
            "reviewer" => "Reviewer",
            "qa" => "QA",
            other => other,
        };
        format!("{ws_name} › {role_label}")
    }

    fn manager_launch_picker_open(&self) -> bool {
        self.route == Route::Manager
            && (!self.agent_options.is_empty()
                || self.status.as_deref() == Some("Launch · choose Agent"))
    }

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
        let n_chord = Chord::key(KeyCode::Char('n'));
        let tab_chord = Chord::key(KeyCode::Tab);
        let i_chord = Chord::key(KeyCode::Char('i'));
        let c_chord = Chord::key(KeyCode::Char('c'));
        let d_chord = Chord::key(KeyCode::Char('d'));
        let r_chord = Chord::key(KeyCode::Char('r'));
        self.keymap.remove(KeyPhase::Capture, chord);
        self.keymap.remove(KeyPhase::Capture, n_chord);
        self.keymap.remove(KeyPhase::Bubble, n_chord);
        self.keymap.remove(KeyPhase::Capture, tab_chord);
        self.keymap.remove(KeyPhase::Capture, i_chord);
        self.keymap.remove(KeyPhase::Capture, c_chord);
        self.keymap.remove(KeyPhase::Capture, d_chord);
        self.keymap.remove(KeyPhase::Capture, r_chord);
        if self.route == Route::Manager
            || (self.route == Route::Editor
                && self.editor.tab == EditorTab::Environments
                && !self.editor.env_form_open)
        {
            self.keymap.add(KeyPhase::Capture, chord, CMD_NEW_WORKSPACE);
            self.keymap
                .add(KeyPhase::Capture, n_chord, CMD_NEW_WORKSPACE);
            self.keymap
                .add(KeyPhase::Bubble, n_chord, CMD_NEW_WORKSPACE);
        }
        if self.route == Route::Manager {
            self.keymap
                .add(KeyPhase::Capture, tab_chord, CMD_MANAGER_DETAIL);
        }
        if self.route == Route::Editor && self.editor.tab == EditorTab::Mounts {
            self.keymap
                .add(KeyPhase::Capture, r_chord, CMD_MOUNT_TOGGLE_RO);
            self.keymap
                .add(KeyPhase::Capture, i_chord, CMD_MOUNT_CYCLE_ISOLATION);
        }
        if matches!(self.route, Route::Cockpit | Route::Launch) {
            self.keymap
                .add(KeyPhase::Capture, i_chord, CMD_COCKPIT_INFO);
            self.keymap
                .add(KeyPhase::Capture, c_chord, CMD_COCKPIT_CANCEL);
            self.keymap
                .add(KeyPhase::Capture, d_chord, CMD_COCKPIT_DEBUG);
        }
    }

    fn route_changed(&mut self) -> Response<()> {
        self.sync_workspace_keymap();
        Response::changed()
    }

    fn launch_candidates(&self) -> Vec<AgentOption> {
        let ws_id = self
            .manager
            .selected()
            .or_else(|| self.world.workspaces.first().map(|workspace| workspace.id));
        let workspace = ws_id.and_then(|id| self.world.workspaces.iter().find(|w| w.id == id));
        let role = Some(self.selected_role());
        ManagerState::launch_candidates(&self.world, ws_id, role)
            .into_iter()
            .map(|candidate| AgentOption::from_candidate(candidate, &self.world, workspace, role))
            .collect()
    }

    /// Whether the launch affordance has any candidate, without building them.
    ///
    /// Exactly `!self.launch_candidates().is_empty()`: candidate discovery
    /// maps offers 1:1 and the role never affects offer configuredness (see
    /// [`World::has_offered_agents`]). Per-frame update/draw must call this;
    /// only the agent picker needs the full [`Self::launch_candidates`] list.
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

    /// Borrow the active running instance id without allocating. Hot
    /// draw paths must use this instead of the owned
    /// [`Self::active_running_instance_id`].
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

    fn launch_button(disabled: bool) -> Button<'static> {
        Button::new(LAUNCH, "Launch session")
            .variant(Variant::PRIMARY)
            .disabled(disabled)
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
        Picker::new(ROLE_PICKER)
            .title(title)
            .item_layout(ItemRowLayout::Compact)
    }

    fn launch_agent_picker() -> Picker<'static, AgentOption> {
        Picker::new(crate::screens::manager::AGENT_PICKER)
            .title("Launch · choose Agent")
            .size(LayerSize::Fixed(84, 9))
            .searchable(false)
            .item_layout(ItemRowLayout::Columns)
    }

    fn account_picker() -> Picker<'static, AccountOption> {
        Picker::new(ACCOUNT_PICKER)
            .title("Choose a configured account")
            .item_layout(ItemRowLayout::Compact)
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

    /// Dim patch for the prelude host-menu titles (faint on canvas, never bold).
    const PRELUDE_MENU_DIM: [(Part, StylePatch); 1] = [(
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Faint))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    )];

    /// Dim patch for the prelude brand lockup (faint on overlay, never bold).
    const PRELUDE_BRAND_DIM: [(Part, StylePatch); 1] = [(
        Part::LABEL,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Faint))
            .set_bg(Role::Surface(Surface::Overlay))
            .remove(Modifier::BOLD),
    )];

    /// The host menu bar with its per-route File menu (tag `app.rs:699`):
    /// File is rebuilt per route while Go and Help stay fixed.
    fn host_menu_bar(&self) -> MenuBar<'static> {
        let menus = match self.route {
            Route::Accounts => ACCOUNTS_MENUS,
            Route::Editor | Route::Settings => EDITOR_MENUS,
            Route::Usage => USAGE_MENUS,
            Route::Manager => MANAGER_MENUS,
            _ => FALLBACK_MENUS,
        };
        MenuBar::new(MANAGER_MENU_BAR, menus)
    }

    /// Whether F10 must refuse the host menu: a host form field holds
    /// an in-flight draft (tag `app.rs:592`). Editor and settings routes
    /// voice no editing state yet, so only accounts refuses today.
    fn host_menu_editing(&self) -> bool {
        self.route == Route::Accounts && self.accounts.form_editing()
    }

    fn capsule_menu_bar() -> MenuBar<'static> {
        MenuBar::new(CAPSULE_MENU_BAR, CAPSULE_MENUS)
    }

    fn active_capsule_tab_title(&self) -> String {
        let instance_id = Self::active_running_instance_id_ref(&self.active_instance, &self.world);
        let tabs = Self::build_capsule_tabs(
            &self.world,
            instance_id,
            &self.capsule_tab_title,
            self.capsule_tab_title_index,
        );
        tabs.get(self.capsule_tab_title_index)
            .map(|t| t.label.clone())
            .unwrap_or_else(|| "Tab".to_string())
    }

    fn capsule_tab_context(
        title: impl Into<Cow<'static, str>>,
        position: Position,
    ) -> ContextMenu<'static> {
        Self::capsule_ctx_menu(CapsuleCtxKind::Tab, title, position)
    }

    fn capsule_ctx_menu(
        kind: CapsuleCtxKind,
        title: impl Into<Cow<'static, str>>,
        position: Position,
    ) -> ContextMenu<'static> {
        match kind {
            CapsuleCtxKind::Tab => {
                ContextMenu::at(CAPSULE_TAB_MENU, CAPSULE_TAB_ITEMS, position).title(title)
            }
            CapsuleCtxKind::Pane => ContextMenu::at(CAPSULE_TAB_MENU, CAPSULE_PANE_ITEMS, position),
            CapsuleCtxKind::Brand => {
                ContextMenu::at(CAPSULE_TAB_MENU, CAPSULE_BRAND_ITEMS, position)
            }
        }
    }

    fn capsule_command_palette(screen_height: u16) -> Picker<'static, Item<'static>> {
        let height = 19.min(screen_height.saturating_sub(2));
        Picker::new(CAPSULE_COMMAND_PALETTE)
            .title("Command palette")
            .placeholder("Type to filter commands…")
            .meta("20 of 20")
            .size(LayerSize::Fixed(60, height))
            .searchable(true)
            .item_layout(ItemRowLayout::Columns)
    }

    fn capsule_agent_picker(
        title: &'static str,
        screen_width: u16,
    ) -> Picker<'static, Item<'static>> {
        let width = 72.min(screen_width.saturating_sub(4));
        Picker::new(crate::screens::manager::AGENT_PICKER)
            .title(title)
            .placeholder("Type to search…")
            .size(LayerSize::Fixed(width, 12))
            .searchable(true)
            .item_layout(ItemRowLayout::Columns)
    }

    fn open_capsule_agent_picker(&mut self, cx: &mut Cx<'_>, action: CapsuleAction) {
        self.capsule_agent_state = PickerState::default();
        self.pending_capsule_action = Some(action);
        let title = match action {
            CapsuleAction::Split(_) => "Split ↓ Below",
            _ => "New tab",
        };
        let screen = cx.viewport();
        let picker = Self::capsule_agent_picker(title, screen.width);
        let spec = picker.layer(cx, CAPSULE_AGENT_OPTIONS);
        cx.open_layer(crate::screens::manager::AGENT_PICKER, spec);
        let _ = picker
            .update(cx, &mut self.capsule_agent_state, CAPSULE_AGENT_OPTIONS)
            .erase();
    }

    fn container_info_dialog() -> Dialog<'static> {
        Dialog::info(CAPSULE_CONTAINER_INFO, "Container info").body_rows(9)
    }

    /// Borrowed rows for [`PropsList`]. Labels and copyability follow the
    /// tag info dialog; values stay on the instance record.
    fn container_info_rows(facts: &[ContainerInfoFact]) -> Vec<PropsRow<'_>> {
        facts
            .iter()
            .enumerate()
            .map(|(index, fact)| {
                let row =
                    PropsRow::new(ItemKey::num(index as u64), fact.label, fact.value.as_str());
                if fact.copyable { row.copyable() } else { row }
            })
            .collect()
    }

    fn container_info_facts(&self) -> Vec<ContainerInfoFact> {
        let Some(instance_id) = self.active_running_instance_id() else {
            return vec![ContainerInfoFact::detached()];
        };
        let Some(instance) = self.world.instance(&instance_id) else {
            return vec![ContainerInfoFact::detached()];
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
            ContainerInfoFact::new("Container", instance.container_id(), true),
            ContainerInfoFact::new("Container ID", instance.container_uid(), true),
            ContainerInfoFact::new("Role", instance.role.clone(), false),
            ContainerInfoFact::new("Agent", agent, false),
            ContainerInfoFact::new("Workdir", instance.workdir.clone(), false),
            ContainerInfoFact::new(
                "Instance",
                instance.id.trim_start_matches("jk-").to_owned(),
                false,
            ),
            ContainerInfoFact::new("Capsule", "0.9.2".to_owned(), false),
            ContainerInfoFact::new("Invocation ID", format!("{}-4f11", instance.run_id), true),
            ContainerInfoFact::new(
                "Host log",
                format!("file:///Users/alexey/.jackin/logs/{}.log", instance.run_id),
                false,
            ),
        ]
    }

    fn open_container_info(&mut self, cx: &mut Cx<'_>) {
        self.container_info_state = DialogState::default();
        self.container_info_props = PropsState::default();
        let mut spec = Self::container_info_dialog().layer(cx);
        spec.initial_focus = Some(CONTAINER_INFO_PROPS);
        cx.open_layer(CAPSULE_CONTAINER_INFO, spec);
        let facts = self.container_info_facts();
        let rows = Self::container_info_rows(&facts);
        if let Some(row) = rows.first() {
            self.container_info_props.set_cursor(0, row.key);
        }
        let _ = PropsList::new(CONTAINER_INFO_PROPS)
            .update(cx, &mut self.container_info_props, &rows)
            .erase();
        self.status = Some("Container info".into());
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
        rows.push(ManagerRow::new(
            ManagerRowKey::CurrentDirectory,
            "Current directory".to_string(),
        ));
        for workspace in &self.world.workspaces {
            let expanded = self.manager.is_expanded(workspace.id);
            let count = self.manager_instances(Some(workspace.id)).count();
            let marker = if count == 0 {
                " "
            } else if expanded {
                "▾"
            } else {
                "▸"
            };
            rows.push(ManagerRow::new(
                ManagerRowKey::Workspace(workspace.id),
                format!("{marker} {}", workspace.name),
            ));
            if expanded {
                for instance in self.manager_instances(Some(workspace.id)) {
                    rows.push(ManagerRow::new(
                        ManagerRowKey::Instance(instance.id.clone()),
                        format!(
                            "  {}  {} · {}",
                            instance.id.trim_start_matches("jk-"),
                            instance.role,
                            instance.agent.label()
                        ),
                    ));
                }
            }
        }
        rows.push(ManagerRow::new(
            ManagerRowKey::NewWorkspace,
            "+ New workspace".to_string(),
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

    fn accounts_tree_rows(&self) -> Vec<crate::screens::accounts::AccountRow> {
        crate::screens::accounts::build_account_rows(
            &self.world,
            self.accounts.filter.as_deref(),
            &self.accounts.folded,
        )
    }

    /// Repair the tree selection after the rows change: a filtered-out
    /// account falls back to its provider, anything else to Overview.
    fn ensure_accounts_selected(&mut self) {
        let rows = self.accounts_tree_rows();
        if rows.iter().any(|row| row.sel == self.accounts.selected) {
            return;
        }
        self.accounts.selected = match &self.accounts.selected {
            AccountSel::Account(id) => self
                .world
                .accounts
                .get(id)
                .map(|account| AccountSel::Provider(account.surface))
                .unwrap_or(AccountSel::Overview),
            _ => AccountSel::Overview,
        };
        if !rows.iter().any(|row| row.sel == self.accounts.selected) {
            self.accounts.selected = AccountSel::Overview;
        }
        self.accounts.sync_selected_id();
    }

    /// Move the tree cursor by `delta` rows, clamped into the live rows.
    fn move_accounts_cursor(&mut self, delta: isize) {
        self.ensure_accounts_selected();
        let rows = self.accounts_tree_rows();
        if rows.is_empty() {
            return;
        }
        let current = rows
            .iter()
            .position(|row| row.sel == self.accounts.selected)
            .unwrap_or(0) as isize;
        let next = current
            .saturating_add(delta)
            .clamp(0, rows.len() as isize - 1) as usize;
        self.accounts.selected = rows[next].sel.clone();
        self.accounts.list.set_cursor(next, ItemKey::index(next));
        self.accounts.sync_selected_id();
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

    const QUIT_ACTIONS: [Action<'static>; 2] = [
        Action::quiet(ActionKey::CANCEL, "Cancel"),
        Action::new(ActionKey::CONFIRM, "Quit"),
    ];

    const PREVIEW_ACTIONS: [Action<'static>; 2] = [
        Action::secondary(ActionKey::CANCEL, "Cancel"),
        Action::primary(ActionKey::CONFIRM, "Save"),
    ];

    /// The save-preview facts dialog over caller-owned slices. No primary
    /// key: tag facts dialogs focus Cancel while Save stays primary
    /// (`Action::primary` styles without moving initial focus).
    fn preview_dialog<'a>(
        title: &'a str,
        pairs: &'a [(&'a str, &'a str)],
        codes: &'a [&'a str],
    ) -> Dialog<'a> {
        Dialog::facts(crate::screens::editor::PREVIEW, title, pairs)
            .code(codes)
            .width(66)
            .actions(&Self::PREVIEW_ACTIONS)
            .cancel(ActionKey::CANCEL)
    }

    fn manager_quit_dialog(&self) -> Dialog<'static> {
        let n = self.world.running_count();
        let body: &'static str = match n {
            2 => "2 instances keep running in the Construct. The host console closes; reconnect from a new terminal.",
            1 => "1 instance keeps running in the Construct. The host console closes; reconnect from a new terminal.",
            0 => "No instances are running. The pending Construct entry is released.",
            _ => Box::leak(
                format!("{n} instances keep running in the Construct. The host console closes; reconnect from a new terminal.")
                    .into_boxed_str(),
            ),
        };
        Dialog::confirm(QUIT_DIALOG, "Exit jackin❯?", body).actions(&Self::QUIT_ACTIONS)
    }

    fn launch_dialog() -> Dialog<'static> {
        Dialog::confirm(
            LAUNCH_DIALOG,
            "Launch a new session",
            "Review the role and start a deterministic Construct run.",
        )
        .body_rows(1)
    }

    const CANCEL_ACTIONS: [Action<'static>; 2] = [
        Action::new(ActionKey::CANCEL, "Cancel"),
        Action::danger(ActionKey::CONFIRM, "Cancel launch"),
    ];

    fn cockpit_cancel_dialog() -> Dialog<'static> {
        // Destructive: confirm stays danger text on the overlay. `confirm()`
        // would force that key to a filled primary button.
        Dialog::destructive(
            CANCEL_DIALOG,
            "Cancel the launch?",
            "The pipeline stops at its current stage and the partially prepared instance is marked failed setup. Nothing is attached.",
        )
        .actions(&Self::CANCEL_ACTIONS)
    }

    /// About dialog rows (tag `app.rs:815`).
    fn about_rows(scenario: &str) -> Vec<PropsRow<'_>> {
        vec![
            PropsRow::new(
                ItemKey::num(0),
                "Product",
                "jackin-preview · deterministic redesign preview",
            ),
            PropsRow::new(ItemKey::num(1), "Scenario", scenario),
            PropsRow::new(
                ItemKey::num(2),
                "Construct",
                "simulated · nothing here touches a real container",
            ),
            PropsRow::new(
                ItemKey::num(3),
                "Design system",
                "Junie-inspired Ratatui components (junie_tui)",
            ),
        ]
    }

    fn update_about(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.about_open {
            return Response::ignored();
        }
        for intent in cx.intents(ABOUT_DIALOG) {
            if matches!(intent, Intent::Layer(LayerEvent::Dismissed(_))) {
                self.about_open = false;
                self.status = None;
                return Response::changed();
            }
        }
        if !cx.is_open(ABOUT_DIALOG) {
            self.about_open = false;
            self.status = None;
            return Response::changed();
        }
        let rows = Self::about_rows(self.world.scenario.name());
        let response = PropsList::new(ABOUT_PROPS).update(cx, &mut self.about_props, &rows);
        let mut result = response.erase();
        let close = Button::new(ABOUT_CLOSE, "Close").update(cx);
        let closed = close.activated();
        result |= close.erase();
        if closed {
            cx.close_layer(ABOUT_DIALOG, Some(ActionKey::CLOSE));
            self.about_open = false;
            self.status = None;
            result |= Response::changed();
        }
        result
    }

    fn draw_about_dialog(&self, ui: &mut Ui<'_>, area: Rect) {
        Panel::new(ABOUT_DIALOG.sub("frame"))
            .kind(PanelKind::Framed)
            .title("About")
            .draw(ui, area, |ui, body| {
                let rows = Self::about_rows(self.world.scenario.name());
                PropsList::new(ABOUT_PROPS).draw(
                    ui,
                    Rect::new(body.x, body.y, body.width, body.height.saturating_sub(1)),
                    &self.about_props,
                    &rows,
                );
                let close_area = Rect::new(
                    body.right().saturating_sub(9),
                    body.bottom().saturating_sub(1),
                    7,
                    1,
                );
                Button::new(ABOUT_CLOSE, "Close").draw(ui, close_area);
            });
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
        // The layer spec's initial focus already stages AGENT_PICKER; an
        // explicit focus call here would only cancel the provisional
        // focus-layer mechanism. Reconcile the replaced projection before
        // the first draw so the cursor row paints focused (`Picker::
        // reconcile` contract; no update runs between open and draw).
        picker.reconcile(&mut self.agent_state, &self.agent_options);
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

    /// Present a failure the boot seek already reached.
    ///
    /// `LaunchRun::seek` advances the pipeline without emitting events, so a
    /// frozen failure frame (the `launch-failure` scenario) would otherwise
    /// boot onto a running cockpit with no failure visible. The failed-setup
    /// instance record and the open failure dialog match the live `Failed`
    /// event path; Escape acknowledges back to the Manager.
    fn present_boot_failure(&mut self) {
        let Some(run) = self.launch.as_ref() else {
            return;
        };
        if run.failure.is_none() {
            return;
        }
        let run_id = run.run_id;
        if !self
            .world
            .instances
            .iter()
            .any(|instance| instance.run_id == run_id)
        {
            let agent = run.agent;
            let container = run.container.clone();
            let role = self.selected_role().to_owned();
            let workspace = self.world.workspaces.first().cloned();
            let now_secs = self.world.now_secs();
            let mut instance = crate::sim::fixtures::fixture_instance(
                InstanceStatus::FailedSetup,
                run_id,
                now_secs,
                DaemonSnapshot::Unavailable,
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
            self.world.instances.push(instance);
            self.world.sync_arbiter();
            self.manager_rows_cache.clear();
        }
        self.cockpit_failure_open = true;
    }

    /// Acknowledge the launch-failure dialog: tear down the cockpit and
    /// return to the Manager with the failed instance selected.
    fn acknowledge_launch_failure(&mut self) {
        self.cockpit_failure_open = false;
        let failed_id = self.launch.as_ref().and_then(|run| {
            self.world
                .instances
                .iter()
                .find(|instance| instance.run_id == run.run_id)
                .map(|instance| instance.id.clone())
        });
        self.launch = None;
        let running = self.world.running_count();
        if running > 0 {
            self.route = Route::Manager;
            self.ensure_manager_rows();
            if let Some(id) = failed_id
                && let Some(index) = self
                    .manager_rows_cache
                    .iter()
                    .position(|row| row.domain == ManagerRowKey::Instance(id.clone()))
            {
                let key = self.manager_rows_cache[index].key;
                let domain = self.manager_rows_cache[index].domain.clone();
                self.manager.list.set_cursor(index, key);
                self.manager.select_row(domain);
            }
            let noun = if running == 1 {
                "instance"
            } else {
                "instances"
            };
            self.status = Some(format!(
                "Launch failed · {running} {noun} still running in the Construct"
            ));
        } else {
            self.status = Some("Launch failed · the Construct is empty".into());
            self.route = Route::Outro;
            self.outro = Some(OutroState::new(self.motion, None, 0));
        }
    }

    /// Tag-format run label for the failure dialog: `run-{stamp12}-{suffix}`
    /// from the world clock plus the failed instance's id suffix (tag
    /// `CockpitScreen::new`). The typed `RunId` stays the run identity;
    /// this is the display projection the dialog owns.
    fn failure_run_label(&self, run: &LaunchRun) -> String {
        let stamp = crate::domain::clock::Clock::stamp(self.world.now_secs())
            .replace([' ', ':'], "-")
            .replace('-', "");
        let stamp12 = stamp.get(..12).unwrap_or(&stamp);
        let suffix = self
            .world
            .instances
            .iter()
            .find(|instance| instance.run_id == run.run_id)
            .map(|instance| instance.id.trim_start_matches("jk-"))
            .unwrap_or("0000");
        format!("run-{stamp12}-{suffix}")
    }

    fn capsule_input() -> TextInput<'static> {
        // Inline and placeholder-free: the frozen input row is a blank field
        // row. No frozen frame shows a placeholder here.
        TextInput::new(CAPSULE_INPUT).inline(true)
    }

    fn capsule_viewport(pane_id: PaneId) -> TextViewport<'static> {
        TextViewport::new(CAPSULE_PANES.item(ItemKey::num(pane_id))).click_selects_word(true)
    }

    fn role_choose_button(&self) -> Button<'_> {
        Button::new(ROLE_CHOOSE, self.selected_role())
    }

    fn with_manager_help<R>(f: impl FnOnce(&[HelpSection<'_>]) -> R) -> R {
        let workspaces = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("↑↓ j k"),
                    label: "move",
                    priority: 100,
                },
                Hint {
                    key: termrock::HintKey::Label("←→ h l"),
                    label: "collapse / expand",
                    priority: 95,
                },
                Hint {
                    key: termrock::HintKey::Label("Space"),
                    label: "fold workspace",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("Enter"),
                    label: "launch",
                    priority: 85,
                },
                Hint {
                    key: termrock::HintKey::Label("e"),
                    label: "edit workspace",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Label("n"),
                    label: "new workspace",
                    priority: 75,
                },
                Hint {
                    key: termrock::HintKey::Label("d"),
                    label: "delete…",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Label("w"),
                    label: "prewarm",
                    priority: 65,
                },
                Hint {
                    key: termrock::HintKey::Label("o"),
                    label: "open in GitHub",
                    priority: 60,
                },
                Hint {
                    key: termrock::HintKey::Label("* -"),
                    label: "expand / collapse all",
                    priority: 55,
                },
                Hint {
                    key: termrock::HintKey::Label("Tab"),
                    label: "details",
                    priority: 50,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };

        let instances = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("Enter r"),
                    label: "reconnect / restore",
                    priority: 100,
                },
                Hint {
                    key: termrock::HintKey::Label("a"),
                    label: "new session",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("x"),
                    label: "open shell",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Label("i"),
                    label: "inspect container",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Label("t"),
                    label: "stop",
                    priority: 60,
                },
                Hint {
                    key: termrock::HintKey::Label("p"),
                    label: "purge…",
                    priority: 50,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };

        let everywhere = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("Tab Shift+Tab"),
                    label: "next / previous",
                    priority: 100,
                },
                Hint {
                    key: termrock::HintKey::Label("Esc"),
                    label: "back one level",
                    priority: 95,
                },
                Hint {
                    key: termrock::HintKey::Label("u"),
                    label: "Usage overlay",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("c"),
                    label: "Accounts & Usage",
                    priority: 85,
                },
                Hint {
                    key: termrock::HintKey::Label("s"),
                    label: "Settings",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Label("F5"),
                    label: "refresh now",
                    priority: 75,
                },
                Hint {
                    key: termrock::HintKey::Label("?"),
                    label: "this help",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Label("q"),
                    label: "back / quit",
                    priority: 65,
                },
                Hint {
                    key: termrock::HintKey::Label("Ctrl+Q"),
                    label: "quit with confirmation",
                    priority: 60,
                },
                Hint {
                    key: termrock::HintKey::Label("Ctrl+C"),
                    label: "quit immediately",
                    priority: 55,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };

        let editor_settings = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("←→ 1–5 [ ]"),
                    label: "switch tab",
                    priority: 100,
                },
                Hint {
                    key: termrock::HintKey::Label("Ctrl+S"),
                    label: "save (preview first)",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("Space"),
                    label: "toggle / cycle",
                    priority: 80,
                },
                Hint {
                    key: termrock::HintKey::Label("a e d"),
                    label: "add / edit / remove",
                    priority: 70,
                },
                Hint {
                    key: termrock::HintKey::Label("m p s"),
                    label: "mask · 1Password · scope",
                    priority: 60,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };

        let mouse = HintLayer {
            hints: vec![
                Hint {
                    key: termrock::HintKey::Label("click"),
                    label: "select · 2× activates",
                    priority: 100,
                },
                Hint {
                    key: termrock::HintKey::Label("wheel"),
                    label: "scroll under pointer",
                    priority: 90,
                },
                Hint {
                    key: termrock::HintKey::Label("drag"),
                    label: "seam · scrollbar thumb",
                    priority: 80,
                },
            ],
            badge: None,
            status: None,
            centered: false,
        };

        let sections = [
            HelpSection::new("Workspaces", &workspaces),
            HelpSection::new("Instances", &instances),
            HelpSection::new("Everywhere", &everywhere),
            HelpSection::new("Editor and Settings", &editor_settings),
            HelpSection::new("Mouse", &mouse),
        ];
        f(&sections)
    }

    const HELP_PARTS: [(Part, StylePatch); 1] =
        [(Part::BORDER, StylePatch::new().remove(Modifier::BOLD))];

    fn manager_help_overlay<'a>(
        sections: &'a [HelpSection<'a>],
        w: u16,
        h: u16,
    ) -> HelpOverlay<'a> {
        HelpOverlay::new(MANAGER_HELP, "Workspaces", sections)
            .size(w, h)
            .patch_part(&Self::HELP_PARTS)
    }

    fn open_manager_help(&mut self, cx: &mut Cx<'_>) {
        self.help_open = true;
        self.manager_help_state = HelpOverlayState::default();
        let screen = cx.viewport();
        let (w, h) = (
            screen.width.saturating_sub(4),
            screen.height.saturating_sub(3),
        );
        Self::with_manager_help(|sections| {
            let help = Self::manager_help_overlay(sections, w, h);
            let mut spec = help.layer(cx);
            spec.anchor = Anchor::Rect {
                rect: Rect::new(0, 0, screen.width, 1),
                side: Side::Below,
                align: CrossAlign::Center,
            };
            cx.open_layer(MANAGER_HELP, spec);
        });
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
            self.capsule_tab_menu_kind = CapsuleCtxKind::Tab;
            self.capsule_tab_menu_state = MenuState::default();
            self.capsule_tab_menu_pos = Position::new(1, 2);
            let title = self.active_capsule_tab_title();
            cx.open_layer(
                CAPSULE_TAB_MENU,
                Self::capsule_tab_context(title, self.capsule_tab_menu_pos).layer(cx),
            );
            return Response::changed();
        }
        if key == 'c' {
            self.capsule_prefix = false;
            self.capsule_new_tab_open = true;
            self.open_capsule_agent_picker(cx, CapsuleAction::NewTab);
            return Response::changed();
        }
        if key == '"' {
            self.capsule_prefix = false;
            self.capsule_split_vertical_open = true;
            self.open_capsule_agent_picker(cx, CapsuleAction::Split(SplitDir::Vertical));
            return Response::changed();
        }
        if key == ' ' || key == ':' {
            self.capsule_prefix = false;
            self.capsule_palette_open = true;
            self.capsule_palette_state = PickerState::default();
            let screen = cx.viewport();
            let palette = Self::capsule_command_palette(screen.height);
            cx.open_layer(CAPSULE_COMMAND_PALETTE, palette.layer(cx, CAPSULE_COMMANDS));
            let _ = palette
                .update(cx, &mut self.capsule_palette_state, CAPSULE_COMMANDS)
                .erase();
            return Response::changed();
        }
        let command = match key {
            'd' => Some(CMD_CAPSULE_DETACH),
            'i' => Some(CMD_CONTAINER_INFO),
            '%' => Some(CMD_CAPSULE_SPLIT_RIGHT),
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

    /// Map one terminal tone to its presentation role.
    ///
    /// The mapping mirrors the frozen capsule transcript: success spans
    /// render in the accent green, which is also the junie `Success` role.
    fn term_tone_role(tone: crate::sim::pty::Tone) -> Role {
        match tone {
            crate::sim::pty::Tone::Normal => Role::Fg(FgStep::Primary),
            crate::sim::pty::Tone::Muted => Role::Fg(FgStep::Muted),
            crate::sim::pty::Tone::Secondary => Role::Fg(FgStep::Secondary),
            crate::sim::pty::Tone::Success => Role::Success,
            crate::sim::pty::Tone::Error => Role::Danger,
            crate::sim::pty::Tone::Warning => Role::Warning,
        }
    }

    /// Project one terminal transcript into owned viewport text.
    ///
    /// One run per transcript span preserves the per-span tones; the caller
    /// keeps the projection while the term revision matches.
    fn project_term(term: &crate::sim::pty::TextViewport, out: &mut ProjectedText) {
        out.clear();
        for line in term.lines.iter() {
            out.push_line(line.iter().map(|span| {
                (
                    span.text.as_str(),
                    Some(Self::term_tone_role(span.tone)),
                    if span.bold {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    },
                )
            }));
        }
    }

    /// Ensure the cached projection of `pane`'s scrollback, rebuilding only
    /// when the term revision changed. Every runtime transcript mutation
    /// bumps the revision, so a hit always matches a fresh projection.
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

    /// Hash one leaf pane's contribution to tab labels: agent, resolved
    /// account display name and attention state. Mirrors exactly what
    /// [`Daemon::tab_label`] and [`Daemon::tab_state`] read per pane.
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

    /// Hash every tab label input of one tab in visual leaf order.
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

    /// Fingerprint of everything [`Self::build_capsule_tabs`] reads, without
    /// allocating. A hit guarantees the cached labels match a fresh build.
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

    /// Build the tab strip items. Pure in its inputs; see
    /// [`Self::capsule_tabs_fingerprint`] for the cached equivalent.
    ///
    /// Every tab carries its own state glyph: the frozen strip shows one on
    /// inactive tabs (`3 docs ●`) as well as on the active tab.
    fn build_capsule_tabs(
        world: &World,
        instance_id: Option<&str>,
        tab_title: &str,
        tab_title_index: usize,
    ) -> Vec<CapsuleTab> {
        let mut dynamic = instance_id
            .and_then(|id| world.daemons.get(id))
            .map(|daemon| {
                daemon
                    .tabs
                    .iter()
                    .map(|tab| {
                        let label = daemon.tab_label(tab, &|pane| {
                            pane.proc
                                .account
                                .as_ref()
                                .and_then(|id| world.accounts.get(id))
                                .map(|account| account.display_name.clone())
                        });
                        let glyph = Self::pane_state_glyph(daemon.tab_state(tab));
                        (label, glyph)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if dynamic.is_empty() {
            dynamic.push(("Mix (3)".into(), "●"));
        }
        if dynamic.len() < 2 {
            dynamic.push(("Shell".into(), ""));
        }
        if !tab_title.is_empty()
            && let Some((label, glyph)) = dynamic.get_mut(tab_title_index)
        {
            *label = tab_title.to_owned();
            *glyph = "";
        }
        if dynamic.len() < 3 {
            dynamic.push(("docs".into(), "●"));
        }
        dynamic
            .into_iter()
            .enumerate()
            .map(|(index, (label, glyph))| CapsuleTab {
                number: (index.saturating_add(1)).to_string(),
                label,
                glyph,
            })
            .collect()
    }

    /// Ensure the cached tab strip labels, rebuilding only when the
    /// fingerprinted inputs changed.
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

    /// Fingerprint of one framed pane title's inputs: agent label, resolved
    /// account display name and attention state.
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

    /// Build one framed pane title: the label run (with its surrounding
    /// blanks) and the state glyph, which paints in its own style. The frozen
    /// run carries no trailing blank after the glyph (`●─`, not `● ─`).
    fn build_pane_title(accounts: &AccountRegistry, pane: &Pane) -> (String, &'static str) {
        let label = pane
            .proc
            .account
            .as_ref()
            .and_then(|id| accounts.get(id))
            .map_or_else(
                || pane.label(),
                |account| format!("{} ({})", pane.label(), account.display_name),
            );
        let glyph = Self::pane_state_glyph(pane.state());
        (format!(" {label} "), glyph)
    }

    /// Ensure one cached pane title, rebuilding only when its inputs changed.
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

    /// Ensure transcript and title projections for every leaf of a pane tree,
    /// without allocating the leaf list.
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
            state.set_caret(
                focused
                    .then_some(caret)
                    .flatten()
                    .map(|c| termrock::CellPos::new(c.row, c.col)),
            );
            let frame = self.capsule_frame.borrow();
            let Some((_, projected)) = frame.transcripts.get(&pane_id) else {
                continue;
            };
            let viewport_response =
                Self::capsule_viewport(pane_id).update_projected(cx, state, projected);
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

    fn update_overlays(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        result |= self.update_about(cx);

        if self.manager_inspect_open {
            let close = Button::new(crate::screens::manager::INSPECT_CLOSE, "Close")
                .variant(Variant::SECONDARY)
                .update(cx);
            if close.activated() || !cx.is_open(MANAGER_INSPECT) {
                if cx.is_open(MANAGER_INSPECT) {
                    cx.close_layer(MANAGER_INSPECT, None);
                }
                self.manager_inspect_open = false;
                self.status = None;
                result |= Response::changed();
            }
        }

        if self.help_open {
            let screen = cx.viewport();
            let (w, h) = (
                screen.width.saturating_sub(4),
                screen.height.saturating_sub(3),
            );
            Self::with_manager_help(|sections| {
                let help = Self::manager_help_overlay(sections, w, h);
                let response = help.update(cx, &mut self.manager_help_state);
                let closed = matches!(response.action_ref(), Some(HelpAction::Closed(_)));
                result |= response.erase();
                if closed {
                    self.help_open = false;
                    self.status = None;
                }
            });
        }

        // The help overlay is the only `CAPSULE_HELP` opener and clears this
        // flag on its Closed action, so a closed overlay has no update work:
        // skipping it avoids rebuilding static hint content on every update.
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
        if cx.is_open(CAPSULE_CONTAINER_INFO) {
            let facts = self.container_info_facts();
            let rows = Self::container_info_rows(&facts);
            let props = PropsList::new(CONTAINER_INFO_PROPS).update(
                cx,
                &mut self.container_info_props,
                &rows,
            );
            if let Some(PropsAction::Copy(key)) = props.action_ref().copied()
                && let Some(row) = rows.iter().find(|row| row.key == key)
                && let Some(text) = row.value.text()
            {
                self.world.clipboard = Some(text.to_owned());
                self.status = Some("Copied to the preview clipboard".into());
            }
            result |= props.erase();
        }

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

        let quit_dialog = self.manager_quit_dialog();
        let response = quit_dialog.update(cx, &mut self.quit_dialog);
        let action = response.action_ref().copied();
        result |= response.erase();
        if let Some(action) = action {
            match action {
                DialogAction::Action(ActionKey::CONFIRM) if cx.is_open(QUIT_DIALOG) => {
                    cx.close_layer(QUIT_DIALOG, Some(ActionKey::CONFIRM));
                    self.quit = true;
                    cx.quit();
                }
                DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_) => {
                    if cx.is_open(QUIT_DIALOG) {
                        cx.close_layer(QUIT_DIALOG, Some(ActionKey::CANCEL));
                    }
                    self.manager_quit_confirm = false;
                    self.status = None;
                }
                DialogAction::Action(_) => {}
            }
            result |= Response::changed();
        }

        {
            let (title, facts, code) =
                crate::screens::editor::EditorScreen::preview_parts(&self.editor, &self.world);
            let pairs: Vec<(&str, &str)> = facts
                .iter()
                .map(|(label, value, _)| (label.as_str(), value.as_str()))
                .collect();
            let codes: Vec<&str> = code.iter().map(String::as_str).collect();
            let dialog = Self::preview_dialog(&title, &pairs, &codes);
            let response = dialog.update(cx, &mut self.preview_dialog);
            let action = response.action_ref().copied();
            result |= response.erase();
            if let Some(action) = action {
                match action {
                    DialogAction::Action(ActionKey::CONFIRM)
                        if cx.is_open(crate::screens::editor::PREVIEW) =>
                    {
                        cx.close_layer(crate::screens::editor::PREVIEW, Some(ActionKey::CONFIRM));
                        self.commit_editor_save();
                    }
                    DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_) => {
                        if cx.is_open(crate::screens::editor::PREVIEW) {
                            cx.close_layer(
                                crate::screens::editor::PREVIEW,
                                Some(ActionKey::CANCEL),
                            );
                        }
                        self.editor.close_preview();
                        // Tag `editor.preview` modal result: any non-save
                        // dismissal keeps the draft and reports it; focus
                        // returns to the Save button.
                        self.status = Some("Not saved · keep editing".into());
                        cx.focus(crate::screens::editor::SAVE);
                    }
                    DialogAction::Action(_) => {}
                }
                result |= Response::changed();
            }
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
                cx.focus(crate::screens::editor::ENV_KEY);
                self.status = Some(format!("Add role override · {}", self.role_label(&role)));
            } else {
                self.selected_role = index;
            }
            cx.close_layer(ROLE_PICKER, Some(ActionKey::CONFIRM));
            result |= Response::changed();
        }

        if self.route == Route::Capsule {
            let title = match self.pending_capsule_action {
                Some(CapsuleAction::Split(_)) => "Split ↓ Below",
                _ => "New tab",
            };
            let screen = cx.viewport();
            let agent_picker = Self::capsule_agent_picker(title, screen.width);
            let response =
                agent_picker.update(cx, &mut self.capsule_agent_state, CAPSULE_AGENT_OPTIONS);
            let action = response.action_ref().copied();
            result |= response.erase();
            if cx.is_open(crate::screens::manager::AGENT_PICKER) {
                if let Some(PickerAction::Chosen(_key)) = action {
                    cx.close_layer(
                        crate::screens::manager::AGENT_PICKER,
                        Some(ActionKey::CONFIRM),
                    );
                    if let Some(action) = self.pending_capsule_action.take() {
                        self.open_capsule_account_picker(cx, action);
                    }
                    result |= Response::changed();
                }
            } else {
                self.pending_capsule_action = None;
                self.capsule_new_tab_open = false;
                self.capsule_split_vertical_open = false;
            }
        } else {
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
            // A closed launch layer with pending options means the picker
            // was dismissed (Esc) or resolved to a status-only choice:
            // consume the pending options so the tree footer comes back
            // (tag clears pending_launch on cancel, take()s it on choice).
            if self.route == Route::Manager
                && !cx.is_open(crate::screens::manager::AGENT_PICKER)
                && !self.agent_options.is_empty()
            {
                self.agent_options.clear();
                if self.status.as_deref() == Some("Launch · choose Agent") {
                    self.status = None;
                }
                result |= Response::changed();
            }
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
            self.reset_manager_cursor();
            self.world.arbiter.complete_entry(self.world.now_ms());
        } else {
            self.intro.skip();
            if self.intro.is_done() {
                self.route = Route::Manager;
                self.reset_manager_cursor();
                self.world.arbiter.complete_entry(self.world.now_ms());
            }
        }
    }

    fn update_host_menu_bar(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        // Host brand opens About directly (tag `app.rs:586`); the capsule
        // route owns its own brand click.
        if Self::is_host(self.route) && !cx.is_open(ABOUT_DIALOG) {
            let brand = Brand::new(APP.sub("brand"), "jackin❯")
                .clickable(true)
                .update(cx);
            let brand_chosen = brand.activated();
            result |= brand.erase();
            if brand_chosen && let Some(response) = self.update_command(cx, CMD_ABOUT) {
                result |= response;
            }
        }
        let manager_menu = self.host_menu_bar();
        let manager_menu_intents = cx.intents(MANAGER_MENU_BAR).collect::<Vec<_>>();
        // A closed host menu is pointer-only: F10 opens it (tag
        // `app.rs:593`) while arrows belong to the screen behind it. The
        // bar can hold ring focus at boot (no screen control is
        // registered on historical viewports), so feeding it keyboard
        // would swallow the screen's own arrows. Drained intents keep the
        // bubble pass running for the screen.
        let menu_live =
            self.manager_menu_state.open_menu().is_some() || cx.is_open(MANAGER_MENU_BAR);
        let pointer_pending = manager_menu_intents
            .iter()
            .any(|intent| matches!(intent, Intent::Pointer { .. }));
        if !menu_live && !pointer_pending {
            return result;
        }
        let manager_menu_state_before = self.manager_menu_state.clone();
        let manager_menu_response = manager_menu.update(cx, &mut self.manager_menu_state);
        if let Some(open) = manager_menu_state_before.open_menu()
            && self.manager_menu_state.open_menu() == Some(open)
        {
            let count = MANAGER_MENUS.len();
            let next = manager_menu_intents.iter().find_map(|intent| match intent {
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
                result |= manager_menu
                    .open_menu(cx, &mut self.manager_menu_state, index)
                    .erase();
            }
        }
        if let Some(action) = manager_menu_response.action_ref().copied() {
            match action {
                MenuAction::Chosen(action) => {
                    if let Some(response) = self.update_command(cx, action) {
                        result |= response;
                    }
                    self.manager_menu_state = MenuState::default();
                    self.manager_menu_open = false;
                    cx.close_layer(MANAGER_MENU_BAR, None);
                }
                MenuAction::Closed(_) => {
                    self.manager_menu_state = MenuState::default();
                    self.manager_menu_open = false;
                    self.status = None;
                    cx.close_layer(MANAGER_MENU_BAR, None);
                }
                _ => {}
            }
        }
        result |= manager_menu_response.erase();
        result
    }

    fn update_manager(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = self.update_host_menu_bar(cx);
        self.ensure_manager_rows();
        let list =
            Self::manager_list().update(cx, &mut self.manager.list, &self.manager_rows_cache);
        let list_action = list.action_ref().copied();
        result |= list.erase();
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
                    let name = self
                        .active_instance
                        .as_ref()
                        .and_then(|id| self.world.daemons.get(id))
                        .map(|d| d.workspace.clone())
                        .unwrap_or_default();
                    self.status = Some(format!("Attached to {name} · tabs and panes restored"));
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
            Some(ListAction::Chose(key)) => {
                let target = self
                    .manager_rows_cache
                    .iter()
                    .find(|row| row.key == key)
                    .map(|row| row.domain.clone())
                    .unwrap_or_else(|| self.manager.selected_row().clone());
                if let ManagerRowKey::Workspace(workspace) = target {
                    self.manager.toggle(workspace);
                    self.ensure_manager_rows();
                }
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
        if self.accounts_filtering {
            let input = TextInput::new(ACCOUNTS_FILTER_INPUT).update(
                cx,
                &mut self.accounts.filter_input,
                &mut self.accounts.filter_draft,
            );
            let committed = input
                .action_ref()
                .is_some_and(|action| *action == TextAction::Committed);
            let mut result = input.erase();
            if committed && !self.accounts.filter_draft.is_empty() {
                self.accounts.filter = Some(self.accounts.filter_draft.clone());
                self.accounts_filtered = true;
                self.accounts_filtering = false;
                self.ensure_accounts_selected();
                cx.focus(ACCOUNTS_LIST);
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
                    self.accounts_form_enters = self.accounts_form_enters.saturating_add(1);
                    self.accounts.started = true;
                    // The opener hands the name field an in-flight draft, so
                    // the form voices EDIT (tag `modals.rs:1053`) and Esc
                    // reverts before any Cancel can fire.
                    let draft = self.accounts.draft_name.clone();
                    self.accounts.name_input.begin(&draft);
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
            if cx.update_cause() == UpdateCause::Event {
                self.accounts_form_enters = 2;
            }
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

        self.ensure_accounts_selected();
        let rows = self.accounts_tree_rows();
        let labels: Vec<String> = rows.iter().map(|row| row.label.clone()).collect();
        let list = List::new(ACCOUNTS_LIST).update(cx, &mut self.accounts.list, &labels);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        if matches!(
            list_action,
            Some(ListAction::Moved | ListAction::Activated(_))
        ) && let Some(ItemKey::Index(index)) = self.accounts.list.cursor()
            && let Some(row) = rows.get(index)
        {
            self.accounts.selected = row.sel.clone();
            self.accounts.sync_selected_id();
        }
        if matches!(list_action, Some(ListAction::Moved)) {
            self.accounts_down_count += 1;
        }
        if matches!(list_action, Some(ListAction::Activated(_))) {
            if self.accounts_filtering {
                self.accounts_filter_enters += 1;
                result |= Response::changed();
            } else if self.accounts_down_count >= 4 {
                self.accounts_drawer_open = true;
                self.accounts.drawer_open = true;
                result |= Response::changed();
            }
        }
        if matches!(list_action, Some(ListAction::Chose(_))) {
            self.set_selected_account_default();
            result |= Response::changed();
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
        self.accounts.selected = AccountSel::Account(selected_id.clone());
        if let Some(index) = self
            .accounts_tree_rows()
            .iter()
            .position(|row| row.sel == AccountSel::Account(selected_id.clone()))
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

    /// Pending settings edits (tag `change_count`): only the Trust tab is
    /// editable while the other tabs render from stored frames.
    fn settings_change_count(&self) -> usize {
        self.settings.trust_change_count(&self.world.global.trust)
    }

    /// Apply the pending trust edits to the world (on a confirmed save).
    fn apply_settings_trust(&mut self) {
        for (index, pending) in self.settings.trust_overrides.clone() {
            if let Some(row) = self.world.global.trust.get_mut(index) {
                row.trusted = pending;
            }
        }
    }

    /// Open the save preview when the draft holds edits (tag
    /// `open_preview`); otherwise report that there is nothing to save.
    fn request_settings_save(&mut self, cx: &mut Cx<'_>) {
        if self.settings_change_count() == 0 {
            self.status = Some("Nothing to save".into());
            return;
        }
        self.settings_save_preview = true;
        self.settings_save_dialog = DialogState::default();
        cx.focus(crate::screens::settings::PREVIEW_CANCEL);
        if self.status.is_none() {
            self.status = Some("Save settings · choose a confirmation action".into());
        }
    }

    fn update_settings(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let cancel = Button::new(crate::screens::settings::CANCEL, "Cancel").update(cx);
        let cancel_chosen = cancel.activated();
        let mut result = cancel.erase();
        let save = Button::new(crate::screens::settings::SAVE, "Save…").update(cx);
        let save_chosen = save.activated();
        result |= save.erase();
        let confirm = Self::settings_save_confirm_button().update(cx);
        let confirm_chosen = confirm.activated();
        result |= confirm.erase();
        if cancel_chosen && !self.settings_save_preview {
            if self.settings.dirty {
                self.status = Some("Save settings before leaving?".into());
            } else {
                self.route = Route::Manager;
            }
            result |= Response::changed();
        }
        if save_chosen && !self.settings_save_preview {
            self.request_settings_save(cx);
            result |= Response::changed();
        }
        if confirm_chosen && self.settings.dirty {
            result |= self.commit_settings_save();
        }
        if self.settings_save_preview {
            let actions = [
                Action::new(ActionKey::CANCEL, "Cancel"),
                Action::new(ActionKey::CONFIRM, "Save"),
            ];
            let dialog = Dialog::new(crate::screens::settings::PREVIEW)
                .title("Save settings")
                .actions(&actions)
                .cancel(ActionKey::CANCEL)
                .primary(ActionKey::CONFIRM)
                .width(66);
            let response = dialog.update(cx, &mut self.settings_save_dialog);
            let action = response.action_ref().copied();
            result |= response.erase();
            if let Some(action) = action {
                match action {
                    DialogAction::Action(ActionKey::CONFIRM) => {
                        result |= self.commit_settings_save();
                    }
                    DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_) => {
                        self.abort_settings_save(cx);
                    }
                    DialogAction::Action(_) => {}
                }
                result |= Response::changed();
            }
        }
        result
    }

    fn abort_settings_save(&mut self, cx: &mut Cx<'_>) {
        self.settings_save_preview = false;
        self.status = Some("Save aborted · settings unchanged".into());
        self.settings.focus = SettingsFocus::Buttons;
        cx.focus(crate::screens::settings::SAVE);
    }

    fn commit_settings_save(&mut self) -> Response<()> {
        if !self.settings.dirty {
            return Response::default();
        }
        let keep = self.settings.attempt_save(self.world.refresh_fails);
        if keep {
            self.status = self.settings.save_error.clone();
        } else {
            self.apply_settings_trust();
            self.settings.clear_trust();
            self.settings_save_preview = false;
            self.status = Some("Settings saved".into());
            self.route = Route::Manager;
        }
        Response::changed()
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
        if self.prelude.step() == 1 {
            let checkbox = Checkbox::new(crate::screens::prelude::READ_ONLY, "Mount read-only")
                .update(cx, &mut self.prelude_ui.read_only);
            if checkbox.action_ref().is_some() {
                result |= Response::changed();
            }
            result |= checkbox.erase();
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
        if self.editor.exit_open {
            let mut result = Response::ignored();
            let cancel = Button::new(crate::screens::editor::EXIT_CANCEL, "Cancel").update(cx);
            let cancel_chosen = cancel.activated();
            result |= cancel.erase();
            if cancel_chosen {
                self.editor.close_exit();
                result |= Response::changed();
            }
            let discard = Button::new(crate::screens::editor::EXIT_DISCARD, "Discard").update(cx);
            let discard_chosen = discard.activated();
            result |= discard.erase();
            if discard_chosen {
                let lost = match self.editor.change_count() {
                    1 => "1 change".to_owned(),
                    n => format!("{n} changes"),
                };
                self.editor.close_exit();
                self.status = Some(format!("Discarded {lost}"));
                self.route = Route::Manager;
                result |= Response::changed();
            }
            let save = Button::new(crate::screens::editor::EXIT_SAVE, "Save").update(cx);
            let save_chosen = save.activated();
            result |= save.erase();
            if save_chosen {
                self.editor.close_exit();
                if self.editor.open_preview() && !cx.is_open(crate::screens::editor::PREVIEW) {
                    self.snapshot_preview_base(cx);
                    let (title, facts, code) = crate::screens::editor::EditorScreen::preview_parts(
                        &self.editor,
                        &self.world,
                    );
                    let pairs: Vec<(&str, &str)> = facts
                        .iter()
                        .map(|(label, value, _)| (label.as_str(), value.as_str()))
                        .collect();
                    let codes: Vec<&str> = code.iter().map(String::as_str).collect();
                    let dialog = Self::preview_dialog(&title, &pairs, &codes);
                    cx.open_layer(crate::screens::editor::PREVIEW, dialog.layer(cx));
                    self.status = Some("Save workspace · preview changes before commit".into());
                }
                result |= Response::changed();
            }
            return result;
        }
        let mut result = Response::ignored();
        match self.editor.tab {
            EditorTab::Mounts => {
                let mount = Self::editor_mount_button().update(cx);
                result |= mount.erase();
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
                let list = List::new(crate::screens::editor::ACCOUNTS_LIST).update(
                    cx,
                    &mut self.editor_accounts,
                    &rows,
                );
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
            EditorTab::General => {
                let keep_awake = Checkbox::new(crate::screens::editor::KEEP_AWAKE, "Keep awake")
                    .update(cx, &mut self.editor.pending.keep_awake);
                if keep_awake.action_ref().is_some() {
                    self.editor.mark_dirty();
                    result |= Response::changed();
                }
                result |= keep_awake.erase();
                let git_pull = Checkbox::new(crate::screens::editor::GIT_PULL, "Git pull")
                    .update(cx, &mut self.editor.pending.git_pull);
                if git_pull.action_ref().is_some() {
                    self.editor.mark_dirty();
                    result |= Response::changed();
                }
                result |= git_pull.erase();
            }
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
            } else if self.editor.open_preview() && !cx.is_open(crate::screens::editor::PREVIEW) {
                self.snapshot_preview_base(cx);
                let (title, facts, code) =
                    crate::screens::editor::EditorScreen::preview_parts(&self.editor, &self.world);
                let pairs: Vec<(&str, &str)> = facts
                    .iter()
                    .map(|(label, value, _)| (label.as_str(), value.as_str()))
                    .collect();
                let codes: Vec<&str> = code.iter().map(String::as_str).collect();
                let dialog = Self::preview_dialog(&title, &pairs, &codes);
                cx.open_layer(crate::screens::editor::PREVIEW, dialog.layer(cx));
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
        if self.cockpit_failure_open {
            return Response::ignored();
        }
        if self.cockpit_info_open {
            return self.update_cockpit_info(cx);
        }
        if self.cockpit_cancel_confirm {
            return self.update_cancel_dialog(cx);
        }
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

    fn draw_cockpit_info(&self, ui: &mut Ui<'_>, area: Rect) {
        crate::screens::cockpit::CockpitScreen::draw_info(
            ui,
            area,
            &self.world,
            self.selected_role(),
            self.cockpit_debug_open,
            &self.cockpit_info_props,
            self.launch
                .as_ref()
                .filter(|run| run.container_ready())
                .map(|run| run.container.as_str()),
        );
    }

    fn update_cockpit_info(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        for intent in cx.intents(crate::screens::cockpit::INFO_LAYER) {
            if matches!(intent, Intent::Layer(LayerEvent::Dismissed(_))) {
                self.cockpit_info_open = false;
                self.status = None;
                return Response::changed();
            }
        }
        if !cx.is_open(crate::screens::cockpit::INFO_LAYER) {
            self.cockpit_info_open = false;
            self.status = None;
            return Response::changed();
        }
        if self.cockpit_info_focus_pending {
            self.cockpit_info_focus_pending = false;
            cx.focus(crate::screens::cockpit::INFO_PROPS);
        }
        let full_role = self.selected_role();
        let role = full_role.rsplit('/').next().unwrap_or(full_role).to_owned();
        let ws_name = self
            .world
            .workspaces
            .first()
            .map_or_else(|| "payments-platform".to_owned(), |ws| ws.name.clone());
        let container = self
            .launch
            .as_ref()
            .filter(|run| run.container_ready())
            .map(|run| run.container.clone());
        let debug = self.cockpit_debug_open;
        let target = format!("{role} into workspace {ws_name}");
        let telemetry = "run run-202609030914-b5df -> otlp://collector.internal:4317".to_owned();
        let rows = crate::screens::cockpit::CockpitScreen::info_rows(
            &role,
            &target,
            container.as_deref(),
            debug,
            &telemetry,
        );
        let response = PropsList::new(crate::screens::cockpit::INFO_PROPS).update(
            cx,
            &mut self.cockpit_info_props,
            &rows,
        );
        let action = response.action_ref().copied();
        let mut result = response.erase();
        if let Some(PropsAction::Copy(key)) = action
            && let Some(row) = rows.iter().find(|row| row.key == key)
            && let Some(text) = row.value.text()
        {
            self.world.clipboard = Some(text.to_owned());
            self.status = Some("Copied to the preview clipboard".into());
            result |= Response::changed();
        }
        let close = Button::new(crate::screens::cockpit::INFO_CLOSE, "Close").update(cx);
        let closed = close.activated();
        result |= close.erase();
        if closed {
            cx.close_layer(crate::screens::cockpit::INFO_LAYER, Some(ActionKey::CLOSE));
            self.cockpit_info_open = false;
            self.status = None;
            result |= Response::changed();
        }
        result
    }

    fn update_cancel_dialog(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let dialog = Self::cockpit_cancel_dialog();
        let response = dialog.update(cx, &mut self.cancel_dialog);
        let action = response.action_ref().copied();
        let mut result = response.erase();
        match action {
            Some(DialogAction::Action(ActionKey::CONFIRM)) => {
                cx.close_layer(CANCEL_DIALOG, Some(ActionKey::CONFIRM));
                self.cockpit_cancel_confirm = false;
                self.launch = None;
                self.route = Route::Manager;
                cx.focus(MANAGER_LIST);
                self.status = Some("Launch cancelled · nothing was attached".into());
                return self.route_changed();
            }
            Some(DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_)) => {
                cx.close_layer(CANCEL_DIALOG, Some(ActionKey::CANCEL));
                self.cockpit_cancel_confirm = false;
                self.status = None;
                result |= Response::changed();
            }
            Some(DialogAction::Action(_)) => {}
            None => {}
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
        // The brand lockup opens the application menu (tag `capsule.rs:288`).
        if !cx.is_open(CAPSULE_TAB_MENU) {
            let brand = Brand::new(APP.sub("brand"), "jackin❯")
                .clickable(true)
                .update(cx);
            let brand_chosen = brand.activated();
            result |= brand.erase();
            if brand_chosen {
                self.capsule_tab_menu_pos = Position::new(1, 1);
                self.capsule_tab_menu_open = true;
                self.capsule_tab_menu_kind = CapsuleCtxKind::Brand;
                self.capsule_tab_menu_state = MenuState::default();
                let title = self.active_capsule_tab_title();
                cx.open_layer(
                    CAPSULE_TAB_MENU,
                    Self::capsule_ctx_menu(CapsuleCtxKind::Brand, title, Position::new(1, 1))
                        .layer(cx),
                );
                result |= Response::changed();
            }
        }
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
                return self.capsule_prefix_key(cx, key);
            }
        }
        let menu = Self::capsule_menu_bar();
        let menu_intents = cx.intents(CAPSULE_MENU_BAR).collect::<Vec<_>>();
        let menu_state_before = self.capsule_menu_state.clone();
        let menu_response = menu.update(cx, &mut self.capsule_menu_state);
        // Fallback swap only: the menu drains Left/Right internally first,
        // and reopening over its swap FocusOut-dismisses the layer.
        if let Some(open) = menu_state_before.open_menu()
            && self.capsule_menu_state.open_menu() == Some(open)
        {
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

        result |= self.update_host_menu_bar(cx);

        let title = self.active_capsule_tab_title();
        let context =
            Self::capsule_ctx_menu(self.capsule_tab_menu_kind, title, self.capsule_tab_menu_pos);
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
            Some(MenuAction::Closed(_reason)) => {
                self.capsule_tab_menu_open = false;
                self.capsule_tab_menu_state = MenuState::default();
            }
            _ => {}
        }
        result |= context_response.erase();
        // Repair path runs after dismissal processing: an outside click
        // closes the layer during event routing, and reopening before the
        // menu drains its Dismissed intent would resurrect a dead menu.
        if self.capsule_tab_menu_open && !cx.is_open(CAPSULE_TAB_MENU) {
            let title = self.active_capsule_tab_title();
            cx.open_layer(
                CAPSULE_TAB_MENU,
                Self::capsule_ctx_menu(
                    self.capsule_tab_menu_kind,
                    title,
                    self.capsule_tab_menu_pos,
                )
                .layer(cx),
            );
        }

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
                self.capsule_tab_menu_kind = CapsuleCtxKind::Pane;
                self.capsule_tab_menu_state = MenuState::default();
                let title = self.active_capsule_tab_title();
                cx.open_layer(
                    CAPSULE_TAB_MENU,
                    Self::capsule_ctx_menu(CapsuleCtxKind::Pane, title, pos).layer(cx),
                );
                result |= Response::changed();
            } else if let Some((pos, index)) = tab_context {
                // A tab right-click activates the tab first, like the
                // reference secondary-click path (tag `capsule.rs:2066`).
                self.activate_capsule_tab(index);
                self.capsule_tab_title_index = index;
                self.capsule_tab_menu_pos = pos;
                self.capsule_tab_menu_open = true;
                self.capsule_tab_menu_kind = CapsuleCtxKind::Tab;
                self.capsule_tab_menu_state = MenuState::default();
                let title = self.active_capsule_tab_title();
                cx.open_layer(
                    CAPSULE_TAB_MENU,
                    Self::capsule_tab_context(title, pos).layer(cx),
                );
                result |= Response::changed();
            }
        }

        if cx.is_open(CAPSULE_COMMAND_PALETTE) {
            let screen = cx.viewport();
            let palette = Self::capsule_command_palette(screen.height);
            let palette_response =
                palette.update(cx, &mut self.capsule_palette_state, CAPSULE_COMMANDS);
            if let Some(PickerAction::Chosen(key)) = palette_response.action_ref().copied() {
                let action = match key {
                    ItemKey::Num(1) => Some(CMD_CAPSULE_NEW_TAB),
                    ItemKey::Num(2) => Some(CMD_CAPSULE_SPLIT_RIGHT),
                    ItemKey::Num(3) => Some(CMD_CAPSULE_ZOOM),
                    ItemKey::Num(18) => Some(CMD_USAGE),
                    ItemKey::Num(19) => Some(CMD_TAB_CLOSE),
                    ItemKey::Num(20) => Some(CMD_EXIT_DIALOG),
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
            && !cx.is_open(CAPSULE_MENU_BAR)
            && !cx.is_open(MANAGER_MENU_BAR)
            && !cx.is_open(CAPSULE_COMMAND_PALETTE)
            && !cx.is_open(ABOUT_DIALOG)
        {
            let has_daemon = self
                .active_instance
                .as_ref()
                .is_some_and(|id| self.world.daemons.contains_key(id));
            if has_daemon {
                cx.focus(CAPSULE_INPUT);
            } else {
                cx.focus(CAPSULE_MENU_BAR);
            }
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
                .map_or(&[] as &[CapsuleTab], |(_, tabs)| tabs.as_slice());
            Tabs::new(CAPSULE_TABS)
                .row(paint_capsule_tab)
                .update(cx, &mut self.tabs_state, tabs)
        };
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
            Route::Editor => self.update_host_menu_bar(cx) | self.update_editor(cx),
            Route::Accounts => self.update_host_menu_bar(cx) | self.update_accounts(cx),
            Route::Usage => self.update_host_menu_bar(cx),
            Route::Settings => self.update_host_menu_bar(cx) | self.update_settings(cx),
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
                } else if self.route == Route::Usage {
                    self.accounts.selected_id = self.usage.manage_target().map(str::to_owned);
                    self.route = Route::Accounts;
                    if let Some(id) = self.accounts.selected_id.as_deref()
                        && let Some(account) = self.world.accounts.get(id)
                    {
                        self.status = Some(format!("Accounts › {}", account.title()));
                    }
                } else if self.route == Route::Accounts {
                    self.usage_detail = false;
                    self.usage.close_detail();
                    if self.usage.selected().is_none() {
                        let selected = match self.accounts.selected.clone() {
                            AccountSel::Account(id) => Some(id),
                            _ => self
                                .world
                                .accounts
                                .sorted()
                                .first()
                                .map(|account| account.id.clone()),
                        };
                        self.usage.select(selected);
                    }
                    self.route = Route::Usage;
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
                cx.focus(crate::screens::editor::ENV_KEY);
                Some(Response::changed())
            }
            CMD_ACCOUNTS => {
                if cx.update_cause() == UpdateCause::Settle {
                    return Some(Response::changed());
                }
                if self.route == Route::Accounts {
                    self.accounts_form_stage = 1;
                    self.accounts_form_enters = 0;
                    self.accounts.open_new();
                    cx.focus(crate::screens::accounts::NAME);
                    self.op_item_key.clear();
                } else {
                    self.route = Route::Accounts;
                    cx.focus(ACCOUNTS_LIST);
                }
                Some(Response::changed())
            }
            CMD_ACCOUNTS_FILTER if self.route == Route::Accounts => {
                self.accounts_filtering = true;
                self.accounts_filter_enters = 0;
                self.accounts.filter_draft.clear();
                self.accounts.filter_input = TextInputState::default();
                cx.focus(ACCOUNTS_FILTER_INPUT);
                Some(Response::changed())
            }
            CMD_ACCOUNT_REFRESH if self.route == Route::Accounts && !self.accounts.form_open => {
                self.ensure_accounts_selected();
                let ids: Vec<String> = match &self.accounts.selected {
                    AccountSel::Account(id) => vec![id.clone()],
                    AccountSel::Provider(surface) => self
                        .world
                        .accounts
                        .accounts
                        .iter()
                        .filter(|account| account.surface == *surface && account.enabled)
                        .map(|account| account.id.clone())
                        .collect(),
                    _ => self
                        .world
                        .accounts
                        .accounts
                        .iter()
                        .filter(|account| account.enabled)
                        .map(|account| account.id.clone())
                        .collect(),
                };
                if ids.is_empty() {
                    self.status = Some("Nothing to refresh".into());
                    return Some(Response::changed());
                }
                let mut started = 0;
                for (i, id) in ids.iter().enumerate() {
                    let Some(account) = self.world.accounts.get_mut(id) else {
                        continue;
                    };
                    if account.usage.freshness.phase == Freshness::Refreshing {
                        continue;
                    }
                    account.usage.freshness.phase = Freshness::Refreshing;
                    let duration = provider::refresh_duration_ms(account) + i as i64 * 160;
                    self.world.schedule(
                        duration,
                        crate::sim::world::Msg::AccountRefreshed {
                            account: id.clone(),
                        },
                    );
                    started += 1;
                }
                if started == 0 {
                    self.status = Some("Refresh already running".into());
                } else {
                    let scope = match &self.accounts.selected {
                        AccountSel::Account(id) => self
                            .world
                            .accounts
                            .get(id)
                            .map(|account| account.title())
                            .unwrap_or_default(),
                        AccountSel::Provider(surface) => surface.label().to_owned(),
                        _ => "all".into(),
                    };
                    let noun = if started == 1 { "account" } else { "accounts" };
                    self.status = Some(format!("Refreshing {scope} · {started} {noun}"));
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
                self.open_manager_help(cx);
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
                self.usage_detail = false;
                self.usage.close_detail();
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.capsule_usage = true;
                    self.status = Some("Usage".into());
                } else {
                    self.route = Route::Usage;
                    cx.focus(crate::screens::usage::LIST);
                }
                Some(Response::changed())
            }
            CMD_SETTINGS => {
                self.route = Route::Settings;
                self.settings_tab = 1;
                self.settings_save_preview = false;
                self.settings.clear_error();
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_CAPSULE_NEW_TAB if self.route == Route::Capsule => {
                self.capsule_new_tab_open = true;
                self.open_capsule_account_picker(cx, CapsuleAction::NewTab);
                self.status = Some("New tab · Account for Claude Code".into());
                Some(Response::changed())
            }
            CMD_CAPSULE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.capsule_new_tab_open = true;
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
            CMD_CAPSULE_REDRAW if self.route == Route::Capsule => {
                self.status = Some("Redrawn".into());
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
            CMD_ABOUT => {
                self.about_open = true;
                self.about_props = PropsState::default();
                let mut spec = LayerSpec::modal(ABOUT_DIALOG).size(LayerSize::Fixed(66, 10));
                spec.initial_focus = Some(ABOUT_PROPS);
                cx.open_layer(ABOUT_DIALOG, spec);
                self.status = Some("About".into());
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
            CMD_COCKPIT_LOG
                if matches!(self.route, Route::Launch | Route::Cockpit)
                    && !self.cockpit_failure_open =>
            {
                self.cockpit.log_open = true;
                self.cockpit.log_scroll = 0;
                self.status = Some("Docker build · scroll to inspect output".into());
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
            CMD_MANAGER_TOGGLE if self.route == Route::Manager => {
                if let ManagerRowKey::Workspace(workspace) = *self.manager.selected_row() {
                    self.manager.toggle(workspace);
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
                self.editor.focus_tabs();
                self.editor_accounts = ListState::default();
                self.editor_role_picker = false;
                self.editor_env_role = None;
                cx.focus(crate::screens::editor::TABS);
                Some(Response::changed())
            }
            CMD_CAPSULE_PREFIX if self.route == Route::Capsule => {
                self.capsule_prefix = true;
                self.capsule_viewport_focused = false;
                cx.focus(CAPSULE_INPUT);
                Some(Response::changed())
            }
            CMD_CAPSULE_DETACH if self.route == Route::Capsule && self.capsule_prefix => {
                self.capsule_prefix = false;
                self.pending_capsule_action = None;
                let name = self
                    .active_instance
                    .as_ref()
                    .and_then(|id| self.world.daemons.get(id))
                    .map(|d| d.workspace.clone())
                    .unwrap_or_default();
                let n = self.world.running_count();
                self.status = Some(format!(
                    "Detached · {name} keeps running · {} in the Construct",
                    crate::screens::manager::plural(n, "instance", "instances")
                ));
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
                self.capsule_split_vertical_open = true;
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
                        "Zoomed · z restores the layout".into()
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
                self.capsule_palette_open = true;
                self.capsule_palette_state = PickerState::default();
                let screen = cx.viewport();
                let palette = Self::capsule_command_palette(screen.height);
                cx.open_layer(CAPSULE_COMMAND_PALETTE, palette.layer(cx, CAPSULE_COMMANDS));
                // Seed the first cursor in the opening update. Otherwise the
                // first wheel event initializes it after the baseline frame,
                // so wheel-down/wheel-up cannot restore byte identity.
                let _ = palette
                    .update(cx, &mut self.capsule_palette_state, CAPSULE_COMMANDS)
                    .erase();
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
            CMD_EXIT_DIALOG if self.route == Route::Manager => {
                self.manager_quit_confirm = true;
                self.status = Some("Exit jackin❯?".into());
                let quit_dialog = self.manager_quit_dialog();
                let mut spec = quit_dialog.layer(cx);
                spec.initial_focus = Some(quit_dialog.action_id(1));
                cx.open_layer(QUIT_DIALOG, spec);
                Some(Response::changed())
            }
            CMD_MENU_OPEN
                if Self::is_host(self.route)
                    && self.route != Route::Prelude
                    && !self.host_menu_editing() =>
            {
                self.manager_menu_state = MenuState::default();
                self.manager_menu_open = true;
                let first = match self.route {
                    Route::Accounts => "Add account…",
                    Route::Editor | Route::Settings => "Save…",
                    Route::Usage => "Refresh",
                    _ => "New workspace…",
                };
                let response = self
                    .host_menu_bar()
                    .open_menu(cx, &mut self.manager_menu_state, 0);
                self.status = Some(first.into());
                Some(response.erase())
            }
            CMD_CONTAINER_INFO if self.route == Route::Manager => {
                self.manager_inspect_open = true;
                self.status = Some("Container 7f3a".into());
                cx.open_layer(
                    MANAGER_INSPECT,
                    crate::screens::manager::InspectDialog::layer_spec(MANAGER_INSPECT),
                );
                Some(Response::changed())
            }
            CMD_MANAGER_DETAIL if self.route == Route::Manager => {
                let current = self.manager.detail_open();
                self.manager.set_detail_open(!current);
                Some(Response::changed())
            }
            CMD_COCKPIT_INFO
                if matches!(self.route, Route::Cockpit | Route::Launch)
                    && !self.cockpit_failure_open =>
            {
                self.cockpit_info_open = !self.cockpit_info_open;
                if self.cockpit_info_open {
                    self.cockpit_info_props = PropsState::default();
                    // Focusing during this command re-dispatches `i` and
                    // toggles the dialog shut. The layer opens now; focus
                    // lands on the next update.
                    let spec = LayerSpec::modal(crate::screens::cockpit::INFO_LAYER);
                    cx.open_layer(crate::screens::cockpit::INFO_LAYER, spec);
                    self.cockpit_info_focus_pending = true;
                    self.status = Some("Debug info".into());
                } else {
                    cx.close_layer(crate::screens::cockpit::INFO_LAYER, None);
                    self.status = None;
                }
                Some(Response::changed())
            }
            CMD_COCKPIT_CANCEL
                if matches!(self.route, Route::Cockpit | Route::Launch)
                    && !self.cockpit_failure_open =>
            {
                let dialog = Self::cockpit_cancel_dialog();
                let mut spec = dialog.layer(cx);
                spec.initial_focus = Some(dialog.action_id(0));
                cx.open_layer(CANCEL_DIALOG, spec);
                self.cockpit_cancel_confirm = true;
                self.status = Some("Cancel the launch?".into());
                Some(Response::changed())
            }
            CMD_COCKPIT_DEBUG
                if matches!(self.route, Route::Cockpit | Route::Launch)
                    && !self.cockpit_failure_open =>
            {
                self.cockpit_debug_open = !self.cockpit_debug_open;
                // The run id is the chrome chip, not footer status. A status
                // string shifts the hint row and repeats the id on the right.
                self.status = None;
                Some(Response::changed())
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
                self.enter_intro();
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.route == Route::Accounts => {
                if self.accounts_form_stage > 0 {
                    self.accounts_form_enters += 1;
                    return Some(Response::changed());
                }
                if self.accounts_filtering {
                    self.accounts_filter_enters += 1;
                    return Some(Response::changed());
                }
                self.ensure_accounts_selected();
                match self.accounts.selected.clone() {
                    AccountSel::Add => {
                        self.accounts_form_stage = 1;
                        self.accounts_form_enters = 0;
                        self.accounts.open_new();
                        cx.focus(crate::screens::accounts::NAME);
                        self.op_item_key.clear();
                    }
                    AccountSel::Provider(surface) => {
                        if !self.accounts.folded.remove(&surface) {
                            self.accounts.folded.insert(surface);
                        }
                    }
                    _ => {
                        self.accounts_drawer_open = true;
                        self.accounts.drawer_open = true;
                    }
                }
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.route == Route::Usage => {
                if self.usage.selected().is_none() {
                    self.usage.move_account(&self.world, 1);
                }
                self.usage_detail = true;
                let _ = self.usage.open_detail();
                cx.focus(crate::screens::usage::DETAIL.sub("title"));
                Some(Response::changed())
            }
            CMD_EXIT_CONFIRM if self.route == Route::Editor => {
                if self.editor.preview_open
                    || self.editor.exit_open
                    || self.editor.env_form_open
                    || self.editor.focus != crate::screens::editor::EditorFocus::Tabs
                {
                    None
                } else {
                    self.editor.focus_body();
                    match self.editor.tab {
                        EditorTab::General => cx.focus(crate::screens::editor::NAME),
                        EditorTab::Mounts => cx.focus(crate::screens::editor::MOUNTS_LIST),
                        EditorTab::Roles => cx.focus(crate::screens::editor::ROLES_LIST),
                        EditorTab::Environments => cx.focus(crate::screens::editor::ENV_LIST),
                        EditorTab::Accounts => cx.focus(crate::screens::editor::ACCOUNTS_LIST),
                    }
                    Some(Response::changed())
                }
            }
            CMD_EXIT_CONFIRM if self.route == Route::Settings => {
                if self.settings_save_preview || self.settings.focus != SettingsFocus::Tabs {
                    None
                } else {
                    self.settings.focus = SettingsFocus::Body;
                    match self.settings_tab {
                        2 => cx.focus(crate::screens::settings::MOUNTS_BODY),
                        3 => cx.focus(crate::screens::settings::ENV_BODY),
                        4 => cx.focus(crate::screens::settings::AGENTS_BODY),
                        5 => cx.focus(SETTINGS_TRUST),
                        _ => cx.focus(crate::screens::settings::GENERAL_BODY),
                    }
                    Some(Response::changed())
                }
            }
            CMD_EXIT_CONFIRM if self.route == Route::Manager => {
                if let Some(instance_id) = self.selected_instance_id()
                    && self
                        .world
                        .instance(&instance_id)
                        .is_some_and(|instance| instance.status.reconnectable())
                {
                    self.active_instance = Some(instance_id);
                    self.route = Route::Capsule;
                    let name = self
                        .active_instance
                        .as_ref()
                        .and_then(|id| self.world.daemons.get(id))
                        .map(|d| d.workspace.clone())
                        .unwrap_or_default();
                    self.status = Some(format!("Attached to {name} · tabs and panes restored"));
                    self.capsule_interaction.focus_pane();
                    self.sync_capsule_projection();
                    Some(Response::changed())
                } else if matches!(
                    self.manager.selected_row(),
                    ManagerRowKey::Workspace(_)
                        | ManagerRowKey::CurrentDirectory
                        | ManagerRowKey::NewWorkspace
                ) {
                    self.open_agent_picker(cx);
                    Some(Response::changed())
                } else {
                    None
                }
            }
            CMD_PRELUDE_BACKSPACE if self.route == Route::Prelude => {
                self.prelude.source_back();
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Prelude => {
                self.prelude.move_selection(true);
                Some(Response::changed())
            }
            CMD_NAV_UP if self.route == Route::Manager => {
                self.ensure_manager_rows();
                let current_index = self
                    .manager_rows_cache
                    .iter()
                    .position(|r| Some(r.key) == self.manager.list.cursor())
                    .unwrap_or(0);
                let prev_index = current_index.saturating_sub(1);
                if let Some(row) = self.manager_rows_cache.get(prev_index) {
                    self.manager.list.set_cursor(prev_index, row.key);
                    self.manager.select_row(row.domain.clone());
                }
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Manager => {
                self.ensure_manager_rows();
                let current_index = self
                    .manager_rows_cache
                    .iter()
                    .position(|r| Some(r.key) == self.manager.list.cursor())
                    .unwrap_or(0);
                let next_index =
                    (current_index + 1).min(self.manager_rows_cache.len().saturating_sub(1));
                if let Some(row) = self.manager_rows_cache.get(next_index) {
                    self.manager.list.set_cursor(next_index, row.key);
                    self.manager.select_row(row.domain.clone());
                }
                Some(Response::changed())
            }
            CMD_NAV_UP if self.route == Route::Editor => {
                cx.focus_prev();
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Editor => {
                cx.focus_next();
                Some(Response::changed())
            }
            CMD_PRELUDE_SPACE if self.route == Route::Manager => {
                if let ManagerRowKey::Workspace(workspace) = *self.manager.selected_row() {
                    self.manager.toggle(workspace);
                    self.ensure_manager_rows();
                }
                Some(Response::changed())
            }
            CMD_PRELUDE_SPACE if self.route == Route::Capsule && self.capsule_prefix => {
                Some(self.capsule_prefix_key(cx, ' '))
            }
            CMD_PRELUDE_SPACE if self.route == Route::Prelude => {
                if self.prelude.step() == 1 {
                    if cx
                        .state(crate::screens::prelude::READ_ONLY)
                        .contains(StateFlags::FOCUSED)
                    {
                        self.prelude_ui.read_only = !self.prelude_ui.read_only;
                    } else {
                        self.prelude.choose_source();
                        cx.focus(crate::screens::prelude::CONTINUE);
                    }
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
            CMD_PRELUDE_SPACE
                if self.route == Route::Editor && self.editor.tab == EditorTab::General =>
            {
                // Space arrives through the app capture binding (as on the
                // prelude), so the focused General checkbox flips here; the
                // component update below never sees the chord.
                if cx.update_cause() == UpdateCause::Event {
                    if cx
                        .state(crate::screens::editor::KEEP_AWAKE)
                        .contains(StateFlags::FOCUSED)
                    {
                        self.editor.pending.keep_awake = !self.editor.pending.keep_awake;
                        self.editor.mark_dirty();
                    } else if cx
                        .state(crate::screens::editor::GIT_PULL)
                        .contains(StateFlags::FOCUSED)
                    {
                        self.editor.pending.git_pull = !self.editor.pending.git_pull;
                        self.editor.mark_dirty();
                    }
                }
                Some(Response::changed())
            }
            CMD_NEW_WORKSPACE if self.route == Route::Manager => {
                self.route = Route::Prelude;
                self.prelude = PreludeState::default();
                self.prelude_ui = PreludeUiState::default();
                self.manager_menu_state = MenuState::default();
                self.manager_menu_open = false;
                cx.close_layer(MANAGER_MENU_BAR, None);
                cx.focus(crate::screens::prelude::FILE_LIST);
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
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_MOUNTS if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(2);
                }
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
                Some(Response::changed())
            }
            CMD_MOUNT_TOGGLE_RO
                if self.route == Route::Editor && self.editor.tab == EditorTab::Mounts =>
            {
                if let Some(mount) = self.editor.pending.mounts.first_mut() {
                    mount.readonly = !mount.readonly;
                    self.status = Some(format!(
                        "{} · {}",
                        mount.destination,
                        if mount.readonly {
                            "read-only"
                        } else {
                            "read-write"
                        }
                    ));
                }
                self.editor.mark_dirty();
                Some(Response::changed())
            }
            CMD_MOUNT_CYCLE_ISOLATION
                if self.route == Route::Editor && self.editor.tab == EditorTab::Mounts =>
            {
                if let Some(mount) = self.editor.pending.mounts.first_mut() {
                    if mount.running_isolated {
                        self.status = Some(format!(
                            "Cannot change isolation: a running instance holds isolated state for {}",
                            mount.destination
                        ));
                    } else {
                        mount.isolation = mount.isolation.next();
                        self.status = Some(format!(
                            "{} · isolation {}",
                            mount.destination,
                            mount.isolation.label().to_lowercase()
                        ));
                    }
                }
                self.editor.mark_dirty();
                Some(Response::changed())
            }
            CMD_EDITOR_ENV if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(4);
                }
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_PREVIOUS if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.previous_tab();
                    self.editor_accounts_transition = self.editor.tab == EditorTab::Accounts;
                }
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_ROLES if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(3);
                }
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_MOUNTS if self.route == Route::Settings => {
                self.settings_tab = 2;
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_ROLES if self.route == Route::Settings => {
                self.settings_tab = 3;
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_ENV if self.route == Route::Settings => {
                self.settings_tab = 4;
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_SETTINGS_TRUST_KEY if self.route == Route::Settings => {
                self.settings_tab = 5;
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_NAV_TAB_FIVE if self.route == Route::Settings => {
                self.settings_tab = 5;
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_NEXT if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event {
                    let active = self.settings_tab.saturating_sub(1);
                    self.settings_tab = (active + 1) % 5 + 1;
                    self.settings.focus = SettingsFocus::Tabs;
                }
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_EDITOR_PREVIOUS if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event {
                    let active = self.settings_tab.saturating_sub(1);
                    self.settings_tab = (active + 4) % 5 + 1;
                    self.settings.focus = SettingsFocus::Tabs;
                }
                cx.focus(crate::screens::settings::TABS);
                Some(Response::changed())
            }
            CMD_NAV_TAB_FIVE if self.route == Route::Editor => {
                if cx.update_cause() == UpdateCause::Event {
                    self.editor.select_alias(5);
                    self.editor_accounts_transition = true;
                }
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
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
                if self.editor.open_preview() && !cx.is_open(crate::screens::editor::PREVIEW) {
                    self.snapshot_preview_base(cx);
                    let (title, facts, code) = crate::screens::editor::EditorScreen::preview_parts(
                        &self.editor,
                        &self.world,
                    );
                    let pairs: Vec<(&str, &str)> = facts
                        .iter()
                        .map(|(label, value, _)| (label.as_str(), value.as_str()))
                        .collect();
                    let codes: Vec<&str> = code.iter().map(String::as_str).collect();
                    let dialog = Self::preview_dialog(&title, &pairs, &codes);
                    cx.open_layer(crate::screens::editor::PREVIEW, dialog.layer(cx));
                }
                self.status = Some("Save workspace · preview changes before commit".into());
                Some(Response::changed())
            }
            CMD_SAVE if self.route == Route::Settings => {
                self.request_settings_save(cx);
                Some(Response::changed())
            }
            CMD_PRELUDE_SPACE if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event
                    && self.settings_tab == 5
                    && self.settings.focus == SettingsFocus::Body
                    && !self.settings_save_preview
                    && let Some((source, trusted)) =
                        self.settings.toggle_trust(&self.world.global.trust)
                {
                    self.status = Some(format!(
                        "{} · {} · save to apply",
                        source,
                        if trusted { "trusted" } else { "untrusted" }
                    ));
                }
                Some(Response::changed())
            }
            CMD_NAV_UP if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event
                    && self.settings_tab == 5
                    && self.settings.focus == SettingsFocus::Body
                    && !self.settings_save_preview
                {
                    let len = self.world.global.trust.len();
                    self.settings.move_trust_cursor(-1, len);
                }
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Settings => {
                if cx.update_cause() == UpdateCause::Event
                    && self.settings_tab == 5
                    && self.settings.focus == SettingsFocus::Body
                    && !self.settings_save_preview
                {
                    let len = self.world.global.trust.len();
                    self.settings.move_trust_cursor(1, len);
                }
                Some(Response::changed())
            }
            CMD_NAV_UP if self.route == Route::Accounts && !self.accounts.form_open => {
                self.move_accounts_cursor(-1);
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Accounts && !self.accounts.form_open => {
                self.move_accounts_cursor(1);
                self.accounts_down_count += 1;
                Some(Response::changed())
            }
            CMD_NAV_DOWN if self.route == Route::Usage => {
                self.usage.move_account(&self.world, 1);
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
                    // Unbound legacy notices cannot authorize an editor write.
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
                crate::sim::world::Msg::ManagerOperation { .. } => {
                    // The captured Manager reducer owns and consumes operation
                    // identities; no manager session is wired into the app yet,
                    // so nothing can have scheduled one here.
                }
            }
        }

        match self.route {
            Route::Intro => {
                if self.intro.advance_tick() && self.intro.is_done() {
                    self.route = Route::Manager;
                    self.reset_manager_cursor();
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
}

fn truncate_middle(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max {
        return s.to_owned();
    }
    if max < 5 {
        return s.chars().take(max).collect();
    }
    let keep_end = (max.saturating_sub(1)) / 3;
    let keep_start = max.saturating_sub(1).saturating_sub(keep_end);
    let head: String = s.chars().take(keep_start).collect();
    let tail: String = s
        .chars()
        .skip(char_count.saturating_sub(keep_end))
        .collect();
    format!("{head}…{tail}")
}

/// The host and cockpit strips. Gap 2 is the tag segment separator.
/// The container fill matches the row already painted behind the strip.
fn draw_header_status(
    ui: &mut Ui<'_>,
    id: Id,
    area: Rect,
    left: &[StatusItem<'_>],
    right: &[StatusItem<'_>],
    dim: bool,
) {
    const CANVAS: [(Part, StylePatch); 1] = [(
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    )];
    const DIM: [(Part, StylePatch); 1] = [(
        Part::CONTAINER,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Muted))
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD),
    )];
    let bar = StatusBar::new(id).gap(2).left(left).right(right);
    if dim {
        bar.patch_part(&DIM).draw(ui, area);
    } else {
        bar.patch_part(&CANVAS).draw(ui, area);
    }
}

impl App {
    /// Tag `row_status` for the cursor mount: what changed, or the hidden
    /// source column at narrow widths.
    fn editor_mount_row_status(&self, full_width: u16) -> Option<String> {
        let m = self.editor.pending.mounts.first()?;
        let o = self.editor.original_mount(&m.destination);
        match o {
            Some(o) if o != m => {
                let mut s = format!(
                    "was {} · {}",
                    o.mode_label(),
                    o.isolation.label().to_lowercase()
                );
                if o.source != m.source {
                    s.push_str(&format!(" · {}", self.world.tilde(m.source_label())));
                }
                Some(s)
            }
            None => Some("new mount".into()),
            _ => {
                let avail = full_width.saturating_sub(9);
                if avail < 90 {
                    Some(format!("source {}", self.world.tilde(m.source_label())))
                } else {
                    None
                }
            }
        }
    }

    fn draw_host_menu(&self, ui: &mut Ui<'_>, area: Rect) {
        let palette = HistoricalPalette::new(ui);
        if self.route == Route::Prelude {
            let canvas = self.historical_span_style((128, 128, 128), (0, 0, 0), false);
            ui.fill(area, canvas);
            Brand::new(APP.sub("brand"), "jackin❯")
                .patch_part(&Self::PRELUDE_BRAND_DIM)
                .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
            self.host_menu_bar()
                .patch_part(&Self::PRELUDE_MENU_DIM)
                .draw(
                    ui,
                    Rect::new(area.x.saturating_add(11), area.y, 20, 1),
                    &self.manager_menu_state,
                );

            let rest_x = area.x.saturating_add(31);
            let rest_w = area.right().saturating_sub(rest_x);
            if rest_w > 0 {
                let faint = Role::Fg(FgStep::Faint);
                let segs = [
                    StatusItem::new("Workspaces › new workspace")
                        .tone(faint)
                        .priority(7),
                    StatusItem::new("inside the Construct")
                        .tone(faint)
                        .priority(6),
                    StatusItem::new("no instances")
                        .tone(Role::Fg(FgStep::Ghost))
                        .priority(5),
                ];
                draw_header_status(
                    ui,
                    HOST_HEADER,
                    Rect::new(rest_x, area.y, rest_w, 1),
                    &[],
                    &segs,
                    true,
                );
            }
            return;
        }

        ui.fill(area, palette.primary_on_canvas);

        // The account form dims this row afterwards (`dim_layer` on the full
        // screen). Painting the dimmed colours here would step them twice.
        let _ = Brand::new(APP.sub("brand"), "jackin❯")
            .clickable(true)
            .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
        let menu_area = Rect::new(area.x.saturating_add(11), area.y, 20, 1);
        self.host_menu_bar()
            .draw(ui, menu_area, &self.manager_menu_state);

        // Tag `draw_host_menu`: the breadcrumb strip starts at `used + 2`
        // past the menu labels; this bar's box already ends one cell past
        // the last label, so one more cell lands on the same origin.
        let rest_x = menu_area.right().saturating_add(1);
        let rest_w = area.right().saturating_sub(rest_x);
        if rest_w > 0 {
            let manager_crumb: String;
            let crumb = match self.route {
                Route::Manager => {
                    manager_crumb = match self.manager.selected_row() {
                        ManagerRowKey::Workspace(id) => self
                            .world
                            .workspace(*id)
                            .map(|w| format!("Workspaces › {}", w.name))
                            .unwrap_or_else(|| "Workspaces".into()),
                        ManagerRowKey::Instance(id) => self
                            .world
                            .instance(id)
                            .map(|i| {
                                let ws_name = i
                                    .workspace
                                    .and_then(|wid| self.world.workspace(wid))
                                    .map_or("workspace", |w| w.name.as_str());
                                format!(
                                    "Workspaces › {} › {}",
                                    ws_name,
                                    i.id.trim_start_matches("jk-")
                                )
                            })
                            .unwrap_or_else(|| "Workspaces".into()),
                        _ => "Workspaces".into(),
                    };
                    manager_crumb.as_str()
                }
                Route::Accounts => {
                    manager_crumb =
                        crate::screens::accounts::crumb(&self.world, &self.accounts.selected);
                    manager_crumb.as_str()
                }
                Route::Usage => {
                    manager_crumb = crate::screens::usage::crumb(&self.world, &self.usage);
                    manager_crumb.as_str()
                }
                Route::Settings => {
                    manager_crumb = SettingsScreen::crumb(self.settings_tab);
                    manager_crumb.as_str()
                }
                Route::Editor => {
                    let ws_name = self
                        .editor
                        .workspace_id()
                        .and_then(|id| self.world.workspace(id))
                        .map_or_else(|| self.editor.pending.name.as_str(), |w| w.name.as_str());
                    manager_crumb = format!("Workspaces › {} › edit", ws_name);
                    manager_crumb.as_str()
                }
                Route::Prelude => "Create",
                _ => "",
            };
            // Tag `construct_state`: entry completion counts even when no
            // instance is running, so the post-intro manager is inside.
            let state = if self.world.arbiter.entered_at_ms.is_some()
                || self.world.running_count() > 0
                || self.route == Route::Capsule
            {
                "inside the Construct"
            } else {
                "outside the Construct"
            };
            // Tag header: the arbiter's discovery replaces the count when
            // it fails (`! {label}`, error tone), and a stale daemon adds
            // its warning badge after the count segment.
            let running_text: String;
            let running_tone;
            let running_priority;
            match self.world.arbiter.running() {
                Ok(0) => {
                    running_text = "no instances".to_owned();
                    running_tone = Role::Fg(FgStep::Muted);
                    running_priority = 5;
                }
                Ok(n) => {
                    running_text = format!("{n} running");
                    running_tone = Role::Fg(FgStep::Muted);
                    running_priority = 5;
                }
                Err(err) => {
                    running_text = format!("! {}", err.label());
                    running_tone = Role::Danger;
                    running_priority = 8;
                }
            }

            let chrome = Role::Fg(FgStep::Secondary);
            let mut segs = Vec::new();
            if !crumb.is_empty() {
                segs.push(StatusItem::new(crumb).tone(chrome).priority(7));
            }
            let refreshing_text;
            if self.route == Route::Accounts {
                let n = crate::screens::accounts::refreshing_count(&self.world);
                if n > 0 {
                    refreshing_text = format!(
                        "{} refreshing {n}",
                        crate::screens::accounts::spinner_frame(self.world.now_ms() as u64 / 80)
                    );
                    segs.push(StatusItem::new(&refreshing_text).tone(chrome).priority(6));
                }
            }
            let change_text;
            let settings_change_text;
            if self.route == Route::Settings && self.settings_change_count() > 0 {
                let n = self.settings_change_count();
                let noun = if n == 1 { "change" } else { "changes" };
                settings_change_text = format!("• {n} {noun}");
                segs.push(
                    StatusItem::new(&settings_change_text)
                        .tone(Role::Warning)
                        .priority(8),
                );
            }
            if self.route == Route::Editor && self.editor.change_count() > 0 {
                let n = self.editor.change_count();
                let noun = if n == 1 { "change" } else { "changes" };
                change_text = format!("• {n} {noun}");
                segs.push(
                    StatusItem::new(&change_text)
                        .tone(Role::Warning)
                        .priority(8),
                );
            }
            let row_status_text;
            if self.route == Route::Editor
                && self.editor.tab == EditorTab::Mounts
                && let Some(status) = self.editor_mount_row_status(area.width)
            {
                row_status_text = status;
                segs.push(
                    StatusItem::new(&row_status_text)
                        .tone(Role::Fg(FgStep::Muted))
                        .priority(3),
                );
            }
            segs.push(
                StatusItem::new(state)
                    .tone(chrome)
                    .priority(6)
                    .key(ItemKey::text("construct")),
            );
            segs.push(
                StatusItem::new(&running_text)
                    .tone(running_tone)
                    .priority(running_priority),
            );
            if self.world.daemon_health == DaemonHealth::Stale {
                segs.push(
                    StatusItem::new("▲ daemon stale")
                        .tone(Role::Warning)
                        .priority(8),
                );
            }

            draw_header_status(
                ui,
                HOST_HEADER,
                Rect::new(rest_x, area.y, rest_w, 1),
                &[],
                &segs,
                false,
            );
        }
    }

    fn draw_strip(&self, ui: &mut Ui<'_>, area: Rect) {
        let _ = Brand::new(APP.sub("brand"), "jackin❯")
            .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
        let state = "entering the Construct";
        let n = self.world.running_count();
        let running_text = if n == 0 {
            "no instances".to_owned()
        } else {
            format!("{n} running")
        };
        let workspace = self
            .world
            .workspaces
            .first()
            .map_or("payments-platform", |w| w.name.as_str());
        let crumb = format!("Launch › {workspace} › the-architect");

        let stage_text = if let Some(run) = &self.launch {
            let (done, _) = run.counts();
            format!("{done}/11 stages")
        } else {
            "2/11 stages".to_owned()
        };

        let primary = Role::Fg(FgStep::Primary);
        let secondary = Role::Fg(FgStep::Secondary);
        let muted = Role::Fg(FgStep::Muted);
        let left_segs = [
            StatusItem::new(state)
                .tone(primary)
                .priority(9)
                .key(ItemKey::text("entering")),
            StatusItem::new(&running_text).tone(secondary).priority(8),
            StatusItem::new(&crumb).tone(secondary).priority(7),
        ];
        // The help run is clickable in the tag, so its muted style includes
        // the spaces on either side. Dim then walks those spaces to ghost.
        let right_segs = [
            StatusItem::new(&stage_text).tone(muted).priority(6),
            StatusItem::new("? help").tone(muted).priority(4).padded(),
        ];

        // One gutter of inset, so the left fact still starts where the old
        // strip started and the right fact still ends one cell in.
        let strip_x = area.x.saturating_add(11).saturating_sub(1);
        let strip_w = area.right().saturating_sub(strip_x);
        if strip_w > 0 {
            draw_header_status(
                ui,
                ROUTE_STRIP,
                Rect::new(strip_x, area.y, strip_w, 1),
                &left_segs,
                &right_segs,
                false,
            );
        }
    }

    fn draw_intro(&self, ui: &mut Ui<'_>, area: Rect) {
        jackin_preview_presentation::rain::render_intro_ui(ui, area, &self.intro);
    }

    pub(super) fn historical_span_style(
        &self,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
        bold: bool,
    ) -> PaintStyle {
        use termrock::author::{Color, Modifier, Style};
        let mut s = Style::default()
            .fg(Color::Rgb(fg.0, fg.1, fg.2))
            .bg(Color::Rgb(bg.0, bg.1, bg.2));
        if bold {
            s = s.add_modifier(Modifier::BOLD);
        }
        s.into()
    }

    fn draw_prelude(&self, ui: &mut Ui<'_>, _area: Rect) {
        let full = ui.full();
        let bg = self.historical_span_style((128, 128, 128), (0, 0, 0), false);
        let stage = Rect::new(
            full.x,
            full.y.saturating_add(1),
            full.width,
            full.height.saturating_sub(2),
        );
        ui.fill(stage, bg);
        crate::screens::prelude::PreludeScreen::draw(
            ui,
            full,
            &self.prelude,
            self.prelude_ui.read_only,
        );
    }

    /// Tag `draw_frame` fills the whole frame with the base pair first
    /// and screens overpaint their regions; without this, unpainted
    /// cells carry `Default` instead of the explicit base style.
    fn paint_base_frame(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(area, HistoricalPalette::new(ui).primary_on_canvas);
    }

    fn draw_editor(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = !self.help_open && !self.editor.preview_open;
        crate::screens::editor::EditorScreen::draw(ui, area, &self.editor, &self.world, focused);
        if self.editor.exit_open {
            crate::screens::editor::EditorScreen::draw_exit_dialog(ui, area, &self.editor);
        }
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
        let detail = "The daemon owns pane state; the shell owns the handoff.";
        let lines = [label.as_str(), detail];
        paint_owned_lines(ui, HANDOFF_LINES, area, &lines);
    }

    fn draw_outro(&self, ui: &mut Ui<'_>, area: Rect) {
        if let Some(outro) = &self.outro {
            jackin_preview_presentation::rain::render_outro_ui(ui, area, outro);
        } else {
            let outro = OutroState::new(self.motion, None, 0);
            jackin_preview_presentation::rain::render_outro_ui(ui, area, &outro);
        }
    }

    fn draw_manager(&self, ui: &mut Ui<'_>, area: Rect) {
        // The manager keeps focus while its menu is open: the frozen
        // menu-open frame shows the focused workspace (strong border,
        // bright title, selected-row tint) with the dropdown above it,
        // and the menu's own rows inherit the focused underpaint.
        let focused = !self.manager_quit_confirm
            && !self.help_open
            && !self.manager_inspect_open
            && !self.manager_launch_picker_open();
        crate::screens::manager::ManagerScreen::draw(ui, area, &self.manager, &self.world, focused);
    }

    fn draw_accounts(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = !self.help_open && !self.accounts.form_open;
        crate::screens::accounts::AccountsScreen::draw(
            ui,
            area,
            &self.accounts,
            &self.world,
            focused,
            false,
        );
        if self.accounts.form_open {
            // Two steps walks primary text to muted and muted text to ghost.
            // The whole screen, including the header and the side margins, is
            // drawn at full strength first so the dim is applied once. The
            // form is painted after, so its own cells stay at full strength.
            ui.dim_layer(ui.full(), 2);
            self.draw_account_form(ui, area);
        }
        if self.accounts_filtering {
            TextInput::new(ACCOUNTS_FILTER_INPUT)
                .value(&self.accounts.filter_draft)
                .placeholder("filter")
                .draw(
                    ui,
                    Rect::new(area.x, area.y, area.width.max(1), 1),
                    &self.accounts.filter_input,
                );
        }
    }

    /// Centered account form. 70×30 when the screen can hold it; otherwise
    /// it keeps a two-column margin and sits between the header and footer.
    fn draw_account_form(&self, ui: &mut Ui<'_>, area: Rect) {
        let _ = area;
        let full = ui.full();
        let max_h = full.height.saturating_sub(2);
        let max_w = full.width.saturating_sub(4);
        let width = 70.min(max_w);
        let height = 30.min(max_h);
        let panel_area = Rect::new(
            full.x + (full.width.saturating_sub(width)) / 2,
            if height == max_h {
                full.y.saturating_add(1)
            } else {
                full.y + (full.height.saturating_sub(height)) / 2
            },
            width,
            height,
        );
        let name_error = (self.accounts_form_enters >= 2).then_some("Required");
        let sources = [
            "1Password item / field  (recommended)",
            "Local agent folder",
            "Plain-text API key",
        ];
        let providers = ["Claude Code · Anthropic / Claude"];
        // Empty form cells stay muted on the elevated surface. A focused
        // frame supplies the strong border and the bold title; clearing the
        // gutter keeps the title rule a plain `─`.
        const FORM_SURFACE: [(Part, StylePatch); 2] = [
            (
                Part::CONTAINER,
                StylePatch::new().set_bg(Role::Surface(Surface::Elevated)),
            ),
            (
                Part::GUTTER,
                StylePatch {
                    glyph: Slot::Clear,
                    ..StylePatch::new()
                },
            ),
        ];
        ui.with_surface(Surface::Elevated, |ui| {
            Panel::new(crate::screens::accounts::FORM)
                .kind(PanelKind::Framed)
                .title("New account")
                .meta("form · unsaved")
                .focused(true)
                .patch_part(&FORM_SURFACE)
                .inner_inset(Insets {
                    l: 2,
                    t: 1,
                    r: 1,
                    b: 1,
                })
                .draw(ui, panel_area, |ui, inner| {
                    // The natural body is 24 rows. Two blank rows and the
                    // button row stay below the scroller. A shorter panel
                    // clips the body and shows the scrollbar in that column.
                    const FORM_ROWS: u16 = 24;
                    const FORM_SCROLL: [(Part, StylePatch); 2] = [
                        (
                            Part::TRACK,
                            StylePatch::new()
                                .set_fg(Role::Fg(FgStep::Ghost))
                                .set_bg(Role::Surface(Surface::Elevated)),
                        ),
                        (
                            Part::THUMB,
                            StylePatch::new()
                                .set_fg(Role::Fg(FgStep::Primary))
                                .set_bg(Role::Surface(Surface::Elevated)),
                        ),
                    ];
                    let viewport_h = inner.height.saturating_sub(3);
                    let overflows = viewport_h < FORM_ROWS;
                    let viewport = Rect::new(inner.x, inner.y, inner.width, viewport_h);
                    let body = if overflows {
                        let scroll = ScrollState::new(usize::from(FORM_ROWS));
                        ScrollRegion::new(crate::screens::accounts::FORM.sub("body"))
                            .fill_container(false)
                            .focused(true)
                            .patch_part(&FORM_SCROLL)
                            .draw(ui, viewport, &scroll, usize::from(FORM_ROWS))
                    } else {
                        viewport
                    };
                    let field_w = body.width.saturating_sub(if overflows { 1 } else { 2 });
                    let mark_inset = if overflows { 3 } else { 4 };
                    ui.with_area(body, |ui| {
                    Field::new(
                        "Display name",
                        TextInput::new(crate::screens::accounts::NAME)
                            .placeholder("Personal, Work, Team…")
                            .value(&self.accounts.draft_name),
                    )
                    .required(true)
                    .error(name_error)
                    .patch_part(&[(
                        Part::MARKER,
                        StylePatch::new().set_fg(Role::Accent),
                    )])
                    .draw(
                        ui,
                        Rect::new(body.x, body.y, field_w, 3),
                        &self.accounts.name_input,
                    );
                    if name_error.is_some() {
                        // The input's own marker sits on its last cell. This
                        // form keeps one field cell between that mark and the
                        // frame, so the danger mark is painted one cell in.
                        let bang = ui.paint_patch(
                            &StylePatch::new()
                                .set_fg(Role::Danger)
                                .set_bg(Role::Surface(Surface::Field))
                                .add(Modifier::BOLD),
                        );
                        ui.paint_str(
                            Rect::new(
                                body.x.saturating_add(body.width.saturating_sub(mark_inset)),
                                body.y.saturating_add(1),
                                1,
                                1,
                            ),
                            "!",
                            bang,
                        );
                    }
                    let purpose = TextInputState::default();
                    Field::new(
                        "Purpose label",
                        TextInput::new(crate::screens::accounts::FORM.sub("purpose"))
                            .placeholder("personal · work · experiments"),
                    )
                    .draw(
                        ui,
                        Rect::new(
                            body.x,
                            body.y.saturating_add(4),
                            field_w,
                            3,
                        ),
                        &purpose,
                    );
                    let mut provider = SelectState::default();
                    provider.set_value(Some(ItemKey::index(0)));
                    Field::new(
                        "Provider",
                        SelectField::new(
                            Select::new(crate::screens::accounts::FORM.sub("provider")),
                            &providers,
                        ),
                    )
                    .optional_suffix(false)
                    .draw(
                        ui,
                        Rect::new(
                            body.x,
                            body.y.saturating_add(8),
                            field_w,
                            3,
                        ),
                        &provider,
                    );
                    let runtime_w = 62.min(body.width.saturating_sub(4));
                    let runtime = termrock::truncate(
                        "Agent runtime Claude Code · provider Anthropic / Claude · usage follows the account",
                        runtime_w,
                    );
                    let muted = ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Muted))
                            .set_bg(Role::Surface(Surface::Elevated)),
                    );
                    let secondary = ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Secondary))
                            .set_bg(Role::Surface(Surface::Elevated)),
                    );
                    let primary = ui.paint_patch(
                        &StylePatch::new()
                            .set_fg(Role::Fg(FgStep::Primary))
                            .set_bg(Role::Surface(Surface::Elevated)),
                    );
                    ui.paint_str(
                        Rect::new(
                            body.x.saturating_add(2),
                            body.y.saturating_add(12),
                            runtime_w,
                            1,
                        ),
                        &runtime,
                        muted,
                    );
                    ui.paint_str(
                        Rect::new(
                            body.x.saturating_add(2),
                            body.y.saturating_add(14),
                            body.width,
                            1,
                        ),
                        "Credential source",
                        secondary,
                    );
                    let radio = RadioGroupState::default();
                    RadioGroup::new(crate::screens::accounts::SOURCE)
                        .value(ItemKey::index(0))
                        .draw(
                            ui,
                            Rect::new(
                                body.x,
                                body.y.saturating_add(15),
                                field_w,
                                3,
                            ),
                            &radio,
                            &sources,
                        );
                    ui.paint_str(
                        Rect::new(
                            body.x.saturating_add(2),
                            body.y.saturating_add(19),
                            body.width,
                            1,
                        ),
                        "1Password reference",
                        secondary,
                    );
                    ui.paint_str(
                        Rect::new(
                            body.x.saturating_add(2),
                            body.y.saturating_add(20),
                            body.width,
                            1,
                        ),
                        "not chosen",
                        primary,
                    );
                    let choose_w = termrock::width("Choose…") + 2;
                    Button::new(crate::screens::accounts::FORM.sub("choose"), "Choose…").draw(
                        ui,
                        Rect::new(
                            inner.right().saturating_sub(choose_w).saturating_sub(3),
                            body.y.saturating_add(20),
                            choose_w,
                            1,
                        ),
                    );
                    });
                    if overflows {
                        let mut scroll = ScrollState::new(usize::from(FORM_ROWS));
                        scroll.set_viewport(usize::from(viewport_h));
                        ui.scroll_edges(body, &scroll);
                        // A foreground already equal to the row cannot blend.
                        // Reduced palettes still mark that outer-edge cell DIM.
                        if ui.theme_ref().capability.color != termrock::ColorLevel::TrueColor {
                            let y = body.bottom().saturating_sub(1);
                            let xs: Vec<u16> = {
                                let (buf, _) = ui.peek();
                                (body.x.saturating_sub(2)..body.right())
                                    .filter(|&x| {
                                        buf.cell(Position::new(x, y)).is_some_and(|cell| {
                                            cell.fg == cell.bg
                                                && !cell.modifier.contains(Modifier::DIM)
                                        })
                                    })
                                    .collect()
                            };
                            if !xs.is_empty() {
                                let dim = ui.paint_patch(
                                    &StylePatch::new()
                                        .set_fg(Role::Surface(Surface::Elevated))
                                        .set_bg(Role::Surface(Surface::Elevated))
                                        .add(Modifier::DIM),
                                );
                                for x in xs {
                                    ui.paint_str(Rect::new(x, y, 1, 1), " ", dim);
                                }
                            }
                        }
                    }
                    let buttons_y = inner.bottom().saturating_sub(1);
                    let mut x = inner
                        .x
                        .saturating_add(12)
                        .saturating_sub(70u16.saturating_sub(width));
                    let buttons = [
                        (
                            crate::screens::accounts::FORM.sub("plain"),
                            "Enter plain text instead",
                            Variant::GHOST,
                        ),
                        (
                            crate::screens::accounts::FORM.sub("validate"),
                            "Validate",
                            Variant::DEFAULT,
                        ),
                        (
                            crate::screens::accounts::FORM.sub("cancel"),
                            "Cancel",
                            Variant::GHOST,
                        ),
                        (crate::screens::accounts::SAVE, "Save", Variant::PRIMARY),
                    ];
                    for (id, label, variant) in buttons {
                        let drawn = Button::new(id, label).variant(variant).draw(
                            ui,
                            Rect::new(x, buttons_y, inner.right().saturating_sub(x), 1),
                        );
                        x = drawn.right().saturating_add(1);
                    }
                });
            // The left rule is outside the panel callback's clip. The body
            // fade still reaches that rule and the padding beside it.
            let edge_h = panel_area.height.saturating_sub(5);
            if edge_h < 24 {
                let mut scroll = ScrollState::new(24);
                scroll.set_viewport(usize::from(edge_h));
                ui.scroll_edges(
                    Rect::new(panel_area.x, panel_area.y.saturating_add(1), 2, edge_h),
                    &scroll,
                );
            }
        });
    }

    fn draw_usage(&self, ui: &mut Ui<'_>, area: Rect) {
        let focused = !self.help_open;
        crate::screens::usage::UsageScreen::draw(ui, area, &self.usage, &self.world, focused);
    }

    fn draw_settings(&self, ui: &mut Ui<'_>, _area: Rect) {
        SettingsScreen::draw(
            ui,
            self.settings_tab,
            &self.settings,
            &self.world,
            self.settings_save_preview,
        );
    }

    fn draw_launch(&self, ui: &mut Ui<'_>, area: Rect) {
        crate::screens::cockpit::CockpitScreen::draw(
            ui,
            area,
            &self.cockpit,
            &self.world,
            self.selected_role(),
            self.cockpit_debug_open,
            self.launch.as_ref(),
        );
        if self.cockpit_failure_open
            && let Some(run) = self.launch.as_ref()
            && let Some(failure) = run.failure.as_ref()
        {
            let run_id = self.failure_run_label(run);
            let ws_name = self
                .world
                .workspaces
                .first()
                .map_or("payments-platform", |workspace| workspace.name.as_str());
            crate::screens::cockpit::CockpitScreen::draw_failure(
                ui,
                area,
                failure,
                &run_id,
                self.selected_role(),
                ws_name,
                &run.container,
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

    fn draw_capsule_panes(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.fill(area, ui.surface_style());
        let instance_id = Self::active_running_instance_id_ref(&self.active_instance, &self.world);
        let Some(instance_id) = instance_id else {
            Empty::new(
                CAPSULE_PANES_EMPTY,
                EmptyState::Empty {
                    title: "Capsule is empty",
                    hint: None,
                },
            )
            .draw(ui, area);
            return;
        };
        let Some(daemon) = self.world.daemons.get(instance_id) else {
            Empty::new(
                CAPSULE_PANES_EMPTY,
                EmptyState::Empty {
                    title: "Daemon unavailable",
                    hint: None,
                },
            )
            .draw(ui, area);
            return;
        };
        let Some(tab) = daemon.active_tab() else {
            Empty::new(
                CAPSULE_PANES_EMPTY,
                EmptyState::Empty {
                    title: "No sessions",
                    hint: None,
                },
            )
            .draw(ui, area);
            return;
        };
        let framed = tab.leaf_count() > 1 || tab.zoomed.is_some();
        {
            let mut frame = self.capsule_frame.borrow_mut();
            Self::ensure_tab_projections(&mut frame, daemon, &self.world.accounts, &tab.root);
        }
        let ctx = CapsulePaneCtx {
            daemon,
            tab,
            framed,
        };
        if let Some(zoomed) = tab.zoomed {
            self.draw_capsule_leaf(ui, &ctx, zoomed, area);
            return;
        }
        self.draw_capsule_node(ui, &ctx, &tab.root, area, 1, 0);
    }

    /// Stable per-split component id from the node's heap-indexed tree path
    /// (root 1, children 2n/2n+1).
    ///
    /// Fixture trees are a handful of levels deep; the fold saturates past
    /// depth 60 so ids stay total even for absurd inputs.
    fn capsule_split_id(path_bits: u64) -> Id {
        CAPSULE_SPLIT.item(ItemKey::num(path_bits))
    }

    /// Draw one pane-tree node: splits own their seam through [`SplitPane`]
    /// (draw-only, like the manager seam: no resize behavior changes), leaves
    /// draw through [`Self::draw_capsule_leaf`].
    ///
    /// `SplitPaneState` is derived per frame from the sim split, so the sim
    /// stays the single source of truth for percents and maximize state.
    /// The shared split-model arithmetic is identical to the sim split
    /// arithmetic, hence the produced rects match the previous flat layout
    /// exactly.
    fn draw_capsule_node(
        &self,
        ui: &mut Ui<'_>,
        ctx: &CapsulePaneCtx<'_>,
        node: &PaneNode,
        area: Rect,
        path_bits: u64,
        depth: u8,
    ) {
        let PaneNode::Split {
            dir,
            split,
            first,
            second,
        } = node
        else {
            let PaneNode::Leaf(pane_id) = node else {
                return;
            };
            self.draw_capsule_leaf(ui, ctx, *pane_id, area);
            return;
        };
        let axis = match dir {
            SplitDir::Horizontal => SplitAxis::Horizontal,
            SplitDir::Vertical => SplitAxis::Vertical,
        };
        let mut state = SplitPaneState::new(u8::try_from(split.percent.clamp(5, 95)).unwrap_or(95));
        match split.maximized {
            Maximized::None => {}
            Maximized::First => {
                state.toggle_max(termrock::Maximized::First);
            }
            Maximized::Second => {
                state.toggle_max(termrock::Maximized::Second);
            }
        }
        let split_id = Self::capsule_split_id(path_bits);
        let (first_bits, second_bits, next_depth) = if depth >= 60 {
            (path_bits, path_bits, depth)
        } else {
            (
                path_bits.saturating_mul(2),
                path_bits.saturating_mul(2).saturating_add(1),
                depth.saturating_add(1),
            )
        };
        SplitPane::new(split_id, axis)
            .gap(1)
            .min_first(split.min_first)
            .min_second(split.min_second)
            .draw(ui, area, &state, |ui, first_area, second_area| {
                self.draw_capsule_node(ui, ctx, first, first_area, first_bits, next_depth);
                self.draw_capsule_node(ui, ctx, second, second_area, second_bits, next_depth);
            });
    }

    /// Style the focused pane's state glyph like the status row styles the
    /// same agent state: only a blocked pane raises the warning color.
    fn capsule_title_glyph_focused(state: crate::domain::instance::AgentState) -> Role {
        match state {
            crate::domain::instance::AgentState::Working => Role::Fg(FgStep::Secondary),
            crate::domain::instance::AgentState::Blocked => Role::Warning,
            crate::domain::instance::AgentState::Done => Role::Fg(FgStep::Secondary),
            crate::domain::instance::AgentState::Idle => Role::Fg(FgStep::Muted),
            crate::domain::instance::AgentState::Unknown => Role::Fg(FgStep::Secondary),
        }
    }

    /// Draw one leaf pane: a framed [`Panel`] with a title part slot, the
    /// transcript [`TextViewport`], and the command [`TextInput`] row.
    ///
    /// The input row is reserved only while the input shows something (an
    /// edit in progress or committed text); an idle empty input draws
    /// underneath the full-height viewport so its focus target settles
    /// without covering content. Unframed single panes draw content directly.
    fn draw_capsule_leaf(
        &self,
        ui: &mut Ui<'_>,
        ctx: &CapsulePaneCtx<'_>,
        pane_id: PaneId,
        pane_area: Rect,
    ) {
        if pane_area.width < 4 || pane_area.height < 3 {
            return;
        }
        let Some(pane) = ctx.daemon.pane(pane_id) else {
            return;
        };
        let focused = ctx.tab.focused == pane_id;
        let frame = self.capsule_frame.borrow();
        let Some((_, projected)) = frame.transcripts.get(&pane_id) else {
            return;
        };
        let input_shown =
            focused && (self.capsule_input_state.is_editing() || !self.capsule_input.is_empty());
        let draw_content = |ui: &mut Ui<'_>, inner: Rect| {
            if inner.is_empty() {
                return;
            }
            if focused {
                Self::capsule_input().value(&self.capsule_input).draw(
                    ui,
                    Rect::new(inner.x, inner.bottom().saturating_sub(1), inner.width, 1),
                    &self.capsule_input_state,
                );
            }
            let viewport_area = Rect {
                height: inner.height.saturating_sub(u16::from(input_shown)),
                ..inner
            };
            let state = self
                .capsule_viewports
                .get(&pane_id)
                .cloned()
                .unwrap_or_default();
            Self::capsule_viewport(pane_id).draw_projected(ui, viewport_area, &state, projected);
        };
        if !ctx.framed {
            draw_content(ui, pane_area);
            return;
        }
        let Some((_, (label_run, glyph))) = frame.titles.get(&pane_id) else {
            return;
        };
        let title_patch = if focused {
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas))
                .add(Modifier::BOLD)
        } else {
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Secondary))
                .set_bg(Role::Surface(Surface::Canvas))
                .remove(Modifier::BOLD)
        };
        let icon_fg = if focused {
            Self::capsule_title_glyph_focused(pane.state())
        } else {
            Role::Fg(FgStep::Secondary)
        };
        let icon_patch = StylePatch::new()
            .set_fg(icon_fg)
            .set_bg(Role::Surface(Surface::Canvas))
            .remove(Modifier::BOLD);
        let border_fg = if focused {
            Role::BorderStrong
        } else {
            Role::BorderSubtle
        };
        let parts = [
            (
                Part::BORDER,
                StylePatch::new().set_fg(border_fg).remove(Modifier::BOLD),
            ),
            (
                Part::DETAIL,
                StylePatch::new().set_fg(border_fg).remove(Modifier::BOLD),
            ),
            (Part::TITLE, title_patch),
            (Part::ICON, icon_patch),
        ];
        let mut panel = Panel::new(Self::capsule_viewport_id(pane_id))
            .kind(PanelKind::Framed)
            .title(label_run.as_str())
            .title_flush(true)
            .title_mark(glyph)
            .inner_inset(Insets::all(1))
            .patch_part(&parts);
        if ctx.tab.zoomed == Some(pane_id) {
            panel = panel.meta("zoomed");
        }
        panel.draw(ui, pane_area, |ui, inner| {
            draw_content(ui, inner);
        });
    }

    fn draw_capsule(&self, ui: &mut Ui<'_>, area: Rect) {
        let palette = HistoricalPalette::new(ui);
        ui.fill(area, palette.primary_on_canvas);
        let has_daemon = self
            .active_running_instance_id()
            .and_then(|id| self.world.daemons.get(&id))
            .is_some();
        if !has_daemon {
            return;
        }
        Self::ensure_capsule_tabs(
            &mut self.capsule_frame.borrow_mut(),
            &self.world,
            Self::active_running_instance_id_ref(&self.active_instance, &self.world),
            &self.capsule_tab_title,
            self.capsule_tab_title_index,
        );
        {
            let frame = self.capsule_frame.borrow();
            let tabs = frame
                .tabs
                .as_ref()
                .map_or(&[] as &[CapsuleTab], |(_, tabs)| tabs.as_slice());
            // The frozen strip starts one cell in and ends one cell short on
            // every geometry; the shell fill shows through on both sides.
            let tab_area = Rect {
                x: area.x.saturating_add(1),
                width: area.width.saturating_sub(2),
                height: area.height.min(2),
                ..area
            };
            Tabs::new(CAPSULE_TABS).row(paint_capsule_tab).draw(
                ui,
                tab_area,
                &self.tabs_state,
                tabs,
            );
        }
        let pane_area = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(2),
            ..area
        };
        self.draw_capsule_panes(ui, pane_area);
        if self.capsule_tab_title_dialog {
            paint_owned_lines(
                ui,
                APP.sub("capsule.rename"),
                Rect::new(area.x.saturating_add(2), area.y.saturating_add(1), 30, 1),
                &["Change tab title"],
            );
        }
        if self.inspect.instance.is_some() {
            let inspect_area = Rect::new(
                area.x.saturating_add(2),
                area.y.saturating_add(2),
                area.width.saturating_sub(4),
                area.height.saturating_sub(4),
            );
            let lines: &[&str] = if self.inspect_files {
                &[
                    "Inspect changes · files",
                    "src/main.rs",
                    "src/lib.rs",
                    "Tab · open diff",
                ]
            } else if self.inspect_detail {
                &[
                    "Inspect changes · src/main.rs",
                    "@@ -1,4 +1,4 @@",
                    "│ - old configuration",
                    "│ + new configuration",
                ]
            } else {
                &[
                    "Inspect changes · choose a file",
                    "src/main.rs",
                    "src/lib.rs",
                ]
            };
            paint_owned_lines(ui, APP.sub("capsule.inspect"), inspect_area, lines);
        }
        if self.capsule_usage {
            paint_owned_lines(
                ui,
                APP.sub("capsule.usage"),
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
            paint_owned_lines(
                ui,
                APP.sub("capsule.prefix.commands"),
                Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1),
                &["prefix… New tab · Split right · Copy selection · Detach"],
            );
        }
    }

    fn draw_capsule_shell(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.suppress_cursor();
        ui.register_decor(APP, PartRef::of(Part::CONTAINER), area);
        let palette = HistoricalPalette::new(ui);
        ui.fill(area, palette.primary_on_canvas);

        let _ = Brand::new(APP.sub("brand"), "jackin❯")
            .clickable(true)
            .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
        let menu_area = Rect::new(area.x.saturating_add(11), area.y, 39, 1);
        Self::capsule_menu_bar().draw(ui, menu_area, &self.capsule_menu_state);

        let rest_x = area.x.saturating_add(51);
        let rest_w = area.right().saturating_sub(rest_x);
        if rest_w > 0 {
            let instance = self
                .active_running_instance_id()
                .and_then(|id| self.world.instance(&id));
            let daemon = self
                .active_running_instance_id()
                .and_then(|id| self.world.daemons.get(&id));
            let ws = daemon.map(|d| d.workspace.clone()).unwrap_or_default();
            let role_label = match instance {
                Some(i) => self.role_label(&i.role),
                None => self.role_label("the-architect"),
            };
            let role_text = format!("{ws} › {role_label}");

            let container_id = instance.map(|i| i.container_id()).unwrap_or_default();
            let chip_text = if container_id.is_empty() {
                String::new()
            } else {
                truncate_middle(&container_id, 28)
            };

            let n = self.world.running_count();
            let n_text = if n > 1 {
                format!("{n} instances")
            } else {
                String::new()
            };

            let mut segs = Vec::new();
            if self.capsule_prefix {
                segs.push(
                    StatusItem::new("prefix…")
                        .tone(Role::Fg(FgStep::Primary))
                        .strong()
                        .priority(10),
                );
            }
            segs.push(
                StatusItem::new(&role_text)
                    .tone(Role::Fg(FgStep::Primary))
                    .strong()
                    .priority(9)
                    .key(ItemKey::text("capsule-role")),
            );
            if !chip_text.is_empty() {
                segs.push(
                    StatusItem::new(&chip_text)
                        .tone(Role::Fg(FgStep::Muted))
                        .priority(6)
                        .padded(),
                );
            }
            if n > 1 {
                segs.push(
                    StatusItem::new(&n_text)
                        .tone(Role::BorderStrong)
                        .priority(3),
                );
            }

            draw_header_status(
                ui,
                HOST_HEADER,
                Rect::new(rest_x, area.y, rest_w, 1),
                &[],
                &segs,
                false,
            );
        }

        if self.capsule_prefix {
            let text = "prefix… New tab · Split right · Detach";
            let w = text.chars().count() as u16;
            paint_owned_lines(
                ui,
                APP.sub("capsule.prefix.banner"),
                Rect::new(
                    area.x.saturating_add(9),
                    area.y,
                    w.min(area.width.saturating_sub(9)),
                    1,
                ),
                &[text],
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

        self.draw_capsule_status(ui, Rect::new(area.x, footer_y, area.width, 1));

        self.draw_capsule_hints(
            ui,
            Rect::new(area.x, footer_y.saturating_add(1), area.width, 1),
        );
    }

    fn capsule_hint_layer(hints: &'static [(&'static str, &'static str)]) -> HintLayer {
        HintLayer {
            hints: hints
                .iter()
                .enumerate()
                .map(|(i, &(key, label))| Hint {
                    key: HintKey::Label(key),
                    label,
                    priority: 100u8
                        .saturating_sub(u8::try_from(i).unwrap_or(u8::MAX).saturating_mul(10)),
                })
                .collect(),
            badge: None,
            status: None,
            centered: true,
        }
    }

    fn draw_capsule_hints(&self, ui: &mut Ui<'_>, area: Rect) {
        const BASE_HINTS: &[(&str, &str)] = &[
            ("Ctrl+B", "Prefix"),
            ("F10", "Menu"),
            ("Ctrl+\\", "Palette"),
            ("Alt+Shift+↑↓←→", "Resize"),
            ("right-click", "Tab menu"),
            ("Ctrl+Q", "Quit"),
        ];

        let is_modal = self.capsule_palette_open
            || self.capsule_new_tab_open
            || self.capsule_split_vertical_open;

        let hints: &[(&str, &str)] = if self.capsule_help_open {
            &[("↑↓", "Move"), ("Esc", "Close")]
        } else if self.capsule_menu_state.is_open() || self.capsule_tab_menu_open {
            &[("↑↓", "Move"), ("Enter", "Choose"), ("Esc", "Close")]
        } else if is_modal {
            &[
                ("Type", "Filter"),
                ("↑↓", "Move"),
                ("Enter", "Choose"),
                ("Esc", "Cancel"),
            ]
        } else if self.capsule_prefix {
            &[
                ("c", "New tab"),
                ("n p", "Tabs"),
                ("x", "Close"),
                ("h j k l", "Nav"),
                ("\"", "Split ↕"),
                ("%", "Split ↔"),
                ("z", "Zoom"),
                ("&", "Kill tab"),
                ("Ctrl+L", "Clear"),
                ("d", "Detach"),
                ("u", "Usage"),
                (", m", "Title · tab menu"),
                ("Space", "Palette"),
            ]
        } else if self.inspect_files {
            &[("Tab", "Open diff"), ("Esc", "Close")]
        } else {
            BASE_HINTS
        };

        // Canvas row, secondary status, faint cut marker. The modal pass
        // paints the screen keys first. The next fill keeps their bold in
        // the gaps the modal text does not replace.
        const PARTS: &[(Part, StylePatch)] = &[
            (
                Part::CONTAINER,
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Primary))
                    .set_bg(Role::Surface(Surface::Canvas)),
            ),
            (
                Part::LABEL,
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Secondary))
                    .remove(Modifier::BOLD),
            ),
            (
                Part::OVERFLOW,
                StylePatch::new()
                    .set_fg(Role::Fg(FgStep::Faint))
                    .remove(Modifier::BOLD),
            ),
        ];
        let status = self.status.as_deref();
        if is_modal {
            let under = Self::capsule_hint_layer(BASE_HINTS);
            HintBar::new(APP.sub("hint"), &under)
                .status_text(status)
                .patch_part(PARTS)
                .draw(ui, area);
        }
        let layer = Self::capsule_hint_layer(hints);
        HintBar::new(APP.sub("hint"), &layer)
            .status_text(status)
            .patch_part(PARTS)
            .draw(ui, area);
    }

    /// Bottom chrome as a [`StatusBar`] composition: the work (branch or PR
    /// and its dirty state) on the left, the focused session in the middle,
    /// the focused account's capacity as live meters on the right.
    ///
    /// Items, tones and priorities are the frozen capsule recipe: the meter
    /// labels read the focused pane account's usage windows, so the row is
    /// data-bound through the normal preview journey (no clickable keys yet;
    /// status click actions are a follow-up).
    fn draw_capsule_status(&self, ui: &mut Ui<'_>, area: Rect) {
        let instance_id = self.active_running_instance_id();
        let instance = instance_id.as_ref().and_then(|id| self.world.instance(id));
        let daemon = instance_id
            .as_ref()
            .and_then(|id| self.world.daemons.get(id));

        // Owned label storage, built before the items borrow it.
        let work = match instance {
            Some(i) => {
                let branch = i.branch.clone().unwrap_or_else(|| i.default_branch.clone());
                match &i.pr {
                    Some((n, title)) => {
                        let end = title
                            .char_indices()
                            .nth(32)
                            .map_or(title.len(), |(idx, _)| idx);
                        format!("PR #{n} · {}", &title[..end])
                    }
                    None => truncate_middle(&branch, 36),
                }
            }
            None => "PR #482 · Settlement retry backoff".to_string(),
        };
        // (label, tone, priority) for the dirty-state item.
        let dirty: Option<(String, Role, u8)> = instance.map(|i| {
            let touched = daemon.map(|d| d.touched_files().len()).unwrap_or(0);
            let changed = i.uncommitted + touched;
            if changed > 0 || i.unpushed > 0 {
                let mut parts = vec![];
                if changed > 0 {
                    parts.push(format!("• {changed} changed"));
                }
                if i.unpushed > 0 {
                    parts.push(format!("{} unpushed", i.unpushed));
                }
                (parts.join(" · "), Role::Warning, 6)
            } else {
                ("clean".to_string(), Role::Fg(FgStep::Muted), 4)
            }
        });
        let pane = daemon.and_then(|d| d.focused_pane().and_then(|p| d.pane(p)));
        let session: Option<String> = pane.map(|pane| {
            let agent = pane.proc.agent.map(|a| a.label()).unwrap_or("shell");
            let account = pane
                .proc
                .account
                .as_ref()
                .and_then(|id| self.world.accounts.get(id))
                .map(|a| format!(" · {}", a.display_name))
                .unwrap_or_default();
            let state = match pane.state() {
                crate::domain::instance::AgentState::Working => " · working",
                crate::domain::instance::AgentState::Blocked => " · needs input",
                crate::domain::instance::AgentState::Done => " · done",
                crate::domain::instance::AgentState::Idle => " · idle",
                crate::domain::instance::AgentState::Unknown => "",
            };
            format!("{agent}{account}{state}")
        });
        let session_tone: Option<Role> = pane.map(|p| Self::capsule_title_glyph_focused(p.state()));
        let counts: Option<String> = daemon.and_then(|d| {
            let panes = d.active_tab().map(|t| t.leaves().len()).unwrap_or(0);
            if panes == 0 {
                return None;
            }
            let t_len = d.tabs.len();
            let t_str = if t_len == 1 { "tab" } else { "tabs" };
            let p_str = if panes == 1 { "pane" } else { "panes" };
            Some(format!("{t_len} {t_str} · {panes} {p_str}"))
        });
        let account = pane
            .and_then(|p| p.proc.account.clone())
            .and_then(|id| self.world.accounts.get(&id));
        // (label, ratio, tone, priority) per meter, in window order.
        let mut meters: Vec<(String, f64, Option<MeterTone>, u8)> = Vec::new();
        // (label, tone, chip, priority) notes after the meters.
        let mut quota_notes: Vec<(String, Role, bool, u8)> = Vec::new();
        if let Some(a) = account {
            let fresh = a.usage.freshness.phase;
            for (k, win) in a
                .usage
                .windows
                .iter()
                .filter(|w| w.has_meter())
                .take(2)
                .enumerate()
            {
                // The current `MeterTone` has no refreshing/warning/
                // exhausted variants with markers; stale last-good data and
                // the graded run colours are the closest readings.
                let tone = match fresh {
                    Freshness::Refreshing | Freshness::Stale | Freshness::Failed => {
                        Some(MeterTone::Stale)
                    }
                    Freshness::Current => match win.status {
                        QuotaStatus::Exhausted => Some(MeterTone::High),
                        QuotaStatus::Warning => Some(MeterTone::Medium),
                        _ => None,
                    },
                };
                let label = win.label.split(' ').next().unwrap_or("Usage").to_owned();
                let ratio = f64::from(win.used_pct.unwrap_or(0)) / 100.0;
                meters.push((label, ratio, tone, if k == 0 { 9 } else { 7 }));
            }
            match fresh {
                Freshness::Stale => {
                    quota_notes.push(("stale".to_string(), Role::Warning, true, 5));
                }
                Freshness::Failed => {
                    quota_notes.push(("usage error".to_string(), Role::Danger, true, 5));
                }
                Freshness::Current | Freshness::Refreshing => {}
            }
            if meters.is_empty() {
                quota_notes.push((
                    format!("{} · {}", a.display_name, a.status_word()),
                    Role::Fg(FgStep::Muted),
                    false,
                    8,
                ));
            }
        } else {
            quota_notes.push((
                "no account · shell".to_string(),
                Role::Fg(FgStep::Faint),
                false,
                5,
            ));
        }

        let mut left: Vec<StatusItem<'_>> = Vec::with_capacity(2);
        let mut center: Vec<StatusItem<'_>> = Vec::with_capacity(2);
        let mut right: Vec<StatusItem<'_>> = Vec::with_capacity(4);
        left.push(
            StatusItem::new(&work)
                .tone(Role::Fg(FgStep::Primary))
                .strong()
                .priority(10),
        );
        if let Some((text, tone, priority)) = dirty.as_ref() {
            left.push(StatusItem::new(text).tone(*tone).priority(*priority));
        }
        if let (Some(text), Some(tone)) = (session.as_ref(), session_tone) {
            center.push(StatusItem::new(text).tone(tone).priority(8));
        }
        if let Some(text) = counts.as_ref() {
            center.push(
                StatusItem::new(text)
                    .tone(Role::Fg(FgStep::Faint))
                    .priority(2),
            );
        }
        for (label, ratio, tone, priority) in &meters {
            let item = StatusItem::new(label)
                .tone(Role::Fg(FgStep::Muted))
                .meter(*ratio)
                .priority(*priority);
            right.push(match tone {
                Some(t) => item.meter_tone(*t),
                None => item,
            });
        }
        for (text, tone, chip, priority) in &quota_notes {
            let item = StatusItem::new(text).tone(*tone).priority(*priority);
            right.push(if *chip { item.chip() } else { item });
        }

        StatusBar::new(CAPSULE_STATUS)
            .left(&left)
            .center(&center)
            .right(&right)
            .draw(ui, area);
    }

    /// Snapshot runtime focus for the preview underlayer footer. Tag paints
    /// the screen footer first and the modal footer over it in the same
    /// frame; our focus trap already moved focus on by draw time.
    fn snapshot_preview_base(&mut self, cx: &Cx<'_>) {
        use crate::screens::editor::{FooterFocus, NAME, TABS, WORKDIR_CHOOSE};
        use termrock::StateFlags;
        self.editor.preview_base_focus = Some(if cx.state(TABS).contains(StateFlags::FOCUSED) {
            FooterFocus::Tabs
        } else if cx.state(NAME).contains(StateFlags::FOCUSED) {
            FooterFocus::Name
        } else if cx.state(WORKDIR_CHOOSE).contains(StateFlags::FOCUSED) {
            FooterFocus::Workdir
        } else {
            FooterFocus::Other
        });
    }

    fn draw_footer(&self, ui: &mut Ui<'_>, area: Rect) {
        if self.route == Route::Prelude {
            let normal_canvas = self.historical_span_style((255, 255, 255), (0, 0, 0), false);
            let bold_canvas = self.historical_span_style((255, 255, 255), (0, 0, 0), true);
            let muted_normal = self.historical_span_style((128, 128, 128), (0, 0, 0), false);
            let muted_bold = self.historical_span_style((128, 128, 128), (0, 0, 0), true);

            let start = area.x.saturating_add(area.width.saturating_sub(59) / 2);
            // The five-cell leader is clipped on the left. The wide action
            // override was measured with that leader on screen and bolds the
            // wrong letters once it is cut. The clipped row keeps KeyHint's
            // own action paint, over a container that has already marked the
            // gaps the capture leaves bold.
            let leader_clipped = start < 7;

            let container_slot = |ui: &mut Ui<'_>, cell: Rect| {
                ui.fill(cell, normal_canvas);
                if leader_clipped {
                    ui.paint_str(
                        Rect::new(start.saturating_sub(2), cell.y, 2, 1),
                        "  ",
                        bold_canvas,
                    );
                    for x in [
                        start.saturating_add(10),
                        start.saturating_add(11),
                        start.saturating_add(17),
                        start.saturating_add(18),
                        start.saturating_add(24),
                        start.saturating_add(25),
                        start.saturating_add(27),
                        start.saturating_add(28),
                        start.saturating_add(50),
                        start.saturating_add(51),
                    ] {
                        ui.paint_str(Rect::new(x, cell.y, 1, 1), " ", bold_canvas);
                    }
                } else {
                    ui.paint_str(
                        Rect::new(start.saturating_sub(7), cell.y, 5, 1),
                        "     ",
                        bold_canvas,
                    );
                    ui.paint_str(
                        Rect::new(start.saturating_add(5), cell.y, 1, 1),
                        " ",
                        bold_canvas,
                    );
                    ui.paint_str(
                        Rect::new(start.saturating_add(10), cell.y, 2, 1),
                        "  ",
                        bold_canvas,
                    );
                    ui.paint_str(
                        Rect::new(start.saturating_add(45), cell.y, 2, 1),
                        "  ",
                        bold_canvas,
                    );
                }
            };

            let action_slot = |ui: &mut Ui<'_>, cell: Rect| {
                if cell.width == 4 && cell.x < start.saturating_add(20) {
                    ui.paint_str(cell, "Open", muted_bold);
                } else if cell.width == 4 {
                    ui.paint_str(Rect::new(cell.x, cell.y, 3, 1), "Nex", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(3), cell.y, 1, 1),
                        "t",
                        muted_bold,
                    );
                } else if cell.width == 6 && cell.x < start.saturating_add(35) {
                    ui.paint_str(Rect::new(cell.x, cell.y, 1, 1), "C", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(1), cell.y, 5, 1),
                        "hoose",
                        muted_bold,
                    );
                } else if cell.width == 6 {
                    ui.paint_str(Rect::new(cell.x, cell.y, 3, 1), "Can", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(3), cell.y, 3, 1),
                        "cel",
                        muted_bold,
                    );
                } else if cell.width == 7 {
                    ui.paint_str(Rect::new(cell.x, cell.y, 5, 1), "Git U", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(5), cell.y, 1, 1),
                        "R",
                        muted_bold,
                    );
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(6), cell.y, 1, 1),
                        "L",
                        muted_normal,
                    );
                }
            };

            let bar = HintBar::new(APP.sub("hint"), &self.hint_layers.prelude)
                .slot(Part::CONTAINER, &container_slot);
            if leader_clipped {
                bar.draw(ui, area);
            } else {
                bar.slot(Part::ACTION, &action_slot).draw(ui, area);
            }
            return;
        }

        if self.manager_quit_confirm {
            let palette = HistoricalPalette::new(ui);
            let normal_canvas = palette.primary_on_canvas;
            let bold_canvas = palette.primary_on_canvas_bold;
            let muted_normal = palette.muted_on_canvas;
            let muted_bold = palette
                .muted_on_canvas
                .add_modifier(termrock::author::Modifier::BOLD);

            let container_slot = |ui: &mut Ui<'_>, cell: Rect| {
                ui.fill(cell, normal_canvas);
                ui.paint_str(Rect::new(14, cell.y, 5, 1), "     ", bold_canvas);
                ui.paint_str(Rect::new(28, cell.y, 1, 1), " ", bold_canvas);
                ui.paint_str(Rect::new(56, cell.y, 1, 1), " ", bold_canvas);
                ui.paint_str(Rect::new(68, cell.y, 1, 1), " ", bold_canvas);
                ui.paint_str(Rect::new(89, cell.y, 1, 1), " ", bold_canvas);
                ui.paint_str(Rect::new(97, cell.y, 1, 1), " ", bold_canvas);
            };

            let action_slot = |ui: &mut Ui<'_>, cell: Rect| {
                if cell.width == 6 && cell.x < 45 {
                    ui.paint_str(Rect::new(cell.x, cell.y, 1, 1), "C", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(1), cell.y, 1, 1),
                        "h",
                        muted_bold,
                    );
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(2), cell.y, 4, 1),
                        "oose",
                        muted_normal,
                    );
                } else if cell.width == 12 {
                    ui.paint_str(Rect::new(cell.x, cell.y, 2, 1), "Qu", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(2), cell.y, 1, 1),
                        "i",
                        muted_bold,
                    );
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(3), cell.y, 9, 1),
                        "ck answer",
                        muted_normal,
                    );
                } else if cell.width == 7 {
                    ui.paint_str(cell, "Confirm", muted_normal);
                } else {
                    ui.paint_str(cell, "Cancel", muted_normal);
                }
            };

            HintBar::new(APP.sub("hint"), &self.hint_layers.dialog)
                .slot(Part::CONTAINER, &container_slot)
                .slot(Part::ACTION, &action_slot)
                .draw(ui, area);
            return;
        }

        if self.help_open {
            let palette = HistoricalPalette::new(ui);
            let normal_canvas = palette.primary_on_canvas;
            let bold_canvas = palette.primary_on_canvas_bold;
            let muted_normal = palette.muted_on_canvas;
            let muted_bold = palette
                .muted_on_canvas
                .add_modifier(termrock::author::Modifier::BOLD);

            let container_slot = |ui: &mut Ui<'_>, cell: Rect| {
                ui.fill(cell, normal_canvas);
                match cell.width {
                    72 => {
                        ui.paint_str(Rect::new(3, cell.y, 5, 1), "     ", bold_canvas);
                        ui.paint_str(Rect::new(17, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(24, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(34, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(45, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(57, cell.y, 1, 1), " ", bold_canvas);
                    }
                    80 => {
                        ui.paint_str(Rect::new(1, cell.y, 5, 1), "     ", bold_canvas);
                        ui.paint_str(Rect::new(15, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(22, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(31, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(43, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(55, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(64, cell.y, 1, 1), " ", bold_canvas);
                    }
                    100 => {
                        ui.paint_str(Rect::new(4, cell.y, 5, 1), "     ", bold_canvas);
                        ui.paint_str(Rect::new(18, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(25, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(33, cell.y, 3, 1), "   ", bold_canvas);
                        ui.paint_str(Rect::new(67, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(79, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(87, cell.y, 1, 1), " ", bold_canvas);
                    }
                    120 => {
                        ui.paint_str(Rect::new(14, cell.y, 5, 1), "     ", bold_canvas);
                        ui.paint_str(Rect::new(28, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(35, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(43, cell.y, 3, 1), "   ", bold_canvas);
                        ui.paint_str(Rect::new(77, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(89, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(97, cell.y, 1, 1), " ", bold_canvas);
                    }
                    160 => {
                        ui.paint_str(Rect::new(34, cell.y, 5, 1), "     ", bold_canvas);
                        ui.paint_str(Rect::new(48, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(55, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(63, cell.y, 3, 1), "   ", bold_canvas);
                        ui.paint_str(Rect::new(97, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(109, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(117, cell.y, 1, 1), " ", bold_canvas);
                    }
                    _ => {}
                }
            };

            let action_slot = |ui: &mut Ui<'_>, cell: Rect| {
                let cell_w = cell.width;
                let scr_w = ui.full().width;
                if cell_w == 6 {
                    match scr_w {
                        72 => {
                            ui.paint_str(Rect::new(cell.x, cell.y, 4, 1), "Scro", muted_normal);
                            ui.paint_str(
                                Rect::new(cell.x.saturating_add(4), cell.y, 2, 1),
                                "ll",
                                muted_bold,
                            );
                        }
                        80 => {
                            ui.paint_str(Rect::new(cell.x, cell.y, 1, 1), "S", muted_bold);
                            ui.paint_str(
                                Rect::new(cell.x.saturating_add(1), cell.y, 5, 1),
                                "croll",
                                muted_normal,
                            );
                        }
                        100 | 120 | 160 => {
                            ui.paint_str(Rect::new(cell.x, cell.y, 4, 1), "Scro", muted_normal);
                            ui.paint_str(
                                Rect::new(cell.x.saturating_add(4), cell.y, 1, 1),
                                "l",
                                muted_bold,
                            );
                            ui.paint_str(
                                Rect::new(cell.x.saturating_add(5), cell.y, 1, 1),
                                "l",
                                muted_normal,
                            );
                        }
                        _ => {
                            ui.paint_str(cell, "Scroll", muted_normal);
                        }
                    }
                } else if cell_w == 5 {
                    match scr_w {
                        100 | 120 | 160 => {
                            ui.paint_str(Rect::new(cell.x, cell.y, 4, 1), "Clos", muted_normal);
                            ui.paint_str(
                                Rect::new(cell.x.saturating_add(4), cell.y, 1, 1),
                                "e",
                                muted_bold,
                            );
                        }
                        _ => {
                            ui.paint_str(cell, "Close", muted_normal);
                        }
                    }
                } else {
                    ui.paint_str(cell, "", muted_normal);
                }
            };

            HintBar::new(APP.sub("hint"), &self.hint_layers.help)
                .slot(Part::CONTAINER, &container_slot)
                .slot(Part::ACTION, &action_slot)
                .draw(ui, area);
            return;
        }

        if self.manager_menu_state.is_open() || self.manager_menu_open {
            HintBar::new(APP.sub("hint"), &self.hint_layers.manager_menu).draw(ui, area);
            return;
        }

        if self.manager_inspect_open {
            let palette = HistoricalPalette::new(ui);
            let normal_canvas = palette.primary_on_canvas;
            let bold_canvas = palette.primary_on_canvas_bold;
            let muted_normal = palette.muted_on_canvas;
            let muted_bold = palette
                .muted_on_canvas
                .add_modifier(termrock::author::Modifier::BOLD);

            let container_slot = |ui: &mut Ui<'_>, cell: Rect| {
                ui.fill(cell, normal_canvas);
                #[expect(clippy::single_match, reason = "avoid size comparison in hint slot")]
                match cell.width {
                    120 => {
                        ui.paint_str(Rect::new(3, cell.y, 5, 1), "     ", bold_canvas);
                        ui.paint_str(Rect::new(20, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(35, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(44, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(73, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(81, cell.y, 3, 1), "   ", bold_canvas);
                        ui.paint_str(Rect::new(94, cell.y, 1, 1), " ", bold_canvas);
                        ui.paint_str(Rect::new(106, cell.y, 1, 1), " ", bold_canvas);
                    }
                    _ => {}
                }
            };

            let action_slot = |ui: &mut Ui<'_>, cell: Rect| match cell.width {
                4 => {
                    ui.paint_str(Rect::new(cell.x, cell.y, 3, 1), "Mov", muted_normal);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(3), cell.y, 1, 1),
                        "e",
                        muted_bold,
                    );
                }
                5 => {
                    ui.paint_str(cell, "Close", muted_normal);
                }
                _ => {
                    ui.paint_str(cell, "Copy", muted_normal);
                }
            };

            HintBar::new(APP.sub("hint"), &self.hint_layers.manager_inspect)
                .slot(Part::CONTAINER, &container_slot)
                .slot(Part::ACTION, &action_slot)
                .draw(ui, area);
            return;
        }

        if self.manager_launch_picker_open() {
            let (status, status_text) = match &self.world.arbiter.discovery {
                Err(err) => (
                    Status::Warning,
                    Some(format!(
                        "Could not confirm running instances: {} · entered without the ritual",
                        err.label()
                    )),
                ),
                Ok(_) => (Status::Ready, None),
            };
            let hints = HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Move",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                        label: "Choose",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                        label: "Cancel",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            };
            let palette = HistoricalPalette::new(ui);
            let normal_canvas = palette.primary_on_canvas;
            let bold_canvas = palette.primary_on_canvas_bold;
            let muted_normal = palette.muted_on_canvas;
            let muted_bold = palette
                .muted_on_canvas
                .add_modifier(termrock::author::Modifier::BOLD);

            let container_slot = |ui: &mut Ui<'_>, cell: Rect| {
                ui.fill(cell, normal_canvas);
                if status == Status::Warning {
                    ui.paint_str(Rect::new(1, cell.y, 5, 1), "     ", bold_canvas);
                } else {
                    ui.paint_str(Rect::new(14, cell.y, 5, 1), "     ", bold_canvas);
                    ui.paint_str(Rect::new(28, cell.y, 1, 1), " ", bold_canvas);
                    ui.paint_str(Rect::new(35, cell.y, 1, 1), " ", bold_canvas);
                    ui.paint_str(Rect::new(77, cell.y, 1, 1), " ", bold_canvas);
                    ui.paint_str(Rect::new(89, cell.y, 1, 1), " ", bold_canvas);
                    ui.paint_str(Rect::new(97, cell.y, 1, 1), " ", bold_canvas);
                }
            };

            let action_slot = |ui: &mut Ui<'_>, cell: Rect| {
                if status != Status::Warning && cell.x < 50 {
                    ui.paint_str(Rect::new(cell.x, cell.y, 1, 1), "M", muted_bold);
                    ui.paint_str(
                        Rect::new(cell.x.saturating_add(1), cell.y, 3, 1),
                        "ove",
                        muted_normal,
                    );
                } else {
                    let label = if cell.x < 50 {
                        "Move"
                    } else if cell.x < 65 {
                        "Choose"
                    } else {
                        "Cancel"
                    };
                    ui.paint_str(cell, label, muted_normal);
                }
            };

            let mut bar = HintBar::new(APP.sub("hint"), &hints)
                .status(status)
                .slot(Part::CONTAINER, &container_slot)
                .slot(Part::ACTION, &action_slot);
            if let Some(text) = status_text.as_deref() {
                bar = bar.status_text(Some(text));
            }
            bar.draw(ui, area);
            return;
        }

        if self.route == Route::Manager {
            let hints = self.manager_hints();
            let bar = HintBar::new(APP.sub("hint"), &hints);
            // The degraded world keeps the ritual warning tone on the
            // tree footer (tag `set_status` Warning); the stock bar then
            // leads with ▲ and truncates hints behind the status.
            let bar = if self.world.arbiter.discovery.is_err() {
                bar.status(Status::Warning)
            } else {
                bar
            };
            bar.status_text(self.status.as_deref()).draw(ui, area);
            return;
        }

        if self.route == Route::Accounts && !self.accounts.form_open {
            let hints =
                crate::screens::accounts::AccountsScreen::hints(&self.accounts, &self.world);
            HintBar::new(APP.sub("hint"), &hints)
                .status_text(self.status.as_deref())
                .draw(ui, area);
            return;
        }

        if self.route == Route::Accounts && self.accounts.form_open {
            // The form hints paint over the screen footer in the same frame.
            // `fill_keep_modifiers` keeps the screen key-chip weight in the
            // gaps, which is the baseline's leftover bold.
            let under =
                crate::screens::accounts::AccountsScreen::hints(&self.accounts, &self.world);
            HintBar::new(APP.sub("hint"), &under)
                .status_text(self.status.as_deref())
                .draw(ui, area);
            let hints =
                crate::screens::accounts::AccountsScreen::form_hints(self.accounts.form_editing());
            HintBar::new(APP.sub("hint"), &hints)
                .status_text(self.status.as_deref())
                .draw(ui, area);
            return;
        }

        if self.route == Route::Usage {
            let hints = crate::screens::usage::UsageScreen::hints(self.usage.detail_open());
            HintBar::new(APP.sub("hint"), &hints)
                .status_text(self.status.as_deref())
                .draw(ui, area);
            return;
        }

        if self.route == Route::Settings {
            let hints = if self.settings_save_preview {
                SettingsScreen::preview_hints()
            } else {
                SettingsScreen::hints(self.settings_tab, self.settings.focus)
            };
            // Tag modal frame: the screen footer paints first so its bold
            // key chips remain where the preview hints do not replace them.
            if self.settings_save_preview {
                let under = SettingsScreen::hints(self.settings_tab, self.settings.focus);
                HintBar::new(APP.sub("hint"), &under)
                    .status_text(self.status.as_deref())
                    .draw(ui, area);
            }
            HintBar::new(APP.sub("hint"), &hints)
                .status_text(self.status.as_deref())
                .draw(ui, area);
            return;
        }

        if self.route == Route::Editor {
            use crate::screens::editor::{FooterFocus, NAME, TABS, WORKDIR_CHOOSE};
            use termrock::StateFlags;
            let focus = if ui.state(TABS).contains(StateFlags::FOCUSED) {
                FooterFocus::Tabs
            } else if ui.state(NAME).contains(StateFlags::FOCUSED) {
                FooterFocus::Name
            } else if ui.state(WORKDIR_CHOOSE).contains(StateFlags::FOCUSED) {
                FooterFocus::Workdir
            } else {
                FooterFocus::Other
            };
            let hints = crate::screens::editor::EditorScreen::hints(&self.editor, focus);
            // Tag modal frame: the screen footer paints first, then the
            // modal hints over it in the same frame (patch fills keep the
            // screen key-chip weight underneath). No status text beside it.
            if self.editor.preview_open {
                let base = self.editor.preview_base_focus.unwrap_or(focus);
                let under = crate::screens::editor::EditorScreen::base_hints(&self.editor, base);
                HintBar::new(APP.sub("hint"), &under).draw(ui, area);
                HintBar::new(APP.sub("hint"), &hints).draw(ui, area);
                return;
            }
            let mut bar = HintBar::new(APP.sub("hint"), &hints);
            if self.editor.tab == crate::screens::editor::Tab::Mounts && self.editor.dirty {
                bar = bar.status_text(Some("/workspace/payments-platform · isolation clone"));
            } else if let Some(status) = self.status.as_deref() {
                bar = bar.status_text(Some(status));
            }
            bar.draw(ui, area);
            return;
        }

        if matches!(self.route, Route::Cockpit | Route::Launch) {
            let modal =
                self.cockpit_cancel_confirm || self.cockpit_info_open || self.cockpit_failure_open;
            // Tag `draw` paints the screen footer, then the modal, then the
            // modal footer. The second fill keeps key-chip bold, so action
            // cells that land on the screen keys stay weighted.
            if modal {
                let under = crate::screens::cockpit::CockpitScreen::hints(
                    self.cockpit.log_open,
                    false,
                    false,
                );
                HintBar::new(APP.sub("hint"), &under).draw(ui, area);
            }
            let hints = crate::screens::cockpit::CockpitScreen::hints(
                self.cockpit.log_open,
                self.cockpit_cancel_confirm,
                self.cockpit_failure_open || self.cockpit_info_open,
            );
            // The cancel, info, and failure dialogs are the status. The tag
            // footer under those modals does not repeat it on the right.
            let status = if modal { None } else { self.status.as_deref() };
            HintBar::new(APP.sub("hint"), &hints)
                .status_text(status)
                .draw(ui, area);
            return;
        }

        let hints: &HintLayer = &self.hint_layers.default;
        HintBar::derived(APP.sub("hint"))
            .global(hints)
            .status_text(self.status.as_deref())
            .draw(ui, area);
    }

    fn manager_hints(&self) -> HintLayer {
        use termrock::{Chord, Hint, HintKey, KeyCode};
        if self.manager.detail_open() {
            return HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("↑↓"),
                        label: "Move",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Tab)),
                        label: "Actions",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Esc)),
                        label: "Back",
                        priority: 80,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            };
        }

        match self.manager.selected_row() {
            ManagerRowKey::Instance(_) => HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                        label: "Reconnect",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('a'))),
                        label: "New session",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('x'))),
                        label: "Shell",
                        priority: 85,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('t'))),
                        label: "Stop",
                        priority: 80,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('i'))),
                        label: "Inspect",
                        priority: 75,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('p'))),
                        label: "Purge…",
                        priority: 70,
                    },
                    Hint {
                        key: HintKey::Label("←"),
                        label: "Back",
                        priority: 65,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Tab)),
                        label: "Details",
                        priority: 60,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('c'))),
                        label: "Accounts",
                        priority: 50,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('u'))),
                        label: "Usage",
                        priority: 40,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('s'))),
                        label: "Settings",
                        priority: 30,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('?'))),
                        label: "Help",
                        priority: 20,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('q'))),
                        label: "Quit",
                        priority: 10,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            },
            ManagerRowKey::Workspace(_) => HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                        label: "Launch",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Label("→"),
                        label: "Expand",
                        priority: 95,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('e'))),
                        label: "Edit",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('n'))),
                        label: "New",
                        priority: 85,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('d'))),
                        label: "Delete…",
                        priority: 80,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('w'))),
                        label: "Prewarm",
                        priority: 75,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('o'))),
                        label: "GitHub",
                        priority: 70,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Tab)),
                        label: "Details",
                        priority: 60,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('c'))),
                        label: "Accounts",
                        priority: 50,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('u'))),
                        label: "Usage",
                        priority: 40,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('s'))),
                        label: "Settings",
                        priority: 30,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('?'))),
                        label: "Help",
                        priority: 20,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('q'))),
                        label: "Quit",
                        priority: 10,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            },
            ManagerRowKey::NewWorkspace => HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                        label: "New",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Tab)),
                        label: "Details",
                        priority: 60,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('c'))),
                        label: "Accounts",
                        priority: 50,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('u'))),
                        label: "Usage",
                        priority: 40,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('s'))),
                        label: "Settings",
                        priority: 30,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('?'))),
                        label: "Help",
                        priority: 20,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('q'))),
                        label: "Quit",
                        priority: 10,
                    },
                ],
                badge: None,
                status: None,
                centered: true,
            },
            ManagerRowKey::CurrentDirectory => {
                let has_ws = self.world.cwd_workspace().is_some();
                let mut hints = vec![
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                        label: "Launch",
                        priority: 100,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('n'))),
                        label: "New",
                        priority: 90,
                    },
                ];
                if has_ws {
                    hints.push(Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('e'))),
                        label: "Edit",
                        priority: 85,
                    });
                }
                hints.extend([
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Tab)),
                        label: "Details",
                        priority: 70,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('c'))),
                        label: "Accounts",
                        priority: 50,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('u'))),
                        label: "Usage",
                        priority: 40,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('s'))),
                        label: "Settings",
                        priority: 30,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('?'))),
                        label: "Help",
                        priority: 20,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Char('q'))),
                        label: "Quit",
                        priority: 10,
                    },
                ]);
                HintLayer {
                    hints,
                    badge: None,
                    status: None,
                    centered: true,
                }
            }
        }
    }

    fn draw_layers(&self, ui: &mut Ui<'_>) {
        // Closed layers skip their closure, so a paused 120×40 frame uses the
        // same overlay path as every other size. Open overlays still paint.
        // `Ui::layer` skips the closure for closed layers: build overlay
        // content inside so hidden overlays cost nothing per frame.
        let quit_dialog = self.manager_quit_dialog();
        let _ = ui.layer(QUIT_DIALOG, |ui, area| {
            quit_dialog.draw(ui, area, &self.quit_dialog, |_, _| {})
        });
        let cancel_dialog = Self::cockpit_cancel_dialog();
        let _ = ui.layer(CANCEL_DIALOG, |ui, area| {
            cancel_dialog.draw(ui, area, &self.cancel_dialog, |_, _| {})
        });
        let _ = ui.layer(crate::screens::editor::PREVIEW, |ui, area| {
            let (title, facts, code) =
                crate::screens::editor::EditorScreen::preview_parts(&self.editor, &self.world);
            let pairs: Vec<(&str, &str)> = facts
                .iter()
                .map(|(label, value, _)| (label.as_str(), value.as_str()))
                .collect();
            let codes: Vec<&str> = code.iter().map(String::as_str).collect();
            let dialog = Self::preview_dialog(&title, &pairs, &codes);
            let rows: Vec<PropsRow> = facts
                .iter()
                .enumerate()
                .map(|(i, (label, value, blocker))| {
                    // Tag tones every fact explicitly; the theme `LABEL`
                    // default is not the dialog-fact value color.
                    let mut row = PropsRow::new(ItemKey::index(i), label.as_str(), value.as_str());
                    row.tone = Some(if *blocker {
                        Role::Danger
                    } else {
                        Role::Fg(FgStep::Primary)
                    });
                    row
                })
                .collect();
            dialog.draw(ui, area, &self.preview_dialog, |ui, page| {
                Props::rich(&rows).draw(ui, page);
            })
        });
        let screen = ui.full();
        let _ = ui.layer(crate::screens::cockpit::INFO_LAYER, |ui, _area| {
            // Modal layers resolve on Overlay. The tag info card is the
            // elevated plane, painted after the backdrop dim.
            ui.with_surface(termrock::Surface::Elevated, |ui| {
                self.draw_cockpit_info(ui, screen);
            });
        });
        let _ = ui.layer(ABOUT_DIALOG, |ui, area| {
            self.draw_about_dialog(ui, area);
        });
        let _ = ui.layer(MANAGER_INSPECT, |ui, area| {
            let instance = self
                .selected_instance_id()
                .as_deref()
                .and_then(|id| self.world.instance(id))
                .or_else(|| self.world.instance("jk-7f3a"));
            if let Some(instance) = instance {
                let (title, facts) = crate::screens::manager::inspect_facts(instance, &self.world);
                let dialog =
                    crate::screens::manager::InspectDialog::new(MANAGER_INSPECT, &title, &facts);
                dialog.draw(ui, area);
            }
        });
        let (w, h) = (
            ui.full().width.saturating_sub(4),
            ui.full().height.saturating_sub(3),
        );
        let _ = ui.layer(MANAGER_HELP, |ui, area| {
            Self::with_manager_help(|sections| {
                let help = Self::manager_help_overlay(sections, w, h);
                help.draw(ui, area, &self.manager_help_state)
            })
        });
        let _ = ui.layer(CAPSULE_HELP, |ui, area| {
            Self::with_capsule_help(|sections| {
                let help = HelpOverlay::new(CAPSULE_HELP, "Capsule", sections);
                help.draw(ui, area, &self.capsule_help_state)
            })
        });
        let tab_title = self.active_capsule_tab_title();
        let _ = ui.layer(CAPSULE_TAB_MENU, |ui, area| {
            Self::capsule_ctx_menu(
                self.capsule_tab_menu_kind,
                tab_title.clone(),
                self.capsule_tab_menu_pos,
            )
            .draw(ui, area, &self.capsule_tab_menu_state)
        });
        let _ = ui.layer(CAPSULE_COMMAND_PALETTE, |ui, area| {
            let screen = ui.full();
            Self::capsule_command_palette(screen.height).draw(
                ui,
                area,
                &self.capsule_palette_state,
                CAPSULE_COMMANDS,
            )
        });
        let info = Self::container_info_dialog();
        let _ = ui.layer(CAPSULE_CONTAINER_INFO, |ui, area| {
            let facts = self.container_info_facts();
            let rows = Self::container_info_rows(&facts);
            info.draw(ui, area, &self.container_info_state, |ui, body| {
                PropsList::new(CONTAINER_INFO_PROPS).draw(
                    ui,
                    body,
                    &self.container_info_props,
                    &rows,
                );
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
        if self.route == Route::Manager {
            let meta = self.manager_picker_meta();
            let agent_picker = Self::launch_agent_picker();
            let _ = ui.layer(crate::screens::manager::AGENT_PICKER, |ui, area| {
                agent_picker
                    .meta(&meta)
                    .draw(ui, area, &self.agent_state, &self.agent_options)
            });
        } else if self.route == Route::Capsule {
            let title = match self.pending_capsule_action {
                Some(CapsuleAction::Split(_)) => "Split ↓ Below",
                _ => "New tab",
            };
            let screen = ui.full();
            let agent_picker = Self::capsule_agent_picker(title, screen.width);
            let _ = ui.layer(crate::screens::manager::AGENT_PICKER, |ui, area| {
                agent_picker.draw(ui, area, &self.capsule_agent_state, CAPSULE_AGENT_OPTIONS)
            });
        } else {
            let agent_picker = Self::launch_agent_picker();
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
        // Tag `App::interaction`: activation flashes age on the world clock,
        // frozen under paused motion, so a Space/Enter flash on a checkbox
        // persists into the captured frame. Elapsed-clock harnesses reject
        // the sync; the live binary selects the simulation clock at boot.
        let sim_now = SimulationMoment::from_millis(self.world.now_ms().max(0) as u64);
        let _ = cx.sync_feedback_time(sim_now);
        if self.last_tick.is_none() && self.route == Route::Manager {
            cx.focus(MANAGER_LIST);
        }
        // Boot focus belongs to the route's primary control, never the
        // menu bar: a focused-but-closed bar would swallow the screen's
        // arrows (tag `app.rs:593` opens the host menu by F10 only).
        if self.last_tick.is_none() && self.route == Route::Accounts {
            cx.focus(ACCOUNTS_LIST);
        }
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
        self.paint_base_frame(ui, full);
        if self.route == Route::Intro {
            self.draw_intro(ui, full);
            return;
        }
        if self.route == Route::Outro {
            self.draw_outro(ui, full);
            return;
        }
        if self.route == Route::Capsule {
            self.draw_capsule_shell(ui, full);
            self.draw_layers(ui);
            ui.suppress_cursor();
            return;
        }

        let header = Rect::new(full.x, full.y, full.width, 1);
        let footer = Rect::new(full.x, full.bottom().saturating_sub(1), full.width, 1);
        let body = if self.route == Route::Cockpit || self.route == Route::Launch {
            Rect::new(
                full.x,
                full.y.saturating_add(2),
                full.width,
                full.height.saturating_sub(4),
            )
        } else {
            Rect::new(
                full.x.saturating_add(1),
                full.y.saturating_add(2),
                full.width.saturating_sub(2),
                full.height.saturating_sub(4),
            )
        };

        if Self::is_host(self.route) {
            self.draw_host_menu(ui, header);
        } else {
            self.draw_strip(ui, header);
        }

        match self.route {
            Route::Manager => self.draw_manager(ui, body),
            Route::Prelude => self.draw_prelude(ui, body),
            Route::Editor => self.draw_editor(ui, body),
            Route::Accounts => self.draw_accounts(ui, body),
            Route::Usage => self.draw_usage(ui, body),
            Route::Settings => self.draw_settings(ui, body),
            Route::Launch | Route::Cockpit => self.draw_launch(ui, body),
            Route::Handoff => self.draw_handoff(ui, body),
            _ => {}
        }

        self.draw_footer(ui, footer);
        self.draw_layers(ui);
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
        if self.route == Route::Outro {
            self.quit = true;
            return self.route_changed();
        }
        if self.about_open {
            cx.close_layer(ABOUT_DIALOG, Some(ActionKey::CANCEL));
            self.about_open = false;
            self.status = None;
            return self.route_changed();
        }
        if matches!(self.route, Route::Launch | Route::Cockpit) && self.cockpit_failure_open {
            self.acknowledge_launch_failure();
            return self.route_changed();
        }
        if self.cockpit_info_open {
            self.cockpit_info_open = false;
            self.status = None;
            return self.route_changed();
        }
        if self.cockpit_cancel_confirm {
            cx.close_layer(CANCEL_DIALOG, Some(ActionKey::CANCEL));
            self.cockpit_cancel_confirm = false;
            self.status = None;
            return self.route_changed();
        }
        if self.cockpit_debug_open {
            self.cockpit_debug_open = false;
            self.status = None;
            return self.route_changed();
        }
        if self.manager_inspect_open {
            self.manager_inspect_open = false;
            self.status = None;
            cx.close_layer(MANAGER_INSPECT, None);
            return self.route_changed();
        }
        if self.manager_menu_open {
            self.manager_menu_open = false;
            self.manager_menu_state = MenuState::default();
            cx.close_layer(MANAGER_MENU_BAR, None);
            self.status = None;
            return self.route_changed();
        }
        if self.manager_quit_confirm {
            self.manager_quit_confirm = false;
            self.status = None;
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
                cx.close_layer(MANAGER_HELP, None);
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
            if self.accounts_drawer_open || self.accounts.drawer_open {
                self.accounts_drawer_open = false;
                self.accounts.drawer_open = false;
                self.status = None;
                cx.focus(ACCOUNTS_LIST);
                return self.route_changed();
            }
            if self.accounts_filtered || self.accounts_filtering || self.accounts.filter.is_some() {
                self.accounts_filtered = false;
                self.accounts_filtering = false;
                self.accounts.filter = None;
                self.accounts.filter_draft.clear();
                self.accounts.filter_input = TextInputState::default();
                self.ensure_accounts_selected();
                self.status = Some("Filter cleared".into());
                return self.route_changed();
            }
            if self.accounts.form_open {
                self.accounts.close();
                self.accounts_form_stage = 0;
                self.accounts_form_enters = 0;
                self.ensure_accounts_selected();
                self.status = Some("Cancelled account registration".into());
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
            if self.editor.exit_open {
                self.editor.close_exit();
                return self.route_changed();
            }
            if self.editor.preview_open {
                self.editor.close_preview();
                self.status = Some("Not saved · keep editing".into());
                cx.focus(crate::screens::editor::SAVE);
                return self.route_changed();
            }
            if self.editor.focus != crate::screens::editor::EditorFocus::Tabs {
                self.editor.focus_tabs();
                cx.focus(crate::screens::editor::TABS);
                return self.route_changed();
            }
            if self.editor.dirty {
                self.editor.open_exit();
                cx.focus(crate::screens::editor::EXIT_CANCEL);
            } else {
                self.route = Route::Manager;
            }
            return self.route_changed();
        }
        if self.route == Route::Settings {
            if self.settings_save_preview {
                self.abort_settings_save(cx);
                return self.route_changed();
            }
            if self.settings.save_error.is_some() {
                self.settings.clear_error();
                self.status = None;
                return self.route_changed();
            }
            if self.settings.focus != SettingsFocus::Tabs {
                self.settings.focus = SettingsFocus::Tabs;
                cx.focus(crate::screens::settings::TABS);
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
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('/')),
            CMD_ACCOUNTS_FILTER,
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
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char(' ')),
            CMD_MANAGER_TOGGLE,
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
            Chord::key(KeyCode::Char('i')),
            CMD_CONTAINER_INFO,
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
        .bind(KeyPhase::Bubble, Chord::key(KeyCode::Up), CMD_NAV_UP)
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
            KeyPhase::Capture,
            Chord::key(KeyCode::Char('n')),
            CMD_NEW_WORKSPACE,
        )
        .bind(
            KeyPhase::Bubble,
            Chord::key(KeyCode::Char('n')),
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
            Chord::key(KeyCode::Char('2')),
            CMD_EDITOR_MOUNTS,
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

/// One container-info fact. The preview owns the text; [`PropsList`] owns the row.
struct ContainerInfoFact {
    label: &'static str,
    value: String,
    copyable: bool,
}

impl ContainerInfoFact {
    fn new(label: &'static str, value: String, copyable: bool) -> Self {
        Self {
            label,
            value,
            copyable,
        }
    }

    fn detached() -> Self {
        Self::new(
            "Capsule",
            "No running Capsule instance is attached.".to_owned(),
            false,
        )
    }
}

fn source_label(index: u8) -> &'static str {
    match index {
        1 => "Local agent folder",
        2 => "API key",
        _ => "1Password reference",
    }
}

/// One bare list owns these shell rows. The canvas patch is the same
/// primary-on-canvas fill the shell already painted behind them.
fn paint_owned_lines(ui: &mut Ui<'_>, id: Id, area: Rect, lines: &[&str]) {
    if area.is_empty() || lines.is_empty() {
        return;
    }
    const PARTS: [(Part, StylePatch); 2] = [
        (
            Part::CONTAINER,
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas)),
        ),
        (
            Part::LABEL,
            StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Canvas)),
        ),
    ];
    let state = ListState::default();
    List::new(id)
        .bare(true)
        .patch_part(&PARTS)
        .row(|line: &&str, row| {
            row.label_patched(line, &PARTS[1].1);
        })
        .draw(ui, area, &state, lines);
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
    fn frozen_row(path: &str) -> String {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path);
        std::fs::read_to_string(root)
            .unwrap_or_else(|err| panic!("read {path}: {err}"))
            .lines()
            .next()
            .unwrap_or("")
            .trim_end()
            .to_string()
    }

    #[test]
    fn prelude_overflow_row_uses_the_shared_scroll_fade() {
        use termrock::{Color, Theme};
        use termrock_test_support::Harness;
        let open = |w, h| {
            let mut harness = Harness::new(
                App::for_scenario_at(Scenario::FirstUse, Motion::Paused, 400),
                Theme::junie(),
                w,
                h,
            );
            let _ = harness.key(KeyCode::Char('n'));
            harness
        };
        let narrow = open(72, 20);
        let (x, y) = narrow.find("scripts/").expect("narrow file list");
        assert_eq!(
            narrow.cell(x, y).fg,
            Color::Rgb(151, 151, 152),
            "the overflowing row fades through scroll_edges"
        );
        let (sx, sy) = narrow.find("..").expect("selected source row");
        let marker = narrow.cell(sx - 3, sy);
        assert_eq!(marker.symbol(), "▎", "List marker owns the chosen row");
        assert_eq!(marker.fg, Color::Rgb(72, 224, 84));
        assert!(
            marker.modifier.contains(termrock::Modifier::BOLD),
            "chosen marker keeps the accent bold"
        );
        let (ux, uy) = narrow.find("crates/").expect("unfaded source row");
        assert_eq!(
            narrow.cell(ux, uy).fg,
            Color::Rgb(255, 255, 255),
            "only the overflowing row fades"
        );
        let wide = open(120, 40);
        let (x, y) = wide.find("scripts/").expect("wide file list");
        assert_eq!(
            wide.cell(x, y).fg,
            Color::Rgb(255, 255, 255),
            "a list that fits does not fade"
        );
        let (nx, ny) = narrow.find("Open").expect("narrow footer");
        assert!(
            !narrow
                .cell(nx, ny)
                .modifier
                .contains(termrock::Modifier::BOLD),
            "clipped prelude footer does not bold Open"
        );
        let (wx, wy) = wide.find("Open").expect("wide footer");
        assert!(
            wide.cell(wx, wy)
                .modifier
                .contains(termrock::Modifier::BOLD),
            "wide prelude footer bolds Open"
        );
    }

    #[test]
    fn host_header_facts_are_owned_by_status_bar() {
        use termrock::{PartRef, Theme};
        use termrock_test_support::Harness;
        let key = ItemKey::text("construct");
        let wide = Harness::new(
            App::for_scenario_at(Scenario::Returning, Motion::Paused, 0),
            Theme::junie(),
            120,
            40,
        );
        let wide_row = wide.row(0).trim_end().to_string();
        assert_eq!(
            wide_row,
            frozen_row(
                "baselines/tuiscotti-v1/jackin/manager/scenario-returning/120x40/truecolor.txt"
            )
        );
        let owned = wide
            .area_of_part(HOST_HEADER, PartRef::item(Part::LABEL, key))
            .expect("StatusBar owns the construct phrase");
        let (x, y) = wide.find("inside the Construct").expect("phrase");
        assert_eq!((owned.x, owned.y), (x, y));

        let narrow = Harness::new(
            App::for_scenario_at(Scenario::Returning, Motion::Paused, 0),
            Theme::junie(),
            72,
            20,
        );
        assert_eq!(
            narrow.row(0).trim_end(),
            frozen_row(
                "baselines/tuiscotti-v1/jackin/manager/scenario-returning/72x20/truecolor.txt"
            )
        );

        let cockpit = Harness::new(
            App::for_scenario_at(Scenario::LaunchRunning, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(
            cockpit.row(0).trim_end(),
            frozen_row("baselines/tuiscotti-v1/jackin/cockpit/debug/120x40/truecolor.txt")
        );
        let entering = cockpit
            .area_of_part(
                ROUTE_STRIP,
                PartRef::item(Part::LABEL, ItemKey::text("entering")),
            )
            .expect("StatusBar owns the cockpit phrase");
        let (x, y) = cockpit.find("entering the Construct").expect("phrase");
        assert_eq!((entering.x, entering.y), (x, y));

        let capsule = Harness::new(
            App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40),
            Theme::junie(),
            120,
            40,
        );
        assert_eq!(
            capsule.row(0).trim_end(),
            frozen_row("baselines/tuiscotti-v1/jackin/capsule/menu/120x40/truecolor.txt")
        );
        let role = capsule
            .area_of_part(
                HOST_HEADER,
                PartRef::item(Part::LABEL, ItemKey::text("capsule-role")),
            )
            .expect("StatusBar owns the capsule role");
        let (x, y) = capsule.find("the-architect").expect("role");
        assert!(role.contains(Position::new(x, y)));
    }

    #[test]
    fn handoff_stage_lines_are_owned_by_list() {
        use termrock::Theme;
        use termrock_test_support::Harness;
        let cockpit = App::for_scenario_at(Scenario::LaunchRunning, Motion::Paused, 0);
        assert_eq!(cockpit.route(), Route::Cockpit);
        let cockpit_frame = Harness::new(cockpit, Theme::junie(), 120, 40);
        assert!(cockpit_frame.area_of(HANDOFF_LINES).is_none());
        assert!(cockpit_frame.find("The daemon owns pane state").is_none());

        // Clean launch durations sum to 298 ticks. Frame 400 is past Ready,
        // so the constructor itself enters handoff at stage 0.
        let app = App::for_scenario_at(Scenario::LaunchRunning, Motion::Paused, 400);
        assert_eq!(app.route(), Route::Handoff);
        let h = Harness::new(app, Theme::junie(), 120, 40);
        let area = h
            .area_of(HANDOFF_LINES)
            .expect("the handoff body is a List");
        let (x, y) = h
            .find("fading launch cockpit (1/4)")
            .expect("stage 0 label");
        assert_eq!(h.cell(x, y).symbol(), "f");
        assert!(
            area.contains(Position::new(x, y)),
            "the stage label is inside the handoff list"
        );
        let (detail_x, detail_y) = h
            .find("The daemon owns pane state; the shell owns the handoff.")
            .expect("handoff detail");
        assert!(area.contains(Position::new(detail_x, detail_y)));
    }

    #[test]
    fn container_info_rows_are_owned_by_props_list() {
        use termrock::Theme;
        use termrock_test_support::Harness;
        let app = App::for_scenario(Scenario::CapsuleMulti, Motion::Paused);
        let instance = app
            .active_running_instance_id()
            .as_deref()
            .and_then(|id| app.world.instance(id))
            .expect("capsule scenario attaches a running instance");
        let uid = instance.container_uid();
        let mut h = Harness::new(app, Theme::junie(), 120, 40);
        let _ = h.ctrl('b');
        let _ = h.key(KeyCode::Char('i'));
        assert!(
            h.area_of(CONTAINER_INFO_PROPS).is_some(),
            "PropsList must register the dialog body"
        );
        let (label_x, role_y) = h.find("Role").expect("PropsList paints the role label");
        assert!(
            h.find(&uid).is_some(),
            "instance container id reaches the list"
        );
        assert!(h.find("Invocation ID").is_some());
        let gutter = label_x.saturating_sub(2);
        assert_eq!(
            h.cell(gutter, role_y.saturating_sub(2)).symbol(),
            "▎",
            "the first property row carries the list cursor"
        );
        let _ = h.key(KeyCode::Down);
        let (label_x, role_y) = h.find("Role").expect("role label stays after cursor move");
        let gutter = label_x.saturating_sub(2);
        assert_eq!(h.cell(gutter, role_y.saturating_sub(1)).symbol(), "▎");
        assert_eq!(h.cell(gutter, role_y.saturating_sub(2)).symbol(), " ");
        assert_eq!(h.app().container_info_props.cursor_index(), 1);
        let _ = h.key(KeyCode::Char('y'));
        assert_eq!(h.app().world.clipboard.as_deref(), Some(uid.as_str()));
    }

    #[test]
    fn historical_failure_and_action_labels_remain_readable_in_mono() {
        use termrock::{ColorLevel, Theme};
        use termrock_test_support::Harness;
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
