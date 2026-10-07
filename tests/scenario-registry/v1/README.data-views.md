# DATA-VIEWS Scenario Registry

This file describes the DATA-VIEWS group registry.
It uses simple sentences.
Each sentence holds one fact.

## Files

- `scenario-registry.data-views.json` holds 30 rows.
  It covers 7 components: Grid, CodeEditor, DiffView,
  TextViewport, TerminalView, ScrollRegion, DataTable.
  It uses the same stable IDs as `registry.json`.
- `validate-data-views.py` checks the registry.
  Run it from the repo root: `python3 tests/scenario-registry/v1/validate-data-views.py`.
  It must print `PASS`.

## Words

- **Reference**: the code before the refactor.
  It lives on branch `visual-baseline`.
  Its frozen tag is `visual-baseline`.
  All source symbols come from the Reference.
- **Candidate**: the code after the refactor.
  It lives on branch `termrock-implementation`.
  The Candidate must show the same output as the Reference.
- **Coverage row**: one test case in the registry.
  It has a stable ID.
  It names the application, the component, and the part.
  It lists input steps and checkpoints.
  It lists visual, state, action, and negative checks.
  It names the source symbols and the snapshot files.
- **Checkpoint**: one named point in a Coverage row.
  The test stops at the Checkpoint.
  The test captures the screen at the Checkpoint.
  The test compares the capture with the snapshot.
- **PTY**: a pseudo-terminal.
  It runs the application as a real terminal runs it.
  It sends keys and reads screen output.
  Tests that need a PTY use the `pty-capture` adapter.
  Tests that draw to a buffer use the `headless-tick` adapter.
- **Grid**: the `DataGrid` widget with typed cells.
  Enter edits, Bool toggles inline, Json opens the viewer.
  Edits queue as pending changes with Preview, Discard and Save.
  `s` cycles the sort, `u` undoes, `Ctrl+S` commits.
- **CodeEditor**: a document editor with a gutter.
  `i` edits, `Esc` commits, `{` and `}` jump between blocks.
  The gutter holds the bar, the marker, and the numbers.
  `Ctrl+R` runs the block; runs may add diagnostics.
- **DiffView**: one file in unified or review presentation.
  Unified lists `+`/`-` rows with old and new numbers.
  Review pairs old and new columns with bold changed runs.
  Narrow panes fall back to unified automatically.
- **TextViewport**: selectable read-only styled lines.
  It owns retention, follow, wrap, marks, and copy.
  Jackin panes and the holla preview are TextViewports.
  Copy returns source bytes, never display text.
- **TerminalView**: the showcase terminal page composition.
  No widget of that name exists.
  A TextViewport holds the launch transcript with follow.
  Ticks advance the seven stages; `y` copies a selection.
- **ScrollRegion**: the scroll model plus its painters.
  No widget of that name exists.
  `ScrollState` holds offset, content, and viewport lengths.
  The scrollbar draws the thumb; presses grab, drags follow.
- **DataTable**: the `DataTable` widget with sortable headers.
  Enter selects, `s` sorts, hover lifts the row.
  Editable columns open an in-place cell editor.
  Commits validate and emit events; there is no pending queue.

## Rules

- Do not change a stable ID.
- Do not change `ref_source` without reading the source.
- Candidate symbols stay empty until Stage B knows them.
- Mapped legacy roots keep their old path.
  They point to the new row ID.
- Pending roots have no row yet.
  Keep them in the list.
- Some roots are shared with other groups.
  Each group pins a different widget on the same page.
  The entry note names the overlap.
- Grid rows cite `grid.rs`.
  DataTable pages and the tables roots are not DataGrids.
- DataTable rows cite `table.rs`.
  They pin sorting, selection, hover, and cell editing.
- TerminalView rows cite `viewport.rs` and `terminal.rs`.
  The seam and the step rail stay owned elsewhere.
- ScrollRegion rows cite `scroll.rs` and `scrollbar.rs`.
  The list column stays owned by NAVIGATION.

## Candidate port

- This file is the `termrock-implementation` port of the Reference slice.
- Source: `visual-baseline` commit `82fb265b2667a0064cd2444dc7870f86fae195eb`.
- The 18 carried rows and their `registry.json` entries are byte-identical to the source.
- Phase-8d added the 12 S1 rows from `76537c27e6e00ef93a2d09d280c8c9cb6097e446`.
  Their bodies match the source; `results` hold the Candidate run.
- The validator resolves Reference paths through git at `ref_commit`.
  The Candidate worktree holds no `src/` or `snapshots/` copies.
