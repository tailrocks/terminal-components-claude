# F23b — lifecycle (V01/V02)

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Retain the F21 owned-PTY lifecycle matrix with exact mode/termios assertions, bounded cleanup and scoped platform claims.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Task-process lifecycle integration under HP15 is deferred; it is distinct from restoring the preview terminal.

Family policy: [F23 shared context](shared-contracts.md#f23-shared-context).

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**Evidence and classification:** Guard/panic/idempotence units and historical PTY checks; fresh F21 suspension failure.

**Root fix / retained acceptance:** F21 owned-PTY matrix, exact mode/termios assertions and scoped best-effort documentation. Counter callbacks are not actual terminal proof.

## Evidence

**Slice status:** current shared slice complete.

**Matrix:** `tests/terminal_suspend.rs` (owned PTY, helper job-control parent) covers normal exit, external SIGTSTP with exact termios assertions while stopped (ICANON, ECHO, bracketed paste released, alternate screen left), resize while suspended, `fg` re-entry at the new geometry, final termios equality and bounded reaping; `runtime.rs` units cover partial startup, a failing setup, a panicking setup and idempotent leave/re-enter. Counter callbacks are not used as terminal proof; the PTY bytes and termios are.

**Platform scope:** macOS host only; Linux job control remains pending external evidence (stated in F21).

**Deferred remainder:** task-process lifecycle under HP15 (real PTYs) is deferred by contract.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
