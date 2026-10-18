//! Source-number gutters share the grid's authoritative column geometry.
use junie_tui::{
    App, CellRef, Column, ColumnKey, Cx, GlyphRole, Grid, GridGutter, GridModel, GridState, Id,
    ItemKey, Rect, Response, RowDecor, Theme, Ui,
};
use junie_tui::{Color, Family, KeyCode, Part, Role, StateFlags, StylePatch};
use junie_tui_testing::Harness;
const ID: Id = Id::root("gutter.grid");
struct Model {
    order: Vec<usize>,
    more: bool,
}
impl GridModel for Model {
    fn row_count(&self) -> usize {
        self.order.len()
    }
    fn row_key(&self, row: usize) -> ItemKey {
        assert!(row < self.order.len(), "synthetic row has no key");
        ItemKey::index(self.order.get(row).copied().unwrap_or(row))
    }
    fn has_more(&self) -> bool {
        self.more
    }
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.order.get(row)?;
        ["aaaa", "bbbb", "cccc"].get(col).map(|s| CellRef::new(s))
    }
    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        assert!(row < self.order.len(), "synthetic row has no source number");
        RowDecor {
            marker: Some(GlyphRole::Dirty),
            number: Some(
                self.order
                    .get(row)
                    .copied()
                    .unwrap_or(row)
                    .saturating_add(1),
            ),
            ..RowDecor::default()
        }
    }
}
struct Page {
    model: Model,
    state: GridState,
    columns: [Column<'static>; 3],
    area: Rect,
    numbers: bool,
    compact: bool,
    disabled: bool,
    fit: junie_tui::GridColumnFit,
    reserve: u16,
    actions: Vec<junie_tui::GridAction>,
    defaults: Vec<(Part, StylePatch)>,
    patches: Vec<(Part, StylePatch)>,
    slot: Option<Part>,
}
impl Page {
    fn new(rows: usize) -> Self {
        let mut columns = [
            Column::new(ColumnKey::num(0), "alpha"),
            Column::new(ColumnKey::num(1), "bravo"),
            Column::new(ColumnKey::num(2), "charlie"),
        ];
        for col in &mut columns {
            col.min_width = 8;
            col.max_width = 8;
        }
        Self {
            model: Model {
                order: (0..rows).collect(),
                more: false,
            },
            state: GridState::default(),
            columns,
            area: Rect::new(0, 0, 40, 6),
            numbers: true,
            compact: false,
            disabled: false,
            fit: junie_tui::GridColumnFit::Whole,
            reserve: 4,
            actions: Vec::new(),
            defaults: Vec::new(),
            patches: Vec::new(),
            slot: None,
        }
    }
    fn grid(&self) -> Grid<'_> {
        let grid = Grid::new(ID, &self.columns)
            .gutter(if self.compact {
                GridGutter::Compact
            } else {
                GridGutter::Detailed {
                    row_numbers: self.numbers,
                    min_digits: 2,
                }
            })
            .disabled(self.disabled)
            .column_fit(self.fit)
            .right_reserve(self.reserve)
            .fetch_on_activate(true)
            .column_gap(2)
            .part_defaults(&self.defaults)
            .patch_part(&self.patches);
        match self.slot {
            Some(part) => grid.slot(part, &slot_probe),
            None => grid,
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Grid::new(ID, &self.columns)
            .gutter(if self.compact {
                GridGutter::Compact
            } else {
                GridGutter::Detailed {
                    row_numbers: self.numbers,
                    min_digits: 2,
                }
            })
            .disabled(self.disabled)
            .column_fit(self.fit)
            .right_reserve(self.reserve)
            .fetch_on_activate(true)
            .column_gap(2)
            .update(cx, &mut self.state, &self.model);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.grid().draw(ui, self.area, &self.state, &self.model);
    }
}
#[test]
fn source_numbers_reorder_and_gutter_click_keeps_change_marker_separate() {
    let mut h = Harness::new(Page::new(40), Theme::junie(), 40, 6);
    assert_eq!(h.cell(4, 1).symbol(), "1");
    assert_eq!(h.cell(6, 0).symbol(), "a");
    assert_eq!(h.cell(16, 0).symbol(), "b");
    assert_eq!(h.cell(26, 0).symbol(), "c");
    h.app_mut().model.order.reverse();
    h.draw();
    assert_eq!(h.cell(3, 1).symbol(), "4");
    assert_eq!(h.cell(4, 1).symbol(), "0");
    let _ = h.click(4, 1);
    assert!(h.app().state.selected_rows().contains(ItemKey::index(39)));
    assert_ne!(h.cell(1, 1).symbol(), " ");
    assert_ne!(h.cell(2, 1).symbol(), " ");
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(39), ColumnKey::num(0)))
    );
    let _ = h.click(17, 1);
    assert_eq!(
        h.app().state.cursor(),
        Some((ItemKey::index(39), ColumnKey::num(1)))
    );
}
#[test]
fn loaded_count_controls_number_width_and_disabled_numbers_have_no_row_toggle() {
    let mut h = Harness::new(Page::new(100), Theme::junie(), 40, 6);
    assert_eq!(h.cell(7, 0).symbol(), "a");
    h.app_mut().numbers = false;
    h.draw();
    assert_eq!(h.cell(3, 0).symbol(), "a");
    let _ = h.click(1, 1);
    assert!(h.app().state.selected_rows().is_empty());
}

