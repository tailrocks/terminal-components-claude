//! Product-owned row changes address Grid through stable keys, never indices.
use junie_tui::{
    App, BlurPolicy, Button, CellRef, Column, ColumnKey, Cx, EditIntent, FieldError, Grid,
    GridEditor, GridModel, GridState, Id, ItemKey, KeyCode, Rect, Response, Theme, Ui,
};
use junie_tui_testing::Harness;

const ID: Id = Id::root("keyed.grid");
const OTHER: Id = Id::root("keyed.other");

#[derive(Clone, Debug, PartialEq, Eq)]
struct Model {
    rows: Vec<(u64, Vec<String>)>,
    commits: usize,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::index(row), |r| ItemKey::num(r.0))
    }
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.rows.get(row)?.1.get(col).map(|s| CellRef::new(s))
    }
}
impl GridEditor for Model {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_> {
        EditIntent::Inline {
            initial: self.cell(row, col).map_or("", |c| c.text),
        }
    }
    fn apply_cycle(&mut self, _: usize, _: usize) {}
    fn is_editable(&self, _: usize, _: usize) -> bool {
        true
    }
    fn commit_cell(&mut self, _: usize, _: usize, _: &str) -> Result<(), FieldError> {
        self.commits = self.commits.saturating_add(1);
        Err(FieldError::new("invalid cell"))
    }
}
struct Page {
    model: Model,
    columns: Vec<Column<'static>>,
    state: GridState,
}
impl Page {
    fn new() -> Self {
        Self {
            model: Model {
                rows: (0..40)
                    .map(|row| (row, (0..8).map(|col| format!("r{row}c{col}")).collect()))
                    .collect(),
                commits: 0,
            },
            columns: (0..8)
                .map(|i| {
                    let mut c = Column::new(ColumnKey::num(i), "value");
                    c.min_width = 8;
                    c.editable = true;
                    c
                })
                .collect(),
            state: GridState::default(),
        }
    }
    fn move_to(&mut self, row: u64, col: u16) -> Result<(), String> {
        Grid::new(ID, &self.columns)
            .move_cursor_to(
                &mut self.state,
                &self.model,
                ItemKey::num(row),
                ColumnKey::num(col),
            )
            .map_err(|e| e.to_string())
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = Grid::new(ID, &self.columns)
            .blur(BlurPolicy::Keep)
            .update_editable(cx, &mut self.state, &mut self.model)
            .erase();
        let _ = Button::new(OTHER, "other").update(cx);
        r
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(ID, &self.columns).draw(ui, Rect::new(0, 0, 24, 7), &self.state, &self.model);
        Button::new(OTHER, "other").draw(ui, Rect::new(26, 0, 8, 1));
    }
}

#[test]
fn keyed_move_reveals_both_axes_without_focus_or_model_changes() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 36, 9);
    let _ = h.key(KeyCode::Char(' '));
    let checked = h.app().state.selected_rows().clone();
    let model = h.app().model.clone();
    let focus = h.focus();
    assert_eq!(h.app_mut().move_to(39, 7), Ok(()));
    h.draw();
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(39), ColumnKey::num(7)))
    );
    assert!((0..7).any(|y| h.row(y).contains("r39c7")));
    assert!(h.app().state.col_offset() > 0);
    assert_eq!(h.app().state.selected_rows(), &checked);
    assert_eq!(h.app().model, model);
    assert_eq!(h.focus(), focus);
    assert!(!h.app().state.is_editing());
}

#[test]
fn invalid_removed_and_capped_out_keys_are_atomic() {
    let mut p = Page::new();
    assert_eq!(p.move_to(20, 3), Ok(()));
    for (row, col, expected) in [
        (99, 3, "grid row key is absent"),
        (20, 99, "grid column key is absent"),
    ] {
        let before = p.state.clone();
        let model = p.model.clone();
        assert_eq!(
            p.move_to(row, col).as_ref().map_err(String::as_str),
            Err(expected)
        );
        assert_eq!(p.state, before);
        assert_eq!(p.model, model);
    }
    p.model.rows.retain(|(key, _)| *key != 20);
    let before = p.state.clone();
    assert_eq!(
        p.move_to(20, 3).as_ref().map_err(String::as_str),
        Err("grid row key is absent")
    );
    assert_eq!(p.state, before);
    p.columns = (0..=junie_tui::GRID_MAX_COLUMNS)
        .map(|i| Column::new(ColumnKey::num(i as u16), "column"))
        .collect();
    assert_eq!(
        p.move_to(0, 64).as_ref().map_err(String::as_str),
        Err("grid column key is absent")
    );
    assert_eq!(p.state, before);
    p.columns.clear();
    assert_eq!(
        p.move_to(0, 0).as_ref().map_err(String::as_str),
        Err("grid column key is absent")
    );
    assert_eq!(p.state, before);
}

#[test]
fn current_model_and_column_order_resolve_stable_keys() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 36, 9);
    h.app_mut().model.rows.swap(5, 30);
    h.app_mut().columns.swap(1, 7);
    assert_eq!(h.app_mut().move_to(5, 1), Ok(()));
    h.draw();
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(5), ColumnKey::num(1)))
    );
    assert!((0..7).any(|y| h.row(y).contains("r5c7")));
    assert_eq!(h.app().state.col_offset(), 7);
}

#[test]
fn retained_edit_refuses_move_until_explicit_cancel_without_commit() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 36, 9);
    let _ = h.key(KeyCode::F(2));
    let _ = h.type_str("draft");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().state.edit_error().is_some());
    let _ = h.click_id(OTHER);
    let state = h.app().state.clone();
    let model = h.app().model.clone();
    let focus = h.focus();
    assert_eq!(
        h.app_mut().move_to(39, 7).as_ref().map_err(String::as_str),
        Err("grid inline edit is active")
    );
    assert_eq!(h.app().state, state);
    assert_eq!(h.app().model, model);
    assert!(h.app_mut().state.cancel_edit());
    assert!(!h.app_mut().state.cancel_edit());
    assert_eq!(h.app().state.cursor(), state.cursor());
    assert_eq!(h.app().state.selected_rows(), state.selected_rows());
    assert_eq!(h.app().state.edit_cell(), None);
    assert_eq!(h.app().state.edit_draft(), None);
    assert_eq!(h.app().state.edit_error(), None);
    assert_eq!(h.app().model, model);
    assert_eq!(h.focus(), focus);
    assert_eq!(h.app_mut().move_to(39, 7), Ok(()));
    h.draw();
    assert!((0..7).any(|y| h.row(y).contains("r39c7")));
}
