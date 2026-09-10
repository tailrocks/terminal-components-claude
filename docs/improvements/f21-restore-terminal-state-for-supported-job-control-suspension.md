# F21 — Restore terminal state for supported job-control suspension

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase

Complete the shared runtime suspend/resume contract used by the Holla preview. Use isolated owned PTYs to prove restoration and re-entry; this is preview runtime correctness.

Apply every retained acceptance clause for this slice. Historical findings must
be checked against current code before editing. Preserve surrounding behavior.

- [ ] Implement the stated shared/Holla slice and necessary caller migrations.
- [ ] Retain relevant deterministic contract and Holla owner regressions.
- [ ] Inspect affected captures/terminal evidence and record scope and results.

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

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
