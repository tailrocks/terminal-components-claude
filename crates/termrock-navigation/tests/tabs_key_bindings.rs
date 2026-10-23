#![allow(clippy::unwrap_used, reason = "real Tabs binding contract assertions")]

use core::cell::{Cell, RefCell};

use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::{Color, Style};
use termrock_controls::Button;
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyModifiers, RoutedInput};
use termrock_core::{Id, ItemKey, KeyCode, Part, PartRef, Response};
use termrock_fields::{TextInput, TextInputState};
use termrock_navigation::{
    TabCloseReveal, Tabs, TabsAction, TabsCmd, TabsKeyBindingsError, TabsState,
};
use termrock_runtime::keymap::{Binding, BindingState, Bindings, Hint, HintKey, KeyMap};
use termrock_runtime::{App, Cx, Dismiss, LayerSize, LayerSpec, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TABS: Id = Id::root("tabs.key-bindings");
const HOST: Id = Id::root("tabs.key-bindings.host");
const MODAL: Id = Id::root("tabs.key-bindings.modal");
const AREA: Rect = Rect::new(0, 0, 40, 2);
const HOST_AREA: Rect = Rect::new(0, 4, 32, 3);
const FIRST_TAB_ONLY: [ItemKey; 1] = [ItemKey::Num(1)];

const fn binding(
    action: &'static str,
    chord: Option<Chord>,
    cmd: TabsCmd,
    label: &'static str,
    priority: u8,
    visible: bool,
) -> Binding<TabsCmd> {
    Binding {
        action: ActionKey::custom(action),
        chord,
        cmd,
        label,
        priority,
        visible,
    }
}

const NEW_T: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new",
    Some(Chord::key(KeyCode::Char('t'))),
    TabsCmd::New,
    "New tab",
    77,
    true,
)];

const NEW_PROFILE: [Binding<TabsCmd>; 4] = [
    binding(
        "tabs.new",
        Some(Chord::key(KeyCode::Char('t'))),
        TabsCmd::New,
        "New tab",
        77,
        true,
    ),
    binding(
        "tabs.new.template",
        Some(Chord::with(KeyCode::Char('n'), KeyModifiers::CONTROL)),
        TabsCmd::New,
        "New from template",
        66,
        true,
    ),
    binding(
        "tabs.new.hidden",
        Some(Chord::with(KeyCode::Char('n'), KeyModifiers::ALT)),
        TabsCmd::New,
        "Hidden new alias",
        99,
        false,
    ),
    binding(
        "tabs.new.palette",
        None,
        TabsCmd::New,
        "New from palette",
        40,
        true,
    ),
];

const CLOSE_DELETE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close",
    Some(Chord::key(KeyCode::Delete)),
    TabsCmd::Close,
    "Close tab",
    73,
    true,
)];

const CLOSE_PROFILE: [Binding<TabsCmd>; 2] = [
    binding(
        "tabs.close",
        Some(Chord::key(KeyCode::Delete)),
        TabsCmd::Close,
        "Close tab",
        73,
        true,
    ),
    binding(
        "tabs.close.window",
        Some(Chord::with(KeyCode::Char('w'), KeyModifiers::CONTROL)),
        TabsCmd::Close,
        "Close window",
        61,
        true,
    ),
];

