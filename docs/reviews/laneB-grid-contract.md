# Lane B adjudication: Grid and hidden-focus contract

**Status:** accepted for Q1–Q3. This record resolves the three questions in
`docs/plans/slice6-tablepro.md` that block Slice 4I and Slice 6. Lane A still owns
the corresponding edits to `COMPONENT_ARCHITECTURE.md`, `REFACTORING_STATE.md`,
and `crates/**`.

## Scope and evidence

The review compared the accepted generic-grid boundary with TablePro's current
sorting, key bindings, editing errors, focus-ring behavior, and regression
inventory. The library grid sees only borrowed presentation cells. TablePro's
adapter still has the domain values needed for NULLs-last and numeric comparison.
The eleven application chords named by the Slice 6 plan need keyed cursor state,
while the migrated assertions need only the horizontal offset beyond the other
semantic state already exposed below. The explorer drawer needs keyboard reachability
without claiming drawable or clickable geometry.

The measured migration contract remains 23 TablePro application tests and 42
render digests. The current legacy TablePro test target remains green at 41 tests.
These counts are acceptance inventory, not permission to reduce or rename coverage.

## Q1: sorting ownership

The TablePro adapter owns row order and the comparator. The library `Grid` owns
neither an `order` permutation nor a comparison policy. It cannot reproduce
TablePro's NULLs-last, numeric-aware `cmp_cells` semantics from `CellRef`, and a
text-based fallback would silently change product behavior.

`Grid` emits `GridAction::Sort(ColumnKey, SortDir)`. A table-backed adapter handles
that action by requesting or re-running the server query. A result-grid adapter
handles the same action by sorting its own display-to-storage `order` with its own
domain comparator. The adapter then presents rows to `GridModel` in display order.
The grid continues to address cursor, selection, edit, and actions by `ItemKey` and
`ColumnKey`, so an adapter reorder cannot retarget state to a different logical row.

There is no library `local_sort` switch, no library comparison hook, and no
lexicographic comparison of rendered cell text. The existing architecture phrase
"sort-as-permutation" means a model/adapter reorder preserves keyed identity; it
does not assign the permutation to `GridState`.

## Q2: `GridState` accessors

The accepted public reader set is exactly:

```rust
impl GridState {
    pub fn cursor(&self) -> Option<(ItemKey, ColumnKey)>;
    pub fn selected_rows(&self) -> &KeySet;
    pub fn is_editing(&self) -> bool;
    pub fn edit_error(&self) -> Option<&FieldError>;
    pub fn col_offset(&self) -> usize;
}
```

`edit_error` preserves the typed `FieldError`; reducing it to `Option<&str>` would
discard its machine-readable code and contradict the shared error contract.
Horizontal inspection exposes only `col_offset()`. The earlier proposed
`row_window()` and `col_window()` readers are rejected: they expose derived layout
and cache state rather than durable application semantics. TablePro can retain the
assertion's intent by checking `col_offset() > 0`; it does not need a public window
range. No mutable state, row index, column index, editor draft, reconciliation stamp,
or cached rectangle becomes public.

Delete-to-NULL and the other application chords remain TablePro bindings. They read
`cursor()` and call the adapter; they do not become database-shaped `GridAction`
variants. An edit failure stays in the inline editor and is observable through the
typed `edit_error()` reader.

## Q3: focus without geometry

Add an explicit `Ui::register_focus_only` API. Do not weaken
`Ui::register_control`, do not pass `Rect::ZERO` through it, and do not synthesize a
hit region.

The intended surface is:

```rust
pub fn register_focus_only(&mut self, id: Id, focus: Focusability);
```

It registers only a `FocusEntry` in the current scope and layer, with
`area: Rect::ZERO` and `swallows_typing: false`. `Focusable` and
`FocusableReadOnly` are reachable in normal traversal; `Disabled` remains recorded
but unreachable. `ClickOnly` has no focus-only meaning and is a no-op. The read-only
form declares `StateFlags::READ_ONLY`, as `register_control` does.

This is a narrow exception to the draw-registration rule: an application may
declare a hidden keyboard stop intentionally, but a component with no drawable area
still must not call `register_control`. Because no control/hit region exists,
`Harness::click_id(id)` remains ignored and reports `Diagnostic::UnaddressableId`.
The explorer drawer uses this explicit API; it must not be remodeled as a popover or
given fabricated geometry.

## Corrections to earlier records

The Slice 6 plan's recommended answers need three corrections when Lane A records
this adjudication:

1. Sorting order and comparison belong to the adapter, not merely to an unspecified
   caller; `Grid` emits the keyed sort action and never sorts `CellRef::text`.
2. `GridState::edit_error()` returns `Option<&FieldError>`, not `Option<&str>`, and
   horizontal state exposes `col_offset()` only, not `row_window()` or
   `col_window()`.
3. Zero-area focus is not admitted through `register_control`. It gets the explicit
   `register_focus_only` API so focus registration cannot accidentally create a hit
   target or relax empty-area rejection.

## Acceptance conditions

Q1 is complete when the public action contains `Sort(ColumnKey, SortDir)`, the grid
contains no order vector, local-sort flag, or cell-text comparator, and a test proves
an adapter reorder preserves cursor, selection, and pending edit identity by key.
TablePro must separately prove server-sort dispatch and local result sorting retain
the existing NULLs-last and numeric ordering.

Q2 is complete when the five readers above are the only new public `GridState`
readers, the edit reader returns the original `FieldError`, and tests cover keyed
cursor reconciliation, selected-row borrowing, edit lifecycle/error preservation,
and horizontal offset after reveal. No public mutable or geometry-cache access is
allowed.

Q3 is complete when `register_focus_only` is distinct from control registration,
focus traversal reaches its enabled entry, disabled/read-only semantics match the
normal ring, no hit region is registered, and `click_id` produces the specified
unaddressable diagnostic. Existing empty-area rejection for `register_control` and
hit regions must remain green.

The contract is not integrated until Lane A records it in the architecture and
ledger, implements the library half, and returns the focused library tests green.
Slice 6 then preserves all 23 application tests, all 42 digests through the required
reviewed baseline process, and the 41 currently passing legacy tests until their
final migration disposition is recorded.
