//! `jackin-preview-host-ui`
//!
//! Host chrome, sidebar, status bar, global command palette, tab strip.

#![forbid(unsafe_code)]

pub mod accounts;
pub mod cockpit;
pub mod editor;
pub mod manager;
pub mod manager_actions;
pub mod prelude;
pub mod settings;
pub mod usage;

pub use accounts::AccountsState;
pub use cockpit::{AccountLine, CockpitScreen, CockpitState, HandoffState};
pub use editor::{EditorScreen, EditorState, PendingWorkspace, Tab as EditorTab};
pub use manager::{
    INSPECT, INSPECT_CLOSE, InspectDialog, LaunchCandidate, ManagerRowKey, ManagerScreen,
    ManagerState,
};
pub use manager_actions::{
    Action as ManagerAction, Effect as ManagerEffect, Fact as ManagerFact,
    FactTone as ManagerFactTone, InstanceTarget, LaunchTarget, ManagerActions, RepositoryError,
    RepositoryTarget, Review as ManagerReview, SessionChoice, SessionReview,
    Target as ManagerTarget, inspect_facts,
};
pub use settings::SettingsState;
pub use usage::{Tab as UsageTab, UsageState};
