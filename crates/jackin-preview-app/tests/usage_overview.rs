//! Usage overview is the live framed panel, not the stored 120×40 painter.
use jackin_preview_app::{App, Motion, Scenario};
use termrock::KeyCode;
use termrock_test_support::Harness;

fn frozen(path: &str) -> Vec<String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../")
        .join(path);
    std::fs::read_to_string(&root)
        .unwrap()
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect()
}

fn usage_frame(width: u16, height: u16) -> Harness<App> {
    let app = App::for_scenario_at(Scenario::AccountsMixed, Motion::Paused, 40);
    let mut harness = Harness::new(app, termrock::Theme::junie(), width, height);
    let _ = harness.key(KeyCode::Char('u'));
    harness
}

#[test]
fn usage_overview_matches_baseline_text_at_every_canonical_size() {
    for (width, height) in [(72u16, 20u16), (80, 24), (100, 30), (120, 40), (160, 50)] {
        let harness = usage_frame(width, height);
        let expected = frozen(&format!(
            "baselines/tuiscotti-v1/jackin/usage/overview/{width}x{height}/truecolor.txt"
        ));
        assert!(
            harness.row(2).contains("Usage · read-only"),
            "{width}x{height} panel title must come from the live frame, got {:?}",
            harness.row(2)
        );
        for (y, expected_row) in expected.iter().enumerate() {
            assert_eq!(
                harness.row(y as u16).trim_end(),
                expected_row,
                "{width}x{height} row {y}"
            );
        }
    }
}

#[test]
fn usage_overview_list_follows_account_names() {
    let mut app = App::for_scenario_at(Scenario::AccountsMixed, Motion::Paused, 40);
    let account = app
        .world
        .accounts
        .accounts
        .iter_mut()
        .find(|account| account.display_name == "Personal")
        .expect("personal account");
    account.display_name = "Renamed personal".into();
    let mut harness = Harness::new(app, termrock::Theme::junie(), 120, 40);
    let _ = harness.key(KeyCode::Char('u'));
    assert!(
        harness.text().contains("Renamed personal"),
        "the list must read account data, not a stored frame"
    );
    assert!(harness.row(2).contains("Usage · read-only"));
}
