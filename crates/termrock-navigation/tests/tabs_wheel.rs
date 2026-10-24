use core::cell::{Cell, RefCell};
use core::fmt;

use ratatui_core::layout::{Position, Rect};
use termrock_controls::Button;
use termrock_core::{Axis, Id, ItemKey, Part, PartRef, Response};
use termrock_navigation::tabs::TabWindowReceipt;
use termrock_navigation::{TabMarks, Tabs, TabsAction, TabsState};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::{App, Cx, LayerSpec, ScrollRegion, ScrollState, Status, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TABS: Id = Id::root("tabs.planned-wheel");
const PARENT: Id = Id::root("tabs.planned-wheel.parent");
const OVERLAY: Id = Id::root("tabs.planned-wheel.modal");
const WIDTH: u16 = 48;
const HEIGHT: u16 = 4;
const PARENT_CONTENT: usize = 64;

type KeyFn = fn(&Tab) -> ItemKey;
type MarkFn = fn(&Tab) -> TabMarks;
type TabsWidget = Tabs<'static, Tab, KeyFn>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DrawKind {
    Legacy,
    Planned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifecycleObservation {
    Update {
        before: Option<ItemKey>,
        after: Option<ItemKey>,
    },
    Draw {
        kind: DrawKind,
        first: Option<ItemKey>,
    },
}

#[derive(Clone, Debug)]
struct Tab {
    id: u64,
    label: String,
    mark: TabMarks,
    displays: Cell<usize>,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.displays.set(self.displays.get().saturating_add(1));
        f.write_str(&self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_mark(tab: &Tab) -> TabMarks {
    tab.mark
}

fn widget(passive: bool) -> TabsWidget {
    Tabs::new(TABS)
        .allow_new(true)
        .closable(true)
        .wrap(true)
        .passive(passive)
        .key(tab_key as KeyFn)
        .item_marks(tab_mark as MarkFn)
}

fn fixture_items() -> Vec<Tab> {
    [
        "Overview",
        "Editor",
        "Network",
        "Terminal",
        "History",
        "👩‍💻",
        "Settings",
        "Plugins",
        "Search",
        "Tasks",
        "Metrics",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, label)| Tab {
        id: u64::try_from(index + 1).unwrap_or_default(),
        label: label.to_owned(),
        mark: match index {
            2 => TabMarks::default().dirty(true),
            5 => TabMarks::default().busy(true),
            8 => TabMarks::default().error(true),
            _ => TabMarks::default(),
        },
        displays: Cell::new(0),
    })
    .collect()
}

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    keymap: KeyMap,
    previous: Cell<Option<TabWindowReceipt>>,
    passive: bool,
    empty_clip: bool,
    empty_area: bool,
    render_tabs: Cell<bool>,
    update_tabs_component: Cell<bool>,
    planned: Cell<bool>,
    modal: bool,
    status: Status,
    last_first: Cell<Option<ItemKey>>,
    last_last_visible: Cell<Option<ItemKey>>,
    last_content_len: Cell<usize>,
    last_viewport_len: Cell<usize>,
    lifecycle: RefCell<Vec<LifecycleObservation>>,
    plan_passes: Cell<usize>,
}

impl Page {
    fn new() -> Self {
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::num(1));
        Self {
            state,
            items: fixture_items(),
            actions: Vec::new(),
            keymap: KeyMap::new(),
            previous: Cell::new(None),
            passive: false,
            empty_clip: false,
            empty_area: false,
            render_tabs: Cell::new(true),
            update_tabs_component: Cell::new(true),
            planned: Cell::new(true),
            modal: false,
            status: Status::Ready,
            last_first: Cell::new(None),
            last_last_visible: Cell::new(None),
            last_content_len: Cell::new(0),
            last_viewport_len: Cell::new(0),
            lifecycle: RefCell::new(Vec::new()),
            plan_passes: Cell::new(0),
        }
    }

    fn update_tabs(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.modal && !cx.is_open(OVERLAY) {
            cx.open_layer(OVERLAY, LayerSpec::modal(OVERLAY).initial_focus(OVERLAY));
        }
        let mut response =
            widget(self.passive)
                .status(self.status)
                .update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        let mut response = response.erase();
        if self.modal {
            response |= Button::new(OVERLAY, "Modal").update(cx).erase();
        }
        response
    }

    fn draw_tabs(&self, ui: &mut Ui<'_>, area: Rect) {
        if !self.render_tabs.get() {
            return;
        }
        let area = if self.empty_area {
            Rect::new(area.x, area.y, 0, area.height)
        } else {
            area
        };
        let clip = if self.empty_clip {
            Rect::new(area.x, area.y, 0, 0)
        } else {
            area
        };
        let tabs = widget(self.passive).status(self.status);
        if !self.planned.get() {
            let first_index = self
                .state
                .first()
                .and_then(|first| self.items.iter().position(|item| tab_key(item) == first))
                .unwrap_or(0)
                .min(self.items.len());
            let first = self.items.get(first_index).map(tab_key);
            let _ = tabs.draw(ui, area, &self.state, &self.items);
            self.lifecycle
                .borrow_mut()
                .push(LifecycleObservation::Draw {
                    kind: DrawKind::Legacy,
                    first,
                });
            return;
        }
        self.plan_passes
            .set(self.plan_passes.get().saturating_add(1));
        let layout = tabs.layout_default(area, clip, &self.state, &self.items, self.previous.get());
        self.last_first.set(layout.first_key());
        self.last_last_visible
            .set(layout.rows().last().map(|row| row.key));
        self.last_content_len.set(layout.content_len);
        self.last_viewport_len.set(layout.viewport_len);
        let next_receipt = layout.window_receipt();
        assert!(
            tabs.draw_layout(ui, &layout).is_ok(),
            "fixture plan matches its painter"
        );
        self.lifecycle
            .borrow_mut()
            .push(LifecycleObservation::Draw {
                kind: DrawKind::Planned,
                first: layout.first_key(),
            });
        self.previous.set(next_receipt);
    }

    fn draw_modal(&self, ui: &mut Ui<'_>) {
        if self.modal {
            let _ = ui.layer(OVERLAY, |layer, area| {
                Button::new(OVERLAY, "Modal").draw(layer, area);
            });
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.update_tabs_component.get() {
            let before = self.state.first();
            let response = self.update_tabs(cx);
            self.lifecycle
                .borrow_mut()
                .push(LifecycleObservation::Update {
                    before,
                    after: self.state.first(),
                });
            response
        } else {
            Response::ignored()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.draw_tabs(ui, ui.full());
        self.draw_modal(ui);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

struct NestedPage {
    page: Page,
    parent: ScrollState,
}

impl NestedPage {
    fn new() -> Self {
        Self {
            page: Page::new(),
            parent: ScrollState::new(PARENT_CONTENT),
        }
    }
}

impl App for NestedPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let parent = ScrollRegion::new(PARENT).scrollbar_visible(false);
        parent.update(cx, &mut self.parent, PARENT_CONTENT) | self.page.update_tabs(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        ScrollRegion::new(PARENT).scrollbar_visible(false).draw(
            ui,
            area,
            &self.parent,
            PARENT_CONTENT,
        );
        self.page.draw_tabs(ui, area);
        self.page.draw_modal(ui);
    }

    fn keymap(&self) -> &KeyMap {
        &self.page.keymap
    }
}

fn area_or_empty(area: Option<Rect>) -> Rect {
    area.unwrap_or(Rect::new(0, 0, 0, 0))
}

fn wheel_point<A: App>(h: &Harness<A>, owner: Id) -> Position {
    let area = area_or_empty(h.area_of(owner));
    assert!(
        !area.is_empty(),
        "the wheel owner has a visible control area"
    );
    Position::new(area.x.saturating_add(1), area.y)
}

fn exercise_one_native_notch(axis: Axis) {
    let mut theme = Theme::junie();
    theme.design.motion.wheel_rows = 4;
    let mut h = Harness::new(Page::new(), theme, WIDTH, HEIGHT);
    let _ = h.tick();
    h.app_mut().actions.clear();

    assert_eq!(h.app().state.first(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
    let visible_count = h
        .app()
        .items
        .iter()
        .filter(|item| {
            h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(item.id)))
                .is_some()
        })
        .count();
    assert!(
        visible_count >= 2,
        "fixture shows several tabs: {visible_count}"
    );
    assert!(visible_count < h.app().items.len(), "fixture overflows");

    let position = wheel_point(&h, TABS);
    let published_owner = h
        .runtime()
        .registry()
        .hit_scroll(position, axis)
        .map(|hit| hit.owner);
    let focus_before = h.focus();
    let capture_before = h.runtime().capture_owner();
    let response = h.wheel(axis, 1, position.x, position.y);

    assert_eq!(
        h.app().state.first(),
        Some(ItemKey::num(2)),
        "one positive {axis:?} wheel notch advances exactly one keyed tab even when the theme scales the delta"
    );
    assert_eq!(
        published_owner,
        Some(TABS),
        "the planned strip owns the wheel"
    );
    assert!(
        response.is_changed(),
        "moving the strip changes its response"
    );
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
    assert!(
        h.app().actions.is_empty(),
        "wheel never activates, closes, or creates"
    );
    assert_eq!(
        h.focus(),
        focus_before,
        "wheel does not move keyboard focus"
    );
    assert_eq!(h.runtime().capture_owner(), capture_before);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let _ = h.tick();
    assert_eq!(
        h.app().state.first(),
        Some(ItemKey::num(2)),
        "ordinary update/draw preserves the manually browsed window"
    );
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(2)))
            .is_some()
    );
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
    assert!(h.app().actions.is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn horizontal_wheel_moves_the_planned_keyed_window_without_activation() {
    exercise_one_native_notch(Axis::H);
}

#[test]
fn vertical_wheel_moves_the_planned_keyed_window_without_activation() {
    exercise_one_native_notch(Axis::V);
}

fn scroll_to_endpoint<A: App>(h: &mut Harness<A>, owner: Id, axis: Axis, limit: usize) {
    let mut reached = false;
    for _ in 0..limit {
        let position = wheel_point(h, owner);
        let response = h.wheel(axis, 1, position.x, position.y);
        if !response.is_changed() {
            reached = true;
            break;
        }
    }
    assert!(reached, "bounded fixture reaches its consumed endpoint");
}

#[test]
fn wheel_sign_moves_one_key_and_both_endpoints_are_consumed() {
    let mut theme = Theme::junie();
    theme.design.motion.wheel_rows = 5;
    let mut h = Harness::new(Page::new(), theme, WIDTH, HEIGHT);
    let _ = h.tick();
    let at_start = wheel_point(&h, TABS);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(at_start, Axis::H)
            .map(|hit| hit.owner),
        Some(TABS)
    );
    let negative = h.wheel(Axis::H, -1, at_start.x, at_start.y);
    assert!(negative.is_consumed());
    assert!(!negative.is_changed());
    assert_eq!(h.app().state.first(), Some(ItemKey::num(1)));
    let zero = h.wheel(Axis::H, 0, at_start.x, at_start.y);
    assert!(zero.is_consumed());
    assert!(!zero.is_changed());
    assert_eq!(h.app().state.first(), Some(ItemKey::num(1)));

    let positive = h.wheel(Axis::H, 1, at_start.x, at_start.y);
    assert!(positive.is_changed());
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
    let negative_h = h.wheel(Axis::H, -1, at_start.x, at_start.y);
    assert!(negative_h.is_changed());
    assert_eq!(h.app().state.first(), Some(ItemKey::num(1)));
    let positive_h = h.wheel(Axis::H, 1, at_start.x, at_start.y);
    assert!(positive_h.is_changed());
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));

    let limit = h.app().items.len().saturating_add(2);
    scroll_to_endpoint(&mut h, TABS, Axis::H, limit);
    let endpoint_position = wheel_point(&h, TABS);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(endpoint_position, Axis::H)
            .map(|hit| hit.owner),
        Some(TABS),
        "the planned overflow target remains published at its endpoint"
    );
    let endpoint_first = h.app().state.first();
    let endpoint = h.wheel(Axis::H, 1, endpoint_position.x, endpoint_position.y);
    assert!(endpoint.is_consumed());
    assert!(!endpoint.is_changed());
    assert_eq!(h.app().state.first(), endpoint_first);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let end_index = h
        .app()
        .items
        .iter()
        .position(|item| Some(ItemKey::num(item.id)) == endpoint_first)
        .unwrap_or(usize::MAX);
    assert!(end_index > 0 && end_index < h.app().items.len());
    let reverse = h.wheel(Axis::V, -1, endpoint_position.x, endpoint_position.y);
    assert!(reverse.is_changed());
    assert_eq!(
        h.app().state.first(),
        Some(ItemKey::num(h.app().items[end_index - 1].id)),
        "a negative notch on the vertical axis moves exactly one key backward"
    );
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(1)));
    let forward = h.wheel(Axis::H, 1, endpoint_position.x, endpoint_position.y);
    assert!(forward.is_changed());
    assert_eq!(h.app().state.first(), endpoint_first);
}

