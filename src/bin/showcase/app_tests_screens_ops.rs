//! The Screens pages Disk cleanup, Activity and Workspaces through the
//! real app (plan §7 SC49, SC51, SC52): each page composes library
//! components, not the planned-page stub; options set through `--opt`
//! change the playground, the States rows render, and the key
//! interactions work.

use ratatui::crossterm::event::KeyCode;

use crate::app::PageId;
use crate::app_tests_feedback::Harness;
use crate::pages::template::Section;
use junie_tui::core::id::WidgetId;

fn typed(h: &mut Harness, s: &str) {
    for c in s.chars() {
        h.key(KeyCode::Char(c));
    }
}

// ------------------------------------------------------------ Disk cleanup

const DISK: WidgetId = WidgetId::of("diskcleanup");

#[test]
fn disk_cleanup_shows_the_flow_not_the_stub() {
    for (w, h) in [(120, 40), (80, 24)] {
        let mut h = Harness::new(w, h, PageId::DiskCleanup);
        let text = h.text();
        assert!(!text.contains("arrives with"), "{text}");
        assert!(text.contains("/Volumes/Backup"), "{w}: {text}");
        // a short terminal scrolls the page by section: focus reveals one
        h.focus(DISK.sub("tree"));
        let text = h.text();
        for needle in ["Usage · ~/work", "backend"] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
    let text = Harness::new(120, 40, PageId::DiskCleanup).page_text();
    for needle in [
        "Cleanup categories",
        "4 of 10 · 8.9 GiB",
        "Docker is running",
        "Review deletion…",
        "review, then type",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn disk_cleanup_checklist_counts_and_skips_the_category_in_use() {
    let mut h = Harness::new(140, 48, PageId::DiskCleanup);
    h.focus(DISK.sub("categories"));
    // the cursor starts on the first (checked) row: Space unchecks it
    h.key(KeyCode::Char(' '));
    assert!(h.text().contains("3 of 10 · 8.1 GiB"), "{}", h.text());
    // the last row is in use: the cursor never parks on it, so it stays
    // unchecked however far the keys go
    for _ in 0..12 {
        h.key(KeyCode::Down);
    }
    h.key(KeyCode::Char(' '));
    let (_, y) = h.at("Docker build cache");
    assert!(!h.row(y).contains('✓'), "{}", h.row(y));
    assert!(h.text().contains("4 of 10"), "{}", h.text());
}

#[test]
fn disk_cleanup_review_takes_two_gates_and_reclaims() {
    let mut h = Harness::new(140, 48, PageId::DiskCleanup);
    h.focus(DISK.sub("categories"));
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("gate 1 of 2"), "{}", h.text());
    // Gate 1 opens on Cancel: Continue is one step right
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    let text = h.text();
    assert!(text.contains("gate 2 of 2"), "{text}");
    assert!(text.contains("TRASH 4"), "{text}");
    // the action stays disarmed until the phrase matches
    typed(&mut h, "TRASH");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("gate 2 of 2"), "not armed yet");
    typed(&mut h, " 4");
    h.key(KeyCode::Enter);
    assert!(!h.text().contains("gate 2 of 2"), "{}", h.text());
    assert_eq!(
        h.status(),
        "Moved 4 categories to the Trash · 8.9 GiB reclaimed"
    );
    let text = h.text();
    assert!(text.contains("0 of 6"), "{text}");
    assert!(!text.contains("Xcode DerivedData"), "{text}");
    assert!(
        text.contains("833 of 926 GiB"),
        "the root meter drops: {text}"
    );
}

#[test]
fn disk_cleanup_cancel_changes_nothing_and_dry_run_has_one_gate() {
    let mut h = Harness::new(140, 48, PageId::DiskCleanup);
    h.click_id(DISK.sub("review"));
    assert!(h.text().contains("gate 1 of 2"));
    h.key(KeyCode::Esc);
    assert_eq!(h.status(), "Cleanup cancelled · nothing changed");
    assert!(h.text().contains("4 of 10"));

    h.opt("diskcleanup.mode=dry-run");
    let text = h.text();
    assert!(text.contains("dry run · nothing is"), "{text}");
    assert!(text.contains("review only"), "{text}");
    h.click_id(DISK.sub("review"));
    assert!(h.text().contains("Dry run · 4 categories"), "{}", h.text());
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.status(), "Dry run: 4 categories · 8.9 GiB would be freed");
    assert!(h.text().contains("4 of 10"), "a dry run keeps every row");
}

