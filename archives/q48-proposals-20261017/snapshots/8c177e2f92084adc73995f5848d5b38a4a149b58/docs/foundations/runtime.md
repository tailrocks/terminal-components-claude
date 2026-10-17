# Runtime, frame phases, focus, capture, and time

**Foundation F03 · canonical owner of dispatch lifecycle, published geometry, focus, pointer capture, hover feedback, and supplied time**

**Legacy families:** C03, C04.

This page defines the generic Termrock runtime boundary. It does not define a Jackin router, executor, service layer, async runtime, dependency-injection container, or product event loop. It supplies the small driver needed by this repository's future library/conformance consumers while preserving the observable behavior of the frozen applications.

This page specifies the target runtime contract for a future in-place refactor on `termrock-implementation`; it does not describe an implemented runtime. The branch starts from baseline commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`, whose current source names are implementation evidence only: [`src/core/focus.rs`](../../src/core/focus.rs), [`src/core/hit.rs`](../../src/core/hit.rs), [`src/ui/ctx.rs`](../../src/ui/ctx.rs), and [`src/runtime.rs`](../../src/runtime.rs) expose `WidgetId`, `Focus`, `FocusRing`, `HitRegistry`, `Interaction`, `RenderCtx`, and an `Application`/`TerminalSession` loop. The target public API uses `Scene`, `Runtime`, `Cx`, `Ui`, `MeasureCx`, `Moment`, and explicit update/draw phases. No legacy name is a required future alias.

## Source and oracle references

- Migration input: `termrock-library-spec/foundations/runtime.md`, the F03 entry in `termrock-library-spec/reference/foundations.json`, and the runtime/ownership sections of `termrock-library-spec/SPECIFICATION.md`, `PUBLIC-API.md`, and `reference/TYPES.md`.
- Frozen source: [`src/core/focus.rs`](../../src/core/focus.rs), [`src/core/hit.rs`](../../src/core/hit.rs), [`src/ui/ctx.rs`](../../src/ui/ctx.rs), and [`src/runtime.rs`](../../src/runtime.rs) at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`.
- Identity and source reconciliation: [`identity.md`](identity.md).
- Input/response contract: [`input-actions.md`](input-actions.md).
- Layer barriers and restoration: [`layers.md`](layers.md).
- Layout publication: [`../foundations/layout.md`](../foundations/layout.md).
- Visual and interaction evidence: [`../verification/visual-parity.md`](../verification/visual-parity.md), [`../verification/interaction-parity.md`](../verification/interaction-parity.md), and [`../verification/oracle-and-provenance.md`](../verification/oracle-and-provenance.md).

## Target runtime surface

```rust
trait Scene {
    fn update(&mut self, cx: &mut Cx<'_>, cause: UpdateCause);
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
}

Runtime::new(scene: A, theme: Theme) -> Runtime<A> where A: Scene
Runtime::handle(
    &mut self,
    cause: UpdateCause,
    moment: Moment,
) -> Result<UpdateReport, RuntimeError>
Runtime::draw(&mut self, area: Rect) -> Result<PaintedFrame, RuntimeError>
Runtime::presented(&mut self, token: FrameToken) -> Result<(), RuntimeError>

Moment::from_millis(value: u64) -> Moment
AnimationSample::phase(index: u64) -> AnimationSample
AnimationSample::timed(
    now: Moment,
    epoch: Moment,
    cadence: Duration,
    policy: MotionPolicy,
) -> Result<AnimationSample, ClockError>
```

`Scene` is a small generic consumer adapter. It is not a universal dynamic `Widget` trait, application router, business-state container, or alternate product framework. `UpdateCause` includes boot, input, tick, model change, and resize; every cause carries a supplied monotonic `Moment`. The shared shapes and error vocabulary are defined in [`../api/types.md`](../api/types.md).

## Ownership and phase flow

The runtime processes a frame in this order:

1. **Boot/reconcile.** Initialize caller-owned component state and reconcile current source revisions before any coordinate-dependent dispatch.
2. **Measure/layout.** Components measure from borrowed props and constraints. Geometry is derived from the current viewport; rectangles do not live in durable component state.
3. **Update.** Resolve the highest applicable layer/scope, offer normalized input through `Cx`, update durable state, and collect typed response/runtime requests. Semantic changes happen here.
4. **Draw.** Read-only component state and models are painted through `Ui`. `Ui` may emit cell writes, cursor intent, current geometry, hit regions, focus registrations, and layout facts; it may not commit semantic state, select an item, close a layer, execute a command, read a clock/environment variable, or do IO.
5. **Publish.** Commit the complete geometry/hit/focus snapshot transactionally. A failed draw or registration discards the candidate snapshot rather than leaving half-new regions.
6. **Present.** Emit the frame and retain a `FrameToken`. A presentation acknowledgement allows activation feedback to remain visible before a later queued event erases it.
7. **Dispatch.** Coordinate-dependent pointer input uses the last committed valid geometry. A resize or topology change invalidates that geometry and requires a fresh measure/draw/publish before pointer dispatch resumes.

