//! Explicit spinner spacing preserves measurement and authoritative painting.
use junie_tui::{ColorLevel, Constraints, Id, Rect, Spinner, Theme};
use junie_tui_testing::Scene;
#[test]
fn reference_gap_one_and_explicit_zero_match_measurement_and_cells() {
    for gap in [0_u16, 1, 4] {
        let mut scene = Scene::new("spinner_gap", Theme::junie(), ColorLevel::TrueColor, 20, 2);
        scene.draw(|ui, _| {
            let spinner = Spinner::new(Id::root("spinner")).label("Wait").gap(gap);
            assert_eq!(
                spinner.measure(ui, Constraints::loose(20, 2)).preferred,
                (5 + gap, 1)
            );
            assert_eq!(
                spinner.draw(ui, Rect::new(1, 0, 19, 2)),
                Rect::new(1, 0, 5 + gap, 1)
            );
        });
        assert_eq!(
            scene.buffer().cell((1, 0)).map(junie_tui::Cell::symbol),
            Some("⠋")
        );
        assert_eq!(
            scene
                .buffer()
                .cell((2 + gap, 0))
                .map(junie_tui::Cell::symbol),
            Some("W")
        );
    }
}
#[test]
fn omitted_gap_keeps_theme_default_and_empty_label_needs_no_gap() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [
            ColorLevel::TrueColor,
            ColorLevel::Ansi256,
            ColorLevel::Ansi16,
            ColorLevel::Mono,
        ] {
            let gap = theme.design.space.gap.max(1);
            let mut default = Scene::new("spinner_default", theme.clone(), level, 20, 1);
            let mut explicit = Scene::new("spinner_explicit", theme.clone(), level, 20, 1);
            default.draw(|ui, area| {
                Spinner::new(Id::root("spinner"))
                    .label("Wait")
                    .draw(ui, area);
            });
            explicit.draw(|ui, area| {
                Spinner::new(Id::root("spinner"))
                    .label("Wait")
                    .gap(gap)
                    .draw(ui, area);
            });
            assert_eq!(default.buffer(), explicit.buffer());
        }
    }
    let mut scene = Scene::new("spinner_empty", Theme::junie(), ColorLevel::TrueColor, 1, 1);
    scene.draw(|ui, area| {
        let s = Spinner::new(Id::root("spinner")).gap(9);
        assert_eq!(s.measure(ui, Constraints::loose(20, 1)).preferred, (1, 1));
        assert_eq!(s.draw(ui, area), area);
    });
}
