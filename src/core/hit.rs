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
    /// A light-dismiss backdrop ([`HitRegistry::register_backdrop`]): a
    /// click on it means "outside the popup that owns it".
    pub backdrop: bool,
    /// A modal's own body ([`HitRegistry::register_surface`]): it catches
    /// presses so nothing falls through, but it is no control, so a press
    /// on it is dead space inside the modal.
    pub surface: bool,
}

/// Which way a scrollbar moves its content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Vertical,
    Horizontal,
}

/// A scrollbar as drawn this frame: its hit id, the container it scrolls,
/// its axis and its track. The pointer layer grabs it on press, so a drag
/// carries the axis and track instead of each widget inferring them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bar {
    pub id: WidgetId,
    pub owner: WidgetId,
    pub axis: Axis,
    pub track: Rect,
}

#[derive(Debug, Default, Clone)]
pub struct HitRegistry {
    regions: Vec<HitRegion>,
    /// Barrier stack: each entry is the region index a layer starts at.
    /// Only the top layer is reachable; the one below it is what a
    /// pass-through backdrop lets a click fall into.
    barriers: Vec<usize>,
    /// Scrollbars registered this frame.
    bars: Vec<Bar>,
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
            backdrop: false,
            surface: false,
        });
    }

    /// Register a modal's body (a dialog's or a modal frame's surface): it
    /// blocks presses from falling through like [`Self::register`], and
    /// [`Self::is_surface`] tells the pointer layer a press on it is dead
    /// space inside the modal
    /// ([`crate::core::pointer::PointerState::surface_press`]).
    pub fn register_surface(&mut self, id: WidgetId, area: Rect) {
        if area.is_empty() {
            return;
        }
        self.regions.push(HitRegion {
            id,
            area,
            scroll_only: false,
            pass_through: false,
            hover_only: false,
            backdrop: false,
            surface: true,
        });
    }

    /// Whether `id` was registered this frame as a modal's surface.
    pub fn is_surface(&self, id: WidgetId) -> bool {
        self.regions.iter().rev().any(|r| r.id == id && r.surface)
    }

    /// Register a scrollbar: a clickable region that the pointer layer
    /// also knows as `owner`'s bar on `axis`.
    pub fn register_bar(&mut self, bar: Bar) {
        if bar.track.is_empty() {
            return;
        }
        self.register(bar.id, bar.track);
        self.bars.push(bar);
    }

    /// The scrollbar registered under `id` this frame.
    pub fn bar(&self, id: WidgetId) -> Option<Bar> {
        self.bars.iter().rev().find(|b| b.id == id).copied()
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
            backdrop: true,
            surface: false,
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
            backdrop: false,
            surface: false,
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
            backdrop: false,
            surface: false,
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

    /// When the topmost clickable region under `pos` is a light-dismiss
    /// backdrop: its id, and the clickable region the same position lands
    /// on in the first non-empty layer beneath it (whatever its
    /// `pass_through`). A secondary click there closes the popup and opens
    /// the context menu of what is beneath (macOS), see
    /// [`crate::core::pointer::PointerState::route_secondary`]. Empty
    /// layers (a modal layer opened only to hold the popup) are skipped; a
    /// non-empty layer with nothing under `pos` (a dialog the click missed)
    /// stops the search, since what is under a dialog is unreachable.
    pub fn backdrop_at(&self, pos: Position) -> Option<(WidgetId, Option<WidgetId>)> {
        let top = self
            .reachable()
            .iter()
            .rev()
            .find(|r| r.clickable() && r.area.contains(pos))?;
        if !top.backdrop {
            return None;
        }
        let mut end = *self.barriers.last()?;
        for &start in self.barriers.iter().rev().skip(1) {
            let layer = &self.regions[start..end];
            if !layer.is_empty() {
                return Some((top.id, Self::top_clickable(layer, pos)));
            }
            end = start;
        }
        Some((top.id, Self::top_clickable(&self.regions[..end], pos)))
    }

    fn top_clickable(layer: &[HitRegion], pos: Position) -> Option<WidgetId> {
        layer
            .iter()
            .rev()
            .find(|r| r.clickable() && !r.backdrop && r.area.contains(pos))
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

    /// Every region a wheel at `pos` can reach, topmost first: the
    /// candidates [`hit_scroll`](Self::hit_scroll) picks the first of. A
    /// wheel router offers the notch to each in turn until one owner takes
    /// it, so a wheel over a list row (registered above its list) still
    /// scrolls the list.
    pub fn scroll_stack(&self, pos: Position) -> Vec<WidgetId> {
        let mut out: Vec<WidgetId> = Vec::new();
        for r in self.reachable().iter().rev() {
            if !r.hover_only && r.area.contains(pos) && !out.contains(&r.id) {
                out.push(r.id);
            }
        }
        out
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
    fn scroll_stack_lists_the_row_then_its_list_once_each() {
        let mut h = HitRegistry::default();
        let list = WidgetId::of("list");
        let row = list.child(1);
        h.register(list, Rect::new(0, 0, 20, 5));
        h.register_scroll(list, Rect::new(0, 0, 20, 5));
        h.register(row, Rect::new(0, 1, 20, 1));
        assert_eq!(h.scroll_stack(Position::new(3, 1)), vec![row, list]);
        assert_eq!(h.scroll_stack(Position::new(3, 0)), vec![list]);
    }

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
    fn a_backdrop_reports_what_is_beneath_it_across_empty_layers() {
        let mut h = HitRegistry::default();
        let row = WidgetId::of("row");
        let pop = WidgetId::of("pop");
        let back = WidgetId::of("pop.backdrop");
        h.register(row, Rect::new(0, 5, 20, 1));
        // a modal layer opened only to hold the popup
        h.push_barrier();
        h.push_barrier();
        h.register_backdrop(back, Rect::new(0, 0, 40, 20), false);
        h.register(pop, Rect::new(10, 0, 10, 4));
        assert_eq!(h.backdrop_at(Position::new(2, 5)), Some((back, Some(row))));
        assert_eq!(h.backdrop_at(Position::new(2, 9)), Some((back, None)));
        assert_eq!(h.backdrop_at(Position::new(12, 1)), None, "on the popup");
    }

    #[test]
    fn a_backdrop_over_a_dialog_never_reaches_the_page_under_the_dialog() {
        let mut h = HitRegistry::default();
        let row = WidgetId::of("row");
        let dialog = WidgetId::of("dialog");
        let back = WidgetId::of("select.backdrop");
        h.register(row, Rect::new(0, 5, 20, 1));
        h.push_barrier();
        h.register(dialog, Rect::new(10, 2, 20, 10));
        h.push_barrier();
        h.register_backdrop(back, Rect::new(0, 0, 40, 20), false);
        assert_eq!(h.backdrop_at(Position::new(2, 5)), Some((back, None)));
        assert_eq!(
            h.backdrop_at(Position::new(12, 5)),
            Some((back, Some(dialog)))
        );
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
