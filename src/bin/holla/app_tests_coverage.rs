//! Deterministic coverage of the App's surface contracts.
//!
//! These tests intentionally stay on ratatui's TestBackend.  They cover
//! model state, focus and hit registration; PTY snapshots and live spinner
//! phases belong to the visual-baseline suite and are not part of this file.

use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Position;

use junie_tui::core::event::{MouseKind, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::widgets::scrollbar;

use crate::app_tests::H;
use crate::app_tests_parity::open;
use crate::domain::plan::PlanPhase;
use crate::domain::usage::UsageStore;
use crate::scenario::{Motion, Scenario};
use crate::screens::{GateTarget, Go, Page};
use crate::screens::{cleanup, disk, files, finder, plan, review, snapshot};

type SurfaceFn = fn(&mut H);

struct SurfaceCase {
    name: &'static str,
    build: fn() -> H,
    open: SurfaceFn,
    marker: &'static str,
    focus: WidgetId,
}

fn assert_surface(h: &H, name: &str, marker: &str, focus: WidgetId) {
    let text = h.text();
    assert!(
        text.contains(marker),
        "{name}: missing marker {marker:?}\n{text}"
    );
    assert_eq!(
        h.app.focus.current(),
        Some(focus),
        "{name}: focus owner\n{text}"
    );
    assert!(
        h.app.hits.area_of(focus).is_some(),
        "{name}: focus owner has no hit area\n{text}"
    );
}

fn push(h: &mut H, page: Page) {
    h.app.go(Go::Push(page));
    h.draw();
}

fn build_first() -> H {
    H::new(Scenario::FirstUse, Motion::Paused, 40, 120, 40)
}

fn build_rust() -> H {
    H::new(Scenario::RustDirty, Motion::Paused, 40, 120, 40)
}

fn build_docker() -> H {
    H::new(Scenario::DockerCleanup, Motion::Reduced, 0, 120, 40)
}

fn build_custom() -> H {
    H::new(Scenario::ParityCustomActions, Motion::Paused, 40, 120, 40)
}

fn build_browser() -> H {
    H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 120, 40)
}

fn build_files() -> H {
    H::new(Scenario::ParityFiles, Motion::Reduced, 40, 120, 40)
}

fn build_disk() -> H {
    H::new(Scenario::ParityDiskNavigation, Motion::Reduced, 40, 120, 40)
}

fn build_insights() -> H {
    H::new(Scenario::ParityInsights, Motion::Paused, 40, 120, 40)
}

fn build_delete() -> H {
    H::new(Scenario::ParityDeleteSafety, Motion::Reduced, 0, 120, 40)
}

fn build_report() -> H {
    H::new(Scenario::ParityCleanupResults, Motion::Reduced, 0, 120, 40)
}

fn page_finder(h: &mut H) {
    push(h, Page::Finder { group: None });
}

fn page_disk(h: &mut H) {
    let path = h.app.world.location.cwd.clone();
    push(h, Page::Disk { path });
    h.ticks(60);
}

fn page_plan(h: &mut H) {
    let plan = h.app.world.plans.first().expect("plan fixture").id.clone();
    push(h, Page::PlanReview { plan });
}

fn page_review(h: &mut H) {
    let plan = h.app.world.plans.first().expect("plan fixture").id.clone();
    push(h, Page::Review(GateTarget::Plan(plan)));
}

fn page_trust(h: &mut H) {
    push(
        h,
        Page::Trust {
            config: "/Users/alex/work/team/.holla.toml".into(),
            then: "Deploy preview".into(),
        },
    );
}

fn page_args(h: &mut H) {
    let item = h
        .app
        .world
        .items()
        .into_iter()
        .find(|item| !item.args.is_empty())
        .expect("an argument-bearing fixture item")
        .id;
    push(h, Page::Args { item });
}

fn page_snapshot(h: &mut H) {
    push(
        h,
        Page::Snapshot {
            kind: "port".into(),
        },
    );
}

