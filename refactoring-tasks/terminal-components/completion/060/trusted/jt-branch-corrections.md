# TablePro branch-review interaction obligations — TASK-060

## Authority and evidence boundary

This supplement binds BD-JT-06/07/09 filter, Table Grid and architecture-adversary separation owned primarily by TASK-060. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-FILTER-POLICY: source predicate semantics — BD-JT-06

`FILTER-NOTCONTAINS` enables NotContains on a seeded column.

1. **filter-notcontains-no-predicate:** apply filter. Require result set unchanged versus source; main ILIKE evaluator exclusion is wrong future path.
2. **filter-match-all-label-only:** toggle ALL/ANY lead. Require conjunction unchanged459–474; label reload alone must not switch OR semantics.

Sources: oracle `tabs.rs:183–259,459–474`; main `filter_editor.rs:188–276`.

## JT-PICKER-PASTE: Table Grid inline cell — BD-JT-07

`TABLE-CELL-EDITING` with TabList Picker open above inline editable cell.

1. **table-cell-receives-picker-paste:** paste while Picker visible. Require inline cell draft changes through app-owned route, not Query-only helper.

Sources: oracle `workbench.rs:1194–1202`, `app.rs:236–254`.

## JT-ARCHITECTURE-ADVERSARIES: retain without UX expansion — BD-JT-09 (ADJ-18)

Preserve row_identity, undo_contract and editable Table-path runtime witnesses as generic adversaries. Do not treat editable SELECT grid contract tests as authorization for live Query editing.

Sources: `row_actions_contract.rs`, oracle `tabs.rs:2030–2070`.

## Verification binding

R-001/CHK-004, TP-039–042 and R-002/CHK-006 bind named branches; TASK-064 closes TablePro union.