#[test]
fn disk_cleanup_options_drive_the_widgets() {
    let mut h = Harness::new(140, 48, PageId::DiskCleanup);
    h.opt("diskcleanup.mode=permanent");
    assert!(h.text().contains("permanent delete"), "{}", h.text());
    assert!(h.text().contains("DELETE 4"));
    h.opt("diskcleanup.typed=off");
    assert!(h.text().contains("review, then confirm"));
    let (_, y) = h.at("▸ frontend");
    assert!(h.row(y).contains("8.8 GiB"));
    h.opt("diskcleanup.sizes=off");
    assert!(!h.row(y).contains("8.8 GiB"), "{}", h.row(y));
    h.opt("diskcleanup.meter_visual=block");
    let (_, y) = h.at("/Volumes/Backup");
    assert!(!h.row(y).contains('━'), "block meters: {}", h.row(y));
    h.opt("diskcleanup.scan=scanning");
    let text = h.text();
    assert!(text.contains("scanning ~/work"), "{text}");
    assert!(text.contains("scanning · 3 left"), "{text}");
    // a held scan cannot be reviewed: its sizes are not final
    let review = h.area(DISK.sub("review"));
    h.click(review.x + 1, review.y);
    assert!(!h.text().contains("gate 1 of 2"));
}

#[test]
fn disk_cleanup_rescan_marks_folders_busy_until_done() {
    let mut h = Harness::new(140, 48, PageId::DiskCleanup);
    h.click_id(DISK.sub("rescan"));
    assert!(h.text().contains("scanning · 5 left"), "{}", h.text());
    for _ in 0..40 {
        h.tick();
    }
    assert_eq!(h.status(), "Scan complete · 31.8 GiB in ~/work");
    assert!(h.text().contains("largest first"));
}

#[test]
fn disk_cleanup_narrow_collapses_each_pair_into_a_drawer() {
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
    assert!(h.text().contains("Details"), "wide: both panes");
    h.opt("diskcleanup.collapse_below=108");
    h.focus(DISK.sub("tree"));
    let text = h.text();
    assert!(!text.contains("Details"), "{text}");
    assert!(
        text.contains("→ ~/work · 31.8 GiB · Tab shows details"),
        "{text}"
    );
    // Tab moves into the detail: the drawer covers the tree
    h.key(KeyCode::Tab);
    assert!(h.app.shell.focus.is(DISK.sub("details")));
    let text = h.text();
    assert!(
        text.contains("Details") && text.contains("Freshness"),
        "{text}"
    );
    assert!(!text.contains("▸ frontend"), "{text}");
    // Esc returns to the tree and closes the drawer
    h.key(KeyCode::Esc);
    assert!(h.app.shell.focus.is(DISK.sub("tree")));
    assert!(h.text().contains("▸ frontend"));
}

