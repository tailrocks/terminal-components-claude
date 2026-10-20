use core::cmp::Ordering;
use core::ops::Range;

use ratatui_core::layout::Rect;

use crate::grid::{
    CellAction, CellRef, Column, ColumnKey, Grid, GridAction, GridGutter, GridModel, GridState,
    NavUnit, SortDir,
};
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::response::Response;
use crate::ui::{Cx, FrameRead, Ui};
use termrock_collections::collection::KeySet;
use termrock_runtime::decor::{CellDecor, RowDecor};
use termrock_runtime::empty::RowTotal;
use termrock_theme::{Family, GlyphRole, Variant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableSort {
    #[default]
    Text,
    NumericDisplay,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TableAction {
    Grid(GridAction),
    SortChanged(Option<(ColumnKey, SortDir)>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableColumn<'a> {
    pub column: Column<'a>,
    pub sort: TableSort,
    pub flexible: bool,
}

impl<'a> TableColumn<'a> {
    pub const fn new(column: Column<'a>) -> Self {
        Self {
            column,
            sort: TableSort::Text,
            flexible: false,
        }
    }

    #[must_use]
    pub const fn sort(mut self, sort: TableSort) -> Self {
        self.sort = sort;
        self
    }

    #[must_use]
    pub const fn flexible(mut self) -> Self {
        self.flexible = true;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableState {
    grid: GridState,
    order: Vec<usize>,
    sort: Option<(ColumnKey, SortDir)>,
    model_keys: Vec<ItemKey>,
    model_revision: Option<u64>,
    model_revision_seen: bool,
}

impl TableState {
    pub const fn cursor(&self) -> Option<(ItemKey, ColumnKey)> {
        self.grid.cursor()
    }

    pub const fn selected_rows(&self) -> &KeySet {
        self.grid.selected_rows()
    }

    pub const fn sort(&self) -> Option<(ColumnKey, SortDir)> {
        self.sort
    }

    pub const fn grid_state(&self) -> &GridState {
        &self.grid
    }

    pub fn visible_rows(&self, total: usize) -> Range<usize> {
        self.grid.visible_rows(total)
    }

    fn sync_model<M: GridModel + ?Sized>(&mut self, model: &M) {
        let revision = model.revision();
        let changed = match revision {
            Some(revision) if self.model_revision_seen => self.model_revision != Some(revision),
            _ => {
                self.model_keys.len() != model.row_count()
                    || self
                        .model_keys
                        .iter()
                        .enumerate()
                        .any(|(index, key)| model.row_key(index) != *key)
            }
        };
        if changed {
            self.grid.invalidate_model();
        }
        if changed || !self.model_revision_seen {
            self.model_keys.clear();
            self.model_keys
                .extend((0..model.row_count()).map(|index| model.row_key(index)));
        }
        self.model_revision = revision;
        self.model_revision_seen = true;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    id: Id,
    columns: &'a [TableColumn<'a>],
}

impl<'a> Table<'a> {
    pub const fn new(id: Id, columns: &'a [TableColumn<'a>]) -> Self {
        Self { id, columns }
    }

    pub fn position_label(&self, state: &TableState, total: usize) -> String {
        let sort = match state.sort {
            Some((key, SortDir::Asc)) => self
                .columns
                .iter()
                .find(|column| column.column.key == key)
                .map_or_else(
                    || "sorted ▴".to_owned(),
                    |column| format!("sorted by {} ▴", column.column.title.to_lowercase()),
                ),
            Some((key, SortDir::Desc)) => self
                .columns
                .iter()
                .find(|column| column.column.key == key)
                .map_or_else(
                    || "sorted ▾".to_owned(),
                    |column| format!("sorted by {} ▾", column.column.title.to_lowercase()),
                ),
            None => "unsorted".to_owned(),
        };
        let range = state.visible_rows(total);
        if !state.grid.rows_overflow() {
            return sort;
        }
        format!("{sort} · {}–{} of {total}", range.start + 1, range.end)
    }

    fn grid_columns<M: GridModel + ?Sized>(
        &self,
        area: Rect,
        state: &TableState,
        model: &M,
    ) -> (Vec<Column<'a>>, bool, bool, usize) {
        let mut horizontal = false;
        let mut vertical = false;
        for _ in 0..3 {
            let body_rows = usize::from(area.height.saturating_sub(1 + u16::from(horizontal)));
            vertical = model.row_count() > body_rows;
            let viewport = area.width.saturating_sub(5 + u16::from(vertical));
            let offset = state
                .grid
                .col_offset()
                .min(self.columns.len().saturating_sub(1));
            let mut widths = self
                .columns
                .iter()
                .enumerate()
                .map(|(column_index, column)| {
                    if column.flexible {
                        column
                            .column
                            .min_width
                            .max(crate::text::width(column.column.title))
                    } else {
                        let mut width = column
                            .column
                            .min_width
                            .max(crate::text::width(column.column.title));
                        for row in 0..model.row_count() {
                            if let Some(cell) = model.cell(row, column_index) {
                                width = width.max(crate::text::width(cell.text));
                            }
                        }
                        width.min(column.column.max_width.max(column.column.min_width))
                    }
                })
                .collect::<Vec<_>>();
            let mut used = 0u16;
            let mut visible = 0usize;
            for width in widths.iter().skip(offset) {
                let next = used
                    .saturating_add(*width)
                    .saturating_add(u16::from(visible > 0) * 2);
                if next > viewport && visible > 0 {
                    break;
                }
                used = next;
                visible = visible.saturating_add(1);
            }
            visible = visible.max(usize::from(offset < widths.len()));
            let end = offset.saturating_add(visible).min(widths.len());
            if let Some(flex) = (offset..end).find(|&index| self.columns[index].flexible) {
                let fixed = widths
                    .iter()
                    .skip(offset)
                    .take(visible)
                    .fold(0u16, |sum, width| sum.saturating_add(*width));
                let gaps = visible.saturating_sub(1).saturating_mul(2) as u16;
                let available = viewport.saturating_sub(gaps).saturating_sub(fixed);
                if let Some(width) = widths.get_mut(flex) {
                    *width = width.saturating_add(available);
                }
            }
            let hidden = end < self.columns.len();
            let result = self
                .columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    let width = widths
                        .get(index)
                        .copied()
                        .unwrap_or(column.column.min_width);
                    Column {
                        min_width: width,
                        max_width: width,
                        ..column.column
                    }
                })
                .collect();
            if horizontal == hidden {
                return (result, horizontal, vertical, visible);
            }
            horizontal = hidden;
        }
        (Vec::new(), horizontal, vertical, 0)
    }

    fn grid<'b>(&self, columns: &'b [Column<'a>]) -> Grid<'b> {
        Grid::new(self.id, columns)
            .nav(NavUnit::Row)
            .gutter(GridGutter::Detailed {
                row_numbers: false,
                min_digits: 0,
            })
            .right_reserve(2)
            .column_gap(2)
            .row_nav_scrolls_columns(true)
            .sort_current_column(true)
    }

    fn ordered_rows<M: GridModel + ?Sized>(
        &self,
        model: &M,
        sort: Option<(ColumnKey, SortDir)>,
    ) -> Vec<usize> {
        let mut order = (0..model.row_count()).collect::<Vec<_>>();
        let Some((key, direction)) = sort else {
            return order;
        };
        let Some(column_index) = self
            .columns
            .iter()
            .position(|column| column.column.key == key && column.column.sortable)
        else {
            return order;
        };
        let sort_kind = self.columns[column_index].sort;
        order.sort_by(|&a, &b| {
            let left = model.cell(a, column_index).map_or("", |cell| cell.text);
            let right = model.cell(b, column_index).map_or("", |cell| cell.text);
            let compared = match sort_kind {
                TableSort::Text => left.to_lowercase().cmp(&right.to_lowercase()),
                TableSort::NumericDisplay => parse_display_number(left)
                    .partial_cmp(&parse_display_number(right))
                    .unwrap_or(Ordering::Equal),
            }
            .then_with(|| a.cmp(&b));
            match direction {
                SortDir::Asc => compared,
                SortDir::Desc => compared.reverse(),
            }
        });
        order
    }

    pub fn update<M: GridModel + ?Sized>(
        &self,
        cx: &mut Cx<'_>,
        state: &mut TableState,
        model: &M,
    ) -> Response<TableAction> {
        state.sync_model(model);
        let area = cx.area(self.id).unwrap_or_default();
        let (columns, _horizontal, _vertical, _) = self.grid_columns(area, state, model);
        let grid = self.grid(&columns);
        state.order = self.ordered_rows(model, state.sort);
        let response = {
            let ordered = OrderedModel {
                source: model,
                order: &state.order,
            };
            grid.update(cx, &mut state.grid, &ordered)
        };

        if let Some(GridAction::Sort(key, _)) = response.action_ref() {
            let key = *key;
            state.sort = match state.sort {
                Some((current, SortDir::Asc)) if current == key => Some((key, SortDir::Desc)),
                Some((current, SortDir::Desc)) if current == key => None,
                _ => Some((key, SortDir::Asc)),
            };
            state.grid.set_sort(state.sort);
            state.order = self.ordered_rows(model, state.sort);
            if let Some((row, column)) = state.grid.cursor() {
                let ordered = OrderedModel {
                    source: model,
                    order: &state.order,
                };
                let _ = grid.move_cursor_to(&mut state.grid, &ordered, row, column);
            }
            let cleared = state.sort.is_none();
            return response.map_action(|action| match action {
                GridAction::Sort(_, _) if cleared => TableAction::SortChanged(None),
                GridAction::Sort(_, _) => TableAction::SortChanged(state.sort),
                other => TableAction::Grid(other),
            });
        }
        response.map_action(TableAction::Grid)
    }

    pub fn draw<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &TableState,
        model: &M,
    ) -> Rect {
        let (columns, horizontal, vertical, visible) = self.grid_columns(area, state, model);
        let grid = self.grid(&columns);
        let order = self.ordered_rows(model, state.sort);
        let ordered = OrderedModel {
            source: model,
            order: &order,
        };
        let grid_area = if horizontal {
            Rect {
                height: area.height.saturating_sub(1),
                ..area
            }
        } else {
            area
        };
        let drawn = grid.draw(ui, grid_area, &state.grid, &ordered);
        if horizontal {
            self.draw_horizontal_bar(ui, area, state, visible, vertical);
        }
        drawn
    }

    fn draw_horizontal_bar(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &TableState,
        visible: usize,
        vertical: bool,
    ) {
        if area.is_empty() || area.height == 0 {
            return;
        }
        let track_area = Rect {
            x: area.x.saturating_add(3),
            y: area.bottom().saturating_sub(1),
            width: area.width.saturating_sub(5 + u16::from(vertical)),
            height: 1,
        };
        let track = ui.style(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::TRACK,
            crate::response::StateFlags::empty(),
        );
        let thumb = ui.style(
            Family::SCROLLBAR,
            Variant::DEFAULT,
            Part::THUMB,
            crate::response::StateFlags::empty(),
        );
        ui.glyph(track_area, GlyphRole::ScrollTrack, track.style);
        let total = self.columns.len();
        let thumb_width = usize::from(track_area.width)
            .saturating_mul(visible.max(1))
            .checked_div(total.max(1))
            .unwrap_or(0)
            .max(1)
            .min(usize::from(track_area.width));
        let travel = usize::from(track_area.width).saturating_sub(thumb_width);
        let max_offset = total.saturating_sub(visible.max(1));
        let left = state
            .grid
            .col_offset()
            .saturating_mul(travel)
            .checked_div(max_offset.max(1))
            .unwrap_or(0);
        let thumb_area = Rect {
            x: track_area
                .x
                .saturating_add(left.min(usize::from(u16::MAX)) as u16),
            width: thumb_width.min(usize::from(u16::MAX)) as u16,
            ..track_area
        };
        ui.glyph(thumb_area, GlyphRole::ScrollThumb, thumb.style);
        ui.register_part(self.id, PartRef::of(Part::TRACK), track_area);
        ui.register_part(self.id, PartRef::of(Part::THUMB), thumb_area);
    }
}

struct OrderedModel<'a, M: ?Sized> {
    source: &'a M,
    order: &'a [usize],
}

impl<M: GridModel + ?Sized> GridModel for OrderedModel<'_, M> {
    fn row_count(&self) -> usize {
        self.order.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.source.row_key(self.order[row])
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.source.cell(self.order[row], col)
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        self.source.row_decor(self.order[row])
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        self.source.cell_decor(self.order[row], col)
    }

    fn total(&self) -> RowTotal {
        self.source.total()
    }

    fn has_more(&self) -> bool {
        self.source.has_more()
    }

    fn read_only_reason(&self) -> Option<&str> {
        self.source.read_only_reason()
    }

    fn actions(&self, row: usize, col: usize) -> &[CellAction] {
        self.source.actions(self.order[row], col)
    }
}

