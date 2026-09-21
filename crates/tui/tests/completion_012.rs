//! TASK-012 regression and mutation witnesses: pure layout clipping and
//! split geometry (`layout.rs`, `measure.rs`, `components/split.rs`).
//!
//! One separately named case per witness boundary in
//! `trusted/source-witnesses.md` (W-012-01…W-012-08): allocation, insets,
//! measure purity, seam geometry, split modes, empty callbacks, first-frame
//! facts and axis-specific minima. Positive cases pin the exact source rule;
//! negative cases attempt escaped paint/registration, stale seam hits and a
//! shared always-first fallback, and must fail loudly if the behavior drifts.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::too_many_lines
    )
)]

use std::cell::Cell;

use junie_tui::layout::{
    Insets, Maximized, SplitAxis, SplitModel, Track, columns, columns_measured, distribute, inset,
    rows, rows_measured,
};
use junie_tui::{
    App, Constraints, Cx, Family, Focusability, FrameRead, GlyphRole, Id, KeyCode, MouseKind, Part,
    PartRef, Rect, Response, SplitAction, SplitPane, SplitPaneState, StateFlags, Theme, Ui,
    UpdateCause, Variant, truncate, wrap, wrapped_rows,
};
use junie_tui_testing::Harness;
use junie_tui_testing::perf::{Counting, allocs};

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const ID: Id = Id::root("completion012.split");
const EVIL: Id = Id::root("completion012.evil");

const STATES: [StateFlags; 6] = [
    StateFlags::empty(),
    StateFlags::FOCUSED,
    StateFlags::HOVERED,
    StateFlags::PRESSED,
    StateFlags::DISABLED,
    StateFlags::SELECTED,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SeamCfg {
    Full,
    End(u16),
}

struct Page {
    axis: SplitAxis,
    area: Rect,
    gap: u16,
    min_first: u16,
    min_second: u16,
    resizable: bool,
    seam: SeamCfg,
    evil: bool,
    state: SplitPaneState,
    bodies: Cell<Option<(Rect, Rect)>>,
    draws: Cell<u32>,
    calls: Cell<u32>,
    answer: Cell<i32>,
    logical: Option<Rect>,
    last: Response<SplitAction>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            axis: SplitAxis::Horizontal,
            area: Rect::new(0, 0, 40, 10),
            gap: 1,
            min_first: 1,
            min_second: 1,
            resizable: true,
            seam: SeamCfg::Full,
            evil: false,
            state: SplitPaneState::default(),
            bodies: Cell::new(None),
            draws: Cell::new(0),
            calls: Cell::new(0),
            answer: Cell::new(0),
            logical: None,
            last: Response::default(),
        }
    }
}

impl Page {
    fn split(&self) -> SplitPane<'static> {
        let pane = SplitPane::new(ID, self.axis)
            .gap(self.gap)
            .min_first(self.min_first)
            .min_second(self.min_second)
            .resizable(self.resizable);
        match self.seam {
            SeamCfg::Full => pane,
            SeamCfg::End(n) => pane.seam_end(n),
        }
    }

    fn bodies(&self) -> (Rect, Rect) {
        self.bodies.get().expect("split body ran")
    }
}

impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            cx.focus(ID);
        }
        self.logical = cx.layout(ID).and_then(|facts| facts.logical_area);
        let r = self.split().update(cx, &mut self.state);
        self.last = r.clone();
        r.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.draws.set(self.draws.get() + 1);
        let answer = self
            .split()
            .draw(ui, self.area, &self.state, |ui, first, second| {
                self.calls.set(self.calls.get() + 1);
                self.bodies.set(Some((first, second)));
                if self.evil {
                    let style = ui.surface_style();
                    ui.paint_str(ui.full(), "EVIL", style);
                    ui.register_control(EVIL, ui.full(), Focusability::Focusable);
                }
                17
            });
        self.answer.set(answer);
    }
}

fn harness(page: Page, w: u16, h: u16) -> Harness<Page> {
    Harness::new(page, Theme::junie(), w, h)
}

fn seam(h: &Harness<Page>) -> Option<Rect> {
    h.area_of_part(ID, PartRef::of(Part::SEAM))
}

fn assert_blank(h: &Harness<Page>, w: u16, hh: u16) {
    for y in 0..hh {
        for x in 0..w {
            assert_eq!(h.cell(x, y).symbol(), " ", "painted cell at {x},{y}");
        }
    }
}

fn assert_inside(r: Rect, area: Rect, what: &str) {
    assert!(
        r.x >= area.x && r.y >= area.y && r.right() <= area.right() && r.bottom() <= area.bottom(),
        "{what} {r:?} escapes {area:?}"
    );
}

// ── W-012-01 allocation ──

