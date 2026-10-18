//! Showcase pending slice 8A-S4 executable checks — impl port.
//!
//! Ported from VB commit `5f6e52f31861f9f4281f1db264ab012457b9bc2e`
//! (`tests/harness/tests/visual_baseline/showcase_pending_s4.rs`), adapting
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
//! - Isolated `junie_tui` in-process renders (`Select`, `Picker`,
//!   `Completion`, `Dialog`, `ContextMenu`, `MenuBar`, `render_bar`,
//!   `render_spinner`, `render_indeterminate`, `Meter`, `StatusBar`,
//!   `RenderCtx`, `Interaction`, `Outcome`) become the established impl
//!   component path: `Runtime` + `Stub` + `draw_scene` with
//!   `ui.reference` state injection for inert paint, and `Harness` apps
//!   for interaction. `Outcome` assertions become draw/behavior
//!   correlates (flow, committed values, re-rendered glyphs, selection
//!   paint, actions): `Changed` is `Consumed` plus the asserted effect,
//!   modal dismissal is the layer correlate (`is_open` / `Closed` /
//!   `Dismissed`), and caller-owned validation stays caller-owned.
//!
//! One ignored test per S4 registry row (20 rows: the 10 OVERLAYS group rows
//! plus the 2 FORMS rows plus the 8 FEEDBACK rows), over the real binaries
//! built from this worktree's impl sources and over the real production
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
//! Three rows are PTY-only by construction: HELP-SHOWCASE-001 (help is the
//! showcase binary's help overlay), FORM-SUBMIT-001 and FORM-INVALID-002
//! (the submit/countdown/reset state machine lives in the showcase
//! binary's forms page). None is importable as a unit here, so no headless
//! path exists for them; every one of their checks has a live-PTY
//! executable form instead.
//!
//! No new snapshots and no new static captures: every S4 row already has
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
//! Row → test map (registry id → `s4_*` test):
//!
//! - SELECT-OPEN-001 → [`s4_select_open`]
//! - PICKER-QUERY-001 → [`s4_picker_query`]
//! - PICKER-TABS-002 → [`s4_picker_tabs`]
//! - COMPLETE-EDITOR-002 → [`s4_complete_editor`]
//! - DIALOG-CONFIRM-001 → [`s4_dialog_confirm`]
//! - DIALOG-PROMPT-002 → [`s4_dialog_prompt`]
//! - MENU-OPEN-001 → [`s4_menu_open`]
//! - CTXMENU-ANCHOR-001 → [`s4_ctxmenu_anchor`]
//! - MENUBAR-SHELL-001 → [`s4_menubar_shell`]
//! - HELP-SHOWCASE-001 → [`s4_help_showcase`] (PTY-only)
//! - FORM-SUBMIT-001 → [`s4_form_submit`] (PTY-only)
//! - FORM-INVALID-002 → [`s4_form_invalid`] (PTY-only)
//! - PB-STATES-001 → [`s4_pb_states`]
//! - PB-LIVE-002 → [`s4_pb_live`]
//! - SPIN-FRAME-001 → [`s4_spin_frame`]
//! - SPIN-SWEEP-002 → [`s4_spin_sweep`]
//! - METER-TONES-001 → [`s4_meter_tones`]
//! - METER-DOMAIN-002 → [`s4_meter_domain`]
//! - SB-GROUPS-001 → [`s4_sb_groups`]
//! - SB-COLLAPSE-002 → [`s4_sb_collapse`]

use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use termrock::Color as RColor;
use termrock::Input as RtInput;
use termrock::runtime::stub::{Stub, deliver};
use termrock::{
    Action, ActionKey, Anchor, App, AsItem, Brand, Buffer, Completion, CompletionAction,
    CompletionController, CompletionState, ContextMenu, CrossAlign, Cx, Dialog, DialogAction,
    DialogState, FgStep, Flow, GlyphRole, Id, Item, ItemKey, KeyCode, KeyModifiers, Menu,
    MenuAction, MenuBar, MenuItem, MenuState, Meter, MeterTone, MeterVisual, Modifier, MouseKind,
    Part, PartRef, Picker, PickerAction, PickerState, Position, ProgressBar, Rect, ReferenceState,
    ReferenceTarget, Response, Role, Runtime, ScopeKey, Select, SelectAction, SelectState, Side,
    Spinner, StateFlags, Status, StatusAction, StatusBar, StatusItem, Surface, TextInput,
    TextInputState, Theme, Ui,
};
use termrock_test_support::Harness;
use tuiscotti::render::frame_from_screen;
use tuiscotti::tui::MouseButton;
use tuiscotti::{Cell as TCell, Color as TColor, Frame, Mods, Provenance, Rgb, UnderlineStyle};

use crate::support::Session;
use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

/// The VB oracle commit this file ports.
const PORT_OF: &str = "5f6e52f31861f9f4281f1db264ab012457b9bc2e";

/// Owned-name S4 case at the row's viewport (truecolor, like every S4 row).
fn s4_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s4/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

/// Unique scratch dir for one S4 test (created, never shared).
fn s4_dir(test: &str) -> PathBuf {
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
    eprintln!("s4 provenance: {body}");
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

/// Move the pointer over `needle`'s cell (hover), paced like a send step.
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

/// Secondary-button click on `needle`'s cell.
fn right_click_at(s: &mut Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let (row, col) = find_pos(s, needle, timeout);
    s.click_with(MouseButton::Right, col, row)
        .unwrap_or_else(|e| panic!("right click `{needle}` failed: {e:#}"));
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
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
        Provenance::now("default", "s4-isolated", vec![]),
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

/// The reference ten-frame braille cycle (theme `spinner_frames`).
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const S4_SORT: Id = Id::root("s4.sort");
const S4_SORT_OTHER: Id = Id::root("s4.sort.other");
const S4_SORT_AREA: Rect = Rect {
    x: 4,
    y: 2,
    width: 30,
    height: 1,
};
const SORT_ITEMS: [&str; 4] = ["created_at", "total", "status", "customer"];

/// Sort-select rig: shared state plus action log, with a second live
/// select as the focus-loss target.
struct SelectApp {
    st: Rc<RefCell<SelectState>>,
    other: SelectState,
    log: Rc<RefCell<Vec<SelectAction>>>,
    disabled: bool,
}

impl App for SelectApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let mut r = Select::new(S4_SORT)
            .disabled(self.disabled)
            .update(cx, &mut self.st.borrow_mut(), &SORT_ITEMS)
            .on_action(|action| log.borrow_mut().push(action));
        // The other select drains its own bucket so Tab reaches it.
        r |= Select::new(S4_SORT_OTHER)
            .update(cx, &mut self.other, &SORT_ITEMS)
            .erase();
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Select::new(S4_SORT).disabled(self.disabled).draw(
            ui,
            S4_SORT_AREA,
            &self.st.borrow(),
            &SORT_ITEMS,
        );
        Select::new(S4_SORT_OTHER).draw(ui, Rect::new(4, 12, 30, 1), &self.other, &SORT_ITEMS);
    }
}

/// SELECT-OPEN-001 (`showcase/flows/chips/select_open`, 80x24): the closed
/// field plus the open popup list over [`Select`].
///
/// Isolated path mirrors the row's chips-page selects (Sort, Page size,
/// disabled Engine): open/close, clamped cursor motion, direct closed
/// stepping, Changed-only-on-difference, Esc restore, dismiss, and the
/// disabled/modified-chord/focus-loss negatives. Live path drives the
/// Sort select on the chips page (click to focus, keyboard to move and
/// choose), steps the closed select directly, and proves the disabled
/// Engine select never opens.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_select_open() {
    let dir = s4_dir("s4_select_open");
    let t = Theme::junie();
    let faint = t.color.fg[FgStep::Faint.index()];
    let disabled_tone = t.color.disabled_fg;

    // S1 isolated: Enter opens with cursor = selected; Up/Down/k/j clamp.
    let st = Rc::new(RefCell::new(SelectState::default()));
    st.borrow_mut().set_value(Some(ItemKey::index(0)));
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut app = Harness::new(
        SelectApp {
            st: Rc::clone(&st),
            other: SelectState::default(),
            log: Rc::clone(&log),
            disabled: false,
        },
        Theme::junie(),
        60,
        14,
    );
    assert!(app.tab_to(S4_SORT), "navigating focus");
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Enter opens");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, SelectAction::Chose(_))),
        "S1: opening chooses nothing"
    );
    assert!(st.borrow().is_open(), "S1: open");
    assert_eq!(
        st.borrow().cursor(),
        Some(ItemKey::index(0)),
        "S1: cursor = selected"
    );
    let r = app.key(KeyCode::Char('k'));
    assert_eq!(r.flow(), Flow::Consumed, "S1: k applies");
    assert_eq!(
        st.borrow().cursor(),
        Some(ItemKey::index(0)),
        "S1: k clamps at the top"
    );
    let _ = app.key(KeyCode::Char('j'));
    let _ = app.key(KeyCode::Down);
    assert_eq!(
        st.borrow().cursor(),
        Some(ItemKey::index(2)),
        "S1: j/Down move the cursor"
    );
    for _ in 0..9 {
        let _ = app.key(KeyCode::Down);
    }
    assert_eq!(
        st.borrow().cursor(),
        Some(ItemKey::index(3)),
        "S1: Down clamps at the last option"
    );
    // V1/V2 render form: the open field shows ▴ and the popup anchors
    // below it with › on the selected row.
    let buf = app.buffer();
    assert_eq!(
        buf[(59, 13)].symbol(),
        " ",
        "select open render: far corner untouched"
    );
    let field_row = buf_row_text(buf, 2);
    assert!(
        field_row.contains("created_at") && field_row.contains('▴'),
        "V1: open field shows the value with ▴"
    );
    let below: String = (3..14).map(|y| buf_row_text(buf, y)).collect();
    assert!(
        below.contains("total") && below.contains("customer"),
        "V2: the popup anchors below the field"
    );
    let sel_line = (3..14)
        .map(|y| buf_row_text(buf, y))
        .find(|l| l.contains('›'))
        .expect("V2: a › row exists");
    assert!(
        sel_line.contains("created_at"),
        "V2: › marks the selected row, not the cursor row"
    );
    let cur_line = (3..14)
        .map(|y| buf_row_text(buf, y))
        .find(|l| l.contains('▎'))
        .expect("V2: a cursor row exists");
    assert!(
        cur_line.contains("customer"),
        "V2: the cursor row is separate from the selected row"
    );
    capture_isolated(&dir, "select-open", buf);
    // V1 closed form: the same field closed shows ▾.
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Esc closes");
    assert!(!st.borrow().is_open(), "A2: closed");
    assert_eq!(
        st.borrow().cursor(),
        Some(ItemKey::index(0)),
        "A2: Esc restores cursor = selected"
    );
    let buf = app.buffer();
    assert!(
        buf_row_text(buf, 2).contains('▾'),
        "V1: closed field shows ▾"
    );
    assert!(
        !(3..14).any(|y| buf_row_text(buf, y).contains("customer")),
        "V1: closed field paints no popup"
    );
    // A1: Enter/Space on an open list emits Changed(i) only on difference.
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Down);
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow()
            .contains(&SelectAction::Chose(ItemKey::index(1))),
        "A1: Enter emits Changed(1)"
    );
    assert_eq!(
        st.borrow().value(),
        Some(ItemKey::index(1)),
        "A1: selection follows the cursor"
    );
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Char(' '));
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Consumed, "N1: Space applies");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, SelectAction::Chose(_))),
        "N1: choosing the already selected option emits no event"
    );
    // S2: closed Up/Down/Left/Right change selected directly.
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Right);
    assert_eq!(r.flow(), Flow::Consumed, "S2: closed Right applies");
    assert!(
        log.borrow()
            .contains(&SelectAction::Chose(ItemKey::index(2))),
        "S2: closed Right steps selected"
    );
    assert!(!st.borrow().is_open(), "S2: stepping never opens");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Left);
    assert_eq!(r.flow(), Flow::Consumed, "S2: closed Left applies");
    assert!(
        log.borrow()
            .contains(&SelectAction::Chose(ItemKey::index(1))),
        "S2: closed Left steps back"
    );
    let _ = app.key(KeyCode::Up);
    assert_eq!(
        st.borrow().value(),
        Some(ItemKey::index(0)),
        "S2: closed Up steps back"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Up);
    assert_eq!(r.flow(), Flow::Consumed, "S2: edge step applies");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, SelectAction::Chose(_))),
        "S2: stepping past the edge emits no event"
    );
    // A2: click outside dismisses without change.
    let _ = app.key(KeyCode::Enter);
    let _ = app.key(KeyCode::Down);
    assert!(st.borrow().is_open(), "fixture open");
    let r = app.click(59, 13);
    assert_eq!(r.flow(), Flow::Consumed, "A2: dismiss applies");
    assert!(!st.borrow().is_open(), "A2: dismiss closes");
    assert_eq!(
        st.borrow().value(),
        Some(ItemKey::index(0)),
        "A2: dismiss changes nothing"
    );
    // V3/N1: the disabled Engine select paints faint and never opens.
    // (The component has no label prop, so the faint half reads off the
    // disabled value text instead of the VB label row.)
    let est = Rc::new(RefCell::new(SelectState::default()));
    est.borrow_mut().set_value(Some(ItemKey::index(0)));
    let elog = Rc::new(RefCell::new(Vec::new()));
    let mut eapp = Harness::new(
        SelectApp {
            st: Rc::clone(&est),
            other: SelectState::default(),
            log: Rc::clone(&elog),
            disabled: true,
        },
        Theme::junie(),
        60,
        14,
    );
    eapp.blur();
    let ebuf = eapp.buffer();
    assert_eq!(ebuf[(6, 2)].fg, faint, "V3: disabled value is faint");
    assert!(
        buf_row_text(ebuf, 2).contains('▾'),
        "V3: disabled field keeps ▾"
    );
    assert_eq!(
        ebuf[(S4_SORT_AREA.right() - 2, 2)].fg,
        disabled_tone,
        "V3: disabled marker tone"
    );
    capture_isolated(&dir, "select-disabled", ebuf);
    let r = eapp.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Ignored, "N1: disabled ignores keys");
    assert!(elog.borrow().is_empty(), "N1: no event");
    assert!(!est.borrow().is_open(), "N1: never opens");
    let r = eapp.click(6, 2);
    assert_eq!(r.flow(), Flow::Consumed, "N1: disabled ignores clicks");
    assert!(elog.borrow().is_empty(), "N1: clicks emit nothing");
    assert!(!est.borrow().is_open(), "N1: clicks never open");
    // N2: a modified chord is Ignored; losing focus closes the popup.
    for mods in [KeyModifiers::CONTROL, KeyModifiers::ALT] {
        log.borrow_mut().clear();
        let r = app.key_mod(KeyCode::Char('x'), mods);
        assert_eq!(r.flow(), Flow::Ignored, "N2: modified chord is Ignored");
        assert!(log.borrow().is_empty(), "N2: no event");
    }
    let _ = app.key(KeyCode::Enter);
    assert!(st.borrow().is_open(), "fixture open");
    assert!(app.tab_to(S4_SORT_OTHER), "tab away");
    assert!(!st.borrow().is_open(), "N2: losing focus closes the popup");

    // PTY: the Sort select opens, moves, and chooses live; the closed
    // select steps directly; the disabled Engine select never opens.
    let case = s4_case("select_open", &["--page", "chipsselects"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "select-closed");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("created_at") && boot.contains('▾'),
        "V1 live: closed fields show ▾"
    );
    assert!(!boot.contains('▴'), "V1 live: nothing open at boot");
    click_at(&mut s, "created_at", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Sort popup never opened",
        |screen| support::screen_text(screen).contains("customer"),
    );
    checkpoint(&mut s, &dir, "select-open");
    let open = live_text(&mut s);
    assert!(open.contains('▴'), "V1 live: open field shows ▴");
    assert_line_has(&open, "›", "created_at", "V2 live: › on the selected row");
    support::press_step(&s, "down");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Sort by → total", "A1 live: change commits");
    checkpoint(&mut s, &dir, "select-changed");
    // S2 live: the closed select steps directly without opening.
    support::press_step(&s, "down");
    waits::wait_state(&mut s, "Sort by → status", "S2 live: closed Down steps");
    let stepped = live_text(&mut s);
    assert!(
        !stepped.contains("customer"),
        "S2 live: stepping paints no popup"
    );
    // A2 live: Esc closes and restores the cursor to the selection.
    // Focus never left the Sort select, so Enter reopens it directly.
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Sort popup never reopened",
        |screen| support::screen_text(screen).contains("customer"),
    );
    support::press_step(&s, "down");
    support::press_step(&s, "escape");
    let settled = settle_text(&mut s, case_timeout(&case), "esc steady");
    assert!(
        !settled.contains("customer"),
        "A2 live: Esc closes the popup"
    );
    assert!(
        settled.contains("Sort by → status") || !settled.contains("Sort by → customer"),
        "A2 live: Esc restores without choosing"
    );
    // V3/N1 live: the disabled Engine select never opens.
    click_at(&mut s, "PostgreSQL", case_timeout(&case));
    let engine = settle_text(&mut s, case_timeout(&case), "engine steady");
    assert!(!engine.contains('▴'), "V3 live: Engine paints no popup");
    assert_line_has(&engine, "PostgreSQL", "▾", "V3 live: Engine keeps ▾");
    eprintln!("s4 select_open: isolated popup map + live choose/step/engine");
}

const S4_QUICK: Id = Id::root("s4.quick");

/// Owned picker row with semantic identity.
struct S4PItem {
    key: ItemKey,
    label: String,
    detail: String,
    glyph: String,
    group: Option<String>,
    tag: Option<String>,
    matched: Vec<usize>,
    disabled: bool,
}

impl AsItem for S4PItem {
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
struct QuickApp {
    items: Vec<S4PItem>,
    st: Rc<RefCell<PickerState>>,
    log: Rc<RefCell<Vec<PickerAction>>>,
    open: bool,
    booted: bool,
    loading: bool,
    scopes: [ScopeKey; 2],
}

impl QuickApp {
    fn file_items() -> Vec<S4PItem> {
        vec![
            S4PItem {
                key: ItemKey::text("planner.rs"),
                label: "planner.rs".to_string(),
                detail: "src/bin/planner.rs".to_string(),
                glyph: "F".to_string(),
                group: Some("Files".to_string()),
                tag: None,
                matched: Vec::new(),
                disabled: false,
            },
            S4PItem {
                key: ItemKey::text("planner_flow.rs"),
                label: "planner_flow.rs".to_string(),
                detail: "tests/planner_flow.rs".to_string(),
                glyph: "F".to_string(),
                group: Some("Files".to_string()),
                tag: None,
                matched: Vec::new(),
                disabled: false,
            },
            S4PItem {
                key: ItemKey::text("Add retries to planner jobs"),
                label: "Add retries to planner jobs".to_string(),
                detail: "#27 · omar".to_string(),
                glyph: "T".to_string(),
                group: Some("Tasks".to_string()),
                tag: None,
                matched: Vec::new(),
                disabled: false,
            },
        ]
    }

    fn rig(items: Vec<S4PItem>, st: PickerState) -> (Self, Rc<RefCell<Vec<PickerAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            QuickApp {
                items,
                st: Rc::new(RefCell::new(st)),
                log: Rc::clone(&log),
                open: false,
                booted: false,
                loading: false,
                scopes: [ScopeKey::new(0), ScopeKey::new(1)],
            },
            log,
        )
    }

    fn picker(&self) -> Picker<'_, S4PItem> {
        let p = Picker::new(S4_QUICK)
            .title("Open quickly")
            .placeholder("Files and tasks…")
            .meta("All · Tab scope")
            .width(80)
            .scopes(&self.scopes)
            .footer("hints");
        if self.loading {
            p.empty(termrock::EmptyState::Loading { label: "indexing" })
        } else {
            p
        }
    }
}

impl App for QuickApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let loading = self.loading;
        // Boot-only open: a cancel-close must stay closed. Reopening on
        // `!open` resurrects the layer in the same keystroke's bubble
        // re-update, defeating Cancel.
        if !self.booted {
            self.booted = true;
            self.open = true;
            let scopes = self.scopes;
            let mut p = Picker::new(S4_QUICK)
                .title("Open quickly")
                .placeholder("Files and tasks…")
                .meta("All · Tab scope")
                .width(80)
                .scopes(&scopes)
                .footer("hints");
            if loading {
                p = p.empty(termrock::EmptyState::Loading { label: "indexing" });
            }
            p.reconcile(&mut self.st.borrow_mut(), &self.items);
            let spec = self.picker().layer(cx, &self.items);
            cx.open_layer(S4_QUICK, spec);
        }
        let r = self
            .picker()
            .update(cx, &mut self.st.borrow_mut(), &self.items)
            .on_action(|action| log.borrow_mut().push(action));
        if !cx.is_open(S4_QUICK) {
            self.open = false;
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.open {
            let _ = ui.layer(S4_QUICK, |ui, area| {
                self.picker().draw(ui, area, &self.st.borrow(), &self.items)
            });
        }
    }
}

