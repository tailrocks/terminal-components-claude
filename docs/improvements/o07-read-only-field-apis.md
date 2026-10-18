# O07 — read-only field APIs

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

Deferred. Optional field-shaped read-only APIs are unnecessary while Holla composes existing input, viewport and editor primitives.

## Later

- [ ] Establish the retained product/consumer admission condition before implementation.
- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Classification:** optional opportunity · P3.

Correct F18 fixture now using existing viewport/editor. Require a field-shaped selectable noneditable consumer that cannot use them before extending all input APIs. Medium state/API risk; navigation/copy/mutation rejection must be specified together.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
