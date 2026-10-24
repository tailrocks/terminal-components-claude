use ratatui_core::layout::{Position, Rect};

use crate::focus::FocusRing;
use crate::hit::{Hit, RegionKind, Registry, ReleaseFrom};
use crate::id::{Id, PartRef};
use crate::layer::LayerId;
use termrock_core::outcome::Outcome;

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
            && region.pointer_enabled()
            && region.part == part
            && !region.area.is_empty()
            && target_eligible(ring, top, owner, region.layer, region.kind))
        .then_some(region.area)
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Capture {
    pub owner: Id,
    pub part: PartRef,
    pub origin: Position,
    pub area: Rect,
    pub generation: u32,
}

impl Capture {
    pub const fn local(&self, pos: Position) -> Position {
        Position {
            x: pos.x.saturating_sub(self.area.x),
            y: pos.y.saturating_sub(self.area.y),
        }
    }

    pub const fn contains(&self, pos: Position) -> bool {
        self.area.contains(pos)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CaptureSlot {
    live: Option<Capture>,
}

impl CaptureSlot {
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

    pub(crate) fn release_if_stale(&mut self, reg: &Registry) {
        if let Some(c) = self.live
            && (!reg.has_owner(c.owner) || reg.generation() < c.generation)
        {
            self.live = None;
        }
    }
}

/// What a right-button click outside a popup does (ported from PR #26
/// `src/core/pointer.rs::SecondaryOutside`, commit f4867e9b5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecondaryOutside {
    /// Dismiss the popup and let the click reopen or land beneath it.
    #[default]
    Reopen,
    /// Dismiss the popup and swallow the click.
    Close,
    /// Dismiss nothing: the popup eats the click.
    Ignore,
}

/// Where a secondary (right button) press is routed (ported from PR #26
/// `src/core/pointer.rs::SecondaryRoute`, commit f4867e9b5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecondaryRoute {
    /// Nothing was under the pointer.
    Nothing,
    /// The press landed on this hit target.
    Target(Hit),
    /// The press landed on a backdrop; `reopen` is the target beneath it
    /// when the policy reopens the layer under the popup.
    Outside { backdrop: Id, reopen: Option<Hit> },
    /// The press is swallowed by the popup.
    Swallowed,
}

/// Tracks the secondary button across a press-drag-release so a drag that
/// leaves a row can still deliver its release (ported from PR #26
/// `src/core/pointer.rs` secondary handling, commit f4867e9b5).
#[derive(Debug, Clone, Copy, Default)]
pub struct SecondaryTracker {
    hover: Option<Hit>,
    secondary_at: Option<Position>,
}

