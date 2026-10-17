//! Application shell: one runtime session that owns the route (intro,
//! host control plane, cockpit, handoff, Capsule, outro), the modal stack,
//! focus/hover/press state, the identity strip and the footer.

use junie_tui::clock::Clock;
use junie_tui::core::event::{Input, Key, Outcome};
use junie_tui::core::focus::FocusChange;
use junie_tui::core::id::WidgetId;
use junie_tui::core::intent::Phase;
use junie_tui::core::pointer::{
    MouseTarget, PointerEvent, PointerHost, PointerParts, PointerRouter,
};
use junie_tui::core::response::Response;
use junie_tui::shell::{self, PageKey, Shell};
use junie_tui::theme::{BadgeKind, Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::layer::{
    DismissReason, Dismissed, LayerCx, LayerEntry, LayerPointer, ShellLayers, Taken,
};
use junie_tui::ui::leave::{KeyContext, LeaveKey, QuitKeys, letters_allowed};
use junie_tui::ui::text::truncate;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::picker::{Picker, PickerAction};
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::too_small::TooSmall;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
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
use crate::screens::modals::{
    ChoiceDialog, FormAction, FormDialog, HelpOverlay, InfoDialog, InfoResult,
};
use crate::screens::prelude::PreludeScreen;
use crate::screens::settings::SettingsScreen;
use crate::screens::usage::UsageScreen;
use crate::screens::{AppLayer, Cx, Go, LayerTag, ModalResult, Request, Screen};
use crate::sim::launch::LaunchPlan;
use crate::sim::world::{Msg, World};
use junie_tui::shell::Page as _;
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::menu::{MenuBar, MenuBarAction, MenuItem};

/// The canonical product mark; every brand lockup renders exactly this.
pub const BRAND_MARK: &str = "jackin❯";
const STRIP_HELP: WidgetId = WidgetId::of("strip.help");
const HOST_MENU: WidgetId = WidgetId::of("host.menu");
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

/// The app's layer stack: every layer reads the world and reports a
/// [`ModalResult`] to the route that opened it.
type Layers = ShellLayers<ModalResult, World, Route>;
type Entry = LayerEntry<ModalResult, World, Route>;

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

/// Every route that owns a screen, for broadcasting focus changes.
const ROUTES: [Route; 8] = [
    Route::Manager,
    Route::Accounts,
    Route::Usage,
    Route::Settings,
    Route::Editor,
    Route::Prelude,
    Route::Cockpit,
    Route::Capsule,
];

pub struct App {
    pub theme: Theme,
    pub scenario: Scenario,
    pub world: World,
    pub route: Route,
    pub intro: Option<IntroState>,
    pub outro: Option<OutroState>,
    pub handoff: Option<u64>,
    pub screens: Screens,
    layers: Layers,
    /// Focus, ring, hit regions, pointer, the footer status (5 s on the
    /// world clock; an error stays until the next action), size, motion
    /// and quit: the library shell's state.
    pub shell: Shell,
    /// Which keys ask to quit. Default [`QuitKeys::DEFAULT`]: Esc never
    /// quits, `q` never fires over a field or unsaved edits.
    pub quit_keys: QuitKeys,
    /// A quit is waiting on the screen's unsaved-changes question: when
    /// that flow leaves for the manager (Save or Discard), the app quits.
    quit_after_leave: bool,
    /// The application menu bar shown on every host screen.
    pub host_menu: MenuBar,
    host_menu_route: Route,
    intro_guard: u8,
    pub clipboard_gen: u32,
    exit_after_status: Option<i64>,
}

impl App {
    /// The top layer as an `L` (proof accessor).
    #[cfg(test)]
    pub fn top_layer<L: junie_tui::ui::layer::Layer<ModalResult, World>>(&self) -> Option<&L> {
        self.layers.top_as()
    }

    /// How many layers are open (proof accessor).
    #[cfg(test)]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// The top layer's dismissal policy (proof accessor).
    #[cfg(test)]
    pub fn top_dismiss(&self) -> Option<junie_tui::ui::popup::Dismiss> {
        self.layers.top().map(|t| t.layer.spec().dismiss)
    }

    /// Pure fixture entry point: one coherent world per scenario.
    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64, theme: Theme) -> Self {
        let mut world = crate::domain::fixtures::world_for(scenario);
        world.clock.running = motion != Motion::Paused;
        let mut shell = Shell::new();
        shell.motion = motion;
        let mut app = Self {
            theme: theme.centred_hints(),
            scenario,
            world,
            route: Route::Manager,
            intro: None,
            outro: None,
            handoff: None,
            screens: Screens::default(),
            layers: ShellLayers::new(),
            shell,
            quit_keys: QuitKeys::DEFAULT,
            quit_after_leave: false,
            host_menu: Self::build_host_menu(Route::Manager),
            host_menu_route: Route::Manager,
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
                self.outro = Some(OutroState::new(self.shell.motion, elapsed, frame));
                self.route = Route::Outro;
                return;
            }
            _ => {}
        }
        match self.world.arbiter.request_entry() {
            EntryDecision::PlayIntro => {
                if frame >= rain::INTRO_END && self.shell.motion != Motion::Reduced {
                    self.world.arbiter.complete_entry(self.world.now_ms());
                    self.enter_manager();
                } else {
                    self.intro = Some(IntroState::new(self.shell.motion, frame));
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
                        "Could not confirm running instances: {} · list may be incomplete",
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
                    let (_, reqs) = self.shell.deliver(true, |cx| {
                        if let Some(c) = self.screens.cockpit.as_mut() {
                            c.seek(frame, &mut self.world, cx);
                        }
                        Outcome::Ignored
                    });
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
        self.on_route(Route::Manager, Outcome::Ignored, |s, w, cx| {
            s.enter(w, cx);
            Outcome::Ignored
        });
    }

    pub fn set_status(&mut self, s: &str, tone: Tone) {
        self.shell.status.set_at(s, tone, self.world.clock.now());
    }

    fn animating(&self) -> bool {
        matches!(
            self.route,
            Route::Intro | Route::Outro | Route::Handoff | Route::Cockpit | Route::Capsule
        ) || self.shell.pointer.has_flash()
            || self
                .screens
                .get(self.route)
                .is_some_and(|s| s.animating(&self.world))
            || self.layer_animates()
            || !self.world.jobs.is_empty()
            || self.world.daemons.values().any(|d| !d.panes.is_empty())
    }

    pub fn tick_interval(&self) -> std::time::Duration {
        if self.shell.motion == Motion::Paused {
            return std::time::Duration::from_millis(500);
        }
        std::time::Duration::from_millis(self.route.tick_ms(self.animating()))
    }

    /// Milliseconds on the pointer clock (the simulated world clock).
    fn now_ms(&self) -> u64 {
        self.world.now_ms().max(0) as u64
    }

    // ------------------------------------------------------------- input

    /// Hand every focus change the input caused to every open layer and
    /// the screens, so fields end their edits on the event path.
    fn blur_all(&mut self, ch: FocusChange) -> Outcome {
        let mut out = self.layers.on_blur(ch);
        {
            // what an ended edit reported (a form field changed) reaches
            // the owner, as Enter's does
            out = self.settle_layers(out);
            // a route behind the shown one may only report a status (the
            // background-request policy, `Shell::deliver`)
            for route in ROUTES {
                out =
                    out.or(self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_blur(ch, w, cx)));
            }
        }
        out
    }

    fn handle_input(&mut self, input: Input) -> Outcome {
        match input {
            Input::Resize(..) => Outcome::Changed,
            Input::Tick => self.on_tick(),
            Input::Paste(text) => self.on_paste(&text),
            Input::Key(key) => {
                self.shell.pointer.suppress_hover();
                // shortcuts first, then type-ahead (one rule, in the library)
                junie_tui::core::list::route_key(&key, || self.on_key(key))
            }
            Input::Mouse(m) => PointerRouter::route(self, m),
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
        let now = self.now_ms();
        if self.shell.pointer.expire_flash(now) {
            out = Outcome::Changed;
        }
        // a held drag past a selecting body's edge autoscrolls, a press
        // held on a scrollbar track keeps paging (the simulated clock
        // keeps its own tick rate)
        out = out.or(PointerRouter::tick(self));
        if self.shell.status.expire(self.world.clock.now()) {
            out = Outcome::Changed;
        }
        if let Some(at) = self.exit_after_status
            && self.world.now_ms() >= at
        {
            self.shell.quit = true;
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
                        self.shell.quit = true;
                    }
                }
                return Outcome::Changed;
            }
            Route::Handoff => {
                // a paused clock holds the handoff (an explicit clock step
                // runs it, `advance_clock`)
                if self.world.clock.running {
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
        // the top layer's tick
        if let Some(top) = self.layers.top_mut() {
            let o = top.layer.on_tick(&self.world);
            out = self.settle_layers(out.or(o));
        }
        // screen tick
        let route = self.route;
        out = out.or(self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_tick(w, cx)));
        // messages
        for m in msgs {
            out = out.or(self.dispatch_msg(m));
        }
        out
    }

    fn dispatch_msg(&mut self, m: Msg) -> Outcome {
        let route = self.route;
        let (o, reqs) = self
            .shell
            .deliver(true, |cx| match self.screens.get_mut(route) {
                Some(s) => s.on_msg(&m, &mut self.world, cx),
                None => Outcome::Ignored,
            });
        // the manager and accounts follow the world from behind; the
        // background-request policy keeps only their statuses
        let (o, behind) = self.shell.deliver(false, |cx| {
            let mut o = o;
            if !o.consumed() && route != Route::Manager {
                o = self.screens.manager.on_msg(&m, &mut self.world, cx);
            }
            if !o.consumed() && route != Route::Accounts {
                o = self.screens.accounts.on_msg(&m, &mut self.world, cx);
            }
            o
        });
        let o = o.or(self.apply_requests(reqs, route));
        o.or(self.apply_requests(behind, route))
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
        self.on_route(Route::Capsule, Outcome::Ignored, |s, w, cx| {
            s.enter(w, cx);
            Outcome::Ignored
        });
    }

    fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(top) = self.layers.top_mut() {
            let o = top.layer.on_paste(text).or(Outcome::Consumed);
            return self.settle_layers(o);
        }
        let route = self.route;
        self.on_route(route, Outcome::Consumed, |s, w, cx| s.on_paste(text, w, cx))
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.shell.too_small {
            if key.is_char('q') || key.ctrl_char('c') {
                self.shell.quit = true;
            }
            return Outcome::Consumed;
        }
        match self.route {
            Route::Intro => {
                if self.intro_guard > 0 && self.shell.motion != Motion::Paused {
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
                    self.shell.quit = true;
                }
                return Outcome::Consumed;
            }
            Route::Outro => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    if let Some(s) = self.outro.as_mut() {
                        s.skip();
                        if s.is_done() {
                            self.shell.quit = true;
                        }
                    }
                    return Outcome::Changed;
                }
                if key.ctrl_char('c') {
                    self.shell.quit = true;
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
        if !self.layers.is_empty() {
            return self.layer_key(key);
        }
        // back on the page: a quit that waited on the leave question was
        // cancelled
        self.quit_after_leave = false;
        if key.ctrl_char('c') && self.route != Route::Cockpit && self.route != Route::Capsule {
            let kc = KeyContext::default();
            if self.quit_keys.classify(&key, kc) == Some(LeaveKey::Quit) {
                return self.request_quit();
            }
        }
        let route = self.route;
        // an open page menu owns the keyboard like a text field does: no
        // global chord reaches past it
        let editing = self
            .screens
            .get(route)
            .is_some_and(|s| s.is_editing() || s.layer_open());
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
        if host && self.host_menu.is_open() {
            let (o, ev) = self.host_menu.on_key(&key);
            return match ev {
                Some(MenuBarAction::Chosen(mi, ii)) => {
                    let label = self.host_menu.menus[mi][ii].label.clone();
                    self.run_host_menu(&label)
                }
                Some(MenuBarAction::Brand) => {
                    self.open_about();
                    Outcome::Changed
                }
                _ => o.or(Outcome::Changed),
            };
        }
        // plain F10 is the menu bar; Shift+F10 is the page's context key
        if host && !editing && key.code == KeyCode::F(10) && !key.shift() {
            self.sync_host_menu();
            self.host_menu.open_menu(0);
            return Outcome::Changed;
        }
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
                KeyCode::Char(_)
                    if key.ctrl()
                        && self.quit_keys.classify(&key, KeyContext::default())
                            == Some(LeaveKey::Quit) =>
                {
                    return self.request_quit();
                }
                _ => {}
            }
        }
        let out = self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_key(&key, w, cx));
        if out.consumed() {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
                && key.plain()
                && !editing
                && let Some(f) = self.shell.focus.current()
            {
                let now = self.now_ms();
                self.shell.pointer.flash(f, now);
            }
            return out;
        }
        match key.code {
            KeyCode::Tab => {
                self.shell.focus.next(&self.shell.ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                self.shell.focus.prev(&self.shell.ring);
                Outcome::Changed
            }
            // a quit chord an editing field passed on (fields bind no
            // Ctrl+Q): through the same guard
            KeyCode::Char(_)
                if host
                    && key.ctrl()
                    && self.quit_keys.classify(
                        &key,
                        KeyContext {
                            text_focus: editing,
                            ..KeyContext::default()
                        },
                    ) == Some(LeaveKey::Quit) =>
            {
                self.request_quit()
            }
            KeyCode::Char('q') if key.plain() && !editing && host => {
                // a letter never leaves unsaved edits (keyboard audit #4)
                let dirty = self
                    .screens
                    .get_mut(route)
                    .is_some_and(|s| !s.stakes(&self.world).is_clean());
                let kc = KeyContext {
                    text_focus: editing,
                    dirty,
                    ..KeyContext::default()
                };
                if !letters_allowed(kc) {
                    self.set_status("Unsaved changes · save, or Esc to leave", Tone::Secondary);
                } else if route == Route::Manager {
                    return self.request_quit();
                } else {
                    self.go(Go::Manager);
                }
                Outcome::Changed
            }
            KeyCode::Esc => {
                let o = self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_esc_top(w, cx));
                if o != Outcome::Ignored || !host {
                    return o;
                }
                // the top of the ladder: Esc never quits unless configured
                if self.quit_keys.classify(&key, KeyContext::default()) == Some(LeaveKey::Quit) {
                    return self.request_quit();
                }
                self.set_status("Esc never quits · Ctrl+Q quits", Tone::Secondary);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    /// Every quit (Ctrl+Q, Ctrl+C, `q`, the menu) goes through here: a
    /// clean screen quits at once (running instances keep running in the
    /// Construct, so nothing is lost); unsaved edits ask the screen's own
    /// Save / Discard / Cancel first, and the app quits when that flow
    /// leaves the screen.
    fn request_quit(&mut self) -> Outcome {
        let route = self.route;
        let dirty = self
            .screens
            .get_mut(route)
            .is_some_and(|s| !s.stakes(&self.world).is_clean());
        if !dirty {
            self.world.arbiter.release_entry();
            self.shell.quit = true;
            return Outcome::Consumed;
        }
        self.quit_after_leave = true;
        self.on_route(route, Outcome::Ignored, |s, w, cx| s.request_leave(w, cx))
    }

    fn is_host(route: Route) -> bool {
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

    /// The menu bar of the host screens: the same bar as the Capsule's, with
    /// the screen's own actions under File and the destinations under Go.
    fn build_host_menu(route: Route) -> MenuBar {
        let file: Vec<MenuItem> = match route {
            Route::Manager => vec![
                MenuItem::new("New workspace…").shortcut("n"),
                MenuItem::new("Edit workspace…").shortcut("e"),
                MenuItem::new("Launch…").shortcut("Enter"),
                MenuItem::new("Prewarm").shortcut("w"),
                MenuItem::new("Delete workspace…")
                    .shortcut("d")
                    .danger()
                    .separator(),
                MenuItem::new("Refresh").shortcut("F5").separator(),
                MenuItem::new("Quit").shortcut("Ctrl+Q"),
            ],
            Route::Editor | Route::Settings => vec![
                MenuItem::new("Save…").shortcut("Ctrl+S").separator(),
                MenuItem::new("Cancel").shortcut("Esc"),
            ],
            Route::Accounts => vec![
                MenuItem::new("Add account…").shortcut("a"),
                MenuItem::new("Edit account…").shortcut("e"),
                MenuItem::new("Validate").shortcut("v").separator(),
                MenuItem::new("Refresh selection").shortcut("r"),
                MenuItem::new("Refresh all").shortcut("F5"),
                MenuItem::new("Filter…").shortcut("/"),
            ],
            Route::Usage => vec![
                MenuItem::new("Refresh").shortcut("r"),
                MenuItem::new("Manage in Accounts").shortcut("m"),
            ],
            _ => vec![MenuItem::new("Cancel").shortcut("Esc")],
        };
        let go = vec![
            MenuItem::new("Workspace manager").shortcut("Esc"),
            MenuItem::new("Account & Usage Center").shortcut("c"),
            MenuItem::new("Usage").shortcut("u"),
            MenuItem::new("Global settings").shortcut("s"),
        ];
        let help = vec![
            MenuItem::new("Key reference").shortcut("?"),
            MenuItem::new("About jackin-preview"),
        ];
        MenuBar::new(HOST_MENU, vec![("File", file), ("Go", go), ("Help", help)])
            .brand(Lockup::new(BRAND_MARK))
    }

    fn sync_host_menu(&mut self) {
        if self.host_menu_route != self.route && Self::is_host(self.route) {
            self.host_menu = Self::build_host_menu(self.route);
            self.host_menu_route = self.route;
        }
    }

    /// A menu item on a host screen either navigates or presses the key the
    /// item names, so menus and keys can never disagree.
    fn run_host_menu(&mut self, label: &str) -> Outcome {
        let editing_screen = matches!(self.route, Route::Editor | Route::Settings | Route::Prelude);
        let key = |code: KeyCode, mods: KeyModifiers| Input::Key(Key { code, mods });
        match label {
            "Workspace manager" => {
                if editing_screen {
                    // the editor asks about unsaved work on its own way out
                    return shell::handle(self, key(KeyCode::Esc, KeyModifiers::NONE));
                }
                self.go(Go::Manager);
            }
            "Account & Usage Center" | "Usage" | "Global settings" => {
                if editing_screen {
                    self.set_status("Save or cancel this screen first", Tone::Secondary);
                    return Outcome::Changed;
                }
                match label {
                    "Account & Usage Center" => self.go(Go::Accounts { select: None }),
                    "Usage" => self.go(Go::Usage { select: None }),
                    _ => self.go(Go::Settings),
                }
            }
            "Key reference" => self.open_help(),
            "About jackin-preview" => self.open_about(),
            "Quit" => {
                return self.request_quit();
            }
            "New workspace…" => {
                return shell::handle(self, key(KeyCode::Char('n'), KeyModifiers::NONE));
            }
            "Edit workspace…" | "Edit account…" => {
                return shell::handle(self, key(KeyCode::Char('e'), KeyModifiers::NONE));
            }
            "Launch…" => return shell::handle(self, key(KeyCode::Enter, KeyModifiers::NONE)),
            "Prewarm" => return shell::handle(self, key(KeyCode::Char('w'), KeyModifiers::NONE)),
            "Delete workspace…" => {
                return shell::handle(self, key(KeyCode::Char('d'), KeyModifiers::NONE));
            }
            "Refresh" | "Refresh selection" => {
                return shell::handle(
                    self,
                    key(
                        if self.route == Route::Manager {
                            KeyCode::F(5)
                        } else {
                            KeyCode::Char('r')
                        },
                        KeyModifiers::NONE,
                    ),
                );
            }
            "Refresh all" => return shell::handle(self, key(KeyCode::F(5), KeyModifiers::NONE)),
            "Save…" => {
                return shell::handle(self, key(KeyCode::Char('s'), KeyModifiers::CONTROL));
            }
            "Cancel" => return shell::handle(self, key(KeyCode::Esc, KeyModifiers::NONE)),
            "Add account…" => {
                return shell::handle(self, key(KeyCode::Char('a'), KeyModifiers::NONE));
            }
            "Validate" => return shell::handle(self, key(KeyCode::Char('v'), KeyModifiers::NONE)),
            "Filter…" => return shell::handle(self, key(KeyCode::Char('/'), KeyModifiers::NONE)),
            "Manage in Accounts" => {
                return shell::handle(self, key(KeyCode::Char('m'), KeyModifiers::NONE));
            }
            _ => self.set_status(
                &format!("{label}: not available in the preview"),
                Tone::Secondary,
            ),
        }
        Outcome::Changed
    }

    fn open_about(&mut self) {
        let props = vec![
            junie_tui::widgets::props::Prop::new(
                "Product",
                "jackin-preview · deterministic redesign preview",
            ),
            junie_tui::widgets::props::Prop::new("Scenario", self.world.scenario.name()),
            junie_tui::widgets::props::Prop::new(
                "Construct",
                "simulated · nothing here touches a real container",
            ),
            junie_tui::widgets::props::Prop::new(
                "Design system",
                "Junie-inspired Ratatui components (junie_tui)",
            ),
        ];
        let d = crate::screens::modals::InfoDialog::new(WidgetId::of("host.about"), "About", props)
            .width(64);
        self.open_layer(Box::new(d), LayerTag::new("about"), self.route);
    }

    /// Row zero of every host screen: the menu bar, then the Construct state
    /// and the breadcrumb right-aligned in what is left of the row.
    fn draw_host_menu(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, route: Route) {
        let t = self.theme;
        self.sync_host_menu();
        self.host_menu.render(area, buf, ctx, t.canvas);
        let used = self
            .host_menu
            .areas
            .iter()
            .map(|r| r.right())
            .max()
            .unwrap_or(area.x)
            .max(self.host_menu.brand_area.right());
        let rest = Rect::new(used + 2, area.y, area.right().saturating_sub(used + 2), 1);
        let (state, tone) = self.construct_state(route);
        let mut right = vec![];
        if let Some(s) = self.screens.get(route) {
            right.push(Segment::new(s.crumb(&self.world), Tone::Secondary).priority(7));
            right.extend(s.strip_right(&self.world));
        }
        right.push(Segment::new(state, tone).priority(6));
        match self.world.arbiter.running() {
            Ok(0) => right.push(Segment::new("no instances", Tone::Muted).priority(5)),
            Ok(n) => right.push(
                Segment::new(format!("{n} running"), Tone::Secondary)
                    .lead(self.theme.glyphs.dot_on, Tone::Active)
                    .priority(5),
            ),
            Err(e) => right.push(Segment::new(format!("! {}", e.label()), Tone::Error).priority(8)),
        }
        if self.world.daemon_health == crate::sim::world::DaemonHealth::Stale {
            right.push(Segment::new("▲ daemon stale", Tone::Warning).priority(8));
        }
        if !self.world.jobs.is_empty() {
            right.push(
                Segment::new(
                    format!(
                        "{} working",
                        junie_tui::widgets::progress::spinner_frame(ctx.interaction.tick)
                    ),
                    Tone::Secondary,
                )
                .priority(4),
            );
        }
        segments::render(rest, buf, ctx, &[], &right, t.canvas);
    }

    fn open_help(&mut self) {
        if !self.layers.is_empty() {
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
                        ("Ctrl+B", "pane commands…"),
                        ("Ctrl+\\", "command palette"),
                        ("Ctrl+Q", "quit"),
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
                        ("&", "close tab"),
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
                        ("v", "unified / side by side"),
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
                        ("x", "cancel launch…"),
                        ("t", "toggle debug"),
                        ("Ctrl+Q", "quit · asks only about unsaved edits"),
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
                        ("t", "enable / disable"),
                        ("v", "validate"),
                        ("d", "remove…"),
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
                        ("Ctrl+Q", "quit · asks only about unsaved edits"),
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
                        ("Esc", "cancel the edit"),
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
                        ("d", "remove · auth back to sync"),
                        ("u", "undo row"),
                    ],
                ),
                (
                    "Save",
                    vec![
                        ("Ctrl+S", "preview then save"),
                        ("Esc", "back · asks when dirty"),
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
                        ("d", "remove · auth back to sync"),
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
                        ("Esc", "back · asks when dirty"),
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
                        ("Ctrl+Q", "quit · asks only about unsaved edits"),
                        ("Ctrl+C", "quit · same as Ctrl+Q"),
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
        self.open_layer(Box::new(h), LayerTag::new("help"), self.route);
    }

    // ------------------------------------------------------------ layers

    /// Open `layer` on top for `owner`: focus moves into it (the stack
    /// remembers the control it covers, K9) and the page under it shows
    /// no hover or press.
    fn open_layer(&mut self, layer: Box<AppLayer>, tag: LayerTag, owner: Route) {
        self.layers.open(layer, tag, owner, &mut self.shell.focus);
        self.shell.pointer.hover = None;
        self.shell.pointer.pressed = None;
        if let Some(c) = self.screens.capsule.as_mut() {
            c.dialog_open = true;
        }
    }

    /// A layer closed: the Capsule takes input again once none is open.
    fn layer_gone(&mut self) {
        if self.layers.is_empty()
            && let Some(c) = self.screens.capsule.as_mut()
        {
            c.dialog_open = false;
        }
    }

    /// Hand what the top layer reported to its owner: an event leaves the
    /// layer open; a result closes it (the stack gives focus back to the
    /// control it covered, K9) and reaches the screen that opened it.
    fn settle_layers(&mut self, mut out: Outcome) -> Outcome {
        while let Some(taken) = self.layers.take(&mut self.shell.focus) {
            out = out.or(match taken {
                Taken::Event(r) => self.layer_event(r),
                Taken::Closed(entry, r) => {
                    self.layer_gone();
                    self.layer_closed(entry, r)
                }
            });
        }
        out
    }

    /// An event from the top layer, which stays open: the shell answers
    /// the ones that need a layer (a picker's rows, a form's dependent
    /// fields, a copied value); the rest reach the owner.
    fn layer_event(&mut self, r: ModalResult) -> Outcome {
        let Some(top) = self.layers.top() else {
            return Outcome::Ignored;
        };
        let (tag, owner) = (top.tag.clone(), top.owner);
        let r = match r {
            ModalResult::PickerEvent(PickerAction::QueryChanged) => {
                self.refresh_picker();
                return Outcome::Changed;
            }
            ModalResult::PickerEvent(_) => return Outcome::Consumed,
            ModalResult::FormEvent(FormAction::Changed(_)) => {
                self.form_changed();
                return Outcome::Changed;
            }
            ModalResult::FormEvent(FormAction::Choose(name)) => {
                ModalResult::FormAction(format!("choose:{name}"), self.top_form_values())
            }
            ModalResult::FormEvent(FormAction::Action(name)) => {
                ModalResult::FormAction(name, self.top_form_values())
            }
            ModalResult::Info(InfoResult::Copy(v)) => {
                self.copy(v);
                return Outcome::Changed;
            }
            r => r,
        };
        let scope = r == ModalResult::Scope;
        let o = self.tell_owner(owner, &tag, r);
        if scope {
            // the owner switched scope: it supplies the rows again
            self.refresh_picker();
        }
        o.or(Outcome::Changed)
    }

    /// A layer closed with `r`: the screen that opened it hears it.
    fn layer_closed(&mut self, entry: Entry, r: ModalResult) -> Outcome {
        let cancel_index = entry.get::<Dialog>().and_then(|d| d.cancel_index);
        let r = match r {
            ModalResult::Help => return Outcome::Changed,
            // Save hands over the form's values
            ModalResult::FormEvent(FormAction::Save) => ModalResult::Form(Some(
                entry
                    .get::<FormDialog>()
                    .map(FormDialog::values)
                    .unwrap_or_default(),
            )),
            // a dismissed dialog resolves to its cancel action, as its Esc
            // does
            ModalResult::Dialog { action: None, text } => ModalResult::Dialog {
                action: cancel_index,
                text,
            },
            r => r,
        };
        if entry.tag.kind == "quit" && entry.owner == self.route {
            if let ModalResult::Dialog {
                action: Some(a), ..
            } = r
                && Some(a) != cancel_index
            {
                self.world.arbiter.release_entry();
                self.shell.quit = true;
            }
            return Outcome::Changed;
        }
        self.tell_owner(entry.owner, &entry.tag, r)
    }

    /// Tell `owner`, the screen that opened a layer, what it reported.
    fn tell_owner(&mut self, owner: Route, tag: &LayerTag, r: ModalResult) -> Outcome {
        self.on_route(owner, Outcome::Changed, |s, w, cx| {
            s.on_layer_result(tag, r, w, cx)
        })
        .or(Outcome::Changed)
    }

    /// The values of the form on top (a chooser or action button
    /// reports them with its name).
    fn top_form_values(&self) -> crate::screens::modals::FormValues {
        self.layers
            .top_as::<FormDialog>()
            .map(FormDialog::values)
            .unwrap_or_default()
    }

    /// Put `v` on the preview clipboard.
    fn copy(&mut self, v: String) {
        self.world.clipboard = Some(v);
        self.clipboard_gen += 1;
        self.set_status("Copied to the preview clipboard", Tone::Secondary);
    }

    /// The picker on top asks its owner for rows matching its query.
    fn refresh_picker(&mut self) {
        let Some(top) = self.layers.top() else {
            return;
        };
        let Some(p) = top.get::<Picker>() else {
            return;
        };
        let (tag, owner, query) = (top.tag.clone(), top.owner, p.query.clone());
        let items = self
            .screens
            .get_mut(owner)
            .and_then(|s| s.picker_items(&tag, &query, &self.world));
        if let Some(items) = items
            && let Some(p) = self.layers.top_as_mut::<Picker>()
        {
            let n = items.len();
            p.set_items(items);
            if tag.kind == "palette" {
                p.scope = Some(format!("{n} of 20"));
            }
        }
    }

    /// A field of the form on top changed: its owner reveals or hides the
    /// fields that depend on it.
    fn form_changed(&mut self) {
        let Some(top) = self.layers.top_mut() else {
            return;
        };
        let (tag, owner) = (top.tag.clone(), top.owner);
        let Some(form) = top.get_mut::<FormDialog>() else {
            return;
        };
        if let Some(s) = self.screens.get_mut(owner) {
            s.form_changed(&tag, form, &self.world);
        }
    }

    /// A key while a layer is open: the top layer has it (its own Esc
    /// honours its dismissal policy); an Esc it passes on dismisses it.
    fn layer_key(&mut self, key: Key) -> Outcome {
        let Some(top) = self.layers.top_mut() else {
            return Outcome::Ignored;
        };
        let mut cx = LayerCx {
            focus: &mut self.shell.focus,
            ring: &self.shell.ring,
            app: &self.world,
        };
        let o = top.layer.on_key(&key, &mut cx);
        if o == Outcome::Ignored && key.is(KeyCode::Esc) {
            return self.dismiss_layer(DismissReason::Esc);
        }
        self.settle_layers(o.or(Outcome::Consumed))
    }

    /// Dismiss the top layer for `reason` under its own policy: it closes
    /// with its cancel result, or stays (a refused outside click flashes
    /// its frame).
    fn dismiss_layer(&mut self, reason: DismissReason) -> Outcome {
        let d = self.layers.dismiss(reason, &mut self.shell.focus);
        self.dismissed(reason, d)
    }

    /// What a dismissal of the top layer did (see [`Self::dismiss_layer`]).
    fn dismissed(
        &mut self,
        reason: DismissReason,
        d: Dismissed<ModalResult, World, Route>,
    ) -> Outcome {
        match d {
            Dismissed::None => Outcome::Ignored,
            Dismissed::Refused => {
                let dismiss = self
                    .layers
                    .top()
                    .map(|t| t.layer.spec().dismiss)
                    .unwrap_or_default();
                let now = self.now_ms();
                if reason == DismissReason::OutsideClick
                    && junie_tui::ui::modal::refuse_outside_click(
                        &mut self.shell.pointer,
                        dismiss,
                        now,
                    )
                {
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            Dismissed::Closed(entry, r) => {
                self.layer_gone();
                let o = match r {
                    Some(r) => self.layer_closed(entry, r),
                    None => Outcome::Changed,
                };
                self.settle_layers(o)
            }
        }
    }

    /// Whether the top layer animates (a loading chain, a refresh): the
    /// shell ticks faster while it does.
    fn layer_animates(&self) -> bool {
        self.layers.top().is_some_and(|t| t.layer.animating())
    }

    /// Whether a field in the top layer is being edited (the EDIT badge).
    fn layer_editing(&self) -> bool {
        self.layers.top().is_some_and(|t| t.layer.is_editing())
    }

    /// The footer's hints for the top layer: its own, or this app's line
    /// for the library modals that report none.
    fn layer_hints(&self) -> Vec<Hint> {
        let Some(top) = self.layers.top() else {
            return vec![];
        };
        let own = top.layer.hints(self.shell.focus.current());
        if !own.is_empty() {
            return own;
        }
        if let Some(p) = top.get::<Picker>() {
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
        } else if top.get::<ChoiceDialog>().is_some() {
            vec![
                hint("↑↓", "Choose"),
                hint("Enter", "Confirm"),
                hint("Tab", "Buttons"),
                hint("Esc", "Cancel"),
            ]
        } else if let Some(f) = top.get::<FormDialog>() {
            if f.is_editing() {
                junie_tui::widgets::keyhint::EDIT_FIELD.to_vec()
            } else {
                vec![
                    hint("Tab", "Next field"),
                    hint("Enter", "Edit / Save"),
                    hint("Esc", "Cancel"),
                ]
            }
        } else if let Some(i) = top.get::<InfoDialog>() {
            let mut v = vec![hint("↑↓", "Move")];
            if let Some((k, a)) = i.props.reveal.hint() {
                v.push(hint(k, a));
            }
            v.extend([hint("y", "Copy"), hint("Esc", "Close")]);
            v
        } else if top.get::<HelpOverlay>().is_some() {
            vec![hint("↑↓", "Scroll"), hint("Esc", "Close")]
        } else {
            vec![]
        }
    }

    // ------------------------------------------------------------- mouse

    /// One pointer event the router resolved: the footer's hints first
    /// (they are drawn above every layer), then the top layer while one
    /// is open, then the chrome (host menu bar, identity strip) and the
    /// route's screen.
    fn deliver_pointer(&mut self, ev: &PointerEvent) -> Outcome {
        if matches!(self.route, Route::Intro | Route::Outro | Route::Handoff) {
            // a completed click skips the animation as Enter does
            return if ev.is(Phase::Click) {
                self.skip_animation()
            } else {
                Outcome::Ignored
            };
        }
        // a footer hint naming one key runs it (mouse #1)
        if ev.on(HintBar::ID) {
            return match self.shell.footer_keys.clicked(ev) {
                Some(key) => self.on_key(key).or(Outcome::Changed),
                None => Outcome::Consumed,
            };
        }
        if !self.layers.is_empty() {
            return self.layer_pointer(ev);
        }
        if let Some(o) = self.chrome_pointer(ev) {
            return o;
        }
        self.screen_pointer(ev)
    }

    /// A completed click on the intro or the outro skips it as Enter does
    /// ([`junie_tui::core::pointer::PointerConfig::click_skips_animation`]).
    fn skip_animation(&mut self) -> Outcome {
        if matches!(self.route, Route::Intro | Route::Outro)
            && self.shell.pointer.config.click_skips_animation
        {
            return self.on_key(Key {
                code: KeyCode::Enter,
                mods: KeyModifiers::NONE,
            });
        }
        Outcome::Ignored
    }

    /// An event while a layer is open: the frame's `×` means what Esc
    /// means and a click on its backdrop is an outside click, both under
    /// the layer's own policy (`ShellLayers::pointer`); anything else is
    /// the layer's.
    fn layer_pointer(&mut self, ev: &PointerEvent) -> Outcome {
        if ev.is(Phase::Click) && ev.id == junie_tui::ui::modal::CLOSE {
            return self.dismiss_layer(DismissReason::CloseButton);
        }
        match self
            .layers
            .pointer(ev, &mut self.shell.focus, &self.shell.ring, &self.world)
        {
            LayerPointer::Handled(o) => self.settle_layers(o),
            LayerPointer::Dismissed(d) => self.dismissed(DismissReason::OutsideClick, d),
        }
    }

    /// An event on the chrome around the screen: the host menu bar and
    /// the identity strip's segments; `None` when it is not the chrome's.
    fn chrome_pointer(&mut self, ev: &PointerEvent) -> Option<Outcome> {
        if Self::is_host(self.route) {
            // a title opens on the press (MenuBar::open_on); a click
            // chooses a row, the brand opens About
            if let Some(r) = self.host_menu.route(ev) {
                let (o, act) = r.into_parts();
                return Some(match act {
                    Some(MenuBarAction::Chosen(mi, ii)) => {
                        let label = self.host_menu.menus[mi][ii].label.clone();
                        self.run_host_menu(&label)
                    }
                    Some(MenuBarAction::Brand) => {
                        self.open_about();
                        Outcome::Changed
                    }
                    _ => o,
                });
            }
            // a click anywhere else closes the open menu
            if self.host_menu.is_open() && ev.is(Phase::Click) {
                self.host_menu.close();
                return Some(Outcome::Changed);
            }
        }
        if !ev.is(Phase::Click) {
            return None;
        }
        let go = match ev.id {
            STRIP_HELP => {
                self.open_help();
                return Some(Outcome::Changed);
            }
            STRIP_USAGE => Go::Usage { select: None },
            STRIP_ACCOUNTS => Go::Accounts { select: None },
            STRIP_SETTINGS => Go::Settings,
            _ => return None,
        };
        self.go(go);
        Some(Outcome::Changed)
    }

    /// An event on the route's screen, with the requests it made.
    fn screen_pointer(&mut self, ev: &PointerEvent) -> Outcome {
        let route = self.route;
        self.on_route(route, Outcome::Ignored, |s, w, cx| s.pointer(ev, w, cx))
    }

    // ----------------------------------------------------------- requests

    /// Hand an event to `route`'s screen (`absent` when it is not mounted)
    /// through [`junie_tui::shell::Shell::deliver`] (a route behind the
    /// shown one keeps only its statuses), then apply what it asked for.
    fn on_route(
        &mut self,
        route: Route,
        absent: Outcome,
        f: impl FnOnce(&mut dyn Screen, &mut World, &mut Cx) -> Outcome,
    ) -> Outcome {
        let shown = route == self.route;
        let (o, reqs) = self
            .shell
            .deliver(shown, |cx| match self.screens.get_mut(route) {
                Some(s) => f(s, &mut self.world, cx),
                None => absent,
            });
        o.or(self.apply_requests(reqs, route))
    }

    fn apply_requests(&mut self, requests: Vec<Request>, owner: Route) -> Outcome {
        let mut out = Outcome::Ignored;
        for r in requests {
            out = Outcome::Changed;
            match r {
                Request::Status(s, tone) => self.set_status(&s, tone),
                Request::StatusAs(s, kind) => self.set_status(&s, kind.tone()),
                Request::Open(layer, tag) => self.open_layer(layer, tag, owner),
                // the owner closes its layer without a result
                Request::Close => {
                    self.layers.pop(&mut self.shell.focus);
                    self.layer_gone();
                }
                Request::Go(g) => self.go(g),
                Request::Help => self.open_help(),
                Request::Back => self.go(Go::Manager),
                Request::Copy(s) => self.copy(s),
                Request::Top(f) => {
                    if let Some(top) = self.layers.top_mut() {
                        f(&mut *top.layer);
                    }
                }
                Request::RefusedOutsideClick(_) => {}
            }
        }
        out
    }

    pub fn go(&mut self, g: Go) {
        // the unsaved-changes question of a quit resolved to leaving
        if self.quit_after_leave && matches!(g, Go::Manager) {
            self.quit_after_leave = false;
            self.world.arbiter.release_entry();
            self.shell.quit = true;
            return;
        }
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
                    self.shell.motion,
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
                self.screens.capsule = Some(CapsuleScreen::new(&instance, &mut self.world, pane));
                if self.route == Route::Cockpit {
                    self.handoff = Some(0);
                    self.route = Route::Handoff;
                    if self.shell.motion == Motion::Reduced {
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
                        junie_tui::ui::text::plural(n, "instance", "instances")
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
                            junie_tui::ui::text::plural(
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
                self.shell.quit = true;
            }
        }
    }

    /// Foreground instance left: decide between the outro, still-inside
    /// feedback, an already-ended Construct, or a fail-closed return.
    fn leave_construct(&mut self, instance: &str) {
        let decision = self.world.arbiter.request_exit(self.world.now_ms());
        match decision {
            ExitDecision::Outro { elapsed_secs } => {
                self.outro = Some(OutroState::new(self.shell.motion, elapsed_secs, 0));
                self.route = Route::Outro;
                self.layers.clear(&mut self.shell.focus);
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
                    junie_tui::ui::text::plural(remaining, "instance remains", "instances remain")
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
        self.on_route(route, Outcome::Ignored, |s, w, cx| {
            s.enter(w, cx);
            Outcome::Ignored
        });
    }

    // ------------------------------------------------------------ render

    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        match self.route {
            Route::Intro => {
                let mut state = self
                    .intro
                    .take()
                    .unwrap_or_else(|| IntroState::new(self.shell.motion, 0));
                rain::render_intro(buf, area, &mut state, &t);
                self.intro = Some(state);
            }
            Route::Outro => {
                let mut state = self
                    .outro
                    .take()
                    .unwrap_or_else(|| OutroState::new(self.shell.motion, None, 0));
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
                // the layers, bottom to top
                for entry in self.layers.iter_mut() {
                    entry.layer.render(area, buf, ctx, &self.world);
                }
                if !self.layers.is_empty() {
                    // footer hints for the modal
                    let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
                    self.draw_footer(footer, buf, ctx, true);
                }
            }
        }
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
        if Self::is_host(route) {
            self.draw_host_menu(header, buf, ctx, route);
        } else if route != Route::Capsule {
            self.draw_strip(header, buf, ctx, route);
        }
        if let Some(s) = self.screens.get_mut(route) {
            s.render(body, buf, ctx, &self.world);
        }
        self.draw_footer(footer, buf, ctx, false);
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
            Ok(n) => left.push(
                Segment::new(format!("{n} running"), Tone::Secondary)
                    .lead(t.glyphs.dot_on, Tone::Active)
                    .priority(8),
            ),
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
                    format!(
                        "{} · {}×{}",
                        t.level.label(),
                        self.shell.size.0,
                        self.shell.size.1
                    ),
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

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, ctx: &RenderCtx, modal: bool) {
        let t = self.theme;
        let reveal = ctx.reveal_hint;
        fill(buf, area, t.base());
        let hints: Vec<Hint> = if modal {
            self.layer_hints()
        } else {
            let focus = self.shell.focus.current();
            self.screens
                .get(self.route)
                .map(|s| s.hints(focus, &self.world))
                .unwrap_or_default()
        };
        // the focused cursor row is cut: name the reveal key
        let mut hints = hints;
        junie_tui::ui::reveal::advertise(&mut hints, reveal, |h| h.key, hint);
        let editing =
            self.screens.get(self.route).is_some_and(|s| s.is_editing()) || self.layer_editing();
        let badge = if editing {
            Some(("EDIT", BadgeKind::Edit))
        } else {
            None
        };
        let status = self.shell.status.get();
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
        // one hint surface: topmost modal › open menu › the screen's own context › fallback
        let modal_layer = modal.then(|| HintLayer::new(hints.clone()));
        let menu_layer =
            (!modal && Self::is_host(self.route) && self.host_menu.is_open()).then(|| {
                HintLayer::new(vec![
                    hint("←→", "Menu"),
                    hint("↑↓", "Move"),
                    hint("Enter", "Choose"),
                    hint("Esc", "Close"),
                ])
            });
        let screen_layer = (!modal && !hints.is_empty()).then(|| HintLayer::new(hints.clone()));
        let fallback = HintLayer::new(vec![hint("?", "Help"), hint("Esc", "Back")]);
        let mut layer = HintBar::resolve(&[modal_layer, menu_layer, screen_layer, Some(fallback)]);
        layer.badge = badge;
        layer.status = status;
        let spots = HintBar::render_at(area, buf, &t, &layer);
        // an inert frame (the handoff) takes no clicks
        if !ctx.inert {
            self.shell.footer_drawn(layer.hints, spots);
        }
    }
}

/// The brand on the too-small screen.
const BRAND_PADDED: &str = " jackin❯ ";

impl shell::App for App {
    type Status = ();
    fn shell(&self) -> &Shell {
        &self.shell
    }
    fn shell_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }
    fn theme(&self) -> Theme {
        self.theme
    }
    fn too_small(&self) -> TooSmall<'_> {
        TooSmall::new(BRAND_PADDED).brand_style(Lockup::style(&self.theme))
    }
    fn now_ms(&self) -> u64 {
        App::now_ms(self)
    }
    fn tick(&self) -> u64 {
        (self.world.now_ms() / 80) as u64
    }
    fn tick_interval(&self) -> std::time::Duration {
        App::tick_interval(self)
    }
    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        App::draw(self, area, buf, ctx)
    }
    fn on_input(&mut self, input: Input) -> Outcome {
        self.handle_input(input)
    }
    /// Paused captures step the world clock through the app's own tick,
    /// so jobs on virtual time (a failing save, a role load, the launch
    /// pipeline) complete and the frame then settles.
    fn advance_clock(&mut self, ms: u64) -> Option<Outcome> {
        Some(junie_tui::clock::run_step(
            self,
            |a| &mut a.world.clock,
            ms,
            Self::on_tick,
        ))
    }
    fn on_blur(&mut self, ch: FocusChange) -> Outcome {
        self.blur_all(ch)
    }
    fn layer_open(&self) -> bool {
        !self.layers.is_empty()
    }
    fn page_under_layer(&self) -> bool {
        self.layers.hides_page_focus()
            || self.host_menu.hides_page_focus()
            || self.screens.get(self.route).is_some_and(|s| s.layer_open())
    }
    fn page_key(&self) -> PageKey {
        PageKey(self.route as u64)
    }
    fn primary_focus(&self) -> Option<WidgetId> {
        self.screens.get(self.route).and_then(|s| s.primary_focus())
    }
}

impl PointerHost for App {
    fn pointer_parts(&mut self) -> PointerParts<'_> {
        self.shell.pointer_parts()
    }

    fn layer_open(&self) -> bool {
        !self.layers.is_empty()
    }

    fn now_ms(&self) -> u64 {
        App::now_ms(self)
    }

    fn deliver(&mut self, ev: &PointerEvent) -> Response<()> {
        self.deliver_pointer(ev).into()
    }

    /// A click on nothing: the intro or the outro skips; the top layer's
    /// own policy decides (a refused outside click flashes); with no
    /// layer, an open host menu closes.
    fn outside(&mut self, _pos: Position) -> Response<()> {
        if matches!(self.route, Route::Intro | Route::Outro | Route::Handoff) {
            return self.skip_animation().into();
        }
        if !self.layers.is_empty() {
            return self
                .dismiss_layer(DismissReason::OutsideClick)
                .or(Outcome::Changed)
                .into();
        }
        if Self::is_host(self.route) && self.host_menu.is_open() {
            self.host_menu.close();
        }
        Response::changed()
    }
}
