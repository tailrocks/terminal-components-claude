//! Split handle: the gap strip between two panes of a [`Split`]. Mouse-only
//! affordance (keyboard resize stays a chord on the owning pane): draws a
//! quiet rule in border-subtle, border-strong while hovered, and a heavy
//! rule while dragged so the affordance survives monochrome terminals.

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::ui::ctx::RenderCtx;
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
        let area = area.intersection(*buf.area());
        self.area = area;
        if area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let hovered = ctx.interaction.hovered(self.id);
        let pressed = ctx.interaction.pressed == Some(self.id);
        let style = t.border(hovered || pressed).bg(bg);
        let glyph = match (self.dir, pressed) {
            (SplitDir::Horizontal, false) => "│",
            (SplitDir::Horizontal, true) => "┃",
            (SplitDir::Vertical, false) => "─",
            (SplitDir::Vertical, true) => "━",
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
