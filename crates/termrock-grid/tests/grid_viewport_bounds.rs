use ratatui_core::layout::Rect;
use termrock_grid::{GridViewportPolicy, solve_grid_viewport};

#[test]
fn body_requires_space_after_reserved_rows_and_columns() {
    let policy = GridViewportPolicy {
        header_rows: 3,
        filter_rows: 2,
        pending_rows: 2,
        status_rows: 1,
        status_inset: 3,
        right_reserve: 2,
    };
    for (width, height, expected) in [(2, 10, false), (3, 9, false), (3, 10, true)] {
        let view = solve_grid_viewport(Rect::new(4, 5, width, height), &policy);
        assert_eq!(view.has_grid_body(), expected, "{width}x{height}");
        if expected {
            assert_eq!(view.grid, Rect::new(4, 10, 1, 1));
        }
    }
}

#[test]
fn status_fit_uses_the_configured_marker_inset() {
    for inset in [0, 1, 3, 40] {
        let view = solve_grid_viewport(
            Rect::new(4, 5, 20, 8),
            &GridViewportPolicy {
                status_rows: 1,
                status_inset: inset,
                ..GridViewportPolicy::default()
            },
        );
        let available = 20_u16.saturating_sub(inset);
        assert!(view.status_text_fits(available));
        assert!(!view.status_text_fits(available.saturating_add(1)));
        assert!(view.status_text_fits(0));
    }
}
