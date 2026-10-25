#![cfg(test)]

use super::{App, Motion};
use crate::domain::mise::Trust;
use crate::domain::ranking::{Pin, Usage};
use termrock::{FeedbackClock, KeyCode, SimulationMoment, Theme};
use termrock_test_support::Harness;

const FIXTURE_FRAME_MS: u64 = 4_000;
const FILTER_ID: termrock::Id = termrock::Id::root("holla.task-args.filter");
const NEXTTEST_COMMAND: &str = "mise run test-nextest";

#[derive(Clone, Copy)]
enum RememberedKind {
    Recent,
    Pin,
}

fn fixture(width: u16, height: u16, trusted: bool) -> Harness<App> {
    fixture_with_remembered(width, height, trusted, None)
}

fn fixture_with_remembered(
    width: u16,
    height: u16,
    trusted: bool,
    remembered: Option<RememberedKind>,
) -> Harness<App> {
    let source_theme = Theme::junie();
    let mut app = App::for_nextest_args_fixture(Motion::Paused, FIXTURE_FRAME_MS)
        .with_source_theme(source_theme.clone());
    if trusted
        && let Some(mise) = &mut app.world.mise
        && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
    {
        task.trust = Trust::Trusted;
    }
    if let Some(remembered) = remembered {
        let path = app.world.cwd.clone();
        match remembered {
            RememberedKind::Recent => app.world.memory.usage.push(Usage {
                path,
                command: NEXTTEST_COMMAND.to_owned(),
                count: 1,
            }),
            RememberedKind::Pin => {
                app.world.memory.pins.clear();
                app.world.memory.pins.push(Pin {
                    path,
                    command: NEXTTEST_COMMAND.to_owned(),
                });
            }
        }
    }
    assert_eq!(app.fixture_time_ms(), 4_000_i64);
    let initial = SimulationMoment::from_millis(FIXTURE_FRAME_MS);
    Harness::new_with_feedback_clock(
        app,
        source_theme,
        width,
        height,
        FeedbackClock::Simulation { initial },
    )
}

fn remembered_query(kind: RememberedKind) -> &'static str {
    match kind {
        RememberedKind::Recent => "used 1 time",
        RememberedKind::Pin => "pinned here",
    }
}

fn remembered_id(kind: RememberedKind) -> String {
    let prefix = match kind {
        RememberedKind::Recent => "recent",
        RememberedKind::Pin => "pin",
    };
    format!("{prefix}:{NEXTTEST_COMMAND}")
}

fn enter_remembered(harness: &mut Harness<App>, kind: RememberedKind) -> Option<termrock::Id> {
    let _ = harness.type_str(remembered_query(kind));
    let selected = harness
        .app()
        .home
        .selected(&harness.app().world)
        .expect("query selects the remembered nextest row");
    assert_eq!(selected.id, remembered_id(kind));
    assert_eq!(selected.intent_id(), Some("task:test-nextest"));
    let prior_focus = harness.focus();
    let _ = harness.key(KeyCode::Enter);
    prior_focus
}

fn enter_nextest(harness: &mut Harness<App>) -> Option<termrock::Id> {
    let _ = harness.type_str("test-nextest");
    let prior_focus = harness.focus();
    let _ = harness.key(KeyCode::Enter);
    prior_focus
}

fn approve_trust(harness: &mut Harness<App>) {
    let _ = harness.key(KeyCode::Right);
    let _ = harness.key(KeyCode::Enter);
}

fn task_trust(harness: &Harness<App>) -> Option<Trust> {
    harness
        .app()
        .world
        .mise
        .as_ref()?
        .tasks
        .iter()
        .find(|task| task.id == "test-nextest")
        .map(|task| task.trust)
}

