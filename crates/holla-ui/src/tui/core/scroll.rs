//! Scrolling as a first-class behaviour.
//!
//! [`ScrollState`] is a pure model: content length, viewport length and
//! offset. It knows nothing about rendering; the scrollbar widget derives
//! geometry from it.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    pub offset: usize,
    pub content_len: usize,
    pub viewport_len: usize,
    /// Rows between the thumb's top and the pointer while the thumb is
    /// held, so a drag moves the thumb with the pointer instead of
    /// centring it under the pointer on the first motion.
    pub(crate) grab: Option<usize>,
}

impl ScrollState {
    pub fn new(content_len: usize) -> Self {
        Self {
            offset: 0,
            content_len,
            viewport_len: 0,
            grab: None,
        }
    }

    pub fn max_offset(&self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }

    pub fn overflows(&self) -> bool {
        self.content_len > self.viewport_len && self.viewport_len > 0
    }

    /// The viewport shows the last row of the content.
    pub fn at_end(&self) -> bool {
        self.offset >= self.max_offset()
    }

    pub fn set_viewport(&mut self, len: usize) {
        self.viewport_len = len;
        self.clamp();
    }

    pub fn set_content(&mut self, len: usize) {
        self.content_len = len;
        self.clamp();
    }

    pub fn clamp(&mut self) {
        self.offset = self.offset.min(self.max_offset());
    }

    /// Move the viewport by `delta` rows, clamped. Returns whether the
    /// offset changed, so a caller can tell a repaint from a consumed
    /// no-op at the boundary.
    pub fn scroll_by(&mut self, delta: isize) -> bool {
        let before = self.offset;
        self.offset = self
            .offset
            .saturating_add_signed(delta)
            .min(self.max_offset());
        self.offset != before
    }

    pub fn scroll_to(&mut self, offset: usize) -> bool {
        let before = self.offset;
        self.offset = offset.min(self.max_offset());
        self.offset != before
    }

    pub fn page_up(&mut self) -> bool {
        self.scroll_by(-(self.viewport_len.max(1) as isize))
    }

    pub fn page_down(&mut self) -> bool {
        self.scroll_by(self.viewport_len.max(1) as isize)
    }

    pub fn jump_start(&mut self) -> bool {
        self.scroll_to(0)
    }

    pub fn jump_end(&mut self) -> bool {
        self.scroll_to(self.max_offset())
    }

    /// Move the viewport the minimum amount so `index` is visible.
    pub fn ensure_visible(&mut self, index: usize) -> bool {
        if self.viewport_len == 0 {
            return false;
        }
        let before = self.offset;
        if index < self.offset {
            self.offset = index;
        } else if index >= self.offset + self.viewport_len {
            self.offset = index + 1 - self.viewport_len;
        }
        self.clamp();
        self.offset != before
    }

    /// Range of content indices currently in view.
    pub fn visible_range(&self) -> std::ops::Range<usize> {
        let end = (self.offset + self.viewport_len).min(self.content_len);
        self.offset..end
    }

    /// Thumb geometry for a track of `track_len` cells: (start, len).
    pub fn thumb(&self, track_len: usize) -> (usize, usize) {
        if !self.overflows() || track_len == 0 {
            return (0, track_len);
        }
        let len = ((self.viewport_len * track_len) / self.content_len).max(1);
        let len = len.min(track_len);
        let max_off = self.max_offset();
        let start = ((self.offset * (track_len - len)) + max_off / 2)
            .checked_div(max_off)
            .unwrap_or(0);
        (start.min(track_len - len), len)
    }

    /// The offset whose thumb starts at `start` on a track of `track_len`
    /// cells: the inverse of [`thumb`](Self::thumb).
    pub fn offset_for_thumb_start(&self, start: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        let usable = track_len.saturating_sub(len).max(1);
        let start = start.min(usable);
        (start * self.max_offset() + usable / 2) / usable
    }

    /// Map a track position to an offset, centring the thumb under the
    /// pointer (a click on the track).
    pub fn offset_for_track_pos(&self, pos: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        self.offset_for_thumb_start(pos.saturating_sub(len / 2), track_len)
    }

    /// The pointer went down on the track at row `pos`. On the thumb this
    /// only remembers where the thumb was grabbed; on the track it jumps
    /// the thumb under the pointer. Returns whether the offset changed.
    pub fn press_track(&mut self, pos: usize, track_len: usize) -> bool {
        if !self.overflows() || track_len == 0 {
            self.grab = None;
            return false;
        }
        let (start, len) = self.thumb(track_len);
        if (start..start + len).contains(&pos) {
            self.grab = Some(pos - start);
            return false;
        }
        self.grab = Some(len / 2);
        self.scroll_to(self.offset_for_track_pos(pos, track_len))
    }

