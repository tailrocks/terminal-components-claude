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
    InspectDialog, LaunchCandidate, ManagerRowKey, ManagerScreen, ManagerState, INSPECT,
    INSPECT_CLOSE,
};
pub use manager_actions::{
    inspect_facts, Action as ManagerAction, Effect as ManagerEffect, Fact as ManagerFact,
    FactTone as ManagerFactTone, InstanceTarget, LaunchTarget, ManagerActions, RepositoryError,
    RepositoryTarget, Review as ManagerReview, SessionChoice, SessionReview,
    Target as ManagerTarget,
};
pub use settings::SettingsState;
pub use usage::{Tab as UsageTab, UsageState};
