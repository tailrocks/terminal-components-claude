use termrock_collections::SelectMode;
use termrock_core::{Id, ItemKey, KeyCode, KeyModifiers, Response};
use termrock_navigation::{List, ListState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::Theme;

const LIST: Id = Id::root("list.multi.range");
const ITEMS: [&str; 5] = ["alpha", "beta", "gamma", "delta", "epsilon"];

struct Page {
    state: ListState,
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
        list().update(cx, &mut self.state, &ITEMS).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        list().draw(ui, ui.full(), &self.state, &ITEMS);
    }
}

fn assert_state(h: &Harness<Page>, cursor: &str, checked: &[&str]) {
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text(cursor)));

    let actual = h.app().state.checked().keys();
    assert_eq!(actual.len(), checked.len());
    for item in ITEMS {
        let present = actual.iter().any(|key| key == &ItemKey::text(item));
        assert_eq!(present, checked.contains(&item), "checked keys: {actual:?}");
    }
}

#[test]
fn shift_extension_skips_disabled_and_plain_move_sets_a_fresh_anchor() {
    let mut h = Harness::new(
        Page {
            state: ListState::default(),
        },
        Theme::junie(),
        24,
        6,
    );
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
