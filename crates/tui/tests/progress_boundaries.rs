//! Rendered progress boundaries stay deterministic at fill and sweep edges.
use junie_tui::{
    ColorLevel, GlyphRole, Id, Meter, MeterTone, MeterVisual, ProgressBar, Spinner, Status, Theme,
};
use junie_tui_testing::Scene;

const BAR: Id = Id::root("progress.boundaries");
const METER: Id = Id::root("meter.boundaries");
const SPINNER: Id = Id::root("spinner.boundaries");

fn draw(ratio: Option<f64>, frame: usize) -> Scene {
    let mut scene = Scene::new("progress", Theme::junie(), ColorLevel::TrueColor, 30, 1);
    scene.draw(|ui, area| {
        let bar = ProgressBar::new(BAR).frame(frame);
        if let Some(ratio) = ratio {
            bar.ratio(ratio).draw(ui, area);
        } else {
            bar.draw(ui, area);
        }
    });
    scene
}

fn count(scene: &Scene, symbol: &str) -> usize {
    scene
        .buffer()
        .content
        .iter()
        .filter(|cell| cell.symbol() == symbol)
        .count()
}

fn draw_meter(theme: Theme, ratio: Option<f64>, visual: MeterVisual, value: Option<&str>) -> Scene {
    let mut scene = Scene::new("meter", theme, ColorLevel::TrueColor, 30, 1);
    scene.draw(|ui, area| {
        let meter = Meter::new(METER).visual(visual);
        let meter = if let Some(ratio) = ratio {
            meter.ratio(ratio)
        } else {
            meter
        };
        let meter = if let Some(value) = value {
            meter.value(value)
        } else {
            meter
        };
        meter.draw(ui, area);
    });
    scene
}

#[test]
fn determinate_fill_rounds_at_clamped_edges() {
    for (ratio, filled) in [
        (0.0, 0),
        (0.02, 0),
        (0.04, 1),
        (0.5, 12),
        (0.999, 23),
        (1.0, 23),
        (1.5, 23),
        (-0.5, 0),
    ] {
        let scene = draw(Some(ratio), 0);
        assert_eq!(count(&scene, "━"), filled, "ratio {ratio}");
        assert_eq!(count(&scene, "─"), 23 - filled, "ratio {ratio}");
        assert!(scene.text().contains(&format!(
            "{}%",
            (ratio.clamp(0.0, 1.0) * 100.0).round() as u8
        )));
    }
}

#[test]
fn indeterminate_sweep_uses_integer_period_33() {
    let frame = |n| draw(None, n).text();
    assert_eq!(frame(33), frame(0));
    assert_eq!(frame(66), frame(0));
    assert_ne!(frame(32), frame(0));
    assert_eq!(frame(34), format!("━{}  ", "─".repeat(27)));
}

#[test]
fn progress_states_use_theme_status_glyphs() {
    for theme in [Theme::junie(), Theme::paper()] {
        let busy_glyph = theme
            .design
            .motion
            .spinner_frames
            .first()
            .copied()
            .unwrap_or("");
        for (status, expected) in [
            (Status::Error, theme.design.glyphs.get(GlyphRole::Error)),
            (Status::Busy, busy_glyph),
            (Status::Loading, busy_glyph),
        ] {
            let mut scene = Scene::new(
                "progress_status",
                theme.clone(),
                ColorLevel::TrueColor,
                30,
                1,
            );
            scene.draw(|ui, area| {
                ProgressBar::new(BAR)
                    .ratio(0.5)
                    .status(status)
                    .draw(ui, area);
            });
            assert!(
                scene
                    .buffer()
                    .content
                    .iter()
                    .any(|cell| cell.symbol() == expected),
                "{status:?}: {:?}",
                scene.text()
            );
        }
    }
}

