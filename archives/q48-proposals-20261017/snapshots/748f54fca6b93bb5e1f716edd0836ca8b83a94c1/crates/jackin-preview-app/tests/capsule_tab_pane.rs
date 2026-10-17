//! WI-JACKIN-CAPSULE-01/03/04: capsule tab/pane dispatch through owned components.
//!
//! The capsule tab strip and split panes render through reusable Termrock
//! components (`Tabs`, `SplitPane`, `Panel`, `Empty`, `TextViewport`,
//! `TextInput`). These tests pin that dispatch:
//!
//! * the component path reproduces the frozen audit rows exactly (symbols);
//! * the paused 120x40 audit state renders through the same component
//!   path (WI-JACKIN-CAPSULE-03 deleted the historical frame): all 40
//!   rows match frozen, tab selection repaints the strip, and the
//!   scrolled panes wear the owned viewport fade the painter lacked;
//! * the paused zoomed state renders through the same component path
//!   (WI-JACKIN-CAPSULE-04 deleted the zoom frame and gate): all 40 rows
//!   match frozen, the `zoomed` badge is stock `Panel` meta, tab renames
//!   repaint the zoomed strip, and unzooming restores the split;
//! * typing into the capsule input reaches the live component path and
//!   clearing it restores the component body (dispatch boundary).
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

/// Component path parity: at full motion (no ticks) every row matches the
/// frozen audit text exactly.
#[test]
fn capsule_component_path_matches_audit_body() {
    for (w, h) in [(120u16, 40u16), (80u16, 24u16)] {
        let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
        let harness = Harness::new(app, termrock::Theme::junie(), w, h);
        let actual = rows(&harness, h);
        let expected = frozen(&format!(
            "baselines/tuiscotti-v1/jackin/capsule/audit/{w}x{h}/truecolor.txt"
        ));
        assert_eq!(expected.len(), usize::from(h), "frozen frame has {h} rows");
        for (y, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
            assert_eq!(e, a, "{w}x{h} component-path row {y} must match frozen");
        }
    }
}

/// The component path owns the status row outside the audit state: it
/// renders the session meter from the focused account's usage windows.
/// The low-priority tab/pane counts only survive where nothing else needs
/// the room, so the audit geometries never show them.
#[test]
fn capsule_component_status_shows_session_meter() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Full, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let status = harness.row(38);
    assert!(
        status.contains("Session") && status.contains("76%"),
        "component status must show the session meter, got {status:?}"
    );
    assert!(
        !status.contains("tabs ·"),
        "tab/pane counts must not crowd out the meter, got {status:?}"
    );
}

/// Gate deletion: the exact frozen audit state renders through the owned
/// component path — all 40 rows keep matching frozen with no historical
/// frame left to serve them.
#[test]
fn audited_paused_capsule_renders_through_components() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let actual = rows(&harness, 40);
    let expected = frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt");
    assert_eq!(expected.len(), 40, "frozen frame has 40 rows");
    for (y, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(e, a, "audited paused row {y} must match frozen");
    }
}

/// Body ownership: renaming a tab repaints the paused audit strip
/// through the component path — the content cannot be a stored answer.
#[test]
fn paused_audit_tab_rename_repaints_the_strip() {
    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let expected = frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt");
    assert_eq!(
        harness.row(2).trim_end(),
        expected[2],
        "control strip must match frozen"
    );

    let mut app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let daemon = app
        .world
        .daemons
        .get_mut("jk-7f3a")
        .expect("capsule fixture daemon exists");
    daemon.tabs[0].custom_label = Some("renamed".to_string());
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let strip = harness.row(2);
    assert!(
        strip.contains("renamed"),
        "renamed tab must repaint the paused strip, got {strip:?}"
    );
    assert_eq!(
        harness.row(0).trim_end(),
        expected[0],
        "shell row must not move with tab data"
    );
}

