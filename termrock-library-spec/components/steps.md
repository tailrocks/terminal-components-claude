# W16 · Steps

**Group:** Feedback · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller supplies lifecycle status and current step; no dependency graph, job scheduler, retry policy or launch engine.

Legacy family mapping: `C26`  
Proposed implementation home: `crates/termrock/src/components/steps.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/steps.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/steps.rs) — observed Git blob `8bf98f38e7745631eb3ec8a526523fcf4032abec`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: steps.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/steps.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/taskrunner](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/taskrunner)
- [snapshots/jackin/cockpit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/cockpit)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Steps::new(id: Id, steps: &'a [StepItem<'a>], revision: Revision) -> Steps<'a>

update(&self, cx: &mut Cx<'_>, state: &mut StepsState) -> Response<StepsAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &StepsState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
mode(StepsMode) // Display or Navigable
animation(AnimationSample)
patch(StylePatch)
row(&StepRowPainter)
```

**Durable state:** StepsState only in navigable mode: cursor key and ScrollState. Display mode has no interaction state.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** StepsAction::Activate { key: ItemKey, origin: ActivationOrigin }

## Visual parts and composition

Advertised styled parts: `container`, `row`, `gutter`, `marker`, `label`, `detail`, `status`, `scrollbar`, `fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md), [spinner](./spinner.md)

## Behavioral contract

1. Preserve queued/running/skipped/blocked/done/failed glyphs, indentation and details. Do not turn every step green: use the baseline status recipe.
2. Display-only progress introduces no focus stops. Navigable mode returns a key for inspecting a step without changing its lifecycle.
3. Animation is sampled at explicit phase/time; polling or redraw count does not advance a step.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | `pointer_down`, `held_press`, `release_inside`, `release_outside`, `removed_target`, `disabled_target`, `keyboard_activate`, `activation_feedback`, `feedback_expired` | eligible actionable part; composite uses real child action controls |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| readiness | `loading`, `partial`, `error`, `retry_or_refresh_when_offered` | explicit readiness or item lifecycle supplied by caller, not synthetic whole-widget states |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |
| motion | `all_unique_phases`, `phase_wrap`, `before_at_after_boundary`, `paused`, `reduced`, `same_time_repaint` | explicit animation sample; static completion/failure is tested only where the supplied status supports it |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W16-01 | every lifecycle state including skipped and blocked |
| W16-02 | running spinner through all unique phases |
| W16-03 | navigation to a failed step without changing the caller lifecycle |
| W16-04 | partial details/long labels and overflow |
| W16-05 | live source replacement preserves current key |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/steps.json](../capture-plans/steps.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
