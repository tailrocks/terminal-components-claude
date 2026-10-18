#![doc = r" Scrolling as a pure model (`COMPONENT_ARCHITECTURE.md` §8.3, §18.1)."]
#![doc = r""]
#![doc = r" [`ScrollState`] knows content length, viewport length and offset and"]
#![doc = r" nothing about rendering. Every field is private and every mutator"]
#![doc = r" clamps. `ensure_visible_on_next_layout` is set only by cursor motion and"]
#![doc = r" consumed by the next `draw`, generalising `Picker::cursor_dirty`."]

use core::ops::Range;

use crate::response::Response;
use termrock_core::geometry::Headroom;

#[doc = r" Offset, content length and viewport length on one axis."]
#[doc = r""]
#[doc = r" A second instance models the other axis: the grid's column window and"]
#[doc = r" the tab strip's window are each a `ScrollState` over item ordinals while"]
#[doc = r" the vertical state stays untouched. Size `content_len`/`viewport_len` so"]
#[doc = r" `max_offset` is the window's legal range. A full window that always shows"]
#[doc = r" `viewport` items uses `content_len = len`; a window that may rest"]
#[doc = r" partially past the end — first index in `0..len`, as the tab strip's"]
#[doc = r" trailing window does — uses `content_len = len + viewport_len - 1` so"]
#[doc = r" `max_offset` is `len - 1` and `ensure_visible`/`scroll_by` never clamp a"]
#[doc = r" legal window. A target-first reveal with no meaningful viewport (the"]
#[doc = r" grid's `Whole` fit) keeps `viewport_len` zero and reveals with"]
#[doc = r" `scroll_to`, which then cannot clamp a live ordinal either."]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    offset: usize,
    content_len: usize,
    viewport_len: usize,
    reveal: Option<usize>,
    #[doc = r" The thumb row held by an in-progress track press, so a drag keeps"]
    #[doc = r" the grabbed row under the pointer instead of recentring the thumb."]
    grab: Option<usize>,
}

impl ScrollState {
    #[doc = r" A state for `content_len` items with no viewport yet."]
    pub const fn new(content_len: usize) -> Self {
        ScrollState {
            offset: 0,
            content_len,
            viewport_len: 0,
            reveal: None,
            grab: None,
        }
    }

    #[doc = r" The first visible index."]
    pub const fn offset(&self) -> usize {
        self.offset
    }

    #[doc = r" The content length."]
    pub const fn content_len(&self) -> usize {
        self.content_len
    }

    #[doc = r" The viewport length."]
    pub const fn viewport_len(&self) -> usize {
        self.viewport_len
    }

    #[doc = r" The largest legal offset."]
    pub const fn max_offset(&self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }

    #[doc = r" Whether content exceeds the viewport."]
    pub const fn overflows(&self) -> bool {
        self.content_len > self.viewport_len && self.viewport_len > 0
    }

    #[doc = r" Whether the offset is at the start."]
    pub const fn at_start(&self) -> bool {
        self.offset == 0
    }

    #[doc = r" Whether the offset is at the end."]
    pub const fn at_end(&self) -> bool {
        self.offset >= self.max_offset()
    }

    #[doc = r" Set the viewport length and clamp."]
    pub fn set_viewport(&mut self, len: usize) {
        self.viewport_len = len;
        self.clamp();
    }

    #[doc = r" Set the content length and clamp."]
    pub fn set_content(&mut self, len: usize) {
        self.content_len = len;
        self.clamp();
    }

    #[doc = r" Clamp the offset to the content."]
    pub fn clamp(&mut self) {
        self.offset = self.offset.min(self.max_offset());
    }

    #[doc = r" Scroll by a signed delta, clamped."]
    pub fn scroll_by(&mut self, delta: isize) {
        self.offset = self
            .offset
            .saturating_add_signed(delta)
            .min(self.max_offset());
    }

    #[doc = r" Scroll to an offset, clamped."]
    pub fn scroll_to(&mut self, offset: usize) {
        self.offset = offset.min(self.max_offset());
    }

    #[doc = r" Scroll up by one viewport."]
    pub fn page_up(&mut self) {
        let page = self.viewport_len.max(1) as isize;
        self.scroll_by(page.saturating_neg());
    }

    #[doc = r" Scroll down by one viewport."]
    pub fn page_down(&mut self) {
        self.scroll_by(self.viewport_len.max(1) as isize);
    }

    #[doc = r" Jump to the start."]
    pub fn jump_start(&mut self) {
        self.offset = 0;
    }

    #[doc = r" Jump to the end."]
    pub fn jump_end(&mut self) {
        self.offset = self.max_offset();
    }

    #[doc = r" Move the viewport the minimum amount so `index` is visible."]
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

    #[doc = r" Request that `index` be revealed by the next layout (cursor motion"]
    #[doc = r" only — a wheel never sets this)."]
    pub fn ensure_visible_on_next_layout(&mut self, index: usize) {
        self.reveal = Some(index);
    }

    #[doc = r" The pending reveal request, if any."]
    pub const fn pending_reveal(&self) -> Option<usize> {
        self.reveal
    }

    #[doc = r" Clear any pending reveal request."]
    pub fn clear_reveal(&mut self) {
        self.reveal = None;
    }

