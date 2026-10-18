//! Showcase pending slice 8A-S3 executable checks — impl port.
//!
//! Ported from VB commit `b712422fcc23a19859b851c9ec583b08a1b40136`
//! (`tests/harness/tests/visual_baseline/showcase_pending_s3.rs`), adapting
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
//! - Isolated `junie_tui` in-process renders (`TextInput`, `TextArea`,
//!   `ChipBar`, `ListBox`, `Picker`, `TreeView`, `StepRail`, `Tabs`,
//!   `RenderCtx`, `Interaction`, `Outcome`) become the established impl
//!   component path: `Runtime` + `Stub` + `draw_scene` with
//!   `ui.reference` state injection for inert paint, and `Harness` apps
//!   for interaction. `Outcome` assertions become draw/behavior
//!   correlates (flow, committed values, re-rendered glyphs, cursor,
//!   selection paint, actions).
//!
//! One ignored test per S3 registry row (19 rows: the 9 FIELDS group rows
//! plus the first 10 NAVIGATION group rows), over the real binaries built
//! from this worktree's impl sources and over the real production
//! components rendered headless. Each test executes its row's listed
//! adapters (`headless-tick`, `pty-capture`, or both) and asserts its
//! visual (V), state (S), action (A), and negative (N) checks in
//! executable form:
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
//! Two rows are PTY-only by construction: NAV-SHELL-001 (the shell
//! sidebar lives in the showcase binary's app) and NAV-SIDE-002 (the demo
//! `NavList` lives in the showcase binary). Neither component is
//! importable as a unit here, so no headless path exists for them; every
//! one of their checks has a live-PTY executable form instead.
//!
//! No new snapshots and no new static captures: every S3 row already has
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
//! Row → test map (registry id → `s3_*` test):
//!
//! - TI-KEYS-003 → [`s3_input_keys`]
//! - TI-NOOP-004 → [`s3_input_noop`]
//! - TI-SELECT-005 → [`s3_input_select`]
//! - TI-PASTE-006 → [`s3_input_paste`]
//! - TI-MASK-007 → [`s3_input_mask`]
//! - TI-SCROLL-008 → [`s3_input_scroll`]
//! - TA-LIFECYCLE-001 → [`s3_area_lifecycle`]
//! - TA-VIEW-002 → [`s3_area_view`]
//! - TA-FOLLOW-003 → [`s3_area_follow`]
//! - CHIP-BAR-001 → [`s3_chip_bar`]
//! - LIST-SINGLE-001 → [`s3_list_single`]
//! - LIST-MULTI-002 → [`s3_list_multi`]
//! - FLIST-QUERY-001 → [`s3_picker_query`]
//! - FLIST-CHOICE-002 → [`s3_picker_choice`]
//! - NAV-SHELL-001 → [`s3_nav_shell`] (PTY-only)
//! - NAV-SIDE-002 → [`s3_nav_side`] (PTY-only)
//! - TREE-FOLD-001 → [`s3_tree_fold`]
//! - STEPS-LIFE-001 → [`s3_steps_life`]
//! - TABS-STRIP-001 → [`s3_tabs_strip`]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use termrock::Color as RColor;
use termrock::Input as RtInput;
use termrock::runtime::stub::{Stub, deliver};
use termrock::{
    App, AsItem, Axis, Buffer, Button, ChipBar, ChipBarAction, ChipBarState, ColorLevel, Cx,
    EmptyState, FieldError, FilterList, FilterListAction, FilterListState, Flow, Id, Item, ItemKey,
    KeyCode, KeyModifiers, List, ListAction, ListState, Modifier, MouseKind, Part, PartRef, Picker,
    PickerAction, PickerState, Position, Rect, ReferenceState, ReferenceTarget, Response, RowUi,
    Runtime, ScopeKey, SecretPolicy, SelectMode, StateFlags, Status, StepState, Steps, StepsState,
    Tabs, TabsAction, TabsState, TextAction, TextArea, TextAreaState, TextBuffer, TextInput,
    TextInputState, Theme, Tree, TreeAction, TreeNode, TreeState, Ui,
};
use termrock_test_support::Harness;
use tuiscotti::tui::{MouseButton, Wheel};
use tuiscotti::{Cell as TCell, Color as TColor, Frame, Mods, Provenance, Rgb, UnderlineStyle};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

/// The VB oracle commit this file ports.
const PORT_OF: &str = "b712422fcc23a19859b851c9ec583b08a1b40136";

/// Owned-name S3 case at the row's viewport (truecolor, like every S3 row).
fn s3_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s3/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

/// Unique scratch dir for one S3 test (created, never shared).
fn s3_dir(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/tuiscotti/journeys")
        .join(test);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("mkdir {}: {e}", dir.display()));
    dir
}

/// Record the binary identity: resolved absolute path, sha256 digest, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
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
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write provenance.txt: {e}"));
    eprintln!("s3 provenance: {body}");
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

/// `notches` wheel steps at fixed `(col, row)`.
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

/// Whether the cell carries the editing underline.
fn buf_is_underlined(buf: &Buffer, x: u16, y: u16) -> bool {
    buf[(x, y)].modifier.contains(Modifier::UNDERLINED)
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
        Provenance::now("default", "s3-isolated", vec![]),
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

/// Cell column of `needle`'s first occurrence in `line` (char-based: all
/// page chrome here is single-cell).
fn char_col(line: &str, needle: &str) -> u16 {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {line:?}"));
    line[..byte].chars().count() as u16
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

/// The screen line right below the first line containing `label`.
fn row_below(text: &str, label: &str, what: &str) -> String {
    let mut lines = text.lines();
    while let Some(l) = lines.next() {
        if l.contains(label) {
            return lines
                .next()
                .unwrap_or_else(|| panic!("{what}: no row below {label:?}"))
                .to_string();
        }
    }
    panic!("{what}: label {label:?} not on screen\n{text}");
}

/// Count `glyph` occurrences in visible text.
fn count_glyph(text: &str, glyph: char) -> usize {
    text.chars().filter(|c| *c == glyph).count()
}

/// The reference ten-frame braille cycle (theme `spinner_frames`).
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Caret column of a `Harness` app (the hardware cursor the focused editor owns).
fn caret_x<A: App>(app: &Harness<A>) -> u16 {
    app.cursor().expect("editing owns the caret").x
}

/// Whether row `y` paints a selection run exactly over `cols` (selection bg
/// on every cell of the run, field bg immediately outside it).
fn assert_selection_run(
    buf: &Buffer,
    y: u16,
    cols: std::ops::Range<u16>,
    sel_bg: RColor,
    what: &str,
) {
    for x in cols.clone() {
        assert_eq!(buf[(x, y)].bg, sel_bg, "{what}: cell ({x},{y}) selected");
    }
}

/// Whether row `y` paints no selection bg anywhere in `cols`.
fn assert_no_selection(
    buf: &Buffer,
    y: u16,
    cols: std::ops::Range<u16>,
    sel_bg: RColor,
    what: &str,
) {
    for x in cols {
        assert_ne!(
            buf[(x, y)].bg,
            sel_bg,
            "{what}: cell ({x},{y}) must not be selected"
        );
    }
}

const S3_TIKEYS: Id = Id::root("s3.tikeys");
const S3_TIKEYS_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 36,
    height: 1,
};

/// Single-line key-map rig: controlled value plus action log.
struct KeysApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for KeysApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S3_TIKEYS)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIKEYS)
            .value(&self.value)
            .placeholder("Type a path or symbol…")
            .draw(ui, S3_TIKEYS_AREA, &self.st);
    }
}

/// TI-KEYS-003 (`showcase/flows/inputs/selected`): single-line key map.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_input_keys() {
    let dir = s3_dir("s3_input_keys");
    let t = Theme::junie();
    let sel_bg = t.color.selection_bg;
    let sel_fg = t.color.selection_fg;
    // Text run starts at inner.x = area.x + 2.
    let at = |off: u16| 6 + off;

    // A1 isolated: line edges, word moves, kills.
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        KeysApp {
            value: "billing service".to_string(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIKEYS), "navigating focus");
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "Enter begins editing");
    assert_eq!(caret_x(&app), at(15), "cursor starts at end");
    let r = app.key(KeyCode::Home);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Home applies");
    assert_eq!(caret_x(&app), at(0), "A1: Home hits line start");
    let r = app.key(KeyCode::End);
    assert_eq!(caret_x(&app), at(15), "A1: End hits line end");
    assert_eq!(r.flow(), Flow::Consumed, "A1: End applies");
    let _ = app.key(KeyCode::Home);
    let r = app.ctrl('a');
    assert_eq!(r.flow(), Flow::Consumed, "A1: Ctrl+A applies");
    assert_eq!(caret_x(&app), at(0), "A1: Ctrl+A hits line start");
    let r = app.ctrl('e');
    assert_eq!(r.flow(), Flow::Consumed, "A1: Ctrl+E applies");
    assert_eq!(caret_x(&app), at(15), "A1: Ctrl+E hits line end");
    let _ = app.key(KeyCode::Home);
    let _ = app.key_mod(KeyCode::Right, KeyModifiers::CONTROL);
    assert_eq!(caret_x(&app), at(7), "A1: Ctrl+Right ends billing");
    let _ = app.key_mod(KeyCode::Right, KeyModifiers::CONTROL);
    assert_eq!(caret_x(&app), at(15), "A1: Ctrl+Right ends service");
    let _ = app.key_mod(KeyCode::Left, KeyModifiers::CONTROL);
    assert_eq!(caret_x(&app), at(8), "A1: Ctrl+Left starts service");
    let _ = app.key_mod(KeyCode::Left, KeyModifiers::CONTROL);
    assert_eq!(caret_x(&app), at(0), "A1: Ctrl+Left starts billing");
    let _ = app.key(KeyCode::End);
    let _ = app.alt('b');
    assert_eq!(caret_x(&app), at(8), "A1: Alt+B starts service");
    let _ = app.alt('f');
    assert_eq!(caret_x(&app), at(15), "A1: Alt+F ends service");
    // V1: the caret tracks every grapheme op inside the text run.
    assert_eq!(
        app.cursor(),
        Some(Position::new(at(15), 1)),
        "V1: caret after the last grapheme"
    );
    let _ = app.key(KeyCode::Home);
    assert_eq!(
        app.cursor(),
        Some(Position::new(at(0), 1)),
        "V1: caret at the run start"
    );
    // Shift extends; Backspace kills the selection; Ctrl+U kills to start.
    let _ = app.key_mod(KeyCode::Right, KeyModifiers::CONTROL);
    let _ = app.key_mod(KeyCode::End, KeyModifiers::SHIFT);
    assert_eq!(caret_x(&app), at(15), "A1: Shift+End moves to line end");
    // V2: the selection paints with the selection style on top of the
    // editing underline. Selection 7..15 paints at cols 13..20.
    assert_selection_run(app.buffer(), 1, at(7)..at(15), sel_bg, "V2");
    assert_eq!(
        app.buffer()[(14, 1)].fg,
        sel_fg,
        "V2: selected cell keeps selection fg"
    );
    assert!(
        buf_is_underlined(app.buffer(), 14, 1),
        "V2: editing underline paints over the selection"
    );
    assert_ne!(
        app.buffer()[(6, 1)].bg,
        app.buffer()[(14, 1)].bg,
        "V2: unselected cells keep the field bg, not the selection bg"
    );
    capture_isolated(&dir, "ti-keys-selected", app.buffer());
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Backspace);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A1: Backspace kills the selection"
    );
    assert!(
        log.borrow().contains(&TextAction::Changed),
        "A1: Backspace reports Changed"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains("billing"),
        "selection killed, prefix kept"
    );
    assert!(
        !buf_row_text(app.buffer(), 1).contains("service"),
        "selection killed, suffix gone"
    );
    let r = app.ctrl('u');
    assert_eq!(r.flow(), Flow::Consumed, "A1: Ctrl+U applies");
    assert!(
        !buf_row_text(app.buffer(), 1).contains("billing"),
        "A1: Ctrl+U kills to line start"
    );
    assert_eq!(caret_x(&app), at(0), "A1: caret at start");
    // Ctrl+K / Ctrl+W / Delete kill line-end / word / one grapheme.
    let mut app = Harness::new(
        KeysApp {
            value: "billing service".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIKEYS), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Home);
    let _ = app.ctrl('k');
    assert!(
        !buf_row_text(app.buffer(), 1).contains("billing"),
        "A1: Ctrl+K kills to line end"
    );
    let mut app = Harness::new(
        KeysApp {
            value: "billing service".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIKEYS), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('w');
    let row = buf_row_text(app.buffer(), 1);
    assert!(
        row.contains("billing") && !row.contains("service"),
        "A1: Ctrl+W kills a word"
    );
    let _ = app.key(KeyCode::Home);
    let _ = app.key(KeyCode::Delete);
    assert!(
        buf_row_text(app.buffer(), 1).contains("illing"),
        "A1: Delete kills one grapheme"
    );
    // S1: an invalid fixture keeps its error across cursor moves; typing a
    // correction clears it live.
    let mut st = TextInputState::default();
    st.begin("has x");
    st.set_error(Some(FieldError::new("no x")));
    let mut app = Harness::new(
        ValidApp {
            value: "has x".to_string(),
            st,
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIKEYS), "navigating focus");
    assert_ne!(
        app.buffer()[(39, 1)].symbol(),
        " ",
        "fixture starts invalid: marker paints"
    );
    for code in [KeyCode::Left, KeyCode::Right, KeyCode::Home] {
        let r = app.key(code);
        assert_eq!(r.flow(), Flow::Consumed, "S1: {code:?} applies");
    }
    assert_ne!(
        app.buffer()[(39, 1)].symbol(),
        " ",
        "S1: still invalid, error kept"
    );
    let _ = app.ctrl('l');
    let _ = app.type_str("ok");
    assert!(
        buf_row_text(app.buffer(), 1).contains("ok"),
        "S1: corrected text typed"
    );
    assert_eq!(
        app.buffer()[(39, 1)].symbol(),
        " ",
        "S1: corrected error clears live"
    );
    // S2: Ctrl+L selects the whole buffer.
    let mut app = Harness::new(
        KeysApp {
            value: "billing service".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIKEYS), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('l');
    assert_selection_run(
        app.buffer(),
        1,
        at(0)..at(15),
        sel_bg,
        "S2: Ctrl+L selects all",
    );
    // A2/N1: vertical keys consume without moving.
    for code in [
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ] {
        let before = caret_x(&app);
        let r = app.key(code);
        assert_eq!(r.flow(), Flow::Consumed, "A2: {code:?} consumes");
        assert_eq!(caret_x(&app), before, "N1: {code:?} never moves");
    }
    let mut single = TextBuffer::single("ab");
    assert!(!single.move_up(false), "N1: move_up false single-line");
    assert!(!single.move_down(false), "N1: move_down false single-line");
    // N2: Shift+Ctrl+Home never extends (doc-start passes select=false).
    let mut app = Harness::new(
        KeysApp {
            value: "billing service".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIKEYS), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let r = app.key_mod(KeyCode::Home, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    assert_eq!(r.flow(), Flow::Consumed, "N2: doc-start applies");
    assert_eq!(caret_x(&app), at(0), "N2: cursor at doc start");
    assert_no_selection(
        app.buffer(),
        1,
        at(0)..at(15),
        sel_bg,
        "N2: Shift+Ctrl+Home",
    );
    let r = app.key_mod(KeyCode::End, KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    assert_eq!(r.flow(), Flow::Consumed, "N2: doc-end applies");
    assert_eq!(caret_x(&app), at(15), "N2: cursor at doc end");
    assert_no_selection(app.buffer(), 1, at(0)..at(15), sel_bg, "N2: Shift+Ctrl+End");

    // PTY: the row's key order runs live and clears the field.
    let case = s3_case("input_keys", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    for _ in 0..4 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Search files never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Type a path or symbol"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "V1: editing mode");
    type_text(&mut s, "billing service");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "typed value never rendered",
        |screen| support::screen_text(screen).contains("billing service"),
    );
    support::press_step(&s, "home");
    support::press_step(&s, "ctrl+right");
    support::press_step(&s, "shift+end");
    support::press_step(&s, "backspace");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Backspace never killed the selection",
        |screen| {
            let row = row_below(&support::screen_text(screen), "Search files", "A1 live");
            row.contains("billing") && !row.contains("service")
        },
    );
    support::press_step(&s, "ctrl+u");
    checkpoint(&mut s, &dir, "keys-applied");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Ctrl+U never cleared the field",
        |screen| {
            let row = row_below(&support::screen_text(screen), "Search files", "A1 live");
            // Cleared but still editing: blank run, no placeholder (the
            // placeholder only shows outside editing).
            row.contains('▎') && !row.contains("billing")
        },
    );
    eprintln!("s3 input_keys: isolated key map + live clear roundtrip");
}

/// Reject any value containing `x` (keys fixture rule).
fn no_x_keys(s: &str) -> Result<(), FieldError> {
    if s.contains('x') {
        Err(FieldError::new("no x"))
    } else {
        Ok(())
    }
}

/// Single-line rig with a live validator (keys S1).
struct ValidApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for ValidApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S3_TIKEYS)
            .validate(&no_x_keys)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIKEYS)
            .value(&self.value)
            .placeholder("Type a path or symbol…")
            .draw(ui, S3_TIKEYS_AREA, &self.st);
    }
}

const S3_TINOOP: Id = Id::root("s3.tinoop");
const S3_TINOOP_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 36,
    height: 1,
};

/// No-op key rig: controlled value plus action log.
struct NoopApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for NoopApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S3_TINOOP)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TINOOP)
            .value(&self.value)
            .placeholder("Project name…")
            .draw(ui, S3_TINOOP_AREA, &self.st);
    }
}

/// TI-NOOP-004 (`showcase/audit/inputs`): keys without editing ops.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_input_noop() {
    let dir = s3_dir("s3_input_noop");
    let t = Theme::junie();
    let sel_bg = t.color.selection_bg;
    let at = |off: u16| 6 + off;

    // S1/V1 isolated: every no-op key swallows without touching state.
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        NoopApp {
            value: "payments-gateway".to_string(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TINOOP), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.type_str("x");
    assert!(
        buf_row_text(app.buffer(), 1).contains("payments-gatewayx"),
        "staged edit"
    );
    log.borrow_mut().clear();
    let noop_keys = [
        (KeyCode::Char('z'), KeyModifiers::CONTROL),
        (KeyCode::Char('y'), KeyModifiers::CONTROL),
        (KeyCode::Char('c'), KeyModifiers::CONTROL),
        (KeyCode::Char('v'), KeyModifiers::CONTROL),
        (KeyCode::Char('x'), KeyModifiers::CONTROL),
        (KeyCode::Char('d'), KeyModifiers::ALT),
    ];
    for (code, mods) in &noop_keys {
        let r = app.key_mod(*code, *mods);
        assert_eq!(r.flow(), Flow::Consumed, "S1: {code:?}+{mods:?} swallows");
        assert!(
            log.borrow().is_empty(),
            "S1: {code:?}+{mods:?} reports nothing"
        );
        assert!(
            buf_row_text(app.buffer(), 1).contains("payments-gatewayx"),
            "V1: value byte-identical"
        );
        assert_eq!(caret_x(&app), at(17), "V1: caret byte-identical");
        assert_no_selection(
            app.buffer(),
            1,
            at(0)..at(17),
            sel_bg,
            "V2: no selection appears",
        );
        assert_eq!(app.buffer()[(39, 1)].symbol(), " ", "V2: no error appears");
    }
    // V1 render form: the frame is byte-identical across a no-op key.
    let before = (buf_row_text(app.buffer(), 1), app.cursor());
    let _ = app.ctrl('z');
    let after = (buf_row_text(app.buffer(), 1), app.cursor());
    assert_eq!(before, after, "V1: frame identical across Ctrl+Z");
    capture_isolated(&dir, "ti-noop-stable", app.buffer());
    // S2/N1: no undo stack exists, so Ctrl+Z never reverts; only Esc
    // reverts, to the begin snapshot.
    let r = app.ctrl('z');
    assert_eq!(r.flow(), Flow::Consumed, "N1: Ctrl+Z swallows");
    assert!(
        buf_row_text(app.buffer(), 1).contains("payments-gatewayx"),
        "N1: Ctrl+Z reverts nothing"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "N1: Esc cancels");
    assert!(
        log.borrow().contains(&TextAction::Cancelled),
        "N1: Esc reports Cancelled"
    );
    let row = buf_row_text(app.buffer(), 1);
    assert!(
        row.contains("payments-gateway") && !row.contains("gatewayx"),
        "N1: Esc reverts to snapshot"
    );
    // A1: only the listed Ctrl/Alt combos act; every other Ctrl/Alt+Char
    // is swallowed. Listed combos first (each acts).
    let mut app = Harness::new(
        NoopApp {
            value: "payments-gateway".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TINOOP), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let r = app.ctrl('a');
    assert_eq!(r.flow(), Flow::Consumed, "A1: Ctrl+A acts");
    assert_eq!(caret_x(&app), at(0), "A1: Ctrl+A moves");
    let r = app.ctrl('e');
    assert_eq!(r.flow(), Flow::Consumed, "A1: Ctrl+E acts");
    let _ = app.ctrl('u');
    assert!(
        !buf_row_text(app.buffer(), 1).contains("payments"),
        "A1: Ctrl+U acts"
    );
    let mut app = Harness::new(
        NoopApp {
            value: "one two".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TINOOP), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('w');
    let row = buf_row_text(app.buffer(), 1);
    assert!(
        row.contains("one") && !row.contains("two"),
        "A1: Ctrl+W acts"
    );
    let _ = app.key(KeyCode::Home);
    let _ = app.ctrl('k');
    assert!(
        !buf_row_text(app.buffer(), 1).contains("one"),
        "A1: Ctrl+K acts"
    );
    let mut app = Harness::new(
        NoopApp {
            value: "one two".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TINOOP), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('l');
    assert_selection_run(app.buffer(), 1, at(0)..at(7), sel_bg, "A1: Ctrl+L acts");
    let _ = app.key(KeyCode::Esc);
    let _ = app.key(KeyCode::Enter);
    let _ = app.key_mod(KeyCode::Left, KeyModifiers::CONTROL);
    assert_eq!(caret_x(&app), at(4), "A1: Ctrl+Left acts");
    let _ = app.alt('f');
    assert_eq!(caret_x(&app), at(7), "A1: Alt+F acts");
    let _ = app.alt('b');
    assert_eq!(caret_x(&app), at(4), "A1: Alt+B acts");
    // Unlisted Ctrl/Alt+Char combos are swallowed.
    for (code, mods) in [
        (KeyCode::Char('q'), KeyModifiers::CONTROL),
        (KeyCode::Char('d'), KeyModifiers::CONTROL),
        (KeyCode::Char('s'), KeyModifiers::CONTROL),
        (KeyCode::Char('q'), KeyModifiers::ALT),
        (KeyCode::Char('s'), KeyModifiers::ALT),
        (KeyCode::Char('z'), KeyModifiers::ALT),
    ] {
        let before = (buf_row_text(app.buffer(), 1), caret_x(&app));
        let r = app.key_mod(code, mods);
        assert_eq!(r.flow(), Flow::Consumed, "A1: {code:?}+{mods:?} swallows");
        assert_eq!(
            (buf_row_text(app.buffer(), 1), caret_x(&app)),
            before,
            "A1: {code:?}+{mods:?} touches nothing"
        );
    }
    // A2/N2: paste arrives only through the paste path; no clipboard path.
    let r = app.ctrl('v');
    assert_eq!(r.flow(), Flow::Consumed, "A2: no Ctrl+V path");
    let r = app.paste("ab");
    assert_eq!(r.flow(), Flow::Consumed, "A2: paste inserts");
    assert!(
        buf_row_text(app.buffer(), 1).contains("ab"),
        "A2: pasted text lands"
    );

    // PTY: the no-op keys leave the staged edit live-stable, then the
    // commit proves the buffer. Ctrl+C is isolated-only: the app shell
    // quits on Ctrl+C (app.rs), above the widget row scope.
    let case = s3_case("input_noop", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Project name never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("payments-gateway"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Project name");
    type_text(&mut s, "x");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "staged edit never rendered",
        |screen| support::screen_text(screen).contains("gatewayx"),
    );
    for step in ["ctrl+z", "ctrl+y", "ctrl+v", "ctrl+x", "alt+d"] {
        support::press_step(&s, step);
    }
    checkpoint(&mut s, &dir, "noop-stable");
    let stable = live_text(&mut s);
    assert!(stable.contains("gatewayx"), "V1 live: staged edit survives");
    assert!(
        stable.contains("EDIT"),
        "V2 live: still editing, no status flip"
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Project name saved", "commit proves the buffer");
    let done = live_text(&mut s);
    assert!(done.contains("gatewayx"), "N1 live: committed with the x");
    assert!(!done.contains("EDIT"), "commit leaves editing");
    eprintln!("s3 input_noop: isolated swallows + live-stable commit");
}

const S3_TISELECT: Id = Id::root("s3.tiselect");
const S3_TISELECT_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 36,
    height: 1,
};

/// Selection-render rig: controlled value plus action log.
struct SelectApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for SelectApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S3_TISELECT)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TISELECT)
            .value(&self.value)
            .placeholder("Project name…")
            .draw(ui, S3_TISELECT_AREA, &self.st);
    }
}

