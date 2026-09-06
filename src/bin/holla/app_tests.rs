//! End-to-end interaction tests through the real App on a TestBackend.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Position;

use junie_tui::core::event::{Input, Key, Mouse, MouseKind, Outcome};
use junie_tui::theme::Theme;

use crate::app::App;
use crate::scenario::{Motion, Scenario};

pub struct H {
    pub app: App,
    pub term: Terminal<TestBackend>,
}

impl H {
    pub fn new(scenario: Scenario, motion: Motion, frame: u64, w: u16, h: u16) -> Self {
        let app = App::for_scenario(scenario, motion, frame, Theme::junie());
        let term = Terminal::new(TestBackend::new(w, h)).unwrap();
        let mut hh = Self { app, term };
        hh.draw();
        hh
    }
    pub fn draw(&mut self) {
        self.term.draw(|f| self.app.render(f)).unwrap();
    }
    pub fn key(&mut self, code: KeyCode) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        }));
        self.draw();
        o
    }
    pub fn ctrl(&mut self, c: char) -> Outcome {
        let o = self.app.handle(Input::Key(Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::CONTROL,
        }));
        self.draw();
        o
    }
    pub fn type_str(&mut self, s: &str) {
        for c in s.chars() {
            self.key(KeyCode::Char(c));
        }
    }
    pub fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.app.handle(Input::Tick);
        }
        self.draw();
    }
    #[allow(dead_code)] // mouse paths grow in P1+
    pub fn mouse(&mut self, kind: MouseKind, x: u16, y: u16) -> Outcome {
        let o = self.app.handle(Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
        }));
        self.draw();
        o
    }
    #[allow(dead_code)] // mouse paths grow in P1+
    pub fn click(&mut self, x: u16, y: u16) {
        self.mouse(MouseKind::Down, x, y);
        self.mouse(MouseKind::Up, x, y);
    }
    pub fn resize(&mut self, w: u16, h: u16) {
        self.term.backend_mut().resize(w, h);
        self.app.handle(Input::Resize(w, h));
        self.draw();
    }
    pub fn text(&self) -> String {
        let buf = self.term.backend().buffer();
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }
    pub fn find(&self, needle: &str) -> Option<(u16, u16)> {
        let buf = self.term.backend().buffer();
        let want: Vec<&str> =
            unicode_segmentation::UnicodeSegmentation::graphemes(needle, true).collect();
        for y in 0..buf.area.height {
            let cells: Vec<&str> = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
            for x in 0..cells.len().saturating_sub(want.len() - 1) {
                if cells[x..x + want.len()] == want[..] {
                    return Some((x as u16, y));
                }
            }
        }
        None
    }
}

// --------------------------------------------------------- determinism

#[test]
fn paused_frames_are_byte_identical() {
    let a = H::new(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    let b = H::new(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
    assert_eq!(a.text(), b.text());
}

#[test]
fn paused_ticks_do_not_advance_the_world() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    let before = h.text();
    h.ticks(20);
    assert_eq!(before, h.text());
    // discovery is a virtual-clock event: paused means it never fires
    assert!(h.app.world.discovering());
}

#[test]
fn frame_seek_lands_on_discovered_state() {
    let h = H::new(Scenario::FirstUse, Motion::Paused, 4_000, 120, 40);
    assert!(!h.app.world.discovering());
    assert!(!h.text().contains("discovering…"), "{}", h.text());
    let early = H::new(Scenario::FirstUse, Motion::Paused, 100, 120, 40);
    assert!(early.app.world.discovering());
    assert!(early.text().contains("discovering…"), "{}", early.text());
}

// ------------------------------------------------------------- chrome

#[test]
fn chrome_has_brand_menu_crumb_host_and_hint_bar() {
    let h = H::new(Scenario::RustDirty, Motion::Paused, 0, 120, 40);
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
fn remote_production_host_is_unmistakable() {
    let h = H::new(Scenario::RemoteHost, Motion::Paused, 0, 120, 40);
    let t = h.text();
    assert!(t.contains("◆ prod-eu-1 · ssh · production"), "{t}");
    assert!(t.contains("/srv/payments"), "{t}");
}

#[test]
fn too_small_notice_below_minimum() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    h.resize(71, 19);
    let t = h.text();
    assert!(t.contains("Terminal too small"), "{t}");
    assert!(t.contains("Need 72×20, have 71×19"), "{t}");
    assert!(t.contains("holla❯"), "{t}");
    // q quits from the notice
    h.key(KeyCode::Char('q'));
    assert!(h.app.quit);
}

#[test]
fn menu_bar_opens_and_quit_is_confirmable() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    h.key(KeyCode::F(10));
    let t = h.text();
    assert!(t.contains("Quit"), "{t}");
    assert!(t.contains("Ctrl+Q"), "{t}");
    // the menu layer owns the hint bar while open
    let last = t.lines().nth(39).unwrap_or("");
    assert!(last.contains("Menu"), "{last:?}");
    h.key(KeyCode::Esc);
    // quit goes through a confirmation whose default is Cancel
    h.ctrl('q');
    let t = h.text();
    assert!(t.contains("Quit holla?"), "{t}");
    assert!(h.find("Cancel").is_some(), "{t}");
    h.key(KeyCode::Esc);
    assert!(!h.app.quit);
    assert!(!h.text().contains("Quit holla?"));
}

