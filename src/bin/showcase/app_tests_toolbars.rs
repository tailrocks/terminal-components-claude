//! The Toolbars page (plan §7.5, SC28), driven through the real app: one
//! focus stop per toolbar, arrows inside it, Enter and clicks running
//! commands and toggles, the options and the States grid.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::template::Section;
use crate::pages::toolbars::ToolbarsPage;
use junie_tui::ui::specimen::SpecimenState;

fn page(h: &Harness) -> &ToolbarsPage {
    h.app.pages[PageId::Toolbars.index()]
        .as_any()
        .and_then(|a| a.downcast_ref::<ToolbarsPage>())
        .expect("the Toolbars page")
}

fn harness() -> Harness {
    Harness::on(120, 40, PageId::Toolbars)
}

#[test]
fn a_toolbar_is_one_tab_stop_and_arrows_move_inside_it() {
    let mut h = harness();
    let (query, format) = (page(&h).query.id, page(&h).format.id);
    h.focus(query);
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.focus.current(),
        Some(format),
        "Tab leaves the toolbar"
    );
    h.focus(query);
    // Run, Run all, then Stop is disabled: → skips it to Format
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    assert_eq!(page(&h).query.cursor, 3);
    assert_eq!(h.bars(), 1, "one ▎, on the cursor item");
    let x = page(&h).query.areas[3].x;
    let (_, y) = h.find("Format").unwrap();
    assert_eq!(h.row(y).chars().nth(x as usize), Some('▎'));
}

#[test]
fn enter_runs_a_command_and_the_run_state_enables_stop() {
    let mut h = harness();
    let query = page(&h).query.id;
    h.focus(query);
    h.key(KeyCode::Enter);
    assert_eq!(page(&h).last, "Activated(0) · Run");
    assert!(page(&h).running);
    assert!(page(&h).query.items[0].disabled, "Run waits");
    assert!(!page(&h).query.items[2].disabled, "Stop is enabled");
    let (x, y) = h.find("Stop").unwrap();
    h.click(x + 1, y);
    assert_eq!(page(&h).last, "Activated(2) · Stop");
    assert!(!page(&h).running);
    let (x, y) = h.find("Bold").unwrap();
    h.click(x, y);
    assert_eq!(page(&h).last, "Toggled(0, true) · Bold");
    assert_eq!(h.app.focus.current(), Some(page(&h).format.id));
}

#[test]
fn options_rebuild_the_toolbars_through_their_builders() {
    let mut h = harness();
    let query = page(&h).query.id;
    h.opt("toolbars.skip_disabled=off");
    h.focus(query);
    h.key(KeyCode::Right);
    h.key(KeyCode::Right);
    assert_eq!(page(&h).query.cursor, 2, "disabled Stop is reachable");
    h.opt("toolbars.wrap=on");
    h.focus(query);
    h.key(KeyCode::End);
    h.key(KeyCode::Right);
    assert_eq!(page(&h).query.cursor, 0, "wraps");
    let before = page(&h).query.areas[1].x;
    h.opt("toolbars.gap=2");
    assert_eq!(
        page(&h).query.areas[1].x,
        before + 1,
        "one more cell between"
    );
    h.opt("toolbars.nav=vim");
    h.focus(query);
    h.key(KeyCode::Char('l'));
    assert_eq!(page(&h).query.cursor, 1, "vim: l moves right");
}

#[test]
fn the_states_grid_draws_every_toolbar_state() {
    let mut h = harness();
    h.section(Section::States);
    let rows = h.app.pages[PageId::Toolbars.index()]
        .template()
        .unwrap()
        .state_rows()
        .to_vec();
    use SpecimenState::*;
    for s in [
        Rest,
        Hover,
        Pressed,
        Focused,
        FocusedHover,
        Selected,
        Disabled,
        Narrow,
    ] {
        assert!(rows.contains(&s), "{s:?}");
    }
    assert!(h.bars() <= 1);
}
