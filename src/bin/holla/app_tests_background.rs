


use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::shell::Page;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::widgets::keyhint::Hint;

use crate::app::{TabEntry, TabKind};
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Cx, Go, Holla, Screen};
use crate::sim::world::World;



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





#[test]
fn a_background_tabs_tick_status_shows_when_the_tab_is_shown() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.app.tabs.push(TabEntry {
        kind: TabKind::Plan("probe".into()),
        stack: vec![Box::new(TickProbe { ticked: false })],
        pending: vec![],
    });
    assert_eq!(h.app.active, 0);
    h.ticks(1);
    assert_eq!(h.app.tabs.len(), 2, "a background tab never closes itself");
    assert!(!h.text().contains("probe finished in the background"));
    h.alt(KeyCode::Char('1'));
    assert_eq!(h.app.active, 1);
    assert!(
        h.text().contains("probe finished in the background"),
        "the status waited for the tab:\n{}",
        h.text()
    );
}



#[test]
fn ctrl_page_keys_step_the_tabs() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.app.tabs.push(TabEntry {
        kind: TabKind::Plan("probe".into()),
        stack: vec![Box::new(TickProbe { ticked: true })],
        pending: vec![],
    });
    h.ctrl(KeyCode::PageDown);
    assert_eq!(h.app.active, 1);
    h.ctrl(KeyCode::PageDown);
    assert_eq!(h.app.active, 0, "wraps to the first tab");
    h.ctrl(KeyCode::PageUp);
    assert_eq!(h.app.active, 1);
}