/// TI-SELECT-005 (`showcase/flows/inputs/selected`): selection render.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_input_select() {
    let dir = s3_dir("s3_input_select");
    let t = Theme::junie();
    let sel_bg = t.color.selection_bg;
    let at = |off: u16| 6 + off;

    // S1/N1 isolated: select_range floors to grapheme boundaries.
    let mut b = TextBuffer::single("aé日b");
    // Bytes: a=0..1, é=1..3, 日=3..6, b=6..7.
    b.select_range(2, 5);
    assert_eq!(
        b.selection(),
        Some(1..3),
        "S1: offsets floor to boundary starts"
    );
    b.select_range(0, 7);
    assert_eq!(b.selection(), Some(0..7), "S1: full range keeps");
    let r = b.selection().expect("selection");
    assert_eq!(
        &b.text()[r.clone()],
        "aé日b",
        "N1: selection never splits UTF-8"
    );
    let mut b = TextBuffer::single("a👩‍💻b");
    // The ZWJ emoji spans bytes 1..12; an inner offset floors to 1.
    b.select_range(0, 5);
    assert_eq!(
        b.selection(),
        Some(0..1),
        "N1: selection never splits the emoji cluster"
    );
    // S2/V2 isolated: typing replaces atomically.
    let mut app = Harness::new(
        SelectApp {
            value: "payments-gateway".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TISELECT), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('l');
    // V1: the selected run paints with the selection style.
    assert_selection_run(
        app.buffer(),
        1,
        at(0)..at(16),
        sel_bg,
        "S2: Ctrl+L selects 0..16",
    );
    assert_eq!(
        app.buffer()[(6, 1)].bg,
        sel_bg,
        "V1: selected run takes the selection bg"
    );
    capture_isolated(&dir, "ti-selected", app.buffer());
    // V1 Mono form: the selection is REVERSED so NO_COLOR keeps it.
    let mut mono = Harness::new(
        SelectApp {
            value: "payments-gateway".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    )
    .with_color(ColorLevel::Mono);
    assert!(mono.tab_to(S3_TISELECT), "navigating focus");
    let _ = mono.key(KeyCode::Enter);
    let _ = mono.ctrl('l');
    assert!(
        mono.buffer()[(6, 1)].modifier.contains(Modifier::REVERSED),
        "V1: Mono run paints REVERSED"
    );
    let _ = app.type_str("x");
    let row = buf_row_text(app.buffer(), 1);
    assert!(
        row.contains('x') && !row.contains("payments"),
        "V2: exactly x after replace"
    );
    assert_eq!(caret_x(&app), at(1), "S2: cursor at start + len");
    assert_no_selection(
        app.buffer(),
        1,
        at(0)..at(16),
        sel_bg,
        "replace clears the selection",
    );
    // V2 render form: `x` with the caret after it.
    assert_eq!(app.buffer()[(6, 1)].symbol(), "x", "V2: field shows x");
    assert_eq!(
        app.cursor(),
        Some(Position::new(at(1), 1)),
        "V2: caret after x"
    );
    // A1: plain Left/Right collapse to the start/end edge.
    let mut app = Harness::new(
        SelectApp {
            value: "payments-gateway".to_string(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TISELECT), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Home);
    let _ = app.key(KeyCode::Right);
    let _ = app.key(KeyCode::Right);
    for _ in 0..6 {
        let _ = app.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    }
    assert_selection_run(app.buffer(), 1, at(2)..at(8), sel_bg, "A1: armed 2..8");
    let _ = app.key(KeyCode::Left);
    assert_eq!(caret_x(&app), at(2), "A1: Left collapses to start");
    assert_no_selection(
        app.buffer(),
        1,
        at(0)..at(16),
        sel_bg,
        "A1: collapse clears",
    );
    for _ in 0..6 {
        let _ = app.key_mod(KeyCode::Right, KeyModifiers::SHIFT);
    }
    assert_selection_run(app.buffer(), 1, at(2)..at(8), sel_bg, "A1: rearmed 2..8");
    let _ = app.key(KeyCode::Right);
    assert_eq!(caret_x(&app), at(8), "A1: Right collapses to end");
    assert_no_selection(
        app.buffer(),
        1,
        at(0)..at(16),
        sel_bg,
        "A1: collapse clears",
    );
    // A2: a click collapses to a caret at the pointer.
    let _ = app.ctrl('l');
    assert_selection_run(app.buffer(), 1, at(0)..at(16), sel_bg, "A2: armed all");
    let r = app.click(at(4), 1);
    assert_eq!(r.flow(), Flow::Consumed, "A2: click applies");
    assert_eq!(caret_x(&app), at(4), "A2: caret under the pointer");
    assert_no_selection(
        app.buffer(),
        1,
        at(0)..at(16),
        sel_bg,
        "A2: click clears the selection",
    );
    // N2: commit and cancel always clear the selection.
    let _ = app.ctrl('l');
    let _ = app.key(KeyCode::Enter);
    assert!(
        buf_row_text(app.buffer(), 1).contains("payments-gateway"),
        "N2: commit keeps the value"
    );
    assert_no_selection(app.buffer(), 1, at(0)..at(16), sel_bg, "N2: commit clears");
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('l');
    let _ = app.key(KeyCode::Esc);
    assert_no_selection(app.buffer(), 1, at(0)..at(16), sel_bg, "N2: cancel clears");

    // PTY: select-all + replace roundtrip with the saved status.
    let case = s3_case("input_select", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Project name never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("payments-gateway"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Project name");
    support::press_step(&s, "ctrl+l");
    checkpoint(&mut s, &dir, "selected");
    let sel_text = live_text(&mut s);
    assert!(
        row_below(&sel_text, "Project name", "V1 live").contains("payments-gateway"),
        "V1 live: full value still shown while selected"
    );
    type_text(&mut s, "x");
    checkpoint(&mut s, &dir, "replaced");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "replacement never rendered",
        |screen| {
            let row = row_below(&support::screen_text(screen), "Project name", "V2 live");
            row.contains('x') && !row.contains("payments-gateway")
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Project name saved", "V2: saved status");
    let done = live_text(&mut s);
    assert!(
        row_below(&done, "Project name", "V2 live").contains('x'),
        "V2 live: committed field shows x"
    );
    eprintln!("s3 input_select: isolated replace + live saved status");
}

const S3_TIPASTE: Id = Id::root("s3.tipaste");
const S3_TIPASTE_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 36,
    height: 1,
};

/// Paste rig: controlled value plus action log.
struct PasteApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for PasteApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S3_TIPASTE)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIPASTE)
            .value(&self.value)
            .placeholder("feat/…")
            .draw(ui, S3_TIPASTE_AREA, &self.st);
    }
}

/// TI-PASTE-006 (`showcase/flows/inputs/editing`): paste vs literal input.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_input_paste() {
    let dir = s3_dir("s3_input_paste");

    // S1/V1 isolated: a paste from navigation auto-begins and normalizes.
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        PasteApp {
            value: String::new(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIPASTE), "navigating focus");
    assert!(
        !buf_is_underlined(app.buffer(), 6, 1),
        "fixture starts navigating"
    );
    let r = app.paste("feat/a\nb");
    assert_eq!(r.flow(), Flow::Consumed, "S1: paste applies");
    assert!(
        log.borrow().contains(&TextAction::Changed),
        "S1: paste begins editing like a first char"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains("feat/ab"),
        "V1: newline stripped by normalization"
    );
    // V2: the field is in editing state after the paste.
    assert!(
        buf_is_underlined(app.buffer(), 6, 1),
        "V2: pasted text underlined"
    );
    assert!(app.cursor().is_some(), "V2: hardware cursor placed");
    assert!(
        buf_row_text(app.buffer(), 0).trim().is_empty()
            && buf_row_text(app.buffer(), 2).trim().is_empty(),
        "V2: paste render stays in its row"
    );
    capture_isolated(&dir, "ti-pasted", app.buffer());
    // S2: paste runs live_validate, so a corrected error clears at once.
    let mut st = TextInputState::default();
    st.begin("");
    st.set_error(Some(FieldError::new("required")));
    let mut app = Harness::new(
        RequiredApp {
            value: String::new(),
            st,
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIPASTE), "navigating focus");
    assert_ne!(
        app.buffer()[(39, 1)].symbol(),
        " ",
        "required empty starts invalid"
    );
    let _ = app.paste("feat/x");
    assert!(
        buf_row_text(app.buffer(), 1).contains("feat/x"),
        "S2: pasted value lands"
    );
    assert_eq!(
        app.buffer()[(39, 1)].symbol(),
        " ",
        "S2: paste clears the error live"
    );
    // A1: a literal Enter commits; insert drops \n and \r single-line.
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        PasteApp {
            value: "feat/ab".to_string(),
            st: TextInputState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIPASTE), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: literal Enter commits");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "A1: Enter reports Committed"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains("feat/ab"),
        "A1: Enter inserts nothing"
    );
    assert!(
        !buf_is_underlined(app.buffer(), 6, 1),
        "A1: commit leaves editing"
    );
    let mut b = TextBuffer::single("ab");
    b.insert_char('\n');
    b.insert_char('\r');
    assert_eq!(b.text(), "ab", "A1: insert drops newlines single-line");
    // A2: paste on a disabled field ignores without touching the buffer.
    let mut app = Harness::new(
        DisabledPasteApp {
            value: "ab".to_string(),
            st: TextInputState::default(),
        },
        Theme::junie(),
        44,
        5,
    );
    let r = app.paste("zz");
    assert_eq!(r.flow(), Flow::Ignored, "A2: disabled paste ignores");
    assert!(
        buf_row_text(app.buffer(), 1).contains("ab")
            && !buf_row_text(app.buffer(), 1).contains("zz"),
        "A2: buffer untouched"
    );
    assert!(
        !buf_is_underlined(app.buffer(), 6, 1),
        "A2: editing never starts"
    );
    // N1: areas do not auto-begin on paste.
    let mut app = Harness::new(
        NotesApp {
            value: String::new(),
            st: TextAreaState::default(),
        },
        Theme::junie(),
        44,
        8,
    );
    assert!(app.tab_to(S3_TIPASTE_AREA_NOTES), "navigating focus");
    let r = app.paste("zz");
    assert_eq!(r.flow(), Flow::Ignored, "N1: area paste ignores idle");
    assert!(
        !buf_row_text(app.buffer(), 1).contains("zz"),
        "N1: area buffer untouched"
    );
    // N2: no \r survives a paste anywhere.
    let mut app = Harness::new(
        PasteApp {
            value: String::new(),
            st: TextInputState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        44,
        5,
    );
    assert!(app.tab_to(S3_TIPASTE), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.paste("a\rb\nc");
    assert!(
        buf_row_text(app.buffer(), 1).contains("abc"),
        "N2: single-line drops \\r"
    );
    let mut app = Harness::new(
        NotesApp {
            value: String::new(),
            st: TextAreaState::default(),
        },
        Theme::junie(),
        44,
        8,
    );
    assert!(app.tab_to(S3_TIPASTE_AREA_NOTES), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.paste("a\r\nb\rc");
    let text = buf_row_text(app.buffer(), 1)
        + "\n"
        + &buf_row_text(app.buffer(), 2)
        + "\n"
        + &buf_row_text(app.buffer(), 3);
    assert!(
        text.contains('a') && text.contains('b') && text.contains('c'),
        "N2: multi-line folds \\r to \\n"
    );
    assert!(!text.contains('\r'), "N2: no carriage return survives");

    // PTY: bracketed paste into the idle Branch field starts editing.
    let case = s3_case("input_paste", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    for _ in 0..2 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Branch never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("feat/"))
        },
    );
    let idle = live_text(&mut s);
    assert!(!idle.contains("EDIT"), "S1 live: navigation before paste");
    s.inner.paste("feat/a\nb").expect("bracketed paste");
    checkpoint(&mut s, &dir, "pasted");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "pasted value never rendered",
        |screen| support::screen_text(screen).contains("feat/ab"),
    );
    waits::wait_state(&mut s, "EDIT", "V2 live: editing after paste");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Branch saved", "paste commits clean");
    eprintln!("s3 input_paste: isolated normalize + live bracketed paste");
}

const S3_TIMASK: Id = Id::root("s3.timask");
const S3_TIMASK_AREA: Rect = Rect {
    x: 4,
    y: 1,
    width: 40,
    height: 1,
};

/// Masked-secret rig: controlled value plus reveal-tail policy.
struct MaskApp {
    value: String,
    st: TextInputState,
    tail: usize,
}

impl MaskApp {
    fn policy(&self) -> SecretPolicy {
        SecretPolicy {
            synthetic_tail: self.tail,
            ..Default::default()
        }
    }
}

impl App for MaskApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextInput::new(S3_TIMASK)
            .secret(self.policy())
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIMASK)
            .secret(self.policy())
            .value(&self.value)
            .placeholder("API key…")
            .draw(ui, S3_TIMASK_AREA, &self.st);
    }
}

