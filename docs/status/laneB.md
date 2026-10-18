# Lane B: TablePro and Grid status

## Status

Historical planning lane. Application migration exists in the split workspace; the crate migration is not ready for release. This page records candidate acceptance requirements, not an active lane assignment.

## Authority

Current direction: [architecture](../architecture.md), [design](../design.md), [status](../status.md), and [plan](../plan.md). Product behavior follows `design/interaction-polish`; reusable APIs and package boundaries follow the split architecture represented by `main`.

## Current evidence

TablePro code currently lives across `crates/tablepro`, `crates/tablepro-ui`, `crates/tablepro-domain`, `crates/tablepro-sql`, and `crates/tablepro-demo`. Grid code lives in `crates/termrock-grid`; focus registration lives in `crates/termrock-runtime`. The old `apps/tablepro` and Slice 5/6 sequencing claims are historical and do not describe current paths.

The old Grid review proposed adapter-owned sorting, keyed `GridAction::Sort(ColumnKey, SortDir)`, typed edit errors, limited semantic state readers, and a separate focus-only registration API. Some APIs now exist, but this does not prove all proposed API limits, consumer needs, behavior, or visual parity. See [historical review](../reviews/laneB-grid-contract.md).

## Remaining work

- Compare current TablePro interaction, output, and colors against `design/interaction-polish`.
- Verify every accepted behavior against current app and library source; decide each old Grid proposal against actual current public APIs.
- Verify migrated tests and visual evidence against the current source revision before claiming completion.

## Verification

Inspect current manifests, crate exports, focused tests, and design-branch behavior. Historical test counts and green runs are not current proof.
