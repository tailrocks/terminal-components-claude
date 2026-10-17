//! Holla UI and terminal application.

pub mod app;
pub mod screens;
pub mod tui;

pub mod domain {
    pub use holla_domain::*;
    pub use holla_sim::fixtures;
}
pub mod catalog {
    pub use holla_catalog::*;
}
pub mod plan {
    pub use holla_plan::*;
}
pub mod sim {
    pub use holla_sim::*;
}
pub mod scenario {
    pub use holla_domain::scenario::*;
}
pub mod clock {
    pub use holla_domain::clock::*;
}

pub use app::App;
pub use holla_domain::scenario::{Motion, Scenario};

use clap::{CommandFactory, FromArgMatches, Parser, ValueEnum};

struct Options {
    level: crate::tui::theme::ColorLevel,
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

impl From<ColorArg> for crate::tui::theme::ColorLevel {
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
    name = "holla",
    about = "Context-adaptive action launcher",
    long_about = "Holla — this folder, this host, right now (deterministic preview on the Junie design system)"
)]
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

fn scenario_names(scenarios: &[Scenario]) -> String {
    scenarios
        .iter()
        .map(|s| s.name())
        .collect::<Vec<_>>()
        .join(", ")
}

fn cli_after_help() -> String {
    format!(
        "Scenarios: {concept}\n\
         Parity:    {parity}\n\
         Motion:    explicit --motion wins; otherwise HOLLA_NO_MOTION=1 selects reduced motion\n\
         Frame:     with --motion paused, the exact fixture tick to render\n\n\
         Keys: type to search · ↑↓ move · Enter run · Alt+Enter alternatives · Tab preview · Ctrl+↑↓ scope · Ctrl+G activities · F1 key reference · Ctrl+Q quit\n\
         Preview: every scenario is captured under snapshots/holla/; src/bin/holla/README.md lists what each scenario shows.\n\
         Everything is simulated in memory; no stack command is ever executed.",
        concept = scenario_names(&Scenario::CONCEPT),
        parity = scenario_names(&Scenario::PARITY),
    )
}

fn parse_args() -> Options {
    let cli = Cli::command().after_help(cli_after_help()).get_matches();
    let cli = Cli::from_arg_matches(&cli).unwrap_or_else(|error| error.exit());
    let no_motion = std::env::var_os("HOLLA_NO_MOTION").is_some_and(|v| !v.is_empty() && v != "0");
    Options {
        level: cli
            .color
            .map(Into::into)
            .unwrap_or_else(crate::tui::theme::ColorLevel::detect),
        scenario: cli.scenario,
        motion: Motion::resolve(cli.motion.map(Motion::from), no_motion),
        frame: cli.frame,
    }
}

pub fn run() -> std::process::ExitCode {
    let opts = parse_args();
    let theme = crate::tui::theme::Theme::for_level(opts.level);
    let mut app = App::for_scenario(opts.scenario, opts.motion, opts.frame, theme);
    if std::env::var_os("HOLLA_NO_HISTORY").is_some_and(|v| v == "1") {
        app.world.memory.usage = holla_domain::usage::UsageStore::disabled();
    }
    let _ = crate::tui::runtime::drain_pending_input();
    match crate::tui::runtime::run(&mut app) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("holla: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

impl crate::tui::runtime::Application for App {
    fn handle(
        &mut self,
        input: crate::tui::core::event::Input,
    ) -> crate::tui::core::event::Outcome {
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
