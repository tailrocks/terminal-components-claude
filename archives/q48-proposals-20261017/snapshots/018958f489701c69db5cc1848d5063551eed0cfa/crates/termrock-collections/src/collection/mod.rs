//! The one collection vocabulary (`COMPONENT_ARCHITECTURE.md` §12.2).

pub mod key;
pub mod reconcile;

pub use key::{
    ByIndex, DefaultRow, KeyFn, KeySet, SelectMode, index_of, index_of_with, key_at, key_at_with,
};
pub use reconcile::{CollectionCore, Reconcile, Reconciliation, StepDir};
