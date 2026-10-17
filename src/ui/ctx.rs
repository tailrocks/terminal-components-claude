//! Per-frame render context.
//!
//! Widgets never read global state directly. They receive a [`RenderCtx`]
//! that tells them what is focused/hovered/pressed and lets them register
//! hit regions and focus-ring entries for the *next* event cycle.
//!
//! Layering is owned here too. A widget that floats content above the page
//! (a select's option list, a dropdown) queues it with
//! [`RenderCtx::overlay`] instead of painting it in place, so whatever the
//! page draws afterwards can never cover or clip it. Queued overlays land
//! when a modal starts ([`RenderCtx::begin_modal`]: they belong to the page
//! the modal covers) and at the end of the frame
//! ([`RenderCtx::flush_overlays`]).

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};

use crate::core::focus::FocusRing;
use crate::core::hit::HitRegistry;
use crate::core::id::WidgetId;
use crate::theme::Theme;

/// Snapshot of interaction state relevant to rendering.
#[derive(Debug, Clone, Copy, Default)]
pub struct Interaction {
    pub focus: Option<WidgetId>,
    pub hover: Option<WidgetId>,
    pub pressed: Option<WidgetId>,
    /// Brief pressed feedback after an activation (keyboard or mouse).
    pub flash: Option<WidgetId>,
    /// Focus is hidden while a modal or a keyboard-owned popup covers the
    /// content below it: the page draws no focus indicator, so the only
    /// one on screen is inside the topmost layer. Set it at frame start
    /// with [`RenderCtx::page_under_layer`]; [`RenderCtx::begin_modal`]
    /// clears it for the layer it opens.
    pub focus_hidden: bool,
    /// Hover feedback is suppressed until the pointer moves again after a
    /// keyboard-driven change.
    pub hover_suppressed: bool,
    /// Focus last moved by the pointer (a click), not the keyboard.
    /// Controls under [`FocusVisible::KeyboardOnly`] hide their focus ring
    /// then, as macOS does for buttons clicked with the mouse.
    pub pointer_focus: bool,
    pub tick: u64,
    /// The terminal has not reported pointer motion without a button held
    /// ([`crate::core::pointer::PointerState::reports_motion`]): hover
    /// does not follow the pointer, so hover-driven behaviour (a menu-bar
    /// hover switch) is not relied on. Default `false`.
    pub motion_unreported: bool,
    /// How long the pointer has rested on [`Self::hover`], in ms
    /// ([`crate::core::pointer::PointerState::hover_ms`]). Delayed hover
    /// content (an expansion tooltip after [`crate::ui::reveal::Reveal::delay_ms`],
    /// a [`crate::ui::tooltip`]) waits on it. `None` (the default,
    /// and for a paused capture): the time is unknown and nothing waits.
    pub hover_ms: Option<u64>,
}

/// When a control shows its focus indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusVisible {
    /// Whenever it holds focus.
    #[default]
    Always,
    /// Only when the keyboard put focus there: a control focused by a
    /// click shows no ring until the next key (macOS without Full Keyboard
    /// Access).
    KeyboardOnly,
}

