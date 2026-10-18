# F23d — input-flood fairness (V04)

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Measure the shared event pump using finite queues and bounded Holla PTY input floods. Define tick/quit fairness and change bounded draining only if measured behavior violates the contract.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

No unrelated benchmark project; a source-only risk is not a confirmed freeze.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Ignored/consumed events drain until empty before tick checks. Source risk only; no sustained starvation measured.

**Root fix / retained acceptance:** Finite injected queue and bounded owned-PTY flood measure dispatch/tick/quit latency. Establish fairness contract, then implement bounded draining if it fails. Do not label absent sustained-flood proof a confirmed freeze.

## Evidence

**Slice status:** current shared slice complete.

**Contract and measurement:** `src/runtime.rs: drain_ready_inputs` drains ready events until the queue is empty, the application asks to quit, or `DRAIN_BUDGET = 256` unchanged events have been dispatched; the tick check then runs. `app_tests_proofs.rs: a_finite_flood_of_ignored_input_never_starves_the_tick_check` injects a finite queue of an unbound chord and asserts the tick check is reached; `tests/holla_pty.rs: a_bounded_input_flood_is_drained_fairly_and_a_quit_behind_it_is_honoured` floods a fresh Holla process on an owned PTY with single-byte `Ctrl+B` and asserts the frame still advances and a quit queued behind the flood is honoured within the bound.

**Result:** the measured behaviour met the contract only after bounded draining was introduced; the original unbounded drain is the recorded source risk, not a confirmed sustained freeze.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
