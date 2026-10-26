pub mod chrome;
pub mod footer;
pub mod keyboard;
pub mod nav;
pub mod page;
pub mod quit;
pub mod report;
pub mod router;
pub mod shell;
pub mod status;

pub use chrome::{Chrome, Regions};
pub use footer::{Footer, FooterStatus, HINT_AUTO, HINT_NORMAL, HINT_PINNED, Hint};
pub use nav::{Crumb, Nav, PageKey, Router};
pub use page::{Page, PageLeaf, page_leaf, rebuild_on_tick, replay_key};
pub use quit::{
    KeyContext, Leave, LeaveDialog, LeaveDialogKind, LeaveGuard, LeaveKey, QuitKeys, Resolved,
    Stakes,
};
pub use report::{ReportContent, ReportRows, TextReport};
pub use router::{KeyBinding, KeyHost, KeyRouter, ShellAction, ShellKeys, Tier, chord, lookup};
pub use shell::{AppShell, AppShellSlots, EmptyLeaf, ShellSlots};
pub use status::{JobKey, StatusLine, TONE_ERROR, TONE_NORMAL, TONE_SECONDARY, TONE_SUCCESS};
