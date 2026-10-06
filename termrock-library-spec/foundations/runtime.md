# F03 · Frame driver, focus, pointer capture and time

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C03, C04  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/core/focus.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/focus.rs)
- [src/core/hit.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/hit.rs)
- [src/ui/ctx.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/ctx.rs)
- [src/runtime.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/runtime.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
trait Scene {
    fn update(&mut self, cx: &mut Cx<'_>, cause: UpdateCause);
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
}
Runtime::new(scene: A, theme: Theme) -> Runtime<A> where A: Scene
Runtime::handle(&mut self, cause: UpdateCause, moment: Moment) -> Result<UpdateReport, RuntimeError>
Runtime::draw(&mut self, area: Rect) -> Result<PaintedFrame, RuntimeError>
Runtime::presented(&mut self, token: FrameToken) -> Result<(), RuntimeError>
Moment::from_millis(value: u64) -> Moment
AnimationSample::phase(index: u64) -> AnimationSample
AnimationSample::timed(now: Moment, epoch: Moment, cadence: Duration, policy: MotionPolicy) -> Result<AnimationSample, ClockError>
```

## Contract

1. Scene is only a minimal update/draw adapter for a gallery or future consumer. It is not a Jackin app, router, executor framework, async runtime, DI container or service layer. No navigation tree or business state lives in Runtime.
2. First initialize/reconcile, then draw and publish geometry before accepting pointer input. Each update uses the last committed, still-valid frame geometry. Resize or topology changes invalidate it and require a fresh layout publication before coordinate-dependent input can be dispatched.
3. Paint and hit regions have one source. Frame publication is transactional: failed draw/registration must not leave half-new hitboxes. Disabled/inert overlays still block underlying targets where required. Invisible or removed owners cannot keep focus or capture.
4. Tab order follows visible enabled controls in render order. Focus is distinct from active selection and edit state. Modals trap, save and restore focus to the surviving owner; no owner means a documented valid fallback.
5. Hover is suppressed by keyboard input until pointer motion resumes. Press capture, completed-click eligibility, scrollbar drag and double-click recognition are runtime-owned. Revalidate stable target and eligibility at release.
6. Time is supplied and monotonic. Glyph phase and elapsed interaction feedback are separate inputs; no draw-count clocks. Baseline activation feedback is 140 ms under the characterized clock mapping. Present the changed activation frame before draining a later event that would erase it; presentation acknowledgement tests cover delayed output.
7. Preserve public update/draw purity using immutable state/model references and confined APIs. Rust signatures alone cannot stop side effects hidden behind arbitrary Fn closures or interior mutability; add state-before/after tests and restricted callback review. Do not claim a type signature is a complete purity proof.
8. Ui may publish geometry/layout facts, cursor intent and paint data, but not mutate semantic widget state. Reconciliation from those facts happens in an explicit update before the next render/input as needed. Do not run unbounded update/draw stabilization loops.

## Required proof

- boot without geometry cannot dispatch pointer event
- resize during drag and disappearance of captured row
- focus+hover suppression and restoration
- input flood does not starve animation deadlines or presentation
- draw twice does not change semantic state
- modal close with removed prior focus target
- nonmonotonic time rejected; paused glyph phase still permits data updates

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
