//! Command-line entry point for the deterministic Jackin Preview shell.

use junie_tui::{ColorLevel, Theme};

fn main() -> std::io::Result<()> {
    let mut scenario = jackin_app::Scenario::Returning;
    let mut motion =
        jackin_app::Motion::resolve(None, std::env::var_os("JACKIN_NO_MOTION").is_some());
    let mut frame = 0_u64;
    let mut theme = Theme::junie();
    let mut requested_color = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => {
                if let Some(value) = args.next()
                    && let Some(parsed) = jackin_app::Scenario::from_name(&value)
                {
                    scenario = parsed;
                }
            }
            "--motion" => {
                if let Some(value) = args.next()
                    && let Some(parsed) = jackin_app::Motion::from_name(&value)
                {
                    motion = parsed;
                }
            }
            "--frame" => {
                if let Some(value) = args.next()
                    && let Ok(parsed) = value.parse::<u64>()
                {
                    frame = parsed;
                }
            }
            "--theme" => {
                if let Some(value) = args.next()
                    && value.eq_ignore_ascii_case("paper")
                {
                    theme = Theme::paper();
                }
            }
            "--color" => {
                requested_color = args.next().and_then(|value| parse_color_level(&value));
            }
            _ => {}
        }
    }
    if let Some(level) = requested_color {
        theme = theme.for_level(level);
    }
    jackin_app::run_scenario_with_theme(scenario, motion, frame, theme)
}

fn parse_color_level(value: &str) -> Option<ColorLevel> {
    match value.to_ascii_lowercase().as_str() {
        "truecolor" | "24bit" => Some(ColorLevel::TrueColor),
        "256" | "ansi256" => Some(ColorLevel::Ansi256),
        "16" | "ansi16" => Some(ColorLevel::Ansi16),
        "none" | "mono" => Some(ColorLevel::Mono),
        _ => None,
    }
}
