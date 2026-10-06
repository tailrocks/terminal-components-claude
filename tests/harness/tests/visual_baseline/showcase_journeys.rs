//! Showcase app-journey suite, slice 1 (VB Phase-7a, §15).
//!
//! Multi-step PTY journeys over the real `showcase` binary built from this
//! worktree's verified VB sources (`env!("CARGO_BIN_EXE_showcase")`, the
//! harness `[[bin]]` compiled from `../../src/bin/showcase/main.rs`).
//! Journeys assert needle state across steps — they never gate snapshots,
//! so every case here uses owned (dynamic) names: the approval-inventory
//! parser in [`crate::support::suite_capture_names`] only scans static
//! representative declarations, and journey names must stay out of the
//! committed `snapshots/` inventory.
//!
//! Per-test scratch (checkpoints + provenance) lands under this harness
//! crate's `target/tuiscotti/journeys/<test>/` (gitignored, unique per
//! test fn). Checkpoints are text frames (`.txt`); `provenance.txt`
//! records the binary path, size, mtime, and argv behind each journey.
//! Typed input is synthetic and in-memory only (simulation data).

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::Session;

use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, SHOWCASE};

const BOOT: &str = "Junie Design system";
const COLS: u16 = 120;
const ROWS: u16 = 40;

/// Every Showcase page in `]`-cycle order (`NAV_ENTRIES`): slug for the
/// checkpoint filename, header crumb (`/ <section> / <label>`) as proof.
const PAGES: [(&str, &str); 23] = [
    ("overview", "/ Foundations / Overview"),
    ("buttons", "/ Components / Buttons"),
    ("inputs", "/ Components / Inputs"),
    ("textareas", "/ Components / Text areas"),
    ("forms", "/ Components / Forms"),
    ("lists", "/ Components / Lists"),
    ("trees", "/ Components / Trees"),
    ("tables", "/ Components / Tables"),
    ("editabletables", "/ Components / Editable tables"),
    ("panels", "/ Components / Panels"),
    ("sidebars", "/ Components / Sidebars"),
    ("dialogs", "/ Components / Dialogs"),
    ("progress", "/ Components / Progress"),
    ("scrolling", "/ Components / Scrolling"),
    ("terminal", "/ Components / Terminal"),
    ("codeeditor", "/ Components / Code editor"),
    ("diff", "/ Components / Diff"),
    ("datagrid", "/ Components / Data grid"),
    ("chipsselects", "/ Components / Chips & selects"),
    ("pickers", "/ Components / Pickers"),
    ("chrome", "/ Components / Chrome"),
    ("settings", "/ Screens / Settings"),
    ("taskrunner", "/ Screens / Task runner"),
];

/// Owned-name journey case at the canonical 120x40 truecolor geometry.
fn journey_case(name: &str, args: &'static [&'static str]) -> Case {
    Case::dynamic(
        format!("journeys/showcase/{name}"),
        SHOWCASE,
        args,
        COLS,
        ROWS,
        Color::Truecolor,
        BOOT,
    )
}

/// Unique scratch dir for one journey test (created, never shared).
fn journey_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary path + identity behind a journey: absolute path,
/// byte size, mtime, and argv. Written to `provenance.txt`, echoed too.
fn write_provenance(dir: &Path, case: &Case) {
    let meta = std::fs::metadata(case.bin)
        .unwrap_or_else(|e| panic!("stat {}: {e}", case.bin));
    let modified = meta
        .modified()
        .map(|t| format!("{t:?}"))
        .unwrap_or_else(|_| "unknown".to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nbinary: {}\nsize_bytes: {}\nmodified: {modified}\nargv: {}\n",
        case.bin,
        meta.len(),
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("journey provenance: {body}");
}

/// Save the live screen text as `<name>.txt` in the journey dir.
fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
        .observe_now()
        .unwrap_or_else(|e| panic!("checkpoint `{name}` sample failed: {e:#}"));
    let text = support::screen_text(&obs.screen);
    let body = format!(
        "# checkpoint {name} {}x{}\n{text}\n",
        obs.screen.cols(),
        obs.screen.rows()
    );
    let path = dir.join(format!("{name}.txt"));
    std::fs::write(&path, body).unwrap_or_else(|e| panic!("write {}: {e}", path.display()));
    eprintln!("checkpoint {name} ({} bytes visible text)", text.len());
}

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

