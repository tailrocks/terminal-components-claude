use ratatui_core::layout::Rect;
use termrock_core::{Id, ItemKey, KeyCode, Response};
use termrock_navigation::{
    FilterList, FilterListState, Item, ItemLabelOverflow, SemanticItemRow, Tree, TreeNode,
    TreeState,
};
use termrock_runtime::{App, Cx, Part, PartRef, Ui, UpdateCause};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, GlyphRole, StylePatch, Theme};

const TREE: Id = Id::root("semantic.item.tree");
const FILTER: Id = Id::root("semantic.item.filter");
fn tree_items() -> Vec<Item<'static>> {
    vec![
        Item::new(
            ItemKey::text("workspace"),
            "A long workspace identity with Unicode 界 and an extended trailing identity tail",
        )
        .glyph("W")
        .detail("accounts/primary/context")
        .tag("ACTIVE")
        .group("WORKSPACES"),
        Item::new(ItemKey::text("workspace-child"), "child")
            .glyph("D")
            .detail("leaf metadata")
            .tag("READY"),
    ]
}

fn filter_items() -> Vec<Item<'static>> {
    vec![
        Item::new(ItemKey::text("open"), "Open document")
            .glyph("F")
            .detail("recently used")
            .tag("OPEN"),
        Item::new(ItemKey::text("save"), "Save document")
            .glyph("S")
            .detail("local changes")
            .tag("DIRTY"),
    ]
}

fn tree_node(item: &Item<'_>) -> TreeNode {
    if item.key == ItemKey::text("workspace") {
        TreeNode::parent(0)
    } else {
        TreeNode::leaf(0)
    }
}

fn tree(
    row: SemanticItemRow,
) -> Tree<'static, Item<'static>, impl Fn(&Item<'static>) -> ItemKey, SemanticItemRow> {
    Tree::new(TREE)
        .key(|item: &Item<'static>| item.key)
        .node(&tree_node)
        .row_adapter(row)
}

struct Page {
    tree_state: TreeState,
    filter_state: FilterListState,
    tree_width: u16,
    filter_mode: bool,
    marker: bool,
    tree_items: Vec<Item<'static>>,
    filter_items: Vec<Item<'static>>,
    row: SemanticItemRow,
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(if self.filter_mode { FILTER } else { TREE });
        }
        if self.filter_mode {
            FilterList::new(FILTER)
                .searchable(false)
                .row(SemanticItemRow::new())
                .update(cx, &mut self.filter_state, &self.filter_items)
                .erase()
        } else {
            tree(self.row)
                .update(cx, &mut self.tree_state, &self.tree_items)
                .erase()
        }
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.filter_mode {
            FilterList::new(FILTER)
                .searchable(false)
                .row(SemanticItemRow::new())
                .draw(
                    ui,
                    Rect::new(5, 2, self.tree_width, 3),
                    &self.filter_state,
                    &self.filter_items,
                );
            return;
        }
        let trailing = |_: &Item<'static>| {
            self.marker
                .then_some((GlyphRole::Checked, StylePatch::new()))
        };
        let tree = tree(self.row).trailing_mark(&trailing);
        tree.draw(
            ui,
            Rect::new(5, 2, self.tree_width, 4),
            &self.tree_state,
            &self.tree_items,
        );
    }
}

fn page(width: u16, level: ColorLevel) -> Harness<Page> {
    Harness::new(
        Page {
            tree_state: TreeState::default(),
            filter_state: FilterListState::default(),
            tree_width: width,
            filter_mode: false,
            marker: false,
            tree_items: tree_items(),
            filter_items: filter_items(),
            row: SemanticItemRow::new(),
        },
        Theme::junie().for_level(level),
        72,
        10,
    )
}

