//! Showcase pending slice 8A-S5 executable checks — impl port.
//!
//! Ported from VB commit `68d98ac99238c8580464f238a65b0f862ecc3dbd`
//! (`tests/harness/tests/visual_baseline/showcase_pending_s5.rs`), adapting
//! only paths, binary resolution, harness API, and the isolated
//! component-path translation:
//!
//! - `Session` is the impl [`support::Session`] wrapper (live samples via
//!   `s.inner.observe_now()` / `s.inner.snapshot()`), not
//!   `tuiscotti::tui::Session` directly.
//! - The subject is the binary built from this impl worktree's sources,
//!   resolved via [`support::try_resolve_bin`] (name -> executed path +
//!   sha256); [`write_provenance`] records path, digest, size, mtime, and
//!   argv behind each check.
//! - `support::frame_from_screen` (VB helper over `Provenance`) becomes
//!   `tuiscotti::render::frame_from_screen` (provenance `"default"`).
//! - Isolated `junie_tui` in-process renders (`HintBar`, `HintLayer`,
//!   `keyhint`, `DataGrid`, `CodeEditor`, `DiffView`, `TextViewport`,
//!   `ScrollPanel`, `scrollbar`, `ScrollState`) become the established impl
//!   component path: `Runtime` + `Stub` + `draw_scene` with
//!   `ui.reference` state injection for inert paint, and `Harness` apps
//!   for interaction. `Outcome` assertions become draw/behavior
//!   correlates (flow, committed values, re-rendered glyphs, selection
//!   paint, actions): `Changed` is `Consumed` plus the asserted effect,
//!   caller-owned validation stays caller-owned, and caller-owned label
//!   composition (position labels, pending breakdowns) is mirrored from
//!   the owning page and asserted through the component facts it reads.
//!
//! One ignored test per S5 registry row (14 rows: the last 14 unrun
//! showcase rows in registry order after S4 — the 3 FEEDBACK hint rows
//! plus the 11 DATA-VIEWS rows), over the real binaries built from this
//! worktree's impl sources and over the real production components
//! rendered headless. Each test executes its row's listed adapters
//! (`headless-tick`, `pty-capture`, or both) and asserts its visual (V),
//! state (S), action (A), and negative (N) checks in executable form:
//!
//! - Isolated path: the production component renders into a `·`-sentinel
//!   buffer (any 1-cell overflow fails the surround check), so glyphs,
//!   tones, bold, widths, hit regions, focus-ring stops, and actions are
//!   asserted exactly.
//! - PTY path: the live app boots the row's page and drives the row's
//!   inputs; V cells are probed live (needles plus fg/bg/bold cell
//!   reads), S labels are read live, A outcomes are proven by their
//!   screen correlates (moved, committed, reverted, byte-identical),
//!   since PTY cannot see the action, and every N absence is paired with
//!   a presence proof in the same test so no absence passes vacuously.
//!
//! One row is PTY-only by construction: TERM-RUN-001 (the stage engine,
//! reset, and advance live in the showcase binary's terminal page).
//! None of it is importable from this conformance crate, so no headless
//! path exists for it; every one of its checks has a live-PTY executable
//! form instead.
//!
//! No new snapshots and no new static captures: every S5 row already has
//! approved frames gated cell-exact by the ported matrices (`showcase.rs`,
//! `pointer.rs`, `audit.rs`), so every case here uses owned (dynamic)
//! names and stays out of the `snapshots/` inventory.
//!
//! Per-test scratch lands under this conformance crate's
//! `target/tuiscotti/journeys/<test>/` (gitignored): the row's named
//! checkpoint(s) as text frames, `provenance.txt` (binary path, digest,
//! size, mtime, argv), and `isolated-*.json` (headless captures). Typed
//! input is synthetic and in-memory only (simulation data).
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

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

use termrock::Color as RColor;
use termrock::Input as RtInput;
use termrock::runtime::stub::{Stub, deliver};
use termrock::{
    Align, App, Axis, Buffer, Button, CellDecor, CellRef, CodeAction, CodeDiagnostic, CodeEditor,
    CodeEditorState, CodeSeverity, Column, ColumnKey, Cx, DiffLineKind, DiffMode, DiffRow,
    DiffSource, DiffView, DiffViewState, EditIntent, EmptyState, FgStep, FieldError, Flow,
    GlyphRole, Grid, GridAction, GridColumnFit, GridEditor, GridGutter, GridModel,
    GridOverflowIndicator, GridSortIndicator, GridState, Hint, HintBar, HintKey, HintLayer, Id,
    Invalidate, ItemKey, KeyCode, Modifier, MouseKind, NavUnit, Position, Rect, ReferenceState,
    ReferenceTarget, Response, Role, RowDecor, RowTotal, Runtime, ScrollRegion, ScrollState,
    Status, SyntaxRole, TabBehavior, TextViewport, Theme, Ui, ViewportAction, ViewportLine,
    ViewportState,
};
use termrock_test_support::Harness;
use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::{MouseButton, MouseMods, Wheel};
use tuiscotti::{Cell as TCell, Color as TColor, Frame, Mods, Provenance, Rgb, UnderlineStyle};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

/// The VB oracle commit this file ports.
const PORT_OF: &str = "68d98ac99238c8580464f238a65b0f862ecc3dbd";

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
        "profile: tuiscotti-default\ntransport: pty\nport_of: {PORT_OF}\nbinary: {}\nsha256: {}\nsize_bytes: {}\nmodified_unix: {modified}\nargv: {}\n",
        subject.path.display(),
        subject.sha256,
        subject.len,
        argv.join(" "),
    );
    std::fs::write(dir.join(file), &body)
        .unwrap_or_else(|e| panic!("write {}: {e}", dir.join(file).display()));
    eprintln!("s5 provenance ({file}): {body}");
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
    s.type_text(text).expect("type_text");
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