impl Interaction {
    /// Whether `id` shows as focused: it holds focus and no layer covers
    /// it. Use this for visuals.
    pub fn focused(&self, id: WidgetId) -> bool {
        !self.focus_hidden && self.has_focus(id)
    }
    /// [`Self::focused`] for an owner that may have opened its own popup
    /// (a tab strip's or chip bar's `+N` menu, a select's list): while
    /// `layer_open`, the popup is the topmost layer and the owner under
    /// it shows no focus, so exactly one indicator stays on screen.
    pub fn focused_under(&self, id: WidgetId, layer_open: bool) -> bool {
        !layer_open && self.focused(id)
    }
    /// The focused widget as visuals may show it: `None` while a layer
    /// covers the page. A container that marks "focus is somewhere in
    /// me" (a pane title, a card) reads this, never the raw focus, so it
    /// shows nothing under a modal or popup.
    pub fn shown_focus(&self) -> Option<WidgetId> {
        self.focus.filter(|_| !self.focus_hidden)
    }
    /// Whether `id` holds keyboard focus, shown or not. Use this for
    /// behaviour (commit an edit on blur, close a dropdown): a layer
    /// opening above a control hides its focus but does not blur it.
    pub fn has_focus(&self, id: WidgetId) -> bool {
        self.focus == Some(id)
    }
    /// The hovered widget, unless hover is suppressed by keyboard use: what
    /// pointer-following behaviour (a menu cursor, a menu-bar switch)
    /// should read, so the keyboard wins until the pointer moves again.
    pub fn live_hover(&self) -> Option<WidgetId> {
        self.hover.filter(|_| !self.hover_suppressed)
    }
    /// [`Self::focused`] under a [`FocusVisible`] policy.
    pub fn focus_visible(&self, id: WidgetId, mode: FocusVisible) -> bool {
        self.focused(id) && (mode == FocusVisible::Always || !self.pointer_focus)
    }
    pub fn hovered(&self, id: WidgetId) -> bool {
        !self.hover_suppressed && self.hover == Some(id)
    }
    pub fn pressed(&self, id: WidgetId) -> bool {
        (self.pressed == Some(id) && self.hover == Some(id)) || self.flash == Some(id)
    }
    /// Any of `ids` is pressed: a row counts a press on its cells, its
    /// toggle or its × as its own, just as it counts their hover.
    pub fn pressed_any(&self, ids: impl IntoIterator<Item = WidgetId>) -> bool {
        ids.into_iter().any(|id| self.pressed(id))
    }
    /// Any of `ids` is hovered.
    pub fn hovered_any(&self, ids: impl IntoIterator<Item = WidgetId>) -> bool {
        ids.into_iter().any(|id| self.hovered(id))
    }
    /// The keyboard activation flash landed on `container`. Applications
    /// flash the focused id, which for a list, tree, table, grid, radio
    /// group, chip set, tab strip or menu is the container: the container
    /// applies it to its cursor child.
    pub fn flash_within(&self, container: WidgetId) -> bool {
        self.flash == Some(container)
    }
}

/// Visual state of a single control, resolved from interaction + own flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VisualState {
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub selected: bool,
    pub disabled: bool,
    pub error: bool,
    pub editing: bool,
    pub busy: bool,
}

/// Deferred drawing queued by [`RenderCtx::overlay`]. It owns what it draws
/// (no borrow of the frame), so a context never outlives its borrows.
type OverlayFn = Box<dyn for<'r> FnOnce(&mut Buffer, &mut RenderCtx<'r>)>;

pub struct RenderCtx<'a> {
    pub theme: &'a Theme,
    pub interaction: Interaction,
    pub hits: &'a mut HitRegistry,
    pub ring: &'a mut FocusRing,
    /// Where the hardware cursor should be placed, if any widget is editing.
    pub cursor: Option<Position>,
    /// Frame-level flag: a modal is open, so page content must neither
    /// register hits nor focus entries.
    pub inert: bool,
    /// Overlays waiting to be drawn, with the `inert` flag they were queued
    /// under (a popup on a page behind a modal stays unreachable).
    overlays: Vec<(bool, OverlayFn)>,
    /// App-wide default for revealing truncated text; widgets may override
    /// it (see [`crate::ui::reveal`]).
    pub reveal: crate::ui::reveal::Reveal,
    /// App-wide tooltip policy (see [`crate::ui::tooltip`]); a call may
    /// override it.
    pub tooltips: crate::ui::tooltip::Tooltips,
    /// App-wide default for content wider than its area (horizontal
    /// scrolling and its bar); widgets may override it.
    pub hscroll: crate::core::scroll::HScroll,
    /// App-wide default for a press on the bare track of a vertical
    /// scrollbar (`Page`, the default, moves a page toward the pointer as
    /// macOS and Windows do; `Jump` moves the thumb under it); widgets with
    /// a `track_click` field may override it.
    pub track_click: crate::core::scroll::TrackClick,
    /// The frame's single truncated-text reveal, if any.
    pub(crate) reveal_slot: crate::ui::reveal::Slot,
    /// The reveal key hint of the focused widget's cut cursor row, set
    /// while it renders: footers drawn after the content append it with
    /// [`crate::ui::reveal::advertise`], so the key is named exactly when
    /// it does something, in every app and for every widget.
    pub reveal_hint: Option<(&'static str, &'static str)>,
    /// The full text of the focused widget's cut cursor value, set while
    /// it renders ([`crate::ui::reveal::peek`]): the footer drawn after
    /// the content shows it in its status slot when nothing else is said
    /// there (the peek line, SI8), so a cut value reads in full on focus,
    /// with no key and no pointer.
    pub peek: Option<String>,
    /// The scroll offsets of the [`ScrollView`](crate::ui::scroll_view::ScrollView)s
    /// drawing now, innermost last. A view fills its cell once its offset
    /// is final (after its content is drawn); overlays queued inside it
    /// read the cells when they are flushed, so they land where their
    /// anchor is on screen.
    pub(crate) shifts: Vec<Shift>,
}

