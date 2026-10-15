//! Independent owner-surface retention probe.
use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

use termrock_core::id::Part;
use termrock_runtime::EmptyState;
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Family, FgStep, Role, StylePatch, Surface, Theme};
#[test]
fn delayed_owner_surface_survives_child_reset_and_dimming() {
    let theme = Theme::junie().override_family(Family::EMPTY, |r| {
        r.part(Part::TITLE)
            .base(StylePatch::new().clear_fg().clear_bg());
    });
    let mut scene = Scene::new(
        "independent_empty_delayed",
        theme,
        ColorLevel::TrueColor,
        20,
        3,
    );
    scene.draw(|ui, area| {
        let owner = ui.with_surface(Surface::Overlay, |ui| {
            ui.paint_patch(
                &StylePatch::new()
                    .set_bg(Role::HoverSurface)
                    .set_fg(Role::Fg(FgStep::Ghost)),
            )
        });
        let _ = ui.with_surface(Surface::Canvas, |ui| {
            ui.paint_patch(&StylePatch::new().set_bg(Role::HoverSurface))
        });
        EmptyState::Empty {
            title: "T",
            hint: Some("H"),
        }
        .draw_inherited(ui, area, 0, owner);
        ui.dim_layer(Rect::new(0, 0, 20, 3), 1);
    });
    assert!(!scene.buffer().content().iter().any(|c| c.symbol() == "T"));
    assert!(
        scene
            .buffer()
            .content()
            .iter()
            .all(|c| c.bg == Color::Rgb(39, 39, 42))
    );
    assert!(scene.buffer().content().iter().any(|c| c.symbol() == "H"));
}
