//! The update-phase context (`COMPONENT_ARCHITECTURE.md` §17.0 A2, §21 items 6, 18).
//!
//! `Cx<'f>` holds the frozen intent queue separately from its mutable
//! services, so `IntentIter<'f>` never locks `Cx` and a component may call
//! any `&mut self` service inside its drain loop.

use core::time::Duration;

use ratatui_core::layout::{Position, Rect};

use crate::action::ActionKey;
use crate::capture::{Capture, CaptureSlot};
use crate::diagnostics::Diagnostics;
use crate::event::Chord;
use crate::focus::FocusRing;
use crate::hit::Registry;
use crate::id::{Id, PartRef};
use crate::intent::{IntentIter, IntentQueue};
use crate::keymap::{BindingRegistry, KeyMap};
use crate::layer::{Anchor, DismissReason, LayerEvent, LayerId, LayerSize, LayerSpec, LayerStack};
use crate::response::StateFlags;
use crate::theme::{DesignTokens, Theme};

use super::UiCore;
use super::derived::DerivedCache;

/// Draw-time facts a component reports upward (§4 S6).
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct LayoutFacts {
    /// Visible items or rows.
    pub viewport_len: usize,
    /// Total items or rows.
    pub content_len: usize,
    /// Rows the component occupied.
    pub rows: u16,
    /// Columns the component occupied.
    pub cols: u16,
}

impl LayoutFacts {
    /// Facts for a scrolling collection.
    pub const fn new(viewport_len: usize, content_len: usize, rows: u16, cols: u16) -> Self {
        LayoutFacts {
            viewport_len,
            content_len,
            rows,
            cols,
        }
    }
}

/// Runtime-resolved interaction state at the start of the frame.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Snapshot {
    pub(crate) focus: Option<Id>,
    pub(crate) focus_visible: bool,
    pub(crate) hover: Option<(Id, PartRef)>,
    pub(crate) hover_suppressed: bool,
    pub(crate) pressed: Option<(Id, PartRef)>,
    pub(crate) capture: Option<Id>,
}

/// Last frame's facts: geometry, layout, declared flags and the snapshot.
#[derive(Debug, Default, Clone)]
pub(crate) struct LastFrame {
    pub(crate) registry: Registry,
    pub(crate) ring: FocusRing,
    pub(crate) layout: Vec<(Id, LayoutFacts)>,
    pub(crate) declared: Vec<(Id, StateFlags)>,
    pub(crate) bindings: BindingRegistry,
    pub(crate) snapshot: Snapshot,
}

impl LastFrame {
    /// Runtime-resolved flags for `id` (§17.0 A2 `FrameRead::state`).
    pub(crate) fn state(&self, id: Id) -> StateFlags {
        let s = self.snapshot;
        let mut f = StateFlags::empty();
        if s.focus == Some(id) {
            f |= StateFlags::FOCUSED;
            if s.focus_visible {
                f |= StateFlags::FOCUS_VISIBLE;
            }
        }
        if s.hover.is_some_and(|(owner, _)| owner == id) && !s.hover_suppressed {
            f |= StateFlags::HOVERED;
        }
        if s.pressed.is_some_and(|(owner, _)| owner == id) || s.capture == Some(id) {
            f |= StateFlags::PRESSED;
        }
        if self.ring.entry(id).is_some_and(|e| e.disabled) {
            f |= StateFlags::DISABLED;
        }
        if let Some((_, d)) = self.declared.iter().find(|(i, _)| *i == id) {
            f |= *d;
        }
        f
    }

    pub(crate) fn layout_of(&self, id: Id) -> Option<LayoutFacts> {
        self.layout
            .iter()
            .rev()
            .find(|(i, _)| *i == id)
            .map(|(_, l)| *l)
    }

    /// The hovered part of `owner`, unless keyboard input currently suppresses
    /// hover styling.
    pub(crate) fn hovered_part(&self, owner: Id) -> Option<PartRef> {
        if self.snapshot.hover_suppressed {
            return None;
        }
        self.snapshot
            .hover
            .filter(|(id, _)| *id == owner)
            .map(|(_, part)| part)
    }

    pub(crate) fn pressed_part(&self, owner: Id) -> Option<PartRef> {
        self.snapshot
            .pressed
            .filter(|(id, _)| *id == owner)
            .map(|(_, part)| part)
    }
}

/// Shared read accessors — one vocabulary for both phases.
pub trait FrameRead {
    /// Runtime-resolved focus / hover / press / disabled flags for `id`,
    /// plus whatever `id` declared last frame.
    fn state(&self, id: Id) -> StateFlags;
    /// The hovered sub-region of `owner`, or `None` when no live hover matches.
    /// Keyboard input suppresses this result until the pointer moves.
    fn hovered_part(&self, _owner: Id) -> Option<PartRef> {
        None
    }
    /// The pressed or captured sub-region of `owner`.
    fn pressed_part(&self, _owner: Id) -> Option<PartRef> {
        None
    }
    /// The theme.
    fn theme(&self) -> &Theme;
    /// The design tokens.
    fn design(&self) -> &DesignTokens;
    /// LAST frame's geometry; `None` on frame 1 or when `id` did not draw.
    fn area(&self, id: Id) -> Option<Rect>;
    /// LAST frame's layout facts for `id`.
    fn layout(&self, id: Id) -> Option<LayoutFacts>;
}