#[test]
fn help_opens_from_empty_query_question_mark() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    h.key(KeyCode::Char('?'));
    let t = h.text();
    assert!(t.contains("Key reference"), "{t}");
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Key reference"));
}

// -------------------------------------------------------------- query

#[test]
fn typing_filters_and_esc_clears() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.type_str("git");
    let t = h.text();
    assert!(t.contains("git"), "{t}");
    assert!(t.contains("Check git status"), "{t}");
    assert!(!t.contains("Connect to"), "{t}");
    h.key(KeyCode::Esc);
    assert!(h.text().contains("Connect to"), "{}", h.text());
}

#[test]
fn empty_query_q_asks_to_quit_but_typed_q_filters() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    h.key(KeyCode::Char('q'));
    assert!(h.text().contains("Quit holla?"));
    h.key(KeyCode::Esc);
    // with text in the query, q is just a character
    h.type_str("se");
    let before = h.app.home.query.clone();
    h.key(KeyCode::Char('q'));
    assert_eq!(h.app.home.query, format!("{before}q"));
    assert!(!h.text().contains("Quit holla?"));
}

#[test]
fn enter_on_top_match_reports_simulated_run() {
    // rust-dirty's pinned `make test` outranks everything at this path
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Would run: make test"), "{t}");
    assert!(t.contains("nothing executed"), "{t}");
}

#[test]
fn down_moves_into_results_and_back() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 4_000, 120, 40);
    h.type_str("monit");
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    // §8.6: Enter opens the snapshot; the btm handoff is one confirm away
    let t = h.text();
    assert!(t.contains("System snapshot"), "{t}");
    assert!(t.contains("Handoff"), "{t}");
    h.key(KeyCode::Right); // Close → Open btm (simulated)
    h.key(KeyCode::Enter);
    assert!(h.text().contains("simulated handoff"), "{}", h.text());
}

#[test]
fn nothing_matches_state() {
    let mut h = H::new(Scenario::FirstUse, Motion::Paused, 0, 120, 40);
    h.type_str("zzz");
    assert!(h.text().contains("Nothing matches"), "{}", h.text());
}

// ----------------------------------------------------- P2 root experience

#[test]
fn priority_stack_has_reasons_and_scope_tags() {
    let h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    assert!(t.contains("Suggested here"), "{t}");
    assert!(t.contains("Recent here"), "{t}");
    assert!(t.contains("Explore"), "{t}");
    // a reason on every suggestion: memory, freshness, git truth
    assert!(t.contains("pinned here"), "{t}");
    assert!(t.contains("used 6 times in this project"), "{t}");
    assert!(t.contains("branch is 3 commits behind"), "{t}");
    // nonlocal rows carry explicit scope
    assert!(t.contains("· host"), "{t}");
}

#[test]
fn preview_answers_the_seven_questions() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.ctrl('p');
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
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("Will happen"));
}

#[test]
fn scope_cycle_narrows_and_esc_widens() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.ctrl('s');
    let t = h.text();
    assert!(t.contains("scope: here"), "{t}");
    assert!(!t.contains("Connect to prod-eu-1"), "{t}");
    h.ctrl('s');
    h.ctrl('s');
    h.ctrl('s');
    assert!(h.text().contains("scope: host"), "{}", h.text());
    assert!(h.text().contains("Connect to prod-eu-1"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("scope: host"), "{}", h.text());
}

