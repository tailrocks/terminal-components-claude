//! TASK-029 witnesses: deterministic progress/spinner motion and semantic Meter paint.
//!
//! Positive cases pin every owned boundary: determinate fill rounding,
//! indeterminate sweep period, paused/done/failed/busy states, ten-frame
//! spinner timing (braille and ASCII), gap geometry, Tick independence,
//! meter zero/full/no-ratio/series modes, line/block readout and on-fill,
//! stale/unknown tones, threshold boundaries at every capability, tiny-width
//! fallbacks, and the ordered `ReferenceLift` policy. Negative cases reject
//! the recorded mutants: unknown-as-zero, off-by-one thresholds,
//! float-clamped animation, RGB role inference, and derived-beats-explicit.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects,
    clippy::too_many_lines,
    reason = "deterministic paint assertions"
)]
use junie_tui::theme::downgrade_color;
use junie_tui::{
    App, Cell, ColorLevel, Constraints, Cx, GlyphRole, Id, Input, Key, KeyCode, KeyModifiers,
    Meter, MeterFillRest, MeterTone, MeterVisual, Part, ProgressBar, Rect, Response, Role, Spinner,
    Status, StylePatch, Surface, Theme, Ui,
};
use junie_tui_testing::{Harness, Scene};
use ratatui_core::style::Color;

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];
const SURFACES: [Surface; 7] = [
    Surface::Canvas,
    Surface::Surface,
    Surface::Elevated,
    Surface::Overlay,
    Surface::Popover,
    Surface::Field,
    Surface::FieldHover,
];
const BAR: Id = Id::root("w029.bar");
const METER: Id = Id::root("w029.meter");
const SPIN: Id = Id::root("w029.spin");

/// Direct transcription of the accepted ordered lift (HIST:HL-METER-POLICY):
/// Canvas first, then Surface/Elevated, then Field, else Popover.
fn pinned_lift(theme: &Theme, surface: Surface) -> Color {
    let bg = theme.bg(surface);
    if bg == theme.bg(Surface::Canvas) {
        theme.bg(Surface::Elevated)
    } else if bg == theme.bg(Surface::Surface) || bg == theme.bg(Surface::Elevated) {
        theme.bg(Surface::Overlay)
    } else if bg == theme.bg(Surface::Field) {
        theme.bg(Surface::FieldHover)
    } else {
        theme.bg(Surface::Popover)
    }
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "builder temporaries read once"
)]
fn draw_bar(theme: Theme, level: ColorLevel, w: u16, bar: ProgressBar<'_>) -> Scene {
    let mut scene = Scene::new("w029-bar", theme, level, w, 1);
    scene.draw(|ui, area| {
        bar.draw(ui, area);
    });
    scene
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "builder temporaries read once"
)]
fn draw_meter(theme: Theme, level: ColorLevel, w: u16, meter: Meter<'_>) -> Scene {
    let mut scene = Scene::new("w029-meter", theme, level, w, 1);
    scene.draw(|ui, area| {
        meter.draw(ui, area);
    });
    scene
}

fn count_symbol(scene: &Scene, sym: &str) -> usize {
    scene
        .buffer()
        .content
        .iter()
        .filter(|c| c.symbol() == sym)
        .count()
}

// ── W-029-01 progress states and determinate rounding ──

#[test]
fn w029_01_determinate_fill_rounds_from_zero_edge() {
    // Width 30: tail is 7 cells, so the track is 23 wide.
    for (ratio, filled, pct) in [
        (0.0, 0, "0%"),
        (0.02, 0, "2%"),
        (0.04, 1, "4%"),
        (0.5, 12, "50%"),
        (0.999, 23, "100%"),
        (1.0, 23, "100%"),
        (1.5, 23, "100%"),
        (-0.5, 0, "0%"),
    ] {
        let scene = draw_bar(
            Theme::junie(),
            ColorLevel::TrueColor,
            30,
            ProgressBar::new(BAR).ratio(ratio),
        );
        let text = scene.text();
        assert_eq!(count_symbol(&scene, "━"), filled, "ratio {ratio}");
        assert_eq!(count_symbol(&scene, "─"), 23 - filled, "ratio {ratio}");
        assert!(text.contains(pct), "ratio {ratio}: {text:?}");
    }
}

