//! Holla: a context-adaptive action launcher for the terminal, as a
//! deterministic preview on the Junie design system. Every external system
//! (mise, git, gh, docker, btm, pg_activity, ssh, the filesystem) is an
//! in-memory simulation; every operator interaction is real. No stack
//! command is ever spawned.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_coverage;
#[cfg(test)]
mod app_tests_flows;
#[cfg(test)]
mod app_tests_parity;
#[cfg(test)]
mod app_tests_proofs;
#[cfg(test)]
mod app_tests_rows;
mod clock;
mod domain;
mod scenario;
mod screens;
mod sim;

use clap::{Parser, ValueEnum};
use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

use crate::app::App;
use crate::scenario::{Motion, Scenario};

struct Options {
    level: ColorLevel,
    scenario: Scenario,
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
#[command(name = "holla", about = "Context-adaptive action launcher")]
struct Cli {
    #[arg(short = 'c', long, value_enum, value_name = "LEVEL")]
    color: Option<ColorArg>,
    #[arg(short = 's', long, value_parser = parse_scenario, default_value = "first-use")]
    scenario: Scenario,
    #[arg(short = 'm', long, value_enum)]
    motion: Option<MotionArg>,
    #[arg(short = 'f', long, default_value_t = 0)]
    frame: u64,
}

fn parse_scenario(value: &str) -> Result<Scenario, String> {
    Scenario::from_name(value).ok_or_else(|| {
        format!(
            "unknown scenario {value:?}; use one of {}",
            Scenario::ALL
                .iter()
                .map(|s| s.name())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

fn parse_args() -> Options {
    let cli = Cli::parse();
    let no_motion = std::env::var_os("HOLLA_NO_MOTION").is_some_and(|v| !v.is_empty() && v != "0");
    Options {
        level: cli
            .color
            .map(ColorLevel::from)
            .unwrap_or_else(ColorLevel::detect),
        scenario: cli.scenario,
        motion: Motion::resolve(cli.motion.map(Motion::from), no_motion),
        frame: cli.frame,
    }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::for_scenario(opts.scenario, opts.motion, opts.frame, theme);
    if std::env::var_os("HOLLA_NO_HISTORY").is_some_and(|v| v == "1") {
        // the opt-out: nothing is read, learned or written
        app.world.memory.usage = crate::domain::usage::UsageStore::disabled();
    }
    let _ = junie_tui::runtime::drain_pending_input();
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