#[test]
fn mise_trust_gate_then_child_task_runs() {
    let mut h = H::new(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    h.type_str("reset-db");
    let t = h.text();
    assert!(t.contains("untrusted config"), "{t}");
    assert!(t.contains("▲"), "{t}");
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Trust this task file?"), "{t}");
    assert!(
        t.contains("~/work/monorepo/projects/backend/mise.toml"),
        "{t}"
    );
    assert!(t.contains("this exact file only"), "{t}");
    // focus starts on Close; move right to Trust file, confirm
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Trusted mise.toml"), "{}", h.text());
    // the task is now ready and runs simulated
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Would run: mise run //projects/backend:reset-db"),
        "{}",
        h.text()
    );
}

#[test]
fn monorepo_child_shows_parent_ecosystem_with_scope_tags() {
    let h = H::new(Scenario::MonorepoChild, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    // local frontend task ranks; parent workspace rows are tagged
    assert!(t.contains("Run dev"), "{t}");
    assert!(t.contains("workspace"), "{t}");
}

#[test]
fn clone_flow_collects_argument_then_reviews() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.type_str("clone");
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Clone which repository?"), "{t}");
    assert!(t.contains("pave-io/pave"), "{t}");
    h.key(KeyCode::Enter);
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
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Would clone pave-io/pave"),
        "{}",
        h.text()
    );
}

#[test]
fn long_running_task_starts_named_activity() {
    let mut h = H::new(Scenario::MonorepoChild, Motion::Paused, 4_000, 120, 40);
    let before = h.app.world.activities.len();
    h.type_str("vite");
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Activity started: Run dev"), "{t}");
    assert_eq!(h.app.world.activities.len(), before + 1);
    let a = h.app.world.activities.last().unwrap();
    assert_eq!(a.scope, "~/work/monorepo/apps/frontend");
    assert_eq!(a.state, crate::domain::activity::ActivityState::Running);
}

#[test]
fn launch_failure_is_honest() {
    let mut h = H::new(Scenario::LaunchFailure, Motion::Paused, 4_000, 120, 40);
    h.type_str("mise run test");
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("failed · exit code 1"), "{t}");
    let a = h.app.world.activities.last().unwrap();
    assert_eq!(a.state, crate::domain::activity::ActivityState::Failed);
}

#[test]
fn hard_cases_shows_docker_failure_and_detached_head() {
    let h = H::new(Scenario::HardCases, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    assert!(t.contains("docker: discovery failed"), "{t}");
    assert!(t.contains("detached at 9f3a21c"), "{t}");
}

#[test]
fn actions_menu_offers_alternatives_and_pinning() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.type_str("cargo build");
    h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Pin here"), "{t}");
    assert!(t.contains("Copy command"), "{t}");
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    // pinning a row makes it outrank everything next time
    h.type_str("status");
    h.ctrl('o');
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Pinned git status here"), "{}", h.text());
    h.key(KeyCode::Esc);
    assert!(h.app.world.memory.pin_at("~/work/pave", "git status"));
}

// ---------------------------------------------------------- P3 safety + plans

use crate::app::Route;
use crate::domain::docker::{ContainerState, Health};

/// Open the docker cleanup plan from home.
fn open_docker_plan(h: &mut H) {
    h.type_str("clean up docker");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Plan, "{}", h.text());
}

/// Type the gate phrase, move focus to the buttons, reach for Run.
fn try_confirm(h: &mut H, phrase: &str) {
    h.key(KeyCode::Enter); // open gate 2
    h.type_str(phrase);
    h.key(KeyCode::Enter); // leave the ack input
    h.key(KeyCode::Right); // Cancel → Run plan (skipped while disabled)
    h.key(KeyCode::Enter);
}

#[test]
fn broad_action_opens_gate_one_review_surface() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
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
    let mut h = H::new(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);
    // step 0 is required and refuses exclusion
    h.key(KeyCode::Char(' '));
    assert!(
        h.text().contains("required · cannot exclude"),
        "{}",
        h.text()
    );
    // exclude the container removal: images + volumes recalculate
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    let t = h.text();
    assert!(t.contains("Excluded Remove stopped containers"), "{t}");
    assert!(t.contains("needs Remove stopped containers"), "{t}");
    // parallel branches stay free
    assert!(!t.contains("needs Inspect Docker usage"), "{t}");
    // restoring clears the block
    h.key(KeyCode::Char(' '));
    assert!(
        !h.text().contains("needs Remove stopped containers"),
        "{}",
        h.text()
    );
}

