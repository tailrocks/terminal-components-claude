//! Keyed tree navigation with stable branch expansion.

use tui_next::{
    Cx, Id, ItemKey, Rect, Response, RowUi, Tree, TreeAction, TreeNode, TreeState, Ui, id,
};

use crate::data::{TREE, TREE_LABELS};

use super::{Page, frame, lines};

const PROJECT: Id = id!("trees.project");

fn node_key(node: &TreeNode) -> ItemKey {
    node.key().unwrap_or(ItemKey::Num(0))
}

fn node_copy(node: &TreeNode) -> TreeNode {
    *node
}

fn node_label(node: &TreeNode) -> (&'static str, &'static str) {
    let index = match node.key() {
        Some(ItemKey::Num(key)) => key.saturating_sub(1) as usize,
        _ => usize::MAX,
    };
    TREE_LABELS.get(index).copied().unwrap_or(("unknown", ""))
}

fn node_row(node: &TreeNode, row: &mut RowUi<'_>) {
    let (label, meta) = node_label(node);
    row.label(label);
    row.meta(meta);
}

fn project_tree()
-> Tree<'static, TreeNode, impl Fn(&TreeNode) -> ItemKey, impl Fn(&TreeNode, &mut RowUi<'_>)> {
    Tree::new(PROJECT)
        .key(node_key)
        .node(&node_copy)
        .row(node_row)
}

/// Project navigation owns expansion by stable item key. No depth-derived key
/// can alias a sibling or move focus after a branch changes shape.
#[derive(Debug)]
pub(crate) struct TreesPage {
    state: TreeState,
    chosen: Option<ItemKey>,
    last: &'static str,
}

impl TreesPage {
    pub(crate) fn new() -> Self {
        let mut state = TreeState::new();
        // Match the legacy tree's first-level-open presentation. Descendants
        // remain closed until the user opens them, so keyboard expansion has
        // a deterministic, visible state transition.
        for key in [1_u64, 20, 26] {
            state.expand(ItemKey::Num(key));
        }
        Self {
            state,
            chosen: None,
            last: "project loaded",
        }
    }
}

impl Default for TreesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TreesPage {
    fn title(&self) -> &'static str {
        "Trees"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let result = project_tree().update(cx, &mut self.state, TREE);
        if let Some(action) = result.action_ref() {
            self.last = match action {
                TreeAction::Expanded(_) => "branch expanded",
                TreeAction::Collapsed(_) => "branch collapsed",
                TreeAction::Chose(_) | TreeAction::Activated(_) => "file selected",
                TreeAction::Moved => "cursor moved",
            };
            if let TreeAction::Chose(key) | TreeAction::Activated(key) = action {
                self.chosen = Some(*key);
            }
        }
        result.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "keyed branches · ←/→ expand · Enter select",
            |ui, body| {
                let tree_area = Rect {
                    height: body.height.saturating_sub(3),
                    ..body
                };
                project_tree().draw(ui, tree_area, &self.state, TREE);
                let selected = self
                    .chosen
                    .and_then(|key| match key {
                        ItemKey::Num(n) => TREE_LABELS.get(n.saturating_sub(1) as usize),
                        _ => None,
                    })
                    .map_or("none", |(name, _)| *name);
                let summary = format!(
                    "selected: {selected} · expanded: {} · {}",
                    self.state.expanded().len_in(TREE.len()),
                    self.last
                );
                let summary_area = Rect {
                    y: tree_area.bottom(),
                    height: 1,
                    ..body
                };
                let _ = ui.paint_str(summary_area, &summary, ui.surface_style());
                lines(
                    ui,
                    Rect {
                        y: summary_area.y.saturating_add(1),
                        height: 1,
                        ..body
                    },
                    &["Rows keep their identity while the visible pre-order window changes."],
                );
            },
        );
    }
}
