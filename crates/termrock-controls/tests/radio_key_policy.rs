use ratatui_core::layout::Rect;
use termrock_controls::choice::{
    InheritedFormState, RadioGroup, RadioGroupAction, RadioGroupState,
};
use termrock_controls::{ChoiceCmd, RadioKeyPolicy, RadioNavigation};
use termrock_core::action::ActionKey;
use termrock_core::event::Chord;
use termrock_core::id::{Id, ItemKey};
use termrock_core::keys::{KeyCode, KeyModifiers};
use termrock_runtime::{App, BindingState, Bindings, Cx, KeyMap, Response, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const RADIO: Id = Id::root("controls.radio.key-policy");
const AREA: Rect = Rect::new(2, 2, 30, 4);
const ITEMS: [&str; 3] = ["alpha", "beta", "gamma"];

struct RadioPage {
    items: Vec<&'static str>,
    selected: ItemKey,
    state: RadioGroupState,
    policy: RadioKeyPolicy,
    navigation: RadioNavigation,
    disabled: bool,
    keymap: KeyMap,
}

impl RadioPage {
    fn new(policy: RadioKeyPolicy, navigation: RadioNavigation) -> Self {
        Self {
            items: ITEMS.to_vec(),
            selected: ItemKey::text("beta"),
            state: RadioGroupState::default(),
            policy,
            navigation,
            disabled: false,
            keymap: KeyMap::new(),
        }
    }

    fn selected_item(&self) -> Option<&'static str> {
        self.items
            .iter()
            .copied()
            .find(|item| ItemKey::text(item) == self.selected)
    }
}

