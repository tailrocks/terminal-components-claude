//! Busy-button spinner frames: multi-frame regression over the full cycle.
//!
//! Priority regression: the reference [`Button`](junie_tui::widgets::button::Button)
//! reads the spinner glyph from the *current* render tick
//! (`spinner_frame(ctx.interaction.tick)`), so every tick in the 10-frame
//! braille cycle must surface a distinct glyph and the cycle boundary must
//! wrap exactly. These tests render the real production Button directly
//! into a Ratatui buffer and convert each buffer through the component
//! capture path ([`support::capture_buffer`], the pure-view capture
//! over `tuiscotti::ratatui::from_buffer`) into a canonical
//! [`Frame`](tuiscotti::Frame); assertions run against both the buffer
//! cells and the captured frames.
//!
//! Evidence: every test writes its captured frames and a human-readable
//! record under a unique run dir in `target/tuiscotti-actuals/` (gitignored
//! scratch, like `target/tuiscotti/`). The dir is printed via `eprintln`.
//!
//! The showcase busy control under test is the `Start long job` secondary
//! button (`ButtonsPage` index 8), busy while its wall-clock job runs.
//! Geometry below pins its glyph positions, total width, and clipping.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use junie_tui::core::{focus::FocusRing, hit::HitRegistry, id::WidgetId};
use junie_tui::theme::Theme;
use junie_tui::ui::ctx::{Interaction, RenderCtx};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::progress::{SPINNER, spinner_frame};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tuiscotti::{Frame, Provenance};

use crate::support;

/// Label of the real showcase busy button (`ButtonsPage` index 8).
const LABEL: &str = "Start long job";
/// Display width of [`LABEL`] (all ASCII).
const LABEL_W: u16 = 14;
/// Full busy-button width: label + 2 padding + 2 marker cells.
const FULL_W: u16 = LABEL_W + 2 + 2;
/// Capture viewport: the full button plus trailing blanks.
const VIEW_COLS: u16 = 24;
const VIEW_ROWS: u16 = 1;

fn button_id() -> WidgetId {
    WidgetId::of("test.busy_frames")
}

/// Render one busy-button frame at `tick` into a `VIEW_COLS`×1 buffer and
/// capture it through the component path. `on` selects the toggle state
/// (`None` = plain busy button, `Some` = busy + checked/unchecked).
/// `area_w` narrows the render area to exercise clipping.
fn render_capture(tick: u64, on: Option<bool>, area_w: u16) -> (Buffer, Frame, Button) {
    let theme = Theme::junie();
    let id = button_id();
    let mut button = match on {
        None => Button::secondary(id, LABEL),
        Some(on) => Button::toggle(id, LABEL, on),
    };
    button.busy = true;
    let area = Rect::new(0, 0, area_w, VIEW_ROWS);
    let mut buf = Buffer::empty(Rect::new(0, 0, VIEW_COLS, VIEW_ROWS));
    let mut hits = HitRegistry::default();
    let mut ring = FocusRing::default();
    let mut ctx = RenderCtx::new(
        &theme,
        Interaction {
            tick,
            ..Default::default()
        },
        &mut hits,
        &mut ring,
    );
    button.render(area, &mut buf, &mut ctx, theme.canvas);
    let frame = support::capture_buffer(
        &buf,
        Provenance::now("default", "button_busy_frames", vec![]),
    );
    (buf, frame, button)
}

/// Glyph of the marker cell (x = 1) in a captured frame.
fn marker_glyph(frame: &Frame) -> &str {
    frame
        .get(1, 0)
        .unwrap_or_else(|| panic!("frame has no marker cell"))
        .symbol
        .as_str()
}

/// One row of buffer symbols for diagnostics.
fn buf_row(buf: &Buffer) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, 0)].symbol().to_owned())
        .collect::<String>()
        .trim_end()
        .to_owned()
}

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
            .join(format!(
                "button_busy_frames_{}_{}",
                std::process::id(),
                nanos
            ));
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|e| panic!("create evidence dir {}: {e}", dir.display()));
        eprintln!("button_busy_frames evidence: {}", dir.display());
        dir
    })
}

fn write_evidence(name: &str, bytes: &[u8]) {
    let path = evidence_dir().join(name);
    std::fs::write(&path, bytes)
        .unwrap_or_else(|e| panic!("write evidence {}: {e}", path.display()));
}