fn page_files(h: &mut H) {
    push(
        h,
        Page::Files {
            path: "/Users/alex/work/site".into(),
            query: None,
        },
    );
    h.ticks(8);
}

fn page_find(h: &mut H) {
    push(h, Page::Find);
    h.ticks(8);
}

fn page_disk_overview(h: &mut H) {
    push(h, Page::DiskOverview);
}

fn page_top_files(h: &mut H) {
    push(h, Page::TopFiles);
}

fn page_cleanup(h: &mut H) {
    push(h, Page::Cleanup { category: None });
}

fn page_cleanup_gate(h: &mut H) {
    open(h, "Analyze disk usage");
    h.ticks(60);
    h.key(KeyCode::Down);
    h.key(KeyCode::Char(' '));
    h.key(KeyCode::Char('d'));
}

fn page_report(h: &mut H) {
    push(h, Page::Report { index: 0 });
}

fn page_config(h: &mut H) {
    push(
        h,
        Page::Config {
            path: "/Users/alex/work/team/.holla.toml".into(),
        },
    );
}

fn page_cases() -> Vec<SurfaceCase> {
    vec![
        SurfaceCase {
            name: "Finder",
            build: build_first,
            open: page_finder,
            marker: "Here",
            focus: finder::FINDER,
        },
        SurfaceCase {
            name: "Disk",
            build: build_disk,
            open: page_disk,
            marker: "Disk › Usage",
            focus: disk::TREE,
        },
        SurfaceCase {
            name: "PlanReview",
            build: build_docker,
            open: page_plan,
            marker: "Clean Docker completely",
            focus: plan::OUTLINE,
        },
        SurfaceCase {
            name: "Review",
            build: build_docker,
            open: page_review,
            marker: "gate 1 of 2",
            focus: review::GATE_CANCEL,
        },
        SurfaceCase {
            name: "Trust",
            build: build_custom,
            open: page_trust,
            marker: "Trust",
            focus: review::TRUST_CANCEL,
        },
        SurfaceCase {
            name: "Args",
            build: build_rust,
            open: page_args,
            marker: "Arguments",
            focus: review::ARG.child(0),
        },
        SurfaceCase {
            name: "Snapshot",
            build: build_rust,
            open: page_snapshot,
            marker: "snapshot · read-only",
            focus: snapshot::BODY,
        },
        SurfaceCase {
            name: "Files",
            build: build_browser,
            open: page_files,
            marker: "Files ›",
            focus: files::LIST,
        },
        SurfaceCase {
            name: "Find",
            build: build_files,
            open: page_find,
            marker: "Files ›",
            focus: files::LIST,
        },
        SurfaceCase {
            name: "DiskOverview",
            build: build_disk,
            open: page_disk_overview,
            marker: "Disk › Overview",
            focus: disk::LIST,
        },
        SurfaceCase {
            name: "TopFiles",
            build: build_disk,
            open: page_top_files,
            marker: "Top files",
            focus: disk::LIST,
        },
        SurfaceCase {
            name: "Cleanup",
            build: build_insights,
            open: page_cleanup,
            marker: "Cleanup",
            focus: cleanup::LIST,
        },
        SurfaceCase {
            name: "CleanupGate",
            build: build_delete,
            open: page_cleanup_gate,
            marker: "gate 1 of 2",
            focus: review::GATE_CANCEL,
        },
        SurfaceCase {
            name: "Report",
            build: build_report,
            open: page_report,
            marker: "Cleanup report",
            focus: snapshot::BODY,
        },
        SurfaceCase {
            name: "Config",
            build: build_custom,
            open: page_config,
            marker: "snapshot · read-only",
            focus: snapshot::BODY,
        },
    ]
}

