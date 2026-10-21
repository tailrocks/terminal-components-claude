#![allow(
    dead_code,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    clippy::arithmetic_side_effects,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::indexing_slicing,
    clippy::many_single_char_names,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::too_many_lines,
    clippy::unwrap_used,
    clippy::expect_used
)]

use termrock::{Axis, KeyCode, MouseKind};

use jackin_preview_app::Route;
mod support;
use jackin_preview_app::{Motion, Scenario};
use support::H;

fn row(h: &H, y: u16) -> String {
    h.text().lines().nth(y as usize).unwrap_or("").to_owned()
}

fn last_row(h: &H) -> String {
    let t = h.text();
    t.lines().next_back().unwrap_or("").to_owned()
}

#[test]
fn capsule_has_a_menu_bar_and_a_status_bar_instead_of_the_identity_line() {
    let h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    assert_eq!(h.app().route(), Route::Capsule);
    let top = row(&h, 0);
    assert!(!top.contains("inside the Construct"), "{top}");
    assert!(
        top.contains("jackin❯") && top.contains("File") && top.contains("Help"),
        "{top}"
    );
    assert!(row(&h, 2).contains("Shell"), "{}", row(&h, 2));
    assert!(top.contains("payments-platform ›"), "{top}");
    let status = row(&h, 38);
    assert!(status.contains("PR #482"), "{status}");
    assert!(
        status.contains('%') && (status.contains("━") || status.contains("╍")),
        "usage meter missing: {status}"
    );
    assert!(last_row(&h).contains("Ctrl+B"), "{}", last_row(&h));
}

#[test]
fn capsule_export_closes_the_file_menu_and_writes_nothing() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let clipboard = h.app().world.clipboard.clone();
    h.key(KeyCode::F(10));
    for _ in 0..3 {
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("Export selected file…"), "{}", h.text());
    h.key(KeyCode::Enter);
    let closed = h.text();
    assert!(
        !closed.contains("Export selected file"),
        "export row stayed open: {closed}"
    );
    assert!(
        !closed.contains("Account for Claude Code"),
        "Down did not leave the New tab row: {closed}"
    );
    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(h.app().world.clipboard, clipboard);
    h.key(KeyCode::F(10));
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    assert!(
        h.text().contains("Export selected file and reveal…"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Enter);
    let revealed = h.text();
    assert!(!revealed.contains("Export selected file"), "{revealed}");
    assert!(
        !revealed.contains("Account for Claude Code"),
        "reveal row was not chosen: {revealed}"
    );
    assert_eq!(h.app().route(), Route::Capsule);
    assert_eq!(h.app().world.clipboard, clipboard);
}

#[test]
fn menu_bar_opens_switches_and_runs_an_action() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::F(10));
    assert!(
        h.text().contains("New tab") && h.text().contains("Split right"),
        "{}",
        h.text()
    );
    assert!(
        last_row(&h).contains("Menu") && last_row(&h).contains("Choose"),
        "{}",
        last_row(&h)
    );
    h.key(KeyCode::Right);
    assert!(h.text().contains("Copy selection"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Copy selection"));
    h.key(KeyCode::F(10));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("New tab"), "spawn picker: {}", h.text());
    h.key(KeyCode::Esc);

    let (x, y) = h.find("View").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Zoom pane"), "{}", h.text());
    let (ux, uy) = h.find("Usage").unwrap();
    h.click(ux, uy);
    assert!(h.text().contains("Overview"), "usage dialog: {}", h.text());
}

#[test]
fn capsule_prefix_space_paints_the_command_palette_cell() {
    let closed = H::new(Scenario::CapsuleMulti, Motion::Paused, 40, 120, 40);
    assert!(
        !closed.text().contains("Command palette"),
        "closed capsule must stay off the palette frame"
    );
    let mut menu = H::new(Scenario::CapsuleMulti, Motion::Paused, 40, 120, 40);
    menu.ctrl('b');
    menu.key(KeyCode::Char('m'));
    assert!(
        !menu.text().contains("Command palette"),
        "tab menu must not paint the command palette"
    );
    let mut open = H::new(Scenario::CapsuleMulti, Motion::Paused, 40, 120, 40);
    open.ctrl('b');
    open.key(KeyCode::Char(' '));
    let (x, y) = open.find("Command palette").expect("Command palette");
    let cell = open.buffer().cell((x, y)).expect("Command palette cell");
    assert_eq!(cell.symbol(), "C");
    assert!(open.text().contains("Command palette"), "{}", open.text());
}

#[test]
fn capsule_prefix_m_paints_the_change_title_cell() {
    let closed = H::new(Scenario::CapsuleMulti, Motion::Paused, 40, 120, 40);
    let before = closed.buffer().cell((4, 5)).expect("closed capsule cell");
    assert_ne!(
        (
            before.symbol(),
            before.fg,
            before.bg,
            before.modifier.contains(termrock::Modifier::BOLD),
        ),
        (
            "C",
            termrock::Color::from_u32(0x00_ff_ff_ff),
            termrock::Color::from_u32(0x00_2f_5a_a8),
            true,
        ),
        "closed capsule has no Change title",
    );
    assert!(
        !closed.text().contains("Change title…"),
        "closed capsule must stay off the menu frame: {}",
        closed.text()
    );
    let mut open = H::new(Scenario::CapsuleMulti, Motion::Paused, 40, 120, 40);
    open.ctrl('b');
    open.key(KeyCode::Char('m'));
    let cell = open.buffer().cell((4, 5)).expect("Change title");
    assert_eq!(cell.symbol(), "C");
    assert_eq!(cell.fg, termrock::Color::from_u32(0x00_ff_ff_ff));
    assert_eq!(cell.bg, termrock::Color::from_u32(0x00_2f_5a_a8));
    assert!(cell.modifier.contains(termrock::Modifier::BOLD));
    assert!(open.text().contains("Change title…"), "{}", open.text());
}

