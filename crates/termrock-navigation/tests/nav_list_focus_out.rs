use ratatui_core::layout::Position;
use termrock_controls::Button;
use termrock_core::{Id, Intent, ItemKey, KeyCode, MouseKind, Part, PartRef, Phase, Response};
use termrock_navigation::{NavList, NavListAction, NavListState};
use termrock_runtime::{App, Cx, Dismiss, LayerEvent, LayerSize, LayerSpec, Rect, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("nav-list.focus-out");
const OTHER: Id = Id::root("nav-list.focus-out.other");
const TOOLTIP: Id = Id::root("nav-list.focus-out.tooltip");
const ROWS: [&str; 12] = [
    "row-0", "row-1", "row-2", "row-3", "row-4", "row-5", "row-6", "row-7", "row-8", "row-9",
    "row-10", "row-11",
];

fn key(item: &&'static str) -> ItemKey {
    ItemKey::text(item)
}

fn list() -> NavList<'static, &'static str, fn(&&'static str) -> ItemKey> {
    NavList::new(LIST)
        .key(key as fn(&&'static str) -> ItemKey)
        .scrollable(true)
}

struct Page {
    state: NavListState,
    items: Vec<&'static str>,
    mounted: bool,
    probe_before_normal_update: bool,
    focus_away_on_press: bool,
    focus_out_handled: usize,
    unsupported: usize,
    actions: Vec<NavListAction>,
    open_tooltip_on_press: bool,
    probe_mixed_layer_settle: bool,
    mixed_batch_confirmed: bool,
    mixed_helper_returned_none: bool,
    mixed_state_unchanged_before_fallback: bool,
    mixed_layer_receipt_valid: bool,
    mixed_fallback_calls: usize,
    mixed_navigation_facts_unchanged_after_fallback: bool,
    mixed_actions_unchanged_after_fallback: bool,
}

impl Page {
    fn new() -> Self {
        Self {
            state: NavListState::new(),
            items: ROWS.to_vec(),
            mounted: true,
            probe_before_normal_update: false,
            focus_away_on_press: false,
            focus_out_handled: 0,
            unsupported: 0,
            actions: Vec::new(),
            open_tooltip_on_press: false,
            probe_mixed_layer_settle: false,
            mixed_batch_confirmed: false,
            mixed_helper_returned_none: false,
            mixed_state_unchanged_before_fallback: false,
            mixed_layer_receipt_valid: false,
            mixed_fallback_calls: 0,
            mixed_navigation_facts_unchanged_after_fallback: false,
            mixed_actions_unchanged_after_fallback: false,
        }
    }

    fn update_list(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = list().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.action_ref() {
            self.actions.push(*action);
        }
        response.erase()
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut result = Button::new(OTHER, "Other").update(cx).erase();
        let open_tooltip_after_press = self.open_tooltip_on_press
            && cx.update_cause() == UpdateCause::Event
            && cx.intents(LIST).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Press,
                        ..
                    }
                )
            });
        if !self.mounted {
            if cx.update_cause() == UpdateCause::Settle {
                match list().update_focus_out(cx) {
                    Some(response) => {
                        self.focus_out_handled += 1;
                        result |= response;
                    }
                    None => self.unsupported += 1,
                }
            }
            return result;
        }

        if self.probe_mixed_layer_settle && cx.update_cause() == UpdateCause::Settle {
            let mut focus_out = false;
            let mut opened_layer = false;
            for intent in cx.intents(LIST) {
                match intent {
                    Intent::FocusOut { .. } => focus_out = true,
                    Intent::Layer(LayerEvent::Opened) => opened_layer = true,
                    _ => {}
                }
            }
            self.mixed_batch_confirmed = focus_out && opened_layer;

            let before_fallback = self.state.clone();
            let handled = list().update_focus_out(cx);
            self.mixed_helper_returned_none = handled.is_none();
            self.mixed_state_unchanged_before_fallback = self.state == before_fallback;
            if handled.is_none() {
                let cursor_before_fallback = self.state.cursor();
                let current_before_fallback = self.state.current();
                let scroll_before_fallback = *self.state.scroll();
                let actions_before_fallback = self.actions.clone();
                let receipt = cx.take_layer_intent_receipt(TOOLTIP);
                self.mixed_layer_receipt_valid = receipt.is_some_and(|receipt| {
                    receipt.event() == LayerEvent::Opened && receipt.opening().owner() == LIST
                });
                self.mixed_fallback_calls += 1;
                result |= self.update_list(cx);
                let scroll_after_fallback = self.state.scroll();
                self.mixed_navigation_facts_unchanged_after_fallback = self.state.cursor()
                    == cursor_before_fallback
                    && self.state.current() == current_before_fallback
                    && scroll_after_fallback.offset() == scroll_before_fallback.offset()
                    && scroll_after_fallback.content_len() == scroll_before_fallback.content_len()
                    && scroll_after_fallback.viewport_len()
                        == scroll_before_fallback.viewport_len()
                    && scroll_after_fallback.follow_cursor()
                        == scroll_before_fallback.follow_cursor()
                    && scroll_after_fallback.context() == scroll_before_fallback.context();
                self.mixed_actions_unchanged_after_fallback =
                    self.actions == actions_before_fallback;
            }
        } else if self.probe_before_normal_update && cx.update_cause() == UpdateCause::Event {
            assert!(list().update_focus_out(cx).is_none());
            result |= self.update_list(cx);
        } else {
            result |= self.update_list(cx);
        }
        if open_tooltip_after_press {
            self.open_tooltip_on_press = false;
            cx.open_layer(
                TOOLTIP,
                LayerSpec::tooltip(LIST, Position::new(0, 0))
                    .size(LayerSize::Fixed(6, 2))
                    .dismiss(Dismiss::NONE),
            );
            cx.focus(OTHER);
        }
        if self.focus_away_on_press
            && cx.intents(LIST).any(|intent| {
                matches!(
                    intent,
                    Intent::Pointer {
                        phase: Phase::Press,
                        ..
                    }
                )
            })
        {
            self.focus_away_on_press = false;
            self.items.pop();
            self.mounted = false;
            cx.focus(OTHER);
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OTHER, "Other").draw(ui, Rect::new(0, 0, 24, 1));
        list().draw(ui, Rect::new(0, 1, 24, 5), &self.state, &self.items);
    }
}

