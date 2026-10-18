//! Nonsearchable picker preserves navigation and rejects invisible query edits.
use junie_tui::{
    ActivationKey, App, BlurPolicy, Cx, FilterPolicy, Id, Item, ItemKey, KeyCode, Picker,
    PickerAction, PickerState, Rect, Response, TextInput, TextInputState, Theme, TypingPolicy, Ui,
    UpdateCause,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("nonsearch.picker");
const BACKGROUND: Id = Id::root("nonsearch.background");
const STALE: ItemKey = ItemKey::num(1);
const FRESH: ItemKey = ItemKey::num(2);
const STALE_ITEM: Item<'static> = Item::new(STALE, "Stale");
const FRESH_ITEM: Item<'static> = Item::new(FRESH, "Fresh");
const ITEMS: &[Item<'static>] = &[STALE_ITEM, FRESH_ITEM];
struct Page {
    state: PickerState,
    searchable: bool,
    toggle_on_enter: bool,
    caller: bool,
    items: Vec<Item<'static>>,
    chosen: Option<ItemKey>,
    changes: usize,
    background: TextInputState,
    value: String,
}
impl Page {
    fn picker(&self) -> Picker<'static, Item<'static>> {
        Picker::new(ID)
            .title("Actions")
            .width(64)
            .searchable(self.searchable)
            .filter(if self.caller {
                FilterPolicy::Caller
            } else {
                FilterPolicy::Label
            })
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.background.begin("");
            cx.open_layer(ID, self.picker().layer(cx, &self.items));
        }
        let _ = TextInput::new(BACKGROUND).blur(BlurPolicy::Keep).update(
            cx,
            &mut self.background,
            &mut self.value,
        );
        if self.toggle_on_enter && cx.activation_key() == Some(ActivationKey::Enter) {
            self.toggle_on_enter = false;
            self.searchable = false;
        }
        let props = self.picker();
        let mut response = props.update(cx, &mut self.state, &self.items);
        match response.take_action() {
            Some(PickerAction::Chosen(key)) => self.chosen = Some(key),
            Some(PickerAction::QueryChanged) => {
                self.changes = self.changes.saturating_add(1);
                if self.caller {
                    self.items = vec![FRESH_ITEM];
                    assert!(!props.reconcile(&mut self.state, &self.items));
                }
            }
            _ => {}
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(BACKGROUND)
            .value(&self.value)
            .blur(BlurPolicy::Keep)
            .typing_policy(TypingPolicy::Fallback { cursor: true })
            .draw(ui, Rect::new(0, 0, 30, 1), &self.background);
        ui.layer(ID, |ui, area| {
            self.picker().draw(ui, area, &self.state, &self.items)
        });
    }
}
fn page(searchable: bool, toggle: bool, caller: bool) -> Harness<Page> {
    let mut state = PickerState::default();
    if toggle {
        state.set_query("Stale");
    }
    Harness::new(
        Page {
            state,
            searchable,
            toggle_on_enter: toggle,
            caller,
            items: if caller {
                vec![STALE_ITEM]
            } else {
                ITEMS.to_vec()
            },
            chosen: None,
            changes: 0,
            background: TextInputState::default(),
            value: String::new(),
        },
        Theme::junie(),
        120,
        30,
    )
}
#[test]
fn nonsearch_geometry_no_cursor_and_modal_query_barrier() {
    let mut h = page(false, false, false);
    assert!(!h.text().contains("Type to search…"));
    assert_eq!(h.cursor(), None);
    let area = h.layer_area(ID);
    assert_eq!(area.map(|area| area.height), Some(7));
    if let Some(area) = area {
        let text = h.text();
        assert!(
            text.lines()
                .nth(usize::from(area.y + 2))
                .is_some_and(|line| line.contains("Stale"))
        );
    }
    let _ = h.type_str("hidden");
    let _ = h.paste("query");
    assert_eq!(h.app().state.query(), "");
    assert_eq!(h.app().background.draft_text(), Some(""));
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, Some(FRESH));
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.layer_area(ID), None);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn toggle_same_enter_refreshes_caller_projection_before_activation() {
    let mut h = page(true, true, true);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, None);
    assert_eq!(h.app().changes, 1);
    assert_eq!(h.app().state.query(), "");
    assert_eq!(h.app().state.cursor(), Some(FRESH));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().chosen, Some(FRESH));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn pure_reconcile_reports_clear_and_toggle_back_stays_empty() {
    let mut state = PickerState::default();
    state.set_query("Stale");
    let picker = Picker::new(ID)
        .searchable(false)
        .filter(FilterPolicy::Caller);
    assert!(picker.reconcile(&mut state, ITEMS));
    assert_eq!(state.query(), "");
    assert!(!picker.reconcile(&mut state, ITEMS));
    assert!(!Picker::new(ID).reconcile(&mut state, ITEMS));
    assert_eq!(state.query(), "");
}
#[test]
fn default_picker_still_accepts_query_and_paste() {
    let mut h = page(true, false, false);
    assert!(h.text().contains("Type to search…"));
    let _ = h.type_str("Sta");
    let _ = h.paste("le");
    assert_eq!(h.app().state.query(), "Stale");
    assert_eq!(h.app().state.cursor(), Some(STALE));
}

