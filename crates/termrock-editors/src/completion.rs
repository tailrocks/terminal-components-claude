use core::fmt;

use ratatui_core::layout::Rect;

use termrock_core::action::ActionKey;
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::Response;
use termrock_layout::{Constraints, Size};
use termrock_overlays::completion::{
    AsItem, Completion, CompletionAction, CompletionController, CompletionState, Item,
};
use termrock_runtime::event::{Chord, KeyCode, KeyModifiers};
use termrock_runtime::intent::Intent;
use termrock_runtime::ui::{Cx, FrameRead, Ui};

use crate::code::{CodeAction, CodeEditor, CodeEditorState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionTrigger {
    TextChanged,
    CursorMoved,
    Manual,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CompletionValue {
    key: ItemKey,
    label: String,
    glyph: String,
    matched: Vec<usize>,
    insertion: String,
    detail: String,
    disabled: bool,
}

impl CompletionValue {
    pub fn new(key: ItemKey, label: impl Into<String>, insertion: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            glyph: String::new(),
            matched: Vec::new(),
            insertion: insertion.into(),
            detail: String::new(),
            disabled: false,
        }
    }

    #[must_use]
    pub fn glyph(mut self, glyph: impl Into<String>) -> Self {
        self.glyph = glyph.into();
        self
    }

    #[must_use]
    pub fn matched(mut self, indices: impl Into<Vec<usize>>) -> Self {
        self.matched = indices.into();
        self
    }

    #[must_use]
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }

    #[must_use]
    pub const fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl fmt::Debug for CompletionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletionValue")
            .field("key", &self.key)
            .field("label", &"[redacted]")
            .field("glyph", &self.glyph)
            .field("matched_count", &self.matched.len())
            .field("insertion", &"[redacted]")
            .field("detail", &"[redacted]")
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl AsItem for CompletionValue {
    fn as_item(&self) -> Item<'_> {
        Item::new(self.key, &self.label)
            .glyph(&self.glyph)
            .matched(&self.matched)
            .insert(&self.insertion)
            .detail(&self.detail)
            .disabled(self.disabled)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CompletionBatch {
    pub items: Vec<CompletionValue>,
    pub replace_len: usize,
}

impl fmt::Debug for CompletionBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletionBatch")
            .field("item_count", &self.items.len())
            .field("replace_len", &self.replace_len)
            .finish()
    }
}

pub trait CompletionProvider {
    fn complete(
        &self,
        text: &str,
        cursor: usize,
        trigger: CompletionTrigger,
    ) -> Option<CompletionBatch>;
}

impl<F> CompletionProvider for F
where
    F: Fn(&str, usize, CompletionTrigger) -> Option<CompletionBatch>,
{
    fn complete(
        &self,
        text: &str,
        cursor: usize,
        trigger: CompletionTrigger,
    ) -> Option<CompletionBatch> {
        self(text, cursor, trigger)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CompletingCodeEditorState {
    editor: CodeEditorState,
    completion: CompletionState,
    items: Vec<CompletionValue>,
}

impl fmt::Debug for CompletingCodeEditorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletingCodeEditorState")
            .field("editor", &self.editor)
            .field("completion_open", &self.completion.is_open())
            .field("completion_item_count", &self.items.len())
            .finish()
    }
}

impl CompletingCodeEditorState {
    pub fn new(text: &str) -> Self {
        Self {
            editor: CodeEditorState::new(text),
            completion: CompletionState::default(),
            items: Vec::new(),
        }
    }

    pub const fn editor(&self) -> &CodeEditorState {
        &self.editor
    }

    pub fn editor_mut(&mut self) -> &mut CodeEditorState {
        &mut self.editor
    }

    pub const fn is_completion_open(&self) -> bool {
        self.completion.is_open()
    }
}

pub struct CompletingCodeEditor<'a> {
    editor: CodeEditor<'a>,
    id: Id,
    popup_id: Id,
    provider: &'a dyn CompletionProvider,
}

impl fmt::Debug for CompletingCodeEditor<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompletingCodeEditor")
            .field("id", &self.id)
            .field("popup_id", &self.popup_id)
            .finish_non_exhaustive()
    }
}