#[test]
fn native_recent_selection_enters_trust_then_typed_args_and_simulates_once() {
    let mut harness = fixture_with_remembered(120, 40, false, Some(RememberedKind::Recent));
    let before_cwd = harness.app().world.cwd.clone();
    let before_host = harness.app().world.host.clone();
    let before_revision = harness.app().world.effect_revision;
    let before_activity_count = harness.app().world.activities.len();

    let _ = enter_remembered(&mut harness, RememberedKind::Recent);
    if harness.text().contains("Trust this task file?") {
        approve_trust(&mut harness);
    }
    assert!(harness.text().contains("Task inputs"), "{}", harness.text());
    assert_eq!(task_trust(&harness), Some(Trust::Trusted));
    assert_eq!(harness.focus(), Some(FILTER_ID));
    let _ = harness.paste("test(tests::parser)");
    let _ = harness.ctrl('s');
    let review = harness.text();
    assert_eq!(review.matches("Review task inputs").count(), 1, "{review}");
    assert!(review.contains("program 'mise'"), "{review}");
    assert!(review.contains("argv[0] 'run'"), "{review}");
    assert!(review.contains("argv[1] 'test-nextest'"), "{review}");
    assert!(
        review.contains("argv[2] '--filterset=test(tests::parser)'"),
        "{review}"
    );
    assert!(review.contains("argv[3] '-P'"), "{review}");
    assert!(review.contains("argv[4] 'default'"), "{review}");

    let _ = harness.key(KeyCode::Enter);
    let status = harness.app().status.as_deref().unwrap_or("");
    assert!(
        status.contains("Simulation confirmed · nothing executed"),
        "{status:?}"
    );
    assert!(
        status.contains("argv[2] '--filterset=test(tests::parser)'"),
        "{status:?}"
    );
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().world.host, before_host);
    assert_eq!(harness.app().world.effect_revision, before_revision);
    assert_eq!(harness.app().world.activities.len(), before_activity_count);
}

#[test]
fn trusted_recent_and_pin_selection_review_one_structured_request() {
    for kind in [RememberedKind::Recent, RememberedKind::Pin] {
        let mut harness = fixture_with_remembered(120, 40, true, Some(kind));
        let before_cwd = harness.app().world.cwd.clone();
        let before_host = harness.app().world.host.clone();
        let before_revision = harness.app().world.effect_revision;
        let before_activity_count = harness.app().world.activities.len();

        let _ = enter_remembered(&mut harness, kind);
        assert!(harness.text().contains("Task inputs"), "{}", harness.text());
        assert!(!harness.text().contains("Trust this task file?"));
        assert_eq!(harness.focus(), Some(FILTER_ID));
        let _ = harness.paste("suite; $(touch x)");
        let _ = harness.key(KeyCode::Tab);
        let profile = termrock::Id::root("holla.task-args.profile");
        assert_eq!(harness.focus(), Some(profile), "{}", harness.text());
        let _ = harness.key(KeyCode::Enter);
        let _ = harness.key(KeyCode::Down);
        let _ = harness.key(KeyCode::Enter);
        assert!(!harness.is_open(profile), "choice closes the native Select");

        let _ = harness.ctrl('s');
        let review = harness.text();
        assert_eq!(review.matches("Review task inputs").count(), 1, "{review}");
        assert!(review.contains("program 'mise'"), "{review}");
        assert!(review.contains("argv[0] 'run'"), "{review}");
        assert!(review.contains("argv[1] 'test-nextest'"), "{review}");
        assert!(
            review.contains("argv[2] '--filterset=suite; $(touch x)'"),
            "{review}"
        );
        assert!(review.contains("argv[3] '-P'"), "{review}");
        assert!(review.contains("argv[4] 'ci'"), "{review}");
        assert!(review.contains("target '~/work/pave'"), "{review}");
        assert!(review.contains("workdir '~/work/pave'"), "{review}");
        assert!(review.contains("Simulate"), "{review}");

        let _ = harness.key(KeyCode::Enter);
        let status = harness.app().status.as_deref().unwrap_or("");
        assert!(
            status.contains("Simulation confirmed · nothing executed"),
            "{status:?}"
        );
        assert!(
            status.contains("argv[2] '--filterset=suite; $(touch x)'"),
            "{status:?}"
        );
        assert_eq!(harness.app().world.cwd, before_cwd);
        assert_eq!(harness.app().world.host, before_host);
        assert_eq!(harness.app().world.effect_revision, before_revision);
        assert_eq!(harness.app().world.activities.len(), before_activity_count);
    }
}

