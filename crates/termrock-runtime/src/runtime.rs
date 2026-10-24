pub mod clipboard;
pub(crate) mod feedback;
#[cfg(feature = "crossterm")]
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
    LayerEventReceipt, LayerId, OpenLayer, PendingLayerEvent, backdrop_area, resolve_layer_area,
};
use crate::measure::Size;
#[cfg(any(test, feature = "testing"))]
use crate::response::StateFlags;
use crate::response::{Invalidate, Response};
use crate::theme::Theme;
use crate::ui::cx::{FrameServices, LastFrame, WheelRouteAck};
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

    /// Select the keyboard-protocol policy used by a normal session launch.
    ///
    /// The default honors an explicit `TERMROCK_KEYBOARD_PROTOCOL` or legacy
    /// `JUNIE_TUI_KEYBOARD_PROTOCOL` override, and stays Off when neither is
    /// set. An application can return `Auto`, `On`, or `Off` directly; that
    /// application choice takes precedence over environment overrides.
    fn keyboard_protocol(&self) -> termrock_core::keys::keyboard::KeyboardProtocol {
        default_app_keyboard_protocol(
            termrock_core::keys::keyboard::KeyboardProtocol::current_override(),
        )
    }

    /// Service one explicitly addressed, opt-in wheel fallback attempt.
    ///
    /// The callback receives no ordinary input intents and must dispatch only
    /// the named wheel registration. It must acknowledge the attempt through
    /// [`Cx::acknowledge_wheel_route`] after checking the live source revision.
    /// The returned response contributes repaint or layout invalidation only;
    /// it does not describe whether this target handled the wheel.
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

    /// Default policy used by text editors when they receive focus or typing.
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

    /// Accept one visual-baseline clock step while motion is paused.
    ///
    /// The runtime calls this only when [`crate::test_clock_enabled`] is true and
    /// [`crate::parse_test_clock_step`] returns `step_ms`. Return true only when motion is
    /// paused and the step is accepted. True consumes the paste: it is not delivered to the
    /// typing owner. The default reports that motion is not paused and does not consume.
    #[must_use = "false leaves the paste on the typing-owner path"]
    fn accept_paused_clock_step(&mut self, _step_ms: u64) -> bool {
        false
    }
}

/// Local result from one exact opted-in wheel target.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelRouteResult {
    /// This target cannot handle the wheel and made no wheel-related change.
    Ignored,
    /// This target accepted the wheel without reporting a model change.
    Consumed,
    /// This target changed its model in response to the wheel.
    Changed,
}

/// Opaque identity and immutable input data for one wheel-route attempt.
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
            position,
            modifiers,
            registry_generation: target.registry_generation,
            source_revision: target.source_revision,
        }
    }

    /// Return the private event identity shared by attempts for one physical wheel.
    pub const fn event_id(self) -> u64 {
        self.event_id
    }

    /// Return this target's position in the bounded route.
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

    /// Return the already-scaled delta shared by every target in this route.
    pub const fn delta(self) -> i16 {
        self.delta
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
    /// Targeted, empty-intent wheel fallback service.
    ScrollRoute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Press {
    owner: Id,
    part: PartRef,
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
}

#[cfg(feature = "testing")]
pub struct RenderModel<'a, M: ?Sized, F> {
    model: &'a M,
    paint: F,
    snapshot: RenderSnapshot,
}

#[cfg(feature = "testing")]
impl<M: ?Sized, F> core::fmt::Debug for RenderModel<'_, M, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RenderModel").finish_non_exhaustive()
    }
}

#[cfg(feature = "testing")]
impl RenderSnapshot {
    pub fn bind_model<'a, M: ?Sized, F>(&self, model: &'a M, paint: F) -> RenderModel<'a, M, F>
    where
        F: Fn(&M, &mut Ui<'_>, Rect),
    {
        let mut snapshot = self.clone();
        snapshot.cache_identity = std::sync::Arc::new(());
        RenderModel {
            model,
            paint,
            snapshot,
        }
    }
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

#[cfg(feature = "testing")]
#[derive(Default)]
struct ProjectionFacts {
    last: LastFrame,
    cursor: Option<Position>,
}

#[cfg(feature = "testing")]
#[must_use = "acknowledge the completed projection to inspect its geometry"]
pub struct ProjectedFrame<'a, A: App> {
    runtime: &'a mut Runtime<A>,
    snapshot: crate::ui::cx::Snapshot,
}

#[cfg(feature = "testing")]
impl<A: App> core::fmt::Debug for ProjectedFrame<'_, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProjectedFrame").finish_non_exhaustive()
    }
}

#[cfg(feature = "testing")]
impl<A: App> ProjectedFrame<'_, A> {
    pub fn commit_inspected(self) {
        let output = self
            .runtime
            .projection
            .get_or_insert_with(ProjectionFacts::default);
        let frame = &mut self.runtime.frame;
        core::mem::swap(&mut output.last.registry, &mut frame.registry);
        core::mem::swap(&mut output.last.ring, &mut frame.ring);
        core::mem::swap(&mut output.last.bindings, &mut frame.bindings);
        core::mem::swap(
            &mut output.last.scoped_bubble_bindings,
            &mut frame.scoped_bubble_bindings,
        );
        core::mem::swap(&mut output.last.typing_bindings, &mut frame.typing_bindings);
        output.last.typing = frame.typing_resolved;
        output.last.layout.clear();
        output.last.layout.append(&mut frame.layout);
        output.last.declared.clear();
        output.last.declared.append(&mut frame.declared);
        output.last.snapshot = self.snapshot;
        output.cursor = self.runtime.painted_cursor;
    }
}

#[cfg(feature = "testing")]
impl<A: App> Drop for ProjectedFrame<'_, A> {
    fn drop(&mut self) {
        self.runtime.invalidate_prepared();
    }
}

pub struct PendingInput(Input);

impl PendingInput {
    pub fn into_input(self) -> Input {
        self.0
    }
}

/// An input rejected until the runtime has a settled, presented frame.
pub struct PendingRoutedInput(RoutedInput);

impl PendingRoutedInput {
    /// Recover the exact routed input for a later retry.
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

/// Opaque identity for one runtime-prepared terminal frame.
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

/// Focus transition retained until the next settle after publishing a frame.
/// Keep the originating layer identity because the new focus ring may no longer
/// contain a control from a layer that closed during publication.
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
    registry_generation: u32,
    capture: Option<Capture>,
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
    layer_events_pending: Vec<PendingLayerEvent>,
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
    #[cfg(feature = "testing")]
    projection: Option<ProjectionFacts>,
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
        Runtime {
            app,
            theme,
            pointer_click_slop: 1,
            last: LastFrame::default(),
            focus: FocusState::default(),
            services: FrameServices {
                runtime_id,
                feedback: feedback::FeedbackState::new(clock),
                ..FrameServices::default()
            },
            intents: IntentQueue::new(),
            inter: Interaction::default(),
            held_text_drag: None,
            cancelled_text_drag: None,
            pending_tick: false,
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
            #[cfg(feature = "testing")]
            projection: None,
        }
    }

    /// Set app-wide pointer click slop in terminal cells.
    ///
    /// Defaults to one cell, matching the frozen visual baseline. The click window remains
    /// configurable through `Theme::design.motion.double_click_ms`.
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

    fn top(&self) -> LayerId {
        self.services.layers.top()
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

        self.services.repaint = true;

        if via != FocusVia::Restore || self.restore_after_publication.is_none() {
            self.dismiss_on_focus_out(to);
        }
        true
    }

    fn focus_out_layer_is_live(&self, layer: Option<LayerId>) -> bool {
        match layer {
            // Unknown provenance preserves legacy blur delivery. PAGE is always live.
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

    fn enqueue_key(&mut self, k: Key) {
        self.inter.last_input_key = true;
        self.inter.hover_suppressed = true;
        self.focus.set_visible(true);
        self.last.snapshot.focus_visible = true;
        self.last.snapshot.hover_suppressed = true;
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
        if let Some(owner) = current {
            self.intents.key(owner, k);
        }
    }

    fn ring_proves_disabled(&self, id: Id) -> bool {
        self.last.ring.entry(id).is_some_and(|e| e.disabled)
    }

    fn deliverable(&self, hit: Hit) -> bool {
        crate::capture::target_eligible(&self.last.ring, self.top(), hit.owner, hit.layer, hit.kind)
    }

    fn is_disabled_primary_hit(&self, hit: Hit) -> bool {
        hit.layer == self.top()
            && matches!(hit.kind, RegionKind::Control | RegionKind::Part)
            && self
                .last
                .ring
                .entry(hit.owner)
                .is_some_and(|entry| entry.disabled && entry.layer == hit.layer)
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
        let hit = self.last.registry.hit(m.pos);
        let top = self.top();
        let outside = hit.is_none_or(|h| h.layer < top);
        let mut disabled_primary_refusal = false;
        match m.kind {
            MouseKind::Move => self.pointer_moved(m, top, hover_was_suppressed),
            MouseKind::Down => {
                if outside {
                    self.outside_click();
                    return false;
                }
                if let Some(h) = hit {
                    disabled_primary_refusal = self.is_disabled_primary_hit(h);
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
                if outside {
                    self.outside_click();
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
                for candidate in self.last.registry.scroll_candidates(m.pos, axis, top) {
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
                                .route_key_is_eligible(*key, axis, m.pos, top)
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
        let Some(hit) = self.last.registry.hit(pos) else {
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

    fn pointer_moved(&mut self, m: Mouse, top: LayerId, hover_was_suppressed: bool) {
        let live = self.last.registry.hit_live(m.pos, top);
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
        let same = hit.is_some_and(|h| h.owner == p.owner && h.part == p.part);
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

    /// Re-evaluate a potential multi-click at activation time. A long-held press
    /// keeps its down-time count for selection, but must not become a double-click
    /// after the configured interval has elapsed.
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

    fn outside_click(&mut self) {
        let Some(top) = self.services.layers.top_layer().copied() else {
            return;
        };
        if top.spec.dismiss.outside_click {
            self.dismiss_top(DismissReason::OutsideClick);
        }
    }

    fn dismiss_top(&mut self, reason: DismissReason) -> bool {
        let Some(top) = self.services.layers.top_layer().copied() else {
            return false;
        };
        let closed =
            self.services
                .layers
                .close_with_update(top.id, LayerEvent::Dismissed(reason), None);
        let changed = !closed.is_empty();
        if let Some(root) = closed.first() {
            self.services.direct_closed_layers.push(*root);
        }
        self.services.closed_layers.extend(closed);
        if changed {
            self.services.reset_pointer_clicks = true;
        }

        if reason == DismissReason::Esc {
            self.intents.cancel(top.spec.owner);
        }
        true
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
        // Completion is opt-in, and only direct closes can trigger it. A later direct close of
        // an ancestor supersedes a child receipt created earlier in the same update.
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
            let id = pending.opening.id();
            let ev = pending.event;
            let owner = pending.opening.owner();
            self.layer_events_pending.push(pending);
            self.intents.layer(owner, ev);
            if owner != id {
                self.intents.layer(id, ev);
            }
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
            self.services
                .events
                .extend(events.into_iter().map(|pending| LayerEventReceipt {
                    opening: pending.opening,
                    event: pending.event,
                    update: pending.update,
                }));
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

            self.layer_events_pending = core::mem::take(&mut self.services.events)
                .into_iter()
                .map(|receipt| PendingLayerEvent {
                    opening: receipt.opening,
                    event: receipt.event,
                    update: receipt.update,
                })
                .collect();

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
            || self.services.capture.get() != guard.capture
            || self.last.registry.generation() != guard.registry_generation
            || self.services.runtime_id != attempt.runtime_id
            || attempt.layer() != guard.top_layer
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
                current.position(),
                current.modifiers(),
            );
            self.services.active_wheel_route = Some(attempt);
            self.services.wheel_route_ack.set(WheelRouteAck::Unset);
            let empty_intents = IntentQueue::new();
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
            || (self.bootstrapped && self.pending_tick)
    }

    pub const fn needs_present(&self) -> bool {
        !self.presented
    }

    /// Settle runtime focus and update work against the supplied area and retain one final frame.
    /// No terminal output is performed here.
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
            // Geometry is published and focus has settled; input remains blocked until output is
            // acknowledged by the session after a successful terminal flush.
            self.presented = false;
            return Ok(token);
        }
        self.invalidate_prepared();
        Err(PreparedFrameError::DidNotSettle)
    }

    /// Copy a prepared frame into Ratatui's callback buffer without running application code.
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

    /// Mark a prepared frame as physically presented after the terminal backend succeeds.
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
            if completion_dispatch || focus_result_ack_dispatch {
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

    /// Handle one decoded terminal event, including supported middle presses.
    /// Inputs are retained unchanged when the runtime is not ready.
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
        let mut key_input = None;
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
                let swallows = self.swallows_typing() || self.last.typing.fallback.is_some();
                let scoped = self
                    .last
                    .typing
                    .owner
                    .and_then(|owner| self.core.keymap.before_typing(owner, k));
                if let Some(cmd) =
                    scoped.or_else(|| self.core.keymap.lookup(KeyPhase::Capture, k, swallows))
                {
                    let r = self.run_update(Some(cmd), UpdateCause::Event, activation_key);
                    drop(input);
                    return self.finish(r);
                }
                key_input = Some(*k);
                self.enqueue_key(*k);
            }
            RoutedInput::Input(Input::Mouse(m)) => {
                disabled_primary_refusal = self.enqueue_mouse(*m);
                wheel_route_attempt = self.services.pending_wheel_route.take();
            }
            RoutedInput::Input(Input::Paste(s)) => {
                let consumed = crate::test_clock_enabled()
                    && crate::parse_test_clock_step(s)
                        .is_some_and(|step_ms| self.app.accept_paused_clock_step(step_ms));
                if !consumed && let Some(owner) = self.last.typing.owner {
                    self.intents.paste(owner, s.as_str());
                }
            }
            RoutedInput::MiddleClick { pos, mods } => {
                self.enqueue_middle_click(*pos, *mods);
            }
            _ => {}
        }
        // A key event belongs to the input scope that was active when it
        // arrived. An update may open/close a layer or move focus; bubble
        // fallback for the same physical key must not then act on that new
        // scope.
        let input_top_layer = self.services.layers.top();
        let swallow_middle =
            matches!(&input, RoutedInput::MiddleClick { .. }) && input_top_layer != LayerId::PAGE;
        let input_focus = self.focus.current();
        let wheel_route_guard = wheel_route_attempt.map(|_| WheelRouteGuard {
            focus: self.focus.current(),
            top_layer: self.services.layers.top(),
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
        let mut r = self.run_update(None, update_cause, activation_key);
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

        if let Some(k) = key_input
            && !r.is_consumed()
        {
            let scoped_bindings = self.scoped_bubble_bindings(&k);
            for (owner, action, chord) in scoped_bindings {
                if self.services.layers.top() != input_top_layer
                    || self.focus.current() != input_focus
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
                {
                    input_scope_changed = true;
                    break;
                }
            }
            self.intents.clear();
            if !input_scope_changed
                && !r.is_consumed()
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
                    // The previous ring was moved into `frame` immediately above.
                    from_layer: previous.and_then(|id| self.frame.ring.entry(id).map(|e| e.layer)),
                },
                |pending| PendingFocus {
                    to: reconciled,
                    via,
                    ..pending
                },
            );
            self.pending_focus = (pending_from.from != reconciled).then_some(pending_from);
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
                self.top(),
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
        let region =
            self.last
                .registry
                .text_drag_region(capture.owner, capture.part, self.top())?;
        if region.generation() != self.last.registry.generation()
            || capture.generation > region.generation()
            || !region.can_scroll_at(motion.position)
            || crate::capture::target_area(
                &self.last.registry,
                &self.last.ring,
                self.top(),
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
                self.last
                    .registry
                    .hit_live(position, self.top())
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
        })
    }

    pub const fn cursor_position(&self) -> Option<Position> {
        self.cursor
    }

    /// The last present kept a steady bar. The session emits that shape.
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

#[cfg(feature = "testing")]
impl<A: App> Runtime<A> {
    pub fn hover(&self) -> Option<Id> {
        self.inter.hover.map(|(o, _)| o)
    }

    pub fn state_of(&self, id: Id) -> StateFlags {
        self.last.state(id)
    }

    pub const fn focus_visible(&self) -> bool {
        self.focus.visible()
    }

    pub fn top_layer(&self) -> LayerId {
        self.services.layers.top()
    }

    pub fn is_open(&self, id: Id) -> bool {
        self.services.layers.is_open(id)
    }

    pub const fn cursor(&self) -> Option<Position> {
        self.cursor
    }

    pub fn resolved(&self, id: Id, p: Part) -> crate::theme::Resolved {
        let mine = self.frame.styled_queries.iter().rev();
        if let Some((_, _, _, _, r)) = mine.clone().find(|(o, _, _, q, _)| *o == id && *q == p) {
            return *r;
        }
        let (f, v) = mine.clone().find(|(o, _, _, _, _)| *o == id).map_or(
            (crate::theme::Family::BUTTON, crate::theme::Variant::DEFAULT),
            |(_, f, v, _, _)| (*f, *v),
        );
        self.resolved_in(f, v, id, p)
    }

    pub fn resolved_in(
        &self,
        f: crate::theme::Family,
        v: crate::theme::Variant,
        id: Id,
        p: Part,
    ) -> crate::theme::Resolved {
        self.theme
            .resolve(f, v, p, self.last.state(id), crate::theme::Surface::Canvas)
    }

    pub const fn last_invalidate(&self) -> Invalidate {
        self.last_invalidate
    }

    pub fn records(&self) -> &[&'static str] {
        &self.services.records
    }

    pub fn clear_records(&mut self) {
        self.services.records.clear();
    }

    pub fn projection_registry(&self) -> Option<&crate::hit::Registry> {
        self.projection.as_ref().map(|output| &output.last.registry)
    }

    pub fn projection_ring(&self) -> Option<&FocusRing> {
        self.projection.as_ref().map(|output| &output.last.ring)
    }

    pub fn projection_cursor(&self) -> Option<Position> {
        self.projection.as_ref().and_then(|output| output.cursor)
    }

    pub fn draw_projection(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        snapshot: &RenderSnapshot,
        paint: impl FnOnce(&mut Ui<'_>, Rect),
    ) -> ProjectedFrame<'_, A> {
        self.cache_snapshot = None;
        self.draw_with_buffer(area, buf, Some(snapshot), |_, ui| paint(ui, area));
        ProjectedFrame {
            runtime: self,
            snapshot: snapshot.last.snapshot,
        }
    }

    pub fn draw_bound_projection<M: ?Sized, F>(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        binding: &RenderModel<'_, M, F>,
    ) -> ProjectedFrame<'_, A>
    where
        F: Fn(&M, &mut Ui<'_>, Rect),
    {
        self.draw_with_buffer(area, buf, Some(&binding.snapshot), |_, ui| {
            (binding.paint)(binding.model, ui, area);
        });
        ProjectedFrame {
            runtime: self,
            snapshot: binding.snapshot.last.snapshot,
        }
    }

    pub fn draw_scene(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        f: impl FnOnce(&mut Ui<'_>, Rect),
    ) -> PaintedFrame<'_, A> {
        self.draw_with_buffer(area, buf, None, |_, ui| f(ui, area));
        PaintedFrame {
            runtime: self,
            committed: false,
        }
    }

    pub fn draw_buffer(&mut self, area: Rect, buf: &mut Buffer) -> PaintedFrame<'_, A> {
        self.refuse_legacy_continuation_draw();
        self.draw_with_buffer(area, buf, None, A::draw);
        PaintedFrame {
            runtime: self,
            committed: false,
        }
    }

    pub fn region_count(&self) -> usize {
        self.last.registry.len()
    }

    pub const fn registry(&self) -> &crate::hit::Registry {
        &self.last.registry
    }

    pub fn advance_clock(&mut self, ms: u64) {
        let _ = self.advance_to(
            self.now()
                .saturating_add(core::time::Duration::from_millis(ms)),
        );
    }

    pub fn repaint_after(&self) -> Option<core::time::Duration> {
        self.services
            .repaint_at
            .map(|at| at.saturating_duration_since(self.now()))
    }

    pub fn capture_owner(&self) -> Option<Id> {
        self.services.capture.get().map(|c| c.owner)
    }

    pub fn set_focus(&mut self, id: Option<Id>) {
        let from = self.focus.current();
        if from != id {
            self.invalidate_prepared();
            self.pending_focus = Some(PendingFocus {
                from,
                to: id,
                via: FocusVia::Programmatic,
                from_layer: from.and_then(|owner| self.last.ring.entry(owner).map(|e| e.layer)),
            });
            self.focus.set(id);
            self.last.snapshot.focus = id;
        }
    }

    pub fn intent_probes(&self) -> usize {
        self.intents.probes()
    }

    pub fn style_cache_stats(&self) -> (u64, u64) {
        self.core.style_cache.stats()
    }

    pub fn open_spec(&self, id: Id) -> Option<crate::layer::LayerSpec> {
        self.services
            .layers
            .layers()
            .iter()
            .find(|l| l.id == id)
            .map(|l| l.spec)
    }

    pub fn layer_area(&self, id: Id) -> Option<Rect> {
        self.frame
            .layers
            .active()
            .iter()
            .find(|l| l.id == id)
            .map(|l| l.area)
    }

    pub fn record_diagnostic(&mut self, d: Diagnostic) {
        self.services.diagnostics.push(d);
    }
}

#[cfg(any(test, feature = "testing"))]
pub mod stub {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::{Position, Rect};

    use super::{App, Runtime, UpdateCause};
    use crate::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
    use crate::focus::Focusability;
    use crate::hit::{Axes, Headroom};
    use crate::id::{Id, Part, PartRef};
    use crate::intent::{Intent, Phase};
    use crate::layer::LayerSpec;
    use crate::response::{Response, StateFlags};
    use crate::theme::Theme;
    use crate::ui::{Cx, Ui};

    #[derive(Clone, Debug)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "a test stub's knobs: one flag per registration shape, set by field                   init from `Control::new`, never a state machine"
    )]
    pub struct Control {
        pub id: Id,
        pub area: Rect,
        pub focus: Focusability,
        pub editor: bool,
        pub captures: bool,
        pub scroll: bool,
        pub decor: bool,
    }

    impl Control {
        pub const fn new(id: Id, area: Rect) -> Self {
            Control {
                id,
                area,
                focus: Focusability::Focusable,
                editor: false,
                captures: false,
                scroll: false,
                decor: false,
            }
        }
    }

    #[derive(Default, Debug)]
    pub struct Stub {
        pub page: Vec<Control>,
        pub layers: Vec<(Id, Vec<Control>)>,
        pub log: Vec<(Id, String)>,
        pub consume_keys: bool,
        pub focus_request: Option<Id>,
        pub open_request: Option<(Id, LayerSpec)>,
        pub close_request: Option<Id>,
        pub resize_request: Option<(Id, crate::layer::LayerSize)>,
        pub updates: usize,
        pub events_started: bool,
        pub chase: Vec<Id>,
        pub esc_hits: usize,
        pub skip_drain: Option<Id>,
    }

    impl Stub {
        pub fn controls(&self) -> impl Iterator<Item = &Control> + '_ {
            self.page
                .iter()
                .chain(self.layers.iter().flat_map(|(_, c)| c.iter()))
        }

        pub fn saw(&self, id: Id, needle: &str) -> bool {
            self.log.iter().any(|(o, s)| *o == id && s.contains(needle))
        }

        pub fn count(&self, id: Id, needle: &str) -> usize {
            self.log
                .iter()
                .filter(|(o, s)| *o == id && s.contains(needle))
                .count()
        }
    }

    impl App for Stub {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.updates = self.updates.saturating_add(1);

            if cx.update_cause() == UpdateCause::Bootstrap {
                return Response::ignored();
            }
            if matches!(cx.update_cause(), UpdateCause::Event | UpdateCause::Tick) {
                self.events_started = true;
            }
            let mut r = Response::ignored();
            let controls: Vec<Control> = self.controls().cloned().collect();
            for c in &controls {
                if self.skip_drain == Some(c.id) {
                    continue;
                }
                for it in cx.intents(c.id) {
                    self.log.push((c.id, format!("{it:?}")));
                    match it {
                        Intent::Key(k) if c.editor && k.is(KeyCode::Esc) => {
                            r |= Response::changed();
                        }
                        Intent::Key(_) if self.consume_keys => r |= Response::consumed(),
                        Intent::Pointer {
                            phase: Phase::Press,
                            part,
                            ..
                        } if c.captures => {
                            assert!(cx.capture(c.id, part));
                            self.log
                                .push((c.id, format!("origin: {:?}", cx.capture_origin())));
                            r |= Response::changed();
                        }
                        Intent::Pointer {
                            phase: Phase::Move, ..
                        } => {}
                        Intent::Pointer { .. } | Intent::Wheel { .. } => r |= Response::changed(),
                        _ => {}
                    }
                }
            }
            let layer_ids: Vec<Id> = self
                .layers
                .iter()
                .map(|(id, _)| *id)
                .filter(|id| self.skip_drain != Some(*id))
                .collect();
            for id in layer_ids {
                for it in cx.intents(id) {
                    self.log.push((id, format!("{it:?}")));
                }
            }
            if !self.events_started {
                return r;
            }
            if let Some(f) = self.focus_request.take() {
                cx.focus(f);
            }
            if let Some(next) = self.chase.pop() {
                cx.focus(next);
            }
            if let Some((id, spec)) = self.open_request.take() {
                cx.open_layer(id, spec);
            }
            if let Some(id) = self.close_request.take() {
                cx.close_layer(id, None);
            }
            if let Some((id, size)) = self.resize_request.take() {
                cx.resize_layer(id, size);
            }
            r
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            for c in &self.page {
                register(ui, c);
            }
            for (layer, controls) in &self.layers {
                ui.layer(*layer, |ui, _area| {
                    for c in controls {
                        register(ui, c);
                    }
                });
            }
        }

        fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            self.esc_hits = self.esc_hits.saturating_add(1);
            Response::consumed()
        }
    }

    fn register(ui: &mut Ui<'_>, c: &Control) {
        if c.decor {
            ui.register_decor(c.id, PartRef::of(Part::CONTAINER), c.area);
            return;
        }
        if c.editor {
            ui.register_editor(c.id, c.area, c.focus, StateFlags::EDITING);
            ui.set_cursor(c.id, Position::new(c.area.x, c.area.y));
        } else {
            ui.register_control(c.id, c.area, c.focus);
        }
        ui.register_part(c.id, PartRef::of(Part::LABEL), c.area);
        if c.scroll {
            ui.register_scroll(c.id, c.area, Axes::V, Headroom::default());
        }
    }

    pub const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 12,
    };

    pub fn runtime(stub: Stub) -> (Runtime<Stub>, Buffer) {
        let mut rt = Runtime::new(stub, Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..16 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                return (rt, buf);
            }
            let _ = rt.settle();
        }
        panic!("initial fixture focus did not settle");
    }

    pub fn key(code: KeyCode) -> Input {
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    pub fn mouse(kind: MouseKind, x: u16, y: u16) -> Input {
        Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        })
    }

    pub fn deliver<A: App>(rt: &mut Runtime<A>, input: Input) -> Response<()> {
        assert!(
            rt.bootstrapped,
            "behavioral fixture must explicitly initialize"
        );
        let area = if rt.screen().is_empty() {
            SCREEN
        } else {
            rt.screen()
        };
        let mut buffer = Buffer::empty(area);
        for _ in 0..16 {
            if rt.needs_settle() {
                let _ = rt.settle();
            }
            if !rt.needs_present() && !rt.needs_settle() {
                return rt
                    .handle(input)
                    .expect("fixture publication must accept input");
            }
            rt.draw_buffer(area, &mut buffer).commit_presented();
        }
        panic!("fixture publication did not settle: {:?}", rt.diagnostics());
    }

    pub fn step(rt: &mut Runtime<Stub>, buf: &mut Buffer, input: Input) -> Response<()> {
        let r = deliver(rt, input);
        rt.draw_buffer(SCREEN, buf).commit_presented();
        r
    }
}

