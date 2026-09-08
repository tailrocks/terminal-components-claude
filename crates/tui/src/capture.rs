//! Pointer capture (`COMPONENT_ARCHITECTURE.md` §8.2).
//!
//! While a capture is live, every `Drag`/`Release` goes to the capturing
//! owner with `local` computed against the captured area, hit-testing for
//! other widgets is suppressed, `PRESSED` stays set regardless of hover, and
//! release activates iff the pointer is inside the captured area. Captures
//! are released on resize, generation mismatch, or when the actual part is
//! absent, disabled, or blocked by the live top layer. Nested captures are
//! rejected, never stacked.

use ratatui_core::layout::{Position, Rect};

use crate::focus::FocusRing;
use crate::hit::{RegionKind, Registry};
use crate::id::{Id, PartRef};
use crate::layer::LayerId;

pub(crate) fn target_eligible(
    ring: &FocusRing,
    top: LayerId,
    owner: Id,
    layer: LayerId,
    kind: RegionKind,
) -> bool {
    layer == top
        && matches!(kind, RegionKind::Control | RegionKind::Part)
        && !ring.entry(owner).is_some_and(|entry| entry.disabled)
}

pub(crate) fn target_area(
    registry: &Registry,
    ring: &FocusRing,
    top: LayerId,
    owner: Id,
    part: PartRef,
) -> Option<Rect> {
    registry.regions().iter().rev().find_map(|region| {
        (region.owner == owner
            && region.part == part
            && !region.area.is_empty()
            && target_eligible(ring, top, owner, region.layer, region.kind))
        .then_some(region.area)
    })
}

/// A live pointer capture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capture {
    /// The capturing owner.
    pub owner: Id,
    /// The part that claimed it.
    pub part: PartRef,
    /// Where the pointer was when the claim was made.
    pub origin: Position,
    /// The area `local` is computed against.
    pub area: Rect,
    /// The registry generation the claim was made against.
    pub generation: u32,
}

impl Capture {
    /// `pos` relative to the captured area's origin (saturating).
    pub const fn local(&self, pos: Position) -> Position {
        Position {
            x: pos.x.saturating_sub(self.area.x),
            y: pos.y.saturating_sub(self.area.y),
        }
    }

    /// Whether `pos` is inside the captured area.
    pub const fn contains(&self, pos: Position) -> bool {
        self.area.contains(pos)
    }
}

/// The runtime's single capture slot.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CaptureSlot {
    live: Option<Capture>,
}

impl CaptureSlot {
    /// Claim; `false` if another capture is live.
    pub(crate) fn claim(&mut self, c: Capture) -> bool {
        if self.live.is_some() {
            return false;
        }
        self.live = Some(c);
        true
    }

    pub(crate) const fn get(&self) -> Option<Capture> {
        self.live
    }

    pub(crate) fn release(&mut self) -> Option<Capture> {
        self.live.take()
    }