fn slot_probe(ui: &mut Ui<'_>, area: Rect) {
    ui.paint_str(
        Rect::new(0, area.y, 40, 1),
        "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!",
        ui.surface_style(),
    );
}
#[test]
fn detailed_parts_resolve_authored_defaults_then_theme_then_instance() {
    for (part, x) in [
        (Part::GUTTER, 0),
        (Part::MARKER, 1),
        (Part::CHANGE, 2),
        (Part::ROW_NUMBER, 4),
    ] {
        let mut page = Page::new(40);
        page.defaults
            .push((part, StylePatch::new().set_fg(Role::Danger)));
        let mut theme = Theme::junie();
        theme.color.danger = Color::Rgb(13, 17, 19);
        theme.color.accent = Color::Rgb(23, 29, 31);
        theme.color.warning = Color::Rgb(37, 41, 43);
        let mut default_page = Page::new(40);
        default_page
            .defaults
            .push((part, StylePatch::new().set_fg(Role::Danger)));
        let baseline = Harness::new(default_page, theme.clone(), 40, 6);
        assert_eq!(
            baseline.cell(x, 2).fg,
            Color::Rgb(13, 17, 19),
            "authored {part:?}"
        );
        // An explicit recipe must beat the borrowed author default, including states.
        theme = theme.define_family(Family::GRID, |family| {
            family
                .part(part)
                .base(StylePatch::new().set_fg(Role::Accent))
                .when(
                    StateFlags::FOCUSED | StateFlags::CHECKED,
                    StylePatch::new().set_fg(Role::Accent),
                );
        });
        let mut h = Harness::new(page, theme, 40, 6);
        let _ = h.key(KeyCode::Char(' '));
        assert_eq!(h.cell(x, 1).fg, Color::Rgb(23, 29, 31), "theme {part:?}");
        h.app_mut()
            .patches
            .push((part, StylePatch::new().set_fg(Role::Warning)));
        h.draw();
        assert_eq!(h.cell(x, 1).fg, Color::Rgb(37, 41, 43));
    }
}
#[test]
fn detailed_slots_replace_only_their_clipped_subcells() {
    for (part, xs) in [
        (Part::GUTTER, 0..1),
        (Part::MARKER, 1..2),
        (Part::CHANGE, 2..3),
        (Part::ROW_NUMBER, 3..5),
    ] {
        let mut h = Harness::new(Page::new(40), Theme::junie(), 40, 6);
        let before = h.buffer().clone();
        h.app_mut().slot = Some(part);
        h.draw();
        for y in 0..6 {
            for x in 0..40 {
                if y > 0 && xs.contains(&x) {
                    assert_eq!(h.cell(x, y).symbol(), "!", "{part:?} at{x},{y}");
                } else {
                    assert_eq!(
                        h.cell(x, y),
                        &before[(x, y)],
                        "slot escaped{part:?} at{x},{y}"
                    );
                }
            }
        }
    }
}

