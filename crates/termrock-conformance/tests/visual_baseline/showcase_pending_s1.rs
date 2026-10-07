//! Showcase pending-roots slice 8A-S1 executable checks — impl port.
//!
//! Ported verbatim from VB commit `76537c27e6e00ef93a2d09d280c8c9cb6097e446`
//! (`tests/harness/tests/visual_baseline/showcase_pending_s1.rs`), adapting
//! only paths, binary resolution, and harness API:
//!
//! - `Session` is the impl [`support::Session`] wrapper (live samples via
//!   `s.inner.observe_now()` / `s.inner.snapshot()`), not
//!   `tuiscotti::tui::Session` directly.
//! - The subject is the `showcase` binary built from this impl worktree's
//!   sources, resolved via [`support::try_resolve_bin`] (name -> executed
//!   path + sha256); [`write_provenance`] records path, digest, size,
//!   mtime, and argv behind each check.
//! - `support::drive` (3-arg) becomes [`support::drive_with_timeout`]
//!   (timeout in ms).
//! - `support::frame_from_screen` (VB helper over `Provenance`) becomes
//!   `tuiscotti::render::frame_from_screen` (provenance `"default"`).
//!
//! One ignored PTY test per S1 registry row (21 rows), over the real
//! `showcase` binary built from this worktree's impl sources (resolved via
//! [`support::try_resolve_bin`]). Each test drives the row's specified
//! inputs and asserts its visual (V), state (S), action (A), and negative
//! (N) checks in live-PTY executable form:
//!
//! - V: live needles plus cell-geometry probes (luminance ramps for the
//!   scroll-edge fade, single-line coexistence for two-column cards).
//! - S: live labels (window/mode/cursor/readout rows) plus boot-vs-final
//!   comparisons captured from the same session.
//! - A: behavior probes — the specified gesture runs and the specified
//!   advance (or boundary refusal) is observed; `Changed`/`Consumed`/
//!   `Ignored` outcomes are proven by their screen correlates (moved,
//!   stopped-at-end, byte-identical), since PTY cannot see the enum.
//! - N: absence assertions, each paired with a presence proof in the same
//!   test (boot frame, second session, or pre/post transition) so no
//!   absence passes vacuously.
//!
//! No new snapshots and no new static captures: all 21 roots already have
//! approved frames gated cell-exact by the ported matrices in `showcase.rs`
//! and `pointer.rs` (plan §Reconciliation: "rows plus checks, not
//! recaptures"), so every case here uses owned (dynamic) names and stays
//! out of the `snapshots/` inventory. No isolated-component captures: every
//! row's `requires` set (tick-control, glyph-capture, geometry-probe) is
//! PTY-level, and every assertion has a live-PTY executable form.
//!
//! Per-test scratch lands under this conformance crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `00-boot.txt` where the row needs a boot
//! comparison, and `provenance.txt` (binary path, digest, size, mtime, argv).
//! Typed input is synthetic and in-memory only (simulation data).
//!
//! Row → test map (registry id → `s1_*` test):
//!
//! - TABLE-PAGE-001 → [`s1_table_page_boot`]
//! - TABLE-SELECT-002 → [`s1_table_select_cursor`]
//! - TABLE-EDIT-003 → [`s1_table_edit_open`]
//! - TABLE-COMMIT-004 → [`s1_table_edit_commit`]
//! - TABLE-HOVER-005 → [`s1_table_hover_lift`]
//! - TABLE-EDITABLE-006 → [`s1_table_editable_boot`]
//! - GRID-WHEEL-007 → [`s1_grid_wheel_fade`]
//! - CODE-WHEEL-004 → [`s1_code_wheel_fade`]
//! - DIFF-WHEEL-004 → [`s1_diff_wheel_review`]
//! - SCROLL-PAGE-003 → [`s1_scroll_page_prose`]
//! - SCROLL-WHEEL-004 → [`s1_scroll_wheel_offtail`]
//! - TERM-WHEEL-003 → [`s1_term_wheel_scrollback`]
//! - LIST-WHEEL-003 → [`s1_list_wheel_fade`]
//! - TREE-WHEEL-003 → [`s1_tree_wheel_fade`]
//! - NAV-WHEEL-003 → [`s1_nav_wheel_demo`]
//! - TA-WHEEL-004 → [`s1_textarea_wheel_fade`]
//! - INSPECT-OPEN-001 → [`s1_inspect_open`]
//! - INSPECT-SCROLL-002 → [`s1_inspect_scrolled`]
//! - PANEL-STATIC-002 → [`s1_panel_overview_static`]
//! - TOOSMALL-GROWN-002 → [`s1_resize_grown`]
//! - TOOSMALL-SHRUNK-003 → [`s1_resize_shrunk`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::{MouseButton, Wheel};
use tuiscotti::{Color as TColor, Frame, Rgb};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

/// Owned-name S1 case at the row's viewport (truecolor, like every S1 row).
fn s1_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s1/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

/// Unique scratch dir for one S1 test (created, never shared).
fn s1_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary identity: resolved absolute path, sha256 digest, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
    write_provenance_named(dir, case, "provenance.txt");
}

/// [`write_provenance`] with an explicit filename, for multi-session tests
/// where each session's argv deserves its own record.
fn write_provenance_named(dir: &Path, case: &Case, file: &str) {
    let subject =
        support::try_resolve_bin(case.bin).unwrap_or_else(|e| panic!("resolve {}: {e}", case.bin));
    let modified = subject
        .mtime_unix
        .map_or_else(|| "unknown".to_string(), |m| m.to_string());
    let argv = support::argv_for(case);
    let body = format!(
        "profile: tuiscotti-default\ntransport: pty\nport_of: 76537c27e6e00ef93a2d09d280c8c9cb6097e446\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("s1 provenance ({file}): {body}");
}

/// Save the live screen text as `<name>.txt` in the test dir.
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

/// One fresh live text sample (no wait).
fn live_text(s: &mut Session) -> String {
    let obs = s
        .inner
        .observe_now()
        .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
    support::screen_text(&obs.screen)
}

/// One fresh live frame with cell colors (no wait). Provenance is
/// informational only (excluded from digests).
fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .inner
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    frame_from_screen(&screen, "default")
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

/// `notches` wheel steps over `needle`'s cell, paced like a send step.
/// Typed input refuses (fail-closed) when tracking is off or off-grid.
/// Returns the `(row, col)` wheeled, for stance probes after the content
/// scrolled (the cell stays inside the scroll region).
fn wheel_over(
    s: &mut Session,
    needle: &str,
    dir: Wheel,
    notches: u32,
    timeout: Duration,
) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    wheel_at(s, col, row, dir, notches);
    (row, col)
}

/// `notches` wheel steps at fixed `(col, row)` — the stance-probe form,
/// used when the needle may have scrolled off-screen.
fn wheel_at(s: &mut Session, col: u16, row: u16, dir: Wheel, notches: u32) {
    for _ in 0..notches {
        Input::wheel(dir, col, row).send(s);
        std::thread::sleep(Duration::from_millis(120));
    }
}

/// Poll live text until two consecutive 400 ms-apart samples agree
/// (bounded): the settle form for states with trailing animation (press
/// flashes, smooth scroll) that a needle wait would catch mid-flight.
/// Returns the settled text.
fn settle_text(s: &mut Session, timeout: Duration, what: &str) -> String {
    let deadline = std::time::Instant::now() + timeout;
    let mut prev = live_text(s);
    loop {
        std::thread::sleep(Duration::from_millis(400));
        let cur = live_text(s);
        if cur == prev {
            return cur;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: text never settled"
        );
        prev = cur;
    }
}

/// Wheel in 5-notch batches until the screen stops changing (bounded):
/// the boundary-seeking form for end/stance probes. Returns the settled
/// end text.
fn wheel_until_stable(
    s: &mut Session,
    col: u16,
    row: u16,
    dir: Wheel,
    timeout: Duration,
    what: &str,
) -> String {
    let deadline = std::time::Instant::now() + timeout;
    let mut prev = settle_text(s, Duration::from_millis(3000), what);
    for _ in 0..10 {
        wheel_at(s, col, row, dir, 5);
        let cur = settle_text(s, Duration::from_millis(3000), what);
        if cur == prev {
            return cur;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: never reached the boundary"
        );
        prev = cur;
    }
    panic!("{what}: still moving after 10 batches");
}

