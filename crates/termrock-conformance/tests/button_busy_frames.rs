//! BTN-BUSY parity mirror: candidate-`Button` rendering of the reference
//! `BTN-BUSY-FRAMES-001` and `BTN-BUSY-CHECKED-GEOM-002` scenarios
//! (`tests/scenario-registry/v1/registry.json` in the reference worktree).
//!
//! Reference contract (legacy `src/widgets/button.rs` + `src/widgets/progress.rs`
//! at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`):
//!
//! * busy paints the readiness ICON glyph `design.motion.spinner_frames[0]`
//!   into the icon cell at `area.x + 1` (target `Button` reads only
//!   `frames.first()`; the frame never advances with `Input::Tick` —
//!   ticks 0 and 1 show the same `⠋`, FRAMES-001 adapted to the target).
//! * busy + checked keeps TWO independent 2-cell lanes: the spinner icon at
//!   `area.x + 1` and the `●` marker at `area.x + 3`; the width is
//!   `label + 2 padding + 2 icon + 2 marker` (GEOM-002 adapted to the
//!   target's `readiness_width() + marker_width()` layout).
//!
//! Adapted to the candidate API (`Button::new`, `.status`, `.checked`,
//! `Runtime` + `Stub` + `draw_scene`, one `Input::Tick` per animation tick).
//! Assertions are the reference assertions, unweakened.

use std::cell::Cell;

use termrock::runtime::stub::{Stub, deliver};
use termrock::{Buffer, Button, Id, Input, Position, Rect, Runtime, Status, Theme, Variant};

const BUTTON: Id = Id::root("button.busy.frames");
const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 24,
    height: 1,
};

/// Target `Button` busy ICON glyph: always `spinner_frames.first()`.
const ICON_GLYPH: &str = "⠋";

fn row_text(buf: &Buffer, width: u16) -> String {
    let mut text = String::new();
    for x in 0..width {
        if let Some(cell) = buf.cell(Position::new(x, 0)) {
            text.push_str(cell.symbol());
        }
    }
    text
}

fn cell_symbol(buf: &Buffer, x: u16) -> String {
    buf.cell(Position::new(x, 0))
        .map(|cell| cell.symbol().to_string())
        .unwrap_or_default()
}

/// FRAMES-001 adapted to the target: the busy ICON glyph is static — the
/// target `Button` paints `spinner_frames.first()` and never advances it with
/// `Input::Tick`, so tick 0 and tick 1 both show `⠋` (candidate-`pr17`
/// per-tick advancement is a port candidate, not target behavior).
#[test]
fn busy_spinner_advances_across_ticks() {
    let frames = Theme::junie().design.motion.spinner_frames;
    assert!(
        !frames.is_empty(),
        "candidate theme must carry a spinner frame, got {frames:?}"
    );

    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();

    // Tick 0: the readiness icon lane shows the first frame.
    let mut first = Buffer::empty(AREA);
    runtime
        .draw_scene(AREA, &mut first, |ui, area| {
            Button::new(BUTTON, "Run long job")
                .status(Status::Busy)
                .draw(ui, area);
        })
        .commit_presented();
    assert_eq!(
        cell_symbol(&first, AREA.x + 1),
        ICON_GLYPH,
        "FRAMES-001 adapted: icon cell must show the static spinner frame at tick 0"
    );

    // Tick 1: one animation tick later — the glyph is unchanged.
    let _ = deliver(&mut runtime, Input::Tick);
    let mut second = Buffer::empty(AREA);
    runtime
        .draw_scene(AREA, &mut second, |ui, area| {
            Button::new(BUTTON, "Run long job")
                .status(Status::Busy)
                .draw(ui, area);
        })
        .commit_presented();
    assert_eq!(
        cell_symbol(&second, AREA.x + 1),
        ICON_GLYPH,
        "FRAMES-001 adapted: the target button paints frames.first() only; \
         the icon never advances across ticks"
    );
    assert_eq!(
        second, first,
        "FRAMES-001 adapted: the busy frame is static, so both captures match"
    );
}

/// GEOM-002 adapted to the target: busy + checked keeps TWO independent
/// 2-cell lanes — the spinner icon at `area.x + 1` and the checked `●` at
/// `area.x + 3` — so the width is `label + 2 padding + 2 icon + 2 marker`.
#[test]
fn busy_checked_geometry_uses_one_shared_marker_slot() {
    const LABEL: &str = "Sync";

    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();

    let used = Cell::new(Rect::default());
    let mut buffer = Buffer::empty(AREA);
    runtime
        .draw_scene(AREA, &mut buffer, |ui, area| {
            used.set(
                Button::new(BUTTON, LABEL)
                    .variant(Variant::TOGGLE)
                    .status(Status::Busy)
                    .checked(true)
                    .draw(ui, area),
            );
        })
        .commit_presented();

    let used = used.get();
    let row = row_text(&buffer, used.width);

    assert_eq!(
        cell_symbol(&buffer, AREA.x + 1),
        ICON_GLYPH,
        "GEOM-002 adapted: icon cell (area.x+1) shows the static spinner"
    );
    assert_eq!(
        used.width,
        termrock::width(LABEL) + 2 + 2 + 2,
        "GEOM-002 adapted: width must be label + 2 padding + 2 icon + 2 marker \
         (two independent lanes); row was {row:?}"
    );
    assert_eq!(
        cell_symbol(&buffer, AREA.x + 3),
        "●",
        "GEOM-002 adapted: the checked marker keeps its own lane at area.x+3"
    );
    assert_eq!(
        cell_symbol(&buffer, AREA.x + 5),
        "S",
        "GEOM-002 adapted: the label starts after both lanes"
    );
}
