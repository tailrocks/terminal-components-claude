pub mod clipboard;
pub(crate) mod feedback;
mod time;
pub use feedback::{ActivationFeedback, FeedbackClock, FeedbackClockError, SimulationMoment};
pub(crate) mod typing;
pub use time::{ClockError, Moment};
pub use typing::TypingPolicy;

use core::time::Duration;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::terminal::Frame;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::action::ActionKey;
use crate::capture::Capture;
use crate::cursor;
use crate::diagnostics::Diagnostic;
use crate::event::{Axis, Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, RoutedInput};
use crate::focus::{FocusRing, FocusState, ScopeId};
use crate::hit::{Hit, RegionKind, ScrollRouteKey, ScrollRoutePolicy, TextDragRegion};
use crate::id::{Id, Part, PartRef};
use crate::intent::{FocusVia, IntentQueue, Phase};
use crate::keymap::{BindingTableId, KeyMap, KeyPhase};
use crate::layer::{
    Backdrop, DismissReason, FocusContinuationOutcome, LayerCompletion, LayerEvent,
    LayerEventReceipt, LayerId, OpenLayer, RoutedLayerEvent, backdrop_area, resolve_layer_area,
};
use crate::measure::Size;

use crate::response::{Invalidate, Response};
use crate::theme::Theme;
use crate::ui::cx::{FrameServices, LastFrame, ManagedTooltip, WheelRouteAck};
use crate::ui::{Cx, FrameState, Ui, UiCore};

static NEXT_RUNTIME_ID: AtomicU64 = AtomicU64::new(1);

fn select_cursor_owner(
    frame: &FrameState,
    retained_request: Option<crate::ui::RetainedCursorRequest>,
    retained_cursor_owner: Option<Id>,
    focus: Option<Id>,
) -> Option<Id> {
    let visible_retention = retained_request.is_some_and(|request| {
        request.mode == crate::ui::CursorRetentionMode::VisibleOwner
            && !request.ambiguous
            && retained_cursor_owner == Some(request.owner)
    });
    if !visible_retention {
        return frame.typing_resolved.cursor.or(retained_cursor_owner);
    }

    if frame.typing_resolved.owner.is_some() {
        return frame.typing_resolved.cursor;
    }

    let focused_top_cursor_request = focus.and_then(|owner| {
        frame
            .cursors
            .iter()
            .enumerate()
            .find(|(index, request)| {
                Some(*index) != retained_request.map(|retained| retained.cursor_index)
                    && request.owner == owner
                    && request.layer == frame.top
                    && !request.inert
                    && (!request.typing_offer
                        || frame.typing.iter().any(|declaration| {
                            declaration.owner == owner
                                && declaration.layer == request.layer
                                && declaration.cursor
                        }))
            })
            .map(|(_, request)| request.owner)
    });
    focused_top_cursor_request.or(retained_cursor_owner)
}

fn next_runtime_id() -> u64 {
    NEXT_RUNTIME_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
            next.checked_add(1)
        })
        .expect("runtime identity space exhausted")
}

fn default_app_keyboard_protocol(
    explicit_override: Option<termrock_core::keys::keyboard::KeyboardProtocol>,
) -> termrock_core::keys::keyboard::KeyboardProtocol {
    explicit_override.unwrap_or(termrock_core::keys::keyboard::KeyboardProtocol::Off)
}

pub trait App {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;

    fn draw(&self, ui: &mut Ui<'_>);

    fn keyboard_protocol(&self) -> termrock_core::keys::keyboard::KeyboardProtocol {
        default_app_keyboard_protocol(
            termrock_core::keys::keyboard::KeyboardProtocol::current_override(),
        )
    }

    fn route_wheel_attempt(
        &mut self,
        _cx: &mut Cx<'_>,
        _attempt: WheelRouteAttempt,
    ) -> Response<()> {
        Response::ignored()
    }

    fn should_quit(&self) -> bool {
        false
    }

    fn keymap(&self) -> &KeyMap {
        KeyMap::EMPTY_REF
    }

    fn tooltips(&self) -> crate::Tooltips {
        crate::Tooltips::DEFAULT
    }

    fn edit_entry(&self) -> termrock_core::edit_entry::EditEntry {
        termrock_core::edit_entry::EditEntry::Immediate
    }

    fn min_size(&self) -> Size {
        Size {
            min: (72, 20),
            preferred: (120, 40),
        }
    }

    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    #[must_use = "false leaves the paste on the typing-owner path"]
    fn accept_paused_clock_step(&mut self, _step_ms: u64) -> bool {
        false
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelRouteResult {
    Ignored,

    Consumed,

    Changed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WheelRouteAttempt {
    runtime_id: u64,
    event_id: u64,
    attempt_index: u32,
    key: ScrollRouteKey,
    owner: Id,
    part: PartRef,
    layer: LayerId,
    axis: Axis,
    delta: i16,
    unscaled_delta: i16,
    position: Position,
    modifiers: KeyModifiers,
    registry_generation: u32,
    source_revision: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WheelRouteTarget {
    runtime_id: u64,
    event_id: u64,
    attempt_index: u32,
    key: ScrollRouteKey,
    owner: Id,
    part: PartRef,
    layer: LayerId,
    registry_generation: u32,
    source_revision: u64,
}

impl WheelRouteAttempt {
    const fn new(
        target: WheelRouteTarget,
        axis: Axis,
        delta: i16,
        unscaled_delta: i16,
        position: Position,
        modifiers: KeyModifiers,
    ) -> Self {
        Self {
            runtime_id: target.runtime_id,
            event_id: target.event_id,
            attempt_index: target.attempt_index,
            key: target.key,
            owner: target.owner,
            part: target.part,
            layer: target.layer,
            axis,
            delta,
            unscaled_delta,
            position,
            modifiers,
            registry_generation: target.registry_generation,
            source_revision: target.source_revision,
        }
    }

    pub const fn event_id(self) -> u64 {
        self.event_id
    }

    pub const fn attempt_index(self) -> u32 {
        self.attempt_index
    }

    pub const fn owner(self) -> Id {
        self.owner
    }

    pub const fn part(self) -> PartRef {
        self.part
    }

    pub const fn layer(self) -> LayerId {
        self.layer
    }

    pub const fn axis(self) -> Axis {
        self.axis
    }

    pub const fn delta(self) -> i16 {
        self.delta
    }

    pub const fn unscaled_delta(self) -> i16 {
        self.unscaled_delta
    }

    pub const fn position(self) -> Position {
        self.position
    }

    pub const fn modifiers(self) -> KeyModifiers {
        self.modifiers
    }

    pub const fn registry_generation(self) -> u32 {
        self.registry_generation
    }

    pub const fn source_revision(self) -> u64 {
        self.source_revision
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationKey {
    Enter,
    Space,
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateCause {
    Bootstrap,
    Event,
    Tick,
    Settle,

    ScrollRoute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Press {
    owner: Id,
    part: PartRef,
    layer: LayerId,
    area: Rect,
    dragging: bool,
    pos: Position,
    click_count: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DragMotion {
    position: Position,
    modifiers: KeyModifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LiveTextDrag {
    capture: Capture,
    press: Press,
    motion: DragMotion,
    region: TextDragRegion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HeldTextDrag {
    region: TextDragRegion,
    position: Position,
    modifiers: KeyModifiers,
    click_count: u8,
    next_due: Option<Moment>,
}

#[derive(Clone, Copy, Debug)]
struct LastClick {
    owner: Id,
    part: PartRef,
    at: Moment,
    pos: Position,
    count: u8,
}

#[derive(Clone, Copy, Debug, Default)]
struct Interaction {
    pointer_position: Option<Position>,
    hover: Option<(Id, PartRef)>,
    hover_suppressed: bool,
    hover_passes: u8,
    press: Option<Press>,
    drag_motion: Option<DragMotion>,
    last_click: Option<LastClick>,
    last_input_key: bool,
}

const MAX_FOCUS_PASSES: usize = 4;
const TEXT_DRAG_AUTOSCROLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Default)]
pub struct RenderSnapshot {
    cache_identity: std::sync::Arc<()>,
    last: LastFrame,
    layers: Vec<OpenLayer>,
    inert_floor: LayerId,
    top: LayerId,
    keymap: KeyMap,
    tooltips: crate::Tooltips,
}

impl core::fmt::Debug for RenderSnapshot {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RenderSnapshot")
            .field("focus", &self.last.snapshot.focus)
            .field("layers", &self.layers.len())
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderSnapshotError {
    Uninitialized,
    NeedsSettle,
    NeedsPresentation,
}

impl core::fmt::Display for RenderSnapshotError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Uninitialized => "runtime is not initialized",
            Self::NeedsSettle => "runtime focus needs settling",
            Self::NeedsPresentation => "runtime needs successful presentation",
        })
    }
}
impl core::error::Error for RenderSnapshotError {}

pub struct PendingInput(Input);

impl PendingInput {
    pub fn into_input(self) -> Input {
        self.0
    }
}

pub struct PendingRoutedInput(RoutedInput);

impl PendingRoutedInput {
    pub fn into_routed_input(self) -> RoutedInput {
        self.0
    }
}

impl core::fmt::Debug for PendingRoutedInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PendingRoutedInput { event: <redacted> }")
    }
}

impl core::fmt::Debug for PendingInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PendingInput { event: <redacted> }")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PreparedFrame {
    runtime: u64,
    generation: u64,
    area: Rect,
}

impl PreparedFrame {
    pub const fn area(self) -> Rect {
        self.area
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreparedFrameError {
    Stale,
    AreaChanged { expected: Rect, actual: Rect },
    DidNotSettle,
}

impl core::fmt::Display for PreparedFrameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Stale => f.write_str("prepared terminal frame is stale"),
            Self::AreaChanged { expected, actual } => {
                write!(f, "prepared frame area {expected:?} changed to {actual:?}")
            }
            Self::DidNotSettle => f.write_str("runtime did not settle before frame preparation"),
        }
    }
}

impl core::error::Error for PreparedFrameError {}

struct PreparedOutput {
    token: PreparedFrame,
    buffer: Buffer,
    cursor: Option<Position>,
    cursor_bar: bool,
    projected: bool,
}

#[must_use = "commit only after successful output; dropping aborts publication"]
pub struct PaintedFrame<'a, A: App> {
    runtime: &'a mut Runtime<A>,
    committed: bool,
}

impl<A: App> core::fmt::Debug for PaintedFrame<'_, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PaintedFrame").finish_non_exhaustive()
    }
}

impl<A: App> PaintedFrame<'_, A> {
    pub fn commit_presented(mut self) {
        self.runtime.commit_frame();
        self.committed = true;
    }

    pub const fn cursor_bar(&self) -> bool {
        self.runtime.cursor_bar()
    }
}

