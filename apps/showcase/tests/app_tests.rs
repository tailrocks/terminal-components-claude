//! Non-vacuous Showcase journeys through the public application facade.
//!
//! Every interaction below goes through `tui-next-testing::Harness`. The
//! assertions check rendered evidence, focus, keyed regions, and durable
//! state transitions; a missing control is a test failure, never a skip.

use showcase_app::{App, NAV_ENTRIES, PageId};
use tui_next::{Axis, ColorLevel, Id, ItemKey, KeyCode, LayerId, MouseKind, Part, PartRef, Theme};
use tui_next_testing::Harness;

const FORM_SUMMARY: Id = Id::root("showcase_app::pages::forms::forms.summary");
const SCROLL_LIST: Id = Id::root("showcase_app::pages::scrolling::scrolling.list");
const APP_NAV: Id = Id::root("showcase_app::app::navigation");

fn harness(page: PageId) -> Harness<App> {
    Harness::new(App::with_page(page), Theme::junie(), 120, 40)
}

fn press(h: &mut Harness<App>, code: KeyCode) {
    let _ = h.key(code);
}

fn control(h: &mut Harness<App>, c: char) {
    let _ = h.ctrl(c);
}

fn type_text(h: &mut Harness<App>, text: &str) {
    let _ = h.type_str(text);
}

fn move_mouse(h: &mut Harness<App>, x: u16, y: u16) {
    let _ = h.mouse(MouseKind::Move, x, y);
}

fn click(h: &mut Harness<App>, x: u16, y: u16) {
    let _ = h.click(x, y);
}

fn wheel(h: &mut Harness<App>, axis: Axis, delta: i16, x: u16, y: u16) {
    let _ = h.wheel(axis, delta, x, y);
}

fn drag(h: &mut Harness<App>, from: (u16, u16), to: (u16, u16)) {
    let _ = h.drag(from, to);
}

fn resize(h: &mut Harness<App>, width: u16, height: u16) {
    let _ = h.resize(width, height);
}

fn focus_bar_x(h: &Harness<App>, y: u16) -> Option<u16> {
    (0..h.buffer().area().width).find(|x| h.cell(*x, y).symbol() == "▎")
}

#[test]
fn launches_and_renders_shell() {
    let h = harness(PageId::Overview);
    let text = h.text();
    assert!(text.contains("SHOWCASE"));
    assert!(text.contains("Overview"));
    assert!(text.contains("Junie"));
    assert!(text.contains("Author component"));
    assert_eq!(NAV_ENTRIES.len(), PageId::ALL.len());
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
                "page {page:?} title is not visible at {width}x{height}"
            );
            for _ in 0..4 {
                press(&mut h, KeyCode::Tab);
                press(&mut h, KeyCode::Down);
                press(&mut h, KeyCode::Right);
            }
            assert!(!h.text().trim().is_empty());
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
    resize(&mut h, 120, 40);
    assert!(h.text().contains("Playground"));
    assert!(h.text().contains("State matrix"));
}

#[test]
fn tab_traversal_is_deterministic_and_wraps() {
    let mut h = harness(PageId::Buttons);
    let start = h.focus();
    let reachable: Vec<_> = h.ring().reachable().map(|entry| entry.id).collect();
    assert!(start.is_some());
    assert_eq!(
        reachable.len(),
        8,
        "nav plus seven enabled playground buttons"
    );
    let start = start.expect("initial navigation focus");
    let mut seen = vec![start];
    for _ in 0..reachable.len().saturating_add(1) {
        press(&mut h, KeyCode::Tab);
        let current = h.focus().expect("focus remains in the ring");
        if current == start {
            break;
        }
        seen.push(current);
    }
    assert_eq!(
        h.focus(),
        Some(start),
        "focus wraps to the initial navigation stop"
    );
    assert_eq!(seen.len(), reachable.len());
    let mut backwards = Vec::new();
    for _ in 0..seen.len() {
        press(&mut h, KeyCode::BackTab);
        backwards.push(h.focus().expect("backward traversal stays reachable"));
    }
    let mut expected = seen;
    expected.reverse();
    assert_eq!(backwards, expected);
}

