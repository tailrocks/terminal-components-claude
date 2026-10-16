//! Scrolling as a first-class behaviour.
//!
//! [`ScrollState`] is a pure model: content length, viewport length and
//! offset. It knows nothing about rendering; the scrollbar widget derives
//! geometry from it. [`ScrollPane`] puts both axes, the policies and the
//! drawn tracks together, and [`Scrollable`] gives every widget the same
//! wheel and scrollbar handling.

use ratatui::layout::{Position, Rect};

use crate::core::event::Outcome;
use crate::core::hit::{Axis, Bar};
use crate::core::id::WidgetId;
use crate::core::response::Response;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    pub offset: usize,
    pub content_len: usize,
    pub viewport_len: usize,
    /// Rows between the thumb's top and the pointer while the thumb is
    /// held, so a drag moves the thumb with the pointer instead of
    /// centring it under the pointer on the first motion.
    pub(crate) grab: Option<usize>,
    /// The held press pages the track ([`TrackClick::Page`]): motion while
    /// it is held never jumps the thumb under the pointer.
    pub(crate) paging: bool,
    /// When [`follow`](Self::follow) brings the cursor back into view.
    pub follow_cursor: FollowCursor,
    /// The cursor and viewport length of the last [`follow`](Self::follow),
    /// so `OnMove` can tell a cursor move from a repaint.
    pub(crate) followed: Option<(usize, usize)>,
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
        }
    }

    /// Builder for [`follow_cursor`](Self::follow_cursor).
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

    /// Keep a list cursor in view the way the list's
    /// [`follow_cursor`](Self::follow_cursor) policy says. Call it from
    /// render: under the default `OnMove` the viewport moves only when the
    /// cursor (or the viewport size) changed since the last call, so the
    /// wheel and the scrollbar can move the view away from the cursor
    /// without the next frame snapping it back. Returns whether the offset
    /// changed.
    pub fn follow(&mut self, cursor: usize) -> bool {
        let key = (cursor, self.viewport_len);
        if self.follow_cursor == FollowCursor::OnMove && self.followed == Some(key) {
            return false;
        }
        self.followed = Some(key);
        self.ensure_visible(cursor)
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

    /// The pointer moved to row `pos` while held: the thumb follows it,
    /// keeping the grabbed row under the pointer. A drag without a press
    /// behaves like a press. Returns whether the offset changed.
    pub fn drag_track(&mut self, pos: usize, track_len: usize) -> bool {
        if !self.overflows() || track_len == 0 {
            return false;
        }
        let Some(grab) = self.grab else {
            // a held page press keeps paging (on the app tick); motion
            // never turns it into a jump
            if self.paging {
                return false;
            }
            return self.press_track(pos, track_len);
        };
        let target = self.offset_for_thumb_start(pos.saturating_sub(grab), track_len);
        self.scroll_to(target)
    }

    /// The pointer was released: the next press starts a new grab.
    pub fn release_track(&mut self) {
        self.grab = None;
        self.paging = false;
    }

    /// A press on the track under `click`. Off the thumb, `Jump` moves the
    /// thumb under the pointer and `Page` moves one viewport toward the
    /// pointer; on the thumb this only grabs it. Returns whether the offset
    /// changed.
    pub fn press_track_with(&mut self, pos: usize, track_len: usize, click: TrackClick) -> bool {
        if click == TrackClick::Jump || !self.overflows() || track_len == 0 {
            return self.press_track(pos, track_len);
        }
        let (start, len) = self.thumb(track_len);
        if (start..start + len).contains(&pos) {
            self.grab = Some(pos - start);
            return false;
        }
        // paging never grabs: motion while held pages no further and never
        // jumps; the app tick repeats the page toward the pointer
        self.grab = None;
        self.paging = true;
        if pos < start {
            self.page_up()
        } else {
            self.page_down()
        }
    }
}

/// When a list's viewport follows its cursor
/// ([`ScrollState::follow`]).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum FollowCursor {
    /// Only after the cursor moves. Scrolling by wheel or scrollbar moves
    /// the view, never the cursor, and the view stays where the user left
    /// it until the cursor moves again (macOS and WAI-ARIA listbox: the
    /// selection is scrolled into view when it changes, not on every
    /// repaint).
    #[default]
    OnMove,
    /// On every call: the cursor is pinned in view and the view can only
    /// leave it by moving the cursor.
    Always,
}

/// When a scrollbar is drawn.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// Only while the content overflows (the macOS default).
    #[default]
    Auto,
    /// Always, with a full-length thumb when the content fits; the bar's
    /// row or column is reserved either way.
    Always,
    /// Never; the content still scrolls by wheel and keyboard.
    Never,
}

