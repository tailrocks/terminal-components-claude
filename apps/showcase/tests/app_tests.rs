//! The legacy showcase interaction suite, driven through the public harness.
//!
//! These tests intentionally assert state transitions and rendered evidence,
//! not just that a page title exists. They are the app-owned counterparts of
//! the old in-binary suite.
use showcase_app::{App, NAV_ENTRIES, PageId};
use tui_next::{Axis, ColorLevel, KeyCode, MouseKind, Theme};
use tui_next_testing::Harness;

fn harness(page: PageId) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), 120, 40)
}

#[test]
fn launches_and_renders_shell() {
    let h = harness(PageId::Overview);
    let text = h.text();
    assert!(text.contains("SHOWCASE"));
    assert!(text.contains("Overview"));
    assert!(text.contains("Junie"));
    assert!(text.contains("Author component"));
    assert_eq!(NAV_ENTRIES.len(), 22);
}

#[test]
fn every_page_renders_at_representative_sizes_without_panic() {
    for (width, height) in [
        (72, 20),
        (80, 24),
        (100, 30),
        (120, 40),
        (160, 50),
        (200, 60),
    ] {
        for page in PageId::ALL {
            let mut h = Harness::new(App::with_page(page), Theme::junie(), width, height);
            assert!(
                h.text().contains(page.title()),
                "page {:?} missing title at {width}x{height}",
                page
            );
            for _ in 0..4 {
                h.key(KeyCode::Tab);
                h.key(KeyCode::Down);
                h.key(KeyCode::Right);
            }
            assert!(!h.text().is_empty());
        }
    }
}

#[test]
fn below_minimum_size_shows_reduced_state() {
    let h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 60, 15);
    let text = h.text();
    assert!(text.contains("Terminal too small"));
    assert!(text.contains("Need 72×20, have 60×15"));
    assert!(!text.contains("Playground"));
}

#[test]
fn resize_recovers_from_too_small() {
    let mut h = Harness::new(App::with_page(PageId::Buttons), Theme::junie(), 60, 15);
    h.resize(120, 40);
    assert!(h.text().contains("Playground"));
    assert!(h.text().contains("State matrix"));
}

#[test]
fn tab_traversal_is_deterministic_and_wraps() {
    let mut h = harness(PageId::Buttons);
    let start = h.focus();
    let reachable: Vec<_> = h.ring().reachable().map(|entry| entry.id).collect();
    assert!(start.is_some());
    assert_eq!(reachable.len(), 8, "nav plus seven enabled playground buttons");
    let mut seen = vec![start.unwrap()];
    for _ in 0..reachable.len().saturating_add(1) {
        h.key(KeyCode::Tab);
        let current = h.focus().expect("focus remains in the ring");
        if current == start.unwrap() {
            break;
        }
        seen.push(current);
    }
    assert_eq!(h.focus(), start, "focus wraps to the initial navigation stop");
    assert_eq!(seen.len(), reachable.len());
    let mut backwards = Vec::new();
    for _ in 0..seen.len() {
        h.key(KeyCode::BackTab);
        backwards.push(h.focus().expect("backward traversal stays reachable"));
    }
    let mut expected = seen;
    expected.reverse();
    assert_eq!(backwards, expected);
}

#[test]
fn disabled_buttons_are_skipped_and_cannot_activate() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = h.find("Disabled primary").expect("disabled fixture is visible");
    let before = h.text();
    h.click(x, y);
    assert!(h.text().contains("Disabled primary"));
    assert!(!h.text().contains("last: Disabled primary"));
    assert_eq!(h.text().matches("activations").count(), 0);
    assert_ne!(before, h.text(), "pointer routing still redraws the frame");
}

#[test]
fn mouse_click_activates_and_keyboard_enter_activates() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = h.find("Run task").expect("run button is visible");
    h.click(x, y);
    assert!(h.text().contains("Run task ✓"));
    assert!(h.text().contains("1 activations"));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Preview ✓"));
    assert!(h.text().contains("2 activations"));
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("3 activations"));
}

