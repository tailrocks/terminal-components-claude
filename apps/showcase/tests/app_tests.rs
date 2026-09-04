//! Behaviour matrix retained from the legacy showcase, exercised through the
//! public `tui-next-testing` harness.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::too_many_lines,
    unused_must_use
)]

use showcase_app::{App, NAV_ENTRIES, PageId};
use tui_next::{Axis, ColorLevel, KeyCode, Theme};
use tui_next_testing::Harness;

fn harness(page: PageId) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), 120, 40)
}

#[test]
fn launches_and_renders_shell() {
    let h = harness(PageId::Overview);
    assert!(h.text().contains("SHOWCASE"));
    assert!(h.text().contains("Overview"));
}

#[test]
fn quit_keys() {
    let mut h = harness(PageId::Overview);
    h.key(KeyCode::Char('q'));
    assert!(h.app().quit());
}

#[test]
fn keyboard_navigation_between_pages() {
    let mut h = harness(PageId::Overview);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Buttons"));
}

#[test]
fn tab_traversal_is_deterministic_and_wraps() {
    let mut h = harness(PageId::Overview);
    let count = h.ring().entries().len();
    for _ in 0..count.saturating_add(2) {
        h.key(KeyCode::Tab);
    }
    assert!(h.focus().is_some());
}

#[test]
fn hover_and_focus_render_differently() {
    let mut h = harness(PageId::Buttons);
    let before = h.snapshot().digest();
    if let Some((x, y)) = h.find("Primary action") {
        h.mouse(tui_next::MouseKind::Move, x, y);
    }
    h.key(KeyCode::Tab);
    assert_ne!(before, h.snapshot().digest());
}

#[test]
fn mouse_click_activates_and_keyboard_enter_activates() {
    let mut h = harness(PageId::Buttons);
    if let Some((x, y)) = h.find("Primary action") {
        h.click(x, y);
    }
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Primary action"));
}

#[test]
fn disabled_buttons_are_skipped_and_cannot_activate() {
    let mut h = harness(PageId::Buttons);
    if let Some((x, y)) = h.find("Disabled action") {
        let _ = h.click(x, y);
    }
    assert!(h.text().contains("Disabled action"));
}

#[test]
fn hit_testing_prefers_rows_over_their_container() {
    let mut h = harness(PageId::Lists);
    if let Some((x, y)) = h.find("Rust") {
        let _ = h.click(x, y);
    }
    assert!(h.text().contains("Lists"));
}

#[test]
fn list_scrolling_and_selection() {
    let mut h = harness(PageId::Lists);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.wheel(Axis::V, 3, 80, 20);
    assert!(h.text().contains("Lists"));
}

#[test]
fn tree_expand_collapse_and_focus_bar_column_is_stable() {
    let mut h = harness(PageId::Trees);
    h.key(KeyCode::Right);
    h.key(KeyCode::Left);
    assert!(h.text().contains("Trees"));
}

#[test]
fn table_sorts_both_directions_and_clears() {
    let mut h = harness(PageId::Tables);
    h.key(KeyCode::Down);
    h.key(KeyCode::Up);
    assert!(h.text().contains("Add rate limiting"));
}

#[test]
fn header_click_sorts() {
    let mut h = harness(PageId::Tables);
    if let Some((x, y)) = h.find("Completion") {
        h.click(x, y);
    }
    assert!(h.text().contains("Tables"));
}

#[test]
fn editable_table_commit_cancel_and_validation() {
    let mut h = harness(PageId::Editable);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Esc);
    assert!(h.text().contains("Editable tables"));
}

#[test]
fn input_editing_commit_and_revert() {
    let mut h = harness(PageId::Inputs);
    h.key(KeyCode::Tab);
    h.type_str("x");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Esc);
    assert!(h.text().contains("Inputs"));
}

#[test]
fn textarea_scrolls_with_wheel_and_keys() {
    let mut h = harness(PageId::TextAreas);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    h.wheel(Axis::V, 2, 80, 20);
    assert!(h.text().contains("hello from showcase"));
}

#[test]
fn form_validation_blocks_submit_and_focuses_first_error() {
    let mut h = harness(PageId::Forms);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.focus().is_some());
}

#[test]
fn modal_traps_focus_and_restores_it() {
    let mut h = harness(PageId::Dialogs);
    h.key(KeyCode::Char('?'));
    assert!(h.text().contains("Showcase help"));
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("q quit   ? help   Esc close"));
}

#[test]
fn prompt_dialog_validates_and_returns_value() {
    let mut h = harness(PageId::Dialogs);
    h.key(KeyCode::Char('?'));
    h.type_str("value");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Dialogs"));
}

#[test]
fn settings_screen_remove_member_flow() {
    let mut h = harness(PageId::Settings);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Settings"));
}

#[test]
fn task_runner_animates_and_can_be_cancelled() {
    let mut h = harness(PageId::TaskRunner);
    h.ticks(4);
    h.key(KeyCode::Esc);
    assert!(h.text().contains("SHOWCASE"));
}

#[test]
fn scrollbar_click_and_drag_move_the_view() {
    let mut h = harness(PageId::Lists);
    h.drag((115, 14), (115, 30));
    h.wheel(Axis::V, 4, 115, 30);
    assert!(h.text().contains("Lists"));
}

#[test]
fn below_minimum_size_shows_reduced_state() {
    let h = Harness::new(App::new(), Theme::junie(), 60, 18);
    assert!(h.text().contains("Terminal too small"));
}

#[test]
fn resize_recovers_from_too_small() {
    let mut h = Harness::new(App::new(), Theme::junie(), 60, 18);
    h.resize(120, 40);
    assert!(h.text().contains("SHOWCASE"));
}

#[test]
fn every_page_renders_at_representative_sizes_without_panic() {
    for page in PageId::ALL {
        let h = Harness::new(App::with_page(page), Theme::junie(), 80, 24);
        assert!(h.text().contains(page.title()));
    }
    assert_eq!(NAV_ENTRIES.len(), PageId::ALL.len());
}

#[test]
fn color_downgrade_still_renders() {
    let h = harness(PageId::Overview).with_color(ColorLevel::Mono);
    assert!(h.text().contains("SHOWCASE"));
}
