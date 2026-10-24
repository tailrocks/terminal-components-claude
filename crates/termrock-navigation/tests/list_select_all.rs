use core::cell::RefCell;

use termrock_collections::SelectMode;
use termrock_controls::Button;
use termrock_core::action::ActionKey;
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::{Flow, Response, StateFlags};
use termrock_core::{Chord, KeyCode, KeyModifiers};
use termrock_navigation::{List, ListAction, ListSelectionCmd, ListState};
use termrock_runtime::keymap::{Binding, Hint, HintKey, KeyMap};
use termrock_runtime::layer::LayerSpec;
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("list.select-all");
const OTHER: Id = Id::root("list.select-all.other");
const MODAL: Id = Id::root("list.select-all.modal");
const ROWS: [Row; 5] = [
    Row::new("alpha", false),
    Row::new("beta", false),
    Row::new("gamma", true),
    Row::new("delta", false),
    Row::new("epsilon", false),
];
const ALL_ENABLED: [Row; 5] = [
    Row::new("alpha", false),
    Row::new("beta", false),
    Row::new("gamma", false),
    Row::new("delta", false),
    Row::new("epsilon", false),
];
const CUSTOM_SELECTION: &[Binding<ListSelectionCmd>] = &[Binding {
    action: ActionKey::custom("list.select-all-custom"),
    chord: Some(Chord::key(KeyCode::F(9))),
    cmd: ListSelectionCmd::SelectAll,
    label: "Everything",
    priority: 70,
    visible: true,
}];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Row {
    name: &'static str,
    disabled: bool,
}

impl Row {
    const fn new(name: &'static str, disabled: bool) -> Self {
        Self { name, disabled }
    }
}

fn row_key(row: &Row) -> ItemKey {
    ItemKey::text(row.name)
}

fn row_disabled(row: &Row) -> bool {
    row.disabled
}

fn draw_row(row: &Row, ui: &mut termrock_runtime::RowUi<'_>) {
    ui.label(row.name);
}

type RowPainter = for<'r> fn(&Row, &mut termrock_runtime::RowUi<'r>);
type TestList = List<'static, Row, fn(&Row) -> ItemKey, RowPainter>;

#[derive(Clone, Copy)]
enum SelectionBindings {
    Defaults,
    Replace(&'static [Binding<ListSelectionCmd>]),
}

#[derive(Clone, Copy)]
enum NextUpdate {
    Mode(SelectMode),
    Bindings(&'static [Binding<ListSelectionCmd>]),
    EmptyItems,
}

struct Page {
    state: ListState,
    items: Vec<Row>,
    mode: SelectMode,
    selection: SelectionBindings,
    bindings_before_key_row: bool,
    actions: Vec<ListAction>,
    hints: RefCell<Vec<Hint>>,
    keymap: KeyMap,
    other_control: bool,
    widget_disabled: bool,
    reference_only: bool,
    modal_open: bool,
    next_update: Option<NextUpdate>,
}

impl Page {
    fn new(mode: SelectMode, items: &[Row]) -> Self {
        Self {
            state: ListState::default(),
            items: items.to_vec(),
            mode,
            selection: SelectionBindings::Defaults,
            bindings_before_key_row: false,
            actions: Vec::new(),
            hints: RefCell::new(Vec::new()),
            keymap: KeyMap::new(),
            other_control: false,
            widget_disabled: false,
            reference_only: false,
            modal_open: false,
            next_update: None,
        }
    }