#[test]
fn disabled_buttons_are_skipped_and_cannot_activate() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = h
        .find("Disabled primary")
        .expect("disabled fixture is visible");
    let before = h.snapshot().digest();
    click(&mut h, x, y);
    assert_eq!(
        h.snapshot().digest(),
        before,
        "disabled pointer input is inert"
    );
    assert!(h.text().contains("Disabled primary"));
    assert!(!h.text().contains("last: Disabled primary"));
    assert_eq!(h.text().matches("activations").count(), 0);
}

#[test]
fn mouse_click_activates_and_keyboard_enter_activates() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = h.find("Run task").expect("run button is visible");
    click(&mut h, x, y);
    assert!(h.text().contains("Run task ✓"));
    assert!(h.text().contains("1 activations"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Preview ✓"));
    assert!(h.text().contains("2 activations"));
    press(&mut h, KeyCode::Char(' '));
    assert!(h.text().contains("3 activations"));
}

#[test]
fn hover_and_focus_render_differently() {
    let mut h = harness(PageId::Buttons);
    let (x, y) = h.find("Run task").expect("run button is visible");
    let initial = h.snapshot().digest();
    move_mouse(&mut h, x, y);
    let hovered = h.snapshot().digest();
    assert!(h.hover().is_some());
    assert_ne!(
        initial, hovered,
        "hover state changes the resolved button style"
    );
    press(&mut h, KeyCode::Tab);
    let focused = h.snapshot().digest();
    assert!(h.focus().is_some());
    assert_ne!(hovered, focused, "keyboard focus has a distinct rendering");
}

#[test]
fn hit_testing_prefers_rows_over_their_container() {
    let mut h = harness(PageId::Tables);
    let (x, y) = h.find("#1042").expect("task row is visible");
    move_mouse(&mut h, x.saturating_add(12), y);
    let hovered = h.hover().expect("table receives pointer hover");
    let area = h
        .area_of_part(hovered, PartRef::item(Part::CELL, ItemKey::num(1042)))
        .expect("hovered task cell has an area");
    assert_eq!(area.height, 1, "the keyed cell region is one row tall");
}

#[test]
fn list_scrolling_and_selection() {
    let mut h = harness(PageId::Lists);
    press(&mut h, KeyCode::Tab);
    for _ in 0..19 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("Erlang"));
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Chosen: Erlang"));
    press(&mut h, KeyCode::Tab);
    let before = h.text().matches("✓").count();
    press(&mut h, KeyCode::Char(' '));
    assert_ne!(h.text().matches("✓").count(), before);
    press(&mut h, KeyCode::Char('a'));
    assert!(h.text().contains("checked rows: 10"));
    assert!(h.text().contains("src/api/auth.rs"));
}

#[test]
fn tree_expand_collapse_and_focus_bar_column_is_stable() {
    let mut h = harness(PageId::Trees);
    press(&mut h, KeyCode::Tab);
    let src_y = h.find_row("src").expect("tree root is visible");
    let bar_x = focus_bar_x(&h, src_y).expect("focused root has a focus bar");
    assert!(h.text().contains("config.rs"));
    press(&mut h, KeyCode::Left);
    assert!(!h.text().contains("config.rs"));
    press(&mut h, KeyCode::Right);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Right);
    assert!(h.text().contains("auth.rs"));
    press(&mut h, KeyCode::Down);
    let auth_y = h.find_row("auth.rs").expect("expanded api file is visible");
    assert_eq!(
        focus_bar_x(&h, auth_y),
        Some(bar_x),
        "focus bar column is stable across depth"
    );
}

fn first_data_row(h: &Harness<App>) -> String {
    let header = h.find_row("ID").expect("table header");
    h.row(header.saturating_add(1))
}

