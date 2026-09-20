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

pub(crate) fn wheel_below(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row + 1, Scroll::Down).expect("wheel scroll");
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

/// Resize `from` → `to`, waiting until the emulator reports the new geometry
/// before settling (the reflowed frame is the gated state).
fn resize_case(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows);
    find(&mut s, case.needle);
    support::settle_and_gate(&mut s, &case.name);
}

fn resize_to(s: &mut Session, cols: u16, rows: u16) {
    resize_geometry(s, cols, rows);
    // emulator blanks/scrolls the alt-screen on resize and reports the new
    // geometry before the app redraws (~300 ms): without this pause settle
    // can gate the blank post-resize frame
    std::thread::sleep(Duration::from_millis(700));
}

fn resize_geometry(s: &mut Session, cols: u16, rows: u16) {
    s.resize(cols, rows).expect("resize");
    s.wait_until(|screen| screen.size() == (cols, rows))
        .unwrap_or_else(|e| panic!("never reached {cols}x{rows}: {e:#}"));
}

// ------------------------------------------------------------------ hover --

crate::baseline_combo_tests!(showcase_hover_buttons_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/hover/buttons/120x40/truecolor",
        SHOWCASE,
        &["--page", "buttons"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        hover_over(s, "Preview");
    });
});

crate::baseline_combo_tests!(showcase_hover_lists_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/hover/lists/120x40/truecolor",
        SHOWCASE,
        &["--page", "lists"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        hover_over(s, "Python");
    });
});

crate::baseline_combo_tests!(showcase_hover_tables_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/hover/tables/120x40/truecolor",
        SHOWCASE,
        &["--page", "tables"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, variant| {
        if variant.cols == 72 {
            hover_over(s, "#1040");
        } else {
            hover_over(s, "#1042");
        }
    });
});

// ------------------------------------------------------------ drag-select --

crate::baseline_combo_tests!(
    showcase_flows_diff_drag_selected_matrix,
    |cols, rows, color| {
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
        support::run_live_combo(&case, cols, rows, color, |s, _| {
            let (row, col) = find(s, "attempts = 3");
            s.drag(col, row, col + 11, row).expect("drag select");
        });
    }
);

// ------------------------------------------------------ S9 context menu --

// Right (secondary) click on a session row opens its context menu, titled
// with the row label (chrome.rs `PageEvent::Secondary`).
crate::baseline_combo_tests!(showcase_flows_chrome_context_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/flows/chrome/context/120x40/truecolor",
        SHOWCASE,
        &["--page", "chrome"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        let (row, col) = find(s, "Codex (Primary)");
        s.click_with(MouseButton::Right, col, row)
            .expect("right click");
    });
});

// --------------------------------------------------- S12 inspector scroll --

// The spec's "wheel over inspector" cannot work as written: the inspector
// registers no scroll region, so the wheel is `Outcome::Ignored` and the
// runtime (which repaints only on `Outcome::Changed`) never redraws — the
// frame would be byte-identical to `inspector_open`. The capturable form
// of the idea: inspector open on the lists page while a wheel scroll runs
// under it; the scroll returns Changed, and the repainted inspector shows
// the pointer's `mouse` row from the same interaction.
crate::baseline_combo_tests!(
    showcase_flows_inspector_scrolled_matrix,
    |cols, rows, color| {
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
        support::run_live_combo(&case, cols, rows, color, |s, _| {
            wheel_down(s, "Python", 2);
        });
    }
);

// ------------------------------------------------------ wheel scroll-fade --

crate::baseline_combo_tests!(
    showcase_fade_lists_wheel_fade_matrix,
    |cols, rows, color| {
        let case = Case::new(
            "showcase/fade/lists/wheel-fade/120x40/truecolor",
            SHOWCASE,
            &["--page", "lists"],
            120,
            40,
            Color::Truecolor,
            SHOWCASE_BOOT,
        );
        support::run_live_combo(&case, cols, rows, color, |s, _| {
            wheel_down(s, "Python", 2);
        });
    }
);

