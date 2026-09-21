//! Reusable component-family observation adapter (`oracle-components`).
//!
//! TASK-006 classifies all 54 [`component-parity.tsv`] rows, expands every
//! oracle-renderable family into actual production widget/view fixtures, and
//! observes complete frames through production paths only (`Harness` /
//! `Runtime` + real widget `update` / `draw`). No frame is hardcoded, no
//! oracle snapshot is read, and no renderer is substituted.
//!
//! Lane summary: 42 families render directly through a production widget
//! fixture, 5 are observed through the shared corpus compositions, and 7 are
//! architecture-only with an explicit non-frame disposition bound to a future
//! owner (notably `testing-registry`, bound to TASK-073 and TASK-031, which
//! has no screenshot, direct-capture, or PTY identity).
//!
//! [`component-parity.tsv`]: https://github.com/donbeave/terminal-components-claude/blob/refactor/holla-parity/docs/refactoring-plan/component-parity.tsv

#![allow(
    clippy::missing_errors_doc,
    reason = "all fallible APIs return AdapterError with variant-level meaning"
)]

pub mod color;
pub mod disposition;
pub mod error;
pub mod expansion;
pub mod families;
pub mod fixtures;
pub mod observe;
pub mod states;
pub mod union;

pub use color::{ColorSpec, Origin, TerminalSize};
pub use disposition::{
    Disposition, Lane, OldNewMapping, architecture_owners, disposition, mapping,
};
pub use error::AdapterError;
pub use expansion::{
    ExpandedCase, Facet, FadePosition, Membership, ViewportMutation, expand, fade_heights,
    membership,
};
pub use families::{FAMILY_COUNT, Family, family_rows, rows};
pub use fixtures::{capture, capture_repeat_pair};
pub use observe::{CellObs, FrameObservation, HitObs};
pub use states::{ComponentState, applicable_states, excluded_states};
pub use union::{APP_NAMESPACES, join_namespaces};

/// Accepted producer product identity for TASK-006.
pub const PRODUCT: &str = "oracle-complete";

/// Adapter namespace joined with the four accepted application namespaces.
pub const NAMESPACE: &str = "components";

/// UI oracle commit that defines the component inventory.
pub const UI_ORACLE: &str = "02f5294bfdbf38004cc49130d0aff1d01f31434c";

/// Families rendered directly through a production widget fixture.
pub const DIRECT_FAMILY_COUNT: usize = 42;

/// Families observed through shared corpus compositions (no own frame).
pub const COMPOSITION_FAMILY_COUNT: usize = 5;

/// Architecture-only families with an explicit non-frame disposition.
pub const ARCHITECTURE_FAMILY_COUNT: usize = 7;
