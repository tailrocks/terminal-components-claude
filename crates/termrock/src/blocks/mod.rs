//! Reusable layout blocks built from public Termrock components.

mod line;
mod master_detail;
mod section;
mod shell;
mod split;
mod status;
mod tabs;
mod toolbar;

pub use line::paint_row;
pub use master_detail::{
    MasterDetail, MasterDetailAction, MasterDetailError, MasterDetailPolicy, MasterDetailState,
};
pub use section::paint_section_header;
pub use shell::{AppShell, AppShellExt, AppShellSlots, Chrome, EmptyLeaf, Regions, ShellSlots};
pub use split::{SplitBlock, SplitBlockError};
pub use status::{StatusSeg, StatusTone, paint_status};
pub use tabs::{Tab, paint_tabs};
pub use toolbar::{
    ToolItem, Toolbar, ToolbarAction, ToolbarActionEvent, ToolbarActions, ToolbarError,
    ToolbarReferenceState, ToolbarState, ToolbarTip, ToolbarTooltipPolicy,
};
