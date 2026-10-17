//! Reachability gate (owner rule 2026-10-17): every screen and overlay of
//! holla keeps all of its content reachable at every size of the matrix:
//! shown, scrolled to, or cut with its `…` reveal
//! ([`junie_tui::shell::reach`]). Known gaps are the `holla` lines of
//! `tests/reach_allow.txt`, each with its finding id
//! (docs/audits/design-review-2026-10/responsive.md); the gate fails on a
//! new gap and on a fixed one whose line is still there.
//!
//! `REACH_DUMP=<case>:<w>:<h>` prints one case's frame at a size
//! (`cargo nextest run --bin holla reach_dump --no-capture`).

use junie_tui::shell::reach::{Case, Harness, Reveal};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;

use crate::app::App;
use crate::app_tests::H;
use crate::app_tests_context::shift_f10;
use crate::app_tests_parity::open;
use crate::scenario::{Motion, Scenario};
use crate::screens::{Go, Page};

impl Harness for H {
    type App = App;

    fn fresh() -> Self {
        H::new(Scenario::RustDirty, Motion::Reduced, 0, 120, 40)
    }

    fn into_parts(self) -> (App, Terminal<TestBackend>) {
        (self.app, self.term)
    }

    fn from_parts(app: App, term: Terminal<TestBackend>) -> Self {
        H { app, term }
    }
}

const fn page(name: &'static str, base: fn(&mut H)) -> Case<H> {
    Case::page(name, base)
}

const fn layer(name: &'static str, base: fn(&mut H), open: fn(&mut H)) -> Case<H> {
    Case::layer(name, base, open)
}

fn at(h: &mut H, scenario: Scenario) {
    *h = H::new(scenario, Motion::Reduced, 0, 120, 40);
}

fn paused(h: &mut H, scenario: Scenario) {
    *h = H::new(scenario, Motion::Paused, 40, 120, 40);
}

fn push(h: &mut H, page: Page) {
    h.app.go(Go::Push(page));
    h.draw();
}

fn open_disk(h: &mut H) {
    at(h, Scenario::ParityDiskNavigation);
    h.ticks(2);
    open(h, "Analyze disk usage");
    h.ticks(40);
}

fn open_files(h: &mut H) {
    *h = H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 120, 40);
    open(h, "Browse ~/work/site");
    h.ticks(8);
}

fn open_cleanup(h: &mut H) {
    paused(h, Scenario::ParityInsights);
    push(h, Page::Cleanup { category: None });
}

fn docker_plan(h: &mut H) {
    at(h, Scenario::DockerCleanup);
    h.key(KeyCode::Enter);
}

fn activities(h: &mut H) {
    at(h, Scenario::ActivitiesMulti);
    h.alt(KeyCode::Char('1'));
}

/// A list row cut with `…` shows in full in the preview beside the list
/// as the cursor steps onto it.
const ROWS: &[Reveal<H>] = &[Reveal {
    step: |h| {
        h.key(KeyCode::Down);
    },
    times: 20,
}];

