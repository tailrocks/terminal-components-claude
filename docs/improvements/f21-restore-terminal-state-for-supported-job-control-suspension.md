# F21 — Restore terminal state for supported job-control suspension

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the shared runtime suspend/resume contract used by the Holla preview. Use isolated owned PTYs to prove restoration and re-entry; this is preview runtime correctness.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [x] Implement the stated shared/Holla slice and necessary caller migrations.
- [x] Retain relevant deterministic contract and Holla owner regressions.
- [x] Inspect affected captures/terminal evidence and record scope and results.

## Later

Real task PTY execution, process-group cancellation and production OS integrations under HP15 remain deferred. Record unavailable platform evidence as pending.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**P1 · confirmed lifecycle defect in an isolated external-SIGTSTP flow.**
Current runtime has no suspend/continue ownership. Terminal auditor created an
owned controlling PTY with a non-orphan foreground app process group, ran
Showcase, sent SIGTSTP and returned foreground to its shell process. The stopped
application left ICANON/ECHO disabled and termios different from the saved state.
SIGCONT followed by normal quit restored the original termios. This proves the
specific suspension gap, not failure of normal teardown or every signal route.

Root: restoration belongs to startup/exit only, not job-control lifecycle.
Define suspend/re-enter as explicit TerminalSession transitions: restore owned
modes before supported suspension, re-acquire on continuation, rebuild geometry
and fully redraw. Integrate signals through a safe event/notification boundary,
not arbitrary allocation/rendering in a signal handler. Keep repeated transitions
idempotent and panic-hook ownership explicit. Raw-mode Ctrl+Z is a key byte, not
equivalent proof of external SIGTSTP handling.

Risk: medium/high platform/terminal lifecycle. The successful temporary probe
has known startup/deadline/reap cleanup weaknesses; do not retain it unchanged.
Track only live owned children, guard startup, bound all waits and reap on failure.
Acceptance: retained owned-PTY
normal, partial-startup, post-init draw/read error, panic, repeated enter/leave
and suspend/continue cases; shell canonical/echo state while stopped, resize
while suspended, resumed focus/full redraw and final exact termios. Explicitly
document unsupported platforms and unrecoverable cases: SIGKILL cannot unwind,
and a lost output device cannot receive restoration escapes. Do not promise
“every exit path”. Never run these probes against the user's live terminal.

## Evidence

**Slice status:** current shared slice complete.

**Runtime:** `src/runtime.rs` owns suspend and re-entry as explicit `TerminalSession` transitions: a SIGTSTP handler only sets an atomic flag (`job_control`), the event loop restores the owned modes (`suspend()`), stops itself (`stop_self()`), and on SIGCONT re-acquires the terminal, clears and swaps both buffers and delivers the new geometry as `Input::Resize` (`reenter()`); repeated transitions are idempotent; `poll_uninterrupted` treats EINTR as no event. Tests: `runtime.rs: suspension_leaves_and_reacquires_idempotently`, `interrupted_poll_is_not_an_error`, `failed_terminal_setup_restores_state_and_preserves_error`, `panicking_terminal_setup_restores_state`, `successful_setup_transfers_restoration_and_leaves_once`.

**Owned-PTY proof:** `tests/terminal_suspend.rs: external_sigtstp_restores_the_shell_and_fg_reenters_at_the_new_geometry` spawns a helper job-control parent on an owned PTY, runs the Holla preview as a foreground process group, sends SIGTSTP, asserts the shell-side termios has ICANON and ECHO back, bracketed paste and the alternate screen released, resizes while stopped, continues with `fg`, asserts re-entry at the new geometry with bracketed paste re-enabled, quits, and asserts the final termios equals the launch state and the child is reaped. Every wait is bounded; the helper is reaped by `waitpid`.

**Limits (documented, not passed):** SIGKILL cannot unwind; a lost output device cannot receive restoration escapes; the proof ran on macOS only (Linux is pending external evidence). The probe never touches the user's live terminal.

Provenance: every cited capture carries `<name>.manifest.json` (source git revision `1aaa9b0` with the dirty working tree of this change set, binary sha256 of `target/debug/holla`, arguments, geometry, colour environment, tmux 3.7c, Python 3.14.7, Pillow 12.3.0, the JetBrainsMono NFM font files) and `<name>.png.fidelity.json`; the `.txt` capture is authoritative for content. Platform scope: macOS (Darwin 25.6.0) host for every PTY and capture run; Linux behaviour is fixture-modeled only.