#[test]
fn invalid_phrase_cannot_proceed() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    open_docker_plan(&mut h);
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(
        t.contains("Type REMOVE ALL DOCKER DATA ON devbox to confirm"),
        "{t}"
    );
    // wrong host bound: the phrase names devbox, not prod-eu-1
    h.type_str("REMOVE ALL DOCKER DATA ON prod-eu-1");
    h.key(KeyCode::Enter); // leave input → Cancel
    h.key(KeyCode::Right); // Run stays disabled, focus cannot land on it
    h.key(KeyCode::Enter); // the only reachable action is Cancel
    let t = h.text();
    assert!(!t.contains("Finished:"), "{t}");
    assert!(!h.app.plan.as_ref().unwrap().plan.ran);
}

#[test]
fn correct_phrase_runs_and_failure_propagates() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
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
    let plan = &h.app.plan.as_ref().unwrap().plan;
    assert!(plan.ran);
    // honest world mutation: cache pruned, web/cron gone, payments-old kept
    let d = h.app.world.docker.as_ref().unwrap();
    assert_eq!(d.build_cache_bytes, 0);
    assert!(!d.containers.iter().any(|c| c.name == "web"));
    assert!(d.containers.iter().any(|c| c.name == "payments-old"));
    // per-step output: focus the failed step, its lines show
    h.key(KeyCode::Down);
    let t = h.text();
    assert!(t.contains("Output — Remove stopped containers"), "{t}");
    assert!(t.contains("Removed web"), "{t}");
}

#[test]
fn debian_plan_parallel_branches_and_i_understand_phrase() {
    let mut h = H::new(Scenario::UpgradePlan, Motion::Paused, 4_000, 120, 40);
    h.type_str("upgrade everything");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("· apt"), "{t}");
    assert!(t.contains("· mise"), "{t}");
    // exclude Apply upgrades: autoremove and verify recalculate, mise free
    h.key(KeyCode::Down);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    let t = h.text();
    assert!(t.contains("needs Apply upgrades"), "{t}");
    try_confirm(&mut h, "I UNDERSTAND: UPGRADE EVERYTHING ON devbox-deb");
    let t = h.text();
    assert!(t.contains("Finished:"), "{t}");
    assert!(t.contains("skipped"), "{t}");
    // apt never applied; the mise branch still ran
    assert_eq!(h.app.world.debian.as_ref().unwrap().pending, 47);
    let mise = h.app.world.mise.as_ref().unwrap();
    assert!(
        mise.tools
            .iter()
            .all(|t| !matches!(t.state, crate::domain::mise::ToolState::Outdated { .. }))
    );
}

