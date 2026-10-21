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
    fn narrow_query_typing_keeps_the_prefix() {
        let mut app = Harness::new(production(), Theme::junie(), 72, 20);
        let _ = app.key(KeyCode::Tab);
        let _ = app.type_str("SELECT * FROM ord");
        let _ = app.key(KeyCode::Enter);
        let _ = app.type_str(" WHERE st");
        let _ = app.key(KeyCode::Tab);
        let mid = app.text();
        assert!(
            mid.contains("SELECT") && mid.contains("status"),
            "tab from the narrow explorer must type into the query\n{mid}"
        );
    }

    #[test]
    fn a_narrow_workbench_collapses_the_explorer_without_ctrl_b() {
        let mut narrow = Harness::new(production(), Theme::junie(), 72, 20);
        let _ = narrow.key(KeyCode::Tab);
        assert_eq!(
            narrow.cell(1, 6).symbol(),
            "E",
            "page width under 100 leaves the explorer rail\n{}",
            narrow.text()
        );
        let wide = Harness::new(production(), Theme::junie(), 120, 40);
        assert_ne!(
            wide.cell(1, 6).symbol(),
            "E",
            "120 keeps the explorer panel"
        );
    }

    #[test]
    fn header_drops_the_size_badge_before_help() {
        let left = [1u16, 8, 10, 12, 18, 6];
        let left_pri = [9u8, 9, 9, 8, 7, 8];
        let help = 6u16;
        let dropped = |width, badge| {
            super::super::header_overflow(width, &left, &left_pri, &[badge, help], &[1, 4])
        };
        assert_eq!(
            dropped(72, 17),
            Some(([false, false], 69)),
            "72 drops the right cluster"
        );
        assert_eq!(
            dropped(80, 17),
            Some(([false, true], 69)),
            "80 drops the badge"
        );
        assert!(dropped(100, 18).is_none(), "100 keeps badge and help");
        assert!(dropped(120, 18).is_none(), "120 keeps badge and help");
    }

    #[test]
    fn footer_keeps_explain_when_the_status_does_not_fit() {
        let costs = [12u16, 12, 15, 16, 8, 10];
        let status = 25u16;
        assert_eq!(
            super::super::fit_workbench_hints(&costs, 72, status),
            (5, false, true)
        );
        assert_eq!(
            super::super::fit_workbench_hints(&costs, 80, status),
            (6, false, false)
        );
        assert_eq!(
            super::super::fit_workbench_hints(&costs, 100, status),
            (5, true, true)
        );
        assert_eq!(
            super::super::fit_workbench_hints(&costs, 120, status),
            (6, true, false)
        );
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
