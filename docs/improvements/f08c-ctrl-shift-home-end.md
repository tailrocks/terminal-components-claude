# F08c — Ctrl+Shift+Home/End

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. Shared TextArea/CodeEditor/cell-editor document-selection shortcut. No selected Holla field requires these editing owners; retain for later unless a necessary shared migration exposes this path.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

Family policy: [F08 shared context](shared-contracts.md#f08-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** P2 confirmed defect. `field_common.rs:78/79` calls document movement with `false` rather than Shift; TextArea probe moves to byte 0 with no selection. Shared by code/text/cell editing.

**Root fix / retained acceptance:** Propagate Shift at shared action boundary. Low risk. New/existing/reversed selections extend through Unicode document endpoints; unshifted navigation, empty text, line Home/End and all supported owners remain correct.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
