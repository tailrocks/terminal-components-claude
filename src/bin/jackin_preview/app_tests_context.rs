//! Context menus through the real app: a right-click on a row opens its
//! menu at the pointer, the context keys (Shift+F10, the Menu key) open it
//! under the cursor row, and a right-click outside an open menu closes it
//! and reopens at the new spot.

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use junie_tui::core::event::{Input, Key, MouseKind};
use junie_tui::core::id::WidgetId;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::accounts::{MENU as ACCOUNTS_MENU, Sel};
use crate::screens::manager::{MENU, RowKey};
use junie_tui::widgets::menu::ContextMenu;

fn shift_f10(h: &mut H) {
    h.app.handle(Input::Key(Key {
        code: KeyCode::F(10),
        mods: KeyModifiers::SHIFT,
    }));
    h.draw();
}

fn row(h: &H, y: u16) -> String {
    h.text().lines().nth(y as usize).unwrap_or("").to_owned()
}

/// `needle` in the left column (`x < max_x`): a tree row, not the crumb or
/// the detail pane.
fn find_left(h: &H, needle: &str, max_x: u16) -> Option<(u16, u16)> {
    let text = h.text();
    text.lines().enumerate().find_map(|(y, line)| {
        let cells: Vec<char> = line.chars().collect();
        let want: Vec<char> = needle.chars().collect();
        (0..cells.len().saturating_sub(want.len()))
            .find(|&x| x < max_x as usize && cells[x..x + want.len()] == want[..])
            .map(|x| (x as u16, y as u16))
    })
}

/// Click item `i` of the open menu `id` where it is drawn.
fn choose(h: &mut H, id: WidgetId, i: usize) {
    let r = h.app.hits.area_of(id.child(i)).expect("the item is drawn");
    h.click(r.x + 2, r.y);
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

// ------------------------------------------------------------- manager

fn manager() -> H {
    let h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Manager, "{}", h.text());
    h
}

fn tree_row(h: &H, needle: &str) -> (u16, u16) {
    find_left(h, needle, 38).unwrap_or_else(|| panic!("{needle} in the tree: {}", h.text()))
}

/// The open manager menu's rectangle.
fn manager_menu(h: &H) -> Rect {
    h.app
        .screens
        .manager
        .menu
        .menu()
        .expect("the manager menu is open")
        .area
}

#[test]
fn manager_row_menu_opens_at_the_pointer_and_runs_the_row_action() {
    let mut h = manager();
    let (x, y) = tree_row(&h, "infra-control-plane");
    h.mouse(MouseKind::Secondary, x + 3, y);
    let area = manager_menu(&h);
    assert_eq!((area.x, area.y), (x + 3, y + 1), "at the pointer");
    let text = h.text();
    assert!(
        text.contains("Prewarm") && text.contains("Delete…") && text.contains("Open on GitHub"),
        "{text}"
    );
    assert!(
        row(&h, area.y + 1).contains("infra-control-plane"),
        "the title names the row: {text}"
    );
    assert!(
        matches!(h.app.screens.manager.selected, RowKey::Workspace(_)),
        "the row the menu is about is selected"
    );
    let menu = h.app.screens.manager.menu.menu().unwrap();
    assert!(menu.items[4].danger, "Delete… is a danger item");
    assert_eq!(
        menu.items[1].shortcut.as_deref(),
        Some("e"),
        "the row's key is shown"
    );
    // Edit: the same path as the e key
    choose(&mut h, MENU, 1);
    assert_eq!(h.app.route, Route::Editor, "{}", h.text());
}

#[test]
fn manager_instance_menu_disables_what_cannot_run_and_inspects() {
    let mut h = manager();
    // expand payments-platform and right-click its preserved instance
    let (x, y) = tree_row(&h, "payments-platform");
    h.click(x, y);
    h.key(KeyCode::Right);
    let (ix, iy) = tree_row(&h, "c41e");
    h.mouse(MouseKind::Secondary, ix, iy);
    let menu = h.app.screens.manager.menu.menu().unwrap();
    let labels: Vec<&str> = menu.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Restore",
            "New session…",
            "Shell",
            "Inspect",
            "Stop…",
            "Purge…"
        ]
    );
    assert!(
        menu.items[1].disabled && menu.items[2].disabled && menu.items[4].disabled,
        "a stopped instance has no session, shell or stop: {labels:?}"
    );
    assert!(menu.items[5].danger);
    choose(&mut h, MENU, 3);
    assert!(h.text().contains("Container c41e"), "{}", h.text());
}

#[test]
fn manager_shift_f10_opens_under_the_cursor_row() {
    let mut h = manager();
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    let (_, y) = tree_row(&h, "release-automation");
    shift_f10(&mut h);
    assert!(
        h.text().contains("Prewarm"),
        "Shift+F10 opens the row menu, not the menu bar: {}",
        h.text()
    );
    assert_eq!(manager_menu(&h).y, y + 1, "under the cursor row");
    // the menu owns the keyboard: a global letter does not leave the page
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app.route, Route::Manager);
    h.key(KeyCode::Esc);
    assert!(h.app.screens.manager.menu.menu().is_none());
    assert_eq!(h.app.route, Route::Manager, "Esc closes only the menu");
    // the Menu key opens it too; Enter runs Launch…
    h.key(KeyCode::Menu);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Launch · choose Agent"), "{}", h.text());
}

