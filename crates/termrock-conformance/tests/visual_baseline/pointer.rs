//! Pointer & geometry group: the captures the `tuisnap run` CLI could not
//! express — hover states (`showcase/hover/`), diff drag-select
//! (`showcase/flows/`, including canonical drag-select expansion),
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

use tuiscotti::tui::{MouseButton, Wheel};

use crate::support::scoped_targets::Target;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, HOLLA, SHOWCASE, Session, TABLEPRO};

const SHOWCASE_BOOT: &str = "Junie Design system";
const HOLLA_BOOT: &str = "holla❯";

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

/// Typed hover over `needle`: unique-target resolve plus a mode-gated
/// button-less motion report (replaces `find` + raw SGR-1006 bytes).
fn hover_over(s: &mut Session, needle: &str) {
    let target = Target::resolve(s, needle);
    let (col, row) = target.offset(2, 0);
    Input::move_to(col, row).send(s);
}

/// `notches` wheel-down steps over `needle`'s cell, paced like a send step.
pub(crate) fn wheel_down(s: &mut Session, needle: &str, notches: u32) {
    wheel_notches(s, needle, Wheel::Down, notches, (0, 0));
}

pub(crate) fn wheel_below(s: &mut Session, needle: &str, notches: u32) {
    wheel_notches(s, needle, Wheel::Down, notches, (0, 1));
}

/// `notches` wheel-up steps over `needle`'s cell — the tail-following
/// surfaces (log, terminal viewport) move off the tail and reveal the
/// bottom fade.
fn wheel_up(s: &mut Session, needle: &str, notches: u32) {
    wheel_notches(s, needle, Wheel::Up, notches, (0, 0));
}

/// Typed wheel steps over `needle`'s cell plus `offset`: each notch is a
/// mode-gated typed send (paced like a send step), then the target is
/// re-resolved so a stale cell never feeds the next interaction.
fn wheel_notches(s: &mut Session, needle: &str, dir: Wheel, notches: u32, offset: (i16, i16)) {
    let mut target = Target::resolve(s, needle);
    let (col, row) = target.offset(offset.0, offset.1);
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
    target.refresh(s);
}

/// Resize `from` → `to`, waiting until the emulator reports the new geometry
/// before settling (the reflowed frame is the gated state).
fn resize_case(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows, case_timeout(case));
    waits::wait_state(&mut s, case.needle, "resized boot needle");
    support::settle_and_gate(&mut s, &case.name);
}

fn resize_to(s: &mut Session, cols: u16, rows: u16, timeout: Duration) {
    resize_geometry(s, cols, rows, timeout);
    // emulator blanks/scrolls the alt-screen on resize and reports the new
    // geometry before the app redraws (~300 ms): without this pause settle
    // can gate the blank post-resize frame
    std::thread::sleep(Duration::from_millis(700));
}

fn resize_geometry(s: &mut Session, cols: u16, rows: u16, timeout: Duration) {
    s.resize(cols, rows).expect("resize");
    support::wait_screen(
        s,
        timeout,
        &format!("never reached {cols}x{rows}"),
        |screen| screen.cols() == cols && screen.rows() == rows,
    );
}

fn run_resize_matrix(representative: &Case, mut capture: impl FnMut(&Case, u16, u16)) {
    let mut failures = Vec::new();
    for &(cols, rows) in &support::CANONICAL_SIZES {
        for color in support::CANONICAL_COLORS {
            let case = representative.resize_variant(cols, rows, color);
            let name = case.name.to_string();
            if !support::collect_matrix(&name, || capture(&case, cols, rows)) {
                failures.push(name);
            }
        }
    }
    support::finish_matrix(&failures);
}

