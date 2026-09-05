use junie_tui::{
    CellRef, Column, ColumnKey, Cx, EditIntent, FieldError, Grid, GridEditor, GridModel, GridState,
    ItemKey,
};

struct Model;

impl GridModel for Model {
    fn row_count(&self) -> usize {
        1
    }

    fn row_key(&self, _row: usize) -> ItemKey {
        ItemKey::num(1)
    }

    fn cell(&self, _row: usize, _col: usize) -> Option<CellRef<'_>> {
        Some(CellRef::new("value"))
    }
}

impl GridEditor for Model {
    fn edit_intent(&self, _row: usize, _col: usize) -> EditIntent<'_> {
        EditIntent::Cycle
    }

    fn apply_cycle(&mut self, _row: usize, _col: usize) {}

    fn commit_cell(&mut self, _row: usize, _col: usize, _text: &str) -> Result<(), FieldError> {
        Ok(())
    }

    fn is_editable(&self, _row: usize, _col: usize) -> bool {
        true
    }
}

fn cannot_select_editing_through_shared_model(
    grid: &Grid<'_>,
    cx: &mut Cx<'_>,
    state: &mut GridState,
    model: &Model,
) {
    let _ = grid.update_editable(cx, state, model);
}

fn main() {
    let columns = [Column::new(ColumnKey::num(1), "value")];
    let _grid = Grid::new(junie_tui::Id::root("grid"), &columns);
}
