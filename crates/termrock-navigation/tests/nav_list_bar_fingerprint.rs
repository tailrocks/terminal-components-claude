use ratatui_core::layout::Rect;
use termrock_core::{Id, ItemKey, Response};
use termrock_navigation::{NavList, NavListState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const ID: Id = Id::root("bar-fingerprint");
const ITEMS: &[Entry] = &[
    Entry {
        key: "first",
        label: "界e\u{301}🙂",
    },
    Entry {
        key: "second",
        label: "Second",
    },
];
#[derive(Clone, Copy)]
struct Entry {
    key: &'static str,
    label: &'static str,
}
fn key(e: &Entry) -> ItemKey {
    ItemKey::text(e.key)
}
impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}
struct Page {
    state: NavListState,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        NavList::new(ID)
            .key(key as fn(&Entry) -> ItemKey)
            .update(cx, &mut self.state, ITEMS)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        NavList::new(ID).key(key as fn(&Entry) -> ItemKey).draw(
            ui,
            Rect::new(2, 1, 16, 2),
            &self.state,
            ITEMS,
        );
    }
}
#[test]
fn pinned_pre_fill_bar_frame_and_regions_remain_unchanged() {
    let mut h = Harness::new(
        Page {
            state: NavListState::new(),
        },
        Theme::junie(),
        24,
        4,
    );
    assert!(h.tab_to(ID));
    assert_eq!(h.snapshot().digest(), 4_491_769_324_398_299_356);
    assert_eq!(h.area_of(ID), Some(Rect::new(2, 1, 16, 2)));
    for (index, item) in ITEMS.iter().enumerate() {
        assert_eq!(
            h.area_of_part(
                ID,
                termrock_core::PartRef::item(termrock_core::Part::ROW, key(item))
            ),
            Some(Rect::new(2, 1 + index as u16, 16, 1))
        );
    }
}
