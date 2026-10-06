//! Live busy-button regressions: the PTY spinner-cycle test and the
//! row-geometry test.
//!
//! Candidate port of `tests/harness/tests/visual_baseline/button_busy_live.rs`
//! at `origin/visual-baseline` (`3c5833cb1`). The headless pin this module
//! extends lives in the sibling `button_busy_frames` test target
//! (`tests/button_busy_frames.rs`): one buffer render per tick, busy+checked
//! clipping. This module proves the same contract through the two paths
//! those renders cannot reach:
//!
//! * `busy_spinner_cycle_advances_live` drives the REAL showcase binary in a
//!   PTY via tuiscotti, clicks `Start long job` (`ButtonsPage` index 8),
//!   and samples the marker cell across the whole 28-tick busy window. The
//!   reference reads the glyph from the *current* render tick
//!   (`spinner_frame(ctx.interaction.tick)`, `button.rs`), so consecutive
//!   frames must walk the 10-frame braille cycle forward with no repeats,
//!   no jumps, and full coverage — then the window must close with
//!   `Long job finished ✓` and the plain label restored. A frozen glyph, a
//!   subset cycle, or a stuck-busy button all fail.
//! * `busy_checked_row_geometry_shared_marker` renders the REAL production
//!   [`Button`](termrock::Button) as a busy+checked row leader with two
//!   subsequent controls through the reference [`row_layout`] chaining,
//!   captured via the component path ([`capture_buffer`]). The reference
//!   shares one 2-cell marker between busy and checked (`width()` counts the
//!   marker once), so followers must sit exactly where the idle row puts
//!   them; the clipping ladder then narrows the row until followers drop
//!   out and the leader itself clips.
//!
//! Evidence: every test writes its captured frames and a human-readable
//! record under a unique run dir in `target/tuiscotti-actuals/` (gitignored
//! scratch, like `target/tuiscotti/`). The dir is printed via `eprintln`.
//!
//! No snapshot gating here on purpose: these are contract tests — headless
//! in the default nextest run, plus one ignored PTY probe. This file
//! deliberately declares no ported-matrix captures.
//!
//! Port notes (setup only; assertions are the reference assertions,
//! unweakened):
//!
//! * The live probe spawns through [`support::spawn`] with a suite [`Case`](support::Case)
//!   (the suite convention) instead of a hand-built `Tui` argv; the PTY
//!   mechanics and boot needle are unchanged.
//! * The suite [`Session`](support::Session) wrapper returns a canonical
//!   [`Frame`](tuiscotti::Frame) straight from `snapshot()`, so the sampler
//!   reads cells and window text off the frame instead of converting a
//!   screen first.
//! * The geometry test drives the candidate `Runtime` + `Stub` +
//!   `draw_scene` (one `Input::Tick` per animation tick, as in
//!   `tests/button_busy_frames.rs`); logical widths come from
//!   `Button::measure` and rendered widths from the `draw` return.
//! * [`row_layout`] has no candidate equivalent (`layout::distribute`
//!   pre-subtracts every gap and re-shares the remainder, which diverges
//!   from the reference chaining under narrowing), so the 8-line reference
//!   helper is reproduced verbatim; the widths it chains are the
//!   candidate's own measured widths.
//! * [`capture_buffer`] mirrors `tuiscotti::ratatui::from_buffer`
//!   cell-for-cell (wide cells, continuations, style mapping) because
//!   tuiscotti pins ratatui 0.29 while the candidate buffer is
//!   ratatui-core 0.1, so `from_buffer` cannot take it directly.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use termrock::runtime::stub::{Stub, deliver};
use termrock::{
    Buffer, Button, Color as RColor, Constraints, Id, Input, Modifier, Position, Rect, Runtime,
    Status, Theme, Variant,
};
use tuiscotti::{Cell as TCell, Color as TColor, Frame, Mods, Provenance, Rgb, UnderlineStyle};

use crate::support;

