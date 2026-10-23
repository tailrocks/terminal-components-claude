use std::cell::{Cell, RefCell};

use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};
use termrock_core::{Axis, Id, ItemKey, Part, PartRef, Response};
use termrock_navigation::{Item, SemanticItemRow, Tree, TreeBranchClick, TreeNode, TreeState};
use termrock_runtime::{App, Cx, Family, RowUi, Ui, Variant};
use termrock_test_support::Harness;
use termrock_theme::{ColorLevel, GlyphRole, Role, StylePatch, Theme};

const TREE: Id = Id::root("tree.trailing-reserve");
const SEMANTIC_TREE: Id = Id::root("tree.trailing-reserve.semantic");

#[derive(Clone, Copy)]
struct Entry {
    name: &'static str,
    depth: u16,
    parent: bool,
    marked: bool,
    label: &'static str,
    metadata: &'static str,
}

fn entry_key(entry: &Entry) -> ItemKey {
    ItemKey::text(entry.name)
}

fn entry_node(entry: &Entry) -> TreeNode {
    if entry.parent {
        TreeNode::parent(entry.depth)
    } else {
        TreeNode::leaf(entry.depth)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReserveOrder {
    Unset,
    BeforeKey,
    AfterKey,
    AfterRow,
}

fn tree_base<'a>(
    reserve: Option<u16>,
    order: ReserveOrder,
) -> Tree<'a, Entry, fn(&Entry) -> ItemKey> {
    let key: fn(&Entry) -> ItemKey = entry_key;
    match (reserve, order) {
        (Some(cells), ReserveOrder::BeforeKey) => Tree::new(TREE).trailing_reserve(cells).key(key),
        (Some(cells), ReserveOrder::AfterKey) => Tree::new(TREE).key(key).trailing_reserve(cells),
        _ => Tree::new(TREE).key(key),
    }
}

struct Page {
    state: TreeState,
    entries: Vec<Entry>,
    area: Rect,
    clip: Option<Rect>,
    reserve: Option<u16>,
    order: ReserveOrder,
    custom_renderer: bool,
    mark_patch: StylePatch,
    row_widths: RefCell<Vec<(ItemKey, u16)>>,
    mark_calls: Cell<usize>,
    renderer_areas: RefCell<Vec<Rect>>,
}

