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
        env_flag("PERF_TARGET"),
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
        black_box(tui_next::text::width(&line));
    });
    report("width_10k_grapheme_line", &s);
    assert_eq!(s.allocs, 0);
}

#[test]
fn truncate_10k_grapheme_line_to_80() {
    let _g = lock();
    let line = unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(tui_next::text::truncate(&line, 80));
    });
    report("truncate_10k_grapheme_line_to_80", &s);
}

/// The `RowUi` equivalent of the legacy `fit`: paint a 10k-grapheme line
/// into 80 columns through the clipping writer (R5: 0 allocations).
#[test]
fn fit_10k_grapheme_line_to_80() {
    let _g = lock();
    let line = unicode_line(10_000);
    let mut scene = Scene::new("fit", Theme::junie(), ColorLevel::TrueColor, 120, 3);
    scene.draw(|_, _| {});
    let s = bench(10, iters(1000), &mut || {
        scene.draw(|ui, _| {
            let st = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::LABEL,
                    StateFlags::empty(),
                )
                .style;
            black_box(ui.paint_str(Rect::new(0, 0, 80, 1), &line, st));
        });
    });
    report("fit_10k_grapheme_line_to_80", &s);
    // The painter allocates nothing; ratatui's `Cell` stores a symbol longer
    // than 24 bytes (the ZWJ family emoji in this fixture) on the heap, so the
    // count is bounded by the wide emoji painted into 80 columns, never by the
    // line length (the legacy `fit` cost 3 owned strings on top of that).
    assert!(
        s.allocs <= 8,
        "R5: the row painter allocated {} times",
        s.allocs
    );
}

#[test]
fn truncate_middle_10k_to_40() {
    let _g = lock();
    let line = unicode_line(10_000);
    let s = bench(10, iters(1000), &mut || {
        black_box(tui_next::text::truncate_middle(&line, 40));
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
        black_box(tui_next::text::wrap(&line, 80));
    });
    report("wrap_10k_graphemes_to_80", &s);
}

#[test]
fn fuzzy_10k_grapheme_label() {
    let _g = lock();
    let label = unicode_line(10_000);
    let s = bench(3, iters(100), &mut || {
        black_box(tui_next::text::fuzzy(&label, "abc"));
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
        black_box(tui_next::text::TextBuffer::pos_of(&doc, off));
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
    let tb = tui_next::text::TextBuffer::multi(doc);
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

#[test]
fn intents_drain_is_o_1_when_the_queue_is_empty() {
    let _g = lock();
    let (mut small, _) = probe_runtime(20);
    let (mut large, _) = probe_runtime(500);
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
    // with two intents (a key to the focused probe and a pointer press), probes stay cheap
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
    check_ratio(
        "intents_drain_500_vs_20",
        s500.ns,
        s20.ns,
        1.1,
        env_flag("PERF_TARGET"),
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