const BAD_NEW_COMMAND: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.invalid-command",
    Some(Chord::key(KeyCode::Char('i'))),
    TabsCmd::Close,
    "Invalid New",
    1,
    true,
)];
const BAD_CLOSE_COMMAND: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.invalid-command",
    Some(Chord::key(KeyCode::Char('i'))),
    TabsCmd::New,
    "Invalid Close",
    1,
    true,
)];
const DUPLICATE_NEW_ACTION: [Binding<TabsCmd>; 2] = [
    binding(
        "tabs.new.duplicate",
        Some(Chord::key(KeyCode::Char('i'))),
        TabsCmd::New,
        "New A",
        1,
        true,
    ),
    binding(
        "tabs.new.duplicate",
        Some(Chord::key(KeyCode::Char('j'))),
        TabsCmd::New,
        "New B",
        1,
        true,
    ),
];
const CONFLICTS_WITH_BASE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.right",
    Some(Chord::key(KeyCode::Right)),
    TabsCmd::New,
    "New on Right",
    1,
    true,
)];
const CONFLICTING_SHIFTED_ALIASES: [Binding<TabsCmd>; 2] = [
    binding(
        "tabs.new.q",
        Some(Chord::key(KeyCode::Char('q'))),
        TabsCmd::New,
        "New q",
        1,
        true,
    ),
    binding(
        "tabs.new.shift-q",
        Some(Chord::with(KeyCode::Char('q'), KeyModifiers::SHIFT)),
        TabsCmd::New,
        "New shifted q",
        1,
        true,
    ),
];
const CROSS_GROUP_DUPLICATE_ACTION: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new",
    Some(Chord::key(KeyCode::Delete)),
    TabsCmd::Close,
    "Close with New action identity",
    1,
    true,
)];
const CROSS_GROUP_CHORD: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.t",
    Some(Chord::key(KeyCode::Char('t'))),
    TabsCmd::Close,
    "Close on t",
    1,
    true,
)];
const CLOSE_ON_RIGHT: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.right",
    Some(Chord::key(KeyCode::Right)),
    TabsCmd::Close,
    "Close on Right",
    1,
    true,
)];

#[derive(Clone, Copy, Debug)]
struct Config {
    configured: bool,
    new_keys: Option<&'static [Binding<TabsCmd>]>,
    close_keys: Option<&'static [Binding<TabsCmd>]>,
    allow_new: bool,
    closable: bool,
    closable_keys: Option<&'static [ItemKey]>,
    passive: bool,
    host_input: bool,
    modal: bool,
    reference_tabs: bool,
    group_order: GroupOrder,
}

#[derive(Clone, Copy, Debug)]
enum GroupOrder {
    BeforeKeyAndRow,
    AfterKeyAndRow,
    RowThenKey,
    CloseFirst,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            configured: false,
            new_keys: None,
            close_keys: None,
            allow_new: true,
            closable: true,
            closable_keys: None,
            passive: false,
            host_input: false,
            modal: false,
            reference_tabs: false,
            group_order: GroupOrder::AfterKeyAndRow,
        }
    }
}

#[derive(Clone, Copy)]
struct PlanCase {
    config: Config,
    empty_area: bool,
}

struct Tab {
    id: u64,
    label: &'static str,
}

impl core::fmt::Display for Tab {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.id)
}

fn tab_row(tab: &Tab, row: &mut termrock_runtime::RowUi<'_>) {
    row.label(tab.label);
}

type TabsWidget =
    Tabs<'static, Tab, fn(&Tab) -> ItemKey, for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)>;

struct Page {
    config: Config,
    next_config: Option<Config>,
    plan: Option<PlanCase>,
    plan_rejected: Cell<bool>,
    sentinel: bool,
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    keymap: KeyMap,
    hints: RefCell<Vec<Hint>>,
    host_state: TextInputState,
    host_value: String,
}

impl Page {
    fn new(config: Config) -> Self {
        Self {
            config,
            next_config: None,
            plan: None,
            plan_rejected: Cell::new(false),
            sentinel: false,
            state: TabsState::default(),
            items: vec![
                Tab {
                    id: 1,
                    label: "Alpha",
                },
                Tab {
                    id: 2,
                    label: "Beta",
                },
            ],
            actions: Vec::new(),
            keymap: KeyMap::new(),
            hints: RefCell::new(Vec::new()),
            host_state: TextInputState::default(),
            host_value: String::new(),
        }
    }

