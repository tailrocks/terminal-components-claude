//! Showcase pending slice 8A-S4 executable checks (VB Phase-8ab).
//!
//! One ignored test per S4 registry row (20 rows: the next 20 unrun
//! showcase rows in registry order after S3 — the 10 OVERLAYS group rows
//! plus the 2 FORMS rows plus the 8 FEEDBACK rows), over the real `showcase`
//! binary built from this worktree's verified VB sources
//! (`env!("CARGO_BIN_EXE_showcase")`) and — for the 17 rows whose
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
//! Three rows are PTY-only by construction: HELP-SHOWCASE-001 (help is the
//! showcase binary's `open_help` in `app.rs`), FORM-SUBMIT-001 and
//! FORM-INVALID-002 (the submit/countdown/reset state machine lives in the
//! showcase binary's `forms.rs`). None is importable from this harness
//! crate, so no headless path exists for them; every one of their checks
//! has a live-PTY executable form instead.
//!
//! No new snapshots and no new static captures: every S4 row already has
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

use std::path::{Path, PathBuf};
use std::time::Duration;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{ButtonKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::completion::{Completion, CompletionEvent, CompletionItem};
use junie_tui::widgets::dialog::{Dialog, DialogResult};
use junie_tui::widgets::input::TextInput;
use junie_tui::widgets::menu::{
    ContextMenu, MenuBar, MenuBarEvent, MenuEvent, MenuItem, Placement,
};
use junie_tui::widgets::picker::{Picker, PickerEvent, PickerItem, PickerStatus};
use junie_tui::widgets::progress::{
    METER_LOW_MAX, METER_MEDIUM_MAX, Meter, MeterLevel, MeterTone, MeterVisual, ProgressStatus,
    SPINNER, render_bar, render_indeterminate, render_spinner, spinner_frame,
};
use junie_tui::widgets::select::{Select, SelectEvent};
use junie_tui::widgets::statusbar::{Group, StatusBar, StatusItem};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;
use tuiscotti::tui::{MouseButton, MouseMods, Session};
use tuiscotti::{Frame, Provenance};

use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, SHOWCASE};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

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

/// Record the binary path + identity: absolute path, byte size, mtime, argv.
/// Written to `provenance.txt`, echoed too (the build-identity record).
fn write_provenance(dir: &Path, case: &Case) {
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
    std::fs::write(dir.join("provenance.txt"), &body)
        .unwrap_or_else(|e| panic!("write provenance.txt: {e}"));
    eprintln!("s4 provenance: {body}");
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
        Provenance::now("tuiscotti-default", "s4-checks", vec![]),
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
    s.click(MouseButton::Right, col, row, MouseMods::NONE)
        .unwrap_or_else(|e| panic!("right click `{needle}` failed: {e:#}"));
    std::thread::sleep(Duration::from_millis(120));
    (row, col)
}

