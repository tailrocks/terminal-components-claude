#![allow(
    clippy::unwrap_used,
    reason = "focused public API regression assertions"
)]

use std::cell::RefCell;

use termrock_core::EditEntry;
use termrock_fields::{
    BlurPolicy, EditCapabilities, FocusSelect, TextArea, TextAreaState, TextCmd, TextEditCommand,
};
use termrock_runtime::{
    App, BindingState, Bindings, Cx, Focusability, Id, KeyCode, Rect, Response, StateFlags, Theme,
    Ui,
};
use termrock_test_support::Harness;

const ID: Id = Id::root("textarea.session-policy");
const OTHER: Id = Id::root("textarea.session-policy.other");
const AREA: Rect = Rect::new(0, 0, 28, 5);

struct Consumer {
    state: TextAreaState,
    value: String,
    textarea_edit_entry: EditEntry,
    focus_select: FocusSelect,
    esc_reverts: bool,
    blur: Option<BlurPolicy>,
    focus_on_bootstrap: bool,
    form_api: bool,
    inherited_disabled: bool,
    app_edit_entry: EditEntry,
    use_default_session_policies: bool,
    contextual_paste_once: bool,
    contextual_paste_attempted: bool,
    contextual_paste_result: Option<bool>,
    contextual_paste_capabilities: Option<EditCapabilities>,
    published_hints: RefCell<Vec<&'static str>>,
}

impl Consumer {
    fn new(
        value: &str,
        textarea_edit_entry: EditEntry,
        focus_select: FocusSelect,
        esc_reverts: bool,
    ) -> Self {
        Self {
            state: TextAreaState::default(),
            value: value.to_owned(),
            textarea_edit_entry,
            focus_select,
            esc_reverts,
            blur: None,
            focus_on_bootstrap: true,
            form_api: false,
            inherited_disabled: false,
            app_edit_entry: EditEntry::Immediate,
            use_default_session_policies: false,
            contextual_paste_once: false,
            contextual_paste_attempted: false,
            contextual_paste_result: None,
            contextual_paste_capabilities: None,
            published_hints: RefCell::new(Vec::new()),
        }
    }

    fn field(&self) -> TextArea<'static> {
        let field = TextArea::new(ID, 4);
        let field = if let Some(policy) = self.blur {
            field.blur(policy)
        } else {
            field
        };
        if self.use_default_session_policies {
            field
        } else {
            field
                .edit_entry(self.textarea_edit_entry)
                .focus_select(self.focus_select)
                .esc_reverts(self.esc_reverts)
        }
    }
}

