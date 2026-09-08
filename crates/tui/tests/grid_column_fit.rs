//! Complete columns form the logical viewport; clipped preview does not.
use junie_tui::{
    App, CellRef, Column, ColumnKey, Cx, EditIntent, FieldError, Grid, GridAction, GridColumnFit,
    GridEditor, GridModel, GridState, Id, ItemKey, KeyCode, Rect, Response, SortDir, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("fit.grid");
#[derive(Default)]
struct Model {
    commits: Vec<(usize, String)>,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        1
    }
    fn row_key(&self, _: usize) -> ItemKey {
        ItemKey::num(0)
    }
    fn cell(&self, _: usize, col: usize) -> Option<CellRef<'_>> {
        ["aaaaaaaa", "bbbbbbbbbbbb", "cccccccccccccccc"]
            .get(col)
            .map(|s| CellRef::new(s))
    }
}
impl GridEditor for Model {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_> {
        EditIntent::Inline {
            initial: self.cell(row, col).map_or("", |cell| cell.text),
        }
    }
    fn is_editable(&self, _: usize, _: usize) -> bool {
        true
    }
    fn apply_cycle(&mut self, _: usize, _: usize) {}
    fn commit_cell(&mut self, _: usize, col: usize, value: &str) -> Result<(), FieldError> {
        self.commits.push((col, value.to_owned()));
        Ok(())
    }
}

struct Page {
    model: Model,
    actions: Vec<GridAction>,
    state: GridState,
    columns: [Column<'static>; 3],
    area: Rect,
}
impl Page {
    fn new() -> Self {
        let mut columns = [
            Column::new(ColumnKey::num(0), "alpha"),
            Column::new(ColumnKey::num(1), "bravo"),
            Column::new(ColumnKey::num(2), "charlie"),
        ];
        for (c, w) in columns.iter_mut().zip([8, 12, 16]) {
            c.editable = true;
            c.min_width = w;
            c.max_width = w;
        }
        Self {
            model: Model::default(),
            actions: Vec::new(),
            state: GridState::default(),
            columns,
            area: Rect::new(0, 0, 32, 5),
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Grid::new(ID, &self.columns)
            .column_gap(2)
            .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
            .update_editable(cx, &mut self.state, &mut self.model);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(ID, &self.columns)
            .column_gap(2)
            .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
            .draw(ui, self.area, &self.state, &self.model);
    }
}
#[test]
fn preview_is_painted_but_keyboard_reveal_places_target_in_complete_viewport() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 32, 5);
    assert_eq!(h.cell(26, 1).symbol(), "c");
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(0), ColumnKey::num(2)))
    );
    assert_eq!(h.app().state.col_offset(), 1);
    assert_eq!(h.cell(2, 1).symbol(), "b");
    assert_eq!(h.cell(16, 1).symbol(), "c");
}
#[test]
fn preview_body_click_does_not_select_the_painted_column() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 32, 5);
    let before = h.app().state.cursor();
    let _ = h.click(28, 1);
    assert_eq!(h.app().state.cursor(), before);
    assert_eq!(h.app().state.col_offset(), 0);
    assert_eq!(h.cell(26, 1).symbol(), "c");
}

#[test]
fn six_cell_preview_threshold_is_exact() {
    for (width, preview) in [(31, false), (32, true), (33, true)] {
        let mut page = Page::new();
        page.area.width = width;
        let h = Harness::new(page, Theme::junie(), width, 5);
        assert_eq!(h.row(1).contains('c'), preview);
    }
}
#[test]
fn pending_keyed_reveal_survives_zero_layout_and_column_reorder() {
    for reorder in [false, true] {
        let mut page = Page::new();
        page.area = Rect::ZERO;
        let mut h = Harness::new(page, Theme::junie(), 32, 5);
        let page = h.app_mut();
        let moved = Grid::new(ID, &page.columns)
            .column_gap(2)
            .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
            .move_cursor_to(
                &mut page.state,
                &page.model,
                ItemKey::num(0),
                ColumnKey::num(2),
            );
        assert_eq!(moved, Ok(()));
        h.draw();
        if reorder {
            h.app_mut().columns.swap(0, 2);
        }
        h.app_mut().area = Rect::new(0, 0, 32, 5);
        h.draw();
        assert_eq!(
            h.app().state.cursor(),
            Some((ItemKey::num(0), ColumnKey::num(2)))
        );
        let expected_x = if reorder { 2 } else { 16 };
        assert_eq!(h.cell(expected_x, 0).symbol(), "c");
    }
}
#[test]
fn sticky_columns_stay_put_while_preview_target_reveals() {
    let mut page = Page::new();
    page.columns[0].sticky = true;
    let mut h = Harness::new(page, Theme::junie(), 32, 5);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.col_offset(), 1);
    assert_eq!(h.cell(2, 1).symbol(), "a");
    assert_eq!(h.cell(12, 1).symbol(), "c");
}
#[test]
fn resize_keeps_reference_offset_and_cursor_even_when_target_becomes_preview() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 32, 5);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    h.app_mut().area.width = 24;
    let _ = h.resize(24, 5);
    assert_eq!(h.app().state.col_offset(), 1);
    assert_eq!(h.cell(2, 0).symbol(), "b");
    assert_eq!(h.cell(16, 1).symbol(), "c");
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(0), ColumnKey::num(2)))
    );
}

