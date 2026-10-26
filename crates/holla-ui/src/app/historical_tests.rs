#![cfg(test)]
use super::{App, Motion, Route, Scenario, dialogs};
use crate::domain::{mise::Trust, ranking::Pin};
use ratatui::layout::Position;
use termrock::{
    Color, ColorLevel, Dialog, FeedbackClock, KeyCode, Part, PartRef, SimulationMoment, Theme,
};
use termrock_test_support::{Baseline, Harness};

const BORDER_BASELINE: Baseline = Baseline::new(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/baselines/app.txt"
));
fn fixture(
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    width: u16,
    height: u16,
) -> Harness<App> {
    let source_theme = Theme::junie();
    let app = App::for_scenario_without_history(scenario, motion, frame)
        .with_source_theme(source_theme.clone());
    let frame = u64::try_from(app.fixture_time_ms());
    assert!(frame.is_ok(), "fixture time must be nonnegative");
    let initial = SimulationMoment::from_millis(frame.unwrap_or_default());
    Harness::new_with_feedback_clock(
        app,
        source_theme,
        width,
        height,
        FeedbackClock::Simulation { initial },
    )
}

fn home_display_rows(world: &crate::sim::world::World) -> Vec<(String, String)> {
    let recent_first = world.scenario.boot_title() == crate::screens::finder::RECENT;
    crate::sim::catalogue::sections(world)
        .into_iter()
        .flat_map(|section| {
            let name = if recent_first && section.name == "Suggested here" {
                "Explore"
            } else {
                section.name
            };
            crate::sim::catalogue::visible(&section.actions, None, "", &world.memory, &world.cwd)
                .into_iter()
                .map(move |action| (name.to_owned(), action.id))
        })
        .collect()
}

fn home_visual_extent(rows: &[(String, String)]) -> usize {
    let mut sections = 0usize;
    let mut previous = None;
    for (section, _) in rows {
        if previous != Some(section.as_str()) {
            sections = sections.saturating_add(1);
            previous = Some(section.as_str());
        }
    }
    rows.len()
        .saturating_add(sections)
        .saturating_add(sections.saturating_sub(1))
}

fn exposed_track_cells(
    track: ratatui::layout::Rect,
    thumb: ratatui::layout::Rect,
) -> Vec<(u16, u16)> {
    let mut cells = Vec::new();
    for y in track.y..track.y.saturating_add(track.height) {
        for x in track.x..track.x.saturating_add(track.width) {
            let in_thumb = x >= thumb.x
                && x < thumb.x.saturating_add(thumb.width)
                && y >= thumb.y
                && y < thumb.y.saturating_add(thumb.height);
            if !in_thumb {
                cells.push((x, y));
            }
        }
    }
    cells
}

fn find(h: &Harness<App>, needle: &str) -> (u16, u16) {
    let position = h.find(needle);
    assert!(position.is_some(), "missing fixture text: {needle}");
    position.unwrap_or_default()
}

type TaskTrustSnapshot = Vec<(String, String, crate::domain::mise::Trust)>;

fn task_trust_snapshot(h: &Harness<App>) -> Option<TaskTrustSnapshot> {
    h.app().world.mise.as_ref().map(|mise| {
        mise.tasks
            .iter()
            .map(|task| (task.id.clone(), task.defined_in.clone(), task.trust))
            .collect()
    })
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

    let last = t.lines().nth(39).unwrap_or("");
    assert!(last.contains("Type") && last.contains("Filter"), "{last:?}");
    assert_eq!(t.matches("Run top match").count(), 1, "{t}");

    assert!(last.contains("EDIT"), "{last:?}");
    assert!(t.contains("Search actions"), "{t}");
}

#[test]
fn menu_bar_opens_and_quit_is_confirmable() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.key(KeyCode::F(10));
    let t = h.text();
    assert!(t.contains("Quit"), "{t}");
    assert!(t.contains("Ctrl+Q"), "{t}");

    let last = t.lines().nth(39).unwrap_or("");
    assert!(last.contains("Menu"), "{last:?}");
    let _ = h.key(KeyCode::Esc);

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

#[test]
fn empty_query_q_asks_to_quit_but_typed_q_filters() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.key(KeyCode::Char('q'));
    assert!(h.text().contains("Quit holla?"));
    let _ = h.key(KeyCode::Esc);

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
    assert!(t.contains("scope ‹ here ›"), "{t}");
    assert!(!t.contains("Connect to prod-eu-1"), "{t}");
    let _ = h.ctrl('s');
    let _ = h.ctrl('s');
    let _ = h.ctrl('s');
    assert!(h.text().contains("scope ‹ host ›"), "{}", h.text());
    assert!(h.text().contains("Connect to prod-eu-1"), "{}", h.text());
    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("scope ‹ host ›"), "{}", h.text());
}

#[test]
fn down_moves_into_results_and_back() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("monit");
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);

    let t = h.text();
    assert!(t.contains("System snapshot"), "{t}");
    assert!(t.contains("Handoff"), "{t}");
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("simulated handoff"), "{}", h.text());
}

#[test]
fn enter_on_top_match_reports_simulated_run() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Would run: make test"), "{t}");
    assert!(t.contains("nothing executed"), "{t}");
}

#[test]
fn actions_menu_offers_alternatives_and_pinning() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Pin here"), "{t}");
    assert!(t.contains("Copy command"), "{t}");
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);

    let _ = h.type_str("status");
    let _ = h.ctrl('o');
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Pinned git status here"), "{}", h.text());
    let _ = h.key(KeyCode::Esc);
    assert!(h.app().world.memory.pin_at("~/work/pave", "git status"));
}

#[test]
fn alias_prompt_saves_and_query_matches_expansion() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Set alias…"), "{t}");

    for _ in 0..4 {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Alias for “cargo build”"), "{t}");
    let _ = h.type_str("cb");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Alias cb → cargo build"), "{}", h.text());
    assert_eq!(h.app().world.memory.alias_for("cargo build"), Some("cb"));

    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("cb");
    assert!(h.text().contains("cargo build"), "{}", h.text());
}

#[test]
fn hide_removes_row_exact_query_resurfaces_and_reset_restores() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');

    for _ in 0..5 {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Hidden cargo build here · Reset ranking restores"),
        "{}",
        h.text()
    );
    assert!(h.app().world.memory.hidden_at("~/work/pave", "cargo build"));

    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("cargo b");
    assert!(
        !h.text().contains("cargo build · cargo build"),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("used 6 times"), "{}", h.text());

    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Unhide here"), "{t}");

    for _ in 0..6 {
        let _ = h.key(KeyCode::Down);
    }
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Reset ranking for cargo build"),
        "{}",
        h.text()
    );
    assert!(!h.app().world.memory.hidden_at("~/work/pave", "cargo build"));
}

#[test]
fn actions_picker_omits_search_query_as_pinned() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    assert!(!h.text().contains("Type to search…"), "{}", h.text());
}

#[test]
fn strip_lists_seeded_activities_with_states() {
    let h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    let t = h.text();

    let row1: String = t.lines().nth(1).unwrap_or_default().to_owned();
    assert!(row1.contains("● 1 dev server"), "{row1}");
    assert!(row1.contains("… 2 test watch"), "{row1}");
    assert!(row1.contains("− 3 deploy logs"), "{row1}");
    assert!(row1.contains("✗ 4 seed db"), "{row1}");
    assert!(row1.contains("0 home"), "{row1}");
}

#[test]
fn follow_logs_merges_services_with_identity() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("follow container logs");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Activity, "{}", h.text());
    let t = h.text();
    assert!(
        t.lines().nth(2).unwrap_or("").contains("container logs"),
        "{t}"
    );

    assert!(t.contains("api"), "{t}");
    assert!(t.contains("redis"), "{t}");
    assert!(t.contains("worker"), "{t}");
    assert!(t.contains("GET /health 200"), "{t}");
    assert!(t.contains("background save done"), "{t}");

    assert!(!t.contains("payments-old |"), "{t}");

    let _ = h.key(KeyCode::Char('0'));
    assert!(h.app().route == Route::Home);
    assert!(
        h.app()
            .world
            .activities
            .iter()
            .any(|a| a.name == "container logs")
    );
}