/// Buffer pre-filled with `·` sentinels; the component renders into a
/// sub-area and every cell outside must still be `·` (no 1-cell overflow).
fn sentinel_buf(cols: u16, rows: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, cols, rows));
    for y in 0..rows {
        for x in 0..cols {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_symbol("·");
            }
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

/// One inert component-path render into a sentinel buffer: `ticks`
/// animation ticks delivered first, then one `draw_scene` with `target`
/// injected via `ui.reference` (the `Interaction` equivalent: focus,
/// hover, press owned by the runtime, semantic state by props).
fn render_isolated(
    cols: u16,
    rows: u16,
    ticks: u64,
    target: Option<(Id, ReferenceState)>,
    draw: impl FnOnce(&mut Ui<'_>, Rect),
) -> Buffer {
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();
    for _ in 0..ticks {
        let _ = deliver(&mut runtime, RtInput::Tick);
    }
    let view = Rect::new(0, 0, cols, rows);
    let mut buf = sentinel_buf(cols, rows);
    runtime
        .draw_scene(view, &mut buf, |ui, area| {
            if let Some((id, state)) = target {
                ui.reference(Some(ReferenceTarget::new(id, state)), |ui| draw(ui, area));
            } else {
                draw(ui, area);
            }
        })
        .commit_presented();
    buf
}

/// Colour conversion mirroring `tuiscotti::ratatui` (private there): named
/// ANSI colours become their palette index, the rest map one-to-one.
fn convert_color(c: RColor) -> TColor {
    match c {
        RColor::Reset => TColor::Default,
        RColor::Black => TColor::Indexed(0),
        RColor::Red => TColor::Indexed(1),
        RColor::Green => TColor::Indexed(2),
        RColor::Yellow => TColor::Indexed(3),
        RColor::Blue => TColor::Indexed(4),
        RColor::Magenta => TColor::Indexed(5),
        RColor::Cyan => TColor::Indexed(6),
        RColor::Gray => TColor::Indexed(7),
        RColor::DarkGray => TColor::Indexed(8),
        RColor::LightRed => TColor::Indexed(9),
        RColor::LightGreen => TColor::Indexed(10),
        RColor::LightYellow => TColor::Indexed(11),
        RColor::LightBlue => TColor::Indexed(12),
        RColor::LightMagenta => TColor::Indexed(13),
        RColor::LightCyan => TColor::Indexed(14),
        RColor::White => TColor::Indexed(15),
        RColor::Indexed(i) => TColor::Indexed(i),
        RColor::Rgb(r, g, b) => TColor::Rgb(Rgb::new(r, g, b)),
    }
}

/// Modifier conversion mirroring `tuiscotti::ratatui` (private there).
fn convert_mods(m: Modifier) -> Mods {
    Mods {
        hidden: m.contains(Modifier::HIDDEN),
        blink: m.intersects(Modifier::SLOW_BLINK | Modifier::RAPID_BLINK),
        bold: m.contains(Modifier::BOLD),
        dim: m.contains(Modifier::DIM),
        italic: m.contains(Modifier::ITALIC),
        underline: m.contains(Modifier::UNDERLINED),
        underline_style: if m.contains(Modifier::UNDERLINED) {
            UnderlineStyle::Single
        } else {
            UnderlineStyle::None
        },
        strikethrough: m.contains(Modifier::CROSSED_OUT),
        reverse: m.contains(Modifier::REVERSED),
    }
}

/// Capture a candidate buffer through the component path. The conversion
/// mirrors `tuiscotti::ratatui::from_buffer` (wide cells, continuations,
/// cursor-less): empty symbols become continuation followers, wide symbols
/// emit a lead plus a follower (downgraded at the exact row end).
fn capture_buffer(buf: &Buffer) -> Frame {
    let cols = buf.area.width;
    let rows = buf.area.height;
    let mut frame = Frame::blank(
        cols,
        rows,
        Provenance::now("default", "s5-isolated", vec![]),
    );
    for y in 0..rows {
        let mut x = 0u16;
        while x < cols {
            let Some(rc) = buf.cell((x, y)) else {
                x += 1;
                continue;
            };
            if rc.symbol().is_empty() {
                let mut cont = TCell::blank(x, y);
                cont.fg = convert_color(rc.fg);
                cont.bg = convert_color(rc.bg);
                cont.mods = convert_mods(rc.modifier);
                cont.underline_color = convert_color(rc.underline_color);
                cont.width = 0;
                cont.continuation = true;
                cont.symbol = String::new();
                frame.set(cont);
                x += 1;
                continue;
            }
            let symbol = rc.symbol().to_string();
            let width = u8::try_from(unicode_width::UnicodeWidthStr::width(symbol.as_str()).min(2))
                .unwrap_or(u8::MAX)
                .max(1);
            let lead = TCell {
                x,
                y,
                symbol,
                width,
                continuation: false,
                fg: convert_color(rc.fg),
                bg: convert_color(rc.bg),
                mods: convert_mods(rc.modifier),
                underline_color: convert_color(rc.underline_color),
            };
            if width == 2 && x + 1 >= cols {
                let mut narrow = lead;
                narrow.width = 1;
                frame.set(narrow);
                x += 1;
                continue;
            }
            frame.set(lead);
            if width == 2 && x + 1 < cols {
                let mut cont = TCell::blank(x + 1, y);
                cont.fg = convert_color(rc.fg);
                cont.bg = convert_color(rc.bg);
                cont.mods = convert_mods(rc.modifier);
                cont.underline_color = convert_color(rc.underline_color);
                cont.width = 0;
                cont.continuation = true;
                cont.symbol = String::new();
                frame.set(cont);
                x += 2;
            } else {
                x += 1;
            }
        }
    }
    frame
}

/// Headless capture plus JSON evidence in the test dir.
fn capture_isolated(dir: &Path, name: &str, buf: &Buffer) -> Frame {
    let frame = capture_buffer(buf);
    std::fs::write(
        dir.join(format!("isolated-{name}.json")),
        frame.to_json_pretty(),
    )
    .unwrap_or_else(|e| panic!("write isolated-{name}.json: {e}"));
    eprintln!("isolated {name}: digest {:016x}", frame.digest());
    frame
}

/// Cell column of `needle`'s first occurrence in `line` (char-based: byte
/// offsets lie when multibyte sentinels or glyphs precede the needle).
fn char_col(line: &str, needle: &str) -> u16 {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {line:?}"));
    line[..byte].chars().count() as u16
}

/// One `key label` hint pair (the VB `hint()` helper's component-path form:
/// descriptive keycaps declare no routing chord).
fn s5_hint(key: &'static str, label: &'static str) -> Hint {
    Hint {
        key: HintKey::Label(key),
        label,
        priority: 10,
    }
}

/// HB-LAYERS-001 (`showcase/pages/chrome`, 80x24): the shell-owned hint
/// surface — layer precedence, the zoom badge, and the pinned status.
///
/// Isolated path proves [`HintBar::resolve`] order (menu › context ›
/// screen › empty), the badge-preserving layer, the menu row's exact
/// glyphs, and the single-row shape (a modal wins the one row; there is
/// no second row to paint). Live path boots the chrome page, focuses the
/// bar (bar layer), opens the View menu (menu layer), and chooses Zoom
/// pane (badge + status, screen layer restored).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_hint_layers() {
    let dir = s5_dir("s5_hint_layers");
    let menu = HintLayer {
        hints: vec![
            s5_hint("↑↓", "Move"),
            s5_hint("← →", "Switch menu"),
            s5_hint("Enter", "Choose"),
            s5_hint("Esc", "Close"),
        ],
        badge: None,
        status: None,
        centered: false,
    };
    let context = HintLayer {
        hints: vec![
            s5_hint("↑↓", "Move"),
            s5_hint("Enter", "Choose"),
            s5_hint("Esc", "Close"),
        ],
        badge: None,
        status: None,
        centered: false,
    };
    let screen = HintLayer {
        hints: vec![
            s5_hint("↑↓", "Move"),
            s5_hint("m", "Context menu"),
            s5_hint("right-click", "Context menu"),
            s5_hint("Tab", "Next"),
        ],
        badge: None,
        status: Some("last: nothing yet".into()),
        centered: false,
    };

    // S1: resolve takes the first present layer: menu, context, screen.
    assert_eq!(
        HintBar::resolve(&[Some(&menu), Some(&context), Some(&screen)]),
        Some(&menu),
        "S1: menu wins over context and screen"
    );
    assert_eq!(
        HintBar::resolve(&[None, Some(&context), Some(&screen)]),
        Some(&context),
        "S1: context wins over screen"
    );
    assert_eq!(
        HintBar::resolve(&[None, None, Some(&screen)]),
        Some(&screen),
        "S1: screen wins over nothing"
    );
    assert_eq!(
        HintBar::resolve(&[None, None, None]),
        None,
        "S1: no layer resolves to no layer"
    );
    assert!(
        HintLayer::empty().is_empty(),
        "S1: the empty fallback contributes nothing"
    );
    // S2: zooming adds the badge without changing the hints.
    let zoomed = HintLayer {
        badge: Some("ZOOM"),
        ..screen.clone()
    };
    assert_eq!(zoomed.hints, screen.hints, "S2: badge keeps the hints");
    assert_eq!(zoomed.status, screen.status, "S2: badge keeps the status");

    // V1/V2 render form: the menu row's glyphs plus the badge lead.
    let buf = render_isolated(80, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &menu).draw(ui, Rect::new(0, 1, 80, 1));
    });
    let area = Rect::new(0, 1, 80, 1);
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("Move")
            && row.contains("Switch menu")
            && row.contains("Choose")
            && row.contains("Close"),
        "V1: the menu layer swaps the footer\n{row:?}"
    );
    assert!(
        !row.contains('…'),
        "V1: all four menu hints fit (no cut marker)\n{row:?}"
    );
    assert_surround_intact(&buf, area, "N1: one row only");
    capture_isolated(&dir, "hintbar-menu", &buf);
    let buf = render_isolated(80, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &zoomed).draw(ui, Rect::new(0, 1, 80, 1));
    });
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
/// Isolated path renders [`HintBar`] with toned readiness into sentinel
/// buffers: error/warning marks with bold mark cells in tone, the plain
/// secondary form, the mark-follows-readiness table, the width-plus-3
/// hint reservation, and the only-the-mark-is-bold scan. Live path boots
/// the chrome page, runs two menu actions (status replacement), and
/// clicks the status (no input: byte-identical screen).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_keyhint_status() {
    let dir = s5_dir("s5_keyhint_status");
    let t = Theme::junie();
    let hints = vec![s5_hint("↑↓", "Move"), s5_hint("m", "Context menu")];
    let toned = |status: &'static str| HintLayer {
        hints: hints.clone(),
        badge: None,
        status: Some(status.into()),
        centered: false,
    };

    // V1: an error status renders `! message` in error with a bold mark;
    // a warning renders `▲ message`. (Tone arrives through the bar's
    // readiness prop; the layer owns only the text.)
    let layer = toned("disk low");
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &layer)
            .status(Status::Error)
            .draw(ui, Rect::new(0, 1, 60, 1));
    });
    let area = Rect::new(0, 1, 60, 1);
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("! disk low"),
        "V1: error mark + message\n{row:?}"
    );
    let mark_x = char_col(&row, "!");
    assert!(buf_is_bold(&buf, mark_x, 1), "V1: the error mark is bold");
    assert_eq!(
        buf[(mark_x, 1)].fg,
        t.color.danger,
        "V1: the error mark takes error"
    );
    assert_eq!(
        buf[(mark_x + 2, 1)].fg,
        t.color.danger,
        "V1: the message takes error too"
    );
    assert!(
        !buf_is_bold(&buf, mark_x + 2, 1),
        "N2: the message is never bold"
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "keyhint-error", &buf);
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &layer)
            .status(Status::Warning)
            .draw(ui, Rect::new(0, 1, 60, 1));
    });
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("▲ disk low"),
        "V1: warning mark + message\n{row:?}"
    );
    let mark_x = char_col(&row, "▲");
    assert!(buf_is_bold(&buf, mark_x, 1), "V1: the warning mark is bold");
    assert_eq!(
        buf[(mark_x, 1)].fg,
        t.color.warning,
        "V1: the warning mark takes warning"
    );
    // V2 + N1: a plain secondary status renders unmarked in its tone and
    // never carries ▲ or !.
    let layer = toned("last: View › Zoom pane");
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &layer).draw(ui, Rect::new(0, 1, 60, 1));
    });
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
        t.color.fg[FgStep::Secondary.index()],
        "V2: the plain status takes its tone"
    );
    assert!(
        (status_x..60).all(|x| !buf_is_bold(&buf, x, 1)),
        "N2: no bold anywhere in a plain status"
    );
    // S1: the mark follows the readiness: Error !, Warning ▲, Ready none,
    // Busy/Loading a spinner frame.
    let spinner: &str = t.design.motion.spinner_frames[0];
    for (status, mark) in [
        (Status::Error, Some("!")),
        (Status::Warning, Some("▲")),
        (Status::Ready, None),
    ] {
        let layer = toned("msg");
        let buf = render_isolated(60, 1, 0, None, |ui, _| {
            HintBar::new(Id::root("s5.hint"), &layer)
                .status(status)
                .draw(ui, Rect::new(0, 0, 60, 1));
        });
        let row = buf_row_text(&buf, 0);
        match mark {
            Some(m) => assert!(
                row.contains(&format!("{m} msg")),
                "S1: {status:?} carries {m:?}\n{row:?}"
            ),
            None => {
                assert!(row.contains("msg"), "S1: {status:?} renders\n{row:?}");
                assert!(
                    !row.contains('▲') && !row.contains('!'),
                    "S1: {status:?} carries no mark\n{row:?}"
                );
            }
        }
    }
    for status in [Status::Busy, Status::Loading] {
        let layer = toned("msg");
        let buf = render_isolated(60, 1, 0, None, |ui, _| {
            HintBar::new(Id::root("s5.hint"), &layer)
                .status(status)
                .draw(ui, Rect::new(0, 0, 60, 1));
        });
        let row = buf_row_text(&buf, 0);
        assert!(
            row.contains(&format!("{spinner} msg")),
            "S1: {status:?} spins before the message\n{row:?}"
        );
    }
    // S2: the status reserves its width plus 3 from the hint limit — the
    // hint block ends where the status reservation begins, never under it.
    let many = HintLayer {
        hints: vec![
            s5_hint("↑↓", "Move"),
            s5_hint("m", "Context menu"),
            s5_hint("right-click", "Context menu"),
            s5_hint("Tab", "Next"),
        ],
        badge: None,
        status: None,
        centered: false,
    };
    let buf = render_isolated(80, 1, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &many).draw(ui, Rect::new(0, 0, 80, 1));
    });
    let row = buf_row_text(&buf, 0);
    for label in ["Move", "Context menu", "Next"] {
        assert!(
            row.contains(label),
            "S2: all four hints fit with no status\n{row:?}"
        );
    }
    assert!(!row.contains('…'), "S2: no cut without a status");
    let many_status = HintLayer {
        status: Some("last: View › Zoom pane".into()),
        ..many.clone()
    };
    let buf = render_isolated(80, 1, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &many_status).draw(ui, Rect::new(0, 0, 80, 1));
    });
    let row = buf_row_text(&buf, 0);
    assert!(
        row.contains('…'),
        "S2: the status reservation drops hints\n{row:?}"
    );
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
/// Isolated path renders [`HintBar`] into sentinel buffers:
/// drop-from-the-right with the faint `…`, the leading badge block,
/// centered mid-row measurement with badge/status clamps, the drawn
/// count with its 2-cell cut reserve, and the 1..16-cell containment
/// sweep. Live path boots the chrome page at 72x20 (dropped hints +
/// `…`), zooms for the badge, clicks the row (no input), and widens to
/// 120x30 (dropped hints redrawn).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_keyhint_drop() {
    let dir = s5_dir("s5_keyhint_drop");
    let t = Theme::junie();
    let hints = vec![
        s5_hint("Enter", "Open"),
        s5_hint("Space", "Choose"),
        s5_hint("g", "Git URL"),
        s5_hint("Tab", "Next"),
        s5_hint("Esc", "Cancel"),
    ];
    let layer = HintLayer {
        hints,
        badge: None,
        status: None,
        centered: false,
    };

    // V1: hints drop from the right and a faint … marks the cut.
    let buf = render_isolated(30, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &layer).draw(ui, Rect::new(0, 1, 30, 1));
    });
    let area = Rect::new(0, 1, 30, 1);
    let row = buf_row_text(&buf, 1);
    // S1: exactly the first two hints draw (the drawn-count correlate:
    // head present, mid and tail absent, cut marked).
    assert!(
        row.contains("Enter")
            && row.contains("Open")
            && row.contains("Space")
            && row.contains("Choose"),
        "V1/S1: the two-head stays\n{row:?}"
    );
    assert!(
        !row.contains("Git URL") && !row.contains("Next") && !row.contains("Cancel"),
        "V1/S1: mid and tail drop\n{row:?}"
    );
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
        t.color.fg[FgStep::Faint.index()],
        "V1: the cut marker is faint"
    );
    assert_surround_intact(&buf, area, "N1: containment");
    capture_isolated(&dir, "keyhint-drop", &buf);
    // V2: a badge renders as ` TEXT ` leading; centered rows sit mid-row.
    let badged = HintLayer {
        badge: Some("ZOOM"),
        status: Some("last: View › Zoom pane".into()),
        ..layer.clone()
    };
    let buf = render_isolated(72, 3, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &badged).draw(ui, Rect::new(0, 1, 72, 1));
    });
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
    let centered = HintLayer {
        centered: true,
        ..layer.clone()
    };
    let buf = render_isolated(72, 1, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &centered).draw(ui, Rect::new(0, 0, 72, 1));
    });
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
    let tight = HintLayer {
        badge: Some("ZOOM"),
        status: Some("last: View › Zoom pane".into()),
        centered: true,
        ..layer.clone()
    };
    let buf = render_isolated(40, 1, 0, None, |ui, _| {
        HintBar::new(Id::root("s5.hint"), &tight).draw(ui, Rect::new(0, 0, 40, 1));
    });
    let row = buf_row_text(&buf, 0);
    let badge_end = row.find("ZOOM").unwrap() + "  ZOOM ".len() - 1;
    let status_x = row.find("last:").unwrap();
    let block_start = row.find("Enter").unwrap_or(status_x);
    assert!(
        block_start >= badge_end,
        "S2/N2: centered block never slides under the badge or status\n{row:?}"
    );
    if row.contains("Enter") {
        assert!(
            block_start >= badge_end,
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
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let _ = runtime.initialize();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
        for cell in buf.content.iter_mut() {
            cell.set_symbol(".");
        }
        let narrow = HintLayer {
            hints: vec![s5_hint("Esc", "Cancel")],
            badge: Some("EDIT"),
            status: Some("long failure message".into()),
            centered: width % 2 == 0,
        };
        runtime
            .draw_scene(Rect::new(0, 0, 20, 3), &mut buf, |ui, _| {
                HintBar::new(Id::root("s5.hint"), &narrow)
                    .status(Status::Error)
                    .draw(ui, area);
            })
            .commit_presented();
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

// ------------------------------------------------------------- grid rig --

const S5_GRID: Id = Id::root("s5.grid");

const S5_COL_CUSTOMER: usize = 1;
const S5_COL_PLAN: usize = 2;
const S5_COL_SEATS: usize = 3;
const S5_COL_ACTIVE: usize = 5;
const S5_COL_RENEWED: usize = 6;
const S5_COL_NOTES: usize = 7;

/// Page-shaped grid columns (the customers table: id primary read-only,
/// mrr read-only, the rest writable). Widths mirror the datagrid page's
/// sampled `COLUMNS`.
const S5_COLUMNS: [Column<'static>; 8] = [
    Column {
        key: ColumnKey::num(0),
        title: "id",
        subtitle: None,
        align: Align::Left,
        min_width: 9,
        max_width: 9,
        flex: 0,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: Some(GlyphRole::PrimaryKey),
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(1),
        title: "customer",
        subtitle: None,
        align: Align::Left,
        min_width: 25,
        max_width: 25,
        flex: 0,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(2),
        title: "plan",
        subtitle: None,
        align: Align::Left,
        min_width: 10,
        max_width: 10,
        flex: 0,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(3),
        title: "seats",
        subtitle: None,
        align: Align::Right,
        min_width: 7,
        max_width: 7,
        flex: 0,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(4),
        title: "mrr",
        subtitle: None,
        align: Align::Right,
        min_width: 7,
        max_width: 7,
        flex: 0,
        sortable: true,
        editable: false,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(5),
        title: "active",
        subtitle: None,
        align: Align::Left,
        min_width: 8,
        max_width: 8,
        flex: 0,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(6),
        title: "renewed_at",
        subtitle: None,
        align: Align::Left,
        min_width: 12,
        max_width: 12,
        flex: 0,
        sortable: true,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
    Column {
        key: ColumnKey::num(7),
        title: "notes",
        subtitle: None,
        align: Align::Left,
        min_width: 27,
        max_width: 27,
        flex: 0,
        sortable: false,
        editable: true,
        sticky: false,
        prefix_glyph: None,
        badge: None,
        filtered: false,
    },
];

/// The row's grid with the datagrid page's own props.
fn s5_grid() -> Grid<'static> {
    Grid::new(S5_GRID, &S5_COLUMNS)
        .nav(NavUnit::Cell)
        .column_gap(2)
        .gutter(GridGutter::Detailed {
            row_numbers: true,
            min_digits: 2,
        })
        .right_reserve(4)
        .column_fit(GridColumnFit::CompleteWithPreview { min_width: 6 })
        .sort_indicator(GridSortIndicator::ActiveOnly)
        .overflow_indicator(GridOverflowIndicator::Count)
        .fetch_label("Enter fetches more")
}

/// One stored cell: ghost cells (`NULL`, `DEFAULT`) read muted italic.
#[derive(Clone, Debug, PartialEq, Eq)]
struct S5Cell {
    text: String,
    ghost: bool,
}

impl S5Cell {
    fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ghost: false,
        }
    }

    fn ghost(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ghost: true,
        }
    }
}

