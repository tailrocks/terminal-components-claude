//! Application shell: chrome (menu bar + right-hand segments), the modal
//! stack, focus/hover plumbing, the hint bar and the too-small notice.
//! Routes are surfaces; P0 ships Home only, the rest land per phase.

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::ui::text::{truncate, truncate_middle, width};
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::menu::{MenuBar, MenuBarEvent, MenuItem};
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::{self, Segment};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::scenario::{Motion, Scenario};
use crate::screens::activity::{self, ActivityScreen};
use crate::screens::home::HomeScreen;
use crate::screens::plan::PlanScreen;
use crate::screens::{Cx, Go, Modal, ModalResult, ModalTag, Request, Screen};
use crate::sim::world::{Msg, World};

pub const MIN_WIDTH: u16 = 72;
pub const MIN_HEIGHT: u16 = 20;

/// The canonical product mark; every brand lockup renders exactly this.
pub const BRAND_MARK: &str = "holla❯";
const STRIP_HELP: WidgetId = WidgetId::of("strip.help");
const HOST_MENU: WidgetId = WidgetId::of("host.menu");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Home,
    Plan,
    Activity,
}

struct ModalEntry {
    modal: Modal,
    tag: ModalTag,
    owner: Route,
    saved_focus: Option<WidgetId>,
}

pub struct App {
    pub theme: Theme,
    /// Kept for capture logs and later per-scenario routing.
    #[allow(dead_code)]
    pub scenario: Scenario,
    pub motion: Motion,
    pub world: World,
    pub route: Route,
    pub home: HomeScreen,
    pub plan: Option<PlanScreen>,
    pub activity: ActivityScreen,
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
    host_menu: MenuBar,
    last_click: Option<(WidgetId, i64)>,
    pub clipboard_gen: u32,
}

impl App {
    /// Pure fixture entry point: one coherent world per scenario.
    pub fn for_scenario(scenario: Scenario, motion: Motion, frame: u64, theme: Theme) -> Self {
        let mut world = crate::domain::fixtures::world_for(scenario);
        world.clock.running = motion != Motion::Paused;
        world.seek(frame);
        Self {
            theme,
            scenario,
            motion,
            world,
            route: Route::Home,
            home: HomeScreen::default(),
            plan: None,
            activity: ActivityScreen::default(),
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
            host_menu: Self::build_host_menu(),
            last_click: None,
            clipboard_gen: 0,
        }
    }

    pub fn set_status(&mut self, s: &str, tone: Tone) {
        self.status = Some((s.to_owned(), tone, self.world.now_ms() + 5_000));
    }

    fn animating(&self) -> bool {
        self.flash.is_some() || self.world.discovering()
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
        let interval = if self.animating() { 80 } else { 200 };
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
        // screen tick
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let o = match self.route {
            Route::Home => self.home.on_tick(&mut self.world, &mut cx),
            Route::Plan => match &mut self.plan {
                Some(p) => p.on_tick(&mut self.world, &mut cx),
                None => Outcome::Ignored,
            },
            Route::Activity => self.activity.on_tick(&mut self.world, &mut cx),
        };
        out = out.or(o);
        let reqs = std::mem::take(&mut cx.requests);
        out = out.or(self.apply_requests(reqs, self.route));
        for m in msgs {
            out = out.or(self.dispatch_msg(m));
        }
        out
    }

    fn dispatch_msg(&mut self, m: Msg) -> Outcome {
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let o = match self.route {
            Route::Home => self.home.on_msg(&m, &mut self.world, &mut cx),
            Route::Plan => match &mut self.plan {
                Some(p) => p.on_msg(&m, &mut self.world, &mut cx),
                None => Outcome::Ignored,
            },
            Route::Activity => self.activity.on_msg(&m, &mut self.world, &mut cx),
        };
        let reqs = std::mem::take(&mut cx.requests);
        o.or(self.apply_requests(reqs, self.route))
    }