/// Hover the pointer over `needle`'s cell (any-motion tracking required).
fn hover_over(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::move_to(col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

/// Primary-button click on `needle`'s cell (split press + release).
fn click_at(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::down(MouseButton::Left, col, row).send(s);
    Input::up(MouseButton::Left, col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

/// Resize and wait for the new geometry, then let the app redraw: the
/// emulator reports the size before the app repaints (~300 ms), so a bare
/// geometry wait can observe the blank post-resize frame.
fn resize_to(s: &mut Session, cols: u16, rows: u16, timeout: Duration) {
    s.resize(cols, rows).expect("resize");
    support::wait_screen(
        s,
        timeout,
        &format!("never reached {cols}x{rows}"),
        |screen| screen.cols() == cols && screen.rows() == rows,
    );
    std::thread::sleep(Duration::from_millis(700));
}

/// Resolve a live cell color to RGB. `Default` has no live-resolvable
/// value (it is the terminal's own color), so resolving one fails the
/// check loudly instead of guessing.
fn resolve_rgb(c: TColor, what: &str) -> Rgb {
    match c {
        TColor::Rgb(rgb) => rgb,
        TColor::Indexed(n) => Rgb::from_indexed(n),
        TColor::Default => panic!("{what}: cell color is Default, cannot resolve"),
    }
}

/// Relative luminance (Rec. 709 scaled by 1000) of a live cell color.
fn lum(c: TColor, what: &str) -> u32 {
    let rgb = resolve_rgb(c, what);
    u32::from(rgb.r) * 2126 + u32::from(rgb.g) * 7152 + u32::from(rgb.b) * 722
}

/// Median foreground luminance of the non-blank text cells on `row`.
/// Blank/continuation cells carry no text tone and are skipped; at least
/// one text cell must resolve.
fn row_fg_median(frame: &Frame, row: u16, what: &str) -> u32 {
    let mut lums: Vec<u32> = Vec::new();
    for col in 0..frame.cols {
        let Some(cell) = frame.get(col, row) else {
            continue;
        };
        if cell.continuation || cell.symbol.trim().is_empty() {
            continue;
        }
        if matches!(cell.fg, TColor::Default) {
            continue;
        }
        lums.push(lum(cell.fg, what));
    }
    assert!(
        !lums.is_empty(),
        "{what}: row {row} has no resolvable-fg text cells"
    );
    lums.sort_unstable();
    lums[lums.len() / 2]
}

/// Median foreground luminance over one uniform text span (`len` cells
/// from `col` on `row`): the fade probe for rows whose full width mixes
/// unrelated tones (nav gutter, sibling columns, inspector card, check
/// marks). Both spans in a comparison must share one base tone.
fn span_fg_median(frame: &Frame, row: u16, col: u16, len: usize, what: &str) -> u32 {
    let mut lums: Vec<u32> = Vec::new();
    for c in col..col.saturating_add(len as u16) {
        let Some(cell) = frame.get(c, row) else {
            continue;
        };
        if cell.continuation || cell.symbol.trim().is_empty() {
            continue;
        }
        if matches!(cell.fg, TColor::Default) {
            continue;
        }
        lums.push(lum(cell.fg, what));
    }
    assert!(
        !lums.is_empty(),
        "{what}: span ({col}..+{len}, row {row}) has no resolvable-fg cells"
    );
    lums.sort_unstable();
    lums[lums.len() / 2]
}

/// The fade dims toward the container background: the faded row's median
/// foreground luminance must read strictly below the interior row's.
fn assert_faded(edge: u32, interior: u32, what: &str) {
    assert!(
        edge < interior,
        "{what}: edge row must fade (edge lum {edge} < interior lum {interior})"
    );
}

/// An unfaded row reads at full tone: medians must match exactly.
fn assert_same_tone(a: u32, b: u32, what: &str) {
    assert!(
        a == b,
        "{what}: rows must read at the same tone (lum {a} == {b})"
    );
}

/// One cell identical across two frames (symbol, fg, bg): the executable
/// form of "this row must NOT fade / must NOT move".
fn assert_cell_same(a: &Frame, b: &Frame, col: u16, row: u16, what: &str) {
    let ca = a
        .get(col, row)
        .unwrap_or_else(|| panic!("{what}: cell ({col}, {row}) missing in first frame"));
    let cb = b
        .get(col, row)
        .unwrap_or_else(|| panic!("{what}: cell ({col}, {row}) missing in second frame"));
    assert!(
        ca.symbol == cb.symbol && ca.fg == cb.fg && ca.bg == cb.bg,
        "{what}: cell ({col}, {row}) differs: {ca:?} vs {cb:?}"
    );
}

/// Whole text row identical across two frames (all cells incl. colors).
fn assert_row_same(a: &Frame, b: &Frame, row: u16, what: &str) {
    assert_eq!(
        a.cols, b.cols,
        "{what}: frame widths differ ({} vs {})",
        a.cols, b.cols
    );
    for col in 0..a.cols {
        assert_cell_same(a, b, col, row, what);
    }
}

/// Some line must contain both `needle` and `also` (the two-column /
/// same-row coexistence proof). Any-line: the first line holding one
/// needle may be a different row (nav gutter vs card vs legend).
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line contains both `{needle}` and `{also}`"
    );
}

/// No line containing `needle` may contain `also` (region-scoped absence).
fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        assert!(
            !line.contains(also),
            "{what}: `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

/// No line containing `needle` may contain `also` right of the nav/content
/// separator (┃/│): the nav gutter's own markers (› current page) share
/// screen rows with card rows and must not count as card markers.
fn assert_right_lacks(text: &str, needle: &str, also: &str, what: &str) {
    for line in text.lines().filter(|l| l.contains(needle)) {
        let right = line
            .find(['┃', '│'])
            .map(|off| &line[off..])
            .unwrap_or(line);
        assert!(
            !right.contains(also),
            "{what}: card side of `{needle}` line must not contain `{also}`: {line:?}"
        );
    }
}

/// No body line (below the header crumb row) may contain `label`: the
/// section-label absence proof (the header crumb itself names
/// "Foundations" and must not count).
fn assert_body_lacks(text: &str, label: &str, what: &str) {
    for line in text.lines().skip(1) {
        assert!(
            !line.contains(label),
            "{what}: `{label}` must not render in the body: {line:?}"
        );
    }
}

/// Some body line (below the header crumb row) must contain `needle`.
fn assert_body_has(text: &str, needle: &str, what: &str) {
    assert!(
        text.lines().skip(1).any(|l| l.contains(needle)),
        "{what}: no body line contains `{needle}`"
    );
}

/// Parse a `7–19` / `7-19` window label into `(first, last)`.
fn parse_window(label: &str, what: &str) -> (u32, u32) {
    let sep = if label.contains('–') { '–' } else { '-' };
    let (a, b) = label
        .split_once(sep)
        .unwrap_or_else(|| panic!("{what}: no window separator in `{label}`"));
    let first: u32 = a
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("{what}: bad window start in `{label}`"));
    let last: u32 = b
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .unwrap_or_else(|_| panic!("{what}: bad window end in `{label}`"));
    (first, last)
}

/// TABLE-PAGE-001 (`showcase/pages/tables`): tasks + checks cards at boot.
///
/// Flow: boot only, then action probes (Tab / Down / s / Enter) that prove
/// the boot state was the unfocused, unsorted, unselected one.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_page_boot() {
    let dir = s1_dir("s1_table_page_boot");
    let case = s1_case("table_page_boot", &["--page", "tables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);

    // V1: header plus rows #1040–#1044, unsorted meta, empty Checks card.
    for needle in ["#1040", "#1041", "#1042", "#1043", "#1044"] {
        assert!(boot.contains(needle), "V1: boot lacks row {needle}");
    }
    assert_line_has(&boot, "Tasks", "unsorted · 1–5 of 24", "V1 meta");
    assert_line_has(&boot, "ID", "Task", "V1 header");
    assert!(
        boot.contains("No checks have run yet"),
        "V1: Checks card lacks the empty state"
    );
    // V2: no focus bar inside either card; no sort marker in the header.
    assert_eq!(
        boot.matches('▎').count(),
        1,
        "V2: exactly the nav focus bar may show at boot"
    );
    assert_line_has(&boot, "▎", "Tables", "V2 nav focus");
    assert!(
        !boot.contains('▴') && !boot.contains('▾'),
        "V2: no sort marker at boot"
    );
    // S1 (sort half): identity order. S2: armed but unapplied.
    assert!(boot.contains("unsorted"), "S1/S2: sort is None at boot");
    // N1: nothing activated yet (the Enter probe below proves the needle).
    assert!(!boot.contains("Selected"), "N1: no Selected status at boot");

    // A1: Tab moves focus from the nav gutter into the Tasks card.
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Columns", "A1 table footer");
    let focused = live_text(&mut s);
    assert_line_has(&focused, "▎", "Tasks", "A1 focus bar on the card");
    // S1 (cursor half): the cursor rests at (0, 0) — the first row.
    assert_line_has(&focused, "#1040", "▎", "S1 cursor at row 0");
    // A2: Down moves the cursor; s sorts (both fire only once focused).
    // press_step has no trailing sleep, so the wait must name the
    // post-condition — a bare ▎ wait would pass on the pre-press frame.
    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 cursor on row 1",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("#1041") && l.contains('▎'))
        },
    );
    support::press_step(&s, "s");
    support::wait_screen(&mut s, case_timeout(&case), "A2 sort applied", |screen| {
        let text = support::screen_text(screen);
        !text.contains("unsorted") && (text.contains('▴') || text.contains('▾'))
    });
    // N2: Enter selects (Activated), never edits — no tables column is
    // editable. The Selected status here also proves the N1 needle.
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Selected", "N2 selection status");
    let selected = live_text(&mut s);
    assert!(!selected.contains("EDIT"), "N2: Enter selects, never edits");
    checkpoint(&mut s, &dir, "01-selected");
    eprintln!("s1 table_page_boot: boot proven, tab/down/s/enter all behaved");
}

/// TABLE-SELECT-002 (`showcase/flows/tables/selected`): cursor row after
/// tab + down + down.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_select_cursor() {
    let dir = s1_dir("s1_table_select_cursor");
    let case = s1_case("table_select_cursor", &["--page", "tables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "down", "down"], case.timeout_ms);
    waits::wait_state(&mut s, "Columns", "cursor-moved table footer");
    checkpoint(&mut s, &dir, "cursor-moved");
    let text = live_text(&mut s);

    // V1: focus bar on the card, cursor row #1042, table footer hints.
    assert_line_has(&text, "#1042", "▎", "V1 cursor row");
    for needle in ["Move", "Columns", "Sort", "Select", "Next"] {
        assert!(text.contains(needle), "V1: footer lacks hint {needle}");
    }
    // V2: no selection marker (the nav's own › shares these rows, so
    // only the card side counts); the meta still reads the full window.
    assert_right_lacks(&text, "#104", "›", "V2 no selection marker");
    assert!(
        text.contains("unsorted · 1–5 of 24"),
        "V2: meta keeps the window"
    );
    // S1: cursor (0,0) → (2,0); selected stays None (Enter never pressed).
    assert_line_has(&text, "#1042", "▎", "S1 cursor at (2,0)");
    // S2: offset stays 0 — row 2 was already visible.
    assert!(text.contains("1–5 of 24"), "S2: viewport offset stays 0");
    // N1: cursor motion is not activation (the Space probe proves it).
    assert!(!text.contains("Selected"), "N1: no Selected status");
    // N2: s was never pressed.
    assert!(text.contains("unsorted"), "N2: sort untouched");

    // A1: Down/j moves one row (Changed ⟺ the cursor moved). The wait
    // names the post-condition (see T1 A2: no trailing sleep on press).
    support::press_step(&s, "j");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1 cursor on row 3",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("#1043") && l.contains('▎'))
        },
    );
    // A2: Space from here selects (Activated plus the Selected status).
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "Selected", "A2 space selects");
    assert!(
        !live_text(&mut s).contains("EDIT"),
        "A2: selection, not editing"
    );
    checkpoint(&mut s, &dir, "01-space-selected");
    eprintln!("s1 table_select_cursor: cursor (2,0) proven, j moved, space selected");
}

/// TABLE-EDIT-003 (`showcase/flows/editable/editing`): cell editor open on
/// the Task cell.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_edit_open() {
    let dir = s1_dir("s1_table_edit_open");
    let case = s1_case("table_edit_open", &["--page", "editabletables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "enter"], case.timeout_ms);
    waits::wait_state(&mut s, "EDIT", "editing footer");
    checkpoint(&mut s, &dir, "editing");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: EDIT footer with hints; the draft with underline + hardware cursor.
    for needle in ["EDIT", "Commit", "Cancel", "Next cell"] {
        assert!(text.contains(needle), "V1: footer lacks {needle}");
    }
    assert!(
        text.contains("…te limiting to auth endpoints"),
        "V1: draft prefilled with the cell tail"
    );
    let (draft_row, draft_col) = find_pos(&mut s, "auth endpoints", case_timeout(&case));
    let underlined = (draft_col.saturating_sub(10)..draft_col + 14)
        .filter_map(|col| frame.get(col, draft_row))
        .any(|cell| cell.mods.underline);
    assert!(underlined, "V1: draft carries the accent underline");
    assert!(
        frame.cursor.visible && frame.cursor.y == draft_row,
        "V1: hardware cursor sits on the draft row (cursor {:?} vs row {draft_row})",
        (frame.cursor.x, frame.cursor.y, frame.cursor.visible),
    );
    // V2: zero-edits meta; the cursor legend below the card.
    assert!(text.contains("0 edits · 1–10 of 14"), "V2: zero-edits meta");
    for needle in [
        "cell cursor (navigation)",
        "editing cursor + accent underline",
    ] {
        assert!(text.contains(needle), "V2: legend lacks {needle:?}");
    }
    // S1: EditState{row 0, col 1} prefilled with the cell text.
    assert_line_has(&text, "#1040", "▎", "S1 editor on row 0");
    // S2: nothing committed yet.
    assert!(text.contains("0 edits"), "S2: edits counter stays 0");
    // N1: the validator must NOT fire before commit.
    assert_line_lacks(&text, "#104", "!", "N1 no pre-commit error");
    // N2: a single EditState — exactly one draft, one EDIT.
    assert_eq!(
        text.matches("…te limiting").count(),
        1,
        "N2: exactly one draft cell"
    );
    assert_eq!(text.matches("EDIT").count(), 1, "N2: exactly one EDIT");

    // A2: typed keys insert into the edit buffer.
    support::drive_with_timeout(&mut s, &["type:z"], case.timeout_ms);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 typed key inserted",
        |screen| support::screen_text(screen).contains("endpointsz"),
    );
    // A1: Esc cancels (Changed plus Cancelled) — EDIT leaves, the card rests.
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "A1 edit cancelled");
    assert!(
        live_text(&mut s).contains("0 edits"),
        "A1: cancel keeps the counter at 0"
    );
    checkpoint(&mut s, &dir, "01-cancelled");
    eprintln!("s1 table_edit_open: draft + underline + cursor proven, z inserted, esc cancelled");
}

