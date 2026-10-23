#![allow(
    clippy::unwrap_used,
    reason = "real Tabs digit-key contract assertions"
)]

use core::cell::{Cell, RefCell};
use core::fmt;

use ratatui_core::layout::Rect;
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyModifiers};
use termrock_core::{Id, ItemKey, KeyCode, Part, PartRef, Response};
use termrock_navigation::{Tabs, TabsAction, TabsCmd, TabsKeyBindingsError, TabsState};
use termrock_runtime::keymap::{Binding, BindingState, Bindings, Hint, KeyMap};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TABS: Id = Id::root("tabs.digit-jump");

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

type TabsWidget = Tabs<'static, Tab, fn(&Tab) -> ItemKey>;

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

const NEW_PROFILE: [Binding<TabsCmd>; 2] = [
    binding(
        "tabs.new.custom",
        Some(Chord::key(KeyCode::Char('t'))),
        TabsCmd::New,
        "Open tab",
        77,
        true,
    ),
    binding(
        "tabs.new.palette",
        Some(Chord::with(KeyCode::Char('p'), KeyModifiers::ALT)),
        TabsCmd::New,
        "Open from palette",
        46,
        false,
    ),
];

const CLOSE_PROFILE: [Binding<TabsCmd>; 2] = [
    binding(
        "tabs.close.custom",
        Some(Chord::key(KeyCode::Delete)),
        TabsCmd::Close,
        "Close current",
        73,
        true,
    ),
    binding(
        "tabs.close.window",
        Some(Chord::with(KeyCode::Char('w'), KeyModifiers::CONTROL)),
        TabsCmd::Close,
        "Close window",
        62,
        true,
    ),
];

const NEW_ON_ONE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.on-one",
    Some(Chord::key(KeyCode::Char('1'))),
    TabsCmd::New,
    "New on 1",
    30,
    true,
)];

const NEW_ON_SHIFT_ONE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.on-shift-one",
    Some(Chord::with(KeyCode::Char('1'), KeyModifiers::SHIFT)),
    TabsCmd::New,
    "New on shifted 1",
    30,
    true,
)];

const NEW_DUPLICATE_DIGIT_ACTION: [Binding<TabsCmd>; 1] = [binding(
    "tabs.tab-1",
    Some(Chord::key(KeyCode::F(10))),
    TabsCmd::New,
    "New on F10",
    30,
    true,
)];

const CLOSE_ON_NINE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.on-nine",
    Some(Chord::key(KeyCode::Char('9'))),
    TabsCmd::Close,
    "Close on 9",
    30,
    true,
)];

const CLOSE_ON_SHIFT_NINE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.on-shift-nine",
    Some(Chord::with(KeyCode::Char('9'), KeyModifiers::SHIFT)),
    TabsCmd::Close,
    "Close on shifted 9",
    30,
    true,
)];

const CLOSE_DUPLICATE_DIGIT_ACTION: [Binding<TabsCmd>; 1] = [binding(
    "tabs.tab-9",
    Some(Chord::key(KeyCode::F(11))),
    TabsCmd::Close,
    "Close on F11",
    30,
    true,
)];

struct Page {
    digit_jump: Option<bool>,
    next_digit_jump: Option<bool>,
    custom_groups: bool,
    passive: bool,
    reference_tabs: bool,
    plan: bool,
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    keymap: KeyMap,
    hints: RefCell<Vec<Hint>>,
    plan_rejected: Cell<bool>,
}

impl Page {
    fn new(digit_jump: Option<bool>, count: usize, active_index: usize) -> Self {
        let items: Vec<_> = (1..=count)
            .map(|id| Tab {
                id: id as u64,
                label: format!("Tab{id}"),
            })
            .collect();
        let mut state = TabsState::default();
        if let Some(tab) = items.get(active_index) {
            state.set_active(active_index, ItemKey::num(tab.id));
        }
        Self {
            digit_jump,
            next_digit_jump: None,
            custom_groups: false,
            passive: false,
            reference_tabs: false,
            plan: false,
            state,
            items,
            actions: Vec::new(),
            keymap: KeyMap::new(),
            hints: RefCell::new(Vec::new()),
            plan_rejected: Cell::new(false),
        }
    }

