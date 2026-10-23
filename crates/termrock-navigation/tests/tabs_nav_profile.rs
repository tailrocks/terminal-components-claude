#![allow(
    clippy::unwrap_used,
    reason = "real Tabs navigation profile assertions"
)]

use core::cell::Cell;

use ratatui_core::layout::Rect;
use termrock_core::action::ActionKey;
use termrock_core::event::{Chord, KeyModifiers};
use termrock_core::{Id, ItemKey, KeyCode, Part, PartRef};
use termrock_navigation::{
    TabNavProfile, Tabs, TabsAction, TabsCmd, TabsKeyBindingsError, TabsState,
};
use termrock_runtime::keymap::{Binding, BindingState, Bindings, KeyMap};
use termrock_runtime::{App, Cx, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const TABS: Id = Id::root("tabs.nav-profile");
const AREA: Rect = Rect::new(0, 0, 40, 2);

const fn binding(
    action: &'static str,
    chord: Chord,
    cmd: TabsCmd,
    label: &'static str,
    priority: u8,
    visible: bool,
) -> Binding<TabsCmd> {
    Binding {
        action: ActionKey::custom(action),
        chord: Some(chord),
        cmd,
        label,
        priority,
        visible,
    }
}

const NEW_SHIFT_LEFT: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.shift-left",
    Chord::with(KeyCode::Left, KeyModifiers::SHIFT),
    TabsCmd::New,
    "New tab",
    55,
    false,
)];
const CLOSE_SHIFT_LEFT: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.shift-left",
    Chord::with(KeyCode::Left, KeyModifiers::SHIFT),
    TabsCmd::Close,
    "Close tab",
    55,
    false,
)];
const NEW_H: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.h",
    Chord::key(KeyCode::Char('h')),
    TabsCmd::New,
    "New tab",
    55,
    false,
)];
const NEW_SHIFT_H: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.shift-h",
    Chord::with(KeyCode::Char('h'), KeyModifiers::SHIFT),
    TabsCmd::New,
    "New tab",
    55,
    false,
)];
const NEW_UPPER_H: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new.upper-h",
    Chord::key(KeyCode::Char('H')),
    TabsCmd::New,
    "New tab",
    55,
    false,
)];
const CLOSE_H: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close.h",
    Chord::key(KeyCode::Char('h')),
    TabsCmd::Close,
    "Close tab",
    55,
    false,
)];
const NEW_REUSED_ACTION: [Binding<TabsCmd>; 1] = [binding(
    "tabs.previous",
    Chord::key(KeyCode::F(12)),
    TabsCmd::New,
    "New tab",
    55,
    false,
)];
const SHIFTED_CHAR_ALIASES: [Binding<TabsCmd>; 2] = [NEW_H[0], NEW_SHIFT_H[0]];
const NEW_T: [Binding<TabsCmd>; 1] = [binding(
    "tabs.new",
    Chord::key(KeyCode::Char('t')),
    TabsCmd::New,
    "New tab",
    77,
    true,
)];
const CLOSE_DELETE: [Binding<TabsCmd>; 1] = [binding(
    "tabs.close",
    Chord::key(KeyCode::Delete),
    TabsCmd::Close,
    "Close tab",
    73,
    true,
)];

macro_rules! digit_new_binding {
    ($name:ident, $id:literal, $digit:literal) => {
        const $name: [Binding<TabsCmd>; 1] = [binding(
            $id,
            Chord::key(KeyCode::Char($digit)),
            TabsCmd::New,
            "New on digit",
            40,
            false,
        )];
    };
}

digit_new_binding!(NEW_1, "tabs.new.digit-1", '1');
digit_new_binding!(NEW_2, "tabs.new.digit-2", '2');
digit_new_binding!(NEW_3, "tabs.new.digit-3", '3');
digit_new_binding!(NEW_4, "tabs.new.digit-4", '4');
digit_new_binding!(NEW_5, "tabs.new.digit-5", '5');
digit_new_binding!(NEW_6, "tabs.new.digit-6", '6');
digit_new_binding!(NEW_7, "tabs.new.digit-7", '7');
digit_new_binding!(NEW_8, "tabs.new.digit-8", '8');
digit_new_binding!(NEW_9, "tabs.new.digit-9", '9');

