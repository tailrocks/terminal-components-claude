use core::time::Duration;

use ratatui_core::layout::{Position, Rect};

use crate::action::ActionKey;
use crate::author::VerticalWindow;
use crate::capture::{Capture, CaptureSlot};
use crate::diagnostics::Diagnostics;
use crate::event::Chord;
use crate::focus::{FocusRing, ScopeId};
use crate::hit::Registry;
use crate::id::{Id, PartRef};
use crate::intent::{IntentIter, IntentQueue};
use crate::keymap::{BindingRegistry, KeyMap, ScopedBubbleBinding};
use crate::layer::{
    Anchor, DismissReason, FocusContinuationOutcome, LayerCompletion, LayerEvent,
    LayerEventReceipt, LayerId, LayerOpening, LayerSize, LayerSpec, LayerStack, UpdateToken,
};
use crate::response::StateFlags;
use crate::runtime::UpdateCause;
use crate::theme::{DesignTokens, Theme};

use super::UiCore;
use super::derived::DerivedCache;

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct LayoutFacts {
    pub viewport_len: usize,
    pub content_len: usize,
    pub rows: u16,
    pub cols: u16,
    pub logical_area: Option<Rect>,
    /// The prepared, physically visible rows of a logically taller component body.
    pub vertical_window: Option<VerticalWindow>,
}

impl LayoutFacts {
    pub const fn new(viewport_len: usize, content_len: usize, rows: u16, cols: u16) -> Self {
        LayoutFacts {
            viewport_len,
            content_len,
            rows,
            cols,
            logical_area: None,
            vertical_window: None,
        }
    }

    #[must_use]
    pub const fn with_logical_area(mut self, area: Rect) -> Self {
        self.logical_area = Some(area);
        self
    }