/// Fade ownership: the paused audit panes wear the owned viewport
/// scroll-edge fade. The deleted painter predated the fade and painted
/// these cells full-bright (fg 179/255 vs 98/102/140 below), so these
/// pins fail on the historical frame and pass on the components.
#[test]
fn paused_audit_body_shows_owned_viewport_fade() {
    use termrock::Color;

    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    for (x, y, symbol, fg) in [
        (61u16, 5u16, "•", Color::Rgb(98, 98, 98)),
        (69, 5, "c", Color::Rgb(140, 140, 140)),
        (61, 6, " ", Color::Rgb(102, 102, 102)),
        (61, 22, "b", Color::Rgb(140, 140, 140)),
        (61, 23, "=", Color::Rgb(102, 102, 102)),
    ] {
        let cell = harness.cell(x, y);
        assert_eq!(cell.symbol(), symbol, "faded cell ({x},{y}) symbol");
        assert_eq!(cell.fg, fg, "faded cell ({x},{y}) must wear the fade");
        assert_eq!(cell.bg, Color::Rgb(0, 0, 0), "faded cell ({x},{y}) bg");
    }
}

/// Component-path journey: keys reach the live capsule input, the tab
/// strip stays intact through the journey, and clearing the input
/// restores the component body.
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
        harness.row(38).contains("Session") && harness.row(38).contains("76%"),
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

/// Zoom ownership (WI-JACKIN-CAPSULE-04): the paused zoomed capsule
/// renders through the owned `Panel`/`TextViewport` composition. The real
/// prefix journey reproduces the frozen zoom frame exactly; renaming a tab
/// repaints the zoomed strip (a stored frame cannot); the `zoomed` badge
/// wears the focused border color; unzooming restores the split layout.
#[test]
fn paused_zoom_renders_through_components() {
    use termrock::{Color, KeyCode};

    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let mut harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let _ = harness.ctrl('b');
    let _ = harness.key(KeyCode::Char('z'));
    let actual = rows(&harness, 40);
    let expected = frozen("baselines/tuiscotti-v1/jackin/capsule/zoom/120x40/truecolor.txt");
    assert_eq!(expected.len(), 40, "frozen zoom frame has 40 rows");
    for (y, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(e, a, "zoomed paused row {y} must match frozen");
    }
    for (x, symbol) in [
        (110u16, " "),
        (111, "z"),
        (112, "o"),
        (113, "o"),
        (114, "m"),
        (115, "e"),
        (116, "d"),
        (117, " "),
    ] {
        let cell = harness.cell(x, 4);
        assert_eq!(cell.symbol(), symbol, "zoomed badge cell ({x},4) symbol");
        assert_eq!(
            cell.fg,
            Color::Rgb(77, 77, 77),
            "zoomed badge cell ({x},4) must wear the border color"
        );
        assert_eq!(cell.bg, Color::Rgb(0, 0, 0), "zoomed badge cell ({x},4) bg");
    }

    let mut app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let daemon = app
        .world
        .daemons
        .get_mut("jk-7f3a")
        .expect("capsule fixture daemon exists");
    daemon.tabs[0].custom_label = Some("renamed".to_string());
    let mut harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let _ = harness.ctrl('b');
    let _ = harness.key(KeyCode::Char('z'));
    assert!(
        harness.row(2).contains("renamed"),
        "renamed tab must repaint the zoomed strip, got {:?}",
        harness.row(2)
    );
    let _ = harness.ctrl('b');
    let _ = harness.key(KeyCode::Char('z'));
    assert!(
        harness.row(4).contains("Codex (Primary)"),
        "unzoom must restore the split layout, got {:?}",
        harness.row(4)
    );
    assert!(
        !harness.row(4).contains("zoomed"),
        "unzoom must drop the badge, got {:?}",
        harness.row(4)
    );
}

/// Paused input parity: with the historical frame gone, paused mode no
/// longer bypasses the component path — typing reaches the live input
/// and clearing it restores the frozen-exact audit frame.
#[test]
fn paused_capsule_input_reaches_the_component_path() {
    use termrock::KeyCode;

    let app = App::for_scenario_at(Scenario::CapsuleMulti, Motion::Paused, 40);
    let mut harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let expected = frozen("baselines/tuiscotti-v1/jackin/capsule/audit/120x40/truecolor.txt");
    let _ = harness.key(KeyCode::Enter);
    let _ = harness.type_str("hello");
    assert!(
        (0..40).any(|y| harness.row(y).contains("hello")),
        "typed text must be visible in the paused capsule input"
    );
    for _ in 0..5 {
        let _ = harness.key(KeyCode::Backspace);
    }
    for y in 0..40u16 {
        assert_eq!(
            harness.row(y).trim_end(),
            expected[usize::from(y)],
            "clearing paused input must restore frozen row {y}"
        );
    }
}
