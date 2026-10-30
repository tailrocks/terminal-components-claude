#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "focused public-rendering regression assertions"
)]

use std::cell::RefCell;
use std::rc::Rc;

use ratatui_core::style::Color;
use termrock_core::{Id, Response};
use termrock_runtime::{App, Cx, LayerSpec, Rect, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{FgStep, Role, StylePatch, Surface, Theme};

const OVERLAY: Id = Id::root("recolor_provenance.overlay");
const AREA: Rect = Rect::new(0, 0, 12, 2);

struct View {
    identity: bool,
    layer_probe: bool,
    changed_channel: Option<bool>,
}

impl App for View {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.layer_probe && cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(OVERLAY, LayerSpec::modal(OVERLAY));
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let style = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Field)),
        );
        ui.paint_str(Rect::new(1, 0, 1, 1), "R", style);
        if self.layer_probe {
            ui.layer(OVERLAY, |ui, area| {
                ui.recolor_pair(area, |fg, bg| (fg, bg));
            });
        } else if self.identity {
            ui.recolor_fg(AREA, |fg, _| Some(fg));
            ui.dim_layer(AREA, 1);
        } else if let Some(change_fg) = self.changed_channel {
            ui.recolor_pair(AREA, |fg, bg| {
                if change_fg {
                    (Color::Rgb(1, 2, 3), bg)
                } else {
                    (fg, Color::Rgb(4, 5, 6))
                }
            });
            ui.dim_layer(AREA, 1);
        } else {
            ui.dim_layer(AREA, 1);
        }
    }
}

#[test]
fn no_op_foreground_recolor_keeps_semantic_role_and_full_cell() {
    let expected = Harness::new(
        View {
            identity: false,
            layer_probe: false,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );
    let actual = Harness::new(
        View {
            identity: true,
            layer_probe: false,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );

    assert_eq!(actual.cell(1, 0), expected.cell(1, 0));
}

#[test]
fn identity_pair_recolor_keeps_unpainted_overlay_transparent() {
    let harness = Harness::new(
        View {
            identity: false,
            layer_probe: true,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );

    assert_eq!(harness.cell(1, 0).symbol(), "R");
}

#[test]
fn changing_one_pair_channel_keeps_other_semantic_role_through_dimming() {
    let baseline = Harness::new(
        View {
            identity: false,
            layer_probe: false,
            changed_channel: None,
        },
        Theme::junie(),
        12,
        2,
    );

    for change_fg in [true, false] {
        let changed = Harness::new(
            View {
                identity: false,
                layer_probe: false,
                changed_channel: Some(change_fg),
            },
            Theme::junie(),
            12,
            2,
        );
        let expected = baseline.cell(1, 0);
        let actual = changed.cell(1, 0);
        assert_eq!(actual.symbol(), expected.symbol());
        assert_eq!(actual.modifier, expected.modifier);
        if change_fg {
            assert_ne!(actual.fg, expected.fg);
            assert_eq!(actual.bg, expected.bg);
        } else {
            assert_eq!(actual.fg, expected.fg);
            assert_ne!(actual.bg, expected.bg);
        }
    }
}

#[test]
fn rgb_recolor_visits_only_rgb_channels_in_foreground_then_background_order() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let callback_calls = Rc::clone(&calls);
    let harness = Harness::new(
        RgbCallbackView {
            calls: callback_calls,
        },
        Theme::junie(),
        12,
        1,
    );

    assert_eq!(
        *calls.borrow(),
        vec![Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6)]
    );
    assert_eq!(harness.cell(1, 0).fg, Color::Rgb(7, 8, 9));
    assert_eq!(harness.cell(1, 0).bg, Color::Rgb(10, 11, 12));
}

struct RgbCallbackView {
    calls: Rc<RefCell<Vec<Color>>>,
}

impl App for RgbCallbackView {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let style = termrock_theme::PaintStyle::new()
            .fg(Color::Rgb(1, 2, 3))
            .bg(Color::Rgb(4, 5, 6));
        ui.paint_str(Rect::new(1, 0, 1, 1), "C", style);
        let calls = Rc::clone(&self.calls);
        ui.recolor_rgb(Rect::new(1, 0, 1, 1), move |color| {
            calls.borrow_mut().push(color);
            match color {
                Color::Rgb(1, 2, 3) => Color::Rgb(7, 8, 9),
                Color::Rgb(4, 5, 6) => Color::Rgb(10, 11, 12),
                other => other,
            }
        });
    }
}
