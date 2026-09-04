//! Visual digest matrix for the migrated showcase shell.
#![allow(clippy::arithmetic_side_effects)]

use showcase_app::{App, PageId};
use tui_next::{ColorLevel, Theme};
use tui_next_testing::Harness;

const BASELINE: tui_next_testing::Baseline = tui_next_testing::Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/showcase.txt"
));

/// Keep the legacy test name while covering shell, pages, dimensions, themes and colour.
#[test]
fn showcase_visual_baseline() {
    for page in PageId::ALL {
        for (width, height) in [(120, 40), (80, 24)] {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in [ColorLevel::TrueColor, ColorLevel::Mono] {
                    let h = Harness::new(App::with_page(page), theme.clone(), width, height)
                        .with_color(color);
                    h.snapshot().named(page.title()).assert_against(&BASELINE);
                }
            }
        }
    }
}
