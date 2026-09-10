//! Tree cache ownership survives replacement and clone divergence.
#![cfg(test)]
use junie_tui::{App, Cx, Id, ItemKey, Response, Theme, Tree, TreeNode, TreeState, Ui};
use junie_tui_testing::Harness;
const ID: Id = Id::root("replacement.tree");
const ROWS: &[&str] = &["public", "public_child", "analytics", "analytics_child"];
fn tree() -> Tree<'static, &'static str> {
    Tree::new(ID).node(&|label: &&str| {
        let node = if label.ends_with("_child") {
            TreeNode::leaf(1)
        } else {
            TreeNode::parent(0)
        };
        node.keyed(ItemKey::text(label))
    })
}
fn state(schema: &str) -> TreeState {
    let mut s = TreeState::default();
    s.expand(ItemKey::text(schema));
    s
}
struct Page {
    state: TreeState,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        tree().update(cx, &mut self.state, ROWS).erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        tree().draw(ui, ui.full(), &self.state, ROWS);
    }
}
#[test]
fn replacing_equal_generation_state_rebuilds_visible_children() {
    let mut h = Harness::new(
        Page {
            state: state("public"),
        },
        Theme::junie(),
        50,
        12,
    );
    assert!(h.text().contains("public_child"));
    assert!(!h.text().contains("analytics_child"));
    h.app_mut().state = state("analytics");
    h.draw();
    assert!(h.text().contains("analytics_child"), "{}", h.text());
    assert!(!h.text().contains("public_child"));
}

#[global_allocator]
static ALLOCATOR: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;

#[test]
fn diverging_clones_with_equal_generations_do_not_alias() {
    let mut original = state("public");
    let mut sibling = original.clone();
    original.collapse(ItemKey::text("public"));
    original.expand(ItemKey::text("public"));
    sibling.collapse(ItemKey::text("public"));
    sibling.expand(ItemKey::text("analytics"));
    let mut h = Harness::new(Page { state: original }, Theme::junie(), 50, 12);
    assert!(h.text().contains("public_child"));
    h.app_mut().state = sibling;
    h.draw();
    assert!(!h.text().contains("public_child"));
    assert!(h.text().contains("analytics_child"));
}

#[test]
fn inverted_expansion_replacements_and_empty_state_do_not_alias() {
    let mut first = TreeState::new();
    first.expand_all();
    first.collapse(ItemKey::text("analytics"));
    let mut second = TreeState::new();
    second.expand_all();
    second.collapse(ItemKey::text("public"));
    let mut h = Harness::new(Page { state: first }, Theme::junie(), 50, 12);
    assert!(h.text().contains("public_child"));
    h.app_mut().state = second;
    h.draw();
    assert!(!h.text().contains("public_child"));
    assert!(h.text().contains("analytics_child"));
    h.app_mut().state = TreeState::new();
    h.draw();
    assert!(!h.text().contains("public_child"));
    assert!(!h.text().contains("analytics_child"));
}

#[test]
fn warmed_state_mutations_allocate_nothing() {
    let mut state = state("public");
    let before = junie_tui_testing::perf::allocs();
    for _ in 0..100 {
        state.toggle(ItemKey::text("public"));
    }
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}

#[test]
fn clone_divergence_allocates_one_lineage_then_reuses_it() {
    let original = state("public");
    let mut sibling = original.clone();
    let before = junie_tui_testing::perf::allocs();
    sibling.collapse(ItemKey::text("public"));
    assert_eq!(junie_tui_testing::perf::allocs() - before, 1);
    let before = junie_tui_testing::perf::allocs();
    for _ in 0..100 {
        sibling.toggle(ItemKey::text("public"));
    }
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
    assert!(original.is_expanded(ItemKey::text("public")));
}

#[test]
fn state_remains_const_constructible_send_and_sync() {
    const EMPTY: TreeState = TreeState::new();
    fn send_sync<T: Send + Sync>() {}
    send_sync::<TreeState>();
    assert!(EMPTY.expanded().is_empty());
}

#[test]
fn warmed_toggles_and_publication_allocate_nothing() {
    let mut h = Harness::new(
        Page {
            state: state("public"),
        },
        Theme::junie(),
        50,
        12,
    );
    for _ in 0..4 {
        h.app_mut().state.toggle(ItemKey::text("public"));
        h.draw();
    }
    let before = junie_tui_testing::perf::allocs();
    for _ in 0..100 {
        h.app_mut().state.toggle(ItemKey::text("public"));
        h.draw();
    }
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}