    fn on_paste(&mut self, text: &str) -> Outcome {
        if let Some(top) = self.modals.last_mut() {
            return match &mut top.modal {
                Modal::Dialog(d) => d.on_paste(text),
                Modal::Custom(c) => {
                    let _ = c;
                    Outcome::Consumed
                }
            };
        }
        match self.route {
            Route::Home => self.home.on_paste(text, &mut self.world),
            Route::Plan | Route::Activity => Outcome::Ignored,
        }
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.too_small {
            if key.is_char('q') || key.ctrl_char('c') {
                self.quit = true;
            }
            return Outcome::Consumed;
        }
        if self.modals.is_empty() && key.ctrl_char('c') {
            self.quit = true;
            return Outcome::Consumed;
        }
        if !self.modals.is_empty() {
            return self.modal_key(key);
        }
        // quit is chrome: confirm even while the query owns plain characters
        if key.ctrl_char('q') {
            self.go(Go::Quit);
            return Outcome::Changed;
        }
        if self.host_menu.is_open() {
            let (o, ev) = self.host_menu.on_key(&key);
            return match ev {
                Some(MenuBarEvent::Chosen(mi, ii)) => {
                    let label = self.host_menu.menus[mi][ii].label.clone();
                    self.run_host_menu(&label)
                }
                Some(MenuBarEvent::Brand) => {
                    self.open_about();
                    Outcome::Changed
                }
                _ => o.or(Outcome::Changed),
            };
        }
        // the menu bar is chrome, not a chord: F10 works even while the
        // query owns plain characters
        if key.code == KeyCode::F(10) {
            self.host_menu.open_menu(0);
            return Outcome::Changed;
        }
        if key.code == KeyCode::F(1) {
            self.open_help();
            return Outcome::Changed;
        }
        // activities are chrome too: Ctrl+A cycles the strip from any route,
        // so the keyboard never needs the mouse to reach retained output
        if key.ctrl_char('a') && !self.world.activities.is_empty() {
            let ordered = activity::ordered(&self.world);
            let next = match self.route {
                Route::Activity => {
                    let pos = ordered
                        .iter()
                        .position(|a| Some(a.id) == self.activity.current)
                        .unwrap_or(0);
                    ordered[(pos + 1) % ordered.len()].id
                }
                _ => ordered[0].id,
            };
            self.go(Go::Activity(next));
            return Outcome::Changed;
        }
        let editing = match self.route {
            Route::Home => self.home.is_editing(),
            Route::Plan | Route::Activity => false,
        };
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let mut out = match self.route {
            Route::Home => self.home.on_key(&key, &mut self.world, &mut cx),
            Route::Plan => match &mut self.plan {
                Some(p) => p.on_key(&key, &mut self.world, &mut cx),
                None => Outcome::Ignored,
            },
            Route::Activity => self.activity.on_key(&key, &mut self.world, &mut cx),
        };
        let reqs = std::mem::take(&mut cx.requests);
        out = out.or(self.apply_requests(reqs, self.route));
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
            KeyCode::Esc => {
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                let o = match self.route {
                    Route::Home => self.home.on_esc_top(&mut self.world, &mut cx),
                    Route::Plan => match &mut self.plan {
                        Some(p) => p.on_esc_top(&mut self.world, &mut cx),
                        None => Outcome::Ignored,
                    },
                    Route::Activity => self.activity.on_esc_top(&mut self.world, &mut cx),
                };
                let reqs = std::mem::take(&mut cx.requests);
                o.or(self.apply_requests(reqs, self.route))
            }
            _ => Outcome::Ignored,
        }
    }

    /// The menu bar: the screen's exits under Go, help under Help.
    fn build_host_menu() -> MenuBar {
        let file = vec![MenuItem::new("Quit").shortcut("Ctrl+Q")];
        let go = vec![MenuItem::new("Home").shortcut("0")];
        let help = vec![
            MenuItem::new("Key reference").shortcut("F1"),
            MenuItem::new("About holla"),
        ];
        MenuBar::new(HOST_MENU, vec![("File", file), ("Go", go), ("Help", help)])
            .brand(Lockup::new(BRAND_MARK))
    }

    /// A menu item either navigates or presses the key the item names, so
    /// menus and keys can never disagree.
    fn run_host_menu(&mut self, label: &str) -> Outcome {
        match label {
            "Home" => self.go(Go::Home),
            "Key reference" => self.open_help(),
            "About holla" => self.open_about(),
            "Quit" => self.go(Go::Quit),
            _ => self.set_status(
                &format!("{label}: not available in this build"),
                Tone::Secondary,
            ),
        }
        Outcome::Changed
    }

    fn open_about(&mut self) {
        let props = vec![
            Prop::new("Product", "holla · context-adaptive action launcher"),
            Prop::new("Scenario", self.world.scenario.name()),
            Prop::new(
                "Stack",
                "simulated · mise, git, docker, ssh are never executed",
            ),
            Prop::new(
                "Design system",
                "Junie-inspired Ratatui components (junie_tui)",
            ),
        ];
        let mut d = Dialog::facts(
            WidgetId::of("host.about"),
            "About holla",
            props,
            vec![],
            None,
            junie_tui::widgets::button::Button::primary(WidgetId::of("host.about.ok"), "Close"),
        );
        let close = d.actions.pop().unwrap();
        d.actions.clear();
        d.actions.push(close);
        d.cancel_index = Some(0);
        self.push_modal(Modal::Dialog(d), ModalTag::new("about"), self.route);
    }

    fn open_help(&mut self) {
        if !self.modals.is_empty() {
            self.set_status("Close the dialog first", Tone::Secondary);
            return;
        }
        let props = vec![
            Prop::new("Type", "filter the action list"),
            Prop::new("↑ ↓", "move between query and results"),
            Prop::new("Enter", "run the focused action"),
            Prop::new("Ctrl+O / Ctrl+P", "actions menu / preview"),
            Prop::new("Esc", "clear query · close dialog"),
            Prop::new("F10", "menu bar"),
            Prop::new("?", "this reference (empty query)"),
            Prop::new("q", "quit (empty query)"),
        ];
        let mut d = Dialog::facts(
            WidgetId::of("host.help"),
            "Key reference",
            props,
            vec![],
            None,
            junie_tui::widgets::button::Button::primary(WidgetId::of("host.help.ok"), "Close"),
        );
        // pure reference: one button, Esc maps to it
        let close = d.actions.pop().unwrap();
        d.actions.clear();
        d.actions.push(close);
        d.cancel_index = Some(0);
        self.push_modal(Modal::Dialog(d), ModalTag::new("help"), self.route);
    }

    fn open_quit_confirm(&mut self) {
        // sensitive hosts get the stronger confirmation: the dialog names
        // the remote identity so "which machine am I leaving" is explicit.
        let sensitive = self.world.host.env.sensitive();
        let body = if sensitive {
            format!(
                "You are on {}. The simulated session ends; nothing on that host changes.",
                self.world.host.identity()
            )
        } else {
            "The simulated session ends; nothing on this host changes.".to_owned()
        };
        let title = if sensitive {
            format!("Quit holla on {}?", self.world.host.name)
        } else {
            "Quit holla?".to_owned()
        };
        let mut d = Dialog::confirm(WidgetId::of("host.quit"), &title, &body, "Quit");
        // cancellation is the default: focus starts on Cancel
        d.initial_focus = d.actions[0].id;
        self.push_modal(Modal::Dialog(d), ModalTag::new("quit"), self.route);
    }

    pub fn go(&mut self, g: Go) {
        match g {
            Go::Home => {
                self.route = Route::Home;
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                self.home.enter(&mut self.world, &mut cx);
                let reqs = std::mem::take(&mut cx.requests);
                self.apply_requests(reqs, Route::Home);
            }
            Go::Plan(plan) => {
                self.plan = Some(PlanScreen::new(*plan));
                self.route = Route::Plan;
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                if let Some(p) = &mut self.plan {
                    p.enter(&mut self.world, &mut cx);
                }
                let reqs = std::mem::take(&mut cx.requests);
                self.apply_requests(reqs, Route::Plan);
                self.focus
                    .set(self.plan.as_ref().and_then(|p| p.primary_focus()));
            }
            Go::Activity(id) => {
                self.activity.show(id);
                self.route = Route::Activity;
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                self.activity.enter(&mut self.world, &mut cx);
                let reqs = std::mem::take(&mut cx.requests);
                self.apply_requests(reqs, Route::Activity);
                self.focus.set(self.activity.primary_focus());
            }
            Go::Quit => self.open_quit_confirm(),
        }
    }

    // ------------------------------------------------------------ modals

    fn push_modal(&mut self, modal: Modal, tag: ModalTag, owner: Route) {
        let initial = match &modal {
            Modal::Dialog(d) => Some(d.initial_focus),
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
        let mut cx = Cx {
            focus: &mut self.focus,
            ring: &self.ring,
            requests: vec![],
        };
        let o = match owner {
            Route::Home => self
                .home
                .on_modal(&entry.tag, result, &mut self.world, &mut cx),
            Route::Plan => match &mut self.plan {
                Some(p) => p.on_modal(&entry.tag, result, &mut self.world, &mut cx),
                None => Outcome::Ignored,
            },
            Route::Activity => self
                .activity
                .on_modal(&entry.tag, result, &mut self.world, &mut cx),
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
                        DialogBody::Facts { ack: Some(a), .. } => Some(a.input.text().to_owned()),
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
                out.or(Outcome::Consumed)
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

    fn modal_click(&mut self, id: WidgetId, pos: ratatui::layout::Position) -> Outcome {
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
        match &mut top.modal {
            Modal::Dialog(d) => {
                let o = d.on_click(id, pos, &mut self.focus);
                if let Some(result) = d.result {
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
                    return self.deliver(entry, ModalResult::Dialog { action, text: None });
                }
                o.or(Outcome::Changed)
            }
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

    fn modal_outside_click(&mut self) -> Outcome {
        let Some(top) = self.modals.last_mut() else {
            return Outcome::Ignored;
        };
        match &mut top.modal {
            Modal::Dialog(d) => {
                // typed acknowledgements and safety dialogs stay open
                if matches!(d.body, DialogBody::Facts { .. }) {
                    return Outcome::Consumed;
                }
                let out = d.on_click_outside();
                if d.result.is_some() {
                    let entry = self.pop_modal().unwrap();
                    if entry.tag.kind == "quit" {
                        return Outcome::Changed;
                    }
                    return self.deliver(
                        entry,
                        ModalResult::Dialog {
                            action: None,
                            text: None,
                        },
                    );
                }
                out
            }
            Modal::Custom(c) => {
                if c.cancel_on_outside_click() {
                    let entry = self.pop_modal().unwrap();
                    self.deliver(entry, ModalResult::Cancelled)
                } else {
                    Outcome::Consumed
                }
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
                if self.hover != was || suppressed {
                    Outcome::Changed
                } else {
                    Outcome::Ignored
                }
            }
            MouseKind::Drag => {
                self.hover = self.hits.hit(m.pos);
                Outcome::Consumed
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
                Outcome::Changed
            }
            MouseKind::Up => {
                let hit = self.hits.hit(m.pos);
                let pressed = self.pressed.take();
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
                let _ = double;
                self.last_click = Some((id, self.world.now_ms()));
                if !self.modals.is_empty() {
                    return self.modal_click(id, m.pos);
                }
                if self.host_menu.owns(id) {
                    let (o, ev) = self.host_menu.on_click(id);
                    return match ev {
                        Some(MenuBarEvent::Chosen(mi, ii)) => {
                            let label = self.host_menu.menus[mi][ii].label.clone();
                            self.run_host_menu(&label)
                        }
                        Some(MenuBarEvent::Brand) => {
                            self.open_about();
                            Outcome::Changed
                        }
                        _ => o.or(Outcome::Changed),
                    };
                }
                if self.host_menu.is_open() {
                    self.host_menu.close();
                    return Outcome::Changed;
                }
                if id == STRIP_HELP {
                    self.open_help();
                    return Outcome::Changed;
                }
                // the activity strip is chrome: clickable from every route
                if let Some(aid) = activity::hit_activity(id, &self.world) {
                    self.go(Go::Activity(aid));
                    return Outcome::Changed;
                }
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: vec![],
                };
                let o = match self.route {
                    Route::Home => self.home.on_click(id, m.pos, &mut self.world, &mut cx),
                    Route::Plan => match &mut self.plan {
                        Some(p) => p.on_click(id, m.pos, &mut self.world, &mut cx),
                        None => Outcome::Ignored,
                    },
                    Route::Activity => self.activity.on_click(id, m.pos, &mut self.world, &mut cx),
                };
                let reqs = std::mem::take(&mut cx.requests);
                o.or(self.apply_requests(reqs, self.route))
                    .or(Outcome::Changed)
            }
            MouseKind::Secondary => Outcome::Ignored,
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
                        Modal::Custom(c) => c.on_wheel(delta, m.pos),
                        Modal::Dialog(_) => Outcome::Consumed,
                    };
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                match self.route {
                    Route::Home => self.home.on_wheel(id, delta, m.pos, &mut self.world),
                    Route::Plan => Outcome::Ignored,
                    Route::Activity => self.activity.on_wheel(id, delta, m.pos, &mut self.world),
                }
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
                Request::Copy(_s) => {
                    self.clipboard_gen += 1;
                    self.set_status("Copied · simulated clipboard", Tone::Secondary);
                }
            }
        }
        out
    }

    // ------------------------------------------------------------- render

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
                let pf = match self.route {
                    Route::Home => self.home.primary_focus(),
                    Route::Plan => self.plan.as_ref().and_then(|p| p.primary_focus()),
                    Route::Activity => self.activity.primary_focus(),
                };
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
                    format!(" {BRAND_MARK} "),
                    junie_tui::widgets::brand::Lockup::style(&t),
                ),
                ("Terminal too small".to_owned(), t.secondary()),
                (
                    format!(
                        "Need {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}",
                        area.width, area.height
                    ),
                    t.muted(),
                ),
                ("q Quit".to_owned(), t.faint()),
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
        // modals render last, then the footer speaks for the modal
        if let Some(mut entry) = self.modals.pop() {
            match &mut entry.modal {
                Modal::Dialog(d) => d.render(area, buf, ctx),
                Modal::Custom(c) => c.render(area, buf, ctx, &self.world),
            }
            self.modals.push(entry);
            let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
            self.draw_footer(footer, buf, true);
        }
    }

    /// Menu bar + body + footer (no modals).
    fn draw_frame(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
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
        self.draw_host_menu(header, buf, ctx);
        // the activity strip docks under the menu bar whenever work exists
        if !self.world.activities.is_empty() {
            let strip = Rect::new(area.x, area.y + 1, area.width, 1);
            self.draw_strip(strip, buf, ctx);
        }
        match self.route {
            Route::Home => self.home.render(body, buf, ctx, &self.world),
            Route::Plan => {
                if let Some(p) = &mut self.plan {
                    p.render(body, buf, ctx, &self.world);
                }
            }
            Route::Activity => self.activity.render(body, buf, ctx, &self.world),
        }
        self.draw_footer(footer, buf, false);
        self.host_menu.render_open(area, buf, ctx);
    }

    /// Row one: one tab per activity, state-toned, clickable; the tab of the
    /// open page renders inverted. Tabs overflow silently right-to-left.
    fn draw_strip(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let mut x = area.x + 1;
        for (i, a) in activity::ordered(&self.world).iter().enumerate() {
            let label = format!(" {} {} {} ", activity::state_glyph(a.state), i + 1, a.name);
            let w = width(&label) as u16;
            if x + w + 1 > area.right() {
                break;
            }
            let id = activity::STRIP.child(a.id as usize);
            let r = Rect::new(x, area.y, w, 1);
            ctx.control(id, r, false);
            let active = self.route == Route::Activity && self.activity.current == Some(a.id);
            let st = if active {
                ratatui::style::Style::new()
                    .fg(t.canvas)
                    .bg(t.tone(activity::state_tone(a.state)))
            } else {
                t.row(ctx.state(id), t.canvas)
                    .fg(t.tone(activity::state_tone(a.state)))
            };
            for cx2 in r.x..r.right() {
                buf[(cx2, area.y)].set_style(st);
            }
            buf.set_string(x, area.y, &label, st);
            x += w + 1;
        }
        if x + 10 < area.right() {
            buf.set_string(x, area.y, "0 home", t.faint());
        }
    }

    /// Row zero: the menu bar, then the crumb, discovery state and host
    /// identity right-aligned in what is left of the row.
    fn draw_host_menu(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        self.host_menu.on_hover(ctx.interaction.hover);
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
        let mut right = vec![];
        let crumb = match self.route {
            Route::Home => self.home.crumb(&self.world),
            Route::Plan => self
                .plan
                .as_ref()
                .map(|p| p.crumb(&self.world))
                .unwrap_or_default(),
            Route::Activity => self.activity.crumb(&self.world),
        };
        // the crumb truncates before local chrome drops: identity, color ·
        // size and ? help are terminal facts, the path is scenario data
        let (identity, tone) = match self.world.host.env {
            crate::domain::Environment::Production => (self.world.host.identity(), Tone::Warning),
            crate::domain::Environment::Staging => (self.world.host.identity(), Tone::Warning),
            _ => (self.world.host.identity(), Tone::Secondary),
        };
        let level = format!("{} · {}×{}", t.level.label(), self.size.0, self.size.1);
        // segments::render adds 2 per segment plus a flat 2 — reserve for
        // four right segments (crumb, identity, level, ? help) exactly
        let reserved = width(&identity) + width(&level) + width("? help") + 10;
        let crumb_budget = (rest.width as usize).saturating_sub(reserved);
        if crumb_budget >= 8 && width(&crumb) > crumb_budget {
            right.push(
                Segment::new(truncate_middle(&crumb, crumb_budget), Tone::Secondary).priority(7),
            );
        } else if crumb_budget >= 8 {
            right.push(Segment::new(crumb, Tone::Secondary).priority(7));
        }
        match self.route {
            Route::Home => right.extend(self.home.header_right(&self.world)),
            Route::Plan => {
                if let Some(p) = &self.plan {
                    right.extend(p.header_right(&self.world));
                }
            }
            Route::Activity => right.extend(self.activity.header_right(&self.world)),
        }
        if self.world.discovering() {
            right.push(
                Segment::new(
                    format!(
                        "{} discovering…",
                        junie_tui::widgets::progress::spinner_frame(ctx.interaction.tick)
                    ),
                    Tone::Secondary,
                )
                .priority(5),
            );
        }
        // the host identity never drops: a remote or sensitive machine must
        // stay unmistakable
        right.push(Segment::new(identity, tone).priority(9));
        right.push(Segment::new(level, Tone::Faint).priority(1));
        right.push(
            Segment::new("? help", Tone::Muted)
                .clickable(STRIP_HELP)
                .priority(3),
        );
        segments::render(rest, buf, ctx, &[], &right, t.canvas);
    }

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, modal: bool) {
        let t = self.theme;
        fill(buf, area, t.base());
        let hints: Vec<Hint> = if modal {
            match self.modals.last().map(|m| &m.modal) {
                Some(Modal::Dialog(d)) => {
                    if d.is_editing() {
                        vec![hint("Enter", "Next"), hint("Esc", "Cancel")]
                    } else if matches!(d.body, DialogBody::Facts { .. }) {
                        vec![
                            hint("← →", "Choose"),
                            hint("Enter", "Confirm"),
                            hint("Esc", "Close"),
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
                Some(Modal::Custom(c)) => c.hints(),
                None => vec![],
            }
        } else {
            let focus = self.focus.current();
            match self.route {
                Route::Home => self.home.hints(focus, &self.world),
                Route::Plan => self
                    .plan
                    .as_ref()
                    .map(|p| p.hints(focus, &self.world))
                    .unwrap_or_default(),
                Route::Activity => self.activity.hints(focus, &self.world),
            }
        };
        // the badge speaks for the topmost context only: a modal that is not
        // editing must not inherit the screen's armed query
        let editing = if modal {
            self.modals.last().is_some_and(|m| match &m.modal {
                Modal::Dialog(d) => d.is_editing(),
                Modal::Custom(_) => false,
            })
        } else {
            match self.route {
                Route::Home => self.home.is_editing(),
                Route::Plan | Route::Activity => false,
            }
        };
        let badge = if editing {
            Some(("EDIT", BadgeKind::Edit))
        } else {
            None
        };
        let status = self
            .status
            .as_ref()
            .map(|(s, tone, _)| (truncate(s, area.width.saturating_sub(4) as usize), *tone));
        let status = status.as_ref().map(|(s, tone)| (s.as_str(), *tone));
        // one hint surface: topmost modal › open menu › the screen › fallback
        let modal_layer = modal.then(|| HintLayer::new(hints.clone()));
        let menu_layer = (!modal && self.host_menu.is_open()).then(|| {
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
        layer.status = status.map(|(s, tone)| (s.to_owned(), tone));
        layer.centered = true;
        HintBar::render(area, buf, &t, &layer);
    }
}
