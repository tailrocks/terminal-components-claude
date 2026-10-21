#[path = "common/tab_navigation.rs"]
mod tab_navigation;

use tablepro_ui::{Screen, Surface, TableProApp};
use termrock::{Diagnostic, Id, KeyCode, MouseKind, Theme};
use termrock_test_support::Harness;

const CONNECTIONS: Id = Id::root("tablepro.connections.list");

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
