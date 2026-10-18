#![doc = r" Jackin Preview: a deterministic terminal app built on `termrock`."]
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

#[doc = r" Route-owned screens, split across the host and capsule UI crates and"]
#[doc = r" re-united here under the monolith's `screens::` namespace."]
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

#[doc = r" Domain re-exports (monolith `domain::` namespace)."]
pub mod domain {
    pub use jackin_preview_domain::*;

    pub use jackin_preview_sim::fixtures;
}
#[doc = r" Sim re-exports (monolith `sim::` namespace)."]
pub mod sim {
    pub use jackin_preview_sim::*;
}
#[doc = r" Atmosphere re-exports (monolith `rain::` namespace)."]
pub mod rain {
    pub use jackin_preview_presentation::rain::*;
}
#[doc = r" Arbiter re-exports (monolith `arbiter::` namespace)."]
pub mod arbiter {
    pub use jackin_preview_sim::arbiter::*;
}
#[doc = r" Clock re-exports (monolith `clock::` namespace)."]
pub mod clock {
    pub use jackin_preview_sim::clock::*;
}
#[doc = r" Scenario re-exports (monolith `scenario::` namespace)."]
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

#[doc = r" Run the interactive preview through the public `termrock` entry point."]
pub fn run() -> std::io::Result<()> {
    run_scenario(Scenario::FirstUse, Motion::Full, 0)
}

#[doc = r" Run a pinned scenario through the public `termrock` entry point."]
pub fn run_scenario(scenario: Scenario, motion: Motion, frame: u64) -> std::io::Result<()> {
    run_scenario_with_theme(scenario, motion, frame, termrock::Theme::junie())
}

#[doc = r" Run a pinned scenario with a caller-selected theme."]
pub fn run_scenario_with_theme(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    theme: termrock::Theme,
) -> std::io::Result<()> {
    termrock::run(App::for_scenario_at(scenario, motion, frame), theme)
}
