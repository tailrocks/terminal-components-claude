use std::time::Duration;

use tuiscotti::tui::Wheel;

use crate::support::scoped_targets::Target;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{MouseButton, Session};

use crate::support::{self, Case, Color, HOLLA, SHOWCASE, TABLEPRO};

const SHOWCASE_BOOT: &str = "Junie Design system";
const HOLLA_BOOT: &str = "holla❯";

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

fn hover_over(s: &mut Session, needle: &str) {
    let target = Target::resolve(s, needle);
    let (col, row) = target.offset(2, 0);
    Input::move_to(col, row).send(s);
}

pub(crate) fn wheel_down(s: &mut Session, needle: &str, notches: u32) {
    let mut target = Target::resolve(s, needle);
    for _ in 0..notches {
        let (col, row) = target.point();
        Input::wheel(Wheel::Down, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
        target.refresh(s);
    }
}

pub(crate) fn wheel_below(s: &mut Session, needle: &str, notches: u32) {
    let mut target = Target::resolve(s, needle);
    for _ in 0..notches {
        let (col, row) = target.offset(0, 1);
        Input::wheel(Wheel::Down, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
        target.refresh(s);
    }
}

fn wheel_up(s: &mut Session, needle: &str, notches: u32) {
    let mut target = Target::resolve(s, needle);
    for _ in 0..notches {
        let (col, row) = target.point();
        Input::wheel(Wheel::Up, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
        target.refresh(s);
    }
}

fn resize_case(case: &Case, cols: u16, rows: u16) {
    let timeout = case_timeout(case);
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows, timeout);
    waits::wait_state(&mut s, case.needle, "post-resize boot needle");
    support::settle_and_gate(&mut s, &case.name);
}

fn resize_to(s: &mut Session, cols: u16, rows: u16, timeout: Duration) {
    resize_geometry(s, cols, rows, timeout);

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
        wheel_up(s, "736.67s", 2);
    });
}

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
        let timeout = case_timeout(case);
        let mut s = support::spawn_boot(case);
        resize_geometry(&mut s, cols, rows, timeout);

        s.send_key("ctrl-l").expect("redraw tick");
        std::thread::sleep(Duration::from_millis(700));
        waits::wait_state(
            &mut s,
            "Foundations / Overview",
            "overview header after shrink",
        );
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
        let timeout = case_timeout(case);
        let mut s = support::spawn_boot(case);
        resize_geometry(&mut s, cols, rows, timeout);

        s.send_key("ctrl-l").expect("redraw tick");
        std::thread::sleep(Duration::from_millis(700));
        waits::wait_state(
            &mut s,
            "Foundations / Overview",
            "overview header after grow",
        );
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
        waits::wait_state(&mut s, "TablePro", "tablepro brand after shrink");
        support::settle_and_gate(&mut s, &case.name);
    });
}

#[test]
#[ignore = "visual baseline capture; run with --ignored"]
fn tablepro_resize_workbench_grown_120x40_truecolor() {
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
            waits::wait_state(&mut s, "No results yet", "docked results marker after grow");
        }
        if cols >= 100 {
            waits::wait_state(&mut s, "S audit", "audit marker after grow");
        } else {
            waits::wait_state(&mut s, "S public", "public marker after grow");
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
