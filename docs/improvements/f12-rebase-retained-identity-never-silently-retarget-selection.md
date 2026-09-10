# F12 — Rebase retained identity, never silently retarget selection

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete retained line identity reconciliation for selection, drag, viewport and producer caret. Integrate Holla output churn, resize and follow behavior.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No deferred implementation slice.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed wrong-source copy; P2 reading-position defect.**
`TextViewport::push`, `viewport.rs:171`, saturates selected line indices after
eviction without resolving removed identities; it omits drag and visible anchors.
Cap 3, A/B/C: select A, append D → copied selection becomes B. Press B, append D,
drag → wrong newline range. With follow off, visible B jumps to C although B
survives. Text and interaction agents independently reproduce all three.

Apply one edit delta/stable-line reconciliation to selection head/anchor, active
drag anchor, producer caret and first visible logical position. Clear entirely
evicted selections; define clipping of partially surviving ranges. Preserve
retained reading position while follow is off, then map back into wrapped rows.
Dataset replacement needs an explicit reset/identity contract, not only index
clamping. Risk: medium/high selection semantics. Acceptance: all three repros,
partial/multiline eviction, active drag, wrapped rows, resize, follow-on, empty
and zero retention. Existing passing logical selection across resize must remain.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
