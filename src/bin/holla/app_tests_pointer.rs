


use junie_tui::core::event::MouseKind;
use junie_tui::core::id::{Part, WidgetId};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};


fn part_area(h: &H, owner: WidgetId, part: Part) -> Option<Rect> {
    h.app
        .shell
        .hits
        .regions()
        .iter()
        .rev()
        .find(|r| r.owner == owner && r.part.part == part)
        .map(|r| r.area)
}


#[test]
fn a_click_on_a_footer_hint_runs_its_key() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    let line = h.last_row();
    let at = line
        .find("Tab Preview")
        .unwrap_or_else(|| panic!("the footer names Tab\n{}", h.text()));
    let x = line[..at].chars().count() as u16;
    let y = h.text().lines().count() as u16 - 1;
    let before = h.app.shell.focus.current();
    h.click(x, y);
    assert_ne!(
        h.app.shell.focus.current(),
        before,
        "the click on Tab moved focus to the preview\n{}",
        h.text()
    );
    assert_eq!(
        h.app.shell.focus.current(),
        Some(crate::screens::finder::PREVIEW)
    );
}



#[test]
fn a_press_on_the_jump_picker_scrollbar_pages_its_list() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 100, 20);
    let path = h.app.world.location.cwd.clone();
    h.app.go(Go::Push(Page::Files { path, query: None }));
    h.ticks(8);
    h.key(KeyCode::Char('g'));

    h.type_str("~/work/site/");
    let picker = h
        .app
        .modals
        .top_as::<junie_tui::widgets::picker::Picker>()
        .map(|p| p.id)
        .unwrap_or_else(|| panic!("the jump picker is open\n{}", h.text()));
    let bar = part_area(&h, picker, Part::TRACK)
        .unwrap_or_else(|| panic!("the picker shows a scrollbar\n{}", h.text()));
    let before = scroll_of(&h);
    h.mouse(MouseKind::Down, bar.x, bar.bottom() - 1);
    assert!(
        scroll_of(&h) > before,
        "the press paged the list\n{}",
        h.text()
    );
    h.mouse(MouseKind::Up, bar.x, bar.bottom() - 1);
}

fn scroll_of(h: &H) -> usize {
    h.app
        .modals
        .top_as::<junie_tui::widgets::picker::Picker>()
        .expect("the picker is open")
        .scroll
        .offset
}



fn choice_args() -> (H, WidgetId) {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    let (item, i) = h
        .app
        .world
        .items()
        .into_iter()
        .find_map(|it| {
            let i = it.args.iter().position(|a| !a.options.is_empty())?;
            Some((it.id, i))
        })
        .expect("an item with a choice argument");
    h.app.go(Go::Push(Page::Args { item }));
    h.draw();
    (h, crate::screens::review::ARG.child(i))
}





#[test]
fn a_press_dragged_onto_a_choice_option_chooses_it_on_release() {
    let (mut h, field) = choice_args();

    let (x, y) = h.find("▾").expect("the choice field is drawn");
    let under = h
        .app
        .shell
        .hits
        .hit((x, y).into())
        .expect("the mark is a target");
    assert_eq!(
        h.app.shell.hits.part_of(under).map(|p| p.owner),
        Some(field)
    );
    h.mouse(MouseKind::Down, x, y);
    let option = h
        .app
        .shell
        .hits
        .regions()
        .iter()
        .rev()
        .find(|r| {
            r.owner == field
                && r.part.part == Part::ROW
                && r.part.item.and_then(|k| k.as_index()) == Some(1)
        })
        .map(|r| r.area)
        .unwrap_or_else(|| panic!("the press opened the list\n{}", h.text()));
    h.mouse(MouseKind::Drag, option.x + 1, option.y);
    h.mouse(MouseKind::Up, option.x + 1, option.y);
    assert!(
        part_area(&h, field, Part::ROW).is_none(),
        "the release closed the list\n{}",
        h.text()
    );
    let line = h.row(y);
    assert!(
        line.contains("ci") || line.contains("INT"),
        "the release chose the second option\n{}",
        h.text()
    );

    h.click(x, y);
    assert!(
        part_area(&h, field, Part::ROW).is_some(),
        "a click leaves the list open\n{}",
        h.text()
    );
}




#[test]
fn a_click_on_dead_space_closes_the_open_menu_bar() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::F(10));
    assert!(h.app.menu_open());

    let y = h.text().lines().count() as u16 - 3;
    h.click(100, y);
    assert!(!h.app.menu_open(), "the menu bar closed\n{}", h.text());
}
