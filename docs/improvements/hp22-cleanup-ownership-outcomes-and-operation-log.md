# HP22 — Cleanup ownership, outcomes and operation log

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Keep simulated execution/report ownership outside modal lifetime. Model mixed per-path outcomes, quit-wait, in-memory operation logs, log failure, restart and honest Trash-versus-physical capacity.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real cleanup worker ownership, durable JSONL logging and physical capacity/OS integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve cleanup progress, mixed outcomes, estimates, per-item operation logs and ownership. Current history records only simulated successful removal and falsely frees Trash space immediately.

**Source evidence and mandatory scope:** [matrix HP22](../holla-parity-matrix.md#hp22--cleanup-ownership-outcomes-and-operation-log) — `LD016`, `LD059`, `LD060`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep cleanup in an activity independent of modal lifetime. Before irreversible commit allow cancellation; after commit keep execution/report ownership until settled and make quit wait explicit. Report removed/trashed/would-remove/failed/skipped separately, with counts, path/reason detail and log health. Same-volume Trash preserves filesystem used bytes until emptied; estimates and measured capacity change remain separate.

**Architecture / reusable components:** Typed result carries mode, dry-run, selected/executed identities, estimated size and observed outcome through worker, UI and log. Preserve JSONL v1 per requested item (including duplicates/protected/process-skipped) at XDG cache or ~/.cache/holla/ops.log: timestamp_ms/mode/path/size/outcome/error. Log failure is visible and does not invent rollback of completed deletion. Keep worker handle/report outside confirmation enum.

**Required deterministic fixture:** `parity-cleanup-results` — mixed success/fail/skip, duplicate, permission change, vanished target, process skip, dry-run, log write failure, receiver failure, result arrives during quit dialog, Stay/Leave and Trash versus permanent capacity.

**Acceptance / automated verification:** Assert every requested path gets one correct audit outcome; no success-only record after failure, correct first-six/overflow/full-log summary, persistent restart readability and JSON escaping. Assert no lost worker/report on modal replacement; quit completion is acknowledged. Apply only successful effects and rescan; reflect APFS clone/purgeable/hardlink estimation limits honestly. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture running/quit-wait/finished, mixed report with log failure, dry-run history and Trash estimated-versus-physical capacity.

**Dependencies:** HP14/HP15/HP18/HP20/HP21/HP23; F10–F15/F21/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
