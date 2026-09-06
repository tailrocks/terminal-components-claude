//! Route screens and the contract they share with the shell: input,
//! requests back to the app, modal ownership, rendering and hints.

// The contract lands whole; later phases (P1+) grow into the parts P0
// does not exercise yet.
#![allow(dead_code)]

pub mod activity;
pub mod home;
pub mod modals;
pub mod plan;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::Hint;
use junie_tui::widgets::segments::Segment;
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::sim::world::{Msg, World};

/// Identifies a modal's purpose and target for its owning screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModalTag {
    pub kind: &'static str,
    pub key: String,
    pub n: usize,
}

impl ModalTag {
    pub fn new(kind: &'static str) -> Self {
        Self {
            kind,
            key: String::new(),
            n: 0,
        }
    }
    pub fn key(mut self, k: impl Into<String>) -> Self {
        self.key = k.into();
        self
    }
    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }
}

/// A screen-specific modal that still lives on the shared modal stack.
pub trait CustomModal {
    fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing, w: &World) -> Outcome;
    fn on_click(
        &mut self,
        _id: WidgetId,
        _pos: Position,
        _focus: &mut Focus,
        _w: &World,
    ) -> Outcome {
        Outcome::Consumed
    }
    fn on_wheel(&mut self, _delta: i32, _pos: Position) -> Outcome {
        Outcome::Consumed
    }
    fn on_tick(&mut self, _w: &World) -> Outcome {
        Outcome::Ignored
    }
    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World);
    fn done(&mut self) -> Option<ModalResult>;
    fn initial_focus(&self) -> WidgetId;
    fn hints(&self) -> Vec<Hint>;
    fn cancel_on_outside_click(&self) -> bool {
        true
    }
}

#[allow(clippy::large_enum_variant)] // dialogs dominate; boxing churns every call site
pub enum Modal {
    Dialog(Dialog),
    Custom(Box<dyn CustomModal>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalResult {
    /// `action` is the pressed button index (None = cancelled); `text` the
    /// prompt's value.
    Dialog {
        action: Option<usize>,
        text: Option<String>,
    },
    Cancelled,
    Custom(String),
}

/// Navigation the app performs on a screen's behalf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Go {
    Home,
    /// Open the gate-1 review surface for a compound intent.
    Plan(Box<crate::domain::plan::Plan>),
    /// Switch to a named activity's retained-output page.
    Activity(u32),
    Quit,
}

pub enum Request {
    Status(String),
    Error(String),
    Open(Box<Modal>, ModalTag),
    Close,
    Go(Go),
    Copy(String),
    /// Open the key reference for the current screen.
    Help,
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
    pub fn status(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Status(s.into()));
    }
    pub fn error(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Error(s.into()));
    }
    pub fn open(&mut self, modal: Modal, tag: ModalTag) {
        self.requests.push(Request::Open(Box::new(modal), tag));
    }
    pub fn close(&mut self) {
        self.requests.push(Request::Close);
    }
    pub fn go(&mut self, g: Go) {
        self.requests.push(Request::Go(g));
    }
    pub fn help(&mut self) {
        self.requests.push(Request::Help);
    }
    pub fn copy(&mut self, s: impl Into<String>) {
        self.requests.push(Request::Copy(s.into()));
    }
}

/// Everything a route screen can do. Default bodies let small screens stay
/// small; the shell calls only what it needs.
pub trait Screen {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome;
    fn on_click(&mut self, _id: WidgetId, _pos: Position, _w: &mut World, _cx: &mut Cx) -> Outcome {
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
    /// Crumb after the host identity: `Home`, `Plan › Upgrade everything`.
    fn crumb(&self, w: &World) -> String;
    /// Extra right-hand header segments (working spinner, counts).
    fn header_right(&self, _w: &World) -> Vec<Segment> {
        vec![]
    }
    fn is_editing(&self) -> bool {
        false
    }
    fn animating(&self, _w: &World) -> bool {
        false
    }
    /// The route became active.
    fn enter(&mut self, _w: &mut World, _cx: &mut Cx) {}
    /// Where keyboard focus lands when nothing better is known.
    fn primary_focus(&self) -> Option<WidgetId>;
    /// Esc at the top of this screen's ladder: leave it?
    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Home);
        Outcome::Changed
    }
}

#[allow(dead_code)] // P2+
pub fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
}