/// Slice 1, journey 1: visit every Showcase page in one session (`]`
/// cycles `NAV_ENTRIES` order), proving each with its header crumb.
#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_all_pages() {
    let dir = journey_dir("journey_showcase_all_pages");
    let case = journey_case("all_pages", &["--page", "overview"]);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    waits::wait_state(&mut s, PAGES[0].1, "boot page crumb");
    checkpoint(&mut s, &dir, "00-overview");
    for (i, (slug, crumb)) in PAGES.iter().enumerate().skip(1) {
        support::press_step(&s, "]");
        waits::wait_state(&mut s, crumb, &format!("page {slug}"));
        checkpoint(&mut s, &dir, &format!("{i:02}-{slug}"));
    }
    eprintln!("journey all_pages: visited {} pages", PAGES.len());
}

/// Slice 1, journey 2: fill the task form and submit, asserting the
/// pre-success transient (`Creating task…`, ~1.8 s busy window) before
/// the success state (`Task created ✓`). Synthetic input, in-memory.
#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_form_save_transient() {
    let dir = journey_dir("journey_showcase_form_save_transient");
    let case = journey_case("form_save_transient", &["--page", "forms"]);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive(
        &mut s,
        &["tab", "enter", "type:Fix the login redirect loop", "enter"],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-filled");
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, "Creating task…", "pre-success transient");
    checkpoint(&mut s, &dir, "02-saving-transient");
    waits::wait_state(&mut s, "Task created ✓", "success");
    checkpoint(&mut s, &dir, "03-saved-success");
    eprintln!("journey form_save_transient: transient then success, both seen");
}

/// Slice 1, journey 3: one dialog action (confirm the destructive
/// `Delete branch?` dialog via its action button) plus one menu level
/// traversal (chrome File menu open, then across to the View menu).
#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_dialog_action_and_menu() {
    let dir = journey_dir("journey_showcase_dialog_action_and_menu");

    // Dialog action: `d` opens with Cancel focused; `right` moves to the
    // Delete action, `enter` runs it; the result proves the dialog closed.
    let dialogs = journey_case("dialog_action", &["--page", "dialogs"]);
    write_provenance(&dir, &dialogs);
    let mut s = support::spawn_boot(&dialogs);
    support::press_step(&s, "d");
    waits::wait_state(&mut s, "Delete branch?", "destructive dialog open");
    checkpoint(&mut s, &dir, "01-dialog-open");
    support::press_step(&s, "right");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        support::DEFAULT_WAIT,
        "dialog action ran and dialog closed",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Branch feat/rate-limit deleted")
                && !text.contains("Delete branch?")
        },
    );
    checkpoint(&mut s, &dir, "02-dialog-action-done");

    // Menu level traversal: focus the menu bar, open File, cross one
    // level to View; File's items must be gone, View's shown.
    let chrome = journey_case("menu_traversal", &["--page", "chrome"]);
    let mut s = support::spawn_boot(&chrome);
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "← → Menu", "menu bar focused");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "New tab", "file menu open");
    checkpoint(&mut s, &dir, "03-menu-file");
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        support::DEFAULT_WAIT,
        "menu level traversed to view",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Zoom pane") && !text.contains("New tab")
        },
    );
    checkpoint(&mut s, &dir, "04-menu-view");
    eprintln!("journey dialog_action_and_menu: delete ran, file→view crossed");
}

/// Slice 1, journey 4: choose a picker level, assert the transient
/// footer feedback (`Chose Safe Mode (Full)`) before its 4 s expiry,
/// then prove the expiry (absence after presence — never vacuous).
#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_feedback_expiry() {
    let dir = journey_dir("journey_showcase_feedback_expiry");
    let case = journey_case("feedback_expiry", &["--page", "pickers"]);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive(
        &mut s,
        &[
            "tab",
            "tab",
            "tab",
            "enter",
            "down",
            "enter",
            "wait:Chose Safe Mode (Full)",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "01-feedback-present");
    waits::wait_gone(&mut s, "Chose Safe Mode (Full)", "footer status expiry");
    checkpoint(&mut s, &dir, "02-feedback-expired");
    eprintln!("journey feedback_expiry: transient seen, then expired");
}
