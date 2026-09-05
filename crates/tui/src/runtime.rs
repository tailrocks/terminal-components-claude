//! The runtime (`COMPONENT_ARCHITECTURE.md` §3.3, §8, §9, §17.0 A1).
//!
//! `Runtime<A>` owns all interaction state — focus, hover, press, flash,
//! capture, layers, cursor, regions — and runs the exact two-phase frame
//! sequence: `handle` (steps 1–9, no buffer in scope) and `draw` (steps
//! 10–15, no `&mut` app state in scope). The application owns only domain
//! state and `XState`s.

#[cfg(feature = "crossterm")]
pub(crate) mod session;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::terminal::Frame;

use crate::action::ActionKey;
use crate::capture::Capture;
use crate::cursor::{self, CursorDecision};
use crate::diagnostics::Diagnostic;
use crate::event::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind};
use crate::focus::{FocusRing, FocusState, ScopeId};
use crate::hit::{Hit, RegionKind};
use crate::id::{Id, Part, PartRef};
use crate::intent::{FocusVia, IntentQueue, Phase};
use crate::keymap::{BindingTableId, KeyMap, KeyPhase};
use crate::layer::{
    Backdrop, DismissReason, LayerEvent, LayerId, OpenLayer, backdrop_area, resolve_anchor,
};
use crate::measure::Size;
use crate::response::{Invalidate, Response, StateFlags};
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
    hover: Option<(Id, PartRef)>,
    hover_suppressed: bool,
    press: Option<Press>,
    flash: Option<(Id, u64)>,
    last_click: Option<(Id, PartRef, u64)>,
    last_input_key: bool,
}

/// Passes of `app.update` before the runtime gives up settling focus.
const MAX_FOCUS_PASSES: usize = 4;

/// The runtime.
pub struct Runtime<A: App> {
    app: A,
    theme: Theme,
    screen: Rect,
    last: LastFrame,
    focus: FocusState,
    services: FrameServices,
    intents: IntentQueue,
    inter: Interaction,
    clock_ms: u64,
    frame: FrameState,
    core: UiCore,
    generation: u32,
    cursor: Option<Position>,
    last_invalidate: Invalidate,
    pending_focus: Option<(Option<Id>, Option<Id>, FocusVia)>,
    keymap_conflict_key: Option<(Option<(Id, BindingTableId)>, u64)>,
    keymap_conflicts: Vec<Diagnostic>,
    staged_focus: Option<(Option<Id>, FocusVia)>,
    layer_events_pending: Vec<(Id, LayerEvent)>,
    /// Layers closed by `DismissReason::FocusOut` since the last settle
    /// (§29.8 D2). Drained in lockstep with `services.closed_layers`.
    focus_out_closed: Vec<Id>,
    unsettled: usize,
    bootstrapped: bool,
}

impl<A: App> core::fmt::Debug for Runtime<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Runtime")
            .field("screen", &self.screen)
            .field("focus", &self.focus.current())
            .field("generation", &self.generation)
            .field("top_layer", &self.services.layers.top())
            .finish_non_exhaustive()
    }
}

impl<A: App> Runtime<A> {
    /// A runtime for `app` under `theme`.
    pub fn new(app: A, theme: Theme) -> Self {
        let mut core = UiCore::default();
        core.keymap.clone_from(app.keymap());
        Runtime {
            app,
            theme,
            screen: Rect::ZERO,
            last: LastFrame::default(),
            focus: FocusState::default(),
            services: FrameServices::default(),
            intents: IntentQueue::new(),
            inter: Interaction::default(),
            clock_ms: 0,
            frame: FrameState::default(),
            core,
            generation: 0,
            cursor: None,
            last_invalidate: Invalidate::None,
            pending_focus: None,
            keymap_conflict_key: None,
            keymap_conflicts: Vec::new(),
            staged_focus: None,
            layer_events_pending: Vec::new(),
            focus_out_closed: Vec::new(),
            unsettled: 0,
            bootstrapped: false,
        }
    }

    /// The application.
    pub const fn app(&self) -> &A {
        &self.app
    }

    /// The application, mutably.
    pub const fn app_mut(&mut self) -> &mut A {
        &mut self.app
    }

    /// The theme.
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Replace the theme; derived caches are dropped.
    pub fn set_theme(&mut self, t: Theme) {
        self.theme = t;
        self.core.clear_caches();
    }

    /// Last frame's area of a control (`None` before it first draws).
    pub fn area_of(&self, id: Id) -> Option<Rect> {
        self.last.registry.area_of(id)
    }

    /// Last frame's area of a component's sub-region.
    pub fn area_of_part(&self, id: Id, p: PartRef) -> Option<Rect> {
        self.last.registry.area_of_part(id, p)
    }

    /// Last frame's focus ring.
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

    /// Whether a repaint (or a timed repaint) is pending: the loop should
    /// wait at the tick cadence.
    pub fn wants_tick(&self) -> bool {
        self.services.repaint
            || self.services.repaint_after.is_some()
            || self.inter.flash.is_some()
            || self.last_invalidate >= Invalidate::Paint
    }

    /// The earliest outstanding repaint deadline requested by the app.
    ///
    /// The duration is relative to the update that established the deadline.
    /// It persists across unrelated input and is consumed when the runtime
    /// receives [`Input::Tick`]. Multiple requests keep the shortest duration.
    pub const fn next_deadline(&self) -> Option<core::time::Duration> {
        self.services.repaint_after
    }

    /// The virtual clock, in milliseconds (advanced by `Input::Tick`).
    pub const fn clock_ms(&self) -> u64 {
        self.clock_ms
    }

    /// The terminal size as last resized or drawn.
    pub const fn screen(&self) -> Rect {
        self.screen
    }

    fn top(&self) -> LayerId {
        self.services.layers.top()
    }

