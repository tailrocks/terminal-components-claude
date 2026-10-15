//! The runtime (`COMPONENT_ARCHITECTURE.md` §3.3, §8, §9, §17.0 A1).
//!
//! `Runtime<A>` owns all interaction state — focus, hover, press, flash,
//! capture, layers, cursor and regions. `initialize`, `handle` and `settle`
//! run updates without a buffer. `draw` paints an exclusively borrowed candidate;
//! only `PaintedFrame::commit_presented` publishes its routing geometry after
//! successful output. Painting and publication never call application updates.

pub(crate) mod feedback;
#[cfg(feature = "crossterm")]
pub(crate) mod session;
mod time;
pub use feedback::{ActivationFeedback, FeedbackClock, FeedbackClockError, SimulationMoment};
pub(crate) mod typing;
pub use time::{ClockError, Moment};
pub use typing::TypingPolicy;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::terminal::Frame;

use crate::action::ActionKey;
use crate::capture::Capture;
use crate::cursor;
use crate::diagnostics::Diagnostic;
use crate::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use crate::focus::{FocusRing, FocusState, ScopeId};
use crate::hit::Hit;
use crate::id::{Id, Part, PartRef};
use crate::intent::{FocusVia, IntentQueue, Phase};
use crate::keymap::{BindingTableId, KeyMap, KeyPhase};
use crate::layer::{
    Backdrop, DismissReason, LayerEvent, LayerId, OpenLayer, backdrop_area, resolve_anchor,
};
use crate::measure::Size;
#[cfg(any(test, feature = "testing"))]
use crate::response::StateFlags;
use crate::response::{Invalidate, Response};
use crate::theme::Theme;
use crate::ui::cx::{FrameServices, LastFrame};
use crate::ui::{Cx, FrameState, Ui, UiCore};

/// The application entry point.
pub trait App {
    /// The only input entry point: intents are already resolved.
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()>;

    /// Pure paint.
    fn draw(&self, ui: &mut Ui<'_>);

    /// Whether the runtime loop should exit.
    fn should_quit(&self) -> bool {
        false
    }

    /// The application chord layer.
    fn keymap(&self) -> &KeyMap {
        KeyMap::EMPTY_REF
    }

    /// The smallest and preferred terminal size.
    fn min_size(&self) -> Size {
        Size {
            min: (72, 20),
            preferred: (120, 40),
        }
    }

    /// The product Esc ladder (§3.3 step 8c): called when Esc was consumed
    /// by no component and dismissed no layer.
    fn on_esc(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
}

/// Narrow physical key origin for an admitted update; never an activation grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActivationKey {
    /// An unmodified Enter press or repeat.
    Enter,
    /// An unmodified Space press or repeat.
    Space,
}

/// Why the runtime is invoking [`App::update`].
///
/// The cause is scoped to one update pass. A focus-settling rerun is always
/// [`UpdateCause::Settle`], so a single physical timer delivery cannot advance
/// application time more than once.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateCause {
    /// The first update, before the first draw or externally handled event.
    Bootstrap,
    /// An externally supplied key, mouse, paste or resize event.
    Event,
    /// A timer/deadline delivery.
    Tick,
    /// A rerun used to settle a focus transition staged by an earlier pass.
    Settle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Press {
    owner: Id,
    part: PartRef,
    area: Rect,
    dragging: bool,
    /// Where the pointer was when the button went down — `Cx::capture`'s
    /// `origin`, so `pos - origin` is the offset within the thumb (MA-5).
    pos: Position,
}

/// Runtime-owned interaction bookkeeping (§1.2(4), §8.6).
#[derive(Clone, Copy, Debug, Default)]
struct Interaction {
    pointer_position: Option<Position>,
    hover: Option<(Id, PartRef)>,
    hover_suppressed: bool,
    hover_passes: u8,
    press: Option<Press>,
    last_click: Option<(Id, PartRef, Moment)>,
    last_input_key: bool,
}

/// Passes of `app.update` before the runtime gives up settling focus.
const MAX_FOCUS_PASSES: usize = 4;

/// Immutable read facts for deterministic, effect-free production-view projection.
/// The default is an explicitly empty interaction snapshot, suitable for fixtures.
#[derive(Clone, Default)]
pub struct RenderSnapshot {
    cache_identity: std::sync::Arc<()>,
    last: LastFrame,
    layers: Vec<OpenLayer>,
    inert_floor: LayerId,
    top: LayerId,
    keymap: KeyMap,
}

/// An immutable model and painter bound to one projection cache epoch.
/// Ordinary model mutation is excluded for the binding's lifetime. Derived
/// caches may therefore be reused across captures of this same binding.
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
    /// Bind an immutable model and painter to a fresh derived-cache epoch.
    /// Rebinding, including a replacement at the same address, always isolates
    /// model caches. Interior mutation must obey the same view contract as App.
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

/// Why a runtime cannot provide a compatible, settled render snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderSnapshotError {
    /// Explicit initialization has not completed.
    Uninitialized,
    /// A committed focus transition still requires an update.
    NeedsSettle,
    /// Model, theme or interaction changes require successful presentation.
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

/// A completed projection candidate. Its acknowledgment updates inspection
/// outputs only; it never publishes live input compatibility or focus callbacks.
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
    /// Record a completed buffer's geometry and cursor for inspection.
    pub fn commit_inspected(self) {
        let output = self
            .runtime
            .projection
            .get_or_insert_with(ProjectionFacts::default);
        let frame = &mut self.runtime.frame;
        core::mem::swap(&mut output.last.registry, &mut frame.registry);
        core::mem::swap(&mut output.last.ring, &mut frame.ring);
        core::mem::swap(&mut output.last.bindings, &mut frame.bindings);
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
        self.runtime.presented = false;
    }
}

/// An input retained until initialization, presentation and focus settling finish.
/// Debug deliberately omits the event, including pasted text.
pub struct PendingInput(Input);

impl PendingInput {
    /// Recover the original owned event for retry.
    pub fn into_input(self) -> Input {
        self.0
    }
}

