//! Showcase pending slice 8A-S5 executable checks (VB Phase-8ac).
//!
//! One ignored test per S5 registry row (14 rows: the last 14 unrun
//! showcase rows in registry order after S4 — the 3 FEEDBACK hint rows
//! plus the 11 DATA-VIEWS rows), over the real `showcase` binary built
//! from this worktree's verified VB sources
//! (`env!("CARGO_BIN_EXE_showcase")`) and — for the 13 rows whose
//! component lives in the `junie_tui` library — over the real production
//! widgets rendered headless. Each test executes its row's listed adapters
//! (`headless-tick`, `pty-capture`, or both) and asserts its visual (V),
//! state (S), action (A), and negative (N) checks in executable form:
//!
//! - Isolated path: the production widget renders into a `·`-sentinel
//!   buffer (any 1-cell overflow fails the surround check), so glyphs,
//!   tones, bold, widths, hit regions, focus-ring stops, and `Outcome`
//!   enums are asserted exactly.
//! - PTY path: the live app boots the row's page and drives the row's
//!   inputs; V cells are probed live (needles plus fg/bg/bold cell
//!   reads), S labels are read live, A outcomes are proven by their
//!   screen correlates (moved, committed, reverted, byte-identical),
//!   since PTY cannot see the enum, and every N absence is paired with
//!   a presence proof in the same test so no absence passes vacuously.
//!
//! One row is PTY-only by construction: TERM-RUN-001 (the stage engine,
//! reset, and advance live in the showcase binary's `terminal.rs`).
//! None of it is importable from this harness crate, so no headless path
//! exists for it; every one of its checks has a live-PTY executable form
//! instead.
//!
//! No new snapshots and no new static captures: every S5 row already has
//! approved frames gated cell-exact by the ported matrices (`showcase.rs`,
//! `pointer.rs`, `audit.rs`), so every case here uses owned (dynamic)
//! names and stays out of the `snapshots/` inventory.
//!
//! Per-test scratch lands under this harness crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `provenance.txt` (binary path, size,
//! mtime, argv), and `isolated-*.json` (headless captures). Typed input
//! is synthetic and in-memory only (simulation data).
//!
//! Row → test map (registry id → `s5_*` test):
//!
//! - HB-LAYERS-001 → [`s5_hint_layers`]
//! - KH-STATUS-001 → [`s5_keyhint_status`]
//! - KH-DROP-002 → [`s5_keyhint_drop`]
//! - GRID-EDIT-001 → [`s5_grid_edit`]
//! - GRID-PENDING-002 → [`s5_grid_pending`]
//! - GRID-COMMIT-003 → [`s5_grid_commit`]
//! - CODE-EDIT-001 → [`s5_code_edit`]
//! - CODE-RUN-002 → [`s5_code_run`]
//! - DIFF-UNIFIED-001 → [`s5_diff_unified`]
//! - DIFF-REVIEW-002 → [`s5_diff_review`]
//! - TERM-RUN-001 → [`s5_term_run`] (PTY-only)
//! - TERM-FOLLOW-002 → [`s5_term_follow`]
//! - SCROLL-REGION-001 → [`s5_scroll_region`]
//! - SCROLL-TRACK-002 → [`s5_scroll_track`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusRing;
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::{BadgeKind, SyntaxTone, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::code::{CodeEditor, Diagnostic, EditorEvent, Severity};
use junie_tui::widgets::diff::{
    DiffFile, DiffHunk, DiffLine, DiffMode, DiffStatus, DiffView, review_lines, unified_lines,
};
use junie_tui::widgets::grid::{
    CellKind, CellValue, ColumnSpec, DataGrid, GridEvent, GridRows, RowState, RowTotal,
};
use junie_tui::widgets::hintbar::{HintBar, HintLayer};
use junie_tui::widgets::keyhint::{self, hint};
use junie_tui::widgets::panel::ScrollPanel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::viewport::{Line, Span, TextViewport, ViewportEvent, line_text};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};
use tuiscotti::tui::{MouseButton, MouseMods, Session, Wheel};
use tuiscotti::{Frame, Provenance};

use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

/// Owned-name S5 case at the row's viewport (truecolor, like every S5 row).
fn s5_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s5/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

/// Unique scratch dir for one S5 test (created, never shared).
fn s5_dir(test: &str) -> PathBuf {
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
    eprintln!("s5 provenance ({file}): {body}");
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

/// One fresh live frame (cell styles included).
fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    support::frame_from_screen(
        &screen,
        Provenance::now("tuiscotti-default", "s5-checks", vec![]),
    )
}

/// Poll until two consecutive samples agree (bounded, fail-closed).
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

/// Primary-button click on `needle`'s cell (split press + release).
fn click_at(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    Input::down(MouseButton::Left, col, row).send(s);
    Input::up(MouseButton::Left, col, row).send(s);
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

/// `notches` wheel steps over `needle`'s cell, paced like a send step.
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

/// Type literal `text` into the live session, paced like a send step.
fn type_text(s: &mut Session, text: &str) {
    s.send_text(text).expect("type_text");
    std::thread::sleep(Duration::from_millis(120));
}

/// Press one key, paced like a send step: sequential unpaced presses
/// overrun the app's per-frame input drain, so every live key goes
/// through here (the `drive` vocabulary keeps its own pacing).
fn press(s: &mut Session, key: &str) {
    support::press_step(s, key);
    std::thread::sleep(Duration::from_millis(120));
}

/// Tab until the footer shows `needle` (bounded): the focus-seeking form
/// for pages whose tab order is asserted hop by hop.
fn tab_until(s: &mut Session, needle: &str, timeout: Duration, what: &str) {
    let deadline = std::time::Instant::now() + timeout;
    for _ in 0..8 {
        if live_text(s).contains(needle) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "{what}: `{needle}` never appeared"
        );
        support::press_step(s, "tab");
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(
        live_text(s).contains(needle),
        "{what}: `{needle}` never appeared after 8 tabs"
    );
}

/// Wait (bounded) for `needle` to leave the screen, WITHOUT the
/// presence gate: for closings whose presence the preceding asserts
/// already proved on a settled screen. `wait_gone` re-samples presence
/// first, so a dialog Esc-closed 120 ms earlier (every `press` settles)
/// fails it vacuously — the open-wait plus content asserts above are the
/// presence proof, this is only the expiry.
fn wait_absent(s: &mut Session, needle: &str, timeout: Duration, what: &str) {
    support::wait_screen(
        s,
        timeout,
        &format!("{what}: `{needle}` never expired"),
        |screen| !support::screen_text(screen).contains(needle),
    );
}

/// Assert one screen line contains both `needle` and `also`.
fn assert_line_has(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: no line holds both {needle:?} and {also:?}\n{text}"
    );
}

/// Assert no screen line contains both `needle` and `also`.
fn assert_line_lacks(text: &str, needle: &str, also: &str, what: &str) {
    assert!(
        !text.lines().any(|l| l.contains(needle) && l.contains(also)),
        "{what}: a line holds both {needle:?} and {also:?}\n{text}"
    );
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

// ------------------------------------------------------- isolated harness --

fn s5_key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn s5_key_mods(code: KeyCode, mods: KeyModifiers) -> Key {
    Key { code, mods }
}

fn s5_id(name: &str) -> WidgetId {
    WidgetId::of(name)
}

/// Owned render stage: theme plus fresh registries per render.
struct Stage {
    theme: Theme,
    hits: HitRegistry,
    ring: FocusRing,
}

impl Stage {
    fn new() -> Self {
        Self {
            theme: Theme::junie(),
            hits: HitRegistry::default(),
            ring: FocusRing::default(),
        }
    }

    fn ctx(&mut self, ix: Interaction) -> RenderCtx<'_> {
        RenderCtx::new(&self.theme, ix, &mut self.hits, &mut self.ring)
    }

    fn bg(&self) -> ratatui::style::Color {
        self.theme.canvas
    }
}

/// Buffer pre-filled with `·` sentinels; the widget renders into a sub-area
/// and every cell outside must still be `·` (no 1-cell overflow).
fn sentinel_buf(cols: u16, rows: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, cols, rows));
    for y in 0..rows {
        for x in 0..cols {
            buf[(x, y)].set_symbol("·");
        }
    }
    buf
}

fn assert_surround_intact(buf: &Buffer, area: Rect, what: &str) {
    let (w, h) = (buf.area.width, buf.area.height);
    for y in 0..h {
        for x in 0..w {
            if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
                continue;
            }
            assert_eq!(
                buf[(x, y)].symbol(),
                "·",
                "{what}: surround cell ({x},{y}) touched"
            );
        }
    }
}

fn buf_row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_owned())
        .collect()
}

fn buf_is_bold(buf: &Buffer, x: u16, y: u16) -> bool {
    buf[(x, y)].modifier.contains(Modifier::BOLD)
}

/// Cell column of `needle`'s first occurrence in `line` (char-based: byte
/// offsets lie when multibyte sentinels or glyphs precede the needle).
fn char_col(line: &str, needle: &str) -> u16 {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {line:?}"));
    line[..byte].chars().count() as u16
}

/// Headless capture plus JSON evidence in the test dir.
fn capture_isolated(dir: &Path, name: &str, buf: &Buffer) -> Frame {
    let frame = support::capture_buffer(buf, Provenance::now("default", "s5-isolated", vec![]));
    std::fs::write(
        dir.join(format!("isolated-{name}.json")),
        frame.to_json_pretty(),
    )
    .unwrap_or_else(|e| panic!("write isolated-{name}.json: {e}"));
    eprintln!("isolated {name}: digest {:016x}", frame.digest());
    frame
}

/// HB-LAYERS-001 (`showcase/pages/chrome`, 80x24): the shell-owned hint
/// surface — layer precedence, the zoom badge, and the pinned status.
///
/// Isolated path proves [`HintBar::resolve`] order (menu › context ›
/// screen › empty), the badge-preserving builder, the menu row's exact
/// glyphs, and the single-row shape (a modal wins the one row; there is
/// no second row to paint). Live path boots the chrome page, focuses the
/// bar (bar layer), opens the View menu (menu layer), and chooses Zoom
/// pane (badge + status, screen layer restored).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_hint_layers() {
    let dir = s5_dir("s5_hint_layers");
    let menu = HintLayer::new(vec![
        hint("↑↓", "Move"),
        hint("← →", "Switch menu"),
        hint("Enter", "Choose"),
        hint("Esc", "Close"),
    ]);
    let context = HintLayer::new(vec![
        hint("↑↓", "Move"),
        hint("Enter", "Choose"),
        hint("Esc", "Close"),
    ]);
    let screen = HintLayer::new(vec![
        hint("↑↓", "Move"),
        hint("m", "Context menu"),
        hint("right-click", "Context menu"),
        hint("Tab", "Next"),
    ])
    .status("last: nothing yet", Tone::Secondary);

    // S1: resolve takes the first present layer: menu, context, screen.
    assert_eq!(
        HintBar::resolve(&[
            Some(menu.clone()),
            Some(context.clone()),
            Some(screen.clone())
        ]),
        menu,
        "S1: menu wins over context and screen"
    );
    assert_eq!(
        HintBar::resolve(&[None, Some(context.clone()), Some(screen.clone())]),
        context,
        "S1: context wins over screen"
    );
    assert_eq!(
        HintBar::resolve(&[None, None, Some(screen.clone())]),
        screen,
        "S1: screen wins over nothing"
    );
    assert_eq!(
        HintBar::resolve(&[None, None, None]),
        HintLayer::default(),
        "S1: no layer is the empty fallback"
    );
    // S2: zooming adds the badge without changing the hints.
    let zoomed = screen.clone().badge("ZOOM", BadgeKind::Edit);
    assert_eq!(zoomed.hints, screen.hints, "S2: badge keeps the hints");
    assert_eq!(zoomed.status, screen.status, "S2: badge keeps the status");

    // V1/V2 render form: the menu row's glyphs plus the badge lead.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 3);
    let area = Rect::new(0, 1, 80, 1);
    let n = HintBar::render(area, &mut buf, &stage.theme, &menu);
    assert_eq!(n, 4, "V1: all four menu hints fit");
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("Move")
            && row.contains("Switch menu")
            && row.contains("Choose")
            && row.contains("Close"),
        "V1: the menu layer swaps the footer\n{row:?}"
    );
    assert_surround_intact(&buf, area, "N1: one row only");
    capture_isolated(&dir, "hintbar-menu", &buf);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 3);
    let area = Rect::new(0, 1, 80, 1);
    HintBar::render(area, &mut buf, &stage.theme, &zoomed);
    let row = buf_row_text(&buf, 1);
    assert!(row.contains(" ZOOM "), "V2: the ZOOM badge leads\n{row:?}");
    assert!(
        row.contains("last: nothing yet"),
        "V2: `last: …` keeps the right edge\n{row:?}"
    );
    assert_eq!(
        char_col(&row, "last:"),
        80 - "last: nothing yet".chars().count() as u16 - 1,
        "V2: the status pins the right edge minus one"
    );
    assert_surround_intact(&buf, area, "N1: one row only");

    // PTY: base → bar layer → menu layer → zoomed screen layer.
    let case = s5_case("hint_layers", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "base");
    let (base_row, _) = find_pos(&mut s, "last: nothing yet", case_timeout(&case));
    // Row 21, not 23: the page footer sits two rows up while the app's
    // own hint bar owns the bottom row.
    assert_eq!(base_row, 21, "N2: the footer sits above the app hints");
    let base = live_text(&mut s);
    assert!(
        base.contains("Context menu"),
        "base: screen hints render (presence)"
    );
    // A2: focus on the menu bar swaps to the bar layer.
    press(&mut s, "tab");
    waits::wait_state(&mut s, "← → Menu", "A2 live: bar layer swaps in");
    checkpoint(&mut s, &dir, "bar-focused");
    // V1 live: an open menu swaps the footer to the menu layer.
    press(&mut s, "enter");
    waits::wait_state(&mut s, "New tab", "File menu never opened");
    press(&mut s, "right");
    waits::wait_state(&mut s, "Zoom pane", "View menu never opened");
    checkpoint(&mut s, &dir, "menu");
    let (menu_row, _) = find_pos(&mut s, "Switch menu", case_timeout(&case));
    assert_eq!(
        menu_row, base_row,
        "N2: the footer never moves between layers"
    );
    let open = live_text(&mut s);
    assert_line_has(&open, "Switch menu", "Close", "V1 live: menu layer");
    // A1 + V2 + S2 live: choosing Zoom pane closes the menu (screen layer
    // back), adds the badge, and keeps the hints and the right edge.
    press(&mut s, "enter");
    // Choosing leaves focus on the bar, whose layer carries no status —
    // one tab (any non-bar stop) restores the screen layer with `last:`.
    press(&mut s, "tab");
    waits::wait_state(&mut s, "last: View › Zoom pane", "A1 live: choice records");
    checkpoint(&mut s, &dir, "zoomed");
    let (zoom_row, _) = find_pos(&mut s, "ZOOM", case_timeout(&case));
    assert_eq!(
        zoom_row, base_row,
        "N2: the footer never moves between layers"
    );
    let zoomed_text = live_text(&mut s);
    // The badge steals the room `Context menu` needs, so the footer
    // keeps the first hint, marks the cut, and holds the right edge.
    assert_line_has(
        &zoomed_text,
        "ZOOM",
        "Move",
        "V2/S2 live: badge + first hint",
    );
    assert_line_has(&zoomed_text, "ZOOM", "…", "V2/S2 live: badge + marked cut");
    assert!(
        zoomed_text.contains("last: View › Zoom pane"),
        "V2 live: `last: …` keeps the right edge"
    );
    eprintln!("s5 hint_layers: resolve order + live base/bar/menu/zoom footer");
}

