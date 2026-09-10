# HP19 — Disk tree, top files and selection

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build hierarchical root/path navigation, sort/fold/selection, percentages, cleanup routing and rescan on immutable fixture identities. Simulate Spotlight and Linux fallback outcomes.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Live Spotlight queries/stat concurrency/timeouts, real disk traversal and physical post-cleanup scans.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve root overview/custom path, expanding size tree, noise folding, arbitrary selection and Spotlight top files. Current disk page is flat and ignores requested path.

**Source evidence and mandatory scope:** [matrix HP19](../holla-parity-matrix.md#hp19--disk-tree-top-files-and-selection) — `LD001`, `LD002`, `LD003`, `LD012`, `LD013`, `LD014`, `LD015`, `LD020`, `LD021`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Make Here/selected/custom root explicit and validate the path with recoverable inline errors. Offer actual hierarchical allocated-largest-first tree, apparent-sort alternative, parent percentages and presentation-only noise folding. Space selects files/directories; parent dominates descendants without losing stricter safety policies. Keep paths/focus/checks stable across sorting/live updates and rescan after cleanup. Add macOS global Top files as a distinct scope with tree-scan fallback on Linux.

**Architecture / reusable components:** Use observed tree identities and selection closure, not indices. Top-files adapter models >=100 MiB query, top 50, exact paths, NUL-safe records, 16-way stat and five-second timeout; unavailable differs from empty. Reuse TreeView, Picker/Input, Props, progress and existing Disk composition.

**Required deterministic fixture:** `parity-disk-navigation` — home roots/cached sizes, custom file/dir/missing/relative input, nested large tree, all 14 folded names, re-sort, overlapping selection/disappearance, delete refresh; Spotlight duplicates/stat error/timeout/empty/Linux.

**Acceptance / automated verification:** Assert root-filtered observations, expand/collapse, size sort and honest displayed units, fold toggles preserve totals, arbitrary multi-selection counts/estimates and no target drift. Assert cleanup route/reset/rescan and independent global top-files selection through same safety gate. Exercise arrows/Enter/Space/sort/fold/rescan and pointer equivalents. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture custom-path error, expanded/folded tree, overlapping selection, post-cleanup rescan and Top files unavailable/empty/list states.

**Dependencies:** HP03/HP18/HP20/HP21/HP22/HP23; F02/F05/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
