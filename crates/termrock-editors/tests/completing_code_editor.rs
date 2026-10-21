use std::cell::RefCell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_core::action::ActionKey;
use termrock_core::id::{Id, ItemKey};
use termrock_editors::code::{CodeAction, CodeEditor};
use termrock_editors::completion::{
    CompletionBatch, CompletionProvider, CompletionTrigger, CompletionValue, FOCUS_NEXT_ACTION,
    MANUAL_COMPLETION_ACTION,
};
use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use termrock_runtime::focus::Focusability;
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Response, Ui};
use termrock_theme::Theme;

const EDITOR: Id = Id::root("completion.composite.editor");
const POPUP: Id = Id::root("completion.composite.popup");
const NEXT: Id = Id::root("completion.composite.next");
const AREA: Rect = Rect::new(0, 0, 100, 32);

struct Provider {
    triggers: RefCell<Vec<CompletionTrigger>>,
    mode: ProviderMode,
}

#[derive(Clone, Copy)]
enum ProviderMode {
    Normal,
    Empty,
    Disabled,
    Noop,
}

impl CompletionProvider for Provider {
    fn complete(
        &self,
        text: &str,
        _cursor: usize,
        trigger: CompletionTrigger,
    ) -> Option<CompletionBatch> {
        self.triggers.borrow_mut().push(trigger);
        let token = text.split_whitespace().last().unwrap_or_default();
        if matches!(self.mode, ProviderMode::Empty) {
            return Some(CompletionBatch {
                items: Vec::new(),
                replace_len: token.len(),
            });
        }
        if matches!(self.mode, ProviderMode::Noop) && token == "co" {
            return Some(CompletionBatch {
                items: vec![CompletionValue::new(ItemKey::num(1), "co", "co")],
                replace_len: token.len(),
            });
        }
        if !matches!(self.mode, ProviderMode::Disabled | ProviderMode::Normal)
            || !matches!(token, "co" | "cou")
        {
            return None;
        }
        let first = CompletionValue::new(ItemKey::num(1), "count", "count")
            .disabled(matches!(self.mode, ProviderMode::Disabled));
        Some(CompletionBatch {
            items: vec![
                first,
                CompletionValue::new(ItemKey::num(2), "country", "country"),
            ],
            replace_len: token.len(),
        })
    }
}

struct EditorApp {
    state: termrock_editors::CompletingCodeEditorState,
    provider: Provider,
    keymap: KeyMap,
    actions: Vec<CodeAction>,
    read_only: bool,
    disabled: bool,
    commit_on_blur: bool,
}

impl App for EditorApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = CodeEditor::new(EDITOR, 6)
            .read_only(self.read_only)
            .disabled(self.disabled)
            .with_completion(POPUP, &self.provider)
            .commit_on_blur(self.commit_on_blur)
            .update(cx, &mut self.state);
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        CodeEditor::new(EDITOR, 6)
            .read_only(self.read_only)
            .disabled(self.disabled)
            .with_completion(POPUP, &self.provider)
            .commit_on_blur(self.commit_on_blur)
            .draw(ui, Rect::new(0, 0, 80, 8), &self.state);
        ui.register_focus_only(NEXT, Focusability::Focusable);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn runtime(text: &str) -> Runtime<EditorApp> {
    runtime_with(text, ProviderMode::Normal)
}

