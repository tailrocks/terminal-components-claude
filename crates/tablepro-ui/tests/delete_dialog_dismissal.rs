#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{Screen, Surface, TableProApp};
use termrock::{Diagnostic, Id, KeyCode, MouseKind, Theme};
use termrock_test_support::Harness;

const CONNECTIONS: Id = Id::root("tablepro.connections.list");
const DELETE_DIALOG: Id = Id::root("tablepro.delete-connection");

fn open_delete_dialog(harness: &mut Harness<TableProApp>) {
    assert!(
        tab_navigation::tab_to(harness, CONNECTIONS),
        "connections list is reachable"
    );
    let _ = harness.key(KeyCode::Char('d'));
    assert!(
        harness.find("Delete connection?").is_some(),
        "the Delete key opens the live connection confirmation"
    );
}

fn assert_no_undelivered_intents(harness: &Harness<TableProApp>) {
    assert!(
        !harness
            .diagnostics()
            .iter()
            .any(|diagnostic| matches!(diagnostic, Diagnostic::UndeliveredIntent { .. })),
        "dialog dismissal left an undelivered intent: {:?}",
        harness.diagnostics()
    );
}

#[test]
fn delete_connection_dialog_dismissal_cancels_and_can_be_reopened() {
    for outside_click in [false, true] {
        let mut app = TableProApp::default();
        app.screen = Screen::Connections;
        app.set_surface(Surface::Connections);
        let original_connections = app.connections_screen.connections.clone();
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        assert!(tab_navigation::tab_to(&mut harness, CONNECTIONS));
        let original_frame = harness.buffer().clone();
        let original_cursor = harness.cursor();

        open_delete_dialog(&mut harness);
        let dialog_focus = harness.focus();
        if outside_click {
            let _ = harness.mouse(MouseKind::Down, 0, 0);
            assert_no_undelivered_intents(&harness);
            let _ = harness.mouse(MouseKind::Up, 0, 0);
        } else {
            let _ = harness.key(KeyCode::Esc);
            assert_no_undelivered_intents(&harness);
        }
        assert!(harness.find("Delete connection?").is_none());
        assert_eq!(
            harness.app().connections_screen.connections,
            original_connections
        );
        assert_eq!(harness.focus(), Some(CONNECTIONS));
        assert_ne!(dialog_focus, Some(CONNECTIONS));
        harness.draw();
        assert_eq!(harness.buffer(), &original_frame);
        assert_eq!(harness.cursor(), original_cursor);

        open_delete_dialog(&mut harness);
        let _ = harness.key(KeyCode::Esc);
        assert_eq!(
            harness.app().connections_screen.connections,
            original_connections
        );
        assert_eq!(harness.focus(), Some(CONNECTIONS));
    }
}

