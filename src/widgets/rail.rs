//! A collapsible sidebar's controls: the rail a collapsed sidebar leaves
//! behind, and the collapse button on the open sidebar's frame.
//!
//! A sidebar hidden by a shortcut (Ctrl+B) must come back by pointer too:
//! macOS sidebars keep a toolbar toggle, and a collapsed pane in an IDE
//! keeps a rail with its name. The rail is a narrow column at the edge the
//! sidebar collapsed into: `›` above the sidebar's name written down the
//! column; a click anywhere on it opens the sidebar again.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::RenderCtx;

/// What a click on the sidebar's controls asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailAction {
    /// The rail was clicked: open the sidebar.
    Expand,
    /// The collapse button was clicked: collapse the sidebar.
    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarRail {
    /// The rail's hit id; the collapse button is `id.sub("collapse")`.
    pub id: WidgetId,
    /// The sidebar's name, written down the rail.
    pub label: String,
    /// A collapsed sidebar leaves a rail to click. Default on; off, it
    /// disappears entirely (only the keyboard brings it back).
    pub collapsed_rail: bool,
}

impl SidebarRail {
    /// Cells the rail takes: the glyph column and a gap.
    pub const WIDTH: u16 = 2;

    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            collapsed_rail: true,
        }
    }

    /// See [`Self::collapsed_rail`].
    pub fn collapsed_rail(mut self, on: bool) -> Self {
        self.collapsed_rail = on;
        self
    }

    /// The collapse button's hit id.
    pub fn collapse_id(&self) -> WidgetId {
        self.id.sub("collapse")
    }

    /// Columns the collapsed sidebar keeps: [`Self::WIDTH`], or 0 without
    /// a rail.
    pub fn width(&self) -> u16 {
        if self.collapsed_rail { Self::WIDTH } else { 0 }
    }

    /// A completed click on `id`.
    pub fn on_click(&self, id: WidgetId) -> Option<RailAction> {
        if id == self.id && self.collapsed_rail {
            Some(RailAction::Expand)
        } else if id == self.collapse_id() {
            Some(RailAction::Collapse)
        } else {
            None
        }
    }

    /// Draw the rail in the first column of `area` (the column the sidebar
    /// collapsed into) and register it. Rest, hover and press come from
    /// the title ladder, so it reads as a control.
    pub fn render_rail(&self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if !self.collapsed_rail || area.is_empty() {
            return;
        }
        let t = ctx.theme;
        let col = Rect::new(area.x, area.y, 1, area.height);
        let st = t
            .interactive(Kind::Title, ctx.state(self.id), t.plane_of(bg))
            .style;
        crate::ui::ctx::fill(buf, col, st);
        buf.set_string(col.x, col.y, t.glyphs.overflow_right, st);
        for (i, ch) in self.label.chars().enumerate() {
            let y = col.y + 2 + i as u16;
            if y >= col.bottom() {
                break;
            }
            buf.set_string(col.x, y, ch.to_string(), st);
        }
        ctx.clickable(self.id, col);
    }

    /// Draw the collapse button (` ‹ `) into the top border of the open
    /// sidebar's frame `panel`, right-aligned, and register it.
    pub fn render_collapse(&self, panel: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        if panel.width < 12 || panel.height < 2 {
            return;
        }
        let t = ctx.theme;
        let id = self.collapse_id();
        let r = Rect::new(panel.right() - 5, panel.y, 3, 1);
        let host = t.border(false).bg(bg);
        let st = t
            .interactive(Kind::Title, ctx.state(id), t.plane_of(bg))
            .style;
        buf.set_string(r.x, r.y, " ", host);
        buf.set_string(r.x + 1, r.y, t.glyphs.overflow_left, st);
        buf.set_string(r.x + 2, r.y, " ", host);
        ctx.clickable(id, r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    #[test]
    fn the_rail_and_the_collapse_button_take_clicks_and_hover() {
        let theme = Theme::junie();
        let rail = SidebarRail::new(WidgetId::of("side"), "Explorer");
        let area = Rect::new(0, 0, 20, 12);
        let draw = |hover: Option<WidgetId>| {
            let mut buf = Buffer::empty(area);
            let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
            let inter = Interaction {
                hover,
                ..Interaction::default()
            };
            {
                let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
                rail.render_rail(area, &mut buf, &mut ctx, theme.canvas);
                rail.render_collapse(Rect::new(4, 0, 16, 6), &mut buf, &mut ctx, theme.canvas);
            }
            (buf, hits)
        };
        let (rest, hits) = draw(None);
        assert_eq!(rest[(0, 0)].symbol(), "›");
        assert_eq!(rest[(0, 2)].symbol(), "E");
        assert_eq!(hits.hit(Position::new(0, 5)), Some(rail.id));
        assert_eq!(rail.on_click(rail.id), Some(RailAction::Expand));
        let c = hits.area_of(rail.collapse_id()).expect("collapse button");
        assert_eq!(rest[(c.x + 1, 0)].symbol(), "‹");
        assert_eq!(
            rail.on_click(rail.collapse_id()),
            Some(RailAction::Collapse)
        );
        let (hover, _) = draw(Some(rail.id));
        assert_ne!(rest[(0, 0)].style(), hover[(0, 0)].style());
        let off = rail.clone().collapsed_rail(false);
        assert_eq!(off.width(), 0);
        assert_eq!(off.on_click(off.id), None);
    }
}
