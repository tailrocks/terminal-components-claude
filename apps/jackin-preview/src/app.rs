//! Jackin Preview application shell.
//!
//! This module owns only interaction state and paints through `tui-next`'s
//! public facade.  Domain and simulation state stay in sibling modules.

use std::time::Duration;

use junie_tui::{
    ActionKey, App as TuiApp, AsItem, Button, Chord, Cx, Dialog, DialogAction, DialogState,
    FrameRead, Id, Item, ItemKey, KeyCode, KeyMap, KeyModifiers, KeyPhase, List, ListAction,
    ListState, Panel, Picker, PickerAction, PickerState, Rect, Response, SecretPolicy, Tabs,
    TabsState, TextInput, TooSmall, Ui, UpdateCause, Variant,
};

use crate::domain::account::{
    Account, CredentialSource, DetectedKind, DuplicateProbe, fingerprint, tail_of,
};
use crate::domain::agent::{Agent, Provider};
use crate::domain::instance::{DaemonSnapshot, InstanceStatus};
use crate::rain::{
    HANDOFF_LEN, INTRO_END, IntroPhase, IntroState, OutroPhase, OutroState, P1_LEN, PHRASES,
};
use crate::scenario::{Motion, Scenario};
use crate::screens::{
    accounts::AccountsState, capsule::CapsuleState, cockpit::CockpitState, editor::EditorState,
    inspect::InspectState, manager::ManagerState, prelude::PreludeState, settings::SettingsState,
    usage::UsageState,
};
use crate::sim::launch::{LaunchEvent, LaunchPlan, LaunchRun, Stage};
use crate::sim::provider;
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

