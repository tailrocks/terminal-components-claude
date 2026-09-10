# F15 — Incremental live-output work, measured separately from idle redraw

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the retained TextViewport incremental-work contract and Holla producer changes after F10–F14. Measure idle, append, tail, batch, wrap, resize, selection and search workloads.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

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

**Slice status:** current shared slice complete.

**Shared boundary:** logical-line parsing is separated from width-dependent visual rows; the visual index is extended on append and tail replacement (`appended()`, `shift_index()` on eviction, `ensure_layout` extends), rebuilt only on dataset replacement, width or wrap change; `WorkCounters { segmented_lines, reflowed_lines, index_rebuilds, index_extends, index_shifts }` expose the work. Tests: `viewport.rs: tail_update_reparses_only_the_changed_line`, `unchanged_overflow_redraw_reuses_layout`, `cached_layout_reflows_after_resize_wrap_and_content_changes`.

**Holla workloads:** `app_tests_proofs.rs: a_burst_of_output_is_appended_not_rebuilt_and_equals_a_fresh_layout` (5000 appended lines: at least 2500 index extensions, zero rebuilds while following, incremental cells equal a fresh layout), `idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree` (idle ticks: zero segmentation and zero tree rebuilds); `output_scrollbar_press_and_drag_redraw_and_find_index_resets_on_a_new_query` (search and selection workloads).

**Measurement:** counters, not wall-clock thresholds, are the retained proof; no universal latency budget is claimed (the contract forbids an arbitrary threshold from one sample).

**Captures inspected:** `shots/h_hp14_burst`.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
