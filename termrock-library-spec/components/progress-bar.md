# W37 · ProgressBar

**Group:** Feedback · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller owns operation progress and lifetime; this component only paints a deterministic sample. It never starts a timer or job.

Legacy family mapping: `C45`  
Proposed implementation home: `crates/termrock/src/components/progress.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/progress.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/progress.rs) — observed Git blob `3efc6b4f9d9eb84eec34da27e65f8c98bef3db6c`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: progress.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/progress.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/progress](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/progress)
- [snapshots/jackin/cockpit](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/cockpit)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
ProgressBar::new(id: Id, value: ProgressValue) -> ProgressBar<'a>

draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
label(&'a str)
status(ProgressStatus) // Active, Done, Error, Paused
animation(AnimationSample)
patch(StylePatch)
```

**Durable state:** None. Value, semantic state and AnimationSample are supplied.



**Typed actions:** None. Cancel/retry/pause controls are separate Buttons.

## Visual parts and composition

Advertised styled parts: `container`, `label`, `fill`, `rest`, `percentage`, `suffix`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: Shared foundations only.

## Behavioral contract

1. Baseline determinate fill uses round(track_width * ratio), with a rounded integer percentage and fixed two-cell suffix column.
2. Active bars use secondary text tone, completion uses success, error uses error and pause uses muted. Preserve suffixes: blank, check, exclamation and double bar.
3. The source omits the label unless width exceeds label width + 8; when the remaining track is less than six cells it shows percentage only. These are executable boundary fixtures.
4. Indeterminate rendering must be characterized from the source across a full cycle for each tested width, not represented by one arbitrary still.
5. Fraction rejects nonfinite inputs before rendering; clamped finite values and explicit status follow the documented value contract. Nonfinite rejection is a robustness extension, not an old screenshot claim.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | **Not applicable:** Display-only surface; Field/Panel may reflect child focus but do not create independent interactive state. | no independent focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| motion | `all_unique_phases`, `phase_wrap`, `before_at_after_boundary`, `paused`, `reduced`, `same_time_repaint` | explicit animation sample; static completion/failure is tested only where the supplied status supports it |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W37-01 | 0%, fractional rounding boundaries, 50%, 100% and completed value |
| W37-02 | active/done/error/paused with aligned suffix column |
| W37-03 | label width+8 and width+9; track length5 versus6 |
| W37-04 | indeterminate full phase cycle at narrow/normal/wide sizes |
| W37-05 | pause/resume/reduced motion do not advance from draw count |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/progress-bar.json](../capture-plans/progress-bar.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
