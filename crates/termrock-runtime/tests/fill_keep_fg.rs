#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "focused render assertions"
)]

use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_core::{Id, Response};
use termrock_runtime::{App, Cx, LayerSpec, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, FgStep, Modifier, Role, StylePatch, Surface, Theme};

const MODAL: Id = Id::root("fill_keep_fg.modal");

struct Page {
    layer: bool,
    clipped: bool,
    background: Color,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.layer && cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(MODAL, LayerSpec::modal(MODAL));
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.layer {
            ui.layer(MODAL, |ui, area| {
                paint_equivalent_fills(ui, area, self.background, self.clipped)
            });
        } else {
            paint_equivalent_fills(ui, ui.full(), self.background, self.clipped);
        }
    }
}

fn paint_equivalent_fills(ui: &mut Ui<'_>, area: Rect, background: Color, clipped: bool) {
    let style = ui.paint_patch(
        &StylePatch::new()
            .set_fg(Role::Fg(FgStep::Primary))
            .set_bg(Role::Surface(Surface::Overlay)),
    );
    ui.fill(
        area,
        style.add_modifier(Modifier::UNDERLINED | Modifier::ITALIC),
    );

    let left = Rect::new(area.x, area.y, 4, 1);
    let right = Rect::new(area.x.saturating_add(6), area.y, 4, 1);
    if clipped {
        ui.with_area(Rect::new(left.x.saturating_add(1), left.y, 2, 1), |ui| {
            ui.fill_keep_fg(left, background);
        });
        ui.with_area(Rect::new(right.x.saturating_add(1), right.y, 2, 1), |ui| {
            ui.fill(right, termrock_theme::PaintStyle::new().bg(background));
        });
    } else {
        ui.fill_keep_fg(left, background);
        ui.fill(right, termrock_theme::PaintStyle::new().bg(background));
    }
    ui.dim_layer(area, 1);
}

#[test]
fn fill_keep_fg_preserves_semantic_ink_after_dimming_on_page_and_layer() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for background in [Color::Black, Color::Reset, Color::Rgb(12, 34, 56)] {
            for layer in [false, true] {
                for clipped in [false, true] {
                    let harness = Harness::new(
                        Page {
                            layer,
                            clipped,
                            background,
                        },
                        Theme::junie().for_level(level),
                        12,
                        1,
                    );
                    for offset in 0..4 {
                        if clipped && (offset == 0 || offset == 3) {
                            continue;
                        }
                        assert_eq!(
                            harness.cell(offset, 0),
                            harness.cell(offset.saturating_add(6), 0),
                            "level={level:?} bg={background:?} layer={layer} clipped={clipped} offset={offset}"
                        );
                    }
                }
            }
        }
    }
}
