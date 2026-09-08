//! Keyed tree navigation with stable branch expansion.

use junie_tui::{
    Cx, Family, FgStep, GlyphRole, Id, ItemKey, Panel, PanelKind, Part, Rect, Response, Role,
    RowUi, StateFlags, StylePatch, Track, Tree, TreeAction, TreeNode, TreeState, Ui, Variant, id,
    layout,
};

use crate::data::{TREE, TREE_LABELS};

use super::{Page, frame};

const PROJECT: Id = id!("trees.project");
const TREE_GUTTER: &[(Part, StylePatch)] = &[(
    Part::GUTTER,
    StylePatch::new().set_glyph(GlyphRole::FocusBar),
)];
const PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(junie_tui::Modifier::BOLD),
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
        row.meta(match meta {
            "2.1 KB" => "2.1 KB ",
            "6.4 KB" => "6.4 KB ",
            "312 B" => "312 B ",
            "3.9 KB" => "3.9 KB ",
            "1.7 KB" => "1.7 KB ",
            "180 B" => "180 B ",
            "9.2 KB" => "9.2 KB ",
            "1.1 KB" => "1.1 KB ",
            "1.2 KB" => "1.2 KB ",
            "14.8 KB" => "14.8 KB ",
            "4.6 KB" => "4.6 KB ",
            "2.8 KB" => "2.8 KB ",
            "1.9 KB" => "1.9 KB ",
            "640 B" => "640 B ",
            "5.3 KB" => "5.3 KB ",
            "3.0 KB" => "3.0 KB ",
            "18 KB" => "18 KB ",
            "44 KB" => "44 KB ",
            "7.7 KB" => "7.7 KB ",
            "2.2 KB" => "2.2 KB ",
            "1.4 KB" => "1.4 KB ",
            "3.5 KB" => "3.5 KB ",
            other => other,
        });
    }
}

fn project_tree()
-> Tree<'static, TreeNode, impl Fn(&TreeNode) -> ItemKey, impl Fn(&TreeNode, &mut RowUi<'_>)> {
    Tree::new(PROJECT)
        .key(node_key)
        .node(&node_copy)
        .row(node_row)
        .patch_part(TREE_GUTTER)
}

fn position_label(state: &TreeState) -> String {
    let scroll = state.scroll();
    if !scroll.overflows() {
        return String::new();
    }
    let range = scroll.visible_range();
    format!(
        "{}–{} of {}",
        range.start + 1,
        range.end,
        scroll.content_len()
    )
}

fn paint_disclosure_glyphs(ui: &mut Ui<'_>, area: Rect, state: &TreeState) {
    let visible_start = state.scroll().offset();
    let visible_end = visible_start.saturating_add(usize::from(area.height));
    let mut display_index = 0usize;
    let mut collapsed_depth = None;
    for node in TREE {
        if collapsed_depth.is_some_and(|depth| node.depth() > depth) {
            continue;
        }
        collapsed_depth = None;
        let key = node_key(node);
        let is_open = node.has_children() && state.is_expanded(key);
        if display_index >= visible_start && display_index < visible_end && node.has_children() {
            let flags = if state.cursor() == Some(key) {
                StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE
            } else {
                StateFlags::empty()
            };
            let style = ui
                .style(Family::TREE, Variant::DEFAULT, Part::ICON, flags)
                .style;
            let glyph = if is_open {
                GlyphRole::Expanded
            } else {
                GlyphRole::Collapsed
            };
            let row = Rect {
                x: area
                    .x
                    .saturating_add(1)
                    .saturating_add(node.depth().saturating_mul(2)),
                y: area
                    .y
                    .saturating_add((display_index - visible_start) as u16),
                width: 1,
                height: 1,
            };
            let _ = ui.glyph(row, glyph, style);
        }
        display_index = display_index.saturating_add(1);
        if node.has_children() && !is_open {
            collapsed_depth = Some(node.depth());
        }
    }
}

fn path_for(key: ItemKey) -> Option<String> {
    let mut stack = Vec::new();
    for (index, node) in TREE.iter().enumerate() {
        stack.truncate(usize::from(node.depth()));
        stack.push(TREE_LABELS.get(index).map(|(label, _)| *label)?);
        if node.key() == Some(key) {
            return Some(stack.join("/"));
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
        // The legacy widget records every top-level entry as expanded, even
        // when the entry is a leaf. Keep those stable keys so the historical
        // folder count and collapse transition remain visible.
        for key in [1_u64, 20, 26, 29, 30] {
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
            "Indent carries hierarchy; the focus bar never moves",
            |ui, body| {
                let columns = layout::columns(body, &[Track::Flex(3), Track::Flex(2)], 2);
                let project = Rect {
                    height: body.height.min(18),
                    ..columns.first().copied().unwrap_or(body)
                };
                Panel::new(PROJECT)
                    .kind(PanelKind::Card)
                    .title("Project")
                    .meta(&position_label(&self.state))
                    .patch_part(PANEL_PARTS)
                    .draw(ui, project, |ui, inner| {
                        project_tree().draw(ui, inner, &self.state, TREE);
                        paint_disclosure_glyphs(ui, inner, &self.state);
                    });
                let selection = Rect {
                    height: body.height.min(10),
                    ..columns.get(1).copied().unwrap_or(body)
                };
                Panel::new(id!("trees.selection"))
                    .kind(PanelKind::Card)
                    .title("Selection")
                    .patch_part(PANEL_PARTS)
                    .draw(ui, selection, |ui, inner| {
                        let label = self.state.chosen().and_then(path_for);
                        let selection = label.as_deref().unwrap_or("Nothing selected");
                        let hint = "Enter on a file selects it";
                        let cursor = format!("cursor  {}", label_for(self.state.cursor()));
                        let visible = format!("visible {} rows", self.state.scroll().content_len());
                        let open = format!(
                            "open    {} folders",
                            self.state.expanded().len_in(TREE.len())
                        );
                        let detail = ui
                            .style(
                                Family::PANEL,
                                Variant::DEFAULT,
                                Part::DETAIL,
                                StateFlags::empty(),
                            )
                            .style;
                        let primary = ui.surface_style().patch(
                            ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))),
                        );
                        let faint = ui.surface_style().patch(
                            ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))),
                        );
                        for (offset, (text, style)) in [
                            (selection, primary),
                            (hint, faint),
                            ("", detail),
                            (cursor.as_str(), detail),
                            (visible.as_str(), detail),
                            (open.as_str(), detail),
                        ]
                        .into_iter()
                        .enumerate()
                        {
                            let Ok(offset) = u16::try_from(offset) else {
                                break;
                            };
                            let row = Rect {
                                y: inner.y.saturating_add(offset),
                                height: 1,
                                ..inner
                            };
                            let _ = ui.paint_str(row, text, style);
                        }
                    });
                let hint_style = ui
                    .style(
                        Family::PANEL,
                        Variant::DEFAULT,
                        Part::DETAIL,
                        StateFlags::empty(),
                    )
                    .style
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
                let _ = ui.paint_str(
                    Rect {
                        x: selection.x.saturating_add(2),
                        y: selection.y.saturating_add(3),
                        width: selection.width.saturating_sub(2),
                        height: 1,
                    },
                    "Enter on a file selects it",
                    hint_style,
                );
            },
        );
    }
}
