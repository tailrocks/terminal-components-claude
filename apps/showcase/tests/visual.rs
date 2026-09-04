//! Visual digest matrix for the migrated showcase shell.
use showcase_app::{App, PageId};
use tui_next::{ColorLevel, Theme};
use tui_next_testing::Harness;

const BASELINE: tui_next_testing::Baseline = tui_next_testing::Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/showcase.txt"
));

const SIZES: [(u16, u16); 4] = [(80, 24), (100, 30), (120, 40), (160, 50)];
const COLORS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

/// Keep the legacy test name while covering every page in the full visual
/// matrix: four supported sizes, four terminal colour levels, and both themes.
#[test]
fn showcase_visual_baseline() {
    for page in PageId::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    let h = Harness::new(App::with_page(page), theme.clone(), width, height)
                        .with_color(color);
                    h.snapshot().named(page.title()).assert_against(&BASELINE);
                }
            }
        }
    }
}
