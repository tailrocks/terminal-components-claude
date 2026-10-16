//! Context menus through the real App: a right click on a row opens its
//! menu at the pointer, Shift+F10 and the Menu key open it below the cursor
//! row, and a right click outside an open menu (or the menu bar) closes it
//! and opens the new menu at the new spot.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};

use junie_tui::core::event::{Input, Key, MouseKind};
use junie_tui::core::id::WidgetId;

use crate::app_tests::H;
use crate::app_tests_parity::open;
use crate::scenario::{Motion, Scenario};
use crate::screens::{files, finder};

const ALTERNATIVES: WidgetId = WidgetId::of("alternatives");
const FILE_ACTIONS: WidgetId = WidgetId::of("files.actions");

pub(crate) fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

pub(crate) fn menu_key(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::Menu,
        mods: KeyModifiers::NONE,
    }));
    h.draw();
}

pub(crate) fn right_click(h: &mut H, x: u16, y: u16) {
    h.mouse(MouseKind::Secondary, x, y);
}

/// The open menu's rectangle (its hit area as drawn this frame).
pub(crate) fn menu_area(h: &H, id: WidgetId) -> Rect {
    h.app
        .hits
        .area_of(id)
        .unwrap_or_else(|| panic!("menu {id:?} not drawn\n{}", h.text()))
}

/// The menu's top-left corner sits right at the pointer (below it).
pub(crate) fn assert_at_pointer(h: &H, id: WidgetId, x: u16, y: u16) {
    let a = menu_area(h, id);
    assert_eq!(
        (a.x, a.y),
        (x, y + 1),
        "menu not at the pointer\n{}",
        h.text()
    );
}

/// The menu sits right below row `y`.
pub(crate) fn assert_below_row(h: &H, id: WidgetId, y: u16) {
    let a = menu_area(h, id);
    assert_eq!(a.y, y + 1, "menu not below the cursor row\n{}", h.text());
}

/// Choose the menu row labelled `label` with the pointer.
pub(crate) fn choose(h: &mut H, label: &str) {
    let (x, y) = h
        .find(label)
        .unwrap_or_else(|| panic!("no {label:?} in the menu\n{}", h.text()));
    h.click(x + 1, y);
}

fn rust() -> H {
    H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40)
}

#[test]
fn a_right_click_on_a_finder_row_opens_its_alternatives_at_the_pointer() {
    let mut h = rust();
    let (x, y) = h.find("Run tests").unwrap();
    right_click(&mut h, x + 12, y);
    assert_eq!(h.app.focus.current(), Some(ALTERNATIVES));
    assert_at_pointer(&h, ALTERNATIVES, x + 12, y);
    assert!(h.text().contains("Arguments…"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
    assert_eq!(h.app.focus.current(), Some(finder::FINDER));
}

#[test]
fn shift_f10_and_the_menu_key_open_the_finder_alternatives_below_the_cursor_row() {
    let mut h = rust();
    let (x, y) = h.find("Run tests").unwrap();
    h.click(x + 2, y);
    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(ALTERNATIVES), "{}", h.text());
    assert_below_row(&h, ALTERNATIVES, y);
    assert!(!h.app.menu_open(), "Shift+F10 never opens the menu bar");
    h.key(KeyCode::Esc);
    assert_eq!(h.app.focus.current(), Some(finder::FINDER));
    menu_key(&mut h);
    assert_eq!(h.app.focus.current(), Some(ALTERNATIVES), "{}", h.text());
    assert_below_row(&h, ALTERNATIVES, y);
    h.key(KeyCode::Esc);
    // plain F10 still opens the menu bar
    h.key(KeyCode::F(10));
    assert!(h.app.menu_open());
}

#[test]
fn a_right_click_outside_an_open_menu_closes_it_and_opens_the_new_menu_there() {
    let mut h = rust();
    let (x, y) = h.find("Run tests").unwrap();
    // every finder row on screen, from the frame before the menu opens
    let rows: Vec<u16> = (1..h.app.size.1 - 2)
        .filter(|&yy| yy != y)
        .filter(|&yy| {
            h.app
                .hits
                .hit(Position::new(x + 12, yy))
                .is_some_and(|hit| (0..200).any(|i| finder::FINDER.child(i) == hit))
        })
        .collect();
    right_click(&mut h, x + 12, y);
    assert_eq!(h.app.modals.len(), 1);
    let first = menu_area(&h, ALTERNATIVES);
    // another row, clear of the open menu
    let oy = *rows
        .iter()
        .find(|&&yy| yy < first.y || yy >= first.bottom())
        .unwrap_or_else(|| panic!("no other finder row\n{}", h.text()));
    let ox = x + 12;
    // the open menu's own backdrop is what is under the pointer now
    right_click(&mut h, ox, oy);
    assert_eq!(h.app.modals.len(), 1, "one menu, not a stack");
    assert_eq!(h.app.focus.current(), Some(ALTERNATIVES));
    assert_at_pointer(&h, ALTERNATIVES, ox, oy);
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty());
}

#[test]
fn a_right_click_while_the_menu_bar_is_open_closes_it_and_opens_the_row_menu() {
    let mut h = rust();
    let (x, y) = h.find("Run tests").unwrap();
    h.key(KeyCode::F(10));
    assert!(h.app.menu_open());
    // to the right of the open File menu
    right_click(&mut h, x + 60, y);
    assert!(!h.app.menu_open(), "the menu bar closed\n{}", h.text());
    assert_eq!(h.app.focus.current(), Some(ALTERNATIVES), "{}", h.text());
    assert_at_pointer(&h, ALTERNATIVES, x + 60, y);
}

#[test]
fn a_right_click_outside_the_help_modal_stays_swallowed() {
    let mut h = rust();
    h.key(KeyCode::F(1));
    assert_eq!(h.app.modals.len(), 1);
    right_click(&mut h, 1, 2);
    assert_eq!(h.app.modals.len(), 1, "{}", h.text());
}

fn files_page() -> H {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 120, 40);
    open(&mut h, "Browse ~/work/site");
    h.ticks(8);
    h
}

#[test]
fn files_rows_open_their_actions_at_the_pointer_and_under_the_cursor() {
    let mut h = files_page();
    let (fx, fy) = h.find("big.log").expect("file row");
    right_click(&mut h, fx + 6, fy);
    assert_eq!(h.app.focus.current(), Some(FILE_ACTIONS));
    assert_at_pointer(&h, FILE_ACTIONS, fx + 6, fy);
    let copies = h.app.clipboard_gen;
    choose(&mut h, "Copy full path");
    assert!(h.app.modals.is_empty());
    assert_eq!(h.app.clipboard_gen, copies + 1, "{}", h.last_row());
    assert!(
        h.app
            .world
            .clipboard
            .as_deref()
            .is_some_and(|c| c.ends_with("big.log")),
        "{:?}",
        h.app.world.clipboard
    );
    assert_eq!(h.app.focus.current(), Some(files::LIST));

    shift_f10(&mut h);
    assert_eq!(h.app.focus.current(), Some(FILE_ACTIONS), "{}", h.text());
    assert_below_row(&h, FILE_ACTIONS, fy);
    h.key(KeyCode::Esc);
    assert_eq!(h.app.focus.current(), Some(files::LIST));
}
