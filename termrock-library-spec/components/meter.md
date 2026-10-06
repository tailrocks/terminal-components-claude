# W39 · Meter

**Group:** Feedback · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Caller determines whether a value is used or remaining, quota freshness, warning and failure. Do not infer a missing value as zero capacity.

Legacy family mapping: `C47`  
Proposed implementation home: `crates/termrock/src/components/meter.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/progress.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/progress.rs) — observed Git blob `3efc6b4f9d9eb84eec34da27e65f8c98bef3db6c`
- [src/bin/jackin_preview/screens/usage.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/bin/jackin_preview/screens/usage.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: meter.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/meter.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/jackin/usage](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/usage)
- [snapshots/jackin/accounts](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/accounts)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Meter::new(id: Id, value: Option<Percent>) -> Meter<'a>

draw(&self, ui: &mut Ui<'_>, area: Rect) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
readout(&'a str)
visual(MeterVisual) // Line or Block
tone(MeterTone)
animation(AnimationSample)
patch(StylePatch)
```

**Durable state:** None. Typed optional percentage and semantic tone are caller-controlled.



**Typed actions:** None.

## Visual parts and composition

Advertised styled parts: `container`, `fill`, `rest`, `value`, `suffix`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [spinner](./spinner.md)

## Behavioral contract

1. Two baseline visual modes: line and filled block. Preserve embedded readout placement, clipping, fill rounding and fixed suffix width.
2. The default consumption thresholds are <=59 low, <=84 medium and >84 high. Domain-explicit Warning/Exhausted/Stale/Refreshing/Error/Unknown overrides generic thresholds.
3. Capacity is not completion: do not use green success fill merely because the meter is full. Unknown draws no invented track; error and refresh remain distinct.
4. Treat quota, currency, reset timestamps and provider labels as caller data. Meter accepts already-safe readout text.

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
| W39-01 | 0,59,60,84,85,100 and unknown |
| W39-02 | Line and Block layouts at rounding/readout boundaries |
| W39-03 | warning/exhausted/stale/refreshing/error/unknown for same underlying value |
| W39-04 | refresh spinner full cycle; unknown no bar |
| W39-05 | remaining versus used values explicitly supplied by caller |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/meter.json](../capture-plans/meter.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
