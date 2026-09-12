# F11 — One retention-aware mutation boundary

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the shared retention mutation boundary, including all ingestion/limit-change paths. Prove Holla Activity and Plan exceed their limits without retaining the wrong tail. Coordinate F12/F13.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

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

**Slice status:** current shared slice complete.

**Shared boundary:** every ingestion path of `TextViewport` (construction, `set_lines`, `push`, `replace_last`, a changed `max_lines`) enforces the declared cap through one transaction that returns the removed-prefix delta; zero and one-line caps are defined. Tests: `viewport.rs: retention_is_enforced_on_every_ingestion_path`, `wraps_long_lines_and_bounds_retention`, `eviction_never_retargets_a_selection`, `producer_caret_follows_eviction_and_replacement`.

**Holla integration:** activities keep `RETAIN_LINES = 4000` with a `dropped` counter and the viewer says `500 earlier lines dropped · last 4000 kept`; plans keep 2000. Tests: `domain/activity.rs: retention_is_bounded_and_counted`; `app_tests_parity.rs: hp14_output_streams_are_exact_and_retention_drops_are_stated`; `app_tests_proofs.rs: a_burst_of_output_is_appended_not_rebuilt_and_equals_a_fresh_layout` (a 5000-line burst retains the right tail and the incremental layout equals a fresh one).

**Captures inspected:** `shots/h_hp14_burst`, `shots/h_hp14_find`.

**Limit:** a line cap does not bound one arbitrarily long line (by contract).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
