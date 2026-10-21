use tablepro_ui::TableProApp;
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const SAFE_MODE: Id = Id::root("tablepro.safe-mode");

fn connected_app() -> TableProApp {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    app
}

#[test]
fn replaced_workbench_rejects_safe_mode_choice() {
    let mut h = Harness::new(connected_app(), Theme::junie(), 120, 40);
    h.draw();
    let _ = h.ctrl('l');
    assert!(h.layer_area(SAFE_MODE).is_some());
    // Local connections start Silent; step to Alert so Enter would change the
    // level on a live picker.
    let _ = h.key(KeyCode::Down);
    let before = h.app().safe_mode();

    let replacement = connected_app();
    h.app_mut().workbench = replacement.workbench;
    let _ = h.key(KeyCode::Enter);

    assert_eq!(h.app().safe_mode(), before);
    assert!(h.app().status().contains("Workbench changed"));
    assert!(h.layer_area(SAFE_MODE).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
