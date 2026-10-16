//! Pointer journeys through the shared router (plan S6): every gesture
//! reaches the widget that owns the part under the pointer.

use junie_tui::core::event::MouseKind;
use junie_tui::core::id::{Part, WidgetId};
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Rect;

use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};

/// The area of the latest region of `owner`'s `part`.
fn part_area(h: &H, owner: WidgetId, part: Part) -> Option<Rect> {
    h.app
        .hits
        .regions()
        .iter()
        .rev()
        .find(|r| r.owner == owner && r.part.part == part)
        .map(|r| r.area)
}

/// M1: a footer hint naming one key is a click target that runs the key.
#[test]
fn a_click_on_a_footer_hint_runs_its_key() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    let line = h.last_row();
    let at = line
        .find("Tab Preview")
        .unwrap_or_else(|| panic!("the footer names Tab\n{}", h.text()));
    let x = line[..at].chars().count() as u16;
    let y = h.text().lines().count() as u16 - 1;
    let before = h.app.focus.current();
    h.click(x, y);
    assert_ne!(
        h.app.focus.current(),
        before,
        "the click on Tab moved focus to the preview\n{}",
        h.text()
    );
    assert_eq!(h.app.focus.current(), Some(crate::screens::finder::PREVIEW));
}

/// M17 (routing): a press on the files page's jump picker's scrollbar
/// pages its list; the app delivered no press to the picker's bar.
#[test]
fn a_press_on_the_jump_picker_scrollbar_pages_its_list() {
    let mut h = H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 100, 20);
    let path = h.app.world.location.cwd.clone();
    h.app.go(Go::Push(Page::Files { path, query: None }));
    h.ticks(8);
    h.key(KeyCode::Char('g'));
    // every entry of the folder is a suggestion: more than the list shows
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

/// The arguments page of the first fixture item with a choice argument,
/// and that choice field's id.
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

/// The Release drift (plan §3): a press on a choice field opens its list
/// and a release on an option chooses it (press-drag-release, as macOS
/// pop-up buttons do); a release in place leaves the list open. The page
/// got no press, drag or release before, so the drag chose nothing.
#[test]
fn a_press_dragged_onto_a_choice_option_chooses_it_on_release() {
    let (mut h, field) = choice_args();
    // the field's value row, by its disclosure mark
    let (x, y) = h.find("▾").expect("the choice field is drawn");
    let under = h.app.hits.hit((x, y).into()).expect("the mark is a target");
    assert_eq!(h.app.hits.part_of(under).map(|p| p.owner), Some(field));
    h.mouse(MouseKind::Down, x, y);
    let option = h
        .app
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
    // a release in place keeps the list open
    h.click(x, y);
    assert!(
        part_area(&h, field, Part::ROW).is_some(),
        "a click leaves the list open\n{}",
        h.text()
    );
}

/// Plan §3 'Dead-space release with the menu bar open': a click on
/// anything but the open menu closes it, dead space included (the menu
/// bar's own light dismiss, through its backdrop).
#[test]
fn a_click_on_dead_space_closes_the_open_menu_bar() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40);
    h.key(KeyCode::F(10));
    assert!(h.app.menu_open());
    // the blank row between the body and the status bar
    let y = h.text().lines().count() as u16 - 3;
    h.click(100, y);
    assert!(!h.app.menu_open(), "the menu bar closed\n{}", h.text());
}