/// Label of the real showcase busy button (`ButtonsPage` index 8).
const LABEL: &str = "Start long job";
/// Display width of [`LABEL`] (all ASCII).
const LABEL_W: u16 = 14;
/// Full busy-button width: label + 2 padding + 2 shared marker cells.
const FULL_W: u16 = LABEL_W + 2 + 2;
/// The reference ten-frame braille cycle (`spinner_frame`, `progress.rs`):
/// the oracle every marker glyph below is compared against.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
/// Reference `spinner_frame(tick) = SPINNER[tick % 10]`.
fn spinner_frame(tick: u64) -> &'static str {
    SPINNER[(tick % SPINNER.len() as u64) as usize]
}
/// Showcase boot needle (same as the ported matrix).
const BOOT: &str = "Junie Design system";
/// Live probe geometry: canonical wide combo, footer + last-line visible.
const PTY_COLS: u16 = 120;
const PTY_ROWS: u16 = 40;
/// Reference busy window: 28 render ticks at 80 ms (`LONG_JOB_TICKS`).
const BUSY_TICKS: u64 = 28;
/// Live-sample cadence: well under the 80 ms tick, like the state waits.
const POLL: Duration = Duration::from_millis(25);

/// Unique evidence dir for this process run (shared by the tests in this
/// file; each test writes distinctly-named files).
fn evidence_dir() -> &'static Path {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/tuiscotti-actuals")
            .join(format!("button_busy_live_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("create evidence dir {}: {e}", dir.display()));
        eprintln!("button_busy_live evidence: {}", dir.display());
        dir
    })
}

fn write_evidence(name: &str, bytes: &[u8]) {
    let path = evidence_dir().join(name);
    std::fs::write(&path, bytes)
        .unwrap_or_else(|e| panic!("write evidence {}: {e}", path.display()));
}

fn spinner_index(glyph: &str) -> Option<usize> {
    SPINNER.into_iter().position(|s| s == glyph)
}

/// First occurrence of `needle` as `(row, col)`, waiting until it appears.
/// Mirrors the pointer group's hand-off (clicks take `(col, row)`).
fn find(s: &mut support::Session, needle: &str, timeout: Duration) -> (u16, u16) {
    let obs = support::wait_screen(
        s,
        timeout,
        &format!("`{needle}` never appeared"),
        |screen| support::screen_find(screen, needle).is_some(),
    );
    support::screen_find(&obs.screen, needle).expect("wait passed with the needle on screen")
}

