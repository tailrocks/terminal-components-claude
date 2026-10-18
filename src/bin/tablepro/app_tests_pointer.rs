


use junie_tui::core::event::MouseKind;
use junie_tui::core::id::{ItemKey, Part, PartRef, WidgetId};
use ratatui::crossterm::event::KeyCode;

use crate::app_tests::H;
use crate::filter_editor::FilterEditor;

fn open_orders(h: &mut H) {
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    h.ticks(3);
}


fn footer(h: &H) -> String {
    h.text().lines().last().unwrap_or_default().to_owned()
}



#[test]
fn a_middle_click_on_a_query_tab_closes_it() {
    let mut h = H::connected(120, 40);
    h.ctrl('t');
    h.ctrl('t');
    let strip = h.wb().strip.id;
    let before = h.wb().tabs.len();
    let last = before - 1;
    let tab = strip.part(PartRef::item(Part::TAB, ItemKey::index(last)));
    let r = h
        .app
        .shell
        .hits
        .area_of(tab)
        .expect("the last tab is drawn");
    h.mouse(MouseKind::Middle, r.x + 1, r.y);
    assert_eq!(h.wb().tabs.len(), before - 1, "{}", h.text());
}


#[test]
fn a_click_on_a_footer_hint_runs_its_key() {
    let mut h = H::connected(120, 40);
    let line = footer(&h);
    let at = line.find("Tab Next").expect("the footer names Tab") as u16;
    let x = line[..at as usize].chars().count() as u16;
    let y = h.text().lines().count() as u16 - 1;
    let before = h.focus();
    h.click(x, y);
    assert_ne!(
        h.focus(),
        before,
        "the click on Tab moves focus\n{}",
        h.text()
    );
}



#[test]
fn the_wheel_over_the_filter_editors_open_select_scrolls_its_list() {
    let mut h = H::connected(120, 40);
    open_orders(&mut h);
    h.key(KeyCode::Char('f'));
    let column = WidgetId::of("filter-editor").sub("col");
    h.tab_to(column);
    h.key(KeyCode::Enter);
    let sel = &h
        .app
        .top_as::<FilterEditor>()
        .expect("the filter editor")
        .column;
    assert!(sel.open, "the list is open");
    assert!(
        sel.options.len() > sel.list.max_rows as usize,
        "orders has more columns than the list shows"
    );

    let row = h
        .app
        .shell
        .hits
        .regions()
        .iter()
        .rev()
        .find(|r| r.owner == column && r.part.part == Part::ROW)
        .map(|r| r.area)
        .expect("the open list's rows are drawn");
    h.mouse(MouseKind::WheelDown, row.x + 1, row.y);
    let sel = &h.app.top_as::<FilterEditor>().unwrap().column;
    assert!(
        sel.list.scroll.offset > 0,
        "the list scrolled\n{}",
        h.text()
    );
}



#[test]
fn a_press_on_the_switcher_scrollbar_pages_its_list() {
    let mut h = H::connected(120, 40);
    h.ctrl('p');
    let switcher = WidgetId::of("switcher");
    let bar = h
        .app
        .shell
        .hits
        .regions()
        .iter()
        .find(|r| r.owner == switcher && r.part.part == Part::TRACK)
        .map(|r| r.area)
        .expect("the switcher shows a scrollbar");
    let before = scroll_of(&h);
    h.mouse(MouseKind::Down, bar.x, bar.bottom() - 1);
    h.mouse(MouseKind::Up, bar.x, bar.bottom() - 1);
    assert!(
        scroll_of(&h) > before,
        "the press paged the list\n{}",
        h.text()
    );
}

fn scroll_of(h: &H) -> usize {
    let p = h
        .app
        .top_as::<junie_tui::widgets::picker::Picker>()
        .expect("the switcher is open");
    p.list.scroll.offset
}
