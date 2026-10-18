use junie_tui::core::event::MouseKind;
use junie_tui::core::id::{Part, WidgetId};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};

fn part_area(h: &H, owner: WidgetId, part: Part) -> Option<Rect> {
    h.app
        .shell
        .hits
        .regions()
        .iter()
        .rev()
        .find(|r| r.owner == owner && r.part.part == part)
        .map(|r| r.area)
}

fn manager() -> H {
    let h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Manager, "{}", h.text());
    h
}

fn footer_at(h: &H, needle: &str) -> (u16, u16) {
    let text = h.text();
    let y = text.lines().count() as u16 - 1;
    let line = text.lines().last().unwrap_or_default();
    let at = line
        .find(needle)
        .unwrap_or_else(|| panic!("the footer names {needle:?}\n{text}"));
    (line[..at].chars().count() as u16, y)
}

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

#[test]
fn the_wheel_over_the_session_strip_moves_its_window() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 72, 24);
    assert_eq!(h.app.route, Route::Capsule, "{}", h.text());

    for d in h.app.world.daemons.values_mut() {
        if let Some(t) = d.tabs.last().cloned() {
            d.tabs.extend(std::iter::repeat_n(t, 6));
        }
    }
    h.draw();
    let tab = part_area(&h, crate::screens::capsule::STRIP, Part::TAB)
        .unwrap_or_else(|| panic!("the session strip shows its tabs\n{}", h.text()));
    let row = |h: &H| {
        h.text()
            .lines()
            .nth(tab.y as usize)
            .unwrap_or_default()
            .to_owned()
    };
    let before = row(&h);
    h.mouse(MouseKind::WheelDown, tab.x, tab.y);
    assert_ne!(row(&h), before, "the wheel moved the strip\n{}", h.text());
}

#[test]
fn the_host_menu_opens_on_the_press_and_a_click_away_closes_it() {
    let mut h = manager();
    let (x, y) = h.find("File").expect("the host menu names File");
    h.mouse(MouseKind::Down, x, y);
    assert!(
        h.app.host_menu.is_open(),
        "the press opened File\n{}",
        h.text()
    );
    h.mouse(MouseKind::Up, x, y);
    assert!(h.app.host_menu.is_open(), "the click kept File open");
    h.click(h.app.shell.size.0 - 3, h.app.shell.size.1 / 2);
    assert!(!h.app.host_menu.is_open(), "a click away closed the menu");
}

#[test]
fn the_focused_host_menu_bar_opens_with_enter() {
    let mut h = manager();
    h.app.shell.focus.focus(WidgetId::of("host.menu"));
    h.key(KeyCode::Enter);
    assert!(h.app.host_menu.is_open(), "Enter opened File\n{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.app.host_menu.is_open());
}
