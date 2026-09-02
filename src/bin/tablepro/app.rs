//! TablePro shell: screens (Connections, Workbench), overlays (dialogs,
//! pickers, filter editor), identity strip, footer hints, routing.

use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{BadgeKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx, fill};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::{Dialog, DialogBody, DialogResult};
use junie_tui::widgets::grid::CellValue;
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::keyhint::{self, Hint, hint};
use junie_tui::widgets::picker::{Picker, PickerEvent, PickerItem};
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::{self, Segment};
use junie_tui::widgets::select::Select;

use crate::connections::{ConnEvent, ConnectionsScreen};
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
}

#[allow(clippy::large_enum_variant)] // one overlay at a time
pub enum Modal {
    Dialog(Dialog),
    Picker(PickerKind, Picker, Option<SwitcherIndex>),
    Filter(FilterEditor),
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
    pub hover: Option<WidgetId>,
    pub pressed: Option<WidgetId>,
    pub hover_suppressed: bool,
    pub modal: Option<Modal>,
    pub size: (u16, u16),
    pub tick: u64,
    pub status: Option<(String, Instant)>,
    flash: Option<(WidgetId, Instant)>,
    pub quit: bool,
    saved_focus: Option<WidgetId>,
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
            hover: None,
            pressed: None,
            hover_suppressed: false,
            modal: None,
            size: (0, 0),
            tick: 0,
            status: None,
            flash: None,
            quit: false,
            saved_focus: None,
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

