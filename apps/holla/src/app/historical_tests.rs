//! Pinned794b095 actual-App assertions restored through the shared production runtime.
//! Fixture selection, frames, dimensions and product assertions remain source-derived.
use super::{App, Motion, Scenario};
use junie_tui::{FeedbackClock, KeyCode, SimulationMoment, Theme};
use junie_tui_testing::Harness;
fn fixture(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    width: u16,
    height: u16,
) -> Harness<App> {
    let app = App::for_scenario(scenario, motion, frame);
    let frame = u64::try_from(app.fixture_time_ms());
    assert!(frame.is_ok(), "fixture time must be nonnegative");
    let initial = SimulationMoment::from_millis(frame.unwrap_or_default());
    Harness::new_with_feedback_clock(
        app,
        Theme::junie(),
        width,
        height,
        FeedbackClock::Simulation { initial },
    )
}
#[test]
fn paused_frames_are_byte_identical() {
    let a = fixture(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    let b = fixture(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    assert_eq!(a.text(), b.text());
}

#[test]
fn paused_ticks_do_not_advance_the_world() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let before = h.text();
    for _ in 0..20 {
        let _ = h.advance(std::time::Duration::from_millis(200));
    }
    assert_eq!(before, h.text());
    // discovery is a virtual-clock event: paused means it never fires
    assert!(h.app().world.discovering());
}

#[test]
fn remote_production_host_is_unmistakable() {
    let h = fixture(Scenario::RemoteHost, Motion::Paused, 0, 120, 40);
    let t = h.text();
    assert!(t.contains("◆ prod-eu-1 · ssh · production"), "{t}");
    assert!(t.contains("/srv/payments"), "{t}");
}

#[test]
fn too_small_notice_below_minimum() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.resize(71, 19);
    let t = h.text();
    assert!(t.contains("Terminal too small"), "{t}");
    assert!(t.contains("Need 72×20, have 71×19"), "{t}");
    assert!(t.contains("holla❯"), "{t}");
    // q quits from the notice
    let _ = h.key(KeyCode::Char('q'));
    assert!(h.app().quit);
}

#[test]
fn hard_cases_shows_docker_failure_and_detached_head() {
    let h = fixture(Scenario::HardCases, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    assert!(t.contains("docker: discovery failed"), "{t}");
    assert!(t.contains("detached at 9f3a21c"), "{t}");
}

#[test]
fn monorepo_child_shows_parent_ecosystem_with_scope_tags() {
    let h = fixture(Scenario::MonorepoChild, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    // local frontend task ranks; parent workspace rows are tagged
    assert!(t.contains("Run dev"), "{t}");
    assert!(t.contains("workspace"), "{t}");
}

#[test]
fn chrome_has_brand_menu_crumb_host_and_hint_bar() {
    let h = fixture(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    let t = h.text();
    assert!(t.contains("holla❯"), "{t}");
    assert!(t.contains("File"), "{t}");
    assert!(t.contains("~/work/pave"), "{t}");
    assert!(t.contains("devbox · local"), "{t}");
    // the hint bar owns the last row, once
    let last = t.lines().nth(39).unwrap_or("");
    assert!(last.contains("Type") && last.contains("Filter"), "{last:?}");
    assert_eq!(t.matches("Run top match").count(), 1, "{t}");
    // the query is armed: EDIT badge and placeholder
    assert!(last.contains("EDIT"), "{last:?}");
    assert!(t.contains("Type to filter"), "{t}");
}

#[test]
fn menu_bar_opens_and_quit_is_confirmable() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.key(KeyCode::F(10));
    let t = h.text();
    assert!(t.contains("Quit"), "{t}");
    assert!(t.contains("Ctrl+Q"), "{t}");
    // the menu layer owns the hint bar while open
    let last = t.lines().nth(39).unwrap_or("");
    assert!(last.contains("Menu"), "{last:?}");
    let _ = h.key(KeyCode::Esc);
    // quit goes through a confirmation whose default is Cancel
    let _ = h.ctrl('q');
    let t = h.text();
    assert!(t.contains("Quit holla?"), "{t}");
    assert!(h.text().contains("Cancel"), "{t}");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.app().quit);
    assert!(!h.text().contains("Quit holla?"));
}

#[test]
fn help_opens_from_empty_query_question_mark() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.key(KeyCode::Char('?'));
    let t = h.text();
    assert!(t.contains("Key reference"), "{t}");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("Key reference"));
}

// -------------------------------------------------------------- query

#[test]
fn empty_query_q_asks_to_quit_but_typed_q_filters() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.key(KeyCode::Char('q'));
    assert!(h.text().contains("Quit holla?"));
    let _ = h.key(KeyCode::Esc);
    // with text in the query, q is just a character
    let _ = h.type_str("se");
    let before = h.app().home.query().to_owned();
    let _ = h.key(KeyCode::Char('q'));
    assert_eq!(h.app().home.query(), format!("{before}q"));
    assert!(!h.text().contains("Quit holla?"));
}

#[test]
fn typing_filters_and_esc_clears() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("git");
    let t = h.text();
    assert!(t.contains("git"), "{t}");
    assert!(t.contains("Check git status"), "{t}");
    assert!(!t.contains("Connect to"), "{t}");
    let _ = h.key(KeyCode::Esc);
    assert!(h.text().contains("Connect to"), "{}", h.text());
}

#[test]
fn scope_cycle_narrows_and_esc_widens() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.ctrl('s');
    let t = h.text();
    assert!(t.contains("scope: here"), "{t}");
    assert!(!t.contains("Connect to prod-eu-1"), "{t}");
    let _ = h.ctrl('s');
    let _ = h.ctrl('s');
    let _ = h.ctrl('s');
    assert!(h.text().contains("scope: host"), "{}", h.text());
    assert!(h.text().contains("Connect to prod-eu-1"), "{}", h.text());
    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("scope: host"), "{}", h.text());
}

#[test]
fn down_moves_into_results_and_back() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("monit");
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    // §8.6: Enter opens the snapshot; the btm handoff is one confirm away
    let t = h.text();
    assert!(t.contains("System snapshot"), "{t}");
    assert!(t.contains("Handoff"), "{t}");
    let _ = h.key(KeyCode::Right); // Close → Open btm (simulated)
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("simulated handoff"), "{}", h.text());
}

#[test]
fn enter_on_top_match_reports_simulated_run() {
    // rust-dirty's pinned `make test` outranks everything at this path
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Would run: make test"), "{t}");
    assert!(t.contains("nothing executed"), "{t}");
}