    fn widget(&self) -> TestList {
        let list = List::new(LIST)
            .select_mode(self.mode)
            .disabled_item(&row_disabled);
        let list = match (self.selection, self.bindings_before_key_row) {
            (SelectionBindings::Defaults, _) => list,
            (SelectionBindings::Replace(bindings), true) => list.selection_bindings(bindings),
            (SelectionBindings::Replace(_), false) => list,
        };
        let list = list
            .key(row_key as fn(&Row) -> ItemKey)
            .row(draw_row as RowPainter);
        match (self.selection, self.bindings_before_key_row) {
            (SelectionBindings::Replace(bindings), false) => list.selection_bindings(bindings),
            _ => list,
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if let Some(change) = self.next_update.take() {
            match change {
                NextUpdate::Mode(mode) => self.mode = mode,
                NextUpdate::Bindings(bindings) => {
                    self.selection = SelectionBindings::Replace(bindings);
                }
                NextUpdate::EmptyItems => self.items.clear(),
            }
        }
        if self.modal_open && !cx.is_open(MODAL) {
            cx.open_layer(MODAL, LayerSpec::modal(MODAL).initial_focus(MODAL));
        }

        let response = self.widget().update(cx, &mut self.state, &self.items);
        if let Some(action) = response.action_ref() {
            self.actions.push(*action);
        }
        let mut response = response.erase();
        if self.other_control {
            response |= Button::new(OTHER, "Other").update(cx).erase();
        }
        if cx.is_open(MODAL) {
            response |= Button::new(MODAL, "Modal").update(cx).erase();
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.widget_disabled {
            ui.declare_state(LIST, StateFlags::DISABLED);
        }
        let list = self.widget();
        if self.reference_only {
            ui.reference(None, |ui| {
                list.draw(ui, ui.full(), &self.state, &self.items)
            });
        } else {
            list.draw(ui, ui.full(), &self.state, &self.items);
        }
        if self.other_control {
            Button::new(OTHER, "Other").draw(ui, ratatui_core::layout::Rect::new(1, 1, 12, 1));
        }
        if self.modal_open {
            ui.layer(MODAL, |ui, area| {
                Button::new(MODAL, "Modal").draw(ui, area);
            });
        }
        let hints = ui.with_focused_hints(|_, layer| layer.hints.clone());
        if let Some(hints) = hints {
            self.hints.replace(hints);
        } else {
            self.hints.borrow_mut().clear();
        }
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness(mode: SelectMode, items: &[Row], height: u16) -> Harness<Page> {
    Harness::new(Page::new(mode, items), Theme::junie(), 32, height)
}

fn assert_checked(h: &Harness<Page>, expected: &[&str]) {
    let actual = h
        .app()
        .items
        .iter()
        .filter(|row| h.app().state.checked().contains(row_key(row)))
        .map(|row| row.name)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    assert_eq!(
        h.app().state.checked().len_in(h.app().items.len()),
        expected.len()
    );
}

fn has_hint(h: &Harness<Page>, label: &str, chord: Chord) -> bool {
    h.app().hints.borrow().iter().any(|hint| {
        hint.label == label && matches!(hint.key, HintKey::Chord(found) if found == chord)
    })
}

fn assert_no_selection_hint(h: &Harness<Page>, context: &str) {
    assert!(
        !h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "Select all" || hint.label == "Everything"),
        "unexpected selection hints in {context}: {:?}",
        h.app().hints.borrow()
    );
}

fn assert_no_default_select_all_hint(h: &Harness<Page>) {
    assert!(
        !h.app()
            .hints
            .borrow()
            .iter()
            .any(|hint| hint.label == "Select all"),
        "unexpected default selection hint: {:?}",
        h.app().hints.borrow()
    );
}

fn indexed_name(index: usize) -> &'static str {
    match index {
        0 => "item-0",
        1 => "item-1",
        2 => "item-2",
        3 => "item-3",
        4 => "item-4",
        5 => "item-5",
        6 => "item-6",
        7 => "item-7",
        8 => "item-8",
        9 => "item-9",
        10 => "item-10",
        11 => "item-11",
        12 => "item-12",
        _ => "unexpected-item",
    }
}

#[test]
fn ctrl_and_super_select_all_are_idempotent_and_preserve_other_list_state() {
    for use_super in [false, true] {
        let mut h = harness(SelectMode::Multi, &ROWS, 3);
        assert!(h.tab_to(LIST));
        assert!(has_hint(
            &h,
            "Select all",
            Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL)
        ));
        assert_eq!(
            h.app()
                .hints
                .borrow()
                .iter()
                .filter(|hint| hint.label == "Select all")
                .count(),
            1,
            "one visible default hint represents the two distinct actions"
        );

        let _ = h.key(KeyCode::Home);
        let _ = h.key(KeyCode::Down);
        let _ = h.key(KeyCode::Char(' '));
        assert_checked(&h, &["beta"]);
        let _ = h.key(KeyCode::End);
        h.app_mut().state.choose(Some(ItemKey::text("beta")));
        let cursor = h.app().state.cursor();
        let chosen = h.app().state.chosen();
        let scroll = h.app().state.scroll().offset();
        assert!(scroll > 0, "fixture is scrolled before Select All");
        h.app_mut().actions.clear();

        let first = if use_super {
            h.key_mod(KeyCode::Char('a'), KeyModifiers::SUPER)
        } else {
            h.ctrl('a')
        };
        assert!(first.is_changed());
        assert_eq!(first.flow(), Flow::Consumed);
        assert_checked(&h, &["alpha", "beta", "delta", "epsilon"]);
        assert!(h.app().actions.is_empty());
        assert_eq!(h.app().state.cursor(), cursor);
        assert_eq!(h.app().state.chosen(), chosen);
        assert_eq!(h.app().state.scroll().offset(), scroll);

        let repeat = if use_super {
            h.key_mod(KeyCode::Char('a'), KeyModifiers::SUPER)
        } else {
            h.ctrl('a')
        };
        assert_eq!(repeat.flow(), Flow::Consumed);
        assert!(!repeat.is_changed());
        assert_checked(&h, &["alpha", "beta", "delta", "epsilon"]);
        assert!(h.app().actions.is_empty());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn plain_a_stays_toggle_all_and_emits_its_existing_action() {
    let mut h = harness(SelectMode::Multi, &ROWS, 4);
    assert!(h.tab_to(LIST));

    let first = h.key(KeyCode::Char('a'));
    assert!(first.is_changed());
    assert_eq!(h.app().actions, [ListAction::ToggledAll]);
    assert_checked(&h, &["alpha", "beta", "delta", "epsilon"]);

    h.app_mut().actions.clear();
    let second = h.key(KeyCode::Char('a'));
    assert!(second.is_changed());
    assert_eq!(h.app().actions, [ListAction::ToggledAll]);
    assert_checked(&h, &[]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn modes_empty_and_all_disabled_sources_have_distinct_select_all_eligibility() {
    let mut empty = harness(SelectMode::Multi, &[], 3);
    assert!(empty.tab_to(LIST));
    assert_no_selection_hint(&empty, "empty source");
    assert_eq!(empty.ctrl('a').flow(), Flow::Ignored);
    assert_checked(&empty, &[]);

    let all_disabled = [Row::new("disabled-a", true), Row::new("disabled-b", true)];
    let mut disabled = harness(SelectMode::Multi, &all_disabled, 3);
    assert!(disabled.tab_to(LIST));
    assert!(has_hint(
        &disabled,
        "Select all",
        Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL)
    ));
    let response = disabled.ctrl('a');
    assert_eq!(response.flow(), Flow::Consumed);
    assert!(!response.is_changed());
    assert_checked(&disabled, &[]);
    assert!(disabled.app().actions.is_empty());

    for mode in [SelectMode::Single, SelectMode::None] {
        let mut h = harness(mode, &ROWS, 4);
        assert!(h.tab_to(LIST));
        let _ = h.key(KeyCode::Home);
        h.app_mut().state.choose(Some(ItemKey::text("beta")));
        let cursor = h.app().state.cursor();
        let chosen = h.app().state.chosen();
        let checked = h.app().state.checked().clone();
        h.app_mut().actions.clear();
        assert_no_selection_hint(&h, "stale-action control");

        for modifiers in [KeyModifiers::CONTROL, KeyModifiers::SUPER] {
            let response = h.key_mod(KeyCode::Char('a'), modifiers);
            assert_eq!(response.flow(), Flow::Ignored);
            assert!(!response.is_changed());
            assert_eq!(h.app().state.cursor(), cursor);
            assert_eq!(h.app().state.chosen(), chosen);
            assert_eq!(h.app().state.checked(), &checked);
            assert!(h.app().actions.is_empty());
        }
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn custom_table_replaces_defaults_and_survives_both_builder_orders() {
    for bindings_before_key_row in [false, true] {
        let mut page = Page::new(SelectMode::Multi, &ROWS);
        page.selection = SelectionBindings::Replace(CUSTOM_SELECTION);
        page.bindings_before_key_row = bindings_before_key_row;
        let mut h = Harness::new(page, Theme::junie(), 32, 4);
        assert!(h.tab_to(LIST));
        assert!(has_hint(&h, "Everything", Chord::key(KeyCode::F(9))));
        assert_no_default_select_all_hint(&h);

        for modifiers in [KeyModifiers::CONTROL, KeyModifiers::SUPER] {
            let response = h.key_mod(KeyCode::Char('a'), modifiers);
            assert_eq!(response.flow(), Flow::Ignored);
            assert_checked(&h, &[]);
        }

        let custom = h.key(KeyCode::F(9));
        assert!(custom.is_changed());
        assert_checked(&h, &["alpha", "beta", "delta", "epsilon"]);
        assert!(h.app().actions.is_empty());
        let repeat = h.key(KeyCode::F(9));
        assert_eq!(repeat.flow(), Flow::Consumed);
        assert!(!repeat.is_changed());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }

    let mut disabled = Page::new(SelectMode::Multi, &ROWS);
    disabled.selection = SelectionBindings::Replace(&[]);
    let mut h = Harness::new(disabled, Theme::junie(), 32, 4);
    assert!(h.tab_to(LIST));
    assert_no_selection_hint(&h, "disabled selection table");
    assert_eq!(h.ctrl('a').flow(), Flow::Ignored);
    assert_eq!(
        h.key_mod(KeyCode::Char('a'), KeyModifiers::SUPER).flow(),
        Flow::Ignored
    );
    assert_checked(&h, &[]);
    assert!(h.app().actions.is_empty());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn component_keymap_remaps_and_unbinds_each_default_action_without_fallback() {
    let mut remapped = Page::new(SelectMode::Multi, &ROWS);
    remapped.keymap = KeyMap::new().bind_component(
        LIST,
        ActionKey::custom("list.select-all"),
        Chord::key(KeyCode::F(9)),
    );
    let mut h = Harness::new(remapped, Theme::junie(), 32, 4);
    assert!(h.tab_to(LIST));
    assert!(has_hint(&h, "Select all", Chord::key(KeyCode::F(9))));
    assert!(!has_hint(
        &h,
        "Select all",
        Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL)
    ));
    assert_eq!(h.ctrl('a').flow(), Flow::Ignored);
    assert_checked(&h, &[]);
    assert!(h.key(KeyCode::F(9)).is_changed());
    assert_checked(&h, &["alpha", "beta", "delta", "epsilon"]);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    for (removed, removed_chord, remaining, remaining_modifiers) in [
        (
            "list.select-all",
            KeyModifiers::CONTROL,
            "list.select-all-super",
            KeyModifiers::SUPER,
        ),
        (
            "list.select-all-super",
            KeyModifiers::SUPER,
            "list.select-all",
            KeyModifiers::CONTROL,
        ),
    ] {
        let mut page = Page::new(SelectMode::Multi, &ROWS);
        let mut keymap = KeyMap::new();
        keymap.remove_component(LIST, ActionKey::custom(removed));
        page.keymap = keymap;
        let mut h = Harness::new(page, Theme::junie(), 32, 4);
        assert!(h.tab_to(LIST));
        let disabled = h.key_mod(KeyCode::Char('a'), removed_chord);
        assert_eq!(disabled.flow(), Flow::Ignored);
        assert_checked(&h, &[]);
        assert!(
            h.key_mod(KeyCode::Char('a'), remaining_modifiers)
                .is_changed()
        );
        assert_checked(&h, &["alpha", "beta", "delta", "epsilon"]);
        assert_ne!(removed, remaining);
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn stale_select_all_actions_do_not_survive_mode_or_override_changes() {
    for change in [
        NextUpdate::Mode(SelectMode::Single),
        NextUpdate::Bindings(&[]),
        NextUpdate::EmptyItems,
    ] {
        let mut h = harness(SelectMode::Multi, &ROWS, 4);
        assert!(h.tab_to(LIST));
        assert!(has_hint(
            &h,
            "Select all",
            Chord::with(KeyCode::Char('a'), KeyModifiers::CONTROL)
        ));
        h.app_mut().next_update = Some(change);

        // Runtime resolves this key against the just-published Multi table;
        // App::update then switches the live widget before it interprets the
        // semantic action.
        let response = h.ctrl('a');
        assert_eq!(response.flow(), Flow::Ignored);
        assert!(!response.is_changed());
        assert_checked(&h, &[]);
        assert!(h.app().actions.is_empty());
        assert_no_selection_hint(&h, "empty selection override");
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}

#[test]
fn select_all_keeps_range_anchor_and_toggle_all_clear_is_a_control() {
    let mut with_select_all = harness(SelectMode::Range, &ALL_ENABLED, 5);
    assert!(with_select_all.tab_to(LIST));
    let _ = with_select_all.key(KeyCode::Home);
    let _ = with_select_all.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_checked(&with_select_all, &["alpha", "beta"]);
    assert_eq!(
        with_select_all.app().state.cursor(),
        Some(ItemKey::text("beta"))
    );
    with_select_all.app_mut().actions.clear();

    assert!(with_select_all.ctrl('a').is_changed());
    assert_checked(
        &with_select_all,
        &["alpha", "beta", "gamma", "delta", "epsilon"],
    );
    assert_eq!(
        with_select_all.app().state.cursor(),
        Some(ItemKey::text("beta"))
    );
    assert!(with_select_all.app().actions.is_empty());

    let cleared = with_select_all.key(KeyCode::Char('a'));
    assert!(cleared.is_changed());
    assert_checked(&with_select_all, &[]);
    assert_eq!(
        with_select_all.app().state.cursor(),
        Some(ItemKey::text("beta"))
    );
    assert!(
        with_select_all
            .key_mod(KeyCode::Down, KeyModifiers::SHIFT)
            .is_changed()
    );
    assert_checked(&with_select_all, &["alpha", "beta", "gamma"]);
    assert_eq!(
        with_select_all.app().state.cursor(),
        Some(ItemKey::text("gamma"))
    );

    let mut toggle_control = harness(SelectMode::Range, &ALL_ENABLED, 5);
    assert!(toggle_control.tab_to(LIST));
    let _ = toggle_control.key(KeyCode::Home);
    let _ = toggle_control.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_checked(&toggle_control, &["alpha", "beta"]);
    let _ = toggle_control.key(KeyCode::Char('a'));
    assert_checked(
        &toggle_control,
        &["alpha", "beta", "gamma", "delta", "epsilon"],
    );
    let _ = toggle_control.key(KeyCode::Char('a'));
    assert_checked(&toggle_control, &[]);
    let _ = toggle_control.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_checked(&toggle_control, &["alpha", "beta", "gamma"]);
    assert!(
        toggle_control.diagnostics().is_empty(),
        "{:?}",
        toggle_control.diagnostics()
    );
}

#[test]
fn selection_uses_stable_keys_and_select_all_does_not_move_overflow_state() {
    let original = (0..12)
        .map(|i| Row::new(indexed_name(i), false))
        .collect::<Vec<_>>();
    let mut h = harness(SelectMode::Multi, &original, 3);
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    h.app_mut().state.choose(Some(ItemKey::text("item-5")));
    let cursor = h.app().state.cursor();
    let chosen = h.app().state.chosen();
    let scroll = h.app().state.scroll().offset();
    assert!(scroll > 0);
    h.app_mut().actions.clear();

    assert!(h.ctrl('a').is_changed());
    let all = original.iter().map(|row| row.name).collect::<Vec<_>>();
    assert_checked(&h, &all);
    assert_eq!(h.app().state.cursor(), cursor);
    assert_eq!(h.app().state.chosen(), chosen);
    assert_eq!(h.app().state.scroll().offset(), scroll);
    assert!(h.app().actions.is_empty());

    let reordered = [
        "item-11", "item-0", "item-10", "item-9", "item-8", "item-7", "item-6", "item-5", "item-4",
        "item-3", "item-1", "item-12",
    ];
    h.app_mut().items = reordered.iter().map(|name| Row::new(name, false)).collect();
    let _ = h.key(KeyCode::F(12));
    assert_checked(&h, &reordered);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("item-11")));
    assert_eq!(h.app().state.chosen(), Some(ItemKey::text("item-5")));

    let before_repeat = h.app().state.scroll().offset();
    let repeat = h.ctrl('a');
    assert_eq!(repeat.flow(), Flow::Consumed);
    assert!(!repeat.is_changed());
    assert_checked(&h, &reordered);
    assert_eq!(h.app().state.scroll().offset(), before_repeat);

    let mut removed = harness(SelectMode::Multi, &ROWS, 4);
    assert!(removed.tab_to(LIST));
    let _ = removed.key(KeyCode::Home);
    let _ = removed.key(KeyCode::Char(' '));
    assert_checked(&removed, &["alpha"]);
    removed.app_mut().items.retain(|row| row.name != "alpha");
    let _ = removed.key(KeyCode::F(12));
    assert_checked(&removed, &[]);
    assert!(
        !removed
            .app()
            .state
            .checked()
            .contains(ItemKey::text("alpha"))
    );
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    assert!(
        removed.diagnostics().is_empty(),
        "{:?}",
        removed.diagnostics()
    );
}

#[test]
fn other_focus_modal_disabled_and_reference_contexts_do_not_act_on_list() {
    let mut other = harness(SelectMode::Multi, &ROWS, 4);
    other.app_mut().other_control = true;
    other.draw();
    assert!(other.tab_to(OTHER));
    other.draw();
    assert_eq!(other.focus(), Some(OTHER));
    assert_no_selection_hint(&other, "other focused control");
    assert_eq!(other.ctrl('a').flow(), Flow::Ignored);
    assert_checked(&other, &[]);

    let mut modal = harness(SelectMode::Multi, &ROWS, 8);
    modal.app_mut().modal_open = true;
    let _ = modal.key(KeyCode::F(12));
    assert!(modal.is_open(MODAL));
    assert!(modal.tab_to(MODAL));
    modal.draw();
    assert_eq!(modal.focus(), Some(MODAL));
    assert_no_selection_hint(&modal, "modal focus");
    assert_eq!(modal.ctrl('a').flow(), Flow::Ignored);
    assert_checked(&modal, &[]);

    let mut disabled = harness(SelectMode::Multi, &ROWS, 4);
    disabled.app_mut().widget_disabled = true;
    disabled.draw();
    disabled.draw();
    assert!(disabled.state_of(LIST).contains(StateFlags::DISABLED));
    assert_no_selection_hint(&disabled, "disabled list");
    assert_eq!(disabled.ctrl('a').flow(), Flow::Ignored);
    assert_checked(&disabled, &[]);

    let mut reference = harness(SelectMode::Multi, &ROWS, 4);
    reference.app_mut().reference_only = true;
    reference.draw();
    assert_no_selection_hint(&reference, "reference context");
    assert_eq!(reference.ctrl('a').flow(), Flow::Ignored);
    assert_checked(&reference, &[]);

    for h in [&other, &modal, &disabled, &reference] {
        assert!(h.app().actions.is_empty());
        assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    }
}
