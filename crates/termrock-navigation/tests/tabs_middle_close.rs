#![allow(clippy::unwrap_used, reason = "test lifecycle assertions")]
#![allow(clippy::panic, reason = "bounded runtime settle helper")]

use core::fmt;

use ratatui::crossterm::event::{Event, MouseButton, MouseEvent, MouseEventKind};
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_core::event::{KeyModifiers, RoutedInput};
use termrock_core::{Id, Intent, ItemKey, Part, PartRef, Phase, Response};
use termrock_navigation::{TabCloseReveal, TabMarks, Tabs, TabsAction, TabsState};
use termrock_runtime::rowui::RowUi;
use termrock_runtime::{App, Cx, Runtime, Ui};
use termrock_theme::{GlyphRole, Theme};

const TABS: Id = Id::root("tabs.middle-close");
const AREA: Rect = Rect::new(0, 0, 48, 5);
const PRIMARY_TAB: PartRef = PartRef::item(Part::TAB, ItemKey::Num(2));

#[derive(Clone, Copy)]
struct Tab {
    key: u64,
    label: &'static str,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.key)
}

fn tab_row(tab: &Tab, row: &mut RowUi<'_>) {
    row.label(tab.label);
}

type TabKeyFn = fn(&Tab) -> ItemKey;
type TabRowFn = for<'a> fn(&Tab, &mut RowUi<'a>);

#[derive(Clone, Copy, Debug)]
enum MiddlePolicyOrder {
    BeforeKey,
    BetweenKeyAndRow,
    AfterRow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointerObservation {
    phase: Phase,
    part: PartRef,
    press_count: Option<u8>,
    click_count: Option<u8>,
}

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    close_keys: Option<Vec<ItemKey>>,
    closable: bool,
    middle_click_closes: Option<bool>,
    middle_policy_order: MiddlePolicyOrder,
    item_marks: bool,
    item_marks_before_middle: bool,
    close_reveal: TabCloseReveal,
    capture_primary_tab: bool,
    reference_draw: bool,
    reorder_items_on_middle: bool,
    remove_item_on_middle: Option<u64>,
    remove_close_key_on_middle: Option<ItemKey>,
    pointers: Vec<PointerObservation>,
}

impl Page {
    fn new() -> Self {
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::num(1));
        Self {
            state,
            items: vec![
                Tab {
                    key: 1,
                    label: "Alpha",
                },
                Tab {
                    key: 2,
                    label: "Beta",
                },
                Tab {
                    key: 3,
                    label: "Gamma",
                },
            ],
            actions: Vec::new(),
            close_keys: None,
            closable: true,
            middle_click_closes: None,
            middle_policy_order: MiddlePolicyOrder::BeforeKey,
            item_marks: false,
            item_marks_before_middle: false,
            close_reveal: TabCloseReveal::Always,
            capture_primary_tab: false,
            reference_draw: false,
            reorder_items_on_middle: false,
            remove_item_on_middle: None,
            remove_close_key_on_middle: None,
            pointers: Vec::new(),
        }
    }
}

fn tabs<'a>(
    closable: bool,
    close_keys: Option<&'a [ItemKey]>,
    middle_click_closes: Option<bool>,
    middle_policy_order: MiddlePolicyOrder,
    item_marks: bool,
    item_marks_before_middle: bool,
    close_reveal: TabCloseReveal,
) -> Tabs<'a, Tab, TabKeyFn, TabRowFn> {
    let mut builder = Tabs::new(TABS)
        .closable(closable)
        .close_reveal(close_reveal);
    if let Some(keys) = close_keys {
        builder = builder.closable_keys(keys);
    }
    if item_marks && item_marks_before_middle {
        builder = builder.item_marks(dirty_marks);
    }

    let key: TabKeyFn = tab_key;
    let row: TabRowFn = tab_row;
    let builder = match (middle_click_closes, middle_policy_order) {
        (Some(enabled), MiddlePolicyOrder::BeforeKey) => {
            builder.middle_click_closes(enabled).key(key).row(row)
        }
        (Some(enabled), MiddlePolicyOrder::BetweenKeyAndRow) => {
            builder.key(key).middle_click_closes(enabled).row(row)
        }
        (Some(enabled), MiddlePolicyOrder::AfterRow) => {
            builder.key(key).row(row).middle_click_closes(enabled)
        }
        (None, _) => builder.key(key).row(row),
    };
    if item_marks && !item_marks_before_middle {
        builder.item_marks(dirty_marks)
    } else {
        builder
    }
}

