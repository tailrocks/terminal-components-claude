#![allow(clippy::expect_used)]

use tablepro_ui::TableProApp;
use termrock::{KeyCode, Modifier, Theme};
use termrock_test_support::Harness;

#[test]
fn enter_on_status_shows_the_inline_editor() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .app()
            .workbench
            .active_grid()
            .is_some_and(|(_, grid)| grid.state.is_editing())
    );
    assert!(harness.row(8).contains("curre…"), "{}", harness.row(8));
    assert!(harness.row(9).contains("livered"), "{}", harness.row(9));
    assert!(harness.row(9).contains("25.90"), "{}", harness.row(9));
    assert!(harness.row(39).contains("EDIT"), "{}", harness.row(39));
    assert!(harness.row(39).contains("Next cell"), "{}", harness.row(39));
    assert!(
        !harness.row(39).contains("Enter Edit"),
        "{}",
        harness.row(39)
    );
    let draft = (0..120)
        .map(|x| &harness.buffer()[(x, 9)])
        .find(|cell| cell.symbol() == "l" && cell.modifier.contains(Modifier::UNDERLINED))
        .expect("the inline draft starts with the first visible scrolled character");
    assert_eq!(draft.symbol(), "l");
    assert_eq!(
        harness.cursor(),
        Some(termrock::Position::new(48, 9)),
        "the inline editor begins at the cell's first text cell"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn committed_status_edits_show_the_pending_bar() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.ctrl('l');
    let _ = harness.type_str("paid");
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.ctrl('l');
    let _ = harness.type_str("shipped");
    let _ = harness.key(KeyCode::Enter);
    let text = harness.text();
    assert!(text.contains("• 2 pending"), "{text}");
    assert!(text.contains("· 2 updates"), "{text}");
    assert!(text.contains("Preview SQL"), "{text}");
    assert!(text.contains("Discard"), "{text}");
    assert!(text.contains("paid"), "{text}");
    assert!(text.contains("shipped"), "{text}");
    assert!(harness.row(0).contains("• 2 pending"), "{}", harness.row(0));
    assert!(harness.row(39).contains("Ctrl+S"), "{}", harness.row(39));
    assert!(
        !harness.row(39).contains("Select row"),
        "{}",
        harness.row(39)
    );
    assert!(harness.row(36).contains("rows 1–24"), "{}", harness.row(36));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn keyboard_filter_keeps_status_pending_window() {
    let mut app = TableProApp::default();
    assert!(app.connect(0));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness.text().contains("public › orders"),
        "{}",
        harness.text()
    );
    let _ = harness.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Char('f'));
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.type_str("pending");
    let _ = harness.key(KeyCode::Enter);
    let text = harness.text();
    assert!(text.contains("filtered (1)"), "{text}");
    assert!(text.contains("status = 'pending'"), "{text}");
    assert!(text.contains("match all ▾"), "{text}");
    assert!(text.contains("+ Add filter"), "{text}");
    assert!(text.contains("status ∇"), "{text}");
    assert!(text.contains("rows 1–24 of 335 loaded"), "{text}");
    assert!(text.contains("~201,559"), "{text}");
    assert!(text.contains("cols 4–6 of 14"), "{text}");
    assert!(text.contains("1 filter applied"), "{text}");
    assert!(text.contains("Shift+F10"), "{text}");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn keyboard_sort_keeps_filtered_pending_window() {
    let mut app = TableProApp::default();
    assert!(app.connect(4));
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Down);
    }
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Home);
    for _ in 0..4 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Char('f'));
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.key(KeyCode::BackTab);
    let _ = harness.type_str("pending");
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Char('s'));
    let text = harness.text();
    assert!(text.contains("status ∇ ▴"), "{text}");
    assert!(
        text.contains("sort status ▴ · filtered (1) · rows 1–24 of 335 loaded · ~201,559 total"),
        "{text}"
    );
    assert!(!text.contains("cols 4"), "{text}");
    assert!(text.contains("a1cc5725-426b-4be8-b1d"), "{text}");
    assert!(text.contains("1617.64"), "{text}");
    assert!(text.contains("Sorted by status ascending"), "{text}");
    assert!(
        !text.contains("Shift+F10"),
        "sort footer must drop the menu hint: {text}"
    );
    assert!(
        !text.contains("USD"),
        "currency column must leave the window: {text}"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
