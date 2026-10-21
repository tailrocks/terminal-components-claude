#[cfg(test)]
mod tests {
    use termrock::{KeyCode, Part, StateFlags, Theme};
    use termrock_test_support::Harness;

    use super::super::TableProApp;

    #[test]
    fn new_connection_starts_with_blank_identity_fields_and_driver_defaults() {
        let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let active_connection = harness.app().workbench().connection.clone();
        let edit_draft = super::super::ConnectionDraft::from_connection(&active_connection);
        assert_eq!(edit_draft.name, active_connection.name);
        assert_eq!(edit_draft.database, active_connection.database);
        assert_eq!(edit_draft.user, active_connection.user);
        let _ = harness.ctrl('n');

        let draft = harness
            .app()
            .connection_draft()
            .expect("new connection form should own a draft");
        assert_eq!(draft.name, "");
        assert_eq!(draft.database, "");
        assert_eq!(draft.user, "");
        assert_eq!(draft.engine, 0);
        assert_eq!(draft.host, "localhost");
        assert_eq!(draft.port, "5432");
        assert_eq!(harness.app().workbench().connection, active_connection);
    }

    #[test]
    fn backtab_right_focuses_test_connection_on_the_empty_card() {
        let test_connection = tablepro_domain::connections::FORM
            .part(Part::ACTIONS)
            .index(0);
        let mut opened = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let _ = opened.ctrl('n');
        assert_eq!(
            opened.focus(),
            Some(tablepro_domain::connections::field::NAME)
        );
        assert!(opened.app().blank_new_connection_draft());
        let _ = opened.key(KeyCode::BackTab);
        let _ = opened.key(KeyCode::Right);
        assert_eq!(opened.focus(), Some(test_connection));
        assert!(opened.app().blank_new_connection_draft());
        assert!(
            !opened
                .runtime()
                .state_of(tablepro_domain::connections::field::NAME)
                .contains(StateFlags::FOCUSED)
        );
        assert!(
            opened
                .runtime()
                .state_of(test_connection)
                .contains(StateFlags::FOCUSED)
        );
        assert_eq!(opened.cursor(), None);
        assert!(!opened.runtime().cursor_bar());

        let mut filled = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let _ = filled.ctrl('n');
        let _ = filled.type_str("Staging replica");
        assert_eq!(
            filled.focus(),
            Some(tablepro_domain::connections::field::NAME)
        );
        assert!(!filled.app().blank_new_connection_draft());
    }
}
