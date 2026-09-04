//! Code editor page with cursor, insert mode and diagnostics.

use junie_tui::{
    CodeAction, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity, Cx, Id, Rect, Response,
    Ui, id,
};

use crate::data::CODE;

use super::{Page, frame, lines};

const EDITOR: Id = id!("editor.code");

fn editor() -> CodeEditor<'static> {
    CodeEditor::new(EDITOR, 12).placeholder("Start typing Rust…")
}

/// The editor's durable document is initialized from the legacy sample and
/// carries a warning marker to keep diagnostics visible in captures.
#[derive(Debug)]
pub(crate) struct EditorPage {
    state: CodeEditorState,
    last: &'static str,
}

impl EditorPage {
    pub(crate) fn new() -> Self {
        let mut state = CodeEditorState::new(CODE);
        state.set_diagnostics(vec![CodeDiagnostic::new(
            3..8,
            CodeSeverity::Info,
            "entry point",
        )]);
        Self {
            state,
            last: "read-only preview",
        }
    }
}

impl Default for EditorPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for EditorPage {
    fn title(&self) -> &'static str {
        "Editor"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let result = editor().update(cx, &mut self.state);
        if let Some(action) = result.action_ref() {
            self.last = match action {
                CodeAction::Changed => "document changed",
                CodeAction::CursorMoved => "cursor moved",
                CodeAction::Committed => "edit committed",
                CodeAction::Leave { .. } => "focus moved",
            };
        }
        result.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "code editor · diagnostics · insert mode",
            |ui, body| {
                let code_area = Rect {
                    height: body.height.saturating_sub(2),
                    ..body
                };
                editor().draw(ui, code_area, &self.state);
                let status = format!(
                    "lines={} · diagnostics={} · {}",
                    self.state.text().lines().count(),
                    self.state.diagnostics().len(),
                    self.last
                );
                let _ = ui.paint_str(
                    Rect {
                        y: code_area.bottom(),
                        height: 1,
                        ..body
                    },
                    &status,
                    ui.surface_style(),
                );
                lines(
                    ui,
                    Rect {
                        y: code_area.bottom().saturating_add(1),
                        height: 1,
                        ..body
                    },
                    &["F2/Insert edits; Esc/Enter commits through the public editor lifecycle."],
                );
            },
        );
    }
}
