//! Fetch-row activation follows the pinned Holla keyboard contract.
use junie_tui::{
    App, CellRef, Column, ColumnKey, Cx, Grid, GridAction, GridModel, GridState, Id, ItemKey,
    KeyCode, Rect, Response, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("fetch.grid");
const COLUMNS: [Column<'static>; 1] = [{
    let mut column = Column::new(ColumnKey::num(0), "value");
    column.editable = true;
    column
}];
struct Model {
    len: usize,
    more: bool,
    hooks: usize,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.len
    }
    fn row_key(&self, row: usize) -> ItemKey {
        assert!(row < self.len);
        ItemKey::index(row)
    }
    fn cell(&self, row: usize, _: usize) -> Option<CellRef<'_>> {
        (row < self.len).then_some(CellRef::new("data"))
    }
    fn has_more(&self) -> bool {
        self.more
    }
}
struct Page {
    model: Model,
    state: GridState,
    activate: bool,
    fetches: usize,
    disabled: bool,
    copies: Vec<String>,
    area: Rect,
}
impl Page {
    fn new(activate: bool, len: usize) -> Self {
        Self {
            model: Model {
                len,
                more: true,
                hooks: 0,
            },
            state: GridState::default(),
            activate,
            fetches: 0,
            disabled: false,
            copies: Vec::new(),
            area: Rect::new(0, 0, 30, 8),
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = Grid::new(ID, &COLUMNS)
            .fetch_on_activate(self.activate)
            .disabled(self.disabled)
            .update(cx, &mut self.state, &self.model);
        r.on_action(|action| match action {
            GridAction::FetchMore => self.fetches = self.fetches.saturating_add(1),
            GridAction::Copy(text) => self.copies.push(text),
            _ => {}
        })
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        Grid::new(ID, &COLUMNS)
            .fetch_on_activate(self.activate)
            .disabled(self.disabled)
            .draw(ui, self.area, &self.state, &self.model);
    }
}
#[test]
fn sentinel_waits_for_activation_and_append_selects_first_new_key() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    assert!(h.app().state.on_fetch_row());
    assert_eq!(h.app().state.cursor(), None);
    assert_eq!(h.app().fetches, 0);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().fetches, 1);
    h.app_mut().model.len = 6;
    let _ = h.key(KeyCode::Null);
    assert!(!h.app().state.on_fetch_row());
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(3), ColumnKey::num(0)))
    );
}
#[test]
fn default_reach_fetch_remains_compatible() {
    let mut h = Harness::new(Page::new(false, 3), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().fetches, 1);
    assert!(!h.app().state.on_fetch_row());
}

#[test]
fn unchanged_removed_and_empty_sentinel_transitions() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    let _ = h.key(KeyCode::Null);
    assert!(h.app().state.on_fetch_row());
    h.app_mut().model.more = false;
    let _ = h.key(KeyCode::Null);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(2), ColumnKey::num(0)))
    );
    let mut empty = Harness::new(Page::new(true, 0), Theme::junie(), 30, 8);
    assert!(empty.app().state.on_fetch_row());
    assert!((0..8).any(|y| empty.row(y).contains("more")));
    let _ = empty.key(KeyCode::Enter);
    assert_eq!(empty.app().fetches, 1);
    empty.app_mut().model.more = false;
    let _ = empty.key(KeyCode::Null);
    assert!(!empty.app().state.on_fetch_row());
    assert_eq!(empty.app().state.cursor(), None);
}
#[test]
fn each_activation_key_fetches_once_and_navigation_never_fetches() {
    for key in [KeyCode::Enter, KeyCode::F(2), KeyCode::Char(' ')] {
        let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
        let _ = h.key(KeyCode::PageDown);
        assert!(h.app().state.on_fetch_row());
        let _ = h.key(KeyCode::Down);
        assert_eq!(h.app().fetches, 0);
        let _ = h.key(key);
        assert_eq!(h.app().fetches, 1);
        assert!(h.app().state.selected_rows().is_empty());
        let _ = h.key(KeyCode::Up);
        assert_eq!(
            h.app().state.cursor(),
            Some((ItemKey::index(2), ColumnKey::num(0)))
        );
    }
}
#[test]
fn clicking_authoritative_fetch_row_selects_and_fetches() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    // Header y=0; three data rows y=1..3; fetch row y=4.
    let _ = h.click(4, 4);
    assert!(h.app().state.on_fetch_row());
    assert_eq!(h.app().fetches, 1);
}