impl core::fmt::Debug for PendingInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PendingInput { event: <redacted> }")
    }
}

/// A painted candidate, exclusively borrowing its originating runtime.
/// Commit only after output succeeds. Dropping it aborts publication.
///
/// A pending frame prevents mutation of the runtime it belongs to:
/// ```compile_fail,E0499
/// use termrock_runtime::{App, Runtime};
/// use ratatui_core::terminal::Frame;
/// fn stale<A: App>(rt: &mut Runtime<A>, buf: &mut Frame<'_>) {
///     let frame = rt.draw(buf);
///     let _ = rt.app_mut();
///     frame.commit_presented();
/// }
/// ```
/// Publication cannot be redirected to a different runtime:
/// ```compile_fail,E0061
/// use termrock_runtime::{App, Runtime};
/// use ratatui_core::terminal::Frame;
/// fn wrong<A: App>(one: &mut Runtime<A>, two: &mut Runtime<A>, buf: &mut Frame<'_>) {
///     let frame = one.draw(buf);
///     frame.commit_presented(two);
/// }
/// ```
/// A successful acknowledgment consumes the frame:
/// ```compile_fail,E0382
/// use termrock_runtime::{App, Runtime};
/// use ratatui_core::terminal::Frame;
/// fn twice<A: App>(rt: &mut Runtime<A>, buf: &mut Frame<'_>) {
///     let frame = rt.draw(buf);
///     frame.commit_presented();
///     frame.commit_presented();
/// }
/// ```
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
    /// Publish this candidate after its buffer and cursor were presented.
    pub fn commit_presented(mut self) {
        self.runtime.commit_frame();
        self.committed = true;
    }
}