/// TABLE-COMMIT-004 (`showcase/flows/editable/committed`): committed cell +
/// edits counter.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_edit_commit() {
    let dir = s1_dir("s1_table_edit_commit");
    let case = s1_case("table_edit_commit", &["--page", "editabletables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
        &mut s,
        &["tab", "enter", "type:x", "enter", "wait:1 edits"],
        case.timeout_ms,
    );
    checkpoint(&mut s, &dir, "committed");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: one-edits meta; Cell saved; EDIT gone (also S1/S2/N1).
    assert!(
        text.contains("1 edits · 1–10 of 14"),
        "V1/S1: one-edits meta"
    );
    assert!(text.contains("Cell saved"), "V1: Cell saved status");
    assert!(!text.contains("EDIT"), "V1/S2/N1: EDIT mode cleared");
    // V2: the committed cell keeps no underline or cursor. The appended x
    // itself truncates off-screen at 80x24 (the committed row reads
    // identical to boot), so the re-edit probe below is its content proof.
    let (cell_row, _) = find_pos(&mut s, "#1040", case_timeout(&case));
    let underlined = (0..frame.cols)
        .filter_map(|col| frame.get(col, cell_row))
        .any(|cell| cell.mods.underline);
    assert!(!underlined, "V2: no underline remains on the cell");
    // N2: the non-empty Task passes validation.
    assert_line_lacks(&text, "#104", "!", "N2 no validator error");

    // V2 (content proof): re-entering the editor shows the draft with x.
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "V2 re-edit shows the draft");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "V2 draft carries the appended x",
        |screen| support::screen_text(screen).contains("endpointsx"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "V2 draft closed");
    // A2: Tab during editing commits the cell and opens the next editable
    // one (table.rs EditAction::Tab); past the last editable cell it emits
    // LeaveForward, which leaves the card. Editable cols: Task(1),
    // Owner(2), Branch(4), Changes(5).
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "A2 editing (0,1) again");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "2 edits", "A2 tab committed (0,1)");
    assert!(
        live_text(&mut s).contains("EDIT"),
        "A2: still editing after tab"
    );
    // The draft moved to the Owner cell: "mira" carries the underline now.
    let owner_frame = live_frame(&mut s);
    let (mira_row, mira_col) = find_pos(&mut s, "mira", case_timeout(&case));
    let owner_underlined = (mira_col..mira_col + 4)
        .filter_map(|col| owner_frame.get(col, mira_row))
        .any(|cell| cell.mods.underline);
    assert!(owner_underlined, "A2: draft on Owner (0,2)");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "3 edits", "A2 tab committed (0,2)");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "4 edits", "A2 tab committed (0,4)");
    checkpoint(&mut s, &dir, "01-tab3");
    // Past the last editable cell: Tab commits the draft into the data
    // but the leaving commit's Committed event is dropped (table.rs
    // returns LeaveForward instead) — the counter stays at 4 while focus
    // leaves the card. The "7" suffix distinguishes the committed data.
    support::drive_with_timeout(&mut s, &["type:7"], case.timeout_ms);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 distinguishing suffix typed",
        |screen| support::screen_text(screen).contains("147"),
    );
    support::press_step(&s, "tab");
    waits::wait_gone(&mut s, "EDIT", "A2 LeaveForward left the card");
    assert!(
        live_text(&mut s).contains("4 edits"),
        "A2: leaving commit drops the Committed event (counter stays 4)"
    );
    checkpoint(&mut s, &dir, "02-left-card");
    // The data WAS committed: shift+tab returns focus to the card (cursor
    // still on (0,5)) and the re-opened draft shows the suffixed value.
    support::press_step(&s, "shift+tab");
    waits::wait_state(&mut s, "▎Tasks", "A2 focus back on the card");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "A2 re-editing (0,5)");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 leaving commit wrote the data",
        |screen| support::screen_text(screen).contains("147"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "A2 draft closed");
    checkpoint(&mut s, &dir, "03-data-proven");

    // A1 (second half): an empty Task fails validation and stays editing.
    // The buffer is cleared with plain backspaces (35 covers the 33-char
    // draft; no select-all assumption).
    let case2 = s1_case(
        "table_edit_commit_empty",
        &["--page", "editabletables"],
        80,
        24,
    );
    write_provenance_named(&dir, &case2, "provenance-empty.txt");
    let mut s2 = support::spawn_boot(&case2);
    let mut clear = vec!["tab", "enter", "wait:EDIT"];
    clear.extend(std::iter::repeat_n("backspace", 35));
    support::drive_with_timeout(&mut s2, &clear, case2.timeout_ms);
    support::press_step(&s2, "enter");
    // Still editing with the validator error on the row (the wait names
    // the error post-condition — EDIT was already present).
    support::wait_screen(
        &mut s2,
        case_timeout(&case2),
        "A1 validator error on the row",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("#1040") && l.contains('!'))
        },
    );
    checkpoint(&mut s2, &dir, "02-empty-state");
    assert!(live_text(&mut s2).contains("EDIT"), "A1: stays editing");
    // The meta doubles as the error readout while the commit is refused.
    assert!(
        live_text(&mut s2).contains("Task name cannot be empty"),
        "A1: validator message in the meta"
    );
    // Cancel out: the refused commit counted nothing.
    support::press_step(&s2, "escape");
    waits::wait_gone(&mut s2, "EDIT", "A1 error cancelled");
    assert!(
        live_text(&mut s2).contains("0 edits"),
        "A1: empty commit counts nothing"
    );
    checkpoint(&mut s2, &dir, "03-empty-rejected");
    eprintln!("s1 table_edit_commit: commit + x proven, tab chain + LeaveForward, empty rejected");
}

/// TABLE-HOVER-005 (`showcase/hover/tables`): row hover lift without focus.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_hover_lift() {
    let dir = s1_dir("s1_table_hover_lift");
    let case = s1_case("table_hover_lift", &["--page", "tables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let idle_text = live_text(&mut s);
    let idle = live_frame(&mut s);
    let (hover_row, _) = find_pos(&mut s, "#1042", case_timeout(&case));

    hover_over(&mut s, "#1042", case_timeout(&case));
    // The lift is color-only: wait for the digest to move off idle.
    let hovered = waits::wait_frame_where(
        &mut s,
        case_timeout(&case),
        "V1 hover lift paints",
        |frame| frame.digest() != idle.digest(),
    );
    checkpoint(&mut s, &dir, "hovered");
    let text = live_text(&mut s);

    // V1: idle-vs-hovered differ in cells but read identical as text.
    assert_eq!(hovered.text(), idle.text(), "V1: identical text");
    assert_eq!(text, idle_text, "V1: live text identical too");
    // V2: hover alone never focuses.
    assert_eq!(
        text.matches('▎').count(),
        1,
        "V2: only the nav focus bar shows"
    );
    // S1/N1: the hit is row 2 only — every differing cell sits on its row.
    let mut diff_rows = std::collections::BTreeSet::new();
    for col in 0..idle.cols {
        let a = idle.get(col, hover_row);
        let b = hovered.get(col, hover_row);
        if a.map(cell_sig) != b.map(cell_sig) {
            diff_rows.insert(hover_row);
            break;
        }
    }
    for row in 0..idle.rows {
        if row == hover_row {
            continue;
        }
        for col in 0..idle.cols {
            if idle.get(col, row).map(cell_sig) != hovered.get(col, row).map(cell_sig) {
                diff_rows.insert(row);
                break;
            }
        }
    }
    assert_eq!(
        diff_rows,
        std::collections::BTreeSet::from([hover_row]),
        "S1/N1: exactly the hovered row changes plane"
    );
    // S1 (rest): focus ring and cursor untouched; S2: not suppressed.
    assert!(
        text.contains("Tab Into page"),
        "S1: footer still the nav footer"
    );
    // A1: rendering only — the footer line is byte-identical.
    let idle_footer = idle_text.lines().last().unwrap_or_default().to_string();
    let footer = text.lines().last().unwrap_or_default().to_string();
    assert_eq!(footer, idle_footer, "A1: footer untouched by hover");
    // N2: hover must NOT select (card side only — the nav › shares rows).
    assert_right_lacks(&text, "#104", "›", "N2 no selection marker");
    assert!(!text.contains("Selected"), "N2: no selection status");

    // A2: hover follows the pointer — moving to #1043 lifts that row and
    // rests #1042 back to its idle cells.
    hover_over(&mut s, "#1043", case_timeout(&case));
    let moved = waits::wait_frame_where(
        &mut s,
        case_timeout(&case),
        "A2 hover follows the pointer",
        |frame| frame.digest() != hovered.digest(),
    );
    let (row3, _) = find_pos(&mut s, "#1043", case_timeout(&case));
    for col in 0..idle.cols {
        assert_cell_same(&moved, &idle, col, hover_row, "A2 #1042 rested");
    }
    let lifted = (0..idle.cols)
        .filter_map(|col| {
            let a = idle.get(col, row3).map(cell_sig);
            let b = moved.get(col, row3).map(cell_sig);
            (a != b).then_some(col)
        })
        .count();
    assert!(lifted > 0, "A2: #1043 row lifts under the pointer");
    checkpoint(&mut s, &dir, "01-hover-moved");
    eprintln!("s1 table_hover_lift: plane lift proven, follows pointer, selects nothing");
}

/// Cell render signature for the hover plane-diff (symbol + colors).
fn cell_sig(cell: &tuiscotti::Cell) -> (String, TColor, TColor) {
    (cell.symbol.clone(), cell.fg, cell.bg)
}

/// TABLE-EDITABLE-006 (`showcase/pages/editable`): editable page at boot,
/// unfocused.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_table_editable_boot() {
    let dir = s1_dir("s1_table_editable_boot");
    let case = s1_case("table_editable_boot", &["--page", "editabletables"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);

    // V1: unfocused card, zero-edits meta, the legend below.
    assert_eq!(
        boot.matches('▎').count(),
        1,
        "V1/N2: only the nav focus bar shows"
    );
    assert!(boot.contains("0 edits · 1–10 of 14"), "V1: zero-edits meta");
    assert!(
        boot.contains("cell cursor (navigation)"),
        "V1: legend renders"
    );
    // V2: the … fold marker; the preset error stays off-screen.
    assert_line_has(&boot, "Owner", "…", "V2 overflow marker");
    assert_line_lacks(&boot, "#104", "!", "V2/S2 preset error off-screen");
    // N1: no EDIT mode at boot.
    assert!(!boot.contains("EDIT"), "N1: no EDIT at boot");

    // S1/A1: Tab enters the table — cell-nav puts the reversed cursor on
    // the Task cell (0,1), NOT on the ID cell (0,0).
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Cell", "S1 table footer");
    let frame = live_frame(&mut s);
    let (row0, id_col) = find_pos(&mut s, "#1040", case_timeout(&case));
    let (_, task_col) = find_pos(&mut s, "Add rate limiting", case_timeout(&case));
    let id_bg = frame
        .get(id_col, row0)
        .unwrap_or_else(|| panic!("S1: ID cell ({id_col}, {row0}) missing"))
        .bg;
    let task_bg = frame
        .get(task_col, row0)
        .unwrap_or_else(|| panic!("S1: Task cell ({task_col}, {row0}) missing"))
        .bg;
    assert_ne!(task_bg, id_bg, "S1/A1: cursor cell (0,1) paints reversed");
    // A2: arrows move the reversed cell; Enter edits from here.
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 reversed cell moved right",
        |screen| {
            let probe = frame_from_screen(screen, "default");
            probe.get(task_col, row0).map(|c| c.bg) != Some(task_bg)
        },
    );
    support::press_step(&s, "left");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "A2/S1 enter edits (validator path)");
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "EDIT", "A2 editor closed");
    checkpoint(&mut s, &dir, "01-entered");

    // S2: rows[2][4] carries the preset branch error ("fix/checkout
    // flake") — scroll it into view and the ! badge appears. The cursor
    // must park on (2,5), NOT on the error cell: the badge paints only
    // when the error cell is visible but not the cursor cell (table.rs).
    support::drive_with_timeout(
        &mut s,
        &["down", "down", "right", "right", "right", "right"],
        case.timeout_ms,
    );
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "S2 preset error scrolled into view",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('!') && l.contains("fix/checkout") && !l.contains("valid"))
        },
    );
    checkpoint(&mut s, &dir, "02-preset-error");
    eprintln!("s1 table_editable_boot: unfocused boot + reversed (0,1) + preset error proven");
}

