//! Editable task rows: keyed selection, commit/cancel and field validation.

use junie_tui::{
    Cx, FieldError, Id, ItemKey, List, ListAction, ListState, Rect, Response, RowUi, TextInput,
    TextInputState, Ui, id, layout,
};

use crate::data::{TASKS, TaskRow, TaskStatus};

use super::{Page, frame, lines};

const ROWS: Id = id!("editable.rows");
const NAME: Id = id!("editable.name");
const CHANGES: Id = id!("editable.changes");

#[derive(Clone, Debug, PartialEq, Eq)]
struct EditableRow {
    id: u32,
    name: String,
    owner: &'static str,
    status: TaskStatus,
    changes: String,
}

impl From<TaskRow> for EditableRow {
    fn from(row: TaskRow) -> Self {
        Self {
            id: row.id,
            name: row.name.to_owned(),
            owner: row.owner,
            status: row.status,
            changes: row.changes.to_string(),
        }
    }
}

fn row_key(row: &EditableRow) -> ItemKey {
    ItemKey::Num(u64::from(row.id))
}

fn row_view(row: &EditableRow, view: &mut RowUi<'_>) {
    view.label(row.name.as_str());
    view.meta(row.owner);
}

fn task_list() -> List<
    'static,
    EditableRow,
    impl Fn(&EditableRow) -> ItemKey,
    impl Fn(&EditableRow, &mut RowUi<'_>),
> {
    List::new(ROWS).key(row_key).row(row_view)
}

fn whole_number(value: &str) -> Result<(), FieldError> {
    if value.is_empty() || value.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(FieldError::new("Changes must be a whole number"))
    }
}

/// The selected record remains domain-owned while text states carry only the
/// in-flight edits.
#[derive(Debug)]
pub(crate) struct EditablePage {
    rows: Vec<EditableRow>,
    list_state: ListState,
    selected: usize,
    name_state: TextInputState,
    changes_state: TextInputState,
    message: &'static str,
}

impl EditablePage {
    pub(crate) fn new() -> Self {
        Self {
            rows: TASKS.iter().copied().map(EditableRow::from).collect(),
            list_state: ListState::default(),
            selected: 0,
            name_state: TextInputState::default(),
            changes_state: TextInputState::default(),
            message: "select a row and press Enter to edit",
        }
    }

    fn selected_key(&self) -> ItemKey {
        self.rows
            .get(self.selected)
            .map_or(ItemKey::Num(0), row_key)
    }
}

impl Default for EditablePage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for EditablePage {
    fn title(&self) -> &'static str {
        "Editable tables"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Response::ignored();
        let list = task_list().update(cx, &mut self.list_state, &self.rows);
        if let Some(ListAction::Chose(key) | ListAction::Activated(key)) = list.action_ref()
            && let Some(index) = self.rows.iter().position(|row| row_key(row) == *key)
        {
            self.selected = index;
            if matches!(list.action_ref(), Some(ListAction::Activated(_))) {
                self.message = "edit row fields";
                cx.focus(NAME);
            } else {
                self.message = "row selected";
            }
        }
        result |= list.erase();
        if let Some(row) = self.rows.get_mut(self.selected) {
            let name = TextInput::new(NAME).placeholder("Task name").update(
                cx,
                &mut self.name_state,
                &mut row.name,
            );
            if let Some(action) = name.action_ref() {
                self.message = match action {
                    junie_tui::TextAction::Committed => "name committed",
                    junie_tui::TextAction::Cancelled => "name edit cancelled",
                    junie_tui::TextAction::Changed => "name draft changed",
                    junie_tui::TextAction::MoveNext | junie_tui::TextAction::MovePrev => {
                        "focus moved"
                    }
                };
            }
            result |= name.erase();
            let changes = TextInput::new(CHANGES)
                .placeholder("Changes")
                .validate(&whole_number)
                .update(cx, &mut self.changes_state, &mut row.changes);
            let changes_action = changes.action_ref().copied();
            if let Some(action) = changes_action {
                self.message = match action {
                    junie_tui::TextAction::Committed => "changes committed",
                    junie_tui::TextAction::Cancelled => "changes edit cancelled",
                    junie_tui::TextAction::Changed => "changes draft changed",
                    junie_tui::TextAction::MoveNext | junie_tui::TextAction::MovePrev => {
                        "focus moved"
                    }
                };
            }
            let invalid_commit = matches!(changes_action, Some(junie_tui::TextAction::Committed))
                && self.changes_state.error().is_some();
            if invalid_commit {
                // TextInput ends a failed commit in Idle after writing the
                // controlled value. Re-arm the same field so the invalid
                // draft remains editable until the user fixes or cancels it.
                self.changes_state.begin(&row.changes);
                cx.focus(CHANGES);
            } else if matches!(changes_action, Some(junie_tui::TextAction::Cancelled)) {
                self.changes_state.set_error(None);
            }
            result |= changes.erase();
            if let Some(error) = self.changes_state.error() {
                self.message = if error.message.as_ref() == "Changes must be a whole number" {
                    "Changes must be a whole number"
                } else {
                    "changes invalid"
                };
            }
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "keyed rows · Enter edits · Esc cancels invalid drafts",
            |ui, body| {
                let (list_area, editor_area) = layout::split_h(body, body.width / 2);
                task_list().draw(ui, list_area, &self.list_state, &self.rows);
                let edit_rows = super::rows(editor_area, 4);
                if let Some(row) = self.rows.get(self.selected) {
                    TextInput::new(NAME)
                        .value(&row.name)
                        .placeholder("Task name")
                        .draw(
                            ui,
                            edit_rows.first().copied().unwrap_or(editor_area),
                            &self.name_state,
                        );
                    TextInput::new(CHANGES)
                        .value(&row.changes)
                        .placeholder("Changes")
                        .validate(&whole_number)
                        .draw(
                            ui,
                            edit_rows.get(1).copied().unwrap_or(editor_area),
                            &self.changes_state,
                        );
                    let info = format!(
                        "#{} · owner={} · status={:?}",
                        row.id, row.owner, row.status
                    );
                    let _ = ui.paint_str(
                        edit_rows.get(2).copied().unwrap_or(editor_area),
                        &info,
                        ui.surface_style(),
                    );
                }
                let mode = if self.name_state.is_editing() || self.changes_state.is_editing() {
                    "EDIT"
                } else {
                    "view"
                };
                let status = if self.changes_state.error().is_some() {
                    // Keep validation evidence visible as a complete message;
                    // the compact split view still has room for the editing
                    // affordance itself, which proves the invalid draft stayed
                    // in the edit lifecycle.
                    format!("EDIT · {}", self.message)
                } else {
                    format!("{mode} · key={:?} · {}", self.selected_key(), self.message)
                };
                let status_area = edit_rows.get(3).copied().unwrap_or(editor_area);
                let _ = ui.paint_str(status_area, &status, ui.surface_style());
                lines(
                    ui,
                    Rect {
                        y: status_area.y.saturating_add(1),
                        height: 1,
                        ..status_area
                    },
                    &["Validation is attached to the changes field; invalid commits stay in EDIT."],
                );
            },
        );
    }
}
