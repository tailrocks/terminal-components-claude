//! External contract tests for `RowUi` marker and cell-owning part glyphs.

use tui_next::{
    ColorLevel, Family, GlyphRole, Id, ItemKey, Part, Rect, RowUi, Slot, StateFlags, Theme, Track,
    Ui, Variant,
};
use tui_next_testing::Scene;

const OWNER: Id = Id::root("rowui.glyph.contract");
const ROW: Rect = Rect::new(2, 0, 12, 1);
const PART_WIDTH: u16 = 6;

fn theme_with_glyph(part: Part, glyph: Slot<GlyphRole>) -> Theme {
    let mut theme = Theme::junie();
    theme.recipes.get_mut(Family::LIST).parts.entry(part).glyph = glyph;
    theme
}

fn cell_symbol(scene: &Scene, x: u16) -> &str {
    scene
        .buffer()
        .cell((x, 0))
        .map_or("<missing>", |cell| cell.symbol())
}

fn draw_marker(glyph: Slot<GlyphRole>) -> Scene {
    let mut scene = Scene::new(
        "rowui_marker_glyph_contract",
        theme_with_glyph(Part::MARKER, glyph),
        ColorLevel::TrueColor,
        16,
        1,
    );
    scene.draw(|ui: &mut Ui<'_>, _| {
        let mut row = RowUi::new(
            ui,
            OWNER,
            Family::LIST,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::index(0),
            ROW,
        );
        row.marker(GlyphRole::CheckboxOn);
        let mut columns = row.columns(&[Track::Flex(1)]);
        columns.cell(0).text("after");
    });
    scene
}

#[test]
fn marker_slots_choose_the_glyph_and_reserve_marker_then_gap() {
    let cases = [
        (Slot::Inherit, ["[", "✓", "]"], 3, 1),
        (Slot::Set(GlyphRole::CheckboxOff), ["[", " ", "]"], 3, 1),
        (Slot::Clear, [" ", "", ""], 1, 1),
    ];

    for (slot, marker_cells, marker_width, gap_width) in cases {
        let scene = draw_marker(slot);
        let marker_x = ROW.x;
        let gap_x = marker_x.saturating_add(marker_width);
        let content_x = gap_x.saturating_add(gap_width);

        for (offset, expected) in marker_cells
            .into_iter()
            .enumerate()
            .take(usize::from(marker_width))
        {
            assert_eq!(
                cell_symbol(&scene, marker_x.saturating_add(offset as u16)),
                expected
            );
        }
        assert_eq!(cell_symbol(&scene, gap_x), " ");
        assert_eq!(cell_symbol(&scene, content_x), "a");
    }
}

fn draw_part(glyph: Slot<GlyphRole>) -> Scene {
    let mut scene = Scene::new(
        "rowui_part_glyph_contract",
        theme_with_glyph(Part::META, glyph),
        ColorLevel::TrueColor,
        18,
        1,
    );
    scene.draw(|ui: &mut Ui<'_>, _| {
        let mut row = RowUi::new(
            ui,
            OWNER,
            Family::LIST,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::index(0),
            ROW,
        );
        row.part(Part::META, PART_WIDTH).text("abcdef");
        let mut columns = row.columns(&[Track::Flex(1)]);
        columns.cell(0).text("left!");
    });
    scene
}

#[test]
fn cell_part_slots_reserve_their_suffix_and_preceding_gap() {
    let part_x = ROW.right().saturating_sub(PART_WIDTH);
    let gap_x = part_x.saturating_sub(1);

    let inherit = draw_part(Slot::Inherit);
    assert_eq!(cell_symbol(&inherit, ROW.x), "l");
    assert_eq!(cell_symbol(&inherit, gap_x), " ");
    assert_eq!(cell_symbol(&inherit, part_x), "a");
    assert_eq!(cell_symbol(&inherit, part_x.saturating_add(5)), "f");

    let set = draw_part(Slot::Set(GlyphRole::CheckboxOn));
    assert_eq!(cell_symbol(&set, ROW.x), "l");
    assert_eq!(cell_symbol(&set, gap_x), " ");
    assert_eq!(cell_symbol(&set, part_x), "a");
    assert_eq!(cell_symbol(&set, part_x.saturating_add(2)), "c");
    assert_eq!(cell_symbol(&set, part_x.saturating_add(3)), "[");
    assert_eq!(cell_symbol(&set, part_x.saturating_add(4)), "✓");
    assert_eq!(cell_symbol(&set, part_x.saturating_add(5)), "]");

    let clear = draw_part(Slot::Clear);
    assert_eq!(cell_symbol(&clear, ROW.x), "l");
    assert_eq!(cell_symbol(&clear, gap_x), " ");
    assert_eq!(cell_symbol(&clear, part_x), "a");
    assert_eq!(cell_symbol(&clear, part_x.saturating_add(4)), "e");
    assert_eq!(cell_symbol(&clear, part_x.saturating_add(5)), " ");
}