impl<A: App> Drop for PaintedFrame<'_, A> {
    fn drop(&mut self) {
        if !self.committed {
            self.runtime.invalidate_prepared();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConflictCacheKey {
    focused: Option<(Id, BindingTableId)>,
    typing_owner: Option<Id>,
    revision: u64,
}

#[derive(Clone, Copy, Debug)]
struct FocusRestore {
    target: Option<Id>,
}

#[derive(Clone, Copy, Debug)]
struct PendingFocus {
    from: Option<Id>,
    to: Option<Id>,
    via: FocusVia,
    from_layer: Option<LayerId>,
}

#[derive(Clone, Copy, Debug)]
struct WheelRouteGuard {
    focus: Option<Id>,
    top_layer: LayerId,
    input_layer: LayerId,
    registry_generation: u32,
    capture: Option<Capture>,
}

#[derive(Clone, Copy, Debug)]
struct DeferredTypeAheadKey {
    key: Key,
    owner: Id,
    owner_layer: LayerId,
    owner_scope: ScopeId,
    top_layer: LayerId,
    layer_generation: u64,
    trap: Option<ScopeId>,
}

pub struct Runtime<A: App> {
    app: A,
    theme: Theme,
    pointer_click_slop: u16,
    last: LastFrame,
    focus: FocusState,
    services: FrameServices,
    intents: IntentQueue,
    inter: Interaction,
    held_text_drag: Option<HeldTextDrag>,
    cancelled_text_drag: Option<(Id, PartRef)>,
    pending_tick: bool,
    pending_managed_tooltip_settle: bool,
    frame: FrameState,
    core: UiCore,
    generation: u32,
    cursor: Option<Position>,
    last_invalidate: Invalidate,
    pending_focus: Option<PendingFocus>,
    restore_after_publication: Option<FocusRestore>,
    keymap_conflict_key: Option<ConflictCacheKey>,
    keymap_conflict_component: Vec<crate::keymap::BindingDescriptor>,
    keymap_conflict_typing: Vec<crate::keymap::BindingDescriptor>,
    keymap_conflicts: Vec<Diagnostic>,
    staged_focus: Option<(Option<Id>, FocusVia)>,
    type_ahead_focus_changed: bool,
    layer_events_pending: Vec<RoutedLayerEvent>,
    focus_out_closed: Vec<Id>,
    completion_waiting: Vec<OpenLayer>,
    continuation_cancels_waiting: Vec<(crate::layer::LayerOpening, Id)>,
    completion_dispatch_pending: bool,
    unsettled: usize,
    bootstrapped: bool,
    presented: bool,
    prepared: Option<PreparedOutput>,
    scratch_buffer: Option<Buffer>,
    prepared_generation: u64,
    painted_cursor: Option<Position>,
    painted_cursor_bar: bool,
    cache_snapshot: Option<std::sync::Arc<()>>,
}

impl<A: App> core::fmt::Debug for Runtime<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Runtime")
            .field("screen", &self.services.viewport)
            .field("focus", &self.focus.current())
            .field("generation", &self.generation)
            .field("top_layer", &self.services.layers.top())
            .finish_non_exhaustive()
    }
}

impl<A: App> Runtime<A> {
    fn prepared_continuation_pending(&self) -> Option<Id> {
        self.completion_waiting
            .iter()
            .find_map(|layer| layer.spec.continues_focus_owner)
            .or_else(|| {
                self.continuation_cancels_waiting
                    .first()
                    .map(|(_, owner)| *owner)
            })
            .or_else(|| {
                self.services
                    .completions
                    .first()
                    .and_then(|(completion, _)| completion.continuation_owner())
            })
            .or_else(|| {
                self.services
                    .focus_continuation_outcomes
                    .first()
                    .map(|(outcome, _)| outcome.owner())
            })
            .or_else(|| {
                self.services
                    .continuation_command_pending_present
                    .as_ref()
                    .copied()
            })
    }

    fn refuse_legacy_continuation_draw(&mut self) {
        if let Some(owner) = self.prepared_continuation_pending() {
            self.services
                .diagnostics
                .push(Diagnostic::FocusContinuationUnacknowledged { owner });
            panic!("focus continuation requires Runtime::prepare and prepared output");
        }
    }

    fn invalidate_prepared(&mut self) {
        if let Some(output) = self.prepared.take() {
            self.scratch_buffer = Some(output.buffer);
        }
        self.presented = false;
        self.prepared_generation = self.prepared_generation.wrapping_add(1);
    }

    #[doc(hidden)]
    pub fn focused_hints_capacity(&self) -> usize {
        self.core.focused_hints.layer.hints.capacity()
    }

    pub fn new(app: A, theme: Theme) -> Self {
        Self::new_with_feedback_clock(app, theme, FeedbackClock::Elapsed)
    }

    pub fn new_with_feedback_clock(app: A, theme: Theme, clock: FeedbackClock) -> Self {
        let mut core = UiCore::default();
        core.keymap.clone_from(app.keymap());
        let runtime_id = next_runtime_id();
        let tooltips = app.tooltips();
        Runtime {
            app,
            theme,
            pointer_click_slop: 1,
            last: LastFrame::default(),
            focus: FocusState::default(),
            services: FrameServices {
                runtime_id,
                feedback: feedback::FeedbackState::new(clock),
                tooltips,
                ..FrameServices::default()
            },
            intents: IntentQueue::new(),
            inter: Interaction::default(),
            held_text_drag: None,
            cancelled_text_drag: None,
            pending_tick: false,
            pending_managed_tooltip_settle: false,
            frame: FrameState::default(),
            core,
            generation: 0,
            cursor: None,
            last_invalidate: Invalidate::None,
            pending_focus: None,
            restore_after_publication: None,
            keymap_conflict_key: None,
            keymap_conflict_component: Vec::new(),
            keymap_conflict_typing: Vec::new(),
            keymap_conflicts: Vec::new(),
            staged_focus: None,
            type_ahead_focus_changed: false,
            layer_events_pending: Vec::new(),
            focus_out_closed: Vec::new(),
            completion_waiting: Vec::new(),
            continuation_cancels_waiting: Vec::new(),
            completion_dispatch_pending: false,
            unsettled: 0,
            bootstrapped: false,
            presented: false,
            prepared: None,
            scratch_buffer: None,
            prepared_generation: 0,
            painted_cursor: None,
            painted_cursor_bar: false,
            cache_snapshot: None,
        }
    }

    pub fn set_pointer_click_slop(&mut self, cells: u16) {
        self.invalidate_prepared();
        self.pointer_click_slop = cells;
    }

    pub const fn app(&self) -> &A {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut A {
        self.invalidate_prepared();
        &mut self.app
    }

    pub fn typing_owner(&self) -> Option<Id> {
        (self.bootstrapped && self.presented && !self.needs_settle())
            .then_some(self.last.typing.owner)
            .flatten()
    }

    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn set_theme(&mut self, t: Theme) {
        self.invalidate_prepared();
        self.theme = t;
        self.core.clear_caches();
    }

    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.last.registry.area_of(id)
    }

    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect> {
        self.last.registry.area_of_part(id, p)
    }

    pub const fn ring(&self) -> &FocusRing {
        &self.last.ring
    }

    pub const fn focus(&self) -> Option<Id> {
        self.focus.current()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.services.diagnostics.items()
    }

    pub fn diagnostics_dropped(&self) -> usize {
        self.services.diagnostics.dropped()
    }

    pub const fn quit_requested(&self) -> bool {
        self.services.quit
    }

    pub fn wants_tick(&self) -> bool {
        self.needs_present() || self.needs_settle() || self.next_deadline().is_some()
    }