    fn widget(&self, digit_jump: Option<bool>) -> TabsWidget {
        let mut tabs = Tabs::new(TABS).allow_new(true).closable(true);
        if let Some(enabled) = digit_jump {
            tabs = tabs.digit_jump(enabled);
        }
        if self.passive {
            tabs = tabs.passive(true);
        }
        if self.custom_groups {
            tabs = tabs
                .try_new_key_bindings(Some(&NEW_PROFILE))
                .expect("the test New profile is valid")
                .try_close_key_bindings(Some(&CLOSE_PROFILE))
                .expect("the test Close profile is valid");
        }
        tabs.key(tab_key as fn(&Tab) -> ItemKey)
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(enabled) = self.next_digit_jump.take() {
            self.digit_jump = Some(enabled);
        }
        let tabs = self.widget(self.digit_jump);
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
        let tabs = self.widget(self.digit_jump);
        if self.plan {
            let planned = self.widget(None);
            let layout = planned.layout_default(area, area, &self.state, &self.items, None);
            self.plan_rejected
                .set(tabs.draw_layout(ui, &layout).is_err());
        } else if self.reference_tabs {
            ui.reference(None, |reference| {
                tabs.draw(reference, area, &self.state, &self.items)
            });
        } else {
            tabs.draw(ui, area, &self.state, &self.items);
        }
        let hints = ui.with_focused_hints(|_, layer| layer.hints.clone());
        self.hints.replace(hints.unwrap_or_default());
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: Page, width: u16) -> Harness<Page> {
    Harness::new(page, Theme::junie(), width, 4)
}

fn digit_action(n: u8) -> &'static str {
    match n {
        1 => "tabs.tab-1",
        2 => "tabs.tab-2",
        3 => "tabs.tab-3",
        4 => "tabs.tab-4",
        5 => "tabs.tab-5",
        6 => "tabs.tab-6",
        7 => "tabs.tab-7",
        8 => "tabs.tab-8",
        9 => "tabs.tab-9",
        _ => unreachable!("only digits 1 through 9 have default actions"),
    }
}

fn assert_digit_descriptors(table: &[Binding<TabsCmd>], enabled: bool) {
    for n in 1..=9 {
        let action = ActionKey::custom(digit_action(n));
        let binding = table.iter().find(|entry| entry.action == action);
        if enabled {
            let binding = binding.unwrap_or_else(|| panic!("missing digit action {n}"));
            assert_eq!(
                binding.chord,
                Some(Chord::key(KeyCode::Char(char::from(b'0' + n))))
            );
            assert_eq!(binding.cmd, TabsCmd::Nth(n));
            assert_eq!(binding.label, format!("Tab {n}"));
            assert_eq!(binding.priority, 10);
            assert!(!binding.visible);
        } else {
            assert!(binding.is_none(), "disabled action {n} remains registered");
        }
    }
}

fn has_digit_action(table: &[Binding<TabsCmd>]) -> bool {
    (1..=9).any(|n| {
        let action = ActionKey::custom(digit_action(n));
        table.iter().any(|entry| entry.action == action)
    })
}

