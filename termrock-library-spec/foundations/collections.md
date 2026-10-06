# F08 · Borrowed collections, presenters and model traits

**Scope:** shared generic library/test infrastructure, not a product subsystem.  
**Legacy families:** C10  
**Visual source:** `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`; interface spellings below are proposals.

## Source references

- [src/widgets/list.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/list.rs)
- [src/widgets/tree.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/tree.rs)
- [src/widgets/grid.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/grid.rs)
- [src/widgets/table.rs](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/table.rs)

[Target architecture](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/architecture/component-architecture.md) · [refactoring task catalog](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/docs/refactoring-plan/task-index.tsv)

## Proposed public surface

Rustdoc-style declaration notation. This is not a compiled implementation. Read with [PUBLIC-API.md](../PUBLIC-API.md) and the [shared type dictionary](../reference/TYPES.md).

```rust
trait TreeSource { fn revision(&self) -> Revision; fn roots(&self) -> &[ItemKey]; fn node(&self, key: ItemKey) -> Option<TreeNode<'_>>; }
trait GridModel {
    fn revision(&self) -> Revision;
    fn row_count(&self) -> usize;
    fn row_key(&self, index: usize) -> Option<ItemKey>;
    fn columns(&self) -> &[GridColumn<'_>];
    fn cell(&self, row: ItemKey, column: ColumnKey) -> Option<CellValue<'_>>;
}
trait GridEditor: GridModel {
    fn editable(&self, cell: CellKey) -> bool;
    fn validate(&self, request: &CellEdit) -> Result<(), ValidationMessage>;
    fn commit(&mut self, request: CellEdit) -> Result<(), ValidationMessage>;
}
trait DiffSource { fn revision(&self) -> Revision; fn rows(&self) -> &[DiffRow<'_>]; }
type RowPainter<T> = dyn Fn(&mut RowUi<'_>, Rect, &T, RowState);
type GridCellPainter = dyn Fn(&mut CellUi<'_>, Rect, CellValue<'_>, CellState);
```

## Contract

1. Borrowed data and stable keys are mandatory. Custom presenters are Fn callbacks with no global Buffer, geometry mutation or event routing access. They paint a reserved row/cell through constrained author interfaces.
2. Use one reconciliation algorithm for stable cursors and anchors. Removal never turns a previously armed action into an action on a fallback row. Disabled/readiness state is checked on the actual live target.
3. GridEditor mutates only the caller-provided local model during update. Its implementations may not perform blocking IO in a UI callback. Saving a database/workspace is a separate application action, not grid behavior.
4. For indexed slices, accept a source Revision in the constructor and cache key lookup/projection by revision. Building an index after a source change may be O(n); ordinary draw targets O(visible rows/cells plus measured overscan). Do not falsely claim every operation is O(1).
5. Type-specific row adapters retain rich parts, selection markers, errors and copy semantics. A ListRow with only a label must not erase the functionality of the baseline custom rows.
6. Keep unbounded domain data out of library-owned state; bound cached projections and invalidate them when revision, dimensions or styling metrics change.

## Required proof

- 100k-row visible-only render callback counter
- insert/remove/reorder/filter with duplicate labels
- ragged grid/missing cell/removed column safety
- read-only model fails compilation on editing entry point
- row/cell override reaches actual pixels
- draw never calls GridEditor::commit

This foundation has no invented independent hover/pressed screenshot. Its visible effects are proved through the components and composed fixtures that use it. Nonvisual invariants have headless/state/compile-fail/protocol tests. Implementation and independent verification are not executed in this document pack.
