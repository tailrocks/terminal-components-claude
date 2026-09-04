//! Code editor page with cursor, insert mode and diagnostics.

use junie_tui::{
    CodeAction, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity, Completion,
    CompletionState, Cx, DiffView, DiffViewState, Id, Item, ItemKey, Rect, Response, Ui, id,
    layout,
};

use crate::data::CODE;

use super::{Page, frame, lines};

const EDITOR: Id = id!("editor.code");
const COMPLETION: Id = id!("editor.completion");
const DIFF: Id = id!("editor.diff");
const SUGGESTIONS: &[Item<'static>] = &[
    Item::new(ItemKey::Num(101), "fn").detail("function keyword"),
    Item::new(ItemKey::Num(102), "let").detail("binding keyword"),
    Item::new(ItemKey::Num(103), "match").detail("pattern match"),
];

fn editor() -> CodeEditor<'static> {
    CodeEditor::new(EDITOR, 12).placeholder("Start typing Rust…")
}

fn completion() -> Completion<'static, Item<'static>> {
    Completion::new(COMPLETION).max_rows(3)
}

fn diff() -> DiffView<'static> {
    DiffView::new(DIFF, None)
}

/// The editor's durable document is initialized from the legacy sample and
/// carries a warning marker to keep diagnostics visible in captures.
#[derive(Debug)]
pub(crate) struct EditorPage {
    state: CodeEditorState,
    completion_state: CompletionState,
    diff_state: DiffViewState,
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
            completion_state: CompletionState::default(),
            diff_state: DiffViewState::default(),
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
        let editor_response = editor().update(cx, &mut self.state);
        if let Some(action) = editor_response.action_ref() {
            self.last = match action {
                CodeAction::Changed => "document changed",
                CodeAction::CursorMoved => "cursor moved",
                CodeAction::Committed => "edit committed",
                CodeAction::Leave { .. } => "focus moved",
            };
        }
        let mut result = editor_response.erase();
        result |= completion()
            .update_for(EDITOR, cx, &mut self.completion_state, SUGGESTIONS)
            .erase();
        result |= diff().update(cx, &mut self.diff_state).erase();
        result.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "code editor · diagnostics · insert mode",
            |ui, body| {
                let (code_area, lower) = layout::split_v(body, body.height.saturating_sub(8));
                editor().draw(ui, code_area, &self.state);
                let (status_area, lower) = layout::split_v(lower, 1);
                let status = format!(
                    "lines={} · diagnostics={} · {}",
                    self.state.text().lines().count(),
                    self.state.diagnostics().len(),
                    self.last
                );
                let _ = ui.paint_str(status_area, &status, ui.surface_style());
                let (diff_area, completion_area) = layout::split_h(lower, lower.width / 2);
                diff().draw(ui, diff_area, &self.diff_state);
                completion().draw(ui, completion_area, &self.completion_state, SUGGESTIONS);
                lines(
                    ui,
                    Rect {
                        y: lower.bottom().saturating_sub(1),
                        height: 1,
                        ..lower
                    },
                    &["F2/Insert edits; the review and completion panes use public components."],
                );
            },
        );
    }
}
