# W06 · ChipBar

**Group:** Collections · **Phase:** P3 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow labels, checked/disabled/closable flags and actions. No filter-query parser or product tags inside the library.

Legacy family mapping: `C15`  
Proposed implementation home: `crates/termrock/src/components/chip.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/chips.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/chips.rs) — observed Git blob `36cb21bf1be548c39544cef2916669a0a4f55b30`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: chip.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/chip.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
ChipBar::new(id: Id, chips: &'a [ChipItem<'a>], revision: Revision) -> ChipBar<'a>

update(&self, cx: &mut Cx<'_>, state: &mut ChipBarState) -> Response<ChipAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ChipBarState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
add_action(Option<ActionMeta<'a>>)
disabled(bool)
patch(StylePatch)
row(&ChipRowPainter)
```

**Durable state:** ChipBarState: cursor key and horizontal scroll; controlled check state remains in rows.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** ChipAction::{Activate { key, origin }, SetChecked { key, checked, origin }, Close { key, origin }, Add { origin }}

## Visual parts and composition

Advertised styled parts: `container`, `gutter`, `marker`, `label`, `close`, `add`, `overflow`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md)

## Behavioral contract

1. Close, activate, checkbox and add affordances use distinct keyed subparts. Closing a chip must not also activate its body.
2. Keep chosen/check markers separate from focus; preserve baseline strip density and overflow affordances.
3. Deletion/reorder preserves neighboring navigation deterministically; all cells and close hitboxes clip to the same strip.

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
| W06-01 | checked/unselected and checked/focused chips |
| W06-02 | close-button press followed by reorder/delete |
| W06-03 | overflow at exact-fit, one-cell-short and long Unicode label widths |
| W06-04 | empty strip with and without Add |
| W06-05 | disabled chip body and close eligibility |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/chip-bar.json](../capture-plans/chip-bar.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