impl Page {
    fn new(area: Rect, entries: Vec<Entry>, reserve: Option<u16>, order: ReserveOrder) -> Self {
        let mut state = TreeState::new();
        for entry in &entries {
            if entry.parent {
                state.expand(entry_key(entry));
            }
        }
        Self {
            state,
            entries,
            area,
            clip: None,
            reserve,
            order,
            custom_renderer: false,
            mark_patch: StylePatch::new(),
            row_widths: RefCell::new(Vec::new()),
            mark_calls: Cell::new(0),
            renderer_areas: RefCell::new(Vec::new()),
        }
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let hierarchy = |entry: &Entry| entry_node(entry);
        tree_base(self.reserve, self.order)
            .node(&hierarchy)
            .row(|_: &Entry, _: &mut RowUi<'_>| {})
            .branch_click(TreeBranchClick::Choose)
            .update(cx, &mut self.state, &self.entries)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.row_widths.borrow_mut().clear();
        self.mark_calls.set(0);
        self.renderer_areas.borrow_mut().clear();

        let hierarchy = |entry: &Entry| entry_node(entry);
        let trailing = |entry: &Entry| {
            self.mark_calls.set(self.mark_calls.get() + 1);
            entry
                .marked
                .then_some((GlyphRole::Checked, self.mark_patch))
        };
        let row = |entry: &Entry, row: &mut RowUi<'_>| {
            self.row_widths
                .borrow_mut()
                .push((entry_key(entry), row.remaining_width()));
            if !entry.metadata.is_empty() {
                row.meta(entry.metadata);
            }
            row.label(entry.label);
        };

        let tree = tree_base(self.reserve, self.order)
            .node(&hierarchy)
            .branch_click(TreeBranchClick::Choose)
            .trailing_mark(&trailing);
        let draw_tree = |ui: &mut Ui<'_>| {
            if self.custom_renderer {
                let renderer = |ui: &mut Ui<'_>, rect, flags, _: ItemKey, _: &Entry| {
                    self.renderer_areas.borrow_mut().push(rect);
                    let style = ui
                        .style(Family::TREE, Variant::DEFAULT, Part::CONTAINER, flags)
                        .style;
                    ui.fill(rect, style);
                    ui.paint_str(rect, "R", style);
                };
                let tree = tree.render_row(&renderer);
                let tree = if self.order == ReserveOrder::AfterRow {
                    if let Some(cells) = self.reserve {
                        tree.trailing_reserve(cells)
                    } else {
                        tree
                    }
                } else {
                    tree
                };
                tree.draw(ui, self.area, &self.state, &self.entries);
            } else {
                let tree = tree.row(row);
                let tree = if self.order == ReserveOrder::AfterRow {
                    if let Some(cells) = self.reserve {
                        tree.trailing_reserve(cells)
                    } else {
                        tree
                    }
                } else {
                    tree
                };
                tree.draw(ui, self.area, &self.state, &self.entries);
            }
        };

        if let Some(clip) = self.clip {
            ui.with_area(clip, draw_tree);
        } else {
            draw_tree(ui);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CellSnapshot {
    symbol: String,
    fg: Color,
    bg: Color,
    underline_color: Color,
    modifier: Modifier,
    skip: bool,
}

#[allow(deprecated)]
fn snapshot(h: &Harness<Page>) -> Vec<CellSnapshot> {
    let area = *h.buffer().area();
    (area.y..area.bottom())
        .flat_map(|y| {
            (area.x..area.right()).map(move |x| {
                let cell = h.buffer().cell((x, y)).expect("inside buffer");
                CellSnapshot {
                    symbol: cell.symbol().to_owned(),
                    fg: cell.fg,
                    bg: cell.bg,
                    underline_color: cell.underline_color,
                    modifier: cell.modifier,
                    skip: cell.skip,
                }
            })
        })
        .collect()
}

#[allow(deprecated)]
fn snapshot_row(h: &Harness<Page>, y: u16) -> Vec<CellSnapshot> {
    let area = *h.buffer().area();
    (area.x..area.right())
        .map(|x| {
            let cell = h.buffer().cell((x, y)).expect("inside buffer");
            CellSnapshot {
                symbol: cell.symbol().to_owned(),
                fg: cell.fg,
                bg: cell.bg,
                underline_color: cell.underline_color,
                modifier: cell.modifier,
                skip: cell.skip,
            }
        })
        .collect()
}

fn harness(page: Page) -> Harness<Page> {
    let width = page.area.right().saturating_add(4).max(20);
    let height = page.area.bottom().saturating_add(3).max(8);
    Harness::new(page, Theme::junie(), width, height)
}

fn leaf(name: &'static str, marked: bool) -> Entry {
    Entry {
        name,
        depth: 0,
        parent: false,
        marked,
        label: "entry label",
        metadata: "",
    }
}

fn deep_entries() -> Vec<Entry> {
    vec![
        Entry {
            name: "root",
            depth: 0,
            parent: true,
            marked: true,
            label: "root",
            metadata: "",
        },
        Entry {
            name: "branch-one",
            depth: 1,
            parent: true,
            marked: true,
            label: "branch one",
            metadata: "",
        },
        Entry {
            name: "branch-two",
            depth: 2,
            parent: true,
            marked: true,
            label: "branch two",
            metadata: "",
        },
        Entry {
            name: "deep-leaf",
            depth: 3,
            parent: false,
            marked: true,
            label: "deep leaf",
            metadata: "",
        },
    ]
}

fn row_area(h: &Harness<Page>, name: &'static str) -> Option<Rect> {
    h.area_of_part(TREE, PartRef::item(Part::ROW, ItemKey::text(name)))
}

fn icon_area(h: &Harness<Page>, name: &'static str) -> Option<Rect> {
    h.area_of_part(TREE, PartRef::item(Part::ICON, ItemKey::text(name)))
}

fn assert_same_tree_presentation(left: &Harness<Page>, right: &Harness<Page>, key: &'static str) {
    assert_eq!(snapshot(left), snapshot(right));
    assert_eq!(left.area_of(TREE), right.area_of(TREE));
    assert_eq!(row_area(left, key), row_area(right, key));
    assert_eq!(icon_area(left, key), icon_area(right, key));
    assert_eq!(
        left.area_of_part(TREE, PartRef::of(Part::TRACK)),
        right.area_of_part(TREE, PartRef::of(Part::TRACK))
    );
    assert_eq!(
        left.area_of_part(TREE, PartRef::of(Part::THUMB)),
        right.area_of_part(TREE, PartRef::of(Part::THUMB))
    );
    assert_eq!(left.focus(), right.focus());
    assert_eq!(left.focus_visible(), right.focus_visible());
    assert_eq!(left.ring().entries().len(), right.ring().entries().len());
    assert_eq!(left.cursor(), right.cursor());
    assert_eq!(left.app().state, right.app().state);
    assert_eq!(left.diagnostics(), right.diagnostics());
}

#[test]
fn default_two_and_normalized_reserves_preserve_tiny_offset_and_deep_frames() {
    let one = vec![leaf("only", true)];
    let overflow = deep_entries();
    let cases = [
        (Rect::new(0, 0, 0, 1), one.clone()),
        (Rect::new(2, 2, 16, 0), one.clone()),
        (Rect::new(2, 2, 1, 1), one.clone()),
        (Rect::new(2, 2, 2, 1), one.clone()),
        (Rect::new(2, 2, 3, 1), one),
        (Rect::new(3, 1, 10, 4), overflow.clone()),
        // Four rows in one cell force a real ScrollRegion track and thumb.
        (Rect::new(2, 1, 12, 1), overflow),
    ];

    for (area, entries) in cases {
        let baseline = harness(Page::new(area, entries.clone(), None, ReserveOrder::Unset));
        if area.height == 1 && entries.len() > 1 {
            assert!(
                baseline
                    .area_of_part(TREE, PartRef::of(Part::TRACK))
                    .is_some()
            );
            assert!(
                baseline
                    .area_of_part(TREE, PartRef::of(Part::THUMB))
                    .is_some()
            );
        }
        for (reserve, order) in [
            (2, ReserveOrder::BeforeKey),
            (2, ReserveOrder::AfterKey),
            (0, ReserveOrder::AfterRow),
            (1, ReserveOrder::BeforeKey),
        ] {
            let configured = harness(Page::new(area, entries.clone(), Some(reserve), order));
            assert_same_tree_presentation(&baseline, &configured, entries[0].name);
        }
    }
}

#[test]
fn reserve_three_moves_the_mark_and_narrows_only_its_row() {
    let area = Rect::new(2, 2, 32, 2);
    let root = Entry {
        name: "marked",
        depth: 0,
        parent: false,
        marked: true,
        label: "A long label that must yield space",
        metadata: "READY",
    };
    let sibling = Entry {
        name: "plain",
        depth: 0,
        parent: false,
        marked: false,
        label: "plain sibling",
        metadata: "FULL",
    };
    let h = harness(Page::new(
        area,
        vec![root, sibling],
        Some(3),
        ReserveOrder::AfterKey,
    ));
    let right = area.right();
    let mark_x = right - 3;
    assert_eq!(row_area(&h, "marked"), Some(Rect::new(2, 2, 32, 1)));
    assert_eq!(row_area(&h, "plain"), Some(Rect::new(2, 3, 32, 1)));
    assert_eq!(
        h.cell(mark_x, 2).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Checked)
    );
    assert_eq!(h.cell(right - 4, 2).symbol(), "Y");
    assert_eq!(h.cell(right - 4, 3).symbol(), "F");
    assert_eq!(h.cell(right - 1, 3).symbol(), "L");
    assert_eq!(h.cell(right - 2, 2).symbol(), " ");
    assert_eq!(h.cell(right - 1, 2).symbol(), " ");
    assert_eq!(h.cell(right - 2, 2).bg, h.cell(right - 1, 2).bg);
    assert_eq!(h.cell(right - 2, 2).fg, h.cell(right - 1, 2).fg);
    assert_eq!(h.cell(mark_x, 2).bg, h.cell(right - 2, 2).bg);
    assert_eq!(
        h.app().row_widths.borrow().as_slice(),
        &[(ItemKey::text("marked"), 26), (ItemKey::text("plain"), 29)]
    );
    assert_eq!(h.app().mark_calls.get(), 2);
    assert!(h.text().contains("READY"));

    let no_option = harness(Page::new(
        area,
        vec![root, sibling],
        None,
        ReserveOrder::Unset,
    ));
    assert_eq!(snapshot_row(&h, 3), snapshot_row(&no_option, 3));
}