#[test]
fn spinner_uses_all_theme_frames_and_wraps_by_frame_count() {
    let ascii = Theme::junie().builder().ascii_glyphs().build();
    for theme in [Theme::junie(), ascii] {
        let frames = theme.design.motion.spinner_frames;
        for (frame, expected) in frames.iter().enumerate() {
            let mut scene = Scene::new("spinner_frame", theme.clone(), ColorLevel::TrueColor, 4, 1);
            scene.draw(|ui, area| {
                Spinner::new(SPINNER).frame(frame).draw(ui, area);
            });
            assert_eq!(
                scene.buffer().cell((0, 0)).map(junie_tui::Cell::symbol),
                Some(*expected)
            );
        }
        let mut wrapped = Scene::new("spinner_wrap", theme.clone(), ColorLevel::TrueColor, 4, 1);
        let mut first = Scene::new("spinner_first", theme, ColorLevel::TrueColor, 4, 1);
        wrapped.draw(|ui, area| {
            Spinner::new(SPINNER).frame(frames.len()).draw(ui, area);
        });
        first.draw(|ui, area| {
            Spinner::new(SPINNER).frame(0).draw(ui, area);
        });
        assert_eq!(wrapped.buffer(), first.buffer());
    }
}

#[test]
fn meter_distinguishes_unknown_zero_and_full() {
    let unknown = draw_meter(Theme::junie(), None, MeterVisual::Line, Some("0"));
    let zero = draw_meter(Theme::junie(), Some(0.0), MeterVisual::Line, None);
    let full = draw_meter(Theme::junie(), Some(1.0), MeterVisual::Line, None);
    assert_eq!(count(&unknown, "━"), 0);
    assert_eq!(count(&unknown, "─"), 0);
    assert!(count(&zero, "─") > 0);
    assert!(count(&full, "━") > 0);
    assert_eq!(count(&full, "─"), 0);
    for visual in [MeterVisual::Block, MeterVisual::Line] {
        assert!(
            draw_meter(Theme::junie(), Some(0.0), visual, None)
                .text()
                .contains("0%")
        );
        assert!(
            draw_meter(Theme::junie(), Some(1.0), visual, None)
                .text()
                .contains("100%")
        );
    }
}

#[test]
fn meter_rendering_tracks_thresholds_and_series_tokens() {
    let theme = Theme::junie();
    for (ratio, tone, value) in [
        (0.59, MeterTone::Low, "59%"),
        (0.60, MeterTone::Medium, "60%"),
        (0.84, MeterTone::Medium, "84%"),
        (0.85, MeterTone::High, "85%"),
    ] {
        let derived = draw_meter(theme.clone(), Some(ratio), MeterVisual::Line, None);
        assert!(derived.text().contains(value), "ratio {ratio}");
        let mut explicit_tone =
            Scene::new("meter_tone", theme.clone(), ColorLevel::TrueColor, 30, 1);
        explicit_tone.draw(|ui, area| {
            Meter::new(METER).ratio(0.5).tone(tone).draw(ui, area);
        });
        assert_eq!(
            derived.buffer().cell((0, 0)).map(|cell| cell.fg),
            explicit_tone.buffer().cell((0, 0)).map(|cell| cell.fg),
            "ratio {ratio}"
        );
    }
    for n in 0..12_u8 {
        let mut scene = Scene::new("meter_series", theme.clone(), ColorLevel::TrueColor, 30, 1);
        scene.draw(|ui, area| {
            Meter::new(METER)
                .ratio(0.5)
                .tone(MeterTone::Series(n))
                .draw(ui, area);
        });
        let mut wrapped = Scene::new(
            "meter_series_wrap",
            theme.clone(),
            ColorLevel::TrueColor,
            30,
            1,
        );
        wrapped.draw(|ui, area| {
            Meter::new(METER)
                .ratio(0.5)
                .tone(MeterTone::Series(n % 6))
                .draw(ui, area);
        });
        assert_eq!(scene.buffer(), wrapped.buffer(), "series {n}");
    }
}
