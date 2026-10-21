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
        let (keep, ellipsis) = dropped(72, 17).expect("72 drops the right cluster");
        assert_eq!(keep, vec![false, false]);
        assert_eq!(ellipsis, 69);
        let (keep, ellipsis) = dropped(80, 17).expect("80 drops the badge");
        assert_eq!(keep, vec![false, true]);
        assert_eq!(ellipsis, 69);
        assert!(dropped(100, 18).is_none(), "100 keeps badge and help");
        assert!(dropped(120, 18).is_none(), "120 keeps badge and help");
    }

    #[test]
    fn footer_drops_the_groups_that_fall_off_the_right_end() {
        use crate::footer::{Hint, plan};
        use termrock::Rect;

        let groups = [
            Hint::new("Ctrl+R", "Run"),
            Hint::new("Alt+R", "Run all"),
            Hint::new("Ctrl+Space", "Complete"),
            Hint::new("Tab", "Indent"),
            Hint::new("F6", "Leave"),
            Hint::new("Esc", "Done"),
        ];
        let area = Rect::new(0, 0, 100, 1);
        let status = 25u16;

        // The status owns the right edge; the groups fill what is left. The
        // pinned `Esc` group survives the cut, and the overflow marker takes
        // the columns the dropped groups left behind.
        let narrow = plan(&groups, area, status, 0);
        assert_eq!(narrow.groups, vec![0, 1, 2, 5]);
        assert_eq!(narrow.overflow_at, Some(59));

        // A wider row keeps every group.
        let wide = Rect::new(0, 0, 120, 1);
        let wide = plan(&groups, wide, status, 0);
        assert_eq!(wide.groups, vec![0, 1, 2, 3, 4, 5]);
        assert_eq!(wide.overflow_at, None);
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