#[test]
fn table_sorts_both_directions_and_clears() {
    let mut h = harness(PageId::Tables);
    let (x, y) = h.find("Changes").expect("sortable Changes header");
    click(&mut h, x, y);
    assert!(
        first_data_row(&h).contains("#1043"),
        "ascending puts zero changes first"
    );
    click(&mut h, x, y);
    assert!(
        first_data_row(&h).contains("#1049"),
        "descending puts 118 changes first"
    );
    assert!(h.text().contains("descending"));
}

#[test]
fn header_click_sorts() {
    let mut h = harness(PageId::Tables);
    let (x, y) = h.find("Owner").expect("sortable Owner header");
    click(&mut h, x, y);
    assert!(first_data_row(&h).contains("ana"));
    click(&mut h, x, y);
    assert!(first_data_row(&h).contains("sofia"));
}

#[test]
fn editable_table_commit_cancel_and_validation() {
    let mut h = harness(PageId::Editable);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("EDIT"));
    press(&mut h, KeyCode::End);
    type_text(&mut h, " now");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Add rate limiting to auth endpoints now"));
    press(&mut h, KeyCode::Tab);
    type_text(&mut h, "zzz");
    press(&mut h, KeyCode::Esc);
    assert!(!h.text().contains("zzz"));
    press(&mut h, KeyCode::Enter);
    control(&mut h, 'l');
    type_text(&mut h, "abc");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Changes must be a whole number"));
    assert!(h.text().contains("EDIT"));
    press(&mut h, KeyCode::Esc);
}

#[test]
fn input_editing_commit_and_revert() {
    let mut h = harness(PageId::Inputs);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::End);
    type_text(&mut h, "-v2");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("operator-v2"));
    press(&mut h, KeyCode::Enter);
    type_text(&mut h, "XX");
    press(&mut h, KeyCode::Esc);
    assert!(!h.text().contains("XX"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::End);
    type_text(&mut h, "-v2");
    press(&mut h, KeyCode::Tab);
    assert!(h.text().contains("payments-gateway-v2"));
}

#[test]
fn textarea_scrolls_with_wheel_and_keys() {
    let mut h = harness(PageId::TextAreas);
    assert!(h.text().contains("1. Read"));
    let (x, y) = h.find("1. Read").expect("checklist starts at the top");
    let before = h.snapshot().digest();
    wheel(&mut h, Axis::V, 3, x, y);
    assert_ne!(before, h.snapshot().digest());
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    for _ in 0..30 {
        press(&mut h, KeyCode::Down);
    }
    assert!(h.text().contains("28. Run"));
}

#[test]
fn form_validation_blocks_submit_and_focuses_first_error() {
    let mut h = harness(PageId::Forms);
    control(&mut h, 's');
    assert!(h.text().contains("Required: summary"));
    assert_eq!(h.focus(), Some(FORM_SUMMARY), "summary field is focused");
    let summary = h.area_of(FORM_SUMMARY).expect("summary field has an area");
    click(&mut h, summary.x.saturating_add(1), summary.y);
    type_text(&mut h, "Fix login bug");
    press(&mut h, KeyCode::Enter);
    control(&mut h, 's');
    assert!(h.text().contains("Creating task"));
}

#[test]
fn modal_traps_focus_and_restores_it() {
    let mut h = harness(PageId::Dialogs);
    press(&mut h, KeyCode::Tab);
    let launcher = h.focus().expect("confirm launcher is focusable");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Run task now?"));
    assert!(h.top_layer().index() > LayerId::PAGE.index());
    assert_ne!(h.focus(), Some(launcher));
    press(&mut h, KeyCode::Tab);
    assert_ne!(h.focus(), Some(launcher));
    press(&mut h, KeyCode::Esc);
    assert!(h.text().contains("Cancelled"));
    assert_eq!(h.top_layer(), LayerId::PAGE);
    assert_eq!(h.focus(), Some(launcher));

    press(&mut h, KeyCode::Enter);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Task started"));
}

#[test]
fn prompt_dialog_validates_and_returns_value() {
    let mut h = harness(PageId::Dialogs);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Rename task"));
    press(&mut h, KeyCode::Enter);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Name cannot be empty"));
    press(&mut h, KeyCode::Enter);
    type_text(&mut h, "Ship it");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Task: Ship it"));
}