/// One scrolled view's offset (columns, rows), filled in when the view
/// ends ([`RenderCtx::shifts`]).
pub(crate) type Shift = std::rc::Rc<std::cell::Cell<(u16, u16)>>;

/// The summed offset of nested views.
pub(crate) fn total_shift(shifts: &[Shift]) -> (u16, u16) {
    shifts.iter().fold((0, 0), |(x, y), s| {
        let (a, b) = s.get();
        (x.saturating_add(a), y.saturating_add(b))
    })
}

/// `draw`, queued inside scrolled views with offsets `shifts`, run so its
/// content coordinates land on screen: it draws into a copy of the screen
/// placed at the content's coordinates, its cells are copied back, and
/// the hit regions, cursor and overlays it adds are moved with them.
fn shifted(shifts: Vec<Shift>, draw: OverlayFn) -> OverlayFn {
    Box::new(move |buf: &mut Buffer, ctx: &mut RenderCtx| {
        let (dx, dy) = total_shift(&shifts);
        if dx == 0 && dy == 0 {
            draw(buf, ctx);
            return;
        }
        let screen = buf.area;
        let view = Rect::new(
            screen.x.saturating_add(dx),
            screen.y.saturating_add(dy),
            screen.width,
            screen.height,
        );
        let mut off = Buffer::empty(view);
        copy_cells(buf, screen, &mut off, view.as_position());
        let mark = ctx.hits.mark();
        let cursor = ctx.cursor;
        let queued = ctx.overlays.len();
        draw(&mut off, ctx);
        copy_cells(&off, view, buf, screen.as_position());
        ctx.hits
            .scroll_since(mark, view, view, screen.as_position());
        if ctx.cursor != cursor {
            ctx.cursor = ctx
                .cursor
                .and_then(|p| view.contains(p).then(|| Position::new(p.x - dx, p.y - dy)));
        }
        // overlays it queued are in the same coordinates
        for item in ctx.overlays.iter_mut().skip(queued) {
            let f = std::mem::replace(&mut item.1, Box::new(|_, _| {}));
            item.1 = shifted(shifts.clone(), f);
        }
    })
}

/// Copy the cells of `from` in `area` to `to` with `area`'s top-left at
/// `at`; cells outside either buffer are skipped.
pub(crate) fn copy_cells(from: &Buffer, area: Rect, to: &mut Buffer, at: Position) {
    let area = area.intersection(from.area);
    for y in 0..area.height {
        for x in 0..area.width {
            let (tx, ty) = (at.x as u32 + x as u32, at.y as u32 + y as u32);
            if tx > u16::MAX as u32 || ty > u16::MAX as u32 {
                continue;
            }
            let p = Position::new(tx as u16, ty as u16);
            if to.area.contains(p) {
                to[(p.x, p.y)] = from[(area.x + x, area.y + y)].clone();
            }
        }
    }
}