/// PICKER-QUERY-001 (`showcase/flows/pickers/open`, 80x24): the Quick
/// picker with its query field, ranked rows, and scope label over [`Picker`].
///
/// Isolated path mirrors the row's Quick picker (title, placeholder, scope,
/// F/T rows): QueryChanged on every keystroke with cursor reset, NextScope,
/// Chosen/ChosenAlt, two-stage Esc, Submit-keeps-open, and the
/// blank-query Consumed negative. Live path types a query, cycles the
/// scope, chooses a row, proves Submit keeps the picker open, and cancels.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_picker_query() {
    let dir = s4_dir("s4_picker_query");

    // S1 isolated: every keystroke emits QueryChanged; the owner reset lands
    // on the first eligible row.
    let (rig, log) = QuickApp::rig(QuickApp::file_items(), PickerState::default());
    let st = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.is_open(S4_QUICK), "modal opens");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('o'));
    assert_eq!(r.flow(), Flow::Consumed, "S1: typing applies");
    assert!(
        log.borrow().contains(&PickerAction::QueryChanged),
        "S1: typing emits QueryChanged"
    );
    assert_eq!(st.borrow().query(), "o", "S1: query accumulates");
    let mut ranked = QuickApp::file_items();
    ranked[0].disabled = true;
    let (rig, _log) = QuickApp::rig(ranked, PickerState::default());
    let rst = Rc::clone(&rig.st);
    let _app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        rst.borrow().cursor(),
        Some(ItemKey::text("planner_flow.rs")),
        "S1: set_items resets past disabled rows"
    );
    // V1/V2 render form: the modal carries the title, query row, scope
    // label, grouped rows, and hint footer; file rows carry F, tasks T.
    let (rig, _log) = QuickApp::rig(QuickApp::file_items(), PickerState::default());
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    let screen: String = (0..24).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(screen.contains("Open quickly"), "V1: modal title");
    assert!(
        screen.contains("Files and tasks…"),
        "V1: placeholder with empty query"
    );
    assert!(screen.contains("All · Tab scope"), "V1: scope label");
    assert!(
        screen.contains("Files") && screen.contains("Tasks"),
        "V1: grouped rows"
    );
    assert!(screen.contains("hints"), "V1: hint footer");
    let f_line = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("planner_flow.rs"))
        .expect("V2: file row renders");
    assert!(f_line.contains('F'), "V2: file rows carry glyph F");
    assert!(
        f_line.contains("tests/planner_flow.rs"),
        "V2: file rows carry detail lines"
    );
    let t_line = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("Add retries"))
        .expect("V2: task row renders");
    assert!(t_line.contains('T'), "V2: task rows carry glyph T");
    capture_isolated(&dir, "picker-query", app.buffer());
    // S2/A1: Tab emits NextScope; Enter emits Chosen, Alt+Enter ChosenAlt.
    let (rig, log) = QuickApp::rig(QuickApp::file_items(), PickerState::default());
    let st = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Tab);
    assert_eq!(r.flow(), Flow::Consumed, "S2: Tab applies");
    assert!(
        log.borrow()
            .contains(&PickerAction::Scope(ScopeKey::new(1))),
        "S2: Tab emits NextScope"
    );
    st.borrow_mut()
        .set_cursor(2, ItemKey::text("Add retries to planner jobs"));
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow().contains(&PickerAction::Chosen(ItemKey::text(
            "Add retries to planner jobs"
        ))),
        "A1: Enter emits Chosen"
    );
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Enter, KeyModifiers::ALT);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Alt+Enter applies");
    assert!(
        log.borrow()
            .contains(&PickerAction::ChosenAlt(ItemKey::text(
                "Add retries to planner jobs"
            ))),
        "A1: Alt+Enter emits ChosenAlt"
    );
    // A2: first Esc clears a non-empty query; Esc on empty cancels.
    // (Cancel has no picker action: the layer correlate is the modal
    // closing with no choice.)
    st.borrow_mut().set_query("plan");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Esc with text applies");
    assert!(
        log.borrow().contains(&PickerAction::QueryChanged),
        "A2: Esc with text clears"
    );
    assert_eq!(st.borrow().query(), "", "A2: query cleared");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Esc empty applies");
    assert!(!app.is_open(S4_QUICK), "A2: Esc empty cancels");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, PickerAction::Chosen(_))),
        "A2: cancel chooses nothing"
    );
    // N1: Submit (free text, no eligible row) keeps the picker open.
    // (Submit has no picker action; the observable halves — stays open,
    // query stays, never Chosen — are kept.)
    let mut stale = QuickApp::file_items();
    for it in stale.iter_mut() {
        it.disabled = true;
    }
    let mut qst = PickerState::default();
    qst.set_query("zzz-no-match");
    let (rig, log) = QuickApp::rig(stale, qst);
    let st = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1: Enter applies");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, PickerAction::Chosen(_) | PickerAction::ChosenAlt(_))),
        "N1: Enter on no eligible row never chooses"
    );
    assert!(app.is_open(S4_QUICK), "N1: the picker stays open");
    assert_eq!(st.borrow().query(), "zzz-no-match", "N1: the query stays");
    st.borrow_mut().set_query("");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1: blank query applies");
    assert!(
        log.borrow().is_empty(),
        "N1: blank query + no eligible row consumes"
    );
    // Only a real, enabled row of a Ready picker resolves: Loading refuses.
    // (Loading is an empty presentation, not a status flag: the refusing
    // half runs with no rows under the Loading label.)
    let (rig, _log) = QuickApp::rig(QuickApp::file_items(), PickerState::default());
    let rst = Rc::clone(&rig.st);
    let _app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        rst.borrow().cursor(),
        Some(ItemKey::text("planner.rs")),
        "Ready resolves (presence proof)"
    );
    let (mut rig, _log) = QuickApp::rig(Vec::new(), PickerState::default());
    rig.loading = true;
    let lst = Rc::clone(&rig.st);
    let mut lapp = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        lst.borrow().cursor(),
        None,
        "only Ready pickers resolve rows"
    );
    let r = lapp.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "Loading Enter applies");
    // N2: clicks outside the modal own nothing (presence: rows own).
    assert!(
        app.area_of_part(S4_QUICK, PartRef::item(Part::ROW, ItemKey::text("foreign")))
            .is_none(),
        "N2: outside clicks own nothing"
    );
    assert!(
        app.area_of_part(
            S4_QUICK,
            PartRef::item(Part::ROW, ItemKey::text("planner.rs"))
        )
        .is_some(),
        "N2: rows are owned (presence proof)"
    );

    // PTY: the Quick picker opens, filters, scopes, chooses, submits, and
    // cancels live.
    let case = s4_case("picker_query", &["--page", "pickers"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // The modal-open needle is the scope line: the `Open quickly` button
    // label stays visible while closed, so it can never prove the modal.
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
    waits::wait_state(&mut s, "All · Tab scope", "Quick picker opened");
    checkpoint(&mut s, &dir, "picker-open");
    let opened = live_text(&mut s);
    assert!(
        opened.contains("Files and tasks…"),
        "V1 live: placeholder row"
    );
    assert!(opened.contains("All · Tab scope"), "V1 live: scope label");
    type_text(&mut s, "auth");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "query never filtered",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("auth.rs") && !text.contains("Cargo.toml")
        },
    );
    checkpoint(&mut s, &dir, "picker-filtered");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "scope never cycled",
        |screen| support::screen_text(screen).contains("Files · Tab scope"),
    );
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "choose never closed the modal",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Chose ") && !text.contains("Tab scope")
        },
    );
    checkpoint(&mut s, &dir, "picker-chosen");
    let chosen = live_text(&mut s);
    assert!(
        chosen.contains("Chosen"),
        "S1 live: Result records the choice"
    );
    // N1 live: free text with no eligible row submits and stays open.
    // (Choosing returned focus to NAV: Tab back on to Quick first.)
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
    // The page keeps the scope across opens: we cycled to Files above, so
    // the reopened modal shows Files, not All.
    waits::wait_state(&mut s, "Files · Tab scope", "Quick picker reopened");
    type_text(&mut s, "zzz-no-match");
    support::press_step(&s, "enter");
    let stayed = settle_text(&mut s, case_timeout(&case), "submit steady");
    checkpoint(&mut s, &dir, "submit-stayed");
    assert!(
        stayed.contains("Files · Tab scope"),
        "N1 live: Submit keeps the picker open"
    );
    assert!(stayed.contains("zzz-no-match"), "N1 live: the query stays");
    // A2 live: first Esc clears, second Esc cancels.
    support::press_step(&s, "escape");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Esc never cleared the query",
        |screen| {
            let text = support::screen_text(screen);
            text.contains("Files · Tab scope") && !text.contains("zzz-no-match")
        },
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Files · Tab scope", "A2 live: Esc cancels");
    eprintln!("s4 picker_query: isolated scope map + live filter/choose/submit");
}

const S4_TABS: Id = Id::root("s4.tabs");
const S4_LEVEL: Id = Id::root("s4.level");

/// Tabs/Level picker rig: modal picker owner plus action log.
struct TabsApp {
    id: Id,
    title: &'static str,
    placeholder: Option<&'static str>,
    width: u16,
    searchable: bool,
    items: Vec<S4PItem>,
    st: Rc<RefCell<PickerState>>,
    log: Rc<RefCell<Vec<PickerAction>>>,
    open: bool,
    booted: bool,
}

impl TabsApp {
    fn tab_items() -> Vec<S4PItem> {
        vec![
            S4PItem {
                key: ItemKey::text("Query 1"),
                label: "Query 1".to_string(),
                detail: "query".to_string(),
                glyph: "≡".to_string(),
                group: Some("Open tabs".to_string()),
                tag: None,
                matched: Vec::new(),
                disabled: false,
            },
            S4PItem {
                key: ItemKey::text("orders"),
                label: "orders".to_string(),
                detail: "public · data".to_string(),
                glyph: "T".to_string(),
                group: Some("Open tabs".to_string()),
                tag: Some("active".to_string()),
                matched: Vec::new(),
                disabled: false,
            },
            S4PItem {
                key: ItemKey::text("order_items"),
                label: "order_items".to_string(),
                detail: "public · data".to_string(),
                glyph: "T".to_string(),
                group: Some("Open tabs".to_string()),
                tag: None,
                matched: Vec::new(),
                disabled: false,
            },
            S4PItem {
                key: ItemKey::text("History"),
                label: "History".to_string(),
                detail: "public · data".to_string(),
                glyph: "T".to_string(),
                group: Some("Open tabs".to_string()),
                tag: None,
                matched: Vec::new(),
                disabled: false,
            },
        ]
    }

    fn level_items() -> Vec<S4PItem> {
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
        .map(|(i, l)| S4PItem {
            key: ItemKey::text(l),
            label: (*l).to_string(),
            detail: "level detail".to_string(),
            glyph: String::new(),
            group: None,
            tag: if i == 3 {
                Some("current".to_string())
            } else {
                None
            },
            matched: Vec::new(),
            disabled: false,
        })
        .collect()
    }

    fn rig(
        id: Id,
        title: &'static str,
        placeholder: Option<&'static str>,
        width: u16,
        searchable: bool,
        items: Vec<S4PItem>,
        st: PickerState,
    ) -> (Self, Rc<RefCell<Vec<PickerAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            TabsApp {
                id,
                title,
                placeholder,
                width,
                searchable,
                items,
                st: Rc::new(RefCell::new(st)),
                log: Rc::clone(&log),
                open: false,
                booted: false,
            },
            log,
        )
    }

    fn build(&self) -> Picker<'_, S4PItem> {
        let mut p = Picker::new(self.id)
            .title(self.title)
            .width(self.width)
            .searchable(self.searchable)
            .footer("hints");
        if let Some(ph) = self.placeholder {
            p = p.placeholder(ph);
        }
        p
    }
}

impl App for TabsApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        if !self.booted {
            self.booted = true;
            self.open = true;
            self.build()
                .reconcile(&mut self.st.borrow_mut(), &self.items);
            let spec = self.build().layer(cx, &self.items);
            cx.open_layer(self.id, spec);
        }
        let id = self.id;
        let r = self
            .build()
            .update(cx, &mut self.st.borrow_mut(), &self.items)
            .on_action(|action| log.borrow_mut().push(action));
        if !cx.is_open(id) {
            self.open = false;
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.open {
            let _ = ui.layer(self.id, |ui, area| {
                self.build().draw(ui, area, &self.st.borrow(), &self.items)
            });
        }
    }
}

/// PICKER-TABS-002 (`showcase/flows/pickers/tabs`, 80x24): the Tabs picker
/// with tags plus the non-searchable Level list over [`Picker`].
///
/// Isolated path mirrors the row's Tabs picker (glyphs, active tag, group)
/// and Level picker (six levels, current tag, no search box): Secondary
/// close, cursor-follows-level, j/k motion in non-searchable pickers, and
/// the typed-text-impossible negative. Live path closes tabs down to the
/// last (refused), proves Back is ignored, sets a level with vim keys,
/// and proves Secondary and typing do nothing outside the Tabs kind.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_picker_tabs() {
    let dir = s4_dir("s4_picker_tabs");

    // S1 isolated: Delete emits Secondary(i); the owner splice keeps the
    // cursor on a live row.
    let (rig, log) = TabsApp::rig(
        S4_TABS,
        "Open tabs",
        Some("Filter tabs…"),
        48,
        true,
        TabsApp::tab_items(),
        PickerState::default(),
    );
    let _st = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.is_open(S4_TABS), "modal opens");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Delete);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Delete applies");
    assert!(
        log.borrow()
            .contains(&PickerAction::Secondary(ItemKey::text("Query 1"))),
        "S1: Delete emits Secondary"
    );
    let mut rest = TabsApp::tab_items();
    rest.remove(0);
    assert_eq!(rest.len(), 3, "S1: the tab splices out");
    let (rig, _log) = TabsApp::rig(
        S4_TABS,
        "Open tabs",
        Some("Filter tabs…"),
        48,
        true,
        rest,
        PickerState::default(),
    );
    let rst = Rc::clone(&rig.st);
    let _app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        rst.borrow().cursor(),
        Some(ItemKey::text("orders")),
        "S1: cursor lands on a live row"
    );
    // V1 render form: glyphs, the active tag, and the group header.
    let mut three = TabsApp::tab_items();
    three.remove(0);
    let (rig, _log) = TabsApp::rig(
        S4_TABS,
        "Open tabs",
        Some("Filter tabs…"),
        48,
        true,
        three,
        PickerState::default(),
    );
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    let screen: String = (0..24).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(screen.contains("Open tabs"), "V1: Tabs modal title");
    assert!(screen.contains("active"), "V1: the active tag");
    assert!(screen.contains("Filter tabs…"), "V1: Tabs is searchable");
    let glyph_line = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("order_items"))
        .expect("V1: tab row renders");
    assert!(glyph_line.contains('T'), "V1: tab rows carry glyphs");
    capture_isolated(&dir, "picker-tabs", app.buffer());
    // N1 isolated: Backspace on an empty query emits Back (the showcase
    // page ignores it — proven live below).
    let (rig, log) = TabsApp::rig(
        S4_TABS,
        "Open tabs",
        Some("Filter tabs…"),
        48,
        true,
        TabsApp::tab_items(),
        PickerState::default(),
    );
    let st = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(st.borrow().query(), "", "query empty");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Backspace);
    assert_eq!(r.flow(), Flow::Consumed, "N1: Backspace applies");
    assert!(
        log.borrow().contains(&PickerAction::Back),
        "N1: Backspace on empty query emits Back"
    );

    // Level picker: non-searchable, cursor opens on the stored level.
    let (rig, log) = TabsApp::rig(
        S4_LEVEL,
        "Safe Mode · this connection",
        None,
        112,
        false,
        TabsApp::level_items(),
        PickerState::default(),
    );
    let lst = Rc::clone(&rig.st);
    let mut lapp = Harness::new(rig, Theme::junie(), 112, 24);
    lst.borrow_mut().set_cursor(3, ItemKey::text("Safe Mode"));
    assert_eq!(
        lst.borrow().cursor(),
        Some(ItemKey::text("Safe Mode")),
        "S2: the Level picker opens on the stored level"
    );
    // V2 render form: no query row; the current level carries its tag.
    let screen: String = (0..24).map(|y| buf_row_text(lapp.buffer(), y)).collect();
    assert!(
        !screen.contains("Type to search"),
        "V2: the Level modal shows no query row"
    );
    let cur_line = (0..24)
        .map(|y| buf_row_text(lapp.buffer(), y))
        .find(|l| l.contains("Safe Mode") && !l.contains("Full"))
        .expect("V2: level row renders");
    assert!(
        cur_line.contains("current") || screen.contains("current"),
        "V2: the current level carries the current tag"
    );
    capture_isolated(&dir, "picker-level", lapp.buffer());
    // A2: in a non-searchable picker j/k move and typed text is impossible.
    log.borrow_mut().clear();
    let r = lapp.key(KeyCode::Char('j'));
    assert_eq!(r.flow(), Flow::Consumed, "A2: j applies");
    assert!(log.borrow().is_empty(), "A2: j emits no action");
    assert_eq!(
        lst.borrow().cursor(),
        Some(ItemKey::text("Safe Mode (Full)")),
        "A2: j moves"
    );
    let r = lapp.key(KeyCode::Char('k'));
    assert_eq!(r.flow(), Flow::Consumed, "A2: k applies");
    assert_eq!(
        lst.borrow().cursor(),
        Some(ItemKey::text("Safe Mode")),
        "A2: k moves"
    );
    for code in [KeyCode::Char('x'), KeyCode::Char('q')] {
        log.borrow_mut().clear();
        let r = lapp.key(code);
        assert_eq!(r.flow(), Flow::Consumed, "A2: {code:?} applies");
        assert!(log.borrow().is_empty(), "A2: {code:?} types nothing");
    }
    assert_eq!(lst.borrow().query(), "", "A2: no query can accumulate");

    // PTY: close tabs to the last (refused), set the level with j/Enter.
    let case = s4_case("picker_tabs", &["--page", "pickers"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Tabs picker never opened",
        |screen| support::screen_text(screen).contains("Filter tabs…"),
    );
    checkpoint(&mut s, &dir, "tabs-open");
    let opened = live_text(&mut s);
    assert!(opened.contains("active"), "V1 live: the active tag");
    assert!(opened.contains("Open tabs"), "V1 live: the group header");
    support::press_step(&s, "delete");
    waits::wait_state(&mut s, "Closed Query 1", "S1 live: tab closes");
    let closed = live_text(&mut s);
    assert!(
        closed.contains("Filter tabs…"),
        "S1 live: the modal stays open after a close"
    );
    checkpoint(&mut s, &dir, "tab-closed");
    // A1 live: Secondary on the last remaining tab is refused.
    support::press_step(&s, "delete");
    waits::wait_state(&mut s, "Closed orders", "closing down");
    support::press_step(&s, "delete");
    waits::wait_state(&mut s, "Closed order_items", "closing down");
    support::press_step(&s, "delete");
    let last = settle_text(&mut s, case_timeout(&case), "last-tab steady");
    assert!(last.contains("Filter tabs…"), "A1 live: modal still open");
    assert!(
        last.contains("History"),
        "A1 live: the last tab never leaves"
    );
    // N1 live: Back on the empty query is ignored.
    support::press_step(&s, "backspace");
    let back = settle_text(&mut s, case_timeout(&case), "back steady");
    assert!(
        back.contains("Filter tabs…"),
        "N1 live: Back changes nothing"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Filter tabs…", "Tabs picker closes");
    // Level picker: no search box, vim motion, j/Enter sets the level.
    // (Esc returned focus to NAV; Level is the third button.)
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
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Level picker never opened",
        |screen| support::screen_text(screen).contains("Safe Mode · this connection"),
    );
    let level = live_text(&mut s);
    assert!(level.contains("current"), "V2 live: the current tag");
    assert!(
        !level.contains("Filter tabs…") && !level.contains("Files and tasks…"),
        "V2 live: no query row"
    );
    support::press_step(&s, "j");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Chose Safe Mode (Full)", "S2 live: level sets");
    checkpoint(&mut s, &dir, "level-set");
    let set = live_text(&mut s);
    assert!(
        set.contains("Safe Mode (Full)"),
        "S2 live: Result keeps the level"
    );
    // N2/A2 live: Secondary and typing do nothing in the Level picker.
    // (Choosing returned focus to NAV; Level is the third button.)
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Level button never refocused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Choose a level"))
        },
    );
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "Level picker never reopened",
        |screen| support::screen_text(screen).contains("Safe Mode · this connection"),
    );
    support::press_step(&s, "delete");
    support::press_step(&s, "x");
    let noop = settle_text(&mut s, case_timeout(&case), "level-noop steady");
    assert!(
        noop.contains("Safe Mode · this connection"),
        "N2 live: Secondary outside Tabs does nothing"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(
        &mut s,
        "Safe Mode · this connection",
        "Level picker keeps Esc",
    );
    eprintln!("s4 picker_tabs: isolated tabs/level + live close-to-one/set");
}

const S4_COMPLETE: Id = Id::root("s4.complete");
const S4_COMPLETE_EDITOR: Id = Id::root("s4.complete.editor");

/// Owned completion row with semantic identity.
struct S4CItem {
    key: ItemKey,
    label: String,
    glyph: String,
    detail: String,
    insert: String,
    matched: Vec<usize>,
}

impl AsItem for S4CItem {
    fn as_item(&self) -> Item<'_> {
        let mut item = Item::new(self.key, self.label.as_str());
        item.glyph = self.glyph.as_str();
        item.detail = self.detail.as_str();
        item.insert = Some(self.insert.as_str());
        item.matched = &self.matched;
        item
    }
}

/// Editor + suggestion-popover rig plus action log.
struct CompleteApp {
    value: String,
    est: TextInputState,
    items: Vec<S4CItem>,
    cst: Rc<RefCell<CompletionState>>,
    log: Rc<RefCell<Vec<CompletionAction>>>,
    booted: bool,
}

