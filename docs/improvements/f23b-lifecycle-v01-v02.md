# F23b — lifecycle (V01/V02)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Retain the F21 owned-PTY lifecycle matrix with exact mode/termios assertions, bounded cleanup and scoped platform claims.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

Task-process lifecycle integration under HP15 is deferred; it is distinct from restoring the preview terminal.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Guard/panic/idempotence units and historical PTY checks; fresh F21 suspension failure.

**Root fix / retained acceptance:** F21 owned-PTY matrix, exact mode/termios assertions and scoped best-effort documentation. Counter callbacks are not actual terminal proof.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