#[test]
fn production_restart_goes_through_two_gates() {
    let mut h = H::new(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    h.type_str("restart");
    h.key(KeyCode::Enter);
    // broad on production: the review surface, not a one-shot confirm
    assert_eq!(h.app.route, Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Restart payments"), "{t}");
    try_confirm(&mut h, "RESTART PAYMENTS ON prod-eu-1");
    assert!(h.text().contains("Finished:"), "{}", h.text());
    let d = h.app.world.docker.as_ref().unwrap();
    let c = d.containers.iter().find(|c| c.name == "payments").unwrap();
    assert_eq!(c.state, ContainerState::Running);
    assert_eq!(c.health, Some(Health::Healthy));
    // back home, the urgency row is gone — the fixture kept its word
    h.key(KeyCode::Esc);
    assert!(!h.text().contains("service is unhealthy"), "{}", h.text());
}

#[test]
fn disk_plan_policy_skips_active_today() {
    let mut h = H::new(Scenario::DiskCleanup, Motion::Paused, 4_000, 120, 40);
    h.type_str("reclaim disk");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Plan, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Remove node_modules"), "{t}");
    assert!(t.contains("active today · never removed"), "{t}");
    assert!(t.contains("Remove .gradle"), "{t}");
}

// ------------------------------------------------------------ P4 activities

#[test]
fn strip_lists_seeded_activities_with_states() {
    let h = H::new(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    let t = h.text();
    // the strip docks under the menu bar on every route
    let row1: String = t.lines().nth(1).unwrap().to_owned();
    assert!(row1.contains("● 1 dev server"), "{row1}");
    assert!(row1.contains("… 2 test watch"), "{row1}");
    assert!(row1.contains("− 3 deploy logs"), "{row1}");
    assert!(row1.contains("✗ 4 seed db"), "{row1}");
    assert!(row1.contains("0 home"), "{row1}");
}

#[test]
fn digit_switches_to_activity_page() {
    let mut h = H::new(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    // home digits stay in the query — switching starts from a strip click
    h.type_str("1");
    assert_eq!(h.app.route, Route::Home, "home digits filter the query");
    h.key(KeyCode::Backspace);
    // click the strip tab for "seed db"
    let (x, _) = h.find("4 seed db").unwrap();
    h.click(x + 1, 1);
    assert_eq!(h.app.route, Route::Activity, "{}", h.text());
    let t = h.text();
    assert!(t.contains("Activity · seed db"), "{t}");
    assert!(t.contains("failed"), "{t}");
    assert!(t.contains("psql: connection to server"), "{t}");
    // now digits switch between activities
    h.key(KeyCode::Char('1'));
    assert!(h.text().contains("Activity · dev server"), "{}", h.text());
    assert!(h.text().contains("http://localhost:5199/"), "{}", h.text());
    // 0 returns to the root without losing anything
    h.key(KeyCode::Char('0'));
    assert_eq!(h.app.route, Route::Home);
    assert_eq!(h.app.world.activities.len(), 4);
}

#[test]
fn switching_away_and_back_preserves_output_and_scope() {
    let mut h = H::new(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    // open activity 1 via the strip, note the page
    let (x, _) = h.find("1 dev server").unwrap();
    h.click(x + 1, 1);
    assert_eq!(h.app.route, Route::Activity);
    let page = h.text();
    assert!(page.contains("Scope"), "{page}");
    assert!(page.contains("~/work/monorepo/apps/frontend"), "{page}");
    assert!(page.contains("vite v5.4 ready in 412 ms"), "{page}");
    // leave for another activity, then home
    h.key(KeyCode::Char('4'));
    assert!(h.text().contains("seed db"), "{}", h.text());
    h.key(KeyCode::Char('0'));
    assert_eq!(h.app.route, Route::Home);
    // back again: same output, same scope, nothing lost
    let (x, _) = h.find("1 dev server").unwrap();
    h.click(x + 1, 1);
    let back = h.text();
    assert!(back.contains("~/work/monorepo/apps/frontend"), "{back}");
    assert!(back.contains("vite v5.4 ready in 412 ms"), "{back}");
    assert!(back.contains("running · 2h ago"), "{back}");
}

#[test]
fn follow_logs_merges_services_with_identity() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    h.type_str("follow container logs");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Activity, "{}", h.text());
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
    h.key(KeyCode::Char('0'));
    assert_eq!(h.app.route, Route::Home);
    assert!(
        h.app
            .world
            .activities
            .iter()
            .any(|a| a.name == "container logs")
    );
}

#[test]
fn merged_logs_scroll_retained_per_activity() {
    let mut h = H::new(Scenario::DockerCleanup, Motion::Paused, 4_000, 120, 40);
    h.type_str("follow container logs");
    h.key(KeyCode::Enter);
    // stream is short at 120x40 body, scroll up then away and back
    h.key(KeyCode::Up);
    h.key(KeyCode::Up);
    h.key(KeyCode::Char('0'));
    let (x, _) = h.find("container logs").unwrap();
    h.click(x + 1, 1);
    assert_eq!(h.app.route, Route::Activity, "{}", h.text());
    assert!(h.text().contains("container logs"), "{}", h.text());
}

#[test]
fn ctrl_a_cycles_activities_from_any_route() {
    let mut h = H::new(Scenario::ActivitiesMulti, Motion::Paused, 4_000, 120, 40);
    // keyboard never needs the mouse: Ctrl+A enters the strip from home
    h.ctrl('a');
    assert_eq!(h.app.route, Route::Activity, "{}", h.text());
    assert!(h.text().contains("Activity · dev server"), "{}", h.text());
    // same chord advances to the next activity
    h.ctrl('a');
    assert!(h.text().contains("Activity · test watch"), "{}", h.text());
    // wraps around
    h.ctrl('a');
    h.ctrl('a');
    h.ctrl('a');
    assert!(h.text().contains("Activity · dev server"), "{}", h.text());
}

// ------------------------------------------------------- P5 depth + hard cases

#[test]
fn git_sync_plan_resolves_per_child_primary_branches() {
    let mut h = H::new(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    h.type_str("update all child");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Plan, "{}", h.text());
    let t = h.text();
    // primary branches come from each child, never a hard-coded main
    assert!(t.contains("Check out main (legacy)"), "{t}");
    assert!(t.contains("Check out trunk (billing)"), "{t}");
    // detached child is policy-skipped, not offered as excludable
    assert!(t.contains("detached at a1b2c3d · skipped"), "{t}");
    // gate 2 binds the phrase to the monorepo root
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("UPDATE ALL CHILD PROJECTS IN ~/work/monorepo"),
        "{}",
        h.text()
    );
}

#[test]
fn git_sync_run_per_child_results_and_honest_effect() {
    let mut h = H::new(Scenario::MonorepoRoot, Motion::Paused, 4_000, 120, 40);
    h.type_str("update all child");
    h.key(KeyCode::Enter);
    try_confirm(&mut h, "UPDATE ALL CHILD PROJECTS IN ~/work/monorepo");
    let t = h.text();
    // diverged billing fails ff-only; siblings unaffected
    assert!(t.contains("diverged · 2 ahead, 3 behind"), "{t}");
    assert!(t.contains("✓ Pull legacy — fast-forward only"), "{t}");
    assert!(t.contains("✗ Pull billing"), "{t}");
    let git = h.app.world.git.as_ref().unwrap();
    let legacy = git
        .children
        .iter()
        .find(|c| c.root.ends_with("legacy"))
        .unwrap();
    let billing = git
        .children
        .iter()
        .find(|c| c.root.ends_with("billing"))
        .unwrap();
    // behind zeroes only for the succeeded pull
    assert_eq!(legacy.behind, 0);
    assert_eq!(billing.behind, 3, "failed pull keeps its behind count");
    assert_eq!(billing.primary_branch, "trunk");
}

#[test]
fn pg_lock_tree_cancel_resumes_waiters() {
    let mut h = H::new(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    h.type_str("locks");
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Database lock tree"), "{t}");
    assert!(t.contains("pid 4201 · payments · ALTER TABLE"), "{t}");
    assert!(t.contains("cancel before terminate"), "{t}");
    // focus starts on Cancel blocker (the policy-recommended action)
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Cancelled pid 4201 · 2 waiting sessions resumed"),
        "{}",
        h.text()
    );
    // fixture truth: blocker gone, waiters unblocked
    let sessions = h.app.world.pg.as_ref().unwrap();
    assert!(!sessions.iter().any(|s| s.pid == 4201));
    assert!(sessions.iter().all(|s| s.blocked_by.is_none()));
    // reopening revalidates: no blocker, healthy status, no dialog
    h.key(KeyCode::Esc);
    h.type_str("locks");
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("No blockers · sessions healthy"),
        "{}",
        h.text()
    );
}

#[test]
fn pg_terminate_revalidates_before_killing() {
    let mut h = H::new(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    h.type_str("locks");
    h.key(KeyCode::Enter);
    // focus starts on Cancel blocker; one Right reaches Terminate
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Terminated pid 4201 after revalidation · 2 waiting sessions resumed"),
        "{}",
        h.text()
    );
    assert!(
        !h.app
            .world
            .pg
            .as_ref()
            .unwrap()
            .iter()
            .any(|s| s.pid == 4201)
    );
}

#[test]
fn ssh_resolution_previews_chain_and_identity_filename() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.type_str("prod-eu-1");
    h.ctrl('p');
    let t = h.text();
    assert!(t.contains("Connect to prod-eu-1?"), "{t}");
    assert!(t.contains("10.20.30.40"), "{t}");
    assert!(t.contains("bastion → prod-eu-1"), "{t}");
    // identity is a FILENAME only — contents never enter the UI
    assert!(t.contains("id_ed25519_prod · filename only"), "{t}");
    assert!(t.contains("Multiplexing"), "{t}");
}