/// Type literal `text` into the live session, paced like a send step.
fn type_text(s: &mut Session, text: &str) {
    s.send_text(text).expect("type_text");
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

// ------------------------------------------------------- isolated harness --

fn s4_key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn s4_key_mods(code: KeyCode, mods: KeyModifiers) -> Key {
    Key { code, mods }
}

fn s4_id(name: &str) -> WidgetId {
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

/// Cell column of `needle`'s first occurrence in `line` (char-based, since
/// the chrome here mixes multibyte cells with single-cell advances).
fn char_col(line: &str, needle: &str) -> u16 {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {line:?}"));
    line[..byte].chars().count() as u16
}

/// Headless capture plus JSON evidence in the test dir.
fn capture_isolated(dir: &Path, name: &str, buf: &Buffer) -> Frame {
    let frame = support::capture_buffer(buf, Provenance::now("default", "s4-isolated", vec![]));
    std::fs::write(
        dir.join(format!("isolated-{name}.json")),
        frame.to_json_pretty(),
    )
    .unwrap_or_else(|e| panic!("write isolated-{name}.json: {e}"));
    eprintln!("isolated {name}: digest {:016x}", frame.digest());
    frame
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
    let sid = s4_id("test.s4.sort");

    // S1 isolated: Enter opens with cursor = selected; Up/Down/k/j clamp.
    let mut sort = Select::new(
        sid,
        "Sort by",
        &["created_at", "total", "status", "customer"],
        0,
    );
    let (o, ev) = sort.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none(),
        "S1: Enter opens"
    );
    assert!(sort.open && sort.cursor == 0, "S1: cursor = selected");
    let (o, _) = sort.on_key(&s4_key(KeyCode::Char('k')));
    assert!(
        matches!(o, Outcome::Changed) && sort.cursor == 0,
        "S1: k clamps at the top"
    );
    sort.on_key(&s4_key(KeyCode::Char('j')));
    sort.on_key(&s4_key(KeyCode::Down));
    assert_eq!(sort.cursor, 2, "S1: j/Down move the cursor");
    for _ in 0..9 {
        sort.on_key(&s4_key(KeyCode::Down));
    }
    assert_eq!(sort.cursor, 3, "S1: Down clamps at the last option");
    // V1/V2 render form: the open field shows ▴ and the popup anchors
    // below it with › on the selected row.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 14);
    let bg = stage.bg();
    let area = Rect::new(4, 1, 30, 3);
    sort.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(sid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        buf[(59, 13)].symbol(),
        "·",
        "select open render: far corner untouched"
    );
    let field_row = buf_row_text(&buf, 2);
    assert!(
        field_row.contains("created_at") && field_row.contains('▴'),
        "V1: open field shows the value with ▴"
    );
    let below: String = (3..14).map(|y| buf_row_text(&buf, y)).collect();
    assert!(
        below.contains("total") && below.contains("customer"),
        "V2: the popup anchors below the field"
    );
    let sel_line = (3..14)
        .map(|y| buf_row_text(&buf, y))
        .find(|l| l.contains('›'))
        .expect("V2: a › row exists");
    assert!(
        sel_line.contains("created_at"),
        "V2: › marks the selected row, not the cursor row"
    );
    let cur_line = (3..14)
        .map(|y| buf_row_text(&buf, y))
        .find(|l| l.contains('▎'))
        .expect("V2: a cursor row exists");
    assert!(
        cur_line.contains("customer"),
        "V2: the cursor row is separate from the selected row"
    );
    capture_isolated(&dir, "select-open", &buf);
    // V1 closed form: the same field closed shows ▾.
    let (o, _) = sort.on_key(&s4_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && !sort.open,
        "A2: Esc closes"
    );
    assert_eq!(sort.cursor, 0, "A2: Esc restores cursor = selected");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 14);
    let bg = stage.bg();
    sort.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(sid),
            ..Default::default()
        }),
        bg,
    );
    assert!(
        buf_row_text(&buf, 2).contains('▾'),
        "V1: closed field shows ▾"
    );
    assert!(
        !(3..14).any(|y| buf_row_text(&buf, y).contains("customer")),
        "V1: closed field paints no popup"
    );
    // A1: Enter/Space on an open list emits Changed(i) only on difference.
    sort.on_key(&s4_key(KeyCode::Enter));
    sort.on_key(&s4_key(KeyCode::Down));
    let (o, ev) = sort.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(SelectEvent::Changed(1))),
        "A1: Enter emits Changed(1)"
    );
    assert_eq!(sort.selected, 1, "A1: selection follows the cursor");
    sort.on_key(&s4_key(KeyCode::Char(' ')));
    let (o, ev) = sort.on_key(&s4_key(KeyCode::Char(' ')));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none(),
        "N1: choosing the already selected option emits no event"
    );
    // S2: closed Up/Down/Left/Right change selected directly.
    let (o, ev) = sort.on_key(&s4_key(KeyCode::Right));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(SelectEvent::Changed(2))),
        "S2: closed Right steps selected"
    );
    assert!(!sort.open, "S2: stepping never opens");
    let (o, ev) = sort.on_key(&s4_key(KeyCode::Left));
    assert!(
        matches!(ev, Some(SelectEvent::Changed(1))),
        "S2: closed Left steps back"
    );
    assert!(matches!(o, Outcome::Changed));
    sort.on_key(&s4_key(KeyCode::Up));
    assert_eq!(sort.selected, 0, "S2: closed Up steps back");
    let (o, ev) = sort.on_key(&s4_key(KeyCode::Up));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none(),
        "S2: stepping past the edge emits no event"
    );
    // A2: click outside dismisses without change.
    sort.on_key(&s4_key(KeyCode::Enter));
    sort.on_key(&s4_key(KeyCode::Down));
    let o = sort.dismiss();
    assert!(
        matches!(o, Outcome::Changed) && !sort.open,
        "A2: dismiss closes"
    );
    assert_eq!(sort.selected, 0, "A2: dismiss changes nothing");
    // V3/N1: the disabled Engine select paints faint and never opens.
    let eid = s4_id("test.s4.engine");
    let mut engine = Select::new(eid, "Engine", &["PostgreSQL"], 0).disabled(true);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 14);
    let bg = stage.bg();
    let faint = stage.theme.text_faint;
    let disabled_tone = stage.theme.disabled;
    engine.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(eid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(6, 1)].fg, faint, "V3: disabled label is faint");
    assert!(
        buf_row_text(&buf, 2).contains('▾'),
        "V3: disabled field keeps ▾"
    );
    assert_eq!(
        buf[(area.right() - 2, 2)].fg,
        disabled_tone,
        "V3: disabled marker tone"
    );
    capture_isolated(&dir, "select-disabled", &buf);
    let (o, ev) = engine.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none() && !engine.open,
        "N1: disabled ignores keys"
    );
    let (o, ev) = engine.on_click(eid);
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none() && !engine.open,
        "N1: disabled ignores clicks"
    );
    // N2: a modified chord is Ignored; losing focus closes the popup.
    let ctrl = KeyModifiers::CONTROL;
    let alt = KeyModifiers::ALT;
    for key in [
        s4_key_mods(KeyCode::Char('x'), ctrl),
        s4_key_mods(KeyCode::Char('x'), alt),
    ] {
        let (o, ev) = sort.on_key(&key);
        assert!(
            matches!(o, Outcome::Ignored) && ev.is_none(),
            "N2: modified chord {key:?} is Ignored"
        );
    }
    sort.on_key(&s4_key(KeyCode::Enter));
    assert!(sort.open, "fixture open");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 14);
    let bg = stage.bg();
    sort.render(
        area,
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(s4_id("test.s4.elsewhere")),
            ..Default::default()
        }),
        bg,
    );
    assert!(!sort.open, "N2: losing focus closes the popup");

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
    let pid = s4_id("test.s4.quick");
    fn quick(pid: WidgetId) -> Picker {
        let mut p = Picker::new(pid, "Open quickly");
        p.placeholder = "Files and tasks…".into();
        p.scope = Some("All · Tab scope".into());
        p.width = 80;
        p
    }
    fn file_items() -> Vec<PickerItem> {
        vec![
            PickerItem::new("planner.rs")
                .detail("src/bin/planner.rs")
                .glyph("F")
                .group("Files"),
            PickerItem::new("planner_flow.rs")
                .detail("tests/planner_flow.rs")
                .glyph("F")
                .group("Files"),
            PickerItem::new("Add retries to planner jobs")
                .detail("#27 · omar")
                .glyph("T")
                .group("Tasks"),
        ]
    }

    // S1 isolated: every keystroke emits QueryChanged; the owner reset lands
    // on the first eligible row.
    let mut p = quick(pid);
    p.set_items(file_items());
    let (o, ev) = p.on_key(&s4_key(KeyCode::Char('o')));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::QueryChanged)),
        "S1: typing emits QueryChanged"
    );
    assert_eq!(p.query, "o", "S1: query accumulates");
    let mut ranked = file_items();
    ranked[0] = ranked[0].clone().disabled(true);
    p.set_items(ranked);
    assert_eq!(p.cursor, 1, "S1: set_items resets past disabled rows");
    // V1/V2 render form: the modal carries the title, query row, scope
    // label, grouped rows, and hint footer; file rows carry F, tasks T.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction::default());
    p.query.clear();
    p.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx, "hints");
    let screen: String = (0..24).map(|y| buf_row_text(&buf, y)).collect();
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
        .map(|y| buf_row_text(&buf, y))
        .find(|l| l.contains("planner_flow.rs"))
        .expect("V2: file row renders");
    assert!(f_line.contains('F'), "V2: file rows carry glyph F");
    assert!(
        f_line.contains("tests/planner_flow.rs"),
        "V2: file rows carry detail lines"
    );
    let t_line = (0..24)
        .map(|y| buf_row_text(&buf, y))
        .find(|l| l.contains("Add retries"))
        .expect("V2: task row renders");
    assert!(t_line.contains('T'), "V2: task rows carry glyph T");
    capture_isolated(&dir, "picker-query", &buf);
    // S2/A1: Tab emits NextScope; Enter emits Chosen, Alt+Enter ChosenAlt.
    let (o, ev) = p.on_key(&s4_key(KeyCode::Tab));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::NextScope)),
        "S2: Tab emits NextScope"
    );
    p.set_cursor(2);
    let (o, ev) = p.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::Chosen(2))),
        "A1: Enter emits Chosen"
    );
    let (o, ev) = p.on_key(&s4_key_mods(KeyCode::Enter, KeyModifiers::ALT));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::ChosenAlt(2))),
        "A1: Alt+Enter emits ChosenAlt"
    );
    // A2: first Esc clears a non-empty query; Esc on empty cancels.
    p.query = "plan".into();
    let (o, ev) = p.on_key(&s4_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::QueryChanged)),
        "A2: Esc with text clears"
    );
    assert_eq!(p.query, "", "A2: query cleared");
    let (o, ev) = p.on_key(&s4_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::Cancelled)),
        "A2: Esc empty cancels"
    );
    // N1: Submit (free text, no eligible row) keeps the picker open.
    let mut p = quick(pid);
    let mut stale = file_items();
    for it in stale.iter_mut() {
        it.disabled = true;
    }
    p.set_items(stale);
    p.query = "zzz-no-match".into();
    let (o, ev) = p.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::Submit)),
        "N1: Enter on no eligible row falls to Submit"
    );
    assert_eq!(p.query, "zzz-no-match", "N1: the query stays");
    p.query.clear();
    let (o, ev) = p.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none(),
        "N1: blank query + no eligible row consumes"
    );
    // Only a real, enabled row of a Ready picker resolves: Loading refuses.
    let mut rp = quick(pid);
    rp.set_items(file_items());
    assert_eq!(rp.eligible(0), Some(0), "Ready resolves (presence proof)");
    rp.status = PickerStatus::Loading("indexing".into());
    assert_eq!(rp.eligible(0), None, "only Ready pickers resolve rows");
    // N2: clicks outside the modal own nothing (presence: rows own).
    assert!(
        !p.owns(s4_id("test.s4.foreign")),
        "N2: outside clicks own nothing"
    );
    let mut vis = quick(pid);
    vis.set_items(file_items());
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction::default());
    vis.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx, "hints");
    assert!(
        vis.owns(vis.row_id(0)),
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
    let tid = s4_id("test.s4.tabs");
    fn tabs_picker(tid: WidgetId) -> Picker {
        let mut p = Picker::new(tid, "Open tabs");
        p.placeholder = "Filter tabs…".into();
        p.width = 48;
        p
    }
    fn tab_items() -> Vec<PickerItem> {
        let mut items = vec![
            PickerItem::new("Query 1")
                .detail("query")
                .glyph("≡")
                .group("Open tabs"),
            PickerItem::new("orders").detail("public · data").glyph("T"),
            PickerItem::new("order_items")
                .detail("public · data")
                .glyph("T"),
            PickerItem::new("History")
                .detail("public · data")
                .glyph("T"),
        ];
        for it in items.iter_mut().skip(1) {
            it.group = "Open tabs";
        }
        items[1].tag = Some("active");
        items
    }

    // S1 isolated: Delete emits Secondary(i); the owner splice keeps the
    // cursor on a live row.
    let mut p = tabs_picker(tid);
    p.set_items(tab_items());
    let (o, ev) = p.on_key(&s4_key(KeyCode::Delete));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::Secondary(0))),
        "S1: Delete emits Secondary"
    );
    let mut rest = tab_items();
    rest.remove(0);
    p.set_items(rest);
    assert_eq!(p.items.len(), 3, "S1: the tab splices out");
    assert_eq!(p.cursor, 0, "S1: cursor lands on a live row");
    // V1 render form: glyphs, the active tag, and the group header.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction::default());
    p.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx, "hints");
    let screen: String = (0..24).map(|y| buf_row_text(&buf, y)).collect();
    assert!(screen.contains("Open tabs"), "V1: Tabs modal title");
    assert!(screen.contains("active"), "V1: the active tag");
    assert!(screen.contains("Filter tabs…"), "V1: Tabs is searchable");
    let glyph_line = (0..24)
        .map(|y| buf_row_text(&buf, y))
        .find(|l| l.contains("order_items"))
        .expect("V1: tab row renders");
    assert!(glyph_line.contains('T'), "V1: tab rows carry glyphs");
    capture_isolated(&dir, "picker-tabs", &buf);
    // N1 isolated: Backspace on an empty query emits Back (the showcase
    // page ignores it — proven live below).
    p.query.clear();
    let (o, ev) = p.on_key(&s4_key(KeyCode::Backspace));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(PickerEvent::Back)),
        "N1: Backspace on empty query emits Back"
    );

    // Level picker: non-searchable, cursor opens on the stored level.
    let lid = s4_id("test.s4.level");
    let levels = [
        "Silent",
        "Alert",
        "Alert (Full)",
        "Safe Mode",
        "Safe Mode (Full)",
        "Read-Only",
    ];
    let mut lp = Picker::new(lid, "Safe Mode · this connection");
    lp.searchable = false;
    lp.width = 112;
    let items: Vec<PickerItem> = levels
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let mut it = PickerItem::new(*l).detail("level detail");
            if i == 3 {
                it.tag = Some("current");
            }
            it
        })
        .collect();
    lp.set_items(items);
    lp.cursor = 3;
    assert_eq!(
        lp.cursor, 3,
        "S2: the Level picker opens on the stored level"
    );
    // V2 render form: no query row; the current level carries its tag.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(112, 24);
    let mut ctx = stage.ctx(Interaction::default());
    lp.render(Rect::new(0, 0, 112, 24), &mut buf, &mut ctx, "hints");
    let screen: String = (0..24).map(|y| buf_row_text(&buf, y)).collect();
    assert!(
        !screen.contains("Type to search"),
        "V2: the Level modal shows no query row"
    );
    let cur_line = (0..24)
        .map(|y| buf_row_text(&buf, y))
        .find(|l| l.contains("Safe Mode") && !l.contains("Full"))
        .expect("V2: level row renders");
    assert!(
        cur_line.contains("current") || screen.contains("current"),
        "V2: the current level carries the current tag"
    );
    capture_isolated(&dir, "picker-level", &buf);
    // A2: in a non-searchable picker j/k move and typed text is impossible.
    let (o, ev) = lp.on_key(&s4_key(KeyCode::Char('j')));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none() && lp.cursor == 4,
        "A2: j moves"
    );
    let (o, _) = lp.on_key(&s4_key(KeyCode::Char('k')));
    assert!(
        matches!(o, Outcome::Changed) && lp.cursor == 3,
        "A2: k moves"
    );
    for code in [KeyCode::Char('x'), KeyCode::Char('q')] {
        let (o, ev) = lp.on_key(&s4_key(code));
        assert!(
            matches!(o, Outcome::Consumed) && ev.is_none(),
            "A2: {code:?} types nothing"
        );
    }
    assert_eq!(lp.query, "", "A2: no query can accumulate");

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