/// KH-STATUS-001 (`showcase/pages/chrome`, 80x24): the toned status that
/// keeps the right edge of the hint row.
///
/// Isolated path renders [`keyhint::render_toned`] into sentinel buffers:
/// error/warning marks with bold mark cells in tone, the plain secondary
/// form, the mark-follows-tone table, the width-plus-3 hint reservation,
/// and the only-the-mark-is-bold scan. Live path boots the chrome page,
/// runs two menu actions (status replacement), and clicks the status
/// (no input: byte-identical screen).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_keyhint_status() {
    let dir = s5_dir("s5_keyhint_status");
    let stage = Stage::new();
    let t = &stage.theme;
    let hints = [hint("↑↓", "Move"), hint("m", "Context menu")];

    // V1: an error status renders `! message` in error with a bold mark;
    // a warning renders `▲ message`.
    let mut buf = sentinel_buf(60, 3);
    let area = Rect::new(0, 1, 60, 1);
    keyhint::render_toned(
        area,
        &mut buf,
        t,
        &hints,
        None,
        Some(("disk low", Tone::Error)),
    );
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("! disk low"),
        "V1: error mark + message\n{row:?}"
    );
    let mark_x = char_col(&row, "!");
    assert!(buf_is_bold(&buf, mark_x, 1), "V1: the error mark is bold");
    assert_eq!(
        buf[(mark_x, 1)].fg,
        t.error,
        "V1: the error mark takes error"
    );
    assert_eq!(
        buf[(mark_x + 2, 1)].fg,
        t.error,
        "V1: the message takes error too"
    );
    assert!(
        !buf_is_bold(&buf, mark_x + 2, 1),
        "N2: the message is never bold"
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "keyhint-error", &buf);
    let mut buf = sentinel_buf(60, 3);
    keyhint::render_toned(
        area,
        &mut buf,
        t,
        &hints,
        None,
        Some(("disk low", Tone::Warning)),
    );
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("▲ disk low"),
        "V1: warning mark + message\n{row:?}"
    );
    let mark_x = char_col(&row, "▲");
    assert!(buf_is_bold(&buf, mark_x, 1), "V1: the warning mark is bold");
    assert_eq!(
        buf[(mark_x, 1)].fg,
        t.warning,
        "V1: the warning mark takes warning"
    );
    // V2 + N1: a plain secondary status renders unmarked in its tone and
    // never carries ▲ or !.
    let mut buf = sentinel_buf(60, 3);
    keyhint::render_toned(
        area,
        &mut buf,
        t,
        &hints,
        None,
        Some(("last: View › Zoom pane", Tone::Secondary)),
    );
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("last: View › Zoom pane"),
        "V2: plain status renders\n{row:?}"
    );
    assert!(
        !row.contains('▲') && !row.contains('!'),
        "N1: a plain status never carries ▲ or !\n{row:?}"
    );
    let status_x = char_col(&row, "last:");
    assert_eq!(
        buf[(status_x, 1)].fg,
        t.tone(Tone::Secondary),
        "V2: the plain status takes its tone"
    );
    assert!(
        (status_x..60).all(|x| !buf_is_bold(&buf, x, 1)),
        "N2: no bold anywhere in a plain status"
    );
    // S1: the mark follows the tone: Error !, Warning ▲, else none.
    for (tone, mark) in [
        (Tone::Error, Some("!")),
        (Tone::Warning, Some("▲")),
        (Tone::Secondary, None),
        (Tone::Normal, None),
        (Tone::Muted, None),
        (Tone::Faint, None),
        (Tone::Success, None),
    ] {
        let mut buf = sentinel_buf(60, 1);
        keyhint::render_toned(
            Rect::new(0, 0, 60, 1),
            &mut buf,
            t,
            &hints,
            None,
            Some(("msg", tone)),
        );
        let row = buf_row_text(&buf, 0);
        match mark {
            Some(m) => assert!(
                row.contains(&format!("{m} msg")),
                "S1: {tone:?} carries {m:?}\n{row:?}"
            ),
            None => {
                assert!(row.contains("msg"), "S1: {tone:?} renders\n{row:?}");
                assert!(
                    !row.contains('▲') && !row.contains('!'),
                    "S1: {tone:?} carries no mark\n{row:?}"
                );
            }
        }
    }
    // S2: the status reserves its width plus 3 from the hint limit — the
    // hint block ends where the status reservation begins, never under it.
    let many = [
        hint("↑↓", "Move"),
        hint("m", "Context menu"),
        hint("right-click", "Context menu"),
        hint("Tab", "Next"),
    ];
    let mut buf = sentinel_buf(80, 1);
    let plain = keyhint::render_toned(Rect::new(0, 0, 80, 1), &mut buf, t, &many, None, None);
    assert_eq!(plain, 4, "S2: all four hints fit with no status");
    let mut buf = sentinel_buf(80, 1);
    let drawn = keyhint::render_toned(
        Rect::new(0, 0, 80, 1),
        &mut buf,
        t,
        &many,
        None,
        Some(("last: View › Zoom pane", Tone::Secondary)),
    );
    assert!(drawn < 4, "S2: the status reservation drops hints");
    let row = buf_row_text(&buf, 0);
    let status_x = char_col(&row, "last:");
    let w = u16::try_from("last: View › Zoom pane".chars().count()).unwrap();
    assert_eq!(
        status_x,
        80 - w - 1,
        "S2: the status pins the right edge minus one"
    );
    // The reservation is status + 3: complete hints stop 3 cells shy
    // (`limit`), but the cut marker may spend the last two, so the block
    // as drawn ends exactly where the status begins.
    let limit = 80 - (w + 3);
    let head: String = row.chars().take(status_x as usize).collect();
    let hint_end = head
        .rfind(|c: char| !c.is_whitespace() && c != '…')
        .map(|b| head[..b].chars().count() as u16 + 1)
        .unwrap_or(0);
    assert!(
        hint_end <= status_x,
        "S2: hints end at {hint_end}, never under the status at {status_x}"
    );
    assert!(
        limit < status_x,
        "S2: the complete-hint limit ({limit}) stays shy of the status"
    );

    // PTY: plain secondary status at boot, replaced by each menu action,
    // and the status line takes no input.
    let case = s5_case("keyhint_status", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("last: nothing yet"),
        "V2 live: plain secondary status at boot"
    );
    assert_line_lacks(&boot, "last:", "▲", "N1 live: no warning mark");
    // A2 live: a new menu action replaces the status text.
    press(&mut s, "tab");
    waits::wait_state(&mut s, "← → Menu", "menu bar never focused");
    press(&mut s, "enter");
    waits::wait_state(&mut s, "New tab", "File menu never opened");
    checkpoint(&mut s, &dir, "menu-open");
    press(&mut s, "enter");
    checkpoint(&mut s, &dir, "after-choose");
    // The choose lands (menu closes, app status shows it) but focus
    // stays on the bar, whose layer carries no `last:` — one tab onto
    // any other stop restores the screen layer with the status.
    press(&mut s, "tab");
    waits::wait_state(
        &mut s,
        "last: File › New tab",
        "A2 live: first action records",
    );
    checkpoint(&mut s, &dir, "first-action");
    tab_until(
        &mut s,
        "← → Menu",
        case_timeout(&case),
        "menu bar never refocused",
    );
    press(&mut s, "enter");
    waits::wait_state(&mut s, "Split right", "File menu never reopened");
    press(&mut s, "down");
    press(&mut s, "enter");
    press(&mut s, "tab");
    waits::wait_state(
        &mut s,
        "last: File › Split right",
        "A2 live: second action replaces",
    );
    let replaced = live_text(&mut s);
    assert!(
        !replaced.contains("New tab"),
        "A2 live: the old status is gone"
    );
    // A1 live: the status line takes no input — clicking it changes nothing.
    let before = settle_text(&mut s, case_timeout(&case), "A1 steady");
    let before_frame = live_frame(&mut s);
    click_at(&mut s, "last:", case_timeout(&case));
    let after = settle_text(&mut s, case_timeout(&case), "A1 steady");
    assert_eq!(
        after, before,
        "A1 live: clicking the status changes nothing"
    );
    let after_frame = live_frame(&mut s);
    assert_eq!(
        after_frame.digest(),
        before_frame.digest(),
        "A1 live: byte-identical frame"
    );
    eprintln!("s5 keyhint_status: toned marks + live replace + no-input click");
}

/// KH-DROP-002 (`showcase/pages/chrome`, 72x20): narrow drop, badge,
/// centered rows, and containment.
///
/// Isolated path renders [`keyhint::render_aligned`] into sentinel
/// buffers: drop-from-the-right with the faint `…`, the leading badge
/// block, centered mid-row measurement with badge/status clamps, the
/// drawn count with its 2-cell cut reserve, and the 1..16-cell
/// containment sweep. Live path boots the chrome page at 72x20 (dropped
/// hints + `…`), zooms for the badge, clicks the row (no input), and
/// widens to 120x30 (dropped hints redrawn).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_keyhint_drop() {
    let dir = s5_dir("s5_keyhint_drop");
    let stage = Stage::new();
    let t = &stage.theme;
    let hints = [
        hint("Enter", "Open"),
        hint("Space", "Choose"),
        hint("g", "Git URL"),
        hint("Tab", "Next"),
        hint("Esc", "Cancel"),
    ];

    // V1: hints drop from the right and a faint … marks the cut.
    let mut buf = sentinel_buf(30, 3);
    let area = Rect::new(0, 1, 30, 1);
    let n = keyhint::render_aligned(area, &mut buf, t, &hints, None, None, false);
    assert!((2..5).contains(&n), "V1: some hints drop, some stay ({n})");
    assert_eq!(n, 2, "S1: render returns the drawn count");
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("Enter")
            && row.contains("Open")
            && char_col(&row, "Enter") < char_col(&row, "Open"),
        "V1: the head stays\n{row:?}"
    );
    assert!(row.contains('…'), "V1: the cut is marked\n{row:?}");
    assert!(!row.contains("Cancel"), "V1: the tail drops\n{row:?}");
    let cut_x = char_col(&row, "…");
    assert_eq!(
        buf[(cut_x, 1)].fg,
        t.text_faint,
        "V1: the cut marker is faint"
    );
    assert_surround_intact(&buf, area, "N1: containment");
    capture_isolated(&dir, "keyhint-drop", &buf);
    // V2: a badge renders as ` TEXT ` leading; centered rows sit mid-row.
    let mut buf = sentinel_buf(72, 3);
    let area = Rect::new(0, 1, 72, 1);
    keyhint::render_aligned(
        area,
        &mut buf,
        t,
        &hints,
        Some(("ZOOM", BadgeKind::Edit)),
        Some(("last: View › Zoom pane", Tone::Secondary)),
        false,
    );
    let row = buf_row_text(&buf, 1);
    let unmarked: String = row.chars().skip(1).collect();
    assert!(
        unmarked.starts_with(" ZOOM "),
        "V2: the badge leads\n{row:?}"
    );
    // The sentinel row has one unpainted cell past the status (the
    // status pins right-edge-minus-one); trim sentinels, not content.
    let drawn = row.trim_end().trim_end_matches('·');
    assert!(
        drawn.ends_with("last: View › Zoom pane"),
        "V2: the status keeps the right edge\n{row:?}"
    );
    let mut buf = sentinel_buf(72, 1);
    keyhint::render_aligned(
        Rect::new(0, 0, 72, 1),
        &mut buf,
        t,
        &hints,
        None,
        None,
        true,
    );
    let row = buf_row_text(&buf, 0);
    // Byte indices lie past multibyte separators — measure in cells.
    let first_b = row.find("Enter").unwrap();
    let last_b = row.rfind("Cancel").unwrap() + "Cancel".len();
    let first = row[..first_b].chars().count();
    let last = row[..last_b].chars().count();
    assert!(first > 4, "V2: centered rows leave the left edge\n{row:?}");
    assert!(
        (first as i32 - (72 - last) as i32).abs() <= 3,
        "V2: centered rows sit mid-row\n{row:?}"
    );
    // S2: centered start clamps between the badge and the status.
    let mut buf = sentinel_buf(40, 1);
    let drawn = keyhint::render_aligned(
        Rect::new(0, 0, 40, 1),
        &mut buf,
        t,
        &hints,
        Some(("ZOOM", BadgeKind::Edit)),
        Some(("last: View › Zoom pane", Tone::Secondary)),
        true,
    );
    let row = buf_row_text(&buf, 0);
    let badge_end = row.find("ZOOM").unwrap() + "  ZOOM ".len() - 1;
    let status_x = row.find("last:").unwrap();
    let block_start = row.find("Enter").unwrap_or(status_x);
    assert!(
        block_start >= badge_end,
        "S2/N2: centered block never slides under the badge or status\n{row:?}"
    );
    if drawn > 0 {
        assert!(
            row.contains("Enter"),
            "S2: the drawn head stays past the badge\n{row:?}"
        );
    }
    let block_end = row[..status_x].rfind("Cancel").map_or_else(
        || {
            row[..status_x]
                .rfind(|c: char| !c.is_whitespace() && c != '…')
                .map(|b| b + 1)
                .unwrap_or(0)
        },
        |b| b + "Cancel".len(),
    );
    assert!(
        block_end <= status_x,
        "N2: centered block never slides under the status\n{row:?}"
    );
    // N1: nothing paints outside the assigned area at any width.
    for width in 1..16u16 {
        let area = Rect::new(2, 1, width, 1);
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
        for cell in &mut buf.content {
            cell.set_symbol(".");
        }
        keyhint::render_aligned(
            area,
            &mut buf,
            t,
            &[hint("Esc", "Cancel")],
            Some(("EDIT", BadgeKind::Edit)),
            Some(("long failure message", Tone::Error)),
            width % 2 == 0,
        );
        for y in 0..3 {
            for x in 0..20 {
                if !area.contains(Position::new(x, y)) {
                    assert_eq!(buf[(x, y)].symbol(), ".", "N1: width {width} leaks");
                }
            }
        }
    }

    // PTY: narrow boot drops hints, zoom adds the badge, widen redraws.
    let case = s5_case("keyhint_drop", &["--page", "chrome"], 72, 20);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "narrow-boot");
    let narrow = live_text(&mut s);
    assert!(narrow.contains('…'), "V1 live: narrow boot marks the cut");
    press(&mut s, "tab");
    waits::wait_state(&mut s, "← → Menu", "menu bar never focused");
    press(&mut s, "enter");
    waits::wait_state(&mut s, "New tab", "File menu never opened");
    press(&mut s, "right");
    waits::wait_state(&mut s, "Zoom pane", "View menu never opened");
    press(&mut s, "enter");
    // Choosing leaves the bar layer (no cut there) — one tab restores
    // the narrow screen layer where the badge and the cut meet.
    press(&mut s, "tab");
    waits::wait_state(&mut s, "ZOOM", "V2 live: zoom badge never led");
    let zoomed = live_text(&mut s);
    assert_line_has(
        &zoomed,
        "ZOOM",
        "…",
        "V2 live: badge leads, cut stays marked",
    );
    checkpoint(&mut s, &dir, "narrow-zoomed");
    // A1 live: the hint row takes no input at any width.
    let before = settle_text(&mut s, case_timeout(&case), "A1 steady");
    click_at(&mut s, "ZOOM", case_timeout(&case));
    let after = settle_text(&mut s, case_timeout(&case), "A1 steady");
    assert_eq!(after, before, "A1 live: clicking the row changes nothing");
    // A2 live: widening the terminal redraws the dropped hints.
    resize_to(&mut s, 120, 30, case_timeout(&case));
    waits::wait_state(&mut s, "right-click", "A2 live: dropped hints never redrew");
    checkpoint(&mut s, &dir, "widened");
    let wide = live_text(&mut s);
    assert_line_has(
        &wide,
        "Context menu",
        "Tab",
        "A2 live: the full row redraws",
    );
    eprintln!("s5 keyhint_drop: narrow drop + badge + widen redraw");
}

