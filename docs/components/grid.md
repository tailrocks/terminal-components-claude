# Grid

Status: proposed target; implementation is future work on `termrock-implementation`.
Owner: termrock-grid.
Visual authority: visual-baseline 4a79c0a2 (commit `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`).
Component ID: W28 · Phase: P5 · Legacy families: C35, C36.

## Purpose and exclusions

Grid renders keyed rows and columns in either a row oriented table recipe or a cell oriented data grid recipe. `GridModel` supplies revision, stable row and column keys, borrowed cell values, readiness, and optional sort or fetch metadata. `GridEditor` is an explicit opt in for local draft validation and commit.

Termrock has one Grid engine. Table row mode and editable cell/grid mode are presentations and navigation policies over that engine. They must not become parallel table and data-grid implementations.

Grid must not own database access, SQL generation, server side sorting, filtering, or fetches; a retained row collection; implicit mutation of a caller model from `draw`; application specific reference viewers, forms, or pending transaction bars; or a generic boxed widget trait or compatibility promise for `WidgetId`, `Outcome`, or `RenderCtx`.

## Public API

The signatures below are the proposed target for the in-place refactor, not a claim about current Rust exports. No signature is source-checked.

```rust
Grid::new(id: Id) -> Grid<'a>

update(
    &self,
    cx: &mut Cx<'_>,
    state: &mut GridState,
    model: &dyn GridModel,
) -> Response<GridAction>

update_editable(
    &self,
    cx: &mut Cx<'_>,
    state: &mut GridState,
    model: &mut dyn GridEditor,
) -> Response<GridAction>

draw(
    &self,
    ui: &mut Ui<'_>,
    area: Rect,
    state: &GridState,
    model: &dyn GridModel,
) -> Rect

measure(
    &self,
    cx: &MeasureCx<'_>,
    model: &dyn GridModel,
    constraints: Constraints,
) -> Size
```

Builders are constrained to declared parts and policies:

```rust
navigation(NavUnit::Row | NavUnit::Cell)
presentation(GridPresentation::Table | GridPresentation::DataGrid)
selection_mode(SelectionMode)
cell(&GridCellPainter)
column_fit(ColumnFit)
readiness(Readiness<'a>)
patch(StylePatch)
```

`GridModel` exposes stable `RowKey` and `ColumnKey` values, a source revision, column metadata, row count or estimate, and borrowed values. Ragged rows and empty sources are valid inputs. `GridEditor` adds explicit validation, draft read/write, and commit operations; the read-only constructor cannot obtain that capability by accident.

`GridState` owns only interaction state: current `CellKey` and optional range-selection anchor; row and column scroll state; selected stable row keys; optional edit draft, error, and edit phase; requested sort or filter intent when the caller has not yet reconciled it; and any local fetch-more/readiness marker needed to avoid duplicate requests. It does not own rows, values, geometry, database metadata, or runtime hit regions. Safe accessors return keys and phases, never mutable domain rows.

The target action family is typed and caller handled:

```rust
GridAction::Activate { cell: CellKey, origin }
GridAction::SortRequested { column: ColumnKey, direction: SortDirection }
GridAction::SelectionRequested(GridSelection)
GridAction::Edited { cell: CellKey }
GridAction::CopyRequested(GridSelection)
GridAction::FetchMore
GridAction::FilterRequested { column: ColumnKey, value: CellValue }
GridAction::Refresh
```

An action identifies a stable key. Display indices are never used as the semantic target of an edit, selection, or activation.

## Ordinary use

Consumer recipe: [EX-10 — One Grid for tables and editable cells](../api/consumer-recipes.md) (`proposed_target`).

Real preview consumers are the Showcase datagrid, tables, and editable pages and the TablePro workbench/grid scenarios. An independent synthetic fixture supplies stable row and column keys, borrowed cell values, sort metadata, and one writable column.

The caller supplies the model and applies typed actions for sort, selection, edit, copy, fetch-more, filter, and refresh. The read-only path uses `update` with `&dyn GridModel` and cannot mutate the model. The editable path uses `update_editable` with `&mut dyn GridEditor`. SQL preview and dirty metadata stay in the application domain adapter, outside Grid.