/// Every tick of the 10-frame cycle renders its own spinner glyph at the
/// marker cell: full coverage, in tick order.
#[test]
fn busy_spinner_covers_full_cycle() {
    assert_eq!(SPINNER.len(), 10, "reference cycle length");
    let mut seen = BTreeSet::new();
    let mut record = String::from("tick glyph frame-text digest\n");
    for tick in 0..SPINNER.len() as u64 {
        let (buf, frame, button) = render_capture(tick, None, FULL_W);
        let glyph = marker_glyph(&frame);
        assert_eq!(
            glyph,
            SPINNER[tick as usize],
            "tick {tick}: marker must be spinner_frame(tick)"
        );
        assert_eq!(
            glyph,
            spinner_frame(tick),
            "tick {tick}: capture must match spinner_frame()"
        );
        // Buffer and captured frame agree on the marker glyph.
        assert_eq!(buf[(1, 0)].symbol(), glyph, "tick {tick}: buffer/frame agree");
        assert_eq!(button.width(), FULL_W, "tick {tick}: total width");
        assert!(
            seen.insert(glyph.to_owned()),
            "tick {tick}: glyph {glyph} already seen — cycle has a duplicate"
        );
        record.push_str(&format!(
            "{tick:>4} {glyph}     {} {:016x}\n",
            frame.text(),
            frame.digest()
        ));
        write_evidence(&format!("tick_{tick:02}.json"), frame.to_json_pretty().as_bytes());
    }
    assert_eq!(seen.len(), SPINNER.len(), "every cycle frame appears");
    record.push_str(&format!(
        "distinct={} expected={} label={LABEL:?} total_width={FULL_W}\n",
        seen.len(),
        SPINNER.len()
    ));
    write_evidence("busy_spinner_covers_full_cycle.txt", record.as_bytes());
    eprintln!("{record}");
}

/// The cycle boundary wraps: tick 10 re-renders tick 0 cell-exactly (glyph,
/// full-frame digest, zero differing cells), and later ticks stay in phase.
#[test]
fn busy_spinner_cycle_boundary_wraps() {
    let pairs: [(u64, u64); 5] = [(10, 0), (11, 1), (19, 9), (20, 0), (21, 1)];
    let mut record = String::from("tick glyph equiv digest\n");
    for (tick, equiv) in pairs {
        let (_, frame, _) = render_capture(tick, None, FULL_W);
        let (_, expected, _) = render_capture(equiv, None, FULL_W);
        assert_eq!(
            marker_glyph(&frame),
            marker_glyph(&expected),
            "tick {tick} must wrap to tick {equiv}"
        );
        assert_eq!(
            frame.digest(),
            expected.digest(),
            "tick {tick} frame digest must equal tick {equiv}"
        );
        let diffs = frame
            .diff_cells(&expected)
            .expect("same-dimension frames diff cleanly");
        assert!(
            diffs.is_empty(),
            "tick {tick} vs {equiv}: differing cells {diffs:?}"
        );
        record.push_str(&format!(
            "{tick:>4} {}      {equiv:<5} {:016x}\n",
            marker_glyph(&frame),
            frame.digest()
        ));
        write_evidence(
            &format!("wrap_tick_{tick:02}.json"),
            frame.to_json_pretty().as_bytes(),
        );
    }
    // The boundary pair is the strongest form: whole-frame JSON identical.
    let (_, ten, _) = render_capture(10, None, FULL_W);
    let (_, zero, _) = render_capture(0, None, FULL_W);
    assert_eq!(
        ten.to_json(),
        zero.to_json(),
        "tick 10 canonical JSON must equal tick 0"
    );
    write_evidence("busy_spinner_cycle_boundary_wraps.txt", record.as_bytes());
    eprintln!("{record}");
}