#[test]
fn ctrl_a_cycles_activities_from_any_route() {
    let mut h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);

    let _ = h.ctrl('a');
    assert!(h.app().route == Route::Activity, "{}", h.text());
    assert!(
        h.text().lines().nth(2).unwrap_or("").contains("dev server"),
        "{}",
        h.text()
    );

    let _ = h.ctrl('a');
    assert!(
        h.text().lines().nth(2).unwrap_or("").contains("test watch"),
        "{}",
        h.text()
    );

    let _ = h.ctrl('a');
    let _ = h.ctrl('a');
    let _ = h.ctrl('a');
    assert!(
        h.text().lines().nth(2).unwrap_or("").contains("dev server"),
        "{}",
        h.text()
    );
}

#[test]
fn digit_switches_to_activity_page() {
    let mut h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);

    let _ = h.type_str("1");
    assert!(h.app().route == Route::Home, "home digits filter the query");
    let _ = h.key(KeyCode::Backspace);

    let (x, _) = find(&h, "4 seed db");
    assert!(x < u16::MAX, "activity label coordinate fits");
    let _ = h.click(x.saturating_add(1), 1);
    assert!(h.app().route == Route::Activity, "{}", h.text());
    let t = h.text();
    assert!(t.lines().nth(2).unwrap_or("").contains("seed db"), "{t}");
    assert!(t.contains("failed"), "{t}");
    assert!(t.contains("psql: connection to server"), "{t}");

    let _ = h.key(KeyCode::Char('1'));
    assert!(
        h.text().lines().nth(2).unwrap_or("").contains("dev server"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("http://localhost:5199/"), "{}", h.text());

    let _ = h.key(KeyCode::Char('0'));
    assert!(h.app().route == Route::Home);
    assert_eq!(h.app().world.activities.len(), 4);
}

#[test]
fn switching_away_and_back_preserves_output_and_scope() {
    let mut h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);

    let (x, _) = find(&h, "1 dev server");
    assert!(x < u16::MAX, "activity label coordinate fits");
    let _ = h.click(x.saturating_add(1), 1);
    assert!(h.app().route == Route::Activity);
    let page = h.text();
    assert!(page.contains("Scope"), "{page}");
    assert!(page.contains("~/work/monorepo/apps/frontend"), "{page}");
    assert!(page.contains("vite v5.4 ready in 412 ms"), "{page}");

    let _ = h.key(KeyCode::Char('4'));
    assert!(h.text().contains("seed db"), "{}", h.text());
    let _ = h.key(KeyCode::Char('0'));
    assert!(h.app().route == Route::Home);

    let (x, _) = find(&h, "1 dev server");
    assert!(x < u16::MAX, "activity label coordinate fits");
    let _ = h.click(x.saturating_add(1), 1);
    let back = h.text();
    assert!(back.contains("~/work/monorepo/apps/frontend"), "{back}");
    assert!(back.contains("vite v5.4 ready in 412 ms"), "{back}");
    assert!(back.contains("running · 2h ago"), "{back}");
}

#[test]
fn merged_logs_scroll_retained_per_activity() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("follow container logs");
    let _ = h.key(KeyCode::Enter);

    let _ = h.key(KeyCode::Up);
    let _ = h.key(KeyCode::Up);
    let _ = h.key(KeyCode::Char('0'));
    let (x, _) = find(&h, "container logs");
    assert!(x < u16::MAX, "activity label coordinate fits");
    let _ = h.click(x.saturating_add(1), 1);
    assert!(h.app().route == Route::Activity, "{}", h.text());
    assert!(h.text().contains("container logs"), "{}", h.text());
}

fn open_docker_plan(h: &mut Harness<App>) {
    let _ = h.type_str("clean up docker");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Plan, "{}", h.text());
}

fn try_confirm(h: &mut Harness<App>, phrase: &str) {
    let _ = h.key(KeyCode::Enter);
    let _ = h.type_str(phrase);
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
}

#[test]
fn broad_action_opens_gate_one_review_surface() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);
    let t = h.text();
    assert!(t.contains("Clean up Docker data"), "{t}");
    assert!(t.contains("review · Space excludes"), "{t}");
    assert!(t.contains("Remove stopped containers"), "{t}");
    assert!(t.contains("Prune builder cache"), "{t}");
    assert!(t.contains("docker system df"), "{t}");
    assert!(t.contains("· builder"), "{t}");
    assert!(t.contains("· required"), "{t}");

    assert!(!t.contains("REMOVE ALL DOCKER DATA"), "{t}");
}

#[test]
fn exclusion_recalculates_dependents_live() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);

    let _ = h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("required · cannot exclude"),
        "{}",
        h.text()
    );

    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let t = h.text();
    assert!(t.contains("Excluded Remove stopped containers"), "{t}");
    assert!(t.contains("needs Remove stopped containers"), "{t}");

    assert!(!t.contains("needs Inspect Docker usage"), "{t}");

    let _ = h.key(KeyCode::Char(' '));
    assert!(
        !h.text().contains("needs Remove stopped containers"),
        "{}",
        h.text()
    );
}

#[test]
fn invalid_phrase_cannot_proceed() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(
        t.contains("Type REMOVE ALL DOCKER DATA ON devbox to confirm"),
        "{t}"
    );

    let _ = h.type_str("REMOVE ALL DOCKER DATA ON prod-eu-1");
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(!t.contains("Finished:"), "{t}");
    assert!(!plan_ran(&h));
}

fn plan_ran(h: &Harness<App>) -> bool {
    let plan = h.app().plan.as_ref();
    assert!(plan.is_some(), "expected reviewed plan");
    plan.is_some_and(|plan| plan.plan().ran())
}

#[test]
fn correct_phrase_runs_and_failure_propagates() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);
    try_confirm(&mut h, "REMOVE ALL DOCKER DATA ON devbox");
    let t = h.text();
    assert!(
        t.contains("Finished: 4 succeeded · 1 failed · 2 skipped"),
        "{t}"
    );
    assert!(
        t.contains("payments-old: bind mount still registered"),
        "{t}"
    );
    assert!(t.contains("needs Remove stopped containers"), "{t}");
    assert!(plan_ran(&h));

    let d = h.app().world.docker.as_ref();
    assert!(d.is_some(), "expected Docker fixture");
    assert_eq!(
        d.map(crate::domain::docker::DockerState::build_cache_bytes),
        Some(0)
    );
    assert!(!d.is_some_and(|docker| docker.containers.iter().any(|c| c.name == "web")));
    assert!(d.is_some_and(|docker| docker.containers.iter().any(|c| c.name == "payments-old")));

    let _ = h.key(KeyCode::Down);
    let t = h.text();
    assert!(t.contains("Output — Remove stopped containers"), "{t}");
    assert!(t.contains("Removed web"), "{t}");
}

#[test]
fn debian_plan_parallel_branches_and_i_understand_phrase() {
    let mut h = fixture(Scenario::UpgradePlan, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("upgrade everything");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("· apt"), "{t}");
    assert!(t.contains("· mise"), "{t}");

    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let t = h.text();
    assert!(t.contains("needs Apply upgrades"), "{t}");
    try_confirm(&mut h, "I UNDERSTAND: UPGRADE EVERYTHING ON devbox-deb");
    let t = h.text();
    assert!(t.contains("Finished:"), "{t}");
    assert!(t.contains("skipped"), "{t}");

    assert_eq!(
        h.app().world.debian.as_ref().map(|debian| debian.pending),
        Some(47)
    );
    let mise = h.app().world.mise.as_ref();
    assert!(mise.is_some(), "expected mise fixture");
    assert!(mise.is_some_and(|mise| {
        mise.tools
            .iter()
            .all(|t| !matches!(t.state, crate::domain::mise::ToolState::Outdated { .. }))
    }));
}

