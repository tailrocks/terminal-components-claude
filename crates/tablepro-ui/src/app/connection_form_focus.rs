#[cfg(test)]
mod tests {
    use termrock::{KeyCode, Part, StateFlags, Theme};
    use termrock_test_support::Harness;

    use super::super::TableProApp;

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
        assert!(opened.app().empty_new_connection_draft());
        let _ = opened.key(KeyCode::BackTab);
        let _ = opened.key(KeyCode::Right);
        assert_eq!(opened.focus(), Some(test_connection));
        assert!(opened.app().empty_new_connection_draft());
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
        assert!(!filled.app().empty_new_connection_draft());
    }
}
