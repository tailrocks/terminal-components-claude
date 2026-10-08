//! Editable task rows: keyed selection, commit/cancel and field validation.

use termrock::{
    Align, CellDecor, CellRef, Column, ColumnKey, Cx, EditIntent, FgStep, FieldError, FrameRead,
    Grid, GridEditor, GridModel, GridState, Id, ItemKey, NavUnit, Panel, Rect, Role, RowDecor,
    RowTotal, StateFlags, StylePatch, Surface, Ui, id,
};

use showcase_data::{TASKS, TaskRow, TaskStatus};

use super::{Page, PageStatus, PageUpdate, frame};

const TABLE: Id = id!("editable.table");
const TASKS_PANEL: Id = id!("editable.tasks.panel");

const LEGEND_CHIP: StylePatch = StylePatch::new()
    .set_fg(Role::Surface(Surface::Canvas))
    .set_bg(Role::Fg(FgStep::Primary));
const LEGEND_CURSOR: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
const LEGEND_ERROR: StylePatch = StylePatch::new().set_fg(Role::Danger);
const LEGEND_TEXT: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));

const COLUMNS: [Column<'static>; 6] = [
    // GridState starts at column zero. Keep the historical ID-first paint
    // through the sticky column while making Task the first edit target.
    Column {
        key: ColumnKey::num(1),
        title: "Task",
        subtitle: None,
        align: Align::Left,
        min_width: 24,
        max_width: 48,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(0),
        title: "ID",
        subtitle: None,
        align: Align::Left,
        min_width: 5,
        max_width: 5,
        sortable: false,
        editable: false,
        sticky: true,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(2),
        title: "Owner",
        subtitle: None,
        align: Align::Left,
        min_width: 8,
        max_width: 8,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(3),
        title: "Status",
        subtitle: None,
        align: Align::Left,
        min_width: 9,
        max_width: 9,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(4),
        title: "Branch",
        subtitle: None,
        align: Align::Left,
        min_width: 22,
        max_width: 22,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(5),
        title: "Changes",
        subtitle: None,
        align: Align::Right,
        min_width: 8,
        max_width: 8,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
];

#[derive(Clone, Debug)]
struct EditableRow {
    id: u32,
    id_text: String,
    name: String,
    owner: String,
    status: TaskStatus,
    branch: String,
    branch_display: String,
    branch_error: bool,
    changes: String,
}

impl From<TaskRow> for EditableRow {
    fn from(row: TaskRow) -> Self {
        Self {
            id: row.id,
            id_text: format!("#{}", row.id),
            name: row.name.to_owned(),
            owner: row.owner.to_owned(),
            status: row.status,
            branch: if row.id == 1042 {
                "fix/checkout flake".to_owned()
            } else {
                row.branch.to_owned()
            },
            branch_display: if row.id == 1042 {
                "fix/checkout flake   !".to_owned()
            } else {
                row.branch.to_owned()
            },
            branch_error: row.id == 1042,
            changes: row.changes.to_string(),
        }
    }
}

#[derive(Debug)]
struct EditableModel {
    rows: Vec<EditableRow>,
    commits: u64,
}

impl EditableModel {
    fn new() -> Self {
        Self {
            rows: TASKS
                .iter()
                .copied()
                .take(14)
                .map(EditableRow::from)
                .collect(),
            commits: 0,
        }
    }
}

impl GridModel for EditableModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::num(0), |item| ItemKey::num(u64::from(item.id)))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let item = self.rows.get(row)?;
        Some(match col {
            0 => CellRef::new(item.name.as_str()),
            1 => CellRef::new(item.id_text.as_str())
                .align(Align::Left)
                .tone(Role::Fg(FgStep::Muted)),
            2 => CellRef::new(item.owner.as_str()),
            3 => match item.status {
                TaskStatus::Running => CellRef::new("▸ Running"),
                TaskStatus::Failed => CellRef::new("Failed").tone(Role::Danger),
                TaskStatus::Paused => CellRef::new("Paused").tone(Role::Warning),
                TaskStatus::Queued => CellRef::new("Queued").tone(Role::Fg(FgStep::Muted)),
                TaskStatus::Done => CellRef::new("Done").tone(Role::Fg(FgStep::Secondary)),
            },
            4 => CellRef::new(item.branch_display.as_str()).tone(Role::Fg(FgStep::Muted)),
            5 => CellRef::new(item.changes.as_str()).align(Align::Right),
            _ => return None,
        })
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        if col == 4 && self.rows.get(row).is_some_and(|item| item.branch_error) {
            CellDecor {
                tone: Some(Role::Danger),
                error: Some("Branch names cannot contain spaces"),
                ..CellDecor::default()
            }
        } else {
            CellDecor::default()
        }
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let mut decor = RowDecor::default();
        if self
            .rows
            .get(row)
            .is_some_and(|item| item.status == TaskStatus::Failed)
        {
            decor.tone = Some(Role::Danger);
        }
        decor
    }

    fn total(&self) -> RowTotal {
        RowTotal::Exact(self.rows.len())
    }
}