/// Consecutive spinner frames from the real binary must walk the braille
/// cycle forward across the whole busy window, then restore the label.
#[test]
#[ignore = "pty probe; run with --ignored"]
fn busy_spinner_cycle_advances_live() {
    assert_eq!(SPINNER.len(), 10, "reference cycle length");
    let case = support::Case::new(
        "button-busy-live",
        support::SHOWCASE,
        &["--page", "buttons"],
        PTY_COLS,
        PTY_ROWS,
        support::Color::Truecolor,
        BOOT,
    );
    let timeout = Duration::from_millis(support::TIMEOUT_MS);
    let mut s: support::Session = support::spawn(&case);
    support::wait_screen(&mut s, timeout, "showcase never booted", |screen| {
        support::screen_text(screen).contains(BOOT)
    });

    // Pin the button geometry before the click. The idle label starts at
    // the button's text cell (gutter + label); going busy inserts the
    // 2-cell marker there and pushes the label right — so the busy marker
    // cell is exactly the idle label start.
    let (row, label_col) = find(&mut s, LABEL, timeout);
    assert!(label_col >= 1, "label at col 0 has no gutter");
    let marker_col = label_col;

    s.click(label_col + 1, row).expect("click the busy button");
    support::wait_screen(
        &mut s,
        timeout,
        "`Working…` never appeared — click did not start the job",
        |screen| support::screen_text(screen).contains("Working…"),
    );

    // Sample the marker cell until the window closes (or the bound hits):
    // every change is one observed frame transition.
    let start = Instant::now();
    let bound = Duration::from_secs(10);
    let mut changes: Vec<(u128, String, Frame)> = Vec::new();
    let mut last = String::new();
    let mut saw_finish = false;
    let mut tail = 0u32;
    loop {
        let frame = s
            .snapshot()
            .unwrap_or_else(|e| panic!("live sample failed: {e:#}"));
        let glyph = frame
            .get(marker_col, row)
            .unwrap_or_else(|| panic!("marker cell ({marker_col}, {row}) vanished"))
            .symbol
            .clone();
        if glyph != last {
            changes.push((start.elapsed().as_millis(), glyph.clone(), frame.clone()));
            last = glyph;
        }
        if frame.text().contains("Long job finished ✓") {
            saw_finish = true;
            tail += 1;
            if tail >= 3 {
                break;
            }
        }
        if start.elapsed() >= bound {
            break;
        }
        std::thread::sleep(POLL);
    }
    assert!(
        saw_finish,
        "busy window never closed within {} ms ({} changes); last frame:\n{}",
        bound.as_millis(),
        changes.len(),
        changes.last().map(|(_, _, f)| f.text()).unwrap_or_default()
    );

    // The spinner run: from the first braille glyph to the first non-braille
    // cell after it (the restored label start). Everything before the run
    // would be a pre-busy sample; everything after is the idle tail.
    let run_start = changes
        .iter()
        .position(|(_, g, _)| spinner_index(g).is_some())
        .expect("no spinner glyph ever sampled at the marker cell");
    let run_end = changes[run_start..]
        .iter()
        .position(|(_, g, _)| spinner_index(g).is_none())
        .map(|i| run_start + i)
        .unwrap_or(changes.len());
    let run = &changes[run_start..run_end];
    assert!(
        run.len() as u64 >= BUSY_TICKS / 2,
        "only {} distinct spinner frames in a {BUSY_TICKS}-tick window — \
         the cycle is frozen or the sampler starved",
        run.len()
    );
    let mut record = format!(
        "ms glyph idx step\nbusy window: {BUSY_TICKS} ticks; observed {} transitions\n",
        run.len().saturating_sub(1)
    );
    let mut seen = BTreeSet::new();
    let mut prev: Option<usize> = None;
    for (ms, glyph, frame) in run {
        let idx = spinner_index(glyph).expect("run holds spinner glyphs only");
        let step = prev.map(|p| (idx + SPINNER.len() - p) % SPINNER.len());
        if let Some(d) = step {
            assert!(
                d == 1 || d == 2,
                "{ms} ms: {glyph} jumps {d} steps forward — frames must \
                 advance one tick at a time (a 2-step is one skipped sample)"
            );
        }
        seen.insert(idx);
        record.push_str(&format!(
            "{ms:>6} {glyph}     {idx:<2}  {} {:016x}\n",
            step.map_or("-".to_string(), |d| format!("+{d}")),
            frame.digest()
        ));
        prev = Some(idx);
    }
    assert_eq!(
        seen.len(),
        SPINNER.len(),
        "the live window must cover the full 10-frame cycle, saw {} distinct",
        seen.len()
    );

    // The tail restores the plain label: the marker cell becomes the label
    // start again and the row reads ` Start long job `.
    let (_, _, last_frame) = changes.last().expect("non-empty samples");
    let tail_text: String = (0..last_frame.cols)
        .filter_map(|x| last_frame.get(x, row).map(|c| c.symbol.clone()))
        .collect();
    assert!(
        tail_text.contains(LABEL),
        "label not restored after the busy window: {tail_text:?}"
    );
    assert_eq!(
        last_frame
            .get(marker_col, row)
            .expect("marker cell")
            .symbol
            .as_str(),
        "S",
        "idle marker cell must be the label start again"
    );

    record.push_str(&format!(
        "distinct={} expected={} tail={tail_text:?}\n",
        seen.len(),
        SPINNER.len()
    ));
    write_evidence("busy_spinner_cycle_advances_live.txt", record.as_bytes());
    write_evidence(
        "live_first.json",
        run.first()
            .expect("non-empty run")
            .2
            .to_json_pretty()
            .as_bytes(),
    );
    write_evidence(
        "live_mid.json",
        run[run.len() / 2].2.to_json_pretty().as_bytes(),
    );
    write_evidence("live_last.json", last_frame.to_json_pretty().as_bytes());
    eprintln!("{record}");
}

// ------------------------------------------------- row geometry (headless) --

/// Second control in the row: plain secondary, no marker of its own.
const FOLLOWER: &str = "Preview";
const FOLLOWER_W: u16 = 9;
/// Third control: a checked toggle, marker included.
const THIRD_W: u16 = 11;
/// Gap between row controls, as the pages lay them out.
const GAP: u16 = 2;
/// Full row width: leader + gap + follower + gap + third.
const ROW_W: u16 = FULL_W + GAP + FOLLOWER_W + GAP + THIRD_W;
/// Capture viewport: the full row plus a sentinel zone proving no overflow.
const VIEW_COLS: u16 = ROW_W + 8;
const VIEW_ROWS: u16 = 1;
/// Sentinel outside painted rects: any paint outside a button's own width
/// dirties it and fails.
const SENTINEL: &str = "·";

