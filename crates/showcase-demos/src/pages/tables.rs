//! Read-only keyed data grid with model-owned sorting.

use termrock::{
    Align, CellRef, Column, ColumnKey, Cx, EmptyState, FgStep, Grid, GridAction, GridModel,
    GridState, Id, ItemKey, NavUnit, Panel, PanelKind, Part, Rect, Role, RowDecor, RowTotal,
    SortDir, StylePatch, Track, Ui, id, layout,
};

use showcase_data::{TASKS, TaskRow, TaskStatus};

use super::{Page, PageUpdate, frame};

const TABLE: Id = id!("tables.tasks");
const CHECKS: Id = id!("tables.checks");
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(termrock::Modifier::BOLD),
)];

const COLUMNS: [Column<'static>; 7] = [
    Column {
        key: ColumnKey::num(0),
        title: "ID",
        subtitle: None,
        align: Align::Left,
        min_width: 5,
        max_width: 5,
        flex: 0,
        sortable: true,
        editable: false,
        sticky: true,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(1),
        title: "Task",
        subtitle: None,
        align: Align::Left,
        min_width: 24,
        max_width: 24,
        flex: 1,
        sortable: false,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(2),
        title: "Owner",
        subtitle: None,
        align: Align::Left,
        min_width: 7,
        max_width: 7,
        flex: 0,
        sortable: true,
        editable: false,
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
        flex: 0,
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
        min_width: 20,
        max_width: 20,
        flex: 0,
        sortable: false,
        editable: false,
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
        min_width: 9,
        max_width: 9,
        flex: 0,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(6),
        title: "Duration",
        subtitle: None,
        align: Align::Right,
        min_width: 9,
        max_width: 9,
        flex: 0,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
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
        .column_gap(2)
        .left_reserve(1)
        .right_reserve(2)
        .sort_indicator(termrock::GridSortIndicator::ActiveOnly)
        .overflow_indicator(termrock::GridOverflowIndicator::Ellipsis)
}

/// `1–21 of 24`, or empty when every row fits (tag:scrollbar.rs
/// `position_label`). The viewport comes from draw-time panel geometry —
/// last-frame layout is unavailable on frame 1, which is what the
/// no-input default capture presents — following the trees page precedent.
fn position_label(state: &GridState, viewport: usize, len: usize) -> String {
    if len <= viewport || viewport == 0 {
        return String::new();
    }
    let mut scroll = *state.scroll();
    scroll.apply_layout(viewport, len);
    let r = scroll.visible_range();
    format!("{}–{} of {len}", r.start + 1, r.end)
}

/// The one tasks-card constructor (§13): the meta gate reads its geometry
/// and draw paints through it, so the two can never disagree.
fn tasks_panel(meta: &str) -> Panel<'_> {
    Panel::new(id!("tables.tasks_panel"))
        .kind(PanelKind::Card)
        .title("Tasks")
        .meta(meta)
        .patch_part(PANEL_PARTS)
}

/// The one checks-card constructor (§13), reached from update and from draw.
fn checks_panel() -> Panel<'static> {
    Panel::new(CHECKS)
        .kind(PanelKind::Card)
        .title("Checks")
        .patch_part(PANEL_PARTS)
}

/// The grid owns only cursor state; the adapter owns row order and domain
/// comparison, preserving keyed identity through every sort request.
#[derive(Debug)]
pub struct TablesPage {
    model: TableModel,
    state: GridState,
    last: String,
    sort: Option<(ColumnKey, SortDir)>,
}

impl TablesPage {
    pub fn new() -> Self {
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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
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
        let _ = checks_panel();
        action.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let meta = "Sort by header, hover rows, select with Enter, overflow scrolls";
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
            // −1: the grid header row (`Grid::chrome`; no note row on a
            // read-only model, no actions bar on this table).
            let viewport = tasks_panel("").inner(ui, tasks).height.saturating_sub(1) as usize;
            let pos = position_label(&self.state, viewport, self.model.row_count());
            let task_meta = if pos.is_empty() {
                self.last.clone()
            } else {
                format!("{} · {pos}", self.last)
            };
            tasks_panel(&task_meta).draw(ui, tasks, |ui, inner| {
                self.draw_tasks(ui, inner);
            });
            if let Some(checks) = regions.get(2).copied() {
                checks_panel().draw(ui, checks, |ui, inner| {
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

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.state.is_editing() {
            &[("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")]
        } else {
            &[
                ("↑ ↓", "Move"),
                ("← →", "Columns"),
                ("s", "Sort column"),
                ("Enter", "Select"),
            ]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }
}

impl TablesPage {
    fn draw_tasks(&self, ui: &mut Ui<'_>, inner: Rect) {
        let grid_area = Rect {
            x: inner.x,
            width: inner.width,
            ..inner
        };
        table().draw(ui, grid_area, &self.state, &self.model);
    }
}