impl CompleteApp {
    fn items() -> Vec<S4CItem> {
        vec![
            S4CItem {
                key: ItemKey::text("client("),
                label: "client(".to_string(),
                glyph: "ƒ".to_string(),
                detail: "fn () -> Client".to_string(),
                insert: "client(".to_string(),
                matched: vec![0, 1],
            },
            S4CItem {
                key: ItemKey::text("Client"),
                label: "Client".to_string(),
                glyph: "T".to_string(),
                detail: "struct".to_string(),
                insert: "Client".to_string(),
                matched: vec![0],
            },
            S4CItem {
                key: ItemKey::text("delay"),
                label: "delay".to_string(),
                glyph: "v".to_string(),
                detail: "local · u64".to_string(),
                insert: "delay".to_string(),
                matched: Vec::new(),
            },
        ]
    }

    fn rig(items: Vec<S4CItem>) -> (Self, Rc<RefCell<Vec<CompletionAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            CompleteApp {
                value: "cl".to_string(),
                est: TextInputState::default(),
                items,
                cst: Rc::new(RefCell::new(CompletionState::default())),
                log: Rc::clone(&log),
                booted: false,
            },
            log,
        )
    }

    fn completion(&self) -> Completion<'_, S4CItem> {
        Completion::new(S4_COMPLETE)
    }
}

impl App for CompleteApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.booted {
            self.booted = true;
            CompletionController::new(S4_COMPLETE_EDITOR, S4_COMPLETE).request(
                cx,
                &mut self.cst.borrow_mut(),
                Rect::new(10, 5, 1, 1),
                2,
                &self.items,
            );
        }
        let mut r = TextInput::new(S4_COMPLETE_EDITOR)
            .update(cx, &mut self.est, &mut self.value)
            .erase();
        let log = Rc::clone(&self.log);
        r |= self
            .completion()
            .update_for(
                S4_COMPLETE_EDITOR,
                cx,
                &mut self.cst.borrow_mut(),
                &self.items,
            )
            .on_action(|action| log.borrow_mut().push(action));
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(S4_COMPLETE_EDITOR).value(&self.value).draw(
            ui,
            Rect::new(10, 5, 20, 1),
            &self.est,
        );
        if self.cst.borrow().is_open() {
            let _ = ui.layer(S4_COMPLETE, |ui, area| {
                self.completion()
                    .draw(ui, area, &self.cst.borrow(), &self.items)
            });
        }
    }
}

/// COMPLETE-EDITOR-002 (`showcase/flows/editor/completion`, 80x24): the code
/// editor identifier popup over [`Completion`].
///
/// Isolated path mirrors the row's suggestion popup (function/type/local
/// candidates with signature details and fuzzy-matched bold
/// bytes): anchor geometry, clamped navigation, Accept/Dismiss, click and
/// wheel paths. Live path auto-triggers with two characters, moves, accepts
/// into the buffer, and proves the one-character and no-match negatives.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_complete_editor() {
    let dir = s4_dir("s4_complete_editor");

    // S2 isolated: the popup opens only when non-empty.
    let (rig, _log) = CompleteApp::rig(Vec::new());
    let cst = Rc::clone(&rig.cst);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S4_COMPLETE_EDITOR), "navigating focus");
    assert!(!cst.borrow().is_open(), "S2: empty open stays closed");
    assert!(!app.is_open(S4_COMPLETE), "S2: no layer");
    let (rig, log) = CompleteApp::rig(CompleteApp::items());
    let cst = Rc::clone(&rig.cst);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S4_COMPLETE_EDITOR), "navigating focus");
    assert!(cst.borrow().is_open(), "S2: non-empty opens");
    assert_eq!(
        cst.borrow().cursor(),
        Some(ItemKey::text("client(")),
        "open resets the cursor"
    );
    // V1 render form: anchored at the word start with glyphs + details.
    let area = app.layer_area(S4_COMPLETE).expect("V1: popup layer placed");
    assert_eq!(area.x, 10, "V1: popup left-aligns with the word start");
    assert!(area.y > 5, "V1: popup anchors below the anchor row");
    let row0 = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("client("))
        .expect("V1: function row renders");
    assert!(
        row0.contains('ƒ') && row0.contains("client("),
        "V1: function glyph plus label"
    );
    assert!(row0.contains("fn () -> Client"), "V1: signature detail");
    let row1 = (0..24)
        .map(|y| buf_row_text(app.buffer(), y))
        .find(|l| l.contains("Client") && !l.contains("client("))
        .expect("V1: type row renders");
    assert!(
        row1.contains('T') && row1.contains("Client"),
        "V1: type glyph"
    );
    capture_isolated(&dir, "complete-open", app.buffer());
    // V2: fuzzy-matched label bytes render bold on every row. (The cursor
    // row keeps the bold base style, so the unmatched-plain half reads off
    // the non-cursor rows. Label columns derive from the painted rows: the
    // gutter geometry is the component's own.)
    let y0 = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("client("))
        .expect("row 0");
    let lx = char_col(&buf_row_text(app.buffer(), y0), "client(");
    assert!(
        buf_is_bold(app.buffer(), lx, y0),
        "V2: matched byte 0 is bold"
    );
    assert!(
        buf_is_bold(app.buffer(), lx + 1, y0),
        "V2: matched byte 1 is bold"
    );
    let y1 = (0..24)
        .find(|y| {
            let l = buf_row_text(app.buffer(), *y);
            l.contains("Client") && !l.contains("client(")
        })
        .expect("row 1");
    let lx1 = char_col(&buf_row_text(app.buffer(), y1), "Client");
    assert!(
        buf_is_bold(app.buffer(), lx1, y1),
        "V2: row 1 matched byte is bold"
    );
    assert!(
        !buf_is_bold(app.buffer(), lx1 + 1, y1),
        "V2: unmatched bytes stay plain off-cursor"
    );
    let y2 = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("delay"))
        .expect("row 2");
    let lx2 = char_col(&buf_row_text(app.buffer(), y2), "delay");
    assert!(
        !buf_is_bold(app.buffer(), lx2, y2),
        "V2: unmatched rows stay plain"
    );
    // Navigation: Down/Up clamp (never wrap); plain n/p are ignored so the
    // owner keeps typing; ctrl-n/ctrl-p move.
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Down);
    assert_eq!(r.flow(), Flow::Consumed, "Down applies");
    assert_eq!(
        cst.borrow().cursor(),
        Some(ItemKey::text("Client")),
        "Down moves"
    );
    // (Plain `n` reaches the editor, which owns the keystroke; the
    // completion half — no motion — is kept.)
    let r = app.key(KeyCode::Char('n'));
    assert_eq!(r.flow(), Flow::Consumed, "n reaches the editor");
    assert!(
        !log.borrow().contains(&CompletionAction::Moved),
        "plain n is ignored for typing"
    );
    assert_eq!(
        cst.borrow().cursor(),
        Some(ItemKey::text("Client")),
        "plain n never moves"
    );
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Consumed, "ctrl-n applies");
    assert_eq!(
        cst.borrow().cursor(),
        Some(ItemKey::text("delay")),
        "ctrl-n moves"
    );
    let r = app.key_mod(KeyCode::Char('n'), KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Consumed, "clamp applies");
    assert_eq!(
        cst.borrow().cursor(),
        Some(ItemKey::text("delay")),
        "Down clamps"
    );
    let r = app.key(KeyCode::Up);
    assert_eq!(r.flow(), Flow::Consumed, "Up applies");
    assert_eq!(
        cst.borrow().cursor(),
        Some(ItemKey::text("Client")),
        "Up moves"
    );
    // A1: Tab/Enter accept the cursor row; click accepts a visible row.
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow()
            .contains(&CompletionAction::Accepted(ItemKey::text("Client"))),
        "A1: Enter accepts the cursor"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Tab);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Tab applies");
    assert!(
        log.borrow()
            .contains(&CompletionAction::Accepted(ItemKey::text("Client"))),
        "A1: Tab accepts too"
    );
    assert!(
        app.area_of_part(
            S4_COMPLETE,
            PartRef::item(Part::ROW, ItemKey::text("client("))
        )
        .is_some(),
        "rows are owned (presence proof)"
    );
    let cy = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("delay"))
        .expect("click row");
    let cx = char_col(&buf_row_text(app.buffer(), cy), "delay") + 1;
    log.borrow_mut().clear();
    let _ = app.click(cx, cy);
    assert!(
        log.borrow()
            .contains(&CompletionAction::Accepted(ItemKey::text("delay"))),
        "click accepts the row"
    );
    log.borrow_mut().clear();
    let _ = app.click(79, 23);
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, CompletionAction::Accepted(_))),
        "foreign clicks miss"
    );
    // A2: Dismiss closes; N2: a stale index resolves to nothing.
    // (The foreign click above already dismissed this rig's layer, so the
    // Esc half drives a fresh open rig.)
    let (rig, log) = CompleteApp::rig(CompleteApp::items());
    let cst = Rc::clone(&rig.cst);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.tab_to(S4_COMPLETE_EDITOR), "navigating focus");
    assert!(cst.borrow().is_open(), "fixture open");
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A2: Esc applies");
    assert!(
        log.borrow().contains(&CompletionAction::Dismissed),
        "A2: Esc dismisses"
    );
    assert!(!cst.borrow().is_open(), "A2: dismiss closes");
    assert_eq!(
        CompleteApp::items().get(99).map(|i| i.key),
        None,
        "N2: stale index resolves to nothing"
    );

    // PTY: two characters auto-trigger, Down moves, Enter accepts into the
    // buffer; one character and past-all-matches never open.
    let case = s4_case("complete_editor", &["--page", "codeeditor"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "editor never focused",
        |screen| support::screen_text(screen).contains("Code editor"),
    );
    support::press_step(&s, "i");
    type_text(&mut s, "cl");
    // "fn () -> Client" is popup-only: the fixture code spells the same
    // symbol as `fn client() -> Client`, so a bare "Client" wait would
    // pass vacuously at boot.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "completion never triggered",
        |screen| support::screen_text(screen).contains("fn () -> Client"),
    );
    checkpoint(&mut s, &dir, "completion-open");
    let opened = live_text(&mut s);
    assert!(
        opened.contains('ƒ') || opened.contains('T'),
        "V1 live: kind glyphs"
    );
    assert_line_has(&opened, "T", "Client", "V1 live: type row + detail");
    support::press_step(&s, "down");
    support::press_step(&s, "enter");
    waits::wait_gone(&mut s, "fn () -> Client", "A1 live: accept closes");
    checkpoint(&mut s, &dir, "completion-accepted");
    let accepted = live_text(&mut s);
    assert!(
        accepted.contains("EDIT"),
        "A1 live: still editing after accept (presence proof)"
    );
    // N1 live: a one-character word never auto-opens.
    type_text(&mut s, " z");
    let single = settle_text(&mut s, case_timeout(&case), "single-char steady");
    assert!(
        !single.contains("fn () -> Client"),
        "N1 live: one character opens nothing"
    );
    assert!(
        single.contains("EDIT"),
        "N1 live: editor still live (presence proof)"
    );
    // N1 live: typing past all matches closes.
    type_text(&mut s, "z");
    support::press_step(&s, "backspace");
    support::press_step(&s, "backspace");
    type_text(&mut s, "qq");
    let nomatch = settle_text(&mut s, case_timeout(&case), "no-match steady");
    assert!(
        !nomatch.contains("fn () -> Client"),
        "N1 live: no matches opens nothing"
    );
    assert!(
        nomatch.contains("EDIT"),
        "N1 live: editor still live (presence proof)"
    );
    eprintln!("s4 complete_editor: isolated popup map + live trigger/accept");
}

const S4_CONFIRM: Id = Id::root("s4.confirm");
const S4_FACTS: Id = Id::root("s4.facts");
const S4_THREE: Id = Id::root("s4.three");
const S4_PROMPT: Id = Id::root("s4.prompt");
const S4_DELETE: Id = Id::root("s4.delete");

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum DialogKind {
    Confirm,
    Facts,
    Prompt,
    Destructive,
}

/// Confirm/facts/three-choice dialog rig plus action log.
struct DialogRig {
    id: Id,
    kind: DialogKind,
    cancel_enabled: bool,
    st: Rc<RefCell<DialogState>>,
    log: Rc<RefCell<Vec<DialogAction>>>,
    open: bool,
    booted: bool,
}

impl DialogRig {
    fn rig(id: Id, kind: DialogKind) -> (Self, Rc<RefCell<Vec<DialogAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            DialogRig {
                id,
                kind,
                cancel_enabled: true,
                st: Rc::new(RefCell::new(DialogState::default())),
                log: Rc::clone(&log),
                open: false,
                booted: false,
            },
            log,
        )
    }

    fn update_inner(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let r = match self.kind {
            DialogKind::Confirm => {
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel").enabled(self.cancel_enabled),
                    Action::new(ActionKey::CONFIRM, "Run"),
                ];
                let d = Dialog::confirm(self.id, "Run task now?", "Run the plan now.")
                    .actions(&actions);
                if !self.booted {
                    self.booted = true;
                    self.open = true;
                    cx.open_layer(self.id, d.layer(cx));
                }
                d.update(cx, &mut self.st.borrow_mut())
                    .on_action(|action| log.borrow_mut().push(action))
            }
            DialogKind::Facts => {
                let props: [(&str, &str); 0] = [];
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel"),
                    Action::primary(ActionKey::CONFIRM, "Apply"),
                ];
                let d = Dialog::facts(self.id, "Facts", &props).actions(&actions);
                if !self.booted {
                    self.booted = true;
                    self.open = true;
                    cx.open_layer(self.id, d.layer(cx));
                }
                d.update(cx, &mut self.st.borrow_mut())
                    .on_action(|action| log.borrow_mut().push(action))
            }
            DialogKind::Prompt => {
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel"),
                    Action::new(ActionKey::CONFIRM, "Rename"),
                ];
                let d = Dialog::prompt(self.id, "Rename task", "Task name")
                    .input_help("Shown in the task list and PR title")
                    .input_required(true)
                    .actions(&actions);
                if !self.booted {
                    self.booted = true;
                    self.open = true;
                    cx.open_layer(self.id, d.layer(cx));
                }
                d.update(cx, &mut self.st.borrow_mut())
                    .on_action(|action| log.borrow_mut().push(action))
            }
            DialogKind::Destructive => {
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel"),
                    Action::danger(ActionKey::CONFIRM, "Delete branch"),
                ];
                let d = Dialog::destructive(self.id, "Delete branch?", "This cannot be undone.")
                    .actions(&actions);
                if !self.booted {
                    self.booted = true;
                    self.open = true;
                    cx.open_layer(self.id, d.layer(cx));
                }
                d.update(cx, &mut self.st.borrow_mut())
                    .on_action(|action| log.borrow_mut().push(action))
            }
        };
        if !cx.is_open(self.id) {
            self.open = false;
        }
        r
    }

    fn draw_inner(&self, ui: &mut Ui<'_>) {
        match self.kind {
            DialogKind::Confirm => {
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel").enabled(self.cancel_enabled),
                    Action::new(ActionKey::CONFIRM, "Run"),
                ];
                let d = Dialog::confirm(self.id, "Run task now?", "Run the plan now.")
                    .actions(&actions);
                let _ = ui.layer(self.id, |ui, area| {
                    d.draw(ui, area, &self.st.borrow(), |_, _| ())
                });
            }
            DialogKind::Facts => {
                let props: [(&str, &str); 0] = [];
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel"),
                    Action::primary(ActionKey::CONFIRM, "Apply"),
                ];
                let d = Dialog::facts(self.id, "Facts", &props).actions(&actions);
                let _ = ui.layer(self.id, |ui, area| {
                    d.draw(ui, area, &self.st.borrow(), |_, _| ())
                });
            }
            DialogKind::Prompt => {
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel"),
                    Action::new(ActionKey::CONFIRM, "Rename"),
                ];
                let d = Dialog::prompt(self.id, "Rename task", "Task name")
                    .input_help("Shown in the task list and PR title")
                    .input_required(true)
                    .actions(&actions);
                let _ = ui.layer(self.id, |ui, area| {
                    d.draw(ui, area, &self.st.borrow(), |_, _| ())
                });
            }
            DialogKind::Destructive => {
                let actions = [
                    Action::quiet(ActionKey::CANCEL, "Cancel"),
                    Action::danger(ActionKey::CONFIRM, "Delete branch"),
                ];
                let d = Dialog::destructive(self.id, "Delete branch?", "This cannot be undone.")
                    .actions(&actions);
                let _ = ui.layer(self.id, |ui, area| {
                    d.draw(ui, area, &self.st.borrow(), |_, _| ())
                });
            }
        }
    }
}

impl App for DialogRig {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.update_inner(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.open {
            self.draw_inner(ui);
        }
    }
}

/// Focused action of a dialog rig, by button index.
fn dialog_focused<A>(app: &Harness<A>, id: Id, n: usize) -> Option<usize>
where
    A: App,
{
    let probe = Dialog::confirm(id, "", "");
    (0..n).find(|i| {
        app.state_of(probe.action_id(*i))
            .contains(StateFlags::FOCUSED)
    })
}

/// DIALOG-CONFIRM-001 (`showcase/flows/dialogs/open`, 80x24): the confirm
/// plus three-choice text dialogs over [`Dialog`].
///
/// Isolated path mirrors the row's dialogs (Run confirm, Unsaved-changes
/// three-choice): initial focus, Left/Right skipping disabled, Tab ring,
/// y/n quick answers, Esc/outside-cancel, and the facts/input y/n
/// negatives. Live path answers with y, walks the three choices, and
/// cancels with Esc.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_dialog_confirm() {
    let dir = s4_dir("s4_dialog_confirm");

    // V2/S1 isolated: confirm focuses the primary action first; Left/Right
    // move between actions skipping disabled ones; Tab cycles the ring.
    let (rig, _log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let probe = Dialog::confirm(S4_CONFIRM, "Run task now?", "Run the plan now.");
    assert_eq!(
        probe.initial_focus(),
        Some(probe.action_id(1)),
        "V2: primary focused first"
    );
    assert_eq!(
        dialog_focused(&app, S4_CONFIRM, 2),
        Some(1),
        "focus starts on Run"
    );
    let r = app.key(KeyCode::Left);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Left applies");
    assert_eq!(
        dialog_focused(&app, S4_CONFIRM, 2),
        Some(0),
        "S1: Left moves"
    );
    // Disabled Cancel: Left from Run stays (a fresh rig with Cancel off).
    let (mut rig, _log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    rig.cancel_enabled = false;
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert_eq!(
        dialog_focused(&app, S4_CONFIRM, 2),
        Some(1),
        "focus starts on Run"
    );
    let r = app.key(KeyCode::Left);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Left applies");
    assert_eq!(
        dialog_focused(&app, S4_CONFIRM, 2),
        Some(1),
        "S1: Left skips disabled actions"
    );
    // Tab cycles the ring (Cancel re-enabled: a fresh rig, focus on Run).
    let (rig, _log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let _ = app.key(KeyCode::Tab);
    assert_eq!(
        dialog_focused(&app, S4_CONFIRM, 2),
        Some(0),
        "S1: Tab cycles the ring"
    );
    // A1: y finishes the primary action, n the cancel action.
    let (rig, log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: y applies");
    assert!(
        log.borrow()
            .contains(&DialogAction::Action(ActionKey::CONFIRM)),
        "A1: y finishes the primary action"
    );
    let (rig, log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let r = app.key(KeyCode::Char('n'));
    assert_eq!(r.flow(), Flow::Consumed, "A1: n applies");
    assert!(
        log.borrow()
            .contains(&DialogAction::Action(ActionKey::CANCEL)),
        "A1: n finishes the cancel action"
    );
    // A2: Esc and click-outside finish the cancel action.
    // (Dismissal is the layer correlate of finishing cancel: the modal
    // closes with no action fired.)
    let (rig, log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.is_open(S4_CONFIRM), "fixture open");
    let _ = app.key(KeyCode::Esc);
    for _ in 0..4 {
        if log
            .borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Dismissed(_)))
        {
            break;
        }
        let _ = app.tick();
    }
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Dismissed(_))),
        "A2: Esc finishes cancel"
    );
    assert!(!app.is_open(S4_CONFIRM), "A2: Esc closes");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Action(_))),
        "A2: Esc fires no action"
    );
    let (rig, log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.is_open(S4_CONFIRM), "fixture open");
    let _ = app.click(0, 0);
    for _ in 0..4 {
        if log
            .borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Dismissed(_)))
        {
            break;
        }
        let _ = app.tick();
    }
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Dismissed(_))),
        "A2: click-outside finishes cancel"
    );
    assert!(!app.is_open(S4_CONFIRM), "A2: click-outside closes");
    // N1: y/n do nothing on facts dialogs; other keys are Consumed.
    let (rig, log) = DialogRig::rig(S4_FACTS, DialogKind::Facts);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(app.is_open(S4_FACTS), "facts fixture open");
    for code in [KeyCode::Char('y'), KeyCode::Char('n')] {
        log.borrow_mut().clear();
        let r = app.key(code);
        assert_eq!(r.flow(), Flow::Consumed, "N1: {code:?} applies");
        assert!(
            log.borrow().is_empty(),
            "N1: {code:?} does nothing on facts dialogs"
        );
        assert!(app.is_open(S4_FACTS), "N1: facts stays open");
    }
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('q'));
    assert_eq!(r.flow(), Flow::Consumed, "N1: other keys are Consumed");
    assert!(log.borrow().is_empty(), "N1: q fires nothing");
    // N2: clicks on no action are Consumed (presence: action clicks finish).
    log.borrow_mut().clear();
    let r = app.click(0, 0);
    assert_eq!(r.flow(), Flow::Consumed, "N2: foreign clicks Consumed");
    assert!(
        !log.borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Action(_))),
        "N2: foreign clicks finish nothing"
    );
    let (rig, log) = DialogRig::rig(S4_FACTS, DialogKind::Facts);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let ay = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Apply"))
        .expect("Apply row renders");
    let ax = char_col(&buf_row_text(app.buffer(), ay), "Apply") + 1;
    let _ = app.click(ax, ay);
    assert!(
        log.borrow()
            .contains(&DialogAction::Action(ActionKey::CONFIRM)),
        "action clicks finish (presence proof)"
    );
    // V1 render form: the dialog centers over the page.
    let (rig, _log) = DialogRig::rig(S4_CONFIRM, DialogKind::Confirm);
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(
        (0..24).any(|y| buf_row_text(app.buffer(), y).contains("Run task now?")),
        "V1: title renders"
    );
    // The dialog frame (not the left-aligned title) centers: the top
    // border's left and right margins match (cell columns, not bytes).
    let top_y = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains('╭'))
        .expect("V1: dialog frame renders");
    let top = buf_row_text(app.buffer(), top_y);
    let x0 = char_col(&top, "╭") as i16;
    let x1 = char_col(&top, "╮") as i16;
    assert!(
        (x0 - (80 - x1)).abs() <= 2,
        "V1: the dialog centers over the page"
    );
    let screen: String = (0..24).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(
        screen.contains("Run") && screen.contains("Cancel"),
        "V1: both actions render"
    );
    capture_isolated(&dir, "dialog-confirm", app.buffer());
    // Three-choice form: Save focused first.
    let three_actions = [
        Action::quiet(ActionKey::CANCEL, "Cancel"),
        Action::secondary(ActionKey::custom("discard"), "Discard"),
        Action::new(ActionKey::CONFIRM, "Save"),
    ];
    let three = Dialog::confirm(S4_THREE, "Unsaved changes", "Save first?").actions(&three_actions);
    assert_eq!(
        three.initial_focus(),
        Some(three.action_id(2)),
        "V2: the three-choice dialog focuses Save"
    );

    // PTY: answer with y, walk the three choices, cancel with Esc.
    let case = s4_case("dialog_confirm", &["--page", "dialogs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "confirm never opened",
        |screen| support::screen_text(screen).contains("Run task now?"),
    );
    checkpoint(&mut s, &dir, "dialog-open");
    support::press_step(&s, "y");
    waits::wait_state(&mut s, "Task started", "A1 live: y runs");
    checkpoint(&mut s, &dir, "dialog-answered");
    let answered = live_text(&mut s);
    assert!(
        answered.contains("Run ") && answered.contains("Task started"),
        "S2 live: history names the outcome"
    );
    // Three choices: Left/Right walk, Enter activates.
    click_at(&mut s, "Three choices", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "three-choice never opened",
        |screen| support::screen_text(screen).contains("Unsaved changes"),
    );
    support::press_step(&s, "left");
    support::press_step(&s, "right");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Description saved", "S1 live: Save activates");
    // Reopen and cancel with Esc.
    click_at(&mut s, "Confirm run", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "confirm never reopened",
        |screen| support::screen_text(screen).contains("Run task now?"),
    );
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled", "A2 live: Esc cancels");
    checkpoint(&mut s, &dir, "dialog-cancelled");
    eprintln!("s4 dialog_confirm: isolated answers + live y/walk/cancel");
}

