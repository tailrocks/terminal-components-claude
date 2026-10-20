use core::cmp::Ordering;

use ratatui_core::layout::Rect;

use crate::grid::{
    CellAction, CellRef, Column, ColumnKey, Grid, GridAction, GridModel, GridState, NavUnit,
    SortDir,
};
use crate::id::{Id, ItemKey};
use crate::response::Response;
use crate::ui::{Cx, Ui};
use termrock_collections::collection::KeySet;
use termrock_runtime::decor::{CellDecor, RowDecor};
use termrock_runtime::empty::RowTotal;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableSort {
    #[default]
    Text,
    NumericDisplay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableColumn<'a> {
    pub column: Column<'a>,
    pub sort: TableSort,
}

impl<'a> TableColumn<'a> {
    pub const fn new(column: Column<'a>) -> Self {
        Self {
            column,
            sort: TableSort::Text,
        }
    }

    #[must_use]
    pub const fn sort(mut self, sort: TableSort) -> Self {
        self.sort = sort;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TableState {
    grid: GridState,
    order: Vec<usize>,
    sort: Option<(ColumnKey, SortDir)>,
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

    fn grid_columns(&self) -> Vec<Column<'a>> {
        self.columns.iter().map(|column| column.column).collect()
    }

    fn grid<'b>(&self, columns: &'b [Column<'a>]) -> Grid<'b> {
        Grid::new(self.id, columns)
            .nav(NavUnit::Row)
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
    ) -> Response<GridAction> {
        let columns = self.grid_columns();
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
            return response;
        }
        response
    }

    pub fn draw<M: GridModel + ?Sized>(
        &self,
        ui: &mut Ui<'_>,
        area: Rect,
        state: &TableState,
        model: &M,
    ) -> Rect {
        let columns = self.grid_columns();
        let grid = self.grid(&columns);
        let order = self.ordered_rows(model, state.sort);
        let ordered = OrderedModel {
            source: model,
            order: &order,
        };
        grid.draw(ui, area, &state.grid, &ordered)
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
