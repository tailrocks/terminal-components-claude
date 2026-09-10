# Proposed runtime API — explicit time and presented-frame lifecycle

Read-only design, not approved implementation. References: CURRENT-CONTRACT.md items1–5/8/11; REFACTORING_STATE.md:1548–1618 historical §54 addendum;1846–2030 checkpoint and residual evidence. Current source inspected at MAIN_BASE. Historical checkpoint counts are not current test evidence.

## Decisions

Keep App::update(&mut self,&mut Cx)->Response<()> and App::draw(&self,&mut Ui). Keep caller state/borrowed props, typed intents, layers and backend-free Runtime. Replace hidden bootstrap and event-count time; separate painting from successful presentation. Runtime owns time/deadlines/feedback; terminal supplies monotonic timestamps and acknowledges successful output. Test harness supplies explicit timestamps and acknowledges its buffer as presented. No backend wall-clock reads in Runtime, Cx, components or deterministic Scene.

Use a small elapsed-time newtype, not std::time::Instant in public core:

```rust
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Moment(Duration); // private, relative to the runtime's epoch
impl Moment {
    pub const ZERO: Self;
    pub const fn from_millis(ms: u64) -> Self;
    pub const fn as_duration(self) -> Duration;
    pub fn checked_add(self, delta: Duration) -> Option<Self>;
    pub fn saturating_duration_since(self, before: Self) -> Duration;
}
impl Cx<'_> {
    pub const fn now(&self) -> Moment;
    pub fn request_repaint_at(&mut self, at: Moment);
    pub fn request_repaint_after(&mut self, delay: Duration); // now + delay
    pub fn activation_feedback(&mut self, owner: Id);
}
```

Deadline storage is absolute Moment, not duration. Preserve min(existing,new); only consume a due deadline, not every Tick. Equal duration requested later must not alter the older timestamp. Reject unrepresentable addition explicitly or saturate to Moment::MAX consistently (never turn overflow into immediate wakeup). Prefer Duration storage to avoid millisecond precision loss; integer millis conversion only at documented app adapter boundaries.

Runtime::next_deadline()->Option<Moment> includes min(app repaint, activation flash expiry, other runtime animation obligations). Clock movement expires flash and requests repaint exactly when visible flags change. Repaint request is separate from next timer. Remove wants_tick cadence inference. Poll can sleep until actual deadline; no deadline means no timer delivery. OS delay means next update sees actual elapsed time, not one fictitious cadence step.

## Lifecycle and publication signatures

```rust
impl<A: App> Runtime<A> {
    pub fn new(app: A, theme: Theme) -> Self; // no update, no effects, now ZERO
    pub fn initialize(&mut self, now: Moment) -> Result<Response<()>, TimeError>;
    pub fn advance_to(&mut self, now: Moment) -> Result<Response<()>, TimeError>;
    pub fn handle(&mut self, input: Input) -> Result<Response<()>, PendingInput>;
    pub fn now(&self) -> Moment;
    pub fn next_deadline(&self) -> Option<Moment>;
    pub fn needs_present(&self) -> bool;
    pub fn draw(&mut self, frame: &mut Frame<'_>) -> PaintedFrame;
    pub fn commit_presented(&mut self, frame: PaintedFrame) -> Result<(), FrameError>;
    pub fn settle(&mut self) -> Response<()>;
}
pub struct PaintedFrame { /* private generation + viewport + runtime epoch */ }
pub struct PendingInput { /* owns original Input, reason + into_input() */ }
pub enum PendingReason { Uninitialized, NeedsPresentation, NeedsSettle }
pub enum TimeError { WentBackwards { previous: Moment, supplied: Moment } }
```

