//! Application shell: one runtime session that owns the route (intro,
//! host control plane, cockpit, handoff, Capsule, outro), the modal stack,
//! focus/hover/press state, the identity strip and the footer.

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::picker::PickerEvent;
use junie_tui::widgets::segments::{self, Segment};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::arbiter::{EntryDecision, ExitDecision};
use crate::domain::instance::InstanceStatus;
use crate::rain::{self, HANDOFF_LEN, IntroState, OutroState, handoff_stage};
use crate::scenario::{Motion, Scenario};
use crate::screens::accounts::AccountsScreen;
use crate::screens::capsule::CapsuleScreen;
use crate::screens::cockpit::CockpitScreen;
use crate::screens::editor::EditorScreen;
use crate::screens::manager::ManagerScreen;
use crate::screens::modals::{FormEvent, HelpOverlay, InfoResult};
use crate::screens::prelude::PreludeScreen;
use crate::screens::settings::SettingsScreen;
use crate::screens::usage::UsageScreen;
use crate::screens::{Cx, Go, Modal, ModalResult, ModalTag, Request, Screen};
use crate::sim::launch::LaunchPlan;
use crate::sim::world::{Msg, World};

pub const MIN_WIDTH: u16 = 72;
pub const MIN_HEIGHT: u16 = 20;

/// The canonical product mark; every brand lockup renders exactly this.
pub const BRAND_MARK: &str = "jackin❯";
const STRIP_HELP: WidgetId = WidgetId::of("strip.help");
const STRIP_USAGE: WidgetId = WidgetId::of("strip.usage");
const STRIP_ACCOUNTS: WidgetId = WidgetId::of("strip.accounts");
const STRIP_SETTINGS: WidgetId = WidgetId::of("strip.settings");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Intro,
    Manager,
    Prelude,
    Editor,
    Settings,
    Accounts,
    Usage,
    Cockpit,
    Handoff,
    Capsule,
    Outro,
}

impl Route {
    fn tick_ms(self, animating: bool) -> u64 {
        match self {
            Route::Intro | Route::Outro | Route::Handoff | Route::Cockpit => rain::TICK_MS,
            Route::Capsule => 80,
            _ => {
                if animating {
                    80
                } else {
                    200
                }
            }
        }
    }
}

struct ModalEntry {
    modal: Modal,
    tag: ModalTag,
    owner: Route,
    saved_focus: Option<WidgetId>,
}

#[derive(Default)]
pub struct Screens {
    pub manager: ManagerScreen,
    pub accounts: AccountsScreen,
    pub usage: UsageScreen,
    pub settings: Option<SettingsScreen>,
    pub editor: Option<EditorScreen>,
    pub prelude: Option<PreludeScreen>,
    pub cockpit: Option<CockpitScreen>,
    pub capsule: Option<CapsuleScreen>,
}

impl Screens {
    fn get_mut(&mut self, route: Route) -> Option<&mut dyn Screen> {
        match route {
            Route::Manager => Some(&mut self.manager),
            Route::Accounts => Some(&mut self.accounts),
            Route::Usage => Some(&mut self.usage),
            Route::Settings => self.settings.as_mut().map(|s| s as &mut dyn Screen),
            Route::Editor => self.editor.as_mut().map(|s| s as &mut dyn Screen),
            Route::Prelude => self.prelude.as_mut().map(|s| s as &mut dyn Screen),
            Route::Cockpit => self.cockpit.as_mut().map(|s| s as &mut dyn Screen),
            Route::Capsule => self.capsule.as_mut().map(|s| s as &mut dyn Screen),
            Route::Intro | Route::Outro | Route::Handoff => None,
        }
    }

    fn get(&self, route: Route) -> Option<&dyn Screen> {
        match route {
            Route::Manager => Some(&self.manager),
            Route::Accounts => Some(&self.accounts),
            Route::Usage => Some(&self.usage),
            Route::Settings => self.settings.as_ref().map(|s| s as &dyn Screen),
            Route::Editor => self.editor.as_ref().map(|s| s as &dyn Screen),
            Route::Prelude => self.prelude.as_ref().map(|s| s as &dyn Screen),
            Route::Cockpit => self.cockpit.as_ref().map(|s| s as &dyn Screen),
            Route::Capsule => self.capsule.as_ref().map(|s| s as &dyn Screen),
            Route::Intro | Route::Outro | Route::Handoff => None,
        }
    }
}

pub struct App {
    pub theme: Theme,
    pub scenario: Scenario,
    pub motion: Motion,
    pub world: World,
    pub route: Route,
    pub intro: Option<IntroState>,
    pub outro: Option<OutroState>,
    pub handoff: Option<u64>,
    pub screens: Screens,
    modals: Vec<ModalEntry>,
    pub focus: Focus,
    pub ring: FocusRing,
    pub hits: HitRegistry,
    pub hover: Option<WidgetId>,
    pub pressed: Option<WidgetId>,
    hover_suppressed: bool,
    flash: Option<(WidgetId, i64)>,
    pub status: Option<(String, Tone, i64)>,
    pub size: (u16, u16),
    pub quit: bool,
    too_small: bool,
    last_click: Option<(WidgetId, i64)>,
    intro_guard: u8,
    pub clipboard_gen: u32,
    exit_after_status: Option<i64>,
}

