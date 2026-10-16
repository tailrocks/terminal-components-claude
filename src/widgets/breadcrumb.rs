//! A breadcrumb trail (`Disk › Usage › Top files`): where a page sits in
//! a stack of pages, with every earlier level clickable to go back to it
//! (the macOS Finder path bar and Xcode jump bar; WAI-ARIA breadcrumb
//! pattern: links to the ancestors, the current page last and not a link).
//!
//! The trail is pointer navigation for a stack the keyboard already walks
//! with Esc; it takes no Tab stop of its own.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::id::WidgetId;
use crate::theme::Kind;
use crate::ui::ctx::RenderCtx;
use crate::ui::text::{truncate, width};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Breadcrumb {
    pub id: WidgetId,
    /// One label per level, outermost first; the last is the current page.
    pub segments: Vec<String>,
    /// The earlier levels are clickable and go back to that level. Default
    /// on; off draws the same trail as plain text.
    pub clickable: bool,
    /// Where each segment was drawn by the last render (`Rect::ZERO` for a
    /// level collapsed into the leading `…`).
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

    /// See [`Self::clickable`].
    pub fn clickable(mut self, on: bool) -> Self {
        self.clickable = on;
        self
    }

    /// The hit id of level `i`.
    pub fn segment_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    /// A completed click: the level it goes back to (0 is the outermost),
    /// or `None` when `id` is not one of the trail's links.
    pub fn on_click(&self, id: WidgetId) -> Option<usize> {
        if !self.clickable {
            return None;
        }
        let last = self.segments.len().checked_sub(1)?;
        (0..last).find(|&i| self.segment_id(i) == id)
    }

    /// Draw the trail on one row of `area`, left-aligned on `bg`. When it
    /// is too wide, the outermost levels collapse into a leading `…` and
    /// the current page keeps the room it needs.
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
        // the first level shown: drop outer levels until the trail fits
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
        let put = |buf: &mut Buffer, x: &mut u16, s: &str, st: ratatui::style::Style| -> Rect {
            if *x >= right {
                return Rect::ZERO;
            }
            let s = truncate(s, (right - *x) as usize);
            let w = width(&s) as u16;
            buf.set_string(*x, y, &s, st);
            let r = Rect::new(*x, y, w, 1);
            *x += w;
            r
        };
        if from > 0 {
            put(buf, &mut x, ell, t.faint().bg(bg));
            put(buf, &mut x, &sep, t.faint().bg(bg));
        }
        for i in from..n {
            let current = i + 1 == n;
            let label = self.segments[i].clone();
            if current {
                // the current page: neutral, never the focus accent
                let st = t.title().bg(bg);
                self.areas[i] = put(buf, &mut x, &label, st);
            } else if self.clickable {
                let id = self.segment_id(i);
                let st = t.interactive(Kind::Title, ctx.state(id), plane).style;
                let r = put(buf, &mut x, &label, st);
                if !r.is_empty() {
                    ctx.clickable(id, r);
                }
                self.areas[i] = r;
                put(buf, &mut x, &sep, t.faint().bg(bg));
            } else {
                let st = t.secondary().bg(bg).remove_modifier(Modifier::BOLD);
                self.areas[i] = put(buf, &mut x, &label, st);
                put(buf, &mut x, &sep, t.faint().bg(bg));
            }
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
}
