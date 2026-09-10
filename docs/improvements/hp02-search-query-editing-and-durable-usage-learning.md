# HP02 — Search, query editing and durable usage learning

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Implement query editing, ranking explanations, recents, decay/expiry, opt-out and store-error states with a virtual clock and in-memory serialized store. Simulate restart, corruption, migration and concurrent merge without reading or writing user history.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Durable usage-store I/O, real restart/migration and concurrent filesystem writer integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P2 · planned, not implemented.**

**Preserved capability / current gap:** Preserve fuzzy matching, query editing, recents, learned query choices and resilient persisted usage. Current usage is in memory; stored last-used timestamps do not establish decay.

**Source evidence and mandatory scope:** [matrix HP02](../holla-parity-matrix.md#hp02--search-query-editing-and-durable-usage-learning) — `L08`, `L10`, `L15`, `L30`, `L09`, `I-H01`, `I-H02`, `I-H03`, `I-H04`, `I-H05`, `I-H06`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Keep exact user aliases strongest. Preserve matching across label/group/description/keywords and grapheme-safe emphasis; learned choices must still match and must not authorize execution. Explain the difference between an explicit pin, a remembered query and contextual frequency. Retain five-item recent projection and truthful invocation history, including failed invocations; headless runs remain outside implicit learning. Implement visible query selection/undo/redo/word deletion.

**Architecture / reusable components:** Use a versioned usage-store adapter with deterministic clock, atomic merge and opt-out. Preserve v1 legacy reads or explicitly migrate that store into the new model without losing choices; writes retain concurrent updates. Reuse Input/Picker and existing ranking explanations. Do not confuse activity logs with usage.

**Required deterministic fixture:** `parity-history` — threshold/ties, exact alias versus learned query, whitespace/case normalization, Unicode, 20-use bound, 10-day decay, 90-day expiry, clock skew, corrupt/versioned store, two writers, save error, HOLLA_NO_HISTORY=1.

**Acceptance / automated verification:** Assert no unmatched learned results; bounded automatic frecency, deterministic ties and positive-only recents. Assert restart continuity, stale query pruning, opt-out performs neither reads nor writes, and save errors do not change action outcome. Exercise query Ctrl-A, Ctrl-Z, redo, word-delete and selected replacement. Never claim ignored clipboard requests as supported copy. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture Recent here, Why this result, changed ranking after restart, disabled-history status and Unicode match emphasis.

**Dependencies:** HP01/HP17; F08/F10/F23. Explicit pins remain new-product behavior, separate from automatic learning.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