    #[doc = r" Apply known viewport geometry. A zero-height layout cannot reveal a"]
    #[doc = r" row, so it retains the request until a usable viewport is available."]
    pub fn apply_layout(&mut self, viewport_len: usize, content_len: usize) {
        self.viewport_len = viewport_len;
        self.content_len = content_len;
        if viewport_len > 0
            && let Some(i) = self.reveal.take()
        {
            self.ensure_visible(i);
        }
        self.clamp();
    }

    #[doc = r" The wheel rule (§8.3): consumed even at the boundary, repaint only"]
    #[doc = r" when the offset moved."]
    pub fn wheel(&mut self, delta: i16) -> Response<()> {
        let before = self.offset;
        self.scroll_by(delta as isize);
        if self.offset == before {
            Response::consumed()
        } else {
            Response::changed()
        }
    }

    #[doc = r" Range of content indices currently in view."]
    pub fn visible_range(&self) -> Range<usize> {
        let end = self
            .offset
            .saturating_add(self.viewport_len)
            .min(self.content_len);
        self.offset..end
    }

    #[doc = r" Headroom on the vertical axis, for `register_scroll`."]
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

    #[doc = r" Thumb geometry for a track of `track_len` cells: `(start, len)`."]
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

    #[doc = r" Inverse of [`thumb`](Self::thumb): the offset whose thumb starts at"]
    #[doc = r" `start` on a track of `track_len` cells."]
    pub fn offset_for_thumb_start(&self, start: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        let usable = track_len.saturating_sub(len).max(1);
        let start = start.min(usable);
        start
            .saturating_mul(self.max_offset())
            .saturating_add(usable / 2)
            .checked_div(usable)
            .unwrap_or(0)
    }

    #[doc = r" The pointer went down on the track at row `pos`. On the thumb this"]
    #[doc = r" only remembers where the thumb was grabbed; on the bare track it"]
    #[doc = r" jumps the thumb under the pointer."]
    pub fn press_track(&mut self, pos: usize, track_len: usize) {
        if !self.overflows() || track_len == 0 {
            self.grab = None;
            return;
        }
        let (start, len) = self.thumb(track_len);
        if (start..start + len).contains(&pos) {
            self.grab = Some(pos - start);
            return;
        }
        self.grab = Some(len / 2);
        self.scroll_to(self.offset_for_track_pos(pos, track_len));
    }

    #[doc = r" The pointer moved to row `pos` while held: the thumb follows it,"]
    #[doc = r" keeping the grabbed row under the pointer. A drag without a press"]
    #[doc = r" behaves like a press."]
    pub fn drag_track(&mut self, pos: usize, track_len: usize) {
        if !self.overflows() || track_len == 0 {
            return;
        }
        let Some(grab) = self.grab else {
            self.press_track(pos, track_len);
            return;
        };
        let target = self.offset_for_thumb_start(pos.saturating_sub(grab), track_len);
        self.scroll_to(target);
    }

    #[doc = r" The pointer was released: the next press starts a new grab."]
    pub fn release_track(&mut self) {
        self.grab = None;
    }

    #[doc = r" Inverse of [`thumb`](Self::thumb): a track position to an offset."]
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
    fn zero_viewports_retain_reveal_until_usable_layout() {
        let mut s = ScrollState::new(50);
        s.ensure_visible_on_next_layout(40);
        for _ in 0..3 {
            s.apply_layout(0, 50);
            assert_eq!(s.pending_reveal(), Some(40));
        }
        s.apply_layout(10, 50);
        assert_eq!(s.pending_reveal(), None);
        assert_eq!(s.offset(), 31);
        let _ = s.wheel(-5);
        s.apply_layout(10, 50);
        assert_eq!(s.offset(), 26, "consumed reveal does not undo a wheel");
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

    #[test]
    fn pressing_the_thumb_grabs_it_and_dragging_keeps_the_grabbed_row_under_the_pointer() {
        let mut s = ScrollState::new(120);
        s.set_viewport(31);
        let track = 31;
        let (start, len) = s.thumb(track);
        assert_eq!((start, len), (0, 8));

        s.press_track(7, track);
        assert_eq!(s.offset(), 0);

        s.drag_track(8, track);
        assert_eq!(s.thumb(track).0, 1);
        assert!(s.offset() > 0 && s.offset() < 8, "{}", s.offset());

        s.drag_track(track - 1, track);
        assert!(s.at_end());
        assert_eq!(s.thumb(track).0, track - len);

        s.drag_track(0, track);
        assert_eq!(s.offset(), 0);

        s.release_track();
        s.press_track(20, track);
        let (start, len) = s.thumb(track);
        assert!(
            (start..start + len).contains(&20),
            "{start}..{}",
            start + len
        );

        let mut fresh = ScrollState::new(120);
        fresh.set_viewport(31);
        fresh.drag_track(track - 1, track);
        assert!(fresh.at_end());
    }

    #[test]
    fn track_press_and_drag_are_no_ops_without_overflow() {
        let mut s = ScrollState::new(3);
        s.set_viewport(5);
        s.press_track(2, 5);
        s.drag_track(4, 5);
        assert_eq!(s.offset(), 0);
        s.release_track();
        assert_eq!(s.offset(), 0);
    }
}