#[cfg(test)]
mod tests {
    use super::stub::{Control, SCREEN, Stub, deliver, key, mouse, runtime, step};
    use super::*;
    use crate::event::MouseKind;
    use crate::focus::Focusability;
    use crate::focus::{ScopeId, ScopeMode};
    use crate::hit::{Axes, Headroom};
    use crate::intent::Intent;
    use crate::layer::{Anchor, LayerSize, LayerSpec, ScreenAlign};
    use crate::ui::FrameRead;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    const A: Id = Id::root("a");
    const B: Id = Id::root("b");
    const CLICK_MODAL: Id = Id::root("click-modal");
    const RETIRED_FOCUS_LAYER: Id = Id::root("retired-focus-layer");
    const RETIRED_FOCUS_CONTROL: Id = Id::root("retired-focus-control");
    const RETIRED_FOCUS_NEXT: Id = Id::root("retired-focus-next");
    const SELECTIVE_LAYER: Id = Id::root("selective-layer");

    #[test]
    fn app_keyboard_policy_default_uses_optional_override_and_off_fallback() {
        use termrock_core::keys::keyboard::KeyboardProtocol;

        assert_eq!(default_app_keyboard_protocol(None), KeyboardProtocol::Off);
        for protocol in [
            KeyboardProtocol::Auto,
            KeyboardProtocol::On,
            KeyboardProtocol::Off,
        ] {
            assert_eq!(default_app_keyboard_protocol(Some(protocol)), protocol);
        }
    }

    #[test]
    fn visible_cursor_retention_respects_top_layer_cursor_owners() {
        let mut frame = FrameState {
            top: crate::layer::LayerId(2),
            ..FrameState::default()
        };
        let retained = crate::ui::RetainedCursorRequest {
            owner: A,
            parent_layer: crate::layer::LayerId::PAGE,
            overlay_layer: frame.top,
            cursor_index: 0,
            mode: crate::ui::CursorRetentionMode::VisibleOwner,
            ambiguous: false,
        };
        frame.cursors.push(crate::cursor::CursorRequest {
            layer: frame.top,
            owner: A,
            pos: Position::new(1, 1),
            inert: false,
            typing_offer: false,
            shape: crate::cursor::CursorShape::Bar,
        });

        // A real editor owns typing in the overlay even when its caret is hidden.
        frame.typing_resolved.owner = Some(B);
        assert_eq!(
            select_cursor_owner(&frame, Some(retained), Some(A), Some(A)),
            None
        );

        // A visible ordinary cursor in the focused overlay has precedence.
        frame.typing_resolved = typing::TypingResolved::default();
        frame.cursors.push(crate::cursor::CursorRequest {
            layer: frame.top,
            owner: B,
            pos: Position::new(2, 1),
            inert: false,
            typing_offer: false,
            shape: crate::cursor::CursorShape::Bar,
        });
        assert_eq!(
            select_cursor_owner(&frame, Some(retained), Some(A), Some(B)),
            Some(B)
        );

        // Preserve strict retention precedence for the pre-existing API.
        let strict = crate::ui::RetainedCursorRequest {
            mode: crate::ui::CursorRetentionMode::FocusedOwner,
            ..retained
        };
        assert_eq!(
            select_cursor_owner(&frame, Some(strict), Some(A), Some(B)),
            Some(A)
        );
    }

    #[test]
    fn real_editable_overlay_typing_suppresses_retained_owner_when_caret_is_absent() {
        use crate::layer::{Anchor, LayerKind, LayerSpec, ScreenAlign};
        use crate::response::StateFlags;
        use crate::ui::UiCore;

        const POPUP: Id = Id::root("cursor-hidden-editor-popup");
        const TERMINAL: Id = Id::root("cursor-hidden-editor-terminal");
        const EDITOR: Id = Id::root("cursor-hidden-editor-input");
        let theme = Theme::junie();
        let mut frame = FrameState::default();
        frame.reset(1, SCREEN);
        frame.top = crate::layer::LayerId(1);
        frame.layers.push(
            POPUP,
            frame.top,
            LayerSpec::popover(POPUP, Anchor::Screen(ScreenAlign::Center)),
            SCREEN,
            SCREEN,
        );
        let mut page = Buffer::empty(SCREEN);
        let mut core = UiCore::default();
        let last = LastFrame::default();
        {
            let mut ui = Ui::new(&mut frame, &mut page, &mut core, &theme, &last);
            ui.register_control(TERMINAL, SCREEN, Focusability::FocusableReadOnly);
            ui.set_bar_cursor(TERMINAL, Position::new(4, 1));
            let _ = ui.layer(POPUP, |ui, _| {
                assert!(ui.retain_visible_owner_cursor(TERMINAL));
                // This is a real editable typing declaration in the actual top
                // layer, but the editor intentionally has no cursor request.
                ui.register_editor(EDITOR, SCREEN, Focusability::Focusable, StateFlags::EDITING);
                ui.publish_typing_target::<u8>(EDITOR, &[], |_| true, false);
            });
        }

        // The editable field is the live fallback typing target in this popup,
        // although no control owns keyboard focus. This is the case where a
        // retained cursor must not leak through an editor with no caret.
        frame.typing_resolved = typing::resolve(&mut frame, None);
        assert_eq!(frame.typing_resolved.owner, Some(EDITOR));
        assert_eq!(frame.typing_resolved.cursor, None);
        assert_eq!(frame.typing_resolved.fallback, Some(EDITOR));
        assert_eq!(frame.cursors.len(), 1, "the editor published no caret");
        assert_eq!(
            select_cursor_owner(&frame, frame.retained_cursor_request, Some(TERMINAL), None,),
            None,
            "typing ownership in the live editable layer suppresses parent fallback"
        );
        assert_eq!(
            cursor::resolve_requests(
                &frame.cursors,
                frame.top,
                Some(EDITOR),
                frame.typing_resolved.owner,
                &mut frame.diagnostics,
                &frame.typing,
            ),
            None,
            "a hidden editor caret cannot reveal the retained terminal caret"
        );
        assert!(
            frame
                .layers
                .active()
                .last()
                .is_some_and(|layer| { layer.spec.kind == LayerKind::Popover && layer.drawn })
        );
    }