crate::baseline_combo_tests!(
    showcase_fade_trees_wheel_fade_matrix,
    |cols, rows, color| {
        let case = Case::new(
            "showcase/fade/trees/wheel-fade/120x40/truecolor",
            SHOWCASE,
            &["--page", "trees"],
            120,
            40,
            Color::Truecolor,
            SHOWCASE_BOOT,
        );
        support::run_live_combo(&case, cols, rows, color, |s, variant| {
            if variant.cols == 72 {
                wheel_down(s, "src", 1);
            } else {
                wheel_down(s, "config.rs", 1);
            }
        });
    }
);

crate::baseline_combo_tests!(showcase_fade_datagrid_wheel_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/fade/datagrid/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "datagrid"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        wheel_down(s, "Northwind Traders", 2);
    });
});

crate::baseline_combo_tests!(holla_fade_browser_wheel_matrix, |cols, rows, color| {
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
    support::run_live_combo_with_compact_sends(
        &case,
        cols,
        rows,
        color,
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
});

crate::baseline_combo_tests!(tablepro_fade_table_wheel_matrix, |cols, rows, color| {
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
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        wheel_down(s, "9157cff3", 3);
    });
});

// The code viewport under the wheel: 22 of 26 lines at boot, the gutter
// fades once the first lines leave the top.
crate::baseline_combo_tests!(showcase_fade_editor_wheel_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/fade/editor/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "codeeditor"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, variant| {
        if variant.rows <= 24 {
            wheel_down(s, "pub async fn fetch", 2);
        } else {
            wheel_down(s, "sleep(delay).await", 2);
        }
    });
});

// The 28-line task description scrolls inside its 8-row viewport.
crate::baseline_combo_tests!(showcase_fade_textarea_wheel_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/fade/textarea/wheel/120x40/truecolor",
        SHOWCASE,
        &["--page", "textareas"],
        120,
        40,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, variant| {
        if variant.rows <= 24 {
            wheel_down(s, "1. Read", 2);
        } else {
            wheel_down(s, "2. Keep the public API", 2);
        }
    });
});

// Review mode (the proven `diff_review` sends), then the wheel over the Old
// pane scrolls the 5-hunk diff.
crate::baseline_combo_tests!(showcase_fade_diff_wheel_matrix, |cols, rows, color| {
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
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        wheel_down(s, "attempts = 3", 2);
    });
});

// `--frame 1600` seeks to the boot stream's end state (400 + 1600 = 2000
// lines) in milliseconds — replacing the ~128 s boot stream and its 180 s
// timeout — then the wheel moves the follow-tail log off the tail.
crate::baseline_combo_tests!(
    showcase_fade_scrolling_wheel_fade_matrix,
    |cols, rows, color| {
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
        support::run_live_combo(&case, cols, rows, color, |s, _| {
            wheel_up(s, "739.63s", 2);
        });
    }
);

// Paged scroll through the wrapped prose; the frame seek keeps the boot
// instant (the page sends, not the stream, are under test).
crate::baseline_combo_tests!(showcase_fade_scroll_page_matrix, |cols, rows, color| {
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
    support::run_live_combo(&case, cols, rows, color, |_, _| ());
});

// Minimum-size reflow: the page's own nav list overflows its 12-row
// viewport, the wheel reveals the last item under the top fade.
crate::baseline_combo_tests!(showcase_fade_sidebars_wheel_matrix, |cols, rows, color| {
    let case = Case::new(
        "showcase/fade/sidebars/wheel/72x20/truecolor",
        SHOWCASE,
        &["--page", "sidebars"],
        72,
        20,
        Color::Truecolor,
        SHOWCASE_BOOT,
    );
    support::run_live_combo(&case, cols, rows, color, |s, _| {
        wheel_down(s, "Members", 2);
    });
});

