use tablepro_ui::TableProApp;
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

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
