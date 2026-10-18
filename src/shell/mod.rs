




















mod app;
mod cx;
mod footer;
mod page;
mod router;
mod status;

pub use crate::ui::layer::LayerTag;
pub use app::{App, PageKey, Shell, blur_fan_out, handle, render};
pub use cx::{Cx, Request, background};
pub use footer::{Footer, FooterStatus, deliverable};
pub use page::{
    Page, PageCx, PageTypes, open_context, rebuild_on_tick, replay_key, row_anchor, row_choice,
};
pub use router::{KeyHost, KeyRouter, ShellAction, ShellKeys};
pub use status::{JobKey, StatusLine};

use crate::runtime::Application;




pub fn run(app: &mut impl Application) -> std::io::Result<()> {
    if let Some(delay) = start_delay() {
        std::thread::sleep(delay);
    }
    let _ = crate::runtime::drain_pending_input();
    crate::runtime::run(app)
}





fn start_delay() -> Option<std::time::Duration> {
    std::env::var("JUNIE_TUI_START_DELAY_MS")
        .ok()?
        .parse()
        .ok()
        .map(std::time::Duration::from_millis)
}