## Ownership

- The caller owns records, sorting, filtering, persistence, SQL, undo policy, fetching, and transactions.
- Grid owns navigation, selection, edit drafts, projection, and typed requests.
- `update` is the only phase that consumes input and changes `GridState`. It resolves runtime focus, hit, pointer capture, keyboard hover suppression, selection, scrolling, editing, validation requests, and typed actions. It may request a model operation but does not perform I/O.
- `draw` is read-only with respect to state and model. It obtains fresh layout facts, resolves the effective row/column projection, paints the declared parts, and registers hit regions whose geometry is the same geometry painted.
- `measure` computes column widths and required content size from the current borrowed model and constraints. It may publish layout facts to the runtime but cannot mutate durable selection, cursor, draft, or sort state. Position labels and fetch sentinels consume the current measured facts, including after resize.
- Runtime owns focus, hit testing, and pointer capture. Grid itself is a focus owner only when its presentation is interactive; decorative table use creates no extra focus stop.
- A read-only model is never mutated through a render or interaction path.
- Editable cell drafts use the shared [`TextInput`](./text-input.md) and [`TextArea`](./text-area.md) contracts where their recipe requires single-line or multiline editing.
- The scrollbar is the shared [`ScrollRegion`](./scroll-region.md) mechanism.

Shared dependencies: [`identity`](../foundations/identity.md), [`input-actions`](../foundations/input-actions.md), [`runtime`](../foundations/runtime.md), [`layout`](../foundations/layout.md), [`theme`](../foundations/theme.md), [`text`](../foundations/text.md), [`collections`](../foundations/collections.md), [`author`](../foundations/author.md), and [`conformance`](../foundations/conformance.md).

## Customization

Advertised parts are `container`, `header`, `row`, `gutter`, `cell`, `cursor`, `selection`, `editor`, `sort-marker`, `footer`, `scrollbar`, `fade`, and `empty`. Part patches change only the declared part. They cannot replace layout, focus/capture ownership, or the whole surface.

Ordinary customization patches declared cell, selection, cursor, or sort-marker tones. The future default theme reproduces the baseline cells, glyphs, spacing, clipping, and color/capability behavior. Preserve the baseline row anatomy including focus gutter, selection marker, pending-state glyphs (`•`, `+`, `−`, `!`), current-cell reverse treatment, dirty/error tones, numeric alignment, sort marker, empty/loading/error/readiness presentations, and the pending footer where the consuming recipe includes it.

Advanced customization keeps the database-grid recipe's header slots: `▪` for a primary-key column, `∇` for an applied column filter, `▴`/`▾` for sort direction, and `‹N`/`N›` for hidden columns. The row prefix distinguishes focus, selection, pending change, and row number. `NULL` and `DEFAULT` are muted italic; empty strings show faint `''`; changed values use warning tone; invalid values use error plus `!`; references end in `→`; the current cell uses a reversed white-on-canvas treatment (error background while invalid); range selection uses the popover plane; queued deletions are faint and struck through. A caller-supplied partial model may end with a `↓` fetch-more row, which has no data identity. Pending transaction bars and domain-specific column glyphs are caller composition, documented for TablePro in its [application contract](../applications/tablepro.md).

In Table row mode, header activation cycles ascending → descending → none and sorting permutes a stable-key projection so selection and edits survive. For the frozen data-grid recipe, estimate widths from the 95th percentile of the first 200 rows, clamp by column type, never go below the header width, and clip the final visible column rather than leaving blank space. Truecolor, 256-color, 16-color, and no-color modes use the shared theme resolver. Fade leaves cursor, selected, marked, and error emphasis readable. Where a baseline row/cell recipe has activation feedback, its phase uses the shared 140 ms timing. Grid has no generic whole-surface activation animation.

## Behavior

Plain Up/Down or `j`/`k` moves rows; Left/Right or `h`/`l` moves cells in cell mode and pages the horizontal projection through the shared layout policy when the presentation is row mode. Home/End address the current row, and Ctrl+Home/Ctrl+End address the data extent. Page keys move by the measured viewport. Shift extends a range where the selected mode permits it.

