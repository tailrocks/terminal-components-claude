//! Jackin, redesigned: an interactive terminal preview of the complete
//! Jackin operator experience built on the Junie design system. Every
//! external system (Docker, daemons, PTYs, providers, 1Password) is a
//! deterministic in-memory simulation; every operator interaction is real.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_chrome;
#[cfg(test)]
mod app_tests_colour;
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
mod arbiter;
mod clock;
mod domain;
mod rain;
mod scenario;
mod screens;
mod sim;

use clap::Parser;
use junie_tui::cli::CommonArgs;

use crate::app::App;
use crate::scenario::Scenario;

/// Set to `1` (anything but empty or `0`) for reduced motion.
const NO_MOTION_ENV: &str = "JACKIN_NO_MOTION";

#[derive(Debug, Parser)]
#[command(
    name = "jackin-preview",
    about = "Jackin terminal preview",
    long_about = "Jackin-preview — Jackin redesigned on the Junie design system (deterministic preview)"
)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    /// The fixture scenario to open
    #[arg(short = 's', long, value_parser = parse_scenario, default_value = "first-use")]
    scenario: Scenario,
}

fn scenario_names() -> String {
    Scenario::ALL
        .iter()
        .map(|s| s.name())
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_scenario(value: &str) -> Result<Scenario, String> {
    Scenario::from_name(value).ok_or_else(|| {
        format!(
            "unknown scenario {value:?}; use one of {}",
            scenario_names()
        )
    })
}

fn cli_after_help() -> String {
    let scenarios = scenario_names();
    format!(
        "Scenarios: {scenarios}\n\
         Motion:    explicit --motion wins; otherwise {NO_MOTION_ENV}=1 selects reduced motion\n\
         Frame:     with --motion paused, the exact fixture tick to render (intro, cockpit, outro phases)\n\n\
         Keys: Tab/Shift+Tab focus · ↑↓ move · Enter launch/activate · Esc back · u Accounts & Usage · s Settings · ? help · q quit\n\
         Everything is simulated in memory; the real Jackin CLI is never touched."
    )
}

fn main() -> std::io::Result<()> {
    let cli: Cli = junie_tui::cli::parse(cli_after_help());
    let motion = cli.common.motion(NO_MOTION_ENV);
    let mut app = App::for_scenario(cli.scenario, motion, cli.common.frame, cli.common.theme());
    // stale key presses from the launching shell must not skip the ritual
    junie_tui::shell::run(&mut app)
}

junie_tui::impl_application!(App);