/// Fixed/Flex/Auto and measured-Auto allocation answer exact widths: the
/// unmeasured Auto takes one cell beside Flex, the measured one its natural
/// size, and leftovers go to the earliest flexible tracks.
#[test]
fn fixed_flex_auto_allocate_exact_widths() {
    let tracks = [Track::Fixed(3), Track::Flex(1), Track::Flex(2), Track::Auto];
    assert_eq!(distribute(12, &tracks, 0, None), vec![3, 3, 5, 1]);
    assert_eq!(
        distribute(12, &tracks, 0, Some(&[0, 0, 0, 5])),
        vec![3, 2, 2, 5]
    );
    // no Flex: unmeasured Autos share the remainder equally
    assert_eq!(
        distribute(10, &[Track::Auto, Track::Auto], 0, None),
        vec![5, 5]
    );
    // measured columns take the natural width on the horizontal axis too
    let cols = columns_measured(
        Rect::new(0, 0, 30, 1),
        &[Track::Auto, Track::Fixed(1), Track::Flex(1)],
        1,
        &[4, 0, 0],
    );
    assert_eq!(
        cols.iter().map(|r| r.width).collect::<Vec<_>>(),
        vec![4, 1, 23]
    );
    let rows_m = rows_measured(
        Rect::new(0, 0, 10, 10),
        &[Track::Auto, Track::Flex(1)],
        &[4, 0],
    );
    assert_eq!(
        rows_m.iter().map(|r| r.height).collect::<Vec<_>>(),
        vec![4, 6]
    );
}

/// The allocation matrix over extents 0..=12, 40 and 120 at zero and nonzero
/// origins: every rect is a half-open subrect of its area, siblings never
/// overlap, and the answer is deterministic.
#[test]
fn allocation_matrix_stays_contained_half_open_and_deterministic() {
    let sets: [&[Track]; 4] = [
        &[Track::Fixed(3), Track::Flex(1), Track::Flex(2), Track::Auto],
        &[Track::Auto, Track::Auto],
        &[Track::Fixed(5)],
        &[Track::Flex(1)],
    ];
    let mut extents: Vec<u16> = (0..=12).collect();
    extents.extend([40, 120]);
    for &ox in &[0u16, 7] {
        for &oy in &[0u16, 3] {
            for &e in &extents {
                let area = Rect::new(ox, oy, e, e);
                for tracks in sets {
                    for spacing in [0u16, 1, 2] {
                        let cols = columns(area, tracks, spacing);
                        assert_eq!(cols.len(), tracks.len());
                        assert_eq!(cols, columns(area, tracks, spacing), "nondeterministic");
                        let mut total = 0u16;
                        let mut prev_right = area.x;
                        for r in &cols {
                            assert_inside(*r, area, "column");
                            assert_eq!(r.y, area.y);
                            assert_eq!(r.height, area.height);
                            assert!(r.x >= prev_right, "columns overlap in {area:?}");
                            prev_right = r.right();
                            total = total.saturating_add(r.width);
                        }
                        let gaps = spacing.saturating_mul((tracks.len() as u16).saturating_sub(1));
                        assert!(
                            total <= e.saturating_sub(gaps),
                            "columns over-allocate {area:?}"
                        );
                        let rows_r = rows(area, tracks);
                        assert_eq!(rows_r, rows(area, tracks), "nondeterministic");
                        let mut prev_bottom = area.y;
                        for r in &rows_r {
                            assert_inside(*r, area, "row");
                            assert!(r.y >= prev_bottom, "rows overlap in {area:?}");
                            prev_bottom = r.bottom();
                        }
                    }
                }
            }
        }
    }
}

/// Negative: absurd Fixed wants and zero extents clip instead of overflowing;
/// empty tracks stay anchored at the container edge, never at the screen corner.
#[test]
fn oversized_fixed_tracks_clip_without_overflow() {
    let area = Rect::new(7, 3, 3, 4);
    let cols = columns(area, &[Track::Fixed(60000), Track::Fixed(5)], 0);
    assert_eq!(cols[0], Rect::new(7, 3, 3, 4));
    assert_eq!(cols[1], Rect::new(10, 3, 0, 4));
    let empty = columns(Rect::new(7, 3, 0, 0), &[Track::Flex(1), Track::Auto], 2);
    assert!(empty.iter().all(|r| r.is_empty()));
    assert!(empty.iter().all(|r| r.x == 7 && r.y == 3));
    let gaps = columns(
        Rect::new(0, 0, 4, 1),
        &[Track::Flex(1), Track::Flex(1)],
        60000,
    );
    assert!(gaps.iter().all(|r| r.is_empty()));
}

// ── W-012-02 insets ──