#[test]
fn manager_right_click_outside_a_menu_reopens_on_the_new_row() {
    let mut h = manager();
    let (x, y) = tree_row(&h, "infra-control-plane");
    h.mouse(MouseKind::Secondary, x + 8, y);
    // payments-platform, above the open menu, sits under its backdrop
    let (px, py) = tree_row(&h, "payments-platform");
    h.mouse(MouseKind::Secondary, px + 2, py);
    let area = manager_menu(&h);
    assert_eq!(
        (area.x, area.y),
        (px + 2, py + 1),
        "reopened at the new spot"
    );
    assert_eq!(h.text().matches("Open on GitHub").count(), 1, "one menu");
    assert!(row(&h, area.y + 1).contains("payments-platform"));

    // the host menu bar is a popup too: a right-click on a row closes it
    // and opens the row's menu
    h.key(KeyCode::Esc);
    let (cx, cy) = tree_row(&h, "customer-portal");
    h.key(KeyCode::F(10));
    assert!(h.app.host_menu.is_open());
    // the row's start, left of the open File menu
    h.mouse(MouseKind::Secondary, cx, cy);
    assert!(!h.app.host_menu.is_open(), "the menu bar closed");
    assert_eq!(manager_menu(&h).y, cy + 1, "the row menu opened");
}

// ------------------------------------------------------------ accounts

fn accounts() -> H {
    let h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route, Route::Accounts, "{}", h.text());
    h
}

fn accounts_menu(h: &H) -> &ContextMenu {
    h.app
        .screens
        .accounts
        .menu
        .menu()
        .expect("the accounts menu is open")
}

fn selected_account(h: &H) -> String {
    match &h.app.screens.accounts.selected {
        Sel::Account(id) => id.clone(),
        s => panic!("not an account: {s:?}"),
    }
}

#[test]
fn accounts_row_menu_sets_the_default_and_explains_what_cannot_run() {
    let mut h = accounts();
    let (x, y) = find_left(&h, "Experiments", 40).unwrap();
    h.mouse(MouseKind::Secondary, x, y);
    let area = accounts_menu(&h).area;
    assert_eq!((area.x, area.y), (x, y + 1), "at the pointer");
    assert!(row(&h, area.y + 1).contains("Experiments"), "titled");
    let labels: Vec<String> = accounts_menu(&h)
        .items
        .iter()
        .map(|i| i.label.clone())
        .collect();
    assert_eq!(
        labels,
        [
            "Refresh",
            "Validate",
            "Show usage",
            "Edit…",
            "Set default",
            "Disable",
            "Remove…"
        ]
    );
    assert!(accounts_menu(&h).items[6].danger);
    let id = selected_account(&h);
    assert!(!h.app.world.accounts.get(&id).unwrap().default_for_provider);
    // Set default: the Space key's path
    choose(&mut h, ACCOUNTS_MENU, 4);
    assert!(h.app.screens.accounts.menu.menu().is_none(), "closed");
    assert!(
        h.app.world.accounts.get(&id).unwrap().default_for_provider,
        "{}",
        h.text()
    );

    // the provider default cannot become the default again
    let (px, py) = find_left(&h, "Personal", 40).unwrap();
    h.mouse(MouseKind::Secondary, px, py);
    let set_default = &accounts_menu(&h).items[4];
    assert!(set_default.disabled, "{set_default:?}");
    h.key(KeyCode::Esc);
    // a discovered account is read-only: no edit, no remove
    let (dx, dy) = find_left(&h, "discovered", 40).unwrap();
    h.mouse(MouseKind::Secondary, dx, dy);
    let m = accounts_menu(&h);
    assert!(!m.items[0].disabled, "Refresh still runs");
    assert!(m.items[3].disabled && m.items[6].disabled, "{:?}", m.items);
}

#[test]
fn accounts_shift_f10_opens_under_the_cursor_row_and_folds() {
    let mut h = accounts();
    h.key(KeyCode::Down);
    let (_, y) = find_left(&h, "Claude", 40).unwrap();
    shift_f10(&mut h);
    assert_eq!(accounts_menu(&h).area.y, y + 1, "under the cursor row");
    assert!(h.text().contains("Refresh provider"), "{}", h.text());
    // the last item collapses the provider
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.app.screens.accounts.menu.menu().is_none());
    assert!(find_left(&h, "Personal", 40).is_none(), "{}", h.text());
}

#[test]
fn accounts_right_click_outside_a_menu_reopens_on_the_new_row() {
    let mut h = accounts();
    let (x, y) = find_left(&h, "Experiments", 40).unwrap();
    h.mouse(MouseKind::Secondary, x, y);
    let (px, py) = find_left(&h, "Personal", 40).unwrap();
    h.mouse(MouseKind::Secondary, px, py);
    let area = accounts_menu(&h).area;
    assert_eq!((area.x, area.y), (px, py + 1), "reopened at the new spot");
    assert!(row(&h, area.y + 1).contains("Personal"));
    assert_eq!(h.text().matches("Show usage").count(), 1, "one menu");
}
