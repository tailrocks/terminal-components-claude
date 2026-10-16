//! Live interactions for `Drive::Live` and `Drive::Resize` rows: the steps
//! the key DSL cannot express (hover, wheel, drag, right click, press
//! without release, resize). Coordinates are always resolved from on-screen
//! text at capture time, never hardcoded, so a layout change shows up as a
//! frame diff rather than a mis-click.
//!
//! The functions ported from the retired `pointer.rs`, `holla.rs` and
//! `audit.rs` keep their exact behaviour (needles, notches, waits).

use std::time::Duration;

use super::{Dismiss, Pointer};
use crate::support::{self, Case, MouseButton, Scroll, Session};

/// First occurrence of `needle` as `(row, col)`, waiting until it appears.
/// The pointer calls take `(col, row)`.
pub fn find(s: &mut Session, needle: &str) -> (u16, u16) {
    s.find(needle).unwrap_or_else(|e| panic!("{e}"))
}

/// Bare-pointer move: a button-less motion report. Every app enables SGR
/// any-motion tracking (`?1003h` + `?1006h`); tuiscotti refuses the move
/// otherwise.
pub fn mouse_move(s: &mut Session, col: u16, row: u16) {
    s.mouse_move(col, row).expect("mouse move report");
}

pub fn hover_over(s: &mut Session, needle: &str) {
    let (row, col) = find(s, needle);
    mouse_move(s, col + 2, row);
}

/// Apply a row's [`Pointer`] step, two cells into the target text.
pub fn pointer(s: &mut Session, pointer: Pointer) {
    let (needle, button) = match pointer {
        Pointer::Hover(needle) => {
            hover_over(s, needle);
            return;
        }
        Pointer::Press(needle) => {
            let (row, col) = find(s, needle);
            // Held: no release follows, so the pressed state stays.
            s.mouse_down(MouseButton::Left, col + 2, row)
                .expect("mouse down report");
            std::thread::sleep(Duration::from_millis(120));
            return;
        }
        Pointer::Click(needle) => (needle, MouseButton::Left),
        Pointer::RightClick(needle) => (needle, MouseButton::Right),
    };
    let (row, col) = find(s, needle);
    s.click_with(button, col + 2, row).expect("click");
    std::thread::sleep(Duration::from_millis(120));
}

/// Dismiss an open overlay: `Esc`, or a primary click on the bottom-right
/// cell (outside any centred or anchored popup).
pub fn dismiss(s: &mut Session, dismiss: Dismiss, cols: u16, rows: u16) {
    match dismiss {
        Dismiss::Esc => s.send_key("escape").expect("escape"),
        Dismiss::Outside => s
            .click_with(MouseButton::Left, cols - 1, rows - 1)
            .expect("outside click"),
    }
    std::thread::sleep(Duration::from_millis(120));
}

/// `notches` wheel-down steps over `needle`'s cell, paced like a send step.
pub fn wheel_down(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row, Scroll::Down).expect("wheel scroll");
        std::thread::sleep(Duration::from_millis(120));
    }
}

/// Wheel-down on the row below `needle` (a blank line with no child
/// region, so the list's own scroll region receives it).
pub fn wheel_below(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row + 1, Scroll::Down).expect("wheel scroll");
        std::thread::sleep(Duration::from_millis(120));
    }
}

/// `notches` wheel-up steps over `needle`'s cell: tail-following surfaces
/// move off the tail and reveal the bottom fade.
pub fn wheel_up(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        s.scroll(col, row, Scroll::Up).expect("wheel scroll");
        std::thread::sleep(Duration::from_millis(120));
    }
}

pub fn resize_geometry(s: &mut Session, cols: u16, rows: u16) {
    s.resize(cols, rows).expect("resize");
    s.wait_until(|screen| screen.size() == (cols, rows))
        .unwrap_or_else(|e| panic!("never reached {cols}x{rows}: {e:#}"));
}

/// Resize, then wait out the post-resize redraw: the emulator blanks or
/// scrolls the alt-screen and reports the new geometry before the app
/// redraws (~300 ms), so settle could otherwise gate the blank frame.
pub fn resize_to(s: &mut Session, cols: u16, rows: u16) {
    resize_geometry(s, cols, rows);
    std::thread::sleep(Duration::from_millis(700));
}

/// Generic resize capture: boot at the start geometry, resize to the
/// target, wait for the boot needle again, settle, gate.
pub fn resize_case(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows);
    find(&mut s, case.needle);
    support::settle_and_gate(&mut s, &case.name);
}

/// Showcase overview resize: shrinking keeps the bottom of the old grid and
/// growing reports the new size before the app paints the extra cells, so a
/// no-op key forces an event-loop tick that repaints the header.
pub fn showcase_overview_resize(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_geometry(&mut s, cols, rows);
    s.send_key("ctrl-l").expect("redraw tick");
    std::thread::sleep(Duration::from_millis(700));
    find(&mut s, "Design system / Overview");
    support::settle_and_gate(&mut s, &case.name);
}

