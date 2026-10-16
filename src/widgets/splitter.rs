//! Split handle: the gap strip between two panes of a [`Split`]. Mouse-only
//! affordance (keyboard resize stays a chord on the owning pane). Rest,
//! hover and pressed walk the theme's seam ladder ([`Kind::Seam`]): a quiet
//! rule at rest, a brighter heavy rule under the pointer, the brightest
//! heavy rule while dragged, so the affordance survives monochrome
//! terminals.
//!
//! Pointer behaviour follows desktop split views: the press target is the
//! seam padded by [`Splitter::hit_pad`] cells on each side (a 1-cell rule
//! is a hard target), a drag moves the seam cell by cell and keeps the
//! offset the pointer grabbed it at, and a double-click restores the
//! starting ratio ([`SeamDoubleClick`]).

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::{RenderCtx, VisualState};
use crate::ui::layout::{Split, SplitDir};

/// What a double-click on the seam does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeamDoubleClick {
    /// Restore the split's starting ratio ([`Split::reset`]), as a
    /// double-click on a macOS or Windows split divider restores it.
    #[default]
    Reset,
    /// Nothing.
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Splitter {
    pub id: WidgetId,
    pub dir: SplitDir,
    /// The drawn seam strip.
    pub area: Rect,
    /// Extra cells on each side of the seam, across the drag axis, that
    /// also take the press where no pane control claims them. Default 1.
    pub hit_pad: u16,
    /// Double-click policy. Default [`SeamDoubleClick::Reset`].
    pub double_click: SeamDoubleClick,
    /// Pointer offset from the split's gap start when the drag started,
    /// so the seam keeps its place under the pointer instead of jumping.
    grab: Option<i32>,
}

impl Splitter {
    pub const fn new(id: WidgetId, dir: SplitDir) -> Self {
        Self {
            id,
            dir,
            area: Rect::ZERO,
            hit_pad: 1,
            double_click: SeamDoubleClick::Reset,
            grab: None,
        }
    }

    pub const fn hit_pad(mut self, cells: u16) -> Self {
        self.hit_pad = cells;
        self
    }

    pub const fn double_click(mut self, policy: SeamDoubleClick) -> Self {
        self.double_click = policy;
        self
    }

    /// The region that takes presses: the drawn seam widened by
    /// [`Self::hit_pad`] across the drag axis, clipped to `clip`.
    pub fn hit_area(&self, clip: Rect) -> Rect {
        if self.area.is_empty() {
            return Rect::ZERO;
        }
        let p = self.hit_pad;
        let r = match self.dir {
            SplitDir::Horizontal => {
                let x = self.area.x.saturating_sub(p);
                Rect::new(x, self.area.y, self.area.right() + p - x, self.area.height)
            }
            SplitDir::Vertical => {
                let y = self.area.y.saturating_sub(p);
                Rect::new(self.area.x, y, self.area.width, self.area.bottom() + p - y)
            }
        };
        r.intersection(clip)
    }

