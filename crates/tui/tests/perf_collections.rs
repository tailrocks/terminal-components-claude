//! Slice 4 collection performance acceptance tests.
//!
//! Separate integration-test binary prevents its codegen from perturbing
//! allocation counts in the established `perf` baseline binary.
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::arithmetic_side_effects,
        clippy::cast_lossless,
        clippy::print_stdout
    )
)]

use std::cell::Cell;
use std::fmt::Write as _;
use std::hint::black_box;

use junie_tui::{
    AsItem, CellRef, ColorLevel, Column, ColumnKey, Grid, GridModel, GridState, Id, Item,
    ItemKey as StableItemKey, Picker, PickerState, Rect, Registry, StepState, Steps, StepsState,
    TextViewport, Theme, Tree, TreeNode, TreeState, ViewportLine, ViewportState, ViewportWorkProbe,
};
use junie_tui_testing::Scene;
use junie_tui_testing::perf::{
    Counting, Stats, bench, big, check_ratio, env_flag, iters, lock, measure_once, report,
    unicode_line_inline,
};

#[global_allocator]
static GLOBAL: Counting = Counting;

// ----------------------------------------------- H. Slice 4 large components

const TREE_ID: Id = Id::root("perf.tree");
const STEPS_ID: Id = Id::root("perf.steps");
const GRID_ID: Id = Id::root("perf.grid");
const VIEWPORT_ID: Id = Id::root("perf.viewport");
const PICKER_ID: Id = Id::root("perf.picker");

struct BorrowedDomainItem<'a> {
    index: usize,
    reads: &'a Cell<usize>,
}

impl AsItem for BorrowedDomainItem<'_> {
    fn as_item(&self) -> Item<'_> {
        self.reads.set(self.reads.get().saturating_add(1));
        Item::new(StableItemKey::index(self.index), "borrowed domain item")
    }
}

fn bench_picker_borrowed_domain(n: usize) -> (Stats, usize, usize) {
    let reads = Cell::new(0);
    let items: Vec<_> = (0..n)
        .map(|index| BorrowedDomainItem {
            index,
            reads: &reads,
        })
        .collect();
    let picker = Picker::<BorrowedDomainItem<'_>>::new(PICKER_ID);
    let state = PickerState::default();
    let mut scene = Scene::new(
        "picker-borrowed-domain",
        Theme::junie(),
        ColorLevel::TrueColor,
        80,
        24,
    );
    scene.draw(|ui, area| {
        picker.draw(ui, area, &state, &items);
    });
    scene.draw(|ui, area| {
        picker.draw(ui, area, &state, &items);
    });
    scene.draw(|ui, area| {
        picker.draw(ui, area, &state, &items);
    });
    reads.set(0);
    scene.draw(|ui, area| {
        picker.draw(ui, area, &state, &items);
    });
    let probe_reads = reads.get();
    reads.set(0);
    let sample = bench(2, iters(200), &mut || {
        scene.draw(|ui, area| {
            black_box(picker.draw(ui, area, &state, &items));
        });
    });
    (
        sample,
        probe_reads,
        scene.registry().map_or(0, Registry::len),
    )
}

#[test]
fn picker_100k_borrowed_domain_render() {
    let _g = lock();
    let (control, control_reads, control_regions) = bench_picker_borrowed_domain(big(1_000));
    let (sample, reads, regions) = bench_picker_borrowed_domain(big(100_000));
    report("picker_100k_borrowed_domain_render", &sample);
    assert_eq!(
        control_reads, 38,
        "1 k picker did not read exactly 19 visible rows"
    );
    assert_eq!(reads, 38, "100 k picker read outside its 19 visible rows");
    assert_eq!(regions, control_regions, "regions grew with domain size");
    assert_eq!(sample.allocs, 0, "warmed borrowed-domain draw allocated");
    assert_eq!(
        sample.bytes, 0,
        "warmed borrowed-domain draw allocated bytes"
    );
    check_ratio(
        "picker_100k_vs_1k_borrowed_domain_render",
        sample.ns,
        control.ns,
        1.5,
        env_flag("PERF_STRICT"),
    );
}