#[test]
fn w029_01_indeterminate_sweep_crosses_and_wraps_period_33() {
    // Width 30 indeterminate: tail is 2 cells, track 28, segment 5,
    // so the integer period is 33 frames.
    let frame_text = |frame: usize| {
        draw_bar(
            Theme::junie(),
            ColorLevel::TrueColor,
            30,
            ProgressBar::new(BAR).frame(frame),
        )
        .text()
    };
    let quiet = "─".repeat(28);
    assert_eq!(frame_text(0), format!("{quiet}  "));
    // Segment wholly inside the track mid-sweep.
    assert_eq!(
        frame_text(14),
        format!("{}━━━━━{}  ", "─".repeat(9), "─".repeat(14))
    );
    assert_eq!(frame_text(5), format!("━━━━━{}  ", "─".repeat(23)));
    assert_eq!(frame_text(28), format!("{}━━━━━  ", "─".repeat(23)));
    assert_eq!(frame_text(32), format!("{}━  ", "─".repeat(27)));
    // Integer wrap: the boundary frame matches frame zero exactly.
    assert_eq!(frame_text(33), frame_text(0));
    assert_eq!(frame_text(66), frame_text(0));
    assert_eq!(frame_text(34), format!("━{}  ", "─".repeat(27)));
    assert_ne!(frame_text(32), frame_text(0));
}

#[test]
fn w029_01_paused_done_failed_busy_glyphs_match_recipe() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            let glyphs = &theme.design.glyphs;
            let frames = theme.design.motion.spinner_frames;
            for (name, bar, expected) in [
                (
                    "done",
                    ProgressBar::new(BAR).ratio(1.0).done(true),
                    glyphs.get(GlyphRole::ProgressDone),
                ),
                (
                    "failed",
                    ProgressBar::new(BAR).ratio(0.3).status(Status::Error),
                    glyphs.get(GlyphRole::Error),
                ),
                (
                    "paused",
                    ProgressBar::new(BAR)
                        .ratio(0.3)
                        .icon(GlyphRole::ProgressPaused),
                    glyphs.get(GlyphRole::ProgressPaused),
                ),
                (
                    "busy-det",
                    ProgressBar::new(BAR).ratio(0.3).status(Status::Busy),
                    frames[0],
                ),
                (
                    "busy-indet",
                    ProgressBar::new(BAR).status(Status::Loading),
                    frames[0],
                ),
            ] {
                let scene = draw_bar(theme.clone(), level, 30, bar);
                assert!(
                    scene
                        .buffer()
                        .content
                        .iter()
                        .any(|c| c.symbol() == expected),
                    "{name} {level:?}: expected {expected:?} in {:?}",
                    scene.text()
                );
            }
        }
    }
    // Junie pins the exact marks: done ✓, failed !, paused ‖, busy ⠋.
    let scene = draw_bar(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        ProgressBar::new(BAR).ratio(1.0).done(true),
    );
    assert!(scene.text().contains('✓'), "{:?}", scene.text());
    let scene = draw_bar(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        ProgressBar::new(BAR).ratio(0.3).status(Status::Error),
    );
    assert!(scene.text().contains('!'), "{:?}", scene.text());
}

// ── W-029-01 spinner phase, gap, and determinism ──

#[test]
fn w029_01_ten_frame_timing_braille_ascii_wrap() {
    let braille = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let ascii_theme = Theme::junie().builder().ascii_glyphs().build();
    let ascii = ascii_theme.design.motion.spinner_frames;
    assert_eq!(ascii.len(), 10);
    assert_eq!(ascii, &["|", "/", "-", "\\", "|", "/", "-", "\\", "|", "/"]);
    for (theme, frames) in [(Theme::junie(), braille.as_slice()), (ascii_theme, ascii)] {
        for (frame, want) in frames.iter().enumerate() {
            let mut scene = Scene::new("w029-spin", theme.clone(), ColorLevel::TrueColor, 4, 1);
            scene.draw(|ui, area| {
                Spinner::new(SPIN).frame(frame).draw(ui, area);
            });
            assert_eq!(
                scene.buffer().cell((0, 0)).map(Cell::symbol),
                Some(*want),
                "frame {frame}"
            );
        }
        // Motion wraps on the integer frame count, not a float clamp.
        for frame in [10, 20, 33, usize::MAX] {
            let mut a = Scene::new("w029-a", theme.clone(), ColorLevel::TrueColor, 4, 1);
            let mut b = Scene::new("w029-b", theme.clone(), ColorLevel::TrueColor, 4, 1);
            a.draw(|ui, area| {
                Spinner::new(SPIN).frame(frame).draw(ui, area);
            });
            b.draw(|ui, area| {
                Spinner::new(SPIN).frame(frame % 10).draw(ui, area);
            });
            assert_eq!(a.buffer(), b.buffer(), "frame {frame}");
        }
    }
}

