use termrock_core::event::{KeyCode, KeyModifiers};
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::Response;
use termrock_navigation::{FilterPolicy, SelectionPolicy};
use termrock_overlays::picker::{Item, Picker, PickerAction, PickerState, ScopeKey};
use termrock_runtime::{App, Cx, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;
const SCOPES: &[ScopeKey] = &[
    ScopeKey::new(0),
    ScopeKey::new(1),
    ScopeKey::new(2),
    ScopeKey::new(3),
];
const ID: Id = Id::root("picker.projection");
const ITEMS: &[Item<'static>; 2] = &[
    Item::new(ItemKey::num(10), "orders").detail("public.orders"),
    Item::new(ItemKey::num(20), "customers").detail("public.customers"),
];
fn picker(policy: FilterPolicy) -> Picker<'static, Item<'static>> {
    Picker::new(ID).width(64).filter(policy).scopes(SCOPES)
}
struct Page {
    state: PickerState,
    items: Vec<Item<'static>>,
    policy: FilterPolicy,
    selection_policy: SelectionPolicy,
    max_rows: Option<u16>,
    chosen: Vec<ItemKey>,
    query_events: usize,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut component = picker(self.policy).selection_policy(self.selection_policy);
        if let Some(rows) = self.max_rows {
            component = component.max_rows(rows);
        }
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(ID, component.layer(cx, &self.items));
        }
        let mut response = component.update(cx, &mut self.state, &self.items);
        match response.take_action() {
            Some(PickerAction::QueryChanged) => {
                self.query_events = self.query_events.saturating_add(1);
                self.items = ITEMS
                    .iter()
                    .copied()
                    .filter(|item| item.detail.contains(self.state.query()))
                    .rev()
                    .collect();
                component.reconcile(&mut self.state, &self.items);
            }
            Some(PickerAction::Scope(_)) => {
                self.items.reverse();
                component.reconcile(&mut self.state, &self.items);
            }
            Some(PickerAction::Chosen(key)) => self.chosen.push(key),
            _ => {}
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ID, |ui, area| {
            let mut component = picker(self.policy).selection_policy(self.selection_policy);
            if let Some(rows) = self.max_rows {
                component = component.max_rows(rows);
            }
            component.draw(ui, area, &self.state, &self.items)
        });
    }
}
fn page(policy: FilterPolicy) -> Harness<Page> {
    page_with_selection(policy, SelectionPolicy::Preserve)
}
fn page_with_selection(policy: FilterPolicy, selection_policy: SelectionPolicy) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            items: ITEMS.to_vec(),
            policy,
            selection_policy,
            max_rows: None,
            chosen: vec![],
            query_events: 0,
        },
        Theme::junie(),
        120,
        20,
    )
}
#[test]
fn caller_detail_matches_survive_query_recompute_without_duplicate_input() {
    let mut h = page(FilterPolicy::Caller);
    for ch in "public".chars() {
        let _ = h.key(KeyCode::Char(ch));
    }
    assert_eq!(h.app().state.query(), "public");
    assert_eq!(h.app().query_events, 6);
    assert!(h.text().contains("orders"));
    assert!(h.text().contains("customers"));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(10)));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, [ItemKey::num(10)]);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.app().state.query(), "");
    assert!(h.layer_area(ID).is_some());
    let _ = h.key(KeyCode::Esc);
    assert!(h.layer_area(ID).is_none());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn keyed_reconcile_preserves_reordered_selection_and_removes_missing_selection() {
    let component = picker(FilterPolicy::Caller);
    let mut state = PickerState::default();
    let [first, second] = ITEMS;
    component.reconcile(&mut state, ITEMS);
    assert_eq!(state.cursor(), Some(ItemKey::num(10)));
    component.reconcile(&mut state, &[*second, *first]);
    assert_eq!(state.cursor(), Some(ItemKey::num(10)));
    component.reconcile(&mut state, &[*second]);
    assert_eq!(state.cursor(), Some(ItemKey::num(20)));
    component.reconcile(&mut state, &[]);
    assert_eq!(state.cursor(), None);
    component.reconcile(&mut state, &[first.disabled(true), *second]);
    assert_eq!(state.cursor(), Some(ItemKey::num(20)));
    assert_eq!(state.query(), "");
}
#[test]
fn label_default_still_refuses_detail_only_matches() {
    let mut h = page(FilterPolicy::Label);
    for ch in "public".chars() {
        let _ = h.key(KeyCode::Char(ch));
    }
    assert_eq!(h.app().state.cursor(), None);
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().chosen.is_empty());
    assert_eq!(h.layer_area(ID).map(|a| a.width), Some(64));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn query_refresh_starts_at_first_match_but_reorder_preserves_key() {
    let mut h = page_with_selection(
        FilterPolicy::Caller,
        SelectionPolicy::FirstEnabledOnFilterChange,
    );
    h.app_mut().state.set_cursor(0, ItemKey::num(10));
    let _ = h.type_str("public");
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(20)));

    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(10)));
    h.app_mut().items.reverse();
    let component =
        picker(FilterPolicy::Caller).selection_policy(SelectionPolicy::FirstEnabledOnFilterChange);
    let items = h.app().items.clone();
    component.reconcile(&mut h.app_mut().state, &items);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(10)));
}

#[test]
fn back_tab_wraps_scopes_and_resets_to_first_enabled_match() {
    let mut h = page_with_selection(
        FilterPolicy::Caller,
        SelectionPolicy::FirstEnabledOnFilterChange,
    );
    h.app_mut().state.set_cursor(0, ItemKey::num(10));
    let _ = h.type_str("public");
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(20)));
    assert_eq!(h.app().state.query(), "public");

    let _ = h.key(KeyCode::BackTab);
    assert_eq!(h.app().state.scope(SCOPES), Some(ScopeKey::new(3)));
    assert_eq!(h.app().state.query(), "public");
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(10)));

    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.app().state.scope(SCOPES), Some(ScopeKey::new(0)));
    assert_eq!(h.app().state.query(), "public");
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(20)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn shifted_back_tab_is_routed_to_previous_picker_scope() {
    let mut h = page_with_selection(
        FilterPolicy::Caller,
        SelectionPolicy::FirstEnabledOnFilterChange,
    );
    let _ = h.type_str("public");
    assert_eq!(h.app().state.query(), "public");

    let _ = h.key_mod(KeyCode::BackTab, KeyModifiers::SHIFT);
    assert_eq!(h.app().state.scope(SCOPES), Some(ScopeKey::new(3)));
    assert_eq!(h.app().state.query(), "public");
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(10)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn scope_reorder_and_empty_projection_settle_before_next_activation() {
    let mut h = page(FilterPolicy::Caller);
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(20)));
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::num(20)));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, [ItemKey::num(20)]);
    let _ = h.key(KeyCode::Char('z'));
    assert_eq!(h.app().state.cursor(), None);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, [ItemKey::num(20)]);
    let _ = h.key(KeyCode::Esc);
    assert!(h.app().state.cursor().is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn max_rows_caps_the_requested_layer_height() {
    let harness = Harness::new(
        Page {
            state: PickerState::default(),
            items: ITEMS.to_vec(),
            policy: FilterPolicy::Label,
            selection_policy: SelectionPolicy::Preserve,
            max_rows: Some(1),
            chosen: vec![],
            query_events: 0,
        },
        Theme::junie(),
        120,
        20,
    );
    let area = harness.layer_area(ID).expect("picker layer is open");
    assert_eq!(area.height, 6, "one row plus the five card rows");
}