fn row_id() -> Id {
    Id::root("test.busy_live_row")
}

/// Reference `row_layout` (`src/widgets/button.rs` at `visual-baseline`),
/// reproduced verbatim: chain each width off the previous rect, clipped to
/// the area remainder, stepping by width + gap.
fn row_layout(area: Rect, widths: &[u16], gap: u16) -> Vec<Rect> {
    let mut x = area.x;
    let mut out = Vec::new();
    for &w in widths {
        let w = w.min(area.right().saturating_sub(x));
        out.push(Rect::new(x, area.y, w, area.height.min(1)));
        x = x.saturating_add(w).saturating_add(gap);
    }
    out
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
    let mut frame = Frame::blank(
        VIEW_COLS,
        VIEW_ROWS,
        Provenance::now("default", "button_busy_live", vec![]),
    );
    for y in 0..VIEW_ROWS {
        let mut x = 0u16;
        while x < VIEW_COLS {
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
            if width == 2 && x + 1 >= VIEW_COLS {
                let mut narrow = lead;
                narrow.width = 1;
                frame.set(narrow);
                x += 1;
                continue;
            }
            frame.set(lead);
            if width == 2 && x + 1 < VIEW_COLS {
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

/// Render the busy-leader row at `tick` into a sentinel buffer and capture
/// it through the component path. `on` selects the leader's toggle state;
/// `busy` toggles the busy flag; `area_w` narrows the row area to exercise
/// follower drop-out and leader clipping.
///
/// Returns the buffer, the captured frame, the logical (measured) widths,
/// the rendered (`draw`-returned) rects, and the layout rects.
fn render_row(
    tick: u64,
    on: Option<bool>,
    busy: bool,
    area_w: u16,
) -> (Buffer, Frame, Vec<u16>, Vec<Rect>, Vec<Rect>) {
    let base = row_id();
    let mut lead = match on {
        None => Button::new(base.index(0), LABEL).variant(Variant::SECONDARY),
        Some(on) => Button::new(base.index(0), LABEL)
            .variant(Variant::TOGGLE)
            .checked(on),
    };
    if busy {
        lead = lead.status(Status::Busy);
    }
    let buttons = [
        lead,
        Button::new(base.index(1), FOLLOWER).variant(Variant::SECONDARY),
        Button::new(base.index(2), "Verbose")
            .variant(Variant::TOGGLE)
            .checked(true),
    ];
    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();
    for _ in 0..tick {
        let _ = deliver(&mut runtime, Input::Tick);
    }
    let area = Rect::new(0, 0, area_w, VIEW_ROWS);
    let view = Rect::new(0, 0, VIEW_COLS, VIEW_ROWS);
    let mut buf = Buffer::empty(view);
    for x in 0..VIEW_COLS {
        if let Some(cell) = buf.cell_mut(Position::new(x, 0)) {
            cell.set_symbol(SENTINEL);
        }
    }
    type RowOut = (Vec<u16>, Vec<Rect>, Vec<Rect>);
    let out: RefCell<Option<RowOut>> = RefCell::new(None);
    runtime
        .draw_scene(view, &mut buf, |ui, _| {
            let widths: Vec<u16> = buttons
                .iter()
                .map(|b| b.measure(&*ui, Constraints::loose(u16::MAX, 1)).preferred.0)
                .collect();
            let rects = row_layout(area, &widths, GAP);
            let used: Vec<Rect> = buttons
                .iter()
                .zip(rects.iter())
                .map(|(b, r)| b.draw(&mut *ui, *r))
                .collect();
            *out.borrow_mut() = Some((widths, used, rects));
        })
        .commit_presented();
    let (widths, used, rects) = out.borrow_mut().take().expect("draw_scene ran");
    let frame = capture_buffer(&buf);
    (buf, frame, widths, used, rects)
}

/// One row of buffer symbols, sentinel zone included.
fn row_text(buf: &Buffer) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, 0)].symbol().to_owned())
        .collect()
}

