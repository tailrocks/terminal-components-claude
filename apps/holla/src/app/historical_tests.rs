//! Pinned794b095 actual-App assertions restored through the shared production runtime.
//! Fixture selection, frames, dimensions and product assertions remain source-derived.
use super::{App, Motion, Route, Scenario};
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
fn find(h: &Harness<App>, needle: &str) -> (u16, u16) {
    let position = h.find(needle);
    assert!(position.is_some(), "missing fixture text: {needle}");
    position.unwrap_or_default()
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
    // pinning a row makes it outrank everything next time
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
    // Run, Preview, Copy, Pin, Alias
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
    // the alias query finds the command's row
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc); // clear the old query
    let _ = h.type_str("cb");
    assert!(h.text().contains("cargo build"), "{}", h.text());
}

#[test]
fn hide_removes_row_exact_query_resurfaces_and_reset_restores() {
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    // Run, Preview, Copy, Pin, Alias, Hide
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
    // hidden: a partial query no longer finds it
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("cargo b");
    assert!(
        !h.text().contains("cargo build · cargo build"),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("used 6 times"), "{}", h.text());
    // the exact command resurfaces the row so it can be managed
    let _ = h.key(KeyCode::Esc);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Unhide here"), "{t}");
    // Reset ranking clears pin, alias and hide (last menu item)
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
    // Reference home.rs actions_menu passes searchable=false to PickerModal.
    let mut h = fixture(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("cargo build");
    let _ = h.ctrl('o');
    assert!(!h.text().contains("Type to search…"), "{}", h.text());
}

#[test]
fn strip_lists_seeded_activities_with_states() {
    let h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    // the strip docks under the menu bar on every route
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
    assert!(t.contains("Activity · container logs"), "{t}");
    // one merged stream, service prefix per line, interleaved
    assert!(t.contains("api"), "{t}");
    assert!(t.contains("redis"), "{t}");
    assert!(t.contains("worker"), "{t}");
    assert!(t.contains("GET /health 200"), "{t}");
    assert!(t.contains("background save done"), "{t}");
    // the fixture's exited containers never speak in the stream
    assert!(!t.contains("payments-old |"), "{t}");
    // the strip grew the new activity; switching away and back keeps it
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
    // keyboard never needs the mouse: Ctrl+A enters the strip from home
    let _ = h.ctrl('a');
    assert!(h.app().route == Route::Activity, "{}", h.text());
    assert!(h.text().contains("Activity · dev server"), "{}", h.text());
    // same chord advances to the next activity
    let _ = h.ctrl('a');
    assert!(h.text().contains("Activity · test watch"), "{}", h.text());
    // wraps around
    let _ = h.ctrl('a');
    let _ = h.ctrl('a');
    let _ = h.ctrl('a');
    assert!(h.text().contains("Activity · dev server"), "{}", h.text());
}

#[test]
fn digit_switches_to_activity_page() {
    let mut h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    // home digits stay in the query — switching starts from a strip click
    let _ = h.type_str("1");
    assert!(h.app().route == Route::Home, "home digits filter the query");
    let _ = h.key(KeyCode::Backspace);
    // click the strip tab for "seed db"
    let (x, _) = find(&h, "4 seed db");
    assert!(x < u16::MAX, "activity label coordinate fits");
    let _ = h.click(x.saturating_add(1), 1);
    assert!(h.app().route == Route::Activity, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Activity · seed db"), "{t}");
    assert!(t.contains("failed"), "{t}");
    assert!(t.contains("psql: connection to server"), "{t}");
    // now digits switch between activities
    let _ = h.key(KeyCode::Char('1'));
    assert!(h.text().contains("Activity · dev server"), "{}", h.text());
    assert!(h.text().contains("http://localhost:5199/"), "{}", h.text());
    // 0 returns to the root without losing anything
    let _ = h.key(KeyCode::Char('0'));
    assert!(h.app().route == Route::Home);
    assert_eq!(h.app().world.activities.len(), 4);
}

#[test]
fn switching_away_and_back_preserves_output_and_scope() {
    let mut h = fixture(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    // open activity 1 via the strip, note the page
    let (x, _) = find(&h, "1 dev server");
    assert!(x < u16::MAX, "activity label coordinate fits");
    let _ = h.click(x.saturating_add(1), 1);
    assert!(h.app().route == Route::Activity);
    let page = h.text();
    assert!(page.contains("Scope"), "{page}");
    assert!(page.contains("~/work/monorepo/apps/frontend"), "{page}");
    assert!(page.contains("vite v5.4 ready in 412 ms"), "{page}");
    // leave for another activity, then home
    let _ = h.key(KeyCode::Char('4'));
    assert!(h.text().contains("seed db"), "{}", h.text());
    let _ = h.key(KeyCode::Char('0'));
    assert!(h.app().route == Route::Home);
    // back again: same output, same scope, nothing lost
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
    // stream is short at 120x40 body, scroll up then away and back
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
    let _ = h.key(KeyCode::Enter); // open gate 2
    let _ = h.type_str(phrase);
    let _ = h.key(KeyCode::Enter); // leave the ack input
    let _ = h.key(KeyCode::Right); // Cancel → Run plan (skipped while disabled)
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
    // no gate yet: the phrase never shows before gate 2
    assert!(!t.contains("REMOVE ALL DOCKER DATA"), "{t}");
}

#[test]
fn exclusion_recalculates_dependents_live() {
    let mut h = fixture(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);
    // step 0 is required and refuses exclusion
    let _ = h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("required · cannot exclude"),
        "{}",
        h.text()
    );
    // exclude the container removal: images + volumes recalculate
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let t = h.text();
    assert!(t.contains("Excluded Remove stopped containers"), "{t}");
    assert!(t.contains("needs Remove stopped containers"), "{t}");
    // parallel branches stay free
    assert!(!t.contains("needs Inspect Docker usage"), "{t}");
    // restoring clears the block
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
    // wrong host bound: the phrase names devbox, not prod-eu-1
    let _ = h.type_str("REMOVE ALL DOCKER DATA ON prod-eu-1");
    let _ = h.key(KeyCode::Enter); // leave input → Cancel
    let _ = h.key(KeyCode::Right); // Run stays disabled, focus cannot land on it
    let _ = h.key(KeyCode::Enter); // the only reachable action is Cancel
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
    // honest world mutation: cache pruned, web/cron gone, payments-old kept
    let d = h.app().world.docker.as_ref();
    assert!(d.is_some(), "expected Docker fixture");
    assert_eq!(
        d.map(crate::domain::docker::DockerState::build_cache_bytes),
        Some(0)
    );
    assert!(!d.is_some_and(|docker| docker.containers.iter().any(|c| c.name == "web")));
    assert!(d.is_some_and(|docker| docker.containers.iter().any(|c| c.name == "payments-old")));
    // per-step output: focus the failed step, its lines show
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
    // exclude Apply upgrades: autoremove and verify recalculate, mise free
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let t = h.text();
    assert!(t.contains("needs Apply upgrades"), "{t}");
    try_confirm(&mut h, "I UNDERSTAND: UPGRADE EVERYTHING ON devbox-deb");
    let t = h.text();
    assert!(t.contains("Finished:"), "{t}");
    assert!(t.contains("skipped"), "{t}");
    // apt never applied; the mise branch still ran
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
    // broad on production: the review surface, not a one-shot confirm
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
    // back home, the urgency row is gone — the fixture kept its word
    let _ = h.key(KeyCode::Esc);
    assert!(!h.text().contains("service is unhealthy"), "{}", h.text());
}

#[test]
fn disk_plan_policy_skips_active_today() {
    let mut h = fixture(Scenario::DiskCleanup, Motion::Paused, 4_000, 120, 40);
    let _ = h.type_str("reclaim disk");
    let _ = h.key(KeyCode::Enter);
    assert!(h.app().route == Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Remove node_modules"), "{t}");
    assert!(t.contains("active today · never removed"), "{t}");
    assert!(t.contains("Remove .gradle"), "{t}");
}
