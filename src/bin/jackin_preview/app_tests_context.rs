use crate::pages::Pages;
use junie_tui::runtime::Application;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use junie_tui::core::event::{Input, Key, MouseKind};
use junie_tui::core::id::WidgetId;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::Go;
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

fn choose(h: &mut H, id: WidgetId, i: usize) {
    let r = h
        .app
        .shell
        .hits
        .area_of(id.child(i))
        .expect("the item is drawn");
    h.click(r.x + 2, r.y);
}

#[test]
fn capsule_tab_menu_reopens_on_another_tab_and_opens_with_shift_f10() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Capsule);
    let (sx, sy) = h.find("Shell").expect("the Shell tab");
    h.mouse(MouseKind::Secondary, sx, sy);
    let first = h.find("Change title…").expect("the Shell tab's menu");

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

    shift_f10(&mut h);
    let t = h.text();
    assert!(t.contains("Copy selection"), "{t}");
    let (cx, cy) = h.find("Copy selection").unwrap();
    h.mouse(MouseKind::Move, cx + 1, cy);
    let t = h.text();
    assert!(t.contains("Select text first"), "nothing is selected\n{t}");
    assert!(!t.contains("Change title…"), "{t}");
    assert!(!t.contains("Key reference"), "not the menu bar\n{t}");
    h.key(KeyCode::Esc);

    h.ctrl('b');
    h.key(KeyCode::Char('m'));
    assert!(h.text().contains("Change title…"), "{}", h.text());
    h.key(KeyCode::Esc);

    h.key(KeyCode::F(10));
    assert!(h.text().contains("Split below"), "{}", h.text());
}

#[test]
fn capsule_pane_right_click_opens_the_pane_menu_at_the_pointer() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let (sx, sy) = h.find("Shell").expect("the Shell tab");

    let (x, y) = (sx + 10, sy + 6);
    h.mouse(MouseKind::Secondary, x, y);
    let (mx, my) = h.find("Copy selection").expect("the pane menu");

    assert_eq!(my, y + 2, "below the pointer\n{}", h.text());
    assert!(
        mx >= x.saturating_sub(2) && mx <= x + 3,
        "at the pointer: {mx}"
    );
    assert!(h.text().contains("Close pane"));
}

fn manager() -> H {
    let h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Manager, "{}", h.text());
    h
}

fn tree_row(h: &H, needle: &str) -> (u16, u16) {
    find_left(h, needle, 38).unwrap_or_else(|| panic!("{needle} in the tree: {}", h.text()))
}

fn manager_menu(h: &H) -> Rect {
    h.app
        .router
        .manager()
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
        matches!(h.app.router.manager().selected, RowKey::Workspace(_)),
        "the row the menu is about is selected"
    );
    let menu = h.app.router.manager().menu.menu().unwrap();
    assert!(menu.items[4].danger, "Delete… is a danger item");
    assert_eq!(
        menu.items[1].shortcut.as_deref(),
        Some("e"),
        "the row's key is shown"
    );

    choose(&mut h, MENU, 1);
    assert_eq!(h.app.route(), Route::Editor, "{}", h.text());
}

#[test]
fn manager_instance_menu_disables_what_cannot_run_and_inspects() {
    let mut h = manager();

    let (x, y) = tree_row(&h, "payments-platform");
    h.click(x, y);
    h.key(KeyCode::Right);
    let (ix, iy) = tree_row(&h, "c41e");
    h.mouse(MouseKind::Secondary, ix, iy);
    let menu = h.app.router.manager().menu.menu().unwrap();
    let labels: Vec<&str> = menu.items.iter().map(|i| i.item.label.as_str()).collect();
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
        menu.items[1].item.disabled && menu.items[2].item.disabled && menu.items[4].item.disabled,
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

    h.key(KeyCode::Char('s'));
    assert_eq!(h.app.route(), Route::Manager);
    h.key(KeyCode::Esc);
    assert!(h.app.router.manager().menu.menu().is_none());
    assert_eq!(h.app.route(), Route::Manager, "Esc closes only the menu");

    h.key(KeyCode::Menu);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Launch · choose Agent"), "{}", h.text());
}

