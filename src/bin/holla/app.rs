//! Application shell: the tab strip (Here plus every activity and plan),
//! the Here tab's page stack, the modal stack, focus/hover/press state, the
//! menu bar, the status bar and the hint bar. The launch pipeline lives
//! here too: trust, arguments and the confirmation gates are resolved by
//! the shell before anything executes.

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::ui::text::{truncate, truncate_middle, width};
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::menu::{
    ContextMenu, MenuBar, MenuBarEvent, MenuEvent, MenuItem, Placement,
};
use junie_tui::widgets::picker::{Picker, PickerEvent, PickerItem};
use junie_tui::widgets::progress::MeterTone;
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::statusbar::{StatusBar, StatusItem};
use junie_tui::widgets::tabs::{TabEvent, TabItem, Tabs};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use crate::domain::action::{Confirmation, Item, Launch, Risk};
use crate::domain::activity::{ActivityKind, ActivityState};
use crate::domain::context::{HostRole, Scope};
use crate::domain::plan::PlanPhase;
use crate::scenario::{Motion, Scenario};
use crate::screens::activity::ActivityTab;
use crate::screens::cleanup::CleanupPage;
use crate::screens::disk::DiskPage;
use crate::screens::files::FilesPage;
use crate::screens::finder::FinderPage;
use crate::screens::modals::{TextLine, TextModal};
use crate::screens::plan::{PlanReviewPage, PlanTab};
use crate::screens::review::{ArgsPage, GatePage, TrustPage};
use crate::screens::snapshot::SnapshotPage;
use crate::screens::{Cx, GateTarget, Go, Modal, ModalResult, ModalTag, Page, Request, Screen};
use crate::sim::world::{Msg, World};

pub const MIN_WIDTH: u16 = 72;
pub const MIN_HEIGHT: u16 = 20;
/// The canonical product mark; every brand lockup renders exactly this.
pub const BRAND_MARK: &str = "holla❯";

pub const MENU: WidgetId = WidgetId::of("shell.menu");
pub const TABS: WidgetId = WidgetId::of("shell.tabs");
const STATUS_PATH: WidgetId = WidgetId::of("shell.status.path");
const STATUS_RUNNING: WidgetId = WidgetId::of("shell.status.running");
const STATUS_GIT: WidgetId = WidgetId::of("shell.status.git");

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabKind {
    Here,
    Activity(String),
    Plan(String),
}

pub struct TabEntry {
    pub kind: TabKind,
    pub stack: Vec<Box<dyn Screen>>,
}

pub(crate) struct ModalEntry {
    modal: Modal,
    tag: ModalTag,
    owner: usize,
    saved_focus: Option<WidgetId>,
}

/// What an alternatives-menu row does.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MenuAction {
    Run,
    Copy,
    Insert,
    Args,
    Alternative(String),
    Pin,
    Alias,
    Hide,
    Reset,
    Why,
}

pub struct App {
    pub theme: Theme,
    pub scenario: Scenario,
    pub motion: Motion,
    pub world: World,
    pub tabs: Vec<TabEntry>,
    pub active: usize,
    pub(crate) modals: Vec<ModalEntry>,
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
    menu: MenuBar,
    strip: Tabs,
    last_click: Option<(WidgetId, i64)>,
    menu_item: Option<String>,
    menu_actions: Vec<MenuAction>,
    pending_gate: Option<GateTarget>,
    pending_confirm: Option<(String, Vec<(String, String)>)>,
    pub clipboard_gen: u32,
    /// The quit was confirmed: every live activity is being stopped and the
    /// shell leaves once the last one has settled.
    pub quitting: bool,
    /// The query that led to the last invocation, for query learning.
    pub last_query: Option<String>,
}

