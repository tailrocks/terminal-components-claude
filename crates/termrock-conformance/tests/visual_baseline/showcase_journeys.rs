use std::path::{Path, PathBuf};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::{self, Case, Color, SHOWCASE};

const BOOT: &str = "Junie Design system";
const COLS: u16 = 120;
const ROWS: u16 = 40;

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

fn journey_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

fn write_provenance(dir: &Path, case: &Case) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: e9428745778bbd2f8b56fb1f85bfd9a3bf0b4a06\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join("provenance.txt").display()));
    eprintln!("journey provenance: {body}");
}

fn checkpoint(s: &mut Session, dir: &Path, name: &str) {
    let obs = s
        .inner
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

#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_form_save_transient() {
    let dir = journey_dir("journey_showcase_form_save_transient");
    let case = journey_case("form_save_transient", &["--page", "forms"]);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive_with_timeout(
        &mut s,
        &["tab", "enter", "type:Fix the login redirect loop", "enter"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-filled");
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, "Creating task…", "pre-success transient");
    checkpoint(&mut s, &dir, "02-saving-transient");
    waits::wait_state(&mut s, "Task created ✓", "success");
    checkpoint(&mut s, &dir, "03-saved-success");
    eprintln!("journey form_save_transient: transient then success, both seen");
}

#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_dialog_action_and_menu() {
    let dir = journey_dir("journey_showcase_dialog_action_and_menu");

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
            text.contains("Branch feat/rate-limit deleted") && !text.contains("Delete branch?")
        },
    );
    checkpoint(&mut s, &dir, "02-dialog-action-done");

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

#[test]
#[ignore = "showcase journey; run with --ignored"]
fn journey_showcase_feedback_expiry() {
    let dir = journey_dir("journey_showcase_feedback_expiry");
    let case = journey_case("feedback_expiry", &["--page", "pickers"]);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    support::drive_with_timeout(
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
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "01-feedback-present");
    waits::wait_gone(&mut s, "Chose Safe Mode (Full)", "footer status expiry");
    checkpoint(&mut s, &dir, "02-feedback-expired");
    eprintln!("journey feedback_expiry: transient seen, then expired");
}
