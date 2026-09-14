//! TablePro, terminal edition: the core database workbench built on the
//! Junie-inspired design system.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_coverage;
mod connections;
mod db;
mod model;
mod sql;
mod tabs;
mod workbench;

use clap::{CommandFactory, Parser, ValueEnum, error::ErrorKind};
use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

use crate::app::App;

struct Options {
    level: ColorLevel,
    connect: Option<String>,
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

#[derive(Debug, Parser)]
#[command(name = "tablepro", about = "TablePro terminal database workbench")]
struct Cli {
    #[arg(short = 'c', long, value_enum, value_name = "LEVEL")]
    color: Option<ColorArg>,
    #[arg(long, value_name = "NAME")]
    connect: Option<String>,
}

fn parse_args() -> Options {
    let cli = Cli::parse();
    Options {
        level: cli
            .color
            .map(ColorLevel::from)
            .unwrap_or_else(ColorLevel::detect),
        connect: cli.connect,
    }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::new(theme);
    if let Some(name) = opts.connect {
        if let Some(i) = app
            .connections
            .connections
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(&name))
        {
            app.connect(i);
        } else {
            Cli::command()
                .error(
                    ErrorKind::ValueValidation,
                    format!("no connection named {name:?}"),
                )
                .exit();
        }
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