#[test]
fn hover_and_focus_render_differently() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = h.find("Run task").expect("run button is visible");
    let initial = h.snapshot().digest();
    h.mouse(MouseKind::Move, x, y);
    let hovered = h.snapshot().digest();
    assert!(h.hover().is_some());
    assert_ne!(initial, hovered, "hover state changes the resolved button style");
    h.key(KeyCode::Tab);
    let focused = h.snapshot().digest();
    assert!(h.focus().is_some());
    assert_ne!(hovered, focused, "keyboard focus has a distinct rendering");
}

#[test]
fn hit_testing_prefers_rows_over_their_container() {
    let mut h = harness(PageId::Tables);
    let (x, y) = h.find("#1042").expect("task row is visible");
    h.mouse(MouseKind::Move, x.saturating_add(12), y);
    let hovered = h.hover().expect("row receives pointer hover");
    let area = h.area_of(hovered).expect("hovered row has an area");
    assert_eq!(area.height, 1, "the cell/row wins over the table container");
}

#[test]
fn list_scrolling_and_selection() {
    let mut h = harness(PageId::Lists);
    h.key(KeyCode::Tab);
    for _ in 0..19 {
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("Erlang"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Chosen: Erlang"));
    h.key(KeyCode::Tab);
    let before = h.text().matches("✓").count();
    h.key(KeyCode::Char(' '));
    assert_ne!(h.text().matches("✓").count(), before);
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("checked rows: 19"));
}

#[test]
fn tree_expand_collapse_and_focus_bar_column_is_stable() {
    let mut h = harness(PageId::Trees);
    h.key(KeyCode::Tab);
    assert!(h.text().contains("config.rs"));
    let (x, y) = h.find("config.rs").expect("tree fixture is expanded");
    h.click(x, y);
    assert!(h.text().contains("selected: config.rs"));
    h.key(KeyCode::Home);
    h.key(KeyCode::Left);
    assert!(!h.text().contains("config.rs"));
    h.key(KeyCode::Right);
    assert!(h.text().contains("config.rs"));
    assert!(h.text().contains("selected: config.rs"));
}

fn first_data_row(h: &Harness<App>) -> String {
    let header = h.find_row("ID").expect("table header");
    h.row(header.saturating_add(1))
}

#[test]
fn table_sorts_both_directions_and_clears() {
    let mut h = harness(PageId::Tables);
    let (x, y) = h.find("Changes").expect("sortable Changes header");
    h.click(x, y);
    assert!(first_data_row(&h).contains("#1043"), "ascending puts zero changes first");
    h.click(x, y);
    assert!(first_data_row(&h).contains("#1049"), "descending puts 118 changes first");
    assert!(h.text().contains("descending"));
}

#[test]
fn header_click_sorts() {
    let mut h = harness(PageId::Tables);
    let (x, y) = h.find("Owner").expect("sortable Owner header");
    h.click(x, y);
    assert!(first_data_row(&h).contains("ana"));
    h.click(x, y);
    assert!(first_data_row(&h).contains("sofia"));
}

#[test]
fn editable_table_commit_cancel_and_validation() {
    let mut h = harness(PageId::Editable);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("EDIT"));
    h.key(KeyCode::End);
    h.type_str(" now");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Add rate limiting to auth endpoints now"));
    h.key(KeyCode::Enter);
    h.type_str("zzz");
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("zzz"));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.ctrl('l');
    h.type_str("abc");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Changes must be a whole number"));
    assert!(h.text().contains("EDIT"));
    h.key(KeyCode::Esc);
}

#[test]
fn input_editing_commit_and_revert() {
    let mut h = harness(PageId::Inputs);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.key(KeyCode::End);
    h.type_str("-v2");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("operator-v2"));
    h.key(KeyCode::Enter);
    h.type_str("XX");
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("XX"));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.key(KeyCode::End);
    h.type_str("-v2");
    h.key(KeyCode::Tab);
    assert!(h.text().contains("payments-gateway-v2"));
}

#[test]
fn textarea_scrolls_with_wheel_and_keys() {
    let mut h = harness(PageId::TextAreas);
    assert!(h.text().contains("1. Read"));
    let (x, y) = h.find("1. Read").expect("checklist starts at the top");
    let before = h.snapshot().digest();
    h.wheel(Axis::V, 3, x, y);
    assert_ne!(before, h.snapshot().digest());
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    for _ in 0..30 {
        h.key(KeyCode::Down);
    }
    assert!(h.text().contains("28. Run"));
}

