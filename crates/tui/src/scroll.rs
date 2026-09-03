//! Scrolling as a pure model (`COMPONENT_ARCHITECTURE.md` §8.3, §18.1).
//!
//! [`ScrollState`] knows content length, viewport length and offset and
//! nothing about rendering. Every field is private and every mutator
//! clamps. `ensure_visible_on_next_layout` is set only by cursor motion and
//! consumed by the next `draw`, generalising `Picker::cursor_dirty`.

use core::ops::Range;

use crate::hit::Headroom;
use crate::response::Response;

/// Offset, content length and viewport length on one axis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    offset: usize,
    content_len: usize,
    viewport_len: usize,
    reveal: Option<usize>,
}

impl ScrollState {
    /// A state for `content_len` items with no viewport yet.
    pub const fn new(content_len: usize) -> Self {
        ScrollState {
            offset: 0,
            content_len,
            viewport_len: 0,
            reveal: None,
        }
    }

    /// The first visible index.
    pub const fn offset(&self) -> usize {
        self.offset
    }

    /// The content length.
    pub const fn content_len(&self) -> usize {
        self.content_len
    }

    /// The viewport length.
    pub const fn viewport_len(&self) -> usize {
        self.viewport_len
    }

    /// The largest legal offset.
    pub const fn max_offset(&self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }

    /// Whether content exceeds the viewport.
    pub const fn overflows(&self) -> bool {
        self.content_len > self.viewport_len && self.viewport_len > 0
    }

    /// Whether the offset is at the start.
    pub const fn at_start(&self) -> bool {
        self.offset == 0
    }

    /// Whether the offset is at the end.
    pub const fn at_end(&self) -> bool {
        self.offset >= self.max_offset()
    }

    /// Set the viewport length and clamp.
    pub fn set_viewport(&mut self, len: usize) {
        self.viewport_len = len;
        self.clamp();
    }

    /// Set the content length and clamp.
    pub fn set_content(&mut self, len: usize) {
        self.content_len = len;
        self.clamp();
    }

    /// Clamp the offset to the content.
    pub fn clamp(&mut self) {
        self.offset = self.offset.min(self.max_offset());
    }

    /// Scroll by a signed delta, clamped.
    pub fn scroll_by(&mut self, delta: isize) {
        self.offset = self
            .offset
            .saturating_add_signed(delta)
            .min(self.max_offset());
    }

    /// Scroll to an offset, clamped.
    pub fn scroll_to(&mut self, offset: usize) {
        self.offset = offset.min(self.max_offset());
    }

    /// Scroll up by one viewport.
    pub fn page_up(&mut self) {
        let page = self.viewport_len.max(1) as isize;
        self.scroll_by(page.saturating_neg());
    }

    /// Scroll down by one viewport.
    pub fn page_down(&mut self) {
        self.scroll_by(self.viewport_len.max(1) as isize);
    }

    /// Jump to the start.
    pub fn jump_start(&mut self) {
        self.offset = 0;
    }

