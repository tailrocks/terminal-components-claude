//! Library-level performance benchmarks (`COMPONENT_ARCHITECTURE.md` §16.6,
//! `docs/audit/performance-audit.md` §7.2 B–F) implementable on the
//! foundations. Run in release, single-threaded, with output visible:
//!
//! ```text
//! cargo test -p tui-next --test perf --release -- --test-threads=1 --nocapture
//! ```
//!
//! See `tui_next_testing::perf` for the environment knobs.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::arithmetic_side_effects,
        clippy::cast_lossless,
        clippy::print_stdout,
        clippy::format_push_string,
        clippy::items_after_statements,
        clippy::map_unwrap_or
    )
)]

use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};

use tui_next::{
    ActionKey, App, Axes, Binding, Checkbox, Chord, CodeDiagnostic, CodeEditor, CodeEditorState,
    CodeSeverity, ColorLevel, Cx, DiffLineKind, DiffRow, DiffSource, DiffView, DiffViewState,
    Family, FieldKind, FieldMut, FieldRef, FieldSpec, FocusRing, Focusability, Form, FormData,
    FormState, FrameRead, Headroom, Highlighter, HintBar, Id, Input, Intent, KeyCode, KeyModifiers,
    LayerId, Overlay, OverlayRule, Part, Position, Rect, Registry, Response, Role, Runtime, Slot,
    StateFlags, StylePatch, SyntaxRole, Theme, Ui, Variant,
};
use tui_next_testing::perf::{
    Counting, bench, check_ratio, env_flag, iters, lock, measure_once, report, unicode_line,
    unicode_line_inline,
};
use tui_next_testing::{NoApp, Scene};

#[global_allocator]
static GLOBAL: Counting = Counting;

// ------------------------------------------------------------ A. frames

#[test]
fn frame_testbackend_empty_120x40() {
    let _g = lock();
    let mut term =
        ratatui_core::terminal::Terminal::new(ratatui_core::backend::TestBackend::new(120, 40))
            .unwrap();
    let s = bench(1, iters(200), &mut || {
        term.draw(|_f| {}).unwrap();
    });
    report("frame_testbackend_empty_120x40", &s);
}

// ------------------------------------------------------------ B. events

#[test]
fn mouse_move_over_1000_regions() {
    let _g = lock();
    let mut plain = Registry::new(1);
    let mut layered = Registry::new(1);
    for y in 0..10u16 {
        for x in 0..100u16 {
            let id = Id::root("perf.region").index(usize::from(y * 100 + x));
            plain.register_control(id, Rect::new(x, y, 1, 1), LayerId::PAGE);
            layered.register_control(id, Rect::new(x, y, 1, 1), LayerId::PAGE);
        }
    }
    for i in 0..10u16 {
        layered.register_control(
            Id::root("perf.modal").index(usize::from(i)),
            Rect::new(i, 12, 1, 1),
            LayerId::PAGE,
        );
    }
    assert_eq!(plain.len(), 1000);
    let s = bench(1, iters(10), &mut || {
        for i in 0..5000u32 {
            let pos = Position::new((i % 125) as u16, ((i / 125) % 20) as u16);
            black_box(plain.hit(pos));
            black_box(layered.hit(pos));
        }
    });
    report("mouse_move_over_1000_regions", &s);
    assert_eq!(s.allocs, 0, "hit-testing must not allocate");
}

#[test]
fn focus_tab_traversal_ring_200() {
    let _g = lock();
    // the ring is built the way the runtime builds it: 200 controls registered in draw order
    let mut scene = Scene::new("ring200", Theme::junie(), ColorLevel::TrueColor, 200, 1);
    scene.draw(|ui, _| {
        for i in 0..200u16 {
            ui.register_control(
                Id::root("perf.stop").index(usize::from(i)),
                Rect::new(i, 0, 1, 1),
                Focusability::Focusable,
            );
        }
    });
    let ring: &FocusRing = scene.ring().expect("scene ring");
    assert_eq!(ring.reachable().count(), 200);
    let mut cur = None;
    let s = bench(1, iters(10), &mut || {
        for _ in 0..10_000 {
            cur = ring.next(cur);
        }
        black_box(cur);
    });
    report("focus_tab_traversal_ring_200", &s);
    assert_eq!(s.allocs, 0, "focus traversal must not allocate");
}

// ------------------------------------------------------------ C. styles

const STATES: [StateFlags; 8] = [
    StateFlags::empty(),
    StateFlags::FOCUSED,
    StateFlags::HOVERED,
    StateFlags::SELECTED,
    StateFlags::FOCUSED.union(StateFlags::SELECTED),
    StateFlags::PRESSED,
    StateFlags::DISABLED,
    StateFlags::ERROR,
];

fn resolve_10k(ui: &mut Ui<'_>) -> u64 {
    let mut acc = 0u64;
    for i in 0..2500usize {
        let s = STATES[i % STATES.len()];
        let row = ui
            .style(Family::LIST, Variant::DEFAULT, Part::CONTAINER, s)
            .style;
        let gutter = ui
            .style(Family::LIST, Variant::DEFAULT, Part::GUTTER, s)
            .style;
        let marker = ui
            .style(Family::LIST, Variant::DEFAULT, Part::MARKER, s)
            .style;
        let meta = ui
            .style(Family::LIST, Variant::DEFAULT, Part::META, s)
            .style;
        acc = acc.wrapping_add(fingerprint(row) ^ fingerprint(gutter));
        acc = acc.wrapping_add(fingerprint(marker) ^ fingerprint(meta));
    }
    acc
}

fn fingerprint(s: tui_next::Style) -> u64 {
    let f = s.fg.map(color_bits).unwrap_or(0);
    let b = s.bg.map(color_bits).unwrap_or(0);
    f ^ (b << 8) ^ ((s.add_modifier.bits() as u64) << 16)
}

fn color_bits(c: tui_next::Color) -> u64 {
    match c {
        tui_next::Color::Rgb(r, g, b) => ((r as u64) << 16) | ((g as u64) << 8) | b as u64,
        tui_next::Color::Indexed(i) => i as u64,
        _ => 1,
    }
}

