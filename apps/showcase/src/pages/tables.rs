//! Read-only keyed data grid with model-owned sorting.

use tui_next::{Align, CellRef, Column, ColumnKey, Cx, Grid, GridAction, GridModel, GridState, Id, ItemKey, NavUnit, Part, Rect, Response, Role, RowDecor, RowTotal, SortDir, StylePatch, Ui, id};

use crate::data::{TASKS, TaskRow, TaskStatus};

use super::{Page, frame, lines};

const TABLE: Id = id!("tables.tasks");
const LABEL_PATCH: StylePatch = StylePatch::new().set_fg(Role::Info);
const PART_PATCH: &[(Part, StylePatch)] = &[(Part::HEADER, LABEL_PATCH)];

const COLUMNS: [Column<'static>; 6] = [
    Column { key: ColumnKey::num(0), title: "ID", subtitle: None, align: Align::Right, min_width: 6, max_width: 8, sortable: true, editable: false, sticky: true, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(1), title: "Task", subtitle: None, align: Align::Left, min_width: 12, max_width: 38, sortable: false, editable: false, sticky: false, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(2), title: "Owner", subtitle: None, align: Align::Left, min_width: 7, max_width: 12, sortable: true, editable: false, sticky: false, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(3), title: "Branch", subtitle: None, align: Align::Left, min_width: 12, max_width: 28, sortable: false, editable: false, sticky: false, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(4), title: "Changes", subtitle: None, align: Align::Right, min_width: 8, max_width: 10, sortable: true, editable: false, sticky: false, prefix_glyph: None, badge: None },
    Column { key: ColumnKey::num(5), title: "Duration", subtitle: None, align: Align::Right, min_width: 9, max_width: 12, sortable: true, editable: false, sticky: false, prefix_glyph: None, badge: None },
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
            changes: if task.changes == 0 { "—".to_owned() } else { task.changes.to_string() },
            duration: if task.duration_s == 0 { "—".to_owned() } else { format!("{}s", task.duration_s) },
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
        Self { rows: TASKS.iter().copied().map(TableRow::from).collect() }
    }

    fn sort(&mut self, key: ColumnKey, direction: SortDir) {
        self.rows.sort_by(|left, right| {
            let ordering = match key.raw() {
                0 => left.task.id.cmp(&right.task.id),
                2 => left.task.owner.cmp(right.task.owner),
                4 => left.task.changes.cmp(&right.task.changes),
                5 => left.task.duration_s.cmp(&right.task.duration_s),
                _ => left.task.id.cmp(&right.task.id),
            };
            if direction == SortDir::Desc { ordering.reverse() } else { ordering }
        });
    }
}

impl GridModel for TableModel {
    fn row_count(&self) -> usize { self.rows.len() }

    fn row_key(&self, row: usize) -> ItemKey {
        self.rows.get(row).map(|item| ItemKey::num(u64::from(item.task.id))).unwrap_or(ItemKey::num(0))
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let item = self.rows.get(row)?;
        let text = match col {
            0 => item.id.as_str(),
            1 => item.task.name,
            2 => item.task.owner,
            3 => item.task.branch,
            4 => item.changes.as_str(),
            5 => item.duration.as_str(),
            _ => return None,
        };
        Some(CellRef::new(text))
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let mut decor = RowDecor::default();
        if self.rows.get(row).is_some_and(|item| item.task.status == TaskStatus::Failed) {
            decor.tone = Some(Role::Danger);
        }
        decor
    }

    fn total(&self) -> RowTotal { RowTotal::Exact(self.rows.len()) }
}

fn table() -> Grid<'static> {
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
    last: &'static str,
}

impl TablesPage {
    pub(crate) fn new() -> Self {
        Self { model: TableModel::new(), state: GridState::default(), last: "ID order" }
    }
}

impl Default for TablesPage {
    fn default() -> Self { Self::new() }
}

impl Page for TablesPage {
    fn title(&self) -> &'static str { "Tables" }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let action = table().update(cx, &mut self.state, &self.model);
        if let Some(GridAction::Sort(key, direction)) = action.action_ref() {
            self.model.sort(*key, *direction);
            self.last = match direction { SortDir::Asc => "ascending", SortDir::Desc => "descending" };
        } else if action.action_ref().is_some_and(|value| matches!(value, GridAction::Moved)) {
            self.last = "cursor moved";
        }
        action.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(ui, area, self.title(), "model-owned sort · keyed rows · click headers", |ui, body| {
            let table_area = Rect { height: body.height.saturating_sub(2), ..body };
            table().draw(ui, table_area, &self.state, &self.model);
            let row = self.state.cursor().map(|(key, _)| format!("cursor={key:?}" )).unwrap_or_else(|| "cursor=none".to_owned());
            let summary = format!("{} · {} · {}", self.last, row, self.model.rows.first().map_or("empty", |r| r.id.as_str()));
            let _ = ui.paint_str(Rect { y: table_area.bottom(), height: 1, ..body }, &summary, ui.surface_style());
            lines(ui, Rect { y: table_area.bottom().saturating_add(1), height: 1, ..body }, &["Sortable headers emit a request; this adapter performs the numeric/domain order."]);
        });
    }
}