Space toggles the current stable row key. Escape clears selection. Enter activates a read-only cell or starts the explicit editable flow. `y` and `Y` request current-cell or selected-range copy; the copied representation is source data, not gutter or truncation decoration.

Editable cells use the shared text editing core. Enter/F2 begins editing only when the cell is writable. Escape rolls back the draft. Enter or the configured commit key validates and commits; invalid input keeps the draft and error on the same stable cell. Tab commits and advances to the next writable stable column; at the boundary it emits a leave action. Read-only columns, primary keys, and deleted rows reject mutation. Boolean and viewer-style cells may use their specialized action recipe while preserving the same typed cell identity.

A first click on a different cell only moves the cursor. A second click on the current editable cell enters editing according to the baseline. Pointer-down alone is not an edit. The painted cell, header, scrollbar, and fetch sentinel hit regions share their exact rectangles. Hover may underline an editable cell only while keyboard hover suppression is off; pointer motion restores hover. Press, drag, and release outside are captured by the originating scroll or selection owner. Vertical and horizontal scroll reach exact endpoints, preserve grab offset, and consume boundary gestures according to the runtime routing contract. Horizontal and vertical wheel input both move their respective bounded viewport without moving the semantic cursor.

Identity and source-change rules:

- `RowKey` and `ColumnKey` survive reorder, filtering, insertion, and removal.
- Cursor, selection, pending edit, and validation error are reconciled by key; a stale key is cleared or reported, never clamped onto a different row.
- Sorting is a request unless the caller explicitly supplies a local sortable model. Local order is a projection over source keys and cannot rewrite the source.
- Source revision changes invalidate stale draft ranges. A pending edit may be committed only to the same cell key and compatible revision.
- Removal of an edited row or column cancels that draft with a typed outcome; it cannot retarget another visible row.
- The fetch-more sentinel has no data key and cannot be treated as a row.

Display text handles wide graphemes, combining marks, tabs, controls, JSON, IDs, null/default values, ragged rows, and narrow cells without splitting a terminal cell or writing outside the allocation.

## Visual matrix

| Axis | Required states | Applies when |
|---|---|---|
| Geometry | `normal`, `zero_area`, `tiny_area`, `nonzero_origin`, `exact_fit`, `one_cell_short`, `unicode_long_content`, `narrow_then_wide` | Always |
| Focus/hover | unfocused, focused, hovered, focus+hover, keyboard-suppressed hover, pointer motion restores hover | Interactive presentation or cell/header owner |
| Activation | No generic whole-surface activation; cell, row, sort, copy, and fetch actions are typed | Child/grid recipe only |
| Editing | navigation, editing, selected text, invalid, read-only, commit, cancel, blur, tab traversal, source revision conflict | Editable presentation |
| Source | empty, ready, selected differs from cursor, disabled entry, reorder, insert, remove, filter, stale target | Model provides the state |
| Readiness | loading, partial, error, retry/refresh when offered | Caller supplies readiness |
| Scroll | no overflow, start, middle, end, wheel boundary, thumb drag, resize while scrolled, edge fade, protected row | Overflow exists; shared ScrollRegion applies |
| Motion | every applicable activation phase, paused/reduced motion | A recipe advertises feedback |

Decorative or unavailable states are marked not applicable. They are never fabricated to fill a matrix.

## Verification

Frozen evidence is pinned to the annotated `visual-baseline` tag at `4a79c0a2d40fca46fc406b77157ce3b3f12ec16b`. Source behavior is `src/widgets/grid.rs` (blob `f68c75c3d09d828c23cdf4a85faf57cdc8427b05`) and `src/widgets/table.rs` (blob `d9a718082697cd77a904f89725247908c8227e00`). Frozen component tests are the inline tests in those two source files, including sort identity, invalid edit preservation, Unicode edit windows, pending changes, range copy, and fetch-more cases. Visual/conformance consumers are Showcase datagrid, tables, and editable pages; TablePro workbench/grid scenarios; the protected snapshots below `snapshots/showcase/pages/datagrid`, `tables`, `editable`, and the matching `tests/visual_baseline` scenarios. These paths are evidence only. New captures must bind to the frozen output before candidate output is accepted. See [`visual-parity`](../verification/visual-parity.md), [`oracle-and-provenance`](../verification/oracle-and-provenance.md), and [`public-api`](../api/public-api.md).

