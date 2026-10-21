//! Ported from the PR #26 reference `tests/focus_gutter.rs`
//! (junie_tui at f4867e9b5f8a08a3bb61ce80d644198a29f51984), widget-level
//! sweep for the navigation crate.
//!
//! Kept from the reference: focus and selection stay distinguishable from
//! each other and from rest without colour — the focus bar glyph is in the
//! cursor row's gutter exactly while the widget holds focus, a checked row
//! keeps its mark at every tier, and the chosen row's accent fill follows
//! the frozen policy (accent while focused, plain surface once focus
//! leaves — the neutral inactive selection the frozen apps show).

use termrock_collections::collection::SelectMode;
use termrock_controls::Button;
use termrock_core::id::{Id, ItemKey};
use termrock_core::response::Response;
use termrock_navigation::{List, ListState, Tree, TreeNode, TreeState};
use termrock_runtime::{App, Cx, Ui};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, GlyphRole, Theme};

const LIST: Id = Id::root("focus.gutter.list");
const TREE: Id = Id::root("focus.gutter.tree");
const BTN: Id = Id::root("focus.gutter.button");

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const ITEMS: [&str; 3] = ["alpha", "beta", "gamma"];
const TREE_ITEMS: [&str; 3] = ["branch", "leaf", "other"];

#[derive(Default)]
struct ListPage {
    state: ListState,
}

impl App for ListPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let list =
            List::new(LIST)
                .select_mode(SelectMode::Multi)
                .update(cx, &mut self.state, &ITEMS);
        let _ = Button::new(BTN, "Other").update(cx);
        list.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        List::new(LIST).select_mode(SelectMode::Multi).draw(
            ui,
            termrock_runtime::Rect::new(0, 1, 16, 3),
            &self.state,
            &ITEMS,
        );
        Button::new(BTN, "Other").draw(ui, ui.full());
    }
}

struct TreePage {
    state: TreeState,
}

fn node(name: &&str) -> TreeNode {
    if *name == "branch" {
        TreeNode::parent(0)
    } else {
        TreeNode::leaf(0)
    }
}

impl App for TreePage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tree = Tree::new(TREE)
            .key(|e: &&str| ItemKey::text(e))
            .node(&node)
            .update(cx, &mut self.state, &TREE_ITEMS);
        let _ = Button::new(BTN, "Other").update(cx);
        tree.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Tree::new(TREE)
            .key(|e: &&str| ItemKey::text(e))
            .node(&node)
            .draw(
                ui,
                termrock_runtime::Rect::new(0, 1, 16, 3),
                &self.state,
                &TREE_ITEMS,
            );
        Button::new(BTN, "Other").draw(ui, ui.full());
    }
}

#[test]
fn list_focus_selection_and_checks_read_without_colour() {
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        let mut focused = Harness::new(
            ListPage {
                state: ListState::default(),
            },
            theme.clone(),
            16,
            4,
        );
        for _ in 0..4 {
            if focused.focus() == Some(LIST) {
                break;
            }
            let _ = focused.key(termrock_core::keys::KeyCode::Tab);
        }
        assert_eq!(
            focused.focus(),
            Some(LIST),
            "{level:?}: the list takes focus"
        );
        let list_top = 1u16;
        // Row families paint the bar while focused, without a focus
        // visibility requirement (unlike buttons).
        assert_eq!(
            focused.cell(0, list_top).symbol(),
            "▎",
            "{level:?}: the focused cursor row paints the bar"
        );
        assert_eq!(
            focused.cell(0, list_top + 1).symbol(),
            " ",
            "{level:?}: non-cursor rows keep a blank gutter"
        );

        let _ = focused.key(termrock_core::keys::KeyCode::Tab);
        assert_eq!(
            focused.focus(),
            Some(BTN),
            "{level:?}: focus moved to the button"
        );
        assert_eq!(
            focused.cell(0, list_top).symbol(),
            " ",
            "{level:?}: without focus the gutter is blank"
        );

        // A checked row keeps its mark at every tier.
        let checked_glyph = Theme::junie().design.glyphs.get(GlyphRole::Checked);
        let mut checked = Harness::new(
            ListPage {
                state: ListState::default(),
            },
            theme,
            16,
            4,
        );
        checked
            .app_mut()
            .state
            .checked_mut()
            .insert(ItemKey::index(1));
        checked.draw();
        let marked = (1..4u16).any(|y| checked.cell(1, y).symbol() == checked_glyph);
        assert!(marked, "{level:?}: the checked mark renders without colour");
    }
}

#[test]
fn the_chosen_rows_fill_follows_the_frozen_inactive_policy() {
    // Frozen: the chosen row carries the accent tint while the list holds
    // focus and reads as the plain surface once focus leaves (the neutral
    // inactive selection; see frozen_ink_pins for the resolver-level pin).
    let mut focused: Harness<ListPage> = Harness::new(ListPage::default(), Theme::junie(), 16, 4);
    focused.app_mut().state.choose(Some(ItemKey::index(1)));
    focused.app_mut().state.set_cursor(1, ItemKey::index(1));
    focused.draw();
    let chosen_row = 2u16;
    let focused_bg = focused.cell(4, chosen_row).bg;

    let _ = focused.key(termrock_core::keys::KeyCode::Tab);
    println!("DBG2 focus={:?}", focused.focus());
    let blurred_bg = focused.cell(4, chosen_row).bg;
    let rest_bg = focused.cell(4, chosen_row + 1).bg;
    assert_ne!(
        focused_bg, blurred_bg,
        "the focused chosen row is tinted, the unfocused one is not"
    );
    assert_eq!(
        blurred_bg, rest_bg,
        "an unfocused chosen row reads as the plain surface"
    );
}

#[test]
fn tree_focus_bar_sits_only_on_the_cursor_row() {
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        let mut focused = Harness::new(
            TreePage {
                state: TreeState::new(),
            },
            theme.clone(),
            16,
            4,
        );
        for _ in 0..4 {
            if focused.focus() == Some(TREE) {
                break;
            }
            let _ = focused.key(termrock_core::keys::KeyCode::Tab);
        }
        assert_eq!(
            focused.focus(),
            Some(TREE),
            "{level:?}: the tree takes focus"
        );
        assert_eq!(
            focused.cell(0, 1).symbol(),
            "▎",
            "{level:?}: the tree cursor row paints the bar"
        );
        assert_eq!(
            focused.cell(0, 2).symbol(),
            " ",
            "{level:?}: other rows keep a blank gutter"
        );

        let _ = focused.key(termrock_core::keys::KeyCode::Tab);
        assert_eq!(
            focused.focus(),
            Some(BTN),
            "{level:?}: focus moved to the button"
        );
        assert_eq!(
            focused.cell(0, 1).symbol(),
            " ",
            "{level:?}: without focus the tree gutters are blank"
        );
    }
}
