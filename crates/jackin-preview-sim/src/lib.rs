//! Deterministic in-memory services: the world, 1Password, provider
//! operations, and the launch pipeline.

#![forbid(unsafe_code)]

/// Construct entry/exit boundary arbiter.
pub mod arbiter;
/// Deterministic changed-file and diff fixture model.
pub mod changes;
/// Virtual clock, jobs, and service state.
pub mod clock;
/// Deterministic scenario data and precedence fixtures.
pub mod fixtures;
/// Deterministic launch-pipeline state machine.
pub mod launch;
/// In-memory 1Password account and vault service.
pub mod onepassword;
/// Deterministic credential validation and usage refresh operations.
pub mod provider;
/// Capsule tabs, panes, transcripts, and daemon simulation.
pub mod pty;
/// Virtual world assembled from the domain fixtures.
pub mod world;

pub use arbiter::{Arbiter, DiscoveryError, EntryDecision, ExitDecision};
pub use changes::{ChangeSet, ChangedFile, DiffLine, DiffLineKind, DiffStatus, Hunk, changes_for};
pub use clock::{Clock, EPOCH_SECS, format_duration};
pub use fixtures::HOME;
pub use launch::{LaunchEvent, LaunchFailure, LaunchPlan, LaunchRun, Stage, StepState};
pub use onepassword::{KeyOutcome, OpError, OpItem, OpSession, SecretClass, SimOnePassword};
pub use provider::{
    CheckRow, FolderProbe, ValidationOutcome, apply_validation, probe_folder, refresh_duration_ms,
    validate, windows_for,
};
pub use pty::{
    AgentProcess, CellPos, Daemon, Direction, Line, Maximized, Pane, PaneId, PaneNode, Position,
    Rect, Seam, Span, Split, SplitDir, Step, Tab, TextViewport, Tone, nearest, script,
};
pub use world::{GithubRepo, GlobalConfig, TrustRow, World, world_for};