/// One clean customer record; pending edits overlay it by id.
#[derive(Clone, Debug)]
struct S5Record {
    id: u64,
    cells: [S5Cell; 8],
}

fn s5_row(index: usize) -> S5Record {
    S5Record {
        id: 1001 + index as u64,
        cells: [
            S5Cell::plain((1001 + index).to_string()),
            S5Cell::plain(format!("Customer {index}")),
            S5Cell::plain("pro"),
            S5Cell::plain((10 + index).to_string()),
            S5Cell::plain("290.00"),
            S5Cell::plain("true"),
            S5Cell::ghost("NULL"),
            S5Cell::ghost("NULL"),
        ],
    }
}

#[derive(Clone, Debug)]
struct S5Snapshot {
    records: HashMap<u64, S5Record>,
    order: Vec<u64>,
    cells: HashMap<(u64, usize), S5Cell>,
    inserted: BTreeSet<u64>,
    deleted: BTreeSet<u64>,
    next_key: u64,
}

/// The row's customer model: the datagrid page's `CustomerModel` queue
/// semantics (keyed records, pending overlay, undoable snapshots) over a
/// fixed row set with a configurable total.
#[derive(Clone, Debug)]
struct S5GridModel {
    records: HashMap<u64, S5Record>,
    order: Vec<u64>,
    cells: HashMap<(u64, usize), S5Cell>,
    inserted: BTreeSet<u64>,
    deleted: BTreeSet<u64>,
    row_errors: HashMap<u64, String>,
    undo: Vec<S5Snapshot>,
    next_key: u64,
    total: RowTotal,
    more: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum S5RowState {
    Clean,
    Modified,
    Error,
}

impl S5GridModel {
    fn with_rows(n: usize, total: RowTotal, more: bool) -> Self {
        let mut model = Self {
            records: HashMap::new(),
            order: Vec::new(),
            cells: HashMap::new(),
            inserted: BTreeSet::new(),
            deleted: BTreeSet::new(),
            row_errors: HashMap::new(),
            undo: Vec::new(),
            next_key: 1001 + n as u64,
            total,
            more,
        };
        for index in 0..n {
            let record = s5_row(index);
            model.order.push(record.id);
            model.records.insert(record.id, record);
        }
        model
    }

    fn snapshot(&self) -> S5Snapshot {
        S5Snapshot {
            records: self.records.clone(),
            order: self.order.clone(),
            cells: self.cells.clone(),
            inserted: self.inserted.clone(),
            deleted: self.deleted.clone(),
            next_key: self.next_key,
        }
    }

    fn restore(&mut self, snap: S5Snapshot) {
        self.records = snap.records;
        self.order = snap.order;
        self.cells = snap.cells;
        self.inserted = snap.inserted;
        self.deleted = snap.deleted;
        self.next_key = snap.next_key;
    }

    fn id_at(&self, row: usize) -> Option<u64> {
        self.order.get(row).copied()
    }

    fn clean(&self, id: u64, col: usize) -> Option<&S5Cell> {
        self.records.get(&id)?.cells.get(col)
    }

    /// Set a pending value; reverting to the stored value clears the change.
    fn record_cell(&mut self, row: usize, col: usize, value: S5Cell) {
        let Some(id) = self.id_at(row) else {
            return;
        };
        let Some(stored) = self.clean(id, col).cloned() else {
            return;
        };
        let before = self.cells.get(&(id, col)).cloned();
        let after = if value == stored && !self.inserted.contains(&id) {
            None
        } else {
            Some(value)
        };
        if before == after {
            return;
        }
        let snap = self.snapshot();
        match after {
            Some(v) => {
                self.cells.insert((id, col), v);
            }
            None => {
                self.cells.remove(&(id, col));
            }
        }
        self.undo.push(snap);
    }

    fn pending_value(&self, row: usize, col: usize) -> Option<&S5Cell> {
        let id = self.id_at(row)?;
        self.cells.get(&(id, col))
    }

    fn value(&self, row: usize, col: usize) -> &S5Cell {
        let id = self.id_at(row).expect("row id");
        self.cells
            .get(&(id, col))
            .or_else(|| self.clean(id, col))
            .expect("cell")
    }

    fn toggle_delete(&mut self, row: usize) {
        let Some(id) = self.id_at(row) else {
            return;
        };
        let snap = self.snapshot();
        if self.inserted.remove(&id) {
            self.records.remove(&id);
            self.order.retain(|kept| *kept != id);
            self.cells.retain(|(kept, _), _| *kept != id);
            self.row_errors.remove(&id);
        } else if self.deleted.remove(&id) {
            // re-queued rows come back clean of cell edits, like the page.
        } else {
            self.deleted.insert(id);
            self.cells.retain(|(kept, _), _| *kept != id);
        }
        self.undo.push(snap);
    }

    fn insert_row(&mut self) -> u64 {
        let snap = self.snapshot();
        let id = self.next_key;
        self.next_key = self.next_key.saturating_add(1).max(id);
        let record = S5Record {
            id,
            cells: [
                S5Cell::ghost("DEFAULT"),
                S5Cell::ghost("NULL"),
                S5Cell::ghost("NULL"),
                S5Cell::ghost("NULL"),
                S5Cell::ghost("DEFAULT"),
                S5Cell::plain("true"),
                S5Cell::ghost("NULL"),
                S5Cell::ghost("NULL"),
            ],
        };
        self.order.push(id);
        self.records.insert(id, record);
        self.inserted.insert(id);
        self.undo.push(snap);
        id
    }

    fn discard(&mut self) {
        for id in std::mem::take(&mut self.inserted) {
            self.records.remove(&id);
            self.order.retain(|kept| kept != &id);
            self.cells.retain(|(kept, _), _| kept != &id);
        }
        self.cells.clear();
        self.deleted.clear();
        self.undo.clear();
        self.row_errors.clear();
    }

    fn undo(&mut self) -> bool {
        let Some(snap) = self.undo.pop() else {
            return false;
        };
        self.restore(snap);
        true
    }

    /// Fold the queue into the clean records (the clean-commit correlate).
    fn commit_ok(&mut self) {
        for ((id, col), value) in std::mem::take(&mut self.cells) {
            if let Some(record) = self.records.get_mut(&id)
                && let Some(cell) = record.cells.get_mut(col)
            {
                *cell = value;
            }
        }
        for id in std::mem::take(&mut self.deleted) {
            self.records.remove(&id);
            self.order.retain(|kept| kept != &id);
        }
        self.inserted.clear();
        self.undo.clear();
        self.row_errors.clear();
    }

    fn commit_err(&mut self, row: usize, message: String) {
        if let Some(id) = self.id_at(row) {
            self.row_errors.insert(id, message);
        }
    }

    /// (updated rows, inserted rows, deleted rows).
    fn counts(&self) -> (usize, usize, usize) {
        let updates = self
            .cells
            .keys()
            .map(|(id, _)| *id)
            .filter(|id| !self.inserted.contains(id))
            .collect::<BTreeSet<_>>()
            .len();
        (updates, self.inserted.len(), self.deleted.len())
    }

    fn pending_total(&self) -> usize {
        let (u, i, d) = self.counts();
        u + i + d
    }

    fn pending_label(&self) -> String {
        let (u, i, d) = self.counts();
        let mut parts = Vec::new();
        if u > 0 {
            parts.push(format!("{u} update{}", if u == 1 { "" } else { "s" }));
        }
        if i > 0 {
            parts.push(format!("{i} insert{}", if i == 1 { "" } else { "s" }));
        }
        if d > 0 {
            parts.push(format!("{d} delete{}", if d == 1 { "" } else { "s" }));
        }
        parts.join(" · ")
    }

    fn row_state(&self, row: usize) -> S5RowState {
        let Some(id) = self.id_at(row) else {
            return S5RowState::Clean;
        };
        if self.row_errors.contains_key(&id) {
            S5RowState::Error
        } else if self.deleted.contains(&id)
            || self.inserted.contains(&id)
            || self.cells.keys().any(|(kept, _)| *kept == id)
        {
            S5RowState::Modified
        } else {
            S5RowState::Clean
        }
    }

    fn len(&self) -> usize {
        self.order.len()
    }
}

impl GridModel for S5GridModel {
    fn row_count(&self) -> usize {
        self.order.len()
    }

    fn row_key(&self, row: usize) -> ItemKey {
        self.id_at(row).map_or(ItemKey::num(0), ItemKey::num)
    }

    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        let id = self.id_at(row)?;
        let clean = self.clean(id, col)?;
        let value = self.cells.get(&(id, col)).unwrap_or(clean);
        Some(CellRef::new(value.text.as_str()))
    }

    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> {
        let Some(id) = self.id_at(row) else {
            return CellDecor::default();
        };
        if self.deleted.contains(&id) {
            return CellDecor {
                tone: Some(Role::Fg(FgStep::Muted)),
                ..CellDecor::default()
            };
        }
        let dirty = self.cells.contains_key(&(id, col)) && !self.inserted.contains(&id);
        if dirty {
            return CellDecor {
                tone: Some(Role::Warning),
                dirty: true,
                ..CellDecor::default()
            };
        }
        CellDecor::default()
    }

    fn row_decor(&self, row: usize) -> RowDecor<'_> {
        let Some(id) = self.id_at(row) else {
            return RowDecor::default();
        };
        let mut decor = RowDecor {
            number: Some(row + 1),
            ..RowDecor::default()
        };
        if self.row_errors.contains_key(&id) {
            decor.marker = Some(GlyphRole::Error);
            decor.tone = Some(Role::Danger);
        } else if self.deleted.contains(&id) {
            decor.marker = Some(GlyphRole::Deleted);
            decor.tone = Some(Role::Fg(FgStep::Muted));
            decor.strike = true;
        } else if self.inserted.contains(&id) {
            decor.marker = Some(GlyphRole::Inserted);
            decor.tone = Some(Role::Fg(FgStep::Secondary));
        } else if self.cells.keys().any(|(kept, _)| *kept == id) {
            decor.marker = Some(GlyphRole::Dirty);
            decor.tone = Some(Role::Warning);
        }
        decor
    }

    fn total(&self) -> RowTotal {
        self.total
    }

    fn has_more(&self) -> bool {
        self.more
    }

    fn pending_count(&self) -> usize {
        self.pending_total()
    }

    fn pending_breakdown(&self) -> String {
        self.pending_label()
    }

    fn row_error(&self, row: usize) -> Option<&str> {
        self.id_at(row)
            .and_then(|id| self.row_errors.get(&id).map(String::as_str))
    }
}

fn s5_validate_cell(col: usize, text: &str) -> Result<S5Cell, FieldError> {
    let trimmed = text.trim();
    const PLANS: &[&str] = &["free", "pro", "team", "enterprise"];
    const NAMES: [&str; 8] = [
        "id",
        "customer",
        "plan",
        "seats",
        "mrr",
        "active",
        "renewed_at",
        "notes",
    ];
    if trimmed.is_empty() {
        return match col {
            S5_COL_CUSTOMER | S5_COL_PLAN | S5_COL_NOTES => Ok(S5Cell::plain("")),
            S5_COL_RENEWED => Ok(S5Cell::ghost("NULL")),
            _ => Err(FieldError::new("Empty: use Delete for NULL")),
        };
    }
    if trimmed.eq_ignore_ascii_case("null") {
        return match col {
            S5_COL_RENEWED | S5_COL_NOTES => Ok(S5Cell::ghost("NULL")),
            _ => Err(FieldError::new(format!("{} is NOT NULL", NAMES[col]))),
        };
    }
    match col {
        S5_COL_SEATS => {
            let numeric = trimmed.parse::<i64>().is_ok()
                || trimmed.parse::<f64>().is_ok_and(|n| n.is_finite());
            if numeric {
                Ok(S5Cell::plain(trimmed))
            } else {
                Err(FieldError::new("Must be a number"))
            }
        }
        S5_COL_ACTIVE => match trimmed.to_ascii_lowercase().as_str() {
            "true" | "t" | "1" | "yes" => Ok(S5Cell::plain("true")),
            "false" | "f" | "0" | "no" => Ok(S5Cell::plain("false")),
            _ => Err(FieldError::new("Must be true or false")),
        },
        S5_COL_NOTES => {
            let json = (trimmed.starts_with('{') && trimmed.ends_with('}'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']'));
            if json {
                Ok(S5Cell::plain(trimmed))
            } else {
                Err(FieldError::new("Must be a JSON object or array"))
            }
        }
        S5_COL_PLAN => {
            if PLANS.contains(&trimmed) {
                Ok(S5Cell::plain(trimmed))
            } else {
                Err(FieldError::new(format!(
                    "Must be one of: {}",
                    PLANS.join(", ")
                )))
            }
        }
        S5_COL_RENEWED => {
            let bytes = trimmed.as_bytes();
            if trimmed.len() >= 10 && bytes[4] == b'-' && bytes[7] == b'-' {
                Ok(S5Cell::plain(trimmed))
            } else {
                Err(FieldError::new("Use YYYY-MM-DD"))
            }
        }
        S5_COL_CUSTOMER => Ok(S5Cell::plain(trimmed)),
        _ => Err(FieldError::new("Cell is read-only")),
    }
}