#[test]
fn style_resolve_10k_parts() {
    let _g = lock();
    let mut scene = Scene::new(
        "style_resolve",
        Theme::junie(),
        ColorLevel::TrueColor,
        120,
        40,
    );
    scene.draw(|_, _| {});
    let s = bench(1, iters(10), &mut || {
        scene.draw(|ui, _| {
            black_box(resolve_10k(ui));
        });
    });
    report("style_resolve_10k_parts", &s);
    assert_eq!(s.allocs, 0, "style resolution must not allocate (R2)");
    // adjudication 2.8: the binding assertion is the memo's health, not a
    // per-query ns ratio. A broken cache key shows up here and nowhere else.
    let (hits, misses) = scene
        .runtime()
        .map(|rt: &Runtime<NoApp>| rt.style_cache_stats())
        .unwrap_or((0, 0));
    let total = hits + misses;
    assert!(total > 0, "no style queries were made");
    let rate = hits as f64 / total as f64;
    println!("PERF-CACHE style_resolve_10k_parts hits={hits} misses={misses} rate={rate:.3}");
    assert!(
        rate >= 0.90,
        "style memo hit rate {rate:.3} < 0.90 (hits={hits}, misses={misses})"
    );
    if env_flag("PERF_STRICT") {
        // Adjudication O4a correction 2: the *binding* style budget lives
        // here, not in `style_resolve_per_frame`'s differential. This is a
        // pure resolution loop of exactly 10 000 queries with no differencing,
        // so it is the low-noise measurement; §25.8's own arithmetic —
        // "≈ 13 ns per query × ~2 000 queries per realistic frame ≈ 26 µs,
        // under 0.2 % of a 16 ms budget" — turned into code is
        // `ns / 10 000 × 2 000 <= 32 000`, i.e. <= 16.0 ns per query.
        const QUERIES: u128 = 10_000;
        const FRAME_QUERIES: u128 = 2_000;
        const BUDGET_NS: u128 = 32_000;
        let per_frame_2k = s.ns / QUERIES * FRAME_QUERIES;
        assert!(
            per_frame_2k <= BUDGET_NS,
            "style resolution is {} ns/query, so a 2 000-query frame costs \
             {per_frame_2k} ns, over the 32 µs (0.2 % of 16 ms) budget",
            s.ns / QUERIES
        );
    }
}

/// Adjudication 2.8: §20.9-1's "ns ≤ 2× the pre-refactor `Theme::row`+`gutter`
/// baseline" is struck — it compared a 30-field `Copy` read with a six-level
/// precedence resolution and was unmeetable by construction. The bound that
/// replaces it is a **per-frame budget**: style resolution is a small share of
/// a realistic frame.
///
/// §16.6 names `frame_showcase_lists_120x40` as the subject; that benchmark
/// needs the showcase application (Slice 5). The stand-in is the same shape
/// built from foundations only: a 40-row `RowUi` frame at 120×40. The two
/// measured frames paint **identically** and differ only in whether the five
/// part styles are resolved per row or hoisted, so the difference is the
/// style-resolution cost and nothing else.
///
/// This test deliberately calls no `report`, so it carries **no** line in
/// `perf_baseline.txt` (named there in the `#` header): it measures a
/// difference of two medians, and a baselined `ns` for a differential invites
/// a meaningless `× 1.2` regression check (Adjudication O4a correction 3).
#[test]
fn style_resolve_per_frame() {
    let _g = lock();
    let mut scene = Scene::new(
        "style_per_frame",
        Theme::junie(),
        ColorLevel::TrueColor,
        120,
        40,
    );
    const PARTS: [Part; 5] = [
        Part::CONTAINER,
        Part::GUTTER,
        Part::MARKER,
        Part::LABEL,
        Part::META,
    ];
    const LABEL: &str = "a list row with a reasonable amount of label text";

    let paint_row = |ui: &mut Ui<'_>, row: Rect, st: &[tui_next::Style; 5]| {
        ui.fill(row, st[0]);
        let gutter = Rect::new(row.x, row.y, 1, 1);
        ui.glyph(gutter, tui_next::GlyphRole::FocusBar, st[1]);
        let marker = Rect::new(row.x + 1, row.y, 1, 1);
        ui.glyph(marker, tui_next::GlyphRole::Chosen, st[2]);
        let label = Rect::new(row.x + 3, row.y, row.width - 6, 1);
        ui.paint_str(label, LABEL, st[3]);
        let meta = Rect::new(row.right() - 2, row.y, 2, 1);
        ui.paint_str(meta, "42", st[4]);
    };

    // A: resolve the five parts for every row
    let resolved_per_row = |ui: &mut Ui<'_>, area: Rect| {
        for (i, row) in area.rows().enumerate() {
            let flags = STATES[i % STATES.len()];
            let mut st = [tui_next::Style::new(); 5];
            for (slot, p) in st.iter_mut().zip(PARTS) {
                *slot = ui.style(Family::LIST, Variant::DEFAULT, p, flags).style;
            }
            paint_row(ui, row, &st);
        }
    };
    // B: the identical painting with the styles hoisted out of the loop
    let hoisted = |ui: &mut Ui<'_>, area: Rect| {
        let mut by_state = [[tui_next::Style::new(); 5]; STATES.len()];
        for (slot, flags) in by_state.iter_mut().zip(STATES) {
            for (s, p) in slot.iter_mut().zip(PARTS) {
                *s = ui.style(Family::LIST, Variant::DEFAULT, p, flags).style;
            }
        }
        for (i, row) in area.rows().enumerate() {
            paint_row(ui, row, &by_state[i % STATES.len()]);
        }
    };

    scene.draw(resolved_per_row);
    scene.draw(hoisted);
    let a = bench(3, iters(50), &mut || scene.draw(resolved_per_row));
    let b = bench(3, iters(50), &mut || scene.draw(hoisted));
    let resolution_ns = a.ns.saturating_sub(b.ns);
    let share = resolution_ns as f64 / a.ns.max(1) as f64;
    println!(
        "PERF style_resolve_per_frame ns={} hoisted_ns={} resolution_ns={resolution_ns} \
         share={share:.3} queries_a=200 queries_b=40 delta=160",
        a.ns, b.ns
    );
    assert_eq!(a.allocs, 0);
    if env_flag("PERF_STRICT") {
        // Adjudication O4a correction 1: the difference covers `DELTA`
        // queries, not `QUERIES_A`. The old `× 10` extrapolated to 1 600
        // queries while claiming 2 000, making the assertion ~20 % weaker
        // than it read. Correction 2: this differential is now the *second*,
        // looser net — the binding per-query budget is asserted in
        // `style_resolve_10k_parts`, which does no differencing — and it is
        // kept because it is the only measurement that includes real painting
        // alongside resolution, and the number Slice 5 will compare against
        // `frame_showcase_lists_120x40`.
        const QUERIES_A: u128 = 200; // 40 rows × 5 parts
        const QUERIES_B: u128 = 40; // 8 states × 5 parts, hoisted
        const DELTA: u128 = QUERIES_A - QUERIES_B; // 160
        let per_frame_2k = resolution_ns.saturating_mul(2_000) / DELTA;
        assert!(
            per_frame_2k <= 32_000,
            "style resolution extrapolates to {per_frame_2k} ns for a 2 000-query frame, \
             over the 32 µs (0.2 % of 16 ms) budget"
        );
        // The ≤ 5 % *share* of §16.6 is written against
        // `frame_showcase_lists_120x40`, which needs Slice 5. This stand-in is
        // the style-densest possible frame — 5 resolutions per painted row,
        // no panel chrome, no borders, no status bar — so its share is an
        // upper bound on the real one and is reported, not asserted.
        println!(
            "PERF-NOTE style_resolve_per_frame: the <= 5 % share binds frame_showcase_lists_120x40 (Slice 5); this stand-in reports {share:.3}"
        );
    }
}