impl ScrollbarVisibility {
    /// Whether a bar for `scroll` is drawn under this policy.
    pub fn shows(self, scroll: &ScrollState) -> bool {
        match self {
            ScrollbarVisibility::Auto => scroll.overflows(),
            ScrollbarVisibility::Always => scroll.viewport_len > 0,
            ScrollbarVisibility::Never => false,
        }
    }
}

/// What a press on the bare scrollbar track (off the thumb) does.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TrackClick {
    /// The thumb jumps under the pointer (macOS "Jump to the spot that's
    /// clicked", GTK's primary click).
    Jump,
    /// The view moves one page toward the pointer, repeating while the
    /// button is held until the thumb reaches it
    /// ([`PointerState::track_repeat_due`](crate::core::pointer::PointerState::track_repeat_due)).
    /// The default: macOS System Settings ("Click in the scroll bar to:
    /// Jump to the next page") and Windows both page.
    #[default]
    Page,
}

/// Horizontal scrolling policy for a widget whose content can be wider
/// than its area. [`RenderCtx::hscroll`](crate::ui::ctx::RenderCtx::hscroll)
/// holds the app-wide default; a widget's `horizontal` field overrides it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HScroll {
    /// Content wider than the area scrolls sideways. Off, the widget keeps
    /// its cut-off behaviour (truncation marks only) and draws no bar.
    pub enabled: bool,
    /// When the horizontal scrollbar is drawn.
    pub bar: ScrollbarVisibility,
    /// Cells per wheel notch or arrow press for free text. Column-based
    /// widgets (grids, tables) always move one column.
    pub step: usize,
    /// What a press on the bare track does.
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
    /// Horizontal scrolling switched off: cut-off text only, no bar.
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

    /// Whether the bar shows for `scroll` (never while disabled).
    pub fn shows_bar(&self, scroll: &ScrollState) -> bool {
        self.enabled && self.bar.shows(scroll)
    }
}

/// A scrolled region: the vertical and horizontal [`ScrollState`], the
/// horizontal policy, the track-click policy and the bars as last drawn.
///
/// Every scrolling widget embeds one (as its `scroll` field) and implements
/// [`Scrollable`], so the wheel, a press on either track, a thumb drag and
/// the paging repeat behave the same in every widget. The pane dereferences
/// to its vertical state, so `widget.scroll.offset` and
/// `widget.scroll.jump_end()` read as they did on a bare [`ScrollState`].
///
/// The bars are drawn by `ScrollPane::draw_v`, `draw_h` and `draw_divider`
/// (in [`crate::widgets::scrollbar`]), which remember the tracks they drew:
/// a press maps through the track the user saw, and a bar that was not
/// drawn this frame has nothing to press.
#[derive(Debug, Default, Clone)]
pub struct ScrollPane {
    /// The vertical state (rows).
    pub v: ScrollState,
    /// The horizontal state (cells for free text, columns for tables and
    /// grids; see [`ScrollPane::columns`]).
    pub h: ScrollState,
    /// Horizontal policy: on or off, bar visibility, step, track click.
    pub h_cfg: HScroll,
    /// What a press on the bare vertical track does; `None` follows the
    /// app-wide [`RenderCtx::track_click`](crate::ui::ctx::RenderCtx::track_click).
    pub track_click: Option<TrackClick>,
    /// The horizontal unit is a column: a wheel notch moves one column
    /// whatever [`HScroll::step`] says (tables and grids).
    pub columns: bool,
    /// The resolved vertical policy, set by [`Self::begin`].
    v_click: TrackClick,
    /// The vertical track as last drawn (empty when no bar was drawn).
    v_track: Rect,
    /// The horizontal track as last drawn.
    h_track: Rect,
    /// The widget the bars were drawn for (their hit ids derive from it).
    owner: Option<WidgetId>,
    /// The axis of the bar the held press started on.
    held: Option<Axis>,
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
    /// A pane over `content_len` rows.
    pub fn new(content_len: usize) -> Self {
        ScrollState::new(content_len).into()
    }

    /// Builder for [`Self::h_cfg`].
    pub fn horizontal(mut self, policy: HScroll) -> Self {
        self.h_cfg = policy;
        self
    }

