//! Grid's editor owns blur policy; applications inspect drafts without staging them.
use junie_tui::{
    App, BlurPolicy, Button, CellRef, Column, ColumnKey, Cx, EditIntent, FieldError, Grid,
    GridEditor, GridModel, GridState, Id, ItemKey, KeyCode, Part, Rect, Response, Ui,
};
use junie_tui_testing::Harness;
const GRID: Id = Id::root("blur.grid");
const OTHER: Id = Id::root("blur.other");
const COL: ColumnKey = ColumnKey::num(1);
struct Model {
    rows: Vec<(u64, String)>,
    commits: usize,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.rows.len()
    }
    fn row_key(&self, row: usize) -> ItemKey {
        self.rows
            .get(row)
            .map_or(ItemKey::num(0), |r| ItemKey::num(r.0))
    }
    fn cell(&self, row: usize, _: usize) -> Option<CellRef<'_>> {
        self.rows.get(row).map(|r| CellRef::new(&r.1))
    }
}
impl GridEditor for Model {
    fn edit_intent(&self, row: usize, _: usize) -> EditIntent<'_> {
        EditIntent::Inline {
            initial: self.rows.get(row).map_or("", |r| &r.1),
        }
    }
    fn apply_cycle(&mut self, _: usize, _: usize) {}
    fn is_editable(&self, _: usize, _: usize) -> bool {
        true
    }
    fn commit_cell(&mut self, row: usize, _: usize, text: &str) -> Result<(), FieldError> {
        self.commits = self.commits.saturating_add(1);
        if text == "bad" {
            return Err(FieldError::new("rejected"));
        }
        if let Some(r) = self.rows.get_mut(row) {
            text.clone_into(&mut r.1);
        }
        Ok(())
    }
}
struct Page {
    grid: GridState,
    model: Model,
    policy: Option<BlurPolicy>,
}
impl Page {
    fn new(policy: BlurPolicy) -> Self {
        Self {
            grid: GridState::default(),
            model: Model {
                rows: vec![(10, "old".into()), (20, "two".into())],
                commits: 0,
            },
            policy: Some(policy),
        }
    }
}
fn columns() -> [Column<'static>; 1] {
    let mut c = Column::new(COL, "value");
    c.editable = true;
    c.min_width = 12;
    [c]
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let columns = columns();
        let grid = Grid::new(GRID, &columns);
        let grid = self
            .policy
            .map_or(grid, |policy| Grid::new(GRID, &columns).blur(policy));
        let r = grid
            .update_editable(cx, &mut self.grid, &mut self.model)
            .erase();
        let _ = Button::new(OTHER, "other").update(cx);
        r
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(GRID, &columns()).draw(ui, Rect::new(0, 0, 20, 6), &self.grid, &self.model);
        Button::new(OTHER, "other").draw(ui, Rect::new(22, 0, 8, 1));
    }
}
fn start(policy: BlurPolicy, draft: &str) -> Harness<Page> {
    let mut h = Harness::new(Page::new(policy), junie_tui::Theme::junie(), 32, 8);
    let _ = h.key(KeyCode::F(2));
    let _ = h.key(KeyCode::End);
    for _ in 0..3 {
        let _ = h.key(KeyCode::Backspace);
    }
    let _ = h.type_str(draft);
    assert_eq!(h.app().grid.edit_cell(), Some((ItemKey::num(10), COL)));
    assert_eq!(h.app().grid.edit_draft(), Some(draft));
    h
}

#[test]
fn keep_retains_draft_across_blur_and_reorder_then_enter_validates() {
    let mut h = start(BlurPolicy::Keep, "new");
    let _ = h.click_id(OTHER);
    assert_eq!(h.app().model.commits, 0);
    assert_eq!(h.app().model.cell(0, 0).map(|c| c.text), Some("old"));
    assert_eq!(h.app().grid.edit_draft(), Some("new"));
    h.app_mut().model.rows.swap(0, 1);
    let _ = h.key(KeyCode::Null);
    assert_eq!(h.app().grid.edit_cell(), Some((ItemKey::num(10), COL)));
    let _ = h.click_id(GRID.part(Part::TEXT));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().model.commits, 1);
    assert_eq!(h.app().model.cell(1, 0).map(|c| c.text), Some("new"));
    assert_eq!(h.app().grid.edit_draft(), None);
    assert_eq!(h.app().grid.edit_cell(), None);
}

#[test]
fn explicit_commit_error_keeps_draft_and_escape_discards_it() {
    let mut h = start(BlurPolicy::Keep, "bad");
    let _ = h.click_id(OTHER);
    let _ = h.click_id(GRID.part(Part::TEXT));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().model.commits, 1);
    assert!(h.app().grid.edit_error().is_some());
    assert_eq!(h.app().grid.edit_draft(), Some("bad"));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().grid.edit_draft(), None);
    assert_eq!(h.app().grid.edit_cell(), None);
    assert_eq!(h.app().model.cell(0, 0).map(|c| c.text), Some("old"));
}

#[test]
fn removal_cancels_retained_draft_without_staging_another_row() {
    let mut h = start(BlurPolicy::Keep, "draft");
    let _ = h.click_id(OTHER);
    h.app_mut().model.rows.remove(0);
    let _ = h.key(KeyCode::Null);
    assert_eq!(h.app().grid.edit_cell(), None);
    assert_eq!(h.app().grid.edit_draft(), None);
    assert_eq!(h.app().model.commits, 0);
    assert_eq!(h.app().model.cell(0, 0).map(|c| c.text), Some("two"));
}

#[test]
fn default_policy_still_commits_on_blur() {
    let mut h = start(BlurPolicy::CommitAndValidate, "new");
    h.app_mut().policy = None;
    let _ = h.click_id(OTHER);
    assert_eq!(h.app().model.commits, 1);
    assert_eq!(h.app().model.cell(0, 0).map(|c| c.text), Some("new"));
    assert_eq!(h.app().grid.edit_cell(), None);
}
