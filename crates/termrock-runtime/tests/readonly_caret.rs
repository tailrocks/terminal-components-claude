#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "runtime fixture assertions"
)]

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Position;
use termrock::{
    App, CodeAction, CodeEditor, CodeEditorState, Cx, Focusability, Id, KeyCode, Part, PartRef,
    Rect, Response, Runtime, StateFlags, TextInput, TextInputState, Theme, Ui, UpdateCause,
};
use termrock_test_support::Harness;

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
    assert!(rt.cursor_bar());
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

const CLICK_EDITOR: Id = Id::root("readonly-click.editor");
const CLICK_PEER: Id = Id::root("readonly-click.peer");
const CLICK_SCREEN: (u16, u16) = (48, 12);
const CLICK_CODE_AREA: Rect = Rect::new(3, 1, 38, 5);
const CLICK_PEER_AREA: Rect = Rect::new(3, 8, 22, 1);
const CLICK_DOCUMENT: &str = "abcde\n0123456789\nlast";

struct ClickProbe {
    code_state: CodeEditorState,
    peer_state: TextInputState,
    peer_value: String,
    read_only: bool,
    disabled: bool,
    code_actions: Vec<CodeAction>,
}

impl ClickProbe {
    fn code(&self) -> CodeEditor<'_> {
        CodeEditor::new(CLICK_EDITOR, 4)
            .read_only(self.read_only)
            .disabled(self.disabled)
    }
}

impl App for ClickProbe {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(CLICK_PEER);
        }
        let read_only = self.read_only;
        let disabled = self.disabled;
        let code_response = CodeEditor::new(CLICK_EDITOR, 4)
            .read_only(read_only)
            .disabled(disabled)
            .update(cx, &mut self.code_state);
        if let Some(action) = code_response.action_ref() {
            self.code_actions.push(action.clone());
        }
        let _ = TextInput::new(CLICK_PEER).update(cx, &mut self.peer_state, &mut self.peer_value);
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.code().draw(ui, CLICK_CODE_AREA, &self.code_state);
        TextInput::new(CLICK_PEER).value(&self.peer_value).draw(
            ui,
            CLICK_PEER_AREA,
            &self.peer_state,
        );
    }
}

fn click_harness(text: &str, read_only: bool, disabled: bool) -> Harness<ClickProbe> {
    Harness::new(
        ClickProbe {
            code_state: CodeEditorState::new(text),
            peer_state: TextInputState::default(),
            peer_value: String::from("peer"),
            read_only,
            disabled,
            code_actions: Vec::new(),
        },
        Theme::junie(),
        CLICK_SCREEN.0,
        CLICK_SCREEN.1,
    )
}

fn code_text_area(harness: &Harness<ClickProbe>) -> Rect {
    harness
        .area_of_part(CLICK_EDITOR, PartRef::of(Part::TEXT))
        .expect("the presented CodeEditor publishes its real TEXT part")
}

fn assert_no_click_diagnostics(harness: &Harness<ClickProbe>) {
    assert!(
        harness.diagnostics().is_empty(),
        "the click journey should not emit runtime diagnostics"
    );
}

