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

fn assert_model_thumb_locality(
    frame_a: &ratatui::buffer::Buffer,
    frame_b: &ratatui::buffer::Buffer,
    track: ratatui::layout::Rect,
    model_thumb_a: ratatui::layout::Rect,
    model_thumb_b: ratatui::layout::Rect,
) {
    let area_a = *frame_a.area();
    let area_b = *frame_b.area();
    assert_eq!(area_a, area_b, "A/B complete frame dimensions");
    let mut differing_coordinates = Vec::new();
    for y in area_a.y..area_a.y.saturating_add(area_a.height) {
        for x in area_a.x..area_a.x.saturating_add(area_a.width) {
            let cell_a = frame_a.cell((x, y));
            let cell_b = frame_b.cell((x, y));
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
    let exposed_track_a = exposed_track_cells(track, model_thumb_a);
    let exposed_track_b = exposed_track_cells(track, model_thumb_b);
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
    let thumb_reference = frame_a.cell(thumb_reference_position);
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
    let exposed_track_reference = frame_a.cell(exposed_track_reference_position);
    assert!(
        exposed_track_reference.is_some(),
        "shared exposed-track reference cell exists"
    );
    let Some(exposed_track_reference) = exposed_track_reference else {
        return;
    };

    for &coordinate in &shared_thumb_coordinates {
        assert_eq!(
            frame_a.cell(coordinate),
            Some(thumb_reference),
            "A shared thumb Cell fields at {coordinate:?}"
        );
        assert_eq!(
            frame_b.cell(coordinate),
            Some(thumb_reference),
            "B shared thumb Cell fields at {coordinate:?}"
        );
    }
    for &coordinate in &shared_exposed_track_coordinates {
        assert_eq!(
            frame_a.cell(coordinate),
            Some(exposed_track_reference),
            "A shared exposed-track Cell fields at {coordinate:?}"
        );
        assert_eq!(
            frame_b.cell(coordinate),
            Some(exposed_track_reference),
            "B shared exposed-track Cell fields at {coordinate:?}"
        );
    }

    for &coordinate in &thumb_only_a {
        assert_eq!(
            frame_a.cell(coordinate),
            Some(thumb_reference),
            "A thumb Cell fields at {coordinate:?}"
        );
        assert_eq!(
            frame_b.cell(coordinate),
            Some(exposed_track_reference),
            "B exposed-track Cell fields at {coordinate:?}"
        );
    }
    for &coordinate in &thumb_only_b {
        assert_eq!(
            frame_a.cell(coordinate),
            Some(exposed_track_reference),
            "A exposed-track Cell fields at {coordinate:?}"
        );
        assert_eq!(
            frame_b.cell(coordinate),
            Some(thumb_reference),
            "B thumb Cell fields at {coordinate:?}"
        );
    }
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

mod tracked_port_owner {
    use super::*;
    use crate::domain::action::{Action, ActionKind, Availability, Risk};
    use crate::sim::world::World;
    use ratatui::layout::Rect;
    use std::collections::BTreeSet;

    const EXPECTED_KEYS: [&str; 11] = [
        "monorepo-root 80 24 junie mono",
        "monorepo-root 80 24 paper truecolor",
        "monorepo-root 80 24 paper 256",
        "monorepo-root 80 24 paper 16",
        "monorepo-root 80 24 paper mono",
        "activities-multi 80 24 junie truecolor",
        "activities-multi 80 24 junie mono",
        "activities-multi 80 24 paper truecolor",
        "activities-multi 80 24 paper 256",
        "activities-multi 80 24 paper 16",
        "activities-multi 80 24 paper mono",
    ];

    const EXISTING_OWNERS: [&str; 6] = [
        "rust-dirty 80 24 junie 256",
        "monorepo-root 80 24 junie 256",
        "monorepo-child 80 24 junie 256",
        "activities-multi 80 24 junie 256",
        "monorepo-root 80 24 junie truecolor",
        "monorepo-root 80 24 junie 16",
    ];

    #[derive(Debug, Clone, Copy)]
    enum SourceTheme {
        Junie,
        Paper,
    }

    impl SourceTheme {
        fn build(self) -> Theme {
            match self {
                Self::Junie => Theme::junie(),
                Self::Paper => Theme::paper(),
            }
        }

        const fn label(self) -> &'static str {
            match self {
                Self::Junie => "junie",
                Self::Paper => "paper",
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    struct Case {
        scenario: Scenario,
        theme: SourceTheme,
        color: ColorLevel,
    }

    const CASES: [Case; 11] = [
        Case {
            scenario: Scenario::MonorepoRoot,
            theme: SourceTheme::Junie,
            color: ColorLevel::Mono,
        },
        Case {
            scenario: Scenario::MonorepoRoot,
            theme: SourceTheme::Paper,
            color: ColorLevel::TrueColor,
        },
        Case {
            scenario: Scenario::MonorepoRoot,
            theme: SourceTheme::Paper,
            color: ColorLevel::Ansi256,
        },
        Case {
            scenario: Scenario::MonorepoRoot,
            theme: SourceTheme::Paper,
            color: ColorLevel::Ansi16,
        },
        Case {
            scenario: Scenario::MonorepoRoot,
            theme: SourceTheme::Paper,
            color: ColorLevel::Mono,
        },
        Case {
            scenario: Scenario::ActivitiesMulti,
            theme: SourceTheme::Junie,
            color: ColorLevel::TrueColor,
        },
        Case {
            scenario: Scenario::ActivitiesMulti,
            theme: SourceTheme::Junie,
            color: ColorLevel::Mono,
        },
        Case {
            scenario: Scenario::ActivitiesMulti,
            theme: SourceTheme::Paper,
            color: ColorLevel::TrueColor,
        },
        Case {
            scenario: Scenario::ActivitiesMulti,
            theme: SourceTheme::Paper,
            color: ColorLevel::Ansi256,
        },
        Case {
            scenario: Scenario::ActivitiesMulti,
            theme: SourceTheme::Paper,
            color: ColorLevel::Ansi16,
        },
        Case {
            scenario: Scenario::ActivitiesMulti,
            theme: SourceTheme::Paper,
            color: ColorLevel::Mono,
        },
    ];

    impl Case {
        fn key(self) -> String {
            let color = match self.color {
                ColorLevel::TrueColor => "truecolor",
                ColorLevel::Ansi256 => "256",
                ColorLevel::Ansi16 => "16",
                ColorLevel::Mono => "mono",
                _ => "other",
            };
            format!(
                "{} 80 24 {} {color}",
                self.scenario.name(),
                self.theme.label()
            )
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum VisualToken {
        Header(String),
        Gap,
        Action(String),
    }

    struct ModelPair {
        harness_a: Harness<App>,
        harness_b: Harness<App>,
        expected_raw: Vec<Action>,
        expected_cwd: String,
        memory_a: crate::domain::ranking::Memory,
        memory_b: crate::domain::ranking::Memory,
        rows_a: Vec<(String, String)>,
        rows_b: Vec<(String, String)>,
        tokens_a: Vec<VisualToken>,
        tokens_b: Vec<VisualToken>,
        port_visual_index: usize,
        extent_a: usize,
        extent_b: usize,
    }

    #[derive(Debug, Clone, Copy)]
    struct NativeGeometry {
        rows: Rect,
        track: Rect,
        thumb: Rect,
    }

    fn action_kind_label(kind: ActionKind) -> &'static str {
        match kind {
            ActionKind::Task => "task",
            ActionKind::Git => "git",
            ActionKind::Plan => "plan",
            ActionKind::Connect => "connect",
            ActionKind::Flow => "flow",
            ActionKind::Clone => "clone",
            ActionKind::System => "system",
        }
    }

    fn action_scope_text(action: &Action) -> String {
        if action.scope_label.is_empty() {
            action.scope.label().to_owned()
        } else {
            format!("{} {}", action.scope.label(), action.scope_label)
        }
    }

    fn action_risk_label(risk: Risk) -> &'static str {
        match risk {
            Risk::ReadOnly => "read only",
            Risk::Bounded => "bounded",
            Risk::Broad => "broad",
        }
    }

    fn action_risk_text(action: &Action) -> String {
        match &action.availability {
            Availability::Ready => action_risk_label(action.risk).to_owned(),
            Availability::NeedsTrust(_) => {
                format!("▲ untrusted config · {}", action_risk_label(action.risk))
            }
            Availability::Blocked(_) => {
                format!("▲ blocked · {}", action_risk_label(action.risk))
            }
        }
    }

    fn visible_actions(world: &World) -> Vec<Action> {
        crate::sim::catalogue::sections(world)
            .into_iter()
            .flat_map(|section| {
                crate::sim::catalogue::visible(
                    &section.actions,
                    None,
                    "",
                    &world.memory,
                    &world.cwd,
                )
            })
            .collect()
    }

    // Keep this oracle aligned with HomeState::ActionColumns::apply: Port is
    // safe to hide offscreen only when it does not move any surviving column.
    fn action_column_maxima(world: &World) -> [u16; 4] {
        let actions = visible_actions(world);
        [
            actions
                .iter()
                .map(|action| termrock::width(&action.title))
                .max()
                .unwrap_or(14),
            actions
                .iter()
                .map(|action| termrock::width(action_kind_label(action.kind)))
                .max()
                .unwrap_or(4),
            actions
                .iter()
                .map(|action| termrock::width(&action_scope_text(action)))
                .max()
                .unwrap_or(4),
            actions
                .iter()
                .map(|action| termrock::width(&action_risk_text(action)))
                .max()
                .unwrap_or(0),
        ]
    }

    fn visual_tokens(rows: &[(String, String)]) -> Vec<VisualToken> {
        let mut tokens = Vec::new();
        let mut previous_section = None;
        for (section, action_id) in rows {
            if previous_section != Some(section.as_str()) {
                if previous_section.is_some() {
                    tokens.push(VisualToken::Gap);
                }
                tokens.push(VisualToken::Header(section.clone()));
                previous_section = Some(section.as_str());
            }
            tokens.push(VisualToken::Action(action_id.clone()));
        }
        tokens
    }

    fn model_pair(case: Case) -> Option<ModelPair> {
        let mut world_a = crate::sim::fixtures::world_for(case.scenario);
        let mut world_b = crate::sim::fixtures::world_for(case.scenario);
        world_a.seek(4_000);
        world_b.seek(4_000);

        let raw_a = crate::sim::catalogue::catalogue(&world_a);
        let raw_b = crate::sim::catalogue::catalogue(&world_b);
        let expected_raw = raw_a.clone();
        let expected_cwd = world_a.cwd.clone();
        assert_eq!(raw_a, raw_b, "{} raw catalogue before hide", case.key());
        assert_eq!(world_a.cwd, world_b.cwd, "{} fixture cwd", case.key());
        assert_eq!(
            world_a.memory,
            world_b.memory,
            "{} initial Memory",
            case.key()
        );
        assert_eq!(
            world_a.scenario,
            world_b.scenario,
            "{} scenario",
            case.key()
        );
        assert_eq!(world_a.clock, world_b.clock, "{} initial clock", case.key());
        assert_eq!(world_a.clock.now_ms, 4_000, "{} fixture time", case.key());

        let port_actions: Vec<_> = raw_a
            .iter()
            .filter(|action| action.id == "system.port")
            .collect();
        assert_eq!(port_actions.len(), 1, "{} actual Port action", case.key());
        let port_action = port_actions.first();
        assert!(port_action.is_some(), "{} unique Port action", case.key());
        let port_action = port_action?;
        let port_pin = Pin {
            path: world_b.cwd.clone(),
            command: port_action.command.clone(),
        };
        assert!(
            !world_a
                .memory
                .hides
                .iter()
                .any(|existing| existing == &port_pin),
            "{} A does not already hide the actual Port command",
            case.key()
        );

        let memory_a = world_a.memory.clone();
        let mut expected_memory_b = memory_a.clone();
        expected_memory_b.hides.push(port_pin.clone());
        world_b.memory.hides.push(port_pin.clone());
        assert_eq!(
            world_a.memory,
            memory_a,
            "{} A Memory unchanged",
            case.key()
        );
        assert_eq!(
            world_b.memory,
            expected_memory_b,
            "{} B adds only the actual cwd-local Port Pin",
            case.key()
        );
        assert_eq!(port_pin.path, world_b.cwd, "{} Port Pin cwd", case.key());
        assert_eq!(
            port_pin.command,
            port_action.command,
            "{} Port Pin actual command",
            case.key()
        );

        let raw_b = crate::sim::catalogue::catalogue(&world_b);
        assert_eq!(raw_b, raw_a, "{} raw catalogue/order remains", case.key());
        assert_eq!(
            raw_b
                .iter()
                .filter(|action| action.id == "system.port")
                .count(),
            1,
            "{} Port remains in B raw catalogue",
            case.key()
        );

        let rows_a = home_display_rows(&world_a);
        let rows_b = home_display_rows(&world_b);
        assert!(
            rows_a
                .iter()
                .any(|(_, action_id)| action_id == "system.port"),
            "{} A displays Port",
            case.key()
        );
        let expected_rows_b: Vec<_> = rows_a
            .iter()
            .filter(|(_, action_id)| action_id != "system.port")
            .cloned()
            .collect();
        assert_eq!(rows_b, expected_rows_b, "{} B hides only Port", case.key());
        let tokens_a = visual_tokens(&rows_a);
        let tokens_b = visual_tokens(&rows_b);
        let expected_tokens_b: Vec<_> = tokens_a
            .iter()
            .filter(|&token| !matches!(token, VisualToken::Action(id) if id == "system.port"))
            .cloned()
            .collect();
        assert_eq!(
            tokens_b,
            expected_tokens_b,
            "{} B retains all section headers, gaps and surviving order",
            case.key()
        );
        let port_visual_index = tokens_a
            .iter()
            .position(|token| matches!(token, VisualToken::Action(id) if id == "system.port"));
        assert!(
            port_visual_index.is_some(),
            "{} Port visual token",
            case.key()
        );
        let port_visual_index = port_visual_index?;
        assert!(
            port_visual_index >= 15,
            "{} Port is below the initial 15-row viewport",
            case.key()
        );

        let extent_a = home_visual_extent(&rows_a);
        let extent_b = home_visual_extent(&rows_b);
        assert_eq!(tokens_a.len(), extent_a, "{} A token extent", case.key());
        assert_eq!(tokens_b.len(), extent_b, "{} B token extent", case.key());
        assert!(
            extent_a > 15 && extent_b > 15,
            "{} both arms overflow",
            case.key()
        );
        assert_eq!(
            extent_a,
            extent_b + 1,
            "{} B removes one visual row",
            case.key()
        );
        assert_eq!(
            action_column_maxima(&world_a),
            action_column_maxima(&world_b),
            "{} ActionColumns maxima stay fixed after hiding Port",
            case.key()
        );

        let memory_b = world_b.memory.clone();
        Some(ModelPair {
            harness_a: harness(world_a, case),
            harness_b: harness(world_b, case),
            expected_raw,
            expected_cwd,
            memory_a,
            memory_b,
            rows_a,
            rows_b,
            tokens_a,
            tokens_b,
            port_visual_index,
            extent_a,
            extent_b,
        })
    }

    fn harness(world: World, case: Case) -> Harness<App> {
        let source_theme = case.theme.build();
        let mut app = App::from_fixture_world(world, Motion::Paused, 4_000);
        app.home.history = false;
        let app = app.with_source_theme(source_theme.clone());
        Harness::new_with_feedback_clock(
            app,
            source_theme,
            80,
            24,
            FeedbackClock::Simulation {
                initial: SimulationMoment::from_millis(4_000),
            },
        )
        .with_color(case.color)
    }

    fn native_geometry(
        harness: &Harness<App>,
        rows: &[(String, String)],
        extent: usize,
        case: Case,
        arm: &str,
    ) -> Option<NativeGeometry> {
        let rows_area = harness.area_of(crate::screens::home::ROWS);
        let track = harness.area_of_part(crate::screens::home::ROWS, PartRef::of(Part::TRACK));
        let thumb = harness.area_of_part(crate::screens::home::ROWS, PartRef::of(Part::THUMB));
        assert!(rows_area.is_some(), "{} {} Home rows", arm, case.key());
        assert!(track.is_some(), "{} {} native track", arm, case.key());
        assert!(thumb.is_some(), "{} {} native thumb", arm, case.key());
        let rows_area = rows_area?;
        let track = track?;
        let thumb = thumb?;
        assert_eq!(
            *harness.buffer().area(),
            Rect::new(0, 0, 80, 24),
            "{} {} complete frame dimensions",
            arm,
            case.key()
        );
        assert_eq!(
            (rows_area.x, rows_area.y, rows_area.width, rows_area.height),
            (1, 6, 78, 15),
            "{} {} native Home rows",
            arm,
            case.key()
        );
        assert_eq!(
            (track.x, track.y, track.width, track.height),
            (78, 6, 1, 15),
            "{} {} native track geometry",
            arm,
            case.key()
        );
        assert_eq!(thumb.x, track.x, "{} {} thumb column", arm, case.key());
        assert_eq!(thumb.y, track.y, "{} {} initial offset", arm, case.key());
        assert_eq!(
            thumb.width,
            track.width,
            "{} {} thumb width",
            arm,
            case.key()
        );

        assert!(
            !rows.is_empty(),
            "{} {} nonempty Home rows",
            arm,
            case.key()
        );
        let (_, first_action_id) = rows.first()?;
        let first_action = harness.area_of_part(
            crate::screens::home::ROWS,
            PartRef::item(Part::ROW, termrock::ItemKey::text(first_action_id)),
        );
        assert!(
            first_action.is_some(),
            "{} {} first native row",
            arm,
            case.key()
        );
        let first_action = first_action?;
        assert_eq!(
            first_action.x,
            rows_area.x,
            "{} {} first row x",
            arm,
            case.key()
        );
        assert_eq!(
            first_action.height,
            1,
            "{} {} first row height",
            arm,
            case.key()
        );
        assert_eq!(
            first_action.y,
            rows_area.y + 1,
            "{} {} header is first row; projected offset is zero",
            arm,
            case.key()
        );

        let viewport = usize::from(rows_area.height);
        let track_len = usize::from(track.height);
        let expected_thumb_height = (viewport.saturating_mul(track_len) / extent)
            .max(1)
            .min(track_len);
        assert_eq!(
            usize::from(thumb.height),
            expected_thumb_height,
            "{} {} model-derived thumb height",
            arm,
            case.key()
        );
        Some(NativeGeometry {
            rows: rows_area,
            track,
            thumb,
        })
    }

    fn assert_arm_fixture(
        harness: &Harness<App>,
        case: Case,
        expected_raw: &[Action],
        expected_cwd: &str,
        expected_memory: &crate::domain::ranking::Memory,
        expected_rows: &[(String, String)],
        arm: &str,
    ) {
        let source_theme = case.theme.build();
        assert_eq!(
            harness.runtime().theme(),
            &source_theme.for_level(case.color)
        );
        assert_eq!(
            harness
                .app()
                .source_theme
                .as_ref()
                .map(|theme| &theme.theme),
            Some(&source_theme),
            "{} {} source theme",
            arm,
            case.key()
        );
        assert!(
            harness.app().route == Route::Home,
            "{} {} route",
            arm,
            case.key()
        );
        let actual_raw = crate::sim::catalogue::catalogue(&harness.app().world);
        assert_eq!(
            actual_raw.as_slice(),
            expected_raw,
            "{} {} raw catalogue/order",
            arm,
            case.key()
        );
        assert_eq!(
            harness.app().world.cwd,
            expected_cwd,
            "{} {} cwd",
            arm,
            case.key()
        );
        assert_eq!(
            &harness.app().world.memory,
            expected_memory,
            "{} {} Memory",
            arm,
            case.key()
        );
        assert_eq!(
            harness.app().world.scenario,
            case.scenario,
            "{} {} scenario",
            arm,
            case.key()
        );
        assert_eq!(
            harness.app().world.clock.now_ms,
            4_000,
            "{} {} time",
            arm,
            case.key()
        );
        assert!(
            !harness.app().world.clock.running,
            "{} {} paused",
            arm,
            case.key()
        );
        assert_eq!(
            harness.app().fixture_time_ms(),
            4_000,
            "{} {} fixture time",
            arm,
            case.key()
        );
        assert_eq!(
            harness.app().motion,
            Motion::Paused,
            "{} {} motion",
            arm,
            case.key()
        );
        assert!(
            !harness.app().home.history,
            "{} {} history disabled",
            arm,
            case.key()
        );
        assert!(
            harness.app().overlay.is_none(),
            "{} {} overlay closed",
            arm,
            case.key()
        );
        assert!(
            !harness.app().menu.is_open(),
            "{} {} menu closed",
            arm,
            case.key()
        );
        assert!(
            harness.app().home.query().is_empty(),
            "{} {} query blank",
            arm,
            case.key()
        );
        assert_eq!(
            harness.app().home.scope,
            None,
            "{} {} scope unchanged",
            arm,
            case.key()
        );
        assert_eq!(
            harness.focus(),
            Some(crate::screens::home::QUERY),
            "{} {} query focus",
            arm,
            case.key()
        );
        assert_eq!(
            home_display_rows(&harness.app().world),
            expected_rows,
            "{} {} rows",
            arm,
            case.key()
        );
        assert_eq!(
            harness.cursor(),
            Some(Position { x: 3, y: 4 }),
            "{} {} cursor",
            arm,
            case.key()
        );
        assert!(
            harness.diagnostics().is_empty(),
            "{} {} diagnostics",
            arm,
            case.key()
        );
    }

    fn assert_visible_prefix_geometry(
        harness_a: &Harness<App>,
        harness_b: &Harness<App>,
        tokens_a: &[VisualToken],
        tokens_b: &[VisualToken],
        port_visual_index: usize,
        case: Case,
    ) {
        for (visual_index, token) in tokens_a.iter().take(15).enumerate() {
            let VisualToken::Action(action_id) = token else {
                continue;
            };
            let row_a = harness_a.area_of_part(
                crate::screens::home::ROWS,
                PartRef::item(Part::ROW, termrock::ItemKey::text(action_id)),
            );
            let row_b = harness_b.area_of_part(
                crate::screens::home::ROWS,
                PartRef::item(Part::ROW, termrock::ItemKey::text(action_id)),
            );
            assert!(row_a.is_some(), "A {} visible row {action_id}", case.key());
            assert!(row_b.is_some(), "B {} visible row {action_id}", case.key());
            assert_eq!(
                row_a,
                row_b,
                "{} surviving prefix row {action_id}",
                case.key()
            );
            let Some(row_area) = row_a else { return };
            let visual_row = u16::try_from(visual_index);
            assert!(visual_row.is_ok(), "{} visible row index fits", case.key());
            let visual_row = visual_row.unwrap_or_default();
            assert_eq!(
                row_area.y,
                6 + visual_row,
                "{} row follows the unchanged model prefix",
                case.key()
            );
        }
        assert_eq!(
            tokens_a[..port_visual_index],
            tokens_b[..port_visual_index],
            "{} complete visual prefix before offscreen Port",
            case.key()
        );
    }

    #[test]
    fn tracked_port_hide_preserves_held_frames_for_exact_keys() {
        let actual_keys: Vec<_> = CASES.iter().map(|case| case.key()).collect();
        let expected_keys: Vec<_> = EXPECTED_KEYS.iter().map(|key| (*key).to_owned()).collect();
        assert_eq!(
            actual_keys, expected_keys,
            "exact tracked owner manifest order"
        );
        let keys: BTreeSet<_> = actual_keys.iter().map(String::as_str).collect();
        assert_eq!(keys.len(), CASES.len(), "tracked keys are unique");
        let existing: BTreeSet<_> = EXISTING_OWNERS.into_iter().collect();
        assert!(
            keys.is_disjoint(&existing),
            "new tracked keys have distinct owners"
        );

        for case in CASES {
            let model = model_pair(case);
            assert!(model.is_some(), "{} source-derived model pair", case.key());
            let Some(model) = model else { return };
            let ModelPair {
                harness_a,
                harness_b,
                expected_raw,
                expected_cwd,
                memory_a,
                memory_b,
                rows_a,
                rows_b,
                tokens_a,
                tokens_b,
                port_visual_index,
                extent_a,
                extent_b,
            } = model;
            assert_arm_fixture(
                &harness_a,
                case,
                &expected_raw,
                &expected_cwd,
                &memory_a,
                &rows_a,
                "A",
            );
            assert_arm_fixture(
                &harness_b,
                case,
                &expected_raw,
                &expected_cwd,
                &memory_b,
                &rows_b,
                "B",
            );

            let geometry_a = native_geometry(&harness_a, &rows_a, extent_a, case, "A");
            let geometry_b = native_geometry(&harness_b, &rows_b, extent_b, case, "B");
            assert!(
                geometry_a.is_some(),
                "{} A model-derived geometry",
                case.key()
            );
            assert!(
                geometry_b.is_some(),
                "{} B model-derived geometry",
                case.key()
            );
            let (Some(geometry_a), Some(geometry_b)) = (geometry_a, geometry_b) else {
                return;
            };
            assert_eq!(
                geometry_a.rows,
                geometry_b.rows,
                "{} same rows area",
                case.key()
            );
            assert_eq!(
                geometry_a.track,
                geometry_b.track,
                "{} same track",
                case.key()
            );
            assert_visible_prefix_geometry(
                &harness_a,
                &harness_b,
                &tokens_a,
                &tokens_b,
                port_visual_index,
                case,
            );

            assert_model_thumb_locality(
                harness_a.buffer(),
                harness_b.buffer(),
                geometry_a.track,
                geometry_a.thumb,
                geometry_b.thumb,
            );
            assert_eq!(
                harness_a.cursor(),
                Some(Position { x: 3, y: 4 }),
                "{} A cursor",
                case.key()
            );
            assert_eq!(
                harness_b.cursor(),
                Some(Position { x: 3, y: 4 }),
                "{} B cursor",
                case.key()
            );
            let selected_a = harness_a.app().home.selected(&harness_a.app().world);
            let selected_b = harness_b.app().home.selected(&harness_b.app().world);
            assert_eq!(
                selected_a.as_ref().map(|action| action.id.as_str()),
                selected_b.as_ref().map(|action| action.id.as_str()),
                "{} selected key remains identical",
                case.key()
            );
            assert!(
                selected_a.is_some(),
                "{} selected action exists",
                case.key()
            );
            harness_b
                .snapshot()
                .named(case.scenario.name())
                .assert_against(&BORDER_BASELINE);
        }
    }
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

    assert_model_thumb_locality(
        harness_a.buffer(),
        harness_b.buffer(),
        track_a,
        model_thumb_a,
        model_thumb_b,
    );

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
fn monorepo_root_ansi16_port_hide_ab_diagnostic() {
    let mut world_a = crate::sim::fixtures::world_for(Scenario::MonorepoRoot);
    let mut world_b = crate::sim::fixtures::world_for(Scenario::MonorepoRoot);
    world_a.seek(4_000);
    world_b.seek(4_000);

    let raw_a_before = crate::sim::catalogue::catalogue(&world_a);
    let raw_b_before = crate::sim::catalogue::catalogue(&world_b);
    assert_eq!(
        raw_a_before, raw_b_before,
        "initial raw catalogue and order"
    );
    assert_eq!(world_a.memory, world_b.memory, "initial Memory values");
    assert_eq!(world_a.cwd, world_b.cwd, "initial cwd values");
    assert_eq!(
        world_a.scenario, world_b.scenario,
        "initial scenario values"
    );
    assert_eq!(world_a.clock, world_b.clock, "initial simulation clocks");
    let memory_a = world_a.memory.clone();
    let memory_b_before = world_b.memory.clone();
    assert_eq!(memory_a, memory_b_before, "initial A/B Memory snapshot");

    let rows_a = home_display_rows(&world_a);
    let rows_b_before = home_display_rows(&world_b);
    assert_eq!(rows_a, rows_b_before, "initial displayed row order");
    let extent_a = home_visual_extent(&rows_a);
    let extent_b_before = home_visual_extent(&rows_b_before);
    assert_eq!(extent_a, extent_b_before, "initial model-derived extents");

    let port_actions: Vec<_> = raw_b_before
        .iter()
        .filter(|action| action.id == "system.port")
        .collect();
    assert_eq!(port_actions.len(), 1, "one actual system.port action");
    let port_action = *port_actions
        .first()
        .expect("the unique raw system.port action exists");
    let port_pin = Pin {
        path: world_b.cwd.clone(),
        command: port_action.command.clone(),
    };
    let mut expected_memory_b = memory_b_before.clone();
    expected_memory_b.hides.push(port_pin.clone());
    world_b.memory.hides.push(port_pin.clone());
    assert_eq!(
        world_b.memory, expected_memory_b,
        "B adds only this Port hide"
    );
    assert_eq!(port_pin.path, world_b.cwd, "B hide uses fixture cwd");
    assert_eq!(
        port_pin.command, port_action.command,
        "B hide uses actual command"
    );

    let raw_a_after = crate::sim::catalogue::catalogue(&world_a);
    let raw_b_after = crate::sim::catalogue::catalogue(&world_b);
    assert_eq!(
        raw_a_after, raw_a_before,
        "A raw catalogue remains unchanged"
    );
    assert_eq!(raw_b_after, raw_a_after, "B keeps raw catalogue and order");
    let rows_b = home_display_rows(&world_b);
    let expected_rows_b: Vec<_> = rows_a
        .iter()
        .filter(|(_, action_id)| action_id != "system.port")
        .cloned()
        .collect();
    assert_eq!(rows_b, expected_rows_b, "B removes only displayed Port");
    assert!(
        rows_a
            .iter()
            .any(|(_, action_id)| action_id == "system.port"),
        "A keeps Port visible"
    );
    assert!(
        rows_b
            .iter()
            .all(|(_, action_id)| action_id != "system.port"),
        "B hides Port from displayed rows"
    );
    assert_eq!(
        rows_a.len(),
        rows_b.len().saturating_add(1),
        "B hides exactly one displayed action"
    );
    let extent_b = home_visual_extent(&rows_b);

    let source_theme = Theme::junie();
    let color = ColorLevel::Ansi16;
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
        ("A", &harness_a, &memory_a, &rows_a, extent_a),
        ("B", &harness_b, &expected_memory_b, &rows_b, extent_b),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            harness.snapshot().key(),
            "harness 80 24 junie 16",
            "{label} exact no-history ANSI16 harness key"
        );
        assert_eq!(harness.runtime().theme(), &expected_runtime_theme);
        assert_eq!(
            harness
                .app()
                .source_theme
                .as_ref()
                .map(|theme| &theme.theme),
            Some(&source_theme),
            "{label} source theme"
        );
        assert_eq!(harness.app().world.scenario, Scenario::MonorepoRoot);
        assert_eq!(harness.app().world.clock.now_ms, 4_000);
        assert!(!harness.app().world.clock.running, "{label} paused clock");
        assert_eq!(harness.app().motion, Motion::Paused);
        assert_eq!(harness.app().fixture_time_ms(), 4_000);
        assert!(!harness.app().home.history, "{label} no-history input");
        assert_eq!(
            harness.app().world.memory,
            *expected_memory,
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
            "{label} model-derived extent"
        );
        assert!(
            expected_extent > expected_rows.len(),
            "{label} model includes section headers"
        );
        let (first_section, first_action_id) = expected_rows
            .first()
            .expect("each ANSI16 model has a first Home action");
        assert!(!first_section.is_empty(), "{label} first section label");

        let cursor = harness.cursor();
        assert_eq!(cursor, Some(Position { x: 3, y: 4 }), "{label} cursor");
        let rows_area = harness
            .area_of(crate::screens::home::ROWS)
            .expect("native Home rows area");
        let track = harness
            .area_of_part(crate::screens::home::ROWS, PartRef::of(Part::TRACK))
            .expect("native Home track");
        let thumb = harness
            .area_of_part(crate::screens::home::ROWS, PartRef::of(Part::THUMB))
            .expect("native Home thumb");
        assert_eq!(
            rows_area,
            ratatui::layout::Rect::new(1, 6, 78, 15),
            "{label} native Home rows geometry"
        );
        assert_eq!(
            track,
            ratatui::layout::Rect::new(78, 6, 1, 15),
            "{label} native track geometry"
        );
        let first_row = harness
            .area_of_part(
                crate::screens::home::ROWS,
                PartRef::item(Part::ROW, termrock::ItemKey::text(first_action_id)),
            )
            .expect("first keyed Home action row");
        assert_eq!(first_row.x, rows_area.x, "{label} first action x");
        assert_eq!(first_row.height, 1, "{label} first action height");
        let expected_first_row_y = rows_area.y.saturating_add(1);
        assert_eq!(
            first_row.y, expected_first_row_y,
            "{label} section header occupies logical row zero"
        );
        let projected_offset = expected_first_row_y.checked_sub(first_row.y);
        assert_eq!(projected_offset, Some(0), "{label} projected offset");

        let viewport = usize::from(rows_area.height);
        let track_len = usize::from(track.height);
        let thumb_len = viewport
            .saturating_mul(track_len)
            .checked_div(expected_extent)
            .unwrap_or_default()
            .max(1)
            .min(track_len);
        let expected_thumb_height =
            u16::try_from(thumb_len).expect("model-derived thumb height fits u16");
        let expected_thumb =
            ratatui::layout::Rect::new(track.x, track.y, track.width, expected_thumb_height);
        assert_eq!(thumb, expected_thumb, "{label} model-derived thumb");
        assert_eq!(thumb.y, track.y, "{label} initial thumb offset is zero");
        model_geometry[arm_index] = Some((rows_area, track, thumb));
    }

    let (rows_a_native, track_a, thumb_a) =
        model_geometry[0].expect("A model/native geometry was recorded");
    let (rows_b_native, track_b, thumb_b) =
        model_geometry[1].expect("B model/native geometry was recorded");
    assert_eq!(rows_a_native, rows_b_native, "A/B native rows rectangles");
    assert_eq!(track_a, track_b, "A/B native track rectangles");

    let frame_area = *harness_a.buffer().area();
    assert_eq!(
        frame_area,
        ratatui::layout::Rect::new(0, 0, 80, 24),
        "A complete frame dimensions"
    );
    assert_eq!(
        *harness_b.buffer().area(),
        frame_area,
        "B complete frame dimensions"
    );
    let cell_count = usize::from(frame_area.width) * usize::from(frame_area.height);
    assert_eq!(cell_count, 1_920, "each arm has all 1,920 terminal cells");
    let cursor_a_before = harness_a.cursor();
    let cursor_b_before = harness_b.cursor();
    assert_eq!(cursor_a_before, Some(Position { x: 3, y: 4 }));
    assert_eq!(cursor_b_before, Some(Position { x: 3, y: 4 }));

    assert_model_thumb_locality(
        harness_a.buffer(),
        harness_b.buffer(),
        track_a,
        thumb_a,
        thumb_b,
    );
    assert_eq!(
        harness_a.cursor(),
        cursor_a_before,
        "A cursor after comparison"
    );
    assert_eq!(
        harness_b.cursor(),
        cursor_b_before,
        "B cursor after comparison"
    );

    let scene_b = harness_b.snapshot().named(Scenario::MonorepoRoot.name());
    let digest_b = scene_b.digest();
    assert!(
        digest_b == 0x643a_4f12_c50a_45a7,
        "B preserves the exact held ANSI16 frame"
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

mod root_100_nonoverflow_port_owner {
    use super::*;
    use crate::domain::action::{Action, ActionKind, Availability, Risk};
    use crate::sim::world::World;
    use ratatui::{buffer::Buffer, layout::Rect, style::Modifier};
    use termrock::{Family, StateFlags, Variant};
    use termrock_test_support::Scene;

    const FRAME_MS: u64 = 4_000;
    const EXPECTED_KEY: &str = "monorepo-root 100 30 junie truecolor";
    const PORT_ID: &str = "system.port";

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum OwnerViewport {
        W100H30,
        W120H40,
        W160H50,
    }

    impl OwnerViewport {
        fn size(self) -> (u16, u16) {
            match self {
                Self::W100H30 => (100, 30),
                Self::W120H40 => (120, 40),
                Self::W160H50 => (160, 50),
            }
        }

        fn frame_area(self) -> Rect {
            let (width, height) = self.size();
            Rect::new(0, 0, width, height)
        }

        fn rows_area(self) -> Rect {
            match self {
                Self::W100H30 => Rect::new(1, 6, 98, 21),
                Self::W120H40 => Rect::new(1, 6, 118, 31),
                Self::W160H50 => Rect::new(1, 6, 158, 41),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum OwnerSourceTheme {
        Junie,
        Paper,
    }

    impl OwnerSourceTheme {
        fn theme(self) -> Theme {
            match self {
                Self::Junie => Theme::junie(),
                Self::Paper => Theme::paper(),
            }
        }

        fn label(self) -> &'static str {
            match self {
                Self::Junie => "junie",
                Self::Paper => "paper",
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct OwnerCase {
        scenario: Scenario,
        viewport: OwnerViewport,
        source_theme: OwnerSourceTheme,
        color: ColorLevel,
    }

    impl OwnerCase {
        const ROOT100_JUNIE_TRUECOLOR: Self = Self {
            scenario: Scenario::MonorepoRoot,
            viewport: OwnerViewport::W100H30,
            source_theme: OwnerSourceTheme::Junie,
            color: ColorLevel::TrueColor,
        };

        fn key(self) -> String {
            let (width, height) = self.viewport.size();
            let color = match self.color {
                ColorLevel::TrueColor => "truecolor",
                ColorLevel::Ansi256 => "256",
                ColorLevel::Ansi16 => "16",
                ColorLevel::Mono => "mono",
                _ => "other",
            };
            format!(
                "{} {width} {height} {} {color}",
                self.scenario.name(),
                self.source_theme.label()
            )
        }
    }

    fn all_owner_cases(scenario: Scenario) -> Vec<OwnerCase> {
        assert!(
            matches!(scenario, Scenario::MonorepoRoot | Scenario::MonorepoChild),
            "finite held owner accepts only Root and Child scenarios"
        );
        let cases: Vec<_> = [
            OwnerViewport::W100H30,
            OwnerViewport::W120H40,
            OwnerViewport::W160H50,
        ]
        .into_iter()
        .flat_map(|viewport| {
            [OwnerSourceTheme::Junie, OwnerSourceTheme::Paper]
                .into_iter()
                .flat_map(move |source_theme| {
                    [
                        ColorLevel::TrueColor,
                        ColorLevel::Ansi256,
                        ColorLevel::Ansi16,
                        ColorLevel::Mono,
                    ]
                    .into_iter()
                    .map(move |color| OwnerCase {
                        scenario,
                        viewport,
                        source_theme,
                        color,
                    })
                })
        })
        .collect();
        assert_eq!(cases.len(), 24, "finite scenario configuration size");

        let mut keys: Vec<_> = cases.iter().map(|case| case.key()).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), 24, "finite scenario keys are unique");
        assert!(
            cases.iter().all(|case| case.scenario == scenario),
            "finite scenario membership is explicit"
        );
        cases
    }

    fn remaining_root_cases() -> Vec<OwnerCase> {
        let cases: Vec<_> = all_owner_cases(Scenario::MonorepoRoot)
            .into_iter()
            .filter(|case| *case != OwnerCase::ROOT100_JUNIE_TRUECOLOR)
            .collect();
        assert_eq!(cases.len(), 23, "Root100 Junie TrueColor is already owned");
        assert!(
            cases
                .iter()
                .all(|case| *case != OwnerCase::ROOT100_JUNIE_TRUECOLOR),
            "remaining Root matrix excludes the existing Root100 owner"
        );
        cases
    }

    fn remaining_child_cases() -> Vec<OwnerCase> {
        let cases = all_owner_cases(Scenario::MonorepoChild);
        assert_eq!(cases.len(), 24, "all finite Child keys are admitted");
        assert!(
            cases
                .iter()
                .all(|case| case.scenario == Scenario::MonorepoChild),
            "Child matrix contains only MonorepoChild keys"
        );
        cases
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum VisualToken {
        Header(String),
        Gap,
        Action(String),
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct PositionedToken {
        token: VisualToken,
        y: u16,
    }

    #[derive(Clone, Copy)]
    struct FrameContract<'a> {
        frame_area: Rect,
        rows_area: Rect,
        tokens_a: &'a [VisualToken],
        tokens_b: &'a [VisualToken],
        port_row: Rect,
        blank_suffix: Rect,
        common_blank_suffix: Rect,
        port_paint: &'a Buffer,
        blank_paint: &'a Buffer,
    }

    fn action_kind_label(kind: ActionKind) -> &'static str {
        match kind {
            ActionKind::Task => "task",
            ActionKind::Git => "git",
            ActionKind::Plan => "plan",
            ActionKind::Connect => "connect",
            ActionKind::Flow => "flow",
            ActionKind::Clone => "clone",
            ActionKind::System => "system",
        }
    }

    fn action_scope_text(action: &Action) -> String {
        if action.scope_label.is_empty() {
            action.scope.label().to_owned()
        } else {
            format!("{} {}", action.scope.label(), action.scope_label)
        }
    }

    fn action_risk_label(risk: Risk) -> &'static str {
        match risk {
            Risk::ReadOnly => "read only",
            Risk::Bounded => "bounded",
            Risk::Broad => "broad",
        }
    }

    fn action_risk_text(action: &Action) -> String {
        match &action.availability {
            Availability::Ready => action_risk_label(action.risk).to_owned(),
            Availability::NeedsTrust(_) => {
                format!("▲ untrusted config · {}", action_risk_label(action.risk))
            }
            Availability::Blocked(_) => {
                format!("▲ blocked · {}", action_risk_label(action.risk))
            }
        }
    }

    fn visible_actions(world: &World) -> Vec<Action> {
        crate::sim::catalogue::sections(world)
            .into_iter()
            .flat_map(|section| {
                crate::sim::catalogue::visible(
                    &section.actions,
                    None,
                    "",
                    &world.memory,
                    &world.cwd,
                )
            })
            .collect()
    }

    fn action_column_maxima(world: &World) -> [u16; 4] {
        let actions = visible_actions(world);
        [
            actions
                .iter()
                .map(|action| termrock::width(&action.title))
                .max()
                .unwrap_or(14),
            actions
                .iter()
                .map(|action| termrock::width(action_kind_label(action.kind)))
                .max()
                .unwrap_or(4),
            actions
                .iter()
                .map(|action| termrock::width(&action_scope_text(action)))
                .max()
                .unwrap_or(4),
            actions
                .iter()
                .map(|action| termrock::width(&action_risk_text(action)))
                .max()
                .unwrap_or(0),
        ]
    }

    fn visual_tokens(rows: &[(String, String)]) -> Vec<VisualToken> {
        let mut tokens = Vec::new();
        let mut previous_section = None;
        for (section, action_id) in rows {
            if previous_section != Some(section.as_str()) {
                if previous_section.is_some() {
                    tokens.push(VisualToken::Gap);
                }
                tokens.push(VisualToken::Header(section.clone()));
                previous_section = Some(section.as_str());
            }
            tokens.push(VisualToken::Action(action_id.clone()));
        }
        tokens
    }

    fn positioned(tokens: &[VisualToken], rows_area: Rect) -> Vec<PositionedToken> {
        tokens
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, token)| PositionedToken {
                token,
                y: rows_area.y.saturating_add(
                    u16::try_from(index).expect("Root token index fits the configured viewport"),
                ),
            })
            .collect()
    }

    fn row_rect(rows_area: Rect, y: u16) -> Rect {
        Rect::new(rows_area.x, y, rows_area.width, 1)
    }

    fn suffix_rect(rows_area: Rect, y: u16) -> Rect {
        Rect::new(
            rows_area.x,
            y,
            rows_area.width,
            rows_area.bottom().saturating_sub(y),
        )
    }

    fn port_position(tokens: &[PositionedToken]) -> Option<u16> {
        tokens.iter().find_map(|positioned| {
            matches!(&positioned.token, VisualToken::Action(id) if id == PORT_ID)
                .then_some(positioned.y)
        })
    }

    fn action_position(tokens: &[PositionedToken], action_id: &str) -> Option<u16> {
        tokens.iter().find_map(|positioned| {
            matches!(&positioned.token, VisualToken::Action(id) if id == action_id)
                .then_some(positioned.y)
        })
    }

    fn assert_generic_home_renderer(world: &World, case: OwnerCase) {
        let width = case.viewport.size().0;
        assert_ne!(
            world.scenario.boot_title(),
            crate::screens::finder::RECENT,
            "generic Home has no recent-items lead"
        );
        assert!(
            !crate::sim::catalogue::docker_cleanup_tag_frame(world),
            "generic Home has no Docker cleanup query patch"
        );
        assert!(crate::sim::catalogue::parity_files_view(world, width).is_none());
        assert!(crate::sim::catalogue::first_use_view(world, width, false).is_none());
        assert!(crate::sim::catalogue::monorepo_root_view(world, width, false).is_none());
        assert!(!crate::sim::catalogue::monorepo_child_tag_frame(world));
        assert!(crate::sim::catalogue::monorepo_child_tag(world).is_none());
        assert!(crate::sim::catalogue::monorepo_child_view(world, width).is_none());
        assert!(crate::sim::catalogue::docker_cleanup_view(world, width).is_none());
        assert!(crate::sim::catalogue::remote_host_view(world, width).is_none());
        assert!(crate::sim::catalogue::activities_multi_view(world, width, false).is_none());
        assert!(crate::sim::catalogue::parity_discovery_view(world, width).is_none());
        assert!(crate::sim::catalogue::parity_discovery(world).is_none());
        assert!(crate::sim::catalogue::parity_history_view(world, width).is_none());
        assert!(crate::sim::catalogue::parity_browser_view(world, width).is_none());
        assert!(crate::sim::catalogue::parity_git_current_view(world, width).is_none());
        assert!(crate::sim::catalogue::parity_git_batch_view(world, width).is_none());
        assert!(crate::sim::catalogue::parity_task_sources_view(world, width).is_none());
        assert!(crate::sim::catalogue::parity_cargo_view(world, width).is_none());
        let special_plate = crate::sim::catalogue::first_use_plate(world, false)
            .or_else(|| crate::sim::catalogue::monorepo_root_plate(world, false))
            .or_else(|| crate::sim::catalogue::monorepo_child_tag(world))
            .or_else(|| crate::sim::catalogue::activities_multi_plate(world, false))
            .or_else(|| crate::sim::catalogue::remote_host_tag(world))
            .or_else(|| crate::sim::catalogue::docker_cleanup_tag(world));
        assert!(
            special_plate.is_none(),
            "generic Home has no custom discovery plate"
        );
        assert!(crate::sim::catalogue::hard_cases_tag(world).is_none());
    }

    fn harness(world: World, case: OwnerCase) -> Harness<App> {
        let source_theme = case.source_theme.theme();
        let (width, height) = case.viewport.size();
        let mut app = App::from_fixture_world(world, Motion::Paused, FRAME_MS);
        app.home.history = false;
        let app = app.with_source_theme(source_theme.clone());
        Harness::new_with_feedback_clock(
            app,
            source_theme,
            width,
            height,
            FeedbackClock::Simulation {
                initial: SimulationMoment::from_millis(FRAME_MS),
            },
        )
        .with_color(case.color)
    }

    fn assert_arm_controls(
        harness: &Harness<App>,
        expected_memory: &crate::domain::ranking::Memory,
        rows: &[(String, String)],
        tokens: &[VisualToken],
        case: OwnerCase,
        label: &str,
    ) {
        let source_theme = case.source_theme.theme();
        let expected_runtime_theme = source_theme.for_level(case.color);
        assert_eq!(
            harness.runtime().theme(),
            &expected_runtime_theme,
            "{label} runtime theme"
        );
        assert_eq!(
            harness
                .app()
                .source_theme
                .as_ref()
                .map(|theme| &theme.theme),
            Some(&source_theme),
            "{label} source theme"
        );
        assert_eq!(
            harness.app().world.scenario,
            case.scenario,
            "{label} scenario"
        );
        assert_eq!(
            harness.app().world.clock.now_ms,
            FRAME_MS as i64,
            "{label} clock"
        );
        assert!(
            !harness.app().world.clock.running,
            "{label} post-App clock is paused"
        );
        assert_eq!(harness.app().motion, Motion::Paused, "{label} motion");
        assert_eq!(
            harness.app().fixture_time_ms(),
            FRAME_MS as i64,
            "{label} fixture time"
        );
        assert!(
            matches!(harness.app().route, Route::Home),
            "{label} Home route"
        );
        assert_eq!(
            harness.app().world.memory,
            *expected_memory,
            "{label} Memory"
        );
        assert_eq!(
            home_display_rows(&harness.app().world),
            rows,
            "{label} full Home action model"
        );
        assert_eq!(harness.app().home.query(), "", "{label} empty query");
        assert_eq!(harness.app().home.scope, None, "{label} no scope");
        assert!(!harness.app().home.history, "{label} history disabled");
        assert!(harness.app().overlay.is_none(), "{label} no overlay");
        assert!(!harness.app().menu.is_open(), "{label} menu closed");
        assert!(
            harness.diagnostics().is_empty(),
            "{label} no diagnostics: {:?}",
            harness.diagnostics()
        );
        assert_eq!(
            harness.focus(),
            Some(crate::screens::home::QUERY),
            "{label} query focus"
        );
        assert_eq!(
            harness.cursor(),
            Some(Position::new(3, 4)),
            "{label} query cursor"
        );
        assert!(harness.hover().is_none(), "{label} no hover");
        assert!(
            !harness
                .state_of(crate::screens::home::ROWS)
                .contains(StateFlags::HOVERED)
        );
        assert!(
            !harness
                .state_of(crate::screens::home::ROWS)
                .contains(StateFlags::PRESSED)
        );
        assert!(
            !harness
                .state_of(crate::screens::home::ROWS)
                .contains(StateFlags::FOCUSED)
        );
        let selected = harness.app().home.selected(&harness.app().world);
        if let Some(selected_action) = selected.as_ref() {
            assert!(
                rows.iter()
                    .any(|(_, action_id)| action_id == &selected_action.id),
                "{label} selected action belongs to the visible scenario model"
            );
        }
        if case.scenario == Scenario::MonorepoRoot {
            assert_ne!(
                selected.as_ref().map(|action| action.id.as_str()),
                Some(PORT_ID),
                "{label} Root Port is not selected"
            );
        }
        assert_eq!(
            *harness.buffer().area(),
            case.viewport.frame_area(),
            "{label} frame"
        );
        assert_eq!(
            harness.area_of(crate::screens::home::ROWS),
            Some(case.viewport.rows_area()),
            "{label} exact rows geometry"
        );
        assert!(
            harness
                .area_of_part(crate::screens::home::ROWS, PartRef::of(Part::TRACK))
                .is_none(),
            "{label} no scrollbar track"
        );
        assert!(
            harness
                .area_of_part(crate::screens::home::ROWS, PartRef::of(Part::THUMB))
                .is_none(),
            "{label} no scrollbar thumb"
        );
        assert!(
            tokens.len() <= usize::from(case.viewport.rows_area().height),
            "{label} model does not overflow"
        );
        assert!(
            !harness
                .app()
                .world
                .discovery
                .iter()
                .any(|item| !item.done && !item.failed),
            "{label} no pending discovery note over trailing blank"
        );
        assert!(
            !harness.app().world.discovery.iter().any(|item| item.failed),
            "{label} no failed discovery note over trailing blank"
        );

        let positions = positioned(tokens, case.viewport.rows_area());
        for (_, action_id) in rows {
            let expected_y = action_position(&positions, action_id);
            assert!(
                expected_y.is_some(),
                "{label} model position for {action_id}"
            );
            let Some(expected_y) = expected_y else { return };
            assert_eq!(
                harness.area_of_part(
                    crate::screens::home::ROWS,
                    PartRef::item(Part::ROW, termrock::ItemKey::text(action_id)),
                ),
                Some(row_rect(case.viewport.rows_area(), expected_y)),
                "{label} zero-offset row geometry for {action_id}"
            );
        }
        let port_row = harness.area_of_part(
            crate::screens::home::ROWS,
            PartRef::item(Part::ROW, termrock::ItemKey::text(PORT_ID)),
        );
        if rows.iter().any(|(_, action_id)| action_id == PORT_ID) {
            assert!(port_row.is_some(), "{label} visible Port row");
        } else {
            assert!(port_row.is_none(), "{label} hidden Port row");
        }
    }

    fn paint_port_scratch(
        action: &Action,
        columns: [u16; 4],
        row: Rect,
        theme: Theme,
        case: OwnerCase,
    ) -> Buffer {
        let (width, height) = case.viewport.size();
        let mut scene = Scene::new("port-row-scratch", theme, case.color, width, height);
        scene.draw(|ui, _| {
            crate::screens::home::paint_port_row_for_test(ui, row, action, columns);
        });
        scene.buffer().clone()
    }

    fn paint_blank_scratch(row: Rect, theme: Theme, case: OwnerCase) -> Buffer {
        let (width, height) = case.viewport.size();
        let mut scene = Scene::new("list-container-scratch", theme, case.color, width, height);
        scene.draw(|ui, _| {
            let style = ui
                .style(
                    Family::LIST,
                    Variant::DEFAULT,
                    Part::CONTAINER,
                    StateFlags::empty(),
                )
                .style;
            ui.fill(row, style);
        });
        scene.buffer().clone()
    }

    fn compare_rect(
        left: &Buffer,
        right: &Buffer,
        left_rect: Rect,
        right_rect: Rect,
        label: &str,
        errors: &mut Vec<String>,
    ) {
        if (left_rect.width, left_rect.height) != (right_rect.width, right_rect.height) {
            errors.push(format!("{label} rectangle sizes differ"));
            return;
        }
        for dy in 0..left_rect.height {
            for dx in 0..left_rect.width {
                let left_position = (
                    left_rect.x.saturating_add(dx),
                    left_rect.y.saturating_add(dy),
                );
                let right_position = (
                    right_rect.x.saturating_add(dx),
                    right_rect.y.saturating_add(dy),
                );
                match (left.cell(left_position), right.cell(right_position)) {
                    (Some(left_cell), Some(right_cell)) if left_cell == right_cell => {}
                    (Some(_), Some(_)) => {
                        errors.push(format!("{label} full Cell differs at {left_position:?}"))
                    }
                    _ => errors.push(format!("{label} missing Cell at {left_position:?}")),
                }
            }
        }
    }

    fn frame_contract_errors(a: &Buffer, b: &Buffer, contract: &FrameContract<'_>) -> Vec<String> {
        let mut errors = Vec::new();
        if a.area() != b.area() || *a.area() != contract.frame_area {
            errors.push("A/B complete frame dimensions differ from configured viewport".to_owned());
        }

        let expected_tokens_b: Vec<_> = contract
            .tokens_a
            .iter()
            .filter(|token| !matches!(token, VisualToken::Action(id) if id == PORT_ID))
            .cloned()
            .collect();
        if expected_tokens_b != contract.tokens_b {
            errors.push("B model is not A minus only the actual Port action".to_owned());
        }

        let positions_a = positioned(contract.tokens_a, contract.rows_area);
        let positions_b = positioned(contract.tokens_b, contract.rows_area);
        let port_y = port_position(&positions_a);
        let expected_port_rect = port_y.map(|y| row_rect(contract.rows_area, y));
        if expected_port_rect != Some(contract.port_row) {
            errors.push("Port oracle row has the wrong model-derived y".to_owned());
        }
        if let Some(port_rect) = expected_port_rect {
            compare_rect(
                a,
                contract.port_paint,
                port_rect,
                port_rect,
                "Port native paint",
                &mut errors,
            );
        } else {
            errors.push("Port action token is absent from A model".to_owned());
        }

        let expected_blank_y = contract
            .rows_area
            .y
            .saturating_add(u16::try_from(contract.tokens_b.len()).unwrap_or(u16::MAX));
        let expected_blank_suffix = suffix_rect(contract.rows_area, expected_blank_y);
        if expected_blank_suffix != contract.blank_suffix
            || contract.tokens_a.len() != contract.tokens_b.len().saturating_add(1)
        {
            errors.push("B blank suffix has the wrong model-derived extent".to_owned());
        }
        compare_rect(
            b,
            contract.blank_paint,
            contract.blank_suffix,
            contract.blank_suffix,
            "B LIST.CONTAINER blank suffix",
            &mut errors,
        );

        let common_blank_y = contract.rows_area.y.saturating_add(
            u16::try_from(contract.tokens_a.len().max(contract.tokens_b.len())).unwrap_or(u16::MAX),
        );
        let expected_common_blank_suffix = suffix_rect(contract.rows_area, common_blank_y);
        if expected_common_blank_suffix != contract.common_blank_suffix {
            errors.push("common blank suffix has the wrong model-derived extent".to_owned());
        }
        compare_rect(
            a,
            b,
            contract.common_blank_suffix,
            contract.common_blank_suffix,
            "common blank suffix",
            &mut errors,
        );

        let survivors_a: Vec<_> = positions_a
            .iter()
            .filter(
                |positioned| !matches!(&positioned.token, VisualToken::Action(id) if id == PORT_ID),
            )
            .collect();
        if survivors_a.len() != positions_b.len() {
            errors.push("survivor model coverage is incomplete".to_owned());
        }
        for (survivor_a, survivor_b) in survivors_a.into_iter().zip(&positions_b) {
            if survivor_a.token != survivor_b.token {
                errors.push("surviving token order differs between A and B models".to_owned());
                continue;
            }
            compare_rect(
                a,
                b,
                row_rect(contract.rows_area, survivor_a.y),
                row_rect(contract.rows_area, survivor_b.y),
                "surviving token",
                &mut errors,
            );
        }

        let area = *a.area();
        for y in area.y..area.bottom() {
            for x in area.x..area.right() {
                let in_rows = x >= contract.rows_area.x
                    && x < contract.rows_area.right()
                    && y >= contract.rows_area.y
                    && y < contract.rows_area.bottom();
                if in_rows {
                    continue;
                }
                if a.cell((x, y)) != b.cell((x, y)) {
                    errors.push(format!("full Cell differs outside Home rows at ({x},{y})"));
                }
            }
        }
        errors.truncate(32);
        errors
    }

    fn first_non_space(buffer: &Buffer, row: Rect) -> Option<Position> {
        for x in row.x..row.right() {
            if buffer
                .cell((x, row.y))
                .is_some_and(|cell| cell.symbol() != " ")
            {
                return Some(Position::new(x, row.y));
            }
        }
        None
    }

    fn flip_foreground(buffer: &mut Buffer, position: Position) {
        let Some(cell) = buffer.cell_mut(position) else {
            return;
        };
        cell.fg = if cell.fg == Color::Reset {
            Color::Indexed(1)
        } else {
            Color::Reset
        };
    }

    fn assert_corruptions_rejected(
        a: &Buffer,
        b: &Buffer,
        contract: &FrameContract<'_>,
        port_y: u16,
        survivor_y: u16,
    ) {
        let survivor_row = row_rect(contract.rows_area, survivor_y);
        let survivor_position = first_non_space(a, survivor_row)
            .expect("Root100 has visible survivor text for corruption controls");

        let mut foreground = a.clone();
        flip_foreground(&mut foreground, survivor_position);
        assert!(
            frame_contract_errors(&foreground, b, contract)
                .iter()
                .any(|error| error.contains("surviving token full Cell differs")),
            "survivor foreground corruption is rejected"
        );

        let mut underline = a.clone();
        if let Some(cell) = underline.cell_mut(survivor_position) {
            if cell.modifier.contains(Modifier::UNDERLINED) {
                cell.modifier.remove(Modifier::UNDERLINED);
            } else {
                cell.modifier.insert(Modifier::UNDERLINED);
            }
        }
        assert!(
            frame_contract_errors(&underline, b, contract)
                .iter()
                .any(|error| error.contains("surviving token full Cell differs")),
            "survivor underline corruption is rejected"
        );

        let mut skip = a.clone();
        #[expect(
            deprecated,
            reason = "Verify complete Cell equality still rejects corruption of the independent legacy skip field"
        )]
        if let Some(cell) = skip.cell_mut(survivor_position) {
            cell.skip = !cell.skip;
        }
        assert!(
            frame_contract_errors(&skip, b, contract)
                .iter()
                .any(|error| error.contains("surviving token full Cell differs")),
            "survivor Cell skip corruption is rejected"
        );

        let mut omitted_tokens_b = contract.tokens_b.to_vec();
        let omitted_index = omitted_tokens_b
            .iter()
            .position(|token| !matches!(token, VisualToken::Action(id) if id == PORT_ID))
            .expect("B has a surviving token to omit");
        omitted_tokens_b.remove(omitted_index);
        let omitted_token_model = FrameContract {
            tokens_b: &omitted_tokens_b,
            ..*contract
        };
        assert!(
            frame_contract_errors(a, b, &omitted_token_model)
                .iter()
                .any(|error| error.contains("B model is not A minus only the actual Port action")),
            "omitted survivor token coverage is rejected"
        );

        let wrong_port_row = row_rect(contract.rows_area, port_y.saturating_add(1));
        let wrong_y = FrameContract {
            port_row: wrong_port_row,
            ..*contract
        };
        assert!(
            frame_contract_errors(a, b, &wrong_y)
                .iter()
                .any(|error| error.contains("Port oracle row")),
            "wrong model-derived y is rejected"
        );

        let port_position = first_non_space(a, row_rect(contract.rows_area, port_y))
            .expect("Port action has visible content");
        let mut port_cell = a.clone();
        flip_foreground(&mut port_cell, port_position);
        assert!(
            frame_contract_errors(&port_cell, b, contract)
                .iter()
                .any(|error| error.contains("Port native paint")),
            "Port Cell corruption is rejected"
        );

        let mut blank_cell = b.clone();
        if let Some(cell) = blank_cell.cell_mut((contract.blank_suffix.x, contract.blank_suffix.y))
        {
            cell.set_symbol("X");
        }
        assert!(
            frame_contract_errors(a, &blank_cell, contract)
                .iter()
                .any(|error| error.contains("B LIST.CONTAINER blank suffix full Cell differs")),
            "blank suffix Cell corruption is rejected"
        );

        let common_position = (
            contract.common_blank_suffix.x,
            contract.common_blank_suffix.y,
        );
        let mut common_suffix_a = a.clone();
        if let Some(cell) = common_suffix_a.cell_mut(common_position) {
            cell.set_symbol("A");
        }
        assert!(
            frame_contract_errors(&common_suffix_a, b, contract)
                .iter()
                .any(|error| error.contains("common blank suffix full Cell differs")),
            "A-only common blank suffix corruption is rejected"
        );

        let mut common_suffix_b = b.clone();
        if let Some(cell) = common_suffix_b.cell_mut(common_position) {
            cell.set_symbol("B");
        }
        assert!(
            frame_contract_errors(a, &common_suffix_b, contract)
                .iter()
                .any(|error| error.contains("common blank suffix full Cell differs")),
            "B-only common blank suffix corruption is rejected"
        );

        let mut outside_rows = b.clone();
        if let Some(cell) = outside_rows.cell_mut((0, 0)) {
            cell.set_symbol("X");
        }
        assert!(
            frame_contract_errors(a, &outside_rows, contract)
                .iter()
                .any(|error| error.contains("outside Home rows")),
            "outside-rows Cell corruption is rejected"
        );
    }

    fn run_owner_case(case: OwnerCase, check_root100_negative_controls: bool) {
        let mut world_a = crate::sim::fixtures::world_for(case.scenario);
        let mut world_b = crate::sim::fixtures::world_for(case.scenario);
        world_a.seek(FRAME_MS);
        world_b.seek(FRAME_MS);

        assert_eq!(
            world_a.scenario, world_b.scenario,
            "pre-App scenario equality"
        );
        assert_eq!(world_a.cwd, world_b.cwd, "pre-App cwd equality");
        assert_eq!(world_a.clock, world_b.clock, "pre-App clock equality");
        assert_eq!(world_a.clock.now_ms, FRAME_MS as i64, "pre-App clock time");
        assert!(world_a.clock.running, "A pre-App seek leaves clock running");
        assert!(world_b.clock.running, "B pre-App seek leaves clock running");
        assert_eq!(world_a.memory, world_b.memory, "pre-App Memory equality");

        let raw_a = crate::sim::catalogue::catalogue(&world_a);
        let raw_b = crate::sim::catalogue::catalogue(&world_b);
        assert_eq!(raw_a, raw_b, "raw catalogue/order before hide");
        let port_count = raw_a.iter().filter(|action| action.id == PORT_ID).count();
        assert_eq!(port_count, 1, "one actual raw Port action");
        let port_action = raw_a
            .iter()
            .find(|action| action.id == PORT_ID)
            .expect("unique Port action is present")
            .clone();
        let port_pin = Pin {
            path: world_b.cwd.clone(),
            command: port_action.command.clone(),
        };
        assert_eq!(port_pin.path, world_b.cwd, "cwd-local hide path");
        assert!(!world_a.memory.hides.iter().any(|pin| pin == &port_pin));

        let memory_a = world_a.memory.clone();
        let mut memory_b = memory_a.clone();
        memory_b.hides.push(port_pin.clone());
        world_b.memory.hides.push(port_pin.clone());
        assert_eq!(world_a.memory, memory_a, "A Memory unchanged");
        assert_eq!(world_b.memory, memory_b, "B adds only actual cwd-local Pin");
        assert_eq!(
            crate::sim::catalogue::catalogue(&world_b),
            raw_a,
            "raw catalogue/order remains after Memory hide"
        );
        assert_eq!(
            crate::sim::catalogue::catalogue(&world_b)
                .iter()
                .filter(|action| action.id == PORT_ID)
                .count(),
            1,
            "Port remains in raw catalogue"
        );
        assert_generic_home_renderer(&world_a, case);
        assert_generic_home_renderer(&world_b, case);

        let rows_a = home_display_rows(&world_a);
        let rows_b = home_display_rows(&world_b);
        assert!(
            rows_a.iter().any(|(_, id)| id == PORT_ID),
            "A displays Port"
        );
        let expected_rows_b: Vec<_> = rows_a
            .iter()
            .filter(|(_, id)| id != PORT_ID)
            .cloned()
            .collect();
        assert_eq!(rows_b, expected_rows_b, "B hides only actual Port row");
        let columns_a = action_column_maxima(&world_a);
        let columns_b = action_column_maxima(&world_b);
        assert_eq!(
            columns_a, columns_b,
            "all ActionColumns maxima remain equal"
        );

        let tokens_a = visual_tokens(&rows_a);
        let tokens_b = visual_tokens(&rows_b);
        let expected_tokens_b: Vec<_> = tokens_a
            .iter()
            .filter(|token| !matches!(token, VisualToken::Action(id) if id == PORT_ID))
            .cloned()
            .collect();
        assert_eq!(tokens_b, expected_tokens_b, "B model is A minus only Port");
        assert_eq!(tokens_a.len(), tokens_b.len().saturating_add(1));
        if case.scenario == Scenario::MonorepoRoot {
            assert_eq!(tokens_a.len(), 19, "Root A complete token model extent");
            assert_eq!(tokens_b.len(), 18, "Root B complete token model extent");
        }
        let rows_area = case.viewport.rows_area();
        assert!(
            tokens_a.len() <= usize::from(rows_area.height),
            "A model does not overflow"
        );
        assert!(
            tokens_b.len() < usize::from(rows_area.height),
            "B has a blank tail"
        );

        let port_y = port_position(&positioned(&tokens_a, rows_area))
            .expect("A model includes the actual Port token");
        let blank_y = rows_area
            .y
            .saturating_add(u16::try_from(tokens_b.len()).expect("B model extent fits rows area"));
        assert_eq!(
            blank_y,
            rows_area.y.saturating_add(
                u16::try_from(tokens_a.len().saturating_sub(1))
                    .expect("A final token index fits rows area")
            ),
            "B blank tail occupies A's final model token row"
        );
        let port_row = row_rect(rows_area, port_y);
        let blank_suffix = suffix_rect(rows_area, blank_y);
        let common_blank_y = rows_area.y.saturating_add(
            u16::try_from(tokens_a.len().max(tokens_b.len()))
                .expect("common blank suffix starts within rows area"),
        );
        let common_blank_suffix = suffix_rect(rows_area, common_blank_y);
        if case == OwnerCase::ROOT100_JUNIE_TRUECOLOR {
            assert_eq!(
                case.key(),
                EXPECTED_KEY,
                "exact Root100 key remains the existing owner"
            );
            assert_eq!(blank_suffix, Rect::new(1, 24, 98, 3), "full B blank suffix");
            assert_eq!(
                common_blank_suffix,
                Rect::new(1, 25, 98, 2),
                "full common blank suffix"
            );
        }

        let mut harness_a = harness(world_a, case);
        let mut harness_b = harness(world_b, case);
        harness_a.draw();
        harness_b.draw();
        if case.scenario == Scenario::MonorepoChild {
            let selected_a = harness_a
                .app()
                .home
                .selected(&harness_a.app().world)
                .map(|action| action.id);
            let selected_b = harness_b
                .app()
                .home
                .selected(&harness_b.app().world)
                .map(|action| action.id);
            assert_eq!(
                selected_a, selected_b,
                "Child A/B preserve fixture-derived selected action identity"
            );
        }
        assert_arm_controls(&harness_a, &memory_a, &rows_a, &tokens_a, case, "A");
        assert_arm_controls(&harness_b, &memory_b, &rows_b, &tokens_b, case, "B");
        assert_eq!(
            harness_a.area_of_part(
                crate::screens::home::ROWS,
                PartRef::item(Part::ROW, termrock::ItemKey::text(PORT_ID)),
            ),
            Some(port_row),
            "A actual Port identity and model-derived row"
        );
        assert!(
            harness_b
                .area_of_part(
                    crate::screens::home::ROWS,
                    PartRef::item(Part::ROW, termrock::ItemKey::text(PORT_ID)),
                )
                .is_none(),
            "B has no native Port row"
        );

        let runtime_theme = harness_a.runtime().theme().clone();
        let port_paint = paint_port_scratch(
            &port_action,
            columns_a,
            port_row,
            runtime_theme.clone(),
            case,
        );
        let blank_paint = paint_blank_scratch(blank_suffix, runtime_theme, case);
        let contract = FrameContract {
            frame_area: case.viewport.frame_area(),
            rows_area,
            tokens_a: &tokens_a,
            tokens_b: &tokens_b,
            port_row,
            blank_suffix,
            common_blank_suffix,
            port_paint: &port_paint,
            blank_paint: &blank_paint,
        };
        let errors = frame_contract_errors(harness_a.buffer(), harness_b.buffer(), &contract);
        assert!(
            errors.is_empty(),
            "{} held-owner locality errors: {errors:?}",
            case.key()
        );

        if check_root100_negative_controls {
            assert_eq!(
                case,
                OwnerCase::ROOT100_JUNIE_TRUECOLOR,
                "negative controls remain on the exact Root100 owner"
            );
            assert_corruptions_rejected(
                harness_a.buffer(),
                harness_b.buffer(),
                &contract,
                port_y,
                positioned(&tokens_a, rows_area)
                    .iter()
                    .find(|positioned| !matches!(&positioned.token, VisualToken::Action(id) if id == PORT_ID))
                    .expect("A has a surviving visible token")
                    .y,
            );
        }

        let held = harness_b.snapshot().named(case.scenario.name());
        assert_eq!(held.key(), case.key(), "exact named held baseline key");
        held.assert_against(&BORDER_BASELINE);
    }

    #[test]
    fn monorepo_root_100_junie_truecolor_port_hide_preserves_held_nonoverflow_frame() {
        run_owner_case(OwnerCase::ROOT100_JUNIE_TRUECOLOR, true);
    }

    #[test]
    fn monorepo_root_remaining_23_nonoverflow_port_hide_preserves_held_frames() {
        for case in remaining_root_cases() {
            run_owner_case(case, false);
        }
    }

    #[test]
    fn monorepo_child_remaining_24_nonoverflow_port_hide_preserves_held_frames() {
        for case in remaining_child_cases() {
            run_owner_case(case, false);
        }
    }
}