#[test]
fn absent_and_explicit_true_keep_all_nine_digit_descriptors_and_frames_exact() {
    let implicit = harness(Page::new(None, 12, 5), 96);
    let explicit = harness(Page::new(Some(true), 12, 5), 96);
    assert_eq!(implicit.buffer(), explicit.buffer());
    assert_eq!(implicit.ring().entries(), explicit.ring().entries());
    assert_eq!(implicit.focus(), explicit.focus());
    assert_eq!(implicit.app().state, explicit.app().state);
    assert_eq!(implicit.app().actions, explicit.app().actions);
    assert_eq!(
        *implicit.app().hints.borrow(),
        *explicit.app().hints.borrow()
    );

    let implicit_tabs = implicit.app().widget(None);
    let explicit_tabs = explicit.app().widget(Some(true));
    let table = implicit_tabs.bindings(BindingState::default());
    assert_eq!(table.len(), 18, "the default FULL table stays unchanged");
    assert_eq!(table, explicit_tabs.bindings(BindingState::default()));
    assert_digit_descriptors(table, true);
    assert!(implicit_tabs.new_key_table().is_empty());
    assert!(implicit_tabs.close_key_table().is_empty());
    assert!(
        implicit
            .app()
            .hints
            .borrow()
            .iter()
            .all(|hint| !(1..=9).any(|n| hint.label == format!("Tab {n}")))
    );

    let mut h = harness(Page::new(None, 12, 5), 96);
    let _ = h.key(KeyCode::Char('1'));
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(1))]
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('9'));
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(9))]
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn false_removes_the_nine_default_descriptors_without_changing_paint_or_hits() {
    let enabled = harness(Page::new(None, 12, 5), 24);
    let disabled = harness(Page::new(Some(false), 12, 5), 24);
    assert_eq!(enabled.buffer(), disabled.buffer());
    assert_eq!(enabled.ring().entries(), disabled.ring().entries());
    assert_eq!(enabled.focus(), disabled.focus());
    assert_eq!(enabled.app().state, disabled.app().state);
    assert_eq!(
        enabled.area_of(TABS),
        disabled.area_of(TABS),
        "digit policy is not geometry"
    );
    assert_eq!(
        enabled.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(1))),
        disabled.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(1)))
    );

    let enabled_table = enabled.app().widget(None).bindings(BindingState::default());
    let disabled_tabs = disabled.app().widget(Some(false));
    let disabled_table = disabled_tabs.bindings(BindingState::default());
    assert_eq!(enabled_table.len(), 18);
    assert_eq!(disabled_table.len(), 9);
    assert_eq!(&disabled_table[..6], &enabled_table[..6]);
    assert_eq!(&disabled_table[6..], &enabled_table[15..]);
    assert_digit_descriptors(disabled_table, false);
    assert!(disabled_tabs.new_key_table().is_empty());
    assert!(disabled_tabs.close_key_table().is_empty());
    assert!(
        disabled.diagnostics().is_empty(),
        "{:?}",
        disabled.diagnostics()
    );
}

#[test]
fn disabled_digits_do_not_dispatch_or_move_a_narrow_overflow_window() {
    let mut disabled = harness(Page::new(Some(false), 12, 6), 18);
    let _ = disabled.tick();
    assert!(
        disabled
            .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(9)))
            .is_none(),
        "tab 9 begins outside the narrow visible window"
    );
    let before_state = disabled.app().state.clone();
    let before_buffer = disabled.buffer().clone();
    let before_first = disabled.app().state.first();
    for digit in ['1', '9'] {
        let _ = disabled.key(KeyCode::Char(digit));
        assert!(disabled.app().actions.is_empty());
        assert_eq!(disabled.app().state, before_state);
        assert_eq!(disabled.app().state.first(), before_first);
        assert_eq!(disabled.buffer(), &before_buffer);
        assert!(
            disabled
                .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(9)))
                .is_none()
        );
    }
    assert!(
        disabled.diagnostics().is_empty(),
        "{:?}",
        disabled.diagnostics()
    );

    let mut enabled = harness(Page::new(None, 12, 6), 18);
    assert!(
        enabled
            .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(9)))
            .is_none()
    );
    let _ = enabled.key(KeyCode::Char('9'));
    assert_eq!(
        enabled.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(9))]
    );
    assert_eq!(enabled.app().state.active(), Some(ItemKey::Num(9)));
    assert!(
        enabled
            .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(9)))
            .is_some(),
        "activating an offscreen current key follows it into the viewport"
    );

    enabled.app_mut().items.swap(1, 8);
    let _ = enabled.tick();
    enabled.app_mut().actions.clear();
    let _ = enabled.key(KeyCode::Char('2'));
    assert_eq!(
        enabled.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(9))],
        "the second digit resolves against the reordered current slice"
    );

    enabled.app_mut().items.remove(0);
    let _ = enabled.tick();
    enabled.app_mut().actions.clear();
    let _ = enabled.key(KeyCode::Char('2'));
    assert_eq!(
        enabled.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(3))],
        "after removal the digit resolves against the new current position"
    );
    assert!(
        enabled.diagnostics().is_empty(),
        "{:?}",
        enabled.diagnostics()
    );
}

