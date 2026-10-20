//! Reusable layout blocks built from public Termrock components.

mod master_detail;
mod split;

pub use master_detail::{
    MasterDetail, MasterDetailAction, MasterDetailError, MasterDetailPolicy, MasterDetailState,
};
pub use split::{SplitBlock, SplitBlockError};