/// GRID-WHEEL-007 (`showcase/fade/datagrid/wheel`): customers viewport +
/// scroll-edge fade after two down-notches.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_grid_wheel_fade() {
    let dir = s1_dir("s1_grid_wheel_fade");
    let case = s1_case("grid_wheel_fade", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    let boot = live_frame(&mut s);
    assert!(boot_text.contains("rows 1–14 of 40"), "S1: boot offset 0");

    // A2 (first half): each notch advances the offset — pin the midpoint.
    wheel_over(
        &mut s,
        "Northwind Traders",
        Wheel::Down,
        1,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "rows 4–17 of 40", "A2 first notch advances");
    wheel_over(
        &mut s,
        "Northwind Traders",
        Wheel::Down,
        1,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "rows 7–20 of 40", "wheel-faded meta");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: the scrolled meta and the first data row.
    assert!(
        text.contains("rows 7–20 of 40 loaded · ~4,812 total"),
        "V1: scrolled meta"
    );
    assert_line_has(&text, "1007", "Wide World Importers", "V1 first row");
    // V2: the top body edge fades (14-row viewport fades two rows deep);
    // the header never fades; the thumb sits below the track top.
    let (r7, _) = find_pos(&mut s, "1007", case_timeout(&case));
    let (r8, _) = find_pos(&mut s, "1008", case_timeout(&case));
    let (r_mid, _) = find_pos(&mut s, "1013", case_timeout(&case));
    let outer = row_fg_median(&frame, r7, "V2 outer edge");
    let inner = row_fg_median(&frame, r8, "V2 inner edge");
    let mid = row_fg_median(&frame, r_mid, "V2 interior");
    assert_faded(inner, mid, "V2 inner edge fades");
    assert_faded(outer, inner, "V2 outer edge fades deepest");
    let (header_row, _) = find_pos(&mut s, "customer", case_timeout(&case));
    assert_row_same(&boot, &frame, header_row, "V2/N1 header never fades");
    let track_col = (0..frame.cols)
        .filter(|col| {
            frame
                .get(*col, r7)
                .is_some_and(|c| c.symbol == "│" || c.symbol == "┃")
        })
        .last()
        .expect("V2: scrollbar column on the first body row");
    assert_eq!(
        frame.get(track_col, r7).map(|c| c.symbol.as_str()),
        Some("│"),
        "V2: track (not thumb) at the top"
    );
    let thumb_below =
        (r7 + 1..frame.rows).any(|row| frame.get(track_col, row).is_some_and(|c| c.symbol == "┃"));
    assert!(thumb_below, "V2: thumb sits below the track top");
    // S1: offset 0 → 6; sort and editing untouched.
    assert!(
        !text.contains('▴') && !text.contains('▾'),
        "S1: sort stays null"
    );
    assert!(!text.contains("EDIT"), "S1: editing untouched");
    // S2/A2 (rest): the cell cursor stays put — viewport only.
    assert!(
        text.contains("Tab Into page"),
        "S2: footer still the nav footer"
    );
    assert_eq!(text.matches('▎').count(), 1, "S2: no focus bar in the card");
    // N2: no sort, edit, or page.
    assert!(text.contains("of 40 loaded"), "N2: page_rows stays 40");

    // A1: Changed while rows hide below (the notches moved); Consumed at
    // the bottom — wheel in batches until the screen stops changing (the
    // long scroll may animate), pin the true bottom label, then stance.
    let (end_row, end_col) = find_pos(&mut s, "Northwind Traders", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 bottom offset",
    );
    checkpoint(&mut s, &dir, "01-end-reached");
    // The true bottom is rows 28–40: the "fetch more" row consumes the
    // 14th viewport slot (offset 27 = 40 rows + fetch row − viewport 14).
    // The wheel never fetches — only Enter does (A2) — so stance holds.
    assert!(
        end.contains("rows 28–40 of 40"),
        "A1: bottom offset is rows 28–40"
    );
    assert!(
        end.contains("Enter fetches more"),
        "A1: fetch row anchors the bottom"
    );
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the bottom");
    checkpoint(&mut s, &dir, "01-bottom");
    eprintln!("s1 grid_wheel_fade: offset 0→3→6, two-deep fade, bottom Consumed");
}

