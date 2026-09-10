# HP12 — IntelliJ metadata cleanup

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model IDEA applicability, bounded fixture candidate traversal, exact-path review, no-match, mixed failures and log-health states using shared simulated cleanup policy.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real IDEA/filesystem discovery, Trash executor and operation-log writes.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve IntelliJ .idea/.iml cleanup and shared bounded filesystem discovery. The preview has no corresponding provider.

**Source evidence and mandatory scope:** [matrix HP12](../holla-parity-matrix.md#hp12--intellij-metadata-cleanup) — `OP49`, `OP50`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Discover when .idea exists or idea is installed, then review exact .idea directories and lowercase .iml files under cwd to depth five. Keep no-match success visible. Do not imply IDE shutdown exists in the baseline; new process-aware guard policy must explain any additional block. Combine overlapping candidates once and preserve per-path errors.

**Architecture / reusable components:** One candidate walker serves IDEA and recursive Gradle: skips symlinks/node_modules, stops descending selected directories, stable sorted paths. One validated Trash executor handles mutation and report/log health. Reuse Disk candidate rows, Props, facts and Plan.

**Required deterministic fixture:** `parity-idea` — .idea marker versus executable, .iml-only applicability, depth-five boundary, nested .idea, mixed files, node_modules/symlink decoys, missing/unreadable entries, failed/skipped/log-failed deletion.

**Acceptance / automated verification:** Assert exact detection/enumeration, no duplicate/nested deletion, no-match no-op and up-to-five error summaries with full report access. Both callers must use the same safety boundary; do not silently discard logging failure or treat skipped targets as full success. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture IDE cleanup discovery, exact paths/Trash facts and partial-failure activity with log-health detail.

**Dependencies:** HP01/HP18/HP20/HP21/HP22; F02/F05/F23. HP11 shares this traversal contract.

## Evidence

**Slice status:** current simulated slice complete · Later operational clauses open.

**Scenario and journey:** `parity-idea` (fixture fn `idea` in src/bin/holla/domain/parity.rs) · `hp12_intellij_cleanup_finds_metadata_case_insensitively_and_logs_a_failed_log_write` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/cleanup.rs: artifacts_need_an_indicator_and_never_nest` (shared `walk_candidates`), `src/bin/holla/domain/cleanup.rs: execution_is_truthful_across_modes_dedup_and_failures` (dedup, per-path failure and log-write failure reporting).

**What the journey asserts:**
- The `idea.clean` label contains `44.0 KiB`.
- Candidate rows contain `app.iml`, `~/work/ide/.idea`, `mod/mod.iml` and `a/b/c/d/e/deep`.
- Rows never contain `Upper` (the `.IML` file is not metadata), `e/f/g` (beyond depth 5), `ide/node_modules`, `elsewhere` (behind the symlink) or `nested` (inside a selected `.idea`).
- The page reports the unreadable folder: text contains `locked`; `4 selected` is shown.
- After gate 1 and the gate 2 phrase `TRASH {n} UNDER /Users/alex/work/ide ON mbp`, the text contains `report` and `write failures` or `EROFS`.
- `/Users/alex/work/ide/app.iml` no longer exists; `/Users/alex/work/ide/node_modules/x/.idea/x.xml` still exists.
- The last `world.reports` entry has `log_failures > 0` and at least 4 `Outcome::Trashed` items.
- Unit (cleanup.rs): `walk_candidates` returns only `Projects/rs/target` for a `/target` selector and nothing for `index.js` ("node_modules is never entered"); execution logs `duplicate` and `covered by its ancestor` skips, counts one `Outcome::Failed`, and with `write_failure = Some("EROFS")` the report has `log_failures == 1`, summary contains `1 log write failures`, and the deletion is not rolled back.

**Captures:** `shots/h_hp12_cleanup` (cleanup page: four IntelliJ candidates, `depth 5 · no symlinks · no node_modules`, `Unreadable 1 folder skipped` naming `~/work/ide/locked`) · `shots/h_hp12_gate2` (gate 2 dialog over the review: Trash mode, exact paths) · base matrix `shots/h_parity_idea_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| OP49 | Journey: applicability from the `.idea` marker, label `44.0 KiB`, exact candidates `app.iml`, `.idea`, `mod/mod.iml`, `a/b/c/d/e/deep`, `Upper.IML` excluded, Trash gate phrase `TRASH {n} UNDER /Users/alex/work/ide ON mbp`, `app.iml` removed and at least 4 `Trashed`; frames `h_hp12_cleanup`, `h_hp12_gate2`. Applicability by the `idea` executable alone is not asserted. | Real IDEA/filesystem discovery and Trash executor. |
| OP50 | Journey: depth bound (`e/f/g` absent), symlink (`elsewhere`) and `node_modules` never entered, nested `.idea` not duplicated, `locked` reported as unreadable; unit: `walk_candidates` never enters `node_modules`, execution logs `duplicate`/`covered by its ancestor` skips, `log_failures == 1`, `1 log write failures` in the summary. No-match no-op is not asserted. | Real traversal, per-path Trash errors and operation-log writes. |

**Deferred remainder:**
- Real IDEA/filesystem discovery, Trash executor and operation-log writes (Later).
- Not proven by tests: applicability via the `idea` executable without `.idea`, `.iml`-only applicability, the no-match no-op, a failed or skipped deletion in this journey (the fixture yields a log failure only), and the up-to-five error summary with full report access.
- No capture of the partial-failure activity with the log-health detail.

**Limits:**
- The unreadable folder, symlink and log failure are simulated filesystem flags; no real Trash or log file is touched.
- Frames are being regenerated; their text is indicative until the integrator records the review.
