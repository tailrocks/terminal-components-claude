# Shared table behavior, stage 1

`termrock-grid` now exposes `Table`, `TableColumn`, `TableSort`, and `TableState`.
`Table` delegates hit testing, focus, row navigation, selection, scroll state,
and painting to the existing `Grid`. Its ordered model view maps sorted rows
back to source indexes while retaining source `ItemKey`s. Sorting stays inside
the library.

The frozen table sort contract is unsorted, ascending, descending, then
unsorted. Text compares lowercase display strings. Numeric display sorting
keeps only ASCII digits, `.` and `-` before parsing as `f64`; source-index ties
reverse along with descending order. Cursor and checked rows stay keyed to
source rows. `s` sorts the current column only for `Table`; `Grid` keeps its
existing bindings and default navigation.

Table updates report `TableAction::SortChanged`, including `None` on the third
sort transition; all non-sort grid actions are carried by `TableAction::Grid`.
The table reconciles key identity automatically. Models can return a revision
from `GridModel::revision` when ordered row-key identity changes; models without
that cheap signal use a complete ordered-key comparison, reusing the snapshot
allocation when keys are unchanged. A changed identity invalidates Grid's
collection stamp so removed selections are dropped even for same-size interior
replacements.

`TableState::visible_rows` exposes the Grid-owned visible range, and
`Table::position_label` formats sort and row-range metadata (for example,
`unsorted · 1–20 of 24`) from that state.

Stage 2 adds table-owned frozen layout. `TableColumn::flexible` allocates the
remaining width after fixed column minima, with two-cell gaps, a three-cell
gutter, a two-cell right reserve, and a one-cell vertical scrollbar reserve.
Horizontal overflow reserves a row before recalculating vertical overflow. The
88x22 and 130x22 reference geometries are checked against their exact widths,
visible column counts, and overflow bars. The horizontal track and thumb use
the shared scrollbar theme styles. `TableState::visible_rows` and
`Table::position_label` expose Grid-owned scroll position and sort metadata.

The task-table consumer remains unchanged until it adopts the layout builder
and paired captures run. Geometry unit coverage passes, but the application has
not yet been checked with direct paired CLI captures.

Focused verification: `cargo nextest run -p termrock-grid` (132 passed),
`cargo clippy -p termrock-grid --all-targets -- -D warnings`, and owned-file
`rustfmt --edition 2024 --check`. Workspace `cargo fmt --all --check` also
reports an unrelated pre-existing line wrap in
`crates/showcase-demos/src/pages/lists.rs`.