fn cases() -> Vec<Case<H>> {
    vec![
        // the Here tab and the pages it pushes
        page("finder", |_| {}).reveals(ROWS),
        page("finder-children", |h| {
            at(h, Scenario::MonorepoRoot);
            h.ctrl(KeyCode::Down);
        }),
        page("finder-history", |h| {
            at(h, Scenario::ParityHistory);
            h.ticks(8);
        }),
        page("finder-page", |h| {
            paused(h, Scenario::FirstUse);
            push(h, Page::Finder { group: None });
        }),
        page("disk", open_disk).reveals(ROWS),
        page("disk-overview", |h| {
            at(h, Scenario::ParityDiskNavigation);
            h.ticks(10);
            open(h, "Disk overview");
        }),
        page("top-files", |h| {
            *h = H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 40, 120, 40);
            push(h, Page::TopFiles);
        }),
        page("files", open_files).reveals(ROWS),
        page("files-find", |h| {
            at(h, Scenario::ParityFiles);
            h.ticks(4);
            open(h, "Find files under home");
        }),
        page("cleanup", open_cleanup).reveals(ROWS),
        page("cleanup-report", |h| {
            at(h, Scenario::ParityCleanupResults);
            push(h, Page::Report { index: 0 });
        }),
        page("cleanup-history", |h| {
            at(h, Scenario::ParityCleanupResults);
            h.ticks(2);
            open(h, "Show cleanup history");
        }),
        page("plan-review", docker_plan),
        page("upgrade-plan", |h| at(h, Scenario::UpgradePlan)),
        page("trust", |h| {
            at(h, Scenario::MonorepoChild);
            h.type_str("test");
            h.key(KeyCode::Enter);
        }),
        page("args", |h| {
            paused(h, Scenario::RustDirty);
            let item = h
                .app
                .world
                .items()
                .into_iter()
                .find(|i| !i.args.is_empty())
                .expect("an item with arguments")
                .id;
            push(h, Page::Args { item });
        }),
        page("snapshot", |h| {
            paused(h, Scenario::RustDirty);
            push(
                h,
                Page::Snapshot {
                    kind: "port".into(),
                },
            );
        }),
        page("config", |h| {
            paused(h, Scenario::ParityCustomActions);
            open(h, "Custom action configuration");
        }),
        // activity and plan tabs
        page("activity", activities),
        page("activity-merged", |h| {
            at(h, Scenario::ActivitiesMulti);
            h.alt(KeyCode::Char('4'));
        }),
        page("activity-failed", |h| {
            *h = H::new(Scenario::LaunchFailure, Motion::Reduced, 30, 120, 40);
        }),
        page("plan-running", |h| {
            at(h, Scenario::UpgradePlan);
            h.key(KeyCode::Char('c'));
            h.key(KeyCode::Right);
            h.key(KeyCode::Enter);
        }),
        // menus and dialogs
        layer(
            "menu-file",
            |_| {},
            |h| {
                h.key(KeyCode::F(10));
            },
        ),
        layer(
            "help",
            |_| {},
            |h| {
                h.key(KeyCode::F(1));
            },
        ),
        layer(
            "about",
            |_| {},
            |h| {
                h.key(KeyCode::F(10));
                h.key(KeyCode::Right);
                h.key(KeyCode::Right);
                h.key(KeyCode::End);
                h.key(KeyCode::Enter);
            },
        ),
        layer(
            "alternatives",
            |_| {},
            |h| {
                h.alt(KeyCode::Enter);
            },
        ),
        layer(
            "why",
            |_| {},
            |h| {
                h.alt(KeyCode::Enter);
                h.key(KeyCode::End);
                h.key(KeyCode::Enter);
            },
        ),
        layer(
            "alias",
            |_| {},
            |h| {
                h.alt(KeyCode::Enter);
                h.key(KeyCode::End);
                for _ in 0..3 {
                    h.key(KeyCode::Up);
                }
                h.key(KeyCode::Enter);
            },
        ),
        layer(
            "activities-picker",
            |h| paused(h, Scenario::ActivitiesMulti),
            |h| {
                h.ctrl(KeyCode::Char('g'));
            },
        ),
        layer(
            "quit-confirm",
            |h| paused(h, Scenario::ActivitiesMulti),
            |h| {
                h.ctrl(KeyCode::Char('q'));
            },
        ),
        layer("close-tab", activities, |h| {
            h.ctrl(KeyCode::Char('w'));
        }),
        layer(
            "confirm-one",
            |h| at(h, Scenario::RemoteHost),
            |h| {
                h.type_str("cancel the blocking");
                h.key(KeyCode::Enter);
            },
        ),
        layer(
            "start-plan",
            |h| at(h, Scenario::UpgradePlan),
            |h| {
                h.key(KeyCode::Char('c'));
            },
        ),
        layer("gate-plan", docker_plan, |h| {
            h.key(KeyCode::Char('c'));
        }),
        layer("gate-typed", docker_plan, |h| {
            h.key(KeyCode::Char('c'));
            h.key(KeyCode::Right);
            h.key(KeyCode::Enter);
        }),
        layer(
            "plan-step-menu",
            |h| {
                at(h, Scenario::UpgradePlan);
                for _ in 0..5 {
                    h.key(KeyCode::Down);
                }
            },
            shift_f10,
        ),
        layer("cleanup-menu", open_cleanup, shift_f10),
        layer("files-jump", open_files, |h| {
            h.key(KeyCode::Char('g'));
        }),
        layer("activity-menu", activities, shift_f10),
    ]
}

junie_tui::reach_tests! {
    "holla", H, cases();
    finder = "finder",
    finder_children = "finder-children",
    finder_history = "finder-history",
    finder_page = "finder-page",
    disk = "disk",
    disk_overview = "disk-overview",
    top_files = "top-files",
    files = "files",
    files_find = "files-find",
    cleanup = "cleanup",
    cleanup_report = "cleanup-report",
    cleanup_history = "cleanup-history",
    plan_review = "plan-review",
    upgrade_plan = "upgrade-plan",
    trust = "trust",
    args = "args",
    snapshot = "snapshot",
    config = "config",
    activity = "activity",
    activity_merged = "activity-merged",
    activity_failed = "activity-failed",
    plan_running = "plan-running",
    menu_file = "menu-file",
    help = "help",
    about = "about",
    alternatives = "alternatives",
    why = "why",
    alias = "alias",
    activities_picker = "activities-picker",
    quit_confirm = "quit-confirm",
    close_tab = "close-tab",
    confirm_one = "confirm-one",
    start_plan = "start-plan",
    gate_plan = "gate-plan",
    gate_typed = "gate-typed",
    plan_step_menu = "plan-step-menu",
    cleanup_menu = "cleanup-menu",
    files_jump = "files-jump",
    activity_menu = "activity-menu",
}