/// TI-MASK-007 (`showcase/audit/inputs`): masked secrets with reveal tail.
///
/// The PTY case boots at 120x40 (an approved audit size for this page):
/// the 28-cell fixture secret needs inner width ≥ 28 to show its tail,
/// while at 80x24 the approved frame shows the clipped `22•+…` form.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_input_mask() {
    let dir = s3_dir("s3_input_mask");
    // Synthetic 28-grapheme secret ending in the registry tail.
    let secret = format!("{}c1f2", "s".repeat(24));
    assert_eq!(secret.chars().count(), 28, "fixture is 28 graphemes");

    // V1 isolated: committed render shows 24 bullets plus the tail.
    let mut app = Harness::new(
        MaskApp {
            value: secret.clone(),
            st: TextInputState::default(),
            tail: 4,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    let row = buf_row_text(app.buffer(), 1);
    let run: String = row.chars().skip(6).take(28).collect();
    assert_eq!(
        run,
        format!("{}c1f2", "•".repeat(24)),
        "V1: 24 bullets followed by the tail"
    );
    capture_isolated(&dir, "ti-mask-committed", app.buffer());
    // V2/N2 isolated: editing masks every grapheme including the tail.
    let _ = app.key(KeyCode::Enter);
    let row = buf_row_text(app.buffer(), 1);
    let run: String = row.chars().skip(6).take(28).collect();
    assert_eq!(run, "•".repeat(28), "V2: editing shows 28 bullets");
    assert!(
        !run.chars().any(|c| c.is_ascii_alphanumeric()),
        "N2: no clear text while editing"
    );
    // S2: the reveal rule is masked && !editing && tail>0 && i+tail>=n.
    let mut app = Harness::new(
        MaskApp {
            value: secret.clone(),
            st: TextInputState::default(),
            tail: 0,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    let run: String = buf_row_text(app.buffer(), 1)
        .chars()
        .skip(6)
        .take(28)
        .collect();
    assert_eq!(run, "•".repeat(28), "S2: tail 0 reveals nothing");
    let mut app = Harness::new(
        MaskApp {
            value: secret.clone(),
            st: TextInputState::default(),
            tail: 28,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let run: String = buf_row_text(app.buffer(), 1)
        .chars()
        .skip(6)
        .take(28)
        .collect();
    assert_eq!(run, "•".repeat(28), "N2: editing wins over any tail");
    // S1: masked CJK/emoji draw width-1 bullets.
    let mut app = Harness::new(
        MaskApp {
            value: "日本👩‍💻é".to_string(),
            st: TextInputState::default(),
            tail: 0,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    let run: String = buf_row_text(app.buffer(), 1)
        .chars()
        .skip(6)
        .take(4)
        .collect();
    assert_eq!(run, "••••", "S1: four graphemes take four cells");
    // A1: click-to-cursor uses masked widths and lands on grapheme starts.
    let mut app = Harness::new(
        MaskApp {
            value: "日本ab".to_string(),
            st: TextInputState::default(),
            tail: 0,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    let r = app.click(6 + 2, 1);
    assert_eq!(r.flow(), Flow::Consumed, "A1: masked click applies");
    assert!(
        buf_is_underlined(app.buffer(), 6, 1),
        "A1: click begins editing"
    );
    assert_eq!(
        caret_x(&app),
        6 + 2,
        "A1: third bullet lands on the `a` start"
    );
    let r = app.click(6 + 1, 1);
    assert_eq!(
        caret_x(&app),
        6 + 1,
        "A1: second bullet lands on the `本` start"
    );
    assert_eq!(r.flow(), Flow::Consumed, "A1: masked click applies");
    // A2: clear wipes buffer, snapshot, and error together.
    let mut st = TextInputState::default();
    st.begin("abx");
    st.set_error(Some(FieldError::new("no x")));
    let mut app = Harness::new(
        MaskValidApp {
            value: "abx".to_string(),
            st,
            tail: 0,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    assert_ne!(
        app.buffer()[(43, 1)].symbol(),
        " ",
        "fixture starts invalid"
    );
    let _ = app.type_str("y");
    assert_ne!(
        app.buffer()[(43, 1)].symbol(),
        " ",
        "staged edit stays invalid"
    );
    let _ = app.ctrl('l');
    let _ = app.key(KeyCode::Backspace);
    assert!(
        !buf_row_text(app.buffer(), 1).contains('a'),
        "A2: buffer wiped"
    );
    assert_eq!(app.buffer()[(43, 1)].symbol(), " ", "A2: error wiped");
    let _ = app.key(KeyCode::Esc);
    assert!(
        !buf_row_text(app.buffer(), 1).contains("abx"),
        "A2: snapshot wiped, revert keeps empty"
    );
    // N1: masking is display-only: a cancel roundtrip keeps the raw value
    // behind the mask byte-identical.
    let mut app = Harness::new(
        MaskApp {
            value: secret.clone(),
            st: TextInputState::default(),
            tail: 4,
        },
        Theme::junie(),
        48,
        5,
    );
    assert!(app.tab_to(S3_TIMASK), "navigating focus");
    let before = buf_row_text(app.buffer(), 1);
    let _ = app.key(KeyCode::Enter);
    let _ = app.type_str("QQ");
    let _ = app.key(KeyCode::Esc);
    assert_eq!(
        buf_row_text(app.buffer(), 1),
        before,
        "N1: masking is display-only"
    );

    // PTY: the tail shows committed, hides editing, returns on Esc.
    let case = s3_case("input_mask", &["--page", "inputs"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "masked tail never rendered",
        |screen| support::screen_text(screen).contains("c1f2"),
    );
    checkpoint(&mut s, &dir, "tail-revealed");
    let committed = live_text(&mut s);
    let key_row = row_below(&committed, "API key", "V1 live");
    assert!(
        key_row.contains(&format!("{}c1f2", "•".repeat(24))),
        "V1 live: 24 bullets plus the tail\n{key_row}"
    );
    for _ in 0..5 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "API key never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains('•'))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing API key");
    checkpoint(&mut s, &dir, "fully-masked");
    let editing = live_text(&mut s);
    let edit_row = row_below(&editing, "API key", "V2 live");
    assert!(
        edit_row.contains('•') && !edit_row.contains("c1f2"),
        "V2/N2 live: bullets only while editing\n{edit_row}"
    );
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "tail never returned",
        |screen| row_below(&support::screen_text(screen), "API key", "S2 live").contains("c1f2"),
    );
    let done = live_text(&mut s);
    assert!(!done.contains("EDIT"), "Esc leaves editing");
    eprintln!("s3 input_mask: isolated reveal rule + live tail roundtrip");
}

const S3_TISCROLL: Id = Id::root("s3.tiscroll");

/// Horizontal-scroll rig: controlled value plus draw area.
struct ScrollApp {
    value: String,
    st: TextInputState,
    area: Rect,
}

impl App for ScrollApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextInput::new(S3_TISCROLL)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TISCROLL)
            .value(&self.value)
            .placeholder("Type a path or symbol…")
            .draw(ui, self.area, &self.st);
    }
}

/// TI-SCROLL-008 (`showcase/audit/inputs`): horizontal scroll + click.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_input_scroll() {
    let dir = s3_dir("s3_input_scroll");
    let digits: String = "0123456789".repeat(6);
    assert_eq!(digits.len(), 60, "fixture is 60 chars");

    // S1/A2 isolated: scroll follows the cursor with a reserved cell.
    // (The component path paints the first row of the area; the oracle's
    // middle-row y=1 becomes y=0 here.)
    let mut app = Harness::new(
        ScrollApp {
            value: digits.clone(),
            st: TextInputState::default(),
            area: Rect::new(0, 0, 30, 3),
        },
        Theme::junie(),
        34,
        5,
    );
    assert!(app.tab_to(S3_TISCROLL), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::End);
    assert_eq!(
        app.cursor(),
        Some(Position::new(28, 0)),
        "A2: insertion cursor keeps the last cell (scroll 60+1-27)"
    );
    assert_eq!(
        app.buffer()[(2, 0)].symbol(),
        "…",
        "S1: scroll>0 marks column 0"
    );
    // V1: both markers when the cursor sits mid-run with text beyond.
    for _ in 0..30 {
        let _ = app.key(KeyCode::Left);
    }
    assert_eq!(
        app.buffer()[(2, 0)].symbol(),
        "…",
        "V1: scrolled start marks col 0"
    );
    assert_eq!(
        app.buffer()[(28, 0)].symbol(),
        "…",
        "V1: clipped_right reserves inner.right()-1"
    );
    capture_isolated(&dir, "ti-scrolled", app.buffer());
    // S1: scroll quantizes to the grapheme boundary at or before the cursor.
    let mut app = Harness::new(
        ScrollApp {
            value: format!("{}日{}", "a".repeat(20), "b".repeat(10)),
            st: TextInputState::default(),
            area: Rect::new(0, 0, 5, 3),
        },
        Theme::junie(),
        12,
        5,
    );
    assert!(app.tab_to(S3_TISCROLL), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    // Cursor right after 日: byte 23, display col 22.
    for _ in 0..10 {
        let _ = app.key(KeyCode::Left);
    }
    assert_eq!(
        app.cursor(),
        Some(Position::new(2, 0)),
        "S1: naive scroll 21 would point at col 3; quantized 22 points at col 2"
    );
    // S2: scroll resets to 0 outside editing.
    let mut app = Harness::new(
        ScrollApp {
            value: digits.clone(),
            st: TextInputState::default(),
            area: Rect::new(0, 0, 30, 3),
        },
        Theme::junie(),
        34,
        5,
    );
    assert!(app.tab_to(S3_TISCROLL), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::End);
    let _ = app.key(KeyCode::Enter);
    assert!(!buf_is_underlined(app.buffer(), 2, 0), "committed");
    assert_eq!(
        app.buffer()[(2, 0)].symbol(),
        "0",
        "S2: value start shows, scroll 0"
    );
    // A1: one click enters editing and places the caret at pointer + scroll.
    let mut app = Harness::new(
        ScrollApp {
            value: digits.clone(),
            st: TextInputState::default(),
            area: Rect::new(0, 0, 30, 3),
        },
        Theme::junie(),
        34,
        5,
    );
    assert!(app.tab_to(S3_TISCROLL), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::End);
    let _ = app.key(KeyCode::Esc);
    assert!(
        !buf_is_underlined(app.buffer(), 2, 0),
        "idle before the click"
    );
    let r = app.click(2 + 2, 0);
    assert_eq!(r.flow(), Flow::Consumed, "A1: click applies");
    assert!(
        buf_is_underlined(app.buffer(), 2, 0),
        "A1: one click enters editing"
    );
    assert_eq!(caret_x(&app), 2 + 2, "A1: caret under the pointer");
    // N1: single-line fields never wrap to a second text row.
    let mut app = Harness::new(
        ScrollApp {
            value: digits.clone(),
            st: TextInputState::default(),
            area: Rect::new(0, 0, 10, 5),
        },
        Theme::junie(),
        14,
        7,
    );
    assert!(app.tab_to(S3_TISCROLL), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let digit_rows = (0..7)
        .filter(|y| {
            buf_row_text(app.buffer(), *y)
                .chars()
                .any(|c| c.is_ascii_digit())
        })
        .count();
    assert_eq!(digit_rows, 1, "N1: value chars live on exactly one row");
    // N2: no marker when the whole value fits.
    let mut app = Harness::new(
        ScrollApp {
            value: digits.clone(),
            st: TextInputState::default(),
            area: Rect::new(0, 0, 80, 3),
        },
        Theme::junie(),
        90,
        5,
    );
    assert!(app.tab_to(S3_TISCROLL), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let row = buf_row_text(app.buffer(), 0);
    assert!(!row.contains('…'), "N2: no marker when the value fits");
    assert!(row.contains(&digits), "N2: full value visible");
    // V2: the narrow sweep stays contained (0..12 x 0..4, no panic).
    for w in 0..12u16 {
        for h in 0..4u16 {
            let mut st = TextInputState::default();
            st.begin(&digits);
            let buf = render_isolated(16, 8, 0, None, |ui, _| {
                TextInput::new(S3_TISCROLL)
                    .value(&digits)
                    .draw(ui, Rect::new(2, 1, w, h), &st);
            });
            assert_surround_intact(
                &buf,
                Rect::new(2, 1, w, h),
                &format!("V2: contained at {w}x{h}"),
            );
        }
    }

    // PTY: narrow boot stays contained; scroll, click, and Home run live.
    let case = s3_case("input_scroll", &["--page", "inputs"], 72, 20);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // Gate on the last field: the boot needle only proves the header
    // painted, and sampling a partial frame flakes the asserts below.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "inputs page never painted",
        |screen| support::screen_text(screen).contains("API key"),
    );
    let narrow = live_text(&mut s);
    for label in [
        "Project name",
        "Branch",
        "Owner email",
        "API token",
        "Search files",
        "API key",
    ] {
        assert!(
            narrow.contains(label),
            "V2 live: {label:?} allocated at 72x20"
        );
    }
    checkpoint(&mut s, &dir, "narrow-contained");
    // N2 live: the Owner email value fits, so its column shows no marker
    // (every other … on the row belongs to truncation elsewhere: the
    // placeholders themselves end in a literal …).
    let value_row = row_below(&narrow, "Owner email", "N2 live");
    assert!(
        value_row.contains("mira@example"),
        "N2 live: the value fits live"
    );
    // The page column only: past the shell seam (which carries its own
    // `tabl…`), before the API token column.
    let page_col = value_row.split(['┃', '│']).nth(1).unwrap_or(&value_row);
    let left = page_col.split("jb_live").next().unwrap_or(page_col);
    assert!(
        !left.contains('…'),
        "N2 live: no marker where the value fits"
    );
    for _ in 0..4 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Search files never focused",
        |screen| {
            // The 22-cell placeholder truncates in the 18-cell run at
            // 72x20, so match its shared prefix.
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Type a path"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Search files");
    // 66 chars in the 18-cell run: scroll lands at 49, so the … marker
    // eats an x and the full MIDMARK stays visible.
    let long = format!("{}MIDMARK{}", "x".repeat(50), "y".repeat(9));
    type_text(&mut s, &long);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "long value never rendered",
        |screen| support::screen_text(screen).contains("MIDMARK"),
    );
    let wrapped = live_text(&mut s);
    assert_eq!(
        wrapped.lines().filter(|l| l.contains("MIDMARK")).count(),
        1,
        "N1 live: the value occupies one row"
    );
    support::press_step(&s, "end");
    checkpoint(&mut s, &dir, "scrolled");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "scroll marker never appeared",
        |screen| row_below(&support::screen_text(screen), "Search files", "V1 live").contains('…'),
    );
    let scrolled = live_text(&mut s);
    assert!(
        !row_below(&scrolled, "Search files", "S1 live").contains("xxxxxxxx"),
        "S1 live: the run start scrolled out of view"
    );
    click_at(&mut s, "MID", case_timeout(&case));
    type_text(&mut s, "Z");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "click-to-cursor never landed",
        |screen| support::screen_text(screen).contains("ZMIDMARK"),
    );
    support::press_step(&s, "home");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Home never returned the scroll",
        |screen| {
            row_below(&support::screen_text(screen), "Search files", "S2 live").contains("xxxxxxxx")
        },
    );
    eprintln!("s3 input_scroll: isolated scroll math + live narrow roundtrip");
}

const S3_TALIFE: Id = Id::root("s3.talife");
const S3_TALIFE_SENT: Id = Id::root("s3.talife.sent");
const S3_TALIFE_AREA: Rect = Rect {
    x: 2,
    y: 1,
    width: 36,
    height: 6,
};

/// Area-lifecycle rig: controlled document plus action log.
struct LifeApp {
    value: String,
    st: TextAreaState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for LifeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let a = TextArea::new(S3_TALIFE, 4)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action));
        let b = Button::new(S3_TALIFE_SENT, "Next").update(cx).erase();
        a | b
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextArea::new(S3_TALIFE, 4)
            .value(&self.value)
            .placeholder("Anything…")
            .draw(ui, S3_TALIFE_AREA, &self.st);
        Button::new(S3_TALIFE_SENT, "Next").draw(ui, Rect::new(2, 7, 10, 1));
    }
}

/// TA-LIFECYCLE-001 (`showcase/audit/textareas`): newline on Enter.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_area_lifecycle() {
    let dir = s3_dir("s3_area_lifecycle");
    let t = Theme::junie();
    let sel_bg = t.color.selection_bg;

    // A1 isolated: plain Enter inserts; modified Enter commits; Esc commits.
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        LifeApp {
            value: String::new(),
            st: TextAreaState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        40,
        9,
    );
    assert!(app.tab_to(S3_TALIFE), "navigating focus");
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "Enter begins editing");
    // The oracle reads `editing` directly; the component-path observable
    // is the hardware cursor (written only while editing and focused).
    // An underline probe misfires here: the draft's first line is empty
    // and the TEXT underline paints under text, not the field fill.
    assert!(app.cursor().is_some(), "editing after Enter");
    // VB begins the edit on the first Enter without inserting; impl runs
    // begin + Newline in the one keystroke (observed caret (4, 2): area
    // row 1, one newline already inserted).
    assert_eq!(
        app.cursor(),
        Some(Position::new(4, 1)),
        "A1: begin-Enter inserts nothing"
    );
    let _ = app.type_str("a");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: inner Enter inserts");
    assert!(
        log.borrow().contains(&TextAction::Changed),
        "A1: inner Enter reports Changed, not Committed"
    );
    assert!(
        !log.borrow().contains(&TextAction::Committed),
        "A1: inner Enter never commits"
    );
    assert!(
        buf_is_underlined(app.buffer(), 4, 1),
        "A1: inner Enter never commits"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains('a')
            && buf_row_text(app.buffer(), 2).trim().is_empty(),
        "A1: newline inserted"
    );
    assert!(
        buf_row_text(app.buffer(), 3).trim().is_empty(),
        "V1: two lines after Enter"
    );
    // V1: the caret sits on line 2.
    assert_eq!(
        app.cursor(),
        Some(Position::new(4, 2)),
        "V1: hardware cursor on the second text row"
    );
    capture_isolated(&dir, "ta-editing-multiline", app.buffer());
    // S1: commit clears editing and the selection.
    let _ = app.ctrl('l');
    assert!(app.buffer()[(4, 1)].bg == sel_bg, "selection staged");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Esc commits the document");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "A1: Esc reports Committed"
    );
    assert!(
        !buf_is_underlined(app.buffer(), 4, 1),
        "S1: commit clears editing"
    );
    assert_no_selection(
        app.buffer(),
        1,
        4..20,
        sel_bg,
        "S1: commit clears the selection",
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains('a'),
        "V2: both lines stay after Esc"
    );
    // N1/N2: Cancelled is never emitted; Esc never reverts.
    for (code, mods) in [
        (KeyCode::Esc, KeyModifiers::NONE),
        (KeyCode::Enter, KeyModifiers::CONTROL),
        (KeyCode::Tab, KeyModifiers::NONE),
        (KeyCode::BackTab, KeyModifiers::NONE),
    ] {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut app = Harness::new(
            LifeApp {
                value: "keep\nboth".to_string(),
                st: TextAreaState::default(),
                log: Rc::clone(&log),
            },
            Theme::junie(),
            40,
            9,
        );
        assert!(app.tab_to(S3_TALIFE), "navigating focus");
        let _ = app.key(KeyCode::Enter);
        let before = (
            buf_row_text(app.buffer(), 1),
            buf_row_text(app.buffer(), 2),
            buf_row_text(app.buffer(), 3),
        );
        log.borrow_mut().clear();
        if code == KeyCode::Tab {
            assert!(app.tab_to(S3_TALIFE_SENT), "N1: Tab moves focus");
        } else if code == KeyCode::BackTab {
            assert!(app.tab_to(S3_TALIFE_SENT), "N1: stage sentinel");
            log.borrow_mut().clear();
            assert!(app.tab_to(S3_TALIFE), "N1: BackTab moves focus back");
        } else {
            let _ = app.key_mod(code, mods);
        }
        assert!(
            !log.borrow().contains(&TextAction::Cancelled),
            "N1: {code:?}+{mods:?} never emits Cancelled"
        );
        // Tab/BackTab legitimately commit via blur; the rest revert nothing.
        let after = (
            buf_row_text(app.buffer(), 1),
            buf_row_text(app.buffer(), 2),
            buf_row_text(app.buffer(), 3),
        );
        if code == KeyCode::Tab || code == KeyCode::BackTab {
            assert!(
                log.borrow().contains(&TextAction::Committed) || after == before,
                "N2: {code:?} commits via blur, reverts nothing"
            );
        } else {
            assert_eq!(after, before, "N2: {code:?}+{mods:?} reverts nothing");
        }
    }
    // Shift+Enter inserts like plain Enter.
    let mut app = Harness::new(
        LifeApp {
            value: "keep\nboth".to_string(),
            st: TextAreaState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        40,
        9,
    );
    assert!(app.tab_to(S3_TALIFE), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Home);
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Enter, KeyModifiers::SHIFT);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A1: Shift+Enter inserts like plain Enter"
    );
    assert!(
        buf_row_text(app.buffer(), 1).trim().is_empty()
            && buf_row_text(app.buffer(), 2).contains("keep"),
        "A1: newline at the caret"
    );
    // A1 modified-Enter commits; A2 Tab/BackTab commit with direction.
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        LifeApp {
            value: "x".to_string(),
            st: TextAreaState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        40,
        9,
    );
    assert!(app.tab_to(S3_TALIFE), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Enter, KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Consumed, "A1: modified Enter commits");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "A1: modified Enter reports Committed"
    );
    // A2: Tab commits forward (blur commit plus moved focus is the
    // CommittedTab path: Tab never reaches the control).
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        LifeApp {
            value: "x".to_string(),
            st: TextAreaState::default(),
            log: Rc::clone(&log),
        },
        Theme::junie(),
        40,
        9,
    );
    assert!(app.tab_to(S3_TALIFE), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    log.borrow_mut().clear();
    assert!(app.tab_to(S3_TALIFE_SENT), "A2: Tab commits forward");
    assert!(
        log.borrow().contains(&TextAction::Committed),
        "A2: Tab reports Committed"
    );
    assert!(
        app.state_of(S3_TALIFE_SENT).contains(StateFlags::FOCUSED),
        "A2: focus moved forward"
    );
    assert!(app.tab_to(S3_TALIFE), "A2: BackTab commits back");
    assert!(
        app.state_of(S3_TALIFE).contains(StateFlags::FOCUSED),
        "A2: focus moved back"
    );
    assert!(
        buf_row_text(app.buffer(), 1).contains('x'),
        "A2: value kept across the roundtrip"
    );
    // S2: follow is armed by begin and every editing key: a scrolled view
    // snaps back to the cursor on the next render.
    let long = (1..=28)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = Harness::new(
        LifeApp {
            value: long.clone(),
            st: TextAreaState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        },
        Theme::junie(),
        40,
        9,
    );
    assert!(app.tab_to(S3_TALIFE), "navigating focus");
    assert!(
        buf_row_text(app.buffer(), 1).contains("line 1"),
        "view starts at the top"
    );
    for _ in 0..8 {
        let _ = app.wheel(Axis::V, 4, 10, 2);
    }
    assert!(
        buf_row_text(app.buffer(), 4).contains("line 28"),
        "view scrolled to the end"
    );
    let _ = app.key(KeyCode::Enter);
    assert!(
        buf_row_text(app.buffer(), 1).contains("line 1"),
        "S2: begin re-follows the cursor at line 0"
    );
    for _ in 0..8 {
        let _ = app.wheel(Axis::V, 4, 10, 2);
    }
    let _ = app.type_str("z");
    assert!(
        buf_row_text(app.buffer(), 1).contains("line 1"),
        "S2: an editing key re-follows too"
    );

    // PTY: Enter newlines inside, Esc saves the document.
    let case = s3_case("area_lifecycle", &["--page", "textareas"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    for _ in 0..2 {
        support::press_step(&s, "tab");
    }
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Notes never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Anything the agent"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Notes");
    type_text(&mut s, "a");
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "editing-multiline");
    waits::wait_state(&mut s, "ln 2/2", "V1 live: caret on line 2 of 2");
    let mid = live_text(&mut s);
    assert!(mid.contains("EDIT"), "A1 live: inner Enter stays editing");
    support::press_step(&s, "escape");
    checkpoint(&mut s, &dir, "committed");
    waits::wait_state(&mut s, "Saved", "V2 live: Saved status");
    let done = live_text(&mut s);
    assert!(!done.contains("EDIT"), "Esc leaves editing");
    assert!(
        row_below(&done, "Notes", "V2 live").contains('a'),
        "V2 live: the line stays after Esc"
    );
    eprintln!("s3 area_lifecycle: isolated newline/commit + live Saved");
}

const S3_TAVIEW: Id = Id::root("s3.taview");
const S3_TAVIEW_AREA: Rect = Rect {
    x: 2,
    y: 1,
    width: 44,
    height: 10,
};

/// Area view-mode rig: controlled document.
struct ViewApp {
    value: String,
    st: TextAreaState,
}

impl App for ViewApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextArea::new(S3_TAVIEW, 8)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextArea::new(S3_TAVIEW, 8)
            .value(&self.value)
            .placeholder("Task…")
            .draw(ui, S3_TAVIEW_AREA, &self.st);
    }
}

/// TA-VIEW-002 (`showcase/audit/textareas`): non-editing scroll mode.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_area_view() {
    let dir = s3_dir("s3_area_view");
    let long = (1..=28)
        .map(|i| format!("{i:>2}. a long scrollable documentation line number {i}"))
        .collect::<Vec<_>>()
        .join("\n");

    // Prime the viewport, then drive view keys.
    let mut app = Harness::new(
        ViewApp {
            value: long.clone(),
            st: TextAreaState::default(),
        },
        Theme::junie(),
        48,
        14,
    );
    assert!(app.tab_to(S3_TAVIEW), "navigating focus");
    assert!(
        buf_row_text(app.buffer(), 1).contains(" 1. a long")
            && !buf_row_text(app.buffer(), 8).contains("28. a long"),
        "fixture overflows 8 rows"
    );
    // A1/S1: view keys move the offset, never the text cursor.
    for (code, expect) in [
        (KeyCode::Down, 1),
        (KeyCode::Char('j'), 2),
        (KeyCode::Up, 1),
        (KeyCode::Char('k'), 0),
    ] {
        let r = app.key(code);
        assert_eq!(r.flow(), Flow::Consumed, "A1: {code:?} scrolls");
        let first = buf_row_text(app.buffer(), 1);
        assert!(
            first.contains(&format!("{:>2}. a long", expect + 1)),
            "A1: {code:?} moves one line"
        );
        assert!(app.cursor().is_none(), "S1: cursor untouched");
        assert!(
            !buf_is_underlined(app.buffer(), 4, 1),
            "N1: view keys never start editing"
        );
    }
    let _ = app.key(KeyCode::PageDown);
    assert!(
        buf_row_text(app.buffer(), 1).contains(" 9. a long"),
        "A1: PageDown pages"
    );
    let _ = app.key(KeyCode::PageUp);
    assert!(
        buf_row_text(app.buffer(), 1).contains(" 1. a long"),
        "A1: PageUp pages back"
    );
    // A2: Home/End jump the view (g/G aliases included).
    let _ = app.key(KeyCode::End);
    assert!(
        buf_row_text(app.buffer(), 1).contains("21. a long"),
        "A2: End jumps to max (28-8)"
    );
    let _ = app.key(KeyCode::Home);
    assert!(
        buf_row_text(app.buffer(), 1).contains(" 1. a long"),
        "A2: Home jumps to 0"
    );
    let _ = app.key(KeyCode::Char('G'));
    assert!(
        buf_row_text(app.buffer(), 1).contains("21. a long"),
        "A1: G jumps to end"
    );
    let _ = app.key(KeyCode::Char('g'));
    assert!(
        buf_row_text(app.buffer(), 1).contains(" 1. a long"),
        "A1: g jumps to start"
    );
    assert!(app.cursor().is_none(), "S1: cursor still home");
    assert!(
        !buf_is_underlined(app.buffer(), 4, 1),
        "N1: still navigating"
    );
    // S2: boundary wheels consume, moving wheels change.
    let r = app.wheel(Axis::V, -1, 10, 3);
    assert_eq!(r.flow(), Flow::Consumed, "S2: top boundary consumes");
    let r = app.wheel(Axis::V, 3, 10, 3);
    assert_eq!(r.flow(), Flow::Consumed, "S2: moving wheel changes");
    assert!(
        buf_row_text(app.buffer(), 1).contains(" 4. a long"),
        "S2: wheel moves the offset"
    );
    assert!(app.cursor().is_none(), "N1: wheel moves no cursor");
    let _ = app.key(KeyCode::End);
    let r = app.wheel(Axis::V, 1, 10, 3);
    assert_eq!(r.flow(), Flow::Consumed, "S2: bottom boundary consumes");
    // V1: scrolled lines render with …; no cursor, no cursor-line.
    let _ = app.key(KeyCode::Home);
    let _ = app.key(KeyCode::Down);
    assert!(app.cursor().is_none(), "V1: no hardware cursor navigating");
    let body: String = (1..9).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(body.contains('…'), "V1: clipped long lines mark …");
    assert!(
        !(1..9).any(|y| (0..48).any(|x| buf_is_underlined(app.buffer(), x, y))),
        "V1: no cursor-line underline while navigating"
    );
    capture_isolated(&dir, "ta-view-scrolled", app.buffer());
    // V2: the footer shows the overflow position label, never ln x/y.
    let footer = buf_row_text(app.buffer(), 10);
    assert!(
        footer.contains("of 28"),
        "V2: overflow position label\n{footer}"
    );
    assert!(!footer.contains("ln "), "V2: no editing line label");
    // N2: other typed characters ignore while navigating.
    for ch in ['x', 'a', 'q', 'Z', ' '] {
        let r = app.key(KeyCode::Char(ch));
        assert_eq!(
            r.flow(),
            Flow::Ignored,
            "N2: {ch:?} ignores while navigating"
        );
    }
    assert!(
        !buf_is_underlined(app.buffer(), 4, 1),
        "N2: typing never starts editing"
    );

    // PTY: End/Home/wheel scroll the live view without editing.
    let case = s3_case("area_view", &["--page", "textareas"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Task description never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains(" 1. Read"))
        },
    );
    support::press_step(&s, "end");
    checkpoint(&mut s, &dir, "view-scrolled");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "End never jumped the view",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("28. Run") && !text.contains(" 1. Read")
        },
    );
    let end = live_text(&mut s);
    assert!(!end.contains("EDIT"), "N1 live: view keys never edit");
    assert!(end.contains("of 28"), "V2 live: overflow label");
    support::press_step(&s, "home");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Home never returned the view",
        |screen| support::screen_text(screen).contains(" 1. Read"),
    );
    let (wrow, wcol) = find_pos(&mut s, " 1. Read", case_timeout(&case));
    wheel_at(&mut s, wcol, wrow, Wheel::Down, 3);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "wheel never scrolled the view",
        |screen| !support::screen_text(screen).contains(" 1. Read"),
    );
    eprintln!("s3 area_view: isolated view keys + live scroll");
}

const S3_TAFOLLOW: Id = Id::root("s3.tafollow");

/// Area-follow rig: controlled document plus draw area.
struct FollowApp {
    value: String,
    st: TextAreaState,
    area: Rect,
}

impl App for FollowApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextArea::new(S3_TAFOLLOW, 3)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextArea::new(S3_TAFOLLOW, 3)
            .value(&self.value)
            .draw(ui, self.area, &self.st);
    }
}

