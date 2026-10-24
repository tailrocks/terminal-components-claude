use termrock_collections::SelectMode;
use termrock_core::response::Flow;
use termrock_core::{Id, ItemKey, KeyCode, KeyModifiers, MouseKind, Response};
use termrock_navigation::{List, ListAction, ListState};
use termrock_runtime::{App, Cx, Part, PartRef, Rect, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("list.multi.range");
const ITEMS: [&str; 5] = ["alpha", "beta", "gamma", "delta", "epsilon"];

struct Page {
    state: ListState,
    actions: Vec<ListAction>,
}

fn key(item: &&str) -> ItemKey {
    ItemKey::text(item)
}

fn disabled(item: &&str) -> bool {
    *item == "gamma"
}

fn list() -> List<'static, &'static str, fn(&&str) -> ItemKey> {
    List::new(LIST)
        .key(key as fn(&&str) -> ItemKey)
        .select_mode(SelectMode::Multi)
        .disabled_item(&disabled)
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

fn assert_state(h: &Harness<Page>, cursor: &str, checked: &[&str]) {
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text(cursor)));

    let selection = h.app().state.checked();
    let actual = ITEMS
        .iter()
        .copied()
        .filter(|item| selection.contains(ItemKey::text(item)))
        .collect::<Vec<_>>();
    assert_eq!(
        selection.len_in(ITEMS.len()),
        checked.len(),
        "cursor {cursor}: checked items {actual:?}, expected {checked:?}"
    );
    assert_eq!(actual.as_slice(), checked, "cursor {cursor}");
}

fn row_area(h: &Harness<Page>, name: &str) -> Rect {
    h.area_of_part(LIST, PartRef::item(Part::ROW, ItemKey::text(name)))
        .expect("visible list row has a registered hit area")
}

#[test]
fn shift_extension_skips_disabled_and_plain_move_sets_a_fresh_anchor() {
    let mut h = harness();
    assert!(h.tab_to(LIST), "list should receive keyboard focus");

    let _ = h.key(KeyCode::Home);
    assert_state(&h, "alpha", &[]);

    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_state(&h, "beta", &["alpha", "beta"]);

    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_state(&h, "gamma", &["alpha", "beta"]);

    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_state(&h, "delta", &["alpha", "beta", "delta"]);

    let _ = h.key(KeyCode::Down);
    assert_state(&h, "epsilon", &["alpha", "beta", "delta"]);

    let _ = h.key_mod(KeyCode::Up, KeyModifiers::SHIFT);
    // Epsilon can only be added here when the plain move reset the range anchor.
    assert_state(&h, "delta", &["alpha", "beta", "delta", "epsilon"]);
}

#[test]
fn multi_commands_and_row_click_update_exact_selection() {
    let mut h = harness();
    assert!(h.tab_to(LIST), "list should receive keyboard focus");

    let _ = h.key(KeyCode::Home);
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    let _ = h.key(KeyCode::Down);
    let _ = h.key_mod(KeyCode::Up, KeyModifiers::SHIFT);
    assert_state(&h, "delta", &["alpha", "beta", "delta", "epsilon"]);

    h.app_mut().actions.clear();
    let space = h.key(KeyCode::Char(' '));
    assert_eq!(space.flow(), Flow::Consumed);
    assert_eq!(
        h.app().actions.as_slice(),
        &[ListAction::Toggled(ItemKey::text("delta"))]
    );
    assert_state(&h, "delta", &["alpha", "beta", "epsilon"]);

    let _ = h.key(KeyCode::Up);
    assert_state(&h, "gamma", &["alpha", "beta", "epsilon"]);
    h.app_mut().actions.clear();
    let disabled_space = h.key(KeyCode::Char(' '));
    assert_eq!(disabled_space.flow(), Flow::Consumed);
    assert!(
        h.app().actions.is_empty(),
        "disabled Space is consumed without a selection action"
    );
    assert_state(&h, "gamma", &["alpha", "beta", "epsilon"]);

    h.app_mut().actions.clear();
    let select_all = h.key(KeyCode::Char('a'));
    assert_eq!(select_all.flow(), Flow::Consumed);
    assert_eq!(h.app().actions.as_slice(), &[ListAction::ToggledAll]);
    assert_state(&h, "gamma", &["alpha", "beta", "delta", "epsilon"]);

    h.app_mut().actions.clear();
    let clear_all = h.key(KeyCode::Char('a'));
    assert_eq!(clear_all.flow(), Flow::Consumed);
    assert_eq!(h.app().actions.as_slice(), &[ListAction::ToggledAll]);
    assert_state(&h, "gamma", &[]);

    h.app_mut().actions.clear();
    let beta = row_area(&h, "beta");
    assert!(beta.width > 0 && beta.height > 0, "beta row is addressable");
    let click = h.click(beta.x.saturating_add(beta.width / 2), beta.y);
    assert_eq!(click.flow(), Flow::Consumed);
    assert_eq!(
        h.app().actions.as_slice(),
        &[ListAction::Toggled(ItemKey::text("beta"))]
    );
    assert_state(&h, "beta", &["beta"]);
}

#[test]
fn multi_cursor_check_and_hover_planes_remain_independent() {
    let mut h = harness();
    assert!(h.tab_to(LIST), "list should receive keyboard focus");
    let _ = h.key(KeyCode::Home);
    let _ = h.key_mod(KeyCode::Down, KeyModifiers::SHIFT);
    assert_state(&h, "beta", &["alpha", "beta"]);
    assert_eq!(
        h.app().actions.last(),
        Some(&ListAction::Toggled(ItemKey::text("beta")))
    );
    assert_eq!(h.focus(), Some(LIST));

    let beta = row_area(&h, "beta");
    let delta = row_area(&h, "delta");
    assert!(beta.width > 3 && beta.height > 0);
    assert!(delta.width > 5 && delta.height > 0);
    assert_eq!(h.cell(beta.x, beta.y).symbol(), "▎");
    assert_eq!(h.cell(beta.x.saturating_add(1), beta.y).symbol(), "✓");
    assert_eq!(h.cell(delta.x, delta.y).symbol(), " ");
    assert_eq!(h.cell(delta.x.saturating_add(1), delta.y).symbol(), " ");

    let hover_x = delta.x.saturating_add(4);
    let before_hover = h.cell(hover_x, delta.y).bg;
    let _ = h.mouse(MouseKind::Move, hover_x, delta.y);
    assert_eq!(h.hover(), Some(LIST), "the registered row receives hover");
    assert_ne!(
        h.cell(hover_x, delta.y).bg,
        before_hover,
        "hover changes the enabled delta row"
    );
    assert_eq!(h.focus(), Some(LIST));
    assert_state(&h, "beta", &["alpha", "beta"]);
    assert_eq!(h.cell(beta.x, beta.y).symbol(), "▎");
    assert_eq!(h.cell(beta.x.saturating_add(1), beta.y).symbol(), "✓");
    assert_eq!(h.cell(delta.x, delta.y).symbol(), " ");
    assert_eq!(h.cell(delta.x.saturating_add(1), delta.y).symbol(), " ");
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