fn viewport_lines(n: usize, extra_capacity: usize) -> Vec<ViewportLine<'static>> {
    let mut lines = Vec::with_capacity(n.saturating_add(extra_capacity));
    lines.resize(n, ViewportLine::Plain("row: lorem ipsum dolor sit amet"));
    lines
}

fn measure_viewport_push(n: usize, pushed: usize) -> (Stats, usize) {
    let mut lines = viewport_lines(n, pushed);
    let probe = ViewportWorkProbe::default();
    let viewport = TextViewport::new(VIEWPORT_ID).wrap(true).work_probe(&probe);
    let state = ViewportState::default();
    let mut scene = Scene::new(
        "viewport-push",
        Theme::junie(),
        ColorLevel::TrueColor,
        80,
        40,
    );
    scene.draw(|ui, area| {
        viewport.draw(ui, area, &state, &lines);
    });
    scene.draw(|ui, area| {
        viewport.draw(ui, area, &state, &lines);
    });

    probe.reset();
    let allocation_stats = measure_once(&mut || {
        lines.extend(core::iter::repeat_n(
            ViewportLine::Plain("appended: lorem ipsum dolor sit amet"),
            pushed,
        ));
        scene.draw(|ui, area| {
            black_box(viewport.draw(ui, area, &state, &lines));
        });
    });
    (allocation_stats, probe.snapshot().indexed_lines)
}

#[test]
fn viewport_100k_lines_push() {
    let _g = lock();
    let control_n = big(10_000);
    let n = big(100_000);
    let pushed = big(1_000);
    let (control, control_indexed) = measure_viewport_push(control_n, pushed);
    let (s, indexed) = measure_viewport_push(n, pushed);
    println!(
        "PERF-NOTE viewport_100k_lines_push lines={n} allocs_at_{control_n}_lines={}",
        control.allocs
    );
    report("viewport_100k_lines_push", &s);
    assert_eq!(
        s.allocs, control.allocs,
        "push allocation count grew with buffer size"
    );
    assert_eq!(control_indexed, pushed);
    assert_eq!(indexed, pushed, "append rebuilt the existing prefix");
}

fn bench_viewport_render(n: usize) -> (Stats, usize, usize) {
    let lines = viewport_lines(n, 0);
    let probe = ViewportWorkProbe::default();
    let viewport = TextViewport::new(VIEWPORT_ID).wrap(true).work_probe(&probe);
    let state = ViewportState::default();
    let mut scene = Scene::new(
        "viewport-render",
        Theme::junie(),
        ColorLevel::TrueColor,
        80,
        40,
    );
    scene.draw(|ui, area| {
        viewport.draw(ui, area, &state, &lines);
    });
    probe.reset();
    let s = bench(1, iters(3), &mut || {
        scene.draw(|ui, area| {
            black_box(viewport.draw(ui, area, &state, &lines));
        });
    });
    let regions = scene.registry().map_or(0, Registry::len);
    let ring = scene.ring().map_or(0, |ring| ring.reachable().count());
    let work = probe.snapshot();
    let (indexed, visible) = (work.indexed_lines, work.visible_rows);
    (s.with_regions(regions, ring), indexed, visible)
}

#[test]
fn viewport_100k_lines_render() {
    let _g = lock();
    let (control, control_indexed, control_visible) = bench_viewport_render(big(1_000));
    let (s, indexed, visible) = bench_viewport_render(big(100_000));
    report("viewport_100k_lines_render", &s);
    assert_eq!(control.allocs, 0, "warmed control frame allocated");
    assert_eq!(s.allocs, 0, "warmed 100 k frame allocated");
    assert_eq!(
        s.allocs, control.allocs,
        "render allocation count grew with buffer size"
    );
    assert_eq!(control_indexed, 0, "warmed control rebuilt its prefix");
    assert_eq!(indexed, 0, "warmed 100 k frame rebuilt its prefix");
    assert_eq!(
        visible, control_visible,
        "visible-row work grew with buffer size"
    );
    check_ratio(
        "viewport_100k_lines_render_vs_1k",
        s.ns,
        control.ns,
        1.5,
        env_flag("PERF_STRICT"),
    );
}

