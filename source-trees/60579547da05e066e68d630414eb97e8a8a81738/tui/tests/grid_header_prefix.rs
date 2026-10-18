//! Header prefix geometry and customization remain owned by the shared grid.
use junie_tui::{
    App, CellRef, Color, Column, ColumnKey, Cx, Family, FgStep, GlyphRole, Grid, GridAction,
    GridHeaderSizing, GridModel, GridState, Id, ItemKey, Part, Rect, Response, Role, StylePatch,
    Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("prefix.grid");
const PREFIX: &[(ColumnKey, GlyphRole, Role)] = &[(
    ColumnKey::num(0),
    GlyphRole::PrimaryKey,
    Role::Fg(FgStep::Faint),
)];
struct Model;
impl GridModel for Model {
    fn row_count(&self) -> usize {
        2
    }
    fn row_key(&self, r: usize) -> ItemKey {
        ItemKey::index(r)
    }
    fn cell(&self, _: usize, _: usize) -> Option<CellRef<'_>> {
        Some(CellRef::new("x"))
    }
}
struct Page {
    state: GridState,
    columns: [Column<'static>; 2],
    prefix: bool,
    patches: Vec<(Part, StylePatch)>,
    slot: Option<Part>,
    scoped: bool,
    area: Rect,
    actions: Vec<GridAction>,
}
impl Page {
    fn new() -> Self {
        let mut cols = [
            Column::new(ColumnKey::num(0), "id"),
            Column::new(ColumnKey::num(1), "other"),
        ];
        for c in &mut cols {
            c.min_width = 1;
            c.sortable = true;
        }
        Self {
            state: GridState::default(),
            columns: cols,
            prefix: true,
            patches: Vec::new(),
            slot: None,
            scoped: false,
            area: Rect::new(0, 0, 30, 5),
            actions: Vec::new(),
        }
    }
    fn props(&self) -> Grid<'_> {
        let g = Grid::new(ID, &self.columns)
            .header_sizing(GridHeaderSizing::Minimum {
                padding: 2,
                cap: 40,
            })
            .header_prefixes(if self.prefix { PREFIX } else { &[] })
            .patch_part(&self.patches);
        if let Some(part) = self.slot {
            g.slot(part, &header)
        } else {
            g
        }
    }
}
fn header(ui: &mut Ui<'_>, area: Rect) {
    ui.paint_str(area, "HEADER SLOT", ui.surface_style());
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Grid::new(ID, &self.columns)
            .header_sizing(GridHeaderSizing::Minimum {
                padding: 2,
                cap: 40,
            })
            .header_prefixes(if self.prefix { PREFIX } else { &[] })
            .update(cx, &mut self.state, &Model);
        if let Some(a) = r.take_action() {
            self.actions.push(a);
        }
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        static SCOPE: [junie_tui::OverlayRule; 1] = [(
            Family::GRID,
            junie_tui::Variant::DEFAULT,
            Part::ICON,
            junie_tui::StateFlags::empty(),
            StylePatch::new().set_fg(Role::Info),
        )];
        if self.scoped {
            ui.with_overlay(&junie_tui::Overlay::new(&SCOPE), |ui| {
                self.props().draw(ui, self.area, &self.state, &Model);
            });
        } else {
            self.props().draw(ui, self.area, &self.state, &Model);
        }
    }
}
#[test]
fn primary_prefix_reserves_two_header_cells_and_keeps_body_text_unprefixed() {
    let mut h = Harness::new(Page::new(), Theme::junie(), 30, 5);
    assert_eq!(h.cell(2, 0).symbol(), "⚷");
    assert_eq!(
        Some(h.cell(2, 0).fg),
        Theme::junie().color.fg.get(FgStep::Faint.index()).copied()
    );
    assert_eq!(h.cell(4, 0).symbol(), "i");
    assert_eq!(h.cell(9, 0).symbol(), "o");
    assert_eq!(h.cell(2, 1).symbol(), "x");
    let _ = h.click(2, 0);
    assert!(matches!(h.app().actions.last(),Some(GridAction::Sort(k,_)) if *k==ColumnKey::num(0)));
    h.app_mut().columns.swap(0, 1);
    h.draw();
    assert_eq!(h.cell(2, 0).symbol(), "o");
    assert_eq!(h.cell(10, 0).symbol(), "⚷");
}
#[test]
fn prefix_defaults_resolve_before_icon_overrides_and_header_slot_replaces_all() {
    let mut theme = Theme::junie();
    theme.color.danger = Color::Rgb(13, 17, 19);
    theme.color.warning = Color::Rgb(23, 29, 31);
    theme.color.info = Color::Rgb(37, 41, 43);
    theme = theme.define_family(Family::GRID, |f| {
        f.part(Part::ICON).base(
            StylePatch::new()
                .set_fg(Role::Danger)
                .set_glyph(GlyphRole::Dirty),
        );
    });
    let mut h = Harness::new(Page::new(), theme, 30, 5);
    assert_eq!(h.cell(2, 0).fg, Color::Rgb(13, 17, 19));
    assert_eq!(h.cell(2, 0).symbol(), "•");
    h.app_mut().scoped = true;
    h.draw();
    assert_eq!(h.cell(2, 0).fg, Color::Rgb(37, 41, 43));
    h.app_mut()
        .patches
        .push((Part::ICON, StylePatch::new().set_fg(Role::Warning)));
    h.draw();
    assert_eq!(h.cell(2, 0).fg, Color::Rgb(23, 29, 31));
    h.app_mut().slot = Some(Part::HEADER);
    h.draw();
    assert_eq!(h.cell(0, 0).symbol(), "H");
    assert_eq!(h.cell(2, 0).symbol(), "A");
}
#[test]
fn absent_prefix_never_invents_status_or_icon() {
    let mut page = Page::new();
    page.prefix = false;
    page.patches
        .push((Part::ICON, StylePatch::new().set_glyph(GlyphRole::Error)));
    let h = Harness::new(page, Theme::junie(), 30, 5);
    assert_eq!(h.cell(2, 0).symbol(), "i");
    assert!(!h.buffer().content.iter().any(|c| c.symbol() == "!"));
}

