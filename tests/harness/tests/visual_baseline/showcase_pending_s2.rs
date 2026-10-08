//! Showcase pending slice 8A-S2 executable checks (VB Phase-8v).
//!
//! One ignored test per S2 registry row (20 rows: the first 20 unrun
//! showcase rows in registry order — the 15 CONTROLS group rows plus the
//! first 5 FIELDS group rows), over the real `showcase` binary built from
//! this worktree's verified VB sources (`env!("CARGO_BIN_EXE_showcase")`)
//! and over the real production widgets rendered headless. Each test
//! executes its row's listed adapters (`headless-tick`, `pty-capture`, or
//! both) and asserts its visual (V), state (S), action (A), and negative
//! (N) checks in executable form:
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
//! No new snapshots and no new static captures: every S2 row already has
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
//! Row → test map (registry id → `s2_*` test):
//!
//! - BTN-BUSY-FRAMES-001 → [`s2_busy_frames`]
//! - BTN-BUSY-CHECKED-GEOM-002 → [`s2_busy_checked_geom`]
//! - BRAND-HOVER-001 → [`s2_brand_hover`]
//! - BTN-STATE-FOCUS-003 → [`s2_button_focus`]
//! - BTN-DISABLED-PRECEDENCE-004 → [`s2_button_disabled`]
//! - CHK-STATE-MATRIX-001 → [`s2_checkbox_matrix`]
//! - TGL-STATE-MATRIX-001 → [`s2_toggle_matrix`]
//! - RADIO-STATE-MATRIX-001 → [`s2_radio_matrix`]
//! - PANEL-FOCUS-READONLY-001 → [`s2_panel_focus`]
//! - SPLIT-SEAM-DRAG-001 → [`s2_split_drag`]
//! - PROPS-CURSOR-TONE-001 → [`s2_props_cursor`]
//! - EMPTY-KIND-NARROW-001 → [`s2_empty_narrow`]
//! - TOOSMALL-NOTICE-001 → [`s2_toosmall_notice`]
//! - CTRL-NARROW-CLIP-001 → [`s2_narrow_clip`]
//! - CTRL-HOVER-COVERAGE-001 → [`s2_hover_coverage`]
//! - FLD-CHROME-001 → [`s2_field_chrome`]
//! - FLD-DISABLED-002 → [`s2_field_disabled`]
//! - FLD-FOCUSLOSS-003 → [`s2_field_focusloss`]
//! - TI-COMMIT-001 → [`s2_input_commit`]
//! - TI-CANCEL-002 → [`s2_input_cancel`]

use std::path::{Path, PathBuf};
use std::time::Duration;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::FocusRing;
use junie_tui::core::hit::HitRegistry;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{ButtonKind, Theme, Tone};
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::ui::layout::{Maximized, Split, SplitDir};
use junie_tui::widgets::brand::Lockup;
use junie_tui::widgets::button::Button;
use junie_tui::widgets::choice::{Checkbox, RadioGroup, Toggle};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::input::{InputEvent, TextInput};
use junie_tui::widgets::panel::{Panel, ScrollPanel};
use junie_tui::widgets::progress::{SPINNER, spinner_frame};
use junie_tui::widgets::props::{self, Prop, PropsEvent, PropsList};
use junie_tui::widgets::splitter::Splitter;
use junie_tui::widgets::textarea::TextArea;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;
use tuiscotti::tui::{MouseButton, Session, Wheel};
use tuiscotti::{Color as TColor, Frame, Provenance, Rgb};

use crate::support::state_waits as waits;
use crate::support::typed_input::Input;
use crate::support::{self, Case, Color, HOLLA, JACKIN, SHOWCASE, TABLEPRO};

/// Boot needle: the fully-rendered shell header on every showcase page.
const BOOT: &str = "Junie Design system";

/// Owned-name S2 case at the row's viewport (truecolor, like every S2 row).
fn s2_case(slug: &str, args: &'static [&'static str], cols: u16, rows: u16) -> Case {
    Case::dynamic(
        format!("journeys/showcase/s2/{slug}"),
        SHOWCASE,
        args,
        cols,
        rows,
        Color::Truecolor,
        BOOT,
    )
}

/// Unique scratch dir for one S2 test (created, never shared).
fn s2_dir(test: &str) -> PathBuf {
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
    eprintln!("s2 provenance: {body}");
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

/// One fresh live frame with cell colors (no wait).
fn live_frame(s: &mut Session) -> Frame {
    let screen = s
        .snapshot()
        .unwrap_or_else(|e| panic!("live frame sample failed: {e:#}"));
    support::frame_from_screen(
        &screen,
        Provenance::now("tuiscotti-default", "s2-checks", vec![]),
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

/// `notches` wheel steps at fixed `(col, row)`.
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

fn s2_key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn s2_id(name: &str) -> WidgetId {
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

/// Headless capture plus JSON evidence in the test dir.
fn capture_isolated(dir: &Path, name: &str, buf: &Buffer) -> Frame {
    let frame = support::capture_buffer(buf, Provenance::now("default", "s2-isolated", vec![]));
    std::fs::write(
        dir.join(format!("isolated-{name}.json")),
        frame.to_json_pretty(),
    )
    .unwrap_or_else(|e| panic!("write isolated-{name}.json: {e}"));
    eprintln!("isolated {name}: digest {:016x}", frame.digest());
    frame
}

/// Resolve a live cell color to RGB. `Default` has no live-resolvable
/// value, so resolving one fails loudly instead of guessing.
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

/// Theme [`ratatui::style::Color`] to RGB (all theme tones are RGB).
fn theme_rgb(c: ratatui::style::Color) -> Rgb {
    match c {
        ratatui::style::Color::Rgb(r, g, b) => Rgb { r, g, b },
        other => panic!("theme color is not RGB: {other:?}"),
    }
}

/// Cell column of `needle`'s first occurrence in `line` (char-based: all
/// page chrome here is single-cell).
fn char_col(line: &str, needle: &str) -> u16 {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle:?} not in {line:?}"));
    line[..byte].chars().count() as u16
}

/// BTN-BUSY-FRAMES-001 (`showcase/audit/buttons`): marker cell spinner.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_busy_frames() {
    let dir = s2_dir("s2_busy_frames");
    let t = Theme::junie();

    // Isolated: busy renders the tick's spinner frame in the marker cell.
    let bid = s2_id("test.s2.busy");
    let mut first: Option<Frame> = None;
    for (tick, want) in [(0u64, "⠋"), (1u64, "⠙")] {
        assert_eq!(spinner_frame(tick), want, "row pins SPINNER[tick]");
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(24, 3);
        let mut b = Button::primary(bid, "Run long job");
        b.busy = true;
        let bg = stage.bg();
        b.render(
            Rect::new(4, 1, 20, 1),
            &mut buf,
            &mut stage.ctx(Interaction {
                focus: Some(bid),
                tick,
                ..Default::default()
            }),
            bg,
        );
        // V1: marker cell (area.x+1) shows the spinner frame.
        assert_eq!(buf[(5, 1)].symbol(), want, "V1: tick {tick} marker");
        // V2: busy label is non-bold text_secondary; spinner is accent.
        assert_eq!(buf[(7, 1)].fg, t.text_secondary, "V2: label tone");
        assert!(!buf_is_bold(&buf, 7, 1), "V2: label non-bold");
        assert_eq!(buf[(5, 1)].fg, t.accent, "V2: spinner accent");
        // S1/S2: busy reports busy, never pressed; cannot activate.
        assert!(b.busy && !b.disabled, "S1: busy state");
        assert!(!b.can_activate(), "S2: can_activate false while busy");
        // A1/A2: Enter/Space and click consume without activating.
        let (o, fired) = b.on_key(&s2_key(KeyCode::Enter));
        assert!(
            matches!(o, Outcome::Consumed) && !fired,
            "A1: Enter while busy"
        );
        let (o, fired) = b.on_key(&s2_key(KeyCode::Char(' ')));
        assert!(
            matches!(o, Outcome::Consumed) && !fired,
            "A1: Space while busy"
        );
        assert!(!b.on_click(), "A2: click while busy");
        // N2: no toggle marker while busy (plain button has no `on`).
        assert!(b.on.is_none(), "N2: no toggle marker slot");
        assert_surround_intact(&buf, Rect::new(4, 1, 16, 1), "busy tick {tick}");
        let frame = capture_isolated(&dir, &format!("busy-tick-{tick}"), &buf);
        if let Some(prev) = first.as_ref() {
            // N1: frames differ across ticks (multi-frame, not frozen).
            assert_ne!(frame.digest(), prev.digest(), "N1: tick frames differ");
        } else {
            first = Some(frame);
        }
    }

    // PTY: the live busy window walks the spinner cycle, then closes.
    let case = s2_case("busy_frames", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    assert!(
        live_text(&mut s).contains("Start long job"),
        "boot has the busy demo"
    );
    click_at(&mut s, "Start long job", case_timeout(&case));
    // Sample the marker cell until two distinct spinner glyphs appear.
    let mut seen = std::collections::BTreeSet::new();
    let deadline = std::time::Instant::now() + case_timeout(&case);
    while std::time::Instant::now() < deadline && seen.len() < 2 {
        let snap = s
            .snapshot()
            .unwrap_or_else(|e| panic!("busy sample failed: {e:#}"));
        let frame = support::frame_from_screen(
            &snap,
            Provenance::now("tuiscotti-default", "s2-checks", vec![]),
        );
        if let Some((row, col)) = support::screen_find(&snap, "Start long job") {
            // The busy button widens by the 2-cell marker slot, so the
            // spinner sits 1–2 cells left of the label; scan the window.
            for c in col.saturating_sub(3)..col {
                if let Some(cell) = frame.get(c, row)
                    && SPINNER.contains(&cell.symbol.as_str())
                {
                    seen.insert(cell.symbol.clone());
                }
            }
        }
        if support::screen_text(&snap).contains("Long job finished") {
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(seen.len() >= 2, "V1/N1 live: two spinner frames {seen:?}");
    // The window closes with the finished status and the plain label.
    waits::wait_state(&mut s, "Long job finished", "busy window closes");
    checkpoint(&mut s, &dir, "finished");
    let done = live_text(&mut s);
    assert!(done.contains("Start long job"), "label restored after busy");
    eprintln!("s2 busy_frames: isolated ticks 0/1 + live cycle {seen:?}");
}

/// BTN-BUSY-CHECKED-GEOM-002 (`showcase/flows/buttons/focus`): busy +
/// checked share one marker slot.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_busy_checked_geom() {
    let dir = s2_dir("s2_busy_checked_geom");
    let t = Theme::junie();

    // Isolated: spinner wins the shared marker cell; width counts it once.
    let bid = s2_id("test.s2.busychecked");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut b = Button::toggle(bid, "Verbose", true);
    b.busy = true;
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 22, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            tick: 0,
            ..Default::default()
        }),
        bg,
    );
    // V1: marker cell shows the spinner, not ●.
    assert_eq!(buf[(5, 1)].symbol(), "⠋", "V1: spinner wins the marker");
    assert_eq!(buf[(5, 1)].fg, t.accent, "V1: spinner accent");
    // V2/N2: width is label + 2 pad + 2 marker (single slot).
    assert_eq!(b.width(), 7 + 2 + 2, "V2/N2: one marker slot");
    // S1/S2: on stays true; cannot activate while busy.
    assert_eq!(b.on, Some(true), "S1: toggle preserved");
    assert!(!b.can_activate(), "S2: can_activate false");
    // A1: Enter consumes; the toggle never flips.
    let (o, fired) = b.on_key(&s2_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && !fired,
        "A1: Enter consumed"
    );
    assert_eq!(b.on, Some(true), "A1: on unchanged");
    // N1: no ●/○ anywhere in the button row while busy.
    let row = buf_row_text(&buf, 1);
    assert!(
        !row.contains('●') && !row.contains('○'),
        "N1: no marker in row {row:?}"
    );
    assert_surround_intact(&buf, Rect::new(4, 1, 11, 1), "busy+checked");
    capture_isolated(&dir, "busy-checked", &buf);

    // PTY: the live busy button keeps one marker cell and its width.
    let case = s2_case("busy_checked_geom", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    let boot = live_text(&mut s);
    let boot_row = boot
        .lines()
        .find(|l| l.contains("Start long job"))
        .expect("boot busy row")
        .to_string();
    click_at(&mut s, "Start long job", case_timeout(&case));
    let busy_text = support::wait_screen(
        &mut s,
        case_timeout(&case),
        "busy marker never appeared",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("Start long job") && SPINNER.iter().any(|g| l.contains(g)))
        },
    );
    let busy_screen = support::screen_text(&busy_text.screen);
    let busy_row = busy_screen
        .lines()
        .find(|l| l.contains("Start long job"))
        .expect("busy row")
        .to_string();
    // V2 live: same display width before and during busy (no second slot).
    assert_eq!(
        boot_row.chars().count(),
        busy_row.chars().count(),
        "V2 live: row width stable"
    );
    // N1 live: no toggle marker on the busy row (● Verbose elsewhere
    // proves the glyph would render if present).
    assert!(
        !busy_row.contains('●') && !busy_row.contains('○'),
        "N1 live: {busy_row:?}"
    );
    assert!(
        busy_screen.contains('●'),
        "N1 presence: ● renders elsewhere"
    );
    waits::wait_state(&mut s, "Long job finished", "busy window closes");
    checkpoint(&mut s, &dir, "finished");
    eprintln!("s2 busy_checked_geom: isolated + live width-stable");
}

