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

use tui_next::{
    App, Axes, ColorLevel, Cx, Family, FocusRing, Focusability, Headroom, Id, Input, LayerId,
    Overlay, OverlayRule, Part, Position, Rect, Registry, Response, Role, Runtime, StateFlags,
    StylePatch, Theme, Ui, Variant,
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
         share={share:.3} queries=200",
        a.ns, b.ns
    );
    assert_eq!(a.allocs, 0);
    if env_flag("PERF_STRICT") {
        // The adjudication's own arithmetic is the machine-independent bound:
        // ~13 ns per query × ~2 000 queries per realistic frame ≈ 26 µs,
        // "under 0.2 % of a 16 ms budget". Asserted here against a 32 µs
        // ceiling (0.2 % of 16 ms), scaled from this frame's 200 queries.
        let per_frame_2k = resolution_ns.saturating_mul(10);
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
                let w = g.glyph.map_or(0, |r| tui_next::width(ui.glyph_str(r)));
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

#[test]
fn textbuffer_pos_of_10k_line() {
    let _g = lock();
    let mut doc = String::new();
    for i in 0..20 {
        doc.push_str(&format!("line {i}\n"));
    }
    doc.push_str(&unicode_line(10_000));
    let off = doc.len();
    let s = bench(10, iters(1000), &mut || {
        black_box(tui_next::TextBuffer::pos_of(&doc, off));
    });
    report("textbuffer_pos_of_10k_line", &s);
    assert_eq!(s.allocs, 0);
}

#[test]
fn textbuffer_offset_at_10k_line() {
    let _g = lock();
    let mut doc = String::new();
    for i in 0..20 {
        doc.push_str(&format!("line {i}\n"));
    }
    doc.push_str(&unicode_line(10_000));
    let tb = tui_next::TextBuffer::multi(doc);
    let s = bench(10, iters(1000), &mut || {
        black_box(tb.offset_at(20, 12_000));
    });
    report("textbuffer_offset_at_10k_line", &s);
    assert_eq!(s.allocs, 0);
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

    // with an intent in the queue, each `cx.intents` call performs exactly
    // one probe: the 500-control frame costs exactly 480 probes more than the
    // 20-control frame, and neither allocates
    let key = || {
        Input::Key(tui_next::Key {
            code: tui_next::KeyCode::Enter,
            mods: tui_next::KeyModifiers::NONE,
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
        "one probe per drain call: {p500} - {p20} != 480"
    );
    let (mut two, _) = probe_runtime(500);
    let s2 = bench(2, iters(200), &mut || {
        let _ = black_box(two.handle(Input::Key(tui_next::Key {
            code: tui_next::KeyCode::Enter,
            mods: tui_next::KeyModifiers::NONE,
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
