# F11 — One retention-aware mutation boundary

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the shared retention mutation boundary, including all ingestion/limit-change paths. Prove Holla Activity and Plan exceed their limits without retaining the wrong tail. Coordinate F12/F13.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No deferred implementation slice.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · confirmed bounded-line contract defect.** `max_lines(3)` followed by
`set_lines(eight)` or applied after `with_lines(eight)` retains eight. Only push
enforces the limit (`viewport.rs:155/166/171/189`); empty replace-last also needs
zero-limit semantics. Holla Activity configures 4,000 and Plan 2,000 but populates
through `set_lines`. Supported API probes and present consumer traces agree.

Enforce the declared cap at construction, replacement, append, tail replacement
and limit changes, through one transaction returning the removed-prefix delta.
Pair with F12 so trimming cannot corrupt identity. Risk: medium visible-tail/API
change. Acceptance: limits 0/1/N, oversized batches, shrinking cap and Holla
integration beyond configured limits retain the correct tail. Define zero and
logical-line versus byte limits. No OOM or byte bound was demonstrated; this
plan uses P2 rather than the text report's P1, while keeping the violated cap
mandatory to fix. A line cap does not bound one arbitrarily long line.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