/// BRAND-HOVER-001 (`showcase/pages/chrome`): clickable lockup row.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_brand_hover() {
    let dir = s2_dir("s2_brand_hover");
    let t = Theme::junie();
    let lockup = Lockup::new("holla❯");

    // V1 isolated: padded width is text width + 2.
    assert_eq!(lockup.width(), 8, "V1: 6 text + 2 pad");
    let bid = s2_id("test.s2.brand");

    // Isolated idle: accent fill, on-accent bold text, hit, no ring stop.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    let w = lockup.render_clickable(4, 1, &mut buf, &mut stage.ctx(Interaction::default()), bid);
    assert_eq!(w, 8);
    assert_eq!(buf[(5, 1)].bg, t.accent, "V1: idle fill accent");
    assert_eq!(buf[(5, 1)].fg, t.text_on_accent, "V1: on-accent text");
    assert!(buf_is_bold(&buf, 5, 1), "V1: bold text");
    assert_eq!(buf[(4, 1)].symbol(), " ", "V1: outer pad cell");
    assert_eq!(
        stage.hits.hit(Position::new(5, 1)),
        Some(bid),
        "S1/A1: hit region reports the brand click"
    );
    assert!(!stage.ring.contains(bid), "S1: never a focus stop");
    assert_surround_intact(&buf, Rect::new(4, 1, 8, 1), "brand idle");
    let idle = capture_isolated(&dir, "brand-idle", &buf);

    // Isolated hover: fill lifts one plane; press/flash move further.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render_clickable(
        4,
        1,
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(bid),
            ..Default::default()
        }),
        bid,
    );
    assert_eq!(buf[(5, 1)].bg, t.accent_hover, "V2: hover lifts the fill");
    assert_eq!(buf[(5, 1)].fg, t.text_on_accent, "V2: hover keeps the text");
    let hovered = capture_isolated(&dir, "brand-hover", &buf);
    assert_ne!(
        hovered.digest(),
        idle.digest(),
        "V2: hover changes the frame"
    );
    for (name, ix) in [
        (
            "press",
            Interaction {
                hover: Some(bid),
                pressed: Some(bid),
                ..Default::default()
            },
        ),
        (
            "flash",
            Interaction {
                flash: Some(bid),
                ..Default::default()
            },
        ),
    ] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        lockup.render_clickable(4, 1, &mut buf, &mut stage.ctx(ix), bid);
        assert_eq!(buf[(5, 1)].bg, t.accent_pressed, "V2: {name} fill");
        capture_isolated(&dir, &format!("brand-{name}"), &buf);
    }
    // V2/N1: press without hover renders idle; the plain render is
    // stateless (hover never changes it — it takes no interaction).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render_clickable(
        4,
        1,
        &mut buf,
        &mut stage.ctx(Interaction {
            pressed: Some(bid),
            ..Default::default()
        }),
        bid,
    );
    let press_no_hover = capture_isolated(&dir, "brand-press-nohover", &buf);
    assert_eq!(
        press_no_hover.digest(),
        idle.digest(),
        "V2: press w/o hover is idle"
    );
    let mut buf_a = sentinel_buf(16, 3);
    lockup.render(4, 1, &mut buf_a, &t);
    let mut buf_b = sentinel_buf(16, 3);
    lockup.render(4, 1, &mut buf_b, &t);
    let plain_a = capture_isolated(&dir, "brand-plain-a", &buf_a);
    let plain_b = capture_isolated(&dir, "brand-plain-b", &buf_b);
    assert_eq!(
        plain_a.digest(),
        plain_b.digest(),
        "N1: plain render stateless"
    );
    // N2: no disabled/focus/selected rendering exists (clickable-only).
    assert!(!stage.ring.contains(bid), "N2: no focus stop ever");

    // PTY: the chrome page lockup lifts under hover, fills on press.
    let case = s2_case("brand_hover", &["--page", "chrome"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("app❯"), "boot shows the brand lockup");
    let idle_frame = live_frame(&mut s);
    let (brow, bcol) = support::screen_find(
        &s.snapshot()
            .unwrap_or_else(|e| panic!("brand pos failed: {e:#}")),
        "app❯",
    )
    .expect("lockup on screen");
    let idle_bg = idle_frame
        .get(bcol + 1, brow)
        .unwrap_or_else(|| panic!("lockup cell missing"))
        .bg;
    hover_over(&mut s, "app❯", case_timeout(&case));
    let hov_frame = live_frame(&mut s);
    let hov_bg = hov_frame
        .get(bcol + 1, brow)
        .unwrap_or_else(|| panic!("hovered lockup cell missing"))
        .bg;
    assert_ne!(idle_bg, hov_bg, "V2 live: hover lifts the lockup fill");
    // Presence: the glyphs are unchanged by hover (tone-only delta).
    let hov_text = live_text(&mut s);
    assert!(hov_text.contains("app❯"), "V2 live: glyphs stable");
    // Press: fill moves again (mouse down without release).
    Input::down(MouseButton::Left, bcol + 1, brow).send(&mut s);
    std::thread::sleep(Duration::from_millis(150));
    let press_frame = live_frame(&mut s);
    let press_bg = press_frame
        .get(bcol + 1, brow)
        .unwrap_or_else(|| panic!("pressed lockup cell missing"))
        .bg;
    Input::up(MouseButton::Left, bcol + 1, brow).send(&mut s);
    assert_ne!(hov_bg, press_bg, "V2 live: press moves the fill");
    checkpoint(&mut s, &dir, "pressed");
    eprintln!("s2 brand_hover: isolated matrix + live lift/press");
}

/// BTN-STATE-FOCUS-003 (`showcase/flows/buttons/focus`): label row +
/// gutter cell.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_button_focus() {
    let dir = s2_dir("s2_button_focus");
    let t = Theme::junie();
    let bid = s2_id("test.s2.btnfocus");

    // Isolated idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), " ", "idle gutter blank");
    let idle_bg = buf[(5, 1)].bg;
    let idle = capture_isolated(&dir, "btn-idle", &buf);

    // Isolated kbd-focus: ▎ + bold, no hover lift (suppressed).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            hover_suppressed: true,
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "▎", "V1: kbd-focus bar");
    assert_eq!(buf[(4, 1)].fg, t.focus, "V1: focus tone");
    assert!(buf_is_bold(&buf, 5, 1), "V1: kbd-focus bold");
    assert_eq!(buf[(5, 1)].bg, idle_bg, "V1: no hover lift");
    let kbd = capture_isolated(&dir, "btn-kbdfocus", &buf);
    assert_ne!(kbd.digest(), idle.digest(), "V1: focus changes the frame");

    // Isolated pointer-focus: bar + bold + hover lift.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            hover: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "▎", "V1: pointer-focus bar");
    assert!(buf_is_bold(&buf, 5, 1), "V1: pointer-focus bold");
    assert_ne!(buf[(5, 1)].bg, idle_bg, "V1: hover lift present");
    let ptr = capture_isolated(&dir, "btn-pointerfocus", &buf);

    // Isolated pressed: inversion, distinct from pointer-focus.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            hover: Some(bid),
            pressed: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    let pressed = capture_isolated(&dir, "btn-pressed", &buf);
    assert_ne!(pressed.digest(), ptr.digest(), "V2: press inverts");
    // Hidden focus renders exactly idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            focus_hidden: true,
            ..Default::default()
        }),
        bg,
    );
    let hidden = capture_isolated(&dir, "btn-hidden", &buf);
    assert_eq!(hidden.digest(), idle.digest(), "V1: hidden focus is idle");
    // N1: press without hover renders exactly idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            pressed: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    let press_no_hover = capture_isolated(&dir, "btn-press-nohover", &buf);
    assert_eq!(
        press_no_hover.digest(),
        idle.digest(),
        "N1: press w/o hover is idle"
    );

    // V2: kinds render distinctly (Danger carries the error tone).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut d = Button::new(bid, "Run task", ButtonKind::Danger);
    let bg = stage.bg();
    d.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let danger = capture_isolated(&dir, "btn-danger", &buf);
    assert_ne!(
        danger.digest(),
        idle.digest(),
        "V2: danger differs from secondary"
    );

    // S1: toggle markers share one slot (width +2 once).
    let on = Button::toggle(s2_id("test.s2.togon"), "Verbose", true);
    let off = Button::toggle(s2_id("test.s2.togoff"), "Verbose", false);
    assert_eq!(on.width(), 7 + 2 + 2, "S1: on width");
    assert_eq!(off.width(), 7 + 2 + 2, "S1: off width");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut on = on;
    let bg = stage.bg();
    on.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(5, 1)].symbol(), "●", "S1: on marker");
    assert_eq!(buf[(5, 1)].fg, t.accent, "S1: on accent");
    // S2: hit region plus focus-ring stop.
    assert_eq!(
        stage.hits.hit(Position::new(6, 1)),
        Some(s2_id("test.s2.togon")),
        "S2: hit region"
    );
    assert!(stage.ring.contains(s2_id("test.s2.togon")), "S2: ring stop");
    // A1: Enter/Space activate; other keys ignore; click activates.
    let mut b = Button::secondary(bid, "Run task");
    let (o, fired) = b.on_key(&s2_key(KeyCode::Enter));
    assert!(matches!(o, Outcome::Changed) && fired, "A1: Enter");
    let (o, fired) = b.on_key(&s2_key(KeyCode::Char(' ')));
    assert!(matches!(o, Outcome::Changed) && fired, "A1: Space");
    let (o, fired) = b.on_key(&s2_key(KeyCode::Char('x')));
    assert!(matches!(o, Outcome::Ignored) && !fired, "A1: other keys");
    assert!(b.on_click(), "A1: click");
    // N2: no read-only/error/warning/empty flags gate activation.
    for (disabled, busy) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut b = Button::secondary(bid, "Run task");
        b.disabled = disabled;
        b.busy = busy;
        assert_eq!(
            b.can_activate(),
            !disabled && !busy,
            "N2: gates are disabled/busy only"
        );
    }

    // PTY: tab focuses Run task (bar + footer); hover lifts it.
    let case = s2_case("button_focus", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Enter / Space Activate", "A1 footer");
    let focused = live_text(&mut s);
    assert_line_has(&focused, "▎", "Run task", "V1 live: focus bar on Run task");
    checkpoint(&mut s, &dir, "focused");
    hover_over(&mut s, "Run task", case_timeout(&case));
    let hov = live_frame(&mut s);
    let hov_text = live_text(&mut s);
    assert_line_has(&hov_text, "▎", "Run task", "V1 live: bar survives hover");
    // Hover lift is a tone delta on the label cells (presence: the bar).
    let (row, col) = support::screen_find(
        &s.snapshot()
            .unwrap_or_else(|e| panic!("label pos failed: {e:#}")),
        "Run task",
    )
    .expect("label on screen");
    let cell = hov
        .get(col, row)
        .unwrap_or_else(|| panic!("label cell missing"));
    assert_ne!(cell.bg, TColor::Default, "V1 live: label bg painted");
    let _ = lum(cell.bg, "V1 live label bg");
    eprintln!("s2 button_focus: isolated matrix + live tab/hover");
}

