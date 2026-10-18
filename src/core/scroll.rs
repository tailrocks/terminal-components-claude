use ratatui::layout::{Position, Rect};

use crate::core::hit::{Axis, Bar};
use crate::core::id::WidgetId;
use crate::core::response::Response;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    pub offset: usize,
    pub content_len: usize,
    pub viewport_len: usize,

    pub(crate) grab: Option<usize>,

    pub(crate) paging: bool,

    pub follow_cursor: FollowCursor,

    pub(crate) followed: Option<(usize, usize)>,

    pub context: usize,

    pub(crate) exact: bool,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ScrollState {
    pub fn new(content_len: usize) -> Self {
        Self {
            offset: 0,
            content_len,
            viewport_len: 0,
            grab: None,
            paging: false,
            follow_cursor: FollowCursor::OnMove,
            followed: None,
            context: 1,
            exact: false,
        }
    }

    pub fn with_context(mut self, rows: usize) -> Self {
        self.context = rows;
        self
    }

    pub fn with_follow_cursor(mut self, follow: FollowCursor) -> Self {
        self.follow_cursor = follow;
        self
    }

    pub fn max_offset(&self) -> usize {
        self.content_len.saturating_sub(self.viewport_len)
    }

    pub fn overflows(&self) -> bool {
        self.content_len > self.viewport_len && self.viewport_len > 0
    }

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

    pub fn pointer_move(&mut self) {
        self.exact = true;
    }

    pub fn ensure_visible(&mut self, index: usize) -> bool {
        let exact = std::mem::take(&mut self.exact);
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
        } else if index + ctx >= self.offset + self.viewport_len {
            self.offset = index + ctx + 1 - self.viewport_len;
        }
        self.clamp();
        self.offset != before
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

    pub fn visible_range(&self) -> std::ops::Range<usize> {
        let end = (self.offset + self.viewport_len).min(self.content_len);
        self.offset..end
    }

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

    pub fn offset_for_thumb_start(&self, start: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        let usable = track_len.saturating_sub(len).max(1);
        let start = start.min(usable);
        (start * self.max_offset() + usable / 2) / usable
    }

    pub fn offset_for_track_pos(&self, pos: usize, track_len: usize) -> usize {
        if !self.overflows() || track_len == 0 {
            return 0;
        }
        let (_, len) = self.thumb(track_len);
        self.offset_for_thumb_start(pos.saturating_sub(len / 2), track_len)
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
        self.scroll_to(self.offset_for_track_pos(pos, track_len))
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
        let target = self.offset_for_thumb_start(pos.saturating_sub(grab), track_len);
        self.scroll_to(target)
    }

    pub fn release_track(&mut self) {
        self.grab = None;
        self.paging = false;
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
        if pos < start {
            self.page_up()
        } else {
            self.page_down()
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FollowCursor {
    #[default]
    OnMove,

    Always,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    #[default]
    Auto,

    Always,

    Never,
}

impl ScrollbarVisibility {
    pub fn shows(self, scroll: &ScrollState) -> bool {
        match self {
            ScrollbarVisibility::Auto => scroll.overflows(),
            ScrollbarVisibility::Always => scroll.viewport_len > 0,
            ScrollbarVisibility::Never => false,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TrackClick {
    Jump,

    #[default]
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HScroll {
    pub enabled: bool,

    pub bar: ScrollbarVisibility,

    pub step: usize,

    pub track_click: TrackClick,
}

impl Default for HScroll {
    fn default() -> Self {
        Self {
            enabled: true,
            bar: ScrollbarVisibility::Auto,
            step: 4,
            track_click: TrackClick::Page,
        }
    }
}

impl HScroll {
    pub fn off() -> Self {
        Self {
            enabled: false,
            bar: ScrollbarVisibility::Never,
            ..Self::default()
        }
    }

    pub fn bar(mut self, bar: ScrollbarVisibility) -> Self {
        self.bar = bar;
        self
    }

    pub fn step(mut self, step: usize) -> Self {
        self.step = step.max(1);
        self
    }

    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.track_click = click;
        self
    }

    pub fn shows_bar(&self, scroll: &ScrollState) -> bool {
        self.enabled && self.bar.shows(scroll)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ScrollPane {
    pub v: ScrollState,

    pub h: ScrollState,

    pub h_cfg: HScroll,

    pub track_click: Option<TrackClick>,

    pub columns: bool,

    v_click: TrackClick,

    v_track: Rect,

    h_track: Rect,

    owner: Option<WidgetId>,

    held: Option<Axis>,

    col_widths: Vec<u16>,
    col_gap: u16,
    col_avail: u16,

    col_reveal: Option<(usize, usize)>,
}

impl std::ops::Deref for ScrollPane {
    type Target = ScrollState;
    fn deref(&self) -> &ScrollState {
        &self.v
    }
}

impl std::ops::DerefMut for ScrollPane {
    fn deref_mut(&mut self) -> &mut ScrollState {
        &mut self.v
    }
}

impl From<ScrollState> for ScrollPane {
    fn from(v: ScrollState) -> Self {
        Self {
            v,
            ..Self::default()
        }
    }
}

impl ScrollPane {
    pub fn new(content_len: usize) -> Self {
        ScrollState::new(content_len).into()
    }

    pub fn set_content(&mut self, len: usize) {
        self.v.set_content(len);
    }

    pub fn set_viewport(&mut self, len: usize) {
        self.v.set_viewport(len);
    }

    pub fn scroll_by(&mut self, delta: isize) -> bool {
        self.v.scroll_by(delta)
    }

    pub fn scroll_to(&mut self, offset: usize) -> bool {
        self.v.scroll_to(offset)
    }

    pub fn ensure_visible(&mut self, index: usize) -> bool {
        self.v.ensure_visible(index)
    }

    pub fn follow(&mut self, cursor: usize) -> bool {
        self.v.follow(cursor)
    }

    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.h_cfg = policy;
        self
    }

    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    pub fn columns(mut self, columns: bool) -> Self {
        self.columns = columns;
        self
    }

    pub fn begin(&mut self, app_click: TrackClick) {
        self.v_click = self.track_click.unwrap_or(app_click);
        self.v_track = Rect::ZERO;
        self.h_track = Rect::ZERO;
    }

    pub fn v_click(&self) -> TrackClick {
        self.v_click
    }

    pub fn v_track(&self) -> Rect {
        self.v_track
    }

    pub fn h_track(&self) -> Rect {
        self.h_track
    }

    pub fn set_track(&mut self, owner: WidgetId, axis: Axis, track: Rect) {
        self.owner = Some(owner);
        match axis {
            Axis::Vertical => self.v_track = track,
            Axis::Horizontal => self.h_track = track,
        }
    }

    pub fn clear_track(&mut self, axis: Axis) {
        match axis {
            Axis::Vertical => self.v_track = Rect::ZERO,
            Axis::Horizontal => self.h_track = Rect::ZERO,
        }
    }

    pub fn held(&self) -> Option<Axis> {
        self.held
    }

    pub fn axis(&mut self, axis: Axis) -> &mut ScrollState {
        match axis {
            Axis::Vertical => &mut self.v,
            Axis::Horizontal => &mut self.h,
        }
    }

    pub fn layout_columns(&mut self, widths: Vec<u16>, gap: u16, avail: u16) -> usize {
        self.col_widths = widths;
        self.col_gap = gap;
        self.col_avail = avail;
        if let Some((index, left)) = self.col_reveal {
            if self.h.offset == left {
                self.place_column(index);
                self.col_reveal = Some((index, self.h.offset));
            } else {
                self.col_reveal = None;
            }
        }
        self.columns_fit(self.h.offset)
    }

    pub fn reveal_column(&mut self, index: usize) -> bool {
        let moved = self.place_column(index);
        self.col_reveal = Some((index, self.h.offset));
        moved
    }

    fn columns_fit(&self, from: usize) -> usize {
        let mut used = 0u16;
        let mut fit = 0;
        for &w in self.col_widths.iter().skip(from) {
            let need = if fit == 0 {
                w
            } else {
                w.saturating_add(self.col_gap)
            };
            if used.saturating_add(need) > self.col_avail {
                break;
            }
            used += need;
            fit += 1;
        }
        fit
    }

    fn place_column(&mut self, index: usize) -> bool {
        let n = self.col_widths.len();
        if n == 0 || self.col_avail == 0 {
            return false;
        }
        let index = index.min(n - 1);
        let before = self.h.offset;
        let mut offset = before.min(n - 1);
        if index < offset {
            offset = index;
        }

        while offset < index && offset + self.columns_fit(offset) <= index {
            offset += 1;
        }

        while offset > 0 && offset - 1 + self.columns_fit(offset - 1) >= n {
            offset -= 1;
        }
        self.h.offset = offset;
        offset != before
    }

    pub fn h_step(&self) -> usize {
        if self.columns {
            1
        } else {
            self.h_cfg.step.max(1)
        }
    }

    pub fn h_scrolls(&self) -> bool {
        self.h_cfg.enabled && self.h.overflows()
    }

    pub fn wheel(&mut self, axis: Axis, delta: i32) -> Response<()> {
        let moved = match axis {
            Axis::Vertical => {
                if !self.v.overflows() {
                    return Response::ignored();
                }

                let page = self.v.viewport_len.max(1) as isize;
                self.v.scroll_by((delta as isize).clamp(-page, page))
            }
            Axis::Horizontal => {
                if !self.h_scrolls() {
                    return Response::ignored();
                }
                self.col_reveal = None;
                let step = self.h_step() as isize;
                self.h.scroll_by(delta.signum() as isize * step)
            }
        };
        changed(moved)
    }

    pub fn press(&mut self, pos: Position) -> Response<()> {
        let axis = if self.h_track.contains(pos) {
            Axis::Horizontal
        } else if self.v_track.contains(pos) {
            Axis::Vertical
        } else {
            self.held = None;
            return Response::ignored();
        };
        self.press_on(axis, pos)
    }

    pub fn press_on(&mut self, axis: Axis, pos: Position) -> Response<()> {
        let (track, click) = match axis {
            Axis::Vertical => (self.v_track, self.v_click),
            Axis::Horizontal => (self.h_track, self.h_cfg.track_click),
        };
        if track.is_empty() {
            self.held = None;
            return Response::ignored();
        }
        self.held = Some(axis);
        if axis == Axis::Horizontal {
            self.col_reveal = None;
        }
        let (cell, len) = track_cell(track, axis, pos);
        changed(self.axis(axis).press_track_with(cell, len, click))
    }

    pub fn drag(&mut self, pos: Position) -> Response<()> {
        let axis = self.held.unwrap_or(Axis::Vertical);
        self.drag_on(axis, pos)
    }

    pub fn drag_on(&mut self, axis: Axis, pos: Position) -> Response<()> {
        let track = match axis {
            Axis::Vertical => self.v_track,
            Axis::Horizontal => self.h_track,
        };
        if track.is_empty() {
            return Response::consumed();
        }
        self.held = Some(axis);
        if axis == Axis::Horizontal {
            self.col_reveal = None;
        }
        let (cell, len) = track_cell(track, axis, pos);
        changed(self.axis(axis).drag_track(cell, len))
    }

    pub fn release(&mut self) {
        self.v.release_track();
        self.h.release_track();
        self.held = None;
    }

    pub fn axis_of(&self, bar_id: WidgetId) -> Option<Axis> {
        let owner = self.owner?;
        if bar_id == owner.sub("scrollbar") {
            Some(Axis::Vertical)
        } else if bar_id == owner.sub("hscrollbar") {
            Some(Axis::Horizontal)
        } else {
            None
        }
    }

    pub fn bar_of(&self, bar_id: WidgetId) -> Option<Bar> {
        let axis = self.axis_of(bar_id)?;
        let track = match axis {
            Axis::Vertical => self.v_track,
            Axis::Horizontal => self.h_track,
        };
        Some(Bar {
            id: bar_id,
            owner: self.owner?,
            axis,
            track,
        })
    }

    pub fn bar(&mut self, bar: &Bar, pos: Position, pressed: bool) -> Response<()> {
        if pressed {
            self.press_on(bar.axis, pos)
        } else {
            self.drag_on(bar.axis, pos)
        }
    }
}

fn track_cell(track: Rect, axis: Axis, pos: Position) -> (usize, usize) {
    match axis {
        Axis::Vertical => (
            pos.y
                .saturating_sub(track.y)
                .min(track.height.saturating_sub(1)) as usize,
            track.height as usize,
        ),
        Axis::Horizontal => (
            pos.x
                .saturating_sub(track.x)
                .min(track.width.saturating_sub(1)) as usize,
            track.width as usize,
        ),
    }
}

fn changed(moved: bool) -> Response<()> {
    if moved {
        Response::changed()
    } else {
        Response::consumed()
    }
}

pub trait Scrollable {
    fn pane(&mut self) -> &mut ScrollPane;

    fn after_scroll(&mut self, _axis: Axis, _moved: bool) -> bool {
        false
    }

    fn wheel_first(&mut self, _axis: Axis, _delta: i32) -> Option<Response<()>> {
        None
    }

    fn wheel(&mut self, axis: Axis, delta: i32) -> Response<()> {
        if let Some(r) = self.wheel_first(axis, delta) {
            return r;
        }
        let r = self.pane().wheel(axis, delta);
        settle(self, axis, r)
    }

    fn bar_press(&mut self, pos: Position) -> Response<()> {
        let r = self.pane().press(pos);
        match self.pane().held() {
            Some(axis) => settle(self, axis, r),
            None => r,
        }
    }

    fn bar_drag(&mut self, pos: Position) -> Response<()> {
        let axis = self.pane().held().unwrap_or(Axis::Vertical);
        let r = self.pane().drag_on(axis, pos);
        settle(self, axis, r)
    }

    fn bar_input(&mut self, bar: &Bar, pos: Position, pressed: bool) -> Response<()> {
        let r = self.pane().bar(bar, pos, pressed);
        settle(self, bar.axis, r)
    }
}

fn settle<S: Scrollable + ?Sized>(w: &mut S, axis: Axis, r: Response<()>) -> Response<()> {
    if w.after_scroll(axis, r.is_changed()) {
        r.repaint()
    } else {
        r
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Outcome;

    fn pane() -> ScrollPane {
        let mut p = ScrollPane::new(100);
        p.set_viewport(10);
        p.h = ScrollState::new(40);
        p.h.set_viewport(10);
        p.begin(TrackClick::Page);
        let owner = WidgetId::of("w");
        p.set_track(owner, Axis::Vertical, Rect::new(10, 0, 1, 10));
        p.set_track(owner, Axis::Horizontal, Rect::new(0, 10, 10, 1));
        p
    }

    #[test]
    fn a_pane_derefs_to_its_vertical_state() {
        let mut p = ScrollPane::new(50);
        p.set_viewport(10);
        assert!(p.jump_end());
        assert_eq!(p.offset, 40);
        assert_eq!(p.v.offset, 40);
    }

    #[test]
    fn a_notch_never_moves_further_than_the_view() {
        let mut p = ScrollPane::new(6);
        p.set_viewport(2);
        assert!(p.wheel(Axis::Vertical, 3).is_changed());
        assert_eq!(p.offset, 2);
        assert!(p.wheel(Axis::Vertical, -3).is_changed());
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn the_wheel_moves_rows_vertically_and_one_step_per_notch_sideways() {
        let mut p = pane();
        assert!(p.wheel(Axis::Vertical, 3).is_changed());
        assert_eq!(p.offset, 3);

        assert!(p.wheel(Axis::Horizontal, 3).is_changed());
        assert_eq!(p.h.offset, 4, "HScroll::step cells");
        let mut cols = pane().columns(true);
        assert!(cols.wheel(Axis::Horizontal, 3).is_changed());
        assert_eq!(cols.h.offset, 1, "a column per notch");

        let r = p.wheel(Axis::Vertical, -10);
        assert!(r.is_changed());
        let r = p.wheel(Axis::Vertical, -1);
        assert!(r.is_consumed() && !r.is_changed());

        let mut fits = pane();
        fits.h.set_viewport(40);
        assert!(!fits.wheel(Axis::Horizontal, 1).is_consumed());
        let mut off = pane().horizontal(HScroll::off());
        assert!(!off.wheel(Axis::Horizontal, 1).is_consumed());
    }

    #[test]
    fn a_press_picks_its_bar_and_the_drag_stays_on_it() {
        let mut p = pane();

        assert!(p.press(Position::new(10, 9)).is_changed());
        assert_eq!((p.offset, p.held()), (10, Some(Axis::Vertical)));
        p.release();
        assert_eq!(p.held(), None);

        assert!(!p.press(Position::new(0, 10)).is_changed());
        assert_eq!(p.held(), Some(Axis::Horizontal));
        assert!(p.drag(Position::new(9, 10)).is_changed());
        assert!(p.h.at_end());
        assert_eq!(p.offset, 10, "the vertical view did not move");

        assert!(!p.press(Position::new(3, 3)).is_consumed());
        assert_eq!(p.held(), None);
    }

    #[test]
    fn jump_stays_an_override_of_the_paging_default() {
        let mut p = pane().track_click(TrackClick::Jump);
        p.begin(TrackClick::Page);
        assert_eq!(p.v_click(), TrackClick::Jump);
        p.set_track(WidgetId::of("w"), Axis::Vertical, Rect::new(10, 0, 1, 10));
        assert!(p.press(Position::new(10, 9)).is_changed());
        assert!(p.at_end(), "Jump: the thumb lands under the pointer");
        let mut app = pane();
        app.begin(TrackClick::Jump);
        assert_eq!(app.v_click(), TrackClick::Jump, "None follows the app");
    }

    #[test]
    fn begin_forgets_the_tracks_so_an_undrawn_bar_cannot_be_pressed() {
        let mut p = pane();
        p.begin(TrackClick::Page);
        assert_eq!(p.v_track(), Rect::ZERO);
        assert!(!p.press(Position::new(10, 9)).is_consumed());
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn a_grabbed_bar_names_its_axis() {
        let mut p = pane();
        let owner = WidgetId::of("w");
        assert_eq!(p.axis_of(owner.sub("scrollbar")), Some(Axis::Vertical));
        assert_eq!(p.axis_of(owner.sub("hscrollbar")), Some(Axis::Horizontal));
        assert_eq!(p.axis_of(WidgetId::of("x")), None);
        let bar = Bar {
            id: owner.sub("hscrollbar"),
            owner,
            axis: Axis::Horizontal,
            track: p.h_track(),
        };
        assert!(p.bar(&bar, Position::new(9, 10), true).is_changed());
        assert_eq!(p.h.offset, 10, "one page sideways");
        assert_eq!(p.offset, 0);
    }

    struct Log {
        pane: ScrollPane,
        follow: bool,
        first: bool,
    }

    impl Scrollable for Log {
        fn pane(&mut self) -> &mut ScrollPane {
            &mut self.pane
        }
        fn after_scroll(&mut self, axis: Axis, _moved: bool) -> bool {
            let was = self.follow;
            if axis == Axis::Vertical {
                self.follow = self.pane.at_end();
            }
            was != self.follow
        }
        fn wheel_first(&mut self, _axis: Axis, _delta: i32) -> Option<Response<()>> {
            self.first.then(Response::consumed)
        }
    }

    #[test]
    fn scrollable_folds_side_effects_into_the_response() {
        let mut log = Log {
            pane: pane(),
            follow: false,
            first: false,
        };
        log.pane.jump_end();

        assert_eq!(log.wheel(Axis::Vertical, 1).outcome(), Outcome::Changed);
        assert!(log.follow);
        assert_eq!(
            log.wheel(Axis::Vertical, 1).outcome(),
            Outcome::Consumed,
            "nothing changed"
        );
        assert_eq!(log.wheel(Axis::Vertical, -1).outcome(), Outcome::Changed);
        assert!(!log.follow);

        log.first = true;
        assert_eq!(log.wheel(Axis::Vertical, -5).outcome(), Outcome::Consumed);
        assert_eq!(log.pane.offset, 89);

        log.pane.scroll_to(80);
        assert_eq!(
            log.bar_press(Position::new(10, 9))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Changed
        );
        assert!(log.follow, "a page to the end follows again");
        assert_eq!(
            log.bar_press(Position::new(3, 3))
                .outcome()
                .or(Outcome::Consumed),
            Outcome::Consumed,
            "routed to the bar, so consumed"
        );
        let id = WidgetId::of("w").sub("hscrollbar");
        assert_eq!(
            match log.pane().bar_of(id) {
                Some(bar) => log.bar_input(&bar, Position::new(9, 10), false).outcome(),
                None => Outcome::Consumed,
            },
            Outcome::Changed
        );
        assert!(log.pane.h.at_end());
    }

    #[test]
    fn visibility_policy_decides_when_a_bar_draws() {
        let mut s = ScrollState::new(10);
        s.set_viewport(20);
        assert!(!ScrollbarVisibility::Auto.shows(&s), "fits: auto hides");
        assert!(ScrollbarVisibility::Always.shows(&s));
        s.set_viewport(4);
        assert!(ScrollbarVisibility::Auto.shows(&s));
        assert!(!ScrollbarVisibility::Never.shows(&s));
        assert!(!HScroll::off().shows_bar(&s), "disabled never draws");
        assert!(HScroll::default().shows_bar(&s));
    }

    #[test]
    fn a_paging_track_press_moves_one_viewport_toward_the_pointer() {
        let mut s = ScrollState::new(100);
        s.set_viewport(10);

        assert_eq!(TrackClick::default(), TrackClick::Page, "macOS, Windows");
        assert!(s.press_track_with(9, 10, TrackClick::Page));
        assert_eq!(s.offset, 10, "one page right, not a jump to the end");
        assert!(s.press_track_with(0, 10, TrackClick::Page));
        assert_eq!(s.offset, 0);
        let (start, _) = s.thumb(10);
        assert!(!s.press_track_with(start, 10, TrackClick::Page));
        assert_eq!(s.grab, Some(0), "the thumb is grabbed, not paged");
    }

    #[test]
    fn motion_after_a_paging_press_never_jumps_the_thumb() {
        let mut s = ScrollState::new(100);
        s.set_viewport(10);
        assert!(s.press_track_with(9, 10, TrackClick::Page));
        assert_eq!(s.offset, 10);

        assert!(!s.drag_track(8, 10));
        assert_eq!(s.offset, 10);

        s.release_track();
        assert!(s.press_track_with(9, 10, TrackClick::Jump));
        assert!(s.drag_track(0, 10));
        assert_eq!(s.offset, 0);
    }

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
        assert!(
            !s.ensure_visible(s.offset + 2),
            "visible rows clear of the context need no scroll"
        );
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

        assert!(!s.press_track(7, track));
        assert_eq!(s.offset, 0);

        assert!(s.drag_track(8, track));
        assert_eq!(s.thumb(track).0, 1);
        assert!(s.offset > 0 && s.offset < 8, "{}", s.offset);

        assert!(s.drag_track(track - 1, track));
        assert!(s.at_end());
        assert_eq!(s.thumb(track).0, track - len);

        assert!(s.drag_track(0, track));
        assert_eq!(s.offset, 0);

        s.release_track();
        assert!(s.press_track(20, track));
        let (start, len) = s.thumb(track);
        assert!(
            (start..start + len).contains(&20),
            "{start}..{}",
            start + len
        );

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
    fn context_rows_stay_visible_around_a_row_brought_into_view() {
        let mut s = ScrollState::new(50).with_context(1);
        s.set_viewport(10);
        assert!(!s.ensure_visible(8), "inside the view, clear of the edge");
        assert!(s.ensure_visible(9), "the last visible row scrolls one on");
        assert_eq!(s.offset, 1, "row 10 shows below row 9");
        assert!(s.ensure_visible(25));
        assert_eq!(s.offset, 17);
        assert!(s.ensure_visible(17));
        assert_eq!(s.offset, 16, "one row above stays in view");
        assert!(s.ensure_visible(0));
        assert_eq!(s.offset, 0, "the edges of the content clamp");
        s.ensure_visible(49);
        assert!(s.at_end());

        let mut short = ScrollState::new(50).with_context(1);
        short.set_viewport(2);
        short.ensure_visible(5);
        assert_eq!(short.offset, 4);
    }

    #[test]
    fn the_default_keeps_one_row_of_context_around_a_row_brought_into_view() {
        let mut s = ScrollState::new(50);
        assert_eq!(s.context, 1);
        s.set_viewport(10);
        assert!(s.ensure_visible(9), "the last visible row scrolls one on");
        assert_eq!(s.offset, 1);
        assert_eq!(ScrollState::default().context, 1);
    }

    #[test]
    fn a_pointer_move_reveals_its_row_without_context_once() {
        let mut s = ScrollState::new(50);
        s.set_viewport(10);
        s.pointer_move();
        assert!(!s.ensure_visible(9), "a clicked edge row stays put");
        assert_eq!(s.offset, 0);
        s.pointer_move();
        assert!(s.ensure_visible(10), "a row past the edge comes in exactly");
        assert_eq!(s.offset, 1);

        assert!(s.ensure_visible(10));
        assert_eq!(s.offset, 2);
    }

    #[test]
    fn ensure_visible_moves_minimally() {
        let mut s = ScrollState::new(50).with_context(0);
        s.set_viewport(10);
        s.ensure_visible(25);
        assert_eq!(s.offset, 16);
        s.ensure_visible(5);
        assert_eq!(s.offset, 5);
        s.ensure_visible(7);
        assert_eq!(s.offset, 5);
    }

    #[test]
    fn follow_on_move_leaves_a_wheeled_view_alone_until_the_cursor_moves() {
        let mut s = ScrollState::new(50);
        s.set_viewport(10);
        s.follow(3);
        assert_eq!(s.offset, 0);

        s.scroll_by(20);
        s.follow(3);
        assert_eq!(s.offset, 20, "a repaint must not snap back");

        s.follow(4);
        assert_eq!(s.offset, 3);

        s.scroll_by(20);
        s.set_viewport(8);
        s.follow(4);
        assert_eq!(s.offset, 3);
    }

    #[test]
    fn follow_always_pins_the_cursor() {
        let mut s = ScrollState::new(50).with_follow_cursor(FollowCursor::Always);
        s.set_viewport(10);
        s.follow(3);
        s.scroll_by(20);
        s.follow(3);
        assert_eq!(s.offset, 2, "pinned with the default row of context");
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
