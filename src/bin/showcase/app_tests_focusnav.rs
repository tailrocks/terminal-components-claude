//! The Focus & navigation page (plan §7.5, SC28), driven through the real
//! app: the ring read back, the shell's focus fallback, Tab in a field,
//! per-widget nav keys, and the shell's input and protocol options.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::focus_navigation::FocusNavPage;
use crate::pages::template::Section;
use junie_tui::core::keyboard::KeyboardProtocol;
use junie_tui::ui::specimen::SpecimenState;

fn page(h: &Harness) -> &FocusNavPage {
    h.app.pages[PageId::FocusNavigation.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<FocusNavPage>())
        .expect("the Focus & navigation page")
}

fn harness() -> Harness {
    Harness::on(120, 40, PageId::FocusNavigation)
}

#[test]
fn tab_moves_through_the_ring_and_the_page_reads_the_change_back() {
    let mut h = harness();
    let name = page(&h).name.id;
    h.focus(name);
    h.key(KeyCode::Tab);
    assert_eq!(h.app.shell.focus.current(), Some(page(&h).email.id));
    assert_eq!(page(&h).last_change, "Name field → Email field");
    assert!(
        h.text().contains("Email field"),
        "the readout names the focus"
    );
    assert!(h.text().contains("stops · reading order"));
}

#[test]
fn a_removed_focused_tag_falls_back_per_the_shell_option() {
    // the showcase shows the library default (`FocusFallback::Nearest`)
    let mut h = harness();
    let (alpha, beta) = (page(&h).tags[0].id, page(&h).tags[1].id);
    h.focus(beta);
    h.key(KeyCode::Delete);
    assert_eq!(page(&h).tags.len(), 3);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(alpha),
        "default nearest: the stop before the removed one"
    );
    assert_eq!(h.bars(), 1);
    let mut h = harness();
    h.opt("shell.focus_fallback=first");
    let beta = page(&h).tags[1].id;
    h.focus(beta);
    h.key(KeyCode::Delete);
    let first = h.app.shell.ring.first();
    assert_eq!(
        h.app.shell.focus.current(),
        first,
        "first: the ring's first stop"
    );
    assert_eq!(h.bars(), 1);
}

#[test]
fn the_showcase_focus_fallback_is_the_library_default() {
    let h = harness();
    assert_eq!(
        h.app.shell.focus_fallback,
        junie_tui::core::focus::FocusFallback::default(),
        "the showcase shows library defaults unless an option says otherwise"
    );
}

#[test]
fn tab_in_a_field_moves_on_unless_the_page_cancels_the_request() {
    let mut h = harness();
    let (name, email) = (page(&h).name.id, page(&h).email.id);
    h.focus(name);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(email),
        "request_focus_next"
    );
    h.opt("focusnavigation.tab_in_field=stays");
    h.focus(name);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(name),
        "cancel_focus_request"
    );
    assert!(h.text().contains("stays: cancel_focus_request"));
}

#[test]
fn the_lists_nav_keys_follow_the_option_and_the_map_shows_them() {
    let mut h = harness();
    let list = page(&h).list.id;
    h.focus(list);
    h.key(KeyCode::Char('j'));
    assert_eq!(page(&h).list.cursor, 0, "desktop keys: j does not move");
    h.opt("focusnavigation.nav=vim");
    h.focus(list);
    h.key(KeyCode::Char('j'));
    assert_eq!(page(&h).list.cursor, 1, "vim: j moves down");
    assert!(h.text().contains("↓ j"), "the Nav map lists j for Down");
    assert!(h.text().contains("last: Down"));
    h.opt("focusnavigation.nav=no-page");
    let (_, y) = h.find("PageDown").expect("the map row");
    assert!(h.row(y).contains('—'), "no key pages");
}

#[test]
fn the_shell_input_and_protocol_options_reach_the_runtime_hooks() {
    let mut h = harness();
    assert!(h.app.input_config().shift_wheel_horizontal);
    h.opt("shell.shift_wheel_horizontal=off");
    assert!(!h.app.input_config().shift_wheel_horizontal);
    h.opt("shell.legacy_ctrl_punctuation=off");
    assert!(!h.app.input_config().legacy_ctrl_punctuation);
    h.opt("shell.platform=macos");
    assert!(
        h.app.input_config().ctrl_click_secondary,
        "macOS: Ctrl+click"
    );
    h.opt("shell.platform=other");
    assert!(!h.app.input_config().ctrl_click_secondary);
    h.opt("shell.keyboard_protocol=off");
    assert_eq!(h.app.keyboard_protocol(), KeyboardProtocol::Off);
    h.opt("shell.keyboard_protocol=on");
    assert_eq!(h.app.keyboard_protocol(), KeyboardProtocol::On);
    // A29: the Alt detection an app or the user overrides
    h.opt("shell.alt_keys=refused");
    assert!(!junie_tui::core::keyboard::alt_delivered());
    h.opt("shell.alt_keys=delivered");
    assert!(junie_tui::core::keyboard::alt_delivered());
    assert!(h.text().contains("Alt keys"), "{}", h.text());
}

#[test]
fn the_states_grid_shows_how_each_control_takes_focus() {
    let mut h = harness();
    h.section(Section::States);
    let rows = h.app.pages[PageId::FocusNavigation.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    use SpecimenState::*;
    for s in [Rest, Hover, Focused, FocusedHover] {
        assert!(rows.contains(&s), "{s:?}");
    }
    assert!(h.bars() <= 1, "samples depict focus");
}
