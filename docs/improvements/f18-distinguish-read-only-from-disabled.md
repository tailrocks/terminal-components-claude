# F18 — Distinguish read-only from disabled

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. Existing showcase disabled/read-only fixture correction. Holla read-only output must remain navigable/selectable under F19.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · design inconsistency.** `pages/textareas.rs:37` labels `.disabled(true)`
as “Read-only transcript”; disabled TextArea refuses focus/keys, unlike
`DESIGN.md`'s readable/navigable read-only contract. Design and ecosystem audits
independently verify it.

Label the existing disabled fixture accurately. Demonstrate a read-only transcript
with existing TextViewport and a stated reason; use read-only CodeEditor for code.
Risk: low fixture correction; keyboard-copy completion depends on F19. Acceptance:
disabled examples remain dim/inert/unfocusable; read-only examples remain readable,
navigable/selectable, expose copy and reject mutation. Add read-only field APIs
only if a real field-shaped consumer cannot compose the existing primitives.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
