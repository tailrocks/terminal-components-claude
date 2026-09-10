# F06 — Preserve DataGrid cursor identity through local sort

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. DataGrid sorting and pending-value policy; outside current Holla compositions.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · design inconsistency with confirmed target change.**
`DataGrid::request_sort`, `src/widgets/grid.rs:1176`, changes `order` without
remapping cursor/range anchor. Supported `s` on rows beta/alpha changes the
focused value beta → alpha without navigation. TablePro query results enable
local sorting; DataTable already preserves source identity. API and verification
agents independently reproduced the mismatch, including pending-value sorting.

Preserve the cursor's source row through ascending/descending/unsorted order.
Explicitly clear or reconcile rectangular display selections; retain source-keyed
checks and pending edits. Risk: medium selection semantics. Acceptance: duplicates,
all sort directions, keyboard/mouse selection, invalid drafts and pending changes;
the same record remains focused and no range silently acquires unrelated rows.
Do not promise globally sorted unloaded data: local-sort and fetch-more contracts
must remain distinct.

A related API probe changes beta to pending aardvark, then sorts ascending;
display reads alpha, aardvark because sorting compares stored rows rather than
effective displayed values. Resolve this policy explicitly: sort effective
values, or label a deliberately committed-data order. Acceptance must include
pending edits and null/type ordering; do not accidentally clear pending changes.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
