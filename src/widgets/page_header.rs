//! A page's header and a section heading.
//!
//! ```text
//!  Cleanup review · 3 selected                     here · ~/work · mbp   (title · status · right)
//!  4 categories · 12.4 GB · Trash · dry run                              (subtitle)
//!
//!    Recommended                                                 3 items (heading · meta)
//! ```
//!
//! [`PageHeader`] is the title row of a screen: the title (bold), a status
//! run right after it in its own tone, a right-aligned meta shown only when
//! it fits beside them, and an optional subtitle row under it. Pages lay
//! their body out under [`PageHeader::height`] plus one blank row
//! ([`PageHeader::body`]).
//!
//! [`Heading`] names a group of rows inside a list or card: quiet text with
//! an optional meta at the right end.
//!
//! Both only draw: they register no targets and take no keys. Their
//! styles set the ink only, so they read on whatever plane they are drawn
//! on.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

use crate::theme::{Theme, Tone};
use crate::ui::text::{truncate, width};

/// A page's title row and optional subtitle; see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHeader {
    pub title: String,
    /// Text right after the title (` · 3 selected`), in its tone. Cut to
    /// the room the row leaves.
    pub status: Option<(String, Tone)>,
    /// A right-aligned meta in the muted tone, drawn only when it fits
    /// beside the title and status with [`Self::right_gap`] cells to
    /// spare.
    pub right: Option<String>,
    /// The spare cells the right meta needs. Default 4.
    pub right_gap: u16,
    /// A second row under the title, in its tone, cut to the width.
    pub subtitle: Option<(String, Tone)>,
    /// Cells left clear at both ends. Default 1.
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

    /// See [`Self::status`].
    pub fn status(mut self, text: &str, tone: Tone) -> Self {
        self.status = Some((text.to_owned(), tone));
        self
    }

    /// See [`Self::right`].
    pub fn right(mut self, text: &str) -> Self {
        self.right = Some(text.to_owned());
        self
    }

    /// See [`Self::right_gap`].
    pub fn right_gap(mut self, cells: u16) -> Self {
        self.right_gap = cells;
        self
    }

    /// See [`Self::subtitle`].
    pub fn subtitle(mut self, text: &str, tone: Tone) -> Self {
        self.subtitle = Some((text.to_owned(), tone));
        self
    }

    /// See [`Self::inset`].
    pub fn inset(mut self, cells: u16) -> Self {
        self.inset = cells;
        self
    }

    /// Rows the header draws: the title row, and the subtitle's.
    pub fn height(&self) -> u16 {
        1 + u16::from(self.subtitle.is_some())
    }

    /// The body under the header in `area`: one blank row after the
    /// header's own rows.
    pub fn body(&self, area: Rect) -> Rect {
        let off = (self.height() + 1).min(area.height);
        Rect::new(area.x, area.y + off, area.width, area.height - off)
    }

    /// Draw the header at the top of `area`; returns the rows drawn.
    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme) -> u16 {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return 0;
        }
        let x = area.x + self.inset.min(area.width);
        let title_w = width(&self.title) as u16;
        // the title is clipped at the area's edge, never cut with `…`
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
                let s = truncate(status, left as usize);
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
        buf.set_string(
            x,
            area.y + 1,
            truncate(sub, area.width.saturating_sub(2 * self.inset) as usize),
            ink(t, *tone),
        );
        2
    }
}

/// A group heading inside a list or card; see the module docs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub text: String,
    /// A short meta at the right end (`3 items`), faint, drawn only when
    /// it fits beside the text with two cells to spare.
    pub meta: Option<String>,
    /// The heading's ink. Default [`Tone::Muted`].
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

    /// See [`Self::meta`].
    pub fn meta(mut self, meta: &str) -> Self {
        self.meta = Some(meta.to_owned());
        self
    }

    /// See [`Self::tone`].
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Draw the heading on the first row of `area`, on the plane `bg`.
    pub fn render(&self, area: Rect, buf: &mut Buffer, t: &Theme, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        let ink = Style::new().fg(t.tone_on(self.tone, bg)).bg(bg);
        buf.set_string(
            area.x,
            area.y,
            truncate(&self.text, area.width as usize),
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

/// Ink-only style in `tone`: the header keeps the plane it is drawn on.
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
        // too narrow for the meta: it is left out, the title stays
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