impl App {
    /// Pure fixture entry point: one coherent world per scenario, advanced
    /// `frame` ticks through its scripted timeline.
    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64, theme: Theme) -> Self {
        let world = crate::domain::fixtures::world_for(scenario, motion);
        let mut app = Self {
            theme,
            scenario,
            motion,
            world,
            tabs: vec![TabEntry {
                kind: TabKind::Here,
                stack: vec![Box::new(FinderPage::new(None))],
            }],
            active: 0,
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
            menu: Self::build_menu(),
            strip: Tabs::with_items(TABS, vec![]),
            last_click: None,
            menu_item: None,
            menu_actions: vec![],
            pending_gate: None,
            pending_confirm: None,
            clipboard_gen: 0,
            quitting: false,
            last_query: None,
        };
        app.start();
        // frames advance through the scripted timeline, then freeze
        let running = app.world.clock.running;
        app.world.clock.running = true;
        for _ in 0..frame {
            app.on_tick();
        }
        app.world.clock.running = running;
        app
    }

    fn start(&mut self) {
        self.enter_top();
        match self.scenario {
            Scenario::DockerCleanup => {
                if let Some(f) = self.finder_mut() {
                    f.query = "docker clean".into();
                }
                self.enter_top();
            }
            Scenario::DiskCleanup => {
                let cwd = self.world.location.cwd.clone();
                self.push_page(Page::Disk { path: cwd });
            }
            Scenario::UpgradePlan => {
                self.push_page(Page::PlanReview {
                    plan: "upgrade".into(),
                });
            }
            Scenario::ActivitiesMulti => {
                let ids: Vec<String> = self.world.activities.iter().map(|a| a.id.clone()).collect();
                for id in ids {
                    self.open_activity_tab(&id, false);
                }
                self.active = 0;
                self.enter_top();
            }
            Scenario::LaunchFailure => {
                let ids: Vec<String> = self.world.activities.iter().map(|a| a.id.clone()).collect();
                for id in ids {
                    self.open_activity_tab(&id, true);
                }
            }
            _ => {}
        }
    }

    fn finder_mut(&mut self) -> Option<&mut FinderPage> {
        self.tabs[0].stack.first_mut()?.as_finder()
    }

    pub fn set_status(&mut self, s: &str, tone: Tone) {
        self.status = Some((s.to_owned(), tone, self.world.now_ms() + 5_000));
    }

    fn top(&self) -> Option<&dyn Screen> {
        self.tabs
            .get(self.active)
            .and_then(|t| t.stack.last())
            .map(|b| b.as_ref())
    }

    fn animating(&self) -> bool {
        self.flash.is_some()
            || self.world.discovering()
            || self.world.live_activities() > 0
            || self
                .world
                .plans
                .iter()
                .any(|p| p.phase == PlanPhase::Running)
            || self.top().is_some_and(|s| s.animating(&self.world))
    }

    pub fn tick_interval(&self) -> std::time::Duration {
        if self.motion == Motion::Paused {
            return std::time::Duration::from_millis(500);
        }
        std::time::Duration::from_millis(if self.animating() { 80 } else { 200 })
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

    /// Run a closure against the top screen with a context, then apply its
    /// requests.
    fn with_top<F: FnOnce(&mut Box<dyn Screen>, &mut World, &mut Cx) -> Outcome>(
        &mut self,
        f: F,
    ) -> Outcome {
        let active = self.active;
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let o = match self.tabs.get_mut(active).and_then(|t| t.stack.last_mut()) {
            Some(s) => f(s, &mut self.world, &mut cx),
            None => Outcome::Ignored,
        };
        let reqs = std::mem::take(&mut cx.requests);
        o.or(self.apply_requests(reqs, active))
    }

    fn enter_top(&mut self) {
        self.with_top(|s, w, cx| {
            s.enter(w, cx);
            Outcome::Changed
        });
        let pf = self.top().and_then(|s| s.primary_focus());
        self.focus.set(pf);
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
        let msgs = self.world.tick();
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
        if self
            .modals
            .last()
            .is_some_and(|m| m.tag.kind == "activities")
        {
            let items = self.activity_picker_items();
            if let Some(top) = self.modals.last_mut()
                && let Modal::Picker(p) = &mut top.modal
            {
                p.refresh_items(items);
            }
        }
        if self.quitting {
            let live = self.world.live_activities();
            if live == 0 {
                self.quit = true;
            } else {
                let text = format!(
                    "stopping {} · {} still cancelling · quitting when every process is gone",
                    crate::screens::plural(live, "activity", "activities"),
                    self.world.cancelling()
                );
                self.status = Some((text, Tone::Secondary, self.world.now_ms() + 5_000));
            }
            out = Outcome::Changed;
        }
        // every screen on every tab ticks so background tabs keep their state
        let active = self.active;
        for ti in 0..self.tabs.len() {
            let mut cx = Cx {
                focus: &mut self.focus,
                ring: &self.ring,
                requests: vec![],
            };
            let mut o = Outcome::Ignored;
            if let Some(s) = self.tabs[ti].stack.last_mut() {
                o = s.on_tick(&mut self.world, &mut cx);
            }
            let reqs = std::mem::take(&mut cx.requests);
            if ti == active {
                out = out.or(o).or(self.apply_requests(reqs, ti));
            }
        }
        for m in msgs {
            out = out.or(self.dispatch_msg(m));
        }
        out
    }

    fn dispatch_msg(&mut self, m: Msg) -> Outcome {
        let mut out;
        match &m {
            Msg::ActivityEnded { id, ok } => {
                if let Some(a) = self.world.activity(id) {
                    let name = a.name.clone();
                    let exit = a.exit.unwrap_or(0);
                    if *ok {
                        self.set_status(&format!("{name} finished · exit 0"), Tone::Secondary);
                    } else if a.state == ActivityState::Stopped {
                        self.set_status(&format!("{name} stopped · exit {exit}"), Tone::Secondary);
                    } else {
                        self.set_status(&format!("{name} failed · exit {exit}"), Tone::Error);
                    }
                }
                out = Outcome::Changed;
            }
            Msg::BatchDone { id, ok } => {
                let label = self
                    .world
                    .batches
                    .iter()
                    .find(|b| &b.id == id)
                    .map(|b| {
                        let (okn, failed, cancelled, _) = b.tally(&self.world);
                        format!(
                            "{} · {okn} ok · {failed} failed · {cancelled} cancelled",
                            b.label
                        )
                    })
                    .unwrap_or_default();
                self.set_status(&label, if *ok { Tone::Secondary } else { Tone::Error });
                out = Outcome::Changed;
            }
            Msg::PlanDone { plan } => {
                if let Some(p) = self.world.plan(plan) {
                    let (ok, failed, excluded, skipped, never) = p.outcome();
                    let tone = if failed > 0 {
                        Tone::Error
                    } else {
                        Tone::Secondary
                    };
                    let _ = (ok, excluded, skipped, never);
                    self.set_status(
                        &if failed > 0 {
                            format!("{} · {failed} failed", p.title)
                        } else {
                            format!("{} finished", p.title)
                        },
                        tone,
                    );
                }
                out = Outcome::Changed;
            }
            Msg::DiscoveryDone { .. } | Msg::PlanStepEnded { .. } | Msg::ScanDone => {
                out = Outcome::Changed
            }
        }
        for ti in 0..self.tabs.len() {
            let mut cx = Cx {
                focus: &mut self.focus,
                ring: &self.ring,
                requests: vec![],
            };
            if let Some(s) = self.tabs[ti].stack.last_mut() {
                s.on_msg(&m, &mut self.world, &mut cx);
            }
            let reqs = std::mem::take(&mut cx.requests);
            if ti == self.active {
                out = out.or(self.apply_requests(reqs, ti));
            }
        }
        out
    }

    fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(top) = self.modals.last_mut() {
            return match &mut top.modal {
                Modal::Dialog(d) => d.on_paste(text),
                Modal::Picker(p) => {
                    // one QueryChanged per paste, never one per grapheme
                    let (o, ev) = p.on_paste(text);
                    if top.tag.kind == "files-jump" {
                        return self.files_jump_event(ev, o);
                    }
                    o.or(Outcome::Changed)
                }
                _ => Outcome::Consumed,
            };
        }
        let text = text.to_owned();
        self.with_top(|s, w, cx| s.on_paste(&text, w, cx))
    }

    fn attached(&self) -> bool {
        self.top().is_some_and(|s| s.is_editing())
            && matches!(
                self.tabs.get(self.active).map(|t| &t.kind),
                Some(TabKind::Activity(_))
            )
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.too_small {
            if key.is_char('q') || key.ctrl_char('c') {
                self.quit = true;
            }
            return Outcome::Consumed;
        }
        if !self.modals.is_empty() {
            return self.modal_key(key);
        }
        if self.menu.is_open() {
            let (o, ev) = self.menu.on_key(&key);
            return match ev {
                Some(MenuBarEvent::Chosen(mi, ii)) => {
                    let label = self.menu.menus[mi][ii].label.clone();
                    self.run_menu(&label)
                }
                Some(MenuBarEvent::Brand) => {
                    self.open_about();
                    Outcome::Changed
                }
                _ => o.or(Outcome::Changed),
            };
        }
        let attached = self.attached();
        if attached {
            // an attached program owns the keyboard except for the detach chord
            if key.ctrl() && matches!(key.code, KeyCode::Char(']') | KeyCode::Char('5')) {
                return self.with_top(|s, w, cx| s.on_key(&key, w, cx));
            }
            return self
                .with_top(|s, w, cx| s.on_key(&key, w, cx))
                .or(Outcome::Consumed);
        }
        let typing = self.top().is_some_and(|s| s.typing_hot());
        // global chords
        match key.code {
            KeyCode::F(10) => {
                self.menu.open_menu(0);
                return Outcome::Changed;
            }
            KeyCode::F(1) => {
                self.open_help();
                return Outcome::Changed;
            }
            KeyCode::Char('?') if !typing && key.plain() => {
                self.open_help();
                return Outcome::Changed;
            }
            KeyCode::Char('q') if key.ctrl() => {
                self.open_quit_confirm();
                return Outcome::Changed;
            }
            KeyCode::Char('c') if key.ctrl() => {
                self.quit = true;
                return Outcome::Consumed;
            }
            KeyCode::Char('g') if key.ctrl() => {
                self.open_activities();
                return Outcome::Changed;
            }
            KeyCode::Char('w') if key.ctrl() => {
                self.go(Go::CloseTab);
                return Outcome::Changed;
            }
            KeyCode::Char(c) if key.alt() && c.is_ascii_digit() => {
                let n = c as usize - '0' as usize;
                if n < self.tabs.len() {
                    self.activate_tab(n);
                } else {
                    self.set_status(&format!("No tab {n}"), Tone::Secondary);
                }
                return Outcome::Changed;
            }
            _ => {}
        }
        // the menu bar is a focus stop: ← → move the cursor, Enter opens
        if self.focus.is(MENU) {
            let (o, ev) = self.menu.on_key(&key);
            if let Some(MenuBarEvent::Opened(_)) = ev {
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
            if key.code == KeyCode::Esc {
                let pf = self.top().and_then(|s| s.primary_focus());
                self.focus.set(pf);
                return Outcome::Changed;
            }
        }
        // the tab strip is a focus stop
        if self.focus.is(TABS) {
            let (o, ev) = self.strip.on_key(&key);
            match ev {
                Some(TabEvent::Activated(i)) => {
                    self.activate_tab(i);
                    self.focus.focus(TABS);
                    return Outcome::Changed;
                }
                Some(TabEvent::Close(i)) => {
                    self.active = i;
                    self.go(Go::CloseTab);
                    return Outcome::Changed;
                }
                _ => {}
            }
            if o.consumed() {
                return o;
            }
            if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                let pf = self.top().and_then(|s| s.primary_focus());
                self.focus.set(pf);
                return Outcome::Changed;
            }
        }
        let out = self.with_top(|s, w, cx| s.on_key(&key, w, cx));
        if out.consumed() {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
                && key.plain()
                && !typing
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
            KeyCode::Esc => {
                if self.active != 0 {
                    self.activate_tab(0);
                    return Outcome::Changed;
                }
                self.with_top(|s, w, cx| s.on_esc_top(w, cx))
            }
            _ => Outcome::Ignored,
        }
    }

    // ------------------------------------------------------------- menus

    fn build_menu() -> MenuBar {
        let file = vec![
            MenuItem::new("Run").shortcut("Enter"),
            MenuItem::new("Alternatives…").shortcut("Alt+Enter"),
            MenuItem::new("Copy command").separator(),
            MenuItem::new("Close tab").shortcut("Ctrl+W").separator(),
            MenuItem::new("Quit").shortcut("Ctrl+Q"),
        ];
        let go = vec![
            MenuItem::new("Here").shortcut("Alt+0"),
            MenuItem::new("Activities…").shortcut("Ctrl+G").separator(),
            MenuItem::new("Parent scope").shortcut("Ctrl+↑"),
            MenuItem::new("Children scope").shortcut("Ctrl+↓"),
            MenuItem::new("System scope").separator(),
            MenuItem::new("Tasks"),
            MenuItem::new("Git"),
            MenuItem::new("Files"),
            MenuItem::new("Disk"),
            MenuItem::new("Services"),
            MenuItem::new("System"),
        ];
        let help = vec![
            MenuItem::new("Key reference").shortcut("F1"),
            MenuItem::new("Why is this here?"),
            MenuItem::new("About holla"),
        ];
        MenuBar::new(MENU, vec![("File", file), ("Go", go), ("Help", help)])
            .brand(Lockup::new(BRAND_MARK))
    }

    fn run_menu(&mut self, label: &str) -> Outcome {
        let key = |code: KeyCode, mods: KeyModifiers| Input::Key(Key { code, mods });
        match label {
            "Run" => return self.handle(key(KeyCode::Enter, KeyModifiers::NONE)),
            "Alternatives…" => return self.handle(key(KeyCode::Enter, KeyModifiers::ALT)),
            "Copy command" => {
                let cmd = self
                    .current_item()
                    .map(|i| i.commands.join("\n"))
                    .unwrap_or_default();
                if cmd.is_empty() {
                    self.set_status("Nothing to copy here", Tone::Secondary);
                } else {
                    self.copy(cmd);
                }
            }
            "Close tab" => self.go(Go::CloseTab),
            "Quit" => self.open_quit_confirm(),
            "Here" => self.activate_tab(0),
            "Activities…" => self.open_activities(),
            "Parent scope" => self.go(Go::Scope(Scope::Parent)),
            "Children scope" => self.go(Go::Scope(Scope::Children)),
            "System scope" => self.go(Go::Scope(Scope::System)),
            "Tasks" | "Git" | "Files" | "Disk" | "Services" | "System" => {
                let g: &'static str = match label {
                    "Tasks" => "Tasks",
                    "Git" => "Git",
                    "Files" => "Files",
                    "Disk" => "Disk",
                    "Services" => "Services",
                    _ => "System",
                };
                self.activate_tab(0);
                self.push_page(Page::Finder { group: Some(g) });
            }
            "Key reference" => self.open_help(),
            "Why is this here?" => self.open_why(),
            "About holla" => self.open_about(),
            _ => self.set_status(
                &format!("{label}: not available in the preview"),
                Tone::Secondary,
            ),
        }
        Outcome::Changed
    }

    fn current_item(&mut self) -> Option<Item> {
        if self.active != 0 {
            return None;
        }
        let id = self.finder_page_current_id()?;
        self.world.items().into_iter().find(|i| i.id == id)
    }

    fn finder_page_current_id(&mut self) -> Option<String> {
        self.tabs[0].stack.last_mut()?.as_finder()?.current_id()
    }

    fn open_about(&mut self) {
        let lines = vec![
            TextLine::kv("Product", "holla · this folder, this host, right now"),
            TextLine::kv("Scenario", self.world.scenario.name()),
            TextLine::kv(
                "Simulation",
                "mise, git, gh, docker, btm, pg_activity, ssh and the filesystem are in-memory fixtures · no stack command is ever executed",
            ),
            TextLine::kv(
                "Design system",
                "Junie TUI (junie_tui) · one accent, state as geometry, one hint bar",
            ),
            TextLine::kv(
                "Model",
                "a tab is something that runs · a page is something you are deciding",
            ),
        ];
        let m = TextModal::new(WidgetId::of("about"), "About holla❯", lines).width(70);
        self.push_modal(Modal::Text(m), ModalTag::new("about"));
    }

    fn open_help(&mut self) {
        let kind = self
            .tabs
            .get(self.active)
            .map(|t| t.kind.clone())
            .unwrap_or(TabKind::Here);
        let mut lines = vec![];
        let mut section = |title: &str, rows: &[(&str, &str)]| {
            lines.push(TextLine::heading(title));
            for (k, v) in rows {
                lines.push(TextLine::kv(k, v));
            }
            lines.push(TextLine::blank());
        };
        match kind {
            TabKind::Here => {
                section(
                    "Finder",
                    &[
                        (
                            "Type",
                            "search actions and resources together · @parent @children @system @all narrow the scope",
                        ),
                        ("↑↓", "move · PgUp PgDn Home End"),
                        (
                            "Enter",
                            "primary action · destructive rows open a review instead",
                        ),
                        (
                            "Alt+Enter",
                            "alternatives · run, copy, insert, arguments, pin, alias, hide, why",
                        ),
                        ("Tab", "preview › tab strip › finder"),
                        (
                            "Ctrl+↑ / Ctrl+↓",
                            "scope outward (parent, system) / inward (children)",
                        ),
                        (
                            "Esc",
                            "clear query › scope back to here › back a page › quit",
                        ),
                        (
                            "Aliases",
                            "gp pull · du disk usage · test the test task here · d then u · dc docker cleanup",
                        ),
                    ],
                );
                section(
                    "Pages",
                    &[
                        (
                            "Disk",
                            "↑↓ move · Space select · a all · Enter drill · c cleanup plan · p details",
                        ),
                        (
                            "Plan review",
                            "Space include or exclude · u undo · Enter step facts · c confirm",
                        ),
                        (
                            "Gates",
                            "Gate 1 reviews facts and the full sequence · Gate 2 types the target-bound phrase",
                        ),
                    ],
                );
            }
            TabKind::Activity(_) => {
                section(
                    "Activity",
                    &[
                        ("↑↓ PgUp PgDn", "scroll · f follow tail · y copy selection"),
                        ("s", "stop · r restart · x close tab"),
                        (
                            "Enter / i",
                            "attach to an interactive program · Ctrl+] detaches",
                        ),
                        ("1–9", "show or hide a merged log stream"),
                        ("Esc", "back to Here · the activity keeps running"),
                    ],
                );
            }
            TabKind::Plan(_) => {
                section(
                    "Plan",
                    &[
                        ("↑↓", "select a step · Enter maximise its output · f follow"),
                        ("r", "retry the failed step · k skip an optional failure"),
                        ("Ctrl+C", "cancel the remaining work"),
                        ("Esc", "back to Here · the plan keeps running"),
                    ],
                );
            }
        }
        section(
            "Everywhere",
            &[
                ("F10", "menu bar · F1 this reference"),
                (
                    "Ctrl+G",
                    "activities · Alt+0 Here · Alt+1–9 tabs · Ctrl+W close tab",
                ),
                ("Ctrl+Q", "quit with confirmation · Ctrl+C quit now"),
                (
                    "Mouse",
                    "click selects · double-click runs · right-click alternatives · wheel scrolls under the pointer",
                ),
            ],
        );
        let m = TextModal::new(WidgetId::of("help"), "Key reference", lines)
            .subtitle(&self.crumb())
            .width(78);
        self.push_modal(Modal::Text(m), ModalTag::new("help"));
    }

    fn open_why(&mut self) {
        let Some(it) = self.current_item() else {
            self.set_status("Select a row first", Tone::Secondary);
            return;
        };
        let cwd = self.world.location.cwd_short();
        let rows = crate::domain::ranking::explain(&it, &cwd);
        let mut lines: Vec<TextLine> = rows.iter().map(|(k, v)| TextLine::kv(k, v)).collect();
        lines.push(TextLine::blank());
        lines.push(TextLine::text("Ranking order: alias › pin › live state › context › query › used here › used anywhere › default. Risk changes the gate, never the rank.", Tone::Muted));
        let m = TextModal::new(
            WidgetId::of("why"),
            &format!("Why is “{}” here?", it.label),
            lines,
        )
        .width(74);
        self.push_modal(Modal::Text(m), ModalTag::new("why"));
    }

    fn open_quit_confirm(&mut self) {
        let n = self.world.live_activities();
        let body = if n > 0 {
            format!(
                "{} still running. Quitting stops what holla started; detached monitors keep running.",
                crate::screens::plural(n, "activity is", "activities are")
            )
        } else {
            "Nothing is running.".into()
        };
        // leaving a remote box says which box
        let title = if self.world.host.remote {
            format!("Quit holla❯ on {}?", self.world.host.name)
        } else {
            "Quit holla❯?".to_owned()
        };
        let body = if self.world.host.remote {
            format!(
                "◆ {} · {} · over SSH. {body}",
                self.world.host.name,
                self.world.host.role.label()
            )
        } else {
            body
        };
        let d = if n > 0 {
            Dialog::destructive(WidgetId::of("quit"), &title, &body, "Stop and quit")
        } else {
            Dialog::confirm(WidgetId::of("quit"), &title, &body, "Quit")
        };
        self.push_modal(Modal::Dialog(d), ModalTag::new("quit"));
    }

    fn activity_picker_items(&self) -> Vec<PickerItem> {
        let mut v = vec![];
        for a in &self.world.activities {
            let glyph = match a.state {
                ActivityState::Running | ActivityState::Queued => "⠋",
                ActivityState::Cancelling => "◐",
                ActivityState::Succeeded => "✓",
                ActivityState::Failed => "!",
                ActivityState::Stopped | ActivityState::Detached => "○",
            };
            let mut detail = format!(
                "{} · {} · {}",
                a.state.label(),
                a.scope.word,
                crate::screens::ticks_label(a.duration_ticks(self.world.tick))
            );
            if a.waiting.is_some() {
                detail = format!("{detail} · waiting for input");
            }
            v.push(PickerItem {
                label: a.name.clone(),
                detail,
                glyph,
                group: "activities",
                tag: a
                    .waiting
                    .as_ref()
                    .map(|p| if p.secret { "password" } else { "input" }),
                matched: vec![],
                disabled: false,
                key: a.id.clone(),
            });
        }
        for p in &self.world.plans {
            if p.phase != PlanPhase::Review {
                let (ok, failed, ..) = p.outcome();
                v.push(PickerItem {
                    label: p.title.clone(),
                    detail: format!(
                        "plan · {} · {ok} done · {failed} failed",
                        if p.phase == PlanPhase::Done {
                            "finished"
                        } else {
                            "running"
                        }
                    ),
                    glyph: if p.phase == PlanPhase::Done {
                        if failed > 0 { "!" } else { "✓" }
                    } else {
                        "⠋"
                    },
                    group: "plans",
                    tag: None,
                    matched: vec![],
                    disabled: false,
                    key: format!("plan:{}", p.id),
                });
            }
        }
        v
    }

    fn open_activities(&mut self) {
        let items = self.activity_picker_items();
        let mut p = Picker::new(WidgetId::of("activities"), "Activities");
        p.searchable = false;
        p.width = 70;
        p.empty_text = "Nothing has been started yet".into();
        p.scope = Some(format!("{} live", self.world.live_activities()));
        p.set_items(items);
        self.push_modal(Modal::Picker(p), ModalTag::new("activities"));
    }

    // ------------------------------------------------------------ modals

    fn push_modal(&mut self, modal: Modal, tag: ModalTag) {
        let initial = match &modal {
            Modal::Dialog(d) => Some(d.initial_focus),
            Modal::Picker(p) => Some(p.id),
            Modal::Menu(m) => Some(m.id),
            Modal::Text(t) => Some(t.id),
        };
        self.modals.push(ModalEntry {
            modal,
            tag,
            owner: self.active,
            saved_focus: self.focus.current(),
        });
        self.focus.set(initial);
        self.hover = None;
        self.pressed = None;
    }

    fn pop_modal(&mut self) -> Option<ModalEntry> {
        let e = self.modals.pop();
        if let Some(e) = &e {
            self.focus.set(e.saved_focus);
        }
        e
    }

    fn deliver(&mut self, entry: ModalEntry, result: ModalResult) -> Outcome {
        let owner = entry.owner;
        let tag = entry.tag.clone();
        // shell-owned dialogs first
        match tag.kind {
            "quit" => {
                if let ModalResult::Dialog {
                    action: Some(1), ..
                } = result
                {
                    let n = self.world.stop_all();
                    if n == 0 {
                        self.quit = true;
                    } else {
                        self.quitting = true;
                        self.set_status(
                            &format!(
                                "stopping {} · quitting when every process is gone",
                                crate::screens::plural(n, "activity", "activities")
                            ),
                            Tone::Secondary,
                        );
                    }
                }
                return Outcome::Changed;
            }
            "confirm-one" => {
                if let ModalResult::Dialog {
                    action: Some(1), ..
                } = result
                    && let Some((item, args)) = self.pending_confirm.take()
                {
                    self.execute_item(&item, args);
                } else {
                    self.pending_confirm = None;
                    self.set_status("Cancelled · nothing was executed", Tone::Secondary);
                }
                return Outcome::Changed;
            }
            "gate2" => {
                if let ModalResult::Dialog {
                    action: Some(1), ..
                } = result
                {
                    self.gate2_confirmed();
                } else {
                    self.set_status("Cancelled · nothing was executed", Tone::Secondary);
                }
                return Outcome::Changed;
            }
            "alias" => {
                if let ModalResult::Dialog {
                    action: Some(1),
                    text: Some(t),
                } = result
                {
                    let alias = t.trim().to_owned();
                    if alias.is_empty() {
                        self.set_status("No alias set: the field was empty", Tone::Secondary);
                    } else {
                        self.world.memory.set_alias(&alias, &tag.key);
                        self.set_status(
                            &format!("Alias {alias} set · it beats learned ranking from now on"),
                            Tone::Secondary,
                        );
                    }
                }
                return Outcome::Changed;
            }
            "activities" => {
                if let ModalResult::Picked(i) = result {
                    // the row's identity, never its index: rows refresh
                    // while the picker is open
                    let key = match &entry.modal {
                        Modal::Picker(p) => p.items.get(i).map(|it| it.key.clone()),
                        _ => None,
                    };
                    match key.as_deref() {
                        Some(k) if k.starts_with("plan:") => {
                            let pid = k.trim_start_matches("plan:").to_owned();
                            if self.world.plan(&pid).is_some() {
                                self.open_plan_tab(&pid);
                            } else {
                                self.set_status("That plan is gone", Tone::Secondary);
                            }
                        }
                        Some(k) if self.world.activity(k).is_some() => {
                            let id = k.to_owned();
                            self.open_activity_tab(&id, true);
                        }
                        _ => self.set_status("That activity is gone", Tone::Secondary),
                    }
                }
                return Outcome::Changed;
            }
            "alternatives" => {
                if let ModalResult::MenuChosen(i) = result {
                    let action = self.menu_actions.get(i).cloned();
                    if let Some(a) = action {
                        self.menu_action(a, &tag.key);
                    }
                }
                return Outcome::Changed;
            }
            "close-tab" => {
                if let ModalResult::Dialog {
                    action: Some(1), ..
                } = result
                {
                    self.close_tab(self.active, true);
                }
                return Outcome::Changed;
            }
            _ => {}
        }
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let o = match self.tabs.get_mut(owner).and_then(|t| t.stack.last_mut()) {
            Some(s) => s.on_modal(&tag, result, &mut self.world, &mut cx),
            None => Outcome::Changed,
        };
        let reqs = std::mem::take(&mut cx.requests);
        o.or(self.apply_requests(reqs, owner)).or(Outcome::Changed)
    }

    fn modal_key(&mut self, key: Key) -> Outcome {
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
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
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Dialog { action, text });
                }
                out.or(Outcome::Consumed)
            }
            Modal::Picker(p) => {
                let (o, ev) = p.on_key(&key);
                if top.tag.kind == "files-jump" {
                    return self.files_jump_event(ev, o);
                }
                match ev {
                    Some(PickerEvent::Chosen(i)) | Some(PickerEvent::ChosenAlt(i)) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::Picked(i))
                    }
                    Some(PickerEvent::Cancelled) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::Cancelled)
                    }
                    _ => o.or(Outcome::Consumed),
                }
            }
            Modal::Menu(m) => {
                let (o, ev) = m.on_key(&key);
                match ev {
                    Some(MenuEvent::Chosen(i)) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::MenuChosen(i))
                    }
                    Some(MenuEvent::Dismissed) => {
                        let entry = self.pop_modal().unwrap();
                        self.deliver(entry, ModalResult::Cancelled)
                    }
                    None => o.or(Outcome::Consumed),
                }
            }
            Modal::Text(t) => {
                let o = t.on_key(&key);
                if t.closed {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Closed);
                }
                o.or(Outcome::Consumed)
            }
        }
    }

    /// The jump picker (HP04): the query re-supplies suggestions, Enter
    /// resolves an exact path over the selected suggestion, and a failed
    /// jump keeps the picker open with the error in its status row.
    fn files_jump_event(&mut self, ev: Option<PickerEvent>, o: Outcome) -> Outcome {
        let active = self.active;
        let query = match self.modals.last() {
            Some(ModalEntry {
                modal: Modal::Picker(p),
                ..
            }) => p.query.clone(),
            _ => return o.or(Outcome::Consumed),
        };
        match ev {
            Some(PickerEvent::QueryChanged) | Some(PickerEvent::Back) => {
                let rows = self
                    .tabs
                    .get_mut(active)
                    .and_then(|t| t.stack.last_mut())
                    .and_then(|s| s.as_files())
                    .map(|f| {
                        f.set_jump_error(None);
                        f.jump_rows(&query, &self.world)
                    });
                if let Some((items, status)) = rows
                    && let Some(ModalEntry {
                        modal: Modal::Picker(p),
                        ..
                    }) = self.modals.last_mut()
                {
                    p.refresh_items(items);
                    p.status = status;
                }
                Outcome::Changed
            }
            Some(PickerEvent::Chosen(_)) | Some(PickerEvent::ChosenAlt(_)) => {
                let selected = match self.modals.last() {
                    Some(ModalEntry {
                        modal: Modal::Picker(p),
                        ..
                    }) => p.current_key().map(str::to_owned),
                    _ => None,
                };
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                let outcome = self
                    .tabs
                    .get_mut(active)
                    .and_then(|t| t.stack.last_mut())
                    .and_then(|s| s.as_files())
                    .map(
                        |f| match f.jump_target(&query, selected.as_deref(), &self.world) {
                            None => Err("Type a path or choose a suggestion".to_owned()),
                            Some(target) => f
                                .perform_jump(&target, &self.world, &mut cx)
                                .map(|_| target),
                        },
                    );
                let requests = std::mem::take(&mut cx.requests);
                match outcome {
                    Some(Ok(target)) => {
                        self.pop_modal();
                        let short = self.world.location.short(&target);
                        self.set_status(&format!("Jumped to {short}"), Tone::Secondary);
                        self.apply_requests(requests, active);
                    }
                    Some(Err(e)) => {
                        let rows = self
                            .tabs
                            .get_mut(active)
                            .and_then(|t| t.stack.last_mut())
                            .and_then(|s| s.as_files())
                            .map(|f| {
                                f.set_jump_error(Some(e.clone()));
                                f.jump_rows(&query, &self.world)
                            });
                        if let Some((items, status)) = rows
                            && let Some(ModalEntry {
                                modal: Modal::Picker(p),
                                ..
                            }) = self.modals.last_mut()
                        {
                            p.refresh_items(items);
                            p.status = status;
                        }
                    }
                    None => {
                        self.pop_modal();
                    }
                }
                Outcome::Changed
            }
            Some(PickerEvent::Cancelled) => {
                self.pop_modal();
                Outcome::Changed
            }
            _ => o.or(Outcome::Consumed),
        }
    }

    fn modal_click(&mut self, id: WidgetId, pos: Position) -> Outcome {
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
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Dialog { action, text });
                }
                out.or(Outcome::Changed)
            }
            Modal::Picker(p) => {
                let ev = p.on_click(id);
                if top.tag.kind == "files-jump" {
                    return self.files_jump_event(ev, Outcome::Changed);
                }
                if let Some(PickerEvent::Chosen(i)) = ev {
                    let entry = self.pop_modal().unwrap();
                    return self.deliver(entry, ModalResult::Picked(i));
                }
                Outcome::Changed
            }
            Modal::Menu(m) => match m.on_click(id) {
                Some(MenuEvent::Chosen(i)) => {
                    let entry = self.pop_modal().unwrap();
                    self.deliver(entry, ModalResult::MenuChosen(i))
                }
                Some(MenuEvent::Dismissed) => {
                    let entry = self.pop_modal().unwrap();
                    self.deliver(entry, ModalResult::Cancelled)
                }
                None => Outcome::Changed,
            },
            Modal::Text(_) => Outcome::Changed,
        }
    }

    fn modal_outside_click(&mut self) -> Outcome {
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
        match &mut top.modal {
            Modal::Dialog(d) => {
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
                    return self.deliver(entry, ModalResult::Dialog { action, text: None });
                }
                out
            }
            Modal::Picker(_) | Modal::Menu(_) | Modal::Text(_) => {
                let entry = self.pop_modal().unwrap();
                self.deliver(entry, ModalResult::Cancelled)
            }
        }
    }

    // ------------------------------------------------------------- mouse

    fn on_mouse(&mut self, m: Mouse) -> Outcome {
        match m.kind {
            MouseKind::Move => {
                let was = self.hover;
                let suppressed = self.hover_suppressed;
                self.hover_suppressed = false;
                self.hover = self.hits.hit(m.pos);
                if self.menu.is_open() {
                    self.menu.on_hover(self.hover);
                }
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
                if !self.modals.is_empty() {
                    return Outcome::Consumed;
                }
                self.with_top(|s, w, _| s.on_drag(pressed, m.pos, w))
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
                    self.with_top(|s, w, _| s.on_press(id, m.pos, w));
                }
                Outcome::Changed
            }
            MouseKind::Up => {
                let hit = self.hits.hit(m.pos);
                let pressed = self.pressed.take();
                let Some(id) = hit else {
                    if !self.modals.is_empty() && pressed.is_none() {
                        return self.modal_outside_click();
                    }
                    if self.menu.is_open() {
                        self.menu.close();
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
                if self.menu.owns(id) {
                    let (o, ev) = self.menu.on_click(id);
                    return match ev {
                        Some(MenuBarEvent::Chosen(mi, ii)) => {
                            let label = self.menu.menus[mi][ii].label.clone();
                            self.run_menu(&label)
                        }
                        Some(MenuBarEvent::Brand) => {
                            self.open_about();
                            Outcome::Changed
                        }
                        _ => o.or(Outcome::Changed),
                    };
                }
                if self.menu.is_open() {
                    self.menu.close();
                    return Outcome::Changed;
                }
                if self.strip.owns(id) {
                    let (_, ev) = self.strip.on_click(id);
                    match ev {
                        Some(TabEvent::Activated(i)) => self.activate_tab(i),
                        Some(TabEvent::Close(i)) => {
                            self.active = i;
                            self.go(Go::CloseTab);
                        }
                        _ => {}
                    }
                    return Outcome::Changed;
                }
                if id == STATUS_RUNNING {
                    self.open_activities();
                    return Outcome::Changed;
                }
                if id == STATUS_PATH {
                    let p = self.world.location.cwd.clone();
                    self.copy(p);
                    return Outcome::Changed;
                }
                if id == STATUS_GIT {
                    self.go(Go::Run {
                        item: "git.status".into(),
                        args: vec![],
                    });
                    return Outcome::Changed;
                }
                let o = self.with_top(|s, w, cx| {
                    if double {
                        let d = s.on_double_click(id, m.pos, w, cx);
                        if d.consumed() {
                            return d;
                        }
                    }
                    s.on_click(id, m.pos, w, cx)
                });
                o.or(Outcome::Changed)
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
                self.with_top(|s, w, cx| s.on_secondary(id, m.pos, w, cx))
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
                        Modal::Text(t) => t.on_wheel(delta),
                        _ => Outcome::Consumed,
                    };
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                self.with_top(|s, w, _| s.on_wheel(id, delta, m.pos, w))
            }
        }
    }

    // ----------------------------------------------------------- requests

    fn apply_requests(&mut self, requests: Vec<Request>, _owner: usize) -> Outcome {
        let mut out = Outcome::Ignored;
        for r in requests {
            out = Outcome::Changed;
            match r {
                Request::Status(s) => self.set_status(&s, Tone::Secondary),
                Request::Error(s) => self.set_status(&s, Tone::Error),
                Request::Open(m, tag) => self.push_modal(*m, tag),
                Request::Go(g) => self.go(g),
                Request::Copy(s) => self.copy(s),
            }
        }
        out
    }

    fn copy(&mut self, s: String) {
        let shown = truncate(&s.replace('\n', " · "), 48);
        self.world.clipboard = Some(s);
        self.clipboard_gen += 1;
        self.set_status(&format!("Copied {shown}"), Tone::Secondary);
    }

    pub fn go(&mut self, g: Go) {
        match g {
            Go::Run { item, args } => {
                self.last_query = self
                    .tabs
                    .first_mut()
                    .and_then(|t| t.stack.first_mut())
                    .and_then(|s| s.as_finder())
                    .map(|f| f.query.clone())
                    .filter(|q| !q.trim().is_empty());
                self.run_item(&item, args)
            }
            Go::Alternatives { item, anchor } => self.open_alternatives(&item, anchor),
            Go::Push(page) => {
                self.activate_tab(0);
                self.push_page(page);
            }
            Go::Pop => self.pop_page(),
            Go::Here => self.activate_tab(0),
            Go::Scope(s) => {
                self.activate_tab(0);
                // scope lives on the root finder; domain pages pop back to it
                while self.tabs[0].stack.len() > 1 {
                    self.tabs[0].stack.pop();
                }
                if let Some(f) = self.finder_mut() {
                    f.set_scope(s);
                }
                self.enter_top();
                let loc = &self.world.location;
                let word = match s {
                    Scope::Here => format!("here · {}", loc.cwd_short()),
                    Scope::Parent => match &loc.workspace {
                        Some(p) => format!("parent · {}", loc.short(&p.root)),
                        None => "parent · no root above".to_owned(),
                    },
                    Scope::Children => format!(
                        "children · {}",
                        crate::screens::plural(loc.children.len(), "project", "projects")
                    ),
                    Scope::System => format!("system · {}", self.world.host.name),
                };
                self.set_status(&format!("Scope {word}"), Tone::Secondary);
            }
            Go::ConfirmPlan(id) => self.confirm_plan(&id),
            Go::Gate1Accepted(target) => self.open_gate2(target),
            Go::Trusted { config, then } => {
                let custom = self
                    .world
                    .custom_project
                    .as_ref()
                    .filter(|c| c.path == config)
                    .cloned();
                if let Some(cfg) = custom {
                    // the file is re-read at approval: an edit during review
                    // revokes the review instead of trusting unseen bytes
                    let now_text = match self.world.fs.get(&cfg.path).map(|n| &n.content) {
                        Some(crate::sim::fs::Content::Text(t)) => t.clone(),
                        _ => String::new(),
                    };
                    let now_digest = crate::domain::digest::sha256_hex(now_text.as_bytes());
                    if now_digest != cfg.digest {
                        self.pop_page();
                        self.reload_project_config();
                        self.set_status(
                            "The configuration changed during review · nothing was trusted or run · review again",
                            Tone::Error,
                        );
                        return;
                    }
                    let cwd = self
                        .find_item(&then)
                        .map(|i| i.scope.runs_in.clone())
                        .unwrap_or(self.world.location.cwd.clone());
                    match self.world.trust.approve(&cfg.digest, &cfg.path, &cwd) {
                        Ok(text) => {
                            self.world.persisted.trust = Some(text);
                            self.pop_page();
                            self.set_status(
                                &format!(
                                    "Trusted {} · this exact content at this path",
                                    self.world.location.short(&config)
                                ),
                                Tone::Secondary,
                            );
                            self.run_item(&then, vec![]);
                        }
                        Err(e) => {
                            self.pop_page();
                            self.set_status(&format!("{e} · nothing was run"), Tone::Error);
                        }
                    }
                    return;
                }
                if !self.world.trusted_now.contains(&config) {
                    self.world.trusted_now.push(config.clone());
                }
                self.pop_page();
                self.set_status(
                    &format!(
                        "Trusted {} · re-resolving the task",
                        self.world.location.short(&config)
                    ),
                    Tone::Secondary,
                );
                self.run_item(&then, vec![]);
            }
            Go::CleanupPlan(selected) => {
                let plan = crate::domain::fixtures::plan_cleanup_work(&self.world, &selected);
                self.world.plans.retain(|p| p.id != "cleanup-work");
                self.world.plans.push(plan);
                self.push_page(Page::PlanReview {
                    plan: "cleanup-work".into(),
                });
            }
            Go::Restart(id) => self.run_derived(&format!("activity.{id}.restart"), ""),
            Go::CloseTab => self.request_close_tab(),
            Go::Quit => {
                if self.quitting {
                    self.set_status(
                        "Already stopping · the shell leaves when every process is gone",
                        Tone::Secondary,
                    );
                } else if self.world.live_activities() > 0 {
                    self.open_quit_confirm();
                } else {
                    self.quit = true;
                }
            }
            Go::Reload => {
                self.reload_project_config();
                self.enter_top();
            }
            Go::Exec { label, argv } => {
                let scope = crate::domain::context::ScopeTag::here(&self.world.location.cwd);
                let script = self.world.script_for(None, &argv);
                let aid = self.world.start_activity(
                    argv,
                    script,
                    &label,
                    &label,
                    ActivityKind::Task,
                    scope,
                );
                self.open_activity_tab(&aid, true);
            }
            Go::Osc52(value) => self.copy_osc52(value),
            Go::Analyze(path) => {
                self.activate_tab(0);
                if !self.world.fs.exists(&path) {
                    self.set_status(&format!("{path}: no such file or directory"), Tone::Error);
                } else {
                    self.world.start_scan(&path);
                    self.push_page(Page::Disk { path });
                }
            }
        }
    }

    fn page_screen(&mut self, page: Page) -> Box<dyn Screen> {
        match page {
            Page::Finder { group } => Box::new(FinderPage::new(group)),
            Page::Disk { path } => Box::new(DiskPage::new(&path)),
            Page::PlanReview { plan } => {
                self.ensure_plan(&plan);
                Box::new(PlanReviewPage::new(&plan))
            }
            Page::Review(target) => Box::new(GatePage::new(target, &self.world)),
            Page::Trust { config, then } => Box::new(TrustPage::new(&config, &then)),
            Page::Args { item } => Box::new(ArgsPage::new(&item, &self.world)),
            Page::Snapshot { kind } => Box::new(SnapshotPage::new(&kind, &self.world)),
            Page::Files { path, query } => {
                Box::new(FilesPage::new(&path, query.as_deref(), &self.world))
            }
            Page::Find => Box::new(FilesPage::find(&self.world)),
            Page::DiskOverview => Box::new(DiskPage::overview()),
            Page::TopFiles => Box::new(DiskPage::top_files()),
            Page::Cleanup { category } => Box::new(CleanupPage::new(category.as_deref())),
            Page::CleanupGate { plan } => {
                Box::new(GatePage::new(GateTarget::Cleanup(plan), &self.world))
            }
            Page::Report { index } => {
                Box::new(SnapshotPage::new(&format!("report:{index}"), &self.world))
            }
            Page::Config { path } => {
                Box::new(SnapshotPage::new(&format!("config:{path}"), &self.world))
            }
        }
    }

    /// Re-parse the project configuration from the filesystem (an edit,
    /// a relocation, a restart).
    fn reload_project_config(&mut self) {
        let Some(cfg) = self.world.custom_project.clone() else {
            return;
        };
        let text = match self.world.fs.get(&cfg.path).map(|n| &n.content) {
            Some(crate::sim::fs::Content::Text(t)) => t.clone(),
            _ => String::new(),
        };
        let builtin = crate::domain::catalog::builtin_ids(&self.world);
        let builtin_refs: Vec<&str> = builtin.iter().map(String::as_str).collect();
        let reserved: Vec<String> = self
            .world
            .custom_global
            .as_ref()
            .map(|g| g.actions.iter().map(|a| a.id.clone()).collect())
            .unwrap_or_default();
        self.world.custom_project = Some(crate::domain::custom::parse_config(
            &cfg.path,
            cfg.origin,
            &text,
            &builtin_refs,
            &reserved,
        ));
    }

    pub(crate) fn push_page(&mut self, page: Page) {
        let screen = self.page_screen(page);
        self.tabs[0].stack.push(screen);
        self.active = 0;
        self.enter_top();
    }

    fn pop_page(&mut self) {
        if self.active == 0 && self.tabs[0].stack.len() > 1 {
            self.tabs[0].stack.pop();
            self.enter_top();
        } else if self.active != 0 {
            self.activate_tab(0);
        }
    }

    fn activate_tab(&mut self, i: usize) {
        if i >= self.tabs.len() {
            return;
        }
        self.active = i;
        self.strip.set_active(i);
        self.enter_top();
    }

    fn open_activity_tab(&mut self, id: &str, activate: bool) {
        if let Some(i) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::Activity(id.to_owned()))
        {
            if activate {
                self.activate_tab(i);
            }
            return;
        }
        self.tabs.push(TabEntry {
            kind: TabKind::Activity(id.to_owned()),
            stack: vec![Box::new(ActivityTab::new(id))],
        });
        if activate {
            let n = self.tabs.len() - 1;
            self.activate_tab(n);
        }
    }

    fn open_plan_tab(&mut self, id: &str) {
        if let Some(i) = self
            .tabs
            .iter()
            .position(|t| t.kind == TabKind::Plan(id.to_owned()))
        {
            self.activate_tab(i);
            return;
        }
        self.tabs.push(TabEntry {
            kind: TabKind::Plan(id.to_owned()),
            stack: vec![Box::new(PlanTab::new(id))],
        });
        let n = self.tabs.len() - 1;
        self.activate_tab(n);
    }

    fn request_close_tab(&mut self) {
        if self.active == 0 {
            self.set_status("Here cannot be closed · Ctrl+Q quits", Tone::Secondary);
            return;
        }
        let live = match &self.tabs[self.active].kind {
            TabKind::Activity(id) => self.world.activity(id).is_some_and(|a| a.state.live()),
            TabKind::Plan(id) => self
                .world
                .plan(id)
                .is_some_and(|p| p.phase == PlanPhase::Running),
            TabKind::Here => false,
        };
        if live {
            let name = self.tab_label(self.active);
            let d = Dialog::destructive(
                WidgetId::of("close-tab"),
                &format!("Stop {name}?"),
                "Closing the tab stops the work it holds. Detach instead to keep a monitor running.",
                "Stop and close",
            );
            self.push_modal(Modal::Dialog(d), ModalTag::new("close-tab"));
        } else {
            self.close_tab(self.active, false);
        }
    }

    fn close_tab(&mut self, i: usize, stop: bool) {
        if i == 0 || i >= self.tabs.len() {
            return;
        }
        match self.tabs[i].kind.clone() {
            TabKind::Activity(id) => {
                let tick = self.world.tick;
                if stop
                    && let Some(a) = self.world.activity_mut(&id)
                    && a.state.live()
                {
                    // the tab stays until the process group is gone: a
                    // stop is a request, not a result
                    a.stop(tick);
                    self.set_status(
                        "Stopping · the tab closes once the process is gone",
                        Tone::Secondary,
                    );
                    return;
                }
                if self.world.activity(&id).is_some_and(|a| a.state.live()) {
                    return;
                }
                self.world.activities.retain(|a| a.id != id);
            }
            TabKind::Plan(id) => {
                if stop && let Some(p) = self.world.plan_mut(&id) {
                    p.cancel_remaining();
                    p.phase = PlanPhase::Done;
                }
            }
            TabKind::Here => {}
        }
        self.tabs.remove(i);
        self.activate_tab(i.saturating_sub(1).min(self.tabs.len() - 1));
    }

    // ------------------------------------------------------- launch pipeline

    fn find_item(&self, id: &str) -> Option<Item> {
        self.world.items().into_iter().find(|i| i.id == id)
    }

    fn run_item(&mut self, id: &str, args: Vec<(String, String)>) {
        let Some(it) = self.find_item(id) else {
            self.set_status(&format!("{id} is no longer available here"), Tone::Error);
            return;
        };
        if let crate::domain::action::Freshness::Unavailable(why) = &it.freshness
            && !matches!(
                it.launch,
                Launch::Snapshot { .. }
                    | Launch::Files { .. }
                    | Launch::Insert
                    | Launch::Config { .. }
            )
        {
            self.set_status(&format!("{} is unavailable · {why}", it.label), Tone::Error);
            return;
        }
        // trust first: a contributed or untrusted definition is reviewed before it runs
        if let Confirmation::Trust { .. } = &it.confirmation {
            let config = self.config_path_for(&it);
            let trusted = match &it.trust_key {
                Some((path, digest)) => {
                    let cwd = it.scope.runs_in.clone();
                    self.world.trust.status(digest, path, &cwd)
                        == crate::domain::custom::TrustStatus::Trusted
                }
                None => self.world.trusted_now.contains(&config),
            };
            if !trusted {
                self.activate_tab(0);
                self.push_page(Page::Trust {
                    config,
                    then: it.id.clone(),
                });
                return;
            }
        }
        // arguments
        if !it.args.is_empty() && args.is_empty() {
            self.activate_tab(0);
            self.push_page(Page::Args {
                item: it.id.clone(),
            });
            return;
        }
        // plans are always reviewed on their own page
        if let Launch::Plan { plan } = &it.launch {
            self.remember_use(&it.id);
            self.activate_tab(0);
            self.push_page(Page::PlanReview { plan: plan.clone() });
            return;
        }
        match &it.confirmation {
            Confirmation::None | Confirmation::Trust { .. } => self.execute_item(&it.id, args),
            Confirmation::One => self.open_confirm_one(&it, args),
            Confirmation::TwoGate { .. } => {
                self.activate_tab(0);
                self.push_page(Page::Review(GateTarget::Item {
                    item: it.id.clone(),
                    args,
                }));
            }
        }
    }

    /// Record the use and the query that led to it, before the result is
    /// known (failed invocations count too). Nothing is learned while
    /// history is disabled.
    fn remember_use(&mut self, id: &str) {
        self.world.record_use(id);
        if let Some(q) = self.last_query.take()
            && !q.trim().is_empty()
        {
            let host = self.world.host.name.clone();
            let now = self.world.now_secs();
            self.world.memory.usage.learn_query(&q, id, &host, now);
        }
        self.persist_usage();
    }

    /// Save the usage store the way a production launch would: merge with
    /// what is on disk, then replace it. A save failure is shown, never
    /// fatal.
    fn persist_usage(&mut self) {
        let now = self.world.now_secs();
        let on_disk = self.world.persisted.frecency.clone();
        match self.world.memory.usage.save(on_disk.as_deref(), now) {
            Ok(text) => self.world.persisted.frecency = Some(text),
            Err(e) if self.world.memory.usage.enabled => {
                self.set_status(&format!("history not saved · {e}"), Tone::Error);
            }
            Err(_) => {}
        }
    }

    fn config_path_for(&self, it: &Item) -> String {
        if let Some((path, _)) = &it.trust_key {
            return path.clone();
        }
        self.world
            .mise
            .configs
            .iter()
            .find(|c| {
                c.tasks
                    .iter()
                    .any(|t| format!("mise.task.{}", t.namespaced) == it.id)
            })
            .map(|c| c.path.clone())
            .unwrap_or_else(|| format!("{}/mise.toml", it.scope.defined_at))
    }

    fn open_confirm_one(&mut self, it: &Item, args: Vec<(String, String)>) {
        let mut facts = crate::screens::finder::item_facts(it, &self.world, &self.theme);
        if !args.is_empty() {
            let a: Vec<String> = args.iter().map(|(k, v)| format!("{k} = {v}")).collect();
            facts.push(Prop::new("Arguments", a.join(" · ")));
        }
        facts.push(
            Prop::new("Confirmation", "one explicit confirmation · not remembered")
                .tone(Tone::Muted),
        );
        let code = it.commands.clone();
        let dangerous = matches!(it.risk, Risk::Destructive | Risk::Privileged);
        let confirm = if dangerous {
            Button::danger(WidgetId::of("confirm-one").sub("ok"), "Run")
        } else {
            Button::primary(WidgetId::of("confirm-one").sub("ok"), "Run")
        };
        let mut d = Dialog::facts(
            WidgetId::of("confirm-one"),
            &it.label,
            facts,
            code,
            None,
            confirm,
        );
        d.initial_focus = if dangerous {
            d.actions[0].id
        } else {
            d.actions[1].id
        };
        d.width = 74;
        self.pending_confirm = Some((it.id.clone(), args));
        self.push_modal(Modal::Dialog(d), ModalTag::new("confirm-one"));
    }

    fn ensure_plan(&mut self, id: &str) {
        if self.world.plan(id).is_some() {
            return;
        }
        let plan = match id {
            "docker-cleanup" => crate::domain::fixtures::plan_docker_cleanup(&self.world),
            "upgrade" => crate::domain::fixtures::plan_upgrade(&self.world),
            "git-pull-all" => crate::domain::fixtures::plan_git_pull_all(&self.world),
            "git-switch-primary" => crate::domain::fixtures::plan_git_switch_primary(&self.world),
            "upgrade-all" => crate::domain::fixtures::plan_upgrade_all(&self.world),
            _ => {
                let sel: Vec<String> = self
                    .world
                    .disk
                    .candidates
                    .iter()
                    .filter(|c| {
                        c.default_selected() && c.path.starts_with(&self.world.location.cwd)
                    })
                    .map(|c| c.path.clone())
                    .collect();
                crate::domain::fixtures::plan_cleanup_work(&self.world, &sel)
            }
        };
        self.world.plans.push(plan);
    }

    /// The plan review's Confirm: gate it when it has a phrase, otherwise one
    /// facts confirmation, then start it.
    fn confirm_plan(&mut self, id: &str) {
        let Some(p) = self.world.plan(id).cloned() else {
            return;
        };
        if p.included_count() == 0 {
            self.set_status("Nothing to run: every step is excluded", Tone::Secondary);
            return;
        }
        if p.phrase.is_some() {
            self.push_page(Page::Review(GateTarget::Plan(id.to_owned())));
        } else {
            let mut facts: Vec<Prop> = vec![
                Prop::new("Plan", format!("{} · {}", p.title, p.intent)).wrap(),
                Prop::new("Host", p.host.clone()).tone(
                    if p.host == self.world.host.name && self.world.host.role.sensitive() {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    },
                ),
                Prop::new(
                    "Steps",
                    format!(
                        "{} included · {} excluded · ~{}",
                        p.included_count(),
                        p.steps.len() - p.included_count(),
                        crate::screens::ticks_label(p.estimate_ticks())
                    ),
                ),
            ];
            for (k, v) in p.facts.iter().filter(|(k, _)| k != "Host") {
                facts.push(Prop::new(k, v.clone()).wrap());
            }
            let code: Vec<String> = p
                .steps
                .iter()
                .filter(|s| s.included)
                .flat_map(|s| s.commands.clone())
                .collect();
            // one risk level per action: a plan without a phrase is not
            // destructive, so its confirmation is the primary button
            let confirm = Button::primary(WidgetId::of("start-plan").sub("ok"), "Start plan");
            let mut d = Dialog::facts(
                WidgetId::of("start-plan"),
                &format!("Start {}?", p.title.to_lowercase()),
                facts,
                code,
                None,
                confirm,
            );
            d.initial_focus = d.actions[0].id;
            d.width = 76;
            self.pending_gate = Some(GateTarget::Plan(id.to_owned()));
            self.push_modal(Modal::Dialog(d), ModalTag::new("gate2"));
        }
    }

    /// Gate 2: the typed target-bound phrase over the review page.
    fn open_gate2(&mut self, target: GateTarget) {
        let (title, facts, code, phrase, confirm_label) = match &target {
            GateTarget::Plan(id) => {
                let Some(p) = self.world.plan(id).cloned() else {
                    return;
                };
                let mut facts = vec![
                    Prop::new("Action", p.intent.clone()).wrap(),
                    Prop::new("Host", p.host.clone()).tone(Tone::Normal),
                    Prop::new("Steps", format!("{} included", p.included_count())),
                ];
                for (k, v) in p.facts.iter().take(3) {
                    facts.push(Prop::new(k, v.clone()).wrap());
                }
                let code: Vec<String> = p
                    .steps
                    .iter()
                    .filter(|s| s.included)
                    .flat_map(|s| s.commands.clone())
                    .collect();
                (
                    p.title.clone(),
                    facts,
                    code,
                    p.gate_phrase().unwrap_or_default(),
                    "Execute".to_owned(),
                )
            }
            GateTarget::Item { item, .. } => {
                let Some(it) = self.find_item(item) else {
                    return;
                };
                let facts = crate::screens::finder::item_facts(&it, &self.world, &self.theme);
                (
                    it.label.clone(),
                    facts,
                    it.commands.clone(),
                    it.confirmation.phrase().unwrap_or_default(),
                    "Execute".to_owned(),
                )
            }
            GateTarget::Cleanup(plan) => {
                let facts = crate::screens::cleanup::plan_facts(plan, &self.world);
                let code: Vec<String> = plan.items.iter().map(|i| i.path.clone()).collect();
                (
                    format!(
                        "{} {} items",
                        if plan.dry_run {
                            "Dry run"
                        } else if plan.mode == crate::domain::cleanup::Mode::Permanent {
                            "Permanently delete"
                        } else {
                            "Trash"
                        },
                        plan.items.len()
                    ),
                    facts,
                    code,
                    plan.phrase(),
                    if plan.dry_run {
                        "Dry run".to_owned()
                    } else {
                        "Execute".to_owned()
                    },
                )
            }
        };
        let confirm = Button::danger(WidgetId::of("gate2").sub("ok"), &confirm_label);
        let mut d = Dialog::facts(
            WidgetId::of("gate2"),
            &format!("{title} · gate 2 of 2"),
            facts,
            code,
            Some(&phrase),
            confirm,
        );
        d.width = 78;
        self.pending_gate = Some(target);
        self.push_modal(Modal::Dialog(d), ModalTag::new("gate2"));
    }

    /// Gate 2 confirmed: re-resolve, then execute (CONCEPT §10 invariants).
    fn gate2_confirmed(&mut self) {
        let Some(target) = self.pending_gate.take() else {
            return;
        };
        match target {
            GateTarget::Plan(id) => {
                let drift = self.world.plan(&id).and_then(|p| {
                    if p.drift.is_some() && !p.drift_consumed {
                        p.drift.clone()
                    } else {
                        None
                    }
                });
                if let Some(d) = drift {
                    if let Some(p) = self.world.plan_mut(&id) {
                        p.drift_consumed = true;
                    }
                    self.set_status(
                        &format!("Plan changed · {d} · confirmation invalidated, review again"),
                        Tone::Error,
                    );
                    // the gate-1 page is still below: refresh it
                    self.enter_top();
                    return;
                }
                // leave the review pages
                while self.tabs[0].stack.len() > 1 {
                    self.tabs[0].stack.pop();
                }
                let tick = self.world.tick;
                if let Some(p) = self.world.plan_mut(&id) {
                    p.arm(tick);
                }
                let title = self
                    .world
                    .plan(&id)
                    .map(|p| p.title.clone())
                    .unwrap_or_default();
                self.open_plan_tab(&id);
                self.set_status(&format!("{title} started"), Tone::Secondary);
            }
            GateTarget::Item { item, args } => {
                while self.tabs[0].stack.len() > 1 {
                    self.tabs[0].stack.pop();
                }
                self.enter_top();
                self.execute_item(&item, args);
            }
            GateTarget::Cleanup(plan) => {
                // the gate revalidates: a plan whose fingerprint drifted
                // since review is refused before any effect
                let outcome = crate::screens::cleanup::commit(&plan, &mut self.world);
                match outcome {
                    Ok(index) => {
                        while self.tabs[0].stack.len() > 1 {
                            self.tabs[0].stack.pop();
                        }
                        self.push_page(Page::Report { index });
                        let summary = self.world.reports[index].summary();
                        self.set_status(
                            &summary,
                            if self.world.reports[index].incomplete() {
                                Tone::Error
                            } else {
                                Tone::Secondary
                            },
                        );
                    }
                    Err(e) => {
                        self.set_status(&e, Tone::Error);
                        self.enter_top();
                    }
                }
            }
        }
    }

    fn activity_name(it: &Item) -> String {
        if let Some(ns) = it.id.strip_prefix("mise.task.") {
            let ns = ns.trim_start_matches("//");
            return ns
                .replace(':', " ")
                .replace("apps/", "")
                .replace("services/", "")
                .replace("tools/", "");
        }
        let l = it.label.to_lowercase();
        for v in ["run ", "start ", "follow ", "open ", "connect to ", "show "] {
            if let Some(rest) = l.strip_prefix(v) {
                return rest.to_owned();
            }
        }
        l
    }

    fn execute_item(&mut self, id: &str, args: Vec<(String, String)>) {
        let Some(it) = self.find_item(id) else {
            return;
        };
        self.remember_use(id);
        match it.launch.clone() {
            Launch::Activity { script } => {
                let name = Self::activity_name(&it);
                let kind = if it.id == "docker.compose_logs" {
                    ActivityKind::Logs {
                        services: self
                            .world
                            .docker
                            .compose
                            .as_ref()
                            .map(|c| {
                                c.services
                                    .iter()
                                    .filter(|s| !matches!(s.as_str(), "db" | "redis"))
                                    .cloned()
                                    .collect()
                            })
                            .unwrap_or_default(),
                    }
                } else {
                    ActivityKind::Task
                };
                // launch-time arguments fill their placeholders in the typed
                // spec; they are never appended as free text
                let argv: Vec<crate::domain::exec::Command> = it
                    .exec
                    .iter()
                    .map(|c| {
                        let mut c = c.clone();
                        for a in &mut c.args {
                            for (k, v) in &args {
                                let ph = format!("<{k}>");
                                if a.contains(&ph) {
                                    *a = a.replace(&ph, v);
                                }
                            }
                        }
                        if c.program == "kill"
                            && let Some(sig) = args.iter().find(|(k, _)| k == "signal")
                        {
                            c.args[0] = format!("-{}", sig.1);
                        }
                        c
                    })
                    .collect();
                if argv.is_empty() && script.is_none() {
                    self.set_status(
                        &format!(
                            "{} has no executable specification in this preview",
                            it.label
                        ),
                        Tone::Error,
                    );
                    return;
                }
                let script = self.world.script_for(script.as_deref(), &argv);
                let aid = self.world.start_activity(
                    argv,
                    script,
                    &name,
                    &it.label,
                    kind,
                    it.scope.clone(),
                );
                if let Some(a) = self.world.activity_mut(&aid) {
                    if !args.is_empty() {
                        // launch arguments are insights; secrets never are
                        a.insights = args
                            .iter()
                            .filter(|(k, _)| !it.args.iter().any(|s| &s.name == k && s.secret))
                            .cloned()
                            .collect();
                    }
                    a.effect = it.effect.clone();
                    if it.id == "system.kill"
                        && let Some(pid) = args
                            .iter()
                            .find(|(k, _)| k == "pid")
                            .and_then(|(_, v)| v.trim().parse::<u32>().ok())
                    {
                        a.effect = Some(crate::domain::effect::Effect::KillProcess(pid));
                    }
                }
                self.open_activity_tab(&aid, true);
                self.set_status(
                    &format!(
                        "Started {name} in {}",
                        self.world.location.short(&it.scope.runs_in)
                    ),
                    Tone::Secondary,
                );
            }
            Launch::Batch => {
                let members: Vec<_> = it
                    .batch
                    .iter()
                    .map(|(name, argv, effect, scope)| {
                        let script = self.world.script_for(None, argv);
                        (
                            name.clone(),
                            argv.clone(),
                            script,
                            effect.clone(),
                            scope.clone(),
                        )
                    })
                    .collect();
                let ids =
                    self.world
                        .start_batch(&it.id, &it.label, it.batch_mode, members, &it.label);
                if let Some(first) = ids.first() {
                    let first = first.clone();
                    self.open_activity_tab(&first, true);
                }
                self.set_status(
                    &format!(
                        "{} · {} started {}",
                        it.label,
                        crate::screens::plural(ids.len(), "activity", "activities"),
                        match it.batch_mode {
                            crate::sim::world::BatchMode::Parallel =>
                                "in parallel · Ctrl+G lists them",
                            crate::sim::world::BatchMode::Sequential =>
                                "one after another · Ctrl+G lists them",
                        }
                    ),
                    Tone::Secondary,
                );
            }
            Launch::Handoff { tool } => {
                let script = match tool.as_str() {
                    "btm" => "monitor:btm".to_owned(),
                    "pg_activity" => "monitor:pg_activity".to_owned(),
                    t if t.starts_with("ssh ") => format!("ssh:{}", t.trim_start_matches("ssh ")),
                    t => format!("handoff:{t}"),
                };
                let kind = if tool.starts_with("ssh ") {
                    ActivityKind::Ssh {
                        alias: tool.trim_start_matches("ssh ").to_owned(),
                    }
                } else {
                    ActivityKind::Monitor { tool: tool.clone() }
                };
                let argv = it.exec.clone();
                let script = self.world.script_for(Some(&script), &argv);
                let aid = self.world.start_activity(
                    argv,
                    script,
                    &tool,
                    &it.label,
                    kind,
                    it.scope.clone(),
                );
                self.open_activity_tab(&aid, true);
                self.set_status(
                    &format!("{tool} opened as an activity · Enter attaches, Ctrl+] detaches"),
                    Tone::Secondary,
                );
            }
            Launch::Disk { path } => {
                let path = if path == "<path>" {
                    args.iter()
                        .find(|(k, _)| k == "path")
                        .map(|(_, v)| v.trim().to_owned())
                        .unwrap_or_default()
                } else {
                    path
                };
                self.activate_tab(0);
                if path.is_empty() {
                    self.push_page(Page::DiskOverview);
                    return;
                }
                if !path.starts_with('/') {
                    self.set_status(&format!("{path}: the path must be absolute"), Tone::Error);
                    return;
                }
                if !self.world.fs.exists(&path) {
                    self.set_status(&format!("{path}: no such file or directory"), Tone::Error);
                    return;
                }
                self.world.start_scan(&path);
                self.push_page(Page::Disk { path });
            }
            Launch::Files { path } => {
                self.activate_tab(0);
                self.push_page(Page::Files { path, query: None });
            }
            Launch::Find => {
                self.activate_tab(0);
                self.push_page(Page::Find);
            }
            Launch::Cleanup { category } => {
                self.activate_tab(0);
                self.push_page(Page::Cleanup { category });
            }
            Launch::TopFiles => {
                self.activate_tab(0);
                self.push_page(Page::TopFiles);
            }
            Launch::Config { path } => {
                self.activate_tab(0);
                self.push_page(Page::Config { path });
            }
            Launch::Copy { value } => self.copy_osc52(value),
            Launch::Group(g) => {
                self.activate_tab(0);
                self.push_page(Page::Finder { group: Some(g) });
            }
            Launch::Scope(s) => self.go(Go::Scope(s)),
            Launch::Snapshot { snapshot } => {
                self.activate_tab(0);
                let kind = if snapshot == "port" {
                    let port = args
                        .iter()
                        .find(|(k, _)| k == "port")
                        .map(|(_, v)| v.clone())
                        .unwrap_or("5173".into());
                    format!("port:{port}")
                } else {
                    snapshot
                };
                self.push_page(Page::Snapshot { kind });
            }
            Launch::OpenActivity { id } => self.open_activity_tab(&id, true),
            Launch::Plan { plan } => {
                self.activate_tab(0);
                self.push_page(Page::PlanReview { plan });
            }
            Launch::Insert => {
                let cmd = it.commands.first().cloned().unwrap_or_default();
                self.set_status(
                    &format!(
                        "Inserted into the shell: {} · holla returns to the prompt",
                        truncate(&cmd, 40)
                    ),
                    Tone::Secondary,
                );
            }
        }
    }

    /// Copy through the terminal's OSC 52 channel: the result names the
    /// actual destination and never claims success the terminal cannot
    /// deliver (I-F11).
    pub fn copy_osc52(&mut self, value: String) {
        let p = &self.world.platform;
        if !p.osc52 {
            self.set_status(
                "Not copied · this terminal does not accept OSC 52 · no system clipboard is wired",
                Tone::Error,
            );
            return;
        }
        let encoded = value.len().div_ceil(3) * 4;
        if encoded > p.osc52_limit {
            self.set_status(
                &format!("Not copied · {encoded} encoded bytes exceed the terminal's {} byte OSC 52 limit", p.osc52_limit),
                Tone::Error,
            );
            return;
        }
        let shown = truncate(&value.replace('\n', " · "), 40);
        self.world.clipboard = Some(value);
        self.clipboard_gen += 1;
        self.set_status(
            &format!("Copied to the terminal clipboard (OSC 52, flushed) · {shown}"),
            Tone::Secondary,
        );
    }

    // --------------------------------------------------------- alternatives

    fn open_alternatives(&mut self, id: &str, anchor: Rect) {
        let Some(it) = self.find_item(id) else {
            return;
        };
        let mut items = vec![];
        let mut actions = vec![];
        let primary = match (&it.launch, &it.confirmation) {
            (_, Confirmation::TwoGate { .. }) => "Review…",
            (Launch::Plan { .. }, _) => "Review the plan",
            (
                Launch::Disk { .. } | Launch::Group(_) | Launch::Snapshot { .. } | Launch::Scope(_),
                _,
            ) => "Open",
            (_, Confirmation::One) => "Run after confirmation…",
            _ => "Run now",
        };
        let mut m = MenuItem::new(primary).shortcut("Enter");
        if matches!(it.risk, Risk::Destructive | Risk::Privileged) {
            m = m.danger();
        }
        items.push(m);
        actions.push(MenuAction::Run);
        if !it.commands.is_empty() {
            items.push(MenuItem::new("Copy command").shortcut("y"));
            actions.push(MenuAction::Copy);
            items.push(MenuItem::new("Insert into shell"));
            actions.push(MenuAction::Insert);
        }
        if !it.args.is_empty() {
            items.push(MenuItem::new("Arguments…"));
            actions.push(MenuAction::Args);
        }
        if let Some(last) = items.last_mut() {
            last.separator_after = true;
        }
        for a in &it.alternatives {
            items.push(MenuItem::new(a.label.clone()));
            actions.push(MenuAction::Alternative(a.item.clone()));
        }
        if !it.alternatives.is_empty()
            && let Some(last) = items.last_mut()
        {
            last.separator_after = true;
        }
        items.push(MenuItem::new(if it.pinned {
            "Unpin here"
        } else {
            "Pin here"
        }));
        actions.push(MenuAction::Pin);
        items.push(MenuItem::new(match self.world.memory.alias_for(&it.id) {
            Some(a) => format!("Change alias ({a})…"),
            None => "Set alias…".into(),
        }));
        actions.push(MenuAction::Alias);
        items.push(MenuItem::new(if it.hidden {
            "Restore here"
        } else {
            "Hide here"
        }));
        actions.push(MenuAction::Hide);
        items.push(MenuItem::new("Reset ranking").separator());
        actions.push(MenuAction::Reset);
        items.push(MenuItem::new("Why is this here?"));
        actions.push(MenuAction::Why);
        let menu = ContextMenu::new(WidgetId::of("alternatives"), items)
            .anchor(anchor, Placement::Below)
            .title(&it.label);
        self.menu_actions = actions;
        self.menu_item = Some(it.id.clone());
        self.push_modal(
            Modal::Menu(menu),
            ModalTag::new("alternatives").key(it.id.clone()),
        );
    }

    /// Alternatives that derive from another row rather than being rows of
    /// their own: dry runs, fetches, per-container operations, an SSH
    /// resolution, activity control, a volume analysis, an alias prompt.
    fn run_derived(&mut self, alt: &str, origin: &str) {
        let tick = self.world.tick;
        let scope = self
            .find_item(origin)
            .map(|i| i.scope.clone())
            .unwrap_or_else(|| crate::domain::context::ScopeTag::here(&self.world.location.cwd));
        if let Some(ns) = alt
            .strip_prefix("mise.task.")
            .and_then(|s| s.strip_suffix(".dry"))
        {
            let argv = vec![crate::domain::exec::Command::argv(
                "mise",
                &["run", "--dry-run", ns],
                &scope.runs_in,
                &self.world.host.name,
            )];
            let script = self.world.script_for(None, &argv);
            let aid = self.world.start_activity(
                argv,
                script,
                &format!("{} · dry run", ns.trim_start_matches("//")),
                "Dry run",
                ActivityKind::Task,
                scope,
            );
            self.open_activity_tab(&aid, true);
            self.set_status("Dry run started · nothing executes", Tone::Secondary);
            return;
        }
        if let Some(rest) = alt.strip_prefix("ssh.")
            && let Some(alias) = rest.strip_suffix(".resolve")
        {
            self.activate_tab(0);
            self.push_page(Page::Snapshot {
                kind: format!("ssh:{alias}"),
            });
            return;
        }
        if let Some(rest) = alt.strip_prefix("activity.") {
            if let Some(aid) = rest.strip_suffix(".stop") {
                if let Some(a) = self.world.activity_mut(aid) {
                    a.stop(tick);
                }
                self.set_status(
                    "Stop requested · output kept · the state settles once the process is gone",
                    Tone::Secondary,
                );
                return;
            }
            if let Some(aid) = rest.strip_suffix(".restart") {
                if let Some(a) = self.world.activity_mut(aid) {
                    if a.state.live() {
                        self.set_status("Still running · stop it first", Tone::Secondary);
                        return;
                    }
                    a.restart(tick);
                    a.advance(tick);
                }
                let aid = aid.to_owned();
                self.open_activity_tab(&aid, true);
                self.set_status("Restarted", Tone::Secondary);
                return;
            }
        }
        if alt.ends_with(".alias") {
            self.menu_action(MenuAction::Alias, origin);
            return;
        }
        self.set_status(
            &format!("{alt}: not available in the preview"),
            Tone::Secondary,
        );
    }

    fn menu_action(&mut self, a: MenuAction, id: &str) {
        let cwd = self.world.location.cwd.clone();
        match a {
            MenuAction::Run => self.run_item(id, vec![]),
            MenuAction::Copy => {
                let cmd = self
                    .find_item(id)
                    .map(|i| i.commands.join("\n"))
                    .unwrap_or_default();
                self.copy(cmd);
            }
            MenuAction::Insert => {
                let cmd = self
                    .find_item(id)
                    .and_then(|i| i.commands.first().cloned())
                    .unwrap_or_default();
                self.set_status(
                    &format!(
                        "Inserted into the shell: {} · holla returns to the prompt",
                        truncate(&cmd, 40)
                    ),
                    Tone::Secondary,
                );
            }
            MenuAction::Args => {
                self.activate_tab(0);
                self.push_page(Page::Args {
                    item: id.to_owned(),
                });
            }
            MenuAction::Alternative(alt) => {
                if self.find_item(&alt).is_some() {
                    self.run_item(&alt, vec![]);
                } else {
                    self.run_derived(&alt, id);
                }
            }
            MenuAction::Pin => {
                let on = self.world.memory.toggle_pin(&cwd, id);
                self.set_status(
                    if on {
                        "Pinned here · it stays near the top in this folder"
                    } else {
                        "Unpinned"
                    },
                    Tone::Secondary,
                );
                self.enter_top();
            }
            MenuAction::Alias => {
                let existing = self.world.memory.alias_for(id).unwrap_or("").to_owned();
                let input = TextInput::new(WidgetId::of("alias").sub("field"), "Alias")
                    .value(&existing)
                    .placeholder("gp · du · dc")
                    .help("exact match beats every learned signal · never drifts");
                let d = Dialog::prompt(WidgetId::of("alias"), "Set an alias", input, "Save");
                self.push_modal(Modal::Dialog(d), ModalTag::new("alias").key(id.to_owned()));
            }
            MenuAction::Hide => {
                let hidden = self
                    .world
                    .memory
                    .hidden
                    .iter()
                    .any(|(p, i)| p == &cwd && i == id);
                if hidden {
                    self.world.memory.restore(&cwd, id);
                    self.set_status("Restored here", Tone::Secondary);
                } else {
                    self.world.memory.hide(&cwd, id);
                    self.set_status(
                        "Hidden here · still reachable by search · Restore from the same menu",
                        Tone::Secondary,
                    );
                }
                self.enter_top();
            }
            MenuAction::Reset => {
                self.world.memory.reset(id);
                self.set_status(
                    "Ranking reset for this row · pins and aliases kept",
                    Tone::Secondary,
                );
                self.enter_top();
            }
            MenuAction::Why => {
                if let Some(it) = self.find_item(id) {
                    let cwd_short = self.world.location.cwd_short();
                    let rows = crate::domain::ranking::explain(&it, &cwd_short);
                    let mut lines: Vec<TextLine> =
                        rows.iter().map(|(k, v)| TextLine::kv(k, v)).collect();
                    lines.push(TextLine::blank());
                    lines.push(TextLine::text("Ranking order: alias › pin › live state › context › query › used here › used anywhere › default. Risk changes the gate, never the rank.", Tone::Muted));
                    let m = TextModal::new(
                        WidgetId::of("why"),
                        &format!("Why is “{}” here?", it.label),
                        lines,
                    )
                    .width(74);
                    self.push_modal(Modal::Text(m), ModalTag::new("why"));
                }
            }
        }
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
                let pf = self.top().and_then(|s| s.primary_focus());
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
                (&*format!(" {BRAND_MARK} "), Lockup::style(&t)),
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
        self.draw_frame(area, buf, ctx);
        if let Some(mut entry) = self.modals.pop() {
            match &mut entry.modal {
                Modal::Dialog(d) => d.render(area, buf, ctx),
                Modal::Picker(p) => p.render(area, buf, ctx, ""),
                Modal::Menu(m) => {
                    ctx.begin_modal();
                    m.render(area, buf, ctx);
                    ctx.ring.register(m.id);
                }
                Modal::Text(m) => m.render(area, buf, ctx),
            }
            self.modals.push(entry);
            let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
            self.draw_footer(footer, buf, true);
        }
    }

    fn crumb(&self) -> String {
        let here = &self.tabs[0];
        let parts: Vec<String> = here.stack.iter().map(|s| s.crumb(&self.world)).collect();
        parts.join(" › ")
    }

    fn tab_label(&self, i: usize) -> String {
        match &self.tabs[i].kind {
            TabKind::Here => self.crumb(),
            TabKind::Activity(id) => self
                .world
                .activity(id)
                .map(|a| a.name.clone())
                .unwrap_or("activity".into()),
            TabKind::Plan(id) => self
                .world
                .plan(id)
                .map(|p| p.title.clone())
                .unwrap_or("plan".into()),
        }
    }

    fn strip_items(&self) -> Vec<TabItem> {
        let mut v = vec![];
        for (i, tab) in self.tabs.iter().enumerate() {
            let label = self.tab_label(i);
            let label = if i == 0 {
                crumb_fit(&label, 48)
            } else {
                truncate(&label, 24)
            };
            let mut item = TabItem::new(&label);
            match &tab.kind {
                TabKind::Here => {}
                TabKind::Activity(id) => {
                    item = item.closable();
                    if let Some(a) = self.world.activity(id) {
                        match a.state {
                            ActivityState::Running | ActivityState::Cancelling => item.busy = true,
                            ActivityState::Queued => item = item.suffix("…"),
                            ActivityState::Failed => item.error = true,
                            ActivityState::Succeeded => item = item.suffix("✓"),
                            ActivityState::Stopped | ActivityState::Detached => {
                                item = item.suffix("○")
                            }
                        }
                    }
                }
                TabKind::Plan(id) => {
                    item = item.closable();
                    if let Some(p) = self.world.plan(id) {
                        match p.phase {
                            PlanPhase::Running => item.busy = true,
                            PlanPhase::Done => {
                                if p.outcome().1 > 0 {
                                    item.error = true;
                                } else {
                                    item = item.suffix("✓");
                                }
                            }
                            PlanPhase::Review => {}
                        }
                    }
                }
            }
            v.push(item);
        }
        v
    }

    fn draw_frame(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let header = Rect::new(area.x, area.y, area.width, 1);
        let strip = Rect::new(area.x + 1, area.y + 2, area.width.saturating_sub(2), 2);
        let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
        let status = Rect::new(area.x, area.bottom() - 2, area.width, 1);
        // one blank row separates the body from the status bar (shell rhythm)
        let body = Rect::new(
            area.x + 1,
            area.y + 4,
            area.width.saturating_sub(2),
            area.height.saturating_sub(7),
        );
        self.draw_menu(header, buf, ctx);
        // tab strip
        let items = self.strip_items();
        let first = self.strip.first;
        self.strip = Tabs::with_items(TABS, items);
        self.strip.first = first;
        self.strip.set_active(self.active);
        self.strip.render(strip, buf, ctx, t.canvas);
        // body
        let active = self.active;
        if let Some(s) = self.tabs.get_mut(active).and_then(|t| t.stack.last_mut()) {
            s.render(body, buf, ctx, &self.world);
        }
        self.draw_status(status, buf, ctx);
        self.draw_footer(footer, buf, false);
        self.menu.render_open(area, buf, ctx);
    }

    /// Row zero: the menu bar, then the project crumb and the host identity
    /// right-aligned in what is left of the row.
    fn draw_menu(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        self.menu.render(area, buf, ctx, t.canvas);
        let used = self
            .menu
            .areas
            .iter()
            .map(|r| r.right())
            .max()
            .unwrap_or(area.x)
            .max(self.menu.brand_area.right());
        let rest = Rect::new(used + 2, area.y, area.right().saturating_sub(used + 2), 1);
        let w = &self.world;
        let mut right = vec![];
        let project = match (&w.location.workspace, &w.location.project) {
            (Some(ws), Some(p)) => format!("{} › {} · {}", ws.name, p.name, p.kind.label()),
            (None, Some(p)) => format!("{} · {}", p.name, p.kind.label()),
            _ => "no project".into(),
        };
        let host = match w.host.role {
            HostRole::Production => format!("◆ {} · production", w.host.name),
            r => format!("{} · {}", w.host.name, r.label()),
        };
        // the identity is the constant frame and the path the variable
        // part: the crumb truncates to what is left rather than dropping
        let budget = (rest.width as usize).saturating_sub(width(&host) + 6);
        if budget >= 12 {
            right
                .push(Segment::new(truncate_middle(&project, budget), Tone::Secondary).priority(6));
        }
        // the `◆` glyph and the word are the identity; no safety tone at rest
        right.push(Segment::new(host, Tone::Normal).bold().priority(9));
        segments::render(rest, buf, ctx, &[], &right, t.canvas);
    }

    fn draw_status(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let w = &self.world;
        let mut bar = StatusBar::new();
        bar.left.push(
            StatusItem::new(truncate_middle(&w.location.cwd_short(), 44), Tone::Normal)
                .strong()
                .clickable(STATUS_PATH)
                .priority(10),
        );
        if let Some(g) = w.git_here() {
            // the branch is an identity, the state after it is the fact that
            // earns the warning tone; the state survives the branch name
            let state = g.state_summary();
            bar.left.push(
                StatusItem::new(truncate_middle(&g.head_label(), 24), Tone::Muted)
                    .clickable(STATUS_GIT)
                    .priority(if state.is_empty() { 5 } else { 4 }),
            );
            if !state.is_empty() {
                bar.left.push(
                    StatusItem::new(state, Tone::Warning)
                        .clickable(STATUS_GIT)
                        .priority(5),
                );
            }
        }
        let bits = self.top().map(|s| s.status(w)).unwrap_or_default();
        if let Some(c) = bits.center {
            bar.center.push(c);
        }
        for r in bits.right {
            bar.right.push(r);
        }
        if self.attached() {
            bar.right.push(
                StatusItem::new("attached", Tone::Secondary)
                    .chip()
                    .priority(8),
            );
        }
        let live = w.live_activities();
        if live > 0 {
            bar.right.push(
                StatusItem::new(
                    crate::screens::plural(live, "running", "running"),
                    Tone::Secondary,
                )
                .busy()
                .chip()
                .clickable(STATUS_RUNNING)
                .priority(6),
            );
        }
        if let Some(fs) = w.disk.root_fs()
            && fs.pct() >= 85
        {
            bar.right.push(
                StatusItem::new(fs.mount.clone(), Tone::Muted)
                    .meter(Some(fs.pct()), MeterTone::Normal)
                    .priority(4),
            );
        }
        bar.right.push(
            StatusItem::new(
                format!(
                    "{} · {}×{}",
                    self.theme.level.label(),
                    self.size.0,
                    self.size.1
                ),
                Tone::Faint,
            )
            .priority(1),
        );
        bar.render(area, buf, ctx);
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
                    v.push(hint("Enter", "Switch"));
                    v.push(hint("Esc", "Cancel"));
                    v
                }
                Some(Modal::Dialog(d)) => {
                    if d.is_editing() {
                        vec![
                            hint("Type", "Phrase"),
                            hint("Enter", "Next"),
                            hint("Esc", "Cancel"),
                        ]
                    } else if matches!(d.body, DialogBody::Facts { .. }) {
                        vec![
                            hint("← →", "Choose"),
                            hint("Enter", "Confirm"),
                            hint("Esc", "Cancel"),
                        ]
                    } else if matches!(d.body, DialogBody::Input(_)) {
                        vec![
                            hint("Enter", "Edit / Save"),
                            hint("Tab", "Next"),
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
                Some(Modal::Menu(_)) => vec![
                    hint("↑↓", "Move"),
                    hint("Enter", "Choose"),
                    hint("Esc", "Close"),
                ],
                Some(Modal::Text(_)) => vec![hint("↑↓", "Scroll"), hint("Esc", "Close")],
                None => vec![],
            }
        } else {
            let focus = self.focus.current();
            if focus == Some(TABS) {
                vec![
                    hint("← →", "Switch tab"),
                    hint("1–9", "Jump"),
                    hint("x", "Close"),
                    hint("Enter", "Into the tab"),
                    hint("Esc", "Back"),
                ]
            } else {
                self.top()
                    .map(|s| s.hints(focus, &self.world))
                    .unwrap_or_default()
            }
        };
        let editing =
            self.modals.last().is_some_and(|m| match &m.modal {
                Modal::Dialog(d) => d.is_editing(),
                _ => false,
            }) || (!modal && self.top().is_some_and(|s| s.is_editing()) && self.active == 0);
        let badge = if editing {
            Some(("EDIT", BadgeKind::Edit))
        } else {
            None
        };
        // an error status carries a `! ` prefix: leave room for it
        let status = self
            .status
            .as_ref()
            .map(|(s, tone, _)| (truncate(s, area.width.saturating_sub(8) as usize), *tone));
        let modal_layer = modal.then(|| HintLayer::new(hints.clone()));
        let menu_layer = (!modal && self.menu.is_open()).then(|| {
            HintLayer::new(vec![
                hint("← →", "Menu"),
                hint("↑↓", "Move"),
                hint("Enter", "Choose"),
                hint("Esc", "Close"),
            ])
        });
        let screen_layer = (!modal && !hints.is_empty()).then(|| HintLayer::new(hints.clone()));
        let fallback = HintLayer::new(vec![hint("F1", "Help"), hint("Esc", "Back")]);
        let mut layer = HintBar::resolve(&[modal_layer, menu_layer, screen_layer, Some(fallback)]);
        layer.badge = badge;
        layer.status = status;
        layer.centered = true;
        HintBar::render(area, buf, &t, &layer);
    }
}

/// Fit a ` › ` breadcrumb into `max` cells by dropping middle crumbs first
/// (`Here › … › Review`), then truncating the last crumb in the middle.
fn crumb_fit(label: &str, max: usize) -> String {
    use junie_tui::ui::text::width;
    if width(label) <= max {
        return label.to_owned();
    }
    let parts: Vec<&str> = label.split(" › ").collect();
    if parts.len() >= 3 {
        let first = parts[0];
        let last = parts[parts.len() - 1];
        let short = format!("{first} › … › {last}");
        if width(&short) <= max {
            return short;
        }
        let room = max.saturating_sub(width(first) + 8);
        return format!("{first} › … › {}", truncate_middle(last, room.max(6)));
    }
    truncate_middle(label, max)
}