/// Insets 0/1/2 against extents 0/1/2/3/40 at both origins stay contained;
/// exact asymmetric arithmetic is pinned.
#[test]
fn insets_saturate_and_stay_contained() {
    assert_eq!(
        inset(
            Rect::new(7, 3, 40, 40),
            Insets {
                l: 1,
                t: 2,
                r: 3,
                b: 4
            }
        ),
        Rect::new(8, 5, 36, 34)
    );
    for &ox in &[0u16, 7] {
        for &oy in &[0u16, 3] {
            for &e in &[0u16, 1, 2, 3, 40] {
                let area = Rect::new(ox, oy, e, e);
                for &n in &[0u16, 1, 2] {
                    for i in [
                        Insets::all(n),
                        Insets::symmetric(n, 1),
                        Insets {
                            l: n,
                            t: 2,
                            r: n,
                            b: 1,
                        },
                    ] {
                        assert_inside(inset(area, i), area, "inset");
                    }
                }
            }
        }
    }
}

/// Negative: `u16::MAX` insets saturate to empty rects anchored at the
/// container's far corner — `.is_empty()` alone cannot see the origin.
#[test]
fn huge_insets_anchor_empty_rects_inside_container() {
    assert_eq!(
        inset(Rect::new(7, 3, 2, 2), Insets::all(u16::MAX)),
        Rect::new(9, 5, 0, 0)
    );
    assert_eq!(
        inset(Rect::new(0, 0, 0, 0), Insets::all(u16::MAX)),
        Rect::new(0, 0, 0, 0)
    );
    for &e in &[0u16, 1, 2, 3, 40] {
        let area = Rect::new(7, 3, e, e);
        let r = inset(area, Insets::all(u16::MAX));
        assert!(r.is_empty());
        assert_inside(r, area, "saturated inset");
    }
}

/// Stacked rects share half-open edges: each rect's far edge plus spacing is
/// the next rect's near edge, with exact pinned coordinates.
#[test]
fn stacked_rects_share_half_open_edges() {
    let cols = columns(
        Rect::new(7, 3, 40, 10),
        &[Track::Fixed(5), Track::Flex(1), Track::Flex(1)],
        2,
    );
    assert_eq!(
        cols,
        vec![
            Rect::new(7, 3, 5, 10),
            Rect::new(14, 3, 16, 10),
            Rect::new(32, 3, 15, 10),
        ]
    );
    assert_eq!(cols[0].right() + 2, cols[1].x);
    assert_eq!(cols[1].right() + 2, cols[2].x);
    assert_eq!(cols[2].right(), 47);
    let stacked = rows(
        Rect::new(7, 3, 10, 10),
        &[Track::Fixed(3), Track::Flex(1), Track::Flex(2)],
    );
    assert_eq!(
        stacked.iter().map(|r| (r.y, r.height)).collect::<Vec<_>>(),
        vec![(3, 3), (6, 3), (9, 4)]
    );
    assert_eq!(stacked[0].bottom(), stacked[1].y);
    assert_eq!(stacked[1].bottom(), stacked[2].y);
    assert_eq!(stacked[2].bottom(), 13);
}

// ── W-012-03 measure purity ──

/// `SplitPane::measure` reports both minima plus the gap along the axis and
/// one cell across it; loose constraints clip the preferred size, tight ones
/// answer exactly the offer.
#[test]
fn split_measure_reports_minima_plus_gap() {
    use junie_tui_testing::Scene;
    let h = SplitPane::new(ID, SplitAxis::Horizontal)
        .gap(2)
        .min_first(3)
        .min_second(5);
    let v = SplitPane::new(ID, SplitAxis::Vertical)
        .gap(2)
        .min_first(3)
        .min_second(5);
    let mut scene = Scene::new(
        "completion012.measure",
        Theme::junie(),
        junie_tui::ColorLevel::TrueColor,
        40,
        10,
    );
    scene.draw(|ui, _| {
        let loose = Constraints::loose(40, 10);
        let hm = h.measure(ui, loose);
        assert_eq!(hm.min, (10, 1));
        assert_eq!(hm.preferred, (40, 10));
        let vm = v.measure(ui, loose);
        assert_eq!(vm.min, (1, 10));
        assert_eq!(vm.preferred, (40, 10));
        let tight = h.measure(ui, Constraints::tight(7, 7));
        assert_eq!((tight.min, tight.preferred), ((7, 7), (7, 7)));
        let small = h.measure(ui, Constraints::loose(6, 1));
        assert_eq!(small.min, (6, 1));
        assert_eq!(small.preferred, (6, 1));
    });
}

