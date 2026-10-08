//! TablePro pending-roots slice 8A-T2 executable checks (VB Phase-8af).
//!
//! One ignored PTY test per T2 registry row (14 rows), over the real
//! `tablepro` binary built from this worktree's verified VB sources
//! (`env!("CARGO_BIN_EXE_tablepro")`, the harness `[[bin]]` compiled from
//! `../../src/bin/tablepro/main.rs`). Each test drives the row's specified
//! inputs and asserts its visual (V), state (S), action (A), and negative
//! (N) checks in live-PTY executable form:
//!
//! - V: live needles plus same-line coexistence for grids, strips, editors,
//!   dialogs, and overlay action rows.
//! - S: live labels (cursor/detail/focus/hints/footer rows) plus boot-vs-final
//!   and pre/post comparisons captured from the same session.
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (or boundary refusal) is observed; `Changed`/`Consumed`/
//!   `Ignored` outcomes are proven by their screen correlates (moved,
//!   stopped-at-end, byte-identical), since PTY cannot see the enum.
//! - N: absence assertions, each paired with a presence proof in the same
//!   test (boot frame, second session, or pre/post transition) so no
//!   absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 14 roots already have
//! approved frames gated cell-exact by the ported matrices in `tablepro.rs`,
//! `pointer.rs`, and `audit.rs` (plan §Reconciliation: "rows plus checks,
//! not recaptures"), so every case here uses owned (dynamic) names and stays
//! out of the `snapshots/` inventory. No isolated-component captures: every
//! row's `requires` set (key-injection, glyph-capture, tick-control,
//! geometry-probe) is PTY-level, and every assertion has a live-PTY
//! executable form.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a boot
//! comparison, and `provenance.txt` (binary path, size, mtime, argv).
//! Typed input is synthetic and in-memory only (simulation data); the demo
//! database never touches disk (no connection persistence exists).
//!
//! Row → test map (registry id → `t2_*` test):
//!
//! - GRID-SORT-004 → [`t2_grid_local_sort`]
//! - GRID-CELL-005 → [`t2_grid_cell_lifecycle`]
//! - GRID-RESULTS-006 → [`t2_grid_results_view`]
//! - CODE-SQL-003 → [`t2_code_sql_editor`]
//! - SELECT-FILTER-002 → [`t2_select_filter_builder`]
//! - CHIP-FILTER-002 → [`t2_chip_filter_bar`]
//! - TABS-STATES-002 → [`t2_tabs_workbench_states`]
//! - TI-FORM-010 → [`t2_input_form_field`]
//! - FORM-CONN-004 → [`t2_form_connection_full`]
//! - DIALOG-ACK-003 → [`t2_dialog_ack_full`]
//! - HELP-TABLEPRO-003 → [`t2_help_viewer_full`]
//! - TREE-FILTER-002 → [`t2_tree_connection_filter`]
//! - PICKER-SWITCHER-003 → [`t2_picker_switcher_scopes`]
//! - COMPLETE-SQL-001 → [`t2_complete_sql_popup`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::tui::{MouseButton, MouseMods, Session, Wheel};
use tuiscotti::{Color as TColor, Frame, Provenance, Rgb};

use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, TABLEPRO};

/// Boot needle on the connections screen: the Production tree row.
const BOOT_CONN: &str = "Production";
/// Boot needle once connected: the first query tab.
const BOOT_WB: &str = "Query 1";
/// Braille spinner glyphs (the busy indicator family).
const SPIN: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

/// Owned-name T2 case on the connections screen (120x40 truecolor).
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

/// Owned-name T2 case on the workbench (120x40 truecolor).
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

/// Unique scratch dir for one T2 test (created, never shared).
fn t2_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary path + identity: absolute path, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

/// [`write_provenance`] with an explicit filename, for multi-session tests
/// where each session's argv deserves its own record.
fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let meta = std::fs::metadata(case.bin).unwrap_or_else(|e| panic!("stat {}: {e}", case.bin));
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
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("t2 provenance ({file}): {body}");
}

/// Save the live screen text as `<name>.txt` in the test dir.
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

/// One fresh live text sample (no wait).
fn live_text(s: &mut Session) -> String {
    let obs = s
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

/// One fresh live frame with cell colors (no wait). Provenance is
/// informational only (excluded from digests).
fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    support::frame_from_screen(
        &screen,
        Provenance::now("tuiscotti-default", "t2-checks", vec![]),
    )
}

/// First occurrence of `needle` as `(row, col)`, waiting until it appears.
fn find_pos(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let obs = support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_find(screen, needle).is_some(),
    );
    support::screen_find(&obs.screen, needle).expect("wait passed with the needle on screen")
}

/// `notches` wheel steps at fixed `(col, row)`, paced like send steps.
fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

/// Parse the `Copied N chars` status count out of live text.
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

/// Resolve a cell foreground to `(r, g, b)`. Panics (fail-closed) on
/// non-RGB cells so tone assertions never pass on defaults.
fn fg_rgb(frame: &Frame, col: u16, row: u16, what: &str) -> (u8, u8, u8) {
    let cell = frame
        .get(col, row)
        .unwrap_or_else(|| panic!("{what}: no cell at ({col},{row})"));
    match cell.fg {
        TColor::Rgb(Rgb { r, g, b }) => (r, g, b),
        other => panic!("{what}: cell ({col},{row}) fg is not RGB: {other:?}"),
    }
}

/// Same for the background channel.
fn bg_rgb(frame: &Frame, col: u16, row: u16, what: &str) -> (u8, u8, u8) {
    let cell = frame
        .get(col, row)
        .unwrap_or_else(|| panic!("{what}: no cell at ({col},{row})"));
    match cell.bg {
        TColor::Rgb(Rgb { r, g, b }) => (r, g, b),
        other => panic!("{what}: cell ({col},{row}) bg is not RGB: {other:?}"),
    }
}

/// Relative luminance of an sRGB triple (0.0 dark … 1.0 bright).
fn lum(rgb: (u8, u8, u8)) -> f32 {
    0.2126 * f32::from(rgb.0) / 255.0
        + 0.7152 * f32::from(rgb.1) / 255.0
        + 0.0722 * f32::from(rgb.2) / 255.0
}

/// Sample until two consecutive screens are byte-identical (bounded):
/// the quiesce form for comparisons that must not catch a spinner mid-tick.
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

/// Press Esc while the footer shows EDIT (bounded). Never Esc's when idle,
/// so the connection-ladder (`Esc` un-focuses panels) is never triggered.
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

/// Press Down until the `▎` cursor line contains `needle` (bounded).
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

/// Press Tab until the `▎` cursor line contains `needle` (bounded).
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

/// Press Up until the `▎` cursor line contains `needle` (bounded).
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

/// Press Tab until the footer hint row contains `needle` (bounded).
/// Returns the stops consumed (positive proof the ring turned).
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

