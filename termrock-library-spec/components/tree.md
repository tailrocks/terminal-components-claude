# W15 · Tree

**Group:** Collections · **Phase:** P2 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

TreeSource provides stable node keys, parent/child relationships, revision and readiness. No filesystem traversal, Docker tree or async loader lives here.

Legacy family mapping: `C25`  
Proposed implementation home: `crates/termrock/src/components/tree.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/tree.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/tree.rs) — observed Git blob `1df281b10a46ab0f62038df46da9a47e3ee5ea07`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: tree.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/tree.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/trees](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/trees)
- [snapshots/jackin/manager](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/manager)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Tree::new(id: Id, source: &'a dyn TreeSource) -> Tree<'a>

update(&self, cx: &mut Cx<'_>, state: &mut TreeState) -> Response<TreeAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &TreeState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
selected(&'a [ItemKey])
branch_activation(BranchActivation)
row(&TreeRowPainter)
readiness(Readiness<'a>)
patch(StylePatch)
```

**Durable state:** TreeState: expanded ItemKeys, cursor key, range anchor and ScrollState; no owned node data.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** TreeAction::{Activate { key, origin }, ExpansionChanged { key, expanded }, LoadChildren { key }, SelectionRequested(SelectionRequest)}

## Visual parts and composition

Advertised styled parts: `container`, `row`, `indent`, `disclosure`, `gutter`, `marker`, `label`, `badge`, `status`, `scrollbar`, `fade`, `empty`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md)

## Behavioral contract

1. Left/right collapses/expands or moves to parent/child according to the reference. Disclosure clicks and body activation are separate hit parts.
2. Expanding an unloaded node emits a keyed load request; caller supplies later children. Collapse or removal while loading cannot retarget the completion.
3. Duplicated labels are valid; duplicated keys are invalid. Filter/replacement retains surviving identities and applies documented nearest-survivor fallback.
4. Custom tree rows inherit indentation, clipping, focus marker and style part overrides from the same component.

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
| W15-01 | collapsed/expanded leaf/branch with loading/error children |
| W15-02 | disclosure click versus body click and branch activation policy |
| W15-03 | remove parent/cursor during press or pending lazy load |
| W15-04 | expand/collapse all on large trees with bounded visible drawing |
| W15-05 | filtered tree preserves ancestors and stable selection |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/tree.json](../capture-plans/tree.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
