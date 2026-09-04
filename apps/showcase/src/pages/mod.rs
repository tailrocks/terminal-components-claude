//! The twenty-two showcase screens.
//!
//! Every screen owns the state for the controls it demonstrates. The shell
//! only selects a screen and supplies its content rectangle; this keeps the
//! application package a consumer of the public `tui-next` facade rather than
//! a second component implementation.

use tui_next::{Panel, PanelKind, Rect, Response, Ui, id};

/// A stateful screen in the showcase.
pub(crate) trait Page: Send {
    /// Stable navigation title.
    fn title(&self) -> &'static str;
    /// Drain this screen's runtime intents.
    fn update(&mut self, cx: &mut tui_next::Cx<'_>) -> Response<()>;
    /// Handle an application-level command before component intents run.
    fn command(&mut self, _cx: &mut tui_next::Cx<'_>, _action: tui_next::ActionKey) -> Response<()> {
        Response::ignored()
    }
    /// Draw this screen into the shell's content rectangle.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
}

/// Draw a screen frame and hand its inset body to the page.
pub(crate) fn frame(
    ui: &mut Ui<'_>,
    area: Rect,
    title: &'static str,
    meta: &'static str,
    body: impl FnOnce(&mut Ui<'_>, Rect),
) {
    Panel::new(id!("page.frame"))
        .kind(PanelKind::Framed)
        .title(title)
        .meta(meta)
        .draw(ui, area, body);
}

/// Paint a set of lines with one-cell spacing, clipping at the body edge.
pub(crate) fn lines(ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
    let style = ui.surface_style();
    for (offset, line) in text.iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        if offset >= area.height {
            break;
        }
        let row = Rect {
            y: area.y.saturating_add(offset),
            height: 1,
            ..area
        };
        let _ = ui.paint_str(row, line, style);
    }
}

/// Split a body into equal-height rows with a one-cell gap.
pub(crate) fn rows(area: Rect, count: u16) -> Vec<Rect> {
    if count == 0 || area.is_empty() {
        return Vec::new();
    }
    let gap = count.saturating_sub(1);
    let height = area.height.saturating_sub(gap) / count;
    let mut result = Vec::with_capacity(usize::from(count));
    let mut y = area.y;
    for index in 0..count {
        let remaining = area.bottom().saturating_sub(y);
        let row_height = if index + 1 == count {
            remaining
        } else {
            height
        };
        result.push(Rect {
            x: area.x,
            y,
            width: area.width,
            height: row_height,
        });
        y = y.saturating_add(row_height).saturating_add(1);
    }
    result
}

pub(crate) mod buttons;
pub(crate) mod author;
pub(crate) mod chips;
pub(crate) mod chrome;
pub(crate) mod dialogs;
pub(crate) mod editable;
pub(crate) mod editor;
pub(crate) mod forms;
pub(crate) mod grid;
pub(crate) mod inputs;
pub(crate) mod lists;
pub(crate) mod overview;
pub(crate) mod panels;
pub(crate) mod pickers;
pub(crate) mod progress;
pub(crate) mod scrolling;
pub(crate) mod settings;
pub(crate) mod sidebars;
pub(crate) mod tables;
pub(crate) mod taskrunner;
pub(crate) mod terminal;
pub(crate) mod textareas;
pub(crate) mod trees;