/// The `&self` measure query resolves through the same accumulation as the
/// painting query for the split seam in both themes and every state.
#[test]
fn resolve_matches_style_for_split_seam() {
    use junie_tui_testing::Scene;
    for theme in [Theme::junie(), Theme::paper()] {
        let mut scene = Scene::new(
            "completion012.accumulate",
            theme,
            junie_tui::ColorLevel::TrueColor,
            40,
            10,
        );
        scene.draw(|ui, _| {
            for &st in &STATES {
                assert_eq!(
                    ui.resolve(Family::SPLIT, Variant::DEFAULT, Part::SEAM, st),
                    ui.style(Family::SPLIT, Variant::DEFAULT, Part::SEAM, st),
                    "{st:?}"
                );
            }
        });
    }
}

/// Negative: 1000 measures with family/variant/part/state patches change no
/// durable state — identical sizes, untouched style cache, no styled-part
/// records, no allocations — and the next real draw is bit-identical.
#[test]
fn thousand_measures_leave_no_trace() {
    use junie_tui_testing::Scene;
    let sp = SplitPane::new(ID, SplitAxis::Horizontal)
        .gap(2)
        .min_first(3)
        .min_second(5);
    let st = SplitPaneState::default();
    let area = Rect::new(0, 0, 40, 10);
    let c = Constraints::loose(40, 10);
    let mut scene = Scene::new(
        "completion012.purity",
        Theme::junie(),
        junie_tui::ColorLevel::TrueColor,
        40,
        10,
    );
    scene.draw(|ui, _| {
        sp.draw(ui, area, &st, |_, _, _| {});
    });
    let painted = scene.text();
    assert!(!painted.trim().is_empty(), "the seam painted nothing");
    scene.draw(|ui, _| {
        sp.draw(ui, area, &st, |_, _, _| {});
        let parts = ui.styled_parts().len();
        assert!(parts > 0, "paint records styled parts");
        let cache = ui.style_cache_stats();
        let expected = sp.measure(ui, c);
        for i in 0..10u32 {
            let flags = STATES[(i as usize) % STATES.len()];
            let _ = ui.resolve(Family::SPLIT, Variant::DEFAULT, Part::SEAM, flags);
            let _ = ui.glyph_str(GlyphRole::FocusBar);
            let _ = sp.measure(ui, c);
        }
        let before = allocs();
        for i in 0..1000u32 {
            let flags = STATES[(i as usize) % STATES.len()];
            let _ = ui.resolve(Family::SPLIT, Variant::DEFAULT, Part::SEAM, flags);
            let _ = ui.glyph_str(GlyphRole::FocusBar);
            assert_eq!(sp.measure(ui, c), expected, "measure {i} drifted");
        }
        assert_eq!(allocs() - before, 0, "1000 measures allocated");
        assert_eq!(ui.styled_parts().len(), parts, "measure recorded parts");
        assert_eq!(ui.style_cache_stats(), cache, "measure touched the cache");
    });
    assert_eq!(scene.text(), painted, "repeat draw drifted");
}

// ── wrapping / truncation pinning at tiny widths ──

/// Row-count sizing agrees with the built lines at 0/1-cell widths, and
/// truncation degrades to an empty string at width zero without panicking.
#[test]
fn wrap_and_truncate_agree_at_tiny_widths() {
    for w in [0u16, 1, 2, 5] {
        for s in ["", "a", "hello world", "supercalifragilistic"] {
            assert_eq!(
                wrap(s, w).len() as u16,
                wrapped_rows(s, w),
                "wrap({s:?}, {w}) disagrees with its row count"
            );
        }
    }
    assert_eq!(truncate("hello", 0), "");
    assert_eq!(truncate("hello", 1), "…");
    assert_eq!(truncate("hi", 5), "hi");
}

// ── W-012-04 split seam ──

/// Both axes return exactly two logical bodies excluding the seam: panes plus
/// seam tile the container with no overlap and no gap, at zero and nonzero
/// origins, and every registration sits inside the area.
#[test]
fn panes_and_seam_tile_container_both_axes() {
    for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
        for area in [Rect::new(0, 0, 40, 10), Rect::new(7, 3, 40, 10)] {
            for percent in [5u8, 50, 95] {
                let h = harness(
                    Page {
                        axis,
                        area,
                        state: SplitPaneState::new(percent),
                        ..Page::default()
                    },
                    120,
                    40,
                );
                let (first, second) = h.app().bodies();
                let registered = seam(&h).expect("a live seam registers");
                assert_inside(first, area, "first pane");
                assert_inside(second, area, "second pane");
                assert_inside(registered, area, "seam");
                match axis {
                    SplitAxis::Horizontal => {
                        assert_eq!(first.x, area.x);
                        assert_eq!(registered.x, first.right());
                        assert_eq!(registered.width, 1);
                        assert_eq!(second.x, registered.right());
                        assert_eq!(second.right(), area.right());
                        assert_eq!((first.y, first.height), (area.y, area.height));
                    }
                    SplitAxis::Vertical => {
                        assert_eq!(first.y, area.y);
                        assert_eq!(registered.y, first.bottom());
                        assert_eq!(registered.height, 1);
                        assert_eq!(second.y, registered.bottom());
                        assert_eq!(second.bottom(), area.bottom());
                        assert_eq!((first.x, first.width), (area.x, area.width));
                    }
                }
                // every container cell belongs to exactly one of the three
                let mut seen = vec![0u32; (area.width * area.height) as usize];
                for r in [first, second, registered] {
                    for p in r.positions() {
                        let i = ((p.y - area.y) * area.width + (p.x - area.x)) as usize;
                        seen[i] += 1;
                    }
                }
                assert!(seen.iter().all(|&n| n == 1), "{axis:?} {percent}% {area:?}");
            }
        }
    }
}

