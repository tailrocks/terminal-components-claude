# HP09 — Docker and Compose outcomes

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Complete each standalone and compound action against the simulated daemon/project inventory. Prove dependency barriers, drift gates, exact IDs/argv and distinct no-op/failure/success effects.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Docker daemon access, real Compose/log streams and destructive container/image/volume operations.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Compose up/down/finite logs, container stop-and-remove, full cleanup and builder prune. Full cleanup graph alone does not cover standalone variants.

**Source evidence and mandatory scope:** [matrix HP09](../holla-parity-matrix.md#hp09--docker-and-compose-outcomes) — `OP31`, `OP32`, `OP33`, `OP34`, `OP35`, `OP36`, `OP37`, `OP38`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Separate Compose project from host container resources. Include all four manifest names, docker executable/daemon/plugin states and finite logs --tail 200 independently of follow logs. Legacy docker.stop-all stops and removes captured running/stopped IDs: offer an explicit Stop and remove all plan retaining that outcome, alongside new stop-only/remove-only alternatives. Full cleanup includes all images, network/system/volume prune; builder prune remains independently callable. Explain named-volume and buildx expansion separately.

**Architecture / reusable components:** Resolve immutable Docker identities into typed argv and dependency barriers. Remove shell substitution, swallowed image errors and preview/execution drift; model effects for every standalone variant. Do not run remove after failed stop or use force implicitly. Use existing resource snapshots, plans, facts and activities.

**Required deterministic fixture:** `parity-docker` — empty daemon, stopped+running containers, daemon/query failure, IDs changed after review, failed stop/remove/image stage, finite versus follow logs, Compose down without volumes, standalone builder prune, complete cleanup.

**Acceptance / automated verification:** Assert exact target classes/argv and scoped counts, successful no-op versus discovery failure, per-stage effects and mixed results. Compare every standalone action with its own world mutation. Require reauthorization after target drift, truthful permanent deletion, failure visibility and dependency blocking while independent prune branches remain independent. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture each standalone variant review/result, finite log completion, daemon failure, stop failure barrier and full cleanup final inventory.

**Dependencies:** HP01/HP14/HP15/HP17/HP21/HP22/HP23; F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
