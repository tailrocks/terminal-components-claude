//! TP-017–026: Workbench explorer, tabs, drawer, maximize, no-tabs.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_Q, PRESET_W, backtab, click_text,
    ctrl, down, end, enter, esc, home, launch_preset_q, launch_preset_w, left, observe, page_down,
    page_up, right, tab, type_text,
};
use tablepro_app::Screen;

fn cap_w(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    scenario: &str,
    checkpoint: &str,
    profile: ColorProfile,
) -> Observation {
    observe(harness, scenario, PRESET_W, checkpoint, profile)
}

fn each_matrix(mut body: impl FnMut(u16, u16, ColorProfile)) {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            body(width, height, profile);
        }
    }
}

#[test]
fn tp017_workbench_focus_cycle() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        let initial = cap_w(&harness, "TP-017", "initial", profile);
        assert_eq!(initial.screen, Screen::Workbench);
        assert_eq!(initial.query_counter, 1);
        let ring_len = harness.ring().entries().len().max(1);
        for i in 0..ring_len {
            tab(&mut harness);
            let _ = cap_w(&harness, "TP-017", "each", profile);
            let _ = i;
        }
        for _ in 0..ring_len {
            backtab(&mut harness);
        }
        let restored = cap_w(&harness, "TP-017", "each", profile);
        assert_eq!(restored.screen, Screen::Workbench);
        assert_eq!(restored.ring.len(), initial.ring.len());
    });
}

#[test]
fn tp018_explorer_lazy_expand() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        right(&mut harness, 1);
        let expanded = cap_w(&harness, "TP-018", "expanded_public_navigation", profile);
        end(&mut harness);
        let lazy = cap_w(&harness, "TP-018", "lazy_audit_selected", profile);
        right(&mut harness, 1);
        let expanding = cap_w(&harness, "TP-018", "expanding", profile);
        oracle_tablepro::ticks(&mut harness, 1);
        let loading = cap_w(&harness, "TP-018", "loading", profile);
        oracle_tablepro::ticks(&mut harness, 2);
        let loaded = cap_w(&harness, "TP-018", "loaded", profile);
        left(&mut harness, 1);
        let collapsed = cap_w(&harness, "TP-018", "collapsed", profile);
        home(&mut harness);
        end(&mut harness);
        page_up(&mut harness, 1);
        page_down(&mut harness, 1);
        let boundary = cap_w(&harness, "TP-018", "boundary", profile);
        for obs in [
            &expanded, &lazy, &expanding, &loading, &loaded, &collapsed, &boundary,
        ] {
            assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
        }
    });
}

#[test]
fn tp019_explorer_filter_refresh_schema() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        type_text(&mut harness, "/");
        type_text(&mut harness, "orders");
        let filtered = cap_w(&harness, "TP-019", "filtered", profile);
        esc(&mut harness);
        let cancelled = cap_w(&harness, "TP-019", "cancelled", profile);
        type_text(&mut harness, "/");
        type_text(&mut harness, "zzzz_no_match");
        let empty = cap_w(&harness, "TP-019", "empty", profile);
        esc(&mut harness);
        type_text(&mut harness, "0");
        type_text(&mut harness, "r");
        let refreshed = cap_w(&harness, "TP-019", "refreshed", profile);
        home(&mut harness);
        down(&mut harness, 8);
        enter(&mut harness);
        let schema_changed = cap_w(&harness, "TP-019", "schema_changed", profile);
        for obs in [&filtered, &cancelled, &empty, &refreshed, &schema_changed] {
            assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
        }
    });
}

#[test]
fn tp020_open_orders_keyboard() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        down(&mut harness, 5);
        enter(&mut harness);
        let orders = cap_w(&harness, "TP-020", "orders", profile);
        down(&mut harness, 1);
        right(&mut harness, 2);
        end(&mut harness);
        let grid_moved = cap_w(&harness, "TP-020", "grid_moved", profile);
        assert_eq!(orders.screen, Screen::Workbench);
        assert_eq!(grid_moved.screen, Screen::Workbench);
    });
}

#[test]
fn tp021_preview_promotion_clicks() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        click_text(&mut harness, "orders");
        let preview = cap_w(&harness, "TP-021", "preview", profile);
        click_text(&mut harness, "customers");
        let replaced = cap_w(&harness, "TP-021", "replaced_preview", profile);
        click_text(&mut harness, "customers");
        let promoted = cap_w(&harness, "TP-021", "promoted", profile);
        click_text(&mut harness, "orders");
        let new_preview = cap_w(&harness, "TP-021", "new_preview", profile);
        for obs in [&preview, &replaced, &promoted, &new_preview] {
            assert_eq!(obs.screen, Screen::Workbench);
        }
    });
}