static OV_A: [OverlayRule; 1] = [(
    Family::LIST,
    Variant::DEFAULT,
    Part::CONTAINER,
    StateFlags::empty(),
    StylePatch::new().set_fg(Role::Warning),
)];
static OV_B: [OverlayRule; 1] = [(
    Family::LIST,
    Variant::DEFAULT,
    Part::LABEL,
    StateFlags::FOCUSED,
    StylePatch::new().add(tui_next::Modifier::ITALIC),
)];

#[test]
fn style_resolve_10k_parts_with_two_overlays() {
    let _g = lock();
    let mut scene = Scene::new(
        "style_resolve_ov",
        Theme::junie(),
        ColorLevel::TrueColor,
        120,
        40,
    );
    scene.draw(|_, _| {});
    let base = bench(1, iters(10), &mut || {
        scene.draw(|ui, _| {
            black_box(resolve_10k(ui));
        });
    });
    let a = Overlay::new(&OV_A);
    let b = Overlay::new(&OV_B);
    let s = bench(1, iters(10), &mut || {
        scene.draw(|ui, _| {
            ui.with_overlay(&a, |ui| {
                ui.with_overlay(&b, |ui| {
                    black_box(resolve_10k(ui));
                });
            });
        });
    });
    report("style_resolve_10k_parts_with_two_overlays", &s);
    assert_eq!(s.allocs, 0, "overlay resolution must not allocate (R3)");
    check_ratio(
        "style_two_overlays_vs_none",
        s.ns,
        base.ns,
        2.0,
        env_flag("PERF_STRICT"),
    );
}

#[test]
fn style_backdrop_full_screen_120x40() {
    let _g = lock();
    let mut scene = Scene::new("backdrop", Theme::junie(), ColorLevel::TrueColor, 120, 40);
    let dim = Rect::new(0, 0, 120, 39);
    let s = bench(1, iters(100), &mut || {
        scene.draw(|ui, area| {
            let st = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::empty(),
                )
                .style;
            ui.fill(area, st);
            ui.dim_layer(dim, 2);
        });
    });
    report("style_backdrop_full_screen_120x40", &s);
    assert_eq!(s.allocs, 0, "backdrop must not allocate");
}

#[test]
fn style_downgrade_theme_all_levels() {
    let _g = lock();
    const LEVELS: [ColorLevel; 4] = [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ];
    let t = Theme::junie();
    let s = bench(1, iters(20), &mut || {
        for l in LEVELS {
            black_box(t.downgrade(l));
        }
    });
    report("style_downgrade_theme_all_levels", &s);
}

// ------------------------------------------------------------ E. text

#[test]
fn width_10k_grapheme_line() {
    let _g = lock();
    let line = unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(tui_next::width(&line));
    });
    report("width_10k_grapheme_line", &s);
    assert_eq!(s.allocs, 0);
}

#[test]
fn truncate_10k_grapheme_line_to_80() {
    let _g = lock();
    let line = unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(tui_next::truncate(&line, 80));
    });
    report("truncate_10k_grapheme_line_to_80", &s);
}

/// The `RowUi` equivalent of the legacy `fit`: paint a 10 k-grapheme line
/// into 80 columns through `RowUi::label` — the ellipsis path this benchmark
/// is named for — over a corpus whose symbols fit ratatui `Cell`'s inline
/// storage. R5: the painter allocates **nothing**.
#[test]
fn fit_10k_grapheme_line_to_80() {
    let _g = lock();
    let line = unicode_line_inline(10_000);
    let mut scene = Scene::new("fit", Theme::junie(), ColorLevel::TrueColor, 120, 3);
    scene.draw(|_, _| {});
    let s = bench(10, iters(1000), &mut || {
        scene.draw(|ui, _| {
            let mut r = tui_next::RowUi::new(
                ui,
                Id::root("perf.fit"),
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                tui_next::ItemKey::index(0),
                Rect::new(0, 0, 80, 1),
            );
            r.label(&line);
        });
    });
    report("fit_10k_grapheme_line_to_80", &s);
    assert_eq!(
        s.allocs, 0,
        "R5: the row painter must allocate nothing; it allocated {}",
        s.allocs
    );
}

/// The ZWJ-emoji corpus, **reported**. Allocations here are ratatui `Cell`
/// heap symbols — a property of the buffer, not of the painter — so the
/// binding assertion is that they are bounded by the **columns painted** and
/// independent of the line length (adjudication 4).
#[test]
fn fit_10k_grapheme_line_to_80_wide() {
    let _g = lock();
    let mut scene = Scene::new("fit_wide", Theme::junie(), ColorLevel::TrueColor, 120, 3);
    scene.draw(|_, _| {});
    let paint = |scene: &mut Scene, line: &str| {
        scene.draw(|ui, _| {
            let mut r = tui_next::RowUi::new(
                ui,
                Id::root("perf.fit.wide"),
                Family::LIST,
                Variant::DEFAULT,
                StateFlags::empty(),
                tui_next::ItemKey::index(0),
                Rect::new(0, 0, 80, 1),
            );
            r.label(line);
        });
    };
    let short = unicode_line(10_000);
    let long = unicode_line(100_000);
    let a = bench(10, iters(200), &mut || paint(&mut scene, &short));
    let b = bench(10, iters(200), &mut || paint(&mut scene, &long));
    report("fit_10k_grapheme_line_to_80_wide", &a);
    println!(
        "PERF fit_100k_grapheme_line_to_80_wide ns={} allocs={} bytes={}",
        b.ns, b.allocs, b.bytes
    );
    assert_eq!(
        a.allocs, b.allocs,
        "cell-symbol allocations must be independent of the line length"
    );
    assert!(
        a.allocs <= 80,
        "cell-symbol allocations must be bounded by the 80 columns painted, got {}",
        a.allocs
    );
}