#[test]
fn disk_cleanup_states_rows_render() {
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
    h.section(Section::States);
    let text = h.text();
    for needle in ["Usage tree", "Categories", "Filesystems", "busy", "narrow"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn disk_cleanup_tab_order_runs_tree_details_then_checklist() {
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
    // the sidebar, the section strip, the usage tree, its details, the
    // checklist
    for _ in 0..4 {
        h.key(KeyCode::Tab);
    }
    assert!(h.app.shell.focus.is(DISK.sub("categories")));
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("gate 1 of 2"), "{}", h.text());
}

// ---------------------------------------------------------------- Activity

const ACT: WidgetId = WidgetId::of("activity");

#[test]
fn activity_shows_a_live_job_not_the_stub() {
    for (w, h) in [(120, 40), (80, 24)] {
        // a short terminal scrolls the page to the side cards
        let text = Harness::new(w, h, PageId::Activity).page_text();
        assert!(!text.contains("arrives with"), "{text}");
        for needle in [
            "api dev",
            "running",
            "streams",
            "output",
            "following",
            "Stop",
        ] {
            assert!(text.contains(needle), "{w}: {needle}: {text}");
        }
    }
    let text = Harness::new(120, 40, PageId::Activity).text();
    for needle in ["Steps", "Healthy", "Command", "mise run dev", "Restart"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn activity_follows_the_tail_and_streams_hide_their_lines() {
    let mut h = Harness::new(120, 40, PageId::Activity);
    // the services keep writing while the log follows its tail
    for _ in 0..3 {
        h.tick();
    }
    assert!(h.text().contains("following"));
    assert!(h.text().contains("scheduler │"), "{}", h.text());
    // a digit hides the third stream (scheduler) while the chips have focus
    h.focus(ACT.sub("streams"));
    h.key(KeyCode::Char('3'));
    assert_eq!(h.status(), "scheduler hidden");
    let text = h.text();
    assert!(!text.contains("scheduler │"), "{text}");
    assert!(text.contains("3 of 4 streams"), "{text}");
    h.key(KeyCode::Char(' '));
    assert_eq!(h.status(), "scheduler shown");
    assert!(h.text().contains("scheduler │"));
    // scrolling up leaves the tail; End follows again
    h.focus(ACT.sub("log"));
    h.key(KeyCode::Up);
    assert!(h.text().contains("scrollback ↑"), "{}", h.text());
    h.key(KeyCode::End);
    assert!(h.text().contains("following"));
}

#[test]
fn activity_stop_is_busy_then_stopped_and_restart_runs_again() {
    let mut h = Harness::new(120, 40, PageId::Activity);
    h.click_id(ACT.sub("stop"));
    assert_eq!(h.status(), "Stopping api dev…");
    assert!(h.text().contains("stopping · SIGTERM sent"), "{}", h.text());
    for _ in 0..10 {
        h.tick();
    }
    assert_eq!(h.status(), "api dev stopped · exit 143");
    let text = h.text();
    assert!(text.contains("stopped · exit 143"), "{text}");
    assert!(text.contains("descendants reaped"), "{text}");
    h.click_id(ACT.sub("restart"));
    assert_eq!(h.status(), "Restarted api dev");
    assert!(h.text().contains("running ·"));
}

#[test]
fn activity_options_drive_the_widgets() {
    let mut h = Harness::new(120, 40, PageId::Activity);
    h.opt("activity.state=failed");
    let text = h.text();
    assert!(text.contains("failed · exit 1"), "{text}");
    assert!(text.contains("EADDRINUSE"), "{text}");
    assert!(text.contains("Free the port"), "the failure notice: {text}");
    h.opt("activity.find=error");
    assert!(h.text().contains("matches for “error”"), "{}", h.text());
    h.opt("activity.find=off");
    h.opt("activity.retention=20");
    assert!(h.text().contains("last 20 kept"), "{}", h.text());
    h.opt("activity.wrap=off");
    assert!(!h.text().contains("│  MTP 421"), "no wrapped continuation");
    // digits are the page's again with the knob off
    h.opt("activity.digit_toggles=off");
    h.focus(ACT.sub("streams"));
    h.key(KeyCode::Char('2'));
    assert!(h.text().contains("4 of 4 streams"));
}

#[test]
fn activity_states_rows_render() {
    let mut h = Harness::new(120, 40, PageId::Activity);
    h.section(Section::States);
    let text = h.text();
    for needle in ["Log", "Streams", "Steps", "overflow", "narrow"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

// -------------------------------------------------------------- Workspaces

const WS: WidgetId = WidgetId::of("workspaces");

#[test]
fn workspaces_shows_tree_detail_and_roster_not_the_stub() {
    // the page scrolls to the card's right edge
    let text = Harness::new(120, 40, PageId::Workspaces).page_text();
    assert!(!text.contains("arrives with"), "{text}");
    for needle in [
        "payments-platform",
        "infra-control-plane",
        "7f3a the-architect",
        "Working dir",
        "/workspace/payments-platform",
        "daemon · 3 s ago",
        "a1c0  release-automat",
        "Launch",
        "Delete…",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
    // narrow (the page at its natural width is below a collapse
    // threshold set above it): the tree alone, the detail is a drawer away
    let mut h = Harness::new(80, 24, PageId::Workspaces);
    h.opt("workspaces.collapse_below=108");
    let text = h.text();
    assert!(!text.contains("arrives with"), "{text}");
    assert!(text.contains("payments-platform"), "{text}");
    assert!(!text.contains("Working dir"), "{text}");
}

#[test]
fn workspaces_narrow_detail_is_a_drawer() {
    let mut h = Harness::new(80, 24, PageId::Workspaces);
    h.opt("workspaces.collapse_below=108");
    h.focus(WS.sub("tree"));
    assert!(h.text().contains("Tab shows details"), "{}", h.text());
    h.key(KeyCode::Tab);
    assert!(h.app.shell.focus.is(WS.sub("detail")));
    let text = h.text();
    assert!(text.contains("Working dir"), "{text}");
    assert!(text.contains("Esc back"), "{text}");
    assert!(
        !text.contains("infra-control-plane"),
        "the drawer covers the tree"
    );
    h.key(KeyCode::Esc);
    assert!(h.app.shell.focus.is(WS.sub("tree")));
    assert!(h.text().contains("infra-control-plane"));
}

#[test]
fn workspaces_detail_follows_the_tree_cursor() {
    let mut h = Harness::new(120, 40, PageId::Workspaces);
    h.focus(WS.sub("tree"));
    // the workspace, its two instances, then the next workspace
    for _ in 0..3 {
        h.key(KeyCode::Down);
    }
    let text = h.text();
    assert!(text.contains("/workspace/infra"), "{text}");
    assert!(text.contains("9 vars · 4 [op]"), "{text}");
}

#[test]
fn workspaces_launch_spins_its_row_until_running() {
    let mut h = Harness::new(120, 40, PageId::Workspaces);
    h.click_id(WS.sub("launch"));
    assert_eq!(h.status(), "Launching payments-platform…");
    let (_, y) = h.at("e5d9 the-architect");
    assert!(h.row(y).contains("starting"), "{}", h.row(y));
    assert!(h.text().contains("2 running"), "starting counts as running");
    for _ in 0..20 {
        h.tick();
    }
    assert_eq!(h.status(), "Instance e5d9 is running");
    let (_, y) = h.at("e5d9 the-architect");
    assert!(h.row(y).contains("just now"), "{}", h.row(y));
}

#[test]
fn workspaces_purge_needs_the_typed_name() {
    let mut h = Harness::new(120, 40, PageId::Workspaces);
    h.click_id(WS.sub("delete"));
    assert!(
        h.text().contains("Delete payments-platform?"),
        "{}",
        h.text()
    );
    typed(&mut h, "payments");
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Delete payments-platform?"), "not armed");
    typed(&mut h, "-platform");
    h.key(KeyCode::Enter);
    assert_eq!(h.status(), "Deleted payments-platform");
    let text = h.text();
    assert!(!text.contains("▾ payments-platform"), "{text}");
    assert!(text.contains("3 saved"), "{text}");
    assert!(text.contains("/workspace/infra"), "the next one: {text}");
}

#[test]
fn workspaces_options_drive_the_master_detail() {
    let mut h = Harness::new(120, 40, PageId::Workspaces);
    h.opt("workspaces.roster=off");
    assert!(!h.text().contains("daemon · 3 s ago"));
    h.opt("workspaces.roster=auto");
    h.opt("workspaces.collapse_below=108");
    assert!(!h.text().contains("Working dir"), "collapsed at 120x40");
    h.opt("workspaces.collapse_below=80");
    h.opt("workspaces.size=share");
    assert!(h.text().contains("Working dir"));
    // without the typed confirm the purge is a plain destructive confirm
    h.opt("workspaces.typed=off");
    h.click_id(WS.sub("delete"));
    h.key(KeyCode::Right);
    h.key(KeyCode::Enter);
    assert_eq!(h.status(), "Deleted payments-platform");
}

#[test]
fn workspaces_states_rows_render() {
    let mut h = Harness::new(120, 40, PageId::Workspaces);
    h.section(Section::States);
    let text = h.text();
    for needle in [
        "Workspace tree",
        "Roster",
        "Master-detail",
        "busy",
        "narrow",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn workspaces_tab_order_runs_tree_detail_roster_then_actions() {
    let mut h = Harness::new(120, 40, PageId::Workspaces);
    let order = [
        WS.sub("sections"),
        WS.sub("tree"),
        WS.sub("detail"),
        WS.sub("roster"),
        WS.sub("launch"),
        WS.sub("edit"),
        WS.sub("prewarm"),
        WS.sub("delete"),
    ];
    for id in order {
        h.key(KeyCode::Tab);
        assert!(h.app.shell.focus.is(id), "{id:?}");
    }
    h.key(KeyCode::Enter);
    assert!(h.text().contains("Delete payments-platform?"));
}