impl GridEditor for S5GridModel {
    fn edit_intent(&self, row: usize, col: usize) -> EditIntent<'_> {
        let Some(id) = self.id_at(row) else {
            return EditIntent::Refuse {
                reason: "Unknown customer row",
            };
        };
        if self.deleted.contains(&id) {
            return EditIntent::Refuse {
                reason: "Row is queued for deletion",
            };
        }
        match col {
            S5_COL_ACTIVE => EditIntent::Cycle,
            S5_COL_NOTES => EditIntent::External,
            S5_COL_CUSTOMER | S5_COL_PLAN | S5_COL_SEATS | S5_COL_RENEWED => {
                let initial = self
                    .cells
                    .get(&(id, col))
                    .or_else(|| self.clean(id, col))
                    .map_or("", |cell| if cell.ghost { "" } else { cell.text.as_str() });
                EditIntent::Inline { initial }
            }
            _ => EditIntent::Refuse {
                reason: "Cell is read-only",
            },
        }
    }

    fn apply_cycle(&mut self, row: usize, col: usize) {
        if col != S5_COL_ACTIVE {
            return;
        }
        let Some(id) = self.id_at(row) else {
            return;
        };
        if self.deleted.contains(&id) {
            return;
        }
        let next = self
            .cells
            .get(&(id, col))
            .or_else(|| self.clean(id, col))
            .is_some_and(|cell| cell.text == "true");
        self.record_cell(row, col, S5Cell::plain(if next { "false" } else { "true" }));
    }

    fn commit_cell(&mut self, row: usize, col: usize, text: &str) -> Result<(), FieldError> {
        let Some(id) = self.id_at(row) else {
            return Err(FieldError::new("Unknown customer row"));
        };
        if self.deleted.contains(&id) {
            return Err(FieldError::new("Row is queued for deletion"));
        }
        let value = s5_validate_cell(col, text)?;
        self.record_cell(row, col, value);
        Ok(())
    }

    fn is_editable(&self, row: usize, col: usize) -> bool {
        matches!(
            col,
            S5_COL_CUSTOMER
                | S5_COL_PLAN
                | S5_COL_SEATS
                | S5_COL_ACTIVE
                | S5_COL_RENEWED
                | S5_COL_NOTES
        ) && self
            .id_at(row)
            .is_some_and(|id| !self.deleted.contains(&id))
    }
}

/// Editable grid rig: the row's model plus the action log.
struct S5GridApp {
    state: GridState,
    model: S5GridModel,
    actions: Vec<GridAction>,
    area: Rect,
    activity: Option<usize>,
}

impl S5GridApp {
    fn new(model: S5GridModel, cols: u16, rows: u16) -> Self {
        Self {
            state: GridState::default(),
            model,
            actions: Vec::new(),
            area: Rect::new(0, 0, cols, rows),
            activity: None,
        }
    }

    /// The cursor as `(row, col)` display indices.
    fn cursor(&self) -> Option<(usize, usize)> {
        let (key, col) = self.state.cursor()?;
        let row = self
            .model
            .order
            .iter()
            .position(|id| ItemKey::num(*id) == key)?;
        let col = usize::from(col.raw());
        (col < 8).then_some((row, col))
    }
}

impl App for S5GridApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = s5_grid().fetch_activity(self.activity).update_editable(
            cx,
            &mut self.state,
            &mut self.model,
        );
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        s5_grid()
            .fetch_activity(self.activity)
            .draw(ui, self.area, &self.state, &self.model);
    }
}