#[test]
fn oversized_budget_keeps_logical_edge_or_suppresses_without_sliding() {
    let area = Rect::new(2, 2, 32, 2);
    let entries = vec![leaf("marked", true), leaf("plain", false)];
    let exact_lane = harness(Page::new(
        area,
        entries.clone(),
        Some(29),
        ReserveOrder::AfterRow,
    ));
    assert_eq!(
        exact_lane.cell(area.x + 3, area.y).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Checked),
        "a budget equal to the logical content lane may consume that lane"
    );
    assert_eq!(
        exact_lane.app().row_widths.borrow().as_slice(),
        &[(ItemKey::text("plain"), 29)]
    );

    for reserve in [30, u16::MAX] {
        let h = harness(Page::new(
            area,
            entries.clone(),
            Some(reserve),
            ReserveOrder::AfterKey,
        ));
        assert_ne!(
            h.cell(area.x + 2, area.y).symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::Checked),
            "unrepresentable marks do not slide over the prefix"
        );
        assert_ne!(
            h.cell(area.right() - 1, area.y).symbol(),
            Theme::junie().design.glyphs.get(GlyphRole::Checked),
            "unrepresentable marks do not slide to the row edge"
        );
        assert_eq!(
            h.app().row_widths.borrow().as_slice(),
            &[(ItemKey::text("plain"), 29)]
        );
        assert_eq!(h.app().mark_calls.get(), 2);
    }

    let mut clipped_page = Page::new(
        Rect::new(2, 2, 32, 1),
        vec![leaf("marked", true)],
        Some(3),
        ReserveOrder::AfterKey,
    );
    clipped_page.clip = Some(Rect::new(2, 2, 29, 1));
    let clipped = harness(clipped_page);
    assert_ne!(
        clipped.cell(30, 2).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Checked),
        "a physical clip edge does not attract the logical mark"
    );
    assert_eq!(
        clipped.app().row_widths.borrow().as_slice(),
        &[(ItemKey::text("marked"), 26)]
    );
}

