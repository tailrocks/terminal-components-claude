use std::cell::RefCell;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_controls::Button;
use termrock_core::edit_entry::EditEntry;
use termrock_core::id::Id;
use termrock_editors::{CodeAction, CodeCmd, CodeEditor, CodeEditorState};
use termrock_runtime::event::{Input, Key, KeyCode, KeyModifiers};
use termrock_runtime::keymap::{Binding, BindingState, Bindings};
use termrock_runtime::response::StateFlags;
use termrock_runtime::runtime::{App, Runtime};
use termrock_runtime::{Cx, Response, Ui};
use termrock_theme::Theme;

const SCREEN: Rect = Rect::new(0, 0, 80, 24);
const EDITOR_AREA: Rect = Rect::new(4, 4, 64, 8);
const ELSEWHERE_AREA: Rect = Rect::new(0, 0, 16, 1);
const EDITOR: Id = Id::root("c4.code.binding_state.editor");
const ELSEWHERE: Id = Id::root("c4.code.binding_state.elsewhere");

struct CodeApp {
    app_entry: Option<EditEntry>,
    widget_entry: Option<EditEntry>,
    read_only: bool,
    disabled: bool,
    state: CodeEditorState,
    actions: Vec<CodeAction>,
    live_hint_labels: RefCell<Vec<&'static str>>,
}

impl CodeApp {
    fn editor(&self) -> CodeEditor<'static> {
        let editor = CodeEditor::new(EDITOR, EDITOR_AREA.height)
            .read_only(self.read_only)
            .disabled(self.disabled);
        match self.widget_entry {
            Some(entry) => editor.edit_entry(entry),
            None => editor,
        }
    }
}

impl App for CodeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let _ = Button::new(ELSEWHERE, "Elsewhere").update(cx);
        let response = self.editor().update(cx, &mut self.state);
        if let Some(action) = response.action_ref() {
            self.actions.push(action.clone());
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let _ = Button::new(ELSEWHERE, "Elsewhere").draw(ui, ELSEWHERE_AREA);
        self.editor().draw(ui, EDITOR_AREA, &self.state);

        // Read the live frame publication, not CodeEditor::bindings' state adapter.
        let labels = ui
            .with_focused_hints(|_, layer| {
                layer
                    .hints
                    .iter()
                    .map(|hint| hint.label)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        *self.live_hint_labels.borrow_mut() = labels;
    }

    fn edit_entry(&self) -> EditEntry {
        self.app_entry.unwrap_or_default()
    }
}

fn app(
    text: &str,
    app_entry: Option<EditEntry>,
    widget_entry: Option<EditEntry>,
    read_only: bool,
    disabled: bool,
) -> CodeApp {
    CodeApp {
        app_entry,
        widget_entry,
        read_only,
        disabled,
        state: CodeEditorState::new(text),
        actions: Vec::new(),
        live_hint_labels: RefCell::new(Vec::new()),
    }
}

fn present(runtime: &mut Runtime<CodeApp>) {
    let mut buffer = Buffer::empty(SCREEN);
    runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
}

fn new_runtime(
    app_entry: Option<EditEntry>,
    widget_entry: Option<EditEntry>,
    read_only: bool,
    disabled: bool,
) -> Runtime<CodeApp> {
    let mut runtime = Runtime::new(
        app("seed", app_entry, widget_entry, read_only, disabled),
        Theme::junie(),
    );
    let _ = runtime.initialize();
    present(&mut runtime);
    focus(&mut runtime, ELSEWHERE);
    assert_eq!(runtime.focus(), Some(ELSEWHERE));
    assert!(
        !runtime.app().state.is_editing(),
        "editor must be nonediting before the tested FocusIn"
    );
    runtime
}

fn focus(runtime: &mut Runtime<CodeApp>, target: Id) {
    runtime.set_focus(Some(target));
    let _ = runtime.settle();
    assert!(
        runtime.diagnostics().is_empty(),
        "focus settlement diagnostics: {:?}",
        runtime.diagnostics()
    );
    present(runtime);
    assert!(
        runtime.diagnostics().is_empty(),
        "focus presentation diagnostics: {:?}",
        runtime.diagnostics()
    );
}

fn focused_editor(
    app_entry: Option<EditEntry>,
    widget_entry: Option<EditEntry>,
    read_only: bool,
    disabled: bool,
) -> Runtime<CodeApp> {
    let mut runtime = new_runtime(app_entry, widget_entry, read_only, disabled);
    assert_eq!(runtime.focus(), Some(ELSEWHERE));
    focus(&mut runtime, EDITOR);
    runtime
}

fn key(runtime: &mut Runtime<CodeApp>, code: KeyCode) -> Response<()> {
    let response = termrock_runtime::runtime::stub::deliver(
        runtime,
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }),
    );
    assert!(
        runtime.diagnostics().is_empty(),
        "key delivery diagnostics for {code:?}: {:?}",
        runtime.diagnostics()
    );
    present(runtime);
    assert!(
        runtime.diagnostics().is_empty(),
        "key presentation diagnostics for {code:?}: {:?}",
        runtime.diagnostics()
    );
    response
}

