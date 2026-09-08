//! Actual Holla production-view and input-lifecycle measurements.
//! Each sample reports totals for a complete fixed-size workload batch.
//! Input pairs include the runtime's required pre-input frame publication.
//! Wall time is informational; allocation limits require independent review.
use holla_app::{App, Motion, Scenario};
use junie_tui::{Id, KeyCode, Theme};
use junie_tui_testing::{
    Harness,
    perf::{Counting, Stats, lock, measure_once},
};

#[global_allocator]
static GLOBAL: Counting = Counting;
const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;
const WARMUPS: usize = 20;
const SAMPLES: usize = 5;
const ITERATIONS: usize = 200;
const QUERY: Id = Id::root("home.query");
const STEPS: Id = Id::root("plan.steps");

fn fixture(scenario: Scenario) -> Harness<App> {
    Harness::new(
        App::for_scenario(scenario, Motion::Paused, 4_000),
        Theme::junie(),
        WIDTH,
        HEIGHT,
    )
}
fn invariant(harness: &Harness<App>) -> (u64, i64) {
    (
        harness.app().effect_revision(),
        harness.app().fixture_time_ms(),
    )
}
fn samples(work: &mut dyn FnMut()) -> Vec<Stats> {
    for _ in 0..WARMUPS {
        work();
    }
    (0..SAMPLES)
        .map(|_| {
            measure_once(&mut || {
                for _ in 0..ITERATIONS {
                    work();
                }
            })
        })
        .collect()
}
#[expect(
    clippy::print_stdout,
    reason = "Performance measurements emit raw total-workload samples for independent review"
)]
fn report(name: &str, samples: &[Stats]) {
    for (index, sample) in samples.iter().enumerate() {
        println!(
            "HOLLA_PERF {name} sample={index} iterations={ITERATIONS} allocs_total={} bytes_total={} ns_total={}",
            sample.allocs, sample.bytes, sample.ns
        );
    }
}
fn frame(scenario: Scenario) {
    let mut harness = fixture(scenario);
    let before = invariant(&harness);
    let rendered = harness.text();
    let measurements = samples(&mut || harness.draw());
    assert_eq!(invariant(&harness), before);
    assert_eq!(harness.text(), rendered);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    report(&format!("frame_{}_120x40", scenario.name()), &measurements);
}
fn query_edits() {
    let mut harness = fixture(Scenario::HardCases).with_auto_draw(false);
    assert!(harness.tab_to(QUERY));
    let _ = harness.type_str("git");
    harness.draw();
    let before = invariant(&harness);
    let rendered = harness.text();
    let measurements = samples(&mut || {
        let _ = harness.key(KeyCode::Char('x'));
        let _ = harness.key(KeyCode::Backspace);
    });
    harness.draw();
    assert_eq!(invariant(&harness), before);
    assert_eq!(harness.text(), rendered);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    report("query_edit_pair_hard_cases", &measurements);
}
fn plan_navigation() {
    let mut harness = fixture(Scenario::DockerCleanup);
    assert!(harness.tab_to(QUERY));
    let _ = harness.type_str("docker system prune");
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.tab_to(STEPS));
    assert!(harness.text().contains("Clean up Docker data"));
    let mut harness = harness.with_auto_draw(false);
    let before = invariant(&harness);
    let rendered = harness.text();
    let measurements = samples(&mut || {
        let _ = harness.key(KeyCode::Down);
        let _ = harness.key(KeyCode::Up);
    });
    harness.draw();
    assert_eq!(invariant(&harness), before);
    assert_eq!(harness.text(), rendered);
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
    report("plan_navigation_pair_docker_cleanup", &measurements);
}
#[test]
fn production_workload_measurements() {
    let _guard = lock();
    for scenario in Scenario::ALL {
        frame(scenario);
    }
    query_edits();
    plan_navigation();
}
