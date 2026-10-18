//! Actual canonical role picker, form, preview and captured save journey.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    reason = "Public journey assertions"
)]
use jackin_app::Scenario;
#[test]
fn canonical_role_key_reaches_durable_workspace_after_captured_save() {
    use jackin_app::{App, Motion, Route};
    use junie_tui::{Id, KeyCode, Theme};
    use junie_tui_testing::Harness;
    let mut h = Harness::new(
        App::for_scenario(Scenario::Returning, Motion::Reduced),
        Theme::junie(),
        120,
        40,
    );
    for _ in 0..8 {
        for _ in 0..3 {
            let _ = h.advance(std::time::Duration::from_millis(200));
        }
        if h.app().route() == Route::Manager {
            break;
        }
        let _ = h.key(KeyCode::Enter);
    }
    for key in [
        KeyCode::Down,
        KeyCode::Char('e'),
        KeyCode::Char('4'),
        KeyCode::Enter,
        KeyCode::End,
        KeyCode::Enter,
    ] {
        let _ = h.key(key);
    }
    assert!(h.text().contains("Add role override"));
    let _ = h.type_str("svc-01");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("New svc-010 environment key"));
    let _ = h.key(KeyCode::Enter);
    let _ = h.type_str("SVC_FLAG");
    for key in [KeyCode::Tab, KeyCode::Tab, KeyCode::Enter] {
        let _ = h.key(key);
    }
    let _ = h.type_str("on");
    let _ = h.key(KeyCode::Tab);
    assert!(h.tab_to(Id::root("editor.cfg").sub("form").sub("save")));
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app()
            .editor
            .pending
            .role_env
            .contains_key("chainargos/svc-010")
    );
    assert!(!h.app().editor.pending.role_env.contains_key("svc-010"));
    assert!(h.app().editor.pending.role_env.keys().all(|key| {
        h.app()
            .world
            .roles
            .iter()
            .any(|role| role.full_name() == *key)
    }));
    assert!(h.tab_to(jackin_app::screens::editor::SAVE));
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Enter);
    for _ in 0..12 {
        let _ = h.advance(std::time::Duration::from_millis(200));
    }
    assert_eq!(h.app().route(), Route::Manager);
    assert!(
        h.app()
            .world
            .workspace(1)
            .unwrap()
            .role_env
            .contains_key("chainargos/svc-010")
    );
    assert!(
        !h.app()
            .world
            .workspace(1)
            .unwrap()
            .role_env
            .contains_key("svc-010")
    );
}