/// Page-shaped grid columns (the customers table: id primary read-only,
/// mrr read-only, the rest writable).
fn s5_grid_columns() -> Vec<ColumnSpec> {
    vec![
        ColumnSpec::new("id", CellKind::Id)
            .primary()
            .read_only()
            .nullable(false),
        ColumnSpec::new("customer", CellKind::Text).nullable(false),
        ColumnSpec::new("plan", CellKind::Enum)
            .enum_values(&["free", "pro", "team", "enterprise"])
            .nullable(false),
        ColumnSpec::new("seats", CellKind::Number).nullable(false),
        ColumnSpec::new("mrr", CellKind::Number).read_only(),
        ColumnSpec::new("active", CellKind::Bool).nullable(false),
        ColumnSpec::new("renewed_at", CellKind::Timestamp),
        ColumnSpec::new("notes", CellKind::Json),
    ]
}

fn s5_grid_rows(n: usize) -> Vec<Vec<CellValue>> {
    (0..n)
        .map(|i| {
            vec![
                CellValue::Int(1001 + i as i64),
                CellValue::Text(format!("Customer {i}")),
                CellValue::Text("pro".into()),
                CellValue::Int(10 + i as i64),
                CellValue::Num(290.0),
                CellValue::Bool(true),
                CellValue::Null,
                CellValue::Null,
            ]
        })
        .collect()
}

/// GRID-EDIT-001 (`showcase/flows/datagrid/selected`, 80x24): the
/// customers cell cursor plus the inline cell editor.
///
/// Isolated path drives a page-shaped [`DataGrid`]: the estimated-total
/// position label, the cursor-row gutter, EditState entry with the cell
/// edit text, validator-gated commit with `CellChanged`, inline Bool
/// toggles, Tab commit-and-advance past read-only/primary columns, and
/// the read-only/deleted/invalid negatives. Live path boots the datagrid
/// page, moves the cursor, edits a customer cell, toggles a Bool inline,
/// and proves the id column and bad numbers never commit.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_grid_edit() {
    let dir = s5_dir("s5_grid_edit");
    let bid = s5_id("test.s5.grid");

    // V1 headless: the cursor cell is highlighted and the meta reads the
    // estimated total.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.local_sort = true;
    g.set_rows(GridRows {
        rows: s5_grid_rows(40),
        total: RowTotal::Estimated(4_812),
        more: true,
    });
    g.scroll.set_viewport(40);
    assert_eq!(
        g.rows_label(),
        "rows 1–40 of 40 loaded · ~4,812 total",
        "V1: estimated-total position label"
    );
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 22);
    let area = Rect::new(0, 1, 80, 20);
    g.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    let mut cursor_marked = false;
    let mut other_marked = false;
    let mut dump = String::new();
    for y in 1..21u16 {
        let line = buf_row_text(&buf, y);
        dump.push_str(&line);
        dump.push('\n');
        if line.contains("1001") {
            cursor_marked = line.starts_with('▎');
        } else if line.contains("1002") {
            other_marked = line.starts_with('▎');
        }
    }
    assert!(
        cursor_marked,
        "V1: the cursor cell row carries the ▎ bar\n{dump}"
    );
    assert!(!other_marked, "V1: only the cursor row is marked\n{dump}");
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "grid-cursor", &buf);

    // S1: Enter on a writable cell opens EditState with the cell edit_text.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    let (o, _) = g.on_key(&s5_key(KeyCode::Right));
    assert!(matches!(o, Outcome::Changed) && g.cursor == (0, 1));
    let (o, ev) = g.on_key(&s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none(),
        "S1: Enter opens"
    );
    let edit = g.edit.as_ref().expect("S1: EditState opens");
    assert_eq!(
        (edit.row, edit.col),
        (0, 1),
        "S1: EditState targets the cell"
    );
    assert_eq!(edit.buffer.text(), "Customer 0", "S1: the cell edit_text");
    // S2/V2: typing commits through default_validator and emits CellChanged.
    for c in " Jr".chars() {
        g.on_key(&s5_key(KeyCode::Char(c)));
    }
    assert_eq!(
        g.edit.as_ref().unwrap().buffer.text(),
        "Customer 0 Jr",
        "V2: while editing, the cell shows the draft text"
    );
    let (o, ev) = g.on_key(&s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed)
            && matches!(ev, Some(GridEvent::CellChanged { row: 0, col: 1 })),
        "S2: commit emits CellChanged"
    );
    assert_eq!(
        g.pending.value(0, 1),
        Some(&CellValue::Text("Customer 0 Jr".into())),
        "V2: commit repaints the typed value"
    );
    // A1: Bool cells toggle inline without opening the editor.
    for _ in 0..4 {
        g.on_key(&s5_key(KeyCode::Right));
    }
    assert_eq!(g.cursor, (0, 5), "A1: cursor reaches the active column");
    let (o, ev) = g.on_key(&s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed)
            && matches!(ev, Some(GridEvent::CellChanged { row: 0, col: 5 })),
        "A1: Bool Enter toggles inline"
    );
    assert!(!g.is_editing(), "A1: no editor opens for Bool");
    assert_eq!(
        g.pending.value(0, 5),
        Some(&CellValue::Bool(false)),
        "A1: the toggle queues false"
    );
    // A2: Tab commits then opens the next writable cell, skipping
    // read_only and primary columns.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    g.on_key(&s5_key(KeyCode::Right));
    g.on_key(&s5_key(KeyCode::Enter));
    assert!(g.is_editing(), "A2: editing the customer cell");
    let (o, _) = g.on_key(&s5_key(KeyCode::Tab));
    assert!(matches!(o, Outcome::Changed), "A2: Tab commits");
    assert_eq!(g.cursor, (0, 2), "A2: Tab advances to the plan cell");
    assert!(g.is_editing(), "A2: Tab opens the next editor");
    // From seats (3), Tab skips the read_only mrr (4) into active (5),
    // whose inline toggle runs instead of an editor.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    for _ in 0..3 {
        g.on_key(&s5_key(KeyCode::Right));
    }
    g.on_key(&s5_key(KeyCode::Enter));
    g.on_key(&s5_key(KeyCode::Tab));
    assert_eq!(g.cursor, (0, 5), "A2: Tab skips the read_only mrr column");
    assert_eq!(
        g.pending.value(0, 5),
        Some(&CellValue::Bool(false)),
        "A2: the skipped-into Bool toggles inline"
    );
    // N1: Enter on a read_only column or a queued-for-deletion row must
    // NOT open the editor.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    let (o, ev) = g.on_key(&s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none() && !g.is_editing(),
        "N1: Enter on the primary id column opens nothing"
    );
    g.toggle_delete(0);
    g.on_key(&s5_key(KeyCode::Right));
    let (o, ev) = g.on_key(&s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none() && !g.is_editing(),
        "N1: Enter on a queued-for-deletion row opens nothing"
    );
    // N2: a failed validation must NOT emit CellChanged: the editor stays
    // open with the error.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    for _ in 0..3 {
        g.on_key(&s5_key(KeyCode::Right));
    }
    g.on_key(&s5_key(KeyCode::Enter));
    g.edit.as_mut().unwrap().buffer.set_text("abc");
    let ev = g.commit_edit();
    assert!(ev.is_none(), "N2: no CellChanged on invalid input");
    assert!(g.is_editing(), "N2: the editor stays open");
    assert_eq!(
        g.edit_error(),
        Some("Must be a number"),
        "N2: the error explains"
    );
    assert!(g.pending.is_empty(), "N2: nothing queues");

    // PTY: move, edit, commit, toggle, refuse.
    let case = s5_case("grid_edit", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("rows 1–14 of 40 loaded · ~4,812 total"),
        "V1 live: the 14-row window of the estimated total"
    );
    press(&mut s, "tab");
    waits::wait_state(&mut s, "Cell", "grid never focused");
    press(&mut s, "down");
    press(&mut s, "right");
    waits::wait_state(&mut s, "1002", "cursor never moved");
    checkpoint(&mut s, &dir, "selected");
    let selected = live_text(&mut s);
    assert_line_has(
        &selected,
        "1002",
        "▎",
        "V1 live: the cursor cell highlights",
    );
    press(&mut s, "enter");
    waits::wait_state(&mut s, "Next cell", "editor never opened");
    type_text(&mut s, " Jr");
    checkpoint(&mut s, &dir, "draft");
    let draft = live_text(&mut s);
    assert!(
        draft.contains("Blue Yonder Airlines Jr"),
        "V2 live: while editing, the cell shows the draft"
    );
    press(&mut s, "enter");
    waits::wait_state(&mut s, "1 pending", "S2 live: commit never queued");
    checkpoint(&mut s, &dir, "committed");
    let committed = live_text(&mut s);
    assert!(
        committed.contains("Blue Yonder Airlines Jr"),
        "V2 live: commit repaints the typed value"
    );
    // A1 live: a Bool cell toggles inline — pending grows, no editor
    // opens. The queue counts dirty ROWS, so the toggle moves down one
    // row first: same-row edits would keep reporting `1 pending`.
    press(&mut s, "down");
    for _ in 0..4 {
        press(&mut s, "right");
    }
    press(&mut s, "enter");
    waits::wait_state(&mut s, "2 pending", "A1 live: Bool toggle never queued");
    let toggled = live_text(&mut s);
    assert!(
        !toggled.contains("Next cell"),
        "A1 live: no editor opens for Bool"
    );
    // N1 live: Enter on the read_only id column opens nothing.
    press(&mut s, "home");
    press(&mut s, "enter");
    let refused = settle_text(&mut s, case_timeout(&case), "N1 steady");
    assert!(
        !refused.contains("Next cell") && refused.contains("2 pending"),
        "N1 live: the id column refuses the editor"
    );
    // N2 live: a bad number never commits — the editor stays open.
    press(&mut s, "right");
    press(&mut s, "right");
    press(&mut s, "right");
    press(&mut s, "enter");
    waits::wait_state(&mut s, "Next cell", "seats editor never opened");
    press(&mut s, "ctrl-l");
    type_text(&mut s, "abc");
    press(&mut s, "enter");
    let invalid = settle_text(&mut s, case_timeout(&case), "N2 steady");
    assert!(
        invalid.contains("Next cell"),
        "N2 live: the editor stays open on invalid input"
    );
    assert!(
        invalid.contains("2 pending"),
        "N2 live: nothing new queues (presence: the old queue)"
    );
    press(&mut s, "esc");
    eprintln!("s5 grid_edit: cursor + inline editor + toggle + refusals");
}