const NEW_DIGITS: [&[Binding<TabsCmd>]; 9] = [
    &NEW_1, &NEW_2, &NEW_3, &NEW_4, &NEW_5, &NEW_6, &NEW_7, &NEW_8, &NEW_9,
];

#[derive(Clone, Copy)]
struct Tab {
    key: u64,
    label: &'static str,
}

impl core::fmt::Display for Tab {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.label)
    }
}

fn tab_key(tab: &Tab) -> ItemKey {
    ItemKey::num(tab.key)
}

fn tab_row(tab: &Tab, row: &mut termrock_runtime::RowUi<'_>) {
    row.label(tab.label);
}

type TabsWidget =
    Tabs<'static, Tab, fn(&Tab) -> ItemKey, for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)>;

#[derive(Clone, Copy, Debug)]
struct Config {
    profile: Option<TabNavProfile>,
    set_profile: bool,
    new_keys: Option<&'static [Binding<TabsCmd>]>,
    close_keys: Option<&'static [Binding<TabsCmd>]>,
    allow_new: bool,
    closable: bool,
    digit_jump: bool,
    wrap: bool,
    passive: bool,
    reference_only: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            profile: None,
            set_profile: false,
            new_keys: None,
            close_keys: None,
            allow_new: true,
            closable: true,
            digit_jump: true,
            wrap: false,
            passive: false,
            reference_only: false,
        }
    }
}

#[derive(Clone, Copy)]
enum BuildOrder {
    ProfileBeforeGroups,
    GroupsBeforeProfile,
    ProfileAfterKey,
    ProfileAfterRow,
}

fn with_profile<K, R>(tabs: Tabs<'static, Tab, K, R>, config: Config) -> Tabs<'static, Tab, K, R> {
    if config.set_profile {
        tabs.try_nav_profile(config.profile)
            .expect("profile and test binding groups are compatible")
    } else {
        tabs
    }
}

fn with_groups<K, R>(tabs: Tabs<'static, Tab, K, R>, config: Config) -> Tabs<'static, Tab, K, R> {
    tabs.try_new_key_bindings(config.new_keys)
        .expect("test New group is compatible")
        .try_close_key_bindings(config.close_keys)
        .expect("test Close group is compatible")
}

fn widget(config: Config, order: BuildOrder) -> TabsWidget {
    let base = Tabs::new(TABS)
        .allow_new(config.allow_new)
        .closable(config.closable)
        .digit_jump(config.digit_jump)
        .wrap(config.wrap)
        .passive(config.passive);
    match order {
        BuildOrder::ProfileBeforeGroups => with_groups(with_profile(base, config), config)
            .key(tab_key as fn(&Tab) -> ItemKey)
            .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)),
        BuildOrder::GroupsBeforeProfile => with_profile(with_groups(base, config), config)
            .key(tab_key as fn(&Tab) -> ItemKey)
            .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)),
        BuildOrder::ProfileAfterKey => with_profile(
            with_groups(base, config).key(tab_key as fn(&Tab) -> ItemKey),
            config,
        )
        .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)),
        BuildOrder::ProfileAfterRow => with_profile(
            with_groups(base, config)
                .row(tab_row as for<'a> fn(&Tab, &mut termrock_runtime::RowUi<'a>)),
            config,
        )
        .key(tab_key as fn(&Tab) -> ItemKey),
    }
}

