use std::cell::RefCell;

use ratatui_core::layout::Rect;
use termrock_core::{Id, ItemKey, KeyCode, StateFlags, Variant};
use termrock_navigation::{
    FilterList, FilterListState, FilterPolicy, Item, List, ListState, Tree, TreeNode, TreeState,
};
use termrock_runtime::{App, Cx, Family, Response, Ui};
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, GlyphRole, StylePatch, Theme};

const TREE: Id = Id::root("row-container-width.tree");
const AUTHOR: Id = Id::root("row-container-width.author");
const LIST: Id = Id::root("row-container-width.list");
const FILTER: Id = Id::root("row-container-width.filter");

#[derive(Clone, Copy)]
struct Node {
    key: &'static str,
    label: &'static str,
    depth: u16,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label)
    }
}

fn node(value: &Node) -> TreeNode {
    TreeNode::leaf(value.depth).keyed(ItemKey::text(value.key))
}

fn mark(_: &Node) -> Option<(GlyphRole, StylePatch)> {
    Some((GlyphRole::Dirty, StylePatch::new()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Observation {
    width: u16,
    area: Rect,
}

struct TreePage<'a> {
    state: TreeState,
    items: &'a [Node],
    width: u16,
    height: u16,
    trailing: bool,
    seen: RefCell<Vec<Observation>>,
}

impl App for TreePage<'_> {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let tree = Tree::new(TREE).node(&node);
        tree.update(cx, &mut self.state, self.items).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let seen = &self.seen;
        let row = move |item: &Node, row: &mut termrock_runtime::RowUi<'_>| {
            seen.borrow_mut().push(Observation {
                width: row.container_width(),
                area: row.area(),
            });
            row.label(item.label);
        };
        let tree = Tree::new(TREE).node(&node).row(row);
        let tree = if self.trailing {
            tree.trailing_mark(&mark)
        } else {
            tree
        };
        tree.draw(
            ui,
            Rect::new(7, 2, self.width, self.height),
            &self.state,
            self.items,
        );
    }
}

fn scene(width: u16, height: u16, trailing: bool) -> (Scene, Vec<Observation>) {
    let items = [
        Node {
            key: "root",
            label: "root-label",
            depth: 0,
        },
        Node {
            key: "nested",
            label: "nested-label",
            depth: 2,
        },
    ];
    let page = TreePage {
        state: TreeState::default(),
        items: &items,
        width,
        height,
        trailing,
        seen: RefCell::new(Vec::new()),
    };
    let observations = &page.seen;
    let mut scene = Scene::new(
        "row-container-width",
        Theme::junie(),
        ColorLevel::TrueColor,
        64,
        8,
    );
    scene.draw(|ui, _| page.draw(ui));
    (scene, observations.borrow().clone())
}

#[test]
fn tree_container_width_precedes_depth_disclosure_and_trailing_reservations() {
    for width in [43, 44, 45] {
        let (scene, observations) = scene(width, 2, true);
        assert_eq!(observations.len(), 2);
        assert_eq!(
            observations.iter().map(|row| row.width).collect::<Vec<_>>(),
            [width, width]
        );
        assert!(observations.iter().all(|row| row.area.width < width));
        assert!(observations[1].area.width < observations[0].area.width);
        assert!(scene.text().contains("root-label"));
    }
}

#[test]
fn tree_container_width_excludes_external_scrollbar_but_not_tree_prefixes() {
    for width in [43, 44, 45] {
        let (_, observations) = scene(width, 1, true);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].width, width.saturating_sub(1));
        assert!(observations[0].area.width < observations[0].width);
    }
}

struct PublicRowWidthPage {
    observed: RefCell<Option<(u16, Rect)>>,
}

impl App for PublicRowWidthPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let row = termrock_runtime::RowUi::new_with_container_width(
            ui,
            AUTHOR,
            Family::TREE,
            Variant::DEFAULT,
            StateFlags::empty(),
            ItemKey::text("author-row"),
            Rect::new(9, 3, 0, 1),
            0,
        );
        *self.observed.borrow_mut() = Some((row.container_width(), row.area()));
    }
}

#[test]
fn zero_width_author_row_reports_zero_without_panicking() {
    let page = PublicRowWidthPage {
        observed: RefCell::new(None),
    };
    let observed = &page.observed;
    let mut scene = Scene::new(
        "zero-row-container-width",
        Theme::junie(),
        ColorLevel::TrueColor,
        20,
        5,
    );
    scene.draw(|ui, _| page.draw(ui));
    assert_eq!(*observed.borrow(), Some((0, Rect::new(9, 3, 0, 1))));
}

struct ListPage {
    state: ListState,
    items: Vec<Node>,
    width: u16,
    height: u16,
    seen: RefCell<Vec<(ItemKey, Observation)>>,
}

fn node_key(value: &Node) -> ItemKey {
    ItemKey::text(value.key)
}

