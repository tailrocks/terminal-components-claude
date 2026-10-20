use ratatui_core::layout::Rect;
use termrock_grid::{
    CellRef, Column, ColumnKey, Grid, GridAction, GridModel, GridState, Table, TableColumn,
    TableSort, TableState,
};
use termrock_runtime::{App, Cx, Id, ItemKey, KeyCode, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const ID: Id = Id::root("table.ordered-model");
const COLUMNS: [TableColumn<'static>; 2] = [
    TableColumn::new(Column {
        sortable: true,
        ..Column::new(ColumnKey::num(1), "Name")
    }),
    TableColumn {
        column: Column {
            sortable: true,
            ..Column::new(ColumnKey::num(2), "Duration")
        },
        sort: TableSort::NumericDisplay,
    },
];

#[derive(Clone)]
struct Model {
    rows: Vec<(u64, &'static str, &'static str)>,
}

impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        ItemKey::num(self.rows[row].0)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let (_, name, duration) = self.rows.get(row)?;
        match col {
            0 => Some(CellRef::new(name)),
            1 => Some(CellRef::new(duration)),
            _ => None,
        }
    }
}

struct TableApp {
    model: Model,
    state: TableState,
    actions: Vec<GridAction>,
}

impl App for TableApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Table::new(ID, &COLUMNS).update(cx, &mut self.state, &self.model);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Table::new(ID, &COLUMNS).draw(ui, Rect::new(0, 0, 42, 7), &self.state, &self.model);
    }
}

fn app() -> TableApp {
    TableApp {
        model: Model {
            rows: vec![
                (10, "beta", "6m 52s"),
                (20, "Éclair", "10"),
                (30, "e\u{301}clair", "2"),
                (40, "Alpha", "1"),
                (50, "éclair", "3"),
                (60, "zeta", "1"),
            ],
        },
        state: TableState::default(),
        actions: Vec::new(),
    }
}

#[test]
fn keyboard_sort_orders_unicode_text_and_keeps_cursor_selection_keyed() {
    let mut harness = Harness::new(app(), Theme::junie(), 42, 7);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Char(' '));
    assert_eq!(
        harness.app().state.cursor().map(|(key, _)| key),
        Some(ItemKey::num(20))
    );
    let _ = harness.key(KeyCode::Char('s'));
    assert_eq!(
        harness.app().state.sort(),
        Some((ColumnKey::num(1), termrock_grid::SortDir::Asc))
    );
    assert_eq!(
        harness.app().state.cursor().map(|(key, _)| key),
        Some(ItemKey::num(20))
    );
    assert!(
        harness
            .app()
            .state
            .selected_rows()
            .contains(ItemKey::num(20))
    );
    assert!(harness.row(1).contains("Alpha"));
    assert!(harness.row(2).contains("beta"));
}

#[test]
fn sortable_header_cycles_ascending_descending_then_source_order() {
    let mut harness = Harness::new(app(), Theme::junie(), 42, 7);
    let _ = harness.click(4, 0);
    assert_eq!(
        harness.app().state.sort(),
        Some((ColumnKey::num(1), termrock_grid::SortDir::Asc))
    );
    assert!(harness.row(1).contains("Alpha"));
    let _ = harness.click(4, 0);
    assert_eq!(
        harness.app().state.sort(),
        Some((ColumnKey::num(1), termrock_grid::SortDir::Desc))
    );
    assert!(harness.row(1).contains("éclair"));
    let _ = harness.click(4, 0);
    assert_eq!(harness.app().state.sort(), None);
    assert!(harness.row(1).contains("beta"));
    assert!(harness.row(2).contains("Éclair"));
    assert!(harness.row(5).contains("éclair"));
}

#[test]
fn numeric_display_sort_uses_filtered_display_text_and_reverses_ties() {
    let mut harness = Harness::new(app(), Theme::junie(), 42, 7);
    let _ = harness.click(15, 0);
    assert_eq!(
        harness.app().state.sort(),
        Some((ColumnKey::num(2), termrock_grid::SortDir::Asc))
    );
    assert!(harness.row(1).contains("1"));
    assert!(harness.row(1).contains("Alpha"));
    assert!(harness.row(2).contains("zeta"));
    assert!(harness.row(3).contains("2"));
    assert!(harness.row(5).contains("10"));
    assert!(harness.row(6).contains("6m 52s"));
    let _ = harness.click(15, 0);
    assert!(harness.row(5).contains("zeta"));
    assert!(harness.row(6).contains("Alpha"));
}

#[test]
fn table_sort_binding_does_not_change_default_grid_bindings() {
    let harness = Harness::new(app(), Theme::junie(), 42, 7);
    assert!(harness.app().actions.is_empty());

    struct GridApp {
        model: Model,
        state: GridState,
        actions: Vec<GridAction>,
    }
    impl App for GridApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let mut response = Grid::new(
                ID,
                &[
                    Column::new(ColumnKey::num(1), "Name"),
                    Column::new(ColumnKey::num(2), "Duration"),
                ],
            )
            .update(cx, &mut self.state, &self.model);
            if let Some(action) = response.take_action() {
                self.actions.push(action);
            }
            response.erase()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            Grid::new(
                ID,
                &[
                    Column::new(ColumnKey::num(1), "Name"),
                    Column::new(ColumnKey::num(2), "Duration"),
                ],
            )
            .draw(ui, Rect::new(0, 0, 42, 7), &self.state, &self.model);
        }
    }
    let mut grid = Harness::new(
        GridApp {
            model: app().model,
            state: GridState::default(),
            actions: Vec::new(),
        },
        Theme::junie(),
        42,
        7,
    );
    let _ = grid.key(KeyCode::Char('s'));
    assert!(grid.app().actions.is_empty());
}
