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
            SplitDir::Vertical => (
                area.height.saturating_sub(gap),
                pos.y.saturating_sub(area.y),
            ),
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

/// What a [`Stack`] does with children that do not fit its rect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Overflow {
    /// The stack scrolls by whole children and shows a scrollbar; the
    /// focused child is kept in view. The default: nothing is lost.
    #[default]
    Scroll,
    /// Children that do not fit are not laid out at all (never drawn half
    /// over a neighbour); no scrollbar.
    Clip,
}

/// One child of a [`Stack`]: the rows it wants and the fewest it accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackChild {
    /// Rows the child takes when there is room. `0` hides it (no gap).
    pub height: u16,
    /// Rows the child can shrink to before the stack overflows. Equal to
    /// `height` for a rigid child (a text field, a button row).
    pub min: u16,
}

impl StackChild {
    /// A child that always takes exactly `height` rows.
    pub const fn fixed(height: u16) -> Self {
        Self {
            height,
            min: height,
        }
    }
    /// A child that prefers `height` rows and can give up rows down to
    /// `min` (a list, a prose block) before the stack overflows.
    pub const fn flex(min: u16, height: u16) -> Self {
        Self { height, min }
    }
}

/// A vertical stack of children that measures before it places: children
/// never run past the stack's rect into the footer, a frame or the next
/// section. When the children's rows exceed the rect, flexible children
/// shrink first (never below [`Stack::min_child_rows`]); what still does
/// not fit overflows by the [`Overflow`] policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stack {
    /// Blank rows between neighbouring children. Default 0.
    pub gap: u16,
    /// What happens to children that still do not fit. Default
    /// [`Overflow::Scroll`].
    pub overflow: Overflow,
    /// The fewest rows a flexible child shrinks to before the stack
    /// overflows, whatever its own `min`. Default 1.
    pub min_child_rows: u16,
    /// Columns between the content and the scrollbar when the stack
    /// scrolls. Default 1.
    pub scrollbar_gap: u16,
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            gap: 0,
            overflow: Overflow::Scroll,
            min_child_rows: 1,
            scrollbar_gap: 1,
        }
    }
}

/// Scroll state of a [`Stack`], kept by its owner between frames. The
/// stack scrolls by whole children: `scroll.offset` is the index of the
/// first child in view, so no child is ever shown cut in half at the top.
#[derive(Debug, Clone, Default)]
pub struct StackScroll {
    pub scroll: crate::core::scroll::ScrollState,
    /// The child last brought into view for focus: the view only follows
    /// focus when it moves, so the wheel can still scroll away from it.
    kept: Option<usize>,
    /// The scrollbar track as last laid out (empty when it fits).
    track: Rect,
}

impl StackScroll {
    /// Wheel over the stack: one whole child per notch, whatever the
    /// app's rows-per-notch `delta` is (a child is already several rows).
    pub fn on_wheel(&mut self, delta: i32) -> bool {
        self.scroll.scroll_by(delta.signum() as isize)
    }
    /// A press or click on the stack's scrollbar.
    pub fn on_scrollbar(&mut self, pos: Position) -> bool {
        !self.track.is_empty()
            && crate::widgets::scrollbar::press(self.track, pos, &mut self.scroll)
    }
    /// A drag along the stack's scrollbar after a press.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> bool {
        !self.track.is_empty() && crate::widgets::scrollbar::drag(self.track, pos, &mut self.scroll)
    }
    /// Show the start again (a form that reopens, a page change).
    pub fn reset(&mut self) {
        self.scroll.offset = 0;
        self.kept = None;
    }
}

/// Where a [`Stack`] placed its children this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackLayout {
    /// One entry per child: its rect when it is in view (always inside the
    /// stack's rect), `None` when it is scrolled out, clipped away or
    /// hidden. A child taller than the whole rect gets the rect.
    pub slots: Vec<Option<Rect>>,
    /// The scrollbar track, when the stack scrolls.
    pub track: Option<Rect>,
    /// Children above the view and below it (for a "more" marker).
    pub hidden_above: usize,
    pub hidden_below: usize,
}

impl StackLayout {
    /// The rect of child `i`, if it is in view.
    pub fn slot(&self, i: usize) -> Option<Rect> {
        self.slots.get(i).copied().flatten()
    }
}