impl<'a> RenderCtx<'a> {
    pub fn new(
        theme: &'a Theme,
        interaction: Interaction,
        hits: &'a mut HitRegistry,
        ring: &'a mut FocusRing,
    ) -> Self {
        Self {
            theme,
            interaction,
            hits,
            ring,
            cursor: None,
            inert: false,
            overlays: Vec::new(),
            reveal: crate::ui::reveal::Reveal::default(),
            tooltips: crate::ui::tooltip::Tooltips::default(),
            hscroll: crate::core::scroll::HScroll::default(),
            track_click: crate::core::scroll::TrackClick::Page,
            reveal_hint: None,
            peek: None,
            shifts: Vec::new(),
            reveal_slot: {
                crate::ui::reveal::clear_cuts();
                crate::ui::reveal::begin_frame();
                crate::ui::tooltip::begin_frame();
                crate::ui::reveal::Slot::default()
            },
        }
    }

    /// A context for drawing a static sample inside a frame (a
    /// [`crate::ui::specimen::StateGrid`] cell): inert, so it registers no
    /// hit, focus stop or cursor; overlays queued on it are dropped with
    /// it; and, unlike [`Self::new`], it leaves the frame's reveal and
    /// tooltip bookkeeping alone.
    pub(crate) fn detached(
        theme: &'a Theme,
        interaction: Interaction,
        hits: &'a mut HitRegistry,
        ring: &'a mut FocusRing,
    ) -> Self {
        Self {
            theme,
            interaction,
            hits,
            ring,
            cursor: None,
            inert: true,
            overlays: Vec::new(),
            reveal: crate::ui::reveal::Reveal::OFF,
            tooltips: crate::ui::tooltip::Tooltips::OFF,
            hscroll: crate::core::scroll::HScroll::default(),
            track_click: crate::core::scroll::TrackClick::Page,
            reveal_hint: None,
            peek: None,
            shifts: Vec::new(),
            reveal_slot: crate::ui::reveal::Slot::default(),
        }
    }

    /// Register a focusable, clickable control occupying `area`.
    pub fn control(&mut self, id: WidgetId, area: Rect, disabled: bool) {
        if self.inert {
            return;
        }
        self.hits.register(id, area);
        if !disabled {
            self.ring.register(id);
        }
    }

    /// Keep `id` in the Tab order without drawing it: a control scrolled
    /// out of a [`Stack`](crate::ui::layout::Stack) stays reachable by
    /// keyboard, and focusing it scrolls it into view on the next frame.
    pub fn focus_stop(&mut self, id: WidgetId) {
        if !self.inert {
            self.ring.register(id);
        }
    }

    /// Register a clickable-only region (no keyboard focus).
    pub fn clickable(&mut self, id: WidgetId, area: Rect) {
        if !self.inert {
            self.hits.register(id, area);
        }
    }

    /// Register `part` of `owner` as a clickable region
    /// ([`HitRegistry::register_part`]); returns the part's id
    /// ([`WidgetId::part`](crate::core::id::WidgetId::part)). Nothing is
    /// registered while the frame is inert.
    pub fn register_part(
        &mut self,
        owner: WidgetId,
        part: crate::core::id::PartRef,
        area: Rect,
    ) -> WidgetId {
        let id = owner.part(part);
        self.register_part_as(id, owner, part, area);
        id
    }

    /// [`Self::register_part`] under an id finer than the part (one id per
    /// grid cell, a sub-control inside a row)
    /// ([`HitRegistry::register_part_as`]).
    pub fn register_part_as(
        &mut self,
        id: WidgetId,
        owner: WidgetId,
        part: crate::core::id::PartRef,
        area: Rect,
    ) {
        if !self.inert {
            self.hits.register_part_as(id, owner, part, area);
        }
    }

    /// The pointer shape over `id` ([`crate::core::shape`]).
    pub fn pointer_shape(&mut self, id: WidgetId, shape: crate::core::shape::PointerShape) {
        if !self.inert {
            self.hits.set_shape(id, shape);
        }
    }

    /// [`Self::clickable`] beneath every region of the current layer: it
    /// only takes cells no other control claims (a splitter's padding).
    pub fn clickable_under(&mut self, id: WidgetId, area: Rect) {
        if !self.inert {
            self.hits.register_under(id, area);
        }
    }