#[test]
fn viewport_100k_lines_reindex() {
    let _g = lock();
    let n = big(100_000);
    let lines = viewport_lines(n, 0);
    let probe = ViewportWorkProbe::default();
    let viewport = TextViewport::new(VIEWPORT_ID).wrap(true).work_probe(&probe);
    let mut state = ViewportState::default();
    let mut scene = Scene::new(
        "viewport-reindex",
        Theme::junie(),
        ColorLevel::TrueColor,
        80,
        40,
    );

    probe.reset();
    let cold = measure_once(&mut || {
        scene.draw(|ui, area| {
            black_box(viewport.draw(ui, area, &state, &lines));
        });
    });
    let cold_indexed = probe.snapshot().indexed_lines;

    probe.reset();
    let width_change = measure_once(&mut || {
        scene.draw(|ui, _| {
            black_box(viewport.draw(ui, Rect::new(0, 0, 79, 40), &state, &lines));
        });
    });
    let width_indexed = probe.snapshot().indexed_lines;

    state.invalidate();
    probe.reset();
    let invalidated = measure_once(&mut || {
        scene.draw(|ui, _| {
            black_box(viewport.draw(ui, Rect::new(0, 0, 79, 40), &state, &lines));
        });
    });
    let invalidated_indexed = probe.snapshot().indexed_lines;

    println!(
        "PERF-NOTE viewport_100k_lines_reindex lines={n} cold_indexed={cold_indexed} \
         width_indexed={width_indexed} invalidated_indexed={invalidated_indexed} \
         width_ns={} invalidated_ns={}",
        width_change.ns, invalidated.ns
    );
    report("viewport_100k_lines_reindex", &cold);
    assert_eq!(cold_indexed, n);
    assert_eq!(width_indexed, n);
    assert_eq!(invalidated_indexed, n);
}

#[test]
fn viewport_layout_10k_grapheme_line() {
    let _g = lock();
    let text = unicode_line_inline(big(10_000));
    let lines = [ViewportLine::Plain(&text)];
    let viewport = TextViewport::new(VIEWPORT_ID).wrap(true);
    let state = ViewportState::default();
    let mut scene = Scene::new(
        "viewport-layout",
        Theme::junie(),
        ColorLevel::TrueColor,
        100,
        3,
    );
    scene.draw(|ui, _| {
        viewport.draw(ui, Rect::new(0, 0, 82, 3), &state, &lines);
    });
    let mut flip = false;
    let s = bench(2, iters(20), &mut || {
        flip = !flip;
        let width = if flip { 82 } else { 83 };
        scene.draw(|ui, _| {
            black_box(viewport.draw(ui, Rect::new(0, 0, width, 3), &state, &lines));
        });
    });
    report("viewport_layout_10k_grapheme_line", &s);
    assert_eq!(s.allocs, 0, "layout must not allocate per grapheme");
}

#[derive(Clone, Copy)]
struct PerfTreeNode {
    key: u64,
    depth: u16,
    parent: bool,
}

fn perf_tree_nodes(n: usize) -> Vec<PerfTreeNode> {
    let mut nodes = Vec::with_capacity(n);
    nodes.push(PerfTreeNode {
        key: 0,
        depth: 0,
        parent: true,
    });
    nodes.extend((1..n).map(|key| PerfTreeNode {
        key: key as u64,
        depth: 1,
        parent: false,
    }));
    nodes
}

