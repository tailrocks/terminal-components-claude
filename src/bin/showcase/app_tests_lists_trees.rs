



use junie_tui::runtime::Application;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::app::PageId;
use crate::app_tests_shell_pages::Harness;
use crate::pages::lists::{ListsPage, MULTI, SINGLE};
use crate::pages::template::Section;
use crate::pages::trees::{CONNECTIONS, TreesPage};
use junie_tui::core::event::Input;
use junie_tui::ui::specimen::SpecimenState;

fn lists(h: &Harness) -> &ListsPage {
    h.app.pages[PageId::Lists.index()]
        .as_any()
        .and_then(|p| p.downcast_ref::<ListsPage>())
        .expect("lists page")
}

fn hint_keys(h: &Harness) -> String {
    h.app.pages[h.app.page.index()]
        .hints(h.app.shell.focus.current())
        .iter()
        .map(|(k, _)| *k)
        .collect::<Vec<_>>()
        .join(" | ")
}

fn single_cursor(h: &Harness) -> usize {
    lists(h).single().cursor
}



#[test]
fn list_hints_name_ends_keys_that_work_under_both_profiles() {
    let mut h = Harness::new(120, 40, PageId::Lists);
    h.app.shell.focus.focus(SINGLE);
    h.draw();
    let keys = hint_keys(&h);
    assert!(keys.contains("Home End"), "{keys}");
    assert!(!keys.contains("g G"), "{keys}");
    h.key(KeyCode::End);
    assert!(single_cursor(&h) > 10, "End jumps to the last row");
    h.key(KeyCode::Home);
    assert_eq!(single_cursor(&h), 0, "Home jumps to the first row");

    h.opt("lists.nav=vim");
    h.opt("lists.type_ahead=off");
    h.app.shell.focus.focus(SINGLE);
    h.draw();
    let keys = hint_keys(&h);
    assert!(keys.contains("g G"), "{keys}");
    h.key_mods(KeyCode::Char('G'), KeyModifiers::SHIFT);
    assert!(single_cursor(&h) > 10, "G jumps to the last row");
    h.key(KeyCode::Char('g'));
    assert_eq!(single_cursor(&h), 0, "g jumps to the first row");

    h.opt("lists.type_ahead=on");
    let keys = hint_keys(&h);
    assert!(keys.contains("Home End") && !keys.contains("g G"), "{keys}");
}

#[test]
fn list_options_rebuild_the_lists_through_their_builders() {
    let mut h = Harness::new(160, 50, PageId::Lists);


    assert!(
        h.text().contains("click Double, check BoxOnly"),
        "{}",
        h.text()
    );
    let (x, y) = h.find("Python").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Chosen: Rust"), "a click does not choose");
    h.click(x, y);
    assert!(h.text().contains("Chosen: Python"), "a double click does");

    h.opt("lists.click=single");
    assert!(h.text().contains("click Single"));
    let (x, y) = h.find("Go").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Chosen: Go"), "{}", h.text());
    h.opt("lists.check=row");
    assert!(h.text().contains("check Row"));
    assert!(h.text().contains("toggle platform+click"));
    h.opt("lists.toggle_mods=any");
    assert!(h.text().contains("toggle Ctrl/Alt/Cmd+click"));

    h.opt("lists.keymap=custom");
    assert!(h.text().contains("t ListCmd::ToggleAll"));
    h.app.shell.focus.focus(MULTI);
    h.key(KeyCode::Char('t'));
    assert!(h.text().contains("10 selected"), "{}", h.text());

    assert!(!h.text().contains("Try a shorter word"));
    h.opt("lists.empty=state");
    assert!(h.text().contains("Try a shorter word"));
}

#[test]
fn list_states_rows_are_drawn() {
    let mut h = Harness::new(160, 60, PageId::Lists);
    h.section(Section::States);
    let rows = h.states();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Selected,
        SpecimenState::Disabled,
        SpecimenState::Empty,
    ] {
        assert!(rows.contains(&s), "{s:?} in {rows:?}");
    }
    assert!(h.live_bars() <= 1, "samples depict focus");
}

fn trees(h: &Harness) -> &TreesPage {
    h.app.pages[PageId::Trees.index()]
        .as_any()
        .and_then(|p| p.downcast_ref::<TreesPage>())
        .expect("trees page")
}



#[test]
fn a_lazy_node_loads_its_children() {
    let mut h = Harness::new(160, 50, PageId::Trees);
    let (x, y) = h.find("orders-db").unwrap();

    h.click(x, y);
    assert!(!h.text().contains("Expand(orders-db)"), "{}", h.text());
    h.click(x, y);
    assert!(h.app.shell.focus.is(CONNECTIONS));
    assert!(h.text().contains("Expand(orders-db)"), "{}", h.text());
    assert!(h.text().contains("loading…"));
    for _ in 0..6 {
        h.app.handle(Input::Tick);
    }
    h.draw();
    assert!(h.text().contains("customers"), "{}", h.text());
    assert!(!h.text().contains("loading…"));
    assert!(trees(&h).conns().node(&[0, 0]).is_some_and(|n| !n.lazy));
}

#[test]
fn tree_options_rebuild_the_trees_through_their_builders() {
    let mut h = Harness::new(160, 50, PageId::Trees);
    h.opt("trees.branch=activate");
    assert!(h.text().contains("branch_activate Activate"));
    assert!(h.text().contains("click Double"), "{}", h.text());
    h.opt("trees.click=single");
    assert!(h.text().contains("click Single"));
    h.opt("trees.keymap=custom");
    assert!(h.text().contains("e TreeCmd::ExpandAll"));
    h.app.shell.focus.focus(crate::pages::trees::PROJECT);
    h.draw();
    let keys = hint_keys(&h);
    assert!(keys.contains('e') && !keys.contains('*'), "{keys}");
    h.opt("trees.indent=4");
    assert!(h.text().contains("indent 4"));
    h.opt("trees.nav=vim");
    h.opt("trees.type_ahead=off");
    let keys = hint_keys(&h);
    assert!(keys.contains("g G"), "{keys}");
}

#[test]
fn tree_states_rows_are_drawn() {
    let mut h = Harness::new(160, 60, PageId::Trees);
    h.section(Section::States);
    let rows = h.states();
    for s in [
        SpecimenState::Rest,
        SpecimenState::Hover,
        SpecimenState::Selected,
        SpecimenState::Busy,
        SpecimenState::Overflow,
    ] {
        assert!(rows.contains(&s), "{s:?} in {rows:?}");
    }
}
