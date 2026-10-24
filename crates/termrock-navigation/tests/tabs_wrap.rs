#![allow(clippy::unwrap_used, reason = "real Tabs wrap contract assertions")]

use core::cell::Cell;
use core::fmt;

use ratatui_core::layout::Rect;
use termrock_core::action::ActionKey;
use termrock_core::{Axis, Id, ItemKey, KeyCode, Part, PartRef, Response};
use termrock_navigation::{Tabs, TabsAction, TabsState};
use termrock_runtime::keymap::{BindingState, Bindings, KeyMap};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TABS: Id = Id::root("tabs.wrap");

type KeyFn = fn(&Tab) -> ItemKey;
type RowFn = for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>);
type TabsWidget = Tabs<'static, Tab, KeyFn, RowFn>;

#[derive(Clone, Debug)]
struct Tab {
    id: u64,
    label: String,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_row(tab: &Tab, row: &mut termrock_runtime::RowUi<'_>) {
    row.label(&tab.label);
}

fn apply_wrap<T, K, R>(tabs: Tabs<'static, T, K, R>, wrap: Option<bool>) -> Tabs<'static, T, K, R> {
    match wrap {
        Some(yes) => tabs.wrap(yes),
        None => tabs,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuilderOrder {
    BeforeKey,
    BetweenKeyAndRow,
    AfterRow,
}

struct Page {
    wrap: Option<bool>,
    planned_wrap: Option<bool>,
    builder_order: BuilderOrder,
    digit_jump: bool,
    plan: bool,
    plan_rejected: Cell<bool>,
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    keymap: KeyMap,
}

impl Page {
    fn new(count: usize, active_index: Option<usize>, wrap: Option<bool>) -> Self {
        let items: Vec<_> = (1..=count)
            .map(|id| Tab {
                id: id as u64,
                label: format!("Tab{id}"),
            })
            .collect();
        let mut state = TabsState::default();
        if let Some(index) = active_index
            && let Some(tab) = items.get(index)
        {
            state.set_active(index, ItemKey::num(tab.id));
        }
        Self {
            wrap,
            planned_wrap: None,
            builder_order: BuilderOrder::BeforeKey,
            digit_jump: true,
            plan: false,
            plan_rejected: Cell::new(false),
            state,
            items,
            actions: Vec::new(),
            keymap: KeyMap::new(),
        }
    }

    fn widget_with_wrap(&self, wrap: Option<bool>) -> TabsWidget {
        let base = Tabs::new(TABS)
            .allow_new(true)
            .closable(true)
            .digit_jump(self.digit_jump);
        match self.builder_order {
            BuilderOrder::BeforeKey => apply_wrap(base, wrap)
                .key(tab_key as KeyFn)
                .row(tab_row as RowFn),
            BuilderOrder::BetweenKeyAndRow => {
                apply_wrap(base.key(tab_key as KeyFn), wrap).row(tab_row as RowFn)
            }
            BuilderOrder::AfterRow => {
                apply_wrap(base.key(tab_key as KeyFn).row(tab_row as RowFn), wrap)
            }
        }
    }

    fn widget(&self) -> TabsWidget {
        self.widget_with_wrap(self.wrap)
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tabs = self.widget();
        let mut response = tabs.update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        let area = Rect {
            height: full.height.min(2),
            ..full
        };
        let tabs = self.widget();
        if self.plan {
            let plan_wrap = self.planned_wrap.or(self.wrap);
            let planned = apply_wrap(
                Tabs::new(TABS)
                    .allow_new(true)
                    .closable(true)
                    .digit_jump(self.digit_jump)
                    .key(tab_key as KeyFn),
                plan_wrap,
            );
            let layout = planned.layout_default(area, area, &self.state, &self.items, None);
            let current = apply_wrap(
                Tabs::new(TABS)
                    .allow_new(true)
                    .closable(true)
                    .digit_jump(self.digit_jump)
                    .key(tab_key as KeyFn),
                self.wrap,
            );
            self.plan_rejected
                .set(current.draw_layout(ui, &layout).is_err());
        } else {
            tabs.draw(ui, area, &self.state, &self.items);
        }
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: Page, width: u16) -> Harness<Page> {
    Harness::new(page, Theme::junie(), width, 4)
}

fn settle(h: &mut Harness<Page>) {
    let _ = h.tick();
    h.app_mut().actions.clear();
}

fn assert_move(h: &mut Harness<Page>, key: KeyCode, target: u64) {
    let response = h.key(key);
    assert!(
        response.is_consumed(),
        "{key:?} should remain a Tabs command"
    );
    assert!(
        response.is_changed(),
        "{key:?} should activate the wrapped tab"
    );
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::num(target))]
    );
    assert_eq!(h.app().state.active(), Some(ItemKey::num(target)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(target)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn previous_next_and_vim_aliases_wrap_only_at_the_list_ends() {
    for (start, key, target) in [
        (0, KeyCode::Left, 9),
        (8, KeyCode::Right, 1),
        (0, KeyCode::Char('h'), 9),
        (8, KeyCode::Char('l'), 1),
    ] {
        let mut page = Page::new(9, Some(start), Some(true));
        page.digit_jump = false;
        let mut h = harness(page, 72);
        settle(&mut h);
        assert_move(&mut h, key, target);
    }

    let mut interior = harness(Page::new(5, Some(2), Some(true)), 64);
    settle(&mut interior);
    assert_move(&mut interior, KeyCode::Left, 2);
    interior.app_mut().actions.clear();
    assert_move(&mut interior, KeyCode::Right, 3);
}

#[test]
fn consumer_remaps_use_the_same_wrapping_prev_next_actions() {
    let mut page = Page::new(6, Some(0), Some(true));
    page.keymap = KeyMap::new()
        .bind_component(
            TABS,
            ActionKey::custom("tabs.previous"),
            termrock_core::Chord::key(KeyCode::F(10)),
        )
        .bind_component(
            TABS,
            ActionKey::custom("tabs.next"),
            termrock_core::Chord::key(KeyCode::F(11)),
        );
    let mut h = harness(page, 64);
    settle(&mut h);
    assert_move(&mut h, KeyCode::F(10), 6);
    h.app_mut().actions.clear();
    assert_move(&mut h, KeyCode::F(11), 1);
}

#[test]
fn absent_and_explicit_false_keep_endpoint_responses_and_frames_exact() {
    for (start, key) in [(0, KeyCode::Left), (4, KeyCode::Right)] {
        let mut absent = harness(Page::new(5, Some(start), None), 48);
        let mut explicit = harness(Page::new(5, Some(start), Some(false)), 48);
        settle(&mut absent);
        settle(&mut explicit);
        assert_eq!(absent.buffer(), explicit.buffer());
        assert_eq!(absent.ring().entries(), explicit.ring().entries());
        assert_eq!(absent.focus(), explicit.focus());
        assert_eq!(absent.app().state, explicit.app().state);
        assert_eq!(absent.area_of(TABS), explicit.area_of(TABS));
        for part in [
            PartRef::item(Part::TAB, ItemKey::num(1)),
            PartRef::item(Part::TAB, ItemKey::num(5)),
            PartRef::item(Part::CLOSE, ItemKey::num(1)),
            PartRef::of(Part::OVERFLOW),
        ] {
            assert_eq!(
                absent.area_of_part(TABS, part),
                explicit.area_of_part(TABS, part)
            );
        }

        let absent_response = absent.key(key);
        let explicit_response = explicit.key(key);
        assert_eq!(
            absent_response.is_consumed(),
            explicit_response.is_consumed()
        );
        assert_eq!(absent_response.is_changed(), explicit_response.is_changed());
        assert_eq!(absent.app().state, explicit.app().state);
        assert_eq!(absent.app().actions, explicit.app().actions);
        assert_eq!(absent.buffer(), explicit.buffer());
        assert_eq!(absent.ring().entries(), explicit.ring().entries());
        assert!(
            absent.diagnostics().is_empty(),
            "{:?}",
            absent.diagnostics()
        );
        assert!(
            explicit.diagnostics().is_empty(),
            "{:?}",
            explicit.diagnostics()
        );
    }
}

#[test]
fn empty_and_single_item_sources_are_bounded() {
    let mut empty = harness(Page::new(0, None, Some(true)), 48);
    settle(&mut empty);
    let empty_state = empty.app().state.clone();
    for key in [KeyCode::Left, KeyCode::Right] {
        let response = empty.key(key);
        assert!(response.is_consumed());
        assert!(!response.is_changed());
        assert!(empty.app().actions.is_empty());
        assert_eq!(empty.app().state, empty_state);
    }
    assert!(empty.diagnostics().is_empty(), "{:?}", empty.diagnostics());

    let mut one = harness(Page::new(1, Some(0), Some(true)), 48);
    settle(&mut one);
    for key in [KeyCode::Left, KeyCode::Right] {
        let response = one.key(key);
        assert!(response.is_consumed());
        assert!(response.is_changed());
        assert_eq!(
            one.app().actions.last(),
            Some(&TabsAction::Activated(ItemKey::num(1)))
        );
        assert_eq!(one.app().state.active(), Some(ItemKey::num(1)));
        one.app_mut().actions.clear();
    }
    assert!(one.diagnostics().is_empty(), "{:?}", one.diagnostics());
}

#[test]
fn wrapping_reconciles_reordered_and_removed_endpoint_keys_before_moving() {
    let mut h = harness(Page::new(5, Some(0), Some(true)), 64);
    settle(&mut h);
    h.app_mut().items.swap(0, 4);
    let _ = h.tick();
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
    h.app_mut().actions.clear();
    assert_move(&mut h, KeyCode::Right, 5);

    h.app_mut().items.remove(4);
    let _ = h.tick();
    assert_eq!(h.app().state.active(), Some(ItemKey::num(5)));
    h.app_mut().actions.clear();
    assert_move(&mut h, KeyCode::Left, 4);
}

#[test]
fn wrapped_activation_follows_hidden_tabs_across_resize_in_both_directions() {
    // Keep the strip narrow enough to overflow, but wide enough for a tab
    // after the left/right overflow, New, and close lanes are reserved.
    let mut h = harness(Page::new(8, Some(0), Some(true)), 24);
    settle(&mut h);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(8)))
            .is_none(),
        "last tab starts outside the narrow strip"
    );
    assert_move(&mut h, KeyCode::Left, 8);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(8)))
            .is_some(),
        "wrapped activation follows the last tab into view"
    );

    let _ = h.resize(36, 4);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(8)));
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(8)))
            .is_some()
    );
    h.app_mut().actions.clear();
    assert_move(&mut h, KeyCode::Right, 1);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(1)))
            .is_some(),
        "wrapped activation follows the first tab back into view"
    );

    let _ = h.resize(24, 4);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(1)))
            .is_some()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn overflow_wheel_and_pointer_navigation_do_not_apply_endpoint_wrapping() {
    let mut h = harness(Page::new(8, Some(1), Some(true)), 20);
    settle(&mut h);
    let active = h.app().state.active();
    let cursor = h.app().state.cursor();
    let right = PartRef::item(Part::OVERFLOW, ItemKey::index(1));
    let area = h
        .area_of_part(TABS, right)
        .expect("the overflow-right target is visible");
    let before_first = h.app().state.first();
    let response = h.click_part(TABS, right);
    assert!(response.is_changed());
    assert_eq!(h.app().state.active(), active);
    assert_eq!(h.app().state.cursor(), cursor);
    assert_ne!(h.app().state.first(), before_first);
    assert!(h.app().actions.is_empty());

    let _ = h.wheel(Axis::H, 1, area.x, area.y);
    assert_eq!(h.app().state.active(), active);
    assert_eq!(h.app().state.cursor(), cursor);
    assert!(h.app().actions.is_empty());

    let mut pointer = harness(Page::new(8, Some(7), Some(true)), 64);
    settle(&mut pointer);
    let tab = pointer
        .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(7)))
        .expect("the clicked tab is visible beside the active endpoint");
    let _ = pointer.click(tab.x, tab.y);
    assert_eq!(
        pointer.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::num(7))]
    );
    assert_eq!(pointer.app().state.active(), Some(ItemKey::num(7)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    assert!(
        pointer.diagnostics().is_empty(),
        "{:?}",
        pointer.diagnostics()
    );
}

#[test]
fn nth_activate_new_and_close_keep_their_existing_semantics() {
    let mut h = harness(Page::new(9, Some(8), Some(true)), 72);
    settle(&mut h);

    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::num(9))]
    );
    h.app_mut().actions.clear();

    assert_move(&mut h, KeyCode::Char('2'), 2);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::num(2))]
    );
    assert_eq!(h.app().state.active(), Some(ItemKey::num(2)));
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('n'));
    assert_eq!(h.app().actions.as_slice(), [TabsAction::New]);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(2)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn wrap_survives_key_row_and_plan_adapters_without_changing_manual_tables() {
    let mut reference_tables = None;
    for order in [
        BuilderOrder::BeforeKey,
        BuilderOrder::BetweenKeyAndRow,
        BuilderOrder::AfterRow,
    ] {
        let mut page = Page::new(5, Some(0), Some(true));
        page.builder_order = order;
        let tabs = page.widget();
        assert!(format!("{tabs:?}").contains("wrap: true"));
        let current_tables = (
            tabs.bindings(BindingState::default()).to_vec(),
            tabs.new_key_table().to_vec(),
            tabs.close_key_table().to_vec(),
        );
        if let Some(expected) = &reference_tables {
            assert_eq!(&current_tables, expected, "{order:?}");
        } else {
            reference_tables = Some(current_tables);
        }

        let mut h = harness(page, 64);
        settle(&mut h);
        assert_move(&mut h, KeyCode::Left, 5);
    }

    let legacy = harness(Page::new(5, Some(0), Some(true)), 48);
    let mut planned_page = Page::new(5, Some(0), Some(true));
    planned_page.plan = true;
    planned_page.planned_wrap = Some(false);
    let mut planned = harness(planned_page, 48);
    assert!(!planned.app().plan_rejected.get());
    assert_eq!(legacy.buffer(), planned.buffer());
    assert_eq!(legacy.ring().entries(), planned.ring().entries());
    for part in [
        PartRef::item(Part::TAB, ItemKey::num(1)),
        PartRef::item(Part::TAB, ItemKey::num(5)),
        PartRef::item(Part::CLOSE, ItemKey::num(1)),
        PartRef::item(Part::OVERFLOW, ItemKey::index(1)),
    ] {
        assert_eq!(
            legacy.area_of_part(TABS, part),
            planned.area_of_part(TABS, part)
        );
    }
    settle(&mut planned);
    assert_move(&mut planned, KeyCode::Left, 5);
}
