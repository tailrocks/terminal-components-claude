//! Explicit clock steps under paused motion (V12): the layers that open
//! only from a job on virtual time are reachable in a paused capture, and
//! the frame is still once the step is done. Driven through the real input
//! path (`Input::Paste` of a `ClockStep` request, as the visual harness's
//! `advance:<ms>` step sends it).

use ratatui::crossterm::event::KeyCode;

use junie_tui::clock::ClockStep;

use crate::app::Route;
use crate::app_tests::H;
use crate::scenario::{Motion, Scenario};

fn paused(scenario: Scenario) -> H {
    let mut h = H::new(scenario, Motion::Paused, 40, 120, 40);
    // the harness's opt-in (`JUNIE_TUI_TEST_CLOCK=1`)
    h.app.shell.clock_steps = true;
    assert_eq!(h.app.route, Route::Manager, "{}", h.text());
    h
}

fn step(h: &mut H, ms: u64) {
    h.paste(&ClockStep::request(ms));
}

/// Ticks under paused motion change nothing, so the frame settles.
fn assert_still(h: &mut H) {
    let before = h.text();
    let now = h.app.world.now_ms();
    h.ticks(6);
    assert_eq!(h.app.world.now_ms(), now, "the paused clock stays stopped");
    assert_eq!(h.text(), before, "the frame is still after the step");
}

#[test]
fn a_clock_step_completes_the_editors_failing_save() {
    let mut h = paused(Scenario::HardCases);
    h.key(KeyCode::Char('e'));
    assert_eq!(h.app.route, Route::Editor, "{}", h.text());
    h.key(KeyCode::Char(']'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('r'));
    h.ctrl('s');
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    // paused: the 900 ms save job never fires on its own
    h.ticks(20);
    assert!(!h.text().contains("Save failed"), "{}", h.text());
    let pasted = h.app.screens.editor.as_ref().unwrap().change_count();
    step(&mut h, 1_000);
    assert!(h.text().contains("Save failed"), "{}", h.text());
    assert_eq!(
        h.app.screens.editor.as_ref().unwrap().change_count(),
        pasted,
        "the step request is not pasted into a field"
    );
    assert_still(&mut h);
}

#[test]
fn a_clock_step_completes_the_global_settings_failing_save() {
    let mut h = paused(Scenario::HardCases);
    h.key(KeyCode::Char('s'));
    assert_eq!(h.app.route, Route::Settings, "{}", h.text());
    h.key(KeyCode::Char('5'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.ctrl('s');
    assert!(h.text().contains("Save settings"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert!(!h.text().contains("Settings error"), "{}", h.text());
    step(&mut h, 1_000);
    assert!(h.text().contains("Settings error"), "{}", h.text());
    assert!(h.text().contains("nothing was written"), "{}", h.text());
    assert_still(&mut h);
}

#[test]
fn a_clock_step_completes_a_trusted_roles_failing_load() {
    let mut h = paused(Scenario::HardCases);
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('3'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("Load role"), "{}", h.text());
    h.type_str("writer");
    h.key(KeyCode::Enter);
    h.ticks(20);
    assert!(!h.text().contains("failed to load"), "{}", h.text());
    step(&mut h, 1_200);
    let t = h.text();
    assert!(t.contains("Role writer failed to load"), "{t}");
    assert!(t.contains("trust stays granted"), "{t}");
    assert_still(&mut h);
}

#[test]
fn a_clock_step_runs_the_launch_pipeline_to_the_credential_error() {
    let mut h = paused(Scenario::HardCases);
    assert_eq!(
        h.app.world.op.session,
        crate::sim::onepassword::OpSession::Locked
    );
    // payments-platform › the-architect, Codex · Primary (1Password)
    h.key(KeyCode::Enter);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Cockpit, "{}", h.text());
    h.ticks(20);
    assert!(
        !h.text().contains("Credential source unavailable"),
        "{}",
        h.text()
    );
    step(&mut h, 2_000);
    assert!(h.app.screens.cockpit.as_ref().unwrap().credential_hold);
    assert!(
        h.text().contains("Credential source unavailable"),
        "{}",
        h.text()
    );
    assert_still(&mut h);
}

#[test]
fn a_clock_step_request_is_pasted_as_text_unless_motion_is_paused() {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 120, 40);
    h.app.shell.clock_steps = true;
    let now = h.app.world.now_ms();
    step(&mut h, 60_000);
    assert!(
        h.app.world.now_ms() < now + 60_000,
        "only a paused clock steps"
    );
}

#[test]
fn without_the_test_opt_in_a_paused_app_pastes_the_request_unchanged() {
    let mut h = H::new(Scenario::HardCases, Motion::Paused, 40, 120, 40);
    h.app.shell.clock_steps = false;
    h.key(KeyCode::Char('e'));
    h.key(KeyCode::Char('3'));
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('a'));
    assert!(h.text().contains("Load role"), "{}", h.text());
    let now = h.app.world.now_ms();
    step(&mut h, 1_200);
    assert_eq!(h.app.world.now_ms(), now, "the clock did not step");
    // the picker's filter received the text as typed
    assert!(h.text().contains("junie-tui:advance"), "{}", h.text());
}