impl App for Consumer {
    fn edit_entry(&self) -> EditEntry {
        self.app_edit_entry
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.focus_on_bootstrap && cx.update_cause() == termrock_runtime::UpdateCause::Bootstrap
        {
            cx.focus(ID);
        }
        let field = self.field();
        if self.contextual_paste_once && !self.contextual_paste_attempted {
            self.contextual_paste_attempted = true;
            cx.clipboard_copy("pasted");
            self.contextual_paste_capabilities =
                Some(field.edit_capabilities(&self.state, &self.value));
            self.contextual_paste_result = Some(field.execute_edit_command(
                cx,
                &mut self.state,
                &mut self.value,
                TextEditCommand::Paste,
            ));
        }
        if self.form_api {
            field
                .update_in_form(
                    cx,
                    &mut self.state,
                    &mut self.value,
                    self.inherited_disabled,
                )
                .erase()
        } else {
            field.update(cx, &mut self.state, &mut self.value).erase()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(OTHER, Rect::new(30, 0, 1, 1), Focusability::Focusable);
        let field = self.field();
        if self.form_api {
            field.draw_in_form(ui, AREA, &self.state, &self.value, self.inherited_disabled);
        } else {
            field.value(&self.value).draw(ui, AREA, &self.state);
        }
        if let Some(hints) = ui.with_focused_hints(|_, layer| {
            layer
                .hints
                .iter()
                .map(|hint| hint.label)
                .collect::<Vec<_>>()
        }) {
            *self.published_hints.borrow_mut() = hints;
        }
    }
}

fn harness(consumer: Consumer) -> Harness<Consumer> {
    Harness::new(consumer, Theme::junie(), 32, 7)
}

fn binding_labels(field: &TextArea<'static>, flags: StateFlags) -> Vec<&'static str> {
    field
        .bindings(BindingState { flags })
        .iter()
        .map(|binding| binding.label)
        .collect()
}

fn published_hints(h: &Harness<Consumer>) -> Vec<&'static str> {
    h.app().published_hints.borrow().as_slice().to_vec()
}

#[test]
fn defaults_remain_immediate_caret_end_and_escape_commit_without_app_inheritance() {
    let mut consumer = Consumer::new("seed", EditEntry::Immediate, FocusSelect::CaretEnd, false);
    consumer.app_edit_entry = EditEntry::Explicit;
    consumer.use_default_session_policies = true;
    let mut h = harness(consumer);

    assert!(h.app().state.is_editing());
    let _ = h.key(KeyCode::Char('X'));
    let _ = h.key(KeyCode::Esc);

    assert_eq!(h.app().value, "seedX");
    assert!(!h.app().state.is_editing());
}

#[test]
fn binding_tables_and_published_hints_follow_entry_phase_and_escape_policy() {
    let mut consumer = Consumer::new("seed", EditEntry::Immediate, FocusSelect::CaretEnd, false);
    consumer.use_default_session_policies = true;
    let h = harness(consumer);
    let default_hints = published_hints(&h);
    let default_bindings = binding_labels(&h.app().field(), StateFlags::FOCUSED);
    let default_enter_action = h
        .app()
        .field()
        .bindings(BindingState::default())
        .iter()
        .find(|binding| binding.cmd == TextCmd::Newline)
        .map(|binding| binding.action);
    let default_escape_action = h
        .app()
        .field()
        .bindings(BindingState::default())
        .iter()
        .find(|binding| binding.cmd == TextCmd::Commit)
        .map(|binding| binding.action);
    assert!(default_hints.contains(&"Done"));
    assert!(default_hints.contains(&"New line"));
    assert!(default_bindings.contains(&"Done"));
    assert!(default_bindings.contains(&"New line"));

    let h = harness(Consumer::new(
        "seed",
        EditEntry::TypeToEdit,
        FocusSelect::CaretEnd,
        false,
    ));
    let type_to_edit_hints = published_hints(&h);
    let type_to_edit_bindings = binding_labels(&h.app().field(), StateFlags::FOCUSED);
    assert!(!type_to_edit_hints.contains(&"Done"));
    assert!(!type_to_edit_hints.contains(&"New line"));
    assert!(!type_to_edit_bindings.contains(&"Done"));
    assert!(!type_to_edit_bindings.contains(&"New line"));

    let mut h = harness(Consumer::new(
        "seed",
        EditEntry::Explicit,
        FocusSelect::CaretEnd,
        true,
    ));
    let explicit_idle_hints = published_hints(&h);
    let explicit_idle_bindings = binding_labels(&h.app().field(), StateFlags::FOCUSED);
    assert!(explicit_idle_hints.contains(&"Begin editing"));
    assert!(!explicit_idle_hints.contains(&"Done"));
    assert!(!explicit_idle_hints.contains(&"New line"));
    assert!(explicit_idle_bindings.contains(&"Begin editing"));
    assert!(!explicit_idle_bindings.contains(&"Done"));
    assert!(!explicit_idle_bindings.contains(&"New line"));
    assert_eq!(
        h.app()
            .field()
            .bindings(BindingState::default())
            .iter()
            .find(|binding| binding.label == "Begin editing")
            .map(|binding| binding.cmd),
        Some(TextCmd::Newline)
    );
    assert_eq!(
        h.app()
            .field()
            .bindings(BindingState::default())
            .iter()
            .find(|binding| binding.label == "Begin editing")
            .map(|binding| binding.action),
        default_enter_action
    );
    let _ = h.key(KeyCode::Enter);
    let explicit_active_hints = published_hints(&h);
    assert!(explicit_active_hints.contains(&"Done"));
    assert!(!explicit_active_hints.contains(&"Cancel"));

    let mut h = harness(Consumer::new(
        "seed",
        EditEntry::TypeToEdit,
        FocusSelect::CaretEnd,
        true,
    ));
    let _ = h.key(KeyCode::Char('X'));
    let reverting_hints = published_hints(&h);
    let reverting_bindings =
        binding_labels(&h.app().field(), StateFlags::FOCUSED | StateFlags::EDITING);
    assert!(reverting_hints.contains(&"Cancel"));
    assert!(!reverting_hints.contains(&"Done"));
    assert!(reverting_bindings.contains(&"Cancel"));
    assert!(!reverting_bindings.contains(&"Done"));
    assert_eq!(
        h.app()
            .field()
            .bindings(BindingState {
                flags: StateFlags::FOCUSED | StateFlags::EDITING,
            })
            .iter()
            .find(|binding| binding.label == "Cancel")
            .map(|binding| binding.cmd),
        Some(TextCmd::Cancel)
    );
    assert_eq!(
        h.app()
            .field()
            .bindings(BindingState {
                flags: StateFlags::FOCUSED | StateFlags::EDITING,
            })
            .iter()
            .find(|binding| binding.cmd == TextCmd::Cancel)
            .map(|binding| binding.action),
        default_escape_action
    );
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seed");
}

#[test]
fn focus_selection_applies_to_new_non_pointer_sessions() {
    for entry in [
        EditEntry::Immediate,
        EditEntry::TypeToEdit,
        EditEntry::Explicit,
    ] {
        for (focus_select, expected) in [
            (FocusSelect::CaretStart, "Xab\ncd"),
            (FocusSelect::CaretEnd, "ab\ncdX"),
            (FocusSelect::SelectAll, "X"),
        ] {
            let mut h = harness(Consumer::new("ab\ncd", entry, focus_select, false));
            if entry == EditEntry::Explicit {
                let _ = h.key(KeyCode::Enter);
            }
            let _ = h.key(KeyCode::Char('X'));
            let _ = h.key(KeyCode::Esc);
            assert_eq!(h.app().value, expected, "{entry:?} with {focus_select:?}");
        }
    }
}

#[test]
fn type_to_edit_waits_for_typing_and_applies_a_whole_paste() {
    let mut h = harness(Consumer::new(
        "seed",
        EditEntry::TypeToEdit,
        FocusSelect::CaretEnd,
        false,
    ));
    assert!(!h.app().state.is_editing());

    let _ = h.key(KeyCode::Right);
    assert!(!h.app().state.is_editing());
    assert_eq!(h.app().value, "seed");

    let _ = h.key(KeyCode::Char('X'));
    assert!(h.app().state.is_editing());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seedX");

    let mut h = harness(Consumer::new(
        "seed",
        EditEntry::TypeToEdit,
        FocusSelect::CaretEnd,
        false,
    ));
    let _ = h.paste("one\ntwo");
    assert!(h.app().state.is_editing());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seedone\ntwo");

    let mut h = harness(Consumer::new(
        "replace this",
        EditEntry::TypeToEdit,
        FocusSelect::SelectAll,
        false,
    ));
    let replacement = "é雪\nwhole pasted value";
    let _ = h.paste(replacement);
    assert!(h.app().state.is_editing());
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, replacement);
}

