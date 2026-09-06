//! Editable task rows: keyed selection, commit/cancel and field validation.

use junie_tui::{
    id, Align, CellDecor, CellRef, Column, ColumnKey, Cx, EditIntent, FgStep, FieldError, Grid,
    GridEditor, GridModel, GridState, Id, ItemKey, NavUnit, Panel, Part, Rect, Response, Role,
    RowDecor, RowTotal, StylePatch, Ui,
};

use crate::data::{TaskRow, TaskStatus, TASKS};

use super::{frame, Page};

const TABLE: Id = id!("editable.table");
const TASKS_PANEL: Id = id!("editable.tasks.panel");
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(junie_tui::Modifier::BOLD),
)];

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
    },
    Column {
        key: ColumnKey::num(0),
        title: "ID",
        subtitle: None,
        align: Align::Right,
        min_width: 5,
        max_width: 5,
        sortable: false,
        editable: false,
        sticky: true,
        prefix_glyph: None,
        badge: None,
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

fn status_text(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Running => "▸ Running",
        TaskStatus::Failed => "Failed",
        TaskStatus::Paused => "Paused",
        TaskStatus::Queued => "Queued",
        TaskStatus::Done => "Done",
    }
}

#[derive(Debug)]
struct EditableModel {
    rows: Vec<EditableRow>,
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
            1 => CellRef::new(item.id_text.as_str()).align(Align::Right),
            2 => CellRef::new(item.owner.as_str()),
            3 => CellRef::new(status_text(item.status)),
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

fn padded(value: &str, width: usize) -> String {
    let value = junie_tui::truncate(value, width as u16);
    format!("{value:<width$}")
}

fn legacy_header(width: u16) -> String {
    if width >= 130 {
        format!(
            "{} {} {} {} {} {}",
            padded("ID", 6),
            padded("Task", 64),
            padded("Owner", 9),
            padded("Status", 10),
            padded("Branch", 24),
            "Changes"
        )
    } else if width >= 90 {
        format!(
            "{} {} {} {} {} …",
            padded("ID", 6),
            padded("Task", 34),
            padded("Owner", 9),
            padded("Status", 10),
            padded("Branch", 22),
        )
    } else if width >= 70 {
        format!(
            "{} {} {} {} …",
            padded("ID", 6),
            padded("Task", 43),
            padded("Owner", 9),
            padded("Status", 10),
        )
    } else {
        format!(
            "{} {} {}…",
            padded("ID", 6),
            padded("Task", 33),
            padded("Owner", 9),
        )
    }
}

fn legacy_row(row: &EditableRow, width: u16, track: &str) -> String {
    let status = status_text(row.status);
    if width >= 130 {
        format!(
            "▎  {} {} {} {} {} {}",
            padded(&row.id_text, 6),
            padded(&row.name, 64),
            padded(&row.owner, 9),
            padded(status, 10),
            padded(&row.branch_display, 24),
            padded(&row.changes, 8),
        )
    } else if width >= 90 {
        format!(
            "▎  {} {}  {} {} {}",
            padded(&row.id_text, 6),
            padded(&row.name, 33),
            padded(&row.owner, 9),
            padded(status, 10),
            row.branch_display,
        )
    } else if width >= 70 {
        format!(
            "▎  {} {} {} {}",
            padded(&row.id_text, 6),
            padded(&row.name, 43),
            padded(&row.owner, 9),
            padded(status, 10),
        )
    } else {
        format!(
            "▎  {} {}  {} {track}",
            padded(&row.id_text, 6),
            padded(&row.name, 32),
            padded(&row.owner, 9),
        )
    }
}

fn legacy_table(ui: &mut Ui<'_>, area: Rect, width: u16, model: &EditableModel) {
    if area.is_empty() {
        return;
    }
    let header_style = ui
        .style(
            junie_tui::Family::GRID,
            junie_tui::Variant::DEFAULT,
            Part::HEADER,
            junie_tui::StateFlags::empty(),
        )
        .style;
    let row_style = ui
        .style(
            junie_tui::Family::GRID,
            junie_tui::Variant::DEFAULT,
            Part::ROW,
            junie_tui::StateFlags::empty(),
        )
        .style;
    let header = Rect {
        x: area.x.saturating_add(3),
        width: area.width.saturating_sub(3),
        height: 1,
        ..area
    };
    ui.fill(header, header_style);
    let _ = ui.paint_str(header, &legacy_header(width), header_style);
    let visible = usize::from(area.height.saturating_sub(1));
    let thumb = visible
        .saturating_mul(visible)
        .checked_div(model.rows.len().max(1))
        .unwrap_or(1)
        .max(1);
    for (offset, row) in model.rows.iter().take(visible).enumerate() {
        let track = if width < 70 {
            if offset < thumb {
                "┃"
            } else {
                "│"
            }
        } else {
            ""
        };
        let row_area = Rect {
            y: area.y.saturating_add(1).saturating_add(offset as u16),
            height: 1,
            ..area
        };
        ui.fill(row_area, row_style);
        let _ = ui.paint_str(row_area, &legacy_row(row, width, track), row_style);
    }
}

fn paint_card_meta(ui: &mut Ui<'_>, area: Rect, text: &str) {
    if text.is_empty() || area.is_empty() {
        return;
    }
    let style = ui
        .style(
            junie_tui::Family::PANEL,
            junie_tui::Variant::DEFAULT,
            Part::DETAIL,
            junie_tui::StateFlags::empty(),
        )
        .style;
    let text_width = junie_tui::width(text);
    let x = area.right().saturating_sub(text_width.saturating_add(2));
    let width = area.right().saturating_sub(x);
    ui.fill(
        Rect {
            x,
            y: area.y,
            width,
            height: 1,
        },
        style,
    );
    let _ = ui.paint_str(
        Rect {
            x,
            y: area.y,
            width: text_width,
            height: 1,
        },
        text,
        style,
    );
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
            0 => item.name = text.to_owned(),
            2 => item.owner = text.to_owned(),
            4 => {
                item.branch = text.to_owned();
                item.branch_display = text.to_owned();
                item.branch_error = false;
            }
            5 => item.changes = text.to_owned(),
            _ => return Err(FieldError::new("Cell is read-only")),
        }
        Ok(())
    }

    fn is_editable(&self, _row: usize, col: usize) -> bool {
        matches!(col, 0 | 2 | 4 | 5)
    }
}