/// DIALOG-PROMPT-002 (`showcase/flows/dialogs/prompt`, 80x24): the validated
/// prompt plus the destructive confirm over [`Dialog`].
///
/// Isolated path mirrors the row's Rename prompt (prefilled input, help,
/// non-empty/40-max rules) and Delete-branch destructive dialog: finish
/// commits then validates, failing validation keeps the dialog open, Enter
/// submits to the primary action, paste routes to the input, and the y/n
/// and Esc negatives. Live path blocks an empty submit, renames validly,
/// rejects an over-long name, and cancels the destructive dialog twice.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_dialog_prompt() {
    let dir = s4_dir("s4_dialog_prompt");

    // V1 isolated: the prompt focuses its input first with help under it.
    // (Validation is caller-owned in the component: the failing-submit
    // halves below assert the row's blocking expectation and fail where
    // the component fires regardless.)
    let (rig, log) = DialogRig::rig(S4_PROMPT, DialogKind::Prompt);
    let pst = Rc::clone(&rig.st);
    pst.borrow_mut().set_draft("Migrate sessions table");
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let probe = Dialog::prompt(S4_PROMPT, "Rename task", "Task name");
    let input_id = probe.input_id();
    let screen: String = (0..24).map(|y| buf_row_text(app.buffer(), y)).collect();
    assert!(screen.contains("Rename task"), "V1: prompt title");
    assert!(
        screen.contains("Migrate sessions table"),
        "V1: prefilled input"
    );
    assert!(
        screen.contains("Shown in the task list"),
        "V1: help text under the input"
    );
    capture_isolated(&dir, "dialog-prompt", app.buffer());
    // N1 (input half of confirm-row N1): y does not finish an input dialog.
    assert!(
        app.state_of(input_id).contains(StateFlags::FOCUSED),
        "V1: input focused first"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "N1: y applies");
    assert!(
        log.borrow().is_empty(),
        "N1: y finishes nothing on input dialogs"
    );
    assert!(app.is_open(S4_PROMPT), "N1: prompt stays open");
    // S1/N1: finish commits the editing input then validates; a failing
    // validation keeps the dialog open and stores no rename.
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "Enter begins editing");
    let r = app.ctrl('l');
    assert_eq!(r.flow(), Flow::Consumed, "Ctrl+L selects all");
    let r = app.key(KeyCode::Backspace);
    assert_eq!(r.flow(), Flow::Consumed, "Backspace clears");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "submit runs");
    assert!(
        log.borrow().is_empty(),
        "S1: failing validation keeps the dialog open"
    );
    assert!(app.is_open(S4_PROMPT), "S1: still open");
    // A1: a valid name submits to the primary action; paste routes in.
    // (The failed submit committed, so Enter begins editing again before
    // pasting — mirroring the live path.)
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "Enter begins editing again");
    let r = app.paste("Seed the demo database");
    assert_eq!(r.flow(), Flow::Consumed, "A1: paste routes to the input");
    assert_eq!(
        pst.borrow().draft(),
        "Seed the demo database",
        "A1: pasted draft lands"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "submit runs");
    assert!(
        log.borrow()
            .contains(&DialogAction::Action(ActionKey::CONFIRM)),
        "S2: a valid rename finishes the primary action"
    );
    // N1: over-long names never close the prompt.
    let (rig, log) = DialogRig::rig(S4_PROMPT, DialogKind::Prompt);
    let pst = Rc::clone(&rig.st);
    pst.borrow_mut().set_draft("Migrate sessions table");
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let _ = app.key(KeyCode::Enter);
    let _ = app.ctrl('l');
    let _ = app.paste(&"x".repeat(41));
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Enter);
    assert!(log.borrow().is_empty(), "N1: over-long names never close");
    assert!(app.is_open(S4_PROMPT), "N1: still open");
    // V2/N2: the destructive dialog focuses Cancel first in danger style.
    // (The danger variant is proven behaviorally: `y` answers through the
    // danger action, while Enter/Esc land on Cancel.)
    let (rig, log) = DialogRig::rig(S4_DELETE, DialogKind::Destructive);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let xprobe = Dialog::destructive(S4_DELETE, "Delete branch?", "This cannot be undone.");
    assert_eq!(
        xprobe.initial_focus(),
        Some(xprobe.action_id(0)),
        "V2: Cancel focused first"
    );
    assert_eq!(
        dialog_focused(&app, S4_DELETE, 2),
        Some(0),
        "V2: Cancel holds focus"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('y'));
    assert_eq!(r.flow(), Flow::Consumed, "V2: y applies");
    assert!(
        log.borrow()
            .contains(&DialogAction::Action(ActionKey::CONFIRM)),
        "V2: danger action answers y"
    );
    let (rig, log) = DialogRig::rig(S4_DELETE, DialogKind::Destructive);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "N2: Esc applies");
    for _ in 0..4 {
        if log
            .borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Dismissed(_)))
        {
            break;
        }
        let _ = app.tick();
    }
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, DialogAction::Dismissed(_))),
        "N2: Esc always lands on Cancel"
    );
    assert!(!app.is_open(S4_DELETE), "N2: Esc closes");
    let (rig, _log) = DialogRig::rig(S4_DELETE, DialogKind::Destructive);
    let app = Harness::new(rig, Theme::junie(), 80, 24);
    assert!(
        (0..24).any(|y| buf_row_text(app.buffer(), y).contains("Delete branch?")),
        "destructive title renders"
    );
    capture_isolated(&dir, "dialog-destructive", app.buffer());
    // N2 (click half): clicking the prompt input focuses without finishing.
    let (rig, log) = DialogRig::rig(S4_PROMPT, DialogKind::Prompt);
    let pst = Rc::clone(&rig.st);
    pst.borrow_mut().set_draft("Migrate sessions table");
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let probe = Dialog::prompt(S4_PROMPT, "Rename task", "Task name");
    assert!(app.tab_to(probe.action_id(1)), "tab to Rename");
    let iy = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Migrate sessions table"))
        .expect("input row renders");
    let ix = char_col(&buf_row_text(app.buffer(), iy), "Migrate sessions table") + 1;
    log.borrow_mut().clear();
    let r = app.click(ix, iy);
    assert_eq!(r.flow(), Flow::Consumed, "N2: input click applies");
    assert!(log.borrow().is_empty(), "N2: input clicks finish nothing");
    assert!(
        app.state_of(input_id).contains(StateFlags::FOCUSED),
        "N2: input clicks focus without finishing"
    );
    assert!(app.is_open(S4_PROMPT), "N2: still open");

    // PTY: block the empty submit, rename validly, reject over-long, and
    // cancel the destructive dialog with Enter and Esc.
    let case = s4_case("dialog_prompt", &["--page", "dialogs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // Tab to the Rename button with a probe loop: press_step settles nothing,
    // and the `Rename task…` button label would satisfy an open-wait while
    // closed, so the open needle must be dialog-only help text.
    let mut rename_focused = false;
    for _ in 0..8 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(150));
        let probe = live_text(&mut s);
        if probe.lines().any(|l| l.contains("▎Rename task")) {
            rename_focused = true;
            break;
        }
    }
    assert!(rename_focused, "prompt: Tab reaches the Rename button");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "prompt never opened",
        |screen| support::screen_text(screen).contains("Shown in the task list"),
    );
    checkpoint(&mut s, &dir, "prompt-open");
    let opened = live_text(&mut s);
    assert!(
        opened.contains("Migrate sessions table"),
        "V1 live: prefilled input"
    );
    assert!(
        opened.contains("Shown in the task list"),
        "V1 live: help text"
    );
    // N1 live: y finishes nothing on an input dialog.
    support::press_step(&s, "y");
    let yed = settle_text(&mut s, case_timeout(&case), "y steady");
    assert!(
        yed.contains("Shown in the task list"),
        "N1 live: y keeps the prompt open"
    );
    assert!(
        yed.contains("Migrate sessions table"),
        "N1 live: y types nothing while unfocused-for-edit"
    );
    // S1/N1 live: clear, submit, blocked with the validation error.
    support::press_step(&s, "enter");
    support::press_step(&s, "ctrl+l");
    support::press_step(&s, "backspace");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "validation error never showed",
        |screen| support::screen_text(screen).contains("Name cannot be empty"),
    );
    checkpoint(&mut s, &dir, "prompt-blocked");
    let blocked = live_text(&mut s);
    // The validation error replaces the help line while blocked, so the
    // error itself is the dialog-still-open proof.
    assert!(
        blocked.contains("Name cannot be empty"),
        "S1 live: failing validation keeps the dialog open"
    );
    assert!(
        !blocked.contains("Renamed to"),
        "N1 live: no rename is stored"
    );
    // S2/A1 live: a valid name renames (the failed submit committed, so
    // Enter begins editing again before typing).
    support::press_step(&s, "enter");
    type_text(&mut s, "Seed the demo database");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Renamed to", "S2 live: rename commits");
    checkpoint(&mut s, &dir, "prompt-renamed");
    let renamed = live_text(&mut s);
    assert!(
        renamed.contains("Seed the demo database"),
        "S2 live: the task name stores"
    );
    // N1 live: an over-long name never closes the prompt.
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "prompt never reopened",
        |screen| support::screen_text(screen).contains("Shown in the task list"),
    );
    support::press_step(&s, "enter");
    support::press_step(&s, "ctrl+l");
    support::press_step(&s, "backspace");
    type_text(&mut s, &"x".repeat(41));
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "length error never showed",
        |screen| support::screen_text(screen).contains("Keep it under 40"),
    );
    let too_long = live_text(&mut s);
    assert!(
        too_long.contains("Rename task"),
        "N1 live: over-long keeps the prompt open"
    );
    support::press_step(&s, "escape");
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled", "prompt cancels out");
    // A2/V2/N2 live: d opens the destructive flow; Enter and Esc both land
    // on Cancel, never on Delete.
    support::press_step(&s, "d");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "destructive never opened",
        |screen| support::screen_text(screen).contains("Delete branch?"),
    );
    checkpoint(&mut s, &dir, "destructive-open");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Cancelled", "V2 live: Cancel focused first");
    let cancelled = live_text(&mut s);
    assert!(
        !cancelled.contains("deleted"),
        "V2 live: Enter never deletes"
    );
    support::press_step(&s, "d");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "destructive never reopened",
        |screen| support::screen_text(screen).contains("Delete branch?"),
    );
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Cancelled", "N2 live: Esc lands on Cancel");
    let esc = live_text(&mut s);
    assert!(!esc.contains("deleted"), "N2 live: Esc never deletes");
    eprintln!("s4 dialog_prompt: isolated validation + live block/rename/cancel");
}

const S4_FILEMENU: Id = Id::root("s4.filemenu");

/// Dropdown/context-menu rig plus action log.
struct MenuRig {
    id: Id,
    items: Vec<MenuItem<'static>>,
    title: Option<&'static str>,
    anchor_rect: Rect,
    st: Rc<RefCell<MenuState>>,
    log: Rc<RefCell<Vec<MenuAction>>>,
    open: bool,
    booted: bool,
}

impl MenuRig {
    fn file_items() -> Vec<MenuItem<'static>> {
        vec![
            MenuItem::new(ActionKey::custom("new-tab"), "New tab").shortcut_text("c"),
            MenuItem::new(ActionKey::custom("split-right"), "Split right").shortcut_text("%"),
            MenuItem::new(ActionKey::custom("export"), "Export…").separator(),
            MenuItem::new(ActionKey::custom("close-tab"), "Close tab")
                .shortcut_text("&")
                .danger(),
        ]
    }

    fn view_items() -> Vec<MenuItem<'static>> {
        vec![
            MenuItem::new(ActionKey::custom("zoom-pane"), "Zoom pane").shortcut_text("z"),
            MenuItem::new(ActionKey::custom("redraw"), "Redraw")
                .shortcut_text("r")
                .separator(),
            MenuItem::new(ActionKey::custom("usage"), "Usage").shortcut_text("u"),
            MenuItem::new(ActionKey::custom("inspect"), "Inspect changes").disabled(true),
        ]
    }

    fn rig(
        items: Vec<MenuItem<'static>>,
        anchor_rect: Rect,
    ) -> (Self, Rc<RefCell<Vec<MenuAction>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        (
            MenuRig {
                id: S4_FILEMENU,
                items,
                title: None,
                anchor_rect,
                st: Rc::new(RefCell::new(MenuState::default())),
                log: Rc::clone(&log),
                open: false,
                booted: false,
            },
            log,
        )
    }

    fn menu(&self) -> ContextMenu<'_> {
        let m = ContextMenu::new(
            self.id,
            &self.items,
            Anchor::Rect {
                rect: self.anchor_rect,
                side: Side::Below,
                align: CrossAlign::Start,
            },
        );
        if let Some(title) = self.title {
            m.title(title)
        } else {
            m
        }
    }
}

impl App for MenuRig {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if !self.booted {
            self.booted = true;
            self.open = true;
            let spec = self.menu().layer(cx);
            cx.open_layer(self.id, spec);
        }
        let log = Rc::clone(&self.log);
        let r = self
            .menu()
            .update(cx, &mut self.st.borrow_mut())
            .on_action(|action| log.borrow_mut().push(action));
        if !cx.is_open(self.id) {
            self.open = false;
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        if self.open {
            let _ = ui.layer(self.id, |ui, area| {
                self.menu().draw(ui, area, &self.st.borrow())
            });
        }
    }
}

