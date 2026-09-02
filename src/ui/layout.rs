//! Split panes and other layout helpers that the workbench composes.

use ratatui::layout::{Position, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDir {
    /// First pane on the left.
    Horizontal,
    /// First pane on top.
    Vertical,
}

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
        self.percent = (self.percent as i16 + delta).clamp(5, 95) as u16;
    }

    /// Layout in either direction.
    pub fn layout(&self, dir: SplitDir, area: Rect, gap: u16) -> (Rect, Rect) {
        match dir {
            SplitDir::Horizontal => self.horizontal(area, gap),
            SplitDir::Vertical => self.vertical(area, gap),
        }
    }

    /// The gap strip between the two panes; empty when one is maximised
    /// or the split collapsed.
    pub fn handle(&self, dir: SplitDir, area: Rect, gap: u16) -> Rect {
        let (a, b) = self.layout(dir, area, gap);
        if a.is_empty() || b.is_empty() || gap == 0 {
            return Rect::ZERO;
        }
        match dir {
            SplitDir::Horizontal => Rect::new(a.right(), area.y, gap, area.height),
            SplitDir::Vertical => Rect::new(area.x, a.bottom(), area.width, gap),
        }
    }

    /// Put the seam under `pos` (clamped by the minima). Returns whether
    /// the ratio changed.
    pub fn drag_to(&mut self, dir: SplitDir, area: Rect, gap: u16, pos: Position) -> bool {
        let (usable, offset) = match dir {
            SplitDir::Horizontal => (area.width.saturating_sub(gap), pos.x.saturating_sub(area.x)),
            SplitDir::Vertical => (area.height.saturating_sub(gap), pos.y.saturating_sub(area.y)),
        };
        if usable < self.min_first + self.min_second || usable == 0 {
            return false;
        }
        let first = offset.clamp(self.min_first, usable - self.min_second);
        let percent = ((first as u32 * 100 + usable as u32 / 2) / usable as u32) as u16;
        let percent = percent.clamp(5, 95);
        let changed = percent != self.percent;
        self.percent = percent;
        changed
    }

    /// Resize by whole cells in the given direction.
    pub fn nudge(&mut self, dir: SplitDir, area: Rect, gap: u16, delta: i16) {
        let (first, _) = self.layout(dir, area, gap);
        let cur = match dir {
            SplitDir::Horizontal => first.width,
            SplitDir::Vertical => first.height,
        } as i32;
        let target = (cur + delta as i32).max(0) as u16;
        let pos = match dir {
            SplitDir::Horizontal => Position::new(area.x + target, area.y),
            SplitDir::Vertical => Position::new(area.x, area.y + target),
        };
        self.drag_to(dir, area, gap, pos);
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

    #[test]
    fn drag_moves_the_seam_and_respects_minima() {
        let mut s = Split::new(50, 10, 10);
        let area = Rect::new(0, 0, 101, 20);
        assert_eq!(s.handle(SplitDir::Horizontal, area, 1), Rect::new(50, 0, 1, 20));
        assert!(s.drag_to(SplitDir::Horizontal, area, 1, Position::new(70, 3)));
        let (a, _) = s.horizontal(area, 1);
        assert_eq!(a.width, 70);
        s.drag_to(SplitDir::Horizontal, area, 1, Position::new(2, 3));
        let (a, _) = s.horizontal(area, 1);
        assert_eq!(a.width, 10, "clamped to min_first");
        s.nudge(SplitDir::Horizontal, area, 1, 5);
        let (a, _) = s.horizontal(area, 1);
        assert_eq!(a.width, 15);
    }
}