/// Mutable services `Cx` exposes; owned by the runtime.
#[derive(Debug, Default)]
pub(crate) struct FrameServices {
    pub(crate) layers: LayerStack,
    pub(crate) capture: CaptureSlot,
    pub(crate) events: Vec<(Id, LayerEvent)>,
    pub(crate) focus_request: Option<Id>,
    pub(crate) repaint: bool,
    pub(crate) repaint_after: Option<Duration>,
    pub(crate) quit: bool,
    #[cfg_attr(
        not(feature = "testing"),
        expect(dead_code, reason = "filled by `Cx::record` under the testing feature")
    )]
    pub(crate) records: Vec<&'static str>,
    pub(crate) diagnostics: Diagnostics,
    pub(crate) closed_layers: Vec<crate::layer::OpenLayer>,
    pub(crate) registry_gen: u32,
    /// Where the pointer was at the last button-down. `Cx::capture` uses it
    /// as the claim's `origin`, so `pos - origin` is the press offset inside
    /// the thumb rather than the offset from the region's top-left (MA-5).
    pub(crate) press_pos: Option<Position>,
}

/// The update-phase context.
pub struct Cx<'f> {
    intents: &'f IntentQueue,
    services: &'f mut FrameServices,
    cache: &'f mut DerivedCache,
    keymap: &'f KeyMap,
    last: &'f LastFrame,
    theme: &'f Theme,
    command: Option<ActionKey>,
}

impl core::fmt::Debug for Cx<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Cx")
            .field("queue_empty", &self.intents.is_empty())
            .field("top_layer", &self.top_layer())
            .field("command", &self.command)
            .finish_non_exhaustive()
    }
}

impl<'f> Cx<'f> {
    pub(crate) fn new(
        intents: &'f IntentQueue,
        services: &'f mut FrameServices,
        core: &'f mut UiCore,
        last: &'f LastFrame,
        theme: &'f Theme,
        command: Option<ActionKey>,
    ) -> Self {
        let cache = &mut core.cache;
        let keymap = &core.keymap;
        Cx {
            intents,
            services,
            cache,
            keymap,
            last,
            theme,
            command,
        }
    }

