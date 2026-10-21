use ratatui_core::style::{Color, Modifier, Style};
use termrock_runtime::{App, Cx, Rect, Response, Theme, Ui};
use termrock_test_support::Harness;
use termrock_test_support::perf::{Counting, allocs};

#[global_allocator]
static GLOBAL: Counting = Counting;

const SCREEN: Rect = Rect::new(0, 0, 8, 2);

struct Backdrop;

struct StyledBackdrop {
    style: Style,
    blank_style: Option<Style>,
    second_style: Option<Style>,
}

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

impl App for StyledBackdrop {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.paint_cell(ratatui_core::layout::Position::new(0, 0), "x", self.style);
        if let Some(style) = self.blank_style {
            ui.paint_cell(ratatui_core::layout::Position::new(1, 0), " ", style);
        }
        if let Some(style) = self.second_style {
            ui.paint_cell(ratatui_core::layout::Position::new(2, 0), "y", style);
        }
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

#[test]
fn reduced_modal_cells_use_muted_ink_and_replace_modifiers() {
    use termrock_runtime::ColorLevel;

    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let mut theme = Theme::junie();
        theme.capability.color = level;
        theme.color.surfaces[0] = Color::Rgb(0, 0, 0);
        theme.color.accent = Color::Rgb(255, 255, 0);
        theme.color.fg[2] = Color::Rgb(150, 150, 150);
        theme.color.fg[3] = Color::Rgb(0, 255, 255);
        let reduced = matches!(level, ColorLevel::Ansi16 | ColorLevel::Mono);
        let app = StyledBackdrop {
            style: Style::new()
                .fg(theme.color.accent)
                .bg(theme.color.surfaces[0])
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            blank_style: Some(
                Style::new()
                    .fg(Color::Black)
                    .bg(Color::Black)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            ),
            second_style: None,
        };
        let runtime = Harness::new(app, theme.clone(), SCREEN.width, SCREEN.height);
        let cell = runtime.cell(0, 0);
        if reduced {
            assert_eq!(cell.fg, theme.color.fg[2], "{level:?}");
            assert_eq!(cell.modifier, Modifier::DIM, "{level:?}");
        } else {
            let expected = theme.backdrop_style(
                Style::new()
                    .fg(theme.color.accent)
                    .bg(theme.color.surfaces[0])
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            );
            assert_eq!(cell.fg, expected.fg.unwrap(), "{level:?}");
            assert_eq!(cell.modifier, expected.add_modifier, "{level:?}");
        }
        assert_eq!(cell.symbol(), "x", "{level:?}");
        let blank = runtime.cell(1, 0);
        assert_eq!(blank.symbol(), " ", "{level:?}");
        assert_eq!(blank.modifier, Modifier::empty(), "{level:?}");
    }
}

#[test]
fn modal_mapping_collision_retains_dim_after_pair_fallback() {
    use termrock_runtime::ColorLevel;

    for level in [ColorLevel::TrueColor, ColorLevel::Ansi256] {
        let mut theme = Theme::junie();
        theme.capability.color = level;
        theme.color.surfaces[0] = Color::Rgb(0, 0, 0);
        theme.color.fg[4] = theme.color.surfaces[0];
        let app = StyledBackdrop {
            style: Style::new()
                .fg(Color::Rgb(1, 2, 3))
                .bg(theme.color.surfaces[0])
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            blank_style: None,
            second_style: None,
        };
        let runtime = Harness::new(app, theme.clone(), SCREEN.width, SCREEN.height);
        let cell = runtime.cell(0, 0);
        assert_eq!(cell.fg, theme.color.fg[2], "{level:?}");
        assert_eq!(cell.bg, theme.color.surfaces[0], "{level:?}");
        assert_eq!(cell.modifier, Modifier::DIM, "{level:?}");
        assert_eq!(cell.symbol(), "x", "{level:?}");
    }
}

#[test]
fn actual_modal_preserves_explicit_reset_as_overlay_background() {
    let theme = Theme::junie();
    let app = StyledBackdrop {
        style: Style::new().fg(Color::White).bg(Color::Reset),
        blank_style: None,
        second_style: Some(Style::new().fg(Color::White)),
    };
    let runtime = Harness::new(app, theme.clone(), SCREEN.width, SCREEN.height);
    assert_eq!(runtime.cell(0, 0).symbol(), "x");
    assert_eq!(runtime.cell(0, 0).bg, theme.color.surfaces[3]);
    assert_eq!(runtime.cell(2, 0).symbol(), "y");
    assert_eq!(runtime.cell(2, 0).bg, theme.color.surfaces[3]);
}
