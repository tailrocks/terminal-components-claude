#![cfg_attr(test, allow(clippy::expect_used, clippy::arithmetic_side_effects))]

use ratatui_core::layout::Rect;
use ratatui_core::style::Color;

use termrock_controls::{Panel, PanelKind};
use termrock_core::id::{Id, Part};
use termrock_layout::{Constraints, Size};
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Modifier, Role, StylePatch, Theme};

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
        let slot = |ui: &mut termrock_runtime::Ui<'_>, _: Rect| {
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

#[test]
fn body_aware_measurement_adds_shared_insets_to_live_body_size() {
    for kind in [PanelKind::Card, PanelKind::Framed] {
        let mut scene = Scene::new(
            "body_measure",
            Theme::junie(),
            ColorLevel::TrueColor,
            32,
            12,
        );
        scene.draw(|ui, _| {
            let panel = Panel::new(ID)
                .kind(kind)
                .title("Diagnostics")
                .meta("λ")
                .badge("界e\u{301}");
            let seen = std::cell::Cell::new(None);
            let body_size = Size {
                min: (8, 2),
                preferred: (12, 4),
            };
            let measured = panel.measure_with_body(ui, Constraints::loose(30, 10), |body| {
                seen.set(Some(body));
                body_size
            });
            let inner = panel.inner(ui, Rect::new(0, 0, 30, 10));
            let body = seen.get().expect("body constraints were supplied");
            assert_eq!(body.max, (inner.width, inner.height));
            let chrome_w = 30_u16.saturating_sub(inner.width);
            let chrome_h = 10_u16.saturating_sub(inner.height);
            assert_eq!(measured.min.0, body_size.min.0.saturating_add(chrome_w));
            assert_eq!(measured.min.1, body_size.min.1.saturating_add(chrome_h));
            let head_width = if kind == PanelKind::Card { 22 } else { 25 };
            assert_eq!(
                measured.preferred.0,
                body_size
                    .preferred
                    .0
                    .saturating_add(chrome_w)
                    .max(head_width)
            );
            assert_eq!(
                measured.preferred.1,
                body_size.preferred.1.saturating_add(chrome_h)
            );
            assert!(!body.tight_w && !body.tight_h);

            let legacy = panel.measure(ui, Constraints::loose(30, 10));
            let one_cell_body = panel
                .measure_with_body(ui, Constraints::loose(30, 10), |c| Size::exact(1, 1).fit(c));
            assert_eq!(
                one_cell_body, legacy,
                "default measurement stays compatible"
            );
        });
    }
}

#[test]
fn body_aware_measurement_saturates_zero_regions_and_keeps_outer_tightness() {
    let mut scene = Scene::new("body_zero", Theme::junie(), ColorLevel::TrueColor, 8, 4);
    scene.draw(|ui, _| {
        let panel = Panel::new(ID).kind(PanelKind::Framed).title("界");
        let seen = std::cell::Cell::new(None);
        let zero = panel.measure_with_body(ui, Constraints::loose(0, 0), |body| {
            seen.set(Some(body));
            Size::exact(40, 10)
        });
        assert_eq!(seen.get().expect("body measured").max, (0, 0));
        assert_eq!(zero, Size::exact(0, 0));

        let tight = panel.measure_with_body(ui, Constraints::tight(8, 4), |body| {
            let inner = panel.inner(ui, Rect::new(0, 0, 8, 4));
            assert_eq!(body.max, (inner.width, inner.height));
            assert!(body.tight_w && body.tight_h);
            Size::exact(body.max.0, body.max.1)
        });
        assert_eq!(tight, Size::exact(8, 4));
    });
}