    /// Register `owner`'s scrollbar on `axis`, drawn in `track`.
    pub fn scrollbar(
        &mut self,
        id: WidgetId,
        track: Rect,
        owner: WidgetId,
        axis: crate::core::hit::Axis,
    ) {
        if !self.inert {
            self.hits.register_bar(crate::core::hit::Bar {
                id,
                owner,
                axis,
                track,
            });
        }
    }

    /// Register the thumb of `owner`'s scrollbar `id` (after
    /// [`Self::scrollbar`]): its cells resolve to [`Part::THUMB`](crate::core::id::Part::THUMB).
    pub fn scrollbar_thumb(&mut self, id: WidgetId, thumb: Rect, owner: WidgetId) {
        if !self.inert {
            self.hits.register_thumb(id, owner, thumb);
        }
    }

    /// Register a wheel-scrollable container.
    pub fn scrollable(&mut self, id: WidgetId, area: Rect) {
        if !self.inert {
            self.hits.register_scroll(id, area);
        }
    }

    /// Register a drag-selecting widget's scrolling body for edge
    /// autoscroll ([`HitRegistry::register_autoscroll`]).
    pub fn autoscroll(&mut self, area: Rect) {
        if !self.inert {
            self.hits.register_autoscroll(area);
        }
    }

    pub fn state(&self, id: WidgetId) -> VisualState {
        VisualState {
            focused: self.interaction.focused(id),
            hovered: self.interaction.hovered(id),
            pressed: self.interaction.pressed(id),
            ..Default::default()
        }
    }

    /// State of `child` inside `container`: pressed when the pointer holds
    /// it, or when the container's keyboard activation flash lands while
    /// `child` carries the cursor.
    pub fn child_state(&self, container: WidgetId, child: WidgetId, cursor: bool) -> VisualState {
        let mut s = self.state(child);
        s.pressed |= cursor && self.interaction.flash_within(container);
        s
    }

    /// Style of a clickable text link on `bg` (a header link, an overflow
    /// badge, a `+`, an app's own clickable label): the title ladder for
    /// this id's hover and press. Callers keep their own tone with
    /// [`Theme::legible`].
    pub fn title_style(&self, id: WidgetId, bg: ratatui::style::Color) -> ratatui::style::Style {
        let t = self.theme;
        let mut s = self.state(id);
        s.focused = false;
        t.interactive(crate::theme::Kind::Title, s, t.plane_of(bg))
            .style
    }

    pub fn set_cursor(&mut self, pos: Position) {
        if !self.inert {
            self.cursor = Some(pos);
        }
    }