/// Minima 0/1/3 and gaps 0/1/2 hold on both axes: panes honor their minima,
/// the seam is exactly the gap, and a zero gap registers no seam.
#[test]
fn minima_and_gaps_hold_across_matrix() {
    for axis in [SplitAxis::Horizontal, SplitAxis::Vertical] {
        for &min_first in &[0u16, 1, 3] {
            for &min_second in &[0u16, 1, 3] {
                for &gap in &[0u16, 1, 2] {
                    let area = Rect::new(2, 2, 40, 24);
                    let h = harness(
                        Page {
                            axis,
                            area,
                            gap,
                            min_first,
                            min_second,
                            ..Page::default()
                        },
                        120,
                        40,
                    );
                    let (first, second) = h.app().bodies();
                    let len = |r: Rect| match axis {
                        SplitAxis::Horizontal => r.width,
                        SplitAxis::Vertical => r.height,
                    };
                    assert!(len(first) >= min_first, "{axis:?} gap {gap}");
                    assert!(len(second) >= min_second, "{axis:?} gap {gap}");
                    match seam(&h) {
                        Some(s) => {
                            assert!(gap > 0, "zero gap registered a seam");
                            assert_eq!(len(s), gap);
                        }
                        None => assert_eq!(gap, 0, "live seam went unregistered"),
                    }
                }
            }
        }
    }
}

/// A drag preserves the pointer's grab offset inside a wide seam on both
/// axes: pressing one cell in moves the seam one cell less than pressing the
/// leading edge, for the same pointer target.
#[test]
fn drag_preserves_grab_offset_both_axes() {
    // horizontal: area 40 wide, gap 3, 50% → first 18, seam x 18..21
    for (press_x, percent, width) in [(18u16, 68u8, 25u16), (19u16, 65u8, 24u16)] {
        let mut h = harness(
            Page {
                gap: 3,
                ..Page::default()
            },
            40,
            10,
        );
        assert_eq!(seam(&h), Some(Rect::new(18, 0, 3, 10)));
        let _ = h.mouse(MouseKind::Down, press_x, 5);
        assert_eq!(h.runtime().capture_owner(), Some(ID));
        let _ = h.mouse(MouseKind::Drag, 25, 5);
        assert_eq!(h.app().state.percent(), percent, "grab at {press_x}");
        assert_eq!(h.app().bodies().0.width, width);
        let _ = h.mouse(MouseKind::Up, 25, 5);
        assert_eq!(h.runtime().capture_owner(), None);
    }
    // vertical: area 40 tall, gap 3, 50% → first 18, seam y 18..21
    let mut h = harness(
        Page {
            axis: SplitAxis::Vertical,
            area: Rect::new(0, 0, 10, 40),
            gap: 3,
            ..Page::default()
        },
        40,
        40,
    );
    assert_eq!(seam(&h), Some(Rect::new(0, 18, 10, 3)));
    let _ = h.mouse(MouseKind::Down, 5, 19);
    let _ = h.mouse(MouseKind::Drag, 5, 25);
    assert_eq!(h.app().state.percent(), 65);
    assert_eq!(h.app().bodies().0.height, 24);
    let _ = h.mouse(MouseKind::Up, 5, 25);
}

/// Drag endpoints clamp to the minima; a repeated drag to the same position
/// is Consumed with no action, while a real move is Changed with `Resized`.
#[test]
fn drag_endpoints_clamp_and_repeats_consume() {
    let mut h = harness(
        Page {
            min_first: 4,
            min_second: 6,
            ..Page::default()
        },
        40,
        10,
    );
    // usable 39: dragging to 0 clamps the first pane to 4 (10%), to 39 to 33 (85%)
    let _ = h.mouse(MouseKind::Down, 19, 5);
    let _ = h.mouse(MouseKind::Drag, 0, 5);
    assert_eq!(h.app().state.percent(), 10);
    assert!(h.app().last.is_changed());
    assert_eq!(
        h.app().last.clone().into_action(),
        Some(SplitAction::Resized(10))
    );
    let _ = h.mouse(MouseKind::Drag, 0, 5);
    assert_eq!(h.app().state.percent(), 10);
    assert!(!h.app().last.is_changed());
    assert!(h.app().last.is_consumed());
    assert_eq!(h.app().last.clone().into_action(), None);
    let _ = h.mouse(MouseKind::Drag, 39, 5);
    assert_eq!(h.app().state.percent(), 85);
    assert_eq!(
        h.app().last.clone().into_action(),
        Some(SplitAction::Resized(85))
    );
    let _ = h.mouse(MouseKind::Up, 39, 5);
}

