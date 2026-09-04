//! Read-only terminal output with line scrolling.

use tui_next::{Cx, Id, Panel, PanelKind, Rect, Response, Status, TextArea, TextAreaState, Ui, id};

use crate::data::log_lines;

use super::{Page, frame, lines};

const OUTPUT: Id = id!("terminal.output");
const PANEL: Id = id!("terminal.panel");

/// Terminal output is a real read-only TextArea so wheel, PageDown and its
/// scrollbar share the same semantics as editable multiline text.
#[derive(Debug)]
pub(crate) struct TerminalPage {
    output: String,
    state: TextAreaState,
}

impl TerminalPage {
    pub(crate) fn new() -> Self {
        Self { output: log_lines(120).join("\n"), state: TextAreaState::default() }
    }
}

impl Default for TerminalPage {
    fn default() -> Self { Self::new() }
}

impl Page for TerminalPage {
    fn title(&self) -> &'static str { "Terminal" }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextArea::new(OUTPUT, 12)
            .read_only(true)
            .status(Status::Ready)
            .update(cx, &mut self.state, &mut self.output)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "read-only output · wheel · PageUp/PageDown", |ui, body| {
            Panel::new(PANEL).kind(PanelKind::Card).title("build output").draw(ui, body, |ui, inner| {
                TextArea::new(OUTPUT, 12)
                    .value(&self.output)
                    .read_only(true)
                    .status(Status::Ready)
                    .draw(ui, inner, &self.state);
            });
            lines(ui, Rect { y: body.bottom().saturating_sub(2), height: 2, ..body }, &["Output remains borrowed by the public control.", "status: ready"]);
        });
    }
}