impl Stack {
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }
    pub fn min_child_rows(mut self, rows: u16) -> Self {
        self.min_child_rows = rows;
        self
    }
    pub fn scrollbar_gap(mut self, cols: u16) -> Self {
        self.scrollbar_gap = cols;
        self
    }

    /// Rows `heights` take stacked, gaps included (hidden children skip
    /// their gap).
    fn total(&self, heights: &[u16]) -> u32 {
        let shown = heights.iter().filter(|&&h| h > 0).count() as u32;
        heights.iter().map(|&h| h as u32).sum::<u32>() + self.gap as u32 * shown.saturating_sub(1)
    }

    /// The rows every child would take with no limit: the height the stack
    /// asks for.
    pub fn measure(&self, children: &[StackChild]) -> u16 {
        let h: Vec<u16> = children.iter().map(|c| c.height).collect();
        self.total(&h).min(u16::MAX as u32) as u16
    }

    /// Lay `children` out in `area`. `focus` is the child holding keyboard
    /// focus, if any: when it changes, the view scrolls the least that
    /// brings it fully into view.
    pub fn layout(
        &self,
        area: Rect,
        children: &[StackChild],
        state: &mut StackScroll,
        focus: Option<usize>,
    ) -> StackLayout {
        let n = children.len();
        let mut h: Vec<u16> = children.iter().map(|c| c.height).collect();
        let room = area.height as u32;
        // flexible children give up rows first, last child first
        let mut deficit = self.total(&h).saturating_sub(room);
        for (i, c) in children.iter().enumerate().rev() {
            if deficit == 0 {
                break;
            }
            let floor = c.min.max(self.min_child_rows).min(c.height);
            let give = (h[i] - floor).min(deficit.min(u16::MAX as u32) as u16);
            h[i] -= give;
            deficit -= give as u32;
        }
        let fits = self.total(&h) <= room;
        let scrolls = !fits && self.overflow == Overflow::Scroll && n > 0;
        let (content, track) = if scrolls && area.width > self.scrollbar_gap + 1 {
            let w = area.width - self.scrollbar_gap - 1;
            (
                Rect::new(area.x, area.y, w, area.height),
                Some(Rect::new(area.right() - 1, area.y, 1, area.height)),
            )
        } else {
            (area, None)
        };
        // the furthest first child that still fills the view to the end
        let max_first = if scrolls {
            (0..n)
                .find(|&i| self.total(&h[i..]) <= room)
                .unwrap_or(n - 1)
        } else {
            0
        };
        let mut first = if scrolls {
            state.scroll.offset.min(max_first)
        } else {
            0
        };
        if scrolls
            && let Some(k) = focus.filter(|&k| k < n)
            && state.kept != Some(k)
        {
            if k < first {
                first = k;
            } else {
                while first < k && self.total(&h[first..=k]) > room {
                    first += 1;
                }
            }
        }
        state.kept = focus;
        state.scroll.content_len = if scrolls { n } else { 0 };
        state.scroll.viewport_len = n - max_first;
        state.scroll.offset = first;
        state.track = track.unwrap_or_default();

        let mut slots = vec![None; n];
        let mut y = area.y as u32;
        let mut placed = false;
        let mut hidden_below = 0;
        let mut full = false;
        for i in first..n {
            if h[i] == 0 {
                continue;
            }
            if full {
                hidden_below += 1;
                continue;
            }
            let top = if placed { y + self.gap as u32 } else { y };
            if top + h[i] as u32 <= area.bottom() as u32 {
                slots[i] = Some(Rect::new(content.x, top as u16, content.width, h[i]));
                y = top + h[i] as u32;
                placed = true;
            } else if !placed && top < area.bottom() as u32 {
                // a single child taller than the stack gets the whole rect
                slots[i] = Some(Rect::new(
                    content.x,
                    top as u16,
                    content.width,
                    area.bottom() - top as u16,
                ));
                y = area.bottom() as u32;
                placed = true;
                full = true;
            } else {
                full = true;
                hidden_below += 1;
            }
        }
        let hidden_above = h[..first].iter().filter(|&&r| r > 0).count();
        StackLayout {
            slots,
            track,
            hidden_above,
            hidden_below,
        }
    }

    /// Draw the scrollbar of a laid-out stack and register `area` as its
    /// wheel region under `id` (wheel and track events come back to the
    /// owner as `id` and [`scrollbar::id_for`](crate::widgets::scrollbar::id_for)`(id)`).
    pub fn chrome(
        layout: &StackLayout,
        area: Rect,
        buf: &mut ratatui::buffer::Buffer,
        ctx: &mut crate::ui::ctx::RenderCtx,
        id: crate::core::id::WidgetId,
        state: &StackScroll,
    ) {
        if let Some(track) = layout.track {
            ctx.scrollable(id, area);
            crate::widgets::scrollbar::render_vertical(track, buf, ctx, id, &state.scroll, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kids(h: &[u16]) -> Vec<StackChild> {
        h.iter().map(|&h| StackChild::fixed(h)).collect()
    }

    #[test]
    fn a_stack_that_fits_places_children_with_gaps_and_no_scrollbar() {
        let s = Stack::default().gap(1);
        let mut st = StackScroll::default();
        let l = s.layout(Rect::new(2, 3, 20, 10), &kids(&[3, 0, 3, 1]), &mut st, None);
        assert_eq!(l.slot(0), Some(Rect::new(2, 3, 20, 3)));
        assert_eq!(l.slot(1), None, "a hidden child takes no gap");
        assert_eq!(l.slot(2), Some(Rect::new(2, 7, 20, 3)));
        assert_eq!(l.slot(3), Some(Rect::new(2, 11, 20, 1)));
        assert_eq!(l.track, None);
        assert!(!st.scroll.overflows());
    }

    #[test]
    fn overflow_never_places_a_child_past_the_rect() {
        let area = Rect::new(0, 0, 30, 8);
        for overflow in [Overflow::Scroll, Overflow::Clip] {
            let s = Stack::default().overflow(overflow);
            let mut st = StackScroll::default();
            let l = s.layout(area, &kids(&[3, 3, 3, 3, 1]), &mut st, None);
            for r in l.slots.iter().flatten() {
                assert!(r.bottom() <= area.bottom(), "{overflow:?}: {r:?}");
            }
            assert_eq!(l.slot(2), None, "the third field does not fit");
            assert_eq!(l.hidden_below, 3);
            assert_eq!(l.track.is_some(), overflow == Overflow::Scroll);
        }
    }

    #[test]
    fn flexible_children_shrink_before_the_stack_scrolls() {
        let s = Stack::default().min_child_rows(2);
        let mut st = StackScroll::default();
        let l = s.layout(
            Rect::new(0, 0, 30, 8),
            &[StackChild::fixed(3), StackChild::flex(0, 10)],
            &mut st,
            None,
        );
        assert_eq!(l.slot(1), Some(Rect::new(0, 3, 30, 5)));
        assert_eq!(l.track, None);
        // below min_child_rows the stack scrolls instead
        let l = s.layout(
            Rect::new(0, 0, 30, 4),
            &[StackChild::fixed(3), StackChild::flex(0, 10)],
            &mut st,
            None,
        );
        assert!(l.track.is_some());
    }

    #[test]
    fn focus_scrolls_the_least_and_the_wheel_can_move_away() {
        let s = Stack::default();
        let area = Rect::new(0, 0, 30, 7);
        let mut st = StackScroll::default();
        let ch = kids(&[3, 3, 3, 3, 1]);
        let l = s.layout(area, &ch, &mut st, Some(3));
        assert_eq!(
            st.scroll.offset, 2,
            "the fourth field is now the last in view"
        );
        assert_eq!(l.slot(3), Some(Rect::new(0, 3, 28, 3)));
        assert_eq!(l.slot(1), None);
        assert_eq!(l.hidden_above, 2);
        // the wheel scrolls back up; focus did not move, so it stays there
        st.on_wheel(-3);
        st.on_wheel(-3);
        let l = s.layout(area, &ch, &mut st, Some(3));
        assert_eq!(st.scroll.offset, 0);
        assert_eq!(l.slot(3), None);
        // moving focus up from below brings the child to the top
        let l = s.layout(area, &ch, &mut st, Some(4));
        assert!(l.slot(4).is_some());
        let l = s.layout(area, &ch, &mut st, Some(0));
        assert_eq!(l.slot(0), Some(Rect::new(0, 0, 28, 3)));
    }

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
        assert_eq!(
            s.handle(SplitDir::Horizontal, area, 1),
            Rect::new(50, 0, 1, 20)
        );
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