#[test]
fn monitor_snapshot_facts_and_btm_handoff() {
    let mut h = H::new(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    h.type_str("monitor");
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("System snapshot"), "{t}");
    assert!(t.contains("prod-eu-1"), "{t}");
    assert!(t.contains("Load"), "{t}");
    assert!(t.contains("3.10 2.80 2.50"), "{t}");
    assert!(t.contains("14200 MB of 16384 MB"), "{t}");
    assert!(t.contains("212 days"), "{t}");
    assert!(t.contains("btm takes over the screen"), "{t}");
    h.key(KeyCode::Right); // Close → Open btm (simulated)
    h.key(KeyCode::Enter);
    assert!(h.text().contains("simulated handoff"), "{}", h.text());
}

#[test]
fn alias_prompt_saves_and_query_matches_expansion() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.type_str("cargo build");
    h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Set alias…"), "{t}");
    // Run, Preview, Copy, Pin, Alias
    for _ in 0..4 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    let t = h.text();
    assert!(t.contains("Alias for “cargo build”"), "{t}");
    h.type_str("cb");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Alias cb → cargo build"), "{}", h.text());
    assert_eq!(h.app.world.memory.alias_for("cargo build"), Some("cb"));
    // the alias query finds the command's row
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc); // clear the old query
    h.type_str("cb");
    assert!(h.text().contains("cargo build"), "{}", h.text());
}