    fn present<A: App>(rt: &mut Runtime<A>, token: PreparedFrame) {
        let mut terminal = Terminal::new(TestBackend::new(SCREEN.width, SCREEN.height))
            .expect("test terminal initializes");
        terminal
            .draw(|frame| {
                rt.draw_prepared(token, frame)
                    .expect("prepared frame draws");
            })
            .expect("test backend draws");
        rt.acknowledge_presented(token)
            .expect("backend output is acknowledged");
    }

    #[derive(Debug, Default)]
    struct BootstrapProbe {
        causes: Vec<UpdateCause>,
    }

    const CONTINUED_OWNER: Id = Id::root("continued-owner");
    const CONTINUED_LAYER: Id = Id::root("continued-layer");
    const CONTINUED_TARGET: Id = Id::root("continued-target");
    const HOVER_OWNER: Id = Id::root("hover-owner");
    const FOCUS_A: Id = Id::root("focus-a");
    const FOCUS_B: Id = Id::root("focus-b");
    const MULTI_PARENT: Id = Id::root("multi-parent");
    const MULTI_CHILD: Id = Id::root("multi-child");
    const MULTI_GRANDCHILD: Id = Id::root("multi-grandchild");

    #[derive(Default)]
    struct UpdateTokenProbe {
        tokens: Vec<crate::layer::UpdateToken>,
    }

