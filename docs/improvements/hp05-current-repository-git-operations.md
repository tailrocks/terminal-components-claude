# HP05 — Current-repository Git operations

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Provide full status, strategy review, exact repository/cwd/argv and distinct success/failure effects in the simulated Git world.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real git invocation, credentials, remotes and repository integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve current Git pull, push and full status. Preview pull is ff-only, status is a snapshot and push has no modeled effect.

**Source evidence and mandatory scope:** [matrix HP05](../holla-parity-matrix.md#hp05--current-repository-git-operations) — `OP01`, `OP02`, `OP03`, `OP04`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** A Git resource exposes full status and synchronization alternatives with effective repository/cwd. Retain fast-forward default while offering the ordinary configured pull behavior after explicit conflict/rebase/merge review; do not silently remove configurations that old git pull supports. Push must change modeled remote state and report rejection/auth/upstream failures. Status retains the detail users obtain from full git status.

**Architecture / reusable components:** Separate repository identity, operation argv and result/effect; remove success-only generic fallbacks for these actions. Reuse snapshot Props/TreeView and activity TextViewport; no Git-specific shared widget.

**Required deterministic fixture:** `parity-git-current` — .git file/dir, subdirectory context distinction, clean/dirty/behind/diverged/detached/rebase, no upstream, rejected push, merge-configured pull and command failure.

**Acceptance / automated verification:** Assert correct repository/cwd for discovery and invocation, preview/argv equality, no push effect on failure and observed state after success. Assert full status detail and truthful unavailable-state recovery. New scope expansion may extend local marker discovery without dropping the old current-folder route. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture full status, pull strategy review, push rejection and successful local/remote before-after state.

**Dependencies:** HP01/HP14/HP15/HP17; F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