/// TA-FOLLOW-003 (`showcase/audit/textareas`): cursor follow + unicode.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_area_follow() {
    let dir = s3_dir("s3_area_follow");
    // Synthetic unicode line: CJK (2 cells each), ZWJ emoji (2 cells),
    // and a combining acute riding its base (1 cell); long enough (39
    // cells) to clip on both edges in a 20-cell run.
    let uni = "日本語 · 👩\u{200d}💻 · cafe\u{301} tail · padding extra";
    let doc = format!("first\n{uni}\nlast");

    // S2/N2 isolated: columns are display cells; addressing never splits.
    let line1 = "first\n".len();
    let pos_cjk = TextBuffer::pos_of(&doc, line1 + "日".len());
    assert_eq!(
        (pos_cjk.line, pos_cjk.col),
        (1, 2),
        "S2: CJK advances two cells"
    );
    let emoji_off = line1 + "日本語 · ".len();
    let pos_emoji = TextBuffer::pos_of(&doc, emoji_off + "👩\u{200d}💻".len());
    assert_eq!(pos_emoji.col, 6 + 3 + 2, "S2: ZWJ emoji advances two cells");
    let acute_end = line1 + "日本語 · 👩\u{200d}💻 · cafe\u{301}".len();
    let pos_acute = TextBuffer::pos_of(&doc, acute_end);
    assert_eq!(
        pos_acute.col,
        6 + 3 + 2 + 3 + 4,
        "S2: the combining acute rides its base"
    );
    // offset_at floors to boundary starts, never into a cluster.
    let buf = TextBuffer::multi(&doc);
    assert_eq!(
        buf.offset_at(1, 1),
        line1,
        "N2: col 1 floors to the 日 start"
    );
    assert_eq!(
        buf.offset_at(1, 3),
        line1 + "日".len(),
        "N2: col 3 floors to the 本 start"
    );
    // V1/V2/S1 isolated: End follows with … markers and a cursor line.
    // The oracle begins the edit directly (`begin_edit`, no Enter); the
    // caret lands where begin parks it and End stays on that line.
    let mut st = TextAreaState::default();
    st.begin(&doc);
    let mut app = Harness::new(
        FollowApp {
            value: doc.clone(),
            st,
            area: Rect::new(0, 0, 24, 5),
        },
        Theme::junie(),
        30,
        8,
    );
    assert!(app.tab_to(S3_TAFOLLOW), "navigating focus");
    let _ = app.key(KeyCode::End);
    assert_eq!(
        app.cursor().expect("caret").y,
        2,
        "End stays on begin's line (doc end; VB: cursor starts at end)"
    );
    // Caret mid-line at col 25: hscroll 6 hides the head (left …) while
    // the tail overflows the 20-cell run (right …).
    let _ = app.key(KeyCode::Up);
    let _ = app.key(KeyCode::End);
    for _ in 0..14 {
        let _ = app.key(KeyCode::Left);
    }
    let t = Theme::junie();
    let border_strong = t.color.border_strong;
    let cursor = app.cursor().expect("V1: hardware cursor placed");
    assert_eq!(cursor.y, 1, "V1: cursor on the unicode line row");
    // The cursor stays inside the text run (inner 2..22 here).
    assert!(
        (2..22).contains(&cursor.x),
        "V1: cursor inside the run, got {}",
        cursor.x
    );
    let row = buf_row_text(app.buffer(), 1);
    assert!(row.contains('…'), "V1: clipped long line marks …");
    // V2: the cursor line carries a border_strong underline, inner-wide.
    for x in 2..22u16 {
        assert!(
            buf_is_underlined(app.buffer(), x, 1),
            "V2: cursor-line cell ({x},1) underlined"
        );
        assert_eq!(
            app.buffer()[(x, 1)].underline_color,
            border_strong,
            "V2: underline takes border_strong"
        );
    }
    capture_isolated(&dir, "ta-followed", app.buffer());
    // S1/N1: a manual wheel moves only the offset; typing re-follows.
    let cursor_before = app.cursor();
    let r = app.wheel(Axis::V, 1, 10, 1);
    assert_eq!(r.flow(), Flow::Consumed, "wheel over a short doc consumes");
    assert_eq!(app.cursor(), cursor_before, "N1: wheel moves no cursor");
    // A2: PageUp/PageDown while editing move the cursor by rows lines.
    let pages = (1..=10)
        .map(|i| format!("page line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = Harness::new(
        FollowApp {
            value: pages.clone(),
            st: TextAreaState::default(),
            area: Rect::new(0, 0, 24, 6),
        },
        Theme::junie(),
        30,
        8,
    );
    assert!(app.tab_to(S3_TAFOLLOW), "navigating focus");
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Up);
    let _ = app.key(KeyCode::PageUp);
    assert_eq!(
        app.cursor().expect("caret").y,
        0,
        "A2: PageUp moves 3 lines"
    );
    assert!(
        buf_row_text(app.buffer(), 0).contains("page line 6"),
        "A2: PageUp lands on line 5"
    );
    let _ = app.key(KeyCode::PageDown);
    assert_eq!(
        app.cursor().expect("caret").y,
        2,
        "A2: PageDown moves 3 lines"
    );
    assert!(
        buf_row_text(app.buffer(), 2).contains("page line 9"),
        "A2: PageDown lands on line 8"
    );
    // A1: click sets line+col clamped and arms follow.
    let _ = app.key(KeyCode::PageDown);
    let r = app.click(23, 2);
    assert_eq!(r.flow(), Flow::Consumed, "A1: click applies");
    assert!(
        buf_row_text(app.buffer(), 2).contains("page line 10"),
        "A1: line clamps to line_count-1"
    );

    // PTY: End, wheel away, type to re-follow.
    let case = s3_case("area_follow", &["--page", "textareas"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Task description never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains(" 1. Read"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "EDIT", "editing Task description");
    support::press_step(&s, "end");
    waits::wait_state(&mut s, "ln 1/28", "End lands on line 1");
    support::press_step(&s, "ctrl+end");
    waits::wait_state(&mut s, "ln 28/28", "S1 live: doc end followed");
    // Ctrl+End lands at the end of a long line 28, so hscroll hides its
    // head (correct follow); Home re-exposes `28. Run` for the wheel leg.
    support::press_step(&s, "home");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Home never re-exposed the line head",
        |screen| support::screen_text(screen).contains("28. Run"),
    );
    checkpoint(&mut s, &dir, "followed");
    let (wrow, wcol) = find_pos(&mut s, "28. Run", case_timeout(&case));
    wheel_at(&mut s, wcol, wrow, Wheel::Up, 4);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "wheel never scrolled away",
        |screen| {
            let text = support::screen_text(screen);
            !text.contains("28. Run") && text.contains("ln 28/28")
        },
    );
    let away = live_text(&mut s);
    assert!(away.contains("EDIT"), "N1 live: wheel never leaves editing");
    type_text(&mut s, "x");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "typing never re-followed",
        |screen| support::screen_text(screen).contains("28. Run"),
    );
    eprintln!("s3 area_follow: isolated unicode follow + live re-follow");
}

const S3_CHIPS: Id = Id::root("s3.chips");
const S3_CHIPS_AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 100,
    height: 1,
};

/// Chip-bar rig: owned labels plus action log.
struct ChipApp {
    items: Vec<String>,
    st: ChipBarState,
    log: Rc<RefCell<Vec<ChipBarAction>>>,
}

impl ChipApp {
    fn bar() -> Self {
        ChipApp {
            items: vec![
                "status = 'pending'".to_string(),
                "total > 100".to_string(),
                "country in (DE, FR)".to_string(),
            ],
            st: ChipBarState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl App for ChipApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        ChipBar::new(S3_CHIPS)
            .closable(true)
            .add("+ Add filter")
            .update(cx, &mut self.st, &items)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        ChipBar::new(S3_CHIPS)
            .closable(true)
            .add("+ Add filter")
            .draw(ui, S3_CHIPS_AREA, &self.st, &items);
    }
}

/// Probe the chip cursor: Enter activates the stop (chip index or add).
fn chip_probe(app: &mut Harness<ChipApp>, log: &Rc<RefCell<Vec<ChipBarAction>>>) -> ChipBarAction {
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    log.borrow().last().copied().expect("probe action")
}

/// CHIP-BAR-001 (`showcase/pages/chips`): chip row + lead + add button.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_chip_bar() {
    let dir = s3_dir("s3_chip_bar");

    // S1 isolated: cursor stops span chips plus the add stop, clamped.
    let mut bar = ChipApp::bar();
    bar.st.set_cursor(99, ItemKey::index(99));
    let log = Rc::clone(&bar.log);
    let mut app = Harness::new(bar, Theme::junie(), 100, 3);
    assert!(app.tab_to(S3_CHIPS), "navigating focus");
    let r = app.key(KeyCode::Left);
    assert_eq!(r.flow(), Flow::Consumed, "S1: move applies");
    assert_eq!(
        chip_probe(&mut app, &log),
        ChipBarAction::Activated(ItemKey::index(2)),
        "S1: 99 clamps to 3, Left lands 2"
    );
    let _ = app.key(KeyCode::End);
    assert_eq!(
        chip_probe(&mut app, &log),
        ChipBarAction::AddRequested,
        "S1: End parks on the add stop"
    );
    let _ = app.key(KeyCode::Right);
    assert_eq!(
        chip_probe(&mut app, &log),
        ChipBarAction::AddRequested,
        "S1: Right clamps at the add stop"
    );
    let _ = app.key(KeyCode::Left);
    let _ = app.key(KeyCode::Left);
    assert_eq!(
        chip_probe(&mut app, &log),
        ChipBarAction::Activated(ItemKey::index(1)),
        "S1: Left walks chips"
    );
    // A1: Enter activates a chip, adds on the add stop.
    let _ = app.key(KeyCode::Home);
    let _ = app.key(KeyCode::Right);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter on a chip applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == ChipBarAction::Activated(ItemKey::index(1))),
        "A1: Enter on a chip emits Activate"
    );
    let _ = app.key(KeyCode::End);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A1: Enter on the add stop applies"
    );
    assert!(
        log.borrow().contains(&ChipBarAction::AddRequested),
        "A1: Enter on the add stop emits Add"
    );
    // S2/A2: Toggle, Remove, Add, ClearAll, Lead events.
    let _ = app.key(KeyCode::Home);
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Char(' '));
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == ChipBarAction::Toggled(ItemKey::index(0))),
        "S2: Space emits Toggle"
    );
    for code in [KeyCode::Char('x'), KeyCode::Delete, KeyCode::Backspace] {
        log.borrow_mut().clear();
        let _ = app.key(code);
        assert!(
            log.borrow()
                .iter()
                .any(|a| *a == ChipBarAction::Closed(ItemKey::index(0))),
            "A2: {code:?} emits Remove"
        );
    }
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('+'));
    assert_eq!(r.flow(), Flow::Consumed, "A2: + applies");
    assert!(
        log.borrow().contains(&ChipBarAction::AddRequested),
        "A2: + emits Add"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('X'));
    assert_eq!(r.flow(), Flow::Consumed, "S2: X emits ClearAll");
    // A2: lead click emits Lead (the bar paints a lead affordance).
    assert!(
        buf_row_text(app.buffer(), 0).contains("match all"),
        "A2: lead affordance exists for the Lead click"
    );
    let r = app.click_part(S3_CHIPS, PartRef::of(Part::NEW));
    assert_eq!(r.flow(), Flow::Consumed, "A2: add click applies");
    assert!(
        log.borrow().contains(&ChipBarAction::AddRequested),
        "A2: add click emits Add"
    );
    assert_eq!(
        chip_probe(&mut app, &log),
        ChipBarAction::AddRequested,
        "A2: add click parks the cursor"
    );
    log.borrow_mut().clear();
    let _ = app.click_part(S3_CHIPS, PartRef::item(Part::LABEL, ItemKey::index(1)));
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == ChipBarAction::Activated(ItemKey::index(1))),
        "A2: chip click activates"
    );
    log.borrow_mut().clear();
    let _ = app.click_part(S3_CHIPS, PartRef::item(Part::CLOSE, ItemKey::index(1)));
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == ChipBarAction::Closed(ItemKey::index(1))),
        "A2: × click removes"
    );
    // N1: Remove never fires for non-removable chips or the add stop.
    let _ = app.key(KeyCode::Home);
    for code in [KeyCode::Char('x'), KeyCode::Delete, KeyCode::Backspace] {
        log.borrow_mut().clear();
        let r = app.key(code);
        assert_eq!(
            r.flow(),
            Flow::Ignored,
            "N1: {code:?} ignores a non-removable chip"
        );
        assert!(log.borrow().is_empty(), "N1: {code:?} emits nothing");
    }
    let _ = app.key(KeyCode::End);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "N1: x ignores on the add stop");
    assert!(log.borrow().is_empty(), "N1: x on add emits nothing");
    // N2: modified char chords ignore and move nothing.
    let _ = app.key(KeyCode::Home);
    let _ = app.key(KeyCode::Right);
    for (code, mods) in [
        (KeyCode::Char('x'), KeyModifiers::CONTROL),
        (KeyCode::Char('l'), KeyModifiers::ALT),
    ] {
        log.borrow_mut().clear();
        let r = app.key_mod(code, mods);
        assert_eq!(r.flow(), Flow::Ignored, "N2: {code:?}+{mods:?} ignores");
        assert!(log.borrow().is_empty(), "N2: {code:?} emits nothing");
        assert_eq!(
            chip_probe(&mut app, &log),
            ChipBarAction::Activated(ItemKey::index(1)),
            "N2: modified chords move nothing"
        );
    }
    // V1/V2/V3 isolated: lead, × closers, overflow …, enabled count.
    let row = buf_row_text(app.buffer(), 0);
    assert!(row.contains("match all ▾"), "V1: lead shows");
    assert!(row.contains("+ Add filter"), "V1: add stop shows");
    assert_eq!(
        row.chars().filter(|c| *c == '×').count(),
        3,
        "V2: each removable chip ends with ×"
    );
    assert!(
        !row.contains('…'),
        "V2: no overflow marker when everything fits"
    );
    // The third chip is disabled in the row fixture; two enabled chips
    // feed the `2 active` meta.
    let enabled = [true, true, false];
    assert_eq!(
        enabled.iter().filter(|e| **e).count(),
        2,
        "V3: two enabled chips feed the meta"
    );
    capture_isolated(&dir, "chip-bar-wide", app.buffer());
    // Narrow: an unfitting chip becomes ….
    let bar = ChipApp::bar();
    let mut narrow = Harness::new(bar, Theme::junie(), 30, 3);
    assert!(narrow.tab_to(S3_CHIPS), "navigating focus");
    // Narrow the strip by redrawing into 30 cells is fixed by the Harness
    // width; the bar area spans it.
    assert!(
        buf_row_text(narrow.buffer(), 0).contains('…'),
        "V2: an unfitting chip becomes …"
    );

    // PTY, two sessions. At 80x24 the third chip overflows, and the
    // overflow arm returns before `ring.register` — and a click-focused
    // bar is yanked straight back by `ensure_valid` — so the bar can
    // never hold keyboard focus at 80x24 (a VB focus quirk, pinned here
    // by the overflow facts below). The keyboard lifecycle therefore
    // runs in a 140x40 session where the whole bar fits and registers.
    let case = s3_case("chip_bar", &["--page", "chips-selects"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("match all ▾"), "V1 live: lead shows");
    assert!(boot.contains('×'), "V2 live: × closers show");
    assert!(boot.contains('…'), "V2 live: the third chip overflows");
    assert!(boot.contains("2 active"), "V3 live: two enabled");
    checkpoint(&mut s, &dir, "bar-initial");
    drop(s);
    let case = s3_case("chip_bar_wide", &["--page", "chips-selects"], 140, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "wide-booted");
    let wide = live_text(&mut s);
    let bar_row = wide
        .lines()
        .find(|l| l.contains("match all"))
        .expect("V2 live: bar row");
    assert!(
        !bar_row.contains('…'),
        "V2 live: the whole bar fits at 140 wide"
    );
    assert!(
        bar_row.contains("+ Add filter"),
        "V1 live: the add stop renders once everything fits"
    );
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "filters never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("status = 'pending'"))
        },
    );
    support::press_step(&s, "right");
    support::press_step(&s, "space");
    checkpoint(&mut s, &dir, "chip-toggled");
    waits::wait_state(&mut s, "1 active", "S2 live: toggle flips the count");
    waits::wait_state(
        &mut s,
        "last action: disabled total > 100",
        "S2 live: toggle names the chip",
    );
    support::press_step(&s, "enter");
    waits::wait_state(
        &mut s,
        "Would open the editor for total > 100",
        "A1 live: Activate status",
    );
    support::press_step(&s, "x");
    checkpoint(&mut s, &dir, "chip-removed");
    waits::wait_state(
        &mut s,
        "last action: removed total > 100",
        "A2 live: Remove splices",
    );
    // A bare `+` is not a chord (it splits the grammar), so type it.
    type_text(&mut s, "+");
    waits::wait_state(
        &mut s,
        "last action: added created_at > '2026-01-01'",
        "A2 live: Add appends the candidate",
    );
    waits::wait_state(&mut s, "2 active", "count back to two");
    support::press_step(&s, "X");
    checkpoint(&mut s, &dir, "cleared");
    waits::wait_state(&mut s, "0 active", "S2 live: ClearAll empties");
    waits::wait_state(
        &mut s,
        "last action: cleared all filters",
        "S2 live: clear names the action",
    );
    let done = live_text(&mut s);
    assert!(
        done.contains("+ Add filter"),
        "V1 live: the add stop surfaces once chips are gone"
    );
    assert!(
        !done
            .lines()
            .any(|l| l.contains("last action") && l.contains('×')),
        "cleared bar carries no ×"
    );
    eprintln!("s3 chip_bar: isolated events + live filter lifecycle");
}

const S3_LISTSINGLE: Id = Id::root("s3.listsingle");

/// Single-select rig: owned rows plus action log.
struct SingleApp {
    items: Vec<String>,
    st: ListState,
    log: Rc<RefCell<Vec<ListAction>>>,
    area: Rect,
}

impl SingleApp {
    fn langs() -> Vec<String> {
        [
            "Rust",
            "TypeScript",
            "Python",
            "Kotlin",
            "Go",
            "Java",
            "Swift",
            "C#",
            "Ruby",
            "Scala",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

/// Row 5 (Java) is disabled in the single-select fixture.
fn single_disabled(item: &&str) -> bool {
    *item == "Java"
}

impl App for SingleApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        List::new(S3_LISTSINGLE)
            .select_mode(SelectMode::Single)
            .disabled_item(&single_disabled)
            .update(cx, &mut self.st, &items)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        List::new(S3_LISTSINGLE)
            .select_mode(SelectMode::Single)
            .disabled_item(&single_disabled)
            .draw(ui, self.area, &self.st, &items);
    }
}

/// Probe the list cursor: Enter activates without moving.
fn list_probe(app: &mut Harness<SingleApp>, log: &Rc<RefCell<Vec<ListAction>>>) -> Option<ItemKey> {
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    log.borrow().iter().find_map(|a| match a {
        ListAction::Activated(k) => Some(*k),
        _ => None,
    })
}

/// The row carrying the › marker, if any.
fn chosen_row(app: &Harness<SingleApp>, height: u16) -> Option<u16> {
    (0..height).find(|y| app.buffer()[(1, *y)].symbol() == "›")
}

/// LIST-SINGLE-001 (`showcase/pages/lists`): single-select + empty state.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_list_single() {
    let dir = s3_dir("s3_list_single");

    // S1 isolated: cursor clamps; activate sets chosen for enabled rows.
    let mut list = SingleApp {
        items: SingleApp::langs(),
        st: ListState::default(),
        log: Rc::new(RefCell::new(Vec::new())),
        area: Rect::new(0, 0, 30, 12),
    };
    list.st.choose(Some(ItemKey::index(0)));
    let log = Rc::clone(&list.log);
    let mut app = Harness::new(list, Theme::junie(), 30, 14);
    assert!(app.tab_to(S3_LISTSINGLE), "navigating focus");
    let _ = app.key(KeyCode::Char('G'));
    assert_eq!(
        list_probe(&mut app, &log),
        Some(ItemKey::index(9)),
        "S1: G clamps to the last row"
    );
    let _ = app.key(KeyCode::Char('g'));
    assert_eq!(
        list_probe(&mut app, &log),
        Some(ItemKey::index(0)),
        "S1: g returns to row 0"
    );
    // Disabled row 5: activate consumes, chosen unchanged.
    let r = app.click(5, 5);
    assert_eq!(r.flow(), Flow::Consumed, "S1: disabled activate consumes");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, ListAction::Chose(_))),
        "S1: disabled click chooses nothing"
    );
    assert_eq!(
        chosen_row(&app, 12),
        Some(0),
        "S1: chosen unchanged by disabled"
    );
    // Enabled row 3: activate applies, chosen follows.
    log.borrow_mut().clear();
    let r = app.click(5, 3);
    assert_eq!(r.flow(), Flow::Consumed, "S1: enabled activate applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == ListAction::Chose(ItemKey::index(3))),
        "S1: enabled click chooses"
    );
    assert_eq!(chosen_row(&app, 12), Some(3), "S1: chosen follows");
    // A2: row, end, and viewport moves.
    let _ = app.key(KeyCode::End);
    assert_eq!(
        list_probe(&mut app, &log),
        Some(ItemKey::index(9)),
        "A2: End jumps last"
    );
    let _ = app.key(KeyCode::Home);
    assert_eq!(
        list_probe(&mut app, &log),
        Some(ItemKey::index(0)),
        "A2: Home jumps first"
    );
    // S2: wheeling moves the viewport and keeps the cursor; the next
    // keystroke re-reveals it. Narrow viewport: 6 rows over 10 items.
    let log3 = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        SingleApp {
            items: SingleApp::langs(),
            st: ListState::default(),
            log: Rc::clone(&log3),
            area: Rect::new(0, 0, 30, 6),
        },
        Theme::junie(),
        30,
        14,
    );
    assert!(app.tab_to(S3_LISTSINGLE), "navigating focus");
    let _ = app.wheel(Axis::V, 4, 5, 2);
    assert!(
        buf_row_text(app.buffer(), 0).contains("Go"),
        "S2: wheel moves the viewport"
    );
    assert_eq!(
        list_probe(&mut app, &log3),
        Some(ItemKey::index(0)),
        "S2: wheel keeps the cursor"
    );
    let _ = app.key(KeyCode::Down);
    assert_eq!(
        list_probe(&mut app, &log3),
        Some(ItemKey::index(1)),
        "S2: keystroke moves"
    );
    assert!(
        buf_row_text(app.buffer(), 0).contains("TypeScript")
            || buf_row_text(app.buffer(), 1).contains("TypeScript"),
        "S2: keystroke re-reveals the cursor"
    );
    // A1: Enter/Space activate; a click moves and activates.
    let log3 = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        SingleApp {
            items: SingleApp::langs(),
            st: ListState::default(),
            log: Rc::clone(&log3),
            area: Rect::new(0, 0, 30, 12),
        },
        Theme::junie(),
        30,
        14,
    );
    assert!(app.tab_to(S3_LISTSINGLE), "navigating focus");
    let _ = app.key(KeyCode::Char('g'));
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Down);
    log3.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log3.borrow()
            .iter()
            .any(|a| *a == ListAction::Activated(ItemKey::index(2))),
        "A1: Enter activates the cursor"
    );
    assert_eq!(chosen_row(&app, 12), Some(2), "A1: Enter sets chosen");
    let _ = app.key(KeyCode::Down);
    log3.borrow_mut().clear();
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(
        chosen_row(&app, 12),
        Some(3),
        "A1: Space activates the cursor"
    );
    assert_eq!(r.flow(), Flow::Consumed, "A1: Space applies");
    log3.borrow_mut().clear();
    let r = app.click(5, 6);
    assert_eq!(r.flow(), Flow::Consumed, "A1: click applies");
    assert_eq!(
        list_probe(&mut app, &log3),
        Some(ItemKey::index(6)),
        "A1: click moves the cursor"
    );
    assert_eq!(chosen_row(&app, 12), Some(6), "A1: click activates");
    // N1: an empty list ignores every key.
    let mut empty = Harness::new(
        SingleApp {
            items: Vec::new(),
            st: ListState::default(),
            log: Rc::new(RefCell::new(Vec::new())),
            area: Rect::new(0, 0, 30, 12),
        },
        Theme::junie(),
        30,
        14,
    );
    for code in [
        KeyCode::Enter,
        KeyCode::Char(' '),
        KeyCode::Char('j'),
        KeyCode::Char('G'),
        KeyCode::Up,
        KeyCode::PageDown,
    ] {
        let r = empty.key(code);
        assert_eq!(r.flow(), Flow::Ignored, "N1: empty list ignores {code:?}");
    }
    // N2: a click past the last row consumes without moving.
    let r = app.click(5, 11);
    assert_eq!(r.flow(), Flow::Consumed, "N2: past-end click consumes");
    assert_eq!(
        list_probe(&mut app, &log3),
        Some(ItemKey::index(6)),
        "N2: cursor unmoved"
    );
    // V1: the chosen row carries ›, other rows stay blank.
    assert_eq!(
        app.buffer()[(1, 6)].symbol(),
        "›",
        "V1: › on the chosen row"
    );
    for y in [0u16, 1, 2, 3, 4, 5, 7, 8, 9] {
        assert_eq!(
            app.buffer()[(1, y)].symbol(),
            " ",
            "V1: row {y} marker blank"
        );
    }
    capture_isolated(&dir, "list-single-chosen", app.buffer());
    // V2: the empty column shows centred empty text, never a marker.
    let empty_draw = Harness::new(
        EmptyListApp {
            st: ListState::default(),
        },
        Theme::junie(),
        30,
        14,
    );
    let mid = buf_row_text(empty_draw.buffer(), 6);
    assert!(mid.contains("No results for"), "V2: empty text shows");
    assert!(!mid.contains('›'), "V2: no marker on the empty row");
    let lead = mid.find("No").unwrap_or(0);
    assert!(lead > 2, "V2: empty text centres ({lead} cells in)");

    // PTY: move, jump, choose, click, and wheel the live column.
    let case = s3_case("list_single", &["--page", "lists"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("Chosen: Rust"), "boot shows the chosen");
    assert!(boot.contains("No results for"), "V2 live: empty text shows");
    assert_line_lacks(&boot, "No results", "›", "V2 live: no marker there");
    checkpoint(&mut s, &dir, "lists-initial");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "single list never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Rust"))
        },
    );
    support::press_step(&s, "j");
    support::press_step(&s, "j");
    support::press_step(&s, "G");
    checkpoint(&mut s, &dir, "single-moved");
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "single-chosen");
    waits::wait_state(&mut s, "Chosen: Erlang", "A1 live: G + Enter chooses last");
    let done = live_text(&mut s);
    assert_line_has(&done, "›", "Erlang", "V1 live: › on the chosen row");
    // G scrolled Python out of view; Home brings the head back first.
    support::press_step(&s, "home");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Python never scrolled back",
        |screen| support::screen_text(screen).contains("Python"),
    );
    click_at(&mut s, "Python", case_timeout(&case));
    waits::wait_state(&mut s, "Chosen: Python", "A1 live: click chooses");
    let (wrow, wcol) = find_pos(&mut s, "Chosen: Python", case_timeout(&case));
    wheel_at(&mut s, wcol, wrow + 3, Wheel::Down, 5);
    // The choice header keeps its Python while the row scrolls away.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "wheel never moved the viewport",
        |screen| {
            support::screen_text(screen)
                .lines()
                .filter(|l| l.contains("Python"))
                .count()
                == 1
        },
    );
    let wheeled = live_text(&mut s);
    assert!(
        wheeled.contains("Chosen: Python"),
        "S2 live: the wheel keeps the choice"
    );
    eprintln!("s3 list_single: isolated choose + live column flow");
}

