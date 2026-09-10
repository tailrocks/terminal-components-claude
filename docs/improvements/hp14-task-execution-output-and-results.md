# HP14 — Task execution, output and results

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Implement typed simulated action outcomes, sequential/parallel scheduling, ordered final bytes, per-task retained output and completion independent of page dismissal. Unknown scripts cannot generically succeed.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Production executor/adapters, actual child processes and aggregate/headless CLI contracts.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve real task specifications, sequential/parallel scheduling, live output, focus and summaries in activities. Current fixtures cover selected scripts, not all execution semantics.

**Source evidence and mandatory scope:** [matrix HP14](../holla-parity-matrix.md#hp14--task-execution-output-and-results) — `L23`, `E25`, `E26`, `E27`, `E28`, `E30`, `E31`, `E32`, `E36`, `E41`, `E42`, `E44`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model queued/running/cancelling/succeeded/failed/cancelled with exact task identity, cwd/host and result. Independent batch jobs continue after peer failure; dependent plan steps block. Preserve per-task retained output, manual reading position and follow-tail, post-completion inspection, empty batch and final shell summary. Every mutating action needs an explicit effect or failure; unknown scripts must not silently succeed.

**Architecture / reusable components:** Replace generic-success fallback and incomplete effect-by-ID switch with typed execution events and action-owned outcomes. Separate runner completion from UI dismissal. Reuse Activities, tabs, TextViewport, StatusBar and Plan; adopt F10–F15 retention contracts.

**Required deterministic fixture:** `parity-executor` — sequential and parallel jobs, first failure, queued cancellation, missing executable/cwd, CRLF/ANSI/invalid UTF-8, no-final-newline, fast exit, stream interleave, burst retention, scrolled output, empty jobs and mixed results.

**Acceptance / automated verification:** Assert final buffered bytes arrive before terminal state, per-task ordered output, no cross-task retargeting, tail preservation, return-to-Here continuity and task/page focus bindings. Assert aggregate CLI failure and truthful UI result even after dismissing output. Test independent versus dependent failure policies separately. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture pending/running/mixed failures, retained scrolled output, final unterminated output, empty batch and completed task after returning from Here.

**Dependencies:** HP15/HP16/HP17; F10–F15/F19/F20/F21/F23. Task ownership, cancellation, stale-result rejection and shutdown are required; a generic worker framework is outside this plan.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
