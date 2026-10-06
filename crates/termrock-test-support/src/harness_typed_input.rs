//! Typed, bounds-checked input delivery for [`Harness`](crate::harness::Harness).
//!
//! Mirror of the reference `typed_input` helper, adapted to the candidate
//! harness API. The reference guards the raw SGR-1006 hover escape with a
//! grid-bounds check (audit G1/P1); the candidate harness already speaks
//! typed [`MouseKind`](termrock::MouseKind), so this module supplies the
//! fail-closed half: every cell coordinate is validated against the live
//! frame before delivery, and hand-rolled `+1` arithmetic has no reason to
//! exist. Off-grid input panics with the offending cell and the frame size
//! instead of silently addressing another widget.

use termrock::{App, Axis, MouseKind, Response};

use crate::harness::Harness;

/// The live frame size as `(width, height)` in cells.
pub fn frame_size<A: App>(h: &Harness<A>) -> (u16, u16) {
    let area = h.buffer().area();
    (area.width, area.height)
}

fn assert_on_grid<A: App>(h: &Harness<A>, what: &str, x: u16, y: u16) {
    let (w, hh) = frame_size(h);
    assert!(
        x < w && y < hh,
        "typed input out of grid: {what} at ({x},{y}) exceeds {w}x{hh}"
    );
}

/// Button-less pointer motion to `(x, y)`: the typed hover (audit P1).
pub fn hover<A: App>(h: &mut Harness<A>, x: u16, y: u16) -> Response<()> {
    assert_on_grid(h, "hover", x, y);
    h.mouse(MouseKind::Move, x, y)
}

/// Press then release at `(x, y)`.
pub fn click_cell<A: App>(h: &mut Harness<A>, x: u16, y: u16) -> Response<()> {
    assert_on_grid(h, "click", x, y);
    h.click(x, y)
}

/// Press at `from`, drag to `to`, release at `to`. Both endpoints are
/// validated; use [`crate::harness_scoped_targets::span_end`] to derive
/// `to` from a resolved target instead of hand-rolling offsets.
pub fn drag_cells<A: App>(h: &mut Harness<A>, from: (u16, u16), to: (u16, u16)) -> Response<()> {
    assert_on_grid(h, "drag-from", from.0, from.1);
    assert_on_grid(h, "drag-to", to.0, to.1);
    h.drag(from, to)
}

/// Wheel motion at `(x, y)`; positive `delta` is down / right.
pub fn wheel_at<A: App>(
    h: &mut Harness<A>,
    axis: Axis,
    delta: i16,
    x: u16,
    y: u16,
) -> Response<()> {
    assert_on_grid(h, "wheel", x, y);
    h.wheel(axis, delta, x, y)
}

#[cfg(test)]
mod tests {
    use termrock::{Cx, Response, Theme, Ui};

    use super::*;

    struct Blank;

    impl App for Blank {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    fn harness() -> Harness<Blank> {
        Harness::new(Blank, Theme::junie(), 20, 5)
    }

    #[test]
    fn frame_size_reports_the_live_grid() {
        let h = harness();
        assert_eq!(frame_size(&h), (20, 5));
    }

    #[test]
    fn hover_delivers_a_typed_move_on_grid() {
        let mut h = harness();
        let _ = hover(&mut h, 3, 1);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    #[should_panic(expected = "typed input out of grid: hover at (20,1) exceeds 20x5")]
    fn hover_off_grid_fails_closed() {
        let mut h = harness();
        let _ = hover(&mut h, 20, 1);
    }

    #[test]
    fn click_and_wheel_deliver_on_grid() {
        let mut h = harness();
        let _ = click_cell(&mut h, 0, 0);
        let _ = wheel_at(&mut h, Axis::V, 1, 19, 4);
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    #[should_panic(expected = "typed input out of grid: click at (0,5) exceeds 20x5")]
    fn click_off_grid_fails_closed() {
        let mut h = harness();
        let _ = click_cell(&mut h, 0, 5);
    }

    #[test]
    fn drag_validates_both_endpoints() {
        let mut h = harness();
        let _ = drag_cells(&mut h, (0, 0), (19, 4));
        assert!(h.diagnostics().is_empty());
    }

    #[test]
    #[should_panic(expected = "typed input out of grid: drag-to at (19,5) exceeds 20x5")]
    fn drag_to_off_grid_fails_closed() {
        let mut h = harness();
        let _ = drag_cells(&mut h, (0, 0), (19, 5));
    }
}