fn runtime_with(text: &str, mode: ProviderMode) -> Runtime<EditorApp> {
    let mut state = termrock_editors::CompletingCodeEditorState::new(text);
    state.editor_mut().jump_to(text.len());
    state.editor_mut().begin_edit();
    let mut runtime = Runtime::new(
        EditorApp {
            state,
            provider: Provider {
                triggers: RefCell::new(Vec::new()),
                mode,
            },
            keymap: KeyMap::new(),
            actions: Vec::new(),
            read_only: false,
            disabled: false,
            commit_on_blur: true,
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.set_focus(Some(EDITOR));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime
}

fn runtime_preserving_blur(text: &str) -> Runtime<EditorApp> {
    let mut runtime = runtime(text);
    runtime.app_mut().commit_on_blur = false;
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime
}

fn key(code: KeyCode, mods: KeyModifiers) -> Input {
    Input::Key(Key { code, mods })
}

fn send(runtime: &mut Runtime<EditorApp>, code: KeyCode, mods: KeyModifiers) {
    let _ = termrock_runtime::stub::deliver(runtime, key(code, mods));
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
}

struct BareEditorApp {
    state: termrock_editors::code::CodeEditorState,
    actions: Vec<CodeAction>,
}

impl App for BareEditorApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = CodeEditor::new(EDITOR, 6).update(cx, &mut self.state);
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        CodeEditor::new(EDITOR, 6).draw(ui, Rect::new(0, 0, 80, 8), &self.state);
    }
}

fn send_bare(runtime: &mut Runtime<BareEditorApp>, code: KeyCode, mods: KeyModifiers) {
    let _ = termrock_runtime::stub::deliver(runtime, key(code, mods));
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
}

fn click(runtime: &mut Runtime<EditorApp>, x: u16, y: u16) {
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = termrock_runtime::stub::deliver(
            runtime,
            Input::Mouse(Mouse {
                kind,
                pos: Position::new(x, y),
                mods: KeyModifiers::NONE,
            }),
        );
        let mut buffer = Buffer::empty(AREA);
        runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    }
}

#[test]
fn typing_opens_completion_tab_accepts_then_next_tab_indents_two_spaces() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char('u'), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::TextChanged]
    );
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Changed));

    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT count");
    assert!(!runtime.app().state.is_completion_open());
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Changed));
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::TextChanged]
    );

    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT count  ");
    assert!(!runtime.app().state.is_completion_open());
}

#[test]
fn escape_dismisses_first_then_commits_editor() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));
    assert!(runtime.app().state.editor().is_editing());

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.editor().is_editing());
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Committed));
}

#[test]
fn remapped_manual_completion_uses_keymap_binding() {
    let mut runtime = runtime("SELECT co");
    runtime.app_mut().keymap.remap_component(
        EDITOR,
        MANUAL_COMPLETION_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(4)),
    );
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(!runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(4), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual, CompletionTrigger::Manual]
    );
}

#[test]
fn default_manual_completion_remains_available_while_popup_is_open() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual, CompletionTrigger::Manual]
    );
}

#[test]
fn manual_completion_is_ignored_while_find_is_typing() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert!(runtime.app().state.editor().is_find_typing());
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.editor().is_find_typing());
    assert!(!runtime.app().state.is_completion_open());
    assert!(runtime.app().provider.triggers.borrow().is_empty());
}

#[test]
fn bare_code_editor_ctrl_space_does_not_trigger_completions() {
    let mut state = termrock_editors::code::CodeEditorState::new("SELECT co");
    state.begin_edit();
    let mut runtime = Runtime::new(
        BareEditorApp {
            state,
            actions: Vec::new(),
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    runtime.set_focus(Some(EDITOR));
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    send_bare(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);

    assert_eq!(runtime.app().state.text(), "SELECT co");
    assert!(runtime.app().state.is_editing());
    assert!(!runtime.app().state.is_find_typing());
    assert!(runtime.app().actions.is_empty());
}

#[test]
fn draw_focus_and_ticks_do_not_request_completion() {
    let mut runtime = runtime("SELECT co");
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    assert!(runtime.app().provider.triggers.borrow().is_empty());
    assert!(!runtime.app().state.is_completion_open());
}

#[test]
fn cursor_motion_refreshes_only_an_open_completion() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Left, KeyModifiers::NONE);
    assert!(runtime.app().provider.triggers.borrow().is_empty());
    send(&mut runtime, KeyCode::Right, KeyModifiers::NONE);
    assert!(runtime.app().provider.triggers.borrow().is_empty());
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));
    let before = runtime.app().state.editor().cursor_offset();
    send(&mut runtime, KeyCode::Left, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().cursor_offset(), before - 1);
    assert_eq!(
        runtime.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual, CompletionTrigger::CursorMoved]
    );
    assert!(runtime.app().state.is_completion_open());
}

