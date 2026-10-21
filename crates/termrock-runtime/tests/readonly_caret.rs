#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "runtime fixture assertions"
)]

use ratatui_core::buffer::Buffer;
use termrock::{
    App, CodeEditor, CodeEditorState, Cx, Focusability, Id, Rect, Response, Runtime, TextInput,
    TextInputState, Theme, Ui, UpdateCause,
};

const EDITOR: Id = Id::root("readonly-caret.editor");
const INPUT: Id = Id::root("readonly-caret.input");
const OTHER: Id = Id::root("readonly-caret.other");
const AREA: Rect = Rect::new(0, 0, 40, 8);

#[derive(Clone, Copy)]
enum FocusTarget {
    Editor,
    Input,
    Other,
}

struct Probe {
    target: FocusTarget,
    disable_editor: bool,
    editor: CodeEditorState,
    input: TextInputState,
    input_value: String,
}

impl Probe {
    fn editor(&self) -> CodeEditor<'_> {
        CodeEditor::new(EDITOR, 4)
            .read_only(true)
            .disabled(self.disable_editor)
    }
}

impl App for Probe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(match self.target {
                FocusTarget::Editor => EDITOR,
                FocusTarget::Input => INPUT,
                FocusTarget::Other => OTHER,
            });
        }
        let disabled = self.disable_editor;
        let _ = CodeEditor::new(EDITOR, 4)
            .read_only(true)
            .disabled(disabled)
            .update(cx, &mut self.editor);
        let _ = TextInput::new(INPUT).read_only(true).update(
            cx,
            &mut self.input,
            &mut self.input_value,
        );
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.editor().draw(ui, Rect::new(0, 0, 30, 5), &self.editor);
        TextInput::new(INPUT)
            .read_only(true)
            .value(&self.input_value)
            .draw(ui, Rect::new(0, 6, 30, 1), &self.input);
        ui.register_control(OTHER, Rect::new(0, 7, 10, 1), Focusability::Focusable);
    }
}

fn runtime(target: FocusTarget, disable_editor: bool) -> Runtime<Probe> {
    let mut rt = Runtime::new(
        Probe {
            target,
            disable_editor,
            editor: CodeEditorState::new("read only"),
            input: TextInputState::default(),
            input_value: String::from("read only"),
        },
        Theme::junie(),
    );
    let _ = rt.initialize();
    rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
        .commit_presented();
    rt
}

#[test]
fn focused_readonly_code_editor_keeps_its_published_bar_caret() {
    let rt = runtime(FocusTarget::Editor, false);

    assert_eq!(rt.focus(), Some(EDITOR));
    assert_eq!(rt.typing_owner(), None);
    assert!(rt.cursor_position().is_some());
    assert!(
        !rt.diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic, termrock::Diagnostic::CursorRejected { .. }))
    );
}

#[test]
fn readonly_fields_without_an_ordinary_cursor_request_do_not_gain_a_caret() {
    for (target, expected_focus) in [(FocusTarget::Input, INPUT), (FocusTarget::Other, OTHER)] {
        let rt = runtime(target, false);
        assert_eq!(rt.focus(), Some(expected_focus));
        assert_eq!(
            rt.cursor_position(),
            None,
            "read-only text input or unfocused editor gained a caret"
        );
    }

    let rt = runtime(FocusTarget::Editor, true);
    assert_ne!(rt.focus(), Some(EDITOR), "disabled editor cannot own focus");
    assert_eq!(rt.cursor_position(), None);
}