/// Negative: degenerate dimensions register no seam; empty bodies stay
/// anchored at the container origin and collapsed survivors fill the area.
#[test]
fn degenerate_dimensions_register_no_seam() {
    for area in [
        Rect::new(0, 0, 0, 10),
        Rect::new(0, 0, 40, 0),
        Rect::new(7, 3, 0, 0),
    ] {
        let h = harness(
            Page {
                area,
                ..Page::default()
            },
            40,
            10,
        );
        assert_eq!(seam(&h), None, "{area:?}");
        let empty = Rect::new(area.x, area.y, 0, 0);
        assert_eq!(h.app().bodies(), (empty, empty));
        assert_eq!(h.app().answer.get(), 17);
    }
    // one live cell on a horizontal split collapses to the second pane
    for area in [Rect::new(0, 0, 1, 10), Rect::new(7, 3, 1, 1)] {
        let h = harness(
            Page {
                area,
                ..Page::default()
            },
            40,
            10,
        );
        assert_eq!(seam(&h), None, "{area:?}");
        assert_eq!(
            h.app().bodies(),
            (Rect::new(area.x, area.y, 0, area.height), area),
            "{area:?}"
        );
    }
}

/// A split hanging over the buffer edge registers only the visible seam
/// intersection; nothing outside the buffer is painted or hittable.
#[test]
fn partial_clip_registers_visible_intersection_only() {
    let mut h = harness(
        Page {
            area: Rect::new(30, 2, 20, 6),
            ..Page::default()
        },
        40,
        10,
    );
    // usable 19, first 9 → seam x 39, one visible column
    assert_eq!(seam(&h), Some(Rect::new(39, 2, 1, 6)));
    let _ = h.mouse(MouseKind::Down, 39, 3);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Up, 39, 3);
    let outside = harness(
        Page {
            area: Rect::new(u16::MAX, u16::MAX, 4, 4),
            ..Page::default()
        },
        40,
        10,
    );
    assert_eq!(seam(&outside), None);
    assert_blank(&outside, 40, 10);
}

/// Negative: releasing outside the seam still ends the capture; the percent
/// is the clamped endpoint, not a stale or repeated value.
#[test]
fn release_outside_seam_clears_capture() {
    let mut h = harness(Page::default(), 40, 10);
    let _ = h.mouse(MouseKind::Down, 19, 5);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Drag, 39, 5);
    assert_eq!(h.app().state.percent(), 95);
    let _ = h.mouse(MouseKind::Up, 39, 5);
    assert_eq!(h.runtime().capture_owner(), None);
    assert_eq!(h.app().state.percent(), 95);
    // a drag with no live capture changes nothing
    let _ = h.mouse(MouseKind::Drag, 5, 5);
    assert_eq!(h.app().state.percent(), 95);
}

// ── W-012-05 split modes ──