#[test]
fn explicit_contextual_paste_is_rejected_while_idle_and_accepted_while_editing() {
    let mut consumer = Consumer::new("seed", EditEntry::Explicit, FocusSelect::CaretEnd, true);
    consumer.contextual_paste_once = true;
    let mut h = harness(consumer);

    assert_eq!(h.app().contextual_paste_result, Some(false));
    assert_eq!(
        h.app()
            .contextual_paste_capabilities
            .map(|capabilities| capabilities.can_paste),
        Some(false)
    );
    assert!(!h.app().state.is_editing());
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seed");

    let mut h = harness(Consumer::new(
        "seed",
        EditEntry::Explicit,
        FocusSelect::CaretEnd,
        true,
    ));
    let _ = h.key(KeyCode::Enter);
    h.app_mut().contextual_paste_once = true;
    let _ = h.key(KeyCode::Right);

    assert_eq!(h.app().contextual_paste_result, Some(true));
    assert_eq!(
        h.app()
            .contextual_paste_capabilities
            .map(|capabilities| capabilities.can_paste),
        Some(true)
    );
    assert!(h.app().state.is_editing());
    assert_eq!(h.app().value, "seed");
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seedpasted");
    assert!(!h.app().state.is_editing());

    let mut consumer = Consumer::new("seed", EditEntry::TypeToEdit, FocusSelect::CaretEnd, false);
    consumer.contextual_paste_once = true;
    let mut h = harness(consumer);
    assert_eq!(h.app().contextual_paste_result, Some(true));
    assert_eq!(
        h.app()
            .contextual_paste_capabilities
            .map(|capabilities| capabilities.can_paste),
        Some(true)
    );
    assert!(h.app().state.is_editing());
    assert_eq!(h.app().value, "seed");
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seedpasted");
}

