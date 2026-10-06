# W33 · SplitPane

**Group:** Layout · **Phase:** P2 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

One axis-parameterized allocation/capture mechanism; caller supplies panes and their content. It owns no PTYs, processes or view routing.

Legacy family mapping: `C42`  
Proposed implementation home: `crates/termrock/src/components/split.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/splitter.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/splitter.rs) — observed Git blob `80c113f7585827d340da45921011c2a1953a8b34`
- [src/ui/layout.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/ui/layout.rs)
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: split.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/split.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/jackin/capsule](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/capsule)
- [snapshots/jackin/accounts](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/jackin/accounts)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
SplitPane::new(id: Id, axis: Axis) -> SplitPane

update(&self, cx: &mut Cx<'_>, state: &mut SplitPaneState) -> Response<SplitAction>
draw<R>(&self, ui: &mut Ui<'_>, area: Rect, state: &SplitPaneState, body: impl FnOnce(&mut Ui<'_>, SplitAreas) -> R) -> R
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
minima(first: u16, second: u16)
seam_width(u16)
resizable(bool)
patch(StylePatch)
```

**Durable state:** SplitPaneState: desired ratio/fixed allocation and optional maximized child key. Pointer-grab offset/capture geometry live in runtime.

Safe access: `ratio() -> SplitRatio`, `maximized() -> Option<ItemKey>`. Explicit resize/zoom commands mutate preference during update, not geometry during draw.

**Typed actions:** SplitAction::{Resized { ratio: SplitRatio }, Maximized { child: Option<ItemKey> }}

## Visual parts and composition

Advertised styled parts: `container`, `first-body`, `seam`, `second-body`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: Shared foundations only.

## Behavioral contract

1. The seam painted and registered must be identical. Pointer capture preserves initial grab offset; release outside still ends drag safely.
2. Clamp displayed allocation to minima without overwriting the preferred ratio: narrowing then expanding restores preference.
3. Keyboard resize and zoom use the same state; nested splits reconcile by stable identity.
4. Below combined minima, use the explicit responsive/drawer composition policy rather than negative or wrapped dimensions.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W33-01 | horizontal and vertical allocations with identical rules |
| W33-02 | drag seam at first/last cell; resize during capture |
| W33-03 | narrow below minima then widen restores preferred ratio |
| W33-04 | keyboard increments and maximize/unmaximize |
| W33-05 | nested panes with one removed while maximized |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/split-pane.json](../capture-plans/split-pane.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
