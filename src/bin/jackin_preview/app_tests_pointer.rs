//! Pointer journeys through the shared router (plan S6): every gesture
//! reaches the widget that owns the part under the pointer.

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::{Part, WidgetId};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};

/// The area of the latest region of `owner`'s `part`.
fn part_area(h: &H, owner: WidgetId, part: Part) -> Option<Rect> {
    h.app
        .hits
        .regions()
        .iter()
        .rev()
        .find(|r| r.owner == owner && r.part.part == part)
        .map(|r| r.area)
}

/// The manager, after the intro.
fn manager() -> H {
    let h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Manager, "{}", h.text());
    h
}

/// The cell of `needle` on the footer row.
fn footer_at(h: &H, needle: &str) -> (u16, u16) {
    let text = h.text();
    let y = text.lines().count() as u16 - 1;
    let line = text.lines().last().unwrap_or_default();
    let at = line
        .find(needle)
        .unwrap_or_else(|| panic!("the footer names {needle:?}\n{text}"));
    (line[..at].chars().count() as u16, y)
}

/// M1: a footer hint naming one key is a click target that runs the key.
#[test]
fn a_click_on_a_footer_hint_runs_its_key() {
    let mut h = manager();
    let (x, y) = footer_at(&h, "u Usage");
    h.click(x, y);
    assert_eq!(
        h.app.route,
        Route::Usage,
        "the click on u opened Usage\n{}",
        h.text()
    );
}

/// M17 (routing): a press on the help overlay's scrollbar pages it; the
/// app delivered no press to the overlay's bar.
#[test]
fn a_press_on_the_help_overlay_scrollbar_pages_it() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 100, 20);
    h.key(KeyCode::Char('?'));
    let help = h
        .app
        .top_layer::<junie_tui::widgets::help::HelpOverlay>()
        .map(|o| o.id)
        .unwrap_or_else(|| panic!("help is open\n{}", h.text()));
    let bar = part_area(&h, help, Part::TRACK)
        .unwrap_or_else(|| panic!("the overlay shows a scrollbar\n{}", h.text()));
    let offset = |h: &H| {
        h.app
            .top_layer::<junie_tui::widgets::help::HelpOverlay>()
            .expect("help is open")
            .scroll
            .offset
    };
    let before = offset(&h);
    h.mouse(MouseKind::Down, bar.x, bar.bottom() - 1);
    assert!(offset(&h) > before, "the press paged help\n{}", h.text());
    h.mouse(MouseKind::Up, bar.x, bar.bottom() - 1);
}