Exact error names can follow repository conventions; invariants are mandatory. PaintedFrame is non-Clone and must not contain borrowed buffer data; commit verifies runtime epoch, newest generation, viewport, and unchanged semantic revision. A token rendered by another Runtime or before app mutation cannot publish. No buffer means no geometry; draw computes pending facts, commit alone swaps authoritative registry/ring/cursor. Backend failure leaves prior frame authoritative and cannot commit the failed frame. This fixes the existing swap-before-terminal-write behavior. Cursor passed to Frame is candidate cursor; published cursor becomes inspectable after commit.

initialize is explicit and idempotent: one Bootstrap update, zero clock advancement beyond supplied epoch, no input intents. It may open product layers/schedule animation through ordinary update. Called by live driver and behavioral Harness, never by pure view capture. Calling draw before initialize is legal pure painting and performs zero update calls. handle before initialize returns owned pending input; no silent dropped first paste/click. A live caller initializes, renders+commits+settles, and retries retained first input.

advance_to rejects backwards time before mutation. Equal time is legal. It updates Runtime's clock; if app deadline due, runs exactly one Tick update at the supplied actual time, followed by ordinary focus Settle reruns only. It never synthesizes N app ticks from elapsed milliseconds. Product models decide how to integrate elapsed interval; runtime animation/flash expiry is processed here. No due app deadline means no app Tick; flash-only expiry only invalidates runtime paint. initialize also rejects invalid temporal ordering. Input::Tick is removed from external event vocabulary: timer delivery is advance_to, not an unparameterized event. This deliberately breaks experimental API to prevent a second event-count clock.

settle explicitly processes pending focus/layer lifecycle from presentation; draw/commit never call App::update. Reconciliation can stage a focus change at commit; settle applies it and emits FocusOut/FocusIn using UpdateCause::Settle. Driver renders again if focus presentation changes. Bound settling with existing diagnostics; never advance time or duplicate Tick. A pure Scene may choose explicit interaction state and render without running settle at all.

## Queued input contract

Conservative, reliable first implementation: after every App::update execution or app_mut access, mark prior facts incompatible until a fresh frame is successfully presented and lifecycle settled. Response::Paint versus Layout remains meaningful scheduling metadata, but cannot be the sole correctness guard: app code can return ignored while changing route, and current code does. Registration-equivalence optimizations may follow evidence; initial implementation does not infer semantic purity from Response.

handle consumes input only with initialized, presented, settled facts. Otherwise returns PendingInput retaining owned paste and exact key/mouse bytes. It never routes against invalid data. Runtime does not accumulate unbounded secret-bearing queue; live driver retains at most current retry and leaves later events in backend queue. Errors/debug must redact paste. Terminal driver processes one event, presents/settles before next event, rather than draining ready events over one stale frame. Harness default uses same bounded drive algorithm. Manual auto_draw(false) tests must explicitly render+commit before another event or assert PendingInput.

Resize carries viewport invalidation before subsequent input; resize itself is processed through update, releases press/capture, invalidates caches, and requests new viewport presentation. After successful new frame, next pointer routes exactly its geometry. Route changes similarly require presentation. Removed owner/part cancels press/capture; no click on retained owner with vanished part. Dynamic disable during capture cancels capture/press without activation; optional semantic cancel intent lets component discard drag state. New modal blocks prior capture immediately and preserves no background wheel/key/drag delivery.

Persist last physical pointer position; commit re-hit-tests it against new live geometry while honoring keyboard hover suppression. Update hover owner AND PartRef even when owner survives. Layout/hit/focus/scroll/cursor facts all originate from the same component layout. A second application overpaint is not repaired by runtime alone.

## Live and pure-view paths

Live run: construct Runtime; initialize(epoch ZERO); paint via terminal.draw; on success commit token; settle; repeat paint until settled; advance_to(start.elapsed()); if due update invalidated facts present; read one Input; handle/retry after publication; repeat. Check time before every backend wait and between input iterations, preventing busy input from starving timers. Wait duration=max(0,next_deadline-now); no arbitrary idle Tick. If a deadline is already due after expensive render, deliver immediately. Bounded settling/deadline loops diagnose zero-delay request loops without fabricating progress.