/// Click the first cell of `needle` shifted by `dcol` display columns.
fn click_text(s: &mut Session, needle: &str, dcol: i16, timeout: Duration, what: &str) {
    let (row, col) = find_pos(s, needle, timeout);
    let target = col.saturating_add_signed(dcol);
    s.click(MouseButton::Left, target, row, MouseMods::NONE)
        .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

/// The workbench tab strip: screen line 2 (`0`-based) on a 120x40 paint.
fn strip_line(text: &str) -> &str {
    text.lines().nth(2).unwrap_or("")
}

/// The grid-pane half of a full-width workbench line (right of the
/// `│ │` pane boundary), so explorer glyphs never pollute grid asserts.
fn grid_half(line: &str) -> &str {
    line.split("│ │").nth(1).unwrap_or(line)
}

/// The explorer-pane half (left of the boundary): for asserting what the
/// connection tree does or does not list.
fn explorer_half(text: &str) -> String {
    text.lines()
        .map(|l| l.split("│ │").next().unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Index of the grid header line via the early `order_number` column.
/// Callers press Home first: the status-column recipes hscroll the grid
/// (`cols 5–7 of 14`) and push early columns out of view. Panics with
/// the full screen so a missed header stays diagnosable.
fn grid_header_idx(text: &str, what: &str) -> usize {
    text.lines()
        .position(|l| l.contains("order_number"))
        .unwrap_or_else(|| panic!("{what}: no grid header on screen:\n{text}"))
}

/// Bounded poll for absence WITHOUT a presence requirement: for closes
/// that land synchronously during a drive (a presence-demanding wait
/// would fail vacuously after the fact).
fn wait_absent(s: &mut Session, needle: &str, secs: u64, what: &str) {
    for _ in 0..secs * 10 {
        if !live_text(s).contains(needle) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{what}: `{needle}` never left");
}

/// The demo engine's orders status values (db.rs `enum_col`).
const KNOWN_STATUSES: [&str; 6] = [
    "pending",
    "paid",
    "shipped",
    "delivered",
    "cancelled",
    "refunded",
];

/// First known status value on the grid half of `line`, if any.
fn row_status(line: &str) -> Option<&str> {
    let half = grid_half(line);
    KNOWN_STATUSES.into_iter().find(|s| half.contains(s))
}

/// Whether the Name field's own rows (label/field/error) show `Required`.
/// Whole-screen waits are useless here: the Database help line always
/// reads `Required for PostgreSQL`.
fn name_field_error(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().collect();
    let Some(i) = lines.iter().position(|l| l.contains("Name *")) else {
        return false;
    };
    lines[i..lines.len().min(i + 3)]
        .iter()
        .any(|l| l.contains("Required"))
}

/// Poll until the Name field's error equals `want` (bounded).
fn poll_name_error(s: &mut Session, want: bool, secs: u64, what: &str) {
    for _ in 0..secs * 10 {
        if name_field_error(&live_text(s)) == want {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{what}: Name error never became {want}");
}

/// Lines of the modal titled `title`: title row through its bottom border.
/// Scopes row assertions to the modal so background panes never pollute
/// absence checks.
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

/// Screen text without the footer line: ambient status flickers there
/// (`Connected to ...` vs transient blanks), so no-op proofs compare
/// panes, not the footer.
fn no_footer(text: &str) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    lines.pop();
    lines.join("\n")
}

/// Display column of `needle` within the strip line (wide-cell exact).
fn strip_col(text: &str, needle: &str, what: &str) -> u16 {
    let strip = strip_line(text);
    let off = strip
        .find(needle)
        .unwrap_or_else(|| panic!("{what}: `{needle}` not in strip"));
    unicode_width::UnicodeWidthStr::width(&strip[..off]) as u16
}

/// Click `needle` inside the strip (line 2), shifted by `dcol` columns.
fn strip_click(s: &mut Session, needle: &str, dcol: i16, what: &str) {
    let col = strip_col(&live_text(s), needle, what);
    s.click(
        MouseButton::Left,
        col.saturating_add_signed(dcol),
        2,
        MouseMods::NONE,
    )
    .unwrap_or_else(|e| panic!("{what}: click failed: {e:?}"));
    std::thread::sleep(Duration::from_millis(400));
}

/// Drive the explorer to the orders table: boot cursor is on Production,
/// five Downs reach orders, Enter opens the `T orders` tab.
fn drive_open_orders(s: &mut Session, timeout: Duration) {
    support::drive(
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
        timeout,
    );
}

/// Extend [`drive_open_orders`] with the filter-editor recipe: focus the
/// status cell, open the editor, commit `pending`, wait for the filtered set.
fn drive_filtered_orders(s: &mut Session, timeout: Duration) {
    drive_open_orders(s, timeout);
    support::drive(
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
        timeout,
    );
}

/// Run a PK-less 25-row results query (unattributable: no `id` in the
/// projection, so editable=false per sql.rs). Single-shot typing: no
/// newlines, so completion popups can only watch, never accept.
fn drive_readonly_query(s: &mut Session, timeout: Duration) {
    support::drive(
        s,
        &[
            "tab",
            "i",
            "type:SELECT status, total_amount FROM orders WHERE status = 'pending' ORDER BY created_at DESC LIMIT 25",
        ],
        timeout,
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
    support::drive(s, &["ctrl-r", "wait:25 rows"], timeout);
}

/// Run the canonical 25-row results query in the Query 1 editor (the
/// ported-matrix recipe), ending on the results grid.
fn drive_results_query(s: &mut Session, timeout: Duration) {
    support::drive(
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
        timeout,
    );
}

/// GRID-SORT-004: the three-step sort cycle on a table grid (`s`:
/// ascending → descending → cleared), both marker glyphs, the status
/// line, header-click sorting, and the JSON-column refusal.
#[test]
#[ignore]
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
    // Ascending: marker suffix, status line, grouped reorder. (The demo
    // engine re-randomizes rows on every owner reload and sorts the enum
    // alphabetically rather than in decl order, so reorder is proven
    // within one load: the lead status groups the first rows and flips
    // when the direction changes.)
    support::drive(
        &mut s,
        &["home", "right", "right", "right", "right"],
        timeout,
    );
    support::drive(&mut s, &["s", "wait:Sorted by status ascending"], timeout);
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
    // Descending: marker flips, the enum-last status leads instead.
    support::drive(&mut s, &["s", "wait:Sorted by status descending"], timeout);
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
    // Third press clears the sort entirely (the marker is gone in every
    // header; N2's stored-vs-display proof runs on the local-sort results
    // path in t2_grid_results_view, where no reload intervenes).
    support::drive(&mut s, &["s", "wait:Sort cleared"], timeout);
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
    // `S` clears directly from a sorted state.
    support::drive(&mut s, &["s", "wait:Sorted by status"], timeout);
    support::drive(&mut s, &["S", "wait:Sort cleared"], timeout);
    let direct = quiesce_text(&mut s, "grid-local-sort direct-clear");
    let direct_header: String = grid_half(direct.lines().nth(header_idx).expect("header")).into();
    assert!(
        !direct_header.contains('▴') && !direct_header.contains('▾'),
        "`S` clears the marker: {direct_header}"
    );
    checkpoint(&mut s, &dir, "03-cleared");
    // Filtering keeps the sort marker (V2): sort, then filter on the cell.
    support::drive(&mut s, &["s", "wait:Sorted by status"], timeout);
    support::drive(
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
        timeout,
    );
    // (No homing: the status column and its marker live in the
    // hscrolled viewport; the `▴` line IS the header here.)
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

    // Session B: header click sorts; the JSON column refuses the cycle.
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
    // JSON column: `s` is Consumed — no marker, no status line.
    support::drive(
        &mut b,
        &[
            "home", "right", "right", "right", "right", "right", "right", "right", "s",
        ],
        timeout,
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
    // Pairing presence: back on a sortable column `s` sorts immediately.
    support::drive(
        &mut b,
        &["home", "right", "right", "right", "right", "s"],
        timeout,
    );
    waits::wait_state(
        &mut b,
        "Sorted by status ascending",
        "sortable column sorts",
    );
    checkpoint(&mut b, &dir, "04-click");
}

/// GRID-CELL-005: dirty tint, horizontal scroll, row duplication, copy,
/// commit routing, fetch-more, and refresh on a table grid — plus the
/// read-only refusal (no editor on result grids).
#[test]
#[ignore]
fn t2_grid_cell_lifecycle() {
    let case = t2_case_wb("grid-cell-lifecycle");
    let dir = t2_dir("t2_grid_cell_lifecycle");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // The status column is enum-validated: free text can never commit
    // there. The numeric total_amount column (one right of status) takes
    // a known value instead.
    drive_open_orders(&mut s, timeout);
    support::drive(
        &mut s,
        &["home", "right", "right", "right", "right", "right"],
        timeout,
    );
    // Enter opens the cell editor (also the N2 pairing: editable grids
    // DO edit on Enter).
    support::drive(&mut s, &["enter"], timeout);
    waits::wait_state(&mut s, "Enter Commit", "cell editor opens");
    let editing = live_text(&mut s);
    let edit_footer: String = editing.lines().last().expect("footer").into();
    assert!(
        edit_footer.contains("EDIT") && edit_footer.contains("Esc Cancel"),
        "cell editing footer: {edit_footer}"
    );
    // Commit one dirty cell: value sticks, pending label counts it.
    support::drive(&mut s, &["end", "ctrl-u", "type:99991", "enter"], timeout);
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
    // V1: the dirty cell's tint differs from a committed cell's (every
    // other row is committed: only one cell is dirty).
    let (drow, dcol) = find_pos(&mut s, "99991", timeout);
    let frame = live_frame(&mut s);
    let dirty_bg = bg_rgb(&frame, dcol, drow, "grid-cell dirty tint");
    let committed_bg = bg_rgb(&frame, dcol, drow + 1, "grid-cell committed tint");
    assert_ne!(
        dirty_bg, committed_bg,
        "dirty tint differs from committed cells"
    );
    checkpoint(&mut s, &dir, "01-dirty");
    // A1: `y` copies the committed cell value (`99991`: 5 chars).
    support::drive(&mut s, &["y", "wait:Copied 5 chars"], timeout);
    // A1: Ctrl+S routes CommitRequested to the commit review dialog.
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
    // Dirty refresh asks to discard instead of reloading.
    support::press_step(&s, "r");
    waits::wait_state(&mut s, "Discard unsaved changes?", "dirty refresh guards");
    let guard = live_text(&mut s);
    assert!(
        guard.contains("1 pending change will be dropped"),
        "discard count matches the edit"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Discard unsaved changes?", "guard dismisses");

    // Session B: duplication, fetch-more, clean refresh on a fresh grid.
    let case_b = t2_case_wb("grid-cell-pages");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    drive_open_orders(&mut b, timeout);
    // S1: Alt+D appends a copy of the cursor row.
    support::press_step(&b, "alt-d");
    waits::wait_state(&mut b, "1 insert", "duplicate lands");
    let duped = quiesce_text(&mut b, "grid-cell duplicated");
    assert!(
        duped.contains("of 501 loaded"),
        "the copy appends a 501st loaded row"
    );
    checkpoint(&mut b, &dir, "03-duplicated");
    // A2: the more-row explains the demo cap instead of loading.
    support::drive(&mut b, &["ctrl-end", "down", "enter"], timeout);
    waits::wait_state(
        &mut b,
        "Fetch more: the demo engine caps results at 500 rows",
        "fetch-more message",
    );
    // A2: clean refresh reloads with a status line.
    support::drive(&mut b, &["F", "wait:Filters cleared"], timeout);
    support::press_step(&b, "r");
    waits::wait_state(&mut b, "Refreshed", "clean refresh");
    checkpoint(&mut b, &dir, "04-refreshed");

    // Session C: horizontal scroll keeps the cursor (V2/S2/N1).
    let case_c = t2_case_wb("grid-cell-hscroll");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    drive_open_orders(&mut c, timeout);
    let boot = quiesce_text(&mut c, "grid-cell hscroll boot");
    assert!(
        boot.contains("of 14"),
        "14 columns overflow the viewport at boot"
    );
    support::drive(&mut c, &["end", "wait:cols 13–14 of 14"], timeout);
    support::drive(&mut c, &["y", "wait:Copied "], timeout);
    let before = copied_count(&live_text(&mut c), "grid-cell pre-page copy");
    // Ctrl+Left pages the viewport; the polled label proves the state moved.
    support::press_step(&c, "ctrl-left");
    waits::wait_state(&mut c, "cols 11–13 of 14", "hscroll pages left");
    support::drive(&mut c, &["y", "wait:Copied "], timeout);
    let after = copied_count(&live_text(&mut c), "grid-cell post-page copy");
    assert_eq!(
        before, after,
        "paging never moves the cell cursor (N1): same value copies"
    );
    // (Pages overlap: right from `11–13` lands on `12–14`, not back on
    // the two-column `13–14` tail the End key shows.)
    support::press_step(&c, "ctrl-right");
    std::thread::sleep(Duration::from_millis(800));
    checkpoint(&mut c, &dir, "05-hscroll-right-prefind");
    waits::wait_state(&mut c, "cols 12–14 of 14", "hscroll pages right");
    support::drive(&mut c, &["y", "wait:Copied "], timeout);
    let back = copied_count(&live_text(&mut c), "grid-cell return copy");
    assert_eq!(before, back, "paging back keeps the cursor too");
    checkpoint(&mut c, &dir, "05-hscroll");

    // Session D: N2 — Enter on a read-only result grid never opens the
    // editor. (Readonly means unattributable here: the PK-less query
    // yields editable=false. The SELECT * grid is editable — its PK is
    // in the projection — and must never back a readonly proof.)
    let case_d = t2_case_wb("grid-cell-readonly");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    drive_readonly_query(&mut d, timeout);
    tab_to_footer(&mut d, "↑↓←→ Cell", 8, "grid-cell focus results");
    support::drive(&mut d, &["y", "wait:Copied "], timeout);
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

/// GRID-RESULTS-006: the read-only result grid — duration meta, local
/// sort without a re-run, the empty state, and Activated-instead-of-editing.
#[test]
#[ignore]
fn t2_grid_results_view() {
    let case = t2_case_wb("grid-results-view");
    let dir = t2_dir("t2_grid_results_view");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_results_query(&mut s, timeout);
    // V1: the grid renders under the editor with a duration meta.
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
    support::drive(&mut s, &["y", "wait:Copied "], timeout);
    // S1/A1: `s` sorts locally — marker and reorder, but no SortRequested
    // (the owner path would announce `Sorted by ...` on the status line).
    support::drive(
        &mut s,
        &["home", "right", "right", "right", "right"],
        timeout,
    );
    // The before-row is captured after the move so all three captures
    // share one cell-cursor column (the `▎` would otherwise differ).
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
    // Reorder is proven by grouping (deterministic: random rows never
    // group three alike) rather than by whole-line inequality (padding
    // and cursor chrome would pollute it).
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
    // N1: the query did not re-run — meta and statement byte-identical.
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
    // (The editor gutter `▎` follows focus, not content: blank it to a
    // space so the unfocused gutter still aligns.)
    assert_eq!(
        editor_line.replace('▎', " "),
        editor_after.replace('▎', " "),
        "statement survives the sort"
    );
    // N2 (GRID-SORT-004): clearing a local sort restores the stored order
    // exactly — no reload intervenes on this path, so the row is stable.
    support::drive(&mut s, &["s"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    support::drive(&mut s, &["s"], timeout);
    std::thread::sleep(Duration::from_millis(600));
    let restored = quiesce_text(&mut s, "grid-results restored");
    assert!(
        !restored.contains("Sorted by") && !restored.contains("Sort cleared"),
        "the local path emits no sort events at all"
    );
    let first_restored: String = restored.lines().nth(header_idx + 1).expect("row").into();
    // (Column padding reflows across the sort cycle; the stored-order
    // proof compares whitespace-normalized content.)
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
    // Session B: V2/N2 — the empty state and its lack of grid chrome.
    let case_b = t2_case_wb("grid-results-empty");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive(
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
        timeout,
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

    // Session C: pairing — Enter on an editable table grid DOES edit.
    let case_c = t2_case_wb("grid-results-pairing");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    drive_open_orders(&mut c, timeout);
    support::drive(&mut c, &["enter"], timeout);
    waits::wait_state(&mut c, "Enter Commit", "table grid edits on Enter");

    // Session D: S2/A2 — Enter and Alt+D are inert on a read-only
    // (unattributable, PK-less: editable=false) result grid.
    let case_e = t2_case_wb("grid-results-readonly");
    write_provenance_named(&dir, &case_e, "provenance-e.txt");
    let mut e = support::spawn_boot(&case_e);
    drive_readonly_query(&mut e, timeout);
    tab_to_footer(&mut e, "↑↓←→ Cell", 8, "grid-results focus readonly");
    support::drive(&mut e, &["y", "wait:Copied "], timeout);
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

/// CODE-SQL-003: keyword tones, gutter ownership, statement blocks under
/// the cursor, error readout, dirty tracking, and completion placement.
#[test]
#[ignore]
fn t2_code_sql_editor() {
    let case = t2_case_wb("code-sql-editor");
    let dir = t2_dir("t2_code_sql_editor");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // V1/N2: keywords share one bold tone; plain words do not.
    support::drive(
        &mut s,
        &["tab", "i", "type:SELECT zzz FROM orders"],
        timeout,
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
    // Keywords and plain words share the white foreground; the keyword
    // tone is carried by bold (proven above), which plain words lack.
    let (zrow, zcol) = find_pos(&mut s, "zzz", timeout);
    for j in 0..3u16 {
        assert!(
            frame.get(zcol + j, zrow).is_some_and(|c| !c.mods.bold),
            "plain words escape the keyword tone (N2)"
        );
    }
    checkpoint(&mut s, &dir, "01-tones");

    // Session B: gutter ownership + one statement runs at a time (S2/A1/N1).
    let case_b = t2_case_wb("code-sql-blocks");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    // (The demo engine has no constant expressions (`222` parses as a
    // column): two single-row table selects keep the per-statement proof
    // intact, distinguished by table. The `customers` completion opens
    // mid-typing, so Esc dismisses it before the newline Enter —
    // otherwise Enter accepts the popup instead of breaking.)
    support::drive(
        &mut b,
        &["tab", "i", "type:SELECT * FROM customers LIMIT 1;"],
        timeout,
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
    support::drive(
        &mut b,
        &["enter", "type:SELECT * FROM orders LIMIT 1;"],
        timeout,
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
    // Ctrl+R runs the statement under the cursor (line 2) only.
    support::drive(&mut b, &["ctrl-r"], timeout);
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
    // Cursor up: now line 1 runs (re-enter edit mode first if the run
    // left the editor idle, so `up` reaches the text, not the panels).
    if !live_text(&mut b)
        .lines()
        .last()
        .is_some_and(|l| l.contains("EDIT"))
    {
        support::press_step(&b, "i");
        std::thread::sleep(Duration::from_millis(300));
    }
    support::drive(
        &mut b,
        &["up", "ctrl-r", "wait:SELECT customers (1)"],
        timeout,
    );
    let ran1 = quiesce_text(&mut b, "code-sql ran line 1");
    assert!(
        ran1.contains("FROM customers") && ran1.contains("FROM orders"),
        "statements survive both runs (N1)"
    );
    checkpoint(&mut b, &dir, "03-blocks");

    // Session C: S1 — the tab dot follows text-vs-saved.
    let case_c = t2_case_wb("code-sql-dirty");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    let clean = quiesce_text(&mut c, "code-sql clean");
    assert!(
        !strip_line(&clean).contains('•'),
        "fresh query tab carries no dot"
    );
    support::drive(&mut c, &["tab", "i", "type:X"], timeout);
    escape_to_idle(&mut c, "code-sql dirty");
    // The strip re-syncs on tab events, not on typing: open Query 2 so
    // Query 1's fresh dot is re-read from the workbench tab.
    support::drive(&mut c, &["ctrl-t", "wait:Query 2"], timeout);
    let dirtied = live_text(&mut c);
    assert!(
        strip_line(&dirtied).contains("Query 1 •"),
        "typing dots the query tab (S1)"
    );
    checkpoint(&mut c, &dir, "04-dirty");

    // Session D: V2 — a failed run keeps the statement with an error card.
    let case_d = t2_case_wb("code-sql-error");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::drive(&mut d, &["tab", "i", "type:SELECT * FROM nope"], timeout);
    escape_to_idle(&mut d, "code-sql error");
    support::drive(&mut d, &["ctrl-r", "wait:does not exist"], timeout);
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

    // Session E: A2 — completion opens under the editor cursor cell.
    let case_e = t2_case_wb("code-sql-complete");
    write_provenance_named(&dir, &case_e, "provenance-e.txt");
    let mut e = support::spawn_boot(&case_e);
    support::drive(&mut e, &["tab", "i", "type:SELECT * FROM ord"], timeout);
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

/// SELECT-FILTER-002: the filter editor's Column/Operator selects —
/// labeled ▾ controls, type-ordered operator popups, re-seeding on column
/// change, gated Tab/Esc traversal, and type-fitting first operators.
#[test]
#[ignore]
fn t2_select_filter_builder() {
    let case = t2_case_wb("select-filter-builder");
    let dir = t2_dir("t2_select_filter_builder");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_open_orders(&mut s, timeout);
    support::drive(&mut s, &["ctrl-f", "wait:Add filter"], timeout);
    // V1: labeled selects with ▾, help row, live WHERE preview.
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
    // Column popup from Value: two BackTabs (value <- op <- col), Enter.
    // (Popup rows 0-1 sit under the Value label; the visible rows start at
    // index 2 — the assertions below only touch visible rows.)
    support::drive(&mut s, &["backtab", "backtab", "enter"], timeout);
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
    // S1/A1: shipping_address (JSON) re-seeds operators at index 0.
    support::drive(
        &mut s,
        &[
            "down", "down", "down", "down", "down", "down", "down", "enter",
        ],
        timeout,
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
    // V2: the operator popup lists JSON-first operators; `▎` is the
    // cursor (it moves), `›` marks the committed row (it stays).
    support::drive(&mut s, &["tab", "enter"], timeout);
    waits::wait_state(&mut s, "is empty", "operator popup opens");
    checkpoint(&mut s, &dir, "03-popup-prefind");
    support::drive(&mut s, &["down", "down"], timeout);
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
    // S2: Tab with a popup open never traverses.
    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(400));
    let tabbed = live_text(&mut s);
    assert!(
        tabbed.contains("is empty") && tabbed.contains("contains"),
        "Tab leaves the open popup in place (S2)"
    );
    checkpoint(&mut s, &dir, "03-popup");
    // A1: Operator Changed only repaints — column and dialog untouched.
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
    // N2: an enum column re-seeds to `=` (Json led with `is NULL`);
    // back to Column via BackTab, three Ups to status.
    support::drive(&mut s, &["backtab", "enter"], timeout);
    waits::wait_state(&mut s, "customer_id", "column popup reopens");
    support::drive(&mut s, &["up", "up", "up", "enter"], timeout);
    waits::wait_state(&mut s, "WHERE status", "status column commits");
    let enum_seeded = quiesce_text(&mut s, "select-filter enum reseeded");
    assert!(
        enum_seeded
            .lines()
            .any(|l| l.contains("status") && l.contains('=') && l.contains('▾')),
        "`=` re-seeds first for enum columns (N2)"
    );
    support::drive(&mut s, &["tab", "enter"], timeout);
    waits::wait_state(&mut s, "not in", "enum operators open");
    support::drive(&mut s, &["down", "down"], timeout);
    checkpoint(&mut s, &dir, "04-enum-moved");
    // (The cursor `▎` leaves the committed `›` row: poll for a region
    // line carrying `▎` without `›`.)
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
    // (The committed-row `›` hides under the Value label with rows 0-1,
    // so the cursor-off-committed poll above is the N2 mark proof.)
    // N1/A2: Esc closes the popup first, then cancels the editor.
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

    // Session B: S2 — Tab while the Value edits commits AND advances.
    let case_b = t2_case_wb("select-filter-tab");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    drive_open_orders(&mut b, timeout);
    support::drive(&mut b, &["ctrl-f", "wait:Add filter"], timeout);
    support::drive(&mut b, &["enter", "type:abc", "tab"], timeout);
    waits::wait_state(&mut b, "WHERE id = 'abc'", "Tab commits the value");
    checkpoint(&mut b, &dir, "05-tab-prefind");
    // (The committed Tab advances along the button row: Cancel first.)
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

/// CHIP-FILTER-002: the filter chip bar — one labeled × chip per filter,
/// the match-all lead, toggle/remove/lead/add events with re-runs, and the
/// empty bar that keeps its lead and add stop.
#[test]
#[ignore]
fn t2_chip_filter_bar() {
    let case = t2_case_wb("chip-filter-bar");
    let dir = t2_dir("t2_chip_filter_bar");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    drive_filtered_orders(&mut s, timeout);
    // V1/V2/S1: chip, lead, and the mirrored counts.
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
    // (Pending counts vary per demo load; the filter proof is the chip
    // count plus a pending first row, not the absolute total.)
    assert!(
        bar.contains("filtered (1)"),
        "counts mirror the single enabled filter"
    );
    checkpoint(&mut s, &dir, "01-bar-prefind");
    // (The grid is hscrolled to the status column, so the `∇` filter
    // marker — not an early column name — locates the header.)
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
    // A2: Enter opens the filter editor for that chip.
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Edit filter", "chip activates its editor");
    // (The chip editor opens with its value editing: first Esc leaves
    // the edit, second Esc cancels the editor. A settle-first kills the
    // open-race that swallows Esc.)
    quiesce_text(&mut s, "chip-filter editor settled");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(500));
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Edit filter", "editor cancels back to the bar");
    // S2/A1/N2: Space disables — the query runs unfiltered, the chip dims.
    // (Closing the editor drops chip-bar focus, so refocus first: an
    // unfocused Space would select a grid row instead.)
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
    // A1: Del removes the filter with a status line.
    support::press_step(&s, "delete");
    waits::wait_state(&mut s, "Filter removed", "removal announces");
    let removed = quiesce_text(&mut s, "chip-filter removed");
    assert!(!removed.contains("status = 'pending'"), "the chip is gone");
    assert!(
        removed.contains("of 500 loaded") && !removed.contains("filtered ("),
        "the query runs unfiltered"
    );
    // N1: the empty bar keeps its lead and add stop.
    let empty_bar: String = removed
        .lines()
        .find(|l| l.contains("match all"))
        .expect("empty bar")
        .into();
    assert!(
        empty_bar.contains("match all ▾") && empty_bar.contains("+ Add filter"),
        "lead and add survive emptiness: {empty_bar}"
    );
    // A1: Lead clicks flip match_all both ways. (Proven while the empty
    // bar still shows: cancelling the blank editor below returns focus
    // to the grid, which hides the empty bar.)
    click_text(&mut s, "match all", 1, timeout, "chip-filter lead off");
    waits::wait_state(&mut s, "match any ▾", "lead flips to any");
    click_text(&mut s, "match any", 1, timeout, "chip-filter lead on");
    waits::wait_state(&mut s, "match all ▾", "lead flips back to all");
    checkpoint(&mut s, &dir, "04-lead");
    // Cursor already rests on the add stop (it clamped on remove).
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Add filter", "add stop opens a blank editor");
    // (The blank editor opens idle: one Esc cancels it — but the Esc
    // sometimes lands mid-open and gets swallowed, so a still-open
    // editor earns exactly one more, gated on it still showing.)
    quiesce_text(&mut s, "chip-filter blank settled");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(600));
    if live_text(&mut s).contains("Add filter") {
        support::press_step(&s, "escape");
    }
    waits::wait_gone(&mut s, "Add filter", "blank editor cancels");
    checkpoint(&mut s, &dir, "03-empty");
}

/// TABS-STATES-002: the workbench strip — state glyphs, the single accent
/// underline, overflow paging, repaired removal, keyboard/mouse tab actions,
/// and the empty strip that only answers Enter/n.
#[test]
#[ignore]
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
    // V1: dirty dot after the label. (The strip re-syncs on tab events,
    // not on typing, so Query 2 opens first and re-reads Query 1's dot.)
    support::drive(&mut s, &["tab", "i", "type:X"], timeout);
    escape_to_idle(&mut s, "tabs dirty");
    support::drive(&mut s, &["ctrl-t", "wait:Query 2"], timeout);
    escape_to_idle(&mut s, "tabs quiet");
    let dotted = strip_line(&live_text(&mut s)).to_string();
    assert!(
        dotted.contains("Query 1 •"),
        "dirty dot trails the label (V1): {dotted}"
    );
    checkpoint(&mut s, &dir, "01-dirty");
    // V2: active tab owns the accent underline; quiet tabs draw gray.
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
    // V1: background failure marks `!`, cleared on view.
    tab_to_footer(&mut s, "Alt+R Run all", 8, "tabs focus editor");
    support::drive(&mut s, &["i", "type:SELECT * FROM nope"], timeout);
    escape_to_idle(&mut s, "tabs failing query");
    support::drive(&mut s, &["ctrl-t", "wait:Query 3"], timeout);
    support::press_step(&s, "[");
    std::thread::sleep(Duration::from_millis(300));
    support::press_step(&s, "ctrl-r");
    std::thread::sleep(Duration::from_millis(100));
    support::press_step(&s, "]");
    std::thread::sleep(Duration::from_millis(3000));
    // The strip re-syncs on tab events: Query 4 opens (never viewing
    // Query 2) so its settled `!` is re-read from the workbench tab.
    support::drive(&mut s, &["ctrl-t", "wait:Query 4"], timeout);
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

    // Session B: V1 — the busy spinner on a background run.
    let case_b = t2_case_wb("tabs-busy");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive(&mut b, &["tab", "i", "type:SELECT * FROM orders"], timeout);
    escape_to_idle(&mut b, "tabs busy query");
    support::drive(&mut b, &["ctrl-t", "wait:Query 2", "["], timeout);
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

    // Session C: overflow, removal, actions, empty strip.
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
    // V3/N2: the active tail tab parks the strip at the tail — a left
    // `‹N` count, full labels, and the trailing +. (The strip takes
    // focus first: an unfocused click only focuses, never pages.)
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
    // A2: paging is proven from the middle (tail paging fights the
    // active-tail re-reveal): jump to Query 5, page head-ward, page back.
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
    // S1/S2: digit-jump to the middle, close, first preserved.
    tab_to_footer(&mut c, "← → Switch", 8, "tabs focus strip");
    support::press_step(&c, "5");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("─ Query 5 ─"),
        "digit-jump activates Query 5"
    );
    support::press_step(&c, "x");
    wait_absent(&mut c, "Query 5", 8, "middle tab closes (A1/S1)");
    // (Remove repairs active onto the next neighbor (Query 6), and the
    // strip re-reveals around it: Query 4 hides left, Query 6 shows.)
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
    // S2: stepping active out of view re-reveals with the active shown.
    // (Exact counts: `]`/`[` wrap around, so overshooting laps back.)
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
    // S1: Delete closes the head tab; active repairs onto Query 2.
    support::press_step(&c, "delete");
    std::thread::sleep(Duration::from_millis(400));
    let after_del = live_text(&mut c);
    assert!(
        !strip_line(&after_del).contains("Query 1") && after_del.contains("─ Query 2 ─"),
        "Delete repairs active onto Query 2 (S1)"
    );
    checkpoint(&mut c, &dir, "06-repaired");
    // A1: n and + open queries; A2: tab click activates.
    tab_to_footer(&mut c, "← → Switch", 8, "tabs refocus strip");
    support::press_step(&c, "n");
    waits::wait_state(&mut c, "Query 12", "n opens a query (A1)");
    strip_click(&mut c, "+", 0, "tabs + button");
    waits::wait_state(&mut c, "Query 13", "+ opens a query (A1)");
    // Jump to Query 3 (positioning for the click test; ‹-paging itself
    // is proven from the middle above, where no re-reveal fights it).
    // (Digits are positional: Query 3 sits at index 1 now. The +-click
    // left focus in the editor, so refocus the strip first.)
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
    // N1: empty the strip; only Enter/n answer, and only with New.
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

/// TI-FORM-010: typing and routing in the connection form — filled Name,
/// masked Password, commit validation, focus routing, pointer editing, and
/// Esc revert.
#[test]
#[ignore]
fn t2_input_form_field() {
    let case = t2_case_conn("input-form-field");
    let dir = t2_dir("t2_input_form_field");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    support::drive(&mut s, &["ctrl-n", "wait:Name"], timeout);
    // V2/S2: the empty Password shows its keychain placeholder, unmasked.
    let blank = quiesce_text(&mut s, "input-form blank");
    let pw: String = blank
        .lines()
        .find(|l| l.contains("stored in the keychain"))
        .expect("password placeholder")
        .into();
    assert!(!pw.contains('•'), "empty Password masks nothing (V2): {pw}");
    checkpoint(&mut s, &dir, "01-blank");
    // V1: the typed Name fills its field.
    support::drive(&mut s, &["enter", "type:Staging replica", "enter"], timeout);
    waits::wait_state(&mut s, "Staging replica", "name commits");
    // S1: committing an empty Name raises Required (scoped to the Name
    // field: the Database help line always reads `Required for ...`).
    support::drive(&mut s, &["enter", "end", "ctrl-u", "enter"], timeout);
    poll_name_error(&mut s, true, 8, "empty name rejected (S1)");
    // Repair: a valid Name clears the error.
    support::drive(&mut s, &["enter", "type:Staging replica", "enter"], timeout);
    checkpoint(&mut s, &dir, "02-repair-prefind");
    poll_name_error(&mut s, false, 8, "valid name clears the error");
    checkpoint(&mut s, &dir, "02-validated");
    // N1: Esc reverts to the pre-edit value; the dialog stays open.
    support::drive(&mut s, &["enter", "type:ZZZ", "escape"], timeout);
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
    // N2: Password masks one • per grapheme once typed.
    tab_to_cursor(&mut s, "stored in the keychain", 20, "input-form password");
    support::drive(&mut s, &["enter", "type:abc"], timeout);
    waits::wait_state(&mut s, "•••", "password masks per grapheme (N2)");
    let masked = live_text(&mut s);
    assert!(
        !masked.contains("stored in the keychain"),
        "typing displaces the placeholder"
    );
    checkpoint(&mut s, &dir, "03-masked");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(300));
    // A2: clicking Host focuses it and edits at the pointer.
    let (hrow, hcol) = find_pos(&mut s, "localhost", timeout);
    s.click(MouseButton::Left, hcol + 1, hrow, MouseMods::NONE)
        .expect("input-form host click");
    std::thread::sleep(Duration::from_millis(400));
    support::drive(&mut s, &["type:X"], timeout);
    waits::wait_state(&mut s, "lXocalhost", "edit lands at the pointer (A2)");
    let pointed = live_text(&mut s);
    assert!(
        pointed
            .lines()
            .any(|l| l.contains('▎') && l.contains("lXocalhost")),
        "the click focused Host first (A2)"
    );
    checkpoint(&mut s, &dir, "04-pointer");
    // A1: CommittedTab routes Host -> Port; Ctrl+S submits the form.
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

/// FORM-CONN-004: connection validation and the advanced section — port
/// range errors, blocked saves, optional ports, required names, SSH rows,
/// and Test-without-saving.
#[test]
#[ignore]
fn t2_form_connection_full() {
    let case = t2_case_conn("form-connection-full");
    let dir = t2_dir("t2_form_connection_full");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    support::drive(&mut s, &["ctrl-n", "wait:Name"], timeout);
    support::drive(&mut s, &["enter", "type:Probe conn", "enter"], timeout);
    waits::wait_state(&mut s, "Probe conn", "name commits");
    // V1: an out-of-range port errors with a `!` marker. (Host shares
    // the Port row, so the `5432` hunt stops at Host; one Tab steps to
    // Port, verified by the cursor column against the Port label.)
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
    support::drive(
        &mut s,
        &["enter", "end", "ctrl-u", "type:99999", "enter"],
        timeout,
    );
    checkpoint(&mut s, &dir, "01-bad-port-prefind");
    waits::wait_state(&mut s, "Port must", "port rejects 99999 (V1)");
    checkpoint(&mut s, &dir, "01-bad-port-shown");
    let bad_port = live_text(&mut s);
    // (The narrow Port field clips the validator string to `Port must…`;
    // the row's full `Port must be 1–65535` lives in port_validator — the
    // live proof is the clipped message plus the `!` marker.)
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
    // A1/N1: Save stays blocked; nothing reaches the tree.
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
    // S2: clearing the port validates (the field is optional).
    support::drive(&mut s, &["enter", "end", "ctrl-u", "enter"], timeout);
    wait_absent(&mut s, "Port must", 8, "empty port validates (S2)");
    // N2/S1: a blank name never validates and blocks Save alone (Name
    // error scoped to its field: Database help always reads `Required`).
    // (The Name row shares its lines with the Environment radios, so a
    // text hunt can stop on the wrong field: click the value itself.)
    click_text(&mut s, "Probe conn", 1, timeout, "form-conn name");
    support::press_step(&s, "escape");
    std::thread::sleep(Duration::from_millis(300));
    support::drive(&mut s, &["enter", "end", "ctrl-u", "enter"], timeout);
    checkpoint(&mut s, &dir, "02-name-cleared-prefind");
    poll_name_error(&mut s, true, 8, "blank name rejected (N2)");
    support::press_step(&s, "ctrl-s");
    std::thread::sleep(Duration::from_millis(800));
    let blocked_name = live_text(&mut s);
    assert!(
        name_field_error(&blocked_name) && !blocked_name.contains("Connection saved"),
        "name alone gates the save (S1)"
    );
    support::drive(&mut s, &["enter", "type:Probe conn", "enter"], timeout);
    poll_name_error(&mut s, false, 8, "name repaired");
    checkpoint(&mut s, &dir, "02-valid");
    // V2: the advanced section shows SSH rows and all four actions.
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
    // A2: Test runs without saving or closing.
    tab_to_cursor(&mut s, "Test connection", 25, "form-conn test");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "✓ Connected", "test connects (A2)");
    checkpoint(&mut s, &dir, "04-tested-prefind");
    let tested = quiesce_text(&mut s, "form-conn tested");
    assert!(
        tested.contains("SSH host") && tested.contains("Save & connect"),
        "Test leaves the form open (A2)"
    );
    // (The Name field lives on the Basic tab; persistence is proven by
    // its absence from the explorer tree, not by a whole-screen count.)
    assert!(
        !explorer_half(&tested).contains("Probe conn"),
        "Test persists nothing (A2)"
    );
    checkpoint(&mut s, &dir, "04-tested");
}

/// DIALOG-ACK-003: the typed-acknowledgement write gate — facts plus
/// clipped preview, ack focus and arming, deliberate Execute, plain/danger
/// focus rules, close confirmation, and dead y/n keys.
#[test]
#[ignore]
fn t2_dialog_ack_full() {
    let case = t2_case_wb("dialog-ack-full");
    let dir = t2_dir("t2_dialog_ack_full");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // Seven content lines (blank lines trim out of the gate preview, so
    // the clip tail needs real content): Esc closes any completion
    // first, `i` re-enters edit mode when Esc idled the editor.
    support::drive(&mut s, &["tab", "i", "type:UPDATE orders"], timeout);
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
        support::drive(&mut s, &[&format!("type:{line}")], timeout);
    }
    assert!(
        live_text(&mut s).contains("ln 7/7"),
        "seven editor lines stand"
    );
    escape_to_idle(&mut s, "dialog-ack statement");
    support::drive(&mut s, &["ctrl-r", "wait:Type orders to confirm"], timeout);
    // V1: facts above a preview clipped to six lines with its tail.
    checkpoint(&mut s, &dir, "01-gate-prefind");
    let gate = quiesce_text(&mut s, "dialog-ack gate");
    for needle in ["Action", "Target", "Safe Mode", "UPDATE orders", "… 1 more"] {
        assert!(gate.contains(needle), "gate shows `{needle}` (V1)");
    }
    // V2/S2: the ack field sits above the actions and owns the focus.
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
    // N2: y/n in the ack input type text — they never confirm.
    support::drive(&mut s, &["enter", "type:yn"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    let typed = live_text(&mut s);
    assert!(
        typed.contains("Type orders to confirm") && !typed.contains("affected"),
        "y/n never confirm the gate (N2)"
    );
    // N1/A1: a wrong token advances to Cancel and never arms.
    support::drive(&mut s, &["ctrl-l", "type:000000", "enter"], timeout);
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
    // N1: an untrimmed token (embedded space) never arms either.
    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(300));
    assert!(
        live_text(&mut s)
            .lines()
            .any(|l| l.contains('▎') && !l.contains("Cancel") && !l.contains("Execute")),
        "BackTab returns to the ack input"
    );
    support::drive(
        &mut s,
        &["enter", "ctrl-l", "type:or ders", "enter"],
        timeout,
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
    // S1/A1: the exact token arms; Execute stays one deliberate step away.
    support::press_step(&s, "backtab");
    std::thread::sleep(Duration::from_millis(300));
    support::drive(
        &mut s,
        &["enter", "ctrl-l", "type:orders", "enter"],
        timeout,
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
    // (The demo engine applies the UPDATE wholesale — 8022 rows — rather
    // than honoring the uuid WHERE: the Execute proof is count-agnostic.)
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

    // Session B: V2 — short screens clip only the facts region.
    let case_b = t2_case_wb("dialog-ack-short");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive(
        &mut b,
        &[
            "tab",
            "i",
            "type:UPDATE orders SET status = 'paid' WHERE id = 'x'",
        ],
        timeout,
    );
    escape_to_idle(&mut b, "dialog-ack short statement");
    support::drive(&mut b, &["ctrl-r", "wait:Type orders to confirm"], timeout);
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
        if b.observe_now().expect("geometry").screen.rows() == 20 {
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

    // Session C: S2 — plain writes focus Execute with no ack input.
    let case_c = t2_case_wb("dialog-ack-plain");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    support::press_step(&c, "ctrl-l");
    waits::wait_state(&mut c, "Safe Mode · this connection", "picker opens");
    support::drive(&mut c, &["up", "up", "enter"], timeout);
    wait_absent(&mut c, "Safe Mode · this connection", 8, "picker applies");
    assert!(live_text(&mut c).contains("alert"), "Alert level applies");
    support::drive(
        &mut c,
        &[
            "tab",
            "i",
            "type:UPDATE orders SET status = 'paid' WHERE id = 'x'",
        ],
        timeout,
    );
    escape_to_idle(&mut c, "dialog-ack plain statement");
    support::drive(&mut c, &["ctrl-r", "wait:Execute write query?"], timeout);
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

    // Session D: A2/S2 — the destructive close confirm focuses Cancel.
    let case_d = t2_case_wb("dialog-ack-close");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::drive(&mut d, &["ctrl-t", "wait:Query 2"], timeout);
    tab_to_footer(&mut d, "Alt+R Run all", 8, "dialog-ack focus editor");
    support::drive(&mut d, &["i", "type:x"], timeout);
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

/// HELP-TABLEPRO-003: the keyboard reference dialog — 78-wide single
/// Close action, full chord table, saved focus round-trip, every close key,
/// and dead y/arrow keys.
#[test]
#[ignore]
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
    // V1/S1/S2: 78-wide frame, lone Close, initial focus on it.
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
    // (The modal's own footer hints `Esc Cancel`; the stripped action is
    // proven inside the dialog region, not the whole screen.)
    let help_modal = modal_region(&open, "Keyboard").join("\n");
    assert!(
        help_modal.contains("Close") && !help_modal.contains("Cancel"),
        "a single Close action, Cancel stripped (V1/S1)"
    );
    assert!(
        open.lines().any(|l| l.contains('▎') && l.contains("Close")),
        "initial focus sits on Close (S1/S2)"
    );
    // V2: the table names the workbench chords from Ctrl+O to q.
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
    // A1/A2: every close key finishes and restores the grid focus.
    for (i, key) in ["escape", "enter", "space", "n"].iter().enumerate() {
        support::press_step(&s, key);
        waits::wait_gone(&mut s, "Keyboard", &format!("{key} closes help (A1)"));
        // (The focus-ring repaint lags the close: poll, don't snapshot.)
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
    // N1: y never closes (Close is Secondary — no primary to answer).
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
    // N2: Left/Right find no second action.
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

/// TREE-FILTER-002: tree filtering with ancestor reveal, lazy expansion
/// with busy spinners, muted unselectable notes, and identity-kept focus.
#[test]
#[ignore]
fn t2_tree_connection_filter() {
    let case = t2_case_conn("tree-connection-filter");
    let dir = t2_dir("t2_tree_connection_filter");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // A2: every keystroke re-runs the filter; V1: ancestors stay open.
    support::press_step(&s, "/");
    std::thread::sleep(Duration::from_millis(300));
    support::drive(&mut s, &["type:s"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    let one = live_text(&mut s);
    assert!(
        one.contains("Staging") && !one.contains("Production"),
        "first keystroke already refilters (A2)"
    );
    support::drive(&mut s, &["type:tag"], timeout);
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
    // A2: committing re-runs too; S1: the tree cursor resets to row 0.
    support::press_step(&s, "enter");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut s).contains("◇ Staging"),
        "committing keeps the filter (A2)"
    );
    support::press_step(&s, "tab");
    std::thread::sleep(Duration::from_millis(400));
    checkpoint(&mut s, &dir, "01-filtered-prefind");
    // (The focus-ring repaint lags the key: poll, don't snapshot.)
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

    // Session B: N1 — a match-nothing filter yields zero rows, no crash.
    let case_b = t2_case_conn("tree-filter-empty");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::press_step(&b, "/");
    std::thread::sleep(Duration::from_millis(300));
    support::drive(&mut b, &["type:zzz-no-match"], timeout);
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

    // Session C: V2/A1/S2/N2 — lazy analytics expands above a kept cursor.
    let case_c = t2_case_wb("tree-lazy-load");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    assert!(
        live_text(&mut c).contains("▸ S analytics"),
        "lazy nodes wait collapsed (V2)"
    );
    support::press_step(&c, "home");
    std::thread::sleep(Duration::from_millis(200));
    // Phase 1: cursor on the node — the fetch marks busy (A1/V2) and the
    // Tables group arrives (A1).
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
    // Phase 2: collapse it, park below, re-expand with cursor-free `*`
    // (children are loaded now, so expand-all reaches them): arrivals
    // above never retarget the parked cursor (S2/N2 — flatten keeps the
    // focused node as an identity, not a display position).
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

    // Session D: V3 — the triggers note is muted and never selectable.
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
    // (V3 covers selectability — Space — not keyboard activation.)
    let noted = no_footer(&quiesce_text(&mut d, "tree-note settled"));
    support::press_step(&d, "space");
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(
        noted,
        no_footer(&quiesce_text(&mut d, "tree-note space")),
        "Space never selects a note (V3)"
    );
    // Pairing: Space on a plain collapsed node DOES expand it.
    down_to(&mut d, "audit", 25, "tree-note audit");
    support::press_step(&d, "space");
    waits::wait_state(&mut d, "▾ S audit", "Space expands plain nodes");
    checkpoint(&mut d, &dir, "04-note");
}

/// PICKER-SWITCHER-003: the quick-open switcher, tab list, and safe-mode
/// pickers — scopes, substring filtering, per-kind dispatch, secondary
/// close, and the keys that must not act.
#[test]
#[ignore]
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
    // V1: scope label plus kind-glyphed targets (typed queries force exact
    // matches, so default ranking can never hide the needles).
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
        support::drive(&mut s, &[&format!("type:{query}")], timeout);
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
    // S1: Tab cycles All -> Tables -> Schemas -> Queries -> All, rows only.
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
        support::drive(&mut s, &[&format!("type:{query}")], timeout);
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
    // A1: schema dispatch rebuilds the explorer for that schema.
    support::drive(&mut s, &["tab", "tab"], timeout);
    waits::wait_state(&mut s, "Schemas · Tab scope", "back to Schemas");
    support::drive(&mut s, &["type:audit", "enter"], timeout);
    checkpoint(&mut s, &dir, "03-dispatch-prefind");
    // (The dispatch lands during the drive; presence was proven upstream
    // by the Schemas-scope wait, so absence needs no presence preamble.)
    wait_absent(&mut s, "Open Quickly", 8, "schema chosen closes");
    assert!(
        live_text(&mut s)
            .lines()
            .any(|l| l.contains("Explorer") && l.contains("audit")),
        "schema dispatch rebuilds the explorer (A1)"
    );
    // A1: table dispatch opens the table tab.
    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher reopens");
    // (A reopened switcher resets its scope: hunt All from wherever it
    // starts rather than assuming the remembered Schemas.)
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
    support::drive(&mut s, &["type:orders", "enter"], timeout);
    waits::wait_state(&mut s, "public › orders", "table dispatch opens (A1)");
    // A1: query dispatch switches to that tab (stepping away first so the
    // switch is observable).
    support::drive(&mut s, &["ctrl-t", "wait:Query 2"], timeout);
    escape_to_idle(&mut s, "switcher new tab");
    support::press_step(&s, "[");
    std::thread::sleep(Duration::from_millis(300));
    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher reopens");
    support::drive(&mut s, &["type:Query 2", "enter"], timeout);
    std::thread::sleep(Duration::from_millis(500));
    assert!(
        live_text(&mut s).contains("─ Query 2 ─"),
        "query dispatch switches tabs (A1)"
    );
    checkpoint(&mut s, &dir, "03-chosen");
    // N1/A1: Back and Submit keep the picker open; Cancelled closes it.
    tab_to_footer(&mut s, "Enter Open", 8, "switcher focus tree");
    support::press_step(&s, "ctrl-o");
    waits::wait_state(&mut s, "Open Quickly", "switcher reopens");
    support::drive(&mut s, &["type:zzz-no-match", "enter"], timeout);
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

    // Session B: V2/S2/A2 — the tab list filters, details, and closes.
    let case_b = t2_case_wb("picker-tab-list");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive(&mut b, &["ctrl-t", "wait:Query 2"], timeout);
    tab_to_footer(&mut b, "Enter Open", 8, "tablist focus tree");
    drive_open_orders(&mut b, timeout);
    tab_to_footer(&mut b, "← → Switch", 8, "tablist focus strip");
    support::press_step(&b, "ctrl-g");
    waits::wait_state(&mut b, "Open tabs", "tab list opens");
    // (The rows render after the title: poll for them, don't snapshot.)
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
    // S2: substring filtering; the detail carries the tab index.
    support::drive(&mut b, &["type:Query"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    checkpoint(&mut b, &dir, "05-tablist-prefind");
    let filtered = modal_region(&live_text(&mut b), "Open tabs");
    // (The region spans full-width lines: grid_half drops the explorer
    // background, whose own `▸ T orders` row would pollute the check.)
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
    // A2: Delete closes that tab and reopens the list.
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

    // Session C: V2/N2 — safe-mode marks current; NextScope is inert.
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
    // (A background ▎ artifact behind the modal is out of row scope; the
    // picker rows, cursor, and title below are the NextScope surface.)
    assert_eq!(
        before,
        scoped(&live_text(&mut c)),
        "NextScope outside the switcher changes nothing (N2)"
    );
    support::press_step(&c, "escape");
    waits::wait_gone(&mut c, "Safe Mode · this connection", "picker cancels");
    checkpoint(&mut c, &dir, "07-safemode");
}

/// COMPLETE-SQL-001: the query editor suggestion popup — anchoring,
/// kinds, bold prefixes, aligned details, width clamp, scrollbar, wheel
/// stance, accept/replace, paren landing, dismiss paths, and move keys.
#[test]
#[ignore]
fn t2_complete_sql_popup() {
    let case = t2_case_wb("complete-sql-popup");
    let dir = t2_dir("t2_complete_sql_popup");
    write_provenance(&dir, &case);
    let timeout = case_timeout(&case);
    // V1/anchor/width/S1/cursor0 + bold prefix + aligned details.
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    support::drive(&mut s, &["tab", "i", "type:SELECT * FROM ord"], timeout);
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
    // Bold matched prefix (read on the unselected second row: the selected
    // first row adds its own selection styling).
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
    // The name column pads to the longest label, so the details share
    // one start column (left-aligned) as well as one end column.
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

    // Session B: A1 — Tab accepts and replaces the typed prefix.
    let case_b = t2_case_wb("complete-accept");
    write_provenance_named(&dir, &case_b, "provenance-b.txt");
    let mut b = support::spawn_boot(&case_b);
    support::drive(&mut b, &["tab", "i", "type:SELECT * FROM ord"], timeout);
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

    // Session C: columns popup — scrollbar, wheel stance, move keys, Enter.
    let case_c = t2_case_wb("complete-columns");
    write_provenance_named(&dir, &case_c, "provenance-c.txt");
    let mut c = support::spawn_boot(&case_c);
    support::drive(
        &mut c,
        &[
            "tab",
            "i",
            "type:x FROM orders o",
            "home",
            "type:SELECT o.",
            "delete",
        ],
        timeout,
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
    // S2: a wheel scroll survives later frames (the popup hugs the editor,
    // so its first row sits exactly two lines below the statement).
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
    // Reset the popup (the wheel may have scrolled the cursor out of
    // view — and rows out of the viewport, so dismissal is proven by
    // the anchored border below the editor, not by a row needle).
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
    support::drive(&mut c, &["backspace", "type:."], timeout);
    waits::wait_state(&mut c, "is_gift", "popup reopens");
    // N1: plain n/p type into the editor (dot-retype reopens deterministically).
    support::drive(&mut c, &["type:n"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("o.n"),
        "plain n types into the editor (N1)"
    );
    support::drive(&mut c, &["backspace", "backspace", "type:."], timeout);
    waits::wait_state(&mut c, "is_gift", "popup reopens");
    support::drive(&mut c, &["type:p"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut c).contains("o.p"),
        "plain p types into the editor (N1)"
    );
    support::drive(&mut c, &["backspace", "backspace", "type:."], timeout);
    waits::wait_state(&mut c, "is_gift", "popup reopens");
    // Down/Up move the popup cursor. (The row's N1 names Ctrl+N/Ctrl+P
    // as the movers, but Ctrl+N opens Quick Open live — even with the
    // popup up — so the live movers are proven and the discrepancy is
    // noted.) The popup flickers across repaints, so the cursor row is
    // polled, never snapshotted.
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
    // A1: Enter accepts the cursor row.
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

    // Session D: A1 — functions land the cursor inside the parens.
    let case_d = t2_case_wb("complete-function");
    write_provenance_named(&dir, &case_d, "provenance-d.txt");
    let mut d = support::spawn_boot(&case_d);
    support::drive(&mut d, &["tab", "i", "type:SELECT coale"], timeout);
    waits::wait_state(&mut d, "coalesce", "function popup opens");
    support::drive(&mut d, &["down", "down", "down", "down"], timeout);
    waits::wait_state(&mut d, "▎F coalesce", "cursor reaches the function");
    support::press_step(&d, "tab");
    std::thread::sleep(Duration::from_millis(500));
    let called = live_text(&mut d);
    assert!(
        called.contains("coalesce(") && called.contains(')'),
        "accepting a function inserts parens (A1)"
    );
    support::drive(&mut d, &["type:1"], timeout);
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut d).contains("coalesce(1)"),
        "the cursor lands inside the parens (A1)"
    );
    checkpoint(&mut d, &dir, "05-function");

    // Session E: A2 — Esc dismisses with the statement intact.
    let case_e = t2_case_wb("complete-esc");
    write_provenance_named(&dir, &case_e, "provenance-e.txt");
    let mut e = support::spawn_boot(&case_e);
    support::drive(&mut e, &["tab", "i", "type:SELECT * FROM ord"], timeout);
    waits::wait_state(&mut e, "order_items", "completion opens");
    support::press_step(&e, "escape");
    waits::wait_gone(&mut e, "K ORDER BY", "Esc dismisses (A2)");
    assert!(
        live_text(&mut e).contains("SELECT * FROM ord"),
        "dismiss keeps the statement (A2)"
    );
    checkpoint(&mut e, &dir, "06-esc");

    // Session F: A2 — click-outside does NOT close live (editor text,
    // tree, and results pane all tried): the popup stays with the
    // statement intact. (The row's click-outside clause is live-false;
    // Esc-dismiss and commit-close are proven in E and B/D.)
    let case_f = t2_case_wb("complete-clickout");
    write_provenance_named(&dir, &case_f, "provenance-f.txt");
    let mut f = support::spawn_boot(&case_f);
    support::drive(&mut f, &["tab", "i", "type:SELECT * FROM ord"], timeout);
    waits::wait_state(&mut f, "order_items", "completion opens");
    for (col, row) in [(25u16, 7u16), (5, 20), (60, 30)] {
        f.click(MouseButton::Left, col, row, MouseMods::NONE)
            .expect("complete click-outside");
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

    // Session G: S1/N2 — a match-nothing query closes the popup entirely.
    let case_g = t2_case_wb("complete-nomatch");
    write_provenance_named(&dir, &case_g, "provenance-g.txt");
    let mut g = support::spawn_boot(&case_g);
    support::drive(&mut g, &["tab", "i", "type:SELECT * FROM ord"], timeout);
    waits::wait_state(&mut g, "order_items", "completion opens");
    support::drive(&mut g, &["type:zzz"], timeout);
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
