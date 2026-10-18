//! The one collection vocabulary (`COMPONENT_ARCHITECTURE.md` §12.2).

pub(crate) mod decor;
pub(crate) mod empty;
pub(crate) mod key;
pub(crate) mod reconcile;
pub(crate) mod rowui;

pub use decor::{CellDecor, RowDecor};
pub use empty::{EmptyState, RowTotal, Status};
pub use key::{ByIndex, DefaultRow, KeyFn, KeySet, RowFn, SelectMode};
pub use reconcile::{CollectionCore, Reconcile, Reconciliation};
pub use rowui::{CellUi, ColumnsUi, MAX_COLUMNS, RowUi};