#[test]
fn reserve_does_not_change_row_or_icon_hits_and_custom_renderer_stays_full_width() {
    let area = Rect::new(2, 2, 32, 1);
    let branch = Entry {
        name: "branch",
        depth: 0,
        parent: true,
        marked: true,
        label: "branch",
        metadata: "",
    };
    let children = vec![
        branch,
        Entry {
            name: "child",
            depth: 1,
            parent: false,
            ..leaf("child", false)
        },
    ];

    let mut default = harness(Page::new(
        area,
        children.clone(),
        Some(2),
        ReserveOrder::Unset,
    ));
    let mut larger = harness(Page::new(
        area,
        children.clone(),
        Some(3),
        ReserveOrder::AfterKey,
    ));
    assert_eq!(row_area(&default, "branch"), row_area(&larger, "branch"));
    assert_eq!(icon_area(&default, "branch"), icon_area(&larger, "branch"));

    let _ = default.click(area.x + 1, area.y);
    let _ = larger.click(area.x + 1, area.y);
    assert!(!default.app().state.is_expanded(ItemKey::text("branch")));
    assert_eq!(default.app().state, larger.app().state);

    let mut row_click_default =
        harness(Page::new(area, vec![branch], Some(2), ReserveOrder::Unset));
    let mut row_click_larger = harness(Page::new(
        area,
        vec![branch],
        Some(3),
        ReserveOrder::AfterRow,
    ));
    let _ = row_click_default.click(area.right() - 2, area.y);
    let _ = row_click_larger.click(area.right() - 3, area.y);
    assert_eq!(
        row_click_default.app().state.chosen(),
        Some(ItemKey::text("branch"))
    );
    assert_eq!(row_click_default.app().state, row_click_larger.app().state);

    let scroll_area = Rect::new(2, 2, 32, 1);
    let mut scroll_default = harness(Page::new(
        scroll_area,
        deep_entries(),
        Some(2),
        ReserveOrder::Unset,
    ));
    let mut scroll_larger = harness(Page::new(
        scroll_area,
        deep_entries(),
        Some(3),
        ReserveOrder::AfterKey,
    ));
    assert_eq!(scroll_default.app().state.scroll().offset(), 0);
    let _ = scroll_default.wheel(Axis::V, 1, scroll_area.x + 4, scroll_area.y);
    let _ = scroll_larger.wheel(Axis::V, 1, scroll_area.x + 4, scroll_area.y);
    assert_eq!(
        scroll_default.app().state.scroll().offset(),
        scroll_larger.app().state.scroll().offset()
    );
    assert!(scroll_default.app().state.scroll().offset() > 0);

    let mut custom_page = Page::new(area, vec![branch], Some(3), ReserveOrder::AfterRow);
    custom_page.custom_renderer = true;
    let custom = harness(custom_page);
    assert_eq!(custom.app().renderer_areas.borrow().as_slice(), &[area]);
    assert_eq!(custom.app().mark_calls.get(), 0);
    assert!(custom.text().contains("R"));
    assert_eq!(row_area(&custom, "branch"), Some(area));
}

