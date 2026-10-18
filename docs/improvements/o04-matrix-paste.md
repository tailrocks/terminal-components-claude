# O04 — matrix paste

## Status

Active specification or plan; completion is not implied by this document.

## Authority

design/interaction-polish defines UI/UX, colors, and behavior; main defines the intended split-crate architecture and reusable API direction.

## Current evidence

Claims below are scoped to their cited sources and recorded test results.

## Remaining work

Keep all listed requirements and open tasks in scope; track implementation state in docs/remaining-work.md.

## Verification

Verify against current sources and task acceptance criteria; historical passes are not current proof.


[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. Optional matrix paste has no accepted current Holla product requirement.

## Later

- [ ] Establish the retained product/consumer admission condition before implementation.
- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Classification:** optional opportunity · P3.

Grid exports TSV-like ranges but pastes into one single-line cell; `A\tB\r\nC\tD` becomes `A\tBC\tD` under existing newline normalization. No matrix-import promise exists. Document current scope; only add import after explicit product choice with parser/type/bounds/readonly validation, atomic pending updates, undo and overwrite confirmation. High data-loss risk; not a routine Unicode patch.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