    /// Builder for [`Self::track_click`].
    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.track_click = Some(click);
        self
    }

    /// Builder for [`Self::columns`].
    pub fn columns(mut self, columns: bool) -> Self {
        self.columns = columns;
        self
    }

    /// Start a frame: resolve the vertical track-click policy against the
    /// app-wide default and forget last frame's tracks (a bar that is not
    /// drawn again cannot be pressed).
    pub fn begin(&mut self, app_click: TrackClick) {
        self.v_click = self.track_click.unwrap_or(app_click);
        self.v_track = Rect::ZERO;
        self.h_track = Rect::ZERO;
    }

    /// The vertical track-click policy in force this frame.
    pub fn v_click(&self) -> TrackClick {
        self.v_click
    }

    /// The vertical track as last drawn, empty when no bar was drawn.
    pub fn v_track(&self) -> Rect {
        self.v_track
    }

    /// The horizontal track as last drawn, empty when no bar was drawn.
    pub fn h_track(&self) -> Rect {
        self.h_track
    }

    /// Record a track drawn for `owner` on `axis` (the `draw_*` methods do
    /// this; a widget that draws a bar by hand calls it itself).
    pub fn set_track(&mut self, owner: WidgetId, axis: Axis, track: Rect) {
        self.owner = Some(owner);
        match axis {
            Axis::Vertical => self.v_track = track,
            Axis::Horizontal => self.h_track = track,
        }
    }

    /// Forget the track drawn on `axis` (its bar is gone this frame).
    pub fn clear_track(&mut self, axis: Axis) {
        match axis {
            Axis::Vertical => self.v_track = Rect::ZERO,
            Axis::Horizontal => self.h_track = Rect::ZERO,
        }
    }

    /// The axis of the bar a held press started on.
    pub fn held(&self) -> Option<Axis> {
        self.held
    }

    /// The state on `axis`.
    pub fn axis(&mut self, axis: Axis) -> &mut ScrollState {
        match axis {
            Axis::Vertical => &mut self.v,
            Axis::Horizontal => &mut self.h,
        }
    }

    /// Cells (or columns) one horizontal wheel notch or arrow press moves.
    pub fn h_step(&self) -> usize {
        if self.columns {
            1
        } else {
            self.h_cfg.step.max(1)
        }
    }

    /// Whether horizontal scrolling can move at all: it is enabled and the
    /// content is wider than the view.
    pub fn h_scrolls(&self) -> bool {
        self.h_cfg.enabled && self.h.overflows()
    }

    /// A wheel notch of `delta` on `axis`. Vertical moves `delta` rows.
    /// Horizontal moves one [`Self::h_step`] per notch whatever the
    /// notch's size, and is ignored while horizontal scrolling is off or
    /// the content fits, so the notch can reach an owner that can still
    /// move. At a boundary the notch is consumed without a repaint.
    pub fn wheel(&mut self, axis: Axis, delta: i32) -> Response<()> {
        let moved = match axis {
            Axis::Vertical => self.v.scroll_by(delta as isize),
            Axis::Horizontal => {
                if !self.h_scrolls() {
                    return Response::ignored();
                }
                let step = self.h_step() as isize;
                self.h.scroll_by(delta.signum() as isize * step)
            }
        };
        changed(moved)
    }

    /// The pointer went down at `pos`: on the horizontal track or the
    /// vertical one (they never overlap). On the thumb this grabs it; on
    /// the bare track it acts per the axis's [`TrackClick`]. A press on
    /// neither track holds nothing and is ignored.
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

    /// [`Self::press`] on the bar of `axis`, wherever `pos` is.
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
        let (cell, len) = track_cell(track, axis, pos);
        changed(self.axis(axis).press_track_with(cell, len, click))
    }

    /// The pointer moved to `pos` while held after [`Self::press`]: the
    /// thumb of the held bar follows, keeping the grabbed cell under the
    /// pointer. With no held bar the vertical one takes the drag.
    pub fn drag(&mut self, pos: Position) -> Response<()> {
        let axis = self.held.unwrap_or(Axis::Vertical);
        self.drag_on(axis, pos)
    }

    /// [`Self::drag`] on the bar of `axis` (a drag that names its bar).
    pub fn drag_on(&mut self, axis: Axis, pos: Position) -> Response<()> {
        let track = match axis {
            Axis::Vertical => self.v_track,
            Axis::Horizontal => self.h_track,
        };
        if track.is_empty() {
            return Response::consumed();
        }
        self.held = Some(axis);
        let (cell, len) = track_cell(track, axis, pos);
        changed(self.axis(axis).drag_track(cell, len))
    }

    /// The button was released: the next press starts a new grab.
    pub fn release(&mut self) {
        self.v.release_track();
        self.h.release_track();
        self.held = None;
    }

    /// The axis of the bar with hit id `bar_id`, when it is one of this
    /// pane's bars.
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

    /// A press (`pressed`) or drag on a [`Bar`] the pointer layer grabbed:
    /// the bar names its axis, so nothing is inferred.
    pub fn bar(&mut self, bar: &Bar, pos: Position, pressed: bool) -> Response<()> {
        if pressed {
            self.press_on(bar.axis, pos)
        } else {
            self.drag_on(bar.axis, pos)
        }
    }
}

