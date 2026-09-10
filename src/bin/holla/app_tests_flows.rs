//! Journey tests (CONCEPT §12) through the real App: gates, plans,
//! activities, trust, scope, ranking controls, snapshots, narrow layouts.

use ratatui::crossterm::event::KeyCode;

use junie_tui::core::event::MouseKind;

use crate::app::TabKind;
use crate::app_tests::H;
use crate::domain::plan::{PlanPhase, StepState};
use crate::scenario::{Motion, Scenario};

fn until<F: Fn(&H) -> bool>(h: &mut H, max_ticks: usize, f: F) -> bool {
    for _ in 0..max_ticks {
        if f(h) {
            return true;
        }
        h.ticks(1);
    }
    f(h)
}

#[test]
fn docker_cleanup_takes_two_gates_and_revalidates_before_executing() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Reduced, 0, 120, 40);
    assert!(
        h.row(4).contains("docker clean"),
        "the query is pre-typed: {}",
        h.row(4)
    );
    assert!(h.text().contains("Clean Docker completely"));
    assert!(
        h.last_row().contains("Review…"),
        "destructive rows never run from Enter: {}",
        h.last_row()
    );
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Here › Clean Docker completely"),
        "plan review page: {}",
        h.text()
    );
    assert!(h.text().contains("needs") && h.text().contains("lane"));
    assert!(
        h.text().contains("join"),
        "the rescan step joins the branches"
    );
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("gate 1 of 2"), "{}", h.text());
    assert!(h.text().contains("Sequence · 8 commands"));
    assert_eq!(
        h.app.focus.current(),
        Some(crate::screens::review::GATE_CANCEL),
        "cancel is the default"
    );
    h.key(KeyCode::Enter);
    assert!(
        !h.text().contains("gate 1 of 2"),
        "Enter on Cancel leaves the review"
    );
    // again, this time continue
    h.key(KeyCode::Enter);
    h.key(KeyCode::Char('c'));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("gate 2 of 2"), "{}", h.text());
    assert!(
        h.text()
            .contains("Type I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox to confirm")
    );
    // a wrong phrase keeps Execute disabled
    h.key(KeyCode::Enter);
    h.type_str("yes");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("gate 2 of 2"),
        "a generic yes never arms the gate"
    );
    assert_eq!(h.app.world.plans[0].phase, PlanPhase::Review);
    // the exact phrase arms it; the first execute finds a changed target
    h.key(KeyCode::Esc);
    h.key(KeyCode::Enter);
    for _ in 0..3 {
        h.key(KeyCode::Backspace);
    }
    h.type_str("I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert!(h.last_row().contains("Plan changed"), "{}", h.last_row());
    assert!(
        h.text().contains("gate 1 of 2") && h.text().contains("Changed"),
        "{}",
        h.text()
    );
    assert_eq!(h.app.world.plans[0].phase, PlanPhase::Review);
    // second pass executes
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Enter);
    h.type_str("I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox");
    h.key(KeyCode::Enter);
    h.key(KeyCode::Tab);
    h.key(KeyCode::Enter);
    assert_eq!(h.tab_kind(), TabKind::Plan("docker-cleanup".into()));
    assert_eq!(h.app.world.plans[0].phase, PlanPhase::Running);
    assert!(until(&mut h, 400, |h| h.app.world.plans[0].phase
        == PlanPhase::Done));
    assert!(h.text().contains("7 succeeded"), "{}", h.text());
    assert!(h.text().contains("100% ✓"));
}

