use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::Wheel;
use tuiscotti::{Color as TColor, Frame, Rgb};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, TABLEPRO};

const BOOT_CONN: &str = "Production";
const BOOT_WB: &str = "Query 1";
const SPIN: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

fn t2_case_conn(slug: &str) -> Case {
    Case::dynamic(
        format!("journeys/tablepro/t2/{slug}"),
        TABLEPRO,
        &[],
        120,
        40,
        Color::Truecolor,
        BOOT_CONN,
    )
}

fn t2_case_wb(slug: &str) -> Case {
    Case::dynamic(
        format!("journeys/tablepro/t2/{slug}"),
        TABLEPRO,
        &["--connect", "Production"],
        120,
        40,
        Color::Truecolor,
        BOOT_WB,
    )
}

fn t2_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: 0ae8cbe8cc208a5cda4d30f70da04c6f435e591f\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("t2 provenance ({file}): {body}");
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

fn case_timeout(case: &Case) -> Duration {
    Duration::from_millis(case.timeout_ms)
}

fn drive_ms(timeout: Duration) -> u64 {
    u64::try_from(timeout.as_millis()).expect("case timeout fits u64 ms")
}

fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .inner
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    frame_from_screen(&screen, "default")
}

fn find_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let obs = support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_find(screen, needle).is_some(),
    );
    support::screen_find(&obs.screen, needle).expect("wait passed with the needle on screen")
}

fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

fn copied_count(text: &str, what: &str) -> u32 {
    let line = text
        .lines()
        .find(|l| l.contains("Copied ") && l.contains(" chars"))
        .unwrap_or_else(|| panic!("{what}: no Copied status on screen"));
    let after = line.split("Copied ").nth(1).expect("count");
    after
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or_else(|_| panic!("{what}: unparsable Copied count"))
}

fn fg_rgb(frame: &Frame, col: u16, row: u16, what: &str) -> (u8, u8, u8) {
    let cell = frame
        .get(col, row)
        .unwrap_or_else(|| panic!("{what}: no cell at ({col},{row})"));
    match cell.fg {
        TColor::Rgb(Rgb { r, g, b }) => (r, g, b),
        other => panic!("{what}: cell ({col},{row}) fg is not RGB: {other:?}"),
    }
}

fn bg_rgb(frame: &Frame, col: u16, row: u16, what: &str) -> (u8, u8, u8) {
    let cell = frame
        .get(col, row)
        .unwrap_or_else(|| panic!("{what}: no cell at ({col},{row})"));
    match cell.bg {
        TColor::Rgb(Rgb { r, g, b }) => (r, g, b),
        other => panic!("{what}: cell ({col},{row}) bg is not RGB: {other:?}"),
    }
}

fn lum(rgb: (u8, u8, u8)) -> f32 {
    0.2126 * f32::from(rgb.0) / 255.0
        + 0.7152 * f32::from(rgb.1) / 255.0
        + 0.0722 * f32::from(rgb.2) / 255.0
}

fn quiesce_text(s: &mut Session, what: &str) -> String {
    let mut prev = live_text(s);
    for _ in 0..8 {
        std::thread::sleep(Duration::from_millis(150));
        let cur = live_text(s);
        if cur == prev {
            return cur;
        }
        prev = cur;
    }
    panic!("{what}: screen never quiesced");
}

fn escape_to_idle(s: &mut Session, what: &str) {
    for _ in 0..6 {
        let idle = live_text(s)
            .lines()
            .last()
            .is_none_or(|l| !l.contains("EDIT"));
        if idle {
            return;
        }
        support::press_step(s, "escape");
        std::thread::sleep(Duration::from_millis(250));
    }
    panic!("{what}: still editing after 6 Esc");
}

fn down_to(s: &mut Session, needle: &str, max: usize, what: &str) {
    for _ in 0..max {
        let hit = live_text(s)
            .lines()
            .any(|l| l.contains('▎') && l.contains(needle));
        if hit {
            return;
        }
        support::press_step(s, "down");
        std::thread::sleep(Duration::from_millis(120));
    }
    panic!("{what}: never reached `{needle}`");
}

