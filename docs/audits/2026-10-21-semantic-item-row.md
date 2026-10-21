# Semantic item row contract

This increment adds a reusable `SemanticItemRow` for existing `Tree` and
`FilterList` consumers. It keeps the old `ItemRow` implementation and the
default `FilterList`/`Tree` row behavior unchanged. The new policy has a
44-cell metadata admission threshold, a two-cell metadata gap, a 12-cell
minimum label budget, and grapheme-safe middle truncation by default. Each is
configurable through builders. An end-truncation option remains available.

`Tree::row` now accepts the existing public `RowFn` contract directly, so a
shared row adapter can be passed without a closure wrapper. The adapter emits
no gutter: Tree retains its disclosure, focus, hit region, row selection,
scrolling, and trailing-mark reservation. `RowUi::meta` still uses its prior
one-cell gap and admission rule; `meta_with_gap` supplies the same behavior
with an explicit gap.

Coverage is in `crates/termrock-navigation/tests/semantic_item_row.rs` and
`crates/termrock-runtime/tests/rowui_metadata_gap.rs`. It checks metadata at
43/44/45 content cells across truecolor, 256-color, 16-color, and monochrome;
middle and end truncation; Unicode content; hit-region registration; cursor
and keyboard navigation; Tree trailing-mark space; and reuse with unrelated
FilterList data, empty collections, and zero-to-narrow content. The facade-only example is
`crates/termrock/examples/semantic_item_row_consumer.rs`.

Focused `cargo nextest` runs, the default-feature example check, strict Clippy,
formatting, and `git diff --check` passed for this increment. The full
Navigation nextest suite reported 232/234 passing; the two failures in
`item_row_columns` (`constructor_defaults_match_the_pre_policy_full_cell_fixture`
and `owner_part_styles_win_over_painter_defaults_in_columns_mode`) reproduced
at the exact merged base `7789d10db849f0a2cf37e3e9579d4c8b5cee179e` before this
change. The full Runtime suite passed 399 tests; nextest reported three
pre-existing ignored tests. This is shared component API evidence only. It does
not migrate the Jackin Manager painter or claim application parity; those
remain a later atomic application-adoption step under the migration plan.