    fn sync_keymap(&mut self) {
        if &self.core.keymap != self.app.keymap() {
            self.core.keymap.clone_from(self.app.keymap());
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
        let key = (
            component.map(|(owner, table, _)| (owner, table)),
            self.core.keymap_revision,
        );
        if self.keymap_conflict_key == Some(key) {
            return;
        }
        self.keymap_conflict_key = Some(key);
        self.keymap_conflicts.clear();
        self.keymap_conflicts.extend(self.core.keymap.conflicts());
        if let Some((owner, _, table)) = component {
            self.keymap_conflicts
                .extend(self.core.keymap.component_conflicts(owner, table));
        }
    }

    /// Whether `to` may be staged as a focus target.
    ///
    /// Every producer of a transition — `Tab` traversal, a press, a layer
    /// restore, `Cx::focus` and `LayerSpec::initial_focus` — funnels through
    /// [`Self::stage_focus`], so this is the one place a target is judged.
    /// A target is refused only when the last frame *proves* it cannot hold
    /// focus:
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
    /// construction, and so does the §21 item 15 restore to an opener a
    /// modal had made inert. Step 14 reconciliation settles those against
    /// the ring the next frame actually produces.
    ///
    /// A refusal is silent: `Diagnostic` has no variant for a rejected focus
    /// target and adding one is a §17.0 A9 amendment, not an implementation
    /// choice.
    fn focus_target_admissible(&self, to: Id) -> bool {
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
        self.dismiss_on_focus_out(to);
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

    fn focused_is_editing(&self) -> bool {
        self.focus.current().is_some_and(|f| {
            self.last.state(f).contains(StateFlags::EDITING) || self.swallows_typing()
        })
    }

    /// Step 1 for a resize.
    fn resize(&mut self, w: u16, h: u16) {
        self.screen = Rect {
            x: 0,
            y: 0,
            width: w,
            height: h,
        };
        self.services.capture.release();
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
        hit.layer == self.top()
            && hit.kind != RegionKind::Decorative
            && !self.ring_proves_disabled(hit.owner)
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
        self.inter.last_input_key = false;
        self.focus.set_visible(false);
        self.last.snapshot.focus_visible = false;
        let hover_was_suppressed = self.inter.hover_suppressed;
        if m.kind == MouseKind::Move {
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
            o == p.owner && part == p.part && self.clock_ms.saturating_sub(at) <= window
        });
        let phase = if double {
            Phase::DoubleClick
        } else {
            Phase::Click
        };
        self.inter.last_click = if double {
            None
        } else {
            Some((p.owner, p.part, self.clock_ms))
        };
        self.intents
            .pointer(p.owner, phase, p.part, m.pos, local, m.mods);
        let flash = self.theme.design.motion.press_flash_ms;
        self.inter.flash = Some((p.owner, self.clock_ms.saturating_add(flash)));
        self.last.snapshot.pressed = Some((p.owner, p.part));
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
            self.stage_focus(target, FocusVia::Restore);
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
    fn run_update(&mut self, command: Option<ActionKey>, cause: UpdateCause) -> Response<()> {
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
                );
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

    /// Run the one update that precedes the first draw or externally handled
    /// event. Keeping this on `Runtime` gives terminal and headless callers
    /// identical lifecycle semantics.
    fn ensure_bootstrap(&mut self) {
        if self.bootstrapped {
            return;
        }
        self.bootstrapped = true;
        self.sync_keymap();
        self.refresh_keymap_conflicts();
        self.services
            .diagnostics
            .extend(self.keymap_conflicts.iter().cloned());
        self.services.registry_gen = self.last.registry.generation();
        self.intents.clear();
        let r = self.run_update(None, UpdateCause::Bootstrap);
        let _ = self.finish(r);
    }

    /// `Runtime::handle` — steps 1–9.
    pub fn handle(&mut self, input: Input) -> Response<()> {
        self.ensure_bootstrap();
        self.services.diagnostics.clear();
        self.sync_keymap();
        self.refresh_keymap_conflicts();
        self.services
            .diagnostics
            .extend(self.keymap_conflicts.iter().cloned());
        self.services.repaint = false;
        self.intents.clear();
        self.services.registry_gen = self.last.registry.generation();
        // focus moved at the last draw's reconcile: deliver FocusOut/FocusIn now
        if let Some((from, to, via)) = self.pending_focus.take() {
            if let Some(old) = from {
                self.intents.focus_out(old, to);
            }
            if let Some(new) = to {
                self.intents.focus_in(new, via);
            }
        }
        self.pump_layer_events();
        let mut key_input: Option<Key> = None;
        let mut update_cause = UpdateCause::Event;
        match input {
            Input::Resize(w, h) => {
                self.resize(w, h);
                // step 1 then the ordinary update pass: the `FocusOut`/
                // `FocusIn` pair staged for a `pending_focus` is already in
                // the queue and must reach `app.update` before `finish`
                // clears it (MA-7).
                let r = self.run_update(None, UpdateCause::Event) | Response::changed().relayout();
                return self.finish(r);
            }
            Input::Tick => {
                update_cause = UpdateCause::Tick;
                // A delivered tick consumes the deadline that woke it. Any
                // replacement requested by the Tick update becomes the next
                // deadline and therefore survives this handle.
                self.services.repaint_after = None;
                self.clock_ms = self
                    .clock_ms
                    .saturating_add(self.theme.design.motion.tick_ms);
                if let Some((_, until)) = self.inter.flash
                    && self.clock_ms >= until
                {
                    self.inter.flash = None;
                    if self.inter.press.is_none() {
                        self.last.snapshot.pressed = None;
                    }
                }
            }
            Input::Key(k) => {
                // step 2: capture chords first
                let swallows = self.swallows_typing();
                if let Some(cmd) = self.core.keymap.lookup(KeyPhase::Capture, &k, swallows) {
                    let r = self.run_update(Some(cmd), UpdateCause::Event);
                    return self.finish(r);
                }
                key_input = Some(k);
                self.enqueue_key(k);
            }
            Input::Mouse(m) => self.enqueue_mouse(m),
            Input::Paste(ref s) => {
                if let Some(owner) = self.focus.current()
                    && self.focused_is_editing()
                {
                    self.intents.paste(owner, &s);
                }
            }
        }
        let mut r = self.run_update(None, update_cause);
        // step 8: bubble
        if let Some(k) = key_input
            && !r.is_consumed()
        {
            if let Some(cmd) = self.core.keymap.lookup(KeyPhase::Bubble, &k, false) {
                r |= self.run_update(Some(cmd), UpdateCause::Event);
            } else if k.code == KeyCode::Esc {
                let dismissable = self
                    .services
                    .layers
                    .top_layer()
                    .is_some_and(|l| l.spec.dismiss.esc);
                if dismissable {
                    self.dismiss_top(DismissReason::Esc);
                    r |= self.run_update(None, UpdateCause::Event).repaint();
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
        self.finish(r)
    }

    /// Step 9.
    fn finish(&mut self, mut r: Response<()>) -> Response<()> {
        if self.services.repaint {
            r = r.repaint();
        }
        self.intents.clear();
        self.last_invalidate = r.invalidate();
        r
    }

    /// `Runtime::draw` — steps 10–15.
    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        self.draw_into(area, buf);
        if let Some(p) = self.cursor {
            frame.set_cursor_position(p);
        }
    }

    /// The draw phase into a bare buffer (headless scenes and tests).
    pub(crate) fn draw_into(&mut self, area: Rect, buf: &mut Buffer) {
        self.draw_with_buffer(area, buf, A::draw);
    }

    fn draw_with_buffer(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        paint: impl FnOnce(&A, &mut Ui<'_>),
    ) {
        self.ensure_bootstrap();
        // step 10: new frame state
        if area != self.screen {
            self.screen = area;
            self.core.clear_caches();
        }
        self.generation = self.generation.wrapping_add(1);
        self.core.begin_cache_frame(self.generation);
        self.core.style_cache.clear();
        self.frame.reset(self.generation, area);
        self.frame.inert_floor = self.services.layers.inert_floor();
        self.frame.top = self.services.layers.top();
        // Prepare every open layer's draw target. Its focus scope is armed
        // only when `Ui::layer` performs a live draw.
        for l in self.services.layers.layers() {
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
                &self.last,
            );
            paint(app, &mut ui);
            // step 12: composite bottom-to-top
            for i in 0..ui_layer_count(&ui) {
                composite_layer(&mut ui, i, area);
            }
        }
        // step 13: registry swap, stale captures released
        let mut diags = core::mem::take(&mut self.frame.diagnostics);
        self.services.diagnostics.extend(diags.drain(..));
        core::mem::swap(&mut self.last.registry, &mut self.frame.registry);
        self.last.layout.clear();
        self.last.layout.append(&mut self.frame.layout);
        self.last.declared.clear();
        self.last.declared.append(&mut self.frame.declared);
        core::mem::swap(&mut self.last.bindings, &mut self.frame.bindings);
        self.services.capture.release_if_stale(&self.last.registry);
        self.last.snapshot.capture = self.services.capture.get().map(|c| c.owner);
        // step 14: focus reconcile. A modal's trap moves focus into it by
        // rule (c); a popover leaves focus where it is unless the spec named
        // an `initial_focus` (§16.2 case 17: a component stays focused under
        // a popover and its cursor write is rejected)
        let previous = self.focus.current();
        let reconciled = self.frame.ring.reconcile(&self.last.ring, previous);
        core::mem::swap(&mut self.last.ring, &mut self.frame.ring);
        if reconciled != previous {
            let via = if self
                .staged_focus
                .is_some_and(|(_, v)| v == FocusVia::Restore)
            {
                FocusVia::Restore
            } else {
                FocusVia::Programmatic
            };
            self.pending_focus = Some((previous, reconciled, via));
            self.focus.set(reconciled);
            // the frame was painted with the old focus: ask for another one
            self.services.repaint = true;
        }
        self.staged_focus = None;
        self.last.snapshot.focus = self.focus.current();
        self.last.snapshot.focus_visible = self.focus.visible();
        // hover is re-resolved against the new registry so a control that
        // moved under a still pointer does not keep a stale hover
        if let Some((owner, _)) = self.inter.hover
            && !self.last.registry.has_owner(owner)
        {
            self.inter.hover = None;
            self.last.snapshot.hover = None;
        }
        // step 15: cursor
        self.cursor = match self.frame.cursor {
            None => None,
            Some(req) => match cursor::resolve(req, self.frame.top, self.focus.current()) {
                CursorDecision::Keep(p) => Some(p),
                CursorDecision::Reject(d) => {
                    self.services.diagnostics.push(d);
                    None
                }
                CursorDecision::Silent => None,
            },
        };
    }

    /// The cursor position kept by the last draw.
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

    /// The draw phase with a closure instead of `App::draw` (headless scenes).
    pub fn draw_scene(&mut self, area: Rect, buf: &mut Buffer, f: impl FnOnce(&mut Ui<'_>, Rect)) {
        self.draw_with_buffer(area, buf, |_, ui| f(ui, area));
    }

    /// The draw phase into a bare buffer.
    pub fn draw_buffer(&mut self, area: Rect, buf: &mut Buffer) {
        self.draw_into(area, buf);
    }

    /// The number of regions registered last frame.
    pub fn region_count(&self) -> usize {
        self.last.registry.len()
    }

    /// Last frame's registry.
    pub const fn registry(&self) -> &crate::hit::Registry {
        &self.last.registry
    }

    /// Advance the virtual clock by one tick without input handling.
    pub fn advance_clock(&mut self, ms: u64) {
        self.clock_ms = self.clock_ms.saturating_add(ms);
    }

    /// Whether a timed repaint was requested.
    pub const fn repaint_after(&self) -> Option<core::time::Duration> {
        self.services.repaint_after
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
#[cfg(test)]
pub(crate) mod stub {
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
    pub(crate) struct Stub {
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

    pub(crate) const SCREEN: Rect = Rect {
        x: 0,
        y: 0,
        width: 40,
        height: 12,
    };

    /// A runtime that has drawn once.
    pub(crate) fn runtime(stub: Stub) -> (Runtime<Stub>, Buffer) {
        let mut rt = Runtime::new(stub, Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        (rt, buf)
    }

    pub(crate) fn key(code: KeyCode) -> Input {
        Input::Key(Key {
            code,
            mods: KeyModifiers::NONE,
        })
    }

    pub(crate) fn mouse(kind: MouseKind, x: u16, y: u16) -> Input {
        Input::Mouse(Mouse {
            kind,
            pos: Position::new(x, y),
            mods: KeyModifiers::NONE,
        })
    }

    /// Handle then draw, like the harness.
    pub(crate) fn step(rt: &mut Runtime<Stub>, buf: &mut Buffer, input: Input) -> Response<()> {
        let r = rt.handle(input);
        rt.draw_buffer(SCREEN, buf);
        r
    }
}

#[cfg(test)]
mod tests {
    use super::stub::{Control, SCREEN, Stub, key, mouse, runtime, step};
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
        let mut buf = Buffer::empty(SCREEN);

        assert_eq!(rt.clock_ms(), 0);
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(rt.app().causes, vec![UpdateCause::Bootstrap]);
        assert_eq!(rt.clock_ms(), 0);
        assert_eq!(
            rt.next_deadline(),
            Some(core::time::Duration::from_millis(33))
        );

        rt.draw_buffer(SCREEN, &mut buf);
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
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        rt.app_mut().causes.clear();

        let _ = rt.handle(Input::Tick);
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
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        assert_eq!(
            rt.next_deadline(),
            Some(core::time::Duration::from_millis(40))
        );

        let _ = rt.handle(Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }));
        assert_eq!(
            rt.next_deadline(),
            Some(core::time::Duration::from_millis(40))
        );

        let _ = rt.handle(Input::Mouse(Mouse {
            kind: MouseKind::Move,
            pos: Position::new(1, 1),
            mods: KeyModifiers::NONE,
        }));
        assert_eq!(
            rt.next_deadline(),
            Some(core::time::Duration::from_millis(10))
        );

        let _ = rt.handle(Input::Tick);
        assert_eq!(rt.next_deadline(), None);
    }

    #[test]
    fn headless_tick_uses_the_same_update_cause_without_wall_clock() {
        let mut rt = Runtime::new(BootstrapProbe::default(), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        rt.app_mut().causes.clear();

        let before = rt.clock_ms();
        let _ = rt.handle(Input::Tick);
        assert_eq!(rt.app().causes, vec![UpdateCause::Tick]);
        assert_eq!(rt.clock_ms(), before + rt.theme().design.motion.tick_ms);
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
            if cx.update_cause() != UpdateCause::Bootstrap
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
        plain.draw_buffer(SCREEN, &mut expected);

        let mut runtime = Runtime::new(
            ReferenceLayerApp {
                open: true,
                draw_live: false,
            },
            Theme::junie(),
        );
        let _ = runtime.handle(Input::Tick);
        let mut actual = Buffer::empty(SCREEN);
        runtime.draw_buffer(SCREEN, &mut actual);
        assert_eq!(actual, expected, "an undrawn layer left a backdrop");
        assert_eq!(runtime.focus(), Some(REFERENCE_PAGE));
        assert!(runtime.diagnostics().is_empty());

        runtime.app_mut().draw_live = true;
        actual.reset();
        runtime.draw_buffer(SCREEN, &mut actual);
        assert_ne!(actual, expected, "a later legitimate layer draw was lost");
        assert_eq!(runtime.focus(), Some(REFERENCE_LAYER_CONTROL));
        assert!(runtime.diagnostics().is_empty());
    }

    #[derive(Default)]
    struct GapCache(u32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum RouteCmd {
        Default,
        Latent,
        Alternate,
    }

    const DEFAULT_ACTION: ActionKey = ActionKey::custom("runtime.route.default");
    const LATENT_ACTION: ActionKey = ActionKey::custom("runtime.route.latent");
    const DEFAULT_BINDINGS: &[crate::Binding<RouteCmd>] = &[
        crate::Binding {
            action: DEFAULT_ACTION,
            chord: Some(crate::Chord::key(KeyCode::Enter)),
            cmd: RouteCmd::Default,
            label: "Default",
            priority: 80,
            visible: true,
        },
        crate::Binding {
            action: LATENT_ACTION,
            chord: None,
            cmd: RouteCmd::Latent,
            label: "Latent",
            priority: 60,
            visible: true,
        },
    ];
    const ALTERNATE_BINDINGS: &[crate::Binding<RouteCmd>] = &[crate::Binding {
        action: ActionKey::custom("runtime.route.alternate"),
        chord: Some(crate::Chord::key(KeyCode::F(2))),
        cmd: RouteCmd::Alternate,
        label: "Alternate",
        priority: 80,
        visible: true,
    }];
    const TAB_ACTION: ActionKey = ActionKey::custom("runtime.route.tab");
    const TAB_BINDINGS: &[crate::Binding<RouteCmd>] = &[crate::Binding {
        action: TAB_ACTION,
        chord: Some(crate::Chord::key(KeyCode::Tab)),
        cmd: RouteCmd::Alternate,
        label: "Tab action",
        priority: 80,
        visible: true,
    }];

    #[derive(Debug)]
    struct RouteApp {
        keymap: KeyMap,
        options: u8,
        typed: Vec<(Id, RouteCmd)>,
        raw: usize,
        app_commands: usize,
    }

    impl Default for RouteApp {
        fn default() -> Self {
            RouteApp {
                keymap: KeyMap::new(),
                options: ROUTE_PUBLISH,
                typed: Vec::new(),
                raw: 0,
                app_commands: 0,
            }
        }
    }

    impl RouteApp {
        fn table(&self) -> &'static [crate::Binding<RouteCmd>] {
            if self.options & ROUTE_TAB != 0 {
                TAB_BINDINGS
            } else if self.options & ROUTE_ALTERNATE != 0 {
                ALTERNATE_BINDINGS
            } else {
                DEFAULT_BINDINGS
            }
        }
    }

    impl App for RouteApp {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            if cx.command().is_some() {
                self.app_commands = self.app_commands.saturating_add(1);
                return Response::consumed();
            }
            let mut response = Response::ignored();
            let table = self.table();
            for owner in [A, B] {
                if owner == B && self.options & ROUTE_SECOND == 0 {
                    continue;
                }
                for intent in cx.intents(owner) {
                    match intent {
                        crate::Intent::Binding(action) => {
                            if let Some(command) = crate::Binding::command(table, action) {
                                self.typed.push((owner, command));
                                response |= Response::consumed();
                            }
                        }
                        crate::Intent::Key(_) => {
                            self.raw = self.raw.saturating_add(1);
                            if self.options & ROUTE_CONSUME_RAW != 0 {
                                response |= Response::consumed();
                            }
                        }
                        _ => {}
                    }
                }
            }
            response
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            let table = self.table();
            ui.register_control(A, Rect::new(0, 0, 10, 1), Focusability::Focusable);
            if self.options & ROUTE_PUBLISH != 0 {
                ui.publish_bindings(A, ui.state(A), table);
            }
            if self.options & ROUTE_SECOND != 0 {
                ui.register_control(B, Rect::new(12, 0, 10, 1), Focusability::Focusable);
                if self.options & ROUTE_PUBLISH != 0 {
                    ui.publish_bindings(B, ui.state(B), table);
                }
            }
            let _ =
                crate::HintBar::derived(Id::root("runtime.hints")).draw(ui, Rect::new(0, 1, 40, 1));
        }

        fn keymap(&self) -> &KeyMap {
            &self.keymap
        }
    }

    fn route_runtime(app: RouteApp) -> (Runtime<RouteApp>, Buffer) {
        let area = Rect::new(0, 0, 40, 3);
        let mut buffer = Buffer::empty(area);
        let mut runtime = Runtime::new(app, Theme::junie());
        runtime.draw_buffer(area, &mut buffer);
        runtime.draw_buffer(area, &mut buffer);
        (runtime, buffer)
    }

    fn hint_row(buffer: &Buffer) -> String {
        (0..40)
            .filter_map(|x| buffer.cell(Position::new(x, 1)))
            .map(ratatui_core::buffer::Cell::symbol)
            .collect()
    }

    #[test]
    fn default_action_routes_to_typed_command() {
        let (mut runtime, _) = route_runtime(RouteApp::default());
        let response = runtime.handle(key(KeyCode::Enter));
        assert!(response.is_consumed());
        assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Default)]);
        assert_eq!(runtime.app().raw, 0);
    }

    #[test]
    fn owner_remove_suppresses_handling_and_hint() {
        let mut app = RouteApp::default();
        app.keymap.remove_component(A, DEFAULT_ACTION);
        let (mut runtime, mut buffer) = route_runtime(app);
        assert!(!hint_row(&buffer).contains("Default"));
        let response = runtime.handle(key(KeyCode::Enter));
        runtime.draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer);
        assert!(!response.is_consumed());
        assert!(runtime.app().typed.is_empty());
        assert_eq!(runtime.app().raw, 1);
    }

    #[test]
    fn remap_old_is_raw_and_new_is_typed_and_shown() {
        let mut app = RouteApp::default();
        app.keymap
            .remap_component(A, DEFAULT_ACTION, crate::Chord::key(KeyCode::F(3)));
        let (mut runtime, mut buffer) = route_runtime(app);
        assert!(hint_row(&buffer).contains("F3"));
        let old = runtime.handle(key(KeyCode::Enter));
        assert!(!old.is_consumed());
        assert_eq!(runtime.app().raw, 1);
        runtime.draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer);
        let new = runtime.handle(key(KeyCode::F(3)));
        assert!(new.is_consumed());
        assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Default)]);
    }

    #[test]
    fn latent_action_activates_when_component_bound() {
        let mut app = RouteApp::default();
        app.keymap = app
            .keymap
            .bind_component(A, LATENT_ACTION, crate::Chord::key(KeyCode::F(4)));
        let (mut runtime, buffer) = route_runtime(app);
        assert!(hint_row(&buffer).contains("F4"));
        assert!(runtime.handle(key(KeyCode::F(4))).is_consumed());
        assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Latent)]);
    }

    #[test]
    fn component_override_is_owner_scoped() {
        let mut app = RouteApp {
            options: ROUTE_PUBLISH | ROUTE_SECOND,
            ..RouteApp::default()
        };
        app.keymap
            .remap_component(A, DEFAULT_ACTION, crate::Chord::key(KeyCode::F(5)));
        let (mut runtime, mut buffer) = route_runtime(app);
        let _ = runtime.handle(key(KeyCode::Tab));
        runtime.draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer);
        assert_eq!(runtime.focus(), Some(B));
        assert!(runtime.handle(key(KeyCode::Enter)).is_consumed());
        assert_eq!(runtime.app().typed, vec![(B, RouteCmd::Default)]);
    }

    #[test]
    fn capture_keymap_wins_over_component_binding() {
        let mut app = RouteApp::default();
        app.keymap.add(
            KeyPhase::Capture,
            crate::Chord::key(KeyCode::Enter),
            ActionKey::SAVE,
        );
        let (mut runtime, _) = route_runtime(app);
        assert!(runtime.handle(key(KeyCode::Enter)).is_consumed());
        assert_eq!(runtime.app().app_commands, 1);
        assert!(runtime.app().typed.is_empty());
    }

    #[test]
    fn bubble_runs_only_when_raw_key_is_ignored() {
        let mut app = RouteApp::default();
        app.keymap.add(
            KeyPhase::Bubble,
            crate::Chord::key(KeyCode::F(6)),
            ActionKey::SAVE,
        );
        let (mut runtime, _) = route_runtime(app);
        assert!(runtime.handle(key(KeyCode::F(6))).is_consumed());
        assert_eq!(runtime.app().app_commands, 1);

        runtime.app_mut().options |= ROUTE_CONSUME_RAW;
        assert!(runtime.handle(key(KeyCode::F(6))).is_consumed());
        assert_eq!(runtime.app().app_commands, 1);
    }

    #[test]
    fn same_flags_table_switch_invalidates_focused_hints() {
        let (mut runtime, mut buffer) = route_runtime(RouteApp::default());
        assert!(hint_row(&buffer).contains("Default"));
        let capacity = runtime.core.focused_hints.layer.hints.capacity();
        runtime.app_mut().options |= ROUTE_ALTERNATE;
        runtime.draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer);
        assert!(hint_row(&buffer).contains("Alternate"));
        assert_eq!(runtime.core.focused_hints.layer.hints.capacity(), capacity);
    }

    #[test]
    fn map_replacement_invalidates_focused_hints() {
        let (mut runtime, mut buffer) = route_runtime(RouteApp::default());
        runtime.app_mut().keymap.remap_component(
            A,
            DEFAULT_ACTION,
            crate::Chord::key(KeyCode::F(7)),
        );
        let _ = runtime.handle(Input::Tick);
        runtime.draw_buffer(Rect::new(0, 0, 40, 3), &mut buffer);
        assert!(hint_row(&buffer).contains("F7"));
    }

    #[test]
    fn unpublished_legacy_control_receives_raw_key() {
        let app = RouteApp {
            options: 0,
            ..RouteApp::default()
        };
        let (mut runtime, _) = route_runtime(app);
        assert!(!runtime.handle(key(KeyCode::Enter)).is_consumed());
        assert_eq!(runtime.app().raw, 1);
        assert!(runtime.app().typed.is_empty());
    }

    #[test]
    fn component_tab_binding_precedes_focus_traversal() {
        let app = RouteApp {
            options: ROUTE_PUBLISH | ROUTE_SECOND | ROUTE_TAB,
            ..RouteApp::default()
        };
        let (mut runtime, _) = route_runtime(app);
        assert!(runtime.handle(key(KeyCode::Tab)).is_consumed());
        assert_eq!(runtime.focus(), Some(A));
        assert_eq!(runtime.app().typed, vec![(A, RouteCmd::Alternate)]);
    }

    #[test]
    fn removed_tab_binding_falls_back_to_focus_traversal() {
        let mut app = RouteApp {
            options: ROUTE_PUBLISH | ROUTE_SECOND | ROUTE_TAB,
            ..RouteApp::default()
        };
        app.keymap.remove_component(A, TAB_ACTION);
        let (mut runtime, _) = route_runtime(app);
        let _ = runtime.handle(key(KeyCode::Tab));
        assert_eq!(runtime.focus(), Some(B));
        assert!(runtime.app().typed.is_empty());
    }

    fn three() -> Stub {
        Stub {
            page: vec![
                Control::new(A, Rect::new(0, 0, 10, 1)),
                Control::new(B, Rect::new(0, 2, 10, 1)),
                Control::new(C, Rect::new(0, 4, 10, 1)),
            ],
            consume_keys: true,
            ..Stub::default()
        }
    }

    #[test]
    fn derived_cache_is_fresh_when_component_reappears_after_a_frame_gap() {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let area = Rect::new(0, 0, 4, 1);
        let mut buffer = Buffer::empty(area);
        let component = Id::root("cache-gap");

        runtime.draw_scene(area, &mut buffer, |ui, _| {
            ui.cache::<GapCache>(component).0 = 7;
        });
        runtime.draw_scene(area, &mut buffer, |_ui, _| {});

        let observed = core::cell::Cell::new(u32::MAX);
        runtime.draw_scene(area, &mut buffer, |ui, _| {
            observed.set(ui.cache::<GapCache>(component).0);
        });
        assert_eq!(observed.get(), 0);
    }

    #[test]
    fn first_draw_focuses_the_first_control_and_tab_walks_the_ring() {
        let (mut rt, mut buf) = runtime(three());
        assert_eq!(rt.focus(), Some(A));
        let r = step(&mut rt, &mut buf, key(KeyCode::Tab));
        assert_eq!(rt.focus(), Some(B));
        assert!(rt.app().saw(A, "FocusOut") && rt.app().saw(B, "FocusIn { via: Keyboard }"));
        assert!(rt.focus_visible());
        let _ = r;
        let _ = step(&mut rt, &mut buf, key(KeyCode::BackTab));
        assert_eq!(rt.focus(), Some(A));
        let _ = step(&mut rt, &mut buf, key(KeyCode::BackTab));
        assert_eq!(rt.focus(), Some(C));
    }

    #[test]
    fn keys_go_to_the_focused_owner_and_are_consumed_or_bubbled() {
        let (mut rt, mut buf) = runtime(three());
        let r = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(r.is_consumed());
        assert_eq!(rt.app().count(A, "Key("), 1);
        assert_eq!(rt.app().count(B, "Key("), 0);
        rt.app_mut().consume_keys = false;
        let r = step(&mut rt, &mut buf, key(KeyCode::Esc));
        assert!(r.is_consumed(), "the on_esc ladder consumed it");
        assert_eq!(rt.app().esc_hits, 1);
    }

    #[test]
    fn press_focuses_the_owner_and_click_is_delivered_with_local_coordinates() {
        let (mut rt, mut buf) = runtime(three());
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 2));
        assert_eq!(rt.focus(), Some(B));
        assert!(!rt.focus_visible());
        assert!(rt.state_of(B).contains(StateFlags::PRESSED));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 2));
        assert!(rt.app().saw(B, "phase: Press"));
        assert!(rt.app().saw(B, "phase: Click"));
        assert!(rt.app().saw(B, "local: Position { x: 3, y: 0 }"));
        // hover never focuses
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 4));
        assert_eq!(rt.hover(), Some(C));
        assert_eq!(rt.focus(), Some(B));
        assert!(rt.state_of(C).contains(StateFlags::HOVERED));
        // a key suppresses hover until the pointer moves
        let _ = step(&mut rt, &mut buf, key(KeyCode::Down));
        assert!(!rt.state_of(C).contains(StateFlags::HOVERED));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 4, 4));
        assert!(rt.state_of(C).contains(StateFlags::HOVERED));
    }

    #[test]
    fn pointer_move_delivers_move_to_topmost_live_part() {
        let layer = Id::root("top");
        let decor = Id::root("decor");
        let mut s = Stub {
            page: vec![Control::new(A, Rect::new(0, 0, 10, 1))],
            ..Stub::default()
        };
        s.layers.push((
            layer,
            vec![
                Control::new(B, Rect::new(0, 0, 10, 1)),
                Control {
                    decor: true,
                    ..Control::new(decor, Rect::new(0, 0, 10, 1))
                },
            ],
        ));
        let (mut rt, mut buf) = runtime(s);
        rt.app_mut().open_request = Some((layer, LayerSpec::modal(layer)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        rt.app_mut().log.clear();

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 0));

        assert_eq!(rt.app().count(B, "phase: Move"), 1);
        assert_eq!(rt.app().count(A, "phase: Move"), 0);
        assert_eq!(rt.app().count(decor, "Pointer"), 0);
        assert!(rt.app().saw(B, "part: PartRef { part: Part::LABEL"));
    }

    #[test]
    fn pointer_move_over_decorative_delivers_nothing() {
        let decor = Id::root("decor");
        let s = Stub {
            page: vec![Control {
                decor: true,
                ..Control::new(decor, Rect::new(0, 0, 10, 1))
            }],
            ..Stub::default()
        };
        let (mut rt, mut buf) = runtime(s);

        let response = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 0));

        assert_eq!(rt.hover(), None);
        assert_eq!(response.invalidate(), Invalidate::None);
        assert_eq!(rt.app().count(decor, "Pointer"), 0);
    }

    #[test]
    fn captured_move_is_drag_not_move() {
        let mut s = three();
        s.page[0].captures = true;
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 0));
        assert_eq!(rt.capture_owner(), Some(A));
        rt.app_mut().log.clear();

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 2));

        assert_eq!(rt.app().count(A, "phase: Drag"), 1);
        assert_eq!(rt.app().count(A, "phase: Move"), 0);
        assert_eq!(rt.app().count(B, "Pointer"), 0);
    }

    #[test]
    fn pointer_move_never_focuses() {
        let (mut rt, mut buf) = runtime(three());
        assert_eq!(rt.focus(), Some(A));

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 2));

        assert_eq!(rt.focus(), Some(A));
        assert_eq!(rt.hover(), Some(B));
        assert_eq!(rt.app().count(B, "phase: Move"), 1);
        assert_eq!(rt.app().count(B, "phase: Press"), 0);
        assert_eq!(rt.app().count(B, "phase: Click"), 0);
    }

    /// §73 AC-1. A control the last frame registered as
    /// `Focusability::Disabled` is registered and never reachable — by `Tab`
    /// through `focus_target_admissible`, and now by the pointer through
    /// `deliverable`. No `Press`, no `Release`, no `Click`, no `Secondary`
    /// reaches it, no press is recorded for it, and a refused press moves no
    /// focus. Before §73 the two disagreed: keyboard refused the control
    /// while a click still activated it.
    #[test]
    fn ac1_a_disabled_control_refuses_every_activating_pointer_intent() {
        let mut s = three();
        s.page[1].focus = Focusability::Disabled;
        let (mut rt, mut buf) = runtime(s);
        assert_eq!(rt.focus(), Some(A));

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 2));
        assert_eq!(rt.app().count(B, "phase: Press"), 0);
        assert!(!rt.state_of(B).contains(StateFlags::PRESSED));
        assert_eq!(rt.focus(), Some(A), "a refused press moved focus");

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 2));
        assert_eq!(rt.app().count(B, "phase: Release"), 0);
        assert_eq!(rt.app().count(B, "phase: Click"), 0);

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Secondary, 3, 2));
        assert_eq!(rt.app().count(B, "phase: Secondary"), 0);
        assert_eq!(rt.app().count(B, "Pointer"), 0);

        // the enabled neighbour is untouched by the refusal
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 4));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 4));
        assert_eq!(rt.focus(), Some(C));
        assert_eq!(rt.app().count(C, "phase: Click"), 1);
    }

    /// §73 AC-2. The refusal is scoped to activation and to the ring's own
    /// proof. Hover and `Phase::Move` come from `hit_live`, so a disabled
    /// control still hovers; `Intent::Wheel` comes from `hit_scroll`, so its
    /// scroll region still scrolls; a `Focusability::ClickOnly` owner takes
    /// no ring entry at all, so it is never proven disabled and still
    /// presses and clicks; and a hit below the top layer is `outside`
    /// before deliverability is consulted, so the disabled control does not
    /// swallow an outside-click dismissal.
    #[test]
    fn ac2_the_refusal_leaves_move_wheel_click_only_and_outside_clicks_intact() {
        let disabled = Id::root("disabled-scroll");
        let click_only = Id::root("click-only");
        let popover = Id::root("popover");
        let mut s = Stub {
            page: vec![
                Control::new(A, Rect::new(0, 0, 10, 1)),
                Control {
                    focus: Focusability::Disabled,
                    scroll: true,
                    ..Control::new(disabled, Rect::new(20, 0, 10, 10))
                },
                Control {
                    focus: Focusability::ClickOnly,
                    ..Control::new(click_only, Rect::new(0, 6, 10, 1))
                },
            ],
            consume_keys: true,
            ..Stub::default()
        };
        s.layers
            .push((popover, vec![Control::new(B, Rect::new(0, 9, 10, 1))]));
        let (mut rt, mut buf) = runtime(s);

        // hover and Move still reach a disabled control
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Move, 22, 3));
        assert_eq!(rt.hover(), Some(disabled));
        assert_eq!(rt.app().count(disabled, "phase: Move"), 1);

        // and its scroll region still scrolls
        let _ = step(
            &mut rt,
            &mut buf,
            mouse(MouseKind::Wheel(crate::event::Axis::V, 1), 22, 3),
        );
        assert!(rt.app().saw(disabled, "Wheel { axis: V, delta: 3"));

        // a ClickOnly owner has no ring entry, so nothing proves it disabled
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 6));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 6));
        assert_eq!(rt.app().count(click_only, "phase: Press"), 1);
        assert_eq!(rt.app().count(click_only, "phase: Click"), 1);
        assert_eq!(rt.focus(), Some(A), "a ClickOnly owner took focus");

        // a press on the disabled control below an open popover is still an
        // outside click
        rt.app_mut().open_request = Some((
            popover,
            LayerSpec::popover(popover, crate::layer::Anchor::Point(Position::new(0, 9))),
        ));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(rt.is_open(popover));

        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 22, 3));

        assert!(!rt.is_open(popover));
        assert!(rt.app().saw(popover, "Dismissed(OutsideClick)"));
        assert_eq!(rt.app().count(disabled, "phase: Press"), 0);
    }

    #[test]
    fn hover_transition_requests_paint() {
        let (mut rt, mut buf) = runtime(three());

        let entered = step(&mut rt, &mut buf, mouse(MouseKind::Move, 3, 2));
        let stayed = step(&mut rt, &mut buf, mouse(MouseKind::Move, 4, 2));
        let left = step(&mut rt, &mut buf, mouse(MouseKind::Move, 30, 10));
        let stayed_out = step(&mut rt, &mut buf, mouse(MouseKind::Move, 31, 10));

        assert_eq!(entered.invalidate(), Invalidate::Paint);
        assert_eq!(stayed.invalidate(), Invalidate::None);
        assert_eq!(left.invalidate(), Invalidate::Paint);
        assert_eq!(stayed_out.invalidate(), Invalidate::None);

        let _ = step(&mut rt, &mut buf, key(KeyCode::Down));
        let suppressed_out = step(&mut rt, &mut buf, mouse(MouseKind::Move, 32, 10));
        assert_eq!(suppressed_out.invalidate(), Invalidate::None);
    }

    #[test]
    fn double_click_within_the_window_and_wheel_routing() {
        let mut s = three();
        s.page.push(Control {
            scroll: true,
            ..Control::new(Id::root("list"), Rect::new(20, 0, 10, 10))
        });
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 1, 0));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 1, 0));
        assert_eq!(rt.app().count(A, "DoubleClick"), 1);
        let _ = step(
            &mut rt,
            &mut buf,
            mouse(MouseKind::Wheel(crate::event::Axis::V, 1), 25, 5),
        );
        assert!(rt.app().saw(Id::root("list"), "Wheel { axis: V, delta: 3"));
        // a wheel over a non-scroll region is dropped, never chained
        let r = step(
            &mut rt,
            &mut buf,
            mouse(MouseKind::Wheel(crate::event::Axis::V, 1), 1, 0),
        );
        assert!(!r.is_consumed());
    }

    #[test]
    fn resize_releases_capture_and_relayouts() {
        let (mut rt, buf) = runtime(three());
        let r = rt.handle(Input::Resize(50, 20));
        assert_eq!(r.invalidate(), Invalidate::Layout);
        assert_eq!(rt.screen(), Rect::new(0, 0, 50, 20));
        let _ = buf;
    }

    #[test]
    fn focus_transition_settles() {
        // one programmatic hop per pass: A → B → C settles within the budget
        let mut s = three();
        s.chase = vec![C, B];
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert_eq!(rt.focus(), Some(C));
        assert!(
            rt.diagnostics()
                .iter()
                .all(|d| !matches!(d, Diagnostic::FocusTransitionDidNotSettle { .. }))
        );
        // FocusOut/FocusIn were enqueued per hop, never an input intent twice
        assert_eq!(rt.app().count(A, "Key("), 1);
        assert!(rt.app().saw(B, "FocusIn { via: Programmatic }"));
        assert!(rt.app().saw(C, "FocusIn { via: Programmatic }"));
    }

    #[test]
    fn a_fifth_focus_pass_is_diagnosed_and_applied() {
        let mut s = three();
        s.chase = vec![C, B, C, B, C, B, C];
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        assert!(
            rt.diagnostics()
                .iter()
                .any(|d| matches!(d, Diagnostic::FocusTransitionDidNotSettle { .. }))
        );
        let settled = rt.focus().expect("the give-up path applies the transition");
        // §21 item 11 says the pair is *applied*: the give-up path used to
        // enqueue `FocusOut`/`FocusIn` and then `intents.clear()` them, so
        // neither ever reached a component (MI-7). They are now delivered by
        // the next `handle`.
        rt.app_mut().chase.clear();
        rt.app_mut().log.clear();
        let _ = step(&mut rt, &mut buf, Input::Tick);
        assert!(
            rt.app().saw(settled, "FocusIn"),
            "the matching FocusIn must reach the new owner: {:?}",
            rt.app().log
        );
        assert!(
            rt.app().log.iter().any(|(_, s)| s.contains("FocusOut")),
            "the pending FocusOut must be delivered too: {:?}",
            rt.app().log
        );
    }

    #[test]
    fn undelivered_intents_are_diagnosed_only_for_control_owners() {
        let mut s = three();
        s.page.clear();
        s.page.push(Control::new(A, Rect::new(0, 0, 10, 1)));
        let (mut rt, mut buf) = runtime(s);
        // the stub drains everything; empty its control list so A stays registered last frame but undrained
        rt.app_mut().page.clear();
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 1, 0));
        assert!(
            rt.diagnostics()
                .iter()
                .any(|d| matches!(d, Diagnostic::UndeliveredIntent { owner } if *owner == A))
        );
    }

    #[test]
    fn paste_reaches_only_an_editing_owner() {
        let mut s = three();
        s.page[0].editor = true;
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, Input::Paste("hi".to_owned()));
        assert!(rt.app().saw(A, "Paste"));
        assert!(
            rt.app()
                .log
                .iter()
                .all(|(_, message)| !message.contains("hi")),
            "paste payload reached the runtime debug log: {:?}",
            rt.app().log
        );
        let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
        assert_eq!(rt.focus(), Some(B));
        let _ = step(&mut rt, &mut buf, Input::Paste("no".to_owned()));
        assert!(!rt.app().saw(B, "Paste"));
    }

    #[test]
    fn capture_keymap_runs_before_dispatch_and_skips_bare_chars_while_typing() {
        use crate::action::ActionKey;
        use crate::keymap::{KeyMap, KeyPhase};
        struct Mapped(Stub, KeyMap);
        impl App for Mapped {
            fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
                if let Some(cmd) = cx.command() {
                    self.0.log.push((KeyMap::OWNER, format!("cmd:{cmd:?}")));
                    return Response::consumed();
                }
                self.0.update(cx)
            }
            fn draw(&self, ui: &mut Ui<'_>) {
                self.0.draw(ui);
            }
            fn keymap(&self) -> &KeyMap {
                &self.1
            }
        }
        let mut s = three();
        s.page[0].editor = true;
        let km = KeyMap::new().bind(
            KeyPhase::Capture,
            crate::event::Chord::key(KeyCode::Char('q')),
            ActionKey::CLOSE,
        );
        let mut rt = Runtime::new(Mapped(s, km), Theme::junie());
        let mut buf = Buffer::empty(SCREEN);
        rt.draw_buffer(SCREEN, &mut buf);
        // the editor swallows typing: `q` reaches the editor, not the keymap
        let _ = rt.handle(key(KeyCode::Char('q')));
        assert!(rt.app().0.saw(A, "Char('q')"));
        assert!(!rt.app().0.saw(KeyMap::OWNER, "cmd"));
        rt.draw_buffer(SCREEN, &mut buf);
        let _ = rt.handle(key(KeyCode::Tab));
        rt.draw_buffer(SCREEN, &mut buf);
        let _ = rt.handle(key(KeyCode::Char('q')));
        assert!(rt.app().0.saw(KeyMap::OWNER, "cmd:ActionKey"));
    }

    #[test]
    fn layer_scope_is_armed_at_open_and_the_cursor_is_kept_on_the_top_layer() {
        let dlg = Id::root("dlg");
        let ok = Id::root("ok");
        let mut s = three();
        s.page[0].editor = true;
        s.layers.push((
            dlg,
            vec![Control {
                editor: true,
                ..Control::new(ok, Rect::new(5, 5, 5, 1))
            }],
        ));
        let (mut rt, mut buf) = runtime(s);
        assert_eq!(rt.cursor(), Some(Position::new(0, 0)));
        rt.app_mut().open_request = Some((dlg, LayerSpec::modal(dlg)));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Enter));
        let _ = step(&mut rt, &mut buf, Input::Tick);
        assert!(rt.is_open(dlg));
        assert_eq!(rt.top_layer(), LayerId(1));
        // focus reconciled into the trap; the page editor's cursor is silently discarded (inert)
        assert_eq!(rt.focus(), Some(ok));
        assert_eq!(rt.cursor(), Some(Position::new(5, 5)));
        assert!(
            rt.diagnostics()
                .iter()
                .all(|d| !matches!(d, Diagnostic::CursorRejected { .. }))
        );
        assert_eq!(rt.ring().reachable().count(), 1);
        assert!(rt.app().saw(dlg, "Layer(Opened)"));
    }

    /// §3.3 step 9: the dismissal of a layer whose owner registers only
    /// decoration — every `Dialog` (`components/dialog.rs` registers
    /// `Decorative` regions for its own id) — must still be reported when
    /// the owner does not drain it. This is the gated
    /// `if cx.is_open(id) { dialog.update(cx, …) }` shape: `is_open` is
    /// false by the time `update` re-runs, so the `Cancel` and
    /// `Layer(Dismissed)` the runtime addressed to the owner are dropped.
    /// Before the guard was widened this loss was **silent**, because
    /// `Registry::delivers_to` requires a `Control` or `Part` region.
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
        // the guard runs per *pass*, and the dismissal re-runs `update` while
        // the focus restore settles, so the owner is named once per pass; what
        // the invariant claims is that it is named, and that nobody else is.
        assert!(!undelivered.is_empty(), "the dismissal was lost in silence");
        assert!(
            undelivered.iter().all(|o| *o == dlg),
            "{:?}",
            rt.diagnostics()
        );
    }

    /// The other half of the same rule (§21 item 13): widening the guard for
    /// runtime-addressed intents must not start diagnosing a decorative
    /// container for a **pointer** intent. It cannot: `deliverable` never
    /// routes a pointer intent to a `Decorative` region, so the bucket is
    /// never created.
    #[test]
    fn a_decorative_owner_is_not_diagnosed_for_a_pointer_intent() {
        let decor = Id::root("decor");
        let mut s = Stub {
            page: vec![
                Control::new(A, Rect::new(0, 0, 10, 1)),
                Control {
                    decor: true,
                    ..Control::new(decor, Rect::new(0, 4, 10, 3))
                },
            ],
            ..Stub::default()
        };
        s.skip_drain = Some(decor);
        let (mut rt, mut buf) = runtime(s);
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Down, 3, 5));
        let _ = step(&mut rt, &mut buf, mouse(MouseKind::Up, 3, 5));
        assert!(
            rt.diagnostics()
                .iter()
                .all(|d| !matches!(d, Diagnostic::UndeliveredIntent { .. })),
            "{:?}",
            rt.diagnostics()
        );
        assert!(!rt.app().saw(decor, "Pointer"));
    }

    #[test]
    fn disabled_controls_are_registered_but_never_focused() {
        let mut s = three();
        s.page[1].focus = Focusability::Disabled;
        let (mut rt, mut buf) = runtime(s);
        assert!(rt.ring().is_registered(B));
        assert!(rt.state_of(B).contains(StateFlags::DISABLED));
        let _ = step(&mut rt, &mut buf, key(KeyCode::Tab));
        assert_eq!(rt.focus(), Some(C));
    }

    /// S1: a `Cx::focus` naming a `Disabled` entry is refused outright — not
    /// applied for one frame and then undone by step 14 reconciliation,
    /// which would deliver `FocusIn` to a control that can never be focused.
    #[test]
    fn a_programmatic_focus_request_for_a_disabled_control_is_refused() {
        let mut s = three();
        s.page[1].focus = Focusability::Disabled;
        s.focus_request = Some(B);
        let (mut rt, mut buf) = runtime(s);
        assert_eq!(rt.focus(), Some(A));

        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

        assert_eq!(rt.focus(), Some(A), "focus must not move to a disabled id");
        assert!(!rt.app().saw(B, "FocusIn"), "{:?}", rt.app().log);
        assert!(!rt.app().saw(A, "FocusOut"), "{:?}", rt.app().log);
    }

    /// S1: `ClickOnly` is "a hit target, never in the ring". Such an id is
    /// registered in the *registry* but has no ring entry, and that pair is
    /// exactly the proof a focus request naming it is wrong.
    #[test]
    fn a_programmatic_focus_request_for_a_click_only_control_is_refused() {
        let mut s = three();
        s.page[1].focus = Focusability::ClickOnly;
        s.focus_request = Some(B);
        let (mut rt, mut buf) = runtime(s);
        assert!(!rt.ring().is_registered(B));
        assert!(rt.registry().has_owner(B));

        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

        assert_eq!(rt.focus(), Some(A));
        assert!(!rt.app().saw(B, "FocusIn"), "{:?}", rt.app().log);
    }

    /// S1: opening a modal makes the page inert in the very same pass, while
    /// the page controls are still in *last* frame's ring. A focus request
    /// naming one of them is refused, so nothing behind the modal can take
    /// focus out of it.
    #[test]
    fn a_focus_request_for_a_control_the_open_modal_made_inert_is_refused() {
        const DLG: Id = Id::root("runtime.inert-dialog");
        const OK: Id = Id::root("runtime.inert-dialog.ok");
        let mut s = three();
        s.layers = vec![(DLG, vec![Control::new(OK, Rect::new(2, 2, 6, 1))])];
        let (mut rt, mut buf) = runtime(s);
        assert_eq!(rt.focus(), Some(A));

        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG)));
        rt.app_mut().focus_request = Some(B);
        let _ = step(&mut rt, &mut buf, key(KeyCode::Char('x')));

        assert!(rt.open_spec(DLG).is_some_and(|s| s.inert_below));
        assert_ne!(rt.focus(), Some(B), "an inert page control took focus");
        assert!(!rt.app().saw(B, "FocusIn"), "{:?}", rt.app().log);
        assert_eq!(
            rt.focus(),
            Some(OK),
            "step 14 reconciliation puts focus inside the modal instead"
        );
    }

    /// S1's counterpart: an id the last frame never saw is *unknown*, not
    /// proven unreachable, and must still be admitted — a layer's own
    /// controls are absent from the ring until they first draw, so
    /// `LayerSpec::initial_focus` names an unknown id by construction.
    #[test]
    fn initial_focus_into_a_layer_that_has_never_drawn_is_still_admitted() {
        const DLG: Id = Id::root("runtime.initial-focus-dialog");
        const OK: Id = Id::root("runtime.initial-focus-dialog.ok");
        let mut s = three();
        s.layers = vec![(DLG, vec![Control::new(OK, Rect::new(2, 2, 6, 1))])];
        let (mut rt, _buf) = runtime(s);
        assert!(!rt.ring().is_registered(OK));
        assert!(!rt.registry().has_owner(OK));

        rt.app_mut().open_request = Some((DLG, LayerSpec::modal(DLG).initial_focus(OK)));
        let _ = rt.handle(key(KeyCode::Char('x')));

        assert_eq!(
            rt.focus(),
            Some(OK),
            "the named target holds focus before the layer has ever drawn"
        );
    }
}
