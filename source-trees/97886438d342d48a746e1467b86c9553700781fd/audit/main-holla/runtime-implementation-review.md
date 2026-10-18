# Review of runtime-proposal.md after focus integration

Read-only review against integrated focus source and full docs/audit/main-holla/runtime-proposal.md. No repository edits. The proposal preserves required invariants, but its independently movable frame token, runtime identity/error checks and fully bundled migration are larger than necessary.

## 1. Replace identity-bearing tokens with exclusive publication guards

```rust
#[must_use = "commit only after successful presentation; dropping aborts"]
pub struct PaintedFrame<'r, A: App> { runtime: &'r mut Runtime<A> }
impl<A: App> Runtime<A> {
    pub fn paint<'r>(&'r mut self, frame: &mut Frame<'_>) -> PaintedFrame<'r, A>;
    // testing-only counterpart returns same guard after painting a bare buffer
}
impl<A: App> PaintedFrame<'_, A> {
    pub fn commit_presented(self); // commits only its originating runtime
}
```

Painting writes proposed FrameState, not LastFrame. Guard owns exclusive runtime borrow; it cannot be passed to another runtime, cloned or used twice. Runtime cannot be moved/mutated/painted while guard is live. No global atomic, counter uniqueness, pointer identity, Rc/Arc, per-frame allocation, or runtime-ID overflow policy needed. Drop leaves LastFrame unchanged and presentation-invalid flag set; next paint resets pending FrameState. Even mem::forget cannot grant valid presentation: mark invalid before paint and clear only on commit. A terminal error drops guard; old frame remains historical, but input is blocked until successful new publication because physical output may be partial.

Terminal adapter can retain guard through transport success:

```rust
let mut painted = None;
let slot = &mut painted;
let runtime = &mut rt;
terminal.draw(move |frame| *slot = Some(runtime.paint(frame)))?;
if let Some(frame) = painted { frame.commit_presented(); }
```

Local ratatui-core0.1.2 src/terminal/render.rs:81–83 declares FnOnce. External guard_borrow_probe.rs compiled with rustc edition2024 proves the lifetime shape. This is a borrowing proof, not yet a real Terminal integration test. The production driver should encode the callback's guaranteed paint (the Option above is adapter glue) and explicitly fail if a wrapper promises success without invoking it. Canonical headless rendering acknowledges its completed buffer through the same guard.

Errors shrink: no WrongRuntime/StaleToken FrameError because these states become compile failures. Retain tests as compile-fail mappings plus successful transport/failed transport behavioral tests. Runtime frame validity and input preconditions remain necessary after guard drop and later app mutation.

## 2. Keep explicit time, simplify staged introduction

Moment(Duration), Cx::now, absolute next deadline, advance_to and separate Tick cause are appropriate. Prefer name advance_to for explicit monotonic time; equal timestamp legal, backward timestamp typed error before mutation. Clock progression never depends on events, app routing or number of paints. Remove Input::Tick only when boundary/event owner releases those files and consumers migrate; do not retain a production compatibility Tick that advances arbitrary milliseconds. Existing test ticks may be renamed frame_steps with explicit cadence and mapped per fixture, not exposed as normal Runtime clock semantics.

initialize(now) performs exactly one Bootstrap; repeated initialize may be no-op only at same time or documented independent of time. Cleaner split initialize() at current now plus advance_to(now): avoids ambiguous repeated initialize changing clock. Final recommendation `initialize(&mut self)->Response<()>` and `advance_to(Moment)->Result<Response<()>,TimeError>`; runtime starts ZERO. Live initializes at epoch0. Fixtures initialize before advancing their explicit fixture timestamp, or pure scene never initializes. Keep caller-facing duration helpers only if overflow cannot produce early deadline.

## 3. app_mut and publication invalidation

Keep current app_mut()->&mut A, mark frame invalid BEFORE returning borrow. No mutation guard needed: conservative invalidation is correct even for a read via app_mut. set_theme invalidates caches/presentation too. Every update execution invalidates presented compatibility regardless of Response because ignored responses do not prove unchanged geometry. Clock movement invalidates only when it invokes update or changes runtime-visible feedback; reading Cx time only occurs during update. Deferred focus/layer lifecycle changes also invalidate until reflected by a committed frame.

Do not mark every settle() invocation invalid unconditionally: a no-pending-work settle must be no-op or driver loops forever. Bound each actual focus transition with existing diagnostics; process pending focus/layer notifications once, request new frame only for an actual update/state change. Once focus stabilizes, repeated paint+commit cannot generate another lifecycle update.

PendingInput owns original event and redacts paste in Debug/errors. No runtime-owned unbounded queue. First key/paste/mouse is returned pending until initialized and presented. After a pending input, driver initializes/presents/settles then retries once valid. Route/resize batches process one event per committed geometry. No hidden draw inside handle.