#[test]
fn remapped_focus_binding_works_while_completion_is_open() {
    let mut runtime = runtime("SELECT co");
    runtime.app_mut().keymap.remap_component(
        EDITOR,
        FOCUS_NEXT_ACTION,
        termrock_runtime::Chord::key(KeyCode::F(5)),
    );
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(6), KeyModifiers::NONE);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::F(5), KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert_eq!(runtime.focus(), Some(NEXT));
    assert_eq!(
        runtime.app().actions.last(),
        Some(&CodeAction::Leave { backward: false })
    );
}

#[test]
fn find_shortcut_closes_completion_before_find_keys_are_used() {
    let mut tab = runtime("SELECT co");
    send(&mut tab, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(tab.app().state.is_completion_open());

    send(&mut tab, KeyCode::Char('f'), KeyModifiers::CONTROL);
    assert!(!tab.app().state.is_completion_open());
    assert!(!tab.is_open(POPUP));
    assert!(tab.app().state.editor().is_find_typing());

    send(&mut tab, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(tab.app().state.editor().text(), "SELECT co");
    assert!(!tab.app().state.is_completion_open());
    assert!(tab.app().state.editor().is_find_typing());

    let mut enter = runtime("SELECT co");
    send(&mut enter, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut enter, KeyCode::Char('f'), KeyModifiers::CONTROL);
    send(&mut enter, KeyCode::Enter, KeyModifiers::NONE);
    assert!(!enter.app().state.editor().is_find_typing());
    assert!(!enter.app().state.is_completion_open());

    let mut escape = runtime("SELECT co");
    send(&mut escape, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut escape, KeyCode::Char('f'), KeyModifiers::CONTROL);
    send(&mut escape, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!escape.app().state.editor().is_find_typing());
    assert!(!escape.app().state.is_completion_open());
}

#[test]
fn printable_remapped_command_matches_with_completion_open_or_closed() {
    let mut closed = runtime("SELECT co");
    closed.app_mut().keymap.remap_component(
        EDITOR,
        ActionKey::custom("Left"),
        termrock_runtime::Chord::key(KeyCode::Char('h')),
    );
    let before = closed.app().state.editor().cursor_offset();
    send(&mut closed, KeyCode::Char('h'), KeyModifiers::NONE);
    let closed_result = (
        closed.app().state.editor().text().to_owned(),
        closed.app().state.editor().cursor_offset(),
    );
    assert_eq!(closed_result.0, "SELECT co");
    assert_eq!(closed_result.1, before - 1);

    let mut open = runtime("SELECT co");
    open.app_mut().keymap.remap_component(
        EDITOR,
        ActionKey::custom("Left"),
        termrock_runtime::Chord::key(KeyCode::Char('h')),
    );
    send(&mut open, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(open.app().state.is_completion_open());
    let before = open.app().state.editor().cursor_offset();
    send(&mut open, KeyCode::Char('h'), KeyModifiers::NONE);
    assert_eq!(
        (
            open.app().state.editor().text().to_owned(),
            open.app().state.editor().cursor_offset()
        ),
        closed_result
    );
    assert_eq!(open.app().state.editor().cursor_offset(), before - 1);
}

#[test]
fn f6_commits_and_moves_focus_in_both_directions() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::F(6), KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.app().state.editor().is_editing());
    assert_eq!(runtime.focus(), Some(NEXT));
    assert_eq!(
        runtime.app().actions.last(),
        Some(&CodeAction::Leave { backward: false })
    );

    runtime.set_focus(Some(EDITOR));
    runtime.app_mut().state.editor_mut().begin_edit();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    send(&mut runtime, KeyCode::F(6), KeyModifiers::SHIFT);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.app().state.editor().is_editing());
    assert_eq!(runtime.focus(), Some(NEXT));
    assert_eq!(
        runtime.app().actions.last(),
        Some(&CodeAction::Leave { backward: true })
    );
}