    pub fn next_deadline(&self) -> Option<Moment> {
        if self.pending_tick {
            return Some(self.now());
        }
        [
            self.services.repaint_at,
            self.services.feedback.elapsed_deadline(),
            self.held_text_drag.and_then(|drag| drag.next_due),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    pub fn activation_feedback(&self) -> Option<ActivationFeedback> {
        self.services.feedback.active(self.now())
    }

    pub const fn now(&self) -> Moment {
        self.services.now
    }

    pub fn advance_to(&mut self, now: Moment) -> Result<(), ClockError> {
        if now < self.now() {
            return Err(ClockError {
                current: self.now(),
                requested: now,
            });
        }
        if now != self.now() {
            self.invalidate_prepared();
        }
        self.services.now = now;
        let repaint_due = self.services.repaint_at.is_some_and(|at| at <= now);
        let text_drag_due = self
            .held_text_drag
            .is_some_and(|drag| drag.next_due.is_some_and(|at| at <= now));
        if repaint_due {
            self.services.repaint_at = None;
        }
        if !self.pending_tick && (repaint_due || text_drag_due) {
            self.pending_tick = true;
        }
        if self.services.feedback.expire_elapsed(now) {
            if self.inter.press.is_none() {
                self.last.snapshot.pressed = None;
            }
            self.invalidate_prepared();
            self.services.repaint = true;
        }
        Ok(())
    }

    pub fn clock_ms(&self) -> u64 {
        u64::try_from(self.now().as_duration().as_millis()).unwrap_or(u64::MAX)
    }

    pub const fn screen(&self) -> Rect {
        self.services.viewport
    }

    fn pointer_input_layer(&self) -> LayerId {
        self.services.layers.pointer_input_layer()
    }

    fn pointer_live_hit(&self, pos: Position) -> Option<Hit> {
        let input_layer = self.pointer_input_layer();
        let passive = self.services.layers.pointer_transparent_suffix();
        self.last
            .registry
            .hit_passive_hover(pos, passive)
            .or_else(|| self.last.registry.hit_live(pos, input_layer))
    }

    fn pointer_hit(&self, pos: Position) -> Option<Hit> {
        self.last
            .registry
            .hit_on_layer(pos, self.pointer_input_layer())
    }

    fn sync_keymap(&mut self, snapshot: Option<&KeyMap>) {
        let keymap = snapshot.unwrap_or_else(|| self.app.keymap());
        if &self.core.keymap != keymap {
            self.core.keymap.clone_from(keymap);

            self.core.keymap_revision = self.core.keymap_revision.wrapping_add(1);
            self.core.focused_hints.invalidate();
            self.keymap_conflict_key = None;
        }
    }

    fn refresh_keymap_conflicts(&mut self) {
        let component = self.focus.current().and_then(|owner| {
            self.last
                .bindings
                .get(owner)
                .map(|(published, table)| (owner, published.table, table))
        });
        let typing = self.last.typing.fallback.and_then(|owner| {
            self.last
                .typing_bindings
                .get(owner)
                .map(|(_, table)| (owner, table))
        });
        let component_table = component.map_or(&[][..], |(_, _, table)| table);
        let typing_table = typing.map_or(&[][..], |(_, table)| table);
        let key = ConflictCacheKey {
            focused: component.map(|(owner, table, _)| (owner, table)),
            typing_owner: typing.map(|(owner, _)| owner),
            revision: self.core.keymap_revision,
        };

        if self.keymap_conflict_key == Some(key)
            && self.keymap_conflict_component == component_table
            && self.keymap_conflict_typing == typing_table
        {
            return;
        }
        self.keymap_conflict_key = Some(key);
        self.keymap_conflict_component.clear();
        self.keymap_conflict_component
            .extend_from_slice(component_table);
        self.keymap_conflict_typing.clear();
        self.keymap_conflict_typing.extend_from_slice(typing_table);
        self.keymap_conflicts.clear();
        self.keymap_conflicts.extend(self.core.keymap.conflicts());
        if let Some((owner, _, table)) = component {
            self.keymap_conflicts
                .extend(self.core.keymap.component_conflicts(owner, table));
        }
        if let Some((owner, table)) = typing {
            self.keymap_conflicts
                .extend(self.core.keymap.component_conflicts(owner, table));
        }
    }

    fn provisional_focus_layer(&self, to: Id) -> Option<OpenLayer> {
        let layer = self.services.initial_focus_layer?;
        self.services
            .layers
            .top_layer()
            .copied()
            .filter(|top| top.spec.initial_focus == Some(to) && top.layer == layer)
    }

    fn focus_target_admissible(&self, to: Id) -> bool {
        if self.provisional_focus_layer(to).is_some() {
            return true;
        }
        if self.ring_proves_disabled(to) {
            return false;
        }
        match self.last.ring.entry(to) {
            Some(e) => {
                self.last
                    .ring
                    .active_trap()
                    .is_none_or(|t| self.last.ring.within(e.scope, t))
                    && e.layer >= self.services.layers.inert_floor()
            }
            None => !self.last.registry.has_owner(to),
        }
    }

    fn stage_focus(&mut self, to: Option<Id>, via: FocusVia) {
        if to.is_some_and(|id| !self.focus_target_admissible(id)) {
            return;
        }
        if via != FocusVia::Restore {
            self.restore_after_publication = None;
        }
        self.staged_focus = Some((to, via));
    }

    fn apply_staged_focus(&mut self) -> bool {
        let Some((to, via)) = self.staged_focus.take() else {
            return false;
        };
        let from = self.focus.current();
        if from == to {
            return false;
        }
        if let Some(old) = from {
            let from_layer = self.last.ring.entry(old).map(|entry| entry.layer);
            if self.focus_out_layer_is_live(from_layer) {
                self.intents.focus_out(old, to);
            }
        }
        if let Some(new) = to {
            self.intents.focus_in(new, via);
        }
        self.focus.set(to);
        self.last.snapshot.focus = to;
        self.type_ahead_focus_changed = true;

        self.services.repaint = true;

        if via != FocusVia::Restore || self.restore_after_publication.is_none() {
            self.dismiss_on_focus_out(to);
        }
        true
    }

    fn focus_out_layer_is_live(&self, layer: Option<LayerId>) -> bool {
        match layer {
            None | Some(LayerId::PAGE) => true,
            Some(layer) => self
                .services
                .layers
                .layers()
                .iter()
                .any(|open| open.layer == layer),
        }
    }

    fn focus_left_scope(&self, to: Option<Id>, scope: ScopeId) -> bool {
        let Some(id) = to else {
            return true;
        };
        if self
            .provisional_focus_layer(id)
            .is_some_and(|layer| layer.scope() == scope)
        {
            return false;
        }
        self.last
            .ring
            .entry(id)
            .is_some_and(|e| !self.last.ring.within(e.scope, scope))
    }

    fn dismiss_on_focus_out(&mut self, to: Option<Id>) {
        while let Some(top) = self.services.layers.top_layer().copied() {
            if !top.spec.dismisses_on_focus_out() || !self.focus_left_scope(to, top.scope()) {
                break;
            }
            self.focus_out_closed.push(top.id);
            if !self.dismiss_top(DismissReason::FocusOut) {
                break;
            }
        }
    }

    fn swallows_typing(&self) -> bool {
        self.focus
            .current()
            .and_then(|f| self.last.ring.entry(f))
            .is_some_and(|e| e.swallows_typing)
    }

    fn resize(&mut self, w: u16, h: u16) {
        self.services.viewport = Rect {
            x: 0,
            y: 0,
            width: w,
            height: h,
        };
        self.services.capture.release();
        self.services.press_pos = None;
        self.inter.press = None;
        self.inter.drag_motion = None;
        self.held_text_drag = None;
        self.cancelled_text_drag = None;
        self.inter.hover = None;
        self.last.snapshot.pressed = None;
        self.last.snapshot.hover = None;
        self.last.snapshot.capture = None;
        self.core.clear_caches();
        self.last_invalidate = Invalidate::Layout;
    }

    fn enqueue_key(&mut self, k: Key, deferred_type_ahead: Option<DeferredTypeAheadKey>) {
        self.prepare_keyboard_input();
        let current = self.focus.current();
        if !matches!(k.code, KeyCode::Tab | KeyCode::BackTab)
            && let Some(owner) = self.last.typing.fallback
        {
            if let Some((_, table)) = self.last.typing_bindings.get(owner)
                && let Some((action, chord)) = self.core.keymap.component_binding(owner, table, &k)
            {
                self.intents.binding(owner, action, chord);
                return;
            }
            if k.bare_char().is_some() {
                self.intents.key(owner, k);
                return;
            }
        }

        if let Some(owner) = current
            && let Some((_, table)) = self.last.bindings.get(owner)
            && let Some((action, chord)) = self.core.keymap.component_binding(owner, table, &k)
        {
            self.intents.binding(owner, action, chord);
            return;
        }

        if k.code == KeyCode::Tab && k.mods.difference(KeyModifiers::SHIFT).is_empty() {
            let next = if k.mods.contains(KeyModifiers::SHIFT) {
                self.last.ring.prev(current)
            } else {
                self.last.ring.next(current)
            };
            self.stage_focus(next, FocusVia::Keyboard);
            return;
        }
        if k.code == KeyCode::BackTab {
            let prev = self.last.ring.prev(current);
            self.stage_focus(prev, FocusVia::Keyboard);
            return;
        }
        if let Some(owner) = current
            && deferred_type_ahead.is_none()
        {
            self.intents.key(owner, k);
        }
    }

    fn deferred_type_ahead_key(&self, key: Key) -> Option<DeferredTypeAheadKey> {
        termrock_core::typeahead::TypeAhead::char_of(&key)?;
        let owner = self.focus.current()?;
        if self.last.snapshot.focus != Some(owner)
            || self.last.typing.fallback.is_some()
            || self.last.typing.owner == Some(owner)
        {
            return None;
        }
        let entry = self.last.ring.entry(owner)?;
        if !self
            .last
            .type_aheads
            .iter()
            .any(|declaration| declaration.owner == owner && declaration.layer == entry.layer)
        {
            return None;
        }
        if entry.disabled
            || entry.swallows_typing
            || entry.area.is_empty()
            || !self.last.ring.contains(owner)
            || self.last.declared.iter().any(|(id, state)| {
                *id == owner && state.contains(crate::response::StateFlags::EDITING)
            })
            || entry.layer < self.services.layers.inert_floor()
        {
            return None;
        }

        Some(DeferredTypeAheadKey {
            key,
            owner,
            owner_layer: entry.layer,
            owner_scope: entry.scope,
            top_layer: self.services.layers.top(),
            layer_generation: self.services.layers.next_generation(),
            trap: self.last.ring.active_trap(),
        })
    }

    fn deferred_type_ahead_is_current(
        &self,
        deferred: DeferredTypeAheadKey,
        input_scope_changed: bool,
    ) -> bool {
        if input_scope_changed
            || self.type_ahead_focus_changed
            || self.focus.current() != Some(deferred.owner)
            || self.last.snapshot.focus != Some(deferred.owner)
            || self.services.layers.top() != deferred.top_layer
            || self.services.layers.next_generation() != deferred.layer_generation
            || self.last.ring.active_trap() != deferred.trap
            || self.last.typing.fallback.is_some()
            || self.last.typing.owner == Some(deferred.owner)
        {
            return false;
        }

        self.last.type_aheads.iter().any(|declaration| {
            declaration.owner == deferred.owner && declaration.layer == deferred.owner_layer
        }) && self.last.ring.contains(deferred.owner)
            && self.last.ring.entry(deferred.owner).is_some_and(|entry| {
                entry.layer == deferred.owner_layer
                    && entry.scope == deferred.owner_scope
                    && !entry.disabled
                    && !entry.swallows_typing
                    && !entry.area.is_empty()
                    && entry.layer >= self.services.layers.inert_floor()
            })
            && !self.last.declared.iter().any(|(id, state)| {
                *id == deferred.owner && state.contains(crate::response::StateFlags::EDITING)
            })
    }

    fn prepare_keyboard_input(&mut self) {
        self.inter.last_input_key = true;
        self.inter.hover_suppressed = true;
        self.focus.set_visible(true);
        self.last.snapshot.focus_visible = true;
        self.last.snapshot.hover_suppressed = true;
    }

    fn top_transparent_tooltip_allows_escape(&self) -> bool {
        let Some(top) = self.services.layers.top_layer() else {
            return false;
        };
        top.layer == self.services.layers.top()
            && self.services.layers.pointer_transparent_suffix().next() == Some(top.layer)
            && top.spec.dismiss.allows(DismissReason::Esc)
    }

    fn ring_proves_disabled(&self, id: Id) -> bool {
        self.last.ring.entry(id).is_some_and(|e| e.disabled)
    }

    fn deliverable(&self, hit: Hit) -> bool {
        crate::capture::target_eligible(
            &self.last.ring,
            self.pointer_input_layer(),
            hit.owner,
            hit.layer,
            hit.kind,
        )
    }

    fn is_disabled_primary_hit(&self, hit: Hit) -> bool {
        hit.layer == self.pointer_input_layer()
            && matches!(hit.kind, RegionKind::Control | RegionKind::Part)
            && self
                .last
                .ring
                .entry(hit.owner)
                .is_some_and(|entry| entry.disabled && entry.layer == hit.layer)
    }

    fn managed_tooltip_source_is_live(&self, managed: ManagedTooltip) -> bool {
        managed.parent_layer == self.services.layers.pointer_input_layer()
            && managed.source_is_live(&self.last.registry)
    }

    fn dismiss_managed_tooltip_for_source_press(&mut self, hit: Hit) {
        let Some(managed) = self.services.managed_tooltip else {
            return;
        };
        if hit.owner == managed.owner
            && hit.part == managed.source_part
            && hit.layer == managed.parent_layer
        {
            self.dismiss_layer(managed.opening.id(), DismissReason::OutsideClick);
        }
    }

    fn local_in(area: Rect, pos: Position) -> Position {
        Position {
            x: pos.x.saturating_sub(area.x),
            y: pos.y.saturating_sub(area.y),
        }
    }

    fn region_area(&self, hit: Hit) -> Rect {
        if hit.part == PartRef::of(Part::CONTAINER) {
            self.last.registry.area_of(hit.owner).unwrap_or_default()
        } else {
            self.last
                .registry
                .area_of_part(hit.owner, hit.part)
                .or_else(|| self.last.registry.area_of(hit.owner))
                .unwrap_or_default()
        }
    }

    fn enqueue_mouse(&mut self, m: Mouse) -> bool {
        self.services.pointer_press_count = None;
        self.inter.pointer_position = Some(m.pos);
        if matches!(m.kind, MouseKind::Down | MouseKind::Up) {
            self.inter.drag_motion = None;
        }
        if m.kind == MouseKind::Down {
            self.held_text_drag = None;
            self.cancelled_text_drag = None;
        }
        if m.kind == MouseKind::Up {
            self.held_text_drag = None;
            self.cancelled_text_drag = None;
        }
        if m.kind == MouseKind::Up {
            self.services.press_pos = None;
        }
        self.inter.last_input_key = false;
        self.focus.set_visible(false);
        self.last.snapshot.focus_visible = false;
        let hover_was_suppressed = self.inter.hover_suppressed;
        if m.kind == MouseKind::Move {
            self.inter.hover_passes = 0;
            self.inter.hover_suppressed = false;
            self.last.snapshot.hover_suppressed = false;
        }

        if let Some(cap) = self.services.capture.get() {
            self.pointer_captured(cap, m);
            return false;
        }
        let input_layer = self.pointer_input_layer();
        let hit = self.last.registry.hit_on_layer(m.pos, input_layer);
        let mut disabled_primary_refusal = false;
        match m.kind {
            MouseKind::Move => self.pointer_moved(m, hover_was_suppressed),
            MouseKind::Down => {
                self.dismiss_transparent_tooltips_outside(m.pos);
                if input_layer != LayerId::PAGE && hit.is_none() {
                    self.outside_click_at(input_layer);
                    return false;
                }
                if let Some(h) = hit {
                    disabled_primary_refusal = self.is_disabled_primary_hit(h);
                    self.dismiss_managed_tooltip_for_source_press(h);
                    self.pointer_down(h, m);
                }
            }
            MouseKind::Drag => {
                let Some(p) = self.inter.press else {
                    return false;
                };
                self.inter.drag_motion = Some(DragMotion {
                    position: m.pos,
                    modifiers: m.mods,
                });
                if !p.dragging {
                    self.intents.pointer(
                        p.owner,
                        Phase::DragStart,
                        p.part,
                        m.pos,
                        Self::local_in(p.area, m.pos),
                        m.mods,
                    );
                    if let Some(pr) = self.inter.press.as_mut() {
                        pr.dragging = true;
                    }
                }
                self.intents.pointer(
                    p.owner,
                    Phase::Drag,
                    p.part,
                    m.pos,
                    Self::local_in(p.area, m.pos),
                    m.mods,
                );
            }
            MouseKind::Up => {
                if let Some(p) = self.inter.press.take() {
                    self.pointer_up(p, hit, m);
                }
            }
            MouseKind::Secondary => {
                self.dismiss_transparent_tooltips_outside(m.pos);
                if input_layer != LayerId::PAGE && hit.is_none() {
                    self.outside_click_at(input_layer);
                    return false;
                }
                let Some(h) = hit else {
                    return false;
                };
                if !self.deliverable(h) {
                    return false;
                }
                let area = self.region_area(h);
                self.intents.pointer(
                    h.owner,
                    Phase::Secondary,
                    h.part,
                    m.pos,
                    Self::local_in(area, m.pos),
                    m.mods,
                );
            }
            MouseKind::SecondaryUp => {}
            MouseKind::Wheel(axis, delta) => {
                let mut skipped_passive_reveal = false;
                for candidate in self
                    .last
                    .registry
                    .scroll_candidates(m.pos, axis, input_layer)
                {
                    let hit = candidate.hit();
                    if candidate.passive_reveal() {
                        if skipped_passive_reveal {
                            continue;
                        }
                        let headroom = candidate.headroom();
                        let has_directional_extent = if delta > 0 {
                            headroom.down > 0
                        } else {
                            headroom.up > 0
                        };
                        if !m.mods.is_empty()
                            || axis != Axis::V
                            || delta == 0
                            || !has_directional_extent
                        {
                            skipped_passive_reveal = true;
                            continue;
                        }
                        self.intents.wheel(
                            hit.owner,
                            axis,
                            delta.saturating_mul(3),
                            hit.part,
                            m.pos,
                        );
                        break;
                    }

                    let rows = self.theme.design.motion.wheel_rows as i16;
                    let scaled_delta = delta.saturating_mul(rows);
                    self.intents
                        .wheel(hit.owner, axis, scaled_delta, hit.part, m.pos);
                    if let Some((key, source_revision)) = self
                        .last
                        .registry
                        .routed_scroll_info(candidate.registration_index())
                        .filter(|(key, _)| {
                            self.last
                                .registry
                                .route_key_is_eligible(*key, axis, m.pos, input_layer)
                        })
                        && let Some(event_id) = self.services.next_wheel_route_event()
                    {
                        self.services.pending_wheel_route = Some(WheelRouteAttempt::new(
                            WheelRouteTarget {
                                runtime_id: self.services.runtime_id,
                                event_id,
                                attempt_index: 0,
                                key,
                                owner: hit.owner,
                                part: hit.part,
                                layer: hit.layer,
                                registry_generation: self.last.registry.generation(),
                                source_revision,
                            },
                            axis,
                            scaled_delta,
                            delta,
                            m.pos,
                            m.mods,
                        ));
                    }
                    break;
                }
            }
        }
        disabled_primary_refusal
    }

    fn enqueue_middle_click(&mut self, pos: Position, mods: KeyModifiers) {
        self.services.pointer_press_count = None;
        self.inter.pointer_position = Some(pos);
        let Some(hit) = self.pointer_hit(pos) else {
            return;
        };
        if !self.deliverable(hit) {
            return;
        }
        let area = self.region_area(hit);
        self.intents.pointer(
            hit.owner,
            Phase::Middle,
            hit.part,
            pos,
            Self::local_in(area, pos),
            mods,
        );
    }

    fn pointer_moved(&mut self, m: Mouse, hover_was_suppressed: bool) {
        let live = self.pointer_live_hit(m.pos);
        let previous = self.inter.hover;
        let visible_previous = if hover_was_suppressed { None } else { previous };
        self.inter.hover = live.map(|h| (h.owner, h.part));
        self.last.snapshot.hover = self.inter.hover;
        if self.inter.hover != visible_previous {
            self.services.repaint = true;
        }
        if let Some(h) = live
            && h.kind != RegionKind::HoverOnly
        {
            let area = self.region_area(h);
            self.intents.pointer(
                h.owner,
                Phase::Move,
                h.part,
                m.pos,
                Self::local_in(area, m.pos),
                m.mods,
            );
        }
    }

    fn pointer_down(&mut self, h: Hit, m: Mouse) {
        if !self.deliverable(h) {
            return;
        }
        let area = self.region_area(h);
        let window = self.theme.design.motion.double_click_ms;
        let count = self.inter.last_click.filter(|last| {
            last.owner == h.owner
                && last.part == h.part
                && self.now().saturating_duration_since(last.at)
                    < core::time::Duration::from_millis(window)
                && last.pos.x.abs_diff(m.pos.x) <= self.pointer_click_slop
                && last.pos.y.abs_diff(m.pos.y) <= self.pointer_click_slop
        });
        let count = count.map_or(1, |last| last.count.saturating_add(1));
        self.services.pointer_press_count = Some((h.owner, h.part, count));
        self.inter.press = Some(Press {
            owner: h.owner,
            part: h.part,
            layer: h.layer,
            area,
            dragging: false,
            pos: m.pos,
            click_count: count,
        });
        self.services.press_pos = Some(m.pos);
        self.last.snapshot.pressed = Some((h.owner, h.part));
        if self.last.ring.contains(h.owner) {
            self.stage_focus(Some(h.owner), FocusVia::Pointer);
        }
        self.intents.pointer(
            h.owner,
            Phase::Press,
            h.part,
            m.pos,
            Self::local_in(area, m.pos),
            m.mods,
        );
    }

    fn pointer_up(&mut self, p: Press, hit: Option<Hit>, m: Mouse) {
        self.last.snapshot.pressed = None;
        self.services.pointer_press_count = Some((p.owner, p.part, p.click_count));
        let local = Self::local_in(p.area, m.pos);
        self.intents
            .pointer(p.owner, Phase::Release, p.part, m.pos, local, m.mods);
        if p.dragging {
            self.intents
                .pointer(p.owner, Phase::DragEnd, p.part, m.pos, local, m.mods);
            return;
        }
        let same =
            hit.is_some_and(|h| h.owner == p.owner && h.part == p.part && h.layer == p.layer);
        if !same {
            return;
        }
        let click_count = self.valid_click_count(p.owner, p.part, m.pos);
        self.services.pointer_press_count = Some((p.owner, p.part, click_count));
        let double = click_count.is_multiple_of(2);
        let phase = if double {
            Phase::DoubleClick
        } else {
            Phase::Click
        };
        self.inter.last_click = Some(LastClick {
            owner: p.owner,
            part: p.part,
            at: self.now(),
            pos: m.pos,
            count: click_count,
        });
        self.intents
            .pointer(p.owner, phase, p.part, m.pos, local, m.mods);
        let flash = self.theme.design.motion.press_flash_ms;
        self.services.feedback.activate(
            p.owner,
            p.part,
            self.now(),
            core::time::Duration::from_millis(flash),
        );
        self.last.snapshot.pressed = self.services.feedback.pressed();
    }

    fn pointer_captured(&mut self, cap: Capture, m: Mouse) {
        let local = cap.local(m.pos);
        match m.kind {
            MouseKind::Drag | MouseKind::Move => {
                self.inter.drag_motion = Some(DragMotion {
                    position: m.pos,
                    modifiers: m.mods,
                });
                self.intents
                    .pointer(cap.owner, Phase::Drag, cap.part, m.pos, local, m.mods);
            }
            MouseKind::Up => {
                let press = self.inter.press.take();
                self.services.pointer_press_count =
                    press.map(|press| (press.owner, press.part, press.click_count));
                self.intents
                    .pointer(cap.owner, Phase::Release, cap.part, m.pos, local, m.mods);
                self.intents
                    .pointer(cap.owner, Phase::DragEnd, cap.part, m.pos, local, m.mods);
                if cap.contains(m.pos) {
                    if let Some(press) = press {
                        let click_count = self.valid_click_count(press.owner, press.part, m.pos);
                        self.inter.last_click = Some(LastClick {
                            owner: press.owner,
                            part: press.part,
                            at: self.now(),
                            pos: m.pos,
                            count: click_count,
                        });
                        self.services.pointer_press_count =
                            Some((press.owner, press.part, click_count));
                    }
                    self.intents
                        .pointer(cap.owner, Phase::Click, cap.part, m.pos, local, m.mods);
                } else {
                    self.inter.last_click = None;
                }
                self.services.capture.release();
                self.last.snapshot.pressed = None;
                self.last.snapshot.capture = None;
            }
            MouseKind::Down => {
                self.services.press_pos = Some(m.pos);
                self.intents
                    .pointer(cap.owner, Phase::Press, cap.part, m.pos, local, m.mods);
            }
            MouseKind::Secondary | MouseKind::SecondaryUp | MouseKind::Wheel(..) => {}
        }
    }

    fn valid_click_count(&self, owner: Id, part: PartRef, pos: Position) -> u8 {
        let window = self.theme.design.motion.double_click_ms;
        self.inter
            .last_click
            .filter(|last| {
                last.owner == owner
                    && last.part == part
                    && self.now().saturating_duration_since(last.at)
                        < core::time::Duration::from_millis(window)
                    && last.pos.x.abs_diff(pos.x) <= self.pointer_click_slop
                    && last.pos.y.abs_diff(pos.y) <= self.pointer_click_slop
            })
            .map_or(1, |last| last.count.saturating_add(1))
    }

    fn outside_click_at(&mut self, input_layer: LayerId) {
        let Some(target) = self
            .services
            .layers
            .layers()
            .iter()
            .find(|layer| layer.layer == input_layer)
            .copied()
        else {
            return;
        };
        if target.spec.dismiss.outside_click {
            self.dismiss_layer(target.id, DismissReason::OutsideClick);
        }
    }

    fn dismiss_transparent_tooltips_outside(&mut self, pos: Position) {
        let suffix: Vec<_> = self.services.layers.pointer_transparent_suffix().collect();
        for layer_id in suffix {
            let Some(open) = self
                .services
                .layers
                .layers()
                .iter()
                .find(|layer| layer.layer == layer_id)
                .copied()
            else {
                continue;
            };
            if !open.spec.dismiss.outside_click {
                continue;
            }
            let area = self
                .frame
                .layers
                .active()
                .iter()
                .find(|draw| draw.id == open.id)
                .map_or_else(
                    || resolve_layer_area(self.services.viewport, open.spec),
                    |draw| draw.area,
                );
            if !area.contains(pos) {
                self.dismiss_layer(open.id, DismissReason::OutsideClick);
            }
        }
    }

    fn dismiss_top(&mut self, reason: DismissReason) -> bool {
        let Some(top) = self.services.layers.top_layer().copied() else {
            return false;
        };
        self.dismiss_layer(top.id, reason)
    }

    fn dismiss_layer(&mut self, id: Id, reason: DismissReason) -> bool {
        let Some(target) = self.services.layers.get(id).copied() else {
            return false;
        };
        let closed =
            self.services
                .layers
                .close_with_update(id, LayerEvent::Dismissed(reason), None);
        let changed = !closed.is_empty();
        if let Some(root) = closed.first() {
            self.services.direct_closed_layers.push(*root);
        }
        self.clear_managed_tooltip_for_closed(&closed);
        self.services.closed_layers.extend(closed);
        if changed {
            self.services.reset_pointer_clicks = true;
        }

        if reason == DismissReason::Esc {
            self.intents.cancel(target.spec.owner);
        }
        changed
    }

    fn clear_managed_tooltip_for_closed(&mut self, closed: &[OpenLayer]) {
        if self
            .services
            .managed_tooltip
            .is_some_and(|managed| closed.iter().any(|layer| layer.opening == managed.opening))
        {
            self.services.managed_tooltip = None;
        }
    }

    fn settle_closed_layers(&mut self) {
        let closed: Vec<OpenLayer> = core::mem::take(&mut self.services.closed_layers);
        if closed.is_empty() {
            return;
        }
        let direct = core::mem::take(&mut self.services.direct_closed_layers);
        let still_open: Vec<_> = self
            .services
            .layers
            .layers()
            .iter()
            .map(|layer| layer.opening)
            .collect();
        let closed_openings: Vec<_> = closed.iter().map(|layer| layer.opening).collect();
        let closed_continuations: Vec<_> = self
            .services
            .focus_continuation_acks
            .iter()
            .filter(|(opening, _, _)| closed_openings.contains(opening))
            .map(|(opening, owner, _)| (*opening, *owner))
            .collect();
        let is_descendant = |child: &OpenLayer, ancestor: &OpenLayer| {
            let mut parent = child.opening.parent();
            while let Some(current) = parent {
                if current.id == ancestor.id
                    && current.layer == ancestor.layer
                    && current.generation == ancestor.opening.generation()
                {
                    return true;
                }
                parent = closed
                    .iter()
                    .find(|layer| {
                        layer.id == current.id
                            && layer.layer == current.layer
                            && layer.opening.generation() == current.generation
                    })
                    .and_then(|layer| layer.opening.parent());
            }
            false
        };
        self.completion_waiting.retain(|waiting| {
            !waiting.opening.parent().is_some_and(|parent| {
                closed_openings.iter().any(|opening| {
                    opening.id() == parent.id
                        && opening.layer_id() == parent.layer
                        && opening.generation() == parent.generation
                })
            })
        });

        for layer in &direct {
            if layer.spec.continues_focus_owner.is_none() {
                continue;
            }
            let superseded = direct.iter().any(|ancestor| {
                ancestor.opening != layer.opening && is_descendant(layer, ancestor)
            });
            if !superseded {
                self.completion_waiting.push(*layer);
            }
        }
        for (opening, owner) in closed_continuations {
            if !self
                .completion_waiting
                .iter()
                .any(|waiting| waiting.opening == opening)
                && !self
                    .continuation_cancels_waiting
                    .contains(&(opening, owner))
            {
                self.continuation_cancels_waiting.push((opening, owner));
            }
        }
        self.services
            .focus_continuation_acks
            .retain(|(opening, _, _)| {
                still_open.contains(opening)
                    || self
                        .completion_waiting
                        .iter()
                        .any(|waiting| waiting.opening == *opening)
            });
        let focus_out = core::mem::take(&mut self.focus_out_closed);
        for l in &closed {
            if let Some(c) = self.services.capture.get()
                && self.last.registry.layer_of(c.owner) == Some(l.layer)
            {
                self.services.capture.release();
                self.last.snapshot.capture = None;
            }
        }

        if let Some(last) = direct.last()
            && last.spec.restore_focus
            && !focus_out.contains(&last.id)
        {
            let target = last
                .restore_to
                .or_else(|| self.focus.take_restore(last.scope()));
            self.restore_after_publication = Some(FocusRestore { target });

            if self.focus.current().is_some_and(|owner| {
                self.last
                    .ring
                    .entry(owner)
                    .is_some_and(|entry| closed.iter().any(|layer| layer.layer == entry.layer))
            }) {
                self.stage_focus(None, FocusVia::Restore);
            }
            self.services.repaint = true;
        }
    }

    fn pump_layer_events(&mut self) {
        for pending in self.services.layers.take_pending_receipts() {
            let receipt = LayerEventReceipt {
                opening: pending.opening,
                event: pending.event,
                update: pending.update,
            };
            let id = receipt.opening().id();
            let event = receipt.event();
            let owner = receipt.opening().owner();
            if owner != id {
                self.intents.layer(owner, event);
            }
            let token = self.intents.layer_tracked(id, event);
            self.layer_events_pending
                .push(RoutedLayerEvent::new(receipt, token));
        }
    }

    fn run_update(
        &mut self,
        command: Option<ActionKey>,
        cause: UpdateCause,
        activation_key: Option<ActivationKey>,
    ) -> Response<()> {
        self.invalidate_prepared();
        self.core.begin_cache_frame(self.generation.wrapping_add(1));
        let mut folded = Response::ignored();
        let mut first_pass = true;
        loop {
            let pass_cause = if first_pass {
                cause
            } else {
                UpdateCause::Settle
            };
            self.pump_layer_events();
            self.apply_staged_focus();

            self.pump_layer_events();
            self.services.focus_request = None;
            let events = core::mem::take(&mut self.layer_events_pending);
            self.services.events.extend(events);
            self.services.tooltips = self.app.tooltips();
            let edit_entry = self.app.edit_entry();
            let r = {
                let mut cx = Cx::new_with_cause(
                    &self.intents,
                    &mut self.services,
                    &mut self.core,
                    &self.last,
                    &self.theme,
                    command,
                    pass_cause,
                )
                .with_edit_entry(edit_entry)
                .with_activation_key(activation_key);
                self.app.update(&mut cx)
            };
            if first_pass {
                self.services.active_wheel_route = None;
            }
            folded |= r;

            for owner in self.intents.undrained() {
                if self.last.registry.delivers_to(owner)
                    || self.intents.has_runtime_addressed(owner)
                {
                    self.services
                        .diagnostics
                        .push(Diagnostic::UndeliveredIntent { owner });
                }
            }

            self.layer_events_pending = core::mem::take(&mut self.services.events);

            for l in self.services.layers.layers() {
                if let Some(id) = l.restore_to
                    && self.focus.restore_target(l.scope()).is_none()
                {
                    self.focus.save_restore(l.scope(), id);
                }
            }
            self.settle_closed_layers();
            if let Some(f) = self.services.focus_request.take() {
                self.stage_focus(Some(f), FocusVia::Programmatic);
            }
            let moves = self
                .staged_focus
                .is_some_and(|(to, _)| to != self.focus.current());
            if !moves {
                self.staged_focus = None;
                break;
            }
            self.intents.clear();
            if first_pass {
                self.unsettled = 0;
            }
            first_pass = false;
            self.unsettled = self.unsettled.saturating_add(1);
            if self.unsettled >= MAX_FOCUS_PASSES {
                let (target, via) = self.staged_focus.unwrap_or((None, FocusVia::Programmatic));
                self.services
                    .diagnostics
                    .push(Diagnostic::FocusTransitionDidNotSettle { target });
                let from = self.focus.current();
                if self.apply_staged_focus() {
                    let from_layer = from.and_then(|id| self.last.ring.entry(id).map(|e| e.layer));
                    self.pending_focus = Some(PendingFocus {
                        from,
                        to: target,
                        via,
                        from_layer,
                    });
                }
                self.intents.clear();
                break;
            }
        }
        self.unsettled = 0;
        folded
    }

    fn wheel_route_context_is_live(
        &self,
        guard: WheelRouteGuard,
        attempt: WheelRouteAttempt,
        response: &Response<()>,
    ) -> bool {
        if response.invalidate() == Invalidate::Layout
            || self.services.wheel_route_invalidated
            || self.services.focus_request.is_some()
            || self.staged_focus.is_some()
            || self.pending_focus.is_some()
            || self.focus.current() != guard.focus
            || self.services.layers.top() != guard.top_layer
            || self.pointer_input_layer() != guard.input_layer
            || self.services.capture.get() != guard.capture
            || self.last.registry.generation() != guard.registry_generation
            || self.services.runtime_id != attempt.runtime_id
            || attempt.layer() != guard.input_layer
            || attempt.registry_generation() != guard.registry_generation
        {
            return false;
        }
        if self
            .last
            .registry
            .routed_scroll_source_revision(attempt.key)
            != Some(attempt.source_revision)
        {
            return false;
        }
        self.last
            .registry
            .routed_scroll_hit(
                attempt.key,
                attempt.axis(),
                attempt.position(),
                attempt.layer(),
            )
            .is_some_and(|hit| hit.owner == attempt.owner() && hit.part == attempt.part())
    }

    fn service_ignored_wheel_route(
        &mut self,
        first: WheelRouteAttempt,
        guard: WheelRouteGuard,
        mut folded: Response<()>,
    ) -> Response<()> {
        if self.services.wheel_route_ack.get()
            != WheelRouteAck::Acknowledged(WheelRouteResult::Ignored)
            || !self.wheel_route_context_is_live(guard, first, &folded)
        {
            return folded;
        }

        let mut current = first;
        let hit_stack = self
            .last
            .registry
            .routed_scroll_policy(first.key)
            .is_some_and(|policy| {
                matches!(
                    policy,
                    ScrollRoutePolicy::HitStack | ScrollRoutePolicy::EventLocalHitStack
                )
            });
        let mut offered_owners = if hit_stack {
            let mut owners = Vec::with_capacity(self.last.registry.len());
            owners.push(first.owner());
            owners
        } else {
            Vec::new()
        };
        for _ in 0..self.last.registry.len() {
            let Some((key, source_revision)) = self.last.registry.next_scroll_route_target(
                current.key,
                current.axis(),
                current.position(),
                current.layer(),
                &offered_owners,
            ) else {
                break;
            };
            let Some(index) = current.attempt_index.checked_add(1) else {
                break;
            };
            let Some(hit) = self.last.registry.routed_scroll_hit(
                key,
                current.axis(),
                current.position(),
                current.layer(),
            ) else {
                break;
            };
            if !self.wheel_route_context_is_live(guard, current, &folded) {
                break;
            }
            let attempt = WheelRouteAttempt::new(
                WheelRouteTarget {
                    runtime_id: current.runtime_id,
                    event_id: current.event_id,
                    attempt_index: index,
                    key,
                    owner: hit.owner,
                    part: hit.part,
                    layer: hit.layer,
                    registry_generation: current.registry_generation(),
                    source_revision,
                },
                current.axis(),
                current.delta(),
                current.unscaled_delta(),
                current.position(),
                current.modifiers(),
            );
            self.services.active_wheel_route = Some(attempt);
            self.services.wheel_route_ack.set(WheelRouteAck::Unset);
            let empty_intents = IntentQueue::new();
            self.services.tooltips = self.app.tooltips();
            let edit_entry = self.app.edit_entry();
            let response = {
                let mut cx = Cx::new_with_cause(
                    &empty_intents,
                    &mut self.services,
                    &mut self.core,
                    &self.last,
                    &self.theme,
                    None,
                    UpdateCause::ScrollRoute,
                )
                .with_edit_entry(edit_entry);
                self.app.route_wheel_attempt(&mut cx, attempt)
            };
            self.services.active_wheel_route = None;
            folded |= response;
            if !self.wheel_route_context_is_live(guard, attempt, &folded)
                || self.services.wheel_route_ack.get()
                    != WheelRouteAck::Acknowledged(WheelRouteResult::Ignored)
            {
                break;
            }
            if hit_stack {
                offered_owners.push(attempt.owner());
            }
            current = attempt;
        }
        self.services.active_wheel_route = None;
        folded
    }

    pub fn initialize(&mut self) -> Response<()> {
        if self.bootstrapped {
            return Response::ignored();
        }
        self.bootstrapped = true;
        self.sync_keymap(None);
        self.refresh_keymap_conflicts();
        self.services
            .diagnostics
            .extend(self.keymap_conflicts.iter().cloned());
        self.services.registry_gen = self.last.registry.generation();
        self.intents.clear();
        let r = self.run_update(None, UpdateCause::Bootstrap, None);
        self.finish(r)
    }

    pub const fn needs_settle(&self) -> bool {
        self.pending_focus.is_some()
            || self.completion_dispatch_pending
            || self.services.focus_result_ack_dispatch_pending
            || self.pending_managed_tooltip_settle
            || (self.bootstrapped && self.pending_tick)
    }

    pub const fn needs_present(&self) -> bool {
        !self.presented
    }

    pub fn prepare(&mut self, area: Rect) -> Result<PreparedFrame, PreparedFrameError> {
        if !self.bootstrapped {
            let _ = self.initialize();
        }
        if let Some(output) = self.prepared.take() {
            self.scratch_buffer = Some(output.buffer);
        }
        let mut passes = 0;
        while passes < MAX_FOCUS_PASSES * MAX_FOCUS_PASSES {
            if self.needs_settle() {
                let _ = self.settle();
                passes += 1;
                continue;
            }
            let mut buffer = self
                .scratch_buffer
                .take()
                .unwrap_or_else(|| Buffer::empty(area));
            buffer.resize(area);
            buffer.reset();
            self.draw_with_buffer(area, &mut buffer, None, A::draw);
            self.commit_frame();
            if !self.presented || self.needs_settle() {
                self.scratch_buffer = Some(buffer);
                passes += 1;
                continue;
            }
            self.prepared_generation = self.prepared_generation.wrapping_add(1);
            let token = PreparedFrame {
                runtime: self.services.runtime_id,
                generation: self.prepared_generation,
                area,
            };
            self.prepared = Some(PreparedOutput {
                token,
                buffer,
                cursor: self.painted_cursor,
                cursor_bar: self.painted_cursor_bar,
                projected: false,
            });

            self.presented = false;
            return Ok(token);
        }
        self.invalidate_prepared();
        Err(PreparedFrameError::DidNotSettle)
    }

    pub fn draw_prepared(
        &mut self,
        token: PreparedFrame,
        frame: &mut Frame<'_>,
    ) -> Result<bool, PreparedFrameError> {
        let actual = frame.area();
        let runtime_id = self.services.runtime_id;
        let generation = self.prepared_generation;
        let result = if let Some(output) = self.prepared.as_mut() {
            if token.runtime != runtime_id
                || token != output.token
                || token.generation != generation
            {
                Err(PreparedFrameError::Stale)
            } else if token.area != actual {
                Err(PreparedFrameError::AreaChanged {
                    expected: token.area,
                    actual,
                })
            } else {
                frame.buffer_mut().merge(&output.buffer);
                if let Some(cursor) = output.cursor {
                    frame.set_cursor_position(cursor);
                }
                output.projected = true;
                Ok(output.cursor_bar)
            }
        } else {
            Err(PreparedFrameError::Stale)
        };
        if matches!(&result, Err(PreparedFrameError::AreaChanged { .. })) {
            self.invalidate_prepared();
        }
        result
    }

    pub fn acknowledge_presented(
        &mut self,
        token: PreparedFrame,
    ) -> Result<(), PreparedFrameError> {
        let runtime_id = self.services.runtime_id;
        let Some(output) = self.prepared.take_if(|output| {
            token.runtime == runtime_id && token == output.token && output.projected
        }) else {
            return Err(PreparedFrameError::Stale);
        };
        self.scratch_buffer = Some(output.buffer);
        self.presented = true;
        self.services.continuation_command_pending_present = None;
        for (_, age) in &mut self.services.completions {
            *age = age.saturating_add(1);
        }
        self.services.completions.retain(|(completion, age)| {
            if *age < 32 {
                true
            } else {
                self.services
                    .diagnostics
                    .push(Diagnostic::FocusContinuationUnacknowledged {
                        owner: completion.opening().owner(),
                    });
                false
            }
        });
        self.services
            .focus_continuation_result_acks
            .retain(|(opening, owner, _)| {
                self.services.completions.iter().any(|(completion, _)| {
                    completion.opening() == *opening
                        && completion.continuation_owner() == Some(*owner)
                })
            });
        for (_, age) in &mut self.services.focus_continuation_outcomes {
            *age = age.saturating_add(1);
        }
        self.services
            .focus_continuation_outcomes
            .retain(|(outcome, age)| {
                if *age < 32 {
                    true
                } else {
                    self.services
                        .diagnostics
                        .push(Diagnostic::FocusContinuationUnacknowledged {
                            owner: outcome.owner(),
                        });
                    false
                }
            });
        Ok(())
    }

    pub fn settle(&mut self) -> Response<()> {
        if !self.bootstrapped {
            return Response::ignored();
        }
        let Some(pending) = self.pending_focus.take() else {
            let completion_dispatch = core::mem::take(&mut self.completion_dispatch_pending);
            let focus_result_ack_dispatch =
                core::mem::take(&mut self.services.focus_result_ack_dispatch_pending);
            let managed_tooltip_settle = core::mem::take(&mut self.pending_managed_tooltip_settle);
            if completion_dispatch || focus_result_ack_dispatch || managed_tooltip_settle {
                self.intents.clear();
                let r = self.run_update(None, UpdateCause::Settle, None);
                return self.finish(r);
            }
            if !core::mem::take(&mut self.pending_tick) {
                return Response::ignored();
            }
            self.intents.clear();
            self.enqueue_due_text_drag();
            self.pump_layer_events();
            let r = self.run_update(None, UpdateCause::Tick, None);
            return self.finish(r);
        };
        self.intents.clear();
        if let Some(old) = pending.from
            && self.focus_out_layer_is_live(pending.from_layer)
        {
            self.intents.focus_out(old, pending.to);
        }
        if let Some(new) = pending.to {
            self.intents.focus_in(new, pending.via);
        }
        self.dismiss_on_focus_out(pending.to);
        let r = self.run_update(None, UpdateCause::Settle, None);
        self.finish(r)
    }

    pub fn handle(&mut self, input: Input) -> Result<Response<()>, PendingInput> {
        if !self.bootstrapped || !self.presented || self.needs_settle() {
            return Err(PendingInput(input));
        }
        Ok(self.handle_ready(RoutedInput::Input(input)))
    }

    pub fn handle_routed(
        &mut self,
        input: RoutedInput,
    ) -> Result<Response<()>, PendingRoutedInput> {
        if !self.bootstrapped || !self.presented || self.needs_settle() {
            return Err(PendingRoutedInput(input));
        }
        Ok(self.handle_ready(input))
    }

    fn handle_ready(&mut self, input: RoutedInput) -> Response<()> {
        self.type_ahead_focus_changed = false;
        let activation_key = match &input {
            RoutedInput::Input(Input::Key(key)) if key.mods.is_empty() => match key.code {
                KeyCode::Enter => Some(ActivationKey::Enter),
                KeyCode::Char(' ') => Some(ActivationKey::Space),
                _ => None,
            },
            _ => None,
        };
        self.services.diagnostics.clear();
        self.sync_keymap(None);
        self.refresh_keymap_conflicts();
        self.services
            .diagnostics
            .extend(self.keymap_conflicts.iter().cloned());
        self.services.repaint = false;
        self.intents.clear();
        self.services.pending_wheel_route = None;
        self.services.active_wheel_route = None;
        self.services
            .wheel_route_ack
            .set(crate::ui::cx::WheelRouteAck::Unset);
        self.services.wheel_route_invalidated = false;
        self.services.registry_gen = self.last.registry.generation();
        self.pump_layer_events();
        let key_input_top_layer = self.services.layers.top();
        let key_input_focus = self.focus.current();
        let mut key_input = None;
        let mut deferred_type_ahead = None;
        let mut capture_response = None;
        let mut wheel_route_attempt = None;
        let mut disabled_primary_refusal = false;
        let mut update_cause = UpdateCause::Event;
        match &input {
            RoutedInput::Input(Input::Resize(w, h)) => {
                self.resize(*w, *h);

                let r = self.run_update(None, UpdateCause::Event, activation_key)
                    | Response::changed().relayout();
                drop(input);
                return self.finish(r);
            }
            RoutedInput::Input(Input::Tick) => {
                update_cause = UpdateCause::Tick;
                self.enqueue_due_text_drag();
            }
            RoutedInput::Input(Input::Key(k)) => {
                if k.code == KeyCode::Esc
                    && k.mods.is_empty()
                    && self.top_transparent_tooltip_allows_escape()
                {
                    self.prepare_keyboard_input();
                    if self.dismiss_top(DismissReason::Esc) {
                        let r = self
                            .run_update(None, UpdateCause::Event, activation_key)
                            .repaint();
                        drop(input);
                        return self.finish(r);
                    }
                }
                let original_type_ahead = self.deferred_type_ahead_key(*k);
                let swallows = self.swallows_typing() || self.last.typing.fallback.is_some();
                let scoped = self
                    .last
                    .typing
                    .owner
                    .and_then(|owner| self.core.keymap.before_typing(owner, k));
                if let Some(cmd) =
                    scoped.or_else(|| self.core.keymap.lookup(KeyPhase::Capture, k, swallows))
                {
                    let response = self.run_update(Some(cmd), UpdateCause::Event, activation_key);
                    let Some(deferred) = original_type_ahead else {
                        drop(input);
                        return self.finish(response);
                    };
                    if response.is_consumed()
                        || !self.deferred_type_ahead_is_current(deferred, false)
                    {
                        drop(input);
                        return self.finish(response);
                    }
                    capture_response = Some(response);
                    key_input = Some(*k);
                    deferred_type_ahead = Some(deferred);
                    self.enqueue_key(*k, deferred_type_ahead);
                } else {
                    key_input = Some(*k);
                    deferred_type_ahead = original_type_ahead;
                    self.enqueue_key(*k, deferred_type_ahead);
                }
            }
            RoutedInput::Input(Input::Mouse(m)) => {
                disabled_primary_refusal = self.enqueue_mouse(*m);
                wheel_route_attempt = self.services.pending_wheel_route.take();
            }
            RoutedInput::Input(Input::Paste(s)) => {
                let consumed = crate::test_clock_enabled()
                    && crate::parse_test_clock_step(s)
                        .is_some_and(|step_ms| self.app.accept_paused_clock_step(step_ms));
                if !consumed
                    && let Some(owner) = self.last.typing.owner.or(self.last.typing.paste_owner)
                {
                    self.intents.paste(owner, s.as_str());
                }
            }
            RoutedInput::MiddleClick { pos, mods } => {
                self.enqueue_middle_click(*pos, *mods);
            }
            _ => {}
        }

        let (input_top_layer, input_focus) = if capture_response.is_some() {
            (key_input_top_layer, key_input_focus)
        } else {
            (self.services.layers.top(), self.focus.current())
        };
        let input_pointer_layer = self.pointer_input_layer();
        let swallow_middle = matches!(&input, RoutedInput::MiddleClick { .. })
            && input_pointer_layer != LayerId::PAGE;
        let wheel_route_guard = wheel_route_attempt.map(|_| WheelRouteGuard {
            focus: self.focus.current(),
            top_layer: self.services.layers.top(),
            input_layer: self.pointer_input_layer(),
            registry_generation: self.last.registry.generation(),
            capture: self.services.capture.get(),
        });
        if let Some(attempt) = wheel_route_attempt {
            self.services.active_wheel_route = Some(attempt);
            self.services
                .wheel_route_ack
                .set(crate::ui::cx::WheelRouteAck::Unset);
            self.services.wheel_route_invalidated = false;
        }
        let mut r = if let Some(capture_response) = capture_response {
            capture_response | self.run_update(None, update_cause, activation_key)
        } else {
            self.run_update(None, update_cause, activation_key)
        };
        if disabled_primary_refusal && !r.is_consumed() {
            r |= Response::consumed();
        }
        if swallow_middle {
            r |= Response::consumed();
        }
        if let (Some(attempt), Some(guard)) = (wheel_route_attempt, wheel_route_guard) {
            r = self.service_ignored_wheel_route(attempt, guard, r);
        }
        let mut input_scope_changed =
            self.services.layers.top() != input_top_layer || self.focus.current() != input_focus;
        if let Some(deferred) = deferred_type_ahead
            && !self.deferred_type_ahead_is_current(deferred, input_scope_changed)
        {
            input_scope_changed = true;
        }

        if let Some(k) = key_input
            && !r.is_consumed()
        {
            let scoped_bindings = self.scoped_bubble_bindings(&k);
            for (owner, action, chord) in scoped_bindings {
                if self.services.layers.top() != input_top_layer
                    || self.focus.current() != input_focus
                    || deferred_type_ahead.is_some_and(|deferred| {
                        !self.deferred_type_ahead_is_current(deferred, input_scope_changed)
                    })
                {
                    input_scope_changed = true;
                    break;
                }
                if !self.scoped_bubble_binding_is_current(&k, owner, action, chord) {
                    continue;
                }
                self.intents.clear();
                self.intents.binding(owner, action, chord);
                r |= self.run_update(None, UpdateCause::Event, activation_key);
                if r.is_consumed() {
                    break;
                }
                if self.services.layers.top() != input_top_layer
                    || self.focus.current() != input_focus
                    || deferred_type_ahead.is_some_and(|deferred| {
                        !self.deferred_type_ahead_is_current(deferred, input_scope_changed)
                    })
                {
                    input_scope_changed = true;
                    break;
                }
            }
            self.intents.clear();
            if !input_scope_changed
                && !r.is_consumed()
                && deferred_type_ahead.is_none_or(|deferred| {
                    self.deferred_type_ahead_is_current(deferred, input_scope_changed)
                })
                && let Some(cmd) = self.core.keymap.lookup(KeyPhase::Bubble, &k, false)
            {
                r |= self.run_update(Some(cmd), UpdateCause::Event, activation_key);
            } else if !input_scope_changed && !r.is_consumed() && k.code == KeyCode::Esc {
                let dismissable = self
                    .services
                    .layers
                    .top_layer()
                    .is_some_and(|l| l.spec.dismiss.esc);
                if dismissable {
                    self.dismiss_top(DismissReason::Esc);
                    r |= self
                        .run_update(None, UpdateCause::Event, activation_key)
                        .repaint();
                } else {
                    self.services.tooltips = self.app.tooltips();
                    let edit_entry = self.app.edit_entry();
                    let esc = {
                        let mut cx = Cx::new_with_cause(
                            &self.intents,
                            &mut self.services,
                            &mut self.core,
                            &self.last,
                            &self.theme,
                            None,
                            UpdateCause::Event,
                        )
                        .with_edit_entry(edit_entry);
                        self.app.on_esc(&mut cx)
                    };
                    r |= esc;
                    self.settle_closed_layers();
                    if let Some(f) = self.services.focus_request.take() {
                        self.stage_focus(Some(f), FocusVia::Programmatic);
                        self.apply_staged_focus();
                    }
                }
            }
        }
        if let Some(deferred) = deferred_type_ahead
            && !r.is_consumed()
            && self.deferred_type_ahead_is_current(deferred, input_scope_changed)
        {
            self.intents.clear();
            self.intents.key(deferred.owner, deferred.key);
            r |= self.run_update(None, UpdateCause::Event, activation_key);
        }
        drop(input);
        self.finish(r)
    }

    fn scoped_bubble_bindings(
        &self,
        key: &crate::event::Key,
    ) -> Vec<(crate::Id, crate::ActionKey, crate::event::Chord)> {
        let Some(focus) = self.last.snapshot.focus else {
            return Vec::new();
        };
        let Some(entry) = self.last.ring.entry(focus) else {
            return Vec::new();
        };
        if !self.last.ring.contains(focus) || entry.layer != self.services.layers.top() {
            return Vec::new();
        }
        let mut bindings: Vec<_> = self
            .last
            .scoped_bubble_bindings
            .iter()
            .filter(|published| {
                published.layer == entry.layer
                    && self.last.ring.scope_layer(published.scope) == Some(entry.layer)
                    && self.last.ring.within(entry.scope, published.scope)
            })
            .filter_map(|published| {
                let (_, descriptors) = self.last.bindings.get(published.owner)?;
                let (action, chord) =
                    self.core
                        .keymap
                        .component_binding(published.owner, descriptors, key)?;
                Some((
                    self.last.ring.scope_depth(published.scope)?,
                    published.owner,
                    action,
                    chord,
                ))
            })
            .collect();
        bindings.sort_by_key(|(depth, _, _, _)| core::cmp::Reverse(*depth));
        bindings
            .into_iter()
            .map(|(_, owner, action, chord)| (owner, action, chord))
            .collect()
    }

    fn scoped_bubble_binding_is_current(
        &self,
        key: &crate::event::Key,
        owner: crate::Id,
        action: crate::ActionKey,
        chord: crate::event::Chord,
    ) -> bool {
        let Some(focus) = self.focus.current() else {
            return false;
        };
        let Some(entry) = self.last.ring.entry(focus) else {
            return false;
        };
        let top = self.services.layers.top();
        if self.last.snapshot.focus != Some(focus)
            || !self.last.ring.contains(focus)
            || entry.layer != top
        {
            return false;
        }
        let scope_is_current = self.last.scoped_bubble_bindings.iter().any(|published| {
            published.owner == owner
                && published.layer == top
                && self.last.ring.scope_layer(published.scope) == Some(top)
                && self.last.ring.within(entry.scope, published.scope)
        });
        if !scope_is_current {
            return false;
        }
        let Some((_, descriptors)) = self.last.bindings.get(owner) else {
            return false;
        };
        self.core.keymap.component_binding(owner, descriptors, key) == Some((action, chord))
    }

    fn finish(&mut self, mut r: Response<()>) -> Response<()> {
        self.services.pending_wheel_route = None;
        self.services.active_wheel_route = None;
        self.services.wheel_route_ack.set(WheelRouteAck::Unset);
        self.services.wheel_route_invalidated = false;
        self.apply_text_drag_cancellations();
        self.reconcile_held_pointer();
        if core::mem::take(&mut self.services.reset_pointer_clicks) {
            self.inter.last_click = None;
            self.services.pointer_press_count = None;
        }
        if self.services.repaint {
            r = r.repaint();
        }
        self.intents.clear();
        self.last_invalidate = r.invalidate();
        r
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) -> PaintedFrame<'_, A> {
        self.refuse_legacy_continuation_draw();
        let area = frame.area();
        let buf = frame.buffer_mut();
        self.draw_with_buffer(area, buf, None, A::draw);
        if let Some(cursor) = self.painted_cursor {
            frame.set_cursor_position(cursor);
        }
        PaintedFrame {
            runtime: self,
            committed: false,
        }
    }

    fn draw_with_buffer(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        snapshot: Option<&RenderSnapshot>,
        paint: impl FnOnce(&A, &mut Ui<'_>),
    ) {
        self.invalidate_prepared();
        self.sync_keymap(snapshot.map(|value| &value.keymap));

        let same_snapshot = match (&self.cache_snapshot, snapshot) {
            (None, None) => true,
            (Some(previous), Some(next)) => std::sync::Arc::ptr_eq(previous, &next.cache_identity),
            _ => false,
        };
        if area != self.frame.screen || !same_snapshot {
            self.core.clear_caches();
        }
        if !same_snapshot {
            self.cache_snapshot =
                snapshot.map(|value| std::sync::Arc::clone(&value.cache_identity));
        }

        self.generation = self.generation.wrapping_add(1);
        self.core.begin_cache_frame(self.generation);
        self.core.style_cache.clear();
        self.frame.reset(self.generation, area);
        self.frame.tooltips = snapshot.map_or(self.services.tooltips, |value| value.tooltips);
        self.frame.managed_tooltip_opening = if snapshot.is_none() {
            self.services.managed_tooltip.and_then(|managed| {
                self.services
                    .layers
                    .top_layer()
                    .is_some_and(|layer| layer.opening == managed.opening)
                    .then_some(managed.opening)
            })
        } else {
            None
        };
        self.frame.inert_floor = snapshot.map_or_else(
            || self.services.layers.inert_floor(),
            |value| value.inert_floor,
        );
        self.frame.top = snapshot.map_or_else(|| self.services.layers.top(), |value| value.top);

        let layers = snapshot.map_or_else(
            || self.services.layers.layers(),
            |value| value.layers.as_slice(),
        );
        for l in layers {
            self.frame.layer_openings.push(l.opening);
            let rect = resolve_layer_area(area, l.spec);
            self.frame.layers.push(l.id, l.layer, l.spec, rect, area);
        }

        {
            let app = &self.app;
            let mut ui = Ui::new(
                &mut self.frame,
                buf,
                &mut self.core,
                &self.theme,
                snapshot.map_or(&self.last, |value| &value.last),
            );
            paint(app, &mut ui);

            for i in 0..ui_layer_count(&ui) {
                composite_layer(&mut ui, i, area);
            }
        }
        let focus =
            snapshot.map_or_else(|| self.focus.current(), |value| value.last.snapshot.focus);
        self.frame.typing_resolved = typing::resolve(&mut self.frame, focus);
        let retained_request = self.frame.retained_cursor_request;
        let retained_cursor_owner = retained_request
            .filter(|retained| {
                if retained.ambiguous {
                    return false;
                }
                let Some(top) = self.frame.layers.active().last() else {
                    return false;
                };
                let Some(request) = self.frame.cursors.get(retained.cursor_index) else {
                    return false;
                };
                let Some(area) = self.frame.registry.area_of(retained.owner) else {
                    return false;
                };
                (retained.mode == crate::ui::CursorRetentionMode::VisibleOwner
                    || (focus == Some(retained.owner)
                        && self.focus.current() == Some(retained.owner)))
                    && self.frame.top == retained.overlay_layer
                    && top.layer == retained.overlay_layer
                    && top.drawn
                    && top.spec.kind == crate::layer::LayerKind::Popover
                    && !top.spec.inert_below
                    && retained.parent_layer >= self.frame.inert_floor
                    && self.frame.registry.layer_of(retained.owner) == Some(retained.parent_layer)
                    && request.owner == retained.owner
                    && request.layer == retained.overlay_layer
                    && !request.inert
                    && !request.typing_offer
                    && self.frame.screen.contains(request.pos)
                    && !area.is_empty()
                    && area.contains(request.pos)
            })
            .map(|retained| retained.owner);
        let cursor_owner =
            select_cursor_owner(&self.frame, retained_request, retained_cursor_owner, focus);
        let resolved = cursor::resolve_requests(
            &self.frame.cursors,
            self.frame.top,
            cursor_owner,
            self.frame.typing_resolved.owner,
            &mut self.frame.diagnostics,
            &self.frame.typing,
        );
        self.painted_cursor = resolved.map(|(pos, _)| pos);
        self.painted_cursor_bar =
            resolved.is_some_and(|(_, shape)| shape == cursor::CursorShape::Bar);
    }

    fn commit_geometry(&mut self) {
        self.services.viewport = self.frame.screen;

        self.services.diagnostics.append(&self.frame.diagnostics);
        core::mem::swap(&mut self.last.registry, &mut self.frame.registry);
        self.last.layout.clear();
        self.last.layout.append(&mut self.frame.layout);
        self.last.declared.clear();
        self.last.declared.append(&mut self.frame.declared);
        core::mem::swap(&mut self.last.bindings, &mut self.frame.bindings);
        core::mem::swap(
            &mut self.last.scoped_bubble_bindings,
            &mut self.frame.scoped_bubble_bindings,
        );
        core::mem::swap(
            &mut self.last.typing_bindings,
            &mut self.frame.typing_bindings,
        );
        core::mem::swap(&mut self.last.type_aheads, &mut self.frame.type_aheads);
        self.last.typing = self.frame.typing_resolved;
    }

    fn commit_frame(&mut self) {
        let initial = self.services.layers.top_layer().and_then(|layer| {
            let target = layer.spec.initial_focus?;
            self.provisional_focus_layer(target)
                .map(|layer| (target, layer.scope()))
        });
        self.services.initial_focus_layer = None;
        self.commit_geometry();
        self.presented = true;
        self.retire_unpublished_managed_tooltip();

        let previous = self.focus.current();
        let restoration = self.restore_after_publication.take();
        let restoring = restoration.is_some() && self.services.deferred_focus.is_none();
        let backwards = self
            .services
            .deferred_focus
            .is_some_and(|request| request.backwards);
        let reconciled = if let Some(request) = self.services.deferred_focus.take() {
            if request
                .anchor
                .is_some_and(|id| self.frame.ring.contains(id))
            {
                if request.backwards {
                    self.frame.ring.prev(request.anchor)
                } else {
                    self.frame.ring.next(request.anchor)
                }
            } else {
                self.frame.ring.reconcile(&self.last.ring, request.anchor)
            }
        } else if let Some(restoration) = restoration {
            self.frame
                .ring
                .reconcile(&self.last.ring, restoration.target)
        } else {
            self.frame.ring.reconcile(&self.last.ring, previous)
        };
        core::mem::swap(&mut self.last.ring, &mut self.frame.ring);
        let admissible = |id| {
            self.focus_target_admissible(id)
                && initial.is_none_or(|(target, scope)| {
                    id != target
                        || self
                            .last
                            .ring
                            .entry(id)
                            .is_some_and(|entry| self.last.ring.within(entry.scope, scope))
                })
        };
        let reconciled = reconciled.and_then(|id| {
            if admissible(id) {
                return Some(id);
            }

            let entries = self.last.ring.entries();
            let after = entries
                .iter()
                .position(|entry| entry.id == id)
                .map_or(0, |i| i.saturating_add(1));
            if backwards {
                entries
                    .iter()
                    .take(after.saturating_sub(1))
                    .rev()
                    .chain(entries.iter().skip(after.saturating_sub(1)).rev())
                    .find(|entry| admissible(entry.id))
                    .map(|entry| entry.id)
            } else {
                entries
                    .iter()
                    .skip(after)
                    .chain(entries.iter().take(after))
                    .find(|entry| admissible(entry.id))
                    .map(|entry| entry.id)
            }
        });
        if reconciled != previous {
            let via = if restoring
                || self
                    .staged_focus
                    .is_some_and(|(_, v)| v == FocusVia::Restore)
            {
                FocusVia::Restore
            } else {
                FocusVia::Programmatic
            };

            let pending_from = self.pending_focus.map_or_else(
                || PendingFocus {
                    from: previous,
                    to: reconciled,
                    via,

                    from_layer: previous.and_then(|id| self.frame.ring.entry(id).map(|e| e.layer)),
                },
                |pending| PendingFocus {
                    to: reconciled,
                    via,
                    ..pending
                },
            );
            self.pending_focus = (pending_from.from != reconciled).then_some(pending_from);
            if self.focus.current() != reconciled {
                self.type_ahead_focus_changed = true;
            }
            self.focus.set(reconciled);

            self.invalidate_prepared();
            self.services.repaint = true;
        }
        self.staged_focus = None;
        self.last.snapshot.focus = self.focus.current();
        if self.pending_focus.is_none() && !self.services.focus_continuation_result_acks.is_empty()
        {
            let mut became_ready = false;
            for (_, _, ready) in &mut self.services.focus_continuation_result_acks {
                became_ready |= !*ready;
                *ready = true;
            }
            self.completion_dispatch_pending |= became_ready;
        }
        if self.pending_focus.is_none()
            && (!self.completion_waiting.is_empty()
                || !self.continuation_cancels_waiting.is_empty())
        {
            let restored_focus = self.focus.current();
            let mut dispatch_completion = false;
            let canceled = core::mem::take(&mut self.continuation_cancels_waiting);
            for (opening, owner) in canceled {
                self.store_focus_continuation_outcome(FocusContinuationOutcome::new(
                    opening,
                    owner,
                    restored_focus,
                    false,
                ));
                dispatch_completion = true;
            }
            let waiting = core::mem::take(&mut self.completion_waiting);
            for layer in waiting {
                let Some(event) = layer.closed_event else {
                    continue;
                };
                let continuation_acknowledged =
                    layer.spec.continues_focus_owner.is_some_and(|owner| {
                        let parent = layer.opening.parent();
                        let parent_layer = parent.map_or(LayerId::PAGE, |parent| parent.layer);
                        let parent_is_current = match parent {
                            None => self.services.layers.top() == LayerId::PAGE,
                            Some(parent) => self.services.layers.layers().iter().any(|open| {
                                open.id == parent.id
                                    && open.layer == parent.layer
                                    && open.opening.generation() == parent.generation
                                    && self.services.layers.top() == open.layer
                            }),
                        };
                        let owner_is_live =
                            self.last.ring.entry(owner).is_some_and(|entry| {
                                entry.layer == parent_layer && !entry.disabled
                            });
                        let opening_not_replaced = self.services.layers.get(layer.id).is_none();
                        restored_focus == Some(owner)
                            && parent_is_current
                            && owner_is_live
                            && opening_not_replaced
                            && self.services.focus_continuation_acks.iter().any(
                                |(opening, ack_owner, _)| {
                                    *opening == layer.opening && *ack_owner == owner
                                },
                            )
                    });
                if let Some(owner) = layer.spec.continues_focus_owner {
                    let outcome = FocusContinuationOutcome::new(
                        layer.opening,
                        owner,
                        restored_focus,
                        continuation_acknowledged,
                    );
                    self.store_focus_continuation_outcome(outcome);
                }
                if self.services.completions.len() == 64 {
                    let oldest = self.services.completions.remove(0).0;
                    self.services
                        .diagnostics
                        .push(Diagnostic::FocusContinuationUnacknowledged {
                            owner: oldest.opening().owner(),
                        });
                }
                self.services.completions.push((
                    LayerCompletion::new(
                        layer.opening,
                        event,
                        restored_focus,
                        layer.spec.continues_focus_owner,
                        continuation_acknowledged,
                    ),
                    0,
                ));
                dispatch_completion = true;
                self.services
                    .focus_continuation_acks
                    .retain(|(opening, _, _)| *opening != layer.opening);
            }
            self.completion_dispatch_pending |= dispatch_completion;
        }
        self.last.snapshot.focus_visible = self.focus.visible();
        self.reconcile_held_pointer();
        self.reconcile_hover();

        self.cursor = self.painted_cursor;
    }

    fn retire_unpublished_managed_tooltip(&mut self) {
        let Some(managed) = self.services.managed_tooltip else {
            return;
        };
        let frame_proves_draw = self.frame.managed_tooltip_opening == Some(managed.opening)
            && self.frame.managed_tooltip_drawn;
        let stack_has_top = self
            .services
            .layers
            .top_layer()
            .is_some_and(|layer| layer.opening == managed.opening);
        if frame_proves_draw && stack_has_top && self.managed_tooltip_source_is_live(managed) {
            return;
        }

        if stack_has_top {
            self.dismiss_layer(managed.opening.id(), DismissReason::Programmatic);
        } else {
            self.services.managed_tooltip = None;
        }
        self.pending_managed_tooltip_settle = true;
        self.presented = false;
        self.invalidate_prepared();
        self.services.repaint = true;
    }

    fn store_focus_continuation_outcome(&mut self, outcome: FocusContinuationOutcome) {
        if self.services.focus_continuation_outcomes.len() == 64 {
            let oldest = self.services.focus_continuation_outcomes.remove(0).0;
            self.services
                .diagnostics
                .push(Diagnostic::FocusContinuationUnacknowledged {
                    owner: oldest.owner(),
                });
        }
        self.services.focus_continuation_outcomes.push((outcome, 0));
    }

    fn reconcile_held_pointer(&mut self) {
        let capture = self.services.capture.get();
        self.services.capture.release_if_stale(&self.last.registry);
        let eligible = |owner, part| {
            crate::capture::target_area(
                &self.last.registry,
                &self.last.ring,
                self.pointer_input_layer(),
                owner,
                part,
            )
            .is_some()
        };
        let capture_invalid = capture.is_some_and(|capture| {
            self.services.capture.get().is_none() || !eligible(capture.owner, capture.part)
        });
        let press_invalid = self
            .inter
            .press
            .is_some_and(|press| !eligible(press.owner, press.part));
        if capture_invalid {
            self.services.capture.release();
            self.services.reset_pointer_clicks = true;
        }
        if capture_invalid || press_invalid {
            self.inter.press = None;
            self.services.press_pos = None;
            self.inter.drag_motion = None;
        }
        if let Some((owner, part)) = self.cancelled_text_drag
            && !self
                .services
                .capture
                .get()
                .is_some_and(|capture| capture.owner == owner && capture.part == part)
        {
            self.cancelled_text_drag = None;
        }

        let pressed = self
            .inter
            .press
            .map(|press| (press.owner, press.part))
            .or_else(|| self.services.feedback.pressed());
        let capture = self.services.capture.get().map(|capture| capture.owner);
        if self.last.snapshot.capture != capture || self.last.snapshot.pressed != pressed {
            self.invalidate_prepared();
            self.services.repaint = true;
        }
        self.last.snapshot.capture = capture;
        self.last.snapshot.pressed = pressed;
        self.reconcile_held_text_drag();
    }

    fn live_text_drag(&self) -> Option<LiveTextDrag> {
        let capture = self.services.capture.get()?;
        let press = self.inter.press?;
        let motion = self.inter.drag_motion?;
        if press.owner != capture.owner || press.part != capture.part {
            return None;
        }
        if self.cancelled_text_drag == Some((capture.owner, capture.part)) {
            return None;
        }
        let region = self.last.registry.text_drag_region(
            capture.owner,
            capture.part,
            self.pointer_input_layer(),
        )?;
        if region.generation() != self.last.registry.generation()
            || capture.generation > region.generation()
            || !region.can_scroll_at(motion.position)
            || crate::capture::target_area(
                &self.last.registry,
                &self.last.ring,
                self.pointer_input_layer(),
                capture.owner,
                capture.part,
            )
            .is_none()
        {
            return None;
        }
        Some(LiveTextDrag {
            capture,
            press,
            motion,
            region,
        })
    }

    fn text_drag_deadline(now: Moment) -> Option<Moment> {
        let deadline = now.saturating_add(TEXT_DRAG_AUTOSCROLL_INTERVAL);
        (deadline > now).then_some(deadline)
    }

    fn same_text_drag_registration(left: TextDragRegion, right: TextDragRegion) -> bool {
        left.owner() == right.owner()
            && left.part() == right.part()
            && left.layer() == right.layer()
            && left.area() == right.area()
            && left.axes() == right.axes()
            && left.source_revision() == right.source_revision()
    }

    fn reconcile_held_text_drag(&mut self) {
        let previous = self.held_text_drag;
        let Some(live) = self.live_text_drag() else {
            self.held_text_drag = None;
            return;
        };
        let next_due = match previous
            .filter(|held| Self::same_text_drag_registration(held.region, live.region))
        {
            Some(held) => held.next_due,
            None => Self::text_drag_deadline(self.now()),
        };
        self.held_text_drag = Some(HeldTextDrag {
            region: live.region,
            position: live.motion.position,
            modifiers: live.motion.modifiers,
            click_count: live.press.click_count,
            next_due,
        });
    }

    fn enqueue_due_text_drag(&mut self) {
        let Some(held) = self.held_text_drag else {
            return;
        };
        if !held.next_due.is_some_and(|deadline| deadline <= self.now()) {
            return;
        }
        let Some(live) = self.live_text_drag() else {
            self.held_text_drag = None;
            return;
        };
        if !Self::same_text_drag_registration(held.region, live.region)
            || held.position != live.motion.position
        {
            self.held_text_drag = None;
            self.reconcile_held_text_drag();
            return;
        }

        self.intents.pointer(
            live.capture.owner,
            Phase::Drag,
            live.capture.part,
            held.position,
            live.capture.local(held.position),
            held.modifiers,
        );
        self.services.pointer_press_count =
            Some((live.press.owner, live.press.part, held.click_count));
        self.held_text_drag = Some(HeldTextDrag {
            region: live.region,
            position: live.motion.position,
            modifiers: live.motion.modifiers,
            click_count: live.press.click_count,
            next_due: Self::text_drag_deadline(self.now()),
        });
    }

    fn apply_text_drag_cancellations(&mut self) {
        let cancellations = core::mem::take(&mut self.services.text_drag_cancellations);
        for (owner, part) in cancellations {
            let captured = self
                .services
                .capture
                .get()
                .is_some_and(|capture| capture.owner == owner && capture.part == part);
            let pressed = self
                .inter
                .press
                .is_some_and(|press| press.owner == owner && press.part == part);
            let held = self
                .held_text_drag
                .is_some_and(|drag| drag.region.owner() == owner && drag.region.part() == part);
            if captured || pressed || held {
                self.cancelled_text_drag = Some((owner, part));
                self.held_text_drag = None;
            }
        }
    }

    fn reconcile_hover(&mut self) {
        let painted_hover = if self.last.snapshot.hover_suppressed {
            None
        } else {
            self.last.snapshot.hover
        };
        let mut hover = if self.inter.hover_suppressed || self.services.capture.get().is_some() {
            None
        } else {
            self.inter.pointer_position.and_then(|position| {
                self.pointer_live_hit(position)
                    .map(|hit| (hit.owner, hit.part))
            })
        };
        if hover == painted_hover {
            self.inter.hover_passes = 0;
        } else {
            self.inter.hover_passes = self.inter.hover_passes.saturating_add(1);
            if self.inter.hover_passes == 4 {
                self.inter.hover_suppressed = true;
                self.last.snapshot.hover_suppressed = true;
                hover = None;
                self.services
                    .diagnostics
                    .push(Diagnostic::HoverLayoutDidNotSettle);
                self.invalidate_prepared();
                self.services.repaint = true;
            }
        }
        self.inter.hover = hover;
        self.last.snapshot.hover = hover;
        if hover != painted_hover {
            self.invalidate_prepared();
            self.services.repaint = true;
        }
    }

    pub fn render_snapshot(&self) -> Result<RenderSnapshot, RenderSnapshotError> {
        if !self.bootstrapped {
            return Err(RenderSnapshotError::Uninitialized);
        }
        if self.needs_settle() {
            return Err(RenderSnapshotError::NeedsSettle);
        }
        if self.needs_present() {
            return Err(RenderSnapshotError::NeedsPresentation);
        }
        Ok(RenderSnapshot {
            cache_identity: std::sync::Arc::new(()),
            last: self.last.clone(),
            layers: self.services.layers.layers().to_vec(),
            inert_floor: self.services.layers.inert_floor(),
            top: self.services.layers.top(),
            keymap: self.core.keymap.clone(),
            tooltips: self.services.tooltips,
        })
    }

    pub const fn cursor_position(&self) -> Option<Position> {
        self.cursor
    }

    pub const fn cursor_bar(&self) -> bool {
        self.painted_cursor_bar
    }
}

fn ui_layer_count(ui: &Ui<'_>) -> usize {
    ui.layer_count()
}

fn composite_layer(ui: &mut Ui<'_>, i: usize, screen: Rect) {
    let Some((backdrop, drawn)) = ui.layer_meta(i) else {
        return;
    };
    if drawn && matches!(backdrop, Backdrop::Dim { .. }) {
        let area = backdrop_area(screen, backdrop);
        ui.modal_backdrop(area);
    }
    if drawn {
        ui.composite(i);
    }
}
