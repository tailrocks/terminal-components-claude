//! Showcase reference observation and Instant clock adapter (`oracle-showcase`).
//!
//! Drives production [`showcase_app::App`] through the `junie-tui` runtime
//! (`update` / `draw`). Oracle membership is 23 pages including Diff;
//! production currently exposes 22 routes. This crate observes that gap and
//! does not patch product UX. Pointer targets are frozen coordinates or
//! source `Id`s — never buffer text-search selectors.

#![allow(
    clippy::missing_errors_doc,
    reason = "all fallible APIs return AdapterError with variant-level meaning"
)]

pub mod actions;
pub mod clock;
pub mod color;
pub mod driver;
pub mod error;
pub mod observe;
pub mod pages;
pub mod scenarios;
pub mod source_ids;

pub use actions::{Action, ActionProgram, PointerKind};
pub use clock::{InstantClock, deadlines};
pub use color::{ColorSpec, TerminalSize};
pub use driver::{ShowcaseDriver, capture_case};
pub use error::AdapterError;
pub use observe::{CellObs, FrameObservation, HitObs};
pub use pages::{OraclePageId, production_nav_len, production_page_len};
pub use scenarios::{
    ExpandedCase, Membership, ScenarioPage, ScenarioRow, default_page_frames, expand, membership,
    rows, sc_base_overview_80x24_truecolor,
};

/// Accepted producer product identity for TASK-002.
pub const PRODUCT: &str = "oracle-showcase";

/// Oracle page count, including Diff.
pub const ORACLE_PAGE_COUNT: usize = 23;

/// Default page frames: 23 pages × 4 sizes × 4 colors.
pub const DEFAULT_PAGE_FRAME_COUNT: usize = 368;

/// UI oracle commit that defines the 23-page inventory.
pub const UI_ORACLE: &str = "02f5294bfdbf38004cc49130d0aff1d01f31434c";
