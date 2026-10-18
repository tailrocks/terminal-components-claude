# Historical audit evidence index

## Status

This directory contains retained summaries and machine-readable audit outputs from a past Holla integration investigation. It is not an active project plan.

## Authority

Current authority is [architecture](../../../architecture.md), [design](../../../design.md), [status](../../../status.md), and [plan](../../../plan.md). Historical claims below do not establish current implementation state or product parity.

## Current evidence

The previously listed per-commit patch files are not present in this checkout and have been removed from the index. The following retained artifacts are present and may be read for historical investigation:

- [Audit summary](AUDIT.md)
- [Historical contract summary](CURRENT-CONTRACT.md)
- [Current-source gap analysis from that investigation](current-source-gaps.md)
- [Early report](early/REPORT.md)
- [Middle reports](middle/early-large-report.md) and [report 11-21](middle/report-11-21.md)
- [Late audit](late/AUDIT.md) and [coverage integrity review](late/coverage-integrity.md)
- [API report](api/REPORT.md)
- [Integrator semantic review](integrator-semantic-review.md)
- [Historical named-obligation inventory](historical-named-obligation-inventory.json)
- [Semantic coverage output](semantic-coverage-final.json)

These files describe the investigation and its then-current source snapshot. They are not substitutes for the design branch’s product requirements or fresh source review.

## Remaining work

No patch bundle is represented by this index. Recover the exact source revisions from Git history when a historical claim needs patch-level verification; do not infer missing patch contents from the old commit table.

## Verification

This index was checked against files retained under this directory. Recheck each referenced file before relying on this archive after further repository changes.