impl App for ListPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let row = |item: &Node, row: &mut termrock_runtime::RowUi<'_>| row.label(item.label);
        List::new(LIST)
            .key(node_key)
            .row(row)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let seen = &self.seen;
        let row = move |item: &Node, row: &mut termrock_runtime::RowUi<'_>| {
            seen.borrow_mut().push((
                node_key(item),
                Observation {
                    width: row.container_width(),
                    area: row.area(),
                },
            ));
            row.label(item.label);
        };
        List::new(LIST).key(node_key).row(row).draw(
            ui,
            Rect::new(7, 2, self.width, self.height),
            &self.state,
            &self.items,
        );
    }
}

fn list_harness(width: u16, height: u16) -> termrock_test_support::Harness<ListPage> {
    termrock_test_support::Harness::new(
        ListPage {
            state: ListState::default(),
            items: vec![
                Node {
                    key: "alpha",
                    label: "alpha",
                    depth: 0,
                },
                Node {
                    key: "beta",
                    label: "beta",
                    depth: 0,
                },
                Node {
                    key: "gamma",
                    label: "gamma",
                    depth: 0,
                },
            ],
            width,
            height,
            seen: RefCell::new(Vec::new()),
        },
        Theme::junie(),
        64,
        8,
    )
}

#[test]
fn list_container_width_excludes_scrollbar_before_internal_columns_and_survives_scroll_reorder() {
    for width in [43, 44, 45] {
        let mut harness = list_harness(width, 1);
        assert_eq!(harness.app().seen.borrow().len(), 1);
        let (key, observation) = harness.app().seen.borrow()[0];
        assert_eq!(observation.width, width - 1);
        assert!(observation.area.width < observation.width);
        assert!(observation.area.right() < 7 + width);
        let _ = harness.key(KeyCode::End);
        harness.draw();
        assert_eq!(harness.app().seen.borrow().len(), 1);
        assert_eq!(harness.app().seen.borrow()[0].0, ItemKey::text("gamma"));
        assert_eq!(harness.app().seen.borrow()[0].1.width, width - 1);

        harness.app_mut().items.swap(1, 2);
        let _ = harness.tick();
        assert_eq!(harness.app().seen.borrow().len(), 1);
        assert_eq!(harness.app().seen.borrow()[0].1.width, width - 1);
        assert_ne!(key, harness.app().seen.borrow()[0].0);
    }

    let no_scrollbar = list_harness(43, 3);
    assert_eq!(no_scrollbar.app().seen.borrow()[0].1.width, 43);
    for width in [0, 1, 2, 3, 4] {
        let narrow = list_harness(width, 1);
        assert!(
            narrow
                .app()
                .seen
                .borrow()
                .iter()
                .all(|(_, row)| row.area.width <= row.width)
        );
    }
}

struct FilterPage {
    state: FilterListState,
    items: Vec<Item<'static>>,
    width: u16,
    height: u16,
    seen: RefCell<Vec<(ItemKey, Observation)>>,
}

impl App for FilterPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let row = |item: &Item<'static>, row: &mut termrock_runtime::RowUi<'_>| {
            row.label(item.label);
        };
        FilterList::new(FILTER)
            .searchable(true)
            .filter(FilterPolicy::Label)
            .row(row)
            .update(cx, &mut self.state, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let seen = &self.seen;
        let row = move |item: &Item<'static>, row: &mut termrock_runtime::RowUi<'_>| {
            seen.borrow_mut().push((
                item.key,
                Observation {
                    width: row.container_width(),
                    area: row.area(),
                },
            ));
            row.label(item.label);
        };
        FilterList::new(FILTER)
            .searchable(true)
            .filter(FilterPolicy::Label)
            .row(row)
            .draw(
                ui,
                Rect::new(7, 2, self.width, self.height),
                &self.state,
                &self.items,
            );
    }
}

fn filter_harness(width: u16, height: u16) -> termrock_test_support::Harness<FilterPage> {
    termrock_test_support::Harness::new(
        FilterPage {
            state: FilterListState::default(),
            items: vec![
                Item::new(ItemKey::text("delta"), "delta"),
                Item::new(ItemKey::text("epsilon"), "epsilon"),
                Item::new(ItemKey::text("zeta"), "zeta"),
            ],
            width,
            height,
            seen: RefCell::new(Vec::new()),
        },
        Theme::junie(),
        64,
        8,
    )
}

#[test]
fn filter_list_width_tracks_scrollbar_and_filtering_not_internal_layout() {
    for width in [43, 44, 45] {
        let mut harness = filter_harness(width, 2);
        assert_eq!(harness.app().seen.borrow().len(), 2);
        let first = harness.app().seen.borrow()[0].1;
        assert_eq!(first.width, width - 1);
        assert_eq!(first.area.width, first.width);
        assert!(first.area.right() < 7 + width);

        harness.app_mut().state.set_query("zeta");
        let _ = harness.tick();
        assert_eq!(harness.app().seen.borrow().len(), 1);
        let filtered = harness.app().seen.borrow()[0];
        assert_eq!(filtered.0, ItemKey::text("zeta"));
        assert_eq!(filtered.1.width, width);
    }

    for width in [0, 1, 2, 3, 4] {
        let narrow = filter_harness(width, 1);
        assert!(
            narrow
                .app()
                .seen
                .borrow()
                .iter()
                .all(|(_, row)| row.area.width <= row.width)
        );
    }
}
