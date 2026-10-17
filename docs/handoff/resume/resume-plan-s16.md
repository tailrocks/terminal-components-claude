> **Status 2026-10-18: open.** Work is on `origin/wip/plan-s16` at `b249a81be` (one ungated WIP commit on an older base; rebase first). Next steps: [../README.md](../README.md). The scratchpad paths below no longer exist; the edit scripts were already applied.

I stopped as asked. Nothing was pushed: the gate never ran, so I have no fully-gated commit to push.

**Done so far:** one local WIP commit, `b249a81be`, on branch `plan-s16`. It builds (`cargo build --all-targets` passes), but nothing else has been run. It holds the S16a library work:
- **`RowMark`** (in `src/ui/row.rs`): pointer `›`, unsaved-changes `•` and partly checked `–` marks, with `RowPaint::dirty` and `RowPaint::status` to draw them.
- **Header rows:** `RowSource::is_header` in `src/core/list.rs`. A header never takes the cursor, even with `skip_disabled` off.
- **`ListItem`:** `header()` (cursor, type-ahead, Select All and clicks pass over it), `status(Tone)` / `status_glyph` (the glyph sits in the mark column, `!` for an error), `mark(RowMark)` and `default_status_glyph`.
- **`ListBox`:** `dirty_lane(bool)` reserves 2 cells so labels don't move when a row turns dirty; `set_mark`; the cursor moves off a header row when the list draws. In multi mode a partly checked row shows `–`, and checking it checks it fully. This is the part P0-G1 needs.
- **Tables:** `Column::marker()` draws each row's marks; `DataTable::set_mark` / `mark`; `Cell::strong` makes a cell bold.
- **Trees:** `TreeNode` gets `meta_tone`, `trailing(glyph, Tone)` and `cells(Vec<Cell>)`. `TreeView` gets `columns(Vec<Column>)` and `column_title`: a header row, right-aligned cells with one blank between columns, sharing the strip with `reserve_right`. The strip is dropped when the labels would get narrower than `MIN_LABEL`. This is the part P0-G2 and the EXPLAIN tree-table need.
- **Catalog:** `src/catalog.rs` has the 20 new capability lines.

**Not done (resume here, in order):**
1. **Tests and coverage gate.** The `catalog` tests `gate_fails_when_a_new_builder_has_no_coverage` and `gate_fails_when_a_new_public_function_has_no_coverage` fail. Each of the 20 new ids needs either a declaration in `src/bin/showcase/coverage.rs` or an allowlist line `id | reason | SC35` in `tests/showcase_coverage_allow.txt`. I also wrote unit tests (`row_extension_tests` in `list.rs`, `tree_table_tests` in `tree.rs`), but the command that appended them was refused, so they are not in the commit and have to be added again. Then run clippy, fmt, nextest and the visual gate, and push the library.
2. **S16d tablepro EXPLAIN** (`src/bin/tablepro/tabs.rs` around lines 1865–1975): move to `TreeView::columns` with cost 13, rows 8, actual 10, % 4 and a 1-wide hot-marker column. The cell tone and bold rules differ a little from the current hand-drawn ones (the focused row is no longer bold), so captures will change and must be attributed. Also drop the `meta` index parsing.
3. **Showcase:**
   - P0-G1: per-row partial check on the Disk cleanup categories (`pages/disk_cleanup.rs`).
   - P0-G2: a real size column on the Disk cleanup usage tree.
   - EXPLAIN tree-table on the Query workbench page.
   - SC35 demos on Lists & trees, then delete the allowlist lines and update the `docs/remaining-work.md` and plan rows.
4. **Not started:** S16b holla and S16c jackin row migrations (the 69 list-key arms, CO11, CO10 bars), CO1 router guard, the 11 `y Copy` literals, and the tablepro history `!` overdraw moving to `ListItem::status`.

The edit scripts I used (`e1.py`–`e6.py`) are in `/private/tmp/claude-501/-Users-donbeave-Projects-terminal-components-claude/42325d29-0331-4e82-a42c-409070ca54fc/scratchpad/plan-s16/` and have already been applied. I did not take the push lock.