/// GRID-EDIT-001 (`showcase/flows/datagrid/selected`, 80x24): the
/// customers cell cursor plus the inline cell editor.
///
/// Isolated path drives a page-shaped [`Grid`]: the estimated-total
/// position label, the cursor-row gutter, edit entry with the cell
/// draft text, validator-gated commit folding into the pending queue,
/// inline Bool toggles, Tab commit-and-advance past read-only/primary
/// columns, and the read-only/deleted/invalid negatives. Live path boots
/// the datagrid page, moves the cursor, edits a customer cell, toggles a
/// Bool inline, and proves the id column and bad numbers never commit.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_grid_edit() {
    let dir = s5_dir("s5_grid_edit");

    // V1 headless: the cursor cell is highlighted and the meta reads the
    // estimated total.
    let model = S5GridModel::with_rows(40, RowTotal::Estimated(4_812), true);
    let state = GridState::default();
    assert_eq!(
        s5_grid().rows_label_for(&state, &model, Rect::new(0, 0, 80, 41)),
        "rows 1–40 of 40 loaded · ~4,812 total",
        "V1: estimated-total position label"
    );
    let mut app = Harness::new(
        S5GridApp::new(
            S5GridModel::with_rows(40, RowTotal::Estimated(4_812), true),
            80,
            22,
        ),
        Theme::junie(),
        80,
        22,
    );
    assert!(app.tab_to(S5_GRID), "V1: the grid takes focus");
    let text = app.text();
    let mut cursor_marked = false;
    let mut other_marked = false;
    for line in text.lines() {
        if line.contains("1001") {
            cursor_marked = line.starts_with('▎');
        } else if line.contains("1002") {
            other_marked = line.starts_with('▎');
        }
    }
    assert!(
        cursor_marked,
        "V1: the cursor cell row carries the ▎ bar\n{text}"
    );
    assert!(!other_marked, "V1: only the cursor row is marked\n{text}");

    // S1: Enter on a writable cell opens the editor with the cell draft.
    let mut app = Harness::new(
        S5GridApp::new(S5GridModel::with_rows(3, RowTotal::Exact(3), false), 80, 22),
        Theme::junie(),
        80,
        22,
    );
    assert!(app.tab_to(S5_GRID), "S1: the grid takes focus");
    let r = app.key(KeyCode::Right);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Right moves");
    assert_eq!(app.app().cursor(), Some((0, 1)), "S1: cursor reaches (0,1)");
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Enter opens");
    assert!(
        app.app().actions[before..].is_empty(),
        "S1: opening emits no action"
    );
    assert!(app.app().state.is_editing(), "S1: the editor opens");
    let (key, col) = app
        .app()
        .state
        .edit_cell()
        .expect("S1: the editor targets the cell");
    assert_eq!(
        (key, col),
        (ItemKey::num(1001), ColumnKey::num(1)),
        "S1: the editor targets the cell"
    );
    assert_eq!(
        app.app().state.edit_draft(),
        Some("Customer 0"),
        "S1: the cell draft text"
    );
    // S2/V2: typing commits through the validator and folds pending (the
    // `CellChanged` correlate: editor closed, pending holds the value).
    let _ = app.type_str(" Jr");
    assert_eq!(
        app.app().state.edit_draft(),
        Some("Customer 0 Jr"),
        "V2: while editing, the cell shows the draft text"
    );
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S2: commit applies");
    assert!(!app.app().state.is_editing(), "S2: commit closes");
    assert_eq!(
        app.app().model.pending_value(0, 1).map(|c| c.text.as_str()),
        Some("Customer 0 Jr"),
        "V2: commit repaints the typed value"
    );
    // A1: Bool cells toggle inline without opening the editor.
    for _ in 0..4 {
        let _ = app.key(KeyCode::Right);
    }
    assert_eq!(
        app.app().cursor(),
        Some((0, 5)),
        "A1: cursor reaches the active column"
    );
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Bool Enter toggles inline");
    assert!(
        !app.app().state.is_editing(),
        "A1: no editor opens for Bool"
    );
    assert_eq!(
        app.app().model.pending_value(0, 5).map(|c| c.text.as_str()),
        Some("false"),
        "A1: the toggle queues false"
    );
    // A2: Tab commits then opens the next writable cell, skipping
    // read_only and primary columns.
    let mut app = Harness::new(
        S5GridApp::new(S5GridModel::with_rows(3, RowTotal::Exact(3), false), 80, 22),
        Theme::junie(),
        80,
        22,
    );
    assert!(app.tab_to(S5_GRID), "A2: the grid takes focus");
    let _ = app.key(KeyCode::Right);
    let _ = app.key(KeyCode::Enter);
    assert!(
        app.app().state.is_editing(),
        "A2: editing the customer cell"
    );
    let r = app.key(KeyCode::Tab);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Tab commits");
    assert_eq!(
        app.app().cursor(),
        Some((0, 2)),
        "A2: Tab advances to the plan cell"
    );
    assert!(
        app.app().state.is_editing(),
        "A2: Tab opens the next editor"
    );
    // From seats (3), Tab skips the read_only mrr (4) into active (5),
    // whose inline toggle runs instead of an editor.
    let mut app = Harness::new(
        S5GridApp::new(S5GridModel::with_rows(3, RowTotal::Exact(3), false), 80, 22),
        Theme::junie(),
        80,
        22,
    );
    assert!(app.tab_to(S5_GRID), "A2: the grid takes focus");
    for _ in 0..3 {
        let _ = app.key(KeyCode::Right);
    }
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Tab);
    assert_eq!(
        app.app().cursor(),
        Some((0, 5)),
        "A2: Tab skips the read_only mrr column"
    );
    assert_eq!(
        app.app().model.pending_value(0, 5).map(|c| c.text.as_str()),
        Some("false"),
        "A2: the skipped-into Bool toggles inline"
    );
    // N1: Enter on a read_only column or a queued-for-deletion row must
    // NOT open the editor.
    let mut app = Harness::new(
        S5GridApp::new(S5GridModel::with_rows(3, RowTotal::Exact(3), false), 80, 22),
        Theme::junie(),
        80,
        22,
    );
    assert!(app.tab_to(S5_GRID), "N1: the grid takes focus");
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1: Enter on id is refused");
    assert_eq!(
        app.app().actions.as_slice(),
        &[GridAction::Activated(ItemKey::num(1001))],
        "N1: refusal still activates the row"
    );
    assert!(
        !app.app().state.is_editing(),
        "N1: Enter on the primary id column opens nothing"
    );
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.toggle_delete(0);
    let mut app = Harness::new(S5GridApp::new(model, 80, 22), Theme::junie(), 80, 22);
    assert!(app.tab_to(S5_GRID), "N1: the grid takes focus");
    let _ = app.key(KeyCode::Right);
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1: Enter on deleted is refused");
    assert_eq!(
        &app.app().actions[before..],
        &[GridAction::Activated(ItemKey::num(1001))],
        "N1: refusal still activates the row"
    );
    assert!(
        !app.app().state.is_editing(),
        "N1: Enter on a queued-for-deletion row opens nothing"
    );
    // N2: a failed validation must NOT fold pending: the editor stays
    // open with the error.
    let mut app = Harness::new(
        S5GridApp::new(S5GridModel::with_rows(3, RowTotal::Exact(3), false), 80, 22),
        Theme::junie(),
        80,
        22,
    );
    assert!(app.tab_to(S5_GRID), "N2: the grid takes focus");
    for _ in 0..3 {
        let _ = app.key(KeyCode::Right);
    }
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('l');
    let _ = app.type_str("abc");
    let _ = app.key(KeyCode::Enter);
    assert!(app.app().state.is_editing(), "N2: the editor stays open");
    assert_eq!(
        app.app().state.edit_error().map(|e| e.message.as_ref()),
        Some("Must be a number"),
        "N2: the error explains"
    );
    assert_eq!(app.app().model.pending_total(), 0, "N2: nothing queues");

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
/// Isolated path drives a page-shaped [`Grid`]: `record_cell` queuing
/// with revert-to-stored clearing, inserted-row deletion removing the
/// row entirely, undo/discard queue ops, the three bar buttons emitting
/// `PreviewRequested`/`DiscardRequested`/`CommitRequested`, and discard
/// clearing only the queue. (`p`/`u`/`U` are page-keymap bindings over
/// these component facts; the live path proves the keys.) Live path
/// replays the matrix `GRID_EDITS` (seats 600/12), inserts and deletes a
/// row, previews the SQL dialog, undoes one change, and discards the rest.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_grid_pending() {
    let dir = s5_dir("s5_grid_pending");

    // S1: record_cell queues values; reverting to the stored value clears.
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.record_cell(0, 3, S5Cell::plain("600"));
    assert_eq!(model.counts(), (1, 0, 0), "S1: one change queues");
    assert_eq!(
        model.row_state(0),
        S5RowState::Modified,
        "S1: the row reads Modified"
    );
    model.record_cell(0, 3, S5Cell::plain("10"));
    assert_eq!(model.pending_total(), 0, "S1: reverting clears the change");
    assert_eq!(
        model.row_state(0),
        S5RowState::Clean,
        "S1: the row reads Clean again"
    );
    // S2: deleting an inserted row removes it entirely instead of queueing.
    let len = model.len();
    let id = model.insert_row();
    assert_eq!(model.len(), len + 1, "S2: the row exists");
    assert!(model.inserted.contains(&id), "S2: the insert queues");
    model.toggle_delete(len);
    assert_eq!(model.len(), len, "S2: the inserted row is gone entirely");
    assert_eq!(model.pending_total(), 0, "S2: no delete queues");
    // V1 headless: the pending bar shows its three buttons as separate stops.
    model.record_cell(0, 3, S5Cell::plain("600"));
    let ids = [
        s5_grid().preview_id(),
        s5_grid().discard_id(),
        s5_grid().save_id(),
    ];
    assert!(
        ids[0] != ids[1] && ids[1] != ids[2] && ids[0] != ids[2],
        "V1: three separate bar stops"
    );
    let mut app = Harness::new(S5GridApp::new(model, 80, 22), Theme::junie(), 80, 22);
    assert!(app.tab_to(S5_GRID), "V1: the grid takes focus");
    let bar = app.text();
    assert!(
        bar.contains("Preview SQL") && bar.contains("Discard") && bar.contains("Save"),
        "V1: Preview SQL, Discard and Save render\n{bar}"
    );
    assert!(bar.contains("1 pending"), "V1: the bar counts the queue");
    // A1: Preview SQL emits PreviewRequested; u undoes one queued change;
    // Discard emits DiscardRequested (the `p`/`u`/`U` component facts; the
    // page keymap binds the keys, proven live).
    let before = app.app().actions.len();
    let _ = app.click_id(ids[0]);
    assert_eq!(
        &app.app().actions[before..],
        &[GridAction::PreviewRequested],
        "A1: Preview SQL emits PreviewRequested"
    );
    // Two queued changes, then `u` undoes one: drive through the model
    // queue the page's `u` binding owns (unit correlate).
    let mut queued = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    queued.record_cell(0, 3, S5Cell::plain("600"));
    queued.record_cell(1, 3, S5Cell::plain("12"));
    assert_eq!(queued.pending_total(), 2, "A1: two changes queue");
    assert!(queued.undo(), "A1: u runs");
    assert_eq!(queued.pending_total(), 1, "A1: u undoes one queued change");
    let mut app = Harness::new(S5GridApp::new(queued, 80, 22), Theme::junie(), 80, 22);
    assert!(app.tab_to(S5_GRID), "A1: the grid takes focus");
    let before = app.app().actions.len();
    let _ = app.click_id(ids[1]);
    assert_eq!(
        &app.app().actions[before..],
        &[GridAction::DiscardRequested],
        "A1: Discard emits DiscardRequested"
    );
    // A2: the bar buttons emit PreviewRequested/DiscardRequested/CommitRequested.
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.record_cell(0, 3, S5Cell::plain("600"));
    let mut app = Harness::new(S5GridApp::new(model, 80, 22), Theme::junie(), 80, 22);
    assert!(app.tab_to(S5_GRID), "A2: the grid takes focus");
    for (id, want, what) in [
        (ids[0], GridAction::PreviewRequested, "Preview SQL"),
        (ids[1], GridAction::DiscardRequested, "Discard"),
        (ids[2], GridAction::CommitRequested, "Save"),
    ] {
        let before = app.app().actions.len();
        let _ = app.click_id(id);
        assert_eq!(&app.app().actions[before..], &[want], "A2: {what} emits");
    }
    // N2: discard clears only the queue — committed rows are untouched.
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.record_cell(0, 3, S5Cell::plain("600"));
    model.insert_row();
    model.toggle_delete(1);
    assert_eq!(model.counts(), (1, 1, 1), "N2: one of each queues");
    model.discard();
    assert_eq!(model.pending_total(), 0, "N2: the queue clears");
    assert_eq!(model.len(), 3, "N2: committed rows are untouched");
    assert_eq!(
        model.value(0, 3),
        &S5Cell::plain("10"),
        "N2: stored values are untouched"
    );

    // PTY: the matrix GRID_EDITS plus insert, delete, preview, undo, discard.
    let case = s5_case("grid_pending", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
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
        case.timeout_ms,
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
/// Isolated path drives a page-shaped [`Grid`]: `CommitRequested` from
/// the Save button (the `Ctrl+S` component fact; the page binds the key,
/// proven live), the `Loading rows` empty-grid form, the commit
/// rejection marking exactly one row with its error, the pending queue
/// surviving failure, and the clean-commit fold. Live path replays the
/// matrix failed-save flow (seats 600/12), rides the 4-tick saving
/// window, proves the `!` mark plus the error text, and proves a clean
/// queue reports `Nothing to save`.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_grid_commit() {
    let dir = s5_dir("s5_grid_commit");

    // A1 headless: the Save button emits CommitRequested (the Ctrl+S
    // component fact; the page binds the key, proven live).
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.record_cell(0, 3, S5Cell::plain("600"));
    let mut app = Harness::new(S5GridApp::new(model, 80, 22), Theme::junie(), 80, 22);
    assert!(app.tab_to(S5_GRID), "A1: the grid takes focus");
    let before = app.app().actions.len();
    let _ = app.click_id(s5_grid().save_id());
    assert_eq!(
        &app.app().actions[before..],
        &[GridAction::CommitRequested],
        "A1: Save emits CommitRequested"
    );
    // V1 headless, empty grid: the loading form with its label.
    let model = S5GridModel::with_rows(0, RowTotal::Exact(0), false);
    let state = GridState::default();
    let buf = render_isolated(80, 12, 0, None, |ui, _| {
        s5_grid()
            .empty(EmptyState::Loading {
                label: "Loading rows",
            })
            .draw(ui, Rect::new(0, 1, 80, 10), &state, &model);
    });
    let mut loading = String::new();
    for y in 1..11u16 {
        loading.push_str(&buf_row_text(&buf, y));
        loading.push('\n');
    }
    assert!(
        loading.contains("Loading rows"),
        "V1: the empty grid shows Loading rows\n{loading}"
    );
    assert_surround_intact(&buf, Rect::new(0, 1, 80, 10), "V1: containment");
    capture_isolated(&dir, "grid-loading", &buf);
    // S1/S2 headless: the round-trip marks exactly the rejected row.
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.record_cell(0, 3, S5Cell::plain("600"));
    assert_eq!(
        model.row_state(0),
        S5RowState::Modified,
        "S1: the edit queues first"
    );
    model.commit_err(0, "seats above the plan limit (500)".into());
    assert_eq!(
        model.row_state(0),
        S5RowState::Error,
        "S2: the rejected row marks Error"
    );
    assert_eq!(
        model.row_errors.get(&1001).map(String::as_str),
        Some("seats above the plan limit (500)"),
        "S2: the row owns its error"
    );
    // N1: valid queued edits are NOT marked when only one row is rejected.
    model.record_cell(1, 3, S5Cell::plain("12"));
    assert_eq!(
        model.row_state(1),
        S5RowState::Modified,
        "N1: the valid row stays Modified"
    );
    // N2: the pending queue is NOT cleared on failure.
    assert_eq!(model.counts(), (2, 0, 0), "N2: the queue survives failure");
    // A clean commit folds the queue instead.
    let mut model = S5GridModel::with_rows(3, RowTotal::Exact(3), false);
    model.record_cell(0, 3, S5Cell::plain("12"));
    model.commit_ok();
    assert_eq!(model.pending_total(), 0, "clean commit: the queue folds");
    assert_eq!(
        model.value(0, 3),
        &S5Cell::plain("12"),
        "clean commit: stored"
    );
    // V1 headless, loaded grid: the fetch row spins while saving. (After
    // the queue checks: the `fetching…` wording has no impl correlate —
    // the fetch row spins the `Enter fetches more` label instead.)
    let mut rig = S5GridApp::new(
        S5GridModel::with_rows(40, RowTotal::Estimated(4_812), true),
        80,
        44,
    );
    rig.activity = Some(0);
    let mut app = Harness::new(rig, Theme::junie(), 80, 44);
    assert!(app.tab_to(S5_GRID), "V1: the grid takes focus");
    let saving = app.text();
    assert!(
        saving.contains("Enter fetches more"),
        "V1: the fetch row renders in view (presence)\n{saving}"
    );
    assert!(
        saving.contains("fetching…"),
        "V1: the loaded grid spins fetching… while saving\n{saving}"
    );

    // PTY: the matrix failed-save flow.
    let case = s5_case("grid_commit", &["--page", "datagrid"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::drive_with_timeout(
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
        case.timeout_ms,
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
    support::drive_with_timeout(
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
        case_timeout(&casew).as_millis() as u64,
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

// ------------------------------------------------------------ editor rig --

const S5_CODE: Id = Id::root("s5.code");

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
fn s5_highlight(src: &str) -> Vec<(std::ops::Range<usize>, SyntaxRole)> {
    let mut out = Vec::new();
    for word in ["fn ", "let "] {
        let mut from = 0;
        while let Some(p) = src[from..].find(word) {
            let kw = &word[..word.len() - 1];
            out.push((from + p..from + p + kw.len(), SyntaxRole::Keyword));
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
        out.push((from + p..end, SyntaxRole::Str));
        from = end;
    }
    out
}

const S5_SAMPLE: &str = "fn fetch(url: &str) -> Body {\n    let body = get(url);\n    log(\"done\");\n    body\n}\n\nfn client() -> Client {\n    Client::new().unwrap()\n}\n";

/// The row's editor with the codeeditor page's own props.
fn s5_code() -> CodeEditor<'static> {
    CodeEditor::new(S5_CODE, 12)
        .highlighter(&s5_highlight)
        .segmenter(&s5_blocks)
        .tab_behavior(TabBehavior::Leave)
}

/// Editor rig: durable state plus the action log.
struct S5CodeApp {
    state: CodeEditorState,
    actions: Vec<CodeAction>,
    area: Rect,
}

impl S5CodeApp {
    fn new(text: &str, area: Rect) -> Self {
        Self {
            state: CodeEditorState::new(text),
            actions: Vec::new(),
            area,
        }
    }
}

impl App for S5CodeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = s5_code().update(cx, &mut self.state);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        s5_code().draw(ui, self.area, &self.state);
    }
}

/// CODE-EDIT-001 (`showcase/pages/codeeditor`, 80x24): gutter, navigation
/// and editing modes, and the footer.
///
/// Isolated path drives a [`CodeEditor`] over a two-block sample: gutter
/// geometry (bar + block marker + right-aligned numbers), syntax tones
/// through the production render, edit entry/commit, block jumps, find
/// with match stepping, h/l pan and paging, and the
/// navigation-typing/Esc negatives. (The `ln L/N · col C` footer is
/// page-owned in the impl; the live path proves it.) Live path boots the
/// editor page on retry.rs and walks the same flow.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_code_edit() {
    let dir = s5_dir("s5_code_edit");
    let t = Theme::junie();

    // V1 headless: the gutter shows the cursor bar, block marker › and
    // right-aligned numbers.
    let mut app = Harness::new(
        S5CodeApp::new(S5_SAMPLE, Rect::new(0, 1, 60, 12)),
        Theme::junie(),
        60,
        14,
    );
    assert!(app.tab_to(S5_CODE), "V1: the editor takes focus");
    let buf = app.buffer().clone();
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
    // V2 headless: keywords and strings carry syntax tones through the
    // production render.
    assert!(buf_is_bold(&buf, 6, 1), "V2: the `fn` keyword reads bold");
    assert_eq!(
        buf[(6, 1)].fg,
        t.color.fg[FgStep::Primary.index()],
        "V2: the keyword takes primary"
    );
    let third = buf_row_text(&buf, 3);
    let str_x = char_col(&third, "\"done\"");
    assert_eq!(
        buf[(str_x, 3)].fg,
        t.color.fg[FgStep::Secondary.index()],
        "V2: strings take secondary"
    );

    // S1: i/Enter/a enters editing; Esc commits back to navigation.
    let mut app = Harness::new(
        S5CodeApp::new(S5_SAMPLE, Rect::new(0, 1, 60, 12)),
        Theme::junie(),
        60,
        14,
    );
    assert!(app.tab_to(S5_CODE), "S1: the editor takes focus");
    let r = app.key(KeyCode::Char('i'));
    assert_eq!(r.flow(), Flow::Consumed, "S1: i edits");
    assert!(app.app().state.is_editing(), "S1: editing");
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Esc commits");
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::Committed],
        "S1: Esc reports Committed"
    );
    assert!(!app.app().state.is_editing(), "S1: back to navigation");
    assert_eq!(
        app.app().state.text(),
        S5_SAMPLE,
        "S1: the document is kept"
    );
    // S2: { and } jump to the previous/next block start.
    let second = s5_code().blocks(&app.app().state)[1].start;
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('}'));
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::CursorMoved],
        "S2: }} jumps to the next block ({r:?})"
    );
    assert_eq!(app.app().state.cursor_offset(), second, "S2: next block");
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('{'));
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::CursorMoved],
        "S2: {{ jumps back ({r:?})"
    );
    assert_eq!(app.app().state.cursor_offset(), 0, "S2: back to start");
    // A1: / opens the find bar; n/N step through matches with CursorMoved.
    let r = app.key(KeyCode::Char('/'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: / finds");
    assert!(
        app.text().contains("find "),
        "A1: the find bar opens\n{}",
        app.text()
    );
    let _ = app.type_str("client");
    assert!(
        app.text().contains("find client"),
        "A1: the needle types\n{}",
        app.text()
    );
    let at = app.app().state.cursor_offset();
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Enter);
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::CursorMoved],
        "A1: Enter lands ({r:?} at {at})"
    );
    assert_eq!(
        app.app().state.cursor_offset(),
        S5_SAMPLE.find("client").unwrap(),
        "A1: Enter lands on the match"
    );
    let before = app.app().actions.len();
    let _ = app.key(KeyCode::Char('n'));
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::CursorMoved],
        "A1: n steps with CursorMoved"
    );
    // A2: h/l pan 8 cells; PageUp/Down move by the viewport height.
    let mut app = Harness::new(
        S5CodeApp::new(S5_SAMPLE, Rect::new(0, 1, 60, 12)),
        Theme::junie(),
        60,
        14,
    );
    assert!(app.tab_to(S5_CODE), "A2: the editor takes focus");
    let _ = app.key(KeyCode::Char('l'));
    let panned = app.row(1);
    assert!(panned.contains("(url"), "A2: l pans 8 cells\n{panned:?}");
    let _ = app.key(KeyCode::Char('h'));
    let back = app.row(1);
    assert!(back.contains("fn fetch"), "A2: h pans back\n{back:?}");
    let top = app.app().state.cursor_offset();
    let before = app.app().actions.len();
    let _ = app.key(KeyCode::PageDown);
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::CursorMoved],
        "A2: PageDown moves"
    );
    assert!(
        app.app().state.cursor_offset() > top,
        "A2: PageDown moves by the viewport"
    );
    // N1: typing in navigation must NOT mutate the document.
    let before_text = app.app().state.text().to_owned();
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(
        r.flow(),
        Flow::Ignored,
        "N1: typing in navigation is Ignored"
    );
    assert!(
        app.app().actions[before..].is_empty(),
        "N1: typing in navigation reports nothing"
    );
    assert_eq!(app.app().state.text(), before_text, "N1: untouched");
    // N2: Esc with no find bar open must NOT consume.
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Ignored, "N2: a bare Esc returns Ignored");
    assert!(
        app.app().actions[before..].is_empty(),
        "N2: a bare Esc reports nothing"
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
    let t = Theme::junie();

    // S1/V1 headless: running arms the block and spins the marker column.
    let mut state = CodeEditorState::new(S5_SAMPLE);
    let block = s5_code().blocks(&state)[1].clone();
    state.jump_to(block.start);
    assert_eq!(
        s5_code().current_block(&state),
        Some(block.clone()),
        "S1: the cursor owns block two"
    );
    state.set_running(Some(block.clone()));
    let buf = render_isolated(60, 14, 3, None, |ui, _| {
        s5_code().draw(ui, Rect::new(0, 1, 60, 12), &state);
    });
    let area = Rect::new(0, 1, 60, 12);
    let row7 = buf_row_text(&buf, 7);
    let marker = buf[(1, 7)].symbol().to_owned();
    assert!(
        "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏".contains(&marker),
        "V1: the marker column spins\n{row7:?} ({marker:?})"
    );
    assert_eq!(
        buf[(1, 7)].fg,
        t.color.accent,
        "V1: the spinner takes accent"
    );
    assert_surround_intact(&buf, area, "V1: containment");
    capture_isolated(&dir, "code-running", &buf);
    // A1 headless: between blocks the cursor keeps the previous block
    // (the `or_else` fallback); `None` happens only before the first
    // block start, which needs leading blank lines.
    let first = s5_code().blocks(&state)[0].clone();
    state.jump_to(S5_SAMPLE.find("\n\n").unwrap() + 1);
    assert_eq!(
        s5_code().current_block(&state),
        Some(first),
        "A1: between blocks the previous block runs"
    );
    let mut lead = CodeEditorState::new("\n\nfn main() {}");
    lead.jump_to(0);
    assert_eq!(
        s5_code().current_block(&lead),
        None,
        "A1: None before the first block"
    );
    // V2/S2 headless: the unwrap line owns the ! marker; the footer names it.
    let p = S5_SAMPLE.find(".unwrap()").unwrap();
    state.set_diagnostics(vec![CodeDiagnostic::new(
        p + 1..p + 9,
        CodeSeverity::Warning,
        "unwrap() panics on Err; propagate with ? instead",
    )]);
    state.set_running(None);
    let buf = render_isolated(60, 14, 0, None, |ui, _| {
        s5_code().draw(ui, Rect::new(0, 1, 60, 12), &state);
    });
    assert_eq!(buf[(1, 8)].symbol(), "!", "V2: the ! marker owns the slot");
    assert!(buf_is_bold(&buf, 1, 8), "V2: the diagnostic marker is bold");
    assert_eq!(
        buf[(1, 8)].fg,
        t.color.warning,
        "V2: a Warning takes warning"
    );
    let footer = buf_row_text(&buf, 12);
    assert!(
        footer.contains("unwrap() panics"),
        "V2: the footer names the diagnostic\n{footer:?}"
    );
    // A2/N2 headless: any edit reports Changed (the page clears on it).
    let mut app = Harness::new(
        S5CodeApp::new(S5_SAMPLE, Rect::new(0, 1, 60, 12)),
        Theme::junie(),
        60,
        14,
    );
    assert!(app.tab_to(S5_CODE), "A2: the editor takes focus");
    let _ = app.key(KeyCode::Char('i'));
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Consumed, "A2: an edit applies");
    assert_eq!(
        &app.app().actions[before..],
        &[CodeAction::Changed],
        "A2: an edit reports Changed"
    );
    // N1 headless: arming a run never touches the document text.
    let mut state = CodeEditorState::new(S5_SAMPLE);
    let block = s5_code().blocks(&state)[1].clone();
    state.set_running(Some(block));
    assert_eq!(state.text(), S5_SAMPLE, "N1: the run mutates no text");

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

