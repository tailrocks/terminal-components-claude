//! Pointer text selection in holla's fields through the real App: click
//! counts, press and drag reach the argument fields (interaction #16).

use junie_tui::core::event::MouseKind;
use junie_tui::theme::Theme;
use ratatui::crossterm::event::KeyCode;

use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};

/// The selected cells of row `y`, as drawn.
fn selected_on_row(h: &H, y: u16) -> String {
    let sel = Theme::junie().selection().bg.expect("a selection colour");
    let buf = h.term.backend().buffer();
    (0..buf.area.width)
        .filter(|&x| buf[(x, y)].bg == sel)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

/// The arguments page with "alpha beta gamma" in its first field.
fn args() -> (H, u16, u16) {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    let item = h
        .app
        .world
        .items()
        .into_iter()
        .find(|item| !item.args.is_empty())
        .expect("an argument-bearing fixture item")
        .id;
    h.app.go(Go::Push(Page::Args { item }));
    h.draw();
    h.key(KeyCode::Enter);
    for _ in 0..40 {
        h.key(KeyCode::Backspace);
    }
    h.type_str("alpha beta gamma");
    h.key(KeyCode::Tab);
    let (x, y) = h
        .find("alpha beta gamma")
        .unwrap_or_else(|| panic!("{}", h.text()));
    (h, x, y)
}

#[test]
fn a_double_click_in_an_argument_field_selects_the_word() {
    let (mut h, x, y) = args();
    h.click(x + 7, y);
    h.click(x + 7, y);
    assert_eq!(selected_on_row(&h, y), "beta", "{}", h.text());
}

#[test]
fn dragging_in_an_argument_field_selects_and_a_double_press_drags_by_words() {
    let (mut h, x, y) = args();
    h.mouse(MouseKind::Down, x, y);
    h.mouse(MouseKind::Drag, x + 3, y);
    h.mouse(MouseKind::Up, x + 3, y);
    assert_eq!(selected_on_row(&h, y), "alp", "{}", h.text());
    h.click(x + 1, y);
    h.mouse(MouseKind::Down, x + 1, y);
    h.mouse(MouseKind::Drag, x + 7, y);
    h.mouse(MouseKind::Up, x + 7, y);
    assert_eq!(selected_on_row(&h, y), "alpha beta", "{}", h.text());
}

/// A background screen whose field reports its blur in the status bar.
struct BlurReporter;

const REPORTED: junie_tui::core::id::WidgetId = junie_tui::core::id::WidgetId::of("test.reported");

impl crate::screens::Screen for BlurReporter {
    fn on_key(
        &mut self,
        _key: &junie_tui::core::event::Key,
        _w: &mut crate::sim::world::World,
        _cx: &mut crate::screens::Cx,
    ) -> junie_tui::core::event::Outcome {
        junie_tui::core::event::Outcome::Ignored
    }
    fn on_blur(
        &mut self,
        ch: junie_tui::core::focus::FocusChange,
        _w: &mut crate::sim::world::World,
        cx: &mut crate::screens::Cx,
    ) -> junie_tui::core::event::Outcome {
        if ch.from == REPORTED {
            cx.status("Background field saved");
            return junie_tui::core::event::Outcome::Changed;
        }
        junie_tui::core::event::Outcome::Ignored
    }
    fn render(
        &mut self,
        _area: ratatui::layout::Rect,
        _buf: &mut ratatui::buffer::Buffer,
        _ctx: &mut junie_tui::ui::ctx::RenderCtx,
        _w: &crate::sim::world::World,
    ) {
    }
    fn hints(
        &self,
        _focus: Option<junie_tui::core::id::WidgetId>,
        _w: &crate::sim::world::World,
    ) -> Vec<junie_tui::widgets::keyhint::Hint> {
        vec![]
    }
    fn crumb(&self, _w: &crate::sim::world::World) -> String {
        "Reporter".into()
    }
    fn primary_focus(&self) -> Option<junie_tui::core::id::WidgetId> {
        None
    }
}

#[test]
fn a_status_raised_by_blur_in_a_background_tab_shows_when_it_activates() {
    let mut h = H::new(Scenario::ActivitiesMulti, Motion::Reduced, 0, 120, 40);
    assert!(h.app.tabs.len() > 2, "several tabs");
    let bg = 1;
    assert_ne!(h.app.active, bg);
    h.app.tabs[bg].stack.push(Box::new(BlurReporter));
    // the background field loses focus: its blur hook reports
    h.app.focus.focus(REPORTED);
    h.app
        .focus
        .focus(junie_tui::core::id::WidgetId::of("test.elsewhere"));
    h.app.handle(junie_tui::core::event::Input::Tick);
    h.draw();
    assert!(
        !h.text().contains("Background field saved"),
        "not over another tab's page\n{}",
        h.text()
    );
    // activating the tab delivers it
    h.alt(KeyCode::Char('1'));
    assert_eq!(h.app.active, bg);
    assert!(h.text().contains("Background field saved"), "{}", h.text());
}
