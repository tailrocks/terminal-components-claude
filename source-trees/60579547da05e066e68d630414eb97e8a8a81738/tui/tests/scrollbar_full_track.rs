//! Full-height scrollbar geometry from pinned Holla scrollbar.rs.
use junie_tui::{ColorLevel, Id, Rect, ScrollRegion, ScrollState, Theme};
use junie_tui_testing::Scene;

#[test]
fn default_thumb_uses_full_track_including_endpoints_and_tiny_bounds() {
    for height in [1_u16, 2, 6, 10] {
        for end in [false, true] {
            let mut state = ScrollState::new(100);
            state.set_viewport(usize::from(height));
            if end {
                state.scroll_to(state.max_offset());
            }
            let mut scene = Scene::new("full_track", Theme::junie(), ColorLevel::TrueColor, 8, 12);
            let area = Rect::new(2, 1, 4, height);
            scene.draw(|ui, _| {
                ScrollRegion::new(Id::root("bar")).draw(ui, area, &state, 100);
            });
            let (start, len) = state.thumb(usize::from(height));
            for y in 0..height {
                let expected = if (start..start + len).contains(&usize::from(y)) {
                    "┃"
                } else {
                    "│"
                };
                assert_eq!(
                    scene.buffer().cell((5, y + 1)).map(junie_tui::Cell::symbol),
                    Some(expected),
                    "height={height} end={end} y={y}"
                );
            }
        }
    }
}

#[test]
fn custom_caps_are_decoration_outside_the_full_height_thumb() {
    let mut theme = Theme::junie();
    let mut set = theme.design.glyphs.scrollbar();
    set.begin = "^";
    set.end = "v";
    theme.design.glyphs.set_scrollbar(set);
    let mut state = ScrollState::new(100);
    state.set_viewport(6);
    state.scroll_to(47);
    let mut scene = Scene::new("custom_caps", theme, ColorLevel::TrueColor, 5, 6);
    scene.draw(|ui, area| {
        ScrollRegion::new(Id::root("bar")).draw(ui, area, &state, 100);
    });
    assert_eq!(
        scene.buffer().cell((4, 0)).map(junie_tui::Cell::symbol),
        Some("^")
    );
    assert_eq!(
        scene.buffer().cell((4, 5)).map(junie_tui::Cell::symbol),
        Some("v")
    );
}

struct Page {
    scroll: ScrollState,
}
impl junie_tui::App for Page {
    fn update(&mut self, cx: &mut junie_tui::Cx<'_>) -> junie_tui::Response<()> {
        ScrollRegion::new(Id::root("bar")).update(cx, &mut self.scroll, 100)
    }
    fn draw(&self, ui: &mut junie_tui::Ui<'_>) {
        ScrollRegion::new(Id::root("bar")).draw(ui, Rect::new(2, 2, 5, 10), &self.scroll, 100);
    }
}
#[test]
fn pointer_endpoints_and_thumb_drag_share_full_track_origin() {
    let mut h = junie_tui_testing::Harness::new(
        Page {
            scroll: ScrollState::new(100),
        },
        Theme::junie(),
        10,
        15,
    );
    let _ = h.click(6, 11);
    assert_eq!(h.app().scroll.offset(), 90);
    let _ = h.click(6, 2);
    assert_eq!(h.app().scroll.offset(), 0);
    let _ = h.drag((6, 2), (6, 11));
    assert_eq!(h.app().scroll.offset(), 90);
    let _ = h.drag((6, 11), (6, 2));
    assert_eq!(h.app().scroll.offset(), 0);
}

#[test]
fn explicit_track_replacements_bypass_typed_caps() {
    for clear in [false, true] {
        let mut theme = Theme::junie();
        let mut set = theme.design.glyphs.scrollbar();
        set.begin = "^";
        set.end = "v";
        theme.design.glyphs.set_scrollbar(set);
        theme
            .design
            .glyphs
            .set(junie_tui::GlyphRole::RuleQuiet, "~");
        let patch = if clear {
            junie_tui::StylePatch {
                glyph: junie_tui::Slot::Clear,
                ..junie_tui::StylePatch::new()
            }
        } else {
            junie_tui::StylePatch::new().set_glyph(junie_tui::GlyphRole::RuleQuiet)
        };
        let patches = [(junie_tui::Part::TRACK, patch)];
        let mut state = ScrollState::new(100);
        state.set_viewport(6);
        state.scroll_to(47);
        let mut scene = Scene::new("track_replacement", theme, ColorLevel::TrueColor, 5, 6);
        scene.draw(|ui, area| {
            ScrollRegion::new(Id::root("bar"))
                .patch_part(&patches)
                .draw(ui, area, &state, 100);
        });
        for y in [0, 5] {
            assert_eq!(
                scene.buffer().cell((4, y)).map(junie_tui::Cell::symbol),
                Some(if clear { " " } else { "~" })
            );
        }
    }
}