#[test]
fn metadata_admission_uses_the_container_content_width_threshold() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        // Tree owns three leading cells (gutter, disclosure and marker). The
        // semantic row's content therefore receives exactly 43, 44 or 45.
        for (content_width, show_metadata) in [(43_u16, false), (44, true), (45, true)] {
            let mut h = page(content_width.saturating_add(3), level);
            h.app_mut()
                .tree_state
                .set_cursor(0, ItemKey::text("workspace"));
            h.draw();
            let row = h.area_of_part(TREE, PartRef::item(Part::ROW, ItemKey::text("workspace")));
            assert!(row.is_some(), "visible tree row at width {content_width}");
            let text = h.text();
            assert_eq!(
                text.contains("ACTIVE"),
                show_metadata,
                "content width {content_width}, profile {level:?}: {text}"
            );
            assert!(
                text.contains("W"),
                "container's first content slot remains visible"
            );
            assert!(
                text.contains('…'),
                "long labels use middle truncation: {text}"
            );
        }
    }
}

#[test]
fn tree_keeps_gutter_focus_hit_and_trailing_reservation_ownership() {
    let mut h = page(54, ColorLevel::TrueColor);
    h.app_mut().marker = true;
    h.app_mut()
        .tree_state
        .set_cursor(0, ItemKey::text("workspace"));
    h.app_mut().tree_state.expand(ItemKey::text("workspace"));
    h.draw();
    let row = h
        .area_of_part(TREE, PartRef::item(Part::ROW, ItemKey::text("workspace")))
        .expect("tree registers the semantic row hit region");
    assert_eq!(h.cell(row.x.saturating_add(3), row.y).symbol(), "W");
    assert_eq!(
        h.cell(row.right().saturating_sub(2), row.y).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Checked)
    );
    assert_eq!(
        h.app().tree_state.cursor(),
        Some(ItemKey::text("workspace"))
    );
    assert!(
        h.cell(row.x, row.y).symbol() != "W",
        "Tree retains its gutter"
    );

    let _ = h.key(KeyCode::Down);
    assert_eq!(
        h.app().tree_state.cursor(),
        Some(ItemKey::text("workspace-child"))
    );
}

#[test]
fn semantic_row_is_reusable_with_filter_list_domain_data() {
    let mut h = page(50, ColorLevel::Ansi256);
    h.app_mut().filter_mode = true;
    h.draw();
    assert!(h.text().contains("Open document"));
    assert!(h.text().contains("recently used"));
    assert!(h.text().contains("DIRTY"));
    assert_eq!(
        h.area_of_part(FILTER, PartRef::item(Part::ROW, ItemKey::text("save")))
            .map(|area| area.width),
        Some(50)
    );
}

#[test]
fn empty_and_zero_through_narrow_regions_remain_safe() {
    for width in [0, 1, 3, 4, 5, 8] {
        let mut h = page(width, ColorLevel::Mono);
        h.draw();
    }
    for width in [0, 1, 3, 4, 8] {
        let mut h = page(width, ColorLevel::Mono);
        h.app_mut().tree_items.clear();
        h.app_mut().filter_items.clear();
        h.app_mut().filter_mode = true;
        h.draw();
        assert_eq!(h.app().filter_state.cursor(), None);
    }
}

#[test]
fn public_policy_builders_are_independent_and_keep_gutter_out_of_tree_rows() {
    let base = SemanticItemRow::new();
    let narrow = base.metadata_min_width(64);
    let compact = base.metadata_gap(0).minimum_label_width(4);
    assert_ne!(base, narrow);
    assert_ne!(base, compact);
    assert_eq!(base, SemanticItemRow::default());
}

#[test]
fn label_overflow_is_a_reusable_policy_and_middle_is_grapheme_safe() {
    let mut h = page(46, ColorLevel::TrueColor);
    h.app_mut().row = SemanticItemRow::new()
        .metadata_min_width(u16::MAX)
        .label_overflow(ItemLabelOverflow::End);
    h.draw();
    let end_truncated = h.text();
    assert!(end_truncated.contains("A long workspace"));
    assert!(!end_truncated.contains("trailing identity tail"));

    let mut middle = page(46, ColorLevel::TrueColor);
    middle.app_mut().row = SemanticItemRow::new().metadata_min_width(u16::MAX);
    middle.draw();
    let middle_truncated = middle.text();
    assert!(middle_truncated.contains("tail"));
    assert!(middle_truncated.contains('…'));
}
