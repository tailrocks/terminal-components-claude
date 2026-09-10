# F23e — dirty/idle performance (V05)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Retain F15 workloads and large Holla finder/tree/output/plan producer fixtures with work counts, allocations, median/tail timing and fresh-layout equality.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Standalone list/table/grid performance coverage unrelated to the selected Holla compositions remains deferred.

- [ ] Complete the remaining owner/coverage clauses of the retained contract.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Fixed idle cache; F15 replacement scaling measured, no whole-app performance bound.

**Root fix / retained acceptance:** Retained F15 workloads plus large list/tree/table/grid rendering and producer synchronization; work counts, allocations, median/tail timings, fresh-layout equivalence.

## Evidence

**Slice status:** current Holla/shared slice complete · standalone list/table/grid performance stays Later.

**Workloads:** `app_tests_proofs.rs: a_burst_of_output_is_appended_not_rebuilt_and_equals_a_fresh_layout` (5000-line producer burst on the activity viewport: `WorkCounters` show appends, not rebuilds, and the incremental layout equals a fresh layout cell for cell), `idle_ticks_rebuild_nothing_on_the_finder_and_the_disk_tree` (finder `rebuilds` and disk `rebuilds` counters stay flat across idle ticks). The retained gate is work counts and fresh-layout equality; timings are not asserted as thresholds.

**Deferred remainder:** list/table/grid coverage outside the Holla compositions (Later checkbox); allocation and median/tail timing tables are not retained as tests (they would be host samples, which the contract says are not a budget).

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