    /// Jump to the end.
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
        } else if index >= self.offset.saturating_add(self.viewport_len) {
            self.offset = index.saturating_add(1).saturating_sub(self.viewport_len);
        }
        self.clamp();
    }

    /// Request that `index` be revealed by the next layout (cursor motion
    /// only — a wheel never sets this).
    pub fn ensure_visible_on_next_layout(&mut self, index: usize) {
        self.reveal = Some(index);
    }

    /// The pending reveal request, if any.
    pub const fn pending_reveal(&self) -> Option<usize> {
        self.reveal
    }

    /// Called by `draw` after it knows the viewport: applies the pending
    /// reveal and clears it.
    pub fn apply_layout(&mut self, viewport_len: usize, content_len: usize) {
        self.viewport_len = viewport_len;
        self.content_len = content_len;
        if let Some(i) = self.reveal.take() {
            self.ensure_visible(i);
        }
        self.clamp();
    }

    /// The wheel rule (§8.3): consumed even at the boundary, repaint only
    /// when the offset moved.
    pub fn wheel(&mut self, delta: i16) -> Response<()> {
        let before = self.offset;
        self.scroll_by(delta as isize);
        if self.offset == before {
            Response::consumed()
        } else {
            Response::changed()
        }
    }

    /// Range of content indices currently in view.
    pub fn visible_range(&self) -> Range<usize> {
        let end = self
            .offset
            .saturating_add(self.viewport_len)
            .min(self.content_len);
        self.offset..end
    }

    /// Headroom on the vertical axis, for `register_scroll`.
    pub fn headroom_v(&self) -> Headroom {
        Headroom {
            up: self.offset.min(usize::from(u16::MAX)) as u16,
            down: self
                .max_offset()
                .saturating_sub(self.offset)
                .min(usize::from(u16::MAX)) as u16,
            left: 0,
            right: 0,
        }
    }

    /// Thumb geometry for a track of `track_len` cells: `(start, len)`.
    pub fn thumb(&self, track_len: usize) -> (usize, usize) {
        if !self.overflows() || track_len == 0 {
            return (0, track_len);
        }
        let len = self
            .viewport_len
            .saturating_mul(track_len)
            .checked_div(self.content_len)
            .unwrap_or(0)
            .max(1)
            .min(track_len);
        let max_off = self.max_offset();
        let usable = track_len.saturating_sub(len);
        let start = self
            .offset
            .saturating_mul(usable)
            .saturating_add(max_off / 2)
            .checked_div(max_off)
            .unwrap_or(0);
        (start.min(usable), len)
    }

    /// Inverse of [`thumb`](Self::thumb): a track position to an offset.
    pub fn offset_for_track_pos(&self, pos: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        let usable = track_len.saturating_sub(len).max(1);
        let pos = pos.saturating_sub(len / 2).min(usable);
        pos.saturating_mul(self.max_offset())
            .saturating_add(usable / 2)
            .checked_div(usable)
            .unwrap_or(0)
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
        assert_eq!(s.offset(), 90);
        s.scroll_by(-500);
        assert_eq!(s.offset(), 0);
        s.page_down();
        assert_eq!(s.offset(), 10);
        s.jump_end();
        assert_eq!(s.offset(), s.max_offset());
        s.set_content(5);
        assert_eq!(s.offset(), 0);
    }

    #[test]
    fn ensure_visible_moves_minimally() {
        let mut s = ScrollState::new(50);
        s.set_viewport(10);
        s.ensure_visible(25);
        assert_eq!(s.offset(), 16);
        s.ensure_visible(5);
        assert_eq!(s.offset(), 5);
        s.ensure_visible(7);
        assert_eq!(s.offset(), 5);
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
        assert_eq!(s.offset_for_track_pos(19, 20), s.max_offset());
        assert_eq!(s.offset_for_track_pos(0, 20), 0);
    }

    #[test]
    fn wheel_at_the_boundary_is_consumed_without_repaint() {
        let mut s = ScrollState::new(30);
        s.set_viewport(10);
        let r = s.wheel(-3);
        assert!(r.is_consumed() && !r.is_changed());
        let r = s.wheel(3);
        assert!(r.is_consumed() && r.is_changed());
        s.jump_end();
        let r = s.wheel(3);
        assert!(r.is_consumed() && !r.is_changed());
        assert!(
            s.pending_reveal().is_none(),
            "a wheel never requests a reveal"
        );
    }

    #[test]
    fn ensure_visible_on_next_layout_is_set_only_by_cursor_motion() {
        let mut s = ScrollState::new(50);
        s.set_viewport(10);
        let _ = s.wheel(5);
        assert_eq!(s.pending_reveal(), None);
        s.ensure_visible_on_next_layout(40);
        assert_eq!(s.pending_reveal(), Some(40));
        s.apply_layout(10, 50);
        assert_eq!(s.pending_reveal(), None);
        assert_eq!(s.offset(), 31);
    }

    #[test]
    fn fields_are_private_and_every_mutator_clamps() {
        let mut s = ScrollState::new(3);
        s.set_viewport(10);
        s.scroll_to(99);
        assert_eq!(s.offset(), 0);
        s.page_up();
        assert_eq!(s.offset(), 0);
        s.set_content(100);
        s.scroll_to(99);
        assert_eq!(s.offset(), 90);
        s.set_viewport(200);
        assert_eq!(s.offset(), 0);
        assert_eq!(s.visible_range(), 0..100);
        assert_eq!(s.headroom_v().down, 0);
    }
}