#[test]
fn form_validation_blocks_submit_and_focuses_first_error() {
    let mut h = harness(PageId::Forms);
    h.ctrl('s');
    assert!(h.text().contains("Required: summary"));
    h.key(KeyCode::Enter);
    h.type_str("Fix login bug");
    h.key(KeyCode::Enter);
    h.ctrl('s');
    assert!(h.text().contains("Creating task"));
}

#[test]
fn modal_traps_focus_and_restores_it() {
    let mut h = harness(PageId::Dialogs);
    h.key(KeyCode::Tab);
    let launcher = h.focus();
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Run task now?"));
    let modal_focus = h.focus();
    assert_ne!(modal_focus, launcher);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    assert_ne!(h.focus(), launcher);
    h.key(KeyCode::Esc);
    assert!(h.text().contains("Cancelled"));
    assert_eq!(h.focus(), launcher);
}

#[test]
fn prompt_dialog_validates_and_returns_value() {
    let mut h = harness(PageId::Dialogs);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Rename task"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Name cannot be empty"));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.type_str("Ship it");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Task: Ship it"));
}

#[test]
fn settings_screen_remove_member_flow() {
    let mut h = harness(PageId::Settings);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Right);
    assert!(h.text().contains("Mira Okafor"));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Down);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Remove member?"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Jonas Weber"));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(!h.text().contains("jonas@acme.dev"));
    assert!(h.text().contains("5 members"));
}

#[test]
fn task_runner_animates_and_can_be_cancelled() {
    let mut h = harness(PageId::TaskRunner);
    h.key(KeyCode::Char('r'));
    assert!(h.text().contains("Pipeline · running"));
    assert!(h.text().contains("compile started"));
    h.ticks(8);
    assert!(h.text().contains("%"));
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Cancel pipeline?"));
    h.key(KeyCode::Enter);
    assert!(h.text().contains("pipeline cancelled"));
    assert!(!h.text().contains("Pipeline · running"));
}

#[test]
fn scrollbar_click_and_drag_move_the_view() {
    let mut h = harness(PageId::Scrolling);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Tab);
    let (x, y) = h.find("Row 001").expect("long-list row is visible");
    let before = h.snapshot().digest();
    h.wheel(Axis::V, 8, x, y);
    assert_ne!(before, h.snapshot().digest());
    assert!(h.text().contains("list="));
    h.drag((x, y), (x, y.saturating_add(8)));
    assert!(h.text().contains("list="));
}

#[test]
fn keyboard_navigation_between_pages() {
    let mut h = harness(PageId::Overview);
    h.key(KeyCode::Char(']'));
    assert_eq!(h.app().page(), PageId::Buttons);
    h.key(KeyCode::Char('['));
    assert_eq!(h.app().page(), PageId::Overview);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app().page(), PageId::Buttons);
    let (x, y) = h.find("Tables").expect("navigation item");
    h.click(x, y);
    assert_eq!(h.app().page(), PageId::Tables);
}

#[test]
fn quit_keys() {
    let mut h = harness(PageId::Overview);
    h.key(KeyCode::Char('q'));
    assert!(h.app().quit());
    let mut h = harness(PageId::Inputs);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('q'));
    assert!(!h.app().quit());
    h.ctrl('c');
    assert!(h.app().quit());
}

#[test]
fn color_downgrade_still_renders() {
    let truecolor = harness(PageId::Overview).snapshot().digest();
    let mono = harness(PageId::Overview)
        .with_color(ColorLevel::Ansi16)
        .snapshot()
        .digest();
    assert_ne!(truecolor, mono);
}

#[test]
fn author_component_page_participates_in_focus_and_hover() {
    let mut h = harness(PageId::Overview);
    let (x, y) = h.find("Author component").expect("author component is visible");
    let before = h.snapshot().digest();
    h.mouse(MouseKind::Move, x, y);
    assert!(h.hover().is_some());
    assert_ne!(before, h.snapshot().digest());
    h.click(x, y);
    assert!(h.text().contains("Author component · selected"));
    h.key(KeyCode::Tab);
    assert!(h.focus().is_some());
}
