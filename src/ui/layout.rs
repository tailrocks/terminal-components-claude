//! Split panes and other layout helpers that the workbench composes.

use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maximized {
    None,
    First,
    Second,
}

/// A two-pane split with a ratio (in percent of the first pane) and a
/// maximize state. Direction is chosen by the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    pub percent: u16,
    pub min_first: u16,
    pub min_second: u16,
    pub maximized: Maximized,
}

impl Split {
    pub const fn new(percent: u16, min_first: u16, min_second: u16) -> Self {
        Self {
            percent,
            min_first,
            min_second,
            maximized: Maximized::None,
        }
    }

    pub fn toggle_max(&mut self, which: Maximized) {
        self.maximized = if self.maximized == which {
            Maximized::None
        } else {
            which
        };
    }

    pub fn grow(&mut self, delta: i16) {
        self.percent = (self.percent as i16 + delta).clamp(10, 90) as u16;
    }

    /// Vertical split: first pane on top. `gap` rows between them.
    pub fn vertical(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        match self.maximized {
            Maximized::First => (area, Rect::ZERO),
            Maximized::Second => (Rect::ZERO, area),
            Maximized::None => {
                let usable = area.height.saturating_sub(gap);
                if usable < self.min_first + self.min_second {
                    // not enough room for both: give everything to the first
                    return (area, Rect::ZERO);
                }
                let mut first = (usable as u32 * self.percent as u32 / 100) as u16;
                first = first.clamp(self.min_first, usable - self.min_second);
                (
                    Rect::new(area.x, area.y, area.width, first),
                    Rect::new(area.x, area.y + first + gap, area.width, usable - first),
                )
            }
        }
    }

    /// Horizontal split: first pane on the left.
    pub fn horizontal(&self, area: Rect, gap: u16) -> (Rect, Rect) {
        match self.maximized {
            Maximized::First => (area, Rect::ZERO),
            Maximized::Second => (Rect::ZERO, area),
            Maximized::None => {
                let usable = area.width.saturating_sub(gap);
                if usable < self.min_first + self.min_second {
                    return (Rect::ZERO, area);
                }
                let mut first = (usable as u32 * self.percent as u32 / 100) as u16;
                first = first.clamp(self.min_first, usable - self.min_second);
                (
                    Rect::new(area.x, area.y, first, area.height),
                    Rect::new(area.x + first + gap, area.y, usable - first, area.height),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_respect_minimums_and_maximize() {
        let s = Split::new(60, 5, 5);
        let (a, b) = s.vertical(Rect::new(0, 0, 80, 30), 1);
        assert_eq!(a.height + b.height + 1, 30);
        assert_eq!(a.height, 17);
        let mut m = s;
        m.toggle_max(Maximized::Second);
        let (a, b) = m.vertical(Rect::new(0, 0, 80, 30), 1);
        assert!(a.is_empty());
        assert_eq!(b.height, 30);
        let (a, b) = s.vertical(Rect::new(0, 0, 80, 8), 1);
        assert_eq!(a.height, 8);
        assert!(b.is_empty());
    }
}
