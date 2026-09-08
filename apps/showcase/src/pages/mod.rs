//! The twenty-two showcase screens.
//!
//! Every screen owns the state for the controls it demonstrates. The shell
//! only selects a screen and supplies its content rectangle; this keeps the
//! application package a consumer of the public `junie-tui` facade rather than
//! a second component implementation.

use junie_tui::{Family, Part, Rect, Response, StateFlags, Ui, Variant, truncate, width};

/// A stateful screen in the showcase.
pub(crate) trait Page: Send {
    /// Stable navigation title.
    fn title(&self) -> &'static str;
    /// Drain this screen's runtime intents.
    fn update(&mut self, cx: &mut junie_tui::Cx<'_>) -> Response<()>;
    /// Handle an application-level command before component intents run.
    fn command(
        &mut self,
        _cx: &mut junie_tui::Cx<'_>,
        _action: junie_tui::ActionKey,
    ) -> Response<()> {
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
    // The historical shell has a title row, a blank row, then page content;
    // it does not put a second card around every page.  Keep the title/meta
    // paint behind the new Ui boundary, while leaving ownership of the page
    // body with the migrated component composition.
    if area.is_empty() {
        return;
    }
    ui.fill(area, ui.surface_style());
    let title_style = ui
        .style(
            Family::PANEL,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::empty(),
        )
        .style;
    let meta_style = ui
        .style(
            Family::LIST,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
        )
        .style;
    let title_area = Rect { height: 1, ..area };
    let title_width = width(title).min(area.width);
    ui.paint_str(title_area, title, title_style);
    if !meta.is_empty() && area.width > title_width.saturating_add(4) {
        let meta_area = Rect {
            x: area.x.saturating_add(title_width).saturating_add(2),
            width: area.width.saturating_sub(title_width).saturating_sub(3),
            height: 1,
            ..area
        };
        let fitted = truncate(meta, meta_area.width);
        ui.paint_str(meta_area, &fitted, meta_style);
    }
    let body_area = Rect {
        y: area.y.saturating_add(2),
        height: area.height.saturating_sub(2),
        ..area
    };
    body(ui, body_area);
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

pub(crate) mod author;
pub(crate) mod buttons;
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