/// The cell of `track` under `pos` on `axis`, clamped to the track (a
/// pointer past either end still drags to that end), and the track length.
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

/// `changed` when the view moved, else consumed without a repaint.
fn changed(moved: bool) -> Response<()> {
    if moved {
        Response::changed()
    } else {
        Response::consumed()
    }
}

/// A widget that scrolls through a [`ScrollPane`]. The wheel, the bars and
/// their drags come from the provided methods, once for every widget; a
/// widget only hands out its pane and, when scrolling has side effects
/// (a log's tail follow, a reading position), reacts in
/// [`Self::after_scroll`].
///
/// The `on_*` methods returning [`Outcome`] are adapters for routers not
/// yet on `Response`; they go with the per-widget handlers (S6, S15).
pub trait Scrollable {
    /// The widget's pane.
    fn pane(&mut self) -> &mut ScrollPane;

    /// The view on `axis` may have moved by wheel or bar (`moved` says
    /// whether it did). Returns whether anything else changed that needs a
    /// repaint (a tail follow switching off, for instance).
    fn after_scroll(&mut self, _axis: Axis, _moved: bool) -> bool {
        false
    }

    /// A wheel notch the widget handles before its pane (a revealed label
    /// that reads on, an open popup that takes the wheel). `None` lets the
    /// pane scroll.
    fn wheel_first(&mut self, _axis: Axis, _delta: i32) -> Option<Response<()>> {
        None
    }

    /// A wheel notch of `delta` on `axis` ([`ScrollPane::wheel`]).
    fn wheel(&mut self, axis: Axis, delta: i32) -> Response<()> {
        if let Some(r) = self.wheel_first(axis, delta) {
            return r;
        }
        let r = self.pane().wheel(axis, delta);
        settle(self, axis, r)
    }

    /// A press on either of the widget's bars ([`ScrollPane::press`]).
    fn bar_press(&mut self, pos: Position) -> Response<()> {
        let r = self.pane().press(pos);
        match self.pane().held() {
            Some(axis) => settle(self, axis, r),
            None => r,
        }
    }

    /// A drag along the held bar ([`ScrollPane::drag`]).
    fn bar_drag(&mut self, pos: Position) -> Response<()> {
        let axis = self.pane().held().unwrap_or(Axis::Vertical);
        let r = self.pane().drag_on(axis, pos);
        settle(self, axis, r)
    }

    /// A press or drag on a [`Bar`] grabbed by the pointer layer
    /// ([`ScrollPane::bar`]): how a router reaches any widget's bars.
    fn bar_input(&mut self, bar: &Bar, pos: Position, pressed: bool) -> Response<()> {
        let r = self.pane().bar(bar, pos, pressed);
        settle(self, bar.axis, r)
    }

    /// Adapter: [`Self::wheel`] vertically.
    fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.wheel(Axis::Vertical, delta).outcome()
    }

    /// Adapter: [`Self::wheel`] horizontally.
    fn on_wheel_h(&mut self, delta: i32) -> Outcome {
        self.wheel(Axis::Horizontal, delta).outcome()
    }

    /// Adapter: [`Self::bar_press`]. The router sent the press to one of
    /// this widget's bars, so a press beside both tracks is still consumed.
    fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        match self.bar_press(pos).outcome() {
            Outcome::Ignored => Outcome::Consumed,
            o => o,
        }
    }

    /// Adapter: [`Self::bar_drag`].
    fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        self.bar_drag(pos).outcome()
    }

    /// Adapter: a drag while `pressed` (one of this widget's bar ids) is
    /// held. The axis follows the id, so a drag works whether or not the
    /// press was routed first.
    fn on_bar_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        let axis = self.pane().axis_of(pressed).unwrap_or(Axis::Vertical);
        let r = self.pane().drag_on(axis, pos);
        settle(self, axis, r).outcome()
    }
}