#[test]
fn upgrade_plan_exclusion_recalculates_and_failure_blocks_dependents() {
    let mut h = H::new(Scenario::UpgradePlan, Motion::Reduced, 0, 120, 40);
    assert!(h.text().contains("Here › Upgrade everything"));
    assert!(h.text().contains("Final verification"));
    // required steps refuse exclusion
    h.key(KeyCode::Char(' '));
    assert!(
        h.last_row().contains("required by the plan"),
        "{}",
        h.last_row()
    );
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("excluded"));
    assert!(h.text().contains("blocked · needs 06"), "{}", h.text());
    assert_eq!(h.app.world.plans[0].steps[7].state, StepState::Blocked);
    h.key(KeyCode::Char('u'));
    assert_eq!(h.app.world.plans[0].steps[7].state, StepState::Waiting);
    // exclude the optional cleanup instead, then start
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Char('c'));
    assert!(
        h.text().contains("Start upgrade everything?"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.tab_kind(), TabKind::Plan("upgrade".into()));
    // both branches run after preflight
    assert!(until(&mut h, 200, |h| h.app.world.plans[0].running().len() == 2));
    assert!(until(&mut h, 600, |h| h.app.world.plans[0].phase
        == PlanPhase::Done));
    let p = &h.app.world.plans[0];
    assert_eq!(p.steps[5].state, StepState::Failed, "the mise branch fails");
    assert_eq!(
        p.steps[3].state,
        StepState::Succeeded,
        "the Debian branch finishes"
    );
    assert_eq!(
        p.steps[7].state,
        StepState::Blocked,
        "verification reflects the failure"
    );
    assert_eq!(p.steps[6].state, StepState::Excluded);
    assert!(
        h.text().contains("1 failed · 1 never started · 1 excluded"),
        "{}",
        h.text()
    );
    // the run landed the cursor on the failed step: retry from there;
    // succeeded work is untouched
    h.key(KeyCode::Char('r'));
    assert_eq!(h.app.world.plans[0].steps[5].state, StepState::Ready);
    assert_eq!(h.app.world.plans[0].steps[3].state, StepState::Succeeded);
    assert!(until(&mut h, 400, |h| h.app.world.plans[0].phase
        == PlanPhase::Done));
    assert_eq!(
        h.app.world.plans[0].steps[7].state,
        StepState::Succeeded,
        "verification runs once the retry passes"
    );
}