fn has_label(table: &[Binding<CodeCmd>], expected: &str) -> bool {
    table.iter().any(|binding| binding.label == expected)
}

fn assert_live_navigation(runtime: &Runtime<CodeApp>) {
    let labels = runtime.app().live_hint_labels.borrow();
    assert!(labels.contains(&"Edit (i)"), "live labels: {labels:?}");
    assert!(!labels.contains(&"Done"), "live labels: {labels:?}");
}

fn assert_live_editing(runtime: &Runtime<CodeApp>) {
    let labels = runtime.app().live_hint_labels.borrow();
    assert!(labels.contains(&"Done"), "live labels: {labels:?}");
    assert!(labels.contains(&"New line"), "live labels: {labels:?}");
    assert!(!labels.contains(&"Edit (i)"), "live labels: {labels:?}");
}

fn assert_adapter_matches_runtime(runtime: &Runtime<CodeApp>, editing: bool) {
    let flags = runtime.state_of(EDITOR);
    let table = runtime.app().editor().bindings(BindingState { flags });
    assert_eq!(
        has_label(table, "Done"),
        editing,
        "runtime flags: {flags:?}"
    );
    assert_eq!(
        has_label(table, "Edit (i)"),
        !editing,
        "runtime flags: {flags:?}"
    );
}

#[test]
fn state_adapter_matches_runtime_state_across_edit_entry_policies() {
    for (app_entry, widget_entry, editing) in [
        (None, None, true),
        (Some(EditEntry::TypeToEdit), None, false),
        (Some(EditEntry::Explicit), None, false),
        (Some(EditEntry::Explicit), Some(EditEntry::Immediate), true),
    ] {
        let runtime = focused_editor(app_entry, widget_entry, false, false);
        assert_eq!(
            runtime.state_of(EDITOR).contains(StateFlags::EDITING),
            editing,
            "app entry {app_entry:?}, widget entry {widget_entry:?}"
        );
        assert_adapter_matches_runtime(&runtime, editing);
    }
}

#[test]
fn explicit_and_type_to_edit_apps_publish_navigation_until_editing_starts() {
    let mut explicit = focused_editor(Some(EditEntry::Explicit), None, false, false);
    assert_eq!(explicit.focus(), Some(EDITOR));
    assert!(explicit.state_of(EDITOR).contains(StateFlags::FOCUSED));
    assert!(!explicit.app().state.is_editing());
    assert_live_navigation(&explicit);
    assert_adapter_matches_runtime(&explicit, false);

    let _ = key(&mut explicit, KeyCode::Char('x'));
    assert_eq!(explicit.app().state.text(), "seed");
    assert!(!explicit.app().state.is_editing());
    assert_live_navigation(&explicit);
    assert_adapter_matches_runtime(&explicit, false);

    let enter = key(&mut explicit, KeyCode::Enter);
    assert!(enter.is_changed());
    assert!(explicit.app().state.is_editing());
    assert_eq!(explicit.app().state.text(), "seed");
    assert_live_editing(&explicit);
    assert_adapter_matches_runtime(&explicit, true);

    let inserted = key(&mut explicit, KeyCode::Char('x'));
    assert!(inserted.is_changed());
    assert_eq!(explicit.app().state.text(), "xseed");
    assert_eq!(explicit.app().actions.last(), Some(&CodeAction::Changed));

    let committed = key(&mut explicit, KeyCode::Esc);
    assert!(committed.is_changed());
    assert_eq!(explicit.app().actions.last(), Some(&CodeAction::Committed));
    assert_eq!(explicit.focus(), Some(EDITOR));
    assert!(explicit.state_of(EDITOR).contains(StateFlags::FOCUSED));
    assert!(!explicit.app().state.is_editing());
    assert_eq!(explicit.app().state.text(), "xseed");
    assert_live_navigation(&explicit);
    assert_adapter_matches_runtime(&explicit, false);

    let mut type_to_edit = focused_editor(Some(EditEntry::TypeToEdit), None, false, false);
    assert_eq!(type_to_edit.focus(), Some(EDITOR));
    assert!(type_to_edit.state_of(EDITOR).contains(StateFlags::FOCUSED));
    assert!(!type_to_edit.app().state.is_editing());
    assert_live_navigation(&type_to_edit);
    assert_adapter_matches_runtime(&type_to_edit, false);

    let typed = key(&mut type_to_edit, KeyCode::Char('q'));
    assert!(typed.is_changed());
    assert!(type_to_edit.app().state.is_editing());
    assert_eq!(type_to_edit.app().state.text(), "qseed");
    assert_eq!(
        type_to_edit.app().actions.last(),
        Some(&CodeAction::Changed)
    );
    assert_live_editing(&type_to_edit);
    assert_adapter_matches_runtime(&type_to_edit, true);

    let committed = key(&mut type_to_edit, KeyCode::Esc);
    assert!(committed.is_changed());
    assert_eq!(
        type_to_edit.app().actions.last(),
        Some(&CodeAction::Committed)
    );
    assert_eq!(type_to_edit.focus(), Some(EDITOR));
    assert!(!type_to_edit.app().state.is_editing());
    assert_eq!(type_to_edit.app().state.text(), "qseed");
    assert_live_navigation(&type_to_edit);
    assert_adapter_matches_runtime(&type_to_edit, false);
}