#[test]
fn fetch_sentinel_uses_gutter_geometry_without_fabricating_number_or_key() {
    for rows in [0, 2] {
        let mut page = Page::new(rows);
        page.model.more = true;
        let mut h = Harness::new(page, Theme::junie(), 40, 6);
        let y = rows as u16 + 1;
        assert_ne!(h.cell(6, y).symbol(), " ");
        assert_eq!(h.cell(4, y).symbol(), " ", "sentinel has no source number");
        let _ = h.click(4, y);
        assert!(h.app().state.on_fetch_row());
        assert!(h.app().actions.contains(&junie_tui::GridAction::FetchMore));
    }
}
#[test]
fn compact_default_and_right_reserve_are_independent() {
    let mut page = Page::new(40);
    page.compact = true;
    page.reserve = 0;
    let mut h = Harness::new(page, Theme::junie(), 40, 6);
    assert_eq!(h.cell(2, 0).symbol(), "a");
    assert_eq!(h.cell(12, 0).symbol(), "b");
    let before = h.app().state.cursor();
    let _ = h.click(1, 1);
    assert_eq!(h.app().state.cursor(), before);
    assert!(h.app().state.selected_rows().is_empty());
    h.app_mut().reserve = 36;
    h.draw();
    assert!(!h.row(0).contains("alpha"));
    let _ = h.click(20, 1);
    assert_eq!(h.app().state.cursor(), before);
}
#[test]
fn tiny_detailed_geometry_clips_numbers_slots_and_columns() {
    for width in 0..=8 {
        let mut page = Page::new(100);
        page.area = Rect::new(2, 1, width, 4);
        page.slot = Some(Part::ROW_NUMBER);
        let h = Harness::new(page, Theme::junie(), 20, 6);
        for y in 0..6 {
            for x in 0..20 {
                if !(2..2 + width).contains(&x) || !(1..5).contains(&y) {
                    assert_eq!(h.cell(x, y).symbol(), " ", "escaped width{width} at{x},{y}");
                }
            }
        }
    }
}

#[test]
fn hidden_focus_glyph_and_fetch_gutter_preserve_source_cells_and_slots() {
    for level in [
        junie_tui::ColorLevel::TrueColor,
        junie_tui::ColorLevel::Ansi256,
        junie_tui::ColorLevel::Ansi16,
        junie_tui::ColorLevel::Mono,
    ] {
        let mut page = Page::new(2);
        page.model.more = true;
        let mut h = Harness::new(page, Theme::junie(), 40, 6).with_color(level);
        h.draw();
        assert_eq!(h.cell(0, 2).symbol(), "▎");
        assert_eq!(h.cell(0, 2).fg, h.cell(0, 2).bg);
        assert_eq!(h.cell(0, 3).symbol(), "▎");
        assert_eq!(h.cell(0, 3).fg, h.cell(0, 3).bg);
        let _ = h.key(KeyCode::Char('G'));
        assert_ne!(h.cell(0, 3).fg, h.cell(0, 3).bg);
        h.app_mut().slot = Some(Part::GUTTER);
        h.draw();
        assert_eq!(h.cell(0, 3).symbol(), "!");
        assert_eq!(h.cell(1, 3).symbol(), " ");
        h.app_mut().slot = Some(Part::ROW_NUMBER);
        h.draw();
        assert_eq!(h.cell(4, 3).symbol(), " ");
    }
}

#[test]
fn disabled_detailed_parts_receive_disabled_recipe_and_cannot_select() {
    let mut page = Page::new(40);
    page.disabled = true;
    let mut theme = Theme::junie();
    theme.color.danger = Color::Rgb(13, 17, 19);
    theme = theme.define_family(Family::GRID, |family| {
        for part in [Part::GUTTER, Part::MARKER, Part::CHANGE, Part::ROW_NUMBER] {
            family
                .part(part)
                .when(StateFlags::DISABLED, StylePatch::new().set_fg(Role::Danger));
        }
    });
    let mut h = Harness::new(page, theme, 40, 6);
    for x in [0, 1, 2, 4] {
        assert_eq!(h.cell(x, 1).fg, Color::Rgb(13, 17, 19));
    }
    let before = h.app().state.cursor();
    let _ = h.click(4, 2);
    assert_eq!(h.app().state.cursor(), before);
    assert!(h.app().state.selected_rows().is_empty());
    assert!(h.app().actions.is_empty());
}