#[test]
fn activities_survive_navigation_and_merged_logs_keep_identity() {
    let mut h = H::new(Scenario::ActivitiesMulti, Motion::Reduced, 0, 120, 40);
    assert!(
        h.row(2).contains("frontend dev") && h.row(2).contains("btm"),
        "{}",
        h.row(2)
    );
    assert!(h.text().contains("Running"));
    h.alt(KeyCode::Char('4'));
    assert!(h.text().contains("merged logs"), "{}", h.text());
    assert!(h.text().contains("worker") && h.text().contains("scheduler"));
    let before = h.app.world.activities[3].output.len();
    h.key(KeyCode::Char('2'));
    assert!(
        !h.text().contains("worker     INFO"),
        "worker stream hidden: {}",
        h.text()
    );
    assert!(h.text().contains("scheduler  INFO"));
    h.key(KeyCode::Char('2'));
    h.alt(KeyCode::Char('0'));
    assert_eq!(h.tab_kind(), TabKind::Here);
    h.ticks(30);
    h.alt(KeyCode::Char('4'));
    assert!(
        h.app.world.activities[3].output.len() >= before,
        "output retained and growing"
    );
    assert!(
        h.text().contains("api        INFO  listening"),
        "earliest lines are still there: {}",
        h.text()
    );
    // attach and detach a monitor
    h.alt(KeyCode::Char('5'));
    assert!(h.text().contains("btm screen"));
    h.key(KeyCode::Enter);
    assert!(h.row(38).contains("attached"), "{}", h.row(38));
    assert!(h.last_row().contains("Detach"));
    h.key(KeyCode::Char('q'));
    assert!(h.text().contains("program exited"));
    // the activities picker
    h.ctrl(KeyCode::Char('g'));
    assert!(
        h.text().contains("Activities") && h.text().contains("api tests"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert_eq!(h.tab_kind(), TabKind::Activity("act-3".into()));
    assert!(h.text().contains("128 tests run: 128 passed"));
    // closing a running tab asks; closing a finished one does not
    h.ctrl(KeyCode::Char('w'));
    assert_eq!(h.app.tabs.len(), 5);
    h.alt(KeyCode::Char('1'));
    h.ctrl(KeyCode::Char('w'));
    assert!(h.text().contains("Stop frontend dev?"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert_eq!(h.app.tabs.len(), 5);
}

#[test]
fn nested_child_trusts_then_runs_in_the_child_directory() {
    let mut h = H::new(Scenario::MonorepoChild, Motion::Reduced, 0, 120, 40);
    assert!(
        h.text().contains("From acme · the parent root"),
        "{}",
        h.text()
    );
    assert!(
        h.text().contains("Start development ecosystem")
            || h.text().contains("Start required containers")
    );
    h.type_str("test");
    assert!(
        h.row(7).contains("Run tests") && h.row(7).contains("alias test"),
        "{}",
        h.row(7)
    );
    assert!(h.last_row().contains("Trust…"));
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Trust ~/work/acme/apps/frontend/mise.toml?"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("[tasks.test]"));
    assert_eq!(
        h.app.focus.current(),
        Some(crate::screens::review::TRUST_CANCEL)
    );
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.app
            .world
            .trusted_now
            .iter()
            .any(|c| c.ends_with("apps/frontend/mise.toml"))
    );
    assert!(
        h.text().contains("arguments"),
        "structured arguments follow trust: {}",
        h.text()
    );
    h.ctrl(KeyCode::Char('s'));
    assert!(matches!(h.tab_kind(), TabKind::Activity(_)));
    assert!(
        h.text().contains("apps/frontend"),
        "runs in the child: {}",
        h.text()
    );
    assert!(h.text().contains("mise run //apps/frontend:test"));
}

#[test]
fn scope_axis_and_tokens_narrow_and_widen() {
    let mut h = H::new(Scenario::MonorepoRoot, Motion::Reduced, 0, 120, 40);
    h.ctrl(KeyCode::Down);
    assert!(h.text().contains("scope ‹ children ›"), "{}", h.text());
    assert!(h.text().contains("//apps/frontend:dev"));
    h.ctrl(KeyCode::Up);
    h.ctrl(KeyCode::Up);
    assert!(h.text().contains("scope ‹ parent ›"));
    assert!(h.text().contains("No project root above"));
    h.ctrl(KeyCode::Up);
    assert!(h.text().contains("scope ‹ system ›"));
    assert!(h.text().contains("Clean Docker completely"));
    h.key(KeyCode::Esc);
    assert!(h.text().contains("scope ‹ here ›"));
    h.type_str("@children dev");
    assert!(h.text().contains("//services/api:dev"), "{}", h.text());
    assert!(
        !h.text().contains("Start development ecosystem"),
        "tokens filter exactly"
    );
}

#[test]
fn disk_cleanup_selects_by_freshness_and_gates_on_the_path() {
    let mut h = H::new(Scenario::DiskCleanup, Motion::Paused, 80, 120, 40);
    assert!(h.text().contains("Here › Disk › Usage"));
    assert!(h.text().contains("[✓] backend › target"), "{}", h.text());
    assert!(
        h.text().contains("[ ] frontend › dist"),
        "recent artifacts start unselected"
    );
    assert!(h.text().contains("activity unknown"));
    assert!(h.text().contains("scan complete"));
    h.key(KeyCode::Char('c'));
    assert!(
        h.text().contains("Clean developer artifacts under ~/work"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("join"));
    h.key(KeyCode::Char('c'));
    assert!(h.text().contains("gate 1 of 2"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Type REMOVE ALL BUILD ARTIFACTS UNDER /Users/alex/work to confirm"),
        "{}",
        h.text()
    );
    // the streaming scan at an early frame shows fewer candidates
    let early = H::new(Scenario::DiskCleanup, Motion::Paused, 10, 120, 40);
    assert!(early.text().contains("scanning"));
    assert!(!early.text().contains("~/Library/Logs"));
}

#[test]
fn remote_host_is_unmistakable_and_binds_the_phrase_to_the_host() {
    let mut h = H::new(Scenario::RemoteHost, Motion::Reduced, 0, 120, 40);
    assert!(
        h.row(0).contains("◆ prod-eu-1 · production"),
        "{}",
        h.row(0)
    );
    assert!(h.text().contains("on prod-eu-1"));
    h.type_str("restart payments");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("gate 1 of 2"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Type RESTART PAYMENTS ON prod-eu-1 to confirm"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    for _ in 0..16 {
        h.key(KeyCode::Backspace);
    }
    h.type_str("blocking");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Who is blocking the database?"));
    assert!(h.text().contains("└ 48211"), "blocker tree: {}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    h.type_str("resources");
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("System resources") && h.text().contains("Memory"),
        "{}",
        h.text()
    );
    assert!(h.text().contains("Open btm"), "btm handoff offered");
}

#[test]
fn alternatives_pin_alias_hide_and_why() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.alt(KeyCode::Enter);
    assert!(
        h.text().contains("Run now")
            && h.text().contains("Pin here")
            && h.text().contains("Why is this here?"),
        "{}",
        h.text()
    );
    h.key(KeyCode::Esc);
    // pin the second row: the menu ends with Pin · Alias · Hide · Reset · Why
    h.key(KeyCode::Down);
    h.alt(KeyCode::Enter);
    h.key(KeyCode::End);
    for _ in 0..4 {
        h.key(KeyCode::Up);
    }
    h.key(KeyCode::Enter);
    assert!(h.last_row().contains("Pinned"), "{}", h.last_row());
    assert!(
        h.row(7).contains("pinned here"),
        "pinned row rises: {}",
        h.row(7)
    );
    // alias
    h.alt(KeyCode::Enter);
    h.key(KeyCode::End);
    for _ in 0..3 {
        h.key(KeyCode::Up);
    }
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Set an alias"), "{}", h.text());
    h.key(KeyCode::Enter);
    h.type_str("tt");
    h.key(KeyCode::Enter);
    assert!(h.last_row().contains("Alias tt set"), "{}", h.last_row());
    h.type_str("tt");
    assert!(h.row(7).contains("alias tt"), "{}", h.row(7));
    h.key(KeyCode::Esc);
    // why is this here
    h.alt(KeyCode::Enter);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Why is “"), "{}", h.text());
    assert!(h.text().contains("never changes the rank"));
    h.key(KeyCode::Esc);
    // hide
    h.alt(KeyCode::Enter);
    h.key(KeyCode::End);
    h.key(KeyCode::Up);
    h.key(KeyCode::Up);
    h.key(KeyCode::Enter);
    assert!(h.last_row().contains("Hidden here"), "{}", h.last_row());
}

#[test]
fn launch_failure_keeps_output_and_offers_follow_ups() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Reduced, 30, 120, 40);
    assert!(matches!(h.tab_kind(), TabKind::Activity(_)));
    assert!(h.text().contains("EADDRINUSE"), "{}", h.text());
    assert!(h.text().contains("failed") && h.text().contains("exit 1"));
    assert!(
        h.row(2).contains("!"),
        "the tab carries the error glyph: {}",
        h.row(2)
    );
    assert!(h.text().contains("Find the process on port 5173"));
    h.key(KeyCode::Tab);
    assert!(
        h.app
            .focus
            .current()
            .is_some_and(|f| f != crate::screens::activity::VIEW)
    );
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Port 5173"), "{}", h.text());
    assert!(h.text().contains("node (stale vite)"));
    assert!(h.text().contains("Stop node (stale vite) (pid 5120)"));
}

#[test]
fn hard_cases_stay_legible_and_report_failed_discovery() {
    let mut h = H::new(Scenario::HardCases, Motion::Reduced, 0, 100, 30);
    assert!(h.text().contains("…"), "long labels truncate");
    assert!(h.row(28).contains("unavailable"), "{}", h.row(28));
    assert!(h.text().contains("Children · 14 projects"));
    h.resize(80, 24);
    assert!(
        h.row(22).contains("northwind"),
        "the path survives narrow: {}",
        h.row(22)
    );
    h.resize(100, 30);
    h.ctrl(KeyCode::Up);
    h.ctrl(KeyCode::Up);
    h.type_str("docker");
    assert!(h.text().contains("Docker"), "{}", h.text());
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Cannot connect to the Docker daemon"),
        "{}",
        h.text()
    );
    let wide = H::new(Scenario::HardCases, Motion::Reduced, 0, 160, 50);
    assert!(wide.text().contains("northwind-traders-platform-migration"));
}

#[test]
fn menu_bar_help_and_about() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    h.key(KeyCode::F(10));
    assert!(
        h.text().contains("Alternatives…") && h.text().contains("Close tab"),
        "{}",
        h.text()
    );
    assert!(h.last_row().contains("Menu"));
    h.key(KeyCode::Right);
    assert!(h.text().contains("Parent scope") && h.text().contains("Activities…"));
    h.key(KeyCode::Right);
    h.key(KeyCode::End);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("About holla❯"), "{}", h.text());
    h.key(KeyCode::Esc);
    h.key(KeyCode::F(1));
    assert!(h.text().contains("Key reference"));
    assert_eq!(h.text().matches("Esc Close").count(), 1, "one hint surface");
    h.key(KeyCode::Esc);
    let (x, y) = h.find("Go").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Children scope"));
    let (x, y) = h.find("Disk").unwrap();
    h.click(x, y);
    assert!(h.text().contains("Here › Disk"), "{}", h.text());
}

