use termrock_collections::SelectMode;
use termrock_core::action::ActionKey;
use termrock_core::{Chord, Id, ItemKey, KeyCode, Response};
use termrock_navigation::{List, ListAction, ListState};
use termrock_runtime::{App, Cx, KeyMap, Part, PartRef, Ui};
use termrock_test_support::Harness;
use termrock_theme::{GlyphRole, Theme};

const LIST: Id = Id::root("list.single.chosen-cursor");
const ITEMS: [&str; 3] = ["one", "two", "three"];
const OVERFLOW_ITEMS: [&str; 8] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight",
];

fn one() -> ItemKey {
    ItemKey::text("one")
}

fn two_key() -> ItemKey {
    ItemKey::text("two")
}

fn three_key() -> ItemKey {
    ItemKey::text("three")
}

struct Page {
    state: ListState,
    actions: Vec<ListAction>,
    items: &'static [&'static str],
    mode: SelectMode,
    disable_all: bool,
    keymap: KeyMap,
}

fn key(item: &&str) -> ItemKey {
    ItemKey::text(item)
}

fn list(mode: SelectMode) -> List<'static, &'static str, fn(&&str) -> ItemKey> {
    List::new(LIST)
        .key(key as fn(&&str) -> ItemKey)
        .select_mode(mode)
}

fn disabled(_item: &&str) -> bool {
    true
}

impl Page {
    fn new(items: &'static [&'static str], mode: SelectMode) -> Self {
        Self {
            state: ListState::default(),
            actions: Vec::new(),
            items,
            mode,
            disable_all: false,
            keymap: KeyMap::new(),
        }
    }

    fn list(&self) -> List<'static, &'static str, fn(&&str) -> ItemKey> {
        let widget = list(self.mode);
        if self.disable_all {
            widget.disabled_item(&disabled)
        } else {
            widget
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = self.list().update(cx, &mut self.state, self.items);
        if let Some(action) = response.action_ref() {
            self.actions.push(*action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.list().draw(ui, ui.full(), &self.state, self.items);
    }

    fn keymap(&self) -> &KeyMap {
        &self.keymap
    }
}

fn harness() -> Harness<Page> {
    Harness::new(Page::new(&ITEMS, SelectMode::Single), Theme::junie(), 24, 6)
}

#[test]
fn single_space_chooses_two_while_the_cursor_moves_to_three() {
    let mut h = harness();
    assert!(
        h.tab_to(LIST),
        "the real list should receive keyboard focus"
    );
    assert_eq!(h.focus(), Some(LIST));
    assert_eq!(h.app().state.cursor(), Some(one()));

    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(two_key()));
    h.app_mut().actions.clear();

    let _ = h.key(KeyCode::Char(' '));
    let choose_actions = h.app().actions.clone();
    let chosen_after_choose = h.app().state.chosen();

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Down);
    let move_actions = h.app().actions.clone();
    let cursor_after_move = h.app().state.cursor();
    let chosen_after_move = h.app().state.chosen();
    h.draw();

    let two = h
        .area_of_part(LIST, PartRef::item(Part::ROW, two_key()))
        .expect("Two has a real registered row hit area");
    let three = h
        .area_of_part(LIST, PartRef::item(Part::ROW, three_key()))
        .expect("Three has a real registered row hit area");
    assert!(two.width > 0 && two.height > 0, "Two row bounds: {two:?}");
    assert!(
        three.width > 0 && three.height > 0,
        "Three row bounds: {three:?}"
    );

    let theme = Theme::junie();
    let chosen_glyph = theme.design.glyphs.get(GlyphRole::Chosen);
    let focus_glyph = theme.design.glyphs.get(GlyphRole::FocusBar);
    let two_gutter = h.cell(two.x, two.y).symbol();
    let two_marker = h.cell(two.x.saturating_add(1), two.y).symbol();
    let three_gutter = h.cell(three.x, three.y).symbol();
    let three_marker = h.cell(three.x.saturating_add(1), three.y).symbol();
    let focus = h.focus();
    let diagnostics_empty = h.diagnostics().is_empty();

    let checks = [
        (
            "Space reports Chose(Two)",
            choose_actions.as_slice() == [ListAction::Chose(two_key())],
        ),
        (
            "Space stores Two as chosen",
            chosen_after_choose == Some(two_key()),
        ),
        (
            "the following Down reports Moved",
            move_actions.as_slice() == [ListAction::Moved],
        ),
        (
            "the cursor moves to Three",
            cursor_after_move == Some(three_key()),
        ),
        (
            "the chosen key remains Two",
            chosen_after_move == Some(two_key()),
        ),
        ("Junie Chosen is the canonical glyph", chosen_glyph == "›"),
        ("Junie FocusBar is the canonical glyph", focus_glyph == "▎"),
        ("Two has no focus gutter", two_gutter == " "),
        ("Two retains its chosen marker", two_marker == chosen_glyph),
        ("Three has the focused gutter", three_gutter == focus_glyph),
        ("Three has no chosen marker", three_marker == " "),
        ("the List retains actual focus", focus == Some(LIST)),
        ("the run has no diagnostics", diagnostics_empty),
    ];
    let failed = checks
        .into_iter()
        .filter_map(|(check, passed)| (!passed).then_some(check))
        .collect::<Vec<_>>();
    assert!(
        failed.is_empty(),
        "failed checks: {failed:?}; Space actions={choose_actions:?}; move actions={move_actions:?}; chosen after Space={chosen_after_choose:?}; chosen after Down={chosen_after_move:?}; cursor after Down={cursor_after_move:?}; rows Two={two:?}, Three={three:?}; cells Two=({two_gutter:?}, {two_marker:?}), Three=({three_gutter:?}, {three_marker:?}); focus={focus:?}; diagnostics={:?}",
        h.diagnostics()
    );
}