/// GRID-PENDING-002 (`showcase/flows/datagrid/preview`, 80x24): the
/// pending-change queue plus the Preview SQL dialog.
///
/// Isolated path drives a page-shaped [`DataGrid`]: `record_cell` queuing
/// with revert-to-stored clearing, inserted-row deletion removing the row
/// entirely, the `p`/`u`/`U` keys, the three bar buttons via `on_bar_key`,
/// and discard clearing only the queue. Live path replays the matrix
/// `GRID_EDITS` (seats 600/12), inserts and deletes a row, previews the
/// SQL dialog, undoes one change, and discards the rest.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_grid_pending() {
    let dir = s5_dir("s5_grid_pending");
    let bid = s5_id("test.s5.grid");

    // S1: record_cell queues values; reverting to the stored value clears.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    g.record_cell(0, 3, CellValue::Int(600));
    assert_eq!(g.pending.counts(), (1, 0, 0), "S1: one change queues");
    assert_eq!(
        g.row_state(0),
        RowState::Modified,
        "S1: the row reads Modified"
    );
    g.record_cell(0, 3, CellValue::Int(10));
    assert!(g.pending.is_empty(), "S1: reverting clears the change");
    assert_eq!(
        g.row_state(0),
        RowState::Clean,
        "S1: the row reads Clean again"
    );
    // S2: deleting an inserted row removes it entirely instead of queueing.
    let len = g.len();
    let ev = g.insert_row();
    assert!(
        matches!(ev, Some(GridEvent::RowInserted(_))),
        "S2: insert reports"
    );
    assert_eq!(g.len(), len + 1, "S2: the row exists");
    let src = len;
    assert!(g.pending.inserted.contains(&src), "S2: the insert queues");
    g.toggle_delete(src);
    assert_eq!(g.len(), len, "S2: the inserted row is gone entirely");
    assert!(
        g.pending.is_empty(),
        "S2: no delete queues for an inserted row"
    );
    // V1 headless: the pending bar shows its three buttons as separate stops.
    g.record_cell(0, 3, CellValue::Int(600));
    let ids = g.bar_ids();
    assert!(
        ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2],
        "V1: three separate bar stops"
    );
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 22);
    let area = Rect::new(0, 1, 80, 20);
    g.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    let bar = buf_row_text(&buf, 20);
    assert!(
        bar.contains("Preview SQL") && bar.contains("Discard") && bar.contains("Save"),
        "V1: Preview SQL, Discard and Save render\n{bar:?}"
    );
    assert!(bar.contains("1 pending"), "V1: the bar counts the queue");
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "grid-pending-bar", &buf);
    // A1: p emits PreviewSql; u undoes one queued change; U discards.
    let (o, ev) = g.on_key(&s5_key(KeyCode::Char('p')));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(GridEvent::PreviewSql)),
        "A1: p emits PreviewSql"
    );
    g.record_cell(1, 3, CellValue::Int(12));
    assert_eq!(g.pending.total(), 2, "A1: two changes queue");
    let (o, _) = g.on_key(&s5_key(KeyCode::Char('u')));
    assert!(matches!(o, Outcome::Changed), "A1: u runs");
    assert_eq!(g.pending.total(), 1, "A1: u undoes one queued change");
    let (o, ev) = g.on_key(&s5_key(KeyCode::Char('U')));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(GridEvent::DiscardRequested)),
        "A1: U emits DiscardRequested"
    );
    // A2: the bar buttons emit PreviewSql/DiscardRequested/CommitRequested.
    let (o, ev) = g.on_bar_key(ids[0], &s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(GridEvent::PreviewSql)),
        "A2: Preview SQL emits PreviewSql"
    );
    let (o, ev) = g.on_bar_key(ids[1], &s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(GridEvent::DiscardRequested)),
        "A2: Discard emits DiscardRequested"
    );
    let (o, ev) = g.on_bar_key(ids[2], &s5_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(GridEvent::CommitRequested)),
        "A2: Save emits CommitRequested"
    );
    // N2: discard clears only the queue — committed rows are untouched.
    g.record_cell(0, 3, CellValue::Int(600));
    g.insert_row();
    g.toggle_delete(1);
    assert_eq!(g.pending.counts(), (1, 1, 1), "N2: one of each queues");
    g.discard();
    assert!(g.pending.is_empty(), "N2: the queue clears");
    assert_eq!(g.len(), 3, "N2: committed rows are untouched");
    assert_eq!(
        g.value(0, 3),
        &CellValue::Int(10),
        "N2: stored values are untouched"
    );

    // PTY: the matrix GRID_EDITS plus insert, delete, preview, undo, discard.
    let case = s5_case("grid_pending", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive(
        &mut s,
        &[
            "tab",
            "right",
            "right",
            "right",
            "enter",
            "ctrl-l",
            "type:600",
            "enter",
            "down",
            "enter",
            "ctrl-l",
            "type:12",
            "enter",
            "wait:2 pending",
        ],
        case_timeout(&case),
    );
    checkpoint(&mut s, &dir, "pending");
    // `+`/`-` are not chord names (the parser eats `+`), so they go as
    // literal text. Insert parks the cursor on the new row, so step up
    // first: deleting the inserted row itself would remove it entirely.
    type_text(&mut s, "+");
    waits::wait_state(&mut s, "Row inserted", "insert never reported");
    press(&mut s, "up");
    type_text(&mut s, "-");
    waits::wait_state(&mut s, "queued for deletion", "delete never queued");
    checkpoint(&mut s, &dir, "queued");
    let queued = live_text(&mut s);
    assert!(
        queued.contains("• 4 pending") || queued.contains("4 pending"),
        "V1 live: 2 changed · 1 inserted · 1 deleted queue"
    );
    assert_line_has(
        &queued,
        "Preview SQL",
        "Save",
        "V1 live: the bar shows its buttons",
    );
    assert!(queued.contains("Discard"), "V1 live: Discard renders");
    // V2 + N1 live: the preview dialog lists UPDATE/INSERT/DELETE — and an
    // inserted row renders no UPDATE of its own.
    press(&mut s, "p");
    waits::wait_state(&mut s, "Pending changes", "preview never opened");
    checkpoint(&mut s, &dir, "preview");
    let preview = live_text(&mut s);
    assert!(
        preview.contains("UPDATE customers SET seats = 600"),
        "V2 live: the seats=600 UPDATE lists"
    );
    assert!(
        preview.contains("UPDATE customers SET seats = 12"),
        "V2 live: the seats=12 UPDATE lists"
    );
    assert!(
        preview.contains("INSERT INTO customers"),
        "V2 live: the INSERT lists"
    );
    assert!(
        preview.contains("DELETE FROM customers"),
        "V2 live: the DELETE lists"
    );
    assert!(
        preview.contains("2 changed · 1 inserted · 1 deleted"),
        "V2 live: the counts list"
    );
    assert_eq!(
        preview.matches("UPDATE").count(),
        2,
        "N1 live: inserted rows render no UPDATE"
    );
    press(&mut s, "esc");
    wait_absent(
        &mut s,
        "Pending changes",
        case_timeout(&case),
        "preview never closed",
    );
    // A1 live: u undoes one queued change; U discards the rest.
    press(&mut s, "u");
    waits::wait_state(&mut s, "3 pending", "A1 live: u never undid one");
    press(&mut s, "U");
    waits::wait_state(&mut s, "Changes discarded", "A1 live: U never discarded");
    let clean = live_text(&mut s);
    // The blurb always says "pending-change", so pin the bar's shape
    // (`• N pending · …`) and the marker glyph instead of the word.
    assert!(
        !clean.contains("pending ·"),
        "A1 live: the pending bar is gone"
    );
    assert!(!clean.contains('•'), "A1 live: no modified markers survive");
    assert!(
        clean.contains("Northwind Traders") && clean.contains("Blue Yonder Airlines"),
        "N2 live: committed rows are untouched (presence)"
    );
    eprintln!("s5 grid_pending: queue + preview dialog + undo + discard");
}

/// GRID-COMMIT-003 (`showcase/flows/datagrid/failed`, 80x24): the commit
/// round-trip plus the row-level error.
///
/// Isolated path drives a page-shaped [`DataGrid`]: `Ctrl+S` emitting
/// `CommitRequested`, the `Loading rows…` overlay on an empty grid, the
/// spinner `fetching…` row on a loaded grid, `apply_commit_result` marking
/// exactly the rejected row, and the pending queue surviving failure.
/// Live path replays the matrix failed-save flow (seats 600/12), rides the
/// 4-tick saving window, proves the `!` mark plus the error text, and
/// proves a clean queue reports `Nothing to save`.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_grid_commit() {
    let dir = s5_dir("s5_grid_commit");
    let bid = s5_id("test.s5.grid");

    // A1 headless: Ctrl+S emits CommitRequested.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    g.record_cell(0, 3, CellValue::Int(600));
    let (o, ev) = g.on_key(&s5_key_mods(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(GridEvent::CommitRequested)),
        "A1: Ctrl+S emits CommitRequested"
    );
    // V1 headless, empty grid: the spinner with `Loading rows…`.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_loading(true);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 12);
    let area = Rect::new(0, 1, 80, 10);
    g.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    let mut loading = String::new();
    for y in 1..11u16 {
        loading.push_str(&buf_row_text(&buf, y));
        loading.push('\n');
    }
    assert!(
        loading.contains("Loading rows…"),
        "V1: the empty grid shows Loading rows…\n{loading}"
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "grid-loading", &buf);
    // V1 headless, loaded grid: the fetch row spins `fetching…` while saving.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(40),
        total: RowTotal::Estimated(4_812),
        more: true,
    });
    g.scroll.set_viewport(14);
    g.record_cell(0, 3, CellValue::Int(600));
    g.set_loading(true);
    g.scroll.jump_end();
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 22);
    let area = Rect::new(0, 1, 80, 20);
    g.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    let mut saving = String::new();
    for y in 1..21u16 {
        saving.push_str(&buf_row_text(&buf, y));
        saving.push('\n');
    }
    assert!(
        saving.contains("fetching…"),
        "V1: the loaded grid spins fetching… while saving\n{saving}"
    );
    // S1/S2 headless: the round-trip marks exactly the rejected row.
    assert_eq!(
        g.row_state(0),
        RowState::Modified,
        "S1: the edit queues first"
    );
    g.set_loading(false);
    g.apply_commit_result(Err((0, "seats above the plan limit (500)".into())));
    assert_eq!(
        g.row_state(0),
        RowState::Error,
        "S2: the rejected row marks Error"
    );
    assert_eq!(
        g.row_errors.get(&0).map(String::as_str),
        Some("seats above the plan limit (500)"),
        "S2: the row owns its error"
    );
    // N1: valid queued edits are NOT marked when only one row is rejected.
    g.record_cell(1, 3, CellValue::Int(12));
    assert_eq!(
        g.row_state(1),
        RowState::Modified,
        "N1: the valid row stays Modified"
    );
    // N2: the pending queue is NOT cleared on failure.
    assert_eq!(
        g.pending.counts(),
        (2, 0, 0),
        "N2: the queue survives failure"
    );
    // A clean commit folds the queue instead.
    let mut g = DataGrid::new(bid, s5_grid_columns()).editable(true);
    g.set_rows(GridRows {
        rows: s5_grid_rows(3),
        total: RowTotal::Exact(3),
        more: false,
    });
    g.record_cell(0, 3, CellValue::Int(12));
    g.apply_commit_result(Ok(()));
    assert!(g.pending.is_empty(), "clean commit: the queue folds");
    assert_eq!(g.value(0, 3), &CellValue::Int(12), "clean commit: stored");

    // PTY: the matrix failed-save flow.
    let case = s5_case("grid_commit", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive(
        &mut s,
        &[
            "tab",
            "right",
            "right",
            "right",
            "enter",
            "ctrl-l",
            "type:600",
            "enter",
            "down",
            "enter",
            "ctrl-l",
            "type:12",
            "enter",
            "wait:2 pending",
        ],
        case_timeout(&case),
    );
    // The fetch row only renders in view, so jump to the bottom first:
    // while saving it spins `fetching…` there.
    press(&mut s, "G");
    waits::wait_state(&mut s, "Enter fetches more", "fetch row never showed");
    // S1/A1/V1 live: Ctrl+S arms the save — sample tight until it lands,
    // since the 4-tick window is ~320 ms.
    press(&mut s, "ctrl-s");
    let mut saw_saving = false;
    let mut saw_fetching = false;
    let deadline = std::time::Instant::now() + case_timeout(&case);
    loop {
        let text = live_text(&mut s);
        saw_saving |= text.contains("Saving…");
        saw_fetching |= text.contains("fetching…");
        if text.contains("Save failed") {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "A1 live: the save never finished"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(saw_saving, "S1 live: Saving… arms the round-trip");
    assert!(
        saw_fetching,
        "V1 live: the grid spins fetching… while saving"
    );
    checkpoint(&mut s, &dir, "failed");
    // V2/N1/N2 live: back at the top the rejected row marks `!` with the
    // error text, the valid row keeps `•`, and the queue survives. The
    // narrow bar truncates the detail past usefulness, so the mark wait
    // is a line predicate (the 600 row owns `!`); the wide session below
    // proves the full error string.
    press(&mut s, "g");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "V2 live: the rejected row never marked !",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains(" 600 ") && l.contains('!'))
        },
    );
    checkpoint(&mut s, &dir, "marked");
    let marked = live_text(&mut s);
    assert_eq!(
        marked.matches('!').count(),
        1,
        "V2 live: exactly the rejected row marks !"
    );
    assert_line_has(&marked, " 12 ", "•", "N1 live: the valid row keeps •");
    assert!(
        marked.contains("2 pending"),
        "N2 live: the queue survives failure"
    );
    // V2 live, wide: the bar has room for the whole rejection, so the
    // full error string proves what the narrow prefix only hints at.
    let casew = s5_case("grid_commit_wide", &["--page", "datagrid"], 120, 40);
    write_provenance_named(&dir, &casew, "provenance-wide.txt");
    let mut w = support::spawn_boot(&casew);
    support::drive(
        &mut w,
        &[
            "tab",
            "right",
            "right",
            "right",
            "enter",
            "ctrl-l",
            "type:600",
            "enter",
            "wait:1 pending",
        ],
        case_timeout(&casew),
    );
    press(&mut w, "ctrl-s");
    waits::wait_state(&mut w, "Save failed", "wide save never failed");
    press(&mut w, "g");
    waits::wait_state(
        &mut w,
        "seats above the plan limit (500)",
        "V2 live: the full error text never showed",
    );
    checkpoint(&mut w, &dir, "marked-wide");
    // A2 live: a clean queue reports Nothing to save instead.
    let case2 = s5_case("grid_commit_clean", &["--page", "datagrid"], 80, 24);
    write_provenance_named(&dir, &case2, "provenance-clean.txt");
    let mut c = support::spawn_boot(&case2);
    press(&mut c, "tab");
    waits::wait_state(&mut c, "Cell", "clean grid never focused");
    press(&mut c, "ctrl-s");
    waits::wait_state(
        &mut c,
        "Nothing to save",
        "A2 live: clean queue never reported",
    );
    checkpoint(&mut c, &dir, "clean-queue");
    eprintln!("s5 grid_commit: saving window + row error + clean queue");
}

/// Paragraph blocks (runs of non-blank lines), the page's rule.
fn s5_blocks(src: &str) -> Vec<std::ops::Range<usize>> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    let mut end = 0;
    let mut off = 0;
    for line in src.split_inclusive('\n') {
        if line.trim().is_empty() {
            if let Some(s) = start.take() {
                out.push(s..end);
            }
        } else {
            if start.is_none() {
                start = Some(off);
            }
            end = off + line.trim_end_matches('\n').len();
        }
        off += line.len();
    }
    if let Some(s) = start {
        out.push(s..end);
    }
    out
}

/// Minimal tones: `fn`/`let` keywords, `"…"` strings.
fn s5_highlight(src: &str) -> Vec<(std::ops::Range<usize>, SyntaxTone)> {
    let mut out = Vec::new();
    for word in ["fn ", "let "] {
        let mut from = 0;
        while let Some(p) = src[from..].find(word) {
            let kw = &word[..word.len() - 1];
            out.push((from + p..from + p + kw.len(), SyntaxTone::Keyword));
            from += p + word.len();
        }
    }
    let mut from = 0;
    while let Some(p) = src[from..].find('"') {
        let rest = &src[from + p + 1..];
        let end = rest
            .find('"')
            .map(|q| from + p + 1 + q + 1)
            .unwrap_or(src.len());
        out.push((from + p..end, SyntaxTone::Str));
        from = end;
    }
    out
}

const S5_SAMPLE: &str = "fn fetch(url: &str) -> Body {\n    let body = get(url);\n    log(\"done\");\n    body\n}\n\nfn client() -> Client {\n    Client::new().unwrap()\n}\n";

