# Performance: scrolling tables, grids, lists and trees

PO request (2026-10-16): table components lag when scrolling. Plan entry:
`docs/visual-testing/plan.md` §P, task T31.

## How it is measured

Release profile, in-process `TestBackend` (no terminal I/O; a real
terminal adds the diff write and the emulator's parse for every frame).

| Harness | Run |
|---|---|
| Widgets alone (DataGrid, DataTable, ListBox, TreeView; 500 / 10k / 100k rows; 11 columns; long-cell variants) | `cargo nextest run --cargo-profile release --test table_perf --run-ignored only --no-capture` |
| TablePro app (customers: 48,912 rows, 500 loaded, 11 columns; query results of `SELECT * FROM customers`) | `cargo nextest run --cargo-profile release --bin tablepro --run-ignored only -E 'test(measure_table_surfaces)' --no-capture` |

Scenarios at 120x40 and 200x55: wheel burst, PageDown, End/Home, key
repeat Down, horizontal wheel, hover moves over rows, resize. Numbers are
from a shared developer machine under load (other agents compiling), so
single figures vary by about ±50%; the ratios are what matter.

## Findings

| # | Hotspot | Measured (before) | Root cause | Status |
|---|---|---|---|---|
| P1 | The event loop draws one full frame per state-changing event | 60-notch wheel burst on the customers table: 60 frames, 17.8 ms at 120x40 and 56.5 ms at 200x55 (draw ≈ 0.25–1 ms per frame, handling ≈ 1 µs per event); 120 key repeats: 33 ms / 128 ms. One frame for the whole burst: 0.3–0.6 ms. On a real terminal every one of those frames is also written and parsed, so the screen keeps scrolling after the wheel stops. | `drain_ready_inputs` returns on the first `Changed` and the loop draws before the next queued event (the right rule for activation, which must see newly rendered controls, but applied to every input). | open |
| P2 | Per-frame cost grows with the length of cut cell values | DataGrid, 2,000 rows with 64 KiB JSON/text cells: 19–24 ms per frame once those columns are on screen (0.25 ms otherwise); DataTable: 37 ms per frame at 200x55. | `truncate`, `fit` and the reveal's `is_cut` measure the full string (`width(s)`) for every visible cell; the grid also normalises the whole JSON document, sanitises up to 10,000 chars and clones the full value twice (`text != v.text()`, `&v.text()`) per visible cell. | open |
| P3 | Column width sampling reads full values | `set_rows` with 64 KiB cells: 65 ms for 2,000 rows (200 sampled rows × 11 columns, full `CellValue::text()` clone + width). The sample size (200) is hardcoded. | `sample_widths` clones and measures each sampled value in full although the result is clamped to the column's max width. | open |
| P4 | Row state lookup scans every pending cell | `row_state` runs `pending.cells.keys().any(..)` per visible row: O(visible rows × pending cells) per frame after a large paste. | Linear scan over a `HashMap`. | open |

Checked and not a problem (frame cost does not depend on the row count):

- All four widgets render only `scroll.visible_range()` rows; DataGrid
  only the columns in view. Frame cost at 500, 10k and 100k rows is the
  same within noise (DataGrid 120x40: 255 / 225 / 242 µs).
- Hit regions and focus entries are registered for visible rows and cells
  only (O(viewport) per frame, ~1,200 regions at 200x55).
- No per-frame sorting or filtering: DataTable sorts on `sort_by` /
  `set_rows`, DataGrid on `set_rows` with `local_sort`; TreeView
  flattens on expand/collapse/filter, not per frame.
- Event handling is ≤ 1.3 µs per event in every scenario.
- Building 100k-row datasets (not per frame): DataGrid 44 ms, DataTable
  85 ms, ListBox 7 ms, TreeView 10 ms, dominated by the caller's own
  string allocation.
