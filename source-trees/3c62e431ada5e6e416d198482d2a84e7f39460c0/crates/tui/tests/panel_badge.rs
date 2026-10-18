//! Public Panel badge contract: cells, isolation, geometry and clipping.
#![cfg_attr(test, allow(clippy::expect_used, clippy::arithmetic_side_effects))]

use junie_tui::{
    Color, ColorLevel, Constraints, Id, Modifier, Panel, PanelKind, Part, Rect, Role, StylePatch,
    Theme,
};
use junie_tui_testing::Scene;

const ID: Id = Id::root("panel.badge.public");

#[test]
fn badge_override_changes_only_its_padded_cells() {
    let patch = [(
        Part::BADGE,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Rgb(1, 2, 3)))
            .set_bg(Role::Custom(Color::Rgb(4, 5, 6)))
            .add(Modifier::BOLD),
    )];
    for kind in [PanelKind::Card, PanelKind::Framed] {
        let render = |patched| {
            let mut scene = Scene::new("badge", Theme::junie(), ColorLevel::TrueColor, 40, 5);
            scene.draw(|ui, area| {
                let panel = Panel::new(ID)
                    .kind(kind)
                    .title("Title")
                    .meta("M")
                    .badge("EDIT");
                if patched {
                    panel.patch_part(&patch).draw(ui, area, |_, _| {});
                } else {
                    panel.draw(ui, area, |_, _| {});
                }
            });
            scene
        };
        let plain = render(false);
        let patched = render(true);
        let mut changed = 0;
        for pos in Rect::new(0, 0, 40, 5).positions() {
            let a = plain.buffer().cell(pos).expect("inside frame");
            let b = patched.buffer().cell(pos).expect("inside frame");
            if a != b {
                changed += 1;
                assert_eq!(pos.y, 0);
                assert_eq!(b.fg, Color::Rgb(1, 2, 3));
                assert_eq!(b.bg, Color::Rgb(4, 5, 6));
                assert!(b.modifier.contains(Modifier::BOLD));
            }
        }
        assert_eq!(changed, 6, "four letters and two padded cells");
        assert!(patched.text().contains(" EDIT "));
    }
}

#[test]
fn badge_slot_is_clipped_to_badge_and_preserves_body_and_metadata() {
    let mut scene = Scene::new("badge_slot", Theme::junie(), ColorLevel::TrueColor, 40, 5);
    scene.draw(|ui, area| {
        let slot = |ui: &mut junie_tui::Ui<'_>, _: Rect| {
            for row in area.rows() {
                ui.paint_str(
                    row,
                    "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
                    ui.surface_style(),
                );
            }
        };
        Panel::new(ID)
            .title("Title")
            .meta("META")
            .badge("EDIT")
            .slot(Part::BADGE, &slot)
            .draw(ui, area, |ui, body| {
                ui.paint_str(body, "Body", ui.surface_style());
            });
    });
    let text = scene.text();
    assert_eq!(text.matches('Z').count(), 6);
    assert!(text.contains("Title") && text.contains("META") && text.contains("Body"));
}

#[test]
fn preferred_width_fits_badge_and_narrow_width_hides_it_without_clipping_neighbors() {
    for kind in [PanelKind::Card, PanelKind::Framed] {
        let mut scene = Scene::new(
            "badge_measure",
            Theme::junie(),
            ColorLevel::TrueColor,
            40,
            8,
        );
        scene.draw(|ui, _| {
            let panel = Panel::new(ID)
                .kind(kind)
                .title("Title")
                .meta("META")
                .badge("界e\u{301}");
            let size = panel.measure(ui, Constraints::loose(40, 8));
            assert_eq!(
                size.preferred.0,
                if kind == PanelKind::Card { 19 } else { 22 }
            );
            panel.draw(ui, Rect::new(3, 1, size.preferred.0, 3), |_, _| {});
            panel.draw(ui, Rect::new(3, 5, size.preferred.0 - 1, 3), |_, _| {});
        });
        let text = scene.text();
        assert_eq!(
            text.matches("界e\u{301}").count(),
            1,
            "badge should fit exactly once: {text}"
        );
        for y in [1, 5] {
            assert_eq!(
                scene.buffer().cell((2, y)).expect("outside panel").symbol(),
                " "
            );
        }
    }
}

#[test]
fn empty_badge_is_identical_to_no_badge_at_every_small_extent() {
    for kind in [PanelKind::Card, PanelKind::Framed] {
        for width in 0..=16 {
            for height in 0..=4 {
                let render = |empty| {
                    let mut scene =
                        Scene::new("empty_badge", Theme::junie(), ColorLevel::TrueColor, 22, 7);
                    scene.draw(|ui, _| {
                        let panel = Panel::new(ID).kind(kind).title("Title").meta("M");
                        let area = Rect::new(3, 2, width, height);
                        if empty {
                            panel.badge("").draw(ui, area, |_, _| {});
                        } else {
                            panel.draw(ui, area, |_, _| {});
                        }
                    });
                    scene
                };
                assert_eq!(render(false).buffer(), render(true).buffer());
            }
        }
    }
}
