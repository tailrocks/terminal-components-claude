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
    let text = Harness::new(120, 40, PageId::DiskCleanup).text();
    for needle in [
        "Cleanup categories",
        "4 of 10 · 8.9 GiB",
        "Docker is running",
        "Review deletion…",
        "TRASH 4",
    ] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn disk_cleanup_checklist_counts_and_skips_the_category_in_use() {
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
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
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
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
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
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
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
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
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
    h.click_id(DISK.sub("rescan"));
    assert!(h.text().contains("scanning · 5 left"), "{}", h.text());
    for _ in 0..40 {
        h.tick();
    }
    assert_eq!(h.status(), "Scan complete · 31.8 GiB in ~/work");
    assert!(h.text().contains("largest first"));
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
fn disk_cleanup_tab_order_runs_tree_then_checklist() {
    let mut h = Harness::new(120, 40, PageId::DiskCleanup);
    // the sidebar, the section strip, the usage tree, the checklist
    for _ in 0..3 {
        h.key(KeyCode::Tab);
    }
    assert!(h.app.shell.focus.is(DISK.sub("categories")));
    h.key(KeyCode::Char('d'));
    assert!(h.text().contains("gate 1 of 2"), "{}", h.text());
}
