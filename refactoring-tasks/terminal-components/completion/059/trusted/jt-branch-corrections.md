# TablePro branch-review interaction obligations — TASK-059

## Authority and evidence boundary

This supplement binds BD-JT-03/07/08 explorer, tab lifecycle and Picker paste routing owned primarily by TASK-059. ADJ-09 TablePro Picker-under-modal paste applies. Oracle `02f5294bfdbf38004cc49130d0aff1d01f31434c`. Planning branches only.

## JT-PICKER-PASTE: explorer filter recipient — BD-JT-07

`EXPLORER-FILTER-EDITING` opens Workbench Explorer with inline filter field editing; TabList or SafeMode Picker open above.

1. **explorer-filter-receives-picker-paste:** EXPLORER-FILTER-EDITING; PASTE(" probe "). Require explorer filter draft changes; Picker query unchanged; no Query-only helper substitution.

Sources: oracle `workbench.rs:1194–1202`, `app.rs:236–254,638–644`.

## JT-SURFACES

Surface enum seeds alone are insufficient (BD-JT-03); tab ownership and explorer lifecycle branches bind through TP-* and TASK-064 closure.

## Verification binding

R-001/CHK-004 and R-002/CHK-006 bind named branches; TASK-064 closes TablePro union.