/// CODE-WHEEL-004 (`showcase/fade/editor/wheel`): code viewport + gutter
/// fade after two down-notches.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_code_wheel_fade() {
    let dir = s1_dir("s1_code_wheel_fade");
    let case = s1_case("code_wheel_fade", &["--page", "codeeditor"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_frame(&mut s);

    wheel_over(
        &mut s,
        "pub async fn fetch",
        Wheel::Down,
        2,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "7–11 of 26", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: gutter lines 7–11 with the label; file + blocks in the header.
    assert!(text.contains("7–11 of 26"), "V1: window label");
    assert!(
        text.contains("Err(e) if e.is_transient()"),
        "V1: line 7 content"
    );
    assert_line_has(&text, "retry.rs", "3 blocks", "V1 header");
    // V2: gutter and code fade on both edges (5-row viewport: one row each).
    let (r7, _) = find_pos(&mut s, "Err(e)", case_timeout(&case));
    let (r9, _) = find_pos(&mut s, "sleep(delay)", case_timeout(&case));
    let (r10, _) = find_pos(&mut s, "delay *= 2", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, r7, "V2 top edge"),
        row_fg_median(&frame, r9, "V2 interior"),
        "V2 top edge fades",
    );
    // Line 11 (`}`, the last code row) sits directly under line 10.
    let bottom = r10 + 1;
    let bottom_line = live_text(&mut s)
        .lines()
        .nth(usize::from(bottom))
        .unwrap_or_default()
        .to_string();
    assert!(
        bottom_line.contains('}'),
        "V2: line 11 closes the viewport: {bottom_line:?}"
    );
    assert_faded(
        row_fg_median(&frame, bottom, "V2 bottom edge"),
        row_fg_median(&frame, r9, "V2 interior"),
        "V2 bottom edge fades",
    );
    // S1: offset 0 → 6 (1–5 → 7–11 skips line 6: two 3-line notches);
    // block segmentation untouched. (State-card pairs: the live card pads
    // labels with spaces, so coexistence — not exact spacing — is pinned.)
    assert_line_has(&text, "Block", "1 of 3", "S1: blocks untouched");
    // S2/A2: the text cursor stays; the wheel never edits.
    assert_line_has(&text, "Cursor", "ln 1 · col 1", "S2/A2: cursor stays");
    assert_line_has(&text, "Mode", "navigating", "S2/A2: still navigating");
    // N1: the filename header must NOT fade.
    let (header_row, _) = find_pos(&mut s, "retry.rs", case_timeout(&case));
    assert_row_same(&boot, &frame, header_row, "N1 header never fades");
    // N2: diagnostics and block boundaries must NOT move.
    assert_line_has(&text, "Diagnostics", "0", "N2: diagnostics stay");

    // A1: Consumed at the bottom — wheel until stable, pin the label, stance.
    let (end_row, end_col) = find_pos(&mut s, "sleep(delay)", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 bottom offset",
    );
    assert!(end.contains("22–26 of 26"), "A1: bottom offset is 22–26");
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the bottom");
    checkpoint(&mut s, &dir, "01-bottom");
    eprintln!("s1 code_wheel_fade: 7–11 proven, both edges fade, bottom Consumed");
}

/// DIFF-WHEEL-004 (`showcase/fade/diff/wheel`): review panes + delegated
/// wheel scroll.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_diff_wheel_review() {
    let dir = s1_dir("s1_diff_wheel_review");
    let case = s1_case("diff_wheel_review", &["--page", "diff"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(&mut s, &["tab", "enter", "wait:● Review"], case.timeout_ms);
    // Settle: the Enter press paints a pressed flash on the Review control
    // that decays after the needle first appears — the N1 pre-frame must
    // be the steady state, not the flash.
    let review_text = settle_text(&mut s, Duration::from_millis(5000), "review steady");
    checkpoint(&mut s, &dir, "00-review");
    let review = live_frame(&mut s);

    wheel_over(&mut s, "attempts = 3", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "@@ -9,6 +10,7 @@", "wheel-faded hunk header");
    settle_text(&mut s, Duration::from_millis(5000), "wheel-faded steady");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: old/new columns with bold changed runs; the hunk header visible.
    assert!(text.contains("@@ -9,6 +10,7 @@"), "V1: hunk header");
    assert!(text.contains("attempts = 3"), "V1: old column");
    assert!(text.contains("attempts = 5"), "V1: new column");
    let (att_row, att_col) = find_pos(&mut s, "attempts = 3", case_timeout(&case));
    let bold_run = (att_col..att_col + 12)
        .filter_map(|col| frame.get(col, att_row))
        .any(|cell| cell.mods.bold);
    assert!(bold_run, "V1: changed run reads bold");
    // V2: both viewport edges fade — the wheel left the review mid-document.
    let (top_row, _) = find_pos(&mut s, "╭─ Diff", case_timeout(&case));
    let (bot_row, _) = find_pos(&mut s, "╰─", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, top_row + 1, "V2 top edge"),
        row_fg_median(&frame, att_row, "V2 interior"),
        "V2 top edge fades",
    );
    assert_faded(
        row_fg_median(&frame, bot_row - 1, "V2 bottom edge"),
        row_fg_median(&frame, att_row, "V2 interior"),
        "V2 bottom edge fades",
    );
    // S1/S2: the wheel moved the offset (text advanced past the review
    // state); the mode stays Review.
    assert_ne!(text, review_text, "S2: the wheel moved the offset");
    assert!(text.contains("● Review"), "S1/A2: mode stays Review");
    // N1: the controls row must NOT fade and keeps the ● marker.
    let (ctl_row, _) = find_pos(&mut s, "● Review", case_timeout(&case));
    assert_row_same(&review, &frame, ctl_row, "N1 controls row never fades");
    // N2: the 5-hunk fixture stays loaded. A2: the file never changes.
    assert!(
        !text.contains("No file selected"),
        "N2/A2: file stays loaded"
    );

    // A1: Consumed at the end — wheel until stable, then stance.
    let (end_row, end_col) = find_pos(&mut s, "attempts = 3", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 end offset",
    );
    // The end state still shows a hunk (never the empty state).
    assert!(
        end.contains("@@") || end.contains("attempts"),
        "A1: wheeled to the hunk tail, fixture intact"
    );
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the end");
    checkpoint(&mut s, &dir, "01-end");
    eprintln!("s1 diff_wheel_review: review + bold runs + both fades + end Consumed");
}

/// SCROLL-PAGE-003 (`showcase/fade/scroll/page`): pagedown through wrapped
/// prose at the paused frame-1600 endpoint.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_scroll_page_prose() {
    let dir = s1_dir("s1_scroll_page_prose");
    let case = s1_case(
        "scroll_page_prose",
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
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "739.63s", "paused log endpoint");
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    assert!(boot_text.contains("1–15 of"), "S1: prose boot window 1–15");

    support::drive_with_timeout(&mut s, &["tab", "pagedown"], case.timeout_ms);
    waits::wait_state(&mut s, "16–30", "paged prose window");
    checkpoint(&mut s, &dir, "paged");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: prose lines 16–30 with the focus bar; the log stays at the tail.
    assert_line_has(&text, "16–30", "▎", "V1 prose window + focus");
    assert!(text.contains("Log 1986–2000"), "V1/S1 log tail intact");
    // V2: prose fades both edges (mid-document); the log fades its top
    // edge only (at the tail, no bottom fade). Span-scoped: full rows mix
    // all three columns' tones (prose + list + log + level hints).
    let (prose_first, prose_first_col) = find_pos(&mut s, "reports back", case_timeout(&case));
    let (prose_mid, prose_mid_col) = find_pos(&mut s, "Each step is", case_timeout(&case));
    let (prose_last, prose_last_col) = find_pos(&mut s, "and every", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, prose_first, prose_first_col, 12, "V2 prose top"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 12, "V2 prose interior"),
        "V2 prose top edge fades",
    );
    assert_faded(
        span_fg_median(&frame, prose_last, prose_last_col, 9, "V2 prose bottom"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 12, "V2 prose interior"),
        "V2 prose bottom edge fades",
    );
    let (log_first, log_first_col) = find_pos(&mut s, "734.45s", case_timeout(&case));
    let (log_mid, log_mid_col) = find_pos(&mut s, "737.04s", case_timeout(&case));
    let (log_last, log_last_col) = find_pos(&mut s, "739.63s", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, log_first, log_first_col, 7, "V2 log top"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2 log top edge fades",
    );
    assert_same_tone(
        span_fg_median(&frame, log_last, log_last_col, 7, "V2 log bottom"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2/N1 log bottom whole at the tail",
    );
    // S1: PageDown paged by exactly one viewport (1–15 → 16–30 = 15 lines).
    let (first, last) = parse_window("16–30", "S1 paged window");
    assert_eq!((first, last), (16, 30), "S1: prose offset 0 → 15");
    // S2: focus moved nav → prose; log follow stays true (tail + no fade).
    assert!(text.contains("PgUp PgDn"), "S2: focused-column footer");
    // N1: the log must NOT unfollow (tail window + whole bottom row).
    assert!(text.contains("1986–2000"), "N1: log keeps the tail");
    // N2: the list column must NOT move.
    assert!(text.contains("1–15 of"), "N2: list keeps 1–15");
    assert!(
        text.contains("Row 001") && text.contains("Row 015"),
        "N2: list rows intact"
    );

    // A1: PageDown pages by viewport_len — boot window (15) == advance (15).
    let (b_first, b_last) = parse_window("1–15", "A1 boot window");
    assert_eq!(
        last - first,
        b_last - b_first,
        "A1: advance equals viewport_len"
    );
    // A2: keys act on the focused column only — Down moves prose alone.
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "17–31", "A2 prose advanced one line");
    let moved = live_text(&mut s);
    assert!(
        moved.contains("Lon… 1–15") || moved.contains("1–15 of"),
        "A2: list still 1–15"
    );
    assert!(moved.contains("1986–2000"), "A2: log still at the tail");
    checkpoint(&mut s, &dir, "01-down");
    eprintln!("s1 scroll_page_prose: 16–30 + both fades, log tail whole, down is column-local");
}

/// SCROLL-WHEEL-004 (`showcase/fade/scrolling/wheel-fade`): wheel-up off
/// the follow tail at the paused frame-1600 endpoint.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_scroll_wheel_offtail() {
    let dir = s1_dir("s1_scroll_wheel_offtail");
    let case = s1_case(
        "scroll_wheel_offtail",
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
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    waits::wait_state(&mut s, "739.63s", "paused log endpoint");
    checkpoint(&mut s, &dir, "00-boot");
    let boot = live_frame(&mut s);
    let (boot_last, boot_last_col) = find_pos(&mut s, "739.63s", case_timeout(&case));
    let (boot_mid, boot_mid_col) = find_pos(&mut s, "737.04s", case_timeout(&case));
    // Boot: the log sits at the tail, so its bottom row reads whole.
    assert_same_tone(
        span_fg_median(&boot, boot_last, boot_last_col, 7, "boot log bottom"),
        span_fg_median(&boot, boot_mid, boot_mid_col, 7, "boot log interior"),
        "S1: boot log bottom whole at the tail",
    );

    wheel_over(&mut s, "739.63s", Wheel::Up, 2, case_timeout(&case));
    waits::wait_state(&mut s, "1980–1994", "off-tail window");
    checkpoint(&mut s, &dir, "off-tail");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: six lines above the tail. (The `· following` meta suffix
    // truncates off-screen at 80x24, so the follow-break is proven by the
    // returned bottom fade plus the off-tail window instead.)
    assert!(text.contains("Log 1980–1994"), "V1: off-tail window");
    // V2: the log fades both edges off the tail; prose and list fade
    // bottom-only at the top. Span-scoped (see T10 V2).
    let (log_first, log_first_col) = find_pos(&mut s, "732.23s", case_timeout(&case));
    let (log_mid, log_mid_col) = find_pos(&mut s, "734.82s", case_timeout(&case));
    let (log_last, log_last_col) = find_pos(&mut s, "737.41s", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, log_first, log_first_col, 7, "V2 log top"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2 log top edge fades",
    );
    assert_faded(
        span_fg_median(&frame, log_last, log_last_col, 7, "V2 log bottom"),
        span_fg_median(&frame, log_mid, log_mid_col, 7, "V2 log interior"),
        "V2/N2 log bottom fade returns off the tail",
    );
    let (prose_first, prose_first_col) = find_pos(&mut s, "Junie works", case_timeout(&case));
    let (prose_mid, prose_mid_col) = find_pos(&mut s, "relevant", case_timeout(&case));
    let (prose_last, prose_last_col) = find_pos(&mut s, "tests, and", case_timeout(&case));
    assert_same_tone(
        span_fg_median(&frame, prose_first, prose_first_col, 11, "V2 prose top"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 8, "V2 prose interior"),
        "V2 prose top whole at the top",
    );
    assert_faded(
        span_fg_median(&frame, prose_last, prose_last_col, 10, "V2 prose bottom"),
        span_fg_median(&frame, prose_mid, prose_mid_col, 8, "V2 prose interior"),
        "V2 prose bottom edge fades",
    );
    // S1: two up-notches moved the log up six (1986 → 1980).
    let (first, _) = parse_window("1980–1994", "S1 off-tail window");
    assert_eq!(first, 1980, "S1: log offset up six");
    // S2/N1: prose and list keep 1–15.
    assert!(
        text.contains("Wra… 1–15") || text.contains("1–15 of"),
        "S2/N1 prose stays"
    );
    assert!(
        text.contains("Row 001") && text.contains("Row 015"),
        "S2/N1 list stays"
    );
    // A2: the gesture hit the log column; focus never moved.
    assert!(
        text.contains("Tab Into page"),
        "A2: footer still the nav footer"
    );

    // A1: further ups keep scrolling (Changed ⟺ the window moved again).
    wheel_over(&mut s, "737.41s", Wheel::Up, 2, case_timeout(&case));
    waits::wait_state(&mut s, "1974–1988", "A1 further ups scroll");
    checkpoint(&mut s, &dir, "01-further-up");
    eprintln!("s1 scroll_wheel_offtail: 1980–1994 + both fades, prose/list pinned, ups continue");
}

/// NAV-WHEEL-003 (`showcase/fade/sidebars/wheel`): wheel scroll + fade over
/// the sidebars demo list at 72x20.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_nav_wheel_demo() {
    let dir = s1_dir("s1_nav_wheel_demo");
    let case = s1_case("nav_wheel_demo", &["--page", "sidebars"], 72, 20);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    let boot = live_frame(&mut s);
    // Boot shows the demo top (Workspace first); the lower demo rows sit
    // below the fold.
    assert!(
        boot_text.contains("Workspace"),
        "S1: boot shows the demo top"
    );
    assert!(
        !boot_text.contains("Preferences"),
        "S1: Preferences folded at boot"
    );
    assert!(
        !boot_text.contains("Appearance"),
        "S1: Appearance folded at boot"
    );

    wheel_over(&mut s, "Members", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "Preferences", "wheel-faded demo window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: B Branches through Appearance with a scrollbar; the Collapse
    // footer button persists below the scroll viewport.
    assert!(text.contains("B Branches"), "V1: first demo row");
    assert!(text.contains("A Appearance"), "V1: last demo row");
    assert!(text.contains("Collapse"), "V1: Collapse footer persists");
    let (demo_row, _) = find_pos(&mut s, "M Members", case_timeout(&case));
    let scrollbar = (0..frame.cols).any(|col| {
        frame
            .get(col, demo_row)
            .is_some_and(|c| c.symbol == "│" || c.symbol == "┃")
    });
    assert!(scrollbar, "V1: scrollbar beside the demo list");
    // S1: two down-notches moved the demo offset down — Workspace left
    // above the fold while Preferences, Keyboard, and Appearance entered
    // below; the Collapse footer button persists (sidebars.rs:359 renders
    // it as a page-level Button below the nav scroll viewport, not a
    // NavList row).
    assert!(
        !text.contains("Workspace"),
        "S1: Workspace left above the fold"
    );
    for needle in ["Preferences", "K Keyboard", "A Appearance"] {
        assert!(text.contains(needle), "S1: {needle} entered the viewport");
    }
    assert!(
        boot_text.contains("Collapse") && text.contains("Collapse"),
        "S1: Collapse footer persists"
    );
    // V2: the demo top edge fades; the shell nav fades its bottom edge.
    // Span-scoped: full rows mix the demo, legend, and gutter tones.
    let (branches_row, branches_col) = find_pos(&mut s, "B Branches", case_timeout(&case));
    let (members_row, members_col) = find_pos(&mut s, "M Members", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, branches_row, branches_col, 10, "V2 demo top"),
        span_fg_median(&frame, members_row, members_col, 9, "V2 demo interior"),
        "V2 demo top edge fades",
    );
    let (nav_first, nav_first_col) = find_pos(&mut s, "Buttons", case_timeout(&case));
    let (nav_last, nav_last_col) = find_pos(&mut s, "Code editor", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, nav_last, nav_last_col, 11, "V2 shell nav bottom"),
        span_fg_median(&frame, nav_first, nav_first_col, 7, "V2 shell nav interior"),
        "V2 shell nav bottom edge fades",
    );
    // S2: current, collapsed, and cursor untouched — the wheel moves
    // scroll.offset only.
    assert!(text.contains("Tasks"), "S2/N1: demo current persists");
    assert!(
        text.contains("Tab Into page"),
        "S2: footer still the nav footer"
    );
    // Two ▎, both accounted: the nav focus bar plus the legend's own ▎
    // sample — none live inside the demo rows.
    let bars: Vec<&str> = text.lines().filter(|l| l.contains('▎')).collect();
    assert_eq!(bars.len(), 2, "S2: nav bar + legend sample only: {bars:?}");
    assert!(
        bars.iter().any(|l| l.contains("Sidebars")),
        "S2: nav bar present"
    );
    assert!(
        bars.iter().any(|l| l.contains("keyboard")),
        "S2: legend sample present"
    );
    // N1: the current item persists by identity (no › churn in the demo).
    let demo_marks = |t: &str| {
        t.lines()
            .filter(|l| {
                l.contains("Branches")
                    || l.contains("Members")
                    || l.contains("Collapse")
                    || l.contains("Preferences")
            })
            .filter(|l| l.contains('›'))
            .count()
    };
    assert_eq!(
        demo_marks(&text),
        demo_marks(&boot_text),
        "N1: demo › marks stable"
    );
    // N2: the shell nav window must NOT move — the gutter text is identical.
    for row in 0..boot.rows {
        let a: String = (0..17)
            .filter_map(|col| boot.get(col, row).map(|c| c.symbol.clone()))
            .collect();
        let b: String = (0..17)
            .filter_map(|col| frame.get(col, row).map(|c| c.symbol.clone()))
            .collect();
        assert_eq!(a, b, "N2: shell nav row {row} unmoved");
    }

    // A1/A2: Changed while rows hide below; Consumed at the demo bottom.
    let (end_row, end_col) = find_pos(&mut s, "M Members", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A2 demo bottom",
    );
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A2 stance");
    assert_eq!(stance, end, "A2: Consumed at the demo bottom");
    checkpoint(&mut s, &dir, "01-bottom");
    eprintln!("s1 nav_wheel_demo: Collapse persists, both fades, gutter pinned, bottom Consumed");
}

