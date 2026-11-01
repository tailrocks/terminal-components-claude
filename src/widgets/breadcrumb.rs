use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::theme::Kind;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::{truncate, truncate_middle, width};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumb {
    pub id: WidgetId,

    pub segments: Vec<String>,

    pub clickable: bool,

    pub areas: Vec<Rect>,
}

impl Breadcrumb {
    pub fn new(id: WidgetId, segments: Vec<String>) -> Self {
        Self {
            id,
            segments,
            clickable: true,
            areas: vec![],
        }
    }

    pub fn clickable(mut self, on: bool) -> Self {
        self.clickable = on;
        self
    }

    pub fn segment_id(&self, i: usize) -> WidgetId {
        self.id.part(Self::segment_part(i))
    }

    fn segment_part(i: usize) -> PartRef {
        PartRef::item(Part::ROW, ItemKey::index(i))
    }

    pub fn on_click(&self, id: WidgetId) -> Option<usize> {
        if !self.clickable {
            return None;
        }
        let last = self.segments.len().checked_sub(1)?;
        self.id.item_of(id, Part::ROW, 0..last)
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        self.areas = vec![Rect::ZERO; self.segments.len()];
        if area.is_empty() || self.segments.is_empty() {
            return;
        }
        let t = ctx.theme;
        let sep = format!(" {} ", t.glyphs.overflow_right);
        let sep_w = width(&sep) as u16;
        let ell = t.glyphs.ellipsis;
        let n = self.segments.len();
        let seg_w = |s: &String| width(s) as u16;

        let total = |from: usize| -> u16 {
            let lead = if from > 0 {
                width(ell) as u16 + sep_w
            } else {
                0
            };
            lead + self.segments[from..].iter().map(seg_w).sum::<u16>()
                + sep_w * (n - from - 1) as u16
        };
        let mut from = 0;
        while from + 1 < n && total(from) > area.width {
            from += 1;
        }
        let plane = t.plane_of(bg);
        let mut x = area.x;
        let y = area.y;
        let right = area.right();
        let put = |buf: &mut Buffer,
                   x: &mut u16,
                   s: &str,
                   st: ratatui::style::Style,
                   middle: bool|
         -> Rect {
            if *x >= right {
                return Rect::ZERO;
            }
            let room = (right - *x) as usize;
            let s = if middle {
                truncate_middle(s, room)
            } else {
                truncate(s, room)
            };
            let w = width(&s) as u16;
            buf.set_string(*x, y, &s, st);
            let r = Rect::new(*x, y, w, 1);
            *x += w;
            r
        };
        if from > 0 {
            put(buf, &mut x, ell, t.faint().bg(bg), false);
            put(buf, &mut x, &sep, t.faint().bg(bg), false);
        }
        for i in from..n {
            let current = i + 1 == n;
            let label = self.segments[i].clone();
            if current {
                let st = t.title().bg(bg);
                self.areas[i] = put(buf, &mut x, &label, st, true);
            } else if self.clickable {
                let id = self.segment_id(i);
                let st = t.interactive(Kind::Title, ctx.state(id), plane).style;
                let r = put(buf, &mut x, &label, st, false);
                if !r.is_empty() {
                    ctx.register_part(self.id, Self::segment_part(i), r);
                }
                self.areas[i] = r;
                put(buf, &mut x, &sep, t.faint().bg(bg), false);
            } else {
                let st = t.secondary().bg(bg).remove_modifier(Modifier::BOLD);
                self.areas[i] = put(buf, &mut x, &label, st, false);
                put(buf, &mut x, &sep, t.faint().bg(bg), false);
            }
        }
    }
}

impl crate::ui::measure::Measure for Breadcrumb {
    fn measure(&self, _avail: ratatui::layout::Size, ctx: &RenderCtx) -> ratatui::layout::Size {
        if self.segments.is_empty() {
            return ratatui::layout::Size::new(0, 0);
        }
        let sep = width(&format!(" {} ", ctx.theme.glyphs.overflow_right)) as u16;
        let labels: u16 = self.segments.iter().map(|s| width(s) as u16).sum();
        let gaps = sep * (self.segments.len() - 1) as u16;
        ratatui::layout::Size::new(labels + gaps, 1)
    }
}

impl crate::ui::specimen::Specimen for Breadcrumb {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        match state {
            Rest | Overflow | Narrow => true,
            Hover | Pressed => self.clickable && self.segments.len() > 1,
            _ => false,
        }
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let mut b = self.clone();
        if state == Overflow {
            let at = b.segments.len().saturating_sub(1);
            for (k, s) in ["Volumes", "Archive", "2026", "October"].iter().enumerate() {
                b.segments.insert(at + k, (*s).to_owned());
            }
        }

        let first = b.segment_id(0);
        match state {
            Hover => ctx.interaction.hover = Some(first),
            Pressed => {
                ctx.interaction.hover = Some(first);
                ctx.interaction.pressed = Some(first);
            }
            _ => {}
        }
        crate::ui::ctx::fill(buf, area, ratatui::style::Style::new().bg(bg));
        b.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for Breadcrumb {
    type Action = usize;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<usize> {
        use crate::core::response::Response;
        if !ev.is(crate::core::intent::Phase::Click) {
            return Response::ignored();
        }
        match self.on_click(ev.id) {
            Some(level) => Response::emit(level),
            None => Response::ignored(),
        }
    }
}
