//! The Toolbars page (plan §7.5, SC28), driven through the real app: one
//! focus stop per toolbar, arrows inside it, Enter and clicks running
//! commands and toggles, the options and the States grid.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_tabs::Harness;
use crate::pages::template::Section;
use crate::pages::toolbars::ToolbarsPage;
use junie_tui::core::id::{ItemKey, Part, PartRef};
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
    let (query, narrow) = (page(&h).query.id, page(&h).narrow.id);
    h.focus(query);
    h.key(KeyCode::Tab);
    assert_eq!(
        h.app.shell.focus.current(),
        Some(narrow),
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
    assert_eq!(h.app.shell.focus.current(), Some(page(&h).format.id));
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

/// Items that do not fit the narrow toolbar sit behind `»`, the last
/// arrow stop: Enter opens a menu of them (shortcuts, toggle state, why a
/// disabled one waits) above the page, and choosing one runs it.
#[test]
fn the_narrow_toolbar_keeps_hidden_commands_in_its_overflow_menu() {
    let mut h = harness();
    let narrow = page(&h).narrow.id;
    assert_eq!(page(&h).narrow.hidden(), &[2, 3, 4, 5, 6]);
    assert!(h.text().contains("behind »: Stop (No query is running)"));
    h.focus(narrow);
    h.key(KeyCode::End);
    assert!(page(&h).narrow.on_more(), "» is the last stop");
    assert_eq!(h.bars(), 1);
    let more = h.app.shell.hits.area_of(page(&h).narrow.more_id()).unwrap();
    assert_eq!(
        h.row(more.y).chars().nth(more.x as usize),
        Some('▎'),
        "the one ▎ on the button"
    );
    h.key(KeyCode::Enter);
    assert!(page(&h).narrow.menu_open());
    assert!(h.text().contains("No query is running"), "the reason");
    assert!(h.text().contains("○ Wrap"), "a toggle's state");
    assert_eq!(h.bars(), 0, "the toolbar under its menu shows no focus");
    // Stop is disabled: the cursor starts on Format
    h.key(KeyCode::Enter);
    assert!(!page(&h).narrow.menu_open());
    assert_eq!(page(&h).last, "Activated(3) · Format");
    assert!(page(&h).narrow.on_more(), "focus returns to »");
    // by mouse: the button opens it, a row runs its item
    let more = h.app.shell.hits.area_of(page(&h).narrow.more_id()).unwrap();
    h.click(more.x + 1, more.y);
    assert!(page(&h).narrow.menu_open());
    // Wrap is the menu's fourth row (Stop, Format, Explain, Wrap, ...)
    let row = page(&h)
        .narrow
        .menu_id()
        .part(PartRef::item(Part::ROW, ItemKey::index(3)));
    let r = h.app.shell.hits.area_of(row).expect("the Wrap row");
    h.click(r.x + 2, r.y);
    assert_eq!(page(&h).last, "Toggled(5, true) · Wrap");
    // a click outside closes it without running anything
    h.click(more.x + 1, more.y);
    assert!(page(&h).narrow.menu_open());
    let (x, y) = h.find("Formatting toolbar").unwrap();
    h.click(x, y);
    assert!(!page(&h).narrow.menu_open());
    assert_eq!(page(&h).last, "Toggled(5, true) · Wrap");
}

#[test]
fn the_overflow_option_swaps_the_glyph_or_turns_the_button_off() {
    let mut h = harness();
    h.opt("toolbars.overflow=…");
    let more = h.app.shell.hits.area_of(page(&h).narrow.more_id()).unwrap();
    assert_eq!(h.row(more.y).chars().nth(more.x as usize + 1), Some('…'));
    h.opt("toolbars.overflow=off");
    assert!(
        h.app
            .shell
            .hits
            .area_of(page(&h).narrow.more_id())
            .is_none()
    );
    assert!(page(&h).narrow.hidden().is_empty());
    assert!(h.text().contains("everything fits: no » button"));
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
        Overflow,
        Narrow,
    ] {
        assert!(rows.contains(&s), "{s:?}");
    }
    assert!(h.bars() <= 1);
}