#[test]
fn esc_reverts_cancels_non_explicit_sessions() {
    for entry in [EditEntry::Immediate, EditEntry::TypeToEdit] {
        let mut h = harness(Consumer::new("seed", entry, FocusSelect::CaretEnd, true));
        let _ = h.key(KeyCode::Char('X'));
        let _ = h.key(KeyCode::Esc);

        assert_eq!(h.app().value, "seed");
        assert!(!h.app().state.is_editing());
    }
}

#[test]
fn explicit_enter_starts_once_and_escape_commits_even_when_reverts_is_enabled() {
    let mut consumer = Consumer::new("ab\ncd", EditEntry::Explicit, FocusSelect::CaretEnd, true);
    consumer.form_api = true;
    let mut h = harness(consumer);
    assert!(!h.app().state.is_editing());

    let _ = h.key(KeyCode::Char('Z'));
    assert!(!h.app().state.is_editing());
    assert_eq!(h.app().value, "ab\ncd");

    let _ = h.key(KeyCode::Enter);
    assert!(h.app().state.is_editing());
    assert_eq!(h.app().value, "ab\ncd");

    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Char('X'));
    let _ = h.key(KeyCode::Esc);

    assert_eq!(h.app().value, "ab\ncd\nX");
    assert!(!h.app().state.is_editing());
}

#[test]
fn pointer_caret_placement_overrides_focus_selection() {
    let mut consumer = Consumer::new("abcd", EditEntry::Immediate, FocusSelect::SelectAll, false);
    consumer.focus_on_bootstrap = false;
    let mut h = harness(consumer);

    let _ = h.click(4, 0);
    let _ = h.key(KeyCode::Char('X'));
    let _ = h.key(KeyCode::Esc);

    assert_eq!(h.app().value, "abXcd");
}

#[test]
fn keep_blur_preserves_the_retained_caret_across_refocus() {
    let mut consumer = Consumer::new(
        "ab\ncd",
        EditEntry::Immediate,
        FocusSelect::CaretStart,
        false,
    );
    consumer.blur = Some(BlurPolicy::Keep);
    let mut h = harness(consumer);

    let _ = h.key(KeyCode::Char('X'));
    h.blur();
    assert!(h.app().state.is_editing());
    h.app_mut().textarea_edit_entry = EditEntry::TypeToEdit;
    h.app_mut().focus_select = FocusSelect::SelectAll;
    assert!(h.tab_to(ID));
    assert!(h.app().state.is_editing());

    h.app_mut().blur = Some(BlurPolicy::Commit);
    let _ = h.key(KeyCode::Char('Y'));
    let _ = h.key(KeyCode::Esc);

    assert_eq!(h.app().value, "XYab\ncd");
    assert!(!h.app().state.is_editing());
}

#[test]
fn keep_blur_preserves_explicit_escape_provenance_after_entry_policy_changes() {
    let mut consumer = Consumer::new("seed", EditEntry::Explicit, FocusSelect::CaretEnd, true);
    consumer.blur = Some(BlurPolicy::Keep);
    let mut h = harness(consumer);
    assert!(!h.app().state.is_editing());

    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Char('X'));
    h.blur();
    assert!(h.app().state.is_editing());

    h.app_mut().textarea_edit_entry = EditEntry::Immediate;
    assert!(h.tab_to(ID));
    assert!(h.app().state.is_editing());
    assert!(published_hints(&h).contains(&"Done"));
    assert!(!published_hints(&h).contains(&"Cancel"));

    let _ = h.key(KeyCode::Char('Y'));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().value, "seedXY");
    assert!(!h.app().state.is_editing());
}

#[test]
fn inherited_form_disabling_still_blocks_session_entry() {
    let mut consumer = Consumer::new("seed", EditEntry::TypeToEdit, FocusSelect::SelectAll, false);
    consumer.form_api = true;
    consumer.inherited_disabled = true;
    consumer.focus_on_bootstrap = false;
    let mut h = harness(consumer);

    let _ = h.click(4, 0);
    let _ = h.key(KeyCode::Char('X'));

    assert_eq!(h.app().value, "seed");
    assert!(!h.app().state.is_editing());
}