impl SecondaryTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// The target the pointer currently hovers.
    pub const fn hover(&self) -> Option<Hit> {
        self.hover
    }

    /// Record a secondary press; returns the hit target, if any.
    pub fn on_secondary(&mut self, hits: &Registry, pos: Position) -> Option<Hit> {
        self.secondary_at = Some(pos);
        let hit = hits.hit(pos);
        self.hover = hit;
        hit
    }

    /// Route a secondary press by the outside policy.
    pub fn route_secondary(
        &mut self,
        hits: &Registry,
        pos: Position,
        policy: SecondaryOutside,
    ) -> SecondaryRoute {
        let Some(hit) = self.on_secondary(hits, pos) else {
            return SecondaryRoute::Nothing;
        };
        let Some((backdrop, beneath)) = backdrop_at(hits, pos) else {
            return SecondaryRoute::Target(hit);
        };
        match policy {
            SecondaryOutside::Ignore => SecondaryRoute::Swallowed,
            SecondaryOutside::Close => SecondaryRoute::Outside {
                backdrop,
                reopen: None,
            },
            SecondaryOutside::Reopen => {
                self.hover = beneath;
                SecondaryRoute::Outside {
                    backdrop,
                    reopen: beneath,
                }
            }
        }
    }

    /// A secondary drag keeps updating the hover, like the primary drag
    /// does, so the release can choose a new target.
    pub fn on_secondary_drag(&mut self, hits: &Registry, pos: Position) -> Outcome {
        let was = self.hover;
        self.hover = hits.hit(pos);
        if self.hover != was {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    /// A secondary release is a click only when the pointer moved since the
    /// press and the target under it accepts a secondary release. Adapted
    /// from PR #26 (commit f4867e9b5), which keyed acceptance by the hit's
    /// control id: parity's `Hit` carries the owner, so acceptance is
    /// checked against `hit.owner` — the exact id a widget passes to
    /// `Registry::accept_release`, not a coarser class.
    pub fn on_secondary_up(&mut self, hits: &Registry, pos: Position) -> Option<Hit> {
        let from = self.secondary_at.take()?;
        if from == pos {
            return None;
        }
        let hit = hits.hit(pos)?;
        hits.accepts_release(hit.owner, ReleaseFrom::Secondary)
            .then_some(hit)
    }
}

/// The topmost backdrop at `pos` and the target beneath it (ported from
/// PR #26 `src/core/hit.rs::backdrop_at`, commit f4867e9b5; adapted to
/// parity's layers instead of a barrier stack).
pub(crate) fn backdrop_at(hits: &Registry, pos: Position) -> Option<(Id, Option<Hit>)> {
    let top = hits.region_at(pos)?;
    if top.kind != RegionKind::Backdrop {
        return None;
    }
    let beneath = hits
        .regions()
        .iter()
        .rev()
        .find(|r| {
            r.layer < top.layer
                && r.area.contains(pos)
                && r.pointer_enabled()
                && matches!(r.kind, RegionKind::Control | RegionKind::Part)
        })
        .map(|r| Hit {
            owner: r.owner,
            part: r.part,
            layer: r.layer,
            kind: r.kind,
            local: Position::new(
                pos.x.saturating_sub(r.area.x),
                pos.y.saturating_sub(r.area.y),
            ),
        });
    Some((top.owner, beneath))
}

/// What a scrollbar bar should do for a pointer phase (ported from PR #26
/// `src/core/pointer.rs::scroll_input`, commit f4867e9b5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarInput {
    /// A press on the bar: jump toward the pointer or grab the thumb.
    Press,
    /// A drag while the thumb is held.
    Drag,
    /// The press ended: release the grab.
    Release,
    /// Not a bar phase.
    None,
}

/// Route a pointer phase to bar input.
pub const fn bar_input_for(phase: termrock_core::intent::Phase) -> BarInput {
    use termrock_core::intent::Phase;
    match phase {
        Phase::Press => BarInput::Press,
        Phase::DragStart | Phase::Drag => BarInput::Drag,
        Phase::Click | Phase::Release | Phase::DragEnd => BarInput::Release,
        _ => BarInput::None,
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
        reg.register_control(Id::root("thumb"), Rect::new(0, 0, 1, 1), LayerId::PAGE);
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

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 3));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 4, 4));
        assert!(rt.app().saw(THUMB, "Click"));
    }

    #[test]
    fn origin_is_the_press_position() {
        let (mut rt, mut buf) = runtime(stub());

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 5, 4));
        assert!(
            rt.app().saw(THUMB, "origin: Some(Position { x: 5, y: 4 })"),
            "origin must be the press position, not the area origin: {:?}",
            rt.app().log
        );

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

// Ported from PR #26 `pointer.rs` secondary and bar-input tests (commit
// f4867e9b5), adapted to the parity registry and SecondaryTracker.
#[cfg(test)]
mod pointer_tests {
    use super::*;
    use crate::id::Part;

