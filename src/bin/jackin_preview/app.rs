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
use junie_tui::shell::{self, Chrome, Menus, PageKey, Router, Shell};
use junie_tui::theme::{Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::layer::{
    DismissReason, Dismissed, LayerCx, LayerEntry, LayerPointer, ShellLayers, Taken,
};
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::hintbar::HintBar;
use junie_tui::widgets::keyhint::hint;
use junie_tui::widgets::picker::{Picker, PickerAction};
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::too_small::TooSmall;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::arbiter::{EntryDecision, ExitDecision};
use crate::domain::instance::InstanceStatus;
use crate::pages::{Page, Pages, TAB_ACCOUNTS, TAB_MANAGER, TAB_USAGE};
use crate::rain::{self, HANDOFF_LEN, IntroState, OutroState, handoff_stage};
use crate::scenario::{Motion, Scenario};
use crate::screens::capsule::CapsuleScreen;
use crate::screens::cockpit::CockpitScreen;
use crate::screens::editor::EditorScreen;
use crate::screens::modals::{FormAction, FormDialog, HelpOverlay, InfoResult};
use crate::screens::prelude::PreludeScreen;
use crate::screens::settings::SettingsScreen;
use crate::screens::{AppLayer, Cx, Go, LayerTag, ModalResult, Request, Screen};
use crate::sim::launch::LaunchPlan;
use crate::sim::world::{Msg, World};
use junie_tui::shell::Page as _;
use junie_tui::ui::leave::{LeaveGuard, Stakes};
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::dialog::DialogResult;
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

pub struct App {
    pub theme: Theme,
    pub scenario: Scenario,
    pub world: World,
    pub intro: Option<IntroState>,
    pub outro: Option<OutroState>,
    /// The handoff's frame while the cockpit hands over to the Capsule
    /// (both are on the router, the Capsule on top).
    pub handoff: Option<u64>,
    /// The pages ([`shell::Router`], [`crate::pages`]): the manager,
    /// Accounts and Usage keep their tabs; every other route opens over
    /// the manager, and Back is the manager.
    pub router: Router<Page>,
    layers: Layers,
    /// Focus, ring, hit regions, pointer, the footer status (5 s on the
    /// world clock; an error stays until the next action), size, motion
    /// and quit: the library shell's state.
    pub shell: Shell,
    /// The shell's keys ([`shell::KeyRouter`], [`shell_keys`]): Ctrl+C
    /// and Ctrl+Q quit through the guard, Esc never quits, `q` leaves the
    /// host screen (never over a field or unsaved edits), `u`/`c`/`s` the
    /// host routes.
    pub keys: shell::ShellKeys<JackinCmd>,
    /// The quit's leave guard ([`Shell::quit_through`]): while a quit
    /// waits on the screen's unsaved-changes question it is pending, and
    /// when that flow leaves for the manager (Save or Discard), the app
    /// quits.
    quit_guard: LeaveGuard<()>,
    /// The application menu bar shown on every host screen, built from
    /// [`Self::host_menus`].
    pub host_menu: MenuBar,
    /// The host menus, each item with its command (M23).
    host_menus: Menus<HostCmd>,
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
            theme,
            scenario,
            world,
            intro: None,
            outro: None,
            handoff: None,
            router: crate::pages::router(),
            layers: ShellLayers::new(),
            shell,
            keys: shell_keys(),
            quit_guard: LeaveGuard::new(WidgetId::of("quit.guard")),
            host_menu: Self::host_bar(&Self::build_host_menu(Route::Manager)),
            host_menus: Self::build_host_menu(Route::Manager),
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
                        if let Some(c) = self.router.cockpit_mut() {
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
        self.show_tab(TAB_MANAGER);
        self.on_route(Route::Manager, Outcome::Ignored, |s, w, cx| {
            s.enter(w, cx);
            Outcome::Ignored
        });
    }

    /// The shown route: a full-screen sequence (the outro, the intro, the
    /// handoff) over the router, else the router's shown page.
    pub fn route(&self) -> Route {
        if self.outro.is_some() {
            Route::Outro
        } else if self.intro.is_some() {
            Route::Intro
        } else if self.handoff.is_some() {
            Route::Handoff
        } else {
            self.router.current().route()
        }
    }

    /// Show the root of `tab` (the manager, Accounts or Usage): the pages
    /// opened over the manager are dropped, and a sequence still on
    /// screen ends.
    fn show_tab(&mut self, tab: usize) {
        self.intro = None;
        self.handoff = None;
        self.router.select(TAB_MANAGER);
        self.router.pop_to(0);
        self.router.select(tab);
    }

    /// Open `page` over the manager (Back is the manager) and enter it.
    fn open_page(&mut self, page: Page) {
        self.show_tab(TAB_MANAGER);
        self.router.push(page);
        self.enter_route();
    }

    pub fn set_status(&mut self, s: &str, tone: Tone) {
        self.shell.status.set_at(s, tone, self.world.clock.now());
    }

    fn animating(&self) -> bool {
        matches!(
            self.route(),
            Route::Intro | Route::Outro | Route::Handoff | Route::Cockpit | Route::Capsule
        ) || self.shell.pointer.has_flash()
            || self
                .router
                .screen(self.route())
                .is_some_and(|s| s.animating(&self.world))
            || self.layer_animates()
            || !self.world.jobs.is_empty()
            || self.world.daemons.values().any(|d| !d.panes.is_empty())
    }

    pub fn tick_interval(&self) -> std::time::Duration {
        if self.shell.motion == Motion::Paused {
            return std::time::Duration::from_millis(500);
        }
        std::time::Duration::from_millis(self.route().tick_ms(self.animating()))
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
            for route in self.router.routes() {
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
            Input::Mouse(m) => PointerRouter::route(self, m),
            other => shell::KeyRouter::input(self, other),
        }
    }

    fn on_tick(&mut self) -> Outcome {
        let interval = self.route().tick_ms(true) as i64;
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
        match self.route() {
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
        let route = self.route();
        out = out.or(self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_tick(w, cx)));
        // messages
        for m in msgs {
            out = out.or(self.dispatch_msg(m));
        }
        out
    }

    fn dispatch_msg(&mut self, m: Msg) -> Outcome {
        // the job's result is here: its progress status ends before the
        // owner reports the result (a failure that only opens a dialog
        // leaves no "Saving…" under it)
        let ended = m.job().is_some_and(|job| self.shell.status.end_job(job));
        let route = self.route();
        let (o, reqs) = self
            .shell
            .deliver(true, |cx| match self.router.screen_mut(route) {
                Some(s) => s.on_msg(&m, &mut self.world, cx),
                None => Outcome::Ignored,
            });
        // the manager and accounts follow the world from behind; the
        // background-request policy keeps only their statuses
        let (o, behind) = self.shell.deliver(false, |cx| {
            let mut o = o;
            if !o.consumed() && route != Route::Manager {
                o = self.router.manager_mut().on_msg(&m, &mut self.world, cx);
            }
            if !o.consumed() && route != Route::Accounts {
                o = self.router.accounts_mut().on_msg(&m, &mut self.world, cx);
            }
            o
        });
        let o = o.or(self.apply_requests(reqs, route));
        let o = o.or(self.apply_requests(behind, route));
        if ended { o.or(Outcome::Changed) } else { o }
    }

    fn finish_intro(&mut self) {
        self.world.arbiter.complete_entry(self.world.now_ms());
        self.intro = None;
        self.enter_manager();
    }

    fn finish_handoff(&mut self) {
        self.handoff = None;
        // the Capsule takes the cockpit's place over the manager
        if let Some(capsule) = self.router.back() {
            self.router.replace(capsule);
        }
        self.on_route(Route::Capsule, Outcome::Ignored, |s, w, cx| {
            s.enter(w, cx);
            Outcome::Ignored
        });
    }

    /// A key on a full-screen sequence (the intro, the outro, the
    /// handoff): Enter and Esc skip it; Ctrl+C leaves the intro and the
    /// outro at once. `None` on any other route.
    fn sequence_key(&mut self, key: &Key) -> Option<Outcome> {
        match self.route() {
            Route::Intro => {
                if self.intro_guard > 0 && self.shell.motion != Motion::Paused {
                    return Some(Outcome::Consumed);
                }
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    if let Some(s) = self.intro.as_mut() {
                        s.skip();
                        if s.is_done() {
                            self.finish_intro();
                        }
                    }
                    return Some(Outcome::Changed);
                }
                if key.ctrl_char('c') {
                    self.world.arbiter.release_entry();
                    self.shell.quit = true;
                }
                Some(Outcome::Consumed)
            }
            Route::Outro => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    if let Some(s) = self.outro.as_mut() {
                        s.skip();
                        if s.is_done() {
                            self.shell.quit = true;
                        }
                    }
                    return Some(Outcome::Changed);
                }
                if key.ctrl_char('c') {
                    self.shell.quit = true;
                }
                Some(Outcome::Consumed)
            }
            Route::Handoff => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.finish_handoff();
                }
                Some(Outcome::Changed)
            }
            _ => None,
        }
    }

    /// The host screen's page is editing or has its own menu open.
    fn host_editing(&self) -> bool {
        self.router
            .screen(self.route())
            .is_some_and(|s| s.is_editing() || s.layer_open())
    }

    /// `q` on a host screen: never over unsaved edits; the manager quits,
    /// any other host screen goes back to it.
    fn leave_key(&mut self) -> Outcome {
        let route = self.route();
        let dirty = self
            .router
            .screen_mut(route)
            .is_some_and(|s| !s.stakes(&self.world).is_clean());
        if dirty {
            self.set_status("Unsaved changes · save, or Esc to leave", Tone::Secondary);
        } else if route == Route::Manager {
            return self.request_quit();
        } else {
            self.go(Go::Manager);
        }
        Outcome::Changed
    }

    /// Every quit (Ctrl+Q, Ctrl+C, `q`, the menu) goes through here: a
    /// clean screen quits at once (running instances keep running in the
    /// Construct, so nothing is lost); unsaved edits ask the screen's own
    /// Save / Discard / Cancel first, and the app quits when that flow
    /// leaves the screen.
    fn request_quit(&mut self) -> Outcome {
        let stakes = shell::App::stakes(self);
        match self
            .shell
            .quit_through(&mut self.quit_guard, &stakes, &self.theme.strings)
        {
            Ok(()) => {
                self.world.arbiter.release_entry();
                Outcome::Consumed
            }
            // the guard keeps the quit pending; the screen asks its own
            // Save / Discard / Cancel (its save previews first), not the
            // guard's generic question (S20 moves it into EditorShell)
            Err(_) => {
                let route = self.route();
                self.on_route(route, Outcome::Ignored, |s, w, cx| s.request_leave(w, cx))
            }
        }
    }

    /// Quit now: the user already answered (or nothing is at stake).
    fn quit_now(&mut self) {
        self.world.arbiter.release_entry();
        self.shell.quit = true;
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

    /// The menus of the host screens: the same bar as the Capsule's, with
    /// the screen's own actions under File and the destinations under Go.
    /// Each item carries its command; an action item presses the key it
    /// names, so menus and keys can never disagree.
    fn build_host_menu(route: Route) -> Menus<HostCmd> {
        use HostCmd::{Press, Quit};
        let k = |c: char| Press(plain(KeyCode::Char(c)));
        let file: Vec<(MenuItem, HostCmd)> = match route {
            Route::Manager => vec![
                (MenuItem::new("New workspace…").shortcut("n"), k('n')),
                (MenuItem::new("Edit workspace…").shortcut("e"), k('e')),
                (
                    MenuItem::new("Launch…").shortcut("Enter"),
                    Press(plain(KeyCode::Enter)),
                ),
                (MenuItem::new("Prewarm").shortcut("w"), k('w')),
                (
                    MenuItem::new("Delete workspace…")
                        .shortcut("d")
                        .danger()
                        .separator(),
                    k('d'),
                ),
                (
                    MenuItem::new("Refresh").shortcut("F5").separator(),
                    Press(plain(KeyCode::F(5))),
                ),
                (MenuItem::new("Quit").shortcut("Ctrl+Q"), Quit),
            ],
            Route::Editor | Route::Settings => vec![
                (
                    MenuItem::new("Save…").shortcut("Ctrl+S").separator(),
                    Press(Key {
                        code: KeyCode::Char('s'),
                        mods: KeyModifiers::CONTROL,
                    }),
                ),
                (
                    MenuItem::new("Cancel").shortcut("Esc"),
                    Press(plain(KeyCode::Esc)),
                ),
            ],
            Route::Accounts => vec![
                (MenuItem::new("Add account…").shortcut("a"), k('a')),
                (MenuItem::new("Edit account…").shortcut("e"), k('e')),
                (MenuItem::new("Validate").shortcut("v").separator(), k('v')),
                (MenuItem::new("Refresh selection").shortcut("r"), k('r')),
                (
                    MenuItem::new("Refresh all").shortcut("F5"),
                    Press(plain(KeyCode::F(5))),
                ),
                (MenuItem::new("Filter…").shortcut("/"), k('/')),
            ],
            Route::Usage => vec![
                (MenuItem::new("Refresh").shortcut("r"), k('r')),
                (MenuItem::new("Manage in Accounts").shortcut("m"), k('m')),
            ],
            _ => vec![(
                MenuItem::new("Cancel").shortcut("Esc"),
                Press(plain(KeyCode::Esc)),
            )],
        };
        let go = vec![
            (
                MenuItem::new("Workspace manager").shortcut("Esc"),
                HostCmd::Manager,
            ),
            (
                MenuItem::new("Account & Usage Center").shortcut("c"),
                HostCmd::Accounts,
            ),
            (MenuItem::new("Usage").shortcut("u"), HostCmd::Usage),
            (
                MenuItem::new("Global settings").shortcut("s"),
                HostCmd::Settings,
            ),
        ];
        let help = vec![
            (MenuItem::new("Key reference").shortcut("?"), HostCmd::Help),
            (MenuItem::new("About jackin-preview"), HostCmd::About),
        ];
        Menus::new()
            .menu("File", file)
            .menu("Go", go)
            .menu("Help", help)
    }

    /// The host menu bar showing `menus`.
    fn host_bar(menus: &Menus<HostCmd>) -> MenuBar {
        menus.bar(HOST_MENU).brand(Lockup::new(BRAND_MARK))
    }

    fn sync_host_menu(&mut self) {
        if self.host_menu_route != self.route() && Self::is_host(self.route()) {
            self.host_menus = Self::build_host_menu(self.route());
            self.host_menu = Self::host_bar(&self.host_menus);
            self.host_menu_route = self.route();
        }
    }

    /// What the host menu bar's `action` asks for: a chosen item runs its
    /// command, the brand opens About.
    fn host_menu_action(&mut self, action: MenuBarAction, o: Outcome) -> Outcome {
        if let MenuBarAction::Brand = action {
            self.open_about();
            return Outcome::Changed;
        }
        match self.host_menus.command(action) {
            Some(cmd) => self.run_host_cmd(cmd),
            None => o,
        }
    }

    /// Run a host menu command: a destination navigates (an editing
    /// screen keeps the user until it is saved or cancelled), an action
    /// presses its key through the shell's key order.
    fn run_host_cmd(&mut self, cmd: HostCmd) -> Outcome {
        let editing_screen = matches!(
            self.route(),
            Route::Editor | Route::Settings | Route::Prelude
        );
        match cmd {
            HostCmd::Manager if editing_screen => {
                // the editor asks about unsaved work on its own way out
                return shell::KeyRouter::replay(self, &plain(KeyCode::Esc));
            }
            HostCmd::Manager => self.go(Go::Manager),
            HostCmd::Accounts | HostCmd::Usage | HostCmd::Settings if editing_screen => {
                self.set_status("Save or cancel this screen first", Tone::Secondary);
            }
            HostCmd::Accounts => self.go(Go::Accounts { select: None }),
            HostCmd::Usage => self.go(Go::Usage { select: None }),
            HostCmd::Settings => self.go(Go::Settings),
            HostCmd::Help => self.open_help(),
            HostCmd::About => self.open_about(),
            HostCmd::Quit => return self.request_quit(),
            HostCmd::Press(key) => return shell::KeyRouter::replay(self, &key),
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
        self.open_layer(Box::new(d), LayerTag::new("about"), self.route());
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
        if let Some(s) = self.router.screen(route) {
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
            .router
            .screen(self.route())
            .map(|s| s.crumb(&self.world))
            .unwrap_or_default();
        let sections: Vec<(&str, Vec<(&str, &str)>)> = match self.route() {
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
                        ("H J K L", "resize split"),
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
                ("Everywhere", vec![("F5", "refresh now")]),
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
        // the screen keys each screen handles in its own code, by hand
        // until they move onto key maps (S16); the shell's from ShellKeys
        let mut sections: Vec<junie_tui::widgets::help::HelpSection> = sections
            .into_iter()
            .map(|(title, rows)| {
                rows.into_iter().fold(
                    junie_tui::widgets::help::HelpSection::new(title),
                    |s, (k, v)| s.row(k, v),
                )
            })
            .collect();
        sections.push(self.keys.help_section("Shell", |c| {
            Some(match c {
                JackinCmd::Usage => "Usage",
                JackinCmd::Accounts => "Accounts",
                JackinCmd::Settings => "Settings",
                JackinCmd::Leave => "Leave this screen",
            })
        }));
        let h = HelpOverlay::from_keymaps(WidgetId::of("help"), &scope, sections);
        self.open_layer(Box::new(h), LayerTag::new("help"), self.route());
    }

    // ------------------------------------------------------------ layers

    /// Open `layer` on top for `owner`: focus moves into it (the stack
    /// remembers the control it covers, K9) and the page under it shows
    /// no hover or press.
    fn open_layer(&mut self, layer: Box<AppLayer>, tag: LayerTag, owner: Route) {
        self.layers.open(layer, tag, owner, &mut self.shell.focus);
        self.shell.pointer.hover = None;
        self.shell.pointer.pressed = None;
        if let Some(c) = self.router.capsule_mut() {
            c.dialog_open = true;
        }
    }

    /// A layer closed: the Capsule takes input again once none is open.
    fn layer_gone(&mut self) {
        if self.layers.is_empty()
            && let Some(c) = self.router.capsule_mut()
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
                self.copy(v, None);
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

    /// Put `v` on the preview clipboard through the shell's
    /// ([`Shell::copy_at`]): the status says what was copied, named
    /// `what` when the screen gave a name.
    fn copy(&mut self, v: String, what: Option<String>) {
        let now = self.world.clock.now();
        self.shell.copy_at(&v, what.as_deref(), now);
        self.world.clipboard = Some(v);
        self.clipboard_gen += 1;
    }

    /// The picker on top asks its owner for rows matching its query.
    fn refresh_picker(&mut self) {
        let Some(top) = self.layers.top() else {
            return;
        };
        let Some(p) = top.get::<Picker>() else {
            return;
        };
        let (tag, owner, query) = (top.tag.clone(), top.owner, p.query().to_owned());
        let items = self
            .router
            .screen_mut(owner)
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
        if let Some(s) = self.router.screen_mut(owner) {
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

    // ------------------------------------------------------------- mouse

    /// One pointer event the router resolved: the footer's hints first
    /// (they are drawn above every layer), then the top layer while one
    /// is open, then the chrome (host menu bar, identity strip) and the
    /// route's screen.
    fn deliver_pointer(&mut self, ev: &PointerEvent) -> Outcome {
        if matches!(self.route(), Route::Intro | Route::Outro | Route::Handoff) {
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
                Some(key) => shell::KeyRouter::replay(self, &key).or(Outcome::Changed),
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
        if matches!(self.route(), Route::Intro | Route::Outro)
            && self.shell.pointer.config.click_skips_animation
        {
            return shell::KeyRouter::replay(
                self,
                &Key {
                    code: KeyCode::Enter,
                    mods: KeyModifiers::NONE,
                },
            );
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
        if Self::is_host(self.route()) {
            // a title opens on the press (MenuBar::open_on); a click
            // chooses a row, the brand opens About
            if let Some(r) = self.host_menu.route(ev) {
                let (o, act) = r.into_parts();
                return Some(match act {
                    Some(a) => self.host_menu_action(a, o),
                    None => o,
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
        let route = self.route();
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
        let shown = route == self.route();
        let (o, reqs) = self
            .shell
            .deliver(shown, |cx| match self.router.screen_mut(route) {
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
                Request::Progress(s, job) => {
                    let now = self.world.clock.now();
                    self.shell.status.progress_at(job, s, now);
                }
                Request::Open(layer, tag) => self.open_layer(layer, tag, owner),
                // the owner closes its layer without a result
                Request::Close => {
                    self.layers.pop(&mut self.shell.focus);
                    self.layer_gone();
                }
                Request::Go(g) => self.go(g),
                Request::Help => self.open_help(),
                Request::Back => self.go(Go::Manager),
                Request::Copy(s, what) => self.copy(s, what),
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
        // (Save or Discard): the quit goes on
        if self.quit_guard.pending().is_some() && matches!(g, Go::Manager) {
            self.quit_guard.resolve(DialogResult::Action(1));
            self.quit_now();
            return;
        }
        let now = self.world.now_ms();
        match g {
            Go::Manager => {
                self.show_tab(TAB_MANAGER);
                self.enter_route();
            }
            Go::Settings => {
                let page = Page::Settings(Box::new(SettingsScreen::new(&self.world)));
                self.open_page(page);
            }
            Go::Accounts { select } => {
                self.show_tab(TAB_ACCOUNTS);
                self.router.accounts_mut().select(select);
                self.enter_route();
            }
            Go::Usage { select } => {
                self.show_tab(TAB_USAGE);
                self.router.usage_mut().select(select);
                self.enter_route();
            }
            Go::Editor { workspace, pending } => {
                let page = Page::Editor(Box::new(EditorScreen::new(
                    &self.world,
                    workspace,
                    pending.map(|b| *b),
                )));
                self.open_page(page);
            }
            Go::Prelude => {
                let page = Page::Prelude(Box::new(PreludeScreen::new(&self.world)));
                self.open_page(page);
            }
            Go::Launch {
                workspace,
                role,
                agent,
                account,
                plan,
            } => {
                let page = Page::Cockpit(Box::new(CockpitScreen::new(
                    &self.world,
                    workspace,
                    role,
                    agent,
                    account,
                    plan,
                    self.shell.motion,
                )));
                self.open_page(page);
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
                let page = Page::Capsule(Box::new(CapsuleScreen::new(
                    &instance,
                    &mut self.world,
                    pane,
                )));
                if self.route() == Route::Cockpit {
                    // the cockpit hands over: the Capsule goes on top of
                    // it until the handoff ends (`finish_handoff`)
                    self.router.push(page);
                    self.handoff = Some(0);
                    if self.shell.motion == Motion::Reduced {
                        self.finish_handoff();
                    }
                } else {
                    self.open_page(page);
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
                    .router
                    .capsule()
                    .map(|c| c.instance.clone())
                    .unwrap_or_default();
                if let Some(d) = self.world.daemons.get_mut(&inst) {
                    d.attached_by = None;
                }
                self.show_tab(TAB_MANAGER);
                self.router
                    .manager_mut()
                    .select_instance(&inst, &self.world);
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
                // the Capsule closes
                self.show_tab(TAB_MANAGER);
                self.leave_construct(&instance);
            }
            Go::LaunchFailedAck { instance } => {
                // the cockpit closes
                self.show_tab(TAB_MANAGER);
                if self.world.running_count() > 0 {
                    if let Some(i) = instance {
                        self.router.manager_mut().select_instance(&i, &self.world);
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
            // the screen asked and the user chose to quit
            Go::Quit => self.quit_now(),
        }
    }

    /// Foreground instance left: decide between the outro, still-inside
    /// feedback, an already-ended Construct, or a fail-closed return.
    fn leave_construct(&mut self, instance: &str) {
        let decision = self.world.arbiter.request_exit(self.world.now_ms());
        match decision {
            ExitDecision::Outro { elapsed_secs } => {
                self.outro = Some(OutroState::new(self.shell.motion, elapsed_secs, 0));
                self.layers.clear(&mut self.shell.focus);
            }
            ExitDecision::StillInside { remaining } => {
                self.show_tab(TAB_MANAGER);
                self.router
                    .manager_mut()
                    .select_instance(instance, &self.world);
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
                self.router.manager_mut().still_inside =
                    Some((text.clone(), rows, self.world.now_ms() + 6_000));
                self.set_status(&text, Tone::Secondary);
            }
            ExitDecision::AlreadyEnded => {
                self.show_tab(TAB_MANAGER);
                self.enter_route();
                self.set_status(
                    "Another client already ended the Construct · returning to the host",
                    Tone::Secondary,
                );
                self.exit_after_status = Some(self.world.now_ms() + 2_500);
            }
            ExitDecision::Unknown(e) => {
                self.show_tab(TAB_MANAGER);
                self.enter_route();
                self.set_status(&format!("Could not confirm remaining instances: {} · returning to host without the outro", e.label()), Tone::Warning);
                self.exit_after_status = Some(self.world.now_ms() + 2_500);
            }
        }
    }

    fn enter_route(&mut self) {
        let route = self.route();
        self.on_route(route, Outcome::Ignored, |s, w, cx| {
            s.enter(w, cx);
            Outcome::Ignored
        });
    }

    // ------------------------------------------------------------ render

    fn draw(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        match self.route() {
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
                    let footer = Self::chrome(r).regions(area).footer;
                    self.footer(true).render(footer, buf, ctx, &mut self.shell);
                }
            }
        }
    }

    /// The rows around `route`'s page ([`Chrome`]): a header row (the host
    /// menu bar or the identity strip) and the footer, a blank row between
    /// each and the body. The Capsule owns its chrome (menu bar, tabs,
    /// status bar) above the footer; the cockpit keeps the identity strip
    /// but uses the full width; a host screen keeps a column clear on each
    /// side.
    fn chrome(route: Route) -> Chrome {
        match route {
            Route::Capsule => Chrome::new().header(0).gap(0),
            Route::Cockpit => Chrome::new(),
            _ => Chrome::new().inset(1),
        }
    }

    /// Strip + body + footer for a route (no modals).
    fn draw_frame(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, route: Route) {
        let t = self.theme;
        fill(buf, area, t.base());
        let shell::Regions {
            header,
            body,
            footer,
        } = Self::chrome(route).regions(area);
        if Self::is_host(route) {
            self.draw_host_menu(header, buf, ctx, route);
        } else if !header.is_empty() {
            self.draw_strip(header, buf, ctx, route);
        }
        if let Some(s) = self.router.screen_mut(route) {
            s.render(body, buf, ctx, &self.world);
        }
        self.footer(false).render(footer, buf, ctx, &mut self.shell);
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
        if let Some(s) = self.router.screen(route) {
            left.push(Segment::new(s.crumb(&self.world), Tone::Secondary).priority(7));
        }
        let mut right = vec![];
        if let Some(s) = self.router.screen(route) {
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

    /// The footer ([`shell::Footer`]): the top layer's hints while one is
    /// open (library modals name their own keys, CO4), else the open host
    /// menu's, else the screen's, else `? Help · Esc Back`; the EDIT badge
    /// while a field edits; the status line, where the Manager's
    /// still-inside feedback takes an empty slot.
    fn footer(&self, modal: bool) -> shell::Footer {
        use junie_tui::core::hints::{HasHints, HintCx};
        let focus = self.shell.focus.current();
        let editing = self
            .router
            .screen(self.route())
            .is_some_and(|s| s.is_editing())
            || self.layer_editing();
        let still = self
            .router
            .manager()
            .still_inside
            .as_ref()
            .map(|(s, _, _)| s.as_str());
        let status = match (still, self.shell.status.get()) {
            (Some(s), None) if self.route() == Route::Manager => Some((s, Tone::Secondary)),
            (_, s) => s,
        };
        let footer = if modal {
            shell::Footer::new().layer(self.layers.top().map(|t| t.layer.hints(focus)))
        } else {
            let menu = Self::is_host(self.route())
                .then(|| HasHints::hints(&self.host_menu, &HintCx::new(focus)));
            shell::Footer::new()
                .layer(menu)
                .layer(
                    self.router
                        .screen(self.route())
                        .map(|s| s.hints(focus, &self.world)),
                )
                .fallback(vec![hint("?", "Help"), hint("Esc", "Back")])
        };
        footer.editing(editing).status(status)
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
            || self
                .router
                .screen(self.route())
                .is_some_and(|s| s.layer_open())
    }
    /// The router's key for a page; a full-screen sequence has its own.
    fn page_key(&self) -> PageKey {
        match self.route() {
            r @ (Route::Intro | Route::Outro | Route::Handoff) => PageKey(u64::MAX - r as u64),
            _ => self.router.key(),
        }
    }
    /// What quitting would lose: every page's stakes (only the editor
    /// and settings ever hold unsaved edits).
    fn stakes(&mut self) -> Stakes {
        let world = &self.world;
        self.router
            .pages_mut()
            .fold(Stakes::Clean, |s, p| s.max(p.screen_mut().stakes(world)))
    }
    fn primary_focus(&self) -> Option<WidgetId> {
        self.router
            .screen(self.route())
            .and_then(|s| s.primary_focus())
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
        if matches!(self.route(), Route::Intro | Route::Outro | Route::Handoff) {
            return self.skip_animation().into();
        }
        if !self.layers.is_empty() {
            return self
                .dismiss_layer(DismissReason::OutsideClick)
                .or(Outcome::Changed)
                .into();
        }
        if Self::is_host(self.route()) && self.host_menu.is_open() {
            self.host_menu.close();
        }
        Response::changed()
    }
}

/// What a host menu item does ([`App::build_host_menu`]): a choice
/// resolves to its command, never to its label (M23).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostCmd {
    /// Go › Workspace manager.
    Manager,
    /// Go › Account & Usage Center.
    Accounts,
    /// Go › Usage.
    Usage,
    /// Go › Global settings.
    Settings,
    /// Help › Key reference.
    Help,
    /// Help › About jackin-preview.
    About,
    /// File › Quit.
    Quit,
    /// A screen action: press the key the item names.
    Press(Key),
}

/// `code` with no modifier.
fn plain(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

/// jackin's own shell commands ([`shell::ShellAction::App`]): the host
/// screens' letters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JackinCmd {
    /// `u`: Usage (from the manager and accounts).
    Usage,
    /// `c`: Accounts (from the manager and usage).
    Accounts,
    /// `s`: Settings (from the manager, accounts and usage).
    Settings,
    /// `q`: leave the host screen (the manager quits).
    Leave,
}

/// jackin's shell keys: the standard set without a quit letter (`q`
/// leaves the screen instead), the host letters before the page, and the
/// menu bar never over an editing field.
pub fn shell_keys() -> shell::ShellKeys<JackinCmd> {
    use junie_tui::core::keymap::KeyBinding;
    use shell::ShellAction::{self, App as Cmd};
    shell::ShellKeys::new()
        .quit(junie_tui::ui::leave::QuitKeys::DEFAULT.letter(None))
        .bind(KeyBinding::char('u'), Cmd(JackinCmd::Usage))
        .bind(KeyBinding::char('c'), Cmd(JackinCmd::Accounts))
        .bind(KeyBinding::char('s'), Cmd(JackinCmd::Settings))
        .bind_fallback(KeyBinding::char('q'), Cmd(JackinCmd::Leave))
        .idle_only(ShellAction::MenuBar)
}

impl shell::KeyHost for App {
    type Cmd = JackinCmd;

    fn shell_keys(&self) -> &shell::ShellKeys<JackinCmd> {
        &self.keys
    }

    /// The intro, outro and handoff own the keyboard, then the layer
    /// stack. With neither, the user is back on the page: a quit that
    /// waited on the leave question was cancelled.
    fn layer_key(&mut self, key: &Key) -> Option<Outcome> {
        if let Some(o) = self.sequence_key(key) {
            return Some(o);
        }
        if !self.layers.is_empty() {
            return Some(App::layer_key(self, *key));
        }
        self.quit_guard.resolve(DialogResult::Cancelled);
        None
    }

    fn menu_key(&mut self, key: &Key) -> Option<Outcome> {
        if !(Self::is_host(self.route()) && self.host_menu.is_open()) {
            return None;
        }
        let (o, ev) = self.host_menu.on_key(key);
        Some(match ev {
            Some(a @ (MenuBarAction::Chosen(..) | MenuBarAction::Brand)) => {
                self.host_menu_action(a, o)
            }
            _ => o.or(Outcome::Changed),
        })
    }

    /// The cockpit and the capsule are sessions that own the keyboard; a
    /// host screen captures while a field edits or its own menu is open.
    fn capturing(&self) -> bool {
        matches!(self.route(), Route::Cockpit | Route::Capsule) || self.host_editing()
    }

    fn shell_action(&mut self, action: shell::ShellAction<JackinCmd>, _key: &Key) -> Outcome {
        use shell::ShellAction;
        let route = self.route();
        if !Self::is_host(route) {
            return Outcome::Ignored;
        }
        match action {
            ShellAction::MenuBar => {
                self.sync_host_menu();
                self.host_menu.open_menu(0);
                Outcome::Changed
            }
            ShellAction::Help => {
                self.open_help();
                Outcome::Changed
            }
            ShellAction::App(JackinCmd::Usage)
                if matches!(route, Route::Manager | Route::Accounts) =>
            {
                self.go(Go::Usage { select: None });
                Outcome::Changed
            }
            ShellAction::App(JackinCmd::Accounts)
                if matches!(route, Route::Manager | Route::Usage) =>
            {
                self.go(Go::Accounts { select: None });
                Outcome::Changed
            }
            ShellAction::App(JackinCmd::Settings)
                if matches!(route, Route::Manager | Route::Accounts | Route::Usage) =>
            {
                self.go(Go::Settings);
                Outcome::Changed
            }
            ShellAction::App(JackinCmd::Leave) => self.leave_key(),
            _ => Outcome::Ignored,
        }
    }

    /// The host menu bar as a focus stop (K11): Enter or ↓ opens it, ← →
    /// move between its titles, Esc hands focus back to the page.
    fn chrome_key(&mut self, key: &Key) -> Outcome {
        if !(Self::is_host(self.route()) && self.shell.focus.is(HOST_MENU)) {
            return Outcome::Ignored;
        }
        self.sync_host_menu();
        let (o, ev) = self.host_menu.on_key(key);
        if let Some(MenuBarAction::Opened(_)) = ev {
            return Outcome::Changed;
        }
        if o.consumed() {
            return o;
        }
        if key.is(KeyCode::Esc) {
            let pf = self
                .router
                .screen(self.route())
                .and_then(|s| s.primary_focus());
            self.shell.focus.set(pf);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn page_key(&mut self, key: &Key) -> Outcome {
        let route = self.route();
        self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_key(key, w, cx))
    }

    fn layer_paste(&mut self, text: &str) -> Option<Outcome> {
        let top = self.layers.top_mut()?;
        let o = top.layer.on_paste(text).or(Outcome::Consumed);
        Some(self.settle_layers(o))
    }

    fn page_paste(&mut self, text: &str) -> Outcome {
        let route = self.route();
        self.on_route(route, Outcome::Consumed, |s, w, cx| s.on_paste(text, w, cx))
    }

    /// Esc at the top: the page's ladder. A session (cockpit, capsule)
    /// keeps an Esc it does not use; a host screen at its top shows that
    /// Esc never quits.
    fn esc_top(&mut self) -> Outcome {
        let route = self.route();
        let o = self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_esc_top(w, cx));
        if Self::is_host(route) {
            o
        } else {
            o.or(Outcome::Consumed)
        }
    }

    /// Ctrl+C before it quits: the shown page's interrupt
    /// ([`shell::Page::on_interrupt`], SI17).
    fn interrupt(&mut self) -> Outcome {
        let route = self.route();
        self.on_route(route, Outcome::Ignored, |s, w, cx| s.on_interrupt(w, cx))
    }

    fn request_quit(&mut self) -> Outcome {
        App::request_quit(self)
    }

    fn note(&mut self, text: &str) {
        self.set_status(text, Tone::Secondary);
    }
}
