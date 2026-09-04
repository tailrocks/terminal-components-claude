//! Hit registry (`COMPONENT_ARCHITECTURE.md` §3.3, §8.3, §21 items 12–13).
//!
//! Every region carries `{owner, part, area, layer, kind, generation}`. The
//! registry is rebuilt per frame; last registration wins; `hit` returns the
//! topmost region regardless of layer and the runtime compares layers, which
//! is what makes "click outside" a real test.

use ratatui_core::layout::{Position, Rect};

use crate::diagnostics::Diagnostic;
use crate::event::Axis;
use crate::id::{Id, Part, PartRef};
use crate::layer::LayerId;

/// What a region is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum RegionKind {
    /// Focusable, delivers intents.
    Control,
    /// Sub-region of a `Control`; delivers to the control's owner.
    Part,
    /// Wheel target only.
    Scroll,
    /// Paints and answers `area_of`; never delivers, never diagnosed.
    Decorative,
}

/// A resolved pointer hit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hit {
    /// The owner.
    pub owner: Id,
    /// The part under the pointer.
    pub part: PartRef,
    /// The region's layer.
    pub layer: LayerId,
    /// The region kind.
    pub kind: RegionKind,
    /// Position relative to the region's origin.
    pub local: Position,
}

/// How far a scroll region can move on each side.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Headroom {
    /// Rows above.
    pub up: u16,
    /// Rows below.
    pub down: u16,
    /// Columns to the left.
    pub left: u16,
    /// Columns to the right.
    pub right: u16,
}

/// Which axes a scroll region handles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axes {
    /// Vertical only.
    V,
    /// Horizontal only.
    H,
    /// Both.
    Both,
}

impl Axes {
    /// Whether the region handles `axis`.
    pub const fn handles(self, axis: Axis) -> bool {
        matches!(
            (self, axis),
            (Axes::Both, _) | (Axes::V, Axis::V) | (Axes::H, Axis::H)
        )
    }
}

/// One registered region.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Region {
    /// The owner.
    pub owner: Id,
    /// The part (`PartRef::of(Part::CONTAINER)` for a control's own area).
    pub part: PartRef,
    /// The area.
    pub area: Rect,
    /// The layer.
    pub layer: LayerId,
    /// The kind.
    pub kind: RegionKind,
    /// The frame generation.
    pub generation: u32,
    scroll: Option<(Axes, Headroom)>,
}

impl Region {
    /// The scroll axes and headroom of a `Scroll` region.
    pub const fn scroll(&self) -> Option<(Axes, Headroom)> {
        self.scroll
    }
}

/// The per-frame region list.
#[derive(Clone, Debug)]
pub struct Registry {
    regions: Vec<Region>,
    generation: u32,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Registry {
    /// An empty registry for generation `generation`.
    pub fn new(generation: u32) -> Self {
        Registry {
            regions: Vec::new(),
            generation,
        }
    }

    /// Reset for generation `generation`, keeping the allocation.
    pub(crate) fn reset(&mut self, generation: u32) {
        self.regions.clear();
        self.generation = generation;
    }

    /// The generation.
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Every region, in registration order.
    pub fn regions(&self) -> &[Region] {
        &self.regions
    }

    /// The region count.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Whether nothing is registered.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Register a `Control` region. A second `Control` for the same owner in
    /// one frame is recorded as `DuplicateId`, never a panic.
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

    /// Register a `Part` region under `owner`.
    pub fn register_part(&mut self, owner: Id, part: PartRef, area: Rect, layer: LayerId) {
        self.push(owner, part, area, layer, RegionKind::Part, None);
    }

    /// Register a `Decorative` region under `owner`.
    pub fn register_decor(&mut self, owner: Id, part: PartRef, area: Rect, layer: LayerId) {
        self.push(owner, part, area, layer, RegionKind::Decorative, None);
    }

    /// Register a `Scroll` region.
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
        self.regions.push(Region {
            owner,
            part,
            area,
            layer,
            kind,
            generation: self.generation,
            scroll,
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

    /// The topmost non-scroll region covering `pos`, ordered by
    /// `(layer, registration index)`.
    ///
    /// z-order is the **layer** order, not the call order (§9.1): a page
    /// control drawn after a popover must not shadow the popover, or the
    /// runtime reads `hit.layer < top_layer` and treats a click *on* the
    /// popover as an outside click.
    pub fn hit(&self, pos: Position) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind != RegionKind::Scroll && r.area.contains(pos))
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    /// The topmost intent-delivering region at `pos` on `layer`.
    ///
    /// Decorative regions are skipped before ordering, so decoration painted
    /// after a live part cannot hide that part from pointer movement. Regions
    /// below `layer` are never considered.
    pub(crate) fn hit_live(&self, pos: Position, layer: LayerId) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.layer == layer
                    && matches!(r.kind, RegionKind::Control | RegionKind::Part)
                    && r.area.contains(pos)
            })
            .max_by_key(|(i, _)| *i)
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    /// The innermost scroll region covering `pos` that handles `axis`,
    /// returned even at zero headroom (§8.3). Ordered by
    /// `(layer, registration index)`, like [`Registry::hit`].
    pub fn hit_scroll(&self, pos: Position, axis: Axis) -> Option<Hit> {
        self.regions
            .iter()
            .enumerate()
            .filter(|(_, r)| {
                r.kind == RegionKind::Scroll
                    && r.area.contains(pos)
                    && r.scroll.is_some_and(|(axes, _)| axes.handles(axis))
            })
            .max_by_key(|(i, r)| (r.layer, *i))
            .map(|(_, r)| Self::hit_of(r, pos))
    }

