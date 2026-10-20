# TablePro picker consumer migration

Status: implementation staged on `parity/tablepro-picker-live-20261020`; not
integrated into the acceptance branch. Product parity remains incomplete.

## Fixed references

- Frozen tag object: `2718a6b8127a522a3d351d0b4bdb7542fc8da542`.
- Frozen source commit: `4169783ae883788186d161af09ba9c481bd84e5c`.
- Pinned tuiscotti commit: `a1ea27f757d75d07b05499276cd932e9f8e067ad`.
- Candidate source base: `2de0d01255390d2a02b7775ee452268782802a38`.

## Change

`tablepro-ui` now composes the quick switcher and tab list through Termrock's
`PickerSlotExt`. Tab rows use application-owned `TabSummary` data with stable
`TabKey` identity. Picker configuration carries the frozen widths, row cap,
center alignment, substring filtering, and list cursor style. Tab activation
and Delete resolve the selected stable key against current workbench state.
Delete keeps the frozen immediate-close behavior, including for dirty tabs.

Removed production imports and the two historical ANSI painters for the quick
switcher and tab-list picker. Removed their exact-size, terminal, test-build,
connection, tab-count, and color dispatch predicates. Removed the unrendered
query editor and cursor registration used by the historical tab-list painter.
Reference snapshots remain in the test corpus.

## Checks

- `cargo fmt -p tablepro-ui --check`: pass.
- `cargo nextest run -p tablepro-ui --test quick_switcher_contract --no-fail-fast`:
  9 passed before adding the frozen-geometry regression.
- `cargo clippy -p tablepro-ui --lib -- -D warnings`: pass.
- Isolated-target `cargo nextest run -p tablepro-ui --no-fail-fast` on clean
  source base `2de0d01255390d2a02b7775ee452268782802a38`: 184 passed, 3
  failed. Candidate with the new geometry regression: 183 passed, 4 failed.
  Candidate and base used separate `/tmp/termrock-target-picker-*` Cargo
  targets. The base failures reproduce the candidate's 3 unrelated failures:
  - `perf::frame_tablepro_grid_500x14_120x40`: 152 allocations, limit 100.
  - `row_actions_contract::inserting_after_five_hundred_rows_reveals_the_new_owned_record`:
    `h.tab_to(id)` returned false.
  - `visual::tablepro_visual_baseline`: `grid-cell-editing 120 40 junie
    truecolor` expected digest `9f202c0c39eb897b`, actual
    `45ebad62a9783294`.
- `cargo fmt --all --check`: pre-existing formatting differences in
  `crates/showcase-demos/src/pages/chrome.rs`; TablePro-scoped format check
  passes.
- `cargo clippy -p tablepro-ui --lib -- -D warnings`: pass.

The new `tab_list_picker_keeps_the_frozen_single_tab_geometry` regression is
currently red. Frozen source `Picker::height()` computes `rows + 7`; shared
Picker currently computes `rows + 5`. At 120×40, one tab currently resolves
to `(28, 11, 64, 6)`; frozen geometry is `(28, 16, 64, 8)`. Shared picker
measurement and layer alignment must be fixed before this consumer can pass.

## Remaining evidence and gaps

- Real-binary paired capture and replay through pinned tuiscotti remain
  required before PR integration.
- Query edit-menu and pointer drag-selection contracts depend on shared field
  work; no app-local input workaround was added.
- This scoped change does not audit or remove other TablePro historical replay
  paths.
