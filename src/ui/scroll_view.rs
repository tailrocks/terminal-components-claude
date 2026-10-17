//! A two-axis scroll container: content is laid out at its natural size
//! ([`Measure`](crate::ui::measure::Measure)) and, when that is larger than
//! the room the view has, the view shows a window of it with scrollbars on
//! the axes that overflow, so no content is ever squeezed out of reach.
//!
//! Standards: macOS scroll views (scrollers appear only while the content
//! overflows, the content keeps its size and scrolls instead of being cut),
//! WCAG 2.2 1.4.10 Reflow (content is reachable in both directions without
//! loss) and 2.4.11 Focus Not Obscured (a control that takes keyboard focus
//! is brought fully into view, with one row or column of context).
//!
//! How content gets there:
//!
//! - **Pointer**: the vertical wheel scrolls rows, Shift+wheel and the
//!   horizontal wheel scroll columns, and both dotted-track bars take a
//!   press (page or jump per [`TrackClick`]) and a drag, through the shared
//!   [`ScrollPane`] (S3). A scrollable child inside the view keeps the
//!   wheel while it can scroll on that axis and passes it on when its
//!   content fits (scroll chaining, [`ScrollPane::wheel`]).
//! - **Keyboard**: moving focus to a control the view clips scrolls it into
//!   view; so does a list cursor (the single focus gutter, `▎`) or an
//!   editing caret moving out of view. Keys the focused control leaves
//!   unused scroll the view ([`ScrollCmd`]: Page Up/Down, Alt+Page Up/Down
//!   sideways, Home/End).
//!
//! How it draws: content that fits is drawn in place, exactly as without
//! the view. Content that overflows is drawn once at full size into a
//! buffer off screen, and the window in view is copied to the screen; the
//! hit regions, the editing caret, cut-text reveals and overlays (popups)
//! queued by the content are moved with it, and pointer events on the
//! content report positions in the coordinates the content drew in
//! ([`HitRegistry::scroll_since`](crate::core::hit::HitRegistry::scroll_since)),
//! so widgets inside need no change. Every control stays in the focus ring
//! whether it is in view or not.
//!
//! Two calling styles: [`ScrollView::render`] draws the content with a
//! closure; [`ScrollView::begin`] and [`ScrollView::end`] bracket drawing
//! done by code that takes the buffer itself (a page drawing its body).

use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::{Position, Rect, Size};
use ratatui::widgets::Padding;

use crate::core::event::{Key, Outcome};
use crate::core::hit::{Axis, HitMark};
use crate::core::id::WidgetId;
use crate::core::keymap::{Actions, KeyMap};
use crate::core::pointer::PointerEvent;
use crate::core::response::Response;
use crate::core::scroll::{ScrollPane, Scrollable, ScrollbarVisibility, TrackClick};
use crate::ui::ctx::{RenderCtx, Shift, copy_cells};

/// The axes a [`ScrollView`] may scroll on. On an axis that does not
/// scroll the content gets the view's size and lays itself out in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Axes {
    pub vertical: bool,
    pub horizontal: bool,
}

impl Axes {
    /// Both axes (the default).
    pub const BOTH: Self = Self {
        vertical: true,
        horizontal: true,
    };
    /// Rows only: the content always gets the view's width.
    pub const VERTICAL: Self = Self {
        vertical: true,
        horizontal: false,
    };
    /// Columns only: the content always gets the view's height.
    pub const HORIZONTAL: Self = Self {
        vertical: false,
        horizontal: true,
    };
}

impl Default for Axes {
    fn default() -> Self {
        Self::BOTH
    }
}

/// A key a [`ScrollView`] answers when the focused control inside it left
/// the key unused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollCmd {
    /// One view up.
    PageUp,
    /// One view down.
    PageDown,
    /// One view left.
    PageLeft,
    /// One view right.
    PageRight,
    /// The top of the content.
    Top,
    /// The bottom of the content.
    Bottom,
}