#[test]
fn w029_01_gap_boundary_zero_one_and_frame_anchor() {
    for gap in 0..=4_u16 {
        let mut scene = Scene::new("w029-gap", Theme::junie(), ColorLevel::TrueColor, 20, 1);
        scene.draw(|ui, area| {
            let spinner = Spinner::new(SPIN).label("Wait").gap(gap);
            assert_eq!(
                spinner.measure(ui, Constraints::loose(20, 1)).preferred,
                (5 + gap, 1)
            );
            assert_eq!(spinner.draw(ui, area), Rect::new(0, 0, 5 + gap, 1));
        });
        // The frame cell never moves; only the label column shifts.
        assert_eq!(
            scene.buffer().cell((0, 0)).map(Cell::symbol),
            Some("⠋"),
            "gap {gap}"
        );
        assert_eq!(
            scene.buffer().cell((1 + gap, 0)).map(Cell::symbol),
            Some("W"),
            "gap {gap}"
        );
    }
    // An omitted gap is exactly the theme default (junie gap 2).
    let mut scene = Scene::new("w029-gap-def", Theme::junie(), ColorLevel::TrueColor, 20, 1);
    scene.draw(|ui, area| {
        Spinner::new(SPIN).label("Wait").draw(ui, area);
    });
    assert_eq!(scene.buffer().cell((3, 0)).map(Cell::symbol), Some("W"));
}

#[test]
fn w029_01_repeat_draw_is_byte_identical() {
    let mut bar_det = Scene::new("w029-r", Theme::junie(), ColorLevel::TrueColor, 30, 1);
    bar_det.draw(|ui, area| {
        ProgressBar::new(BAR).ratio(0.4).frame(7).draw(ui, area);
    });
    let first = bar_det.buffer().clone();
    bar_det.draw(|ui, area| {
        ProgressBar::new(BAR).ratio(0.4).frame(7).draw(ui, area);
    });
    assert_eq!(bar_det.buffer(), &first);

    let mut sweep = Scene::new("w029-r", Theme::junie(), ColorLevel::TrueColor, 30, 1);
    sweep.draw(|ui, area| {
        ProgressBar::new(BAR).frame(21).draw(ui, area);
    });
    let first = sweep.buffer().clone();
    sweep.draw(|ui, area| {
        ProgressBar::new(BAR).frame(21).draw(ui, area);
    });
    assert_eq!(sweep.buffer(), &first);

    let mut spin = Scene::new("w029-r", Theme::junie(), ColorLevel::TrueColor, 10, 1);
    spin.draw(|ui, area| {
        Spinner::new(SPIN).label("wait").frame(3).draw(ui, area);
    });
    let first = spin.buffer().clone();
    spin.draw(|ui, area| {
        Spinner::new(SPIN).label("wait").frame(3).draw(ui, area);
    });
    assert_eq!(spin.buffer(), &first);

    for visual in [MeterVisual::Line, MeterVisual::Block] {
        let mut meter = Scene::new("w029-r", Theme::junie(), ColorLevel::TrueColor, 30, 1);
        meter.draw(|ui, area| {
            Meter::new(METER)
                .ratio(0.7)
                .visual(visual)
                .frame(9)
                .draw(ui, area);
        });
        let first = meter.buffer().clone();
        meter.draw(|ui, area| {
            Meter::new(METER)
                .ratio(0.7)
                .visual(visual)
                .frame(9)
                .draw(ui, area);
        });
        assert_eq!(meter.buffer(), &first, "{visual:?}");
    }
}

struct Fixed029 {
    frame: usize,
}

impl App for Fixed029 {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ProgressBar::new(BAR)
            .ratio(0.4)
            .frame(self.frame)
            .draw(ui, Rect::new(0, 0, 30, 1));
        ProgressBar::new(Id::root("w029.sweep"))
            .frame(self.frame)
            .draw(ui, Rect::new(0, 1, 30, 1));
        Spinner::new(SPIN)
            .label("wait")
            .frame(self.frame)
            .draw(ui, Rect::new(0, 2, 10, 1));
        Meter::new(METER)
            .ratio(0.7)
            .frame(self.frame)
            .draw(ui, Rect::new(10, 2, 20, 1));
    }
}