    pub fn animating(&self) -> bool {
        self.connections.animating()
            || self.workbench.as_ref().is_some_and(|w| w.animating())
            || self.flash.is_some()
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

    fn interaction(&self) -> Interaction {
        let flash = match self.flash {
            Some((id, at)) if at.elapsed() < Duration::from_millis(140) => Some(id),
            _ => None,
        };
        Interaction {
            focus: self.focus.current(),
            hover: self.hover,
            pressed: self.pressed,
            flash,
            focus_hidden: false,
            hover_suppressed: self.hover_suppressed,
            tick: self.tick,
        }
    }

    // ---------------------------------------------------------------- input

    pub fn handle(&mut self, input: Input) -> Outcome {
        match input {
            Input::Resize(w, h) => {
                self.size = (w, h);
                Outcome::Changed
            }
            Input::Tick => self.on_tick(),
            Input::Paste(text) => {
                if let Some(Modal::Dialog(d)) = self.modal.as_mut() {
                    return d.on_paste(&text);
                }
                if let Some(Modal::Filter(f)) = self.modal.as_mut() {
                    if f.value.editing {
                        return f.value.on_paste(&text);
                    }
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
                self.hover_suppressed = true;
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
        let mut out = Outcome::Ignored;
        if self.animating() {
            self.tick = self.tick.wrapping_add(1);
            out = Outcome::Changed;
        }
        if let Some((_, at)) = self.flash
            && at.elapsed() >= Duration::from_millis(140)
        {
            self.flash = None;
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
        if self.modal.is_some() {
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
                self.flash = Some((f, Instant::now()));
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
                self.set_status("Cannot execute write queries: TablePro's Safe Mode is set to read-only for this connection".into());
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
                                "deliberate confirmation required"
                            } else {
                                "confirmation required"
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
                        "deliberate confirmation required"
                    } else if level.requires_confirmation() || deletes > 0 {
                        "confirmation required"
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
        self.saved_focus = self.focus.current();
        self.focus.set(Some(d.initial_focus));
        self.modal = Some(Modal::Dialog(d));
        self.hover = None;
        self.pressed = None;
    }

    fn open_help(&mut self) {
        let text = "Tab / Shift+Tab   move focus · 0 explorer · Esc back\n\
                    Ctrl+O            Open Quickly (tables, schemas, tabs, queries)\n\
                    Ctrl+T / Ctrl+W   new / close tab · [ ] switch · Ctrl+G tab list\n\
                    Ctrl+R / F5       run statement at cursor · Alt+R run all\n\
                    Ctrl+X / Alt+X    EXPLAIN / EXPLAIN ANALYZE\n\
                    Esc / Ctrl+C      cancel a running query\n\
                    Ctrl+D            Data / Structure · Ctrl+F filter (grid) or find (editor)\n\
                    Ctrl+S            save pending row changes · p preview SQL\n\
                    Ctrl+Y            query history · Ctrl+L Safe Mode · Ctrl+B explorer · z zoom\n\
                    q                 quit";
        let mut d = Dialog::confirm(HELP_DIALOG, "Keyboard", text, "Close");
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
        p.width = 72;
        self.scope = 0;
        p.scope = Some("All · Tab scope".into());
        self.saved_focus = self.focus.current();
        self.modal = Some(Modal::Picker(PickerKind::Switcher, p, Some(index)));
        self.refresh_switcher();
    }

    fn refresh_switcher(&mut self) {
        let Some(Modal::Picker(PickerKind::Switcher, p, Some(index))) = self.modal.as_mut() else {
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
            })
            .collect();
        p.set_items(items);
        p.cursor = w.active;
        self.saved_focus = self.focus.current();
        self.modal = Some(Modal::Picker(PickerKind::TabList, p, None));
    }

    fn open_safe_mode_picker(&mut self) {
        let Some(w) = self.workbench.as_ref() else {
            return;
        };
        let current = w.connection.safe_mode;
        let mut p = Picker::new(WidgetId::of("safemode"), "Safe Mode · this connection");
        p.searchable = false;
        p.width = 74;
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
            })
            .collect();
        p.set_items(items);
        p.cursor = SafeMode::ALL
            .iter()
            .position(|s| *s == current)
            .unwrap_or(0);
        p.empty_text = String::new();
        self.saved_focus = self.focus.current();
        self.modal = Some(Modal::Picker(PickerKind::SafeMode, p, None));
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
        };
        self.saved_focus = self.focus.current();
        self.focus.focus(if value.is_empty() {
            editor.value.id
        } else {
            editor.apply.id
        });
        self.modal = Some(Modal::Filter(editor));
    }

    fn close_modal(&mut self) {
        self.modal = None;
        self.focus.set(self.saved_focus.take());
    }

    fn modal_key(&mut self, key: Key) -> Outcome {
        let Some(modal) = self.modal.as_mut() else {
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
                let idx = match &self.modal {
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
        let Some(Modal::Filter(f)) = self.modal.as_mut() else {
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
                let Some(pressed) = self.pressed else {
                    return Outcome::Ignored;
                };
                if self.modal.is_some() {
                    return Outcome::Consumed;
                }
                match self.screen {
                    Screen::Workbench => self
                        .workbench
                        .as_mut()
                        .map(|w| w.on_drag(pressed, m.pos))
                        .unwrap_or(Outcome::Ignored),
                    Screen::Connections => Outcome::Ignored,
                }
            }
            MouseKind::Down => {
                let hit = self.hits.hit(m.pos);
                self.pressed = hit;
                self.hover = hit;
                let Some(id) = hit else {
                    return if self.modal.is_some() {
                        Outcome::Consumed
                    } else {
                        Outcome::Ignored
                    };
                };
                if self.modal.is_some() {
                    return Outcome::Changed;
                }
                if self.ring.contains(id) {
                    self.focus.focus(id);
                }
                Outcome::Changed
            }
            MouseKind::Up => {
                let hit = self.hits.hit(m.pos);
                let pressed = self.pressed.take();
                let Some(id) = hit else {
                    // click outside a modal cancels it (dialogs decide themselves)
                    if let Some(modal) = self.modal.as_mut()
                        && pressed.is_none()
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
                            Modal::Picker(..) | Modal::Filter(_) => {
                                self.close_modal();
                                return Outcome::Changed;
                            }
                        }
                    }
                    return Outcome::Changed;
                };
                if pressed != Some(id) {
                    return Outcome::Changed;
                }
                self.flash = Some((id, Instant::now()));
                if let Some(modal) = self.modal.as_mut() {
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
                        Modal::Filter(f) => {
                            let o = Self::filter_click(f, id, m.pos, &mut self.focus);
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
                        };
                        let o = w.on_click(id, m.pos, &mut cx, &mut self.history);
                        let reqs = std::mem::take(&mut cx.requests);
                        self.workbench = Some(w);
                        let o2 = self.apply_requests(reqs);
                        o.or(o2).or(Outcome::Changed)
                    }
                }
            }
            MouseKind::WheelUp
            | MouseKind::WheelDown
            | MouseKind::WheelLeft
            | MouseKind::WheelRight => {
                let horizontal = matches!(m.kind, MouseKind::WheelLeft | MouseKind::WheelRight);
                let delta = match m.kind {
                    MouseKind::WheelUp | MouseKind::WheelLeft => -3,
                    _ => 3,
                };
                if let Some(Modal::Picker(_, p, _)) = self.modal.as_mut() {
                    return p.on_wheel(delta);
                }
                if self.modal.is_some() {
                    return Outcome::Consumed;
                }
                let Some(id) = self.hits.hit_scroll(m.pos) else {
                    return Outcome::Ignored;
                };
                match self.screen {
                    Screen::Workbench => self
                        .workbench
                        .as_mut()
                        .map(|w| w.on_wheel(id, delta, horizontal))
                        .unwrap_or(Outcome::Ignored),
                    Screen::Connections => Outcome::Ignored,
                }
            }
        }
    }

