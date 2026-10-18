# Historical Grid contract review: TablePro

## Status

Historical candidate contract. The review was accepted for its former lane; it does not block current work or establish current API authority. Re-evaluate each item against current source and `design/interaction-polish` before changing code.

## Authority

Current architecture and product authorities are [architecture](../architecture.md), [design](../design.md), and [status](../status.md). The original review is available at immutable commit [`d1122f69`](https://github.com/tailrocks/terminal-components-claude/blob/d1122f69/docs/reviews/laneB-grid-contract.md).

## Current evidence

Current Grid implementation resides in `crates/termrock-grid`; TablePro app behavior is distributed across current `crates/tablepro*` packages. `GridAction::Sort(ColumnKey, SortDir)`, `GridState::{cursor, selected_rows, is_editing, edit_error, col_offset}`, and `Ui::register_focus_only` are present in source. Additional `GridState` readers also exist, so the historical claim that these are the exact reader set is false for current source. Implementation presence does not establish product parity or complete verification.

## Candidate contract to verify

- Sorting remains adapter-owned. Grid emits a keyed sort request and never compares rendered cell text. TablePro adapters preserve SQL NULL ordering and numeric comparison; result adapters preserve their own display-to-storage ordering.
- Cursor, selection, and pending edit remain associated with stable `ItemKey` and `ColumnKey` across reorder.
- `edit_error` preserves typed `FieldError`.
- Focus-only registration creates a keyboard focus entry without drawable area or hit target. Empty-area `register_control` remains rejected.
- TablePro behavior, tests, and visual output match the accepted product reference.

The original review proposed a minimal exact GridState reader set and rejected geometry/cache readers. Current public API has since evolved; do not remove or hide existing public APIs based solely on this historical proposal.

## Remaining work

Inspect current exports and consumers, map each candidate requirement to current evidence, and record any real gap in the current canonical plan. No historical lane blocker remains active by virtue of this document.

## Verification

Use current source, focused tests, and visual comparison at the accepted design revision. The former test totals and green test run are not evidence for current source.