#[test]
fn single_enter_activates_two_and_the_following_down_only_moves() {
    let mut h = harness();
    assert!(
        h.tab_to(LIST),
        "the real list should receive keyboard focus"
    );
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(two_key()));

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Enter);
    let enter_actions = h.app().actions.clone();
    let chosen_after_enter = h.app().state.chosen();

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Enter);
    let repeated_enter_actions = h.app().actions.clone();
    let chosen_after_repeated_enter = h.app().state.chosen();

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Down);
    let move_actions = h.app().actions.clone();
    let cursor = h.app().state.cursor();
    let chosen_after_move = h.app().state.chosen();
    h.draw();

    let two = h
        .area_of_part(LIST, PartRef::item(Part::ROW, two_key()))
        .expect("Two has a real registered row hit area");
    let three = h
        .area_of_part(LIST, PartRef::item(Part::ROW, three_key()))
        .expect("Three has a real registered row hit area");
    let theme = Theme::junie();
    let chosen_glyph = theme.design.glyphs.get(GlyphRole::Chosen);
    let focus_glyph = theme.design.glyphs.get(GlyphRole::FocusBar);
    let two_gutter = h.cell(two.x, two.y).symbol();
    let two_marker = h.cell(two.x.saturating_add(1), two.y).symbol();
    let three_gutter = h.cell(three.x, three.y).symbol();
    let three_marker = h.cell(three.x.saturating_add(1), three.y).symbol();

    assert_eq!(
        enter_actions.as_slice(),
        &[ListAction::Activated(two_key())]
    );
    assert_eq!(chosen_after_enter, Some(two_key()));
    assert_eq!(
        repeated_enter_actions.as_slice(),
        &[ListAction::Activated(two_key())]
    );
    assert_eq!(chosen_after_repeated_enter, Some(two_key()));
    assert_eq!(move_actions.as_slice(), &[ListAction::Moved]);
    assert_eq!(cursor, Some(three_key()));
    assert_eq!(chosen_after_move, Some(two_key()));
    assert_eq!(two_gutter, " ");
    assert_eq!(two_marker, chosen_glyph);
    assert_eq!(three_gutter, focus_glyph);
    assert_eq!(three_marker, " ");
    assert_eq!(h.focus(), Some(LIST));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().actions, [ListAction::Activated(three_key())]);
    assert_eq!(h.app().state.chosen(), Some(three_key()));
    assert_eq!(h.app().state.cursor(), Some(three_key()));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn remapped_activate_commits_single_choice_without_enter_fallback() {
    let mut page = Page::new(&ITEMS, SelectMode::Single);
    page.keymap = KeyMap::new().bind_component(
        LIST,
        ActionKey::custom("list.activate"),
        Chord::key(KeyCode::F(9)),
    );
    let mut h = Harness::new(page, Theme::junie(), 24, 6);
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::Down);
    assert_eq!(h.app().state.cursor(), Some(two_key()));
    h.app_mut().actions.clear();

    let _ = h.key(KeyCode::F(9));
    assert_eq!(h.app().actions, [ListAction::Activated(two_key())]);
    assert_eq!(h.app().state.chosen(), Some(two_key()));

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.app().actions.is_empty(),
        "Enter must not bypass the remap"
    );
    assert_eq!(h.app().state.chosen(), Some(two_key()));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn none_multi_and_range_activation_preserve_selection_state() {
    for mode in [SelectMode::None, SelectMode::Multi, SelectMode::Range] {
        let mut page = Page::new(&ITEMS, mode);
        page.state.choose(Some(two_key()));
        page.state.checked_mut().insert(three_key());
        let mut h = Harness::new(page, Theme::junie(), 24, 6);
        assert!(h.tab_to(LIST), "{mode:?} List should receive focus");
        h.app_mut().actions.clear();

        let _ = h.key(KeyCode::Enter);

        assert_eq!(h.app().actions, [ListAction::Activated(one())], "{mode:?}");
        assert_eq!(h.app().state.chosen(), Some(two_key()), "{mode:?}");
        assert!(h.app().state.checked().contains(three_key()), "{mode:?}");
        assert_eq!(h.app().state.cursor(), Some(one()), "{mode:?}");
        assert!(
            h.diagnostics().is_empty(),
            "{mode:?}: {:?}",
            h.diagnostics()
        );
    }
}