/// The sentinel zone past `painted_end` (exclusive) must be untouched.
fn assert_sentinel_intact(buf: &Buffer, painted_end: u16, what: &str) {
    for x in painted_end..VIEW_COLS {
        assert_eq!(
            buf[(x, 0)].symbol(),
            SENTINEL,
            "{what}: cell x={x} painted past the row end {painted_end}"
        );
    }
}

/// Busy + checked leader: shared marker space, exact follower positions,
/// and the clipping ladder — through the real layout helper.
#[test]
fn busy_checked_row_geometry_shared_marker() {
    let theme = Theme::junie();
    assert_eq!(ROW_W, 42, "fixture row width");
    let mut record = format!("label={LABEL:?} full_w={FULL_W} row_w={ROW_W}\n");

    for on in [true, false] {
        let tick = 3;
        let what = format!("busy+on={on}");
        let (buf, frame, widths, _, rects) = render_row(tick, Some(on), true, ROW_W);

        // Shared marker space: busy+checked is label + 2 pad + 2 marker.
        assert_eq!(widths[0], FULL_W, "{what}: leader width");
        assert_eq!(widths[1], FOLLOWER_W, "{what}: follower width");
        assert_eq!(widths[2], THIRD_W, "{what}: third width");
        assert_eq!(rects[0], Rect::new(0, 0, FULL_W, 1), "{what}: leader rect");
        assert_eq!(
            rects[1],
            Rect::new(FULL_W + GAP, 0, FOLLOWER_W, 1),
            "{what}: follower must start exactly past the shared marker"
        );
        assert_eq!(
            rects[2],
            Rect::new(FULL_W + GAP + FOLLOWER_W + GAP, 0, THIRD_W, 1),
            "{what}: third control position"
        );

        // Marker precedence: the spinner wins over the toggle dot at x = 1.
        assert_eq!(
            buf[(1, 0)].symbol(),
            SPINNER[tick as usize],
            "{what}: marker"
        );
        assert_eq!(
            buf[(1, 0)].fg,
            theme.color.accent,
            "{what}: marker keeps the accent tone"
        );
        assert_eq!(
            frame.get(1, 0).expect("marker cell").symbol.as_str(),
            SPINNER[tick as usize],
            "{what}: captured marker"
        );
        assert_eq!(buf[(0, 0)].symbol(), " ", "{what}: leader gutter x=0");
        assert_eq!(buf[(2, 0)].symbol(), " ", "{what}: gap x=2");
        assert_eq!(buf[(3, 0)].symbol(), "S", "{what}: label starts x=3");
        assert_eq!(
            buf[(FULL_W - 1, 0)].symbol(),
            " ",
            "{what}: trailing pad x=w-1"
        );

        // Subsequent controls: gutter, then text at x+1, at exact offsets.
        let fx = FULL_W + GAP;
        assert_eq!(buf[(fx + 1, 0)].symbol(), "P", "{what}: follower label");
        assert_eq!(
            buf[(fx + FOLLOWER_W - 1, 0)].symbol(),
            " ",
            "{what}: follower trailing pad"
        );
        let tx = fx + FOLLOWER_W + GAP;
        assert_eq!(buf[(tx + 1, 0)].symbol(), "●", "{what}: third marker");
        assert_eq!(buf[(tx + 3, 0)].symbol(), "V", "{what}: third label");

        // Total width: the row ends at ROW_W and nothing paints past it.
        assert_eq!(
            buf[(ROW_W - 1, 0)].symbol(),
            " ",
            "{what}: row trailing pad"
        );
        assert_sentinel_intact(&buf, ROW_W, &what);

        // Busy must not move followers by even a cell: the idle row with
        // the same toggle state places every subsequent control identically.
        let (idle_buf, _, _, _, idle_rects) = render_row(tick, Some(on), false, ROW_W);
        assert_eq!(idle_rects, rects, "{what}: busy/idle rects identical");
        for x in fx..ROW_W {
            assert_eq!(
                idle_buf[(x, 0)].symbol(),
                buf[(x, 0)].symbol(),
                "{what}: follower cell x={x} differs between busy and idle"
            );
        }
        assert_eq!(
            idle_buf[(1, 0)].symbol(),
            if on { "●" } else { "○" },
            "{what}: idle toggle marker underneath"
        );

        record.push_str(&format!(
            "{what:<13} tick={tick} rects={rects:?} marker={} digest={:016x} row={:?}\n",
            SPINNER[tick as usize],
            frame.digest(),
            row_text(&buf),
        ));
        write_evidence(
            &format!("row_busy_on_{on}.json"),
            frame.to_json_pretty().as_bytes(),
        );
    }

    // Clipping ladder: narrowing the row drops followers first (their x
    // still derives from the shared leader width), then clips the leader
    // itself with the `fit` contract.
    for area_w in [ROW_W, 30, 24, 20, 18, 12, 8, 6, 4, 3, 2] {
        let what = format!("clip area_w={area_w}");
        let (buf, frame, widths, used, rects) = render_row(5, Some(true), true, area_w);
        let leader_w = FULL_W.min(area_w);
        assert_eq!(widths[0], FULL_W, "{what}: logical width");
        assert_eq!(used[0].width, leader_w, "{what}: rendered width");
        assert_eq!(rects[0].width, leader_w, "{what}: leader rect width");
        // Follower rects chain off the rendered leader width at every size.
        let fx = leader_w + GAP;
        assert_eq!(rects[1].x, fx, "{what}: follower x");
        assert_eq!(
            rects[1].width,
            FOLLOWER_W.min(area_w.saturating_sub(fx)),
            "{what}: follower width"
        );
        assert_eq!(
            rects[2].x,
            rects[1].x + rects[1].width + GAP,
            "{what}: third control chains off the follower"
        );
        // Nothing paints past the last non-empty rect.
        let painted_end = rects
            .iter()
            .filter(|r| r.width > 0)
            .map(|r| r.right())
            .max()
            .unwrap_or(0);
        assert_sentinel_intact(&buf, painted_end, &what);
        let row = row_text(&buf);
        if leader_w == FULL_W {
            assert!(row.contains(LABEL), "{what}: full leader shows it");
            assert_eq!(buf[(1, 0)].symbol(), SPINNER[5], "{what}: marker");
            // A partially clipped follower truncates with `…` and keeps its
            // trailing pad; a zero-width follower paints nothing at all.
            let follower_w = rects[1].width;
            if follower_w > 0 && follower_w < FOLLOWER_W {
                assert!(
                    row.contains('…'),
                    "{what}: clipped follower {row:?} must show …"
                );
                assert_eq!(
                    buf[(fx + follower_w - 1, 0)].symbol(),
                    " ",
                    "{what}: clipped follower keeps its trailing pad"
                );
            }
        } else if leader_w > 3 {
            assert!(row.contains('…'), "{what}: clipped row {row:?} must show …");
        } else if leader_w == 3 {
            assert!(
                row.starts_with(&format!(" {}", SPINNER[5])),
                "{what}: marker stomps the ellipsis: {row:?}"
            );
        } else {
            assert!(
                !row.contains(LABEL),
                "{what}: gutter+pad-only row {row:?} must not show the label"
            );
        }
        // Marker survives until the area is too narrow for a text cell.
        if leader_w > 2 {
            assert_eq!(buf[(1, 0)].symbol(), SPINNER[5], "{what}: marker");
            assert_eq!(
                frame.get(1, 0).expect("marker cell").symbol.as_str(),
                SPINNER[5],
                "{what}: captured marker"
            );
        }
        record.push_str(&format!(
            "{what:<16} leader_w={leader_w} rects={rects:?} row={row:?}\n"
        ));
        write_evidence(
            &format!("row_clip_{area_w:02}.json"),
            frame.to_json_pretty().as_bytes(),
        );
    }

    // The plain busy leader (no toggle state) shares the same geometry:
    // marker space is marker space regardless of what claims it.
    let (buf, _, widths, _, _) = render_row(7, None, true, ROW_W);
    assert_eq!(widths[0], FULL_W, "busy-only leader width");
    assert_eq!(buf[(1, 0)].symbol(), SPINNER[7], "busy-only marker");
    assert_eq!(
        spinner_frame(7),
        SPINNER[7],
        "capture contract matches spinner_frame()"
    );
    record.push_str(&format!(
        "busy-only w={} row={:?}\n",
        widths[0],
        row_text(&buf)
    ));

    write_evidence(
        "busy_checked_row_geometry_shared_marker.txt",
        record.as_bytes(),
    );
    eprintln!("{record}");
}
