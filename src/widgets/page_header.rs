use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::theme::{Theme, Tone};
use crate::ui::text::width;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHeader {
    pub title: String,

    pub status: Option<(String, Tone)>,

    pub right: Option<String>,

    pub right_gap: u16,

    pub subtitle: Option<(String, Tone)>,

    pub inset: u16,
}

impl PageHeader {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_owned(),
            status: None,
            right: None,
            right_gap: 4,
            subtitle: None,
            inset: 1,
        }
    }

    pub fn status(mut self, text: &str, tone: Tone) -> Self {
        self.status = Some((text.to_owned(), tone));
        self
    }

    pub fn right(mut self, text: &str) -> Self {
        self.right = Some(text.to_owned());
        self
    }

    pub fn right_gap(mut self, cells: u16) -> Self {
        self.right_gap = cells;
        self
    }

    pub fn subtitle(mut self, text: &str, tone: Tone) -> Self {
        self.subtitle = Some((text.to_owned(), tone));
        self
    }

    pub fn inset(mut self, cells: u16) -> Self {
        self.inset = cells;
        self
    }

    pub fn height(&self) -> u16 {
        1 + u16::from(self.subtitle.is_some())
    }

    pub fn body(&self, area: Rect) -> Rect {
        let off = (self.height() + 1).min(area.height);
        Rect::new(area.x, area.y + off, area.width, area.height - off)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme) -> u16 {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return 0;
        }
        let x = area.x + self.inset.min(area.width);
        let title_w = width(&self.title) as u16;

        buf.set_stringn(
            x,
            area.y,
            &self.title,
            area.right().saturating_sub(x) as usize,
            t.title(),
        );
        let mut used = title_w;
        if let Some((status, tone)) = &self.status {
            let left = area.width.saturating_sub(title_w + 2 * self.inset);
            if left > 0 {
                let s = crate::ui::reveal::cut(Rect::new(x + title_w, area.y, left, 1), status);
                used += width(&s) as u16;
                buf.set_string(x + title_w, area.y, s, ink(t, *tone));
            }
        }
        if let Some(right) = &self.right {
            let rw = width(right) as u16;
            if rw + used + self.right_gap < area.width {
                buf.set_string(
                    area.right().saturating_sub(rw + self.inset),
                    area.y,
                    right,
                    t.muted(),
                );
            }
        }
        let Some((sub, tone)) = &self.subtitle else {
            return 1;
        };
        if area.height < 2 {
            return 1;
        }
        let row = Rect::new(x, area.y + 1, area.width.saturating_sub(2 * self.inset), 1);
        buf.set_string(x, row.y, crate::ui::reveal::cut(row, sub), ink(t, *tone));
        2
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub text: String,

    pub meta: Option<String>,

    pub tone: Tone,
}

impl Heading {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            meta: None,
            tone: Tone::Muted,
        }
    }

    pub fn meta(mut self, meta: &str) -> Self {
        self.meta = Some(meta.to_owned());
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let ink = Style::new().fg(t.tone_on(self.tone, bg)).bg(bg);
        buf.set_string(
            area.x,
            area.y,
            crate::ui::reveal::cut(Rect::new(area.x, area.y, area.width, 1), &self.text),
            ink,
        );
        if let Some(meta) = &self.meta {
            let mw = width(meta) as u16;
            if width(&self.text) as u16 + 2 + mw <= area.width {
                buf.set_string(
                    area.right() - mw,
                    area.y,
                    meta,
                    Style::new().fg(t.tone_on(Tone::Faint, bg)).bg(bg),
                );
            }
        }
    }
}

impl crate::ui::measure::Measure for PageHeader {
    fn measure(
        &self,
        _avail: ratatui::layout::Size,
        _ctx: &crate::ui::ctx::RenderCtx,
    ) -> ratatui::layout::Size {
        let w = |s: &str| width(s) as u16;
        let used = w(&self.title) + self.status.as_ref().map_or(0, |(s, _)| w(s));
        let row = match &self.right {
            Some(r) => (used + self.right_gap + w(r) + 1).max(used + 2 * self.inset),
            None => used + 2 * self.inset,
        };
        let sub = self
            .subtitle
            .as_ref()
            .map_or(0, |(s, _)| w(s) + 2 * self.inset);
        ratatui::layout::Size::new(row.max(sub), self.height())
    }
}

impl crate::ui::measure::Measure for Heading {
    fn measure(
        &self,
        _avail: ratatui::layout::Size,
        _ctx: &crate::ui::ctx::RenderCtx,
    ) -> ratatui::layout::Size {
        let meta = self.meta.as_deref().map_or(0, |m| 2 + width(m) as u16);
        ratatui::layout::Size::new(width(&self.text) as u16 + meta, 1)
    }
}