impl<'a> CompletingCodeEditor<'a> {
    pub(crate) const fn new(
        editor: CodeEditor<'a>,
        id: Id,
        popup_id: Id,
        provider: &'a dyn CompletionProvider,
    ) -> Self {
        Self {
            editor,
            id,
            popup_id,
            provider,
        }
    }

    fn completion(&self) -> Completion<'_, CompletionValue> {
        Completion::new(self.popup_id)
    }

    /// Configure whether leaving the editor commits its edit session.
    ///
    /// The default is to commit. Set to `false` when the owning view temporarily removes the
    /// editor while preserving its edit session. Completion popups still dismiss on focus loss.
    #[must_use]
    pub const fn commit_on_blur(mut self, commit: bool) -> Self {
        self.editor = self.editor.commit_on_blur(commit);
        self
    }

    /// Open the editor's find bar, dismissing any active completion popup first.
    pub fn open_find(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletingCodeEditorState,
    ) -> Response<CodeAction> {
        if self.editor.is_disabled() {
            return Response::ignored();
        }
        CompletionController::new(self.id, self.popup_id).dismiss(cx, &mut state.completion);
        state.items.clear();
        self.editor.open_find(cx, &mut state.editor)
    }

    /// Measure the editor using its existing intrinsic sizing contract.
    pub fn measure(&self, ui: &Ui<'_>, constraints: Constraints) -> Size {
        self.editor.measure(ui, constraints)
    }

    fn request(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletingCodeEditorState,
        trigger: CompletionTrigger,
    ) {
        let Some(anchor) = self.editor.cursor_cell(cx, &state.editor) else {
            CompletionController::new(self.id, self.popup_id).dismiss(cx, &mut state.completion);
            state.items.clear();
            return;
        };
        let Some(batch) =
            self.provider
                .complete(state.editor.text(), state.editor.cursor_offset(), trigger)
        else {
            CompletionController::new(self.id, self.popup_id).dismiss(cx, &mut state.completion);
            state.items.clear();
            return;
        };
        state.items = batch.items;
        CompletionController::new(self.id, self.popup_id).request(
            cx,
            &mut state.completion,
            anchor,
            batch.replace_len,
            &state.items,
        );
    }

    pub fn update(
        &self,
        cx: &mut Cx<'_>,
        state: &mut CompletingCodeEditorState,
    ) -> Response<CodeAction> {
        let focus_out = cx
            .intents(self.id)
            .any(|intent| matches!(intent, Intent::FocusOut { .. }));
        if focus_out && state.completion.is_open() {
            CompletionController::new(self.id, self.popup_id).dismiss(cx, &mut state.completion);
            state.items.clear();
        }
        let was_open = state.completion.is_open();
        let manual_requested = cx.intents(self.id).any(|intent| {
            matches!(intent, Intent::Binding(action) if action == MANUAL_COMPLETION_ACTION)
        });
        if self.editor.is_disabled() || self.editor.is_read_only() {
            CompletionController::new(self.id, self.popup_id).dismiss(cx, &mut state.completion);
            state.items.clear();
            return self.editor.update(cx, &mut state.editor);
        }

        let completing = self.completion();
        let completion_response =
            completing.update_for(self.id, cx, &mut state.completion, &state.items);
        let mut dismissed_outside = false;
        match completion_response.action_ref() {
            Some(CompletionAction::Accepted(key)) => {
                let Some(item) = state
                    .items
                    .iter()
                    .find(|item| item.key == *key)
                    .map(AsItem::as_item)
                else {
                    CompletionController::new(self.id, self.popup_id)
                        .dismiss(cx, &mut state.completion);
                    state.items.clear();
                    return if completion_response.is_changed() {
                        Response::changed()
                    } else {
                        Response::consumed()
                    };
                };
                let inserted = CompletionController::new(self.id, self.popup_id).accept_with(
                    cx,
                    &mut state.completion,
                    item,
                    |replace_len, insertion| state.editor.splice_completion(replace_len, insertion),
                );
                state.items.clear();
                return if inserted {
                    Response::action(CodeAction::Changed)
                } else {
                    Response::consumed()
                };
            }
            Some(CompletionAction::Dismissed) => {
                state.items.clear();
                let escape = cx.intents(self.id).any(|intent| match intent {
                    Intent::Cancel => true,
                    Intent::Binding(action) => action == DISMISS_COMPLETION_ACTION,
                    _ => false,
                });
                let owner_pointer = cx
                    .intents(self.id)
                    .any(|intent| matches!(intent, Intent::Pointer { .. }));
                let owner_binding = cx
                    .intents(self.id)
                    .any(|intent| matches!(intent, Intent::Binding(_)));
                let focus_out = cx
                    .intents(self.id)
                    .any(|intent| matches!(intent, Intent::FocusOut { .. }));
                if escape || (!owner_binding && !owner_pointer && !focus_out) {
                    return if completion_response.is_changed() {
                        Response::changed()
                    } else {
                        Response::consumed()
                    };
                }
                dismissed_outside = owner_pointer && !owner_binding;
            }
            Some(CompletionAction::Moved) | None => {}
        }

        let editor_response = self.editor.update(cx, &mut state.editor);
        if manual_requested {
            if !state.editor.is_find_typing()
                && !self.editor.is_read_only()
                && !self.editor.is_disabled()
            {
                state.editor.begin_edit();
                self.request(cx, state, CompletionTrigger::Manual);
                return Response::changed();
            }
            return Response::consumed();
        }
        if was_open && state.editor.is_find_typing() {
            CompletionController::new(self.id, self.popup_id).dismiss(cx, &mut state.completion);
            state.items.clear();
            return Self::combine_response(&completion_response, editor_response);
        }
        let action = editor_response.action_ref().cloned();
        let editor_response = match action {
            Some(CodeAction::Changed)
                if !self.editor.is_read_only() && !self.editor.is_disabled() =>
            {
                self.request(cx, state, CompletionTrigger::TextChanged);
                editor_response.repaint()
            }
            Some(CodeAction::CursorMoved) if was_open && !dismissed_outside => {
                self.request(cx, state, CompletionTrigger::CursorMoved);
                editor_response.repaint()
            }
            Some(CodeAction::Committed | CodeAction::Leave { .. }) => {
                CompletionController::new(self.id, self.popup_id)
                    .dismiss(cx, &mut state.completion);
                state.items.clear();
                editor_response
            }
            _ => editor_response,
        };
        Self::combine_response(&completion_response, editor_response)
    }

    fn combine_response(
        completion: &Response<CompletionAction>,
        editor: Response<CodeAction>,
    ) -> Response<CodeAction> {
        if editor.is_changed() {
            editor
        } else if completion.is_changed() {
            Response::changed()
        } else if editor.is_consumed() || completion.is_consumed() {
            Response::consumed()
        } else {
            editor
        }
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &CompletingCodeEditorState) -> Rect {
        let used = self.editor.draw(ui, area, &state.editor);
        if state.completion.is_open() {
            let _ = ui.layer(self.popup_id, |ui, popup_area| {
                self.completion()
                    .draw(ui, popup_area, &state.completion, &state.items);
                let _ = ui.retain_owner_cursor(self.id);
            });
        }
        let live = ui.state(self.id);
        ui.publish_dynamic_bindings(
            self.id,
            live,
            core::iter::once((MANUAL_COMPLETION_ACTION, Some(COMPLETION_MANUAL_CHORD))),
        );
        used
    }
}

pub const COMPLETION_MANUAL_CHORD: Chord = Chord::with(KeyCode::Char(' '), KeyModifiers::CONTROL);
pub const COMPLETION_NEXT_FOCUS_CHORD: Chord = Chord::key(KeyCode::F(6));
pub const COMPLETION_PREV_FOCUS_CHORD: Chord = Chord::with(KeyCode::F(6), KeyModifiers::SHIFT);

pub const MANUAL_COMPLETION_ACTION: ActionKey = ActionKey::custom("completion.manual");
pub const FOCUS_NEXT_ACTION: ActionKey = ActionKey::custom("focus.next");
pub const FOCUS_PREVIOUS_ACTION: ActionKey = ActionKey::custom("focus.previous");
const DISMISS_COMPLETION_ACTION: ActionKey = ActionKey::custom("completion.dismiss");
