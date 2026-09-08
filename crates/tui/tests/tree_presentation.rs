//! Prefix geometry and cursor presentation remain opt-in and interaction-owned.
use junie_tui::{
    App, Cx, Id, ItemKey, KeyCode, Rect, Response, StateFlags, Theme, Tree, TreeBranchClick,
    TreeNode, TreeState, Ui,
};
use junie_tui_testing::Harness;
use std::cell::RefCell;
const ID: Id = Id::root("tree.presentation");
fn node(item: &&str) -> TreeNode {
    if *item == "branch" {
        TreeNode::parent(0)
    } else {
        TreeNode::leaf(1)
    }
}
fn key(item: &&str) -> ItemKey {
    ItemKey::text(item)
}
struct Page {
    state: TreeState,
    gap: u16,
    span: u16,
    cursor_selected: bool,
    custom: bool,
    width: u16,
    seen: RefCell<Vec<(ItemKey, StateFlags)>>,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            state: TreeState::new(),
            gap: 0,
            span: 1,
            cursor_selected: false,
            custom: false,
            width: 20,
            seen: RefCell::new(Vec::new()),
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Tree::new(ID)
            .gutter_gap(self.gap)
            .disclosure_hit_width(self.span)
            .cursor_selected(self.cursor_selected)
            .key(key)
            .node(&node)
            .branch_click(TreeBranchClick::Choose)
            .update(cx, &mut self.state, &["branch", "child"])
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let tree = Tree::new(ID)
            .gutter_gap(self.gap)
            .disclosure_hit_width(self.span)
            .cursor_selected(self.cursor_selected)
            .key(key)
            .node(&node)
            .branch_click(TreeBranchClick::Choose);
        let area = Rect::new(1, 1, self.width, 4);
        if self.custom {
            let renderer = |_: &mut Ui<'_>, _, flags, key, _: &&str| {
                self.seen.borrow_mut().push((key, flags));
            };
            tree.render_row(&renderer)
                .draw(ui, area, &self.state, &["branch", "child"]);
        } else {
            tree.draw(ui, area, &self.state, &["branch", "child"]);
        }
    }
}
#[test]
fn default_prefix_and_opt_in_gap_move_paint_and_hits_together() {
    let default = Harness::new(Page::default(), Theme::junie(), 24, 8);
    assert_eq!(default.cell(2, 1).symbol(), "▸");
    assert_eq!(default.cell(4, 1).symbol(), "b");
    for x in [3, 4] {
        let mut h = Harness::new(
            Page {
                gap: 1,
                span: 2,
                ..Page::default()
            },
            Theme::junie(),
            24,
            8,
        );
        assert_eq!(h.cell(2, 1).symbol(), " ");
        assert_eq!(h.cell(3, 1).symbol(), "▸");
        assert_eq!(h.cell(5, 1).symbol(), "b");
        let _ = h.click(x, 1);
        assert!(h.app().state.is_expanded(ItemKey::text("branch")));
    }
    let mut h = Harness::new(
        Page {
            gap: 1,
            span: 2,
            ..Page::default()
        },
        Theme::junie(),
        24,
        8,
    );
    let _ = h.click(2, 1);
    assert!(!h.app().state.is_expanded(ItemKey::text("branch")));
    assert_eq!(h.app().state.chosen(), Some(ItemKey::text("branch")));
}
#[test]
fn custom_renderer_uses_same_hit_span_and_zero_span_disables_it() {
    for span in [0, 2] {
        let mut h = Harness::new(
            Page {
                gap: 1,
                span,
                custom: true,
                ..Page::default()
            },
            Theme::junie(),
            24,
            8,
        );
        let _ = h.click(4, 1);
        assert_eq!(
            h.app().state.is_expanded(ItemKey::text("branch")),
            span == 2
        );
    }
}
#[test]
fn tiny_rows_and_saturating_gap_cannot_register_outside_row() {
    for width in [0, 1, 2] {
        let mut h = Harness::new(
            Page {
                width,
                gap: u16::MAX,
                span: u16::MAX,
                ..Page::default()
            },
            Theme::junie(),
            24,
            8,
        );
        let _ = h.click(4, 1);
        assert!(!h.app().state.is_expanded(ItemKey::text("branch")));
    }
}
fn selected(h: &Harness<Page>, name: &str) -> Option<bool> {
    h.app()
        .seen
        .borrow()
        .iter()
        .find(|(key, _)| *key == ItemKey::text(name))
        .map(|(_, flags)| flags.contains(StateFlags::SELECTED))
}
#[test]
fn cursor_presentation_does_not_write_chosen_and_overrides_stale_choice() {
    let mut h = Harness::new(
        Page {
            custom: true,
            cursor_selected: true,
            ..Page::default()
        },
        Theme::junie(),
        24,
        8,
    );
    assert_eq!(selected(&h, "branch"), Some(true));
    assert_eq!(h.app().state.chosen(), None);
    let _ = h.click(8, 1);
    assert_eq!(h.app().state.chosen(), Some(ItemKey::text("branch")));
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("child")));
    assert_eq!(selected(&h, "branch"), Some(false));
    assert_eq!(selected(&h, "child"), Some(true));
    assert_eq!(h.app().state.chosen(), Some(ItemKey::text("branch")));
    h.app_mut().cursor_selected = false;
    let _ = h.tick();
    assert_eq!(selected(&h, "branch"), Some(true));
    assert_eq!(selected(&h, "child"), Some(false));
}
