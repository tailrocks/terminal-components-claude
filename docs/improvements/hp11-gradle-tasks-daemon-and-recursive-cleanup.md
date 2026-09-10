# HP11 — Gradle tasks, daemon and recursive cleanup

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model installed/wrapper/missing states, tasks, recursive fixture candidates, daemon-stop prerequisites, Trash/dry-run and partial outcomes.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real Gradle commands, process probes/stops, filesystem traversal and cleanup.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve installed-Gradle clean/build/test and recursive .gradle/build cleanup with daemon stop. One wrapper-clean fixture does not cover these.

**Source evidence and mandatory scope:** [matrix HP11](../holla-parity-matrix.md#hp11--gradle-tasks-daemon-and-recursive-cleanup) — `OP44`, `OP45`, `OP46`, `OP47`, `OP48`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep Gradle command actions for build.gradle/build.gradle.kts with installed gradle. Wrapper preference may extend the new model but must retain legacy routes. Separate ordinary tool clean from depth-five Trash cleanup and global cache insights. Resolve exact candidates, review rebuild cost and show daemon stop as an explicit prerequisite; stop failure/unknown state must prevent unsafe cache deletion.

**Architecture / reusable components:** Share bounded candidate traversal with HP12 and deletion policy with HP20/HP21. Activity owns daemon state and cleanup report. Reuse tasks, candidates, Plan and facts; avoid standalone daemon widget.

**Required deterministic fixture:** `parity-gradle` — installed/wrapper-only/missing tool, both build files, clean/build/test success/failure, mixed build/.gradle candidates depth 5/6, symlink/node_modules exclusion, daemon active/stop failed, no candidates.

**Acceptance / automated verification:** Assert exact commands/cwd, selected directories only, no traversal through links, ignored node_modules, safe empty result, mixed failure report. Dry-run must not stop a daemon or mutate cleanup targets; explicit audit logging remains allowed. Distinguish tool-native clean from Trash and prerequisite failure from completed cleanup. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture three task choices, daemon-stop barrier, Trash candidate plan, dry-run and failed prerequisite.

**Dependencies:** HP07/HP12/HP14/HP18/HP20/HP21/HP22; F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
