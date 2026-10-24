use core::{cell::Cell, fmt};

use ratatui_core::layout::Rect;
use termrock_core::{Axis, Id, ItemKey, KeyCode, Part, PartRef, Response, StateFlags};
use termrock_navigation::tabs::TabWindowReceipt;
use termrock_navigation::{TabMarks, Tabs, TabsAction, TabsState};
use termrock_runtime::keymap::KeyMap;
use termrock_runtime::{App, Cx, FrameRead, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TABS: Id = Id::root("tabs.builtin-overflow");
const MENU: Id = TABS.sub("overflow-menu");
const MENU_ITEMS: Id = MENU.sub("items");
const WIDTH: u16 = 32;
const HEIGHT: u16 = 16;

type TabKey = fn(&Tab) -> ItemKey;
type MarkFn = fn(&Tab) -> TabMarks;
type TabsWidget = Tabs<'static, Tab, TabKey>;

#[derive(Clone, Debug)]
struct Tab {
    id: u16,
    label: String,
    displays: Cell<usize>,
    marks: Cell<usize>,
}

impl fmt::Display for Tab {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.displays.set(self.displays.get().saturating_add(1));
        f.write_str(&self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(u64::from(tab.id))
}

fn tab_mark(tab: &Tab) -> TabMarks {
    tab.marks.set(tab.marks.get().saturating_add(1));
    TabMarks::default()
}

fn tabs(passive: bool) -> TabsWidget {
    Tabs::new(TABS)
        .key(tab_key as TabKey)
        .item_marks(tab_mark as MarkFn)
        .passive(passive)
}

struct Page {
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    keymap: KeyMap,
    passive: bool,
    disabled: bool,
}

impl Page {
    fn new() -> Self {
        let items = (1..=12)
            .map(|id| Tab {
                id,
                label: format!("Workspace-{id:02}-long"),
                displays: Cell::new(0),
                marks: Cell::new(0),
            })
            .collect();
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::num(1));
        Self {
            state,
            items,
            actions: Vec::new(),
            keymap: KeyMap::new(),
            passive: false,
            disabled: false,
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = tabs(self.passive).update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.disabled {
            ui.declare_state(TABS, StateFlags::DISABLED);
        }
        let area = ui.full().intersection(Rect::new(0, 0, WIDTH, 2));
        let widget = tabs(self.passive);
        let previous = ui.layout(TABS).map(|facts| TabWindowReceipt {
            viewport_len: facts.viewport_len,
            content_len: facts.content_len,
        });
        let layout = widget.layout_default(area, ui.full(), &self.state, &self.items, previous);
        assert!(widget.draw_layout(ui, &layout).is_ok());
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness() -> Harness<Page> {
    Harness::new(Page::new(), Theme::junie(), WIDTH, HEIGHT)
}

fn more_part() -> PartRef {
    PartRef::item(Part::OVERFLOW, ItemKey::index(2))
}

#[test]
fn more_click_opens_the_native_list_and_selection_activates_the_frozen_key_after_reorder() {
    let mut h = harness();
    let _ = h.tick();
    let more = h
        .area_of_part(TABS, more_part())
        .expect("the planned default row publishes its real More hit");
    assert_eq!(more.width, 4);
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    let hidden = h
        .app()
        .items
        .iter()
        .filter(|tab| {
            h.area_of_part(TABS, PartRef::item(Part::TAB, tab_key(tab)))
                .is_none()
        })
        .map(|tab| tab.id)
        .collect::<Vec<_>>();
    assert!(
        hidden.len() >= 3,
        "fixture has enough hidden stable keys: {hidden:?}"
    );
    let chosen = ItemKey::num(u64::from(hidden[1]));
    let other = hidden[2];

    let opened = h.click_part(TABS, more_part());
    assert!(opened.is_changed());
    assert!(h.is_open(MENU));
    assert_eq!(h.focus(), Some(MENU_ITEMS));
    assert!(
        h.area_of_part(MENU_ITEMS, PartRef::item(Part::ROW, chosen))
            .is_some()
    );

    let chosen_index = h
        .app()
        .items
        .iter()
        .position(|tab| tab_key(tab) == chosen)
        .expect("the frozen menu key still exists");
    let other_index = h
        .app()
        .items
        .iter()
        .position(|tab| tab.id == other)
        .expect("the other frozen menu key still exists");
    h.app_mut().items.swap(chosen_index, other_index);
    let _ = h.tick();
    assert!(
        h.is_open(MENU),
        "reordering keeps the open stable-key projection"
    );

    let activated = h.click_part(MENU_ITEMS, PartRef::item(Part::ROW, chosen));
    assert!(activated.is_changed());
    assert_eq!(
        h.app().actions.as_slice(),
        &[TabsAction::Activated(chosen)],
        "the menu row keeps its original stable key through reorder",
    );
    assert_eq!(h.app().state.active(), Some(chosen));
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, chosen))
            .is_some(),
        "choosing a hidden key reveals it in the newly planned strip"
    );
    assert!(!h.is_open(MENU));
    assert_eq!(h.focus(), Some(TABS));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn down_opens_more_and_escape_restores_focus_to_the_tabs_owner() {
    let mut h = harness();
    let _ = h.tick();
    assert!(h.tab_to(TABS));
    h.draw();
    let before = h
        .app()
        .items
        .iter()
        .map(|tab| (tab.displays.get(), tab.marks.get()))
        .collect::<Vec<_>>();

    let opened = h.key(KeyCode::Down);
    assert!(opened.is_changed());
    assert!(h.is_open(MENU));
    assert_eq!(h.focus(), Some(MENU_ITEMS));
    for (tab, (displays, marks)) in h.app().items.iter().zip(before) {
        assert_eq!(tab.displays.get(), displays.saturating_add(1));
        assert_eq!(tab.marks.get(), marks.saturating_add(1));
    }

    let _ = h.tick();
    assert!(h.is_open(MENU), "the Opened receipt preserves this opening");

    let dismissed = h.key(KeyCode::Esc);
    assert!(dismissed.is_consumed());
    assert!(!h.is_open(MENU));
    assert_eq!(h.focus(), Some(TABS));

    let reopened = h.click_part(TABS, more_part());
    assert!(reopened.is_changed());
    assert!(h.is_open(MENU));
    let outside = h.click(0, HEIGHT.saturating_sub(1));
    assert!(outside.is_consumed());
    assert!(
        !h.is_open(MENU),
        "an outside click dismisses the native menu"
    );
    assert_eq!(h.focus(), Some(TABS));
    assert!(h.app().actions.is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn removing_a_frozen_menu_key_closes_without_activating_a_replacement() {
    let mut h = harness();
    let _ = h.tick();
    let removed = h
        .app()
        .items
        .iter()
        .find(|tab| {
            h.area_of_part(TABS, PartRef::item(Part::TAB, tab_key(tab)))
                .is_none()
        })
        .map(|tab| tab.id)
        .expect("fixture has an unallocated hidden key");
    let _ = h.click_part(TABS, more_part());
    assert!(h.is_open(MENU));
    assert_eq!(h.focus(), Some(MENU_ITEMS));
    assert!(
        h.area_of_part(
            MENU_ITEMS,
            PartRef::item(Part::ROW, ItemKey::num(u64::from(removed))),
        )
        .is_some()
    );

    let index = h
        .app()
        .items
        .iter()
        .position(|tab| tab.id == removed)
        .expect("hidden key remains present before removal");
    h.app_mut().items.remove(index);
    let _ = h.tick();

    assert!(!h.is_open(MENU));
    assert!(h.app().actions.is_empty());
    assert_eq!(h.app().state.active(), Some(ItemKey::num(1)));
    assert!(
        h.area_of_part(
            MENU_ITEMS,
            PartRef::item(Part::ROW, ItemKey::num(u64::from(removed))),
        )
        .is_none()
    );
    assert_eq!(h.focus(), Some(TABS));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn disabled_and_passive_planned_tabs_cannot_open_the_hidden_menu() {
    let mut disabled = harness();
    let _ = disabled.tick();
    disabled.app_mut().disabled = true;
    disabled.draw();
    assert!(disabled.area_of_part(TABS, more_part()).is_some());
    let _ = disabled.click_part(TABS, more_part());
    assert!(!disabled.is_open(MENU));
    assert!(disabled.app().actions.is_empty());
    assert!(
        disabled.diagnostics().is_empty(),
        "{:?}",
        disabled.diagnostics()
    );

    let mut passive = harness();
    let _ = passive.tick();
    passive.app_mut().passive = true;
    passive.draw();
    assert!(passive.area_of_part(TABS, more_part()).is_some());
    let _ = passive.click_part(TABS, more_part());
    assert!(!passive.is_open(MENU));
    assert!(passive.app().actions.is_empty());
    assert!(
        passive.diagnostics().is_empty(),
        "{:?}",
        passive.diagnostics()
    );
}

#[test]
fn wheel_input_in_the_open_menu_does_not_move_the_tabs_window() {
    let mut h = harness();
    h.app_mut().items.extend((13_u16..=24).map(|id| Tab {
        id,
        label: format!("Workspace-{id:02}-long"),
        displays: Cell::new(0),
        marks: Cell::new(0),
    }));
    let _ = h.tick();
    let _ = h.click_part(TABS, more_part());
    assert!(h.is_open(MENU));
    let list_area = h
        .area_of(MENU_ITEMS)
        .expect("the native menu List owns a real control area");
    let first = h.app().state.first();
    let hidden_keys = h
        .app()
        .items
        .iter()
        .filter(|tab| {
            h.area_of_part(TABS, PartRef::item(Part::TAB, tab_key(tab)))
                .is_none()
        })
        .map(tab_key)
        .collect::<Vec<_>>();
    assert!(!hidden_keys.is_empty(), "fixture has hidden keys");
    assert!(
        hidden_keys.len() > usize::from(list_area.height),
        "hidden rows exceed the native menu viewport: hidden={}, viewport={}",
        hidden_keys.len(),
        list_area.height
    );
    let first_hidden = *hidden_keys.first().expect("hidden keys are nonempty");
    let last_hidden = *hidden_keys.last().expect("hidden keys are nonempty");
    assert!(
        h.area_of_part(TABS, PartRef::item(Part::TAB, last_hidden))
            .is_none()
    );
    assert!(
        h.area_of_part(MENU_ITEMS, PartRef::item(Part::ROW, last_hidden))
            .is_none()
    );
    let row_before = h
        .area_of_part(MENU_ITEMS, PartRef::item(Part::ROW, first_hidden))
        .expect("the first frozen menu row is visible before scrolling");
    let position = (
        list_area.x.saturating_add(1),
        list_area.y.saturating_add(list_area.height / 2),
    );

    let wheel = h.wheel(Axis::V, 1, position.0, position.1);

    assert!(wheel.is_consumed());
    assert!(h.is_open(MENU));
    assert_eq!(h.app().state.first(), first);
    assert!(h.app().actions.is_empty());
    assert_ne!(
        h.area_of_part(MENU_ITEMS, PartRef::item(Part::ROW, first_hidden)),
        Some(row_before),
        "the menu List consumes its wheel and advances its own row viewport",
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