## 4. Pure production Scene with minimal new surface

Runtime::new remains effect-free. paint/paint_buffer/Scene::draw never initialize. Existing Scene's NoApp closure path already builds shared Ui painter; remove bootstrap from that path rather than introducing a second rendering engine. Add `Scene::draw_app(&impl App)` convenience delegating to existing closure as `app.draw(ui)`; domain model stays borrowed and is not cloned. App effects counters and semantic state compared before first render and after repeated draws.

Layer-dependent pure fixtures require explicit test-only render state (layers/specs/focus/hover/press/capture/cursor inputs). A public library testing-feature RenderSnapshot builder can seed render state WITHOUT calling App::update/open-layer domain code; validate duplicates and modal scopes in the same layer/Ui pipeline. Keep secret-bearing app state outside Debug of that snapshot. Prefer fixture seed builders in tui-testing over exposing FrameServices internals. Do not claim production modal pure coverage until these seed builders exist and real modal view tests use them.

## 5. Dependency-ordered implementation slices

A. **Explicit initialization + pure painter prerequisite.** Add initialize(), remove ensure_bootstrap from both live and bare painting; live adapter and Harness explicitly initialize. Scene never initializes. Central runtime stub helper gets initialize; update raw Runtime tests/examples explicitly. Add first paint/update/effect invariance and exactly-once bootstrap/settle tests. No time behavior change yet; remaining event clock explicitly tracked. This is smallest next architectural slice, but requires coordinated ownership of session.rs,harness.rs,digest.rs plus test-only raw Runtime callers; runtime.rs/cx.rs alone cannot land coherent migration.

B. **Publication guard + input compatibility.** Split existing draw frame swap into paint and commit; exclusive guard; app_mut/set_theme/update invalidation; pending-input API; terminal and Harness acknowledge success and process one event per frame; pending focus settle explicit. Add transport failure, queued resize/route, first-paste, dropped guard, compile-fail wrong-runtime/alias cases. Run library/component conformance plus all app behavioral smoke before retirement of old draw APIs.

C. **Explicit time + actual scheduler.** Coordinate event/lib exports; Moment/Cx now/advance_to/absolute request storage; runtime flash deadline + activation request; terminal Instant epoch and earliest-deadline poll; Harness explicit clock. Migrate Showcase Buttons first with2199/2200/global-status proof, then Jackin/Holla simulation adapters and all remaining timers. Keep production simulation effects fake. Compatibility event-count paths removed in same slice, fixture mappings retained as explicit model construction.

D. **Committed interaction reconciliation.** Re-hit stationary pointer; cancel removed/disabled/inert capture and press; ensure top nested modal blocks background input even before controls appear. This can be parallel test characterization, but shared writer only after B.

Per-slice tests remain requirement-identity mapped; no baseline approval or glyph/style expectation changes. Some public signature changes touch examples and xtask exact signatures: update actual scanner to meaningful boundary, never append meaningless calls merely to pass.

## 6. Focus admissibility timing and first-candidate concern

In integrated runtime step13 swaps NEW registry/layout/declarations/bindings before step14 computes reconciliation. step14 calls new frame.ring.next/reconcile while old last.ring remains available for nearest-survivor historical ordering; then swaps ring. Finally focus_target_admissible reads NEW last.ring AND NEW last.registry. There is no old/new registry mismatch at that final check.

For valid unique control IDs registered through public Ui, an item from new ring.reachable is necessarily admissible:

- reachable already filters disabled and innermost trap;
- Ui::register_entry/register_focus_only skip inert/reference subtrees before ring insertion;
- Ui::new sets page inert from inert_floor; Ui::layer sets each layer inert from same floor;
- ring registration therefore cannot contain a live entry below inert_floor;
- candidate has a ring entry, so unknown-owner/no-ring registry branch is irrelevant.

Nested modals: only top modal controls register below-inert suppression; highest-layer trap is active. A disabled first top-modal control is excluded by reachable; next enabled top-modal control is selected. Absent/zero-size top modal leaves no reachable entry and legitimately yields None, with layer barrier still required independently.

Thus filter cannot discard the first candidate while a valid alternative exists under the declared uniqueness/inert registration invariants. HOWEVER duplicate IDs are diagnosed rather than rejected: a disabled first duplicate plus enabled second duplicate can make reachable yield that Id while entry(id) finds disabled first, so filter drops to None despite later valid distinct control. This is malformed registration but demonstrates why filter-after-selection is brittle. Recommend a separate bounded hardening: integrate admissibility predicate into candidate traversal/reconciliation, or reject duplicate focus owner registration consistently with hit registry policy; do not silently fallback across modal scopes. Need adversarial duplicate test and nested-modal enabled-alternative test before claiming universal robustness. No source edit made here; current valid-input proof is source-based, not newly executed nested test evidence.