#[test]
fn production_restart_goes_through_two_gates() {
    let mut h = fixture(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("restart");
    let _ = h.key(KeyCode::Enter);

    assert!(h.app().route == Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Restart payments"), "{t}");
    try_confirm(&mut h, "RESTART PAYMENTS ON prod-eu-1");
    assert!(h.text().contains("Finished:"), "{}", h.text());
    let c = h.app().world.docker.as_ref().and_then(|docker| {
        docker
            .containers
            .iter()
            .find(|container| container.name == "payments")
    });
    assert_eq!(
        c.map(|container| container.state),
        Some(crate::domain::docker::ContainerState::Running)
    );
    assert_eq!(
        c.and_then(|container| container.health),
        Some(crate::domain::docker::Health::Healthy)
    );

    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("service is unhealthy"), "{}", h.text());
}

#[test]
fn disk_plan_policy_skips_active_today() {
    let mut h = fixture(Scenario::DiskCleanup, Motion::Paused, 4_000, 120, 40);
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("reclaim disk");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Remove node_modules"), "{t}");
    assert!(t.contains("active today · never removed"), "{t}");
    assert!(t.contains("Remove .gradle"), "{t}");
}

#[test]
fn mise_trust_gate_then_child_task_runs() {
    const RESET_ID: &str = "//projects/backend:reset-db";
    const TARGET_FILE: &str = "~/work/monorepo/projects/backend/mise.toml";
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let before = task_trust_snapshot(&h);
    assert!(before.is_some(), "monorepo fixture must include mise tasks");
    let Some(before) = before else { return };
    let target_before: Vec<_> = before
        .iter()
        .filter(|(_, file, _)| file == TARGET_FILE)
        .cloned()
        .collect();
    assert!(
        target_before
            .iter()
            .any(|(id, _, trust)| id == RESET_ID && *trust == Trust::Untrusted),
        "the selected child task must begin untrusted: {target_before:?}"
    );
    let unrelated_before: Vec<_> = before
        .iter()
        .filter(|(_, file, _)| file != TARGET_FILE)
        .cloned()
        .collect();
    let before_revision = h.app().world.effect_revision;
    let before_cwd = h.app().world.cwd.clone();
    let before_activities = h.app().world.activities.len();

    let _ = h.type_str("reset-db");
    let t = h.text();
    assert!(t.contains("untrusted config"), "{t}");
    assert!(t.contains("▲"), "{t}");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Trust this task file?"), "{t}");
    assert!(t.contains(TARGET_FILE), "{t}");
    assert!(t.contains("this exact file only"), "{t}");

    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Trusted mise.toml"), "{t}");
    assert!(t.contains("Would run:"), "{t}");
    let status = h.app().status.as_deref().unwrap_or("");
    assert!(
        status.contains("Trusted mise.toml · task re-resolved"),
        "{status:?}"
    );
    assert!(
        status.contains("Would run: mise run //projects/backend:reset-db"),
        "{status:?}"
    );
    assert!(status.contains("simulated, nothing executed"), "{status:?}");
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.cwd, before_cwd);
    assert_eq!(h.app().world.activities.len(), before_activities);

    let after = task_trust_snapshot(&h);
    assert!(after.is_some(), "mise state must remain available");
    let Some(after) = after else { return };
    let target_after: Vec<_> = after
        .iter()
        .filter(|(_, file, _)| file == TARGET_FILE)
        .cloned()
        .collect();
    let expected_target_after: Vec<_> = target_before
        .iter()
        .map(|(id, file, trust)| {
            if id == RESET_ID {
                (id.clone(), file.clone(), Trust::Trusted)
            } else {
                (id.clone(), file.clone(), *trust)
            }
        })
        .collect();
    assert_eq!(target_after, expected_target_after);
    let unrelated_after: Vec<_> = after
        .into_iter()
        .filter(|(_, file, _)| file != TARGET_FILE)
        .collect();
    assert_eq!(unrelated_after, unrelated_before);
}

#[test]
fn cancelling_mise_trust_does_not_continue_or_change_trust() {
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let before = task_trust_snapshot(&h);
    assert!(before.is_some(), "monorepo fixture must include mise tasks");
    let Some(before) = before else { return };
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();

    let _ = h.type_str("reset-db");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Trust this task file?"), "{}", h.text());
    let _ = h.key(KeyCode::Esc);

    let t = h.text();
    assert!(!t.contains("Trust this task file?"), "{t}");
    assert!(!t.contains("Would run:"), "{t}");
    assert_eq!(task_trust_snapshot(&h), Some(before));
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
}

#[test]
fn changed_action_context_rejects_mise_trust_without_continuation() {
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let before = task_trust_snapshot(&h);
    assert!(before.is_some(), "monorepo fixture must include mise tasks");
    let Some(before) = before else { return };
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();

    let _ = h.type_str("reset-db");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Trust this task file?"), "{}", h.text());
    h.app_mut().world.cwd = "~/work/monorepo/projects/backend".into();
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);

    let status = h.app().status.as_deref().unwrap_or("");
    assert!(
        status.contains("Action context changed · review it again"),
        "{status:?}"
    );
    assert!(!status.contains("Would run:"), "{status:?}");
    assert_eq!(task_trust_snapshot(&h), Some(before));
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
}

#[test]
fn changed_mise_task_file_rejects_exact_file_trust() {
    const RESET_ID: &str = "//projects/backend:reset-db";
    const OLD_FILE: &str = "~/work/monorepo/projects/backend/mise.toml";
    const NEW_FILE: &str = "~/work/monorepo/projects/backend/changed.mise.toml";
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let before = task_trust_snapshot(&h);
    assert!(before.is_some(), "monorepo fixture must include mise tasks");
    let Some(before) = before else { return };
    let before_revision = h.app().world.effect_revision;
    let before_cwd = h.app().world.cwd.clone();
    let before_activities = h.app().world.activities.len();

    let _ = h.type_str("reset-db");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Trust this task file?"), "{}", h.text());
    let mut changed = false;
    if let Some(mise) = h.app_mut().world.mise.as_mut() {
        for task in &mut mise.tasks {
            if task.id == RESET_ID {
                task.defined_in = NEW_FILE.into();
                changed = true;
            }
        }
    }
    assert!(changed, "fixture reset-db task must be available");
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);

    let status = h.app().status.as_deref().unwrap_or("");
    assert!(
        status.contains("Task file changed · nothing trusted"),
        "{status:?}"
    );
    assert!(!status.contains("Would run:"), "{status:?}");
    let after = task_trust_snapshot(&h);
    assert!(after.is_some(), "mise state must remain available");
    let Some(after) = after else { return };
    let before_id_trust: Vec<_> = before
        .iter()
        .map(|(id, _, trust)| (id.clone(), *trust))
        .collect();
    let after_id_trust: Vec<_> = after
        .iter()
        .map(|(id, _, trust)| (id.clone(), *trust))
        .collect();
    assert_eq!(after_id_trust, before_id_trust);
    assert!(after.iter().any(|(id, file, trust)| {
        id == RESET_ID && file == NEW_FILE && *trust == Trust::Untrusted
    }));
    assert!(
        !after
            .iter()
            .any(|(_, file, trust)| { file == OLD_FILE && *trust == Trust::Untrusted })
    );
    assert_eq!(h.app().world.cwd, before_cwd);
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
}

