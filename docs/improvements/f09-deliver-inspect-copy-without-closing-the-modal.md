# F09 — Deliver Inspect copy without closing the modal

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. Jackin Inspect copy-effect routing; Holla copy delivery must use its own owner and current tests.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed defect by complete source trace, not full-app replay.**
`InspectChanges` advertises `y Copy` but discards `DiffView::on_key` events at
`src/bin/jackin_preview/screens/inspect.rs:250/343`. `CustomModal` lacks a
nonclosing effect route; `Request::Copy` already owns preview clipboard updates
at `src/bin/jackin_preview/app.rs:1964`. Interaction agent and main independently
traced the entire missing path.

Add a narrow typed nonclosing effect/request path through the existing custom
modal interface. Forward copy to the existing owner, preserve selection/focus
and keep Inspect open. Risk: medium internal interface risk; inspect every
CustomModal implementation and preserve close/done semantics. Acceptance:
compact/open-file and advanced/diff mouse selection → y delivers exact text,
increments preview clipboard generation once and shows truthful feedback without
closing. Empty selection does not fabricate a copy. Later keyboard selection
uses the same path. This must not introduce OS clipboard writes.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
