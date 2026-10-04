//! Jackin Preview: a deterministic terminal app built on `termrock`.
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

pub mod app;
pub mod cli;
pub mod screens;

pub mod domain {
    pub use jackin_preview_domain::*;
}
pub mod sim {
    pub use jackin_preview_sim::*;
}
pub mod rain {
    pub use jackin_preview_presentation::rain::*;
}
pub mod arbiter {
    pub use jackin_preview_sim::arbiter::*;
}
pub mod clock {
    pub use jackin_preview_domain::clock::*;
}
pub mod scenario {
    pub use jackin_preview_domain::scenario::*;
}

pub use app::{
    ACCOUNT_ADD, ACCOUNT_PICKER, ACCOUNTS, ACCOUNTS_LIST, APP, App, CAPSULE, CAPSULE_PANES,
    CAPSULE_TABS, ENTER, LAUNCH, LAUNCH_CANCEL, LAUNCH_DIALOG, LAUNCH_RETRY, MANAGER, MANAGER_LIST,
    ROLE_CHOOSE, ROLE_PICKER, Route, SETTINGS, SETTINGS_TRUST, USAGE,
};
pub use domain::instance::RunId;
pub use rain::{INTRO_END, TICK_MS};
pub use scenario::{Motion, Scenario};
pub use sim::world::{World, world_for};

/// Run the interactive preview through the public `termrock` entry point.
pub fn run() -> std::io::Result<()> {
    run_scenario(Scenario::FirstUse, Motion::Full, 0)
}

/// Run a pinned scenario through the public `termrock` entry point.
pub fn run_scenario(scenario: Scenario, motion: Motion, frame: u64) -> std::io::Result<()> {
    run_scenario_with_theme(scenario, motion, frame, termrock::Theme::junie())
}

/// Run a pinned scenario with a caller-selected theme.
pub fn run_scenario_with_theme(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    theme: termrock::Theme,
) -> std::io::Result<()> {
    termrock::run(App::for_scenario_at(scenario, motion, frame), theme)
}