// ------------------------------------------------------------------ hover --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_hover_buttons_matrix() {
    let case = Case::new(
        "showcase/hover/buttons/120x40/truecolor",
        SHOWCASE,
        &["--page", "buttons"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        hover_over(s, "Preview");
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_hover_lists_matrix() {
    let case = Case::new(
        "showcase/hover/lists/120x40/truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        hover_over(s, "Python");
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_hover_tables_matrix() {
    let case = Case::new(
        "showcase/hover/tables/120x40/truecolor",
        SHOWCASE,
        &["--page", "tables"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, variant| {
        if variant.cols == 72 {
            hover_over(s, "#1040");
        } else {
            hover_over(s, "#1042");
        }
    });
}

// ------------------------------------------------------------ drag-select --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_diff_drag_selected_matrix() {
    let case = Case::new(
        "showcase/flows/diff/drag-selected/120x40/truecolor",
        SHOWCASE,
        &["--page", "diff"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["tab", "enter", "wait:● Review"]);
    support::run_canonical_live(&case, |s, _| {
        let target = Target::resolve(s, "attempts = 3");
        Input::drag(target.point(), target.span_end()).send(s);
    });
}

// ------------------------------------------------------ S9 context menu --

/// Right (secondary) click on a session row opens its context menu, titled
/// with the row label (chrome.rs `PageEvent::Secondary`).
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_flows_chrome_context_matrix() {
    let case = Case::new(
        "showcase/flows/chrome/context/120x40/truecolor",
        SHOWCASE,
        &["--page", "chrome"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        let target = Target::resolve(s, "Codex (Primary)");
        let (col, row) = target.point();
        Input::down(MouseButton::Right, col, row).send(s);
        Input::up(MouseButton::Right, col, row).send(s);
    });
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
fn showcase_flows_inspector_scrolled_matrix() {
    let case = Case::new(
        "showcase/flows/inspector/scrolled/120x40/truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["i"]);
    support::run_canonical_live(&case, |s, _| {
        wheel_down(s, "Python", 2);
    });
}

// ------------------------------------------------------ wheel scroll-fade --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_lists_wheel_fade_matrix() {
    let case = Case::new(
        "showcase/fade/lists/wheel-fade/120x40/truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        wheel_down(s, "Python", 2);
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_trees_wheel_fade_matrix() {
    let case = Case::new(
        "showcase/fade/trees/wheel-fade/120x40/truecolor",
        SHOWCASE,
        &["--page", "trees"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, variant| {
        if variant.cols == 72 {
            wheel_down(s, "src", 1);
        } else {
            wheel_down(s, "config.rs", 1);
        }
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_datagrid_wheel_matrix() {
    let case = Case::new(
        "showcase/fade/datagrid/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "datagrid"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        wheel_down(s, "Northwind Traders", 2);
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_fade_browser_wheel_matrix() {
    let case = Case::new(
        "holla/fade/browser_wheel/120x40/truecolor",
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
    support::run_canonical_live_with_compact_sends(
        &case,
        100,
        &[
            "type:Browse ~/work/site",
            "enter",
            "wait:16 entries",
            // Home removes width-dependent cursor drift before selecting
            // big.log. Right opens the compact preview drawer.
            "home",
            "down",
            "down",
            "down",
            "right",
            "wait:first 2000 lines",
        ],
        |s, _| {
            wheel_down(s, "line 5", 2);
        },
    );
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_fade_table_wheel_matrix() {
    let case = Case::new(
        "tablepro/fade/table_wheel/120x40/truecolor",
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
    support::run_canonical_live(&case, |s, _| {
        wheel_down(s, "9157cff3", 3);
    });
}

/// The code viewport under the wheel: 22 of 26 lines at boot, the gutter
/// fades once the first lines leave the top.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_editor_wheel_matrix() {
    let case = Case::new(
        "showcase/fade/editor/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "codeeditor"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, variant| {
        if variant.rows <= 24 {
            wheel_down(s, "pub async fn fetch", 2);
        } else {
            wheel_down(s, "sleep(delay).await", 2);
        }
    });
}

/// The 28-line task description scrolls inside its 8-row viewport.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_textarea_wheel_matrix() {
    let case = Case::new(
        "showcase/fade/textarea/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "textareas"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, variant| {
        if variant.rows <= 24 {
            wheel_down(s, "1. Read", 2);
        } else {
            wheel_down(s, "2. Keep the public API", 2);
        }
    });
}

/// Review mode (the proven diff_review sends), then the wheel over the Old
/// pane scrolls the 5-hunk diff.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_diff_wheel_matrix() {
    let case = Case::new(
        "showcase/fade/diff/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "diff"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .sends(&["tab", "enter", "wait:● Review"]);
    support::run_canonical_live(&case, |s, _| {
        wheel_down(s, "attempts = 3", 2);
    });
}

/// `--frame 1600` seeks to the boot stream's end state (400 + 1600 = 2000
/// lines) in milliseconds — replacing the ~128 s boot stream and its 180 s
/// timeout — then the wheel moves the follow-tail log off the tail.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_scrolling_wheel_fade_matrix() {
    let case = Case::new(
        "showcase/fade/scrolling/wheel-fade/120x40/truecolor",
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
    support::run_canonical_live(&case, |s, _| {
        wheel_up(s, "739.63s", 2);
    });
}

/// Paged scroll through the wrapped prose; the frame seek keeps the boot
/// instant (the page sends, not the stream, are under test).
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_scroll_page_matrix() {
    let case = Case::new(
        "showcase/fade/scroll/page/120x40/truecolor",
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
    support::run_canonical_live(&case, |_, _| ());
}

/// Minimum-size reflow: the page's own nav list overflows its 12-row
/// viewport, the wheel reveals the last item under the top fade.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_sidebars_wheel_matrix() {
    let case = Case::new(
        "showcase/fade/sidebars/wheel/72x20/truecolor",
        SHOWCASE,
        &["--page", "sidebars"],
        72,
        20,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        wheel_down(s, "Members", 2);
    });
}

/// Frame-seeked mid-run terminal (60 ticks: Build container 24/40), wheel
/// up twice into the scrollback — replaces the <20 s boot demo wait.
#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_fade_terminal_scrollback_matrix() {
    let case = Case::new(
        "showcase/fade/terminal/scrollback/80x24/truecolor",
        SHOWCASE,
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_canonical_live(&case, |s, _| {
        wheel_up(s, "#4 RUN cargo build", 2);
    });
}

// ----------------------------------------------------------------- resize --

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_resize_overview_shrunk_80x24_truecolor() {
    let case = Case::new(
        "showcase/resize/overview_shrunk/80x24/truecolor",
        SHOWCASE,
        &["--page", "overview"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .timeout(15_000);
    run_resize_matrix(&case, |case, cols, rows| {
        let mut s = support::spawn_boot(case);
        resize_geometry(&mut s, cols, rows, case_timeout(case));
        // Shrinking rows keeps the bottom of the old grid until the app
        // redraws; a no-op key forces an event-loop tick so the header
        // (previously above the new viewport) is painted again.
        Input::key("ctrl-l").send(&mut s);
        std::thread::sleep(Duration::from_millis(700));
        waits::wait_state(&mut s, "Foundations / Overview", "resized overview header");
        support::settle_and_gate(&mut s, &case.name);
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn showcase_resize_overview_grown_120x40_truecolor() {
    let case = Case::new(
        "showcase/resize/overview_grown/120x40/truecolor",
        SHOWCASE,
        &["--page", "overview"],
        80,
        24,
        Color::Truecolor,
        SHOWCASE_BOOT,
    )
    .timeout(15_000);
    run_resize_matrix(&case, |case, cols, rows| {
        let mut s = support::spawn_boot(case);
        resize_geometry(&mut s, cols, rows, case_timeout(case));
        // Growing the grid reports the new size before the app paints the
        // extra cells; a no-op key forces an event-loop tick so the header
        // is drawn into the grown viewport.
        Input::key("ctrl-l").send(&mut s);
        std::thread::sleep(Duration::from_millis(700));
        waits::wait_state(&mut s, "Foundations / Overview", "resized overview header");
        support::settle_and_gate(&mut s, &case.name);
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_resize_rust_dirty_shrunk_80x24_truecolor() {
    let case = Case::new(
        "holla/resize/rust-dirty_shrunk/80x24/truecolor",
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
    run_resize_matrix(&case, resize_case);
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn holla_resize_rust_dirty_grown_120x40_truecolor() {
    let case = Case::new(
        "holla/resize/rust-dirty_grown/120x40/truecolor",
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
    run_resize_matrix(&case, resize_case);
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
        "tablepro/resize/workbench_shrunk/80x24/truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        120,
        40,
        Color::Truecolor,
        "Query 1",
    )
    .sends(&["wait:S audit"])
    .timeout(15000);
    run_resize_matrix(&case, |case, cols, rows| {
        let timeout = case_timeout(case);
        let mut s = support::spawn_boot(case);
        resize_to(&mut s, cols, rows, timeout);
        support::wait_screen(
            &mut s,
            timeout,
            "`Connected to` status never expired",
            |screen| !support::screen_text(screen).contains("Connected to"),
        );
        waits::wait_state(&mut s, "TablePro", "resized workbench");
        support::settle_and_gate(&mut s, &case.name);
    });
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
        "tablepro/resize/workbench_grown/120x40/truecolor",
        TABLEPRO,
        &["--connect", "Production"],
        80,
        24,
        Color::Truecolor,
        "Query 1",
    )
    .timeout(15000);
    run_resize_matrix(&case, |case, cols, rows| {
        let timeout = case_timeout(case);
        let mut s = support::spawn_boot(case);
        resize_to(&mut s, cols, rows, timeout);
        if cols >= 120 {
            waits::wait_state(&mut s, "No results yet", "grown docked layout");
        }
        if cols >= 100 {
            waits::wait_state(&mut s, "S audit", "grown schema tree");
        } else {
            waits::wait_state(&mut s, "S public", "grown schema tree");
        }
        support::wait_screen(
            &mut s,
            timeout,
            "`Connected to` status never expired",
            |screen| !support::screen_text(screen).contains("Connected to"),
        );
        support::settle_and_gate(&mut s, &case.name);
    });
}
