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
}

impl ScrollState {
    pub fn new(content_len: usize) -> Self {
        Self {
            offset: 0,
            content_len,
            viewport_len: 0,
        }
    }

    pub fn max_offset(&self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }

    pub fn overflows(&self) -> bool {
        self.content_len > self.viewport_len && self.viewport_len > 0
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

    pub fn scroll_by(&mut self, delta: isize) {
        self.offset = self
            .offset
            .saturating_add_signed(delta)
            .min(self.max_offset());
    }

    pub fn scroll_to(&mut self, offset: usize) {
        self.offset = offset.min(self.max_offset());
    }

    pub fn page_up(&mut self) {
        self.scroll_by(-(self.viewport_len.max(1) as isize));
    }

    pub fn page_down(&mut self) {
        self.scroll_by(self.viewport_len.max(1) as isize);
    }

    pub fn jump_start(&mut self) {
        self.offset = 0;
    }

    pub fn jump_end(&mut self) {
        self.offset = self.max_offset();
    }

    /// Move the viewport the minimum amount so `index` is visible.
    pub fn ensure_visible(&mut self, index: usize) {
        if self.viewport_len == 0 {
            return;
        }
        if index < self.offset {
            self.offset = index;
        } else if index >= self.offset + self.viewport_len {
            self.offset = index + 1 - self.viewport_len;
        }
        self.clamp();
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

    /// Inverse of [`thumb`](Self::thumb): map a track position to an offset
    /// (used for scrollbar click / drag).
    pub fn offset_for_track_pos(&self, pos: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        let usable = track_len.saturating_sub(len).max(1);
        let pos = pos.saturating_sub(len / 2).min(usable);
        (pos * self.max_offset() + usable / 2) / usable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