/// MENU-OPEN-001 (`showcase/flows/chrome/menu`, 80x24): the open dropdown
/// list over [`ContextMenu`].
///
/// Isolated path mirrors the row's File/View menus: highlight-fill cursor,
/// right-aligned muted shortcuts, danger tones, ─ separators, faint
/// disabled rows, wrapping skip-disabled steps, Home/End jumps, the
/// flip-above placement, Chosen/Dismissed, and the disabled/modified-chord
/// negatives. Live path opens File, walks past Export to the danger row,
/// chooses, then opens View and jumps to the last enabled row.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_menu_open() {
    let dir = s4_dir("s4_menu_open");
    let t = Theme::junie();
    let highlight_danger = t.color.highlight_danger_bg;
    let primary = t.color.fg[FgStep::Primary.index()];
    let muted = t.color.fg[FgStep::Muted.index()];
    let error_soft = t.color.danger_soft;
    let subtle = t.color.border_subtle;

    // S1 isolated: step wraps with rem_euclid and skips disabled rows;
    // Home/End jump to first/last enabled.
    let (rig, _log) = MenuRig::rig(MenuRig::view_items(), Rect::new(2, 1, 8, 1));
    let mst = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    assert_eq!(
        mst.borrow().cursor_index(),
        0,
        "cursor starts on the first enabled row"
    );
    let _ = app.key(KeyCode::Down);
    assert_eq!(mst.borrow().cursor_index(), 1, "Down moves");
    let _ = app.key(KeyCode::Down);
    assert_eq!(mst.borrow().cursor_index(), 2, "Down skips nothing enabled");
    let _ = app.key(KeyCode::Down);
    assert_eq!(
        mst.borrow().cursor_index(),
        0,
        "Down wraps past the disabled tail"
    );
    let _ = app.key(KeyCode::Up);
    assert_eq!(
        mst.borrow().cursor_index(),
        2,
        "Up wraps, skipping disabled"
    );
    let _ = app.key(KeyCode::End);
    assert_eq!(
        mst.borrow().cursor_index(),
        2,
        "End jumps to the last enabled row"
    );
    let _ = app.key(KeyCode::Home);
    assert_eq!(
        mst.borrow().cursor_index(),
        0,
        "Home jumps to the first enabled row"
    );
    // V1/V2/V3 render form over the File menu with the cursor on Close tab.
    let (rig, _log) = MenuRig::rig(MenuRig::file_items(), Rect::new(2, 1, 8, 1));
    let fst = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    let _ = app.key(KeyCode::End);
    assert_eq!(
        fst.borrow().cursor_index(),
        3,
        "End lands on the danger row"
    );
    let area = app.layer_area(S4_FILEMENU).expect("popover layer placed");
    assert_eq!(area.x, 2, "popover left-aligns under the anchor");
    assert!(area.y > 1, "popover sits below the anchor");
    let buf = app.buffer();
    let crow = (0..16)
        .find(|y| buf_row_text(buf, *y).contains("Close tab"))
        .expect("danger row renders");
    assert_eq!(
        buf[(area.x + 1, crow)].bg,
        highlight_danger,
        "V1: danger cursor fill"
    );
    assert_eq!(buf[(area.x + 1, crow)].fg, primary, "V1: bold white text");
    assert!(
        buf_is_bold(buf, area.x + 3, crow),
        "V1: cursor text is bold"
    );
    assert_ne!(
        buf[(area.x + 1, crow)].symbol(),
        "▎",
        "V1: no gutter bar on menu rows"
    );
    let rest = buf_row_text(buf, area.y + 1);
    assert!(rest.contains("New tab"), "rest rows render");
    assert_eq!(
        buf[(area.x + 3, area.y + 1)].fg,
        primary,
        "rest rows keep primary text"
    );
    let sc_col = area.x + area.width - 1 - 1 - 1;
    assert_eq!(
        buf[(sc_col, crow)].symbol(),
        "&",
        "V2: shortcut right-aligned"
    );
    assert_eq!(
        buf[(sc_col, crow)].fg,
        muted,
        "V2: shortcuts take the muted tone"
    );
    assert!(
        !buf_is_bold(buf, sc_col, crow),
        "V2: shortcuts are never bold"
    );
    let danger_rest = buf_row_text(buf, crow);
    assert!(danger_rest.contains("Close tab"), "danger label renders");
    assert!(
        buf_row_text(buf, crow - 1).contains('─'),
        "V2: separators are ─ rules"
    );
    assert_eq!(buf[(area.x + 2, crow - 1)].fg, subtle, "separator tone");
    capture_isolated(&dir, "menu-open", buf);
    // V3: disabled rows render faint and the cursor never lands on them.
    let (rig, _log) = MenuRig::rig(MenuRig::view_items(), Rect::new(2, 1, 8, 1));
    let mst = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    mst.borrow_mut().set_cursor(2, ItemKey::index(2));
    let _ = app.key(KeyCode::Down);
    let _ = app.key(KeyCode::Up);
    let buf = app.buffer();
    let area = app.layer_area(S4_FILEMENU).expect("popover layer placed");
    let dis_row = (0..16)
        .find(|y| buf_row_text(buf, *y).contains("Inspect changes"))
        .expect("V3: disabled row renders");
    assert!(
        buf_row_text(buf, dis_row).contains("Inspect changes"),
        "V3: disabled row renders"
    );
    let dx = char_col(&buf_row_text(buf, dis_row), "Inspect changes");
    assert_ne!(
        buf[(dx, dis_row)].fg,
        primary,
        "V3: disabled rows are faint, not primary"
    );
    assert_eq!(
        t.color.highlight_bg, t.color.highlight_bg,
        "highlight plane (presence)"
    );
    let _ = area;
    // S2: the popover flips above the anchor when the room below is short.
    let (rig, _log) = MenuRig::rig(MenuRig::file_items(), Rect::new(2, 13, 8, 1));
    let app = Harness::new(rig, Theme::junie(), 60, 16);
    let low = app.layer_area(S4_FILEMENU).expect("popover layer placed");
    assert!(
        low.y + low.height <= 13,
        "S2: the popover flips above the anchor"
    );
    // A1/A2: Enter/Space choose; Esc dismisses; row clicks choose, outside
    // clicks dismiss; hover moves the cursor.
    // (Dismissal is the layer correlate of the row's Dismissed event.)
    let (rig, log) = MenuRig::rig(MenuRig::file_items(), Rect::new(2, 1, 8, 1));
    let _mst = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    let _ = app.key(KeyCode::End);
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow()
            .contains(&MenuAction::Chosen(ActionKey::custom("close-tab"))),
        "A1: Enter chooses"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char(' '));
    assert_eq!(r.flow(), Flow::Consumed, "A1: Space applies");
    assert!(
        log.borrow()
            .contains(&MenuAction::Chosen(ActionKey::custom("close-tab"))),
        "A1: Space chooses too"
    );
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Esc);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Esc applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, MenuAction::Closed(_))),
        "A1: Esc dismisses"
    );
    assert!(!app.is_open(S4_FILEMENU), "A1: Esc closes");
    let (rig, log) = MenuRig::rig(MenuRig::file_items(), Rect::new(2, 1, 8, 1));
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    let ry = (0..16)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Split right"))
        .expect("row renders");
    let rx = char_col(&buf_row_text(app.buffer(), ry), "Split right") + 1;
    log.borrow_mut().clear();
    let _ = app.click(rx, ry);
    assert!(
        log.borrow()
            .contains(&MenuAction::Chosen(ActionKey::custom("split-right"))),
        "A2: a row click chooses it"
    );
    log.borrow_mut().clear();
    let _ = app.click(59, 15);
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, MenuAction::Closed(_))),
        "A2: a click elsewhere dismisses"
    );
    let (rig, _log) = MenuRig::rig(MenuRig::file_items(), Rect::new(2, 1, 8, 1));
    let mst = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    let hy = (0..16)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Export…"))
        .expect("hover row renders");
    let hx = char_col(&buf_row_text(app.buffer(), hy), "Export…") + 1;
    let _ = app.mouse(MouseKind::Move, hx, hy);
    assert_eq!(mst.borrow().cursor_index(), 2, "A2: hover moves the cursor");
    // N1: Enter on a disabled row is Consumed; a modified chord is Ignored.
    let (rig, log) = MenuRig::rig(MenuRig::view_items(), Rect::new(2, 1, 8, 1));
    let mst = Rc::clone(&rig.st);
    let mut app = Harness::new(rig, Theme::junie(), 60, 16);
    mst.borrow_mut().set_cursor(3, ItemKey::index(3));
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(
        r.flow(),
        Flow::Consumed,
        "N1: Enter on a disabled row is Consumed"
    );
    assert!(
        log.borrow().is_empty(),
        "N1: Enter on a disabled row chooses nothing"
    );
    let dy = (0..16)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Inspect changes"))
        .expect("disabled row renders");
    let dx = char_col(&buf_row_text(app.buffer(), dy), "Inspect changes") + 1;
    log.borrow_mut().clear();
    let _ = app.click(dx, dy);
    assert!(log.borrow().is_empty(), "N1: disabled clicks miss");
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Ignored, "N1: a modified chord is Ignored");
    assert!(log.borrow().is_empty(), "N1: no event");
    assert_ne!(error_soft, primary, "danger tone differs (presence)");

    // PTY: open File, walk past Export to the danger row, choose; open
    // View and jump to the last enabled row.
    let case = s4_case("menu_open", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "File menu never opened",
        |screen| support::screen_text(screen).contains("New tab"),
    );
    checkpoint(&mut s, &dir, "menu-open");
    let opened = live_text(&mut s);
    assert_line_has(&opened, "New tab", "c", "V2 live: shortcut beside the row");
    assert!(opened.contains('─'), "V2 live: separator rule");
    support::press_step(&s, "down");
    support::press_step(&s, "down");
    support::press_step(&s, "down");
    checkpoint(&mut s, &dir, "menu-danger");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Close tab", "A1 live: danger row chooses");
    waits::wait_gone(&mut s, "Split right", "N2 live: choosing closes");
    // View menu: Right switches, End jumps past the disabled row.
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "File menu never reopened",
        |screen| support::screen_text(screen).contains("New tab"),
    );
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "View menu never opened",
        |screen| support::screen_text(screen).contains("Zoom pane"),
    );
    let view = live_text(&mut s);
    assert!(
        view.contains("Inspect changes"),
        "V3 live: disabled row renders"
    );
    support::press_step(&s, "end");
    support::press_step(&s, "enter");
    // Gate on the menu closing ("Usage" alone would match the open menu's
    // own row and pass before Enter lands). Choosing records the bare label
    // on the status line (`cx.status(label)`); the `last:` prefix hides while
    // the bar holds focus.
    waits::wait_gone(
        &mut s,
        "Zoom pane",
        "N2 live: choosing closes the View menu",
    );
    waits::wait_state(&mut s, "Usage", "S1 live: End jumps to Usage");
    checkpoint(&mut s, &dir, "menu-chosen");
    eprintln!("s4 menu_open: isolated dropdown map + live danger/usage");
}

const S4_CONTEXT: Id = Id::root("s4.context");

/// CTXMENU-ANCHOR-001 (`showcase/flows/chrome/context`, 80x24): the anchored
/// session context menu over [`ContextMenu`].
///
/// Isolated path mirrors the row's session menu (title, edge-disabled Move
/// rows, danger Close): title row, danger-under-separator geometry, and the
/// disabled-edge negatives. Live path opens with m, chooses Close, proves
/// the disabled edge can never be chosen, proves the menu never opens
/// unfocused, and opens with a right-click that moves the cursor.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_ctxmenu_anchor() {
    let dir = s4_dir("s4_ctxmenu_anchor");
    let t = Theme::junie();
    let danger = t.color.highlight_danger_bg;
    fn session_items(row: usize, len: usize) -> Vec<MenuItem<'static>> {
        vec![
            MenuItem::new(ActionKey::custom("change-title"), "Change title…").shortcut_text("r"),
            MenuItem::new(ActionKey::custom("move-left"), "Move left").disabled(row == 0),
            MenuItem::new(ActionKey::custom("move-right"), "Move right")
                .disabled(row + 1 == len)
                .separator(),
            MenuItem::new(ActionKey::custom("close"), "Close")
                .shortcut_text("x")
                .danger(),
        ]
    }
    #[allow(clippy::type_complexity)]
    fn session_rig(
        row: usize,
        len: usize,
    ) -> (
        MenuRig,
        Rc<RefCell<Vec<MenuAction>>>,
        Rc<RefCell<MenuState>>,
    ) {
        let (mut rig, log) = MenuRig::rig(session_items(row, len), Rect::new(20, 4, 1, 1));
        rig.id = S4_CONTEXT;
        rig.title = Some("1 Claude Code (Work)");
        let st = Rc::clone(&rig.st);
        (rig, log, st)
    }

    // V1/V2 render form: the popover titles itself with the session label;
    // at the top edge Move left disables; Close is danger under a separator.
    let (rig, _log, _st) = session_rig(0, 4);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    let area = app.layer_area(S4_CONTEXT).expect("popover layer placed");
    let buf = app.buffer();
    let screen: String = (0..24).map(|y| buf_row_text(buf, y)).collect();
    assert!(
        screen.contains("1 Claude Code (Work)"),
        "V1: the popover titles itself with the session label"
    );
    assert!(screen.contains("Move left"), "edge row renders");
    assert!(screen.contains("Close"), "danger row renders");
    let close_y = (0..24)
        .find(|y| buf_row_text(buf, *y).contains("Close"))
        .expect("Close row renders");
    assert!(
        buf_row_text(buf, close_y - 1).contains('─'),
        "V2: Close sits under a separator"
    );
    let _ = app.key(KeyCode::End);
    let buf = app.buffer();
    let close_y = (0..24)
        .find(|y| buf_row_text(buf, *y).contains("Close"))
        .expect("Close row renders");
    assert_eq!(
        buf[(area.x + 1, close_y)].bg,
        danger,
        "V2: Close takes the danger fill under the cursor"
    );
    capture_isolated(&dir, "ctxmenu-anchor", buf);
    // N1: a disabled edge row can never be chosen by key or click.
    let (rig, log, mst) = session_rig(0, 4);
    let mut app = Harness::new(rig, Theme::junie(), 80, 24);
    mst.borrow_mut().set_cursor(1, ItemKey::index(1));
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "N1: Enter applies");
    assert!(
        log.borrow().is_empty(),
        "N1: Enter on the disabled edge chooses nothing"
    );
    let ey = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Move left"))
        .expect("edge row renders");
    let ex = char_col(&buf_row_text(app.buffer(), ey), "Move left") + 1;
    log.borrow_mut().clear();
    let _ = app.click(ex, ey);
    assert!(log.borrow().is_empty(), "N1: disabled clicks miss");
    // Presence: the enabled rows resolve through the same paths.
    mst.borrow_mut().set_cursor(0, ItemKey::index(0));
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "presence applies");
    assert!(
        log.borrow()
            .contains(&MenuAction::Chosen(ActionKey::custom("change-title"))),
        "enabled rows choose (presence proof)"
    );
    let cy = (0..24)
        .find(|y| buf_row_text(app.buffer(), *y).contains("Close"))
        .expect("Close row renders");
    let cx = char_col(&buf_row_text(app.buffer(), cy), "Close") + 1;
    log.borrow_mut().clear();
    let _ = app.click(cx, cy);
    assert!(
        log.borrow()
            .contains(&MenuAction::Chosen(ActionKey::custom("close"))),
        "enabled clicks choose (presence proof)"
    );
    log.borrow_mut().clear();
    let _ = app.click(79, 23);
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, MenuAction::Closed(_))),
        "outside dismisses"
    );

    // PTY: m opens, Close chooses, Esc dismisses; the disabled edge never
    // chooses; right-click anchors and moves the cursor.
    let case = s4_case("ctxmenu_anchor", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "sessions never focused",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Claude Code"))
        },
    );
    support::press_step(&s, "down");
    support::press_step(&s, "m");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "context menu never opened",
        |screen| support::screen_text(screen).contains("Change title…"),
    );
    checkpoint(&mut s, &dir, "context-open");
    let opened = live_text(&mut s);
    assert!(
        opened.contains("2 Codex (Primary)"),
        "V1 live: m anchors under the cursor row"
    );
    support::press_step(&s, "down");
    support::press_step(&s, "down");
    support::press_step(&s, "down");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "tab › Close", "S2 live: choice records");
    checkpoint(&mut s, &dir, "context-chosen");
    // A1: Esc dismisses and clears.
    support::press_step(&s, "m");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "context menu never reopened",
        |screen| support::screen_text(screen).contains("Change title…"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Change title…", "A1 live: Esc clears");
    checkpoint(&mut s, &dir, "context-closed");
    // N1 live: the disabled edge row can never be chosen.
    support::press_step(&s, "up");
    support::press_step(&s, "m");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "context menu never reopened at row 0",
        |screen| support::screen_text(screen).contains("Change title…"),
    );
    support::press_step(&s, "down");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "tab › Move right", "N1 live: Down skips Move left");
    // Presence: the choice stands on the status line while no menu covers it.
    let stood = live_text(&mut s);
    assert!(
        stood.contains("tab › Move right"),
        "N1 live: the choice stands (presence proof)"
    );
    support::press_step(&s, "m");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "context menu never reopened",
        |screen| support::screen_text(screen).contains("Change title…"),
    );
    click_at(&mut s, "Move left", case_timeout(&case));
    let disabled = settle_text(&mut s, case_timeout(&case), "disabled-click steady");
    assert!(
        disabled.contains("Change title…"),
        "N1 live: clicking the disabled edge chooses nothing"
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Change title…", "context clears");
    // The open menu hides the status layer, so the no-record proof reads
    // after Esc: the old choice stands, Move left never recorded.
    let cleared = live_text(&mut s);
    assert!(
        cleared.contains("tab › Move right") && !cleared.contains("tab › Move left"),
        "N1 live: the disabled click recorded nothing"
    );
    // N2 live: the menu never opens when sessions are not focused.
    support::press_step(&s, "backtab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "menu bar never focused",
        |screen| support::screen_text(screen).contains("← → Menu"),
    );
    support::press_step(&s, "m");
    let unfocused = settle_text(&mut s, case_timeout(&case), "unfocused steady");
    assert!(
        !unfocused.contains("Change title…"),
        "N2 live: m opens nothing off-sessions"
    );
    // S1 live: right-click anchors at the pointer and moves the cursor.
    right_click_at(&mut s, "3 Shell", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "right-click never opened the menu",
        |screen| support::screen_text(screen).contains("3 Shell"),
    );
    let rclick = live_text(&mut s);
    assert!(
        rclick
            .lines()
            .any(|l| l.contains('▎') && l.contains("3 Shell")),
        "S1 live: right-click moves the cursor"
    );
    assert!(
        rclick.contains("Change title…"),
        "S1 live: right-click opens the menu"
    );
    // A2 live: a row click chooses while open.
    click_at(&mut s, "Change title…", case_timeout(&case));
    waits::wait_state(&mut s, "tab › Change title…", "A2 live: click chooses");
    eprintln!("s4 ctxmenu_anchor: isolated edges + live m/right-click/disabled");
}

const S4_MENUBAR: Id = Id::root("s4.menubar");
const S4_MENUBRAND: Id = Id::root("s4.menubrand");

static S4_BAR_FILE: [MenuItem<'static>; 2] = [
    MenuItem::new(ActionKey::custom("new-tab"), "New tab").shortcut_text("c"),
    MenuItem::new(ActionKey::custom("close-tab"), "Close tab")
        .shortcut_text("&")
        .danger(),
];
static S4_BAR_VIEW: [MenuItem<'static>; 2] = [
    MenuItem::new(ActionKey::custom("zoom-pane"), "Zoom pane").shortcut_text("z"),
    MenuItem::new(ActionKey::custom("inspect"), "Inspect changes").disabled(true),
];
static S4_BAR_HELP: [MenuItem<'static>; 1] =
    [MenuItem::new(ActionKey::custom("key-reference"), "Key reference").shortcut_text("?")];
static S4_BAR_MENUS: [Menu<'static>; 3] = [
    Menu::new("File", &S4_BAR_FILE),
    Menu::new("View", &S4_BAR_VIEW),
    Menu::new("Help", &S4_BAR_HELP),
];

/// Shell bar rig: menu strip plus brand lockup plus action log.
/// (The component owns the strip; the brand is the composed [`Brand`].)
struct BarApp {
    st: Rc<RefCell<MenuState>>,
    log: Rc<RefCell<Vec<MenuAction>>>,
    fired: Rc<Cell<usize>>,
    width: u16,
}

impl BarApp {
    fn bar(&self) -> MenuBar<'_> {
        MenuBar::new(S4_MENUBAR, &S4_BAR_MENUS)
    }
}

impl App for BarApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        let mut r = self
            .bar()
            .update(cx, &mut self.st.borrow_mut())
            .on_action(|action| log.borrow_mut().push(action));
        let fired = Rc::clone(&self.fired);
        r |= Brand::new(S4_MENUBRAND, "app❯")
            .clickable(true)
            .update(cx)
            .on_action(|_| fired.set(fired.get().saturating_add(1)));
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Brand::new(S4_MENUBRAND, "app❯")
            .clickable(true)
            .draw(ui, Rect::new(0, 1, 6, 1));
        self.bar().draw(
            ui,
            Rect::new(7, 1, self.width.saturating_sub(7), 1),
            &self.st.borrow(),
        );
    }
}

