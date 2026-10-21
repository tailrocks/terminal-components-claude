#![forbid(unsafe_code)]
#![expect(
    clippy::pedantic,
    reason = "fixture and rendering code favors explicit deterministic data"
)]
#![expect(
    clippy::arithmetic_side_effects,
    reason = "all arithmetic is over bounded deterministic fixture values"
)]
#![allow(clippy::indexing_slicing)]
#![cfg_attr(test, allow(clippy::expect_used, clippy::panic, clippy::unwrap_used))]

pub mod app;
pub mod cli;

pub mod screens {
    pub use jackin_preview_capsule_ui::screens::{
        FileBrowserAction, FileBrowserEntry, FileBrowserState, OpFlowAction, OpFlowStage,
        OpFlowState, OpFlowStatus,
    };
    pub use jackin_preview_capsule_ui::screens::{capsule, file_browser, inspect, op_flow};
    pub use jackin_preview_host_ui::screens::{
        accounts, cockpit, editor, manager, manager_actions, settings, usage,
    };
    pub use jackin_preview_presentation::prelude;
}

pub mod domain {
    pub use jackin_preview_domain::*;

    pub use jackin_preview_sim::fixtures;
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
    pub use jackin_preview_sim::clock::*;
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

pub fn run() -> std::io::Result<()> {
    run_scenario(Scenario::FirstUse, Motion::Full, 0)
}

pub fn run_scenario(scenario: Scenario, motion: Motion, frame: u64) -> std::io::Result<()> {
    run_scenario_with_theme(scenario, motion, frame, termrock::Theme::junie())
}

pub fn run_scenario_with_theme(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    theme: termrock::Theme,
) -> std::io::Result<()> {
    termrock::run(App::for_scenario_at(scenario, motion, frame), theme)
}
