use ratatui_core::style::Color;
use termrock::author::PaintStyle;
use termrock::{App, Cx, Rect, Response, ScrollState, Theme, Ui};
use termrock_test_support::Harness;

struct ScrollPage {
    width: u16,
    height: u16,
    backgrounds: Vec<Color>,
    edge_area: Option<Rect>,
}

impl App for ScrollPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = usize::from(y) * usize::from(self.width) + usize::from(x);
                ui.paint_str(
                    Rect::new(x, y, 1, 1),
                    "x",
                    PaintStyle::new()
                        .fg(Color::Rgb(255, 255, 255))
                        .bg(self.backgrounds[index])
                        .add_modifier(ratatui_core::style::Modifier::UNDERLINED),
                );
            }
        }

        let mut state = ScrollState::default();
        state.set_content(10);
        state.set_viewport(usize::from(self.height));
        ui.scroll_edges(self.edge_area.unwrap_or_else(|| ui.full()), &state);
    }
}

fn render(
    width: u16,
    height: u16,
    backgrounds: Vec<Color>,
    edge_area: Option<Rect>,
) -> Harness<ScrollPage> {
    Harness::new(
        ScrollPage {
            width,
            height,
            backgrounds,
            edge_area,
        },
        Theme::junie(),
        width.max(1),
        height.max(1),
    )
}

#[test]
fn ninth_distinct_background_can_win_scroll_edge_majority() {
    let mut backgrounds = (1..=8)
        .map(|index| Color::Rgb(index, 0, 0))
        .collect::<Vec<_>>();
    backgrounds.extend(std::iter::repeat_n(Color::Rgb(0, 0, 0), 32));

    let harness = render(10, 4, backgrounds, None);
    let edge = harness.cell(5, 3);
    assert_ne!(
        edge.fg,
        Color::Rgb(255, 255, 255),
        "the 32-cell ninth-color majority must receive overflow fading; bg={:?}, modifier={:?}",
        edge.bg,
        edge.modifier
    );
    assert_eq!(edge.bg, Color::Rgb(0, 0, 0));
    assert_eq!(edge.symbol(), "x");
    assert!(
        edge.modifier
            .contains(ratatui_core::style::Modifier::UNDERLINED)
    );
}

#[test]
fn mixed_reset_index_and_rgb_tie_uses_last_distinct_background() {
    let mut backgrounds = (1..=8)
        .map(|index| Color::Rgb(index, 10, 20))
        .collect::<Vec<_>>();
    backgrounds.extend(std::iter::repeat_n(Color::Reset, 14));
    backgrounds.extend(std::iter::repeat_n(Color::Indexed(42), 14));
    backgrounds.extend([
        Color::Reset,
        Color::Indexed(42),
        Color::Reset,
        Color::Indexed(42),
    ]);

    let harness = render(10, 4, backgrounds, None);
    let reset = harness.cell(8, 3);
    let indexed = harness.cell(9, 3);
    assert_eq!(reset.bg, Color::Reset);
    assert_eq!(indexed.bg, Color::Indexed(42));
    assert!(
        !reset.modifier.contains(ratatui_core::style::Modifier::DIM),
        "the earlier tied background must remain untouched"
    );
    assert!(
        indexed
            .modifier
            .contains(ratatui_core::style::Modifier::DIM),
        "equal counts resolve to the last distinct background in row-major order"
    );
}

#[test]
fn one_column_and_empty_scroll_areas_are_safe() {
    let backgrounds = vec![Color::Reset, Color::Indexed(42), Color::Reset, Color::Reset];
    let harness = render(1, 4, backgrounds, Some(Rect::new(0, 0, 1, 4)));
    assert!(
        harness
            .cell(0, 3)
            .modifier
            .contains(ratatui_core::style::Modifier::DIM)
    );

    let empty = render(1, 4, vec![Color::Reset; 4], Some(Rect::new(0, 0, 0, 4)));
    assert!(
        !empty
            .cell(0, 3)
            .modifier
            .contains(ratatui_core::style::Modifier::DIM),
        "an empty area leaves cells unchanged"
    );
}
