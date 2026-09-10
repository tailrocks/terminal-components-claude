//! Surfaces and the contract they share with the shell. A tab is something
//! that runs (an activity, an executing plan); a page is something you are
//! deciding (the finder, a domain page, Disk, a plan review, a gate). The
//! Here tab holds a stack of pages; every other tab holds one screen.

pub mod activity;
pub mod disk;
pub mod finder;
pub mod modals;
pub mod plan;
pub mod review;
pub mod snapshot;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::Hint;
use junie_tui::widgets::menu::ContextMenu;
use junie_tui::widgets::picker::Picker;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::domain::context::Scope;
use crate::sim::world::{Msg, World};
use modals::TextModal;

/// Identifies a modal's purpose for its owning screen or the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalTag {
    pub kind: &'static str,
    pub key: String,
}

impl ModalTag {
    pub fn new(kind: &'static str) -> Self {
        Self {
            kind,
            key: String::new(),
        }
    }
    pub fn key(mut self, k: impl Into<String>) -> Self {
        self.key = k.into();
        self
    }
}

#[allow(clippy::large_enum_variant)]
pub enum Modal {
    Dialog(Dialog),
    Picker(Picker),
    /// An anchored alternatives menu; drawn last, keyboard goes to it.
    Menu(ContextMenu),
    Text(TextModal),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalResult {
    /// The pressed button index (None = cancelled) and the typed text.
    Dialog {
        action: Option<usize>,
        text: Option<String>,
    },
    Picked(usize),
    Cancelled,
    MenuChosen(usize),
    Closed,
}

/// What Enter should do with an item once the shell has resolved trust,
/// arguments and gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Go {
    /// Run an item by id (the shell resolves trust, arguments and gates).
    Run {
        item: String,
        args: Vec<(String, String)>,
    },
    /// Show the alternatives menu for an item at an anchor.
    Alternatives {
        item: String,
        anchor: Rect,
    },
    /// Push a page onto the Here tab.
    Push(Page),
    /// Pop the top page of the Here tab.
    Pop,
    /// Activate the Here tab (keeping its stack).
    Here,
    /// Switch the finder scope.
    Scope(Scope),
    /// Confirm a reviewed plan: gate it or start it.
    ConfirmPlan(String),
    /// A gate-1 page was accepted for a plan or an item.
    Gate1Accepted(GateTarget),
    /// A trust page accepted a configuration.
    Trusted {
        config: String,
        then: String,
    },
    /// Build the cleanup plan for the selected candidates and review it.
    CleanupPlan(Vec<String>),
    /// Restart an activity in place (a failed launch's retry).
    Restart(String),
    /// Close the active tab (asks when something runs).
    CloseTab,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateTarget {
    Plan(String),
    Item {
        item: String,
        args: Vec<(String, String)>,
    },
}

/// A page the Here tab can hold; built lazily by the shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Finder { group: Option<&'static str> },
    Disk { path: String },
    PlanReview { plan: String },
    Review(GateTarget),
    Trust { config: String, then: String },
    Args { item: String },
    Snapshot { kind: String },
}

pub enum Request {
    Status(String),
    Error(String),
    Open(Box<Modal>, ModalTag),
    Go(Go),
    Copy(String),
}

pub struct Cx<'a> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub requests: Vec<Request>,
}

impl Cx<'_> {
    pub fn status(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Status(s.into()));
    }
    pub fn error(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Error(s.into()));
    }
    pub fn open(&mut self, modal: Modal, tag: ModalTag) {
        self.requests.push(Request::Open(Box::new(modal), tag));
    }
    pub fn go(&mut self, g: Go) {
        self.requests.push(Request::Go(g));
    }
    pub fn copy(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Copy(s.into()));
    }
}

/// Status-bar contribution of a screen: the centre item and extra right
/// items.
#[derive(Default)]
pub struct StatusBits {
    pub center: Option<StatusItem>,
    pub right: Vec<StatusItem>,
}