#[test]
fn already_trusted_mise_task_runs_on_enter() {
    const RESET_ID: &str = "//projects/backend:reset-db";
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let mut found = false;
    if let Some(mise) = h.app_mut().world.mise.as_mut() {
        for task in &mut mise.tasks {
            if task.id == RESET_ID {
                task.trust = Trust::Trusted;
                found = true;
            }
        }
    }
    assert!(found, "fixture reset-db task must be available");
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();

    let _ = h.type_str("reset-db");
    let _ = h.key(KeyCode::Enter);

    let t = h.text();
    assert!(!t.contains("Trust this task file?"), "{t}");
    assert!(t.contains("Would run:"), "{t}");
    let status = h.app().status.as_deref().unwrap_or("");
    assert!(
        status.contains("Would run: mise run //projects/backend:reset-db"),
        "{status:?}"
    );
    assert!(status.contains("simulated, nothing executed"), "{status:?}");
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
}

#[test]
fn clone_flow_collects_argument_then_reviews() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("clone");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Clone which repository?"), "{t}");
    assert!(t.contains("pave-io/pave"), "{t}");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    for fact in [
        "Account",
        "Owner",
        "Protocol",
        "Destination",
        "Primary branch",
        "Fork",
    ] {
        assert!(t.contains(fact), "missing {fact} in {t}");
    }
    assert!(t.contains("github.com/octocat"), "{t}");
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Would clone pave-io/pave"),
        "{}",
        h.text()
    );
}

#[test]
fn clone_destination_validation_keeps_literal_data_and_rejects_line_controls() {
    let literal = r#" spaced/with "quotes" & café "#;
    assert!(dialogs::validate_clone_destination(literal).is_ok());
    assert_eq!(
        dialogs::validate_clone_destination("folder\0name")
            .err()
            .map(|error| error.to_string()),
        Some("Destination cannot contain control characters".into())
    );
    assert_eq!(
        dialogs::validate_clone_destination("folder\nname")
            .err()
            .map(|error| error.to_string()),
        Some("Destination cannot contain control characters".into())
    );
}

fn open_clone_options(h: &mut Harness<App>) {
    let _ = h.type_str("clone");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Clone which repository?"), "{}", h.text());
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Clone pave-io/pave?"), "{}", h.text());
    assert!(h.text().contains("ssh"), "{}", h.text());
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Edit clone options"), "{}", h.text());
    assert_eq!(h.focus(), Some(dialogs::CLONE_DESTINATION));
    assert!(h.text().contains("~/work/pave/pave"), "{}", h.text());
    assert!(h.area_of(dialogs::CLONE_PROTOCOL).is_some());
    assert!(h.area_of(dialogs::CLONE_DESTINATION).is_some());
}

#[test]
fn clone_options_fields_fit_the_short_supported_layout() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 80, 24);
    open_clone_options(&mut h);
    let text = h.text();
    assert!(text.contains("Protocol"), "{text}");
    assert!(text.contains("Destination"), "{text}");
    assert_eq!(h.focus(), Some(dialogs::CLONE_DESTINATION));
    let cancel = Dialog::new(dialogs::DIALOG).action_id(0);
    let _ = h.click_id(cancel);
    assert!(h.text().contains("Clone pave-io/pave?"), "{}", h.text());
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn clone_options_propagate_literal_values_and_cancel_only_the_draft() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();
    open_clone_options(&mut h);

    let _ = h.click_id(dialogs::CLONE_PROTOCOL);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Enter);
    let _ = h.click_id(dialogs::CLONE_DESTINATION);
    let _ = h.ctrl('l');
    let destination = r#"release & notes "café""#;
    let _ = h.paste(destination);
    let review = Dialog::new(dialogs::DIALOG).action_id(1);
    let _ = h.click_id(review);

    let bound = format!("{}/{}", h.app().world.cwd, destination);
    let text = h.text();
    assert!(text.contains("Protocol"), "{text}");
    assert!(text.contains("https"), "{text}");
    assert!(text.contains("Destination"), "{text}");
    assert!(text.contains("release & notes"), "{text}");

    // A later edit is a draft until Review accepts it.
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("release & notes"), "{}", h.text());
    let _ = h.click_id(dialogs::CLONE_DESTINATION);
    let _ = h.ctrl('l');
    let _ = h.paste("discard this draft");
    let cancel = Dialog::new(dialogs::DIALOG).action_id(0);
    let _ = h.click_id(cancel);
    let text = h.text();
    assert!(text.contains("https"), "{text}");
    assert!(text.contains("release & notes"), "{text}");
    assert!(!text.contains("discard this draft"), "{text}");

    let confirm = Dialog::new(dialogs::DIALOG).action_id(1);
    let _ = h.click_id(confirm);
    let expected =
        format!("Would clone pave-io/pave via https to {bound} · simulated, nothing executed");
    assert_eq!(h.app().status.as_deref(), Some(expected.as_str()));
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn clone_options_reject_empty_destination_and_keep_the_field_focused() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();
    open_clone_options(&mut h);
    let _ = h.ctrl('l');
    let _ = h.key(KeyCode::Backspace);
    let review = Dialog::new(dialogs::DIALOG).action_id(1);
    let _ = h.click_id(review);

    let text = h.text();
    assert!(text.contains("Enter a destination"), "{text}");
    assert_eq!(h.focus(), Some(dialogs::CLONE_DESTINATION));
    assert!(matches!(
        h.app().overlay.as_ref(),
        Some(super::Overlay::Dialog(_))
    ));
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn clone_options_revalidate_repo_login_cwd_and_action_before_review() {
    for stale in ["repo", "login", "cwd", "action"] {
        let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
        let before_revision = h.app().world.effect_revision;
        let before_activities = h.app().world.activities.len();
        open_clone_options(&mut h);
        match stale {
            "repo" => {
                if let Some(github) = &mut h.app_mut().world.github {
                    github.repos.retain(|repo| repo.slug() != "pave-io/pave");
                }
            }
            "login" => {
                if let Some(github) = &mut h.app_mut().world.github {
                    github.login.push_str("-changed");
                }
            }
            "cwd" => h.app_mut().world.cwd.push_str("/changed"),
            "action" => h.app_mut().world.discovery.clear(),
            _ => {}
        }
        let review = Dialog::new(dialogs::DIALOG).action_id(1);
        let _ = h.click_id(review);
        assert!(
            h.app()
                .status
                .as_deref()
                .is_some_and(|status| status.contains("Clone target changed")),
            "stale {stale}: {}",
            h.text()
        );
        assert_eq!(h.app().world.effect_revision, before_revision, "{stale}");
        assert_eq!(h.app().world.activities.len(), before_activities, "{stale}");
        assert!(h.diagnostics().is_empty(), "{stale}: {:?}", h.diagnostics());
    }
}

#[test]
fn clone_picker_rejects_context_changed_before_repository_choice() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();
    let _ = h.type_str("clone");
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("Clone which repository?"), "{}", h.text());

    h.app_mut().world.host.env = crate::domain::host::Environment::Production;
    let _ = h.key(KeyCode::Enter);

    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.contains("Clone target changed")),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("Clone pave-io/pave?"), "{}", h.text());
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn clone_confirmation_revalidates_accepted_snapshot() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let before_revision = h.app().world.effect_revision;
    let before_activities = h.app().world.activities.len();
    open_clone_options(&mut h);
    let review = Dialog::new(dialogs::DIALOG).action_id(1);
    let _ = h.click_id(review);
    assert!(h.text().contains("Clone pave-io/pave?"), "{}", h.text());

    if let Some(github) = &mut h.app_mut().world.github {
        github.login.push_str("-changed");
    }
    let confirm = Dialog::new(dialogs::DIALOG).action_id(1);
    let _ = h.click_id(confirm);
    assert!(
        h.app()
            .status
            .as_deref()
            .is_some_and(|status| status.contains("Clone target changed")),
        "{}",
        h.text()
    );
    assert_eq!(h.app().world.effect_revision, before_revision);
    assert_eq!(h.app().world.activities.len(), before_activities);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}

