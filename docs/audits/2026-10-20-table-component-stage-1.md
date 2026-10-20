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

This stage does not implement the frozen table geometry, flexible column
allocation, gutter, style metadata, overflow, or scrollbars. Those remain the
next library stage. The task-table consumer remains unchanged until that
presentation API lands and paired captures run.

Focused verification: `cargo nextest run -p termrock-grid --test
table_ordered_model`. Workspace `cargo fmt --all --check` also reports an
unrelated pre-existing line wrap in `crates/showcase-demos/src/pages/lists.rs`;
owned Rust files are formatted directly with `rustfmt`.
