//! Pointer & geometry group: the captures the `tuisnap run` CLI could not
//! express — hover states (`showcase/hover/`), diff drag-select
//! (`showcase/flows/`, including the 8-combo audit-flow variant matrix),
//! the right-click context menu and the inspector-under-scroll rows
//! (§3.7 S9/S12), wheel scrolling with scroll-fade evidence (`<app>/fade/`,
//! including the §3.3 rows, several frame-seeked with `--motion paused
//! --frame N` instead of the multi-minute boot streams), and mid-session
//! resize sequences (`<app>/resize/`) — closing honest gaps 1–2 of
//! `docs/baseline/tuisnap-coverage.md` through the library's
//! click/drag/scroll/resize API. Every state is deterministic (paused motion
//! or content/settle waits on seeded fixtures); each frame goes through the
//! same cell+pixel gate as the ported matrix. Reference evidence for the
//! target states (legacy corpus): `shots/f_buttons_hover.*`,
//! `shots/f_lists_hover.*`, `shots/f_tables_hover.*`,
//! `shots/audit-flows/diff_drag_*`, `shots/fade/*`.

use std::time::Duration;

use tuisnap::pty::{MouseButton, Scroll, Session};

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

/// `notches` wheel-up steps over `needle`'s cell — the tail-following
/// surfaces (log, terminal viewport) move off the tail and reveal the
/// bottom fade.
fn wheel_up(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row, Scroll::Up).expect("wheel scroll");
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
    // emulator blanks/scrolls the alt-screen on resize and reports the new
    // geometry before the app redraws (~300 ms): without this pause settle
    // can gate the blank post-resize frame
    std::thread::sleep(Duration::from_millis(700));
    find(&mut s, case.needle);
    support::settle_and_gate(&mut s, &case.name);
}

