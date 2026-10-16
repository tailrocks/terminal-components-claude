//! Split handle: the gap strip between two panes of a [`Split`]. Mouse-only
//! affordance (keyboard resize stays a chord on the owning pane). Rest,
//! hover and pressed walk the theme's seam ladder ([`Kind::Seam`]): a quiet
//! rule at rest, a brighter heavy rule under the pointer, the brightest
//! heavy rule while dragged, so the affordance survives monochrome
//! terminals.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::{RenderCtx, VisualState};
use crate::ui::layout::{Split, SplitDir};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Splitter {
    pub id: WidgetId,
    pub dir: SplitDir,
    pub area: Rect,
}

impl Splitter {
    pub const fn new(id: WidgetId, dir: SplitDir) -> Self {
        Self {
            id,
            dir,
            area: Rect::ZERO,
        }
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
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let hovered = ctx.interaction.hovered(self.id);
        // a drag keeps the seam pressed while the pointer runs ahead of it
        let pressed = held || ctx.interaction.pressed == Some(self.id);
        let s = VisualState {
            hovered,
            pressed,
            ..Default::default()
        };
        let style = t.interactive(Kind::Seam, s, t.plane_of(bg)).style.bg(bg);
        let glyph = match (self.dir, hovered || pressed) {
            (SplitDir::Horizontal, false) => t.glyphs.scroll_track,
            (SplitDir::Horizontal, true) => t.glyphs.scroll_thumb,
            (SplitDir::Vertical, false) => t.glyphs.scroll_h_track,
            (SplitDir::Vertical, true) => t.glyphs.scroll_h_thumb,
        };
        for pos in area.positions() {
            buf.set_string(pos.x, pos.y, glyph, style);
        }
        ctx.clickable(self.id, area);
    }

    /// A drag with this handle pressed: move the seam under the pointer.
    pub fn on_drag(&self, split: &mut Split, container: Rect, gap: u16, pos: Position) -> Outcome {
        if split.drag_to(self.dir, container, gap, pos) {
            Outcome::Changed
        } else {
            Outcome::Consumed
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
}