`MeasureCx` is read-only. `Cx` is the update context for owner-resolved intents and runtime requests. `Ui` is a constrained draw/publication context. These names replace the current `RenderCtx` concept; `RenderCtx` may remain private during migration but is not a target public compatibility surface.

There is no unbounded update/draw stabilization loop. If draw emits facts that require reconciliation, the runtime schedules one explicit update before the next eligible input/frame and reports the invalidation. Repeated draw with unchanged inputs is observably idempotent.

## Focus ownership

Focus is a runtime-owned control identity, distinct from active selection, current keyed item, edit phase, and cursor position.

- The focus ring is rebuilt from visible enabled controls in render order. Tab order therefore follows deterministic reading/render order.
- Disabled, removed, invisible, or inert controls are not reachable.
- A modal layer pushes a focus barrier; only controls registered within that layer are reachable while it is open.
- Opening a layer saves the surviving prior focus owner. Closing restores it if still present and eligible; otherwise choose the layer's documented fallback or `None`.
- Focus visibility may be hidden behind an overlay, but hidden focus must not silently activate content below it.
- Components report focus identity and render state; they do not maintain parallel `owns()`/`locate()` focus engines.

Focus repair happens during update/reconcile, not as an incidental mutation in draw. Stable `Id`/`ItemKey` rules are owned by [`identity.md`](identity.md).

## Geometry, hit testing, and pointer capture

Paint regions and hit regions share one frame publication source. Registration is last-writer/topmost aware, includes explicit scroll-only regions, and is clipped to the current viewport. Inert lower layers remain blocked when an overlay requires a barrier. Invisible or removed owners cannot retain focus or capture.

Pointer gestures are runtime state:

1. `Down` identifies an eligible target and may establish capture.
2. `Drag` routes to the captured target where its contract permits, even when the pointer leaves the original rectangle.
3. `Up` revalidates stable identity, current eligibility, and release location before emitting a typed action.
4. Release outside, a deleted target, disabled/read-only ineligible state, or a closed layer cancels activation and releases capture.

Scrollbar drags, split-pane drags, text selection, double-click recognition, and press/activation feedback use the same runtime capture lifecycle. A completed click is not emitted on `Down`. Outside dismissal of a layer is one gesture and cannot fall through to the content below.

Keyboard input suppresses stale hover feedback until real pointer motion resumes. `Interaction::pressed`/activation feedback is derived from held capture or a presentation-acknowledged activation phase; it is not a semantic state hidden inside a component draw callback.

## Time and animation policy

Time is supplied, monotonic, and checked. Drawing never advances time and no animation uses draw count as a clock. Glyph phase and elapsed interaction feedback are separate inputs. `MotionPolicy` supports full, reduced, and paused motion; pausing a spinner must not pause externally supplied model updates.

The runtime uses the activation duration and presentation-order rule defined
by the [interaction contract](../design/interaction-contract.md). Spinner
phase, progress changes, hover, and activation feedback do not share one
implicit wall-clock cadence.

The optional terminal/session adapter owns raw mode, alternate screen, mouse capture, bracketed paste, cursor visibility, line wrap, and supported job-control restoration. Those concerns are an edge adapter, not runtime semantic state; see [`session.md`](session.md) when the parent documentation migrates that contract.

## Purity and failure rules

The runtime must fail closed on duplicate identity, stale geometry, invalid capture owner, nonmonotonic time, invalid frame token, and failed transactional publication. Typed errors use safe descriptions and never include secret values.

Immutable Rust signatures help but do not fully prove draw purity: interior mutability and arbitrary callbacks can hide writes. Conformance therefore combines constrained `Ui`/slot APIs, state-before/after assertions, callback review, and draw-twice tests. Runtime code must not expose raw registries to components or allow callbacks to retain a `'static` mutable model/executor handle.

## Required proof and parity coverage

The implementation and conformance suites must include:

- boot with no geometry refusing pointer dispatch;
- resize during drag and disappearance of the captured row releasing capture without retargeting;
- focus + hover suppression after keyboard input, followed by restoration after pointer motion;
- focus barrier and restoration after modal close when the prior target was removed;
- input flood bounded by a drain budget so animation/presentation deadlines are not starved;
- draw twice leaving semantic state, source revision, focus, capture, and layer ownership unchanged;
- nonmonotonic time rejection and paused glyph phase still permitting model/data updates;
- delayed presentation acknowledgement preserving the activation frame and
  its interaction-contract timing;
- topmost hit ownership, inert lower-layer blocking, and transactional publication after registration failure;
- exact observation under the comparison rules in the
  [verification contracts](../verification/README.md).

Visible focus, hover, pressed, cursor, scrollbar, fade, resize, and modal outcomes are compared through the frozen application fixtures. This foundation does not claim an independent screenshot oracle. `ExistingOracle`, `ExtractedOracle`, and `Extension` remain separate verification lanes.

## Implementation acceptance

P1 is complete only when a generic Scene/Runtime path can boot, update, measure, draw, publish, present, and dispatch against caller-owned state; runtime owns focus/hit/capture/time; stale geometry and invalid captures fail closed; and draw is semantically read-only. The four frozen applications remain unchanged reference consumers throughout later phases. No runtime task may add Jackin services, Holla/TablePro product behavior, or a new repository/application framework.