impl<A: App> Drop for PaintedFrame<'_, A> {
    fn drop(&mut self) {
        if !self.committed {
            self.runtime.presented = false;
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

/// The runtime.
pub struct Runtime<A: App> {
    app: A,
    theme: Theme,
    last: LastFrame,
    focus: FocusState,
    services: FrameServices,
    intents: IntentQueue,
    inter: Interaction,
    pending_tick: bool,
    frame: FrameState,
    core: UiCore,
    generation: u32,
    cursor: Option<Position>,
    last_invalidate: Invalidate,
    pending_focus: Option<(Option<Id>, Option<Id>, FocusVia)>,
    restore_after_publication: Option<FocusRestore>,
    keymap_conflict_key: Option<ConflictCacheKey>,
    keymap_conflict_typing: Vec<crate::keymap::BindingDescriptor>,
    keymap_conflicts: Vec<Diagnostic>,
    staged_focus: Option<(Option<Id>, FocusVia)>,
    layer_events_pending: Vec<(Id, LayerEvent)>,
    /// Layers closed by `DismissReason::FocusOut` since the last settle
    /// (§29.8 D2). Drained in lockstep with `services.closed_layers`.
    focus_out_closed: Vec<Id>,
    unsettled: usize,
    bootstrapped: bool,
    presented: bool,
    painted_cursor: Option<Position>,
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
    /// A runtime for `app` under `theme`.
    pub fn new(app: A, theme: Theme) -> Self {
        Self::new_with_feedback_clock(app, theme, FeedbackClock::Elapsed)
    }

    /// Construct with an immutable feedback clock, before bootstrap or activation.
    /// Simulation origins must match the already-seeked domain fixture.
    pub fn new_with_feedback_clock(app: A, theme: Theme, clock: FeedbackClock) -> Self {
        let mut core = UiCore::default();
        core.keymap.clone_from(app.keymap());
        Runtime {
            app,
            theme,
            last: LastFrame::default(),
            focus: FocusState::default(),
            services: FrameServices {
                feedback: feedback::FeedbackState::new(clock),
                ..FrameServices::default()
            },
            intents: IntentQueue::new(),
            inter: Interaction::default(),
            pending_tick: false,
            frame: FrameState::default(),
            core,
            generation: 0,
            cursor: None,
            last_invalidate: Invalidate::None,
            pending_focus: None,
            restore_after_publication: None,
            keymap_conflict_key: None,
            keymap_conflict_typing: Vec::new(),
            keymap_conflicts: Vec::new(),
            staged_focus: None,
            layer_events_pending: Vec::new(),
            focus_out_closed: Vec::new(),
            unsettled: 0,
            bootstrapped: false,
            presented: false,
            painted_cursor: None,
            cache_snapshot: None,
            #[cfg(feature = "testing")]
            projection: None,
        }
    }

    /// The application.
    pub const fn app(&self) -> &A {
        &self.app
    }

    /// The application, mutably.
    pub const fn app_mut(&mut self) -> &mut A {
        self.presented = false;
        &mut self.app
    }

    /// Effective typing owner from compatible, successfully presented geometry.
    /// Returns `None` while initialization, settling or publication is required.
    pub fn typing_owner(&self) -> Option<Id> {
        (self.bootstrapped && self.presented && !self.needs_settle())
            .then_some(self.last.typing.owner)
            .flatten()
    }

    /// The theme.
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Replace the theme; derived caches are dropped.
    pub fn set_theme(&mut self, t: Theme) {
        self.presented = false;
        self.theme = t;
        self.core.clear_caches();
    }

    /// Last successfully presented area of a control (`None` before publication).
    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.last.registry.area_of(id)
    }

    /// Last successfully presented area of a component's sub-region.
    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect> {
        self.last.registry.area_of_part(id, p)
    }

    /// Last successfully presented focus ring.
    pub const fn ring(&self) -> &FocusRing {
        &self.last.ring
    }

    /// The focused control.
    pub const fn focus(&self) -> Option<Id> {
        self.focus.current()
    }

    /// Diagnostics recorded since the last `handle`.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        self.services.diagnostics.items()
    }

    /// Diagnostics dropped beyond the retained 64.
    pub fn diagnostics_dropped(&self) -> usize {
        self.services.diagnostics.dropped()
    }

    /// Whether `Cx::quit` was called.
    pub const fn quit_requested(&self) -> bool {
        self.services.quit
    }

    /// Whether immediate repaint or timed work is pending.
    /// Drivers schedule `next_deadline`; this does not imply a tick cadence.
    pub fn wants_tick(&self) -> bool {
        self.needs_present() || self.needs_settle() || self.next_deadline().is_some()
    }

    /// Earliest absolute application or runtime-feedback deadline.
    pub fn next_deadline(&self) -> Option<Moment> {
        if self.pending_tick {
            return Some(self.now());
        }
        match (
            self.services.repaint_at,
            self.services.feedback.elapsed_deadline(),
        ) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// The one current activation-feedback record, independent of held press/capture.
    pub fn activation_feedback(&self) -> Option<ActivationFeedback> {
        self.services.feedback.active(self.now())
    }

    /// Explicit monotonic elapsed time, independent of input count.
    pub const fn now(&self) -> Moment {
        self.services.now
    }

    /// Advance explicit elapsed time and stage at most one due update.
    /// This never calls `App::update`. Settle and publish before retrying input.
    /// Equal time may stage work armed since the previous scheduler turn.
    /// A due callback's immediate rearm waits for the next `advance_to` call.
    ///
    /// # Errors
    /// Backwards time is rejected without changing any runtime state.
    pub fn advance_to(&mut self, now: Moment) -> Result<(), ClockError> {
        if now < self.now() {
            return Err(ClockError {
                current: self.now(),
                requested: now,
            });
        }
        self.services.now = now;
        if !self.pending_tick && self.services.repaint_at.is_some_and(|at| at <= now) {
            self.pending_tick = true;
            self.services.repaint_at = None;
        }
        if self.services.feedback.expire_elapsed(now) {
            if self.inter.press.is_none() {
                self.last.snapshot.pressed = None;
            }
            self.presented = false;
            self.services.repaint = true;
        }
        Ok(())
    }

    /// Explicit elapsed milliseconds, truncated only for this legacy inspector.
    pub fn clock_ms(&self) -> u64 {
        u64::try_from(self.now().as_duration().as_millis()).unwrap_or(u64::MAX)
    }

    /// The terminal size as last resized or successfully presented.
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
            // Caches retain only the immediately previous equality key; wrapping
            // changes that key while explicit invalidation clears both consumers.
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
        let typing_table = typing.map_or(&[][..], |(_, table)| table);
        let key = ConflictCacheKey {
            focused: component.map(|(owner, table, _)| (owner, table)),
            typing_owner: typing.map(|(owner, _)| owner),
            revision: self.core.keymap_revision,
        };
        // A captured filter can change the accepted subset without changing
        // its static source table. Exact descriptors are the cache identity;
        // comparing only BindingTableId would retain stale conflict results.
        if self.keymap_conflict_key == Some(key) && self.keymap_conflict_typing == typing_table {
            return;
        }
        self.keymap_conflict_key = Some(key);
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

    /// A newly opened top layer can reparent an existing owner. Until its first
    /// successful publication, the prior ring cannot prove that owner's scope.
    fn provisional_focus_layer(&self, to: Id) -> Option<OpenLayer> {
        let layer = self.services.initial_focus_layer?;
        self.services
            .layers
            .top_layer()
            .copied()
            .filter(|top| top.spec.initial_focus == Some(to) && top.layer == layer)
    }

    /// Whether `to` may be staged as a focus target.
    ///
    /// Immediate transition producers — `Tab` traversal, a press,
    /// `Cx::focus` and `LayerSpec::initial_focus` — funnel through
    /// [`Self::stage_focus`], which judges immediate focus admission.
    /// A newly opened layer's initial focus is provisional until its first
    /// publication: that owner may be moving from a prior scope. Otherwise a
    /// target is refused when the last frame proves it cannot hold focus:
    ///
    /// * the ring holds it as `Disabled`, so it is registered and never
    ///   reachable;
    /// * the ring holds it outside the armed trap, which is what a trap
    ///   means;
    /// * the ring holds it on a layer the live stack has since made inert —
    ///   the frame in which a modal opens over a control that is still in
    ///   last frame's ring;
    /// * it registered a region last frame but took no ring entry at all,
    ///   which is `Focusability::ClickOnly` and a decoration-only owner.
    ///
    /// An id the last frame never saw is **unknown**, not proven bad, and is
    /// admitted. A layer's own controls are absent from the ring until they
    /// first draw, so `LayerSpec::initial_focus` names an unknown id by
    /// construction. Historical layer restoration is instead validated only
    /// against successful publication. Step 14 reconciliation settles fresh
    /// targets against the ring the next frame actually produces.
    ///
    /// A refusal is silent: `Diagnostic` has no variant for a rejected focus
    /// target and adding one is a §17.0 A9 amendment, not an implementation
    /// choice.
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

    /// Apply a staged transition: enqueue `FocusOut` then `FocusIn`.
    fn apply_staged_focus(&mut self) -> bool {
        let Some((to, via)) = self.staged_focus.take() else {
            return false;
        };
        let from = self.focus.current();
        if from == to {
            return false;
        }
        if let Some(old) = from {
            self.intents.focus_out(old, to);
        }
        if let Some(new) = to {
            self.intents.focus_in(new, via);
        }
        self.focus.set(to);
        self.last.snapshot.focus = to;
        // Focus is visible state. Terminal sessions redraw only when the
        // response asks for paint; without this request a rapid Tab+Enter
        // sequence can deliver Enter against the previous frame's binding
        // table, so the newly focused control never sees its activation.
        self.services.repaint = true;

        // Detaching a closed owner is not navigation out of surviving layers.
        // The restoration target has no focus authority until publication.
        if via != FocusVia::Restore || self.restore_after_publication.is_none() {
            self.dismiss_on_focus_out(to);
        }
        true
    }

    /// Whether `to` is *known* to sit outside `scope`.
    ///
    /// An unregistered target is deliberately not "outside": a layer's own
    /// controls are absent from the last frame's ring until they first draw,
    /// so a `LayerSpec::initial_focus` into a freshly opened layer would
    /// otherwise read as a focus-out and dismiss the layer that asked for
    /// it. Focus at `None` is outside every scope.
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

    /// §29.8 D2: close every top layer that dismisses on focus-out and whose
    /// scope no longer contains the new focus target.
    ///
    /// This is the producer `DismissReason::FocusOut` never had, and it is
    /// what keeps focus from resting on a control *behind* an open popover:
    /// `Tab` is runtime focus policy, intercepted before any intent is
    /// enqueued, so the component itself can never see it.
    ///
    /// The walk stops at the first layer that keeps the target or refuses
    /// focus-out dismissal. A `Modal` refuses through
    /// `LayerSpec::dismisses_on_focus_out` itself, which is the only
    /// exclusion there is; and because `LayerStack::close` takes everything
    /// above a layer with it, stopping there is also what stops a popover
    /// underneath a modal being closed out from under a modal that
    /// legitimately holds focus.
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

    /// Step 1 for a resize.
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
        self.inter.hover = None;
        self.last.snapshot.pressed = None;
        self.last.snapshot.hover = None;
        self.last.snapshot.capture = None;
        self.core.clear_caches();
        self.last_invalidate = Invalidate::Layout;
    }

    /// Steps 3–6 for a key.
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
        // An explicitly published Tab/BackTab command belongs to the focused
        // component. Only an unbound Tab reaches runtime focus traversal.
        if let Some(owner) = current
            && let Some((_, table)) = self.last.bindings.get(owner)
            && let Some((action, chord)) = self.core.keymap.component_binding(owner, table, &k)
        {
            self.intents.binding(owner, action, chord);
            return;
        }
        // Tab / Shift+Tab are runtime focus policy (step 5), against the last ring.
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

    /// Whether last frame's ring **proves** `id` cannot be interacted with.
    ///
    /// The ring is the only record of a registered-but-unreachable control:
    /// `Focusability::Disabled` takes an entry with `disabled: true`, so an
    /// id the ring never saw — an unknown id, or the `Focusability::ClickOnly`
    /// hit target that takes no entry at all — is not proven anything here.
    /// This is the single authority both [`Self::focus_target_admissible`] and
    /// [`Self::deliverable`] read, so keyboard reachability and pointer
    /// activation can no longer disagree about the same control (§73).
    fn ring_proves_disabled(&self, id: Id) -> bool {
        self.last.ring.entry(id).is_some_and(|e| e.disabled)
    }

    /// Whether a hit may receive an **activating** pointer intent — `Press`
    /// and the `Release`/`Click`/`DoubleClick`/`DragStart`/`Drag`/`DragEnd`
    /// sequence that a press owns, and `Secondary`.
    ///
    /// §73. A disabled control is registered, hit-testable and hoverable, and
    /// it is exactly as unreachable by pointer as it already was by `Tab`:
    /// keyboard refused it in [`Self::focus_target_admissible`] while a click
    /// still delivered `Press`/`Click` to it, so a disabled `Button` could be
    /// activated with the mouse. The refusal is scoped to activation:
    ///
    /// * `Phase::Move` and hover come from `hit_live` and are untouched — a
    ///   disabled control still reports hover and still paints it;
    /// * `Intent::Wheel` comes from `hit_scroll` and is untouched — a disabled
    ///   control that owns a scroll region still scrolls;
    /// * a `ClickOnly` owner takes no ring entry, so it is never proven
    ///   disabled and keeps delivering;
    /// * a hit below the top layer is still `outside`, so outside-click
    ///   dismissal is decided before this predicate is ever consulted.
    ///
    /// The refusal is silent, like the focus refusal it now matches.
    fn deliverable(&self, hit: Hit) -> bool {
        crate::capture::target_eligible(&self.last.ring, self.top(), hit.owner, hit.layer, hit.kind)
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

    /// Steps 3–6 for a pointer event.
    fn enqueue_mouse(&mut self, m: Mouse) {
        self.inter.pointer_position = Some(m.pos);
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
        // a live capture short-circuits hit-testing (§8.2)
        if let Some(cap) = self.services.capture.get() {
            self.pointer_captured(cap, m);
            return;
        }
        let hit = self.last.registry.hit(m.pos);
        let top = self.top();
        let outside = hit.is_none_or(|h| h.layer < top);
        match m.kind {
            MouseKind::Move => self.pointer_moved(m, top, hover_was_suppressed),
            MouseKind::Down => {
                if outside {
                    self.outside_click();
                    return;
                }
                if let Some(h) = hit {
                    self.pointer_down(h, m);
                }
            }
            MouseKind::Drag => {
                let Some(p) = self.inter.press else { return };
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
                    return;
                }
                let Some(h) = hit else { return };
                if !self.deliverable(h) {
                    return;
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
                // a wheel over the page below a popover must not scroll the
                // page: the same top-layer filter `deliverable` applies (MI-5)
                if let Some(h) = self.last.registry.hit_scroll(m.pos, axis)
                    && h.layer == top
                {
                    let rows = self.theme.design.motion.wheel_rows as i16;
                    self.intents
                        .wheel(h.owner, axis, delta.saturating_mul(rows), h.part, m.pos);
                }
            }
        }
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
        if let Some(h) = live {
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

    /// Primary button down on a deliverable hit: press bookkeeping, press-focuses-owner.
    fn pointer_down(&mut self, h: Hit, m: Mouse) {
        if !self.deliverable(h) {
            return;
        }
        let area = self.region_area(h);
        self.inter.press = Some(Press {
            owner: h.owner,
            part: h.part,
            area,
            dragging: false,
            pos: m.pos,
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

    /// Primary button up after a press: release, then click / double-click
    /// on the same target, or drag end.
    fn pointer_up(&mut self, p: Press, hit: Option<Hit>, m: Mouse) {
        self.last.snapshot.pressed = None;
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
        let window = self.theme.design.motion.double_click_ms;
        let double = self.inter.last_click.is_some_and(|(o, part, at)| {
            o == p.owner
                && part == p.part
                && self.now().saturating_duration_since(at)
                    <= core::time::Duration::from_millis(window)
        });
        let phase = if double {
            Phase::DoubleClick
        } else {
            Phase::Click
        };
        self.inter.last_click = if double {
            None
        } else {
            Some((p.owner, p.part, self.now()))
        };
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
                self.intents
                    .pointer(cap.owner, Phase::Drag, cap.part, m.pos, local, m.mods);
            }
            MouseKind::Up => {
                self.intents
                    .pointer(cap.owner, Phase::Release, cap.part, m.pos, local, m.mods);
                self.intents
                    .pointer(cap.owner, Phase::DragEnd, cap.part, m.pos, local, m.mods);
                if cap.contains(m.pos) {
                    self.intents
                        .pointer(cap.owner, Phase::Click, cap.part, m.pos, local, m.mods);
                }
                self.services.capture.release();
                self.inter.press = None;
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

    /// A click whose hit is below the top layer, or nowhere.
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
        let closed = self
            .services
            .layers
            .close(top.id, LayerEvent::Dismissed(reason));
        self.services.closed_layers.extend(closed);
        // §6.1: `Cancel` is "Esc reached this owner after layer dismissal".
        // An outside click or a programmatic close is not a cancellation.
        if reason == DismissReason::Esc {
            self.intents.cancel(top.spec.owner);
        }
        true
    }

    /// Handle closed layers staged by `close_layer` / dismissal: stage the
    /// focus restore and release captures whose layer vanished (F8).
    fn settle_closed_layers(&mut self) {
        let closed: Vec<OpenLayer> = core::mem::take(&mut self.services.closed_layers);
        if closed.is_empty() {
            return;
        }
        let focus_out = core::mem::take(&mut self.focus_out_closed);
        for l in &closed {
            if let Some(c) = self.services.capture.get()
                && self.last.registry.layer_of(c.owner) == Some(l.layer)
            {
                self.services.capture.release();
                self.last.snapshot.capture = None;
            }
        }
        // The lowest closed layer's restore target wins — except on the
        // focus-out path (§29.8 D2). `LayerSpec::popover` sets
        // `restore_focus`, so restoring here would put focus straight back
        // on the opener, making the `Tab` that dismissed the layer a no-op
        // and re-creating the legacy key swallow by accident.
        if let Some(first) = closed.first()
            && first.spec.restore_focus
            && !focus_out.contains(&first.id)
        {
            let target = first
                .restore_to
                .or_else(|| self.focus.take_restore(first.scope()));
            self.restore_after_publication = Some(FocusRestore { target });
            // Deliver the existing owner's FocusOut during close settlement,
            // but never send FocusIn to an unvalidated historical opener.
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

    /// Move staged layer lifecycle events into the queue (as `Intent::Layer`
    /// to the layer's owner) and into the `Cx::layer_event` list.
    fn pump_layer_events(&mut self) {
        for (id, ev) in self.services.layers.take_pending() {
            self.layer_events_pending.push((id, ev));
            let owner = self.services.layers.get(id).map_or(id, |l| l.spec.owner);
            self.intents.layer(owner, ev);
            if owner != id {
                self.intents.layer(id, ev);
            }
        }
    }

    /// Step 7: `app.update` with the frozen queue, re-run while focus moves.
    fn run_update(
        &mut self,
        command: Option<ActionKey>,
        cause: UpdateCause,
        activation_key: Option<ActivationKey>,
    ) -> Response<()> {
        self.presented = false;
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
            // a focus-out dismissal fired inside `apply_staged_focus` staged
            // its `Dismissed(FocusOut)` after the pump above; deliver it in
            // this same pass rather than a frame later (§29.8 D2)
            self.pump_layer_events();
            self.services.focus_request = None;
            let events = core::mem::take(&mut self.layer_events_pending);
            self.services.events.extend(events);
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
                .with_activation_key(activation_key);
                self.app.update(&mut cx)
            };
            folded |= r;
            // Undelivered intents are diagnosed per pass, before buckets are
            // cleared. A bucket the RUNTIME addressed (`Layer`, `Cancel`,
            // `FocusIn`/`FocusOut`) is diagnosed whatever the owner
            // registered: pointer intents already cannot reach a `Decorative`
            // owner (`deliverable`), so §21 item 13's escape for container
            // regions does not apply to them, and a layer owner that
            // registers only decor — every `Dialog` — would otherwise lose
            // its own dismissal in silence.
            for owner in self.intents.undrained() {
                if self.last.registry.delivers_to(owner)
                    || self.intents.has_runtime_addressed(owner)
                {
                    self.services
                        .diagnostics
                        .push(Diagnostic::UndeliveredIntent { owner });
                }
            }
            // unread layer events are kept for the next handle
            self.layer_events_pending = core::mem::take(&mut self.services.events);
            // remember the opener of every newly opened layer (§8.1 restore map)
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
                    // §21 item 11: the pair is *applied*. `intents.clear()`
                    // below would swallow it, so it is delivered by the next
                    // `handle` through `pending_focus` (MI-7).
                    self.pending_focus = Some((from, target, via));
                }
                self.intents.clear();
                break;
            }
        }
        self.unsettled = 0;
        folded
    }

    /// Run the application's bootstrap update exactly once.
    ///
    /// Live drivers call this before their first frame so initialization can
    /// establish focus, layers and repaint deadlines. It never advances time.
    /// Repeated calls return an ignored response without updating the app.
    ///
    /// Painting never calls this method: rendering a supplied model, including
    /// its very first frame, cannot run application initialization or effects.
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

    /// Whether an update must settle focus discovered by a committed frame.
    pub const fn needs_settle(&self) -> bool {
        self.pending_focus.is_some() || (self.bootstrapped && self.pending_tick)
    }

    /// Whether another successful presentation is required before input.
    pub const fn needs_present(&self) -> bool {
        !self.presented
    }

    /// Deliver committed focus transitions without consuming external input.
    pub fn settle(&mut self) -> Response<()> {
        if !self.bootstrapped {
            return Response::ignored();
        }
        let Some((from, to, via)) = self.pending_focus.take() else {
            if !core::mem::take(&mut self.pending_tick) {
                return Response::ignored();
            }
            self.intents.clear();
            self.pump_layer_events();
            let r = self.run_update(None, UpdateCause::Tick, None);
            return self.finish(r);
        };
        self.intents.clear();
        if let Some(old) = from {
            self.intents.focus_out(old, to);
        }
        if let Some(new) = to {
            self.intents.focus_in(new, via);
        }
        self.dismiss_on_focus_out(to);
        let r = self.run_update(None, UpdateCause::Settle, None);
        self.finish(r)
    }

    /// Route one event using compatible, successfully presented geometry.
    ///
    /// # Errors
    /// Returns the original event without effects if initialization, presentation
    /// or explicit focus settling is still required. Retry after completing it.
    pub fn handle(&mut self, input: Input) -> Result<Response<()>, PendingInput> {
        if !self.bootstrapped || !self.presented || self.needs_settle() {
            return Err(PendingInput(input));
        }
        let activation_key = match &input {
            Input::Key(key) if key.mods.is_empty() => match key.code {
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
        self.services.registry_gen = self.last.registry.generation();
        self.pump_layer_events();
        let mut key_input = None;
        let mut update_cause = UpdateCause::Event;
        match &input {
            Input::Resize(w, h) => {
                self.resize(*w, *h);
                // step 1 then the ordinary update pass: the `FocusOut`/
                // `FocusIn` pair staged for a `pending_focus` is already in
                // the queue and must reach `app.update` before `finish`
                // clears it (MA-7).
                let r = self.run_update(None, UpdateCause::Event, activation_key)
                    | Response::changed().relayout();
                drop(input);
                return Ok(self.finish(r));
            }
            Input::Tick => {
                update_cause = UpdateCause::Tick;
                // Explicit ticks request an update at unchanged time. Only
                // advance_to consumes an elapsed absolute deadline.
            }
            Input::Key(k) => {
                // step 2: capture chords first
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
                    return Ok(self.finish(r));
                }
                key_input = Some(*k);
                self.enqueue_key(*k);
            }
            Input::Mouse(m) => self.enqueue_mouse(*m),
            Input::Paste(s) => {
                if let Some(owner) = self.last.typing.owner {
                    self.intents.paste(owner, s.as_str());
                }
            }
        }
        let mut r = self.run_update(None, update_cause, activation_key);
        // step 8: bubble
        if let Some(k) = key_input
            && !r.is_consumed()
        {
            // The raw pass already delivered (or diagnosed) every intent in
            // the frozen queue. A follow-up bubble or Esc-dismissal pass must
            // observe only its own command and fresh layer/focus
            // notifications — never the same physical Key twice — while
            // per-pass iteration stays immutable.
            self.intents.clear();
            if let Some(cmd) = self.core.keymap.lookup(KeyPhase::Bubble, &k, false) {
                r |= self.run_update(Some(cmd), UpdateCause::Event, activation_key);
            } else if k.code == KeyCode::Esc {
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
                    let esc = {
                        let mut cx = Cx::new_with_cause(
                            &self.intents,
                            &mut self.services,
                            &mut self.core,
                            &self.last,
                            &self.theme,
                            None,
                            UpdateCause::Event,
                        );
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
        Ok(self.finish(r))
    }

    /// Step 9.
    fn finish(&mut self, mut r: Response<()>) -> Response<()> {
        self.reconcile_held_pointer();
        if self.services.repaint {
            r = r.repaint();
        }
        self.intents.clear();
        self.last_invalidate = r.invalidate();
        r
    }

    /// Paint a candidate into a terminal frame without publishing input geometry.
    /// Keep the returned guard until terminal output succeeds, then call
    /// [`PaintedFrame::commit_presented`]. Dropping it requires another frame.
    pub fn draw(&mut self, frame: &mut Frame<'_>) -> PaintedFrame<'_, A> {
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
        self.presented = false;
        self.sync_keymap(snapshot.map(|value| &value.keymap));
        // Cache storage follows the painted size and immutable snapshot identity,
        // not live screen publication. A projection must neither clear every warm
        // frame nor lend derived data to a different snapshot or live model.
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
        // step 10: new frame state
        self.generation = self.generation.wrapping_add(1);
        self.core.begin_cache_frame(self.generation);
        self.core.style_cache.clear();
        self.frame.reset(self.generation, area);
        self.frame.inert_floor = snapshot.map_or_else(
            || self.services.layers.inert_floor(),
            |value| value.inert_floor,
        );
        self.frame.top = snapshot.map_or_else(|| self.services.layers.top(), |value| value.top);
        // Prepare every open layer's draw target. Its focus scope is armed
        // only when `Ui::layer` performs a live draw.
        let layers = snapshot.map_or_else(
            || self.services.layers.layers(),
            |value| value.layers.as_slice(),
        );
        for l in layers {
            let rect = resolve_anchor(area, l.spec.anchor, l.spec.size);
            self.frame.layers.push(l.id, l.layer, l.spec, rect, area);
        }
        // step 11: app.draw into the page, layers into pooled buffers
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
            // step 12: composite bottom-to-top
            for i in 0..ui_layer_count(&ui) {
                composite_layer(&mut ui, i, area);
            }
        }
        let focus =
            snapshot.map_or_else(|| self.focus.current(), |value| value.last.snapshot.focus);
        self.frame.typing_resolved = typing::resolve(&mut self.frame, focus);
        self.painted_cursor = cursor::resolve_requests(
            &self.frame.cursors,
            self.frame.top,
            self.frame.typing_resolved.cursor,
            &mut self.frame.diagnostics,
            &self.frame.typing,
        );
    }

    fn commit_geometry(&mut self) {
        self.services.viewport = self.frame.screen;
        // Step 13: registry swap; held targets reconcile after the ring swap.
        let mut diags = core::mem::take(&mut self.frame.diagnostics);
        self.services.diagnostics.extend(diags.drain(..));
        core::mem::swap(&mut self.last.registry, &mut self.frame.registry);
        self.last.layout.clear();
        self.last.layout.append(&mut self.frame.layout);
        self.last.declared.clear();
        self.last.declared.append(&mut self.frame.declared);
        core::mem::swap(&mut self.last.bindings, &mut self.frame.bindings);
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
        // step 14: focus reconcile. A modal's trap moves focus into it by
        // rule (c); a popover leaves focus where it is unless the spec named
        // an `initial_focus` (§16.2 case 17: a component stays focused under
        // a popover and its cursor write is rejected)
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
            // An ambiguous duplicate ID can make the first reachable entry
            // inadmissible. Continue in registration order to a valid owner.
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
            // No update has observed the intermediate target yet. Preserve the
            // original source when several presentations precede explicit settle.
            let from = self.pending_focus.map_or(previous, |(from, _, _)| from);
            self.pending_focus = (from != reconciled).then_some((from, reconciled, via));
            self.focus.set(reconciled);
            // This output used the old focus. Even a coalesced round trip with
            // no callback needs another presentation before input is compatible.
            self.presented = false;
            self.services.repaint = true;
        }
        self.staged_focus = None;
        self.last.snapshot.focus = self.focus.current();
        self.last.snapshot.focus_visible = self.focus.visible();
        self.reconcile_held_pointer();
        self.reconcile_hover();
        // Cursor matches the actual output, painted before focus reconciliation.
        self.cursor = self.painted_cursor;
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
        }
        if capture_invalid || press_invalid {
            self.inter.press = None;
            self.services.press_pos = None;
        }
        // Gesture cancellation never erases independently timed feedback.
        let pressed = self
            .inter
            .press
            .map(|press| (press.owner, press.part))
            .or_else(|| self.services.feedback.pressed());
        let capture = self.services.capture.get().map(|capture| capture.owner);
        if self.last.snapshot.capture != capture || self.last.snapshot.pressed != pressed {
            self.presented = false;
            self.services.repaint = true;
        }
        self.last.snapshot.capture = capture;
        self.last.snapshot.pressed = pressed;
    }

    fn reconcile_hover(&mut self) {
        // Only successful live publication can change hover from new geometry.
        // Repaint uses these facts; no synthetic Move or application update runs.
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
                // An immutable view may move away whenever it is hovered, so
                // no fixed point exists. Publish an explicitly suppressed
                // hover state rather than starving retained input forever.
                self.inter.hover_suppressed = true;
                self.last.snapshot.hover_suppressed = true;
                hover = None;
                self.services
                    .diagnostics
                    .push(Diagnostic::HoverLayoutDidNotSettle);
                self.presented = false;
                self.services.repaint = true;
            }
        }
        self.inter.hover = hover;
        self.last.snapshot.hover = hover;
        if hover != painted_hover {
            self.presented = false;
            self.services.repaint = true;
        }
    }

    /// Capture frozen read facts without updating or initializing the application.
    ///
    /// # Errors
    /// Rejects uninitialized, unsettled or incompatible runtime state. An old
    /// registry after `app_mut` is never presented as the current model's snapshot.
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

    /// The cursor position kept by the last successful presentation.
    pub const fn cursor_position(&self) -> Option<Position> {
        self.cursor
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
        ui.dim_layer(area, 2);
    }
    if drawn {
        ui.composite(i);
    }
}

/// Inspection for the test harness (§17.0 A1). Never in a release binary.
#[cfg(feature = "testing")]
impl<A: App> Runtime<A> {
    /// The hovered control.
    pub fn hover(&self) -> Option<Id> {
        self.inter.hover.map(|(o, _)| o)
    }

    /// Runtime-resolved flags for `id`.
    pub fn state_of(&self, id: Id) -> StateFlags {
        self.last.state(id)
    }

    /// Whether focus is painted.
    pub const fn focus_visible(&self) -> bool {
        self.focus.visible()
    }

    /// The top layer.
    pub fn top_layer(&self) -> LayerId {
        self.services.layers.top()
    }

    /// Whether layer `id` is open.
    pub fn is_open(&self, id: Id) -> bool {
        self.services.layers.is_open(id)
    }

    /// The cursor kept by the last draw.
    pub const fn cursor(&self) -> Option<Position> {
        self.cursor
    }

    /// What `id` actually got for `p` in the last draw.
    ///
    /// Returns the `Resolved` the component recorded when it queried the
    /// style, so the family and variant are the ones the component itself
    /// used — a `List`, a `Tabs` or a `Field` is never resolved through the
    /// button recipe (§16.4's theme-coupling migration contract). When `id`
    /// never styled `p`, the family and variant of any other query `id` made
    /// this frame are reused; when `id` styled nothing at all this falls back
    /// to `Family::BUTTON`. [`Runtime::resolved_in`] is the explicit escape
    /// hatch for a part that is never painted.
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

    /// Resolve `p` for `id` under an explicit family and variant.
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

    /// The invalidation returned by the last `handle`.
    pub const fn last_invalidate(&self) -> Invalidate {
        self.last_invalidate
    }

    /// Tags recorded through `Cx::record`.
    pub fn records(&self) -> &[&'static str] {
        &self.services.records
    }

    /// Drop recorded tags.
    pub fn clear_records(&mut self) {
        self.services.records.clear();
    }

    /// Registry produced by the last acknowledged projection, separate from live geometry.
    pub fn projection_registry(&self) -> Option<&crate::hit::Registry> {
        self.projection.as_ref().map(|output| &output.last.registry)
    }

    /// Focus ring produced by the last acknowledged projection.
    pub fn projection_ring(&self) -> Option<&FocusRing> {
        self.projection.as_ref().map(|output| &output.last.ring)
    }

    /// Cursor produced by the last acknowledged projection.
    pub fn projection_cursor(&self) -> Option<Position> {
        self.projection.as_ref().and_then(|output| output.cursor)
    }

    /// Project a one-shot painter without reconciling focus or running callbacks.
    /// Its derived model caches are fresh: an arbitrary closure carries no
    /// persistent model identity. Use `draw_bound_projection` for warmed captures.
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

    /// Project a scoped immutable model, reusing only this binding's caches.
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

    /// The draw phase with a closure instead of `App::draw` (headless scenes).
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

    /// The draw phase into a bare buffer.
    pub fn draw_buffer(&mut self, area: Rect, buf: &mut Buffer) -> PaintedFrame<'_, A> {
        self.draw_with_buffer(area, buf, None, A::draw);
        PaintedFrame {
            runtime: self,
            committed: false,
        }
    }

    /// The number of regions registered last frame.
    pub fn region_count(&self) -> usize {
        self.last.registry.len()
    }

    /// Last frame's registry.
    pub const fn registry(&self) -> &crate::hit::Registry {
        &self.last.registry
    }

    /// Advance explicit elapsed milliseconds through the public clock contract.
    pub fn advance_clock(&mut self, ms: u64) {
        let _ = self.advance_to(
            self.now()
                .saturating_add(core::time::Duration::from_millis(ms)),
        );
    }

    /// Whether a timed repaint was requested.
    pub fn repaint_after(&self) -> Option<core::time::Duration> {
        self.services
            .repaint_at
            .map(|at| at.saturating_duration_since(self.now()))
    }

    /// The live capture's owner.
    pub fn capture_owner(&self) -> Option<Id> {
        self.services.capture.get().map(|c| c.owner)
    }

    /// Force focus (a harness `blur`); delivered as `FocusOut`/`FocusIn` on
    /// the next `handle`.
    pub fn set_focus(&mut self, id: Option<Id>) {
        let from = self.focus.current();
        if from != id {
            self.presented = false;
            self.pending_focus = Some((from, id, FocusVia::Programmatic));
            self.focus.set(id);
            self.last.snapshot.focus = id;
        }
    }

    /// Intent-queue bucket probes since the runtime was built (adjudication
    /// 2.6): a frame with an empty queue must perform **zero**, and a frame
    /// with intents exactly one per `cx.intents` call.
    pub fn intent_probes(&self) -> usize {
        self.intents.probes()
    }

    /// `(hits, misses)` of the §11.1 A3 style memo (adjudication 2.8).
    pub fn style_cache_stats(&self) -> (u64, u64) {
        self.core.style_cache.stats()
    }

    /// The live spec of an open layer.
    pub fn open_spec(&self, id: Id) -> Option<crate::layer::LayerSpec> {
        self.services
            .layers
            .layers()
            .iter()
            .find(|l| l.id == id)
            .map(|l| l.spec)
    }

    /// The area the resolver gave layer `id` in the last draw.
    pub fn layer_area(&self, id: Id) -> Option<Rect> {
        self.frame
            .layers
            .active()
            .iter()
            .find(|l| l.id == id)
            .map(|l| l.area)
    }

    /// Record a diagnostic on the harness's behalf (`UnaddressableId`).
    pub fn record_diagnostic(&mut self, d: Diagnostic) {
        self.services.diagnostics.push(d);
    }
}

/// A scripted application for runtime-level tests.
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

    /// One control the stub draws.
    #[derive(Clone, Debug)]
    #[expect(
        clippy::struct_excessive_bools,
        reason = "a test stub's knobs: one flag per registration shape, set by field                   init from `Control::new`, never a state machine"
    )]
    pub(crate) struct Control {
        pub(crate) id: Id,
        pub(crate) area: Rect,
        pub(crate) focus: Focusability,
        /// Registers as an editor (swallows typing, consumes Esc as cancel).
        pub(crate) editor: bool,
        /// Claims capture on `Press`.
        pub(crate) captures: bool,
        /// Also registers a scroll region.
        pub(crate) scroll: bool,
        /// Registers a `Decorative` region only — no control, no part. This
        /// is what a `Dialog` registers for its own id.
        pub(crate) decor: bool,
    }

    impl Control {
        pub(crate) const fn new(id: Id, area: Rect) -> Self {
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
        pub(crate) page: Vec<Control>,
        pub(crate) layers: Vec<(Id, Vec<Control>)>,
        /// Every intent seen, as `(owner, Debug text)`.
        pub(crate) log: Vec<(Id, String)>,
        pub(crate) consume_keys: bool,
        pub(crate) focus_request: Option<Id>,
        pub(crate) open_request: Option<(Id, LayerSpec)>,
        pub(crate) close_request: Option<Id>,
        pub(crate) resize_request: Option<(Id, crate::layer::LayerSize)>,
        /// `update` calls so far.
        pub(crate) updates: usize,
        pub(crate) events_started: bool,
        /// Focus requests issued from inside `update` on each pass (settling tests).
        pub(crate) chase: Vec<Id>,
        pub(crate) esc_hits: usize,
        /// An owner whose bucket `update` never drains (the gated-shape app).
        pub(crate) skip_drain: Option<Id>,
    }

    impl Stub {
        pub(crate) fn controls(&self) -> impl Iterator<Item = &Control> + '_ {
            self.page
                .iter()
                .chain(self.layers.iter().flat_map(|(_, c)| c.iter()))
        }

        pub(crate) fn saw(&self, id: Id, needle: &str) -> bool {
            self.log.iter().any(|(o, s)| *o == id && s.contains(needle))
        }

        pub(crate) fn count(&self, id: Id, needle: &str) -> usize {
            self.log
                .iter()
                .filter(|(o, s)| *o == id && s.contains(needle))
                .count()
        }
    }

    impl App for Stub {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            self.updates = self.updates.saturating_add(1);
            // The bootstrap pass is lifecycle plumbing. Keep this fixture's
            // queued test actions for the first externally handled event.
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

    /// A runtime that has drawn once.
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

    /// Behavioral fixture delivery: publish real geometry, settle, then assert acceptance.
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

    /// Handle then draw, like the harness.
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
    use crate::layer::LayerSpec;
    use crate::ui::FrameRead;

    const A: Id = Id::root("a");
    const B: Id = Id::root("b");
    const C: Id = Id::root("c");
    const ROUTE_PUBLISH: u8 = 1 << 0;
    const ROUTE_ALTERNATE: u8 = 1 << 1;
    const ROUTE_SECOND: u8 = 1 << 2;
    const ROUTE_CONSUME_RAW: u8 = 1 << 3;
    const ROUTE_TAB: u8 = 1 << 4;

    #[derive(Default)]
    struct BootstrapProbe {
        causes: Vec<UpdateCause>,
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

    #[derive(Default)]
    struct GapCache(u32);
}