Use [`../reference/capture-plans/grid.json`](../reference/capture-plans/grid.json). It is a planned capture manifest with no candidate-approved expected artifacts. Capture trusted output from the pinned baseline first, then classify evidence as `ExistingOracle`, `ExtractedOracle`, or `Extension` before candidate testing.

Run each required case at dimensions `72×20`, `80×24`, `100×30`, `120×40`, and `160×50`, with truecolor, 256-color, 16-color, `none`, and `nocolor` capabilities where the case applies.

| Case | Required observation |
|---|---|
| W28-01 | Row-table and rich cell-grid presentations remain baseline-identical. |
| W28-02 | First click on a new cell selects; second click on the current editable cell edits. |
| W28-03 | Read-only activation cannot mutate the model; include a compile-fail consumer. |
| W28-04 | Valid/invalid commit, Escape rollback, and Tab traversal preserve the cell key. |
| W28-05 | Sort, reorder, removal, and column removal cannot retarget a draft. |
| W28-06 | Empty, ragged, and large models; horizontal/vertical scroll; fetch-more sentinel. |
| W28-07 | Header/cell/row patches affect only declared actual cells. |

Every capture records dimensions, symbols, wide-cell continuation, foreground, background, supported modifiers, cursor position/visibility, focus and capture owners, selected stable key, navigation key, draft and committed state, action count, and action target. Invalid oracle setup is a blocked capture, never a passing result.

Required negative tests:

- A `&dyn GridModel` consumer cannot call editor mutation or commit methods.
- `draw` cannot change cursor, selection, sort, draft, model values, or action count.
- Reorder/remove/column removal during editing cannot apply a draft to a new key.
- A pointer release outside the grid cannot leave capture or a pressed cell.
- Rags, empty data, zero/tiny allocations, wide glyphs, and long JSON cannot panic or write outside the buffer.
- Duplicate ids, stale source revisions, and unavailable rows fail closed.
- A custom part painter cannot replace focus, capture, layout, or another part.

Grid acceptance requires a compiling external consumer, source-state tests, exact applicable snapshots, and independent review of the evidence. A painted placeholder or a custom call that is later overwritten fails acceptance.

## Rejected use

Forbidden: painting padded row strings with a selection marker and tracking a selected index instead of using Grid.

```rust
// Forbidden: preview owns grid rendering and index selection.
ui.paint_str(row, 0, "› ada    34   active   ");
if clicked { picked = row_index; }
```

Rule: [ARC-012](../architecture/component-composition.md). Preview code must not recreate component rendering or generic interaction. The following patterns are prohibited in preview drawing paths:

```text
historical full-screen painters
snapshot files or exported frame data used as application output
hardcoded answer tables for 72x20, 80x24, 100x30, 120x40, or 160x50
fixture/scenario IDs that select a different painter
paused mode that bypasses component updates or layers
raw buffer access through termrock::author or an alias
direct ui.paint_str, ui.fill, set_string, or equivalent rendering of controls
padded strings that simulate columns, selection, fields, menus, or buttons
manual borders, focus gutters, scrollbars, or widget cursors
normal component drawing followed by a historical overlay
app-specific screens moved into a termrock crate under a generic name
fake state flags that display success without the normal component action
```

Use `Grid::new` with stable `RowKey`/`ColumnKey` values and handle the typed `GridAction`.

## Known gaps

- Capture plan W28 is planned and uncaptured; no expected artifacts are bound.
- Implementation is future work on `termrock-implementation`.
- [FIX-003 historical editor and cockpit paths](../implementation/code-remediation-backlog.md): pending.
- [FIX-007 signature drift](../implementation/code-remediation-backlog.md): the contract API is a proposed target and no signature is source-checked.