fn default_row_widget(config: Config) -> Tabs<'static, Tab, fn(&Tab) -> ItemKey> {
    let tabs = Tabs::<Tab>::new(TABS)
        .key(tab_key as fn(&Tab) -> ItemKey)
        .allow_new(config.allow_new)
        .closable(config.closable)
        .digit_jump(config.digit_jump)
        .wrap(config.wrap)
        .passive(config.passive);
    let tabs = if config.set_profile {
        tabs.try_nav_profile(config.profile)
            .expect("profile and test binding groups are compatible")
    } else {
        tabs
    };
    tabs.try_new_key_bindings(config.new_keys)
        .expect("test New group is compatible")
        .try_close_key_bindings(config.close_keys)
        .expect("test Close group is compatible")
}

struct Page {
    config: Config,
    state: TabsState,
    items: Vec<Tab>,
    actions: Vec<TabsAction>,
    keymap: KeyMap,
    plan_profile: Option<Option<TabNavProfile>>,
    plan_rejected: Cell<bool>,
}

impl Page {
    fn new(config: Config) -> Self {
        Self {
            config,
            state: TabsState::default(),
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
            keymap: KeyMap::new(),
            plan_profile: None,
            plan_rejected: Cell::new(false),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tabs = widget(self.config, BuildOrder::ProfileBeforeGroups);
        let mut response = tabs.update(cx, &mut self.state, &self.items);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let tabs = widget(self.config, BuildOrder::ProfileBeforeGroups);
        if let Some(profile) = self.plan_profile {
            let planned_config = Config {
                profile,
                set_profile: true,
                ..self.config
            };
            let planned = default_row_widget(planned_config);
            let actual = default_row_widget(self.config);
            let layout = planned.layout_default(AREA, AREA, &self.state, &self.items, None);
            self.plan_rejected
                .set(actual.draw_layout(ui, &layout).is_err());
        } else if self.config.reference_only {
            ui.reference(None, |reference_ui| {
                tabs.draw(reference_ui, AREA, &self.state, &self.items)
            });
        } else {
            tabs.draw(ui, AREA, &self.state, &self.items);
        }
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: Page) -> Harness<Page> {
    Harness::new(page, Theme::junie(), 48, 8)
}

fn profile_config(profile: TabNavProfile) -> Config {
    Config {
        profile: Some(profile),
        set_profile: true,
        ..Config::default()
    }
}

fn has_action(tabs: &TabsWidget, action: &'static str) -> bool {
    tabs.bindings(BindingState::default())
        .iter()
        .any(|binding| binding.action == ActionKey::custom(action))
}

#[test]
fn unset_and_explicit_none_keep_the_current_default_table_cells_and_dispatch() {
    let legacy = Tabs::<Tab>::new(TABS).allow_new(true).closable(true);
    let legacy_table = legacy.bindings(BindingState::default());
    let explicit_none = legacy.try_nav_profile(None).unwrap();
    assert_eq!(
        legacy_table,
        explicit_none.bindings(BindingState::default())
    );

    let implicit = harness(Page::new(Config::default()));
    let explicit = harness(Page::new(Config {
        set_profile: true,
        ..Config::default()
    }));
    assert_eq!(implicit.buffer(), explicit.buffer());
    assert_eq!(implicit.ring().entries(), explicit.ring().entries());
    assert_eq!(implicit.focus(), explicit.focus());
    for action in [
        "tabs.previous",
        "tabs.previous-vim",
        "tabs.next",
        "tabs.next-vim",
        "tabs.activate",
        "tabs.activate-space",
        "tabs.tab-1",
        "tabs.tab-9",
        "tabs.close",
        "tabs.close-delete",
        "tabs.new",
    ] {
        assert!(has_action(
            &widget(Config::default(), BuildOrder::ProfileBeforeGroups),
            action
        ));
        assert!(has_action(
            &widget(
                Config {
                    set_profile: true,
                    ..Config::default()
                },
                BuildOrder::ProfileBeforeGroups
            ),
            action
        ));
    }

    let mut h = harness(Page::new(Config::default()));
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
    let _ = h.key_mod(KeyCode::Left, KeyModifiers::SHIFT);
    assert!(h.app().actions.is_empty(), "unset has no Shift+arrow alias");
    let _ = h.key(KeyCode::Char('H'));
    assert!(
        h.app().actions.is_empty(),
        "unset has no uppercase Vim alias"
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn desktop_and_vim_profiles_publish_and_dispatch_their_semantic_aliases() {
    let desktop = widget(
        profile_config(TabNavProfile::Desktop),
        BuildOrder::ProfileBeforeGroups,
    );
    let desktop_table = desktop.bindings(BindingState::default());
    assert_eq!(desktop_table[0].action, ActionKey::custom("tabs.previous"));
    assert_eq!(desktop_table[1].action, ActionKey::custom("tabs.next"));
    assert!(desktop_table[0].visible);
    assert_eq!(desktop_table[0].label, "Prev tab");
    assert_eq!(desktop_table[0].priority, 60);
    assert_eq!(
        desktop_table[2].action,
        ActionKey::custom("tabs.previous-shift-arrow")
    );
    assert_eq!(
        desktop_table[3].action,
        ActionKey::custom("tabs.next-shift-arrow")
    );
    assert!(!desktop_table[2].visible && desktop_table[2].priority == 10);
    assert!(!has_action(&desktop, "tabs.previous-vim"));
    assert!(!has_action(&desktop, "tabs.next-vim"));
    assert!(!has_action(&desktop, "tabs.previous-vim-upper"));
    assert!(!has_action(&desktop, "tabs.next-vim-upper"));

    let mut desktop_h = harness(Page::new(profile_config(TabNavProfile::Desktop)));
    let _ = desktop_h.key(KeyCode::Right);
    desktop_h.app_mut().actions.clear();
    let _ = desktop_h.key_mod(KeyCode::Left, KeyModifiers::SHIFT);
    assert_eq!(
        desktop_h.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(1))]
    );
    desktop_h.app_mut().actions.clear();
    let prior = desktop_h.app().state.active();
    for code in [
        KeyCode::Char('h'),
        KeyCode::Char('H'),
        KeyCode::Char('l'),
        KeyCode::Char('L'),
    ] {
        let _ = desktop_h.key(code);
    }
    assert!(desktop_h.app().actions.is_empty());
    assert_eq!(desktop_h.app().state.active(), prior);

    let vim = widget(
        profile_config(TabNavProfile::Vim),
        BuildOrder::ProfileBeforeGroups,
    );
    let vim_table = vim.bindings(BindingState::default());
    for action in [
        "tabs.previous",
        "tabs.next",
        "tabs.previous-vim",
        "tabs.next-vim",
        "tabs.previous-vim-upper",
        "tabs.next-vim-upper",
        "tabs.previous-shift-arrow",
        "tabs.next-shift-arrow",
    ] {
        assert!(has_action(&vim, action), "Vim profile publishes {action}");
    }
    assert!(vim_table[0].visible);
    assert!(vim_table[1].visible);
    assert!(
        vim_table[2..8].iter().all(|binding| !binding.visible),
        "aliases stay hidden while arrows remain truthful visible hints"
    );

    let mut vim_h = harness(Page::new(profile_config(TabNavProfile::Vim)));
    let _ = vim_h.key(KeyCode::Right);
    vim_h.app_mut().actions.clear();
    for (code, mods, expected) in [
        (KeyCode::Char('h'), KeyModifiers::NONE, 1),
        (KeyCode::Char('l'), KeyModifiers::SHIFT, 2),
        (KeyCode::Char('L'), KeyModifiers::NONE, 3),
        (KeyCode::Char('H'), KeyModifiers::SHIFT, 2),
        (KeyCode::Right, KeyModifiers::SHIFT, 3),
    ] {
        let _ = vim_h.key_mod(code, mods);
        assert_eq!(
            vim_h.app().actions.last(),
            Some(&TabsAction::Activated(ItemKey::Num(expected)))
        );
    }
    vim_h.app_mut().actions.clear();
    for mods in [
        KeyModifiers::CONTROL,
        KeyModifiers::ALT,
        KeyModifiers::SUPER,
    ] {
        let _ = vim_h.key_mod(KeyCode::Char('h'), mods);
    }
    assert!(
        vim_h.app().actions.is_empty(),
        "modified chords do not navigate"
    );
    assert!(vim_h.diagnostics().is_empty(), "{:?}", vim_h.diagnostics());
}

#[test]
fn profile_preserves_activation_digit_new_close_and_wrap_commands() {
    let config = Config {
        wrap: true,
        ..profile_config(TabNavProfile::Vim)
    };
    let mut h = harness(Page::new(config));
    let _ = h.key(KeyCode::Char('h'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(3))),
        "all Prev aliases use the existing wrap path"
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('L'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1))),
        "all Next aliases use the existing wrap path"
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('3'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(3)))
    );
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('n'));
    assert_eq!(h.app().actions.as_slice(), [TabsAction::New]);
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Delete);
    assert_eq!(
        h.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(3))]
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    let mut custom = harness(Page::new(Config {
        profile: Some(TabNavProfile::Vim),
        set_profile: true,
        new_keys: Some(&NEW_T),
        close_keys: Some(&CLOSE_DELETE),
        ..Config::default()
    }));
    let table = widget(
        Config {
            profile: Some(TabNavProfile::Vim),
            set_profile: true,
            new_keys: Some(&NEW_T),
            close_keys: Some(&CLOSE_DELETE),
            ..Config::default()
        },
        BuildOrder::GroupsBeforeProfile,
    );
    assert_eq!(table.new_key_table(), &NEW_T);
    assert_eq!(table.close_key_table(), &CLOSE_DELETE);
    let _ = custom.key(KeyCode::Char('n'));
    assert!(
        custom.app().actions.is_empty(),
        "the replaced New chord is absent"
    );
    let _ = custom.key(KeyCode::Char('t'));
    assert_eq!(custom.app().actions.as_slice(), [TabsAction::New]);
    custom.app_mut().actions.clear();
    let _ = custom.key(KeyCode::Char('x'));
    assert!(
        custom.app().actions.is_empty(),
        "the replaced Close chord is absent"
    );
    let _ = custom.key(KeyCode::Delete);
    assert_eq!(
        custom.app().actions.as_slice(),
        [TabsAction::Close(ItemKey::Num(1))]
    );
}