#[test]
fn disabled_fetch_row_ignores_keyboard_and_click() {
    let mut page = Page::new(true, 3);
    page.disabled = true;
    let mut h = Harness::new(page, Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    let _ = h.key(KeyCode::Enter);
    let _ = h.click(4, 4);
    assert_eq!(h.app().fetches, 0);
    assert!(!h.app().state.on_fetch_row());
}
#[test]
fn sentinel_copy_contains_no_model_cell_and_keyed_move_clears_it() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
    assert_eq!(h.app().copies, [String::new()]);
    let page = h.app_mut();
    let moved = Grid::new(ID, &COLUMNS)
        .fetch_on_activate(true)
        .move_cursor_to(
            &mut page.state,
            &page.model,
            ItemKey::index(1),
            ColumnKey::num(0),
        );
    assert_eq!(moved, Ok(()));
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(1), ColumnKey::num(0)))
    );
    assert!(!h.app().state.on_fetch_row());
}

struct Editable(Page);
impl junie_tui::GridEditor for Model {
    fn edit_intent(&self, _: usize, _: usize) -> junie_tui::EditIntent<'_> {
        junie_tui::EditIntent::Inline { initial: "data" }
    }
    fn apply_cycle(&mut self, _: usize, _: usize) {
        self.hooks = self.hooks.saturating_add(1);
    }
    fn commit_cell(&mut self, _: usize, _: usize, _: &str) -> Result<(), junie_tui::FieldError> {
        self.hooks = self.hooks.saturating_add(1);
        Ok(())
    }
    fn is_editable(&self, _: usize, _: usize) -> bool {
        true
    }
}
impl App for Editable {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Grid::new(ID, &COLUMNS)
            .fetch_on_activate(true)
            .blur(junie_tui::BlurPolicy::Keep)
            .update_editable(cx, &mut self.0.state, &mut self.0.model)
            .on_action(|a| {
                if matches!(a, GridAction::FetchMore) {
                    self.0.fetches = self.0.fetches.saturating_add(1);
                }
            })
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.0.draw(ui);
    }
}
#[test]
fn retained_draft_refuses_sentinel_click_without_commit_or_fetch() {
    let mut h = Harness::new(Editable(Page::new(true, 3)), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::F(2));
    assert!(h.app().0.state.is_editing());
    let draft = h.app().0.state.edit_draft().map(str::to_owned);
    let _ = h.click(4, 4);
    assert_eq!(h.app().0.fetches, 0);
    assert_eq!(h.app().0.model.hooks, 0);
    assert!(!h.app().0.state.on_fetch_row());
    assert_eq!(h.app().0.state.edit_draft(), draft.as_deref());
}

#[test]
fn range_can_end_at_sentinel_but_copy_excludes_synthetic_data() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    for _ in 0..3 {
        let _ = h.key_mod(KeyCode::Down, junie_tui::KeyModifiers::SHIFT);
    }
    assert!(h.app().state.on_fetch_row());
    let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
    assert_eq!(h.app().copies, ["data\ndata\ndata\n"]);
    assert_eq!(h.app().fetches, 0);
}

