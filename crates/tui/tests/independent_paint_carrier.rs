use junie_tui::theme::PaintStyle;
use junie_tui::{ColorLevel, FgStep, Modifier, Rect, Role, StylePatch, Surface, Theme};
use junie_tui_testing::Scene;
use ratatui_core::style::{Color, Style};
#[test]
fn independent_channel_copy_patch_and_raw_erasure() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let mut scene = Scene::new("independent_channels", Theme::junie(), level, 12, 1);
        scene.draw(|ui, area| {
            let ghost = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Ghost)));
            let semantic = ui.with_surface(Surface::Overlay, |ui| {
                ui.paint_patch(&StylePatch::new().set_bg(Role::HoverSurface))
            });
            let raw = Style::new().fg(ghost.fg.unwrap());
            let values = [
                ghost.patch(semantic),
                semantic.patch(ghost),
                PaintStyle::new().with_fg_from(ghost).with_bg_from(semantic),
                ghost
                    .add_modifier(Modifier::ITALIC)
                    .remove_modifier(Modifier::ITALIC),
                ghost.fg(ghost.fg.unwrap()),
                ghost.patch(raw),
                ghost.with_fg_from(PaintStyle::from(raw)),
                ghost.with_fg_from(PaintStyle::new()),
            ];
            for (x, st) in values.into_iter().enumerate() {
                ui.paint_str(Rect::new(x as u16, 0, 1, 1), "X", st);
            }
            ui.dim_layer(area, 1);
        });
        for x in 0..4 {
            assert_eq!(
                scene.buffer().cell((x, 0)).unwrap().symbol(),
                " ",
                "{level:?} semantic {x}"
            );
        }
        for x in 4..8 {
            assert_eq!(
                scene.buffer().cell((x, 0)).unwrap().symbol(),
                "X",
                "{level:?} raw/inherit {x}"
            );
        }
    }
}
#[test]
fn independent_right_alignment_moves_unpatched_ghost_origin() {
    use junie_tui::{Align, Family, Id, ItemKey, Part, RowUi, StateFlags, Variant};
    let mut scene = Scene::new(
        "independent_shift",
        Theme::junie(),
        ColorLevel::TrueColor,
        12,
        1,
    );
    scene.draw(|ui, area| {
        let ghost = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Ghost)));
        ui.fill(area, ghost);
        {
            let mut row = RowUi::new(
                ui,
                Id::root("r"),
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::HOVERED,
                ItemKey::index(0),
                area,
            );
            row.part(Part::META, 12).text("x界").align(Align::Right);
        }
        ui.dim_layer(area, 1);
    });
    // META resolves its own foreground, so compare transferred background origin
    // at both lead cells, plus cleared wide continuation.
    for x in [9, 10] {
        assert_eq!(
            scene.buffer().cell((x, 0)).unwrap().bg,
            Color::Rgb(24, 24, 27)
        );
    }
    assert_eq!(
        scene.buffer().cell((11, 0)).unwrap().bg,
        Color::Rgb(0, 0, 0)
    );
}