#[test]
fn long_running_task_starts_named_activity() {
    let mut h = fixture(Scenario::MonorepoChild, Motion::Paused, 4_000, 120, 40);
    let before = h.app().world.activities.len();
    let _ = h.type_str("vite");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Activity started: Run dev"), "{t}");
    assert_eq!(Some(h.app().world.activities.len()), before.checked_add(1));
    let activity = h.app().world.activities.last();
    assert!(activity.is_some(), "started activity must exist");
    let Some(a) = activity else { return };
    assert_eq!(a.scope, "~/work/monorepo/apps/frontend");
    assert_eq!(a.state, crate::domain::activity::ActivityState::Running);
}

#[test]
fn launch_failure_is_honest() {
    let mut h = fixture(Scenario::LaunchFailure, Motion::Paused, 4_000, 120, 40);
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("mise run test");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("failed · exit code 1"), "{t}");
    let activity = h.app().world.activities.last();
    assert!(activity.is_some(), "started activity must exist");
    let Some(a) = activity else { return };
    assert_eq!(a.state, crate::domain::activity::ActivityState::Failed);
}

#[test]
fn nothing_matches_state() {
    let mut h = fixture(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let _ = h.type_str("zzz");
    assert!(h.text().contains("Nothing matches"), "{}", h.text());
}

#[test]
fn priority_stack_has_reasons_and_scope_tags() {
    let h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    assert!(t.contains("Suggested here"), "{t}");
    assert!(t.contains("Recent here"), "{t}");
    assert!(t.contains("Explore"), "{t}");

    assert!(t.contains("pinned here"), "{t}");
    assert!(t.contains("used 6 times in this project"), "{t}");
    assert!(t.contains("branch is 3 commits behind"), "{t}");

    assert!(t.contains("· host"), "{t}");
}

#[test]
fn preview_answers_the_seven_questions() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.ctrl('p');
    let t = h.text();
    for fact in [
        "Will happen",
        "Target",
        "Why recommended",
        "Will change",
        "Freshness",
        "Confirmation",
    ] {
        assert!(t.contains(fact), "missing {fact} in {t}");
    }
    assert!(t.contains("Run (simulated)"), "{t}");
    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("Will happen"));
}

#[test]
fn ssh_resolution_previews_chain_and_identity_filename() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("prod-eu-1");
    let _ = h.ctrl('p');
    let t = h.text();
    assert!(t.contains("Connect to prod-eu-1?"), "{t}");
    assert!(t.contains("10.20.30.40"), "{t}");
    assert!(t.contains("bastion → prod-eu-1"), "{t}");

    assert!(t.contains("id_ed25519_prod · filename only"), "{t}");
    assert!(t.contains("Multiplexing"), "{t}");
}

#[test]
fn monitor_snapshot_facts_and_btm_handoff() {
    let mut h = fixture(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("monitor");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("System snapshot"), "{t}");
    assert!(t.contains("prod-eu-1"), "{t}");
    assert!(t.contains("Load"), "{t}");
    assert!(t.contains("3.10 2.80 2.50"), "{t}");
    assert!(t.contains("14200 MB of 16384 MB"), "{t}");
    assert!(t.contains("212 days"), "{t}");
    assert!(t.contains("btm takes over the screen"), "{t}");
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(h.text().contains("simulated handoff"), "{}", h.text());
}

#[test]
fn quit_confirm_on_production_names_the_remote_identity() {
    let mut h = fixture(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    let _ = h.key(KeyCode::Char('q'));
    let t = h.text();
    assert!(t.contains("Quit holla on prod-eu-1?"), "{t}");
    assert!(t.contains("◆ prod-eu-1"), "{t}");

    let _ = h.key(KeyCode::Enter);
    assert!(!h.app().quit);
}

#[test]
fn pg_lock_tree_cancel_resumes_waiters() {
    let mut h = fixture(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("locks");
    let _ = h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Database lock tree"), "{t}");
    assert!(t.contains("pid 4201 · payments · ALTER TABLE"), "{t}");
    assert!(t.contains("cancel before terminate"), "{t}");

    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Cancelled pid 4201 · 2 waiting sessions resumed"),
        "{}",
        h.text()
    );

    let sessions = h.app().world.pg.as_ref();
    assert!(sessions.is_some(), "fixture sessions must exist");
    let Some(sessions) = sessions else { return };
    assert!(!sessions.iter().any(|s| s.pid == 4201));
    assert!(sessions.iter().all(|s| s.blocked_by.is_none()));

    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("locks");
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text().contains("No blockers · sessions healthy"),
        "{}",
        h.text()
    );
}

#[test]
fn pg_terminate_revalidates_before_killing() {
    let mut h = fixture(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("locks");
    let _ = h.key(KeyCode::Enter);

    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Terminated pid 4201 after revalidation · 2 waiting sessions resumed"),
        "{}",
        h.text()
    );
    let sessions = h.app().world.pg.as_ref();
    assert!(sessions.is_some(), "fixture sessions must exist");
    let Some(sessions) = sessions else { return };
    assert!(!sessions.iter().any(|s| s.pid == 4201));
}

#[test]
fn git_sync_plan_resolves_per_child_primary_branches() {
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("update all child");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Plan, "{}", h.text());
    let t = h.text();

    assert!(t.contains("Check out main (legacy)"), "{t}");
    assert!(t.contains("Check out trunk (billing)"), "{t}");

    assert!(t.contains("detached at a1b2c3d · skipped"), "{t}");

    let _ = h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("UPDATE ALL CHILD PROJECTS IN ~/work/monorepo"),
        "{}",
        h.text()
    );
}

#[test]
fn disk_reclaim_plan_matches_apply_effect() {
    let mut h = fixture(Scenario::DiskCleanup, Motion::Paused, 4_000, 120, 40);
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("reclaim disk");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Plan, "{}", h.text());
    let disk = h.app().world.disk.as_ref();
    assert!(disk.is_some());
    let Some(disk) = disk else { return };
    let before = disk.candidates.len();
    let review = h.app().plan.as_ref();
    assert!(review.is_some());
    let Some(review) = review else { return };
    let removable = review
        .plan()
        .steps()
        .iter()
        .filter(|s| {
            s.id.starts_with("rm:")
                && !matches!(s.state, crate::domain::plan::StepState::PolicySkipped(_))
        })
        .count();
    assert!(removable >= 1);
    let t = h.text();
    assert!(t.contains("inactive 31 days"), "{t}");
    assert!(t.contains("active today · never removed"), "{t}");
    let phrase = review.plan().phrase().to_owned();
    let _ = h.key(KeyCode::Enter);
    let _ = h.type_str(&phrase);
    let _ = h.key(KeyCode::Enter);
    let _ = h.key(KeyCode::Right);
    let _ = h.key(KeyCode::Enter);
    let disk = h.app().world.disk.as_ref();
    assert!(disk.is_some());
    let Some(disk) = disk else { return };
    assert_eq!(
        Some(disk.candidates.len()),
        before.checked_sub(removable),
        "dry-run steps ↔ removals parity"
    );
}