    /// Draw the handle in `area` (the split's gap strip) and register it.
    /// Draws nothing when the strip is empty (gap 0 or maximised).
    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.render_held(area, buf, ctx, bg, false);
    }

    /// [`Self::render`] for an owner that tracks the drag itself: `held`
    /// shows the pressed seam even when the pointer left the strip.
    pub fn render_held(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        held: bool,
    ) {
        let area = area.intersection(*buf.area());
        self.area = area;
        // a drag keeps the seam pressed while the pointer runs ahead of it
        let pressed = held || ctx.interaction.pressed == Some(self.id);
        if !pressed {
            // the drag is over: the next one measures its own grab
            self.grab = None;
        }
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let hovered = ctx.interaction.hovered(self.id);
        let s = VisualState {
            hovered,
            pressed,
            ..Default::default()
        };
        let style = t.interactive(Kind::Seam, s, t.plane_of(bg)).style.bg(bg);
        let glyph = match (self.dir, hovered || pressed) {
            (SplitDir::Horizontal, false) => t.glyphs.divider,
            (SplitDir::Horizontal, true) => t.glyphs.scroll_thumb,
            (SplitDir::Vertical, false) => t.glyphs.h_divider,
            (SplitDir::Vertical, true) => t.glyphs.scroll_h_thumb,
        };
        for pos in area.positions() {
            buf.set_string(pos.x, pos.y, glyph, style);
        }
        // the padding only takes cells no control of the panes claims, so
        // it never steals a scrollbar or a text row next to the seam
        let hit = self.hit_area(*buf.area());
        ctx.clickable_under(self.id, hit);
        ctx.clickable(self.id, area);
        ctx.pointer_shape(
            self.id,
            match self.dir {
                SplitDir::Horizontal => crate::core::shape::PointerShape::ColResize,
                SplitDir::Vertical => crate::core::shape::PointerShape::RowResize,
            },
        );
    }

    fn grab_at(&self, split: &Split, container: Rect, gap: u16, pos: Position) -> i32 {
        let handle = split.handle(self.dir, container, gap);
        let (at, span) = match self.dir {
            SplitDir::Horizontal => (pos.x as i32 - handle.x as i32, handle.width),
            SplitDir::Vertical => (pos.y as i32 - handle.y as i32, handle.height),
        };
        let pad = self.hit_pad as i32;
        at.clamp(-pad, (span as i32 - 1).max(0) + pad)
    }

    /// A press on the handle: remember where on the seam it landed, so
    /// the drag that follows keeps that offset.
    pub fn on_press(&mut self, split: &Split, container: Rect, gap: u16, pos: Position) {
        self.grab = Some(self.grab_at(split, container, gap, pos));
    }

    /// A drag with this handle pressed: move the seam with the pointer,
    /// cell by cell, keeping the offset the press grabbed it at (measured
    /// on the first drag when the owner did not report the press).
    pub fn on_drag(
        &mut self,
        split: &mut Split,
        container: Rect,
        gap: u16,
        pos: Position,
    ) -> Outcome {
        let grab = match self.grab {
            Some(g) => g,
            None => {
                let g = self.grab_at(split, container, gap, pos);
                self.grab = Some(g);
                g
            }
        };
        let shift = |v: u16| (v as i32 - grab).max(0) as u16;
        let target = match self.dir {
            SplitDir::Horizontal => Position::new(shift(pos.x), pos.y),
            SplitDir::Vertical => Position::new(pos.x, shift(pos.y)),
        };
        if split.drag_to(self.dir, container, gap, target) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// A completed click on the handle with its multi-click `count`: a
    /// double-click applies [`Self::double_click`].
    pub fn on_click(&mut self, split: &mut Split, count: u8) -> Outcome {
        self.grab = None;
        if count != 2 {
            return Outcome::Consumed;
        }
        match self.double_click {
            SeamDoubleClick::Reset => {
                let before = *split;
                split.reset();
                if *split == before {
                    Outcome::Consumed
                } else {
                    Outcome::Changed
                }
            }
            SeamDoubleClick::Ignore => Outcome::Consumed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{focus::FocusRing, hit::HitRegistry};
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;

    #[test]
    fn seam_rest_hover_and_drag_differ_at_every_level() {
        let id = WidgetId::of("seam");
        for level in ColorLevel::ALL {
            let t = Theme::for_level(level);
            let draw = |interaction: Interaction| {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
                let mut buf = Buffer::empty(Rect::new(0, 0, 1, 3));
                Splitter::new(id, SplitDir::Horizontal).render(
                    Rect::new(0, 0, 1, 3),
                    &mut buf,
                    &mut ctx,
                    t.canvas,
                );
                let c = buf[(0, 1)].clone();
                let ink = if level == ColorLevel::Mono {
                    None
                } else {
                    Some(c.fg)
                };
                (c.symbol().to_owned(), ink, c.modifier, c.fg != c.bg)
            };
            let rest = draw(Interaction::default());
            let hover = draw(Interaction {
                hover: Some(id),
                ..Default::default()
            });
            let drag = draw(Interaction {
                hover: Some(id),
                pressed: Some(id),
                ..Default::default()
            });
            assert!(rest.3, "{level:?}: the rest seam is visible");
            assert_ne!(rest, hover, "{level:?}");
            assert_ne!(hover, drag, "{level:?}");
            assert_ne!(rest, drag, "{level:?}");
        }
    }

    fn render_seam(seam: &mut Splitter, split: &Split, container: Rect) -> HitRegistry {
        let t = Theme::for_level(ColorLevel::TrueColor);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(container);
        let handle = split.handle(seam.dir, container, 1);
        seam.render(handle, &mut buf, &mut ctx, t.canvas);
        hits
    }

    /// The press target is wider than the 1-cell rule (it was the rule
    /// alone).
    #[test]
    fn the_seam_takes_presses_one_cell_either_side() {
        let id = WidgetId::of("seam");
        let split = Split::new(50, 4, 4);
        let container = Rect::new(0, 0, 41, 5);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        let hits = render_seam(&mut seam, &split, container);
        let x = split.handle(SplitDir::Horizontal, container, 1).x;
        for dx in [x - 1, x, x + 1] {
            assert_eq!(hits.hit(Position::new(dx, 2)), Some(id), "x={dx}");
        }
        assert_eq!(hits.hit(Position::new(x + 2, 2)), None);
        // a pane control next to the seam keeps its cells, whenever it
        // registers
        let mut hits = render_seam(&mut seam, &split, container);
        let bar = WidgetId::of("bar");
        hits.register(bar, Rect::new(x - 1, 0, 1, 5));
        assert_eq!(hits.hit(Position::new(x - 1, 2)), Some(bar));
        assert_eq!(hits.hit(Position::new(x + 1, 2)), Some(id));
        let mut tight = Splitter::new(id, SplitDir::Horizontal).hit_pad(0);
        let hits = render_seam(&mut tight, &split, container);
        assert_eq!(hits.hit(Position::new(x - 1, 2)), None, "hit_pad(0)");
    }

    /// A drag moves the seam by exactly the cells the pointer moved, from
    /// wherever on the padded seam it was grabbed (it snapped the seam to
    /// the pointer and to whole-percent steps).
    #[test]
    fn a_drag_keeps_the_grab_offset_and_moves_cell_by_cell() {
        let id = WidgetId::of("seam");
        let mut split = Split::new(50, 4, 4);
        // 61 usable cells: whole percents cannot place every cell
        let container = Rect::new(0, 0, 62, 5);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        render_seam(&mut seam, &split, container);
        let x0 = split.handle(SplitDir::Horizontal, container, 1).x;
        // grab one cell right of the rule
        seam.on_press(&split, container, 1, Position::new(x0 + 1, 2));
        for step in 1..=7u16 {
            seam.on_drag(&mut split, container, 1, Position::new(x0 + 1 + step, 2));
            let x = split.handle(SplitDir::Horizontal, container, 1).x;
            assert_eq!(x, x0 + step, "step {step}");
        }
        // without a reported press the first drag measures the grab
        let mut split = Split::new(50, 4, 4);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        render_seam(&mut seam, &split, container);
        seam.on_drag(&mut split, container, 1, Position::new(x0 - 1, 2));
        assert_eq!(split.handle(SplitDir::Horizontal, container, 1).x, x0);
        seam.on_drag(&mut split, container, 1, Position::new(x0 - 4, 2));
        assert_eq!(split.handle(SplitDir::Horizontal, container, 1).x, x0 - 3);
    }

    /// A double-click restores the starting ratio; a policy turns it off.
    #[test]
    fn a_double_click_resets_the_split() {
        let id = WidgetId::of("seam");
        let container = Rect::new(0, 0, 62, 5);
        let mut split = Split::new(50, 4, 4);
        let mut seam = Splitter::new(id, SplitDir::Horizontal);
        render_seam(&mut seam, &split, container);
        seam.on_drag(&mut split, container, 1, Position::new(50, 2));
        let moved = split;
        assert_eq!(seam.on_click(&mut split, 1), Outcome::Consumed);
        assert_eq!(split, moved, "a single click changes nothing");
        assert_eq!(seam.on_click(&mut split, 2), Outcome::Changed);
        assert_eq!(split, Split::new(50, 4, 4));
        let mut split = moved;
        let mut off = Splitter::new(id, SplitDir::Horizontal).double_click(SeamDoubleClick::Ignore);
        off.on_click(&mut split, 2);
        assert_eq!(split, moved);
    }
}