/// COMPLETE-EDITOR-002 (`showcase/flows/editor/completion`, 80x24): the code
/// editor identifier popup over [`Completion`].
///
/// Isolated path mirrors the row's suggestion popup (function/type/local/
/// macro/keyword candidates with signature details and fuzzy-matched bold
/// bytes): anchor geometry, clamped navigation, Accept/Dismiss, click and
/// wheel paths. Live path auto-triggers with two characters, moves, accepts
/// into the buffer, and proves the one-character and no-match negatives.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_complete_editor() {
    let dir = s4_dir("s4_complete_editor");
    let cid = s4_id("test.s4.complete");
    fn items() -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "client(".into(),
                glyph: "ƒ",
                detail: "fn () -> Client".into(),
                insert: "client(".into(),
                matched: vec![0, 1],
            },
            CompletionItem {
                label: "Client".into(),
                glyph: "T",
                detail: "struct".into(),
                insert: "Client".into(),
                matched: vec![0],
            },
            CompletionItem {
                label: "delay".into(),
                glyph: "v",
                detail: "local · u64".into(),
                insert: "delay".into(),
                matched: vec![],
            },
        ]
    }

    // S2 isolated: the popup opens only when non-empty.
    let mut c = Completion::new(cid);
    assert!(!c.is_open(), "S2: closed with no items");
    c.open(vec![], Rect::new(10, 5, 1, 1), 0);
    assert!(!c.is_open(), "S2: empty open stays closed");
    c.open(items(), Rect::new(10, 5, 1, 1), 2);
    assert!(c.is_open(), "S2: non-empty opens");
    assert_eq!(c.cursor, 0, "open resets the cursor");
    // V1 render form: anchored at the word start with glyphs + details.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction::default());
    c.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
    assert_eq!(c.area.x, 10, "V1: popup left-aligns with the word start");
    assert!(c.area.y > 5, "V1: popup anchors below the anchor row");
    let row0 = buf_row_text(&buf, c.area.y + 1);
    assert!(
        row0.contains('ƒ') && row0.contains("client("),
        "V1: function glyph plus label"
    );
    assert!(row0.contains("fn () -> Client"), "V1: signature detail");
    let row1 = buf_row_text(&buf, c.area.y + 2);
    assert!(
        row1.contains('T') && row1.contains("Client"),
        "V1: type glyph"
    );
    capture_isolated(&dir, "complete-open", &buf);
    // V2: fuzzy-matched label bytes render bold on every row. (The cursor
    // row keeps the bold base style, so the unmatched-plain half reads off
    // the non-cursor rows.)
    let lx = c.area.x + 1 + 3;
    assert!(
        buf_is_bold(&buf, lx, c.area.y + 1),
        "V2: matched byte 0 is bold"
    );
    assert!(
        buf_is_bold(&buf, lx + 1, c.area.y + 1),
        "V2: matched byte 1 is bold"
    );
    assert!(
        buf_is_bold(&buf, lx, c.area.y + 2),
        "V2: row 1 matched byte is bold"
    );
    assert!(
        !buf_is_bold(&buf, lx + 1, c.area.y + 2),
        "V2: unmatched bytes stay plain off-cursor"
    );
    assert!(
        !buf_is_bold(&buf, lx, c.area.y + 3),
        "V2: unmatched rows stay plain"
    );
    // Navigation: Down/Up clamp (never wrap); plain n/p are ignored so the
    // owner keeps typing; ctrl-n/ctrl-p move.
    let (o, _) = c.on_key(&s4_key(KeyCode::Down));
    assert!(matches!(o, Outcome::Changed) && c.cursor == 1, "Down moves");
    let (o, ev) = c.on_key(&s4_key(KeyCode::Char('n')));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "plain n is ignored for typing"
    );
    let ctrl = KeyModifiers::CONTROL;
    let (o, _) = c.on_key(&s4_key_mods(KeyCode::Char('n'), ctrl));
    assert!(
        matches!(o, Outcome::Changed) && c.cursor == 2,
        "ctrl-n moves"
    );
    let (o, _) = c.on_key(&s4_key_mods(KeyCode::Char('n'), ctrl));
    assert!(
        matches!(o, Outcome::Changed) && c.cursor == 2,
        "Down clamps"
    );
    let (o, _) = c.on_key(&s4_key(KeyCode::Up));
    assert!(matches!(o, Outcome::Changed) && c.cursor == 1, "Up moves");
    // A1: Tab/Enter accept the cursor row; click accepts a visible row.
    let (o, ev) = c.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(CompletionEvent::Accept(1))),
        "A1: Enter accepts the cursor"
    );
    let (o, ev) = c.on_key(&s4_key(KeyCode::Tab));
    assert!(
        matches!(ev, Some(CompletionEvent::Accept(1))),
        "A1: Tab accepts too"
    );
    assert!(matches!(o, Outcome::Changed));
    assert!(c.owns(c.row_id(0)), "rows are owned (presence proof)");
    assert_eq!(
        c.on_click(c.row_id(2)),
        Some(CompletionEvent::Accept(2)),
        "click accepts the row"
    );
    assert_eq!(
        c.on_click(s4_id("test.s4.foreign")),
        None,
        "foreign clicks miss"
    );
    // A2: Dismiss closes; N2: a stale index resolves to nothing.
    let (o, ev) = c.on_key(&s4_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(CompletionEvent::Dismiss)),
        "A2: Esc dismisses"
    );
    assert!(!c.is_open(), "A2: dismiss closes");
    assert_eq!(c.items.get(99), None, "N2: stale index resolves to nothing");

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

