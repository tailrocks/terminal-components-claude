#[cfg(test)]
mod tests {
    use termrock::{KeyCode, Part, StateFlags, Theme};
    use termrock_test_support::Harness;

    use super::super::TableProApp;

    #[test]
    fn new_connection_starts_with_blank_identity_fields_and_driver_defaults() -> Result<(), String>
    {
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
            .ok_or_else(|| "new connection form should own a draft".to_owned())?;
        assert_eq!(draft.name, "");
        assert_eq!(draft.database, "");
        assert_eq!(draft.user, "");
        assert_eq!(draft.engine, 0);
        assert_eq!(draft.host, "localhost");
        assert_eq!(draft.port, "5432");
        assert_eq!(harness.app().workbench().connection, active_connection);
        Ok(())
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

    #[test]
    fn typing_active_name_as_prefix_does_not_replace_it_on_the_next_character() {
        let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let active_name = harness.app().workbench().connection.name.clone();
        let expected = format!("{active_name} replica");

        let _ = harness.ctrl('n');
        let _ = harness.type_str(&active_name);
        let _ = harness.type_str(" replica");

        assert_eq!(
            harness
                .app()
                .connection_draft()
                .map(|draft| draft.name.as_str()),
            Some(expected.as_str())
        );
        assert_name_field_is_visible_and_focused(&harness, &expected);
    }

    #[test]
    fn pasting_after_active_name_prefix_keeps_the_prefix() {
        let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let active_name = harness.app().workbench().connection.name.clone();
        let expected = format!("{active_name} replica");

        let _ = harness.ctrl('n');
        let _ = harness.type_str(&active_name);
        let _ = harness.paste(" replica");

        assert_eq!(
            harness
                .app()
                .connection_draft()
                .map(|draft| draft.name.as_str()),
            Some(expected.as_str())
        );
        assert_name_field_is_visible_and_focused(&harness, &expected);
    }

    #[test]
    fn unicode_active_name_prefix_and_reopen_preserve_text_and_focus() {
        let mut app = TableProApp::default();
        app.connection.name = "東京 e\u{301} 🧪".to_owned();
        app.workbench.connection.name = app.connection.name.clone();
        let mut harness = Harness::new(app, Theme::junie(), 120, 40);
        let active_name = harness.app().connection.name.clone();
        let expected = format!("{active_name} レプリカ界");

        let _ = harness.ctrl('n');
        let _ = harness.type_str(&active_name);
        let _ = harness.paste(" レプリカ界");
        assert_eq!(
            harness
                .app()
                .connection_draft()
                .map(|draft| draft.name.as_str()),
            Some(expected.as_str())
        );
        assert_name_field_is_visible_and_focused(&harness, &expected);

        let cancel = tablepro_domain::connections::FORM
            .part(Part::ACTIONS)
            .index(1);
        let _ = harness.click_id(cancel);
        assert!(!harness.app().connection_form_open());
        let _ = harness.ctrl('n');
        assert_eq!(
            harness.focus(),
            Some(tablepro_domain::connections::field::NAME)
        );
        assert_eq!(
            harness
                .app()
                .connection_draft()
                .map(|draft| draft.name.as_str()),
            Some("")
        );
        let _ = harness.type_str(&expected);
        assert_eq!(
            harness
                .app()
                .connection_draft()
                .map(|draft| draft.name.as_str()),
            Some(expected.as_str())
        );
        assert_name_field_is_visible_and_focused(&harness, &expected);
    }

    #[test]
    fn explicit_select_all_and_save_keep_typed_name_as_domain_data() {
        let mut harness = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        let active_name = harness.app().connection.name.clone();
        let expected = format!("{active_name} replica");

        let _ = harness.ctrl('n');
        let _ = harness.type_str("temporary");
        let _ = harness.ctrl('l');
        let _ = harness.type_str(&expected);
        if let Some(draft) = harness.app_mut().draft.as_mut() {
            draft.database = "example_db".to_owned();
        }
        let save = tablepro_domain::connections::FORM
            .part(Part::ACTIONS)
            .index(2);
        let _ = harness.click_id(save);

        assert!(
            harness
                .app()
                .connections
                .iter()
                .any(|connection| connection.name == expected),
            "saved domain connection should keep the typed field value"
        );
    }

    fn assert_name_field_is_visible_and_focused(harness: &Harness<TableProApp>, expected: &str) {
        let name = tablepro_domain::connections::field::NAME;
        assert_eq!(harness.focus(), Some(name));
        let area = harness.area_of(name).unwrap_or_default();
        let cursor = harness.cursor().unwrap_or_default();
        assert!(area.contains(cursor), "cursor {cursor:?} outside {area:?}");
        assert!(
            harness.snapshot().text().contains(expected),
            "rendered name field does not contain {expected:?}"
        );
    }
}
