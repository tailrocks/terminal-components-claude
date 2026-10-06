# W05 · RadioGroup

**Group:** Controls · **Phase:** P3 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow keyed options and controlled selected key. State owns only navigation/scroll bookkeeping.

Legacy family mapping: `C14`  
Proposed implementation home: `crates/termrock/src/components/choice.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/choice.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/choice.rs) — observed Git blob `9405acf87c152c0766ae9bbcd71c01c9d5db4e80`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: choice.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/choice.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
RadioGroup::new(id: Id, options: &'a [ChoiceItem<'a>], revision: Revision) -> RadioGroup<'a>

update(&self, cx: &mut Cx<'_>, state: &mut RadioGroupState) -> Response<RadioAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &RadioGroupState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
selected(Option<ItemKey>)
orientation(Axis)
disabled(bool)
patch(StylePatch)
row(&ChoiceRowPainter)
```

**Durable state:** RadioGroupState: navigation cursor key and ScrollState, but no chosen domain value.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** RadioAction::Choose { key: ItemKey, origin: ActivationOrigin }

## Visual parts and composition

Advertised styled parts: `container`, `gutter`, `marker`, `label`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md)

## Behavioral contract

1. Arrow navigation changes the candidate cursor; activation chooses the stable key. Do not silently choose merely because focus arrived.
2. Disabled options remain visible but cannot activate; reconciliation keeps a surviving key across reorder.
3. An empty/all-disabled group has no invalid index or accidental activation. Rendering must distinguish chosen and cursor rows.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | `pointer_down`, `held_press`, `release_inside`, `release_outside`, `removed_target`, `disabled_target`, `keyboard_activate`, `activation_feedback`, `feedback_expired` | eligible actionable part; composite uses real child action controls |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W05-01 | chosen item differs from cursor and hovered item |
| W05-02 | disabled first/middle/last options with keyboard and pointer |
| W05-03 | reorder or delete a chosen/cursor item between pointer press and release |
| W05-04 | empty and all-disabled groups; narrow horizontal layout |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/radio-group.json](../capture-plans/radio-group.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
