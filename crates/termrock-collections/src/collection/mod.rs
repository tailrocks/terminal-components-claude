//! The one collection vocabulary (`COMPONENT_ARCHITECTURE.md` §12.2).

pub mod key;
pub mod reconcile;

pub use key::{ByIndex, DefaultRow, KeyFn, KeySet, SelectMode};
pub use reconcile::{CollectionCore, Reconcile, Reconciliation};