    impl App for UpdateTokenProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.tokens.push(cx.update_token());
            Response::ignored()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}

        fn on_esc(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.tokens.push(cx.update_token());
            Response::ignored()
        }
    }

    #[test]
    fn update_tokens_are_unique_per_callback_and_runtime() {
        let mut first = Runtime::new(UpdateTokenProbe::default(), Theme::junie());
        let _ = first.initialize();
        let first_boot = first.app().tokens[0];
        let _ = deliver(&mut first, Input::Tick);
        let first_tick = *first.app().tokens.last().expect("tick update token");
        let _ = deliver(&mut first, key(KeyCode::Esc));
        let first_esc = *first.app().tokens.last().expect("Esc callback token");
        assert_ne!(first_boot, first_tick);
        assert_ne!(first_tick, first_esc);

        let mut second = Runtime::new(UpdateTokenProbe::default(), Theme::junie());
        let _ = second.initialize();
        assert_ne!(first_boot, second.app().tokens[0]);
    }

    #[derive(Default)]
    struct MultiCloseProbe {
        parent: Option<crate::layer::LayerOpening>,
        child: Option<crate::layer::LayerOpening>,
        child_outcome: Option<bool>,
        completed: Vec<crate::layer::LayerOpening>,
    }

    impl App for MultiCloseProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx
                .intents(A)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('m')))
            {
                self.parent = cx.open_layer_with_token(
                    MULTI_PARENT,
                    LayerSpec::modal(MULTI_PARENT)
                        .initial_focus(B)
                        .continues_focus(A),
                );
            }
            if cx
                .intents(A)
                .any(|intent| matches!(intent, Intent::FocusOut { to: Some(B) }))
                && cx.focus_is_continued(A, B)
            {
                assert!(cx.acknowledge_focus_continuation(A, B));
            }
            if cx
                .intents(B)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('n')))
            {
                self.child = cx.open_layer_with_token(
                    MULTI_CHILD,
                    LayerSpec::modal(MULTI_CHILD)
                        .initial_focus(MULTI_GRANDCHILD)
                        .continues_focus(B),
                );
            }
            if cx.intents(B).any(|intent| {
                matches!(
                    intent,
                    Intent::FocusOut {
                        to: Some(MULTI_GRANDCHILD)
                    }
                )
            }) && cx.focus_is_continued(B, MULTI_GRANDCHILD)
            {
                assert!(cx.acknowledge_focus_continuation(B, MULTI_GRANDCHILD));
            }
            if cx
                .intents(MULTI_GRANDCHILD)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')))
            {
                cx.close_layer(MULTI_CHILD, None);
                cx.close_layer(MULTI_PARENT, None);
            }
            for opening in [self.parent, self.child].into_iter().flatten() {
                if cx.take_layer_completion(opening).is_some() {
                    self.completed.push(opening);
                }
            }
            if let Some(opening) = self.child
                && let Some(outcome) = cx.focus_continuation_result(opening, B)
            {
                self.child_outcome = Some(outcome.continued());
                let _ = cx.acknowledge_focus_continuation_result(opening, B);
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(A, Rect::new(0, 0, 8, 1), Focusability::Focusable);
            let _ = ui.layer(MULTI_PARENT, |ui, area| {
                ui.register_control(B, Rect::new(area.x, area.y, 8, 1), Focusability::Focusable);
                let _ = ui.layer(MULTI_CHILD, |ui, child_area| {
                    ui.register_control(
                        MULTI_GRANDCHILD,
                        Rect::new(child_area.x, child_area.y, 8, 1),
                        Focusability::Focusable,
                    );
                });
            });
        }
    }

    #[test]
    fn direct_nested_closes_cancel_child_receipt_and_keep_parent_receipt() {
        let mut rt = Runtime::new(MultiCloseProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(A));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("parent opens");
        let token = rt.prepare(SCREEN).expect("parent focus settles");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('n'))).expect("child opens");
        let token = rt.prepare(SCREEN).expect("child focus settles");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("ancestor closes");
        let token = rt.prepare(SCREEN).expect("descendant cancellation settles");
        assert_eq!(rt.app().child_outcome, Some(false));
        assert!(
            !rt.app()
                .completed
                .contains(&rt.app().child.expect("child opening"))
        );
        assert!(rt.services.focus_continuation_result_acks.is_empty());
        present(&mut rt, token);
    }

    struct OrdinaryLayerProbe;

    impl App for OrdinaryLayerProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx
                .intents(A)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('m')))
            {
                cx.open_layer(CLICK_MODAL, LayerSpec::modal(CLICK_MODAL).initial_focus(B));
            }
            if cx
                .intents(B)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')))
            {
                cx.close_layer(CLICK_MODAL, None);
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(A, Rect::new(0, 0, 8, 1), Focusability::Focusable);
            let _ = ui.layer(CLICK_MODAL, |ui, area| {
                ui.register_control(B, area, Focusability::Focusable);
            });
        }
    }

    #[derive(Default)]
    struct SelectiveLayerReceiptProbe {
        opening: Option<crate::layer::LayerOpening>,
        replaced: Option<crate::layer::LayerOpening>,
        receipts: Vec<crate::layer::LayerEventReceipt>,
        layer_keys: Vec<KeyCode>,
        immediate_reopen: bool,
    }

    impl SelectiveLayerReceiptProbe {
        fn open(&self, cx: &mut Cx<'_>) -> Option<crate::layer::LayerOpening> {
            cx.open_layer_with_token(
                SELECTIVE_LAYER,
                LayerSpec::popover(SELECTIVE_LAYER, Anchor::Screen(ScreenAlign::Center))
                    .size(LayerSize::Fixed(10, 3))
                    .initial_focus(SELECTIVE_LAYER),
            )
        }
    }

    impl App for SelectiveLayerReceiptProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            while let Some(receipt) = cx.take_layer_intent_receipt(SELECTIVE_LAYER) {
                self.receipts.push(receipt);
            }
            if cx
                .intents(A)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('m')))
            {
                self.opening = self.open(cx);
            }
            let close = cx
                .intents(SELECTIVE_LAYER)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')));
            if close {
                cx.close_layer(SELECTIVE_LAYER, None);
                if self.immediate_reopen {
                    self.replaced = self.open(cx);
                }
            } else if cx
                .intents(A)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('r')))
            {
                self.replaced = self.open(cx);
            } else {
                for intent in cx.intents(SELECTIVE_LAYER) {
                    if let Intent::Key(key) = intent {
                        self.layer_keys.push(key.code);
                    }
                }
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(A, Rect::new(0, 0, 8, 1), Focusability::Focusable);
            let _ = ui.layer(SELECTIVE_LAYER, |ui, area| {
                ui.register_control(SELECTIVE_LAYER, area, Focusability::Focusable);
            });
        }
    }

    #[test]
    fn selective_layer_receipts_preserve_mixed_intents_and_reject_old_generation() {
        let mut rt = Runtime::new(SelectiveLayerReceiptProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(A));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);

        let _ = rt.handle(key(KeyCode::Char('m'))).expect("layer opens");
        let token = rt.prepare(SCREEN).expect("layer opens");
        present(&mut rt, token);
        let first = rt.app().opening.expect("first opening");

        let _ = rt.handle(key(KeyCode::Char('x'))).expect("layer closes");
        let token = rt.prepare(SCREEN).expect("layer close settles");
        present(&mut rt, token);

        let _ = rt.handle(key(KeyCode::Char('r'))).expect("layer reopens");
        let token = rt.prepare(SCREEN).expect("replacement settles");
        present(&mut rt, token);
        let replacement = rt.app().replaced.expect("replacement opening");
        assert_ne!(first, replacement);

        let receipts = &rt.app().receipts;
        assert!(receipts.iter().any(|receipt| {
            receipt.opening() == first
                && receipt.event() == LayerEvent::Dismissed(DismissReason::Programmatic)
                && receipt.update().is_some()
        }));
        assert!(receipts.iter().any(
            |receipt| receipt.opening() == replacement && receipt.event() == LayerEvent::Opened
        ));
        assert_eq!(
            rt.app()
                .receipts
                .iter()
                .map(|receipt| (receipt.opening(), receipt.event()))
                .collect::<Vec<_>>(),
            vec![
                (first, LayerEvent::Opened),
                (first, LayerEvent::Dismissed(DismissReason::Programmatic),),
                (replacement, LayerEvent::Opened),
            ],
            "opening identities and delayed close receipts retain chronological order"
        );
    }

    #[test]
    fn same_update_layer_reopen_keeps_new_generation_live() {
        let app = SelectiveLayerReceiptProbe {
            immediate_reopen: true,
            ..SelectiveLayerReceiptProbe::default()
        };
        let mut rt = Runtime::new(app, Theme::junie());
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(A));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);

        let _ = rt.handle(key(KeyCode::Char('m'))).expect("layer opens");
        let token = rt.prepare(SCREEN).expect("layer opens");
        present(&mut rt, token);
        let first = rt.app().opening.expect("first opening");

        let _ = rt
            .handle(key(KeyCode::Char('x')))
            .expect("layer closes and reopens");
        let token = rt.prepare(SCREEN).expect("same-update replacement settles");
        present(&mut rt, token);
        let replacement = rt.app().replaced.expect("replacement opening");
        assert_ne!(first, replacement);
        assert!(
            rt.is_open(SELECTIVE_LAYER),
            "same-ID replacement remains open"
        );
        assert!(
            !rt.app().receipts.iter().any(|receipt| {
                receipt.opening() == first && matches!(receipt.event(), LayerEvent::Dismissed(_))
            }),
            "coalesced old close has no stale dismissal receipt"
        );
        assert_eq!(
            rt.focus(),
            Some(SELECTIVE_LAYER),
            "new layer keeps its focus target"
        );
        assert!(!rt.diagnostics().iter().any(|diagnostic| {
            matches!(diagnostic, Diagnostic::UndeliveredIntent { owner } if *owner == SELECTIVE_LAYER)
        }));
    }

    #[test]
    fn ordinary_layer_close_does_not_enqueue_completion_or_extra_update() {
        let mut rt = Runtime::new(OrdinaryLayerProbe, Theme::junie());
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(A));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("modal opens");
        let token = rt.prepare(SCREEN).expect("modal opens");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("modal closes");
        let token = rt.prepare(SCREEN).expect("modal closes");
        assert!(rt.services.completions.is_empty());
        assert!(!rt.completion_dispatch_pending);
        assert!(!rt.needs_settle());
        present(&mut rt, token);
    }

    #[derive(Debug, Default)]
    struct ContinuationProbe {
        opening: Option<crate::layer::LayerOpening>,
        continuation_opening: Option<crate::layer::LayerOpening>,
        continued: bool,
        acknowledgements: usize,
        acknowledge_owner: bool,
        observe_outcome: bool,
        owner_hidden: bool,
        owner_disabled: bool,
        consume_completion: bool,
        menu_before_outcome_ack: bool,
        reopen_after_close: bool,
        replacement: Option<crate::layer::LayerOpening>,
        duplicate_result_ack: bool,
        duplicate_ack_succeeded: bool,
        outcome: Option<(Option<Id>, bool)>,
        outcome_acknowledgements: usize,
        completion: Option<(Option<Id>, LayerEvent, bool)>,
        completion_before_outcome_ack: bool,
        redirect_after_outcome_ack: Option<Id>,
    }

    impl App for ContinuationProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let owner_intents: Vec<_> = cx.intents(CONTINUED_OWNER).collect();
            for intent in &owner_intents {
                if let Intent::FocusOut { to: Some(target) } = intent {
                    self.continuation_opening =
                        cx.focus_continuation_opening(CONTINUED_OWNER, *target);
                }
                if matches!(intent, Intent::FocusOut { to: Some(target) } if *target == CONTINUED_TARGET)
                {
                    self.continued = cx.focus_is_continued(CONTINUED_OWNER, CONTINUED_TARGET);
                    if self.continued
                        && self.acknowledge_owner
                        && cx.acknowledge_focus_continuation(CONTINUED_OWNER, CONTINUED_TARGET)
                    {
                        self.acknowledgements += 1;
                    }
                }
                if matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('m')) {
                    self.opening = cx.open_layer_with_token(
                        CONTINUED_LAYER,
                        LayerSpec::popover(CONTINUED_LAYER, Anchor::Screen(ScreenAlign::Center))
                            .size(LayerSize::Fixed(10, 3))
                            .initial_focus(CONTINUED_TARGET)
                            .continues_focus(CONTINUED_OWNER),
                    );
                }
            }
            for intent in cx.intents(CONTINUED_TARGET) {
                if matches!(intent, Intent::Key(key) if key.code == KeyCode::Char('x')) {
                    cx.close_layer(CONTINUED_LAYER, None);
                    if self.reopen_after_close {
                        self.replacement = cx.open_layer_with_token(
                            CONTINUED_LAYER,
                            LayerSpec::popover(
                                CONTINUED_LAYER,
                                Anchor::Screen(ScreenAlign::Center),
                            )
                            .size(LayerSize::Fixed(10, 3))
                            .initial_focus(CONTINUED_TARGET)
                            .continues_focus(CONTINUED_OWNER),
                        );
                    }
                }
            }
            if self.menu_before_outcome_ack {
                self.take_completion(cx);
            }
            if self.observe_outcome
                && let Some(opening) = self.continuation_opening
                && let Some(outcome) = cx.focus_continuation_result(opening, CONTINUED_OWNER)
            {
                self.outcome = Some((outcome.restored_focus(), outcome.continued()));
                if cx.acknowledge_focus_continuation_result(opening, CONTINUED_OWNER) {
                    self.outcome_acknowledgements += 1;
                    if let Some(target) = self.redirect_after_outcome_ack {
                        cx.focus(target);
                    }
                }
                if self.duplicate_result_ack {
                    self.duplicate_ack_succeeded =
                        cx.acknowledge_focus_continuation_result(opening, CONTINUED_OWNER);
                }
            }
            if !self.menu_before_outcome_ack {
                self.take_completion(cx);
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(B, Rect::new(20, 0, 10, 1), Focusability::Focusable);
            if !self.owner_hidden {
                ui.register_control(
                    CONTINUED_OWNER,
                    Rect::new(0, 0, 10, 1),
                    if self.owner_disabled {
                        Focusability::Disabled
                    } else {
                        Focusability::Focusable
                    },
                );
            }
            let _ = ui.layer(CONTINUED_LAYER, |ui, area| {
                ui.register_control(CONTINUED_TARGET, area, Focusability::Focusable);
            });
        }
    }

    impl ContinuationProbe {
        fn take_completion(&mut self, cx: &mut Cx<'_>) {
            if let Some(opening) = self.opening
                && self.consume_completion
                && let Some(completion) = cx.take_layer_completion(opening)
            {
                self.completion_before_outcome_ack = self.outcome_acknowledgements == 0;
                self.completion = Some((
                    completion.restored_focus(),
                    completion.event(),
                    completion.continuation_acknowledged(),
                ));
            }
        }
    }

    struct HoverGeometryProbe;

    impl App for HoverGeometryProbe {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let x = if ui.state(HOVER_OWNER).contains(StateFlags::HOVERED) {
                10
            } else {
                0
            };
            ui.register_control(HOVER_OWNER, Rect::new(x, 0, 10, 1), Focusability::Focusable);
        }
    }

    struct FocusCycleProbe;

    impl App for FocusCycleProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(FOCUS_A);
            }
            if cx
                .intents(FOCUS_A)
                .any(|intent| matches!(intent, Intent::FocusIn { .. }))
            {
                cx.focus(FOCUS_B);
            }
            if cx
                .intents(FOCUS_B)
                .any(|intent| matches!(intent, Intent::FocusIn { .. }))
            {
                cx.focus(FOCUS_A);
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(FOCUS_A, Rect::new(0, 0, 1, 1), Focusability::Focusable);
            ui.register_control(FOCUS_B, Rect::new(1, 0, 1, 1), Focusability::Focusable);
        }
    }

    impl App for BootstrapProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.causes.push(cx.update_cause());
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.request_repaint_after(core::time::Duration::from_millis(33));
            }
            Response::ignored()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    #[test]
    fn bootstrap_runs_once_before_first_draw_without_a_tick() {
        let mut rt = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);

        assert_eq!(rt.clock_ms(), 0);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
        assert_eq!(rt.clock_ms(), 0);
        assert_eq!(rt.next_deadline(), Some(Moment::from_millis(33)));

        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
        assert_eq!(rt.clock_ms(), 0);
    }

    #[test]
    fn prepared_frame_publishes_geometry_but_blocks_input_until_acknowledged() {
        let mut rt = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let token = rt.prepare(SCREEN).expect("frame prepares");
        assert_eq!(token.area(), SCREEN);
        assert!(rt.needs_present());
        assert!(rt.handle(key(KeyCode::Enter)).is_err());

        present(&mut rt, token);
        assert!(!rt.needs_present());
        assert!(rt.handle(key(KeyCode::Enter)).is_ok());
    }

    #[test]
    fn unprojected_acknowledgement_preserves_the_prepared_frame() {
        let mut rt = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let token = rt.prepare(SCREEN).expect("frame prepares");

        assert_eq!(
            rt.acknowledge_presented(token),
            Err(PreparedFrameError::Stale)
        );
        assert!(rt.needs_present());

        present(&mut rt, token);
        assert!(!rt.needs_present());
    }

    #[test]
    fn prepared_frame_tokens_are_runtime_and_generation_scoped() {
        let mut first = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let mut second = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let stale = first.prepare(SCREEN).expect("first frame prepares");
        let _ = first
            .prepare(SCREEN)
            .expect("reprepare advances generation");
        let current = second.prepare(SCREEN).expect("second runtime prepares");
        assert!(first.acknowledge_presented(stale).is_err());
        assert!(second.acknowledge_presented(stale).is_err());
        present(&mut second, current);
    }

    #[test]
    fn state_changes_invalidate_prepared_frames() {
        let mut rt = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let token = rt.prepare(SCREEN).expect("frame prepares");
        rt.set_theme(Theme::junie());
        assert!(rt.acknowledge_presented(token).is_err());

        let token = rt.prepare(SCREEN).expect("frame prepares again");
        rt.advance_to(Moment::from_millis(1))
            .expect("clock advances");
        assert!(rt.acknowledge_presented(token).is_err());

        let token = rt
            .prepare(SCREEN)
            .expect("frame prepares after clock change");
        let _ = rt.app_mut();
        assert!(rt.acknowledge_presented(token).is_err());
    }

    #[test]
    fn prepare_reconciles_hover_geometry_before_output() {
        let mut rt = Runtime::new(HoverGeometryProbe, Theme::junie());
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        let _ = rt
            .handle(mouse(MouseKind::Move, 1, 0))
            .expect("pointer moves");
        let token = rt.prepare(SCREEN).expect("hover geometry settles");
        assert_eq!(rt.area_of(HOVER_OWNER).map(|area| area.x), Some(0));
        assert!(!rt.state_of(HOVER_OWNER).contains(StateFlags::HOVERED));
        present(&mut rt, token);
    }

    #[test]
    fn prepare_bounds_repeated_focus_redirection() {
        let mut rt = Runtime::new(FocusCycleProbe, Theme::junie());
        let _ = rt.initialize();
        assert_eq!(rt.prepare(SCREEN), Err(PreparedFrameError::DidNotSettle));
    }

    #[test]
    fn focus_continuation_and_close_receipt_settle_before_prepared_output() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                observe_outcome: true,
                consume_completion: true,
                menu_before_outcome_ack: true,
                duplicate_result_ack: true,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);

        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu and FocusOut settle");
        assert_eq!(rt.focus(), Some(CONTINUED_TARGET));
        assert!(rt.app().continued, "probe: {:?}", rt.app());
        assert_eq!(rt.app().acknowledgements, 1);
        assert!(rt.app().continuation_opening.is_some());
        present(&mut rt, token);

        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        assert!(rt.app().completion.is_none());
        let token = rt.prepare(SCREEN).expect("completion runs before output");
        assert_eq!(
            rt.app().completion,
            Some((
                Some(CONTINUED_OWNER),
                LayerEvent::Dismissed(DismissReason::Programmatic),
                true
            ))
        );
        assert_eq!(rt.app().outcome, Some((Some(CONTINUED_OWNER), true)));
        assert_eq!(rt.app().outcome_acknowledgements, 1);
        assert!(!rt.app().completion_before_outcome_ack);
        assert!(!rt.app().duplicate_ack_succeeded);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut buffer = Buffer::empty(SCREEN);
                let _frame = rt.draw_buffer(SCREEN, &mut buffer);
            }))
            .is_err()
        );
        present(&mut rt, token);
        assert!(rt.services.focus_continuation_outcomes.is_empty());
        assert!(rt.services.focus_continuation_result_acks.is_empty());
    }

    #[test]
    fn focus_redirect_during_result_ack_cancels_same_update_completion() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                observe_outcome: true,
                consume_completion: true,
                redirect_after_outcome_ack: Some(B),
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let token = rt.prepare(SCREEN).expect("redirect settles before receipt");
        assert_eq!(rt.focus(), Some(B));
        assert_eq!(
            rt.app().completion.map(|(_, _, continued)| continued),
            Some(false)
        );
        assert_eq!(rt.app().outcome_acknowledgements, 1);
        present(&mut rt, token);
    }

    #[test]
    fn legacy_draw_refuses_continuation_receipts_before_projection() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                consume_completion: false,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let _token = rt.prepare(SCREEN).expect("completion is prepared");
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut buffer = Buffer::empty(SCREEN);
                let _frame = rt.draw_buffer(SCREEN, &mut buffer);
            }))
            .is_err()
        );
        assert!(rt.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::FocusContinuationUnacknowledged { owner }
                if *owner == CONTINUED_OWNER
        )));
    }

    #[test]
    fn removed_continuation_owner_receives_cancellation_not_success() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                observe_outcome: true,
                consume_completion: true,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        rt.app_mut().owner_hidden = true;
        let token = rt.prepare(SCREEN).expect("owner removal publishes");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let token = rt.prepare(SCREEN).expect("removed owner settles");
        assert_eq!(rt.app().outcome, Some((rt.focus(), false)));
        assert_eq!(rt.app().completion.map(|(_, _, ack)| ack), Some(false));
        present(&mut rt, token);
    }

    #[test]
    fn disabled_continuation_owner_receives_cancellation_not_success() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                observe_outcome: true,
                consume_completion: true,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        rt.app_mut().owner_disabled = true;
        let token = rt.prepare(SCREEN).expect("disabled owner publishes");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let token = rt.prepare(SCREEN).expect("disabled owner settles");
        assert_eq!(rt.app().outcome, Some((rt.focus(), false)));
        assert_eq!(rt.app().completion.map(|(_, _, ack)| ack), Some(false));
        present(&mut rt, token);
    }

    #[test]
    fn same_id_reopen_cannot_reuse_closed_opening_success() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                observe_outcome: true,
                consume_completion: true,
                reopen_after_close: true,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        let old_opening = rt.app().opening.expect("old opening token");
        let _ = rt
            .handle(key(KeyCode::Char('x')))
            .expect("menu closes and reopens");
        let token = rt.prepare(SCREEN).expect("replacement opening settles");
        assert_ne!(rt.app().replacement, Some(old_opening));
        assert_eq!(rt.app().completion.map(|(_, _, ack)| ack), Some(false));
        assert_eq!(
            rt.app().outcome.map(|(_, continued)| continued),
            Some(false)
        );
        present(&mut rt, token);
    }

    #[test]
    fn continuation_outcome_survives_owner_delay_until_exact_ack() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                observe_outcome: false,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let token = rt.prepare(SCREEN).expect("outcome publishes");
        present(&mut rt, token);
        assert_eq!(rt.app().outcome_acknowledgements, 0);
        assert_eq!(rt.services.focus_continuation_outcomes.len(), 1);

        rt.app_mut().observe_outcome = true;
        let token = rt.prepare(SCREEN).expect("owner becomes observable");
        present(&mut rt, token);
        let _ = rt.handle(Input::Tick).expect("delayed owner updates");
        let token = rt.prepare(SCREEN).expect("delayed owner frame prepares");
        assert_eq!(rt.app().outcome, Some((Some(CONTINUED_OWNER), true)));
        assert_eq!(rt.app().outcome_acknowledgements, 1);
        present(&mut rt, token);
        assert!(rt.services.focus_continuation_outcomes.is_empty());
    }

    #[test]
    fn unacknowledged_continuation_receipts_expire_with_diagnostic() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                acknowledge_owner: true,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt.prepare(SCREEN).expect("menu opens");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let token = rt.prepare(SCREEN).expect("completion is queued");
        present(&mut rt, token);
        assert_eq!(rt.services.completions.len(), 1);
        assert_eq!(rt.services.focus_continuation_outcomes.len(), 1);

        for _ in 0..31 {
            let token = rt.prepare(SCREEN).expect("idle frame prepares");
            present(&mut rt, token);
        }
        assert!(rt.services.completions.is_empty());
        assert!(rt.services.focus_continuation_outcomes.is_empty());
        assert!(rt.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            Diagnostic::FocusContinuationUnacknowledged {
                owner: CONTINUED_OWNER
            }
        )));
    }

    #[test]
    fn missing_continuation_ack_cancels_receipt_without_blocking_prepare() {
        let mut rt = Runtime::new(
            ContinuationProbe {
                consume_completion: true,
                ..ContinuationProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let token = rt.prepare(SCREEN).expect("initial frame prepares");
        present(&mut rt, token);
        rt.set_focus(Some(CONTINUED_OWNER));
        let token = rt.prepare(SCREEN).expect("owner focus prepares");
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('m'))).expect("menu opens");
        let token = rt
            .prepare(SCREEN)
            .expect("unacknowledged continuation settles");
        assert!(rt.app().continued);
        assert_eq!(rt.app().acknowledgements, 0);
        present(&mut rt, token);
        let _ = rt.handle(key(KeyCode::Char('x'))).expect("menu closes");
        let token = rt.prepare(SCREEN).expect("missing ack does not deadlock");
        assert_eq!(
            rt.app().completion,
            Some((
                Some(CONTINUED_OWNER),
                LayerEvent::Dismissed(DismissReason::Programmatic),
                false
            ))
        );
        present(&mut rt, token);
    }

    #[derive(Default)]
    struct TickCauseProbe {
        causes: Vec<UpdateCause>,
    }

    impl App for TickCauseProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.causes.push(cx.update_cause());
            if cx.update_cause() == UpdateCause::Tick {
                cx.focus(B);
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(A, Rect::new(0, 0, 8, 1), Focusability::Focusable);
            ui.register_control(B, Rect::new(0, 2, 8, 1), Focusability::Focusable);
        }
    }

    #[test]
    fn tick_cause_is_delivered_once_when_focus_settles() {
        let mut rt = Runtime::new(TickCauseProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = rt.settle();
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        rt.app_mut().causes.clear();

        let _ = deliver(&mut rt, Input::Tick);
        assert_eq!(
            rt.app().causes,
            vec![UpdateCause::Tick, UpdateCause::Settle]
        );
        assert_eq!(rt.focus(), Some(B));
    }

    #[derive(Default)]
    struct DeadlineProbe {
        requests: Vec<core::time::Duration>,
        updates: usize,
    }

    impl App for DeadlineProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            let request = self.requests.get(self.updates).copied();
            self.updates = self.updates.saturating_add(1);
            if let Some(duration) = request {
                cx.request_repaint_after(duration);
            }
            Response::ignored()
        }

        fn draw(&self, _ui: &mut Ui<'_>) {}
    }

    #[test]
    fn repaint_deadline_survives_unrelated_input_and_keeps_the_earliest() {
        let mut rt = Runtime::new(
            DeadlineProbe {
                requests: vec![
                    core::time::Duration::from_millis(40),
                    core::time::Duration::from_millis(100),
                    core::time::Duration::from_millis(10),
                ],
                updates: 0,
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.next_deadline(), Some(Moment::from_millis(40)));

        let _ = deliver(
            &mut rt,
            Input::Key(Key {
                code: KeyCode::Char('x'),
                mods: KeyModifiers::NONE,
            }),
        );
        assert_eq!(rt.next_deadline(), Some(Moment::from_millis(40)));

        let _ = deliver(
            &mut rt,
            Input::Mouse(Mouse {
                kind: MouseKind::Move,
                pos: Position::new(1, 1),
                mods: KeyModifiers::NONE,
            }),
        );
        assert_eq!(rt.next_deadline(), Some(Moment::from_millis(10)));

        rt.advance_to(Moment::from_millis(10)).unwrap();
        let _ = rt.settle();
        assert_eq!(rt.next_deadline(), None);
    }

    #[test]
    fn headless_tick_uses_the_same_update_cause_without_wall_clock() {
        let mut rt = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        rt.app_mut().causes.clear();

        let before = rt.clock_ms();
        let _ = deliver(&mut rt, Input::Tick);
        assert_eq!(rt.app().causes, vec![UpdateCause::Tick]);
        assert_eq!(rt.clock_ms(), before);
    }

    const REFERENCE_LAYER: Id = Id::root("runtime.reference-layer");
    const REFERENCE_PAGE: Id = Id::root("runtime.reference-page");
    const REFERENCE_LAYER_CONTROL: Id = Id::root("runtime.reference-layer-control");

    #[derive(Default)]
    struct ReferenceLayerApp {
        open: bool,
        draw_live: bool,
    }

    impl App for ReferenceLayerApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            for owner in [REFERENCE_PAGE, REFERENCE_LAYER, REFERENCE_LAYER_CONTROL] {
                for intent in cx.intents(owner) {
                    let _ = intent;
                }
            }
            if matches!(cx.update_cause(), UpdateCause::Event | UpdateCause::Tick)
                && self.open
                && !cx.is_open(REFERENCE_LAYER)
            {
                cx.open_layer(
                    REFERENCE_LAYER,
                    LayerSpec::modal(REFERENCE_LAYER).inert_below(false),
                );
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let style = ui.surface_style();
            ui.paint_str(SCREEN, "PAGE", style);
            ui.register_control(REFERENCE_PAGE, SCREEN, Focusability::Focusable);
            if self.draw_live {
                let _ = ui.layer(REFERENCE_LAYER, |ui, area| {
                    let style = ui.surface_style();
                    ui.paint_str(area, "LAYER", style);
                    ui.register_control(REFERENCE_LAYER_CONTROL, area, Focusability::Focusable);
                });
            } else {
                ui.reference(None, |ui| {
                    let _ = ui.layer(REFERENCE_LAYER, |ui, area| {
                        let style = ui.surface_style();
                        ui.paint_str(area, "FORBIDDEN", style);
                        ui.register_control(REFERENCE_LAYER_CONTROL, area, Focusability::Focusable);
                    });
                });
            }
        }
    }

    #[test]
    fn a_layer_attempted_only_in_reference_has_no_backdrop_or_focus_effect() {
        let mut plain = Runtime::new(ReferenceLayerApp::default(), Theme::junie());
        let mut expected = Buffer::empty(SCREEN);
        plain.draw_buffer(SCREEN, &mut expected).commit_presented();

        let mut runtime = Runtime::new(
            ReferenceLayerApp {
                open: true,
                draw_live: false,
            },
            Theme::junie(),
        );
        let _ = runtime.initialize();
        let _ = deliver(&mut runtime, Input::Tick);
        let mut actual = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut actual).commit_presented();
        assert_eq!(actual, expected, "an undrawn layer left a backdrop");
        assert_eq!(runtime.focus(), Some(REFERENCE_PAGE));
        assert!(runtime.diagnostics().is_empty());

        runtime.app_mut().draw_live = true;
        actual.reset();
        runtime.draw_buffer(SCREEN, &mut actual).commit_presented();
        assert_ne!(actual, expected, "a later legitimate layer draw was lost");
        assert_eq!(runtime.focus(), Some(REFERENCE_LAYER_CONTROL));
        assert!(runtime.diagnostics().is_empty());
    }

    #[test]
    fn a_layer_owners_dismissal_is_diagnosed_when_the_owner_does_not_drain_it() {
        let dlg = Id::root("dlg");
        let mut s = Stub {
            page: vec![Control::new(A, Rect::new(0, 0, 10, 1))],
            ..Stub::default()
        };
        s.layers.push((
            dlg,
            vec![Control {
                decor: true,
                ..Control::new(dlg, Rect::new(5, 5, 10, 3))
            }],
        ));
        s.skip_drain = Some(dlg);
        let (mut rt, mut buf) = runtime(s);
        rt.app_mut().open_request = Some((dlg, LayerSpec::modal(dlg)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.is_open(dlg));
        assert!(
            !rt.last.registry.delivers_to(dlg),
            "the owner must register decoration only, as a Dialog does"
        );
        let _ = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(!rt.is_open(dlg));
        let undelivered: Vec<Id> = rt
            .diagnostics()
            .iter()
            .filter_map(|d| match d {
                Diagnostic::UndeliveredIntent { owner } => Some(*owner),
                _ => None,
            })
            .collect();

        assert!(!undelivered.is_empty(), "the dismissal was lost in silence");
        assert!(
            undelivered.iter().all(|o| *o == dlg),
            "{:?}",
            rt.diagnostics()
        );
    }

    #[derive(Default)]
    struct RetiredLayerFocusProbe {
        open_on_enter: bool,
        move_focus_on_key: bool,
        close_on_key: bool,
        nonrestoring_modal: bool,
        page_focus_outs: usize,
        layer_focus_outs: usize,
        layer_focus_ins: usize,
    }

    impl App for RetiredLayerFocusProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                return Response::ignored();
            }

            for intent in cx.intents(A) {
                if matches!(intent, Intent::FocusOut { .. }) {
                    self.page_focus_outs += 1;
                }
                if matches!(intent, Intent::Key(key) if key.code == KeyCode::Enter)
                    && self.open_on_enter
                {
                    self.open_on_enter = false;
                    cx.open_layer(
                        RETIRED_FOCUS_LAYER,
                        if self.nonrestoring_modal {
                            LayerSpec::modal(RETIRED_FOCUS_LAYER)
                                .initial_focus(RETIRED_FOCUS_CONTROL)
                                .restore_focus(false)
                        } else {
                            LayerSpec::popover(
                                RETIRED_FOCUS_LAYER,
                                Anchor::Point(Position::new(20, 8)),
                            )
                            .size(LayerSize::Fixed(8, 3))
                            .initial_focus(RETIRED_FOCUS_CONTROL)
                        },
                    );
                }
            }

            for _ in cx.intents(RETIRED_FOCUS_LAYER) {}

            if cx.is_open(RETIRED_FOCUS_LAYER) {
                for intent in cx.intents(RETIRED_FOCUS_CONTROL) {
                    match intent {
                        Intent::FocusIn { .. } => self.layer_focus_ins += 1,
                        Intent::FocusOut { .. } => self.layer_focus_outs += 1,
                        Intent::Key(_) if self.move_focus_on_key => {
                            self.move_focus_on_key = false;
                            cx.focus(RETIRED_FOCUS_NEXT);
                        }
                        Intent::Key(_) if self.close_on_key => {
                            self.close_on_key = false;
                            cx.close_layer(RETIRED_FOCUS_LAYER, None);
                        }
                        _ => {}
                    }
                }
                for _ in cx.intents(RETIRED_FOCUS_NEXT) {}
            }

            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Focusable);
            let _ = ui.layer(RETIRED_FOCUS_LAYER, |ui, area| {
                ui.register_control(
                    RETIRED_FOCUS_CONTROL,
                    Rect::new(area.x, area.y, 4, 1),
                    Focusability::Focusable,
                );
                ui.register_control(
                    RETIRED_FOCUS_NEXT,
                    Rect::new(area.x, area.y.saturating_add(1), 4, 1),
                    Focusability::Focusable,
                );
            });
        }
    }

    fn step_retired_layer_probe(
        rt: &mut Runtime<RetiredLayerFocusProbe>,
        buf: &mut Buffer,
        input: Input,
    ) -> Response<()> {
        let response = deliver(rt, input);
        rt.draw_buffer(SCREEN, buf).commit_presented();
        response
    }

    #[test]
    fn focus_out_is_delivered_once_while_live_and_skipped_after_layer_close() {
        let mut rt = Runtime::new(RetiredLayerFocusProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        assert_eq!(rt.focus(), Some(A));

        rt.app_mut().open_on_enter = true;
        let _ = step_retired_layer_probe(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.is_open(RETIRED_FOCUS_LAYER), "popover opens on Enter");
        assert!(
            rt.last.ring.entry(RETIRED_FOCUS_CONTROL).is_some(),
            "focus {:?}, probe {:?}, entries {:?}, diagnostics {:?}",
            rt.focus(),
            (rt.app().open_on_enter, rt.app().move_focus_on_key),
            rt.last.ring.entries(),
            rt.diagnostics()
        );
        let first_layer = rt
            .last
            .ring
            .entry(RETIRED_FOCUS_CONTROL)
            .expect("popover control is registered")
            .layer;
        assert_eq!(rt.app().page_focus_outs, 1, "page blur remains delivered");
        assert_eq!(rt.focus(), Some(RETIRED_FOCUS_CONTROL));

        rt.app_mut().move_focus_on_key = true;
        let _ = step_retired_layer_probe(&mut rt, &mut buf, key(KeyCode::Char('m')));
        assert_eq!(
            rt.app().layer_focus_outs,
            1,
            "live layer blur is delivered once"
        );
        assert_eq!(rt.focus(), Some(RETIRED_FOCUS_NEXT));

        let _ = step_retired_layer_probe(&mut rt, &mut buf, mouse(MouseKind::Down, 2, 0));
        assert!(!rt.is_open(RETIRED_FOCUS_LAYER));
        assert_eq!(rt.focus(), Some(A));
        assert_eq!(
            rt.app().layer_focus_outs,
            1,
            "closed layer gets no stale blur"
        );
        assert!(
            !rt.diagnostics().iter().any(
                |diagnostic| matches!(diagnostic, Diagnostic::UndeliveredIntent { owner } if *owner == RETIRED_FOCUS_CONTROL || *owner == RETIRED_FOCUS_NEXT)
            ),
            "closed-layer focus-out must not become an undelivered intent: {:?}",
            rt.diagnostics()
        );

        rt.app_mut().open_on_enter = true;
        let _ = step_retired_layer_probe(&mut rt, &mut buf, key(KeyCode::Enter));
        let reopened_layer = rt
            .last
            .ring
            .entry(RETIRED_FOCUS_CONTROL)
            .expect("reopened popover control is registered")
            .layer;
        assert_ne!(
            first_layer, reopened_layer,
            "each opening gets a new LayerId"
        );
        assert_eq!(rt.focus(), Some(RETIRED_FOCUS_CONTROL));
        assert_eq!(
            rt.app().page_focus_outs,
            2,
            "page blur remains delivered on reopen"
        );
        assert_eq!(
            rt.app().layer_focus_ins,
            2,
            "reopened control receives focus-in"
        );
    }

    #[derive(Default)]
    struct PressCountProbe {
        counts: Vec<(Phase, u8)>,
        phases: Vec<Phase>,
        capture_on_press: bool,
        open_modal_on_click: bool,
    }

    impl App for PressCountProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            for owner in [A, B] {
                for intent in cx.intents(owner) {
                    if let Intent::Pointer { phase, part, .. } = intent {
                        self.phases.push(phase);
                        if matches!(
                            phase,
                            Phase::Press | Phase::Release | Phase::Click | Phase::DoubleClick
                        ) && let Some(count) = cx.pointer_click_count(owner, part)
                        {
                            self.counts.push((phase, count));
                        }
                        if phase == Phase::Press && self.capture_on_press {
                            assert!(cx.capture(owner, part));
                        }
                        if phase == Phase::Click && self.open_modal_on_click {
                            cx.open_layer(CLICK_MODAL, LayerSpec::modal(CLICK_MODAL));
                        }
                    }
                }
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let area = Rect::new(0, 0, 8, 2);
            ui.register_control(A, area, Focusability::Focusable);
            ui.register_part(A, PartRef::of(Part::LABEL), area);
            let other = Rect::new(10, 0, 8, 2);
            ui.register_control(B, other, Focusability::Focusable);
            ui.register_part(B, PartRef::of(Part::LABEL), other);
            ui.layer(CLICK_MODAL, |ui, area| {
                ui.register_control(CLICK_MODAL, area, Focusability::Focusable);
            });
        }
    }

    #[test]
    fn pointer_press_count_tracks_triples_and_keeps_double_click_phase() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }

        for _ in 0..3 {
            let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            rt.advance_to(
                rt.now()
                    .saturating_add(core::time::Duration::from_millis(100)),
            )
            .unwrap();
        }

        assert_eq!(
            rt.app().counts,
            [
                (Phase::Press, 1),
                (Phase::Release, 1),
                (Phase::Click, 1),
                (Phase::Press, 2),
                (Phase::Release, 2),
                (Phase::DoubleClick, 2),
                (Phase::Press, 3),
                (Phase::Release, 3),
                (Phase::Click, 3),
            ]
        );
        assert_eq!(
            rt.app()
                .phases
                .iter()
                .filter(|phase| **phase == Phase::DoubleClick)
                .count(),
            1,
            "existing second-click phase remains available"
        );
    }

    #[test]
    fn pointer_press_count_resets_after_slop_and_target_change() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        for x in [2, 4, 12, 2] {
            let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, x, 0));
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, x, 0));
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        }
        rt.set_pointer_click_slop(2);
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 4, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 4, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(
            rt.app()
                .counts
                .iter()
                .filter_map(|(phase, count)| (*phase == Phase::Press).then_some(*count))
                .collect::<Vec<_>>(),
            [1, 1, 1, 1, 2]
        );
    }

    #[test]
    fn capture_interruption_resets_the_pointer_click_sequence() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let click = |rt: &mut Runtime<PressCountProbe>, buf: &mut Buffer| {
            let _ = deliver(rt, super::stub::mouse(MouseKind::Down, 2, 0));
            rt.draw_buffer(SCREEN, buf).commit_presented();
            let _ = deliver(rt, super::stub::mouse(MouseKind::Up, 2, 0));
            rt.draw_buffer(SCREEN, buf).commit_presented();
        };

        click(&mut rt, &mut buf);
        rt.app_mut().capture_on_press = true;
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 20, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        rt.app_mut().capture_on_press = false;
        click(&mut rt, &mut buf);

        assert_eq!(
            rt.app()
                .counts
                .iter()
                .filter_map(|(phase, count)| (*phase == Phase::Press).then_some(*count))
                .collect::<Vec<_>>(),
            [1, 2, 1]
        );
    }

    #[test]
    fn successful_captured_clicks_keep_the_sequence_count() {
        let mut rt = Runtime::new(
            PressCountProbe {
                capture_on_press: true,
                ..PressCountProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        for _ in 0..3 {
            let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        }
        assert_eq!(
            rt.app()
                .counts
                .iter()
                .filter_map(|(phase, count)| (*phase == Phase::Press).then_some(*count))
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }

    #[test]
    fn a_long_second_press_expires_double_click_at_uncaptured_release() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(100)),
        );
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.app().counts.last(), Some(&(Phase::Press, 2)));
        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(600)),
        );
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(rt.app().counts.last(), Some(&(Phase::Click, 1)));
        assert!(!rt.app().phases.contains(&Phase::DoubleClick));
    }

    #[test]
    fn a_long_second_press_expires_double_click_at_captured_release() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let click = |rt: &mut Runtime<PressCountProbe>, buf: &mut Buffer| {
            let _ = deliver(rt, super::stub::mouse(MouseKind::Down, 2, 0));
            rt.draw_buffer(SCREEN, buf).commit_presented();
            let _ = deliver(rt, super::stub::mouse(MouseKind::Up, 2, 0));
            rt.draw_buffer(SCREEN, buf).commit_presented();
        };
        click(&mut rt, &mut buf);
        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(100)),
        );
        rt.app_mut().capture_on_press = true;
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.app().counts.last(), Some(&(Phase::Press, 2)));
        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(600)),
        );
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(rt.app().counts.last(), Some(&(Phase::Click, 1)));
        assert!(!rt.app().phases.contains(&Phase::DoubleClick));
    }

    #[test]
    fn click_slop_uses_release_position_for_uncaptured_clicks() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 3, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 4, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(rt.app().counts.last(), Some(&(Phase::Press, 2)));
    }

    #[test]
    fn click_slop_uses_release_position_for_captured_clicks() {
        let mut rt = Runtime::new(
            PressCountProbe {
                capture_on_press: true,
                ..PressCountProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let click = |rt: &mut Runtime<PressCountProbe>, buf: &mut Buffer| {
            let _ = deliver(rt, super::stub::mouse(MouseKind::Down, 2, 0));
            rt.draw_buffer(SCREEN, buf).commit_presented();
            let _ = deliver(rt, super::stub::mouse(MouseKind::Up, 3, 0));
            rt.draw_buffer(SCREEN, buf).commit_presented();
        };
        click(&mut rt, &mut buf);
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 4, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(rt.app().counts.last(), Some(&(Phase::Press, 2)));
    }

    #[test]
    fn click_window_is_exclusive_at_exact_boundary() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let window = rt.theme().design.motion.double_click_ms;
        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(window)),
        );
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(rt.app().counts.last(), Some(&(Phase::Press, 1)));
    }

    #[test]
    fn click_window_is_exclusive_when_release_lands_on_deadline() {
        let mut rt = Runtime::new(PressCountProbe::default(), Theme::junie());
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        let window = rt.theme().design.motion.double_click_ms;
        let lead_in = window.saturating_sub(100);
        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(lead_in)),
        );
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert_eq!(rt.app().counts.last(), Some(&(Phase::Press, 2)));
        let _ = rt.advance_to(
            rt.now()
                .saturating_add(core::time::Duration::from_millis(100)),
        );
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(rt.app().counts.last(), Some(&(Phase::Click, 1)));
        assert!(!rt.app().phases.contains(&Phase::DoubleClick));
    }

    #[test]
    fn opening_and_dismissing_a_layer_resets_the_pointer_click_sequence() {
        let mut rt = Runtime::new(
            PressCountProbe {
                open_modal_on_click: true,
                ..PressCountProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Up, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert!(rt.is_open(CLICK_MODAL));

        let _ = deliver(&mut rt, super::stub::key(KeyCode::Esc));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        rt.app_mut().open_modal_on_click = false;
        let _ = deliver(&mut rt, super::stub::mouse(MouseKind::Down, 2, 0));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();

        assert_eq!(
            rt.app()
                .counts
                .iter()
                .filter_map(|(phase, count)| (*phase == Phase::Press).then_some(*count))
                .collect::<Vec<_>>(),
            [1, 1]
        );
    }

    #[test]
    fn settle_skips_focus_out_from_closed_nonrestoring_modal() {
        let mut rt = Runtime::new(
            RetiredLayerFocusProbe {
                nonrestoring_modal: true,
                ..RetiredLayerFocusProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        rt.app_mut().open_on_enter = true;
        let _ = step_retired_layer_probe(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.is_open(RETIRED_FOCUS_LAYER));
        assert_eq!(rt.focus(), Some(RETIRED_FOCUS_CONTROL));

        rt.app_mut().close_on_key = true;
        let _ = deliver(&mut rt, key(KeyCode::Char('c')));
        assert!(!rt.is_open(RETIRED_FOCUS_LAYER));
        rt.draw_buffer(SCREEN, &mut buf).commit_presented();
        assert!(rt.needs_settle(), "publication schedules focus settle");
        let _ = rt.settle();
        assert!(!rt.diagnostics().iter().any(
            |diagnostic| matches!(diagnostic, Diagnostic::UndeliveredIntent { owner } if *owner == RETIRED_FOCUS_CONTROL)
        ), "settle must not deliver blur to a closed nonrestoring layer: {:?}", rt.diagnostics());
        assert_eq!(rt.focus(), Some(A));
    }

    struct UiDiagnosticsProbe;

    impl App for UiDiagnosticsProbe {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.diagnostic(Diagnostic::UnaddressableId {
                id: Id::root("preexisting-author-diagnostic"),
            });
            for _ in 0..69 {
                ui.diagnostic(Diagnostic::InvalidComponentLayout {
                    owner: Id::root("form"),
                    field: None,
                });
            }
        }
    }

    #[test]
    fn component_author_ui_diagnostics_commit_to_the_bounded_runtime_store() {
        let mut rt = Runtime::new(UiDiagnosticsProbe, Theme::junie());
        let _ = rt.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buffer).commit_presented();

        assert_eq!(rt.diagnostics().len(), 64);
        assert_eq!(rt.diagnostics_dropped(), 6);
        assert!(matches!(
            rt.diagnostics().first(),
            Some(Diagnostic::UnaddressableId { id })
                if *id == Id::root("preexisting-author-diagnostic")
        ));
        assert_eq!(
            rt.diagnostics()
                .iter()
                .filter(|diagnostic| matches!(diagnostic, Diagnostic::InvalidComponentLayout { owner, field: None } if *owner == Id::root("form")))
                .count(),
            63
        );
    }

    const BUBBLE_OUTER_SCOPE: Id = Id::root("bubble-outer-scope");
    const BUBBLE_INNER_SCOPE: Id = Id::root("bubble-inner-scope");
    const BUBBLE_CHILD: Id = Id::root("bubble-child");
    const BUBBLE_SIBLING_CHILD: Id = Id::root("bubble-sibling-child");
    const BUBBLE_MODAL: Id = Id::root("bubble-modal");
    const BUBBLE_MODAL_CHILD: Id = Id::root("bubble-modal-child");
    const BUBBLE_OUTER_OWNER: Id = Id::root("bubble-outer-owner");
    const BUBBLE_INNER_OWNER: Id = Id::root("bubble-inner-owner");
    const BUBBLE_ACTION: ActionKey = ActionKey::custom("test.scope.dismiss");
    const BUBBLE_BINDINGS: &[crate::keymap::Binding<()>] = &[crate::keymap::Binding {
        action: BUBBLE_ACTION,
        chord: Some(crate::event::Chord::key(KeyCode::Esc)),
        cmd: (),
        label: "Dismiss",
        priority: 1,
        visible: false,
    }];

    #[derive(Default)]
    struct ScopedDeclineProbe {
        publish_outer: bool,
        open_modal_on_inner: bool,
        open_modal_on_leaf: bool,
        move_focus_to_sibling_on_inner: bool,
        move_focus_to_sibling_on_leaf: bool,
        inner_calls: usize,
        outer_calls: usize,
        esc_calls: usize,
    }

    impl App for ScopedDeclineProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(BUBBLE_CHILD);
            }
            if cx
                .intents(BUBBLE_CHILD)
                .any(|intent| matches!(intent, Intent::Key(key) if key.code == KeyCode::Esc))
            {
                if self.open_modal_on_leaf {
                    cx.open_layer(
                        BUBBLE_MODAL,
                        LayerSpec::modal(BUBBLE_MODAL).dismiss(crate::layer::Dismiss::NONE),
                    );
                }
                if self.move_focus_to_sibling_on_leaf {
                    cx.focus(BUBBLE_SIBLING_CHILD);
                }
            }
            if cx
                .intents(BUBBLE_INNER_OWNER)
                .any(|intent| matches!(intent, Intent::Binding(BUBBLE_ACTION)))
            {
                self.inner_calls += 1;
                if self.open_modal_on_inner {
                    cx.open_layer(
                        BUBBLE_MODAL,
                        LayerSpec::modal(BUBBLE_MODAL).dismiss(crate::layer::Dismiss::NONE),
                    );
                }
                if self.move_focus_to_sibling_on_inner {
                    cx.focus(BUBBLE_SIBLING_CHILD);
                }
                return Response::ignored();
            }
            if cx
                .intents(BUBBLE_OUTER_OWNER)
                .any(|intent| matches!(intent, Intent::Binding(BUBBLE_ACTION)))
            {
                self.outer_calls += 1;
                return Response::consumed();
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.focus_scope(BUBBLE_OUTER_SCOPE, ScopeMode::Normal, |ui| {
                if self.publish_outer {
                    ui.publish_scope_bindings(
                        ScopeId::new(BUBBLE_OUTER_SCOPE),
                        BUBBLE_OUTER_OWNER,
                        StateFlags::empty(),
                        BUBBLE_BINDINGS,
                    );
                }
                ui.focus_scope(BUBBLE_INNER_SCOPE, ScopeMode::Normal, |ui| {
                    ui.publish_scope_bindings(
                        ScopeId::new(BUBBLE_INNER_SCOPE),
                        BUBBLE_INNER_OWNER,
                        StateFlags::empty(),
                        BUBBLE_BINDINGS,
                    );
                    ui.register_control(BUBBLE_CHILD, SCREEN, Focusability::Focusable);
                });
            });
            ui.register_control(
                BUBBLE_SIBLING_CHILD,
                Rect::new(0, 0, 1, 1),
                Focusability::Focusable,
            );
            let _ = ui.layer(BUBBLE_MODAL, |ui, area| {
                ui.register_control(BUBBLE_MODAL_CHILD, area, Focusability::Focusable);
            });
        }

        fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            self.esc_calls += 1;
            Response::consumed()
        }
    }

    fn ready_scoped_decline_probe(publish_outer: bool) -> (Runtime<ScopedDeclineProbe>, Buffer) {
        let mut rt = Runtime::new(
            ScopedDeclineProbe {
                publish_outer,
                ..ScopedDeclineProbe::default()
            },
            Theme::junie(),
        );
        let _ = rt.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        for _ in 0..8 {
            rt.draw_buffer(SCREEN, &mut buffer).commit_presented();
            if !rt.needs_settle() {
                break;
            }
            let _ = rt.settle();
        }
        assert_eq!(rt.focus(), Some(BUBBLE_CHILD));
        (rt, buffer)
    }

    #[test]
    fn declining_scoped_bubble_falls_through_to_application_escape() {
        let (mut rt, _) = ready_scoped_decline_probe(false);
        let _ = deliver(&mut rt, key(KeyCode::Esc));
        assert_eq!(rt.app().inner_calls, 1);
        assert_eq!(rt.app().outer_calls, 0);
        assert_eq!(rt.app().esc_calls, 1);
    }

    #[test]
    fn declining_inner_scope_continues_to_outer_scoped_bubble() {
        let (mut rt, _) = ready_scoped_decline_probe(true);
        let _ = deliver(&mut rt, key(KeyCode::Esc));
        assert_eq!(rt.app().inner_calls, 1);
        assert_eq!(rt.app().outer_calls, 1);
        assert_eq!(rt.app().esc_calls, 0);
    }

    #[test]
    fn opening_modal_from_inner_scope_skips_stale_outer_binding() {
        let (mut rt, _) = ready_scoped_decline_probe(true);
        rt.app_mut().open_modal_on_inner = true;
        let _ = deliver(&mut rt, key(KeyCode::Esc));
        assert_eq!(rt.app().inner_calls, 1);
        assert_eq!(rt.app().outer_calls, 0);
        assert_eq!(rt.app().esc_calls, 0);
        assert!(rt.is_open(BUBBLE_MODAL));
    }

    #[test]
    fn opening_modal_from_leaf_skips_stale_global_escape_fallback() {
        let (mut rt, _) = ready_scoped_decline_probe(true);
        rt.app_mut().open_modal_on_leaf = true;
        let _ = deliver(&mut rt, key(KeyCode::Esc));
        assert!(rt.is_open(BUBBLE_MODAL));
        assert_eq!(rt.app().inner_calls, 0);
        assert_eq!(rt.app().outer_calls, 0);
        assert_eq!(rt.app().esc_calls, 0);
    }

    #[test]
    fn moving_focus_from_leaf_skips_stale_scope_and_global_fallbacks() {
        let (mut rt, _) = ready_scoped_decline_probe(true);
        rt.app_mut().move_focus_to_sibling_on_leaf = true;
        let _ = deliver(&mut rt, key(KeyCode::Esc));
        assert_eq!(rt.focus(), Some(BUBBLE_SIBLING_CHILD));
        assert_eq!(rt.app().inner_calls, 0);
        assert_eq!(rt.app().outer_calls, 0);
        assert_eq!(rt.app().esc_calls, 0);
    }

    #[test]
    fn focus_moving_out_of_inner_scope_skips_stale_scope_candidates() {
        let (mut rt, _) = ready_scoped_decline_probe(true);
        rt.app_mut().move_focus_to_sibling_on_inner = true;
        let _ = deliver(&mut rt, key(KeyCode::Esc));
        assert_eq!(rt.app().inner_calls, 1);
        assert_eq!(rt.focus(), Some(BUBBLE_SIBLING_CHILD));
        assert_eq!(rt.app().outer_calls, 0);
        assert_eq!(rt.app().esc_calls, 0);
    }

    const PASSIVE_HOVER: Id = Id::root("passive-hover");
    const PASSIVE_BUTTON: Id = Id::root("passive-button");
    const PASSIVE_EDITOR: Id = Id::root("passive-editor");
    const PASSIVE_NEXT: Id = Id::root("passive-next");
    const PASSIVE_PAGE_SCROLL: Id = Id::root("passive-page-scroll");
    const PASSIVE_REVEAL_SCROLL: Id = Id::root("passive-reveal-scroll");
    const PASSIVE_SECOND_REVEAL_SCROLL: Id = Id::root("passive-second-reveal-scroll");
    const PASSIVE_MODAL: Id = Id::root("passive-modal");
    const PASSIVE_MODAL_BUTTON: Id = Id::root("passive-modal-button");
    const PASSIVE_MODAL_SCROLL: Id = Id::root("passive-modal-scroll");
    const PASSIVE_BUTTON_AREA: Rect = Rect::new(0, 0, 8, 2);
    const PASSIVE_EDITOR_AREA: Rect = Rect::new(0, 3, 8, 1);
    const PASSIVE_NEXT_AREA: Rect = Rect::new(0, 4, 8, 1);
    const PASSIVE_REVEAL_AREA: Rect = Rect::new(10, 0, 10, 10);

    #[derive(Default)]
    struct PassiveRuntimeProbe {
        pointer: Vec<(Id, crate::intent::Phase)>,
        keys: Vec<(Id, crate::event::Key)>,
        wheels: Vec<(Id, crate::event::Axis, i16)>,
        modal_open: bool,
        open_modal_next: bool,
        capture_button: bool,
        normal_scroll: bool,
        second_reveal_scroll: bool,
        reveal_scroll: bool,
        normal_headroom: crate::hit::Headroom,
        second_reveal_headroom: crate::hit::Headroom,
        reveal_headroom: crate::hit::Headroom,
    }

    impl App for PassiveRuntimeProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.update_cause() == UpdateCause::Bootstrap {
                cx.focus(PASSIVE_EDITOR);
                return Response::ignored();
            }
            for owner in [
                PASSIVE_BUTTON,
                PASSIVE_EDITOR,
                PASSIVE_NEXT,
                PASSIVE_PAGE_SCROLL,
                PASSIVE_SECOND_REVEAL_SCROLL,
                PASSIVE_REVEAL_SCROLL,
                PASSIVE_MODAL_BUTTON,
                PASSIVE_MODAL_SCROLL,
            ] {
                for intent in cx.intents(owner) {
                    match intent {
                        Intent::Pointer { phase, part, .. } => {
                            self.pointer.push((owner, phase));
                            if owner == PASSIVE_BUTTON
                                && phase == crate::intent::Phase::Press
                                && self.capture_button
                            {
                                assert!(cx.capture(PASSIVE_BUTTON, part));
                            }
                        }
                        Intent::Key(key) => self.keys.push((owner, key)),
                        Intent::Wheel { axis, delta, .. } => {
                            self.wheels.push((owner, axis, delta));
                        }
                        _ => {}
                    }
                }
            }
            if self.open_modal_next && cx.update_cause() == UpdateCause::Event {
                self.open_modal_next = false;
                self.modal_open = true;
                cx.open_layer(PASSIVE_MODAL, LayerSpec::modal(PASSIVE_MODAL));
            }
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            ui.register_control(PASSIVE_BUTTON, PASSIVE_BUTTON_AREA, Focusability::Focusable);
            ui.register_part(
                PASSIVE_BUTTON,
                PartRef::of(crate::id::Part::LABEL),
                PASSIVE_BUTTON_AREA,
            );
            ui.register_editor(
                PASSIVE_EDITOR,
                PASSIVE_EDITOR_AREA,
                Focusability::Focusable,
                crate::response::StateFlags::EDITING,
            );
            ui.publish_typing_target::<u8>(PASSIVE_EDITOR, &[], |_| true, false);
            ui.register_control(PASSIVE_NEXT, PASSIVE_NEXT_AREA, Focusability::Focusable);
            ui.register_hover_part(PASSIVE_HOVER, PartRef::of(crate::id::Part::HELP), SCREEN);
            if self.normal_scroll {
                ui.register_scroll(
                    PASSIVE_PAGE_SCROLL,
                    SCREEN,
                    crate::hit::Axes::Both,
                    self.normal_headroom,
                );
            }
            if self.second_reveal_scroll {
                ui.register_passive_reveal_scroll(
                    PASSIVE_SECOND_REVEAL_SCROLL,
                    PASSIVE_REVEAL_AREA,
                    self.second_reveal_headroom,
                );
            }
            if self.reveal_scroll {
                ui.register_passive_reveal_scroll(
                    PASSIVE_REVEAL_SCROLL,
                    PASSIVE_REVEAL_AREA,
                    self.reveal_headroom,
                );
            }
            if self.modal_open {
                let _ = ui.layer(PASSIVE_MODAL, |ui, area| {
                    ui.register_control(PASSIVE_MODAL_BUTTON, area, Focusability::Focusable);
                    if self.normal_scroll {
                        ui.register_scroll(
                            PASSIVE_MODAL_SCROLL,
                            area,
                            crate::hit::Axes::Both,
                            self.normal_headroom,
                        );
                    }
                });
            }
        }
    }

    fn passive_runtime(
        probe: PassiveRuntimeProbe,
        theme: Theme,
    ) -> (Runtime<PassiveRuntimeProbe>, Buffer) {
        let mut rt = Runtime::new(probe, theme);
        let _ = rt.initialize();
        let mut buf = Buffer::empty(SCREEN);
        for _ in 0..16 {
            rt.draw_buffer(SCREEN, &mut buf).commit_presented();
            if !rt.needs_settle() {
                return (rt, buf);
            }
            let _ = rt.settle();
        }
        panic!(
            "passive runtime fixture did not settle: {:?}",
            rt.diagnostics()
        );
    }

    fn passive_present(rt: &mut Runtime<PassiveRuntimeProbe>, buf: &mut Buffer) {
        for _ in 0..16 {
            if rt.needs_settle() {
                let _ = rt.settle();
            }
            if rt.needs_present() {
                rt.draw_buffer(SCREEN, buf).commit_presented();
            }
            if !rt.needs_settle() && !rt.needs_present() {
                return;
            }
        }
        panic!(
            "passive runtime frame did not settle: {:?}",
            rt.diagnostics()
        );
    }

    fn passive_mouse(kind: MouseKind, x: u16, y: u16, mods: crate::event::KeyModifiers) -> Input {
        Input::Mouse(crate::event::Mouse {
            kind,
            pos: Position::new(x, y),
            mods,
        })
    }

    fn passive_deliver(rt: &mut Runtime<PassiveRuntimeProbe>, input: Input, buf: &mut Buffer) {
        let _ = deliver(rt, input);
        passive_present(rt, buf);
    }

    #[test]
    fn hover_only_keeps_first_click_secondary_and_keyboard_ownership_unchanged() {
        let (mut rt, mut buf) = passive_runtime(PassiveRuntimeProbe::default(), Theme::junie());
        assert_eq!(rt.focus(), Some(PASSIVE_EDITOR));

        passive_deliver(&mut rt, mouse(MouseKind::Move, 2, 0), &mut buf);
        assert_eq!(rt.hover(), Some(PASSIVE_HOVER));
        assert_eq!(
            rt.focus(),
            Some(PASSIVE_EDITOR),
            "passive hover never takes focus"
        );
        assert_eq!(rt.capture_owner(), None);
        assert!(
            !rt.app()
                .pointer
                .iter()
                .any(|(owner, _)| *owner == PASSIVE_HOVER)
        );

        passive_deliver(&mut rt, mouse(MouseKind::Down, 2, 0), &mut buf);
        passive_deliver(&mut rt, mouse(MouseKind::Up, 2, 0), &mut buf);
        passive_deliver(&mut rt, mouse(MouseKind::Secondary, 2, 0), &mut buf);
        assert_eq!(
            rt.app()
                .pointer
                .iter()
                .filter(|(owner, phase)| *owner == PASSIVE_BUTTON
                    && *phase == crate::intent::Phase::Click)
                .count(),
            1,
            "the first click is delivered exactly once to the underlying button"
        );
        assert_eq!(
            rt.app()
                .pointer
                .iter()
                .filter(|(owner, phase)| *owner == PASSIVE_BUTTON
                    && *phase == crate::intent::Phase::Secondary)
                .count(),
            1,
            "secondary input also skips the passive region"
        );
        assert!(
            !rt.diagnostics().iter().any(|diagnostic| matches!(
                diagnostic,
                crate::diagnostics::Diagnostic::UndeliveredIntent { .. }
            )),
            "passive regions emit no undelivered intents"
        );

        let (mut rt, mut buf) = passive_runtime(PassiveRuntimeProbe::default(), Theme::junie());
        passive_deliver(&mut rt, mouse(MouseKind::Move, 2, 3), &mut buf);
        assert_eq!(rt.hover(), Some(PASSIVE_HOVER));
        let editor_keys = [
            crate::event::Key {
                code: KeyCode::Char('x'),
                mods: crate::event::KeyModifiers::NONE,
            },
            crate::event::Key {
                code: KeyCode::Char('s'),
                mods: crate::event::KeyModifiers::CONTROL,
            },
            crate::event::Key {
                code: KeyCode::Esc,
                mods: crate::event::KeyModifiers::NONE,
            },
        ];
        for editor_key in editor_keys {
            passive_deliver(&mut rt, Input::Key(editor_key), &mut buf);
            assert_eq!(rt.focus(), Some(PASSIVE_EDITOR));
            assert!(rt.app().keys.contains(&(PASSIVE_EDITOR, editor_key)));
            assert_eq!(
                rt.hover(),
                None,
                "keyboard input suppresses passive hover until pointer motion"
            );
            passive_deliver(&mut rt, mouse(MouseKind::Move, 2, 3), &mut buf);
            assert_eq!(rt.hover(), Some(PASSIVE_HOVER));
        }
        passive_deliver(&mut rt, key(KeyCode::Tab), &mut buf);
        assert_eq!(rt.focus(), Some(PASSIVE_NEXT));
        assert!(!rt.diagnostics().iter().any(|diagnostic| matches!(
            diagnostic,
            crate::diagnostics::Diagnostic::UndeliveredIntent { .. }
        )));
    }

    #[test]
    fn hover_only_does_not_receive_captured_drag_motion() {
        let probe = PassiveRuntimeProbe {
            capture_button: true,
            ..PassiveRuntimeProbe::default()
        };
        let (mut rt, mut buf) = passive_runtime(probe, Theme::junie());
        passive_deliver(&mut rt, mouse(MouseKind::Down, 2, 0), &mut buf);
        assert_eq!(rt.capture_owner(), Some(PASSIVE_BUTTON));
        passive_deliver(&mut rt, mouse(MouseKind::Move, 2, 0), &mut buf);
        assert!(
            rt.app()
                .pointer
                .iter()
                .any(|(owner, phase)| *owner == PASSIVE_BUTTON
                    && *phase == crate::intent::Phase::Drag)
        );
        assert!(
            !rt.app()
                .pointer
                .iter()
                .any(|(owner, _)| *owner == PASSIVE_HOVER)
        );
        passive_deliver(&mut rt, mouse(MouseKind::Up, 2, 0), &mut buf);
        assert_eq!(rt.capture_owner(), None);
    }

    #[test]
    fn passive_reveal_wheel_uses_three_lines_and_same_layer_fallback_rules() {
        use crate::event::{Axis, KeyModifiers};
        use crate::hit::Headroom;

        let mut theme = Theme::junie();
        theme.design.motion.wheel_rows = 5;
        let probe = PassiveRuntimeProbe {
            normal_scroll: true,
            reveal_scroll: true,
            normal_headroom: Headroom {
                up: 10,
                down: 10,
                left: 10,
                right: 10,
            },
            reveal_headroom: Headroom {
                up: 10,
                down: 10,
                ..Headroom::default()
            },
            ..PassiveRuntimeProbe::default()
        };
        let (mut rt, mut buf) = passive_runtime(probe, theme);
        let cases = [
            (
                Axis::V,
                2,
                KeyModifiers::NONE,
                Headroom {
                    up: 10,
                    down: 10,
                    ..Headroom::default()
                },
                vec![(PASSIVE_REVEAL_SCROLL, Axis::V, 6)],
            ),
            (
                Axis::V,
                -4,
                KeyModifiers::NONE,
                Headroom {
                    up: 10,
                    down: 10,
                    ..Headroom::default()
                },
                vec![(PASSIVE_REVEAL_SCROLL, Axis::V, -12)],
            ),
            (
                Axis::V,
                2,
                KeyModifiers::SHIFT,
                Headroom {
                    up: 10,
                    down: 10,
                    ..Headroom::default()
                },
                vec![(PASSIVE_PAGE_SCROLL, Axis::V, 10)],
            ),
            (
                Axis::H,
                2,
                KeyModifiers::NONE,
                Headroom {
                    up: 10,
                    down: 10,
                    ..Headroom::default()
                },
                vec![(PASSIVE_PAGE_SCROLL, Axis::H, 10)],
            ),
            (
                Axis::V,
                0,
                KeyModifiers::NONE,
                Headroom {
                    up: 10,
                    down: 10,
                    ..Headroom::default()
                },
                vec![(PASSIVE_PAGE_SCROLL, Axis::V, 0)],
            ),
            (
                Axis::V,
                2,
                KeyModifiers::NONE,
                Headroom::default(),
                vec![(PASSIVE_PAGE_SCROLL, Axis::V, 10)],
            ),
            (
                Axis::V,
                2,
                KeyModifiers::NONE,
                Headroom {
                    up: 10,
                    down: 0,
                    ..Headroom::default()
                },
                vec![(PASSIVE_PAGE_SCROLL, Axis::V, 10)],
            ),
            (
                Axis::V,
                -2,
                KeyModifiers::NONE,
                Headroom {
                    up: 0,
                    down: 10,
                    ..Headroom::default()
                },
                vec![(PASSIVE_PAGE_SCROLL, Axis::V, -10)],
            ),
        ];
        for (axis, delta, mods, headroom, expected) in cases {
            rt.app_mut().wheels.clear();
            rt.app_mut().reveal_headroom = headroom;
            passive_deliver(
                &mut rt,
                passive_mouse(MouseKind::Wheel(axis, delta), 12, 2, mods),
                &mut buf,
            );
            assert_eq!(
                rt.app().wheels,
                expected,
                "wheel case {axis:?} {delta} {mods:?} {headroom:?}"
            );
        }

        rt.app_mut().wheels.clear();
        rt.app_mut().reveal_headroom = Headroom {
            up: 10,
            down: 10,
            ..Headroom::default()
        };
        passive_deliver(
            &mut rt,
            passive_mouse(MouseKind::Wheel(Axis::V, 2), 4, 2, KeyModifiers::NONE),
            &mut buf,
        );
        assert_eq!(
            rt.app().wheels,
            [(PASSIVE_PAGE_SCROLL, Axis::V, 10)],
            "outside the reveal region, ordinary page scrolling receives the event"
        );

        rt.app_mut().wheels.clear();
        rt.app_mut().second_reveal_scroll = true;
        rt.app_mut().second_reveal_headroom = Headroom {
            up: 10,
            down: 10,
            ..Headroom::default()
        };
        rt.app_mut().reveal_headroom = Headroom {
            up: 10,
            down: 0,
            ..Headroom::default()
        };
        passive_deliver(
            &mut rt,
            passive_mouse(MouseKind::Wheel(Axis::V, 2), 12, 2, KeyModifiers::NONE),
            &mut buf,
        );
        assert_eq!(
            rt.app().wheels,
            [(PASSIVE_PAGE_SCROLL, Axis::V, 10)],
            "a skipped reveal candidate continues to the next normal scroll target, not another reveal"
        );

        rt.app_mut().wheels.clear();
        rt.app_mut().reveal_scroll = false;
        rt.app_mut().second_reveal_scroll = false;
        rt.app_mut().normal_headroom = Headroom::default();
        passive_deliver(
            &mut rt,
            passive_mouse(MouseKind::Wheel(Axis::V, 1), 12, 2, KeyModifiers::NONE),
            &mut buf,
        );
        assert_eq!(
            rt.app().wheels,
            [(PASSIVE_PAGE_SCROLL, Axis::V, 5)],
            "ordinary zero-headroom scroll preserves legacy endpoint consumption"
        );

        rt.app_mut().wheels.clear();
        rt.app_mut().normal_scroll = false;
        rt.app_mut().reveal_scroll = true;
        rt.app_mut().reveal_headroom = Headroom {
            up: 2,
            down: 0,
            ..Headroom::default()
        };
        passive_deliver(
            &mut rt,
            passive_mouse(MouseKind::Wheel(Axis::V, 1), 4, 2, KeyModifiers::NONE),
            &mut buf,
        );
        assert!(
            rt.app().wheels.is_empty(),
            "passing the reveal end does not invent a target"
        );
    }

    #[test]
    fn passive_reveal_wheel_never_falls_through_a_modal_top_layer() {
        use crate::event::{Axis, KeyModifiers};
        use crate::hit::Headroom;

        let probe = PassiveRuntimeProbe {
            normal_scroll: true,
            reveal_scroll: true,
            normal_headroom: Headroom {
                up: 5,
                down: 5,
                ..Headroom::default()
            },
            reveal_headroom: Headroom {
                up: 5,
                down: 5,
                ..Headroom::default()
            },
            ..PassiveRuntimeProbe::default()
        };
        let (mut rt, mut buf) = passive_runtime(probe, Theme::junie());
        rt.app_mut().open_modal_next = true;
        passive_deliver(&mut rt, key(KeyCode::Enter), &mut buf);
        assert!(rt.is_open(PASSIVE_MODAL));
        passive_deliver(&mut rt, mouse(MouseKind::Move, 4, 2), &mut buf);
        assert_eq!(rt.hover(), Some(PASSIVE_MODAL_BUTTON));
        rt.app_mut().wheels.clear();
        passive_deliver(
            &mut rt,
            passive_mouse(MouseKind::Wheel(Axis::V, 2), 12, 2, KeyModifiers::NONE),
            &mut buf,
        );
        assert_eq!(rt.app().wheels, [(PASSIVE_MODAL_SCROLL, Axis::V, 6)]);
        assert!(
            !rt.app()
                .wheels
                .iter()
                .any(|(owner, _, _)| *owner == PASSIVE_PAGE_SCROLL
                    || *owner == PASSIVE_REVEAL_SCROLL)
        );
        assert!(
            !rt.app()
                .pointer
                .iter()
                .any(|(owner, _)| *owner == PASSIVE_HOVER)
        );
    }

    const ROUTE_ROOT: Id = Id::root("wheel-route-root");
    const ROUTE_PARENT: Id = Id::root("wheel-route-parent");
    const ROUTE_CHILD: Id = Id::root("wheel-route-child");
    const ROUTE_MODAL: Id = Id::root("wheel-route-modal");
    const ROUTE_POINT: Position = Position::new(4, 4);
    const ROUTE_OUTER: Rect = Rect::new(0, 0, 24, 10);
    const ROUTE_INNER: Rect = Rect::new(2, 2, 12, 5);

    #[derive(Clone, Copy, Debug)]
    struct RoutedWheelNode {
        owner: Id,
        area: Rect,
        source_revision: u64,
        parent: Option<usize>,
    }

    #[derive(Debug)]
    struct RoutedWheelProbe {
        nodes: Vec<RoutedWheelNode>,
        update_order: Vec<Id>,
        route_enabled: bool,
        focusable_controls: bool,
        capture_on_wheel: bool,
        modal_on_bootstrap: bool,
        initial_result: Option<WheelRouteResult>,
        callback_results: Vec<(Id, Option<WheelRouteResult>)>,
        live_child_revision: Option<u64>,
        live_parent_revision: Option<u64>,
        stale_attempt: Option<WheelRouteAttempt>,
        use_stale_attempt: bool,
        duplicate_initial_ack: bool,
        duplicate_service_ack: bool,
        open_close_on_service: Option<Id>,
        focus_child_on_event: bool,
        initial_relayout: bool,
        service_relayout: bool,
        update_causes: Vec<UpdateCause>,
        service_causes: Vec<UpdateCause>,
        event_updates: usize,
        wheel_deliveries: Vec<(Id, Axis, i16, Position)>,
        initial_attempts: Vec<WheelRouteAttempt>,
        service_attempts: Vec<WheelRouteAttempt>,
        service_intent_counts: Vec<usize>,
        ack_results: Vec<bool>,
    }

    impl RoutedWheelProbe {
        fn new(nodes: Vec<RoutedWheelNode>, update_order: Vec<Id>) -> Self {
            Self {
                nodes,
                update_order,
                route_enabled: true,
                focusable_controls: false,
                capture_on_wheel: false,
                modal_on_bootstrap: false,
                initial_result: Some(WheelRouteResult::Ignored),
                callback_results: Vec::new(),
                live_child_revision: None,
                live_parent_revision: None,
                stale_attempt: None,
                use_stale_attempt: false,
                duplicate_initial_ack: false,
                duplicate_service_ack: false,
                open_close_on_service: None,
                focus_child_on_event: false,
                initial_relayout: false,
                service_relayout: false,
                update_causes: Vec::new(),
                service_causes: Vec::new(),
                event_updates: 0,
                wheel_deliveries: Vec::new(),
                initial_attempts: Vec::new(),
                service_attempts: Vec::new(),
                service_intent_counts: Vec::new(),
                ack_results: Vec::new(),
            }
        }

        fn source_revision(&self, owner: Id) -> Option<u64> {
            self.nodes
                .iter()
                .rev()
                .find(|node| node.owner == owner)
                .map(|node| {
                    if owner == ROUTE_CHILD {
                        self.live_child_revision.unwrap_or(node.source_revision)
                    } else if owner == ROUTE_PARENT {
                        self.live_parent_revision.unwrap_or(node.source_revision)
                    } else {
                        node.source_revision
                    }
                })
        }
    }

    impl App for RoutedWheelProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.update_causes.push(cx.update_cause());
            if cx.update_cause() == UpdateCause::Bootstrap {
                if self.modal_on_bootstrap {
                    cx.open_layer(ROUTE_MODAL, LayerSpec::modal(ROUTE_MODAL));
                }
                return Response::ignored();
            }
            if cx.update_cause() == UpdateCause::Event {
                self.event_updates = self.event_updates.saturating_add(1);
                for owner in self.update_order.clone() {
                    for intent in cx.intents(owner) {
                        if let Intent::Wheel {
                            axis,
                            delta,
                            part,
                            pos,
                        } = intent
                        {
                            self.wheel_deliveries.push((owner, axis, delta, pos));
                            let Some(attempt) = cx.wheel_route_attempt(owner, part, axis) else {
                                continue;
                            };
                            self.initial_attempts.push(attempt);
                            if self.capture_on_wheel && owner == ROUTE_CHILD {
                                assert!(cx.capture(owner, part));
                            }
                            assert_eq!(
                                cx.wheel_route_attempt(
                                    ROUTE_PARENT,
                                    PartRef::of(Part::CONTAINER),
                                    axis,
                                )
                                .is_some(),
                                owner == ROUTE_PARENT
                            );
                            assert!(
                                cx.wheel_route_attempt(owner, PartRef::of(Part::LABEL), axis)
                                    .is_none()
                            );
                            assert!(
                                cx.wheel_route_attempt(
                                    owner,
                                    part,
                                    if axis == Axis::V { Axis::H } else { Axis::V },
                                )
                                .is_none()
                            );
                            if self.use_stale_attempt
                                && let Some(stale) = self.stale_attempt
                            {
                                self.ack_results.push(
                                    cx.acknowledge_wheel_route(&stale, WheelRouteResult::Ignored),
                                );
                            }
                            if self.source_revision(owner) == Some(attempt.source_revision())
                                && let Some(result) = self.initial_result
                            {
                                self.ack_results
                                    .push(cx.acknowledge_wheel_route(&attempt, result));
                                if self.duplicate_initial_ack {
                                    self.ack_results
                                        .push(cx.acknowledge_wheel_route(&attempt, result));
                                }
                            }
                        }
                    }
                }
            }
            if self.focus_child_on_event && cx.update_cause() == UpdateCause::Event {
                cx.focus(ROUTE_CHILD);
            }
            if self.initial_relayout && cx.update_cause() == UpdateCause::Event {
                return Response::ignored().relayout();
            }
            Response::ignored()
        }

        fn route_wheel_attempt(
            &mut self,
            cx: &mut Cx<'_>,
            attempt: WheelRouteAttempt,
        ) -> Response<()> {
            self.service_attempts.push(attempt);
            self.service_causes.push(cx.update_cause());
            self.service_intent_counts
                .push(cx.intents(attempt.owner()).count());
            if let Some(layer) = self.open_close_on_service.take() {
                cx.open_layer(layer, LayerSpec::modal(layer));
                cx.close_layer(layer, None);
            }
            if self.source_revision(attempt.owner()) == Some(attempt.source_revision())
                && let Some(result) = self
                    .callback_results
                    .iter()
                    .find(|(owner, _)| *owner == attempt.owner())
                    .and_then(|(_, result)| *result)
            {
                let accepted = cx.acknowledge_wheel_route(&attempt, result);
                self.ack_results.push(accepted);
                if self.duplicate_service_ack {
                    self.ack_results
                        .push(cx.acknowledge_wheel_route(&attempt, result));
                }
            }
            if self.service_relayout {
                Response::ignored().relayout()
            } else {
                Response::ignored()
            }
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let mut keys = Vec::with_capacity(self.nodes.len());
            for node in &self.nodes {
                if self.focusable_controls {
                    ui.register_control(node.owner, node.area, Focusability::Focusable);
                }
                let key = if self.route_enabled {
                    ui.register_routed_scroll(
                        node.owner,
                        node.area,
                        Axes::V,
                        Headroom::default(),
                        node.source_revision,
                    )
                } else {
                    ui.register_scroll(node.owner, node.area, Axes::V, Headroom::default());
                    None
                };
                keys.push(key);
            }
            if self.modal_on_bootstrap
                && let Some(node) = self.nodes.iter().find(|node| node.owner == ROUTE_CHILD)
            {
                let node = *node;
                let key = ui
                    .layer(ROUTE_MODAL, |ui, _| {
                        if self.focusable_controls {
                            ui.register_control(node.owner, node.area, Focusability::Focusable);
                        }
                        if self.route_enabled {
                            ui.register_routed_scroll(
                                node.owner,
                                node.area,
                                Axes::V,
                                Headroom::default(),
                                node.source_revision,
                            )
                        } else {
                            ui.register_scroll(node.owner, node.area, Axes::V, Headroom::default());
                            None
                        }
                    })
                    .flatten();
                let _ = key;
            }
            for (child_index, node) in self.nodes.iter().enumerate() {
                if let Some(parent_index) = node.parent {
                    let child_key = keys.get(child_index).copied().flatten();
                    let parent_key = keys.get(parent_index).copied().flatten();
                    if let (Some(child), Some(parent)) = (child_key, parent_key) {
                        assert!(ui.link_scroll_parent(child, parent));
                    }
                }
            }
        }
    }

    fn two_level_route_probe(parent_first: bool) -> RoutedWheelProbe {
        let nodes = vec![
            RoutedWheelNode {
                owner: ROUTE_PARENT,
                area: ROUTE_OUTER,
                source_revision: 21,
                parent: None,
            },
            RoutedWheelNode {
                owner: ROUTE_CHILD,
                area: ROUTE_INNER,
                source_revision: 34,
                parent: Some(0),
            },
        ];
        let order = if parent_first {
            vec![ROUTE_PARENT, ROUTE_CHILD]
        } else {
            vec![ROUTE_CHILD, ROUTE_PARENT]
        };
        let mut probe = RoutedWheelProbe::new(nodes, order);
        probe.callback_results = vec![(ROUTE_PARENT, Some(WheelRouteResult::Ignored))];
        probe
    }

    fn three_level_route_probe() -> RoutedWheelProbe {
        let nodes = vec![
            RoutedWheelNode {
                owner: ROUTE_ROOT,
                area: ROUTE_OUTER,
                source_revision: 8,
                parent: None,
            },
            RoutedWheelNode {
                owner: ROUTE_PARENT,
                area: ROUTE_OUTER,
                source_revision: 21,
                parent: Some(0),
            },
            RoutedWheelNode {
                owner: ROUTE_CHILD,
                area: ROUTE_INNER,
                source_revision: 34,
                parent: Some(1),
            },
        ];
        let mut probe = RoutedWheelProbe::new(nodes, vec![ROUTE_CHILD, ROUTE_PARENT, ROUTE_ROOT]);
        probe.callback_results = vec![
            (ROUTE_PARENT, Some(WheelRouteResult::Ignored)),
            (ROUTE_ROOT, Some(WheelRouteResult::Ignored)),
        ];
        probe
    }

    fn ready_routed_runtime(probe: RoutedWheelProbe) -> Runtime<RoutedWheelProbe> {
        let mut runtime = Runtime::new(probe, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(SCREEN);
        for _ in 0..16 {
            if runtime.needs_settle() {
                let _ = runtime.settle();
            }
            if runtime.needs_present() {
                runtime.draw_buffer(SCREEN, &mut buffer).commit_presented();
            }
            if !runtime.needs_settle() && !runtime.needs_present() {
                return runtime;
            }
        }
        panic!(
            "routed wheel fixture did not settle: {:?}",
            runtime.diagnostics()
        );
    }

    fn route_wheel_input(axis: Axis, delta: i16) -> Input {
        Input::Mouse(Mouse {
            kind: MouseKind::Wheel(axis, delta),
            pos: ROUTE_POINT,
            mods: KeyModifiers::NONE,
        })
    }

    #[test]
    fn ignored_routed_child_services_parent_once_in_both_update_orders() {
        for parent_first in [true, false] {
            let mut runtime = ready_routed_runtime(two_level_route_probe(parent_first));
            let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 2));

            let probe = runtime.app();
            assert_eq!(
                probe.event_updates, 1,
                "one physical wheel runs one App event update"
            );
            assert_eq!(
                probe.wheel_deliveries,
                [(
                    ROUTE_CHILD,
                    Axis::V,
                    2 * Theme::junie().design.motion.wheel_rows as i16,
                    ROUTE_POINT
                )]
            );
            assert_eq!(probe.service_attempts.len(), 1);
            let initial = probe.initial_attempts[0];
            let parent = probe.service_attempts[0];
            assert_eq!(initial.owner(), ROUTE_CHILD);
            assert_eq!(parent.owner(), ROUTE_PARENT);
            assert_eq!(initial.event_id(), parent.event_id());
            assert_eq!((initial.attempt_index(), parent.attempt_index()), (0, 1));
            assert_eq!(
                (initial.axis(), initial.delta(), initial.position()),
                (parent.axis(), parent.delta(), parent.position())
            );
            assert_eq!(parent.source_revision(), 21);
            assert_eq!(
                probe.service_intent_counts,
                [0],
                "targeted callback has no replayed intents"
            );
            assert_eq!(probe.service_causes, [UpdateCause::ScrollRoute]);
            assert_eq!(
                probe
                    .update_causes
                    .iter()
                    .filter(|cause| **cause == UpdateCause::Event)
                    .count(),
                1
            );
            assert_eq!(probe.ack_results, [true, true]);
            assert_eq!(runtime.diagnostics(), []);
        }
    }

    #[test]
    fn older_conflicting_owner_regions_keep_one_wheel_delivery_and_suppress_parent_service() {
        let outer_conflict = Rect::new(0, 0, 30, 30);
        let cases = [
            vec![
                RoutedWheelNode {
                    owner: ROUTE_CHILD,
                    area: outer_conflict,
                    source_revision: 99,
                    parent: None,
                },
                RoutedWheelNode {
                    owner: ROUTE_PARENT,
                    area: ROUTE_OUTER,
                    source_revision: 7,
                    parent: None,
                },
                RoutedWheelNode {
                    owner: ROUTE_CHILD,
                    area: ROUTE_INNER,
                    source_revision: 34,
                    parent: Some(1),
                },
            ],
            vec![
                RoutedWheelNode {
                    owner: ROUTE_PARENT,
                    area: outer_conflict,
                    source_revision: 99,
                    parent: None,
                },
                RoutedWheelNode {
                    owner: ROUTE_PARENT,
                    area: ROUTE_OUTER,
                    source_revision: 7,
                    parent: None,
                },
                RoutedWheelNode {
                    owner: ROUTE_CHILD,
                    area: ROUTE_INNER,
                    source_revision: 34,
                    parent: Some(1),
                },
            ],
        ];

        for nodes in cases {
            let mut probe = RoutedWheelProbe::new(nodes, vec![ROUTE_CHILD, ROUTE_PARENT]);
            probe.callback_results = vec![(ROUTE_PARENT, Some(WheelRouteResult::Ignored))];
            let mut runtime = ready_routed_runtime(probe);
            let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 2));

            let app = runtime.app();
            assert_eq!(app.event_updates, 1, "the physical wheel updates once");
            assert_eq!(
                app.wheel_deliveries,
                [(
                    ROUTE_CHILD,
                    Axis::V,
                    2 * Theme::junie().design.motion.wheel_rows as i16,
                    ROUTE_POINT,
                )],
                "the ordinary wheel delivery is preserved exactly once"
            );
            assert_eq!(app.initial_attempts.len(), 1);
            assert_eq!(app.initial_attempts[0].owner(), ROUTE_CHILD);
            assert!(
                app.service_attempts.is_empty(),
                "conflicting older owner metadata blocks the parent callback"
            );
            assert_eq!(app.ack_results, [true]);
            assert_eq!(runtime.diagnostics(), []);
        }
    }

    #[test]
    fn two_ignored_targets_reach_root_once_and_all_ignored_stops_at_root() {
        for root_result in [WheelRouteResult::Consumed, WheelRouteResult::Ignored] {
            let mut probe = three_level_route_probe();
            probe.callback_results = vec![
                (ROUTE_PARENT, Some(WheelRouteResult::Ignored)),
                (ROUTE_ROOT, Some(root_result)),
            ];
            let mut runtime = ready_routed_runtime(probe);
            let _ = deliver(&mut runtime, route_wheel_input(Axis::V, -3));

            let app = runtime.app();
            assert_eq!(app.event_updates, 1);
            assert_eq!(
                app.service_attempts
                    .iter()
                    .map(|attempt| attempt.owner())
                    .collect::<Vec<_>>(),
                [ROUTE_PARENT, ROUTE_ROOT]
            );
            assert_eq!(
                app.service_attempts
                    .iter()
                    .map(|attempt| attempt.attempt_index())
                    .collect::<Vec<_>>(),
                [1, 2]
            );
            assert_eq!(app.service_causes, [UpdateCause::ScrollRoute; 2]);
            assert_eq!(
                app.initial_attempts[0].delta(),
                -3 * Theme::junie().design.motion.wheel_rows as i16
            );
            assert!(app.service_attempts.iter().all(|attempt| {
                attempt.event_id() == app.initial_attempts[0].event_id()
                    && attempt.delta() == app.initial_attempts[0].delta()
                    && attempt.position() == ROUTE_POINT
                    && attempt.axis() == Axis::V
            }));
            assert_eq!(app.ack_results, [true, true, true]);
            assert_eq!(runtime.diagnostics(), []);

            let service_count = runtime.app().service_attempts.len();
            let _ = deliver(&mut runtime, Input::Tick);
            assert_eq!(runtime.app().service_attempts.len(), service_count);
            assert_eq!(runtime.app().event_updates, 1);
        }
    }

    #[test]
    fn moved_consumed_missing_and_stale_results_stop_the_route() {
        for initial_result in [
            Some(WheelRouteResult::Changed),
            Some(WheelRouteResult::Consumed),
            None,
        ] {
            let mut probe = two_level_route_probe(false);
            probe.initial_result = initial_result;
            let mut runtime = ready_routed_runtime(probe);
            let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
            assert!(runtime.app().service_attempts.is_empty());
            assert_eq!(runtime.app().wheel_deliveries.len(), 1);
        }

        for parent_result in [WheelRouteResult::Changed, WheelRouteResult::Consumed] {
            let mut probe = three_level_route_probe();
            probe.callback_results = vec![
                (ROUTE_PARENT, Some(parent_result)),
                (ROUTE_ROOT, Some(WheelRouteResult::Consumed)),
            ];
            let mut runtime = ready_routed_runtime(probe);
            let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
            assert_eq!(
                runtime
                    .app()
                    .service_attempts
                    .iter()
                    .map(|attempt| attempt.owner())
                    .collect::<Vec<_>>(),
                [ROUTE_PARENT],
                "only Ignored may continue to the root"
            );
        }

        let mut probe = two_level_route_probe(false);
        probe.live_child_revision = Some(35);
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert!(
            runtime.app().service_attempts.is_empty(),
            "stale source cannot acknowledge Ignored"
        );

        let mut probe = three_level_route_probe();
        probe.live_parent_revision = Some(22);
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [ROUTE_PARENT]
        );
        assert_eq!(runtime.app().ack_results, [true]);

        let mut probe = three_level_route_probe();
        probe.callback_results.clear();
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [ROUTE_PARENT],
            "a missing parent acknowledgement stops before root"
        );

        let mut probe = three_level_route_probe();
        probe.duplicate_service_ack = true;
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [ROUTE_PARENT],
            "a duplicate parent acknowledgement stops before root"
        );

        let mut probe = two_level_route_probe(false);
        probe.duplicate_initial_ack = true;
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert!(
            runtime.app().service_attempts.is_empty(),
            "duplicate acknowledgement fails closed"
        );
        assert_eq!(runtime.app().ack_results, [true, false]);

        let mut runtime = ready_routed_runtime(two_level_route_probe(false));
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        let previous = runtime.app().initial_attempts[0];
        runtime.app_mut().stale_attempt = Some(previous);
        runtime.app_mut().use_stale_attempt = true;
        let prior_services = runtime.app().service_attempts.len();
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_ne!(
            runtime.app().initial_attempts[1].event_id(),
            previous.event_id()
        );
        assert_eq!(runtime.app().service_attempts.len(), prior_services);
        assert_eq!(
            &runtime.app().ack_results[2..],
            [false, false],
            "a receipt from a previous wheel invalidates the current attempt"
        );
    }

    #[test]
    fn lifecycle_or_layout_invalidation_stops_before_the_next_parent() {
        let mut nodes = two_level_route_probe(false).nodes;
        nodes.insert(
            0,
            RoutedWheelNode {
                owner: ROUTE_ROOT,
                area: ROUTE_OUTER,
                source_revision: 8,
                parent: None,
            },
        );
        nodes[1].parent = Some(0);
        nodes[2].parent = Some(1);

        let mut clean_probe =
            RoutedWheelProbe::new(nodes.clone(), vec![ROUTE_CHILD, ROUTE_PARENT, ROUTE_ROOT]);
        clean_probe.callback_results = vec![
            (ROUTE_PARENT, Some(WheelRouteResult::Ignored)),
            (ROUTE_ROOT, Some(WheelRouteResult::Consumed)),
        ];
        let mut clean_runtime = ready_routed_runtime(clean_probe);
        let _ = deliver(&mut clean_runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            clean_runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [ROUTE_PARENT, ROUTE_ROOT],
            "the unmutated three-level graph reaches both ancestors"
        );
        assert_eq!(clean_runtime.app().event_updates, 1);
        assert_eq!(
            clean_runtime.app().wheel_deliveries,
            [(
                ROUTE_CHILD,
                Axis::V,
                Theme::junie().design.motion.wheel_rows as i16,
                ROUTE_POINT,
            )],
            "clean ancestry preserves one original scaled wheel delivery"
        );
        assert_eq!(clean_runtime.app().initial_attempts.len(), 1);
        assert_eq!(clean_runtime.app().ack_results, [true, true, true]);
        assert_eq!(
            clean_runtime.app().service_intent_counts,
            [0, 0],
            "targeted ancestor callbacks do not replay ordinary intents"
        );
        assert_eq!(
            clean_runtime.app().service_causes,
            [UpdateCause::ScrollRoute; 2]
        );
        assert_eq!(clean_runtime.diagnostics(), []);

        let mut probe =
            RoutedWheelProbe::new(nodes.clone(), vec![ROUTE_CHILD, ROUTE_PARENT, ROUTE_ROOT]);
        probe.open_close_on_service = Some(ROUTE_MODAL);
        probe.callback_results = vec![
            (ROUTE_PARENT, Some(WheelRouteResult::Ignored)),
            (ROUTE_ROOT, Some(WheelRouteResult::Consumed)),
        ];
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(runtime.app().event_updates, 1);
        assert_eq!(
            runtime.app().wheel_deliveries,
            [(
                ROUTE_CHILD,
                Axis::V,
                Theme::junie().design.motion.wheel_rows as i16,
                ROUTE_POINT,
            )],
            "invalidated ancestry preserves one original scaled wheel delivery"
        );
        assert_eq!(
            runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [ROUTE_PARENT],
            "open-close-away-and-back invalidates the remaining route"
        );
        assert_eq!(runtime.app().initial_attempts.len(), 1);
        assert_eq!(runtime.app().ack_results, [true, true]);
        assert_eq!(runtime.app().service_intent_counts, [0]);
        assert_eq!(runtime.app().service_causes, [UpdateCause::ScrollRoute]);
        assert_eq!(runtime.diagnostics(), []);

        let mut probe = RoutedWheelProbe::new(nodes, vec![ROUTE_CHILD, ROUTE_PARENT, ROUTE_ROOT]);
        probe.initial_relayout = true;
        probe.callback_results = vec![(ROUTE_PARENT, Some(WheelRouteResult::Consumed))];
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert!(
            runtime.app().service_attempts.is_empty(),
            "layout invalidation makes routing stale"
        );

        let mut probe = three_level_route_probe();
        probe.service_relayout = true;
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [ROUTE_PARENT],
            "target service layout invalidation stops before root"
        );

        let mut probe = two_level_route_probe(false);
        probe.focus_child_on_event = true;
        probe.focusable_controls = true;
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(runtime.focus(), Some(ROUTE_CHILD));
        assert!(
            runtime.app().service_attempts.is_empty(),
            "a focus transition stops fallback"
        );

        let mut probe = two_level_route_probe(false);
        probe.modal_on_bootstrap = true;
        let mut runtime = ready_routed_runtime(probe);
        assert!(runtime.is_open(ROUTE_MODAL));
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(runtime.app().wheel_deliveries.len(), 1);
        assert_eq!(runtime.app().wheel_deliveries[0].0, ROUTE_CHILD);
        assert!(
            runtime.app().service_attempts.is_empty(),
            "a top modal route does not offer a page target"
        );

        let mut probe = three_level_route_probe();
        probe.focusable_controls = true;
        probe.capture_on_wheel = true;
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(
            &mut runtime,
            Input::Mouse(Mouse {
                kind: MouseKind::Down,
                pos: ROUTE_POINT,
                mods: KeyModifiers::NONE,
            }),
        );
        assert_eq!(runtime.capture_owner(), None);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            runtime
                .app()
                .service_attempts
                .iter()
                .map(|attempt| attempt.owner())
                .collect::<Vec<_>>(),
            [],
            "capture ownership changes invalidate the remaining route"
        );
        assert_eq!(runtime.capture_owner(), Some(ROUTE_CHILD));
    }

    #[test]
    fn plain_scroll_registration_keeps_single_target_wheel_behavior() {
        let mut probe = two_level_route_probe(false);
        probe.route_enabled = false;
        let mut runtime = ready_routed_runtime(probe);
        let _ = deliver(&mut runtime, route_wheel_input(Axis::V, 1));
        assert_eq!(
            runtime
                .app()
                .wheel_deliveries
                .iter()
                .map(|delivery| delivery.0)
                .collect::<Vec<_>>(),
            [ROUTE_CHILD]
        );
        assert!(runtime.app().initial_attempts.is_empty());
        assert!(runtime.app().service_attempts.is_empty());
    }
}
