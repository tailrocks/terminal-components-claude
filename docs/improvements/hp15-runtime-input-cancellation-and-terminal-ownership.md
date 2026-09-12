# HP15 — Runtime input, cancellation and terminal ownership

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Implement simulated prompt attention, masked input, exact input bytes, single-owner routing, cancelling/acknowledged completion and output/report retention on a virtual clock. Model resistant/finished tasks and stale ownership without launching children.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Real task PTYs/stdin, process groups, TERM/KILL escalation, subreaper/reaping and OS-specific process integration.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve prompt input, password attention, cancellation/reaping and terminal restoration. Attached monitor fixtures currently consume ordinary keys without a response model.

**Source evidence and mandatory scope:** [matrix HP15](../parity/holla-parity-matrix.md#hp15--runtime-input-cancellation-and-terminal-ownership) — `E29`, `E33`, `E34`, `E35`, `E38`, `E39`, `E40`, `E43`, `E37`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Add explicit task input mode separate from launch arguments and screen navigation. Show background password/confirmation attention, select the owning task without silently sending keys, and retain task/cwd/host identity. Forward Unicode, CR/DEL/Tab and supported control bytes; Escape returns to controls. Protect secret input from echo/history/inspection/copy. Stop request enters Cancelling; completion waits for owned process cleanup.

**Architecture / reusable components:** One session supervisor owns task PTY, stdin arbitration, process group and output drain across UI lifetime. Future Unix adapter uses controlling terminal and group cancellation: TERM, 750 ms escalation, KILL/reap while ownership remains valid; Linux subreaper versus macOS group extinction are explicit. Scope excludes deliberately escaped sessions. Reuse input/edit affordances, masked Input, Activity and Cancel-default dialog; no claim of full terminal emulation.

**Required deterministic fixture:** `parity-task-input` — partial no-newline Password:, repeated/background prompt, q/h/j/i payload, Ctrl-C/D/Tab/Unicode, task finishes during input, stdin EOF, two prompting tasks, resistant child, spawn/cancel race, terminal/render failure.

**Acceptance / automated verification:** Assert exact input bytes and single target, no navigation leakage, no input after completion, redaction and no stale PID signaling. Assert queued tasks never spawn after cancel, no owned descendants remain before Cancelled, no output/report loss and restored terminal after failure. Run isolated PTY tests on macOS and Linux at operational gate. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture attention/input badge, masked prompt, keep-running/stop dialog, Cancelling and acknowledged completion; retain PTY restoration transcripts.

**Dependencies:** HP14/HP17/HP23; F01/F08/F10–F15/F21/F23.

## Evidence

**Slice status:** current simulated slice complete for the journeyed variants; gap: two prompting tasks, the spawn/cancel race and terminal/render failure named in the required fixture are not journeyed (the race is covered only by a unit test), and the keep-running/stop dialog is asserted only through tab close and quit dialogs · Later operational clauses open.

**Scenario and journey:** `parity-task-input` (fixture fn `task_input` in src/bin/holla/domain/parity.rs) · `hp15_prompts_take_exact_input_and_cancellation_escalates_truthfully` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/activity.rs: prompts_pause_the_script_and_take_exact_input`, `ctrl_c_stops_and_input_is_refused_outside_ownership`, `key_mapping_matches_the_terminal_contract`, `stop_enters_cancelling_and_settles_through_the_supervisor`, `queued_work_never_spawns_after_a_cancel`, `spawn_failure_is_output_and_a_failed_done`; `src/bin/holla/app_tests_flows.rs: activities_survive_navigation_and_merged_logs_keep_identity`, `quitting_a_remote_host_with_work_running_names_the_box`. · row proofs in src/bin/holla/app_tests_rows.rs: `e33_e39_input_mode_returns_on_esc_ends_with_the_program_and_a_kill_reports_reaping`.

**What the journey asserts:**
- "Deploy the release" argv is `./deploy.sh @/srv/app`; after 4 ticks `waiting` is a secret prompt, the text contains `waiting for input` and `input wanted`.
- `i` enters input mode (text contains `typing goes to stdin`); typing `hunter2` never appears in the rendered text; after Enter every `stdin` record has `secret == true` and empty `bytes`; `waiting` is cleared; output later contains `authenticated`.
- The second prompt is non-secret; `n` then Enter settles `Failed`, a line ends with `[y/N] n` (answer echoed on the prompt row), output contains `rollout cancelled`, `exit == Some(2)`.
- Ctrl+D at the password prompt settles `Failed` with output containing `aborted (EOF)` and a line ending `Password: ^D`.
- "Run the stubborn worker": `s` puts the task in `Cancelling` and the text contains `stopping`; 4 ticks later it is still `Cancelling` ("TERM alone does not end it"); 10 more ticks and it is `Stopped` with output containing `SIGKILL`.
- Ctrl+C in input mode puts the task in `Cancelling` and appends the line `^C`.
- On a finished task `i` reports `finished · nothing reads input`.
- `activity.rs: prompts_pause_the_script_and_take_exact_input`: `redacted_len == 7`, `last_line()` stays `Password: ` (no echo), the script clock excludes waiting time, `Overwrite? [y/N] y` echo and `stdin[0].bytes == b"y\r"`.
- `activity.rs: key_mapping_matches_the_terminal_contract`: Enter to `\r`, Backspace to `0x7f`, Tab, Ctrl-C to `3`, Ctrl-D to `4`, Ctrl-@ to `0`, Shift `A`, `é` as UTF-8; `Up` and `Esc` map to `None`, Alt chords to `None`.
- `activity.rs: ctrl_c_stops_and_input_is_refused_outside_ownership`: input after Ctrl+C is `Err(InputError::NoStdin)`; a monitor and a `Queued` task refuse input (`NoStdin`, `NotStarted`); `héllo\ttab\r` is delivered and echoed once.
- `activity.rs: stop_enters_cancelling_and_settles_through_the_supervisor`: cooperative child `Stopped` with `exit 130` after `TERM_EXIT_TICKS`; resistant child stays `Cancelling` until `KILL_AFTER_TICKS`, then `SIGKILL` line and `exit 137`.
- `app_tests_flows.rs: activities_survive_navigation_and_merged_logs_keep_identity`: Ctrl+W on a running tab opens `Stop frontend dev?`, Esc keeps it running (`tabs.len() == 5`); `quitting_a_remote_host_with_work_running_names_the_box`: Ctrl+Q with live work opens `Quit holla❯ on prod-eu-1?`, Esc leaves `quit == false`.

**Captures:** `shots/h_hp15_prompt` (`Password:` row, header `waiting for input`) · `shots/h_hp15_input_mode` (`typing goes to stdin · Esc returns`) · `shots/h_hp15_answered` (secret not echoed, `authenticated`, second prompt) · `shots/h_hp15_cancelled` (`[y/N] n`, `rollout cancelled by operator`, `failed · exit 2`) · `shots/h_hp15_stopping` (`cancelling`, `SIGTERM to the process group`) · `shots/h_hp15_killed` (`no exit after SIGTERM for 750 ms · SIGKILL`, `stopped · exit 137`) · base matrix `shots/h_parity_task-input_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| E29 | UI side only: `hp15_…` shows the prompt without newline and the `waiting for input` badge (`h_hp15_prompt`). No PTY is modeled. | Real PTY, setsid/TIOCSCTTY, `/dev/tty` readers. |
| E33 | `hp15_…`: `i` on a live task enters input mode (`typing goes to stdin`), a finished task reads `finished · nothing reads input`; `e33_e39_input_mode_returns_on_esc_ends_with_the_program_and_a_kill_reports_reaping`: Esc leaves input mode ("Keyboard returned to holla") and input mode ends by itself when the program exits (`input == false` after Succeeded); `ctrl_c_stops_and_input_is_refused_outside_ownership`: monitor and queued tasks refuse. | Writable PTY check. |
| E34 | `key_mapping_matches_the_terminal_contract` (CR, DEL, Tab, Unicode, Shift, Ctrl alphabet/@, navigation and Esc not forwarded); `hp15_…` Ctrl+D and Ctrl+C. | none |
| E35 | `hp15_…`: partial `Password: ` row with `waiting` secret, `input wanted` badge; `prompts_pause_the_script_and_take_exact_input`: nothing emitted while waiting, prompt cleared by the answer. The prompt is declared by the fixture; case-insensitive `password` detection from an idle buffer is not modeled. | Idle PTY buffer detection. |
| E38 | `hp15_…`: `s` gives `Cancelling`, still `Cancelling` after TERM, `Stopped` with `SIGKILL` after the grace; `stop_enters_cancelling_and_settles_through_the_supervisor` (`exit 130`/`137`, `KILL_AFTER_TICKS` is 750 ms); `queued_work_never_spawns_after_a_cancel`. | Real SIGTERM/SIGKILL to process groups; live spawn/cancel race. |
| E39 | `e33_e39_input_mode_returns_on_esc_ends_with_the_program_and_a_kill_reports_reaping`: `s` on the SIGTERM-resistant worker settles Stopped with the output line `killed (SIGKILL) · descendants reaped` (`h_hp15_killed`). | Subreaper (Linux) / group extinction (macOS) and waitpid. |
| E40 | Not represented; no test or fixture models terminal/render failure. | Armed supervisor on early return and terminal restoration. |
| E43 | Not represented; headless stdin bridge is deferred. | Headless stdin to PTY bridge. |
| E37 | `activities_survive_navigation_and_merged_logs_keep_identity`: `Stop frontend dev?` on Ctrl+W, Esc keeps running; `quitting_a_remote_host_with_work_running_names_the_box`: `Quit holla❯ on prod-eu-1?`, Esc keeps running. Confirm-stop path and the Stop/Keep-running wording are not asserted. | none |

**Deferred remainder:**
- Real task PTYs/stdin, process groups, TERM/KILL escalation, subreaper/reaping and OS-specific process integration (Later); isolated PTY tests on macOS and Linux.
- Not proven by tests: two prompting tasks with single-owner routing; no stale PID signalling; no owned descendants before `Stopped`; output/report retention after terminal failure; restored terminal after failure; no navigation leakage from input mode beyond `Up`/`Esc` mapping to `None`.
- Captures named in the contract but absent from tools/holla_parity_flows.sh: keep-running/stop dialog; PTY restoration transcripts.

**Limits:**
- The 750 ms escalation is `KILL_AFTER_TICKS == 10` on a virtual clock; real signal delivery, timing and reaping cannot be proven here.
- Prompt detection is fixture-declared, not parsed from real terminal bytes.
