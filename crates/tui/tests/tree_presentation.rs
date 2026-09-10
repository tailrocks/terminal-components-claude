//! Prefix geometry and cursor presentation remain opt-in and interaction-owned.
use junie_tui::{
    App, Cx, Id, ItemKey, KeyCode, MouseKind, Rect, Response, StateFlags, Theme, Tree,
    TreeBranchClick, TreeNode, TreeState, Ui,
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
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum DisabledCase {
    #[default]
    None,
    Child,
    Owner,
}
struct Page {
    state: TreeState,
    gap: u16,
    span: u16,
    cursor_selected: bool,
    custom: bool,
    disabled: DisabledCase,
    flat: bool,
    items: [&'static str; 2],
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
            disabled: DisabledCase::None,
            flat: false,
            items: ["branch", "child"],
            width: 20,
            seen: RefCell::new(Vec::new()),
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let disabled = |item: &&str| self.disabled == DisabledCase::Child && *item == "child";
        let hierarchy = |item: &&str| {
            if self.flat {
                TreeNode::leaf(0)
            } else {
                node(item)
            }
        };
        Tree::new(ID)
            .disabled(self.disabled == DisabledCase::Owner)
            .disabled_item(&disabled)
            .gutter_gap(self.gap)
            .disclosure_hit_width(self.span)
            .cursor_selected(self.cursor_selected)
            .key(key)
            .node(&hierarchy)
            .branch_click(TreeBranchClick::Choose)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let disabled = |item: &&str| self.disabled == DisabledCase::Child && *item == "child";
        let hierarchy = |item: &&str| {
            if self.flat {
                TreeNode::leaf(0)
            } else {
                node(item)
            }
        };
        let tree = Tree::new(ID)
            .disabled(self.disabled == DisabledCase::Owner)
            .disabled_item(&disabled)
            .gutter_gap(self.gap)
            .disclosure_hit_width(self.span)
            .cursor_selected(self.cursor_selected)
            .key(key)
            .node(&hierarchy)
            .branch_click(TreeBranchClick::Choose);
        let area = Rect::new(1, 1, self.width, 4);
        if self.custom {
            let renderer = |_: &mut Ui<'_>, _, flags, key, _: &&str| {
                self.seen.borrow_mut().push((key, flags));
            };
            tree.render_row(&renderer)
                .draw(ui, area, &self.state, &self.items);
        } else {
            tree.draw(ui, area, &self.state, &self.items);
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

fn has(h: &Harness<Page>, name: &str, flag: StateFlags) -> Option<bool> {
    h.app()
        .seen
        .borrow()
        .iter()
        .find(|(key, _)| *key == ItemKey::text(name))
        .map(|(_, flags)| flags.contains(flag))
}
#[test]
fn custom_row_and_disclosure_hover_project_to_one_key() {
    let mut h = Harness::new(
        Page {
            custom: true,
            ..Page::default()
        },
        Theme::junie(),
        24,
        8,
    );
    let _ = h.mouse(MouseKind::Move, 8, 1);
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(true));
    let _ = h.mouse(MouseKind::Move, 2, 1);
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(true));
    let _ = h.key(KeyCode::Right);
    let _ = h.mouse(MouseKind::Move, 8, 2);
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(false));
    assert_eq!(has(&h, "child", StateFlags::HOVERED), Some(true));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("branch")));
    let _ = h.mouse(MouseKind::Down, 8, 2);
    assert_eq!(has(&h, "branch", StateFlags::PRESSED), Some(false));
    assert_eq!(has(&h, "child", StateFlags::PRESSED), Some(true));
}
#[test]
fn default_row_hover_changes_only_the_hovered_rows_paint() {
    let mut h = Harness::new(Page::default(), Theme::junie(), 24, 8);
    let _ = h.key(KeyCode::Right);
    let before = h.cell(8, 2).bg;
    let first = h.cell(8, 1).clone();
    let _ = h.mouse(MouseKind::Move, 8, 2);
    assert_ne!(h.cell(8, 2).bg, before);
    assert_eq!(h.cell(8, 1), &first);
}
#[test]
fn disabled_rows_and_owner_never_paint_pressed() {
    let mut h = Harness::new(
        Page {
            custom: true,
            disabled: DisabledCase::Child,
            ..Page::default()
        },
        Theme::junie(),
        24,
        8,
    );
    let _ = h.key(KeyCode::Right);
    let _ = h.mouse(MouseKind::Down, 8, 2);
    assert_eq!(has(&h, "child", StateFlags::DISABLED), Some(true));
    assert_eq!(has(&h, "child", StateFlags::PRESSED), Some(false));
    h.app_mut().disabled = DisabledCase::Owner;
    let _ = h.tick();
    let _ = h.mouse(MouseKind::Move, 8, 1);
    assert_eq!(has(&h, "branch", StateFlags::DISABLED), Some(true));
    assert_eq!(has(&h, "branch", StateFlags::PRESSED), Some(false));
}

#[test]
fn keyboard_suppression_and_feedback_remain_runtime_owned() {
    let mut h = Harness::new(
        Page {
            custom: true,
            flat: true,
            ..Page::default()
        },
        Theme::junie(),
        24,
        8,
    );
    let _ = h.mouse(MouseKind::Move, 8, 1);
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(true));
    let _ = h.key(KeyCode::Down);
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(false));
    let _ = h.key(KeyCode::Enter);
    let _ = h.click(8, 2);
    assert!(h.next_deadline().is_some());
    assert_eq!(has(&h, "child", StateFlags::PRESSED), Some(true));
    assert_eq!(has(&h, "branch", StateFlags::PRESSED), Some(false));
    let _ = h.advance(std::time::Duration::from_secs(1));
    assert_eq!(has(&h, "child", StateFlags::PRESSED), Some(false));
    let _ = h.mouse(MouseKind::Move, 8, 1);
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(true));
}
#[test]
fn pointer_publication_after_reorder_targets_new_key_at_position() {
    let mut h = Harness::new(
        Page {
            custom: true,
            flat: true,
            ..Page::default()
        },
        Theme::junie(),
        24,
        8,
    );
    let _ = h.mouse(MouseKind::Move, 8, 2);
    assert_eq!(has(&h, "child", StateFlags::HOVERED), Some(true));
    h.app_mut().items.swap(0, 1);
    h.app_mut().state.invalidate();
    let _ = h.tick();
    assert_eq!(has(&h, "child", StateFlags::HOVERED), Some(false));
    assert_eq!(has(&h, "branch", StateFlags::HOVERED), Some(true));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("branch")));
}
