# F08a — Between upper-value paste

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. TablePro Between-filter upper-field paste; outside the Holla prototype.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

Family policy: [F08 shared context](shared-contracts.md#f08-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** P2 confirmed defect. TablePro `App::handle` at `app.rs:240` checks only `FilterEditor.value`, unlike key/click paths iterating `value` and `value2`. Actual rendered filter probe consumes paste while upper value is editing and leaves it empty.

**Root fix / retained acceptance:** One active-input resolver over the same field set for keys/paste/clicks; reuse F01 modal routing. Low risk. Focus each bound by keyboard and mouse, paste distinct values, apply and assert both; hidden upper field and action focus receive no edit.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