// ------------------------------------------------------------- showcase --

pub fn hover_buttons(s: &mut Session, _: &Case) {
    hover_over(s, "Preview");
}

pub fn hover_lists(s: &mut Session, _: &Case) {
    hover_over(s, "Python");
}

pub fn hover_tables(s: &mut Session, case: &Case) {
    if case.cols == 72 {
        hover_over(s, "#1040");
    } else {
        hover_over(s, "#1042");
    }
}

pub fn diff_drag_select(s: &mut Session, _: &Case) {
    let (row, col) = find(s, "attempts = 3");
    s.drag(col, row, col + 11, row).expect("drag select");
}

/// Right (secondary) click on a session row opens its context menu, titled
/// with the row label (chrome.rs `PageEvent::Secondary`).
pub fn chrome_context(s: &mut Session, _: &Case) {
    let (row, col) = find(s, "Codex (Primary)");
    s.click_with(MouseButton::Right, col, row)
        .expect("right click");
}

/// Two wheel notches over the lists page's `Python` row (inspector
/// scrolled, lists wheel-fade).
pub fn wheel_python(s: &mut Session, _: &Case) {
    wheel_down(s, "Python", 2);
}

pub fn trees_wheel(s: &mut Session, case: &Case) {
    if case.cols == 72 {
        wheel_down(s, "src", 1);
    } else {
        wheel_down(s, "config.rs", 1);
    }
}

pub fn datagrid_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "Northwind Traders", 2);
}

pub fn editor_wheel(s: &mut Session, case: &Case) {
    if case.rows <= 24 {
        wheel_down(s, "pub async fn fetch", 2);
    } else {
        wheel_down(s, "sleep(delay).await", 2);
    }
}

pub fn textarea_wheel(s: &mut Session, case: &Case) {
    if case.rows <= 24 {
        wheel_down(s, "1. Read", 2);
    } else {
        wheel_down(s, "2. Keep the public API", 2);
    }
}

pub fn diff_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "attempts = 3", 2);
}

pub fn scrolling_wheel_up(s: &mut Session, _: &Case) {
    wheel_up(s, "739.63s", 2);
}

pub fn sidebars_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "Members", 2);
}

pub fn terminal_scrollback(s: &mut Session, _: &Case) {
    wheel_up(s, "#4 RUN cargo build", 2);
}

/// Nothing after boot + steps (a live row with the steps as the whole
/// interaction).
pub fn none(_: &mut Session, _: &Case) {}

// ------------------------------------------------------------- tablepro --

pub fn table_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "9157cff3", 3);
}

/// 120x40 → target exercises the <100-col explorer drawer reflow live.
/// `wait:S audit` (row steps) pins the asynchronous schema load before the
/// resize; the status wait runs after the reflow repaint, so the gated frame
/// is the steady state after the transient 5 s `Connected to …` status.
pub fn tablepro_workbench_shrunk(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows);
    s.wait_until(|screen| !screen.text().contains("Connected to"))
        .unwrap_or_else(|e| panic!("`Connected to` status never expired: {e:#}"));
    find(&mut s, "TablePro");
    support::settle_and_gate(&mut s, &case.name);
}

/// 80x24 → target: the schema-load marker is below the fold at 80x24, so
/// the waits run after the grow (`No results yet` only in the docked wide
/// layout, `S audit` once the tree is loaded), then the status-free state.
pub fn tablepro_workbench_grown(case: &Case, cols: u16, rows: u16) {
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
}

// --------------------------------------------------------------- jackin --

/// accounts-mixed never finishes its boot refresh under paused motion, so a
/// repaint lands every 500 ms. The Accounts scroll badge (`· 1–11 of 23`)
/// first appears in the t≈500 ms repaint, while settle can fire at t≈400.
/// Where the list overflows (rows <= 30) wait for the badge itself.
pub fn accounts_badge(s: &mut Session, case: &Case) {
    if case.rows <= 30 {
        s.wait_for_text("of 23")
            .unwrap_or_else(|e| panic!("accounts badge never rendered: {e:#}"));
    }
}

// ---------------------------------------------------------------- holla --

pub fn browser_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "line 5", 2);
}

/// The trust definition is the scrollable body (review.rs TRUST_BODY); two
/// notches put the fade at its top edge.
pub fn trust_body_wheel(s: &mut Session, case: &Case) {
    if case.rows <= 24 {
        wheel_down(s, "Trust scope", 2);
    } else {
        wheel_down(s, "id = \"deploy.preview\"", 2);
    }
}

/// Rows are clickable children and shadow the list's scroll region, so
/// the wheel lands on the blank line below the needle.
pub fn cleanup_list_wheel(s: &mut Session, case: &Case) {
    if case.rows <= 24 {
        wheel_below(s, "Xcode DerivedData", 2);
    } else {
        wheel_below(s, "Yarn cache", 2);
    }
}
