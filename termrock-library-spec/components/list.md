# W12 · List

**Group:** Collections · **Phase:** P2 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

Borrow application rows and a pure row presenter; never copy all rows/labels into a second owned list. Reconcile only on declared source revision changes.

Legacy family mapping: `C22`  
Proposed implementation home: `crates/termrock/src/components/list.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/list.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/list.rs) — observed Git blob `44cbbab4fdd3a04f67a2712f2be241264a966ee1`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: list.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/list.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/lists](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/lists)
- [snapshots/jackin/manager](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/manager)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
List::new(id: Id, rows: &'a [T], revision: Revision) -> List<'a, T> where T: Keyed

update(&self, cx: &mut Cx<'_>, state: &mut ListState) -> Response<ListAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ListState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
selected(&'a [ItemKey])
selection_mode(SelectionMode)
row(&'a RowPainter<T>)
readiness(Readiness<'a>)
disabled(bool)
patch(StylePatch)
```

**Durable state:** ListState: stable cursor key, selection anchor and ScrollState. Controlled selected keys are borrowed separately.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** ListAction::{Activate { key, origin }, SelectionRequested(SelectionRequest)}

## Visual parts and composition

Advertised styled parts: `container`, `row`, `gutter`, `marker`, `label`, `meta`, `status`, `scrollbar`, `fade`, `empty`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md)

## Behavioral contract

1. Navigation cursor and controlled selection are distinct. The same generic painter handles plain and custom rows through constrained RowUi.
2. Click-current-row/activation policy is characterized from the source rather than assuming every initial pointer down activates. Completed gestures only.
3. Retain surviving keys through insert/remove/sort/filter; deleted targets are not replaced by whatever now occupies an old index.
4. Wheel affects the list under the pointer, not the focused list elsewhere. Edge fades never obscure the protected selected/caret row.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | `pointer_down`, `held_press`, `release_inside`, `release_outside`, `removed_target`, `disabled_target`, `keyboard_activate`, `activation_feedback`, `feedback_expired` | eligible actionable part; composite uses real child action controls |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| readiness | `loading`, `partial`, `error`, `retry_or_refresh_when_offered` | explicit readiness or item lifecycle supplied by caller, not synthetic whole-widget states |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W12-01 | cursor, selected row and hovered row all differ |
| W12-02 | empty/loading/partial/error content via shared readiness |
| W12-03 | sort/insert/remove between down and up; duplicate labels with distinct keys |
| W12-04 | wheel at both boundaries without focus change |
| W12-05 | 100k rows: draw callbacks limited to visible rows plus documented overscan |
| W12-06 | custom row part patch visibly changes expected cells |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/list.json](../capture-plans/list.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