#[test]
fn remaps_and_stale_prior_profile_actions_are_filtered_by_the_current_table() {
    let mut page = Page::new(profile_config(TabNavProfile::Desktop));
    page.keymap = KeyMap::new()
        .bind_component(
            TABS,
            ActionKey::custom("tabs.previous"),
            Chord::key(KeyCode::F(10)),
        )
        .bind_component(
            TABS,
            ActionKey::custom("tabs.previous-vim"),
            Chord::key(KeyCode::F(11)),
        );
    let mut remapped = harness(page);
    let _ = remapped.key(KeyCode::Right);
    remapped.app_mut().actions.clear();
    let _ = remapped.key(KeyCode::F(10));
    assert_eq!(
        remapped.app().actions.as_slice(),
        [TabsAction::Activated(ItemKey::Num(1))],
        "a retained semantic action remains consumer-remappable"
    );
    remapped.app_mut().actions.clear();
    let _ = remapped.key(KeyCode::F(11));
    let _ = remapped.key(KeyCode::Char('h'));
    assert!(
        remapped.app().actions.is_empty(),
        "a removed Vim alias cannot be resurrected by a stale remap"
    );

    let mut queued = harness(Page::new(profile_config(TabNavProfile::Vim)));
    let before = queued.app().state.active();
    queued.app_mut().config.profile = Some(TabNavProfile::Desktop);
    queued.app_mut().config.set_profile = true;
    let _ = queued.key(KeyCode::Char('h'));
    assert!(
        queued.app().actions.is_empty(),
        "a prior-frame Vim action is not interpreted under the new Desktop table"
    );
    assert_eq!(queued.app().state.active(), before);
    queued.app_mut().config.profile = Some(TabNavProfile::Vim);
    queued.draw();
    let _ = queued.key(KeyCode::Char('h'));
    assert_eq!(
        queued.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1)))
    );
}

