# Keyed collections, presenters, and reconciliation

**Canonical owner:** borrowed collection models, stable item/row/column
identity, source revisions, keyed projection, reconciliation, and constrained
row/cell presenters. This is the target contract for the in-place Termrock
refactor; it does not preserve index-based legacy APIs as future public
compatibility.

**Specification record:** F08, legacy family C10. The visual authority is the
immutable [`visual-baseline` commit
`4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`](https://github.com/donbeave/terminal-components-claude/tree/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b).

Read this with the [identity and revisions](identity.md), [layout and
measurement](layout.md), [semantic theme](theme.md), [component authoring
surface](author.md), [public API](../api/public-api.md), [shared
types](../api/types.md), and [conformance contract](../verification/conformance.md).
Those documents own identity construction, geometry, style, callback
permissions, and verification authority. This page owns collection data
access and keyed reconciliation.

## Baseline evidence and future names

The frozen implementation provides behavior evidence in
[`src/widgets/list.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/list.rs),
[`src/widgets/tree.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/tree.rs),
[`src/widgets/grid.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/grid.rs),
and [`src/widgets/table.rs`](https://github.com/donbeave/terminal-components-claude/blob/4a79c0a2d40fca46fc406b77157ce3b3f12ec16b/src/widgets/table.rs).
The current `ListBox`, `TreeView`, `DataGrid`, `DataTable`, `WidgetId`, and
index-based `order` fields are legacy evidence. Their visible behavior and
interaction are preserved through the frozen applications while the future
Termrock API uses borrowed models and stable semantic keys.

## Responsibility

This foundation owns:

- borrowed model traits and records for lists, trees, grids, diffs, props,
  pickers, tabs, menus, status items, and other keyed collections;
- `Revision`-aware source access and projection caches;
- one reconciliation algorithm for current/cursor keys, selection anchors,
  scroll anchors, pending edits, and captured action targets;
- visible-only iteration and bounded projections for large collections;
- constrained row/cell/part presenter callbacks that can paint only their
  reserved rectangle;
- explicit read-only versus editable model boundaries.

The identity foundation owns the distinct types and collision checks. Layout
owns rectangles and measurement. Theme owns styles and state presentation.
Runtime owns focus, hit registration, pointer capture, and release-time target
validation. Components choose navigation, selection, readiness, and visual
recipes. Applications own domain data, sorting/filtering authority, IO,
database/workspace persistence, and async fetches.

## Non-goals

- no cloned domain dataset or unbounded library-owned model;
- no positional row/column key fallback, label-as-key convention, or hidden
  index identity;
- no database, SQL, filesystem, network, provider, Docker, undo transaction,
  persistence, or blocking IO in a component callback;
- no global mutable `Buffer`, geometry registry, runtime focus store, executor,
  or event router exposed to presenters;
- no second table/grid engine: `Grid` covers table-row and cell/grid modes;
- no product implementation for Jackin, Holla, TablePro, or Showcase;
- no requirement that every collection operation be O(1). Index construction,
  sorting, filtering, and source replacement may be O(n); ordinary draw work is
  bounded by visible rows/cells plus declared overscan.

## Proposed borrowed model and presenter API

These Rustdoc-style declarations are target notation. Concrete fields remain
private unless a record is explicitly borrowed input.

```rust
pub trait TreeSource {
    fn revision(&self) -> Revision;
    fn roots(&self) -> &[ItemKey];
    fn node(&self, key: ItemKey) -> Option<TreeNode<'_>>;
}

pub trait GridModel {
    fn revision(&self) -> Revision;
    fn row_count(&self) -> usize;
    fn row_key(&self, index: usize) -> Option<ItemKey>;
    fn columns(&self) -> &[GridColumn<'_>];
    fn cell(&self, row: ItemKey, column: ColumnKey) -> Option<CellValue<'_>>;
}

pub trait GridEditor: GridModel {
    fn editable(&self, cell: CellKey) -> bool;
    fn validate(&self, request: &CellEdit) -> Result<(), ValidationMessage>;
    fn commit(&mut self, request: CellEdit) -> Result<(), ValidationMessage>;
}

pub trait DiffSource {
    fn revision(&self) -> Revision;
    fn rows(&self) -> &[DiffRow<'_>];
}

pub type RowPainter<T> = dyn Fn(&mut RowUi<'_>, Rect, &T, RowState);
pub type GridCellPainter =
    dyn Fn(&mut CellUi<'_>, Rect, CellValue<'_>, CellState);
```

`Keyed::key`, `ItemKey`, `ColumnKey`, `CellKey`, and `Revision` come from the
[identity contract](identity.md). `Grid::update` receives `&dyn GridModel` in
read-only mode. `Grid::update_editable` is the explicit mutable boundary for
local model edits; draw and measure receive read-only model access. A
read-only model must fail to compile at the editing entry point rather than
silently discarding an edit.

Borrowed presenters receive immutable typed data, immutable visual state, a
reserved rectangle, and constrained `RowUi`/`CellUi`/`PartUi`. They return
immediate paint work only. They cannot mutate domain data, register arbitrary
hits, change geometry, install focus/capture, access a global buffer, spawn a
task, or perform IO. The [authoring contract](author.md) owns the precise
allowed vocabulary.

## Reconciliation contract

Reconciliation happens during `update` or an explicit caller-invoked
reconcile operation that uses the same algorithm. It never happens in
`draw` or `measure`.

1. Validate source revision and duplicate live keys before publishing
   interactive geometry. Duplicate `ItemKey`, `ColumnKey`, or `CellKey`
   relationships fail closed.
2. Retain the current key, selection anchors, scroll anchor, expanded tree
   keys, and edit target when the identity still exists and remains eligible.
3. When the current item disappears, choose the nearest eligible successor,
   then predecessor, then no current item. This fallback is deterministic and
   documented; an armed activation, pointer capture, or pending edit is never
   transferred to that fallback.
4. Revalidate `enabled`, readiness, editability, and source revision on the
   live target at input and release. A disabled or removed target may remain
   visible but cannot activate.
5. Preserve selection semantics independently from cursor/focus. Reorder,
   insertion, filtering, and duplicate labels do not change the target of a
   keyed selection or command.
6. For tree sources, preserve expansion and current node by key, not flattened
   row index. Parent/child relationships and loading/error readiness remain
   caller data.
7. For grids, preserve row and column keys through sort/reorder, keep a cell
   edit associated with its source `CellKey`, and reject missing rows/columns
   rather than applying an edit to a display neighbor.
8. For retained text/diff rows, use the stable line identity and source
   revision rules in [text](text.md); eviction clips or clears anchors
   explicitly.

Reconciliation can build a new index after a source revision. The cost and
  bounded cache behavior are part of the component's plan; a claim that all
  navigation is O(1) is invalid when the source is not indexed.

## Components and consolidation

The following components consume this foundation while keeping their visual
and behavioral contracts in their own canonical pages:

- `List`, `FilterList`, `NavList`, `Tree`, `Tabs`, `Steps`, `PropsList`,
  `Picker`, `CommandPalette`, `PickerChain`, `Completion`, and menu families
  use keyed rows/stages/items and shared cursor/selection reconciliation;
- `Grid` supplies both table-row and cell/grid presentations. The old
  `DataTable`/table mode and `DataGrid`/cell mode are recipes over one engine;
  they do not become parallel mechanisms;
- `DiffView` consumes stable diff row/hunk keys and the shared text projection;
- `StatusBar` consumes keyed status groups/items; old segments-style display
  is a presentation recipe, not another collection engine;
- `DerivedHintBar` adapts binding metadata and does not render a second hint
  engine;
- `TerminalView` consumes caller-provided terminal cell snapshots and stable
  line/cell identities; it does not parse a terminal stream or own a PTY.

Component pages define row/cell parts, disabled/readiness behavior, focus and
scroll interactions, and exact visual cases. This foundation supplies the
common key/revision model and does not flatten intentional differences.

## Data, cache, and performance invariants

- Caller-owned source values outlive each component call. Props borrow them;
  durable component state never borrows them.
- A projection cache is keyed by source revision and every relevant query,
  width, sort/filter, theme metric, capability, wrap, and readiness input. A
  changed source or dimension invalidates stale derived data before input is
  interpreted.
- Draw visits only visible rows/cells plus declared overscan. Large source
  data stays outside library state. `GridEditor::commit` is update-time local
  mutation only; draw never calls it.
- Custom presenters preserve row/cell semantics: type-specific errors,
  selected/current markers, disabled state, copy policy, and rich parts cannot
  be erased by a minimal label callback.
- Missing cells, ragged rows, missing columns, empty sources, partial/loading
  data, and all-disabled collections have explicit readiness/empty behavior.
- A source replacement does not mutate the caller's new dataset merely because
  an old edit or selection was pending.

## Integration rules

- `update` reconciles source revision, applies normalized intents, and returns
  at most one typed collection action. `draw` reads the reconciled state and
  current borrowed model; it publishes geometry through the runtime. `measure`
  reads the model under constraints without selecting or mutating it.
- Layout rectangles and hit registration are shared with
  [layout](layout.md). Runtime release validation uses stable keys rather than
  converting a captured display index into a new target.
- Theme patches and row/cell slots use the same owner context as the stock
  painter. A slot cannot bypass semantic styling or paint outside its cell.
- Input, responses, focus, pointer capture, and layers are runtime-owned. A
  collection reports typed intents; it does not run the application command.
- Application sorting/filtering/fetching supplies a new borrowed model and
  increments `Revision`. The library may expose a sort request action, but it
  does not perform database or provider work.

## Conformance and negative tests

Collection cases use exact visual output and interaction traces from the
[visual parity](../verification/visual-parity.md) and [interaction
parity](../verification/interaction-parity.md) contracts. Record source
revision, current/selected stable keys, scroll anchor, edit phase, focus and
capture owners, dimensions, visible cells, and typed action target. Use
`ExistingOracle`, `ExtractedOracle`, and `Extension` provenance lanes; an
extension must not be relabeled baseline parity.

Required proof:

- a 100,000-row model invokes presenters only for visible rows/cells plus
  declared overscan;
- insert, remove, reorder, and filter with duplicate labels preserve stable
  keys and do not retarget an armed action;
- tree expansion/filtering preserves keyed ancestors and current node;
- ragged grids, missing cells, removed columns, empty/loading/partial/error
  readiness, and invalid edit targets fail safely;
- a read-only model cannot enter the editing API at compile time;
- row/cell style overrides reach actual painted cells;
- draw never calls `GridEditor::commit` and presenter callbacks cannot mutate
  model or runtime state;
- pending edit/selection/capture is invalidated when its source key is removed;
- source revision changes invalidate stale projections and completion results.

Negative tests must fail when a duplicate key is accepted, a display index
retargets an activation after reorder/removal, an edit commits to a new row,
an inaccessible cell panics, a callback paints outside its reserved rectangle,
a read-only model is mutated, candidate painting can bless expected output, or
an app-specific persistence/provider operation appears inside the component.

## Source-pack coverage

This page migrates F08's borrowed model/presenter and reconciliation contract:
`TreeSource`, `GridModel`, `GridEditor`, `DiffSource`, row/cell callback aliases,
stable-key requirements, source revisions, visible-only rendering, local edit
ownership, duplicate-label/reorder/remove safety, cache invalidation, rich
type-specific rows, and all required proof groups. The F08 mapping to legacy
family C10 remains in the family-disposition ledger. F01 owns identity and
revision construction; F03 owns runtime interaction; F05 owns geometry; F06
owns style; F10 owns authoring permissions; their rules are linked rather than
copied here.