#[test]
fn readonly_code_click_moves_the_inspection_caret_without_claiming_document_input() {
    let mut harness = click_harness(CLICK_DOCUMENT, true, false);
    assert_no_click_diagnostics(&harness);
    assert_eq!(harness.focus(), Some(CLICK_PEER));
    assert_eq!(harness.runtime().typing_owner(), Some(CLICK_PEER));

    let text = code_text_area(&harness);
    assert!(
        text.x > 0 && text.y > 0,
        "the editor and its text part are offset"
    );
    assert!(
        text.width >= 5 && text.height >= 2,
        "the click target is presented"
    );
    let target = Position::new(text.x + 3, text.y + 1);
    assert_eq!(harness.cell(target.x, target.y).symbol(), "3");
    assert_eq!(harness.app().code_state.cursor_offset(), 0);
    assert_ne!(harness.app().code_state.cursor_offset(), 9);

    harness.app_mut().code_actions.clear();
    let _ = harness.click(target.x, target.y);
    assert_no_click_diagnostics(&harness);

    assert_eq!(harness.focus(), Some(CLICK_EDITOR));
    assert_eq!(harness.cursor(), Some(target));
    assert!(harness.cursor_bar());
    assert_eq!(harness.app().code_state.cursor_offset(), 9);
    assert_eq!(harness.app().code_state.cursor_pos().line, 1);
    assert_eq!(harness.app().code_state.cursor_pos().col, 3);
    assert_eq!(harness.app().code_state.text(), CLICK_DOCUMENT);
    assert!(!harness.app().code_state.is_editing());
    assert_eq!(harness.runtime().typing_owner(), None);
    assert!(
        harness
            .app()
            .code_actions
            .contains(&CodeAction::CursorMoved)
    );
    assert!(
        !harness
            .app()
            .code_actions
            .iter()
            .any(|action| { matches!(action, CodeAction::Changed | CodeAction::Committed) })
    );

    let _ = harness.type_str("Q");
    let _ = harness.paste("tail");
    let _ = harness.key(KeyCode::Char('i'));
    assert_eq!(harness.app().code_state.text(), CLICK_DOCUMENT);
    assert!(!harness.app().code_state.is_editing());
    assert_eq!(harness.runtime().typing_owner(), None);
    assert_no_click_diagnostics(&harness);

    assert!(harness.tab_to(CLICK_PEER));
    assert_eq!(harness.focus(), Some(CLICK_PEER));
    assert_eq!(harness.runtime().typing_owner(), Some(CLICK_PEER));
    assert!(harness.state_of(CLICK_PEER).contains(StateFlags::FOCUSED));
    assert!(!harness.state_of(CLICK_EDITOR).contains(StateFlags::FOCUSED));
    let peer_text = harness
        .area_of_part(CLICK_PEER, PartRef::of(Part::TEXT))
        .expect("the writable peer publishes its real TEXT part");
    assert_eq!(harness.app().peer_state.cursor_offset(), Some(4));
    assert_eq!(
        harness
            .cell(peer_text.x.saturating_add(3), peer_text.y)
            .symbol(),
        "r"
    );
    assert_eq!(
        harness.cursor(),
        Some(Position::new(peer_text.x.saturating_add(4), peer_text.y)),
        "the writable peer keeps its ordinary end-of-text caret"
    );
    assert_no_click_diagnostics(&harness);
}

#[test]
fn writable_code_click_keeps_existing_edit_entry_and_text_placement() {
    let mut harness = click_harness(CLICK_DOCUMENT, false, false);
    assert_no_click_diagnostics(&harness);
    let text = code_text_area(&harness);
    let target = Position::new(text.x + 3, text.y + 1);
    assert_eq!(harness.cell(target.x, target.y).symbol(), "3");

    let _ = harness.click(target.x, target.y);
    assert_no_click_diagnostics(&harness);
    assert_eq!(harness.focus(), Some(CLICK_EDITOR));
    assert_eq!(harness.cursor(), Some(target));
    assert_eq!(harness.app().code_state.cursor_offset(), 9);
    assert!(harness.app().code_state.is_editing());
    assert_eq!(harness.runtime().typing_owner(), Some(CLICK_EDITOR));

    let _ = harness.type_str("!");
    assert_eq!(harness.app().code_state.text(), "abcde\n012!3456789\nlast");
    assert_no_click_diagnostics(&harness);
}

#[test]
fn readonly_wide_character_click_uses_its_grapheme_start_byte_and_cell() {
    let document = "a界b\nnext";
    let mut harness = click_harness(document, true, false);
    assert_no_click_diagnostics(&harness);
    let text = code_text_area(&harness);
    let target = Position::new(text.x + 1, text.y);
    assert_eq!(harness.cell(target.x, target.y).symbol(), "界");

    let _ = harness.click(target.x, target.y);
    assert_no_click_diagnostics(&harness);
    assert_eq!(harness.cursor(), Some(target));
    assert_eq!(harness.app().code_state.cursor_offset(), 1);
    assert_eq!(harness.app().code_state.cursor_pos().line, 0);
    assert_eq!(harness.app().code_state.cursor_pos().col, 1);
    assert_eq!(harness.app().code_state.text(), document);
    assert!(!harness.app().code_state.is_editing());
    assert_eq!(harness.runtime().typing_owner(), None);
}

#[test]
fn disabled_code_click_preserves_the_peer_focus_and_editor_state() {
    let mut harness = click_harness(CLICK_DOCUMENT, true, true);
    assert_no_click_diagnostics(&harness);
    assert_eq!(harness.focus(), Some(CLICK_PEER));
    let before = harness.app().code_state.clone();
    let text = code_text_area(&harness);
    let target = Position::new(text.x + 3, text.y + 1);
    assert_eq!(harness.cell(target.x, target.y).symbol(), "3");

    let _ = harness.click(target.x, target.y);
    assert_no_click_diagnostics(&harness);

    assert_eq!(harness.focus(), Some(CLICK_PEER));
    assert_eq!(harness.app().code_state, before);
    assert_eq!(harness.runtime().typing_owner(), Some(CLICK_PEER));
}
