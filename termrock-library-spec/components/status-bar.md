# W40 · StatusBar

**Group:** Chrome · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow left/center/right groups, priorities, labels and action metadata. No service polling or account/instance knowledge.

Legacy family mapping: `C48`  
Proposed implementation home: `crates/termrock/src/components/status.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/statusbar.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/statusbar.rs) — observed Git blob `79b6c1617fe3e6b9a928725469568567f769789d`
- [src/widgets/segments.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/segments.rs) — observed Git blob `78add23bb5f5ef1e2abc6f79631f53b27da71004`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: status.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/status.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/chrome](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/chrome)
- [snapshots/jackin/capsule](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/capsule)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
StatusBar::new(id: Id, groups: &'a [StatusGroup<'a>], revision: Revision) -> StatusBar<'a>

update(&self, cx: &mut Cx<'_>) -> Response<StatusAction>
draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
animation(AnimationSample)
patch(StylePatch)
patch_part(Part, StylePatch)
```

**Durable state:** None for value storage; runtime owns any keyed item hover/press and action feedback.



**Typed actions:** StatusAction::Invoke { key: ItemKey, action: ActionKey, origin: ActivationOrigin }

## Visual parts and composition

Advertised styled parts: `container`, `group`, `item`, `label`, `value`, `icon`, `separator`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [spinner](./spinner.md)

## Behavioral contract

1. Absorb old Segments into one painter with baseline group spacing, truncation and priority-drop order.
2. Each interactive item has its own stable key; noninteractive items are not focus stops.
3. A spinner changes only its declared cells and cannot change layout width unpredictably.
4. Visible metadata is live caller data; the widget never synthesizes a success status from missing values.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | `pointer_down`, `held_press`, `release_inside`, `release_outside`, `removed_target`, `disabled_target`, `keyboard_activate`, `activation_feedback`, `feedback_expired` | eligible actionable part; composite uses real child action controls |
| motion | `all_unique_phases`, `phase_wrap`, `before_at_after_boundary`, `paused`, `reduced`, `same_time_repaint` | explicit animation sample; static completion/failure is tested only where the supplied status supports it |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W40-01 | left/center/right alignment at exact fit |
| W40-02 | narrow width drops items in declared priority order |
| W40-03 | item hover/click versus noninteractive item |
| W40-04 | spinner frames preserve gaps and stable width |
| W40-05 | dynamic removal during pointer press |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/status-bar.json](../capture-plans/status-bar.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