    /// Queue drawing that must sit above everything rendered so far in this
    /// layer: it runs when the layer closes (a modal begins or the frame
    /// ends), so its pixels and hit regions are always on top of the page.
    ///
    /// Inside a [`ScrollView`](crate::ui::scroll_view::ScrollView) the
    /// drawing keeps the content's coordinates it was queued with: it is
    /// moved by the view's final offset when it runs.
    pub fn overlay(
        &mut self,
        draw: impl for<'r> FnOnce(&mut Buffer, &mut RenderCtx<'r>) + 'static,
    ) {
        let draw: OverlayFn = Box::new(draw);
        let draw = if self.shifts.is_empty() {
            draw
        } else {
            shifted(self.shifts.clone(), draw)
        };
        self.overlays.push((self.inert, draw));
    }

    /// [`Self::overlay`] in screen coordinates whatever view queues it:
    /// for drawing that places itself (the frame's reveal slot).
    pub(crate) fn overlay_unshifted(
        &mut self,
        draw: impl for<'r> FnOnce(&mut Buffer, &mut RenderCtx<'r>) + 'static,
    ) {
        self.overlays.push((self.inert, Box::new(draw)));
    }

    /// Draw with `draw` confined to `area`: cells it writes outside `area`
    /// are restored, hit regions it registers are cut to `area` and the
    /// editing cursor is dropped when it lands outside. Use it around
    /// content that may be taller or wider than the rect it was given (a
    /// partially scrolled child, a body that can overflow) so it can never
    /// paint over its neighbours, the footer or a frame. Overlays queued
    /// inside escape the clip on purpose: they float above the page.
    pub fn clip<R>(
        &mut self,
        buf: &mut Buffer,
        area: Rect,
        draw: impl FnOnce(&mut Buffer, &mut Self) -> R,
    ) -> R {
        let area = area.intersection(buf.area);
        let before = buf.clone();
        let hits_from = self.hits.len();
        let cursor = self.cursor;
        let out = draw(buf, self);
        let full = buf.area;
        for y in full.top()..full.bottom() {
            for x in full.left()..full.right() {
                let p = Position::new(x, y);
                if !area.contains(p) {
                    buf[(x, y)] = before[(x, y)].clone();
                }
            }
        }
        self.hits.clip_since(hits_from, area);
        if let Some(p) = self.cursor
            && self.cursor != cursor
            && !area.contains(p)
        {
            self.cursor = cursor;
        }
        out
    }

    /// Draw every queued overlay, oldest first. Overlays queued while
    /// flushing are drawn too, above the one that queued them. Every
    /// application calls this once after drawing its frame.
    pub fn flush_overlays(&mut self, buf: &mut Buffer) {
        // cut titles drawn by context-free chrome reveal like any cut cell
        crate::ui::reveal::flush_cuts(self);
        while !self.overlays.is_empty() {
            let queued = std::mem::take(&mut self.overlays);
            for (inert, draw) in queued {
                let outer = std::mem::replace(&mut self.inert, inert);
                draw(buf, self);
                self.inert = outer;
            }
        }
    }

    /// Declare, before the page draws, whether a layer (a modal, or a
    /// keyboard-owned popup such as an open menu) will be drawn above it
    /// this frame. While one is, page content renders with no focus
    /// indicator: exactly one "you are here" cue stays on screen, inside
    /// the topmost layer. [`Self::begin_modal`] clears the flag for the
    /// layer it opens.
    pub fn page_under_layer(&mut self, open: bool) {
        self.interaction.focus_hidden = open;
    }

    /// Enter modal mode: pending page overlays land first (they belong to
    /// the layer the modal covers), then everything registered so far is
    /// unreachable.
    pub fn begin_modal(&mut self, buf: &mut Buffer) {
        self.flush_overlays(buf);
        self.hits.push_barrier();
        self.ring.push_barrier();
        self.inert = false;
        self.interaction.focus_hidden = false;
    }

    /// Draw a container whose controls Tab visits in visual order, top to
    /// bottom then left to right, whatever order `draw` registers them in
    /// (keyboard audit K30, `order: Visual`): a card that draws its action
    /// row before its header still focuses the header first. A control
    /// with no hit region this frame (a [`Self::focus_stop`]) keeps its
    /// place after the drawn ones.
    pub fn visual_order<T>(&mut self, draw: impl FnOnce(&mut Self) -> T) -> T {
        let start = self.ring.len();
        let out = draw(self);
        let hits = &self.hits;
        self.ring.sort_from(start, |id| {
            hits.regions()
                .iter()
                .rev()
                .find(|r| r.id == *id)
                .map_or((u16::MAX, u16::MAX), |r| (r.area.y, r.area.x))
        });
        out
    }

    /// A panel about to draw over everything drawn so far (a form that
    /// takes the whole page on a narrow terminal): pending page overlays
    /// land first, then everything registered so far is inert, out of the
    /// Tab ring and the pointer's reach (keyboard audit K27). Unlike
    /// [`Self::begin_modal`] the page stays the active layer: nothing is
    /// dimmed and its focus stays visible.
    pub fn cover(&mut self, buf: &mut Buffer) {
        self.flush_overlays(buf);
        self.hits.push_barrier();
        self.ring.push_barrier();
    }

    /// Dim `screen` behind a modal and enter modal mode. The bottom row
    /// stays live: the footer's key hints belong to the modal. Shorthand
    /// for [`Self::modal_backdrop_with`] and [`Backdrop::DEFAULT`].
    pub fn modal_backdrop(&mut self, buf: &mut Buffer, screen: Rect) {
        self.modal_backdrop_with(buf, screen, Backdrop::DEFAULT);
    }

    /// Dim `screen` behind a modal, except the rows `backdrop` keeps live
    /// (an app's footer and status rows whose hints belong to the modal),
    /// and enter modal mode.
    pub fn modal_backdrop_with(&mut self, buf: &mut Buffer, screen: Rect, backdrop: Backdrop) {
        // overlays first, so the page's open popups dim with the page
        self.flush_overlays(buf);
        let t = self.theme;
        let top = backdrop.keep_top.min(screen.height);
        let dim = Rect::new(
            screen.x,
            screen.y + top,
            screen.width,
            screen
                .height
                .saturating_sub(top)
                .saturating_sub(backdrop.keep_bottom),
        );
        for pos in dim.positions() {
            if let Some(c) = buf.cell_mut(pos) {
                // a focus bar under the modal is never "here": the only
                // focus indicator belongs to the modal. An ASCII bar (`|`)
                // is also ordinary text, so it must carry the focus ink.
                let bar = t.glyphs.focus_bar;
                let is_bar = c.symbol() == bar
                    && (!bar.is_ascii() || c.fg == t.focus || c.fg == t.text_on_accent);
                let st = t.backdrop(c.style());
                c.set_style(st);
                // only the backdrop's own dimming survives
                c.modifier = st.add_modifier;
                if is_bar {
                    c.set_symbol(" ");
                }
            }
        }
        self.begin_modal(buf);
    }
}

