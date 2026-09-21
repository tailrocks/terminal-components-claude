//! TP-001: preset C Connections launch through the real CLI and `TableProApp`
//! constructors. Does not seed `set_surface(Connections)`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use std::ffi::OsStr;
use std::path::Path;

use junie_tui::{ColorLevel, Id};
use oracle_tablepro::{
    ALL_SIZES, CHECKPOINT_INITIAL, COLOR_PROFILES, ColorProfile, HELP, Observation, PRESET_C,
    SCENARIO_TP001, invoke_cli, invoke_cli_with_env, launch_preset_c, observe, parse_color_flag,
};
use tablepro_app::{Screen, Surface};

fn bin() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_tablepro"))
}

fn capture_initial(width: u16, height: u16, profile: ColorProfile) -> Observation {
    let harness = launch_preset_c(width, height, profile);
    observe(
        &harness,
        SCENARIO_TP001,
        PRESET_C,
        CHECKPOINT_INITIAL,
        profile,
    )
}

fn assert_connections_chrome(obs: &Observation) {
    assert_eq!(obs.scenario, SCENARIO_TP001);
    assert_eq!(obs.preset, PRESET_C);
    assert_eq!(obs.checkpoint, CHECKPOINT_INITIAL);
    assert_eq!(obs.screen, Screen::Connections);
    assert_eq!(obs.surface, Surface::Connections);
    assert!(obs.filter.is_empty(), "preset C launch has no filter text");
    assert_eq!(obs.selected, 0);
    assert!(!obs.form_open);
    assert!(!obs.quit);
    assert_eq!(obs.query_counter, 1);
    assert_eq!(obs.tab_count, 1);
    assert_eq!(obs.active_query_name.as_deref(), Some("Query 1"));
    assert!(obs.focus.is_some(), "Connections launch focuses a control");
    assert!(
        !obs.ring.is_empty(),
        "Connections launch publishes a focus ring"
    );
    assert!(
        obs.text.contains("TablePro"),
        "identity strip missing TablePro\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Connections"),
        "identity strip missing Connections\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("6 saved"),
        "identity strip missing saved count\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Filter connections"),
        "filter placeholder missing\n{}",
        obs.text
    );
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
        "tree missing first connection\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Production"),
        "tree missing Production\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Move"),
        "footer missing Move\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Connect"),
        "footer missing Connect\n{}",
        obs.text
    );
    assert!(
        obs.text.contains("Filter"),
        "footer missing Filter\n{}",
        obs.text
    );
    assert!(obs.text.contains("New"), "footer missing New\n{}", obs.text);
    assert!(
        obs.text.contains("Next"),
        "footer missing Next\n{}",
        obs.text
    );
    if obs.width >= 100 {
        assert!(
            obs.text.contains("Engine"),
            "detail card missing Engine at {}x{}\n{}",
            obs.width,
            obs.height,
            obs.text
        );
        assert!(
            obs.text.contains("localhost"),
            "detail card missing host at {}x{}\n{}",
            obs.width,
            obs.height,
            obs.text
        );
        assert!(
            obs.text.contains("acme_dev"),
            "detail card missing database at {}x{}\n{}",
            obs.width,
            obs.height,
            obs.text
        );
    }
}

#[test]
fn tp001_help_subprocess_matches_production_bytes() {
    let output = invoke_cli(bin(), &["--help"]).expect("help");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.contains(&0x1b));
    assert_eq!(String::from_utf8_lossy(&output.stdout), HELP);
}

#[test]
fn tp001_color_bad_subprocess_exits_two_without_echo() {
    let secret = "synthetic-color-sentinel";
    assert!(parse_color_flag(secret).is_none());
    let output = invoke_cli(bin(), &["--color", secret]).expect("bad color");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    assert!(!output.stderr.contains(&0x1b));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(!err.contains(secret));
    assert!(err.contains("--color must be truecolor, 256, 16, or none"));
}

#[test]
fn tp001_connect_missing_subprocess_exits_two_without_echo() {
    let name = "missing";
    let output = invoke_cli(bin(), &["--connect", name]).expect("missing connect");
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
    assert!(!output.stderr.contains(&0x1b));
    let err = String::from_utf8_lossy(&output.stderr);
    assert!(!err.contains(name));
    assert!(err.contains("no connection with the requested name"));
}

#[test]
fn tp001_color_aliases_match_cli_parser_and_help() {
    for (value, level) in [
        ("truecolor", ColorLevel::TrueColor),
        ("24bit", ColorLevel::TrueColor),
        ("256", ColorLevel::Ansi256),
        ("ansi256", ColorLevel::Ansi256),
        ("16", ColorLevel::Ansi16),
        ("ansi16", ColorLevel::Ansi16),
        ("none", ColorLevel::Mono),
        ("mono", ColorLevel::Mono),
    ] {
        assert_eq!(parse_color_flag(value), Some(level), "{value}");
        let output = invoke_cli(bin(), &["--color", value, "--help"]).expect("alias help");
        assert_eq!(output.status.code(), Some(0), "{value}");
        assert_eq!(String::from_utf8_lossy(&output.stdout), HELP);
    }
}

#[test]
fn tp001_nocolor_env_still_serves_help() {
    let output = invoke_cli_with_env(bin(), &["--help"], &[("NO_COLOR", OsStr::new("1"))])
        .expect("nocolor help");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout), HELP);
}

#[test]
fn tp001_preset_c_launch_initial_all_sizes_and_profiles() {
    for (width, height) in ALL_SIZES {
        for profile in COLOR_PROFILES {
            let first = capture_initial(width, height, profile);
            assert_eq!(first.width, width);
            assert_eq!(first.height, height);
            assert_eq!(first.profile, profile.label());
            assert_connections_chrome(&first);
            let second = capture_initial(width, height, profile);
            assert_eq!(
                first,
                second,
                "repeat mismatch {}x{} {}",
                width,
                height,
                profile.label()
            );
        }
    }
}

#[test]
fn tp001_nocolor_profile_uses_detect_table() {
    assert_eq!(ColorProfile::NoColor.color_level(), ColorLevel::Mono);
    assert_eq!(ColorProfile::None.color_level(), ColorLevel::Mono);
    assert_eq!(ColorProfile::NoColor.cli_color_value(), None);
    assert_eq!(ColorProfile::None.cli_color_value(), Some("none"));
}

#[test]
fn tp001_launch_does_not_use_surface_seed_path() {
    let harness = launch_preset_c(120, 40, ColorProfile::Truecolor);
    assert_eq!(harness.app().screen(), Screen::Connections);
    assert_eq!(harness.app().surface(), Surface::Connections);
    let tree = Id::root("tablepro.connections.list");
    assert!(
        harness.area_of(tree).is_some() || harness.focus().is_some(),
        "Connections tree or focus missing after real launch"
    );
    assert!(harness.find("Local PostgreSQL").is_some());
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
