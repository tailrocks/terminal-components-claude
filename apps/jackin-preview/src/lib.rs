//! Jackin Preview: a deterministic terminal app built on `tui-next`.
#![deny(unsafe_code)]
#![cfg_attr(
    test,
    allow(
        clippy::arithmetic_side_effects,
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
mod scenario;
pub mod sim;

pub use app::{
    ACCOUNT_PICKER, ACCOUNTS, APP, CAPSULE, CAPSULE_PANES, CAPSULE_TABS, ENTER, LAUNCH,
    LAUNCH_DIALOG, MANAGER, MANAGER_LIST, ROLE_CHOOSE, ROLE_PICKER, SETTINGS, USAGE, App, Route,
};
pub use domain::instance::RunId;
pub use scenario::{Motion, Scenario};

/// Run the interactive preview through the public `tui-next` entry point.
pub fn run() -> std::io::Result<()> {
    tui_next::run(App::default(), tui_next::Theme::junie())
}