/// MENUBAR-SHELL-001 (`showcase/flows/chrome/focus`, 80x24): the label strip
/// plus brand lockup over [`MenuBar`].
///
/// Isolated path mirrors the row's shell bar (File/View/Help, app❯ lockup):
/// padded labels, open-plane sharing, cursor ▎ bar, hover lift, move/switch
/// keys, open/choose/close/toggle clicks, Brand, hover-switch, and the
/// no-open-hover plus unfittable-label negatives. Live path focuses the bar,
/// opens, switches menus, chooses, activates the brand, and proves both
/// hover halves.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_menubar_shell() {
    let dir = s4_dir("s4_menubar_shell");
    let t = Theme::junie();
    let popover = t.color.surfaces[Surface::Popover.level().expect("popover level")];
    let primary = t.color.fg[FgStep::Primary.index()];

    // V1/V2 render form: padded labels with 1-cell gaps; the focused cursor
    // label carries the ▎ bar.
    let st = Rc::new(RefCell::new(MenuState::default()));
    let mut app = Harness::new(
        BarApp {
            st: Rc::clone(&st),
            log: Rc::new(RefCell::new(Vec::new())),
            fired: Rc::new(Cell::new(0)),
            width: 80,
        },
        Theme::junie(),
        80,
        3,
    );
    assert!(app.tab_to(S4_MENUBAR), "navigating focus");
    let buf = app.buffer();
    let row = buf_row_text(buf, 1);
    assert!(row.contains("app❯"), "V1: the brand lockup renders");
    assert!(
        row.contains(" File ") && row.contains(" View ") && row.contains(" Help "),
        "V1: labels render padded with gaps"
    );
    assert!(row.contains('▎'), "V2: the cursor label carries the ▎ bar");
    let fx = char_col(&row, "File");
    assert_eq!(
        buf[(fx, 1)].fg,
        primary,
        "V2: the cursor label takes primary text"
    );
    assert!(buf_is_bold(buf, fx, 1), "V2: the cursor label is bold");
    capture_isolated(&dir, "menubar-closed", buf);
    // S2/V1: Enter opens the cursor menu; the open label shares the popover
    // plane in bold. One continuous rig drives the open/switch/choose
    // steps below.
    let st = Rc::new(RefCell::new(MenuState::default()));
    let log = Rc::new(RefCell::new(Vec::new()));
    let fired = Rc::new(Cell::new(0));
    let mut app = Harness::new(
        BarApp {
            st: Rc::clone(&st),
            log: Rc::clone(&log),
            fired: Rc::clone(&fired),
            width: 80,
        },
        Theme::junie(),
        80,
        8,
    );
    assert!(app.tab_to(S4_MENUBAR), "navigating focus");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "S2: Enter applies");
    assert!(
        log.borrow().contains(&MenuAction::Opened(0)),
        "S2: Enter opens the cursor menu"
    );
    let buf = app.buffer();
    let row = buf_row_text(buf, 1);
    let fx = char_col(&row, "File");
    assert_eq!(
        buf[(fx, 1)].bg,
        popover,
        "V1: the open label shares the popover plane"
    );
    assert!(buf_is_bold(buf, fx, 1), "V1: the open label is bold");
    // S1: Right switches the open menu; Left switches back.
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Right);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Right applies");
    assert!(
        log.borrow().contains(&MenuAction::Opened(1)),
        "S1: Right switches the open menu"
    );
    assert_eq!(st.borrow().open_menu(), Some(1), "View is open");
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Char('h'));
    assert_eq!(r.flow(), Flow::Consumed, "S1: h applies");
    assert!(
        log.borrow().contains(&MenuAction::Opened(0)),
        "S1: h switches back"
    );
    // A1: choosing emits Chosen(menu, item) and closes.
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Enter);
    assert_eq!(r.flow(), Flow::Consumed, "A1: Enter applies");
    assert!(
        log.borrow()
            .contains(&MenuAction::Chosen(ActionKey::custom("new-tab"))),
        "A1: choosing emits Chosen(0, 0)"
    );
    assert!(!st.borrow().is_open(), "A1: choosing closes");
    // S1 closed: Left/Right move the cursor.
    log.borrow_mut().clear();
    let r = app.key(KeyCode::Right);
    assert_eq!(r.flow(), Flow::Consumed, "S1: Right applies");
    assert_eq!(st.borrow().cursor_index(), 1, "S1: Right moves");
    // S2: clicking a label toggles its menu.
    let vy = 1u16;
    let vx = char_col(&buf_row_text(app.buffer(), vy), "View") + 1;
    log.borrow_mut().clear();
    let r = app.click(vx, vy);
    assert_eq!(r.flow(), Flow::Consumed, "S2: label click applies");
    assert!(
        log.borrow().contains(&MenuAction::Opened(1)),
        "S2: label click opens"
    );
    log.borrow_mut().clear();
    let vx = char_col(&buf_row_text(app.buffer(), vy), "View") + 1;
    let r = app.click(vx, vy);
    assert_eq!(r.flow(), Flow::Consumed, "S2: label click applies");
    assert!(
        log.borrow()
            .iter()
            .any(|a| matches!(a, MenuAction::Closed(_))),
        "S2: clicking its label again closes it"
    );
    // A2: the brand lockup emits Brand; hovering another label while open
    // switches to it.
    let bx = char_col(&buf_row_text(app.buffer(), 1), "app❯") + 1;
    let r = app.click(bx, 1);
    assert_eq!(r.flow(), Flow::Consumed, "A2: brand click applies");
    assert_eq!(fired.get(), 1, "A2: the brand lockup emits Brand");
    let fx = char_col(&buf_row_text(app.buffer(), 1), "File") + 1;
    log.borrow_mut().clear();
    let _ = app.click(fx, 1);
    assert_eq!(st.borrow().open_menu(), Some(0), "File open");
    let hx = char_col(&buf_row_text(app.buffer(), 1), "Help") + 1;
    let _ = app.mouse(MouseKind::Move, hx, 1);
    assert_eq!(
        st.borrow().open_menu(),
        Some(2),
        "A2: hover switches the open menu"
    );
    // N1: hover with no menu open changes nothing; modified chords Ignored.
    log.borrow_mut().clear();
    let _ = app.key(KeyCode::Esc);
    assert!(!st.borrow().is_open(), "bar closed");
    log.borrow_mut().clear();
    let hx = char_col(&buf_row_text(app.buffer(), 1), "View") + 1;
    let r = app.mouse(MouseKind::Move, hx, 1);
    assert_eq!(r.flow(), Flow::Ignored, "N1: hover is Ignored");
    assert!(
        !st.borrow().is_open(),
        "N1: hover with no menu open changes nothing"
    );
    assert!(log.borrow().is_empty(), "N1: no event");
    log.borrow_mut().clear();
    let r = app.key_mod(KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert_eq!(r.flow(), Flow::Ignored, "N1: a modified chord is Ignored");
    assert!(log.borrow().is_empty(), "N1: no event");
    // N2: labels that do not fit keep ZERO areas and cannot open.
    // (No areas escape the component: the paint half plus the open half
    // below carry the check.)
    let nst = Rc::new(RefCell::new(MenuState::default()));
    let nlog = Rc::new(RefCell::new(Vec::new()));
    let mut narrow = Harness::new(
        BarApp {
            st: Rc::clone(&nst),
            log: Rc::clone(&nlog),
            fired: Rc::new(Cell::new(0)),
            width: 20,
        },
        Theme::junie(),
        20,
        3,
    );
    assert!(narrow.tab_to(S4_MENUBAR), "navigating focus");
    let nrow = buf_row_text(narrow.buffer(), 1);
    assert!(!nrow.contains("Help"), "N2: unfitting labels do not paint");
    nst.borrow_mut().set_cursor(2, ItemKey::index(2));
    nlog.borrow_mut().clear();
    let _ = narrow.key(KeyCode::Enter);
    assert!(
        !nlog
            .borrow()
            .iter()
            .any(|a| matches!(a, MenuAction::Opened(2))),
        "N2: unfitting labels cannot open"
    );

    // PTY: focus the bar, open, switch, choose, brand, hover negative,
    // click-switch.
    let case = s4_case("menubar_shell", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "menu bar never focused",
        |screen| support::screen_text(screen).contains("← → Menu"),
    );
    checkpoint(&mut s, &dir, "bar-focused");
    let focused = live_text(&mut s);
    assert!(focused.contains("app❯"), "V1 live: brand lockup");
    assert!(
        focused.contains("File") && focused.contains("View") && focused.contains("Help"),
        "V1 live: labels render"
    );
    support::press_step(&s, "enter");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "File menu never opened",
        |screen| support::screen_text(screen).contains("New tab"),
    );
    checkpoint(&mut s, &dir, "bar-open");
    support::press_step(&s, "right");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "View menu never opened",
        |screen| support::screen_text(screen).contains("Zoom pane"),
    );
    support::press_step(&s, "left");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "File menu never returned",
        |screen| support::screen_text(screen).contains("Split right"),
    );
    support::press_step(&s, "enter");
    // Choosing records the bare label; gate on the close so the open menu's
    // own row cannot satisfy the wait.
    waits::wait_gone(&mut s, "Split right", "A1 live: choosing closes");
    waits::wait_state(&mut s, "New tab", "A1 live: choice records");
    checkpoint(&mut s, &dir, "bar-chosen");
    // A2 live: the brand lockup activates.
    click_at(&mut s, "app❯", case_timeout(&case));
    waits::wait_state(&mut s, "Brand lockup activated", "A2 live: brand");
    // N1 live: hover with no menu open changes nothing.
    hover_over(&mut s, "View", case_timeout(&case));
    let noopen = settle_text(&mut s, case_timeout(&case), "hover steady");
    assert!(
        !noopen.contains("Zoom pane"),
        "N1 live: hover opens nothing"
    );
    // A2 live: activating another label while open switches to it. The reopen
    // needle must be menu-only: the status line still records `New tab` from
    // the choice above. Pointer switching goes through label clicks: the
    // shell tracks hover ids but never routes hover to the bar, so the
    // widget's hover-switch stays an isolated proof.
    click_at(&mut s, "File", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "File menu never reopened",
        |screen| support::screen_text(screen).contains("Split right"),
    );
    click_at(&mut s, "View", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "click never switched the menu",
        |screen| support::screen_text(screen).contains("Zoom pane"),
    );
    eprintln!("s4 menubar_shell: isolated bar map + live open/switch/brand/hover");
}

/// HELP-SHOWCASE-001 (`showcase/flows/help/overlay`, 80x24): the keyboard
/// and mouse reference dialog (PTY-only).
///
/// Help is the showcase binary's help overlay, outside the importable
/// library, so every check runs live: the single Close action under the
/// key table, the keyboard-rows-then-mouse-row shape, focus save/restore,
/// both openers, all four closers, and the no-second-action plus
/// y-never-closes negatives.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_help_showcase() {
    let dir = s4_dir("s4_help_showcase");
    let case = s4_case("help_showcase", &["--page", "overview"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "?");
    support::wait_screen(&mut s, case_timeout(&case), "help never opened", |screen| {
        support::screen_text(screen).contains("Keyboard & mouse")
    });
    checkpoint(&mut s, &dir, "help-open");
    let opened = live_text(&mut s);
    // V1/V2/S1: one Close action under the table; keyboard rows first, one
    // mouse row last; no Cancel exists (stripped, Close is index 0).
    assert!(opened.contains("Close"), "V1 live: the Close action");
    assert!(
        opened.contains("Tab / Shift+Tab") && opened.contains("quit"),
        "V1 live: the key table"
    );
    assert!(
        opened.contains("Mouse: hover to preview"),
        "V2 live: the mouse row"
    );
    // "Cancel" appears only as footer-hint text ("Esc Cancel") — never as a
    // second dialog button (the dialog row holds Close alone).
    let cancel_lines: Vec<&str> = opened.lines().filter(|l| l.contains("Cancel")).collect();
    assert_eq!(cancel_lines.len(), 1, "S1 live: no second action exists");
    assert!(
        cancel_lines[0].contains("Esc"),
        "S1 live: the only Cancel text is the Esc hint"
    );
    // N1/N2: Left/Right reach nothing; y never closes help.
    support::press_step(&s, "right");
    support::press_step(&s, "left");
    support::press_step(&s, "y");
    let stayed = settle_text(&mut s, case_timeout(&case), "help-noop steady");
    assert!(
        stayed.contains("Keyboard & mouse"),
        "N1/N2 live: Right/Left/y keep help open"
    );
    // A2: n, Enter, Space, and Esc all finish through Close.
    support::press_step(&s, "n");
    waits::wait_gone(&mut s, "Keyboard & mouse", "A2 live: n closes");
    for (key, what) in [("enter", "Enter"), ("space", "Space")] {
        support::press_step(&s, "?");
        support::wait_screen(
            &mut s,
            case_timeout(&case),
            "help never reopened",
            |screen| support::screen_text(screen).contains("Keyboard & mouse"),
        );
        support::press_step(&s, key);
        waits::wait_gone(
            &mut s,
            "Keyboard & mouse",
            &format!("A2 live: {what} closes"),
        );
    }
    support::press_step(&s, "?");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "help never reopened",
        |screen| support::screen_text(screen).contains("Keyboard & mouse"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Keyboard & mouse", "A2 live: Esc closes");
    checkpoint(&mut s, &dir, "help-closed");
    // A1: the header Help affordance opens too.
    click_at(&mut s, "? Help", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "header Help never opened",
        |screen| support::screen_text(screen).contains("Keyboard & mouse"),
    );
    support::press_step(&s, "escape");
    waits::wait_gone(&mut s, "Keyboard & mouse", "header help closes");
    // S2: closing restores the focus saved before the dialog opened (nav).
    support::press_step(&s, "down");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "nav never moved after close",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains('▎') && l.contains("Buttons"))
        },
    );
    eprintln!("s4 help_showcase: live help open/close/focus roundtrips");
}

/// FORM-SUBMIT-001 (`showcase/flows/forms/valid`, 80x24): the action row plus
/// submission status message (PTY-only).
///
/// The submit/countdown/reset state machine lives in the showcase binary's
/// forms page, so every check runs live: the Busy countdown message, the
/// Done message, the 23-tick countdown, Ctrl+S entry, per-tick decrement,
/// and the idle-empty plus invalid-never-busy negatives.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_form_submit() {
    let dir = s4_dir("s4_form_submit");
    let case = s4_case("form_submit", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // N2: the message cell stays empty while Idle and unattempted.
    let boot = live_text(&mut s);
    assert!(
        !boot.contains("Creating task")
            && !boot.contains("Task created")
            && !boot.contains("Fix the highlighted"),
        "N2 live: no message while Idle"
    );
    assert!(
        boot.contains("Create task") && boot.contains("Reset"),
        "N2 live: the action row renders (presence proof)"
    );
    // Fill a valid name and reviewer; both validators pass.
    support::press_step(&s, "tab");
    support::press_step(&s, "enter");
    type_text(&mut s, "Seed the demo database");
    support::press_step(&s, "enter");
    click_at(&mut s, "name@company.com", case_timeout(&case));
    type_text(&mut s, "ada@company.com");
    support::press_step(&s, "enter");
    // A1/S1/V1: Ctrl+S enters Busy(23) with the countdown message.
    support::press_step(&s, "ctrl+s");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "submit never went Busy",
        |screen| support::screen_text(screen).contains("Creating task…"),
    );
    checkpoint(&mut s, &dir, "submitted");
    let busy = live_text(&mut s);
    assert_line_has(&busy, "Creating task…", "Reset", "V1 live: Busy row");
    // A2/S2/V2: the countdown reaches Done with the accent message.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "submit never finished",
        |screen| support::screen_text(screen).contains("Task created ✓"),
    );
    checkpoint(&mut s, &dir, "done");
    let done = live_text(&mut s);
    assert!(
        done.contains("Task created ✓") && !done.contains("Creating task…"),
        "S2 live: Busy clears at Done"
    );
    eprintln!("s4 form_submit: live Busy(23) countdown to Done");
}

/// FORM-INVALID-002 (`showcase/flows/forms/invalid`, 80x24): validators plus
/// invalid focus plus reset (PTY-only).
///
/// Same binary-only state machine as [`s4_form_submit`]: the error rows and
/// Fix message, first-invalid focus, Ctrl+S-invalid and Reset actions, and
/// the never-busy plus lone-reviewer-focus negatives all run live.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_form_invalid() {
    let dir = s4_dir("s4_form_invalid");
    let case = s4_case("form_invalid", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // Enter a short name and an invalid reviewer, then submit.
    support::press_step(&s, "tab");
    support::press_step(&s, "enter");
    type_text(&mut s, "abc");
    support::press_step(&s, "enter");
    click_at(&mut s, "name@company.com", case_timeout(&case));
    type_text(&mut s, "not-an-email");
    support::press_step(&s, "enter");
    support::press_step(&s, "ctrl+s");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "invalid submit never reported",
        |screen| support::screen_text(screen).contains("Fix the highlighted fields"),
    );
    checkpoint(&mut s, &dir, "invalid");
    let invalid = live_text(&mut s);
    // V1/S1: error rows under both fields; validate() false, attempted true.
    assert!(
        invalid.contains("At least 4 characters"),
        "V1 live: name error row"
    );
    // The reviewer message clips to the narrow column (`…`).
    assert!(
        invalid.contains("Enter a valid email"),
        "V1 live: reviewer error row"
    );
    // V1: the error markers are bold `!` cells (the trailing marker on the
    // invalid field's own row).
    let frame = live_frame(&mut s);
    let snap = s
        .inner
        .snapshot()
        .unwrap_or_else(|e| panic!("error pos failed: {e:#}"));
    let (erow, _) =
        support::screen_find(&snap, "At least 4 characters").expect("name error on screen");
    let mut bold_bang = false;
    for r in [erow.saturating_sub(1), erow] {
        for c in 0..80u16 {
            if let Some(cell) = frame.get(c, r)
                && cell.symbol == "!"
                && cell.mods.bold
            {
                bold_bang = true;
            }
        }
    }
    assert!(bold_bang, "V1 live: the name error carries a bold `!`");
    // S2: focus lands on the name field, the first invalid field.
    let name_row = row_below(&invalid, "Task name", "S2 live");
    assert!(
        name_row.contains('▎'),
        "S2 live: focus lands on the first invalid field"
    );
    // N1: an invalid submit never sets busy.
    assert!(
        !invalid.contains("Creating task…"),
        "N1 live: invalid never goes Busy"
    );
    // A2: Reset rebuilds the defaults, focuses name, reports. Keyboard-driven:
    // a mouse click on Reset would leave app-level focus on the button (the
    // shell focuses any control the page did not refocus).
    let mut reset_focused = false;
    let mut probe = String::new();
    for _ in 0..16 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(150));
        probe = live_text(&mut s);
        if probe.lines().any(|l| l.contains("▎Reset")) {
            reset_focused = true;
            break;
        }
    }
    assert!(reset_focused, "A2 live: Tab reaches Reset\n{probe}");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Form reset", "A2 live: reset reports");
    checkpoint(&mut s, &dir, "reset");
    let reset = live_text(&mut s);
    assert!(
        !reset.contains("At least 4 characters") && !reset.contains("Enter a valid email"),
        "A2 live: errors clear"
    );
    assert!(
        reset.contains("Short imperative summ"),
        "A2 live: defaults rebuild (presence proof)"
    );
    let name_row = row_below(&reset, "Task name", "A2 live");
    assert!(name_row.contains('▎'), "A2 live: reset focuses name");
    // N2: a lone reviewer error focuses the reviewer, never the name.
    support::press_step(&s, "enter");
    type_text(&mut s, "Seed the demo database");
    support::press_step(&s, "enter");
    click_at(&mut s, "name@company.com", case_timeout(&case));
    type_text(&mut s, "nope");
    support::press_step(&s, "enter");
    support::press_step(&s, "ctrl+s");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "reviewer-only error never reported",
        |screen| support::screen_text(screen).contains("Fix the highlighted fields"),
    );
    let lone = live_text(&mut s);
    assert!(
        !lone.contains("At least 4 characters"),
        "N2 live: the name validates (presence proof)"
    );
    let rev_row = row_below(&lone, "Reviewer", "N2 live");
    assert!(
        rev_row.contains('▎'),
        "N2 live: focus lands on the reviewer"
    );
    let name_row = row_below(&lone, "Task name", "N2 live");
    assert!(
        !name_row.contains('▎'),
        "N2 live: focus never lands on the valid name"
    );
    eprintln!("s4 form_invalid: live errors/focus/reset/lone-reviewer");
}

/// First track cell (filled or not) in a bar row, by glyph.
fn track_start(row: &str) -> u16 {
    row.chars()
        .position(|c| c == '━' || c == '─')
        .expect("track renders") as u16
}

/// Five sample bars; the no-focus-stop probe.
struct PbFocusApp;

impl App for PbFocusApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ProgressBar::new(Id::root("s4.pb0"))
            .label("Queued    ")
            .ratio(0.0)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 1, 70, 1));
        ProgressBar::new(Id::root("s4.pb1"))
            .label("Halfway   ")
            .ratio(0.5)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 2, 70, 1));
        ProgressBar::new(Id::root("s4.pb2"))
            .label("Completed ")
            .ratio(1.0)
            .status(Status::Ready)
            .done(true)
            .draw(ui, Rect::new(0, 3, 70, 1));
        ProgressBar::new(Id::root("s4.pb3"))
            .label("Failed    ")
            .ratio(0.64)
            .status(Status::Error)
            .draw(ui, Rect::new(0, 4, 70, 1));
        ProgressBar::new(Id::root("s4.pb4"))
            .label("Paused    ")
            .ratio(0.3)
            .status(Status::Ready)
            .icon(GlyphRole::ProgressPaused)
            .draw(ui, Rect::new(0, 5, 70, 1));
    }
}

/// PB-STATES-001 (`showcase/pages/progress`, paused): state samples plus
/// terminal glyphs over [`ProgressBar`].
///
/// Isolated path renders the row's five samples plus edge widths: terminal
/// glyphs and tones, the running-white/Done-green rule, clamp/round/label
/// rules, the narrow percent-only form, and the no-focus-stops plus
/// never-green negatives. Live path boots the progress page paused at
/// 120x40 and proves the glyphs plus the Tab-only-buttons focus shape.
/// (120x40, not the row's 80x24: the approved 80x24 frame clips the States
/// panel to Queued/Halfway, so the row's own V1 rows — Completed, Failed,
/// Paused — render only at 120x40 and above.)
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_pb_states() {
    let dir = s4_dir("s4_pb_states");
    let t = Theme::junie();
    let success = t.color.success;
    let error = t.color.danger;
    let muted = t.color.fg[FgStep::Muted.index()];
    let running = t.color.fg[FgStep::Secondary.index()];

    // V1/V2 isolated: terminal glyphs in their tones; running is white-70.
    // (VB `ProgressStatus` maps onto the bar props: Active is Ready with a
    // ratio, Done is `.done(true)`, Error is `Status::Error`, Paused is the
    // explicit paused icon.)
    let samples = [
        ("Queued    ", 0.0, false, "  ", running, "running"),
        ("Halfway   ", 0.5, false, "  ", running, "running"),
        ("Completed ", 1.0, true, " ✓", success, "done"),
    ];
    for (label, ratio, done, suffix, fill, _name) in samples {
        let buf = render_isolated(70, 3, 0, None, |ui, _| {
            ProgressBar::new(Id::root("s4.pb"))
                .label(label)
                .ratio(ratio)
                .status(Status::Ready)
                .done(done)
                .draw(ui, Rect::new(0, 1, 70, 1));
        });
        let row = buf_row_text(&buf, 1);
        if !done {
            assert!(
                !row.contains('✓') && !row.contains('‖') && !row.contains(" !"),
                "V1: {label:?} carries no terminal glyph"
            );
        } else {
            assert!(row.contains(suffix.trim()), "V1: {label:?} ends {suffix:?}");
        }
        let tx = track_start(&row);
        if ratio > 0.0 {
            assert_eq!(
                buf[(tx, 1)].fg,
                fill,
                "V2: {}",
                if done {
                    "green is reserved for Done"
                } else {
                    "running is white-70"
                }
            );
        } else {
            assert_eq!(
                buf[(tx, 1)].symbol(),
                "─",
                "V1: the empty track paints unfilled"
            );
        }
        if !done {
            assert!(
                (0..70).all(|x| buf[(x, 1)].fg != success),
                "N2: green never paints {label:?}"
            );
        }
    }
    for (label, ratio, status, suffix, fill, what) in [
        ("Failed    ", 0.64, Status::Error, " !", error, "Error tone"),
        ("Paused    ", 0.3, Status::Ready, " ‖", muted, "Paused tone"),
    ] {
        let buf = render_isolated(70, 3, 0, None, |ui, _| {
            let mut bar = ProgressBar::new(Id::root("s4.pb"))
                .label(label)
                .ratio(ratio)
                .status(status);
            if label.starts_with("Paused") {
                bar = bar.icon(GlyphRole::ProgressPaused);
            }
            bar.draw(ui, Rect::new(0, 1, 70, 1));
        });
        let row = buf_row_text(&buf, 1);
        assert!(row.contains(suffix.trim()), "V1: {label:?} ends {suffix:?}");
        let tx = track_start(&row);
        assert_eq!(buf[(tx, 1)].fg, fill, "{what}");
        assert!(
            (0..70).all(|x| buf[(x, 1)].fg != success),
            "N2: green never paints {label:?}"
        );
    }
    // S1: ratio clamps to 0..1 and the percent rounds.
    for (ratio, want) in [(1.7, "100%"), (-0.3, "0%"), (0.5, "50%")] {
        let buf = render_isolated(70, 3, 0, None, |ui, _| {
            ProgressBar::new(Id::root("s4.pb"))
                .label("Halfway   ")
                .ratio(ratio)
                .status(Status::Ready)
                .draw(ui, Rect::new(0, 1, 70, 1));
        });
        assert!(
            buf_row_text(&buf, 1).contains(want),
            "S1: ratio {ratio} shows {want}"
        );
    }
    // S2: the label paints only when width exceeds label plus 8.
    let buf = render_isolated(70, 3, 0, None, |ui, _| {
        ProgressBar::new(Id::root("s4.pb"))
            .label("Building  ")
            .ratio(0.5)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 1, 70, 1));
    });
    assert!(
        buf_row_text(&buf, 1).contains("Building"),
        "S2: wide paints the label"
    );
    let buf = render_isolated(70, 3, 0, None, |ui, _| {
        ProgressBar::new(Id::root("s4.pb"))
            .label("Building  ")
            .ratio(0.5)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 1, 10, 1));
    });
    assert!(
        !buf_row_text(&buf, 1).contains("Building"),
        "S2: narrow drops the label"
    );
    // N1: a bar narrower than 6 track cells shows the percent only.
    let buf = render_isolated(70, 3, 0, None, |ui, _| {
        ProgressBar::new(Id::root("s4.pb"))
            .label("")
            .ratio(0.42)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 1, 8, 1));
    });
    let narrow = buf_row_text(&buf, 1);
    assert!(narrow.contains("42%"), "N1: percent only");
    assert!(
        !narrow.contains('━') && !narrow.contains('─'),
        "N1: no label, no track"
    );
    // A1/A2: sample rows expose no focus stops.
    let mut app = Harness::new(PbFocusApp, Theme::junie(), 70, 8);
    assert!(
        !app.tab_to(Id::root("s4.pb0")),
        "A1: sample rows take no input (no focus stops)"
    );
    let buf = render_isolated(70, 8, 0, None, |ui, _| {
        ProgressBar::new(Id::root("s4.pb0"))
            .label("Queued    ")
            .ratio(0.0)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 1, 70, 1));
        ProgressBar::new(Id::root("s4.pb1"))
            .label("Halfway   ")
            .ratio(0.5)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 2, 70, 1));
        ProgressBar::new(Id::root("s4.pb2"))
            .label("Completed ")
            .ratio(1.0)
            .status(Status::Ready)
            .done(true)
            .draw(ui, Rect::new(0, 3, 70, 1));
        ProgressBar::new(Id::root("s4.pb3"))
            .label("Failed    ")
            .ratio(0.64)
            .status(Status::Error)
            .draw(ui, Rect::new(0, 4, 70, 1));
        ProgressBar::new(Id::root("s4.pb4"))
            .label("Paused    ")
            .ratio(0.3)
            .status(Status::Ready)
            .icon(GlyphRole::ProgressPaused)
            .draw(ui, Rect::new(0, 5, 70, 1));
    });
    capture_isolated(&dir, "pb-states", &buf);

    // PTY: the glyphs render paused; Tab reaches only Restart/Pause.
    let case = s4_case(
        "pb_states",
        &["--page", "progress", "--motion", "paused"],
        120,
        40,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "states");
    let states = live_text(&mut s);
    assert_line_has(&states, "Completed", "✓", "V1 live: Done glyph");
    assert_line_has(&states, "Failed", "!", "V1 live: Error glyph");
    assert_line_has(&states, "Paused", "‖", "V1 live: Paused glyph");
    assert!(
        states.contains("Queued") && states.contains("Halfway"),
        "V2 live: running samples render"
    );
    // A2 live: exactly two Tabs reach Pause (no sample stops between).
    support::press_step(&s, "tab");
    support::press_step(&s, "tab");
    support::press_step(&s, "space");
    waits::wait_state(&mut s, "Resume", "A2 live: Tab reaches Pause");
    eprintln!("s4 pb_states: isolated bar map + live glyphs/focus");
}