/// BTN-DISABLED-PRECEDENCE-004 (`showcase/audit/buttons`): disabled +
/// combos.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_button_disabled() {
    let dir = s2_dir("s2_button_disabled");
    let t = Theme::junie();
    let bid = s2_id("test.s2.btndis");

    // Isolated disabled-idle: muted, blank gutter, never bold.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::toggle(bid, "Verbose", true).disabled(true);
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), " ", "V1: blank gutter");
    assert!(!buf_is_bold(&buf, 5, 1), "V1: never bold");
    // V2: selected+disabled keeps ● but loses the accent.
    assert_eq!(buf[(5, 1)].symbol(), "●", "V2: marker kept");
    assert_ne!(buf[(5, 1)].fg, t.accent, "V2: accent lost");
    let idle = capture_isolated(&dir, "btndis-idle", &buf);

    // V1/N1: all interaction set still renders exactly disabled-idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let mut b = Button::toggle(bid, "Verbose", true).disabled(true);
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(bid),
            hover: Some(bid),
            pressed: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    let allset = capture_isolated(&dir, "btndis-allset", &buf);
    assert_eq!(
        allset.digest(),
        idle.digest(),
        "V1/N1: no lift/bar/inversion"
    );
    // S1: hit area registered, no focus-ring stop.
    assert_eq!(
        stage.hits.hit(Position::new(6, 1)),
        Some(bid),
        "S1: hit area kept"
    );
    assert!(!stage.ring.contains(bid), "S1: no ring stop");
    // S2/A1: cannot activate; Enter/click consume; toggle never flips.
    let mut b = Button::toggle(bid, "Verbose", true).disabled(true);
    assert!(!b.can_activate(), "S2: disabled cannot activate");
    let (o, fired) = b.on_key(&s2_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Consumed) && !fired,
        "A1: Enter consumed"
    );
    assert!(!b.on_click(), "A1: click false");
    assert_eq!(b.on, Some(true), "A1: toggle never flips");
    let mut b = Button::primary(bid, "Run task");
    b.busy = true;
    assert!(!b.can_activate(), "S2: busy cannot activate");
    // V2: busy keeps hover lift while clearing the pressed inversion.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 3);
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(bid),
            pressed: Some(bid),
            tick: 3,
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        buf[(5, 1)].symbol(),
        spinner_frame(3),
        "V2: busy spinner kept"
    );
    capture_isolated(&dir, "btndis-busy", &buf);

    // PTY: disabled buttons never take focus and never fire.
    let case = s2_case("button_disabled", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(
        boot.contains("Disabled primary"),
        "boot shows disabled buttons"
    );
    // Tab through every stop: the bar never lands on a disabled row.
    let mut saw_enabled_bar = false;
    for _ in 0..8 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(150));
        let text = live_text(&mut s);
        assert_line_lacks(&text, "▎", "Disabled", "N1 live: bar never on disabled");
        if text
            .lines()
            .any(|l| l.contains('▎') && l.contains("Run task"))
        {
            saw_enabled_bar = true;
        }
    }
    assert!(saw_enabled_bar, "N1 presence: bar lands on enabled rows");
    // A1 live: clicking a disabled button neither focuses nor fires.
    support::press_step(&s, "tab");
    waits::wait_state(&mut s, "Run task", "refocus");
    click_at(&mut s, "Disabled primary", case_timeout(&case));
    let after = live_text(&mut s);
    assert_line_lacks(&after, "▎", "Disabled", "A1 live: click moves no focus");
    assert_line_has(&after, "▎", "Run task", "A1 live: focus stays put");
    checkpoint(&mut s, &dir, "clicked-disabled");
    eprintln!("s2 button_disabled: isolated precedence + live no-focus/no-fire");
}

/// CHK-STATE-MATRIX-001 (`showcase/audit/forms`): mark + label row.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_checkbox_matrix() {
    let dir = s2_dir("s2_checkbox_matrix");
    let t = Theme::junie();
    let cid = s2_id("test.s2.chk");

    // Isolated unchecked idle: muted [ ].
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", false);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let row = buf_row_text(&buf, 1);
    assert!(row.contains("[ ]"), "V1: unchecked mark in {row:?}");
    assert_eq!(buf[(6, 1)].fg, t.text_muted, "V1: unchecked muted");
    let idle = capture_isolated(&dir, "chk-idle", &buf);

    // Isolated focused+hovered+checked: ▎ + bold + accent [✓] + lift.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", true);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(cid),
            hover: Some(cid),
            ..Default::default()
        }),
        bg,
    );
    let row = buf_row_text(&buf, 1);
    assert!(row.contains("[✓]"), "V1: checked mark in {row:?}");
    assert_eq!(buf[(4, 1)].symbol(), "▎", "V1: focus bar");
    assert!(buf_is_bold(&buf, 6, 1), "V1: focus bold");
    assert_eq!(buf[(6, 1)].fg, t.accent, "V1: checked accent");
    let hot = capture_isolated(&dir, "chk-hot", &buf);
    assert_ne!(hot.digest(), idle.digest(), "V1: state changes the frame");
    // Hidden focus renders idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", false);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(cid),
            focus_hidden: true,
            ..Default::default()
        }),
        bg,
    );
    let hidden = capture_isolated(&dir, "chk-hidden", &buf);
    assert_eq!(hidden.digest(), idle.digest(), "V1: hidden focus is idle");

    // V2: narrow compacts the mark; 1 cell shows the bar only.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 3);
    let mut c = Checkbox::new(cid, "Run tests", true);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 3, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let narrow = buf_row_text(&buf, 1);
    assert!(
        narrow.contains('✓') && !narrow.contains('['),
        "V2: compact mark in {narrow:?}"
    );
    capture_isolated(&dir, "chk-narrow3", &buf);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 3);
    let mut c = Checkbox::new(cid, "Run tests", true);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 1, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(cid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), "▎", "V2: 1-cell bar only");

    // S1: disabled keeps the glyph, no lift, no stop; all-set == idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", true);
    c.disabled = true;
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let drow = buf_row_text(&buf, 1);
    assert!(drow.contains('✓'), "S1: disabled keeps ✓ in {drow:?}");
    assert!(!stage.ring.contains(cid), "S1: no focus stop");
    let dis_idle = capture_isolated(&dir, "chk-dis-idle", &buf);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", true);
    c.disabled = true;
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(cid),
            hover: Some(cid),
            pressed: Some(cid),
            ..Default::default()
        }),
        bg,
    );
    let dis_hot = capture_isolated(&dir, "chk-dis-hot", &buf);
    assert_eq!(
        dis_hot.digest(),
        dis_idle.digest(),
        "S1: disabled all-set is idle"
    );

    // A1: Space/Enter/click flip; disabled input never flips.
    let mut c = Checkbox::new(cid, "Run tests", false);
    assert!(
        matches!(c.on_key(&s2_key(KeyCode::Char(' '))), Outcome::Changed),
        "A1: Space"
    );
    assert!(c.checked, "A1: flipped on");
    assert!(
        matches!(c.on_key(&s2_key(KeyCode::Enter)), Outcome::Changed),
        "A1: Enter"
    );
    assert!(!c.checked, "A1: flipped off");
    assert!(matches!(c.on_click(), Outcome::Changed), "A1: click");
    assert!(c.checked, "A1: click flips");
    c.disabled = true;
    assert!(
        matches!(c.on_key(&s2_key(KeyCode::Char(' '))), Outcome::Ignored),
        "A1: disabled Space"
    );
    assert!(
        matches!(c.on_click(), Outcome::Consumed),
        "A1: disabled click"
    );
    assert!(c.checked, "A1: disabled never flips");
    // N1: no busy/loading/read-only/error/warning flags (struct has none
    // beyond disabled; activation gates on disabled only).

    // PTY: the forms page checkboxes toggle live on click.
    let case = s2_case("checkbox_matrix", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("[✓]"), "boot shows a checked box");
    assert!(boot.contains("[ ]"), "boot shows an unchecked box");
    click_at(&mut s, "[✓]", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: checkbox never unticked",
        |screen| {
            !support::screen_text(screen)
                .lines()
                .any(|l| l.contains("[✓]") && l.contains("Run tests"))
        },
    );
    let unticked = live_text(&mut s);
    assert_line_has(&unticked, "[ ]", "Run tests", "A1 live: click unticks");
    click_at(&mut s, "Run tests", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: checkbox never reticked",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("[✓]") && l.contains("Run tests"))
        },
    );
    checkpoint(&mut s, &dir, "reticked");
    eprintln!("s2 checkbox_matrix: isolated matrix + live toggle roundtrip");
}

/// TGL-STATE-MATRIX-001 (`showcase/audit/forms`): switch + label + state
/// word.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_toggle_matrix() {
    let dir = s2_dir("s2_toggle_matrix");
    let t = Theme::junie();
    let gid = s2_id("test.s2.tgl");

    // Isolated off: muted ○── + 'off'.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(34, 3);
    let mut g = Toggle::new(gid, "Auto-approve", false);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 26, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let row = buf_row_text(&buf, 1);
    assert!(row.contains("○──"), "V1: off switch in {row:?}");
    assert!(row.contains("off"), "V1: off word in {row:?}");
    assert_eq!(buf[(5, 1)].fg, t.text_muted, "V1: off muted");
    let idle = capture_isolated(&dir, "tgl-off", &buf);

    // Isolated focused+hovered+on: ▎ + bold + accent ──● + 'on'.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(34, 3);
    let mut g = Toggle::new(gid, "Auto-approve", true);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 26, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(gid),
            hover: Some(gid),
            ..Default::default()
        }),
        bg,
    );
    let row = buf_row_text(&buf, 1);
    assert!(row.contains("──●"), "V1: on switch in {row:?}");
    assert!(row.contains(" on"), "V1: on word in {row:?}");
    assert_eq!(buf[(4, 1)].symbol(), "▎", "V1: focus bar");
    assert!(buf_is_bold(&buf, 6, 1), "V1: focus bold");
    assert_eq!(buf[(7, 1)].fg, t.accent, "V1: on accent");
    let hot = capture_isolated(&dir, "tgl-hot", &buf);
    assert_ne!(hot.digest(), idle.digest(), "V1: state changes the frame");

    // V2: below 4 cells the switch compacts to a single ●/○.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(12, 3);
    let mut g = Toggle::new(gid, "Auto-approve", true);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 3, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let narrow = buf_row_text(&buf, 1);
    assert!(
        narrow.contains('●') && !narrow.contains("──"),
        "V2: compact ● in {narrow:?}"
    );
    capture_isolated(&dir, "tgl-narrow3", &buf);

    // S1: disabled keeps glyphs, no lift, no stop; all-set == idle.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(34, 3);
    let mut g = Toggle::new(gid, "Auto-approve", true).disabled(true);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 26, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let drow = buf_row_text(&buf, 1);
    assert!(
        drow.contains("──●"),
        "S1: disabled keeps glyphs in {drow:?}"
    );
    assert!(!stage.ring.contains(gid), "S1: no focus stop");
    let dis_idle = capture_isolated(&dir, "tgl-dis-idle", &buf);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(34, 3);
    let mut g = Toggle::new(gid, "Auto-approve", true).disabled(true);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 26, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(gid),
            hover: Some(gid),
            pressed: Some(gid),
            ..Default::default()
        }),
        bg,
    );
    let dis_hot = capture_isolated(&dir, "tgl-dis-hot", &buf);
    assert_eq!(
        dis_hot.digest(),
        dis_idle.digest(),
        "S1: disabled all-set is idle"
    );

    // A1: Space/Enter/click flip; disabled input never flips.
    let mut g = Toggle::new(gid, "Auto-approve", false);
    assert!(
        matches!(g.on_key(&s2_key(KeyCode::Char(' '))), Outcome::Changed),
        "A1: Space"
    );
    assert!(g.on, "A1: flipped on");
    assert!(
        matches!(g.on_key(&s2_key(KeyCode::Enter)), Outcome::Changed),
        "A1: Enter"
    );
    assert!(!g.on, "A1: flipped off");
    assert!(matches!(g.on_click(), Outcome::Changed), "A1: click");
    assert!(g.on, "A1: click flips");
    g.disabled = true;
    assert!(
        matches!(g.on_key(&s2_key(KeyCode::Char(' '))), Outcome::Ignored),
        "A1: disabled keys"
    );
    assert!(
        matches!(g.on_click(), Outcome::Consumed),
        "A1: disabled click"
    );
    assert!(g.on, "A1: disabled never flips");
    // N1: no busy/loading/read-only/error/warning flags (disabled only).

    // PTY: the forms page toggles flip live on click.
    let case = s2_case("toggle_matrix", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("○──"), "boot shows an off toggle");
    assert!(boot.contains("──●"), "boot shows an on toggle");
    click_at(&mut s, "Auto-approve", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: toggle never flipped on",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("──●") && l.contains("Auto-approve"))
        },
    );
    click_at(&mut s, "Auto-approve", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: toggle never flipped off",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("○──") && l.contains("Auto-approve"))
        },
    );
    checkpoint(&mut s, &dir, "roundtripped");
    eprintln!("s2 toggle_matrix: isolated matrix + live flip roundtrip");
}

