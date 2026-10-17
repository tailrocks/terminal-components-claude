//! Tabs on the library shell: background requests (CO9) and the
//! router's page keys (S8a).

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::shell::Page;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::keyhint::Hint;

use crate::app::TabKind;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Cx, Go, Holla, Screen};
use crate::sim::world::World;

/// A tab page that reports on its first tick: a status, and a request
/// that would act on the screen (closing the tab).
struct TickProbe {
    ticked: bool,
}

impl Page<Holla> for TickProbe {
    fn on_key(&mut self, _key: &Key, _w: &mut World, _cx: &mut Cx) -> Outcome {
        Outcome::Ignored
    }
    fn on_tick(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        if !self.ticked {
            self.ticked = true;
            cx.status("probe finished in the background");
            cx.go(Go::CloseTab);
        }
        Outcome::Changed
    }
    fn render(&mut self, _area: Rect, _buf: &mut Buffer, _ctx: &mut RenderCtx, _w: &World) {}
    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        vec![]
    }
    fn crumb(&self, _w: &World) -> String {
        "Probe".into()
    }
    fn primary_focus(&self) -> Option<WidgetId> {
        None
    }
}

impl Screen for TickProbe {}

/// CO9: a background tab's tick follows the shell's background-request
/// policy, as its blur already did: the status it reports waits for the
/// tab and shows once the tab is shown; what would act on the screen
/// (closing the tab) is dropped. Before, the whole tick report was lost.
#[test]
fn a_background_tabs_tick_status_shows_when_the_tab_is_shown() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.app.add_tab(
        TabKind::Plan("probe".into()),
        Box::new(TickProbe { ticked: false }),
    );
    assert_eq!(h.app.active(), 0);
    h.ticks(1);
    assert_eq!(h.app.tabs.len(), 2, "a background tab never closes itself");
    assert!(!h.text().contains("probe finished in the background"));
    h.alt(KeyCode::Char('1'));
    assert_eq!(h.app.active(), 1);
    assert!(
        h.text().contains("probe finished in the background"),
        "the status waited for the tab:\n{}",
        h.text()
    );
}

/// The library router's page keys reach holla's tabs (S8a, SI23):
/// Ctrl+PgDn/PgUp step the tabs, wrapping, as Alt+digit picks one.
#[test]
fn ctrl_page_keys_step_the_tabs() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.app.add_tab(
        TabKind::Plan("probe".into()),
        Box::new(TickProbe { ticked: true }),
    );
    h.ctrl(KeyCode::PageDown);
    assert_eq!(h.app.active(), 1);
    h.ctrl(KeyCode::PageDown);
    assert_eq!(h.app.active(), 0, "wraps to the first tab");
    h.ctrl(KeyCode::PageUp);
    assert_eq!(h.app.active(), 1);
}
