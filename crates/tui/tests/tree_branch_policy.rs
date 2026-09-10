//! Public branch activation/click policies preserve shared hierarchy ownership.
use junie_tui::{
    App, Cx, Id, ItemKey, KeyCode, Part, PartRef, Response, Theme, Tree, TreeAction,
    TreeBranchActivation, TreeBranchClick, TreeNode, TreeState, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("tree.branch.policy");
fn key(item: &&str) -> ItemKey {
    ItemKey::text(item)
}
fn node(item: &&str) -> TreeNode {
    if *item == "branch" {
        TreeNode::parent(0)
    } else {
        TreeNode::leaf(1)
    }
}
struct Page {
    state: TreeState,
    activation: TreeBranchActivation,
    click: TreeBranchClick,
    disabled: bool,
    action: Option<TreeAction>,
}
impl Page {
    fn new(activation: TreeBranchActivation, click: TreeBranchClick) -> Self {
        Self {
            state: TreeState::new(),
            activation,
            click,
            disabled: false,
            action: None,
        }
    }
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let response = Tree::new(ID)
            .key(key)
            .node(&node)
            .branch_activation(self.activation)
            .branch_click(self.click)
            .disabled(self.disabled)
            .update(cx, &mut self.state, &["branch", "child"]);
        self.action = response.action_ref().copied();
        response.erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        Tree::new(ID)
            .key(key)
            .node(&node)
            .branch_activation(self.activation)
            .branch_click(self.click)
            .disabled(self.disabled)
            .draw(ui, area, &self.state, &["branch", "child"]);
    }
}
fn harness(activation: TreeBranchActivation, click: TreeBranchClick) -> Harness<Page> {
    Harness::new(Page::new(activation, click), Theme::junie(), 20, 5)
}
#[test]
fn default_enter_and_row_click_still_toggle_branches() {
    let mut h = harness(TreeBranchActivation::Toggle, TreeBranchClick::Toggle);
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().state.is_expanded(ItemKey::text("branch")));
    assert_eq!(
        h.app().action,
        Some(TreeAction::Expanded(ItemKey::text("branch")))
    );
    let _ = h.click_part(ID, PartRef::item(Part::ROW, ItemKey::text("branch")));
    assert!(!h.app().state.is_expanded(ItemKey::text("branch")));
    assert_eq!(
        h.app().action,
        Some(TreeAction::Collapsed(ItemKey::text("branch")))
    );
}
#[test]
fn activate_branch_does_not_change_expansion_and_space_still_toggles() {
    let mut h = harness(TreeBranchActivation::Activate, TreeBranchClick::Choose);
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().action,
        Some(TreeAction::Activated(ItemKey::text("branch")))
    );
    assert!(!h.app().state.is_expanded(ItemKey::text("branch")));
    let _ = h.key(KeyCode::Char(' '));
    assert!(h.app().state.is_expanded(ItemKey::text("branch")));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().action,
        Some(TreeAction::Activated(ItemKey::text("branch")))
    );
    assert!(h.app().state.is_expanded(ItemKey::text("branch")));
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.cursor(), Some(ItemKey::text("child")));
    let _ = h.key(KeyCode::Enter);
    assert_eq!(
        h.app().action,
        Some(TreeAction::Activated(ItemKey::text("child")))
    );
}
#[test]
fn choose_branch_click_does_not_toggle_but_disclosure_does() {
    let mut h = harness(TreeBranchActivation::Activate, TreeBranchClick::Choose);
    let _ = h.click_part(ID, PartRef::item(Part::ROW, ItemKey::text("branch")));
    assert_eq!(
        h.app().action,
        Some(TreeAction::Chose(ItemKey::text("branch")))
    );
    assert_eq!(h.app().state.chosen(), Some(ItemKey::text("branch")));
    assert!(!h.app().state.is_expanded(ItemKey::text("branch")));
    let _ = h.click_part(ID, PartRef::item(Part::ICON, ItemKey::text("branch")));
    assert!(h.app().state.is_expanded(ItemKey::text("branch")));
}
#[test]
fn policies_are_independent_and_disabled_owner_cannot_emit() {
    let mut h = harness(TreeBranchActivation::Activate, TreeBranchClick::Toggle);
    let _ = h.click_part(ID, PartRef::item(Part::ROW, ItemKey::text("branch")));
    assert!(h.app().state.is_expanded(ItemKey::text("branch")));
    h.app_mut().disabled = true;
    let _ = h.tick();
    let _ = h.key(KeyCode::Enter);
    assert_eq!(h.app().action, None);
    assert!(h.app().state.is_expanded(ItemKey::text("branch")));
}