/// Normal → maximize each → restore → minima-collapse → restore → keyboard
/// resize: saved ratios, logical order and minima survive every transition.
#[test]
fn collapse_maximize_resize_restore_cycle() {
    let area = Rect::new(0, 0, 120, 40);
    let mut h = harness(
        Page {
            area,
            state: SplitPaneState::new(30),
            ..Page::default()
        },
        120,
        40,
    );
    // usable 119, 30% → first 35, seam x 35..36
    assert_eq!(h.app().bodies().0, Rect::new(0, 0, 35, 40));
    assert_eq!(seam(&h), Some(Rect::new(35, 0, 1, 40)));
    // maximize first: the second pane collapses to a far-edge empty
    h.app_mut().state.toggle_max(Maximized::First);
    h.draw();
    assert_eq!(h.app().bodies(), (area, Rect::new(120, 0, 0, 40)));
    assert_eq!(seam(&h), None);
    assert_eq!(h.app().state.percent(), 30);
    // restore: the exact seam returns
    h.app_mut().state.toggle_max(Maximized::First);
    h.draw();
    assert_eq!(h.app().bodies().0.width, 35);
    assert_eq!(seam(&h), Some(Rect::new(35, 0, 1, 40)));
    // maximize second: mirror image
    h.app_mut().state.toggle_max(Maximized::Second);
    h.draw();
    assert_eq!(h.app().bodies(), (Rect::new(0, 0, 0, 40), area));
    assert_eq!(seam(&h), None);
    h.app_mut().state.toggle_max(Maximized::Second);
    h.draw();
    assert_eq!(h.app().bodies().0.width, 35);
    // minima collapse keeps the second pane and the saved ratio
    h.app_mut().min_first = 70;
    h.app_mut().min_second = 70;
    h.draw();
    assert_eq!(h.app().bodies().1, area);
    assert!(h.app().bodies().0.is_empty());
    assert_eq!(seam(&h), None);
    assert_eq!(h.app().state.percent(), 30);
    h.app_mut().min_first = 1;
    h.app_mut().min_second = 1;
    h.draw();
    assert_eq!(h.app().bodies().0.width, 35);
    assert_eq!(seam(&h), Some(Rect::new(35, 0, 1, 40)));
    // keyboard resize at a width where one cell moves the percent: usable 39
    // at 30% → first 11; → grows to 12 (31%), Home balances back to 50%
    h.app_mut().area = Rect::new(0, 0, 40, 10);
    h.draw();
    assert_eq!(h.app().bodies().0.width, 11);
    let _ = h.key(KeyCode::Right);
    assert_eq!(h.app().state.percent(), 31);
    assert_eq!(h.app().bodies().0.width, 12);
    assert_eq!(
        h.app().last.clone().into_action(),
        Some(SplitAction::Resized(31))
    );
    let _ = h.key(KeyCode::Home);
    assert_eq!(h.app().state.percent(), 50);
    assert_eq!(h.app().bodies().0.width, 19);
}

/// Negative: after a resize, the old seam position is no longer a hit target
/// — only the freshly published strip captures.
#[test]
fn stale_seam_hits_rejected_after_resize() {
    let mut h = harness(Page::default(), 40, 10);
    assert_eq!(seam(&h), Some(Rect::new(19, 0, 1, 10)));
    h.app_mut().area = Rect::new(0, 0, 20, 10);
    h.draw();
    assert_eq!(seam(&h), Some(Rect::new(9, 0, 1, 10)));
    let _ = h.mouse(MouseKind::Down, 19, 5);
    assert_eq!(h.runtime().capture_owner(), None, "stale seam captured");
    let _ = h.mouse(MouseKind::Up, 19, 5);
    let _ = h.mouse(MouseKind::Down, 9, 5);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Up, 9, 5);
}

// ── W-012-06 empty callbacks ──

/// Negative: an empty body runs exactly once per draw, returns its sentinel,
/// and its escaped paint/registration attempts stay clipped — the frame
/// registers nothing and paints nothing.
#[test]
fn empty_body_runs_once_and_contains_escape_attempts() {
    let h = harness(
        Page {
            area: Rect::new(9, 7, 0, 0),
            evil: true,
            ..Page::default()
        },
        40,
        10,
    );
    assert!(h.app().draws.get() > 0);
    assert_eq!(
        h.app().calls.get(),
        h.app().draws.get(),
        "body ran != once per draw"
    );
    assert_eq!(h.app().answer.get(), 17);
    let empty = Rect::new(9, 7, 0, 0);
    assert_eq!(h.app().bodies(), (empty, empty));
    assert_eq!(seam(&h), None);
    assert_eq!(h.area_of(EVIL), None);
    assert!(!h.ring().is_registered(ID));
    assert!(!h.ring().is_registered(EVIL));
    assert_blank(&h, 40, 10);
    assert!(h.runtime().diagnostics().is_empty());
}

// ── W-012-07 first-frame facts ──

/// The first frame publishes logical layout facts for the current allocation
/// before any label paints, and facts plus seam track 120→40→1→120 resizes.
#[test]
fn first_frame_facts_track_allocation_across_resizes() {
    let area = Rect::new(0, 0, 120, 40);
    let mut h = harness(
        Page {
            area,
            ..Page::default()
        },
        120,
        40,
    );
    let _ = h.key(junie_tui::KeyCode::Char('x'));
    assert_eq!(h.app().logical, Some(area), "first-frame facts stale");
    for (w, seam_x) in [(40u16, Some(19u16)), (1u16, None), (120u16, Some(59u16))] {
        h.app_mut().area = Rect::new(0, 0, w, 40);
        h.draw();
        let _ = h.key(junie_tui::KeyCode::Char('x'));
        assert_eq!(h.app().logical, Some(Rect::new(0, 0, w, 40)));
        match seam_x {
            Some(x) => assert_eq!(seam(&h), Some(Rect::new(x, 0, 1, 40))),
            None => assert_eq!(seam(&h), None),
        }
    }
}

