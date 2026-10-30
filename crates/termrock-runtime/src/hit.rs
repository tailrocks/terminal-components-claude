pub use termrock_core::geometry::{Axes, Headroom};

use ratatui_core::layout::{Position, Rect};

use crate::diagnostics::Diagnostic;
use crate::event::Axis;
use crate::id::{Id, Part, PartRef};
use crate::layer::LayerId;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum RegionKind {
    Control,
    Part,
    Scroll,
    Decorative,
    /// A light-dismiss backdrop registered behind a popup or modal.
    Backdrop,
}

/// A scrollbar bar registered by its owning widget (ported from PR #26
/// `src/core/hit.rs::Bar`, commit f4867e9b5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bar {
    /// The bar's own id (the track / thumb control).
    pub id: Id,
    /// The widget the bar scrolls.
    pub owner: Id,
    /// Which axis the bar scrolls.
    pub axis: Axis,
    /// The track area, including the thumb.
    pub track: Rect,
}

/// Whose release a widget accepts (ported from PR #26 `src/core/hit.rs::ReleaseFrom`,
/// commit f4867e9b5): a drag that started on a widget keeps receiving
/// releases, and a secondary-button release may be accepted while pressed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReleaseFrom {
    /// The release of a press that started on `id`.
    Press(Id),
    /// A secondary (right button) release.
    Secondary,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hit {
    pub owner: Id,
    pub part: PartRef,
    pub layer: LayerId,
    pub kind: RegionKind,
    pub local: Position,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Region {
    /// The region's own control id (ported from PR #26
    /// `HitRegion::id`, commit f4867e9b5): equal to `owner` except where a
    /// registration splits the two — a thumb keeps its bar's id and a
    /// backdrop keeps the id it was hit-tested as.
    pub id: Id,
    pub owner: Id,
    pub part: PartRef,
    pub area: Rect,
    pub layer: LayerId,
    pub kind: RegionKind,
    pub generation: u32,
    scroll: Option<(Axes, Headroom)>,
    pointer_enabled: bool,
    pass_through: bool,
}

impl Region {
    pub const fn pointer_enabled(&self) -> bool {
        self.pointer_enabled
    }

    pub const fn scroll(&self) -> Option<(Axes, Headroom)> {
        self.scroll
    }

    /// Whether a click on this backdrop also reaches the layer beneath.
    pub const fn pass_through(&self) -> bool {
        self.pass_through
    }
}

#[derive(Clone, Debug)]
pub struct Registry {
    regions: Vec<Region>,
    generation: u32,
    has_pointer_exclusions: bool,
    bars: Vec<Bar>,
    releases: Vec<(Id, ReleaseFrom)>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Registry {
    pub fn new(generation: u32) -> Self {
        Registry {
            regions: Vec::new(),
            generation,
            has_pointer_exclusions: false,
            bars: Vec::new(),
            releases: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self, generation: u32) {
        self.regions.clear();
        self.generation = generation;
        self.has_pointer_exclusions = false;
        self.bars.clear();
        self.releases.clear();
    }

    pub const fn generation(&self) -> u32 {
        self.generation
    }

    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    pub fn register_control(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
    ) -> Option<Diagnostic> {
        if area.is_empty() {
            return None;
        }
        let dup = self
            .regions
            .iter()
            .find(|r| r.owner == owner && r.kind == RegionKind::Control)
            .map(|r| Diagnostic::DuplicateId {
                id: owner,
                first: r.area,
                second: area,
            });
        self.push(
            owner,
            PartRef::of(Part::CONTAINER),
            area,
            layer,
            RegionKind::Control,
            None,
        );
        dup
    }

    pub(crate) fn register_keyboard_control(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
    ) -> Option<Diagnostic> {
        if area.is_empty() {
            return None;
        }
        let diagnostic = self.register_control(owner, area, layer);

        for region in self
            .regions
            .iter_mut()
            .filter(|region| region.owner == owner)
        {
            region.pointer_enabled = false;
        }
        self.has_pointer_exclusions = true;
        diagnostic
    }

    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect, layer: LayerId) {
        self.push(owner, part, area, layer, RegionKind::Part, None);
    }

    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect, layer: LayerId) {
        self.push(owner, part, area, layer, RegionKind::Decorative, None);
    }

    pub fn register_scroll(
        &mut self,
        owner: Id,
        area: Rect,
        layer: LayerId,
        axes: Axes,
        head: Headroom,
    ) {
        self.push(
            owner,
            PartRef::of(Part::CONTAINER),
            area,
            layer,
            RegionKind::Scroll,
            Some((axes, head)),
        );
    }

    fn push(
        &mut self,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        kind: RegionKind,
        scroll: Option<(Axes, Headroom)>,
    ) {
        if area.is_empty() {
            return;
        }
        self.push_kind(owner, owner, part, area, layer, kind, false);
        // `push_kind` skips an empty area, so a region is present here.
        if let Some(scroll) = scroll
            && let Some(region) = self.regions.last_mut()
        {
            region.scroll = Some(scroll);
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "internal helper for region registration with explicit owner and layer dispatch"
    )]
    fn push_kind(
        &mut self,
        id: Id,
        owner: Id,
        part: PartRef,
        area: Rect,
        layer: LayerId,
        kind: RegionKind,
        pass_through: bool,
    ) {
        if area.is_empty() {
            return;
        }
        let pointer_enabled = !self.has_pointer_exclusions
            || !self
                .regions
                .iter()
                .any(|region| region.owner == owner && !region.pointer_enabled);
        self.regions.push(Region {
            id,
            owner,
            part,
            area,
            layer,
            kind,
            generation: self.generation,
            scroll: None,
            pointer_enabled,
            pass_through,
        });
    }

    const fn hit_of(r: &Region, pos: Position) -> Hit {
        Hit {
            owner: r.owner,
            part: r.part,
            layer: r.layer,
            kind: r.kind,
            local: Position {
                x: pos.x.saturating_sub(r.area.x),
                y: pos.y.saturating_sub(r.area.y),
            },
        }
    }

    pub fn hit(&self, pos: Position) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.pointer_enabled && r.kind != RegionKind::Scroll && r.area.contains(pos)
            })
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    pub(crate) fn hit_live(&self, pos: Position, layer: LayerId) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.pointer_enabled
                    && r.layer == layer
                    && matches!(r.kind, RegionKind::Control | RegionKind::Part)
                    && r.area.contains(pos)
            })
            .max_by_key(|(i, _)| *i)
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    pub fn hit_scroll(&self, pos: Position, axis: Axis) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.pointer_enabled
                    && r.kind == RegionKind::Scroll
                    && r.area.contains(pos)
                    && r.scroll.is_some_and(|(axes, _)| axes.handles(axis))
            })
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id && r.kind == RegionKind::Control)
            .or_else(|| {
                self.regions
                    .iter()
                    .rev()
                    .find(|r| r.owner == id && r.kind == RegionKind::Decorative)
            })
            .map(|r| r.area)
    }

    pub fn area_of_part(&self, id: Id, part: PartRef) -> Option<Rect> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id && r.part == part)
            .map(|r| r.area)
    }

    pub fn layer_of(&self, id: Id) -> Option<LayerId> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id)
            .map(|r| r.layer)
    }

    pub fn delivers_to(&self, id: Id) -> bool {
        self.regions
            .iter()
            .any(|r| r.owner == id && matches!(r.kind, RegionKind::Control | RegionKind::Part))
    }

    /// Whether `id` owns a region or is a region's control id (a backdrop
    /// hit-tested under a derived id, as in PR #26).
    pub fn has_owner(&self, id: Id) -> bool {
        self.regions.iter().any(|r| r.owner == id || r.id == id)
    }

    /// The topmost region at `pos` of any kind, if any.
    pub fn region_at(&self, pos: Position) -> Option<&Region> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.area.contains(pos))
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| r)
    }

    /// Register a scrollbar bar: the track becomes a part region of the
    /// owning widget (ported from PR #26 `src/core/hit.rs::register_bar`,
    /// commit f4867e9b5).
    pub fn register_bar(&mut self, bar: Bar, layer: LayerId) {
        if bar.track.is_empty() {
            return;
        }
        self.register_part(bar.owner, PartRef::of(Part::TRACK), bar.track, layer);
        self.bars.retain(|b| b.id != bar.id);
        self.bars.push(bar);
    }

    /// Register the draggable thumb of an already-registered bar (ported
    /// from PR #26 `src/core/hit.rs::register_thumb`, commit f4867e9b5).
    /// Hits on the thumb resolve to `owner`, the widget the bar scrolls,
    /// while the region keeps `bar_id` as its control id — PR #26's
    /// id/owner split. A thumb without its bar is ignored.
    pub fn register_thumb(&mut self, bar_id: Id, owner: Id, thumb: Rect, layer: LayerId) {
        if self.bar(bar_id).is_none() {
            return;
        }
        self.push_kind(
            bar_id,
            owner,
            PartRef::of(Part::THUMB),
            thumb,
            layer,
            RegionKind::Part,
            false,
        );
    }

    /// The registered bars, oldest first.
    pub fn bars(&self) -> &[Bar] {
        &self.bars
    }

    /// The bar with the given id, if registered this frame.
    pub fn bar(&self, id: Id) -> Option<Bar> {
        self.bars.iter().rev().find(|b| b.id == id).copied()
    }

    /// Accept a release for `id` from `from` this frame (ported from PR #26
    /// `src/core/hit.rs::accept_release`, commit f4867e9b5).
    pub fn accept_release(&mut self, id: Id, from: ReleaseFrom) {
        self.releases.push((id, from));
    }

    /// Whether `id` accepted a release from `from` this frame. Parity keys
    /// acceptance by the hit's owner (the id a widget registers), where
    /// PR #26 keyed it by the hit's control id (commit f4867e9b5) — a
    /// documented adaptation, not a coarsening.
    pub fn accepts_release(&self, id: Id, from: ReleaseFrom) -> bool {
        self.releases.contains(&(id, from))
    }

    /// Register a light-dismiss backdrop owned by `id` (ported from PR #26
    /// `src/core/hit.rs::register_backdrop`, commit f4867e9b5). The backdrop
    /// catches clicks outside the popup so they reach the owner; with
    /// `pass_through` it lets the click also reach the layer beneath.
    pub fn register_backdrop(&mut self, id: Id, area: Rect, layer: LayerId, pass_through: bool) {
        self.register_backdrop_of(id, id, area, layer, pass_through);
    }

    /// Register a light-dismiss backdrop owned by `owner`, hit-tested as `id`.
    /// The registry answers hits with `owner`; `id` names the backdrop
    /// control for `has_owner` style queries (stored as the region's id, as
    /// PR #26's `HitRegion::plain(id, area)` does).
    pub fn register_backdrop_of(
        &mut self,
        id: Id,
        owner: Id,
        area: Rect,
        layer: LayerId,
        pass_through: bool,
    ) {
        self.push_kind(
            id,
            owner,
            PartRef::of(Part::BACKDROP),
            area,
            layer,
            RegionKind::Backdrop,
            pass_through,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: Id = Id::root("a");
    const B: Id = Id::root("b");
    const C: Id = Id::root("c");

    #[test]
    fn last_registration_wins() {
        let mut h = Registry::new(1);
        assert!(
            h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE)
                .is_none()
        );
        assert!(
            h.register_control(B, Rect::new(2, 2, 3, 3), LayerId::PAGE)
                .is_none()
        );
        assert_eq!(h.hit(Position::new(1, 1)).map(|h| h.owner), Some(A));
        assert_eq!(h.hit(Position::new(3, 3)).map(|h| h.owner), Some(B));
        assert_eq!(h.hit(Position::new(20, 20)), None);
    }

    #[test]
    fn higher_layer_shadows_lower() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));
        let hit = h.hit(Position::new(3, 3));
        assert_eq!(hit.map(|h| (h.owner, h.layer)), Some((B, LayerId(1))));

        h.register_control(C, Rect::new(2, 2, 3, 3), LayerId(1));
        assert_eq!(h.hit(Position::new(3, 3)).map(|h| h.owner), Some(C));
    }

    #[test]
    fn a_lower_layer_region_registered_later_does_not_shadow_a_higher_one() {
        let mut h = Registry::new(1);

        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        let hit = h.hit(Position::new(3, 3)).expect("a covering region");
        assert_eq!((hit.owner, hit.layer), (B, LayerId(1)));

        h.register_scroll(
            C,
            Rect::new(2, 2, 3, 3),
            LayerId(1),
            Axes::V,
            Headroom::default(),
        );
        h.register_scroll(
            A,
            Rect::new(0, 0, 10, 10),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert_eq!(
            h.hit_scroll(Position::new(3, 3), Axis::V).map(|x| x.owner),
            Some(C)
        );
    }

    #[test]
    fn hit_returns_a_lower_layer_region_for_the_outside_click_test() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));

        let hit = h.hit(Position::new(8, 8));
        assert_eq!(hit.map(|h| (h.owner, h.layer)), Some((A, LayerId::PAGE)));
    }

    #[test]
    fn hit_returns_part_ref_not_a_derived_id() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        let part = PartRef::item(Part::ROW, crate::id::ItemKey::num(7));
        h.register_part(A, part, Rect::new(0, 3, 10, 1), LayerId::PAGE);
        let hit = h
            .hit(Position::new(4, 3))
            .map(|h| (h.owner, h.part, h.kind, h.local));
        assert_eq!(hit, Some((A, part, RegionKind::Part, Position::new(4, 0))));
        assert_eq!(h.area_of_part(A, part), Some(Rect::new(0, 3, 10, 1)));
        assert_eq!(h.area_of(A), Some(Rect::new(0, 0, 10, 10)));
    }

    #[test]
    fn hit_scroll_returns_the_innermost_handler_of_the_axis() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 20, 20),
            LayerId::PAGE,
            Axes::Both,
            Headroom::default(),
        );
        h.register_scroll(
            B,
            Rect::new(5, 5, 5, 5),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert_eq!(
            h.hit_scroll(Position::new(6, 6), Axis::V).map(|h| h.owner),
            Some(B)
        );
        assert_eq!(
            h.hit_scroll(Position::new(6, 6), Axis::H).map(|h| h.owner),
            Some(A)
        );
        assert_eq!(
            h.hit_scroll(Position::new(1, 1), Axis::V).map(|h| h.owner),
            Some(A)
        );

        assert_eq!(h.hit(Position::new(6, 6)), None);
    }

    #[test]
    fn hit_scroll_returns_a_region_at_zero_headroom() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 20, 20),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        let hit = h.hit_scroll(Position::new(1, 1), Axis::V);
        assert_eq!(hit.map(|h| h.owner), Some(A));
    }

    #[test]
    fn hit_scroll_skips_regions_that_do_not_handle_the_axis() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 20, 20),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        h.register_control(B, Rect::new(0, 0, 20, 20), LayerId::PAGE);
        assert_eq!(h.hit_scroll(Position::new(1, 1), Axis::H), None);
    }

    #[test]
    fn duplicate_id_is_reported_as_a_diagnostic_not_a_panic() {
        let mut h = Registry::new(1);
        assert!(
            h.register_control(A, Rect::new(0, 0, 1, 1), LayerId::PAGE)
                .is_none()
        );
        let d = h.register_control(A, Rect::new(5, 5, 1, 1), LayerId::PAGE);
        assert_eq!(
            d,
            Some(Diagnostic::DuplicateId {
                id: A,
                first: Rect::new(0, 0, 1, 1),
                second: Rect::new(5, 5, 1, 1)
            })
        );
    }

    #[test]
    fn empty_rects_are_rejected() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 0, 5), LayerId::PAGE);
        h.register_part(A, PartRef::of(Part::LABEL), Rect::ZERO, LayerId::PAGE);
        h.register_scroll(
            A,
            Rect::new(1, 1, 3, 0),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        assert!(h.is_empty());
        assert_eq!(h.area_of(A), None);
    }

    // Regression for review finding 2 (2026-10-21): a rejected empty-area
    // registration must never write scroll state onto the previous,
    // unrelated region.
    #[test]
    fn pushing_an_empty_area_never_touches_the_previous_regions_scroll() {
        let mut h = Registry::new(1);
        h.register_scroll(
            A,
            Rect::new(0, 0, 10, 10),
            LayerId::PAGE,
            Axes::V,
            Headroom::default(),
        );
        let scroll = (Axes::V, Headroom::default());
        h.register_scroll(B, Rect::new(0, 0, 10, 0), LayerId::PAGE, scroll.0, scroll.1);
        assert_eq!(h.len(), 1, "the empty scroll area registers nothing");
        let region = h.regions().first().expect("the scrollable region");
        assert_eq!(
            (region.owner, region.scroll()),
            (A, Some(scroll)),
            "the first region keeps its owner and scroll state"
        );
        h.register_part(A, PartRef::of(Part::LABEL), Rect::ZERO, LayerId::PAGE);
        assert_eq!(h.len(), 1);
        assert_eq!(h.regions().first().map(|r| r.scroll()), Some(Some(scroll)));
    }

    #[test]
    fn generation_bump_invalidates_stale_regions() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 1, 1), LayerId::PAGE);
        assert_eq!(h.regions().first().map(|r| r.generation), Some(1));
        h.reset(2);
        assert!(h.is_empty());
        assert_eq!(h.generation(), 2);
        assert_eq!(h.hit(Position::new(0, 0)), None);
    }

    // Ported from PR #26 src/core/hit.rs `parts_resolve_to_their_owner_and_part`
    // (the bar / thumb half), commit f4867e9b5.
    #[test]
    fn a_bar_resolves_to_its_owners_track_and_the_thumb_is_its_own_part() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 20, 5), LayerId::PAGE);
        h.register_bar(
            Bar {
                id: B,
                owner: A,
                axis: Axis::V,
                track: Rect::new(19, 0, 1, 5),
            },
            LayerId::PAGE,
        );
        let on_track = h.hit(Position::new(19, 2)).expect("the track");
        assert_eq!(
            (on_track.owner, on_track.part),
            (A, PartRef::of(Part::TRACK)),
            "a scrollbar resolves to its owner's track"
        );
        assert_eq!(
            h.bars().iter().map(|b| b.owner).collect::<Vec<_>>(),
            vec![A],
            "the frame's scrollbars are listed"
        );
        assert_eq!(h.bar(B).map(|b| b.axis), Some(Axis::V));

        h.register_thumb(B, A, Rect::new(19, 1, 1, 2), LayerId::PAGE);
        let on_thumb = h.hit(Position::new(19, 2)).expect("the thumb");
        assert_eq!(
            (on_thumb.owner, on_thumb.part),
            (A, PartRef::of(Part::THUMB)),
            "the thumb resolves to the bar's owner, its own part"
        );
        let thumb_region = h
            .regions()
            .iter()
            .rev()
            .find(|r| r.part == PartRef::of(Part::THUMB))
            .expect("the thumb region");
        assert_eq!(
            (thumb_region.id, thumb_region.owner),
            (B, A),
            "the thumb keeps the bar's id and the owner's owner"
        );
        assert_eq!(
            h.hit(Position::new(19, 4)).map(|x| x.part),
            Some(PartRef::of(Part::TRACK)),
            "the track below the thumb keeps answering"
        );
        assert_eq!(h.hit(Position::new(25, 2)), None, "outside every region");
    }

    #[test]
    fn a_thumb_without_its_bar_is_ignored_and_an_empty_track_never_registers() {
        let mut h = Registry::new(1);
        h.register_thumb(B, A, Rect::new(0, 0, 1, 1), LayerId::PAGE);
        assert!(h.is_empty(), "a thumb needs its bar first");
        h.register_bar(
            Bar {
                id: B,
                owner: A,
                axis: Axis::V,
                track: Rect::ZERO,
            },
            LayerId::PAGE,
        );
        assert!(
            h.is_empty() && h.bars().is_empty(),
            "an empty track is no bar"
        );
    }

    // Ported from PR #26 `accept_release` / `accepts_release`, commit f4867e9b5.
    #[test]
    fn releases_are_accepted_per_widget_and_source() {
        let mut h = Registry::new(1);
        assert!(!h.accepts_release(A, ReleaseFrom::Press(A)));
        h.accept_release(A, ReleaseFrom::Press(A));
        h.accept_release(B, ReleaseFrom::Secondary);
        assert!(h.accepts_release(A, ReleaseFrom::Press(A)));
        assert!(!h.accepts_release(A, ReleaseFrom::Secondary));
        assert!(h.accepts_release(B, ReleaseFrom::Secondary));
        assert!(!h.accepts_release(B, ReleaseFrom::Press(A)));
        h.reset(2);
        assert!(
            !h.accepts_release(A, ReleaseFrom::Press(A)),
            "a reset drops the frame's releases"
        );
    }

    // Ported from PR #26 `register_backdrop` / `register_backdrop_of`,
    // commit f4867e9b5 (adapted: parity answers hits with the owner).
    #[test]
    fn a_backdrop_catches_clicks_for_its_owner() {
        let mut h = Registry::new(1);
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        h.register_backdrop_of(B, A, Rect::new(0, 0, 40, 20), LayerId(1), false);
        let on_backdrop = h.hit(Position::new(30, 15)).expect("the backdrop");
        assert_eq!(
            (on_backdrop.owner, on_backdrop.part),
            (A, PartRef::of(Part::BACKDROP))
        );
        assert!(
            h.has_owner(B),
            "the backdrop's own control id stays queryable"
        );
        assert!(h.has_owner(A), "the owner stays queryable");
        let backdrop = h
            .regions()
            .iter()
            .find(|r| r.kind == RegionKind::Backdrop)
            .expect("the backdrop region");
        assert_eq!(
            (backdrop.id, backdrop.owner),
            (B, A),
            "the region keeps the backdrop id alongside its owner"
        );
        assert!(
            !h.region_at(Position::new(30, 15))
                .is_some_and(|r| r.pass_through()),
            "a solid backdrop does not pass through"
        );
        // On the backdrop's layer the backdrop also wins over the page
        // control it covers: the popup is the top layer.
        let over_page = h.hit(Position::new(2, 2)).expect("covered by backdrop");
        assert_eq!(
            (over_page.owner, over_page.part, over_page.layer),
            (A, PartRef::of(Part::BACKDROP), LayerId(1))
        );

        h.register_backdrop(C, Rect::new(0, 0, 5, 5), LayerId(2), true);
        assert!(
            h.region_at(Position::new(2, 2))
                .is_some_and(|r| r.pass_through()),
            "a pass-through backdrop says so"
        );
        h.register_backdrop(C, Rect::ZERO, LayerId(2), false);
    }
}

#[cfg(test)]
mod runtime_tests {
    use ratatui_core::layout::Rect;

    use crate::event::{KeyCode, MouseKind};
    use crate::id::Id;
    use crate::layer::LayerSpec;
    use crate::runtime::stub::{Control, Stub, key, mouse, runtime, step};

    const PAGE_A: Id = Id::root("page.a");
    const DLG: Id = Id::root("dlg");
    const OK: Id = Id::root("dlg.ok");

    #[test]
    fn inert_below_registers_nothing() {
        let s = Stub {
            page: vec![Control::new(PAGE_A, Rect::new(0, 0, 10, 1))],
            layers: vec![(DLG, vec![Control::new(OK, Rect::new(2, 4, 5, 1))])],
            ..Stub::default()
        };
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert!(
            rt.registry().area_of(PAGE_A).is_some(),
            "the page registers"
        );

        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert!(
            rt.registry().area_of(PAGE_A).is_none(),
            "a page control below an inert layer registers nothing"
        );
        assert!(rt.registry().area_of(OK).is_some());

        rt.app_mut().log.clear();
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        assert!(!rt.app().saw(PAGE_A, "Press"), "{:?}", rt.app().log);
    }
}
