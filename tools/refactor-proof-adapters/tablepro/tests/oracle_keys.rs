//! Frozen `visual-baseline` key inventory coverage for `tablepro`.
//!
//! The grouped store carries 44 scenarios × 5 sizes × 5 colors = 1100 keys.
//! `ALL` TP rows use four sizes; this module proves the adapter also covers
//! the canonical 72×20 shell and the historically missing
//! `tablepro/query/results/160x50/nocolor` key. Observations drive production
//! handlers; they never bless oracle bytes.

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
    CANONICAL_SIZES, COLOR_PROFILES, ColorProfile, ORACLE_KEY_COUNT, ORACLE_SCENARIOS, PRESET_C,
    PRESET_Q, PRESET_T, PRESET_W, SCENARIO_TP001, ctrl, launch_preset_c, launch_preset_q,
    launch_preset_t, launch_preset_w, observe, oracle_key, ticks,
};
use tablepro_app::{Screen, Surface};

const QUERY_RESULTS_SQL: &str =
    "SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 25";

#[test]
fn oracle_inventory_is_44_times_5_times_5() {
    assert_eq!(ORACLE_SCENARIOS.len(), 44);
    assert_eq!(CANONICAL_SIZES.len(), 5);
    assert_eq!(COLOR_PROFILES.len(), 5);
    assert_eq!(ORACLE_KEY_COUNT, 1100);
    assert_eq!(
        ORACLE_KEY_COUNT,
        ORACLE_SCENARIOS.len() * CANONICAL_SIZES.len() * COLOR_PROFILES.len()
    );
    assert!(CANONICAL_SIZES.contains(&(72, 20)), "72x20 must be covered");
    assert!(CANONICAL_SIZES.contains(&(160, 50)));
    assert!(ORACLE_SCENARIOS.contains(&"query/results"));
    assert!(ORACLE_SCENARIOS.contains(&"audit/production"));
    assert!(ORACLE_SCENARIOS.contains(&"connections/form_advanced"));
    // Historically missing key formats exactly.
    assert_eq!(
        oracle_key("query/results", 160, 50, ColorProfile::NoColor),
        "tablepro/query/results/160x50/nocolor"
    );
    // `none` and `nocolor` are distinct store leaves.
    assert_eq!(
        oracle_key("query/results", 160, 50, ColorProfile::None),
        "tablepro/query/results/160x50/none"
    );
    assert_ne!(
        oracle_key("query/results", 160, 50, ColorProfile::None),
        oracle_key("query/results", 160, 50, ColorProfile::NoColor)
    );
    // No duplicate scenario leaves.
    let mut sorted = ORACLE_SCENARIOS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ORACLE_SCENARIOS.len());
}

#[test]
fn presets_cover_72x20_minimum_shell() {
    for profile in COLOR_PROFILES {
        let c = launch_preset_c(72, 20, profile);
        let obs_c = observe(&c, SCENARIO_TP001, PRESET_C, "initial", profile);
        assert_eq!(obs_c.width, 72);
        assert_eq!(obs_c.height, 20);
        assert_eq!(obs_c.screen, Screen::Connections);
        assert_eq!(obs_c.surface, Surface::Connections);
        let again = observe(
            &launch_preset_c(72, 20, profile),
            SCENARIO_TP001,
            PRESET_C,
            "initial",
            profile,
        );
        assert_eq!(obs_c, again, "72x20 C repeat {}", profile.label());

        let w = launch_preset_w(72, 20, profile);
        let obs_w = observe(&w, "TP-017", PRESET_W, "initial", profile);
        assert_eq!(obs_w.screen, Screen::Workbench);
        assert_eq!(
            obs_w,
            observe(
                &launch_preset_w(72, 20, profile),
                "TP-017",
                PRESET_W,
                "initial",
                profile
            ),
            "72x20 W repeat {}",
            profile.label()
        );

        let t = launch_preset_t(72, 20, profile);
        let obs_t = observe(&t, "TP-027", PRESET_T, "seeded", profile);
        assert_eq!(obs_t.screen, Screen::Workbench);

        let q = launch_preset_q(QUERY_RESULTS_SQL, 72, 20, profile);
        let obs_q = observe(&q, "TP-049", PRESET_Q, "preset", profile);
        assert_eq!(obs_q.screen, Screen::Workbench);
    }
}

fn run_query_results(
    width: u16,
    height: u16,
    profile: ColorProfile,
) -> oracle_tablepro::Observation {
    let mut h = launch_preset_q(QUERY_RESULTS_SQL, width, height, profile);
    ctrl(&mut h, 'r');
    ticks(&mut h, 7);
    observe(&h, "TP-049", PRESET_Q, "results", profile)
}

#[test]
fn historically_missing_query_results_160x50_nocolor_is_covered() {
    let profile = ColorProfile::NoColor;
    let first = run_query_results(160, 50, profile);
    assert_eq!(first.width, 160);
    assert_eq!(first.height, 50);
    assert_eq!(first.profile, "nocolor");
    assert_eq!(first.screen, Screen::Workbench);
    assert!(!first.text.is_empty());
    let second = run_query_results(160, 50, profile);
    assert_eq!(first, second, "missing-key repeat must be stable");
}

#[test]
fn query_results_covers_full_canonical_matrix() {
    for (width, height) in CANONICAL_SIZES {
        for profile in COLOR_PROFILES {
            let obs = run_query_results(width, height, profile);
            assert_eq!(obs.width, width);
            assert_eq!(obs.height, height);
            assert_eq!(obs.profile, profile.label());
            assert_eq!(
                obs.screen,
                Screen::Workbench,
                "{}x{} {}",
                width,
                height,
                profile.label()
            );
            let key = oracle_key("query/results", width, height, profile);
            assert!(
                key.starts_with("tablepro/query/results/"),
                "unexpected key {key}"
            );
        }
    }
}