#[test]
fn w029_01_tick_key_resize_never_advance_without_frame() {
    let mut h = Harness::new(Fixed029 { frame: 4 }, Theme::junie(), 30, 3);
    let settled = h.text();
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    let inputs = [
        Input::Tick,
        Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }),
        Input::Paste("private".into()),
        Input::Resize(30, 3),
    ];
    for _ in 0..25 {
        for input in &inputs {
            let _ = h.handle(input.clone());
            assert_eq!(h.text(), settled);
        }
    }
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
    // Only the frame prop moves pixels; the clock alone never does.
    h.app_mut().frame = 5;
    h.draw();
    let moved = h.text();
    assert_ne!(moved, settled);
    for _ in 0..25 {
        let _ = h.handle(Input::Tick);
        assert_eq!(h.text(), moved);
    }
}

// ── W-029-02 meter modes, readout, and on-fill ──

#[test]
fn w029_02_zero_and_full_runs() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            let empty = draw_meter(
                theme.clone(),
                level,
                30,
                Meter::new(METER).ratio(0.0).visual(MeterVisual::Line),
            );
            assert_eq!(count_symbol(&empty, "━"), 0, "{level:?}");
            assert!(count_symbol(&empty, "─") > 0, "{level:?}");
            assert!(empty.text().contains("0%"), "{level:?}: {:?}", empty.text());
            let full = draw_meter(
                theme.clone(),
                level,
                30,
                Meter::new(METER).ratio(1.0).visual(MeterVisual::Line),
            );
            assert_eq!(count_symbol(&full, "─"), 0, "{level:?}");
            assert!(count_symbol(&full, "━") > 0, "{level:?}");
            assert!(full.text().contains("100%"), "{level:?}: {:?}", full.text());
            // Block mode keeps the readout inside the bar at both extremes.
            for ratio in [0.0, 1.0] {
                let block = draw_meter(
                    theme.clone(),
                    level,
                    30,
                    Meter::new(METER).ratio(ratio).visual(MeterVisual::Block),
                );
                let want = if ratio == 0.0 { "0%" } else { "100%" };
                assert!(block.text().contains(want), "{level:?}: {:?}", block.text());
            }
        }
    }
}

#[test]
fn w029_02_no_ratio_paints_value_only_unknown() {
    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            for visual in [MeterVisual::Line, MeterVisual::Block] {
                let scene = draw_meter(
                    theme.clone(),
                    level,
                    30,
                    Meter::new(METER).value("n/a").visual(visual),
                );
                assert_eq!(count_symbol(&scene, "━"), 0, "{level:?} {visual:?}");
                assert_eq!(count_symbol(&scene, "─"), 0, "{level:?} {visual:?}");
                assert!(scene.text().contains("n/a"), "{level:?} {visual:?}");
            }
            // An explicit stale tone survives without a ratio, still run-less.
            let stale = draw_meter(
                theme.clone(),
                level,
                30,
                Meter::new(METER).value("stale").tone(MeterTone::Stale),
            );
            assert_eq!(count_symbol(&stale, "━"), 0, "{level:?}");
            assert!(stale.text().contains("stale"), "{level:?}");
        }
    }
}

#[test]
fn w029_02_series_tones_wrap_six_tokens() {
    fn thumb_fg(theme: &Theme, level: ColorLevel, tone: MeterTone) -> Option<Color> {
        let scene = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).ratio(0.5).tone(tone),
        );
        scene.buffer().cell((0, 0)).map(|c| c.fg)
    }
    for theme in [Theme::junie(), Theme::paper()] {
        for level in LEVELS {
            for n in 0..12_u8 {
                assert_eq!(
                    thumb_fg(&theme, level, MeterTone::Series(n)),
                    thumb_fg(&theme, level, MeterTone::Series(n % 6)),
                    "series {n} {level:?}"
                );
            }
            // Distinct tokens stay distinct on paint; shared tokens stay shared.
            let tokens = theme.for_level(level).color.meter.series;
            for a in 0..6_u8 {
                for b in 0..6_u8 {
                    let pa = thumb_fg(&theme, level, MeterTone::Series(a));
                    let pb = thumb_fg(&theme, level, MeterTone::Series(b));
                    if tokens[a as usize] == tokens[b as usize] {
                        assert_eq!(pa, pb, "series {a}/{b} {level:?}");
                    } else {
                        assert_ne!(pa, pb, "series {a}/{b} {level:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn w029_02_line_readout_beside_run_geometry() {
    // Width 30, value "50%": tail is 4 cells, track 26, filled 13.
    let scene = draw_meter(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.5).value("50%"),
    );
    assert_eq!(count_symbol(&scene, "━"), 13);
    assert_eq!(count_symbol(&scene, "─"), 13);
    for x in 0..13_u16 {
        assert_eq!(
            scene.buffer().cell((x, 0)).map(Cell::symbol),
            Some("━"),
            "col {x}"
        );
    }
    for x in 13..26_u16 {
        assert_eq!(
            scene.buffer().cell((x, 0)).map(Cell::symbol),
            Some("─"),
            "col {x}"
        );
    }
    let readout: String = scene.text().chars().skip(27).take(3).collect();
    assert_eq!(readout, "50%");
}

#[test]
fn w029_02_block_on_fill_shares_value_plane() {
    // Ratio 0.25 over width 30: filled 8 cells at Low tone, so a long
    // value spans the filled/unfilled boundary inside the bar.
    for level in LEVELS {
        let theme = Theme::junie().for_level(level);
        let scene = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER)
                .ratio(0.25)
                .value("12 of 48 units")
                .visual(MeterVisual::Block),
        );
        let filled = scene.buffer().cell((2, 0)).unwrap();
        let rest = scene.buffer().cell((12, 0)).unwrap();
        // The used share carries the tone as background; the remainder
        // carries the ordered lift of the canvas surface. The theme is
        // already narrowed, so the token is used directly (narrowing
        // twice would shift Mono Gray to White).
        assert_eq!(filled.bg, theme.color.meter.low);
        assert_eq!(rest.bg, pinned_lift(&theme, Surface::Canvas));
        // The on-fill restyle changes the value ink across the boundary.
        assert_ne!(filled.fg, rest.fg);
        // The unfilled ink is the same LABEL ink a value-only meter paints.
        let bare = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).value("12 of 48 units"),
        );
        let bare_value = bare.buffer().cell((0, 0)).unwrap();
        assert_eq!(rest.fg, bare_value.fg);
    }
}