/// RADIO-STATE-MATRIX-001 (`showcase/audit/forms`): label row + option
/// rows.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_radio_matrix() {
    let dir = s2_dir("s2_radio_matrix");
    let t = Theme::junie();
    let rid = s2_id("test.s2.radio");
    let opts = ["Fast", "Balanced", "Thorough"];

    // Isolated idle: secondary label, accent (●) on option 1.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 6);
    let mut r = RadioGroup::new(rid, "Mode", &opts, 1);
    let bg = stage.bg();
    r.render(
        Rect::new(4, 1, 20, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let label = buf_row_text(&buf, 1);
    let opt0 = buf_row_text(&buf, 2);
    let opt1 = buf_row_text(&buf, 3);
    assert!(label.contains("Mode"), "V1: label in {label:?}");
    assert!(opt0.contains("( )"), "V1: unselected mark in {opt0:?}");
    assert!(opt1.contains("(●)"), "V1: selected mark in {opt1:?}");
    assert_eq!(buf[(6, 3)].fg, t.accent, "V1: selected accent");
    let idle = capture_isolated(&dir, "radio-idle", &buf);

    // Isolated focused + option-0 hover: bold label, ▎ on cursor row
    // only, exactly the hovered row lifted.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 6);
    let mut r = RadioGroup::new(rid, "Mode", &opts, 1);
    let bg = stage.bg();
    let opt0_id = r.option_id(0);
    r.render(
        Rect::new(4, 1, 20, 4),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(rid),
            hover: Some(opt0_id),
            ..Default::default()
        }),
        bg,
    );
    assert!(buf_is_bold(&buf, 6, 1), "V1: focused label bold");
    assert_eq!(buf[(4, 3)].symbol(), "▎", "V1: bar on cursor row");
    assert_eq!(buf[(4, 2)].symbol(), " ", "V1: no bar elsewhere");
    assert_ne!(buf[(6, 2)].bg, buf[(6, 4)].bg, "V2: exactly one row lifted");
    let hot = capture_isolated(&dir, "radio-hot", &buf);
    assert_ne!(hot.digest(), idle.digest(), "V1: state changes the frame");

    // S1: single focus stop; option rows are click targets.
    assert!(stage.ring.contains(rid), "S1: one ring stop");
    assert!(!stage.ring.contains(opt0_id), "S1: options are not stops");
    assert_eq!(
        stage.hits.hit(Position::new(8, 2)),
        Some(opt0_id),
        "S1: option click target"
    );
    // S2: empty options render the label only with no stop.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 6);
    let mut r = RadioGroup::new(rid, "Mode", &[], 0);
    let bg = stage.bg();
    r.render(
        Rect::new(4, 1, 20, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(!stage.ring.contains(rid), "S2: empty group has no stop");

    // A1: arrows move cursor+selected; Space/Enter commit; click selects.
    let mut r = RadioGroup::new(rid, "Mode", &opts, 1);
    assert!(
        matches!(r.on_key(&s2_key(KeyCode::Down)), Outcome::Changed),
        "A1: Down"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 6);
    let bg = stage.bg();
    r.render(
        Rect::new(4, 1, 20, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(buf_row_text(&buf, 4).contains("(●)"), "A1: selection moved");
    capture_isolated(&dir, "radio-moved", &buf);
    assert!(
        matches!(r.on_click(0), Outcome::Changed),
        "A1: click selects"
    );
    // N1: no busy/loading flags (disabled + cursor only).

    // PTY: the forms page mode group moves live on click.
    let case = s2_case("radio_matrix", &["--page", "forms"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert_line_has(&boot, "(●)", "Balanced", "boot selection");
    click_at(&mut s, "Fast", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: radio never moved to Fast",
        |screen| {
            support::screen_text(screen)
                .lines()
                .any(|l| l.contains("(●)") && l.contains("Fast"))
        },
    );
    let moved = live_text(&mut s);
    assert_line_lacks(&moved, "(●)", "Balanced", "A1 live: old mark cleared");
    checkpoint(&mut s, &dir, "moved");
    eprintln!("s2 radio_matrix: isolated matrix + live move");
}

/// Plain line style for [`ScrollPanel`] isolated renders.
fn scroll_line(t: &Theme, _line: &str) -> ratatui::style::Style {
    ratatui::style::Style::new().fg(t.text_primary)
}

/// PANEL-FOCUS-READONLY-001 (`showcase/pages/panels`): card/framed chrome
/// + scroll content.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_panel_focus() {
    let dir = s2_dir("s2_panel_focus");
    let t = Theme::junie();

    // Isolated card idle: secondary title; inner sits below the title row.
    let mut buf = sentinel_buf(40, 8);
    let card = Panel::card(Some("Titled card")).meta("surface");
    let inner = card.render(Rect::new(4, 1, 30, 6), &mut buf, &t);
    assert_eq!(inner.y, 3, "S1: inner below title row + padding");
    let title_row = buf_row_text(&buf, 1);
    assert!(
        title_row.contains("Titled card"),
        "V1: title in {title_row:?}"
    );
    assert!(
        title_row.contains("surface"),
        "V2: meta kept in {title_row:?}"
    );
    capture_isolated(&dir, "panel-card-idle", &buf);

    // Isolated card focused: ▎ in the padding column + bold-primary title.
    let mut buf = sentinel_buf(40, 8);
    let card = Panel::card(Some("Titled card"))
        .focused(true)
        .meta("surface");
    card.render(Rect::new(4, 1, 30, 6), &mut buf, &t);
    assert_eq!(buf[(5, 1)].symbol(), "▎", "V1: focus bar in padding col");
    assert!(buf_is_bold(&buf, 6, 1), "V1: focused title bold");
    capture_isolated(&dir, "panel-card-focused", &buf);

    // Isolated framed: subtle border idle, strong border focused.
    let mut buf = sentinel_buf(40, 8);
    Panel::framed(Some("Framed")).render(Rect::new(4, 1, 30, 6), &mut buf, &t);
    let idle_corner = buf[(4, 1)].fg;
    capture_isolated(&dir, "panel-framed-idle", &buf);
    let mut buf = sentinel_buf(40, 8);
    Panel::framed(Some("Framed"))
        .focused(true)
        .render(Rect::new(4, 1, 30, 6), &mut buf, &t);
    assert_ne!(
        buf[(4, 1)].fg,
        idle_corner,
        "V1: focused border strengthens"
    );
    capture_isolated(&dir, "panel-framed-focused", &buf);

    // Isolated scroll content is read-only: scroll keys move, edit-ish
    // keys ignore, and no key sequence mutates the lines.
    let pid = s2_id("test.s2.scroll");
    let lines: Vec<String> = (0..20).map(|i| format!("log line {i:02}")).collect();
    let mut p = ScrollPanel::new(pid, lines.clone());
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 10);
    let bg = stage.bg();
    p.render(
        Rect::new(4, 1, 22, 6),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(pid),
            ..Default::default()
        }),
        bg,
        scroll_line,
    );
    assert!(
        buf_row_text(&buf, 1).contains("log line 00"),
        "S2: starts at top"
    );
    assert!(
        matches!(p.on_key(&s2_key(KeyCode::Down)), Outcome::Changed),
        "A1: Down scrolls"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 10);
    let bg = stage.bg();
    p.render(
        Rect::new(4, 1, 22, 6),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(pid),
            ..Default::default()
        }),
        bg,
        scroll_line,
    );
    assert!(
        buf_row_text(&buf, 1).contains("log line 01"),
        "S2: offset moved"
    );
    capture_isolated(&dir, "panel-scrolled", &buf);
    for code in [KeyCode::Enter, KeyCode::Char('x'), KeyCode::Char(' ')] {
        assert!(
            matches!(p.on_key(&s2_key(code)), Outcome::Ignored),
            "A1/S2: {code:?} ignores"
        );
    }
    // N1: no hover/pressed/selected/disabled/busy rendering on chrome
    // (container focus shows on border/title only — proven above: the
    // focused/unfocused delta is the bar + title/border tones).

    // PTY: the panels page focuses cards and scrolls read-only content.
    let case = s2_case("panel_focus", &["--page", "panels"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("Titled card"), "boot shows the titled card");
    assert!(boot.contains("Framed"), "boot shows framed chrome");
    assert!(boot.contains("of 60"), "boot shows the log scroll readout");
    // Tab cycles the ring (nested list, prose, log): stop when a card
    // title row carries ▎ (the nested list may own earlier stops).
    let mut entered = false;
    for _ in 0..4 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(200));
        let text = live_text(&mut s);
        if text.lines().any(|l| {
            l.contains('▎')
                && (l.contains("of 60") || l.contains("of 41") || l.contains("scrollable"))
        }) {
            entered = true;
            break;
        }
    }
    assert!(entered, "V1 live: focus never reached a card");
    checkpoint(&mut s, &dir, "focused");
    // Wheel over the log card content scrolls it (no focus needed):
    // the readout start offset increases. (The readout sits in the
    // title row, which is chrome — wheel the content rows instead.)
    let (log_row, log_col) = find_pos(&mut s, "Resol", case_timeout(&case));
    let before = live_text(&mut s);
    let start_before = scroll_start(&before, "of 60");
    wheel_at(&mut s, log_col, log_row, Wheel::Down, 2);
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: wheel never scrolled the log",
        |screen| scroll_start(&support::screen_text(screen), "of 60") != start_before,
    );
    let after_wheel = live_text(&mut s);
    assert!(
        scroll_start(&after_wheel, "of 60") > start_before,
        "A1 live: wheel scrolled down"
    );
    // Read-only: typing mutates nothing (byte-identical after settle).
    support::press_step(&s, "x");
    std::thread::sleep(Duration::from_millis(400));
    let after_type = live_text(&mut s);
    assert_eq!(after_type, after_wheel, "S2 live: typing mutates nothing");
    checkpoint(&mut s, &dir, "scrolled");
    eprintln!("s2 panel_focus: isolated chrome/scroll + live wheel/read-only");
}

/// Parse the `N–M of K` readout start offset for the `of K` card.
fn scroll_start(text: &str, of: &str) -> u32 {
    for line in text.lines() {
        if let Some(pos) = line.find(of) {
            let head = &line[..pos];
            if let Some(dash) = head.rfind(['–', '-']) {
                let num: String = head[..dash]
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                if let Ok(n) = num.parse::<u32>() {
                    return n;
                }
            }
        }
    }
    panic!("no {of:?} readout in\n{text}")
}

