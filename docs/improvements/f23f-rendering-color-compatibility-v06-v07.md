# F23f — rendering/color compatibility (V06/V07)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Apply F22 checks and fresh-process palette/NO_COLOR/force-color/TERM assertions to Holla and changed shared primitives. Record actual terminal/font/tmux versions and scope.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

Unrelated application visual matrices and unavailable external-platform evidence remain deferred or explicitly pending.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Four palette buffers and one fresh-process NO_COLOR backend path; real font/terminal breadth unproven.

**Root fix / retained acceptance:** F22 fidelity checks plus subprocess matrix for explicit palette, NO_COLOR empty/nonempty, force-color and relevant TERM policy. Assert color SGR separately from reverse/DIM/bold/underline. Record real emulator/font/tmux versions.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