#[test]
fn page_inventory_has_marker_focus_hit_and_escape_contract() {
    for case in page_cases() {
        let mut h = (case.build)();
        (case.open)(&mut h);
        let depth = h.app.tabs[0].stack.len();
        assert_surface(&h, case.name, case.marker, case.focus);
        assert!(h.app.modals.is_empty(), "{}: unexpected modal", case.name);

        h.key(KeyCode::Esc);
        if case.name == "Finder" {
            assert_eq!(
                h.app.tabs[0].stack.len(),
                depth,
                "{}: Esc changed the finder stack",
                case.name
            );
        } else {
            assert_eq!(
                h.app.tabs[0].stack.len(),
                depth - 1,
                "{}: Esc did not pop the page",
                case.name
            );
        }
        assert!(h.app.modals.is_empty(), "{}: Esc left a modal", case.name);
        assert!(
            h.app.focus.current().is_some(),
            "{}: Esc lost focus",
            case.name
        );
    }
}

fn assert_modal(h: &H, name: &str, marker: &str, expected_focus: Option<WidgetId>) {
    let text = h.text();
    assert_eq!(h.app.modals.len(), 1, "{name}: modal count\n{text}");
    assert!(
        text.contains(marker),
        "{name}: missing marker {marker:?}\n{text}"
    );
    if let Some(expected) = expected_focus {
        assert!(
            h.app.ring.contains(expected),
            "{name}: modal owner is absent from focus ring\n{text}"
        );
    }
    if let Some(focus) = h.app.focus.current() {
        assert!(
            h.app.hits.area_of(focus).is_some() || h.app.ring.contains(focus),
            "{name}: current modal focus is not registered\n{text}"
        );
    }
    assert!(
        !h.app.hits.is_empty(),
        "{name}: modal rendered no hits\n{text}"
    );
}

fn dismiss_modal(h: &mut H, name: &str, owner: Option<WidgetId>) {
    h.key(KeyCode::Esc);
    assert!(h.app.modals.is_empty(), "{name}: Esc did not dismiss");
    assert_eq!(h.app.focus.current(), owner, "{name}: focus not restored");
}

#[test]
fn modal_inventory_has_marker_focus_hits_and_escape_contract() {
    let mut help = build_first();
    let help_owner = help.app.focus.current();
    help.key(KeyCode::F(1));
    assert_modal(
        &help,
        "Text/help",
        "Key reference",
        Some(WidgetId::of("help")),
    );
    dismiss_modal(&mut help, "Text/help", help_owner);

    let mut about = build_rust();
    let about_owner = about.app.focus.current();
    about.key(KeyCode::F(10));
    about.key(KeyCode::Right);
    about.key(KeyCode::Right);
    about.key(KeyCode::End);
    about.key(KeyCode::Enter);
    assert_modal(
        &about,
        "Text/about",
        "About holla❯",
        Some(WidgetId::of("about")),
    );
    dismiss_modal(&mut about, "Text/about", about_owner);

    let mut why = build_rust();
    let why_owner = why.app.focus.current();
    why.alt(KeyCode::Enter);
    why.key(KeyCode::End);
    why.key(KeyCode::Enter);
    assert_modal(&why, "Text/why", "Why is", Some(WidgetId::of("why")));
    dismiss_modal(&mut why, "Text/why", why_owner);

    let mut activities = H::new(Scenario::ActivitiesMulti, Motion::Paused, 40, 120, 40);
    let activities_owner = activities.app.focus.current();
    activities.ctrl(KeyCode::Char('g'));
    assert_modal(&activities, "Picker/activities", "Activities", None);
    dismiss_modal(&mut activities, "Picker/activities", activities_owner);

    let mut jump = build_browser();
    open(&mut jump, "Browse ~/work/site");
    jump.ticks(8);
    let jump_owner = jump.app.focus.current();
    jump.key(KeyCode::Char('g'));
    assert_modal(&jump, "Picker/files-jump", "Go to path", None);
    dismiss_modal(&mut jump, "Picker/files-jump", jump_owner);

    let mut alternatives = build_rust();
    let alternatives_owner = alternatives.app.focus.current();
    alternatives.alt(KeyCode::Enter);
    assert_modal(
        &alternatives,
        "Menu/alternatives",
        "Set alias",
        Some(WidgetId::of("alternatives")),
    );
    dismiss_modal(&mut alternatives, "Menu/alternatives", alternatives_owner);

    let mut dialog = build_docker();
    dialog.key(KeyCode::Enter);
    dialog.key(KeyCode::Char('c'));
    dialog.key(KeyCode::Right);
    let dialog_owner = dialog.app.focus.current();
    dialog.key(KeyCode::Enter);
    assert_modal(&dialog, "Dialog/gate2", "gate 2 of 2", None);
    dismiss_modal(&mut dialog, "Dialog/gate2", dialog_owner);
}