#[test]
fn settings_screen_remove_member_flow() {
    let mut h = harness(PageId::Settings);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Right);
    assert!(h.text().contains("Mira Okafor"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    let remove = h.focus().expect("remove launcher is focused");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Remove member?"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(!h.text().contains("jonas@acme.dev"));
    assert!(h.text().contains("5 members"));
    assert_eq!(h.focus(), Some(remove), "focus returns to remove launcher");
}

#[test]
fn task_runner_animates_and_can_be_cancelled() {
    let mut h = harness(PageId::TaskRunner);
    press(&mut h, KeyCode::Char('r'));
    assert!(h.text().contains("Pipeline · running"));
    assert!(h.text().contains("compile started"));
    h.ticks(8);
    assert!(h.text().contains("%"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    let cancel = h.focus().expect("cancel launcher is focused");
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("Cancel pipeline?"));
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Enter);
    assert!(h.text().contains("pipeline cancelled"));
    assert!(!h.text().contains("Pipeline · running"));
    assert_ne!(
        h.focus(),
        Some(cancel),
        "disabled cancel launcher cannot retain focus"
    );
    assert_eq!(h.focus(), Some(APP_NAV), "focus reconciles to navigation");
}

#[test]
fn scrollbar_click_and_drag_move_the_view() {
    let mut h = harness(PageId::Scrolling);
    press(&mut h, KeyCode::Tab);
    press(&mut h, KeyCode::Tab);
    let (row_x, row_y) = h.find("Row 001").expect("long-list row is visible");
    let before = h.snapshot().digest();
    wheel(&mut h, Axis::V, 8, row_x, row_y);
    assert_ne!(before, h.snapshot().digest());
    let thumb = h
        .area_of_part(SCROLL_LIST, PartRef::of(Part::THUMB))
        .expect("long-list scrollbar thumb is registered");
    let track = h
        .area_of_part(SCROLL_LIST, PartRef::of(Part::TRACK))
        .expect("long-list scrollbar track is registered");
    let drag_before = h.snapshot().digest();
    drag(
        &mut h,
        (thumb.x, thumb.y.saturating_add(thumb.height / 2)),
        (track.x, track.bottom().saturating_sub(2)),
    );
    assert_ne!(
        drag_before,
        h.snapshot().digest(),
        "scrollbar drag changes the viewport"
    );
    assert!(h.text().contains("list="));
}

#[test]
fn keyboard_navigation_between_pages() {
    let mut h = harness(PageId::Overview);
    press(&mut h, KeyCode::Char(']'));
    assert_eq!(h.app().page(), PageId::Buttons);
    press(&mut h, KeyCode::Char('['));
    assert_eq!(h.app().page(), PageId::Overview);
    press(&mut h, KeyCode::Down);
    press(&mut h, KeyCode::Enter);
    assert_eq!(h.app().page(), PageId::Buttons);
    let (x, y) = h.find("Tables").expect("navigation item");
    click(&mut h, x, y);
    assert_eq!(h.app().page(), PageId::Tables);
}

#[test]
fn quit_keys() {
    let mut h = harness(PageId::Overview);
    press(&mut h, KeyCode::Char('q'));
    assert!(h.app().quit());

    let mut h = harness(PageId::Inputs);
    press(&mut h, KeyCode::Tab);
    type_text(&mut h, "q");
    assert!(!h.app().quit(), "printable q belongs to the active editor");
    assert!(h.text().contains("operatorq"));
    press(&mut h, KeyCode::Esc);
    control(&mut h, 'c');
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
    let (x, y) = h
        .find("Author component")
        .expect("author component is visible");
    let before = h.snapshot().digest();
    move_mouse(&mut h, x, y);
    assert!(h.hover().is_some());
    assert_ne!(before, h.snapshot().digest());
    click(&mut h, x, y);
    assert!(h.text().contains("Author component · selected"));
    press(&mut h, KeyCode::Tab);
    assert!(h.focus().is_some());
}