fn dirty_marks(_: &Tab) -> TabMarks {
    TabMarks::default().dirty(true)
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut capture = None;
        let mut middle_received = false;
        for intent in cx.intents(TABS) {
            if let Intent::Pointer { phase, part, .. } = intent {
                self.pointers.push(PointerObservation {
                    phase,
                    part,
                    press_count: cx.pointer_press_count(TABS, part),
                    click_count: cx.pointer_click_count(TABS, part),
                });
                if self.capture_primary_tab && phase == Phase::Press && part == PRIMARY_TAB {
                    capture = Some(part);
                }
                middle_received |= phase == Phase::Middle;
            }
        }
        if middle_received {
            if self.reorder_items_on_middle {
                self.items.swap(1, 2);
                self.reorder_items_on_middle = false;
            }
            if let Some(key) = self.remove_item_on_middle.take() {
                self.items.retain(|tab| tab.key != key);
            }
            if let Some(key) = self.remove_close_key_on_middle.take()
                && let Some(keys) = &mut self.close_keys
            {
                keys.retain(|current| *current != key);
            }
        }
        if let Some(part) = capture {
            assert!(cx.capture(TABS, part), "primary tab press requests capture");
        }

        let tabs = tabs(
            self.closable,
            self.close_keys.as_deref(),
            self.middle_click_closes,
            self.middle_policy_order,
            self.item_marks,
            self.item_marks_before_middle,
            self.close_reveal,
        );
        let mut response = tabs.update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.reference_draw {
            ui.reference(None, |ui| {
                tabs(
                    self.closable,
                    self.close_keys.as_deref(),
                    self.middle_click_closes,
                    self.middle_policy_order,
                    self.item_marks,
                    self.item_marks_before_middle,
                    self.close_reveal,
                )
                .draw(ui, ui.full(), &self.state, &self.items);
            });
        } else {
            tabs(
                self.closable,
                self.close_keys.as_deref(),
                self.middle_click_closes,
                self.middle_policy_order,
                self.item_marks,
                self.item_marks_before_middle,
                self.close_reveal,
            )
            .draw(ui, ui.full(), &self.state, &self.items);
        }
    }
}

fn present(rt: &mut Runtime<Page>) {
    for _ in 0..16 {
        if rt.needs_settle() {
            let _ = rt.settle();
        }
        if rt.needs_present() {
            rt.draw_buffer(AREA, &mut Buffer::empty(AREA))
                .commit_presented();
        }
        if !rt.needs_settle() && !rt.needs_present() {
            return;
        }
    }
    panic!("runtime did not settle and present");
}

fn ready_runtime(page: Page) -> Runtime<Page> {
    let mut rt = Runtime::new(page, Theme::junie());
    let _ = rt.initialize();
    present(&mut rt);
    rt
}

fn presented_buffer(rt: &mut Runtime<Page>) -> Buffer {
    let mut buffer = Buffer::empty(AREA);
    rt.draw_buffer(AREA, &mut buffer).commit_presented();
    buffer
}

fn assert_dirty_mark(rt: &mut Runtime<Page>, key: ItemKey, label: &str) {
    let tab = part_area(rt, Part::TAB, key).expect("marked tab has a committed hit");
    let buffer = presented_buffer(rt);
    let label_width = u16::try_from(label.len()).expect("test labels fit in a terminal row");
    let mark_x = tab.x.saturating_add(2).saturating_add(label_width);
    assert_eq!(
        buffer
            .cell(Position::new(mark_x, tab.y))
            .expect("the mark is inside the tab frame")
            .symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Dirty),
        "the row carries a real nonempty item mark"
    );
    assert_eq!(
        part_area(rt, Part::ICON, key),
        None,
        "item marks do not become separate hit targets"
    );
}

fn routed_mouse(kind: MouseEventKind, x: u16, y: u16) -> RoutedInput {
    RoutedInput::from_crossterm(Event::Mouse(MouseEvent {
        kind,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE.into(),
    }))
    .expect("backend mouse event is retained by the routed decoder")
}

fn route_mouse(rt: &mut Runtime<Page>, kind: MouseEventKind, x: u16, y: u16) -> Response<()> {
    let response = rt
        .handle_routed(routed_mouse(kind, x, y))
        .expect("a presented runtime handles backend-routed input");
    present(rt);
    response
}

