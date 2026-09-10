# HP12 — IntelliJ metadata cleanup

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model IDEA applicability, bounded fixture candidate traversal, exact-path review, no-match, mixed failures and log-health states using shared simulated cleanup policy.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

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

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