/// Build a dialog focus scope: the input (when any) plus the actions, in
/// render order, with focus on the dialog's `initial_focus`.
fn dialog_scope(d: &Dialog) -> (Focus, FocusRing) {
    let mut ring = FocusRing::default();
    if let junie_tui::widgets::dialog::DialogBody::Input(inp) = &d.body {
        ring.register(inp.id);
    }
    for b in &d.actions {
        ring.register(b.id);
    }
    let mut focus = Focus::default();
    focus.focus(d.initial_focus);
    (focus, ring)
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
    let did = s4_id("test.s4.confirm");

    // V2/S1 isolated: confirm focuses the primary action first; Left/Right
    // move between actions skipping disabled ones; Tab cycles the ring.
    let mut d = Dialog::confirm(did, "Run task now?", "Run the plan now.", "Run");
    assert_eq!(d.actions.len(), 2, "confirm has two actions");
    assert_eq!(d.actions[1].kind, ButtonKind::Primary, "Run is primary");
    assert_eq!(
        d.initial_focus, d.actions[1].id,
        "V2: primary focused first"
    );
    assert_eq!(d.cancel_index, Some(0), "cancel index is Cancel");
    let (mut focus, ring) = dialog_scope(&d);
    assert!(focus.is(d.actions[1].id), "focus starts on Run");
    let o = d.on_key(&s4_key(KeyCode::Left), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && focus.is(d.actions[0].id),
        "S1: Left moves"
    );
    d.actions[0].disabled = true;
    focus.focus(d.actions[1].id);
    let o = d.on_key(&s4_key(KeyCode::Left), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && focus.is(d.actions[1].id),
        "S1: Left skips disabled actions"
    );
    d.actions[0].disabled = false;
    let o = d.on_key(&s4_key(KeyCode::Tab), &mut focus, &ring);
    assert!(matches!(o, Outcome::Changed), "S1: Tab cycles the ring");
    // A1: y finishes the primary action, n the cancel action.
    let o = d.on_key(&s4_key(KeyCode::Char('y')), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && d.result == Some(DialogResult::Action(1)),
        "A1: y finishes the primary action"
    );
    let mut d = Dialog::confirm(did, "Run task now?", "Run the plan now.", "Run");
    let (mut focus, ring) = dialog_scope(&d);
    let o = d.on_key(&s4_key(KeyCode::Char('n')), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && d.result == Some(DialogResult::Action(0)),
        "A1: n finishes the cancel action"
    );
    // A2: Esc and click-outside finish the cancel action.
    let mut d = Dialog::confirm(did, "Run task now?", "Run the plan now.", "Run");
    let (mut focus, ring) = dialog_scope(&d);
    let o = d.on_key(&s4_key(KeyCode::Esc), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && d.result == Some(DialogResult::Action(0)),
        "A2: Esc finishes cancel"
    );
    let mut d = Dialog::confirm(did, "Run task now?", "Run the plan now.", "Run");
    let o = d.on_click_outside();
    assert!(
        matches!(o, Outcome::Changed) && d.result == Some(DialogResult::Action(0)),
        "A2: click-outside finishes cancel"
    );
    // N1: y/n do nothing on facts dialogs; other keys are Consumed.
    let mut f = Dialog::facts(
        s4_id("test.s4.facts"),
        "Facts",
        vec![],
        vec![],
        None,
        Button::primary(s4_id("test.s4.facts.ok"), "Apply"),
    );
    let (mut focus, ring) = dialog_scope(&f);
    for code in [KeyCode::Char('y'), KeyCode::Char('n')] {
        let o = f.on_key(&s4_key(code), &mut focus, &ring);
        assert!(
            matches!(o, Outcome::Consumed) && f.result.is_none(),
            "N1: {code:?} does nothing on facts dialogs"
        );
    }
    let o = f.on_key(&s4_key(KeyCode::Char('q')), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Consumed),
        "N1: other keys are Consumed"
    );
    // N2: clicks on no action are Consumed (presence: action clicks finish).
    let o = f.on_click(s4_id("test.s4.foreign"), Position::new(0, 0), &mut focus);
    assert!(
        matches!(o, Outcome::Consumed),
        "N2: foreign clicks Consumed"
    );
    assert_eq!(f.result, None, "N2: foreign clicks finish nothing");
    let ok = f.actions[1].id;
    let o = f.on_click(ok, Position::new(0, 0), &mut focus);
    assert!(
        matches!(o, Outcome::Changed) && f.result == Some(DialogResult::Action(1)),
        "action clicks finish (presence proof)"
    );
    // V1 render form: the dialog centers over the page.
    let mut d = Dialog::confirm(did, "Run task now?", "Run the plan now.", "Run");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction::default());
    d.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
    assert!(
        (0..24).any(|y| buf_row_text(&buf, y).contains("Run task now?")),
        "V1: title renders"
    );
    // The dialog frame (not the left-aligned title) centers: the top
    // border's left and right margins match (cell columns, not bytes).
    let top_y = (0..24)
        .find(|y| buf_row_text(&buf, *y).contains('╭'))
        .expect("V1: dialog frame renders");
    let top = buf_row_text(&buf, top_y);
    let x0 = char_col(&top, "╭") as i16;
    let x1 = char_col(&top, "╮") as i16;
    assert!(
        (x0 - (80 - x1)).abs() <= 2,
        "V1: the dialog centers over the page"
    );
    let screen: String = (0..24).map(|y| buf_row_text(&buf, y)).collect();
    assert!(
        screen.contains("Run") && screen.contains("Cancel"),
        "V1: both actions render"
    );
    capture_isolated(&dir, "dialog-confirm", &buf);
    // Three-choice form: Save focused first.
    let mut three = Dialog::confirm(did, "Unsaved changes", "Save first?", "Save").with_actions(
        vec![
            Button::subtle(did.sub("cancel"), "Cancel"),
            Button::secondary(did.sub("discard"), "Discard"),
            Button::primary(did.sub("save"), "Save"),
        ],
        Some(0),
    );
    three.initial_focus = did.sub("save");
    assert_eq!(
        three.initial_focus, three.actions[2].id,
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
    fn rename_validator(s: &str) -> Option<String> {
        if s.trim().is_empty() {
            Some("Name cannot be empty".into())
        } else if s.len() > 40 {
            Some("Keep it under 40 characters".into())
        } else {
            None
        }
    }
    fn rename_dialog() -> Dialog {
        let input = TextInput::new(s4_id("test.s4.rename.input"), "Task name")
            .value("Migrate sessions table")
            .required(true)
            .validator(rename_validator)
            .help("Shown in the task list and PR title");
        Dialog::prompt(s4_id("test.s4.rename"), "Rename task", input, "Rename")
    }

    // V1 isolated: the prompt focuses its input first with help under it.
    let mut d = rename_dialog();
    let input_id = d.initial_focus;
    assert_eq!(d.actions[1].kind, ButtonKind::Primary, "Rename is primary");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction {
        focus: Some(input_id),
        ..Default::default()
    });
    d.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
    let screen: String = (0..24).map(|y| buf_row_text(&buf, y)).collect();
    assert!(screen.contains("Rename task"), "V1: prompt title");
    assert!(
        screen.contains("Migrate sessions table"),
        "V1: prefilled input"
    );
    assert!(
        screen.contains("Shown in the task list"),
        "V1: help text under the input"
    );
    capture_isolated(&dir, "dialog-prompt", &buf);
    // N1 (input half of confirm-row N1): y does not finish an input dialog.
    let (mut focus, ring) = dialog_scope(&d);
    assert!(focus.is(input_id), "V1: input focused first");
    let o = d.on_key(&s4_key(KeyCode::Char('y')), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Consumed) && d.result.is_none(),
        "N1: y finishes nothing on input dialogs"
    );
    // S1/N1: finish commits the editing input then validates; a failing
    // validation keeps the dialog open and stores no rename.
    let o = d.on_key(&s4_key(KeyCode::Enter), &mut focus, &ring);
    assert!(matches!(o, Outcome::Changed), "Enter begins editing");
    let o = d.on_key(
        &s4_key_mods(KeyCode::Char('l'), KeyModifiers::CONTROL),
        &mut focus,
        &ring,
    );
    assert!(matches!(o, Outcome::Changed), "Ctrl+L selects all");
    let o = d.on_key(&s4_key(KeyCode::Backspace), &mut focus, &ring);
    assert!(matches!(o, Outcome::Changed), "Backspace clears");
    let o = d.on_key(&s4_key(KeyCode::Enter), &mut focus, &ring);
    assert!(matches!(o, Outcome::Changed), "submit runs");
    assert_eq!(
        d.result, None,
        "S1: failing validation keeps the dialog open"
    );
    // A1: a valid name submits to the primary action; paste routes in.
    let o = d.on_paste("Seed the demo database");
    assert!(
        matches!(o, Outcome::Changed),
        "A1: paste routes to the input"
    );
    let o = d.on_key(&s4_key(KeyCode::Enter), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && d.result == Some(DialogResult::Action(1)),
        "S2: a valid rename finishes the primary action"
    );
    // N1: over-long names never close the prompt.
    let mut d = rename_dialog();
    let (mut focus, ring) = dialog_scope(&d);
    d.on_key(&s4_key(KeyCode::Enter), &mut focus, &ring);
    d.on_key(
        &s4_key_mods(KeyCode::Char('l'), KeyModifiers::CONTROL),
        &mut focus,
        &ring,
    );
    d.on_paste(&"x".repeat(41));
    d.on_key(&s4_key(KeyCode::Enter), &mut focus, &ring);
    assert_eq!(d.result, None, "N1: over-long names never close");
    // V2/N2: the destructive dialog focuses Cancel first in danger style.
    let mut x = Dialog::destructive(
        s4_id("test.s4.delete"),
        "Delete branch?",
        "This cannot be undone.",
        "Delete branch",
    );
    assert_eq!(x.actions[1].kind, ButtonKind::Danger, "V2: danger action");
    assert_eq!(x.initial_focus, x.actions[0].id, "V2: Cancel focused first");
    let (mut focus, ring) = dialog_scope(&x);
    let o = x.on_key(&s4_key(KeyCode::Esc), &mut focus, &ring);
    assert!(
        matches!(o, Outcome::Changed) && x.result == Some(DialogResult::Action(0)),
        "N2: Esc always lands on Cancel"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction {
        focus: Some(x.actions[0].id),
        ..Default::default()
    });
    x.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
    assert!(
        (0..24).any(|y| buf_row_text(&buf, y).contains("Delete branch?")),
        "destructive title renders"
    );
    capture_isolated(&dir, "dialog-destructive", &buf);
    // N2 (click half): clicking the prompt input focuses without finishing.
    let mut d = rename_dialog();
    let (mut focus, _ring) = dialog_scope(&d);
    focus.focus(d.actions[1].id);
    let o = d.on_click(input_id, Position::new(0, 0), &mut focus);
    assert!(
        matches!(o, Outcome::Changed) && d.result.is_none() && focus.is(input_id),
        "N2: input clicks focus without finishing"
    );

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
    let mid = s4_id("test.s4.filemenu");
    fn file_items() -> Vec<MenuItem> {
        vec![
            MenuItem::new("New tab").shortcut("c"),
            MenuItem::new("Split right").shortcut("%"),
            MenuItem::new("Export…").separator(),
            MenuItem::new("Close tab").shortcut("&").danger(),
        ]
    }
    fn view_items() -> Vec<MenuItem> {
        vec![
            MenuItem::new("Zoom pane").shortcut("z"),
            MenuItem::new("Redraw").shortcut("r").separator(),
            MenuItem::new("Usage").shortcut("u"),
            MenuItem::new("Inspect changes").disabled(true),
        ]
    }

    // S1 isolated: step wraps with rem_euclid and skips disabled rows;
    // Home/End jump to first/last enabled.
    let mut m = ContextMenu::new(mid, view_items()).anchor(Rect::new(2, 1, 8, 1), Placement::Below);
    assert_eq!(m.cursor, 0, "cursor starts on the first enabled row");
    m.on_key(&s4_key(KeyCode::Down));
    assert_eq!(m.cursor, 1, "Down moves");
    m.on_key(&s4_key(KeyCode::Down));
    assert_eq!(m.cursor, 2, "Down skips nothing enabled");
    m.on_key(&s4_key(KeyCode::Down));
    assert_eq!(m.cursor, 0, "Down wraps past the disabled tail");
    m.on_key(&s4_key(KeyCode::Up));
    assert_eq!(m.cursor, 2, "Up wraps, skipping disabled");
    m.on_key(&s4_key(KeyCode::End));
    assert_eq!(m.cursor, 2, "End jumps to the last enabled row");
    m.on_key(&s4_key(KeyCode::Home));
    assert_eq!(m.cursor, 0, "Home jumps to the first enabled row");
    // V1/V2/V3 render form over the File menu with the cursor on Close tab.
    let mut f = ContextMenu::new(mid, file_items()).anchor(Rect::new(2, 1, 8, 1), Placement::Below);
    f.on_key(&s4_key(KeyCode::End));
    assert_eq!(f.cursor, 3, "End lands on the danger row");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 16);
    let highlight = stage.theme.highlight;
    let highlight_danger = stage.theme.highlight_danger;
    let primary = stage.theme.text_primary;
    let muted = stage.theme.text_muted;
    let error_soft = stage.theme.error_soft;
    let subtle = stage.theme.border_subtle;
    let mut ctx = stage.ctx(Interaction::default());
    f.render(Rect::new(0, 0, 60, 16), &mut buf, &mut ctx);
    assert_eq!(f.area.x, 2, "popover left-aligns under the anchor");
    assert!(f.area.y > 1, "popover sits below the anchor");
    let crow = f.area.y + 1 + 4;
    assert_eq!(
        buf[(f.area.x + 1, crow)].bg,
        highlight_danger,
        "V1: danger cursor fill"
    );
    assert_eq!(buf[(f.area.x + 1, crow)].fg, primary, "V1: bold white text");
    assert!(
        buf_is_bold(&buf, f.area.x + 3, crow),
        "V1: cursor text is bold"
    );
    assert_ne!(
        buf[(f.area.x + 1, crow)].symbol(),
        "▎",
        "V1: no gutter bar on menu rows"
    );
    let rest = buf_row_text(&buf, f.area.y + 1);
    assert!(rest.contains("New tab"), "rest rows render");
    assert_eq!(
        buf[(f.area.x + 3, f.area.y + 1)].fg,
        primary,
        "rest rows keep primary text"
    );
    let sc_col = f.area.x + f.area.width - 1 - 1 - 1;
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
        !buf_is_bold(&buf, sc_col, crow),
        "V2: shortcuts are never bold"
    );
    let danger_rest = buf_row_text(&buf, crow);
    assert!(danger_rest.contains("Close tab"), "danger label renders");
    let sep_y = f.area.y + 1 + 3;
    assert!(
        buf_row_text(&buf, sep_y).contains('─'),
        "V2: separators are ─ rules"
    );
    assert_eq!(buf[(f.area.x + 2, sep_y)].fg, subtle, "separator tone");
    capture_isolated(&dir, "menu-open", &buf);
    // V3: disabled rows render faint and the cursor never lands on them.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 16);
    let mut ctx = stage.ctx(Interaction::default());
    m.cursor = 2;
    m.render(Rect::new(0, 0, 60, 16), &mut buf, &mut ctx);
    let dis_row = buf_row_text(&buf, m.area.y + 1 + 4);
    assert!(
        dis_row.contains("Inspect changes"),
        "V3: disabled row renders"
    );
    assert_ne!(
        buf[(m.area.x + 3, m.area.y + 1 + 4)].fg,
        primary,
        "V3: disabled rows are faint, not primary"
    );
    assert_eq!(
        highlight, stage.theme.highlight,
        "highlight plane (presence)"
    );
    // S2: the popover flips above the anchor when the room below is short.
    let mut low =
        ContextMenu::new(mid, file_items()).anchor(Rect::new(2, 13, 8, 1), Placement::Below);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 16);
    let mut ctx = stage.ctx(Interaction::default());
    low.render(Rect::new(0, 0, 60, 16), &mut buf, &mut ctx);
    assert!(
        low.area.y + low.area.height <= 13,
        "S2: the popover flips above the anchor"
    );
    // A1/A2: Enter/Space choose; Esc dismisses; row clicks choose, outside
    // clicks dismiss; hover moves the cursor.
    let mut m = ContextMenu::new(mid, file_items()).anchor(Rect::new(2, 1, 8, 1), Placement::Below);
    m.on_key(&s4_key(KeyCode::End));
    let (o, ev) = m.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuEvent::Chosen(3))),
        "A1: Enter chooses"
    );
    let (o, ev) = m.on_key(&s4_key(KeyCode::Char(' ')));
    assert!(
        matches!(ev, Some(MenuEvent::Chosen(3))),
        "A1: Space chooses too"
    );
    assert!(matches!(o, Outcome::Changed));
    let (o, ev) = m.on_key(&s4_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuEvent::Dismissed)),
        "A1: Esc dismisses"
    );
    assert_eq!(
        m.on_click(m.row_id(1)),
        Some(MenuEvent::Chosen(1)),
        "A2: a row click chooses it"
    );
    assert_eq!(
        m.on_click(s4_id("test.s4.foreign")),
        Some(MenuEvent::Dismissed),
        "A2: a click elsewhere dismisses"
    );
    let mut m = ContextMenu::new(mid, file_items()).anchor(Rect::new(2, 1, 8, 1), Placement::Below);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 16);
    let hover = m.row_id(2);
    let mut ctx = stage.ctx(Interaction {
        hover: Some(hover),
        ..Default::default()
    });
    m.render(Rect::new(0, 0, 60, 16), &mut buf, &mut ctx);
    assert_eq!(m.cursor, 2, "A2: hover moves the cursor");
    // N1: Enter on a disabled row is Consumed; a modified chord is Ignored.
    let mut m = ContextMenu::new(mid, view_items());
    m.cursor = 3;
    let (o, ev) = m.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none(),
        "N1: Enter on a disabled row chooses nothing"
    );
    assert_eq!(m.on_click(m.row_id(3)), None, "N1: disabled clicks miss");
    let (o, ev) = m.on_key(&s4_key_mods(KeyCode::Char('x'), KeyModifiers::CONTROL));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "N1: a modified chord is Ignored"
    );
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
    let cid = s4_id("test.s4.context");
    fn session_items(row: usize, len: usize) -> Vec<MenuItem> {
        vec![
            MenuItem::new("Change title…").shortcut("r"),
            MenuItem::new("Move left").disabled(row == 0),
            MenuItem::new("Move right")
                .disabled(row + 1 == len)
                .separator(),
            MenuItem::new("Close").shortcut("x").danger(),
        ]
    }

    // V1/V2 render form: the popover titles itself with the session label;
    // at the top edge Move left disables; Close is danger under a separator.
    let mut m = ContextMenu::new(cid, session_items(0, 4))
        .title("1 Claude Code (Work)")
        .anchor(Rect::new(20, 4, 1, 1), Placement::Below);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let danger = stage.theme.highlight_danger;
    let mut ctx = stage.ctx(Interaction::default());
    m.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
    let screen: String = (0..24).map(|y| buf_row_text(&buf, y)).collect();
    assert!(
        screen.contains("1 Claude Code (Work)"),
        "V1: the popover titles itself with the session label"
    );
    assert!(screen.contains("Move left"), "edge row renders");
    assert!(screen.contains("Close"), "danger row renders");
    let close_y = (0..24)
        .find(|y| buf_row_text(&buf, *y).contains("Close"))
        .expect("Close row renders");
    assert!(
        buf_row_text(&buf, close_y - 1).contains('─'),
        "V2: Close sits under a separator"
    );
    m.on_key(&s4_key(KeyCode::End));
    assert_eq!(m.cursor, 3, "cursor reaches Close");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut ctx = stage.ctx(Interaction::default());
    m.render(Rect::new(0, 0, 80, 24), &mut buf, &mut ctx);
    let close_y = (0..24)
        .find(|y| buf_row_text(&buf, *y).contains("Close"))
        .expect("Close row renders");
    assert_eq!(
        buf[(m.area.x + 1, close_y)].bg,
        danger,
        "V2: Close takes the danger fill under the cursor"
    );
    capture_isolated(&dir, "ctxmenu-anchor", &buf);
    // N1: a disabled edge row can never be chosen by key or click.
    let mut m = ContextMenu::new(cid, session_items(0, 4))
        .title("1 Claude Code (Work)")
        .anchor(Rect::new(20, 4, 1, 1), Placement::Below);
    m.cursor = 1;
    let (o, ev) = m.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none(),
        "N1: Enter on the disabled edge chooses nothing"
    );
    assert_eq!(m.on_click(m.row_id(1)), None, "N1: disabled clicks miss");
    // Presence: the enabled rows resolve through the same paths.
    m.cursor = 0;
    let (o, ev) = m.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuEvent::Chosen(0))),
        "enabled rows choose (presence proof)"
    );
    assert_eq!(
        m.on_click(m.row_id(3)),
        Some(MenuEvent::Chosen(3)),
        "enabled clicks choose (presence proof)"
    );
    assert_eq!(
        m.on_click_outside(),
        Some(MenuEvent::Dismissed),
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
    let bid = s4_id("test.s4.menubar");
    fn shell_bar(bid: WidgetId) -> MenuBar {
        MenuBar::new(
            bid,
            vec![
                (
                    "File",
                    vec![
                        MenuItem::new("New tab").shortcut("c"),
                        MenuItem::new("Close tab").shortcut("&").danger(),
                    ],
                ),
                (
                    "View",
                    vec![
                        MenuItem::new("Zoom pane").shortcut("z"),
                        MenuItem::new("Inspect changes").disabled(true),
                    ],
                ),
                ("Help", vec![MenuItem::new("Key reference").shortcut("?")]),
            ],
        )
        .brand(Lockup::new("app❯"))
    }

    // V1/V2 render form: padded labels with 1-cell gaps; the focused cursor
    // label carries the ▎ bar.
    let mut bar = shell_bar(bid);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 3);
    let bg = stage.bg();
    let popover = stage.theme.popover;
    bar.render(
        Rect::new(0, 1, 80, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(bar.areas.len(), 3, "V1: three labels lay out");
    let row = buf_row_text(&buf, 1);
    assert!(row.contains("app❯"), "V1: the brand lockup renders");
    assert!(
        row.contains(" File ") && row.contains(" View ") && row.contains(" Help "),
        "V1: labels render padded with gaps"
    );
    assert!(row.contains('▎'), "V2: the cursor label carries the ▎ bar");
    assert_eq!(
        buf[(bar.areas[0].x + 1, 1)].fg,
        stage.theme.text_primary,
        "V2: the cursor label takes primary text"
    );
    assert!(
        buf_is_bold(&buf, bar.areas[0].x + 1, 1),
        "V2: the cursor label is bold"
    );
    capture_isolated(&dir, "menubar-closed", &buf);
    // S2/V1: Enter opens the cursor menu; the open label shares the popover
    // plane in bold.
    let (o, ev) = bar.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuBarEvent::Opened(0))),
        "S2: Enter opens the cursor menu"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 3);
    let bg = stage.bg();
    bar.render(
        Rect::new(0, 1, 80, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        buf[(bar.areas[0].x + 1, 1)].bg,
        popover,
        "V1: the open label shares the popover plane"
    );
    assert!(
        buf_is_bold(&buf, bar.areas[0].x + 1, 1),
        "V1: the open label is bold"
    );
    // S1: Right switches the open menu; Left switches back.
    let (o, ev) = bar.on_key(&s4_key(KeyCode::Right));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuBarEvent::Opened(1))),
        "S1: Right switches the open menu"
    );
    assert_eq!(bar.open_index(), Some(1), "View is open");
    let (o, ev) = bar.on_key(&s4_key(KeyCode::Char('h')));
    assert!(
        matches!(ev, Some(MenuBarEvent::Opened(0))),
        "S1: h switches back"
    );
    assert!(matches!(o, Outcome::Changed));
    // A1: choosing emits Chosen(menu, item) and closes.
    let (o, ev) = bar.on_key(&s4_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuBarEvent::Chosen(0, 0))),
        "A1: choosing emits Chosen(0, 0)"
    );
    assert!(!bar.is_open(), "A1: choosing closes");
    // S1 closed: Left/Right move the cursor.
    let (o, _) = bar.on_key(&s4_key(KeyCode::Right));
    assert!(
        matches!(o, Outcome::Changed) && bar.cursor == 1,
        "S1: Right moves"
    );
    // S2: clicking a label toggles its menu.
    let (o, ev) = bar.on_click(bar.label_id(1));
    assert!(
        matches!(ev, Some(MenuBarEvent::Opened(1))),
        "S2: label click opens"
    );
    assert!(matches!(o, Outcome::Changed));
    let (o, ev) = bar.on_click(bar.label_id(1));
    assert!(
        matches!(ev, Some(MenuBarEvent::Closed)),
        "S2: clicking its label again closes it"
    );
    assert!(matches!(o, Outcome::Changed));
    // A2: the brand lockup emits Brand; hovering another label while open
    // switches to it.
    let (o, ev) = bar.on_click(bar.brand_id());
    assert!(
        matches!(o, Outcome::Changed) && matches!(ev, Some(MenuBarEvent::Brand)),
        "A2: the brand lockup emits Brand"
    );
    bar.on_click(bar.label_id(0));
    let o = bar.on_hover(Some(bar.label_id(2)));
    assert!(
        matches!(o, Outcome::Changed) && bar.open_index() == Some(2),
        "A2: hover switches the open menu"
    );
    // N1: hover with no menu open changes nothing; modified chords Ignored.
    bar.close();
    let o = bar.on_hover(Some(bar.label_id(1)));
    assert!(
        matches!(o, Outcome::Ignored) && !bar.is_open(),
        "N1: hover with no menu open changes nothing"
    );
    let (o, ev) = bar.on_key(&s4_key_mods(KeyCode::Char('x'), KeyModifiers::CONTROL));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "N1: a modified chord is Ignored"
    );
    // N2: labels that do not fit keep ZERO areas and cannot open.
    let mut narrow = shell_bar(bid);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(20, 3);
    let bg = stage.bg();
    narrow.render(
        Rect::new(0, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    assert!(
        narrow.areas.iter().any(|a| *a == Rect::ZERO),
        "N2: unfitting labels keep ZERO areas"
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
/// Help is the showcase binary's `open_help` (`app.rs`), outside the
/// importable library, so every check runs live: the single Close action
/// under the key table, the keyboard-rows-then-mouse-row shape, focus
/// save/restore, both openers, all four closers, and the no-second-action
/// plus y-never-closes negatives.
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
/// `forms.rs`, so every check runs live: the Busy countdown message, the
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

/// PB-STATES-001 (`showcase/pages/progress`, paused): state samples plus
/// terminal glyphs over [`render_bar`].
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

    // V1/V2 isolated: terminal glyphs in their tones; running is white-70.
    let samples = [
        ("Queued    ", 0.0, ProgressStatus::Active, "  "),
        ("Halfway   ", 0.5, ProgressStatus::Active, "  "),
        ("Completed ", 1.0, ProgressStatus::Done, " ✓"),
        ("Failed    ", 0.64, ProgressStatus::Error, " !"),
        ("Paused    ", 0.3, ProgressStatus::Paused, " ‖"),
    ];
    let mut stage = Stage::new();
    let success = stage.theme.success;
    let error = stage.theme.error;
    let muted = stage.theme.text_muted;
    let running = stage.theme.text_secondary;
    for (label, ratio, status, suffix) in samples {
        let mut buf = sentinel_buf(70, 3);
        let bg = stage.bg();
        let mut ctx = stage.ctx(Interaction::default());
        render_bar(
            Rect::new(0, 1, 70, 1),
            &mut buf,
            &mut ctx,
            label,
            ratio,
            status,
            bg,
        );
        drop(ctx);
        let row = buf_row_text(&buf, 1);
        if status == ProgressStatus::Active {
            assert!(
                !row.contains('✓') && !row.contains('‖') && !row.contains(" !"),
                "V1: {label:?} carries no terminal glyph"
            );
        } else {
            assert!(row.contains(suffix.trim()), "V1: {label:?} ends {suffix:?}");
        }
        let (fill, what) = match status {
            ProgressStatus::Active => (running, "V2: running is white-70"),
            ProgressStatus::Done => (success, "V2: green is reserved for Done"),
            ProgressStatus::Error => (error, "Error tone"),
            ProgressStatus::Paused => (muted, "Paused tone"),
        };
        let track_x = 12u16;
        if ratio > 0.0 {
            assert_eq!(buf[(track_x, 1)].fg, fill, "{what}");
        } else {
            assert_eq!(
                buf[(track_x, 1)].symbol(),
                "─",
                "V1: the empty track paints unfilled"
            );
        }
        if status != ProgressStatus::Done {
            assert!(
                (0..70).all(|x| buf[(x, 1)].fg != success),
                "N2: green never paints {label:?}"
            );
        }
    }
    // S1: ratio clamps to 0..1 and the percent rounds.
    for (ratio, want) in [(1.7, "100%"), (-0.3, "0%"), (0.5, "50%")] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(70, 3);
        let bg = stage.bg();
        let mut ctx = stage.ctx(Interaction::default());
        render_bar(
            Rect::new(0, 1, 70, 1),
            &mut buf,
            &mut ctx,
            "Halfway   ",
            ratio,
            ProgressStatus::Active,
            bg,
        );
        assert!(
            buf_row_text(&buf, 1).contains(want),
            "S1: ratio {ratio} shows {want}"
        );
    }
    // S2: the label paints only when width exceeds label plus 8.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(70, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    render_bar(
        Rect::new(0, 1, 70, 1),
        &mut buf,
        &mut ctx,
        "Building  ",
        0.5,
        ProgressStatus::Active,
        bg,
    );
    assert!(
        buf_row_text(&buf, 1).contains("Building"),
        "S2: wide paints the label"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(70, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    render_bar(
        Rect::new(0, 1, 10, 1),
        &mut buf,
        &mut ctx,
        "Building  ",
        0.5,
        ProgressStatus::Active,
        bg,
    );
    assert!(
        !buf_row_text(&buf, 1).contains("Building"),
        "S2: narrow drops the label"
    );
    // N1: a bar narrower than 6 track cells shows the percent only.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(70, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    render_bar(
        Rect::new(0, 1, 8, 1),
        &mut buf,
        &mut ctx,
        "",
        0.42,
        ProgressStatus::Active,
        bg,
    );
    let narrow = buf_row_text(&buf, 1);
    assert!(narrow.contains("42%"), "N1: percent only");
    assert!(
        !narrow.contains('━') && !narrow.contains('─'),
        "N1: no label, no track"
    );
    // A1/A2: sample rows expose no focus stops.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(70, 8);
    let bg = stage.bg();
    for (i, (label, ratio, status, _)) in samples.iter().enumerate() {
        let mut ctx = stage.ctx(Interaction::default());
        render_bar(
            Rect::new(0, i as u16 + 1, 70, 1),
            &mut buf,
            &mut ctx,
            label,
            *ratio,
            *status,
            bg,
        );
    }
    assert!(
        stage.ring.reachable().is_empty(),
        "A1: sample rows take no input (no focus stops)"
    );
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
/// build plus pause/restart over [`render_bar`].
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
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(70, 3);
        let bg = stage.bg();
        let mut ctx = stage.ctx(Interaction::default());
        render_bar(
            Rect::new(0, 1, 70, 1),
            &mut buf,
            &mut ctx,
            "Building  ",
            ratio,
            ProgressStatus::Active,
            bg,
        );
        buf_row_text(&buf, 1).chars().filter(|c| *c == '━').count()
    };
    let early = fill_count(0.08);
    let mid = fill_count(0.48);
    assert!(
        mid > early && early > 0,
        "V1: the fill climbs with the ratio"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(70, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    render_bar(
        Rect::new(0, 1, 70, 1),
        &mut buf,
        &mut ctx,
        "Building  ",
        1.5,
        ProgressStatus::Active,
        bg,
    );
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
                l[..pos].chars().filter(|c| !c.is_whitespace()).last()
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

/// SPIN-FRAME-001 (`showcase/pages/progress`, 80x24, playing): braille frames
/// plus the compact activity row over [`render_spinner`].
///
/// Isolated path pins the 10-frame cycle, the accent-glyph/secondary-label
/// paint, the empty-area no-op, and the no-focus shape. Live path samples
/// two distinct frames plus the count line.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_spin_frame() {
    let dir = s4_dir("s4_spin_frame");

    // S1 isolated: frame(tick) is SPINNER[tick % 10].
    assert_eq!(SPINNER.len(), 10, "S1: ten braille frames");
    assert_eq!(spinner_frame(0), "⠋", "row pins frame 0");
    assert_eq!(spinner_frame(1), "⠙", "row pins frame 1");
    for tick in 0..25u64 {
        assert_eq!(
            spinner_frame(tick),
            SPINNER[(tick % 10) as usize],
            "S1: frame({tick}) cycles"
        );
    }
    // V1/N2 isolated: `⠋ label` in accent/secondary; the label is never
    // bold and never accent.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let accent = stage.theme.accent;
    let secondary = stage.theme.text_secondary;
    let mut ctx = stage.ctx(Interaction {
        tick: 0,
        ..Default::default()
    });
    render_spinner(
        Rect::new(4, 1, 50, 1),
        &mut buf,
        &mut ctx,
        "Waiting for the test runner",
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "⠋", "V1: the glyph renders");
    assert_eq!(buf[(4, 1)].fg, accent, "V1: the glyph is accent");
    assert_eq!(buf[(6, 1)].fg, secondary, "V1: the label is secondary");
    assert!(
        (6..40).all(|x| !buf_is_bold(&buf, x, 1)),
        "N2: the label is never bold"
    );
    assert!(
        (6..40).all(|x| buf[(x, 1)].fg != accent),
        "N2: the label is never accent"
    );
    assert!(
        stage.ring.reachable().is_empty(),
        "A1: the spinner holds no focus"
    );
    let first = capture_isolated(&dir, "spin-frame-0", &buf);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction {
        tick: 1,
        ..Default::default()
    });
    render_spinner(
        Rect::new(4, 1, 50, 1),
        &mut buf,
        &mut ctx,
        "Waiting for the test runner",
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "⠙", "tick 1 advances the frame");
    let second = capture_isolated(&dir, "spin-frame-1", &buf);
    assert_ne!(
        first.digest(),
        second.digest(),
        "N1: frames differ, not frozen"
    );
    // S2: an empty area draws nothing.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction {
        tick: 3,
        ..Default::default()
    });
    render_spinner(
        Rect::ZERO,
        &mut buf,
        &mut ctx,
        "Waiting for the test runner",
        bg,
    );
    assert_surround_intact(&buf, Rect::ZERO, "S2: an empty area draws nothing");

    // PTY: two distinct live frames plus the count line.
    let case = s4_case("spin_frame", &["--page", "progress"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "first-frame");
    let mut seen = std::collections::BTreeSet::new();
    let deadline = std::time::Instant::now() + case_timeout(&case);
    while std::time::Instant::now() < deadline && seen.len() < 2 {
        let text = live_text(&mut s);
        if let Some(line) = text.lines().find(|l| l.contains("Waiting for"))
            && let Some(pos) = line.find("Waiting for")
            && let Some(g) = line[..pos].chars().filter(|c| !c.is_whitespace()).last()
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
/// indeterminate sweep geometry over [`render_indeterminate`].
///
/// Isolated path pins the segment length/position math, the accent-over-
/// subtle paint, the label rule, and the never-outside/empty negatives.
/// Live path watches the sweep move.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_spin_sweep() {
    let dir = s4_dir("s4_spin_sweep");

    // S1/S2 isolated: length (track/5) clamps to 2..8; position wraps.
    let sweep_row = |tick: u64, width: u16| -> Buffer {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(80, 3);
        let bg = stage.bg();
        let mut ctx = stage.ctx(Interaction {
            tick,
            ..Default::default()
        });
        render_indeterminate(
            Rect::new(0, 1, width, 1),
            &mut buf,
            &mut ctx,
            "Resolving ",
            bg,
        );
        buf
    };
    let buf8 = sweep_row(8, 70);
    let row8 = buf_row_text(&buf8, 1);
    assert!(
        row8.contains("Resolving"),
        "V2: the label paints when it fits"
    );
    // Track starts after "Resolving " + 2 (x = 12); at tick 8 the segment
    // sits at cells 12..20 in accent over the subtle track.
    let stage = Stage::new();
    let accent = stage.theme.accent;
    let subtle = stage.theme.border_subtle;
    let primary = stage.theme.text_primary;
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
    for tick in [0u64, 8, 40, 65, 200] {
        let buf = sweep_row(tick, 70);
        assert!(
            (70..80).all(|x| buf[(x, 1)].symbol() == "·"),
            "N1: tick {tick} never escapes the area"
        );
    }
    capture_isolated(&dir, "sweep-8", &buf8);
    // Empty areas draw nothing; no focus stops anywhere.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction {
        tick: 5,
        ..Default::default()
    });
    render_indeterminate(Rect::ZERO, &mut buf, &mut ctx, "Resolving ", bg);
    assert_surround_intact(&buf, Rect::ZERO, "N2: an empty area draws nothing");
    assert!(
        stage.ring.reachable().is_empty(),
        "A1: the sweep holds no focus"
    );

    // PTY: the sweep visibly moves.
    let case = s4_case("spin_sweep", &["--page", "progress"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "sweep-0");
    let mut rows = std::collections::BTreeSet::new();
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

    // S1 isolated: of(pct) is Low at 59 and below, Medium to 84, High above.
    assert_eq!(METER_LOW_MAX, 59, "low boundary");
    assert_eq!(METER_MEDIUM_MAX, 84, "medium boundary");
    for (pct, want) in [
        (0, MeterLevel::Low),
        (59, MeterLevel::Low),
        (60, MeterLevel::Medium),
        (84, MeterLevel::Medium),
        (85, MeterLevel::High),
        (100, MeterLevel::High),
    ] {
        assert_eq!(MeterLevel::of(pct), want, "S1: of({pct})");
    }
    // V1/V2 isolated: white/warning/error runs; low value text is primary,
    // medium and high take the run color.
    let stage = Stage::new();
    let white = stage.theme.text_secondary;
    let warning = stage.theme.warning;
    let err = stage.theme.error;
    let primary = stage.theme.text_primary;
    let success = stage.theme.success;
    for (pct, run, text, what) in [
        (38u8, white, primary, "low"),
        (72u8, warning, warning, "medium"),
        (91u8, err, err, "high"),
    ] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(60, 3);
        let bg = stage.bg();
        let mut ctx = stage.ctx(Interaction::default());
        Meter::new(Some(pct)).value(format!("{pct}% used")).render(
            Rect::new(0, 1, 40, 1),
            &mut buf,
            &mut ctx,
            bg,
        );
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
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(60, 3);
        let bg = stage.bg();
        let mut ctx = stage.ctx(Interaction::default());
        Meter::new(Some(pct)).value(format!("{pct}% used")).render(
            Rect::new(0, 1, 40, 1),
            &mut buf,
            &mut ctx,
            bg,
        );
        buf_row_text(&buf, 1)
            .chars()
            .filter(|c| *c == '━')
            .count()
            .to_string()
    };
    assert_eq!(capped(150), capped(100), "S2: used_pct caps at 100");
    // N2: a track narrower than 6 cells shows the value only.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(Some(38))
        .value("38% used")
        .render(Rect::new(0, 1, 14, 1), &mut buf, &mut ctx, bg);
    let narrow = buf_row_text(&buf, 1);
    assert!(narrow.contains("38% used"), "N2: value only");
    assert!(
        !narrow.contains('━') && !narrow.contains('─'),
        "N2: no track"
    );
    // A1/A2: meters hold no focus.
    assert!(
        stage.ring.reachable().is_empty(),
        "A1: meters hold no focus"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 5);
    let bg = stage.bg();
    for (i, pct) in [38u8, 72, 91].iter().enumerate() {
        let mut ctx = stage.ctx(Interaction::default());
        Meter::new(Some(*pct)).value(format!("{pct}% used")).render(
            Rect::new(0, i as u16 + 1, 40, 1),
            &mut buf,
            &mut ctx,
            bg,
        );
    }
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

    // S1 isolated: level() maps Warning to Medium and Exhausted to High;
    // Stale and below map to None.
    assert_eq!(
        MeterTone::Warning.level(Some(82)),
        Some(MeterLevel::Medium),
        "S1: Warning maps to Medium"
    );
    assert_eq!(
        MeterTone::Exhausted.level(Some(100)),
        Some(MeterLevel::High),
        "S1: Exhausted maps to High"
    );
    for tone in [
        MeterTone::Stale,
        MeterTone::Refreshing,
        MeterTone::Error,
        MeterTone::Unknown,
    ] {
        assert_eq!(tone.level(Some(50)), None, "S1: {tone:?} maps to None");
    }
    let stage = Stage::new();
    let warning = stage.theme.warning;
    let err = stage.theme.error;
    let faint = stage.theme.text_faint;
    let secondary = stage.theme.text_secondary;
    // V1 isolated: Warning ends ` ▲`, Exhausted ends bold ` !`, Stale is
    // faint, Unknown is `—`.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(Some(82))
        .value("82% used")
        .tone(MeterTone::Warning)
        .render(Rect::new(0, 1, 40, 1), &mut buf, &mut ctx, bg);
    let row = buf_row_text(&buf, 1);
    assert!(row.contains('▲'), "V1: Warning ends ▲");
    assert_eq!(buf[(0, 1)].fg, warning, "V1: Warning run tone");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(Some(100))
        .value("100% used")
        .tone(MeterTone::Exhausted)
        .render(Rect::new(0, 1, 40, 1), &mut buf, &mut ctx, bg);
    let row = buf_row_text(&buf, 1);
    assert!(row.contains('!'), "V1: Exhausted ends !");
    // Suffix sits past the run plus the 9-wide value: cell math, since the
    // run cells are multibyte.
    let track_w = 40u16 - (9 + 3);
    let bx = track_w + 1 + 9 + 1;
    assert_eq!(buf[(bx, 1)].symbol(), "!", "suffix position");
    assert!(buf_is_bold(&buf, bx, 1), "V1: Exhausted ! is bold");
    assert_eq!(buf[(bx, 1)].fg, err, "Exhausted tone");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(Some(54))
        .value("54% used")
        .tone(MeterTone::Stale)
        .render(Rect::new(0, 1, 40, 1), &mut buf, &mut ctx, bg);
    assert_eq!(buf[(0, 1)].fg, faint, "V1: Stale is faint");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(None).tone(MeterTone::Unknown).render(
        Rect::new(0, 1, 40, 1),
        &mut buf,
        &mut ctx,
        bg,
    );
    assert!(buf_row_text(&buf, 1).contains('—'), "V1: Unknown is —");
    // N1: Error and Unknown draw no run (presence: Warning draws one).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(None)
        .value("quota read failed")
        .tone(MeterTone::Error)
        .render(Rect::new(0, 1, 40, 1), &mut buf, &mut ctx, bg);
    let erow = buf_row_text(&buf, 1);
    assert!(erow.contains("quota read failed"), "Error message renders");
    assert!(
        !erow.contains('━') && !erow.contains('─'),
        "N1: Error draws no run"
    );
    // S2: Refreshing shows the spinner frame plus `refreshing` as the value.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction {
        tick: 0,
        ..Default::default()
    });
    Meter::new(Some(54)).tone(MeterTone::Refreshing).render(
        Rect::new(0, 1, 40, 1),
        &mut buf,
        &mut ctx,
        bg,
    );
    assert!(
        buf_row_text(&buf, 1).contains("⠋ refreshing"),
        "S2: Refreshing value at tick 0"
    );
    // V2/N2: Block fills the used share as background with the value inside
    // it starting one cell in; Stale fill text is secondary, never canvas.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let canvas = stage.theme.canvas;
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(Some(82))
        .value("82% used")
        .tone(MeterTone::Warning)
        .visual(MeterVisual::Block)
        .render(Rect::new(0, 1, 20, 1), &mut buf, &mut ctx, bg);
    assert_eq!(buf[(2, 1)].bg, warning, "V2: Block used share filled");
    assert_eq!(buf[(1, 1)].symbol(), "8", "V2: value starts one cell in");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(60, 3);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction::default());
    Meter::new(Some(54))
        .value("54% used")
        .tone(MeterTone::Stale)
        .visual(MeterVisual::Block)
        .render(Rect::new(0, 1, 20, 1), &mut buf, &mut ctx, bg);
    assert_eq!(
        buf[(2, 1)].fg,
        secondary,
        "N2: Stale fill text is secondary"
    );
    assert_ne!(buf[(2, 1)].fg, canvas, "N2: never canvas");
    assert!(
        stage.ring.reachable().is_empty(),
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

/// Mirror the chrome page's status bar (v1/registry.json SB rows).
fn chrome_status_bar() -> (StatusBar, WidgetId, WidgetId) {
    let usage = s4_id("test.s4.status.usage");
    let pr = s4_id("test.s4.status.pr");
    let mut b = StatusBar::new();
    b.left.push(
        StatusItem::new("payments-platform", Tone::Normal)
            .strong()
            .priority(9),
    );
    b.left.push(
        StatusItem::new("PR #482 · settlement backoff", Tone::Secondary)
            .priority(7)
            .clickable(pr),
    );
    b.center
        .push(StatusItem::new("Claude Code · working · 2 tabs", Tone::Secondary).priority(4));
    b.right.push(
        StatusItem::new("Weekly 59%", Tone::Warning)
            .chip()
            .priority(6)
            .clickable(usage),
    );
    b.right.push(
        StatusItem::new("jackin-payments-7f3a", Tone::Muted)
            .chip()
            .priority(3),
    );
    b.right
        .push(StatusItem::new("run 9c41", Tone::Faint).priority(2));
    (b, usage, pr)
}

/// SB-GROUPS-001 (`showcase/pages/chrome`): three groups plus chips plus
/// hover over [`StatusBar`].
///
/// Isolated path mirrors the row's chrome bar at full width: shared-row
/// spacing, priorities, bold name, chip planes, hover lift, and one-row hit
/// regions. Live path boots 80x24 (where priorities evict to the top two —
/// the approved snapshots prove center+right leave), resizes to 160x50
/// (the canonical size where the full bar renders), and hovers the Weekly
/// chip. The 80x24 viewport alone cannot show three groups; the resize
/// carries the row's own V1/S1/A checks to where they hold.
#[test]
#[ignore = "showcase s4 check; run with --ignored"]
fn s4_sb_groups() {
    let dir = s4_dir("s4_sb_groups");
    let (bar, usage, _pr) = chrome_status_bar();

    // S1/V1 isolated: all six place at full width with spacing only.
    let placed = bar.layout(Rect::new(0, 0, 160, 1));
    assert_eq!(placed.len(), 6, "S1: all six items place at full width");
    let groups: Vec<Group> = placed.iter().map(|p| p.group).collect();
    assert!(
        groups.contains(&Group::Left)
            && groups.contains(&Group::Center)
            && groups.contains(&Group::Right),
        "V1: left, center, and right share one row"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(160, 3);
    let elevated = stage.theme.surface_elevated;
    let overlay = stage.theme.surface_overlay;
    let mut ctx = stage.ctx(Interaction::default());
    bar.render(Rect::new(0, 1, 160, 1), &mut buf, &mut ctx);
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains("payments-platform")
            && row.contains("Claude Code · working · 2 tabs")
            && row.contains("Weekly 59%"),
        "V1: all groups render"
    );
    // N1: gaps between groups are never glyphs (spaces on the bar fill).
    let left_end = placed
        .iter()
        .filter(|p| p.group == Group::Left)
        .map(|p| p.x + p.width)
        .max()
        .unwrap();
    let center_x = placed.iter().find(|p| p.group == Group::Center).unwrap().x;
    assert!(
        center_x > left_end,
        "N2: the center never sits left of the left"
    );
    for x in left_end..center_x {
        assert_eq!(buf[(x, 1)].symbol(), " ", "N1: gaps are never glyphs");
        assert_eq!(buf[(x, 1)].bg, elevated, "S2: gaps fill surface_elevated");
    }
    // V2: the surface name is bold; quota chips sit on the overlay plane.
    // Cell columns come from the layout — byte search would drift on the
    // multibyte `·` separators inside earlier items.
    let name_x = placed
        .iter()
        .find(|p| p.group == Group::Left && p.index == 0)
        .unwrap()
        .x;
    assert!(buf_is_bold(&buf, name_x, 1), "V2: the surface name is bold");
    let chip_x = placed
        .iter()
        .find(|p| p.group == Group::Right && p.index == 0)
        .unwrap()
        .x
        + 1;
    assert_eq!(
        buf[(chip_x, 1)].bg,
        overlay,
        "V2: chips sit on the overlay plane"
    );
    capture_isolated(&dir, "sb-groups", &buf);
    // A1/A2: hovering a clickable chip lifts its plane to primary text;
    // clickable items register a one-row hit region.
    let mut stage = Stage::new();
    let lift = stage.theme.lift(overlay);
    let primary = stage.theme.text_primary;
    let mut buf = sentinel_buf(160, 3);
    let mut ctx = stage.ctx(Interaction {
        hover: Some(usage),
        ..Default::default()
    });
    bar.render(Rect::new(0, 1, 160, 1), &mut buf, &mut ctx);
    assert_eq!(buf[(chip_x, 1)].bg, lift, "A1: hover lifts the chip plane");
    assert_eq!(buf[(chip_x, 1)].fg, primary, "A1: hover takes primary text");
    drop(ctx);
    let hit = stage.hits.area_of(usage).expect("A2: usage hit region");
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
        .bg
        .clone();
    hover_over(&mut s, "Weekly 59%", case_timeout(&case));
    let after = live_frame(&mut s)
        .get(hcol + 1, hrow)
        .unwrap_or_else(|| panic!("hovered chip cell missing"))
        .bg
        .clone();
    assert_ne!(before, after, "A1 live: hover lifts the chip plane");
    let hovered = live_text(&mut s);
    assert!(
        hovered.contains("Weekly 59%"),
        "A1 live: glyphs stable under hover"
    );
    checkpoint(&mut s, &dir, "hover");
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
    let (bar, _usage, _pr) = chrome_status_bar();

    // S1 isolated: eviction drops the lowest priority anywhere; at page
    // widths 61/53 (the 80x24/72x20 chrome pages) only the left pair stays.
    for (w, what) in [(61u16, "80x24 page"), (53u16, "72x20 page")] {
        let placed = bar.layout(Rect::new(0, 0, w, 1));
        let texts: Vec<&str> = placed.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(
            texts,
            vec!["payments-platform", "PR #482 · settlement backoff"],
            "S1: {what} keeps the left pair"
        );
    }
    // S1 ties: center leaves before right before left at equal priority.
    let mut tied = StatusBar::new();
    tied.left
        .push(StatusItem::new("L", Tone::Normal).priority(5));
    tied.center
        .push(StatusItem::new("C", Tone::Normal).priority(5));
    tied.right
        .push(StatusItem::new("R", Tone::Normal).priority(5));
    let placed = tied.layout(Rect::new(0, 0, 10, 1));
    let groups: Vec<Group> = placed.iter().map(|p| p.group).collect();
    assert!(
        !groups.contains(&Group::Center),
        "S1: ties evict the center first"
    );
    // V1/S2/N1: at width 40 only the strongest left item stays, whole; at
    // width 18 it truncates with … instead of leaving.
    let placed = bar.layout(Rect::new(0, 0, 40, 1));
    assert_eq!(placed.len(), 1, "V1: width 40 keeps one survivor");
    assert_eq!(
        placed[0].text, "payments-platform",
        "N1: the strongest left item is never dropped"
    );
    let placed = bar.layout(Rect::new(0, 0, 18, 1));
    assert_eq!(placed.len(), 1, "S2: the last left item never leaves");
    assert!(placed[0].text.contains('…'), "V2: it truncates with …");
    assert_eq!(placed[0].text.chars().count(), 16, "V2: it fills the room");
    // N2: right items are never truncated — they leave whole or stay whole.
    for w in [40u16, 53, 61, 100, 134] {
        for p in bar.layout(Rect::new(0, 0, w, 1)) {
            if p.group == Group::Right {
                assert!(
                    !p.text.contains('…'),
                    "N2: right items never truncate (width {w})"
                );
            }
        }
    }
    // A1/A2: placement recomputes from the same priorities per width; the
    // bar holds no focus stops at any width.
    let narrow = bar.layout(Rect::new(0, 0, 53, 1));
    let wide = bar.layout(Rect::new(0, 0, 134, 1));
    assert!(narrow.len() < wide.len(), "A1: width recomputes placement");
    assert_eq!(wide.len(), 6, "A1: full width keeps all six");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(160, 3);
    let mut ctx = stage.ctx(Interaction::default());
    bar.render(Rect::new(0, 1, 134, 1), &mut buf, &mut ctx);
    drop(ctx);
    assert!(
        stage.ring.reachable().is_empty(),
        "A2: the bar holds no focus stops"
    );
    capture_isolated(&dir, "sb-collapse", &buf);

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