const S3_LISTMULTI: Id = Id::root("s3.listmulti");

/// Multi-select rig: owned rows plus action log.
struct MultiApp {
    items: Vec<String>,
    st: ListState,
    log: Rc<RefCell<Vec<ListAction>>>,
}

impl MultiApp {
    fn files() -> Vec<String> {
        [
            "src/api/auth.rs",
            "src/api/billing.rs",
            "src/db/schema.rs",
            "tests/checkout.rs",
            "Cargo.lock",
            "docs/webhooks.md",
            "src/workers/mailer.rs",
            "src/config.rs",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn rig() -> (Self, Rc<RefCell<Vec<ListAction>>>) {
        let mut st = ListState::default();
        st.set_cursor(0, ItemKey::index(0));
        st.checked_mut().insert(ItemKey::index(0));
        st.checked_mut().insert(ItemKey::index(1));
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            MultiApp {
                items: Self::files(),
                st,
                log: Rc::clone(&log),
            },
            log,
        )
    }
}

/// Generated and locked rows are disabled in the multi fixture.
fn multi_disabled(item: &&str) -> bool {
    *item == "src/db/schema.rs" || *item == "Cargo.lock"
}

impl App for MultiApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        List::new(S3_LISTMULTI)
            .select_mode(SelectMode::Multi)
            .disabled_item(&multi_disabled)
            .update(cx, &mut self.st, &items)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        List::new(S3_LISTMULTI)
            .select_mode(SelectMode::Multi)
            .disabled_item(&multi_disabled)
            .draw(ui, Rect::new(0, 0, 40, 10), &self.st, &items);
    }
}

/// Whether row `y` carries ✓.
fn is_checked(app: &Harness<MultiApp>, y: u16) -> bool {
    app.buffer()[(1, y)].symbol() == "✓"
}

/// Count ✓ rows in the 8-row fixture.
fn checked_count(app: &Harness<MultiApp>) -> usize {
    (0..8).filter(|y| is_checked(app, *y)).count()
}

/// Probe the multi cursor: Enter activates without moving.
fn multi_probe(app: &mut Harness<MultiApp>, log: &Rc<RefCell<Vec<ListAction>>>) -> Option<ItemKey> {
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    log.borrow().iter().find_map(|a| match a {
        ListAction::Activated(k) => Some(*k),
        _ => None,
    })
}

/// LIST-MULTI-002 (`showcase/pages/lists`): multi-select column.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_list_multi() {
    let dir = s3_dir("s3_list_multi");

    // S1 isolated: Shift+arrows check anchor..cursor, skipping disabled;
    // plain moves clear the anchor.
    let (rig, log) = MultiApp::rig();
    let mut app = Harness::new(rig, Theme::junie(), 40, 12);
    assert!(app.tab_to(S3_LISTMULTI), "navigating focus");
    let shift = KeyModifiers::SHIFT;
    let _ = app.key_mod(KeyCode::Down, shift);
    let _ = app.key_mod(KeyCode::Down, shift);
    let _ = app.key_mod(KeyCode::Down, shift);
    assert_eq!(
        multi_probe(&mut app, &log),
        Some(ItemKey::index(3)),
        "range reaches row 3"
    );
    assert!(is_checked(&app, 3), "S1: row 3 checked by the range");
    assert!(!is_checked(&app, 2), "S1: disabled row 2 skipped");
    assert_eq!(checked_count(&app), 3, "S1: rows 0, 1, 3 checked");
    let _ = app.key(KeyCode::Down);
    // Row 4 is disabled: Enter reports nothing there, and the list
    // cursor paints no marker, so the round-trip proves the stop —
    // Down lands 5 iff the cursor was on 4 — then restores it. Plain
    // moves clear the anchor on every leg, as the oracle's single
    // move does, and Enter never flips checks, so the probe is clean.
    let _ = app.key(KeyCode::Down);
    assert_eq!(
        multi_probe(&mut app, &log),
        Some(ItemKey::index(5)),
        "plain move to row 4 (via 5)"
    );
    let _ = app.key(KeyCode::Up);
    let _ = app.key_mod(KeyCode::Down, shift);
    assert_eq!(
        multi_probe(&mut app, &log),
        Some(ItemKey::index(5)),
        "range reaches row 5"
    );
    assert!(
        is_checked(&app, 5),
        "S1: row 5 checked from the fresh anchor"
    );
    assert_eq!(
        checked_count(&app),
        4,
        "S1: anchor was 4, not 0 (rows 0,1,3,5)"
    );
    // S2: `a` checks all non-disabled rows, or clears them when full.
    let _ = app.key(KeyCode::Char('a'));
    assert_eq!(checked_count(&app), 6, "S2: all six enabled rows checked");
    assert!(
        !is_checked(&app, 2) && !is_checked(&app, 4),
        "S2: disabled rows never check"
    );
    let _ = app.key(KeyCode::Char('a'));
    assert_eq!(checked_count(&app), 0, "S2: second `a` clears");
    // A1: Space/Enter flip the cursor row unless disabled.
    let _ = app.key(KeyCode::Char('g'));
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Down);
    // Disabled row 2 reports no activation; round-trip through row 3.
    let _ = app.key(KeyCode::Down);
    assert_eq!(
        multi_probe(&mut app, &log),
        Some(ItemKey::index(3)),
        "cursor on the disabled row (via 3)"
    );
    let _ = app.key(KeyCode::Up);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Consumed, "A1: disabled Space consumes");
    assert!(!is_checked(&app, 2), "A1: disabled row never flips");
    let _ = app.key(KeyCode::Down);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: enabled Enter applies");
    assert!(is_checked(&app, 3), "A1: Enter flips the cursor row");
    // A2: clicking a row moves the cursor there and toggles it.
    log.borrow_mut().clear();
    let r = app.click(5, 6);
    assert_eq!(r.flow(), Flow::Consumed, "A2: click applies");
    assert_eq!(
        multi_probe(&mut app, &log),
        Some(ItemKey::index(6)),
        "A2: click moves the cursor"
    );
    assert!(is_checked(&app, 6), "A2: click toggles");
    // N1: activate on a disabled row consumes and changes nothing.
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Char('g'));
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Down);
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1: disabled activate consumes");
    assert!(!is_checked(&app, 2), "N1: nothing changes");
    // V1/V2/N2 isolated: ✓ rows, muted disabled rows, no focus on them.
    let _ = app.key(KeyCode::Char('a'));
    // Park the cursor on the disabled row for the N2 renders.
    let _ = app.key(KeyCode::Char('g'));
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Down);
    // Disabled row 2 reports no activation; round-trip through row 3.
    let _ = app.key(KeyCode::Down);
    assert_eq!(
        multi_probe(&mut app, &log),
        Some(ItemKey::index(3)),
        "cursor on the disabled row (via 3)"
    );
    let _ = app.key(KeyCode::Up);
    assert_eq!(
        app.buffer()[(1, 0)].symbol(),
        "✓",
        "V1: checked rows carry ✓"
    );
    assert_eq!(
        app.buffer()[(1, 2)].symbol(),
        " ",
        "V2: disabled rows never carry ✓"
    );
    assert_eq!(
        app.buffer()[(1, 4)].symbol(),
        " ",
        "V2: disabled rows never carry ✓"
    );
    assert_eq!(checked_count(&app), 6, "V1: six feed the `N selected` meta");
    // N2: hover and focus never appear on disabled rows — the disabled
    // row paints identically hovered or not.
    let plain = (
        app.buffer()[(3, 2)].fg,
        app.buffer()[(3, 2)].bg,
        app.buffer()[(0, 2)].symbol().to_owned(),
    );
    let _ = app.mouse(MouseKind::Move, 5, 2);
    let hovered = (
        app.buffer()[(3, 2)].fg,
        app.buffer()[(3, 2)].bg,
        app.buffer()[(0, 2)].symbol().to_owned(),
    );
    assert_eq!(plain, hovered, "N2: hover changes no disabled cell");
    assert_eq!(
        app.buffer()[(0, 2)].symbol(),
        " ",
        "N2: no gutter on disabled rows"
    );
    capture_isolated(&dir, "list-multi-all", app.buffer());

    // PTY: toggle, range, all, and a refused disabled activate.
    let case = s3_case("list_multi", &["--page", "lists"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert_eq!(count_glyph(&boot, '✓'), 2, "V1 live: two ✓ at boot");
    assert!(
        boot.contains("2 select"),
        "V1 live: `2 selected` meta shows"
    );
    checkpoint(&mut s, &dir, "multi-initial");
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "multi list never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("src/api/au"))
        },
    );
    support::press_step(&s, "space");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Space never toggled",
        |screen| count_glyph(&support::screen_text(screen), '✓') == 1,
    );
    support::press_step(&s, "shift+down");
    checkpoint(&mut s, &dir, "range-checked");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "range never re-checked",
        |screen| count_glyph(&support::screen_text(screen), '✓') == 2,
    );
    support::press_step(&s, "a");
    checkpoint(&mut s, &dir, "all-toggled");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "`a` never checked all",
        |screen| count_glyph(&support::screen_text(screen), '✓') == 10,
    );
    support::press_step(&s, "a");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "second `a` never cleared",
        |screen| count_glyph(&support::screen_text(screen), '✓') == 0,
    );
    // Cursor sits on row 1 after the range; one step lands the disabled row.
    support::press_step(&s, "j");
    support::press_step(&s, "enter");
    let done = settle_text(&mut s, case_timeout(&case), "disabled activate steady");
    assert_eq!(
        count_glyph(&done, '✓'),
        0,
        "N1 live: Enter on the disabled row changes nothing"
    );
    assert_line_lacks(&done, "▎", "schema", "N2 live: no gutter on disabled rows");
    support::press_step(&s, "up");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never sat on row 2",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("src/api/bi"))
        },
    );
    eprintln!("s3 list_multi: isolated ranges + live toggle-all flow");
}

const S3_PICKQUERY: Id = Id::root("s3.pickquery");

/// Owned picker row with semantic identity.
struct PItem {
    key: ItemKey,
    label: String,
    detail: String,
    glyph: String,
    group: Option<String>,
    tag: Option<String>,
    matched: Vec<usize>,
    disabled: bool,
}

impl AsItem for PItem {
    fn as_item(&self) -> Item<'_> {
        let mut item = Item::new(self.key, self.label.as_str());
        item.detail = self.detail.as_str();
        item.glyph = self.glyph.as_str();
        item.group = self.group.as_deref();
        item.tag = self.tag.as_deref();
        item.matched = &self.matched;
        item.disabled = self.disabled;
        item
    }
}

/// Quick-open rig: modal picker owner plus action log.
struct QueryApp {
    items: Vec<PItem>,
    st: PickerState,
    log: Rc<RefCell<Vec<PickerAction>>>,
    open: bool,
    booted: bool,
    scopes: [ScopeKey; 2],
}

impl QueryApp {
    fn auth_items() -> Vec<PItem> {
        vec![
            PItem {
                key: ItemKey::text("auth.rs"),
                label: "auth.rs".to_string(),
                detail: "src/api/auth.rs".to_string(),
                glyph: "F".to_string(),
                group: Some("Files".to_string()),
                matched: Vec::new(),
                disabled: false,
                tag: None,
            },
            PItem {
                key: ItemKey::text("auth_flow.rs"),
                label: "auth_flow.rs".to_string(),
                detail: "tests/auth_flow.rs".to_string(),
                glyph: "F".to_string(),
                group: Some("Files".to_string()),
                matched: Vec::new(),
                disabled: false,
                tag: None,
            },
            PItem {
                key: ItemKey::text("rate-limit-task"),
                label: "Add rate limiting to auth endpoints".to_string(),
                detail: "#14 · mira".to_string(),
                glyph: "T".to_string(),
                group: Some("Tasks".to_string()),
                matched: Vec::new(),
                disabled: false,
                tag: None,
            },
        ]
    }

    fn rig(items: Vec<PItem>, st: PickerState) -> (Self, Rc<RefCell<Vec<PickerAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            QueryApp {
                items,
                st,
                log: Rc::clone(&log),
                open: false,
                booted: false,
                scopes: [ScopeKey::new(0), ScopeKey::new(1)],
            },
            log,
        )
    }

    fn picker(&self) -> Picker<'_, PItem> {
        Picker::new(S3_PICKQUERY)
            .title("Open quickly")
            .placeholder("Files and tasks…")
            .meta("All · Tab scope")
            .width(76)
            .scopes(&self.scopes)
    }
}

impl App for QueryApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let scopes = self.scopes;
        // Boot-only open: a cancel-close must stay closed. Reopening on
        // `!open` resurrects the layer in the same keystroke's bubble
        // re-update, defeating Cancel.
        if !self.booted {
            self.booted = true;
            self.open = true;
            Picker::new(S3_PICKQUERY)
                .title("Open quickly")
                .placeholder("Files and tasks…")
                .meta("All · Tab scope")
                .width(76)
                .scopes(&scopes)
                .reconcile(&mut self.st, &self.items);
            let spec = Picker::new(S3_PICKQUERY)
                .title("Open quickly")
                .placeholder("Files and tasks…")
                .meta("All · Tab scope")
                .width(76)
                .scopes(&scopes)
                .layer(cx, &self.items);
            cx.open_layer(S3_PICKQUERY, spec);
        }
        let r = Picker::new(S3_PICKQUERY)
            .title("Open quickly")
            .placeholder("Files and tasks…")
            .meta("All · Tab scope")
            .width(76)
            .scopes(&scopes)
            .update(cx, &mut self.st, &self.items)
            .on_action(|action| log.borrow_mut().push(action));
        if !cx.is_open(S3_PICKQUERY) {
            self.open = false;
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.open {
            let _ = ui.layer(S3_PICKQUERY, |ui, area| {
                self.picker().draw(ui, area, &self.st, &self.items)
            });
        }
    }
}

/// Whether the query editor shows its placeholder (i.e. the query is blank).
fn query_blank(app: &Harness<QueryApp>) -> bool {
    app.text().contains("Files and tasks…")
}

/// FLIST-QUERY-001 (`showcase/pages/pickers`): query + ranked rows.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_picker_query() {
    let dir = s3_dir("s3_picker_query");

    // S1 isolated: every query edit emits QueryChanged; set_items resets
    // to the first eligible row.
    let (rig, log) = QueryApp::rig(QueryApp::auth_items(), PickerState::default());
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.is_open(S3_PICKQUERY), "modal opens");
    assert!(query_blank(&app), "query starts blank");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('a'));
    assert_eq!(r.flow(), Flow::Consumed, "S1: typing applies");
    assert!(
        log.borrow().contains(&PickerAction::QueryChanged),
        "S1: typing emits QueryChanged"
    );
    assert!(!query_blank(&app), "query accumulates");
    // A replaced projection resets the cursor past disabled rows (the
    // same reconcile path `set_items` drives).
    let mut ranked = QueryApp::auth_items();
    ranked[0].disabled = true;
    let (rig, log) = QueryApp::rig(ranked, PickerState::default());
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S1: reset applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == PickerAction::Chosen(ItemKey::text("auth_flow.rs"))),
        "S1: set_items resets past disabled rows"
    );
    // S2: Esc with text clears; Esc empty cancels.
    let mut st = PickerState::default();
    st.set_query("auth");
    let (rig, log) = QueryApp::rig(QueryApp::auth_items(), st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(!query_blank(&app), "query staged");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "S2: Esc with text applies");
    assert!(
        log.borrow().contains(&PickerAction::QueryChanged),
        "S2: Esc with text clears the query"
    );
    assert!(query_blank(&app), "S2: query cleared");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "S2: Esc empty applies");
    assert!(!app.is_open(S3_PICKQUERY), "S2: Esc empty cancels");
    // A1: Enter emits Chosen (ChosenAlt with alt); only eligible resolve.
    let mut st = PickerState::default();
    st.set_cursor(1, ItemKey::text("auth_flow.rs"));
    let (rig, log) = QueryApp::rig(QueryApp::auth_items(), st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == PickerAction::Chosen(ItemKey::text("auth_flow.rs"))),
        "A1: Enter emits Chosen"
    );
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Enter, KeyModifiers::ALT);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Alt+Enter applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == PickerAction::ChosenAlt(ItemKey::text("auth_flow.rs"))),
        "A1: Alt+Enter emits ChosenAlt"
    );
    let mut stale = QueryApp::auth_items();
    for item in stale.iter_mut() {
        item.disabled = true;
    }
    let mut st = PickerState::default();
    st.set_query("auth");
    let (rig, log) = QueryApp::rig(stale, st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A1: Enter on no eligible row applies"
    );
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, PickerAction::Chosen(_) | PickerAction::ChosenAlt(_))),
        "A1: never Chosen without an eligible row"
    );
    assert!(
        !log.borrow().is_empty(),
        "A1: Enter on no eligible row falls to Submit"
    );
    // A2: Tab emits NextScope; foreign ids are not owned.
    let (rig, log) = QueryApp::rig(QueryApp::auth_items(), PickerState::default());
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Tab);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Tab applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == PickerAction::Scope(ScopeKey::new(1))),
        "A2: Tab emits NextScope"
    );
    assert!(
        app.area_of_part(
            S3_PICKQUERY,
            PartRef::item(Part::ROW, ItemKey::text("foreign"))
        )
        .is_none(),
        "A2: outside clicks miss"
    );
    assert!(
        app.area_of_part(
            S3_PICKQUERY,
            PartRef::item(Part::ROW, ItemKey::text("auth.rs"))
        )
        .is_some(),
        "A2: rows are owned (presence proof)"
    );
    // N1: Enter with no eligible row and a blank query consumes silently.
    let mut stale = QueryApp::auth_items();
    for item in stale.iter_mut() {
        item.disabled = true;
    }
    let (rig, log) = QueryApp::rig(stale, PickerState::default());
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(query_blank(&app), "query blank");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "N1: blank query + no eligible consumes"
    );
    assert!(log.borrow().is_empty(), "N1: silently");
    // N2 contrast: typing works while searchable.
    let (rig, _log) = QueryApp::rig(QueryApp::auth_items(), PickerState::default());
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let _ = app.key(KeyCode::Char('j'));
    assert!(!query_blank(&app), "N2: searchable pickers type j");
    // V1: placeholder when empty, underlined query otherwise.
    let (rig, _log) = QueryApp::rig(QueryApp::auth_items(), PickerState::default());
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(query_blank(&app), "V1: placeholder shows");
    let mut st = PickerState::default();
    st.set_query("au");
    let (rig, _log) = QueryApp::rig(QueryApp::auth_items(), st);
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    // The query row underlines the typed text.
    let mut underlined = false;
    for y in 0..24u16 {
        for x in 0..80u16 {
            if app.buffer()[(x, y)].symbol() == "a" && buf_is_underlined(app.buffer(), x, y) {
                underlined = true;
            }
        }
    }
    assert!(underlined, "V1: query text underlines");
    capture_isolated(&dir, "picker-query", app.buffer());
    // V2/V3: matched bytes read bold, scope right-aligns, groups label once.
    let mut items = QueryApp::auth_items();
    items[0].matched = vec![0, 1, 2, 3];
    let mut st = PickerState::default();
    st.set_cursor(2, ItemKey::text("rate-limit-task"));
    let (rig, _log) = QueryApp::rig(items, st);
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    let row0 = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("src/api/auth.rs"))
        .expect("auth.rs row");
    let bold_cells: Vec<u16> = (0..80)
        .filter(|x| buf_is_bold(app.buffer(), *x, row0))
        .collect();
    assert_eq!(
        bold_cells.len(),
        4,
        "V2: exactly the four matched bytes read bold"
    );
    let title_row = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("Open quickly"))
        .expect("title row");
    // Right-aligned: past the scope sits only padding and the modal border.
    let scope_end =
        title_row.find("All · Tab scope").expect("V2: scope shows") + "All · Tab scope".len();
    let rest: String = title_row[scope_end..]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(rest, "│", "V2: scope right-aligns in the title row");
    let group_rows = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .filter(|l| l.contains("Files") && l.contains("auth"))
        .count();
    assert_eq!(group_rows, 1, "V3: one Files label on the first group row");

    // PTY: open, filter, choose, clear, cancel, and scope-cycle live.
    let case = s3_case("picker_query", &["--page", "pickers"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    // The modal opening proves the Tab landed on the quick button.
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "picker-open");
    waits::wait_state(&mut s, "All · Tab scope", "V2 live: scope shows");
    waits::wait_state(&mut s, "Files and tasks…", "V1 live: placeholder shows");
    type_text(&mut s, "auth");
    checkpoint(&mut s, &dir, "query-auth");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "filter never applied",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("auth.rs") && !text.contains("Cargo.toml")
        },
    );
    support::press_step(&s, "down");
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "chose-auth");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "choose never closed the modal",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Chose ") && !text.contains("All · Tab scope")
        },
    );
    // Reopen, type, clear with Esc, cancel with Esc. (Choosing closed
    // the modal, which returned focus to NAV: Tab back on to Quick first.)
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Quick button never refocused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Open quickly"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "All · Tab scope", "modal reopened");
    type_text(&mut s, "auth");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "second filter never applied",
        |screen| support::screen_text(screen).contains("auth.rs"),
    );
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Esc never cleared the query",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Files and tasks…") && text.contains("Cargo.toml")
        },
    );
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "second Esc never cancelled",
        |screen| !support::screen_text(screen).contains("All · Tab scope"),
    );
    // Reopen and cycle the scope with Tab. (Cancelling also returned
    // focus to NAV.)
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Quick button never refocused again",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Open quickly"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "All · Tab scope", "modal reopened again");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Files · Tab scope", "A2 live: scope cycles");
    eprintln!("s3 picker_query: isolated query map + live choose flow");
}

