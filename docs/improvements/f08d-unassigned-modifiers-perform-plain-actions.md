# F08d — Unassigned modifiers perform plain actions

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Specify and test Holla chords, editor/modal precedence, advertised action reachability and shared Picker/navigation bindings used by Holla. Unassigned modified chords must not perform unrelated plain actions.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

The full TablePro/DataTable and other-application chord audit remains deferred. No universal command bus or broad binding framework.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F08 shared context](shared-contracts.md#f08-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** P2 architecture/API weakness. DataTable Ctrl+S sorts at `table.rs:438`; several navigation controls match code without modifier ownership. TablePro intercepts Save, so this is not a proven broken TablePro Save flow.

**Root fix / retained acceptance:** Define a chord/precedence matrix before changes; explicit bindings act, unassigned nonmodal chords return Ignored, top modal may consume without unrelated action. Medium compatibility risk. Preserve Shift ranges and Picker Ctrl+J/K/N/P; test Save/Run/Find/Close/detach through real owners. `Key::plain` intentionally permits Shift; adding it everywhere is not the complete policy.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
