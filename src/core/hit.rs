//! Mouse hit-testing.
//!
//! Widgets register the rectangles they occupy while rendering. The registry
//! is rebuilt every frame, so hit regions always match what is on screen.
//! Later registrations win, which makes overlays (dialogs) naturally shadow
//! the content below them.

use ratatui::layout::{Position, Rect};

use super::id::WidgetId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitRegion {
    pub id: WidgetId,
    pub area: Rect,
    /// A region that only serves as a scroll container: wheel events are
    /// routed to it, but it is not hoverable/clickable on its own.
    pub scroll_only: bool,
    /// A light-dismiss backdrop whose click also reaches the region
    /// beneath it (see [`HitRegistry::pass_through_target`]).
    pub pass_through: bool,
    /// A region that only takes hover (a cut title revealing its full
    /// text): presses, clicks and the wheel go to what is beneath it.
    pub hover_only: bool,
}

#[derive(Debug, Default, Clone)]
pub struct HitRegistry {
    regions: Vec<HitRegion>,
    /// Barrier stack: each entry is the region index a layer starts at.
    /// Only the top layer is reachable; the one below it is what a
    /// pass-through backdrop lets a click fall into.
    barriers: Vec<usize>,
}

impl HitRegion {
    fn clickable(&self) -> bool {
        !self.scroll_only && !self.hover_only
    }
}

impl HitRegistry {
    pub fn register(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            id,
            area,
            scroll_only: false,
            pass_through: false,
            hover_only: false,
        });
    }

    /// Register a light-dismiss backdrop for a popup layer. With
    /// `pass_through`, a click on it also lands on what is beneath.
    pub fn register_backdrop(&mut self, id: WidgetId, area: Rect, pass_through: bool) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            id,
            area,
            scroll_only: false,
            pass_through,
            hover_only: false,
        });
    }

    pub fn register_scroll(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            id,
            area,
            scroll_only: true,
            pass_through: false,
            hover_only: false,
        });
    }

    /// Register a hover-only region: [`HitRegistry::hover_at`] finds it,
    /// while presses, clicks and the wheel reach whatever is beneath it.
    pub fn register_hover(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            id,
            area,
            scroll_only: false,
            pass_through: false,
            hover_only: true,
        });
    }

    /// Everything registered before this call is unreachable by the mouse.
    pub fn push_barrier(&mut self) {
        self.barriers.push(self.regions.len());
    }

    fn reachable(&self) -> &[HitRegion] {
        match self.barriers.last() {
            Some(&start) => &self.regions[start..],
            None => &self.regions,
        }
    }

    /// The layer directly under the top barrier.
    fn beneath(&self) -> &[HitRegion] {
        match self.barriers.as_slice() {
            [] => &[],
            [top] => &self.regions[..*top],
            [.., below, top] => &self.regions[*below..*top],
        }
    }

    /// When `id` is a pass-through backdrop under `pos`, the clickable
    /// region beneath it that the same click should also reach.
    pub fn pass_through_target(&self, id: WidgetId, pos: Position) -> Option<WidgetId> {
        let top = self
            .reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))?;
        if top.id != id || !top.pass_through {
            return None;
        }
        self.beneath()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))
            .map(|r| r.id)
    }

    /// Topmost clickable widget under the position.
    pub fn hit(&self, pos: Position) -> Option<WidgetId> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))
            .map(|r| r.id)
    }

    /// Topmost hoverable widget under the position: the clickable ones and
    /// hover-only regions ([`HitRegistry::register_hover`]).
    pub fn hover_at(&self, pos: Position) -> Option<WidgetId> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| !r.scroll_only && r.area.contains(pos))
            .map(|r| r.id)
    }

    /// Topmost scroll container under the position (any registered region
    /// whose owner can scroll counts, including scroll-only ones).
    pub fn hit_scroll(&self, pos: Position) -> Option<WidgetId> {
        self.reachable()
            .iter()
            .rev()
            .find(|r| !r.hover_only && r.area.contains(pos))
            .map(|r| r.id)
    }

    pub fn len(&self) -> usize {
        self.regions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }

    /// Clip every region registered at index `from` or later to `area`:
    /// regions are cut to the visible part, and dropped when nothing of
    /// them is left. Backs [`RenderCtx::clip`](crate::ui::ctx::RenderCtx::clip),
    /// so a widget drawn into a clip rect is never clickable outside it.
    pub fn clip_since(&mut self, from: usize, area: Rect) {
        let from = from.min(self.regions.len());
        let mut kept = Vec::with_capacity(self.regions.len() - from);
        for mut r in self.regions.drain(from..) {
            r.area = r.area.intersection(area);
            if !r.area.is_empty() {
                kept.push(r);
            }
        }
        self.regions.extend(kept);
    }

    pub fn area_of(&self, id: WidgetId) -> Option<Rect> {
        self.regions
            .iter()
            .rev()
            .find(|r| r.id == id)
            .map(|r| r.area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_only_regions_take_hover_and_let_clicks_and_the_wheel_through() {
        let mut h = HitRegistry::default();
        let card = WidgetId::of("card");
        let title = WidgetId::of("title");
        h.register(card, Rect::new(0, 0, 20, 5));
        h.register_scroll(card, Rect::new(0, 0, 20, 5));
        h.register_hover(title, Rect::new(2, 0, 8, 1));
        let on_title = Position::new(3, 0);
        assert_eq!(h.hover_at(on_title), Some(title));
        assert_eq!(h.hit(on_title), Some(card), "the click is the card's");
        assert_eq!(h.hit_scroll(on_title), Some(card));
        assert_eq!(h.hover_at(Position::new(3, 2)), Some(card));
    }

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
    fn pass_through_backdrops_reveal_the_layer_beneath() {
        let mut h = HitRegistry::default();
        let below = WidgetId::of("below");
        let back = WidgetId::of("back");
        let solid = WidgetId::of("solid");
        h.register(below, Rect::new(0, 0, 10, 10));
        h.push_barrier();
        h.register_backdrop(back, Rect::new(0, 0, 10, 10), true);
        let p = Position::new(2, 2);
        assert_eq!(h.hit(p), Some(back));
        assert_eq!(h.pass_through_target(back, p), Some(below));
        h.push_barrier();
        h.register_backdrop(solid, Rect::new(0, 0, 10, 10), false);
        assert_eq!(h.pass_through_target(solid, p), None);
    }

    #[test]
    fn scroll_only_regions_ignore_hover() {
        let mut h = HitRegistry::default();
        let a = WidgetId::of("a");
        h.register_scroll(a, Rect::new(0, 0, 10, 10));
        assert_eq!(h.hit(Position::new(1, 1)), None);
        assert_eq!(h.hit_scroll(Position::new(1, 1)), Some(a));
    }
}