#[test]
fn tp022_explorer_drawer_thresholds() {
    for (width, height) in [(80, 24), (100, 30), (101, 30), (102, 30)] {
        for profile in COLOR_PROFILES {
            let mut harness = launch_preset_w(width, height, profile);
            let explorer = cap_w(&harness, "TP-022", "explorer", profile);
            ctrl(&mut harness, 't');
            let editor = cap_w(&harness, "TP-022", "editor", profile);
            type_text(&mut harness, "0");
            let reopened = cap_w(&harness, "TP-022", "reopened", profile);
            for _ in 0..16 {
                tab(&mut harness);
            }
            let closed = cap_w(&harness, "TP-022", "closed", profile);
            ctrl(&mut harness, 'b');
            let hidden = cap_w(&harness, "TP-022", "hidden", profile);
            ctrl(&mut harness, 'b');
            let shown = cap_w(&harness, "TP-022", "shown", profile);
            for obs in [&explorer, &editor, &reopened, &closed, &hidden, &shown] {
                assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
            }
        }
    }
}

#[test]
fn tp023_maximize_query_and_explorer() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_q("SELECT * FROM orders LIMIT 5", width, height, profile);
        type_text(&mut harness, "z");
        let editor_max = observe(&harness, "TP-023", PRESET_Q, "editor_max", profile);
        esc(&mut harness);
        ctrl(&mut harness, 'r');
        oracle_tablepro::ticks(&mut harness, 7);
        for _ in 0..12 {
            tab(&mut harness);
        }
        type_text(&mut harness, "z");
        let results_max = observe(&harness, "TP-023", PRESET_Q, "results_max", profile);
        esc(&mut harness);
        let restored = observe(&harness, "TP-023", PRESET_Q, "restored", profile);
        type_text(&mut harness, "0");
        type_text(&mut harness, "z");
        let explorer_max = observe(&harness, "TP-023", PRESET_Q, "explorer_max_case", profile);
        for obs in [&editor_max, &results_max, &restored, &explorer_max] {
            assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
        }
    });
}

#[test]
fn tp024_tab_cycling_overflow() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        for _ in 0..18 {
            ctrl(&mut harness, 't');
        }
        let overflow = cap_w(&harness, "TP-024", "overflow", profile);
        type_text(&mut harness, "[");
        let previous = cap_w(&harness, "TP-024", "previous", profile);
        type_text(&mut harness, "]");
        let next = cap_w(&harness, "TP-024", "next", profile);
        for _ in 0..8 {
            tab(&mut harness);
        }
        home(&mut harness);
        end(&mut harness);
        left(&mut harness, 1);
        right(&mut harness, 1);
        let strip = cap_w(&harness, "TP-024", "strip_navigation", profile);
        ctrl(&mut harness, 'w');
        let closed = cap_w(&harness, "TP-024", "closed", profile);
        for obs in [&overflow, &previous, &next, &strip, &closed] {
            assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
        }
        assert!(overflow.tab_count >= 1);
    });
}

#[test]
fn tp025_tab_list_picker() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        for _ in 0..18 {
            ctrl(&mut harness, 't');
        }
        ctrl(&mut harness, 'g');
        let picker = cap_w(&harness, "TP-025", "picker", profile);
        type_text(&mut harness, "Query 2");
        let filtered = cap_w(&harness, "TP-025", "filtered", profile);
        enter(&mut harness);
        let switched = cap_w(&harness, "TP-025", "switched", profile);
        ctrl(&mut harness, 'g');
        oracle_tablepro::delete(&mut harness);
        let deleted = cap_w(&harness, "TP-025", "deleted", profile);
        esc(&mut harness);
        let restored = cap_w(&harness, "TP-025", "restored", profile);
        for obs in [&picker, &filtered, &switched, &deleted, &restored] {
            assert_eq!(obs.screen, Screen::Workbench, "{}", obs.checkpoint);
        }
    });
}

#[test]
fn tp026_no_tabs_then_new() {
    each_matrix(|width, height, profile| {
        let mut harness = launch_preset_w(width, height, profile);
        for _ in 0..32 {
            if harness.app().workbench.tabs().is_empty() {
                break;
            }
            ctrl(&mut harness, 'w');
            enter(&mut harness);
        }
        let empty = cap_w(&harness, "TP-026", "empty", profile);
        ctrl(&mut harness, 't');
        let new = cap_w(&harness, "TP-026", "new", profile);
        assert_eq!(empty.screen, Screen::Workbench);
        assert_eq!(new.screen, Screen::Workbench);
    });
}