/// Busy + checked geometry: marker position, total width, marker precedence
/// (spinner wins over the toggle dot), and clipping at narrow widths.
#[test]
fn busy_checked_geometry_recorded() {
    let theme = Theme::junie();
    let mut record = format!(
        "label={LABEL:?} label_w={LABEL_W} full_w={FULL_W} view={VIEW_COLS}x{VIEW_ROWS}\n"
    );

    // Toggle controls without busy: the marker cell shows the on/off dot.
    for on in [true, false] {
        let id = button_id();
        let mut plain = Button::toggle(id, LABEL, on);
        let area = Rect::new(0, 0, FULL_W, VIEW_ROWS);
        let mut buf = Buffer::empty(Rect::new(0, 0, VIEW_COLS, VIEW_ROWS));
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        plain.render(area, &mut buf, &mut ctx, theme.canvas);
        assert_eq!(
            buf[(1, 0)].symbol(),
            if on { "●" } else { "○" },
            "non-busy toggle marker"
        );
    }

    // Busy + checked/unchecked: same geometry as busy-only; the spinner
    // overwrites the toggle dot at x = 1 with the accent tone.
    for on in [true, false] {
        let tick = 3;
        let (buf, frame, button) = render_capture(tick, Some(on), FULL_W);
        assert_eq!(
            button.width(),
            FULL_W,
            "busy+on={on}: marker must not double-count"
        );
        assert_eq!(button.area.width, FULL_W, "busy+on={on}: recorded area");
        assert_eq!(marker_glyph(&frame), SPINNER[tick as usize]);
        assert_eq!(buf[(1, 0)].symbol(), SPINNER[tick as usize]);
        assert_eq!(
            buf[(1, 0)].fg, theme.accent,
            "busy+on={on}: marker keeps the accent tone"
        );
        assert_eq!(buf[(0, 0)].symbol(), " ", "busy+on={on}: gutter x=0");
        assert_eq!(buf[(2, 0)].symbol(), " ", "busy+on={on}: gap x=2");
        assert_eq!(buf[(3, 0)].symbol(), "S", "busy+on={on}: label starts x=3");
        assert_eq!(
            buf[(FULL_W - 1, 0)].symbol(),
            " ",
            "busy+on={on}: trailing pad x=w-1"
        );
        record.push_str(&format!(
            "busy+on={on:<5} tick={tick} w={} area={:?} marker={} row={:?}\n",
            button.width(),
            button.area,
            marker_glyph(&frame),
            buf_row(&buf),
        ));
        write_evidence(
            &format!("busy_checked_on_{on}.json"),
            frame.to_json_pretty().as_bytes(),
        );
    }

    // Clipping: narrowing the area shrinks the rendered width and
    // truncates the text with `…` (the `fit` contract), never panicking.
    // The text region is w-2 cells; at w = 3 the single text cell would be
    // `…`, but the busy marker overwrite stomps it with the spinner glyph,
    // so the row is gutter + spinner + pad. At w = 2 the two cells are
    // gutter + trailing pad and the row is blank by construction.
    for area_w in [FULL_W, 12, 8, 6, 4, 3, 2] {
        let (buf, frame, button) = render_capture(5, Some(true), area_w);
        let expected_w = FULL_W.min(area_w);
        assert_eq!(button.width(), FULL_W, "area_w={area_w}: logical width");
        assert_eq!(
            button.area.width, expected_w,
            "area_w={area_w}: clipped width"
        );
        let row = buf_row(&buf);
        if area_w == FULL_W {
            assert!(
                row.contains(LABEL),
                "area_w={area_w}: full row {row:?} must show the label"
            );
        } else if expected_w > 3 {
            assert!(
                row.contains('…'),
                "area_w={area_w}: clipped row {row:?} must show …"
            );
        } else if expected_w == 3 {
            assert_eq!(
                row,
                format!(" {}", SPINNER[5]),
                "area_w={area_w}: marker stomps the ellipsis"
            );
        } else {
            assert!(
                row.is_empty(),
                "area_w={area_w}: gutter+pad-only row {row:?} must be blank"
            );
        }
        // Marker survives until the area is too narrow for a text cell.
        if expected_w > 2 {
            assert_eq!(buf[(1, 0)].symbol(), SPINNER[5]);
            assert_eq!(marker_glyph(&frame), SPINNER[5]);
        }
        record.push_str(&format!(
            "clip area_w={area_w:<2} rendered_w={} marker={} row={:?}\n",
            button.area.width,
            frame
                .get(1, 0)
                .map(|c| c.symbol.as_str())
                .unwrap_or("<none>"),
            row,
        ));
        write_evidence(
            &format!("busy_checked_clip_{area_w:02}.json"),
            frame.to_json_pretty().as_bytes(),
        );
    }
    write_evidence("busy_checked_geometry_recorded.txt", record.as_bytes());
    eprintln!("{record}");
}
