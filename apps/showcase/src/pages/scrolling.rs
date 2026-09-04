//! A selectable, scrollable text viewport with a real scrollbar.

use tui_next::{Cx, Id, Rect, Response, TextViewport, Ui, ViewportAction, ViewportLine, ViewportState, id};

use crate::data::PROSE;

use super::{Page, frame, lines};

const VIEW: Id = id!("scrolling.prose");

fn viewport() -> TextViewport<'static> {
    TextViewport::new(VIEW).wrap(true)
}

/// The viewport owns only scroll and selection; source lines remain borrowed
/// from the deterministic prose fixture.
#[derive(Debug)]
pub(crate) struct ScrollingPage {
    lines: Vec<ViewportLine<'static>>,
    state: ViewportState,
    last: &'static str,
}

impl ScrollingPage {
    pub(crate) fn new() -> Self {
        let lines = PROSE.lines().map(ViewportLine::Plain).collect();
        let mut state = ViewportState::default();
        state.set_follow(false);
        Self { lines, state, last: "top of document" }
    }
}

impl Default for ScrollingPage {
    fn default() -> Self { Self::new() }
}

impl Page for ScrollingPage {
    fn title(&self) -> &'static str { "Scrolling" }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let result = viewport().update(cx, &mut self.state, &self.lines);
        if let Some(action) = result.action_ref() {
            self.last = match action {
                ViewportAction::SelectionChanged => "selection changed",
                ViewportAction::FollowChanged(true) => "following tail",
                ViewportAction::FollowChanged(false) => "manual scroll",
                ViewportAction::Copy(_) => "copied selection",
            };
        }
        result.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "wheel · PageUp/PageDown · drag scrollbar · select text", |ui, body| {
            let view_area = Rect { height: body.height.saturating_sub(2), ..body };
            viewport().draw(ui, view_area, &self.state, &self.lines);
            let summary = format!("offset={} · lines={} · {}", self.state.scroll().offset(), self.lines.len(), self.last);
            let _ = ui.paint_str(Rect { y: view_area.bottom(), height: 1, ..body }, &summary, ui.surface_style());
            lines(ui, Rect { y: view_area.bottom().saturating_add(1), height: 1, ..body }, &["The track reserves its column even when content fits; drag uses pointer capture."]);
        });
    }
}