#[test]
fn manager_action_buttons_and_detail_open_the_selected_rows_menu() {
    use crate::screens::manager::DETAIL;
    let mut h = manager();
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    let (_, row_y) = tree_row(&h, "release-automation");

    let prewarm = DETAIL.sub("prewarm");
    h.app.shell.focus.focus(prewarm);
    h.draw();
    let b = h
        .app
        .shell
        .hits
        .area_of(prewarm)
        .expect("the Prewarm button");
    shift_f10(&mut h);
    let area = manager_menu(&h);

    assert!(
        area.y == b.y + 1 || area.bottom() == b.y,
        "at the focused button {b:?}: {area:?}\n{}",
        h.text()
    );
    assert_ne!(area.y, row_y + 1, "not under the tree row");
    h.key(KeyCode::Esc);
    assert_eq!(h.app.shell.focus.current(), Some(prewarm), "focus returns");

    h.mouse(MouseKind::Secondary, b.x + 1, b.y);
    let area = manager_menu(&h);
    assert_eq!(area.x, b.x + 1, "at the pointer");
    assert!(
        area.y == b.y + 1 || area.bottom() == b.y,
        "at the pointer row: {area:?}"
    );
    assert!(h.text().contains("Prewarm"), "{}", h.text());

    h.key(KeyCode::Enter);
    assert!(h.text().contains("Launch · choose Agent"), "{}", h.text());
}

#[test]
fn manager_right_click_outside_a_menu_reopens_on_the_new_row() {
    let mut h = manager();
    let (x, y) = tree_row(&h, "infra-control-plane");
    h.mouse(MouseKind::Secondary, x + 8, y);

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

    h.key(KeyCode::Esc);
    let (cx, cy) = tree_row(&h, "customer-portal");
    h.key(KeyCode::F(10));
    assert!(h.app.host_menu.is_open());

    h.mouse(MouseKind::Secondary, cx, cy);
    assert!(!h.app.host_menu.is_open(), "the menu bar closed");
    assert_eq!(manager_menu(&h).y, cy + 1, "the row menu opened");
}

fn accounts() -> H {
    let h = H::new(Scenario::AccountsMixed, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Accounts, "{}", h.text());
    h
}

fn accounts_menu(h: &H) -> &ContextMenu {
    h.app
        .router
        .accounts()
        .menu
        .menu()
        .expect("the accounts menu is open")
}

fn selected_account(h: &H) -> String {
    match &h.app.router.accounts().selected {
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
        .map(|i| i.item.label.clone())
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

    choose(&mut h, ACCOUNTS_MENU, 4);
    assert!(h.app.router.accounts().menu.menu().is_none(), "closed");
    assert!(
        h.app.world.accounts.get(&id).unwrap().default_for_provider,
        "{}",
        h.text()
    );

    let (px, py) = find_left(&h, "Personal", 40).unwrap();
    h.mouse(MouseKind::Secondary, px, py);
    let set_default = &accounts_menu(&h).items[4];
    assert!(set_default.item.disabled, "{set_default:?}");
    h.key(KeyCode::Esc);

    let (dx, dy) = find_left(&h, "discovered", 40).unwrap();
    h.mouse(MouseKind::Secondary, dx, dy);
    let m = accounts_menu(&h);
    assert!(!m.items[0].item.disabled, "Refresh still runs");
    assert!(
        m.items[3].item.disabled && m.items[6].item.disabled,
        "{:?}",
        m.items
    );
}

#[test]
fn accounts_shift_f10_opens_under_the_cursor_row_and_folds() {
    let mut h = accounts();
    h.key(KeyCode::Down);
    let (_, y) = find_left(&h, "Claude", 40).unwrap();
    shift_f10(&mut h);
    assert_eq!(accounts_menu(&h).area.y, y + 1, "under the cursor row");
    assert!(h.text().contains("Refresh provider"), "{}", h.text());

    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.app.router.accounts().menu.menu().is_none());
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

fn editor_mounts() -> H {
    let mut h = manager();
    h.app.go(Go::Editor {
        workspace: Some(1),
        pending: None,
    });
    h.draw();
    let (x, y) = h.find("Mounts").unwrap();
    h.click(x, y);
    assert!(h.text().contains("/workspace/libs"), "{}", h.text());
    h
}

fn editor_menu(h: &H) -> &ContextMenu {
    h.app
        .router
        .editor()
        .unwrap()
        .cfg
        .menu
        .menu()
        .expect("the mounts menu is open")
}

fn settings_menu(h: &H) -> Option<&ContextMenu> {
    h.app.router.settings().unwrap().cfg.menu.menu()
}

#[test]
fn editor_mount_menu_opens_at_the_pointer_and_toggles_read_only() {
    let mut h = editor_mounts();
    let (x, y) = h.find("/workspace/libs").unwrap();
    h.mouse(MouseKind::Secondary, x + 4, y);
    let m = editor_menu(&h);
    assert_eq!((m.area.x, m.area.y), (x + 4, y + 1), "at the pointer");
    assert!(row(&h, m.area.y + 1).contains("/workspace/libs"), "titled");
    let labels: Vec<&str> = m.items.iter().map(|i| i.item.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Edit…",
            "Make read-write",
            "Next isolation",
            "Open source",
            "Scope…",
            "Revert changes",
            "Add mount…",
            "Remove"
        ]
    );
    assert!(m.items[4].item.disabled, "a workspace mount has no scope");
    assert!(m.items[5].item.disabled, "nothing to revert yet");
    assert!(m.items[7].danger);
    let id = m.id;

    choose(&mut h, id, 1);
    let libs = row(&h, y);
    assert!(libs.contains(" rw "), "{libs}");
    h.mouse(MouseKind::Secondary, x + 4, y);
    let m = editor_menu(&h);
    assert_eq!(m.items[1].item.label, "Make read-only");
    assert!(!m.items[5].item.disabled, "the change can be reverted");
}