#[test]
fn profile_group_collisions_are_checked_in_both_setter_orders() {
    let none_new = Tabs::<Tab>::new(TABS)
        .try_new_key_bindings(Some(&NEW_SHIFT_LEFT))
        .unwrap();
    assert!(
        matches!(
            none_new.try_nav_profile(Some(TabNavProfile::Desktop)),
            Err(TabsKeyBindingsError::ConflictingChord { .. })
        ),
        "adding Desktop after a valid None-profile custom group must reject its collision"
    );
    let desktop = Tabs::<Tab>::new(TABS)
        .try_nav_profile(Some(TabNavProfile::Desktop))
        .unwrap();
    assert!(matches!(
        desktop.try_new_key_bindings(Some(&NEW_SHIFT_LEFT)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));

    let none_close = Tabs::<Tab>::new(TABS)
        .try_close_key_bindings(Some(&CLOSE_SHIFT_LEFT))
        .unwrap();
    assert!(matches!(
        none_close.try_nav_profile(Some(TabNavProfile::Vim)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    let vim = Tabs::<Tab>::new(TABS)
        .try_nav_profile(Some(TabNavProfile::Vim))
        .unwrap();
    assert!(matches!(
        vim.try_close_key_bindings(Some(&CLOSE_SHIFT_LEFT)),
        Err(TabsKeyBindingsError::ConflictingChord { .. })
    ));
    assert!(
        Tabs::<Tab>::new(TABS)
            .try_nav_profile(Some(TabNavProfile::Vim))
            .unwrap()
            .try_close_key_bindings(Some(&CLOSE_H))
            .is_err()
    );

    let desktop_h = Tabs::<Tab>::new(TABS)
        .try_nav_profile(Some(TabNavProfile::Desktop))
        .unwrap()
        .try_new_key_bindings(Some(&NEW_H))
        .unwrap();
    assert!(desktop_h.try_nav_profile(None).is_err());
    assert!(
        Tabs::<Tab>::new(TABS)
            .try_nav_profile(Some(TabNavProfile::Desktop))
            .unwrap()
            .try_new_key_bindings(Some(&NEW_H))
            .unwrap()
            .try_nav_profile(Some(TabNavProfile::Vim))
            .is_err()
    );
    assert!(
        Tabs::<Tab>::new(TABS)
            .try_nav_profile(Some(TabNavProfile::Vim))
            .unwrap()
            .try_new_key_bindings(Some(&NEW_H))
            .is_err()
    );
    assert!(
        Tabs::<Tab>::new(TABS)
            .try_nav_profile(Some(TabNavProfile::Vim))
            .unwrap()
            .try_new_key_bindings(Some(&NEW_UPPER_H))
            .is_err()
    );

    assert!(
        Tabs::<Tab>::new(TABS)
            .try_nav_profile(Some(TabNavProfile::Desktop))
            .unwrap()
            .try_new_key_bindings(Some(&SHIFTED_CHAR_ALIASES))
            .is_err()
    );
    assert!(
        Tabs::<Tab>::new(TABS)
            .try_nav_profile(Some(TabNavProfile::Desktop))
            .unwrap()
            .try_new_key_bindings(Some(&NEW_REUSED_ACTION))
            .is_err()
    );
}

#[test]
fn all_digit_chords_remain_reserved_when_digit_jumps_and_new_are_gated() {
    for digits in NEW_DIGITS {
        assert!(
            Tabs::<Tab>::new(TABS)
                .digit_jump(false)
                .allow_new(false)
                .closable(false)
                .passive(true)
                .try_nav_profile(Some(TabNavProfile::Desktop))
                .unwrap()
                .try_new_key_bindings(Some(digits))
                .is_err(),
            "a potential New binding cannot claim digit {:?}",
            digits[0].chord
        );
    }
}

#[test]
fn profile_survives_key_row_and_group_builder_orders_without_changing_layout() {
    let config = Config {
        profile: Some(TabNavProfile::Vim),
        set_profile: true,
        allow_new: true,
        closable: true,
        ..Config::default()
    };
    let widgets = [
        widget(config, BuildOrder::ProfileBeforeGroups),
        widget(config, BuildOrder::GroupsBeforeProfile),
        widget(config, BuildOrder::ProfileAfterKey),
        widget(config, BuildOrder::ProfileAfterRow),
    ];
    for tabs in &widgets {
        assert!(has_action(tabs, "tabs.previous-vim-upper"));
        assert!(format!("{tabs:?}").contains("nav_profile: Some(Vim)"));
    }
    for tabs in &widgets[1..] {
        assert_eq!(
            widgets[0].bindings(BindingState::default()),
            tabs.bindings(BindingState::default())
        );
    }

    let mut actual = Page::new(profile_config(TabNavProfile::Desktop));
    actual.plan_profile = Some(Some(TabNavProfile::Vim));
    let planned = harness(actual);
    let ordinary = harness(Page::new(profile_config(TabNavProfile::Desktop)));
    assert!(!planned.app().plan_rejected.get());
    for part in [
        PartRef::item(Part::TAB, ItemKey::Num(1)),
        PartRef::item(Part::TAB, ItemKey::Num(2)),
        PartRef::item(Part::CLOSE, ItemKey::Num(1)),
        PartRef::of(Part::NEW),
    ] {
        assert_eq!(
            planned.area_of_part(TABS, part),
            ordinary.area_of_part(TABS, part)
        );
    }
}

#[test]
fn passive_and_reference_profiles_do_not_publish_or_consume_navigation() {
    let passive_config = Config {
        passive: true,
        ..profile_config(TabNavProfile::Vim)
    };
    let passive_widget = widget(passive_config, BuildOrder::ProfileBeforeGroups);
    assert!(passive_widget.bindings(BindingState::default()).is_empty());
    assert!(passive_widget.new_key_table().is_empty());
    assert!(passive_widget.close_key_table().is_empty());
    let mut passive = harness(Page::new(passive_config));
    assert_ne!(passive.focus(), Some(TABS));
    let _ = passive.key(KeyCode::Char('l'));
    assert!(passive.app().actions.is_empty());
    let tab = passive
        .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(2)))
        .expect("passive profile still paints live pointer targets");
    let _ = passive.click(tab.x, tab.y);
    assert_eq!(
        passive.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(2)))
    );

    let reference_config = Config {
        reference_only: true,
        ..profile_config(TabNavProfile::Vim)
    };
    let mut reference = harness(Page::new(reference_config));
    assert_ne!(reference.focus(), Some(TABS));
    assert!(
        reference
            .area_of_part(TABS, PartRef::item(Part::TAB, ItemKey::Num(2)))
            .is_none()
    );
    let _ = reference.key(KeyCode::Char('l'));
    assert!(reference.app().actions.is_empty());
}

#[test]
fn reordered_stable_items_keep_adjacent_navigation_and_current_overflow_follow() {
    let mut h = harness(Page::new(profile_config(TabNavProfile::Vim)));
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::Num(2)));
    h.app_mut().items.reverse();
    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Char('l'));
    assert_eq!(
        h.app().actions.last(),
        Some(&TabsAction::Activated(ItemKey::Num(1))),
        "Next follows the current position of the stable cursor key after reorder"
    );
    assert_eq!(h.app().state.cursor(), Some(ItemKey::Num(1)));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
