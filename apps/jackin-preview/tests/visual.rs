//! Cell-exact visual digest matrix for the Jackin preview shell.
//!
//! Every deterministic scenario renders one paused fixture frame driven by
//! the real `App` on a `TestBackend`, and the whole buffer (symbol, fg, bg,
//! modifier of every cell) is folded into one digest per matrix cell. The
//! matrix is stable across machines and can only change with an authored
//! render change; regenerate only with `BLESS=1` after inspecting the diff.

use jackin_app::{App, Motion, Scenario};
use junie_tui::{ColorLevel, FeedbackClock, Theme};
use junie_tui_testing::{Baseline, Harness};

const BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/jackin.txt"
));

const SIZES: [(u16, u16); 2] = [(100, 30), (120, 40)];
const COLORS: [ColorLevel; 2] = [ColorLevel::TrueColor, ColorLevel::Mono];

#[test]
fn jackin_visual_baseline() {
    for scenario in Scenario::ALL {
        for (width, height) in SIZES {
            for theme in [Theme::junie(), Theme::paper()] {
                for color in COLORS {
                    let app = App::for_scenario(scenario, Motion::Paused);
                    let mut harness =
                        Harness::new(app, theme.clone(), width, height).with_color(color);
                    harness.draw();
                    harness
                        .snapshot()
                        .named(scenario.name())
                        .assert_against(&BASELINE);
                }
            }
        }
    }
}
