# F04 — Reconcile DataGrid read-only transitions before mutation

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. DataGrid permission-transition defect; no selected Holla design flow requires editable DataGrid.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed API defect; current app transition not established.**
`DataGrid::begin_edit` checks `editable`, but `commit_edit` and an existing edit's
paste path do not recheck it (`src/widgets/grid.rs:539/590`). Public sequence:
begin edit → `editable = false` → paste → commit still emits `CellChanged` and
writes pending data. Text and interaction agents independently reproduced it.
CodeEditor's prior fix does not protect this separate implementation.

Root: permission is checked only at transaction entry. Centralize current
mutation eligibility and reconcile active edit state whenever grid/column
read-only state changes and at every mutation boundary. Define whether the
draft is cancelled or retained read-only; never silently commit forbidden data.
Reuse current edit/pending structures and explicit setter/accessor migration.

Risk: medium transaction/API risk. Acceptance: flip grid and column permissions
mid-edit, then exercise paste, characters, Tab/BackTab, Enter, blur, click, direct
commit and render. Data/pending state must remain unchanged and no write event
may escape. Existing read-only navigation, selection/copy and valid edits remain.
This is UI mutation correctness, not proof of database authorization security.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
