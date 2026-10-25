#![cfg(test)]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Native port-route controls use fixed fixture rows and required harness values"
)]

use super::{App, Motion, Overlay, Route, home};
use crate::{
    dispatch::{self, Destination},
    domain::{
        action::Action,
        ranking::{Pin, Usage},
    },
    scenario::Scenario,
    screens::port_args,
    sim::{catalogue, fixtures},
};
use termrock::{ItemKey, KeyCode, Part, PartRef, Theme};
use termrock_test_support::Harness;

fn harness(app: App, width: u16, height: u16) -> Harness<App> {
    let initial =
        termrock::SimulationMoment::from_millis(u64::try_from(app.fixture_time_ms()).unwrap());
    let theme = Theme::junie();
    let app = app.with_source_theme(theme.clone());
    Harness::new_with_feedback_clock(
        app,
        theme,
        width,
        height,
        termrock::FeedbackClock::Simulation { initial },
    )
}

fn rust_dirty() -> Harness<App> {
    harness(
        App::for_scenario_without_history(Scenario::RustDirty, Motion::Paused, 4_000),
        120,
        40,
    )
}

fn rust_dirty_without_auto_draw() -> Harness<App> {
    rust_dirty().with_auto_draw(false)
}

fn type_port_query(harness: &mut Harness<App>, query: &str) -> Action {
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str(query);
    let registered = harness.app().home.rows(&harness.app().world).len();
    assert!(
        registered > 0,
        "query {query:?} must produce registered Home rows"
    );
    assert!(harness.tab_to(home::ROWS));
    assert!(
        harness
            .area_of_part(
                home::ROWS,
                PartRef::item(Part::ROW, ItemKey::text("system.port"))
            )
            .is_some()
    );
    let mut selected = harness
        .app()
        .home
        .selected(&harness.app().world)
        .expect("the query must select an authored row");
    for _ in 0..registered {
        if selected.id == "system.port" {
            assert_eq!(selected.intent_id(), Some("system.port"));
            return selected;
        }
        let _ = harness.key(KeyCode::Down);
        selected = harness
            .app()
            .home
            .selected(&harness.app().world)
            .expect("registered Home rows must remain selectable");
    }
    assert_eq!(selected.id, "system.port", "selected raw Home row");
    assert_eq!(selected.intent_id(), Some("system.port"));
    selected
}

fn enter_port_editor(harness: &mut Harness<App>, query: &str) -> Action {
    let selected = type_port_query(harness, query);
    assert_eq!(selected.intent_id(), Some("system.port"));
    assert_eq!(selected.id, "system.port");
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    selected
}

fn replace_port_value(harness: &mut Harness<App>, value: &str) {
    let _ = harness.ctrl('l');
    let _ = harness.key(KeyCode::Backspace);
    let _ = harness.paste(value);
}