#[test]
fn remembered_alias_cannot_reach_fixed_preview_or_generic_confirmation() {
    let mut shortcut = fixture_with_remembered(120, 40, true, Some(RememberedKind::Pin));
    let _ = shortcut.type_str(remembered_query(RememberedKind::Pin));
    let selected = shortcut
        .app()
        .home
        .selected(&shortcut.app().world)
        .expect("query selects the pin row");
    assert_eq!(selected.id, remembered_id(RememberedKind::Pin));
    assert_eq!(selected.intent_id(), Some("task:test-nextest"));
    let _ = shortcut.ctrl('p');
    assert!(
        shortcut.text().contains("Task inputs"),
        "{}",
        shortcut.text()
    );
    assert!(!shortcut.text().contains("Would happen"));
    assert!(!shortcut.text().contains("Would run:"));

    let mut actions = fixture_with_remembered(120, 40, true, Some(RememberedKind::Recent));
    let _ = actions.type_str(remembered_query(RememberedKind::Recent));
    let selected = actions
        .app()
        .home
        .selected(&actions.app().world)
        .expect("query selects the recent row");
    assert_eq!(selected.id, remembered_id(RememberedKind::Recent));
    assert_eq!(selected.intent_id(), Some("task:test-nextest"));
    let _ = actions.ctrl('o');
    assert!(actions.text().contains("Preview"), "{}", actions.text());
    let _ = actions.key(KeyCode::Down);
    let _ = actions.key(KeyCode::Enter);
    assert!(actions.text().contains("Task inputs"), "{}", actions.text());
    assert!(!actions.text().contains("Would happen"));
    assert!(!actions.text().contains("Would run:"));
    assert!(actions.app().world.activities.is_empty());

    let mut generic = fixture_with_remembered(120, 40, true, Some(RememberedKind::Pin));
    let _ = generic.type_str(remembered_query(RememberedKind::Pin));
    let row = generic
        .app()
        .home
        .selected(&generic.app().world)
        .expect("query selects the pin row");
    assert_eq!(row.id, remembered_id(RememberedKind::Pin));
    assert_eq!(row.intent_id(), Some("task:test-nextest"));
    let destination = crate::dispatch::confirm_preview(&mut generic.app_mut().world, &row);
    assert!(matches!(
        destination,
        crate::dispatch::Destination::TaskArgs(_)
    ));
    assert!(generic.app().world.activities.is_empty());

    let unknown = crate::domain::action::Action::unavailable_memory(
        NEXTTEST_COMMAND,
        &generic.app().world.cwd,
    );
    let destination = crate::dispatch::invoke(&mut generic.app_mut().world, &unknown);
    assert!(matches!(
        destination,
        crate::dispatch::Destination::Notice(_)
    ));
    assert!(generic.app().world.activities.is_empty());
}

