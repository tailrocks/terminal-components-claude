# W13 · FilterList

**Group:** Collections · **Phase:** P4 · **Classification:** baseline-composition  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

The filter owns a keyed index projection, not cloned domain objects. Query edits return actions; caller remains the committed query authority.

Legacy family mapping: `C23`  
Proposed implementation home: `crates/termrock/src/components/filter_list.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/picker.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/picker.rs) — observed Git blob `084bdb1edaf6142d8a99ad404a8e612e40ef6662`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: filter_list.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/filter_list.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/pickers](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/pickers)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
FilterList::new(id: Id, rows: &'a [T], revision: Revision) -> FilterList<'a, T> where T: Keyed

update(&self, cx: &mut Cx<'_>, state: &mut FilterListState) -> Response<FilterListAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &FilterListState) -> Rect
measure(&self, cx: &MeasureCx<'_>, constraints: Constraints) -> Size
```

Configuration builders:

```rust
query(&'a str)
filter(&'a dyn Fn(&T, &str) -> MatchResult)
row(&'a RowPainter<T>)
readiness(Readiness<'a>)
patch(StylePatch)
```

**Durable state:** FilterListState: query draft, projected keyed cursor and ListState. The original source rows stay borrowed.

Safe access: current stable result/stage key, `query() -> &str` and scroll observations. No mutable access to caller-owned result data.

**Typed actions:** FilterListAction::{QueryChanged, Activate { key, origin }, SelectionRequested(SelectionRequest)}

## Visual parts and composition

Advertised styled parts: `query`, `results`, `row`, `match`, `empty`, `scrollbar`, `fade`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [list](./list.md), [text-input](./text-input.md)

## Behavioral contract

1. Default matching uses the shared Unicode-safe fuzzy matcher; custom ranking/filtering is caller-supplied and side-effect-free.
2. Result matches map to source graphemes, not bytes mistaken for display columns. Keep stable keys when source order or query changes.
3. No-match, no-data, loading and failed-source are different readiness presentations. Highlighted but unavailable rows never activate.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| readiness | `loading`, `partial`, `error`, `retry_or_refresh_when_offered` | explicit readiness or item lifecycle supplied by caller, not synthetic whole-widget states |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W13-01 | empty source versus nonempty source with zero matches |
| W13-02 | paste Unicode query; backspace one grapheme |
| W13-03 | source revision changes while filtered and selected key survives |
| W13-04 | query changes make current row ineligible; no stale activation |
| W13-05 | matched spans across combining characters and CJK |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/filter-list.json](../capture-plans/filter-list.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