#[test]
fn w029_02_stale_unknown_use_own_roles() {
    let stale_ink = Color::Rgb(12, 123, 210);
    let unknown_ink = Color::Rgb(210, 100, 13);
    for level in LEVELS {
        let mut theme = Theme::junie();
        theme.color.meter.stale = stale_ink;
        theme.color.meter.unknown = unknown_ink;
        for (tone, ink) in [
            (MeterTone::Stale, stale_ink),
            (MeterTone::Unknown, unknown_ink),
        ] {
            let scene = draw_meter(
                theme.clone(),
                level,
                30,
                Meter::new(METER).ratio(0.5).tone(tone),
            );
            assert_eq!(
                scene.buffer().cell((0, 0)).map(|c| c.fg),
                Some(downgrade_color(ink, level)),
                "{tone:?} {level:?}"
            );
        }
        // Explicit stale wins over a ratio that would derive High.
        let scene = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).ratio(0.95).tone(MeterTone::Stale),
        );
        assert_eq!(
            scene.buffer().cell((0, 0)).map(|c| c.fg),
            Some(downgrade_color(stale_ink, level)),
            "{level:?}"
        );
    }
}

// ── W-029-03 thresholds, wrap boundaries, and tiny widths ──

#[test]
fn w029_03_thresholds_below_equal_above_all_ladder_rungs() {
    fn derived_fg(theme: &Theme, level: ColorLevel, ratio: f64) -> Option<Color> {
        let scene = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).ratio(ratio).value("v"),
        );
        scene.buffer().cell((0, 0)).map(|c| c.fg)
    }
    // Same ratio as the derived probe: at ratio 0.0 both paint the
    // track cell, at every other ratio both paint the filled thumb.
    fn explicit_fg(theme: &Theme, level: ColorLevel, ratio: f64, tone: MeterTone) -> Option<Color> {
        let scene = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).ratio(ratio).value("v").tone(tone),
        );
        scene.buffer().cell((0, 0)).map(|c| c.fg)
    }
    for theme in [Theme::junie(), Theme::paper()] {
        let thresholds = theme.design.meter;
        assert_eq!((thresholds.low_max, thresholds.medium_max), (59, 84));
        for level in LEVELS {
            // Each boundary probed immediately below, equal, and above.
            for (ratio, tone) in [
                (0.0, MeterTone::Low),
                (0.59, MeterTone::Low),
                (0.60, MeterTone::Medium),
                (0.84, MeterTone::Medium),
                (0.85, MeterTone::High),
                (1.0, MeterTone::High),
            ] {
                assert_eq!(
                    MeterTone::from_ratio(ratio, thresholds),
                    tone,
                    "ratio {ratio} {level:?}"
                );
                assert_eq!(
                    derived_fg(&theme, level, ratio),
                    explicit_fg(&theme, level, ratio, tone),
                    "ratio {ratio} {level:?}"
                );
            }
            // The threshold tone reaches the painted foreground: distinct
            // tokens stay distinct wherever the capability keeps them apart.
            let tokens = theme.for_level(level).color.meter;
            let narrowed = [
                (MeterTone::Low, tokens.low),
                (MeterTone::Medium, tokens.medium),
                (MeterTone::High, tokens.high),
            ];
            for (ta, ca) in narrowed {
                for (tb, cb) in narrowed {
                    let pa = explicit_fg(&theme, level, 0.5, ta);
                    let pb = explicit_fg(&theme, level, 0.5, tb);
                    if ca == cb {
                        assert_eq!(pa, pb, "{ta:?}/{tb:?} {level:?}");
                    } else {
                        assert_ne!(pa, pb, "{ta:?}/{tb:?} {level:?}");
                    }
                }
            }
        }
        // A theme that moves the thresholds moves every derived meter.
        let mut custom = theme.clone();
        custom.design.meter.low_max = 10;
        custom.design.meter.medium_max = 20;
        let tight = custom.design.meter;
        for (ratio, tone) in [
            (0.10, MeterTone::Low),
            (0.11, MeterTone::Medium),
            (0.20, MeterTone::Medium),
            (0.21, MeterTone::High),
        ] {
            assert_eq!(MeterTone::from_ratio(ratio, tight), tone, "ratio {ratio}");
            assert_eq!(
                derived_fg(&custom, ColorLevel::TrueColor, ratio),
                explicit_fg(&custom, ColorLevel::TrueColor, ratio, tone),
                "ratio {ratio}"
            );
        }
    }
}