/// BL-4: `Ui::paint_spans` used to collect a `Vec<RawSpan>` per call, on the
/// row path — one allocation per span-rendered row per frame. It now walks the
/// spans through `Buffer::set_span`, so painting 500 rows × 3 spans records
/// **0** allocations.
///
/// The differential half of `ui::paint_spans_matches_row_ui_label_spans` lives
/// in `tests/render.rs`; the allocation half is here because §16.6 declares
/// `#[global_allocator]` only in this binary.
#[test]
fn paint_spans_500_rows_is_allocation_free() {
    let _g = lock();
    let spans = [
        tui_next::Span::new("plain "),
        tui_next::Span::new("accent").role(Role::Accent),
        tui_next::Span::new(" tail"),
    ];
    let mut scene = Scene::new("spans", Theme::junie(), ColorLevel::TrueColor, 60, 40);
    scene.draw(|_, _| {});
    let s = bench(2, iters(50), &mut || {
        scene.draw(|ui, area| {
            let base = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::LABEL,
                    StateFlags::empty(),
                )
                .style;
            for i in 0..500u16 {
                let row = Rect::new(area.x, i % area.height, area.width, 1);
                black_box(ui.paint_spans(row, &spans, base));
            }
        });
    });
    report("paint_spans_500_rows_is_allocation_free", &s);
    assert_eq!(
        s.allocs, 0,
        "the span painter must not allocate (R5, §20.9-6)"
    );
}

/// Adjudication N2: measurement is `&Ui` and uncached, and must stay
/// allocation-free — it runs once per component per frame.
#[test]
fn measure_is_allocation_free() {
    let _g = lock();
    let mut scene = Scene::new("measure", Theme::junie(), ColorLevel::TrueColor, 120, 40);
    scene.draw(|_, _| {});
    let s = bench(2, iters(100), &mut || {
        scene.draw(|ui, _| {
            for i in 0..100u32 {
                let flags = STATES[(i as usize) % STATES.len()];
                let g = ui.resolve(Family::BUTTON, Variant::DEFAULT, Part::GUTTER, flags);
                let w = match g.glyph {
                    Slot::Set(r) => tui_next::width(ui.glyph_str(r)),
                    Slot::Inherit | Slot::Clear => 0,
                };
                let h = ui
                    .resolve(Family::BUTTON, Variant::DEFAULT, Part::CONTAINER, flags)
                    .size
                    .unwrap_or(1);
                black_box((w, h));
            }
        });
    });
    report("measure_is_allocation_free", &s);
    assert_eq!(s.allocs, 0, "measurement must not allocate (N2)");
}

#[test]
fn truncate_middle_10k_to_40() {
    let _g = lock();
    let line = unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(tui_next::truncate_middle(&line, 40));
    });
    report("truncate_middle_10k_to_40", &s);
    assert!(
        s.allocs <= 1,
        "truncate_middle allocates {} times",
        s.allocs
    );
}

#[test]
fn wrap_10k_graphemes_to_80() {
    let _g = lock();
    let raw = unicode_line(10_000);
    let mut line = String::with_capacity(raw.len() + raw.len() / 8);
    for (i, ch) in raw.chars().enumerate() {
        if i > 0 && i % 9 == 0 {
            line.push(' ');
        }
        line.push(ch);
    }
    let s = bench(3, iters(200), &mut || {
        black_box(tui_next::wrap(&line, 80));
    });
    report("wrap_10k_graphemes_to_80", &s);
}

#[test]
fn fuzzy_10k_grapheme_label() {
    let _g = lock();
    let label = unicode_line(10_000);
    let s = bench(3, iters(100), &mut || {
        black_box(tui_next::fuzzy(&label, "abc"));
    });
    report("fuzzy_10k_grapheme_label", &s);
}

// ------------------------------------------------------------ F. invariants

/// `N` controls that each probe their (empty) intent bucket.
struct Probes(usize);

impl App for Probes {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut n = 0usize;
        for i in 0..self.0 {
            n += cx.intents(Id::root("probe").index(i)).count();
        }
        black_box(n);
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        for i in 0..self.0 {
            let y = (i % 40) as u16;
            let x = ((i / 40) % 120) as u16;
            ui.register_control(
                Id::root("probe").index(i),
                Rect::new(x, y, 1, 1),
                Focusability::Focusable,
            );
        }
    }
}

fn probe_runtime(n: usize) -> (Runtime<Probes>, ratatui_core::buffer::Buffer) {
    let area = Rect::new(0, 0, 120, 40);
    let mut rt = Runtime::new(Probes(n), Theme::junie());
    let mut buf = ratatui_core::buffer::Buffer::empty(area);
    rt.draw_buffer(area, &mut buf);
    (rt, buf)
}

