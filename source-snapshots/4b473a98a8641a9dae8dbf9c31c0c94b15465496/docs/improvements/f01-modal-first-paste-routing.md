# F01 — Modal-first paste routing

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. TablePro modal routing defect; no Holla owner is implemented by this task. Holla modal correctness is current under H00 and the HP interaction gates.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed defect.** `src/bin/tablepro/app.rs:236`, `App::handle`,
routes paste to Dialog and Filter only; other modals fall through to the screen.
Actual App/TestBackend reproduction: connect Production, focus Query, enter
editing, Ctrl+O, paste SQL. Picker stays empty while the hidden query changes.
Interaction and editor agents independently reproduced the same owner violation.

Root: modal hit/focus barriers do not own all input classes, and render-time
focus repair is too late to prevent a hidden editor receiving an event.
Make the active modal the exhaustive first recipient of paste. A modal either
dispatches to its active child or consumes unsupported paste; never fall through.
Reconcile edit/focus transitions at opening/closing ownership boundaries, not
only when an old control happens to render. Reuse current modal variants and
typed outcomes. Do not replace the complete event system.

Risk: medium routing/focus risk. Acceptance: open every TablePro modal over an
editing query/grid, send paste before and after its first render, and assert
only the active eligible child changes. Test queued and delayed input, close/
cancel focus restoration, hidden/removed opener, keyboard/mouse opening, and
ordinary document paste without a modal. Preserve drafts unless the existing
explicit commit/cancel contract says otherwise.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
