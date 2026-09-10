# HP06 — Repository batches, mirrors and hygiene

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Build sibling/mirror/hygiene plans with exact targets, bounded branch review, revalidation and mixed outcomes on fixture repositories/worktrees.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real repository discovery, git operations, branch deletion and worktree/remote checks.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve sibling-repository pull/push/status, origin+GitLab mirroring and Git hygiene. Existing child plans cover only part of these outcomes.

**Source evidence and mandatory scope:** [matrix HP06](../holla-parity-matrix.md#hp06--repository-batches-mirrors-and-hygiene) — `OP05`, `OP06`, `OP08`, `OP10`, `OP11`, `OP07`, `OP09`, `OP12`, `OP13`, `OP14`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Show immediate discovered repositories as resources, including legacy multi-repo threshold behavior as a fixture; new scope controls may expose one repository too. Build plans for parallel pull/push, sequential status --short and origin plus optional GitLab pushes. Label mirror scope exactly; do not call arbitrary remotes included. Offer fetch --prune and gc independently of merged-branch availability. Resolve origin/HEAD then main/master fallback; explain unavailable default.

**Architecture / reusable components:** Use canonical repository/worktree identity, per-repo remote identities and explicit concurrency/failure policies. Merged cleanup reviews sorted unique candidates, current/default exclusions and visible 30-of-N bound; execute git branch -d --, never implicit force. Revalidate branch/worktree/merge state before execution. Reuse Picker, Plan/StepRail and existing facts dialogs.

**Required deterministic fixture:** `parity-git-batch` — zero/one/many immediate repositories, colliding leaf names, failed repo, absent/present GitLab, custom default, no default, no merged branches, >30 branches, branch occupied by another worktree, stale merge eligibility.

**Acceptance / automated verification:** Assert every intended repo/remote operation and cwd, independence versus ordered status, retained per-repo failure output, no hard-coded default, exact delete argv and protected current/default branches. A failed prerequisite blocks only dependents; independent batch peers still run. Test stale confirmation revocation and Git refusal without -D fallback. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture batch selection, mirror preview, mixed results, capped branch review and stale/worktree refusal.

**Dependencies:** HP01/HP05/HP14/HP15/HP21; F02/F05/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