    /// Attach parent-clipped vertical geometry to the prepared layout facts.
    #[must_use]
    pub const fn with_vertical_window(mut self, window: VerticalWindow) -> Self {
        self.vertical_window = Some(window);
        self
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Snapshot {
    pub(crate) focus: Option<Id>,
    pub(crate) focus_visible: bool,
    pub(crate) hover: Option<(Id, PartRef)>,
    pub(crate) hover_suppressed: bool,
    pub(crate) pressed: Option<(Id, PartRef)>,
    pub(crate) capture: Option<Id>,
}

#[derive(Debug, Default, Clone)]
pub struct LastFrame {
    pub(crate) registry: Registry,
    pub(crate) ring: FocusRing,
    pub(crate) layout: Vec<(Id, LayoutFacts)>,
    pub(crate) declared: Vec<(Id, StateFlags)>,
    pub(crate) bindings: BindingRegistry,
    pub(crate) scoped_bubble_bindings: Vec<ScopedBubbleBinding>,
    pub(crate) typing_bindings: BindingRegistry,
    pub(crate) typing: crate::runtime::typing::TypingResolved,
    pub(crate) snapshot: Snapshot,
}

impl LastFrame {
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

pub trait FrameRead {
    fn state(&self, id: Id) -> StateFlags;
    fn hovered_part(&self, _owner: Id) -> Option<PartRef> {
        None
    }
    fn pressed_part(&self, _owner: Id) -> Option<PartRef> {
        None
    }
    fn theme(&self) -> &Theme;
    fn design(&self) -> &DesignTokens;
    fn area(&self, id: Id) -> Option<Rect>;
    fn area_of_part(&self, _owner: Id, _part: PartRef) -> Option<Rect> {
        None
    }
    fn layout(&self, id: Id) -> Option<LayoutFacts>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DeferredFocus {
    pub(crate) anchor: Option<Id>,
    pub(crate) backwards: bool,
}

#[derive(Debug, Default)]
pub struct FrameServices {
    pub(crate) runtime_id: u64,
    pub(crate) viewport: Rect,
    pub(crate) layers: LayerStack,
    pub(crate) capture: CaptureSlot,
    pub(crate) events: Vec<LayerEventReceipt>,
    pub(crate) callback_serial: u64,
    pub(crate) focus_request: Option<Id>,
    pub(crate) initial_focus_layer: Option<LayerId>,
    pub(crate) deferred_focus: Option<DeferredFocus>,
    pub(crate) repaint: bool,
    pub(crate) feedback: crate::runtime::feedback::FeedbackState,
    pub(crate) now: crate::runtime::Moment,
    pub(crate) repaint_at: Option<crate::runtime::Moment>,
    pub(crate) quit: bool,
    #[cfg_attr(
        not(feature = "testing"),
        expect(dead_code, reason = "filled by `Cx::record` under the testing feature")
    )]
    pub(crate) records: Vec<&'static str>,
    pub(crate) diagnostics: Diagnostics,
    pub(crate) closed_layers: Vec<crate::layer::OpenLayer>,
    pub(crate) direct_closed_layers: Vec<crate::layer::OpenLayer>,
    pub(crate) completions: Vec<(LayerCompletion, u8)>,
    pub(crate) focus_continuation_outcomes: Vec<(FocusContinuationOutcome, u8)>,
    pub(crate) focus_continuation_acks: Vec<(LayerOpening, Id, Id)>,
    pub(crate) focus_continuation_result_acks: Vec<(LayerOpening, Id, bool)>,
    pub(crate) focus_result_ack_dispatch_pending: bool,
    pub(crate) continuation_command_pending_present: Option<Id>,
    pub(crate) registry_gen: u32,
    pub(crate) press_pos: Option<Position>,
    pub(crate) clipboard: crate::runtime::clipboard::Clipboard,
    pub(crate) pointer_press_count: Option<(Id, PartRef, u8)>,
    pub(crate) reset_pointer_clicks: bool,
}

pub struct Cx<'f> {
    intents: &'f IntentQueue,
    services: &'f mut FrameServices,
    cache: &'f mut DerivedCache,
    keymap: &'f KeyMap,
    last: &'f LastFrame,
    theme: &'f Theme,
    command: Option<ActionKey>,
    update_cause: UpdateCause,
    activation_key: Option<crate::runtime::ActivationKey>,
    update_token: UpdateToken,
    edit_entry: termrock_core::edit_entry::EditEntry,
}

impl core::fmt::Debug for Cx<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Cx")
            .field("queue_empty", &self.intents.is_empty())
            .field("top_layer", &self.top_layer())
            .field("command", &self.command)
            .field("update_cause", &self.update_cause)
            .finish_non_exhaustive()
    }
}

/// Advance the serial that identifies one update within a runtime.
///
/// The serial saturates at the end of its range instead of aborting the
/// process: only 2^64 updates in one runtime reach the limit.
fn next_callback_serial(serial: &mut u64) -> u64 {
    *serial = serial.saturating_add(1);
    *serial
}

impl<'f> Cx<'f> {
    /// Append a component-author diagnostic to the runtime's bounded diagnostic
    /// stream. Components cannot clear diagnostics emitted by other owners.
    pub fn diagnostic(&mut self, diagnostic: crate::diagnostics::Diagnostic) {
        self.services.diagnostics.push(diagnostic);
    }

    fn invalidate_continuation_result_readiness(&mut self) {
        if !self.services.focus_continuation_result_acks.is_empty() {
            for (_, _, ready) in &mut self.services.focus_continuation_result_acks {
                *ready = false;
            }
            self.services.focus_result_ack_dispatch_pending = true;
        }
    }

    #[cfg(test)]
    pub fn new(
        intents: &'f IntentQueue,
        services: &'f mut FrameServices,
        core: &'f mut UiCore,
        last: &'f LastFrame,
        theme: &'f Theme,
        command: Option<ActionKey>,
    ) -> Self {
        Self::new_with_cause(
            intents,
            services,
            core,
            last,
            theme,
            command,
            UpdateCause::Event,
        )
    }

