//! The one collection vocabulary (`COMPONENT_ARCHITECTURE.md` §12.2).
//!
//! Decor, empty-state and row-paint types live in `termrock-runtime`
//! (modules `decor`, `empty`, `rowui`); this crate owns the key, selection
//! and reconcile mechanics.

pub mod key;
pub mod reconcile;

pub use key::{ByIndex, DefaultRow, KeyFn, KeySet, SelectMode};
pub use reconcile::{CollectionCore, Reconcile, Reconciliation, StepDir};