#[test]
fn narrow_root_uses_a_summary_line_and_a_preview_drawer() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 80, 24);
    assert!(
        h.text()
            .contains("→ git -C /Users/alex/work/holla pull --ff-only"),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("Does"));
    h.key(KeyCode::Tab);
    assert!(
        h.text().contains("Does") && h.text().contains("Gate"),
        "drawer covers the list: {}",
        h.text()
    );
    assert!(!h.text().contains("Suggested here"));
    h.key(KeyCode::Esc);
    assert!(h.text().contains("Suggested here"));
    // typing goes to the query even after the drawer
    h.type_str("du");
    assert!(
        h.row(7).contains("Analyze disk usage") && h.row(7).contains("alias du"),
        "{}",
        h.row(7)
    );
}

#[test]
fn esc_ladder_and_quit_rules() {
    let mut h = H::new(Scenario::FirstUse, Motion::Reduced, 0, 100, 30);
    h.type_str("disk");
    h.ctrl(KeyCode::Up);
    assert!(h.text().contains("scope ‹ parent ›"));
    h.key(KeyCode::Esc);
    assert!(
        !h.row(4).contains("disk"),
        "first Esc clears the query: {}",
        h.row(4)
    );
    assert!(h.text().contains("scope ‹ parent ›"));
    h.key(KeyCode::Esc);
    assert!(
        h.text().contains("scope ‹ here ›"),
        "second Esc returns to here"
    );
    assert!(!h.app.quit);
    h.key(KeyCode::Esc);
    assert!(
        h.app.quit,
        "Esc at an empty root with nothing running quits"
    );
    let mut a = H::new(Scenario::ActivitiesMulti, Motion::Reduced, 0, 100, 30);
    a.key(KeyCode::Esc);
    assert!(!a.app.quit);
    assert!(a.text().contains("Quit holla❯?"), "{}", a.text());
    a.key(KeyCode::Esc);
    a.ctrl(KeyCode::Char('q'));
    assert!(a.text().contains("Stop and quit"));
    a.key(KeyCode::Right);
    a.key(KeyCode::Enter);
    assert!(a.app.quit);
}

#[test]
fn mouse_selects_runs_and_opens_alternatives() {
    let mut h = H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40);
    let (x, y) = h.find("Run tests").unwrap();
    h.click(x + 2, y);
    assert!(h.row(y).contains("›"), "{}", h.row(y));
    h.mouse(MouseKind::Secondary, x + 2, y);
    assert!(h.text().contains("Arguments…"), "{}", h.text());
    h.key(KeyCode::Esc);
    let (sx, sy) = h.find("scope ‹ here ›").unwrap();
    h.click(sx + 2, sy);
    assert!(
        h.text().contains("scope ‹ parent ›") || h.text().contains("scope ‹ system ›"),
        "{}",
        h.text()
    );
    h.click(3, 38);
    assert!(h.last_row().contains("Copied"), "{}", h.last_row());
}

#[test]
fn every_scenario_is_deterministic_at_a_frame() {
    for s in Scenario::ALL {
        let a = H::new(s, Motion::Paused, 25, 100, 30);
        let b = H::new(s, Motion::Paused, 25, 100, 30);
        assert_eq!(a.text(), b.text(), "{s:?}");
    }
}
