//! Junie-inspired Ratatui design-system laboratory.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_coverage;
mod data;
mod pages;

use crate::app::{App, Motion, PageId};
use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum};
use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

struct Options {
    level: ColorLevel,
    page: Option<PageId>,
    motion: Motion,
    frame: u64,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ColorArg {
    #[value(name = "truecolor", alias = "24bit")]
    TrueColor,
    #[value(name = "256")]
    Ansi256,
    #[value(name = "16")]
    Ansi16,
    #[value(name = "none", alias = "mono")]
    Mono,
}

impl From<ColorArg> for ColorLevel {
    fn from(value: ColorArg) -> Self {
        match value {
            ColorArg::TrueColor => Self::TrueColor,
            ColorArg::Ansi256 => Self::Ansi256,
            ColorArg::Ansi16 => Self::Ansi16,
            ColorArg::Mono => Self::Mono,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum MotionArg {
    Full,
    Reduced,
    Paused,
}

#[derive(Debug, Parser)]
#[command(
    name = "junie-tui",
    about = "Junie-inspired Ratatui design system laboratory",
    long_about = "Junie-tui — Junie-inspired Ratatui design system laboratory"
)]
struct Cli {
    #[arg(short = 'c', long, value_enum, value_name = "LEVEL")]
    color: Option<ColorArg>,
    #[arg(short = 'p', long, value_parser = parse_page, value_name = "NAME")]
    page: Option<PageId>,
    #[arg(short = 'm', long, value_enum)]
    motion: Option<MotionArg>,
    #[arg(short = 'f', long, default_value_t = 0)]
    frame: u64,
}

fn parse_page(value: &str) -> Result<PageId, String> {
    PageId::from_name(value).ok_or_else(|| format!("unknown page {value:?}"))
}

fn cli_after_help() -> String {
    "Motion:    --motion paused pins tick-derived surfaces at --frame N for capture\n\n\
     Keys: Tab/Shift+Tab focus · arrows move · Enter/Space activate · Esc back · [ ] pages · ? help · q quit"
        .to_string()
}

fn parse_args() -> Options {
    let cli = Cli::command().after_help(cli_after_help()).get_matches();
    let cli = Cli::from_arg_matches(&cli).unwrap_or_else(|error| error.exit());
    Options {
        level: cli
            .color
            .map(ColorLevel::from)
            .unwrap_or_else(ColorLevel::detect),
        page: cli.page,
        motion: match cli.motion.unwrap_or(MotionArg::Full) {
            MotionArg::Full | MotionArg::Reduced => Motion::Full,
            MotionArg::Paused => Motion::Paused,
        },
        frame: cli.frame,
    }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::with_motion(theme, opts.motion, opts.frame);
    if let Some(p) = opts.page {
        app.goto(p);
    }
    junie_tui::runtime::run(&mut app)
}

impl junie_tui::runtime::Application for App {
    fn handle(&mut self, input: Input) -> Outcome {
        App::handle(self, input)
    }
    fn render(&mut self, frame: &mut ratatui::Frame) {
        App::render(self, frame)
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
    fn tick_interval(&self) -> std::time::Duration {
        App::tick_interval(self)
    }
}
