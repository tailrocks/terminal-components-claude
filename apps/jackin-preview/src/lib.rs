//! Jackin Preview: a deterministic terminal app built on `junie-tui`.
#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used
    )
)]

mod app;
#[allow(
    dead_code,
    reason = "the private arbiter retains deterministic lifecycle branches for fixture coverage"
)]
mod arbiter;
#[allow(
    dead_code,
    reason = "the private clock retains deterministic formatting helpers for fixture coverage"
)]
mod clock;
#[allow(
    dead_code,
    reason = "private domain fixtures cover deterministic product states not on the shell path"
)]
mod domain;
#[allow(
    dead_code,
    reason = "private atmosphere helpers cover deterministic capture states beyond the shell path"
)]
mod rain;
mod scenario;
#[allow(
    dead_code,
    reason = "private screen adapters retain deterministic flows for later shell composition"
)]
mod screens;
#[allow(
    dead_code,
    reason = "private simulators retain deterministic provider and terminal fixture branches"
)]
mod sim;

pub use app::{
    ACCOUNT_ADD, ACCOUNT_PICKER, ACCOUNTS, ACCOUNTS_LIST, APP, App, CAPSULE, CAPSULE_PANES,
    CAPSULE_TABS, ENTER, LAUNCH, LAUNCH_CANCEL, LAUNCH_DIALOG, LAUNCH_RETRY, LAUNCH_STEPS, MANAGER,
    MANAGER_LIST, ROLE_CHOOSE, ROLE_PICKER, Route, SETTINGS, SETTINGS_TRUST, USAGE,
};
pub use domain::instance::RunId;
pub use rain::{INTRO_END, TICK_MS};
pub use scenario::{Motion, Scenario};
pub use sim::world::{World, world_for};

/// Run the interactive preview through the public `junie-tui` entry point.
pub fn run() -> std::io::Result<()> {
    run_scenario(Scenario::Returning, Motion::Full, 0)
}

/// Run a pinned scenario through the public `junie-tui` entry point.
pub fn run_scenario(scenario: Scenario, motion: Motion, frame: u64) -> std::io::Result<()> {
    run_scenario_with_theme(scenario, motion, frame, junie_tui::Theme::junie())
}

/// Run a pinned scenario with a caller-selected theme.
pub fn run_scenario_with_theme(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    theme: junie_tui::Theme,
) -> std::io::Result<()> {
    junie_tui::run(App::for_scenario_at(scenario, motion, frame), theme)
}