fn outside(h: &H) -> Position {
    for y in 0..h.app.size.1 {
        for x in 0..h.app.size.0 {
            let p = Position::new(x, y);
            if h.app.hits.hit(p).is_none() {
                return p;
            }
        }
    }
    panic!(
        "no outside hit position in {}x{}",
        h.app.size.0, h.app.size.1
    );
}

fn drag_scrollbar(h: &mut H, container: WidgetId, name: &str) {
    let id = scrollbar::id_for(container);
    let track = h
        .app
        .hits
        .area_of(id)
        .unwrap_or_else(|| panic!("{name}: missing scrollbar hit\n{}", h.text()));
    let before = h.text();
    let start = track.y + track.height.min(3).saturating_sub(1);
    let bottom = track.bottom().saturating_sub(1);
    h.mouse(MouseKind::Down, track.x, start);
    assert_eq!(
        h.mouse(MouseKind::Drag, track.x, bottom),
        Outcome::Changed,
        "{name}: scrollbar drag did not change the model"
    );
    h.mouse(MouseKind::Up, track.x, bottom);
    assert_ne!(h.text(), before, "{name}: scrollbar drag did not redraw");
    assert_eq!(
        h.app.focus.current(),
        Some(container),
        "{name}: scrollbar did not restore container focus"
    );
}

#[test]
fn raw_scrollbars_route_pointer_press_and_drag() {
    let mut finder = H::new(Scenario::ParityDiscovery, Motion::Reduced, 0, 72, 20);
    finder.ticks(60);
    drag_scrollbar(&mut finder, finder::FINDER, "finder list");

    let mut files = H::new(Scenario::ParityBrowser, Motion::Reduced, 40, 72, 20);
    open(&mut files, "Browse ~/work/site");
    files.ticks(8);
    drag_scrollbar(&mut files, files::LIST, "files list");

    let mut cleanup = H::new(Scenario::ParityInsights, Motion::Paused, 40, 72, 20);
    page_cleanup(&mut cleanup);
    drag_scrollbar(&mut cleanup, cleanup::LIST, "cleanup list");

    let mut modal = H::new(Scenario::RustDirty, Motion::Paused, 40, 72, 20);
    modal.key(KeyCode::F(1));
    drag_scrollbar(&mut modal, WidgetId::of("help"), "help modal");
}

