//! Borrowed fetch wording and canonical activity preserve synthetic-row ownership.
use junie_tui::{
    App, CellRef, Color, Column, ColumnKey, Cx, Family, GlyphRole, Grid, GridAction, GridGutter,
    GridModel, GridState, Id, ItemKey, KeyCode, Part, Rect, Response, Role, StylePatch, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("fetch.presentation");
struct Model;
impl GridModel for Model {
    fn row_count(&self) -> usize {
        2
    }
    fn row_key(&self, r: usize) -> ItemKey {
        assert!(r < 2);
        ItemKey::index(r)
    }
    fn cell(&self, r: usize, _: usize) -> Option<CellRef<'_>> {
        (r < 2).then_some(CellRef::new("data"))
    }
    fn has_more(&self) -> bool {
        true
    }
}
struct Page {
    state: GridState,
    columns: [Column<'static>; 1],
    label: String,
    activity: Option<usize>,
    glyph: GlyphRole,
    default_label: bool,
    area: Rect,
    patches: Vec<(Part, StylePatch)>,
    actions: Vec<GridAction>,
}
impl Page {
    fn new() -> Self {
        Self {
            state: GridState::default(),
            columns: [Column::new(ColumnKey::num(0), "data")],
            label: "2 loaded · Enter fetches more".into(),
            activity: None,
            glyph: GlyphRole::MoreRows,
            default_label: false,
            area: Rect::new(0, 0, 40, 6),
            patches: Vec::new(),
            actions: Vec::new(),
        }
    }
    fn props(&self) -> Grid<'_> {
        let grid = Grid::new(ID, &self.columns)
            .gutter(GridGutter::Detailed {
                row_numbers: true,
                min_digits: 2,
            })
            .fetch_on_activate(true)
            .fetch_glyph(self.glyph)
            .fetch_activity(self.activity)
            .patch_part(&self.patches);
        if self.default_label {
            grid
        } else {
            grid.fetch_label(&self.label)
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Grid::new(ID, &self.columns)
            .gutter(GridGutter::Detailed {
                row_numbers: true,
                min_digits: 2,
            })
            .fetch_on_activate(true)
            .update(cx, &mut self.state, &Model);
        if let Some(a) = r.take_action() {
            self.actions.push(a);
        }
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.props().draw(ui, self.area, &self.state, &Model);
    }
}
#[test]
fn borrowed_label_begins_after_gutter_and_idle_glyph_and_click_still_fetches() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 40, 6);
    assert_eq!(h.cell(6, 3).symbol(), "↓");
    assert_eq!(h.cell(8, 3).symbol(), "2");
    assert_eq!(h.cell(4, 3).symbol(), " ");
    let _ = h.key(KeyCode::Char('G'));
    assert!(
        !h.app()
            .actions
            .iter()
            .any(|a| matches!(a, GridAction::FetchMore))
    );
    let _ = h.click(20, 3);
    assert!(matches!(
        h.app().actions.last(),
        Some(GridAction::FetchMore)
    ));
}
#[test]
fn activity_uses_canonical_frame_and_keeps_row_fill_but_progress_style_ownership() {
    let mut page = Page::new();
    page.activity = Some(3);
    page.label = "fetching…".into();
    page.patches
        .push((Part::ROW, StylePatch::new().set_bg(Role::DangerTint)));
    let mut theme = Theme::junie();
    theme.color.danger_tint = Color::Rgb(13, 17, 19);
    theme.color.info = Color::Rgb(23, 29, 31);
    let glyph = theme.design.motion.spinner_frames.get(3).copied();
    theme = theme.define_family(Family::PROGRESS, |f| {
        f.part(Part::ICON)
            .base(StylePatch::new().set_fg(Role::Info));
        f.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let h = Harness::new(page, theme, 40, 6);
    assert_eq!(Some(h.cell(6, 3).symbol()), glyph);
    assert_eq!(h.cell(8, 3).symbol(), "f");
    for x in [6, 8] {
        assert_eq!(h.cell(x, 3).fg, Color::Rgb(23, 29, 31));
        assert_eq!(h.cell(x, 3).bg, Color::Rgb(13, 17, 19));
    }
    assert_eq!(h.cell(4, 3).symbol(), " ");
}

#[test]
fn default_wording_and_idle_glyph_overrides_preserve_activity_independence() {
    let mut page = Page::new();
    page.default_label = true;
    let mut h = Harness::new(page, Theme::junie(), 40, 6);
    assert_eq!(h.cell(8, 3).symbol(), "m");
    h.app_mut().glyph = GlyphRole::Dirty;
    h.draw();
    assert_eq!(h.cell(6, 3).symbol(), "•");
    h.app_mut()
        .patches
        .push((Part::ROW, StylePatch::new().set_glyph(GlyphRole::Error)));
    h.draw();
    assert_eq!(h.cell(6, 3).symbol(), "!");
    h.app_mut().patches.push((
        Part::ROW,
        StylePatch {
            glyph: junie_tui::Slot::Clear,
            ..StylePatch::new()
        },
    ));
    h.draw();
    assert_eq!(h.cell(6, 3).symbol(), " ");
    assert_eq!(h.cell(8, 3).symbol(), "m");
    let loaded = h.cell(6, 1).clone();
    h.app_mut().activity = Some(1);
    h.draw();
    assert_eq!(h.cell(6, 1), &loaded);
    assert_ne!(h.cell(6, 3).symbol(), "!");
}
#[test]
fn narrow_activity_and_idle_rows_clip_without_touching_neighbor_cells() {
    for activity in [None, Some(0), Some(usize::MAX)] {
        for width in 0..12 {
            let mut page = Page::new();
            page.area = Rect::new(2, 1, width, 5);
            page.activity = activity;
            let h = Harness::new(page, Theme::junie(), 40, 8);
            for y in 0..8 {
                for x in 0..40 {
                    if !h.app().area.contains((x, y).into()) {
                        assert_eq!(h.cell(x, y).symbol(), " ");
                    }
                }
            }
        }
    }
}
