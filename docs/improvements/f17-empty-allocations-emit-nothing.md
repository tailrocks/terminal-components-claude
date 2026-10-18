# F17 — Empty allocations emit nothing

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

Deferred. Existing showcase Buttons zero-allocation bug. Current Holla and changed-widget containment checks remain required.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed containment defect.** Buttons draws State matrix headers without
checking positive inner height (`pages/buttons.rs:151`). At 72×20 Primary/Secondary
appear outside the zero-height allocation in the blank row below the page.
The row loop is bounded, separate metadata/header writes are not. Design image
review and ecosystem source verification agree.

Bound every composed section, including headers, metadata, hit regions and
cursor, at one allocation boundary; combine with F16 for reveal rather than
painting elsewhere. Risk: low for clipping, medium if combined with navigation.
Acceptance: nonzero-origin sentinel tests for zero/one/small dimensions leave
all outside cells untouched; blank row remains blank in the reproduced frame.
The shared wrap helper's `w.max(1)` behavior does not itself guarantee zero-area
containment—callers must enforce their allocation.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
