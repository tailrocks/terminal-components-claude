# Performance: scrolling tables, grids, lists and trees

PO request (2026-10-16): table components lag when scrolling. Plan entry:
`docs/visual-testing/plan.md` §P, task T31.

## How it is measured

Release profile, in-process `TestBackend` (no terminal I/O; a real
terminal adds the diff write and the emulator's parse for every frame).

| Harness | Run |
|---|---|
| Widgets alone (DataGrid, DataTable, ListBox, TreeView; 500 / 10k / 100k rows; 11 columns; long-cell variants; hover reveal of a long value) | `cargo nextest run --cargo-profile release --test table_perf --run-ignored only --no-capture` |
| TablePro app (customers: 48,912 rows, 500 loaded, 11 columns; query results of `SELECT * FROM customers`), per-event drawing against the runtime's `InputPump` | `cargo nextest run --cargo-profile release --bin tablepro --run-ignored only -E 'test(measure_table_surfaces)' --no-capture` |

Scenarios at 120x40 and 200x55: wheel burst, PageDown, End/Home, key
repeat Down, horizontal wheel, hover moves over rows, resize. The numbers
come from a shared developer machine with other agents compiling, so a
single figure varies by about ±50%. Read the ratios, not the absolutes.

## Findings

| # | Hotspot | Measured (before) | Root cause | Status |
|---|---|---|---|---|
| P1 | The event loop drew one full frame per state-changing event | Wheel burst of 60 notches on the customers table: 60 frames, 17.8 ms at 120x40 and 56.5 ms at 200x55. Draw costs about 0.25–1 ms per frame; handling costs about 1 µs per event. 120 key repeats: 33 ms / 128 ms. On a real terminal each of those frames is also written and parsed, so the screen keeps scrolling after the wheel stops. | `drain_ready_inputs` returned on the first `Changed`, and the loop drew before the next queued event. That is the right rule for activation, which must see newly rendered controls, but it was applied to every input. | **fixed** `39a19b4a9`; resize bursts in `perf(runtime): fold window-resize bursts` |
| P1b | A coalesced wheel burst stopped after one notch on the horizontal axis | Found by the identical-buffer guard: 4 wheel-right notches moved 1 column, not 4. | Wheel routing resolved to the topmost region, a cell or row id. `owns()` checks those ids against the *current* visible range, which the burst had already scrolled away. | **fixed** `39a19b4a9`: the four widgets also register their scroll region above their rows |
| P2 | Per-frame cost grew with the length of cut cell values | DataGrid, 2,000 rows with 64 KiB JSON/text cells: 19–24 ms per frame once those columns were on screen (0.25 ms otherwise). DataTable: 37 ms per frame at 200x55. | `truncate`, `fit` and the reveal's `is_cut` measured the full string for every visible cell. The grid also normalised the whole JSON document, sanitised up to 10,000 chars and cloned the full value twice per visible cell. | **fixed** `3dcff98fa` |
| P3 | Column width sampling read full values, and its size was hardcoded | `set_rows` with 64 KiB cells: 65 ms for 2,000 rows (200 sampled rows × 11 columns, each a full clone plus a full width). | `sample_widths` cloned and measured each value in full, although the result is clamped to the column max. | **fixed** `3dcff98fa` (`width_sample_rows`) |
| P4 | Row state scanned every pending cell for every visible row | O(visible rows × pending cells) per frame after a large paste. | Linear scan over a `HashMap`. | **fixed** `3dcff98fa`: the edited rows are collected once per frame |

Checked and not a problem (frame cost does not depend on the row count):

- All four widgets render only `scroll.visible_range()` rows, and DataGrid
  only the columns in view. Frame cost at 500, 10k and 100k rows is equal
  within noise. Example, DataGrid at 120x40 after the fixes: 157, 148 and
  164 µs.
- Hit regions and focus entries are registered for visible rows and
  cells only. That is O(viewport) per frame, about 1,200 regions at
  200x55. A guard asserts that 100k rows register exactly what 500 rows
  register.
- Nothing sorts or filters per frame. DataTable sorts on `sort_by` and
  `set_rows`, DataGrid on `set_rows` with `local_sort`, and TreeView
  flattens on expand, collapse and filter.
- Event handling costs at most 1.3 µs per event in every scenario.

## Before and after (release)

| Scenario | Before | After |
|---|---|---|
| customers table, 60-notch wheel burst, 120x40 | 60 frames, 17.8 ms | 1 frame, 0.16 ms |
| customers table, 60-notch wheel burst, 200x55 | 60 frames, 56.5 ms | 1 frame, 0.32 ms |
| customers table, 120 × Down key repeat, 200x55 | 120 frames, 128 ms | 2 frames (`max_batch` 64), 0.60 ms |
| customers table, 20 × PageDown, 200x55 | 20 frames, 13.1 ms | 1 frame, 0.28 ms |
| customers table, horizontal wheel × 20, 200x55 | 20 frames, 30.2 ms | 1 frame, 0.30 ms |
| customers table, hover sweep over the rows, 200x55 | 41 frames, 36.5 ms | 1 frame, 0.30 ms |
| query results, 60-notch wheel burst, 200x55 | 60 frames, 46.8 ms | 1 frame, 0.27 ms |
| window drag (30 resizes) | 30 frames (0.4–0.8 ms each) | 1 frame |
| End/Home alternating × 10 | 10 frames | 10 frames (different keys never fold, by design) |
| DataGrid frame, 64 KiB cells on screen, 120x40 / 200x55 | 19.3 / 23.7 ms | 0.26 / 0.62 ms |
| DataTable frame, 64 KiB cells on screen, 200x55 | 37.0 ms | 0.38 ms |
| DataGrid `set_rows`, 2,000 rows of 64 KiB cells | 65 ms | 22 ms (the rest is the caller's own allocation) |
| DataGrid / DataTable / ListBox / TreeView frame, 100k rows, 120x40 | 242 / 315 / 176 / 163 µs | 164 / 183 / 96 / 96 µs (the machine was less loaded; no algorithmic change) |

## Configuration (new knobs)

| Knob | Type | Default | Effect |
|---|---|---|---|
| `runtime::Application::coalesce` | `fn(&self) -> Coalesce` | `Coalesce::default()` | The app's input coalescing policy. It is read before each drain, so it can change at runtime. |
| `Coalesce::wheel` | `bool` | `true` | Wheel notches with the same modifiers, on either axis, draw once per burst. |
| `Coalesce::pointer_moves` | `bool` | `true` | Pointer moves draw once per burst. The last move is then handled again against the new hit regions. |
| `Coalesce::key_repeat` | `bool` | `true` | Repeats of one navigation key (arrows, PageUp/PageDown, Home/End; plain, Shift or Ctrl) draw once per burst. |
| `Coalesce::resize` | `bool` | `true` | Consecutive resizes draw once, at the last size. |
| `Coalesce::max_batch` | `usize` | `64` | Most events folded into one frame. |
| `Coalesce::max_latency` | `Duration` | `16 ms` | Longest a burst is handled before a frame is drawn. |
| `Coalesce::OFF` | const | | Draws after every change, as before. |
| `DataGrid::width_sample_rows` (field and builder) | `usize` | `DEFAULT_WIDTH_SAMPLE_ROWS` = 200 | Leading rows measured to size the columns, once per `set_rows`. |

`InputPump` (public) is the runtime's dispatcher, so apps can test their
own bursts with it. `drain_ready_inputs` keeps its uncoalesced behaviour.
`ui::text::wider_than` and `width_capped` are public helpers for bounded
measurement.

## Guards

Every guard counts something, except one, whose loose wall-clock budget
is documented in the test.

| Guard | Kind | Where |
|---|---|---|
| A 60-notch wheel burst draws 1 frame and every notch is handled. `OFF` draws 60. | count | `runtime::tests::a_wheel_burst_draws_one_frame_and_handles_every_notch` |
| 120 key repeats draw 2 frames (`max_batch`). With `key_repeat` off they draw 10 of 10. | count | `runtime::tests::a_held_key_draws_once_per_batch` |
| A window drag of 30 resizes draws 1 frame at the last size. | count | `runtime::tests::a_window_drag_draws_once_at_the_last_size` |
| Enter after a burst is held for the burst's frame. Enter never folds. | count | `runtime::tests::a_different_input_waits_for_the_frame_of_the_burst`, `queued_activation_waits_for_new_controls_to_render` (now through the default pump) |
| A hover sweep draws once and its last move is handled again. | count | `runtime::tests::a_pointer_sweep_draws_once_then_settles_hover_on_the_new_frame` |
| Bursts over the customers table draw 4 frames instead of more than 90. The final buffer is identical to per-event drawing (no visual change), at 120x40 and 200x55. | count + buffer equality | `tablepro perf_tests::input_bursts_over_the_customers_table_draw_once_and_land_identically` |
| Hit regions per frame are equal at 500 and 100k rows (all four widgets). | count | `table_perf::hit_regions_per_frame_do_not_grow_with_the_row_count` |
| The wheel routes to the widget for a whole burst on stale regions (all four widgets). | count | `table_perf::the_wheel_over_rows_routes_to_the_widget_for_the_whole_burst` |
| Cell text reads at most 4w+64 bytes of a 2 MB value. | count | `grid::tests::cell_text_reads_a_prefix_bounded_by_the_cell_width` |
| Bounded `truncate`, `truncate_middle`, `wider_than` and `cell_text` match the previous implementations on a width corpus (ZWJ, combining marks, CJK, variation selectors, control characters, long values). | equality | `ui::text::tests::bounded_measurement_matches_full_measurement`, `grid::tests::cell_text_from_a_prefix_matches_the_whole_value` |
| A frame of 128 KiB cells costs at most 8× a frame of short cells, plus 25 ms. Debug build before the fix: 798 ms against 5.9 ms. | wall-clock ratio, best of 5 | `table_perf::long_cut_values_cost_what_their_visible_part_costs` |

## Remaining risks

- Wheel notches in a burst are routed through the hit regions of the frame
  the burst started on. The four table-like widgets register their scroll
  region above their rows, so this is safe for them. A scroll container
  that resolves the wheel through child ids which depend on its scroll
  offset, without its own top scroll region, could drop notches within a
  burst. In that case the app sets `Coalesce { wheel: false, .. }`; the
  better fix is the same one-line registration in that widget.
- Hovering a cut value wraps the whole value for its reveal on every
  frame: about 0.25 ms extra per 64 KiB, linear in length. A multi-megabyte
  JSON cell under a resting pointer costs tens of ms per frame. The `↓ N
  more` count needs the total line count, so a bounded wrap must count
  lines without building them. Not done in this pass.
- Every frame is still a full render that ratatui diffs. Frame cost is
  O(viewport), not O(data): about 0.15–0.3 ms at 120x40 and 0.3–0.6 ms at
  200x55 in release, of which `Buffer` setup and grapheme-aware
  `set_string` are the bulk. Damage tracking was not needed once bursts
  coalesce.
- `TreeView::flatten` clones every visible label on expand, collapse and
  filter. That is O(rows), about 10 ms at 100k rows, per toggle and not per
  frame.
- The wall-clock guard could flake on a pathologically loaded runner. Its
  margin is more than 10× the measured gap, and the deterministic guards
  above cover the same fix.