#[test]
fn immediate_focus_and_widget_override_publish_edit_then_escape_returns_to_navigation() {
    for (app_entry, widget_entry) in [
        (None, None),
        (Some(EditEntry::Explicit), Some(EditEntry::Immediate)),
    ] {
        let mut runtime = focused_editor(app_entry, widget_entry, false, false);
        assert_eq!(runtime.focus(), Some(EDITOR));
        assert!(runtime.state_of(EDITOR).contains(StateFlags::FOCUSED));
        assert!(runtime.app().state.is_editing());
        assert_live_editing(&runtime);
        assert_adapter_matches_runtime(&runtime, true);

        let inserted = key(&mut runtime, KeyCode::Char('z'));
        assert!(inserted.is_changed());
        assert_eq!(runtime.app().state.text(), "zseed");
        assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Changed));

        let committed = key(&mut runtime, KeyCode::Esc);
        assert!(committed.is_changed());
        assert_eq!(runtime.app().actions.last(), Some(&CodeAction::Committed));
        assert_eq!(runtime.focus(), Some(EDITOR));
        assert!(runtime.state_of(EDITOR).contains(StateFlags::FOCUSED));
        assert!(!runtime.app().state.is_editing());
        assert_eq!(runtime.app().state.text(), "zseed");
        assert_live_navigation(&runtime);
        assert_adapter_matches_runtime(&runtime, false);
    }
}

#[test]
fn read_only_and_disabled_editors_keep_text_and_nonediting_state() {
    let mut read_only = focused_editor(Some(EditEntry::Immediate), None, true, false);
    assert_eq!(read_only.focus(), Some(EDITOR));
    assert!(read_only.state_of(EDITOR).contains(StateFlags::READ_ONLY));
    assert!(!read_only.app().state.is_editing());
    assert_live_navigation(&read_only);
    assert_adapter_matches_runtime(&read_only, false);
    let blocked = key(&mut read_only, KeyCode::Enter);
    assert!(blocked.is_consumed());
    assert!(!read_only.app().state.is_editing());
    assert_eq!(read_only.app().state.text(), "seed");

    let mut disabled = new_runtime(Some(EditEntry::Immediate), None, false, true);
    assert_eq!(disabled.focus(), Some(ELSEWHERE));
    assert!(disabled.state_of(EDITOR).contains(StateFlags::DISABLED));
    assert!(!disabled.state_of(EDITOR).contains(StateFlags::FOCUSED));
    assert!(!disabled.app().state.is_editing());
    assert_adapter_matches_runtime(&disabled, false);

    let _ = key(&mut disabled, KeyCode::Tab);
    assert_eq!(disabled.focus(), Some(ELSEWHERE));
    assert!(disabled.state_of(EDITOR).contains(StateFlags::DISABLED));
    assert!(!disabled.state_of(EDITOR).contains(StateFlags::FOCUSED));
    let _ = key(&mut disabled, KeyCode::Enter);
    let _ = key(&mut disabled, KeyCode::Char('x'));
    assert_eq!(disabled.focus(), Some(ELSEWHERE));
    assert!(!disabled.app().state.is_editing());
    assert_eq!(disabled.app().state.text(), "seed");
}