    // Ported from PR #26 `pointer.rs` (commit f4867e9b5), adapted: parity
    // keys release acceptance by the hit's owner (PR #26 keyed it by the
    // hit's control id), so acceptance here is exact per widget owner.
    #[test]
    fn a_secondary_release_on_an_accepting_owner_is_a_click() {
        use termrock_core::id::ItemKey;
        use termrock_core::intent::Phase;

        let mut h = Registry::new(1);
        let list = Id::root("list");
        h.register_control(list, Rect::new(0, 0, 10, 3), LayerId::PAGE);
        let row = PartRef::item(Part::ROW, ItemKey::num(2));
        h.register_part(list, row, Rect::new(0, 2, 10, 1), LayerId::PAGE);
        h.accept_release(list, ReleaseFrom::Secondary);

        let mut t = SecondaryTracker::new();
        let press = t
            .on_secondary(&h, Position::new(2, 0))
            .expect("the press lands on the list");
        assert_eq!((press.owner, press.layer), (list, LayerId::PAGE));
        assert_eq!(
            t.on_secondary_drag(&h, Position::new(2, 2)),
            Outcome::Changed,
            "the drag moves the hover onto the row"
        );
        let hit = t
            .on_secondary_up(&h, Position::new(2, 2))
            .expect("the row accepts a secondary release");
        assert_eq!((hit.owner, hit.part), (list, row));

        // Without the movement there is no click, and an unaccepting
        // target never converts the release.
        let mut t = SecondaryTracker::new();
        let _ = t.on_secondary(&h, Position::new(2, 2));
        assert_eq!(t.on_secondary_up(&h, Position::new(2, 2)), None);
        let mut h2 = h.clone();
        h2.reset(2);
        h2.register_control(list, Rect::new(0, 0, 10, 3), LayerId::PAGE);
        h2.register_part(list, row, Rect::new(0, 2, 10, 1), LayerId::PAGE);
        let mut t = SecondaryTracker::new();
        let _ = t.on_secondary(&h2, Position::new(2, 0));
        assert_eq!(
            t.on_secondary_up(&h2, Position::new(2, 2)),
            None,
            "no secondary release without an accepting target"
        );
        assert_eq!(bar_input_for(Phase::Secondary), BarInput::None);
    }

    // Ported from PR #26 `pointer.rs::route_secondary` behaviour with
    // `SecondaryOutside::Reopen` (commit f4867e9b5).
    #[test]
    fn a_secondary_outside_a_backdrop_reopens_the_layer_beneath() {
        let mut h = Registry::new(1);
        let page = Id::root("page");
        let owner = Id::root("select");
        let back = owner.part(Part::BACKDROP);
        h.register_control(page, Rect::new(0, 0, 20, 10), LayerId::PAGE);
        h.register_control(owner, Rect::new(10, 2, 5, 3), LayerId(1));
        h.register_backdrop_of(back, owner, Rect::new(0, 0, 20, 10), LayerId(1), false);
        assert!(
            h.has_owner(back),
            "the backdrop's own id stays queryable after registration"
        );

        let mut t = SecondaryTracker::new();
        assert_eq!(
            t.route_secondary(&h, Position::new(12, 3), SecondaryOutside::Ignore),
            SecondaryRoute::Swallowed,
            "Ignore swallows a press on the backdrop"
        );
        match t.route_secondary(&h, Position::new(2, 5), SecondaryOutside::Reopen) {
            SecondaryRoute::Outside {
                backdrop,
                reopen: Some(reopen),
            } => {
                assert_eq!(backdrop, owner);
                assert_eq!(
                    (reopen.owner, reopen.part),
                    (page, PartRef::of(Part::CONTAINER)),
                    "the page beneath the backdrop is the reopen target"
                );
            }
            other => panic!("expected an Outside reopen route, got {other:?}"),
        }
        assert_eq!(
            t.hover().map(|x| x.owner),
            Some(page),
            "Reopen moves the hover beneath the backdrop"
        );

        let mut t = SecondaryTracker::new();
        match t.route_secondary(&h, Position::new(2, 5), SecondaryOutside::Close) {
            SecondaryRoute::Outside { backdrop, reopen } => {
                assert_eq!(backdrop, owner);
                assert_eq!(reopen, None, "Close dismisses without reopening");
            }
            other => panic!("expected an Outside route, got {other:?}"),
        }
    }

    #[test]
    fn bar_input_routes_the_press_phases() {
        use termrock_core::intent::Phase;
        assert_eq!(bar_input_for(Phase::Press), BarInput::Press);
        assert_eq!(bar_input_for(Phase::DragStart), BarInput::Drag);
        assert_eq!(bar_input_for(Phase::Drag), BarInput::Drag);
        assert_eq!(bar_input_for(Phase::Click), BarInput::Release);
        assert_eq!(bar_input_for(Phase::Release), BarInput::Release);
        assert_eq!(bar_input_for(Phase::DragEnd), BarInput::Release);
        assert_eq!(bar_input_for(Phase::Move), BarInput::None);
        assert_eq!(bar_input_for(Phase::DoubleClick), BarInput::None);
    }
}
