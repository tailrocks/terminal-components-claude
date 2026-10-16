//! TablePro shell: screens (Connections, Workbench), overlays (dialogs,
//! pickers, filter editor), identity strip, footer hints, routing.

use junie_tui::core::focus::forward_blur;
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::core::pointer::{FocusOn, PointerState, Release, SecondaryRoute};
use junie_tui::theme::{BadgeKind, Theme, Tone};
use junie_tui::ui::context::ContextConfig;
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::ui::modal::{Dismissal, ModalStack};
use junie_tui::ui::popup::Dismiss;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::field_common::{route_click, route_drag, route_press};
use junie_tui::widgets::grid::CellValue;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{self, Hint, hint};
use junie_tui::widgets::menu::MenuEvent;
use junie_tui::widgets::picker::{Picker, PickerEvent, PickerItem};
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::select::Select;

use crate::connections::{ConnEvent, ConnectionsScreen};
use crate::context::{self, CtxMenu};
use crate::db::{Catalog, Environment, SafeMode};
use crate::model::{History, SwitchTarget, SwitcherIndex};
use crate::sql::{self, Decision};
use crate::tabs::{Filter, FilterOp};
use crate::workbench::{WorkTab, Workbench};

pub const MIN_WIDTH: u16 = 72;
pub const MIN_HEIGHT: u16 = 20;

const STRIP_SAFE: WidgetId = WidgetId::of("strip.safe");
const STRIP_SCOPE: WidgetId = WidgetId::of("strip.scope");
const STRIP_CONN: WidgetId = WidgetId::of("strip.conn");
const STRIP_HELP: WidgetId = WidgetId::of("strip.help");
const HELP_DIALOG: WidgetId = WidgetId::of("dialog.help");
const SAFETY_DIALOG: WidgetId = WidgetId::of("dialog.safety");
const DISCARD_DIALOG: WidgetId = WidgetId::of("dialog.discard");
const CLOSE_DIALOG: WidgetId = WidgetId::of("dialog.close");
const COMMIT_DIALOG: WidgetId = WidgetId::of("dialog.commit");
const PREVIEW_DIALOG: WidgetId = WidgetId::of("dialog.preview");
const VIEWER_DIALOG: WidgetId = WidgetId::of("dialog.viewer");
const QUIT_DIALOG: WidgetId = WidgetId::of("dialog.quit");

/// Things screens ask the app to do.
pub enum Request {
    OpenDialog(Box<Dialog>),
    Status(String),
    EditFilter(Option<usize>),
    FilterOnCell(usize, CellValue),
    ConfirmDiscard,
    CommitPending,
    PreviewSql,
    OpenQuery(String, bool),
    OpenTableFiltered(String, String, String),
    OpenViewer(String, CellValue),
    ConfirmCloseTab(usize),
}

pub struct Cx<'a> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub requests: Vec<Request>,
    /// Multi-click count of the click being delivered
    /// ([`PointerState::count_click`]); 1 for every other event.
    pub clicks: u8,
}

impl Cx<'_> {
    pub fn focus_next(&mut self) {
        self.focus.next(self.ring);
    }
    pub fn focus_prev(&mut self) {
        self.focus.prev(self.ring);
    }
    pub fn status(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Status(s.into()));
    }
    pub fn open(&mut self, d: Dialog) {
        self.requests.push(Request::OpenDialog(Box::new(d)));
    }
}

/// Tick delivery (`--motion`), the same vocabulary as the other binaries.
///
/// - `Full`: live ticks advance the world (connect, schema load, query run,
///   commit) and the decorative phase (activity spinners).
/// - `Reduced`: the world still advances, the decorative phase stays at its
///   start, so spinners do not animate.
/// - `Paused`: the world is fast-forwarded `--frame N` ticks at startup and
///   never ticks again; the decorative phase is pinned at `N`, the status
///   line never ages out, and a press flash stays visible. Captures pin
///   every tick-derived and wall-clock-derived state this way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Motion {
    #[default]
    Full,
    Reduced,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Connections,
    Workbench,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Switcher,
    TabList,
    SafeMode,
}

pub struct FilterEditor {
    pub index: Option<usize>,
    pub column: Select,
    pub op: Select,
    pub value: TextInput,
    pub value2: TextInput,
    pub apply: Button,
    pub cancel: Button,
    pub area: Rect,
    columns: Vec<(String, crate::db::ColType)>,
    /// A form holds typed input: only Esc (or Cancel) discards it, a stray
    /// click outside does not.
    pub dismiss: Dismiss,
}

#[allow(clippy::large_enum_variant)] // overlays are few and short-lived
pub enum Modal {
    Dialog(Dialog),
    Picker(PickerKind, Picker, Option<SwitcherIndex>),
    Filter(FilterEditor),
    /// A context menu on an explorer row, a tab or a grid cell.
    Menu(Box<CtxMenu>),
}

impl junie_tui::ui::modal::Modal for Modal {
    fn dismiss(&self) -> Dismiss {
        match self {
            Modal::Dialog(d) => d.dismiss,
            Modal::Picker(_, p, _) => p.dismiss,
            Modal::Filter(f) => f.dismiss,
            Modal::Menu(m) => m.menu.dismiss,
        }
    }
}

pub struct App {
    pub theme: Theme,
    pub screen: Screen,
    pub connections: ConnectionsScreen,
    pub workbench: Option<Workbench>,
    pub history: History,
    pub focus: Focus,
    pub ring: FocusRing,
    pub hits: HitRegistry,
    /// Hover, press, flash and click-count state.
    pub pointer: PointerState,
    /// Context menus: which keys open one at the focused cursor, and
    /// whether a right-click moves the cursor onto its row. Default
    /// [`ContextConfig::DEFAULT`] (Shift+F10 and the Menu key).
    pub context: ContextConfig,
    /// Origin of the pointer clock.
    epoch: Instant,
    /// Open overlays, bottom to top; each restores the focus it covered.
    pub modals: ModalStack<Modal>,
    pub size: (u16, u16),
    pub tick: u64,
    /// Tick delivery; see [`Motion`].
    pub motion: Motion,
    pub status: Option<(String, Instant)>,
    pub quit: bool,
    too_small: bool,
    /// Simulated commit in flight: ticks left.
    committing: Option<u32>,
    scope: usize,
    switcher_targets: Vec<SwitchTarget>,
}

impl App {
    pub fn new(theme: Theme) -> Self {
        let connections = ConnectionsScreen::new(crate::db::connections());
        let mut focus = Focus::default();
        focus.focus(WidgetId::of("connections").sub("tree"));
        Self {
            theme,
            screen: Screen::Connections,
            connections,
            workbench: None,
            history: History::seeded(),
            focus,
            ring: FocusRing::default(),
            hits: HitRegistry::default(),
            pointer: PointerState::default(),
            context: ContextConfig::DEFAULT,
            epoch: Instant::now(),
            modals: ModalStack::new(),
            size: (0, 0),
            tick: 0,
            motion: Motion::Full,
            status: None,
            quit: false,
            too_small: false,
            committing: None,
            scope: 0,
            switcher_targets: vec![],
        }
    }

    /// Connect immediately (used by `--connect` and tests).
    pub fn connect(&mut self, index: usize) {
        let Some(c) = self.connections.connections.get(index).cloned() else {
            return;
        };
        let mut wb = Workbench::new(c, Catalog::acme_prod());
        wb.new_query("");
        // start on the explorer so the first Enter opens a table
        self.focus.focus(crate::workbench::EXPLORER);
        self.workbench = Some(wb);
        self.screen = Screen::Workbench;
        self.set_status(format!(
            "Connected to {}",
            self.workbench.as_ref().unwrap().connection.name
        ));
    }

    /// Apply `--motion` / `--frame`. `Paused` fast-forwards `frame` ticks
    /// synchronously (each one exactly what a live tick does), then pins the
    /// decorative phase at `frame`; no tick is delivered afterwards.
    pub fn set_motion(&mut self, motion: Motion, frame: u64) {
        self.motion = Motion::Full;
        if motion == Motion::Paused {
            for _ in 0..frame {
                self.advance();
            }
            self.tick = frame;
        }
        self.motion = motion;
    }

    pub fn animating(&self) -> bool {
        if self.motion == Motion::Paused {
            return false;
        }
        self.connections.animating()
            || self.workbench.as_ref().is_some_and(|w| w.animating())
            || self.pointer.has_flash()
            || self.committing.is_some()
    }

    pub fn tick_interval(&self) -> Duration {
        if self.animating() {
            Duration::from_millis(80)
        } else {
            Duration::from_millis(400)
        }
    }

    fn set_status(&mut self, s: String) {
        self.status = Some((s, Instant::now()));
    }

