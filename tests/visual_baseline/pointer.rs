//! Pointer & geometry group: the captures the `tuisnap run` CLI could not
//! express — hover states, diff drag-select, wheel scrolling with scroll-fade
//! evidence, and mid-session resize sequences — closing honest gaps 1–2 of
//! `docs/baseline/tuisnap-coverage.md` through the library's
//! click/drag/scroll/resize API. Every state is deterministic (paused motion
//! or content/settle waits on seeded fixtures); each frame goes through the
//! same cell+pixel gate as the ported matrix. Reference evidence for the
//! target states: `shots/f_buttons_hover.*`, `shots/f_lists_hover.*`,
//! `shots/f_tables_hover.*`, `shots/audit-flows/diff_drag_*`, `shots/fade/*`.

use std::time::Duration;

use tuisnap::pty::{Scroll, Session};

use crate::support::{self, Case, Color, HOLLA, SHOWCASE, TABLEPRO};

const SHOWCASE_BOOT: &str = "Junie Design system";
const HOLLA_BOOT: &str = "holla❯";

/// First occurrence of `needle` as `(row, col)` — `Screen::find` order —
/// waiting until it appears. The hand-off to the pointer calls swaps the
/// axes (they take `(col, row)`).
fn find(s: &mut Session, needle: &str) -> (u16, u16) {
    let mut hit = None;
    s.wait_until(|screen| {
        hit = screen.find(needle);
        hit.is_some()
    })
    .unwrap_or_else(|e| panic!("`{needle}` never appeared: {e:#}"));
    hit.expect("wait_until passed with the needle on screen")
}

/// Bare-pointer move. termlens models clicks, drags and the wheel but has no
/// button-less motion report, so the bytes a real terminal sends for one are
/// written verbatim: every app enables SGR any-motion tracking
/// (`?1003h` + `?1006h` via crossterm's `EnableMouseCapture`), where a move
/// to `(col, row)` is `CSI < 35 ; col+1 ; row+1 M`.
fn mouse_move(s: &mut Session, col: u16, row: u16) {
    s.type_text(&format!("\x1b[<35;{};{}M", col + 1, row + 1))
        .expect("mouse move report");
}

fn hover_over(s: &mut Session, needle: &str) {
    let (row, col) = find(s, needle);
    mouse_move(s, col + 2, row);
}

/// `notches` wheel-down steps over `needle`'s cell, paced like a send step.
fn wheel_down(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row, Scroll::Down).expect("wheel scroll");
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn spawn_boot(case: &Case) -> Session {
    let mut s = support::spawn(case);
    support::boot(&mut s, case.needle);
    if !case.sends.is_empty() {
        support::drive(&mut s, case.sends);
    }
    s
}

/// Resize `from` → `to`, waiting until the emulator reports the new geometry
/// before settling (the reflowed frame is the gated state).
fn resize_case(case: &Case, cols: u16, rows: u16) {
    let mut s = spawn_boot(case);
    s.resize(cols, rows).expect("resize");
    s.wait_until(|screen| screen.size() == (cols, rows))
        .unwrap_or_else(|e| panic!("never reached {cols}x{rows}: {e:#}"));
    support::settle_and_gate(&mut s, case.name);
}

