//! Public-consumer proof of compact layout and replacement renderer ownership.
use junie_tui::{
    ColorLevel, Id, ItemKey, NavList, NavListState, Rect, StateFlags, Style, Theme, Ui,
};
use junie_tui_testing::Scene;
use std::cell::RefCell;

const NAV: Id = Id::root("nav.compact.contract");
const ITEMS: [(&str, &str); 3] = [("one", "A"), ("two", "B"), ("three", "B")];
fn key(item: &(&str, &str)) -> ItemKey {
    ItemKey::text(item.0)
}
fn section<'a>(item: &'a (&str, &str)) -> &'a str {
    item.1
}

#[test]
fn auto_compact_threshold_comes_from_sectioned_content() {
    // Expanded: A, one, gap, B, two, three = six rows.
    for (height, expected) in [(5, vec![1, 2, 3]), (6, vec![2, 5, 6])] {
        let calls = RefCell::new(Vec::new());
        let painter = |_ui: &mut Ui<'_>, area: Rect, _flags, item_key, _item: &(&str, &str)| {
            calls.borrow_mut().push((area.y, item_key));
        };
        let mut scene = Scene::new("compact", Theme::junie(), ColorLevel::TrueColor, 20, 8);
        scene.draw(|ui, _| {
            NavList::new(NAV)
                .key(key)
                .row(|item, row| row.label(item.0))
                .section(&section)
                .compact_when_clipped()
                .render_row(&painter)
                .draw(
                    ui,
                    Rect::new(2, 1, 12, height),
                    &NavListState::new(),
                    &ITEMS,
                );
        });
        assert_eq!(
            calls.borrow().iter().map(|(y, _)| *y).collect::<Vec<_>>(),
            expected
        );
        assert_eq!(
            calls
                .borrow()
                .iter()
                .map(|(_, key)| *key)
                .collect::<Vec<_>>(),
            ITEMS.iter().map(key).collect::<Vec<_>>()
        );
    }
}

#[test]
fn full_row_override_is_clipped_and_receives_current_key_flags() {
    let calls = RefCell::new(Vec::new());
    let painter = |ui: &mut Ui<'_>, area: Rect, flags, item_key, _item: &(&str, &str)| {
        calls.borrow_mut().push((flags, item_key));
        // Deliberately try to escape all four boundaries. Only the row may change.
        ui.fill(Rect::new(0, 0, 20, 8), Style::new());
        ui.paint_str(
            Rect::new(0, area.y, 20, 1),
            "XXXXXXXXXXXXXXXXXXXX",
            Style::new(),
        );
    };
    let mut state = NavListState::new();
    state.set_current(Some(ItemKey::text("two")));
    let mut scene = Scene::new(
        "compact_override",
        Theme::junie(),
        ColorLevel::TrueColor,
        20,
        8,
    );
    scene.draw(|ui, _| {
        NavList::new(NAV)
            .key(key)
            .row(|item, row| row.label(item.0))
            .section(&section)
            .compact()
            .render_row(&painter)
            .draw(ui, Rect::new(2, 1, 12, 2), &state, &ITEMS);
    });
    assert_eq!(
        calls.borrow().len(),
        2,
        "clipped item must not invoke renderer"
    );
    for (flags, key) in calls.borrow().iter() {
        assert_eq!(
            flags.contains(StateFlags::SELECTED),
            *key == ItemKey::text("two")
        );
    }
    for y in 0..8 {
        for x in 0..20 {
            let expected = if (1..3).contains(&y) && (2..14).contains(&x) {
                "X"
            } else {
                " "
            };
            assert_eq!(
                scene.buffer().cell((x, y)).map(junie_tui::Cell::symbol),
                Some(expected)
            );
        }
    }
}
