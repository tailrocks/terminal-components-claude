use termrock_collections::SelectMode;
use termrock_core::{Id, ItemKey, KeyCode, Response};
use termrock_navigation::{List, ListAction, ListState};
use termrock_runtime::{App, Cx, Part, PartRef, Ui};
use termrock_test_support::Harness;
use termrock_theme::{GlyphRole, Theme};

const LIST: Id = Id::root("list.single.chosen-cursor");
const ITEMS: [&str; 3] = ["one", "two", "three"];

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
}

fn key(item: &&str) -> ItemKey {
    ItemKey::text(item)
}

fn list() -> List<'static, &'static str, fn(&&str) -> ItemKey> {
    List::new(LIST)
        .key(key as fn(&&str) -> ItemKey)
        .select_mode(SelectMode::Single)
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = list().update(cx, &mut self.state, &ITEMS);
        if let Some(action) = response.action_ref() {
            self.actions.push(*action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        list().draw(ui, ui.full(), &self.state, &ITEMS);
    }
}

fn harness() -> Harness<Page> {
    Harness::new(
        Page {
            state: ListState::default(),
            actions: Vec::new(),
        },
        Theme::junie(),
        24,
        6,
    )
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

    h.app_mut().actions.clear();
    let _ = h.key(KeyCode::Down);
    let move_actions = h.app().actions.clone();
    let cursor = h.app().state.cursor();

    assert_eq!(
        enter_actions.as_slice(),
        &[ListAction::Activated(two_key())]
    );
    assert_eq!(move_actions.as_slice(), &[ListAction::Moved]);
    assert_eq!(cursor, Some(three_key()));
    assert_eq!(h.focus(), Some(LIST));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