/// One warmed expand/collapse pair. Accessor counts are binding: an ordinary
/// toggle splices the cached projection and must not walk the borrowed source.
fn bench_tree_toggle(n: usize, iterations: usize) -> (Stats, usize) {
    let nodes = perf_tree_nodes(n);
    let node_accesses = Cell::new(0usize);
    let node = |item: &PerfTreeNode| {
        node_accesses.set(node_accesses.get().saturating_add(1));
        if item.parent {
            TreeNode::parent(item.depth)
        } else {
            TreeNode::leaf(item.depth)
        }
        .keyed(StableItemKey::num(item.key))
    };
    let tree = Tree::new(TREE_ID).node(&node).row(|_, _| {});
    let mut state = TreeState::new();
    let mut scene = Scene::new("tree-toggle", Theme::junie(), ColorLevel::TrueColor, 80, 40);

    scene.draw(|ui, area| {
        tree.draw(ui, area, &state, &nodes);
    });
    assert_eq!(
        node_accesses.get(),
        nodes.len(),
        "cold index did not scan once"
    );

    // Reserve the large visible projection once, matching the legacy warmed
    // harness. Later toggles may change its length but not rebuild its source.
    state.expand(StableItemKey::num(0));
    scene.draw(|ui, area| {
        tree.draw(ui, area, &state, &nodes);
    });
    state.collapse(StableItemKey::num(0));
    scene.draw(|ui, area| {
        tree.draw(ui, area, &state, &nodes);
    });
    node_accesses.set(0);

    let s = bench(1, iters(iterations), &mut || {
        black_box(state.toggle(StableItemKey::num(0)));
        scene.draw(|ui, area| {
            tree.draw(ui, area, &state, &nodes);
        });
        black_box(state.toggle(StableItemKey::num(0)));
        scene.draw(|ui, area| {
            tree.draw(ui, area, &state, &nodes);
        });
    });
    (s, node_accesses.get())
}

#[test]
fn tree_100k_nodes_flatten() {
    let _g = lock();
    let (s, node_accesses) = bench_tree_toggle(big(100_000), 10);
    report("tree_100k_nodes_flatten", &s);
    assert_eq!(
        node_accesses, 0,
        "expand/collapse rescanned the borrowed 100 k source"
    );
    assert!(
        s.allocs < 2 * 10 * 40,
        "expand/collapse pair exceeded 10 x viewport per toggle: {} allocs",
        s.allocs
    );
}

fn bench_tree_render(nodes: &[PerfTreeNode]) -> (Stats, usize, usize, usize) {
    let node_accesses = Cell::new(0usize);
    let painted = Cell::new(0usize);
    let node = |item: &PerfTreeNode| {
        node_accesses.set(node_accesses.get().saturating_add(1));
        if item.parent {
            TreeNode::parent(item.depth)
        } else {
            TreeNode::leaf(item.depth)
        }
        .keyed(StableItemKey::num(item.key))
    };
    let row = |_: &PerfTreeNode, _: &mut junie_tui::RowUi<'_>| {
        painted.set(painted.get().saturating_add(1));
    };
    let tree = Tree::new(TREE_ID).node(&node).row(row);
    let mut state = TreeState::new();
    state.expand(StableItemKey::num(0));
    let mut scene = Scene::new("tree-render", Theme::junie(), ColorLevel::TrueColor, 80, 40);
    scene.draw(|ui, area| {
        tree.draw(ui, area, &state, nodes);
    });

    node_accesses.set(0);
    painted.set(0);
    scene.draw(|ui, area| {
        tree.draw(ui, area, &state, nodes);
    });
    let probe_nodes = node_accesses.get();
    let probe_rows = painted.get();
    node_accesses.set(0);
    painted.set(0);
    let s = bench(2, iters(50), &mut || {
        scene.draw(|ui, area| {
            black_box(tree.draw(ui, area, &state, nodes));
        });
    });
    let regions = scene.registry().map_or(0, Registry::len);
    (s.with_regions(regions, 1), probe_nodes, probe_rows, regions)
}

#[test]
fn tree_100k_nodes_render() {
    let _g = lock();
    let small = perf_tree_nodes(big(1_000));
    let large = perf_tree_nodes(big(100_000));
    let (control, control_nodes, control_rows, control_regions) = bench_tree_render(&small);
    let (s, node_accesses, painted, regions) = bench_tree_render(&large);
    report("tree_100k_nodes_render", &s);
    assert_eq!(node_accesses, 0, "warmed draw rescanned the source");
    assert_eq!(control_nodes, 0, "control draw rescanned the source");
    assert_eq!(painted, control_rows, "row work grew with node count");
    assert_eq!(regions, control_regions, "regions grew with node count");
    assert_eq!(s.allocs, control.allocs, "allocations grew with node count");
    check_ratio(
        "tree_100k_vs_1k_render",
        s.ns,
        control.ns,
        1.5,
        env_flag("PERF_STRICT"),
    );
}

