# O05 — terminal extensions/OS clipboard

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

Deferred. Optional terminal protocols and real OS clipboard transport are outside the simulated preview; truthful simulated copy behavior remains required.

## Later

- [ ] Establish the retained product/consumer admission condition before implementation.
- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Classification:** optional opportunity · P3.

Explicit-width protocols, synchronized output, keyboard enhancement and real clipboard transport are not automatically required. First reproduce a named terminal/backend problem or present consumer need, verify capability/fallback and cleanup, and preserve preview side-effect boundaries. Medium/high compatibility/privacy risk. F21's proven suspend defect is not deferred under this item.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