#[test]
fn finder_file_and_modal_mouse_contracts() {
    let mut finder = build_rust();
    let (x, y) = finder.find("Run tests").expect("finder row");
    let pos = Position::new(x + 2, y);
    let row = finder.app.hits.hit(pos).expect("finder row hit");
    finder.mouse(MouseKind::Move, pos.x, pos.y);
    assert_eq!(finder.app.hover, Some(row));
    finder.click(pos.x, pos.y);
    assert_eq!(finder.app.focus.current(), Some(finder::FINDER));
    assert!(finder.row(y).contains("›"), "{}", finder.row(y));

    finder.mouse(MouseKind::Secondary, pos.x, pos.y);
    assert!(!finder.app.modals.is_empty());
    assert_eq!(
        finder.app.focus.current(),
        Some(WidgetId::of("alternatives"))
    );
    assert!(
        finder
            .app
            .hits
            .area_of(WidgetId::of("alternatives"))
            .is_some()
    );
    finder.key(KeyCode::Esc);
    assert!(finder.app.modals.is_empty());

    // Two releases at the same virtual time exercise App's double-click path.
    let mut double = build_rust();
    let (dx, dy) = double.find("Run tests").expect("double-click row");
    let tabs = double.app.tabs.len();
    let stack = double.app.tabs[0].stack.len();
    double.click(dx + 2, dy);
    double.click(dx + 2, dy);
    assert!(
        double.app.tabs.len() > tabs
            || double.app.tabs[0].stack.len() > stack
            || !double.app.modals.is_empty(),
        "double-click did not activate a finder row\n{}",
        double.text()
    );

    let mut files_page = build_browser();
    open(&mut files_page, "Browse ~/work/site");
    files_page.ticks(8);
    let (fx, fy) = files_page.find("big.log").expect("browser file row");
    let file_pos = Position::new(fx + 2, fy);
    let file_hit = files_page.app.hits.hit(file_pos).expect("file row hit");
    files_page.mouse(MouseKind::Move, file_pos.x, file_pos.y);
    assert_eq!(files_page.app.hover, Some(file_hit));
    files_page.click(file_pos.x, file_pos.y);
    assert_eq!(files_page.app.focus.current(), Some(files::LIST));
    files_page.mouse(MouseKind::Secondary, file_pos.x, file_pos.y);
    assert!(files_page.text().contains("Open in the OS"));
    assert_eq!(
        files_page.app.focus.current(),
        Some(WidgetId::of("files.actions"))
    );
    files_page.key(KeyCode::Esc);

    let mut preview = build_browser();
    open(&mut preview, "Browse ~/work/site");
    preview.ticks(8);
    let (px, py) = preview.find("big.log").expect("preview file row");
    preview.click(px + 2, py);
    preview.key(KeyCode::Right);
    let area = preview
        .app
        .hits
        .area_of(files::PREVIEW)
        .expect("file preview hit");
    let before = preview.text();
    preview.mouse(MouseKind::WheelDown, area.x + 2, area.y + 2);
    assert_ne!(preview.text(), before, "file preview wheel did not redraw");
    preview.mouse(MouseKind::Down, area.x + 2, area.y + 2);
    preview.mouse(
        MouseKind::Drag,
        area.x + 2,
        area.y + area.height.saturating_sub(2),
    );
    preview.mouse(
        MouseKind::Up,
        area.x + 2,
        area.y + area.height.saturating_sub(2),
    );
    assert_eq!(preview.app.focus.current(), Some(files::PREVIEW));

    let mut modal = build_rust();
    modal.key(KeyCode::F(1));
    let p = outside(&modal);
    let consumed = modal.mouse(MouseKind::Secondary, p.x, p.y);
    assert_eq!(consumed, Outcome::Consumed);
    assert_eq!(modal.app.modals.len(), 1);
    modal.mouse(MouseKind::Down, p.x, p.y);
    modal.mouse(MouseKind::Up, p.x, p.y);
    assert!(modal.app.modals.is_empty(), "outside click left Text modal");

    let mut quit = H::new(Scenario::ActivitiesMulti, Motion::Paused, 40, 120, 40);
    quit.key(KeyCode::Esc);
    assert!(!quit.app.modals.is_empty());
    let p = outside(&quit);
    quit.mouse(MouseKind::Down, p.x, p.y);
    quit.mouse(MouseKind::Up, p.x, p.y);
    assert!(quit.app.modals.is_empty(), "outside click left quit dialog");
    assert!(!quit.app.quit, "outside click confirmed quit");
}

#[test]
fn resize_ladder_preserves_inventory_focus_and_hits() {
    for case in page_cases() {
        let mut h = (case.build)();
        (case.open)(&mut h);
        assert_surface(&h, case.name, case.marker, case.focus);
        for (width, height) in [
            (40, 10),
            (100, 30),
            (200, 60),
            (72, 20),
            (40, 10),
            (120, 40),
        ] {
            h.resize(width, height);
            let text = h.text();
            if width < 72 || height < 20 {
                assert!(
                    text.contains("Terminal too small"),
                    "{}: missing small-terminal state at {width}x{height}\n{text}",
                    case.name
                );
                continue;
            }
            assert_eq!(
                h.app.focus.current(),
                Some(case.focus),
                "{}: focus after {width}x{height}\n{text}",
                case.name
            );
            assert!(
                h.app.hits.area_of(case.focus).is_some(),
                "{}: hit after {width}x{height}\n{text}",
                case.name
            );
        }
    }
}

