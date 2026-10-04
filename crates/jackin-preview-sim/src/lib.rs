//! `jackin-preview-sim`
//!
//! Streaming log generators, synthetic latency, simulated failure modes.

#![forbid(unsafe_code)]

pub mod arbiter;
pub mod changes;
pub mod fixtures;
pub mod launch;
pub mod onepassword;
pub mod provider;
pub mod pty;
pub mod world;

pub use arbiter::{Arbiter, DiscoveryError, EntryDecision, ExitDecision};
pub use changes::{ChangeSet, ChangedFile, DiffLine, DiffLineKind, DiffStatus, Hunk, changes_for};
pub use launch::{LaunchEvent, LaunchFailure, LaunchPlan, LaunchRun, Stage};
pub use onepassword::{KeyOutcome, OpError, OpItem, OpSession, SecretClass, SimOnePassword};
pub use provider::{
    CheckRow, FolderProbe, ValidationOutcome, apply_validation, probe_folder, refresh_duration_ms,
    validate, windows_for,
};
pub use pty::{
    AgentProcess, Daemon, Direction, Line, Maximized, Pane, PaneId, PaneNode, Seam, Span, Split,
    SplitDir, Step, Tab, TextViewport, Tone, nearest, script,
};
pub use world::{GithubRepo, GlobalConfig, TrustRow, World, world_for};