/// Adjudication 2.6: a ±10 % wall-clock band on a ~600 ns measurement cannot
/// detect a regression in the 500 probes it names (they are ≈0.1 % of it). The
/// binding assertion is a **deterministic probe count**; the ratio is reported
/// always and asserted only under `PERF_STRICT=1`, with a 1.25× band.
#[test]
fn intents_drain_is_o_1_when_the_queue_is_empty() {
    let _g = lock();
    let (mut small, _) = probe_runtime(20);
    let (mut large, _) = probe_runtime(500);

    // settle the initial focus, whose `FocusOut`/`FocusIn` pair is delivered
    // by the first `handle` and does fill the queue
    for _ in 0..2 {
        let _ = small.handle(Input::Tick);
        let _ = large.handle(Input::Tick);
    }
    // an empty queue short-circuits before the bucket table is touched
    let before = large.intent_probes();
    let _ = large.handle(Input::Tick);
    assert_eq!(
        large.intent_probes(),
        before,
        "a frame with an empty queue must perform 0 bucket probes"
    );
    let before20 = small.intent_probes();
    let _ = small.handle(Input::Tick);
    assert_eq!(
        small.intent_probes() - before20,
        large.intent_probes() - before,
        "probe cost is independent of the component count when the queue is empty"
    );

    let s20 = bench(2, iters(200), &mut || {
        let _ = black_box(small.handle(Input::Tick));
    });
    let s500 = bench(2, iters(200), &mut || {
        let _ = black_box(large.handle(Input::Tick));
    });
    report("intents_drain_is_o_1_when_the_queue_is_empty", &s500);
    println!(
        "PERF intents_drain_20_controls ns={} allocs={}",
        s20.ns, s20.allocs
    );
    assert_eq!(s500.allocs, 0);

    // With an intent in the queue, each `cx.intents` call performs exactly one
    // probe: the 500-control frame costs exactly 480 probes more than the
    // 20-control frame, and neither allocates. The difference is the asserted
    // form because `intent_probes()` is cumulative since construction and also
    // counts the enqueue path, so no absolute count is stable.
    //
    // Adjudication O4b correction 2: the constant 480 encodes **one update
    // pass**. `Runtime::handle`'s focus re-run loop is bounded at four passes
    // (§3.3 step 7), so a legitimate second pass makes the delta 960 — which
    // is a real behaviour change and is exactly what this equality is here to
    // catch. Do **not** relax it to `% 480 == 0`; re-adjudicate the pass count
    // instead.
    let key = || {
        Input::Key(tui_next::Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        })
    };
    let probes_for = |n: usize| {
        let (mut rt, _) = probe_runtime(n);
        for _ in 0..2 {
            let _ = rt.handle(Input::Tick);
        }
        let before = rt.intent_probes();
        let _ = rt.handle(key());
        rt.intent_probes() - before
    };
    let (p20, p500) = (probes_for(20), probes_for(500));
    println!("PERF-PROBES intents_drain probes_20={p20} probes_500={p500}");
    assert_eq!(
        p500 - p20,
        480,
        "one probe per drain call in one update pass: {p500} - {p20} != 480 \
         (960 would mean a second focus pass, not a drain regression)"
    );
    let (mut two, _) = probe_runtime(500);
    let s2 = bench(2, iters(200), &mut || {
        let _ = black_box(two.handle(Input::Key(tui_next::Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        })));
    });
    println!(
        "PERF intents_drain_500_controls_1_intent ns={} allocs={}",
        s2.ns, s2.allocs
    );
    assert_eq!(s2.allocs, 0, "probing must not allocate");
    // The raw 500-vs-20 wall-clock ratio measures the *stub application's*
    // own `for i in 0..n` update loop, which is O(n) by construction, so it is
    // reported and never asserted. What §16.6 means by "costs the same" is the
    // per-drain cost, which is O(1): that is the asserted ratio.
    check_ratio("intents_drain_500_vs_20", s500.ns, s20.ns, 1.25, false);
    check_ratio(
        "intents_drain_ns_per_control",
        s500.ns.saturating_mul(20),
        s20.ns.saturating_mul(500),
        1.25,
        env_flag("PERF_STRICT"),
    );
}

#[test]
fn render_twice_allocates_the_same() {
    let _g = lock();
    let mut scene = Scene::new("twice", Theme::junie(), ColorLevel::TrueColor, 80, 40);
    let draw = |ui: &mut Ui<'_>, area: Rect| {
        for (i, row) in area.rows().enumerate() {
            let st = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    STATES[i % STATES.len()],
                )
                .style;
            ui.fill(row, st);
            ui.paint_str(row, "row label with some text", st);
            ui.register_control(Id::root("twice").index(i), row, Focusability::Focusable);
        }
    };
    // two warm-up frames: the runtime double-buffers the registry and ring
    scene.draw(draw);
    scene.draw(draw);
    let a = measure_once(&mut || scene.draw(draw));
    let b = measure_once(&mut || scene.draw(draw));
    println!(
        "PERF render_twice_allocates_the_same ns={} allocs={} bytes={}",
        b.ns, b.allocs, b.bytes
    );
    assert_eq!(
        a.allocs, b.allocs,
        "identical frames must allocate identically"
    );
    assert_eq!(a.bytes, b.bytes);
}

#[test]
fn hit_registry_size_is_bounded() {
    let _g = lock();
    let mut scene = Scene::new("hits", Theme::junie(), ColorLevel::TrueColor, 120, 40);
    let s = bench(1, iters(50), &mut || {
        scene.draw(|ui, area| {
            for (i, row) in area.rows().enumerate() {
                ui.register_control(Id::root("hits").index(i), row, Focusability::Focusable);
                ui.register_scroll(
                    Id::root("hits.scroll").index(i),
                    row,
                    Axes::V,
                    Headroom::default(),
                );
            }
        });
    });
    let (hits, ring) = scene
        .runtime()
        .map(|rt: &Runtime<NoApp>| (rt.region_count(), rt.ring().reachable().count()))
        .unwrap_or((0, 0));
    let s = s.with_regions(hits, ring);
    report("hit_registry_size_is_bounded", &s);
    assert_eq!(hits, 80);
}

// ---------------------------------------------------- G. components (Slice 2)
//
// Appended by the Slice 2 prototype package. Every threshold is §16.6's:
// `list_100k_rows_render` < 500 allocs/frame and ns <= 1.5x the 1 k control,
// `list_100k_select_all` < 100 allocs, `event_dispatch_is_not_o_n` 0 allocs
// and ns within 3x of the 100-row case, `frame_showcase_buttons_120x40` the
// migrated showcase page's frame.

use tui_next::{Button, List, ListState, SelectMode, Status};
use tui_next_testing::perf::{Stats, big};

const LIST_ID: Id = Id::root("perf.list");

/// Rows are `u32`s formatted in place, so the fixture itself contributes no
/// per-row allocation and the numbers measure the component.
fn perf_rows(n: usize) -> Vec<u32> {
    (0..n).map(|i| i as u32).collect()
}

type PerfKeyFn = fn(&u32) -> tui_next::ItemKey;
type PerfRowFn = fn(&u32, &mut tui_next::RowUi<'_>);

fn perf_list<'a>() -> List<'a, u32, PerfKeyFn, PerfRowFn> {
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "the `&T` shape is `KeyFn<T>`'s, not a choice this fixture can make"
    )]
    fn key(r: &u32) -> tui_next::ItemKey {
        tui_next::ItemKey::num(u64::from(*r))
    }
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "the `&T` shape is `RowFn<T>`'s, not a choice this fixture can make"
    )]
    fn row(r: &u32, u: &mut tui_next::RowUi<'_>) {
        u.gutter();
        u.label_fmt(format_args!("row {r}"));
        u.part(Part::META, 8).num(i64::from(*r));
    }
    let k: PerfKeyFn = key;
    let p: PerfRowFn = row;
    List::new(LIST_ID).key(k).row(p)
}

/// One `List::draw` of `rows.len()` rows into a 120x40 scene, with `checked`
/// keys already selected (the `KeySet` lookup per visible row is what §16.6's
/// 1.5x bound is really about). Returns the per-frame stats and the number of
/// regions the frame registered — the deterministic half of "render cost is a
/// function of the viewport, not of the collection".
fn bench_list_render(rows: &[u32], checked: usize) -> (Stats, usize) {
    let mut scene = Scene::new("list", Theme::junie(), ColorLevel::TrueColor, 120, 40);
    let mut st = ListState::default();
    for r in rows.iter().take(checked) {
        st.checked_mut()
            .insert(tui_next::ItemKey::num(u64::from(*r)));
    }
    scene.draw(|ui, area| {
        perf_list().draw(ui, area, &st, rows);
    });
    let s = bench(2, iters(50), &mut || {
        scene.draw(|ui, area| {
            black_box(perf_list().draw(ui, area, &st, rows));
        });
    });
    let regions = scene.registry().map_or(0, Registry::len);
    (s, regions)
}

