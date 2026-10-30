use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock::author::PaintStyle;
use termrock::{App, Cx, Response, ScrollState, Theme, Ui};
use termrock_test_support::{Harness, perf};

#[global_allocator]
static ALLOCATOR: perf::Counting = perf::Counting;

struct Page(usize);

impl App for Page {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        for y in 0..4u16 {
            for x in 0..12u16 {
                let index = (usize::from(y) * 12 + usize::from(x)) % self.0;
                ui.paint_str(
                    Rect::new(x, y, 1, 1),
                    "x",
                    PaintStyle::new()
                        .fg(Color::Rgb(255, 255, 255))
                        .bg(Color::Rgb(index as u8, 0, 0)),
                );
            }
        }

        let mut state = ScrollState::default();
        state.set_content(100);
        state.set_viewport(4);
        ui.scroll_edges(ui.full(), &state);
    }
}

#[test]
fn warm_redraw_reuses_majority_count_storage() {
    for colors in [8usize, 9, 12, 48] {
        let mut harness = Harness::new(Page(colors), Theme::junie(), 12, 4);
        for _ in 0..20 {
            harness.draw();
        }

        let allocations_before = perf::allocs();
        let bytes_before = perf::bytes();
        for _ in 0..200 {
            harness.draw();
        }
        let allocations = perf::allocs() - allocations_before;
        let bytes = perf::bytes() - bytes_before;
        println!("ALLOCS colors={colors} frames=200 allocations={allocations} bytes={bytes}");
        assert_eq!(
            allocations, 0,
            "warmed redraw must reuse background-count storage for {colors} colors"
        );
        assert_eq!(
            bytes, 0,
            "warmed redraw must allocate no bytes for {colors} colors"
        );
    }
}