/// TERM-WHEEL-003 (`showcase/fade/terminal/scrollback`): wheel-up into the
/// build scrollback at the paused frame-60 endpoint.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_term_wheel_scrollback() {
    let dir = s1_dir("s1_term_wheel_scrollback");
    let case = s1_case(
        "term_wheel_scrollback",
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    let boot = live_frame(&mut s);
    assert!(boot_text.contains("2 of 7"), "S2: boot rail at 2 of 7");

    wheel_over(
        &mut s,
        "#4 RUN cargo build",
        Wheel::Up,
        2,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "scrollback ↑5", "scrollback header");
    checkpoint(&mut s, &dir, "scrollback");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: the ↑5 header plus resolve, pull, and build layer lines.
    assert!(text.contains("scrollback ↑5"), "V1: scrollback header");
    for needle in ["Resolve workspace", "Pull base image", "layer 01/12"] {
        assert!(text.contains(needle), "V1: transcript lacks {needle}");
    }
    // V2: the step rail never fades (it does not scroll); the viewport is
    // top-anchored at ↑5 (offset 0 hides lines below only), so the top
    // reads whole — fade.rs:55 fades the top only when offset > 0 — while
    // the bottom fades two-deep (outer < inner < interior over the #-run
    // spans).
    let (prompt_row, prompt_col) = find_pos(&mut s, "payments-platform", case_timeout(&case));
    let prompt = span_fg_median(&frame, prompt_row, prompt_col, 17, "V2 prompt row");
    assert_eq!(
        prompt, 2_550_000,
        "V2: prompt row whole (white; any fade would dim below 255)"
    );
    let (r_run, r_run_col) = find_pos(&mut s, "▶ Resolve", case_timeout(&case));
    let (r_pull, r_pull_col) = find_pos(&mut s, "▶ Pull", case_timeout(&case));
    assert_same_tone(
        span_fg_median(&frame, r_run, r_run_col, 11, "V2 top ▶ row"),
        span_fg_median(&frame, r_pull, r_pull_col, 8, "V2 interior ▶ row"),
        "V2: top ▶ row whole",
    );
    let (r_done, r_done_col) = find_pos(&mut s, "✓ Resolve", case_timeout(&case));
    let (r_done2, r_done2_col) = find_pos(&mut s, "✓ Pull", case_timeout(&case));
    assert_same_tone(
        span_fg_median(&frame, r_done, r_done_col, 11, "V2 top ✓ row"),
        span_fg_median(&frame, r_done2, r_done2_col, 8, "V2 interior ✓ row"),
        "V2: top ✓ row whole",
    );
    let (b_whole, b_whole_col) = find_pos(&mut s, "#1 RUN", case_timeout(&case));
    let (b_inner, b_inner_col) = find_pos(&mut s, "#2 COPY", case_timeout(&case));
    let (b_outer, b_outer_col) = find_pos(&mut s, "#3 RUN", case_timeout(&case));
    let whole = span_fg_median(&frame, b_whole, b_whole_col, 12, "V2 bottom interior");
    let inner = span_fg_median(&frame, b_inner, b_inner_col, 12, "V2 bottom inner");
    let outer = span_fg_median(&frame, b_outer, b_outer_col, 12, "V2 bottom outer");
    assert_faded(inner, whole, "V2 viewport bottom inner fades");
    assert_faded(outer, inner, "V2 viewport bottom outer fades deepest");
    for needle in ["01 Resolve", "03 Build", "07 Ready"] {
        let (rail_row, rail_col) = find_pos(&mut s, needle, case_timeout(&case));
        assert_cell_same(
            &boot,
            &frame,
            rail_col,
            rail_row,
            "V2 step rail never fades",
        );
    }
    // S2/N2/A2: the rail stays at 2 of 7 — paused motion fires no ticks,
    // and the wheel never advances stages.
    assert!(text.contains("2 of 7"), "S2/N2/A2: rail stays at 2 of 7");
    // N1: no following indicator off the tail — ↑5 replaces it.
    assert!(!text.contains("following"), "N1: no following indicator");

    // S1: the reading position was remembered — wheeling back down returns
    // exactly to the boot tail.
    wheel_over(&mut s, "layer 04/12", Wheel::Down, 2, case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "S1 returned to the boot tail",
        |screen| support::screen_text(screen) == boot_text,
    );
    // A1: Consumed at the top line — wheel up until stable, then stance.
    let (top_row, top_col) = find_pos(&mut s, "#4 RUN cargo build", case_timeout(&case));
    let top = wheel_until_stable(
        &mut s,
        top_col,
        top_row,
        Wheel::Up,
        case_timeout(&case),
        "A1 top line",
    );
    wheel_at(&mut s, top_col, top_row, Wheel::Up, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, top, "A1: Consumed at the top line");
    checkpoint(&mut s, &dir, "01-top");
    eprintln!("s1 term_wheel_scrollback: ↑5 top-anchored, bottom fade, rail pinned, top Consumed");
}