impl App for RadioPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let group = RadioGroup::new(RADIO)
            .key_policy(self.policy)
            .key(|item: &&'static str| ItemKey::text(item))
            .value(self.selected)
            .navigation(self.navigation)
            .disabled(self.disabled);
        let response = group.update(cx, &mut self.state, &self.items);
        if let Some(RadioGroupAction::Chose(key)) = response.action_ref() {
            self.selected = *key;
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        RadioGroup::new(RADIO)
            .key_policy(self.policy)
            .key(|item: &&'static str| ItemKey::text(item))
            .value(self.selected)
            .navigation(self.navigation)
            .disabled(self.disabled)
            .draw(ui, AREA, &self.state, &self.items);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(page: RadioPage) -> Harness<RadioPage> {
    Harness::new(page, Theme::junie(), 40, 8)
}

#[test]
fn default_profile_retains_existing_jk_and_arrow_navigation() {
    let mut h = harness(RadioPage::new(
        RadioKeyPolicy::Default,
        RadioNavigation::Immediate,
    ));
    assert_eq!(h.app().selected_item(), Some("beta"));

    let _ = h.key(KeyCode::Char('j'));
    assert_eq!(h.app().selected_item(), Some("gamma"));
    let _ = h.key(KeyCode::Up);
    assert_eq!(h.app().selected_item(), Some("beta"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn desktop_profile_publishes_and_interprets_only_desktop_navigation_keys() {
    let mut h = harness(RadioPage::new(
        RadioKeyPolicy::Desktop,
        RadioNavigation::Immediate,
    ));
    let _ = h.key(KeyCode::Char('j'));
    let _ = h.key(KeyCode::Char('k'));
    assert_eq!(h.app().selected_item(), Some("beta"));

    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().selected_item(), Some("gamma"));
    let _ = h.key(KeyCode::Home);
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().selected_item(), Some("gamma"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn vim_profile_handles_arrows_jk_uppercase_aliases_and_g_without_hl_or_modified_shortcuts() {
    let mut h = harness(RadioPage::new(
        RadioKeyPolicy::Vim,
        RadioNavigation::Immediate,
    ));
    let _ = h.key(KeyCode::Up);
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key(KeyCode::Char('k'));
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key_mod(KeyCode::Char('J'), KeyModifiers::SHIFT);
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key_mod(KeyCode::Char('K'), KeyModifiers::SHIFT);
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key(KeyCode::Char('j'));
    assert_eq!(h.app().selected_item(), Some("beta"));

    let _ = h.key(KeyCode::Char('h'));
    let _ = h.key(KeyCode::Char('l'));
    let _ = h.ctrl('j');
    assert_eq!(h.app().selected_item(), Some("beta"));

    let _ = h.key(KeyCode::Char('g'));
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key(KeyCode::Char('G'));
    assert_eq!(h.app().selected_item(), Some("gamma"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn no_home_end_rejects_default_home_end_and_vim_keys_but_keeps_remaps() {
    let mut page = RadioPage::new(RadioKeyPolicy::NoHomeEnd, RadioNavigation::Immediate);
    let mut keymap = KeyMap::new();
    keymap.remap_component(
        RADIO,
        ActionKey::custom("First"),
        Chord::with(KeyCode::Char('u'), KeyModifiers::ALT),
    );
    page.keymap = keymap;
    let mut h = harness(page);

    let bindings = RadioGroup::<&'static str>::new(RADIO)
        .key_policy(RadioKeyPolicy::NoHomeEnd)
        .bindings(BindingState::default());
    assert!(!bindings.iter().any(|binding| {
        binding.chord == Some(Chord::key(KeyCode::Home))
            || binding.chord == Some(Chord::key(KeyCode::End))
    }));
    assert!(bindings.iter().any(|binding| {
        binding.cmd == ChoiceCmd::First && binding.action == ActionKey::custom("First")
    }));

    let _ = h.key(KeyCode::Home);
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key_mod(KeyCode::Char('u'), KeyModifiers::ALT);
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key(KeyCode::Char('j'));
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key(KeyCode::Char('k'));
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key_mod(KeyCode::Char('J'), KeyModifiers::SHIFT);
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key_mod(KeyCode::Char('K'), KeyModifiers::SHIFT);
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key(KeyCode::Char('g'));
    assert_eq!(h.app().selected_item(), Some("beta"));
    let _ = h.key(KeyCode::Char('G'));
    assert_eq!(h.app().selected_item(), Some("beta"));

    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().selected_item(), Some("gamma"));
    let mut home_remap = KeyMap::new();
    home_remap.remap_component(RADIO, ActionKey::custom("First"), Chord::key(KeyCode::Home));
    h.app_mut().keymap = home_remap;
    let _ = h.key(KeyCode::Home);
    assert_eq!(h.app().selected_item(), Some("alpha"));
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().selected_item(), Some("alpha"));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn key_policy_does_not_change_immediate_or_deferred_navigation_timing() {
    let mut immediate = harness(RadioPage::new(
        RadioKeyPolicy::Vim,
        RadioNavigation::Immediate,
    ));
    let _ = immediate.key(KeyCode::Char('j'));
    assert_eq!(immediate.app().selected_item(), Some("gamma"));

    let mut deferred = harness(RadioPage::new(
        RadioKeyPolicy::Vim,
        RadioNavigation::Deferred,
    ));
    let _ = deferred.key(KeyCode::Char('j'));
    assert_eq!(deferred.app().selected_item(), Some("beta"));
    assert_eq!(deferred.app().state.cursor_index(), 2);
    let _ = deferred.key(KeyCode::Char(' '));
    assert_eq!(deferred.app().selected_item(), Some("gamma"));
}

#[test]
fn disabled_empty_and_reordered_radio_groups_keep_their_existing_state_contracts() {
    let mut disabled_page = RadioPage::new(RadioKeyPolicy::Vim, RadioNavigation::Immediate);
    disabled_page.disabled = true;
    let mut disabled = harness(disabled_page);
    let _ = disabled.key(KeyCode::Char('j'));
    let _ = disabled.key(KeyCode::Down);
    assert_eq!(disabled.app().selected_item(), Some("beta"));
    assert_ne!(disabled.focus(), Some(RADIO));

    let mut empty_page = RadioPage::new(RadioKeyPolicy::Vim, RadioNavigation::Immediate);
    empty_page.items.clear();
    let mut empty = harness(empty_page);
    let _ = empty.key(KeyCode::Char('g'));
    let _ = empty.key(KeyCode::Char('G'));
    assert_eq!(empty.app().selected, ItemKey::text("beta"));
    assert_eq!(empty.area_of(RADIO), None);

    let mut reordered = harness(RadioPage::new(
        RadioKeyPolicy::Vim,
        RadioNavigation::Immediate,
    ));
    let _ = reordered.key(KeyCode::Char('j'));
    assert_eq!(reordered.app().selected_item(), Some("gamma"));
    reordered.app_mut().items = vec!["gamma", "alpha", "beta"];
    let _ = reordered.key(KeyCode::Char('j'));
    assert_eq!(reordered.app().selected_item(), Some("alpha"));
    assert_eq!(reordered.app().state.cursor_index(), 1);
}

#[test]
fn radio_form_adapter_preserves_the_opt_in_key_policy() {
    struct FormRadioPage {
        selected: usize,
        state: RadioGroupState,
    }

    impl App for FormRadioPage {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            RadioGroup::new(RADIO)
                .key_policy(RadioKeyPolicy::Vim)
                .update_in_form(cx, &mut self.state, &mut self.selected, &ITEMS, false)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            RadioGroup::new(RADIO)
                .key_policy(RadioKeyPolicy::Vim)
                .draw_in_form(
                    ui,
                    AREA,
                    &self.state,
                    self.selected,
                    &ITEMS,
                    InheritedFormState::default(),
                );
        }
    }

    let mut h = Harness::new(
        FormRadioPage {
            selected: 1,
            state: RadioGroupState::default(),
        },
        Theme::junie(),
        40,
        8,
    );
    let _ = h.key(KeyCode::Char('j'));
    assert_eq!(h.app().selected, 2);
}