// -------------------------------------------------------------- diff rig --

const S5_DIFF: Id = Id::root("s5.diff");
const S5_DIFF_EMPTY: Id = Id::root("s5.diff.empty");

/// One owned diff row: hunk starts plus kinded text lines.
#[derive(Clone, Debug)]
enum S5DiffRow {
    Hunk { old_start: usize, new_start: usize },
    Line { kind: DiffLineKind, text: String },
}

/// Page-shaped diff file (Modified config/service.toml, 5 hunks, +10 −5).
fn s5_diff_sample() -> Vec<S5DiffRow> {
    let mut rows = Vec::new();
    for i in 0..5 {
        rows.push(S5DiffRow::Hunk {
            old_start: i * 8 + 1,
            new_start: i * 9 + 1,
        });
        rows.push(S5DiffRow::Line {
            kind: DiffLineKind::Context,
            text: format!("[service.worker_{i}]"),
        });
        rows.push(S5DiffRow::Line {
            kind: DiffLineKind::Remove,
            text: "attempts = 3".into(),
        });
        rows.push(S5DiffRow::Line {
            kind: DiffLineKind::Add,
            text: "attempts = 5".into(),
        });
        rows.push(S5DiffRow::Line {
            kind: DiffLineKind::Add,
            text: "backoff = \"exponential\"".into(),
        });
        rows.push(S5DiffRow::Line {
            kind: DiffLineKind::Context,
            text: "region = \"tokyo\"".into(),
        });
        rows.push(S5DiffRow::Line {
            kind: DiffLineKind::Context,
            text: "enabled = true".into(),
        });
    }
    rows
}

/// Borrowed diff source over owned rows.
#[derive(Clone, Debug)]
struct S5DiffSource {
    rows: Vec<S5DiffRow>,
}

impl S5DiffSource {
    fn sample() -> Self {
        Self {
            rows: s5_diff_sample(),
        }
    }
}

impl DiffSource for S5DiffSource {
    fn revision(&self) -> u64 {
        1
    }

    fn path(&self) -> &str {
        "config/service.toml"
    }

    fn status_marker(&self) -> &str {
        "M"
    }

    fn status_label(&self) -> &str {
        "modified"
    }

    fn status_role(&self) -> Role {
        Role::Warning
    }

    fn row_count(&self) -> usize {
        self.rows.len()
    }

    fn row(&self, index: usize) -> Option<DiffRow<'_>> {
        match self.rows.get(index)? {
            S5DiffRow::Hunk {
                old_start,
                new_start,
            } => Some(DiffRow::Hunk {
                old_start: *old_start,
                new_start: *new_start,
            }),
            S5DiffRow::Line { kind, text } => Some(DiffRow::Line { kind: *kind, text }),
        }
    }
}

/// Diff rig: the view plus the page's Empty toggle (the `set_file(None)`
/// lever) and the action log.
struct S5DiffApp {
    state: DiffViewState,
    source: Option<S5DiffSource>,
    actions: Vec<ViewportAction>,
    area: Rect,
    empty_area: Rect,
}

impl S5DiffApp {
    fn new(source: Option<S5DiffSource>, area: Rect) -> Self {
        let empty_area = Rect::new(area.x, area.bottom().saturating_sub(1), 10, 1);
        Self {
            state: DiffViewState::default(),
            source,
            actions: Vec::new(),
            area,
            empty_area,
        }
    }

    fn view(&self) -> DiffView<'_> {
        DiffView::new(S5_DIFF, self.source.as_ref().map(|s| s as &dyn DiffSource))
    }
}

impl App for S5DiffApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let view = DiffView::new(S5_DIFF, self.source.as_ref().map(|s| s as &dyn DiffSource));
        let mut response = view.update(cx, &mut self.state);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        let mut out = response.erase();
        let mut empty = Button::new(S5_DIFF_EMPTY, "Empty").update(cx);
        if empty.take_action().is_some() {
            self.source = None;
        }
        out |= empty.erase();
        out
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.view().draw(ui, self.area, &self.state);
        Button::new(S5_DIFF_EMPTY, "Empty").draw(ui, self.empty_area);
    }
}

