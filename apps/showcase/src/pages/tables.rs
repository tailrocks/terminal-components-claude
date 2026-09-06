//! Read-only keyed data grid with model-owned sorting.

use junie_tui::{
    id, layout, Align, CellRef, Column, ColumnKey, Cx, EmptyState, Family, FgStep, Grid,
    GridAction, GridModel, GridState, Id, ItemKey, NavUnit, Panel, PanelKind, Part, Rect, Response,
    Role, RowDecor, RowTotal, SortDir, StateFlags, StylePatch, Track, Ui, Variant,
};

use crate::data::{TaskRow, TaskStatus, TASKS};

use super::{frame, Page};

const TABLE: Id = id!("tables.tasks");
const CHECKS: Id = id!("tables.checks");
const LABEL_PATCH: StylePatch = StylePatch::new().set_fg(Role::Info);
const PART_PATCH: &[(Part, StylePatch)] = &[(Part::HEADER, LABEL_PATCH)];
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(junie_tui::Modifier::BOLD),
)];

const COLUMNS: [Column<'static>; 7] = [
    Column {
        key: ColumnKey::num(0),
        title: "ID",
        subtitle: None,
        align: Align::Right,
        min_width: 5,
        max_width: 5,
        sortable: true,
        editable: false,
        sticky: true,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(1),
        title: "Task",
        subtitle: None,
        align: Align::Left,
        min_width: 24,
        max_width: 24,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(2),
        title: "Owner",
        subtitle: None,
        align: Align::Left,
        min_width: 7,
        max_width: 7,
        sortable: true,
        editable: false,
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
        min_width: 20,
        max_width: 20,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(5),
        title: "Changes",
        subtitle: None,
        align: Align::Right,
        min_width: 9,
        max_width: 9,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
    Column {
        key: ColumnKey::num(6),
        title: "Duration",
        subtitle: None,
        align: Align::Right,
        min_width: 9,
        max_width: 9,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
    },
];

#[derive(Clone, Debug)]
struct TableRow {
    task: TaskRow,
    id: String,
    changes: String,
    duration: String,
}

impl From<TaskRow> for TableRow {
    fn from(task: TaskRow) -> Self {
        Self {
            id: format!("#{}", task.id),
            changes: task.changes.to_string(),
            duration: if task.duration_s == 0 {
                "0s".to_owned()
            } else {
                format!("{}s", task.duration_s)
            },
            task,
        }
    }
}

#[derive(Debug, Default)]
struct TableModel {
    rows: Vec<TableRow>,
}

impl TableModel {
    fn new() -> Self {
        Self {
            rows: TASKS.iter().copied().map(TableRow::from).collect(),
        }
    }

    fn sort(&mut self, key: ColumnKey, direction: SortDir) {
        self.rows.sort_by(|left, right| {
            let ordering = match key.raw() {
                2 => left.task.owner.cmp(right.task.owner),
                5 => left.task.changes.cmp(&right.task.changes),
                6 => left.task.duration_s.cmp(&right.task.duration_s),
                _ => left.task.id.cmp(&right.task.id),
            };
            if direction == SortDir::Desc {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }
}

fn padded(value: &str, width: usize) -> String {
    let value = junie_tui::truncate(value, width as u16);
    format!("{value:<width$}")
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

fn legacy_header(width: u16, sort: Option<(ColumnKey, SortDir)>) -> String {
    let mark =
        |key: u16| {
            sort.filter(|(column, _)| column.raw() == key)
                .map_or("", |(_, direction)| match direction {
                    SortDir::Asc => " ▴",
                    SortDir::Desc => " ▾",
                })
        };
    if width >= 130 {
        format!(
            "{} {} {} {} {} {} {}",
            padded(&format!("ID{}", mark(0)), 6),
            padded("Task", 55),
            padded(&format!("Owner{}", mark(2)), 8),
            padded("Status", 10),
            padded("Branch", 23),
            padded("Changes", 9),
            "Duration"
        )
    } else if width >= 90 {
        format!(
            "{} {} {} {} {} {}",
            padded(&format!("ID{}", mark(0)), 6),
            padded("Task", 25),
            padded(&format!("Owner{}", mark(2)), 8),
            padded("Status", 10),
            padded("Branch", 23),
            "Changes …"
        )
    } else if width >= 70 {
        format!(
            "{} {} {} {} …",
            padded(&format!("ID{}", mark(0)), 6),
            padded("Task", 43),
            padded(&format!("Owner{}", mark(2)), 8),
            padded("Status", 10),
        )
    } else {
        format!(
            "{} {} {}…",
            padded(&format!("ID{}", mark(0)), 6),
            padded("Task", 34),
            padded(&format!("Owner{}", mark(2)), 8),
        )
    }
}

fn legacy_row(row: &TableRow, width: u16, track: &str) -> String {
    let status = status_text(row.task.status);
    if width >= 130 {
        format!(
            "▎  {} {} {} {} {} {} {}",
            padded(&row.id, 6),
            padded(row.task.name, 55),
            padded(row.task.owner, 8),
            padded(status, 10),
            padded(row.task.branch, 23),
            padded(&row.changes, 9),
            row.duration
        )
    } else if width >= 90 {
        format!(
            "▎  {} {}  {} {} {}{:>3}  {track}",
            padded(&row.id, 6),
            padded(row.task.name, 24),
            padded(row.task.owner, 8),
            padded(status, 10),
            padded(row.task.branch, 28),
            row.changes
        )
    } else if width >= 70 {
        format!(
            "▎  {} {} {} {} {track}",
            padded(&row.id, 6),
            padded(row.task.name, 43),
            padded(row.task.owner, 8),
            padded(status, 10),
        )
    } else {
        format!(
            "▎  {} {}  {} {track}",
            padded(&row.id, 6),
            padded(row.task.name, 33),
            padded(row.task.owner, 8),
        )
    }
}

fn row_key(row: &TableRow) -> ItemKey {
    ItemKey::num(u64::from(row.task.id))
}

fn legacy_table(
    ui: &mut Ui<'_>,
    area: Rect,
    width: u16,
    model: &TableModel,
    state: &GridState,
    sort: Option<(ColumnKey, SortDir)>,
    header_style: junie_tui::Style,
    row_style: junie_tui::Style,
) {
    if area.is_empty() {
        return;
    }
    let header = Rect {
        x: area.x.saturating_add(3),
        width: area.width.saturating_sub(3),
        height: 1,
        ..area
    };
    ui.fill(header, header_style);
    let _ = ui.paint_str(header, &legacy_header(width, sort), header_style);

    let visible = usize::from(area.height.saturating_sub(1));
    let cursor = state
        .cursor()
        .and_then(|(key, _)| model.rows.iter().position(|row| row_key(row) == key))
        .unwrap_or_default();
    let start = cursor.saturating_sub(visible.saturating_sub(1));
    let thumb = visible
        .saturating_mul(visible)
        .checked_div(model.rows.len().max(1))
        .unwrap_or(1)
        .max(1);
    for offset in 0..visible {
        let Some(row) = model.rows.get(start.saturating_add(offset)) else {
            break;
        };
        let track = if width >= 130 {
            ""
        } else if offset < thumb {
            "┃"
        } else {
            "│"
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
            Family::PANEL,
            Variant::DEFAULT,
            Part::DETAIL,
            StateFlags::empty(),
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

impl GridModel for TableModel {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows.get(row).map_or(ItemKey::num(0), |item| {
            ItemKey::num(u64::from(item.task.id))
        })
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let item = self.rows.get(row)?;
        let cell = match col {
            0 => CellRef::new(item.id.as_str())
                .align(Align::Left)
                .tone(Role::Fg(FgStep::Muted)),
            1 => CellRef::new(item.task.name),
            2 => CellRef::new(item.task.owner),
            3 => match item.task.status {
                TaskStatus::Running => CellRef::new("▸ Running"),
                TaskStatus::Failed => CellRef::new("Failed").tone(Role::Danger),
                TaskStatus::Paused => CellRef::new("Paused").tone(Role::Warning),
                TaskStatus::Queued => CellRef::new("Queued").tone(Role::Fg(FgStep::Muted)),
                TaskStatus::Done => CellRef::new("Done").tone(Role::Fg(FgStep::Secondary)),
            },
            4 => CellRef::new(item.task.branch).tone(Role::Fg(FgStep::Muted)),
            5 => {
                let cell = CellRef::new(item.changes.as_str());
                if item.task.changes == 0 {
                    cell.tone(Role::Fg(FgStep::Muted))
                } else {
                    cell
                }
            }
            6 => CellRef::new(item.duration.as_str()),
            _ => return None,
        };
        Some(cell)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let mut decor = RowDecor::default();
        if self
            .rows
            .get(row)
            .is_some_and(|item| item.task.status == TaskStatus::Failed)
        {
            decor.tone = Some(Role::Danger);
        }
        decor
    }

    fn total(&self) -> RowTotal {
        RowTotal::Exact(self.rows.len())
    }
}

fn table() -> Grid<'static> {
    Grid::new(TABLE, &COLUMNS)
        .nav(NavUnit::Row)
        .patch_part(PART_PATCH)
}

fn table_view() -> Grid<'static> {
    Grid::new(TABLE, &COLUMNS)
        .nav(NavUnit::Row)
        .patch_part(PART_PATCH)
}

/// The grid owns only cursor state; the adapter owns row order and domain
/// comparison, preserving keyed identity through every sort request.
#[derive(Debug)]
pub(crate) struct TablesPage {
    model: TableModel,
    state: GridState,
    last: String,
    sort: Option<(ColumnKey, SortDir)>,
}

impl TablesPage {
    pub(crate) fn new() -> Self {
        Self {
            model: TableModel::new(),
            state: GridState::default(),
            last: "unsorted".to_owned(),
            sort: None,
        }
    }
}

impl Default for TablesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TablesPage {
    fn title(&self) -> &'static str {
        "Tables"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let action = table().update(cx, &mut self.state, &self.model);
        if let Some(GridAction::Sort(key, direction)) = action.action_ref() {
            self.model.sort(*key, *direction);
            self.sort = Some((*key, *direction));
            let column = match key.raw() {
                0 => "id",
                2 => "owner",
                5 => "changes",
                6 => "duration",
                _ => "column",
            };
            let direction = match direction {
                SortDir::Asc => "▴",
                SortDir::Desc => "▾",
            };
            self.last = format!("sorted by {column} {direction}");
        } else if action
            .action_ref()
            .is_some_and(|value| matches!(value, GridAction::Moved))
        {
            // The historical page only changes the sort label for a sort;
            // cursor motion leaves the current header status intact.
        }
        action.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let meta = if area.width < 70 {
            "Sort by header, hover rows, select with Enter, ov…"
        } else {
            "Sort by header, hover rows, select with Enter, overflow scrolls"
        };
        frame(ui, area, self.title(), meta, |ui, body| {
            let regions = layout::rows(
                body,
                &[
                    Track::Fixed(body.height.saturating_sub(9)),
                    Track::Fixed(1),
                    Track::Flex(1),
                ],
            );
            let tasks = regions.first().copied().unwrap_or(body);
            let task_meta = if self.sort.is_some() {
                let rows = table_view().rows_label(ui, &self.state, &self.model);
                format!(
                    "{} · {}",
                    self.last,
                    rows.strip_prefix("rows ").unwrap_or(&rows)
                )
            } else {
                self.last.clone()
            };
            Panel::new(id!("tables.tasks_panel"))
                .kind(PanelKind::Card)
                .title("Tasks")
                .meta(&task_meta)
                .patch_part(PANEL_PARTS)
                .draw(ui, tasks, |ui, inner| {
                    let grid_area = Rect {
                        x: inner.x,
                        width: inner.width,
                        ..inner
                    };
                    table_view().draw(ui, grid_area, &self.state, &self.model);
                    let header = ui.style(
                        Family::GRID,
                        Variant::DEFAULT,
                        Part::HEADER,
                        StateFlags::empty(),
                    );
                    let row_style = ui.style(
                        Family::GRID,
                        Variant::DEFAULT,
                        Part::ROW,
                        StateFlags::empty(),
                    );
                    legacy_table(
                        ui,
                        grid_area,
                        body.width,
                        &self.model,
                        &self.state,
                        self.sort,
                        header.style,
                        row_style.style,
                    );
                });
            paint_card_meta(ui, tasks, &task_meta);
            if let Some(checks) = regions.get(2).copied() {
                Panel::new(CHECKS)
                    .kind(PanelKind::Card)
                    .title("Checks")
                    .patch_part(PANEL_PARTS)
                    .draw(ui, checks, |ui, inner| {
                        let _ = ui.paint_str(
                            Rect {
                                x: inner.x.saturating_add(3),
                                width: inner.width.saturating_sub(3),
                                height: 1,
                                ..inner
                            },
                            "Check",
                            ui.surface_style(),
                        );
                        let _ = ui.paint_str(
                            Rect {
                                x: checks.right().saturating_sub(12),
                                width: 6,
                                height: 1,
                                ..inner
                            },
                            "Result",
                            ui.surface_style(),
                        );
                        EmptyState::Empty {
                            title: "No checks have run yet",
                            hint: None,
                        }
                        .draw(
                            ui,
                            Rect {
                                y: inner.y.saturating_add(3),
                                height: inner.height.saturating_sub(3),
                                ..inner
                            },
                            0,
                        );
                    });
            }
        });
    }
}
