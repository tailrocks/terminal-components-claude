//! Showcase pages. Each page owns its widgets and routes events to them.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::dialog::{Dialog, DialogResult};

pub mod buttons;
pub mod chips;
pub mod dialogs;
pub mod editable;
pub mod editor;
pub mod forms;
pub mod grid;
pub mod inputs;
pub mod lists;
pub mod overview;
pub mod panels;
pub mod pickers;
pub mod progress;
pub mod scrolling;
pub mod settings;
pub mod sidebars;
pub mod tables;
pub mod taskrunner;
pub mod textareas;
pub mod trees;

/// Event delivered to a page after the app has resolved hit-testing.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)] // DialogClosed carries the prompt text; events are short-lived
pub enum PageEvent {
    Key(Key),
    /// Text pasted while a control is editing.
    Paste(String),
    /// Completed click (down and up on the same id).
    Click {
        id: WidgetId,
        pos: Position,
    },
    /// Pointer moved while the button is held; `pressed` is the id where
    /// the press started.
    Drag {
        pressed: WidgetId,
        pos: Position,
    },
    /// Wheel over `id` (a scroll container).
    Wheel {
        id: WidgetId,
        delta: i32,
    },
    Tick,
    DialogClosed {
        id: WidgetId,
        result: DialogResult,
        /// Text entered in a prompt dialog, if any.
        value: Option<String>,
    },
}

/// Things a page may ask the app to do.
#[derive(Debug)]
pub enum Request {
    OpenDialog(Box<Dialog>),
    Status(String),
}

pub struct PageCtx<'a> {
    pub focus: &'a mut Focus,
    pub ring: &'a FocusRing,
    pub requests: Vec<Request>,
}

impl PageCtx<'_> {
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

pub type Hint = (&'static str, &'static str);

pub trait Page {
    fn title(&self) -> &'static str;
    fn blurb(&self) -> &'static str;
    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx);
    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome;
    /// Contextual key hints for the footer given the focused widget.
    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint>;
    /// True while a control on this page is in edit mode.
    fn editing(&self) -> bool {
        false
    }
    /// True when the page needs periodic ticks (spinners, progress).
    fn animating(&self) -> bool {
        false
    }
}

/// Shared helpers for laying out demo sections.
pub mod layout {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Color;

    use junie_tui::theme::Theme;

    /// Small muted label above a control.
    pub fn caption(x: u16, y: u16, buf: &mut Buffer, t: &Theme, text: &str, bg: Color) {
        buf.set_string(x, y, text, t.muted().bg(bg));
    }

    /// Split vertically into rows with fixed heights; the last takes the rest.
    pub fn rows(area: Rect, heights: &[u16]) -> Vec<Rect> {
        let mut y = area.y;
        let mut out = Vec::new();
        for (i, &h) in heights.iter().enumerate() {
            let last = i == heights.len() - 1;
            let h = if last {
                area.bottom().saturating_sub(y)
            } else {
                h.min(area.bottom().saturating_sub(y))
            };
            out.push(Rect::new(area.x, y, area.width, h));
            y = y.saturating_add(h);
        }
        out
    }

    /// Two columns with a gap; if too narrow, stack vertically.
    pub fn columns(area: Rect, left_w: u16, gap: u16) -> (Rect, Rect) {
        if area.width < left_w + gap + 20 {
            let h = area.height / 2;
            return (
                Rect::new(area.x, area.y, area.width, h),
                Rect::new(area.x, area.y + h, area.width, area.height - h),
            );
        }
        (
            Rect::new(area.x, area.y, left_w, area.height),
            Rect::new(
                area.x + left_w + gap,
                area.y,
                area.width - left_w - gap,
                area.height,
            ),
        )
    }
}