    fn with_groups<K, R>(
        mut tabs: Tabs<'static, Tab, K, R>,
        config: Config,
    ) -> Tabs<'static, Tab, K, R> {
        if !config.configured {
            return tabs;
        }
        tabs = match config.group_order {
            GroupOrder::CloseFirst => tabs
                .try_close_key_bindings(config.close_keys)
                .expect("test Close table is valid")
                .try_new_key_bindings(config.new_keys)
                .expect("test New table is valid"),
            _ => tabs
                .try_new_key_bindings(config.new_keys)
                .expect("test New table is valid")
                .try_close_key_bindings(config.close_keys)
                .expect("test Close table is valid"),
        };
        tabs
    }

    fn widget(config: Config) -> TabsWidget {
        let mut base = Tabs::new(TABS)
            .allow_new(config.allow_new)
            .closable(config.closable)
            .close_reveal(TabCloseReveal::Always);
        if let Some(keys) = config.closable_keys {
            base = base.closable_keys(keys);
        }
        if config.passive {
            base = base.passive(true);
        }
        match config.group_order {
            GroupOrder::BeforeKeyAndRow => Self::with_groups(base, config)
                .key(tab_key as fn(&Tab) -> ItemKey)
                .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)),
            GroupOrder::AfterKeyAndRow | GroupOrder::CloseFirst => Self::with_groups(
                base.key(tab_key as fn(&Tab) -> ItemKey)
                    .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)),
                config,
            ),
            GroupOrder::RowThenKey => Self::with_groups(
                base.row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>))
                    .key(tab_key as fn(&Tab) -> ItemKey),
                config,
            ),
        }
    }

    fn default_row_widget(config: Config) -> Tabs<'static, Tab, fn(&Tab) -> ItemKey> {
        let mut base = Tabs::new(TABS)
            .allow_new(config.allow_new)
            .closable(config.closable);
        if let Some(keys) = config.closable_keys {
            base = base.closable_keys(keys);
        }
        if config.passive {
            base = base.passive(true);
        }
        match config.group_order {
            GroupOrder::BeforeKeyAndRow => {
                Self::with_groups(base, config).key(tab_key as fn(&Tab) -> ItemKey)
            }
            GroupOrder::AfterKeyAndRow | GroupOrder::CloseFirst | GroupOrder::RowThenKey => {
                Self::with_groups(base.key(tab_key as fn(&Tab) -> ItemKey), config)
            }
        }
    }

    fn area(&self) -> Rect {
        AREA
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(config) = self.next_config.take() {
            self.config = config;
        }
        if self.config.modal && !cx.is_open(MODAL) {
            cx.open_layer(
                MODAL,
                LayerSpec::modal(MODAL)
                    .size(LayerSize::Fixed(16, 3))
                    .dismiss(Dismiss::NONE)
                    .initial_focus(MODAL),
            );
        }
        if cx.update_cause() == UpdateCause::Bootstrap {
            self.host_state.begin(&self.host_value);
        }

        let tabs = Self::widget(self.config);
        let mut response = tabs.update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        let mut response = response.erase();
        if self.config.host_input {
            response |= TextInput::new(HOST)
                .update(cx, &mut self.host_state, &mut self.host_value)
                .erase();
        }
        if cx.is_open(MODAL) {
            response |= Button::new(MODAL, "Modal").update(cx).erase();
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.sentinel {
            ui.fill(ui.full(), Style::default().bg(Color::Blue));
        }
        let tabs = Self::widget(self.config);
        if let Some(plan) = self.plan {
            let planned = Self::default_row_widget(plan.config);
            let actual = Self::default_row_widget(self.config);
            let area = if plan.empty_area {
                Rect::new(AREA.x, AREA.y, 0, 0)
            } else {
                self.area()
            };
            let layout = planned.layout_default(area, area, &self.state, &self.items, None);
            self.plan_rejected
                .set(actual.draw_layout(ui, &layout).is_err());
        } else if self.config.reference_tabs {
            ui.reference(None, |reference_ui| {
                tabs.draw(reference_ui, self.area(), &self.state, &self.items)
            });
        } else {
            tabs.draw(ui, self.area(), &self.state, &self.items);
        }

        if self.config.host_input {
            TextInput::new(HOST)
                .value(&self.host_value)
                .draw(ui, HOST_AREA, &self.host_state);
        }
        if self.config.modal {
            ui.layer(MODAL, |layer, area| {
                Button::new(MODAL, "Modal").draw(layer, area);
            });
        }
        let hints = ui.with_focused_hints(|_, layer| layer.hints.clone());
        self.hints.replace(hints.unwrap_or_default());
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: Page, width: u16) -> Harness<Page> {
    Harness::new(page, Theme::junie(), width, 8)
}

fn hint(page: &Page, label: &'static str, chord: Chord, priority: u8) -> bool {
    page.hints.borrow().contains(&Hint {
        key: HintKey::Chord(chord),
        label,
        priority,
    })
}

fn action_new(h: &Harness<Page>) {
    assert_eq!(h.app().actions.last(), Some(&TabsAction::New));
}

fn middle(h: &mut Harness<Page>, part: PartRef) {
    let area = h
        .area_of_part(TABS, part)
        .expect("the expected tab part has a live hit target");
    let input = RoutedInput::MiddleClick {
        pos: Position::new(area.x.saturating_add(area.width / 2), area.y),
        mods: KeyModifiers::NONE,
    };
    if let Err(pending) = h.runtime_mut().handle_routed(input) {
        let input = pending.into_routed_input();
        h.draw();
        let _ = h
            .runtime_mut()
            .handle_routed(input)
            .expect("runtime is ready after its scheduled presentation");
    }
    h.draw();
}

#[test]
fn absent_and_explicit_none_keep_the_current_default_table_and_cells_exact() {
    let defaults = Config::default();
    let implicit = harness(Page::new(defaults), 48);
    let explicit = harness(
        Page::new(Config {
            configured: true,
            ..defaults
        }),
        48,
    );

    assert_eq!(implicit.buffer(), explicit.buffer());
    assert_eq!(implicit.ring().entries(), explicit.ring().entries());
    assert_eq!(implicit.focus(), explicit.focus());
    assert_eq!(implicit.app().state, explicit.app().state);
    assert_eq!(implicit.app().actions, explicit.app().actions);
    assert_eq!(
        *implicit.app().hints.borrow(),
        *explicit.app().hints.borrow()
    );
    assert_eq!(
        Page::widget(defaults).bindings(BindingState::default()),
        Page::widget(Config {
            configured: true,
            ..defaults
        })
        .bindings(BindingState::default())
    );
    assert_eq!(Page::widget(defaults).new_key_table(), &[]);
    assert_eq!(Page::widget(defaults).close_key_table(), &[]);

    let table = Page::widget(defaults).bindings(BindingState::default());
    assert_eq!(
        table.len(),
        18,
        "the default FULL static table remains intact"
    );
    assert_eq!(
        Binding::command(table, ActionKey::custom("tabs.new")),
        Some(TabsCmd::New)
    );
    assert_eq!(
        Binding::command(table, ActionKey::custom("tabs.close")),
        Some(TabsCmd::Close)
    );
    for (action, command) in [
        ("tabs.previous", TabsCmd::Prev),
        ("tabs.previous-vim", TabsCmd::Prev),
        ("tabs.next", TabsCmd::Next),
        ("tabs.next-vim", TabsCmd::Next),
        ("tabs.activate", TabsCmd::Activate),
        ("tabs.activate-space", TabsCmd::Activate),
        ("tabs.tab-1", TabsCmd::Nth(1)),
        ("tabs.tab-9", TabsCmd::Nth(9)),
    ] {
        assert_eq!(
            Binding::command(table, ActionKey::custom(action)),
            Some(command),
            "current default action {action} stays in the base table"
        );
    }

    let mut h = harness(Page::new(defaults), 48);
    assert_eq!(h.focus(), Some(TABS));
    let _ = h.key(KeyCode::Right);
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(2)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('h'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char(' '));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('2'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(2)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('n'));
    action_new(&h);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('x'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Close(ItemKey::Num(2)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Delete);
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Close(ItemKey::Num(2)))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn custom_groups_replace_defaults_and_keep_metadata_remaps_and_dispatch_aligned() {
    let config = Config {
        configured: true,
        new_keys: Some(&NEW_PROFILE),
        close_keys: Some(&CLOSE_PROFILE),
        ..Config::default()
    };
    let tabs = Page::widget(config);
    assert_eq!(tabs.new_key_table(), &NEW_PROFILE);
    assert_eq!(tabs.close_key_table(), &CLOSE_PROFILE);
    let base = tabs.bindings(BindingState::default());
    assert!(Binding::command(base, ActionKey::custom("tabs.new")).is_none());
    assert!(Binding::command(base, ActionKey::custom("tabs.close")).is_none());
    assert!(Binding::command(base, ActionKey::custom("tabs.close-delete")).is_none());

    let mut page = Page::new(config);
    page.keymap = KeyMap::new().bind_component(
        TABS,
        ActionKey::custom("tabs.new.palette"),
        Chord::key(KeyCode::F(10)),
    );
    let mut h = harness(page, 48);
    assert!(hint(h.app(), "New tab", Chord::key(KeyCode::Char('t')), 77));
    assert!(hint(
        h.app(),
        "New from template",
        Chord::with(KeyCode::Char('n'), KeyModifiers::CONTROL),
        66
    ));
    assert!(hint(
        h.app(),
        "New from palette",
        Chord::key(KeyCode::F(10)),
        40
    ));
    assert!(hint(h.app(), "Close tab", Chord::key(KeyCode::Delete), 73));
    assert!(hint(
        h.app(),
        "Close window",
        Chord::with(KeyCode::Char('w'), KeyModifiers::CONTROL),
        61
    ));
    assert!(
        !h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "Hidden new alias")
    );

    let _ = h.key(KeyCode::Char('n'));
    assert!(
        h.app().actions.is_empty(),
        "the replaced default n is absent"
    );
    let _ = h.key(KeyCode::Char('t'));
    action_new(&h);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::F(10));
    action_new(&h);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Delete);
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(1))]
    );
    assert_eq!(
        h.app().items.len(),
        2,
        "the host still owns close responses"
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('x'));
    assert!(
        h.app().actions.is_empty(),
        "the replaced default x is absent"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn empty_groups_disable_only_keyboard_and_preserve_pointer_middle_and_geometry() {
    let config = Config {
        configured: true,
        new_keys: Some(&[]),
        close_keys: Some(&[]),
        ..Config::default()
    };
    let mut h = harness(Page::new(config), 14);
    let tabs = Page::widget(config);
    assert!(tabs.new_key_table().is_empty());
    assert!(tabs.close_key_table().is_empty());
    let base = tabs.bindings(BindingState::default());
    assert!(Binding::command(base, ActionKey::custom("tabs.new")).is_none());
    assert!(Binding::command(base, ActionKey::custom("tabs.close")).is_none());
    assert!(
        h.app().hints.borrow().iter().all(|hint| {
            hint.label != "New" && hint.label != "Close" && hint.label != "New tab"
        })
    );
    let _ = h.key(KeyCode::Char('n'));
    let _ = h.key(KeyCode::Char('x'));
    let _ = h.key(KeyCode::Delete);
    let _ = h.key(KeyCode::Char('t'));
    assert!(h.app().actions.is_empty());

    let mut pointer_h = harness(Page::new(config), 48);
    let pointer_new = pointer_h
        .area_of_part(TABS, PartRef::of(Part::NEW))
        .expect("keyboard override does not remove the pointer New part");
    let _ = pointer_h.click(pointer_new.x, pointer_new.y);
    assert_eq!(pointer_h.app().actions.as_slice(), [TabsAction::New]);
    pointer_h.app_mut().actions.clear();
    let close = pointer_h
        .area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::Num(1)))
        .expect("keyboard override does not remove the eligible close hit");
    let _ = pointer_h.click(close.x, close.y);
    assert_eq!(
        pointer_h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(1))]
    );
    assert_eq!(
        pointer_h.app().items.len(),
        2,
        "the item owner applies the response"
    );
    pointer_h.app_mut().actions.clear();
    middle(&mut pointer_h, PartRef::item(Part::TAB, ItemKey::Num(2)));
    assert_eq!(
        pointer_h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(2))]
    );
    assert!(
        pointer_h.diagnostics().is_empty(),
        "{:?}",
        pointer_h.diagnostics()
    );

    let painted_groups = harness(Page::new(config), 14);
    let plain = harness(Page::new(Config::default()), 14);
    assert_eq!(
        painted_groups.buffer(),
        plain.buffer(),
        "key groups do not alter strip cells"
    );
    for part in [
        PartRef::of(Part::NEW),
        PartRef::item(Part::CLOSE, ItemKey::Num(1)),
        PartRef::item(Part::OVERFLOW, ItemKey::Index(1)),
    ] {
        assert_eq!(
            painted_groups.area_of_part(TABS, part),
            plain.area_of_part(TABS, part)
        );
    }
}

