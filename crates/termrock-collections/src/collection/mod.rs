#![doc = r" The one collection vocabulary (`COMPONENT_ARCHITECTURE.md` §12.2)."]
#![doc = r""]
#![doc = r" Decor, empty-state and row-paint types live in `termrock-runtime`"]
#![doc = r" (modules `decor`, `empty`, `rowui`); this crate owns the key, selection"]
#![doc = r" and reconcile mechanics."]

pub mod key;
pub mod reconcile;

pub use key::{ByIndex, DefaultRow, KeyFn, KeySet, SelectMode};
pub use reconcile::{CollectionCore, Reconcile, Reconciliation, StepDir};
