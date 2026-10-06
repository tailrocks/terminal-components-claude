# W28 · Grid

**Group:** Data and text · **Phase:** P5 · **Classification:** baseline-component  
**Visual commit:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`  
**Status:** proposed API; implementation, compilation, captures and independent review not executed.

## Purpose and boundary

GridModel supplies revision, stable row/column keys and borrowed cell values. GridEditor adds local edit validation/commit; SQL, undo transactions, persistence and fetch work stay in the caller.

Legacy family mapping: `C35`, `C36`  
Proposed implementation home: `crates/termrock/src/components/grid.rs`. Thin wrappers may share this file; this is not permission for duplicate engines.

## Pinned references

- [src/widgets/grid.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/grid.rs) — observed Git blob `f68c75c3d09d828c23cdf4a85faf57cdc8427b05`
- [src/widgets/table.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/table.rs) — observed Git blob `d9a718082697cd77a904f89725247908c8227e00`
- [API ownership architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) and [refactoring task index](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv).
- [Refactored API-direction reference: grid.rs](https://github.com/donbeave/terminal-components-claude/blob/f758dc3f88196c3b93d70994dc7095edbfce1c93/crates/tui/src/components/grid.rs) at `f758dc3f88196c3b93d70994dc7095edbfce1c93`. This is not the visual oracle and these proposed signatures are not claimed to be identical exports.

Snapshot discovery roots (not a claim that every state is already captured):

- [snapshots/showcase/pages/datagrid](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/datagrid)
- [snapshots/showcase/pages/tables](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/tables)
- [snapshots/showcase/pages/editable](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/snapshots/showcase/pages/editable)

## Proposed public API

Signature-declaration notation; consuming builders omit `self -> Self`. See [shared public API](../PUBLIC-API.md), [type dictionary](../reference/TYPES.md), and [visual proof contract](../VISUAL-VERIFICATION.md).

```rust
Grid::new(id: Id) -> Grid<'a>

update(&self, cx: &mut Cx<'_>, state: &mut GridState, model: &dyn GridModel) -> Response<GridAction>
update_editable(&self, cx: &mut Cx<'_>, state: &mut GridState, model: &mut dyn GridEditor) -> Response<GridAction>
draw(&self, ui: &mut Ui<'_>, area: Rect, state: &GridState, model: &dyn GridModel) -> Rect
measure(&self, cx: &MeasureCx<'_>, model: &dyn GridModel, constraints: Constraints) -> Size
```

Configuration builders:

```rust
navigation(NavUnit) // Row or Cell
presentation(GridPresentation) // Table or DataGrid recipes
selection_mode(SelectionMode)
cell(&GridCellPainter)
column_fit(ColumnFit)
readiness(Readiness<'a>)
patch(StylePatch)
```

**Durable state:** GridState: cursor CellKey, selection anchor, row/column scroll and optional edit draft. No rows, column values, geometry or database metadata are owned.

Safe access: `cursor() -> Option<CellKey>`, selection and edit phase. The read-only GridModel path never exposes GridEditor mutation.

**Typed actions:** GridAction::{Activate { cell: CellKey, origin }, SortRequested { column: ColumnKey, direction: SortDirection }, SelectionRequested(GridSelection), Edited { cell: CellKey }, CopyRequested(GridSelection), FetchMore}

## Visual parts and composition

Advertised styled parts: `container`, `header`, `row`, `gutter`, `cell`, `cursor`, `selection`, `editor`, `sort-marker`, `footer`, `scrollbar`, `fade`, `empty`.

Standard style overrides follow the shared theme precedence. Only explicitly supported parts accept replacement slots; geometry, focus/capture and whole-surface ownership are not replaceable. Decorative fragments may use their parent's attribution scope. Preserve baseline text, glyphs, spacing, surface and clipping; do not infer a new design from the component name.

Implementation dependencies: [scroll-region](./scroll-region.md), [text-input](./text-input.md)

## Behavioral contract

1. One renderer and interaction engine serves row-oriented table and cell-oriented editable grid; do not retain DataTable as a parallel implementation.
2. Read-only update receives &dyn GridModel and cannot mutate it. Editable update explicitly receives &mut dyn GridEditor; only update may request/commit an edit.
3. First click chooses a different cell; completed click on the current editable cell enters editing according to the baseline. Pointer-down alone must not start an edit.
4. Sorting and source changes preserve semantic CellKey; a stale edit or destructive row action cannot move to a different row after sorting.
5. Validate source ranges and ragged/empty models, clip wide glyph continuation cells, and share Field/TextEditorCore behavior rather than embedding an independent editor.

## Applicable states

| Axis | Required states / disposition | Target |
|---|---|---|
| geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | component |
| focus_hover | `unfocused`, `focused`, `hovered`, `focus_and_hover`, `keyboard_suppresses_hover`, `pointer_motion_restores_hover` | interactive owner or child; Brand only in interactive mode; Steps only in navigable mode; no decorative focus stop |
| activation | **Not applicable:** No generic whole-surface activation. Text/scroll/selection gestures and child actions are covered by their own cases. | none at this surface |
| editing | `navigation`, `editing`, `selected_text`, `invalid`, `read_only`, `commit`, `cancel_or_finish_per_component`, `blur`, `tab_traversal`, `source_revision_conflict` | editable specialization |
| source | `empty`, `ready`, `selected_differs_from_cursor`, `disabled_entry`, `reorder`, `insert`, `remove`, `filter`, `stale_target` | keyed records; chosen/cursor distinction only when both exist |
| readiness | `loading`, `partial`, `error`, `retry_or_refresh_when_offered` | explicit readiness or item lifecycle supplied by caller, not synthetic whole-widget states |
| scroll | `no_overflow`, `start`, `middle`, `end`, `wheel_boundary`, `thumb_drag`, `resize_while_scrolled`, `edge_fade`, `protected_row` | component scroll region; horizontal strips use start/end; fade/protection only on applicable content rows |

Each advertised visual variant gets every individually applicable state. Mandatory combinations and fixture dimensions follow the shared proof contract. The component-specific cases below refine these axes; they do not replace them.

## Required acceptance cases

| Case ID | Required observation |
|---|---|
| W28-01 | row-table and rich-cell-grid baseline presentations |
| W28-02 | current-cell second click edits; new-cell first click only selects |
| W28-03 | read-only activation cannot mutate source, proven by compile-fail consumer |
| W28-04 | valid/invalid edit commit, Escape rollback, Tab move to next editable cell |
| W28-05 | sort/reorder/remove edited row; column removal during editing |
| W28-06 | empty/ragged/large models; horizontal and vertical scroll plus fetch-more sentinel |
| W28-07 | custom header/cell/row overrides alter only declared actual cells |

For every case record exact cells/cursor, the stable focus/capture/layer owners, action count and target, and relevant draft/selection/source state. Cases introducing new safety/API behavior need an Extension disposition when no baseline counterpart exists. Invalid or unavailable oracle setup is a blocked capture, never a passing test.

## Reference/capture deliverable

Use [capture-plans/grid.json](../capture-plans/grid.json). It specifies required source roots, dimensions, capabilities and cases. It deliberately has no approved expected artifacts. Capture from the pinned source using trusted adapters, seal numeric gestures/time samples, then compare candidate public code.

Accept this component only after its external-consumer API example compiles, source-state tests and exact applicable snapshots pass, no semantic state changes during draw, documented parts affect actual cells, and an independent reviewer checks the API and evidence. A painted placeholder or a call later overwritten by custom paint is a failure.