#[test]
fn wider_target_uses_prior_complete_count_and_can_remain_preview() {
    let mut page = Page::new();
    for (col, width) in page.columns.iter_mut().zip([8, 8, 24]) {
        col.min_width = width;
        col.max_width = width;
    }
    let mut h = Harness::new(page, Theme::junie(), 32, 5);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.col_offset(), 1);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(0), ColumnKey::num(2)))
    );
    assert_eq!(h.cell(12, 1).symbol(), "c");
    let _ = h.click(14, 1);
    assert_eq!(h.app().state.col_offset(), 1);
}

#[test]
fn preview_keyboard_editor_uses_painted_rect_and_commits_same_column() {
    let mut page = Page::new();
    for (col, width) in page.columns.iter_mut().zip([8, 8, 24]) {
        col.min_width = width;
        col.max_width = width;
    }
    let mut h = Harness::new(page, Theme::junie(), 32, 5);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.edit_draft(), Some("cccccccccccccccc"));
    assert_eq!(h.app().state.col_offset(), 1);
    assert!(
        h.cursor()
            .is_some_and(|pos| (12..32).contains(&pos.x) && pos.y == 1)
    );
    let _ = h.click(14, 1);
    let _ = h.key(KeyCode::Char('Z'));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.edit_draft(), None);
    assert_eq!(h.app().model.commits, vec![(2, "ccccccccccccccccZ".into())]);
}

#[test]
fn preview_header_still_sorts_while_body_click_is_excluded() {
    let mut page = Page::new();
    for column in &mut page.columns {
        column.sortable = true;
    }
    let mut h = Harness::new(page, Theme::junie(), 32, 5);
    let _ = h.click(28, 0);
    assert_eq!(
        h.app().actions,
        vec![GridAction::Sort(ColumnKey::num(2), SortDir::Asc)]
    );
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::num(0), ColumnKey::num(0)))
    );
}
#[test]
fn pending_reveal_survives_tiny_unpublished_columns_and_removed_key() {
    for width in 0..=2 {
        let mut page = Page::new();
        page.area.width = width;
        let mut h = Harness::new(page, Theme::junie(), 32, 5);
        let page = h.app_mut();
        let result = Grid::new(ID, &page.columns)
            .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
            .move_cursor_to(
                &mut page.state,
                &page.model,
                ItemKey::num(0),
                ColumnKey::num(2),
            );
        assert!(result.is_ok());
        page.columns[2].key = ColumnKey::num(9);
        h.draw();
        h.app_mut().area.width = 32;
        let _ = h.resize(32, 5);
        assert_eq!(
            h.app().state.cursor(),
            Some((ItemKey::num(0), ColumnKey::num(9)))
        );
        assert_eq!(h.app().state.col_offset(), 1);
        assert_eq!(h.cell(16, 1).symbol(), "c");
    }
}

#[test]
fn sticky_only_tiny_viewport_does_not_consume_pending_nonsticky_reveal() {
    let mut page = Page::new();
    page.columns[0].sticky = true;
    page.columns[0].min_width = 40;
    page.columns[0].max_width = 40;
    page.area.width = 10;
    let mut h = Harness::new(page, Theme::junie(), 64, 5);
    let page = h.app_mut();
    assert!(
        Grid::new(ID, &page.columns)
            .column_gap(2)
            .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
            .move_cursor_to(
                &mut page.state,
                &page.model,
                ItemKey::num(0),
                ColumnKey::num(2)
            )
            .is_ok()
    );
    let _ = h.key(KeyCode::Null);
    h.app_mut().area.width = 64;
    let _ = h.resize(64, 5);
    assert_eq!(h.app().state.col_offset(), 1);
    assert_eq!(h.cell(44, 1).symbol(), "c");
}
