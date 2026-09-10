# HP10 — Homebrew service lifecycle

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Model schemas, cache freshness, availability, limits, target identity and start/stop/restart results using fixture service data and in-memory cache state.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real brew services commands and durable service-cache I/O.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve Homebrew service discovery, cache and start/stop/restart. Existing remote systemd resources are a different provider.

**Source evidence and mandatory scope:** [matrix HP10](../holla-parity-matrix.md#hp10--homebrew-service-lifecycle) — `OP39`, `OP40`, `OP41`, `OP42`, `OP43`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Expose service resources on hosts with brew, including Linuxbrew. Support both array and services-array JSON forms, valid sorted unique names and all three verbs regardless initial status. Preserve 30-action/10-service bound visibly or allow explicit further discovery. Show cache age; refresh stale names before target-changing operations.

**Architecture / reusable components:** Versioned service-cache adapter preserves v1, 300-second TTL, tolerant corruption and atomic replacement. Bind action identity to service/host/verb and re-observe status after mutation. Reuse resource Picker/Props, alternatives and Activity.

**Required deterministic fixture:** `parity-brew-services` — both schemas, malformed rows, no brew, empty/failing command, fresh/stale/wrong-version cache, 11 services, started/stopped/error state and each verb.

**Acceptance / automated verification:** Assert cache hit avoids probe, expiry boundary/corruption refresh, cache write failure remains nonfatal, deterministic IDs/order/count, exact brew services argv and observed lifecycle outcome. Missing service after review must not retarget another row. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture macOS/Linuxbrew service alternatives, stale-cache refresh, failed stop and successful restart.

**Dependencies:** HP01/HP14/HP17/HP23; F02/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