/// CODE-EDIT-001 (`showcase/pages/codeeditor`, 80x24): gutter, navigation
/// and editing modes, and the footer.
///
/// Isolated path drives a [`CodeEditor`] over a two-block sample: gutter
/// geometry (bar + `›` + right-aligned numbers), the `ln L/N · col C`
/// footer, syntax tones through the production render, edit
/// entry/commit, block jumps, find with match stepping, h/l pan and
/// paging, and the navigation-typing/Esc negatives. Live path boots the
/// editor page on retry.rs and walks the same flow.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_code_edit() {
    let dir = s5_dir("s5_code_edit");
    let bid = s5_id("test.s5.code");

    // V1 headless: the gutter shows the cursor bar, block marker › and
    // right-aligned numbers.
    let mut e = CodeEditor::new(bid, S5_SAMPLE)
        .highlighter(s5_highlight)
        .segmenter(s5_blocks);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(60, 14);
    let area = Rect::new(0, 1, 60, 12);
    e.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    let first = buf_row_text(&buf, 1);
    assert!(
        first.starts_with("▎›"),
        "V1: cursor bar + block marker\n{first:?}"
    );
    assert!(
        first.contains(" 1 "),
        "V1: right-aligned line number\n{first:?}"
    );
    let second = buf_row_text(&buf, 2);
    assert!(
        second.starts_with("  ") && second.contains(" 2 "),
        "V1: plain gutter off the cursor\n{second:?}"
    );
    // V2 headless: the footer reads ln L/N · col C; keywords and strings
    // carry syntax tones.
    let footer = buf_row_text(&buf, 12);
    assert!(
        footer.contains("ln 1/10 · col 1"),
        "V2: position readout\n{footer:?}"
    );
    assert!(buf_is_bold(&buf, 6, 1), "V2: the `fn` keyword reads bold");
    assert_eq!(
        buf[(6, 1)].fg,
        stage.theme.text_primary,
        "V2: the keyword takes primary"
    );
    let third = buf_row_text(&buf, 3);
    let str_x = char_col(&third, "\"done\"");
    assert_eq!(
        buf[(str_x, 3)].fg,
        stage.theme.text_secondary,
        "V2: strings take secondary"
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "code-gutter", &buf);

    // S1: i/Enter/a enters editing; Esc commits back to navigation.
    let mut e = CodeEditor::new(bid, S5_SAMPLE).segmenter(s5_blocks);
    let (o, _) = e.on_key(&s5_key(KeyCode::Char('i')));
    assert!(matches!(o, Outcome::Changed) && e.editing, "S1: i edits");
    let (o, ev) = e.on_key(&s5_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(EditorEvent::Committed)),
        "S1: Esc commits"
    );
    assert!(!e.editing, "S1: back to navigation");
    assert_eq!(e.text(), S5_SAMPLE, "S1: the document is kept");
    // S2: { and } jump to the previous/next block start.
    let second = e.blocks()[1].start;
    let (o, ev) = e.on_key(&s5_key(KeyCode::Char('}')));
    assert!(
        matches!(ev, Some(EditorEvent::CursorMoved)) && e.cursor_offset() == second,
        "S2: }} jumps to the next block ({o:?})"
    );
    let (o, ev) = e.on_key(&s5_key(KeyCode::Char('{')));
    assert!(
        matches!(ev, Some(EditorEvent::CursorMoved)) && e.cursor_offset() == 0,
        "S2: {{ jumps back ({o:?})"
    );
    // A1: / opens the find bar; n/N step through matches with CursorMoved.
    let (o, _) = e.on_key(&s5_key(KeyCode::Char('/')));
    assert!(
        matches!(o, Outcome::Changed) && e.find.is_some(),
        "A1: / finds"
    );
    for c in "client".chars() {
        e.on_key(&s5_key(KeyCode::Char(c)));
    }
    assert!(
        e.find.as_ref().is_some_and(|f| !f.matches.is_empty()),
        "A1: the needle matches"
    );
    let at = e.cursor_offset();
    let (o, ev) = e.on_key(&s5_key(KeyCode::Enter));
    assert!(
        matches!(ev, Some(EditorEvent::CursorMoved)),
        "A1: Enter lands ({o:?} at {at})"
    );
    let first_match = e.cursor_offset();
    let (_, ev) = e.on_key(&s5_key(KeyCode::Char('n')));
    assert!(
        matches!(ev, Some(EditorEvent::CursorMoved)),
        "A1: n steps with CursorMoved"
    );
    let _ = first_match;
    // A2: h/l pan 8 cells; PageUp/Down move by the viewport height.
    let mut e = CodeEditor::new(bid, S5_SAMPLE).segmenter(s5_blocks);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(60, 14);
    e.render(
        Rect::new(0, 1, 60, 12),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    e.on_key(&s5_key(KeyCode::Char('l')));
    assert_eq!(e.hscroll, 8, "A2: l pans 8 cells");
    e.on_key(&s5_key(KeyCode::Char('h')));
    assert_eq!(e.hscroll, 0, "A2: h pans back");
    // N1: typing in navigation must NOT mutate the document.
    let before = e.text().to_owned();
    let (o, ev) = e.on_key(&s5_key(KeyCode::Char('x')));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "N1: typing in navigation is Ignored"
    );
    assert_eq!(e.text(), before, "N1: the document is untouched");
    // N2: Esc with no find bar open must NOT consume.
    let (o, ev) = e.on_key(&s5_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "N2: a bare Esc returns Ignored"
    );

    // PTY: navigate, jump blocks, edit, commit, refuse.
    let case = s5_case("code_edit", &["--page", "codeeditor"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    press(&mut s, "tab");
    // The footer reads ln/col only while focused (`navigating` shows
    // regardless and would pass before the tab lands).
    waits::wait_state(&mut s, "ln 1/26 · col 1", "editor never focused");
    checkpoint(&mut s, &dir, "navigating");
    let nav = live_text(&mut s);
    assert!(
        nav.contains("ln 1/26 · col 1"),
        "V2 live: the footer reads ln 1/26 · col 1"
    );
    assert!(nav.contains('›'), "V1 live: the block marker renders");
    assert!(nav.contains("3 blocks"), "meta: 3 blocks (presence)");
    // S2 live: } jumps to the next block start (line 18).
    press(&mut s, "}");
    waits::wait_state(&mut s, "ln 18", "S2 live: } never jumped blocks");
    // S1 live: i enters editing; Esc commits back, keeping the document.
    press(&mut s, "i");
    waits::wait_state(&mut s, "editing", "S1 live: i never entered editing");
    checkpoint(&mut s, &dir, "editing");
    // A single char: a 2+ char word would trigger completion, whose Esc
    // dismisses the popup instead of committing.
    type_text(&mut s, "x");
    press(&mut s, "esc");
    waits::wait_state(&mut s, "navigating", "S1 live: Esc never committed");
    let kept = live_text(&mut s);
    assert!(
        kept.contains("xfn client"),
        "S1 live: Esc keeps the document"
    );
    // N1 live: typing in navigation mutates nothing.
    let before = settle_text(&mut s, case_timeout(&case), "N1 steady");
    press(&mut s, "x");
    let after = settle_text(&mut s, case_timeout(&case), "N1 steady");
    assert_eq!(
        after, before,
        "N1 live: typing in navigation changes nothing"
    );
    // N2 live: a bare Esc drops focus back to the nav — the document it
    // leaves alone (no ▎ in the editor, app hints back, ln/col gone).
    press(&mut s, "esc");
    let bare = settle_text(&mut s, case_timeout(&case), "N2 steady");
    assert!(
        bare.contains("xfn client"),
        "N2 live: Esc keeps the document"
    );
    assert!(
        bare.contains("↑ ↓ Move"),
        "N2 live: focus drops back to the nav"
    );
    assert!(
        !bare.contains("ln 18/26"),
        "N2 live: the editor footer reads position, not ln/col"
    );
    eprintln!("s5 code_edit: gutter + modes + jumps + refusals");
}

/// CODE-RUN-002 (`showcase/flows/editor/diag`, 80x24): the running block
/// plus diagnostics.
///
/// Isolated path drives a [`CodeEditor`]: `set_running` painting the
/// accent spinner in the marker column, `current_block` keeping the
/// previous block between blocks (`None` only before the first block),
/// a Warning diagnostic owning the `!` marker slot with its footer
/// message, and `Changed` firing on edit (which the page uses to clear
/// diagnostics). Live path runs the second block paused (the running
/// state, frozen) and playing (the unwrap warning lands), proves a
/// between-blocks run executes the previous block, and proves an edit
/// clears the diagnostic without the run ever touching the text.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_code_run() {
    let dir = s5_dir("s5_code_run");
    let bid = s5_id("test.s5.code");

    // S1/V1 headless: running arms the block and spins the marker column.
    let mut e = CodeEditor::new(bid, S5_SAMPLE)
        .highlighter(s5_highlight)
        .segmenter(s5_blocks);
    let block = e.blocks()[1].clone();
    e.jump_to(block.start);
    assert_eq!(
        e.current_block(),
        Some(block.clone()),
        "S1: the cursor owns block two"
    );
    e.set_running(Some(block.clone()));
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(60, 14);
    let area = Rect::new(0, 1, 60, 12);
    e.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            tick: 3,
            ..Default::default()
        }),
        s5bg,
    );
    let row7 = buf_row_text(&buf, 7);
    let marker = buf[(1, 7)].symbol().to_owned();
    assert!(
        "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".contains(&marker),
        "V1: the marker column spins\n{row7:?} ({marker:?})"
    );
    assert_eq!(
        buf[(1, 7)].fg,
        stage.theme.accent,
        "V1: the spinner takes accent"
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "code-running", &buf);
    // A1 headless: between blocks the cursor keeps the previous block
    // (the `or_else` fallback); `None` happens only before the first
    // block start, which needs leading blank lines.
    let first = e.blocks()[0].clone();
    e.jump_to(S5_SAMPLE.find("\n\n").unwrap() + 1);
    assert_eq!(
        e.current_block(),
        Some(first),
        "A1: between blocks the previous block runs"
    );
    let mut lead = CodeEditor::new(bid, "\n\nfn main() {}").segmenter(s5_blocks);
    lead.jump_to(0);
    assert_eq!(
        lead.current_block(),
        None,
        "A1: None before the first block"
    );
    // V2/S2 headless: the unwrap line owns the ! marker; the footer names it.
    let p = S5_SAMPLE.find(".unwrap()").unwrap();
    e.diagnostics.push(Diagnostic {
        range: p + 1..p + 9,
        severity: Severity::Warning,
        message: "unwrap() panics on Err; propagate with ? instead".into(),
    });
    e.set_running(None);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(60, 14);
    e.render(
        Rect::new(0, 1, 60, 12),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    assert_eq!(buf[(1, 8)].symbol(), "!", "V2: the ! marker owns the slot");
    assert!(buf_is_bold(&buf, 1, 8), "V2: the diagnostic marker is bold");
    assert_eq!(
        buf[(1, 8)].fg,
        stage.theme.warning,
        "V2: a Warning takes warning"
    );
    let footer = buf_row_text(&buf, 12);
    assert!(
        footer.contains("unwrap() panics"),
        "V2: the footer names the diagnostic\n{footer:?}"
    );
    // A2/N2 headless: any edit reports Changed (the page clears on it).
    e.begin_edit();
    let (o, ev) = e.on_key(&s5_key(KeyCode::Char('x')));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(EditorEvent::Changed)),
        "A2: an edit reports Changed"
    );
    // N1 headless: arming a run never touches the document text.
    let mut e = CodeEditor::new(bid, S5_SAMPLE).segmenter(s5_blocks);
    e.set_running(Some(e.blocks()[1].clone()));
    assert_eq!(e.text(), S5_SAMPLE, "N1: the run mutates no text");

    // PTY, paused: the running state frozen plus the between-blocks refusal.
    let case = s5_case(
        "code_run_paused",
        &["--page", "codeeditor", "--motion", "paused"],
        80,
        24,
    );
    write_provenance_named(&dir, &case, "provenance-paused.txt");
    let mut s = support::spawn_boot(&case);
    press(&mut s, "tab");
    waits::wait_state(&mut s, "ln 1/26 · col 1", "paused editor never focused");
    press(&mut s, "}");
    waits::wait_state(&mut s, "ln 18", "} never reached block two");
    press(&mut s, "ctrl-r");
    waits::wait_state(&mut s, "running", "S1 live: Ctrl+R never armed the run");
    checkpoint(&mut s, &dir, "running");
    let running = live_text(&mut s);
    assert!(
        "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".chars().any(|c| running.contains(c)),
        "V1 live: the marker column spins while running"
    );
    // A1 live: between blocks Ctrl+R runs the previous block (block 1
    // has no unwrap/todo, so no diagnostic lands). A fresh session (this
    // one is mid-run and paused ticks never land): the run needs ticks,
    // so this one plays.
    // 120x40: at 80x24 the columns stack and the Diagnostics card's
    // y+4<bottom gate never opens, so the empty panel is unprovable.
    let case_between = s5_case("code_run_between", &["--page", "codeeditor"], 120, 40);
    write_provenance_named(&dir, &case_between, "provenance-between.txt");
    let mut b = support::spawn_boot(&case_between);
    press(&mut b, "tab");
    waits::wait_state(&mut b, "ln 1/26 · col 1", "between editor never focused");
    press(&mut b, "}");
    waits::wait_state(&mut b, "ln 18", "} never reached block two");
    press(&mut b, "up");
    waits::wait_state(&mut b, "ln 17", "never stepped between blocks");
    press(&mut b, "ctrl-r");
    waits::wait_state(
        &mut b,
        "Block ran in 77 ms",
        "A1 live: the previous block never ran",
    );
    checkpoint(&mut b, &dir, "between-ran");
    let between = live_text(&mut b);
    assert!(
        between.contains("Nothing flagged"),
        "A1 live: block 1 flags nothing (presence: the empty panel)"
    );
    // PTY, playing: the run lands the unwrap warning; an edit clears it.
    // 120x40 for the same stacked-columns reason as the between case.
    let case2 = s5_case("code_run", &["--page", "codeeditor"], 120, 40);
    write_provenance_named(&dir, &case2, "provenance-playing.txt");
    let mut p = support::spawn_boot(&case2);
    press(&mut p, "tab");
    waits::wait_state(&mut p, "ln 1/26 · col 1", "playing editor never focused");
    press(&mut p, "}");
    waits::wait_state(&mut p, "ln 18", "} never reached block two");
    press(&mut p, "ctrl-r");
    waits::wait_state(
        &mut p,
        "Block ran in 77 ms",
        "S2 live: the run never landed",
    );
    checkpoint(&mut p, &dir, "diag");
    let diag = live_text(&mut p);
    // The panel truncates the 52-char row to its ~35 cells; the
    // prefix pins the line number and the message head.
    assert!(
        diag.contains("ln 19 · unwrap() panics"),
        "V2 live: the Diagnostics panel lists ln · message"
    );
    assert_line_has(&diag, "unwrap()", "!", "V2 live: the unwrap line owns !");
    // The editor truncates the 43-char statement (`…`), so the head
    // proves the text survived the run byte for byte.
    assert!(
        diag.contains("Client::builder().timeout(10)"),
        "N1 live: the run mutates no text (presence)"
    );
    assert_line_has(
        &diag,
        "Client::builder",
        "!",
        "V2 live: the editor line owns the ! marker",
    );
    // A2/N2 live: any edit clears the diagnostics.
    press(&mut p, "i");
    waits::wait_state(&mut p, "editing", "edit never started");
    type_text(&mut p, "x");
    press(&mut p, "esc");
    waits::wait_state(
        &mut p,
        "Nothing flagged",
        "N2 live: the diagnostic survives",
    );
    eprintln!("s5 code_run: running spinner + unwrap warning + edit clears");
}