#[test]
fn hide_removes_row_exact_query_resurfaces_and_reset_restores() {
    let mut h = H::new(Scenario::RustDirty, Motion::Paused, 4_000, 120, 40);
    h.type_str("cargo build");
    h.ctrl('o');
    // Run, Preview, Copy, Pin, Alias, Hide
    for _ in 0..5 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    assert!(
        h.text()
            .contains("Hidden cargo build here · Reset ranking restores"),
        "{}",
        h.text()
    );
    assert!(h.app.world.memory.hidden_at("~/work/pave", "cargo build"));
    // hidden: a partial query no longer finds it
    h.key(KeyCode::Esc);
    h.key(KeyCode::Esc);
    h.type_str("cargo b");
    assert!(
        !h.text().contains("cargo build · cargo build"),
        "{}",
        h.text()
    );
    assert!(!h.text().contains("used 6 times"), "{}", h.text());
    // the exact command resurfaces the row so it can be managed
    h.key(KeyCode::Esc);
    h.type_str("cargo build");
    h.ctrl('o');
    let t = h.text();
    assert!(t.contains("Unhide here"), "{t}");
    // Reset ranking clears pin, alias and hide (last menu item)
    for _ in 0..6 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Enter);
    assert!(
        h.text().contains("Reset ranking for cargo build"),
        "{}",
        h.text()
    );
    assert!(!h.app.world.memory.hidden_at("~/work/pave", "cargo build"));
}

#[test]
fn disk_reclaim_plan_matches_apply_effect() {
    let mut h = H::new(Scenario::DiskCleanup, Motion::Paused, 4_000, 120, 40);
    h.type_str("reclaim disk");
    h.key(KeyCode::Enter);
    assert_eq!(h.app.route, Route::Plan, "{}", h.text());
    let before = h.app.world.disk.as_ref().unwrap().candidates.len();
    let plan = &h.app.plan.as_ref().unwrap().plan;
    let removable = plan
        .steps
        .iter()
        .filter(|s| {
            s.id.starts_with("rm:")
                && !matches!(s.state, crate::domain::plan::StepState::PolicySkipped(_))
        })
        .count();
    assert!(removable >= 1);
    // freshness is the step branch: the dry run shows the same grouping
    // the reviewer saw, and active-today artifacts are policy-skipped
    let t = h.text();
    assert!(t.contains("inactive 31 days"), "{t}");
    assert!(t.contains("active today · never removed"), "{t}");
    h.key(KeyCode::Enter);
    let phrase = h.app.plan.as_ref().unwrap().plan.phrase.clone();
    h.type_str(&phrase);
    h.key(KeyCode::Enter);
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let after = h.app.world.disk.as_ref().unwrap().candidates.len();
    assert_eq!(after, before - removable, "dry-run steps ↔ removals parity");
}

#[test]
fn quit_confirm_on_production_names_the_remote_identity() {
    let mut h = H::new(Scenario::RemoteHost, Motion::Paused, 4_000, 120, 40);
    h.key(KeyCode::Char('q'));
    let t = h.text();
    assert!(t.contains("Quit holla on prod-eu-1?"), "{t}");
    assert!(t.contains("◆ prod-eu-1"), "{t}");
    // cancel is the default focus
    h.key(KeyCode::Enter);
    assert!(!h.app.quit);
}
