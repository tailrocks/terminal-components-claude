# HP15 — Runtime input, cancellation and terminal ownership

[Tracker](../../IMPROVEMENTS_PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Implement simulated prompt attention, masked input, exact input bytes, single-owner routing, cancelling/acknowledged completion and output/report retention on a virtual clock. Model resistant/finished tasks and stale ownership without launching children.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [ ] Implement all mapped UI variants and typed simulated outcomes.
- [ ] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [ ] Inspect keyboard/pointer, size/color and decisive state captures.
- [ ] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real task PTYs/stdin, process groups, TERM/KILL escalation, subreaper/reaping and OS-specific process integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve prompt input, password attention, cancellation/reaping and terminal restoration. Attached monitor fixtures currently consume ordinary keys without a response model.

**Source evidence and mandatory scope:** [matrix HP15](../holla-parity-matrix.md#hp15--runtime-input-cancellation-and-terminal-ownership) — `E29`, `E33`, `E34`, `E35`, `E38`, `E39`, `E40`, `E43`, `E37`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Add explicit task input mode separate from launch arguments and screen navigation. Show background password/confirmation attention, select the owning task without silently sending keys, and retain task/cwd/host identity. Forward Unicode, CR/DEL/Tab and supported control bytes; Escape returns to controls. Protect secret input from echo/history/inspection/copy. Stop request enters Cancelling; completion waits for owned process cleanup.

**Architecture / reusable components:** One session supervisor owns task PTY, stdin arbitration, process group and output drain across UI lifetime. Future Unix adapter uses controlling terminal and group cancellation: TERM, 750 ms escalation, KILL/reap while ownership remains valid; Linux subreaper versus macOS group extinction are explicit. Scope excludes deliberately escaped sessions. Reuse input/edit affordances, masked Input, Activity and Cancel-default dialog; no claim of full terminal emulation.

**Required deterministic fixture:** `parity-task-input` — partial no-newline Password:, repeated/background prompt, q/h/j/i payload, Ctrl-C/D/Tab/Unicode, task finishes during input, stdin EOF, two prompting tasks, resistant child, spawn/cancel race, terminal/render failure.

**Acceptance / automated verification:** Assert exact input bytes and single target, no navigation leakage, no input after completion, redaction and no stale PID signaling. Assert queued tasks never spawn after cancel, no owned descendants remain before Cancelled, no output/report loss and restored terminal after failure. Run isolated PTY tests on macOS and Linux at operational gate. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture attention/input badge, masked prompt, keep-running/stop dialog, Cancelling and acknowledged completion; retain PTY restoration transcripts.

**Dependencies:** HP14/HP17/HP23; F01/F08/F10–F15/F21/F23.

## Evidence

Pending implementation. Record current source findings, tests/scenarios,
inspected capture paths, provenance/platform scope and any deferred remainder.