fn parse_display_number(value: &str) -> Option<f64> {
    let cleaned = value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == '.' || *character == '-')
        .collect::<String>();
    cleaned
        .parse::<f64>()
        .ok()
        .filter(|number| !number.is_nan())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Rows;

    impl GridModel for Rows {
        fn row_count(&self) -> usize {
            24
        }

        fn row_key(&self, row: usize) -> ItemKey {
            ItemKey::num(row as u64)
        }

        fn cell(&self, row: usize, column: usize) -> Option<CellRef<'_>> {
            let _ = row;
            Some(CellRef::new(match column {
                0 => "#1234",
                1 => "task title",
                2 => "Owner",
                3 => "Running",
                4 => "branch/termrock-refactor",
                5 => "12345",
                6 => "6m 52s",
                _ => return None,
            }))
        }
    }

    fn columns() -> [TableColumn<'static>; 7] {
        [
            Column::new(ColumnKey::num(1), "ID"),
            Column::new(ColumnKey::num(2), "Task"),
            Column::new(ColumnKey::num(3), "Owner"),
            Column::new(ColumnKey::num(4), "Status"),
            Column::new(ColumnKey::num(5), "Branch"),
            Column::new(ColumnKey::num(6), "Changes"),
            Column::new(ColumnKey::num(7), "Duration"),
        ]
        .map(TableColumn::new)
        .map(|mut column| {
            column.column.min_width = match column.column.key.raw() {
                1 => 5,
                2 => 24,
                3 => 7,
                4 => 9,
                5 => 20,
                _ => 9,
            };
            column.column.max_width = column.column.min_width;
            if column.column.key == ColumnKey::num(2) {
                column.flexible = true;
            }
            column
        })
    }

    #[test]
    fn frozen_baseline_widths_and_scrollbar_decision_match_both_reference_sizes() {
        let columns = columns();
        let table = Table::new(Id::root("table.geometry"), &columns);
        let model = Rows;
        let state = TableState::default();

        let (narrow, horizontal, vertical, visible) =
            table.grid_columns(Rect::new(0, 0, 88, 22), &state, &model);
        assert_eq!(
            narrow
                .iter()
                .map(|column| column.min_width)
                .collect::<Vec<_>>(),
            [5, 33, 7, 9, 20, 9, 9]
        );
        assert!(horizontal);
        assert!(vertical);
        assert_eq!(visible, 5);

        let (wide, horizontal, vertical, visible) =
            table.grid_columns(Rect::new(0, 0, 130, 22), &state, &model);
        assert_eq!(
            wide.iter()
                .map(|column| column.min_width)
                .collect::<Vec<_>>(),
            [5, 53, 7, 9, 20, 9, 9]
        );
        assert!(!horizontal);
        assert!(vertical);
        assert_eq!(visible, 7);
    }
}
