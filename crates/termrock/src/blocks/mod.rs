//! Reusable layout blocks built from public Termrock components.

mod master_detail;
mod shell;
mod split;

pub use master_detail::{
    MasterDetail, MasterDetailAction, MasterDetailError, MasterDetailPolicy, MasterDetailState,
};
pub use shell::{AppShell, AppShellSlots, Chrome, EmptyLeaf, Regions, ShellSlots};
pub use split::{SplitBlock, SplitBlockError};
