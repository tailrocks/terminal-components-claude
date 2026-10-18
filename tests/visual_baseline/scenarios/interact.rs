use std::time::Duration;

use super::{Dismiss, Pointer};
use crate::support::{self, Case, MouseButton, Scroll, Session};

pub fn find(s: &mut Session, needle: &str) -> (u16, u16) {
    s.find(needle).unwrap_or_else(|e| panic!("{e}"))
}

pub fn mouse_move(s: &mut Session, col: u16, row: u16) {
    s.mouse_move(col, row).expect("mouse move report");
}

pub fn hover_over(s: &mut Session, needle: &str) {
    let (row, col) = find(s, needle);
    mouse_move(s, col + 2, row);
}

pub fn pointer(s: &mut Session, pointer: Pointer) {
    let (needle, button) = match pointer {
        Pointer::Hover(needle) => {
            hover_over(s, needle);
            return;
        }
        Pointer::Press(needle) => {
            let (row, col) = find(s, needle);

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

pub fn dismiss(s: &mut Session, dismiss: Dismiss, cols: u16, rows: u16) {
    match dismiss {
        Dismiss::Esc => s.send_key("escape").expect("escape"),
        Dismiss::Outside => s
            .click_with(MouseButton::Left, cols - 1, rows - 1)
            .expect("outside click"),
    }
    std::thread::sleep(Duration::from_millis(120));
}

fn notch(s: &mut Session, col: u16, row: u16, dir: Scroll) {
    s.scroll(col, row, dir).expect("wheel scroll");
    std::thread::sleep(Duration::from_millis(120));
    s.wait_answer()
        .unwrap_or_else(|e| panic!("wheel notch at ({col}, {row}): {e}"));
}

pub fn wheel_down(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        notch(s, col, row, Scroll::Down);
    }
}

pub fn wheel_below(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        notch(s, col, row + 1, Scroll::Down);
    }
}

pub fn wheel_up(s: &mut Session, needle: &str, notches: u32) {
    let (row, col) = find(s, needle);
    for _ in 0..notches {
        notch(s, col, row, Scroll::Up);
    }
}

pub fn resize_geometry(s: &mut Session, cols: u16, rows: u16) {
    s.resize(cols, rows).expect("resize");
    s.wait_until(|screen| screen.size() == (cols, rows))
        .unwrap_or_else(|e| panic!("never reached {cols}x{rows}: {e:#}"));
}

pub fn resize_to(s: &mut Session, cols: u16, rows: u16) {
    resize_geometry(s, cols, rows);
    std::thread::sleep(Duration::from_millis(700));
}

pub fn resize_case(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows);
    find(&mut s, case.needle);
    support::settle_and_gate(&mut s, &case.name);
}

pub fn showcase_overview_resize(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_geometry(&mut s, cols, rows);
    s.send_key("ctrl-l").expect("redraw tick");
    std::thread::sleep(Duration::from_millis(700));
    find(&mut s, "Design system  Overview");
    support::settle_and_gate(&mut s, &case.name);
}

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

pub fn chrome_context(s: &mut Session, _: &Case) {
    let (row, col) = find(s, "Codex (Primary)");
    s.click_with(MouseButton::Right, col, row)
        .expect("right click");
}

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

pub fn none(_: &mut Session, _: &Case) {}

pub fn table_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "9157cff3", 3);
}

pub fn tablepro_workbench_shrunk(case: &Case, cols: u16, rows: u16) {
    let mut s = support::spawn_boot(case);
    resize_to(&mut s, cols, rows);
    s.wait_until(|screen| !screen.text().contains("Connected to"))
        .unwrap_or_else(|e| panic!("`Connected to` status never expired: {e:#}"));
    find(&mut s, "TablePro");
    support::settle_and_gate(&mut s, &case.name);
}

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

pub fn accounts_badge(s: &mut Session, case: &Case) {
    if case.rows <= 30 {
        s.wait_for_text("of 23")
            .unwrap_or_else(|e| panic!("accounts badge never rendered: {e:#}"));
    }
}

pub fn capsule_brand_about(s: &mut Session, _: &Case) {
    pointer(s, Pointer::Click("jackin❯"));
    s.wait_for_text("About jackin-preview")
        .unwrap_or_else(|e| panic!("{e}"));
    s.send_key("enter").expect("enter");
}

pub fn browser_wheel(s: &mut Session, _: &Case) {
    wheel_down(s, "line 5", 2);
}

pub fn trust_body_wheel(s: &mut Session, case: &Case) {
    if case.rows > 24 {
        wheel_down(s, "id = \"deploy.preview\"", 2);
    }
}

pub fn cleanup_list_wheel(s: &mut Session, case: &Case) {
    if case.rows <= 24 {
        wheel_below(s, "Xcode DerivedData", 1);
    } else {
        wheel_below(s, "Yarn cache", 1);
    }
}