    /// The last `Control` region of `id`, else its last `Decorative` region.
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

    /// The last region of `id` tagged `part`, any kind.
    pub fn area_of_part(&self, id: Id, part: PartRef) -> Option<Rect> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id && r.part == part)
            .map(|r| r.area)
    }

    /// The region of `id` under `layer` (for cursor and capture checks).
    pub fn layer_of(&self, id: Id) -> Option<LayerId> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.owner == id)
            .map(|r| r.layer)
    }

    /// Whether `id` registered a `Control` or `Part` region (§3.3 step 9).
    pub fn delivers_to(&self, id: Id) -> bool {
        self.regions
            .iter()
            .any(|r| r.owner == id && matches!(r.kind, RegionKind::Control | RegionKind::Part))
    }

    /// Whether `id` registered anything.
    pub fn has_owner(&self, id: Id) -> bool {
        self.regions.iter().any(|r| r.owner == id)
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
        // and the last registration on the *same* layer still wins
        h.register_control(C, Rect::new(2, 2, 3, 3), LayerId(1));
        assert_eq!(h.hit(Position::new(3, 3)).map(|h| h.owner), Some(C));
    }

    /// z-order is the layer order, **not** the call order (§9.1). A page
    /// control registered after a popover must not shadow it: the runtime
    /// compares `hit.layer < top_layer` to detect an outside click, so a
    /// registration-ordered `hit` would dismiss the popover on a click that
    /// landed on it (MA-1).
    #[test]
    fn a_lower_layer_region_registered_later_does_not_shadow_a_higher_one() {
        let mut h = Registry::new(1);
        // the popover draws first, the page control after it
        h.register_control(B, Rect::new(2, 2, 3, 3), LayerId(1));
        h.register_control(A, Rect::new(0, 0, 10, 10), LayerId::PAGE);
        let hit = h.hit(Position::new(3, 3)).expect("a covering region");
        assert_eq!((hit.owner, hit.layer), (B, LayerId(1)));
        // scroll routing obeys the same order
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
        // a point outside the dialog still resolves to the page region; the
        // runtime compares `hit.layer < top_layer` to decide "outside"
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
        // scroll regions are not pointer targets
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
}

/// MI-3: `inert_below` is enforced by `Ui::register_entry`, not by
/// `Registry`. At the registry level the old test only proved that an
/// unregistered region is not hit.
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
        // `LayerSpec::modal` sets `inert_below`
        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));
        assert!(
            rt.registry().area_of(PAGE_A).is_none(),
            "a page control below an inert layer registers nothing"
        );
        assert!(rt.registry().area_of(OK).is_some());
        // and a click on the page's old rect reaches nothing on that layer
        rt.app_mut().log.clear();
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        assert!(!rt.app().saw(PAGE_A, "Press"), "{:?}", rt.app().log);
    }
}
