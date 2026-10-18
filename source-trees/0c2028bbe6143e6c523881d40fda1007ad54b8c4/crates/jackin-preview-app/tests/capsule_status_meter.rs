//! WI-JACKIN-CAPSULE-02: capsule status row through owned components.
//!
//! The capsule status strip renders through the reusable [`StatusBar`] with
//! inline [`Meter`] items, fed by the focused pane account's usage windows.
//! These tests pin that composition:
//!
//! * the component-path row matches the frozen audit row exactly (text) at
//!   all five geometries;
//! * the meters follow fixture data through the real preview journey —
//!   changed usage repaints the run, so the content cannot be a stored
//!   answer;
//! * the paused 120x40 audit frame is component-owned on every row
//!   (WI-JACKIN-CAPSULE-03 deleted the historical frame): every row
//!   still matches frozen, and mutated usage moves row 38 while the
//!   body rows stay put;
//! * the row wears the frozen tones (stale meter run, muted labels,
//!   warning center, bold identity).
use jackin_preview_app::{App, Motion, Scenario};
use termrock_test_support::Harness;

fn frozen(path: &str) -> Vec<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(path);
    std::fs::read_to_string(&root)
        .unwrap()
        .lines()
        .map(|l| l.trim_end().to_string())
        .collect()
}

/// Component-path parity: at full motion (no ticks) the status row
/// matches the frozen audit text exactly.
#[test]
fn capsule_status_row_matches_frozen_at_all_sizes() {
    for (w, h) in [
        (72u16, 20u16),
        (80u16, 24u16),
        (100u16, 30u16),
        (120u16, 40u16),
        (160u16, 50u16),
    ] {
        let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
        let harness = Harness::new(app, termrock::Theme::junie(), w, h);
        let expected = frozen(&format!(
            "baselines/tuiscotti-v1/jackin/capsule/audit/{w}x{h}/truecolor.txt"
        ));
        let y = usize::from(h - 2);
        assert_eq!(
            harness.row(h - 2).trim_end(),
            expected[y],
            "{w}x{h} component-path status row must match frozen"
        );
    }
}

/// An 11-cell meter run with `filled` heavy cells.
fn run(filled: usize) -> String {
    format!("{}{}", "━".repeat(filled), "─".repeat(11 - filled))
}

/// Data binding: the meters read the focused pane account's usage windows.
/// Mutating the fixture repaints the run and the readout through the real
/// journey; the unmutated control keeps the fixture values.
#[test]
fn capsule_status_meter_follows_account_usage() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    assert!(
        harness.row(38).contains(&format!("Session {} 76%", run(8))),
        "control must show the fixture session meter, got {:?}",
        harness.row(38)
    );

    let mut app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let account = app
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("capsule fixture account exists");
    for win in account.usage.windows.iter_mut() {
        if win.id == "session" {
            win.used_pct = Some(38);
        }
        if win.id == "weekly" {
            win.used_pct = Some(59);
        }
    }
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    assert!(
        harness.row(38).contains(&format!("Session {} 38%", run(4))),
        "mutated session usage must repaint the meter, got {:?}",
        harness.row(38)
    );

    let mut app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let account = app
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("capsule fixture account exists");
    for win in account.usage.windows.iter_mut() {
        if win.id == "weekly" {
            win.used_pct = Some(59);
        }
    }
    let harness = Harness::new(app, termrock::Theme::junie(), 160, 50);
    assert!(
        harness.row(48).contains(&format!("Weekly {} 59%", run(6))),
        "mutated weekly usage must repaint the wide meter, got {:?}",
        harness.row(48)
    );
}

/// Full-frame ownership: the paused 120x40 audit frame still matches
/// frozen on every row with no historical frame left — mutated usage
/// moves row 38 through the component path while the body rows stay put.
#[test]
fn paused_audit_frame_is_fully_component_owned() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let expected = frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt");
    for y in 0..40u16 {
        assert_eq!(
            harness.row(y).trim_end(),
            expected[usize::from(y)],
            "audited paused row {y} must match frozen"
        );
    }

    let mut app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let account = app
        .world
        .accounts
        .get_mut("acct-claude-work")
        .expect("capsule fixture account exists");
    for win in account.usage.windows.iter_mut() {
        if win.id == "session" {
            win.used_pct = Some(12);
        }
    }
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    assert!(
        harness.row(38).contains(&format!("Session {} 12%", run(1))),
        "mutated usage must move the narrowed audit row, got {:?}",
        harness.row(38)
    );
    for y in (0..40u16).filter(|y| *y != 38) {
        assert_eq!(
            harness.row(y).trim_end(),
            expected[usize::from(y)],
            "body row {y} must stay frozen under mutated usage"
        );
    }
}

/// Tones: the stale meter run, muted labels, warning center and bold
/// identity match the frozen cells.
#[test]
fn capsule_status_row_wears_frozen_tones() {
    use termrock::Color;

    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let cell = |x: u16| harness.cell(x, 38);
    // Identity: bold primary.
    assert_eq!(cell(1).symbol(), "P");
    assert_eq!(cell(1).fg, Color::Rgb(255, 255, 255));
    assert!(cell(1).modifier.contains(termrock::Modifier::BOLD));
    // Center: warning.
    assert_eq!(cell(48).symbol(), "C");
    assert_eq!(cell(48).fg, Color::Rgb(245, 158, 9));
    // Meter label and value: muted.
    assert_eq!(cell(94).symbol(), "S");
    assert_eq!(cell(94).fg, Color::Rgb(128, 128, 128));
    assert_eq!(cell(114).symbol(), "7");
    assert_eq!(cell(114).fg, Color::Rgb(128, 128, 128));
    // Stale run: faint fill, subtle track, fill-wearing suffix.
    assert_eq!(cell(102).symbol(), "━");
    assert_eq!(cell(102).fg, Color::Rgb(77, 77, 77));
    assert_eq!(cell(110).symbol(), "─");
    assert_eq!(cell(110).fg, Color::Rgb(38, 38, 38));
    assert_eq!(cell(117).symbol(), " ");
    assert_eq!(cell(117).fg, Color::Rgb(77, 77, 77));
}