#[test]
fn list_1k_rows_render() {
    let _g = lock();
    let rows = perf_rows(big(1_000));
    let (s, regions) = bench_list_render(&rows, 50);
    report("list_1k_rows_render", &s);
    assert!(
        s.allocs < 500,
        "the 1 k control must already be flat: {} allocs",
        s.allocs
    );
    assert!(regions > 0, "the control frame registered nothing");
}

#[test]
fn list_100k_rows_render() {
    let _g = lock();
    let small = perf_rows(big(1_000));
    let large = perf_rows(big(100_000));
    let (control, control_regions) = bench_list_render(&small, 50);
    let (s, regions) = bench_list_render(&large, 5_000);
    report("list_100k_rows_render", &s);
    assert!(
        s.allocs < 500,
        "R1: a 100 k list frame must stay under 500 allocations, got {}",
        s.allocs
    );
    // The binding half of R1 is deterministic, not statistical: only the
    // viewport is painted, so a hundred-times-larger collection registers the
    // same regions and paints the same rows. A wall-clock ratio taken from two
    // adjacent benches on a loaded machine drifts by several x; this does not.
    assert!(control_regions > 0);
    assert_eq!(
        regions, control_regions,
        "a 100 k list must register exactly what the 1 k list does"
    );
    // …and the ns ratio is reported, asserted only under `PERF_STRICT=1` on a
    // pinned runner, exactly like every other ns bound in this file.
    check_ratio(
        "list_100k_vs_1k_render",
        s.ns,
        control.ns,
        1.5,
        env_flag("PERF_STRICT"),
    );
}

/// An application that owns the list and forwards one update per frame.
struct ListApp {
    rows: Vec<u32>,
    st: ListState,
    mode: SelectMode,
}

impl App for ListApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        perf_list()
            .select_mode(self.mode)
            .update(cx, &mut self.st, &self.rows)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        perf_list()
            .select_mode(self.mode)
            .draw(ui, ui.full(), &self.st, &self.rows);
    }
}

fn list_runtime(n: usize, mode: SelectMode) -> (Runtime<ListApp>, ratatui_core::buffer::Buffer) {
    let area = Rect::new(0, 0, 120, 40);
    let mut rt = Runtime::new(
        ListApp {
            rows: perf_rows(n),
            st: ListState::default(),
            mode,
        },
        Theme::junie(),
    );
    let mut buf = ratatui_core::buffer::Buffer::empty(area);
    rt.draw_buffer(area, &mut buf);
    for _ in 0..2 {
        let _ = rt.handle(Input::Tick);
        rt.draw_buffer(area, &mut buf);
    }
    (rt, buf)
}

/// R7: `ToggledAll` is a set-level operation. Materialising 100 000
/// `ItemKey`s to express "everything is checked" is the defect this pins.
#[test]
fn list_100k_select_all() {
    let _g = lock();
    let (mut rt, mut buf) = list_runtime(big(100_000), SelectMode::Multi);
    let area = Rect::new(0, 0, 120, 40);
    let toggle = || {
        Input::Key(tui_next::Key {
            code: KeyCode::Char('a'),
            mods: KeyModifiers::NONE,
        })
    };
    // warm: the first toggle also settles focus
    let _ = rt.handle(toggle());
    rt.draw_buffer(area, &mut buf);
    let s = measure_once(&mut || {
        let _ = black_box(rt.handle(toggle()));
    });
    println!(
        "PERF list_100k_select_all ns={} allocs={} bytes={}",
        s.ns, s.allocs, s.bytes
    );
    report("list_100k_select_all", &s);
    assert!(
        s.allocs < 100,
        "R7: `ToggledAll` must not materialise 100 000 keys, got {} allocs",
        s.allocs
    );
    // `a` toggles: the measured press cleared the set the warm press filled,
    // so one more press proves the set-level "everything" is reachable at all
    let _ = rt.handle(toggle());
    assert!(rt.app().st.checked().contains(tui_next::ItemKey::num(99)));
    assert!(
        rt.app()
            .st
            .checked()
            .contains(tui_next::ItemKey::num(99_999))
    );
}

/// Dispatch cost is a function of the **registered** regions, which is a
/// function of the viewport — never of the collection behind it.
#[test]
fn event_dispatch_is_not_o_n() {
    let _g = lock();
    let area = Rect::new(0, 0, 120, 40);
    let (mut small, mut small_buf) = list_runtime(100, SelectMode::Single);
    let (mut large, mut large_buf) = list_runtime(big(100_000), SelectMode::Single);
    let click = |x: u16, y: u16| {
        [
            Input::Mouse(tui_next::Mouse {
                kind: tui_next::MouseKind::Down,
                pos: Position::new(x, y),
                mods: KeyModifiers::NONE,
            }),
            Input::Mouse(tui_next::Mouse {
                kind: tui_next::MouseKind::Up,
                pos: Position::new(x, y),
                mods: KeyModifiers::NONE,
            }),
        ]
    };
    let run = |rt: &mut Runtime<ListApp>, buf: &mut ratatui_core::buffer::Buffer| {
        for i in rt.app().rows.iter().take(1) {
            black_box(i);
        }
        for input in click(10, 5) {
            let _ = black_box(rt.handle(input));
        }
        rt.draw_buffer(area, buf);
    };
    let s_small = bench(2, iters(50), &mut || run(&mut small, &mut small_buf));
    let s_large = bench(2, iters(50), &mut || run(&mut large, &mut large_buf));
    let s = Stats {
        allocs: s_large.allocs,
        ..s_large
    };
    report("event_dispatch_is_not_o_n", &s);
    println!(
        "PERF event_dispatch_100_rows ns={} allocs={}",
        s_small.ns, s_small.allocs
    );
    assert_eq!(s_large.allocs, 0, "dispatch must not allocate");
    assert_eq!(s_small.allocs, 0, "dispatch must not allocate");
    check_ratio(
        "event_dispatch_100k_vs_100",
        s_large.ns,
        s_small.ns,
        3.0,
        env_flag("PERF_STRICT"),
    );
}

// ------------------------------------------------ the migrated showcase page