impl Actions for ScrollCmd {
    /// Page Up/Down and Home/End scroll a macOS scroll view (Fn+arrows on
    /// a laptop). macOS has no sideways page key; Alt (Option) with Page
    /// Up/Down is the closest chord a terminal delivers, the way Option
    /// widens a move elsewhere.
    fn default_keymap() -> KeyMap<Self> {
        use crate::core::keymap::KeyBinding;
        use ratatui::crossterm::event::{KeyCode, KeyModifiers};
        let none = KeyModifiers::NONE;
        let alt = KeyModifiers::ALT;
        vec![
            (KeyBinding::new(KeyCode::PageUp, none), ScrollCmd::PageUp),
            (
                KeyBinding::new(KeyCode::PageDown, none),
                ScrollCmd::PageDown,
            ),
            (KeyBinding::new(KeyCode::PageUp, alt), ScrollCmd::PageLeft),
            (
                KeyBinding::new(KeyCode::PageDown, alt),
                ScrollCmd::PageRight,
            ),
            (KeyBinding::new(KeyCode::Home, none), ScrollCmd::Top),
            (KeyBinding::new(KeyCode::End, none), ScrollCmd::Bottom),
        ]
    }

    fn label(self) -> &'static str {
        match self {
            ScrollCmd::PageUp => "Scroll up a page",
            ScrollCmd::PageDown => "Scroll down a page",
            ScrollCmd::PageLeft => "Scroll left a page",
            ScrollCmd::PageRight => "Scroll right a page",
            ScrollCmd::Top => "Scroll to the top",
            ScrollCmd::Bottom => "Scroll to the bottom",
        }
    }
}

/// Where a [`ScrollView`] put its parts for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ViewLayout {
    /// The cells that show content.
    pub viewport: Rect,
    /// The content at full size, top-left on the viewport's top-left
    /// (its coordinates while it draws). Never smaller than the viewport.
    pub content: Rect,
    /// Where the content draws: [`Self::content`] less the padding.
    pub inner: Rect,
    /// The vertical bar's track, when it shows.
    pub v_track: Option<Rect>,
    /// The horizontal bar's track, when it shows.
    pub h_track: Option<Rect>,
    /// The blank cell where both bars meet.
    pub corner: Option<Rect>,
}

impl ViewLayout {
    /// Whether the content is larger than the viewport on either axis.
    pub fn overflows(&self) -> bool {
        self.content.width > self.viewport.width || self.content.height > self.viewport.height
    }
}

/// What the view was following last frame: focus moving, a caret or the
/// focus gutter moving, scrolls the view; a repaint does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Followed {
    focus: Option<WidgetId>,
    target: Rect,
}

/// A two-axis scroll container (module docs).
#[derive(Debug, Clone)]
pub struct ScrollView {
    pub id: WidgetId,
    /// Both axes' state and drawn tracks (S3).
    pub pane: ScrollPane,
    /// The axes that scroll. Default both.
    pub axes: Axes,
    /// When the bars show: `Auto` (the default) while the content
    /// overflows on that axis, `Always` whenever the axis scrolls (the
    /// bar's room is kept), `Never` (the view still scrolls by wheel and
    /// keys).
    pub bars: ScrollbarVisibility,
    /// Blank cells around the content, inside the scrolled area (they
    /// scroll with it). Default none.
    pub padding: Padding,
    /// Blank columns between the content and the vertical bar. Default
    /// [`crate::widgets::scrollbar::GAP`].
    pub scrollbar_gap: u16,
    /// Keys the view answers when the focused control leaves them unused
    /// (default [`ScrollCmd::default_keymap`]).
    pub keys: KeyMap<ScrollCmd>,
    /// Keep the focused control (its caret, its focus gutter) in view.
    /// Default on (WCAG 2.4.11).
    pub follow_focus: bool,
    /// Rows and columns kept visible beyond a control brought into view
    /// (standards-visual SVg9). Default 1.
    pub context: u16,
    followed: Option<Followed>,
    layout: ViewLayout,
}

