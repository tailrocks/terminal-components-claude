use tablepro_ui::TableProApp;
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

#[test]
fn keyboard_sort_scrolls_to_created_at_ascending() {
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
    for _ in 0..12 {
        let _ = harness.key(KeyCode::Right);
    }
    let _ = harness.key(KeyCode::Char('s'));
    let text = harness.text();
    assert!(
        text.contains("sort created_at ▴"),
        "missing sort status: {text}"
    );
    assert!(
        text.contains("Sorted by created_at ascending"),
        "missing footer: {text}"
    );
    assert!(text.contains("‹10"), "missing left overflow: {text}");
    assert!(
        text.contains("cols 11–13 of 14"),
        "missing column window: {text}"
    );
    assert!(
        text.contains("2024-01-01 02:13:21+00"),
        "rows are not ascending: {text}"
    );
    assert!(
        text.contains("2025-06-25 12:30:14"),
        "first updated_at drifted: {text}"
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