    /// This owner's intents for the frame. Borrows only the frozen queue;
    /// an empty queue costs one `bool` check.
    pub fn intents(&self, id: Id) -> IntentIter<'f> {
        self.intents.iter(id)
    }

    pub(crate) fn claim_binding_chord(&self, owner: Id, chord: Chord) -> Option<ActionKey> {
        self.intents.claim_binding_chord(owner, chord)
    }

    pub(crate) fn swallows_typing(&self, owner: Id) -> bool {
        self.last
            .ring
            .entry(owner)
            .is_some_and(|entry| entry.swallows_typing)
    }

    pub(crate) fn effective_chord(
        &self,
        owner: Id,
        action: ActionKey,
        default: Option<Chord>,
    ) -> Option<Chord> {
        self.keymap.component_chord(owner, action, default)
    }

    /// A component's runtime-owned derived cache together with its intents.
    ///
    /// Crate-private because cached values are implementation details, never
    /// semantic application state. Returning both borrows the frozen queue
    /// and the independent cache at once without allocating an intent copy.
    pub(crate) fn intents_with_cache<T: Default + 'static>(
        &mut self,
        id: Id,
    ) -> (IntentIter<'f>, &mut T) {
        (self.intents.iter(id), self.cache.get_mut::<T>(id))
    }

    /// A component's runtime-owned derived cache.
    pub(crate) fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.cache.get_mut::<T>(id)
    }

    /// The application `KeyMap` command matched this pass, if any.
    pub const fn command(&self) -> Option<ActionKey> {
        self.command
    }

    /// Stage a focus transition (applied after this pass, §3.3 step 7).
    pub fn focus(&mut self, id: Id) {
        self.services.focus_request = Some(id);
    }

    /// Ask for a repaint regardless of the returned `Response`.
    pub fn request_repaint(&mut self) {
        self.services.repaint = true;
    }

    /// Ask for a repaint after `d`.
    pub fn request_repaint_after(&mut self, d: Duration) {
        self.services.repaint_after = Some(match self.services.repaint_after {
            Some(cur) => cur.min(d),
            None => d,
        });
    }

    /// Claim pointer capture; `false` if another capture is live.
    pub fn capture(&mut self, owner: Id, part: PartRef) -> bool {
        let area = self
            .last
            .registry
            .area_of_part(owner, part)
            .or_else(|| self.last.registry.area_of(owner))
            .unwrap_or_default();
        // §8.2: the origin is where the pointer *was*, so a splitter or a
        // scrollbar thumb computes `pos - origin` without the press offset
        // inside the thumb leaking into the delta (MA-5).
        let origin = self
            .services
            .press_pos
            .unwrap_or_else(|| Position::new(area.x, area.y));
        self.services.capture.claim(Capture {
            owner,
            part,
            origin,
            area,
            generation: self.services.registry_gen,
        })
    }

    /// The capturing owner, if any.
    pub fn capture_owner(&self) -> Option<Id> {
        self.services.capture.get().map(|c| c.owner)
    }

    /// Release the live capture.
    pub fn release_capture(&mut self) {
        self.services.capture.release();
    }

    /// Where the live capture began.
    pub fn capture_origin(&self) -> Option<Position> {
        self.services.capture.get().map(|c| c.origin)
    }

    /// The live capture's area.
    pub fn capture_area(&self) -> Option<Rect> {
        self.services.capture.get().map(|c| c.area)
    }

    /// Open a layer; assigns its `LayerId` (§21 item 14). The current focus
    /// becomes the restore target when `spec.restore_focus`.
    pub fn open_layer(&mut self, id: Id, spec: LayerSpec) {
        let restore = if spec.restore_focus {
            self.last.snapshot.focus.or(self.services.focus_request)
        } else {
            None
        };
        if self.services.layers.open(id, spec, restore).is_some()
            && let Some(f) = spec.initial_focus
        {
            self.services.focus_request = Some(f);
        }
    }

    /// Update an open layer's requested size (Adjudication N1).
    ///
    /// No-op when `id` is not open or the size is unchanged; the next `draw`
    /// re-resolves the anchor, so a size asserted in `update` takes effect in
    /// the very same frame. Safe to call unconditionally every frame — that
    /// is the intended use: the component that owns the content re-asserts
    /// its size, and a description that grows or a theme swap corrects the
    /// layer without the opener predicting anything.
    pub fn resize_layer(&mut self, id: Id, size: LayerSize) {
        if let Some(spec) = self.services.layers.spec_mut(id)
            && spec.size != size
        {
            spec.size = size;
            self.services.repaint = true;
        }
    }

    /// Update an open layer's anchor (a popover whose owner moved).
    /// No-op when `id` is not open or the anchor is unchanged.
    pub fn reanchor_layer(&mut self, id: Id, anchor: Anchor) {
        if let Some(spec) = self.services.layers.spec_mut(id)
            && spec.anchor != anchor
        {
            spec.anchor = anchor;
            self.services.repaint = true;
        }
    }

    /// Close a layer with an action (`Closed(key)`) or without
    /// (`Dismissed(Programmatic)`).
    pub fn close_layer(&mut self, id: Id, with: Option<ActionKey>) {
        let ev = match with {
            Some(k) => LayerEvent::Closed(k),
            None => LayerEvent::Dismissed(DismissReason::Programmatic),
        };
        let closed = self.services.layers.close(id, ev);
        self.services.closed_layers.extend(closed);
    }

    /// Take the pending lifecycle event of layer `id`.
    pub fn layer_event(&mut self, id: Id) -> Option<LayerEvent> {
        let pos = self.services.events.iter().position(|(i, _)| *i == id)?;
        Some(self.services.events.remove(pos).1)
    }

    /// The top of the layer stack.
    pub fn top_layer(&self) -> LayerId {
        self.services.layers.top()
    }

    /// Whether layer `id` is open.
    pub fn is_open(&self, id: Id) -> bool {
        self.services.layers.is_open(id)
    }

    /// Ask the runtime loop to exit.
    pub fn quit(&mut self) {
        self.services.quit = true;
    }

    /// Record a tag for tests (`Runtime::records`).
    #[cfg(feature = "testing")]
    pub fn record(&mut self, tag: &'static str) {
        self.services.records.push(tag);
    }
}

impl FrameRead for Cx<'_> {
    fn state(&self, id: Id) -> StateFlags {
        self.last.state(id)
    }

    fn hovered_part(&self, owner: Id) -> Option<PartRef> {
        self.last.hovered_part(owner)
    }

    fn pressed_part(&self, owner: Id) -> Option<PartRef> {
        self.last.pressed_part(owner)
    }

    fn theme(&self) -> &Theme {
        self.theme
    }

    fn design(&self) -> &DesignTokens {
        &self.theme.design
    }

    fn area(&self, id: Id) -> Option<Rect> {
        self.last.registry.area_of(id)
    }

    fn layout(&self, id: Id) -> Option<LayoutFacts> {
        self.last.layout_of(id)
    }
}

impl super::Ui<'_> {
    /// The hovered sub-region of `owner`, or `None` when keyboard input
    /// currently suppresses hover styling.
    pub fn hovered_part(&self, owner: Id) -> Option<PartRef> {
        FrameRead::hovered_part(self, owner)
    }

    /// The pressed or captured sub-region of `owner`.
    pub fn pressed_part(&self, owner: Id) -> Option<PartRef> {
        FrameRead::pressed_part(self, owner)
    }
}
