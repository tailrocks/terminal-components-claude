# Lane B status

## Current state

The fresh Q1–Q3 Grid review is accepted and recorded in
`docs/reviews/laneB-grid-contract.md`. It assigns sorting order and comparison to the
TablePro adapter, requires `GridAction::Sort(ColumnKey, SortDir)`, fixes the exact
`GridState` reader surface, and chooses a separate `Ui::register_focus_only` API for
the explorer drawer's hidden keyboard stop.

Lane B has not started the TablePro migration. The preserved regression inventory is
23 application tests, 42 render digests, and the currently green 41-test legacy
TablePro target.

## Completed

- Q1 resolved: adapters present already ordered rows and own their domain comparator;
  the generic grid emits a keyed sort request and never compares rendered text.
- Q2 resolved: `cursor`, `selected_rows`, `is_editing`, typed `edit_error`, and
  `col_offset` are the exact public readers. Derived row/column windows stay private.
- Q3 resolved: hidden keyboard reachability uses `register_focus_only`, never a
  zero-area `register_control`; it creates no hit region.
- Corrections and acceptance conditions are explicit in the review for Lane A to
  transcribe and implement.

## Blockers

1. **Lane A integration:** Lane A must record the accepted contract in
   `COMPONENT_ARCHITECTURE.md` and `REFACTORING_STATE.md`, then implement and verify
   the `Grid` sort action, exact `GridState` accessors, and
   `Ui::register_focus_only` under `crates/**`. Lane B does not own those files.
2. **Slice 5:** Slice 6 package move cannot begin until Slice 5 closes the crate
   rename, removes the root binary layout, and establishes `apps/tablepro`. This is
   an explicit precondition in `docs/plans/slice6-tablepro.md`.

## Next action

After both blockers clear, start Slice 6 package 6-0, then execute the TablePro work
packages in the dependency order from `docs/plans/slice6-tablepro.md`. Preserve all
23 application tests and 42 digest keys; classify any moved baseline before Lane A
runs the repository-owned bless operation. Do not replace domain sorting with cell
text sorting and do not reach into `crates/**` for missing primitives.
