use ratatui_core::style::{Color, Modifier, Style};
use termrock_runtime::{App, Cx, Rect, Response, Theme, Ui};
use termrock_test_support::Harness;
use termrock_test_support::perf::{Counting, allocs};

#[global_allocator]
static GLOBAL: Counting = Counting;

const SCREEN: Rect = Rect::new(0, 0, 8, 2);

struct Backdrop;

impl App for Backdrop {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.paint_cell(
            ratatui_core::layout::Position::new(0, 0),
            "x",
            Style::new()
                .fg(Color::White)
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        );
        ui.modal_backdrop(SCREEN);
    }
}

#[test]
fn warmed_modal_backdrop_does_not_allocate() {
    let mut runtime = Harness::new(Backdrop, Theme::junie(), SCREEN.width, SCREEN.height);
    runtime.draw();
    let before = allocs();
    runtime.draw();
    assert_eq!(allocs().checked_sub(before), Some(0));
}