    fn filter_click(
        f: &mut FilterEditor,
        id: WidgetId,
        pos: Position,
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
        for inp in [&mut f.value, &mut f.value2] {
            if inp.id == id {
                let was = focus.is(id);
                focus.focus(id);
                return FilterOutcome::Keep(inp.on_click(pos, was));
            }
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
            self.draw(area, buf, &mut ctx);
            cursor = ctx.cursor;
        }
        self.hits = hits;
        self.ring = ring;
        if self.modal.is_none() {
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
        let modal = self.modal.take();
        if let Some(mut modal) = modal {
            match &mut modal {
                Modal::Dialog(d) => d.render(area, buf, ctx),
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
                    p.render(area, buf, ctx, hints);
                }
                Modal::Filter(f) => Self::draw_filter_editor(f, area, buf, ctx),
            }
            self.modal = Some(modal);
        }
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
                    if let Some(ms) = w.running() {
                        right.push(
                            Segment::new(
                                format!(
                                    "{} running {}",
                                    junie_tui::widgets::progress::spinner_frame(
                                        ctx.interaction.tick
                                    ),
                                    crate::tabs::duration_label(ms)
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

    fn draw_footer(&mut self, area: Rect, buf: &mut Buffer, _ctx: &mut RenderCtx) {
        let t = self.theme;
        let editing = match self.screen {
            Screen::Connections => self.connections.is_editing(),
            Screen::Workbench => self.workbench.as_ref().is_some_and(|w| w.is_editing()),
        };
        let hints: Vec<Hint> = match &self.modal {
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
            Some(Modal::Picker(..)) => vec![],
            Some(Modal::Filter(_)) => vec![
                hint("Tab", "Next field"),
                hint("Enter", "Apply"),
                hint("Esc", "Cancel"),
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
        if self.modal.is_none() && !editing {
            hints.push(hint("Tab", "Next"));
        }
        let badge = if editing && self.modal.is_none() {
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
        let dim = Rect::new(
            screen.x,
            screen.y,
            screen.width,
            screen.height.saturating_sub(1),
        );
        for pos in dim.positions() {
            if let Some(c) = buf.cell_mut(pos) {
                let st = t.backdrop(c.style());
                c.set_style(st);
                c.modifier = ratatui::style::Modifier::empty();
            }
        }
        ctx.begin_modal();
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
        ctx.hits.register(WidgetId::of("filter-editor"), area);
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
