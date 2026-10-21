# Shared property sheet contract

## Scope

This increment adds an opt-in `PropsList::sheet()` presentation in the shared controls crate. Existing `PropsList::new` and `Props` callers keep the compact column presentation. The sheet presentation places a shared focus gutter and spacer before labels, reserves a terminal-cell-safe value column and trailing edge, and shows `y copy` only for the focused copyable row. Stable `ItemKey` identity, `PropsState`, and typed `PropsAction::Copy` remain the state and action contracts.

`PropsList::measure_rows` measures borrowed live rows with the same label/value and wrapping geometry used by sheet drawing and update. The existing data-free `measure` remains available for compatibility. Empty sheet data produces zero measured size and publishes no focusable control. The renderer does not create a text cursor for read-only values.

## Evidence

The frozen source `4169783ae883788186d161af09ba9c481bd84e5c` and fixed design source `f4867e9b5f8a08a3bb61ce80d644198a29f51984` have identical `src/widgets/props.rs` and `src/widgets/info_dialog.rs` digests, as recorded in plan `09aa1428a553fa3000bc3e8fd4d4ce9e6c936c51e4c55662d9a027561637a02d`. That implementation uses `RowUi::paint(..., RowLook::Bar)`, two leading cells, a measured label column, an eight-cell focused copy reservation, keyed selection, and full-value copy actions. The fixed height of eleven rows applies only to the five-fact, no-intro, no-detail example and is not encoded here.

The focused regression first used the compact constructor with sheet assertions and failed at the real rendered label column (`left: Some('n')`, expected `Some('R')`). Switching the same consumer to `.sheet()` passed. Focused nextest run `513656df-8e89-44dc-89b2-df1310b9d7c5` passed all three sheet tests. Final full controls nextest `b05f8780-fb32-4de1-bbaf-511f3a15fadf` passed all 146 tests with zero skipped. Strict scoped Clippy (`cargo clippy -p termrock-controls --all-targets -- -D warnings`) and rustfmt/diff checks passed after the final keyed-reorder correction. The tests exercise the rendered focus gutter and label origin, focus-only copy hint, keyed row hit target, keyboard and pointer copy actions, absence of a read-only cursor, focused wrapped-value tail reveal/collapse, live-row measurement, Unicode, four color profiles, zero-size draw/measurement, large-list scrolling, and stable cursor identity after live-data reorder. The scroll/reorder exercise uses a second theme (`Theme::paper`) in addition to the Junie tests.

No frozen application binary was launched for this increment. This is shared component evidence only, not application parity approval.

## Remaining work

The shared dialog chrome, real header/footer close controls, modal command scope, and Jackin live-data migration remain separate increments. A full alternate theme matrix and the full pinned Tuiscotti comparison remain outstanding; these results do not establish application parity. The compact `PropsList` default is intentionally unchanged until consumers opt into the sheet policy.
