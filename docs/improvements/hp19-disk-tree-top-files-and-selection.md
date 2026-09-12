# HP19 — Disk tree, top files and selection

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build hierarchical root/path navigation, sort/fold/selection, percentages, cleanup routing and rescan on immutable fixture identities. Simulate Spotlight and Linux fallback outcomes.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live Spotlight queries/stat concurrency/timeouts, real disk traversal and physical post-cleanup scans.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve root overview/custom path, expanding size tree, noise folding, arbitrary selection and Spotlight top files. Current disk page is flat and ignores requested path.

**Source evidence and mandatory scope:** [matrix HP19](../parity/holla-parity-matrix.md#hp19--disk-tree-top-files-and-selection) — `LD001`, `LD002`, `LD003`, `LD012`, `LD013`, `LD014`, `LD015`, `LD020`, `LD021`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Make Here/selected/custom root explicit and validate the path with recoverable inline errors. Offer actual hierarchical allocated-largest-first tree, apparent-sort alternative, parent percentages and presentation-only noise folding. Space selects files/directories; parent dominates descendants without losing stricter safety policies. Keep paths/focus/checks stable across sorting/live updates and rescan after cleanup. Add macOS global Top files as a distinct scope with tree-scan fallback on Linux.

**Architecture / reusable components:** Use observed tree identities and selection closure, not indices. Top-files adapter models >=100 MiB query, top 50, exact paths, NUL-safe records, 16-way stat and five-second timeout; unavailable differs from empty. Reuse TreeView, Picker/Input, Props, progress and existing Disk composition.

**Required deterministic fixture:** `parity-disk-navigation` — home roots/cached sizes, custom file/dir/missing/relative input, nested large tree, all 14 folded names, re-sort, overlapping selection/disappearance, delete refresh; Spotlight duplicates/stat error/timeout/empty/Linux.

**Acceptance / automated verification:** Assert root-filtered observations, expand/collapse, size sort and honest displayed units, fold toggles preserve totals, arbitrary multi-selection counts/estimates and no target drift. Assert cleanup route/reset/rescan and independent global top-files selection through same safety gate. Exercise arrows/Enter/Space/sort/fold/rescan and pointer equivalents. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture custom-path error, expanded/folded tree, overlapping selection, post-cleanup rescan and Top files unavailable/empty/list states.

**Dependencies:** HP03/HP18/HP20/HP21/HP22/HP23; F02/F05/F23.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-disk-navigation` (`disk_navigation` in src/bin/holla/domain/parity.rs) · `hp19_tree_navigation_sorting_folding_selection_and_top_files` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/sim/fs.rs: scan_deduplicates_hardlinks_and_never_follows_links`, `src/widgets/tree.rs: delayed_children_above_the_cursor_keep_the_focused_node`, `src/bin/holla/domain/cleanup.rs: execution_is_truthful_across_modes_dedup_and_failures` (ancestor dedup only). · row proofs in src/bin/holla/app_tests_rows.rs: `ld002_ld003_the_overview_lists_home_folders_first_and_a_custom_path_is_validated`.

**What the journey asserts:**
- After `Analyze disk usage` the page reads "scan complete"; the `target` row (allocated 500 blocks) sits above `vm.img` (allocated 40 blocks, apparent 900).
- `s` shows "Sorted by apparent" and `vm.img` moves above `target`; `s` again restores the allocated order.
- With the cursor on the noise folder the status reads "noise folded"; `f` shows "unfolded"; `f` refolds.
- Home, Down, Down lands on "~/work/big/noise"; Space gives "1 item selected"; after unfolding, Right, Down, Space on a child gives "covered by a selected ancestor".
- `a` gives "Every visible entry selected"; `a` again gives "Everything unselected".
- `t` opens "Disk › Top files"; exactly one row contains both "iso.iso" and "MiB" (the duplicate Spotlight hit collapses); "big.mov" is listed; "vanished.bin" carries "stat failed"; "small.bin" (50 MiB) is absent.
- Space then `d` on a Top files row opens the shared gate ("gate 1 of 2" or "Review · trash").
- In the `parity-platforms-linux` world `Top files on this Mac` reports "unavailable · Spotlight is unavailable on Linux" and the tab stays `TabKind::Here`.
- Unit `scan_deduplicates_hardlinks_and_never_follows_links`: `a.allocated == BLOCK * (10 + 1 + 1)` (hardlink counted once), the symlink child has no children, sparse file is `(100 * BLOCK, 2 * BLOCK)` apparent/allocated, an unreadable folder yields `errors() == 1` with a positive partial total.
- Unit `delayed_children_above_the_cursor_keep_the_focused_node`: `cursor_path()` stays `[1]` after children are inserted above it; `selected` stays `[1, 0, 0]` when the cursor moves.

**Captures:** `shots/h_hp19_tree` (scan complete, allocated sort, "noise folded"), `shots/h_hp19_apparent` ("Sorted by apparent"), `shots/h_hp19_unfolded` (noise unfolded), `shots/h_hp19_selected` ("1 selected"), `shots/h_hp19_top_files` ("Disk › Top files" with "vanished.bin" "stat failed"); base matrix `shots/h_parity_disk_navigation_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| LD001 | `disk.usage` opens on the cwd root `~/work/big` ("scan complete", `h_hp19_tree`); alternatives `disk.overview`, `disk.scan-custom`, `disk.top-files` exist in catalog.rs but the journey exercises only `disk.usage` and `disk.top-files`. | real cwd resolution and filesystem traversal |
| LD002 | `ld002_ld003_the_overview_lists_home_folders_first_and_a_custom_path_is_validated`: "Disk overview" (`Here › Disk › Overview`) lists the home folders first in alphabetical order (`overview_rows()` kind `home folder`) followed by the detected insight roots. | live analysis from overview rows, cached sizes on disk |
| LD003 | `ld002_ld003_the_overview_lists_home_folders_first_and_a_custom_path_is_validated`: "Analyze a custom path…" opens the arguments page seeded with cwd ("must exist · relative paths are rejected"); `relative/x` is refused with "the path must be absolute", `/Users/alex/nowhere` with "no such file or directory", and `/Users/alex/work/big` starts the scan and opens `Disk usage · ~/work/big`. Esc cancels the page (`ArgsPage`). | real path existence check |
| LD012 | allocated order `target < vm.img` then "Sorted by apparent" with `vm.img < target`; Down/Right/Home navigation; percentages not asserted (rendered by `disk.rs` row meta only). | real allocated/apparent stat |
| LD013 | fixture creates all 14 `NOISE_NAMES`; "noise folded" then "unfolded" then refold; totals unchanged is not asserted numerically. | none |
| LD014 | "1 item selected", "covered by a selected ancestor", "Every visible entry selected", "Everything unselected"; unit `delayed_children_above_the_cursor_keep_the_focused_node` keeps `cursor_path() == [1]`; disappearance is not exercised here (see HP21 `gone-later`). | none |
| LD015 | `d` from Top files opens "gate 1 of 2"; post-cleanup rescan and selection reset are not asserted by `hp19_*`. | physical post-cleanup rescan |
| LD020 | "Disk › Top files": one `iso.iso` row (dedup), `big.mov`, `vanished.bin` "stat failed", `small.bin` absent (below 100 MiB); top-50 cap and NUL-safe records not asserted. | live `mdfind` query, real stat |
| LD021 | Linux: "unavailable · Spotlight is unavailable on Linux", tab stays `TabKind::Here`; macOS timeout is asserted in `hp23_*` ("did not finish within 5 s"); empty-list state not asserted. | killable query, 16-way stat concurrency, 5 s timer |

**Deferred remainder:**
- Live Spotlight queries, stat concurrency and timeouts; real disk traversal; physical post-cleanup scans (Later section).
- Acceptance clauses not proven by the tests: custom-path validation errors, overview rows with cached size/age, parent percentages, fold-preserves-totals numerically, rescan after cleanup, Top files empty state, pointer equivalents on the disk tree (HP21 and HP22 click the tree; `hp19_*` is keyboard only).

**Limits:**
- The simulated `Fs` models allocated/apparent/sparse bytes; no real APFS or ext4 numbers are measured.
- Spotlight availability, duplicates and the vanished file come from `Spotlight::Available` fixture data, not from `mdfind`.