/// Page-shaped diff file (Modified config/service.toml, 5 hunks, +10 −5).
fn s5_diff_sample() -> DiffFile {
    DiffFile {
        path: "config/service.toml".into(),
        status: DiffStatus::Modified,
        hunks: (0..5)
            .map(|i| DiffHunk {
                old_start: i * 8 + 1,
                new_start: i * 9 + 1,
                lines: vec![
                    DiffLine::context(format!("[service.worker_{i}]")),
                    DiffLine::remove("attempts = 3"),
                    DiffLine::add("attempts = 5"),
                    DiffLine::add("backoff = \"exponential\""),
                    DiffLine::context("region = \"tokyo\""),
                    DiffLine::context("enabled = true"),
                ],
            })
            .collect(),
    }
}

/// DIFF-UNIFIED-001 (`showcase/flows/diff/empty`, 80x24): the unified
/// listing plus the empty file state.
///
/// Isolated path drives a [`DiffView`] over a page-shaped file: the `M
/// path +N −M · K hunks` header with muted `@@` lines, old/new gutters
/// with bold toned `+`/`-` markers, `set_file(None)` rendering `No file
/// selected` with the scroll reset, the no-hunks `(no textual changes)`
/// form, viewport delegation, and the no-`Old`/`New` plus
/// no-kept-offset negatives. Live path boots the diff page, scrolls the
/// listing, and toggles Empty on.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_diff_unified() {
    let dir = s5_dir("s5_diff_unified");
    let bid = s5_id("test.s5.diff");
    let file = s5_diff_sample();

    // V1 headless: the header plus muted @@ hunk lines.
    let lines = unified_lines(&file);
    assert_eq!(
        line_text(&lines[0]),
        "M config/service.toml  +10 −5 · 5 hunks",
        "V1: header with counts"
    );
    assert!(lines[0][0].bold, "V1: the status marker is bold");
    assert_eq!(
        lines[0][0].tone,
        Tone::Warning,
        "V1: Modified takes warning"
    );
    assert_eq!(
        line_text(&lines[1]),
        "@@ -1,4 +1,5 @@",
        "V1: hunks open with @@ lines"
    );
    assert_eq!(lines[1][0].tone, Tone::Muted, "V1: @@ lines are muted");
    // V2 headless: old/new numbers, a bold marker, and the toned text.
    let rem = lines
        .iter()
        .find(|l| line_text(l).contains("- attempts = 3"))
        .expect("V2: the removed row lists");
    assert_eq!(rem[4].text, "-", "V2: the - marker");
    assert!(rem[4].bold, "V2: the marker is bold");
    assert_eq!(rem[4].tone, Tone::Error, "V2: removes take error");
    assert_eq!(rem[5].tone, Tone::Error, "V2: removed text takes error");
    assert_eq!(line_text(&rem[..4]), "  2     ", "V2: old/new numbers");
    let add = lines
        .iter()
        .find(|l| line_text(l).contains("+ attempts = 5"))
        .expect("V2: the added row lists");
    assert!(
        add[4].bold && add[4].tone == Tone::Success,
        "V2: + bold success"
    );
    // S1: set_file(None) renders No file selected and resets follow/scroll.
    let mut v = DiffView::new(bid);
    v.set_file(Some(file.clone()));
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 24);
    let area = Rect::new(0, 1, 78, 22);
    v.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "diff-unified", &buf);
    v.on_wheel(5);
    assert_eq!(v.term.scroll.offset, 5, "S1: scrolled first");
    v.set_file(None);
    assert_eq!(v.term.scroll.offset, 0, "S1/N2: the offset resets");
    v.layout(78);
    let empty: Vec<String> = v.term.lines().map(|l| line_text(l)).collect();
    assert_eq!(
        empty,
        vec!["No file selected"],
        "S1: the empty line renders"
    );
    // S2: a file with no hunks renders (no textual changes).
    let mut bare = file.clone();
    bare.hunks.clear();
    let bare_lines = unified_lines(&bare);
    assert!(
        bare_lines
            .iter()
            .any(|l| line_text(l) == "(no textual changes)"),
        "S2: the no-hunks form"
    );
    // A2: keys, wheel and drags delegate to the wrapped TextViewport.
    let mut v = DiffView::new(bid);
    v.set_file(Some(file.clone()));
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 24);
    v.render(
        Rect::new(0, 1, 78, 22),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    let (o, _) = v.on_key(&s5_key(KeyCode::Down));
    assert!(
        matches!(o, Outcome::Changed) && v.term.scroll.offset == 1,
        "A2: Down delegates to the viewport"
    );
    // N1: the unified listing must NOT show the Old/New column headers.
    let all: String = unified_lines(&file)
        .iter()
        .map(|l| line_text(l))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !all.contains("Old") && !all.contains("New"),
        "N1: no review headers in unified"
    );

    // PTY: boot, scroll, empty.
    let case = s5_case("diff_unified", &["--page", "diff"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "unified");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("M config/service.toml") && boot.contains("+10 −5 · 5 hunks"),
        "V1 live: header with counts"
    );
    assert!(boot.contains("@@"), "V1 live: @@ hunk lines (presence)");
    assert!(
        boot.contains("- attempts = 3") && boot.contains("+ attempts = 5"),
        "V2 live: marked rows list"
    );
    assert!(
        !boot.contains("Old"),
        "N1 live: no review headers in unified"
    );
    // A2 live: keys delegate — Down scrolls the listing.
    click_at(&mut s, "attempts = 3", case_timeout(&case));
    waits::wait_state(&mut s, "Select", "diff view never focused");
    press(&mut s, "down");
    press(&mut s, "down");
    press(&mut s, "down");
    waits::wait_state(&mut s, "backoff", "listing never scrolled");
    checkpoint(&mut s, &dir, "scrolled");
    let scrolled = live_text(&mut s);
    assert!(
        !scrolled.contains("@@ -1,6"),
        "A2 live: the first hunk scrolled out"
    );
    // A1 live: the Empty toggle calls set_file(None) and clears. A
    // click, not tabs: focus sits in the view, so two tabs would wrap
    // through the nav and land on Review instead of Empty.
    click_at(&mut s, "Empty", case_timeout(&case));
    waits::wait_state(&mut s, "No file selected", "S1 live: empty never rendered");
    checkpoint(&mut s, &dir, "empty");
    eprintln!("s5 diff_unified: listing + scroll + empty");
}

/// DIFF-REVIEW-002 (`showcase/flows/diff/drag-selected`, 80x24):
/// side-by-side review plus drag selection copy.
///
/// Isolated path drives a [`DiffView`] over a page-shaped file: the `Old
/// │ New` columns with faint numbers and bold changed runs, the
/// `2*(numw+1+16)+3` fallback threshold with the mode retained, `y`
/// emitting `Copy`, Esc clearing, wheel scrolling, unpaired blank sides,
/// and tab expansion before measuring. Live path toggles Review on,
/// drags across the columns, copies with `y`, clears with Esc, and
/// toggles the mode off and on.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_diff_review() {
    let dir = s5_dir("s5_diff_review");
    let bid = s5_id("test.s5.diff");
    let file = s5_diff_sample();

    // V1 headless: Old │ New columns with faint numbers, bold changed runs.
    let rows = review_lines(&file, 78);
    assert!(
        line_text(&rows[1]).contains("Old") && line_text(&rows[1]).contains("New"),
        "V1: the column headers"
    );
    let paired = rows
        .iter()
        .find(|l| {
            let t = line_text(l);
            t.contains("attempts = 3") && t.contains("attempts = 5")
        })
        .expect("V1: a paired change row");
    let bolds: Vec<&str> = paired
        .iter()
        .filter(|s| s.bold)
        .map(|s| s.text.as_str())
        .collect();
    assert_eq!(bolds, vec!["3", "5"], "V1: the differing run reads bold");
    assert!(
        paired.iter().any(|s| s.text.contains("│")),
        "V1: the │ separator"
    );
    // S1: review_fits falls back below 2*(numw+1+16)+3 cells; retained mode.
    let mut v = DiffView::new(bid);
    v.set_file(Some(file.clone()));
    v.set_mode(DiffMode::Review);
    assert_eq!(v.layout_mode(42), DiffMode::Unified, "S1: 42 falls back");
    assert_eq!(v.layout_mode(43), DiffMode::Review, "S1: 43 fits review");
    assert_eq!(v.mode, DiffMode::Review, "S1: the mode is retained");
    // N1: unpaired sides must NOT pair text: the empty side stays blank.
    let lopsided = DiffFile {
        path: "x".into(),
        status: DiffStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![
                DiffLine::remove("gone one"),
                DiffLine::remove("gone two"),
                DiffLine::add("fresh"),
            ],
        }],
    };
    let rows = review_lines(&lopsided, 78);
    let lone = rows
        .iter()
        .find(|l| line_text(l).contains("gone two"))
        .expect("N1: the unpaired row");
    let lone_text = line_text(lone);
    let halves: Vec<&str> = lone_text.split('│').collect();
    assert_eq!(halves.len(), 2, "N1: old │ new shape");
    assert!(
        halves[1].trim().is_empty(),
        "N1: the empty side stays blank\n{lone_text}"
    );
    // N2: tabs expand to four spaces before measuring.
    let tabbed = DiffFile {
        path: "x".into(),
        status: DiffStatus::Modified,
        hunks: vec![DiffHunk {
            old_start: 1,
            new_start: 1,
            lines: vec![DiffLine::remove("a\tb"), DiffLine::add("a\tc")],
        }],
    };
    let rows = review_lines(&tabbed, 78);
    let all: String = rows.iter().map(|l| line_text(l)).collect();
    assert!(!all.contains('\t'), "N2: no raw tabs survive");
    let tabbed_row = rows
        .iter()
        .find(|l| line_text(l).contains("a    b"))
        .expect("N2: the tab expands to four spaces");
    let tabbed_text = line_text(tabbed_row);
    let halves: Vec<&str> = tabbed_text.split(" │ ").collect();
    assert_eq!(
        halves[0].chars().count(),
        halves[1].chars().count(),
        "N2: columns measure identically"
    );
    // V2/S2/A2 headless: drag selects, y copies, Esc clears, wheel scrolls.
    let mut v = DiffView::new(bid);
    v.set_file(Some(file.clone()));
    v.set_mode(DiffMode::Review);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(80, 24);
    let area = Rect::new(0, 1, 78, 22);
    v.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "diff-review", &buf);
    let o = v.on_click(Position::new(10, 4));
    assert!(matches!(o, Outcome::Consumed), "V2: click anchors");
    let o = v.on_drag(Position::new(22, 4));
    assert!(matches!(o, Outcome::Changed), "V2: drag selects");
    assert!(v.term.has_selection(), "V2: the range selects");
    let (o, ev) = v.on_key(&s5_key(KeyCode::Char('y')));
    assert!(matches!(o, Outcome::Changed), "S2: y runs");
    let copied = match ev {
        Some(ViewportEvent::Copy(t)) => t,
        other => panic!("S2: y emits Copy, got {other:?}"),
    };
    assert!(!copied.is_empty(), "S2: the copy keeps text");
    assert!(v.term.has_selection(), "V2: copy keeps the selection");
    let (o, _) = v.on_key(&s5_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && !v.term.has_selection(),
        "A2: Esc clears the selection"
    );
    let o = v.on_wheel(2);
    assert!(
        matches!(o, Outcome::Changed) && v.term.scroll.offset == 2,
        "A2: wheel scrolls via the viewport"
    );

    // PTY: review on, drag, copy, clear, toggle off/on.
    let case = s5_case("diff_review", &["--page", "diff"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    press(&mut s, "tab");
    waits::wait_state(&mut s, "Toggle", "review toggle never focused");
    press(&mut s, "enter");
    waits::wait_state(&mut s, "● Review", "A1 live: Review never toggled on");
    checkpoint(&mut s, &dir, "review");
    let review = live_text(&mut s);
    assert!(
        review.contains("Old") && review.contains("New") && review.contains("│"),
        "V1 live: Old │ New columns"
    );
    // V2 live: the dragged range paints the selection style. Anchor
    // inclusion wobbles run to run (press/drag coalescing), so a long
    // drag pins a wide interior with both ends clear of the edges.
    let (row, col) = find_pos(&mut s, "attempts = 3", case_timeout(&case));
    support::drag_path(&mut s, (col, row), (col + 20, row));
    std::thread::sleep(Duration::from_millis(300));
    checkpoint(&mut s, &dir, "drag-selected");
    let frame = live_frame(&mut s);
    let sel_bg = frame.get(col + 8, row).expect("V2: style cell").bg.clone();
    for c in col + 4..col + 16 {
        assert_eq!(
            frame.get(c, row).map(|cell| &cell.bg),
            Some(&sel_bg),
            "V2 live: the dragged range paints one style"
        );
    }
    let before = frame.get(col - 1, row).expect("V2: lead cell").bg.clone();
    assert_ne!(sel_bg, before, "V2 live: the style starts past the lead");
    let edge = frame.get(col + 22, row).expect("V2: edge cell").bg.clone();
    assert_ne!(sel_bg, edge, "V2 live: the style ends past the range");
    // S2 live: y with a selection reports Selection copied in demo.
    press(&mut s, "y");
    waits::wait_state(
        &mut s,
        "Selection copied in demo",
        "S2 live: copy never reported",
    );
    let kept = live_frame(&mut s);
    assert_eq!(
        kept.get(col, row).map(|cell| &cell.bg),
        Some(&sel_bg),
        "V2 live: copy keeps the demo text selected"
    );
    // A2 live: Esc clears the selection (the status line keeps its
    // report — only the selection style must go); wheel scrolls.
    press(&mut s, "esc");
    std::thread::sleep(Duration::from_millis(400));
    let cleared = live_frame(&mut s);
    assert_ne!(
        cleared.get(col, row).map(|cell| &cell.bg),
        Some(&sel_bg),
        "A2 live: Esc clears the selection style"
    );
    wheel_over(&mut s, "attempts = 3", Wheel::Down, 2, case_timeout(&case));
    waits::wait_state(&mut s, "backoff", "wheel never scrolled");
    let wheeled = live_text(&mut s);
    assert!(
        !wheeled.contains("M config/service.toml"),
        "A2 live: the wheel scrolled the header out"
    );
    // A1 live: the toggle flips the mode off and back on. Home first so
    // the Old header is on screen for the gone-wait's presence leg.
    press(&mut s, "home");
    waits::wait_state(&mut s, "Old", "never scrolled home");
    // Focus sits in the view: one tab only reaches the nav, so seek.
    tab_until(
        &mut s,
        "Toggle",
        case_timeout(&case),
        "review toggle never refocused",
    );
    press(&mut s, "enter");
    // Presence proven by the home-wait above; this is the expiry.
    wait_absent(
        &mut s,
        "Old",
        case_timeout(&case),
        "A1 live: unified never returned",
    );
    press(&mut s, "enter");
    waits::wait_state(&mut s, "Old", "A1 live: review never returned");
    eprintln!("s5 diff_review: columns + drag copy + toggles");
}

/// TERM-RUN-001 (`showcase/flows/terminal/mid`, 80x24): the launch
/// transcript plus the step rail run (PTY-only).
///
/// The stage engine, reset, and advance live in the showcase binary's
/// `terminal.rs`, outside the importable library, so every check runs
/// live: the bold prompt plus `jackin launch`, `▶` stage opens, `✓` done
/// lines with the `N lines · following` meta, reset clearing the
/// viewport and re-queueing the rail, the Skipped/cached stage, tick
/// advancement with clean/failure restarts, and the no-`✗`/no-`Blocked`
/// plus post-run-silence negatives.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_term_run() {
    // 120x40, not the row's 80x24: the rail hides per-step metas below
    // the mw+12 threshold at 80 columns (`cached`, `blocked` clip away),
    // so the S2/N1/A2 checks need the room the matrix uses.
    let dir = s5_dir("s5_term_run");
    let case = s5_case("term_run", &["--page", "terminal"], 120, 40).timeout(60_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // V1: the transcript opens with the bold prompt plus jackin launch.
    let boot = live_text(&mut s);
    assert!(
        boot.contains("jackin launch"),
        "V1: the launch command opens the transcript"
    );
    let (prompt_row, prompt_col) = find_pos(&mut s, "payments-platform ❯", case_timeout(&case));
    let frame = live_frame(&mut s);
    assert!(
        frame
            .get(prompt_col, prompt_row)
            .is_some_and(|c| c.mods.bold),
        "V1: the prompt reads bold"
    );
    // V1/V2/A1: stages open with ▶, finish with ✓, ticks advance the rail.
    waits::wait_state(&mut s, "▶ Resolve workspace", "stage 0 never opened");
    waits::wait_state(&mut s, "✓ Resolve workspace", "A1: stage 0 never finished");
    waits::wait_state(&mut s, "▶ Pull base image", "S2: Running never moved on");
    checkpoint(&mut s, &dir, "mid");
    let mid = live_text(&mut s);
    assert!(
        mid.contains("lines · following"),
        "V2: the meta follows the tail"
    );
    // S2: stage 3 is Skipped/cached.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "S2: the skipped stage never showed",
        |screen| support::screen_text(screen).contains("cached"),
    );
    // N1 + end: a clean run shows no ✗ and no Blocked states.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "the run never finished",
        |screen| support::screen_text(screen).contains("7 of 7"),
    );
    checkpoint(&mut s, &dir, "done");
    let done = live_text(&mut s);
    assert!(!done.contains('✗'), "N1: no failure line on a clean run");
    assert!(
        !done.contains("blocked"),
        "N1: no blocked states on a clean run"
    );
    assert!(
        done.contains('✓'),
        "N1: done lines prove the run (presence)"
    );
    // N2: ticks after the last stage append no more output.
    let settled = settle_text(&mut s, case_timeout(&case), "N2 steady");
    assert_eq!(settled, done, "N2: the transcript freezes at the end");
    // S1/A2: Run restarts clean — the viewport clears, the rail re-queues.
    // Boot focus is the nav (no term hints), so the tabs land hop by hop.
    // The rail takes no hop: it is not selectable, never joins the ring,
    // and its Move hints stay dark — the rail reports, it is not driven.
    assert!(
        !done.contains("f Follow"),
        "S1 live: boot focus is the nav (presence)"
    );
    press(&mut s, "tab");
    waits::wait_state(&mut s, "f Follow", "term never focused");
    press(&mut s, "tab");
    waits::wait_state(&mut s, "Enter Activate", "run never focused");
    press(&mut s, "enter");
    waits::wait_state(&mut s, "0 of 7 · running", "S1 live: reset never re-queued");
    let restarted = live_text(&mut s);
    assert!(
        !restarted.contains('✓'),
        "S1 live: the transcript clears (presence leg: the done screen held ✓)"
    );
    assert!(
        restarted.contains("jackin launch"),
        "S1 live: the fresh prompt reopens"
    );
    checkpoint(&mut s, &dir, "restarted");
    // The restart is clean: stage 1 finishes (a failure would ✗ instead).
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A2 live: the clean restart never passed stage 1",
        |screen| support::screen_text(screen).contains("✓ Pull base image"),
    );
    // A2: Run with a failure arms fail_at=1.
    press(&mut s, "tab");
    waits::wait_state(&mut s, "Enter Activate", "fail never focused");
    press(&mut s, "enter");
    waits::wait_state(&mut s, "failed", "A2 live: the failure never armed");
    let failed = live_text(&mut s);
    assert!(failed.contains('✗'), "A2 live: the ✗ failure line shows");
    assert!(failed.contains("blocked"), "A2 live: later stages block");
    checkpoint(&mut s, &dir, "failed");
    eprintln!("s5 term_run: launch transcript + rail + clean/fail restarts");
}