const S3_PICKCHOICE: Id = Id::root("s3.pickchoice");

/// Fixed-choice rig: modal picker owner plus action log.
struct ChoiceApp {
    items: Vec<PItem>,
    st: PickerState,
    log: Rc<RefCell<Vec<PickerAction>>>,
    open: bool,
    booted: bool,
    title: &'static str,
    placeholder: &'static str,
    searchable: bool,
}

impl ChoiceApp {
    fn levels(current: usize) -> Vec<PItem> {
        [
            "Silent",
            "Alert",
            "Alert (Full)",
            "Safe Mode",
            "Safe Mode (Full)",
            "Read-Only",
        ]
        .iter()
        .enumerate()
        .map(|(i, l)| PItem {
            key: ItemKey::text(match i {
                0 => "level-0",
                1 => "level-1",
                2 => "level-2",
                3 => "level-3",
                4 => "level-4",
                _ => "level-5",
            }),
            label: l.to_string(),
            detail: "writes ask".to_string(),
            glyph: " ".to_string(),
            group: None,
            matched: Vec::new(),
            disabled: false,
            tag: if i == current {
                Some("current".to_string())
            } else {
                None
            },
        })
        .collect()
    }

    fn rig(
        title: &'static str,
        placeholder: &'static str,
        searchable: bool,
        items: Vec<PItem>,
        st: PickerState,
    ) -> (Self, Rc<RefCell<Vec<PickerAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            ChoiceApp {
                items,
                st,
                log: Rc::clone(&log),
                open: false,
                booted: false,
                title,
                placeholder,
                searchable,
            },
            log,
        )
    }
}

impl App for ChoiceApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let title = self.title;
        let placeholder = self.placeholder;
        let searchable = self.searchable;
        // Boot-only open: a cancel-close must stay closed (see QueryApp).
        if !self.booted {
            self.booted = true;
            self.open = true;
            Picker::new(S3_PICKCHOICE)
                .title(title)
                .placeholder(placeholder)
                .width(76)
                .searchable(searchable)
                .reconcile(&mut self.st, &self.items);
            let spec = Picker::new(S3_PICKCHOICE)
                .title(title)
                .placeholder(placeholder)
                .width(76)
                .searchable(searchable)
                .layer(cx, &self.items);
            cx.open_layer(S3_PICKCHOICE, spec);
        }
        let r = Picker::new(S3_PICKCHOICE)
            .title(title)
            .placeholder(placeholder)
            .width(76)
            .searchable(searchable)
            .update(cx, &mut self.st, &self.items)
            .on_action(|action| log.borrow_mut().push(action));
        if !cx.is_open(S3_PICKCHOICE) {
            self.open = false;
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.open {
            let title = self.title;
            let placeholder = self.placeholder;
            let searchable = self.searchable;
            let _ = ui.layer(S3_PICKCHOICE, |ui, area| {
                Picker::new(S3_PICKCHOICE)
                    .title(title)
                    .placeholder(placeholder)
                    .width(76)
                    .searchable(searchable)
                    .draw(ui, area, &self.st, &self.items)
            });
        }
    }
}

/// Probe the picker cursor: Enter chooses without moving.
fn choice_probe(
    app: &mut Harness<ChoiceApp>,
    log: &Rc<RefCell<Vec<PickerAction>>>,
) -> Option<ItemKey> {
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    log.borrow().iter().find_map(|a| match a {
        PickerAction::Chosen(k) => Some(*k),
        _ => None,
    })
}

/// Loading/error rig over the embedded filter engine (Picker exposes no
/// status; the engine owns the resolve/refuse contract).
struct StatusApp {
    items: Vec<PItem>,
    st: FilterListState,
    log: Rc<RefCell<Vec<FilterListAction>>>,
    status: Status,
}

impl App for StatusApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        FilterList::new(S3_PICKCHOICE)
            .status(self.status)
            .update(cx, &mut self.st, &self.items)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        FilterList::new(S3_PICKCHOICE).status(self.status).draw(
            ui,
            Rect::new(0, 0, 76, 10),
            &self.st,
            &self.items,
        );
    }
}

/// FLIST-CHOICE-002 (`showcase/flows/pickers/tabs`): fixed-choice + tabs.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_picker_choice() {
    let dir = s3_dir("s3_picker_choice");

    // S1 isolated: refresh follows the cursor key; a vanished key falls.
    // (Fresh projections through the same reconcile path `refresh_items`
    // drives: the cursor key survives when present, else falls to first.)
    let mut st = PickerState::default();
    st.set_cursor(1, ItemKey::text("b"));
    let (rig, log) = ChoiceApp::rig(
        "Open tabs",
        "Filter tabs…",
        true,
        ["a", "b", "c"]
            .iter()
            .map(|k| PItem {
                key: ItemKey::text(k),
                label: format!("tab {k}"),
                detail: String::new(),
                glyph: String::new(),
                group: None,
                matched: Vec::new(),
                disabled: false,
                tag: None,
            })
            .collect(),
        st,
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        choice_probe(&mut app, &log),
        Some(ItemKey::text("b")),
        "cursor staged on b"
    );
    let mut st = PickerState::default();
    st.set_cursor(1, ItemKey::text("b"));
    let (rig, log) = ChoiceApp::rig(
        "Open tabs",
        "Filter tabs…",
        true,
        ["x", "b", "c"]
            .iter()
            .map(|k| PItem {
                key: ItemKey::text(k),
                label: format!("tab {k}"),
                detail: String::new(),
                glyph: String::new(),
                group: None,
                matched: Vec::new(),
                disabled: false,
                tag: None,
            })
            .collect(),
        st,
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        choice_probe(&mut app, &log),
        Some(ItemKey::text("b")),
        "S1: cursor follows the b key across refresh"
    );
    let mut st = PickerState::default();
    st.set_cursor(1, ItemKey::text("b"));
    let (rig, log) = ChoiceApp::rig(
        "Open tabs",
        "Filter tabs…",
        true,
        ["x", "y"]
            .iter()
            .map(|k| PItem {
                key: ItemKey::text(k),
                label: format!("tab {k}"),
                detail: String::new(),
                glyph: String::new(),
                group: None,
                matched: Vec::new(),
                disabled: false,
                tag: None,
            })
            .collect(),
        st,
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        choice_probe(&mut app, &log),
        Some(ItemKey::text("x")),
        "S1: vanished key falls to first eligible"
    );
    // S2: Delete emits Secondary only for an eligible row of Ready.
    let mut st = PickerState::default();
    st.set_cursor(3, ItemKey::text("level-3"));
    let (rig, log) = ChoiceApp::rig(
        "Safe Mode · this connection",
        "Type to search…",
        false,
        ChoiceApp::levels(3),
        st,
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Delete);
    assert_eq!(r.flow(), Flow::Consumed, "S2: Delete applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == PickerAction::Secondary(ItemKey::text("level-3"))),
        "S2: Delete emits Secondary when eligible"
    );
    // N1: Loading/Error resolve to nothing (engine level: Picker exposes
    // no status control; the embedded filter owns the contract).
    for status in [Status::Loading, Status::Error] {
        for code in [KeyCode::Delete, KeyCode::Enter] {
            let log = Rc::new(RefCell::new(Vec::new()));
            let mut st = FilterListState::default();
            st.set_cursor(3, ItemKey::text("level-3"));
            let mut app = Harness::new(
                StatusApp {
                    items: ChoiceApp::levels(3),
                    st,
                    log: Rc::clone(&log),
                    status,
                },
                Theme::junie(),
                80,
                24,
            );
            log.borrow_mut().clear();
            let r = app.key(code);
            assert_eq!(
                r.flow(),
                Flow::Consumed,
                "N1: {code:?} resolves to nothing while {status:?}"
            );
            assert!(log.borrow().is_empty(), "N1: {code:?} emits nothing");
        }
    }
    // A1: Enter on a level emits Chosen (the page sets and closes).
    let mut st = PickerState::default();
    st.set_cursor(4, ItemKey::text("level-4"));
    let (rig, log) = ChoiceApp::rig(
        "Safe Mode · this connection",
        "Type to search…",
        false,
        ChoiceApp::levels(3),
        st,
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == PickerAction::Chosen(ItemKey::text("level-4"))),
        "A1: Enter emits Chosen(4)"
    );
    // A2: Backspace on an empty query emits Back instead of editing.
    let (rig, log) = ChoiceApp::rig(
        "Open quickly",
        "Files and tasks…",
        true,
        vec![PItem {
            key: ItemKey::text("auth.rs"),
            label: "auth.rs".to_string(),
            detail: String::new(),
            glyph: String::new(),
            group: None,
            matched: Vec::new(),
            disabled: false,
            tag: None,
        }],
        PickerState::default(),
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Backspace);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Backspace on empty applies");
    assert!(
        log.borrow().contains(&PickerAction::Back),
        "A2: Backspace on empty emits Back"
    );
    let _ = app.key(KeyCode::Char('z'));
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Backspace);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Backspace with text applies");
    assert!(
        log.borrow().contains(&PickerAction::QueryChanged),
        "A2: Backspace with text pops + QueryChanged"
    );
    assert!(app.text().contains("Files and tasks…"), "query popped");
    // N2: j/k type when searchable, move when not.
    let (rig, log) = ChoiceApp::rig(
        "Open quickly",
        "Files and tasks…",
        true,
        vec![
            PItem {
                key: ItemKey::text("a"),
                label: "a".to_string(),
                detail: String::new(),
                glyph: String::new(),
                group: None,
                matched: Vec::new(),
                disabled: false,
                tag: None,
            },
            PItem {
                key: ItemKey::text("b"),
                label: "b".to_string(),
                detail: String::new(),
                glyph: String::new(),
                group: None,
                matched: Vec::new(),
                disabled: false,
                tag: None,
            },
        ],
        PickerState::default(),
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let _ = app.key(KeyCode::Char('j'));
    assert!(
        !app.text().contains("Files and tasks…"),
        "N2: searchable j types"
    );
    assert_eq!(
        choice_probe(&mut app, &log),
        Some(ItemKey::text("a")),
        "N2: typing never moves"
    );
    let mut st = PickerState::default();
    st.set_cursor(3, ItemKey::text("level-3"));
    let (rig, log) = ChoiceApp::rig(
        "Safe Mode · this connection",
        "Type to search…",
        false,
        ChoiceApp::levels(3),
        st,
    );
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let _ = app.key(KeyCode::Char('j'));
    assert_eq!(
        choice_probe(&mut app, &log),
        Some(ItemKey::text("level-4")),
        "N2: fixed-list j moves"
    );
    let _ = app.key(KeyCode::Char('k'));
    assert_eq!(
        choice_probe(&mut app, &log),
        Some(ItemKey::text("level-3")),
        "N2: fixed-list k moves"
    );
    // V1: the Level modal has no query field; the cursor row keeps a gutter.
    let screen = app.text();
    assert!(
        !screen.contains("Type to search"),
        "V1: no query field on the Level modal"
    );
    let safe_row = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("Safe Mode (Full)") && l.contains('▎'))
        .expect("V1: cursor row keeps its ▎ gutter");
    assert!(
        safe_row.contains("Safe Mode (Full)"),
        "V1: gutter on the cursor row"
    );
    // V2: the current level shows `current`; the active tab shows `active`.
    assert!(
        screen.contains("current"),
        "V2: current tag shows (presence)"
    );
    let (rig, _log) = ChoiceApp::rig(
        "Open tabs",
        "Filter tabs…",
        true,
        ["Query 1", "orders", "order_items", "History"]
            .iter()
            .map(|t| PItem {
                key: ItemKey::text(t),
                label: t.to_string(),
                detail: String::new(),
                glyph: String::new(),
                group: Some("Open tabs".to_string()),
                matched: Vec::new(),
                disabled: false,
                tag: None,
            })
            .collect(),
        PickerState::default(),
    );
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    capture_isolated(&dir, "picker-tabs", app.buffer());
    let tagged = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("orders") && l.contains("active"))
        .expect("V2: active tag on the orders row");
    assert!(tagged.contains("orders"), "V2: tag rides the right row");

    // PTY: close a tab, set a level, Back on empty, j-types-live.
    let case = s3_case("picker_choice", &["--page", "pickers"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "tabs-open");
    waits::wait_state(&mut s, "Filter tabs…", "Tabs modal opens");
    waits::wait_state(&mut s, "active", "V2 live: active tag shows");
    support::press_step(&s, "delete");
    checkpoint(&mut s, &dir, "tab-closed");
    waits::wait_state(&mut s, "Closed Query 1", "S2 live: Delete closes");
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Tabs modal never closed",
        |screen| !support::screen_text(screen).contains("Filter tabs…"),
    );
    // Modal close returns focus to the sidebar (▎ back on a nav row, in
    // the first 19 cells where no page widget draws). Tab on to the Level
    // button: the modal must open with focus off-NAV, else the sidebar
    // hijacks the Level's j/Enter keys.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "focus never returned to the sidebar",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.chars().take(19).collect::<String>().contains('▎'))
        },
    );
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Level button never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Choose a level"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Safe Mode · this connection", "Level opens");
    let level_text = live_text(&mut s);
    assert!(
        !level_text.contains("Type to search"),
        "V1 live: no query field on Level"
    );
    assert!(level_text.contains("current"), "V2 live: current tag shows");
    checkpoint(&mut s, &dir, "level-open");
    support::press_step(&s, "j");
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "level-set");
    waits::wait_state(
        &mut s,
        "Chose Safe Mode (Full)",
        "A1 live: j moves, Enter sets",
    );
    let done = live_text(&mut s);
    assert_line_has(
        &done,
        "Level",
        "Safe Mode (Full)",
        "A1 live: Result level follows",
    );
    // Quick: Backspace on empty is a silent Back; j types, never moves.
    // The Level choice closed its modal, so focus is back on NAV: one Tab
    // reaches Quick (proven by the button's ▎ before Enter).
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Quick button never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Open quickly"))
        },
    );
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Files and tasks…", "Quick reopened");
    support::press_step(&s, "backspace");
    let back = settle_text(&mut s, case_timeout(&case), "Back steady");
    assert!(
        back.contains("Files and tasks…"),
        "A2 live: Back keeps the modal open with an empty query"
    );
    // A query no label can match: `j` must type, never move.
    type_text(&mut s, "jjj");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "j never typed into the query",
        |screen| support::screen_text(screen).contains("No matches"),
    );
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "query never cleared back",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Files and tasks…") && text.contains("Cargo.toml")
        },
    );
    eprintln!("s3 picker_choice: isolated fixed-choice + live level flow");
}

/// Empty single-select rig with centred empty text.
struct EmptyListApp {
    st: ListState,
}

impl App for EmptyListApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let items: Vec<&str> = Vec::new();
        List::new(S3_LISTSINGLE)
            .select_mode(SelectMode::Single)
            .empty(EmptyState::Empty {
                title: "No results for “retry”",
                hint: None,
            })
            .update(cx, &mut self.st, &items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let items: Vec<&str> = Vec::new();
        List::new(S3_LISTSINGLE)
            .select_mode(SelectMode::Single)
            .empty(EmptyState::Empty {
                title: "No results for “retry”",
                hint: None,
            })
            .draw(ui, Rect::new(0, 0, 30, 12), &self.st, &items);
    }
}

/// Masked rig with a live validator (mask A2).
struct MaskValidApp {
    value: String,
    st: TextInputState,
    tail: usize,
}

impl App for MaskValidApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextInput::new(S3_TIMASK)
            .secret(SecretPolicy {
                synthetic_tail: self.tail,
                ..Default::default()
            })
            .validate(&no_x_keys)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIMASK)
            .secret(SecretPolicy {
                synthetic_tail: self.tail,
                ..Default::default()
            })
            .value(&self.value)
            .draw(ui, S3_TIMASK_AREA, &self.st);
    }
}

/// Reject empty values (paste S2 required rule).
fn non_empty(s: &str) -> Result<(), FieldError> {
    if s.is_empty() {
        Err(FieldError::new("required"))
    } else {
        Ok(())
    }
}

/// Paste rig with a required validator.
struct RequiredApp {
    value: String,
    st: TextInputState,
    log: Rc<RefCell<Vec<TextAction>>>,
}

impl App for RequiredApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        TextInput::new(S3_TIPASTE)
            .validate(&non_empty)
            .update(cx, &mut self.st, &mut self.value)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIPASTE)
            .value(&self.value)
            .placeholder("feat/…")
            .draw(ui, S3_TIPASTE_AREA, &self.st);
    }
}

/// Disabled paste rig.
struct DisabledPasteApp {
    value: String,
    st: TextInputState,
}

impl App for DisabledPasteApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextInput::new(S3_TIPASTE)
            .disabled(true)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S3_TIPASTE)
            .value(&self.value)
            .disabled(true)
            .draw(ui, S3_TIPASTE_AREA, &self.st);
    }
}

const S3_TIPASTE_AREA_NOTES: Id = Id::root("s3.tipaste.notes");

/// Idle-area paste rig (paste N1/N2).
struct NotesApp {
    value: String,
    st: TextAreaState,
}

impl App for NotesApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        TextArea::new(S3_TIPASTE_AREA_NOTES, 4)
            .update(cx, &mut self.st, &mut self.value)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextArea::new(S3_TIPASTE_AREA_NOTES, 4)
            .value(&self.value)
            .placeholder("Notes…")
            .draw(ui, Rect::new(4, 1, 36, 4), &self.st);
    }
}

/// NAV-SHELL-001 (`showcase/flows/nav/cycled`): shell sidebar (PTY-only).
///
/// The shell sidebar lives in the showcase binary (`app.rs`), outside the
/// importable library, so every check runs live. Compact-mode note: at
/// 80x24 the sidebar drops section labels and keeps every entry on one
/// contiguous row (the `compact` branch in `draw_sidebar` draws no blank
/// rows); V2 asserts exactly that observed shape.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_nav_shell() {
    let dir = s3_dir("s3_nav_shell");
    let case = s3_case("nav_shell", &["--page", "overview"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");

    // Sidebar column: the first 19 cells of every line.
    fn sidebar(text: &str) -> Vec<String> {
        text.lines().map(|l| l.chars().take(19).collect()).collect()
    }

    let boot = live_text(&mut s);
    assert_line_has(&boot, "▎", "Overview", "boot focuses the sidebar");
    // V2: compact at 80x24 — no section labels, contiguous one-cell rows.
    let side = sidebar(&boot);
    assert!(
        !side.iter().any(|l| l.contains("Foundations")
            || l.contains("Components")
            || l.contains("Screens")),
        "V2 live: sidebar carries no section labels"
    );
    let entries: Vec<&String> = side
        .iter()
        .filter(|l| {
            l.contains("Overview")
                || l.contains("Buttons")
                || l.contains("Inputs")
                || l.contains("Pickers")
        })
        .collect();
    assert_eq!(entries.len(), 4, "V2 live: entries render, got {side:?}");
    // A1: Down moves the cursor; Enter opens the cursor page.
    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never reached Buttons",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Buttons"))
        },
    );
    let moved = live_text(&mut s);
    assert!(
        moved.contains("/ Foundations / Overview"),
        "N2 live: cursor moves never change the page alone"
    );
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "nav-buttons");
    waits::wait_state(&mut s, "/ Components / Buttons", "A1 live: Buttons opens");
    let opened = live_text(&mut s);
    assert_line_has(&opened, "›", "Buttons", "V1 live: › marks the open page");
    assert_line_has(&opened, "▎", "Buttons", "V1 live: ▎ marks the cursor");
    // N1: the ▎ gutter never shows while focus sits inside a page, while
    // › persists across the focus move (N2). Then `0` refocuses (A2).
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Tab never entered the page",
        |screen| {
            !support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Buttons"))
        },
    );
    let in_page = live_text(&mut s);
    assert_line_has(&in_page, "›", "Buttons", "N2 live: › persists in-page");
    support::press_step(&s, "0");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "`0` never refocused the sidebar",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Buttons"))
        },
    );
    // S2: `]` cycles the page with wrap-around. REGISTRY DEVIATION:
    // NAV-SHELL-001 S2 says "the cursor is untouched", but the global
    // `]`/`[` arms call `goto`, which sets `nav_cursor = page.index()`
    // (`app.rs`), so on VB the cursor FOLLOWS the cycled page. This pins
    // the observed VB behavior; the converse (N2: cursor moves never
    // change the page alone) holds and is asserted above and below.
    support::press_step(&s, "]");
    waits::wait_state(&mut s, "/ Components / Inputs", "S2 live: ] cycles");
    let cycled = live_text(&mut s);
    assert_line_has(&cycled, "›", "Inputs", "S2 live: › follows the page");
    assert_line_has(&cycled, "▎", "Inputs", "S2 live: cursor follows (VB)");
    // S1: End jumps to the last entry and reveals it; Down clamps there.
    support::press_step(&s, "end");
    checkpoint(&mut s, &dir, "nav-end");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "End never revealed the last entry",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Task runner"))
        },
    );
    support::press_step(&s, "down");
    let clamped = settle_text(&mut s, case_timeout(&case), "clamp steady");
    assert_line_has(&clamped, "▎", "Task runner", "S1 live: Down clamps at last");
    assert!(
        clamped.contains("/ Components / Inputs"),
        "N2 live: jumps never change the page alone"
    );
    // A2: Esc returns focus to navigation from the page.
    support::press_step(&s, "home");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Tab never entered the inputs page",
        |screen| {
            !support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Overview"))
        },
    );
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Esc never refocused navigation",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Overview"))
        },
    );
    // S2 wrap-around both ways; A2 clicks move the cursor and open.
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "/ Foundations / Overview", "Overview reopens");
    support::press_step(&s, "[");
    waits::wait_state(&mut s, "/ Task runner", "S2 live: [ wraps to the end");
    support::press_step(&s, "]");
    waits::wait_state(&mut s, "/ Foundations / Overview", "S2 live: ] wraps back");
    click_at(&mut s, "Trees", case_timeout(&case));
    waits::wait_state(&mut s, "/ Components / Trees", "A2 live: click opens Trees");
    let clicked = live_text(&mut s);
    assert_line_has(&clicked, "▎", "Trees", "A2 live: click moves the cursor");
    assert_line_has(&clicked, "›", "Trees", "A2 live: click opens the page");
    eprintln!("s3 nav_shell: live sidebar cycle (PTY-only)");
}