impl GridEditor for EditableModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_> {
        let Some(item) = self.rows.get(row) else {
            return EditIntent::Refuse {
                reason: "Unknown task row",
            };
        };
        let initial = match col {
            0 => item.name.as_str(),
            2 => item.owner.as_str(),
            4 => item.branch.as_str(),
            5 => item.changes.as_str(),
            _ => {
                return EditIntent::Refuse {
                    reason: "Cell is read-only",
                };
            }
        };
        EditIntent::Inline { initial }
    }

    fn apply_cycle(&mut self, _row: usize, _col: usize) {}

    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError> {
        let Some(item) = self.rows.get_mut(row) else {
            return Err(FieldError::new("Unknown task row"));
        };
        match col {
            0 if text.trim().is_empty() => {
                return Err(FieldError::new("Task name cannot be empty"));
            }
            2 if text.trim().is_empty() || text.contains(' ') => {
                return Err(FieldError::new("Owner is a single handle"));
            }
            4 if text.contains(' ') => {
                return Err(FieldError::new("Branch names cannot contain spaces"));
            }
            5 if text.parse::<u32>().is_err() => {
                return Err(FieldError::new("Changes must be a whole number"));
            }
            0 => text.clone_into(&mut item.name),
            2 => text.clone_into(&mut item.owner),
            4 => {
                text.clone_into(&mut item.branch);
                text.clone_into(&mut item.branch_display);
                item.branch_error = false;
            }
            5 => text.clone_into(&mut item.changes),
            _ => return Err(FieldError::new("Cell is read-only")),
        }
        self.commits = self.commits.saturating_add(1);
        Ok(())
    }

    fn is_editable(&self, _row: usize, col: usize) -> bool {
        matches!(col, 0 | 2 | 4 | 5)
    }
}

fn table() -> Grid<'static> {
    Grid::new(TABLE, &COLUMNS)
        .nav(NavUnit::Cell)
        .column_gap(2)
        .left_reserve(1)
        .right_reserve(2)
        .overflow_indicator(termrock::GridOverflowIndicator::Ellipsis)
}

/// `1–6 of 14`, or empty when every row fits (tag:scrollbar.rs
/// `position_label`). The viewport comes from draw-time panel geometry —
/// last-frame layout is unavailable on frame 1, which is what the
/// no-input default capture presents — following the tables page precedent.
fn position_label(state: &GridState, viewport: usize, len: usize) -> String {
    if len <= viewport || viewport == 0 {
        return String::new();
    }
    let mut scroll = *state.scroll();
    scroll.apply_layout(viewport, len);
    let r = scroll.visible_range();
    format!("{}–{} of {len}", r.start + 1, r.end)
}

/// Both phases derive the same task-card meta line, so neither can drift.
/// Update has no panel geometry, so it passes an empty position label; its
/// panel is discarded.
fn tasks_status<E: core::fmt::Display>(error: Option<E>, edits: u32, pos: &str) -> String {
    if let Some(error) = error {
        return error.to_string();
    }
    if pos.is_empty() {
        return format!("{edits} edits");
    }
    format!("{edits} edits · {pos}")
}

/// The one task-card constructor (§13), reached from update and from draw.
/// The card wears the grid's focus: the tag gates its `▎` head gutter on
/// `focused(table.id)`, and the unfocused boot frame shows a bare title.
fn tasks_panel(meta: &str, focused: bool) -> Panel<'_> {
    // No TITLE patch: the shared recipe already carries the tag rule
    // (secondary idle, primary+BOLD focused), and a patch would clobber the
    // focused arm.
    Panel::new(TASKS_PANEL)
        .title("Tasks")
        .meta(meta)
        .focused(focused)
}

