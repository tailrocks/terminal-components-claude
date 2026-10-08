//! Keyed tree navigation with stable branch expansion.

use termrock::{
    Cx, FgStep, FrameRead, Id, ItemKey, Panel, PanelKind, Part, Rect, Role, RowUi, StateFlags,
    StylePatch, Tree, TreeAction, TreeNode, TreeState, Ui, id, truncate,
};

use showcase_data::TREE_LABELS;

pub const TREE: &[TreeNode] = &[
    TreeNode::parent(0).keyed(ItemKey::Num(1)),
    TreeNode::parent(1).keyed(ItemKey::Num(2)),
    TreeNode::leaf(2).keyed(ItemKey::Num(3)),
    TreeNode::leaf(2).keyed(ItemKey::Num(4)),
    TreeNode::leaf(2).keyed(ItemKey::Num(5)),
    TreeNode::parent(2).keyed(ItemKey::Num(6)),
    TreeNode::leaf(3).keyed(ItemKey::Num(7)),
    TreeNode::leaf(3).keyed(ItemKey::Num(8)),
    TreeNode::leaf(3).keyed(ItemKey::Num(9)),
    TreeNode::parent(1).keyed(ItemKey::Num(10)),
    TreeNode::leaf(2).keyed(ItemKey::Num(11)),
    TreeNode::leaf(2).keyed(ItemKey::Num(12)),
    TreeNode::leaf(2).keyed(ItemKey::Num(13)),
    TreeNode::parent(1).keyed(ItemKey::Num(14)),
    TreeNode::leaf(2).keyed(ItemKey::Num(15)),
    TreeNode::leaf(2).keyed(ItemKey::Num(16)),
    TreeNode::leaf(1).keyed(ItemKey::Num(17)),
    TreeNode::leaf(1).keyed(ItemKey::Num(18)),
    TreeNode::leaf(1).keyed(ItemKey::Num(19)),
    TreeNode::parent(0).keyed(ItemKey::Num(20)),
    TreeNode::leaf(1).keyed(ItemKey::Num(21)),
    TreeNode::leaf(1).keyed(ItemKey::Num(22)),
    TreeNode::parent(1).keyed(ItemKey::Num(23)),
    TreeNode::leaf(2).keyed(ItemKey::Num(24)),
    TreeNode::leaf(2).keyed(ItemKey::Num(25)),
    TreeNode::parent(0).keyed(ItemKey::Num(26)),
    TreeNode::leaf(1).keyed(ItemKey::Num(27)),
    TreeNode::leaf(1).keyed(ItemKey::Num(28)),
    TreeNode::leaf(0).keyed(ItemKey::Num(29)),
    TreeNode::leaf(0).keyed(ItemKey::Num(30)),
];

use super::{Page, PageUpdate, frame};

const PROJECT: Id = id!("trees.project");
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::DETAIL,
    StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
)];

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
    if !node.has_children() {
        row.meta(meta);
    }
}

fn project_tree(
    parts: &'static [(Part, StylePatch)],
) -> Tree<'static, TreeNode, impl Fn(&TreeNode) -> ItemKey, impl Fn(&TreeNode, &mut RowUi<'_>)> {
    Tree::new(PROJECT)
        .key(node_key)
        .node(&node_copy)
        .row(node_row)
        .patch_part(parts)
}

/// The one project-card constructor (§13), shared by update and draw.
fn project_panel(meta: &str, focused: bool) -> Panel<'_> {
    Panel::new(PROJECT)
        .kind(PanelKind::Card)
        .title("Project")
        .meta(meta)
        .focused(focused)
        .patch_part(PANEL_PARTS)
}

fn position_label(state: &TreeState, viewport_h: usize) -> String {
    let scroll = state.scroll();
    let content_len = scroll.content_len();
    let viewport = if scroll.viewport_len() > 0 {
        scroll.viewport_len()
    } else {
        viewport_h
    };
    if content_len <= viewport || viewport == 0 {
        return String::new();
    }
    let offset = scroll.offset();
    let start = offset.saturating_add(1);
    let end = (offset.saturating_add(viewport)).min(content_len);
    format!("{start}–{end} of {content_len}")
}

fn columns(area: Rect, left_w: u16, gap: u16) -> (Rect, Rect) {
    if area.width < left_w.saturating_add(gap).saturating_add(20) {
        let h = area.height / 2;
        return (
            Rect { height: h, ..area },
            Rect {
                y: area.y.saturating_add(h),
                height: area.height.saturating_sub(h),
                ..area
            },
        );
    }
    (
        Rect {
            width: left_w,
            ..area
        },
        Rect {
            x: area.x.saturating_add(left_w).saturating_add(gap),
            width: area.width.saturating_sub(left_w).saturating_sub(gap),
            ..area
        },
    )
}

fn path_and_depth_for(key: ItemKey) -> Option<(String, usize)> {
    let mut stack = Vec::new();
    for (index, node) in TREE.iter().enumerate() {
        stack.truncate(usize::from(node.depth()));
        stack.push(TREE_LABELS.get(index).map(|(label, _)| *label)?);
        if node.key() == Some(key) {
            let depth = usize::from(node.depth());
            return Some((stack.join("/"), depth));
        }
    }
    None
}

fn label_for(key: Option<ItemKey>) -> &'static str {
    key.and_then(|key| {
        TREE.iter()
            .position(|node| node.key() == Some(key))
            .and_then(|index| TREE_LABELS.get(index).map(|(label, _)| *label))
    })
    .unwrap_or("src")
}

/// Project navigation owns expansion by stable item key. No depth-derived key
/// can alias a sibling or move focus after a branch changes shape.
#[derive(Debug)]
pub struct TreesPage {
    state: TreeState,
    chosen: Option<ItemKey>,
    last: &'static str,
}