fn tab_to_cursor(s: &mut Session, needle: &str, max: usize, what: &str) {
    for _ in 0..max {
        let hit = live_text(s)
            .lines()
            .any(|l| l.contains('▎') && l.contains(needle));
        if hit {
            return;
        }
        support::press_step(s, "tab");
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("{what}: cursor never reached `{needle}`");
}

fn up_to(s: &mut Session, needle: &str, max: usize, what: &str) {
    for _ in 0..max {
        let hit = live_text(s)
            .lines()
            .any(|l| l.contains('▎') && l.contains(needle));
        if hit {
            return;
        }
        support::press_step(s, "up");
        std::thread::sleep(Duration::from_millis(120));
    }
    panic!("{what}: never reached `{needle}`");
}

fn tab_to_footer(s: &mut Session, needle: &str, max: usize, what: &str) -> usize {
    for used in 0..max {
        let hit = live_text(s)
            .lines()
            .last()
            .is_some_and(|l| l.contains(needle));
        if hit {
            return used;
        }
        support::press_step(s, "tab");
        std::thread::sleep(Duration::from_millis(200));
    }
    panic!("{what}: footer never showed `{needle}`");
}

fn click_text(s: &mut Session, needle: &str, dcol: i16, timeout: Duration, what: &str) {
    let (row, col) = find_pos(s, needle, timeout);
    let target = col.saturating_add_signed(dcol);
    s.click(target, row)
        .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

fn strip_line(text: &str) -> &str {
    text.lines().nth(2).unwrap_or("")
}

fn grid_half(line: &str) -> &str {
    line.split("│ │").nth(1).unwrap_or(line)
}

fn explorer_half(text: &str) -> String {
    text.lines()
        .map(|l| l.split("│ │").next().unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}

fn grid_header_idx(text: &str, what: &str) -> usize {
    text.lines()
        .position(|l| l.contains("order_number"))
        .unwrap_or_else(|| panic!("{what}: no grid header on screen:\n{text}"))
}

fn wait_absent(s: &mut Session, needle: &str, secs: u64, what: &str) {
    for _ in 0..secs * 10 {
        if !live_text(s).contains(needle) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{what}: `{needle}` never left");
}

const KNOWN_STATUSES: [&str; 6] = [
    "pending",
    "paid",
    "shipped",
    "delivered",
    "cancelled",
    "refunded",
];

fn row_status(line: &str) -> Option<&str> {
    let half = grid_half(line);
    KNOWN_STATUSES.into_iter().find(|s| half.contains(s))
}

fn name_field_error(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let Some(i) = lines.iter().position(|l| l.contains("Name *")) else {
        return false;
    };
    lines[i..lines.len().min(i + 3)]
        .iter()
        .any(|l| l.contains("Required"))
}

fn poll_name_error(s: &mut Session, want: bool, secs: u64, what: &str) {
    for _ in 0..secs * 10 {
        if name_field_error(&live_text(s)) == want {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{what}: Name error never became {want}");
}

fn modal_region(text: &str, title: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let top = lines
        .iter()
        .position(|l| l.contains(title))
        .unwrap_or_else(|| panic!("modal `{title}` not open"));
    let bottom = lines[top..]
        .iter()
        .position(|l| l.contains('╰'))
        .map(|i| top + i)
        .unwrap_or(lines.len() - 1);
    lines[top..=bottom].iter().map(|s| s.to_string()).collect()
}

fn no_footer(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines.pop();
    lines.join("\n")
}

fn strip_col(text: &str, needle: &str, what: &str) -> u16 {
    let strip = strip_line(text);
    let off = strip
        .find(needle)
        .unwrap_or_else(|| panic!("{what}: `{needle}` not in strip"));
    unicode_width::UnicodeWidthStr::width(&strip[..off]) as u16
}

fn strip_click(s: &mut Session, needle: &str, dcol: i16, what: &str) {
    let col = strip_col(&live_text(s), needle, what);
    s.click(col.saturating_add_signed(dcol), 2)
        .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

fn drive_open_orders(s: &mut Session, timeout: Duration) {
    support::drive_with_timeout(
        s,
        &[
            "down",
            "down",
            "down",
            "down",
            "down",
            "enter",
            "wait:public › orders",
        ],
        drive_ms(timeout),
    );
}

fn drive_filtered_orders(s: &mut Session, timeout: Duration) {
    drive_open_orders(s, timeout);
    support::drive_with_timeout(
        s,
        &[
            "home",
            "right",
            "right",
            "right",
            "right",
            "f",
            "backtab",
            "backtab",
            "enter",
            "ctrl-l",
            "type:pending",
            "enter",
            "wait:filtered (1)",
        ],
        drive_ms(timeout),
    );
}

fn drive_readonly_query(s: &mut Session, timeout: Duration) {
    support::drive_with_timeout(
        s,
        &[
            "tab",
            "i",
            "type:SELECT status, total_amount FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 25",
        ],
        drive_ms(timeout),
    );
    support::press_step(s, "escape");
    std::thread::sleep(Duration::from_millis(300));
    if !live_text(s)
        .lines()
        .last()
        .is_some_and(|l| l.contains("EDIT"))
    {
        support::press_step(s, "i");
        std::thread::sleep(Duration::from_millis(200));
    }
    escape_to_idle(s, "readonly query");
    support::drive_with_timeout(s, &["ctrl-r", "wait:25 rows"], drive_ms(timeout));
}

fn drive_results_query(s: &mut Session, timeout: Duration) {
    support::drive_with_timeout(
        s,
        &[
            "tab",
            "i",
            "type:SELECT * FROM ord",
            "enter",
            "type: WHERE st",
            "tab",
            "type: = 'pending' ORDER BY created_at DESC LIMIT 25",
            "escape",
            "ctrl-r",
            "wait:25 rows",
        ],
        drive_ms(timeout),
    );
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_grid_local_sort() {
    let case = t2_case_wb("grid-local-sort");
    let dir = t2_dir("t2_grid_local_sort");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_open_orders(&mut s, timeout);
    let open = quiesce_text(&mut s, "grid-local-sort open");
    let header_idx = grid_header_idx(&open, "grid-local-sort open");
    let header: String = grid_half(open.lines().nth(header_idx).expect("header")).into();
    assert!(
        !header.contains('▴') && !header.contains('▾'),
        "fresh grid shows no sort marker: {header}"
    );
    assert!(
        open.contains("of 500 loaded"),
        "orders table loads its first page"
    );
    checkpoint(&mut s, &dir, "01-open");

    support::drive_with_timeout(
        &mut s,
        &["home", "right", "right", "right", "right"],
        drive_ms(timeout),
    );
    support::drive_with_timeout(
        &mut s,
        &["s", "wait:Sorted by status ascending"],
        drive_ms(timeout),
    );
    let asc = quiesce_text(&mut s, "grid-local-sort asc");
    let asc_header: String = grid_half(asc.lines().nth(header_idx).expect("header")).into();
    assert!(
        asc_header.contains("status ▴"),
        "asc marker suffixes the column: {asc_header}"
    );
    let first_asc: String = asc.lines().nth(header_idx + 1).expect("row").into();
    let lead_asc = row_status(&first_asc).expect("asc lead status");
    for i in 1..=3 {
        let r: String = asc.lines().nth(header_idx + i).expect("row").into();
        assert_eq!(
            row_status(&r),
            Some(lead_asc),
            "ascending groups its lead (V2): {r}"
        );
    }

    support::drive_with_timeout(
        &mut s,
        &["s", "wait:Sorted by status descending"],
        drive_ms(timeout),
    );
    let desc = quiesce_text(&mut s, "grid-local-sort desc");
    let desc_header: String = grid_half(desc.lines().nth(header_idx).expect("header")).into();
    assert!(
        desc_header.contains("status ▾"),
        "desc marker suffixes the column: {desc_header}"
    );
    let first_desc: String = desc.lines().nth(header_idx + 1).expect("row").into();
    let lead_desc = row_status(&first_desc).expect("desc lead status");
    assert_ne!(
        lead_asc, lead_desc,
        "direction flips the lead status (V2): {first_asc} vs {first_desc}"
    );
    for i in 1..=3 {
        let r: String = desc.lines().nth(header_idx + i).expect("row").into();
        assert_eq!(
            row_status(&r),
            Some(lead_desc),
            "descending groups its lead (V2): {r}"
        );
    }
    checkpoint(&mut s, &dir, "02-desc");

    support::drive_with_timeout(&mut s, &["s", "wait:Sort cleared"], drive_ms(timeout));
    let cleared = quiesce_text(&mut s, "grid-local-sort cleared");
    let cleared_header: String = grid_half(cleared.lines().nth(header_idx).expect("header")).into();
    assert!(
        !cleared_header.contains('▴') && !cleared_header.contains('▾'),
        "cleared grid drops the marker: {cleared_header}"
    );
    let first_cleared: String = cleared.lines().nth(header_idx + 1).expect("row").into();
    assert!(
        first_cleared.contains('│'),
        "cleared grid still lists rows: {first_cleared}"
    );

    support::drive_with_timeout(&mut s, &["s", "wait:Sorted by status"], drive_ms(timeout));
    support::drive_with_timeout(&mut s, &["S", "wait:Sort cleared"], drive_ms(timeout));
    let direct = quiesce_text(&mut s, "grid-local-sort direct-clear");
    let direct_header: String = grid_half(direct.lines().nth(header_idx).expect("header")).into();
    assert!(
        !direct_header.contains('▴') && !direct_header.contains('▾'),
        "`S` clears the marker: {direct_header}"
    );
    checkpoint(&mut s, &dir, "03-cleared");

    support::drive_with_timeout(&mut s, &["s", "wait:Sorted by status"], drive_ms(timeout));
    support::drive_with_timeout(
        &mut s,
        &[
            "f",
            "backtab",
            "backtab",
            "enter",
            "ctrl-l",
            "type:pending",
            "enter",
            "wait:filtered (1)",
        ],
        drive_ms(timeout),
    );

    let kept = quiesce_text(&mut s, "grid-local-sort filtered");
    let kept_header: String = grid_half(
        kept.lines()
            .find(|l| l.contains('▴'))
            .expect("sorted header"),
    )
    .into();
    assert!(
        kept_header.contains("status") && kept_header.contains('▴') && kept_header.contains('∇'),
        "filter keeps the sort marker beside its own: {kept_header}"
    );

    let case_b = t2_case_wb("grid-local-sort-click");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    drive_open_orders(&mut b, timeout);
    click_text(&mut b, "order_number", 1, timeout, "grid-local-sort click");
    waits::wait_state(&mut b, "Sorted by order_number ascending", "click sorts");
    let clicked = quiesce_text(&mut b, "grid-local-sort clicked");
    let clicked_idx = grid_header_idx(&clicked, "grid-local-sort clicked");
    let clicked_header: String =
        grid_half(clicked.lines().nth(clicked_idx).expect("header")).into();
    assert!(
        clicked_header.contains("order_number ▴"),
        "header click sorts ascending: {clicked_header}"
    );

    support::drive_with_timeout(
        &mut b,
        &[
            "home", "right", "right", "right", "right", "right", "right", "right", "s",
        ],
        drive_ms(timeout),
    );
    std::thread::sleep(Duration::from_millis(500));
    let refused = quiesce_text(&mut b, "grid-local-sort json-refused");
    let refused_header: String = grid_half(
        refused
            .lines()
            .find(|l| l.contains("shipping_address"))
            .expect("header"),
    )
    .into();
    assert!(
        !refused_header.contains('▴') && !refused_header.contains('▾'),
        "JSON column takes no marker: {refused_header}"
    );
    assert!(
        !refused.contains("Sorted by shipping_address"),
        "JSON column raises no sort status"
    );

    support::drive_with_timeout(
        &mut b,
        &["home", "right", "right", "right", "right", "s"],
        drive_ms(timeout),
    );
    waits::wait_state(
        &mut b,
        "Sorted by status ascending",
        "sortable column sorts",
    );
    checkpoint(&mut b, &dir, "04-click");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_grid_cell_lifecycle() {
    let case = t2_case_wb("grid-cell-lifecycle");
    let dir = t2_dir("t2_grid_cell_lifecycle");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    drive_open_orders(&mut s, timeout);
    support::drive_with_timeout(
        &mut s,
        &["home", "right", "right", "right", "right", "right"],
        drive_ms(timeout),
    );

    support::drive_with_timeout(&mut s, &["enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "Enter Commit", "cell editor opens");
    let editing = live_text(&mut s);
    let edit_footer: String = editing.lines().last().expect("footer").into();
    assert!(
        edit_footer.contains("EDIT") && edit_footer.contains("Esc Cancel"),
        "cell editing footer: {edit_footer}"
    );

    support::drive_with_timeout(
        &mut s,
        &["end", "ctrl-u", "type:99991", "enter"],
        drive_ms(timeout),
    );
    std::thread::sleep(Duration::from_millis(1000));
    checkpoint(&mut s, &dir, "01-dirty-prefind");
    waits::wait_state(&mut s, "• 1 ", "dirty bar counts the edit");
    let dirty = quiesce_text(&mut s, "grid-cell dirty");
    assert!(dirty.contains("99991"), "committed value shows");
    assert!(dirty.contains("• 1 pending"), "dirty action bar counts it");
    assert!(
        dirty.contains("Preview SQL") && dirty.contains("Discard") && dirty.contains("Save"),
        "dirty actions offered"
    );

    let (drow, dcol) = find_pos(&mut s, "99991", timeout);
    let frame = live_frame(&mut s);
    let dirty_bg = bg_rgb(&frame, dcol, drow, "grid-cell dirty tint");
    let committed_bg = bg_rgb(&frame, dcol, drow + 1, "grid-cell committed tint");
    assert_ne!(
        dirty_bg, committed_bg,
        "dirty tint differs from committed cells"
    );
    checkpoint(&mut s, &dir, "01-dirty");

    support::drive_with_timeout(&mut s, &["y", "wait:Copied 5 chars"], drive_ms(timeout));

    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, "Save changes?", "commit review opens");
    let review = live_text(&mut s);
    assert!(
        review.contains("Type orders to confirm"),
        "commit ack names the table"
    );
    assert!(
        review.contains("1 update · 0 inserts · 0 deletes"),
        "commit scope counts the edit"
    );
    checkpoint(&mut s, &dir, "02-commit");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Save changes?", "commit review dismisses");

    support::press_step(&s, "r");
    waits::wait_state(&mut s, "Discard unsaved changes?", "dirty refresh guards");
    let guard = live_text(&mut s);
    assert!(
        guard.contains("1 pending change will be dropped"),
        "discard count matches the edit"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Discard unsaved changes?", "guard dismisses");

    let case_b = t2_case_wb("grid-cell-pages");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    drive_open_orders(&mut b, timeout);

    support::press_step(&b, "alt-d");
    waits::wait_state(&mut b, "1 insert", "duplicate lands");
    let duped = quiesce_text(&mut b, "grid-cell duplicated");
    assert!(
        duped.contains("of 501 loaded"),
        "the copy appends a 501st loaded row"
    );
    checkpoint(&mut b, &dir, "03-duplicated");

    support::drive_with_timeout(&mut b, &["ctrl-end", "down", "enter"], drive_ms(timeout));
    waits::wait_state(
        &mut b,
        "Fetch more: the demo engine caps results at 500 rows",
        "fetch-more message",
    );

    support::drive_with_timeout(&mut b, &["F", "wait:Filters cleared"], drive_ms(timeout));
    support::press_step(&b, "r");
    waits::wait_state(&mut b, "Refreshed", "clean refresh");
    checkpoint(&mut b, &dir, "04-refreshed");

    let case_c = t2_case_wb("grid-cell-hscroll");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    drive_open_orders(&mut c, timeout);
    let boot = quiesce_text(&mut c, "grid-cell hscroll boot");
    assert!(
        boot.contains("of 14"),
        "14 columns overflow the viewport at boot"
    );
    support::drive_with_timeout(&mut c, &["end", "wait:cols 13–14 of 14"], drive_ms(timeout));
    support::drive_with_timeout(&mut c, &["y", "wait:Copied "], drive_ms(timeout));
    let before = copied_count(&live_text(&mut c), "grid-cell pre-page copy");

    support::press_step(&c, "ctrl-left");
    waits::wait_state(&mut c, "cols 11–13 of 14", "hscroll pages left");
    support::drive_with_timeout(&mut c, &["y", "wait:Copied "], drive_ms(timeout));
    let after = copied_count(&live_text(&mut c), "grid-cell post-page copy");
    assert_eq!(
        before, after,
        "paging never moves the cell cursor (N1): same value copies"
    );

    support::press_step(&c, "ctrl-right");
    std::thread::sleep(Duration::from_millis(800));
    checkpoint(&mut c, &dir, "05-hscroll-right-prefind");
    waits::wait_state(&mut c, "cols 12–14 of 14", "hscroll pages right");
    support::drive_with_timeout(&mut c, &["y", "wait:Copied "], drive_ms(timeout));
    let back = copied_count(&live_text(&mut c), "grid-cell return copy");
    assert_eq!(before, back, "paging back keeps the cursor too");
    checkpoint(&mut c, &dir, "05-hscroll");

    let case_d = t2_case_wb("grid-cell-readonly");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    drive_readonly_query(&mut d, timeout);
    tab_to_footer(&mut d, "↑↓←→ Cell", 8, "grid-cell focus results");
    support::drive_with_timeout(&mut d, &["y", "wait:Copied "], drive_ms(timeout));
    let ro_before = no_footer(&quiesce_text(&mut d, "grid-cell readonly before"));
    assert!(
        !ro_before.lines().last().is_some_and(|l| l.contains("EDIT")),
        "results grid idles without EDIT"
    );
    support::press_step(&d, "enter");
    std::thread::sleep(Duration::from_millis(600));
    checkpoint(&mut d, &dir, "06-activated-prefind");
    let ro_after = live_text(&mut d);
    assert!(
        !ro_after.lines().last().is_some_and(|l| l.contains("EDIT"))
            && !ro_after.contains("Enter Commit")
            && !ro_after.contains("• 1 "),
        "Enter emits Activated with no visible edit (N2)"
    );
    support::press_step(&d, "alt-d");
    std::thread::sleep(Duration::from_millis(600));
    let ro_dup = live_text(&mut d);
    assert!(
        !ro_dup.contains("1 insert") && !ro_dup.contains("501"),
        "read-only grids refuse duplication too"
    );
    checkpoint(&mut d, &dir, "06-readonly");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_grid_results_view() {
    let case = t2_case_wb("grid-results-view");
    let dir = t2_dir("t2_grid_results_view");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_results_query(&mut s, timeout);

    let ran = quiesce_text(&mut s, "grid-results ran");
    let meta: String = ran
        .lines()
        .find(|l| l.contains("25 rows · "))
        .expect("result meta")
        .into();
    assert!(meta.contains(" ms"), "duration meta: {meta}");
    assert!(
        ran.contains("SELECT orders (25)"),
        "result tab counts the rows"
    );
    let editor_line: String = ran
        .lines()
        .find(|l| l.contains("FROM orders"))
        .expect("editor statement")
        .into();
    let header_idx = grid_header_idx(&ran, "grid-results header");
    checkpoint(&mut s, &dir, "01-results");
    tab_to_footer(&mut s, "↑↓←→ Cell", 8, "grid-results focus");
    support::drive_with_timeout(&mut s, &["y", "wait:Copied "], drive_ms(timeout));

    support::drive_with_timeout(
        &mut s,
        &["home", "right", "right", "right", "right"],
        drive_ms(timeout),
    );

    let settled = quiesce_text(&mut s, "grid-results settled");
    let first_before: String = settled.lines().nth(header_idx + 1).expect("row").into();
    support::press_step(&s, "s");
    let mut sorted = String::new();
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        sorted = live_text(&mut s);
        if sorted.contains("status ▴") {
            break;
        }
    }
    assert!(
        sorted.contains("status ▴"),
        "local sort marks the result header"
    );
    assert!(
        !sorted.contains("Sorted by"),
        "no SortRequested leaves the widget (S1)"
    );

    let lead_after =
        row_status(sorted.lines().nth(header_idx + 1).expect("row")).expect("sorted lead status");
    for i in 1..=3 {
        let r: String = sorted.lines().nth(header_idx + i).expect("row").into();
        assert_eq!(
            row_status(&r),
            Some(lead_after),
            "local sort groups its lead: {r}"
        );
    }

    let meta_after: String = sorted
        .lines()
        .find(|l| l.contains("25 rows · "))
        .expect("result meta")
        .into();
    assert_eq!(meta, meta_after, "duration meta survives the sort");
    let editor_after: String = sorted
        .lines()
        .find(|l| l.contains("FROM orders"))
        .expect("editor statement")
        .into();

    assert_eq!(
        editor_line.replace('▎', " "),
        editor_after.replace('▎', " "),
        "statement survives the sort"
    );

    support::drive_with_timeout(&mut s, &["s"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    support::drive_with_timeout(&mut s, &["s"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(600));
    let restored = quiesce_text(&mut s, "grid-results restored");
    assert!(
        !restored.contains("Sorted by") && !restored.contains("Sort cleared"),
        "the local path emits no sort events at all"
    );
    let first_restored: String = restored.lines().nth(header_idx + 1).expect("row").into();

    let norm = |s: &str| {
        grid_half(s)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    assert_eq!(
        norm(&first_before),
        norm(&first_restored),
        "local sort only permutes the display order (N2)"
    );
    checkpoint(&mut s, &dir, "02-sorted");

    let case_b = t2_case_wb("grid-results-empty");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive_with_timeout(
        &mut b,
        &[
            "tab",
            "i",
            "type:SELECT * FROM ord",
            "enter",
            "type: WHERE st",
            "tab",
            "type: = 'zzz-no-match'",
            "escape",
            "ctrl-r",
            "wait:No rows",
        ],
        drive_ms(timeout),
    );
    let empty = quiesce_text(&mut b, "grid-results empty");
    assert!(empty.contains("No rows"), "empty state label");
    assert!(
        empty.contains("The query matched nothing"),
        "empty state hint"
    );
    let grid_text: String = empty.lines().map(grid_half).collect::<Vec<_>>().join("\n");
    assert!(
        !grid_text.contains('▴') && !grid_text.contains('▾') && !grid_text.contains('∇'),
        "empty state shows no grid chrome (N2)"
    );
    checkpoint(&mut b, &dir, "03-empty");

    let case_c = t2_case_wb("grid-results-pairing");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    drive_open_orders(&mut c, timeout);
    support::drive_with_timeout(&mut c, &["enter"], drive_ms(timeout));
    waits::wait_state(&mut c, "Enter Commit", "table grid edits on Enter");

    let case_e = t2_case_wb("grid-results-readonly");
    write_provenance_named(&dir, &case_e, "provenance-e.txt");
    let mut e = support::spawn_boot(&case_e);
    drive_readonly_query(&mut e, timeout);
    tab_to_footer(&mut e, "↑↓←→ Cell", 8, "grid-results focus readonly");
    support::drive_with_timeout(&mut e, &["y", "wait:Copied "], drive_ms(timeout));
    let ro_before = no_footer(&quiesce_text(&mut e, "grid-results readonly before"));
    assert!(
        !ro_before.lines().last().is_some_and(|l| l.contains("EDIT")),
        "results grid idles without EDIT"
    );
    support::press_step(&e, "enter");
    std::thread::sleep(Duration::from_millis(600));
    let ro_after = live_text(&mut e);
    assert!(
        !ro_after.lines().last().is_some_and(|l| l.contains("EDIT"))
            && !ro_after.contains("Enter Commit")
            && !ro_after.contains("• 1 "),
        "Enter emits Activated with no visible edit (A2)"
    );
    support::press_step(&e, "alt-d");
    std::thread::sleep(Duration::from_millis(600));
    let ro_dup = live_text(&mut e);
    assert!(
        !ro_dup.contains("1 insert") && !ro_dup.contains("of 501"),
        "read-only grids refuse duplication (A2)"
    );
    checkpoint(&mut e, &dir, "04-readonly");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_code_sql_editor() {
    let case = t2_case_wb("code-sql-editor");
    let dir = t2_dir("t2_code_sql_editor");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["tab", "i", "type:SELECT zzz FROM orders"],
        drive_ms(timeout),
    );
    escape_to_idle(&mut s, "code-sql keywords");
    let frame = live_frame(&mut s);
    let (krow, kcol) = find_pos(&mut s, "SELECT", timeout);
    for (i, expect) in ["SELECT", "FROM"].iter().enumerate() {
        let (r, c) = find_pos(&mut s, expect, timeout);
        assert_eq!((r, c), (krow, kcol + i as u16 * 11), "keyword row");
        for j in 0..expect.len() as u16 {
            let cell = frame
                .get(c + j, r)
                .unwrap_or_else(|| panic!("code-sql: no cell for {expect}[{j}]"));
            assert!(cell.mods.bold, "keyword `{expect}` is bold");
        }
    }
    let select_fg = fg_rgb(&frame, kcol, krow, "code-sql SELECT tone");
    let (frow, fcol) = (krow, kcol + 11);
    assert_eq!(
        select_fg,
        fg_rgb(&frame, fcol, frow, "code-sql FROM tone"),
        "keywords share one tone (S2)"
    );

    let (zrow, zcol) = find_pos(&mut s, "zzz", timeout);
    for j in 0..3u16 {
        assert!(
            frame.get(zcol + j, zrow).is_some_and(|c| !c.mods.bold),
            "plain words escape the keyword tone (N2)"
        );
    }
    checkpoint(&mut s, &dir, "01-tones");

    let case_b = t2_case_wb("code-sql-blocks");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);

    support::drive_with_timeout(
        &mut b,
        &["tab", "i", "type:SELECT * FROM customers LIMIT 1;"],
        drive_ms(timeout),
    );
    support::press_step(&b, "escape");
    std::thread::sleep(Duration::from_millis(300));
    if !live_text(&mut b)
        .lines()
        .last()
        .is_some_and(|l| l.contains("EDIT"))
    {
        support::press_step(&b, "i");
        std::thread::sleep(Duration::from_millis(200));
    }
    support::drive_with_timeout(
        &mut b,
        &["enter", "type:SELECT * FROM orders LIMIT 1;"],
        drive_ms(timeout),
    );
    let two = live_text(&mut b);
    let line1: String = two
        .lines()
        .find(|l| l.contains("FROM customers"))
        .expect("first statement")
        .into();
    let line2: String = two
        .lines()
        .find(|l| l.contains("FROM orders"))
        .expect("second statement")
        .into();
    assert_ne!(line1, line2, "statements sit on separate lines");
    assert!(
        !line1.contains('▎') && line2.contains('▎'),
        "cursor line owns the gutter bar (V1)"
    );
    checkpoint(&mut b, &dir, "02-multiline");
    escape_to_idle(&mut b, "code-sql blocks");

    support::drive_with_timeout(&mut b, &["ctrl-r"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(2500));
    checkpoint(&mut b, &dir, "03-run-prefind");
    waits::wait_state(&mut b, "SELECT orders (1)", "line-2 run lands");
    let ran2 = quiesce_text(&mut b, "code-sql ran line 2");
    assert!(
        !ran2.contains("SELECT customers (1)"),
        "line-1 statement never ran"
    );
    assert!(
        ran2.contains("FROM customers") && ran2.contains("FROM orders"),
        "running keeps the typed statements (N1)"
    );

    if !live_text(&mut b)
        .lines()
        .last()
        .is_some_and(|l| l.contains("EDIT"))
    {
        support::press_step(&b, "i");
        std::thread::sleep(Duration::from_millis(300));
    }
    support::drive_with_timeout(
        &mut b,
        &["up", "ctrl-r", "wait:SELECT customers (1)"],
        drive_ms(timeout),
    );
    let ran1 = quiesce_text(&mut b, "code-sql ran line 1");
    assert!(
        ran1.contains("FROM customers") && ran1.contains("FROM orders"),
        "statements survive both runs (N1)"
    );
    checkpoint(&mut b, &dir, "03-blocks");

    let case_c = t2_case_wb("code-sql-dirty");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    let clean = quiesce_text(&mut c, "code-sql clean");
    assert!(
        !strip_line(&clean).contains('•'),
        "fresh query tab carries no dot"
    );
    support::drive_with_timeout(&mut c, &["tab", "i", "type:X"], drive_ms(timeout));
    escape_to_idle(&mut c, "code-sql dirty");

    support::drive_with_timeout(&mut c, &["ctrl-t", "wait:Query 2"], drive_ms(timeout));
    let dirtied = live_text(&mut c);
    assert!(
        strip_line(&dirtied).contains("Query 1 •"),
        "typing dots the query tab (S1)"
    );
    checkpoint(&mut c, &dir, "04-dirty");

    let case_d = t2_case_wb("code-sql-error");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::drive_with_timeout(
        &mut d,
        &["tab", "i", "type:SELECT * FROM nope"],
        drive_ms(timeout),
    );
    escape_to_idle(&mut d, "code-sql error");
    support::drive_with_timeout(
        &mut d,
        &["ctrl-r", "wait:does not exist"],
        drive_ms(timeout),
    );
    let failed = quiesce_text(&mut d, "code-sql failed");
    assert!(
        failed.contains("SELECT * FROM nope"),
        "failed run keeps the statement (V2)"
    );
    assert!(
        failed.contains("Error") && failed.contains("failed · "),
        "error readout renders below (V2)"
    );
    assert!(
        failed.contains("relation \"nope\" does not exist"),
        "engine message surfaces"
    );
    checkpoint(&mut d, &dir, "05-error");

    let case_e = t2_case_wb("code-sql-complete");
    write_provenance_named(&dir, &case_e, "provenance-e.txt");
    let mut e = support::spawn_boot(&case_e);
    support::drive_with_timeout(
        &mut e,
        &["tab", "i", "type:SELECT * FROM ord"],
        drive_ms(timeout),
    );
    let mut popped = String::new();
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        popped = live_text(&mut e);
        if popped
            .lines()
            .any(|l| l.contains('▎') && l.contains("orders"))
        {
            break;
        }
    }
    let editor_idx = popped
        .lines()
        .position(|l| l.contains("FROM ord"))
        .expect("editor line");
    let pick_idx = popped
        .lines()
        .position(|l| l.contains('▎') && l.contains("orders"))
        .expect("completion selection");
    assert!(
        pick_idx > editor_idx,
        "completion opens from the cursor cell downward (A2)"
    );
    escape_to_idle(&mut e, "code-sql complete");
    let closed = quiesce_text(&mut e, "code-sql complete closed");
    assert!(
        !closed
            .lines()
            .any(|l| l.contains('▎') && l.contains("orders")),
        "Esc closes the completion popup"
    );
    checkpoint(&mut e, &dir, "06-complete");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_select_filter_builder() {
    let case = t2_case_wb("select-filter-builder");
    let dir = t2_dir("t2_select_filter_builder");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_open_orders(&mut s, timeout);
    support::drive_with_timeout(&mut s, &["ctrl-f", "wait:Add filter"], drive_ms(timeout));

    let open = quiesce_text(&mut s, "select-filter open");
    let selects: String = modal_region(&open, "Add filter")
        .iter()
        .find(|l| l.contains("id") && l.contains('▾'))
        .expect("select row")
        .into();
    assert!(
        selects.contains("id") && selects.contains('='),
        "Column/Operator defaults share the select row: {selects}"
    );
    assert!(
        open.contains("Operators that fit the col"),
        "operator help row renders"
    );
    assert!(
        open.contains("WHERE id = ''"),
        "blank preview shows the default predicate"
    );
    checkpoint(&mut s, &dir, "01-open");

    support::drive_with_timeout(&mut s, &["backtab", "backtab", "enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "customer_id", "column popup opens");
    let cols = live_text(&mut s);
    assert!(
        cols.contains("shipping_address"),
        "column popup lists down to the JSON column"
    );
    assert!(
        cols.lines().any(|l| l.contains('▎') && l.contains('▴')),
        "the open select carries ▴"
    );

    support::drive_with_timeout(
        &mut s,
        &[
            "down", "down", "down", "down", "down", "down", "down", "enter",
        ],
        drive_ms(timeout),
    );
    waits::wait_state(&mut s, "shipping_address", "column commits");
    let reseeded = quiesce_text(&mut s, "select-filter reseeded");
    let op_row: String = reseeded
        .lines()
        .find(|l| l.contains("shipping_address") && l.contains("is NULL"))
        .expect("reseeded select row")
        .into();
    assert!(
        op_row.contains('▾'),
        "reseeded Operator stays a closed select: {op_row}"
    );
    assert!(
        reseeded.contains("NULL") && reseeded.contains("shipping_address"),
        "preview follows the JSON column"
    );
    checkpoint(&mut s, &dir, "02-reseeded");

    support::drive_with_timeout(&mut s, &["tab", "enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "is empty", "operator popup opens");
    checkpoint(&mut s, &dir, "03-popup-prefind");
    support::drive_with_timeout(&mut s, &["down", "down"], drive_ms(timeout));
    checkpoint(&mut s, &dir, "03-popup-moved");
    let mut cursor_moved = false;
    for _ in 0..80 {
        if live_text(&mut s)
            .lines()
            .any(|l| l.contains('▎') && l.contains("is empty"))
        {
            cursor_moved = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(cursor_moved, "cursor reaches a visible row");
    let ops = live_text(&mut s);
    let pos = |needle: &str| {
        ops.lines()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("operator `{needle}` listed"))
    };
    assert!(
        pos("is empty") < pos("contains"),
        "visible JSON operators keep type order (V2)"
    );
    assert!(
        ops.lines()
            .any(|l| l.contains('›') && l.contains("is NULL")),
        "› stays on the committed operator (V2)"
    );

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(400));
    let tabbed = live_text(&mut s);
    assert!(
        tabbed.contains("is empty") && tabbed.contains("contains"),
        "Tab leaves the open popup in place (S2)"
    );
    checkpoint(&mut s, &dir, "03-popup");

    support::press_step(&s, "enter");
    wait_absent(&mut s, "contains", 8, "operator commits");
    let repainted = quiesce_text(&mut s, "select-filter repainted");
    let repainted_row: String = repainted
        .lines()
        .find(|l| l.contains("shipping_address") && l.contains("is empty"))
        .expect("repainted select row")
        .into();
    assert!(
        repainted_row.contains('▾'),
        "committed Operator closes its select: {repainted_row}"
    );
    assert!(
        repainted.contains("Add filter") && !repainted.contains("filtered ("),
        "operator change applies no filter (A1)"
    );

    support::drive_with_timeout(&mut s, &["backtab", "enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "customer_id", "column popup reopens");
    support::drive_with_timeout(&mut s, &["up", "up", "up", "enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "WHERE status", "status column commits");
    let enum_seeded = quiesce_text(&mut s, "select-filter enum reseeded");
    assert!(
        enum_seeded
            .lines()
            .any(|l| l.contains("status") && l.contains('=') && l.contains('▾')),
        "`=` re-seeds first for enum columns (N2)"
    );
    support::drive_with_timeout(&mut s, &["tab", "enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "not in", "enum operators open");
    support::drive_with_timeout(&mut s, &["down", "down"], drive_ms(timeout));
    checkpoint(&mut s, &dir, "04-enum-moved");

    let mut enum_moved = false;
    for _ in 0..80 {
        if modal_region(&live_text(&mut s), "Add filter")
            .iter()
            .any(|l| l.contains('▎') && !l.contains('›'))
        {
            enum_moved = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(enum_moved, "enum cursor leaves the committed row");
    let enum_ops = live_text(&mut s);
    let epos = |needle: &str| {
        enum_ops
            .lines()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("operator `{needle}` listed"))
    };
    assert!(
        epos("not in") < epos("is NULL"),
        "visible enum operators keep type order (N2)"
    );

    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "not in", "Esc closes the popup (N1)");
    let kept = live_text(&mut s);
    assert!(
        kept.contains("Add filter"),
        "closing the popup keeps the editor (N1)"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Add filter", "Esc cancels the editor (A2)");
    checkpoint(&mut s, &dir, "04-cancelled");

    let case_b = t2_case_wb("select-filter-tab");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    drive_open_orders(&mut b, timeout);
    support::drive_with_timeout(&mut b, &["ctrl-f", "wait:Add filter"], drive_ms(timeout));
    support::drive_with_timeout(&mut b, &["enter", "type:abc", "tab"], drive_ms(timeout));
    waits::wait_state(&mut b, "WHERE id = 'abc'", "Tab commits the value");
    checkpoint(&mut b, &dir, "05-tab-prefind");

    let advanced = live_text(&mut b);
    assert!(
        advanced
            .lines()
            .any(|l| l.contains('▎') && l.contains("Cancel")),
        "committed Tab advances out of the Value (S2)"
    );
    support::press_step(&b, "tab");
    std::thread::sleep(Duration::from_millis(300));
    let applied = live_text(&mut b);
    assert!(
        applied
            .lines()
            .any(|l| l.contains('▎') && l.contains("Add filter")),
        "idle Tab traverses to the apply button (S2)"
    );
    support::press_step(&b, "escape");
    waits::wait_gone(&mut b, "Add filter", "idle Esc cancels (A2)");
    checkpoint(&mut b, &dir, "05-tab");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_chip_filter_bar() {
    let case = t2_case_wb("chip-filter-bar");
    let dir = t2_dir("t2_chip_filter_bar");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_filtered_orders(&mut s, timeout);

    let bar = quiesce_text(&mut s, "chip-filter bar");
    let chips: String = bar
        .lines()
        .find(|l| l.contains("match all"))
        .expect("chip bar")
        .into();
    assert!(
        chips.contains("status = 'pending'") && chips.contains('×'),
        "one labeled chip with ×: {chips}"
    );
    assert!(
        chips.contains("match all ▾"),
        "lead matches match_all: {chips}"
    );
    assert!(chips.contains("+ Add filter"), "add stop renders: {chips}");

    assert!(
        bar.contains("filtered (1)"),
        "counts mirror the single enabled filter"
    );
    checkpoint(&mut s, &dir, "01-bar-prefind");

    let bar_header = bar
        .lines()
        .position(|l| l.contains('∇'))
        .expect("filtered header");
    assert!(
        bar.lines()
            .nth(bar_header + 1)
            .is_some_and(|l| l.contains("pending")),
        "every filtered row is pending"
    );
    checkpoint(&mut s, &dir, "01-bar");
    tab_to_footer(&mut s, "Enter Edit filter", 8, "chip-filter focus");

    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Edit filter", "chip activates its editor");

    quiesce_text(&mut s, "chip-filter editor settled");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(500));
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Edit filter", "editor cancels back to the bar");

    tab_to_footer(&mut s, "Enter Edit filter", 8, "chip-filter refocus");
    let (on_row, on_col) = find_pos(&mut s, "status = 'pending'", timeout);
    let on_lum = lum(fg_rgb(
        &live_frame(&mut s),
        on_col,
        on_row,
        "chip-filter enabled tone",
    ));
    support::press_step(&s, "space");
    std::thread::sleep(Duration::from_millis(1500));
    checkpoint(&mut s, &dir, "02-disabled-prefind");
    waits::wait_state(&mut s, "of 500 loaded", "disabled chip leaves the query");
    let off = quiesce_text(&mut s, "chip-filter disabled");
    assert!(
        off.contains("of 500 loaded"),
        "disabled chip contributes nothing (N2)"
    );
    assert!(
        off.contains("status = 'pending'"),
        "the chip itself stays listed (S1)"
    );
    let off_lum = lum(fg_rgb(
        &live_frame(&mut s),
        on_col,
        on_row,
        "chip-filter disabled tone",
    ));
    assert!(
        off_lum < on_lum,
        "disabled chip dims (S2): {on_lum:.2} -> {off_lum:.2}"
    );
    checkpoint(&mut s, &dir, "02-disabled");
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "filtered (1)", "chip re-enables");
    let re = quiesce_text(&mut s, "chip-filter re-enabled");
    let re_header = re
        .lines()
        .position(|l| l.contains('∇'))
        .expect("filtered header");
    assert!(
        re.lines()
            .nth(re_header + 1)
            .is_some_and(|l| l.contains("pending")),
        "the filter applies again"
    );

    support::press_step(&s, "delete");
    waits::wait_state(&mut s, "Filter removed", "removal announces");
    let removed = quiesce_text(&mut s, "chip-filter removed");
    assert!(!removed.contains("status = 'pending'"), "the chip is gone");
    assert!(
        removed.contains("of 500 loaded") && !removed.contains("filtered ("),
        "the query runs unfiltered"
    );

    let empty_bar: String = removed
        .lines()
        .find(|l| l.contains("match all"))
        .expect("empty bar")
        .into();
    assert!(
        empty_bar.contains("match all ▾") && empty_bar.contains("+ Add filter"),
        "lead and add survive emptiness: {empty_bar}"
    );

    click_text(&mut s, "match all", 1, timeout, "chip-filter lead off");
    waits::wait_state(&mut s, "match any ▾", "lead flips to any");
    click_text(&mut s, "match any", 1, timeout, "chip-filter lead on");
    waits::wait_state(&mut s, "match all ▾", "lead flips back to all");
    checkpoint(&mut s, &dir, "04-lead");

    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Add filter", "add stop opens a blank editor");

    quiesce_text(&mut s, "chip-filter blank settled");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(600));
    if live_text(&mut s).contains("Add filter") {
        support::press_step(&s, "escape");
    }
    waits::wait_gone(&mut s, "Add filter", "blank editor cancels");
    checkpoint(&mut s, &dir, "03-empty");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_tabs_workbench_states() {
    let case = t2_case_wb("tabs-workbench-states");
    let dir = t2_dir("t2_tabs_workbench_states");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let fresh = strip_line(&quiesce_text(&mut s, "tabs fresh")).to_string();
    assert!(
        fresh.contains("Query 1")
            && !fresh.contains('•')
            && !SPIN.chars().any(|c| fresh.contains(c)),
        "fresh strip idles glyph-free: {fresh}"
    );

    support::drive_with_timeout(&mut s, &["tab", "i", "type:X"], drive_ms(timeout));
    escape_to_idle(&mut s, "tabs dirty");
    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], drive_ms(timeout));
    escape_to_idle(&mut s, "tabs quiet");
    let dotted = strip_line(&live_text(&mut s)).to_string();
    assert!(
        dotted.contains("Query 1 •"),
        "dirty dot trails the label (V1): {dotted}"
    );
    checkpoint(&mut s, &dir, "01-dirty");

    let pair = live_text(&mut s);
    let active_col = strip_col(&pair, "Query 2", "tabs active tab") + 3;
    let quiet_col = strip_col(&pair, "Query 1", "tabs quiet tab") + 3;
    let frame = live_frame(&mut s);
    let active_fg = fg_rgb(&frame, active_col, 3, "tabs accent rule");
    let quiet_fg = fg_rgb(&frame, quiet_col, 3, "tabs quiet rule");
    assert_eq!(
        frame
            .get(active_col, 3)
            .map(|c| c.symbol.clone())
            .unwrap_or_default(),
        "━",
        "active rule uses ━"
    );
    assert_eq!(
        frame
            .get(quiet_col, 3)
            .map(|c| c.symbol.clone())
            .unwrap_or_default(),
        "─",
        "quiet rule uses ─"
    );
    assert_ne!(
        active_fg, quiet_fg,
        "one accent underline per screen (V2): {active_fg:?} vs {quiet_fg:?}"
    );
    checkpoint(&mut s, &dir, "02-quiet");

    tab_to_footer(&mut s, "Alt+R Run all", 8, "tabs focus editor");
    support::drive_with_timeout(&mut s, &["i", "type:SELECT * FROM nope"], drive_ms(timeout));
    escape_to_idle(&mut s, "tabs failing query");
    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 3"], drive_ms(timeout));
    support::press_step(&s, "[");
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&s, "ctrl-r");
    std::thread::sleep(Duration::from_millis(100));
    support::press_step(&s, "]");
    std::thread::sleep(Duration::from_millis(3000));

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 4"], drive_ms(timeout));
    let mut flagged = String::new();
    for _ in 0..200 {
        std::thread::sleep(Duration::from_millis(50));
        flagged = strip_line(&live_text(&mut s)).to_string();
        if flagged.contains("Query 2 !") {
            break;
        }
    }
    assert!(
        flagged.contains("Query 2 !"),
        "unseen failure flags the tab (V1): {flagged}"
    );
    checkpoint(&mut s, &dir, "03-error");
    support::press_step(&s, "[");
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&s, "[");
    std::thread::sleep(Duration::from_millis(500));
    let seen = live_text(&mut s);
    assert!(!strip_line(&seen).contains('!'), "viewing clears the flag");
    assert!(
        seen.contains("does not exist"),
        "the error card explains the flag"
    );

    let case_b = t2_case_wb("tabs-busy");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive_with_timeout(
        &mut b,
        &["tab", "i", "type:SELECT * FROM orders"],
        drive_ms(timeout),
    );
    escape_to_idle(&mut b, "tabs busy query");
    support::drive_with_timeout(&mut b, &["ctrl-t", "wait:Query 2", "["], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&b, "ctrl-r");
    std::thread::sleep(Duration::from_millis(100));
    support::press_step(&b, "]");
    let mut spun = false;
    for _ in 0..120 {
        std::thread::sleep(Duration::from_millis(50));
        if strip_line(&live_text(&mut b))
            .chars()
            .any(|c| SPIN.contains(c))
        {
            spun = true;
            break;
        }
    }
    assert!(spun, "busy tab spins while the run is in flight (V1)");
    checkpoint(&mut b, &dir, "04-busy");

    let case_c = t2_case_wb("tabs-overflow");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    for _ in 0..10 {
        support::press_step(&c, "ctrl-t");
        std::thread::sleep(Duration::from_millis(150));
    }
    waits::wait_state(&mut c, "Query 11", "eleven queries open");
    tab_to_footer(&mut c, "Enter Open", 8, "tabs focus tree");
    drive_open_orders(&mut c, timeout);

    tab_to_footer(&mut c, "← → Switch", 8, "tabs focus strip for paging");
    let tail = strip_line(&quiesce_text(&mut c, "tabs tail")).to_string();
    assert!(
        tail.contains('‹') && !tail.contains('›'),
        "tail view counts the hidden head (V3): {tail}"
    );
    assert!(
        tail.trim_end().ends_with('+'),
        "new-tab button trails (V3): {tail}"
    );
    assert!(!tail.contains('…'), "labels never shrink (N2): {tail}");
    assert!(tail.contains("T orders"), "the active tail tab shows");

    support::press_step(&c, "5");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("─ Query 5 ─"),
        "digit-jump activates Query 5"
    );
    let middle = strip_line(&live_text(&mut c)).to_string();
    assert!(
        middle.contains('‹') && middle.contains('›') && middle.contains("Query 5"),
        "middle view overflows both ways: {middle}"
    );
    strip_click(&mut c, "‹", 0, "tabs page left");
    let mut paged = String::new();
    for _ in 0..30 {
        std::thread::sleep(Duration::from_millis(100));
        paged = strip_line(&live_text(&mut c)).to_string();
        if paged != middle {
            break;
        }
    }
    checkpoint(&mut c, &dir, "05-page-stuck");
    assert!(
        paged != middle && paged.contains("Query 5"),
        "‹ pages first head-ward, active kept (A2): {paged}"
    );
    assert!(!paged.contains('…'), "paged view keeps full labels (N2)");
    strip_click(&mut c, "›", 0, "tabs page right");
    let mut back = String::new();
    for _ in 0..30 {
        std::thread::sleep(Duration::from_millis(100));
        back = strip_line(&live_text(&mut c)).to_string();
        if back == middle {
            break;
        }
    }
    assert!(back == middle, "› pages first tail-ward (A2): {back}");
    checkpoint(&mut c, &dir, "05-overflow");

    tab_to_footer(&mut c, "← → Switch", 8, "tabs focus strip");
    support::press_step(&c, "5");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("─ Query 5 ─"),
        "digit-jump activates Query 5"
    );
    support::press_step(&c, "x");
    wait_absent(&mut c, "Query 5", 8, "middle tab closes (A1/S1)");

    let repaired = strip_line(&quiesce_text(&mut c, "tabs repaired")).to_string();
    assert!(
        repaired.contains("Query 6"),
        "the next neighbor survives the middle close (S1): {repaired}"
    );
    let repaired_full = live_text(&mut c);
    assert!(
        repaired_full.contains("─ Query 6 ─"),
        "remove repairs active onto Query 6 (S1)"
    );

    for _ in 0..6 {
        support::press_step(&c, "]");
        std::thread::sleep(Duration::from_millis(150));
    }
    let tail_revealed = strip_line(&live_text(&mut c)).to_string();
    assert!(
        tail_revealed.contains("T orders")
            && tail_revealed.contains('‹')
            && !tail_revealed.contains('›'),
        "stepping tail-ward re-reveals the tail (S2): {tail_revealed}"
    );
    for _ in 0..10 {
        support::press_step(&c, "[");
        std::thread::sleep(Duration::from_millis(150));
    }
    let head_revealed = strip_line(&live_text(&mut c)).to_string();
    assert!(
        head_revealed.contains("Query 1")
            && head_revealed.contains('›')
            && !head_revealed.contains('‹'),
        "stepping head-ward re-reveals the head (S2): {head_revealed}"
    );

    support::press_step(&c, "delete");
    std::thread::sleep(Duration::from_millis(400));
    let after_del = live_text(&mut c);
    assert!(
        !strip_line(&after_del).contains("Query 1") && after_del.contains("─ Query 2 ─"),
        "Delete repairs active onto Query 2 (S1)"
    );
    checkpoint(&mut c, &dir, "06-repaired");

    tab_to_footer(&mut c, "← → Switch", 8, "tabs refocus strip");
    support::press_step(&c, "n");
    waits::wait_state(&mut c, "Query 12", "n opens a query (A1)");
    strip_click(&mut c, "+", 0, "tabs + button");
    waits::wait_state(&mut c, "Query 13", "+ opens a query (A1)");

    tab_to_footer(&mut c, "← → Switch", 8, "tabs refocus for jump");
    support::press_step(&c, "2");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("─ Query 3 ─"),
        "digit-jump activates Query 3"
    );
    assert!(
        strip_line(&live_text(&mut c)).contains("Query 3"),
        "the strip re-reveals the jumped tab"
    );
    strip_click(&mut c, "Query 3", 2, "tabs activate by click");
    std::thread::sleep(Duration::from_millis(400));
    let clicked = live_text(&mut c);
    assert!(
        clicked.contains("─ Query 3 ─"),
        "tab click activates its pane (A2)"
    );
    checkpoint(&mut c, &dir, "07-actions");

    tab_to_footer(&mut c, "← → Switch", 8, "tabs focus for emptying");
    for _ in 0..20 {
        let strip = strip_line(&live_text(&mut c)).to_string();
        if !strip.contains("Query") && !strip.contains("T ") {
            break;
        }
        support::press_step(&c, "x");
        std::thread::sleep(Duration::from_millis(200));
    }
    let empty = strip_line(&quiesce_text(&mut c, "tabs empty")).to_string();
    assert!(
        !empty.contains("Query") && !empty.contains("T ") && empty.contains('+'),
        "strip empties to the + button: {empty}"
    );
    let inert = quiesce_text(&mut c, "tabs empty settled");
    support::press_step(&c, "x");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        inert,
        quiesce_text(&mut c, "tabs x on empty"),
        "x answers nothing on an empty strip (N1)"
    );
    tab_to_footer(&mut c, "← → Switch", 8, "tabs refocus when empty");
    support::press_step(&c, "enter");
    waits::wait_state(&mut c, "Query", "Enter creates on empty (N1)");
    tab_to_footer(&mut c, "← → Switch", 8, "tabs focus again");
    support::press_step(&c, "x");
    waits::wait_gone(&mut c, "Query", "strip empties again");
    support::press_step(&c, "n");
    waits::wait_state(&mut c, "Query", "n creates on empty (N1)");
    checkpoint(&mut c, &dir, "08-empty");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_input_form_field() {
    let case = t2_case_conn("input-form-field");
    let dir = t2_dir("t2_input_form_field");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    support::drive_with_timeout(&mut s, &["ctrl-n", "wait:Name"], drive_ms(timeout));

    let blank = quiesce_text(&mut s, "input-form blank");
    let pw: String = blank
        .lines()
        .find(|l| l.contains("stored in the keychain"))
        .expect("password placeholder")
        .into();
    assert!(!pw.contains('•'), "empty Password masks nothing (V2): {pw}");
    checkpoint(&mut s, &dir, "01-blank");

    support::drive_with_timeout(
        &mut s,
        &["enter", "type:Staging replica", "enter"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut s, "Staging replica", "name commits");

    support::drive_with_timeout(
        &mut s,
        &["enter", "end", "ctrl-u", "enter"],
        drive_ms(timeout),
    );
    poll_name_error(&mut s, true, 8, "empty name rejected (S1)");

    support::drive_with_timeout(
        &mut s,
        &["enter", "type:Staging replica", "enter"],
        drive_ms(timeout),
    );
    checkpoint(&mut s, &dir, "02-repair-prefind");
    poll_name_error(&mut s, false, 8, "valid name clears the error");
    checkpoint(&mut s, &dir, "02-validated");

    support::drive_with_timeout(&mut s, &["enter", "type:ZZZ", "escape"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    let reverted = live_text(&mut s);
    assert!(
        reverted.contains("Staging replica") && !reverted.contains("ZZZ"),
        "Esc reverts the field (N1)"
    );
    assert!(
        reverted.contains("New connection") || reverted.contains("Save"),
        "Esc never closes the form (N1)"
    );

    tab_to_cursor(&mut s, "stored in the keychain", 20, "input-form password");
    support::drive_with_timeout(&mut s, &["enter", "type:abc"], drive_ms(timeout));
    waits::wait_state(&mut s, "•••", "password masks per grapheme (N2)");
    let masked = live_text(&mut s);
    assert!(
        !masked.contains("stored in the keychain"),
        "typing displaces the placeholder"
    );
    checkpoint(&mut s, &dir, "03-masked");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(300));

    let (hrow, hcol) = find_pos(&mut s, "localhost", timeout);
    s.click(hcol + 1, hrow).expect("input-form host click");
    std::thread::sleep(Duration::from_millis(400));
    support::drive_with_timeout(&mut s, &["type:X"], drive_ms(timeout));
    waits::wait_state(&mut s, "lXocalhost", "edit lands at the pointer (A2)");
    let pointed = live_text(&mut s);
    assert!(
        pointed
            .lines()
            .any(|l| l.contains('▎') && l.contains("lXocalhost")),
        "the click focused Host first (A2)"
    );
    checkpoint(&mut s, &dir, "04-pointer");

    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(300));
    let routed = live_text(&mut s);
    assert!(
        routed
            .lines()
            .any(|l| l.contains('▎') && l.contains("5432")),
        "CommittedTab advances focus (A1)"
    );
    support::press_step(&s, "ctrl-s");
    waits::wait_state(&mut s, "Connection saved", "form submits (A1)");
    let saved = quiesce_text(&mut s, "input-form saved");
    assert!(
        saved.contains("Staging replica"),
        "the tree lists the new connection"
    );
    checkpoint(&mut s, &dir, "05-saved");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_form_connection_full() {
    let case = t2_case_conn("form-connection-full");
    let dir = t2_dir("t2_form_connection_full");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    support::drive_with_timeout(&mut s, &["ctrl-n", "wait:Name"], drive_ms(timeout));
    support::drive_with_timeout(
        &mut s,
        &["enter", "type:Probe conn", "enter"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut s, "Probe conn", "name commits");

    tab_to_cursor(&mut s, "5432", 20, "form-conn port");
    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(300));
    let ported = live_text(&mut s);
    let labels: String = ported
        .lines()
        .find(|l| l.contains("Host") && l.contains("Port"))
        .expect("form labels")
        .into();
    let portcol =
        unicode_width::UnicodeWidthStr::width(&labels[..labels.find("Port").expect("label")]);
    let curline: String = ported
        .lines()
        .find(|l| l.contains('▎') && l.contains("5432"))
        .expect("port cursor")
        .into();
    let curcol =
        unicode_width::UnicodeWidthStr::width(&curline[..curline.find('▎').expect("cursor")]);
    assert!(
        curcol + 4 >= portcol,
        "Port owns the focus, not Host: {curline}"
    );
    support::drive_with_timeout(
        &mut s,
        &["enter", "end", "ctrl-u", "type:99999", "enter"],
        drive_ms(timeout),
    );
    checkpoint(&mut s, &dir, "01-bad-port-prefind");
    waits::wait_state(&mut s, "Port must", "port rejects 99999 (V1)");
    checkpoint(&mut s, &dir, "01-bad-port-shown");
    let bad_port = live_text(&mut s);

    assert!(
        bad_port.contains("Port must…"),
        "range message shows, field-clipped (V1)"
    );
    assert!(
        bad_port
            .lines()
            .any(|l| l.contains("99999") && l.contains('!')),
        "the field carries `!` (V1)"
    );
    checkpoint(&mut s, &dir, "01-bad-port");

    support::press_step(&s, "ctrl-s");
    std::thread::sleep(Duration::from_millis(800));
    let blocked = live_text(&mut s);
    assert!(
        blocked.contains("Port must…"),
        "the error stays visible (A1)"
    );
    assert!(
        !blocked.contains("Connection saved") && blocked.contains("Save"),
        "invalid Save never persists (A1)"
    );
    assert_eq!(
        blocked.matches("Probe conn").count(),
        1,
        "unvalidated name stays out of the tree (N1)"
    );

    support::drive_with_timeout(
        &mut s,
        &["enter", "end", "ctrl-u", "enter"],
        drive_ms(timeout),
    );
    wait_absent(&mut s, "Port must", 8, "empty port validates (S2)");

    click_text(&mut s, "Probe conn", 1, timeout, "form-conn name");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(
        &mut s,
        &["enter", "end", "ctrl-u", "enter"],
        drive_ms(timeout),
    );
    checkpoint(&mut s, &dir, "02-name-cleared-prefind");
    poll_name_error(&mut s, true, 8, "blank name rejected (N2)");
    support::press_step(&s, "ctrl-s");
    std::thread::sleep(Duration::from_millis(800));
    let blocked_name = live_text(&mut s);
    assert!(
        name_field_error(&blocked_name) && !blocked_name.contains("Connection saved"),
        "name alone gates the save (S1)"
    );
    support::drive_with_timeout(
        &mut s,
        &["enter", "type:Probe conn", "enter"],
        drive_ms(timeout),
    );
    poll_name_error(&mut s, false, 8, "name repaired");
    checkpoint(&mut s, &dir, "02-valid");

    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "SSH host", "advanced tab opens");
    let advanced = quiesce_text(&mut s, "form-conn advanced");
    for needle in [
        "SSH host",
        "SSH user",
        "Test connection",
        "Cancel",
        "Save",
        "Save & connect",
    ] {
        assert!(advanced.contains(needle), "advanced shows `{needle}` (V2)");
    }
    checkpoint(&mut s, &dir, "03-advanced");

    tab_to_cursor(&mut s, "Test connection", 25, "form-conn test");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "✓ Connected", "test connects (A2)");
    checkpoint(&mut s, &dir, "04-tested-prefind");
    let tested = quiesce_text(&mut s, "form-conn tested");
    assert!(
        tested.contains("SSH host") && tested.contains("Save & connect"),
        "Test leaves the form open (A2)"
    );

    assert!(
        !explorer_half(&tested).contains("Probe conn"),
        "Test persists nothing (A2)"
    );
    checkpoint(&mut s, &dir, "04-tested");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_dialog_ack_full() {
    let case = t2_case_wb("dialog-ack-full");
    let dir = t2_dir("t2_dialog_ack_full");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::drive_with_timeout(
        &mut s,
        &["tab", "i", "type:UPDATE orders"],
        drive_ms(timeout),
    );
    for line in [
        "SET status = 'paid',",
        "total_amount = 0,",
        "currency = 'USD',",
        "notes = 'x',",
        "is_gift = false,",
        "WHERE id = '00000000-0000-0000-0000-000000000000'",
    ] {
        support::press_step(&s, "escape");
        std::thread::sleep(Duration::from_millis(300));
        if !live_text(&mut s)
            .lines()
            .last()
            .is_some_and(|l| l.contains("EDIT"))
        {
            support::press_step(&s, "i");
            std::thread::sleep(Duration::from_millis(200));
        }
        support::press_step(&s, "enter");
        std::thread::sleep(Duration::from_millis(200));
        support::drive_with_timeout(&mut s, &[&format!("type:{line}")], drive_ms(timeout));
    }
    assert!(
        live_text(&mut s).contains("ln 7/7"),
        "seven editor lines stand"
    );
    escape_to_idle(&mut s, "dialog-ack statement");
    support::drive_with_timeout(
        &mut s,
        &["ctrl-r", "wait:Type orders to confirm"],
        drive_ms(timeout),
    );

    checkpoint(&mut s, &dir, "01-gate-prefind");
    let gate = quiesce_text(&mut s, "dialog-ack gate");
    for needle in ["Action", "Target", "Safe Mode", "UPDATE orders", "… 1 more"] {
        assert!(gate.contains(needle), "gate shows `{needle}` (V1)");
    }

    let ack_idx = gate
        .lines()
        .position(|l| l.contains("Type orders to confirm"))
        .expect("ack label");
    let acts_idx = gate
        .lines()
        .position(|l| l.contains("Cancel") && l.contains("Execute"))
        .expect("action row");
    assert!(ack_idx < acts_idx, "ack field above the actions (V2)");
    assert!(
        gate.lines()
            .any(|l| l.contains('▎') && !l.contains("Cancel")),
        "focus starts outside the actions (S2)"
    );
    assert!(
        !gate.lines().nth(acts_idx).is_some_and(|l| l.contains('▎')),
        "neither action owns the focus (S2)"
    );
    checkpoint(&mut s, &dir, "01-gate");

    support::drive_with_timeout(&mut s, &["enter", "type:yn"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    let typed = live_text(&mut s);
    assert!(
        typed.contains("Type orders to confirm") && !typed.contains("affected"),
        "y/n never confirm the gate (N2)"
    );

    support::drive_with_timeout(
        &mut s,
        &["ctrl-l", "type:000000", "enter"],
        drive_ms(timeout),
    );
    std::thread::sleep(Duration::from_millis(400));
    let wrong = live_text(&mut s);
    assert!(
        wrong
            .lines()
            .any(|l| l.contains('▎') && l.contains("Cancel")),
        "Enter only advances focus (A1)"
    );
    support::press_step(&s, "right");
    std::thread::sleep(Duration::from_millis(400));
    let stayed = live_text(&mut s);
    assert!(
        stayed
            .lines()
            .any(|l| l.contains('▎') && l.contains("Cancel")),
        "unarmed Right never reaches Execute (N1)"
    );
    assert!(
        stayed.contains("Type orders to confirm"),
        "the wrong token executes nothing (N1)"
    );
    checkpoint(&mut s, &dir, "02-unarmed");

    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s)
            .lines()
            .any(|l| l.contains('▎') && !l.contains("Cancel") && !l.contains("Execute")),
        "BackTab returns to the ack input"
    );
    support::drive_with_timeout(
        &mut s,
        &["enter", "ctrl-l", "type:or ders", "enter"],
        drive_ms(timeout),
    );
    std::thread::sleep(Duration::from_millis(400));
    support::press_step(&s, "right");
    std::thread::sleep(Duration::from_millis(400));
    let spaced = live_text(&mut s);
    assert!(
        spaced
            .lines()
            .any(|l| l.contains('▎') && l.contains("Cancel")),
        "spaced token never arms (N1)"
    );

    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(
        &mut s,
        &["enter", "ctrl-l", "type:orders", "enter"],
        drive_ms(timeout),
    );
    std::thread::sleep(Duration::from_millis(400));
    let armed = live_text(&mut s);
    assert!(
        armed
            .lines()
            .any(|l| l.contains('▎') && l.contains("Cancel")),
        "arming Enter still only advances (A1)"
    );
    assert!(
        armed.contains("Type orders to confirm") && !armed.contains("affected"),
        "advancing never executes (A1)"
    );
    support::press_step(&s, "right");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut s)
            .lines()
            .any(|l| l.contains('▎') && l.contains("Execute")),
        "armed Right reaches Execute (S1)"
    );
    checkpoint(&mut s, &dir, "03-armed");
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(3000));
    checkpoint(&mut s, &dir, "04-executed-prefind");

    waits::wait_state(&mut s, "rows affected", "armed Execute runs");
    assert!(
        live_text(&mut s).contains("UPDATE orders ("),
        "the result names the executed statement"
    );
    wait_absent(
        &mut s,
        "Type orders to confirm",
        8,
        "execution closes the gate",
    );
    checkpoint(&mut s, &dir, "04-executed");

    let case_b = t2_case_wb("dialog-ack-short");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive_with_timeout(
        &mut b,
        &[
            "tab",
            "i",
            "type:UPDATE orders SET status = 'paid' WHERE id = 'x'",
        ],
        drive_ms(timeout),
    );
    escape_to_idle(&mut b, "dialog-ack short statement");
    support::drive_with_timeout(
        &mut b,
        &["ctrl-r", "wait:Type orders to confirm"],
        drive_ms(timeout),
    );
    let facts = [
        "Action",
        "Target",
        "Scope",
        "Risk",
        "Reversible",
        "Safe Mode",
    ];
    let tall = facts
        .iter()
        .filter(|f| live_text(&mut b).contains(**f))
        .count();
    b.resize(120, 20).expect("resize short");
    for _ in 0..40 {
        std::thread::sleep(Duration::from_millis(100));
        if b.inner.observe_now().expect("geometry").screen.rows() == 20 {
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(800));
    let short = live_text(&mut b);
    assert!(
        short.contains("Type orders to confirm")
            && short.contains("Cancel")
            && short.contains("Execute"),
        "ack and actions survive the short screen (V2)"
    );
    let clipped = facts.iter().filter(|f| short.contains(**f)).count();
    assert!(
        clipped < tall,
        "only the facts region clips (V2): {tall} -> {clipped}"
    );
    checkpoint(&mut b, &dir, "05-short");

    let case_c = t2_case_wb("dialog-ack-plain");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    support::press_step(&c, "ctrl-l");
    waits::wait_state(&mut c, "Safe Mode · this connection", "picker opens");
    support::drive_with_timeout(&mut c, &["up", "up", "enter"], drive_ms(timeout));
    wait_absent(&mut c, "Safe Mode · this connection", 8, "picker applies");
    assert!(live_text(&mut c).contains("alert"), "Alert level applies");
    support::drive_with_timeout(
        &mut c,
        &[
            "tab",
            "i",
            "type:UPDATE orders SET status = 'paid' WHERE id = 'x'",
        ],
        drive_ms(timeout),
    );
    escape_to_idle(&mut c, "dialog-ack plain statement");
    support::drive_with_timeout(
        &mut c,
        &["ctrl-r", "wait:Execute write query?"],
        drive_ms(timeout),
    );
    let plain = quiesce_text(&mut c, "dialog-ack plain gate");
    assert!(
        !plain.contains("to confirm"),
        "Alert gates carry no ack input (S2)"
    );
    assert!(
        plain
            .lines()
            .any(|l| l.contains('▎') && l.contains("Execute")),
        "plain writes focus Execute (S2)"
    );
    support::press_step(&c, "escape");
    waits::wait_gone(&mut c, "Execute write query?", "plain gate cancels");
    checkpoint(&mut c, &dir, "06-plain");

    let case_d = t2_case_wb("dialog-ack-close");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::drive_with_timeout(&mut d, &["ctrl-t", "wait:Query 2"], drive_ms(timeout));
    tab_to_footer(&mut d, "Alt+R Run all", 8, "dialog-ack focus editor");
    support::drive_with_timeout(&mut d, &["i", "type:x"], drive_ms(timeout));
    escape_to_idle(&mut d, "dialog-ack dirty tab");
    tab_to_footer(&mut d, "← → Switch", 8, "dialog-ack focus strip");
    support::press_step(&d, "x");
    waits::wait_state(
        &mut d,
        "Close tab with unsaved work?",
        "close confirm opens",
    );
    let confirm = live_text(&mut d);
    let acts: String = confirm
        .lines()
        .find(|l| l.contains("Cancel") && l.contains("Close anyway"))
        .expect("destructive action row")
        .into();
    assert!(
        acts.find("Cancel") < acts.find("Close anyway"),
        "Cancel stands first (A2): {acts}"
    );
    assert!(
        confirm
            .lines()
            .any(|l| l.contains('▎') && l.contains("Cancel")),
        "destructive gates focus Cancel (S2)"
    );
    support::press_step(&d, "escape");
    waits::wait_gone(
        &mut d,
        "Close tab with unsaved work?",
        "Esc lands on Cancel (A2)",
    );
    assert!(
        strip_line(&live_text(&mut d)).contains("Query 2"),
        "the tab survives the cancelled close (A2)"
    );
    checkpoint(&mut d, &dir, "07-close");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_help_viewer_full() {
    let case = t2_case_wb("help-viewer-full");
    let dir = t2_dir("t2_help_viewer_full");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_open_orders(&mut s, timeout);
    support::press_step(&s, "?");
    waits::wait_state(&mut s, "Keyboard", "help opens");

    let open = quiesce_text(&mut s, "help open");
    let border: String = open
        .lines()
        .skip(5)
        .find(|l| {
            l.contains('╭') && l.contains('╮') && {
                let a = l.find('╭').expect("open");
                let b = l.find('╮').expect("close");
                (2 + l[a + '╭'.len_utf8()..b].chars().count()) >= 70
                    && (2 + l[a + '╭'.len_utf8()..b].chars().count()) <= 85
            }
        })
        .expect("help border")
        .into();
    let top = border.find('╭').expect("open");
    let end = border.find('╮').expect("close");
    assert_eq!(
        2 + border[top + '╭'.len_utf8()..end].chars().count(),
        78,
        "help frame is 78 wide (V1)"
    );

    let help_modal = modal_region(&open, "Keyboard").join("\n");
    assert!(
        help_modal.contains("Close") && !help_modal.contains("Cancel"),
        "a single Close action, Cancel stripped (V1/S1)"
    );
    assert!(
        open.lines().any(|l| l.contains('▎') && l.contains("Close")),
        "initial focus sits on Close (S1/S2)"
    );

    let region: Vec<&str> = open
        .lines()
        .skip_while(|l| !l.contains("Keyboard"))
        .take_while(|l| !l.contains("Close"))
        .collect();
    for chord in [
        "Ctrl+O", "Ctrl+R", "Ctrl+T", "Ctrl+G", "Ctrl+S", "Ctrl+D", "Ctrl+Y", "Alt+D",
    ] {
        assert!(
            region.iter().any(|l| l.contains(chord)),
            "chord table names `{chord}` (V2)"
        );
    }
    assert!(
        region.iter().any(|l| l.contains(" q ")),
        "chord table ends at q (V2)"
    );
    checkpoint(&mut s, &dir, "01-open");

    for (i, key) in ["escape", "enter", "space", "n"].iter().enumerate() {
        support::press_step(&s, key);
        waits::wait_gone(&mut s, "Keyboard", &format!("{key} closes help (A1)"));

        let mut restored = false;
        for _ in 0..30 {
            if live_text(&mut s)
                .lines()
                .last()
                .is_some_and(|l| l.contains("↑↓←→ Cell"))
            {
                restored = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(restored, "{key} restores the grid focus (A2)");
        if i < 3 {
            support::press_step(&s, "?");
            waits::wait_state(&mut s, "Keyboard", "help reopens");
        }
    }
    checkpoint(&mut s, &dir, "02-closes");

    support::press_step(&s, "?");
    waits::wait_state(&mut s, "Keyboard", "help reopens for N1");
    let before_y = quiesce_text(&mut s, "help before y");
    support::press_step(&s, "y");
    std::thread::sleep(Duration::from_millis(500));
    let after_y = live_text(&mut s);
    assert!(
        after_y.contains("Keyboard")
            && after_y
                .lines()
                .any(|l| l.contains('▎') && l.contains("Close")),
        "y leaves help open and focused (N1)"
    );
    assert_eq!(before_y, after_y, "y changes nothing at all (N1)");

    support::press_step(&s, "left");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(
        before_y,
        live_text(&mut s),
        "Left finds no second action (N2)"
    );
    support::press_step(&s, "right");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(
        before_y,
        live_text(&mut s),
        "Right finds no second action (N2)"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Keyboard", "help closes at the end");
    checkpoint(&mut s, &dir, "03-negatives");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_tree_connection_filter() {
    let case = t2_case_conn("tree-connection-filter");
    let dir = t2_dir("t2_tree_connection_filter");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    support::press_step(&s, "/");
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut s, &["type:s"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    let one = live_text(&mut s);
    assert!(
        one.contains("Staging") && !one.contains("Production"),
        "first keystroke already refilters (A2)"
    );
    support::drive_with_timeout(&mut s, &["type:tag"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    let narrowed = live_text(&mut s);
    assert!(
        narrowed.contains("▾ Acme") && narrowed.contains("◇ Staging"),
        "match ancestors stay visible and open (V1)"
    );
    assert_eq!(
        narrowed.matches('◇').count(),
        1,
        "only the match keeps its marker (V1)"
    );

    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut s).contains("◇ Staging"),
        "committing keeps the filter (A2)"
    );
    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(400));
    checkpoint(&mut s, &dir, "01-filtered-prefind");

    let mut row0 = false;
    for _ in 0..50 {
        if live_text(&mut s)
            .lines()
            .any(|l| l.contains('▎') && l.contains("Acme"))
        {
            row0 = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(row0, "filter reset the cursor to row 0 (S1)");
    checkpoint(&mut s, &dir, "01-filtered");

    let case_b = t2_case_conn("tree-filter-empty");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::press_step(&b, "/");
    std::thread::sleep(Duration::from_millis(300));
    support::drive_with_timeout(&mut b, &["type:zzz-no-match"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    let zero = live_text(&mut b);
    assert!(
        !zero.contains("Production") && !zero.contains("Staging") && !zero.contains("Acme"),
        "nothing matches, zero rows (N1)"
    );
    assert!(
        zero.contains("zzz-no-match"),
        "the app stays responsive (N1)"
    );
    support::press_step(&b, "escape");
    waits::wait_state(&mut b, "Production", "Esc restores the rows (N1)");
    checkpoint(&mut b, &dir, "02-empty");

    let case_c = t2_case_wb("tree-lazy-load");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    assert!(
        live_text(&mut c).contains("▸ S analytics"),
        "lazy nodes wait collapsed (V2)"
    );
    support::press_step(&c, "home");
    std::thread::sleep(Duration::from_millis(200));

    down_to(&mut c, "analytics", 25, "tree-lazy analytics");
    support::press_step(&c, "right");
    let mut busy = false;
    for _ in 0..60 {
        std::thread::sleep(Duration::from_millis(50));
        if live_text(&mut c)
            .lines()
            .find(|l| l.contains("analytics"))
            .is_some_and(|l| l.chars().any(|ch| SPIN.contains(ch)))
        {
            busy = true;
            break;
        }
    }
    assert!(busy, "expanding marks the node busy (A1/V2)");
    waits::wait_state(&mut c, "▾ S analytics", "children arrive");
    let grown = live_text(&mut c);
    let grown_lines: Vec<&str> = grown.lines().collect();
    let arow = grown_lines
        .iter()
        .position(|l| l.contains("▾ S analytics"))
        .expect("analytics row");
    assert!(
        grown_lines[arow + 1..(arow + 7).min(grown_lines.len())]
            .iter()
            .any(|l| l.contains("Tables")),
        "analytics fetched its own Tables group (A1)"
    );

    up_to(&mut c, "analytics", 25, "tree-lazy revisit");
    support::press_step(&c, "left");
    waits::wait_state(&mut c, "▸ S analytics", "analytics collapses");
    down_to(&mut c, "audit", 25, "tree-lazy park on audit");
    support::press_step(&c, "*");
    waits::wait_state(&mut c, "▾ S analytics", "expand-all re-expands");
    assert!(
        live_text(&mut c)
            .lines()
            .any(|l| l.contains('▎') && l.contains("audit")),
        "arrivals above never retarget the cursor (S2/N2)"
    );
    checkpoint(&mut c, &dir, "03-lazy");

    let case_d = t2_case_wb("tree-note-row");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::press_step(&d, "home");
    std::thread::sleep(Duration::from_millis(200));
    down_to(&mut d, "analytics", 25, "tree-note analytics");
    support::press_step(&d, "right");
    waits::wait_state(&mut d, "▾ S analytics", "analytics expands");
    down_to(&mut d, "Tables", 25, "tree-note analytics tables");
    support::press_step(&d, "right");
    std::thread::sleep(Duration::from_millis(500));
    down_to(&mut d, "daily_revenue", 25, "tree-note table");
    support::press_step(&d, "right");
    std::thread::sleep(Duration::from_millis(500));
    down_to(&mut d, "Triggers", 25, "tree-note triggers");
    support::press_step(&d, "right");
    std::thread::sleep(Duration::from_millis(500));
    down_to(&mut d, "No triggers", 25, "tree-note row");
    let (nrow, ncol) = find_pos(&mut d, "No triggers", timeout);
    assert_eq!(
        fg_rgb(&live_frame(&mut d), ncol, nrow, "tree-note tone"),
        (128, 128, 128),
        "note rows render muted (V3)"
    );

    let noted = no_footer(&quiesce_text(&mut d, "tree-note settled"));
    support::press_step(&d, "space");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        noted,
        no_footer(&quiesce_text(&mut d, "tree-note space")),
        "Space never selects a note (V3)"
    );

    down_to(&mut d, "audit", 25, "tree-note audit");
    support::press_step(&d, "space");
    waits::wait_state(&mut d, "▾ S audit", "Space expands plain nodes");
    checkpoint(&mut d, &dir, "04-note");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_picker_switcher_scopes() {
    let case = t2_case_wb("picker-switcher-scopes");
    let dir = t2_dir("t2_picker_switcher_scopes");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher opens");

    assert!(
        modal_region(&live_text(&mut s), "Open Quickly")
            .iter()
            .any(|l| l.contains("All · Tab scope")),
        "switcher labels its scope (V1)"
    );
    for (query, needle) in [
        ("orders", "T orders"),
        ("public", "S public"),
        ("Query 1", "Query 1"),
    ] {
        support::drive_with_timeout(&mut s, &[&format!("type:{query}")], drive_ms(timeout));
        std::thread::sleep(Duration::from_millis(400));
        assert!(
            modal_region(&live_text(&mut s), "Open Quickly")
                .iter()
                .any(|l| l.contains(needle)),
            "All scope ranks `{needle}` (V1)"
        );
        for _ in 0..query.len() + 2 {
            support::press_step(&s, "backspace");
            std::thread::sleep(Duration::from_millis(60));
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    checkpoint(&mut s, &dir, "01-all");

    for (scope, query, present, absent) in [
        ("Tables", "orders", "T orders", "S public"),
        ("Schemas", "public", "S public", "T orders"),
        ("Queries", "Query 1", "Query 1", "T orders"),
    ] {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(400));
        assert!(
            modal_region(&live_text(&mut s), "Open Quickly")
                .iter()
                .any(|l| l.contains(&format!("{scope} · Tab scope"))),
            "scope advances to {scope} (S1)"
        );
        support::drive_with_timeout(&mut s, &[&format!("type:{query}")], drive_ms(timeout));
        std::thread::sleep(Duration::from_millis(400));
        let region = modal_region(&live_text(&mut s), "Open Quickly");
        assert!(
            region.iter().any(|l| l.contains(present)),
            "{scope} scope lists {present} (S1)"
        );
        assert!(
            !region.iter().any(|l| l.contains(absent)),
            "{scope} scope drops {absent} (S1)"
        );
        for _ in 0..query.len() + 2 {
            support::press_step(&s, "backspace");
            std::thread::sleep(Duration::from_millis(60));
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        modal_region(&live_text(&mut s), "Open Quickly")
            .iter()
            .any(|l| l.contains("All · Tab scope")),
        "scope wraps back to All (S1)"
    );
    checkpoint(&mut s, &dir, "02-scopes");

    support::drive_with_timeout(&mut s, &["tab", "tab"], drive_ms(timeout));
    waits::wait_state(&mut s, "Schemas · Tab scope", "back to Schemas");
    support::drive_with_timeout(&mut s, &["type:audit", "enter"], drive_ms(timeout));
    checkpoint(&mut s, &dir, "03-dispatch-prefind");

    wait_absent(&mut s, "Open Quickly", 8, "schema chosen closes");
    assert!(
        live_text(&mut s)
            .lines()
            .any(|l| l.contains("Explorer") && l.contains("audit")),
        "schema dispatch rebuilds the explorer (A1)"
    );

    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher reopens");

    let mut all_scope = false;
    for _ in 0..5 {
        if live_text(&mut s).contains("All · Tab scope") {
            all_scope = true;
            break;
        }
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(300));
    }
    assert!(all_scope, "scope back to All");
    support::drive_with_timeout(&mut s, &["type:orders", "enter"], drive_ms(timeout));
    waits::wait_state(&mut s, "public › orders", "table dispatch opens (A1)");

    support::drive_with_timeout(&mut s, &["ctrl-t", "wait:Query 2"], drive_ms(timeout));
    escape_to_idle(&mut s, "switcher new tab");
    support::press_step(&s, "[");
    std::thread::sleep(Duration::from_millis(300));
    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher reopens");
    support::drive_with_timeout(&mut s, &["type:Query 2", "enter"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        live_text(&mut s).contains("─ Query 2 ─"),
        "query dispatch switches tabs (A1)"
    );
    checkpoint(&mut s, &dir, "03-chosen");

    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher reopens");
    support::drive_with_timeout(&mut s, &["type:zzz-no-match", "enter"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        live_text(&mut s).contains("Open Quickly"),
        "a match-nothing query is not a command (N1)"
    );
    assert!(
        !strip_line(&live_text(&mut s)).contains("zzz"),
        "Submit opens no tab (N1)"
    );
    for _ in 0..13 {
        support::press_step(&s, "backspace");
        std::thread::sleep(Duration::from_millis(80));
    }
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut s).contains("Open Quickly"),
        "Back on an empty query keeps the picker open (N1)"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Open Quickly", "Cancelled closes (A1)");
    checkpoint(&mut s, &dir, "04-n1");

    let case_b = t2_case_wb("picker-tab-list");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive_with_timeout(&mut b, &["ctrl-t", "wait:Query 2"], drive_ms(timeout));
    tab_to_footer(&mut b, "Enter Open", 8, "tablist focus tree");
    drive_open_orders(&mut b, timeout);
    tab_to_footer(&mut b, "← → Switch", 8, "tablist focus strip");
    support::press_step(&b, "ctrl-g");
    waits::wait_state(&mut b, "Open tabs", "tab list opens");

    let mut list = modal_region(&live_text(&mut b), "Open tabs");
    for _ in 0..30 {
        if ["≡ Query 1", "≡ Query 2", "active"]
            .iter()
            .all(|n| list.iter().any(|l| l.contains(n)))
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
        list = modal_region(&live_text(&mut b), "Open tabs");
    }
    for needle in ["≡ Query 1", "≡ Query 2", "active"] {
        assert!(
            list.iter().any(|l| l.contains(needle)),
            "tab list shows `{needle}` (V2)"
        );
    }
    assert!(
        list.iter()
            .any(|l| l.contains('▎') && l.contains("orders") && l.contains("active")),
        "cursor and tag sit on the active tab (V2)"
    );

    support::drive_with_timeout(&mut b, &["type:Query"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    checkpoint(&mut b, &dir, "05-tablist-prefind");
    let filtered = modal_region(&live_text(&mut b), "Open tabs");

    assert!(
        !filtered.iter().any(|l| grid_half(l).contains("T orders")),
        "substring match drops the table tab (S2)"
    );
    for (label, index) in [("Query 1", "0"), ("Query 2", "1")] {
        let row: String = filtered
            .iter()
            .find(|l| l.contains(label))
            .unwrap_or_else(|| panic!("{label} row survives (S2)"))
            .into();
        assert!(
            row.split(label).nth(1).is_some_and(|d| d.contains(index)),
            "detail carries index {index} for {label} (S2): {row}"
        );
    }
    checkpoint(&mut b, &dir, "05-filtered");

    for _ in 0..5 {
        support::press_step(&b, "backspace");
        std::thread::sleep(Duration::from_millis(80));
    }
    std::thread::sleep(Duration::from_millis(400));
    for _ in 0..4 {
        if live_text(&mut b)
            .lines()
            .any(|l| l.contains('▎') && l.contains("Query 1"))
        {
            break;
        }
        support::press_step(&b, "up");
        std::thread::sleep(Duration::from_millis(200));
    }
    support::press_step(&b, "delete");
    waits::wait_gone(&mut b, "≡ Query 1", "Secondary closes Query 1 (A2)");
    assert!(
        live_text(&mut b).contains("Open tabs"),
        "the list reopens after Secondary (A2)"
    );
    assert!(
        !strip_line(&live_text(&mut b)).contains("Query 1"),
        "the strip drops the closed tab (A2)"
    );
    support::press_step(&b, "escape");
    waits::wait_gone(&mut b, "Open tabs", "tab list cancels");
    checkpoint(&mut b, &dir, "06-secondary");

    let case_c = t2_case_wb("picker-safe-mode");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    support::press_step(&c, "ctrl-l");
    waits::wait_state(&mut c, "Safe Mode · this connection", "picker opens");
    let levels = ["Silent", "Alert", "Safe Mode", "Read-Only"];
    let shown = live_text(&mut c);
    for level in levels {
        assert!(shown.contains(level), "safe-mode lists {level} (V2)");
    }
    assert!(
        shown
            .lines()
            .any(|l| l.contains('›') && l.contains("Safe Mode") && l.contains("current")),
        "› marks the current level (V2)"
    );
    let scoped = |text: &str| {
        text.lines()
            .filter(|l| {
                l.contains("Safe Mode")
                    || l.contains("Silent")
                    || l.contains("Alert")
                    || l.contains("Read-Only")
                    || l.contains("current")
                    || l.contains('›')
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let before = scoped(&live_text(&mut c));
    support::press_step(&c, "tab");
    std::thread::sleep(Duration::from_millis(500));

    assert_eq!(
        before,
        scoped(&live_text(&mut c)),
        "NextScope outside the switcher changes nothing (N2)"
    );
    support::press_step(&c, "escape");
    waits::wait_gone(&mut c, "Safe Mode · this connection", "picker cancels");
    checkpoint(&mut c, &dir, "07-safemode");
}

#[test]
#[ignore = "tablepro t2 check; run with --ignored"]
fn t2_complete_sql_popup() {
    let case = t2_case_wb("complete-sql-popup");
    let dir = t2_dir("t2_complete_sql_popup");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);

    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    support::drive_with_timeout(
        &mut s,
        &["tab", "i", "type:SELECT * FROM ord"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut s, "order_items", "completion opens");
    let popped = live_text(&mut s);
    let editor_idx = popped
        .lines()
        .position(|l| l.contains("SELECT * FROM ord"))
        .expect("editor line");
    let top_idx = popped
        .lines()
        .enumerate()
        .skip(editor_idx + 1)
        .find(|(_, l)| l.contains('╭'))
        .map(|(i, _)| i)
        .expect("popup top");
    assert_eq!(
        top_idx,
        editor_idx + 1,
        "popup anchors directly below the cursor line (V1)"
    );
    let top_line: String = popped.lines().nth(top_idx).expect("top").into();
    let left = unicode_width::UnicodeWidthStr::width(&top_line[..top_line.find('╭').expect("l")]);
    let right = unicode_width::UnicodeWidthStr::width(&top_line[..top_line.find('╮').expect("r")]);
    assert!(
        (24..=48).contains(&(right - left + 1)),
        "popup width clamps to 24..48 (V2): {}",
        right - left + 1
    );
    let col_n: usize = popped
        .lines()
        .find_map(|l| {
            l.find("· col ").map(|i| {
                l[i + "· col ".len()..]
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .unwrap_or(0)
            })
        })
        .expect("cursor column readout");
    let editor_line: String = popped.lines().nth(editor_idx).expect("editor").into();
    let prefix = editor_line.find("SELECT").expect("statement");
    let cursor_disp = unicode_width::UnicodeWidthStr::width(&editor_line[..prefix]) + col_n - 1;
    assert!(
        left <= cursor_disp && cursor_disp <= right,
        "popup spans the cursor column (V1): {left} <= {cursor_disp} <= {right}"
    );
    for needle in [
        "T orders",
        "T order_items",
        "V order_totals_by_day",
        "K ORDER BY",
    ] {
        assert!(
            popped.contains(needle),
            "popup shows kind glyphs (V1): {needle}"
        );
    }
    assert!(
        popped
            .lines()
            .any(|l| l.contains('▎') && l.contains("T orders")),
        "open resets the cursor to row 0 (S1)"
    );

    let (prow, pcol) = find_pos(&mut s, "order_items", timeout);
    let frame = live_frame(&mut s);
    for j in 0..3u16 {
        assert!(
            frame.get(pcol + j, prow).is_some_and(|c| c.mods.bold),
            "matched prefix is bold (V1)"
        );
    }
    for j in 3..11u16 {
        assert!(
            frame.get(pcol + j, prow).is_some_and(|c| !c.mods.bold),
            "unmatched suffix stays plain (V1)"
        );
    }
    let mut starts = vec![];
    let mut ends = vec![];
    let mut details = vec![];
    for label in ["T orders ", "T order_items ", "V order_totals_by_day "] {
        let row: String = popped
            .lines()
            .find(|l| l.contains(label))
            .unwrap_or_else(|| panic!("popup row {label}"))
            .into();
        let ds = row.find("public ·").expect("detail");
        let de = row.find(" rows │").expect("detail end") + " rows".len();
        starts.push(unicode_width::UnicodeWidthStr::width(&row[..ds]));
        ends.push(unicode_width::UnicodeWidthStr::width(&row[..de]));
        details.push(row[ds..de].to_string());
    }

    assert_eq!(
        (starts[0], starts[1]),
        (starts[1], starts[2]),
        "details share one start column (V1): {starts:?}"
    );
    assert_eq!(
        ends[0], ends[1],
        "details share one end column (V1): {ends:?}"
    );
    assert_eq!(
        ends[1], ends[2],
        "details share one end column (V1): {ends:?}"
    );
    assert!(
        details[0] != details[1] && details[1] != details[2],
        "aligned details differ, so the alignment is real: {details:?}"
    );
    checkpoint(&mut s, &dir, "01-anchored");

    let case_b = t2_case_wb("complete-accept");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive_with_timeout(
        &mut b,
        &["tab", "i", "type:SELECT * FROM ord"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut b, "order_items", "completion opens");
    support::press_step(&b, "tab");
    std::thread::sleep(Duration::from_millis(500));
    let accepted = live_text(&mut b);
    assert!(
        accepted.contains("FROM orders") && !accepted.contains("ordorders"),
        "accept replaces replace_len bytes (A1)"
    );
    assert!(
        !accepted.contains("K ORDER BY"),
        "accepting closes the popup (A1)"
    );
    checkpoint(&mut b, &dir, "02-accepted");

    let case_c = t2_case_wb("complete-columns");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    support::drive_with_timeout(
        &mut c,
        &[
            "tab",
            "i",
            "type:x FROM orders o",
            "home",
            "type:SELECT o.",
            "delete",
        ],
        drive_ms(timeout),
    );
    waits::wait_state(&mut c, "is_gift", "columns popup opens");
    let cols = live_text(&mut c);
    assert!(
        cols.lines().any(|l| l.contains('┃')),
        "overflow rows show a scrollbar (V2)"
    );
    let ceditor = cols
        .lines()
        .position(|l| l.contains("SELECT o."))
        .expect("editor");
    let ctop: String = cols
        .lines()
        .skip(ceditor + 1)
        .find(|l| l.contains('╭'))
        .expect("popup top")
        .into();
    let cleft = ctop.find('╭').expect("l");
    let cright = ctop.find('╮').expect("r");
    let cspan = unicode_width::UnicodeWidthStr::width(&ctop[..cright])
        - unicode_width::UnicodeWidthStr::width(&ctop[..cleft])
        + 1;
    assert!(
        (24..=48).contains(&cspan),
        "columns popup clamps to 24..48 (V2): {cspan}"
    );

    let top_row = |text: &str| {
        let lines: Vec<&str> = text.lines().collect();
        let e = lines
            .iter()
            .position(|l| l.contains("SELECT o."))
            .expect("editor");
        lines[e + 2].to_string()
    };
    let pre_wheel = top_row(&live_text(&mut c));
    assert!(pre_wheel.contains("C id"), "cursor row starts on top");
    let (wrow, wcol) = find_pos(&mut c, "is_gift", timeout);
    wheel_at(&mut c, wcol, wrow, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(300));
    let scrolled = top_row(&live_text(&mut c));
    assert_ne!(
        pre_wheel, scrolled,
        "the wheel scrolls the popup viewport (S2)"
    );
    std::thread::sleep(Duration::from_millis(800));
    assert_eq!(
        scrolled,
        top_row(&live_text(&mut c)),
        "render never pulls the viewport back (S2)"
    );
    checkpoint(&mut c, &dir, "03-scrolled");

    support::press_step(&c, "escape");
    let mut dismissed = false;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        let t = live_text(&mut c);
        let lines: Vec<&str> = t.lines().collect();
        if let Some(e) = lines.iter().position(|l| l.contains("SELECT o."))
            && !lines[e + 1].contains('╭')
        {
            dismissed = true;
            break;
        }
    }
    assert!(dismissed, "popup dismisses");
    support::drive_with_timeout(&mut c, &["backspace", "type:."], drive_ms(timeout));
    waits::wait_state(&mut c, "is_gift", "popup reopens");

    support::drive_with_timeout(&mut c, &["type:n"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("o.n"),
        "plain n types into the editor (N1)"
    );
    support::drive_with_timeout(
        &mut c,
        &["backspace", "backspace", "type:."],
        drive_ms(timeout),
    );
    waits::wait_state(&mut c, "is_gift", "popup reopens");
    support::drive_with_timeout(&mut c, &["type:p"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("o.p"),
        "plain p types into the editor (N1)"
    );
    support::drive_with_timeout(
        &mut c,
        &["backspace", "backspace", "type:."],
        drive_ms(timeout),
    );
    waits::wait_state(&mut c, "is_gift", "popup reopens");

    if live_text(&mut c).contains("Open Quickly") {
        support::press_step(&c, "escape");
        wait_absent(&mut c, "Open Quickly", 8, "stray switcher closes");
    }
    checkpoint(&mut c, &dir, "03-cursor-prefind");
    let cursor_row = |c: &mut Session| {
        for _ in 0..50 {
            if let Some(row) = live_text(c)
                .lines()
                .find(|l| l.contains("▎C "))
                .map(str::to_string)
            {
                return row;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("cursor row never settled:\n{}", live_text(c))
    };
    let cur_before = cursor_row(&mut c);
    support::press_step(&c, "down");
    std::thread::sleep(Duration::from_millis(400));
    let cur_moved = cursor_row(&mut c);
    assert_ne!(cur_before, cur_moved, "Down moves the popup cursor (N1)");
    assert!(
        live_text(&mut c).contains("o. FROM"),
        "Down types nothing (N1)"
    );
    support::press_step(&c, "up");
    std::thread::sleep(Duration::from_millis(400));
    let cur_back = cursor_row(&mut c);
    assert_eq!(cur_before, cur_back, "Up moves back (N1)");

    support::press_step(&c, "down");
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&c, "enter");
    std::thread::sleep(Duration::from_millis(500));
    let entered = live_text(&mut c);
    assert!(
        entered.contains("o.notes"),
        "Enter accepts the cursor row (A1)"
    );
    assert!(
        !entered.contains("is_gift"),
        "accepting closes the popup (A1)"
    );
    checkpoint(&mut c, &dir, "04-columns");

    let case_d = t2_case_wb("complete-function");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::drive_with_timeout(
        &mut d,
        &["tab", "i", "type:SELECT coale"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut d, "coalesce", "function popup opens");
    support::drive_with_timeout(&mut d, &["down", "down", "down", "down"], drive_ms(timeout));
    waits::wait_state(&mut d, "▎F coalesce", "cursor reaches the function");
    support::press_step(&d, "tab");
    std::thread::sleep(Duration::from_millis(500));
    let called = live_text(&mut d);
    assert!(
        called.contains("coalesce(") && called.contains(')'),
        "accepting a function inserts parens (A1)"
    );
    support::drive_with_timeout(&mut d, &["type:1"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut d).contains("coalesce(1)"),
        "the cursor lands inside the parens (A1)"
    );
    checkpoint(&mut d, &dir, "05-function");

    let case_e = t2_case_wb("complete-esc");
    write_provenance_named(&dir, &case_e, "provenance-e.txt");
    let mut e = support::spawn_boot(&case_e);
    support::drive_with_timeout(
        &mut e,
        &["tab", "i", "type:SELECT * FROM ord"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut e, "order_items", "completion opens");
    support::press_step(&e, "escape");
    waits::wait_gone(&mut e, "K ORDER BY", "Esc dismisses (A2)");
    assert!(
        live_text(&mut e).contains("SELECT * FROM ord"),
        "dismiss keeps the statement (A2)"
    );
    checkpoint(&mut e, &dir, "06-esc");

    let case_f = t2_case_wb("complete-clickout");
    write_provenance_named(&dir, &case_f, "provenance-f.txt");
    let mut f = support::spawn_boot(&case_f);
    support::drive_with_timeout(
        &mut f,
        &["tab", "i", "type:SELECT * FROM ord"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut f, "order_items", "completion opens");
    for (col, row) in [(25u16, 7u16), (5, 20), (60, 30)] {
        f.click(col, row).expect("complete click-outside");
        std::thread::sleep(Duration::from_millis(500));
    }
    let clicked_out = live_text(&mut f);
    assert!(
        clicked_out.contains("K ORDER BY"),
        "click-outside leaves the popup up (A2 deviation, noted)"
    );
    assert!(
        clicked_out.contains("SELECT * FROM ord"),
        "click-outside keeps the statement (A2)"
    );
    checkpoint(&mut f, &dir, "07-clickout");

    let case_g = t2_case_wb("complete-nomatch");
    write_provenance_named(&dir, &case_g, "provenance-g.txt");
    let mut g = support::spawn_boot(&case_g);
    support::drive_with_timeout(
        &mut g,
        &["tab", "i", "type:SELECT * FROM ord"],
        drive_ms(timeout),
    );
    waits::wait_state(&mut g, "order_items", "completion opens");
    support::drive_with_timeout(&mut g, &["type:zzz"], drive_ms(timeout));
    std::thread::sleep(Duration::from_millis(500));
    let nomatch = live_text(&mut g);
    assert!(
        nomatch.contains("ordzzz"),
        "the editor keeps the unmatched text (N2)"
    );
    assert!(
        !nomatch.contains("K ORDER BY"),
        "no empty box survives a match-nothing query (S1/N2)"
    );
    assert!(
        !nomatch
            .lines()
            .last()
            .is_some_and(|l| l.contains("Enter Accept")),
        "the completion footer leaves with the popup (N2)"
    );
    checkpoint(&mut g, &dir, "08-nomatch");
}