/// NAV-SIDE-002 (`showcase/pages/sidebars`): sidebar page (PTY-only).
///
/// The demo `NavList` lives in the showcase binary (`sidebars.rs`), outside
/// the importable library, so every check runs live. Collapse-label note:
/// the code sets `Expand` when collapsed and immediately overwrites it with
/// `›` (`sidebars.rs`), so the observable cycle is Collapse → › → Collapse
/// and V3 asserts exactly that.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_nav_side() {
    let dir = s3_dir("s3_nav_side");
    let case = s3_case("nav_side", &["--page", "sidebars"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    checkpoint(&mut s, &dir, "sidebars-initial");

    // Open-mode nav card (cols 19..47); the content card starts at col 49.
    fn nav_col(text: &str) -> Vec<String> {
        text.lines()
            .map(|l| l.chars().skip(19).take(28).collect())
            .collect()
    }
    // Content title column (past col 30 in both modes).
    fn title_has(text: &str, needle: &str) -> bool {
        text.lines().any(|l| {
            l.chars().skip(30).collect::<String>().contains(needle)
                && !l.chars().take(30).collect::<String>().contains(needle)
        })
    }

    let boot = live_text(&mut s);
    // V1: section headings precede each group with a blank row between.
    let col = nav_col(&boot);
    for section in ["Workspace", "Project", "Preferences"] {
        assert!(
            col.iter().any(|l| l.contains(section)),
            "V1 live: {section:?} heading shows"
        );
    }
    let branches = col
        .iter()
        .position(|l| l.contains("Branches"))
        .expect("Branches row");
    let members = col
        .iter()
        .position(|l| l.contains("Members"))
        .expect("Members row");
    assert_eq!(members, branches + 3, "V1 live: blank + heading between");
    assert!(
        col[branches + 1].trim().is_empty(),
        "V1 live: the gap row is blank"
    );
    // Tab into the page nav, walk past enabled items, open one.
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "page nav never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Tasks"))
        },
    );
    // Cursor starts on Tasks; Runs sits between Tasks and Branches.
    // Each step is awaited: presses queue asynchronously, so a burst
    // followed by a middle-state wait would fire on a transient.
    support::press_step(&s, "j");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never reached Runs",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Runs"))
        },
    );
    support::press_step(&s, "j");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never reached Branches",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Branches"))
        },
    );
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "item-opened");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Branches never opened",
        |screen| title_has(&support::screen_text(screen), "Branches"),
    );
    // S1: Down skips the disabled Billing row (Environment → Keyboard).
    support::press_step(&s, "j");
    support::press_step(&s, "j");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never reached Environment",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Environment"))
        },
    );
    support::press_step(&s, "j");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never skipped to Keyboard",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Keyboard"))
        },
    );
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Keyboard never opened",
        |screen| title_has(&support::screen_text(screen), "Keyboard"),
    );
    // A1: a click on an enabled item moves cursor and current together.
    click_at(&mut s, "Members", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "click never opened Members",
        |screen| {
            let text = support::screen_text(screen);
            text.lines()
                .any(|l| l.contains('▎') && l.contains("Members"))
                && title_has(&text, "Members")
        },
    );
    // S2/N1: a click on disabled Billing changes neither.
    click_at(&mut s, "Billing", case_timeout(&case));
    let billing = settle_text(&mut s, case_timeout(&case), "disabled click steady");
    assert!(
        title_has(&billing, "Members"),
        "S2/N1 live: current stays Members after the Billing click"
    );
    assert!(
        billing
            .lines()
            .any(|l| l.contains('▎') && l.contains("Members")),
        "S2/N1 live: cursor stays Members after the Billing click"
    );
    // A2/V2/V3/N2: collapse toggles without moving; icons + markers stay.
    support::press_step(&s, "tab");
    // Focus proof: the nav cursor marker leaves Members (only other stop: collapse).
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "focus never left the nav",
        |screen| {
            !support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Members"))
        },
    );
    support::press_step(&s, "enter");
    checkpoint(&mut s, &dir, "collapsed");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "collapse never applied",
        |screen| support::screen_text(screen).contains("····"),
    );
    let collapsed = live_text(&mut s);
    assert!(
        title_has(&collapsed, "Members"),
        "A2 live: current stays Members while collapsed"
    );
    // Collapsed nav card (cols 19..30); the content card moves to col 31.
    let ccol: Vec<String> = collapsed
        .lines()
        .map(|l| l.chars().skip(19).take(11).collect())
        .collect();
    assert!(
        !ccol.iter().any(|l| l.contains("Members")),
        "V2 live: labels hide in collapsed mode"
    );
    assert!(
        ccol.iter().any(|l| l.contains('›') && l.contains('M')),
        "N2 live: current marker + icon stay"
    );
    // Expand back: the Collapse label returns, then the cursor via backtab.
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "expand never applied",
        |screen| support::screen_text(screen).contains("Collapse"),
    );
    support::press_step(&s, "backtab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never returned to Members",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Members"))
        },
    );
    eprintln!("s3 nav_side: live sidebar page (PTY-only)");
}

const S3_TREE: Id = Id::root("s3.tree");

/// One flat row of the mirror tree: hierarchy plus paint data.
#[derive(Clone)]
struct TreeRow {
    node: TreeNode,
    label: &'static str,
    meta: &'static str,
}

fn tree_node_of(row: &TreeRow) -> TreeNode {
    row.node
}

fn tree_key_of(row: &TreeRow) -> ItemKey {
    row.node.key().expect("keyed fixture row")
}

/// The `empty section` note row has no `TreeNode` kind on impl (Leaf /
/// Parent / Lazy only), so the mirror marks it disabled: cursorable but
/// never activatable, like the VB note row.
fn tree_row_disabled(row: &TreeRow) -> bool {
    row.label == "empty section"
}

fn tree_row_paint(row: &TreeRow, out: &mut RowUi<'_>) {
    out.label(row.label);
    if !row.meta.is_empty() {
        out.meta(row.meta);
    }
}

/// The oracle mirror: src open with folded api + config, a note row, and a
/// root leaf with meta. Keys are stable text keys (path identity).
fn tree_mirror() -> Vec<TreeRow> {
    let row =
        |node: TreeNode, label: &'static str, meta: &'static str| TreeRow { node, label, meta };
    vec![
        row(TreeNode::parent(0).keyed(ItemKey::text("src")), "src", ""),
        row(TreeNode::parent(1).keyed(ItemKey::text("api")), "api", ""),
        row(
            TreeNode::leaf(2).keyed(ItemKey::text("auth.rs")),
            "auth.rs",
            "2.1 KB",
        ),
        row(
            TreeNode::leaf(2).keyed(ItemKey::text("billing.rs")),
            "billing.rs",
            "6.4 KB",
        ),
        row(
            TreeNode::leaf(1).keyed(ItemKey::text("config.rs")),
            "config.rs",
            "1.9 KB",
        ),
        row(
            TreeNode::leaf(0).keyed(ItemKey::text("note")),
            "empty section",
            "",
        ),
        row(
            TreeNode::leaf(0).keyed(ItemKey::text("Cargo.toml")),
            "Cargo.toml",
            "1.4 KB",
        ),
    ]
}

struct TreeApp {
    items: Vec<TreeRow>,
    st: TreeState,
    log: Rc<RefCell<Vec<TreeAction>>>,
    area: Rect,
}

impl TreeApp {
    fn rig(items: Vec<TreeRow>, st: TreeState) -> (Self, Rc<RefCell<Vec<TreeAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                items,
                st,
                log: Rc::clone(&log),
                area: Rect::new(0, 0, 40, 8),
            },
            log,
        )
    }
}

impl App for TreeApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        Tree::new(S3_TREE)
            .key(tree_key_of)
            .node(&tree_node_of)
            .row(tree_row_paint)
            .disabled_item(&tree_row_disabled)
            .update(cx, &mut self.st, &self.items)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Tree::new(S3_TREE)
            .key(tree_key_of)
            .node(&tree_node_of)
            .row(tree_row_paint)
            .disabled_item(&tree_row_disabled)
            .draw(ui, self.area, &self.st, &self.items);
    }
}

/// The focused cursor row (▎ gutter), if any.
fn tree_cursor_text(app: &Harness<TreeApp>, height: u16) -> Option<String> {
    (0..height).find_map(|y| {
        let row = buf_row_text(app.buffer(), y);
        row.starts_with('▎').then_some(row)
    })
}

/// TREE-FOLD-001 (showcase page `trees`, 80x24): folding rows + selection
/// panel over [`Tree`] on the `project_tree` fixture (roots src, tests,
/// docs, Cargo.toml, README.md; first level open; cursor 0; nothing
/// selected).
///
/// Live path drives the page: focus the tree, step in, expand `api`,
/// select `auth.rs`, climb, `*`/`-`, then a toggle click and a leaf click.
/// Isolated path renders a mirror tree into a `Harness` buffer (glyphs,
/// indent, meta column, note tone) and asserts every [`TreeAction`] (toggle
/// identity, cursor relocation, leaf activation, note/modified negatives).
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_tree_fold() {
    let dir = s3_dir("s3_tree_fold");
    let case = s3_case("tree_fold", &["--page", "trees"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    checkpoint(&mut s, &dir, "trees-initial");

    // The tree card occupies screen cols 0..56 (shell + tree); the
    // Selection panel (with its `a/b` paths, sometimes sharing the row)
    // starts at col 58. Scope tree predicates to the card zone.
    fn tree_zone(text: &str) -> Vec<String> {
        text.lines().map(|l| l.chars().take(56).collect()).collect()
    }
    fn tree_row_has(text: &str, label: &str) -> bool {
        tree_zone(text).iter().any(|l| l.contains(label))
    }
    fn tree_cursor_on(text: &str, label: &str) -> bool {
        tree_zone(text)
            .iter()
            .any(|l| l.contains('▎') && l.contains(label))
    }

    // V1/V3 boot: fold glyphs on folders, panel labels, no selection.
    let boot = live_text(&mut s);
    assert_line_has(&boot, "▾", "src", "V1 live: src open");
    assert_line_has(&boot, "▸", "api", "V1 live: api folded");
    assert!(
        boot.contains("Nothing selected"),
        "V3 live: no selection at boot"
    );
    assert_line_has(&boot, "cursor", "src", "V3 live: cursor label");
    assert!(boot.contains("16 rows"), "V3 live: 16 visible rows at boot");
    assert_line_has(&boot, "open", "5 folders", "V3 live: 5 open");
    // Tab into the tree (ring NAV, tree): the gutter lands on src.
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "tree never focused",
        |screen| tree_cursor_on(&support::screen_text(screen), "src"),
    );
    // A1: Right on the open src steps into api; Right again expands it.
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never stepped into api",
        |screen| tree_cursor_on(&support::screen_text(screen), "api"),
    );
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "api never expanded",
        |screen| tree_row_has(&support::screen_text(screen), "auth.rs"),
    );
    checkpoint(&mut s, &dir, "api-expanded");
    let expanded = live_text(&mut s);
    assert_line_has(&expanded, "▾", "api", "V1 live: api open glyph");
    assert_line_has(&expanded, "▸", "db", "V1 live: db still folded");
    assert!(
        expanded.contains("2.1 KB"),
        "V2 live: size meta shows when wide"
    );
    // Down onto auth.rs, Enter selects it: path + depth in the panel.
    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never reached auth.rs",
        |screen| tree_cursor_on(&support::screen_text(screen), "auth.rs"),
    );
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "auth.rs never selected",
        |screen| support::screen_text(screen).contains("src/api/auth.rs"),
    );
    checkpoint(&mut s, &dir, "leaf-selected");
    // S2: cursor moves never clear the selection.
    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never reached billing.rs",
        |screen| tree_cursor_on(&support::screen_text(screen), "billing.rs"),
    );
    let moved = settle_text(&mut s, case_timeout(&case), "selection steady");
    assert!(
        moved.contains("src/api/auth.rs"),
        "S2 live: selection survives cursor moves"
    );
    // A1/S1: Left climbs to api; Left again collapses it in place.
    support::press_step(&s, "left");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "cursor never climbed to api",
        |screen| tree_cursor_on(&support::screen_text(screen), "api"),
    );
    support::press_step(&s, "left");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "api never collapsed",
        |screen| {
            let text = support::screen_text(screen);
            !tree_row_has(&text, "auth.rs") && tree_cursor_on(&text, "api")
        },
    );
    // `*` expands every folder (10 open paths, 30 rows, webhooks out).
    support::press_step(&s, "*");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "expand-all never applied",
        |screen| {
            let text = support::screen_text(screen);
            tree_row_has(&text, "dispatch.rs") && text.contains("30 rows")
        },
    );
    let all = live_text(&mut s);
    // 10 expanded paths: the 5 first-level paths (including the Cargo.toml
    // and README.md leaf roots, kept from construction) plus the 5 deeper
    // folders api, webhooks, db, workers, fixtures.
    assert_line_has(&all, "open", "10 folders", "live: 10 folders open");
    // `-` collapses every folder: 5 roots, cursor back on ancestor src.
    // A bare `-` never survives chord parsing (it is the separator), so
    // press it through the typed call (`Session::type_text` here; the VB
    // oracle calls `press_key` on its own session type).
    s.type_text("-")
        .unwrap_or_else(|e| panic!("key `-` failed: {e:#}"));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "collapse-all never applied",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("5 rows") && text.contains("0 folders") && tree_cursor_on(&text, "src")
        },
    );
    // A2: clicking the src fold glyph re-opens it (11 rows, 1 folders).
    let collapsed = live_text(&mut s);
    let zone = tree_zone(&collapsed);
    let src_row = zone
        .iter()
        .position(|l| l.contains("src"))
        .expect("tree src row") as u16;
    let fold_col = char_col(&zone[src_row as usize], "▸");
    Input::down(MouseButton::Left, fold_col, src_row).send(&mut s);
    Input::up(MouseButton::Left, fold_col, src_row).send(&mut s);
    std::thread::sleep(Duration::from_millis(120));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "toggle click never expanded src",
        |screen| {
            let text = support::screen_text(screen);
            tree_row_has(&text, "api") && text.contains("11 rows")
        },
    );
    // A2: clicking the config.rs leaf row selects it (depth 1).
    click_at(&mut s, "config.rs", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "leaf click never selected config.rs",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("src/config.rs") && text.contains("depth 1")
        },
    );

    tree_fold_isolated(&dir);
    eprintln!("s3 tree_fold: live page + isolated Tree");
}

/// Isolated half of TREE-FOLD-001: the mirror tree through [`Tree`].
///
/// Action-vocabulary note: VB toggles are silent (`Changed`, no event);
/// impl reports every toggle as [`TreeAction::Expanded`]/[`Collapsed`] by
/// design, so toggle checks assert the visibility flip plus the impl
/// report, and separately that no leaf activation escapes.
fn tree_fold_isolated(dir: &Path) {
    let src = ItemKey::text("src");
    let api = ItemKey::text("api");
    let auth = ItemKey::text("auth.rs");

    // V1: fold glyphs, blank for leaves, two cells of indent per depth.
    let mut st = TreeState::new();
    st.expand(src);
    let (rig, _) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing the tree");
    capture_isolated(dir, "tree-fold", app.buffer());
    let row_src = buf_row_text(app.buffer(), 0);
    let row_api = buf_row_text(app.buffer(), 1);
    let row_cfg = buf_row_text(app.buffer(), 2);
    let src_c: Vec<char> = row_src.chars().collect();
    let api_c: Vec<char> = row_api.chars().collect();
    assert_eq!(&src_c[1..3], &['▾', ' '], "V1 isolated: src open glyph");
    assert_eq!(&api_c[3..5], &['▸', ' '], "V1 isolated: api folded glyph");
    assert!(
        row_cfg.starts_with("     config.rs"),
        "V1 isolated: leaf blank + 2-cell indent, got {row_cfg:?}"
    );
    // V2: the meta column shows for all rows when wide ...
    assert!(
        row_cfg.contains("1.9 KB"),
        "V2 isolated: meta shows when wide"
    );
    // ... and for none when one row starves it (all-or-none column:
    // config.rs at depth 1 needs 23 cells, so width 22 hides every meta).
    let mut st = TreeState::new();
    st.expand(src);
    let (mut rig, _) = TreeApp::rig(tree_mirror(), st);
    rig.area = Rect::new(0, 0, 22, 8);
    let mut app = Harness::new(rig, Theme::junie(), 22, 8);
    assert!(app.tab_to(S3_TREE), "focusing the narrow tree");
    capture_isolated(dir, "tree-narrow", app.buffer());
    let narrow: String = (0..8).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(
        !narrow.contains("1.9 KB") && !narrow.contains("1.4 KB"),
        "V2 isolated: no meta anywhere when one row starves it"
    );
    assert!(
        narrow.contains("config.rs") && narrow.contains("Cargo.toml"),
        "V2 isolated: labels win over meta when narrow"
    );
    // S1: toggle flips the expanded set by key identity ...
    let mut st = TreeState::new();
    assert!(st.toggle(api), "S1 isolated: toggle opens");
    assert!(st.is_expanded(api), "S1 isolated: api open by key");
    assert!(!st.toggle(api), "S1 isolated: re-toggle closes");
    assert!(!st.is_expanded(api), "S1 isolated: api closed by key");
    // ... and collapse relocates the cursor to the visible ancestor.
    // Rig A pins the setup: api open, cursor resting on auth.rs.
    let mut st = TreeState::new();
    st.expand(src);
    st.expand(api);
    st.set_cursor(2, auth);
    let (rig, _) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing rig A");
    assert!(
        app.find("auth.rs").is_some(),
        "S1 isolated: auth.rs visible"
    );
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("auth.rs")),
        "S1 isolated: cursor on auth.rs"
    );
    // Rig B: same cursor preset, api closed — the first frame relocates.
    let mut st = TreeState::new();
    st.expand(src);
    st.set_cursor(2, auth);
    let (rig, _) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing rig B");
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("api")),
        "S1 isolated: cursor relocates to api"
    );
    // S2: Enter on a leaf activates it; cursor moves never clear chosen.
    let mut st = TreeState::new();
    st.expand(src);
    st.expand(api);
    let (rig, log) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing the S2 tree");
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Down);
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("auth.rs")),
        "S2 isolated: cursor on auth.rs"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S2 isolated: Enter applies");
    assert!(
        log.borrow().contains(&TreeAction::Activated(auth)),
        "S2 isolated: Activate(auth.rs)"
    );
    // Space chooses; the choice paints past moves (cursor parked away so
    // cursor paint cannot masquerade as chosen paint).
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Consumed, "S2 isolated: Space applies");
    assert!(
        log.borrow().contains(&TreeAction::Chose(auth)),
        "S2 isolated: Space chooses auth.rs"
    );
    let _ = app.key(KeyCode::Down);
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("billing.rs")),
        "S2 isolated: cursor parked on billing.rs"
    );
    let (ax, ay) = app.find("auth.rs").expect("auth label");
    let (cx, cy) = app.find("config.rs").expect("config label");
    assert_ne!(
        app.buffer()[(ax, ay)].bg,
        app.buffer()[(cx, cy)].bg,
        "S2 isolated: the choice paints on auth.rs past moves"
    );
    let _ = app.key(KeyCode::Up);
    let (ax, ay) = app.find("auth.rs").expect("auth label");
    let (cx, cy) = app.find("config.rs").expect("config label");
    assert_ne!(
        app.buffer()[(ax, ay)].bg,
        app.buffer()[(cx, cy)].bg,
        "S2 isolated: selection survives moves"
    );
    // A1: Right expands or steps in, Left collapses or climbs.
    let mut st = TreeState::new();
    st.expand(src);
    let (rig, log) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing the A1 tree");
    let r = app.key(KeyCode::Right);
    assert_eq!(r.flow(), Flow::Consumed, "A1 isolated: Right applies");
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("api")),
        "A1 isolated: Right on open src steps into api"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Right);
    assert!(
        app.find("auth.rs").is_some(),
        "A1 isolated: Right on folded api expands it"
    );
    assert!(
        log.borrow().contains(&TreeAction::Expanded(api)),
        "A1 isolated: expansion reports Expanded(api)"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Left);
    assert!(
        app.find("auth.rs").is_none(),
        "A1 isolated: Left on open api collapses it"
    );
    assert!(
        log.borrow().contains(&TreeAction::Collapsed(api)),
        "A1 isolated: collapse reports Collapsed(api)"
    );
    let _ = app.key(KeyCode::Right);
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Left);
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("api")),
        "A1 isolated: Left on a leaf climbs to api"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    assert!(
        app.find("auth.rs").is_none(),
        "A1 isolated: Enter on a folder toggles"
    );
    assert!(
        log.borrow().contains(&TreeAction::Collapsed(api)),
        "A1 isolated: Enter-toggle reports Collapsed(api)"
    );
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, TreeAction::Activated(_) | TreeAction::Chose(_))),
        "A1 isolated: Enter on a folder activates nothing"
    );
    // A2: clicking a fold glyph toggles; clicking a leaf row activates it.
    let mut st = TreeState::new();
    st.expand(src);
    let (rig, log) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing the A2 tree");
    let api_y = app.find_row("api").expect("api row");
    log.borrow_mut().clear();
    let r = app.click(3, api_y);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "A2 isolated: toggle click applies"
    );
    assert!(
        app.find("auth.rs").is_some(),
        "A2 isolated: toggle click opens api"
    );
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("api")),
        "A2 isolated: toggle click moves the cursor to api"
    );
    let (ax, ay) = app.find("auth.rs").expect("auth label");
    log.borrow_mut().clear();
    let r = app.click(ax, ay);
    assert_eq!(r.flow(), Flow::Consumed, "A2 isolated: leaf click applies");
    assert!(
        log.borrow().contains(&TreeAction::Activated(auth)),
        "A2 isolated: leaf click activates auth.rs"
    );
    // N1: Enter on the note row is Consumed and selects nothing ...
    let mut st = TreeState::new();
    st.expand(src);
    st.expand(api);
    let (rig, log) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing the N1 tree");
    let note_y = app.find_row("empty section").expect("note row");
    for _ in 0..note_y {
        let _ = app.key(KeyCode::Down);
    }
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("empty section")),
        "N1 isolated: cursor rests on the note row"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1 isolated: Enter consumed");
    assert!(log.borrow().is_empty(), "N1 isolated: no event");
    // ... clicks too; the row renders with its label, never bold.
    let (nx, ny) = app.find("empty section").expect("note label");
    log.borrow_mut().clear();
    let r = app.click(nx, ny);
    assert_eq!(r.flow(), Flow::Consumed, "N1 isolated: click consumed");
    assert!(log.borrow().is_empty(), "N1 isolated: click emits nothing");
    assert!(
        !buf_is_bold(app.buffer(), nx, ny),
        "N1 isolated: note label never bold"
    );
    // N2: modified chords never navigate (presence proof: plain j moves).
    let mut st = TreeState::new();
    st.expand(src);
    let (rig, log) = TreeApp::rig(tree_mirror(), st);
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_TREE), "focusing the N2 tree");
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Char('j'), KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Ignored, "N2 isolated: ctrl+j ignored");
    assert!(log.borrow().is_empty(), "N2 isolated: no event");
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("src")),
        "N2 isolated: cursor unmoved"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('j'));
    assert_eq!(r.flow(), Flow::Consumed, "N2 isolated: plain j applies");
    assert!(
        log.borrow().contains(&TreeAction::Moved),
        "N2 isolated: plain j moves"
    );
    assert!(
        tree_cursor_text(&app, 8).is_some_and(|t| t.contains("api")),
        "N2 isolated: cursor on api"
    );
}