#[test]
fn remembered_alias_cancel_at_trust_args_and_review_restores_home_focus() {
    let mut trust = fixture_with_remembered(120, 40, false, Some(RememberedKind::Recent));
    let prior_focus = enter_remembered(&mut trust, RememberedKind::Recent);
    assert!(trust.text().contains("Trust this task file?"));
    let _ = trust.key(KeyCode::Esc);
    assert_eq!(trust.focus(), prior_focus);
    assert_eq!(task_trust(&trust), Some(Trust::Untrusted));
    assert!(trust.app().world.activities.is_empty());
    assert!(trust.app().status.is_none());

    let mut args = fixture_with_remembered(120, 40, false, Some(RememberedKind::Pin));
    let prior_focus = enter_remembered(&mut args, RememberedKind::Pin);
    approve_trust(&mut args);
    assert!(args.text().contains("Task inputs"));
    let _ = args.key(KeyCode::Esc);
    assert_eq!(args.focus(), Some(FILTER_ID));
    let _ = args.key(KeyCode::Esc);
    assert_eq!(args.focus(), prior_focus);
    assert_eq!(task_trust(&args), Some(Trust::Trusted));
    assert!(args.app().world.activities.is_empty());
    assert_eq!(args.app().world.effect_revision, 0);

    let mut review = fixture_with_remembered(120, 40, true, Some(RememberedKind::Pin));
    let prior_focus = enter_remembered(&mut review, RememberedKind::Pin);
    let _ = review.ctrl('s');
    assert!(review.text().contains("Review task inputs"));
    let _ = review.key(KeyCode::Esc);
    assert_eq!(review.focus(), prior_focus);
    assert!(review.app().status.is_none());
    assert!(review.app().world.activities.is_empty());
    assert_eq!(review.app().world.effect_revision, 0);
}

#[test]
fn remembered_alias_context_change_at_review_discards_the_accepted_plan() {
    let mut harness = fixture_with_remembered(120, 40, true, Some(RememberedKind::Pin));
    let _ = enter_remembered(&mut harness, RememberedKind::Pin);
    let _ = harness.ctrl('s');
    assert!(harness.text().contains("Review task inputs"));
    harness.app_mut().world.cwd.push_str("/changed");
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .app()
            .status
            .as_deref()
            .is_some_and(|message| message.contains("context changed"))
    );
    assert!(harness.app().world.activities.is_empty());
    assert_eq!(harness.app().world.effect_revision, 0);
}

#[test]
fn native_trust_args_review_and_simulation_use_one_exact_request() {
    let mut harness = fixture(120, 40, false);
    let before_cwd = harness.app().world.cwd.clone();
    let before_revision = harness.app().world.effect_revision;
    let before_activity_count = harness.app().world.activities.len();

    let _ = enter_nextest(&mut harness);
    assert!(
        harness.text().contains("Trust this task file?"),
        "{}",
        harness.text()
    );
    approve_trust(&mut harness);
    assert!(harness.text().contains("Task inputs"), "{}", harness.text());
    assert_eq!(harness.focus(), Some(FILTER_ID));
    assert!(
        harness.text().contains("Ctrl+S Review inputs"),
        "{}",
        harness.text()
    );
    assert!(
        !harness.text().contains("Ctrl+S Scope"),
        "{}",
        harness.text()
    );

    let _ = harness.paste("test(tests::parser)");
    let _ = harness.key(KeyCode::Tab);
    let profile = termrock::Id::root("holla.task-args.profile");
    assert_eq!(harness.focus(), Some(profile), "{}", harness.text());
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.is_open(profile), "{}", harness.text());
    let _ = harness.key(KeyCode::Down);
    let _ = harness.key(KeyCode::Enter);
    assert!(!harness.is_open(profile), "choice closes the native Select");
    assert_eq!(harness.focus(), Some(profile), "{}", harness.text());
    let _ = harness.ctrl('s');
    let review = harness.text();
    assert!(review.contains("Review task inputs"), "{review}");
    assert!(review.contains("test(tests::parser)"), "{review}");
    assert!(
        review.contains("--filterset=test(tests::parser)"),
        "{review}"
    );
    assert!(review.contains("'ci'"), "{review}");
    assert!(review.contains("target '~/work/pave'"), "{review}");
    assert!(review.contains("workdir '~/work/pave'"), "{review}");
    assert!(review.contains("Simulate"), "{review}");

    let _ = harness.key(KeyCode::Enter);
    let status = harness.app().status.as_deref().unwrap_or("");
    assert!(
        status.contains("Simulation confirmed · nothing executed"),
        "{status:?}"
    );
    assert!(
        status.contains("'--filterset=test(tests::parser)'"),
        "{status:?}"
    );
    assert!(status.contains("argv[3] '-P'"), "{status:?}");
    assert!(status.contains("argv[4] 'ci'"), "{status:?}");
    assert_eq!(task_trust(&harness), Some(Trust::Trusted));
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().world.effect_revision, before_revision);
    assert_eq!(harness.app().world.activities.len(), before_activity_count);
}