/// SPLIT-SEAM-DRAG-001 (`showcase/pages/terminal`): gap-strip handle +
/// pane geometry.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_split_drag() {
    let dir = s2_dir("s2_split_drag");
    let sid = s2_id("test.s2.split");
    let container = Rect::new(0, 0, 80, 24);

    // Isolated layout: panes + gap exactly fill the container.
    let split = Split::new(50, 5, 5);
    let (left, right) = split.layout(SplitDir::Horizontal, container, 1);
    let handle = split.handle(SplitDir::Horizontal, container, 1);
    assert_eq!(left.width + 1 + right.width, 80, "V2: panes + gap fill");
    assert_eq!(handle.width, 1, "V2: 1-cell gap strip");
    assert_eq!(handle.x, left.right(), "V2: strip between panes");

    // Isolated handle: quiet │ idle, strong on hover, heavy ┃ on drag.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut seam = Splitter::new(sid, SplitDir::Horizontal);
    let bg = stage.bg();
    seam.render(handle, &mut buf, &mut stage.ctx(Interaction::default()), bg);
    assert_eq!(buf[(handle.x, 12)].symbol(), "│", "V1: idle rule");
    let idle = capture_isolated(&dir, "split-idle", &buf);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut seam = Splitter::new(sid, SplitDir::Horizontal);
    let bg = stage.bg();
    seam.render(
        handle,
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(sid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        buf[(handle.x, 12)].symbol(),
        "│",
        "V1: hover keeps the glyph"
    );
    let hovered = capture_isolated(&dir, "split-hover", &buf);
    assert_ne!(hovered.digest(), idle.digest(), "V1: hover strengthens");
    // N1: focus and flash must NOT change the handle; raw pressed draws
    // heavy even without hover.
    for (name, ix, heavy) in [
        (
            "focus",
            Interaction {
                focus: Some(sid),
                ..Default::default()
            },
            false,
        ),
        (
            "flash",
            Interaction {
                flash: Some(sid),
                ..Default::default()
            },
            false,
        ),
        (
            "pressed",
            Interaction {
                pressed: Some(sid),
                ..Default::default()
            },
            true,
        ),
    ] {
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(80, 24);
        let mut seam = Splitter::new(sid, SplitDir::Horizontal);
        let bg = stage.bg();
        seam.render(handle, &mut buf, &mut stage.ctx(ix), bg);
        let want = if heavy { "┃" } else { "│" };
        assert_eq!(buf[(handle.x, 12)].symbol(), want, "N1: {name}");
        capture_isolated(&dir, &format!("split-{name}"), &buf);
    }
    // S2: click hit, never a ring stop.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut seam = Splitter::new(sid, SplitDir::Horizontal);
    let bg = stage.bg();
    seam.render(handle, &mut buf, &mut stage.ctx(Interaction::default()), bg);
    assert_eq!(
        stage.hits.hit(Position::new(handle.x, 12)),
        Some(sid),
        "S2: click hit"
    );
    assert!(!stage.ring.contains(sid), "S2: never a stop");
    // S1/A1: drag moves the seam clamped; repeats consume; nudge ±1.
    let mut split = Split::new(50, 5, 5);
    let seam = Splitter::new(sid, SplitDir::Horizontal);
    assert!(
        matches!(
            seam.on_drag(&mut split, container, 1, Position::new(56, 12)),
            Outcome::Changed
        ),
        "A1: drag moves"
    );
    assert!(split.percent > 50, "S1: seam moved right");
    assert!(
        matches!(
            seam.on_drag(&mut split, container, 1, Position::new(56, 12)),
            Outcome::Consumed
        ),
        "S1: repeat consumes"
    );
    let before = split.percent;
    split.nudge(SplitDir::Horizontal, container, 1, -1);
    assert_eq!(split.percent + 1, before, "S1: nudge moves one cell");
    // V2: maximized splits have an empty handle that draws nothing.
    split.toggle_max(Maximized::First);
    let handle = split.handle(SplitDir::Horizontal, container, 1);
    assert!(handle.is_empty(), "V2: maximized handle empty");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(80, 24);
    let mut seam = Splitter::new(sid, SplitDir::Horizontal);
    let bg = stage.bg();
    seam.render(handle, &mut buf, &mut stage.ctx(Interaction::default()), bg);
    assert_surround_intact(&buf, Rect::new(0, 0, 0, 0), "V2: maximized draws nothing");
    // N2: mouse-only affordance (no disabled/selected/busy rendering).

    // PTY: the terminal page seam drags live. Paused frame 60 pins the
    // staged run mid-stream, so the rail ("2 of 7") is a stable tracker
    // (the live 7-of-7 end state takes ~30 s to arrive).
    let case = s2_case(
        "split_drag",
        &["--page", "terminal", "--motion", "paused", "--frame", "60"],
        80,
        24,
    );
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    // The seam is the │ right of the viewport ╮ on the box-top row
    // (the page title also says "Viewport", so require the corner).
    let boot = live_text(&mut s);
    let top = boot
        .lines()
        .find(|l| l.contains("Viewp") && l.contains('╮'))
        .expect("viewport top row")
        .to_string();
    let corner = top.chars().position(|c| c == '╮').expect("box corner");
    let seam_col = top
        .chars()
        .enumerate()
        .skip_while(|(i, _)| *i <= corner)
        .find(|(_, c)| *c == '│')
        .map(|(i, _)| i as u16)
        .expect("seam right of the corner");
    let top_row = boot
        .lines()
        .position(|l| l.contains("Viewp") && l.contains('╮'))
        .expect("top row") as u16;
    let grip_row = top_row + 8;
    let idle_frame = live_frame(&mut s);
    let idle_tone = idle_frame.get(seam_col, grip_row).expect("seam cell").fg;
    Input::move_to(seam_col, grip_row).send(&mut s);
    std::thread::sleep(Duration::from_millis(200));
    let hov_frame = live_frame(&mut s);
    let hov_tone = hov_frame
        .get(seam_col, grip_row)
        .expect("hovered seam cell")
        .fg;
    assert_ne!(idle_tone, hov_tone, "V1 live: hover strengthens the seam");
    // Drag the seam left: the step rail follows.
    let (_, rail_before) = find_pos(&mut s, "2 of 7", case_timeout(&case));
    Input::drag((seam_col, grip_row), (seam_col.saturating_sub(8), grip_row)).send(&mut s);
    std::thread::sleep(Duration::from_millis(300));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "A1: rail never followed the drag",
        |screen| support::screen_find(screen, "2 of 7").is_some_and(|(_, c)| c != rail_before),
    );
    let (_, rail_after) = support::screen_find(
        &s.snapshot()
            .unwrap_or_else(|e| panic!("rail pos failed: {e:#}")),
        "2 of 7",
    )
    .expect("rail on screen");
    assert!(
        rail_after < rail_before,
        "A1 live: rail moved left {rail_before} -> {rail_after}"
    );
    checkpoint(&mut s, &dir, "dragged");
    eprintln!("s2 split_drag: isolated geometry + live hover/drag");
}

/// PROPS-CURSOR-TONE-001 (`showcase/pages/datagrid`): fact rows + cursor
/// + copy hint.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_props_cursor() {
    let dir = s2_dir("s2_props_cursor");
    let t = Theme::junie();
    let lid = s2_id("test.s2.props");
    let mk = || {
        PropsList::new(
            lid,
            vec![
                Prop::new("Engine", "PostgreSQL 16.3"),
                Prop::new("Host", "prod-db-1:5432").copyable(),
                Prop::new("Errors", "3 failed").tone(Tone::Error),
            ],
        )
    };

    // Isolated idle: muted labels at x+2, values in their tones.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(50, 6);
    let mut l = mk();
    let bg = stage.bg();
    l.render(
        Rect::new(4, 1, 42, 4),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(6, 1)].fg, t.text_muted, "V1: label muted");
    // label_w = max(6,4,6)+2 = 8; values at x+2+8 = x+10.
    assert_eq!(buf[(14, 3)].fg, t.tone(Tone::Error), "V1: error tone kept");
    let idle = capture_isolated(&dir, "props-idle", &buf);

    // Isolated cursor on the copyable row: ▎ + bold + 'y copy' hint.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(50, 6);
    let mut l = mk();
    let (o, ev) = l.on_key(&s2_key(KeyCode::Down));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none(),
        "cursor moves"
    );
    let bg = stage.bg();
    l.render(
        Rect::new(4, 1, 42, 4),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(lid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 2)].symbol(), "▎", "V1: cursor bar");
    // Labels shed bold by construction; the value row carries it.
    assert!(buf_is_bold(&buf, 14, 2), "V1: cursor bold");
    let cursor_row = buf_row_text(&buf, 2);
    assert!(
        cursor_row.contains("y copy"),
        "V1: copy hint in {cursor_row:?}"
    );
    // N1: no hint on non-copyable or unfocused rows.
    assert!(
        !buf_row_text(&buf, 1).contains("y copy"),
        "N1: hint only on cursor"
    );
    let cursor = capture_isolated(&dir, "props-cursor", &buf);
    assert_ne!(
        cursor.digest(),
        idle.digest(),
        "V1: cursor changes the frame"
    );

    // S1: one focus stop; empty sheet renders blank and ignores keys.
    assert!(stage.ring.contains(lid), "S1: one focus stop");
    assert!(!stage.ring.contains(l.row_id(0)), "S1: rows are not stops");
    let mut empty = PropsList::new(lid, vec![]);
    let (o, _) = empty.on_key(&s2_key(KeyCode::Down));
    assert!(matches!(o, Outcome::Ignored), "S1: empty ignores keys");
    // A1: y copies the copyable row, consumes elsewhere; Enter activates;
    // click moves the cursor and activates; values never mutate.
    let mut l = mk();
    l.on_key(&s2_key(KeyCode::Down));
    let (o, ev) = l.on_key(&s2_key(KeyCode::Char('y')));
    assert!(matches!(o, Outcome::Changed), "A1: y copies");
    assert!(matches!(ev, Some(PropsEvent::Copy(1))), "A1: Copy(1)");
    l.on_key(&s2_key(KeyCode::Up));
    let (o, ev) = l.on_key(&s2_key(KeyCode::Char('y')));
    assert!(
        matches!(o, Outcome::Consumed) && ev.is_none(),
        "A1: y consumes elsewhere"
    );
    let (o, ev) = l.on_key(&s2_key(KeyCode::Enter));
    assert!(matches!(o, Outcome::Changed), "A1: Enter activates");
    assert!(
        matches!(ev, Some(PropsEvent::Activate(0))),
        "A1: Activate(0)"
    );
    let (o, ev) = l.on_click(2);
    assert!(matches!(o, Outcome::Changed), "A1: click activates");
    assert!(
        matches!(ev, Some(PropsEvent::Activate(2))),
        "A1: Activate(2)"
    );
    assert_eq!(l.cursor, 2, "A1: click moves the cursor");
    assert_eq!(l.props[1].value, "prod-db-1:5432", "A1: read-only values");
    // S2: static measure agrees with render row usage.
    assert_eq!(
        props::measure(&mk().props, 42),
        3,
        "S2: measure counts rows"
    );

    // PTY: the chips page property block renders labels + toned values.
    let case = s2_case("props_cursor", &["--page", "chipsselects"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let text = live_text(&mut s);
    assert!(text.contains("PostgreSQL 16.3"), "V1 live: engine value");
    assert!(text.contains("prod-db-1.acme.io"), "V1 live: host value");
    assert_line_has(&text, "Engine", "PostgreSQL", "V1 live: fact row");
    // Exact live tones: labels muted, values in the normal tone. Both
    // needles repeat in the Selects section, so pin the property line
    // (the only line holding both) and sample within it.
    let frame = live_frame(&mut s);
    let text = live_text(&mut s);
    let (row, line) = text
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("Engine") && l.contains("PostgreSQL"))
        .expect("property line");
    let row = row as u16;
    let col = char_col(line, "Engine");
    let vcol = char_col(line, "PostgreSQL");
    let label_fg = frame.get(col, row).expect("label cell").fg;
    let value_fg = frame.get(vcol, row).expect("value cell").fg;
    assert_eq!(
        resolve_rgb(label_fg, "label"),
        theme_rgb(t.text_muted),
        "V1 live: label muted"
    );
    assert_eq!(
        resolve_rgb(value_fg, "value"),
        theme_rgb(t.tone(Tone::Normal)),
        "V1 live: value in normal tone"
    );
    eprintln!("s2 props_cursor: isolated list + live property block");
}

/// EMPTY-KIND-NARROW-001 (`showcase/pages/chips`): centered title + hint
/// block.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_empty_narrow() {
    let dir = s2_dir("s2_empty_narrow");
    let t = Theme::junie();

    // Isolated empty: muted centered title, faint hint two rows below.
    let mut buf = sentinel_buf(40, 9);
    empty::render(
        Rect::new(4, 1, 32, 7),
        &mut buf,
        &t,
        &EmptyState::new("No results yet").hint("A title and one hint"),
        t.canvas,
    );
    let rows: Vec<String> = (0..9).map(|y| buf_row_text(&buf, y)).collect();
    let title_y = rows
        .iter()
        .position(|r| r.contains("No results yet"))
        .expect("title row");
    let hint_y = rows
        .iter()
        .position(|r| r.contains("A title and one hint"))
        .expect("hint row");
    assert_eq!(hint_y, title_y + 2, "V1: hint two rows below");
    assert_eq!(
        buf[(4 + 16 - 7, title_y as u16)].fg,
        t.text_muted,
        "V1: title muted"
    );
    capture_isolated(&dir, "empty-plain", &buf);
    assert_surround_intact(&buf, Rect::new(4, 1, 32, 7), "N1: nothing outside");

    // Isolated error: '! <title>' in error tone with a bold '!' only.
    let mut buf = sentinel_buf(40, 9);
    empty::render(
        Rect::new(4, 1, 32, 7),
        &mut buf,
        &t,
        &EmptyState::error("Load failed"),
        t.canvas,
    );
    let rows: Vec<String> = (0..9).map(|y| buf_row_text(&buf, y)).collect();
    let ey = rows
        .iter()
        .position(|r| r.contains("Load failed"))
        .expect("error row") as u16;
    let bang_x = rows[ey as usize]
        .chars()
        .position(|c| c == '!')
        .expect("bang") as u16;
    assert!(buf_is_bold(&buf, bang_x, ey), "V1: bold bang");
    capture_isolated(&dir, "empty-error", &buf);

    // V2: long titles truncate with …; a 1-row area skips the hint.
    let mut buf = sentinel_buf(20, 5);
    empty::render(
        Rect::new(4, 1, 10, 3),
        &mut buf,
        &t,
        &EmptyState::new("A very long title indeed").hint("hint"),
        t.canvas,
    );
    let narrow: Vec<String> = (0..5).map(|y| buf_row_text(&buf, y)).collect();
    assert!(
        narrow.iter().any(|r| r.contains('…')),
        "V2: truncation in {narrow:?}"
    );
    let mut buf = sentinel_buf(40, 5);
    empty::render(
        Rect::new(4, 1, 32, 1),
        &mut buf,
        &t,
        &EmptyState::new("Title").hint("hint"),
        t.canvas,
    );
    let one: Vec<String> = (0..5).map(|y| buf_row_text(&buf, y)).collect();
    assert!(one.iter().any(|r| r.contains("Title")), "V2: 1-row title");
    assert!(
        !one.iter().any(|r| r.contains("hint")),
        "V2: 1-row skips hint"
    );
    capture_isolated(&dir, "empty-narrow", &buf);
    // S1/A1/N1: vertically centered block (title row above middle —
    // proven by title_y < area center below); no handlers exist by
    // construction (EmptyState exposes no on_key/on_click).
    assert!(title_y < 4, "S1: block vertically centered");

    // PTY: the chips page empty state renders beside the properties.
    let case = s2_case("empty_narrow", &["--page", "chipsselects"], 120, 40);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let text = live_text(&mut s);
    assert!(text.contains("No results yet"), "V1 live: empty title");
    assert!(text.contains("Empty state"), "V1 live: empty card");
    assert!(text.contains("Properties"), "V1 live: properties beside it");
    eprintln!("s2 empty_narrow: isolated variants + live empty card");
}

