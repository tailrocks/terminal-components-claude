//! TablePro, terminal edition: the core database workbench built on the
//! Junie-inspired design system.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_context;
#[cfg(test)]
mod app_tests_copy;
#[cfg(test)]
mod app_tests_coverage;
#[cfg(test)]
mod app_tests_fields;
#[cfg(test)]
mod app_tests_mouse;
#[cfg(test)]
mod app_tests_splits;
mod connections;
mod context;
mod db;
mod model;
#[cfg(test)]
mod motion_tests;
#[cfg(test)]
mod perf_tests;
mod sql;
mod tabs;
mod workbench;

use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum, error::ErrorKind};
use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

use crate::app::{App, Motion};
use crate::db::Connection;

struct Options {
    level: ColorLevel,
    connect: Option<String>,
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

impl From<MotionArg> for Motion {
    fn from(value: MotionArg) -> Self {
        match value {
            MotionArg::Full => Self::Full,
            MotionArg::Reduced => Self::Reduced,
            MotionArg::Paused => Self::Paused,
        }
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "tablepro",
    about = "TablePro terminal database workbench",
    long_about = "TablePro — terminal database workbench on the Junie design system"
)]
struct Cli {
    #[arg(short = 'c', long, value_enum, value_name = "LEVEL")]
    color: Option<ColorArg>,
    #[arg(long, value_name = "NAME")]
    connect: Option<String>,
    #[arg(short = 'm', long, value_enum)]
    motion: Option<MotionArg>,
    #[arg(short = 'f', long, default_value_t = 0)]
    frame: u64,
}

fn cli_after_help() -> String {
    "Motion:    --motion paused pins tick-derived surfaces at --frame N for capture\n\n\
     Keys: Ctrl+O open quickly · Ctrl+T new query · Ctrl+R run · Ctrl+Y history · ? help · q quit"
        .to_string()
}

fn parse_args() -> Options {
    let cli = Cli::command().after_help(cli_after_help()).get_matches();
    let cli = Cli::from_arg_matches(&cli).unwrap_or_else(|error| error.exit());
    options(cli)
}

fn options(cli: Cli) -> Options {
    Options {
        level: cli
            .color
            .map(ColorLevel::from)
            .unwrap_or_else(ColorLevel::detect),
        connect: cli.connect,
        motion: cli.motion.map(Motion::from).unwrap_or_default(),
        frame: cli.frame,
    }
}

fn connection_index(connections: &[Connection], name: &str) -> Result<usize, clap::Error> {
    connections
        .iter()
        .position(|c| c.name.eq_ignore_ascii_case(name))
        .ok_or_else(|| {
            Cli::command().error(
                ErrorKind::ValueValidation,
                format!("no connection named {name:?}"),
            )
        })
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::new(theme);
    if let Some(name) = opts.connect {
        let i = connection_index(&app.connections.connections, &name)
            .unwrap_or_else(|error| error.exit());
        app.connect(i);
    }
    app.set_motion(opts.motion, opts.frame);
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
