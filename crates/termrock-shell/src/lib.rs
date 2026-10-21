//! The application shell: routing, navigation, chrome, quit guard, status and
//! footer ownership, ported from the PR #26 shell layer onto the parity
//! runtime contracts (`App`/`Cx`/`Ui`/`Response`).
//!
//! The parity runtime already owns focus, hit testing, pointer routing, layers
//! and the update loop, so this crate owns only what apps still carry per app:
//! the page router, the navigation stack, chrome geometry, the quit guard, the
//! status line, the footer hint vocabulary and keyboard-deliverability, and the
//! reachability harness used to prove every word stays reachable.
//!
//! Nothing here draws its own pixels: the shell composes through the shared
//! composition slots ([`shell::AppShell`]) and returns vocabulary the app
//! renders with existing components. The one exception is
//! [`report::TextReport`], a content block that paints through the runtime
//! [`termrock_runtime::Ui`] like the other Termrock blocks do.
//!
//! The `reach` module is a test tool compiled only with the crate's
//! default-off `testing` feature, so production builds never carry it.

pub mod chrome;
pub mod finder;
pub mod finder_screen;
pub mod footer;
pub mod keyboard;
pub mod nav;
pub mod page;
pub mod quit;
#[cfg(feature = "testing")]
pub mod reach;
pub mod report;
pub mod router;
pub mod shell;
pub mod status;

pub use chrome::{Chrome, Regions};
pub use finder::{
    FinderBand, FinderColumns, FinderContent, FinderLayout, FinderLine, FinderPlate, FinderPolicy,
    FinderPreview, FinderPreviewTitle, FinderRow, FinderTone, finder_band,
};
pub use finder_screen::{
    ActionRow, BAND_LABEL, BAND_META, BAND_RULE, BandChip, BandMarker, FINDER_GREY, FinderGrid,
    FinderLead, ScopeBand, paint_action_row,
};
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
