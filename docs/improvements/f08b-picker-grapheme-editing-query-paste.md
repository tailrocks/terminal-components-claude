# F08b — Picker grapheme editing/query paste

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Implement grapheme-safe query edits and one QueryChanged event per paste in the shared Picker; route through Holla modal ownership. Prove typed/pasted query equivalence and the full widget state matrix.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

TablePro picker-owner paste integration and F08a remain deferred; do not claim those application routes fixed.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F08 shared context](shared-contracts.md#f08-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** P2 confirmed deletion defect + coverage gap. `picker.rs:199` uses `String::pop`: Backspace on 👩‍💻 leaves woman+ZWJ. No query-paste entry point; owners consume paste or leak it through F01.

**Root fix / retained acceptance:** Reuse TextBuffer/grapheme operations and add query paste emitting existing `QueryChanged` exactly once. Medium owner/API risk. Typed/pasted Unicode queries produce identical rows; complete-cluster Backspace, clear, disabled/search-disabled/loading/error and escape behaviors are tested in widget and real owner. Coordinate F02, but keep destructive eligibility separate.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