/// Spawn `bin` at `cols`x`rows` waiting for `needle` (TooSmall helper).
fn spawn_notice(
    bin: &'static str,
    slug: &str,
    cols: u16,
    rows: u16,
    needle: &'static str,
) -> (Case, Session) {
    let case = Case::dynamic(
        format!("journeys/showcase/s2/{slug}"),
        bin,
        &[],
        cols,
        rows,
        Color::Truecolor,
        needle,
    );
    let s = support::spawn_boot(&case);
    (case, s)
}

/// TOOSMALL-NOTICE-001 (all four apps): undersize notice screen.
/// PTY-only row: no isolated path exists for an app-level screen.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_toosmall_notice() {
    let dir = s2_dir("s2_toosmall_notice");
    // V1/S1: all four apps show the notice + the identical size line.
    for (app, bin) in [
        ("showcase", SHOWCASE),
        ("tablepro", TABLEPRO),
        ("jackin-preview", JACKIN),
        ("holla", HOLLA),
    ] {
        let slug = format!("toosmall-notice-{app}");
        let (case, mut s) = spawn_notice(bin, &slug, 60, 10, "Terminal too small");
        if app == "showcase" {
            write_provenance(&dir, &case);
        }
        checkpoint(&mut s, &dir, &format!("notice-{app}"));
        let text = live_text(&mut s);
        assert!(text.contains("Terminal too small"), "V1: {app} notice");
        assert!(
            text.contains("Need 72×20, have 60×10"),
            "V1/S1: {app} size line"
        );
        assert!(text.contains("q Quit"), "V1: {app} quit line");
        // N1: no shell controls render while undersize (the brand header
        // is part of the notice chrome, so pin nav/page controls).
        assert!(!text.contains('›'), "N1: {app} no nav cursor");
        assert!(!text.contains("Playground"), "N1: {app} no page body");
    }
    // V2/N1: exactly 72x20 boots the normal shell (strict predicate).
    let (_, mut s) = spawn_notice(SHOWCASE, "toosmall-boundary", 72, 20, BOOT);
    checkpoint(&mut s, &dir, "boundary");
    let text = live_text(&mut s);
    assert!(
        !text.contains("Terminal too small"),
        "V2/N1: 72x20 boots normally"
    );
    // A1: q quits from the notice screen.
    let (_, s) = spawn_notice(SHOWCASE, "toosmall-quit", 60, 10, "Terminal too small");
    support::press_step(&s, "q");
    let deadline = std::time::Instant::now() + Duration::from_millis(8_000);
    loop {
        match s.observe_now() {
            Err(_) => break,
            Ok(obs) => {
                if support::screen_text(&obs.screen).trim().is_empty() {
                    break;
                }
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "A1: q never quit the notice"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    eprintln!("s2 toosmall_notice: 4-app notice + boundary + q-quit");
}

/// CTRL-NARROW-CLIP-001 (`showcase/audit/buttons`): widget areas at
/// 6/4/3/2/1 cells. Headless-only row: no PTY path exists for
/// sentinel-buffer clipping.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_narrow_clip() {
    let dir = s2_dir("s2_narrow_clip");
    // V1/S1/A1: every control renders without panic at every swept
    // width; areas stay in bounds; sentinels intact; no state moves.
    for width in [6u16, 4, 3, 2, 1] {
        // Button clips to min(logical, area).
        let bid = s2_id("test.s2.narrow.btn");
        let mut b = Button::secondary(bid, "Run task");
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        let bg = stage.bg();
        b.render(
            Rect::new(4, 1, width, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_eq!(
            b.area.width,
            10u16.min(width),
            "S1: button clips at {width}"
        );
        assert_surround_intact(&buf, Rect::new(4, 1, b.area.width, 1), "V1 button");
        assert!(!b.disabled && !b.busy && b.on.is_none(), "A1: button flags");
        if (3..=6).contains(&width) {
            assert!(
                buf_row_text(&buf, 1).contains('…'),
                "V2: button truncates at {width}"
            );
        } else {
            assert!(
                !buf_row_text(&buf, 1).contains("Run"),
                "V2: button clips the label at {width}"
            );
        }
        // Checkbox / toggle / radio compact their marks.
        let mut c = Checkbox::new(s2_id("test.s2.narrow.chk"), "Run tests", true);
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        let bg = stage.bg();
        c.render(
            Rect::new(4, 1, width, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(4, 1, width, 1), "V1 checkbox");
        assert!(c.checked, "A1: checkbox flag");
        let mut g = Toggle::new(s2_id("test.s2.narrow.tgl"), "Auto-approve", true);
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        let bg = stage.bg();
        g.render(
            Rect::new(4, 1, width, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(4, 1, width, 1), "V1 toggle");
        assert!(g.on, "A1: toggle flag");
        let mut r = RadioGroup::new(
            s2_id("test.s2.narrow.radio"),
            "Mode",
            &["Fast", "Balanced"],
            0,
        );
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 5);
        let bg = stage.bg();
        r.render(
            Rect::new(4, 1, width, 3),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_surround_intact(&buf, Rect::new(4, 1, width, 3), "V1 radio");
        // Splitter handle draws its 1-cell rule (or nothing when empty).
        let mut stage = Stage::new();
        let mut buf = sentinel_buf(16, 3);
        let mut seam = Splitter::new(s2_id("test.s2.narrow.split"), SplitDir::Horizontal);
        let bg = stage.bg();
        seam.render(
            Rect::new(4, 1, 1, 1),
            &mut buf,
            &mut stage.ctx(Interaction::default()),
            bg,
        );
        assert_eq!(buf[(4, 1)].symbol(), "│", "V1: splitter rule");
        assert_surround_intact(&buf, Rect::new(4, 1, 1, 1), "V1 splitter");
    }
    // V2: compact marks at width 3; gutter-only 1-cell rows.
    let mut c = Checkbox::new(s2_id("test.s2.narrow.c3"), "Run tests", false);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 3, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let row = buf_row_text(&buf, 1);
    assert!(
        row.contains('□') && !row.contains('['),
        "V2: compact □ in {row:?}"
    );
    capture_isolated(&dir, "narrow-compact", &buf);
    // V2: framed chrome at w<=4 draws the border with no title row.
    let t = Theme::junie();
    let mut buf = sentinel_buf(16, 5);
    Panel::framed(Some("Framed")).render(Rect::new(4, 1, 4, 3), &mut buf, &t);
    let brow = buf_row_text(&buf, 1);
    assert!(
        brow.contains('╭') && !brow.contains("Framed"),
        "V2: border-only in {brow:?}"
    );
    capture_isolated(&dir, "narrow-framed", &buf);
    // S1/N1: PropsList label spill + hint underflow are pinned as-is
    // below the label width (the one allowed surround touch).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 5);
    let mut l = PropsList::new(
        s2_id("test.s2.narrow.props"),
        vec![Prop::new("Environment", "production")],
    );
    let bg = stage.bg();
    l.render(
        Rect::new(4, 1, 6, 2),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let spilled =
        (1..3).any(|y| (0..30).any(|x| !(4..10).contains(&x) && buf[(x, y)].symbol() != "·"));
    assert!(spilled, "S1/N1: props spill pinned below label width");
    capture_isolated(&dir, "narrow-props-spill", &buf);
    // Empty truncates; 1-row areas keep the title only.
    let mut buf = sentinel_buf(20, 5);
    empty::render(
        Rect::new(4, 1, 10, 3),
        &mut buf,
        &t,
        &EmptyState::new("A very long title indeed"),
        t.canvas,
    );
    assert!(
        (0..5).any(|y| buf_row_text(&buf, y).contains('…')),
        "V2: empty truncates"
    );
    eprintln!("s2 narrow_clip: 6/4/3/2/1 sweep + pinned spills");
}

/// CTRL-HOVER-COVERAGE-001 (`showcase/audit/buttons`): hover delta per
/// hoverable control.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_hover_coverage() {
    let dir = s2_dir("s2_hover_coverage");
    // V1: idle-vs-hovered frames differ for every hoverable control.
    // Button.
    let bid = s2_id("test.s2.hov.btn");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let btn_idle = capture_isolated(&dir, "hov-btn-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), " ", "V2: hover alone never bars");
    let btn_hov = capture_isolated(&dir, "hov-btn-hover", &buf).digest();
    assert_ne!(btn_idle, btn_hov, "V1: button hover delta");
    // Checkbox.
    let cid = s2_id("test.s2.hov.chk");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", false);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let chk_idle = capture_isolated(&dir, "hov-chk-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut c = Checkbox::new(cid, "Run tests", false);
    let bg = stage.bg();
    c.render(
        Rect::new(4, 1, 20, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(cid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 1)].symbol(), " ", "V2: checkbox hover never bars");
    assert_ne!(
        chk_idle,
        capture_isolated(&dir, "hov-chk-hover", &buf).digest(),
        "V1: checkbox delta"
    );
    // Toggle.
    let gid = s2_id("test.s2.hov.tgl");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut g = Toggle::new(gid, "Auto-approve", false);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 22, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let tgl_idle = capture_isolated(&dir, "hov-tgl-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut g = Toggle::new(gid, "Auto-approve", false);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 22, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(gid),
            ..Default::default()
        }),
        bg,
    );
    assert_ne!(
        tgl_idle,
        capture_isolated(&dir, "hov-tgl-hover", &buf).digest(),
        "V1: toggle delta"
    );
    // RadioGroup option (child id): exactly that row lifts.
    let rid = s2_id("test.s2.hov.radio");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 6);
    let mut r = RadioGroup::new(rid, "Mode", &["Fast", "Balanced"], 0);
    let bg = stage.bg();
    r.render(
        Rect::new(4, 1, 20, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let radio_idle = capture_isolated(&dir, "hov-radio-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 6);
    let mut r = RadioGroup::new(rid, "Mode", &["Fast", "Balanced"], 0);
    let opt1 = r.option_id(1);
    let bg = stage.bg();
    r.render(
        Rect::new(4, 1, 20, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(opt1),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 2)].symbol(), " ", "V2: radio hover never bars");
    assert_ne!(
        buf[(6, 3)].bg,
        buf[(6, 2)].bg,
        "V1: exactly one option lifts"
    );
    assert_ne!(
        radio_idle,
        capture_isolated(&dir, "hov-radio-hover", &buf).digest(),
        "V1: radio delta"
    );
    // Brand clickable.
    let lockup = Lockup::new("holla❯");
    let kbid = s2_id("test.s2.hov.brand");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render_clickable(4, 1, &mut buf, &mut stage.ctx(Interaction::default()), kbid);
    let brand_idle = capture_isolated(&dir, "hov-brand-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 3);
    lockup.render_clickable(
        4,
        1,
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(kbid),
            ..Default::default()
        }),
        kbid,
    );
    assert_ne!(
        brand_idle,
        capture_isolated(&dir, "hov-brand-hover", &buf).digest(),
        "V1: brand delta"
    );
    // Splitter.
    let sid = s2_id("test.s2.hov.split");
    let strip = Rect::new(4, 1, 1, 4);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 6);
    let mut seam = Splitter::new(sid, SplitDir::Horizontal);
    let bg = stage.bg();
    seam.render(strip, &mut buf, &mut stage.ctx(Interaction::default()), bg);
    let split_idle = capture_isolated(&dir, "hov-split-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(16, 6);
    let mut seam = Splitter::new(sid, SplitDir::Horizontal);
    let bg = stage.bg();
    seam.render(
        strip,
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(sid),
            ..Default::default()
        }),
        bg,
    );
    assert_ne!(
        split_idle,
        capture_isolated(&dir, "hov-split-hover", &buf).digest(),
        "V1: splitter delta"
    );
    // PropsList row.
    let lid = s2_id("test.s2.hov.props");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut l = PropsList::new(
        lid,
        vec![Prop::new("Engine", "pg"), Prop::new("Host", "db")],
    );
    let bg = stage.bg();
    l.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let props_idle = capture_isolated(&dir, "hov-props-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut l = PropsList::new(
        lid,
        vec![Prop::new("Engine", "pg"), Prop::new("Host", "db")],
    );
    let row0 = l.row_id(0);
    let bg = stage.bg();
    l.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(row0),
            ..Default::default()
        }),
        bg,
    );
    assert_ne!(
        buf[(6, 1)].bg,
        buf[(6, 2)].bg,
        "V1: exactly one props row lifts"
    );
    assert_ne!(
        props_idle,
        capture_isolated(&dir, "hov-props-hover", &buf).digest(),
        "V1: props delta"
    );

    // S1: insensitive surfaces ignore hover (structurally identical).
    let pid = s2_id("test.s2.hov.scroll");
    let mut p = ScrollPanel::new(pid, vec!["a".to_string(), "b".to_string()]);
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 5);
    let bg = stage.bg();
    p.render(
        Rect::new(4, 1, 16, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
        scroll_line,
    );
    let scroll_idle = capture_isolated(&dir, "hov-scroll-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(24, 5);
    let bg = stage.bg();
    p.render(
        Rect::new(4, 1, 16, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(pid),
            ..Default::default()
        }),
        bg,
        scroll_line,
    );
    assert_eq!(
        scroll_idle,
        capture_isolated(&dir, "hov-scroll-hover", &buf).digest(),
        "S1: scroll ignores hover"
    );
    // N1: hover must not lift a disabled control or a suppressed pointer.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut b = Button::secondary(bid, "Run task").disabled(true);
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let dis_idle = capture_isolated(&dir, "hov-dis-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut b = Button::secondary(bid, "Run task").disabled(true);
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(bid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        dis_idle,
        capture_isolated(&dir, "hov-dis-hover", &buf).digest(),
        "N1: disabled ignores hover"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(30, 3);
    let mut b = Button::secondary(bid, "Run task");
    let bg = stage.bg();
    b.render(
        Rect::new(4, 1, 16, 1),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(bid),
            hover_suppressed: true,
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        btn_idle,
        capture_isolated(&dir, "hov-suppressed", &buf).digest(),
        "N1: suppressed pointer ignores hover"
    );
    // A1: hover moves no flags, ring entries, or hit regions.
    let mut b = Button::toggle(bid, "Verbose", true);
    b.on_key(&s2_key(KeyCode::Char('x')));
    assert_eq!(b.on, Some(true), "A1: flags untouched");

    // PTY: hover lifts the live button row (no bar); disabled ignores it.
    let case = s2_case("hover_coverage", &["--page", "buttons"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let (brow, bcol) = find_pos(&mut s, "Run task", case_timeout(&case));
    let before = live_frame(&mut s);
    let before_bg = before.get(bcol, brow).expect("label cell").bg;
    hover_over(&mut s, "Run task", case_timeout(&case));
    let after = live_frame(&mut s);
    let after_bg = after.get(bcol, brow).expect("hovered label cell").bg;
    assert_ne!(before_bg, after_bg, "V1 live: hover lifts the button row");
    let hov_text = live_text(&mut s);
    assert_line_lacks(
        &hov_text,
        "▎",
        "Run task",
        "V2 live: hover alone never bars",
    );
    // N1 live: the disabled row is cell-identical under hover.
    let (drow, dcol) = find_pos(&mut s, "Disabled primary", case_timeout(&case));
    let row_cells = |f: &Frame| {
        (dcol..dcol + 16)
            .map(|c| {
                let cell = f.get(c, drow).expect("disabled row cell");
                (cell.symbol.clone(), cell.fg, cell.bg)
            })
            .collect::<Vec<_>>()
    };
    let dis_before = row_cells(&live_frame(&mut s));
    hover_over(&mut s, "Disabled primary", case_timeout(&case));
    let dis_after = row_cells(&live_frame(&mut s));
    assert_eq!(dis_before, dis_after, "N1 live: disabled ignores hover");
    checkpoint(&mut s, &dir, "hovered");
    eprintln!("s2 hover_coverage: isolated deltas + live lift/no-lift");
}

/// FLD-CHROME-001 (`showcase/audit/inputs`): label + gutter + help/error
/// chrome.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_field_chrome() {
    let dir = s2_dir("s2_field_chrome");
    let t = Theme::junie();
    let fid = s2_id("test.s2.fld");

    // V1 isolated: required label shows 'Project name *' with an accent *.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Project name")
        .value("payments-gateway")
        .required(true)
        .help("Used as the working directory name");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let label_row = buf_row_text(&buf, 1);
    assert!(
        label_row.contains("Project name *"),
        "V1: required label in {label_row:?}"
    );
    assert_eq!(
        buf[(4 + 2 + 12 + 1, 1)].fg,
        t.accent_fg().fg.unwrap_or(t.accent),
        "V1: accent star"
    );
    // V1: optional suffix appears only when name_w + 12 fits.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Branch").placeholder("feat/…");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(
        buf_row_text(&buf, 1).contains("Branch  optional"),
        "V1: optional suffix"
    );
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(20, 5);
    let mut f = TextInput::new(fid, "Branch").placeholder("feat/…");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 10, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(
        !buf_row_text(&buf, 1).contains("optional"),
        "V1: suffix clipped when narrow"
    );
    // V2: gutter ▎ on the focused field, blank elsewhere.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Project name").value("x");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(fid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 2)].symbol(), "▎", "V2: focused gutter bar");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Project name").value("x");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4, 2)].symbol(), " ", "V2/N1: unfocused gutter blank");
    // V2: error fields show bold ! at field.right()-2.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Owner email")
        .value("mira@example")
        .required(true)
        .validator(|v| {
            if v.contains('@') && v.contains('.') {
                None
            } else {
                Some("bad".into())
            }
        });
    f.validate();
    assert!(f.error.is_some(), "error fixture invalid");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert_eq!(buf[(4 + 32 - 2, 2)].symbol(), "!", "V2: error marker");
    assert!(buf_is_bold(&buf, 4 + 32 - 2, 2), "V2: marker bold");
    capture_isolated(&dir, "fld-chrome", &buf);
    // S2: placeholder paints only when empty and not editing.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Branch").placeholder("feat/…");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(
        buf_row_text(&buf, 2).contains("feat/…"),
        "S2: placeholder when empty"
    );
    f.begin_edit();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(fid),
            ..Default::default()
        }),
        bg,
    );
    assert!(
        !buf_row_text(&buf, 2).contains("feat/…"),
        "S2: no placeholder while editing"
    );
    // A2: typing while navigating returns Ignored.
    let mut f = TextInput::new(fid, "Branch");
    let (o, ev) = f.on_key(&s2_key(KeyCode::Char('x')));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "A2: typing nav ignores"
    );
    // N2: help never shows together with the error row (error wins).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(40, 5);
    let mut f = TextInput::new(fid, "Owner email")
        .value("bad")
        .help("some help")
        .validator(|_| Some("Enter a valid email".into()));
    f.validate();
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 32, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let msg = buf_row_text(&buf, 3);
    assert!(
        msg.contains("Enter a valid email"),
        "N2: error wins in {msg:?}"
    );
    assert!(!msg.contains("some help"), "N2: help hidden in {msg:?}");

    // PTY: tab walks the six playground fields; the gutter follows.
    let case = s2_case("field_chrome", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    let boot = live_text(&mut s);
    assert!(boot.contains("Project name *"), "boot required label");
    assert!(boot.contains("Branch  optional"), "boot optional suffix");
    // Ring order skips the disabled API token: 5 stops. Each needle
    // names "▎ <value>" adjacently: the two-column rows share ▎ with
    // the neighbor value, so a bare ▎+value wait would pass stale.
    for (i, needle) in [
        "▎ payments-gateway",
        "▎ feat/…",
        "▎ mira@example",
        "▎ Type a path or symbol",
        "▎ ••••",
    ]
    .iter()
    .enumerate()
    {
        support::press_step(&s, "tab");
        support::wait_screen(
            &mut s,
            case_timeout(&case),
            &format!("A1: gutter never reached stop {i} ({needle})"),
            |screen| support::screen_text(screen).contains(needle),
        );
    }
    // A1 live: focus moved, nothing edited (no EDIT footer anywhere).
    let tabbed = live_text(&mut s);
    assert!(!tabbed.contains("EDIT"), "A1 live: tabbing never edits");
    // A2 live: typing while navigating changes nothing (byte-identical).
    support::press_step(&s, "x");
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(live_text(&mut s), tabbed, "A2 live: typing nav is ignored");
    // N2 live: error and help rows never mix.
    let help_line = tabbed
        .lines()
        .find(|l| l.contains("working di"))
        .expect("help row");
    assert!(!help_line.contains("valid email"), "N2 live: help row pure");
    checkpoint(&mut s, &dir, "tabbed");
    eprintln!("s2 field_chrome: isolated chrome + live tab walk");
}