#[test]
fn git_sync_run_per_child_results_and_honest_effect() {
    let mut h = fixture(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("update all child");
    let _ = h.key(KeyCode::Enter);
    let billing_before = h
        .app()
        .world
        .git
        .as_ref()
        .and_then(|git| git.children.iter().find(|c| c.root.ends_with("billing")))
        .cloned();
    assert!(billing_before.is_some());
    try_confirm(&mut h, "UPDATE ALL CHILD PROJECTS IN ~/work/monorepo");
    let t = h.text();
    assert!(t.contains("diverged · 2 ahead, 3 behind"), "{t}");
    assert!(t.contains("✓ Pull legacy — fast-forward only"), "{t}");

    assert!(t.contains("− Pull billing"), "{t}");
    assert!(
        t.contains("diverged · 2 ahead, 3 behind · untouched"),
        "{t}"
    );
    let git = h.app().world.git.as_ref();
    assert!(git.is_some());
    let Some(git) = git else { return };
    let legacy = git.children.iter().find(|c| c.root.ends_with("legacy"));
    let billing = git.children.iter().find(|c| c.root.ends_with("billing"));
    assert!(legacy.is_some());
    assert!(billing.is_some());
    let (Some(legacy), Some(billing)) = (legacy, billing) else {
        return;
    };
    assert_eq!(legacy.behind, 0);
    assert_eq!(
        billing.behind, 3,
        "policy-skipped pull keeps its behind count"
    );
    assert_eq!(billing.primary_branch, "trunk");
    assert_eq!(Some(billing), billing_before.as_ref());
}

fn assert_hidden_port_fixture_retains_border_frame(scenario: Scenario) {
    let mut world = crate::sim::fixtures::world_for(scenario);
    world.seek(4_000);

    let raw_before = crate::sim::catalogue::catalogue(&world);
    let port_actions: Vec<_> = raw_before
        .iter()
        .filter(|action| action.id == "system.port")
        .collect();
    assert_eq!(
        port_actions.len(),
        1,
        "canonical Port action in raw catalogue for {scenario:?}"
    );
    let port_command = port_actions
        .first()
        .map(|action| action.command.clone())
        .unwrap_or_default();
    let normal_rows = home_display_rows(&world);
    assert_eq!(
        normal_rows
            .iter()
            .filter(|(_, id)| id == "system.port")
            .count(),
        1,
        "Port is normally visible for {scenario:?}"
    );

    let original_hides = world.memory.hides.len();
    world.memory.hides.push(Pin {
        path: world.cwd.clone(),
        command: port_command.clone(),
    });
    assert_eq!(world.memory.hides.len(), original_hides.saturating_add(1));
    let added_hide = world.memory.hides.last();
    assert!(added_hide.is_some(), "local Port hide was inserted");
    let Some(added_hide) = added_hide else { return };
    assert_eq!(added_hide.path, world.cwd, "hide uses fixture cwd");
    assert_eq!(
        added_hide.command, port_command,
        "hide uses canonical command"
    );

    let raw_after = crate::sim::catalogue::catalogue(&world);
    assert_eq!(
        raw_after, raw_before,
        "raw catalogue remains unchanged for {scenario:?}"
    );
    assert_eq!(
        raw_after
            .iter()
            .filter(|action| action.id == "system.port")
            .count(),
        1,
        "local hide leaves Port in the raw catalogue for {scenario:?}"
    );

    let historical_rows = home_display_rows(&world);
    let expected_rows: Vec<_> = normal_rows
        .iter()
        .filter(|(_, id)| id != "system.port")
        .cloned()
        .collect();
    assert_eq!(
        historical_rows, expected_rows,
        "only Port is hidden for {scenario:?}"
    );
    assert_eq!(
        normal_rows.len(),
        historical_rows.len().saturating_add(1),
        "one displayed action is hidden for {scenario:?}"
    );
    if scenario == Scenario::RustDirty {
        assert_eq!(normal_rows.len(), 16, "normal RustDirty action count");
        assert_eq!(
            historical_rows.len(),
            15,
            "historical RustDirty action count"
        );
        assert_eq!(home_visual_extent(&historical_rows), 20);
    }

    let source_theme = Theme::junie();
    let mut app = App::from_fixture_world(world, Motion::Paused, 4_000);
    app.home.history = false;
    let app = app.with_source_theme(source_theme.clone());
    let frame = u64::try_from(app.fixture_time_ms());
    assert!(frame.is_ok(), "fixture time must be nonnegative");
    let initial = SimulationMoment::from_millis(frame.unwrap_or_default());
    let mut h = Harness::new_with_feedback_clock(
        app,
        source_theme,
        80,
        24,
        FeedbackClock::Simulation { initial },
    )
    .with_color(ColorLevel::Ansi256);

    let rows = crate::screens::home::ROWS;
    let rows_area = h.area_of(rows);
    let track = h.area_of_part(rows, PartRef::of(Part::TRACK));
    let thumb = h.area_of_part(rows, PartRef::of(Part::THUMB));
    assert!(
        rows_area.is_some(),
        "native Home rows area for {scenario:?}"
    );
    assert!(track.is_some(), "native Home track for {scenario:?}");
    assert!(thumb.is_some(), "native Home thumb for {scenario:?}");
    let (Some(rows_area), Some(track), Some(thumb)) = (rows_area, track, thumb) else {
        return;
    };

    assert_eq!(
        (rows_area.x, rows_area.y, rows_area.width, rows_area.height),
        (1, 6, 78, 15),
        "native Home rows viewport for {scenario:?}"
    );
    assert_eq!(
        (track.x, track.y, track.width, track.height),
        (78, 6, 1, 15),
        "native Home track for {scenario:?}"
    );
    assert_eq!(thumb.x, track.x, "thumb column for {scenario:?}");
    assert_eq!(
        thumb.y, track.y,
        "initial zero-offset thumb for {scenario:?}"
    );
    assert_eq!(thumb.width, track.width, "thumb width for {scenario:?}");

    let visual_extent = home_visual_extent(&historical_rows);
    let viewport = usize::from(rows_area.height);
    let track_len = usize::from(track.height);
    assert!(visual_extent > 0, "nonempty Home model for {scenario:?}");
    assert_eq!(viewport, track_len, "native viewport and track length");
    let expected_thumb_len = viewport
        .saturating_mul(track_len)
        .checked_div(visual_extent)
        .unwrap_or_default()
        .max(1)
        .min(track_len);
    assert_eq!(
        usize::from(thumb.height),
        expected_thumb_len,
        "ScrollState thumb from model extent for {scenario:?}"
    );

    let exposed = exposed_track_cells(track, thumb);
    let expected_first_exposed_y = match scenario {
        Scenario::RustDirty | Scenario::MonorepoChild => Some(17),
        Scenario::MonorepoRoot | Scenario::ActivitiesMulti => Some(18),
        _ => None,
    };
    assert!(expected_first_exposed_y.is_some(), "expected held case");
    let expected_first_exposed_y = expected_first_exposed_y.unwrap_or_default();
    let expected_exposed: Vec<_> = (expected_first_exposed_y..=20).map(|y| (78, y)).collect();
    assert_eq!(
        exposed, expected_exposed,
        "model-derived track for {scenario:?}"
    );

    let current = h.buffer().clone();
    let current_cursor = h.cursor();
    assert_eq!(current_cursor, Some(Position { x: 3, y: 4 }));
    for &(x, y) in &exposed {
        let cell = current.cell((x, y));
        assert!(cell.is_some(), "track cell ({x},{y}) exists");
        let Some(cell) = cell else { continue };
        assert_eq!(cell.symbol(), "┆", "track glyph at ({x},{y})");
        assert_eq!(cell.fg, Color::Indexed(238), "track color at ({x},{y})");
        assert_eq!(cell.bg, Color::Indexed(16), "track background at ({x},{y})");
    }
    for y in thumb.y..thumb.y.saturating_add(thumb.height) {
        let cell = current.cell((thumb.x, y));
        assert!(cell.is_some(), "thumb cell ({},{y}) exists", thumb.x);
        let Some(cell) = cell else { continue };
        assert_eq!(cell.symbol(), "┃", "thumb glyph at ({},{y})", thumb.x);
        assert_eq!(
            cell.fg,
            Color::Indexed(245),
            "thumb color at ({},{y})",
            thumb.x
        );
    }

    let mut historical_theme = h.runtime().theme().clone();
    historical_theme.color.border_subtle = Color::Indexed(235);
    h.runtime_mut().set_theme(historical_theme);
    h.draw();
    assert_eq!(h.cursor(), current_cursor, "cursor after palette replay");
    h.snapshot()
        .named(scenario.name())
        .assert_against(&BORDER_BASELINE);
}

#[test]
fn rust_dirty_hidden_port_retains_historical_border_frame() {
    assert_hidden_port_fixture_retains_border_frame(Scenario::RustDirty);
}

#[test]
fn monorepo_root_hidden_port_retains_historical_border_frame() {
    assert_hidden_port_fixture_retains_border_frame(Scenario::MonorepoRoot);
}

#[test]
fn monorepo_root_truecolor_port_visibility_ab_characterization() {
    let mut world_a = crate::sim::fixtures::world_for(Scenario::MonorepoRoot);
    let mut world_b = crate::sim::fixtures::world_for(Scenario::MonorepoRoot);
    world_a.seek(4_000);
    world_b.seek(4_000);

    let raw_a = crate::sim::catalogue::catalogue(&world_a);
    let raw_b = crate::sim::catalogue::catalogue(&world_b);
    assert_eq!(
        raw_a, raw_b,
        "complete raw catalogues before the intervention"
    );
    assert_eq!(world_a.memory, world_b.memory, "initial Memory values");
    assert_eq!(world_a.cwd, world_b.cwd, "initial cwd values");
    assert_eq!(
        world_a.scenario, world_b.scenario,
        "initial scenario values"
    );
    assert_eq!(world_a.clock, world_b.clock, "initial simulation clocks");

    let rows_a = home_display_rows(&world_a);
    let rows_b_before = home_display_rows(&world_b);
    assert_eq!(
        rows_a, rows_b_before,
        "ordered displayed rows before the intervention"
    );
    let extent_a = home_visual_extent(&rows_a);
    let extent_b_before = home_visual_extent(&rows_b_before);
    assert_eq!(
        extent_a, extent_b_before,
        "visual extents before the intervention"
    );
    let port_actions: Vec<_> = raw_b
        .iter()
        .filter(|action| action.id == "system.port")
        .collect();
    assert_eq!(
        port_actions.len(),
        1,
        "one actual system.port catalogue action"
    );
    let Some(port_action) = port_actions.first() else {
        return;
    };
    let port_pin = Pin {
        path: world_b.cwd.clone(),
        command: port_action.command.clone(),
    };
    let memory_b_before = world_b.memory.clone();
    let mut expected_memory_b = memory_b_before.clone();
    expected_memory_b.hides.push(port_pin.clone());
    world_b.memory.hides.push(port_pin.clone());
    assert_eq!(
        world_b.memory, expected_memory_b,
        "B adds only the actual Port hide"
    );
    assert_eq!(port_pin.path, world_b.cwd, "B hide uses the fixture cwd");
    assert_eq!(
        port_pin.command, port_action.command,
        "B hide uses actual command text"
    );

    let raw_b_after = crate::sim::catalogue::catalogue(&world_b);
    assert_eq!(
        raw_b_after, raw_a,
        "raw catalogue and order after local hide"
    );
    assert_eq!(
        raw_b_after
            .iter()
            .filter(|action| action.id == "system.port")
            .count(),
        1,
        "Port remains in B's raw catalogue"
    );
    assert!(
        rows_a
            .iter()
            .any(|(_, action_id)| action_id == "system.port"),
        "A keeps Port in its visible model"
    );
    let rows_b = home_display_rows(&world_b);
    let expected_rows_b: Vec<_> = rows_a
        .iter()
        .filter(|(_, action_id)| action_id != "system.port")
        .cloned()
        .collect();
    assert_eq!(
        rows_b, expected_rows_b,
        "B removes only Port from displayed order"
    );
    assert!(
        rows_b
            .iter()
            .all(|(_, action_id)| action_id != "system.port"),
        "B removes Port from its visible model"
    );
    assert_eq!(
        rows_a.len(),
        rows_b.len().saturating_add(1),
        "B hides exactly one displayed action"
    );
    let extent_b = home_visual_extent(&rows_b);

    let source_theme = Theme::junie();
    let color = ColorLevel::TrueColor;
    let make_harness = |world| {
        let mut app = App::from_fixture_world(world, Motion::Paused, 4_000);
        app.home.history = false;
        let app = app.with_source_theme(source_theme.clone());
        Harness::new_with_feedback_clock(
            app,
            source_theme.clone(),
            80,
            24,
            FeedbackClock::Simulation {
                initial: SimulationMoment::from_millis(4_000),
            },
        )
        .with_color(color)
    };
    let harness_a = make_harness(world_a);
    let harness_b = make_harness(world_b);

    let expected_runtime_theme = source_theme.for_level(color);
    let mut model_geometry = [None, None];
    for (arm_index, (label, harness, expected_memory, expected_rows, expected_extent)) in [
        ("A", &harness_a, memory_b_before, &rows_a, extent_a),
        ("B", &harness_b, expected_memory_b, &rows_b, extent_b),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(harness.snapshot().key(), "harness 80 24 junie truecolor");
        assert_eq!(harness.runtime().theme(), &expected_runtime_theme);
        assert_eq!(
            harness
                .app()
                .source_theme
                .as_ref()
                .map(|theme| &theme.theme),
            Some(&source_theme),
            "{label} retains the Junie source theme"
        );
        assert_eq!(harness.app().world.scenario, Scenario::MonorepoRoot);
        assert_eq!(harness.app().world.clock.now_ms, 4_000);
        assert!(!harness.app().world.clock.running, "{label} remains paused");
        assert_eq!(harness.app().motion, Motion::Paused);
        assert_eq!(harness.app().fixture_time_ms(), 4_000);
        assert!(!harness.app().home.history, "{label} uses no-history input");
        assert_eq!(
            harness.app().world.memory,
            expected_memory,
            "{label} Memory"
        );
        assert_eq!(
            home_display_rows(&harness.app().world),
            *expected_rows,
            "{label} displayed row order"
        );
        assert_eq!(
            home_visual_extent(&home_display_rows(&harness.app().world)),
            expected_extent,
            "{label} model-derived visual extent"
        );
        assert!(
            expected_extent > expected_rows.len(),
            "{label} model includes its initial section heading"
        );
        assert!(
            !expected_rows.is_empty(),
            "{label} model must expose a first Home action"
        );
        let Some((first_section, first_action_id)) = expected_rows.first() else {
            return;
        };
        assert!(
            !first_section.is_empty(),
            "{label} first model section name"
        );
        assert_eq!(harness.cursor(), Some(Position { x: 3, y: 4 }));

        let rows_area = harness.area_of(crate::screens::home::ROWS);
        let track = harness.area_of_part(crate::screens::home::ROWS, PartRef::of(Part::TRACK));
        let thumb = harness.area_of_part(crate::screens::home::ROWS, PartRef::of(Part::THUMB));
        assert!(rows_area.is_some(), "{label} native Home rows area");
        assert!(track.is_some(), "{label} native Home track");
        assert!(thumb.is_some(), "{label} native Home thumb");
        let (Some(rows_area), Some(track), Some(thumb)) = (rows_area, track, thumb) else {
            return;
        };
        assert_eq!(
            (rows_area.x, rows_area.y, rows_area.width, rows_area.height),
            (1, 6, 78, 15),
            "{label} native Home rows geometry"
        );
        assert_eq!(
            (track.x, track.y, track.width, track.height),
            (78, 6, 1, 15),
            "{label} native track geometry"
        );
        assert_eq!(thumb.x, track.x, "{label} thumb column");
        assert_eq!(thumb.y, track.y, "{label} initial zero-offset thumb");
        assert_eq!(thumb.width, track.width, "{label} thumb width");
        let first_row_area = harness.area_of_part(
            crate::screens::home::ROWS,
            PartRef::item(Part::ROW, termrock::ItemKey::text(first_action_id)),
        );
        assert!(first_row_area.is_some(), "{label} first keyed action row");
        let Some(first_row_area) = first_row_area else {
            return;
        };
        assert_eq!(first_row_area.x, rows_area.x, "{label} first action row x");
        assert_eq!(first_row_area.height, 1, "{label} first action row height");
        let expected_first_action_y = rows_area.y.saturating_add(1);
        assert_eq!(
            first_row_area.y, expected_first_action_y,
            "{label} first section header occupies logical row 0"
        );
        let projected_offset = expected_first_action_y.checked_sub(first_row_area.y);
        assert_eq!(
            projected_offset,
            Some(0),
            "{label} drawn Home projection offset"
        );
        let viewport = usize::from(rows_area.height);
        let track_len = usize::from(track.height);
        let expected_thumb_len = viewport
            .saturating_mul(track_len)
            .checked_div(expected_extent)
            .unwrap_or_default()
            .max(1)
            .min(track_len);
        assert_eq!(
            usize::from(thumb.height),
            expected_thumb_len,
            "{label} native thumb from model extent"
        );
        let expected_thumb_height = u16::try_from(expected_thumb_len);
        assert!(
            expected_thumb_height.is_ok(),
            "{label} model thumb height fits native geometry"
        );
        let model_thumb = ratatui::layout::Rect::new(
            track.x,
            track.y,
            track.width,
            expected_thumb_height.unwrap_or_default(),
        );
        assert_eq!(thumb, model_thumb, "{label} model-derived thumb rectangle");
        model_geometry[arm_index] = Some((track, model_thumb));
    }

    let Some((track_a, model_thumb_a)) = model_geometry[0] else {
        return;
    };
    let Some((track_b, model_thumb_b)) = model_geometry[1] else {
        return;
    };
    assert_eq!(track_a, track_b, "A/B native track rectangles");

    let area_a = *harness_a.buffer().area();
    let area_b = *harness_b.buffer().area();
    assert_eq!(area_a, area_b, "A/B complete frame dimensions");
    let mut differing_coordinates = Vec::new();
    for y in area_a.y..area_a.y.saturating_add(area_a.height) {
        for x in area_a.x..area_a.x.saturating_add(area_a.width) {
            let cell_a = harness_a.buffer().cell((x, y));
            let cell_b = harness_b.buffer().cell((x, y));
            assert!(cell_a.is_some(), "A cell ({x},{y}) exists");
            assert!(cell_b.is_some(), "B cell ({x},{y}) exists");
            let (Some(cell_a), Some(cell_b)) = (cell_a, cell_b) else {
                return;
            };
            if cell_a != cell_b {
                differing_coordinates.push((x, y));
            }
        }
    }

    let coordinates_in_rect = |rect: ratatui::layout::Rect| {
        let mut coordinates = Vec::new();
        for y in rect.y..rect.y.saturating_add(rect.height) {
            for x in rect.x..rect.x.saturating_add(rect.width) {
                coordinates.push((x, y));
            }
        }
        coordinates
    };
    let thumb_coordinates_a = coordinates_in_rect(model_thumb_a);
    let thumb_coordinates_b = coordinates_in_rect(model_thumb_b);
    let thumb_only_a: Vec<_> = thumb_coordinates_a
        .iter()
        .copied()
        .filter(|coordinate| !thumb_coordinates_b.contains(coordinate))
        .collect();
    let thumb_only_b: Vec<_> = thumb_coordinates_b
        .iter()
        .copied()
        .filter(|coordinate| !thumb_coordinates_a.contains(coordinate))
        .collect();
    let mut expected_difference = thumb_only_a.clone();
    expected_difference.extend(thumb_only_b.iter().copied());
    expected_difference.sort_unstable();
    differing_coordinates.sort_unstable();
    assert!(
        !expected_difference.is_empty(),
        "model-derived thumb symmetric difference is nonempty"
    );
    assert_eq!(
        differing_coordinates, expected_difference,
        "complete Cell differences equal only the model-derived thumb symmetric difference"
    );

    let shared_thumb_coordinates: Vec<_> = thumb_coordinates_a
        .iter()
        .copied()
        .filter(|coordinate| thumb_coordinates_b.contains(coordinate))
        .collect();
    let exposed_track_a = exposed_track_cells(track_a, model_thumb_a);
    let exposed_track_b = exposed_track_cells(track_b, model_thumb_b);
    let shared_exposed_track_coordinates: Vec<_> = exposed_track_a
        .iter()
        .copied()
        .filter(|coordinate| exposed_track_b.contains(coordinate))
        .collect();
    assert!(
        !shared_thumb_coordinates.is_empty(),
        "shared model-derived thumb reference region is nonempty"
    );
    assert!(
        !shared_exposed_track_coordinates.is_empty(),
        "shared model-derived exposed-track reference region is nonempty"
    );

    let thumb_reference_position = shared_thumb_coordinates.first().copied();
    assert!(
        thumb_reference_position.is_some(),
        "shared thumb reference position exists"
    );
    let Some(thumb_reference_position) = thumb_reference_position else {
        return;
    };
    let thumb_reference = harness_a.buffer().cell(thumb_reference_position);
    assert!(
        thumb_reference.is_some(),
        "shared thumb reference cell exists"
    );
    let Some(thumb_reference) = thumb_reference else {
        return;
    };

    let exposed_track_reference_position = shared_exposed_track_coordinates.first().copied();
    assert!(
        exposed_track_reference_position.is_some(),
        "shared exposed-track reference position exists"
    );
    let Some(exposed_track_reference_position) = exposed_track_reference_position else {
        return;
    };
    let exposed_track_reference = harness_a.buffer().cell(exposed_track_reference_position);
    assert!(
        exposed_track_reference.is_some(),
        "shared exposed-track reference cell exists"
    );
    let Some(exposed_track_reference) = exposed_track_reference else {
        return;
    };

    for &coordinate in &shared_thumb_coordinates {
        assert_eq!(
            harness_a.buffer().cell(coordinate),
            Some(thumb_reference),
            "A shared thumb Cell fields at {coordinate:?}"
        );
        assert_eq!(
            harness_b.buffer().cell(coordinate),
            Some(thumb_reference),
            "B shared thumb Cell fields at {coordinate:?}"
        );
    }
    for &coordinate in &shared_exposed_track_coordinates {
        assert_eq!(
            harness_a.buffer().cell(coordinate),
            Some(exposed_track_reference),
            "A shared exposed-track Cell fields at {coordinate:?}"
        );
        assert_eq!(
            harness_b.buffer().cell(coordinate),
            Some(exposed_track_reference),
            "B shared exposed-track Cell fields at {coordinate:?}"
        );
    }

    for &coordinate in &thumb_only_a {
        assert_eq!(
            harness_a.buffer().cell(coordinate),
            Some(thumb_reference),
            "A thumb Cell fields at {coordinate:?}"
        );
        assert_eq!(
            harness_b.buffer().cell(coordinate),
            Some(exposed_track_reference),
            "B exposed-track Cell fields at {coordinate:?}"
        );
    }
    for &coordinate in &thumb_only_b {
        assert_eq!(
            harness_a.buffer().cell(coordinate),
            Some(exposed_track_reference),
            "A exposed-track Cell fields at {coordinate:?}"
        );
        assert_eq!(
            harness_b.buffer().cell(coordinate),
            Some(thumb_reference),
            "B thumb Cell fields at {coordinate:?}"
        );
    }

    assert_eq!(harness_a.cursor(), Some(Position { x: 3, y: 4 }));
    assert_eq!(harness_b.cursor(), Some(Position { x: 3, y: 4 }));
    let scene_b = harness_b.snapshot();
    let digest_b = scene_b.digest();
    assert!(
        digest_b == 0x5bf3_beea_13a9_33dd,
        "B must retain the unchanged held digest when local Port hiding is sufficient"
    );
}

#[test]
fn monorepo_child_hidden_port_retains_historical_border_frame() {
    assert_hidden_port_fixture_retains_border_frame(Scenario::MonorepoChild);
}

#[test]
fn activities_multi_hidden_port_retains_historical_border_frame() {
    assert_hidden_port_fixture_retains_border_frame(Scenario::ActivitiesMulti);
}