#[test]
fn digit_toggle_rejects_an_old_queued_action_and_reenables_the_same_bindings() {
    let mut h = harness(Page::new(None, 12, 5), 96);
    let _ = h.tick();
    let initial_state = h.app().state.clone();
    let initial_table = h
        .app()
        .widget(None)
        .bindings(BindingState::default())
        .to_vec();

    h.app_mut().next_digit_jump = Some(false);
    let _ = h.key(KeyCode::Char('9'));
    assert!(
        h.app().actions.is_empty(),
        "the action queued from the prior true frame is absent in the current false map"
    );
    assert_eq!(h.app().state, initial_state);
    let disabled_table = h
        .app()
        .widget(h.app().digit_jump)
        .bindings(BindingState::default());
    assert_digit_descriptors(disabled_table, false);
    assert!(h.app().state.first().is_some());

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('1'));
    assert!(
        h.app().actions.is_empty(),
        "disabled digit stays unregistered"
    );
    assert_eq!(h.app().state, initial_state);

    h.app_mut().next_digit_jump = Some(true);
    let _ = h.tick();
    let restored = h
        .app()
        .widget(h.app().digit_jump)
        .bindings(BindingState::default());
    assert_eq!(restored, initial_table);
    assert_digit_descriptors(restored, true);
    let _ = h.key(KeyCode::Char('9'));
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(9))]
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let mut remapped_digit = Page::new(Some(false), 12, 6);
    remapped_digit.keymap = KeyMap::new().bind_component(
        TABS,
        ActionKey::custom("tabs.tab-9"),
        Chord::key(KeyCode::F(11)),
    );
    let mut h = harness(remapped_digit, 64);
    let _ = h.tick();
    let state = h.app().state.clone();
    let _ = h.key(KeyCode::F(11));
    assert!(
        h.app().actions.is_empty(),
        "a component remap cannot restore a removed digit action"
    );
    assert_eq!(h.app().state, state);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn disabled_digits_leave_arrows_vim_enter_space_and_pointer_tabs_unchanged() {
    let mut h = harness(Page::new(Some(false), 12, 0), 64);
    for (key, expected) in [
        (KeyCode::Right, ItemKey::Num(2)),
        (KeyCode::Char('h'), ItemKey::Num(1)),
        (KeyCode::Char('l'), ItemKey::Num(2)),
        (KeyCode::Enter, ItemKey::Num(2)),
        (KeyCode::Char(' '), ItemKey::Num(2)),
    ] {
        let response = h.key(key);
        assert!(
            response.is_consumed(),
            "existing command {key:?} stays bound"
        );
        assert_eq!(
            h.app().actions.last(),
            Some(&TabsAction::Activated(expected))
        );
        h.app_mut().actions.clear();
    }
    let tab = h
        .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(3)))
        .expect("the third tab remains a live pointer target");
    let _ = h.click(tab.x, tab.y);
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(3))]
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn false_keeps_custom_supplemental_tables_metadata_and_unrelated_remaps() {
    let mut page = Page::new(Some(false), 12, 0);
    page.custom_groups = true;
    page.keymap = KeyMap::new().bind_component(
        TABS,
        ActionKey::custom("tabs.next"),
        Chord::key(KeyCode::F(10)),
    );
    let mut h = harness(page, 64);
    let tabs = h.app().widget(Some(false));
    let base = tabs.bindings(BindingState::default());
    assert_eq!(base.len(), 6);
    assert_digit_descriptors(base, false);
    assert_eq!(tabs.new_key_table(), &NEW_PROFILE);
    assert_eq!(tabs.close_key_table(), &CLOSE_PROFILE);
    for (table, expected) in [
        (tabs.new_key_table(), &NEW_PROFILE[..]),
        (tabs.close_key_table(), &CLOSE_PROFILE[..]),
    ] {
        assert_eq!(table, expected);
        assert!(!has_digit_action(table));
    }
    assert_eq!(
        base,
        &h.app().widget(None).bindings(BindingState::default())[..6]
    );
    let manually_published: Vec<_> = base
        .iter()
        .chain(tabs.new_key_table())
        .chain(tabs.close_key_table())
        .copied()
        .collect();
    assert!(!has_digit_action(&manually_published));
    assert!(
        h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "Open tab")
    );
    assert!(
        h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "Close current")
    );

    let _ = h.key(KeyCode::F(10));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(2)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('t'));
    assert_eq!(h.app().actions.as_slice(), [TabsAction::New]);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Delete);
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(2))]
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn digit_setting_survives_key_and_row_type_changes_and_builder_orders() {
    let before = Tabs::<Tab>::new(TABS)
        .digit_jump(false)
        .key(tab_key as fn(&Tab) -> ItemKey)
        .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>));
    let between = Tabs::<Tab>::new(TABS)
        .key(tab_key as fn(&Tab) -> ItemKey)
        .digit_jump(false)
        .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>));
    let after = Tabs::<Tab>::new(TABS)
        .key(tab_key as fn(&Tab) -> ItemKey)
        .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>))
        .digit_jump(false);
    let tables = [
        before.bindings(BindingState::default()),
        between.bindings(BindingState::default()),
        after.bindings(BindingState::default()),
    ];
    assert!(tables.windows(2).all(|pair| pair[0] == pair[1]));
    for table in tables {
        assert_digit_descriptors(table, false);
    }
    assert!(format!("{before:?}").contains("digit_jump: false"));
    assert!(format!("{between:?}").contains("digit_jump: false"));
    assert!(format!("{after:?}").contains("digit_jump: false"));

    let off_then_new = Tabs::<Tab>::new(TABS)
        .allow_new(true)
        .digit_jump(false)
        .try_new_key_bindings(Some(&NEW_PROFILE))
        .expect("the unrelated New profile remains valid");
    let new_then_off = Tabs::<Tab>::new(TABS)
        .allow_new(true)
        .try_new_key_bindings(Some(&NEW_PROFILE))
        .expect("the unrelated New profile remains valid")
        .digit_jump(false);
    assert_eq!(
        off_then_new.bindings(BindingState::default()),
        new_then_off.bindings(BindingState::default())
    );
    assert_eq!(off_then_new.new_key_table(), &NEW_PROFILE);
    assert_eq!(new_then_off.new_key_table(), &NEW_PROFILE);

    let variants = [
        (
            Tabs::<Tab>::new(TABS),
            Tabs::<Tab>::new(TABS).digit_jump(false),
            6,
        ),
        (
            Tabs::<Tab>::new(TABS).closable(true),
            Tabs::<Tab>::new(TABS).closable(true).digit_jump(false),
            8,
        ),
        (
            Tabs::<Tab>::new(TABS).allow_new(true),
            Tabs::<Tab>::new(TABS).allow_new(true).digit_jump(false),
            7,
        ),
        (
            Tabs::<Tab>::new(TABS).allow_new(true).closable(true),
            Tabs::<Tab>::new(TABS)
                .allow_new(true)
                .closable(true)
                .digit_jump(false),
            9,
        ),
    ];
    for (enabled, disabled, expected_len) in variants {
        let enabled = enabled.bindings(BindingState::default());
        let disabled = disabled.bindings(BindingState::default());
        assert_eq!(disabled.len(), expected_len);
        assert_eq!(&disabled[..6], &enabled[..6]);
        assert_eq!(&disabled[6..], &enabled[15..]);
        assert_digit_descriptors(disabled, false);
    }
}

