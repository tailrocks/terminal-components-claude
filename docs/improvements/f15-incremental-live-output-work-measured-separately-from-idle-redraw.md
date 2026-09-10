# F15 — Incremental live-output work, measured separately from idle redraw

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the retained TextViewport incremental-work contract and Holla producer changes after F10–F14. Measure idle, append, tail, batch, wrap, resize, selection and search workloads.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

## Later

No unrelated application performance project.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P2 · architecture/API weakness with measured performance impact.** `replace_last` dirties the
whole document; `ensure_layout` recreates all cells, potentially twice for final
scrollbar width (`viewport.rs:196/245/305`). A release probe's 20 updates of
80-character rows at 80×20 averaged approximately 4.47/34.41/172.49 ms per update
at 1k/10k/50k lines. Only the first replacement changes the contents; the remaining
calls supply the same line again, yet all dirty the cache. An independent repeat
observed the same scaling. These are single shared-host samples, not frame latency
or a promised budget. Holla also reconstructs complete filtered vectors on output
changes. The prior idle-cache fix remains valid and independently retested.

After F10–F13, separate logical-line parsing from width-dependent visual rows;
invalidate changed lines only, apply append/tail/batch deltas at producers, and
avoid reparsing source merely to resolve scrollbar width. Risk: medium/high stale
cache risk. Acceptance: fresh-layout equality and instrumented zero unchanged-line
reparses for a tail update; bounded retained state and correct anchors under churn.
Retain workloads for idle, append-at-cap, tail replacement, batch, resize, wrap,
selection and search at agreed sizes. Record allocations/reflows plus median/tail
timings, establish a representative latency target, then prove it; no arbitrary
threshold inferred from one sample. Preserve existing idle zero-reflow gate.

Full text evidence, temporary probe results and exact limits:
[text verification](../plan-text-verification.md). Temporary probes are current
evidence, not repository regression tests; implementation must retain them.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