#[test]
fn w029_03_tiny_width_exact_fallbacks() {
    // Value-only fallbacks at degenerate widths; the indeterminate bar
    // has no percentage to report, so it paints nothing.
    for (w, meter, bar_det, bar_indet, spin) in [
        (1_u16, "5", "5", " ", "⠋"),
        (2, "50", "50", "  ", "⠋ "),
        (3, "50%", "50%", "   ", "⠋  "),
    ] {
        let scene = draw_meter(
            Theme::junie(),
            ColorLevel::TrueColor,
            w,
            Meter::new(METER).ratio(0.5),
        );
        assert_eq!(scene.text(), meter, "meter line w={w}");
        let scene = draw_meter(
            Theme::junie(),
            ColorLevel::TrueColor,
            w,
            Meter::new(METER).ratio(0.5).visual(MeterVisual::Block),
        );
        assert_eq!(scene.text(), meter, "meter block w={w}");
        let scene = draw_bar(
            Theme::junie(),
            ColorLevel::TrueColor,
            w,
            ProgressBar::new(BAR).ratio(0.5),
        );
        assert_eq!(scene.text(), bar_det, "bar det w={w}");
        let scene = draw_bar(
            Theme::junie(),
            ColorLevel::TrueColor,
            w,
            ProgressBar::new(BAR),
        );
        assert_eq!(scene.text(), bar_indet, "bar indet w={w}");
        let mut scene = Scene::new("w029-tiny", Theme::junie(), ColorLevel::TrueColor, w, 1);
        scene.draw(|ui, area| {
            Spinner::new(SPIN).label("Wait").draw(ui, area);
        });
        assert_eq!(scene.text(), spin, "spinner w={w}");
    }
    // A zero-width rect paints nothing and measures nothing.
    let scene = draw_meter(
        Theme::junie(),
        ColorLevel::TrueColor,
        0,
        Meter::new(METER).ratio(0.5),
    );
    assert_eq!(scene.text(), "");
    let scene = draw_bar(
        Theme::junie(),
        ColorLevel::TrueColor,
        0,
        ProgressBar::new(BAR).ratio(0.5),
    );
    assert_eq!(scene.text(), "");
}

// ── W-029-04 ordered reference policy and authored wins ──

#[test]
fn w029_04_ordered_reference_policy_through_block_rest() {
    // Authored alias collisions: equal surface colors still follow the
    // Canvas → Surface/Elevated → Field → Popover order, never RGB guess.
    for colors in [
        [
            Color::Red,
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Red,
            Color::Magenta,
        ],
        [
            Color::Black,
            Color::Red,
            Color::Blue,
            Color::Green,
            Color::Yellow,
            Color::Magenta,
            Color::Magenta,
        ],
    ] {
        let mut source = Theme::paper();
        source.capability_palettes = None;
        let [canvas, surface, elevated, overlay, popover, field, field_hover] = colors;
        source.color.surfaces = [canvas, surface, elevated, overlay, popover];
        source.color.field = field;
        source.color.field_hover = field_hover;
        source.color.meter.fill_rest = MeterFillRest::ReferenceLift;
        for level in [ColorLevel::TrueColor, ColorLevel::Mono] {
            for surface in SURFACES {
                let theme = source.for_level(level);
                let expected = pinned_lift(&theme, surface);
                let mut scene = Scene::new("w029-alias", theme, level, 12, 1);
                scene.draw(|ui, area| {
                    ui.with_surface(surface, |ui| {
                        Meter::new(METER)
                            .ratio(0.0)
                            .visual(MeterVisual::Block)
                            .draw(ui, area);
                    });
                });
                assert!(
                    scene
                        .buffer()
                        .content
                        .iter()
                        .all(|cell| cell.bg == expected),
                    "{level:?}/{surface:?}"
                );
            }
        }
    }
}

