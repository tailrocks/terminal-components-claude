# F05 — Preserve TreeView target across lazy insertion

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Implement the shared TreeView identity contract and integrate it into HP04/HP19 where trees are used. Prove delayed insertion, collapse, filtering, fallback and reveal using primitive tests and Holla fixtures.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No separate TablePro feature work. Minimal caller migrations and existing regression checks required by a shared API change remain part of this slice.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed defect.** `TreeView::set_children` calls `flatten`, which keeps
only a clamped numeric cursor (`src/widgets/tree.rs:145/203/227`). Inserting
children above a focused sibling moves the cursor to a new child although the
old node survives. TablePro's `Workbench::tick_explorer` is a present delayed-load
consumer. API and independent design probes agree.

Root: flattened display position is treated as node identity. Capture the
focused path before rebuild, resolve it afterward, and define fallback for a
removed/hidden target (visible ancestor, then surviving neighbor). Keep cursor
and selected path distinct; do not steal focus when delivery occurs elsewhere.
Risk: medium collapse/filter behavior risk. Acceptance: delayed nested loads,
focus on later siblings, collapse, filters, empty children and keyboard/mouse
toggle paths preserve or deliberately relocate the target and reveal it.
Positional paths do not solve arbitrary sibling reordering; that remains a
separate owner-identity contract, not justification for a global key framework.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
