//! Repeat-selected activation uses semantic cursor identity, never click timing.
use junie_tui::{
    App, Cx, Id, ItemKey, KeyCode, MouseKind, Part, PartRef, Response, Theme, Tree, TreeAction,
    TreeBranchClick, TreeNode, TreeState, Ui,
};
use junie_tui_testing::Harness;
use std::time::Duration;
const ID: Id = Id::root("tree.selected.click");
#[derive(Clone, Copy)]
struct Entry {
    name: &'static str,
    branch: bool,
    disabled: bool,
}
impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}
fn key(e: &Entry) -> ItemKey {
    ItemKey::text(e.name)
}
fn node(e: &Entry) -> TreeNode {
    if e.branch {
        TreeNode::parent(0)
    } else {
        TreeNode::leaf(0)
    }
}
struct Page {
    state: TreeState,
    items: Vec<Entry>,
    enabled: bool,
    disabled: bool,
    action: Option<TreeAction>,
}
impl Page {
    fn new(enabled: bool) -> Self {
        Self {
            state: TreeState::new(),
            items: vec![
                Entry {
                    name: "branch",
                    branch: true,
                    disabled: false,
                },
                Entry {
                    name: "leaf",
                    branch: false,
                    disabled: false,
                },
                Entry {
                    name: "other",
                    branch: false,
                    disabled: false,
                },
            ],
            enabled,
            disabled: false,
            action: None,
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = Tree::new(ID)
            .key(key)
            .node(&node)
            .disabled_item(&|e| e.disabled)
            .branch_click(TreeBranchClick::Choose)
            .activate_selected_on_click(self.enabled)
            .disabled(self.disabled)
            .update(cx, &mut self.state, &self.items);
        self.action = r.action_ref().copied();
        r.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        Tree::new(ID)
            .key(key)
            .node(&node)
            .disabled_item(&|e| e.disabled)
            .branch_click(TreeBranchClick::Choose)
            .activate_selected_on_click(self.enabled)
            .disabled(self.disabled)
            .draw(ui, area, &self.state, &self.items);
    }
}
fn harness(enabled: bool) -> Harness<Page> {
    Harness::new(Page::new(enabled), Theme::junie(), 24, 6)
}
fn click(h: &mut Harness<Page>, name: &str) {
    let _ = h.click_part(ID, PartRef::item(Part::ROW, ItemKey::text(name)));
}
fn action(h: &Harness<Page>, expected: TreeAction) {
    assert_eq!(h.app().action, Some(expected));
}
#[test]
fn first_selected_branch_click_activates_without_toggling() {
    let mut h = harness(true);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("branch")));
    click(&mut h, "branch");
    action(&h, TreeAction::Activated(ItemKey::text("branch")));
    assert!(!h.app().state.is_expanded(ItemKey::text("branch")));
}
#[test]
fn unselected_leaf_chooses_then_delayed_repeat_activates() {
    let mut h = harness(true);
    click(&mut h, "leaf");
    action(&h, TreeAction::Chose(ItemKey::text("leaf")));
    let _ = h.advance(Duration::from_secs(10));
    click(&mut h, "leaf");
    action(&h, TreeAction::Activated(ItemKey::text("leaf")));
}
#[test]
fn keyboard_selection_then_first_click_activates() {
    let mut h = harness(true);
    let _ = h.key(KeyCode::Down);
    click(&mut h, "leaf");
    action(&h, TreeAction::Activated(ItemKey::text("leaf")));
}
#[test]
fn fast_second_click_after_cursor_change_only_chooses() {
    let mut h = harness(true);
    click(&mut h, "leaf");
    h.app_mut().state.set_cursor(2, ItemKey::text("other"));
    let _ = h.tick();
    click(&mut h, "leaf");
    action(&h, TreeAction::Chose(ItemKey::text("leaf")));
}
#[test]
fn default_slow_repeat_chooses_and_fast_double_click_activates() {
    let mut h = harness(false);
    click(&mut h, "leaf");
    let _ = h.advance(Duration::from_secs(10));
    click(&mut h, "leaf");
    action(&h, TreeAction::Chose(ItemKey::text("leaf")));
    click(&mut h, "leaf");
    action(&h, TreeAction::Activated(ItemKey::text("leaf")));
}
#[test]
fn disclosure_remains_expansion_only() {
    let mut h = harness(true);
    let _ = h.click_part(ID, PartRef::item(Part::ICON, ItemKey::text("branch")));
    action(&h, TreeAction::Expanded(ItemKey::text("branch")));
    let _ = h.advance(Duration::from_secs(1));
    let _ = h.click_part(ID, PartRef::item(Part::ICON, ItemKey::text("branch")));
    action(&h, TreeAction::Collapsed(ItemKey::text("branch")));
}
#[test]
fn drag_and_canceled_press_do_not_activate() {
    let mut h = harness(true);
    let _ = h.mouse(MouseKind::Down, 10, 0);
    let _ = h.mouse(MouseKind::Drag, 10, 2);
    let _ = h.mouse(MouseKind::Up, 10, 2);
    assert!(!matches!(h.app().action, Some(TreeAction::Activated(_))));
    let _ = h.mouse(MouseKind::Down, 10, 0);
    let _ = h.mouse(MouseKind::Up, 23, 5);
    assert!(!matches!(h.app().action, Some(TreeAction::Activated(_))));
    click(&mut h, "other");
    action(&h, TreeAction::Chose(ItemKey::text("other")));
}
#[test]
fn disabled_and_removed_rows_cannot_activate() {
    let mut h = harness(true);
    let _ = h.mouse(MouseKind::Down, 10, 0);
    for item in &mut h.app_mut().items {
        if item.name == "branch" {
            item.disabled = true;
        }
    }
    h.app_mut().state.invalidate();
    let _ = h.tick();
    let _ = h.mouse(MouseKind::Up, 10, 0);
    assert!(!matches!(h.app().action, Some(TreeAction::Activated(_))));
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("branch")));
    let _ = h.mouse(MouseKind::Down, 10, 1);
    h.app_mut().items.remove(1);
    let _ = h.tick();
    let _ = h.mouse(MouseKind::Up, 10, 1);
    assert!(!matches!(h.app().action, Some(TreeAction::Activated(_))));
    h.app_mut().disabled = true;
    let _ = h.tick();
    let _ = h.click(10, 1);
    assert!(!matches!(h.app().action, Some(TreeAction::Activated(_))));
}
#[test]
fn selection_survives_reorder_and_has_no_positional_activation() {
    let mut h = harness(true);
    let _ = h.key(KeyCode::Down);
    h.app_mut().items.swap(1, 2);
    h.app_mut().state.invalidate();
    let _ = h.tick();
    click(&mut h, "other");
    action(&h, TreeAction::Chose(ItemKey::text("other")));
    h.app_mut().items.swap(1, 2);
    h.app_mut().state.invalidate();
    let _ = h.tick();
    click(&mut h, "other");
    action(&h, TreeAction::Activated(ItemKey::text("other")));
}

#[test]
fn no_cursor_in_disabled_tree_cannot_activate() {
    let mut page = Page::new(true);
    for item in &mut page.items {
        item.disabled = true;
    }
    let mut h = Harness::new(page, Theme::junie(), 24, 6);
    assert_eq!(h.app().state.cursor(), None);
    let _ = h.click(10, 0);
    assert_eq!(h.app().state.cursor(), None);
    assert!(!matches!(h.app().action, Some(TreeAction::Activated(_))));
}
