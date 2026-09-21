//! Fail-closed adapter errors.

use core::fmt;

use crate::pages::OraclePageId;

/// Adapter failure. Missing oracle data or unfrozen selectors fail closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterError {
    /// Production App has no route for this oracle page (observed, not repaired).
    ProductionRouteAbsent {
        /// Missing oracle page.
        page: OraclePageId,
    },
    /// Focus/present loop did not settle.
    PresentDidNotSettle,
    /// Runtime rejected input because presentation/settle is still required.
    PendingInput,
    /// Backwards Instant/Moment movement.
    ClockBackwards,
    /// Pointer/focus label without a frozen coordinate or Tab count.
    UnfrozenSelector {
        /// Source label from the scenario row.
        label: String,
    },
    /// Scenario action cannot be executed without a candidate-derived finder.
    UninterpretedAction {
        /// Raw action token.
        raw: String,
    },
    /// Source `Id` had no published area.
    UnaddressableId,
    /// Buffer cell outside the published frame.
    MissingCell {
        /// Column.
        x: u16,
        /// Row.
        y: u16,
    },
    /// Scenario inventory is malformed or incomplete.
    Catalog {
        /// Diagnostic.
        message: String,
    },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProductionRouteAbsent { page } => {
                write!(f, "production has no route for oracle page {}", page.slug())
            }
            Self::PresentDidNotSettle => f.write_str("runtime present/settle did not finish"),
            Self::PendingInput => f.write_str("runtime input pending present/settle"),
            Self::ClockBackwards => f.write_str("monotonic Instant/Moment cannot move backwards"),
            Self::UnfrozenSelector { label } => {
                write!(f, "selector {label:?} has no frozen coordinate/Tab count")
            }
            Self::UninterpretedAction { raw } => {
                write!(f, "action {raw:?} is not a frozen key/time/tick/resize")
            }
            Self::UnaddressableId => f.write_str("source id has no published area"),
            Self::MissingCell { x, y } => write!(f, "missing cell at {x},{y}"),
            Self::Catalog { message } => f.write_str(message),
        }
    }
}

impl core::error::Error for AdapterError {}
