use ratatui_core::layout::Rect;
use termrock_grid::{
    CellRef, Column, ColumnKey, Grid, GridAction, GridModel, GridState, Table, TableAction,
    TableColumn, TableSort, TableState,
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
    TableColumn::new(Column {
        sortable: true,
        ..Column::new(ColumnKey::num(2), "Duration")
    })
    .sort(TableSort::NumericDisplay),
];

#[derive(Clone)]
struct Model {
    rows: Vec<(u64, &'static str, &'static str)>,
    revision: Option<u64>,
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

    fn revision(&self) -> Option<u64> {
        self.revision
    }
}

struct TableApp {
    model: Model,
    state: TableState,
    actions: Vec<TableAction>,
    area: Rect,
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
        Table::new(ID, &COLUMNS).draw(ui, self.area, &self.state, &self.model);
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
            revision: None,
        },
        state: TableState::default(),
        actions: Vec::new(),
        area: Rect::new(0, 0, 42, 7),
    }
}

#[test]
fn visible_range_and_position_label_come_from_table_scroll_state() {
    let mut app = app();
    app.model
        .rows
        .extend([(70, "eta", "7"), (80, "theta", "8"), (90, "iota", "9")]);
    app.area = Rect::new(0, 0, 42, 5);
    let mut harness = Harness::new(app, Theme::junie(), 42, 5);
    let _ = harness.tick();

    let table = Table::new(ID, &COLUMNS);
    assert_eq!(harness.app().state.visible_rows(9), 0..4);
    assert_eq!(
        table.position_label(&harness.app().state, 9),
        "unsorted · 1–4 of 9"
    );
}

#[test]
fn same_extent_interior_key_replacement_drops_removed_selection_without_revision() {
    let mut harness = Harness::new(app(), Theme::junie(), 42, 7);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Char(' '));
    let _ = harness.key(KeyCode::Up);
    assert!(
        harness
            .app()
            .state
            .selected_rows()
            .contains(ItemKey::num(20))
    );
    assert_eq!(
        harness.app().state.cursor().map(|(key, _)| key),
        Some(ItemKey::num(10))
    );

    harness.app_mut().model.rows[1].0 = 70;
    let _ = harness.tick();

    assert_eq!(harness.app().model.rows.len(), 6);
    assert_eq!(harness.app().model.rows[0].0, 10);
    assert_eq!(harness.app().model.rows[5].0, 60);
    assert!(
        !harness
            .app()
            .state
            .selected_rows()
            .contains(ItemKey::num(20))
    );
    assert!(
        !harness
            .app()
            .state
            .selected_rows()
            .contains(ItemKey::num(70))
    );
}

#[test]
fn same_extent_interior_key_replacement_drops_removed_selection_with_revision() {
    let mut app = app();
    app.model.revision = Some(1);
    let mut harness = Harness::new(app, Theme::junie(), 42, 7);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Char(' '));
    let _ = harness.key(KeyCode::Up);
    harness.app_mut().model.rows[1].0 = 70;
    harness.app_mut().model.revision = Some(2);
    let _ = harness.tick();

    assert!(
        !harness
            .app()
            .state
            .selected_rows()
            .contains(ItemKey::num(20))
    );
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
    assert_eq!(
        harness.app().actions,
        [
            TableAction::SortChanged(Some((ColumnKey::num(1), termrock_grid::SortDir::Asc))),
            TableAction::SortChanged(Some((ColumnKey::num(1), termrock_grid::SortDir::Desc))),
            TableAction::SortChanged(None),
        ]
    );
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