const SHOW: Id = Id::root("perf.showcase.buttons");

/// `(label, variant, disabled, checked)` — the nine buttons of
/// `src/bin/showcase/pages/buttons.rs`, migrated.
const SHOWCASE_BUTTONS: [(&str, Variant, bool, Option<bool>); 9] = [
    ("Run task", Variant::PRIMARY, false, None),
    ("Preview", Variant::SECONDARY, false, None),
    ("Cancel", Variant::SUBTLE, false, None),
    ("Delete branch", Variant::DANGER, false, None),
    ("Auto-approve", Variant::TOGGLE, false, Some(false)),
    ("Verbose", Variant::TOGGLE, false, Some(true)),
    ("Disabled primary", Variant::PRIMARY, true, None),
    ("Disabled", Variant::SECONDARY, true, None),
    ("Start long job", Variant::SECONDARY, false, None),
];

/// The Buttons page as a frame profile: nine buttons, four group captions and
/// a twenty-four-cell reference matrix.
struct ShowcaseButtons;

impl ShowcaseButtons {
    fn button(i: usize) -> Button<'static> {
        let (label, variant, disabled, checked) = SHOWCASE_BUTTONS[i];
        let mut b = Button::new(SHOW.index(i), label)
            .variant(variant)
            .disabled(disabled)
            .status(Status::Ready);
        if let Some(on) = checked {
            b = b.checked(on);
        }
        b
    }
}

impl App for ShowcaseButtons {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();
        for i in 0..SHOWCASE_BUTTONS.len() {
            r |= ShowcaseButtons::button(i).update(cx).erase();
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let mut x = area.x;
        let mut y = area.y;
        for i in 0..SHOWCASE_BUTTONS.len() {
            let w = 20u16.min(area.width);
            if x + w > area.right() {
                x = area.x;
                y += 2;
            }
            ShowcaseButtons::button(i).draw(ui, Rect::new(x, y, w, 1));
            x += w + 2;
        }
    }
}

#[test]
fn frame_showcase_buttons_120x40() {
    let _g = lock();
    let area = Rect::new(0, 0, 120, 40);
    let mut rt = Runtime::new(ShowcaseButtons, Theme::junie());
    let mut buf = ratatui_core::buffer::Buffer::empty(area);
    // two warm frames: the runtime double-buffers the registry and the ring
    rt.draw_buffer(area, &mut buf);
    rt.draw_buffer(area, &mut buf);
    let s = bench(2, iters(200), &mut || {
        rt.draw_buffer(area, &mut buf);
    });
    let s = s.with_regions(rt.region_count(), rt.ring().reachable().count());
    report("frame_showcase_buttons_120x40", &s);
    assert_eq!(
        rt.ring().reachable().count(),
        7,
        "nine buttons, two of them disabled"
    );
    assert_eq!(s.allocs, 0, "a button frame must not allocate");
}

const QUERY_EDITOR: Id = Id::root("perf.query_editor");
static HIGHLIGHT_CALLS: AtomicUsize = AtomicUsize::new(0);

struct DenseHighlighter;

impl Highlighter for DenseHighlighter {
    fn highlight(&self, text: &str) -> Vec<(core::ops::Range<usize>, SyntaxRole)> {
        HIGHLIGHT_CALLS.fetch_add(1, Ordering::Relaxed);
        text.match_indices(|character: char| character.is_ascii_alphanumeric())
            .map(|(start, value)| {
                (
                    start..start.saturating_add(value.len()),
                    SyntaxRole::Keyword,
                )
            })
            .collect()
    }
}

static DENSE_HIGHLIGHTER: DenseHighlighter = DenseHighlighter;

struct QueryEditor {
    state: CodeEditorState,
}

impl QueryEditor {
    fn with_lines(count: usize) -> Self {
        let mut source = String::with_capacity(count.saturating_mul(32));
        let mut diagnostics = Vec::with_capacity(count);
        for index in 0..count {
            use core::fmt::Write as _;
            let start = source.len();
            let _ = writeln!(source, "select column_{index} from table_{index};");
            diagnostics.push(CodeDiagnostic::new(
                start..start.saturating_add(6),
                if index % 3 == 0 {
                    CodeSeverity::Error
                } else {
                    CodeSeverity::Warning
                },
                "dense diagnostic",
            ));
        }
        let mut state = CodeEditorState::new(&source);
        state.set_diagnostics(diagnostics);
        QueryEditor { state }
    }
}

impl App for QueryEditor {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        CodeEditor::new(QUERY_EDITOR, 39)
            .highlighter(&DENSE_HIGHLIGHTER)
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        CodeEditor::new(QUERY_EDITOR, 39)
            .highlighter(&DENSE_HIGHLIGHTER)
            .draw(ui, ui.full(), &self.state);
    }
}

fn prepare_query_editor(
    runtime: &mut Runtime<QueryEditor>,
    area: Rect,
    buffer: &mut ratatui_core::buffer::Buffer,
) {
    runtime.draw_buffer(area, buffer);
    runtime.draw_buffer(area, buffer);
    let input = |code| {
        Input::Key(tui_next::Key {
            code,
            mods: KeyModifiers::NONE,
        })
    };
    let _ = runtime.handle(input(KeyCode::Tab));
    let _ = runtime.handle(input(KeyCode::Char('/')));
    runtime.draw_buffer(area, buffer);
    for character in "column".chars() {
        let _ = runtime.handle(input(KeyCode::Char(character)));
    }
    let _ = runtime.handle(input(KeyCode::Enter));
    runtime.draw_buffer(area, buffer);
}

#[test]
fn frame_tablepro_query_editor_2k_lines() {
    let _g = lock();
    let area = Rect::new(0, 0, 120, 40);
    let mut runtime = Runtime::new(QueryEditor::with_lines(2_000), Theme::junie());
    let mut small_runtime = Runtime::new(QueryEditor::with_lines(100), Theme::junie());
    let mut buffer = ratatui_core::buffer::Buffer::empty(area);
    let mut small_buffer = ratatui_core::buffer::Buffer::empty(area);
    prepare_query_editor(&mut runtime, area, &mut buffer);
    prepare_query_editor(&mut small_runtime, area, &mut small_buffer);
    let warm_highlights = HIGHLIGHT_CALLS.load(Ordering::Relaxed);
    let sample = bench(2, iters(100), &mut || {
        runtime.draw_buffer(area, &mut buffer);
    });
    let small_sample = bench(2, iters(100), &mut || {
        small_runtime.draw_buffer(area, &mut small_buffer);
    });
    report("frame_tablepro_query_editor_2k_lines", &sample);
    assert!(
        sample.allocs < 40,
        "query editor frame allocated {} times; budget is below 40",
        sample.allocs
    );
    assert!(small_sample.allocs < 40);
    assert_eq!(
        HIGHLIGHT_CALLS.load(Ordering::Relaxed),
        warm_highlights,
        "warm frames must reuse dense highlighting"
    );
    check_ratio(
        "query_editor_2k_vs_100_lines",
        sample.ns,
        small_sample.ns,
        2.0,
        env_flag("PERF_STRICT"),
    );
}

