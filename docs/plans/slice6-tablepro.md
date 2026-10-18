# TablePro acceptance plan

## Status

The historical package move is present in the workspace. TablePro code is split across `crates/tablepro`, `crates/tablepro-ui`, `crates/tablepro-domain`, `crates/tablepro-sql`, and `crates/tablepro-demo`. The broader refactor remains incomplete and not ready. This plan tracks behavior/API acceptance in current crates; it does not schedule another app-package move.

## Authority

Current direction: [architecture](../architecture.md), [design](../design.md), [status](../status.md), and [plan](../plan.md). `design/interaction-polish` controls UI, colors, and behavior. `main` controls reusable crate boundaries and architecture.

## Current evidence

Current crate manifests establish the split. They do not prove TablePro behavior, API completeness, test coverage, performance, or visual parity. The old `apps/tablepro` and `Slice 5/6` path/work-package instructions are retired.

The detailed historical acceptance source is preserved at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/plans/slice6-tablepro.md). It retains the original surface inventory, capability checklist, regression scenarios, performance thresholds, and open questions. Those details are candidates for current verification, not current source locations or completion claims.

## Remaining work

Verify these behavior and API contracts against current `crates/tablepro*` and the design reference:

- TablePro owns SQL/domain ordering, including NULLs-last and numeric comparison. Generic Grid emits keyed sort requests and never compares rendered text. Reordering preserves cursor, selection, and edit identity.
- Grid edit requests preserve typed errors, stable item/column keys, and each app-specific edit policy. Result grids remain read-only where required.
- Connection, query, filter, history, structure, plan, result, picker, and dialog journeys retain their keyboard and pointer behavior.
- Secret input remains masked and absent from rendered output, logs, and copied data. Stored-value masking follows product behavior.
- Timing, status expiry, focus, overlays, and capture behavior remain deterministic.
- Historical unit, integration, visual, and performance obligations are mapped to current tests and captures. Every visible difference is compared with `design/interaction-polish` and classified before any baseline change.
- Reusable APIs stay in shared crates; SQL vocabulary, database policy, and workflow composition stay in app crates.

Record unresolved findings in the canonical plan with current source references. Do not treat historical Slice 5/6 gates, counts, or sequencing as blockers.

## Verification

Use current source, public consumer builds, focused tests, performance measurements, and visual comparisons against the accepted design branch. A package manifest or successful build alone does not establish completion.