#[test]
fn already_trusted_path_opens_args_without_a_second_trust_gate() {
    let mut harness = fixture(120, 40, true);
    let _ = enter_nextest(&mut harness);
    assert!(harness.text().contains("Task inputs"), "{}", harness.text());
    assert!(
        !harness.text().contains("Trust this task file?"),
        "{}",
        harness.text()
    );

    assert_eq!(harness.focus(), Some(FILTER_ID));
    assert!(
        harness.text().contains("Ctrl+S Review inputs"),
        "{}",
        harness.text()
    );
    assert!(
        !harness.text().contains("Ctrl+S Scope"),
        "{}",
        harness.text()
    );
    let _ = harness.ctrl('s');
    let review = harness.text();
    assert!(review.contains("Review task inputs"), "{review}");
    assert!(review.contains("'default'"), "{review}");
    assert!(!review.contains("--filterset="), "{review}");
    let _ = harness.key(KeyCode::Esc);
}

#[test]
fn clearing_optional_filter_keeps_the_default_profile_and_omits_filter_argv() {
    let mut harness = fixture(120, 40, true);
    let _ = enter_nextest(&mut harness);
    assert!(harness.text().contains("Task inputs"), "{}", harness.text());
    assert_eq!(harness.focus(), Some(FILTER_ID));
    let _ = harness.paste("test(tests::parser)");
    let _ = harness.ctrl('l');
    let _ = harness.key(KeyCode::Backspace);
    assert_eq!(harness.focus(), Some(FILTER_ID));
    let _ = harness.ctrl('s');

    let review = harness.text();
    assert!(review.contains("Review task inputs"), "{review}");
    assert!(!review.contains("--filterset="), "{review}");
    assert!(review.contains("'default'"), "{review}");
}

#[test]
fn preview_shortcuts_cannot_bypass_the_declared_input_producer() {
    let mut shortcut = fixture(120, 40, true);
    let _ = shortcut.type_str("test-nextest");
    let _ = shortcut.ctrl('p');
    assert!(
        shortcut.text().contains("Task inputs"),
        "{}",
        shortcut.text()
    );
    assert!(
        !shortcut.text().contains("Would happen"),
        "{}",
        shortcut.text()
    );

    let mut actions = fixture(120, 40, true);
    let _ = actions.type_str("test-nextest");
    let _ = actions.ctrl('o');
    assert!(actions.text().contains("Preview"), "{}", actions.text());
    let _ = actions.key(KeyCode::Down);
    let _ = actions.key(KeyCode::Enter);
    assert!(actions.text().contains("Task inputs"), "{}", actions.text());
    assert!(
        !actions.text().contains("Would happen"),
        "{}",
        actions.text()
    );
    assert!(actions.app().world.activities.is_empty());
}

#[test]
fn cancel_at_trust_args_and_review_restores_focus_without_simulation() {
    let mut trust = fixture(120, 40, false);
    let prior_focus = enter_nextest(&mut trust);
    assert!(trust.text().contains("Trust this task file?"));
    let _ = trust.key(KeyCode::Esc);
    assert_eq!(trust.focus(), prior_focus);
    assert_eq!(task_trust(&trust), Some(Trust::Untrusted));
    assert!(trust.app().world.activities.is_empty());

    let mut args = fixture(120, 40, false);
    let prior_focus = enter_nextest(&mut args);
    approve_trust(&mut args);
    assert!(args.text().contains("Task inputs"));
    let _ = args.key(KeyCode::Esc);
    assert!(args.text().contains("Task inputs"));
    assert_eq!(args.focus(), Some(FILTER_ID));
    let _ = args.key(KeyCode::Esc);
    assert_eq!(args.focus(), prior_focus);
    assert_eq!(task_trust(&args), Some(Trust::Trusted));
    assert!(args.app().world.activities.is_empty());

    let mut review = fixture(120, 40, true);
    let prior_focus = enter_nextest(&mut review);
    let _ = review.ctrl('s');
    assert!(review.text().contains("Review task inputs"));
    let _ = review.key(KeyCode::Esc);
    assert_eq!(review.focus(), prior_focus);
    assert!(review.app().status.is_none());
    assert!(review.app().world.activities.is_empty());
    assert_eq!(review.app().world.effect_revision, 0);
}