    pub fn new_with_cause(
        intents: &'f IntentQueue,
        services: &'f mut FrameServices,
        core: &'f mut UiCore,
        last: &'f LastFrame,
        theme: &'f Theme,
        command: Option<ActionKey>,
        update_cause: UpdateCause,
    ) -> Self {
        // The callback serial identifies one update within a runtime. It
        // saturates only after 2^64 updates, and saturating keeps the token
        // monotone instead of aborting the process.
        let callback_serial = next_callback_serial(&mut services.callback_serial);
        let update_token = UpdateToken::new(services.runtime_id, callback_serial);
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
            update_cause,
            activation_key: None,
            update_token,
            edit_entry: termrock_core::edit_entry::EditEntry::Immediate,
        }
    }

    pub(crate) fn with_activation_key(
        mut self,
        key: Option<crate::runtime::ActivationKey>,
    ) -> Self {
        if self.update_cause == UpdateCause::Event {
            self.activation_key = key;
        }
        self
    }

    pub fn intents(&self, id: Id) -> IntentIter<'f> {
        self.intents.iter(id)
    }

    pub fn claim_binding_chord(&self, owner: Id, chord: Chord) -> Option<ActionKey> {
        self.intents.claim_binding_chord(owner, chord)
    }

    pub fn swallows_typing(&self, owner: Id) -> bool {
        self.last
            .ring
            .entry(owner)
            .is_some_and(|entry| entry.swallows_typing)
    }

    pub fn effective_chord(
        &self,
        owner: Id,
        action: ActionKey,
        default: Option<Chord>,
    ) -> Option<Chord> {
        self.keymap.component_chord(owner, action, default)
    }

    pub fn intents_with_cache<T: Default + 'static>(&mut self, id: Id) -> (IntentIter<'f>, &mut T) {
        (self.intents.iter(id), self.cache.get_mut::<T>(id))
    }

    pub fn cache<T: Default + 'static>(&mut self, id: Id) -> &mut T {
        self.cache.get_mut::<T>(id)
    }

    pub const fn command(&self) -> Option<ActionKey> {
        self.command
    }

    pub const fn update_cause(&self) -> UpdateCause {
        self.update_cause
    }

    /// App-wide default for text editor edit-entry behavior.
    pub const fn edit_entry(&self) -> termrock_core::edit_entry::EditEntry {
        self.edit_entry
    }

    pub(crate) const fn with_edit_entry(
        mut self,
        edit_entry: termrock_core::edit_entry::EditEntry,
    ) -> Self {
        self.edit_entry = edit_entry;
        self
    }

    pub const fn activation_key(&self) -> Option<crate::runtime::ActivationKey> {
        self.activation_key
    }

    pub fn focus(&mut self, id: Id) {
        self.invalidate_continuation_result_readiness();
        self.services.initial_focus_layer = None;
        self.services.deferred_focus = None;
        self.services.focus_request = Some(id);
    }

    pub fn focus_next(&mut self) {
        self.invalidate_continuation_result_readiness();
        self.services.initial_focus_layer = None;
        self.services.focus_request = None;
        self.services.deferred_focus = Some(DeferredFocus {
            anchor: self.last.snapshot.focus,
            backwards: false,
        });
        self.services.repaint = true;
    }

    pub fn activation_feedback(&self) -> Option<crate::runtime::ActivationFeedback> {
        self.services.feedback.active(self.services.now)
    }

    pub fn sync_feedback_time(
        &mut self,
        now: crate::runtime::SimulationMoment,
    ) -> Result<(), crate::runtime::FeedbackClockError> {
        if self.services.feedback.sync(now)? {
            self.services.repaint = true;
        }
        Ok(())
    }

    pub fn flash_activation(&mut self, owner: Id) {
        self.flash_activation_part(owner, PartRef::of(crate::id::Part::CONTAINER));
    }

    pub fn flash_activation_part(&mut self, owner: Id, part: PartRef) {
        self.services.feedback.activate(
            owner,
            part,
            self.services.now,
            Duration::from_millis(self.theme.design.motion.press_flash_ms),
        );
        self.services.repaint = true;
    }

    pub fn focus_prev(&mut self) {
        self.invalidate_continuation_result_readiness();
        self.services.initial_focus_layer = None;
        self.services.focus_request = None;
        self.services.deferred_focus = Some(DeferredFocus {
            anchor: self.last.snapshot.focus,
            backwards: true,
        });
        self.services.repaint = true;
    }

    pub fn request_repaint(&mut self) {
        self.services.repaint = true;
    }

    pub const fn viewport(&self) -> Rect {
        self.services.viewport
    }

    /// Return text copied through this runtime's shared clipboard service.
    pub fn clipboard_text(&self) -> Option<&str> {
        self.services.clipboard.text()
    }

    /// Copy text into this runtime's shared clipboard service.
    pub fn clipboard_copy(&mut self, text: &str) {
        self.services.clipboard.copy(text);
    }

    /// Clear and wipe this runtime's shared clipboard contents.
    pub fn clipboard_clear(&mut self) {
        self.services.clipboard.clear();
    }

    /// Install an optional integration hook for copied text.
    pub fn set_clipboard_copy_hook(&mut self, hook: impl Fn(&str) + 'static) {
        self.services.clipboard.set_copy_hook(hook);
    }

    /// Remove the optional integration hook for copied text.
    pub fn clear_clipboard_copy_hook(&mut self) {
        self.services.clipboard.clear_copy_hook();
    }

    pub const fn now(&self) -> crate::runtime::Moment {
        self.services.now
    }

    pub fn request_repaint_at(&mut self, deadline: crate::runtime::Moment) {
        self.services.repaint_at = Some(
            self.services
                .repaint_at
                .map_or(deadline, |current| current.min(deadline)),
        );
    }

    pub fn request_repaint_after(&mut self, d: Duration) {
        self.request_repaint_at(self.now().saturating_add(d));
    }

    pub fn capture(&mut self, owner: Id, part: PartRef) -> bool {
        let Some(area) = crate::capture::target_area(
            &self.last.registry,
            &self.last.ring,
            self.top_layer(),
            owner,
            part,
        ) else {
            return false;
        };

        let Some(origin) = self.services.press_pos else {
            return false;
        };
        self.services.capture.claim(Capture {
            owner,
            part,
            origin,
            area,
            generation: self.services.registry_gen,
        })
    }

    pub fn capture_owner(&self) -> Option<Id> {
        self.services.capture.get().map(|c| c.owner)
    }

    /// Return the click count for this component's current press or release intent.
    pub fn pointer_press_count(&self, owner: Id, part: PartRef) -> Option<u8> {
        self.services
            .pointer_press_count
            .filter(|(current_owner, current_part, _)| {
                *current_owner == owner && *current_part == part
            })
            .map(|(_, _, count)| count)
    }

    /// Return the click count for this component's current click intent.
    pub fn pointer_click_count(&self, owner: Id, part: PartRef) -> Option<u8> {
        self.pointer_press_count(owner, part)
    }

    pub fn release_capture(&mut self) {
        self.services.capture.release();
    }

    pub fn capture_origin(&self) -> Option<Position> {
        self.services.capture.get().map(|c| c.origin)
    }

    pub fn capture_area(&self) -> Option<Rect> {
        self.services.capture.get().map(|c| c.area)
    }

    pub fn open_layer(&mut self, id: Id, spec: LayerSpec) {
        let _ = self.open_layer_with_token(id, spec);
    }

    pub fn open_layer_with_token(&mut self, id: Id, spec: LayerSpec) -> Option<LayerOpening> {
        self.invalidate_continuation_result_readiness();
        let restore = if spec.restore_focus {
            self.last.snapshot.focus.or(self.services.focus_request)
        } else {
            None
        };
        let opening =
            self.services
                .layers
                .open_with_runtime(id, spec, restore, self.services.runtime_id);
        if let Some(opening) = opening {
            self.services.reset_pointer_clicks = true;
            self.services.initial_focus_layer = None;
            if let Some(target) = spec.initial_focus {
                self.focus(target);
                self.services.initial_focus_layer = Some(opening.layer_id());
            }
        }
        opening
    }

    pub fn layer_opening(&self, id: Id) -> Option<LayerOpening> {
        self.services
            .layers
            .get(id)
            .map(|layer| layer.opening)
            .filter(|opening| opening.id() == id && opening.runtime() == self.services.runtime_id)
    }

    /// Consume a completion receipt for this exact layer opening, if focus restoration settled.
    pub fn take_layer_completion(&mut self, opening: LayerOpening) -> Option<LayerCompletion> {
        let position = self
            .services
            .completions
            .iter()
            .position(|(completion, _)| completion.opening() == opening)?;
        let candidate = self.services.completions[position].0;
        if candidate.continuation_acknowledged()
            && let Some(owner) = candidate.continuation_owner()
            && self
                .services
                .focus_continuation_result_acks
                .iter()
                .find(|(ack_opening, ack_owner, _)| *ack_opening == opening && *ack_owner == owner)
                .is_none_or(|(_, _, ready)| !*ready)
            && candidate.restored_focus() == Some(owner)
            && self.continuation_owner_is_current(opening, owner)
        {
            return None;
        }
        let completion = self.services.completions.remove(position).0;
        let still_continued = completion.continuation_owner().is_some_and(|owner| {
            completion.continuation_acknowledged()
                && completion.restored_focus() == Some(owner)
                && self.continuation_owner_is_current(opening, owner)
        });
        if let Some(owner) = completion.continuation_owner() {
            self.services
                .focus_continuation_result_acks
                .retain(|(ack_opening, ack_owner, _)| {
                    *ack_opening != opening || *ack_owner != owner
                });
            if still_continued {
                self.services.continuation_command_pending_present = Some(owner);
            }
        }
        Some(completion.with_continuation_acknowledged(still_continued))
    }

    fn continuation_owner_is_current(&self, opening: LayerOpening, owner: Id) -> bool {
        if opening.runtime() != self.services.runtime_id
            || self.services.layers.get(opening.id()).is_some()
            || self.last.snapshot.focus != Some(owner)
        {
            return false;
        }
        let (parent_layer, parent_is_current) = match opening.parent() {
            None => (LayerId::PAGE, self.services.layers.top() == LayerId::PAGE),
            Some(parent) => {
                let current = self.services.layers.layers().iter().any(|layer| {
                    layer.id == parent.id
                        && layer.layer == parent.layer
                        && layer.opening.generation() == parent.generation
                        && self.services.layers.top() == layer.layer
                });
                (parent.layer, current)
            }
        };
        parent_is_current
            && self
                .last
                .ring
                .entry(owner)
                .is_some_and(|entry| entry.layer == parent_layer && !entry.disabled)
    }

    /// Return the exact child-layer opening that owns this pending focus continuation.
    pub fn focus_continuation_opening(&self, owner: Id, to: Id) -> Option<LayerOpening> {
        self.focus_is_continued(owner, to)
            .then(|| self.services.layers.top_layer().map(|layer| layer.opening))
            .flatten()
    }

    /// Observe the settled result for an exact continuation without consuming it.
    pub fn focus_continuation_result(
        &self,
        opening: LayerOpening,
        owner: Id,
    ) -> Option<FocusContinuationOutcome> {
        self.services
            .focus_continuation_outcomes
            .iter()
            .map(|(outcome, _)| *outcome)
            .find(|outcome| outcome.opening() == opening && outcome.owner() == owner)
            .map(|outcome| {
                outcome.with_continued(
                    outcome.continued()
                        && outcome.restored_focus() == Some(owner)
                        && self.continuation_owner_is_current(opening, owner),
                )
            })
    }

    /// Acknowledge the exact continuation result after the owner applies its resume or blur.
    pub fn acknowledge_focus_continuation_result(
        &mut self,
        opening: LayerOpening,
        owner: Id,
    ) -> bool {
        let Some(position) = self
            .services
            .focus_continuation_outcomes
            .iter()
            .position(|(outcome, _)| outcome.opening() == opening && outcome.owner() == owner)
        else {
            return false;
        };
        self.services.focus_continuation_outcomes.remove(position);
        if self.services.completions.iter().any(|(completion, _)| {
            completion.opening() == opening
                && completion.continuation_owner() == Some(owner)
                && completion.continuation_acknowledged()
        }) {
            if self.services.focus_continuation_result_acks.len() == 64 {
                let oldest = self.services.focus_continuation_result_acks.remove(0);
                self.services.diagnostics.push(
                    crate::diagnostics::Diagnostic::FocusContinuationUnacknowledged {
                        owner: oldest.1,
                    },
                );
            }
            self.services
                .focus_continuation_result_acks
                .push((opening, owner, false));
        }
        self.services.focus_result_ack_dispatch_pending = true;
        true
    }

    /// Whether the exact focus-out transition is owned by the active child layer.
    pub fn focus_is_continued(&self, owner: Id, to: Id) -> bool {
        if !self.intents(owner).any(|intent| {
            matches!(intent, crate::intent::Intent::FocusOut { to: Some(target) } if target == to)
        }) {
            return false;
        }
        let Some(top) = self.services.layers.top_layer() else {
            return false;
        };
        if top.spec.continues_focus_owner != Some(owner) {
            return false;
        }
        let parent_layer = match top.opening.parent() {
            None => Some(LayerId::PAGE),
            Some(parent) => self
                .services
                .layers
                .layers()
                .iter()
                .find(|layer| {
                    layer.layer == parent.layer
                        && layer.id == parent.id
                        && layer.opening.generation() == parent.generation
                })
                .map(|layer| layer.layer),
        };
        let Some(parent_layer) = parent_layer else {
            return false;
        };
        if !self
            .last
            .ring
            .entry(owner)
            .is_some_and(|entry| !entry.disabled && entry.layer == parent_layer)
        {
            return false;
        }
        self.last
            .ring
            .entry(to)
            .is_some_and(|entry| !entry.disabled && entry.layer == top.layer)
            || (top.spec.initial_focus == Some(to) && self.last.snapshot.focus == Some(to))
    }

    /// Acknowledge that the owning component reconciled state for this exact continuation.
    pub fn acknowledge_focus_continuation(&mut self, owner: Id, to: Id) -> bool {
        if !self.focus_is_continued(owner, to) {
            return false;
        }
        let Some(opening) = self.services.layers.top_layer().map(|layer| layer.opening) else {
            return false;
        };
        if !self
            .services
            .focus_continuation_acks
            .contains(&(opening, owner, to))
        {
            self.services
                .focus_continuation_acks
                .push((opening, owner, to));
        }
        true
    }

    pub fn resize_layer(&mut self, id: Id, size: LayerSize) {
        if let Some(spec) = self.services.layers.spec_mut(id)
            && spec.size != size
        {
            spec.size = size;
            self.services.repaint = true;
        }
    }

    pub fn reanchor_layer(&mut self, id: Id, anchor: Anchor) {
        if let Some(spec) = self.services.layers.spec_mut(id)
            && spec.anchor != anchor
        {
            spec.anchor = anchor;
            self.services.repaint = true;
        }
    }

    pub fn close_layer(&mut self, id: Id, with: Option<ActionKey>) {
        self.invalidate_continuation_result_readiness();
        let ev = match with {
            Some(k) => LayerEvent::Closed(k),
            None => LayerEvent::Dismissed(DismissReason::Programmatic),
        };
        let closed = self
            .services
            .layers
            .close_with_update(id, ev, Some(self.update_token));
        if !closed.is_empty() {
            self.services.reset_pointer_clicks = true;
            self.services.direct_closed_layers.push(closed[0]);
        }
        if self
            .services
            .initial_focus_layer
            .is_some_and(|id| closed.iter().any(|layer| layer.layer == id))
        {
            self.services.initial_focus_layer = None;
        }
        self.services.closed_layers.extend(closed);
    }

    pub fn layer_event(&mut self, id: Id) -> Option<LayerEvent> {
        let pos = self
            .services
            .events
            .iter()
            .position(|receipt| receipt.opening().id() == id)?;
        Some(self.services.events.remove(pos).event())
    }

    /// Takes the next pending event receipt for this component's exact layer id.
    pub fn layer_event_receipt(&mut self, id: Id) -> Option<LayerEventReceipt> {
        let pos = self
            .services
            .events
            .iter()
            .position(|receipt| receipt.opening().id() == id)?;
        Some(self.services.events.remove(pos))
    }

    /// Claims the next exact layer event receipt and only its matching layer
    /// intent. Unrelated intents owned by the layer remain available to their
    /// component and continue to participate in undelivered-intent checks.
    pub fn take_layer_intent_receipt(&mut self, id: Id) -> Option<LayerEventReceipt> {
        let pos = self
            .services
            .events
            .iter()
            .position(|receipt| receipt.opening().id() == id)?;
        let receipt = *self.services.events.get(pos)?;
        if !self.intents.claim_next_layer_event(id, receipt.event()) {
            return None;
        }
        Some(self.services.events.remove(pos))
    }

    /// Returns the opaque identity of this application update callback.
    pub const fn update_token(&self) -> UpdateToken {
        self.update_token
    }

    pub fn top_layer(&self) -> LayerId {
        self.services.layers.top()
    }

    /// Return the currently focused registered control when it belongs to
    /// `scope` on the topmost interactive layer.
    pub fn focused_in_scope(&self, scope: ScopeId) -> Option<Id> {
        let focused = self.last.snapshot.focus?;
        let entry = self.last.ring.entry(focused)?;
        let scope_layer = self.last.ring.scope_layer(scope)?;
        let top = self.services.layers.top();
        (entry.id == focused
            && !entry.disabled
            && self.last.ring.contains(focused)
            && self.last.ring.within(entry.scope, scope)
            && entry.layer == scope_layer
            && entry.layer == top)
            .then_some(focused)
    }

    pub fn is_open(&self, id: Id) -> bool {
        self.services.layers.is_open(id)
    }

    pub fn quit(&mut self) {
        self.services.quit = true;
    }

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

    fn area_of_part(&self, owner: Id, part: PartRef) -> Option<Rect> {
        self.last.registry.area_of_part(owner, part)
    }

    fn layout(&self, id: Id) -> Option<LayoutFacts> {
        self.last.layout_of(id)
    }
}