const CMD_QUIT: ActionKey = ActionKey::custom("jackin.quit");
const CMD_MANAGER: ActionKey = ActionKey::custom("jackin.manager");
const CMD_ACCOUNTS: ActionKey = ActionKey::custom("jackin.accounts");
const CMD_USAGE: ActionKey = ActionKey::custom("jackin.usage");
const CMD_SETTINGS: ActionKey = ActionKey::custom("jackin.settings");
const CMD_CAPSULE: ActionKey = ActionKey::custom("jackin.capsule");
const CMD_NEW_WORKSPACE: ActionKey = ActionKey::custom("jackin.new-workspace");
const CMD_EDITOR_NEXT: ActionKey = ActionKey::custom("jackin.editor.next-tab");
const CMD_EDITOR_PREVIOUS: ActionKey = ActionKey::custom("jackin.editor.previous-tab");
const CMD_SAVE: ActionKey = ActionKey::custom("jackin.save");
const CMD_MANAGER_EXPAND: ActionKey = ActionKey::custom("jackin.manager.expand");
const CMD_EDITOR_OPEN: ActionKey = ActionKey::custom("jackin.editor.open");
const CMD_CAPSULE_PREFIX: ActionKey = ActionKey::custom("jackin.capsule.prefix");
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
    route: Route,
    motion: Motion,
    quit: bool,
    keymap: KeyMap,
    tabs_state: TabsState,
    launch_dialog: DialogState,
    role_state: PickerState,
    account_state: PickerState,
    roles: Vec<RoleOption>,
    account_options: Vec<AccountOption>,
    op_options: Vec<AccountOption>,
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
        Self {
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
            route,
            motion,
            quit: false,
            keymap: app_keymap(),
            tabs_state: TabsState::default(),
            launch_dialog: DialogState::default(),
            role_state: PickerState::default(),
            account_state: PickerState::default(),
            roles,
            account_options,
            op_options: Vec::new(),
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

    /// Whether the exit ritual has completed.
    pub const fn should_quit(&self) -> bool {
        self.quit
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

    fn role_picker() -> Picker<'static, RoleOption> {
        Picker::new(ROLE_PICKER).title("Choose a role")
    }

    fn account_picker() -> Picker<'static, AccountOption> {
        Picker::new(ACCOUNT_PICKER).title("Choose a configured account")
    }

    fn shell_panel<'a>(meta: &'a str) -> Panel<'a> {
        Panel::new(APP).title("Jackin Preview").meta(meta)
    }

    fn manager_rows(&self) -> Vec<String> {
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
        self.picker_mode = Some(PickerMode::Launch);
        self.account_state = PickerState::default();
        let spec = picker.layer(cx, &self.account_options);
        cx.open_layer(ACCOUNT_PICKER, spec);
    }

    fn open_op_picker(&mut self, cx: &mut Cx<'_>) {
        self.accounts.op_stage = 0;
        self.accounts.op_item.clear();
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
        let role_label = self.selected_role().to_owned();
        let role = Button::new(ROLE_CHOOSE, &role_label).update(cx);
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
        let picker_items = match self.picker_mode {
            Some(PickerMode::OnePassword) => &self.op_options,
            _ => &self.account_options,
        };
        let response = picker.update(cx, &mut self.account_state, picker_items);
        let action = response.action_ref().copied();
        result |= response.erase();
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
                        self.set_op_stage(3);
                        self.status = Some(account.label.clone());
                    }
                    _ => {
                        let item = self.accounts.op_item.clone();
                        let title = item
                            .split_once(" · ")
                            .map_or(item.as_str(), |(_, name)| name);
                        if let Ok(reference) = self.world.op.reference(
                            "chainargos.1password.com",
                            "Engineering",
                            match title {
                                "Anthropic · Work" | "Work" => "it_ant01",
                                "OpenAI · Codex Primary" | "Codex Primary" => "it_cdx01",
                                "OpenAI · Throttled sandbox" | "Throttled sandbox" => "it_thr01",
                                "xAI · Grok Team" | "Grok Team" => "it_grk01",
                                _ => "it_ocg01",
                            },
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
                _ => {
                    self.status = Some(format!("Selected reference · {}", account.detail));
                    self.picker_mode = None;
                    cx.close_layer(ACCOUNT_PICKER, Some(ActionKey::CONFIRM));
                }
            }
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
        let rows = self.manager_rows();
        let list = List::new(MANAGER_LIST).update(cx, &mut self.manager.list, &rows);
        let list_action = list.action_ref().copied();
        let mut result = list.erase();
        match list_action {
            Some(ListAction::Activated(_)) => {
                if self.world.running_count() == 1 {
                    self.route = Route::Capsule;
                }
                result |= Response::changed();
            }
            Some(ListAction::Chose(_)) => {
                self.manager.set_detail_open(true);
                self.status = Some("Workspaces › infra-control-plane".into());
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
        let button = Self::launch_button(self.world.workspaces.is_empty()).update(cx);
        let chosen = button.activated();
        result |= button.erase();
        if chosen {
            self.open_launch_dialog(cx);
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
                    let folder = Self::account_folder_input().update(
                        cx,
                        &mut self.accounts.folder_input,
                        &mut self.accounts.masked_input,
                    );
                    result |= folder.erase();
                }
                2 => {
                    let secret = Self::account_secret_input().update(
                        cx,
                        &mut self.accounts.secret_input,
                        &mut self.accounts.masked_input,
                    );
                    result |= secret.erase();
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
        let result = button.erase();
        if chosen {
            self.trusted = !self.trusted;
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
        let save = Self::editor_save_button().update(cx);
        let chosen = save.activated();
        let mut result = save.erase();
        if chosen {
            self.editor.dirty = false;
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
            CMD_MANAGER => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.status = Some("Detached from Capsule".into());
                    self.route = Route::Manager;
                } else {
                    self.route = if self.route == Route::Usage {
                        Route::Accounts
                    } else {
                        Route::Manager
                    };
                }
                Some(Response::changed())
            }
            CMD_ACCOUNTS => {
                if self.route == Route::Accounts {
                    self.accounts.open_new();
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
            CMD_USAGE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.capsule_usage = true;
                    self.status = Some("Usage".into());
                } else {
                    self.route = Route::Usage;
                }
                Some(Response::changed())
            }
            CMD_SETTINGS => {
                self.route = Route::Settings;
                Some(Response::changed())
            }
            CMD_CAPSULE => {
                if self.route == Route::Capsule && self.capsule_prefix {
                    self.capsule_prefix = false;
                    self.status = Some("New tab · Account for Claude Code".into());
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
                }
                Some(Response::changed())
            }
            CMD_EDITOR_OPEN if self.route == Route::Manager => {
                self.route = Route::Editor;
                self.editor = EditorState::default();
                Some(Response::changed())
            }
            CMD_CAPSULE_PREFIX if self.route == Route::Capsule => {
                self.capsule_prefix = true;
                self.status = Some("prefix… New tab · Split · Copy · Detach".into());
                Some(Response::changed())
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
                        self.route = Route::Manager;
                        self.status =
                            Some("Still inside the Construct · another instance is running".into());
                    } else if self.world.scenario == Scenario::OutroLast {
                        self.outro = Some(OutroState::new(
                            self.motion,
                            Some((self.world.now_ms().max(0) as u64) / 1000),
                            0,
                        ));
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
                self.editor.select_index(match self.editor.tab {
                    crate::screens::editor::Tab::General => 1,
                    crate::screens::editor::Tab::Mounts => 2,
                    crate::screens::editor::Tab::Roles => 3,
                    crate::screens::editor::Tab::Environments => 4,
                    crate::screens::editor::Tab::Accounts => 1,
                });
                Some(Response::changed())
            }
            CMD_EDITOR_PREVIOUS if self.route == Route::Editor => {
                self.editor.tab = crate::screens::editor::Tab::General;
                Some(Response::changed())
            }
            CMD_SAVE if self.route == Route::Editor => {
                self.editor.dirty = true;
                self.status = Some("Save workspace · preview changes before commit".into());
                Some(Response::changed())
            }
            CMD_SAVE if self.route == Route::Settings => {
                self.settings.dirty = true;
                self.status = Some("Save settings · choose a confirmation action".into());
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
                        if self.world.workspaces.is_empty() {
                            self.world
                                .workspaces
                                .push(crate::domain::workspace::Workspace::new(
                                    id,
                                    self.prelude.name(),
                                    "/Users/alexey/src/new-workspace",
                                ));
                        }
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
        let lines = [
            format!("{workspace} · edit · {tab}"),
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
        let rows = self.manager_rows();
        let list_area = Rect {
            y: area.y.saturating_add(2),
            height: area.height.saturating_sub(5),
            ..area
        };
        paint_lines(
            ui,
            Rect { height: 2, ..area },
            &[format!(
                "Current directory · {} · {} running",
                self.world.home,
                self.world.running_count()
            )],
        );
        List::new(MANAGER_LIST).draw(ui, list_area, &self.manager.list, &rows);
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
        let account_picker = match self.picker_mode {
            Some(PickerMode::OnePassword) => {
                Self::account_picker().title("Choose 1Password account")
            }
            _ => Self::account_picker(),
        };
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
        let meta = format!("scenario · {}", self.world.scenario.name());
        let _shell = Self::shell_panel(&meta);
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
        if matches!(
            self.route,
            Route::Manager | Route::Accounts | Route::Usage | Route::Settings | Route::Capsule
        ) {
            result |= self.update_navigation(cx);
        }
        result |= self.update_route(cx);
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let too_small = TooSmall::new(APP.sub("too-small"), "Jackin Preview");
        if !too_small.fits(ui.design(), full) {
            too_small.draw(ui, full);
            return;
        }
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

    fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
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
