//! Caller projections retain shared input handling and semantic cursor identity.
use junie_tui::{
    App, Cx, FilterPolicy, Id, Item, ItemKey, KeyCode, Picker, PickerAction, PickerState, Response,
    ScopeKey, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;
const SCOPES: &[ScopeKey] = &[ScopeKey::new(0), ScopeKey::new(1)];
const ID: Id = Id::root("picker.projection");
const ITEMS: &[Item<'static>; 2] = &[
    Item::new(ItemKey::num(10), "orders").detail("public.orders"),
    Item::new(ItemKey::num(20), "customers").detail("public.customers"),
];
fn picker(policy: FilterPolicy) -> Picker<'static, Item<'static>> {
    Picker::new(ID)
        .width(64)
        .filter(policy)
        .scopes(SCOPES)
}
struct Page {
    state: PickerState,
    items: Vec<Item<'static>>,
    policy: FilterPolicy,
    chosen: Vec<ItemKey>,
    query_events: usize,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let component = picker(self.policy);
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
            picker(self.policy).draw(ui, area, &self.state, &self.items)
        });
    }
}
fn page(policy: FilterPolicy) -> Harness<Page> {
    Harness::new(
        Page {
            state: PickerState::default(),
            items: ITEMS.to_vec(),
            policy,
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