fn part_area(rt: &Runtime<Page>, part: Part, key: ItemKey) -> Option<Rect> {
    rt.area_of_part(TABS, PartRef::item(part, key))
}

fn middle_down(rt: &mut Runtime<Page>, point: Position) -> Response<()> {
    route_mouse(
        rt,
        MouseEventKind::Down(MouseButton::Middle),
        point.x,
        point.y,
    )
}

fn center(area: Rect) -> Position {
    Position::new(
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    )
}

#[test]
fn default_middle_closes_the_clicked_stable_tab_without_activating_it() {
    let mut rt = ready_runtime(Page::new());
    let target = ItemKey::num(3);
    let cursor = ItemKey::num(2);
    let cursor_tab = part_area(&rt, Part::TAB, cursor).expect("cursor tab has a committed hit");
    let cursor_point = center(cursor_tab);
    let _ = route_mouse(
        &mut rt,
        MouseEventKind::Down(MouseButton::Left),
        cursor_point.x,
        cursor_point.y,
    );
    let _ = route_mouse(
        &mut rt,
        MouseEventKind::Up(MouseButton::Left),
        AREA.right() - 1,
        AREA.bottom() - 1,
    );
    assert_eq!(rt.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(rt.app().state.cursor(), Some(cursor));
    assert!(rt.app().actions.is_empty());
    let before = (
        rt.app().state.active(),
        rt.app().state.cursor(),
        rt.app().state.first(),
        rt.focus(),
        rt.capture_owner(),
    );
    let tab = part_area(&rt, Part::TAB, target).expect("target tab has a committed hit region");

    let response = middle_down(&mut rt, center(tab));

    assert!(response.is_consumed());
    assert_eq!(rt.app().actions, [TabsAction::Close(target)]);
    assert_eq!(
        (
            rt.app().state.active(),
            rt.app().state.cursor(),
            rt.app().state.first(),
            rt.focus(),
            rt.capture_owner(),
        ),
        before,
        "middle close leaves active, cursor, viewport, focus and capture unchanged"
    );
    for kind in [
        MouseEventKind::Up(MouseButton::Middle),
        MouseEventKind::Drag(MouseButton::Middle),
    ] {
        assert!(
            RoutedInput::from_crossterm(Event::Mouse(MouseEvent {
                kind,
                column: tab.x,
                row: tab.y,
                modifiers: KeyModifiers::NONE.into(),
            }))
            .is_none(),
            "middle Up and Drag do not produce routed clicks"
        );
    }
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn marked_rows_keep_default_middle_close_and_item_eligibility() {
    let target = ItemKey::num(3);

    for part in [Part::TAB, Part::CLOSE] {
        let mut page = Page::new();
        page.item_marks = true;
        let mut rt = ready_runtime(page);
        assert_dirty_mark(&mut rt, target, "Gamma");
        let before = (
            rt.app().state.active(),
            rt.app().state.cursor(),
            rt.app().state.first(),
        );
        let area = part_area(&rt, part, target).expect("default middle close keeps this hit");

        let response = middle_down(&mut rt, center(area));

        assert!(response.is_consumed());
        assert_eq!(rt.app().actions, [TabsAction::Close(target)]);
        assert_eq!(
            (
                rt.app().state.active(),
                rt.app().state.cursor(),
                rt.app().state.first()
            ),
            before,
            "closing a marked row leaves the active, cursor and viewport keys unchanged"
        );
        assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
    }

    for close_policy in [None, Some(vec![ItemKey::num(2)])] {
        let mut page = Page::new();
        page.item_marks = true;
        if let Some(keys) = close_policy {
            page.close_keys = Some(keys);
        } else {
            page.closable = false;
        }
        let mut rt = ready_runtime(page);
        assert_dirty_mark(&mut rt, target, "Gamma");
        let tab = part_area(&rt, Part::TAB, target).expect("ineligible row remains visible");
        assert_eq!(part_area(&rt, Part::CLOSE, target), None);
        let before = (rt.app().state.active(), rt.app().state.cursor());

        let response = middle_down(&mut rt, center(tab));

        assert!(!response.is_consumed());
        assert!(rt.app().actions.is_empty());
        assert_eq!((rt.app().state.active(), rt.app().state.cursor()), before);
        assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
    }
}

#[test]
fn marked_rows_keep_middle_opt_out_through_all_builder_orders() {
    let target = ItemKey::num(3);

    for order in [
        MiddlePolicyOrder::BeforeKey,
        MiddlePolicyOrder::BetweenKeyAndRow,
        MiddlePolicyOrder::AfterRow,
    ] {
        for marks_before_middle in [true, false] {
            let mut page = Page::new();
            page.middle_click_closes = Some(false);
            page.middle_policy_order = order;
            page.item_marks = true;
            page.item_marks_before_middle = marks_before_middle;
            let mut rt = ready_runtime(page);
            assert_dirty_mark(&mut rt, target, "Gamma");
            let tab = part_area(&rt, Part::TAB, target).expect("marked tab remains pointerable");
            let state_before = (rt.app().state.active(), rt.app().state.cursor());

            let response = middle_down(&mut rt, center(tab));

            assert!(
                !response.is_consumed(),
                "marked opt-out remains ignored for {order:?}, mapper-before={marks_before_middle}"
            );
            assert!(rt.app().actions.is_empty());
            assert_eq!(
                (rt.app().state.active(), rt.app().state.cursor()),
                state_before
            );

            let tab = part_area(&rt, Part::TAB, target).expect("tab remains live after Middle");
            let point = center(tab);
            let _ = route_mouse(
                &mut rt,
                MouseEventKind::Down(MouseButton::Left),
                point.x,
                point.y,
            );
            let _ = route_mouse(
                &mut rt,
                MouseEventKind::Up(MouseButton::Left),
                point.x,
                point.y,
            );
            assert_eq!(rt.app().actions, [TabsAction::Activated(target)]);

            let close = part_area(&rt, Part::CLOSE, target).expect("default close remains live");
            let point = center(close);
            let _ = route_mouse(
                &mut rt,
                MouseEventKind::Down(MouseButton::Left),
                point.x,
                point.y,
            );
            let _ = route_mouse(
                &mut rt,
                MouseEventKind::Up(MouseButton::Left),
                point.x,
                point.y,
            );
            assert_eq!(
                rt.app().actions,
                [TabsAction::Activated(target), TabsAction::Close(target)],
                "ordinary activation and close work with marked rows for {order:?}, mapper-before={marks_before_middle}"
            );
            assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
        }
    }
}

#[test]
fn visible_close_target_is_also_a_middle_close_target() {
    let mut page = Page::new();
    page.middle_click_closes = Some(true);
    let mut rt = ready_runtime(page);
    let target = ItemKey::num(3);
    let close = part_area(&rt, Part::CLOSE, target).expect("always-revealed close is live");

    let response = middle_down(&mut rt, center(close));

    assert!(response.is_consumed());
    assert_eq!(rt.app().actions, [TabsAction::Close(target)]);
    assert_eq!(rt.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(rt.app().state.cursor(), Some(ItemKey::num(1)));
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn hover_reveal_keeps_a_hidden_close_middle_clickable_through_its_tab_cell() {
    let mut page = Page::new();
    page.close_reveal = TabCloseReveal::Hover;
    let mut rt = ready_runtime(page);
    let target = ItemKey::num(3);
    let tab = part_area(&rt, Part::TAB, target).expect("hidden target still has its tab hit");
    assert_eq!(part_area(&rt, Part::CLOSE, target), None);

    let reserved_close = Position::new(tab.right().saturating_sub(2), tab.y);
    let _ = route_mouse(
        &mut rt,
        MouseEventKind::Moved,
        reserved_close.x,
        reserved_close.y,
    );
    assert!(
        part_area(&rt, Part::CLOSE, target).is_some(),
        "hover reveals the reserved close part"
    );

    let response = middle_down(&mut rt, reserved_close);
    assert!(response.is_consumed());
    assert_eq!(rt.app().actions, [TabsAction::Close(target)]);
    assert_eq!(rt.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(rt.app().state.cursor(), Some(ItemKey::num(1)));
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn opt_out_plain_tabs_and_ineligible_items_do_not_close() {
    let target = ItemKey::num(3);

    for order in [
        MiddlePolicyOrder::BeforeKey,
        MiddlePolicyOrder::BetweenKeyAndRow,
        MiddlePolicyOrder::AfterRow,
    ] {
        let mut opted_out = Page::new();
        opted_out.middle_click_closes = Some(false);
        opted_out.middle_policy_order = order;
        let mut rt = ready_runtime(opted_out);
        let tab = part_area(&rt, Part::TAB, target).expect("tab remains pointer-addressable");
        let state_before = (rt.app().state.active(), rt.app().state.cursor());
        let response = middle_down(&mut rt, center(tab));
        assert!(
            !response.is_consumed(),
            "opt-out middle press stays ignored"
        );
        assert!(rt.app().actions.is_empty());
        assert_eq!(
            (rt.app().state.active(), rt.app().state.cursor()),
            state_before,
            "opt-out remains inert for builder order {order:?}"
        );
        assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
    }

    let mut plain = Page::new();
    plain.closable = false;
    let mut rt = ready_runtime(plain);
    let tab = part_area(&rt, Part::TAB, target).expect("plain tab remains pointer-addressable");
    assert_eq!(part_area(&rt, Part::CLOSE, target), None);
    let response = middle_down(&mut rt, center(tab));
    assert!(!response.is_consumed(), "plain tabs ignore middle press");
    assert!(rt.app().actions.is_empty());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());

    let mut admitted_by_key = Page::new();
    admitted_by_key.closable = false;
    admitted_by_key.close_keys = Some(vec![target]);
    let mut rt = ready_runtime(admitted_by_key);
    let tab = part_area(&rt, Part::TAB, target).expect("key-admitted tab is pointer-addressable");
    let response = middle_down(&mut rt, center(tab));
    assert!(response.is_consumed());
    assert_eq!(rt.app().actions, [TabsAction::Close(target)]);
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());

    let mut empty_policy = Page::new();
    empty_policy.close_keys = Some(Vec::new());
    let mut rt = ready_runtime(empty_policy);
    let tab =
        part_area(&rt, Part::TAB, target).expect("unclosable tab remains pointer-addressable");
    assert_eq!(part_area(&rt, Part::CLOSE, target), None);
    let response = middle_down(&mut rt, center(tab));
    assert!(
        !response.is_consumed(),
        "empty key policy stays authoritative"
    );
    assert!(rt.app().actions.is_empty());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());

    let mut selectively_closable = Page::new();
    selectively_closable.close_keys = Some(vec![ItemKey::num(2)]);
    let mut rt = ready_runtime(selectively_closable);
    let tab = part_area(&rt, Part::TAB, target).expect("ineligible item still has a tab hit");
    assert_eq!(part_area(&rt, Part::CLOSE, target), None);
    let response = middle_down(&mut rt, center(tab));
    assert!(
        !response.is_consumed(),
        "ineligible item ignores middle press"
    );
    assert!(rt.app().actions.is_empty());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn current_items_are_resolved_by_stable_key_at_dispatch_time() {
    let target = ItemKey::num(3);

    let mut page = Page::new();
    page.reorder_items_on_middle = true;
    let mut rt = ready_runtime(page);
    let target_close = part_area(&rt, Part::CLOSE, target).expect("target close is committed");
    let response = middle_down(&mut rt, center(target_close));
    assert!(response.is_consumed());
    assert_eq!(rt.app().actions, [TabsAction::Close(target)]);
    assert_eq!(rt.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(rt.app().state.cursor(), Some(ItemKey::num(1)));
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());

    let mut page = Page::new();
    page.remove_item_on_middle = Some(3);
    let mut rt = ready_runtime(page);
    let stale_close = part_area(&rt, Part::CLOSE, target).expect("target close is committed");
    let response = middle_down(&mut rt, center(stale_close));
    assert!(!response.is_consumed(), "removed stable key stays ignored");
    assert!(
        rt.app().actions.is_empty(),
        "removed stable keys are ignored"
    );
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());

    let mut page = Page::new();
    page.close_keys = Some(vec![ItemKey::num(2), target]);
    page.remove_close_key_on_middle = Some(target);
    let mut rt = ready_runtime(page);
    let stale_close = part_area(&rt, Part::CLOSE, target).expect("target close is committed");
    let response = middle_down(&mut rt, center(stale_close));
    assert!(
        !response.is_consumed(),
        "revoked close eligibility stays ignored"
    );
    assert!(
        rt.app().actions.is_empty(),
        "updated close policy is rechecked"
    );
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn middle_close_does_not_cancel_an_in_flight_primary_tab_capture() {
    let mut page = Page::new();
    page.capture_primary_tab = true;
    page.close_reveal = TabCloseReveal::Hover;
    let mut rt = ready_runtime(page);
    let primary = part_area(&rt, Part::TAB, ItemKey::num(2)).expect("primary tab is live");
    assert_eq!(part_area(&rt, Part::CLOSE, ItemKey::num(2)), None);
    let primary_point = Position::new(primary.x.saturating_add(1), primary.y);

    let _ = route_mouse(
        &mut rt,
        MouseEventKind::Down(MouseButton::Left),
        primary_point.x,
        primary_point.y,
    );
    assert_eq!(rt.capture_owner(), Some(TABS));
    assert_eq!(rt.app().state.active(), Some(ItemKey::num(1)));
    assert_eq!(rt.app().state.cursor(), Some(ItemKey::num(2)));
    assert!(rt.app().actions.is_empty());
    assert_eq!(
        part_area(&rt, Part::CLOSE, ItemKey::num(2)),
        None,
        "the held tab's hidden close remains deferred from the hit registry"
    );
    let primary_press = rt
        .app()
        .pointers
        .iter()
        .find(|pointer| pointer.phase == Phase::Press && pointer.part == PRIMARY_TAB)
        .expect("primary left Down is a real tab press");
    assert_eq!(primary_press.press_count, Some(1));
    assert_eq!(primary_press.click_count, Some(1));

    let other_tab = part_area(&rt, Part::TAB, ItemKey::num(3))
        .expect("middle target remains independently registered");
    assert_eq!(part_area(&rt, Part::CLOSE, ItemKey::num(3)), None);
    let response = middle_down(&mut rt, center(other_tab));
    assert!(response.is_consumed());
    assert_eq!(rt.capture_owner(), Some(TABS));
    assert_eq!(rt.app().actions, [TabsAction::Close(ItemKey::num(3))]);
    assert_eq!(part_area(&rt, Part::CLOSE, ItemKey::num(2)), None);
    let middle = rt
        .app()
        .pointers
        .iter()
        .find(|pointer| pointer.phase == Phase::Middle)
        .expect("the backend middle press reaches Tabs");
    assert_eq!(middle.part, PartRef::item(Part::TAB, ItemKey::num(3)));
    assert_eq!(middle.press_count, None);
    assert_eq!(middle.click_count, None);

    let _ = route_mouse(
        &mut rt,
        MouseEventKind::Up(MouseButton::Left),
        primary_point.x,
        primary_point.y,
    );
    assert_eq!(rt.capture_owner(), None);
    assert_eq!(
        rt.app().actions,
        [
            TabsAction::Close(ItemKey::num(3)),
            TabsAction::Activated(ItemKey::num(2)),
        ]
    );
    assert_eq!(
        rt.app()
            .actions
            .iter()
            .filter(|action| matches!(action, TabsAction::Activated(ItemKey::Num(2))))
            .count(),
        1
    );
    assert_eq!(rt.app().state.active(), Some(ItemKey::num(2)));
    assert_eq!(rt.app().state.cursor(), Some(ItemKey::num(2)));
    let primary_release = rt
        .app()
        .pointers
        .iter()
        .find(|pointer| pointer.phase == Phase::Release && pointer.part == PRIMARY_TAB)
        .expect("the original primary press reaches its actual Release");
    assert_eq!(primary_release.press_count, Some(1));
    assert_eq!(primary_release.click_count, Some(1));
    let primary_click = rt
        .app()
        .pointers
        .iter()
        .find(|pointer| pointer.phase == Phase::Click && pointer.part == PRIMARY_TAB)
        .expect("the surviving primary press reaches its normal Click");
    assert_eq!(primary_click.press_count, Some(1));
    assert_eq!(primary_click.click_count, Some(1));
    for phase in [Phase::Press, Phase::Middle, Phase::Release, Phase::Click] {
        assert_eq!(
            rt.app()
                .pointers
                .iter()
                .filter(|pointer| pointer.phase == phase)
                .count(),
            1,
            "one real pointer event of phase {phase:?}"
        );
    }
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}

#[test]
fn reference_draw_does_not_publish_middle_close_hits() {
    let mut page = Page::new();
    page.reference_draw = true;
    let mut rt = ready_runtime(page);
    assert_eq!(part_area(&rt, Part::CLOSE, ItemKey::num(3)), None);

    let response = middle_down(&mut rt, Position::new(5, 0));

    assert!(!response.is_consumed());
    assert!(rt.app().actions.is_empty());
    assert!(rt.diagnostics().is_empty(), "{:?}", rt.diagnostics());
}