#[test]
fn disabled_and_empty_single_lists_do_not_activate_or_choose() {
    let mut disabled_page = Page::new(&ITEMS, SelectMode::Single);
    disabled_page.disable_all = true;
    let mut disabled = Harness::new(disabled_page, Theme::junie(), 24, 6);
    assert!(disabled.tab_to(LIST));
    let _ = disabled.key(KeyCode::Enter);
    assert!(disabled.app().actions.is_empty());
    assert_eq!(disabled.app().state.chosen(), None);
    assert!(
        disabled.diagnostics().is_empty(),
        "{:?}",
        disabled.diagnostics()
    );

    let mut empty = Harness::new(Page::new(&[], SelectMode::Single), Theme::junie(), 24, 6);
    assert!(empty.tab_to(LIST));
    let _ = empty.key(KeyCode::Enter);
    assert!(empty.app().actions.is_empty());
    assert_eq!(empty.app().state.chosen(), None);
    assert!(empty.diagnostics().is_empty(), "{:?}", empty.diagnostics());
}

#[test]
fn overflowing_single_activation_preserves_the_scroll_position() {
    let last = ItemKey::text("eight");
    let mut h = Harness::new(
        Page::new(&OVERFLOW_ITEMS, SelectMode::Single),
        Theme::junie(),
        24,
        4,
    );
    assert!(h.tab_to(LIST));
    let _ = h.key(KeyCode::End);
    assert_eq!(h.app().state.cursor(), Some(last));
    let before = h.app().state.scroll().offset();
    assert!(
        before > 0,
        "fixture should really overflow, offset={before}"
    );
    h.app_mut().actions.clear();

    let _ = h.key(KeyCode::Enter);

    assert_eq!(h.app().actions, [ListAction::Activated(last)]);
    assert_eq!(h.app().state.chosen(), Some(last));
    assert_eq!(h.app().state.scroll().offset(), before);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
