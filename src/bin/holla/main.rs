//! Holla: a context-adaptive action launcher for the terminal, as a
//! deterministic preview on the Junie design system. Every external system
//! (mise, git, gh, docker, btm, pg_activity, ssh, the filesystem) is an
//! in-memory simulation; every operator interaction is real. No stack
//! command is ever spawned.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_colour;
#[cfg(test)]
mod app_tests_context;
#[cfg(test)]
mod app_tests_context_screens;
#[cfg(test)]
mod app_tests_copy;
#[cfg(test)]
mod app_tests_coverage;
#[cfg(test)]
mod app_tests_fields;
#[cfg(test)]
mod app_tests_flows;
#[cfg(test)]
mod app_tests_mouse;
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

use clap::Parser;
use junie_tui::cli::CommonArgs;

use crate::app::App;
use crate::scenario::Scenario;

/// Set to `1` (anything but empty or `0`) for reduced motion.
const NO_MOTION_ENV: &str = "HOLLA_NO_MOTION";

#[derive(Debug, Parser)]
#[command(
    name = "holla",
    about = "Context-adaptive action launcher",
    long_about = "Holla — this folder, this host, right now (deterministic preview on the Junie design system)"
)]
struct Cli {
    #[command(flatten)]
    common: CommonArgs,
    /// The fixture scenario to open
    #[arg(short = 's', long, value_parser = parse_scenario, default_value = "first-use")]
    scenario: Scenario,
}

fn parse_scenario(value: &str) -> Result<Scenario, String> {
    Scenario::from_name(value).ok_or_else(|| {
        format!(
            "unknown scenario {value:?}; use one of {}",
            scenario_names(&Scenario::ALL)
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
         Motion:    explicit --motion wins; otherwise {NO_MOTION_ENV}=1 selects reduced motion\n\
         Frame:     with --motion paused, the exact fixture tick to render\n\n\
         Keys: type to search · ↑↓ move · Enter run · Alt+Enter alternatives · Tab preview · Ctrl+↑↓ scope · Ctrl+G activities · F1 key reference · Ctrl+Q quit\n\
         Preview: every scenario is captured under snapshots/holla/; src/bin/holla/README.md lists what each scenario shows.\n\
         Everything is simulated in memory; no stack command is ever executed.",
        concept = scenario_names(&Scenario::CONCEPT),
        parity = scenario_names(&Scenario::PARITY),
    )
}

fn main() -> std::io::Result<()> {
    let cli: Cli = junie_tui::cli::parse(cli_after_help());
    let motion = cli.common.motion(NO_MOTION_ENV);
    let mut app = App::for_scenario(cli.scenario, motion, cli.common.frame, cli.common.theme());
    if std::env::var_os("HOLLA_NO_HISTORY").is_some_and(|v| v == "1") {
        // the opt-out: nothing is read, learned or written
        app.world.memory.usage = crate::domain::usage::UsageStore::disabled();
    }
    junie_tui::shell::run(&mut app)
}

junie_tui::impl_application!(App);
