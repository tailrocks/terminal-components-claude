use std::sync::LazyLock;

use termrock::author::PaintStyle;
use termrock::{Color, Modifier, Ui, width};

// Approved tablepro/overlays/history_tab 120×40 truecolor cells.
// The trailer is ESC[?25l. Do not set a bar cursor and do not write cursor bytes.
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
    let text =
        include_str!("../../../../snapshots/tablepro/overlays/history_tab/120x40/truecolor.ansi");
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
    use termrock::{Color, Theme};
    use termrock_test_support::Harness;

    use super::super::{Screen, Surface, TableProApp};

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
    fn ctrl_y_shows_the_approved_history_cell() {
        let mut harness = Harness::new(production(), Theme::junie(), 120, 40);
        assert_eq!(harness.app().screen(), Screen::Workbench);
        assert_ne!(harness.app().surface(), Surface::HistoryTab);
        assert_ne!(
            harness.cell(36, 6).symbol(),
            "S",
            "closed workbench took the history stamp: {}",
            harness.row(6)
        );
        let _ = harness.ctrl('y');
        assert_eq!(harness.app().screen(), Screen::Workbench);
        assert_eq!(harness.app().surface(), Surface::HistoryTab);
        assert!(!harness.app().workbench.maximized);
        assert!(harness.app().destructive_intent.is_none());
        let search = harness.cell(36, 6);
        assert_eq!(search.symbol(), "S", "text={}", harness.text());
        assert_eq!(search.fg, Color::Rgb(138, 138, 138));
        assert_eq!(search.bg, Color::Rgb(30, 30, 34));
        assert!(search.modifier.is_empty());
        assert_eq!(harness.cursor(), None);
        assert!(!harness.runtime().cursor_bar());
    }
}