/// The grid owns cursor and editor state; the model owns the editable task
/// records and their validation rules.
#[derive(Debug)]
pub struct EditablePage {
    model: EditableModel,
    state: GridState,
    edits: u32,
}

impl EditablePage {
    pub fn new() -> Self {
        Self {
            model: EditableModel::new(),
            state: GridState::default(),
            edits: 0,
        }
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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let was_editing = self.state.is_editing();
        let commits_before = self.model.commits;
        let action = table().update_editable(cx, &mut self.state, &mut self.model);
        if was_editing && !self.state.is_editing() {
            self.edits = self.edits.saturating_add(1);
        }
        let _ = tasks_panel(
            &tasks_status(self.state.edit_error(), self.edits, ""),
            false,
        );
        // (`tag:editable.rs:124-128`): a commit reports "Cell saved", a
        // cancel "Edit cancelled". The model counts successful commits, so
        // an edit that ends without one is a cancel.
        let mut update = PageUpdate::from(action.erase());
        if was_editing && !self.state.is_editing() {
            update.status = Some(PageStatus(
                if self.model.commits != commits_before {
                    "Cell saved"
                } else {
                    "Edit cancelled"
                }
                .to_owned(),
            ));
        }
        update
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let blurb = "Navigation is reversed cell; editing is a cursor. They never look alike.";
        frame(ui, area, self.title(), blurb, |ui, body| {
            let card_height = (self.model.rows.len() as u16)
                .saturating_add(4)
                .min(body.height.saturating_sub(4));
            let card = Rect {
                height: card_height,
                ..body
            };
            // −1: the grid header row (`Grid::chrome`; no note row on this
            // table, no actions bar).
            let viewport = tasks_panel("", false)
                .inner(ui, card)
                .height
                .saturating_sub(1) as usize;
            let pos = position_label(&self.state, viewport, self.model.rows.len());
            let task_meta = tasks_status(self.state.edit_error(), self.edits, &pos);
            // The grid itself derives FOCUSED from its inline editor while a
            // cell is being edited; the card mirrors that so its head gutter
            // stays lit through the edit.
            let focused = ui.state(TABLE).contains(StateFlags::FOCUSED)
                || ui.state(table().editor_id()).contains(StateFlags::FOCUSED);
            tasks_panel(&task_meta, focused).draw(ui, card, |ui, inner| {
                table().draw(ui, inner, &self.state, &self.model);
            });
            let legend_y = body.y.saturating_add(card_height).saturating_add(1);
            // (`tag:editable.rs:98-113`; frozen y23-25): the chip is a true
            // fg/bg swap, not `REVERSED` (which would set mods); the prose
            // is muted.
            let legend = [
                ("reversed", "cell cursor (navigation)", &LEGEND_CHIP),
                ("▁", "editing cursor + accent underline", &LEGEND_CURSOR),
                ("!", "validation error", &LEGEND_ERROR),
            ];
            for (offset, (glyph, text, chip)) in legend.iter().enumerate() {
                let Ok(offset) = u16::try_from(offset) else {
                    break;
                };
                let y = legend_y.saturating_add(offset);
                if y >= body.bottom() {
                    break;
                }
                let row = Rect {
                    y,
                    height: 1,
                    ..body
                };
                let glyph_width = if *glyph == "reversed" { 10 } else { 1 };
                let _ = ui.paint_str(
                    Rect {
                        width: glyph_width,
                        ..row
                    },
                    glyph,
                    ui.paint_patch(chip),
                );
                let _ = ui.paint_str(
                    Rect {
                        x: row.x.saturating_add(10),
                        width: row.width.saturating_sub(10),
                        ..row
                    },
                    text,
                    ui.paint_patch(&LEGEND_TEXT),
                );
            }
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.state.is_editing() {
            &[("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else {
            &[
                ("↑ ↓ ← →", "Cell"),
                ("Enter", "Edit"),
                ("s", "Sort"),
                ("click twice", "Edit"),
            ]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }
}