impl ScrollView {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            pane: ScrollPane::default(),
            axes: Axes::BOTH,
            bars: ScrollbarVisibility::Auto,
            padding: Padding::ZERO,
            scrollbar_gap: crate::widgets::scrollbar::GAP,
            keys: ScrollCmd::default_keymap(),
            follow_focus: true,
            context: 1,
            followed: None,
            layout: ViewLayout::default(),
        }
    }

    /// See [`Self::axes`].
    pub fn axes(mut self, axes: Axes) -> Self {
        self.axes = axes;
        self
    }

    /// See [`Self::bars`].
    pub fn bars(mut self, bars: ScrollbarVisibility) -> Self {
        self.bars = bars;
        self
    }

    /// See [`Self::padding`].
    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    /// See [`Self::scrollbar_gap`].
    pub fn scrollbar_gap(mut self, cols: u16) -> Self {
        self.scrollbar_gap = cols;
        self
    }

    /// See [`Self::keys`].
    pub fn keys(mut self, keys: KeyMap<ScrollCmd>) -> Self {
        self.keys = keys;
        self
    }

    /// See [`Self::follow_focus`].
    pub fn follow_focus(mut self, on: bool) -> Self {
        self.follow_focus = on;
        self
    }

    /// See [`Self::context`].
    pub fn context(mut self, cells: u16) -> Self {
        self.context = cells;
        self
    }

    /// What a press on the bare track does (default: the app-wide
    /// [`RenderCtx::track_click`]).
    pub fn track_click(mut self, click: TrackClick) -> Self {
        self.pane = self.pane.track_click(click);
        self.pane.h_cfg.track_click = click;
        self
    }

    /// The layout of the last frame.
    pub fn last_layout(&self) -> ViewLayout {
        self.layout
    }

    /// The scroll offset (columns, rows).
    pub fn offset(&self) -> (usize, usize) {
        (self.pane.h.offset, self.pane.v.offset)
    }

    /// Scroll to `(columns, rows)`, clamped. Returns whether it moved.
    pub fn scroll_to(&mut self, x: usize, y: usize) -> bool {
        let h = self.pane.h.scroll_to(x);
        let v = self.pane.v.scroll_to(y);
        h || v
    }

    /// Back to the top-left, following nothing (a page or section change).
    pub fn reset(&mut self) {
        self.pane.v.offset = 0;
        self.pane.h.offset = 0;
        self.followed = None;
    }

    /// The size to lay content out at in `area`: `measure(area)`, unless
    /// that is taller than the view and no wider, when the vertical bar
    /// will take its columns: then `measure` again at the width the bar
    /// leaves, so content that only overflows downwards never scrolls
    /// sideways by the bar's width. Likewise when it is wider than the
    /// view and no taller: the horizontal bar will take its row, so
    /// `measure` again at the height the bar leaves, and content that
    /// fills the height and only overflows sideways never scrolls down by
    /// the bar's row.
    pub fn fit(&self, area: Size, measure: impl Fn(Size) -> Size) -> Size {
        let size = measure(area);
        let pad = self.padding;
        let (pad_w, pad_h) = (pad.left + pad.right, pad.top + pad.bottom);
        let room = 1 + self.scrollbar_gap;
        let shows = self.bars != ScrollbarVisibility::Never;
        let always = self.bars == ScrollbarVisibility::Always;
        let bar = self.axes.vertical
            && shows
            && area.width > room
            && (always || size.height.saturating_add(pad_h) > area.height);
        if bar && size.width.saturating_add(pad_w) <= area.width {
            let narrower = measure(Size::new(area.width - room, area.height));
            if narrower.width.saturating_add(pad_w) <= area.width - room {
                return narrower;
            }
        }
        let h_bar = self.axes.horizontal
            && shows
            && area.height > 1
            && (always || size.width.saturating_add(pad_w) > area.width);
        if h_bar && size.height.saturating_add(pad_h) <= area.height {
            let shorter = measure(Size::new(area.width, area.height - 1));
            if shorter.height.saturating_add(pad_h) <= area.height - 1 {
                return shorter;
            }
        }
        size
    }

    /// Lay the view out in `area` for content of `content` cells (its
    /// natural size, padding excluded). A bar takes its row or column only
    /// while its axis overflows (or always, per [`Self::bars`]); the two
    /// are settled together, since one bar can make the other axis
    /// overflow.
    pub fn layout(&self, area: Rect, content: Size) -> ViewLayout {
        let pad = self.padding;
        let need_w = content
            .width
            .saturating_add(pad.left)
            .saturating_add(pad.right);
        let need_h = content
            .height
            .saturating_add(pad.top)
            .saturating_add(pad.bottom);
        let shows = self.bars != ScrollbarVisibility::Never;
        let always = self.bars == ScrollbarVisibility::Always;
        let v_room = 1 + self.scrollbar_gap;
        let (mut v, mut h) = (false, false);
        for _ in 0..3 {
            let vw = area.width.saturating_sub(if v { v_room } else { 0 });
            let vh = area.height.saturating_sub(u16::from(h));
            let nv = self.axes.vertical && shows && (always || need_h > vh);
            let nh = self.axes.horizontal && shows && (always || need_w > vw);
            if (nv, nh) == (v, h) {
                break;
            }
            (v, h) = (nv, nh);
        }
        // a bar only when there is room for it beside some content
        let v = v && area.width > v_room;
        let h = h && area.height > 1;
        let vw = area.width.saturating_sub(if v { v_room } else { 0 });
        let vh = area.height.saturating_sub(u16::from(h));
        let viewport = Rect::new(area.x, area.y, vw, vh);
        let cw = if self.axes.horizontal {
            need_w.max(vw)
        } else {
            vw
        };
        let ch = if self.axes.vertical {
            need_h.max(vh)
        } else {
            vh
        };
        let content = Rect::new(area.x, area.y, cw, ch);
        let inner = Rect::new(
            content.x.saturating_add(pad.left),
            content.y.saturating_add(pad.top),
            cw.saturating_sub(pad.left + pad.right),
            ch.saturating_sub(pad.top + pad.bottom),
        );
        let v_track = v.then(|| Rect::new(area.right() - 1, area.y, 1, vh));
        let h_track = h.then(|| Rect::new(area.x, area.bottom() - 1, area.width - u16::from(v), 1));
        let corner = (v && h).then(|| Rect::new(area.right() - 1, area.bottom() - 1, 1, 1));
        ViewLayout {
            viewport,
            content,
            inner,
            v_track,
            h_track,
            corner,
        }
    }

    /// Draw content of natural size `content` into `area` with `draw`,
    /// which gets the rect to draw in (the full-size content, padding
    /// excluded) and the buffer to draw into.
    pub fn render(
        &mut self,
        area: Rect,
        content: Size,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        draw: impl FnOnce(Rect, &mut Buffer, &mut RenderCtx),
    ) {
        let mut frame = self.begin(area, content, buf, ctx);
        let inner = frame.area();
        draw(inner, frame.buffer(buf), ctx);
        self.end(frame, buf, ctx);
    }

    /// Start drawing content of natural size `content` into `area`. Draw
    /// into [`ScrollFrame::buffer`] at [`ScrollFrame::area`], then call
    /// [`Self::end`]. Anything else drawn into that buffer outside the
    /// content (a section strip above it, drawn in the same pass) lands on
    /// screen unchanged.
    pub fn begin(
        &mut self,
        area: Rect,
        content: Size,
        buf: &Buffer,
        ctx: &mut RenderCtx,
    ) -> ScrollFrame {
        let l = self.layout(area, content);
        self.layout = l;
        let p = &mut self.pane;
        p.begin(ctx.track_click);
        p.h_cfg.enabled = self.axes.horizontal;
        p.h_cfg.bar = self.bars;
        p.v.content_len = l.content.height as usize;
        p.v.set_viewport(l.viewport.height as usize);
        p.h.content_len = l.content.width as usize;
        p.h.set_viewport(l.viewport.width as usize);
        p.v.context = self.context as usize;
        p.h.context = self.context as usize;
        if !l.overflows() {
            p.v.offset = 0;
            p.h.offset = 0;
            return ScrollFrame {
                layout: l,
                off: None,
                mark: ctx.hits.mark(),
                cuts: 0,
                cursor: ctx.cursor,
                shift: None,
            };
        }
        // the wheel over the view, beneath everything the content
        // registers (an inner scroller is offered the notch first)
        ctx.scrollable(self.id, l.viewport);
        let shift = Shift::default();
        ctx.shifts.push(shift.clone());
        // the screen and the full-size content in one buffer: code that
        // draws beside the content in the same pass keeps its cells
        let whole = buf.area.union(l.content);
        let mut off = Buffer::empty(whole);
        copy_cells(buf, buf.area, &mut off, buf.area.as_position());
        let mut blank = Cell::default();
        if let Some(c) = buf.cell(l.viewport.as_position()) {
            blank.set_style(c.style());
        }
        for y in l.content.top()..l.content.bottom() {
            for x in l.content.left()..l.content.right() {
                off[(x, y)] = blank.clone();
            }
        }
        ScrollFrame {
            layout: l,
            off: Some(off),
            mark: ctx.hits.mark(),
            cuts: crate::ui::reveal::cuts_len(),
            cursor: ctx.cursor,
            shift: Some(shift),
        }
    }

    /// Finish a frame [`Self::begin`] started: bring what keyboard focus
    /// is on into view when it moved, put the window of the content on
    /// screen with everything it registered, and draw the bars.
    pub fn end(&mut self, frame: ScrollFrame, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let l = frame.layout;
        if let (Some(off), Some(shift)) = (frame.off, frame.shift) {
            ctx.shifts.pop();
            if self.follow_focus {
                self.follow(&off, l, frame.mark, frame.cursor, ctx);
            }
            let (ox, oy) = (self.pane.h.offset as u16, self.pane.v.offset as u16);
            let window = Rect::new(
                l.viewport.x + ox,
                l.viewport.y + oy,
                l.viewport.width,
                l.viewport.height,
            );
            let to = l.viewport.as_position();
            // what was drawn beside the content, then the window
            let screen = buf.area;
            for y in screen.top()..screen.bottom() {
                for x in screen.left()..screen.right() {
                    let p = Position::new(x, y);
                    if !l.content.contains(p) {
                        buf[(x, y)] = off[(x, y)].clone();
                    }
                }
            }
            copy_cells(&off, window, buf, to);
            ctx.hits.scroll_since(frame.mark, l.content, window, to);
            crate::ui::reveal::scroll_cuts_since(frame.cuts, l.content, window, to);
            if ctx.cursor != frame.cursor {
                ctx.cursor = match ctx.cursor {
                    Some(p) if window.contains(p) => Some(Position::new(p.x - ox, p.y - oy)),
                    Some(p) if !l.content.contains(p) => Some(p),
                    _ => frame.cursor,
                };
            }
            shift.set((ox, oy));
        }
        let id = self.id;
        match l.v_track {
            Some(track) => self.pane.draw_v(track, buf, ctx, id, false),
            None => self.pane.clear_track(Axis::Vertical),
        }
        match l.h_track {
            Some(track) => self.pane.draw_h(track, buf, ctx, id, false),
            None => self.pane.clear_track(Axis::Horizontal),
        }
        if let Some(c) = l.corner
            && let Some(cell) = buf.cell_mut(c.as_position())
        {
            cell.set_symbol(" ");
        }
    }

    /// Bring what keyboard focus is on into view when it moved since the
    /// last frame: the caret when one is editing in the content, else the
    /// focused control when it fits the view, else its focus gutter row
    /// (a list cursor), else the control's start.
    fn follow(
        &mut self,
        off: &Buffer,
        l: ViewLayout,
        mark: HitMark,
        cursor_before: Option<Position>,
        ctx: &RenderCtx,
    ) {
        let focus = ctx.interaction.focus;
        let control = focus.and_then(|f| {
            ctx.hits.regions()[mark_regions(ctx, mark)..]
                .iter()
                .rev()
                .find(|r| r.id == f && r.area.intersects(l.content))
                .map(|r| r.area.intersection(l.content))
        });
        let caret = ctx
            .cursor
            .filter(|p| ctx.cursor != cursor_before && l.content.contains(*p))
            .map(|p| Rect::new(p.x, p.y, 1, 1));
        let fits = |r: Rect| r.width <= l.viewport.width && r.height <= l.viewport.height;
        let target = match (caret, control) {
            (Some(c), _) => Some(c),
            (None, Some(r)) if fits(r) => Some(r),
            // the focused control's own gutter: a `▎` drawn elsewhere (a
            // legend explaining the mark) is not where focus is
            (None, Some(r)) => Some(gutter(off, r, ctx).unwrap_or(Rect::new(r.x, r.y, 1, 1))),
            (None, None) => None,
        };
        let Some(target) = target else {
            self.followed = None;
            return;
        };
        let now = Followed { focus, target };
        if self.followed == Some(now) {
            return;
        }
        self.followed = Some(now);
        self.pane
            .v
            .reveal_span((target.y - l.content.y) as usize, target.height as usize);
        self.pane
            .h
            .reveal_span((target.x - l.content.x) as usize, target.width as usize);
    }

    /// A key the focused control left unused: a [`ScrollCmd`] scrolls the
    /// view while it overflows on that axis. `None` for any other key, or
    /// when the view cannot move that way at all.
    pub fn on_key(&mut self, key: &Key) -> Option<Outcome> {
        let cmd = crate::core::keymap::lookup(&self.keys, key)?;
        let (v, h) = (&mut self.pane.v, &mut self.pane.h);
        let moved = match cmd {
            ScrollCmd::PageUp | ScrollCmd::PageDown | ScrollCmd::Top | ScrollCmd::Bottom
                if !v.overflows() =>
            {
                return None;
            }
            ScrollCmd::PageLeft | ScrollCmd::PageRight if !h.overflows() => return None,
            ScrollCmd::PageUp => v.page_up(),
            ScrollCmd::PageDown => v.page_down(),
            ScrollCmd::PageLeft => h.page_up(),
            ScrollCmd::PageRight => h.page_down(),
            ScrollCmd::Top => v.jump_start() | h.jump_start(),
            ScrollCmd::Bottom => v.jump_end(),
        };
        Some(if moved {
            Outcome::Changed
        } else {
            Outcome::Consumed
        })
    }

    /// A pointer event on the view itself (its wheel region or one of its
    /// bars): it scrolls through the shared [`ScrollPane`] path. `None`
    /// when the event is not the view's.
    pub fn pointer(&mut self, ev: &PointerEvent) -> Option<Response<()>> {
        if !ev.on(self.id) {
            return None;
        }
        crate::core::pointer::scroll_input(self, ev)
    }
}