impl TreesPage {
    pub fn new() -> Self {
        let mut state = TreeState::new();
        // Match the legacy tree's first-level-open presentation. Descendants
        // remain closed until the user opens them, so keyboard expansion has
        // a deterministic, visible state transition.
        // The legacy widget records every top-level entry as expanded, even
        // when the entry is a leaf. Keep those stable keys so the historical
        // folder count and collapse transition remain visible.
        for key in [1_u64, 20, 26, 29, 30] {
            state.expand(ItemKey::Num(key));
        }
        state.set_cursor(0, ItemKey::Num(1));
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

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let result = project_tree(&[]).update(cx, &mut self.state, TREE);
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
        let _ = project_panel(&position_label(&self.state, 15), false);
        result.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        frame(
            ui,
            area,
            self.title(),
            "Indent carries hierarchy; the focus bar never moves",
            |ui, body| {
                let (l, r) = columns(body, (body.width * 3 / 5).max(30), 2);
                let project = Rect {
                    height: l.height.min(18),
                    ..l
                };
                let selection = Rect {
                    height: r.height.min(10),
                    ..r
                };
                let viewport_h = project.height.saturating_sub(3) as usize;
                let meta = position_label(&self.state, viewport_h);
                let focused = ui.state(PROJECT).contains(StateFlags::FOCUSED);
                project_panel(&meta, focused).draw(ui, project, |ui, inner| {
                    project_tree(&[]).draw(ui, inner, &self.state, TREE);
                });
                Panel::new(id!("trees.selection"))
                    .kind(PanelKind::Card)
                    .title("Selection")
                    .patch_part(PANEL_PARTS)
                    .draw(ui, selection, |_ui, _inner| {});

                let inner_x = selection.x.saturating_add(2);
                let inner_y = selection.y.saturating_add(2);
                let inner_w = selection.width.saturating_sub(4);
                let content_w = selection.right().saturating_sub(inner_x);
                let card_surface = ui.theme().raise(ui.surface());
                ui.with_surface(card_surface, |ui| {
                    let primary = ui.surface_style().patch(
                        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))),
                    );
                    let secondary = ui.surface_style().patch(
                        ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))),
                    );
                    let muted = ui
                        .surface_style()
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Muted))));
                    let faint = ui
                        .surface_style()
                        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
                    let sel = self.chosen.and_then(path_and_depth_for);
                    let mut y = inner_y;
                    match sel {
                        Some((path, depth)) => {
                            let path_text = truncate(&path, inner_w);
                            let _ = ui.paint_str(
                                Rect {
                                    x: inner_x,
                                    y,
                                    width: inner_w,
                                    height: 1,
                                },
                                &path_text,
                                primary,
                            );
                            y = y.saturating_add(1);
                            let depth_text = format!("depth {depth}");
                            let _ = ui.paint_str(
                                Rect {
                                    x: inner_x,
                                    y,
                                    width: inner_w,
                                    height: 1,
                                },
                                &depth_text,
                                muted,
                            );
                        }
                        None => {
                            let _ = ui.paint_str(
                                Rect {
                                    x: inner_x,
                                    y,
                                    width: inner_w,
                                    height: 1,
                                },
                                "Nothing selected",
                                muted,
                            );
                            y = y.saturating_add(1);
                            let _ = ui.paint_str(
                                Rect {
                                    x: inner_x,
                                    y,
                                    width: content_w,
                                    height: 1,
                                },
                                "Enter on a file selects it",
                                faint,
                            );
                        }
                    }
                    y = y.saturating_add(2);
                    let cur_label = label_for(self.state.cursor());
                    let cur_trunc = truncate(cur_label, inner_w.saturating_sub(8));
                    let _ = ui.paint_str(
                        Rect {
                            x: inner_x,
                            y,
                            width: 8.min(content_w),
                            height: 1,
                        },
                        "cursor",
                        faint,
                    );
                    if content_w > 8 {
                        let _ = ui.paint_str(
                            Rect {
                                x: inner_x.saturating_add(8),
                                y,
                                width: content_w.saturating_sub(8),
                                height: 1,
                            },
                            &cur_trunc,
                            secondary,
                        );
                    }
                    y = y.saturating_add(1);
                    let visible_count = format!("{} rows", self.state.scroll().content_len());
                    let _ = ui.paint_str(
                        Rect {
                            x: inner_x,
                            y,
                            width: 8.min(content_w),
                            height: 1,
                        },
                        "visible",
                        faint,
                    );
                    if content_w > 8 {
                        let _ = ui.paint_str(
                            Rect {
                                x: inner_x.saturating_add(8),
                                y,
                                width: content_w.saturating_sub(8),
                                height: 1,
                            },
                            &visible_count,
                            secondary,
                        );
                    }
                    y = y.saturating_add(1);
                    let open_count =
                        format!("{} folders", self.state.expanded().len_in(TREE.len()));
                    let _ = ui.paint_str(
                        Rect {
                            x: inner_x,
                            y,
                            width: 8.min(content_w),
                            height: 1,
                        },
                        "open",
                        faint,
                    );
                    if content_w > 8 {
                        let _ = ui.paint_str(
                            Rect {
                                x: inner_x.saturating_add(8),
                                y,
                                width: content_w.saturating_sub(8),
                                height: 1,
                            },
                            &open_count,
                            secondary,
                        );
                    }
                });
            },
        );
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[
            ("↑ ↓", "Move"),
            ("← →", "Fold / unfold"),
            ("Enter", "Open"),
            ("*", "Expand all"),
        ]
    }
}
