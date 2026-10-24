use ratatui_core::layout::Position;
use termrock_collections::SelectMode;
use termrock_controls::Button;
use termrock_core::{Id, Intent, ItemKey, KeyCode, MouseKind, Part, PartRef, Phase, Response};
use termrock_navigation::{List, ListAction, ListState};
use termrock_runtime::{App, Cx, Dismiss, LayerEvent, LayerSize, LayerSpec, Rect, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("list.focus-out");
const OTHER: Id = Id::root("list.focus-out.other");
const TOOLTIP: Id = Id::root("list.focus-out.tooltip");
const ROWS: [&str; 20] = [
    "row-0", "row-1", "row-2", "row-3", "row-4", "row-5", "row-6", "row-7", "row-8", "row-9",
    "row-10", "row-11", "row-12", "row-13", "row-14", "row-15", "row-16", "row-17", "row-18",
    "row-19",
];

fn key(item: &&'static str) -> ItemKey {
    ItemKey::text(item)
}

fn list() -> List<'static, &'static str, fn(&&'static str) -> ItemKey> {
    List::new(LIST)
        .key(key as fn(&&'static str) -> ItemKey)
        .select_mode(SelectMode::Multi)
        .activate_focused_on_click(true)
}

struct Page {
    state: ListState,
    rows: Vec<&'static str>,
    mounted: bool,
    service_focus_out: bool,
    probe_before_normal_update: bool,
    focus_away_on_press: bool,
    remove_source_on_press: bool,
    state_before_press: Option<ListState>,
    press_changed_state: bool,
    focus_out_handled: usize,
    unsupported: usize,
    actions: Vec<ListAction>,
    open_tooltip_on_press: bool,
    probe_mixed_layer_settle: bool,
    mixed_batch_confirmed: bool,
    mixed_helper_returned_none: bool,
    mixed_state_unchanged_before_fallback: bool,
    mixed_layer_receipt_valid: bool,
    mixed_fallback_calls: usize,
    mixed_focusout_cleanup_changed_state: bool,
    mixed_actions_unchanged_after_fallback: bool,
}

impl Page {
    fn new() -> Self {
        Self {
            state: ListState::default(),
            rows: ROWS.to_vec(),
            mounted: true,
            service_focus_out: false,
            probe_before_normal_update: false,
            focus_away_on_press: false,
            remove_source_on_press: false,
            state_before_press: None,
            press_changed_state: false,
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
            mixed_focusout_cleanup_changed_state: false,
            mixed_actions_unchanged_after_fallback: false,
        }
    }

    fn update_list(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = list().update(cx, &mut self.state, &self.rows);
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
                match list().update_focus_out(cx, &mut self.state) {
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
            let handled = list().update_focus_out(cx, &mut self.state);
            self.mixed_helper_returned_none = handled.is_none();
            self.mixed_state_unchanged_before_fallback = self.state == before_fallback;
            if handled.is_none() {
                let actions_before_fallback = self.actions.clone();
                let receipt = cx.take_layer_intent_receipt(TOOLTIP);
                self.mixed_layer_receipt_valid = receipt.is_some_and(|receipt| {
                    receipt.event() == LayerEvent::Opened && receipt.opening().owner() == LIST
                });
                self.mixed_fallback_calls += 1;
                result |= self.update_list(cx);
                self.mixed_focusout_cleanup_changed_state = self.state != before_fallback;
                self.mixed_actions_unchanged_after_fallback =
                    self.actions == actions_before_fallback;
            }
        } else if self.service_focus_out && cx.update_cause() == UpdateCause::Settle {
            match list().update_focus_out(cx, &mut self.state) {
                Some(response) => {
                    self.focus_out_handled += 1;
                    result |= response;
                }
                None => result |= self.update_list(cx),
            }
        } else if self.probe_before_normal_update && cx.update_cause() == UpdateCause::Event {
            assert!(list().update_focus_out(cx, &mut self.state).is_none());
            result |= self.update_list(cx);
        } else {
            result |= self.update_list(cx);
        }

        if open_tooltip_after_press {
            self.open_tooltip_on_press = false;
            self.press_changed_state = self
                .state_before_press
                .as_ref()
                .is_some_and(|before| before != &self.state);
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
            self.press_changed_state = self
                .state_before_press
                .as_ref()
                .is_some_and(|before| before != &self.state);
            if self.remove_source_on_press {
                self.rows.clear();
                self.mounted = false;
            }
            cx.focus(OTHER);
        }
        result
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(OTHER, "Other").draw(ui, Rect::new(0, 0, 24, 1));
        list().draw(ui, Rect::new(0, 1, 24, 7), &self.state, &self.rows);
    }
}

fn harness() -> Harness<Page> {
    Harness::new(Page::new(), Theme::junie(), 24, 8)
}

#[test]
fn actual_focus_settle_clears_click_transient_without_reconciling_source() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    h.app_mut().state.choose(Some(ItemKey::text("row-0")));
    h.app_mut()
        .state
        .checked_mut()
        .insert(ItemKey::text("row-1"));
    assert!(h.app().state.scroll().offset() > 0);
    let before = h.app().state.clone();
    let cursor = before.cursor().expect("End selects a row");
    let area = h
        .area_of_part(LIST, PartRef::item(Part::ROW, cursor))
        .expect("last selected row is visible after End");
    h.app_mut().state_before_press = Some(before.clone());
    h.app_mut().focus_away_on_press = true;
    h.app_mut().remove_source_on_press = true;
    let _ = h.mouse(
        MouseKind::Down,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );

    assert_eq!(h.focus(), Some(OTHER));
    assert_eq!(
        h.app().state,
        before,
        "focus-only handling must preserve all ListState"
    );
    assert!(
        h.app().press_changed_state,
        "focused press arms private click state"
    );
    assert!(h.app().focus_out_handled > 0);
    assert_eq!(h.app().unsupported, 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    h.app_mut().mounted = true;
    let _ = h.mouse(
        MouseKind::Up,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    assert_eq!(h.app().state.cursor(), None);
    assert_eq!(h.app().state.chosen(), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn wrong_cause_falls_back_to_same_real_list_and_delivers_action_once() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    h.app_mut().probe_before_normal_update = true;

    let _ = h.key(KeyCode::Down);

    assert_eq!(h.app().actions, vec![ListAction::Moved]);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("row-1")));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn mixed_settle_layer_open_falls_back_after_acknowledged_focus_inspection() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    h.app_mut().state.choose(Some(ItemKey::text("row-0")));
    h.app_mut()
        .state
        .checked_mut()
        .insert(ItemKey::text("row-1"));
    let before_press = h.app().state.clone();
    let cursor = before_press.cursor().expect("End selects a row");
    let area = h
        .area_of_part(LIST, PartRef::item(Part::ROW, cursor))
        .expect("last selected row is visible after End");
    h.app_mut().state_before_press = Some(before_press);
    h.app_mut().open_tooltip_on_press = true;
    h.app_mut().probe_mixed_layer_settle = true;

    let _ = h.mouse(
        MouseKind::Down,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );

    assert_eq!(h.focus(), Some(OTHER));
    assert!(h.app().press_changed_state, "press arms the focused click");
    assert!(
        h.app().mixed_batch_confirmed,
        "Settle had FocusOut + Layer::Opened"
    );
    assert!(h.app().mixed_helper_returned_none);
    assert!(h.app().mixed_state_unchanged_before_fallback);
    assert!(h.app().mixed_layer_receipt_valid);
    assert_eq!(h.app().mixed_fallback_calls, 1);
    assert!(
        h.app().mixed_focusout_cleanup_changed_state,
        "ordinary List update clears its pre-armed click after unsupported inspection"
    );
    assert!(
        h.app().mixed_actions_unchanged_after_fallback,
        "same-pass normal fallback must not add a navigation action"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let _ = h.mouse(
        MouseKind::Up,
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    assert!(!matches!(
        h.app().actions.last(),
        Some(ListAction::Activated(_))
    ));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn focus_only_settle_clears_pending_focused_click_activation() {
    let mut h = harness();
    assert!(h.tab_to(LIST));
    h.app_mut().service_focus_out = true;
    h.app_mut().focus_away_on_press = true;

    let _ = h.mouse(MouseKind::Down, 10, 1);
    assert_eq!(h.focus(), Some(OTHER));
    assert!(h.app().focus_out_handled > 0);
    let _ = h.mouse(MouseKind::Up, 10, 1);

    assert!(!matches!(
        h.app().actions.last(),
        Some(ListAction::Activated(_))
    ));
    assert_eq!(h.app().unsupported, 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
