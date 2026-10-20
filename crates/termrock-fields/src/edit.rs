use ratatui_core::layout::Position;

use crate::Secret;
use crate::id::{Id, PartRef};
use crate::input::EditorDraft;
use crate::text::EditAction;
use crate::ui::Cx;
use termrock_runtime::layer::{FocusContinuationOutcome, LayerOpening};

/// Editing operation offered by a field's contextual edit menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextEditCommand {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

/// Commands currently available for a field's selection and editability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditCapabilities {
    /// Whether this field accepts edits. Independent of copy eligibility.
    pub editable: bool,
    pub can_cut: bool,
    pub can_copy: bool,
    pub can_paste: bool,
    pub can_select_all: bool,
}

/// A request to open the shared edit menu at a terminal position.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditMenuRequest {
    pub owner: Id,
    pub part: PartRef,
    pub anchor: Position,
}

pub(crate) fn capabilities(
    draft: &EditorDraft,
    committed_text: &str,
    is_editing: bool,
    disabled: bool,
    editable: bool,
    secret: bool,
    value_sensitive: bool,
) -> EditCapabilities {
    let selection = is_editing.then(|| draft.selection()).flatten();
    let selected = selection.is_some();
    let content = if is_editing {
        draft.text()
    } else {
        committed_text
    };
    let all_selected = selection.is_some_and(|range| range == (0..content.len()));
    let can_copy = selected && !disabled && !secret && !value_sensitive && !draft.is_sensitive();
    EditCapabilities {
        editable: editable && !disabled,
        can_cut: can_copy && editable,
        can_copy,
        can_paste: editable && !disabled,
        can_select_all: !disabled && !content.is_empty() && !all_selected,
    }
}

pub(crate) fn execute(
    cx: &mut Cx<'_>,
    draft: &mut EditorDraft,
    capabilities: EditCapabilities,
    command: TextEditCommand,
) -> bool {
    match command {
        TextEditCommand::Copy if capabilities.can_copy => {
            if let Some(range) = draft.selection() {
                cx.clipboard_copy(&draft.text()[range]);
                true
            } else {
                false
            }
        }
        TextEditCommand::Cut if capabilities.can_cut => {
            if let Some(range) = draft.selection() {
                cx.clipboard_copy(&draft.text()[range]);
                let changed = draft.apply(EditAction::Paste("")).changed();
                if changed {
                    cx.request_repaint();
                }
                changed
            } else {
                false
            }
        }
        TextEditCommand::Paste if capabilities.can_paste => {
            let Some(text) = cx.clipboard_text().map(|text| Secret::new(text.to_owned())) else {
                return false;
            };
            let changed = draft.apply(EditAction::Paste(text.expose())).changed();
            if changed {
                cx.request_repaint();
            }
            changed
        }
        TextEditCommand::SelectAll if capabilities.can_select_all => {
            let before = draft.selection();
            let _ = draft.apply(EditAction::SelectAll);
            let changed = before != draft.selection();
            if changed {
                cx.request_repaint();
            }
            changed
        }
        _ => false,
    }
}

/// Stage and acknowledge the exact layer opening that temporarily moves focus
/// away from a field. The field state is updated before the runtime records the
/// acknowledgement.
pub(crate) fn retain_focus_continuation(
    cx: &mut Cx<'_>,
    owner: Id,
    to: Option<Id>,
    pending: &mut Option<LayerOpening>,
) -> bool {
    let Some(to) = to else {
        *pending = None;
        return false;
    };
    let Some(opening) = cx.focus_continuation_opening(owner, to) else {
        *pending = None;
        return false;
    };
    *pending = Some(opening);
    if cx.acknowledge_focus_continuation(owner, to) {
        true
    } else {
        *pending = None;
        false
    }
}

pub(crate) fn continuation_result(
    cx: &mut Cx<'_>,
    owner: Id,
    opening: LayerOpening,
) -> Option<FocusContinuationOutcome> {
    cx.focus_continuation_result(opening, owner)
}

pub(crate) fn acknowledge_continuation_result(
    cx: &mut Cx<'_>,
    owner: Id,
    opening: LayerOpening,
) -> bool {
    cx.acknowledge_focus_continuation_result(opening, owner)
}