#[test]
fn tab_context_menu_renames_and_closes_by_mouse_and_keyboard() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let (x, y) = h.find("Shell").unwrap();
    h.mouse(MouseKind::Secondary, x, y);
    assert!(
        h.text().contains("Change title…") && h.text().contains("Close tab"),
        "{}",
        h.text()
    );
    assert!(last_row(&h).contains("Choose"), "{}", last_row(&h));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Change tab title"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.type_str("ops");
    assert!(h.text().contains("ops"), "{}", h.text());
    h.key(KeyCode::Enter);
    if h.text().contains("Change tab title") {
        h.key(KeyCode::Enter);
    }
    assert!(row(&h, 2).contains("ops"), "{}", row(&h, 2));

    h.ctrl('b');
    h.key(KeyCode::Char('m'));
    assert!(h.text().contains("Change title…"), "{}", h.text());
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Close tab?"), "{}", h.text());
    h.key(KeyCode::Esc);

    h.ctrl('b');
    h.key(KeyCode::Char('m'));
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Change title…"));
}

#[test]
fn hint_bar_stays_on_the_last_row_across_layers() {
    let mut h = H::new(Scenario::Returning, Motion::Reduced, 0, 120, 40);
    let base = last_row(&h);
    assert!(base.contains("Enter"), "{base}");
    h.key(KeyCode::Char('?'));
    let help = last_row(&h);
    assert!(help.contains("Esc") && help.contains("Close"), "{help}");
    h.key(KeyCode::Esc);

    h.key(KeyCode::Home);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    let picker = last_row(&h);
    assert!(picker.contains("Choose"), "{picker}");

    let t = h.text();
    assert_eq!(t.matches("Enter Choose").count(), 1, "{t}");
    h.key(KeyCode::Esc);
    let mut c = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    c.ctrl('b');
    assert!(last_row(&c).contains("New tab"), "{}", last_row(&c));
    assert!(row(&c, 0).contains("prefix…"), "{}", row(&c, 0));
}

#[test]
fn inspect_changes_opens_from_the_view_menu_in_both_modes() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::F(10));
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Inspect changes ·"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("@@") || h.text().contains("│"),
        "diff shown: {}",
        h.text()
    );
    h.key(KeyCode::Char('m'));
    assert!(
        h.text().contains("src") && last_row(&h).contains("Tab"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Char('d'));
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Inspect changes ·"), "{}", h.text());
    assert_eq!(h.app().route(), Route::Capsule);
}

#[test]
fn command_palette_scrolls_with_the_wheel_and_keeps_the_selection() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 24);
    h.ctrl('\\');
    assert!(h.text().contains("Command palette"));
    let (x, y) = h.find("New tab").unwrap();
    let before = h.text();
    h.mouse(MouseKind::Wheel(Axis::V, 1), x, y + 2);
    let after = h.text();
    assert_ne!(before, after, "wheel did not move the rows");
    assert!(
        !after.contains("New tab")
            || after
                .lines()
                .nth(y as usize)
                .is_some_and(|l| !l.contains("New tab")),
        "{after}"
    );
    h.mouse(MouseKind::Wheel(Axis::V, -1), x, y + 2);
    assert_eq!(h.text(), before, "wheel up did not restore the rows");
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("New tab"),
        "the selection stayed on the first item: {}",
        h.text()
    );
}

#[test]
fn about_dialog_opens_from_help_and_paints_nothing_when_closed() {
    let mut h = H::new(Scenario::CapsuleMulti, Motion::Reduced, 0, 120, 40);
    let closed = h.text();
    assert!(
        !closed.contains("Design system"),
        "about body painted while closed: {closed}"
    );
    assert!(h.app().world.clipboard.is_none());
    h.key(KeyCode::F(10));
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    let open = h.text();
    assert!(open.contains("jackin-preview · deterministic"), "{open}");
    assert!(open.contains("Design system"), "{open}");
    assert!(open.contains("nothing here touches"), "{open}");
    assert!(open.contains("capsule-multi"), "{open}");
    assert_eq!(h.app().route(), Route::Capsule);
    h.key(KeyCode::Enter);
    let after = h.text();
    assert!(
        !after.contains("Design system"),
        "Close left the about body up: {after}"
    );
    assert_eq!(h.app().route(), Route::Capsule);
    h.key(KeyCode::F(10));
    assert!(
        h.text().contains("New tab"),
        "a closed about layer still ate the file menu: {}",
        h.text()
    );
    assert!(h.app().world.clipboard.is_none());
}