const PERF_DIFF: Id = Id::root("perf.diff");

struct DenseDiffSource;

impl DiffSource for DenseDiffSource {
    fn revision(&self) -> u64 {
        1
    }

    fn path(&self) -> &'static str {
        "src/dense.rs"
    }

    fn status_marker(&self) -> &'static str {
        "M"
    }

    fn status_label(&self) -> &'static str {
        "modified"
    }

    fn row_count(&self) -> usize {
        2_001
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        match index {
            0 => Some(DiffRow::Hunk {
                old_start: 1,
                new_start: 1,
            }),
            1..=2_000 => Some(DiffRow::Line {
                kind: if index.is_multiple_of(3) {
                    DiffLineKind::Add
                } else {
                    DiffLineKind::Context
                },
                text: "let projected = cached.diff_line();",
            }),
            _ => None,
        }
    }
}

struct DenseDiff {
    state: DiffViewState,
}

impl App for DenseDiff {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        DiffView::new(PERF_DIFF, Some(&DenseDiffSource))
            .update(cx, &mut self.state)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        DiffView::new(PERF_DIFF, Some(&DenseDiffSource)).draw(ui, ui.full(), &self.state);
    }
}

#[test]
fn diff_2k_cached_projection_has_zero_warm_allocations() {
    let _guard = lock();
    let area = Rect::new(0, 0, 120, 40);
    let mut runtime = Runtime::new(
        DenseDiff {
            state: DiffViewState::default(),
        },
        Theme::junie(),
    );
    let mut buffer = ratatui_core::buffer::Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer);
    runtime.draw_buffer(area, &mut buffer);
    let sample = bench(2, iters(100), &mut || {
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(area, &mut buffer);
    });
    report("diff_2k_cached_projection", &sample);
    assert_eq!(sample.allocs, 0, "warm update and draw must not allocate");
}

const HINT_CONTROL: Id = Id::root("perf.hint-control");
const HINT_ACTION: ActionKey = ActionKey::custom("perf.hint.activate");

#[derive(Clone, Copy)]
enum HintCmd {
    Activate,
}

const HINT_BINDINGS: &[Binding<HintCmd>] = &[Binding {
    action: HINT_ACTION,
    chord: Some(Chord::key(KeyCode::Enter)),
    cmd: HintCmd::Activate,
    label: "Activate",
    priority: 80,
    visible: true,
}];

#[derive(Default)]
struct DerivedHintApp {
    handled: u64,
}

impl App for DerivedHintApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for intent in cx.intents(HINT_CONTROL) {
            if let Intent::Binding(action) = intent
                && Binding::command(HINT_BINDINGS, action).is_some()
            {
                self.handled = self.handled.saturating_add(1);
                response |= Response::consumed();
            }
        }
        response
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(
            HINT_CONTROL,
            Rect::new(0, 0, 20, 1),
            Focusability::Focusable,
        );
        ui.publish_bindings(HINT_CONTROL, ui.state(HINT_CONTROL), HINT_BINDINGS);
        HintBar::derived(Id::root("perf.hintbar")).draw(ui, Rect::new(0, 1, 80, 1));
    }
}

#[test]
fn frame_hintbar_derived() {
    let _g = lock();
    let area = Rect::new(0, 0, 80, 24);
    let mut runtime = Runtime::new(DerivedHintApp::default(), Theme::junie());
    let mut buffer = ratatui_core::buffer::Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer);
    runtime.draw_buffer(area, &mut buffer);
    let sample = bench(2, iters(200), &mut || {
        runtime.draw_buffer(area, &mut buffer);
    });
    report("frame_hintbar_derived", &sample);
    assert_eq!(sample.allocs, 0, "unchanged derived-hint frame allocated");

    let key = || {
        Input::Key(tui_next::Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        })
    };
    let _ = runtime.handle(key());
    let routing = bench(2, iters(200), &mut || {
        let _ = black_box(runtime.handle(key()));
    });
    assert_eq!(
        routing.allocs, 0,
        "unchanged component key routing allocated"
    );
}

const PERF_FORM: Id = Id::root("perf.form");
const PERF_FORM_FLAG: Id = Id::root("perf.form.flag");
const PERF_FORM_FIELDS: &[FieldSpec<'static>] = &[FieldSpec::new(
    PERF_FORM_FLAG,
    "Enabled",
    FieldKind::Check(Checkbox::new(PERF_FORM_FLAG, "Enabled")),
)];

#[derive(Default)]
struct PerfFormData {
    enabled: bool,
}

impl FormData for PerfFormData {
    fn value(&self, id: Id) -> FieldRef<'_> {
        if id == PERF_FORM_FLAG {
            FieldRef::Flag(self.enabled)
        } else {
            FieldRef::Flag(false)
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        if id == PERF_FORM_FLAG {
            FieldMut::Flag(&mut self.enabled)
        } else {
            FieldMut::ReadOnly
        }
    }
}

#[derive(Default)]
struct PerfFormApp {
    state: FormState,
    data: PerfFormData,
}

impl App for PerfFormApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Form::new(PERF_FORM, PERF_FORM_FIELDS)
            .update(cx, &mut self.state, &mut self.data)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Form::new(PERF_FORM, PERF_FORM_FIELDS).draw(ui, ui.full(), &self.state, &self.data);
    }
}

#[test]
fn frame_form_update_draw() {
    let _guard = lock();
    let area = Rect::new(0, 0, 80, 24);
    let mut runtime = Runtime::new(PerfFormApp::default(), Theme::junie());
    let mut buffer = ratatui_core::buffer::Buffer::empty(area);
    runtime.draw_buffer(area, &mut buffer);
    let _ = runtime.handle(Input::Tick);
    runtime.draw_buffer(area, &mut buffer);
    runtime.draw_buffer(area, &mut buffer);
    let sample = bench(2, iters(200), &mut || {
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(area, &mut buffer);
    });
    report("frame_form_update_draw", &sample);
    assert_eq!(sample.allocs, 0, "warm Form update and draw allocated");
    assert_eq!(sample.bytes, 0, "warm Form update and draw allocated bytes");
}