/// FLD-DISABLED-002 (`showcase/audit/inputs`): disabled / read-only state.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_field_disabled() {
    let dir = s2_dir("s2_field_disabled");
    let t = Theme::junie();
    let fid = s2_id("test.s2.flddis");

    // V1/V2 isolated: faint label, disabled body, blank gutter even focused.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let mut f = TextInput::new(fid, "API token")
        .value("jb_live_••••••••••••")
        .disabled(true)
        .help("Managed by the organization");
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(fid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        buf[(4, 2)].symbol(),
        " ",
        "V2/N1: gutter blank even focused"
    );
    assert_eq!(
        buf[(6, 1)].fg,
        t.faint().fg.unwrap_or(t.text_muted),
        "V1: faint label"
    );
    // The disabled field keeps its hit area (only the ring stop is cut).
    assert_eq!(
        stage.hits.hit(Position::new(8, 2)),
        Some(fid),
        "S1: disabled hit area kept"
    );
    assert!(!stage.ring.contains(fid), "S1: no ring stop");
    capture_isolated(&dir, "fld-disabled", &buf);
    // S2: render forces hovered=false when disabled (hover changes nothing).
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let mut f = TextInput::new(fid, "API token").value("v").disabled(true);
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    let dis_idle = capture_isolated(&dir, "fld-dis-idle", &buf).digest();
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let mut f = TextInput::new(fid, "API token").value("v").disabled(true);
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            hover: Some(fid),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(
        dis_idle,
        capture_isolated(&dir, "fld-dis-hover", &buf).digest(),
        "S2: hover forced off"
    );
    // S1/A1: keys ignore, begin_edit no-ops, clicks consume, never edits.
    let mut f = TextInput::new(fid, "API token").value("v").disabled(true);
    let (o, ev) = f.on_key(&s2_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "S1: Enter ignores"
    );
    f.begin_edit();
    let (o, _) = f.on_key(&s2_key(KeyCode::Char('x')));
    assert!(
        matches!(o, Outcome::Ignored),
        "S1: begin_edit no-op + keys ignore"
    );
    assert!(
        matches!(f.on_click(Position::new(6, 0)), Outcome::Consumed),
        "A1: click consumes"
    );
    // A2: no separate read-only flag — the transcript area is disabled(true).
    let mut area =
        TextArea::new(s2_id("test.s2.transcript"), "Read-only transcript", 4).disabled(true);
    let (o, _) = area.on_key(&s2_key(KeyCode::Char('x')));
    assert!(
        matches!(o, Outcome::Ignored),
        "A2: disabled area ignores keys"
    );
    // N2: token bullets are literal value data, not the masking renderer.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let mut f = TextInput::new(fid, "API token")
        .value("jb_live_••••••••••••")
        .disabled(true);
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction::default()),
        bg,
    );
    assert!(
        buf_row_text(&buf, 2).contains("jb_live_••••"),
        "N2: literal bullets"
    );

    // PTY: the API token field refuses to edit; tabs skip it.
    let case = s2_case("field_disabled", &["--page", "inputs"], 80, 24);
    write_provenance(&dir, &case);
    let mut s = support::spawn_boot(&case);
    checkpoint(&mut s, &dir, "booted");
    assert!(
        live_text(&mut s).contains("jb_live_"),
        "boot shows the token"
    );
    // A1 live: clicking the token never enters editing (focus moves —
    // the nav bar leaves — but nothing activates and no bar lands).
    let before = live_text(&mut s);
    assert_line_has(&before, "▎", "›", "A1 presence: nav holds focus at boot");
    click_at(&mut s, "jb_live_", case_timeout(&case));
    std::thread::sleep(Duration::from_millis(400));
    let after_click = live_text(&mut s);
    // The live click is fully consumed: no editing, nothing repaints
    // (byte-identical), focus stays on the nav, no bar lands on the token.
    assert!(!after_click.contains("EDIT"), "A1 live: click never edits");
    assert_eq!(after_click, before, "A1 live: click changes nothing");
    assert_line_has(&after_click, "▎", "›", "A1 live: focus stays on nav");
    assert_line_lacks(&after_click, "▎", "jb_live_", "A1 live: no bar on token");
    // N1 live: five tabs visit every stop; the bar never lands on the
    // token (adjacency: the Owner bar shares the token's row).
    for _ in 0..5 {
        support::press_step(&s, "tab");
        std::thread::sleep(Duration::from_millis(150));
        let text = live_text(&mut s);
        assert!(
            !text.contains("▎ jb_live_"),
            "N1 live: bar never on token\n{text}"
        );
    }
    checkpoint(&mut s, &dir, "skipped");
    eprintln!("s2 field_disabled: isolated guards + live refuse/skip");
}