#[test]
fn false_keeps_planned_rendering_and_reference_nonpublication_contracts() {
    let mut legacy_page = Page::new(Some(false), 12, 5);
    let legacy = harness(legacy_page, 28);
    legacy_page = Page::new(Some(false), 12, 5);
    legacy_page.plan = true;
    let planned = harness(legacy_page, 28);
    assert!(!planned.app().plan_rejected.get());
    assert_eq!(legacy.buffer(), planned.buffer());
    assert_eq!(legacy.ring().entries(), planned.ring().entries());
    assert_eq!(legacy.app().state, planned.app().state);
    assert_eq!(
        legacy.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(1))),
        planned.area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(1)))
    );

    let mut reference_page = Page::new(Some(false), 12, 0);
    reference_page.reference_tabs = true;
    let mut reference = harness(reference_page, 64);
    assert!(reference.area_of(TABS).is_none());
    assert!(reference.app().hints.borrow().is_empty());
    let _ = reference.key(KeyCode::Char('1'));
    assert!(reference.app().actions.is_empty());
    assert!(reference.area_of(TABS).is_none());
    assert!(
        reference.diagnostics().is_empty(),
        "{:?}",
        reference.diagnostics()
    );
}

#[test]
fn digit_setting_does_not_change_passive_nonpublication() {
    let mut enabled_page = Page::new(None, 5, 2);
    enabled_page.passive = true;
    let mut enabled = harness(enabled_page, 64);
    let mut disabled_page = Page::new(Some(false), 5, 2);
    disabled_page.passive = true;
    let mut disabled = harness(disabled_page, 64);
    let _ = enabled.tick();
    let _ = disabled.tick();

    assert_eq!(enabled.buffer(), disabled.buffer());
    assert!(enabled.area_of(TABS).is_none());
    assert!(disabled.area_of(TABS).is_none());
    assert!(enabled.ring().entries().is_empty());
    assert!(disabled.ring().entries().is_empty());
    assert!(enabled.app().hints.borrow().is_empty());
    assert!(disabled.app().hints.borrow().is_empty());
    for h in [&enabled, &disabled] {
        let tabs = h.app().widget(h.app().digit_jump);
        assert!(tabs.bindings(BindingState::default()).is_empty());
        assert!(tabs.new_key_table().is_empty());
        assert!(tabs.close_key_table().is_empty());
    }

    let before = disabled.app().state.clone();
    let _ = disabled.key(KeyCode::Char('1'));
    assert!(disabled.app().actions.is_empty());
    assert_eq!(disabled.app().state, before);
    assert!(
        disabled.diagnostics().is_empty(),
        "{:?}",
        disabled.diagnostics()
    );
}

