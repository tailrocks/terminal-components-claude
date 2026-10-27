//! Ported from PR #26 `src/shell/chrome.rs` (commit `f4867e9b5`). The source's

use ratatui_core::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Regions {
    pub header: Rect,

    pub body: Rect,

    pub footer: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chrome {
    pub header: u16,

    pub gap: u16,

    pub footer: u16,

    pub inset: u16,
}

impl Default for Chrome {
    fn default() -> Self {
        Self::new()
    }
}

impl Chrome {
    pub const fn new() -> Self {
        Self {
            header: 1,
            gap: 1,
            footer: 1,
            inset: 0,
        }
    }

    #[must_use]
    pub const fn header(mut self, rows: u16) -> Self {
        self.header = rows;
        self
    }

    #[must_use]
    pub const fn gap(mut self, rows: u16) -> Self {
        self.gap = rows;
        self
    }

    #[must_use]
    pub const fn footer(mut self, rows: u16) -> Self {
        self.footer = rows;
        self
    }

    #[must_use]
    pub const fn inset(mut self, cols: u16) -> Self {
        self.inset = cols;
        self
    }

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

    pub const fn rows(&self) -> (u16, u16) {
        (self.header, self.footer)
    }
}
