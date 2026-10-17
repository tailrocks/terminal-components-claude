//! Mouse hit-testing on the shared runtime registry.
//!
//! Widgets register the rectangles they occupy while rendering. The registry
//! is rebuilt every frame, so hit regions always match what is on screen.
//! Later registrations win within a layer, which makes overlays (dialogs)
//! naturally shadow the content below them.
//!
//! WI-HOLLA-FOCUS-01: the region store, ordering, and topmost lookup are the
//! shared `termrock::Registry`; this struct is only a [`WidgetId`] bridge
//! plus the modal barrier policy (one layer per barrier; hits below the
//! barrier floor are unreachable). There is exactly one hit engine.

use std::collections::HashMap;

use ratatui::layout::{Position, Rect};

use super::id::WidgetId;

#[derive(Debug, Default, Clone)]
pub struct HitRegistry {
    inner: termrock::Registry,
    layer: termrock::LayerId,
    floor: Option<termrock::LayerId>,
    owners: HashMap<termrock::Id, WidgetId>,
}

impl HitRegistry {
    pub fn register(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        // A second Control for the same owner in one frame is routine here
        // (the tree registers each toggle twice); the shared registry keeps
        // both regions and reports the duplicate, which has no Holla
        // plumbing to go to.
        let _ = self
            .inner
            .register_control(id.runtime_id(), area, self.layer);
        self.owners.insert(id.runtime_id(), id);
    }

    pub fn register_scroll(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        // Holla collapses every wheel kind to one delta, so containers
        // accept both axes; wheel routing never depends on headroom.
        self.inner.register_scroll(
            id.runtime_id(),
            area,
            self.layer,
            termrock::Axes::Both,
            termrock::Headroom::default(),
        );
        self.owners.insert(id.runtime_id(), id);
    }

    /// Everything registered before this call is unreachable by the mouse.
    /// Barriers stack (a popup inside a dialog): each one moves
    /// registration to a fresh layer above the previous floor.
    pub fn push_barrier(&mut self) {
        self.layer = termrock::LayerId(self.layer.index().saturating_add(1));
        self.floor = Some(self.layer);
    }

    fn resolve(&self, hit: Option<termrock::Hit>) -> Option<WidgetId> {
        hit.filter(|h| self.floor.is_none_or(|f| h.layer >= f))
            .and_then(|h| self.owners.get(&h.owner).copied())
    }

    /// Topmost hoverable/clickable widget under the position.
    pub fn hit(&self, pos: Position) -> Option<WidgetId> {
        self.resolve(self.inner.hit(pos))
    }

    /// Topmost scroll container under the position: the interaction
    /// contract routes the wheel to the container, never to the clickable
    /// row beneath the pointer.
    pub fn hit_scroll(&self, pos: Position, axis: termrock::Axis) -> Option<WidgetId> {
        self.resolve(self.inner.hit_scroll(pos, axis))
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn area_of(&self, id: WidgetId) -> Option<Rect> {
        self.inner.area_of(id.runtime_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use termrock::Axis;

    #[test]
    fn topmost_wins() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        let b = WidgetId::of("b");
        h.register(a, Rect::new(0, 0, 10, 10));
        h.register(b, Rect::new(2, 2, 3, 3));
        assert_eq!(h.hit(Position::new(1, 1)), Some(a));
        assert_eq!(h.hit(Position::new(3, 3)), Some(b));
        assert_eq!(h.hit(Position::new(20, 20)), None);
    }

    #[test]
    fn barrier_shadows_lower_regions() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        let b = WidgetId::of("b");
        h.register(a, Rect::new(0, 0, 10, 10));
        h.push_barrier();
        h.register(b, Rect::new(2, 2, 3, 3));
        assert_eq!(h.hit(Position::new(1, 1)), None);
        assert_eq!(h.hit(Position::new(3, 3)), Some(b));
    }

    #[test]
    fn stacked_barriers_shadow_in_order() {
        let mut h = HitRegistry::default();
        let (page, dialog, popup) = (
            WidgetId::of("page"),
            WidgetId::of("dialog"),
            WidgetId::of("popup"),
        );
        h.register(page, Rect::new(0, 0, 10, 10));
        h.push_barrier();
        h.register(dialog, Rect::new(2, 2, 6, 6));
        h.push_barrier();
        h.register(popup, Rect::new(3, 3, 2, 2));
        assert_eq!(h.hit(Position::new(1, 1)), None);
        assert_eq!(h.hit(Position::new(2, 2)), None);
        assert_eq!(h.hit(Position::new(3, 3)), Some(popup));
    }

    #[test]
    fn scroll_regions_ignore_hover_but_take_the_wheel() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        h.register_scroll(a, Rect::new(0, 0, 10, 10));
        assert_eq!(h.hit(Position::new(1, 1)), None);
        assert_eq!(h.hit_scroll(Position::new(1, 1), Axis::V), Some(a));
        assert_eq!(h.hit_scroll(Position::new(1, 1), Axis::H), Some(a));
        assert_eq!(h.hit_scroll(Position::new(20, 20), Axis::V), None);
    }

    #[test]
    fn wheel_targets_the_container_beneath_clickable_rows() {
        let mut h = HitRegistry::default();
        let list = WidgetId::of("list");
        h.register(list, Rect::new(0, 0, 10, 10));
        h.register_scroll(list, Rect::new(0, 0, 10, 10));
        h.register(list.child(0), Rect::new(0, 0, 10, 1));
        assert_eq!(h.hit(Position::new(1, 0)), Some(list.child(0)));
        assert_eq!(h.hit_scroll(Position::new(1, 0), Axis::V), Some(list));
    }

    #[test]
    fn duplicate_control_registration_is_tolerated() {
        let mut h = HitRegistry::default();
        let t = WidgetId::of("toggle");
        h.register(t, Rect::new(0, 0, 4, 1));
        h.register(t, Rect::new(0, 0, 4, 1));
        assert_eq!(h.hit(Position::new(1, 0)), Some(t));
        assert_eq!(h.len(), 2);
    }

    #[test]
    fn area_lookup_ignores_the_barrier() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        let area = Rect::new(0, 0, 10, 10);
        h.register(a, area);
        h.push_barrier();
        assert_eq!(h.area_of(a), Some(area));
        assert_eq!(h.hit(Position::new(1, 1)), None);
    }
}