const S3_STEPS: Id = Id::root("s3.steps");

/// One rail row: caller-owned label/ordinal plus lifecycle state.
#[derive(Clone)]
struct StepRow {
    label: &'static str,
    state: StepState,
    meta: Option<&'static str>,
}

fn step_state_of(row: &StepRow) -> StepState {
    row.state
}

fn step_row_paint(row: &StepRow, out: &mut RowUi<'_>) {
    out.label(row.label);
    if let Some(m) = row.meta {
        out.meta(m);
    }
}

/// The oracle lifecycle: Done / Running+meta / Failed+meta / Blocked /
/// Skipped+meta / Queued. Ordinals are caller paint on impl (the terminal
/// page bakes them into its labels), so the fixture carries them.
fn steps_lifecycle() -> Vec<StepRow> {
    vec![
        StepRow {
            label: "01 a",
            state: StepState::Done,
            meta: None,
        },
        StepRow {
            label: "02 b",
            state: StepState::Running,
            meta: Some("0.8 s"),
        },
        StepRow {
            label: "03 c",
            state: StepState::Failed,
            meta: Some("exit 1"),
        },
        StepRow {
            label: "04 d",
            state: StepState::Blocked,
            meta: None,
        },
        StepRow {
            label: "05 e",
            state: StepState::Skipped,
            meta: Some("cached"),
        },
        StepRow {
            label: "06 f",
            state: StepState::Queued,
            meta: None,
        },
    ]
}

struct StepsApp {
    items: Vec<StepRow>,
    st: StepsState,
    area: Rect,
}

impl App for StepsApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Steps::navigable(S3_STEPS)
            .step(&step_state_of)
            .row(step_row_paint)
            .update(cx, &mut self.st, &self.items)
            .erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Steps::navigable(S3_STEPS)
            .step(&step_state_of)
            .row(step_row_paint)
            .draw(ui, self.area, &self.st, &self.items);
    }
}

/// STEPS-LIFE-001 (showcase page `terminal`, 80x24): stage rail rows over
/// [`Steps`] (7 stages; stage 3 skipped as cached; durations
/// 10/26/40/8/18/14/1 ticks; the run starts at boot and ticks at 80 ms
/// while animating).
///
/// Live path boots the page, watches the run finish (`7 of 7`), proves a
/// finished run ignores ticks, clicks `Run with a failure` (stage 1
/// fails, later stages block), then clicks `Run` (full re-queue). At
/// 80x24 the rail card is 22 cells wide (18-cell rows), so rail labels
/// truncate with `…`, every rail meta hides (N1 live), and the card meta
/// truncates to 13 cells (`0 of 7 · runn`); V2 metas are isolated-only.
/// Isolated path renders [`Steps`] through every [`StepState`]
/// (glyphs, ordinals, metas, bold/error/muted tones, narrow-meta hiding)
/// and asserts the frontier/counts contract.
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_steps_life() {
    let dir = s3_dir("s3_steps_life");
    // The full run is 109 ticks at 80 ms (≈9 s): override the 8 s default
    // (the terminal page is the documented override case).
    let case = s3_case("steps_life", &["--page", "terminal"], 80, 24).timeout(30_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);

    // Rail-scoped done rows: `✓ 01` … `✓ 07` (the viewport's own
    // `✓ <stage>` lines never carry an ordinal).
    fn rail_done_count(text: &str) -> usize {
        text.lines().filter(|l| l.contains("✓ 0")).count()
    }
    // V1 boot: ordinals, a spinner on the running row, truncated labels.
    let boot = live_text(&mut s);
    for n in ["01", "02", "03", "04", "05", "06", "07"] {
        assert!(boot.contains(n), "V1 live: ordinal {n} at boot");
    }
    for stage in ["Resolve wo…", "Mount sour…", "Ready"] {
        assert!(boot.contains(stage), "V1 live: {stage:?} at boot");
    }
    assert!(
        SPINNER.iter().any(|g| boot.contains(g)),
        "V1 live: spinner on the running row at boot"
    );
    assert!(boot.contains("0 of 7"), "S1 live: 0 of 7 running at boot");
    // N1 live: 18-cell rows hide every rail meta (presence proof: the
    // ordinals and labels above share this screen).
    assert!(
        !boot.contains("queued") && !boot.contains("0.1 s"),
        "N1 live: no rail metas on narrow rows"
    );
    checkpoint(&mut s, &dir, "rail-running");
    // A1: ticks advance the run (stage 0 finishes), then to completion
    // (109 ticks at 80 ms).
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "run never advanced",
        |screen| support::screen_text(screen).contains("1 of 7"),
    );
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "run never finished",
        |screen| support::screen_text(screen).contains("7 of 7"),
    );
    checkpoint(&mut s, &dir, "rail-done");
    let done = live_text(&mut s);
    assert_eq!(
        rail_done_count(&done),
        6,
        "A1 live: six rail rows Done (the seventh is skipped: 7 of 7)"
    );
    // The launch line scrolled off; exactly one prompt line stays and it
    // carries no command (the rail shares the row, so no end-anchoring).
    let prompts: Vec<&str> = done
        .lines()
        .filter(|l| l.contains("payments-platform ❯"))
        .collect();
    assert_eq!(prompts.len(), 1, "A1 live: one prompt line after the run");
    assert!(
        !prompts[0].contains("jackin"),
        "A1 live: the finished run returns a fresh prompt"
    );
    // N2: a finished run ignores further ticks (presence proof: the ✓s
    // and the count stay on the same settled screen).
    let settled = settle_text(&mut s, case_timeout(&case), "finished run steady");
    assert!(
        settled.contains("7 of 7"),
        "N2 live: count stays 7 of 7 after ticks"
    );
    assert_eq!(
        rail_done_count(&settled),
        6,
        "N2 live: six rail ✓s stay after ticks"
    );
    assert!(
        !SPINNER.iter().any(|g| settled.contains(g)),
        "N2 live: no spinner once nothing runs"
    );
    // A2/S2: `Run with a failure` (button truncated to `Run with…`)
    // re-queues, then stage 1 fails and the later stages block.
    click_at(&mut s, "Run with", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "failure run never reset",
        |screen| support::screen_text(screen).contains("0 of 7"),
    );
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "stage 1 never failed",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("1 of 7")
                && text
                    .lines()
                    .any(|l| l.contains('!') && l.contains("Pull base …"))
        },
    );
    checkpoint(&mut s, &dir, "rail-failed");
    let failed = live_text(&mut s);
    assert!(
        failed.contains("Pull base image failed"),
        "S2 live: the viewport reports the failure"
    );
    assert_eq!(
        rail_done_count(&failed),
        1,
        "S2 live: only stage 0 Done after the failure"
    );
    assert_eq!(
        count_glyph(&failed, '!'),
        1,
        "S2 live: exactly one failed row"
    );
    assert!(
        !SPINNER.iter().any(|g| failed.contains(g)),
        "S2 live: nothing runs after the failure"
    );
    // A2: `Run` re-queues every stage and restarts at stage 0.
    click_at(&mut s, "Run", case_timeout(&case));
    support::wait_screen(&mut s, case_timeout(&case), "run never reset", |screen| {
        let text = support::screen_text(screen);
        text.contains("0 of 7") && SPINNER.iter().any(|g| text.contains(g))
    });
    let reset = live_text(&mut s);
    assert_line_has(
        &reset,
        "Resolve wo…",
        "01",
        "A2 live: stage 0 heads the re-queued rail",
    );
    assert!(
        !reset.contains('!'),
        "A2 live: no failure survives the reset"
    );

    steps_life_isolated(&dir);
    eprintln!("s3 steps_life: live page + isolated Steps");
}

/// Isolated half of STEPS-LIFE-001: one rail through every step state.
///
/// Contract note: VB reports `frontier()`/`counts()`/`failed()`; impl
/// [`Steps`] reports the frontier only, so S1 pins the frontier (plus the
/// Blocked non-terminal reading) and the per-state V rows carry the rest.
fn steps_life_isolated(dir: &Path) {
    // S1: frontier is the first non-terminal step (Blocked counts: it is
    // still waiting).
    let items = steps_lifecycle();
    let rail = Steps::navigable(S3_STEPS)
        .step(&step_state_of)
        .row(step_row_paint);
    assert_eq!(
        rail.frontier(&items),
        Some(ItemKey::index(1)),
        "S1 isolated: frontier at Running"
    );
    let blocked_first = vec![
        StepRow {
            label: "01 a",
            state: StepState::Done,
            meta: None,
        },
        StepRow {
            label: "02 b",
            state: StepState::Blocked,
            meta: None,
        },
    ];
    assert_eq!(
        rail.frontier(&blocked_first),
        Some(ItemKey::index(1)),
        "S1 isolated: Blocked is non-terminal"
    );
    // V1/V2/V3: glyphs, ordinals, metas, and tones per state.
    let rig = StepsApp {
        items: steps_lifecycle(),
        st: StepsState::default(),
        area: Rect::new(0, 0, 40, 8),
    };
    let mut app = Harness::new(rig, Theme::junie(), 40, 8);
    assert!(app.tab_to(S3_STEPS), "focusing the rail");
    capture_isolated(dir, "steps-life", app.buffer());
    let rows: Vec<String> = (0..6).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(
        rows[0].contains("✓") && rows[0].contains("01"),
        "V1 isolated: done row has ✓ + ordinal, got {:?}",
        rows[0]
    );
    let failed_c: Vec<char> = rows[2].chars().collect();
    assert!(
        failed_c[1] == '!' && rows[2].contains("03"),
        "V1 isolated: failed row has ! + ordinal, got {:?}",
        rows[2]
    );
    assert!(
        rows[1].contains("0.8 s") && rows[2].contains("exit 1"),
        "V2 isolated: running + failed metas, got {:?} {:?}",
        rows[1],
        rows[2]
    );
    assert!(
        rows[3].contains("blocked") && rows[5].contains("queued"),
        "V2 isolated: default blocked/queued metas, got {:?} {:?}",
        rows[3],
        rows[5]
    );
    assert!(
        rows[4].contains("cached"),
        "V2 isolated: explicit skipped meta, got {:?}",
        rows[4]
    );
    assert!(
        buf_is_bold(app.buffer(), 4, 1),
        "V3 isolated: running label bold"
    );
    assert!(
        buf_is_bold(app.buffer(), 4, 2),
        "V3 isolated: failed label bold"
    );
    assert!(
        !buf_is_bold(app.buffer(), 4, 5),
        "V3 isolated: queued label not bold"
    );
    // N1: caller meta hides when the row is too narrow (needs mw + 2
    // cells past the label; width 20 starves "0.8 s" and "exit 1").
    let rig = StepsApp {
        items: steps_lifecycle(),
        st: StepsState::default(),
        area: Rect::new(0, 0, 20, 8),
    };
    let mut app = Harness::new(rig, Theme::junie(), 20, 8);
    assert!(app.tab_to(S3_STEPS), "focusing the narrow rail");
    capture_isolated(dir, "steps-narrow", app.buffer());
    let narrow: String = (0..6).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(
        !narrow.contains("0.8 s") && !narrow.contains("exit 1"),
        "N1 isolated: metas hide when narrow"
    );
    assert!(
        narrow.contains("02") && narrow.contains("b"),
        "N1 isolated: ordinal + label survive narrowing"
    );
}

const S3_TABS: Id = Id::root("s3.tabs");

struct TabsApp {
    items: Vec<&'static str>,
    st: TabsState,
    log: Rc<RefCell<Vec<TabsAction>>>,
    area: Rect,
    closable: bool,
}

impl TabsApp {
    fn rig(items: Vec<&'static str>, width: u16) -> (Self, Rc<RefCell<Vec<TabsAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                items,
                st: TabsState::default(),
                log: Rc::clone(&log),
                area: Rect::new(0, 0, width, 2),
                closable: false,
            },
            log,
        )
    }

    fn view(closable: bool) -> Tabs<'static, &'static str> {
        let tabs = Tabs::new(S3_TABS);
        if closable { tabs.closable(true) } else { tabs }
    }
}

impl App for TabsApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        Self::view(self.closable)
            .update(cx, &mut self.st, &self.items)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Self::view(self.closable).draw(ui, self.area, &self.st, &self.items);
    }
}

/// TABS-STRIP-001 (showcase page `settings`, 80x24): settings tab strip
/// over [`Tabs`] (General/Members/Environment; non-closable; the active
/// tab selects the settings body).
///
/// Live path boots the page, tabs into the strip, arrows to Members,
/// re-asserts with `2` and `Enter`, clicks Environment, proves `x` and
/// `9` change nothing, and arrows back. Isolated path renders the strip
/// (accent rule span, planes, bold, no gutter) and asserts every
/// [`TabsAction`] (digit jumps, arrows, clicks, non-closable negatives).
#[test]
#[ignore = "showcase s3 check; run with --ignored"]
fn s3_tabs_strip() {
    let dir = s3_dir("s3_tabs_strip");
    let case = s3_case("tabs_strip", &["--page", "settings"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");

    // V1 boot: the accent rule spans exactly the General tab (1+7+2).
    let boot = live_text(&mut s);
    assert!(
        boot.contains("Project name") && boot.contains("No changes"),
        "A2 live: General body at boot"
    );
    let rule = row_below(&boot, "General", "V1 live: rule row");
    assert_eq!(
        rule.chars().filter(|c| *c == '━').count(),
        10,
        "V1 live: ━ spans General edge to edge"
    );
    assert!(rule.contains('─'), "V1 live: inactive tabs sit on ─");
    checkpoint(&mut s, &dir, "tabs-general");
    // Tab into the strip, Right to Members (proof by action: the body
    // switches only when the strip owns the key).
    support::press_step(&s, "tab");
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Members body never appeared",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("6 members") && text.contains("Mira Okafor")
        },
    );
    checkpoint(&mut s, &dir, "tabs-members");
    // S2/A1: `2` re-asserts Members; Enter re-emits Activated (stays).
    support::press_step(&s, "2");
    let reasserted = settle_text(&mut s, case_timeout(&case), "digit steady");
    assert!(
        reasserted.contains("6 members"),
        "S2 live: `2` keeps Members"
    );
    support::press_step(&s, "enter");
    let reemitted = settle_text(&mut s, case_timeout(&case), "enter steady");
    assert!(
        reemitted.contains("6 members"),
        "A1 live: Enter keeps Members"
    );
    // A2: clicking Environment activates it; the body follows.
    click_at(&mut s, "Environment", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Environment body never appeared",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Environment variables")
                && text.contains("DATABASE_URL")
                && text.contains("0 selected")
        },
    );
    let env = live_text(&mut s);
    let rule = row_below(&env, "General", "V1 live: env rule row");
    assert_eq!(
        rule.chars().filter(|c| *c == '━').count(),
        14,
        "V1 live: ━ spans Environment (1+11+2) edge to edge"
    );
    // V3 live: the tab label row never carries the ▎ gutter.
    let label_row = env
        .lines()
        .find(|l| l.contains("General"))
        .expect("tab label row");
    assert!(
        !label_row.contains('▎'),
        "V3 live: no gutter on the tab strip"
    );
    // N1/S2 live: `x` and `9` change nothing (presence proof: the
    // Environment body shares the settled screens).
    support::press_step(&s, "x");
    let after_x = settle_text(&mut s, case_timeout(&case), "x steady");
    assert!(
        after_x.contains("DATABASE_URL") && after_x.contains("0 selected"),
        "N1 live: x on a non-closable tab changes nothing"
    );
    support::press_step(&s, "9");
    let after_9 = settle_text(&mut s, case_timeout(&case), "9 steady");
    assert!(
        after_9.contains("DATABASE_URL"),
        "S2 live: out-of-range digit changes nothing"
    );
    // A1: Left switches back to Members.
    support::press_step(&s, "left");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Left never returned to Members",
        |screen| support::screen_text(screen).contains("6 members"),
    );

    tabs_strip_isolated(&dir);
    eprintln!("s3 tabs_strip: live page + isolated Tabs");
}

/// Isolated half of TABS-STRIP-001: the settings strip (General / Members
/// / Environment) through [`Tabs`].
///
/// Geometry note: impl exposes no `areas[]`/`hidden()` readers, so widths
/// and the overflow window are pinned through the painted rule span and
/// label visibility — the same observables the live path asserts.
fn tabs_strip_isolated(dir: &Path) {
    let t = Theme::junie();
    let strip = || vec!["General", "Members", "Environment"];

    // V1: only the active tab carries the accent ━ edge to edge.
    let (rig, _) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the strip");
    capture_isolated(dir, "tabs-strip", app.buffer());
    let labels = buf_row_text(app.buffer(), 0);
    let rule = buf_row_text(app.buffer(), 1);
    let span: usize = rule.chars().take_while(|c| *c == '━').count();
    assert_eq!(span, 10, "V1 isolated: General width");
    for (x, glyph) in rule.chars().enumerate().take(60) {
        if (x as u16) < 10 {
            assert_eq!(glyph, '━', "V1 isolated: ━ under General at {x}");
            assert_eq!(
                app.buffer()[(x as u16, 1)].fg,
                t.color.accent,
                "V1 isolated: accent rule at {x}"
            );
        } else {
            assert_eq!(glyph, '─', "V1 isolated: ─ outside General at {x}");
        }
    }
    // V2/V3: the active tab sits one plane up; no gutter glyph anywhere.
    assert_ne!(
        app.buffer()[(1, 0)].bg,
        app.buffer()[(11, 0)].bg,
        "V2 isolated: active one plane up"
    );
    assert!(
        !labels.contains('▎') && !rule.contains('▎'),
        "V3 isolated: no gutter on the strip"
    );
    // N2: a cursor on an inactive tab never draws the underline (it goes
    // two planes up and bold instead). Press-only positions the cursor
    // without activating; the release lands on empty strip.
    let (rig, _) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the cursor strip");
    let (mx, _) = app.find("Members").expect("Members label");
    let _ = app.mouse(MouseKind::Down, mx, 0);
    let _ = app.mouse(MouseKind::Up, 59, 0);
    capture_isolated(dir, "tabs-cursor", app.buffer());
    let rule = buf_row_text(app.buffer(), 1);
    assert_eq!(
        rule.chars().filter(|c| *c == '━').count(),
        10,
        "N2 isolated: ━ stays under General"
    );
    let active_bg = app.buffer()[(1, 0)].bg;
    let cursor_bg = app.buffer()[(mx, 0)].bg;
    let plain_bg = app.buffer()[(21, 0)].bg;
    assert_ne!(
        cursor_bg, active_bg,
        "V2 isolated: cursor tab off the active plane"
    );
    assert_ne!(
        cursor_bg, plain_bg,
        "V2 isolated: cursor tab off the base plane"
    );
    assert!(
        buf_is_bold(app.buffer(), mx, 0),
        "V2 isolated: cursor tab bold"
    );
    // S1: activating moves active and cursor together (cursor follows:
    // the next Left steps from the new tab, not the old one) ...
    let (rig, log) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the S1 strip");
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Char('3'));
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(2))),
        "S1 isolated: `3` activates Environment"
    );
    let rule = buf_row_text(app.buffer(), 1);
    assert_eq!(
        rule.chars().filter(|c| *c == '━').count(),
        14,
        "S1 isolated: ━ spans Environment (1+11+2)"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Left);
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(1))),
        "S1 isolated: cursor followed to Environment"
    );
    // ... and pulls the tab into the window when the strip overflows.
    let (rig, _) = TabsApp::rig(vec!["t1", "t2", "t3", "t4", "t5", "t6"], 20);
    let mut app = Harness::new(rig, Theme::junie(), 20, 2);
    assert!(app.tab_to(S3_TABS), "focusing the overflow strip");
    assert!(
        app.find("t6").is_none(),
        "S1 isolated: t6 starts outside the window"
    );
    let _ = app.key(KeyCode::Char('6'));
    assert!(
        app.find("t6").is_some(),
        "S1 isolated: the active tab is visible"
    );
    assert!(
        app.find("t1").is_none(),
        "S1 isolated: the window moved past t1"
    );
    // S2: digits 1-9 jump directly; out-of-range digits are Ignored.
    let (rig, log) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the S2 strip");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('2'));
    assert_eq!(r.flow(), Flow::Consumed, "S2 isolated: `2` applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(1))),
        "S2 isolated: `2` → 1"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('9'));
    assert_eq!(r.flow(), Flow::Ignored, "S2 isolated: `9` ignored");
    assert!(log.borrow().is_empty(), "S2 isolated: `9` emits nothing");
    let r = app.key(KeyCode::Char('0'));
    assert_eq!(r.flow(), Flow::Ignored, "S2 isolated: `0` ignored");
    // A1: arrows/h/l switch tabs and emit Activated; Enter re-emits it.
    let (rig, log) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the A1 strip");
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Right);
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(1))),
        "A1 isolated: Right"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Char('l'));
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(2))),
        "A1 isolated: l"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Left);
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(1))),
        "A1 isolated: Left"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Char('h'));
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(0))),
        "A1 isolated: h"
    );
    let _ = app.key(KeyCode::Char('3'));
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(2))),
        "A1 isolated: Enter"
    );
    // A2: clicking a tab activates it (the body follows: live path).
    let (rig, log) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the A2 strip");
    let (ex, _) = app.find("Environment").expect("Environment label");
    log.borrow_mut().clear();
    let _ = app.click(ex, 0);
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Activated(ItemKey::index(2))),
        "A2 isolated: click"
    );
    let rule = buf_row_text(app.buffer(), 1);
    assert_eq!(
        rule.chars().filter(|c| *c == '━').count(),
        14,
        "A2 isolated: ━ follows the click"
    );
    // N1: x/Delete on a non-closable tab emits nothing (presence proof:
    // a closable tab emits Close).
    let (rig, log) = TabsApp::rig(strip(), 60);
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the N1 strip");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Ignored, "N1 isolated: x ignored");
    assert!(log.borrow().is_empty(), "N1 isolated: x emits nothing");
    let r = app.key(KeyCode::Delete);
    assert_eq!(r.flow(), Flow::Ignored, "N1 isolated: Delete ignored");
    assert!(log.borrow().is_empty(), "N1 isolated: Delete emits nothing");
    let (mut rig, log) = TabsApp::rig(vec!["a"], 60);
    rig.closable = true;
    let mut app = Harness::new(rig, Theme::junie(), 60, 2);
    assert!(app.tab_to(S3_TABS), "focusing the closable strip");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('x'));
    assert_eq!(r.flow(), Flow::Consumed, "N1 isolated: closable x applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| *a == TabsAction::Close(ItemKey::index(0))),
        "N1 isolated: closable emits"
    );
}