#[test]
fn primary_key_role_is_independent_in_every_theme_and_color_mode() {
    for base in [Theme::junie(), Theme::paper()] {
        for level in [
            junie_tui::ColorLevel::TrueColor,
            junie_tui::ColorLevel::Ansi256,
            junie_tui::ColorLevel::Ansi16,
            junie_tui::ColorLevel::Mono,
        ] {
            let theme = base.clone().downgrade(level);
            assert_eq!(theme.design.glyphs.get(GlyphRole::PrimaryKey), "⚷");
            assert_eq!(theme.design.glyphs.get(GlyphRole::PrimaryMark), "▪");
            let previous = theme.design.glyphs.clone();
            let changed = theme.builder().glyph(GlyphRole::PrimaryKey, "K").build();
            for role in GlyphRole::ALL {
                assert_eq!(
                    changed.design.glyphs.get(role),
                    if role == GlyphRole::PrimaryKey {
                        "K"
                    } else {
                        previous.get(role)
                    }
                );
            }
            let h = Harness::new(Page::new(), changed, 30, 5);
            assert_eq!(h.cell(2, 0).symbol(), "K");
        }
    }
}
#[test]
fn right_aligned_prefix_and_title_form_one_group() {
    let mut page = Page::new();
    page.columns[0].align = junie_tui::Align::Right;
    page.columns[0].min_width = 8;
    page.columns[0].max_width = 8;
    let h = Harness::new(page, Theme::junie(), 30, 5);
    assert_eq!(h.cell(5, 0).symbol(), "⚷");
    assert_eq!(h.cell(7, 0).symbol(), "i");
    assert_eq!(h.cell(8, 0).symbol(), "d");
}
#[test]
fn prefix_slot_and_wide_custom_glyph_cannot_escape_one_cell() {
    let mut page = Page::new();
    page.slot = Some(Part::ICON);
    let mut h = Harness::new(page, Theme::junie(), 30, 5);
    assert_eq!(h.cell(2, 0).symbol(), "H");
    assert_eq!(h.cell(3, 0).symbol(), " ");
    assert_eq!(h.cell(4, 0).symbol(), "i");
    h.app_mut().slot = None;
    for width in 0..8 {
        h.app_mut().area = Rect::new(1, 1, width, 3);
        h.draw();
        for y in 0..5 {
            for x in 0..30 {
                if !h.app().area.contains((x, y).into()) {
                    assert_eq!(h.cell(x, y).symbol(), " ");
                }
            }
        }
    }
    let theme = Theme::junie()
        .builder()
        .glyph(GlyphRole::PrimaryKey, "界")
        .build();
    let h = Harness::new(Page::new(), theme, 30, 5);
    assert_eq!(h.cell(3, 0).symbol(), " ");
    assert_eq!(h.cell(4, 0).symbol(), "i");
}