// Frame-seeked mid-run terminal (60 ticks: Build container 24/40), wheel
// up twice into the scrollback — replaces the <20 s boot demo wait.
crate::baseline_combo_tests!(
    showcase_fade_terminal_scrollback_matrix,
    |cols, rows, color| {
        let case = Case::new(
            "showcase/fade/terminal/scrollback/80x24/truecolor",
            SHOWCASE,
            &["--page", "terminal", "--motion", "paused", "--frame", "60"],
            80,
            24,
            Color::Truecolor,
            SHOWCASE_BOOT,
        );
        support::run_live_combo(&case, cols, rows, color, |s, _| {
            wheel_up(s, "#4 RUN cargo build", 2);
        });
    }
);

// ----------------------------------------------------------------- resize --

crate::baseline_combo_tests!(
    showcase_resize_overview_shrunk_80x24_truecolor,
    |cols, rows, color| {
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
        support::run_resize_combo(&case, cols, rows, color, |case, cols, rows| {
            let mut s = support::spawn_boot(case);
            resize_geometry(&mut s, cols, rows);
            // Shrinking rows keeps the bottom of the old grid until the app
            // redraws; a no-op key forces an event-loop tick so the header
            // (previously above the new viewport) is painted again.
            s.send_key("ctrl-l").expect("redraw tick");
            std::thread::sleep(Duration::from_millis(700));
            find(&mut s, "Foundations / Overview");
            support::settle_and_gate(&mut s, &case.name);
        });
    }
);

crate::baseline_combo_tests!(
    showcase_resize_overview_grown_120x40_truecolor,
    |cols, rows, color| {
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
        support::run_resize_combo(&case, cols, rows, color, |case, cols, rows| {
            let mut s = support::spawn_boot(case);
            resize_geometry(&mut s, cols, rows);
            // Growing the grid reports the new size before the app paints the
            // extra cells; a no-op key forces an event-loop tick so the header
            // is drawn into the grown viewport.
            s.send_key("ctrl-l").expect("redraw tick");
            std::thread::sleep(Duration::from_millis(700));
            find(&mut s, "Foundations / Overview");
            support::settle_and_gate(&mut s, &case.name);
        });
    }
);

crate::baseline_combo_tests!(
    holla_resize_rust_dirty_shrunk_80x24_truecolor,
    |cols, rows, color| {
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
        support::run_resize_combo(&case, cols, rows, color, resize_case);
    }
);

crate::baseline_combo_tests!(
    holla_resize_rust_dirty_grown_120x40_truecolor,
    |cols, rows, color| {
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
        support::run_resize_combo(&case, cols, rows, color, resize_case);
    }
);

crate::baseline_combo_tests!(
    tablepro_resize_workbench_shrunk_80x24_truecolor,
    |cols, rows, color| {
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
        support::run_resize_combo(&case, cols, rows, color, |case, cols, rows| {
            let mut s = support::spawn_boot(case);
            resize_to(&mut s, cols, rows);
            s.wait_until(|screen| !screen.text().contains("Connected to"))
                .unwrap_or_else(|e| panic!("`Connected to` status never expired: {e:#}"));
            find(&mut s, "TablePro");
            support::settle_and_gate(&mut s, &case.name);
        });
    }
);

crate::baseline_combo_tests!(
    tablepro_resize_workbench_grown_120x40_truecolor,
    |cols, rows, color| {
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
        support::run_resize_combo(&case, cols, rows, color, |case, cols, rows| {
            let mut s = support::spawn_boot(case);
            resize_to(&mut s, cols, rows);
            if cols >= 120 {
                find(&mut s, "No results yet");
            }
            if cols >= 100 {
                find(&mut s, "S audit");
            } else {
                find(&mut s, "S public");
            }
            s.wait_until(|screen| !screen.text().contains("Connected to"))
                .unwrap_or_else(|e| panic!("`Connected to` status never expired: {e:#}"));
            support::settle_and_gate(&mut s, &case.name);
        });
    }
);