impl App {
    /// Pure fixture entry point: one coherent world per scenario.
    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64, theme: Theme) -> Self {
        let mut world = crate::domain::fixtures::world_for(scenario);
        world.clock.running = motion != Motion::Paused;
        let mut app = Self {
            theme,
            scenario,
            motion,
            world,
            route: Route::Manager,
            intro: None,
            outro: None,
            handoff: None,
            screens: Screens::default(),
            modals: vec![],
            focus: Focus::default(),
            ring: FocusRing::default(),
            hits: HitRegistry::default(),
            hover: None,
            pressed: None,
            hover_suppressed: false,
            flash: None,
            status: None,
            size: (0, 0),
            quit: false,
            too_small: false,
            last_click: None,
            intro_guard: 3,
            clipboard_gen: 0,
            exit_after_status: None,
        };
        app.start(frame);
        app
    }

    fn start(&mut self, frame: u64) {
        match self.scenario {
            Scenario::OutroLast if frame > 0 => {
                // capture the outro directly at a fixture tick
                self.world.arbiter.set_running(0);
                let decision = self.world.arbiter.request_exit(self.world.now_ms());
                let elapsed = match decision {
                    ExitDecision::Outro { elapsed_secs } => elapsed_secs,
                    _ => Some(8_040),
                };
                self.outro = Some(OutroState::new(self.motion, elapsed, frame));
                self.route = Route::Outro;
                return;
            }
            _ => {}
        }
        match self.world.arbiter.request_entry() {
            EntryDecision::PlayIntro => {
                if frame >= rain::INTRO_END && self.motion != Motion::Reduced {
                    self.world.arbiter.complete_entry(self.world.now_ms());
                    self.enter_manager();
                } else {
                    self.intro = Some(IntroState::new(self.motion, frame));
                    self.route = Route::Intro;
                }
            }
            EntryDecision::JoinActive { .. } => self.enter_manager(),
            EntryDecision::Duplicate => {
                self.enter_manager();
                self.set_status(
                    "Another client is entering the Construct · joined without replay",
                    Tone::Secondary,
                );
            }
            EntryDecision::Unknown(e) => {
                self.enter_manager();
                self.set_status(
                    &format!(
                        "Could not confirm running instances: {} · entered without the ritual",
                        e.label()
                    ),
                    Tone::Warning,
                );
            }
        }
        // scenario-specific starting routes
        match self.scenario {
            Scenario::AccountsMixed => self.go(Go::Accounts { select: None }),
            Scenario::LaunchRunning | Scenario::LaunchFailure => {
                let plan = if self.scenario == Scenario::LaunchFailure {
                    LaunchPlan::FailNetwork
                } else {
                    LaunchPlan::Clean
                };
                self.go(Go::Launch {
                    workspace: Some(1),
                    role: "the-architect".into(),
                    agent: crate::domain::agent::Agent::ClaudeCode,
                    account: Some("acct-claude-work".into()),
                    plan,
                });
                if frame > 0 {
                    let mut cx = Cx {
                        focus: &mut self.focus,
                        ring: &self.ring,
                        requests: vec![],
                    };
                    if let Some(c) = self.screens.cockpit.as_mut() {
                        c.seek(frame, &mut self.world, &mut cx);
                    }
                    let reqs = std::mem::take(&mut cx.requests);
                    self.apply_requests(reqs, Route::Cockpit);
                }
            }
            Scenario::CapsuleMulti | Scenario::OutroLast => self.go(Go::Attach {
                instance: "jk-7f3a".into(),
                pane: None,
            }),
            _ => {}
        }
    }

    fn enter_manager(&mut self) {
        self.route = Route::Manager;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        self.screens.manager.enter(&mut self.world, &mut cx);
        let reqs = std::mem::take(&mut cx.requests);
        self.apply_requests(reqs, Route::Manager);
    }

    pub fn set_status(&mut self, s: &str, tone: Tone) {
        self.status = Some((s.to_owned(), tone, self.world.now_ms() + 5_000));
    }

    fn animating(&self) -> bool {
        matches!(
            self.route,
            Route::Intro | Route::Outro | Route::Handoff | Route::Cockpit | Route::Capsule
        ) || self.flash.is_some()
            || self
                .screens
                .get(self.route)
                .is_some_and(|s| s.animating(&self.world))
            || self.modals.last().is_some_and(|m| {
                matches!(m.modal, Modal::Op(_) | Modal::Custom(_) | Modal::Browser(_))
            })
            || !self.world.jobs.is_empty()
            || self.world.daemons.values().any(|d| !d.panes.is_empty())
    }

    pub fn tick_interval(&self) -> std::time::Duration {
        if self.motion == Motion::Paused {
            return std::time::Duration::from_millis(500);
        }
        std::time::Duration::from_millis(self.route.tick_ms(self.animating()))
    }

    fn interaction(&self) -> Interaction {
        let flash = match self.flash {
            Some((id, until)) if self.world.now_ms() < until => Some(id),
            _ => None,
        };
        Interaction {
            focus: self.focus.current(),
            hover: self.hover,
            pressed: self.pressed,
            flash,
            focus_hidden: false,
            hover_suppressed: self.hover_suppressed,
            tick: (self.world.now_ms() / 80) as u64,
        }
    }

    // ------------------------------------------------------------- input

    pub fn handle(&mut self, input: Input) -> Outcome {
        match input {
            Input::Resize(w, h) => {
                self.size = (w, h);
                Outcome::Changed
            }
            Input::Tick => self.on_tick(),
            Input::Paste(text) => self.on_paste(&text),
            Input::Key(key) => {
                self.hover_suppressed = true;
                self.on_key(key)
            }
            Input::Mouse(m) => self.on_mouse(m),
        }
    }

    fn on_tick(&mut self) -> Outcome {
        let interval = self.route.tick_ms(true) as i64;
        let msgs = self.world.tick(interval);
        let mut out = if self.animating() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        };
        if let Some((_, until)) = self.flash
            && self.world.now_ms() >= until
        {
            self.flash = None;
            out = Outcome::Changed;
        }
        if let Some((_, _, until)) = &self.status
            && self.world.now_ms() >= *until
        {
            self.status = None;
            out = Outcome::Changed;
        }
        if let Some(at) = self.exit_after_status
            && self.world.now_ms() >= at
        {
            self.quit = true;
            return Outcome::Changed;
        }
        if self.intro_guard > 0 {
            self.intro_guard -= 1;
        }
        match self.route {
            Route::Intro => {
                if let Some(s) = self.intro.as_mut() {
                    s.on_tick();
                    if s.is_done() {
                        self.finish_intro();
                    }
                }
                return Outcome::Changed;
            }
            Route::Outro => {
                if let Some(s) = self.outro.as_mut() {
                    s.on_tick();
                    if s.is_done() {
                        self.quit = true;
                    }
                }
                return Outcome::Changed;
            }
            Route::Handoff => {
                if self.motion != Motion::Paused {
                    let h = self.handoff.unwrap_or(0) + 1;
                    self.handoff = Some(h);
                    if h >= HANDOFF_LEN {
                        self.finish_handoff();
                    }
                }
                return Outcome::Changed;
            }
            _ => {}
        }
        // modal ticks
        if let Some(top) = self.modals.last_mut() {
            let o = match &mut top.modal {
                Modal::Op(f) => f.tick(self.world.now_ms(), &self.world.op),
                Modal::Browser(b) => b.tick(self.world.now_ms()),
                Modal::Custom(c) => c.on_tick(&self.world),
                _ => Outcome::Ignored,
            };
            out = out.or(o);
        }
        // screen tick
        let route = self.route;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        if let Some(s) = self.screens.get_mut(route) {
            let o = s.on_tick(&mut self.world, &mut cx);
            out = out.or(o);
        }
        let reqs = std::mem::take(&mut cx.requests);
        out = out.or(self.apply_requests(reqs, route));
        // messages
        for m in msgs {
            out = out.or(self.dispatch_msg(m));
        }
        out
    }

    fn dispatch_msg(&mut self, m: Msg) -> Outcome {
        let route = self.route;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let mut o = match self.screens.get_mut(route) {
            Some(s) => s.on_msg(&m, &mut self.world, &mut cx),
            None => Outcome::Ignored,
        };
        if !o.consumed() && route != Route::Manager {
            o = self.screens.manager.on_msg(&m, &mut self.world, &mut cx);
        }
        if !o.consumed() && route != Route::Accounts {
            o = self.screens.accounts.on_msg(&m, &mut self.world, &mut cx);
        }
        let reqs = std::mem::take(&mut cx.requests);
        o.or(self.apply_requests(reqs, route))
    }

    fn finish_intro(&mut self) {
        self.world.arbiter.complete_entry(self.world.now_ms());
        self.intro = None;
        self.enter_manager();
    }

    fn finish_handoff(&mut self) {
        self.handoff = None;
        self.screens.cockpit = None;
        self.route = Route::Capsule;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        if let Some(c) = self.screens.capsule.as_mut() {
            c.enter(&mut self.world, &mut cx);
        }
        let reqs = std::mem::take(&mut cx.requests);
        self.apply_requests(reqs, Route::Capsule);
    }

    fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(top) = self.modals.last_mut() {
            return match &mut top.modal {
                Modal::Dialog(d) => d.on_paste(text),
                Modal::Browser(b) => b.on_paste(text),
                Modal::Form(f) => f.on_paste(text),
                _ => Outcome::Consumed,
            };
        }
        let route = self.route;
        match self.screens.get_mut(route) {
            Some(s) => s.on_paste(text, &mut self.world),
            None => Outcome::Consumed,
        }
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.too_small {
            if key.is_char('q') || key.ctrl_char('c') {
                self.quit = true;
            }
            return Outcome::Consumed;
        }
        match self.route {
            Route::Intro => {
                if self.intro_guard > 0 && self.motion != Motion::Paused {
                    return Outcome::Consumed;
                }
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    if let Some(s) = self.intro.as_mut() {
                        s.skip();
                        if s.is_done() {
                            self.finish_intro();
                        }
                    }
                    return Outcome::Changed;
                }
                if key.ctrl_char('c') {
                    self.world.arbiter.release_entry();
                    self.quit = true;
                }
                return Outcome::Consumed;
            }
            Route::Outro => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    if let Some(s) = self.outro.as_mut() {
                        s.skip();
                        if s.is_done() {
                            self.quit = true;
                        }
                    }
                    return Outcome::Changed;
                }
                if key.ctrl_char('c') {
                    self.quit = true;
                }
                return Outcome::Consumed;
            }
            Route::Handoff => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.finish_handoff();
                }
                return Outcome::Changed;
            }
            _ => {}
        }
        if self.modals.is_empty()
            && key.ctrl_char('c')
            && self.route != Route::Cockpit
            && self.route != Route::Capsule
        {
            self.world.arbiter.release_entry();
            self.quit = true;
            return Outcome::Consumed;
        }
        if !self.modals.is_empty() {
            return self.modal_key(key);
        }
        let route = self.route;
        let editing = self.screens.get(route).is_some_and(|s| s.is_editing());
        // global chords (never while editing, never inside the Capsule/Cockpit)
        let host = matches!(
            route,
            Route::Manager
                | Route::Accounts
                | Route::Usage
                | Route::Settings
                | Route::Editor
                | Route::Prelude
        );
        if host && !editing {
            match key.code {
                KeyCode::Char('?') => {
                    self.open_help();
                    return Outcome::Changed;
                }
                KeyCode::Char('u')
                    if key.plain() && matches!(route, Route::Manager | Route::Accounts) =>
                {
                    self.go(Go::Usage { select: None });
                    return Outcome::Changed;
                }
                KeyCode::Char('c')
                    if key.plain() && matches!(route, Route::Manager | Route::Usage) =>
                {
                    self.go(Go::Accounts { select: None });
                    return Outcome::Changed;
                }
                KeyCode::Char('s')
                    if key.plain()
                        && matches!(route, Route::Manager | Route::Accounts | Route::Usage) =>
                {
                    self.go(Go::Settings);
                    return Outcome::Changed;
                }
                KeyCode::Char('q') if key.ctrl() && route == Route::Manager => {
                    self.open_quit_confirm();
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let mut out = match self.screens.get_mut(route) {
            Some(s) => s.on_key(&key, &mut self.world, &mut cx),
            None => Outcome::Ignored,
        };
        let reqs = std::mem::take(&mut cx.requests);
        out = out.or(self.apply_requests(reqs, route));
        if out.consumed() {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
                && key.plain()
                && !editing
                && let Some(f) = self.focus.current()
            {
                self.flash = Some((f, self.world.now_ms() + 140));
            }
            return out;
        }
        match key.code {
            KeyCode::Tab => {
                self.focus.next(&self.ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                self.focus.prev(&self.ring);
                Outcome::Changed
            }
            KeyCode::Char('q') if key.plain() && !editing && host => {
                if route == Route::Manager {
                    self.go(Go::Quit);
                } else {
                    self.go(Go::Manager);
                }
                Outcome::Changed
            }
            KeyCode::Esc => {
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                let o = match self.screens.get_mut(route) {
                    Some(s) => s.on_esc_top(&mut self.world, &mut cx),
                    None => Outcome::Ignored,
                };
                let reqs = std::mem::take(&mut cx.requests);
                o.or(self.apply_requests(reqs, route))
            }
            _ => Outcome::Ignored,
        }
    }

    fn open_help(&mut self) {
        if !self.modals.is_empty() {
            self.set_status("Close the dialog first", Tone::Secondary);
            return;
        }
        let scope = self
            .screens
            .get(self.route)
            .map(|s| s.crumb(&self.world))
            .unwrap_or_default();
        let sections: Vec<(&str, Vec<(&str, &str)>)> = match self.route {
            Route::Capsule => vec![
                (
                    "Capsule",
                    vec![
                        ("F10", "menu bar"),
                        ("right-click", "tab / pane menu"),
                        ("Ctrl+B", "prefix"),
                        ("Ctrl+\\", "command palette"),
                        ("Ctrl+Q", "exit"),
                        ("Alt+Shift+↑↓←→", "resize split"),
                        ("click", "focus pane"),
                        ("wheel", "scrollback"),
                        ("drag", "select text"),
                        ("2×click", "select word / rename tab"),
                    ],
                ),
                (
                    "Prefix commands",
                    vec![
                        ("c", "new tab"),
                        ("n p", "next / previous tab"),
                        ("0–9", "jump to tab"),
                        ("x", "close pane"),
                        ("&", "kill tab"),
                        ("\" %", "split below / right"),
                        ("z", "zoom"),
                        ("h j k l", "move focus"),
                        ("Ctrl+L", "clear pane"),
                        ("d", "detach"),
                        ("u", "Usage"),
                        (",", "change tab title"),
                        ("m", "tab menu"),
                        ("Space :", "palette"),
                        ("r", "redraw"),
                    ],
                ),
                (
                    "Inspect changes",
                    vec![
                        ("View › Inspect changes", "open"),
                        ("Enter", "open a file's diff"),
                        ("m F2", "compact / advanced"),
                        ("d", "unified / review"),
                        ("Tab", "tree / diff"),
                    ],
                ),
                (
                    "Scrollback",
                    vec![
                        ("↑↓", "scroll"),
                        ("PgUp PgDn", "page"),
                        ("Home End", "oldest / live"),
                        ("Esc", "back to live"),
                        ("y", "copy selection"),
                    ],
                ),
            ],
            Route::Cockpit => vec![
                (
                    "Launch",
                    vec![
                        ("b", "build log"),
                        ("i", "container info"),
                        ("c", "cancel launch"),
                        ("d", "toggle debug"),
                        ("Ctrl+Q", "quit with confirmation"),
                        ("Ctrl+C", "hard abort"),
                    ],
                ),
                (
                    "Build log",
                    vec![
                        ("↑↓ j k", "scroll"),
                        ("PgUp PgDn", "page"),
                        ("End", "follow tail"),
                        ("Esc", "close"),
                    ],
                ),
            ],
            Route::Accounts => vec![
                (
                    "Accounts",
                    vec![
                        ("↑↓ j k", "move"),
                        ("←→ h l", "fold provider / back"),
                        ("Enter", "details"),
                        ("Tab", "inspector → actions"),
                        ("a", "add account…"),
                        ("e", "edit…"),
                        ("Space", "set provider default"),
                        ("d", "enable / disable"),
                        ("v", "validate"),
                        ("x", "remove…"),
                        ("r", "refresh selection"),
                        ("F5", "refresh all"),
                        ("/", "filter"),
                        ("m", "Usage overlay"),
                    ],
                ),
                (
                    "Credential sources",
                    vec![
                        (
                            "1Password",
                            "item / field reference · resolved at launch, never stored",
                        ),
                        ("Local folder", "the agent's own login folder"),
                        (
                            "Plain-text",
                            "typed once · masked · fingerprint + 4-char tail kept",
                        ),
                    ],
                ),
                (
                    "Everywhere",
                    vec![
                        ("Esc", "back one level"),
                        ("s", "Settings"),
                        ("?", "this help"),
                        ("Ctrl+Q", "quit with confirmation"),
                    ],
                ),
            ],
            Route::Usage => vec![
                (
                    "Usage",
                    vec![
                        ("↑↓ j k", "move"),
                        ("Enter", "detail"),
                        ("r", "refresh every account"),
                        ("m", "manage in Accounts"),
                        ("Esc", "close"),
                    ],
                ),
                (
                    "Reading meters",
                    vec![
                        ("━", "used share of the window"),
                        ("▲", "warning ≥ 75 %"),
                        ("!", "exhausted · error"),
                        ("stale", "last good value kept, dimmed"),
                    ],
                ),
            ],
            Route::Editor => vec![
                (
                    "Tabs",
                    vec![
                        ("←→ h l 1–5", "switch tab"),
                        ("[ ]", "switch from anywhere"),
                        ("Enter ↓", "into the tab body"),
                        ("Esc", "back to tabs · leave"),
                    ],
                ),
                (
                    "General",
                    vec![
                        ("Enter", "edit field"),
                        ("Esc", "revert"),
                        ("Space", "toggle"),
                        ("Choose…", "working directory picker"),
                    ],
                ),
                (
                    "Mounts",
                    vec![
                        ("Enter e", "edit…"),
                        ("r", "read-only"),
                        ("i 1 2 3", "isolation"),
                        ("o", "open source"),
                        ("d", "remove (u restores)"),
                        ("a", "add…"),
                    ],
                ),
                (
                    "Roles",
                    vec![
                        ("Space", "allow / disallow"),
                        ("Enter", "set default ★"),
                        ("a", "load role…"),
                    ],
                ),
                (
                    "Environments · Auth",
                    vec![
                        ("Enter e", "edit…"),
                        ("m", "show / mask"),
                        ("p", "re-pick from 1Password"),
                        ("s", "scope…"),
                        ("Space", "cycle auth mode"),
                        ("d", "remove · reset"),
                        ("u", "undo row"),
                    ],
                ),
                (
                    "Save",
                    vec![
                        ("Ctrl+S", "preview then save"),
                        ("Esc", "leave · asks when dirty"),
                    ],
                ),
            ],
            Route::Settings => vec![
                (
                    "Tabs",
                    vec![
                        ("←→ h l 1–5", "switch tab"),
                        ("[ ]", "switch from anywhere"),
                        ("Enter ↓", "into the tab body"),
                        ("Esc", "back to tabs · leave"),
                    ],
                ),
                (
                    "Mounts · Environments · Auth",
                    vec![
                        ("Enter e", "edit…"),
                        ("s", "scope global ⇄ role"),
                        ("r i", "read-only · isolation"),
                        ("m p", "show · 1Password"),
                        ("Space", "cycle auth mode"),
                        ("d", "remove · reset"),
                        ("u", "undo row"),
                        ("c", "manage accounts (Auth)"),
                    ],
                ),
                (
                    "Trust",
                    vec![
                        ("Space", "trust / untrust a role source"),
                        ("o", "open the source"),
                    ],
                ),
                (
                    "Save",
                    vec![
                        ("Ctrl+S", "preview then save"),
                        ("Esc", "leave · asks when dirty"),
                    ],
                ),
            ],
            Route::Prelude => vec![(
                "New workspace",
                vec![
                    ("Enter", "open / choose"),
                    ("Backspace", "up one directory"),
                    ("Space", "choose directory"),
                    ("g", "Git URL…"),
                    ("Tab", "next control"),
                    ("Esc", "back one step · cancel"),
                ],
            )],
            _ => vec![
                (
                    "Workspaces",
                    vec![
                        ("↑↓ j k", "move"),
                        ("←→ h l", "collapse / expand"),
                        ("Space", "fold workspace"),
                        ("Enter", "launch"),
                        ("e", "edit workspace"),
                        ("n", "new workspace"),
                        ("d", "delete…"),
                        ("w", "prewarm"),
                        ("o", "open in GitHub"),
                        ("* -", "expand / collapse all"),
                        ("Tab", "details"),
                    ],
                ),
                (
                    "Instances",
                    vec![
                        ("Enter r", "reconnect / restore"),
                        ("a", "new session"),
                        ("x", "open shell"),
                        ("i", "inspect container"),
                        ("t", "stop"),
                        ("p", "purge…"),
                    ],
                ),
                (
                    "Everywhere",
                    vec![
                        ("Tab Shift+Tab", "next / previous"),
                        ("Esc", "back one level"),
                        ("u", "Usage overlay"),
                        ("c", "Accounts & Usage"),
                        ("s", "Settings"),
                        ("F5", "refresh now"),
                        ("?", "this help"),
                        ("q", "back / quit"),
                        ("Ctrl+Q", "quit with confirmation"),
                        ("Ctrl+C", "quit immediately"),
                    ],
                ),
                (
                    "Editor and Settings",
                    vec![
                        ("←→ 1–5 [ ]", "switch tab"),
                        ("Ctrl+S", "save (preview first)"),
                        ("Space", "toggle / cycle"),
                        ("a e d", "add / edit / remove"),
                        ("m p s", "mask · 1Password · scope"),
                    ],
                ),
                (
                    "Mouse",
                    vec![
                        ("click", "select · 2× activates"),
                        ("wheel", "scroll under pointer"),
                        ("drag", "seam · scrollbar thumb"),
                    ],
                ),
            ],
        };
        let h = HelpOverlay::new(WidgetId::of("help"), &scope, sections);
        self.push_modal(Modal::Help(h), ModalTag::new("help"), self.route);
    }

    fn open_quit_confirm(&mut self) {
        let n = self.world.running_count();
        let body = if n > 0 {
            format!(
                "{} keep running in the Construct. The host console closes; reconnect from a new terminal.",
                crate::screens::plural(n, "instance keeps", "instances keep")
                    .replace("instances keep", "instances")
                    .replace("instance keeps", "instance")
            )
        } else {
            "No instances are running. The pending Construct entry is released.".into()
        };
        let d = Dialog::confirm(WidgetId::of("quit"), "Exit jackin❯?", &body, "Quit");
        self.push_modal(Modal::Dialog(d), ModalTag::new("quit"), self.route);
    }

    // ------------------------------------------------------------ modals

    fn push_modal(&mut self, modal: Modal, tag: ModalTag, owner: Route) {
        let initial = match &modal {
            Modal::Dialog(d) => Some(d.initial_focus),
            Modal::Picker(p) => Some(p.id),
            Modal::Browser(b) => Some(b.initial_focus()),
            Modal::Choice(c) => Some(c.initial_focus()),
            Modal::Form(f) => Some(f.initial_focus()),
            Modal::Op(o) => Some(o.picker.id),
            Modal::Info(i) => Some(i.initial_focus()),
            Modal::Help(h) => Some(h.id),
            Modal::Custom(c) => Some(c.initial_focus()),
        };
        self.modals.push(ModalEntry {
            modal,
            tag,
            owner,
            saved_focus: self.focus.current(),
        });
        self.focus.set(initial);
        self.hover = None;
        self.pressed = None;
        if let Some(c) = self.screens.capsule.as_mut() {
            c.dialog_open = true;
        }
    }

    fn pop_modal(&mut self) -> Option<ModalEntry> {
        let e = self.modals.pop();
        if let Some(e) = &e {
            self.focus.set(e.saved_focus);
        }
        if self.modals.is_empty()
            && let Some(c) = self.screens.capsule.as_mut()
        {
            c.dialog_open = false;
        }
        e
    }

    fn deliver(&mut self, entry: ModalEntry, result: ModalResult) -> Outcome {
        let owner = entry.owner;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let o = match self.screens.get_mut(owner) {
            Some(s) => s.on_modal(&entry.tag, result, &mut self.world, &mut cx),
            None => Outcome::Changed,
        };
        let reqs = std::mem::take(&mut cx.requests);
        o.or(self.apply_requests(reqs, owner)).or(Outcome::Changed)
    }

    fn refresh_picker(&mut self) {
        let Some(top) = self.modals.last_mut() else {
            return;
        };
        let Modal::Picker(p) = &mut top.modal else {
            return;
        };
        let tag = top.tag.clone();
        let query = p.query.clone();
        let owner = top.owner;
        let items = self
            .screens
            .get_mut(owner)
            .and_then(|s| s.picker_items(&tag, &query, &self.world));
        if let Some(items) = items
            && let Some(top) = self.modals.last_mut()
            && let Modal::Picker(p) = &mut top.modal
        {
            let n = items.len();
            p.set_items(items);
            if tag.kind == "palette" {
                p.scope = Some(format!("{n} of 20"));
            }
        }
    }

    fn form_changed(&mut self) {
        let Some(top) = self.modals.last_mut() else {
            return;
        };
        let tag = top.tag.clone();
        let owner = top.owner;
        let Modal::Form(f) = &mut top.modal else {
            return;
        };
        // temporarily take the form out to hand both to the screen
        let mut form = std::mem::replace(
            f,
            crate::screens::modals::FormDialog::new(WidgetId::of("tmp"), "", vec![]),
        );
        if let Some(s) = self.screens.get_mut(owner) {
            s.form_changed(&tag, &mut form, &self.world);
        }
        if let Some(top) = self.modals.last_mut()
            && let Modal::Form(f) = &mut top.modal
        {
            *f = form;
        }
    }

    fn modal_key(&mut self, key: Key) -> Outcome {
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
        // help has top priority and never coexists; Ctrl+Q inside a Capsule dialog dismisses
        let now = self.world.now_ms();
        match &mut top.modal {
            Modal::Dialog(d) => {
                let out = d.on_key(&key, &mut self.focus, &self.ring);
                if let Some(result) = d.result {
                    let text = match &d.body {
                        DialogBody::Input(i) => Some(i.text().to_owned()),
                        _ => None,
                    };
                    let action = match result {
                        DialogResult::Action(i) => Some(i),
                        DialogResult::Cancelled => None,
                    };
                    let cancel = action.is_none() || action == d.cancel_index;
                    let entry = self.pop_modal().unwrap();
                    if entry.tag.kind == "quit" && entry.owner == self.route {
                        if !cancel {
                            self.world.arbiter.release_entry();
                            self.quit = true;
                        }
                        return Outcome::Changed;
                    }
                    return self.deliver(entry, ModalResult::Dialog { action, text });
                }
                out.or(Outcome::Consumed)
            }
            Modal::Picker(p) => {
                let (o, ev) = p.on_key(&key);
                match ev {
                    Some(PickerEvent::QueryChanged) => {
                        self.refresh_picker();
                        Outcome::Changed
                    }
                    Some(PickerEvent::Chosen(i)) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::Picked(i))
                    }
                    Some(PickerEvent::ChosenAlt(i)) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::PickedAlt(i))
                    }
                    Some(PickerEvent::NextScope) => {
                        let tag = top.tag.clone();
                        let owner = top.owner;
                        let mut cx = Cx {
                            focus: &mut self.focus,
                            ring: &self.ring,
                            requests: vec![],
                        };
                        if let Some(s) = self.screens.get_mut(owner) {
                            s.on_modal(&tag, ModalResult::Scope, &mut self.world, &mut cx);
                        }
                        let reqs = std::mem::take(&mut cx.requests);
                        self.apply_requests(reqs, owner);
                        self.refresh_picker();
                        Outcome::Changed
                    }
                    Some(PickerEvent::Cancelled) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::Cancelled)
                    }
                    Some(PickerEvent::Back) | Some(PickerEvent::Secondary(_)) | None => {
                        o.or(Outcome::Consumed)
                    }
                }
            }
            Modal::Browser(b) => {
                let o = b.on_key(&key, &mut self.focus, &self.ring, &self.world);
                if let Some(r) = b.result.take() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Browser(r));
                }
                o.or(Outcome::Consumed)
            }
            Modal::Choice(c) => {
                let o = c.on_key(&key, &mut self.focus, &self.ring);
                if let Some(r) = c.result.take() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Choice(r));
                }
                o.or(Outcome::Consumed)
            }
            Modal::Form(f) => {
                let o = f.on_key(&key, &mut self.focus, &self.ring);
                let events = std::mem::take(&mut f.events);
                let mut out = o.or(Outcome::Consumed);
                for ev in events {
                    match ev {
                        FormEvent::Changed(_) => {
                            self.form_changed();
                            out = Outcome::Changed;
                        }
                        FormEvent::Choose(name) => {
                            let tag = top_tag(&self.modals);
                            let owner = top_owner(&self.modals);
                            let values = form_values(&self.modals);
                            let mut cx = Cx {
                                focus: &mut self.focus,
                                ring: &self.ring,
                                requests: vec![],
                            };
                            if let Some(s) = self.screens.get_mut(owner) {
                                s.on_modal(
                                    &tag,
                                    ModalResult::FormAction(format!("choose:{name}"), values),
                                    &mut self.world,
                                    &mut cx,
                                );
                            }
                            let reqs = std::mem::take(&mut cx.requests);
                            self.apply_requests(reqs, owner);
                            return Outcome::Changed;
                        }
                        FormEvent::Action(name) => {
                            let tag = top_tag(&self.modals);
                            let owner = top_owner(&self.modals);
                            let values = form_values(&self.modals);
                            let mut cx = Cx {
                                focus: &mut self.focus,
                                ring: &self.ring,
                                requests: vec![],
                            };
                            if let Some(s) = self.screens.get_mut(owner) {
                                s.on_modal(
                                    &tag,
                                    ModalResult::FormAction(name, values),
                                    &mut self.world,
                                    &mut cx,
                                );
                            }
                            let reqs = std::mem::take(&mut cx.requests);
                            self.apply_requests(reqs, owner);
                            return Outcome::Changed;
                        }
                        FormEvent::Save => {
                            let values = form_values(&self.modals);
                            let entry = self.pop_modal().unwrap();
                            return self.deliver(entry, ModalResult::Form(Some(values)));
                        }
                        FormEvent::Cancel => {
                            let entry = self.pop_modal().unwrap();
                            return self.deliver(entry, ModalResult::Form(None));
                        }
                    }
                }
                out
            }
            Modal::Op(f) => {
                let o = f.on_key(&key, now, &self.world.op);
                if let Some(r) = f.result.take() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Op(r));
                }
                o.or(Outcome::Consumed)
            }
            Modal::Info(i) => {
                let o = i.on_key(&key, &mut self.focus, &self.ring);
                if let Some(r) = i.result.take() {
                    let copy = matches!(r, InfoResult::Copy(_));
                    if copy {
                        // copying keeps the dialog open
                        let tag = top_tag(&self.modals);
                        let owner = top_owner(&self.modals);
                        if let InfoResult::Copy(v) = &r {
                            self.world.clipboard = Some(v.clone());
                            self.clipboard_gen += 1;
                            self.set_status("Copied to the preview clipboard", Tone::Secondary);
                        }
                        let _ = (tag, owner);
                        return Outcome::Changed;
                    }
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Info(r));
                }
                o.or(Outcome::Consumed)
            }
            Modal::Help(h) => {
                let o = h.on_key(&key);
                if h.closed {
                    self.pop_modal();
                }
                o.or(Outcome::Consumed)
            }
            Modal::Custom(c) => {
                let o = c.on_key(&key, &mut self.focus, &self.ring, &self.world);
                if let Some(r) = c.done() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, r);
                }
                o.or(Outcome::Consumed)
            }
        }
    }

    // ------------------------------------------------------------- mouse

    fn on_mouse(&mut self, m: Mouse) -> Outcome {
        if matches!(self.route, Route::Intro | Route::Outro | Route::Handoff) {
            return Outcome::Ignored;
        }
        match m.kind {
            MouseKind::Move => {
                let was = self.hover;
                let suppressed = self.hover_suppressed;
                self.hover_suppressed = false;
                self.hover = self.hits.hit(m.pos);
                if self.hover != was || suppressed {
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            MouseKind::Drag => {
                self.hover = self.hits.hit(m.pos);
                let Some(pressed) = self.pressed else {
                    return Outcome::Ignored;
                };
                if let Some(top) = self.modals.last_mut() {
                    return match &mut top.modal {
                        Modal::Info(i)
                            if pressed == junie_tui::widgets::scrollbar::id_for(i.id) =>
                        {
                            i.on_click(pressed, m.pos, &mut self.focus)
                        }
                        Modal::Custom(c) => c.on_drag(pressed, m.pos),
                        _ => Outcome::Consumed,
                    };
                }
                let route = self.route;
                match self.screens.get_mut(route) {
                    Some(s) => s.on_drag(pressed, m.pos, &mut self.world),
                    None => Outcome::Ignored,
                }
            }
            MouseKind::Down => {
                let hit = self.hits.hit(m.pos);
                self.pressed = hit;
                self.hover = hit;
                let Some(id) = hit else {
                    return if self.modals.is_empty() {
                        Outcome::Ignored
                    } else {
                        Outcome::Consumed
                    };
                };
                if self.modals.is_empty() && self.ring.contains(id) {
                    self.focus.focus(id);
                }
                if self.modals.is_empty() {
                    let route = self.route;
                    let mut cx = Cx {
                        focus: &mut self.focus,
                        ring: &self.ring,
                        requests: vec![],
                    };
                    let o = match self.screens.get_mut(route) {
                        Some(s) => s.on_press(id, m.pos, &mut self.world, &mut cx),
                        None => Outcome::Ignored,
                    };
                    let reqs = std::mem::take(&mut cx.requests);
                    let _ = o.or(self.apply_requests(reqs, route));
                }
                Outcome::Changed
            }
            MouseKind::Up => {
                let hit = self.hits.hit(m.pos);
                let pressed = self.pressed.take();
                let route = self.route;
                // drags end here
                if let Some(p) = pressed
                    && self.modals.is_empty()
                {
                    let mut cx = Cx {
                        focus: &mut self.focus,
                        ring: &self.ring,
                        requests: vec![],
                    };
                    let o = match self.screens.get_mut(route) {
                        Some(s) => s.on_release(p, m.pos, &mut self.world, &mut cx),
                        None => Outcome::Ignored,
                    };
                    let reqs = std::mem::take(&mut cx.requests);
                    let o = o.or(self.apply_requests(reqs, route));
                    if o == Outcome::Changed && hit != Some(p) {
                        return o;
                    }
                }
                let Some(id) = hit else {
                    // outside a cancelable modal cancels it
                    if !self.modals.is_empty() && pressed.is_none() {
                        return self.modal_outside_click();
                    }
                    return Outcome::Changed;
                };
                if pressed != Some(id) {
                    return Outcome::Changed;
                }
                self.flash = Some((id, self.world.now_ms() + 140));
                let double = self
                    .last_click
                    .take()
                    .is_some_and(|(lid, at)| lid == id && self.world.now_ms() - at < 500);
                self.last_click = Some((id, self.world.now_ms()));
                if !self.modals.is_empty() {
                    return self.modal_click(id, m.pos);
                }
                if id == STRIP_HELP {
                    self.open_help();
                    return Outcome::Changed;
                }
                if id == STRIP_USAGE {
                    self.go(Go::Usage { select: None });
                    return Outcome::Changed;
                }
                if id == STRIP_ACCOUNTS {
                    self.go(Go::Accounts { select: None });
                    return Outcome::Changed;
                }
                if id == STRIP_SETTINGS {
                    self.go(Go::Settings);
                    return Outcome::Changed;
                }
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                let o = match self.screens.get_mut(route) {
                    Some(s) => {
                        if double {
                            let d = s.on_double_click(id, m.pos, &mut self.world, &mut cx);
                            if d.consumed() {
                                d
                            } else {
                                s.on_click(id, m.pos, &mut self.world, &mut cx)
                            }
                        } else {
                            s.on_click(id, m.pos, &mut self.world, &mut cx)
                        }
                    }
                    None => Outcome::Ignored,
                };
                let reqs = std::mem::take(&mut cx.requests);
                o.or(self.apply_requests(reqs, route)).or(Outcome::Changed)
            }
            MouseKind::Secondary => {
                let hit = self.hits.hit(m.pos);
                self.hover = hit;
                if !self.modals.is_empty() {
                    return Outcome::Consumed;
                }
                let Some(id) = hit else {
                    return Outcome::Ignored;
                };
                let route = self.route;
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                let o = match self.screens.get_mut(route) {
                    Some(s) => s.on_secondary(id, m.pos, &mut self.world, &mut cx),
                    None => Outcome::Ignored,
                };
                let reqs = std::mem::take(&mut cx.requests);
                o.or(self.apply_requests(reqs, route))
            }
            MouseKind::WheelUp
            | MouseKind::WheelDown
            | MouseKind::WheelLeft
            | MouseKind::WheelRight => {
                let delta = match m.kind {
                    MouseKind::WheelUp | MouseKind::WheelLeft => -3,
                    _ => 3,
                };
                if let Some(top) = self.modals.last_mut() {
                    return match &mut top.modal {
                        Modal::Picker(p) => p.on_wheel(delta),
                        Modal::Browser(b) => b.on_wheel(delta),
                        Modal::Form(f) => f.on_wheel(delta),
                        Modal::Op(o) => o.on_wheel(delta),
                        Modal::Info(i) => i.on_wheel(delta),
                        Modal::Help(h) => h.on_wheel(delta),
                        Modal::Custom(c) => c.on_wheel(delta, m.pos),
                        Modal::Dialog(_) | Modal::Choice(_) => Outcome::Consumed,
                    };
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                let route = self.route;
                match self.screens.get_mut(route) {
                    Some(s) => s.on_wheel(id, delta, m.pos, &mut self.world),
                    None => Outcome::Ignored,
                }
            }
        }
    }

    fn modal_outside_click(&mut self) -> Outcome {
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
        match &mut top.modal {
            Modal::Dialog(d) => {
                // typed acknowledgements and destructive prompts stay open
                if matches!(d.body, DialogBody::Facts { ack: Some(_), .. }) {
                    return Outcome::Consumed;
                }
                let out = d.on_click_outside();
                if let Some(result) = d.result {
                    let action = match result {
                        DialogResult::Action(i) => Some(i),
                        DialogResult::Cancelled => None,
                    };
                    let entry = self.pop_modal().unwrap();
                    if entry.tag.kind == "quit" {
                        return Outcome::Changed;
                    }
                    return self.deliver(entry, ModalResult::Dialog { action, text: None });
                }
                out
            }
            Modal::Picker(_) | Modal::Op(_) => {
                let entry = self.pop_modal().unwrap();
                self.deliver(entry, ModalResult::Cancelled)
            }
            Modal::Help(_) => {
                self.pop_modal();
                Outcome::Changed
            }
            Modal::Info(_) => {
                let entry = self.pop_modal().unwrap();
                self.deliver(entry, ModalResult::Info(InfoResult::Closed))
            }
            Modal::Custom(c) => {
                if c.cancel_on_outside_click() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Custom("close".into()));
                }
                Outcome::Consumed
            }
            Modal::Browser(_) | Modal::Choice(_) | Modal::Form(_) => Outcome::Consumed,
        }
    }

    fn modal_click(&mut self, id: WidgetId, pos: Position) -> Outcome {
        let now = self.world.now_ms();
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
        match &mut top.modal {
            Modal::Dialog(d) => {
                let out = d.on_click(id, pos, &mut self.focus);
                if let Some(result) = d.result {
                    let text = match &d.body {
                        DialogBody::Input(i) => Some(i.text().to_owned()),
                        _ => None,
                    };
                    let action = match result {
                        DialogResult::Action(i) => Some(i),
                        DialogResult::Cancelled => None,
                    };
                    let cancel = action.is_none() || action == d.cancel_index;
                    let entry = self.pop_modal().unwrap();
                    if entry.tag.kind == "quit" {
                        if !cancel {
                            self.quit = true;
                        }
                        return Outcome::Changed;
                    }
                    return self.deliver(entry, ModalResult::Dialog { action, text });
                }
                out.or(Outcome::Changed)
            }
            Modal::Picker(p) => {
                if let Some(PickerEvent::Chosen(i)) = p.on_click(id) {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Picked(i));
                }
                Outcome::Changed
            }
            Modal::Browser(b) => {
                let o = b.on_click(id, pos, &mut self.focus, &self.world);
                if let Some(r) = b.result.take() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Browser(r));
                }
                o.or(Outcome::Changed)
            }
            Modal::Choice(c) => {
                let o = c.on_click(id, &mut self.focus);
                if let Some(r) = c.result.take() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Choice(r));
                }
                o.or(Outcome::Changed)
            }
            Modal::Form(f) => {
                let o = f.on_click(id, pos, &mut self.focus);
                let events = std::mem::take(&mut f.events);
                for ev in events {
                    match ev {
                        FormEvent::Changed(_) => self.form_changed(),
                        FormEvent::Choose(name) => {
                            let tag = top_tag(&self.modals);
                            let owner = top_owner(&self.modals);
                            let values = form_values(&self.modals);
                            let mut cx = Cx {
                                focus: &mut self.focus,
                                ring: &self.ring,
                                requests: vec![],
                            };
                            if let Some(s) = self.screens.get_mut(owner) {
                                s.on_modal(
                                    &tag,
                                    ModalResult::FormAction(format!("choose:{name}"), values),
                                    &mut self.world,
                                    &mut cx,
                                );
                            }
                            let reqs = std::mem::take(&mut cx.requests);
                            self.apply_requests(reqs, owner);
                            return Outcome::Changed;
                        }
                        FormEvent::Action(name) => {
                            let tag = top_tag(&self.modals);
                            let owner = top_owner(&self.modals);
                            let values = form_values(&self.modals);
                            let mut cx = Cx {
                                focus: &mut self.focus,
                                ring: &self.ring,
                                requests: vec![],
                            };
                            if let Some(s) = self.screens.get_mut(owner) {
                                s.on_modal(
                                    &tag,
                                    ModalResult::FormAction(name, values),
                                    &mut self.world,
                                    &mut cx,
                                );
                            }
                            let reqs = std::mem::take(&mut cx.requests);
                            self.apply_requests(reqs, owner);
                            return Outcome::Changed;
                        }
                        FormEvent::Save => {
                            let values = form_values(&self.modals);
                            let entry = self.pop_modal().unwrap();
                            return self.deliver(entry, ModalResult::Form(Some(values)));
                        }
                        FormEvent::Cancel => {
                            let entry = self.pop_modal().unwrap();
                            return self.deliver(entry, ModalResult::Form(None));
                        }
                    }
                }
                o.or(Outcome::Changed)
            }
            Modal::Op(f) => {
                let o = f.on_click(id, now, &self.world.op);
                if let Some(r) = f.result.take() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Op(r));
                }
                o.or(Outcome::Changed)
            }
            Modal::Info(i) => {
                let o = i.on_click(id, pos, &mut self.focus);
                if let Some(r) = i.result.take() {
                    if let InfoResult::Copy(v) = &r {
                        self.world.clipboard = Some(v.clone());
                        self.clipboard_gen += 1;
                        self.set_status("Copied to the preview clipboard", Tone::Secondary);
                        return Outcome::Changed;
                    }
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Info(r));
                }
                o.or(Outcome::Changed)
            }
            Modal::Help(_) => Outcome::Changed,
            Modal::Custom(c) => {
                let o = c.on_click(id, pos, &mut self.focus, &self.world);
                if let Some(r) = c.done() {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, r);
                }
                o.or(Outcome::Changed)
            }
        }
    }

    // ----------------------------------------------------------- requests

    fn apply_requests(&mut self, requests: Vec<Request>, owner: Route) -> Outcome {
        let mut out = Outcome::Ignored;
        for r in requests {
            out = Outcome::Changed;
            match r {
                Request::Status(s) => self.set_status(&s, Tone::Secondary),
                Request::Error(s) => self.set_status(&s, Tone::Error),
                Request::Open(m, tag) => self.push_modal(*m, tag, owner),
                Request::Close => {
                    self.pop_modal();
                }
                Request::Go(g) => self.go(g),
                Request::Help => self.open_help(),
                Request::Copy(s) => {
                    self.world.clipboard = Some(s);
                    self.clipboard_gen += 1;
                    self.set_status("Copied to the preview clipboard", Tone::Secondary);
                }
                Request::WithForm(f) => {
                    if let Some(top) = self.modals.last_mut()
                        && let Modal::Form(form) = &mut top.modal
                    {
                        f(form);
                    }
                }
            }
        }
        out
    }

    pub fn go(&mut self, g: Go) {
        let now = self.world.now_ms();
        match g {
            Go::Manager => {
                self.route = Route::Manager;
                self.screens.editor = None;
                self.screens.settings = None;
                self.screens.prelude = None;
                self.enter_route();
            }
            Go::Settings => {
                self.screens.settings = Some(SettingsScreen::new(&self.world));
                self.route = Route::Settings;
                self.enter_route();
            }
            Go::Accounts { select } => {
                self.route = Route::Accounts;
                self.screens.accounts.select(select);
                self.enter_route();
            }
            Go::Usage { select } => {
                self.route = Route::Usage;
                self.screens.usage.select(select);
                self.enter_route();
            }
            Go::Editor { workspace, pending } => {
                self.screens.editor = Some(EditorScreen::new(
                    &self.world,
                    workspace,
                    pending.map(|b| *b),
                ));
                self.route = Route::Editor;
                self.enter_route();
            }
            Go::Prelude => {
                self.screens.prelude = Some(PreludeScreen::new(&self.world));
                self.route = Route::Prelude;
                self.enter_route();
            }
            Go::Launch {
                workspace,
                role,
                agent,
                account,
                plan,
            } => {
                self.screens.cockpit = Some(CockpitScreen::new(
                    &self.world,
                    workspace,
                    role,
                    agent,
                    account,
                    plan,
                    self.motion,
                ));
                self.route = Route::Cockpit;
                self.enter_route();
            }
            Go::Attach { instance, pane } => {
                if self
                    .world
                    .instance(&instance)
                    .is_none_or(|i| i.status != InstanceStatus::Running)
                {
                    self.set_status(
                        &format!(
                            "Cannot attach: {} is not running",
                            instance.trim_start_matches("jk-")
                        ),
                        Tone::Error,
                    );
                    return;
                }
                if let Some(d) = self.world.daemons.get_mut(&instance) {
                    d.attached_by = Some("this terminal".into());
                }
                self.screens.capsule = Some(CapsuleScreen::new(&instance, &self.world, pane));
                if self.route == Route::Cockpit {
                    self.handoff = Some(0);
                    self.route = Route::Handoff;
                    if self.motion == Motion::Reduced {
                        self.finish_handoff();
                    }
                } else {
                    self.route = Route::Capsule;
                    self.enter_route();
                    let name = self
                        .world
                        .daemons
                        .get(&instance)
                        .map(|d| d.workspace.clone())
                        .unwrap_or_default();
                    self.set_status(
                        &format!("Attached to {name} · tabs and panes restored"),
                        Tone::Secondary,
                    );
                }
            }
            Go::NewSession {
                instance,
                agent,
                account,
            } => {
                if let Some(d) = self.world.daemons.get_mut(&instance) {
                    d.new_tab(agent, account.clone(), now, false);
                }
                crate::domain::fixtures::refresh_snapshots(&mut self.world);
                self.go(Go::Attach {
                    instance,
                    pane: None,
                });
                let label = agent
                    .map(|a| a.label().to_owned())
                    .unwrap_or("Shell".into());
                self.set_status(&format!("Started {label} in a new tab"), Tone::Secondary);
            }
            Go::Detach => {
                let inst = self
                    .screens
                    .capsule
                    .as_ref()
                    .map(|c| c.instance.clone())
                    .unwrap_or_default();
                if let Some(d) = self.world.daemons.get_mut(&inst) {
                    d.attached_by = None;
                }
                self.screens.capsule = None;
                self.route = Route::Manager;
                self.screens.manager.select_instance(&inst, &self.world);
                self.enter_route();
                let name = self
                    .world
                    .daemons
                    .get(&inst)
                    .map(|d| d.workspace.clone())
                    .unwrap_or_default();
                let n = self.world.running_count();
                self.set_status(
                    &format!(
                        "Detached · {name} keeps running · {} in the Construct",
                        crate::screens::plural(n, "instance", "instances")
                    ),
                    Tone::Secondary,
                );
            }
            Go::InstanceEnded { instance, purge } => {
                let dirty = self.world.instance(&instance).is_some_and(|i| i.is_dirty());
                let now_secs = self.world.clock.now_secs();
                if let Some(i) = self.world.instance_mut(&instance) {
                    i.status = if purge {
                        InstanceStatus::Purged
                    } else if dirty {
                        InstanceStatus::PreservedDirty
                    } else {
                        InstanceStatus::CleanExited
                    };
                    i.last_seen_secs = now_secs;
                }
                self.world.daemons.remove(&instance);
                crate::domain::fixtures::refresh_snapshots(&mut self.world);
                self.world.sync_arbiter();
                self.screens.capsule = None;
                self.leave_construct(&instance);
            }
            Go::LaunchFailedAck { instance } => {
                self.screens.cockpit = None;
                if self.world.running_count() > 0 {
                    self.route = Route::Manager;
                    if let Some(i) = instance {
                        self.screens.manager.select_instance(&i, &self.world);
                    }
                    self.enter_route();
                    self.set_status(
                        &format!(
                            "Launch failed · {} still running in the Construct",
                            crate::screens::plural(
                                self.world.running_count(),
                                "instance",
                                "instances"
                            )
                        ),
                        Tone::Error,
                    );
                } else {
                    // a fresh Construct with no survivor: acknowledge, then the one outro
                    self.leave_construct(instance.as_deref().unwrap_or(""));
                }
            }
            Go::Quit => {
                self.world.arbiter.release_entry();
                self.quit = true;
            }
        }
    }

    /// Foreground instance left: decide between the outro, still-inside
    /// feedback, an already-ended Construct, or a fail-closed return.
    fn leave_construct(&mut self, instance: &str) {
        let decision = self.world.arbiter.request_exit(self.world.now_ms());
        match decision {
            ExitDecision::Outro { elapsed_secs } => {
                self.outro = Some(OutroState::new(self.motion, elapsed_secs, 0));
                self.route = Route::Outro;
                self.modals.clear();
            }
            ExitDecision::StillInside { remaining } => {
                self.route = Route::Manager;
                self.screens.manager.select_instance(instance, &self.world);
                self.enter_route();
                let rows: Vec<String> = self
                    .world
                    .running()
                    .iter()
                    .map(|i| {
                        let ws = i
                            .workspace
                            .and_then(|x| self.world.workspace(x))
                            .map(|x| x.name.clone())
                            .unwrap_or("current directory".into());
                        format!(
                            "{ws} › {} · {}",
                            i.role,
                            self.world
                                .mask_path(&format!("{}/src/{ws}", self.world.home))
                        )
                    })
                    .collect();
                let text = format!(
                    "Still inside the Construct · {}",
                    crate::screens::plural(remaining, "instance remains", "instances remain")
                );
                self.screens.manager.still_inside =
                    Some((text.clone(), rows, self.world.now_ms() + 6_000));
                self.set_status(&text, Tone::Secondary);
            }
            ExitDecision::AlreadyEnded => {
                self.route = Route::Manager;
                self.enter_route();
                self.set_status(
                    "Another client already ended the Construct · returning to the host",
                    Tone::Secondary,
                );
                self.exit_after_status = Some(self.world.now_ms() + 2_500);
            }
            ExitDecision::Unknown(e) => {
                self.route = Route::Manager;
                self.enter_route();
                self.set_status(&format!("Could not confirm remaining instances: {} · returning to host without the outro", e.label()), Tone::Warning);
                self.exit_after_status = Some(self.world.now_ms() + 2_500);
            }
        }
    }

    fn enter_route(&mut self) {
        let route = self.route;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        if let Some(s) = self.screens.get_mut(route) {
            s.enter(&mut self.world, &mut cx);
        }
        let reqs = std::mem::take(&mut cx.requests);
        self.apply_requests(reqs, route);
    }

    // ------------------------------------------------------------ render

    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.size = (area.width, area.height);
        let theme = self.theme;
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = self.interaction();
        let cursor;
        {
            let buf = frame.buffer_mut();
            let mut ctx = RenderCtx::new(&theme, interaction, &mut hits, &mut ring);
            self.draw(area, buf, &mut ctx);
            cursor = ctx.cursor;
        }
        self.hits = hits;
        self.ring = ring;
        if self.modals.is_empty() {
            if !self.too_small && !self.focus.current().is_some_and(|c| self.ring.contains(c)) {
                let pf = self.screens.get(self.route).and_then(|s| s.primary_focus());
                self.focus
                    .set(pf.filter(|p| self.ring.contains(*p)).or(self.ring.first()));
            }
        } else {
            self.focus.ensure_valid(&self.ring);
        }
        if let Some(pos) = cursor {
            frame.set_cursor_position(pos);
        }
    }

    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        fill(buf, area, t.base());
        self.too_small = area.width < MIN_WIDTH || area.height < MIN_HEIGHT;
        if self.too_small {
            let lines = [
                (
                    &*format!(" {BRAND_MARK} "),
                    junie_tui::widgets::brand::Lockup::style(&t),
                ),
                ("Terminal too small", t.secondary()),
                (
                    &*format!(
                        "Need {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}",
                        area.width, area.height
                    ),
                    t.muted(),
                ),
                ("q Quit", t.faint()),
            ];
            let y0 = area.y + area.height.saturating_sub(5) / 2;
            for (i, (text, style)) in lines.iter().enumerate() {
                let w = width(text) as u16;
                let x = area.x + area.width.saturating_sub(w) / 2;
                let y = y0 + i as u16 + if i == 3 { 1 } else { 0 };
                if y < area.bottom() {
                    buf.set_string(x, y, text, if i == 0 { *style } else { style.bg(t.canvas) });
                }
            }
            return;
        }
        match self.route {
            Route::Intro => {
                let mut state = self
                    .intro
                    .take()
                    .unwrap_or_else(|| IntroState::new(self.motion, 0));
                rain::render_intro(buf, area, &mut state, &t);
                self.intro = Some(state);
            }
            Route::Outro => {
                let mut state = self
                    .outro
                    .take()
                    .unwrap_or_else(|| OutroState::new(self.motion, None, 0));
                rain::render_outro(buf, area, &mut state, &t);
                self.outro = Some(state);
            }
            Route::Handoff => {
                let h = self.handoff.unwrap_or(0);
                ctx.inert = true;
                match handoff_stage(h) {
                    rain::HandoffStage::CockpitDim(n) => {
                        self.draw_frame(area, buf, ctx, Route::Cockpit);
                        rain::dim_buffer(buf, area, n, &t);
                    }
                    rain::HandoffStage::Canvas => {}
                    rain::HandoffStage::CapsuleDim(n) => {
                        self.draw_frame(area, buf, ctx, Route::Capsule);
                        rain::dim_buffer(buf, area, n, &t);
                    }
                    rain::HandoffStage::Capsule => self.draw_frame(area, buf, ctx, Route::Capsule),
                }
                ctx.inert = false;
            }
            r => {
                self.draw_frame(area, buf, ctx, r);
                // modals
                if let Some(mut entry) = self.modals.pop() {
                    let hints = self.modal_hints(&entry.modal);
                    match &mut entry.modal {
                        Modal::Dialog(d) => d.render(area, buf, ctx),
                        Modal::Picker(p) => p.render(area, buf, ctx, &hints),
                        Modal::Browser(b) => {
                            let stepper = self.screens.prelude.as_ref().map(|p| p.stepper_line());
                            b.render(area, buf, ctx, stepper.as_deref());
                        }
                        Modal::Choice(c) => c.render(area, buf, ctx),
                        Modal::Form(f) => f.render(area, buf, ctx),
                        Modal::Op(o) => o.render(area, buf, ctx),
                        Modal::Info(i) => i.render(area, buf, ctx),
                        Modal::Help(h) => h.render(area, buf, ctx),
                        Modal::Custom(c) => c.render(area, buf, ctx, &self.world),
                    }
                    self.modals.push(entry);
                    // footer hints for the modal
                    let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
                    self.draw_footer(footer, buf, true);
                }
            }
        }
    }

    /// Pickers draw no hint row of their own: the shell's hint bar is the
    /// one hint surface.
    fn modal_hints(&self, _m: &Modal) -> String {
        String::new()
    }

    /// Strip + body + footer for a route (no modals).
    fn draw_frame(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, route: Route) {
        let t = self.theme;
        fill(buf, area, t.base());
        let header = Rect::new(area.x, area.y, area.width, 1);
        let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
        let body = Rect::new(
            area.x + 1,
            area.y + 2,
            area.width.saturating_sub(2),
            area.height.saturating_sub(4),
        );
        // the Capsule owns its chrome: menu bar, tabs, status bar; the
        // cockpit keeps the identity strip but uses the full width
        let body = match route {
            Route::Capsule => Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1)),
            Route::Cockpit => Rect::new(
                area.x,
                area.y + 2,
                area.width,
                area.height.saturating_sub(4),
            ),
            _ => body,
        };
        if route != Route::Capsule {
            self.draw_strip(header, buf, ctx, route);
        }
        if let Some(s) = self.screens.get_mut(route) {
            s.render(body, buf, ctx, &self.world);
        }
        self.draw_footer(footer, buf, false);
    }

    fn construct_state(&self, route: Route) -> (&'static str, Tone) {
        match route {
            Route::Intro => ("entering the Construct", Tone::Secondary),
            Route::Cockpit | Route::Handoff => ("entering the Construct", Tone::Normal),
            Route::Capsule => ("inside the Construct", Tone::Normal),
            Route::Outro => ("leaving the Construct", Tone::Secondary),
            _ => {
                if self.world.arbiter.entered_at_ms.is_some() || self.world.running_count() > 0 {
                    ("inside the Construct", Tone::Secondary)
                } else {
                    ("outside the Construct", Tone::Secondary)
                }
            }
        }
    }

    fn draw_strip(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, route: Route) {
        let t = self.theme;
        let (state, tone) = self.construct_state(route);
        let lockup = junie_tui::widgets::brand::Lockup::new(BRAND_MARK);
        let pill_w = lockup.render(area.x + 1, area.y, buf, &t);
        let area = Rect::new(
            area.x + 1 + pill_w,
            area.y,
            area.width.saturating_sub(1 + pill_w),
            area.height,
        );
        let mut left = vec![Segment::new(state, tone).priority(9)];
        match self.world.arbiter.running() {
            Ok(0) => left.push(Segment::new("no instances", Tone::Muted).priority(8)),
            Ok(n) => left.push(Segment::new(format!("{n} running"), Tone::Secondary).priority(8)),
            Err(e) => left.push(Segment::new(format!("! {}", e.label()), Tone::Error).priority(8)),
        }
        if self.world.daemon_health == crate::sim::world::DaemonHealth::Stale {
            left.push(Segment::new("▲ daemon stale", Tone::Warning).priority(8))
        }
        if let Some(s) = self.screens.get(route) {
            left.push(Segment::new(s.crumb(&self.world), Tone::Secondary).priority(7));
        }
        let mut right = vec![];
        if let Some(s) = self.screens.get(route) {
            right.extend(s.strip_right(&self.world));
        }
        if !self.world.jobs.is_empty() && !matches!(route, Route::Cockpit | Route::Capsule) {
            right.push(
                Segment::new(
                    format!(
                        "{} working",
                        junie_tui::widgets::progress::spinner_frame(ctx.interaction.tick)
                    ),
                    Tone::Secondary,
                )
                .priority(6),
            );
        }
        if matches!(
            route,
            Route::Manager
                | Route::Accounts
                | Route::Usage
                | Route::Settings
                | Route::Editor
                | Route::Prelude
        ) {
            right.push(
                Segment::new(
                    format!("{} · {}×{}", t.level.label(), self.size.0, self.size.1),
                    Tone::Faint,
                )
                .priority(1),
            );
            if route != Route::Usage {
                right.push(
                    Segment::new("u Usage", Tone::Muted)
                        .clickable(STRIP_USAGE)
                        .priority(2),
                );
            }
            if route != Route::Accounts {
                right.push(
                    Segment::new("c Accounts", Tone::Muted)
                        .clickable(STRIP_ACCOUNTS)
                        .priority(3),
                );
            }
            if route != Route::Settings {
                right.push(
                    Segment::new("s Settings", Tone::Muted)
                        .clickable(STRIP_SETTINGS)
                        .priority(3),
                );
            }
        }
        right.push(
            Segment::new("? help", Tone::Muted)
                .clickable(STRIP_HELP)
                .priority(4),
        );
        segments::render(area, buf, ctx, &left, &right, t.canvas);
    }

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, modal: bool) {
        let t = self.theme;
        fill(buf, area, t.base());
        let hints: Vec<Hint> = if modal {
            match self.modals.last().map(|m| &m.modal) {
                Some(Modal::Picker(p)) => {
                    let mut v = vec![];
                    if p.searchable {
                        v.push(hint("Type", "Filter"));
                    }
                    v.push(hint("↑↓", "Move"));
                    v.push(hint("Enter", "Choose"));
                    v.push(if p.searchable && !p.query.is_empty() {
                        hint("Esc", "Clear")
                    } else {
                        hint("Esc", "Cancel")
                    });
                    v
                }
                Some(Modal::Op(o)) => o.hints(),
                Some(Modal::Dialog(d)) => {
                    if d.is_editing() {
                        vec![hint("Enter", "Next"), hint("Esc", "Cancel")]
                    } else if matches!(d.body, DialogBody::Facts { .. }) {
                        vec![
                            hint("← →", "Choose"),
                            hint("Enter", "Confirm"),
                            hint("Esc", "Cancel"),
                        ]
                    } else {
                        vec![
                            hint("← →", "Choose"),
                            hint("Enter", "Confirm"),
                            hint("Esc", "Cancel"),
                            hint("y / n", "Quick answer"),
                        ]
                    }
                }
                Some(Modal::Browser(_)) => vec![
                    hint("Enter", "Open"),
                    hint("Space", "Choose"),
                    hint("g", "Git URL"),
                    hint("Tab", "Next"),
                    hint("Esc", "Cancel"),
                ],
                Some(Modal::Choice(_)) => vec![
                    hint("↑↓", "Choose"),
                    hint("Enter", "Confirm"),
                    hint("Tab", "Buttons"),
                    hint("Esc", "Cancel"),
                ],
                Some(Modal::Form(f)) => {
                    if f.is_editing() {
                        vec![
                            hint("Enter", "Commit"),
                            hint("Tab", "Next field"),
                            hint("Esc", "Revert"),
                        ]
                    } else {
                        vec![
                            hint("Tab", "Next field"),
                            hint("Enter", "Edit / Save"),
                            hint("Esc", "Cancel"),
                        ]
                    }
                }
                Some(Modal::Info(_)) => {
                    vec![hint("↑↓", "Move"), hint("y", "Copy"), hint("Esc", "Close")]
                }
                Some(Modal::Help(_)) => vec![hint("↑↓", "Scroll"), hint("Esc", "Close")],
                Some(Modal::Custom(c)) => c.hints(),
                _ => vec![],
            }
        } else {
            let focus = self.focus.current();
            self.screens
                .get(self.route)
                .map(|s| s.hints(focus, &self.world))
                .unwrap_or_default()
        };
        let editing = self.screens.get(self.route).is_some_and(|s| s.is_editing())
            || self.modals.last().is_some_and(|m| match &m.modal {
                Modal::Dialog(d) => d.is_editing(),
                Modal::Form(f) => f.is_editing(),
                Modal::Browser(b) => b.is_editing(),
                _ => false,
            });
        let badge = if editing {
            Some(("EDIT", BadgeKind::Edit))
        } else {
            None
        };
        let status = self.status.as_ref().map(|(s, tone, _)| (s.as_str(), *tone));
        // still-inside feedback takes the status slot when present
        let still = self
            .screens
            .manager
            .still_inside
            .as_ref()
            .map(|(s, _, _)| s.clone());
        let status = match (&still, status) {
            (Some(s), None) if self.route == Route::Manager => Some((s.as_str(), Tone::Secondary)),
            (_, s) => s,
        };
        let status =
            status.map(|(s, tone)| (truncate(s, area.width.saturating_sub(4) as usize), tone));
        // one hint surface: topmost modal › the screen's own context › fallback
        let modal_layer = modal.then(|| HintLayer::new(hints.clone()));
        let screen_layer = (!modal && !hints.is_empty()).then(|| HintLayer::new(hints.clone()));
        let fallback = HintLayer::new(vec![hint("?", "Help"), hint("Esc", "Back")]);
        let mut layer = HintBar::resolve(&[modal_layer, screen_layer, Some(fallback)]);
        layer.badge = badge;
        layer.status = status;
        HintBar::render(area, buf, &t, &layer);
    }
}

fn top_tag(modals: &[ModalEntry]) -> ModalTag {
    modals
        .last()
        .map(|m| m.tag.clone())
        .unwrap_or(ModalTag::new(""))
}

fn top_owner(modals: &[ModalEntry]) -> Route {
    modals.last().map(|m| m.owner).unwrap_or(Route::Manager)
}

fn form_values(modals: &[ModalEntry]) -> crate::screens::modals::FormValues {
    match modals.last().map(|m| &m.modal) {
        Some(Modal::Form(f)) => f.values(),
        _ => vec![],
    }
}