// ── W-012-08 axis-specific minima ──

/// Minima 50/50 against 80 available: horizontal keeps the second pane,
/// vertical keeps the first, at zero and nonzero origins; enlarging to 120
/// restores both panes at the saved ratio and shrinking collapses again.
#[test]
fn axis_minima_keep_source_side_and_survive_roundtrip() {
    for (axis, area) in [
        (SplitAxis::Horizontal, Rect::new(0, 0, 80, 40)),
        (SplitAxis::Horizontal, Rect::new(7, 3, 80, 40)),
        (SplitAxis::Vertical, Rect::new(0, 0, 40, 80)),
        (SplitAxis::Vertical, Rect::new(7, 3, 40, 80)),
    ] {
        let mut h = harness(
            Page {
                axis,
                area,
                min_first: 50,
                min_second: 50,
                ..Page::default()
            },
            130,
            130,
        );
        let (first, second) = h.app().bodies();
        match axis {
            SplitAxis::Horizontal => {
                assert_eq!(second, area, "{area:?}: second did not survive");
                assert_eq!(first, Rect::new(area.x, area.y, 0, area.height));
            }
            SplitAxis::Vertical => {
                assert_eq!(first, area, "{area:?}: first did not survive");
                assert_eq!(second, Rect::new(area.x, area.bottom(), area.width, 0));
            }
        }
        assert_eq!(seam(&h), None);
        assert_eq!(h.app().state.percent(), 50, "collapse lost the ratio");
        // enlarge: both panes return at the saved 50% ratio
        h.app_mut().area = match axis {
            SplitAxis::Horizontal => Rect::new(area.x, area.y, 120, area.height),
            SplitAxis::Vertical => Rect::new(area.x, area.y, area.width, 120),
        };
        h.draw();
        let (first, second) = h.app().bodies();
        assert!(
            !first.is_empty() && !second.is_empty(),
            "enlarge did not restore"
        );
        assert!(seam(&h).is_some(), "enlarge left no seam");
        match axis {
            SplitAxis::Horizontal => assert_eq!(first.width, 59),
            SplitAxis::Vertical => assert_eq!(first.height, 59),
        }
        // shrink again: the same source side collapses, ratio intact
        h.app_mut().area = area;
        h.draw();
        let (first, second) = h.app().bodies();
        match axis {
            SplitAxis::Horizontal => assert_eq!((first.is_empty(), second), (true, area)),
            SplitAxis::Vertical => assert_eq!((first, second.is_empty()), (area, true)),
        }
        assert_eq!(h.app().state.percent(), 50);
    }
}

/// Negative: a horizontal minima failure never keeps the first pane — the
/// shared always-first fallback is rejected, not averaged.
#[test]
fn horizontal_collapse_never_keeps_first() {
    let area = Rect::new(0, 0, 80, 8);
    let m = SplitModel::new(SplitAxis::Horizontal, 60, 50, 50);
    let (first, second) = m.layout(area, 1);
    assert_ne!(first, area, "the always-first fallback is back");
    assert!(first.is_empty());
    assert_eq!(second, area);
    let h = harness(
        Page {
            axis: SplitAxis::Horizontal,
            area,
            min_first: 50,
            min_second: 50,
            state: SplitPaneState::new(60),
            ..Page::default()
        },
        120,
        40,
    );
    assert_eq!(h.app().bodies().1, area);
    assert!(h.app().bodies().0.is_empty());
}

// ── R-003 offset seam hitboxes ──

/// A trailing one-cell seam in a two-cell gap is the only hit target: the
/// leading gap cell neither paints the seam nor captures the pointer.
#[test]
fn offset_seam_hitbox_matches_registered_strip() {
    let mut h = harness(
        Page {
            gap: 2,
            seam: SeamCfg::End(1),
            min_first: 4,
            min_second: 4,
            state: SplitPaneState::new(50),
            ..Page::default()
        },
        40,
        10,
    );
    // usable 37, first 18 → gap x 18..20, trailing seam cell x 20
    assert_eq!(seam(&h), Some(Rect::new(20, 0, 1, 10)));
    assert_eq!(h.cell(19, 3).symbol(), " ");
    assert_ne!(h.cell(20, 3).symbol(), " ");
    let _ = h.mouse(MouseKind::Down, 19, 3);
    assert_eq!(h.runtime().capture_owner(), None);
    let _ = h.mouse(MouseKind::Up, 19, 3);
    let _ = h.mouse(MouseKind::Down, 20, 3);
    assert_eq!(h.runtime().capture_owner(), Some(ID));
    let _ = h.mouse(MouseKind::Up, 20, 3);
    // the focus stop follows the registered strip, not the container
    assert_eq!(h.ring().entry(ID).map(|e| e.area), seam(&h));
}