#[test]
fn w029_04_explicit_color_rest_stays_literal() {
    let explicit = Color::Rgb(17, 177, 92);
    for base in [Theme::junie(), Theme::paper()] {
        let mut custom = base;
        custom.color.meter.fill_rest = MeterFillRest::Color(explicit);
        for level in LEVELS {
            let mut scene = Scene::new("w029-rest", custom.clone(), level, 12, 1);
            scene.draw(|ui, area| {
                ui.with_surface(Surface::Popover, |ui| {
                    Meter::new(METER)
                        .ratio(0.0)
                        .visual(MeterVisual::Block)
                        .draw(ui, area);
                });
            });
            assert_eq!(
                scene.buffer().cell((10, 0)).map(|c| c.bg),
                Some(downgrade_color(explicit, level)),
                "{level:?}"
            );
        }
    }
    assert!(matches!(
        Theme::paper().color.meter.fill_rest,
        MeterFillRest::Color(_)
    ));
}

// ── W-029-03/04 negative cases: recorded mutants stay rejected ──

#[test]
fn w029_03_unknown_never_paints_zero_run() {
    // No-ratio "0" is value-only; ratio 0.0 paints the quiet run.
    let unknown = draw_meter(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).value("0"),
    );
    let zero = draw_meter(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.0),
    );
    assert_eq!(count_symbol(&unknown, "─"), 0);
    assert_eq!(count_symbol(&unknown, "━"), 0);
    assert!(count_symbol(&zero, "─") > 0);
    assert_ne!(unknown.buffer(), zero.buffer());
    assert!(unknown.text().contains('0'));
}

#[test]
fn w029_03_off_by_one_threshold_rejected() {
    let theme = Theme::junie();
    let t = theme.design.meter;
    // Whole-percent boundaries: 59 is Low, 60 is Medium, 84 Medium, 85 High.
    assert_eq!(MeterTone::from_ratio(0.59, t), MeterTone::Low);
    assert_eq!(MeterTone::from_ratio(0.60, t), MeterTone::Medium);
    assert_eq!(MeterTone::from_ratio(0.84, t), MeterTone::Medium);
    assert_eq!(MeterTone::from_ratio(0.85, t), MeterTone::High);
    // The painted readout agrees with the tone: no float skew between
    // the percentage text and the threshold decision.
    for (ratio, pct, tone) in [
        (0.59, "59%", MeterTone::Low),
        (0.60, "60%", MeterTone::Medium),
        (0.84, "84%", MeterTone::Medium),
        (0.85, "85%", MeterTone::High),
    ] {
        let derived = draw_meter(
            theme.clone(),
            ColorLevel::TrueColor,
            30,
            Meter::new(METER).ratio(ratio),
        );
        let explicit = draw_meter(
            theme.clone(),
            ColorLevel::TrueColor,
            30,
            Meter::new(METER).ratio(0.5).tone(tone),
        );
        assert!(derived.text().contains(pct), "ratio {ratio}");
        assert_eq!(
            derived.buffer().cell((0, 0)).map(|c| c.fg),
            explicit.buffer().cell((0, 0)).map(|c| c.fg),
            "ratio {ratio}"
        );
    }
    // Adjacent rungs paint apart in truecolor.
    let low = draw_meter(
        theme.clone(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.59),
    );
    let medium = draw_meter(
        theme.clone(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.60),
    );
    assert_ne!(
        low.buffer().cell((0, 0)).map(|c| c.fg),
        medium.buffer().cell((0, 0)).map(|c| c.fg)
    );
}