    /// Milliseconds on the pointer clock.
    fn now_ms(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    /// The clock a press flash is read against: frozen at the epoch under
    /// paused motion, so a flash stays visible for the capture.
    fn flash_clock(&self) -> u64 {
        if self.motion == Motion::Paused {
            0
        } else {
            self.now_ms()
        }
    }

    pub(crate) fn interaction(&self) -> Interaction {
        let flash = self.pointer.flashing(self.flash_clock());
        Interaction {
            focus: self.focus.current(),
            hover: self.pointer.hover,
            pressed: self.pointer.pressed,
            flash,
            focus_hidden: false,
            hover_suppressed: self.pointer.hover_suppressed,
            pointer_focus: self.pointer.pointer_focus,
            tick: self.tick,
        }
    }

    // ---------------------------------------------------------------- input

    pub fn handle(&mut self, input: Input) -> Outcome {
        let out = self.handle_input(input);
        out.or(self.forward_blur())
    }

    /// Hand every focus change the input caused to the modals and the
    /// screens, so fields end their edits on the event path.
    fn forward_blur(&mut self) -> Outcome {
        let mut out = Outcome::Ignored;
        for ch in self.focus.settle(&self.ring) {
            for m in self.modals.iter_mut() {
                out = out.or(match m {
                    Modal::Dialog(d) => d.on_blur(ch),
                    Modal::Filter(f) => {
                        forward_blur(ch, &mut [&mut f.column, &mut f.op, &mut f.value])
                    }
                    Modal::Picker(..) | Modal::Menu(_) => Outcome::Ignored,
                });
            }
            out = out.or(self.connections.on_blur(ch));
            if let Some(w) = self.workbench.as_mut() {
                out = out.or(w.on_blur(ch));
            }
        }
        out
    }

    fn handle_input(&mut self, input: Input) -> Outcome {
        match input {
            Input::Resize(w, h) => {
                self.size = (w, h);
                Outcome::Changed
            }
            Input::Tick => self.on_tick(),
            Input::Paste(text) => {
                if let Some(Modal::Dialog(d)) = self.modals.top_mut() {
                    return d.on_paste(&text);
                }
                if let Some(Modal::Filter(f)) = self.modals.top_mut() {
                    if f.value.editing {
                        return f.value.on_paste(&text);
                    }
                    return Outcome::Consumed;
                }
                if let Some(Modal::Menu(_)) = self.modals.top() {
                    return Outcome::Consumed;
                }
                match self.screen {
                    Screen::Connections => self.connections.on_paste(&text),
                    Screen::Workbench => self
                        .workbench
                        .as_mut()
                        .map(|w| w.on_paste(&text))
                        .unwrap_or(Outcome::Ignored),
                }
            }
            Input::Key(key) => {
                if key.ctrl_char('c') {
                    // cancel a running query first; otherwise quit
                    if let Some(w) = self.workbench.as_mut()
                        && let Some(WorkTab::Query(q)) = w.active_tab_mut()
                        && q.cancel()
                    {
                        self.set_status("Query cancelled".into());
                        return Outcome::Changed;
                    }
                    return self.request_quit();
                }
                self.pointer.suppress_hover();
                self.on_key(key)
            }
            Input::Mouse(m) => self.on_mouse(m),
        }
    }

    fn request_quit(&mut self) -> Outcome {
        let pending = self
            .workbench
            .as_ref()
            .map(|w| w.pending_total())
            .unwrap_or(0);
        let dirty_queries = self
            .workbench
            .as_ref()
            .map(|w| {
                w.tabs
                    .iter()
                    .filter(|t| matches!(t, WorkTab::Query(q) if q.dirty()))
                    .count()
            })
            .unwrap_or(0);
        if pending > 0 || dirty_queries > 0 {
            let mut parts = vec![];
            if pending > 0 {
                parts.push(format!(
                    "{pending} pending row change{}",
                    if pending == 1 { "" } else { "s" }
                ));
            }
            if dirty_queries > 0 {
                parts.push(format!(
                    "{dirty_queries} unsaved quer{}",
                    if dirty_queries == 1 { "y" } else { "ies" }
                ));
            }
            let d = Dialog::destructive(
                QUIT_DIALOG,
                "Quit TablePro?",
                &format!("{} will be lost.", parts.join(" and ")),
                "Quit",
            );
            self.open_dialog(d);
            return Outcome::Changed;
        }
        self.quit = true;
        Outcome::Consumed
    }

    fn on_tick(&mut self) -> Outcome {
        if self.motion == Motion::Paused {
            return Outcome::Ignored;
        }
        self.advance()
    }

    /// One tick of the world. The decorative phase advances only under
    /// full motion.
    fn advance(&mut self) -> Outcome {
        let mut out = Outcome::Ignored;
        if self.animating() {
            if self.motion == Motion::Full {
                self.tick = self.tick.wrapping_add(1);
            }
            out = Outcome::Changed;
        }
        if self.pointer.expire_flash(self.now_ms()) {
            out = Outcome::Changed;
        }
        if let Some((_, at)) = &self.status
            && at.elapsed() > Duration::from_secs(5)
        {
            self.status = None;
            out = Outcome::Changed;
        }
        if let Some(ConnEvent::Connected(i)) = self.connections.tick() {
            self.connect(i);
            out = Outcome::Changed;
        }
        if let Some(left) = self.committing.as_mut() {
            *left = left.saturating_sub(1);
            if *left == 0 {
                self.committing = None;
                self.finish_commit();
            }
            out = Outcome::Changed;
        }
        if let Some(w) = self.workbench.as_mut() {
            if w.tick_explorer() {
                out = Outcome::Changed;
            }
            let conn = w.connection.name.clone();
            let db = w.catalog.database.clone();
            let cat = &w.catalog;
            let mut entries = vec![];
            for t in w.tabs.iter_mut() {
                if let WorkTab::Query(q) = t {
                    let was = q.is_running();
                    entries.extend(q.tick(cat, &conn, &db));
                    if was && !q.is_running() {
                        out = Outcome::Changed;
                    }
                }
            }
            if !entries.is_empty() {
                for e in entries {
                    self.history.push(e);
                }
                // refresh an open history tab
                let hist = &self.history;
                let conn2 = conn.clone();
                for t in w.tabs.iter_mut() {
                    if let WorkTab::History(h) = t {
                        h.refresh(hist, &conn2);
                    }
                }
                out = Outcome::Changed;
            }
            if w.running().is_some() {
                out = Outcome::Changed;
            }
        }
        out
    }

    fn apply_requests(&mut self, requests: Vec<Request>) -> Outcome {
        let mut out = Outcome::Ignored;
        for r in requests {
            out = Outcome::Changed;
            match r {
                Request::OpenDialog(d) => self.open_dialog(*d),
                Request::Status(s) => self.set_status(s),
                Request::EditFilter(i) => self.open_filter_editor(i, None),
                Request::FilterOnCell(col, v) => {
                    let val = match &v {
                        CellValue::Null => (FilterOp::IsNull, String::new()),
                        other => (FilterOp::Eq, other.text()),
                    };
                    self.open_filter_editor(None, Some((col, val.0, val.1)));
                }
                Request::ConfirmDiscard => {
                    let n = self
                        .workbench
                        .as_ref()
                        .map(|w| w.pending_total())
                        .unwrap_or(0);
                    let d = Dialog::destructive(
                        DISCARD_DIALOG,
                        "Discard unsaved changes?",
                        &format!(
                            "{n} pending change{} will be dropped. The rows are reloaded from the server.",
                            if n == 1 { "" } else { "s" }
                        ),
                        "Discard",
                    );
                    self.open_dialog(d);
                }
                Request::CommitPending => self.begin_commit(),
                Request::PreviewSql => self.open_preview(),
                Request::OpenQuery(sql, run) => {
                    if let Some(w) = self.workbench.as_mut() {
                        let i = w.new_query(&sql);
                        if let Some(pf) = w.primary_focus() {
                            self.focus.focus(pf);
                        }
                        if run {
                            let _ = i;
                            let o = self.run_active(false, None);
                            out = out.or(o);
                        }
                    }
                }
                Request::OpenTableFiltered(table, col, value) => {
                    if let Some(w) = self.workbench.as_mut() {
                        let schema = w
                            .catalog
                            .find(None, &table)
                            .map(|t| t.schema.clone())
                            .unwrap_or("public".into());
                        if let Some(i) = w.open_table(&schema, &table, false) {
                            if let Some(WorkTab::Table(t)) = w.tabs.get_mut(i) {
                                t.filters = vec![Filter {
                                    column: col,
                                    op: FilterOp::Eq,
                                    value,
                                    value2: String::new(),
                                    enabled: true,
                                }];
                                let cat = w.catalog.clone();
                                t.load(&cat);
                            }
                            if let Some(pf) = w.primary_focus() {
                                self.focus.focus(pf);
                            }
                        }
                    }
                }
                Request::OpenViewer(name, v) => {
                    let text = match &v {
                        CellValue::Json(j) => pretty_json(j),
                        other => other.text(),
                    };
                    let lines: Vec<String> = text.lines().map(str::to_owned).collect();
                    let mut d = Dialog::facts(
                        VIEWER_DIALOG,
                        &name,
                        vec![],
                        lines,
                        None,
                        Button::secondary(VIEWER_DIALOG.sub("close"), "Close"),
                    );
                    d.actions.remove(0);
                    d.cancel_index = Some(0);
                    d.initial_focus = d.actions[0].id;
                    d.width = 80;
                    self.open_dialog(d);
                }
                Request::ConfirmCloseTab(i) => {
                    let d = Dialog::destructive(
                        CLOSE_DIALOG.child(i),
                        "Close tab with unsaved work?",
                        "Pending row edits and unsaved query text in this tab will be lost.",
                        "Close anyway",
                    );
                    self.open_dialog(d);
                }
            }
        }
        out
    }

    fn on_key(&mut self, key: Key) -> Outcome {
        if self.too_small {
            if key.is_char('q') {
                self.quit = true;
            }
            return Outcome::Consumed;
        }
        if !self.modals.is_empty() {
            return self.modal_key(key);
        }
        // global chords that work everywhere (not while editing text)
        let editing = match self.screen {
            Screen::Connections => self.connections.is_editing(),
            Screen::Workbench => self.workbench.as_ref().is_some_and(|w| w.is_editing()),
        };
        if key.ctrl_char('n') && self.screen == Screen::Connections && !editing {
            self.connections.open_new();
            self.focus
                .focus(WidgetId::of("connections").sub("form").sub("name"));
            return Outcome::Changed;
        }
        if self.screen == Screen::Workbench
            && !editing
            && let Some(w) = self.workbench.as_ref()
            && let Some(menu) = context::on_key(w, &self.context, self.focus.current(), &key)
        {
            return self.open_context(menu);
        }
        if self.screen == Screen::Connections
            && !editing
            && let Some(menu) = context::on_key_connections(
                &self.connections,
                &self.context,
                self.focus.current(),
                &key,
            )
        {
            return self.open_context(menu);
        }
        if self.screen == Screen::Workbench
            && let Some(o) = self.workbench_chord(&key, editing)
        {
            return o;
        }
        // screen-level handling
        let out = match self.screen {
            Screen::Connections => {
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: Vec::new(),
                    clicks: 1,
                };
                let (o, _) = self.connections.on_key(&key, &mut cx);
                let reqs = std::mem::take(&mut cx.requests);
                let o2 = self.apply_requests(reqs);
                o.or(o2)
            }
            Screen::Workbench => {
                let Some(mut w) = self.workbench.take() else {
                    return Outcome::Ignored;
                };
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: Vec::new(),
                    clicks: 1,
                };
                let o = w.on_key(&key, &mut cx, &mut self.history);
                let reqs = std::mem::take(&mut cx.requests);
                self.workbench = Some(w);
                let o2 = self.apply_requests(reqs);
                o.or(o2)
            }
        };
        if out.consumed() {
            if matches!(key.code, KeyCode::Enter | KeyCode::Char(' '))
                && key.plain()
                && !editing
                && let Some(f) = self.focus.current()
            {
                let now = self.now_ms();
                self.pointer.flash(f, now);
            }
            return out;
        }
        // global keys
        match key.code {
            KeyCode::Tab => {
                self.focus.next(&self.ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                self.focus.prev(&self.ring);
                Outcome::Changed
            }
            KeyCode::Char('?') => {
                self.open_help();
                Outcome::Changed
            }
            KeyCode::Char('q') if key.plain() && !editing => self.request_quit(),
            KeyCode::Esc => self.esc_ladder(),
            KeyCode::Char('0') if key.plain() && self.screen == Screen::Workbench => {
                if let Some(w) = self.workbench.as_mut() {
                    w.explorer_visible = true;
                    w.maximized = false;
                }
                self.focus.focus(crate::workbench::EXPLORER);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn workbench_chord(&mut self, key: &Key, editing: bool) -> Option<Outcome> {
        let running = self.workbench.as_ref().and_then(|w| w.running()).is_some();
        // Esc cancels a running query before anything else
        if key.is(KeyCode::Esc)
            && running
            && let Some(w) = self.workbench.as_mut()
        {
            for t in w.tabs.iter_mut() {
                if let WorkTab::Query(q) = t
                    && q.cancel()
                {
                    self.set_status("Query cancelled".into());
                    return Some(Outcome::Changed);
                }
            }
        }
        let alt = key.alt();
        let ctrl = key.ctrl();
        match key.code {
            KeyCode::Char('r') | KeyCode::Char('R') if ctrl => {
                return Some(self.run_active(false, None));
            }
            KeyCode::F(5) if !editing => {
                // F5 runs in a query tab, refreshes elsewhere
                if self.active_is_query() {
                    return Some(self.run_active(false, None));
                }
                return None;
            }
            KeyCode::Char('r') if alt => return Some(self.run_active(true, None)),
            KeyCode::Char('x') if ctrl => return Some(self.run_active(false, Some(false))),
            KeyCode::Char('x') if alt => return Some(self.run_active(false, Some(true))),
            KeyCode::Char('t') if ctrl => {
                if let Some(w) = self.workbench.as_mut() {
                    w.new_query("");
                    if let Some(pf) = w.primary_focus() {
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('w') if ctrl && !editing => {
                if let Some(w) = self.workbench.as_mut() {
                    let i = w.active;
                    if w.tabs.get(i).is_some_and(|t| t.dirty()) {
                        self.apply_requests(vec![Request::ConfirmCloseTab(i)]);
                    } else {
                        w.close_tab(i);
                        let pf = w.primary_focus().unwrap_or(crate::workbench::EXPLORER);
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('o') | KeyCode::Char('p') if ctrl => {
                self.open_switcher();
                return Some(Outcome::Changed);
            }
            KeyCode::Char('g') if ctrl => {
                self.open_tab_list();
                return Some(Outcome::Changed);
            }
            KeyCode::Char('y') if ctrl => {
                if let Some(w) = self.workbench.as_mut() {
                    w.open_history(&self.history);
                    if let Some(pf) = w.primary_focus() {
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('b') if ctrl => {
                if let Some(w) = self.workbench.as_mut() {
                    w.explorer_visible = !w.explorer_visible;
                    if w.explorer_visible {
                        self.focus.focus(crate::workbench::EXPLORER);
                    } else if self.focus.is(crate::workbench::EXPLORER)
                        && let Some(pf) = w.primary_focus()
                    {
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('l') if ctrl && !editing => {
                self.open_safe_mode_picker();
                return Some(Outcome::Changed);
            }
            KeyCode::Char('d') if ctrl && !editing => {
                if let Some(w) = self.workbench.as_mut()
                    && let Some(WorkTab::Table(t)) = w.active_tab_mut()
                {
                    t.mode_tabs
                        .set_active(if t.mode_tabs.active == 0 { 1 } else { 0 });
                    let cat = w.catalog.clone();
                    if let Some(WorkTab::Table(t)) = w.active_tab_mut() {
                        t.structure_refresh(&cat);
                        let pf = if t.mode_tabs.active == 1 {
                            t.structure.id
                        } else {
                            t.grid.id
                        };
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('s') if ctrl && !editing => {
                self.begin_commit();
                return Some(Outcome::Changed);
            }
            KeyCode::Char('f') if ctrl && !editing => {
                if self.active_is_query() {
                    if let Some(w) = self.workbench.as_mut()
                        && let Some(WorkTab::Query(q)) = w.active_tab_mut()
                    {
                        q.editor.open_find();
                        self.focus.focus(q.editor.id);
                    }
                } else {
                    self.open_filter_editor(None, None);
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('z') if key.plain() && !editing => {
                if let Some(w) = self.workbench.as_mut() {
                    w.maximized = !w.maximized;
                    let maximized = w.maximized;
                    let f = self.focus.current();
                    if let Some(WorkTab::Query(q)) = w.active_tab_mut() {
                        // maximize the pane that has focus inside a query tab
                        let target = if f == Some(q.editor.id) {
                            junie_tui::ui::layout::Maximized::First
                        } else {
                            junie_tui::ui::layout::Maximized::Second
                        };
                        q.split.maximized = if maximized {
                            target
                        } else {
                            junie_tui::ui::layout::Maximized::None
                        };
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char('[') if key.plain() && !editing => {
                if let Some(w) = self.workbench.as_mut()
                    && !w.tabs.is_empty()
                {
                    let i = (w.active + w.tabs.len() - 1) % w.tabs.len();
                    w.set_active(i);
                    if let Some(pf) = w.primary_focus()
                        && !self.focus.is(crate::workbench::EXPLORER)
                        && !self.focus.is(crate::workbench::TABSTRIP)
                    {
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Char(']') if key.plain() && !editing => {
                if let Some(w) = self.workbench.as_mut()
                    && !w.tabs.is_empty()
                {
                    let i = (w.active + 1) % w.tabs.len();
                    w.set_active(i);
                    if let Some(pf) = w.primary_focus()
                        && !self.focus.is(crate::workbench::EXPLORER)
                        && !self.focus.is(crate::workbench::TABSTRIP)
                    {
                        self.focus.focus(pf);
                    }
                }
                return Some(Outcome::Changed);
            }
            KeyCode::Up | KeyCode::Down if ctrl => {
                if let Some(w) = self.workbench.as_mut()
                    && let Some(WorkTab::Query(q)) = w.active_tab_mut()
                {
                    q.split.grow(if key.code == KeyCode::Up { -8 } else { 8 });
                }
                return Some(Outcome::Changed);
            }
            _ => {}
        }
        None
    }

    fn active_is_query(&self) -> bool {
        matches!(
            self.workbench.as_ref().and_then(|w| w.active_tab()),
            Some(WorkTab::Query(_))
        )
    }

    fn esc_ladder(&mut self) -> Outcome {
        match self.screen {
            Screen::Connections => Outcome::Consumed,
            Screen::Workbench => {
                let Some(w) = self.workbench.as_mut() else {
                    return Outcome::Ignored;
                };
                if w.maximized {
                    w.maximized = false;
                    if let Some(WorkTab::Query(q)) = w.active_tab_mut() {
                        q.split.maximized = junie_tui::ui::layout::Maximized::None;
                    }
                    return Outcome::Changed;
                }
                let f = self.focus.current();
                if f == Some(crate::workbench::TABSTRIP) {
                    self.focus.focus(crate::workbench::EXPLORER);
                    return Outcome::Changed;
                }
                if f == Some(crate::workbench::EXPLORER) {
                    if w.explorer_filter.text().is_empty() {
                        return Outcome::Consumed;
                    }
                    w.explorer_filter = TextInput::new(w.explorer_filter.id, "")
                        .placeholder("Filter objects")
                        .plain_label();
                    w.explorer.set_filter(None);
                    return Outcome::Changed;
                }
                self.focus.focus(crate::workbench::TABSTRIP);
                Outcome::Changed
            }
        }
    }

    // ---- execution + safety ------------------------------------------

    /// Run the current statement / all / explain in the active query tab,
    /// passing through the Safe Mode gate.
    pub fn run_active(&mut self, all: bool, explain: Option<bool>) -> Outcome {
        let Some(w) = self.workbench.as_mut() else {
            return Outcome::Ignored;
        };
        let level = w.connection.safe_mode;
        let env = w.connection.environment;
        let conn = w.connection.name.clone();
        let db = w.catalog.database.clone();
        let tab_index = w.active;
        let Some(WorkTab::Query(q)) = w.active_tab_mut() else {
            self.set_status("Open a query tab to run SQL (Ctrl+T)".into());
            return Outcome::Changed;
        };
        if q.is_running() {
            self.set_status("Already running · Esc cancels".into());
            return Outcome::Changed;
        }
        let statements = q.statements_to_run(all);
        if statements.is_empty() {
            self.set_status("Nothing to run".into());
            return Outcome::Changed;
        }
        // classify the batch by its worst statement (TablePro: OperationKind.worst)
        let mut worst: Option<(Decision, sql::Statement, String)> = None;
        let mut parse_errors = false;
        for (text, _) in &statements {
            let Ok(mut stmt) = sql::parse(text) else {
                parse_errors = true;
                continue;
            };
            if let Some(analyze) = explain {
                stmt = sql::Statement::Explain {
                    analyze,
                    inner: Box::new(stmt),
                };
            }
            let d = sql::gate(level, &stmt);
            let rank = |d: &Decision| match d {
                Decision::Run => 0,
                Decision::Confirm { deliberate: false } => 1,
                Decision::Confirm { deliberate: true } => 2,
                Decision::Deny => 3,
            };
            if worst.as_ref().is_none_or(|(wd, ws, _)| {
                rank(&d) > rank(wd) || (rank(&d) == rank(wd) && sql::tier(&stmt) > sql::tier(ws))
            }) {
                worst = Some((d, stmt, text.clone()));
            }
        }
        let _ = parse_errors;
        let Some((decision, stmt, text)) = worst else {
            // only syntax errors: run so the error surfaces in the result
            q.start(statements, all, explain);
            return Outcome::Changed;
        };
        match decision {
            Decision::Run => {
                q.start(statements, all, explain);
                Outcome::Changed
            }
            Decision::Deny => {
                self.set_status("Cannot execute write queries: TablePro's Safe Mode is set to read-only for this connection · Ctrl+L changes Safe Mode".into());
                Outcome::Changed
            }
            Decision::Confirm { deliberate } => {
                let table = w.catalog.find(None, stmt.target().unwrap_or(""));
                let risk = sql::assess(&stmt, table);
                w.pending_run = Some((tab_index, statements.clone(), all, explain));
                let tier = sql::tier(&stmt);
                let dangerous = sql::is_dangerous(&stmt);
                let title = if dangerous {
                    "This query may permanently modify or delete data"
                } else if tier == sql::Tier::Safe {
                    "Execute query?"
                } else {
                    "Execute write query?"
                };
                let env_label = format!("{} · {}", conn, env.label());
                let mut facts = vec![
                    Prop::new("Action", risk.action.clone()).tone(if dangerous {
                        Tone::Error
                    } else {
                        Tone::Normal
                    }),
                    Prop::new(
                        "Target",
                        format!(
                            "{env_label} · {db}{}",
                            stmt.target().map(|t| format!(" · {t}")).unwrap_or_default()
                        ),
                    )
                    .tone(if env == Environment::Production {
                        Tone::Normal
                    } else {
                        Tone::Secondary
                    }),
                ];
                if !risk.scope.is_empty() {
                    facts.push(
                        Prop::new("Scope", risk.scope.clone())
                            .tone(Tone::Secondary)
                            .wrap(),
                    );
                }
                if !risk.risk.is_empty() {
                    facts.push(
                        Prop::new("Risk", risk.risk.clone())
                            .tone(if dangerous {
                                Tone::Warning
                            } else {
                                Tone::Secondary
                            })
                            .wrap(),
                    );
                }
                if !risk.reversible.is_empty() {
                    facts.push(
                        Prop::new("Reversible", risk.reversible.to_owned())
                            .tone(Tone::Secondary)
                            .wrap(),
                    );
                }
                facts.push(
                    Prop::new(
                        "Safe Mode",
                        format!(
                            "{} · {}",
                            level.label(),
                            if deliberate {
                                "type to confirm"
                            } else {
                                "confirm before running"
                            }
                        ),
                    )
                    .tone(Tone::Muted),
                );
                let code: Vec<String> = statements
                    .iter()
                    .flat_map(|(s, _)| s.lines().map(|l| l.to_owned()).collect::<Vec<_>>())
                    .collect();
                let token = if deliberate || (dangerous && env == Environment::Production) {
                    Some(stmt.target().unwrap_or("yes").to_owned())
                } else {
                    None
                };
                let confirm = if dangerous || tier == sql::Tier::Destructive {
                    Button::danger(SAFETY_DIALOG.sub("ok"), "Execute")
                } else {
                    Button::primary(SAFETY_DIALOG.sub("ok"), "Execute")
                };
                let mut d =
                    Dialog::facts(SAFETY_DIALOG, title, facts, code, token.as_deref(), confirm);
                if token.is_none() {
                    // dangerous: Cancel keeps focus; plain writes focus Execute
                    d.initial_focus = if dangerous {
                        d.actions[0].id
                    } else {
                        d.actions[1].id
                    };
                }
                d.width = 74;
                let _ = text;
                self.open_dialog(d);
                Outcome::Changed
            }
        }
    }

    fn begin_commit(&mut self) {
        let Some(w) = self.workbench.as_mut() else {
            return;
        };
        let level = w.connection.safe_mode;
        let env = w.connection.environment;
        let conn = w.connection.name.clone();
        let db = w.catalog.database.clone();
        let (schema, name) = match w.active_tab() {
            Some(WorkTab::Table(t)) => (t.schema.clone(), t.name.clone()),
            _ => {
                self.set_status("Nothing to save".into());
                return;
            }
        };
        let table = w.catalog.find(Some(&schema), &name).cloned();
        let Some(table) = table else { return };
        let Some(WorkTab::Table(t)) = w.active_tab_mut() else {
            return;
        };
        if t.grid.pending.is_empty() {
            self.set_status("No pending changes".into());
            return;
        }
        if level == SafeMode::ReadOnly {
            self.set_status(
                "Cannot save changes: TablePro's Safe Mode is set to read-only for this connection"
                    .into(),
            );
            return;
        }
        let sqls = crate::model::preview_sql(&table, &t.columns, &t.grid);
        let (u, i, d) = t.grid.pending.counts();
        let deletes = d;
        let deliberate = level.requires_authentication();
        let mut facts = vec![
            Prop::new("Action", "Save changes".to_owned()),
            Prop::new(
                "Target",
                format!("{} · {} · {} · {}", conn, env.label(), db, t.qualified()),
            )
            .tone(if env == Environment::Production {
                Tone::Normal
            } else {
                Tone::Secondary
            }),
            Prop::new(
                "Scope",
                format!(
                    "{u} update{} · {i} insert{} · {d} delete{}",
                    if u == 1 { "" } else { "s" },
                    if i == 1 { "" } else { "s" },
                    if d == 1 { "" } else { "s" }
                ),
            )
            .tone(Tone::Secondary),
        ];
        if deletes > 0 {
            facts.push(
                Prop::new("Risk", "Deleted rows cannot be restored without a backup.")
                    .tone(Tone::Warning)
                    .wrap(),
            );
        }
        facts.push(
            Prop::new(
                "Transaction",
                "All statements run in one transaction; a failure rolls everything back.",
            )
            .tone(Tone::Muted)
            .wrap(),
        );
        facts.push(
            Prop::new(
                "Safe Mode",
                format!(
                    "{} · {}",
                    level.label(),
                    if deliberate {
                        "type to confirm"
                    } else if level.requires_confirmation() || deletes > 0 {
                        "confirm before running"
                    } else {
                        "runs after this review"
                    }
                ),
            )
            .tone(Tone::Muted),
        );
        let token = if deliberate {
            Some(t.name.clone())
        } else {
            None
        };
        let confirm = if deletes > 0 {
            Button::danger(COMMIT_DIALOG.sub("ok"), "Save")
        } else {
            Button::primary(COMMIT_DIALOG.sub("ok"), "Save")
        };
        let title = if deletes > 0 {
            format!(
                "Delete {deletes} row{} and save?",
                if deletes == 1 { "" } else { "s" }
            )
        } else {
            "Save changes?".to_owned()
        };
        let mut dlg = Dialog::facts(
            COMMIT_DIALOG,
            &title,
            facts,
            sqls,
            token.as_deref(),
            confirm,
        );
        if token.is_none() {
            dlg.initial_focus = if deletes > 0 {
                dlg.actions[0].id
            } else {
                dlg.actions[1].id
            };
        }
        dlg.width = 78;
        self.open_dialog(dlg);
    }

    fn finish_commit(&mut self) {
        let Some(w) = self.workbench.as_mut() else {
            return;
        };
        let conn = w.connection.name.clone();
        let db = w.catalog.database.clone();
        let (schema, name) = match w.active_tab() {
            Some(WorkTab::Table(t)) => (t.schema.clone(), t.name.clone()),
            _ => return,
        };
        let table = w.catalog.find(Some(&schema), &name).cloned();
        let Some(WorkTab::Table(t)) = w.active_tab_mut() else {
            return;
        };
        let sqls = table
            .as_ref()
            .map(|tb| crate::model::preview_sql(tb, &t.columns, &t.grid))
            .unwrap_or_default();
        let n = t.grid.pending.total();
        let qualified = t.qualified();
        let schema = t.schema.clone();
        t.grid.apply_commit_result(Ok(()));
        for s in sqls {
            self.history.push(crate::model::HistoryEntry {
                id: 0,
                sql: s,
                connection: conn.clone(),
                database: db.clone(),
                schema: schema.clone(),
                minutes_ago: 0,
                duration_ms: Some(3),
                rows: Some(1),
                error: None,
                source: crate::model::HistorySource::RowEdits,
            });
        }
        self.set_status(format!(
            "Saved {n} change{} to {qualified}",
            if n == 1 { "" } else { "s" }
        ));
    }

    fn open_preview(&mut self) {
        let Some(w) = self.workbench.as_ref() else {
            return;
        };
        let Some(WorkTab::Table(t)) = w.active_tab() else {
            return;
        };
        let Some(table) = w.catalog.find(Some(&t.schema), &t.name) else {
            return;
        };
        let sqls = crate::model::preview_sql(table, &t.columns, &t.grid);
        let n = sqls.len();
        let mut d = Dialog::facts(
            PREVIEW_DIALOG,
            &format!(
                "SQL preview · {n} statement{}",
                if n == 1 { "" } else { "s" }
            ),
            vec![Prop::new("Applied on", "Save · one transaction").tone(Tone::Muted)],
            sqls,
            None,
            Button::secondary(PREVIEW_DIALOG.sub("close"), "Close"),
        );
        d.actions.remove(0);
        d.cancel_index = Some(0);
        d.initial_focus = d.actions[0].id;
        d.width = 90;
        self.open_dialog(d);
    }

    // ---- overlays ------------------------------------------------------

    fn open_dialog(&mut self, d: Dialog) {
        let initial = Some(d.initial_focus);
        self.modals.push(Modal::Dialog(d), &mut self.focus, initial);
        self.pointer.hover = None;
        self.pointer.pressed = None;
    }

    fn open_help(&mut self) {
        // a key reference: wrapped descriptions hang under their column
        let text = junie_tui::widgets::keyref::KeyReference::new()
            .key("Tab / Shift+Tab", "move focus · 0 explorer · Esc back")
            .key("Ctrl+O", "Open Quickly (tables, schemas, tabs, queries)")
            .key(
                "Ctrl+T / Ctrl+W",
                "new / close tab · [ ] switch · Ctrl+G tab list",
            )
            .key("Ctrl+R / F5", "run statement at cursor · Alt+R run all")
            .key("Ctrl+X / Alt+X", "EXPLAIN / EXPLAIN ANALYZE")
            .key("Esc / Ctrl+C", "cancel a running query")
            .key(
                "Ctrl+D",
                "Data / Structure · Ctrl+F filter (grid) or find (editor)",
            )
            .key("Alt+D", "duplicate the current grid row")
            .key("Ctrl+S", "save pending row changes · p preview SQL")
            .key(
                "Ctrl+Y",
                "query history · Ctrl+L Safe Mode · Ctrl+B explorer · z zoom",
            )
            .key(
                "Shift+F10",
                "menu of the focused row, cell or tab · right-click at the pointer",
            )
            .key("q", "quit")
            .text();
        let mut d = Dialog::confirm(HELP_DIALOG, "Keyboard", &text, "Close");
        d.actions.remove(0);
        d.cancel_index = Some(0);
        d.initial_focus = d.actions[0].id;
        d.actions[0].kind = junie_tui::theme::ButtonKind::Secondary;
        d.width = 78;
        self.open_dialog(d);
    }

    fn open_switcher(&mut self) {
        let Some(w) = self.workbench.as_ref() else {
            return;
        };
        let open: Vec<(usize, String)> = w
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.label()))
            .collect();
        let index = SwitcherIndex::build(&w.catalog, &w.connection.name, &open, &self.history);
        let mut p = Picker::new(WidgetId::of("switcher"), "Open Quickly");
        p.placeholder = "Search tables, views, schemas, tabs, queries…".into();
        p.width = 88;
        self.scope = 0;
        p.scope = Some("All · Tab scope".into());
        let keep = self.focus.current();
        self.modals.push(
            Modal::Picker(PickerKind::Switcher, p, Some(index)),
            &mut self.focus,
            keep,
        );
        self.refresh_switcher();
    }

    fn refresh_switcher(&mut self) {
        let Some(Modal::Picker(PickerKind::Switcher, p, Some(index))) = self.modals.top_mut()
        else {
            return;
        };
        let scopes = ["All", "Tables", "Schemas", "Queries"];
        p.scope = Some(format!("{} · Tab scope", scopes[self.scope]));
        let mut results = index.query(&p.query);
        results.retain(|r| match self.scope {
            1 => r.group == "Tables" || r.group == "Views",
            2 => r.group == "Schemas" || r.group == "Databases",
            3 => r.group == "Recent queries",
            _ => true,
        });
        if p.query.is_empty() && self.scope == 0 {
            // empty query: recent objects first, keep it short
            results.truncate(24);
        }
        let items: Vec<PickerItem> = results
            .iter()
            .map(|r| PickerItem {
                label: r.label.clone(),
                detail: r.path.clone(),
                glyph: match r.group {
                    "Tables" => "T",
                    "Views" => "V",
                    "Schemas" => "S",
                    "Databases" => "D",
                    "Open tabs" => "≡",
                    _ => "Q",
                },
                group: r.group,
                tag: if r.open && r.group != "Open tabs" {
                    Some("open")
                } else {
                    None
                },
                matched: r.matched.clone(),
                disabled: false,
                key: String::new(),
            })
            .collect();
        p.set_items(items);
        // stash the matched targets in order (parallel vector via index)
        self.switcher_targets = results.into_iter().map(|r| r.target).collect();
    }

    fn open_tab_list(&mut self) {
        let Some(w) = self.workbench.as_ref() else {
            return;
        };
        let mut p = Picker::new(WidgetId::of("tablist"), "Open tabs");
        p.placeholder = "Filter tabs…".into();
        let items: Vec<PickerItem> = w
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| PickerItem {
                label: t.label(),
                detail: match t {
                    WorkTab::Table(tt) => format!(
                        "{} · {}",
                        tt.qualified(),
                        if tt.mode_tabs.active == 0 {
                            "data"
                        } else {
                            "structure"
                        }
                    ),
                    WorkTab::Query(q) => {
                        if q.is_running() {
                            "running".into()
                        } else {
                            q.last_status
                                .as_ref()
                                .map(|s| s.0.clone())
                                .unwrap_or("query".into())
                        }
                    }
                    WorkTab::History(_) => "history".into(),
                },
                glyph: match t {
                    WorkTab::Table(_) => "T",
                    WorkTab::Query(_) => "≡",
                    WorkTab::History(_) => "H",
                },
                group: "",
                tag: if i == w.active {
                    Some("active")
                } else if t.dirty() {
                    Some("unsaved")
                } else {
                    None
                },
                matched: vec![],
                disabled: false,
                key: String::new(),
            })
            .collect();
        p.set_items(items);
        p.cursor = w.active;
        let keep = self.focus.current();
        self.modals.push(
            Modal::Picker(PickerKind::TabList, p, None),
            &mut self.focus,
            keep,
        );
    }

    fn open_safe_mode_picker(&mut self) {
        let Some(w) = self.workbench.as_ref() else {
            return;
        };
        let current = w.connection.safe_mode;
        let mut p = Picker::new(WidgetId::of("safemode"), "Safe Mode · this connection");
        p.searchable = false;
        p.width = 112;
        let items: Vec<PickerItem> = SafeMode::ALL
            .iter()
            .map(|s| PickerItem {
                label: s.label().to_owned(),
                detail: s.description().to_owned(),
                glyph: if *s == current { "›" } else { " " },
                group: "",
                tag: if *s == current { Some("current") } else { None },
                matched: vec![],
                disabled: false,
                key: String::new(),
            })
            .collect();
        p.set_items(items);
        p.cursor = SafeMode::ALL
            .iter()
            .position(|s| *s == current)
            .unwrap_or(0);
        p.empty_text = String::new();
        let keep = self.focus.current();
        self.modals.push(
            Modal::Picker(PickerKind::SafeMode, p, None),
            &mut self.focus,
            keep,
        );
    }

    fn open_filter_editor(
        &mut self,
        index: Option<usize>,
        prefill: Option<(usize, FilterOp, String)>,
    ) {
        let Some(w) = self.workbench.as_ref() else {
            return;
        };
        let Some(WorkTab::Table(t)) = w.active_tab() else {
            self.set_status("Filters apply to table tabs".into());
            return;
        };
        let columns = t.columns.clone();
        let names: Vec<&str> = columns.iter().map(|c| c.0.as_str()).collect();
        let existing = index.and_then(|i| t.filters.get(i)).cloned();
        let (col_i, op, value, value2) = match (&existing, &prefill) {
            (Some(f), _) => (
                columns.iter().position(|c| c.0 == f.column).unwrap_or(0),
                f.op,
                f.value.clone(),
                f.value2.clone(),
            ),
            (None, Some((c, op, v))) => (*c, *op, v.clone(), String::new()),
            _ => (
                t.grid.cursor.1.min(columns.len().saturating_sub(1)),
                FilterOp::Eq,
                String::new(),
                String::new(),
            ),
        };
        let ops = FilterOp::ordered_for(columns[col_i].1);
        let op_labels: Vec<&str> = ops.iter().map(|o| o.label()).collect();
        let op_i = ops.iter().position(|o| *o == op).unwrap_or(0);
        let f = WidgetId::of("filter-editor");
        let editor = FilterEditor {
            index,
            column: Select::new(f.sub("col"), "Column", &names, col_i),
            op: Select::new(f.sub("op"), "Operator", &op_labels, op_i)
                .help("Operators that fit the column type come first"),
            value: TextInput::new(f.sub("value"), "Value")
                .value(&value)
                .placeholder("value")
                .plain_label(),
            value2: TextInput::new(f.sub("value2"), "and")
                .value(&value2)
                .plain_label(),
            apply: Button::primary(
                f.sub("apply"),
                if existing.is_some() {
                    "Update filter"
                } else {
                    "Add filter"
                },
            ),
            cancel: Button::subtle(f.sub("cancel"), "Cancel"),
            area: Rect::ZERO,
            columns,
            dismiss: Dismiss::ESCAPE,
        };
        let initial = Some(if value.is_empty() {
            editor.value.id
        } else {
            editor.apply.id
        });
        self.modals
            .push(Modal::Filter(editor), &mut self.focus, initial);
    }

    fn close_modal(&mut self) {
        self.modals.pop(&mut self.focus);
    }

    /// The topmost overlay.
    #[cfg(test)]
    pub fn modal(&self) -> Option<&Modal> {
        self.modals.top()
    }

    fn modal_key(&mut self, key: Key) -> Outcome {
        let Some(modal) = self.modals.top_mut() else {
            return Outcome::Ignored;
        };
        match modal {
            Modal::Dialog(d) => {
                let out = d.on_key(&key, &mut self.focus, &self.ring);
                if let Some(result) = d.result {
                    let id = d.id;
                    let value = match &d.body {
                        DialogBody::Input(i) => Some(i.text().to_owned()),
                        _ => None,
                    };
                    self.close_modal();
                    return self.dialog_closed(id, result, value).or(out);
                }
                out.or(Outcome::Consumed)
            }
            Modal::Picker(kind, p, _) => {
                let kind = *kind;
                let (o, ev) = p.on_key(&key);
                match ev {
                    Some(PickerEvent::QueryChanged) => {
                        if kind == PickerKind::Switcher {
                            self.refresh_switcher();
                        } else if kind == PickerKind::TabList {
                            // simple substring filter
                            let q = p.query.to_lowercase();
                            let w = self.workbench.as_ref().unwrap();
                            let items: Vec<PickerItem> = w
                                .tabs
                                .iter()
                                .enumerate()
                                .filter(|(_, t)| {
                                    q.is_empty() || t.label().to_lowercase().contains(&q)
                                })
                                .map(|(i, t)| PickerItem {
                                    label: t.label(),
                                    detail: i.to_string(),
                                    glyph: "≡",
                                    group: "",
                                    tag: if i == w.active { Some("active") } else { None },
                                    matched: vec![],
                                    disabled: false,
                                    key: String::new(),
                                })
                                .collect();
                            p.set_items(items);
                        }
                        Outcome::Changed
                    }
                    Some(PickerEvent::NextScope) => {
                        if kind == PickerKind::Switcher {
                            self.scope = (self.scope + 1) % 4;
                            self.refresh_switcher();
                        }
                        Outcome::Changed
                    }
                    Some(PickerEvent::Chosen(i)) | Some(PickerEvent::ChosenAlt(i)) => {
                        let alt = matches!(ev, Some(PickerEvent::ChosenAlt(_)));
                        self.picker_chosen(kind, i, alt)
                    }
                    Some(PickerEvent::Secondary(i)) => {
                        if kind == PickerKind::TabList {
                            if let Some(w) = self.workbench.as_mut() {
                                let idx = p
                                    .items
                                    .get(i)
                                    .and_then(|it| it.detail.parse::<usize>().ok())
                                    .unwrap_or(i);
                                w.close_tab(idx);
                            }
                            self.close_modal();
                            self.open_tab_list();
                        }
                        Outcome::Changed
                    }
                    Some(PickerEvent::Cancelled) => {
                        self.close_modal();
                        Outcome::Changed
                    }
                    // a query that matches nothing is not a command: the
                    // picker stays open with it
                    Some(PickerEvent::Back) | Some(PickerEvent::Submit) | None => {
                        o.or(Outcome::Consumed)
                    }
                }
            }
            Modal::Menu(m) => {
                let (o, ev) = m.menu.on_key(&key);
                match ev {
                    Some(MenuEvent::Chosen(i)) => self.context_chosen(i),
                    Some(MenuEvent::Dismissed) => {
                        self.close_modal();
                        Outcome::Changed
                    }
                    None => o.or(Outcome::Consumed),
                }
            }
            Modal::Filter(f) => {
                let out = Self::filter_key(f, &key, &mut self.focus, &self.ring);
                match out {
                    FilterOutcome::Keep(o) => o,
                    FilterOutcome::Apply => self.apply_filter(),
                    FilterOutcome::Cancel => {
                        self.close_modal();
                        Outcome::Changed
                    }
                }
            }
        }
    }

    fn picker_chosen(&mut self, kind: PickerKind, i: usize, alt: bool) -> Outcome {
        match kind {
            PickerKind::Switcher => {
                let target = self.switcher_targets.get(i).cloned();
                self.close_modal();
                let Some(target) = target else {
                    return Outcome::Changed;
                };
                let Some(w) = self.workbench.as_mut() else {
                    return Outcome::Changed;
                };
                match target {
                    SwitchTarget::Table { schema, name } | SwitchTarget::View { schema, name } => {
                        w.open_table(&schema, &name, false);
                        let _ = alt;
                        if let Some(pf) = w.primary_focus() {
                            self.focus.focus(pf);
                        }
                        // reveal in the explorer
                        self.set_status(format!("Opened {schema}.{name}"));
                    }
                    SwitchTarget::Schema(s) => {
                        w.schema = s.clone();
                        w.explorer_filter = TextInput::new(w.explorer_filter.id, "")
                            .placeholder("Filter objects")
                            .plain_label();
                        w.explorer.set_filter(None);
                        let _ = w.explorer.rows();
                        // rebuild explorer with the new current schema
                        let conn = w.connection.clone();
                        let cat = w.catalog.clone();
                        let tabs = std::mem::take(&mut w.tabs);
                        let active = w.active;
                        let qc = w.query_counter;
                        let mut nw = Workbench::new(conn, cat);
                        nw.tabs = tabs;
                        nw.active = active;
                        nw.query_counter = qc;
                        nw.schema = s.clone();
                        nw.rebuild_for_schema();
                        *w = nw;
                        self.focus.focus(crate::workbench::EXPLORER);
                        self.set_status(format!("Schema {s}"));
                    }
                    SwitchTarget::Database(_) => {
                        self.focus.focus(crate::workbench::EXPLORER);
                    }
                    SwitchTarget::OpenTab(i) => {
                        w.set_active(i);
                        if let Some(pf) = w.primary_focus() {
                            self.focus.focus(pf);
                        }
                    }
                    SwitchTarget::RecentQuery(id) => {
                        let sql = self
                            .history
                            .entries
                            .iter()
                            .find(|e| e.id == id)
                            .map(|e| e.sql.clone())
                            .unwrap_or_default();
                        w.new_query(&sql);
                        if let Some(pf) = w.primary_focus() {
                            self.focus.focus(pf);
                        }
                    }
                }
                Outcome::Changed
            }
            PickerKind::TabList => {
                let idx = match self.modals.top() {
                    Some(Modal::Picker(_, p, _)) => p
                        .items
                        .get(i)
                        .and_then(|it| it.detail.parse::<usize>().ok())
                        .unwrap_or(i),
                    _ => i,
                };
                self.close_modal();
                if let Some(w) = self.workbench.as_mut() {
                    w.set_active(idx);
                    if let Some(pf) = w.primary_focus() {
                        self.focus.focus(pf);
                    }
                }
                Outcome::Changed
            }
            PickerKind::SafeMode => {
                self.close_modal();
                let level = SafeMode::ALL[i.min(5)];
                if let Some(w) = self.workbench.as_mut() {
                    w.connection.safe_mode = level;
                    let name = w.connection.name.clone();
                    if let Some(c) = self
                        .connections
                        .connections
                        .iter_mut()
                        .find(|c| c.name == name)
                    {
                        c.safe_mode = level;
                    }
                }
                self.set_status(format!(
                    "Safety level set to {} · saved to the connection",
                    level.label()
                ));
                Outcome::Changed
            }
        }
    }

    fn filter_key(
        f: &mut FilterEditor,
        key: &Key,
        focus: &mut Focus,
        ring: &FocusRing,
    ) -> FilterOutcome {
        use junie_tui::widgets::input::InputEvent;
        let cur = focus.current();
        if key.is(KeyCode::Esc)
            && f.dismiss.escape
            && !f.value.editing
            && !f.value2.editing
            && !f.column.open
            && !f.op.open
        {
            return FilterOutcome::Cancel;
        }
        // focus traversal wins whenever no text field is editing
        if !f.value.editing && !f.value2.editing && !f.column.open && !f.op.open {
            match key.code {
                KeyCode::Tab => {
                    focus.next(ring);
                    return FilterOutcome::Keep(Outcome::Changed);
                }
                KeyCode::BackTab => {
                    focus.prev(ring);
                    return FilterOutcome::Keep(Outcome::Changed);
                }
                _ => {}
            }
        }
        if cur == Some(f.column.id) {
            let (o, ev) = f.column.on_key(key);
            if let Some(junie_tui::widgets::select::SelectEvent::Changed(i)) = ev {
                let ops = FilterOp::ordered_for(f.columns[i].1);
                let labels: Vec<&str> = ops.iter().map(|o| o.label()).collect();
                f.op = Select::new(f.op.id, "Operator", &labels, 0)
                    .help("Operators that fit the column type come first");
            }
            return FilterOutcome::Keep(o.or(Outcome::Consumed));
        }
        if cur == Some(f.op.id) {
            return FilterOutcome::Keep(f.op.on_key(key).0.or(Outcome::Consumed));
        }
        for inp in [&mut f.value, &mut f.value2] {
            if cur == Some(inp.id) {
                let (o, ev) = inp.on_key(key);
                match ev {
                    Some(InputEvent::CommittedTab { backward }) => {
                        if backward {
                            focus.prev(ring)
                        } else {
                            focus.next(ring)
                        }
                    }
                    Some(InputEvent::Committed) => return FilterOutcome::Apply,
                    _ => {}
                }
                if !o.consumed() && key.is(KeyCode::Enter) {
                    return FilterOutcome::Apply;
                }
                return FilterOutcome::Keep(o.or(Outcome::Consumed));
            }
        }
        if cur == Some(f.apply.id) {
            let (o, act) = f.apply.on_key(key);
            if act {
                return FilterOutcome::Apply;
            }
            return FilterOutcome::Keep(o.or(Outcome::Consumed));
        }
        if cur == Some(f.cancel.id) {
            let (o, act) = f.cancel.on_key(key);
            if act {
                return FilterOutcome::Cancel;
            }
            return FilterOutcome::Keep(o.or(Outcome::Consumed));
        }
        match key.code {
            KeyCode::Tab => {
                focus.next(ring);
                FilterOutcome::Keep(Outcome::Changed)
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                FilterOutcome::Keep(Outcome::Changed)
            }
            _ => FilterOutcome::Keep(Outcome::Consumed),
        }
    }

    fn apply_filter(&mut self) -> Outcome {
        let Some(Modal::Filter(f)) = self.modals.top_mut() else {
            return Outcome::Ignored;
        };
        let ops = FilterOp::ordered_for(f.columns[f.column.selected].1);
        let op = ops[f.op.selected.min(ops.len() - 1)];
        if f.value.editing {
            f.value.commit();
        }
        if f.value2.editing {
            f.value2.commit();
        }
        if op.needs_value() && f.value.text().trim().is_empty() {
            f.value.error = Some("A value is required for this operator".into());
            self.focus.focus(f.value.id);
            return Outcome::Changed;
        }
        let filter = Filter {
            column: f.columns[f.column.selected].0.clone(),
            op,
            value: f.value.text().trim().to_owned(),
            value2: f.value2.text().trim().to_owned(),
            enabled: true,
        };
        let index = f.index;
        self.close_modal();
        let mut applied = None;
        if let Some(w) = self.workbench.as_mut() {
            let cat = w.catalog.clone();
            if let Some(WorkTab::Table(t)) = w.active_tab_mut() {
                match index {
                    Some(i) if i < t.filters.len() => t.filters[i] = filter,
                    _ => t.filters.push(filter),
                }
                t.preview = false;
                t.load(&cat);
                let n = t.filters.iter().filter(|f| f.enabled).count();
                applied = Some((n, t.grid.id));
            }
        }
        if let Some((n, gid)) = applied {
            self.set_status(format!(
                "{n} filter{} applied",
                if n == 1 { "" } else { "s" }
            ));
            self.focus.focus(gid);
        }
        Outcome::Changed
    }

    fn dialog_closed(
        &mut self,
        id: WidgetId,
        result: DialogResult,
        _value: Option<String>,
    ) -> Outcome {
        if self.connections.on_dialog_closed(id, result) {
            return Outcome::Changed;
        }
        if id == QUIT_DIALOG {
            if result == DialogResult::Action(1) {
                self.quit = true;
            }
            return Outcome::Changed;
        }
        if id == SAFETY_DIALOG {
            let Some(w) = self.workbench.as_mut() else {
                return Outcome::Changed;
            };
            let pending = w.pending_run.take();
            if result == DialogResult::Action(1) {
                if let Some((tab, statements, all, explain)) = pending
                    && let Some(WorkTab::Query(q)) = w.tabs.get_mut(tab)
                {
                    q.start(statements, all, explain);
                }
            } else {
                self.set_status("Cancelled · nothing was executed".into());
            }
            return Outcome::Changed;
        }
        if id == COMMIT_DIALOG {
            if result == DialogResult::Action(1) {
                self.committing = Some(5);
                self.set_status("Saving…".into());
            } else {
                self.set_status("Changes kept pending".into());
            }
            return Outcome::Changed;
        }
        if id == DISCARD_DIALOG {
            if result == DialogResult::Action(1) {
                if let Some(w) = self.workbench.as_mut() {
                    let cat = w.catalog.clone();
                    if let Some(WorkTab::Table(t)) = w.active_tab_mut() {
                        t.grid.discard();
                        t.load(&cat);
                    }
                }
                self.set_status("Changes discarded".into());
            }
            return Outcome::Changed;
        }
        if let Some(w) = self.workbench.as_mut() {
            for i in 0..w.tabs.len() + 1 {
                if id == CLOSE_DIALOG.child(i) {
                    if result == DialogResult::Action(1) {
                        w.close_tab(i);
                        let pf = w.primary_focus().unwrap_or(crate::workbench::EXPLORER);
                        self.focus.focus(pf);
                    }
                    return Outcome::Changed;
                }
            }
        }
        Outcome::Changed
    }

    // ---------------------------------------------------------------- mouse

    /// A button release at `m.pos` over `hit`.
    fn on_release(&mut self, m: Mouse, hit: Option<WidgetId>) -> Outcome {
        let release = self.pointer.release(hit);
        let Release::Click(id) = release else {
            // click outside a modal cancels it (dialogs decide themselves)
            if let Some(modal) = self.modals.top_mut()
                && release == Release::Outside
            {
                match modal {
                    Modal::Dialog(d) => {
                        let out = d.on_click_outside();
                        if let Some(result) = d.result {
                            let did = d.id;
                            self.close_modal();
                            return self.dialog_closed(did, result, None).or(out);
                        }
                    }
                    // everything else follows its own dismiss policy
                    _ => {
                        if self.modals.outside_click() == Dismissal::Close {
                            self.close_modal();
                        }
                        return Outcome::Changed;
                    }
                }
            }
            return Outcome::Changed;
        };
        let now = self.now_ms();
        self.pointer.flash(id, now);
        // multi-clicks count on the page and in modals, by control, cell
        // and time; fields select a word or a line with them
        let clicks = self.pointer.count_click(id, m.pos, now);
        if let Some(modal) = self.modals.top_mut() {
            return match modal {
                Modal::Dialog(d) => {
                    let out = d.on_click(id, m.pos, &mut self.focus);
                    if let Some(result) = d.result {
                        let did = d.id;
                        let value = match &d.body {
                            DialogBody::Input(i) => Some(i.text().to_owned()),
                            _ => None,
                        };
                        self.close_modal();
                        return self.dialog_closed(did, result, value).or(out);
                    }
                    out.or(Outcome::Changed)
                }
                Modal::Picker(kind, p, _) => {
                    let kind = *kind;
                    if let Some(PickerEvent::Chosen(i)) = p.on_click(id) {
                        return self.picker_chosen(kind, i, false);
                    }
                    Outcome::Changed
                }
                Modal::Menu(menu) => match menu.menu.on_click(id) {
                    Some(MenuEvent::Chosen(i)) => self.context_chosen(i),
                    Some(MenuEvent::Dismissed) => {
                        self.close_modal();
                        Outcome::Changed
                    }
                    None => Outcome::Changed,
                },
                Modal::Filter(f) => {
                    let o = Self::filter_click(f, id, m.pos, clicks, &mut self.focus);
                    match o {
                        FilterOutcome::Keep(o) => o,
                        FilterOutcome::Apply => self.apply_filter(),
                        FilterOutcome::Cancel => {
                            self.close_modal();
                            Outcome::Changed
                        }
                    }
                }
            };
        }
        if id == STRIP_HELP {
            self.open_help();
            return Outcome::Changed;
        }
        if id == STRIP_SAFE {
            self.open_safe_mode_picker();
            return Outcome::Changed;
        }
        if id == STRIP_SCOPE {
            self.open_switcher();
            self.scope = 2;
            self.refresh_switcher();
            return Outcome::Changed;
        }
        if id == STRIP_CONN {
            if self.screen == Screen::Workbench {
                self.open_tab_list();
            }
            return Outcome::Changed;
        }
        match self.screen {
            Screen::Connections => {
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: Vec::new(),
                    clicks,
                };
                let (o, _) = self.connections.on_click(id, m.pos, &mut cx);
                let reqs = std::mem::take(&mut cx.requests);
                let o2 = self.apply_requests(reqs);
                o.or(o2).or(Outcome::Changed)
            }
            Screen::Workbench => {
                let Some(mut w) = self.workbench.take() else {
                    return Outcome::Changed;
                };
                let mut cx = Cx {
                    focus: &mut self.focus,
                    ring: &self.ring,
                    requests: Vec::new(),
                    clicks,
                };
                let o = w.on_click(id, m.pos, &mut cx, &mut self.history);
                let reqs = std::mem::take(&mut cx.requests);
                self.workbench = Some(w);
                let o2 = self.apply_requests(reqs);
                o.or(o2).or(Outcome::Changed)
            }
        }
    }

    fn on_mouse(&mut self, m: Mouse) -> Outcome {
        let before = self.focus.current();
        let out = self.route_mouse(m);
        // focus a click moved shows only on controls that want a ring
        // after a click (FocusVisible)
        let pressed = m.kind == MouseKind::Down && self.pointer.pressed.is_some();
        if pressed || (m.kind == MouseKind::Up && self.focus.current() != before) {
            self.pointer.focused_by_click();
        }
        out
    }

    fn route_mouse(&mut self, m: Mouse) -> Outcome {
        self.pointer.pos = Some(m.pos);
        match m.kind {
            MouseKind::Move => self.pointer.on_move(&self.hits, m.pos),
            MouseKind::Drag => {
                let Some(pressed) = self.pointer.on_drag(&self.hits, m.pos) else {
                    return Outcome::Ignored;
                };
                if let Some(Modal::Dialog(d)) = self.modals.top_mut() {
                    return d.on_drag(pressed, m.pos);
                }
                if let Some(Modal::Filter(f)) = self.modals.top_mut() {
                    return route_drag(&mut [&mut f.value, &mut f.value2], pressed, m.pos)
                        .unwrap_or(Outcome::Consumed);
                }
                if !self.modals.is_empty() {
                    return Outcome::Consumed;
                }
                match self.screen {
                    Screen::Workbench => self
                        .workbench
                        .as_mut()
                        .map(|w| w.on_drag(pressed, m.pos))
                        .unwrap_or(Outcome::Ignored),
                    Screen::Connections => self.connections.on_drag(pressed, m.pos),
                }
            }
            MouseKind::Down => {
                let Some(id) = self.pointer.on_down(&self.hits, m.pos) else {
                    return if !self.modals.is_empty() {
                        Outcome::Consumed
                    } else {
                        self.pointer.dead_space(&mut self.focus);
                        Outcome::Ignored
                    };
                };
                // a press on a modal's own body is dead space inside it:
                // it ends the focused field's edit in place
                if !self.modals.is_empty() {
                    self.pointer.surface_press(&self.hits, id, &mut self.focus);
                }
                let count = self.pointer.press_count(id, m.pos, self.now_ms());
                if let Some(Modal::Filter(f)) = self.modals.top_mut() {
                    route_press(&mut [&mut f.value, &mut f.value2], id, m.pos, count);
                }
                if !self.modals.is_empty() {
                    return Outcome::Changed;
                }
                if self.pointer.config.focus_on == FocusOn::Press && self.ring.contains(id) {
                    self.focus.focus(id);
                }
                // a field remembers the press for the drag; the second
                // press of a double click selects the word
                match self.screen {
                    Screen::Connections => self.connections.on_press(id, m.pos, count),
                    Screen::Workbench => self
                        .workbench
                        .as_mut()
                        .map_or(Outcome::Ignored, |w| w.on_press(id, m.pos, count)),
                }
                .or(Outcome::Changed)
            }
            MouseKind::Up => {
                // a pass-through backdrop: the click closes the popup and
                // then lands on what is beneath it, as a click of its own
                let (hit, beneath) = self.pointer.on_up(&self.hits, m.pos);
                let out = self.on_release(m, hit);
                match beneath {
                    Some(below) => {
                        self.pointer.press(below);
                        out.or(self.on_release(m, Some(below)))
                    }
                    None => out,
                }
            }
            MouseKind::Secondary => match self.pointer.route_secondary(&self.hits, m.pos) {
                SecondaryRoute::Nothing => {
                    if self.modals.is_empty() {
                        Outcome::Ignored
                    } else {
                        Outcome::Consumed
                    }
                }
                SecondaryRoute::Swallowed => Outcome::Consumed,
                SecondaryRoute::Target(id) => self.on_secondary(m, id),
                // a right-click outside an open popup: the click on its
                // backdrop closes it (through its own Dismiss policy, as a
                // primary click would), then the control beneath gets the
                // secondary click at the same spot
                SecondaryRoute::Outside { backdrop, reopen } => {
                    self.pointer.press(backdrop);
                    let out = self.on_release(m, Some(backdrop));
                    match reopen {
                        Some(id) => out.or(self.on_secondary(m, id)),
                        None => out,
                    }
                }
            },
            MouseKind::Middle => Outcome::Ignored,
            MouseKind::WheelUp
            | MouseKind::WheelDown
            | MouseKind::WheelLeft
            | MouseKind::WheelRight => {
                let horizontal = matches!(m.kind, MouseKind::WheelLeft | MouseKind::WheelRight);
                let delta = self.pointer.wheel_delta(m.kind);
                // a sideways wheel reaches only horizontal scrolling
                if let Some(Modal::Picker(_, p, _)) = self.modals.top_mut() {
                    return if horizontal {
                        Outcome::Consumed
                    } else {
                        p.on_wheel(delta)
                    };
                }
                if let Some(Modal::Dialog(d)) = self.modals.top_mut() {
                    return if horizontal {
                        d.on_wheel_h(delta)
                    } else {
                        d.on_wheel(delta)
                    };
                }
                if !self.modals.is_empty() {
                    return Outcome::Consumed;
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                let out = match self.screen {
                    Screen::Workbench => self
                        .workbench
                        .as_mut()
                        .map(|w| w.on_wheel(id, delta, horizontal))
                        .unwrap_or(Outcome::Ignored),
                    Screen::Connections if !horizontal => self.connections.on_wheel(id, delta),
                    Screen::Connections => Outcome::Ignored,
                };
                // rows move under a still pointer: re-resolve hover next frame
                self.pointer.scrolled(m.pos, out);
                out
            }
        }
    }

    /// A secondary click on `id`: the context menu of the row, cell or tab
    /// under it. Dialogs and the other overlays have none.
    fn on_secondary(&mut self, m: Mouse, id: WidgetId) -> Outcome {
        if !self.modals.is_empty() {
            return Outcome::Consumed;
        }
        let cfg = self.context;
        let menu = match (self.screen, self.workbench.as_mut()) {
            (Screen::Workbench, Some(w)) => {
                let menu = context::on_secondary(w, &cfg, id, m.pos);
                // the detail pane follows the row the menu is about
                if let Some(WorkTab::History(h)) = w.active_tab_mut() {
                    h.sync_detail_public(&self.history);
                }
                menu
            }
            (Screen::Connections, _) => {
                context::on_secondary_connections(&mut self.connections, &cfg, id, m.pos)
            }
            _ => None,
        };
        match menu {
            Some(menu) => self.open_context(menu),
            None => Outcome::Ignored,
        }
    }

    /// Open a context menu; its commands act on `menu.owner`, which takes
    /// the focus the menu returns to.
    fn open_context(&mut self, menu: CtxMenu) -> Outcome {
        self.focus.focus(menu.owner);
        let keep = self.focus.current();
        self.modals
            .push(Modal::Menu(Box::new(menu)), &mut self.focus, keep);
        self.pointer.pressed = None;
        Outcome::Changed
    }

    /// Item `i` of the open context menu: close it, then replay the key
    /// the item stands for on the widget it is about.
    fn context_chosen(&mut self, i: usize) -> Outcome {
        let Some(Modal::Menu(m)) = self.modals.top() else {
            return Outcome::Ignored;
        };
        let (key, target, owner) = (m.keys.get(i).copied().flatten(), m.target, m.owner);
        self.close_modal();
        let Some(key) = key else {
            return Outcome::Changed;
        };
        match target {
            context::Target::Connection(row) => self.connections.select_row(row),
            _ => {
                if let Some(w) = self.workbench.as_mut() {
                    context::select(w, target);
                }
            }
        }
        self.focus.focus(owner);
        self.on_key(key).or(Outcome::Changed)
    }

    fn filter_click(
        f: &mut FilterEditor,
        id: WidgetId,
        pos: Position,
        clicks: u8,
        focus: &mut Focus,
    ) -> FilterOutcome {
        if f.column.owns(id) {
            focus.focus(f.column.id);
            let (o, ev) = f.column.on_click(id);
            if let Some(junie_tui::widgets::select::SelectEvent::Changed(i)) = ev {
                let ops = FilterOp::ordered_for(f.columns[i].1);
                let labels: Vec<&str> = ops.iter().map(|o| o.label()).collect();
                f.op = Select::new(f.op.id, "Operator", &labels, 0)
                    .help("Operators that fit the column type come first");
            }
            return FilterOutcome::Keep(o.or(Outcome::Changed));
        }
        if f.op.owns(id) {
            focus.focus(f.op.id);
            return FilterOutcome::Keep(f.op.on_click(id).0.or(Outcome::Changed));
        }
        if let Some(o) = route_click(&mut [&mut f.value, &mut f.value2], id, pos, clicks) {
            focus.focus(id);
            return FilterOutcome::Keep(o);
        }
        if f.apply.id == id && f.apply.on_click() {
            return FilterOutcome::Apply;
        }
        if f.cancel.id == id && f.cancel.on_click() {
            return FilterOutcome::Cancel;
        }
        FilterOutcome::Keep(Outcome::Changed)
    }

    // --------------------------------------------------------------- render

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
            ctx.page_under_layer(!self.modals.is_empty());
            self.draw(area, buf, &mut ctx);
            ctx.flush_overlays(buf);
            cursor = ctx.cursor;
        }
        self.hits = hits;
        self.pointer.refresh_hover(&self.hits);
        self.ring = ring;
        if self.modals.is_empty() {
            if !self.too_small && !self.focus.current().is_some_and(|c| self.ring.contains(c)) {
                self.focus.set(self.ring.first());
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
                ("TablePro", t.title()),
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
                let w = junie_tui::ui::text::width(text) as u16;
                let x = area.x + area.width.saturating_sub(w) / 2;
                let y = y0 + i as u16 + if i == 3 { 1 } else { 0 };
                if y < area.bottom() {
                    buf.set_string(x, y, text, style.bg(t.canvas));
                }
            }
            return;
        }
        let header = Rect::new(area.x, area.y, area.width, 1);
        let footer = Rect::new(area.x, area.bottom() - 1, area.width, 1);
        let body = Rect::new(
            area.x + 1,
            area.y + 2,
            area.width.saturating_sub(2),
            area.height.saturating_sub(4),
        );
        self.draw_strip(header, buf, ctx);
        match self.screen {
            Screen::Connections => self.connections.render(body, buf, ctx),
            Screen::Workbench => {
                if let Some(w) = self.workbench.as_mut() {
                    w.render(body, buf, ctx, &self.history);
                }
            }
        }
        self.draw_footer(footer, buf, ctx);
        // overlays
        let mut modals = std::mem::take(&mut self.modals);
        for modal in modals.iter_mut() {
            match modal {
                Modal::Dialog(d) => d.render_in(
                    area,
                    junie_tui::widgets::dialog::body_bounds(area, 1, 1),
                    buf,
                    ctx,
                ),
                Modal::Picker(kind, p, _) => {
                    let hints = match kind {
                        PickerKind::Switcher => {
                            "↑↓ Move · Enter Open · Alt+Enter New tab · Tab Scope · Esc Clear / Close"
                        }
                        PickerKind::TabList => {
                            "↑↓ Move · Enter Switch · Delete Close tab · Esc Close"
                        }
                        PickerKind::SafeMode => {
                            "↑↓ Move · Enter Set level · Esc Keep · levels are saved to the connection"
                        }
                    };
                    p.hints = hints.to_owned();
                    p.draw(area, buf, ctx);
                }
                Modal::Filter(f) => Self::draw_filter_editor(f, area, buf, ctx),
                Modal::Menu(m) => m.menu.render(area, buf, ctx),
            }
        }
        self.modals = modals;
    }

    fn draw_strip(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let mut left = vec![
            Segment::new("▪", Tone::Success).priority(9),
            Segment::new("TablePro", Tone::Normal).bold().priority(9),
        ];
        let mut right = vec![];
        match self.screen {
            Screen::Connections => {
                left.push(Segment::new("Connections", Tone::Secondary).priority(8));
                let n = self.connections.connections.len();
                left.push(Segment::new(format!("{n} saved"), Tone::Muted).priority(3));
            }
            Screen::Workbench => {
                if let Some(w) = &self.workbench {
                    let c = &w.connection;
                    let env = c.environment;
                    let env_seg = match env {
                        Environment::Production => {
                            Segment::new("◆ production", Tone::Normal).bold()
                        }
                        Environment::Staging => Segment::new("◇ staging", Tone::Secondary),
                        Environment::Development => Segment::new("development", Tone::Muted),
                        Environment::Local => Segment::new("local", Tone::Faint),
                    };
                    left.push(
                        Segment::new(
                            junie_tui::ui::text::truncate_middle(&c.name, 18),
                            Tone::Normal,
                        )
                        .bold()
                        .clickable(STRIP_CONN)
                        .priority(9),
                    );
                    left.push(env_seg.priority(8));
                    left.push(
                        Segment::new(
                            format!("{} › {}", w.catalog.database, w.schema),
                            Tone::Secondary,
                        )
                        .clickable(STRIP_SCOPE)
                        .priority(7),
                    );
                    let level = c.safe_mode;
                    let (tone, bold) = match level {
                        SafeMode::Silent if env == Environment::Production => (Tone::Warning, true),
                        SafeMode::Silent => (Tone::Faint, false),
                        SafeMode::Alert | SafeMode::AlertFull => (Tone::Secondary, false),
                        _ => (Tone::Normal, true),
                    };
                    let mut s = Segment::new(level.token(), tone)
                        .clickable(STRIP_SAFE)
                        .priority(8);
                    if bold {
                        s = s.bold();
                    }
                    left.push(s);
                    if w.running().is_some() {
                        right.push(
                            Segment::new(
                                format!(
                                    "{} running",
                                    junie_tui::widgets::progress::spinner_frame(
                                        ctx.interaction.tick
                                    ),
                                ),
                                Tone::Secondary,
                            )
                            .priority(9),
                        );
                    }
                    let pending = w.pending_total();
                    if pending > 0 {
                        right.push(
                            Segment::new(format!("• {pending} pending"), Tone::Warning).priority(8),
                        );
                    }
                }
            }
        }
        right.push(
            Segment::new(
                format!("{} · {}×{}", t.level.label(), self.size.0, self.size.1),
                Tone::Faint,
            )
            .priority(1),
        );
        right.push(
            Segment::new("? help", Tone::Muted)
                .clickable(STRIP_HELP)
                .priority(4),
        );
        segments::render(area, buf, ctx, &left, &right, t.canvas);
    }

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = self.theme;
        let editing = match self.screen {
            Screen::Connections => self.connections.is_editing(),
            Screen::Workbench => self.workbench.as_ref().is_some_and(|w| w.is_editing()),
        };
        let hints: Vec<Hint> = match self.modals.top() {
            Some(Modal::Dialog(d)) => d.hints(self.focus.current()),
            Some(Modal::Picker(..)) => vec![],
            Some(Modal::Filter(_)) => vec![
                hint("Tab", "Next field"),
                hint("Enter", "Apply"),
                hint("Esc", "Cancel"),
            ],
            Some(Modal::Menu(_)) => vec![
                hint("↑↓", "Move"),
                hint("Enter", "Choose"),
                hint("Esc", "Close"),
            ],
            None => match self.screen {
                Screen::Connections => self.connections.hints(self.focus.current()),
                Screen::Workbench => self
                    .workbench
                    .as_ref()
                    .map(|w| w.hints(self.focus.current()))
                    .unwrap_or_default(),
            },
        };
        let mut hints = hints;
        // the focused cursor row is cut: name the reveal key
        junie_tui::ui::reveal::advertise(&mut hints, ctx.reveal_hint, |h| h.key, hint);
        if self.modals.is_empty() && !editing {
            hints.push(hint("Tab", "Next"));
        }
        let badge = if editing && self.modals.is_empty() {
            Some(("EDIT", BadgeKind::Edit))
        } else {
            None
        };
        let status = self.status.as_ref().map(|s| s.0.as_str());
        keyhint::render(area, buf, &t, &hints, badge, status);
    }

    fn draw_filter_editor(
        f: &mut FilterEditor,
        screen: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
    ) {
        let t = ctx.theme;
        // dim like a dialog, keep the footer
        ctx.modal_backdrop(buf, screen);
        let w = 64u16.min(screen.width.saturating_sub(4));
        let h = 15u16.min(screen.height.saturating_sub(2));
        let area = junie_tui::ui::popup::place(
            screen,
            Rect::ZERO,
            w,
            h,
            junie_tui::ui::popup::Placement::Center,
        );
        f.area = area;
        let bg = t.surface_elevated;
        fill(buf, area, ratatui::style::Style::new().bg(bg));
        let block = ratatui::widgets::Block::new()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(t.border(true).bg(bg));
        ratatui::widgets::Widget::render(block, area, buf);
        // the editor's body: a press on it is dead space inside the modal
        ctx.hits
            .register_surface(WidgetId::of("filter-editor"), area);
        let inner = area.inner(ratatui::layout::Margin::new(3, 1));
        buf.set_string(
            inner.x,
            inner.y,
            if f.index.is_some() {
                "Edit filter"
            } else {
                "Add filter"
            },
            t.title().bg(bg),
        );
        let mut y = inner.y + 2;
        let (l, r) = junie_tui::ui::layout::Split::new(50, 16, 16).horizontal(
            Rect::new(
                inner.x.saturating_sub(1),
                y,
                inner.width + 1,
                Select::HEIGHT,
            ),
            2,
        );
        f.column.render(l, buf, ctx, bg);
        f.op.render(r, buf, ctx, bg);
        y += Select::HEIGHT;
        let ops = FilterOp::ordered_for(f.columns[f.column.selected].1);
        let op = ops[f.op.selected.min(ops.len() - 1)];
        if op.needs_value() {
            if op == FilterOp::Between {
                let (l, r) = junie_tui::ui::layout::Split::new(50, 12, 12).horizontal(
                    Rect::new(
                        inner.x.saturating_sub(1),
                        y,
                        inner.width + 1,
                        TextInput::HEIGHT,
                    ),
                    2,
                );
                f.value.render(l, buf, ctx, bg);
                f.value2.render(r, buf, ctx, bg);
            } else {
                f.value.render(
                    Rect::new(
                        inner.x.saturating_sub(1),
                        y,
                        inner.width + 1,
                        TextInput::HEIGHT,
                    ),
                    buf,
                    ctx,
                    bg,
                );
            }
        } else {
            buf.set_string(
                inner.x + 1,
                y + 1,
                "No value needed for this operator",
                t.muted().bg(bg),
            );
        }
        y += TextInput::HEIGHT + 1;
        let preview = Filter {
            column: f.columns[f.column.selected].0.clone(),
            op,
            value: f.value.text().to_owned(),
            value2: f.value2.text().to_owned(),
            enabled: true,
        }
        .to_sql();
        buf.set_string(
            inner.x,
            y,
            junie_tui::ui::text::truncate(&format!("WHERE {preview}"), inner.width as usize),
            t.secondary().bg(bg),
        );
        let ay = inner.bottom().saturating_sub(1);
        let widths = [f.cancel.width(), f.apply.width()];
        let rects = junie_tui::widgets::button::row_layout_right(
            Rect::new(inner.x, ay, inner.width, 1),
            &widths,
            1,
        );
        f.cancel.render(rects[0], buf, ctx, bg);
        f.apply.render(rects[1], buf, ctx, bg);
        // controls on top of the surface
        ctx.hits.register(f.column.id, f.column.area);
        ctx.hits.register(f.op.id, f.op.area);
        ctx.hits.register(f.value.id, f.value.area);
        ctx.hits.register(f.value2.id, f.value2.area);
        ctx.hits.register(f.cancel.id, f.cancel.area);
        ctx.hits.register(f.apply.id, f.apply.area);
    }
}

enum FilterOutcome {
    Keep(Outcome),
    Apply,
    Cancel,
}

fn pretty_json(j: &str) -> String {
    // minimal pretty printer for the demo's flat/simple JSON
    let mut out = String::new();
    let mut depth = 0usize;
    let mut in_str = false;
    for c in j.chars() {
        match c {
            '"' => {
                in_str = !in_str;
                out.push(c);
            }
            '{' | '[' if !in_str => {
                depth += 1;
                out.push(c);
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
            }
            '}' | ']' if !in_str => {
                depth = depth.saturating_sub(1);
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
                out.push(c);
            }
            ',' if !in_str => {
                out.push(c);
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
            }
            ' ' if !in_str => {}
            _ => out.push(c),
        }
    }
    out
}
