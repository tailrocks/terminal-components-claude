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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn draw(b: &mut Breadcrumb, w: u16, hover: Option<WidgetId>) -> (Buffer, HitRegistry) {
        let theme = Theme::junie();
        let area = Rect::new(0, 0, w, 1);
        let mut buf = Buffer::empty(area);
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let inter = Interaction {
            hover,
            ..Interaction::default()
        };
        {
            let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
            b.render(area, &mut buf, &mut ctx, theme.canvas);
        }
        (buf, hits)
    }

    fn text(buf: &Buffer) -> String {
        buf.content().iter().map(|c| c.symbol()).collect()
    }

    #[test]
    fn measure_is_the_width_that_shows_the_whole_trail() {
        use crate::ui::measure::Measure;
        let theme = Theme::junie();
        let (mut hits, mut ring) = (HitRegistry::default(), FocusRing::default());
        let ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let b = trail();
        let size = b.measure(ratatui::layout::Size::new(5, 5), &ctx);
        assert_eq!(size.height, 1);
        let mut whole = trail();
        let (buf, _) = draw(&mut whole, size.width, None);
        let shown = text(&buf);
        assert!(
            shown.contains("Finder") && shown.contains("Top files"),
            "{shown}"
        );
        assert!(!shown.contains('…'), "{shown}");

        let (buf, _) = draw(&mut trail(), size.width - 1, None);
        assert!(text(&buf).contains('…'));
    }

    fn trail() -> Breadcrumb {
        Breadcrumb::new(
            WidgetId::of("crumbs"),
            vec!["Finder".into(), "Disk › Usage".into(), "Top files".into()],
        )
    }

    #[test]
    fn every_earlier_level_is_a_link_and_the_current_page_is_not() {
        let mut b = trail();
        let (buf, hits) = draw(&mut b, 60, None);
        assert!(text(&buf).starts_with("Finder › Disk › Usage › Top files"));
        let a0 = b.areas[0];
        assert_eq!(
            hits.hit(ratatui::layout::Position::new(a0.x, 0)),
            Some(b.segment_id(0))
        );
        assert_eq!(b.on_click(b.segment_id(1)), Some(1));
        let last = b.areas[2];
        assert_eq!(hits.hit(ratatui::layout::Position::new(last.x, 0)), None);
        assert_eq!(b.on_click(b.segment_id(2)), None);
    }

    #[test]
    fn a_link_hovers_and_the_trail_can_be_plain_text() {
        let mut b = trail();
        let (rest, _) = draw(&mut b, 60, None);
        let id = b.segment_id(0);
        let (hover, _) = draw(&mut b, 60, Some(id));
        assert_ne!(rest[(0, 0)].style(), hover[(0, 0)].style());
        let mut plain = trail().clickable(false);
        let (_, hits) = draw(&mut plain, 60, None);
        assert_eq!(hits.hit(ratatui::layout::Position::new(0, 0)), None);
        assert_eq!(plain.on_click(plain.segment_id(0)), None);
    }

    #[test]
    fn a_narrow_trail_collapses_the_outer_levels() {
        let mut b = trail();
        let (buf, _) = draw(&mut b, 28, None);
        let s = text(&buf);
        assert!(s.starts_with("… › "), "{s}");
        assert!(s.contains("Top files"), "{s}");
        assert_eq!(b.areas[0], Rect::ZERO);
    }

    #[test]
    fn a_current_page_too_wide_is_cut_in_the_middle() {
        let mut b = Breadcrumb::new(
            WidgetId::of("crumbs"),
            vec!["settlement-backoff-retry-policy-v2.md".into()],
        );
        let (buf, _) = draw(&mut b, 20, None);
        let s = text(&buf);
        assert!(s.starts_with("settlement"), "{s}");
        assert!(s.trim_end().ends_with("v2.md"), "{s}");
        assert!(s.contains('…'), "{s}");
    }
}
