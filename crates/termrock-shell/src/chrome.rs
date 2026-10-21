//! Chrome geometry: how the shell splits its frame into header, body and
//! footer regions.
//!
//! Ported from PR #26 `src/shell/chrome.rs` (commit `f4867e9b5`). The source's
//! `Menus<C>` command map and `Chrome::fit` dialog inset are not ported: the
//! parity menu bar resolves choices through [`termrock_core::action::ActionKey`]
//! itself, and the parity dialog owns its chrome through its own layout, so
//! both belong to their component crates rather than to the shell.

use ratatui_core::layout::Rect;

/// The three regions one chrome split produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Regions {
    /// The top strip.
    pub header: Rect,
    /// Everything between the header and the footer.
    pub body: Rect,
    /// The bottom strip.
    pub footer: Rect,
}

/// The chrome budget: rows reserved for the header and footer, the gap
/// between them and the body, and the columns of body inset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    /// Rows reserved for the header.
    pub header: u16,
    /// Blank rows between the strips and the body.
    pub gap: u16,
    /// Rows reserved for the footer.
    pub footer: u16,
    /// Columns inset from the body's left and right edges.
    pub inset: u16,
}

impl Default for Chrome {
    fn default() -> Self {
        Self::new()
    }
}

impl Chrome {
    /// One header row, one gap row, one footer row, no inset.
    pub const fn new() -> Self {
        Self {
            header: 1,
            gap: 1,
            footer: 1,
            inset: 0,
        }
    }

    /// Set the header rows.
    #[must_use]
    pub const fn header(mut self, rows: u16) -> Self {
        self.header = rows;
        self
    }

    /// Set the gap rows.
    #[must_use]
    pub const fn gap(mut self, rows: u16) -> Self {
        self.gap = rows;
        self
    }

    /// Set the footer rows.
    #[must_use]
    pub const fn footer(mut self, rows: u16) -> Self {
        self.footer = rows;
        self
    }

    /// Set the body inset columns.
    #[must_use]
    pub const fn inset(mut self, cols: u16) -> Self {
        self.inset = cols;
        self
    }

    /// Split `area` once into the three regions. The strips keep their rows
    /// while the area is tall enough; the body takes what is left and is
    /// empty when nothing is.
    pub fn regions(&self, area: Rect) -> Regions {
        let header_h = self.header.min(area.height);
        let footer_h = self.footer.min(area.height.saturating_sub(header_h));
        let header = Rect::new(area.x, area.y, area.width, header_h);
        let footer = Rect::new(
            area.x,
            area.bottom().saturating_sub(footer_h),
            area.width,
            footer_h,
        );
        let top = if header_h > 0 {
            header_h.saturating_add(self.gap)
        } else {
            0
        };
        let bottom = if footer_h > 0 {
            footer_h.saturating_add(self.gap)
        } else {
            0
        };
        let body_y = area.y.saturating_add(top).min(area.bottom());
        let body_h = area.height.saturating_sub(top.saturating_add(bottom));
        let body = Rect::new(
            area.x.saturating_add(self.inset.min(area.width / 2)),
            body_y,
            area.width.saturating_sub(self.inset.saturating_mul(2)),
            body_h,
        );
        Regions {
            header,
            body,
            footer,
        }
    }

    /// The reserved header and footer rows.
    pub const fn rows(&self) -> (u16, u16) {
        (self.header, self.footer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_split_the_frame_once() {
        let area = Rect::new(0, 0, 80, 24);
        let r = Chrome::new().regions(area);
        assert_eq!(r.header, Rect::new(0, 0, 80, 1));
        assert_eq!(r.footer, Rect::new(0, 23, 80, 1));
        assert_eq!(r.body, Rect::new(0, 2, 80, 20));
        let r = Chrome::new().inset(1).regions(area);
        assert_eq!(r.body, Rect::new(1, 2, 78, 20));
        let r = Chrome::new().header(0).gap(0).regions(area);
        assert_eq!(r.body, Rect::new(0, 0, 80, 23));
        assert!(r.header.is_empty());

        let r = Chrome::new().regions(Rect::new(0, 0, 10, 2));
        assert_eq!((r.header.height, r.footer.height, r.body.height), (1, 1, 0));
        let r = Chrome::new().regions(Rect::new(0, 0, 10, 0));
        assert!(r.body.is_empty() && r.footer.is_empty());
    }

    #[test]
    fn rows_reports_the_reserved_strips() {
        assert_eq!(Chrome::new().rows(), (1, 1));
        assert_eq!(Chrome::new().header(3).footer(2).rows(), (3, 2));
    }
}