fn ink(t: &Theme, tone: Tone) -> Style {
    Style::new().fg(t.tone(tone))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(buf: &Buffer, y: u16) -> String {
        (0..buf.area.width)
            .map(|x| buf[(x, y)].symbol().to_owned())
            .collect()
    }

    #[test]
    fn a_cut_heading_and_subtitle_reveal_on_hover() {
        let t = Theme::junie();
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let mut buf = Buffer::empty(Rect::new(0, 0, 20, 3));
        let mut ctx = crate::ui::ctx::RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        PageHeader::new("Usage")
            .subtitle("Identities sorted by their worst window first", Tone::Muted)
            .render(Rect::new(0, 0, 20, 2), &mut buf, &t);
        Heading::new("Recoverable toggles until emptied").render(
            Rect::new(0, 2, 20, 1),
            &mut buf,
            &t,
            t.canvas,
        );
        ctx.flush_overlays(&mut buf);
        drop(ctx);
        for y in [1, 2] {
            assert!(row(&buf, y).contains('…'), "{:?}", row(&buf, y));
            assert!(
                hits.hover_at(ratatui::layout::Position::new(4, y))
                    .is_some(),
                "row {y} hovers"
            );
        }
    }

    #[test]
    fn a_header_and_a_heading_at_their_measure_show_everything() {
        use crate::ui::measure::Measure;
        let t = Theme::junie();
        let mut hits = crate::core::hit::HitRegistry::default();
        let mut ring = crate::core::focus::FocusRing::default();
        let ctx = crate::ui::ctx::RenderCtx::new(
            &t,
            crate::ui::ctx::Interaction::default(),
            &mut hits,
            &mut ring,
        );
        let h = PageHeader::new("Usage and capacity")
            .status(" · offline · last good 4 min ago", Tone::Warning)
            .right("refreshes every 2 s")
            .right_gap(4)
            .subtitle("Plan quotas, filesystems and host load", Tone::Muted);
        let size = h.measure(ratatui::layout::Size::ZERO, &ctx);
        let mut buf = Buffer::empty(Rect::new(0, 0, size.width, size.height));
        assert_eq!(h.render(buf.area, &mut buf, &t), size.height);
        let top = row(&buf, 0);
        for part in [
            "Usage and capacity",
            "last good 4 min ago",
            "refreshes every 2 s",
        ] {
            assert!(top.contains(part), "{part:?} in {top:?}");
        }
        assert!(row(&buf, 1).contains("host load"));
        let g = Heading::new("Sections").meta("3 cards");
        let size = g.measure(ratatui::layout::Size::ZERO, &ctx);
        let mut buf = Buffer::empty(Rect::new(0, 0, size.width, 1));
        g.render(buf.area, &mut buf, &t, t.canvas);
        assert_eq!(row(&buf, 0), "Sections  3 cards");
    }

    #[test]
    fn the_title_status_right_meta_and_subtitle_take_their_places() {
        let t = Theme::junie();
        let area = Rect::new(0, 0, 60, 4);
        let mut buf = Buffer::empty(area);
        let h = PageHeader::new("Cleanup review")
            .status(" · 3 selected", Tone::Warning)
            .right("here · mbp")
            .subtitle("4 categories · 12.4 GB", Tone::Muted);
        assert_eq!(h.render(area, &mut buf, &t), 2);
        assert_eq!(
            row(&buf, 0).trim_end(),
            " Cleanup review · 3 selected                     here · mbp"
        );
        assert_eq!(buf[(1, 0)].fg, t.text_primary);
        assert_eq!(buf[(15, 0)].fg, t.warning, "the status in its tone");
        assert_eq!(buf[(50, 0)].fg, t.text_muted);
        assert_eq!(row(&buf, 1).trim_end(), " 4 categories · 12.4 GB");
        assert_eq!(h.body(area), Rect::new(0, 3, 60, 1));

        let narrow = Rect::new(0, 0, 30, 1);
        let mut buf = Buffer::empty(narrow);
        assert_eq!(
            h.clone()
                .subtitle("", Tone::Muted)
                .render(narrow, &mut buf, &t),
            1
        );
        assert!(!row(&buf, 0).contains("mbp"));
        assert!(row(&buf, 0).starts_with(" Cleanup review · 3 selected"));
    }

    #[test]
    fn a_heading_is_quiet_with_its_meta_at_the_end() {
        let t = Theme::junie();
        let area = Rect::new(0, 0, 30, 1);
        let mut buf = Buffer::empty(area);
        Heading::new("Recommended")
            .meta("3 items")
            .render(area, &mut buf, &t, t.canvas);
        assert_eq!(row(&buf, 0), "Recommended            3 items");
        assert_eq!(buf[(0, 0)].fg, t.tone_on(Tone::Muted, t.canvas));
        let mut buf = Buffer::empty(Rect::new(0, 0, 14, 1));
        Heading::new("Recommended").meta("3 items").render(
            Rect::new(0, 0, 14, 1),
            &mut buf,
            &t,
            t.canvas,
        );
        assert_eq!(row(&buf, 0), "Recommended   ", "no room: no meta");
    }
}