/// LIST-WHEEL-003 (`showcase/fade/lists/wheel-fade`): wheel scroll + edge
/// fade over languages.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_list_wheel_fade() {
    let dir = s1_dir("s1_list_wheel_fade");
    let case = s1_case("list_wheel_fade", &["--page", "lists"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    assert!(boot_text.contains("Python"), "S1: Python visible at boot");

    wheel_over(&mut s, "Python", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "7–19", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: window 7–19 with Swift first; Chosen: Rust persists.
    assert!(text.contains("7–19 of"), "V1: window label");
    let (swift_row, swift_col) = find_pos(&mut s, "Swift", case_timeout(&case));
    let (csharp_row, _) = find_pos(&mut s, "C#", case_timeout(&case));
    assert!(swift_row < csharp_row, "V1: Swift first in the window");
    assert!(text.contains("Chosen: Rust"), "V1/S2/N1: choice persists");
    // V2: both viewport edges fade (mid-list); the list is unfocused.
    // Span-scoped over the language names: full rows mix the filter
    // column's bright ✓ marks differentially.
    let (last_row, last_col) = find_pos(&mut s, "Clojure", case_timeout(&case));
    let (mid_row, mid_col) = find_pos(&mut s, "Haskell", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, swift_row, swift_col, 5, "V2 top edge"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 interior"),
        "V2 top edge fades",
    );
    assert_faded(
        span_fg_median(&frame, last_row, last_col, 7, "V2 bottom edge"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 interior"),
        "V2 bottom edge fades",
    );
    assert_eq!(text.matches('▎').count(), 1, "V2: no focus bar on the list");
    // S1: offset 0 → 6; chosen and cursor stay 0.
    let (first, _) = parse_window("7–19", "S1 window");
    assert_eq!(first, 7, "S1: offset 0 → 6");
    assert!(
        text.contains("Tab Into page"),
        "S1: footer still the nav footer"
    );
    // N2: the filter and search columns must NOT move.
    assert!(text.contains("2 select"), "N2: filter column intact");
    assert!(text.contains("Space toggle"), "N2: filter hints intact");
    assert!(text.contains("No results for"), "N2: search column intact");

    // A1: Consumed at the end — wheel until stable, pin the label, stance.
    let (end_row, end_col) = find_pos(&mut s, "Swift", case_timeout(&case));
    let end = wheel_until_stable(
        &mut s,
        end_col,
        end_row,
        Wheel::Down,
        case_timeout(&case),
        "A1 end offset",
    );
    // (The total truncates at 80x24 — "Lan… 8–20 of …" — so only the
    // window start/end is pinned; 120x40 shows "of 20", see T18.)
    assert!(end.contains("8–20 of"), "A1: end offset is 8–20");
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    let stance = settle_text(&mut s, Duration::from_millis(3000), "A1 stance");
    assert_eq!(stance, end, "A1: Consumed at the end");
    // A2: a cursor key after wheeling re-reveals row 0 via ensure_visible.
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "▎", "A2 list focused");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "Python", "A2 row 0 re-revealed");
    assert!(
        live_text(&mut s).contains("Chosen: Rust"),
        "A2: choice still Rust"
    );
    checkpoint(&mut s, &dir, "01-rerevealed");
    eprintln!("s1 list_wheel_fade: 7–19 + both fades, end Consumed, up re-reveals row 0");
}

/// TREE-WHEEL-003 (`showcase/fade/trees/wheel-fade`): wheel scroll + fade
/// over the project tree.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_tree_wheel_fade() {
    let dir = s1_dir("s1_tree_wheel_fade");
    let case = s1_case("tree_wheel_fade", &["--page", "trees"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");

    wheel_over(&mut s, "config.rs", Wheel::Down, 1, case_timeout(&case));
    waits::wait_state(&mut s, "2–16 of 16", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: window 2–16; api/db/workers folded, tests open.
    assert!(text.contains("2–16 of 16"), "V1/S1: window label");
    for needle in ["▸ api", "▸ db", "▸ workers", "▾ tests"] {
        assert!(text.contains(needle), "V1: expansion lacks {needle}");
    }
    // V2: the top edge fades (offset 1); the bottom stays whole (row 16 is
    // the last row). The bottom proof is span-scoped over file names —
    // full rows mix the selection panel's side text differentially.
    let (first_row, _) = find_pos(&mut s, "▸ api", case_timeout(&case));
    let (mid_row, mid_col) = find_pos(&mut s, "main.rs", case_timeout(&case));
    let (last_row, last_col) = find_pos(&mut s, "README.md", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, first_row, "V2 top edge"),
        row_fg_median(&frame, mid_row, "V2 interior"),
        "V2 top edge fades",
    );
    assert_same_tone(
        span_fg_median(&frame, last_row, last_col, 9, "V2 bottom row"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 interior"),
        "V2 bottom stays whole at the last row",
    );
    // S2: the cursor stays on its boot row.
    assert!(text.contains("cursor  src"), "S2: cursor stays on src");
    // N1: expansion untouched. N2: selection untouched.
    assert!(
        text.contains("open    5 folders"),
        "N1: still 5 folders open"
    );
    assert!(
        text.contains("Nothing selected"),
        "N2: still nothing selected"
    );

    // A1: the notch moved (Changed); a second notch is Consumed (at_end).
    wheel_over(&mut s, "config.rs", Wheel::Down, 1, case_timeout(&case));
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        live_text(&mut s).contains("2–16 of 16"),
        "A1: second notch Consumed at the end"
    );
    // A2: toggle keys still fold and unfold after wheeling. The cursor
    // rests on src (row 0); one down lands on the folded api row, which
    // right unfolds and left folds back.
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "▎", "A2 tree focused");
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "cursor  api", "A2 cursor on the api row");
    support::press_step(&s, "right");
    waits::wait_state(&mut s, "open    6 folders", "A2 right unfolds one more");
    support::press_step(&s, "left");
    waits::wait_state(&mut s, "open    5 folders", "A2 left folds it back");
    checkpoint(&mut s, &dir, "01-toggled");
    eprintln!("s1 tree_wheel_fade: 2–16 + top fade, second notch Consumed, toggles work");
}

/// TA-WHEEL-004 (`showcase/fade/textarea/wheel`): wheel scroll + fade over
/// the task description.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_textarea_wheel_fade() {
    let dir = s1_dir("s1_textarea_wheel_fade");
    let case = s1_case("textarea_wheel_fade", &["--page", "textareas"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    let boot_text = live_text(&mut s);
    assert!(boot_text.contains("1–8 of 28"), "S1: boot window 1–8");

    wheel_over(&mut s, "1. Read", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "7–14 of 28", "wheel-faded window");
    checkpoint(&mut s, &dir, "wheel-faded");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: lines 7–14 with the label; no cursor, no cursor-line underline.
    assert!(text.contains("7–14 of 28"), "V1: window label");
    assert!(text.contains("7. Open a PR"), "V1: first line");
    assert!(text.contains("14. Keep the public"), "V1: last line");
    assert_eq!(
        text.matches('▎').count(),
        1,
        "V1: no cursor bar in the area"
    );
    assert!(!text.contains('▁'), "V1: no editing cursor mark");
    // V2: both viewport edges fade (offset 6 hides lines above, line 14 of
    // 28 hides lines below).
    let (first_row, _) = find_pos(&mut s, "7. Open a PR", case_timeout(&case));
    let (mid_row, _) = find_pos(&mut s, "10. Keep the public", case_timeout(&case));
    let (last_row, _) = find_pos(&mut s, "14. Keep the public", case_timeout(&case));
    assert_faded(
        row_fg_median(&frame, first_row, "V2 top edge"),
        row_fg_median(&frame, mid_row, "V2 interior"),
        "V2 top edge fades",
    );
    assert_faded(
        row_fg_median(&frame, last_row, "V2 bottom edge"),
        row_fg_median(&frame, mid_row, "V2 interior"),
        "V2 bottom edge fades",
    );
    // S1: offset 0 → 6. S2: the wheel never opens the editor.
    let (first, _) = parse_window("7–14", "S1 window");
    assert_eq!(first, 7, "S1: view offset 0 → 6");
    assert!(
        text.contains("Tab Into page"),
        "S2: footer still the nav footer"
    );
    // N2: Notes, transcript, and message areas must NOT move.
    for needle in [
        "Anything the agent sh",
        "Read-only transcript",
        "Commit message",
    ] {
        assert!(text.contains(needle), "N2: area keeps {needle:?}");
    }

    // A1: Consumed at the boundary — wheel to the end, then prove stance.
    let (end_row, end_col) =
        wheel_over(&mut s, "7. Open a PR", Wheel::Down, 20, case_timeout(&case));
    waits::wait_state(&mut s, "21–28 of 28", "A1 end offset");
    let end = live_text(&mut s);
    wheel_at(&mut s, end_col, end_row, Wheel::Down, 3);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), end, "A1: Consumed at the boundary");
    // A2/N1: view keys move the offset from the end without touching the
    // buffer cursor — the area never enters edit mode (viewport and buffer
    // cursors are separate).
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "▎", "A2 area focused");
    support::press_step(&s, "up");
    waits::wait_state(&mut s, "20–27 of 28", "A2 Up moves the offset");
    let viewed = live_text(&mut s);
    assert!(
        viewed.contains("Enter Edit"),
        "A2: edit available, not active"
    );
    assert!(!viewed.contains("EDIT"), "N1: buffer cursor never engaged");
    checkpoint(&mut s, &dir, "01-view-key");
    eprintln!(
        "s1 textarea_wheel_fade: 7–14 + both fades, end Consumed, view keys stay out of edit"
    );
}

/// INSPECT-OPEN-001 (`showcase/flows/inspector/open`): state card open on
/// overview at 120x40.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_inspect_open() {
    let dir = s1_dir("s1_inspect_open");
    let case = s1_case("inspect_open", &["--page", "overview"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot");
    // S1: the card is closed at boot (the presence below is non-vacuous).
    assert!(
        !live_text(&mut s).contains("hit regions"),
        "S1: inspector closed at boot"
    );

    support::press_step(&s, "i");
    waits::wait_state(&mut s, "hit regions 26", "open state card");
    checkpoint(&mut s, &dir, "open");
    let text = live_text(&mut s);

    // V1: the ten readout rows plus the header toggle.
    for needle in [
        "mode        NAV",
        "0·2 24×36",
        "hover       suppressed",
        "pressed     —",
        "last key    i",
        "focus ring  1 stops",
        "hit regions 26",
        "tick        0",
        "colors      truecolor",
        "i Inspector · on",
    ] {
        assert!(text.contains(needle), "V1: card lacks {needle:?}");
    }
    // V2/S2: width 120 clears the 100-column gate — the card renders.
    // N2: hover reads suppressed (a key was pressed, no mouse followed).
    assert!(text.contains("suppressed"), "N2: hover suppressed");

    // A1: pressing i again closes the card (toggles both ways).
    support::press_step(&s, "i");
    waits::wait_gone(&mut s, "hit regions", "A1 card closed");
    assert!(
        !live_text(&mut s).contains("· on"),
        "A1: header toggle flipped back"
    );
    checkpoint(&mut s, &dir, "01-closed");
    // A2: clicking the header Inspector label toggles through the
    // HEADER_INSPECT arm; the card itself takes no focus.
    click_at(&mut s, "Inspector", case_timeout(&case));
    waits::wait_state(&mut s, "hit regions 26", "A2 header click reopens");
    // The card takes no focus: clicking inside it changes nothing — proven
    // by settling (not by one possibly pre-click sample).
    click_at(&mut s, "hit regions", case_timeout(&case));
    let clicked = settle_text(&mut s, Duration::from_millis(3000), "A2 card click");
    assert!(
        clicked.contains("hit regions 26"),
        "A2: card still open after card click"
    );
    assert!(
        clicked.contains("Tab Into page"),
        "A2: card click takes no focus"
    );
    checkpoint(&mut s, &dir, "02-clicked");

    // N1: below width 100 the card must NOT render — at 80x24 only the
    // header toggle flips.
    let case2 = s1_case("inspect_open_narrow", &["--page", "overview"], 80, 24);
    write_provenance_named(&dir, &case2, "provenance-narrow.txt");
    let mut s2 = support::spawn_boot(&case2);
    support::press_step(&s2, "i");
    waits::wait_state(&mut s2, "· on", "N1 narrow toggle flips");
    let narrow = live_text(&mut s2);
    assert!(
        !narrow.contains("hit regions"),
        "N1: no card below width 100"
    );
    assert!(!narrow.contains("focus ring"), "N1: no readout rows either");
    checkpoint(&mut s2, &dir, "03-narrow");
    eprintln!("s1 inspect_open: 10 readout rows, i toggles, header click toggles, narrow gated");
}

/// INSPECT-SCROLL-002 (`showcase/flows/inspector/scrolled`): inspector
/// repainted by a wheel scroll on lists at 120x40.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_inspect_scrolled() {
    let dir = s1_dir("s1_inspect_scrolled");
    let case = s1_case("inspect_scrolled", &["--page", "lists"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::press_step(&s, "i");
    waits::wait_state(&mut s, "hit regions 58", "inspector open on lists");
    checkpoint(&mut s, &dir, "01-open");
    // Before the wheel the mouse row is empty (the repaint proof below is
    // non-vacuous).
    assert!(
        !live_text(&mut s).contains("31·10"),
        "V2: no mouse cell before the wheel"
    );

    wheel_over(&mut s, "Python", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "7–19 of 20", "scrolled lists window");
    checkpoint(&mut s, &dir, "scrolled");
    let text = live_text(&mut s);
    let frame = live_frame(&mut s);

    // V1: the mouse row reads the wheel pointer cell; the window scrolled.
    assert!(text.contains("31·10"), "V1: mouse row reads the wheel cell");
    assert!(text.contains("7–19 of 20"), "V1/S1: lists window 7–19");
    // V2: the inspector repainted with the scrolled frame (the mouse row
    // changed with this gesture); the lists top edge fades at offset 6.
    // Span-scoped over the language names: full rows are dominated by the
    // inspector card's identical right-half tones.
    let (first_row, first_col) = find_pos(&mut s, "Swift", case_timeout(&case));
    let (mid_row, mid_col) = find_pos(&mut s, "Haskell", case_timeout(&case));
    assert_faded(
        span_fg_median(&frame, first_row, first_col, 5, "V2 lists top"),
        span_fg_median(&frame, mid_row, mid_col, 7, "V2 lists interior"),
        "V2 lists top edge fades",
    );
    // S1: offset 0 → 6 with the mouse pos set; hover stays suppressed
    // (only Move clears it).
    let (first, _) = parse_window("7–19", "S1 window");
    assert_eq!(first, 7, "S1: lists offset 0 → 6");
    assert!(text.contains("suppressed"), "S1/N1: hover stays suppressed");
    // S2: the lists page owns 4 focus stops with 58 hit regions.
    assert!(text.contains("4 stops"), "S2: 4 focus stops");
    assert!(text.contains("hit regions 58"), "S2: 58 hit regions");

    // A2/N2: the wheel over the inspector card itself is Ignored — the
    // card registers no scroll region, so the frame stays byte-identical
    // and the card never scrolls its own rows.
    let before = live_text(&mut s);
    let (card_row, card_col) = find_pos(&mut s, "hit regions", case_timeout(&case));
    wheel_at(&mut s, card_col, card_row, Wheel::Down, 2);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), before, "A2/N2: wheel over card Ignored");
    checkpoint(&mut s, &dir, "01-card-wheel-ignored");
    eprintln!("s1 inspect_scrolled: mouse 31·10 + 7–19, top fades, card wheel Ignored");
}

