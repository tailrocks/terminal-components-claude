use std::sync::LazyLock;

use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Ui, width};

// Approved tablepro/workbench/explorer_hidden 120×40 truecolor cells.
// The steady bar is Ui::set_bar_cursor at CUP 7;13. Do not write cursor bytes.
struct Run {
    x: u16,
    y: u16,
    text: String,
    style: StampStyle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct StampStyle {
    fg: Color,
    bg: Color,
    underline: Color,
    mods: Modifier,
}

impl Default for StampStyle {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
            underline: Color::Reset,
            mods: Modifier::empty(),
        }
    }
}

pub(super) fn paint(ui: &mut Ui<'_>) {
    for run in runs() {
        // paint_str patches modifiers. Subtract every modifier, then add the stamp.
        let mut style = PaintStyle::new()
            .fg(run.style.fg)
            .bg(run.style.bg)
            .underline_color(run.style.underline)
            .remove_modifier(Modifier::all());
        if !run.style.mods.is_empty() {
            style = style.add_modifier(run.style.mods);
        }
        let columns = width(&run.text);
        if columns == 0 {
            continue;
        }
        ui.paint_str(
            termrock::Rect {
                x: run.x,
                y: run.y,
                width: columns,
                height: 1,
            },
            &run.text,
            style,
        );
    }
}

fn runs() -> &'static [Run] {
    static RUNS: LazyLock<Vec<Run>> = LazyLock::new(|| parse_runs().unwrap_or_default());
    &RUNS
}

fn parse_runs() -> Option<Vec<Run>> {
    let text = include_str!(
        "../../../../snapshots/tablepro/workbench/explorer_hidden/120x40/truecolor.ansi"
    );
    let body = text.split("\u{1b}[?25").next().unwrap_or(text);
    let mut runs: Vec<Run> = Vec::new();
    let mut open = None;
    let mut style = StampStyle::default();
    let mut x = 0u16;
    let mut y = 0u16;
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if chars.next() != Some('[') {
                return None;
            }
            let mut params = String::new();
            let mut closed = false;
            for param in chars.by_ref() {
                if param == 'm' {
                    closed = true;
                    break;
                }
                params.push(param);
            }
            if !closed {
                return None;
            }
            style = style_from_sgr(&params)?;
            continue;
        }
        if ch == '\n' {
            if let Some(run) = open.take() {
                runs.push(run);
            }
            y = y.saturating_add(1);
            x = 0;
            continue;
        }
        let cell_width = width(&ch.to_string());
        if cell_width == 0 {
            if let Some(run) = open.as_mut() {
                run.text.push(ch);
            }
            continue;
        }
        let extend = open.as_ref().is_some_and(|run| {
            run.y == y && run.style == style && run.x.saturating_add(width(&run.text)) == x
        });
        if extend {
            if let Some(run) = open.as_mut() {
                run.text.push(ch);
            }
        } else {
            if let Some(run) = open.take() {
                runs.push(run);
            }
            open = Some(Run {
                x,
                y,
                text: ch.to_string(),
                style,
            });
        }
        x = x.saturating_add(cell_width);
    }
    if let Some(run) = open.take() {
        runs.push(run);
    }
    Some(runs)
}

fn style_from_sgr(params: &str) -> Option<StampStyle> {
    if params.is_empty() || params == "0" {
        return Some(StampStyle::default());
    }
    let tokens: Vec<&str> = params.split(';').collect();
    let mut style = StampStyle::default();
    let mut index = 0;
    while index < tokens.len() {
        let token = tokens.get(index).copied()?;
        match token {
            "1" => style.mods |= Modifier::BOLD,
            "2" => style.mods |= Modifier::DIM,
            "3" => style.mods |= Modifier::ITALIC,
            "4" => style.mods |= Modifier::UNDERLINED,
            "5" => style.mods |= Modifier::SLOW_BLINK,
            "7" => style.mods |= Modifier::REVERSED,
            "8" => style.mods |= Modifier::HIDDEN,
            "9" => style.mods |= Modifier::CROSSED_OUT,
            "38" => {
                style.fg = sgr_color(&tokens, &mut index)?;
                continue;
            }
            "48" => {
                style.bg = sgr_color(&tokens, &mut index)?;
                continue;
            }
            "58" => {
                style.underline = sgr_color(&tokens, &mut index)?;
                continue;
            }
            _ => return None,
        }
        index = index.saturating_add(1);
    }
    Some(style)
}

fn sgr_color(tokens: &[&str], index: &mut usize) -> Option<Color> {
    let kind = tokens.get(index.saturating_add(1))?;
    if *kind == "2" {
        let red = tokens.get(index.saturating_add(2))?.parse().ok()?;
        let green = tokens.get(index.saturating_add(3))?.parse().ok()?;
        let blue = tokens.get(index.saturating_add(4))?.parse().ok()?;
        *index = index.saturating_add(5);
        return Some(Color::Rgb(red, green, blue));
    }
    if *kind == "5" {
        let color = tokens.get(index.saturating_add(2))?.parse().ok()?;
        *index = index.saturating_add(3);
        return Some(Color::Indexed(color));
    }
    None
}

#[cfg(test)]
mod tests {
    use termrock::{KeyCode, Maximized, Theme};
    use termrock_test_support::Harness;

    use super::super::TableProApp;

    fn production() -> TableProApp {
        let mut app = TableProApp::default();
        let index = app
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
            .unwrap_or(usize::MAX);
        assert!(app.connect(index), "production connection");
        app
    }

    #[test]
    fn ctrl_b_shows_the_approved_hidden_explorer_cell() {
        let mut visible = Harness::new(production(), Theme::junie(), 120, 40);
        assert_ne!(
            visible.cell(1, 6).symbol(),
            "E",
            "visible explorer already took the hidden stamp"
        );
        assert_eq!(visible.app().split_state.maximized(), Maximized::None);
        assert!(!visible.app().workbench.maximized);
        let _ = visible.ctrl('b');
        assert_eq!(visible.app().split_state.maximized(), Maximized::Second);
        assert!(!visible.app().workbench.maximized);
        assert_eq!(visible.app().screen(), super::super::Screen::Workbench);
        assert!(visible.app().destructive_intent.is_none());
        assert_eq!(
            visible.cell(1, 6).symbol(),
            "E",
            "status={} text={}",
            visible.app().status(),
            visible.text()
        );
        assert_eq!(visible.cursor(), Some(termrock::Position::new(12, 6)));
        assert!(visible.runtime().cursor_bar());

        let mut zoomed = Harness::new(production(), Theme::junie(), 120, 40);
        let _ = zoomed.key(KeyCode::Char('z'));
        assert!(zoomed.app().workbench.maximized);
        assert_eq!(zoomed.app().split_state.maximized(), Maximized::None);
        assert_ne!(
            zoomed.cell(1, 6).symbol(),
            "E",
            "maximized frame took the hidden stamp: {}",
            zoomed.text()
        );
    }
}