/// Keep every ink of a hand-drawn row readable on its own fill: a cell
/// whose background differs from the `plane` it sits on (a selected or
/// hovered row) takes [`Theme::legible`] of its ink, the step up the text
/// ladder that reads at 4.5:1. Cells on the plane itself are left alone,
/// so the row's rest look keeps its hierarchy. The focus gutter
/// ([`Theme::gutter_symbol`] in [`Theme::focus`]) is a graphic, not text:
/// it keeps the focus ink, so the one focus indicator never changes
/// colour on a fill. Apps that draw list rows by hand call it after the
/// row, the way the library lists resolve their own inks.
pub fn legible_on_fill(buf: &mut Buffer, area: Rect, t: &Theme, plane: ratatui::style::Color) {
    for pos in area.positions() {
        if let Some(c) = buf.cell_mut(pos)
            && c.bg != plane
            && !c.symbol().trim().is_empty()
            && !(c.symbol() == t.glyphs.focus_bar && c.fg == t.focus)
        {
            c.fg = t.legible(c.fg, c.bg);
        }
    }
}

/// Fill an area with a background colour without touching symbols.
pub fn fill(buf: &mut Buffer, area: Rect, style: ratatui::style::Style) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_symbol(" ");
                cell.set_style(style);
            }
        }
    }
}

/// Which rows a modal backdrop leaves undimmed
/// ([`RenderCtx::modal_backdrop_with`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backdrop {
    /// Rows at the bottom of the screen kept live (the footer whose key
    /// hints belong to the modal, a status bar). Default 1.
    pub keep_bottom: u16,
    /// Rows at the top kept live (a header that stays readable). Default 0.
    pub keep_top: u16,
}

impl Backdrop {
    /// One live footer row, nothing kept at the top: today's backdrop.
    pub const DEFAULT: Self = Self {
        keep_bottom: 1,
        keep_top: 0,
    };
    /// Dim everything.
    pub const FULL: Self = Self {
        keep_bottom: 0,
        keep_top: 0,
    };

    pub const fn keep_bottom(mut self, rows: u16) -> Self {
        self.keep_bottom = rows;
        self
    }

    pub const fn keep_top(mut self, rows: u16) -> Self {
        self.keep_top = rows;
        self
    }
}