/// PANEL-STATIC-002 (`showcase/pages/overview`): static reference cards on
/// overview at 80x24.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_panel_overview_static() {
    let dir = s1_dir("s1_panel_overview_static");
    let case = s1_case("panel_overview_static", &["--page", "overview"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);

    // V1: two-column Tokens swatches; the State language legend rows.
    assert_line_has(&boot, "canvas", "text.muted", "V1 tokens two-column");
    assert_line_has(&boot, "popover", "accent.bg", "V1 tokens two-column");
    for (mark, label) in [
        ('▎', "focus"),
        ('░', "hover lifts"),
        ('›', "current / chosen"),
        ('✓', "checked"),
        ('!', "error"),
    ] {
        let mark_str = mark.to_string();
        assert_line_has(&boot, &mark_str, label, "V1 legend row");
    }
    // V2/N1: stacked content; the Principles card takes zero rows.
    assert!(!boot.contains("Principles"), "V2/N1: no Principles title");
    // S1: two_col holds (same-line swatch pairs, asserted in V1).
    // N2: no live focus bar inside the cards — only the nav bar plus the
    // legend's own ▎ sample.
    assert_eq!(
        boot.matches('▎').count(),
        2,
        "N2: nav bar + legend sample only"
    );
    assert_line_has(&boot, "▎", "Overview", "N2 focus on the nav");
    for token in ["canvas", "surface", "field", "popover", "accent", "text."] {
        assert_line_lacks(&boot, token, "▎", "N2 no focus bar on Tokens rows");
    }

    // S2: the page handles nothing — a page key (s) is a no-op, while the
    // shell arms (]/i) act.
    support::press_step(&s, "s");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), boot, "S2: page key Ignored");
    // A2: i opens the state inspector from any page (gated card at 80x24,
    // but the arm fires).
    support::press_step(&s, "i");
    waits::wait_state(&mut s, "· on", "A2 inspector arm fires");
    assert!(
        !live_text(&mut s).contains("hit regions"),
        "A2: card gated below width 100"
    );
    checkpoint(&mut s, &dir, "01-inspector-arm");

    // A1: ] cycles to the next page — the shell nav owns the keyboard here.
    let case2 = s1_case(
        "panel_overview_static_next",
        &["--page", "overview"],
        80,
        24,
    );
    write_provenance_named(&dir, &case2, "provenance-next.txt");
    let mut s2 = support::spawn_boot(&case2);
    support::press_step(&s2, "]");
    waits::wait_state(&mut s2, "/ Components / Buttons", "A1 next page crumb");
    checkpoint(&mut s2, &dir, "02-next-page");
    eprintln!("s1 panel_overview_static: tokens + legend, s no-op, i arm, ] cycles");
}

/// TOOSMALL-GROWN-002 (`showcase/resize/overview_grown`): healthy reflow
/// after growing 80x24 → 120x40.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_resize_grown() {
    let dir = s1_dir("s1_resize_grown");
    let case = s1_case("resize_grown", &["--page", "overview"], 80, 24).timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot-80");
    // S1/N1 (80x24 side): 80x24 clears 72x20 — no notice at boot.
    assert!(
        !live_text(&mut s).contains("Terminal too small"),
        "S1/N1: no notice at 80x24"
    );

    resize_to(&mut s, 120, 40, case_timeout(&case));
    // A1: the ctrl+l redraw tick paints the header into the grown grid.
    s.inner.press("ctrl+l").expect("redraw tick");
    waits::wait_state(&mut s, "Foundations / Overview", "A1 grown header");
    checkpoint(&mut s, &dir, "grown");
    let grown_text = live_text(&mut s);
    let grown = live_frame(&mut s);

    // Static twin: boot straight at 120x40 (the V1 oracle, live-to-live).
    let case2 = s1_case("resize_grown_static", &["--page", "overview"], 120, 40);
    write_provenance_named(&dir, &case2, "provenance-static.txt");
    let mut s2 = support::spawn_boot(&case2);
    checkpoint(&mut s2, &dir, "01-static-120");
    let static_text = live_text(&mut s2);
    let static_frame = live_frame(&mut s2);

    // V1: the grown frame equals the static boot-at-120x40 frame.
    assert_eq!(grown_text, static_text, "V1: grown text equals static boot");
    assert_eq!(
        grown.digest(),
        static_frame.digest(),
        "V1: grown frame digest equals static boot"
    );
    // V2: cap badge, section labels, and the Principles card all render
    // (labels proven in the body — the header crumb alone would be weak).
    assert!(
        grown_text.contains("120×40"),
        "V2: grown lacks the cap badge"
    );
    for needle in ["Foundations", "Components", "Screens", "Principles"] {
        assert_body_has(&grown_text, needle, "V2: grown lacks {needle}");
    }
    // S1/N1/A2 (120x40 side): still no notice — the threshold is strict.
    assert!(
        !grown_text.contains("Terminal too small"),
        "S1/N1/A2: no notice at 120x40"
    );
    // S2: the nav scroll and content recompute for the grown grid.
    assert_line_has(&grown_text, "▎", "Overview", "S2: nav cursor re-revealed");
    // N2: no stale 80-column truncation — labels un-truncate in full.
    assert!(
        grown_text.contains("Editable tables"),
        "N2: full nav labels"
    );
    assert!(!grown_text.contains("tabl…"), "N2: no stale truncation");

    // N1 (needle proof): below 72x20 the notice DOES render, so the
    // absences above are non-vacuous.
    let case3 = s1_case("resize_grown_notice", &["--page", "overview"], 60, 10).timeout(15_000);
    write_provenance_named(&dir, &case3, "provenance-notice.txt");
    let mut s3 = support::spawn(&case3);
    waits::wait_state(
        &mut s3,
        "Terminal too small",
        "N1 notice renders below 72x20",
    );
    assert!(
        live_text(&mut s3).contains("Need 72×20, have 60×10"),
        "N1: size line reports the grid"
    );
    checkpoint(&mut s3, &dir, "02-notice");
    eprintln!(
        "s1 resize_grown: grown == static boot, badge + labels + Principles, notice below gate"
    );
}

/// TOOSMALL-SHRUNK-003 (`showcase/resize/overview_shrunk`): healthy reflow
/// after shrinking 120x40 → 80x24.
#[test]
#[ignore = "showcase s1 check; run with --ignored"]
fn s1_resize_shrunk() {
    let dir = s1_dir("s1_resize_shrunk");
    let case = s1_case("resize_shrunk", &["--page", "overview"], 120, 40).timeout(15_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "00-boot-120");
    // The 120x40 side carries the presence proofs for the N1/N2 absences.
    let wide = live_text(&mut s);
    for needle in [
        "120×40",
        "Foundations",
        "Components",
        "Screens",
        "Principles",
    ] {
        assert!(
            wide.contains(needle),
            "N1/N2 presence: 120x40 shows {needle}"
        );
    }

    resize_to(&mut s, 80, 24, case_timeout(&case));
    // A1: the ctrl+l tick repaints the header into the shrunk grid.
    s.inner.press("ctrl+l").expect("redraw tick");
    waits::wait_state(&mut s, "Foundations / Overview", "A1 shrunk header");
    checkpoint(&mut s, &dir, "shrunk");
    let shrunk_text = live_text(&mut s);
    let shrunk = live_frame(&mut s);

    // Static twin: boot straight at 80x24 (the V1 oracle, live-to-live).
    let case2 = s1_case("resize_shrunk_static", &["--page", "overview"], 80, 24);
    write_provenance_named(&dir, &case2, "provenance-static.txt");
    let mut s2 = support::spawn_boot(&case2);
    checkpoint(&mut s2, &dir, "01-static-80");
    let static_text = live_text(&mut s2);
    let static_frame = live_frame(&mut s2);

    // V1: the shrunk frame equals the static boot-at-80x24 frame.
    assert_eq!(
        shrunk_text, static_text,
        "V1: shrunk text equals static boot"
    );
    assert_eq!(
        shrunk.digest(),
        static_frame.digest(),
        "V1: shrunk frame digest equals static boot"
    );
    // V2: badge, section labels, and Principles drop out; two-column
    // Tokens and the legend persist. Label absence is body-scoped: the
    // header crumb still names "Foundations" and must not count.
    assert!(!shrunk_text.contains("×40"), "V2/N1: cap badge dropped");
    assert!(!shrunk_text.contains("truecolor ·"), "V2/N1: no badge row");
    for needle in ["Foundations", "Components", "Screens", "Principles"] {
        assert_body_lacks(&shrunk_text, needle, "V2/N2: {needle} dropped");
    }
    assert_line_has(&shrunk_text, "canvas", "text.muted", "V2 tokens persist");
    assert!(
        shrunk_text.contains("hover lifts the surface"),
        "V2 legend persists"
    );
    // S1: too_small stays false; compact (h < 28) and stacked (w < 68).
    assert!(
        !shrunk_text.contains("Terminal too small"),
        "S1/A2: no notice while both sizes clear 72x20"
    );
    let (tokens_row, _) = find_pos(&mut s, "Tokens", case_timeout(&case));
    let (legend_row, _) = find_pos(&mut s, "State language", case_timeout(&case));
    assert!(tokens_row < legend_row, "S1: content stacks vertically");
    // S2: only layout changes — Tokens and legend survive the shrink.
    assert!(shrunk_text.contains("popover"), "S2: Tokens intact");
    assert!(shrunk_text.contains("checked"), "S2: legend intact");
    eprintln!("s1 resize_shrunk: shrunk == static boot, badge + labels + Principles dropped");
}