    /// The pointer moved to row `pos` while held: the thumb follows it,
    /// keeping the grabbed row under the pointer. A drag without a press
    /// behaves like a press. Returns whether the offset changed.
    pub fn drag_track(&mut self, pos: usize, track_len: usize) -> bool {
        if !self.overflows() || track_len == 0 {
            return false;
        }
        let Some(grab) = self.grab else {
            return self.press_track(pos, track_len);
        };
        let target = self.offset_for_thumb_start(pos.saturating_sub(grab), track_len);
        self.scroll_to(target)
    }

    /// The pointer was released: the next press starts a new grab.
    pub fn release_track(&mut self) {
        self.grab = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_reports_whether_the_offset_changed() {
        let mut s = ScrollState::new(20);
        s.set_viewport(5);
        assert!(!s.scroll_by(-1), "already at the top");
        assert!(s.scroll_by(1));
        assert!(s.jump_end());
        assert!(!s.page_down(), "already at the end");
        assert!(s.at_end());
        assert!(s.page_up());
        assert!(!s.ensure_visible(s.offset), "visible rows need no scroll");
        assert!(s.ensure_visible(19));
        let mut fits = ScrollState::new(3);
        fits.set_viewport(5);
        assert!(!fits.scroll_by(3));
        assert!(fits.at_end());
    }

    #[test]
    fn pressing_the_thumb_grabs_it_and_dragging_keeps_the_grabbed_row_under_the_pointer() {
        let mut s = ScrollState::new(120);
        s.set_viewport(31);
        let track = 31;
        let (start, len) = s.thumb(track);
        assert_eq!((start, len), (0, 8));
        // a press inside the thumb does not move the view
        assert!(!s.press_track(7, track));
        assert_eq!(s.offset, 0);
        // one row of pointer motion moves the thumb one row, not fifteen
        assert!(s.drag_track(8, track));
        assert_eq!(s.thumb(track).0, 1);
        assert!(s.offset > 0 && s.offset < 8, "{}", s.offset);
        // dragging to the bottom of the track reaches the end exactly
        assert!(s.drag_track(track - 1, track));
        assert!(s.at_end());
        assert_eq!(s.thumb(track).0, track - len);
        // and back to the top
        assert!(s.drag_track(0, track));
        assert_eq!(s.offset, 0);
        // a press on the bare track jumps the thumb under the pointer
        s.release_track();
        assert!(s.press_track(20, track));
        let (start, len) = s.thumb(track);
        assert!(
            (start..start + len).contains(&20),
            "{start}..{}",
            start + len
        );
        // a drag without a press falls back to a press
        let mut fresh = ScrollState::new(120);
        fresh.set_viewport(31);
        assert!(fresh.drag_track(track - 1, track));
        assert!(fresh.at_end());
    }

    #[test]
    fn clamps_offset_to_content() {
        let mut s = ScrollState::new(100);
        s.set_viewport(10);
        s.scroll_by(500);
        assert_eq!(s.offset, 90);
        s.scroll_by(-500);
        assert_eq!(s.offset, 0);
        s.page_down();
        assert_eq!(s.offset, 10);
        s.jump_end();
        assert_eq!(s.offset, s.max_offset());
    }

    #[test]
    fn ensure_visible_moves_minimally() {
        let mut s = ScrollState::new(50);
        s.set_viewport(10);
        s.ensure_visible(25);
        assert_eq!(s.offset, 16);
        s.ensure_visible(5);
        assert_eq!(s.offset, 5);
        s.ensure_visible(7);
        assert_eq!(s.offset, 5);
    }

    #[test]
    fn thumb_covers_track_proportionally() {
        let mut s = ScrollState::new(100);
        s.set_viewport(20);
        assert_eq!(s.thumb(10), (0, 2));
        s.jump_end();
        assert_eq!(s.thumb(10), (8, 2));
        let mut s = ScrollState::new(5);
        s.set_viewport(20);
        assert!(!s.overflows());
        assert_eq!(s.thumb(10), (0, 10));
    }

    #[test]
    fn track_position_round_trips() {
        let mut s = ScrollState::new(200);
        s.set_viewport(20);
        let off = s.offset_for_track_pos(19, 20);
        assert_eq!(off, s.max_offset());
        assert_eq!(s.offset_for_track_pos(0, 20), 0);
    }
}
