use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::core::id::{Part, PartRef, WidgetId};
use crate::theme::Kind;
use crate::ui::ctx::RenderCtx;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailAction {
    Expand,

    Collapse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidebarRail {
    pub id: WidgetId,

    pub label: String,

    pub collapsed_rail: bool,
}

impl SidebarRail {
    pub const WIDTH: u16 = 2;

    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            collapsed_rail: true,
        }
    }

    pub fn collapsed_rail(mut self, on: bool) -> Self {
        self.collapsed_rail = on;
        self
    }

    pub fn collapse_id(&self) -> WidgetId {
        self.id.part(PartRef::of(Part::ICON))
    }

    pub fn width(&self) -> u16 {
        if self.collapsed_rail { Self::WIDTH } else { 0 }
    }

    pub fn on_click(&self, id: WidgetId) -> Option<RailAction> {
        if id == self.id && self.collapsed_rail {
            Some(RailAction::Expand)
        } else if id == self.collapse_id() {
            Some(RailAction::Collapse)
        } else {
            None
        }
    }

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
        ctx.register_part(self.id, PartRef::of(Part::ICON), r);
    }
}

impl crate::ui::specimen::Specimen for SidebarRail {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        self.collapsed_rail && matches!(state, Rest | Hover | Pressed)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        2 + self.label.chars().count() as u16
    }

    fn specimen(
        &self,
        _state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        crate::ui::ctx::fill(buf, area, ratatui::style::Style::new().bg(bg));
        self.render_rail(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for SidebarRail {
    type Action = RailAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<RailAction> {
        use crate::core::response::Response;
        if !ev.is(crate::core::intent::Phase::Click) {
            return Response::ignored();
        }
        match self.on_click(ev.id) {
            Some(a) => Response::emit(a),
            None => Response::ignored(),
        }
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