/// TERM-FOLLOW-002 (`showcase/pages/terminal`, 80x24): follow tail,
/// scrollback, selection, and copy.
///
/// Isolated path drives a [`TextViewport`] over a transcript: wheel and
/// scrollbar moves recomputing follow from `is_at_tail`, `Copy` reporting
/// with the text retained, `f` toggling follow with `FollowChanged`, Esc
/// clearing, drag auto-scroll at the edges from the press anchor, the
/// `y`-with-no-selection `Consumed` negative, and the thumb-press-grabs
/// negative. Live path boots the frame-60 terminal, wheels off the tail,
/// drags a selection, and copies it with `y`.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_term_follow() {
    let dir = s5_dir("s5_term_follow");
    let bid = s5_id("test.s5.term");

    fn transcript(n: usize) -> Vec<Line> {
        (0..n)
            .map(|i| vec![Span::plain(format!("line {i:03} of the transcript"))])
            .collect()
    }
    // S1 headless: wheel moves recompute follow from is_at_tail and
    // remember the reading row.
    let mut v = TextViewport::with_lines(bid, transcript(60)).max_lines(2000);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(60, 22);
    let area = Rect::new(0, 1, 60, 20);
    v.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
    );
    assert_surround_intact(&buf, area, "render: containment");
    capture_isolated(&dir, "term-viewport", &buf);
    v.set_follow(true);
    assert!(v.follow && v.is_at_tail(), "S1: follow starts at the tail");
    let o = v.on_wheel(-3);
    assert!(matches!(o, Outcome::Changed), "S1: wheel up moves");
    assert!(!v.follow, "S1: wheeling up breaks follow");
    assert!(!v.is_at_tail(), "S1: off the tail");
    assert!(v.scrollback_depth() > 0, "S1: scrollback depth grows");
    let o = v.on_wheel(100);
    assert!(matches!(o, Outcome::Changed), "S1: wheel down returns");
    assert!(v.follow && v.is_at_tail(), "S1: the tail resumes follow");
    // V1/S2 headless: a dragged selection reports Copy with its text.
    let o = v.on_click(Position::new(2, 2));
    assert!(matches!(o, Outcome::Consumed), "click anchors");
    let o = v.on_drag(Position::new(12, 4));
    assert!(matches!(o, Outcome::Changed), "drag extends");
    assert!(v.has_selection(), "V1: the selection exists");
    let (o, ev) = v.on_key(&s5_key(KeyCode::Char('y')));
    assert!(matches!(o, Outcome::Changed), "S2: y runs");
    let copied = match ev {
        Some(ViewportEvent::Copy(t)) => t,
        other => panic!("S2: y emits Copy, got {other:?}"),
    };
    assert_eq!(copied.lines().count(), 3, "S2: Copied 3 lines");
    assert!(
        copied.lines().next().is_some_and(|l| !l.is_empty()),
        "S2: the first line is retained"
    );
    // A1 headless: f toggles follow with Following/Pausing; Esc clears.
    let (o, ev) = v.on_key(&s5_key(KeyCode::Char('f')));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(ViewportEvent::FollowChanged(_))),
        "A1: f toggles follow"
    );
    let (o, ev) = v.on_key(&s5_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed)
            && matches!(ev, Some(ViewportEvent::SelectionChanged))
            && !v.has_selection(),
        "A2: Esc clears the selection"
    );
    // A2 headless: drag auto-scrolls at the vertical edges, extending
    // from the press anchor.
    v.on_wheel(-10);
    let before = v.scroll.offset;
    assert!(before > 0, "A2: room to scroll up");
    let o = v.on_click(Position::new(2, 10));
    assert!(matches!(o, Outcome::Consumed), "A2: click anchors");
    v.on_drag(Position::new(2, 0));
    assert_eq!(
        v.scroll.offset,
        before - 1,
        "A2: a drag above the area scrolls up"
    );
    assert!(v.has_selection(), "A2: the drag extends from the anchor");
    v.clear_selection();
    // N1: y with no selection must NOT emit Copy.
    let (o, ev) = v.on_key(&s5_key(KeyCode::Char('y')));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none(),
        "N1: y with no selection returns Consumed"
    );
    // N2: a scrollbar press on the thumb must NOT jump: it grabs.
    // At the end the thumb sits on the last track row.
    v.set_follow(true);
    let offset = v.scroll.offset;
    let o = v.on_scrollbar(Position::new(59, 20));
    assert!(
        matches!(o, Outcome::Consumed) && v.scroll.offset == offset,
        "N2: a thumb press grabs without jumping"
    );

    // PTY: follow toggles, wheel off the tail, drag, copy, clear.
    let case = s5_case(
        "term_follow",
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("2 of 7"), "boot: the rail sits at 2 of 7");
    press(&mut s, "tab");
    waits::wait_state(&mut s, "f Follow", "term never focused");
    // A1 live: f toggles follow with Following/Pausing status.
    press(&mut s, "f");
    waits::wait_state(
        &mut s,
        "Paused at the scrollback position",
        "A1 live: no pause",
    );
    press(&mut s, "f");
    waits::wait_state(&mut s, "Following the tail", "A1 live: no follow");
    // V1/S1 live: wheel up leaves the tail for scrollback.
    wheel_over(
        &mut s,
        "#4 RUN cargo build",
        Wheel::Up,
        2,
        case_timeout(&case),
    );
    waits::wait_state(&mut s, "scrollback ↑5", "V1 live: never left the tail");
    checkpoint(&mut s, &dir, "scrolled");
    // V1 live: a drag switches the meta to selection · y copies.
    let (row, col) = find_pos(&mut s, "Resolve workspace", case_timeout(&case));
    support::drag_path(&mut s, (col, row), (col + 10, row + 1));
    std::thread::sleep(Duration::from_millis(300));
    waits::wait_state(&mut s, "selection · y copies", "V1 live: no selection meta");
    checkpoint(&mut s, &dir, "selected");
    // V2/S2 live: y copies; the step side shows copied: first-line.
    press(&mut s, "y");
    waits::wait_state(&mut s, "Copied ", "S2 live: copy never reported");
    waits::wait_state(&mut s, "copied: ", "V2 live: no copied readout");
    checkpoint(&mut s, &dir, "copied");
    // A2 live: Esc clears the selection.
    press(&mut s, "esc");
    wait_absent(
        &mut s,
        "selection · y copies",
        case_timeout(&case),
        "A2 live: selection never cleared",
    );
    eprintln!("s5 term_follow: tail + scrollback + selection copy");
}

