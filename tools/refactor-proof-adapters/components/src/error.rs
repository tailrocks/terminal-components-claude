//! Fail-closed adapter errors.

use core::fmt;

use crate::families::Family;

/// Adapter failure. Missing catalog data, unaddressable identities, and
/// frame requests for architecture-only families fail closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdapterError {
    /// Component inventory is malformed or incomplete.
    Catalog {
        /// Diagnostic.
        message: String,
    },
    /// Duplicate expansion identity: the union is not a set.
    DuplicateIdentity {
        /// Colliding identity.
        identity: String,
    },
    /// Architecture-only family was asked for a frame it must never have.
    ArchitectureHasNoFrame {
        /// Family with a non-frame disposition.
        family: Family,
    },
    /// State is explicitly non-applicable to this family.
    StateNotApplicable {
        /// Family.
        family: Family,
        /// Requested state token.
        state: &'static str,
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
    /// Fixture interaction did not reach the requested runtime state.
    StateUnreachable {
        /// Diagnostic.
        message: String,
    },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog { message } | Self::StateUnreachable { message } => f.write_str(message),
            Self::DuplicateIdentity { identity } => {
                write!(f, "duplicate expansion identity {identity}")
            }
            Self::ArchitectureHasNoFrame { family } => {
                write!(f, "architecture-only family {} has no frame", family.slug())
            }
            Self::StateNotApplicable { family, state } => {
                write!(f, "state {state} is non-applicable to {}", family.slug())
            }
            Self::UnaddressableId => f.write_str("source id has no published area"),
            Self::MissingCell { x, y } => write!(f, "missing cell at {x},{y}"),
        }
    }
}

impl core::error::Error for AdapterError {}