#[test]
fn command_groups_respect_gates_passive_host_focus_and_close_eligibility() {
    let disabled_new = Config {
        configured: true,
        new_keys: Some(&NEW_T),
        allow_new: false,
        ..Config::default()
    };
    let mut h = harness(Page::new(disabled_new), 48);
    assert!(Page::widget(disabled_new).new_key_table().is_empty());
    assert!(
        !h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "New tab")
    );
    let _ = h.key(KeyCode::Char('t'));
    assert!(h.app().actions.is_empty());
    assert!(h.area_of_part(TABS, PartRef::of(Part::NEW)).is_none());

    let close_subset = Config {
        configured: true,
        close_keys: Some(&CLOSE_DELETE),
        allow_new: false,
        closable_keys: Some(&FIRST_TAB_ONLY),
        ..Config::default()
    };
    let mut h = harness(Page::new(close_subset), 48);
    assert!(hint(h.app(), "Close tab", Chord::key(KeyCode::Delete), 73));
    let _ = h.key(KeyCode::Right);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Delete);
    assert!(
        h.app().actions.is_empty(),
        "a nonclosable cursor cannot close"
    );
    let _ = h.key(KeyCode::Left);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Delete);
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(1))]
    );

    let passive = Config {
        configured: true,
        new_keys: Some(&NEW_T),
        passive: true,
        host_input: true,
        ..Config::default()
    };
    let mut h = harness(Page::new(passive), 48);
    assert_eq!(h.focus(), Some(HOST));
    assert!(
        Page::widget(passive)
            .bindings(BindingState::default())
            .is_empty()
    );
    assert!(Page::widget(passive).new_key_table().is_empty());
    assert!(
        !h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "New tab")
    );
    let _ = h.key(KeyCode::Tab);
    assert_eq!(h.focus(), Some(HOST));
    let _ = h.click(HOST_AREA.x.saturating_add(4), HOST_AREA.y);
    assert!(h.app().host_state.is_editing());
    let _ = h.type_str("t");
    assert_eq!(h.app().host_state.draft_text(), Some("t"));
    assert!(h.app().actions.is_empty());
    let _ = h.click_part(TABS, PartRef::of(Part::NEW));
    assert_eq!(h.app().actions.as_slice(), [TabsAction::New]);
    assert_eq!(
        h.focus(),
        Some(HOST),
        "passive pointer actions keep host focus"
    );

    let host_focused = Config {
        configured: true,
        new_keys: Some(&NEW_T),
        host_input: true,
        ..Config::default()
    };
    let mut h = harness(Page::new(host_focused), 48);
    assert!(h.tab_to(HOST));
    let _ = h.click(HOST_AREA.x.saturating_add(4), HOST_AREA.y);
    let _ = h.type_str("t");
    assert_eq!(h.app().host_state.draft_text(), Some("t"));
    assert!(
        h.app().actions.is_empty(),
        "unfocused Tabs do not own the key"
    );

    for config in [
        Config {
            configured: true,
            new_keys: Some(&NEW_T),
            modal: true,
            ..Config::default()
        },
        Config {
            configured: true,
            new_keys: Some(&NEW_T),
            reference_tabs: true,
            host_input: true,
            ..Config::default()
        },
    ] {
        let mut h = harness(Page::new(config), 48);
        if config.modal {
            assert_eq!(h.focus(), Some(MODAL));
        } else {
            assert_eq!(h.focus(), Some(HOST));
        }
        let _ = h.key(KeyCode::Char('t'));
        assert!(h.app().actions.is_empty());
        assert!(
            !h.app()
                .hints
                .borrow()
                .iter()
                .any(|hint| hint.label == "New tab")
        );
    }
}