#[test]
fn same_active_reorder_removal_and_active_change_reconcile_manual_ownership() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let first_position = wheel_point(&h, TABS);
    let _ = h.wheel(Axis::V, 1, first_position.x, first_position.y);
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));

    h.app_mut().state.set_active(0, ItemKey::num(1));
    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));

    h.app_mut().items.swap(6, 7);
    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));

    h.app_mut().items.remove(1);
    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(3)));
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));

    let active_index = h
        .app()
        .items
        .iter()
        .position(|item| item.id == 9)
        .unwrap_or(usize::MAX);
    assert!(active_index < h.app().items.len());
    h.app_mut().state.set_active(active_index, ItemKey::num(9));
    let _ = h.tick();
    assert_eq!(h.app().state.active(), Some(ItemKey::num(9)));
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(9)))
            .is_some(),
        "changing active identity resumes the existing active-follow behavior"
    );

    h.app_mut().state.set_active(0, ItemKey::num(1));
    let _ = h.tick();
    let limit = h.app().items.len().saturating_add(2);
    scroll_to_endpoint(&mut h, TABS, Axis::H, limit);
    h.app_mut().items.remove(0);
    let _ = h.tick();
    assert_eq!(h.app().state.active(), Some(ItemKey::num(3)));
    assert_eq!(h.app().state.first(), Some(ItemKey::num(3)));
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(3)))
            .is_some(),
        "removing the active key follows the reconciled active identity"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn planned_overflow_arrows_share_manual_follow() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let next = PartRef::item(Part::OVERFLOW, ItemKey::index(1));
    assert!(h.area_of_part(TABS, next).is_some());
    let _ = h.click_part(TABS, next);
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn switching_to_legacy_clears_planned_manual_follow_and_keeps_legacy_arrow_behavior() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let position = wheel_point(&h, TABS);
    let _ = h.wheel(Axis::V, 1, position.x, position.y);
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));

    h.app().lifecycle.borrow_mut().clear();
    h.app().planned.set(false);
    let _ = h.tick();
    // The next real Tick runs the Legacy follow update after its first draw.
    let _ = h.tick();
    let lifecycle = h.app().lifecycle.borrow().clone();
    let first_legacy_draw = lifecycle
        .iter()
        .position(|observation| {
            matches!(
                observation,
                LifecycleObservation::Draw {
                    kind: DrawKind::Legacy,
                    first: Some(first),
                } if *first == ItemKey::num(2)
            )
        })
        .expect("the first transition draw is Legacy and emits the existing key");
    let first_draw = lifecycle
        .iter()
        .position(|observation| matches!(observation, LifecycleObservation::Draw { .. }));
    assert_eq!(first_draw, Some(first_legacy_draw));
    let preserving_update = lifecycle
        .iter()
        .position(|observation| {
            matches!(
                observation,
                LifecycleObservation::Update {
                    before: Some(before),
                    after: Some(after),
                } if *before == ItemKey::num(2) && *after == ItemKey::num(2)
            )
        })
        .expect("the pending planned-frame update preserves the keyed window");
    assert!(preserving_update < first_legacy_draw);
    let legacy_follow_update = lifecycle
        .iter()
        .enumerate()
        .skip(first_legacy_draw.saturating_add(1))
        .find_map(|(index, observation)| {
            matches!(
                observation,
                LifecycleObservation::Update {
                    before: Some(before),
                    after: Some(after),
                } if *before == ItemKey::num(2) && *after == ItemKey::num(1)
            )
            .then_some(index)
        })
        .expect("a later update after the Legacy draw resumes active-follow");
    assert!(legacy_follow_update > first_legacy_draw);
    let legacy_position = wheel_point(&h, TABS);
    assert_ne!(
        h.runtime()
            .registry()
            .hit_scroll(legacy_position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS),
        "legacy RowFn drawing does not claim an unmeasured scroll bound"
    );
    assert_eq!(
        h.app().state.first(),
        Some(ItemKey::num(1)),
        "the next legacy update resumes the pre-feature active-follow policy"
    );

    let right = PartRef::item(Part::OVERFLOW, ItemKey::index(1));
    assert!(h.area_of_part(TABS, right).is_some());
    let _ = h.click_part(TABS, right);
    assert_eq!(
        h.app().state.first(),
        Some(ItemKey::num(1)),
        "legacy overflow actions remain subject to legacy active-follow"
    );

    h.app().planned.set(true);
    let _ = h.tick();
    let planned_position = wheel_point(&h, TABS);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(planned_position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS),
        "the planned path publishes its exact scroll target again"
    );
    let _ = h.wheel(Axis::V, 1, planned_position.x, planned_position.y);
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn fitting_passive_clipped_and_empty_children_leave_vertical_wheel_to_parent() {
    for kind in ["fit", "passive", "clipped", "empty"] {
        let mut app = NestedPage::new();
        if kind == "fit" {
            app.page.items.truncate(1);
        } else if kind == "passive" {
            app.page.passive = true;
        } else if kind == "clipped" {
            app.page.empty_clip = true;
        } else {
            assert_eq!(kind, "empty");
            app.page.items.clear();
        }
        let mut h = Harness::new(app, Theme::junie(), WIDTH, HEIGHT);
        let position = Position::new(8, 0);
        assert_eq!(
            h.runtime()
                .registry()
                .hit_scroll(position, Axis::V)
                .map(|hit| hit.owner),
            Some(PARENT),
            "{kind} planned child leaves the parent as the scroll owner"
        );
        let before_parent = h.app().parent.offset();
        let before_first = h.app().page.state.first();
        let response = h.wheel(Axis::V, 1, position.x, position.y);
        assert!(response.is_changed(), "{kind} parent scrolls");
        assert!(
            h.app().parent.offset() > before_parent,
            "{kind} parent offset advances"
        );
        assert_eq!(h.app().page.state.first(), before_first);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn inert_planned_tabs_do_not_publish_a_scroll_owner() {
    let mut app = NestedPage::new();
    app.page.modal = true;
    let mut h = Harness::new(app, Theme::junie(), WIDTH, HEIGHT);
    let position = Position::new(8, 0);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        None,
        "an inert page Tabs strip does not publish scroll ownership"
    );
    let first = h.app().page.state.first();
    let _ = h.wheel(Axis::V, 1, position.x, position.y);
    assert_eq!(h.app().page.state.first(), first);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn inert_transition_clears_the_previous_planned_scroll_authority() {
    let mut h = Harness::new(NestedPage::new(), Theme::junie(), WIDTH, HEIGHT);
    let position = Position::new(8, 0);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS)
    );
    let moved = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(moved.is_changed());
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));

    h.app_mut().page.modal = true;
    let _ = h.tick();
    assert_ne!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS),
        "an inert planned frame replaces the prior eligible scroll record"
    );
    let first = h.app().page.state.first();
    let _ = h.wheel(Axis::V, 1, position.x, position.y);
    assert_eq!(h.app().page.state.first(), first);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn empty_transition_clears_the_previous_planned_scroll_authority() {
    let mut h = Harness::new(NestedPage::new(), Theme::junie(), WIDTH, HEIGHT);
    let position = Position::new(8, 0);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS)
    );
    let moved = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(moved.is_changed());
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));

    h.app_mut().page.items.clear();
    let _ = h.tick();
    assert_eq!(h.app().page.state.first(), None);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(PARENT),
        "an empty planned frame leaves the parent as the scroll owner"
    );
    let parent_before = h.app().parent.offset();
    let response = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(response.is_changed());
    assert!(h.app().parent.offset() > parent_before);
    assert_eq!(h.app().page.state.first(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn zero_area_transition_clears_the_previous_planned_scroll_authority() {
    let mut h = Harness::new(NestedPage::new(), Theme::junie(), WIDTH, HEIGHT);
    let position = Position::new(8, 0);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS)
    );
    let moved = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(moved.is_changed());
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));

    h.app_mut().page.empty_area = true;
    let _ = h.tick();
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(PARENT),
        "an empty planned area publishes no stale Tabs scroll owner"
    );
    let parent_before = h.app().parent.offset();
    let response = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(response.is_changed());
    assert!(h.app().parent.offset() > parent_before);
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn nonready_status_and_reserved_controls_still_use_the_exact_planned_bound() {
    let mut app = Page::new();
    app.status = Status::Loading;
    let mut h = Harness::new(app, Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let position = wheel_point(&h, TABS);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS),
        "New, Close, marks, and Loading status reservations are measured in the plan"
    );
    let response = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(response.is_changed());
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn overflowing_child_consumes_endpoint_without_parent_fallback() {
    let mut h = Harness::new(NestedPage::new(), Theme::junie(), WIDTH, HEIGHT);
    let position = Position::new(8, 0);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS)
    );
    let limit = h.app().page.items.len().saturating_add(2);
    scroll_to_endpoint(&mut h, TABS, Axis::V, limit);
    let parent_before = h.app().parent.offset();
    let endpoint_position = Position::new(8, 0);
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(endpoint_position, Axis::V)
            .map(|hit| hit.owner),
        Some(TABS)
    );
    let response = h.wheel(Axis::V, 1, endpoint_position.x, endpoint_position.y);
    assert!(response.is_consumed());
    assert!(!response.is_changed());
    assert_eq!(h.app().parent.offset(), parent_before);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn hidden_draw_gap_keeps_manual_window_without_retaining_a_scroll_hit() {
    let mut h = Harness::new(NestedPage::new(), Theme::junie(), WIDTH, HEIGHT);
    let position = Position::new(8, 0);
    let _ = h.wheel(Axis::V, 1, position.x, position.y);
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));

    h.app().page.render_tabs.set(false);
    let _ = h.tick();
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));
    assert_eq!(
        h.runtime()
            .registry()
            .hit_scroll(position, Axis::V)
            .map(|hit| hit.owner),
        Some(PARENT),
        "the absent child draw publishes no stale scroll hit"
    );
    let parent_before = h.app().parent.offset();
    let _ = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(h.app().parent.offset() > parent_before);
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));

    h.app().page.render_tabs.set(true);
    let _ = h.tick();
    assert_eq!(h.app().page.state.first(), Some(ItemKey::num(2)));
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(2)))
            .is_some()
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn resize_cache_gap_without_tabs_update_or_draw_preserves_then_clamps_manual_window() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let limit = h.app().items.len().saturating_add(2);
    scroll_to_endpoint(&mut h, TABS, Axis::H, limit);
    let old_first = h.app().state.first();
    assert_eq!(old_first, Some(ItemKey::num(9)));

    h.app().update_tabs_component.set(false);
    h.app().render_tabs.set(false);
    let _ = h.resize(160, HEIGHT);
    assert_eq!(h.app().state.first(), old_first);
    for _ in 0..2 {
        let _ = h.tick();
    }
    assert_eq!(h.app().state.first(), old_first);
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::num(9)))
            .is_none(),
        "the omitted component publishes no stale item hit"
    );
    assert!(
        h.runtime()
            .registry()
            .hit_scroll(Position::new(8, 0), Axis::H)
            .is_none(),
        "the omitted component publishes no stale wheel owner"
    );

    h.app().lifecycle.borrow_mut().clear();
    h.app().update_tabs_component.set(true);
    h.app().render_tabs.set(true);
    let _ = h.tick();
    let lifecycle = h.app().lifecycle.borrow().clone();
    let unknown_preserving_update = lifecycle
        .iter()
        .position(|observation| {
            matches!(
                observation,
                LifecycleObservation::Update { before, after }
                    if *before == old_first && *after == old_first
            )
        })
        .expect("an update from the Unknown frame preserves the manual first key");
    let fresh_planned_draw = lifecycle
        .iter()
        .position(|observation| {
            matches!(
                observation,
                LifecycleObservation::Draw {
                    kind: DrawKind::Planned,
                    first: Some(first),
                } if *first == ItemKey::num(1)
            )
        })
        .expect("the fresh planned draw clamps its local first key after resize");
    assert!(unknown_preserving_update < fresh_planned_draw);
    let converging_update = lifecycle
        .iter()
        .enumerate()
        .skip(fresh_planned_draw.saturating_add(1))
        .find_map(|(index, observation)| {
            matches!(
                observation,
                LifecycleObservation::Update { before, after }
                    if *before == old_first && *after == Some(ItemKey::num(1))
            )
            .then_some(index)
        })
        .expect("a later update converges stored state to the fresh plan");
    assert!(converging_update > fresh_planned_draw);
    assert_eq!(h.app().last_first.get(), Some(ItemKey::num(1)));
    assert!(
        h.text().contains("Metrics"),
        "the fresh plan clamps to the resized terminal"
    );

    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(1)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn resize_uses_fresh_planned_geometry_and_converges_on_the_next_update() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let limit = h.app().items.len().saturating_add(2);
    scroll_to_endpoint(&mut h, TABS, Axis::H, limit);
    let old_first = h.app().state.first();
    assert_ne!(old_first, Some(ItemKey::num(1)));
    let _ = h.resize(160, HEIGHT);
    assert_eq!(h.app().last_content_len.get(), 11);
    assert_eq!(h.app().last_first.get(), Some(ItemKey::num(1)));
    assert_eq!(
        h.app().state.first(),
        old_first,
        "draw uses fresh local geometry before the next update"
    );
    assert!(
        h.text().contains("Metrics"),
        "the final tab is painted after resize"
    );

    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(1)));
    assert!(h.app().last_viewport_len.get() >= 11);
    assert_eq!(h.app().last_last_visible.get(), Some(ItemKey::num(11)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn same_count_label_replacement_rebuilds_the_exact_manual_bound() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT);
    let _ = h.tick();
    let limit = h.app().items.len().saturating_add(2);
    scroll_to_endpoint(&mut h, TABS, Axis::H, limit);
    let old_first = h.app().state.first();
    assert_eq!(old_first, Some(ItemKey::num(9)));

    h.app().lifecycle.borrow_mut().clear();
    h.app_mut().items[8].label = "S".to_owned();
    h.app_mut().items[10].label = "M".to_owned();
    assert_eq!(h.app().items.len(), 11);
    let _ = h.tick();
    assert_eq!(h.app().last_content_len.get(), 11);
    assert_eq!(h.app().last_first.get(), Some(ItemKey::num(8)));
    assert_eq!(h.app().last_last_visible.get(), Some(ItemKey::num(11)));
    assert_eq!(h.app().state.first(), Some(ItemKey::num(8)));
    let lifecycle = h.app().lifecycle.borrow();
    let fresh_clamped_draw = lifecycle
        .iter()
        .position(|observation| {
            matches!(
                observation,
                LifecycleObservation::Draw {
                    kind: DrawKind::Planned,
                    first: Some(key),
                } if *key == ItemKey::num(8)
            )
        })
        .expect("the first fresh plan records its local clamp to key 8");
    let converging_update = lifecycle
        .iter()
        .position(|observation| {
            matches!(
                observation,
                LifecycleObservation::Update {
                    before: Some(before),
                    after: Some(after),
                } if *before == ItemKey::num(9) && *after == ItemKey::num(8)
            )
        })
        .expect("an actual later update converges stored first from 9 to 8");
    assert!(
        fresh_clamped_draw < converging_update,
        "fresh local clamping precedes the later actual update"
    );
    drop(lifecycle);
    assert!(h.text().contains("Plugins"));
    assert!(h.text().contains("Tasks"));

    let _ = h.tick();
    assert_eq!(h.app().state.first(), Some(ItemKey::num(8)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn update_uses_committed_measurement_without_reformatting_items() {
    let mut h = Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT).with_auto_draw(false);
    let _ = h.tick();
    h.draw();
    for item in &h.app().items {
        item.displays.set(0);
    }
    h.app().plan_passes.set(0);
    let position = wheel_point(&h, TABS);
    let response = h.wheel(Axis::V, 1, position.x, position.y);
    assert!(response.is_changed());
    assert!(
        h.app().items.iter().all(|item| item.displays.get() == 0),
        "wheel update consumes the last committed exact bound without formatting labels"
    );
    h.draw();
    let plan_passes = h.app().plan_passes.get();
    assert!(
        plan_passes > 0,
        "the draw performed at least one actual plan pass"
    );
    assert!(
        h.app()
            .items
            .iter()
            .all(|item| item.displays.get() == plan_passes),
        "each actual plan pass takes one fresh snapshot of every item"
    );
    assert_eq!(h.app().state.first(), Some(ItemKey::num(2)));
}