#[test]
fn confirming_delete_removes_only_the_nonfirst_selected_connection() -> Result<(), &'static str> {
    let mut app = TableProApp::default();
    app.screen = Screen::Connections;
    app.set_surface(Surface::Connections);
    let original_connections = app.connections_screen.connections.clone();
    assert!(original_connections.len() >= 2);
    let Some(target_index) = original_connections
        .iter()
        .position(|connection| connection.name == "Production")
    else {
        return Err("Production fixture connection");
    };
    assert!(
        target_index > 0,
        "the confirmation targets a nonfirst record"
    );
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);

    assert!(tab_navigation::tab_to(&mut harness, CONNECTIONS));
    for _ in 0..(original_connections.len() * 4) {
        if harness.app().connections_screen.selected == target_index {
            break;
        }
        let _ = harness.key(KeyCode::Down);
    }
    assert_eq!(harness.app().connections_screen.selected, target_index);
    assert_eq!(
        harness
            .app()
            .connections_screen
            .connections
            .get(target_index)
            .map(|connection| connection.name.as_str()),
        Some("Production")
    );

    open_delete_dialog(&mut harness);
    let confirmation_focus = harness.focus();
    assert_ne!(confirmation_focus, Some(CONNECTIONS));
    assert!(harness.layer_area(DELETE_DIALOG).is_some());
    assert!(harness.area_of(DELETE_DIALOG).is_some());
    let confirmation_frame = harness.text();
    assert!(confirmation_frame.contains("Delete connection?"));
    assert!(confirmation_frame.contains("prod-db-1.acme.io"));
    assert!(harness.text().contains("Production"));
    assert!(harness.text().contains("prod-db-1.acme.io"));
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Enter);

    let mut expected_connections = original_connections;
    let removed = expected_connections.remove(target_index);
    assert_eq!(removed.name, "Production");
    assert!(harness.find("Delete connection?").is_none());
    assert!(harness.layer_area(DELETE_DIALOG).is_none());
    assert!(harness.area_of(DELETE_DIALOG).is_none());
    assert!(!harness.text().contains("Delete connection?"));
    assert!(!harness.text().contains("prod-db-1.acme.io"));
    assert_eq!(
        harness.app().connections_screen.connections,
        expected_connections,
        "only the reviewed record disappears from the screen model"
    );
    assert!(
        !harness
            .app()
            .connections_screen
            .connections
            .iter()
            .any(|connection| connection.name == "Production")
    );
    assert!(harness.app().connections_screen.selected < expected_connections.len());
    assert!(
        matches!(
            harness
                .app()
                .connections_screen
                .connections
                .get(harness.app().connections_screen.selected),
            Some(connection) if connection.name != "Production"
        ),
        "selection remains on a surviving record"
    );
    assert_eq!(harness.focus(), Some(CONNECTIONS));
    let diagnostic_count = harness.diagnostics().len();
    harness.draw();
    assert_eq!(harness.diagnostics().len(), diagnostic_count);
    let diagnostics_after_redraw = harness.diagnostics().to_vec();
    let post_delete_layer = harness.layer_area(DELETE_DIALOG);
    let post_delete_area = harness.area_of(DELETE_DIALOG);
    let post_delete_focus = harness.focus();
    let post_delete_selected = harness.app().connections_screen.selected;
    let post_delete_frame = harness.text();

    assert!(
        diagnostics_after_redraw.is_empty(),
        "after confirm and a normal redraw: layer={post_delete_layer:?}, area={post_delete_area:?}, focus={post_delete_focus:?}, selected={post_delete_selected}, frame={post_delete_frame:?}, diagnostics={diagnostics_after_redraw:?}"
    );

    assert!(harness.app_mut().connect(target_index));
    assert_eq!(
        harness.app().workbench.connection.name,
        "Scratch",
        "the app's connection model has the same surviving order"
    );
    Ok(())
}

#[test]
fn confirmed_close_receipt_is_drained_before_reopening() -> Result<(), &'static str> {
    let mut app = TableProApp::default();
    app.screen = Screen::Connections;
    app.set_surface(Surface::Connections);
    let original_connections = app.connections_screen.connections.clone();
    let Some(target_index) = original_connections
        .iter()
        .position(|connection| connection.name == "Production")
    else {
        return Err("Production fixture connection");
    };
    let mut harness = Harness::new(app, Theme::junie(), 120, 40);
    assert!(tab_navigation::tab_to(&mut harness, CONNECTIONS));
    for _ in 0..(original_connections.len() * 4) {
        if harness.app().connections_screen.selected == target_index {
            break;
        }
        let _ = harness.key(KeyCode::Down);
    }
    open_delete_dialog(&mut harness);
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Enter);

    let mut expected_connections = original_connections;
    assert_eq!(expected_connections.remove(target_index).name, "Production");
    assert_eq!(
        harness.app().connections_screen.connections,
        expected_connections
    );
    let selected_survivor = harness
        .app()
        .connections_screen
        .connections
        .get(harness.app().connections_screen.selected)
        .map(|connection| connection.name.clone())
        .ok_or("selected surviving connection")?;
    open_delete_dialog(&mut harness);
    assert!(harness.layer_area(DELETE_DIALOG).is_some());
    assert!(harness.text().contains(&selected_survivor));
    assert!(!harness.text().contains("Production"));
    assert_no_undelivered_intents(&harness);
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.layer_area(DELETE_DIALOG).is_none());
    assert_eq!(
        harness.app().connections_screen.connections,
        expected_connections,
        "the old confirmation receipt must not delete from the reopened dialog"
    );
    assert_no_undelivered_intents(&harness);
    Ok(())
}
