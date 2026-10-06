# W07 · Field

**Group:** Forms · **Phase:** P3 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Field is shared chrome only. It must never own a second editor, field value or additional focus stop.

Legacy family mapping: `C16`  
Proposed implementation home: `crates/termrock/src/components/field.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/input.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/input.rs) — observed Git blob `f440b74c7d305e987ec31bf04d34afd993844bb0`
- [src/widgets/textarea.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/textarea.rs) — observed Git blob `5bfa5148a8b335c28ed52a5af305fa9df39917e5`
- [src/widgets/select.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/select.rs) — observed Git blob `3b6f43f8f642ccce512ac53754d114dc978bd149`
- [src/widgets/field_common.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/field_common.rs) — observed Git blob `bd41cdd85e9c2605c356f7468dc0de3e3c50e54b`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: field.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/field.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Field::new(label: &'a str) -> Field<'a>

draw<R>(&self, ui: &mut Ui<'_>, area: Rect, body: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> R
measure(&self, cx: &MeasureCx<'_>, child: Size, constraints: Constraints) -> Size
```

Configuration builders:

```rust
help(&'a str)
error(Option<&'a ValidationMessage>)
required(bool)
plain_label(bool)
child_id(Id)
patch(StylePatch)
```

**Durable state:** None. The child owns its own state and ID.



**Typed actions:** None; child responses pass through unchanged.

## Visual parts and composition

Advertised styled parts: `label`, `required`, `help`, `error`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: Shared foundations only.

## Behavioral contract

1. Default single-line field allocation is label + input row + help/error row, matching the tagged TextInput three-row contract.
2. Error replaces the help line without moving the child unexpectedly. Optional/required annotations disappear only according to the baseline width rule.
3. Composite input constructors may provide .label/.help convenience methods, but they delegate to this same Field painter; do not render Field twice.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | **Not applicable:** Display-only surface; Field/Panel may reflect child focus but do not create independent interactive state. | no independent focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W07-01 | label/help versus label/error at identical allocated height |
| W07-02 | required and optional suffix at its width boundary |
| W07-03 | child editing with error preserves its focus gutter and cursor |
| W07-04 | one Tab stop for a labelled input, not two |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/field.json](../capture-plans/field.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
