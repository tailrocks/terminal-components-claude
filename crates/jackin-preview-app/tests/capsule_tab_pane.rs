//! WI-JACKIN-CAPSULE-01: capsule tab/pane dispatch through owned components.
//!
//! The capsule tab strip and split panes render through reusable Termrock
//! components (`Tabs`, `SplitPane`, `Panel`, `Empty`, `TextViewport`,
//! `TextInput`). These tests pin that dispatch:
//!
//! * outside the frozen audit state the component path reproduces the
//!   frozen audit body rows exactly (symbols; styles are covered by the
//!   visual matrix once the status row follows in WI-JACKIN-CAPSULE-02);
//! * the frozen audit state itself still routes to the historical frame,
//!   which owns the session-meter status row until WI-JACKIN-CAPSULE-02;
//! * typing into the capsule input reaches the live component path and
//!   clearing it returns to the historical frame (dispatch boundary).
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

fn rows(h: &Harness<App>, height: u16) -> Vec<String> {
    (0..height)
        .map(|y| h.row(y).trim_end().to_string())
        .collect()
}

/// Component path parity: at full motion (no ticks) the app leaves the
/// historical gate and every body row outside the status row matches the
/// frozen audit text exactly.
#[test]
fn capsule_component_path_matches_audit_body_outside_status() {
    for (w, h) in [(120u16, 40u16), (80u16, 24u16)] {
        let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
        let harness = Harness::new(app, termrock::Theme::junie(), w, h);
        let actual = rows(&harness, h);
        let expected = frozen(&format!(
            "baselines/tuiscotti-v1/jackin/capsule/audit/{w}x{h}/truecolor.txt"
        ));
        assert_eq!(expected.len(), usize::from(h), "frozen frame has {h} rows");
        let status_y = usize::from(h - 2);
        for (y, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
            if y == status_y {
                continue;
            }
            assert_eq!(e, a, "{w}x{h} component-path row {y} must match frozen");
        }
    }
}

/// The component path owns the status row outside the audit state: it
/// renders live tab/pane counts instead of the frozen session meter.
/// WI-JACKIN-CAPSULE-02 replaces this content with the session meter and
/// then the historical gate below can be removed.
#[test]
fn capsule_component_status_shows_live_counts() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let status = harness.row(38);
    assert!(
        status.contains("3 tabs · 3 panes"),
        "component status must show live counts, got {status:?}"
    );
    let frozen_status =
        frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt")[38].clone();
    assert!(
        !status.trim_end().is_empty() && status.trim_end() != frozen_status,
        "component status must differ from the frozen session meter until WI-JACKIN-CAPSULE-02"
    );
}

/// Gate guard: the exact frozen audit state still renders the historical
/// frame (all 40 rows), which keeps the green 120x40/truecolor audit cell
/// green until WI-JACKIN-CAPSULE-02 moves the status row to components.
#[test]
fn audited_capsule_state_keeps_historical_frame() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let actual = rows(&harness, 40);
    let expected = frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt");
    assert_eq!(expected.len(), 40, "frozen frame has 40 rows");
    for (y, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(e, a, "audited paused row {y} must stay historical");
    }
}

/// Component-path journey: keys reach the live capsule input, the tab
/// strip stays intact through the journey, and clearing the input
/// restores the component body. (The paused historical frame registers
/// no input target, so typing only flows on the component path.)
#[test]
fn capsule_input_keys_reach_the_component_path() {
    use termrock::KeyCode;

    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let mut harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let frozen_tabs =
        frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt")[1].clone();
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.type_str("hello");
    assert!(
        (0..40).any(|y| harness.row(y).contains("hello")),
        "typed text must be visible in the capsule input"
    );
    assert_eq!(
        harness.row(1).trim_end(),
        frozen_tabs,
        "tab strip must survive the input journey"
    );
    assert!(
        harness.row(38).contains("3 tabs"),
        "status must stay on the component path during input"
    );
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Backspace);
    }
    assert!(
        !(0..40).any(|y| harness.row(y).contains("hello")),
        "clearing the input must remove the typed text"
    );
    assert_eq!(
        harness.row(1).trim_end(),
        frozen_tabs,
        "tab strip must survive clearing the input"
    );
}