/// Everything a page or tab screen can do. Default bodies keep small
/// screens small.
pub trait Screen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome;
    fn on_click(&mut self, _id: WidgetId, _pos: Position, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_double_click(
        &mut self,
        _id: WidgetId,
        _pos: Position,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        Outcome::Ignored
    }
    fn on_secondary(
        &mut self,
        _id: WidgetId,
        _pos: Position,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        Outcome::Ignored
    }
    fn on_press(&mut self, _id: WidgetId, _pos: Position, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_drag(&mut self, _pressed: WidgetId, _pos: Position, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_wheel(&mut self, _id: WidgetId, _delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_paste(&mut self, _text: &str, _w: &mut World) -> Outcome {
        Outcome::Ignored
    }
    fn on_tick(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_msg(&mut self, _msg: &Msg, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_modal(
        &mut self,
        _tag: &ModalTag,
        _result: ModalResult,
        _w: &mut World,
        _cx: &mut Cx,
    ) -> Outcome {
        Outcome::Ignored
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World);
    fn hints(&self, focus: Option<WidgetId>, w: &World) -> Vec<Hint>;
    /// Breadcrumb segment shown in the Here tab label (`Disk › Usage`).
    fn crumb(&self, w: &World) -> String;
    fn status(&self, _w: &World) -> StatusBits {
        StatusBits::default()
    }
    /// Letters type into a query on this screen.
    fn typing_hot(&self) -> bool {
        false
    }
    fn is_editing(&self) -> bool {
        false
    }
    fn animating(&self, _w: &World) -> bool {
        false
    }
    fn enter(&mut self, _w: &mut World, _cx: &mut Cx) {}
    fn primary_focus(&self) -> Option<WidgetId>;
    /// Esc at the top of this screen's ladder.
    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Pop);
        Outcome::Changed
    }
    /// The finder behind this screen, when it is one.
    fn as_finder(&mut self) -> Option<&mut finder::FinderPage> {
        None
    }
}

pub fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}

/// `2 min 14 s` from ticks.
pub fn ticks_label(ticks: u64) -> String {
    crate::clock::format_duration(ticks * crate::sim::world::TICK_MS as u64 / 1000)
}

/// Tone for a risk word in rows and previews (D-7).
pub fn risk_tone(risk: crate::domain::action::Risk) -> Tone {
    use crate::domain::action::Risk;
    match risk {
        Risk::ReadOnly => Tone::Faint,
        Risk::Mutating => Tone::Muted,
        Risk::Destructive => Tone::Error,
        Risk::Privileged => Tone::Warning,
    }
}

/// A one-row section heading: text-faint, sentence case (DESIGN hierarchy).
pub fn heading(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    w: u16,
    text: &str,
    t: &junie_tui::theme::Theme,
    bg: ratatui::style::Color,
) {
    if w == 0 {
        return;
    }
    buf.set_string(
        x,
        y,
        junie_tui::ui::text::truncate(text, w as usize),
        t.faint().bg(bg),
    );
}

/// Truncate a ` · `-joined phrase at the last separator that fits, so a
/// cell never ends mid-word; falls back to a plain cut when no separator
/// leaves at least a third of the room.
pub fn truncate_sep(s: &str, max: usize) -> String {
    use junie_tui::ui::text::{truncate, width};
    if width(s) <= max {
        return s.to_owned();
    }
    if max < 4 {
        return truncate(s, max);
    }
    let mut best: Option<usize> = None;
    let mut pos = 0;
    while let Some(i) = s[pos..].find(" · ") {
        let at = pos + i;
        let w = width(&s[..at]);
        if w < max && w * 3 >= max {
            best = Some(at);
        }
        if w + 1 > max {
            break;
        }
        pos = at + 3;
    }
    match best {
        Some(at) => format!("{}…", &s[..at]),
        None => truncate(s, max),
    }
}
