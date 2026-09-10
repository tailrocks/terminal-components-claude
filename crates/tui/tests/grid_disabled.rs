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
    disabled: bool,
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
            disabled: false,
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = Grid::new(ID, &self.columns)
            .disabled(self.disabled)
            .blur(BlurPolicy::Keep)
            .update_editable(cx, &mut self.state, &mut self.model)
            .erase();
        let _ = Button::new(OTHER, "other").update(cx);
        r
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(ID, &self.columns).disabled(self.disabled).draw(
            ui,
            Rect::new(0, 0, 24, 7),
            &self.state,
            &self.model,
        );
        Button::new(OTHER, "other").draw(ui, Rect::new(26, 0, 8, 1));
    }
}

#[test]
fn disabling_published_editor_retains_draft_without_domain_commit() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 36, 9);
    let _ = h.key(KeyCode::F(2));
    assert!(h.app().state.is_editing());
    let state = h.app().state.clone();
    h.app_mut().disabled = true;
    // Queued input reaches the old published editor; current props must veto it.
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().model.commits, 0);
    assert_eq!(h.app().state, state);
    assert_ne!(h.focus(), Some(Grid::new(ID, &[]).editor_id()));
    let _ = h.click(4, 1);
    let _ = h.key(KeyCode::Char('G'));
    assert_eq!(h.app().state, state);
}

#[test]
fn disabling_before_queued_navigation_preserves_cursor_and_selection() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 36, 9);
    let _ = h.key(KeyCode::Char(' '));
    let state = h.app().state.clone();
    h.app_mut().disabled = true;
    let _ = h.key(KeyCode::Char('G'));
    assert_eq!(h.app().state, state);
    assert_ne!(h.focus(), Some(ID));
    let _ = h.key(KeyCode::F(2));
    assert_eq!(h.app().state, state);
    h.app_mut().disabled = false;
    h.draw();
    let _ = h.click(4, 2);
    assert_ne!(h.app().state.cursor(), state.cursor());
}
#[test]
fn disabling_during_pointer_capture_rejects_release_action() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 36, 9);
    let _ = h.mouse(junie_tui::MouseKind::Down, 4, 2);
    let state = h.app().state.clone();
    h.app_mut().disabled = true;
    let _ = h.mouse(junie_tui::MouseKind::Up, 4, 2);
    assert_eq!(h.app().state, state);
    assert_eq!(h.app().model.commits, 0);
}

struct DisabledReadOnly(Page, usize);
impl App for DisabledReadOnly {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Grid::new(ID, &self.0.columns)
            .disabled(true)
            .update(cx, &mut self.0.state, &self.0.model)
            .on_action(|_| self.1 = self.1.saturating_add(1))
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.0.draw(ui);
    }
}
#[test]
fn disabled_read_only_entry_emits_no_actions() {
    let mut page = Page::new();
    page.disabled = true;
    let state = page.state.clone();
    let mut h = Harness::new(DisabledReadOnly(page, 0), Theme::junie(), 36, 9);
    for key in [
        KeyCode::Down,
        KeyCode::Enter,
        KeyCode::F(2),
        KeyCode::Char(' '),
    ] {
        let _ = h.key(key);
    }
    let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
    let _ = h.click(4, 1);
    assert_eq!(h.app().0.state, state);
    assert_eq!(h.app().1, 0);
}
