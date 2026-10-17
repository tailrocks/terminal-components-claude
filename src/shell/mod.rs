//! The application shell: behaviour every application gets from the
//! library instead of writing it (docs/architecture/shared-components-plan.md
//! §1.8). This module grows step by step; today it holds the leaf pieces:
//!
//! - [`Shell`] and [`App`]: the state every app keeps and the trait that
//!   gives an app [`Application`]: the render prologue and epilogue, focus
//!   upkeep ([`PageKey`]) and the blur fan-out.
//! - [`Cx`] and [`Request`]: the event context a page handles input in and
//!   what it asks of the shell, with the background-request policy
//!   ([`background`]).
//! - [`Page`]: what every page of every app can do, so an app's screen
//!   trait only adds its own extras.
//! - [`KeyRouter`], [`ShellKeys`] and [`ShellAction`]: the one key order
//!   every app routes keys in ([`KeyHost`] names its stages).
//! - [`Router`] and [`Nav`]: the page stacks (one per tab), navigation
//!   requests, the leave guard on every move that drops a page, and the
//!   clickable trail ([`Breadcrumb`]).
//! - [`Chrome`]: the frame's header, body and footer regions, and
//!   [`Menus`] whose items carry typed commands.
//! - [`Clipboard`]: copies with a truthful result (OSC 52).
//! - [`Footer`]: the one footer composition (hints of the top layer, the
//!   EDIT badge, the status line with errors over hints, click targets).
//! - [`StatusLine`]: the transient status message and its expiry rule;
//!   [`JobKey`] ties a progress status to the job it reports on.
//! - [`run`]: start an [`Application`] on a terminal, after discarding
//!   input queued before it started.

mod app;
mod chrome;
pub mod clipboard;
mod cx;
mod footer;
mod nav;
mod page;
mod router;
mod status;

pub use crate::ui::layer::LayerTag;
pub use crate::widgets::breadcrumb::Breadcrumb;
pub use app::{App, PageKey, Shell, blur_fan_out, handle, render};
pub use chrome::{Chrome, Menus, Regions};
pub use clipboard::{Clipboard, Copied};
pub use cx::{Cx, Request, background};
pub use footer::{Footer, FooterStatus, deliverable};
pub use nav::{Nav, Router};
pub use page::{
    Page, PageCx, PageTypes, open_context, rebuild_on_tick, replay_key, row_anchor, row_choice,
};
pub use router::{KeyHost, KeyRouter, ShellAction, ShellKeys};
pub use status::{JobKey, StatusLine};

use crate::runtime::Application;

/// Run `app` until it asks to quit. Input already queued when the process
/// starts (key presses typed into the launching shell) is discarded first,
/// so it cannot act on the first screen or skip an opening sequence.
pub fn run(app: &mut impl Application) -> std::io::Result<()> {
    if let Some(delay) = start_delay() {
        std::thread::sleep(delay);
    }
    let _ = crate::runtime::drain_pending_input();
    crate::runtime::run(app)
}

/// Test instrumentation: `JUNIE_TUI_START_DELAY_MS` holds [`run`] for that
/// long before it discards queued input, so a PTY test can type keys
/// "before startup" deterministically
/// (`tests/visual_baseline/startup_input.rs`). Unset in normal use.
fn start_delay() -> Option<std::time::Duration> {
    std::env::var("JUNIE_TUI_START_DELAY_MS")
        .ok()?
        .parse()
        .ok()
        .map(std::time::Duration::from_millis)
}