/// SCROLL-REGION-001 (`showcase/flows/scrolling/scrolled`, 80x24): the
/// three scrolling columns plus position labels.
///
/// Isolated path drives [`ScrollPanel`]: paging by the viewport, `g`/`G`
/// jumps, `f` toggling follow on a tailing panel only, and the fitting
/// column drawing no scrollbar and no label. Live path boots the
/// frame-1600 scrolling page wide (labels, track, thumb, follow
/// toggles) and narrow (wheel-under-pointer, keys-to-focused,
/// page/ends), plus a playing boot proving the log grows a line per
/// tick and stops at the 2000 cap.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_scroll_region() {
    let dir = s5_dir("s5_scroll_region");
    let bid = s5_id("test.s5.scroll");
    fn plain(t: &Theme, _l: &str) -> Style {
        t.secondary()
    }

    // A1 headless: PageUp/Down page by the viewport length; g/G jump.
    let mut log =
        ScrollPanel::new(bid, (0..400).map(|i| format!("log line {i}")).collect()).tail(true);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(40, 17);
    let area = Rect::new(0, 1, 40, 15);
    log.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
        plain,
    );
    let vp = log.scroll.viewport_len;
    assert!(vp > 1, "A1: a viewport lays out");
    log.scroll.jump_end();
    let end = log.scroll.offset;
    let o = log.on_key(&s5_key(KeyCode::PageUp));
    assert!(
        matches!(o, Outcome::Changed) && log.scroll.offset == end - vp,
        "A1: PageUp pages by the viewport"
    );
    let o = log.on_key(&s5_key(KeyCode::PageDown));
    assert!(
        matches!(o, Outcome::Changed) && log.scroll.offset == end,
        "A1: PageDown pages back"
    );
    log.on_key(&s5_key(KeyCode::Char('g')));
    assert_eq!(log.scroll.offset, 0, "A1: g jumps to the start");
    log.on_key(&s5_key(KeyCode::Char('G')));
    assert_eq!(log.scroll.offset, end, "A1/A2: G jumps to the live end");
    // A2 headless: f toggles log follow; prose never follows.
    assert!(log.follow, "A2: a tailing log follows at the end");
    log.on_key(&s5_key(KeyCode::Char('f')));
    assert!(!log.follow, "A2: f pauses follow");
    log.on_key(&s5_key(KeyCode::Char('f')));
    assert!(
        log.follow && log.scroll.at_end(),
        "A2: f resumes at the end"
    );
    let mut prose = ScrollPanel::new(s5_id("test.s5.prose"), vec!["a".into(), "b".into()]);
    let mut stage = Stage::new();
    let s5bg = stage.bg();
    let mut buf = sentinel_buf(40, 17);
    prose.render(
        Rect::new(0, 1, 40, 15),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        s5bg,
        plain,
    );
    prose.on_key(&s5_key(KeyCode::Char('f')));
    assert!(!prose.follow, "A2: prose never follows");
    assert_surround_intact(&buf, area, "render: containment");
    capture_isolated(&dir, "scroll-panel", &buf);
    // N1 headless: a fitting column draws no scrollbar and no label.
    let mut fits = ScrollState::new(3);
    fits.set_viewport(5);
    assert_eq!(
        scrollbar::position_label(&fits),
        "",
        "N1: no position label when content fits"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(5, 7);
    let track = Rect::new(2, 1, 1, 5);
    scrollbar::render_vertical(
        track,
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bid,
        &fits,
        false,
    );
    assert_surround_intact(&buf, track, "N1: no scrollbar when content fits");

    // PTY wide: labels, track, thumb, follow toggles.
    // 160x50: the log meta truncates to `· f…` even at 120 columns, so
    // the `· following` suffix needs the widest canonical size.
    let case = s5_case(
        "scroll_region_wide",
        &[
            "--page",
            "scrolling",
            "--motion",
            "paused",
            "--frame",
            "1600",
        ],
        160,
        50,
    );
    write_provenance_named(&dir, &case, "provenance-wide.txt");
    let mut w = support::spawn_boot(&case);
    waits::wait_state(&mut w, "739.63s", "paused log endpoint never showed");
    checkpoint(&mut w, &dir, "wide-boot");
    let wide = live_text(&mut w);
    assert!(
        wide.contains("of 120"),
        "V1 live: the list shows its a–b of n label"
    );
    assert!(
        wide.contains("of 2000 · following"),
        "V1 live: the log adds · following"
    );
    let frame = live_frame(&mut w);
    let mut tracks = 0;
    let mut thumbs = 0;
    for y in 0..frame.rows {
        for x in 0..frame.cols {
            match frame.get(x, y).map(|c| c.symbol.as_str()) {
                Some("│") => tracks += 1,
                Some("┃") => thumbs += 1,
                _ => {}
            }
        }
    }
    assert!(tracks > 0, "V2 live: overflowing columns draw the │ track");
    assert!(thumbs > 0, "V2 live: the ┃ thumb sits at the offset");
    tab_until(&mut w, "Follow", case_timeout(&case), "log never focused");
    press(&mut w, "f");
    let paused = settle_text(&mut w, case_timeout(&case), "A2 steady");
    assert!(
        !paused.contains("following"),
        "A2 live: f pauses follow (presence: the labels stay)"
    );
    assert!(paused.contains("of 2000"), "A2 live: the log window stays");
    press(&mut w, "f");
    waits::wait_state(&mut w, "following", "A2 live: f never resumed follow");

    // PTY narrow: wheel-under-pointer, keys-to-focused, pages, ends.
    let case2 = s5_case(
        "scroll_region",
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
    write_provenance_named(&dir, &case2, "provenance-narrow.txt");
    let mut s = support::spawn_boot(&case2);
    waits::wait_state(&mut s, "739.63s", "narrow endpoint never showed");
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("1–15 of"), "V1 live: prose boots at 1–15");
    // Narrow clips the total (`Log 1986–2000 …`); the window pins the cap
    // here and the wide boot above proved the full `of 2000`.
    assert!(
        boot.contains("1986–2000"),
        "V1/S1 live: the log caps at 2000"
    );
    // S2 live: the wheel routes under the pointer; the rest stay put.
    // One notch is three lines (the app maps each notch to delta ±3).
    wheel_over(&mut s, "739.63s", Wheel::Up, 1, case_timeout(&case2));
    waits::wait_state(&mut s, "1983–1997", "S2 live: the log never wheeled");
    let wheeled = live_text(&mut s);
    assert!(
        wheeled.contains("1–15 of"),
        "S2/N2 live: prose and list keep 1–15"
    );
    // S2/N2 live: keys route to the focused container only.
    press(&mut s, "tab");
    waits::wait_state(&mut s, "PgUp PgDn", "prose never focused");
    press(&mut s, "pagedown");
    waits::wait_state(&mut s, "16–30", "A1 live: prose never paged");
    let paged = live_text(&mut s);
    assert_line_has(&paged, "16–30", "▎", "A1 live: the paged prose owns focus");
    assert!(
        paged.contains("1983–1997"),
        "N2 live: keys move no unfocused container"
    );
    assert!(paged.contains("Row 001"), "N2 live: the list rows stay");
    // A1/A2 live: g/G ends on the log; G returns to the live end.
    tab_until(&mut s, "Follow", case_timeout(&case2), "log never focused");
    press(&mut s, "g");
    // Narrow clips the total here too, so the title+window pins home.
    waits::wait_state(&mut s, "Log 1–15", "A1 live: g never jumped home");
    press(&mut s, "G");
    waits::wait_state(&mut s, "1986–2000", "A2 live: G never jumped to the end");
    checkpoint(&mut s, &dir, "ends");

    // PTY playing: the log grows a line per tick while follow holds.
    // 120x40: narrow clips every total (`of …`), starving the parser.
    let case3 = s5_case("scroll_region_live", &["--page", "scrolling"], 120, 40);
    write_provenance_named(&dir, &case3, "provenance-live.txt");
    let mut p = support::spawn_boot(&case3);
    fn max_of(text: &str) -> usize {
        text.split("of ")
            .skip(1)
            .filter_map(|tail| {
                tail.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<usize>()
                    .ok()
            })
            .max()
            .unwrap_or(0)
    }
    let n1 = max_of(&live_text(&mut p));
    assert!(
        n1 >= 400,
        "S1 live: the log boots with its 400 lines ({n1})"
    );
    std::thread::sleep(Duration::from_millis(800));
    let n2 = max_of(&live_text(&mut p));
    assert!(
        n2 > n1,
        "S1 live: the log grows while follow holds ({n1} → {n2})"
    );
    checkpoint(&mut p, &dir, "growing");
    eprintln!("s5 scroll_region: columns + routing + growth + cap");
}

/// SCROLL-TRACK-002 (`showcase/flows/scrolling/scrolled`, 80x24): the
/// scrollbar track press plus thumb drag.
///
/// Isolated path drives [`ScrollState`] over a 2000-line log mirror plus
/// the [`scrollbar`] wrappers: proportional thumb geometry with the
/// 1-cell floor, thumb-press holds vs track-press centering, grab-kept
/// drags, press-fallback drags, release clearing, repaint-vs-noop
/// booleans, and the no-overflow no-ops. Live path boots the frame-1600
/// scrolling page, presses the log thumb (holds), drags it two rows
/// (follows), and presses the bare track (jumps centered).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_scroll_track() {
    let dir = s5_dir("s5_scroll_track");
    let bid = s5_id("test.s5.scroll");

    // V1 headless: the thumb spans viewport/content of the track, ≥1 cell.
    let mut log = ScrollState::new(2000);
    log.set_viewport(15);
    assert_eq!(log.thumb(15), (0, 1), "V1: top thumb spans one cell");
    log.jump_end();
    assert_eq!(log.thumb(15), (14, 1), "V1: end thumb sits at the bottom");
    assert_eq!(
        scrollbar::position_label(&log),
        "1986–2000 of 2000",
        "V1 live shape: the tail label"
    );
    // V2/S1 headless: a thumb press holds the offset (grab only).
    let mut held = ScrollState::new(2000);
    held.set_viewport(15);
    assert!(!held.press_track(0, 15), "V2: pressing the thumb holds");
    assert_eq!(held.offset, 0, "N1: a thumb press moves nothing");
    // S1: dragging keeps the grabbed row under the pointer.
    assert!(held.drag_track(2, 15), "S1: the drag moves");
    assert_eq!(held.thumb(15).0, 2, "S1: the thumb follows the pointer");
    // V2/A2 headless: a track press centers the thumb under the pointer.
    let mut jumped = ScrollState::new(2000);
    jumped.set_viewport(15);
    assert!(!jumped.press_track(0, 15), "V2: top press holds at top");
    assert!(jumped.press_track(14, 15), "V2: a track press jumps");
    let (start, len) = jumped.thumb(15);
    assert!(
        (start..start + len).contains(&14),
        "A2: offset_for_track_pos centers the thumb"
    );
    for pos in [0usize, 7, 14] {
        let mut probe = ScrollState::new(2000);
        probe.set_viewport(15);
        probe.scroll_to(probe.offset_for_track_pos(pos, 15));
        let (start, len) = probe.thumb(15);
        assert!(
            (start..start + len).contains(&pos),
            "A2: track pos {pos} centers"
        );
    }
    // S2: a drag without a press behaves like a press; release clears.
    let mut fresh = ScrollState::new(2000);
    fresh.set_viewport(15);
    assert!(fresh.drag_track(14, 15), "S2: press-less drag jumps");
    assert!(fresh.at_end(), "S2: the jump reaches the end");
    fresh.release_track();
    fresh.scroll_to(0);
    assert!(!fresh.press_track(0, 15), "S2: a new press grabs again");
    // A1: press/drag report repaint-vs-noop.
    let mut edge = ScrollState::new(2000);
    edge.set_viewport(15);
    assert!(!edge.press_track(0, 15), "A1: thumb press is a noop false");
    assert!(!edge.drag_track(0, 15), "A1: null drag is a noop false");
    // N2: without overflow, press and drag change nothing.
    let mut fits = ScrollState::new(3);
    fits.set_viewport(5);
    assert!(!fits.press_track(2, 5), "N2: press is a noop");
    assert!(!fits.drag_track(4, 5), "N2: drag is a noop");
    assert_eq!(fits.offset, 0, "N2: the offset never moves");
    // The scrollbar wrappers map through the same core.
    let track = Rect::new(0, 0, 1, 15);
    let mut wrapped = ScrollState::new(2000);
    wrapped.set_viewport(15);
    assert!(
        !scrollbar::press(track, Position::new(0, 0), &mut wrapped),
        "wrappers: thumb press holds"
    );
    assert!(
        scrollbar::drag(track, Position::new(0, 2), &mut wrapped),
        "wrappers: drag follows"
    );
    assert_eq!(wrapped.thumb(15).0, 2, "wrappers: grab kept");
    // Render form: │ track with the ┃ thumb, contained.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(5, 17);
    let area = Rect::new(2, 1, 1, 15);
    wrapped.scroll_to(0);
    scrollbar::render_vertical(
        area,
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bid,
        &wrapped,
        true,
    );
    assert_eq!(buf[(2, 1)].symbol(), "┃", "V2: the thumb leads at the top");
    assert_eq!(buf[(2, 2)].symbol(), "│", "V2: the track follows");
    assert_surround_intact(&buf, area, "render: containment");
    capture_isolated(&dir, "scroll-thumb", &buf);

    // PTY: press the thumb (holds), drag two rows (follows), track jumps.
    let case = s5_case(
        "scroll_track",
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
    waits::wait_state(&mut s, "739.63s", "endpoint never showed");
    checkpoint(&mut s, &dir, "booted");
    // Locate the log scrollbar: the rightmost │/┃ run.
    let frame = live_frame(&mut s);
    let mut track_col = 0u16;
    let mut track_top = 0u16;
    let mut thumb_rows: Vec<u16> = vec![];
    for x in (60..80u16).rev() {
        let mut cells = 0;
        let mut top = 0u16;
        let mut thumbs = vec![];
        for y in 0..frame.rows {
            match frame.get(x, y).map(|c| c.symbol.as_str()) {
                Some("│") => {
                    if cells == 0 {
                        top = y;
                    }
                    cells += 1;
                }
                Some("┃") => {
                    if cells == 0 {
                        top = y;
                    }
                    cells += 1;
                    thumbs.push(y);
                }
                _ => {}
            }
        }
        if cells >= 10 && !thumbs.is_empty() {
            track_col = x;
            track_top = top;
            thumb_rows = thumbs;
            break;
        }
    }
    assert!(!thumb_rows.is_empty(), "V2 live: the log thumb renders");
    let thumb_mid = thumb_rows[thumb_rows.len() / 2];
    // V2/S1/N1 live: pressing the thumb holds the offset.
    s.mouse_down(MouseButton::Left, track_col, thumb_mid, MouseMods::NONE)
        .expect("thumb press");
    std::thread::sleep(Duration::from_millis(300));
    let held_text = live_text(&mut s);
    assert!(
        held_text.contains("1986–2000"),
        "V2/N1 live: a thumb press holds the offset"
    );
    checkpoint(&mut s, &dir, "grabbed");
    // S1 live: dragging keeps the grabbed row under the pointer.
    let target = thumb_mid.saturating_sub(2);
    s.mouse_drag(MouseButton::Left, track_col, target, MouseMods::NONE)
        .expect("thumb drag");
    std::thread::sleep(Duration::from_millis(300));
    let dragged = live_text(&mut s);
    assert!(
        !dragged.contains("1986–2000"),
        "S1 live: the drag moves the view"
    );
    let frame = live_frame(&mut s);
    let mut moved: Vec<u16> = vec![];
    for y in 0..frame.rows {
        if frame.get(track_col, y).is_some_and(|c| c.symbol == "┃") {
            moved.push(y);
        }
    }
    assert_eq!(
        moved.first().copied().unwrap_or(999),
        thumb_rows[0] - 2,
        "S1 live: the thumb sits two rows up"
    );
    s.mouse_up(MouseButton::Left, track_col, target, MouseMods::NONE)
        .expect("thumb release");
    std::thread::sleep(Duration::from_millis(300));
    // V2/A2 live: a bare-track press jumps the thumb under the pointer.
    // Two rows inside the top: the first row may sit above the track's
    // hit rect (a border │), which would swallow the press silently.
    let slot = track_top + 2;
    s.mouse_down(MouseButton::Left, track_col, slot, MouseMods::NONE)
        .expect("track press");
    s.mouse_up(MouseButton::Left, track_col, slot, MouseMods::NONE)
        .expect("track release");
    std::thread::sleep(Duration::from_millis(300));
    checkpoint(&mut s, &dir, "jumped");
    let frame = live_frame(&mut s);
    let mut under: Vec<u16> = vec![];
    for y in 0..frame.rows {
        if frame.get(track_col, y).is_some_and(|c| c.symbol == "┃") {
            under.push(y);
        }
    }
    assert_eq!(
        under.as_slice(),
        &[slot],
        "V2/A2 live: the press puts the thumb under the pointer"
    );
    // The narrow meta clips the total, so the window leaving the end
    // plus the exact thumb geometry above is the jump proof.
    let jumped = live_text(&mut s);
    assert!(
        !jumped.contains("1986–2000"),
        "V2 live: the view jumps with the thumb"
    );
    eprintln!("s5 scroll_track: thumb grab + drag + track jump");
}