#[test]
fn changed_body_before_trust_during_edit_and_at_review_fails_closed() {
    let mut before_trust = fixture(120, 40, false);
    let _ = enter_nextest(&mut before_trust);
    assert!(before_trust.text().contains("Trust this task file?"));
    if let Some(mise) = &mut before_trust.app_mut().world.mise
        && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
    {
        task.command.push_str(" --body-changed");
    }
    approve_trust(&mut before_trust);
    assert!(!before_trust.text().contains("Task inputs"));
    assert_eq!(task_trust(&before_trust), Some(Trust::Untrusted));
    assert!(
        before_trust
            .app()
            .status
            .as_deref()
            .is_some_and(|message| message.contains("definition changed"))
    );
    assert!(before_trust.app().world.activities.is_empty());

    let mut during_edit = fixture(120, 40, false);
    let _ = enter_nextest(&mut during_edit);
    approve_trust(&mut during_edit);
    assert!(during_edit.text().contains("Task inputs"));
    if let Some(mise) = &mut during_edit.app_mut().world.mise
        && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
    {
        task.command.push_str(" --body-changed");
    }
    let _ = during_edit.ctrl('s');
    assert!(!during_edit.text().contains("Review task inputs"));
    assert!(
        during_edit
            .app()
            .status
            .as_deref()
            .is_some_and(|message| message.contains("definition changed"))
    );
    assert!(during_edit.app().world.activities.is_empty());

    let mut at_review = fixture(120, 40, true);
    let _ = enter_nextest(&mut at_review);
    let _ = at_review.ctrl('s');
    assert!(at_review.text().contains("Review task inputs"));
    if let Some(mise) = &mut at_review.app_mut().world.mise
        && let Some(task) = mise.tasks.iter_mut().find(|task| task.id == "test-nextest")
    {
        task.command.push_str(" --body-changed");
    }
    let _ = at_review.key(KeyCode::Enter);
    assert!(
        at_review
            .app()
            .status
            .as_deref()
            .is_some_and(|message| message.contains("definition changed"))
    );
    assert!(at_review.app().world.activities.is_empty());
    assert_eq!(at_review.app().world.effect_revision, 0);
}

#[test]
fn supported_small_terminal_keeps_the_real_args_route_reachable() {
    let mut harness = fixture(72, 20, false);
    let _ = enter_nextest(&mut harness);
    assert!(
        harness.text().contains("Trust this task file?"),
        "{}",
        harness.text()
    );
    approve_trust(&mut harness);
    assert!(harness.text().contains("Task inputs"), "{}", harness.text());
    let _ = harness.ctrl('s');
    assert!(
        harness.text().contains("Review task inputs"),
        "{}",
        harness.text()
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .app()
            .status
            .as_deref()
            .is_some_and(|message| message.contains("nothing executed"))
    );
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn host_context_change_at_review_discards_the_accepted_plan() {
    let mut harness = fixture(120, 40, true);
    let _ = enter_nextest(&mut harness);
    let _ = harness.ctrl('s');
    assert!(harness.text().contains("Review task inputs"));
    harness.app_mut().world.cwd.push_str("/changed");
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .app()
            .status
            .as_deref()
            .is_some_and(|message| message.contains("context changed"))
    );
    assert!(harness.app().world.activities.is_empty());
    assert_eq!(harness.app().world.effect_revision, 0);
}