#[test]
fn removed_or_disabled_bindings_do_not_dispatch_and_empty_items_still_allow_new() {
    let initial = Config {
        configured: true,
        new_keys: Some(&NEW_T),
        close_keys: Some(&CLOSE_DELETE),
        ..Config::default()
    };
    let mut h = harness(Page::new(initial), 48);
    assert!(hint(h.app(), "New tab", Chord::key(KeyCode::Char('t')), 77));
    h.app_mut().next_config = Some(Config {
        configured: true,
        new_keys: Some(&[]),
        close_keys: Some(&[]),
        ..Config::default()
    });
    let _ = h.key(KeyCode::Char('t'));
    assert!(
        h.app().actions.is_empty(),
        "the old presented action was removed"
    );
    assert!(h.app().hints.borrow().iter().all(|hint| {
        hint.label != "New tab" && hint.label != "Close tab" && hint.label != "Close"
    }));
    let _ = h.key(KeyCode::Delete);
    assert!(h.app().actions.is_empty());

    let mut remapped_empty = Page::new(Config {
        configured: true,
        new_keys: Some(&[]),
        ..Config::default()
    });
    remapped_empty.keymap = KeyMap::new().bind_component(
        TABS,
        ActionKey::custom("tabs.new"),
        Chord::key(KeyCode::F(9)),
    );
    let mut h = harness(remapped_empty, 48);
    let _ = h.key(KeyCode::F(9));
    assert!(
        h.app().actions.is_empty(),
        "a component remap cannot restore an absent action"
    );

    let mut empty_items = Page::new(Config {
        configured: true,
        new_keys: Some(&NEW_T),
        ..Config::default()
    });
    empty_items.items.clear();
    let mut h = harness(empty_items, 48);
    let _ = h.key(KeyCode::Char('t'));
    action_new(&h);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn invalid_groups_are_rejected_before_any_widget_can_publish_them() {
    let wrong_new = Tabs::<Tab>::new(TABS)
        .allow_new(false)
        .passive(true)
        .try_new_key_bindings(Some(&BAD_NEW_COMMAND));
    assert_eq!(
        wrong_new.err(),
        Some(TabsKeyBindingsError::NewGroupCommand {
            index: 0,
            found: TabsCmd::Close,
        })
    );
    let wrong_close = Tabs::<Tab>::new(TABS)
        .closable(false)
        .passive(true)
        .try_close_key_bindings(Some(&BAD_CLOSE_COMMAND));
    assert_eq!(
        wrong_close.err(),
        Some(TabsKeyBindingsError::CloseGroupCommand {
            index: 0,
            found: TabsCmd::New,
        })
    );
    assert!(matches!(
        Tabs::<Tab>::new(TABS).try_new_key_bindings(Some(&DUPLICATE_NEW_ACTION)),
        Err(TabsKeyBindingsError::DuplicateAction { .. })
    ));
    assert!(matches!(
        Tabs::<Tab>::new(TABS)
            .allow_new(false)
            .passive(true)
            .try_new_key_bindings(Some(&CONFLICTS_WITH_BASE)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        Tabs::<Tab>::new(TABS)
            .closable(false)
            .passive(true)
            .try_close_key_bindings(Some(&CLOSE_ON_RIGHT)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        Tabs::<Tab>::new(TABS).try_new_key_bindings(Some(&CONFLICTING_SHIFTED_ALIASES)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        Tabs::<Tab>::new(TABS)
            .try_new_key_bindings(Some(&NEW_T))
            .expect("New profile replaces its default group")
            .try_close_key_bindings(Some(&CROSS_GROUP_CHORD)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        Tabs::<Tab>::new(TABS)
            .try_new_key_bindings(Some(&NEW_T))
            .expect("New profile replaces its default group")
            .try_close_key_bindings(Some(&CROSS_GROUP_DUPLICATE_ACTION)),
        Err(TabsKeyBindingsError::DuplicateAction { .. })
    ));

    // Reusing the built-in action identity is valid when replacing that same
    // group; the old default entry is absent from the effective base.
    let replacement = Tabs::<Tab>::new(TABS)
        .allow_new(true)
        .try_new_key_bindings(Some(&NEW_T))
        .expect("the replaced default New action does not conflict with itself");
    assert!(
        Binding::command(
            replacement.bindings(BindingState::default()),
            ActionKey::custom("tabs.new")
        )
        .is_none()
    );
    assert_eq!(replacement.new_key_table(), &NEW_T);
    let close_replacement = Tabs::<Tab>::new(TABS)
        .closable(true)
        .try_close_key_bindings(Some(&CLOSE_DELETE))
        .expect("the replaced default Close action does not conflict with itself");
    assert!(
        Binding::command(
            close_replacement.bindings(BindingState::default()),
            ActionKey::custom("tabs.close")
        )
        .is_none()
    );
    assert_eq!(close_replacement.close_key_table(), &CLOSE_DELETE);
}

#[test]
fn group_overrides_survive_builder_orders_and_do_not_change_planned_rows() {
    let variants = [
        GroupOrder::BeforeKeyAndRow,
        GroupOrder::AfterKeyAndRow,
        GroupOrder::RowThenKey,
        GroupOrder::CloseFirst,
    ];
    let mut reference = None;
    for order in variants {
        let config = Config {
            configured: true,
            new_keys: Some(&NEW_T),
            close_keys: Some(&CLOSE_DELETE),
            group_order: order,
            ..Config::default()
        };
        let tabs = Page::widget(config);
        assert_eq!(tabs.new_key_table(), &NEW_T, "{order:?}");
        assert_eq!(tabs.close_key_table(), &CLOSE_DELETE, "{order:?}");
        let current = (
            tabs.bindings(BindingState::default()).to_vec(),
            tabs.new_key_table().to_vec(),
            tabs.close_key_table().to_vec(),
        );
        if let Some(reference) = &reference {
            assert_eq!(&current, reference, "builder order retains both groups");
        } else {
            reference = Some(current);
        }
    }

    let legacy = harness(Page::new(Config::default()), 14);
    let planned_config = Config {
        configured: true,
        new_keys: Some(&NEW_T),
        close_keys: Some(&CLOSE_DELETE),
        ..Config::default()
    };
    let mut planned_page = Page::new(planned_config);
    planned_page.plan = Some(PlanCase {
        config: planned_config,
        empty_area: false,
    });
    let planned = harness(planned_page, 14);
    assert!(!planned.app().plan_rejected.get());
    assert_eq!(legacy.buffer(), planned.buffer());
    for part in [
        PartRef::of(Part::NEW),
        PartRef::item(Part::CLOSE, ItemKey::Num(1)),
        PartRef::item(Part::OVERFLOW, ItemKey::Index(1)),
    ] {
        assert_eq!(
            legacy.area_of_part(TABS, part),
            planned.area_of_part(TABS, part)
        );
    }
}

#[test]
fn plan_rejects_none_vs_empty_group_changes_before_paint_or_registration() {
    for empty_area in [false, true] {
        let mut page = Page::new(Config {
            configured: true,
            new_keys: Some(&[]),
            ..Config::default()
        });
        page.plan = Some(PlanCase {
            config: Config::default(),
            empty_area,
        });
        page.sentinel = true;
        let h = harness(page, 48);
        assert!(h.app().plan_rejected.get());
        assert_eq!(h.cell(0, 0).bg, Color::Blue);
        assert!(h.area_of(TABS).is_none());
        assert!(h.area_of_part(TABS, PartRef::of(Part::NEW)).is_none());
        assert!(h.app().hints.borrow().is_empty());
    }
}

#[test]
fn plan_rejects_close_only_changes_with_new_group_unchanged() {
    let planned_config = Config {
        configured: true,
        new_keys: Some(&NEW_T),
        ..Config::default()
    };
    let cases: [(Option<&'static [Binding<TabsCmd>]>, bool); 3] = [
        (Some(&[]), false),
        (Some(&[]), true),
        (Some(&CLOSE_PROFILE), false),
    ];

    for (close_keys, empty_area) in cases {
        let actual_config = Config {
            close_keys,
            ..planned_config
        };
        let mut page = Page::new(actual_config);
        page.plan = Some(PlanCase {
            config: planned_config,
            empty_area,
        });
        page.sentinel = true;

        let h = harness(page, 48);
        assert!(h.app().plan_rejected.get());
        assert_eq!(h.cell(0, 0).bg, Color::Blue);
        assert_eq!(h.cell(1, 1).bg, Color::Blue);
        assert!(h.area_of(TABS).is_none());
        for part in [
            PartRef::item(Part::TAB, ItemKey::Num(1)),
            PartRef::item(Part::CLOSE, ItemKey::Num(1)),
            PartRef::of(Part::NEW),
            PartRef::of(Part::OVERFLOW),
        ] {
            assert!(h.area_of_part(TABS, part).is_none(), "{part:?}");
        }
        assert!(h.app().hints.borrow().is_empty());
    }
}