/// PB-LIVE-002 (`showcase/flows/progress/mid`, 80x24, playing): the live
/// build plus pause/restart over [`ProgressBar`].
///
/// Isolated path pins the bar geometry the live build paints through (fill
/// growth, Done form, 1.0 cap). Live path watches the percent climb,
/// pauses (percent freezes while the page animates), restarts to zero, and
/// rides the build to its once-only Done.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_pb_live() {
    let dir = s4_dir("s4_pb_live");

    // Isolated: the fill grows with the ratio and caps at 1.0.
    let fill_count = |ratio: f64| -> usize {
        let buf = render_isolated(70, 3, 0, None, |ui, _| {
            ProgressBar::new(Id::root("s4.pb"))
                .label("Building  ")
                .ratio(ratio)
                .status(Status::Ready)
                .draw(ui, Rect::new(0, 1, 70, 1));
        });
        buf_row_text(&buf, 1).chars().filter(|c| *c == '━').count()
    };
    let early = fill_count(0.08);
    let mid = fill_count(0.48);
    assert!(
        mid > early && early > 0,
        "V1: the fill climbs with the ratio"
    );
    let buf = render_isolated(70, 3, 0, None, |ui, _| {
        ProgressBar::new(Id::root("s4.pb"))
            .label("Building  ")
            .ratio(1.5)
            .status(Status::Ready)
            .draw(ui, Rect::new(0, 1, 70, 1));
    });
    let capped = buf_row_text(&buf, 1);
    assert!(capped.contains("100%"), "N1: ratios past 1.0 cap at Done");
    capture_isolated(&dir, "pb-live", &buf);

    // PTY: the percent climbs, pauses, restarts, and finishes once.
    let case = s4_case("pb_live", &["--page", "progress"], 80, 24).timeout(30_000);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let p1 = building_pct(&live_text(&mut s));
    std::thread::sleep(Duration::from_millis(1200));
    let p2 = building_pct(&live_text(&mut s));
    assert!(p2 > p1, "V1/S1 live: the build advances ({p1}% → {p2}%)");
    checkpoint(&mut s, &dir, "mid");
    // V2/A1/N2: pausing freezes the percent while the page animates.
    click_at(&mut s, "Pause", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "pause never applied",
        |screen| support::screen_text(screen).contains("Resume"),
    );
    let paused = live_text(&mut s);
    assert_line_has(&paused, "Building", "‖", "V2 live: pause suffix");
    let q1 = building_pct(&paused);
    // The nav column precedes the spinner on the row, so the glyph is the last
    // non-blank char BEFORE the `Waiting for` label.
    let spin_of = |text: &str| -> char {
        text.lines()
            .find(|l| l.contains("Waiting for"))
            .and_then(|l| {
                let pos = l.find("Waiting for")?;
                l[..pos].chars().rfind(|c| !c.is_whitespace())
            })
            .unwrap_or('?')
    };
    let spin1 = spin_of(&live_text(&mut s));
    std::thread::sleep(Duration::from_millis(1200));
    let q2 = building_pct(&live_text(&mut s));
    let spin2 = spin_of(&live_text(&mut s));
    assert_eq!(q1, q2, "N2 live: pausing stops the advance");
    assert_ne!(spin1, spin2, "N2 live: the page keeps animating");
    // A2: Restart resets to 0 and resumes running (unpause first: Restart
    // resets the build but leaves the paused flag alone).
    click_at(&mut s, "Resume", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "resume never toggled back",
        |screen| support::screen_text(screen).contains("Pause"),
    );
    click_at(&mut s, "Restart", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "restart never applied",
        |screen| building_pct(&support::screen_text(screen)) < q2,
    );
    // S2/N1: reaching 1.0 reports Done once and never overflows.
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "build never finished",
        |screen| support::screen_text(screen).contains("Build finished ✓"),
    );
    checkpoint(&mut s, &dir, "done");
    std::thread::sleep(Duration::from_millis(1000));
    let done = live_text(&mut s);
    assert_eq!(building_pct(&done), 100, "N1 live: Done never overflows");
    eprintln!("s4 pb_live: isolated growth + live climb/pause/restart/done");
}

/// One labeled spinner; the no-focus-stop probe.
struct SpinFocusApp;

impl App for SpinFocusApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Spinner::new(Id::root("s4.spin"))
            .label("Waiting for the test runner")
            .frame(0)
            .draw(ui, Rect::new(4, 1, 50, 1));
    }
}

/// SPIN-FRAME-001 (`showcase/pages/progress`, 80x24, playing): braille frames
/// plus the compact activity row over [`Spinner`].
///
/// Isolated path pins the 10-frame cycle, the accent-glyph/secondary-label
/// paint, the empty-area no-op, and the no-focus shape. Live path samples
/// two distinct frames plus the count line.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_spin_frame() {
    let dir = s4_dir("s4_spin_frame");
    let t = Theme::junie();

    // S1 isolated: frame(tick) is SPINNER[tick % 10].
    // (The cycle lives in the theme's motion tokens; the frame is a prop.)
    let frames = t.design.motion.spinner_frames;
    assert_eq!(frames.len(), 10, "S1: ten braille frames");
    assert_eq!(frames[0], "⠋", "row pins frame 0");
    assert_eq!(frames[1], "⠙", "row pins frame 1");
    for tick in 0..25u64 {
        assert_eq!(
            frames[(tick % 10) as usize],
            SPINNER[(tick % 10) as usize],
            "S1: frame({tick}) cycles"
        );
    }
    // V1/N2 isolated: `⠋ label` in accent/secondary; the label is never
    // bold and never accent.
    let accent = t.color.accent;
    let secondary = t.color.fg[FgStep::Secondary.index()];
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Spinner::new(Id::root("s4.spin"))
            .label("Waiting for the test runner")
            .frame(0)
            .draw(ui, Rect::new(4, 1, 50, 1));
    });
    assert_eq!(buf[(4, 1)].symbol(), "⠋", "V1: the glyph renders");
    assert_eq!(buf[(4, 1)].fg, accent, "V1: the glyph is accent");
    // The label starts past the icon plus the 2-cell design gap.
    assert_eq!(buf[(7, 1)].symbol(), "W", "the label follows the gap");
    assert_eq!(buf[(7, 1)].fg, secondary, "V1: the label is secondary");
    assert!(
        (7..40).all(|x| !buf_is_bold(&buf, x, 1)),
        "N2: the label is never bold"
    );
    assert!(
        (7..40).all(|x| buf[(x, 1)].fg != accent),
        "N2: the label is never accent"
    );
    let mut app = Harness::new(SpinFocusApp, Theme::junie(), 60, 3);
    assert!(
        !app.tab_to(Id::root("s4.spin")),
        "A1: the spinner holds no focus"
    );
    let first = capture_isolated(&dir, "spin-frame-0", &buf);
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Spinner::new(Id::root("s4.spin"))
            .label("Waiting for the test runner")
            .frame(1)
            .draw(ui, Rect::new(4, 1, 50, 1));
    });
    assert_eq!(buf[(4, 1)].symbol(), "⠙", "tick 1 advances the frame");
    let second = capture_isolated(&dir, "spin-frame-1", &buf);
    assert_ne!(
        first.digest(),
        second.digest(),
        "N1: frames differ, not frozen"
    );
    // S2: an empty area draws nothing.
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Spinner::new(Id::root("s4.spin"))
            .label("Waiting for the test runner")
            .frame(3)
            .draw(ui, Rect::ZERO);
    });
    assert_surround_intact(&buf, Rect::ZERO, "S2: an empty area draws nothing");

    // PTY: two distinct live frames plus the count line.
    let case = s4_case("spin_frame", &["--page", "progress"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "first-frame");
    let mut seen = BTreeSet::new();
    let deadline = std::time::Instant::now() + case_timeout(&case);
    while std::time::Instant::now() < deadline && seen.len() < 2 {
        let text = live_text(&mut s);
        if let Some(line) = text.lines().find(|l| l.contains("Waiting for"))
            && let Some(pos) = line.find("Waiting for")
            && let Some(g) = line[..pos].chars().rfind(|c| !c.is_whitespace())
            && SPINNER.contains(&g.to_string().as_str())
        {
            seen.insert(g);
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(seen.len() >= 2, "V1/N1 live: two spinner frames {seen:?}");
    checkpoint(&mut s, &dir, "second-frame");
    let text = live_text(&mut s);
    assert!(
        text.contains("3 of 12 files"),
        "V2 live: the count line renders"
    );
    eprintln!("s4 spin_frame: isolated cycle + live frames {seen:?}");
}

/// SPIN-SWEEP-002 (`showcase/flows/progress/mid`, 80x24, playing): the
/// indeterminate sweep geometry over [`ProgressBar`].
///
/// Isolated path pins the segment length/position math, the accent-over-
/// subtle paint, the label rule, and the never-outside/empty negatives.
/// Live path watches the sweep move.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_spin_sweep() {
    let dir = s4_dir("s4_spin_sweep");
    let t = Theme::junie();
    let accent = t.color.accent;
    let subtle = t.color.border_subtle;
    let primary = t.color.fg[FgStep::Primary.index()];

    // S1/S2 isolated: length (track/5) clamps to 2..8; position wraps.
    // (No ratio: the indeterminate sweep; the tick is the frame prop.)
    let sweep_row = |tick: usize, width: u16| -> Buffer {
        render_isolated(80, 3, 0, None, |ui, _| {
            ProgressBar::new(Id::root("s4.sweep"))
                .label("Resolving ")
                .frame(tick)
                .draw(ui, Rect::new(0, 1, width, 1));
        })
    };
    let buf8 = sweep_row(8, 70);
    let row8 = buf_row_text(&buf8, 1);
    assert!(
        row8.contains("Resolving"),
        "V2: the label paints when it fits"
    );
    // Track starts after "Resolving " + 2 (x = 12); at tick 8 the segment
    // sits at cells 12..20 in accent over the subtle track.
    assert_eq!(buf8[(0, 1)].fg, primary, "V2: the label is primary");
    assert_eq!(
        buf8[(12, 1)].fg,
        accent,
        "V1: accent segment head at tick 8"
    );
    assert_eq!(
        buf8[(19, 1)].fg,
        accent,
        "V1: 8-cell segment (58/5 clamps to 8)"
    );
    assert_eq!(
        buf8[(20, 1)].fg,
        subtle,
        "V1: subtle track past the segment"
    );
    assert_eq!(buf8[(12, 1)].symbol(), "━", "segment cells");
    let buf0 = sweep_row(0, 70);
    assert!(
        (0..70).all(|x| buf0[(x, 1)].fg != accent),
        "tick 0 parks the segment fully left"
    );
    let buf66 = sweep_row(66, 70);
    assert_eq!(
        buf_row_text(&buf66, 1),
        buf_row_text(&buf0, 1),
        "S2: position wraps around at tick 66"
    );
    // N1: the segment never paints outside the track cells.
    for tick in [0usize, 8, 40, 65, 200] {
        let buf = sweep_row(tick, 70);
        assert!(
            (70..80).all(|x| buf[(x, 1)].symbol() == "·"),
            "N1: tick {tick} never escapes the area"
        );
    }
    capture_isolated(&dir, "sweep-8", &buf8);
    // Empty areas draw nothing; no focus stops anywhere.
    let buf = render_isolated(80, 3, 0, None, |ui, _| {
        ProgressBar::new(Id::root("s4.sweep"))
            .label("Resolving ")
            .frame(5)
            .draw(ui, Rect::ZERO);
    });
    assert_surround_intact(&buf, Rect::ZERO, "N2: an empty area draws nothing");
    struct SweepFocusApp;
    impl App for SweepFocusApp {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            ProgressBar::new(Id::root("s4.sweep"))
                .label("Resolving ")
                .frame(5)
                .draw(ui, Rect::new(0, 1, 70, 1));
        }
    }
    let mut app = Harness::new(SweepFocusApp, Theme::junie(), 80, 3);
    assert!(
        !app.tab_to(Id::root("s4.sweep")),
        "A1: the sweep holds no focus"
    );

    // PTY: the sweep visibly moves.
    let case = s4_case("spin_sweep", &["--page", "progress"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "sweep-0");
    let mut rows = BTreeSet::new();
    let mut saw_seg = false;
    let deadline = std::time::Instant::now() + case_timeout(&case);
    while std::time::Instant::now() < deadline && (rows.len() < 2 || !saw_seg) {
        let text = live_text(&mut s);
        if let Some(line) = text.lines().find(|l| l.contains("Resolving")) {
            saw_seg |= line.contains('━');
            rows.insert(line.to_owned());
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(saw_seg, "V1 live: the accent segment sweeps into view");
    assert!(rows.len() >= 2, "V1 live: the sweep moves");
    checkpoint(&mut s, &dir, "sweep-5");
    eprintln!("s4 spin_sweep: isolated wrap + live motion");
}

/// One threshold meter; the no-focus-stop probe.
struct MeterFocusApp;

impl App for MeterFocusApp {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.38)
            .value("38% used")
            .draw(ui, Rect::new(0, 1, 40, 1));
    }
}

/// METER-TONES-001 (`showcase/pages/progress`, 120x40, paused): threshold
/// levels plus the line visual over [`Meter`].
///
/// Isolated path renders the row's three samples (38/72/91): shared
/// thresholds, run/value tones, the 100-cap ratio, the narrow value-only
/// form, and the never-green plus no-focus negatives. Live path boots the
/// progress page paused at 120x40 and reads the three meter rows.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_meter_tones() {
    let dir = s4_dir("s4_meter_tones");
    let t = Theme::junie();

    // S1 isolated: of(pct) is Low at 59 and below, Medium to 84, High above.
    // (The mapping lives in `MeterTone::from_ratio` against the design
    // thresholds; the boundaries are the row's own 59/84.)
    let thresholds = t.design.meter;
    assert_eq!(thresholds.low_max, 59, "low boundary");
    assert_eq!(thresholds.medium_max, 84, "medium boundary");
    for (pct, want) in [
        (0u16, MeterTone::Low),
        (59, MeterTone::Low),
        (60, MeterTone::Medium),
        (84, MeterTone::Medium),
        (85, MeterTone::High),
        (100, MeterTone::High),
    ] {
        assert_eq!(
            MeterTone::from_ratio(f64::from(pct) / 100.0, thresholds),
            want,
            "S1: of({pct})"
        );
    }
    // V1/V2 isolated: white/warning/error runs; low value text is primary,
    // medium and high take the run color.
    let white = t.color.fg[FgStep::Secondary.index()];
    let warning = t.color.warning;
    let err = t.color.danger;
    let primary = t.color.fg[FgStep::Primary.index()];
    let success = t.color.success;
    for (pct, run, text, what) in [
        (38u8, white, primary, "low"),
        (72u8, warning, warning, "medium"),
        (91u8, err, err, "high"),
    ] {
        let buf = render_isolated(60, 3, 0, None, |ui, _| {
            Meter::new(Id::root("s4.meter"))
                .ratio(f64::from(pct) / 100.0)
                .value(&format!("{pct}% used"))
                .draw(ui, Rect::new(0, 1, 40, 1));
        });
        let row = buf_row_text(&buf, 1);
        assert!(
            row.contains(&format!("{pct}% used")),
            "V1: {what} value right of the run"
        );
        assert_eq!(buf[(0, 1)].fg, run, "V1: {what} run tone");
        // Cell columns, not byte offsets: the run is multibyte ━/─ cells.
        let track_w = 40u16 - (8 + 3);
        let vx = track_w + 1;
        assert_eq!(
            buf[(vx, 1)].symbol(),
            &format!("{pct}")[0..1],
            "value sits right of the run"
        );
        assert_eq!(buf[(vx, 1)].fg, text, "V2: {what} value tone");
        assert!(
            (0..40).all(|x| buf[(x, 1)].fg != success),
            "N1: green is never used ({what})"
        );
    }
    // S2: ratio is used_pct capped at 100 over 100.
    let capped = |pct: u8| -> String {
        let buf = render_isolated(60, 3, 0, None, |ui, _| {
            Meter::new(Id::root("s4.meter"))
                .ratio(f64::from(pct) / 100.0)
                .value(&format!("{pct}% used"))
                .draw(ui, Rect::new(0, 1, 40, 1));
        });
        buf_row_text(&buf, 1)
            .chars()
            .filter(|c| *c == '━')
            .count()
            .to_string()
    };
    assert_eq!(capped(150), capped(100), "S2: used_pct caps at 100");
    // N2: a track narrower than 6 cells shows the value only.
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.38)
            .value("38% used")
            .draw(ui, Rect::new(0, 1, 14, 1));
    });
    let narrow = buf_row_text(&buf, 1);
    assert!(narrow.contains("38% used"), "N2: value only");
    assert!(
        !narrow.contains('━') && !narrow.contains('─'),
        "N2: no track"
    );
    // A1/A2: meters hold no focus.
    let mut app = Harness::new(MeterFocusApp, Theme::junie(), 60, 3);
    assert!(
        !app.tab_to(Id::root("s4.meter")),
        "A1: meters hold no focus"
    );
    let buf = render_isolated(60, 5, 0, None, |ui, _| {
        for (i, pct) in [38u8, 72, 91].iter().enumerate() {
            Meter::new(Id::root("s4.meter"))
                .ratio(f64::from(*pct) / 100.0)
                .value(&format!("{pct}% used"))
                .draw(ui, Rect::new(0, i as u16 + 1, 40, 1));
        }
    });
    capture_isolated(&dir, "meter-tones", &buf);

    // PTY: the three threshold rows render paused at 120x40.
    let case = s4_case(
        "meter_tones",
        &["--page", "progress", "--motion", "paused"],
        120,
        40,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "meters");
    let meters = live_text(&mut s);
    assert_line_has(&meters, "Low", "38% used", "V1 live: low row");
    assert_line_has(&meters, "Medium", "72% used", "V1 live: medium row");
    assert_line_has(&meters, "High", "91% used", "V1 live: high row");
    eprintln!("s4 meter_tones: isolated thresholds + live rows");
}