Behavior Harness: same lifecycle/commit algorithm with a fake monotonic epoch. Add advance_to(Moment), advance_by(Duration), now(). Input helpers keep time unchanged. ticks(n) may temporarily live only as a test migration adapter explicitly equivalent to advance_by(n * named fixture cadence), never production API; remove after mapping all tests. For real-time-sensitive semantics use exact timestamps rather than retained tick counts.

Pure Scene: directly paints App::draw or production view closure from explicit fixture app/state/theme/viewport/runtime interaction snapshot/clock, without initialize/advance_to/settle. Runtime clock in Cx is update-only; visual animation phase is explicit immutable fixture/model prop. Existing draw_scene should become a pure projection that cannot accidentally invoke app update. Open layers for fixtures need explicit immutable render-layer snapshot (ids/specs/scope/interaction state), not synthetic bootstrap that executes product logic. Preferred implement Scene in testing boundary over the same Ui frame renderer with owned RenderState snapshot; do not introduce a second painter. Repeated render verifies complete app/domain/effect state plus canonical cells/registry/cursor. A callback with interior mutability still can violate purity; effect/model guards detect it, static shared-reference scans cannot prove it.

## Caller and test migration inventory

Machine-readable timing-lifecycle-callers.json lists750 current source occurrences, path+line+text. It is a migration lead inventory, not750 required edits or executed tests. Re-scan after changes; classify each occurrence and require surviving obligation mapping.

Core owner: runtime.rs, runtime/session.rs, event.rs, ui/cx.rs, ui/mod.rs/frame storage, public lib exports, diagnostics and testing inspection. Cx constructors receive immutable Moment. Components/button.rs registers semantic activation feedback on keyboard AND pointer activation; other activatable families use same request. No application-owned flash timer. Component unit tests constructing Runtime/draw_buffer migrate initialization/commit explicitly; do not leave a wrapper implicitly updating inside draw.

Testing owner: tui-testing/harness.rs, digest.rs, conformance/driver.rs; runtime stub::runtime/step; existing draw_scene/draw_buffer callers in component modules; perf.rs/grid_model_contract.rs; external docs/examples/compile checks. Central Harness change covers most apps tests but raw Runtime callers listed in JSON require direct migration. Pure Scene tests must not silently inherit Harness bootstrap.

Showcase owner: pages/buttons.rs replaces busy_frames with started/deadline Moment and status completion exactly once; app-level status source preserves global feedback. pages/progress.rs and taskrunner.rs currently own cadence/draw or event-derived animation counters: derive phase from explicit time and schedule next phase. Inspect remaining all22 pages for timers even when grep does not name Input::Tick.

Jackin owner: app.rs route_tick_ms, tick_simulation/UpdateCause::Tick, world.clock, rain/arbiter/launch/account effects. Preserve named fixture frame n as explicit fixture time mapping (n*route cadence) independent of live runtime input count. Live simulation advances by actual elapsed delta since last model step, with paused motion affecting animation policy only where reference contract specifies. A deterministic World clock may remain model-owned as derived simulation time, not a separate scheduling authority. Historical §54.4 prohibition on using runtime/wall delta is explicitly superseded for live elapsed semantics; preserve exact fixture output and sequence boundaries through fixture constructor adapter and reference comparisons.

TablePro owner: any Instant::now/status expiry/edit flash uses Cx time/runtime feedback; keyed domain edit actions remain update-only. Holla new migration: simulation clock maps from runtime time with fixture offsets; fake provider/git/docker/ssh actions remain fake. Existing holla reference source stays immutable.

## Required surviving and new tests