    /// Release when the owner vanished or the generation moved backwards.
    /// Runtime additionally checks the actual part against the published ring
    /// and live layer using `target_area` (§3.3 step 13).
    pub(crate) fn release_if_stale(&mut self, reg: &Registry) {
        if let Some(c) = self.live
            && (!reg.has_owner(c.owner) || reg.generation() < c.generation)
        {
            self.live = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Part;

    fn cap(generation: u32) -> Capture {
        Capture {
            owner: Id::root("thumb"),
            part: PartRef::of(Part::THUMB),
            origin: Position::new(5, 5),
            area: Rect::new(3, 3, 4, 10),
            generation,
        }
    }

    #[test]
    fn capture_claims_and_rejects_a_second_claim() {
        let mut s = CaptureSlot::default();
        assert!(s.claim(cap(1)));
        assert!(!s.claim(Capture {
            owner: Id::root("other"),
            ..cap(1)
        }));
        assert_eq!(s.get().map(|c| c.owner), Some(Id::root("thumb")));
        assert!(s.release().is_some());
        assert!(s.get().is_none());
    }

    #[test]
    fn local_is_computed_against_the_captured_area() {
        let c = cap(1);
        assert_eq!(c.local(Position::new(5, 7)), Position::new(2, 4));
        assert_eq!(c.local(Position::new(0, 0)), Position::new(0, 0));
        assert!(c.contains(Position::new(6, 12)));
        assert!(!c.contains(Position::new(7, 12)));
    }

    #[test]
    fn capture_is_released_when_the_owner_disappears() {
        let mut s = CaptureSlot::default();
        s.claim(cap(1));
        let reg = Registry::new(1);
        s.release_if_stale(&reg);
        assert!(s.get().is_none());
    }

    #[test]
    fn capture_is_released_on_generation_mismatch() {
        let mut s = CaptureSlot::default();
        s.claim(cap(5));
        let mut reg = Registry::new(3);
        reg.register_control(
            Id::root("thumb"),
            Rect::new(0, 0, 1, 1),
            crate::layer::LayerId::PAGE,
        );
        s.release_if_stale(&reg);
        assert!(s.get().is_none());
        s.claim(cap(3));
        s.release_if_stale(&reg);
        assert!(s.get().is_some());
    }
}

#[cfg(test)]
mod runtime_tests {
    use ratatui_core::layout::Rect;

    use crate::event::{Input, MouseKind};
    use crate::id::Id;
    use crate::response::StateFlags;
    use crate::runtime::stub::{Control, Stub, mouse, runtime, step};

    const THUMB: Id = Id::root("thumb");
    const OTHER: Id = Id::root("other");

    fn stub() -> Stub {
        Stub {
            page: vec![
                Control {
                    captures: true,
                    ..Control::new(THUMB, Rect::new(2, 2, 4, 4))
                },
                Control::new(OTHER, Rect::new(20, 2, 5, 1)),
            ],
            ..Stub::default()
        }
    }

    #[test]
    fn drag_and_release_go_to_the_capture_owner() {
        let (mut rt, mut buf) = runtime(stub());
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        assert_eq!(
            rt.state_of(THUMB) & StateFlags::PRESSED,
            StateFlags::PRESSED
        );
        // a drag far outside the thumb, over another control
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Drag, 22, 2));
        assert!(rt.app().saw(THUMB, "phase: Drag"));
        assert!(!rt.app().saw(OTHER, "Drag"));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 22, 2));
        assert!(rt.app().saw(THUMB, "phase: Release"));
        assert!(rt.app().saw(THUMB, "phase: DragEnd"));
    }

    #[test]
    fn local_is_computed_against_the_captured_area_at_runtime() {
        let (mut rt, mut buf) = runtime(stub());
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Drag, 10, 9));
        assert!(rt.app().saw(THUMB, "local: Position { x: 8, y: 7 }"));
    }

    #[test]
    fn pressed_stays_set_while_the_pointer_leaves() {
        let (mut rt, mut buf) = runtime(stub());
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Drag, 30, 10));
        assert!(rt.state_of(THUMB).contains(StateFlags::PRESSED));
        assert!(!rt.state_of(THUMB).contains(StateFlags::HOVERED));
    }

    #[test]
    fn release_outside_the_captured_area_does_not_activate() {
        let (mut rt, mut buf) = runtime(stub());
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 30, 10));
        assert!(rt.app().saw(THUMB, "Release"));
        assert!(!rt.app().saw(THUMB, "Click"));
        // inside: activates
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 4, 4));
        assert!(rt.app().saw(THUMB, "Click"));
    }

    /// §8.2: `Capture.origin` is "where the pointer was when the claim was
    /// made", so a splitter or a scrollbar thumb computes `pos - origin`
    /// without the press offset inside the thumb leaking into the delta.
    #[test]
    fn origin_is_the_press_position() {
        let (mut rt, mut buf) = runtime(stub());
        // press at (5, 4) — inside the thumb, three columns right and two
        // rows below its top-left (2, 2)
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 5, 4));
        assert!(
            rt.app().saw(THUMB, "origin: Some(Position { x: 5, y: 4 })"),
            "origin must be the press position, not the area origin: {:?}",
            rt.app().log
        );
        // a drag one cell right moves the thumb by exactly one cell
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Drag, 6, 4));
        assert!(rt.app().saw(THUMB, "phase: Drag"));
    }

    #[test]
    fn capture_is_released_on_resize() {
        let (mut rt, mut buf) = runtime(stub());
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        assert!(rt.state_of(THUMB).contains(StateFlags::PRESSED));
        let _ = step(&mut rt, &mut buf, Input::Resize(40, 12));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Drag, 22, 2));
        assert!(!rt.app().saw(THUMB, "Drag"), "no capture survives a resize");
    }
}