const SCOPES: &[junie_tui::ScopeKey] = &[junie_tui::ScopeKey::new(0), junie_tui::ScopeKey::new(1)];
struct NavigationPage {
    state: PickerState,
    searchable: bool,
    toggle: std::rc::Rc<std::cell::Cell<bool>>,
    actions: Vec<(PickerAction, String)>,
}
impl NavigationPage {
    fn picker(&self) -> Picker<'static, Item<'static>> {
        Picker::new(ID).scopes(SCOPES).searchable(self.searchable)
    }
}
impl App for NavigationPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.open_layer(ID, self.picker().layer(cx, ITEMS));
        }
        if cx.update_cause() == UpdateCause::Event && self.toggle.replace(false) {
            self.searchable = false;
        }
        let mut response = self.picker().update(cx, &mut self.state, ITEMS);
        if let Some(action) = response.take_action() {
            self.actions.push((action, self.state.query().to_owned()));
        }
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(ID, |ui, area| {
            self.picker().draw(ui, area, &self.state, ITEMS)
        });
    }
}
fn navigation() -> (Harness<NavigationPage>, std::rc::Rc<std::cell::Cell<bool>>) {
    let toggle = std::rc::Rc::new(std::cell::Cell::new(false));
    let mut state = PickerState::default();
    state.set_query("Stale");
    let h = Harness::new(
        NavigationPage {
            state,
            searchable: true,
            toggle: toggle.clone(),
            actions: Vec::new(),
        },
        Theme::junie(),
        80,
        24,
    );
    (h, toggle)
}
#[test]
fn same_escape_on_nonsearch_transition_dismisses_and_reports_clear() {
    let (mut h, toggle) = navigation();
    toggle.set(true);
    let _ = h.key(KeyCode::Esc);
    assert_eq!(h.layer_area(ID), None);
    assert!(
        h.app()
            .actions
            .contains(&(PickerAction::QueryChanged, String::new()))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn same_back_on_nonsearch_transition_preserves_navigation_with_empty_query() {
    let (mut h, toggle) = navigation();
    toggle.set(true);
    let _ = h.key(KeyCode::Backspace);
    assert!(
        h.app()
            .actions
            .contains(&(PickerAction::Back, String::new()))
    );
    assert!(h.layer_area(ID).is_some());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn same_scope_on_nonsearch_transition_advances_scope_with_empty_query() {
    let (mut h, toggle) = navigation();
    toggle.set(true);
    let _ = h.key(KeyCode::Tab);
    assert_eq!(
        h.app().state.scope(SCOPES),
        Some(junie_tui::ScopeKey::new(1))
    );
    assert!(h.app().actions.contains(&(
        PickerAction::Scope(junie_tui::ScopeKey::new(1)),
        String::new()
    )));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