/// DIFF-UNIFIED-001 (`showcase/flows/diff/empty`, 80x24): the unified
/// listing plus the empty file state.
///
/// Isolated path drives a [`DiffView`] over a page-shaped file: the `M
/// path +N −M · K hunks` header with muted `@@` lines, old/new gutters
/// with bold toned `+`/`-` markers, the Empty toggle rendering `No file
/// selected` with the scroll reset, the no-hunks `(no textual changes)`
/// form, viewport delegation, and the no-`Old`/`New` plus
/// no-kept-offset negatives. Live path boots the diff page, scrolls the
/// listing, and toggles Empty on.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_diff_unified() {
    let dir = s5_dir("s5_diff_unified");
    let t = Theme::junie();

    // V1 headless: the header plus muted @@ hunk lines.
    let mut app = Harness::new(
        S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 78, 22)),
        Theme::junie(),
        80,
        24,
    );
    assert!(app.tab_to(S5_DIFF), "V1: the view takes focus");
    let buf = app.buffer().clone();
    let header = buf_row_text(&buf, 1);
    assert!(
        header.starts_with("M config/service.toml  +10 −5 · 5 hunks"),
        "V1: header with counts\n{header:?}"
    );
    assert!(buf_is_bold(&buf, 0, 1), "V1: the status marker is bold");
    assert_eq!(
        buf[(0, 1)].fg,
        t.color.warning,
        "V1: Modified takes warning"
    );
    let hunk = buf_row_text(&buf, 2);
    assert!(
        hunk.starts_with("@@ -1,4 +1,5 @@"),
        "V1: hunks open with @@ lines\n{hunk:?}"
    );
    assert_eq!(
        buf[(0, 2)].fg,
        t.color.fg[FgStep::Muted.index()],
        "V1: @@ lines are muted"
    );
    // V2 headless: old/new numbers, a bold marker, and the toned text.
    let rem_y = (1..23u16)
        .find(|y| buf_row_text(&buf, *y).contains("- attempts = 3"))
        .expect("V2: the removed row lists");
    let rem = buf_row_text(&buf, rem_y);
    let mark_x = char_col(&rem, "- attempts");
    assert_eq!(buf[(mark_x, rem_y)].symbol(), "-", "V2: the - marker");
    assert!(buf_is_bold(&buf, mark_x, rem_y), "V2: the marker is bold");
    assert_eq!(
        buf[(mark_x, rem_y)].fg,
        t.color.danger,
        "V2: removes take error"
    );
    assert_eq!(
        buf[(mark_x + 2, rem_y)].fg,
        t.color.danger,
        "V2: removed text takes error"
    );
    assert_eq!(&rem[..mark_x as usize], "  2     ", "V2: old/new numbers");
    let add_y = (1..23u16)
        .find(|y| buf_row_text(&buf, *y).contains("+ attempts = 5"))
        .expect("V2: the added row lists");
    let add = buf_row_text(&buf, add_y);
    let add_x = char_col(&add, "+ attempts");
    assert!(
        buf_is_bold(&buf, add_x, add_y) && buf[(add_x, add_y)].fg == t.color.success,
        "V2: + bold success"
    );
    // S1: the Empty toggle renders No file selected and resets the scroll.
    for _ in 0..5 {
        let _ = app.key(KeyCode::Down);
    }
    assert_eq!(
        app.app().state.viewport().scroll().offset(),
        5,
        "S1: scrolled first"
    );
    let _ = app.click_id(S5_DIFF_EMPTY);
    let _ = app.tick();
    assert_eq!(
        app.app().state.viewport().scroll().offset(),
        0,
        "S1/N2: the offset resets"
    );
    assert!(
        app.text().contains("No file selected"),
        "S1: the empty line renders\n{}",
        app.text()
    );
    // S2: a file with no hunks renders (no textual changes).
    let mut app = Harness::new(
        S5DiffApp::new(
            Some(S5DiffSource { rows: Vec::new() }),
            Rect::new(0, 1, 78, 22),
        ),
        Theme::junie(),
        80,
        24,
    );
    assert!(app.tab_to(S5_DIFF), "S2: the view takes focus");
    assert!(
        app.text().contains("(no textual changes)"),
        "S2: the no-hunks form\n{}",
        app.text()
    );
    // A2: keys, wheel and drags delegate to the wrapped TextViewport.
    let mut app = Harness::new(
        S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 78, 22)),
        Theme::junie(),
        80,
        24,
    );
    assert!(app.tab_to(S5_DIFF), "A2: the view takes focus");
    let r = app.key(KeyCode::Down);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A2: Down delegates to the viewport"
    );
    assert_eq!(
        app.app().state.viewport().scroll().offset(),
        1,
        "A2: Down scrolls one row"
    );
    // N1: the unified listing must NOT show the Old/New column headers.
    // (Offset 1 after A2: the header row scrolled out; @@ + rows prove
    // the listing still renders.)
    let all = app.text();
    assert!(
        all.contains("@@") && all.contains("attempts = 3"),
        "N1: the listing renders (presence)"
    );
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
/// Isolated path drives a [`DiffView`] over a page-shaped file: the
/// paired columns with faint numbers and bold changed runs, the
/// narrow-width fallback with the mode retained, `y` emitting `Copy`,
/// Esc clearing, wheel scrolling, unpaired blank sides, and tab
/// expansion before measuring. (Check order runs the supported forms
/// before the two known gaps — the narrow fallback and the `Old`/`New`
/// headers — so the gaps fail last, not first.) Live path toggles
/// Review on, drags across the columns, copies with `y`, clears with
/// Esc, and toggles the mode off and on.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_diff_review() {
    let dir = s5_dir("s5_diff_review");

    // V1 headless: paired columns with faint numbers, bold changed runs.
    let mut rig = S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 78, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "V1: the view takes focus");
    let buf = app.buffer().clone();
    let paired_y = (1..23u16)
        .find(|y| {
            let t = buf_row_text(&buf, *y);
            t.contains("attempts = 3") && t.contains("attempts = 5")
        })
        .expect("V1: a paired change row");
    let paired = buf_row_text(&buf, paired_y);
    let bolds: String = (0..78u16)
        .filter(|x| buf_is_bold(&buf, *x, paired_y))
        .map(|x| buf[(x, paired_y)].symbol().to_owned())
        .collect();
    assert!(
        bolds.contains('3') && bolds.contains('5'),
        "V1: the differing run reads bold ({bolds:?})\n{paired:?}"
    );
    assert!(paired.contains('│'), "V1: the │ separator\n{paired:?}");
    // N1: unpaired sides must NOT pair text: the empty side stays blank.
    let lopsided = S5DiffSource {
        rows: vec![
            S5DiffRow::Hunk {
                old_start: 1,
                new_start: 1,
            },
            S5DiffRow::Line {
                kind: DiffLineKind::Remove,
                text: "gone one".into(),
            },
            S5DiffRow::Line {
                kind: DiffLineKind::Remove,
                text: "gone two".into(),
            },
            S5DiffRow::Line {
                kind: DiffLineKind::Add,
                text: "fresh".into(),
            },
        ],
    };
    let mut rig = S5DiffApp::new(Some(lopsided), Rect::new(0, 1, 78, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "N1: the view takes focus");
    let lone = app
        .text()
        .lines()
        .map(str::to_owned)
        .find(|l| l.contains("gone two"))
        .expect("N1: the unpaired row");
    let halves: Vec<&str> = lone.split('│').collect();
    assert_eq!(halves.len(), 2, "N1: old │ new shape");
    assert!(
        halves[1].trim().is_empty(),
        "N1: the empty side stays blank\n{lone}"
    );
    // N2: tabs expand to four spaces before measuring.
    let tabbed = S5DiffSource {
        rows: vec![
            S5DiffRow::Hunk {
                old_start: 1,
                new_start: 1,
            },
            S5DiffRow::Line {
                kind: DiffLineKind::Remove,
                text: "a\tb".into(),
            },
            S5DiffRow::Line {
                kind: DiffLineKind::Add,
                text: "a\tc".into(),
            },
        ],
    };
    let mut rig = S5DiffApp::new(Some(tabbed), Rect::new(0, 1, 78, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "N2: the view takes focus");
    let all = app.text();
    assert!(!all.contains('\t'), "N2: no raw tabs survive");
    let tabbed_row = all
        .lines()
        .map(str::to_owned)
        .find(|l| l.contains("a    b"))
        .expect("N2: the tab expands to four spaces");
    let halves: Vec<&str> = tabbed_row.split(" │ ").collect();
    assert_eq!(
        halves[0].trim_end().chars().count(),
        halves[1].trim_end().chars().count(),
        "N2: columns measure identically"
    );
    // V2/S2/A2 headless: drag selects, y copies, Esc clears, wheel scrolls.
    let mut rig = S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 78, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "V2: the view takes focus");
    let r = app.mouse(MouseKind::Down, 10, 4);
    assert_eq!(r.flow(), Flow::Consumed, "V2: click anchors");
    let _ = app.mouse(MouseKind::Up, 10, 4);
    let r = app.mouse(MouseKind::Down, 10, 4);
    assert_eq!(r.flow(), Flow::Consumed, "V2: drag anchors");
    let r = app.mouse(MouseKind::Drag, 22, 4);
    assert_eq!(r.flow(), Flow::Consumed, "V2: drag selects");
    let _ = app.mouse(MouseKind::Up, 22, 4);
    assert!(
        app.app().state.viewport().selection().is_some(),
        "V2: the range selects"
    );
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "S2: y runs");
    let copied = match app.app().actions[before..].last() {
        Some(ViewportAction::Copy(t)) => t.clone(),
        other => panic!("S2: y emits Copy, got {other:?}"),
    };
    assert!(!copied.is_empty(), "S2: the copy keeps text");
    assert!(
        app.app().state.viewport().selection().is_some(),
        "V2: copy keeps the selection"
    );
    let r = app.wheel(Axis::V, 2, 10, 4);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A2: wheel scrolls via the viewport"
    );
    // Two notches at the design's 3-rows-per-notch wheel scaling.
    assert_eq!(
        app.app().state.viewport().scroll().offset(),
        6,
        "A2: wheel scrolls via the viewport"
    );
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Esc clears");
    assert!(
        app.app().state.viewport().selection().is_none(),
        "A2: Esc clears the selection"
    );
    // S1: review falls back below 43 cells; the retained mode stays Review.
    let mut rig = S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 78, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "S1: the view takes focus");
    assert_eq!(
        app.app().state.mode(),
        DiffMode::Review,
        "S1: the mode is retained"
    );
    let mut rig = S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 42, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "S1: the narrow view takes focus");
    let _ = app.tick();
    let narrow = app.text();
    assert_eq!(
        app.app().state.mode(),
        DiffMode::Review,
        "S1: the mode is retained at 42 cells"
    );
    assert!(
        !narrow.contains('│') && narrow.contains("@@"),
        "S1: 42 cells fall back to unified\n{narrow}"
    );
    // V1 headless: the Old │ New column headers.
    let mut rig = S5DiffApp::new(Some(S5DiffSource::sample()), Rect::new(0, 1, 78, 22));
    rig.state.set_mode(DiffMode::Review);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S5_DIFF), "V1: the view takes focus");
    let wide = app.text();
    assert!(
        wide.lines().any(|l| l.contains("Old") && l.contains("New")),
        "V1: the column headers\n{wide}"
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
    let sel_bg = frame.get(col + 8, row).expect("V2: style cell").bg;
    for c in col + 4..col + 16 {
        assert_eq!(
            frame.get(c, row).map(|cell| &cell.bg),
            Some(&sel_bg),
            "V2 live: the dragged range paints one style"
        );
    }
    let before = frame.get(col - 1, row).expect("V2: lead cell").bg;
    assert_ne!(sel_bg, before, "V2 live: the style starts past the lead");
    let edge = frame.get(col + 22, row).expect("V2: edge cell").bg;
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
/// terminal page, outside the importable library, so every check runs
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

// ------------------------------------------------------------ viewport rig --

const S5_TERM: Id = Id::root("s5.term");

/// Viewport rig: owned transcript lines plus the action log.
struct S5TermApp {
    state: ViewportState,
    lines: Vec<String>,
    actions: Vec<ViewportAction>,
    area: Rect,
}

impl S5TermApp {
    fn transcript(n: usize, area: Rect) -> Self {
        Self {
            state: ViewportState::default(),
            lines: (0..n)
                .map(|i| format!("line {i:03} of the transcript"))
                .collect(),
            actions: Vec::new(),
            area,
        }
    }
}

impl App for S5TermApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let lines: Vec<ViewportLine<'_>> =
            self.lines.iter().map(|l| ViewportLine::Plain(l)).collect();
        let mut response = TextViewport::new(S5_TERM).update(cx, &mut self.state, &lines);
        if let Some(action) = response.take_action() {
            self.actions.push(action);
        }
        response.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let lines: Vec<ViewportLine<'_>> =
            self.lines.iter().map(|l| ViewportLine::Plain(l)).collect();
        TextViewport::new(S5_TERM).draw(ui, self.area, &self.state, &lines);
    }
}

/// TERM-FOLLOW-002 (`showcase/pages/terminal`, 80x24): follow tail,
/// scrollback, selection, and copy.
///
/// Isolated path drives a [`TextViewport`] over a transcript: wheel and
/// scrollbar moves recomputing follow from the tail, `Copy` reporting
/// with the text retained, `f` toggling follow with `FollowChanged`,
/// drag auto-scroll at the edges from the press anchor, the
/// `y`-with-no-selection `Consumed` negative, the thumb-press-grabs
/// negative, and Esc clearing. (Check order runs the supported forms
/// before the Esc gap so it fails last, not first.) Live path boots the
/// frame-60 terminal, wheels off the tail, drags a selection, and copies
/// it with `y`.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_term_follow() {
    let dir = s5_dir("s5_term_follow");

    // N1: y with no selection must NOT emit Copy.
    let mut app = Harness::new(
        S5TermApp::transcript(60, Rect::new(0, 1, 60, 20)),
        Theme::junie(),
        60,
        22,
    );
    assert!(app.tab_to(S5_TERM), "N1: the viewport takes focus");
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "N1: y with no selection is Consumed"
    );
    assert!(
        app.app().actions[before..].is_empty(),
        "N1: y with no selection emits no Copy"
    );
    // S1 headless: wheel moves recompute follow from the tail and
    // remember the reading row.
    let mut app = Harness::new(
        S5TermApp::transcript(60, Rect::new(0, 1, 60, 20)),
        Theme::junie(),
        60,
        22,
    );
    assert!(app.tab_to(S5_TERM), "S1: the viewport takes focus");
    capture_isolated(&dir, "term-viewport", app.buffer());
    assert!(
        app.app().state.follow() && app.app().state.scroll().at_end(),
        "S1: follow starts at the tail"
    );
    let r = app.wheel(Axis::V, -3, 30, 10);
    assert_eq!(r.flow(), Flow::Consumed, "S1: wheel up moves");
    assert!(!app.app().state.follow(), "S1: wheeling up breaks follow");
    assert!(!app.app().state.scroll().at_end(), "S1: off the tail");
    assert!(
        app.app().state.scroll().offset() < app.app().state.scroll().max_offset(),
        "S1: scrollback depth grows"
    );
    let r = app.wheel(Axis::V, 100, 30, 10);
    assert_eq!(r.flow(), Flow::Consumed, "S1: wheel down returns");
    assert!(
        app.app().state.follow() && app.app().state.scroll().at_end(),
        "S1: the tail resumes follow"
    );
    // V1/S2 headless: a dragged selection reports Copy with its text.
    let r = app.mouse(MouseKind::Down, 2, 2);
    assert_eq!(r.flow(), Flow::Consumed, "click anchors");
    let _ = app.mouse(MouseKind::Up, 2, 2);
    let _ = app.mouse(MouseKind::Down, 2, 2);
    let r = app.mouse(MouseKind::Drag, 12, 4);
    assert_eq!(r.flow(), Flow::Consumed, "drag extends");
    let _ = app.mouse(MouseKind::Up, 12, 4);
    assert!(
        app.app().state.selection().is_some(),
        "V1: the selection exists"
    );
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "S2: y runs");
    let copied = match app.app().actions[before..].last() {
        Some(ViewportAction::Copy(t)) => t.clone(),
        other => panic!("S2: y emits Copy, got {other:?}"),
    };
    assert_eq!(copied.lines().count(), 3, "S2: Copied 3 lines");
    assert!(
        copied.lines().next().is_some_and(|l| !l.is_empty()),
        "S2: the first line is retained"
    );
    // A1 headless: f toggles follow with FollowChanged.
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Char('f'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: f toggles follow");
    assert_eq!(
        &app.app().actions[before..],
        &[ViewportAction::FollowChanged(false)],
        "A1: f reports FollowChanged"
    );
    // A2 headless: drag auto-scrolls at the vertical edges, extending
    // from the press anchor.
    let _ = app.wheel(Axis::V, -10, 30, 10);
    let before_off = app.app().state.scroll().offset();
    assert!(before_off > 0, "A2: room to scroll up");
    let r = app.mouse(MouseKind::Down, 2, 10);
    assert_eq!(r.flow(), Flow::Consumed, "A2: click anchors");
    let _ = app.mouse(MouseKind::Up, 2, 10);
    let _ = app.mouse(MouseKind::Down, 2, 10);
    let _ = app.mouse(MouseKind::Drag, 2, 0);
    let _ = app.mouse(MouseKind::Up, 2, 0);
    assert_eq!(
        app.app().state.scroll().offset(),
        before_off - 1,
        "A2: a drag above the area scrolls up"
    );
    assert!(
        app.app().state.selection().is_some(),
        "A2: the drag extends from the anchor"
    );
    // N2: a scrollbar press on the thumb must NOT jump: it grabs.
    // At the end the thumb sits on the last track row.
    let mut app = Harness::new(
        S5TermApp::transcript(60, Rect::new(0, 1, 60, 20)),
        Theme::junie(),
        60,
        22,
    );
    assert!(app.tab_to(S5_TERM), "N2: the viewport takes focus");
    assert!(
        app.app().state.follow() && app.app().state.scroll().at_end(),
        "N2: follow starts at the tail"
    );
    let offset = app.app().state.scroll().offset();
    let r = app.mouse(MouseKind::Down, 59, 20);
    assert_eq!(r.flow(), Flow::Consumed, "N2: a thumb press grabs");
    let _ = app.mouse(MouseKind::Up, 59, 20);
    assert_eq!(
        app.app().state.scroll().offset(),
        offset,
        "N2: a thumb press grabs without jumping"
    );
    // A2 headless: Esc clears the selection.
    let mut app = Harness::new(
        S5TermApp::transcript(60, Rect::new(0, 1, 60, 20)),
        Theme::junie(),
        60,
        22,
    );
    assert!(app.tab_to(S5_TERM), "A2: the viewport takes focus");
    let _ = app.mouse(MouseKind::Down, 2, 2);
    let _ = app.mouse(MouseKind::Up, 2, 2);
    let _ = app.mouse(MouseKind::Down, 2, 2);
    let _ = app.mouse(MouseKind::Drag, 12, 4);
    let _ = app.mouse(MouseKind::Up, 12, 4);
    assert!(
        app.app().state.selection().is_some(),
        "A2: the selection exists"
    );
    let before = app.app().actions.len();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Esc clears the selection");
    assert_eq!(
        &app.app().actions[before..],
        &[ViewportAction::SelectionChanged],
        "A2: Esc reports SelectionChanged"
    );
    assert!(
        app.app().state.selection().is_none(),
        "A2: the selection is gone"
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

// ------------------------------------------------------------- scroll rig --

const S5_SCROLL: Id = Id::root("s5.scroll");

/// Mirror of the scrolling page's `position_label` (page-owned fact):
/// `a–b of n` over the visible range, empty when content fits.
fn s5_position_label(scroll: &ScrollState) -> String {
    if !scroll.overflows() {
        return String::new();
    }
    let range = scroll.visible_range();
    format!(
        "{}–{} of {}",
        range.start.saturating_add(1),
        range.end,
        scroll.content_len()
    )
}

/// Bare [`ScrollRegion`] rig: caller-owned [`ScrollState`] over a
/// content length, remembering the content rect draw returned (the
/// region reserves the bar column only when the content overflows).
struct S5ScrollApp {
    state: ScrollState,
    len: usize,
    area: Rect,
    content: std::cell::Cell<Rect>,
}

impl S5ScrollApp {
    fn new(len: usize, area: Rect) -> Self {
        Self {
            state: ScrollState::new(len),
            len,
            area,
            content: std::cell::Cell::new(area),
        }
    }
}

impl App for S5ScrollApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        ScrollRegion::new(S5_SCROLL).update(cx, &mut self.state, self.len)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let content = ScrollRegion::new(S5_SCROLL).draw(ui, self.area, &self.state, self.len);
        self.content.set(content);
    }
}

