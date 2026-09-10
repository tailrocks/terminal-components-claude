# F03 — Atomic ListBox mutation

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Deferred. ListBox/Settings mutation defect. Holla currently uses Picker and custom resource rows; the selected Files/Disk work uses TreeView. Do not add ListBox merely to activate this task.

## Later

- [ ] Implement the retained contract when selected for a later goal.
- [ ] Retain its acceptance evidence before marking it complete.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed defect.** `ListBox::move_to`, `src/widgets/list.rs:90`, uses a
private range anchor. Settings removal at `src/bin/showcase/pages/settings.rs:579`
replaces public items/checks/cursor but cannot repair that anchor. Reproduction:
Environment list, select zero-based rows 3–4, Remove selected, return to list,
Shift+Up. Current app panics at an out-of-bounds checked index. API App probe and
independent real-terminal replay agree.

Root: no owner can atomically maintain all collection invariants. Introduce
replacement/removal operations owning items, checks, cursor, chosen item,
selection anchor and scroll. Migrate Settings add/remove callers. Choose reset
versus semantic preservation explicitly; bounds clamping alone can still select
the wrong surviving item.

Risk: medium compatibility risk. Recommended end state: invariant-bearing
storage cannot be independently mutated. Stage accessors/checked operations and
migrate consumers before a deliberate visibility change. If public unchecked
writes remain for compatibility, record that residual API weakness; do not call
the bug class eliminated. Acceptance: actual flow regression plus empty/growing/
shrinking collections, removal around both range endpoints, disabled rows,
mouse removal, chosen/check-count consistency and scroll clamping.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
