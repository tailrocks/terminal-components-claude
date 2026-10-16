//! Mouse-only navigation in jackin (docs/audits/design-review-2026-10/mouse.md):
//! every state is reachable, and every view can be left, by pointer alone.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use junie_tui::core::event::MouseKind;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::editor::ROLES;
use junie_tui::ui::modal::{CLOSE, SURFACE};

/// A screen cell no control claims (the dimmed page around a modal).
fn outside(h: &H) -> Position {
    let area = h.term.backend().buffer().area;
    for y in 1..area.height.saturating_sub(1) {
        for x in 0..area.width {
            let p = Position::new(x, y);
            if h.app.hits.hit(p).is_none() {
                return p;
            }
        }
    }
    panic!("no free cell outside the modal");
}

fn hard_editor_roles() -> H {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    for _ in 0..8 {
        h.ticks(3);
        if h.app.route == Route::Manager {
            break;
        }
        h.key(KeyCode::Enter);
    }
    assert!(h.app.world.roles.len() > 100);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('3'));
    h
}

// mouse #4: the wheel scrolls a long list's view and the next frame keeps it
#[test]
fn the_wheel_scrolls_the_roles_list_without_snapping_back_to_the_cursor() {
    let mut h = hard_editor_roles();
    let area = h
        .app
        .hits
        .area_of(ROLES)
        .expect("the Roles list must be on screen");
    let at = Position::new(area.x + 2, area.y + 1);
    let first_row = |t: &str| t.lines().nth(area.y as usize).unwrap_or("").to_owned();
    let top = first_row(&h.text());
    for _ in 0..3 {
        h.mouse(MouseKind::WheelDown, at.x, at.y);
    }
    let scrolled = first_row(&h.text());
    assert_ne!(scrolled, top, "the wheel did not move the Roles list");
    // a repaint (a tick, a hover) keeps the view where the wheel left it
    h.ticks(2);
    h.mouse(MouseKind::Move, at.x, at.y);
    assert_eq!(h.app.hits.area_of(ROLES), Some(area), "the list moved");
    assert_eq!(
        first_row(&h.text()),
        scrolled,
        "the view snapped back to the cursor"
    );
}

// mouse #13: the help overlay leaves the page around it to click, and has
// a close button in its frame and a Done button
#[test]
fn help_closes_by_its_close_button_its_done_button_and_a_click_beside_it() {
    let open = || {
        let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
        h.key(KeyCode::Char('?'));
        assert!(h.text().contains("Keyboard shortcuts"), "{}", h.text());
        h
    };
    let mut h = open();
    let frame = h.app.hits.area_of(SURFACE).expect("help frame");
    assert!(
        frame.width <= 96 && frame.height <= 32,
        "help covers the page: {frame:?}"
    );
    let x = h
        .app
        .hits
        .area_of(CLOSE)
        .expect("help draws a close button");
    assert_eq!(x.y, frame.y, "the close button sits in the top border");
    assert!(h.text().lines().nth(x.y as usize).unwrap().contains('×'));
    h.click(x.x + 1, x.y);
    assert!(!h.text().contains("Keyboard shortcuts"), "× left help open");

    let mut h = open();
    let (dx, dy) = h.find("Done").expect("help draws a Done button");
    h.click(dx, dy);
    assert!(
        !h.text().contains("Keyboard shortcuts"),
        "Done left help open"
    );

    let mut h = open();
    let p = outside(&h);
    h.click(p.x, p.y);
    assert!(
        !h.text().contains("Keyboard shortcuts"),
        "a click beside help left it open"
    );
}

fn new_account_form() -> H {
    let mut h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Accounts);
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("New account"), "{}", h.text());
    h
}

// mouse #13: a form (Dismiss::ESCAPE) closes by the × in its frame
#[test]
fn a_form_closes_by_the_close_button_in_its_frame() {
    let mut h = new_account_form();
    let x = h
        .app
        .hits
        .area_of(CLOSE)
        .expect("form draws a close button");
    h.click(x.x + 1, x.y);
    assert!(!h.text().contains("New account"), "{}", h.text());
    assert_eq!(h.app.route, Route::Accounts);
}

// mouse #24: an outside click the policy refuses flashes the frame
#[test]
fn a_refused_outside_click_flashes_the_form_frame() {
    let mut h = new_account_form();
    let frame = h.app.hits.area_of(SURFACE).expect("form frame");
    let corner = |h: &H| h.term.backend().buffer()[(frame.x, frame.y)].fg;
    let rest = corner(&h);
    let p = outside(&h);
    h.mouse(MouseKind::Down, p.x, p.y);
    h.mouse(MouseKind::Up, p.x, p.y);
    assert!(
        h.text().contains("New account"),
        "the refused click closed it"
    );
    assert_eq!(corner(&h), h.app.theme.warning, "the frame did not flash");
    assert_ne!(rest, h.app.theme.warning);
    // the flash ends
    h.ticks(20);
    assert_eq!(corner(&h), rest);
}
