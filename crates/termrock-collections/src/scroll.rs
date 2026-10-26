use core::ops::Range;

use crate::response::Response;
use termrock_core::geometry::Headroom;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FollowCursor {
    #[default]
    OnMove,

    Always,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TrackClick {
    Jump,

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

    context: usize,

    exact: bool,
    follow_cursor: FollowCursor,

    followed: Option<(usize, usize)>,

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

    pub const fn with_context(mut self, rows: usize) -> Self {
        self.context = rows;
        self
    }

    pub const fn context(&self) -> usize {
        self.context
    }

    pub const fn with_follow_cursor(mut self, follow: FollowCursor) -> Self {
        self.follow_cursor = follow;
        self
    }

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

    pub fn pointer_move(&mut self) {
        self.exact = true;
    }

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