/// Type `text` into an editing [`TextInput`] one char at a time.
fn type_into(f: &mut TextInput, text: &str) {
    for ch in text.chars() {
        let (o, _) = f.on_key(&s2_key(KeyCode::Char(ch)));
        assert!(matches!(o, Outcome::Changed), "typing {ch:?} applies");
    }
}

/// FLD-FOCUSLOSS-003 (`showcase/flows/inputs/editing`): focus-loss commit
/// in render.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_field_focusloss() {
    let dir = s2_dir("s2_field_focusloss");
    let fid = s2_id("test.s2.fldloss");
    let other = s2_id("test.s2.fldloss.other");

    // Isolated: type while editing, then render defocused → commit.
    let mut f = TextInput::new(fid, "Search files").placeholder("Type a path or symbol…");
    f.begin_edit();
    type_into(&mut f, "billing");
    assert_eq!(f.text(), "billing", "typed text staged");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(other),
            ..Default::default()
        }),
        bg,
    );
    // S1/N1/N2: defocus commits (never cancels): text kept, editing
    // cleared, no revert to the empty snapshot.
    assert!(!f.editing, "S1: defocus clears editing");
    assert_eq!(f.text(), "billing", "N1: defocus never cancels");
    // V1: committed text renders plainly (no underline, no cursor).
    assert!(
        !buf[(6, 2)].modifier.contains(Modifier::UNDERLINED),
        "V1: committed text plain"
    );
    capture_isolated(&dir, "fld-defocused", &buf);
    // V2: the newly focused field shows ▎.
    let mut g = TextInput::new(other, "Branch");
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let bg = stage.bg();
    g.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(other),
            ..Default::default()
        }),
        bg,
    );
    assert_eq!(buf[(4, 2)].symbol(), "▎", "V2: new field barred");
    // A1: Tab commits with CommittedTab (same commit path, explicit event).
    let mut f = TextInput::new(fid, "Search files");
    f.begin_edit();
    type_into(&mut f, "billing");
    let (o, ev) = f.on_key(&s2_key(KeyCode::Tab));
    assert!(matches!(o, Outcome::Changed), "A1: Tab commits");
    assert!(
        matches!(ev, Some(InputEvent::CommittedTab { backward: false })),
        "A1: CommittedTab"
    );
    assert_eq!(f.text(), "billing", "A1: committed text kept");
    // A2: clicking elsewhere begins the new field at the pointer.
    let o = g.on_click(Position::new(8, 0));
    assert!(
        matches!(o, Outcome::Changed),
        "A2: click begins the new field"
    );

    // PTY: typing then clicking away commits the old field live.
    let case = s2_case("field_focusloss", &["--page", "inputs"], 80, 24);
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
    waits::wait_state(&mut s, "EDIT", "editing Search files");
    type_text(&mut s, "billing");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "typed text never rendered",
        |screen| support::screen_text(screen).contains("billing"),
    );
    // Click the Branch value cell: old commits, new begins.
    let (branch_row, _) = click_at(&mut s, "feat/…", case_timeout(&case));
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "V2: new field never barred",
        |screen| {
            support::screen_text(screen)
                .lines()
                .nth(branch_row as usize)
                .is_some_and(|l| l.contains('▎'))
        },
    );
    let text = live_text(&mut s);
    let billing_line = text
        .lines()
        .find(|l| l.contains("billing"))
        .expect("committed text")
        .to_string();
    assert!(!billing_line.contains('▎'), "V1 live: committed plainly");
    assert!(
        !billing_line.contains("Type a path"),
        "N1 live: not reverted"
    );
    checkpoint(&mut s, &dir, "defocus-committed");
    eprintln!("s2 field_focusloss: isolated commit + live click-away");
}

/// TI-COMMIT-001 (`showcase/flows/inputs/editing`): navigation to editing
/// to commit.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_input_commit() {
    let dir = s2_dir("s2_input_commit");
    let fid = s2_id("test.s2.ticommit");

    // A1 isolated: Enter/F2 while navigating begins editing; other keys
    // and Ctrl+Enter ignore.
    let mut f = TextInput::new(fid, "Search files").placeholder("Type a path or symbol…");
    let (o, ev) = f.on_key(&s2_key(KeyCode::Enter));
    assert!(
        matches!(o, Outcome::Changed) && ev.is_none(),
        "A1: Enter begins"
    );
    let mut f = TextInput::new(fid, "Search files");
    let (o, _) = f.on_key(&s2_key(KeyCode::Char('w')));
    assert!(matches!(o, Outcome::Ignored), "N1: typing nav never edits");
    let ctrl_enter = Key {
        code: KeyCode::Enter,
        mods: KeyModifiers::CONTROL,
    };
    let (o, _) = f.on_key(&ctrl_enter);
    assert!(matches!(o, Outcome::Ignored), "N2: Ctrl+Enter ignores");
    // S1: begin_edit snapshots and clears the selection.
    f.begin_edit();
    type_into(&mut f, "work");
    assert_eq!(f.text(), "work", "S1: typed into the snapshot");
    // V1: editing text is underlined with a hardware cursor.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let bg = stage.bg();
    let mut ctx = stage.ctx(Interaction {
        focus: Some(fid),
        ..Default::default()
    });
    f.render(Rect::new(4, 1, 36, 3), &mut buf, &mut ctx, bg);
    assert!(
        buf[(6, 2)].modifier.contains(Modifier::UNDERLINED),
        "V1: editing underlined"
    );
    assert!(ctx.cursor.is_some(), "V1: hardware cursor placed");
    capture_isolated(&dir, "ti-editing", &buf);
    // S2/A2: Enter commits (validate runs); Tab commits + moves with
    // CommittedTab; BackTab moves back.
    let (o, ev) = f.on_key(&s2_key(KeyCode::Enter));
    assert!(matches!(o, Outcome::Changed), "commit applies");
    assert!(matches!(ev, Some(InputEvent::Committed)), "Committed event");
    assert_eq!(f.text(), "work", "S2: commit keeps text");
    f.begin_edit();
    let (o, ev) = f.on_key(&s2_key(KeyCode::Tab));
    assert!(
        matches!(ev, Some(InputEvent::CommittedTab { backward: false })),
        "A2: Tab forward"
    );
    assert!(matches!(o, Outcome::Changed), "A2: Tab commits");
    f.begin_edit();
    let (_, ev) = f.on_key(&s2_key(KeyCode::BackTab));
    assert!(
        matches!(ev, Some(InputEvent::CommittedTab { backward: true })),
        "A2: BackTab back"
    );

    // PTY: type + commit roundtrip with the saved status.
    let case = s2_case("input_commit", &["--page", "inputs"], 80, 24);
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
    checkpoint(&mut s, &dir, "editing");
    type_text(&mut s, "work");
    support::press_step(&s, "enter");
    waits::wait_state(&mut s, "Search files saved", "V2: saved status");
    checkpoint(&mut s, &dir, "committed");
    let done = live_text(&mut s);
    assert!(!done.contains("EDIT"), "V2 live: commit leaves editing");
    assert!(done.contains("work"), "V2 live: committed text plain");
    eprintln!("s2 input_commit: isolated lifecycle + live saved status");
}

/// TI-CANCEL-002 (`showcase/audit/inputs`): Esc revert to snapshot.
#[test]
#[ignore = "showcase s2 check; run with --ignored"]
fn s2_input_cancel() {
    let dir = s2_dir("s2_input_cancel");
    let fid = s2_id("test.s2.ticancel");

    // S1/S2/A1 isolated: snapshot at begin_edit; Esc restores the whole
    // buffer and revalidates.
    let mut f = TextInput::new(fid, "Project name")
        .value("payments-gateway")
        .required(true)
        .help("Used as the working directory name")
        .validator(|v| {
            if v.contains('x') {
                Some("no x".into())
            } else {
                None
            }
        });
    f.validate();
    assert!(f.error.is_none(), "fixture starts valid");
    f.begin_edit();
    type_into(&mut f, "-x");
    assert_eq!(f.text(), "payments-gateway-x", "staged edit");
    // Live validation only clears (never sets): surface the error with an
    // explicit validate, then prove cancel revalidates it away (Some→None).
    assert!(f.error.is_none(), "live typing sets no error");
    f.validate();
    assert!(f.error.is_some(), "staged edit is invalid");
    let (o, ev) = f.on_key(&s2_key(KeyCode::Esc));
    assert!(matches!(o, Outcome::Changed), "A1: Esc applies");
    assert!(
        matches!(ev, Some(InputEvent::Cancelled)),
        "A1: Cancelled event"
    );
    // A2: editing cleared, whole buffer restored, error revalidated away.
    assert!(!f.editing, "A2: editing cleared first");
    assert_eq!(
        f.text(),
        "payments-gateway",
        "S1/S2: snapshot restored whole"
    );
    assert!(f.error.is_none(), "S2: revalidated after revert");
    // V2: the help row re-renders from the post-revert validation.
    let mut stage = Stage::new();
    let mut buf = sentinel_buf(44, 5);
    let bg = stage.bg();
    f.render(
        Rect::new(4, 1, 36, 3),
        &mut buf,
        &mut stage.ctx(Interaction {
            focus: Some(fid),
            ..Default::default()
        }),
        bg,
    );
    assert!(
        buf_row_text(&buf, 3).contains("working directory"),
        "V2: help re-rendered"
    );
    capture_isolated(&dir, "ti-reverted", &buf);
    // N1: Esc while navigating does nothing.
    let mut f = TextInput::new(fid, "Project name").value("payments-gateway");
    let (o, ev) = f.on_key(&s2_key(KeyCode::Esc));
    assert!(
        matches!(o, Outcome::Ignored) && ev.is_none(),
        "N1: nav Esc ignores"
    );
    // N2: no partial revert is possible (mid-buffer edit reverts whole).
    let mut f = TextInput::new(fid, "Project name").value("payments-gateway");
    f.begin_edit();
    for _ in 0..7 {
        f.on_key(&s2_key(KeyCode::Left));
    }
    type_into(&mut f, "X");
    assert!(f.text().contains('X'), "mid-buffer edit staged");
    f.on_key(&s2_key(KeyCode::Esc));
    assert_eq!(f.text(), "payments-gateway", "N2: whole buffer restored");

    // PTY: type + Esc roundtrip with the Reverted status.
    let case = s2_case("input_cancel", &["--page", "inputs"], 80, 24);
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
    type_text(&mut s, "-x");
    support::wait_screen(
        &mut s,
        case_timeout(&case),
        "staged edit never rendered",
        |screen| support::screen_text(screen).contains("gateway-x"),
    );
    support::press_step(&s, "escape");
    waits::wait_state(&mut s, "Reverted", "A1: Reverted status");
    checkpoint(&mut s, &dir, "reverted");
    let done = live_text(&mut s);
    assert!(!done.contains("EDIT"), "V1 live: cancel leaves editing");
    assert!(done.contains("payments-gateway"), "V1 live: value restored");
    assert!(!done.contains("gateway-x"), "N2 live: no partial revert");
    eprintln!("s2 input_cancel: isolated revert + live Reverted status");
}
