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
    /// Focus is hidden while a modal covers the content below it.
    pub focus_hidden: bool,
    /// Hover feedback is suppressed until the pointer moves again after a
    /// keyboard-driven change.
    pub hover_suppressed: bool,
    pub tick: u64,
}

impl Interaction {
    pub fn focused(&self, id: WidgetId) -> bool {
        !self.focus_hidden && self.focus == Some(id)
    }
    /// The hovered widget, unless hover is suppressed by keyboard use: what
    /// pointer-following behaviour (a menu cursor, a menu-bar switch)
    /// should read, so the keyboard wins until the pointer moves again.
    pub fn live_hover(&self) -> Option<WidgetId> {
        self.hover.filter(|_| !self.hover_suppressed)
    }
    pub fn hovered(&self, id: WidgetId) -> bool {
        !self.hover_suppressed && self.hover == Some(id)
    }
    pub fn pressed(&self, id: WidgetId) -> bool {
        (self.pressed == Some(id) && self.hover == Some(id)) || self.flash == Some(id)
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

    /// Register a clickable-only region (no keyboard focus).
    pub fn clickable(&mut self, id: WidgetId, area: Rect) {
        if !self.inert {
            self.hits.register(id, area);
        }
    }

    /// Register a wheel-scrollable container.
    pub fn scrollable(&mut self, id: WidgetId, area: Rect) {
        if !self.inert {
            self.hits.register_scroll(id, area);
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

    pub fn set_cursor(&mut self, pos: Position) {
        if !self.inert {
            self.cursor = Some(pos);
        }
    }

    /// Queue drawing that must sit above everything rendered so far in this
    /// layer: it runs when the layer closes (a modal begins or the frame
    /// ends), so its pixels and hit regions are always on top of the page.
    pub fn overlay(
        &mut self,
        draw: impl for<'r> FnOnce(&mut Buffer, &mut RenderCtx<'r>) + 'static,
    ) {
        self.overlays.push((self.inert, Box::new(draw)));
    }

    /// Draw every queued overlay, oldest first. Overlays queued while
    /// flushing are drawn too, above the one that queued them. Every
    /// application calls this once after drawing its frame.
    pub fn flush_overlays(&mut self, buf: &mut Buffer) {
        while !self.overlays.is_empty() {
            let queued = std::mem::take(&mut self.overlays);
            for (inert, draw) in queued {
                let outer = std::mem::replace(&mut self.inert, inert);
                draw(buf, self);
                self.inert = outer;
            }
        }
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

    /// Dim `screen` behind a modal and enter modal mode. The bottom row
    /// stays live: the footer's key hints belong to the modal.
    pub fn modal_backdrop(&mut self, buf: &mut Buffer, screen: Rect) {
        // overlays first, so the page's open popups dim with the page
        self.flush_overlays(buf);
        let t = self.theme;
        let dim = Rect::new(
            screen.x,
            screen.y,
            screen.width,
            screen.height.saturating_sub(1),
        );
        for pos in dim.positions() {
            if let Some(c) = buf.cell_mut(pos) {
                let st = t.backdrop(c.style());
                c.set_style(st);
                c.modifier = ratatui::style::Modifier::empty();
            }
        }
        self.begin_modal(buf);
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