/// Run the widget's [`Scrollable::after_scroll`] and fold what it changed
/// into `r`.
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

    /// A pane over 100 rows in a 10-row view and 40 cells in a 10-cell
    /// view, with both tracks drawn: vertical at column 10, rows 0..10;
    /// horizontal on row 10, columns 0..10.
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
    fn the_wheel_moves_rows_vertically_and_one_step_per_notch_sideways() {
        let mut p = pane();
        assert!(p.wheel(Axis::Vertical, 3).is_changed());
        assert_eq!(p.offset, 3);
        // a notch of any size moves one step sideways
        assert!(p.wheel(Axis::Horizontal, 3).is_changed());
        assert_eq!(p.h.offset, 4, "HScroll::step cells");
        let mut cols = pane().columns(true);
        assert!(cols.wheel(Axis::Horizontal, 3).is_changed());
        assert_eq!(cols.h.offset, 1, "a column per notch");
        // the boundary consumes the notch without a repaint
        let r = p.wheel(Axis::Vertical, -10);
        assert!(r.is_changed());
        let r = p.wheel(Axis::Vertical, -1);
        assert!(r.is_consumed() && !r.is_changed());
        // nothing wider than the view, or sideways off: the notch passes on
        let mut fits = pane();
        fits.h.set_viewport(40);
        assert!(!fits.wheel(Axis::Horizontal, 1).is_consumed());
        let mut off = pane().horizontal(HScroll::off());
        assert!(!off.wheel(Axis::Horizontal, 1).is_consumed());
    }

    #[test]
    fn a_press_picks_its_bar_and_the_drag_stays_on_it() {
        let mut p = pane();
        // the bare vertical track pages (the default)
        assert!(p.press(Position::new(10, 9)).is_changed());
        assert_eq!((p.offset, p.held()), (10, Some(Axis::Vertical)));
        p.release();
        assert_eq!(p.held(), None);
        // the horizontal thumb is grabbed and dragged; the drag stays on
        // the horizontal bar even over the vertical track
        assert!(!p.press(Position::new(0, 10)).is_changed());
        assert_eq!(p.held(), Some(Axis::Horizontal));
        assert!(p.drag(Position::new(9, 10)).is_changed());
        assert!(p.h.at_end());
        assert_eq!(p.offset, 10, "the vertical view did not move");
        // a press on neither bar holds nothing
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

    /// A widget whose pane scrolls and whose tail follow is a side effect.
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
        // at the end the wheel cannot move, but the follow switching on is
        // a change
        assert_eq!(log.on_wheel(1), Outcome::Changed);
        assert!(log.follow);
        assert_eq!(log.on_wheel(1), Outcome::Consumed, "nothing changed");
        assert_eq!(log.on_wheel(-1), Outcome::Changed);
        assert!(!log.follow);
        // the widget's own wheel handling goes first
        log.first = true;
        assert_eq!(log.on_wheel(-5), Outcome::Consumed);
        assert_eq!(log.pane.offset, 89);
        // the bar adapters reach the same pane
        log.pane.scroll_to(80);
        assert_eq!(log.on_scrollbar(Position::new(10, 9)), Outcome::Changed);
        assert!(log.follow, "a page to the end follows again");
        assert_eq!(
            log.on_scrollbar(Position::new(3, 3)),
            Outcome::Consumed,
            "routed to the bar, so consumed"
        );
        let id = WidgetId::of("w").sub("hscrollbar");
        assert_eq!(log.on_bar_drag(id, Position::new(9, 10)), Outcome::Changed);
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
        // the thumb is the first cell of a 10-cell track
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
        // the pointer slides along the held track: no jump under it
        assert!(!s.drag_track(8, 10));
        assert_eq!(s.offset, 10);
        // a fresh press under Jump drags as before
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
    fn follow_on_move_leaves_a_wheeled_view_alone_until_the_cursor_moves() {
        let mut s = ScrollState::new(50);
        s.set_viewport(10);
        s.follow(3);
        assert_eq!(s.offset, 0);
        // the wheel moves the view away from the cursor
        s.scroll_by(20);
        s.follow(3);
        assert_eq!(s.offset, 20, "a repaint must not snap back");
        // a cursor move brings it back into view
        s.follow(4);
        assert_eq!(s.offset, 4);
        // so does a viewport resize
        s.scroll_by(20);
        s.set_viewport(8);
        s.follow(4);
        assert_eq!(s.offset, 4);
    }

    #[test]
    fn follow_always_pins_the_cursor() {
        let mut s = ScrollState::new(50).with_follow_cursor(FollowCursor::Always);
        s.set_viewport(10);
        s.follow(3);
        s.scroll_by(20);
        s.follow(3);
        assert_eq!(s.offset, 3);
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
