//! Reusable layout blocks built from public Termrock components.

mod split;
mod master_detail;

pub use split::{SplitBlock, SplitBlockError};
pub use master_detail::{MasterDetail, MasterDetailAction, MasterDetailError, MasterDetailPolicy, MasterDetailState};
