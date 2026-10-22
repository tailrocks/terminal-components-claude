#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::TableProApp;
use termrock::{Id, KeyCode, MouseKind, Theme};
use termrock_test_support::Harness;

const SAFE_MODE: Id = Id::root("tablepro.safe-mode");

fn connected_app() -> TableProApp {
    let mut app = TableProApp::new();
    assert!(app.connect(0));
    app
}

fn connected_with_two_tabs() -> Result<(Harness<TableProApp>, termrock::Position, Id), &'static str>
{
    let mut app = connected_app();
    let Some(first) = app.workbench.active_key() else {
        return Err("initial query tab");
    };
    let Some(active) = app.workbench.new_query("SELECT 'active safe-mode draft'") else {
        return Err("second query tab");
    };
    assert_ne!(first, active);
    assert!(app.workbench.activate(active));

    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    let active_editor = active.control("query");
    assert!(tab_navigation::tab_to(&mut harness, active_editor));
    let Some(first_tab) = harness.find("Query 1") else {
        return Err("first tab in strip");
    };
    Ok((
        harness,
        termrock::Position::new(first_tab.0, first_tab.1),
        active_editor,
    ))
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

#[test]
fn safe_mode_escape_and_outside_click_keep_the_live_connection_unchanged()
-> Result<(), &'static str> {
    for outside_dismissal in [false, true] {
        let (mut h, underlay_tab, active_editor) = connected_with_two_tabs()?;
        let original_mode = h.app().safe_mode();
        let original_connection = h.app().workbench.connection.clone();
        let original_saved_connections = h.app().connections_screen.connections.clone();
        let original_selection = h.app().connections_screen.selected;
        let original_key = h.app().workbench.active_key();

        let _ = h.ctrl('l');
        assert!(h.layer_area(SAFE_MODE).is_some());
        assert!(h.text().contains("Safe Mode · this connection"));
        let _ = h.key(KeyCode::Down);
        assert!(h.text().contains("Alert"));

        if outside_dismissal {
            let Some(layer) = h.layer_area(SAFE_MODE) else {
                return Err("safe-mode layer");
            };
            assert!(
                underlay_tab.x < layer.x
                    || underlay_tab.x >= layer.right()
                    || underlay_tab.y < layer.y
                    || underlay_tab.y >= layer.bottom(),
                "first tab is outside the safe-mode layer: {layer:?}"
            );
            let _ = h.mouse(MouseKind::Down, underlay_tab.x, underlay_tab.y);
            let _ = h.mouse(MouseKind::Up, underlay_tab.x, underlay_tab.y);
        } else {
            let _ = h.key(KeyCode::Esc);
        }

        assert!(h.layer_area(SAFE_MODE).is_none());
        assert_eq!(h.app().safe_mode(), original_mode);
        assert_eq!(h.app().workbench.connection, original_connection);
        assert_eq!(
            h.app().connections_screen.connections,
            original_saved_connections
        );
        assert_eq!(h.app().connections_screen.selected, original_selection);
        assert_eq!(h.app().workbench.active_key(), original_key);
        assert_eq!(h.focus(), Some(active_editor));
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
    Ok(())
}

#[test]
fn safe_mode_choice_changes_only_the_active_connection_and_header_strip() -> Result<(), &'static str>
{
    let (mut h, _, active_editor) = connected_with_two_tabs()?;
    let original_connection = h.app().workbench.connection.clone();
    let original_saved_connections = h.app().connections_screen.connections.clone();
    let original_key = h.app().workbench.active_key();
    let mut expected_connection = original_connection;
    expected_connection.safe_mode = tablepro_ui::SafeMode::Alert;

    let _ = h.ctrl('l');
    assert!(h.layer_area(SAFE_MODE).is_some());
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);

    assert!(h.layer_area(SAFE_MODE).is_none());
    assert_eq!(h.app().safe_mode(), tablepro_ui::SafeMode::Alert);
    assert_eq!(h.app().workbench.connection, expected_connection);
    assert_eq!(h.app().workbench.active_key(), original_key);
    assert_eq!(
        h.app().connections_screen.connections,
        original_saved_connections,
        "changing the live mode leaves other saved connection choices intact"
    );
    assert!(h.text().contains("alert"), "header strip: {}", h.text());
    assert_eq!(h.focus(), Some(active_editor));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    Ok(())
}
