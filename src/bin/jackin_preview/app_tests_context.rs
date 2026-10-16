//! Context menus through the real app: a right-click on a row opens its
//! menu at the pointer, the context keys (Shift+F10, the Menu key) open it
//! under the cursor row, and a right-click outside an open menu closes it
//! and reopens at the new spot.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use junie_tui::core::event::{Input, Key, MouseKind};

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};

fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

#[test]
fn capsule_tab_menu_reopens_on_another_tab_and_opens_with_shift_f10() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Capsule);
    let (sx, sy) = h.find("Shell").expect("the Shell tab");
    h.mouse(MouseKind::Secondary, sx, sy);
    let first = h.find("Change title…").expect("the Shell tab's menu");
    // the right-click lands on the open menu's backdrop: it closes the
    // menu and opens the docs tab's menu under the docs tab
    let (dx, dy) = h.find("docs").expect("the docs tab");
    h.mouse(MouseKind::Secondary, dx, dy);
    let second = h.find("Change title…").expect("the menu reopens");
    assert_eq!(second.1, first.1, "same row under the strip");
    assert!(
        second.0 > first.0,
        "moved to the docs tab: {first:?} → {second:?}"
    );
    assert_eq!(h.text().matches("Change title…").count(), 1, "one menu");
    assert!(h.text().contains("docs"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Change title…"));

    // Shift+F10 opens the active tab's menu, not the menu bar
    shift_f10(&mut h);
    assert!(h.text().contains("Change title…"), "{}", h.text());
    assert!(!h.text().contains("Split below"), "{}", h.text());
    h.key(KeyCode::Esc);
    // plain F10 still opens the menu bar
    h.key(KeyCode::F(10));
    assert!(h.text().contains("Split below"), "{}", h.text());
}