struct SemanticPage {
    state: TreeState,
    items: [Item<'static>; 1],
    reserve: u16,
}

fn semantic_key(item: &Item<'_>) -> ItemKey {
    item.key
}

fn semantic_node(_: &Item<'_>) -> TreeNode {
    TreeNode::leaf(0)
}

impl App for SemanticPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Tree::new(SEMANTIC_TREE)
            .key(semantic_key)
            .node(&semantic_node)
            .row_adapter(SemanticItemRow::new())
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let trailing = |_: &Item<'static>| Some((GlyphRole::Checked, StylePatch::new()));
        Tree::new(SEMANTIC_TREE)
            .key(semantic_key)
            .node(&semantic_node)
            .row_adapter(
                SemanticItemRow::new()
                    .metadata_min_width(23)
                    .metadata_gap(0)
                    .minimum_label_width(1),
            )
            .trailing_mark(&trailing)
            .trailing_reserve(self.reserve)
            .draw(ui, Rect::new(2, 2, 28, 1), &self.state, &self.items);
    }
}

#[test]
fn semantic_row_adapter_reads_new_budget_without_rebuilding_the_warm_tree_index() {
    let mut h = Harness::new(
        SemanticPage {
            state: TreeState::new(),
            items: [Item::new(ItemKey::text("semantic"), "A long semantic label").detail("META")],
            reserve: 2,
        },
        Theme::junie(),
        36,
        8,
    );
    let before_row = h.area_of_part(
        SEMANTIC_TREE,
        PartRef::item(Part::ROW, ItemKey::text("semantic")),
    );
    assert!(
        h.text().contains("META"),
        "reserve two admits metadata: {}",
        h.text()
    );
    assert_eq!(
        h.cell(28, 2).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Checked)
    );

    h.app_mut().reserve = 3;
    h.draw();
    let after_row = h.area_of_part(
        SEMANTIC_TREE,
        PartRef::item(Part::ROW, ItemKey::text("semantic")),
    );
    assert_eq!(
        before_row, after_row,
        "the same indexed row remains registered"
    );
    assert!(
        !h.text().contains("META"),
        "reserve three narrows semantic admission: {}",
        h.text()
    );
    assert_eq!(
        h.cell(27, 2).symbol(),
        Theme::junie().design.glyphs.get(GlyphRole::Checked)
    );
}

#[test]
fn mark_patch_channels_keep_projection_and_clear_reset_authority() {
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        let theme = Theme::junie().for_level(level);
        let mut page = Page::new(
            Rect::new(2, 2, 28, 1),
            vec![leaf("marked", true)],
            Some(3),
            ReserveOrder::AfterKey,
        );
        page.mark_patch = StylePatch::new()
            .set_fg(Role::Warning)
            .set_bg(Role::Danger)
            .set_underline(Role::Success)
            .add(Modifier::UNDERLINED);
        let h = Harness::new(page, theme.clone(), 36, 8);
        let mark = h.cell(27, 2);
        assert_eq!(mark.symbol(), theme.design.glyphs.get(GlyphRole::Checked));
        assert_eq!(mark.fg, theme.color.warning, "{level:?}");
        assert_eq!(mark.bg, theme.color.danger, "{level:?}");
        assert_eq!(mark.underline_color, theme.color.success, "{level:?}");
        assert!(mark.modifier.contains(Modifier::UNDERLINED), "{level:?}");

        let mut clear_page = Page::new(
            Rect::new(2, 2, 28, 1),
            vec![leaf("marked", true)],
            Some(3),
            ReserveOrder::AfterRow,
        );
        clear_page.mark_patch = StylePatch::new().clear_fg().clear_bg();
        let clear = Harness::new(clear_page, theme.clone(), 36, 8);
        assert_eq!(
            clear.cell(27, 2).fg,
            theme.color.fg[0],
            "Clear returns to the component foreground at {level:?}"
        );
        assert_eq!(
            clear.cell(27, 2).bg,
            theme.color.surfaces[0],
            "Clear returns to the component surface at {level:?}"
        );

        let mut reset_page = Page::new(
            Rect::new(2, 2, 28, 1),
            vec![leaf("marked", true)],
            Some(3),
            ReserveOrder::AfterRow,
        );
        reset_page.mark_patch = StylePatch::new()
            .set_fg(Role::Custom(Color::Reset))
            .set_bg(Role::Custom(Color::Reset));
        let reset = Harness::new(reset_page, theme.clone(), 36, 8);
        assert_eq!(
            reset.cell(27, 2).fg,
            theme.color.fg[0],
            "a reset custom foreground leaves the component default at {level:?}"
        );
        assert_eq!(
            reset.cell(27, 2).bg,
            theme.color.surfaces[0],
            "a reset custom background retains the current surface at {level:?}"
        );
    }
}
