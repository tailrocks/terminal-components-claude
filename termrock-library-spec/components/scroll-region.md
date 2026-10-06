# W45 · ScrollRegion

**Group:** Layout · **Phase:** P2 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

One shared scroll model and painter for list/tree/grid/output/picker/props consumers. Parent remains the focus owner unless its semantics specify otherwise.

Legacy family mapping: `C08`  
Proposed implementation home: `crates/termrock/src/components/scroll_region.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/core/scroll.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/core/scroll.rs)
- [src/widgets/scrollbar.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/scrollbar.rs) — observed Git blob `fa0132637370da4afe2e40a07fb558c135e188df`
- [src/ui/fade.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/fade.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: scroll_region.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/scroll_region.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/scrolling](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/scrolling)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
ScrollRegion::new(owner: Id, axis: Axis, content: Extent) -> ScrollRegion<'a>

update(&self, cx: &mut Cx<'_>, state: &mut ScrollState) -> Response<ScrollAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &ScrollState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
bars(ScrollbarPolicy)
fade(FadePolicy)
protected(&'a [ProtectedRange])
```

**Durable state:** ScrollState: bounded offset and follow/anchor policy; geometry is runtime-owned LayoutFacts. Thumb drag capture is not duplicated in each control.

Expose read-only keyed cursor/navigation/scroll observations as applicable, with explicit invariant-preserving commands. Fields remain private; never expose runtime geometry or mutable domain rows.

**Typed actions:** ScrollAction::{OffsetChanged, FollowChanged(bool)}

## Visual parts and composition

Advertised styled parts: `viewport`, `track`, `thumb`, `start-fade`, `end-fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: Shared foundations only.

## Behavioral contract

1. Wheel routes to the innermost eligible top-layer scroll region under the pointer without changing focus. At the boundary consume according to the reference rather than chain into an unrelated pane.
2. Thumb only appears on overflow; drag preserves grab offset and reaches exact first/last offsets. Paint and hit geometry use the same track calculation.
3. Apply the baseline edge fade only where hidden content exists; protect selected/caret rows. Fade must operate on effective foreground emphasis, not flatten all text to one gray.
4. Compute fresh viewport/visible-range facts before labels use them; draw may publish geometry but cannot silently alter durable scroll semantics.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W45-01 | no-overflow versus first overflowing row |
| W45-02 | top/middle/bottom wheel and scroll boundaries |
| W45-03 | thumb drag with nonzero grab offset to exact ends |
| W45-04 | nested scrollables under modal and disabled regions |
| W45-05 | fade at heights3,4,11,12 with protected rows |
| W45-06 | resize/reflow retains anchored content |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/scroll-region.json](../capture-plans/scroll-region.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
