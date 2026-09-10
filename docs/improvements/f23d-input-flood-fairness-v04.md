# F23d — input-flood fairness (V04)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Measure the shared event pump using finite queues and bounded Holla PTY input floods. Define tick/quit fairness and change bounded draining only if measured behavior violates the contract.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No unrelated benchmark project; a source-only risk is not a confirmed freeze.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Ignored/consumed events drain until empty before tick checks. Source risk only; no sustained starvation measured.

**Root fix / retained acceptance:** Finite injected queue and bounded owned-PTY flood measure dispatch/tick/quit latency. Establish fairness contract, then implement bounded draining if it fails. Do not label absent sustained-flood proof a confirmed freeze.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
