use junie_tui::{
    App, CellRef, Column, ColumnKey, Cx, Grid, GridModel, GridState, Id, ItemKey, Rect, Response,
    Ui,
};

const ID: Id = Id::root("grid.model-only-fixture");
const COLUMNS: [Column<'static>; 1] = [Column::new(ColumnKey::num(1), "Value")];

struct Model;

impl GridModel for Model {
    fn row_count(&self) -> usize {
        1
    }

    fn row_key(&self, _row: usize) -> ItemKey {
        ItemKey::num(1)
    }

    fn cell(&self, _row: usize, col: usize) -> Option<CellRef<'_>> {
        (col == 0).then_some(CellRef::new("model only"))
    }
}

pub(super) struct ModelOnlyApp {
    state: GridState,
    model: Model,
}

impl Default for ModelOnlyApp {
    fn default() -> Self {
        ModelOnlyApp {
            state: GridState::default(),
            model: Model,
        }
    }
}

impl App for ModelOnlyApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Grid::new(ID, &COLUMNS)
            .update(cx, &mut self.state, &self.model)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(ID, &COLUMNS).draw(ui, Rect::new(0, 0, 20, 3), &self.state, &self.model);
    }
}