#[test]
fn history_enabled_and_disabled_states_are_deterministic() {
    let mut enabled = H::new(Scenario::ParityHistory, Motion::Reduced, 0, 120, 40);
    enabled.ticks(8);
    let enabled_text = enabled.text();
    assert!(enabled_text.contains("Recent here"), "{enabled_text}");
    assert!(enabled_text.contains("used"), "{enabled_text}");
    assert!(
        !enabled_text.contains("nothing used here yet"),
        "{enabled_text}"
    );

    let mut enabled_again = H::new(Scenario::ParityHistory, Motion::Reduced, 0, 120, 40);
    enabled_again.ticks(8);
    assert_eq!(enabled_text, enabled_again.text());

    let mut disabled = H::new(Scenario::ParityHistory, Motion::Reduced, 0, 120, 40);
    disabled.app.world.memory.usage = UsageStore::disabled();
    disabled.type_str("x");
    disabled.ctrl(KeyCode::Char('u'));
    disabled.draw();
    let disabled_text = disabled.text();
    assert!(disabled_text.contains("Recent here"), "{disabled_text}");
    assert!(
        disabled_text.contains("nothing used here yet"),
        "{disabled_text}"
    );
    assert_ne!(enabled_text, disabled_text);
}

#[test]
fn plan_review_gate_cancel_and_cleanup_report_routes_keep_model_state() {
    let mut plan = build_docker();
    plan.key(KeyCode::Enter);
    assert_surface(
        &plan,
        "plan review",
        "Clean Docker completely",
        plan::OUTLINE,
    );
    plan.key(KeyCode::Char('c'));
    assert_surface(&plan, "plan gate 1", "gate 1 of 2", review::GATE_CANCEL);
    assert_eq!(plan.app.world.plans[0].phase, PlanPhase::Review);
    plan.key(KeyCode::Enter);
    assert_eq!(plan.app.world.plans[0].phase, PlanPhase::Review);
    assert!(plan.app.modals.is_empty());

    // Re-open the plan, advance to gate 2, then cancel without executing.
    plan.key(KeyCode::Enter);
    plan.key(KeyCode::Char('c'));
    plan.key(KeyCode::Right);
    plan.key(KeyCode::Enter);
    assert!(plan.text().contains("gate 2 of 2"), "{}", plan.text());
    plan.key(KeyCode::Esc);
    assert!(plan.app.modals.is_empty());
    assert_eq!(plan.app.world.plans[0].phase, PlanPhase::Review);

    let mut cleanup = build_report();
    open(&mut cleanup, "Analyze disk usage");
    cleanup.ticks(30);
    let (_, row) = cleanup.find("big").expect("cleanup candidate");
    let tree = cleanup
        .app
        .hits
        .area_of(disk::TREE)
        .expect("cleanup tree hit");
    cleanup.click(tree.x + 6, row);
    cleanup.key(KeyCode::Char('d'));
    assert_surface(
        &cleanup,
        "cleanup gate 1",
        "gate 1 of 2",
        review::GATE_CANCEL,
    );
    cleanup.key(KeyCode::Right);
    cleanup.key(KeyCode::Enter);
    assert!(cleanup.text().contains("gate 2 of 2"), "{}", cleanup.text());
    cleanup.key(KeyCode::Enter);
    cleanup.type_str("TRASH 1 UNDER /Users/alex/Projects/safe ON mbp");
    cleanup.key(KeyCode::Enter);
    cleanup.key(KeyCode::Tab);
    cleanup.key(KeyCode::Enter);
    assert!(cleanup.app.world.cleanup_job.is_some());
    cleanup.ticks(1);
    assert!(cleanup.app.world.cleanup_job.is_none());
    assert!(!cleanup.app.world.reports.is_empty());
    assert_surface(&cleanup, "cleanup report", "Cleanup report", snapshot::BODY);
}
