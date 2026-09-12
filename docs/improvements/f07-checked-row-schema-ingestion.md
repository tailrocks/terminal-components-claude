# F07 — Checked row/schema ingestion

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. DataTable/DataGrid ingestion contract; no current Holla producer uses these widgets. Retain the defect without treating it as fixed.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · architecture/API weakness; malformed-input panic reproduced, no current
app producer proven.** `DataTable::new/set_rows` accept ragged vectors while
sorting indexes `rows[a][col]` (`src/widgets/table.rs:136/209/225`). Two columns
with one-cell rows panic when sorting column 1. DataGrid missing-cell reads use
Null, but local sorting indexes directly (`src/widgets/grid.rs:428/502`).

Root: the accepted data shape is neither enforced nor consistently interpreted.
Define rectangularity and index validity at ingestion; prefer checked rejection
that leaves the previous dataset/edit transaction untouched. Do not silently
equate absent cells with SQL Null. Coordinate schema replacement with active
edits and pending/source identities. Risk: medium public API compatibility.
Acceptance: zero columns, empty/short/extra rows, schema changes during editing,
and invalid source references produce documented errors or deliberate normalization;
well-formed behavior is unchanged. Additive checked APIs are a migration step,
not a complete root fix while unchecked storage remains writable.

Evidence and detailed probes for F02/F03/F05–F07:
[API verification](../plan-verification/plan-api-verification.md), with independent user-flow
checks in [design verification](../plan-verification/plan-design-verification.md).

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