#[test]
fn detailed_reserve_and_scrollbar_share_preview_pointer_geometry() {
    let mut page = Page::new(40);
    page.fit = junie_tui::GridColumnFit::CompleteWithPreview { min_width: 6 };
    page.area.width = 37;
    page.columns[2].min_width = 24;
    page.columns[2].max_width = 24;
    let mut h = Harness::new(page, Theme::junie(), 40, 6);
    // 37 - scrollbar 1 - gutter 6 - right reserve 4 = 26:
    // complete widths 8 + 2 + 8, then six-cell preview at x26.
    assert_eq!(h.cell(26, 1).symbol(), "c");
    let before = h.app().state.cursor();
    let _ = h.click(27, 1);
    assert_eq!(h.app().state.cursor(), before);
    let _ = h.click(17, 1);
    assert_eq!(
        h.app().state.cursor().map(|(_, col)| col),
        Some(ColumnKey::num(1))
    );
    h.app_mut().area.width = 40;
    h.draw();
    let _ = h.click(27, 1);
    assert_eq!(
        h.app().state.cursor().map(|(_, col)| col),
        Some(ColumnKey::num(1))
    );
    let _ = h.click(4, 2);
    assert!(h.app().state.selected_rows().contains(ItemKey::index(1)));
}

#[test]
fn rapid_repeated_gutter_clicks_each_toggle_the_source_row() {
    let mut h = Harness::new(Page::new(40), Theme::junie(), 40, 6);
    for selected in [true, false, true, false] {
        let _ = h.click(4, 2);
        assert_eq!(
            h.app().state.selected_rows().contains(ItemKey::index(1)),
            selected
        );
    }
}

#[test]
fn detailed_gutter_pointer_flags_follow_only_the_addressed_row() {
    let mut theme = Theme::junie();
    theme.color.info = Color::Rgb(13, 17, 19);
    theme.color.danger = Color::Rgb(23, 29, 31);
    theme = theme.define_family(Family::GRID, |family| {
        for part in [Part::GUTTER, Part::MARKER, Part::CHANGE, Part::ROW_NUMBER] {
            family
                .part(part)
                .when(StateFlags::HOVERED, StylePatch::new().set_fg(Role::Info))
                .when(
                    StateFlags::HOVERED | StateFlags::PRESSED,
                    StylePatch::new().set_fg(Role::Danger),
                );
        }
    });
    let mut h = Harness::new(Page::new(40), theme, 40, 6);
    let _ = h.mouse(junie_tui::MouseKind::Move, 4, 2);
    for x in [0, 1, 2, 4] {
        assert_eq!(h.cell(x, 2).fg, Color::Rgb(13, 17, 19));
        assert_ne!(h.cell(x, 3).fg, Color::Rgb(13, 17, 19));
    }
    let _ = h.mouse(junie_tui::MouseKind::Down, 4, 2);
    for x in [0, 1, 2, 4] {
        assert_eq!(h.cell(x, 2).fg, Color::Rgb(23, 29, 31));
    }
    let _ = h.mouse(junie_tui::MouseKind::Up, 4, 2);
    let _ = h.mouse(junie_tui::MouseKind::Move, 7, 3);
    for x in [0, 1, 2, 4] {
        assert_eq!(h.cell(x, 3).fg, Color::Rgb(13, 17, 19));
        assert_ne!(h.cell(x, 2).fg, Color::Rgb(13, 17, 19));
    }
}

#[test]
fn independent_detailed_number_receives_pointer_state_recipe() {
    let theme = Theme::junie().define_family(Family::GRID, |family| {
        family
            .part(Part::ROW_NUMBER)
            .when(StateFlags::HOVERED, StylePatch::new().set_fg(Role::Danger));
    });
    let expected = theme.color.danger;
    let mut h = Harness::new(Page::new(40), theme, 40, 6);
    let _ = h.mouse(junie_tui::MouseKind::Move, 4, 2);
    assert_eq!(h.hover(), Some(ID));
    assert_eq!(
        h.cell(4, 2).fg,
        expected,
        "new number hit must supply its own hovered state"
    );
}