impl Default for Backdrop {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_fill_steps_its_text_up_and_keeps_the_focus_gutter() {
        // 256 colours: the cursor row fill is 22; the focus ink (78) reads
        // on it at 4.4:1 (a graphic needs 3:1), muted text (245) at 2.3:1
        let t = Theme::for_level(crate::theme::ColorLevel::Ansi256);
        let s = VisualState {
            focused: true,
            selected: true,
            ..VisualState::default()
        };
        let fill_bg = t.row(s, t.canvas).bg.expect("a cursor row fill");
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 1));
        let row = Rect::new(0, 0, 12, 1);
        fill(&mut buf, row, ratatui::style::Style::new().bg(fill_bg));
        buf.set_string(0, 0, t.gutter_symbol(s), t.gutter(s, fill_bg, false));
        buf.set_string(
            2,
            0,
            "meta",
            ratatui::style::Style::new().fg(t.text_muted).bg(fill_bg),
        );
        legible_on_fill(&mut buf, row, &t, t.canvas);
        assert_eq!(buf[(0, 0)].fg, t.focus, "the focus gutter keeps its ink");
        let fg = buf[(2, 0)].fg;
        assert!(
            crate::theme::contrast_ratio(fg, fill_bg).is_some_and(|r| r >= 4.5),
            "{fg:?} on {fill_bg:?}"
        );
    }

    #[test]
    fn a_clip_keeps_cells_hits_and_cursor_inside_its_rect() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 4));
        buf.set_string(0, 3, "footer", ratatui::style::Style::new());
        let clip = Rect::new(0, 0, 10, 2);
        let id = WidgetId::of("tall");
        ctx.clip(&mut buf, clip, |buf, ctx| {
            for y in 0..4 {
                buf.set_string(0, y, "xxxxxxxxxx", ratatui::style::Style::new());
            }
            ctx.clickable(id, Rect::new(0, 0, 10, 4));
            ctx.set_cursor(Position::new(1, 3));
        });
        assert_eq!(buf[(0, 1)].symbol(), "x");
        assert_eq!(buf[(0, 2)].symbol(), " ", "nothing drawn below the clip");
        assert_eq!(buf[(0, 3)].symbol(), "f", "the footer survives");
        assert_eq!(ctx.hits.area_of(id), Some(clip));
        assert_eq!(ctx.hits.hit(Position::new(0, 3)), None);
        assert_eq!(ctx.cursor, None);
    }

    #[test]
    fn a_page_under_a_layer_shows_no_focus_and_the_layer_does() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let id = WidgetId::of("field");
        let interaction = Interaction {
            focus: Some(id),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&t, interaction, &mut hits, &mut ring);
        ctx.page_under_layer(true);
        assert!(!ctx.state(id).focused, "the page draws no focus");
        assert!(ctx.interaction.has_focus(id), "but the control keeps it");
        // a bar drawn anyway (a hardcoded glyph) never survives the backdrop
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));
        buf.set_string(0, 0, "▎", ratatui::style::Style::new().fg(t.focus));
        ctx.modal_backdrop(&mut buf, Rect::new(0, 0, 10, 3));
        assert_eq!(buf[(0, 0)].symbol(), " ");
        assert!(ctx.state(id).focused, "the modal's own layer shows focus");
    }

    #[test]
    fn a_backdrop_keeps_the_configured_chrome_rows_live() {
        let t = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&t, Interaction::default(), &mut hits, &mut ring);
        let screen = Rect::new(0, 0, 4, 6);
        let style = ratatui::style::Style::new().fg(t.text_primary).bg(t.canvas);
        let mut buf = Buffer::empty(screen);
        for y in 0..6 {
            buf.set_string(0, y, "abcd", style);
        }
        ctx.modal_backdrop_with(
            &mut buf,
            screen,
            Backdrop::DEFAULT.keep_bottom(2).keep_top(1),
        );
        assert_eq!(
            buf[(0, 0)].style().fg,
            Some(t.text_primary),
            "header row kept"
        );
        assert!(
            buf[(0, 1)].style().fg != Some(t.text_primary),
            "body dimmed"
        );
        assert!(
            buf[(0, 4)].style().fg == Some(t.text_primary),
            "status row kept"
        );
        assert!(
            buf[(0, 5)].style().fg == Some(t.text_primary),
            "footer row kept"
        );
    }
}
