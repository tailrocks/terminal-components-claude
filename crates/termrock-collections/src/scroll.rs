use core::ops::Range;

use crate::response::Response;
use termrock_core::geometry::Headroom;

/// How a view follows its cursor (ported from PR #26 `src/core/scroll.rs`,
/// commit f4867e9b5).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FollowCursor {
    /// Reveal the cursor when it moves; a wheeled view stays put until then.
    #[default]
    OnMove,
    /// Keep the cursor pinned in view on every pass.
    Always,
}

/// What clicking a scrollbar track away from the thumb does (ported from
/// PR #26 `src/core/scroll.rs`, commit f4867e9b5).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TrackClick {
    /// Jump so the thumb centers under the pointer.
    Jump,
    /// Move one viewport toward the pointer (macOS, Windows).
    #[default]
    Page,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    offset: usize,
    content_len: usize,
    viewport_len: usize,
    reveal: Option<usize>,
    grab: Option<usize>,
    /// Rows of context kept visible around a revealed index.
    context: usize,
    /// The next `ensure_visible` keeps no context (a pointer-driven reveal).
    exact: bool,
    follow_cursor: FollowCursor,
    /// The last `(cursor, viewport)` `follow` revealed, for the dedupe.
    followed: Option<(usize, usize)>,
    /// A paging track press is in progress; drags page instead of jumping.
    paging: bool,
}

impl ScrollState {
    pub const fn new(content_len: usize) -> Self {
        ScrollState {
            offset: 0,
            content_len,
            viewport_len: 0,
            reveal: None,
            grab: None,
            context: 0,
            exact: false,
            follow_cursor: FollowCursor::OnMove,
            followed: None,
            paging: false,
        }
    }

    /// Set the rows of context kept around a revealed index.
    pub const fn with_context(mut self, rows: usize) -> Self {
        self.context = rows;
        self
    }

    /// The rows of context kept around a revealed index.
    pub const fn context(&self) -> usize {
        self.context
    }

    /// Set how the view follows its cursor.
    pub const fn with_follow_cursor(mut self, follow: FollowCursor) -> Self {
        self.follow_cursor = follow;
        self
    }

    /// How the view follows its cursor.
    pub const fn follow_cursor(&self) -> FollowCursor {
        self.follow_cursor
    }

    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn content_len(&self) -> usize {
        self.content_len
    }

    pub const fn viewport_len(&self) -> usize {
        self.viewport_len
    }

    pub const fn max_offset(&self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }

    pub const fn overflows(&self) -> bool {
        self.content_len > self.viewport_len && self.viewport_len > 0
    }

    pub const fn at_start(&self) -> bool {
        self.offset == 0
    }

