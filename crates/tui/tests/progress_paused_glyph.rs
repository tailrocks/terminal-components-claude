//! The semantic pause glyph matches pinned Holla and remains customizable.
use junie_tui::{ColorLevel, GlyphRole, Id, ProgressBar, Theme};
use junie_tui_testing::Scene;
#[test]
fn paused_bar_paints_reference_double_vertical_line_in_all_modes() {
    for theme in [
        Theme::junie(),
        Theme::paper(),
        Theme::from_tokens(Theme::junie().color),
    ] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let mut scene = Scene::new("paused", theme.clone(), level, 30, 1);
            scene.draw(|ui, area| {
                ProgressBar::new(Id::root("bar"))
                    .ratio(0.3)
                    .icon(GlyphRole::ProgressPaused)
                    .draw(ui, area);
            });
            assert!(scene.buffer().content.iter().any(|c| c.symbol() == "‖"));
            assert!(!scene.buffer().content.iter().any(|c| c.symbol() == "∥"));
        }
    }
}
#[test]
fn custom_pause_and_ascii_policy_keep_their_bindings() {
    for theme in [
        Theme::junie()
            .builder()
            .glyph(GlyphRole::ProgressPaused, "P")
            .build(),
        Theme::junie().builder().ascii_glyphs().build(),
    ] {
        let expected = theme.design.glyphs.get(GlyphRole::ProgressPaused);
        let mut scene = Scene::new("pause_override", theme, ColorLevel::TrueColor, 30, 1);
        scene.draw(|ui, area| {
            ProgressBar::new(Id::root("bar"))
                .ratio(0.3)
                .icon(GlyphRole::ProgressPaused)
                .draw(ui, area);
        });
        assert!(
            scene
                .buffer()
                .content
                .iter()
                .any(|c| c.symbol() == expected)
        );
    }
}