// ------------------------------------------------------------------ hover --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_buttons_hover_120x40_truecolor() {
    let case = Case::new(
        "showcase_buttons_hover_120x40_truecolor",
        SHOWCASE,
        &["--page", "buttons"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    hover_over(&mut s, "Preview");
    support::settle_and_gate(&mut s, case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_lists_hover_120x40_truecolor() {
    let case = Case::new(
        "showcase_lists_hover_120x40_truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    hover_over(&mut s, "Python");
    support::settle_and_gate(&mut s, case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_tables_hover_120x40_truecolor() {
    let case = Case::new(
        "showcase_tables_hover_120x40_truecolor",
        SHOWCASE,
        &["--page", "tables"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    hover_over(&mut s, "#1042");
    support::settle_and_gate(&mut s, case.name);
}

// ------------------------------------------------------------ drag-select --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_diff_drag_selected_120x40_truecolor() {
    let case = Case::new(
        "showcase_diff_drag-selected_120x40_truecolor",
        SHOWCASE,
        &["--page", "diff"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["tab", "enter", "wait:● Review"]);
    let mut s = spawn_boot(&case);
    let (row, col) = find(&mut s, "attempts = 3");
    s.drag(col, row, col + 11, row).expect("drag select");
    support::settle_and_gate(&mut s, case.name);
}

// ------------------------------------------------------ wheel scroll-fade --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_lists_wheel_fade_120x40_truecolor() {
    let case = Case::new(
        "showcase_lists_wheel-fade_120x40_truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Python", 2);
    support::settle_and_gate(&mut s, case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_trees_wheel_fade_120x40_truecolor() {
    let case = Case::new(
        "showcase_trees_wheel-fade_120x40_truecolor",
        SHOWCASE,
        &["--page", "trees"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "config.rs", 1);
    support::settle_and_gate(&mut s, case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_datagrid_wheel_120x40_truecolor() {
    let case = Case::new(
        "showcase_datagrid_wheel_120x40_truecolor",
        SHOWCASE,
        &["--page", "datagrid"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Northwind Traders", 2);
    support::settle_and_gate(&mut s, case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_browser_wheel_120x40_truecolor() {
    // The big.log preview pane (2000 lines, scrollbared) under the wheel;
    // reduced motion, like the proven browser journeys.
    let case = Case::new(
        "holla_browser_wheel_120x40_truecolor",
        HOLLA,
        &["--scenario", "parity-browser", "--motion", "reduced"],
        120,
        40,
        Color::Truecolor,
        HOLLA_BOOT,
    )
    .sends(&[
        "type:Browse ~/work/site",
        "enter",
        "wait:16 entries",
        "down",
        "down",
        "down",
        "wait:first 2000 lines",
    ]);
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "line 5", 2);
    support::settle_and_gate(&mut s, case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_table_wheel_120x40_truecolor() {
    let case = Case::new(
        "tablepro_table_wheel_120x40_truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        120,
        40,
        Color::Truecolor,
        "Query 1",
    )
    .sends(&[
        "down",
        "down",
        "down",
        "down",
        "down",
        "enter",
        "wait:public › orders",
    ]);
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "9157cff3", 3);
    support::settle_and_gate(&mut s, case.name);
}

// ----------------------------------------------------------------- resize --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_overview_shrunk_80x24_truecolor() {
    let case = Case::new(
        "showcase_overview_shrunk_80x24_truecolor",
        SHOWCASE,
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    resize_case(&case, 80, 24);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_overview_grown_120x40_truecolor() {
    let case = Case::new(
        "showcase_overview_grown_120x40_truecolor",
        SHOWCASE,
        &["--page", "overview"],
        80,
        24,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    resize_case(&case, 120, 40);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_rust_dirty_shrunk_80x24_truecolor() {
    let case = Case::new(
        "holla_rust-dirty_shrunk_80x24_truecolor",
        HOLLA,
        &[
            "--scenario",
            "rust-dirty",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        120,
        40,
        Color::Truecolor,
        HOLLA_BOOT,
    );
    resize_case(&case, 80, 24);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_rust_dirty_grown_120x40_truecolor() {
    let case = Case::new(
        "holla_rust-dirty_grown_120x40_truecolor",
        HOLLA,
        &[
            "--scenario",
            "rust-dirty",
            "--motion",
            "paused",
            "--frame",
            "40",
        ],
        80,
        24,
        Color::Truecolor,
        HOLLA_BOOT,
    );
    resize_case(&case, 120, 40);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_workbench_shrunk_80x24_truecolor() {
    // 120x40 → 80x24 exercises the <100-col explorer drawer reflow live.
    let case = Case::new(
        "tablepro_workbench_shrunk_80x24_truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        120,
        40,
        Color::Truecolor,
        "Query 1",
    );
    resize_case(&case, 80, 24);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_workbench_grown_120x40_truecolor() {
    let case = Case::new(
        "tablepro_workbench_grown_120x40_truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        80,
        24,
        Color::Truecolor,
        "Query 1",
    );
    resize_case(&case, 120, 40);
}