#[test]
fn readonly_or_disabled_editor_dismisses_without_provider_call() {
    for (read_only, disabled) in [(true, false), (false, true)] {
        let mut runtime = runtime("SELECT co");
        runtime.app_mut().read_only = read_only;
        runtime.app_mut().disabled = disabled;
        send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
        assert!(!runtime.app().state.is_completion_open());
        assert!(runtime.app().provider.triggers.borrow().is_empty());
    }
}

#[test]
fn empty_and_noop_acceptance_close_without_reopening() {
    let mut empty = runtime_with("SELECT x", ProviderMode::Empty);
    send(&mut empty, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(!empty.app().state.is_completion_open());

    let mut noop = runtime_with("SELECT co", ProviderMode::Noop);
    send(&mut noop, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(noop.app().state.is_completion_open());
    send(&mut noop, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(noop.app().state.editor().text(), "SELECT co");
    assert!(!noop.app().state.is_completion_open());
    assert_eq!(
        noop.app().provider.triggers.borrow().as_slice(),
        &[CompletionTrigger::Manual]
    );

    let debug = format!("{:?}", noop.app().state);
    assert!(!debug.contains("SELECT co"));
}

#[test]
fn completion_controller_closes_and_reopens_the_same_popup_owner() {
    let mut runtime = runtime("SELECT co");
    runtime.app_mut().state.editor_mut().commit();
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert!(!runtime.app().state.editor().is_editing());

    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.editor().is_editing());
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));

    send(&mut runtime, KeyCode::Esc, KeyModifiers::NONE);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));

    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.is_open(POPUP));
    assert_eq!(runtime.focus(), Some(EDITOR));
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}

#[test]
fn disabled_suggestion_cannot_be_accepted() {
    let mut runtime = runtime_with("SELECT co", ProviderMode::Disabled);
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    send(&mut runtime, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(runtime.app().state.editor().text(), "SELECT co");
    assert!(runtime.app().state.is_completion_open());
}

#[test]
fn outside_pointer_and_focus_loss_dismiss_and_commit() {
    let mut runtime = runtime("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    click(&mut runtime, 98, 30);
    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));
    assert!(runtime.app().state.editor().is_editing());

    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    runtime.set_focus(Some(NEXT));
    assert_eq!(runtime.focus(), Some(NEXT), "focus request before settling");
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    assert_eq!(
        runtime.focus(),
        Some(NEXT),
        "focus after settling the popup close"
    );
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    assert!(!runtime.app().state.is_completion_open());
    assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Committed));
    assert_eq!(runtime.focus(), Some(NEXT));
    assert!(!runtime.app().state.editor().is_editing());
}

#[test]
fn focus_loss_dismisses_completion_but_preserves_configured_edit_session() {
    let mut runtime = runtime_preserving_blur("SELECT co");
    send(&mut runtime, KeyCode::Char(' '), KeyModifiers::CONTROL);
    assert!(runtime.app().state.is_completion_open());
    assert!(runtime.app().state.editor().is_editing());
    let text = runtime.app().state.editor().text().to_owned();
    let cursor = runtime.app().state.editor().cursor_offset();

    runtime.set_focus(Some(NEXT));
    let mut buffer = Buffer::empty(AREA);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();
    let _ = termrock_runtime::stub::deliver(&mut runtime, Input::Tick);
    runtime.draw_buffer(AREA, &mut buffer).commit_presented();

    assert!(!runtime.app().state.is_completion_open());
    assert!(!runtime.is_open(POPUP));
    assert!(runtime.app().state.editor().is_editing());
    assert_eq!(runtime.app().state.editor().text(), text);
    assert_eq!(runtime.app().state.editor().cursor_offset(), cursor);
    assert_ne!(runtime.app().actions.last(), Some(&CodeAction::Committed));
    assert_eq!(runtime.focus(), Some(NEXT));
    assert!(
        runtime.diagnostics().is_empty(),
        "{:?}",
        runtime.diagnostics()
    );
}
