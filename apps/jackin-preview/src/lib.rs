//! Jackin Preview: a deterministic terminal app built on `tui-next`.
#![forbid(unsafe_code)]
#![expect(
    missing_docs,
    reason = "the preview's public fixture model is intentionally data-shaped"
)]
#![expect(
    unreachable_pub,
    reason = "fixture modules are public to integration tests"
)]
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
mod arbiter;
mod clock;
pub mod domain;
pub mod rain;
mod scenario;
pub mod screens;
pub mod sim;

pub use app::{
    ACCOUNT_ADD, ACCOUNT_PICKER, ACCOUNTS, ACCOUNTS_LIST, APP, App, CAPSULE, CAPSULE_PANES,
    CAPSULE_TABS, ENTER, LAUNCH, LAUNCH_CANCEL, LAUNCH_DIALOG, LAUNCH_RETRY, LAUNCH_STEPS, MANAGER,
    MANAGER_LIST, ROLE_CHOOSE, ROLE_PICKER, Route, SETTINGS, SETTINGS_TRUST, USAGE,
};
pub use domain::instance::RunId;
pub use scenario::{Motion, Scenario};

/// Run the interactive preview through the public `tui-next` entry point.
pub fn run() -> std::io::Result<()> {
    run_scenario(Scenario::Returning, Motion::Full, 0)
}

/// Run a pinned scenario through the public `tui-next` entry point.
pub fn run_scenario(scenario: Scenario, motion: Motion, frame: u64) -> std::io::Result<()> {
    tui_next::run(
        App::for_scenario_at(scenario, motion, frame),
        tui_next::Theme::junie(),
    )
}
