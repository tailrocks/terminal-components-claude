use tablepro_ui::TableProApp;
use termrock::{KeyCode, Theme};
use termrock_test_support::Harness;

/// The reference filter editor opens on its value field at every size. The
/// app used to force that focus only at 120×40, and it stole the focus back
/// on every frame, so Tab could not leave the field.
#[test]
fn filter_card_focuses_the_value_field_at_every_size() {
    for (width, height) in [(120u16, 40u16), (100, 30), (80, 24)] {
        let mut app = TableProApp::default();
        assert!(app.connect(4));
        let mut harness = Harness::new(app, Theme::junie(), width, height);
        for _ in 0..5 {
            let _ = harness.key(KeyCode::Down);
        }
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.key(KeyCode::Home);
        let _ = harness.ctrl('f');
        let text = harness.text();
        assert!(text.contains("Add filter"), "{width}x{height}: {text}");
        assert!(
            harness.app().filter_value_focused(),
            "{width}x{height}: the filter card does not focus its value field"
        );
        let _ = harness.key(KeyCode::Tab);
        harness.tick();
        assert!(
            !harness.app().filter_value_focused(),
            "{width}x{height}: the filter card steals the focus back"
        );
    }
}