/// SCROLL-REGION-001 (`showcase/flows/scrolling/scrolled`, 80x24): the
/// three scrolling columns plus position labels.
///
/// Isolated path drives a tailing [`TextViewport`] log: paging by the
/// viewport, `g`/`G` jumps, `f` toggling follow, plus the fitting-column
/// negative (no scrollbar, no label) through [`ScrollRegion`] and the
/// page's label mirror. (The prose-never-follows negative runs after the
/// live boots so the known gap fails last, not first.) Live path boots
/// the frame-1600 scrolling page wide (labels, track, thumb, follow
/// toggles) and narrow (wheel-under-pointer, keys-to-focused,
/// page/ends), plus a playing boot proving the log grows a line per
/// tick and stops at the 2000 cap.
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_scroll_region() {
    let dir = s5_dir("s5_scroll_region");

    // A1 headless: PageUp/Down page by the viewport length; g/G jump.
    let mut app = Harness::new(
        S5TermApp::transcript(400, Rect::new(0, 1, 40, 15)),
        Theme::junie(),
        40,
        17,
    );
    assert!(app.tab_to(S5_TERM), "A1: the log takes focus");
    capture_isolated(&dir, "scroll-panel", app.buffer());
    let vp = app.app().state.scroll().viewport_len();
    assert!(vp > 1, "A1: a viewport lays out");
    let end = app.app().state.scroll().max_offset();
    let r = app.key(KeyCode::Char('G'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: G runs");
    assert_eq!(
        app.app().state.scroll().offset(),
        end,
        "A1: at the live end"
    );
    let r = app.key(KeyCode::PageUp);
    assert_eq!(r.flow(), Flow::Consumed, "A1: PageUp runs");
    assert_eq!(r.invalidate(), Invalidate::Paint, "A1: PageUp repaints");
    assert_eq!(
        app.app().state.scroll().offset(),
        end - vp,
        "A1: PageUp pages by the viewport"
    );
    let r = app.key(KeyCode::PageDown);
    assert_eq!(r.flow(), Flow::Consumed, "A1: PageDown runs");
    assert_eq!(
        app.app().state.scroll().offset(),
        end,
        "A1: PageDown pages back"
    );
    let _ = app.key(KeyCode::Char('g'));
    assert_eq!(
        app.app().state.scroll().offset(),
        0,
        "A1: g jumps to the start"
    );
    let _ = app.key(KeyCode::Char('G'));
    assert_eq!(
        app.app().state.scroll().offset(),
        end,
        "A1/A2: G jumps to the live end"
    );
    // A2 headless: f toggles log follow.
    assert!(
        app.app().state.follow(),
        "A2: a tailing log follows at the end"
    );
    let before = app.app().actions.len();
    let _ = app.key(KeyCode::Char('f'));
    assert!(!app.app().state.follow(), "A2: f pauses follow");
    assert_eq!(
        &app.app().actions[before..],
        &[ViewportAction::FollowChanged(false)],
        "A2: f reports FollowChanged"
    );
    let _ = app.key(KeyCode::Char('f'));
    assert!(
        app.app().state.follow() && app.app().state.scroll().at_end(),
        "A2: f resumes at the end"
    );
    assert_eq!(
        s5_position_label(app.app().state.scroll()),
        format!("{}–400 of 400", end + 1),
        "V1: the tail label"
    );
    // N1 headless: a fitting column draws no scrollbar and no label.
    let mut fits = ScrollState::new(3);
    fits.set_viewport(5);
    assert_eq!(
        s5_position_label(&fits),
        "",
        "N1: no label when content fits"
    );
    let fit = Harness::new(
        S5ScrollApp::new(3, Rect::new(0, 1, 5, 5)),
        Theme::junie(),
        5,
        7,
    );
    assert_eq!(
        fit.app().content.get(),
        Rect::new(0, 1, 5, 5),
        "N1: no bar column reserved when content fits"
    );
    for y in 0..7u16 {
        let row = fit.row(y);
        assert!(
            !row.contains('│') && !row.contains('┃'),
            "N1: no scrollbar when content fits, row {y}: {row:?}"
        );
    }
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
    // The labels derive from the previous frame's layout facts, so they
    // land a frame after the tail; the needles stay byte-identical.
    waits::wait_state(&mut w, "of 120", "V1 live: list label never showed");
    waits::wait_state(
        &mut w,
        "of 2000 · following",
        "V1 live: log label never showed",
    );
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
    waits::wait_state(&mut s, "1–15 of", "V1 live: prose label never showed");
    waits::wait_state(&mut s, "1986–2000", "V1 live: log window never showed");
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
    // A2 headless, last: prose never follows. Fresh viewports start
    // following, so the first `f` pauses; the second must be a no-op.
    // Runs after the live boots so the known gap fails last, not first.
    let mut prose = Harness::new(
        S5TermApp::transcript(2, Rect::new(0, 1, 40, 15)),
        Theme::junie(),
        40,
        17,
    );
    assert!(prose.tab_to(S5_TERM), "A2: prose takes focus");
    let _ = prose.key(KeyCode::Char('f'));
    assert!(!prose.app().state.follow(), "A2: f pauses prose");
    let _ = prose.key(KeyCode::Char('f'));
    assert!(!prose.app().state.follow(), "A2: prose never follows");
    eprintln!("s5 scroll_region: columns + routing + growth + cap");
}

/// SCROLL-TRACK-002 (`showcase/flows/scrolling/scrolled`, 80x24): the
/// scrollbar track press plus thumb drag.
///
/// Isolated path drives [`ScrollState`] over a 2000-line log mirror plus
/// the [`ScrollRegion`] wrappers: proportional thumb geometry with the
/// 1-cell floor, thumb-press holds vs track-press centering, grab-kept
/// drags, press-fallback drags, release clearing, consumed-vs-repaint
/// responses, and the no-overflow no-ops. Live path boots the
/// frame-1600 scrolling page, presses the log thumb (holds), drags it
/// two rows (follows), and presses the bare track (jumps centered).
#[test]
#[ignore = "showcase s5 check; run with --ignored"]
fn s5_scroll_track() {
    let dir = s5_dir("s5_scroll_track");

    // V1 headless: the thumb spans viewport/content of the track, ≥1 cell.
    let mut log = ScrollState::new(2000);
    log.set_viewport(15);
    assert_eq!(log.thumb(15), (0, 1), "V1: top thumb spans one cell");
    log.jump_end();
    assert_eq!(log.thumb(15), (14, 1), "V1: end thumb sits at the bottom");
    assert_eq!(
        s5_position_label(&log),
        "1986–2000 of 2000",
        "V1 live shape: the tail label"
    );
    // V2/S1 headless: a thumb press holds the offset (grab only).
    let mut held = ScrollState::new(2000);
    held.set_viewport(15);
    held.press_track(0, 15);
    assert_eq!(held.offset(), 0, "V2: pressing the thumb holds");
    assert_eq!(held.offset(), 0, "N1: a thumb press moves nothing");
    // S1: dragging keeps the grabbed row under the pointer.
    held.drag_track(2, 15);
    assert!(held.offset() > 0, "S1: the drag moves");
    assert_eq!(held.thumb(15).0, 2, "S1: the thumb follows the pointer");
    // V2/A2 headless: a track press centers the thumb under the pointer.
    let mut jumped = ScrollState::new(2000);
    jumped.set_viewport(15);
    jumped.press_track(0, 15);
    assert_eq!(jumped.offset(), 0, "V2: top press holds at top");
    jumped.press_track(14, 15);
    assert!(jumped.offset() > 0, "V2: a track press jumps");
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
    fresh.drag_track(14, 15);
    assert!(fresh.at_end(), "S2: the jump reaches the end");
    fresh.release_track();
    fresh.scroll_to(0);
    fresh.press_track(0, 15);
    assert_eq!(fresh.offset(), 0, "S2: a new press grabs again");
    // A1: press/drag report consumed-vs-repaint through the region.
    let mut region = Harness::new(
        S5ScrollApp::new(2000, Rect::new(0, 1, 2, 15)),
        Theme::junie(),
        2,
        17,
    );
    let _ = region.tick();
    assert_eq!(
        region.app().state.viewport_len(),
        15,
        "A1: the layout applies"
    );
    let r = region.mouse(MouseKind::Down, 1, 1);
    assert_eq!(r.flow(), Flow::Consumed, "A1: the thumb press grabs");
    // The grab is an offset noop, but the region repaints: the thumb
    // wears PRESSED while captured (contract §states).
    assert_eq!(r.invalidate(), Invalidate::Paint, "A1: the grab repaints");
    assert_eq!(region.app().state.offset(), 0, "A1: the offset holds");
    let r = region.mouse(MouseKind::Drag, 1, 1);
    assert_eq!(r.flow(), Flow::Consumed, "A1: the null drag runs");
    assert_eq!(r.invalidate(), Invalidate::None, "A1: null drag is a noop");
    assert_eq!(region.app().state.offset(), 0, "A1: the null drag holds");
    let _ = region.mouse(MouseKind::Up, 1, 1);
    // The region wrappers map through the same core.
    let r = region.mouse(MouseKind::Down, 1, 1);
    assert_eq!(r.flow(), Flow::Consumed, "wrappers: thumb press holds");
    assert_eq!(region.app().state.offset(), 0, "wrappers: hold");
    let r = region.mouse(MouseKind::Drag, 1, 3);
    assert_eq!(r.flow(), Flow::Consumed, "wrappers: drag follows");
    assert_eq!(region.app().state.thumb(15).0, 2, "wrappers: grab kept");
    let _ = region.mouse(MouseKind::Up, 1, 3);
    // N2: without overflow, press and drag change nothing.
    let mut fits = ScrollState::new(3);
    fits.set_viewport(5);
    fits.press_track(2, 5);
    assert_eq!(fits.offset(), 0, "N2: press is a noop");
    fits.drag_track(4, 5);
    assert_eq!(fits.offset(), 0, "N2: drag is a noop");
    // Render form: │ track with the ┃ thumb, contained.
    let mut rendered = Harness::new(
        S5ScrollApp::new(2000, Rect::new(0, 1, 2, 15)),
        Theme::junie(),
        2,
        17,
    );
    let _ = rendered.tick();
    assert_eq!(rendered.app().state.offset(), 0, "V2: at the top");
    let top = rendered.row(1);
    assert_eq!(
        top.chars().nth(1),
        Some('┃'),
        "V2: the thumb leads at the top, got {top:?}"
    );
    let next = rendered.row(2);
    assert_eq!(
        next.chars().nth(1),
        Some('│'),
        "V2: the track follows, got {next:?}"
    );
    capture_isolated(&dir, "scroll-thumb", rendered.buffer());

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
    s.inner
        .mouse_down(MouseButton::Left, track_col, thumb_mid, MouseMods::NONE)
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
    s.inner
        .mouse_drag(MouseButton::Left, track_col, target, MouseMods::NONE)
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
    s.inner
        .mouse_up(MouseButton::Left, track_col, target, MouseMods::NONE)
        .expect("thumb release");
    std::thread::sleep(Duration::from_millis(300));
    // V2/A2 live: a bare-track press jumps the thumb under the pointer.
    // Two rows inside the top: the first row may sit above the track's
    // hit rect (a border │), which would swallow the press silently.
    let slot = track_top + 2;
    s.inner
        .mouse_down(MouseButton::Left, track_col, slot, MouseMods::NONE)
        .expect("track press");
    s.inner
        .mouse_up(MouseButton::Left, track_col, slot, MouseMods::NONE)
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