- bootstrap_runs_once_before_first_draw_without_a_tick -> explicit_initialize_runs_once_before_live_input; additionally first_draw_does_not_initialize_or_emit_effects.
- headless_tick_uses_the_same_update_cause_without_wall_clock -> explicit_time_drives_same_tick_cause_without_wall_clock; preserve determinism, replace fixed tick-step assertion.
- tick_cause_is_delivered_once_when_focus_settles -> keep intent with advance_to due deadline; only first update Tick, reruns Settle.
- repaint_deadline_survives_unrelated_input_and_keeps_the_earliest -> absolute40ms established at0 remains40 after input at35 requesting10ms; requested45 cannot delay40. Also later requested earlier absolute deadline takes precedence.
- Focus/capture/layer/hit tests retain all identities and assertions; common helper renders/commits before next input. Add deliberately unpresented resize/route test expecting PendingInput then correct target after real presentation.
- flash_without_app_animation_wakes_and_clears; keyboard_mouse_feedback_same_duration; disable_or_modal_cancels_capture; stationary_pointer_retargets_after_layout.
- Buttons2199/2200,1000events_at_same_time,one_large_elapsed_jump,draw1000times,due_while_input_queue_busy; completion count exactly1 and global status repaint asserted.
- first_render_no_domain_effects + validation/edit/layer fixture invariance; mutation introduces draw-time staging and fails.
- failed_terminal_present_cannot_publish_geometry; wrong_runtime_token/stale_token rejected; first_paste_delivered_after_initial_editor_present; zero_size_modal_blocks_input.
- queued terminal resize+click, route key+click, mouse-down+modal+drag/release in actual binary, not only synchronous Harness.

## Review risks to resolve before implementation

Token commit adds API surface but closes backend failure and stale-publication class; borrowing a Frame alone cannot prove terminal output succeeded. Conservative per-update presentation has real work implications: run meaningful allocation/perf gates, do not relax thresholds. Preserve a no-semantic-effect pure scene path without exposing secret-bearing runtime snapshots to Debug/artifacts. Define reduced versus paused simulation behavior from pinned reference per app; do not universally freeze domain time because motion is paused. All proposal signatures need public-consumer compilation/MSRV tests; no implementation or pass is claimed here.

## Deferred focus traversal for navigation-to-content

Add `Cx::focus_next()` and `Cx::focus_previous()` as semantic requests, with optional explicit `Cx::focus_after(owner: Id)` for custom component authors. `focus_next()` captures the current logical focus owner as its anchor at request time. Store a runtime-owned focus request variant rather than calculating an Id from the previous ring. Resolve after the newly presented frame supplies the destination route's ring, then apply through the same admissibility/trap/disabled checks as direct focus. No app ring inspection or page-index IDs.

Direct `focus(id)` still targets a known logical control, admitting not-yet-drawn layer controls per current contract; a next/previous request deliberately waits for next successful publication. Last request wins within an update pass, consistent with existing focus_request behavior. Opening a modal has precedence through the live trap/inert-floor restrictions; traversal can never enter background. If anchor vanished, use the existing nearest-survivor reconciliation policy within active scope, falling back to first/last admissible entry; absent controls yield None. Preserve explicit request through failed presentation/stale-token rejection; consume exactly once on successful resolution. Focus notifications run in explicit settle; repeat draws alone never deliver actions.

Showcase `NavListAction::Chose(key)` changes selected page only. `EnterContent` asks `cx.focus_next()` after route intent is applied; current sidebar owner remains the stable anchor. The new page's first reachable content control follows sidebar in presented reading order; this ordering must be tested from reference geometry. Page-specific enum-to-control-ID tables are unnecessary. If shell ordering intentionally places another global control after sidebar, composition should define a content focus scope and use a generic `focus_scope_first(ScopeId)` request instead; do not special-case the runtime for Showcase.

Tests: choose page while sidebar retains focus; EnterContent focuses the new page's first enabled control; route switch and EnterContent in same update; missing/disabled first content; clipped/zero-size content; nested modal barrier; request survives failed frame; repeated draw does not duplicate FocusIn; no old page activation before new presentation.