#[test]
fn short_out_of_range_nine_remains_consumed_without_activation() {
    let mut h = harness(Page::new(None, 3, 0), 48);
    let state = h.app().state.clone();
    let response = h.key(KeyCode::Char('9'));
    assert!(response.is_consumed(), "the existing Nth miss is consumed");
    assert!(!response.is_changed());
    assert!(h.app().actions.is_empty());
    assert_eq!(h.app().state, state);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn digit_chord_alias_and_action_collisions_stay_reserved_when_jump_is_false() {
    let base = || {
        Tabs::<Tab>::new(TABS)
            .allow_new(false)
            .closable(false)
            .passive(true)
            .digit_jump(false)
    };
    assert!(matches!(
        base().try_new_key_bindings(Some(&NEW_ON_ONE)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        base().try_new_key_bindings(Some(&NEW_ON_SHIFT_ONE)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        base().try_new_key_bindings(Some(&NEW_DUPLICATE_DIGIT_ACTION)),
        Err(TabsKeyBindingsError::DuplicateAction { .. })
    ));
    assert!(matches!(
        base().try_close_key_bindings(Some(&CLOSE_ON_NINE)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        base().try_close_key_bindings(Some(&CLOSE_ON_SHIFT_NINE)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(matches!(
        base().try_close_key_bindings(Some(&CLOSE_DUPLICATE_DIGIT_ACTION)),
        Err(TabsKeyBindingsError::DuplicateAction { .. })
    ));

    let new_first = Tabs::<Tab>::new(TABS)
        .allow_new(false)
        .passive(true)
        .try_new_key_bindings(Some(&NEW_PROFILE))
        .expect("unrelated New profile is valid")
        .digit_jump(false)
        .try_close_key_bindings(Some(&CLOSE_ON_NINE));
    assert!(matches!(
        new_first,
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    let off_first = Tabs::<Tab>::new(TABS)
        .allow_new(false)
        .passive(true)
        .digit_jump(false)
        .try_new_key_bindings(Some(&NEW_PROFILE))
        .expect("unrelated New profile is valid")
        .try_close_key_bindings(Some(&CLOSE_ON_NINE));
    assert!(matches!(
        off_first,
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));

    let close_first = Tabs::<Tab>::new(TABS)
        .closable(false)
        .passive(true)
        .try_close_key_bindings(Some(&CLOSE_PROFILE))
        .expect("unrelated Close profile is valid")
        .digit_jump(false)
        .try_new_key_bindings(Some(&NEW_ON_SHIFT_ONE));
    assert!(matches!(
        close_first,
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    let off_then_close = Tabs::<Tab>::new(TABS)
        .closable(false)
        .passive(true)
        .digit_jump(false)
        .try_close_key_bindings(Some(&CLOSE_PROFILE))
        .expect("unrelated Close profile is valid")
        .try_new_key_bindings(Some(&NEW_ON_SHIFT_ONE));
    assert!(matches!(
        off_then_close,
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
}