#[test]
fn sentinel_reveal_survives_zero_layout_and_resize() {
    let mut page = Page::new(true, 40);
    page.area = Rect::new(0, 0, 30, 0);
    let mut h = Harness::new(page, Theme::junie(), 30, 8);
    h.app_mut().area = Rect::new(0, 0, 30, 8);
    h.draw();
    let _ = h.key(KeyCode::Char('G'));
    assert!(h.app().state.on_fetch_row());
    assert!(h.row(7).contains("more"));
    for height in [4, 8, 3, 8] {
        h.app_mut().area.height = height;
        let _ = h.resize(30, height);
        assert!(h.row(height.saturating_sub(1)).contains("more"));
        assert_eq!(h.app().fetches, 0);
    }
}

#[test]
fn scrolling_away_from_selected_sentinel_does_not_force_reveal() {
    let mut h = Harness::new(Page::new(true, 40), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    assert!(h.row(7).contains("more"));
    let _ = h.mouse(junie_tui::MouseKind::Wheel(junie_tui::Axis::V, -3), 4, 3);
    assert!(!(0..8).any(|y| h.row(y).contains("more")));
    let _ = h.key(KeyCode::Null);
    assert!(!(0..8).any(|y| h.row(y).contains("more")));
    assert!(h.app().state.on_fetch_row());
    assert_eq!(h.app().fetches, 0);
}

#[test]
fn horizontal_navigation_preserves_sentinel_and_vertical_navigation_leaves_it() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    for key in [KeyCode::Left, KeyCode::Right, KeyCode::Home, KeyCode::End] {
        let _ = h.key(key);
        assert!(h.app().state.on_fetch_row());
        assert_eq!(h.app().fetches, 0);
    }
    let _ = h.key(KeyCode::PageUp);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(0), ColumnKey::num(0)))
    );
}

#[test]
fn shrinking_loaded_rows_retargets_sentinel_without_model_key_fabrication() {
    let mut h = Harness::new(Page::new(true, 5), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    h.app_mut().model.len = 2;
    let _ = h.key(KeyCode::Null);
    assert!(h.app().state.on_fetch_row());
    let _ = h.key(KeyCode::Up);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(1), ColumnKey::num(0)))
    );
    let _ = h.key(KeyCode::Down);
    h.app_mut().model.len = 4;
    h.app_mut().model.more = false;
    let _ = h.key(KeyCode::Null);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(2), ColumnKey::num(0)))
    );
    assert_eq!(h.app().fetches, 0);
}

#[test]
fn extending_from_sentinel_anchors_at_synthetic_boundary() {
    let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
    let _ = h.key(KeyCode::Char('G'));
    let _ = h.key_mod(KeyCode::Up, junie_tui::KeyModifiers::SHIFT);
    let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
    assert_eq!(h.app().copies, ["data\n"]);
    let _ = h.key_mod(KeyCode::Down, junie_tui::KeyModifiers::SHIFT);
    let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
    assert_eq!(h.app().copies, ["data\n", ""]);
}

#[test]
fn synthetic_anchor_materializes_on_append_and_clips_when_more_disappears() {
    for append in [false, true] {
        let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
        let _ = h.key(KeyCode::Char('G'));
        let _ = h.key_mod(KeyCode::Up, junie_tui::KeyModifiers::SHIFT);
        if append {
            h.app_mut().model.len = 5;
        }
        h.app_mut().model.more = false;
        let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
        assert_eq!(
            h.app().copies,
            [if append { "data\ndata\n" } else { "data\n" }]
        );
        assert_eq!(h.app().fetches, 0);
    }
}
#[test]
fn horizontal_extension_at_sentinel_copies_no_real_row() {
    for direction in [KeyCode::Left, KeyCode::Right] {
        let mut h = Harness::new(Page::new(true, 3), Theme::junie(), 30, 8);
        let _ = h.key(KeyCode::Char('G'));
        let _ = h.key_mod(direction, junie_tui::KeyModifiers::SHIFT);
        let _ = h.key_mod(KeyCode::Char('c'), junie_tui::KeyModifiers::CONTROL);
        assert_eq!(h.app().copies, [""]);
    }
}