#[test]
fn editor_shift_f10_opens_under_the_cursor_row_and_removes() {
    let mut h = editor_mounts();

    h.key(KeyCode::Enter);
    let (_, y) = h.find("/workspace/payments-platform").unwrap();
    shift_f10(&mut h);
    assert_eq!(editor_menu(&h).area.y, y + 1, "under the cursor row");

    h.key(KeyCode::Char(']'));
    assert!(!h.text().contains("Make read-only"), "{}", h.text());
    assert!(
        !h.text().contains("/workspace/payments-platform"),
        "the page took `]`: the next tab\n{}",
        h.text()
    );

    h.key(KeyCode::Char('['));
    assert!(h.text().contains("/workspace/payments-platform"));
    h.key(KeyCode::Enter);
    shift_f10(&mut h);
    assert!(h.text().contains("Make read-only"), "{}", h.text());
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Removed /workspace/payments-platform"),
        "{}",
        h.text()
    );
}

#[test]
fn settings_env_menu_reopens_on_another_row_and_shows_a_value() {
    let mut h = manager();
    h.app.go(Go::Settings);
    h.draw();
    let (x, y) = h.find("Environments").unwrap();
    h.click(x, y);
    let (_, gy) = h.find("GH_TOKEN").unwrap();
    h.mouse(MouseKind::Secondary, 90, gy);
    let m = settings_menu(&h).expect("GH_TOKEN's menu");
    assert!(
        m.items[1].item.disabled,
        "a 1Password reference cannot be shown"
    );

    let (kx, ky) = h.find("KUBECONFIG").unwrap();
    h.mouse(MouseKind::Secondary, kx, ky);
    let m = settings_menu(&h).expect("reopened");
    assert_eq!((m.area.x, m.area.y), (kx, ky + 1), "at the new spot");
    assert!(row(&h, ky + 2).contains("KUBECONFIG · role sre"), "titled");
    assert_eq!(h.text().matches("1Password reference…").count(), 1);
    let id = m.id;
    choose(&mut h, id, 1);
    assert!(settings_menu(&h).is_none());
    h.mouse(MouseKind::Secondary, kx, ky);
    assert_eq!(settings_menu(&h).unwrap().items[1].item.label, "Mask value");
}

#[test]
fn the_capsule_prefix_waits_until_esc() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app.route(), Route::Capsule);
    let waiting = |h: &H| {
        h.app
            .router
            .capsule()
            .is_some_and(|c| c.prefix_until.is_some())
    };
    h.ctrl('b');
    assert!(waiting(&h), "Ctrl+B starts the prefix");
    h.ticks(200);
    assert!(waiting(&h), "the prefix still waits after 200 ticks");
    h.key(KeyCode::Esc);
    assert!(!waiting(&h), "Esc cancels the prefix");

    if let Some(c) = h.app.router.capsule_mut() {
        c.prefix_timeout_ms = Some(2_000);
    }
    h.ctrl('b');
    assert!(waiting(&h));
    h.ticks(200);
    assert!(!waiting(&h), "a configured timeout still expires");
}

#[test]
fn capsule_pane_menu_routes_the_pointer_by_part() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    shift_f10(&mut h);
    let (cx, cy) = h.find("Copy selection").expect("the pane menu");
    h.click(cx + 1, cy);
    assert!(
        h.text().contains("Copy selection"),
        "a disabled row keeps the menu open\n{}",
        h.text()
    );

    h.click(2, 30);
    assert!(!h.text().contains("Copy selection"), "{}", h.text());
}

#[test]
fn capsule_pane_menu_passes_an_unused_key_to_the_page() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    shift_f10(&mut h);
    assert!(h.text().contains("Copy selection"));
    h.key(KeyCode::F(10));
    let t = h.text();
    assert!(!t.contains("Copy selection"), "the menu closed\n{t}");
    assert!(t.contains("Split below"), "the page took F10\n{t}");
}