#[test]
fn key_tree_toggle_10k() {
    let _g = lock();
    let n = big(10_000);
    let (s, node_accesses) = bench_tree_toggle(n, 100);
    println!("PERF-NOTE key_tree_toggle_10k rows={n} (collapse+expand pair)");
    report("key_tree_toggle_10k", &s);
    assert_eq!(
        node_accesses, 0,
        "stable-key toggle rescanned the borrowed source"
    );
    assert!(
        s.allocs < 2 * 10 * 40,
        "stable-key toggle pair exceeded 10 x viewport per toggle"
    );
}

#[derive(Clone, Copy)]
struct PerfStep {
    key: u64,
    state: StepState,
}

fn bench_steps_render(n: usize) -> (Stats, usize, usize, usize) {
    let mut items = vec![
        PerfStep {
            key: 0,
            state: StepState::Done,
        };
        n
    ];
    if let Some(first) = items.first_mut() {
        first.state = StepState::Running;
    }
    for (index, item) in items.iter_mut().enumerate() {
        item.key = index as u64;
    }
    let state_accesses = Cell::new(0usize);
    let painted = Cell::new(0usize);
    let step = |item: &PerfStep| {
        state_accesses.set(state_accesses.get().saturating_add(1));
        item.state
    };
    let row = |_: &PerfStep, _: &mut junie_tui::RowUi<'_>| {
        painted.set(painted.get().saturating_add(1));
    };
    let steps = Steps::new(STEPS_ID)
        .key(|item: &PerfStep| StableItemKey::num(item.key))
        .step(&step)
        .row(row);
    let state = StepsState::new();
    let mut scene = Scene::new(
        "steps-render",
        Theme::junie(),
        ColorLevel::TrueColor,
        80,
        40,
    );
    scene.draw(|ui, area| {
        steps.draw(ui, area, &state, &items);
    });

    state_accesses.set(0);
    painted.set(0);
    scene.draw(|ui, area| {
        steps.draw(ui, area, &state, &items);
    });
    let probe_states = state_accesses.get();
    let probe_rows = painted.get();
    state_accesses.set(0);
    painted.set(0);
    let s = bench(2, iters(50), &mut || {
        scene.draw(|ui, area| {
            black_box(steps.draw(ui, area, &state, &items));
        });
    });
    let regions = scene.registry().map_or(0, Registry::len);
    (s, probe_states, probe_rows, regions)
}

#[test]
fn steps_100k_rows_render() {
    let _g = lock();
    let (control, control_states, control_rows, control_regions) = bench_steps_render(big(1_000));
    let (s, state_accesses, painted, regions) = bench_steps_render(big(100_000));
    report("steps_100k_rows_render", &s);
    assert_eq!(painted, control_rows, "row work grew with step count");
    assert_eq!(regions, control_regions, "regions grew with step count");
    assert_eq!(
        state_accesses, control_states,
        "frontier work grew with step count"
    );
    assert!(
        state_accesses <= painted.saturating_add(2),
        "warmed frontier scanned beyond viewport: {state_accesses} state reads for {painted} rows"
    );
    assert_eq!(s.allocs, control.allocs, "allocations grew with step count");
    check_ratio(
        "steps_100k_vs_1k_render",
        s.ns,
        control.ns,
        1.5,
        env_flag("PERF_STRICT"),
    );
}

const GRID_COLUMNS: [Column<'static>; 12] = [
    Column::new(ColumnKey::num(0), "c00"),
    Column::new(ColumnKey::num(1), "c01"),
    Column::new(ColumnKey::num(2), "c02"),
    Column::new(ColumnKey::num(3), "c03"),
    Column::new(ColumnKey::num(4), "c04"),
    Column::new(ColumnKey::num(5), "c05"),
    Column::new(ColumnKey::num(6), "c06"),
    Column::new(ColumnKey::num(7), "c07"),
    Column::new(ColumnKey::num(8), "c08"),
    Column::new(ColumnKey::num(9), "c09"),
    Column::new(ColumnKey::num(10), "c10"),
    Column::new(ColumnKey::num(11), "c11"),
];

