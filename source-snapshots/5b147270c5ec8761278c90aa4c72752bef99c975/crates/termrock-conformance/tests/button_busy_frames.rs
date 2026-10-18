//! BTN-BUSY parity mirror: candidate-`Button` rendering of the reference
//! `BTN-BUSY-FRAMES-001` and `BTN-BUSY-CHECKED-GEOM-002` scenarios
//! (`tests/scenario-registry/v1/registry.json` in the reference worktree).
//!
//! Reference contract (legacy `src/widgets/button.rs` + `src/widgets/progress.rs`
//! at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`):
//!
//! * busy paints `spinner_frame(tick) = SPINNER[tick % 10]` into the marker
//!   cell at `area.x + 1`: tick 0 shows `⠋` (U+280B), tick 1 shows `⠙`
//!   (U+2819); the two captures must differ (FRAMES-001 V1 + N1).
//! * busy + checked shares ONE 2-cell marker slot: the spinner overwrites the
//!   `●` at `area.x + 1`, `on` is preserved, and the width is
//!   `label + 2 padding + 2 marker` — no second slot, no `●`/`○` anywhere in
//!   the row (GEOM-002 V1 + V2 + N1 + N2).
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

/// Reference `SPINNER[0]` / `SPINNER[1]` (`spinner_frame(tick)` at tick 0/1).
const TICK0_GLYPH: &str = "⠋";
const TICK1_GLYPH: &str = "⠙";

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

/// FRAMES-001 V1 + N1: the busy spinner advances across ticks — tick 0 shows
/// `⠋`, tick 1 shows `⠙`, and the two frames differ (multi-frame, not frozen).
#[test]
fn busy_spinner_advances_across_ticks() {
    // The candidate theme carries the same ten-frame table the reference
    // `spinner_frame` indexes, so the exact-glyph assertions below compare
    // against the table the candidate itself paints from.
    let frames = Theme::junie().design.motion.spinner_frames;
    assert!(
        frames.len() >= 2,
        "candidate theme must carry a multi-frame spinner, got {frames:?}"
    );
    assert_eq!(frames[0], TICK0_GLYPH);
    assert_eq!(frames[1], TICK1_GLYPH);

    let mut runtime = Runtime::new(Stub::default(), Theme::junie());
    let _ = runtime.initialize();

    // Tick 0: first frame.
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
        TICK0_GLYPH,
        "FRAMES-001 V1: marker cell must show ⠋ at tick 0"
    );

    // Tick 1: one animation tick later.
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
        TICK1_GLYPH,
        "FRAMES-001 V1: marker cell must show ⠙ at tick 1 \
         (spinner_frame(tick) = SPINNER[tick % 10])"
    );
    assert_ne!(
        second, first,
        "FRAMES-001 N1: second-frame capture must NOT equal first-frame capture"
    );
}

/// GEOM-002 V1 + V2 + N1 + N2: busy + checked shares one 2-cell marker slot —
/// the spinner wins at `area.x + 1`, the width is `label + 2 + 2`, and no
/// `●`/`○` appears anywhere in the row.
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
        TICK0_GLYPH,
        "GEOM-002 V1: marker cell (area.x+1) must show the spinner, not ●"
    );
    assert_eq!(
        used.width,
        termrock::width(LABEL) + 2 + 2,
        "GEOM-002 V2: width must be label + 2 padding + 2 marker cells \
         (single shared slot, not doubled); row was {row:?}"
    );
    assert!(
        !row.contains('●') && !row.contains('○'),
        "GEOM-002 N1: ●/○ must NOT appear anywhere in the button row while \
         busy (spinner overwrite wins); row was {row:?}"
    );
}
