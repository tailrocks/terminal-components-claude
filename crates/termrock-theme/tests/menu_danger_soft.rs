use ratatui_core::style::Color;
use termrock_runtime::Rect;
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Family, Part, StateFlags, Theme, Variant};

#[test]
fn rendered_menu_danger_row_uses_the_frozen_soft_ink_at_each_color_level() {
    for (level, expected) in [
        (ColorLevel::TrueColor, Color::Rgb(0xe8, 0xa0, 0xa0)),
        (ColorLevel::Ansi256, Color::Indexed(174)),
        (ColorLevel::Ansi16, Color::LightRed),
        (ColorLevel::Mono, Color::White),
    ] {
        let mut scene = Scene::new("menu-danger-soft", Theme::junie(), level, 1, 1);
        scene.draw(|ui, _| {
            let row = ui.style(
                Family::MENU,
                Variant::DANGER,
                Part::ROW,
                StateFlags::empty(),
            );
            ui.paint_str(Rect::new(0, 0, 1, 1), "X", row.style);
        });

        assert_eq!(
            scene.buffer().cell((0, 0)).map(|cell| cell.fg),
            Some(expected),
            "rendered menu danger row at {level:?}"
        );
    }
}