fn table() -> Grid<'static> {
    Grid::new(TABLE, &COLUMNS).nav(NavUnit::Cell)
}

/// The grid owns cursor and editor state; the model owns the editable task
/// records and their validation rules.
#[derive(Debug)]
pub(crate) struct EditablePage {
    model: EditableModel,
    state: GridState,
    edits: u32,
}

impl EditablePage {
    pub(crate) fn new() -> Self {
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

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let was_editing = self.state.is_editing();
        let action = table().update_editable(cx, &mut self.state, &mut self.model);
        if was_editing && !self.state.is_editing() {
            self.edits = self.edits.saturating_add(1);
        }
        action.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let blurb = if area.width < 60 {
            "Navigation is reversed cell; editing is …"
        } else if area.width < 80 {
            "Navigation is reversed cell; editing is a cursor. They never…"
        } else {
            "Navigation is reversed cell; editing is a cursor. They never look alike."
        };
        frame(ui, area, self.title(), blurb, |ui, body| {
            let card_height = (self.model.rows.len() as u16 + 4).min(body.height.saturating_sub(4));
            let task_meta = self.state.edit_error().map_or_else(
                || format!("{} edits", self.edits),
                |error| error.to_string(),
            );
            Panel::new(TASKS_PANEL)
                .title("Tasks")
                .meta(&task_meta)
                .patch_part(PANEL_PARTS)
                .draw(
                    ui,
                    Rect {
                        height: card_height,
                        ..body
                    },
                    |ui, inner| {
                        table().draw(ui, inner, &self.state, &self.model);
                        if !self.state.is_editing() && self.edits == 0 {
                            legacy_table(ui, inner, body.width, &self.model);
                        }
                    },
                );
            paint_card_meta(
                ui,
                Rect {
                    height: card_height,
                    ..body
                },
                &task_meta,
            );
            let legend_y = body.y.saturating_add(card_height).saturating_add(1);
            let legend = [
                ("reversed", "cell cursor (navigation)"),
                ("▁", "editing cursor + accent underline"),
                ("!", "validation error"),
            ];
            for (offset, (glyph, text)) in legend.iter().enumerate() {
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
                    ui.surface_style(),
                );
                let _ = ui.paint_str(
                    Rect {
                        x: row.x.saturating_add(10),
                        width: row.width.saturating_sub(10),
                        ..row
                    },
                    text,
                    ui.surface_style(),
                );
            }
            if self.state.is_editing() {
                let _ = ui.paint_str(
                    Rect {
                        y: body.bottom().saturating_sub(1),
                        height: 1,
                        ..body
                    },
                    "EDIT",
                    ui.surface_style(),
                );
            }
        });
    }
}
