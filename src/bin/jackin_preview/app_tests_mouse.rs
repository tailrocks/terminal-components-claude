//! Mouse-only navigation in jackin (docs/audits/design-review-2026-10/mouse.md):
//! every state is reachable, and every view can be left, by pointer alone.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use junie_tui::core::event::MouseKind;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::editor::ROLES;

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
