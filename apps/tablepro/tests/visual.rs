//! `TablePro`'s 21-surface × 2-size deterministic visual matrix.

use junie_tui::Theme;
use junie_tui_testing::Harness;
use tablepro_app::{Surface, TableProApp};

fn scene(surface: Surface, width: u16, height: u16) -> (String, String) {
    let mut app = TableProApp::default();
    app.set_surface(surface);
    let harness = Harness::new(app, Theme::junie(), width, height);
    (harness.snapshot().key(), harness.text())
}

#[test]
fn tablepro_visual_baseline() {
    for (width, height) in [(120, 40), (80, 24)] {
        for surface in Surface::ALL {
            let (first, text) = scene(surface, width, height);
            let (second, _) = scene(surface, width, height);
            assert_eq!(
                first,
                second,
                "{width}x{height} {} is not deterministic",
                surface.label()
            );
            assert!(
                text.contains(surface.label()),
                "surface label missing: {}",
                surface.label()
            );
        }
    }
}
