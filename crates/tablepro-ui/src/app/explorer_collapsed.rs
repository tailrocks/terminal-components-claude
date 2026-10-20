#[cfg(test)]
mod tests {
    use termrock::{KeyCode, Maximized, Theme};
    use termrock_test_support::Harness;

    use super::super::{Screen, TableProApp};

    fn production() -> TableProApp {
        let mut app = TableProApp::default();
        let index = app
            .connections
            .iter()
            .position(|connection| connection.name == "Production")
            .unwrap_or(usize::MAX);
        assert!(app.connect(index), "production connection");
        app
    }

    #[test]
    fn ctrl_b_shows_the_approved_hidden_explorer_cell() {
        let mut visible = Harness::new(production(), Theme::junie(), 120, 40);
        assert_ne!(
            visible.cell(1, 6).symbol(),
            "E",
            "visible explorer already took the hidden rail"
        );
        assert_eq!(visible.app().split_state.maximized(), Maximized::None);
        assert!(!visible.app().workbench.maximized);
        let _ = visible.ctrl('b');
        assert_eq!(visible.app().split_state.maximized(), Maximized::Second);
        assert!(!visible.app().workbench.maximized);
        assert_eq!(visible.app().screen(), Screen::Workbench);
        assert!(visible.app().destructive_intent.is_none());
        assert_eq!(
            visible.cell(1, 6).symbol(),
            "E",
            "status={} text={}",
            visible.app().status(),
            visible.text()
        );
        assert_eq!(visible.cursor(), Some(termrock::Position::new(12, 6)));
        assert!(visible.runtime().cursor_bar());

        let mut zoomed = Harness::new(production(), Theme::junie(), 120, 40);
        let _ = zoomed.key(KeyCode::Char('z'));
        assert!(zoomed.app().workbench.maximized);
        assert_eq!(zoomed.app().split_state.maximized(), Maximized::None);
        assert_ne!(
            zoomed.cell(1, 6).symbol(),
            "E",
            "maximized frame took the hidden rail: {}",
            zoomed.text()
        );
    }
}
