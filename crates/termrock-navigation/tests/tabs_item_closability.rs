use std::fmt;

use termrock_core::{Id, ItemKey, KeyCode, MouseKind, Part, PartRef, Response};
use termrock_navigation::{Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{App, Cx, Rect, Ui};
use termrock_test_support::Harness;
use termrock_theme::{GlyphRole, Theme};

const TABS: Id = Id::root("tabs.item-closability");

#[derive(Clone, Copy)]
struct Tab {
    id: u64,
    label: &'static str,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.label(tab.label);
}

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    close_keys: Option<Vec<ItemKey>>,
    legacy_closable: bool,
    actions: Vec<TabsAction>,
}

impl Page {
    fn new(items: Vec<Tab>, close_keys: Option<Vec<ItemKey>>, legacy_closable: bool) -> Self {
        Self {
            state: TabsState::default(),
            items,
            close_keys,
            legacy_closable,
            actions: Vec::new(),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tabs = match self.close_keys.as_deref() {
            Some(keys) => Tabs::new(TABS).closable_keys(keys),
            None => Tabs::new(TABS),
        };
        let tabs = tabs
            .closable(self.legacy_closable)
            .key(tab_key)
            .row(tab_row);
        let mut response = tabs.update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let tabs = match self.close_keys.as_deref() {
            Some(keys) => Tabs::new(TABS).closable_keys(keys),
            None => Tabs::new(TABS),
        };
        let tabs = tabs
            .closable(self.legacy_closable)
            .key(tab_key)
            .row(tab_row);
        tabs.draw(ui, ui.full(), &self.state, &self.items);
    }
}

fn keyed_tabs() -> Harness<Page> {
    Harness::new(
        Page::new(
            vec![
                Tab {
                    id: 1,
                    label: "Pinned",
                },
                Tab {
                    id: 2,
                    label: "Open",
                },
                Tab {
                    id: 3,
                    label: "Other",
                },
            ],
            Some(vec![ItemKey::num(2)]),
            false,
        ),
        Theme::junie(),
        36,
        3,
    )
}

fn close_area(h: &Harness<Page>, key: ItemKey) -> Option<Rect> {
    h.runtime()
        .registry()
        .area_of_part(TABS, PartRef::item(Part::CLOSE, key))
}

fn close_glyph_count(h: &Harness<Page>) -> usize {
    let glyph = Theme::junie().design.glyphs.get(GlyphRole::Close);
    h.row(0).matches(glyph).count()
}

#[test]
fn mixed_eligibility_controls_keyboard_paint_hit_and_click_by_stable_key() {
    let mut h = keyed_tabs();
    let pinned = ItemKey::num(1);
    let open = ItemKey::num(2);

    assert_eq!(close_area(&h, pinned), None);
    assert!(close_area(&h, open).is_some());
    assert_eq!(close_glyph_count(&h), 1);

    let _ = h.key(KeyCode::Char('x'));
    let _ = h.key(KeyCode::Delete);
    assert!(h.app().actions.is_empty(), "pinned item must not close");

    let _ = h.key(KeyCode::Right);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(h.app().actions, [TabsAction::Close(open)]);

    h.app_mut().actions.clear();
    let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, open));
    assert_eq!(h.app().actions, [TabsAction::Close(open)]);
}

#[test]
fn explicit_empty_or_stale_policy_overrides_legacy_all_closable() {
    for keys in [Vec::new(), vec![ItemKey::num(99)]] {
        let mut h = Harness::new(
            Page::new(
                vec![Tab {
                    id: 1,
                    label: "Pinned",
                }],
                Some(keys),
                true,
            ),
            Theme::junie(),
            24,
            3,
        );
        assert_eq!(close_area(&h, ItemKey::num(1)), None);
        assert_eq!(close_glyph_count(&h), 0);
        let _ = h.key(KeyCode::Char('x'));
        let _ = h.key(KeyCode::Delete);
        assert!(h.app().actions.is_empty());
    }
}

#[test]
fn legacy_all_items_and_plain_modes_keep_their_existing_close_controls() {
    let mut all = Harness::new(
        Page::new(
            vec![
                Tab {
                    id: 1,
                    label: "One",
                },
                Tab {
                    id: 2,
                    label: "Two",
                },
            ],
            None,
            true,
        ),
        Theme::junie(),
        24,
        3,
    );
    assert!(close_area(&all, ItemKey::num(1)).is_some());
    assert!(close_area(&all, ItemKey::num(2)).is_some());
    assert_eq!(close_glyph_count(&all), 2);
    let _ = all.key(KeyCode::Char('x'));
    assert_eq!(all.app().actions, [TabsAction::Close(ItemKey::num(1))]);

    let plain = Harness::new(
        Page::new(
            vec![Tab {
                id: 1,
                label: "One",
            }],
            None,
            false,
        ),
        Theme::junie(),
        24,
        3,
    );
    assert_eq!(close_area(&plain, ItemKey::num(1)), None);
    assert_eq!(close_glyph_count(&plain), 0);
}

#[test]
fn reorder_keeps_close_identity_and_removed_pointer_target_is_revalidated() {
    let mut h = keyed_tabs();
    let open = ItemKey::num(2);

    h.app_mut().items.swap(1, 2);
    let _ = h.tick();
    h.app_mut().actions.clear();
    assert!(
        close_area(&h, open).is_some(),
        "reordered eligible close target was not registered: {} {:?}",
        h.row(0),
        h.diagnostics()
    );
    let _ = h.click_part(TABS, PartRef::item(Part::CLOSE, open));
    assert_eq!(h.app().actions, [TabsAction::Close(open)]);

    h.app_mut().actions.clear();
    let area = close_area(&h, open).expect("eligible close target is registered");
    let x = area.x.saturating_add(area.width / 2);
    let y = area.y;
    let _ = h.mouse(MouseKind::Down, x, y);
    h.app_mut().items.retain(|item| item.id != 2);
    let _ = h.mouse(MouseKind::Up, x, y);
    assert!(
        !h.app()
            .actions
            .iter()
            .any(|action| matches!(action, TabsAction::Close(_)))
    );
}