#[test]
fn w029_03_wrap_uses_integer_period_not_float_clamp() {
    // Enormous frames wrap safely on both motion paths.
    let huge = draw_bar(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        ProgressBar::new(BAR).frame(usize::MAX),
    );
    let wrapped = draw_bar(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        ProgressBar::new(BAR).frame(usize::MAX % 33),
    );
    assert_eq!(huge.buffer(), wrapped.buffer());
    // Empty frame tables paint no frame cell; the authored gap still
    // separates the (empty) frame from the label, and measurement agrees.
    let mut theme = Theme::junie();
    theme.design.motion.spinner_frames = &[];
    let mut scene = Scene::new("w029-empty", theme.clone(), ColorLevel::TrueColor, 20, 1);
    scene.draw(|ui, area| {
        let spinner = Spinner::new(SPIN).label("Wait").frame(usize::MAX);
        assert_eq!(
            spinner.measure(ui, Constraints::loose(20, 1)).preferred,
            (6, 1)
        );
        assert_eq!(spinner.draw(ui, area), Rect::new(0, 0, 6, 1));
    });
    assert_eq!(scene.buffer().cell((0, 0)).map(Cell::symbol), Some(" "));
    assert_eq!(scene.buffer().cell((2, 0)).map(Cell::symbol), Some("W"));
    // A busy bar with no frames still paints its track and readout.
    let scene = draw_bar(
        theme,
        ColorLevel::TrueColor,
        30,
        ProgressBar::new(BAR).ratio(0.5).status(Status::Busy),
    );
    assert!(scene.text().contains("50%"));
    assert_eq!(count_symbol(&scene, "━"), 12);
}

#[test]
fn w029_03_equal_rgb_tokens_never_reroute_roles() {
    // The Low token equals the canvas color: the thumb still paints the
    // authored token, and the lift still follows surface order.
    let mut theme = Theme::junie();
    let canvas = theme.bg(Surface::Canvas);
    theme.color.meter.low = canvas;
    for level in LEVELS {
        let scene = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).ratio(0.1).visual(MeterVisual::Line),
        );
        assert_eq!(
            scene.buffer().cell((0, 0)).map(|c| c.fg),
            Some(downgrade_color(canvas, level)),
            "{level:?}"
        );
        let block = draw_meter(
            theme.clone(),
            level,
            30,
            Meter::new(METER).ratio(0.0).visual(MeterVisual::Block),
        );
        assert_eq!(
            block.buffer().cell((10, 0)).map(|c| c.bg),
            Some(pinned_lift(&theme.for_level(level), Surface::Canvas)),
            "{level:?}"
        );
    }
    // Colliding Medium/High tokens stay threshold-driven: the mapping
    // distinguishes rungs even when the tokens do not.
    let mut shared = Theme::junie();
    shared.color.meter.medium = Color::Rgb(9, 9, 9);
    shared.color.meter.high = Color::Rgb(9, 9, 9);
    let t = shared.design.meter;
    assert_eq!(MeterTone::from_ratio(0.6, t), MeterTone::Medium);
    assert_eq!(MeterTone::from_ratio(0.9, t), MeterTone::High);
    let medium = draw_meter(
        shared.clone(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.6),
    );
    let high = draw_meter(
        shared,
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.9),
    );
    assert_eq!(
        medium.buffer().cell((0, 0)).map(|c| c.fg),
        high.buffer().cell((0, 0)).map(|c| c.fg)
    );
}

#[test]
fn w029_03_explicit_beats_derived_everywhere() {
    // Explicit tone beats any ratio derivation, both directions.
    // (Every probe ratio keeps a filled run so the thumb cell exists.)
    for (ratio, tone, token) in [
        (0.95, MeterTone::Low, Theme::junie().color.meter.low),
        (0.25, MeterTone::High, Theme::junie().color.meter.high),
        (
            0.5,
            MeterTone::Series(4),
            Theme::junie().color.meter.series[4],
        ),
    ] {
        let scene = draw_meter(
            Theme::junie(),
            ColorLevel::TrueColor,
            30,
            Meter::new(METER).ratio(ratio).tone(tone),
        );
        assert_eq!(
            scene.buffer().cell((0, 0)).map(|c| c.fg),
            Some(token),
            "{tone:?}"
        );
    }
    // An instance patch beats the tone default on the same part.
    let patches = [(
        Part::THUMB,
        StylePatch::new()
            .set_fg(Role::Custom(Color::Red))
            .set_bg(Role::Custom(Color::Blue)),
    )];
    let scene = draw_meter(
        Theme::junie(),
        ColorLevel::TrueColor,
        30,
        Meter::new(METER).ratio(0.95).patch_part(&patches),
    );
    assert_eq!(
        scene.buffer().cell((0, 0)).map(|c| (c.fg, c.bg)),
        Some((Color::Red, Color::Blue))
    );
}