#[test]
fn ordinary_rust_dirty_home_reaches_the_exact_authored_row_and_default_input() {
    let mut harness = rust_dirty();
    assert!(catalogue::parity_discovery(&harness.app().world).is_none());
    let selected = enter_port_editor(&mut harness, "port");
    assert_eq!(selected.id, "system.port");
    assert!(harness.text().contains("Port input"), "{}", harness.text());
    assert!(harness.text().contains("5173"), "{}", harness.text());
    assert_eq!(harness.app().effect_revision(), 0);
    assert_eq!(harness.app().world.activities.len(), 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn active_draft_ctrl_s_reviews_valid_port_without_an_extra_enter() {
    let mut harness = rust_dirty();
    enter_port_editor(&mut harness, "port");
    replace_port_value(&mut harness, "8080");
    let _ = harness.ctrl('s');
    assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
    let review = harness.text();
    assert!(review.contains("Review port lookup"), "{review}");
    assert!(review.contains("parsed as u16"), "{review}");
    assert!(review.contains("Effective port"), "{review}");
    assert!(review.contains("8080"), "{review}");
    assert!(review.contains("argv[1] '-iTCP:8080'"), "{review}");
    assert!(review.contains("argv[2] '-sTCP:LISTEN'"), "{review}");
    assert_eq!(harness.app().effect_revision(), 0);
    assert_eq!(harness.app().world.activities.len(), 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn blank_first_required_input_stays_open_and_focuses_the_port_field() {
    let mut harness = rust_dirty();
    enter_port_editor(&mut harness, "port");
    replace_port_value(&mut harness, "");
    let _ = harness.ctrl('s');
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    assert_eq!(harness.focus(), Some(port_args::PORT));
    let port_regions: Vec<_> = harness
        .runtime()
        .registry()
        .regions()
        .iter()
        .filter(|region| region.owner == port_args::PORT)
        .map(|region| (region.part, region.kind, region.area))
        .collect();
    let frame = harness.text();
    assert!(
        frame.contains("Enter a non-empty TCP port"),
        "required-field message missing; frame:\n{frame}\nPORT area={:?}, HELP area={:?}, focus={:?}, state={:?}, regions={port_regions:?}, diagnostics={:?}",
        harness.area_of(port_args::PORT),
        harness.area_of_part(port_args::PORT, PartRef::of(Part::HELP)),
        harness.focus(),
        harness.state_of(port_args::PORT),
        harness.diagnostics(),
    );
    assert_eq!(harness.app().effect_revision(), 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn malformed_nonempty_input_reviews_the_captured_legacy_fallback_and_cancel_is_pure() {
    let mut harness = rust_dirty();
    let selected = type_port_query(&mut harness, "port");
    assert_eq!(selected.id, "system.port");
    assert_eq!(selected.intent_id(), Some("system.port"));
    let prior_focus = harness.focus();
    assert_eq!(prior_focus, Some(home::ROWS));
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    let revision = harness.app().effect_revision();
    let before_host = harness.app().world.host.clone();
    let before_cwd = harness.app().world.cwd.clone();
    replace_port_value(&mut harness, "abc");
    let _ = harness.ctrl('s');
    let review = harness.text();
    assert!(
        matches!(harness.app().overlay, Some(Overlay::Dialog(_))),
        "pending={:?}, input_opening={:?}, focus={:?}, top_layer={:?}, port_layer_open={}, diagnostics={:?}\n{review}",
        harness
            .app()
            .pending_port_review
            .as_ref()
            .map(|pending| pending.opening),
        harness
            .app()
            .port_args_opening
            .as_ref()
            .map(|opening| opening.token),
        harness.focus(),
        harness.top_layer(),
        harness.is_open(termrock::Id::root("holla.port-args.dialog")),
        harness.diagnostics(),
    );
    assert!(review.contains("legacy fallback to 5173"), "{review}");
    assert!(
        review.contains("Invalid text: historical fallback"),
        "{review}"
    );
    assert!(review.contains("Effective port"), "{review}");
    assert!(review.contains("argv[1] '-iTCP:5173'"), "{review}");
    assert!(review.contains("argv[2] '-sTCP:LISTEN'"), "{review}");
    assert!(harness.area_of(port_args::DIALOG).is_none());
    assert!(
        !harness
            .state_of(port_args::DIALOG)
            .contains(termrock::StateFlags::FOCUSED)
    );
    assert!(harness.area_of(port_args::PORT).is_none());
    assert!(
        !harness
            .state_of(port_args::PORT)
            .contains(termrock::StateFlags::FOCUSED)
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().overlay.is_none());
    assert_eq!(harness.focus(), prior_focus);
    assert_eq!(harness.app().effect_revision(), revision);
    assert_eq!(harness.app().world.host, before_host);
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().world.activities.len(), 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn accepted_port_input_does_not_open_review_without_its_native_opening_token() {
    let mut harness = rust_dirty();
    enter_port_editor(&mut harness, "port");
    assert!(harness.app().port_args_opening.is_some());
    let missing = harness.app_mut().port_args_opening.take();
    assert!(missing.is_some(), "the native input opening was recorded");

    let before_host = harness.app().world.host.clone();
    let before_cwd = harness.app().world.cwd.clone();
    let before_revision = harness.app().effect_revision();
    let _ = harness.ctrl('s');

    assert!(harness.app().overlay.is_none());
    assert!(harness.app().pending_port_review.is_none());
    assert!(!harness.text().contains("Review port lookup"));
    assert!(harness.text().contains("Port input changed · review again"));
    assert_eq!(harness.app().world.host, before_host);
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().effect_revision(), before_revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn accepted_port_input_is_discarded_if_its_route_changes_before_focus_publication() {
    let mut harness = rust_dirty_without_auto_draw();
    enter_port_editor(&mut harness, "port");
    replace_port_value(&mut harness, "8080");
    let before_host = harness.app().world.host.clone();
    let before_cwd = harness.app().world.cwd.clone();
    let before_revision = harness.app().effect_revision();

    let _ = harness.ctrl('s');
    assert!(harness.app().overlay.is_none());
    assert!(harness.app().pending_port_review.is_some());
    harness.app_mut().route = Route::Files;
    harness.draw();

    assert!(harness.app().pending_port_review.is_none());
    assert!(harness.app().overlay.is_none());
    assert!(!harness.text().contains("Review port lookup"));
    assert!(
        harness
            .text()
            .contains("Port action changed · review again")
    );
    assert_eq!(harness.app().world.host, before_host);
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().effect_revision(), before_revision);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn explicit_simulate_consumes_review_once_and_reports_nothing_executed() {
    let mut harness = rust_dirty();
    enter_port_editor(&mut harness, "port");
    let before_host = harness.app().world.host.clone();
    let before_cwd = harness.app().world.cwd.clone();
    replace_port_value(&mut harness, "8080");
    let _ = harness.ctrl('s');
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.app().overlay.is_none());
    assert!(harness.text().contains("Simulation confirmed"));
    assert!(harness.text().contains("nothing executed"));
    assert!(harness.text().contains("argv[1] '-iTCP:8080'"));
    assert_eq!(harness.app().world.host, before_host);
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().effect_revision(), 0);
    assert_eq!(harness.app().world.activities.len(), 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn cancelling_input_restores_home_focus_and_has_no_effect() {
    let mut harness = rust_dirty();
    let selected = type_port_query(&mut harness, "port");
    assert_eq!(selected.id, "system.port");
    assert_eq!(selected.intent_id(), Some("system.port"));
    let prior_focus = harness.focus();
    assert_eq!(prior_focus, Some(home::ROWS));
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    let revision = harness.app().effect_revision();
    let before_host = harness.app().world.host.clone();
    let before_cwd = harness.app().world.cwd.clone();
    replace_port_value(&mut harness, "8080");
    assert!(harness.text().contains("8080"), "{}", harness.text());
    let _ = harness.key(KeyCode::Esc);
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    assert_eq!(harness.focus(), Some(port_args::PORT));
    assert!(harness.text().contains("5173"), "{}", harness.text());
    let _ = harness.key(KeyCode::Esc);
    assert!(harness.app().overlay.is_none());
    assert_eq!(harness.focus(), prior_focus);
    assert_eq!(harness.app().effect_revision(), revision);
    assert_eq!(harness.app().world.host, before_host);
    assert_eq!(harness.app().world.cwd, before_cwd);
    assert_eq!(harness.app().world.activities.len(), 0);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn stale_context_during_draft_or_review_rejects_without_new_plan() {
    let mut draft = rust_dirty();
    enter_port_editor(&mut draft, "port");
    draft.app_mut().world.effect_revision = 1;
    let _ = draft.ctrl('s');
    assert!(draft.app().overlay.is_none());
    assert!(draft.text().contains("context changed"), "{}", draft.text());
    assert_eq!(draft.app().effect_revision(), 1);
    assert_eq!(draft.app().world.activities.len(), 0);

    let mut review = rust_dirty();
    enter_port_editor(&mut review, "port");
    let _ = review.ctrl('s');
    review.app_mut().world.effect_revision = 1;
    let _ = review.key(KeyCode::Enter);
    assert!(review.app().overlay.is_none());
    assert!(
        review.text().contains("context changed"),
        "{}",
        review.text()
    );
    assert_eq!(review.app().effect_revision(), 1);
    assert_eq!(review.app().world.activities.len(), 0);
}

#[test]
fn recent_pin_and_alias_rows_keep_canonical_identity_through_the_home_route() {
    let make_app = |memory: &str| {
        let mut app = App::for_scenario_without_history(Scenario::RustDirty, Motion::Paused, 4_000);
        let canonical = catalogue::catalogue(&app.world)
            .into_iter()
            .find(|action| action.id == "system.port")
            .unwrap();
        match memory {
            "recent" => app.world.memory.usage.push(Usage {
                path: app.world.cwd.clone(),
                command: canonical.command.clone(),
                count: 1,
            }),
            "pin" => app.world.memory.pins.push(Pin {
                path: app.world.cwd.clone(),
                command: canonical.command.clone(),
            }),
            "alias" => app
                .world
                .memory
                .set_alias("ports", &canonical.command)
                .unwrap(),
            _ => {}
        }
        app
    };

    for (memory, query, expected_id) in [
        (
            "recent",
            "port",
            "recent:lsof -nP -iTCP:<port> -sTCP:LISTEN",
        ),
        ("pin", "port", "pin:lsof -nP -iTCP:<port> -sTCP:LISTEN"),
        ("alias", "ports", "system.port"),
    ] {
        let mut harness = harness(make_app(memory), 120, 40);
        assert!(harness.tab_to(home::QUERY));
        let _ = harness.type_str(query);
        let rows = harness.app().home.rows(&harness.app().world);
        let bound = rows.len();
        assert!(bound > 0, "{memory} query had no registered rows");
        assert!(harness.tab_to(home::ROWS));
        assert!(
            harness
                .area_of_part(
                    home::ROWS,
                    PartRef::item(Part::ROW, ItemKey::text(expected_id)),
                )
                .is_some(),
            "{memory} row must have a native registered hit area"
        );
        let mut selected = harness.app().home.selected(&harness.app().world).unwrap();
        for _ in 0..bound {
            if selected.id == expected_id {
                break;
            }
            let _ = harness.key(KeyCode::Down);
            selected = harness.app().home.selected(&harness.app().world).unwrap();
        }
        assert_eq!(selected.id, expected_id, "{memory} raw row");
        assert_eq!(selected.intent_id(), Some("system.port"), "{memory} intent");
        let before_host = harness.app().world.host.clone();
        let before_cwd = harness.app().world.cwd.clone();
        let before_revision = harness.app().effect_revision();
        let before_activities = harness.app().world.activities.len();
        let _ = harness.key(KeyCode::Enter);
        assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
        replace_port_value(&mut harness, "8080");
        let _ = harness.ctrl('s');
        assert!(matches!(harness.app().overlay, Some(Overlay::Dialog(_))));
        let review = harness.text();
        assert!(review.contains("Review port lookup"), "{memory}: {review}");
        assert!(
            review.contains("Inspect port (simulated)"),
            "{memory}: {review}"
        );
        assert!(
            review.contains(before_host.name.as_str()),
            "{memory}: {review}"
        );
        assert!(review.contains("Environment"), "{memory}: {review}");
        assert!(review.contains("parsed as u16"), "{memory}: {review}");
        assert!(review.contains("Effective port"), "{memory}: {review}");
        assert!(review.contains("8080"), "{memory}: {review}");
        assert!(review.contains("program 'lsof'"), "{memory}: {review}");
        assert!(review.contains("argv[0] '-nP'"), "{memory}: {review}");
        assert!(
            review.contains("argv[1] '-iTCP:8080'"),
            "{memory}: {review}"
        );
        assert!(
            review.contains("argv[2] '-sTCP:LISTEN'"),
            "{memory}: {review}"
        );
        assert_eq!(harness.app().world.host, before_host);
        assert_eq!(harness.app().world.cwd, before_cwd);
        assert_eq!(harness.app().effect_revision(), before_revision);
        assert_eq!(harness.app().world.activities.len(), before_activities);
        assert!(
            harness.diagnostics().is_empty(),
            "{:?}",
            harness.diagnostics()
        );
    }
}

fn pinned_port_harness() -> (Harness<App>, String) {
    let mut app = App::for_scenario_without_history(Scenario::RustDirty, Motion::Paused, 4_000);
    let canonical = catalogue::catalogue(&app.world)
        .into_iter()
        .find(|action| action.id == "system.port")
        .unwrap();
    let pin_id = format!("pin:{}", canonical.command);
    app.world.memory.pins.push(Pin {
        path: app.world.cwd.clone(),
        command: canonical.command,
    });
    (harness(app, 120, 40), pin_id)
}

fn enter_pinned_port_editor(harness: &mut Harness<App>, expected_id: &str) -> Action {
    assert!(harness.tab_to(home::QUERY));
    let _ = harness.type_str("port");
    let registered = harness.app().home.rows(&harness.app().world).len();
    assert!(
        registered > 0,
        "pin query must produce registered Home rows"
    );
    assert!(harness.tab_to(home::ROWS));
    assert!(
        harness
            .area_of_part(
                home::ROWS,
                PartRef::item(Part::ROW, ItemKey::text(expected_id)),
            )
            .is_some(),
        "the pin must have a native registered hit area"
    );
    let mut selected = harness
        .app()
        .home
        .selected(&harness.app().world)
        .expect("registered Home rows must remain selectable");
    for _ in 0..registered {
        if selected.id == expected_id {
            break;
        }
        let _ = harness.key(KeyCode::Down);
        selected = harness
            .app()
            .home
            .selected(&harness.app().world)
            .expect("registered Home rows must remain selectable");
    }
    assert_eq!(selected.id, expected_id, "selected raw pin row");
    assert_eq!(selected.intent_id(), Some("system.port"));
    let selected = selected.clone();
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    selected
}

#[test]
fn removing_a_captured_pin_during_draft_or_review_rejects_without_effects() {
    let (mut draft, pin_id) = pinned_port_harness();
    let selected = enter_pinned_port_editor(&mut draft, &pin_id);
    assert_eq!(selected.id, pin_id);
    assert_eq!(selected.intent_id(), Some("system.port"));
    let draft_host = draft.app().world.host.clone();
    let draft_cwd = draft.app().world.cwd.clone();
    let draft_revision = draft.app().effect_revision();
    draft.app_mut().world.memory.pins.clear();
    let _ = draft.ctrl('s');
    assert!(draft.app().overlay.is_none());
    assert!(
        draft.text().contains("port action changed"),
        "{}",
        draft.text()
    );
    assert!(draft.app().world.memory.pins.is_empty());
    assert_eq!(draft.app().world.host, draft_host);
    assert_eq!(draft.app().world.cwd, draft_cwd);
    assert_eq!(draft.app().effect_revision(), draft_revision);
    assert!(draft.app().world.activities.is_empty());
    assert!(draft.diagnostics().is_empty(), "{:?}", draft.diagnostics());

    let (mut review, pin_id) = pinned_port_harness();
    let selected = enter_pinned_port_editor(&mut review, &pin_id);
    assert_eq!(selected.id, pin_id);
    assert_eq!(selected.intent_id(), Some("system.port"));
    let review_host = review.app().world.host.clone();
    let review_cwd = review.app().world.cwd.clone();
    let review_revision = review.app().effect_revision();
    let _ = review.ctrl('s');
    assert!(matches!(review.app().overlay, Some(Overlay::Dialog(_))));
    let review_text = review.text();
    assert!(
        review_text.contains("Inspect port (simulated)"),
        "{review_text}"
    );
    assert!(
        review_text.contains("argv[1] '-iTCP:5173'"),
        "{review_text}"
    );
    review.app_mut().world.memory.pins.clear();
    let _ = review.key(KeyCode::Enter);
    assert!(review.app().overlay.is_none());
    assert!(
        review.text().contains("port action changed"),
        "{}",
        review.text()
    );
    assert!(review.app().world.memory.pins.is_empty());
    assert_eq!(review.app().world.host, review_host);
    assert_eq!(review.app().world.cwd, review_cwd);
    assert_eq!(review.app().effect_revision(), review_revision);
    assert!(review.app().world.activities.is_empty());
    assert!(
        review.diagnostics().is_empty(),
        "{:?}",
        review.diagnostics()
    );
}

#[test]
fn preview_actions_and_generic_confirmation_cannot_skip_typed_input() {
    let mut native = rust_dirty();
    let _ = type_port_query(&mut native, "port");
    assert!(native.tab_to(home::ROWS));
    let _ = native.ctrl('p');
    assert!(matches!(native.app().overlay, Some(Overlay::PortArgs(_))));
    let _ = native.key(KeyCode::Esc);

    let mut actions = rust_dirty();
    let _ = type_port_query(&mut actions, "port");
    assert!(actions.tab_to(home::ROWS));
    let _ = actions.ctrl('o');
    let _ = actions.key(KeyCode::Down);
    let _ = actions.key(KeyCode::Enter);
    assert!(matches!(actions.app().overlay, Some(Overlay::PortArgs(_))));
    assert_eq!(
        actions
            .app()
            .port_args_opening
            .as_ref()
            .and_then(|opening| opening.origin_focus),
        None,
        "the Actions layer does not supply a root-page focus owner"
    );
    replace_port_value(&mut actions, "8080");
    let _ = actions.ctrl('s');
    assert!(matches!(actions.app().overlay, Some(Overlay::Dialog(_))));
    let review = actions.text();
    assert!(review.contains("Review port lookup"), "{review}");
    assert!(review.contains("argv[1] '-iTCP:8080'"), "{review}");
    assert!(actions.area_of(port_args::DIALOG).is_none());
    assert!(
        !actions
            .state_of(port_args::DIALOG)
            .contains(termrock::StateFlags::FOCUSED)
    );
    assert!(actions.area_of(port_args::PORT).is_none());
    assert!(
        !actions
            .state_of(port_args::PORT)
            .contains(termrock::StateFlags::FOCUSED)
    );
    let _ = actions.key(KeyCode::Esc);
    assert!(actions.app().overlay.is_none());
    assert_eq!(actions.app().effect_revision(), 0);
    assert!(actions.app().world.activities.is_empty());
    assert!(
        actions.diagnostics().is_empty(),
        "{:?}",
        actions.diagnostics()
    );

    let mut world = fixtures::world_for(Scenario::RustDirty);
    let canonical = catalogue::catalogue(&world)
        .into_iter()
        .find(|action| action.id == "system.port")
        .unwrap();
    assert!(matches!(
        dispatch::preview(&mut world, &canonical),
        Destination::PortArgs(_)
    ));
    assert!(matches!(
        dispatch::invoke(&mut world, &canonical),
        Destination::PortArgs(_)
    ));
    assert!(matches!(
        dispatch::confirm_preview(&mut world, &canonical),
        Destination::PortArgs(_)
    ));
    let mut stale = canonical.clone();
    stale.command = "lsof -nP".to_owned();
    assert!(matches!(
        dispatch::invoke(&mut world, &stale),
        Destination::Notice(_)
    ));
    let lookalike = Action::unavailable_memory(&canonical.command, &world.cwd);
    assert!(!matches!(
        dispatch::invoke(&mut world, &lookalike),
        Destination::PortArgs(_)
    ));
}

#[test]
fn home_alias_route_reaches_only_the_current_authored_port_action() {
    let mut app = App::for_scenario_without_history(Scenario::RustDirty, Motion::Paused, 4_000);
    let canonical = catalogue::catalogue(&app.world)
        .into_iter()
        .find(|action| action.id == "system.port")
        .unwrap();
    app.world
        .memory
        .set_alias("ports", &canonical.command)
        .unwrap();
    let mut harness = harness(app, 120, 40);
    let selected = type_port_query(&mut harness, "ports");
    assert_eq!(selected.id, "system.port");
    assert_eq!(selected.intent_id(), Some("system.port"));
    let _ = harness.key(KeyCode::Enter);
    assert!(matches!(harness.app().overlay, Some(Overlay::PortArgs(_))));
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