impl super::Ui<'_> {
    pub fn hovered_part(&self, owner: Id) -> Option<PartRef> {
        FrameRead::hovered_part(self, owner)
    }

    pub fn pressed_part(&self, owner: Id) -> Option<PartRef> {
        FrameRead::pressed_part(self, owner)
    }
}

#[cfg(test)]
mod diagnostics_tests {
    use super::*;
    use crate::intent::IntentQueue;

    #[test]
    fn component_author_diagnostics_use_the_bounded_runtime_store() {
        let intents = IntentQueue::new();
        let mut services = FrameServices::default();
        let mut core = UiCore::default();
        let last = LastFrame::default();
        let theme = Theme::junie();
        let owner = Id::root("form");
        let field = Id::root("name");
        {
            let mut cx = Cx::new(&intents, &mut services, &mut core, &last, &theme, None);
            for _ in 0..70 {
                cx.diagnostic(crate::diagnostics::Diagnostic::InvalidComponentLayout {
                    owner,
                    field: Some(field),
                });
            }
        }
        assert_eq!(services.diagnostics.items().len(), 64);
        assert_eq!(services.diagnostics.dropped(), 6);
        assert!(matches!(
            services.diagnostics.items().first(),
            Some(crate::diagnostics::Diagnostic::InvalidComponentLayout {
                owner: found_owner,
                field: Some(found_field),
            }) if *found_owner == owner && *found_field == field
        ));
    }
}

#[cfg(test)]
mod callback_serial_tests {
    use super::*;

    #[test]
    fn the_callback_serial_saturates_instead_of_aborting_the_process() {
        let mut serial = 0;
        assert_eq!(next_callback_serial(&mut serial), 1);
        assert_eq!(next_callback_serial(&mut serial), 2);
        serial = u64::MAX;
        assert_eq!(next_callback_serial(&mut serial), u64::MAX);
        assert_eq!(serial, u64::MAX);
    }
}