    pub const fn at_end(&self) -> bool {
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
        let page = self.viewport_len.max(1) as isize;
        self.scroll_by(page.saturating_neg());
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

    pub fn ensure_visible(&mut self, index: usize) -> bool {
        let exact = core::mem::take(&mut self.exact);
        if self.viewport_len == 0 {
            return false;
        }
        let before = self.offset;
        // Rows of context kept around the revealed index. A pointer-driven
        // reveal (`pointer_move`) centers exactly, with no context.
        let ctx = if !exact && self.viewport_len > 2 * self.context {
            self.context
        } else {
            0
        };
        if index < self.offset + ctx {
            self.offset = index.saturating_sub(ctx);
        } else if index + ctx >= self.offset.saturating_add(self.viewport_len) {
            self.offset = index
                .saturating_add(ctx)
                .saturating_add(1)
                .saturating_sub(self.viewport_len);
        }
        self.clamp();
        self.offset != before
    }

    /// A pointer-driven reveal: center the row exactly, with no context.
    pub fn pointer_move(&mut self) {
        self.exact = true;
    }

    /// Reveal a span of `len` rows starting at `start`, keeping `context`
    /// rows of context visible when the viewport allows it. A span at least
    /// as long as the viewport aligns its start with the top (ported from
    /// PR #26 `src/core/scroll.rs::reveal_span`).
    pub fn reveal_span(&mut self, start: usize, len: usize) -> bool {
        if self.viewport_len == 0 {
            return false;
        }
        let len = len.max(1);
        let before = self.offset;
        if len >= self.viewport_len {
            self.offset = start;
        } else {
            let ctx = if self.viewport_len >= len + 2 * self.context {
                self.context
            } else {
                0
            };
            if start < self.offset + ctx {
                self.offset = start.saturating_sub(ctx);
            } else if start + len + ctx > self.offset + self.viewport_len {
                self.offset = start + len + ctx - self.viewport_len;
            }
        }
        self.clamp();
        self.offset != before
    }

    /// Reveal the cursor according to the follow policy. With
    /// [`FollowCursor::OnMove`], repeating the same cursor in the same
    /// viewport is a no-op, so a repaint never snaps a wheeled view back
    /// (ported from PR #26 `src/core/scroll.rs::follow`).
    pub fn follow(&mut self, cursor: usize) -> bool {
        let key = (cursor, self.viewport_len);
        if self.follow_cursor == FollowCursor::OnMove && self.followed == Some(key) {
            return false;
        }
        self.followed = Some(key);
        self.ensure_visible(cursor)
    }

    pub fn ensure_visible_on_next_layout(&mut self, index: usize) {
        self.reveal = Some(index);
    }

    pub const fn pending_reveal(&self) -> Option<usize> {
        self.reveal
    }

    pub fn clear_reveal(&mut self) {
        self.reveal = None;
    }

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

    pub fn wheel(&mut self, delta: i16) -> Response<()> {
        let before = self.offset;
        self.scroll_by(delta as isize);
        if self.offset == before {
            Response::consumed()
        } else {
            Response::changed()
        }
    }

    pub fn visible_range(&self) -> Range<usize> {
        let end = self
            .offset
            .saturating_add(self.viewport_len)
            .min(self.content_len);
        self.offset..end
    }

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

    pub fn press_track(&mut self, pos: usize, track_len: usize) -> bool {
        self.paging = false;
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
        let before = self.offset;
        self.scroll_to(self.offset_for_track_pos(pos, track_len));
        self.offset != before
    }

    /// Press the track with an explicit [`TrackClick`] policy: `Jump` puts
    /// the thumb under the pointer, `Page` moves one viewport toward the
    /// pointer and starts a paging drag (ported from PR #26
    /// `src/core/scroll.rs::press_track_with`).
    pub fn press_track_with(&mut self, pos: usize, track_len: usize, click: TrackClick) -> bool {
        if click == TrackClick::Jump || !self.overflows() || track_len == 0 {
            return self.press_track(pos, track_len);
        }
        let (start, len) = self.thumb(track_len);
        if (start..start + len).contains(&pos) {
            self.grab = Some(pos - start);
            return false;
        }
        self.grab = None;
        self.paging = true;
        let before = self.offset;
        if pos < start {
            self.page_up();
        } else {
            self.page_down();
        }
        self.offset != before
    }

    pub fn drag_track(&mut self, pos: usize, track_len: usize) -> bool {
        if !self.overflows() || track_len == 0 {
            return false;
        }
        let Some(grab) = self.grab else {
            // A paging drag keeps paging; an idle drag picks the thumb up.
            if self.paging {
                return false;
            }
            return self.press_track(pos, track_len);
        };
        let before = self.offset;
        let target = self.offset_for_thumb_start(pos.saturating_sub(grab), track_len);
        self.scroll_to(target);
        self.offset != before
    }

    pub fn release_track(&mut self) {
        self.grab = None;
        self.paging = false;
    }

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

    #[test]
    fn a_paging_track_press_moves_one_viewport_toward_the_pointer() {
        let mut s = ScrollState::new(100);
        s.set_viewport(10);

        assert_eq!(TrackClick::default(), TrackClick::Page, "macOS, Windows");
        assert!(s.press_track_with(9, 10, TrackClick::Page));
        assert_eq!(s.offset(), 10, "one page right, not a jump to the end");
        assert!(s.press_track_with(0, 10, TrackClick::Page));
        assert_eq!(s.offset(), 0);
        let (start, _) = s.thumb(10);
        assert!(!s.press_track_with(start, 10, TrackClick::Page));
    }

    #[test]
    fn motion_after_a_paging_press_never_jumps_the_thumb() {
        let mut s = ScrollState::new(100);
        s.set_viewport(10);
        assert!(s.press_track_with(9, 10, TrackClick::Page));
        assert_eq!(s.offset(), 10);

        assert!(!s.drag_track(8, 10));
        assert_eq!(s.offset(), 10);

        s.release_track();
        assert!(s.press_track_with(9, 10, TrackClick::Jump));
        assert!(s.drag_track(0, 10));
        assert_eq!(s.offset(), 0);
    }

    #[test]
    fn follow_on_move_leaves_a_wheeled_view_alone_until_the_cursor_moves() {
        let mut s = ScrollState::new(50).with_context(1);
        s.set_viewport(10);
        s.follow(3);
        assert_eq!(s.offset(), 0);

        s.scroll_by(20);
        s.follow(3);
        assert_eq!(s.offset(), 20, "a repaint must not snap back");

        s.follow(4);
        assert_eq!(s.offset(), 3);

        s.scroll_by(20);
        s.set_viewport(8);
        s.follow(4);
        assert_eq!(s.offset(), 3);
    }

    #[test]
    fn follow_always_pins_the_cursor() {
        let mut s = ScrollState::new(50)
            .with_context(1)
            .with_follow_cursor(FollowCursor::Always);
        s.set_viewport(10);
        s.follow(3);
        s.scroll_by(20);
        s.follow(3);
        assert_eq!(s.offset(), 2, "pinned with the default row of context");
    }

    #[test]
    fn reveal_span_keeps_context_and_aligns_a_long_span_to_the_top() {
        let mut s = ScrollState::new(100).with_context(2);
        s.set_viewport(10);
        // A span below the view: reveal it with its context above.
        assert!(s.reveal_span(40, 3));
        assert_eq!(
            s.offset(),
            35,
            "the span lands at the bottom, context above"
        );
        // A span above the view: reveal it with its context below.
        assert!(s.reveal_span(10, 3));
        assert_eq!(s.offset(), 10 - 2);
        // The same visible span again moves nothing.
        assert!(!s.reveal_span(10, 3));
        // A span as long as the viewport aligns its start to the top.
        assert!(s.reveal_span(30, 10));
        assert_eq!(s.offset(), 30);
        // No viewport, nothing happens.
        let mut empty = ScrollState::new(10);
        assert!(!empty.reveal_span(0, 1));
    }

    #[test]
    fn a_zero_context_keeps_the_frozen_minimal_reveal() {
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
    fn pointer_move_centers_exactly_without_context() {
        let mut s = ScrollState::new(50).with_context(1);
        s.set_viewport(10);
        s.pointer_move();
        s.ensure_visible(25);
        assert_eq!(s.offset(), 16, "an exact reveal keeps no context rows");
    }
}
