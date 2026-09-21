//! TP-002: Connections navigation through real tree/focus handlers.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use oracle_tablepro::{
    ALL_SIZES, COLOR_PROFILES, ColorProfile, Observation, PRESET_C, SCENARIO_TP002, backtab, down,
    end, home, launch_preset_c, observe, tab, up,
};
use tablepro_app::{Screen, Surface};

fn capture(
    harness: &junie_tui_testing::Harness<tablepro_app::TableProApp>,
    checkpoint: &str,
    profile: ColorProfile,
) -> Observation {
    observe(harness, SCENARIO_TP002, PRESET_C, checkpoint, profile)
}

fn run_tp002(width: u16, height: u16, profile: ColorProfile) -> (Observation, Observation) {
    let mut harness = launch_preset_c(width, height, profile);
    down(&mut harness, 8);
    let production = capture(&harness, "production", profile);
    home(&mut harness);
    end(&mut harness);
    up(&mut harness, 1);
    tab(&mut harness);
    backtab(&mut harness);
    let traversal = capture(&harness, "traversal", profile);
    (production, traversal)
}

fn assert_grouped_tree(obs: &Observation) {
    assert_eq!(obs.screen, Screen::Connections);
    assert_eq!(obs.surface, Surface::Connections);
    assert!(!obs.form_open);
    assert!(!obs.quit);
    assert!(obs.focus.is_some(), "tree/focus missing\n{}", obs.text);
    assert!(!obs.ring.is_empty(), "empty focus ring\n{}", obs.text);
    assert!(
        obs.text.contains("Personal"),
        "grouped tree missing Personal\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Acme"),
        "grouped tree missing Acme\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Local PostgreSQL"),
        "tree missing Local PostgreSQL\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Production"),
        "tree missing Production\n{}",
        obs.text
    );
}

#[test]
fn tp002_down_home_end_tab_are_real_handlers() {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            let (production, traversal) = run_tp002(width, height, profile);
            assert_eq!(production.checkpoint, "production");
            assert_eq!(traversal.checkpoint, "traversal");
            assert_grouped_tree(&production);
            assert_grouped_tree(&traversal);
            assert_eq!(production.ring, traversal.ring, "focus ring drifted");
            assert!(
                production.selected < 6,
                "selected out of fixture {}",
                production.selected
            );
            let again = run_tp002(width, height, profile);
            assert_eq!(
                production, again.0,
                "production repeat {}x{}",
                width, height
            );
            assert_eq!(traversal, again.1, "traversal repeat {}x{}", width, height);
        }
    }
}
