//! The author canvas paints exactly what the `Ui` paint surface paints.
//!
//! Every canvas primitive is checked against the `Ui` call it wraps, so a
//! component that moves from `ui.paint_str` to the canvas keeps its pixels.

use termrock::author::Color;
use termrock::author::{
    Position, Rect, put_cell, put_fill, put_frame, put_text, put_text_in, put_text_w,
};
use termrock::{App, Cx, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::{PaintStyle, Theme};

struct Page;

fn style(fg: Color) -> PaintStyle {
    PaintStyle::new().fg(fg).bg(Color::Black)
}

impl App for Page {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let text = "jackin❯";
        let cell = "▎";

        // Row 0: the canvas primitives.
        put_text(ui, 1, 0, text, style(Color::White));
        put_text_w(ui, 10, 0, 6, text, style(Color::White));
        put_text_in(ui, Rect::new(20, 0, 6, 1), text, style(Color::White));
        put_cell(ui, 30, 0, cell, style(Color::Green));
        put_fill(ui, 32, 0, 3, 1, style(Color::Red));
        put_frame(ui, 36, 0, 4, 1, style(Color::Gray));

        // Row 1: the `Ui` calls the canvas wraps.
        ui.paint_str(
            Rect::new(1, 1, text.chars().count() as u16, 1),
            text,
            style(Color::White),
        );
        ui.paint_str(Rect::new(10, 1, 6, 1), text, style(Color::White));
        ui.paint_str(Rect::new(20, 1, 6, 1), text, style(Color::White));
        ui.paint_cell(Position::new(30, 1), cell, style(Color::Green));
        ui.fill(Rect::new(32, 1, 3, 1), style(Color::Red));
        ui.frame(Rect::new(36, 1, 4, 1), style(Color::Gray));
    }
}

#[test]
fn canvas_primitives_paint_the_same_cells_as_the_ui_surface() {
    let mut harness = Harness::new(Page, Theme::junie(), 48, 2);
    harness.draw();

    for x in 0..48 {
        assert_eq!(
            harness.cell(x, 0),
            harness.cell(x, 1),
            "column {x} differs between the canvas and the Ui surface"
        );
    }
    assert!(harness.diagnostics().is_empty());
}
