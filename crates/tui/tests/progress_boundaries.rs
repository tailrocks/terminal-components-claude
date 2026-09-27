//! Rendered progress boundaries stay deterministic at fill and sweep edges.
use junie_tui::{ColorLevel, Id, ProgressBar, Theme};
use junie_tui_testing::Scene;

const BAR: Id = Id::root("progress.boundaries");

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