// ------------------------------------------------------------------ hover --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_hover_buttons_120x40_truecolor() {
    let case = Case::new(
        "showcase/hover/buttons_120x40_truecolor",
        SHOWCASE,
        &["--page", "buttons"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    hover_over(&mut s, "Preview");
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_hover_lists_120x40_truecolor() {
    let case = Case::new(
        "showcase/hover/lists_120x40_truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    hover_over(&mut s, "Python");
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_hover_tables_120x40_truecolor() {
    let case = Case::new(
        "showcase/hover/tables_120x40_truecolor",
        SHOWCASE,
        &["--page", "tables"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    hover_over(&mut s, "#1042");
    support::settle_and_gate(&mut s, &case.name);
}

// ------------------------------------------------------------ drag-select --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_diff_drag_selected_120x40_truecolor() {
    let case = Case::new(
        "showcase/flows/diff_drag-selected_120x40_truecolor",
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
    support::settle_and_gate(&mut s, &case.name);
}

/// The remaining 8 size×colour combos of the drag audit-flow matrix
/// ({80x24,160x50} × {truecolor,none,nocolor} + 120x40 × {none,nocolor}) —
/// same sends and drag as the proven 120x40/truecolor capture; the target
/// cell is located dynamically, so any size works.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_diff_drag_selected_variants() {
    let mut failures = Vec::new();
    for (cols, rows, color) in support::FLOW_VARIANTS {
        let case = Case::dynamic(
            support::showcase_flow_name(support::FLOW_LEAF_DIFF_DRAG_SELECTED, cols, rows, color),
            SHOWCASE,
            &["--page", "diff"],
            cols,
            rows,
            color,
            SHOWCASE_BOOT,
        )
        .sends(&["tab", "enter", "wait:● Review"]);
        let name = case.name.to_string();
        if !support::collect_matrix(&name, || {
            let mut s = spawn_boot(&case);
            let (row, col) = find(&mut s, "attempts = 3");
            s.drag(col, row, col + 11, row).expect("drag select");
            support::settle_and_gate(&mut s, &case.name);
        }) {
            failures.push(name);
        }
    }
    support::finish_matrix(&failures);
}

// ------------------------------------------------------ S9 context menu --

/// Right (secondary) click on a session row opens its context menu, titled
/// with the row label (chrome.rs `PageEvent::Secondary`).
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_chrome_context_120x40_truecolor() {
    let case = Case::new(
        "showcase/flows/chrome_context_120x40_truecolor",
        SHOWCASE,
        &["--page", "chrome"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    let (row, col) = find(&mut s, "Codex (Primary)");
    s.click_with(MouseButton::Right, col, row)
        .expect("right click");
    support::settle_and_gate(&mut s, &case.name);
}

// --------------------------------------------------- S12 inspector scroll --

/// The spec's "wheel over inspector" cannot work as written: the inspector
/// registers no scroll region, so the wheel is `Outcome::Ignored` and the
/// runtime (which repaints only on `Outcome::Changed`) never redraws — the
/// frame would be byte-identical to `inspector_open`. The capturable form
/// of the idea: inspector open on the lists page while a wheel scroll runs
/// under it; the scroll returns Changed, and the repainted inspector shows
/// the pointer's `mouse` row from the same interaction.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_inspector_scrolled_120x40_truecolor() {
    let case = Case::new(
        "showcase/flows/inspector_scrolled_120x40_truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["i"]);
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Python", 2);
    support::settle_and_gate(&mut s, &case.name);
}

// ------------------------------------------------------ wheel scroll-fade --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_lists_wheel_fade_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/lists_wheel-fade_120x40_truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Python", 2);
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_trees_wheel_fade_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/trees_wheel-fade_120x40_truecolor",
        SHOWCASE,
        &["--page", "trees"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "config.rs", 1);
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_datagrid_wheel_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/datagrid_wheel_120x40_truecolor",
        SHOWCASE,
        &["--page", "datagrid"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Northwind Traders", 2);
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_fade_browser_wheel_120x40_truecolor() {
    // The big.log preview pane (2000 lines, scrollbared) under the wheel;
    // reduced motion, like the proven browser journeys.
    let case = Case::new(
        "holla/fade/browser_wheel_120x40_truecolor",
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
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_fade_table_wheel_120x40_truecolor() {
    let case = Case::new(
        "tablepro/fade/table_wheel_120x40_truecolor",
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
    support::settle_and_gate(&mut s, &case.name);
}

// The legacy fade corpus was 100x30; the remapped lists case above runs at
// 120x40. This is the exact-size parity counterpart (§3.3).
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_lists_wheel_fade_100x30_truecolor() {
    let case = Case::new(
        "showcase/fade/lists_wheel-fade_100x30_truecolor",
        SHOWCASE,
        &["--page", "lists"],
        100,
        30,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Python", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// The code viewport under the wheel: 22 of 26 lines at boot, the gutter
/// fades once the first lines leave the top.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_editor_wheel_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/editor_wheel_120x40_truecolor",
        SHOWCASE,
        &["--page", "codeeditor"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "sleep(delay).await", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// The 28-line task description scrolls inside its 8-row viewport.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_textarea_wheel_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/textarea_wheel_120x40_truecolor",
        SHOWCASE,
        &["--page", "textareas"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "2. Keep the public API", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// Review mode (the proven diff_review sends), then the wheel over the Old
/// pane scrolls the 5-hunk diff.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_diff_wheel_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/diff_wheel_120x40_truecolor",
        SHOWCASE,
        &["--page", "diff"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["tab", "enter", "wait:● Review"]);
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "attempts = 3", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// `--frame 1600` seeks to the boot stream's end state (400 + 1600 = 2000
/// lines) in milliseconds — replacing the ~128 s boot stream and its 180 s
/// timeout — then the wheel moves the follow-tail log off the tail.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_scrolling_wheel_fade_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/scrolling_wheel-fade_120x40_truecolor",
        SHOWCASE,
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_up(&mut s, "739.63s", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// Mono twin (the legacy `s_fade_scrolling_mono` counterpart).
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_scrolling_wheel_fade_120x40_none() {
    let case = Case::new(
        "showcase/fade/scrolling_wheel-fade_120x40_none",
        SHOWCASE,
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        120,
        40,
        Color::None,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_up(&mut s, "739.63s", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// Paged scroll through the wrapped prose; the frame seek keeps the boot
/// instant (the page sends, not the stream, are under test).
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_scroll_page_120x40_truecolor() {
    let case = Case::new(
        "showcase/fade/scroll_page_120x40_truecolor",
        SHOWCASE,
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["tab", "pagedown"]);
    let mut s = spawn_boot(&case);
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_scroll_page_80x24_truecolor() {
    let case = Case::new(
        "showcase/fade/scroll_page_80x24_truecolor",
        SHOWCASE,
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        80,
        24,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["tab", "pagedown"]);
    let mut s = spawn_boot(&case);
    support::settle_and_gate(&mut s, &case.name);
}

/// Minimum-size reflow: the page's own nav list overflows its 12-row
/// viewport, the wheel reveals the last item under the top fade.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_sidebars_wheel_72x20_truecolor() {
    let case = Case::new(
        "showcase/fade/sidebars_wheel_72x20_truecolor",
        SHOWCASE,
        &["--page", "sidebars"],
        72,
        20,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_down(&mut s, "Members", 2);
    support::settle_and_gate(&mut s, &case.name);
}

/// Frame-seeked mid-run terminal (60 ticks: Build container 24/40), wheel
/// up twice into the scrollback — replaces the <20 s boot demo wait.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_terminal_scrollback_80x24_truecolor() {
    let case = Case::new(
        "showcase/fade/terminal_scrollback_80x24_truecolor",
        SHOWCASE,
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    let mut s = spawn_boot(&case);
    wheel_up(&mut s, "#4 RUN cargo build", 2);
    support::settle_and_gate(&mut s, &case.name);
}

// ----------------------------------------------------------------- resize --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_resize_overview_shrunk_80x24_truecolor() {
    let case = Case::new(
        "showcase/resize/overview_shrunk_80x24_truecolor",
        SHOWCASE,
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .timeout(15_000);
    let mut s = spawn_boot(&case);
    s.resize(80, 24).expect("resize");
    s.wait_until(|screen| screen.size() == (80, 24))
        .unwrap_or_else(|e| panic!("never reached 80x24: {e:#}"));
    // Shrinking rows keeps the bottom of the old grid until the app
    // redraws; a no-op key forces an event-loop tick so the header
    // (previously above the new viewport) is painted again.
    s.send_key("ctrl-l").expect("redraw tick");
    std::thread::sleep(Duration::from_millis(700));
    find(&mut s, "Foundations / Overview");
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_resize_overview_grown_120x40_truecolor() {
    let case = Case::new(
        "showcase/resize/overview_grown_120x40_truecolor",
        SHOWCASE,
        &["--page", "overview"],
        80,
        24,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .timeout(15_000);
    let mut s = spawn_boot(&case);
    s.resize(120, 40).expect("resize");
    s.wait_until(|screen| screen.size() == (120, 40))
        .unwrap_or_else(|e| panic!("never reached 120x40: {e:#}"));
    // Growing the grid reports the new size before the app paints the
    // extra cells; a no-op key forces an event-loop tick so the header
    // is drawn into the grown viewport.
    s.send_key("ctrl-l").expect("redraw tick");
    std::thread::sleep(Duration::from_millis(700));
    find(&mut s, "Foundations / Overview");
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_resize_rust_dirty_shrunk_80x24_truecolor() {
    let case = Case::new(
        "holla/resize/rust-dirty_shrunk_80x24_truecolor",
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
    )
    .timeout(30000);
    resize_case(&case, 80, 24);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_resize_rust_dirty_grown_120x40_truecolor() {
    let case = Case::new(
        "holla/resize/rust-dirty_grown_120x40_truecolor",
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
    )
    .timeout(30000);
    resize_case(&case, 120, 40);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_resize_workbench_shrunk_80x24_truecolor() {
    // 120x40 → 80x24 exercises the <100-col explorer drawer reflow live.
    // Determinism (the case flaked under load 2026-09-13): `wait:S audit`
    // pins the asynchronous schema load before the resize; the 700 ms pause
    // and the status wait run after the reflow repaint has landed (the
    // emulator blanks/scrolls the alt-screen on resize and reports the new
    // geometry before the app redraws — a wait evaluated on the blank frame
    // passes vacuously); the gated frame is the status-free steady state
    // after the transient 5 s `Connected to …` status — the only
    // deterministic side of that coin (the previous approval had the
    // status baked in and was load-sensitive in both directions).
    let case = Case::new(
        "tablepro/resize/workbench_shrunk_80x24_truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        120,
        40,
        Color::Truecolor,
        "Query 1",
    )
    .sends(&["wait:S audit"])
    .timeout(15000);
    let mut s = spawn_boot(&case);
    s.resize(80, 24).expect("resize");
    s.wait_until(|screen| screen.size() == (80, 24))
        .unwrap_or_else(|e| panic!("never reached 80x24: {e:#}"));
    // the emulator blanks/scrolls the alt-screen on resize and reports the
    // new geometry before the app redraws (~300 ms): without this pause the
    // status wait can pass vacuously on the blank frame and the capture
    // lands in the still-live status window
    std::thread::sleep(Duration::from_millis(700));
    s.wait_until(|screen| !screen.text().contains("Connected to"))
        .unwrap_or_else(|e| panic!("`Connected to` status never expired: {e:#}"));
    find(&mut s, "TablePro");
    support::settle_and_gate(&mut s, &case.name);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_resize_workbench_grown_120x40_truecolor() {
    // 80x24 → 120x40, same hardening as the shrunk twin: the schema-load
    // marker is below the fold at 80x24, so the waits run after the grow —
    // `No results yet` renders only in the docked wide layout, `S audit`
    // only once the tree is loaded, and the last wait pins the
    // transient 5 s `Connected to …` status to its status-free steady
    // state (it was baked into the previous approval, which made the case
    // load-sensitive in both directions).
    let case = Case::new(
        "tablepro/resize/workbench_grown_120x40_truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        80,
        24,
        Color::Truecolor,
        "Query 1",
    )
    .timeout(15000);
    let mut s = spawn_boot(&case);
    s.resize(120, 40).expect("resize");
    s.wait_until(|screen| screen.size() == (120, 40))
        .unwrap_or_else(|e| panic!("never reached 120x40: {e:#}"));
    // the emulator blanks/scrolls the alt-screen on resize and reports the
    // new geometry before the app redraws (~300 ms): without this pause the
    // status wait can pass vacuously on the blank frame and the capture
    // lands in the still-live status window
    std::thread::sleep(Duration::from_millis(700));
    find(&mut s, "No results yet");
    find(&mut s, "S audit");
    s.wait_until(|screen| !screen.text().contains("Connected to"))
        .unwrap_or_else(|e| panic!("`Connected to` status never expired: {e:#}"));
    support::settle_and_gate(&mut s, &case.name);
}
