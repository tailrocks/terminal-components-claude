use tablepro_ui::{Screen, Surface, TableProApp};
use termrock::{Id, KeyCode, Theme};
use termrock_test_support::Harness;

const CONNECTIONS: Id = Id::root("tablepro.connections.list");

fn open_delete_dialog(harness: &mut Harness<TableProApp>) {
    assert!(harness.tab_to(CONNECTIONS), "connections list is reachable");
    let _ = harness.key(KeyCode::Char('d'));
    assert!(
        harness.find("Delete connection?").is_some(),
        "the Delete key opens the live connection confirmation"
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
        assert!(harness.tab_to(CONNECTIONS));
        let original_frame = harness.buffer().clone();
        let original_cursor = harness.cursor();

        open_delete_dialog(&mut harness);
        let dialog_focus = harness.focus();
        if outside_click {
            let _ = harness.click(0, 0);
        } else {
            let _ = harness.key(KeyCode::Esc);
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
