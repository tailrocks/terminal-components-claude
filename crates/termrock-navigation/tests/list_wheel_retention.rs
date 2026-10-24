use termrock_core::{Axis, Id, ItemKey, KeyCode, Response};
use termrock_navigation::{List, ListState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("list.wheel-retention");
const ROW_COUNT: usize = 40;

struct Page {
    items: Vec<String>,
    state: ListState,
}

impl Page {
    fn new() -> Self {
        Self {
            items: (0..ROW_COUNT)
                .map(|index| format!("Row {index:02}"))
                .collect(),
            state: ListState::default(),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        List::new(LIST)
            .key(|row: &String| ItemKey::text(row))
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        List::new(LIST).key(|row: &String| ItemKey::text(row)).draw(
            ui,
            area,
            &self.state,
            &self.items,
        );
    }
}

fn harness() -> Harness<Page> {
    Harness::new(Page::new(), Theme::junie(), 24, 6)
}

fn row_key(index: usize) -> ItemKey {
    ItemKey::text(&format!("Row {index:02}"))
}

#[test]
fn wheel_keeps_offscreen_cursor_through_redraw_then_down_reveals_it() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    assert_eq!(h.focus(), Some(LIST));
    assert_eq!(h.app().state.cursor(), Some(row_key(0)));
    assert!(h.text().contains("Row 00"));

    let _ = h.wheel(Axis::V, 1, 4, 2);
    let wheel_offset = h.app().state.scroll().offset();
    assert!(wheel_offset > 0, "wheel must move the viewport");
    assert_eq!(h.app().state.cursor(), Some(row_key(0)));
    assert!(
        !h.text().contains("Row 00"),
        "wheel must scroll the cursor row out without moving the cursor"
    );

    let _ = h.tick();
    assert_eq!(h.app().state.scroll().offset(), wheel_offset);
    h.draw();
    assert_eq!(h.app().state.scroll().offset(), wheel_offset);
    assert_eq!(h.app().state.cursor(), Some(row_key(0)));
    assert!(!h.text().contains("Row 00"));

    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(row_key(1)));
    assert!(h.app().state.scroll().visible_range().contains(&1));
    assert!(
        h.text().contains("Row 01"),
        "Down reveals and paints the new cursor row"
    );
}

#[test]
fn ordinary_down_navigation_without_scrolling_stays_visible() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    assert_eq!(h.focus(), Some(LIST));
    assert_eq!(h.app().state.cursor(), Some(row_key(0)));

    let _ = h.key(KeyCode::Down);

    assert_eq!(h.app().state.cursor(), Some(row_key(1)));
    assert_eq!(h.app().state.scroll().offset(), 0);
    assert!(h.app().state.scroll().visible_range().contains(&1));
    assert!(h.text().contains("Row 01"));
}