struct PerfGridModel {
    rows: usize,
    cols: usize,
    cells: Vec<String>,
    cell_accesses: Cell<usize>,
    max_row: Cell<usize>,
}

impl PerfGridModel {
    fn load(rows: usize, cols: usize) -> Self {
        let mut cells = Vec::with_capacity(rows.saturating_mul(cols));
        for row in 0..rows {
            for col in 0..cols {
                let mut text = String::with_capacity(8);
                write!(&mut text, "r{row:03}c{col:02}").expect("writing to a String cannot fail");
                cells.push(text);
            }
        }
        PerfGridModel {
            rows,
            cols,
            cells,
            cell_accesses: Cell::new(0),
            max_row: Cell::new(0),
        }
    }

    fn reset_accesses(&self) {
        self.cell_accesses.set(0);
        self.max_row.set(0);
    }
}

impl GridModel for PerfGridModel {
    fn row_count(&self) -> usize {
        self.rows
    }

    fn row_key(&self, row: usize) -> StableItemKey {
        StableItemKey::index(row)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        self.cell_accesses
            .set(self.cell_accesses.get().saturating_add(1));
        self.max_row.set(self.max_row.get().max(row));
        (col < self.cols)
            .then(|| row.saturating_mul(self.cols).saturating_add(col))
            .and_then(|index| self.cells.get(index))
            .map(|text| CellRef::new(text))
    }
}

fn bench_grid_render(rows: usize) -> (Stats, usize, usize, usize) {
    let model = PerfGridModel::load(rows, GRID_COLUMNS.len());
    let grid = Grid::new(GRID_ID, &GRID_COLUMNS);
    let state = GridState::default();
    let mut scene = Scene::new(
        "grid-render",
        Theme::junie(),
        ColorLevel::TrueColor,
        120,
        30,
    );
    scene.draw(|ui, area| {
        grid.draw(ui, area, &state, &model);
    });

    model.reset_accesses();
    scene.draw(|ui, area| {
        grid.draw(ui, area, &state, &model);
    });
    let probe_cells = model.cell_accesses.get();
    let probe_max_row = model.max_row.get();
    model.reset_accesses();
    let s = bench(2, iters(200), &mut || {
        scene.draw(|ui, area| {
            black_box(grid.draw(ui, area, &state, &model));
        });
    });
    let regions = scene.registry().map_or(0, Registry::len);
    (
        s.with_regions(regions, 1),
        probe_cells,
        probe_max_row,
        regions,
    )
}

#[test]
fn grid_500x12_render() {
    let _g = lock();
    let (control, control_cells, control_max_row, control_regions) = bench_grid_render(50);
    let (s, cell_accesses, max_row, regions) = bench_grid_render(500);
    report("grid_500x12_render", &s);
    assert!(s.allocs < 100, "grid frame allocated {} times", s.allocs);
    assert_eq!(
        cell_accesses, control_cells,
        "cell reads grew with row count"
    );
    assert_eq!(max_row, control_max_row, "grid sampled beyond viewport");
    assert!(max_row < 30, "grid touched off-screen row {max_row}");
    assert_eq!(regions, control_regions, "regions grew with row count");
    check_ratio(
        "grid_500_vs_50_render",
        s.ns,
        control.ns,
        1.5,
        env_flag("PERF_STRICT"),
    );
}

/// One pre-formatting conversion: exactly one owned string per cell plus the
/// flat storage allocation. This is the library-side witness for §20.9-11;
/// `TablePro` keeps the application-level benchmark with this same name.
#[test]
fn grid_500x12_load() {
    let _g = lock();
    let mut loaded = None;
    let s = measure_once(&mut || {
        loaded = Some(PerfGridModel::load(500, 12));
    });
    report("grid_500x12_load", &s);
    let model = loaded.as_ref().expect("measured model");
    assert_eq!(model.cells.len(), 6_000);
    assert!(
        s.allocs < 8_000,
        "one owned conversion exceeded 8 000 allocations: {}",
        s.allocs
    );
}
