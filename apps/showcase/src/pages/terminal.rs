//! Read-only terminal output with line scrolling.

use junie_tui::{
    Cx, Id, Panel, PanelKind, Rect, Response, Status, TextArea, TextAreaState, Ui, id,
};

use crate::data::log_lines;

use super::{Page, frame, lines};

const OUTPUT: Id = id!("terminal.output");
const PANEL: Id = id!("terminal.panel");

fn output() -> TextArea<'static> {
    TextArea::new(OUTPUT, 12)
        .read_only(true)
        .status(Status::Ready)
}

fn panel() -> Panel<'static> {
    Panel::new(PANEL)
        .kind(PanelKind::Card)
        .title("build output")
}

/// Terminal output is a real read-only `TextArea` so wheel, `PageDown` and its
/// scrollbar share the same semantics as editable multiline text.
#[derive(Debug)]
pub(crate) struct TerminalPage {
    output: String,
    state: TextAreaState,
}

impl TerminalPage {
    pub(crate) fn new() -> Self {
        Self {
            output: log_lines(120).join("\n"),
            state: TextAreaState::default(),
        }
    }
}

impl Default for TerminalPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TerminalPage {
    fn title(&self) -> &'static str {
        "Terminal"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = output()
            .update(cx, &mut self.state, &mut self.output)
            .erase();
        let _ = panel();
        response
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "read-only output · paging",
            |ui, body| {
                panel().draw(ui, body, |ui, inner| {
                    output().value(&self.output).draw(ui, inner, &self.state);
                });
                lines(
                    ui,
                    Rect {
                        y: body.bottom().saturating_sub(2),
                        height: 2,
                        ..body
                    },
                    &[
                        "Output remains borrowed by the public control.",
                        "status: ready",
                    ],
                );
            },
        );
    }
}