/// Index of the first region registered since `mark`.
fn mark_regions(ctx: &RenderCtx, mark: HitMark) -> usize {
    mark.regions_len().min(ctx.hits.regions().len())
}

/// The rows of the focus gutter (`▎`, [`Theme::gutter_symbol`]) drawn in
/// `content` (the focused control's rect): exactly one control shows it,
/// so it marks where focus is inside a control taller than the view (a
/// list's cursor row).
///
/// [`Theme::gutter_symbol`]: crate::theme::Theme::gutter_symbol
fn gutter(off: &Buffer, content: Rect, ctx: &RenderCtx) -> Option<Rect> {
    let glyph = ctx.theme.glyphs.focus_bar;
    let area = content.intersection(off.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if off[(x, y)].symbol() == glyph {
                let mut h = 1;
                while y + h < area.bottom() && off[(x, y + h)].symbol() == glyph {
                    h += 1;
                }
                return Some(Rect::new(x, y, 1, h));
            }
        }
    }
    None
}

impl Scrollable for ScrollView {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.pane
    }
}

/// One frame of a [`ScrollView`] between [`ScrollView::begin`] and
/// [`ScrollView::end`].
#[derive(Debug)]
pub struct ScrollFrame {
    layout: ViewLayout,
    /// The full-size content and the screen, off screen; `None` when the
    /// content fits and draws in place.
    off: Option<Buffer>,
    mark: HitMark,
    cuts: usize,
    cursor: Option<Position>,
    shift: Option<Shift>,
}

impl ScrollFrame {
    /// Where the content draws: its full size, padding excluded, in the
    /// coordinates of [`Self::buffer`].
    pub fn area(&self) -> Rect {
        self.layout.inner
    }

    /// The frame's layout.
    pub fn layout(&self) -> ViewLayout {
        self.layout
    }

    /// The buffer to draw into: the screen's `buf` when the content fits,
    /// else the frame's own full-size buffer.
    pub fn buffer<'a>(&'a mut self, buf: &'a mut Buffer) -> &'a mut Buffer {
        match self.off.as_mut() {
            Some(off) => off,
            None => buf,
        }
    }
}

#[cfg(test)]
#[path = "scroll_view_tests.rs"]
mod tests;