fn harness() -> Harness<Page> {
    Harness::new(Page::new(), Theme::junie(), 24, 6)
}

#[test]
fn focused_route_settle_preserves_nav_state_without_source_reconciliation() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.current(), Some(ItemKey::text("row-11")));
    assert!(h.app().state.scroll().offset() > 0);
    h.app_mut().actions.clear();
    let before = h.app().state.clone();
    let area = h
        .area_of_part(LIST, PartRef::item(Part::ROW, ItemKey::text("row-11")))
        .expect("last current row is visible after End");

    h.app_mut().focus_away_on_press = true;
    let _ = h.mouse(
        MouseKind::Down,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );

    assert_eq!(h.focus(), Some(OTHER));
    assert_eq!(
        h.app().state,
        before,
        "NavList focus handling must not reconcile source"
    );
    assert!(h.app().focus_out_handled > 0);
    assert_eq!(h.app().unsupported, 0);
    assert!(h.app().actions.is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    h.app_mut().mounted = true;
    let _ = h.mouse(
        MouseKind::Up,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    assert_eq!(h.app().state.current(), None);
    assert_ne!(h.app().state.cursor(), Some(ItemKey::text("row-11")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn wrong_cause_falls_back_to_same_real_nav_list_action_once() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    h.app_mut().probe_before_normal_update = true;

    let _ = h.key(KeyCode::Down);

    assert_eq!(
        h.app().actions,
        vec![NavListAction::Moved(ItemKey::text("row-1"))]
    );
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("row-1")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn mixed_settle_layer_open_falls_back_without_navlist_reconciliation() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().state.current(), Some(ItemKey::text("row-11")));
    assert!(h.app().state.scroll().offset() > 0);
    h.app_mut().actions.clear();
    h.app_mut().open_tooltip_on_press = true;
    h.app_mut().probe_mixed_layer_settle = true;
    let area = h
        .area_of_part(LIST, PartRef::item(Part::ROW, ItemKey::text("row-11")))
        .expect("last current row is visible after End");

    let _ = h.mouse(
        MouseKind::Down,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );

    assert_eq!(h.focus(), Some(OTHER));
    assert!(
        h.app().mixed_batch_confirmed,
        "Settle had FocusOut + Layer::Opened"
    );
    assert!(h.app().mixed_helper_returned_none);
    assert!(h.app().mixed_state_unchanged_before_fallback);
    assert!(h.app().mixed_layer_receipt_valid);
    assert_eq!(h.app().mixed_fallback_calls, 1);
    assert!(
        h.app().mixed_navigation_facts_unchanged_after_fallback,
        "normal fallback may consume a pending reveal, but must preserve cursor/current/scroll facts"
    );
    assert!(h.app().mixed_actions_unchanged_after_fallback);
    assert_eq!(h.app().state.current(), Some(ItemKey::text("row-11")));
    assert_eq!(h.app().actions, Vec::<NavListAction>::new());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