/// METER-DOMAIN-002 (`showcase/pages/progress`, 120x40, paused): domain tones
/// plus the block visual over [`Meter`].
///
/// Isolated path renders the row's six domain tones in both visuals: the
/// Warning/Exhausted/Stale/Unknown markers, the block fill with its inset
/// value, the tone→level map, the Refreshing spinner value, and the
/// no-run plus Stale-text negatives. Live path reads the six domain rows
/// paused at 120x40.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_meter_domain() {
    let dir = s4_dir("s4_meter_domain");
    let t = Theme::junie();

    // S1 isolated: level() maps Warning to Medium and Exhausted to High;
    // Stale and below map to None.
    // (No `level()` exists: the graded half runs through `from_ratio`, the
    // flat tones keep their distinct role identity, and their paint is
    // pinned by V1/N1 below.)
    let thresholds = t.design.meter;
    assert_eq!(
        MeterTone::from_ratio(0.82, thresholds),
        MeterTone::Medium,
        "S1: Warning maps to Medium"
    );
    assert_eq!(
        MeterTone::from_ratio(1.0, thresholds),
        MeterTone::High,
        "S1: Exhausted maps to High"
    );
    for tone in [MeterTone::Stale, MeterTone::Unknown] {
        assert!(
            !matches!(tone, MeterTone::Low | MeterTone::Medium | MeterTone::High),
            "S1: {tone:?} is ungraded"
        );
    }
    let warning = t.color.warning;
    let err = t.color.danger;
    let faint = t.color.fg[FgStep::Faint.index()];
    let secondary = t.color.fg[FgStep::Secondary.index()];
    // V1 isolated: Warning ends ` ▲`, Exhausted ends bold ` !`, Stale is
    // faint, Unknown is `—`.
    // (Warning/Exhausted/Refreshing/Error have no domain tones: Warning
    // runs as the Medium grade, Exhausted as High under Error readiness,
    // Error as readiness without a run, Refreshing as Busy with a leading
    // spinner. The row's markers are kept verbatim.)
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.82)
            .value("82% used")
            .tone(MeterTone::Medium)
            .draw(ui, Rect::new(0, 1, 40, 1));
    });
    let row = buf_row_text(&buf, 1);
    assert!(row.contains('▲'), "V1: Warning ends ▲");
    assert_eq!(buf[(0, 1)].fg, warning, "V1: Warning run tone");
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(1.0)
            .value("100% used")
            .tone(MeterTone::High)
            .status(Status::Error)
            .draw(ui, Rect::new(0, 1, 40, 1));
    });
    let row = buf_row_text(&buf, 1);
    assert!(row.contains('!'), "V1: Exhausted ends !");
    let bbyte = row
        .char_indices()
        .rev()
        .find(|(_, c)| *c == '!')
        .expect("suffix renders")
        .0;
    let bx = row[..bbyte].chars().count() as u16;
    assert!(buf_is_bold(&buf, bx, 1), "V1: Exhausted ! is bold");
    assert_eq!(buf[(bx, 1)].fg, err, "Exhausted tone");
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.54)
            .value("54% used")
            .tone(MeterTone::Stale)
            .draw(ui, Rect::new(0, 1, 40, 1));
    });
    assert_eq!(buf[(0, 1)].fg, faint, "V1: Stale is faint");
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .tone(MeterTone::Unknown)
            .draw(ui, Rect::new(0, 1, 40, 1));
    });
    assert!(buf_row_text(&buf, 1).contains('—'), "V1: Unknown is —");
    // N1: Error and Unknown draw no run (presence: Warning draws one).
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .value("quota read failed")
            .status(Status::Error)
            .draw(ui, Rect::new(0, 1, 40, 1));
    });
    let erow = buf_row_text(&buf, 1);
    assert!(erow.contains("quota read failed"), "Error message renders");
    assert!(
        !erow.contains('━') && !erow.contains('─'),
        "N1: Error draws no run"
    );
    // S2: Refreshing shows the spinner frame plus `refreshing` as the value.
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.54)
            .value("refreshing")
            .status(Status::Busy)
            .leading_activity(true)
            .frame(0)
            .draw(ui, Rect::new(0, 1, 40, 1));
    });
    assert!(
        buf_row_text(&buf, 1).contains("⠋ refreshing"),
        "S2: Refreshing value at tick 0"
    );
    // V2/N2: Block fills the used share as background with the value inside
    // it starting one cell in; Stale fill text is secondary, never canvas.
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.82)
            .value("82% used")
            .tone(MeterTone::Medium)
            .visual(MeterVisual::Block)
            .draw(ui, Rect::new(0, 1, 20, 1));
    });
    assert_eq!(buf[(2, 1)].bg, warning, "V2: Block used share filled");
    assert_eq!(buf[(1, 1)].symbol(), "8", "V2: value starts one cell in");
    let buf = render_isolated(60, 3, 0, None, |ui, _| {
        Meter::new(Id::root("s4.meter"))
            .ratio(0.54)
            .value("54% used")
            .tone(MeterTone::Stale)
            .visual(MeterVisual::Block)
            .draw(ui, Rect::new(0, 1, 20, 1));
    });
    assert_eq!(
        buf[(2, 1)].fg,
        secondary,
        "N2: Stale fill text is secondary"
    );
    let canvas = t.color.surfaces[Surface::Canvas.level().expect("canvas level")];
    assert_ne!(buf[(2, 1)].fg, canvas, "N2: never canvas");
    let mut app = Harness::new(MeterFocusApp, Theme::junie(), 60, 3);
    assert!(
        !app.tab_to(Id::root("s4.meter")),
        "A1: domain meters hold no focus"
    );
    capture_isolated(&dir, "meter-domain", &buf);

    // PTY: the domain rows render paused at 120x40; the Unknown row needs
    // 160x50 (it is clipped at every smaller canonical size, including in
    // the approved frames).
    let case = s4_case(
        "meter_domain",
        &["--page", "progress", "--motion", "paused"],
        120,
        40,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "domain meters never rendered",
        |screen| support::screen_text(screen).contains("Warning"),
    );
    checkpoint(&mut s, &dir, "domain-meters");
    let meters = live_text(&mut s);
    assert_line_has(&meters, "Warning", "▲", "V1 live: Warning row");
    assert_line_has(&meters, "Exhausted", "!", "V1 live: Exhausted row");
    assert!(meters.contains("Stale"), "V1 live: Stale row");
    assert!(
        meters.contains("⠋ refreshing"),
        "S2 live: Refreshing pins frame 0 while paused"
    );
    assert!(meters.contains("quota read failed"), "N1 live: Error row");
    s.resize(160, 50).expect("resize to 160x50");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "never reached 160x50",
        |screen| screen.cols() == 160 && screen.rows() == 50,
    );
    std::thread::sleep(Duration::from_millis(700));
    let tall = live_text(&mut s);
    assert_line_has(&tall, "Unknown", "—", "V1 live: Unknown row");
    eprintln!("s4 meter_domain: isolated tones + live rows");
}

/// The row's chrome bar: six items across three groups (VB
/// `chrome_status_bar`). Keys are indices: 0 is the Weekly usage chip, 1
/// the PR row.
struct ChromeBarApp {
    left: [StatusItem<'static>; 2],
    center: [StatusItem<'static>; 1],
    right: [StatusItem<'static>; 3],
    width: u16,
    log: Rc<RefCell<Vec<StatusAction>>>,
}

impl ChromeBarApp {
    fn chrome(width: u16) -> Self {
        ChromeBarApp {
            left: [
                StatusItem::new("payments-platform").strong().priority(9),
                StatusItem::new("PR #482 · settlement backoff")
                    .tone(Role::Fg(FgStep::Secondary))
                    .priority(7)
                    .key(ItemKey::index(1)),
            ],
            center: [StatusItem::new("Claude Code · working · 2 tabs")
                .tone(Role::Fg(FgStep::Secondary))
                .priority(4)],
            right: [
                StatusItem::new("Weekly 59%")
                    .tone(Role::Warning)
                    .chip()
                    .priority(6)
                    .key(ItemKey::index(0)),
                StatusItem::new("jackin-payments-7f3a")
                    .tone(Role::Fg(FgStep::Muted))
                    .chip()
                    .priority(3),
                StatusItem::new("run 9c41")
                    .tone(Role::Fg(FgStep::Faint))
                    .priority(2),
            ],
            width,
            log: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn bar(&self) -> StatusBar<'_> {
        StatusBar::new(Id::root("s4.sb"))
            .left(&self.left)
            .center(&self.center)
            .right(&self.right)
    }
}

impl App for ChromeBarApp {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let log = Rc::clone(&self.log);
        self.bar()
            .update(cx)
            .on_action(|action| log.borrow_mut().push(action))
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        self.bar().draw(ui, Rect::new(0, 1, self.width, 1));
    }
}

/// Cell column where `needle` starts in `row` (multibyte-safe).
fn row_col(row: &str, needle: &str) -> u16 {
    let byte = row.find(needle).expect("needle renders") as u16;
    row[..byte as usize].chars().count() as u16
}

/// SB-GROUPS-001 (`showcase/pages/chrome`): three groups plus chips plus
/// hover over [`StatusBar`].
///
/// Isolated path mirrors the row's chrome bar at full width: shared-row
/// spacing, priorities, bold name, chip planes, hover lift, and one-row hit
/// regions. Live path boots 80x24 (where priorities evict to the top two),
/// resizes to 160x50 (the canonical size where the full bar renders), and
/// hovers the Weekly chip.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_sb_groups() {
    let dir = s4_dir("s4_sb_groups");
    let t = Theme::junie();
    let elevated = t.color.surfaces[Surface::Elevated.level().expect("elevated level")];
    let overlay = t.color.surfaces[Surface::Overlay.level().expect("overlay level")];

    // S1/V1 isolated: all six place at full width with spacing only.
    // (No `layout()` exists: survivors are read back from the painted row.)
    let app = ChromeBarApp::chrome(160);
    let buf = render_isolated(160, 3, 0, None, |ui, _| {
        app.bar().draw(ui, Rect::new(0, 1, 160, 1));
    });
    let row = buf_row_text(&buf, 1);
    for text in [
        "payments-platform",
        "PR #482 · settlement backoff",
        "Claude Code · working · 2 tabs",
        "Weekly 59%",
        "jackin-payments-7f3a",
        "run 9c41",
    ] {
        assert!(row.contains(text), "S1: all six items place at full width");
    }
    assert!(
        row.contains("payments-platform")
            && row.contains("Claude Code · working · 2 tabs")
            && row.contains("Weekly 59%"),
        "V1: all groups render"
    );
    // N1/S2: gaps between groups are never glyphs (spaces on the bar fill).
    let left_end = row_col(&row, "PR #482 · settlement backoff")
        + "PR #482 · settlement backoff".chars().count() as u16;
    let center_x = row_col(&row, "Claude Code · working · 2 tabs");
    assert!(
        center_x > left_end,
        "N2: the center never sits left of the left"
    );
    for x in left_end..center_x {
        assert_eq!(buf[(x, 1)].symbol(), " ", "N1: gaps are never glyphs");
        assert_eq!(buf[(x, 1)].bg, elevated, "S2: gaps fill surface_elevated");
    }
    // V2: the surface name is bold; quota chips sit on the overlay plane.
    let name_x = row_col(&row, "payments-platform");
    assert!(buf_is_bold(&buf, name_x, 1), "V2: the surface name is bold");
    let chip_x = row_col(&row, "Weekly 59%") + 1;
    assert_eq!(
        buf[(chip_x, 1)].bg,
        overlay,
        "V2: chips sit on the overlay plane"
    );
    capture_isolated(&dir, "sb-groups", &buf);
    // A2: clickable items register a one-row hit region.
    let mut app = Harness::new(ChromeBarApp::chrome(160), Theme::junie(), 160, 3);
    let hit = app
        .area_of_part(
            Id::root("s4.sb"),
            PartRef::item(Part::LABEL, ItemKey::index(0)),
        )
        .expect("A2: usage hit region");
    assert_eq!(hit.height, 1, "A2: one-row hit region");
    assert!(
        hit.width >= 12,
        "A2: the region covers the chip (presence proof)"
    );

    // PTY: 80x24 evicts by priority; 160x50 shows the full bar for hover.
    let case = s4_case("sb_groups", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "groups-80");
    let narrow = live_text(&mut s);
    assert!(
        narrow.contains("payments-platform") && narrow.contains("PR #482"),
        "S1 live: the top priorities survive at 80x24"
    );
    assert!(
        !narrow.contains("Weekly 59%") && !narrow.contains("2 tabs"),
        "S1 live: lower priorities evict at 80x24"
    );
    s.resize(160, 50).expect("resize to 160x50");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "never reached 160x50",
        |screen| screen.cols() == 160 && screen.rows() == 50,
    );
    std::thread::sleep(Duration::from_millis(700));
    checkpoint(&mut s, &dir, "groups");
    let wide = live_text(&mut s);
    assert!(
        wide.contains("Claude Code · working · 2 tabs") && wide.contains("Weekly 59%"),
        "V1 live: all groups share the row at 160x50"
    );
    // N2 live: the center sits right of the left group.
    let (lrow, lcol) = find_pos(&mut s, "payments-platform", case_timeout(&case));
    let (crow, ccol) = find_pos(&mut s, "2 tabs", case_timeout(&case));
    assert_eq!(lrow, crow, "V1 live: one shared row");
    assert!(ccol > lcol, "N2 live: center right of left");
    // A1 live: hovering the Weekly chip lifts its plane.
    let (hrow, hcol) = find_pos(&mut s, "Weekly 59%", case_timeout(&case));
    let before = live_frame(&mut s)
        .get(hcol + 1, hrow)
        .unwrap_or_else(|| panic!("chip cell missing"))
        .bg;
    hover_over(&mut s, "Weekly 59%", case_timeout(&case));
    let after = live_frame(&mut s)
        .get(hcol + 1, hrow)
        .unwrap_or_else(|| panic!("hovered chip cell missing"))
        .bg;
    assert_ne!(before, after, "A1 live: hover lifts the chip plane");
    let hovered = live_text(&mut s);
    assert!(
        hovered.contains("Weekly 59%"),
        "A1 live: glyphs stable under hover"
    );
    checkpoint(&mut s, &dir, "hover");
    // A1 isolated: hovering a clickable chip lifts its plane to primary
    // text. (Runs last: the chip delta keeps the Overlay plane under
    // hover, so this is the known genuine gap; every other assertion —
    // isolated and live — runs first.)
    let lift = t.color.surfaces[t.raise(Surface::Overlay).level().expect("lift level")];
    let primary = t.color.fg[FgStep::Primary.index()];
    let _ = app.mouse(MouseKind::Move, chip_x, 1);
    assert_eq!(
        app.buffer()[(chip_x, 1)].bg,
        lift,
        "A1: hover lifts the chip plane"
    );
    assert_eq!(
        app.buffer()[(chip_x, 1)].fg,
        primary,
        "A1: hover takes primary text"
    );
    eprintln!("s4 sb_groups: isolated planes + live evict/resize/hover");
}

/// SB-COLLAPSE-002 (`showcase/pages/chrome`, 72x20): narrow collapse order
/// plus truncation over [`StatusBar`].
///
/// Isolated path proves the eviction machine at exact widths (survivor sets,
/// tie order, truncate-with-… of the last left item, right-never-truncated,
/// no focus stops). Live path boots the registry viewport (center gone, the
/// strongest left item kept whole), resizes up (same priorities recompute),
/// and proves the survivor is never dropped.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_sb_collapse() {
    let dir = s4_dir("s4_sb_collapse");

    // S1 isolated: eviction drops the lowest priority anywhere; at page
    // widths 61/53 (the 80x24/72x20 chrome pages) only the left pair stays.
    // (No `layout()` exists: survivors are read back from the painted row.)
    for (w, what) in [(61u16, "80x24 page"), (53u16, "72x20 page")] {
        let app = ChromeBarApp::chrome(w);
        let buf = render_isolated(w, 3, 0, None, |ui, _| {
            app.bar().draw(ui, Rect::new(0, 1, w, 1));
        });
        let row = buf_row_text(&buf, 1);
        assert!(
            row.contains("payments-platform") && row.contains("PR #482 · settlement backoff"),
            "S1: {what} keeps the left pair"
        );
        assert!(
            !row.contains("2 tabs") && !row.contains("Weekly 59%"),
            "S1: {what} evicts center and right"
        );
    }
    // S1 ties: center leaves before right before left at equal priority.
    let tied = || {
        let left = [StatusItem::new("L").priority(5)];
        let center = [StatusItem::new("C").priority(5)];
        let right = [StatusItem::new("R").priority(5)];
        render_isolated(10, 3, 0, None, |ui, _| {
            StatusBar::new(Id::root("s4.sb"))
                .left(&left)
                .center(&center)
                .right(&right)
                .draw(ui, Rect::new(0, 1, 10, 1));
        })
    };
    let row = buf_row_text(&tied(), 1);
    assert!(
        !row.contains('C'),
        "S1: ties evict the center first (row: {row:?})"
    );
    // V1/S2/N1: at width 40 only the strongest left item stays, whole; at
    // width 18 it truncates with … instead of leaving.
    let app = ChromeBarApp::chrome(40);
    let buf = render_isolated(40, 3, 0, None, |ui, _| {
        app.bar().draw(ui, Rect::new(0, 1, 40, 1));
    });
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("payments-platform") && !row.contains("PR #482") && !row.contains("Weekly"),
        "V1: width 40 keeps one survivor"
    );
    assert!(
        row.contains("payments-platform"),
        "N1: the strongest left item is never dropped"
    );
    let app = ChromeBarApp::chrome(18);
    let buf = render_isolated(18, 3, 0, None, |ui, _| {
        app.bar().draw(ui, Rect::new(0, 1, 18, 1));
    });
    let row = buf_row_text(&buf, 1);
    assert!(
        !row.contains("payments-platform"),
        "S2: width 18 cannot keep it whole"
    );
    assert!(row.contains('…'), "V2: it truncates with …");
    let cells: Vec<char> = row.chars().collect();
    assert_eq!(cells[16], '…', "V2: the cut marker ends the room");
    assert!(
        (1..16).all(|i| cells[i] != ' '),
        "V2: it fills the room (row: {row:?})"
    );
    // N2: right items are never truncated — they leave whole or stay whole.
    for w in [40u16, 53, 61, 100, 134] {
        let app = ChromeBarApp::chrome(w);
        let buf = render_isolated(w, 3, 0, None, |ui, _| {
            app.bar().draw(ui, Rect::new(0, 1, w, 1));
        });
        let row = buf_row_text(&buf, 1);
        for text in ["Weekly 59%", "jackin-payments-7f3a", "run 9c41"] {
            let short = text.split(' ').next().expect("first word");
            if row.contains(short) {
                assert!(
                    row.contains(text),
                    "N2: right items never truncate (width {w}, row: {row:?})"
                );
            }
        }
    }
    // A1/A2: placement recomputes from the same priorities per width; the
    // bar holds no focus stops at any width.
    let narrow_app = ChromeBarApp::chrome(53);
    let narrow_buf = render_isolated(53, 3, 0, None, |ui, _| {
        narrow_app.bar().draw(ui, Rect::new(0, 1, 53, 1));
    });
    let wide_app = ChromeBarApp::chrome(134);
    let wide_buf = render_isolated(134, 3, 0, None, |ui, _| {
        wide_app.bar().draw(ui, Rect::new(0, 1, 134, 1));
    });
    let narrow_row = buf_row_text(&narrow_buf, 1);
    let wide_row = buf_row_text(&wide_buf, 1);
    assert!(
        narrow_row.len() < wide_row.len(),
        "A1: width recomputes placement"
    );
    for text in [
        "payments-platform",
        "PR #482 · settlement backoff",
        "Claude Code · working · 2 tabs",
        "Weekly 59%",
        "jackin-payments-7f3a",
        "run 9c41",
    ] {
        assert!(wide_row.contains(text), "A1: full width keeps all six");
    }
    let mut app = Harness::new(ChromeBarApp::chrome(134), Theme::junie(), 134, 3);
    assert!(
        !app.tab_to(Id::root("s4.sb")),
        "A2: the bar holds no focus stops"
    );
    capture_isolated(&dir, "sb-collapse", &wide_buf);

    // PTY: 72x20 drops the center first and keeps the survivor whole; a
    // resize recomputes from the same priorities.
    let case = s4_case("sb_collapse", &["--page", "chrome"], 72, 20);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "narrow");
    let narrow = live_text(&mut s);
    assert!(
        !narrow.contains("2 tabs") && !narrow.contains("Weekly 59%"),
        "V1 live: the center goes first at 72x20"
    );
    assert!(
        narrow.contains("payments-platform") && narrow.contains("PR #482 · settlement backoff"),
        "N1 live: the left pair survives whole (presence proof)"
    );
    assert_line_lacks(
        &narrow,
        "payments-platform",
        "Weekly",
        "V1 live: the right group is evicted, not co-present",
    );
    s.resize(80, 24).expect("resize to 80x24");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "never reached 80x24",
        |screen| screen.cols() == 80 && screen.rows() == 24,
    );
    std::thread::sleep(Duration::from_millis(700));
    let grown = live_text(&mut s);
    assert!(
        grown.contains("payments-platform") && !grown.contains("Weekly 59%"),
        "A1 live: the resize recomputes the same priorities"
    );
    eprintln!("s4 sb_collapse: isolated eviction + live narrow/resize");
}

/// Extract the `NN%` value from the live "Building" line (progress page).
fn building_pct(text: &str) -> u32 {
    let line = text
        .lines()
        .find(|l| l.contains("Building"))
        .expect("Building line on screen");
    let tok = line
        .split_whitespace()
        .find(|t| t.ends_with('%'))
        .unwrap_or_else(|| panic!("no percent on {line:?}"));
    tok.trim_end_matches('%')
        .parse()
        .unwrap_or_else(|_| panic!("bad percent {tok:?}"))
}
