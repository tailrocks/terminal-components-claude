# HP14 — Task execution, output and results

[Tracker](PLAN.md) · [Current scope](scope.md) · [Shared acceptance](shared-contracts.md#per-change-acceptance-gate)

## Current phase — simulated representation

Implement typed simulated action outcomes, sequential/parallel scheduling, ordered final bytes, per-task retained output and completion independent of page dismissal. Unknown scripts cannot generically succeed.

Every mapped UI/action variant, named fixture, visible failure/ownership/policy
case and capture requirement below applies to the simulated representation.
Exact process/filesystem behavior is modeled, not claimed as native proof.

- [x] Implement all mapped UI variants and typed simulated outcomes.
- [x] Retain named fixtures and exact target/cwd/argv/state/effect assertions.
- [x] Inspect keyboard/pointer, size/color and decisive state captures.
- [x] Record row-level representation evidence and deferred operational clauses.

## Later — operational and non-UI integration

Production executor/adapters, actual child processes and aggregate/headless CLI contracts.

- [ ] Complete remaining non-UI contracts and connect production adapters.
- [ ] Prove the retained operational/platform contract with isolated resources.

## Full retained contract

The classification and source observations below are the planning baseline.
Current-phase completion does not imply deferred clauses have passed.

**A · P1 · planned, not implemented.**

**Preserved capability / current gap:** Preserve real task specifications, sequential/parallel scheduling, live output, focus and summaries in activities. Current fixtures cover selected scripts, not all execution semantics.

**Source evidence and mandatory scope:** [matrix HP14](../parity/holla-parity-matrix.md#hp14--task-execution-output-and-results) — `L23`, `E25`, `E26`, `E27`, `E28`, `E30`, `E31`, `E32`, `E36`, `E41`, `E42`, `E44`. Exact old source/test citations and current preview anchors are in those rows.

**Junie interaction and safety:** Model queued/running/cancelling/succeeded/failed/cancelled with exact task identity, cwd/host and result. Independent batch jobs continue after peer failure; dependent plan steps block. Preserve per-task retained output, manual reading position and follow-tail, post-completion inspection, empty batch and final shell summary. Every mutating action needs an explicit effect or failure; unknown scripts must not silently succeed.

**Architecture / reusable components:** Replace generic-success fallback and incomplete effect-by-ID switch with typed execution events and action-owned outcomes. Separate runner completion from UI dismissal. Reuse Activities, tabs, TextViewport, StatusBar and Plan; adopt F10–F15 retention contracts.

**Required deterministic fixture:** `parity-executor` — sequential and parallel jobs, first failure, queued cancellation, missing executable/cwd, CRLF/ANSI/invalid UTF-8, no-final-newline, fast exit, stream interleave, burst retention, scrolled output, empty jobs and mixed results.

**Acceptance / automated verification:** Assert final buffered bytes arrive before terminal state, per-task ordered output, no cross-task retargeting, tail preservation, return-to-Here continuity and task/page focus bindings. Assert aggregate CLI failure and truthful UI result even after dismissing output. Test independent versus dependent failure policies separately. The common keyboard/mouse/state and simulation/operational gates above apply to every mapped UI workflow.

**Capture / visual verification:** Capture pending/running/mixed failures, retained scrolled output, final unterminated output, empty batch and completed task after returning from Here.

**Dependencies:** HP15/HP16/HP17; F10–F15/F19/F20/F21/F23. Task ownership, cancellation, stale-result rejection and shutdown are required; a generic worker framework is outside this plan.

## Evidence

**Slice status:** current simulated slice complete for the journeyed variants; gap: missing executable/cwd, queued cancellation and fast exit are proven only at unit level, parallel scheduling is asserted only for whichever mode the git batch fixture declares, and empty jobs, scrolled-output continuity and the final shell summary are not journeyed · Later operational clauses open.

**Scenario and journey:** `parity-executor` (fixture fn `executor` in src/bin/holla/domain/parity.rs) · `hp14_output_streams_are_exact_and_retention_drops_are_stated` in src/bin/holla/app_tests_parity.rs · supporting unit tests: `src/bin/holla/domain/activity.rs: final_bytes_land_before_the_terminal_state_and_fast_exits_keep_output`, `scripts_emit_in_order_and_settle_the_end_state`, `spawn_failure_is_output_and_a_failed_done`, `queued_work_never_spawns_after_a_cancel`, `retention_is_bounded_and_counted`, `raw_streams_normalise_like_the_executor`; `src/widgets/viewport.rs: follows_tail_and_wheel_leaves_it`, `reading_position_is_retained_while_follow_is_off`, `retention_is_enforced_on_every_ingestion_path`, `eviction_never_retargets_a_selection`; `src/bin/holla/app_tests_proofs.rs: a_burst_of_output_is_appended_not_rebuilt_and_equals_a_fresh_layout`; `src/bin/holla/app_tests_flows.rs: activities_survive_navigation_and_merged_logs_keep_identity`.

**What the journey asserts:**
- "Emit a mixed stream" argv is `./emit-stream @/Users/alex/work/batch` and settles `Succeeded`.
- Output lines are exactly `first` (CRLF stripped), `\u{1b}[32mgreen\u{1b}[0m` (bytes kept), a line containing `bad \u{fffd} byte` (invalid UTF-8 replaced), and `last fragment` (unterminated final fragment is a line).
- The rendered text contains `green` and never a raw `\u{1b}`.
- "Emit a burst": `output.len() == 4000`, `dropped == 500`, `output[0]` is `burst line 500`, and the screen text contains `500 earlier lines dropped`.
- `parity-git-batch` push: `batch.mode` equals the item's `batch_mode`; first member `Running`; in `Sequential` mode the second is `Queued` and the text contains `queued`; in `Parallel` mode the second is `Running`.
- `activity.rs: final_bytes_land_before_the_terminal_state_and_fast_exits_keep_output`: a zero-tick task is `Succeeded` with `output.len() == 2` ("fast task output is drained before Done"); a line buffered past the end tick is the `last_line()` at `Succeeded` (`late buffered`).
- `activity.rs: spawn_failure_is_output_and_a_failed_done`: `Failed`, `exit == Some(127)`, first line contains `cannot start`, later input is `Err(InputError::Finished)`.
- `activity.rs: queued_work_never_spawns_after_a_cancel`: a `Queued` task stopped becomes `Stopped` with `exit == Some(130)` and never emits `would run`.
- `viewport.rs: follows_tail_and_wheel_leaves_it`: wheel up clears `follow`, `End` restores it; `reading_position_is_retained_while_follow_is_off`: the viewed line stays on top while lines append and evict.
- `app_tests_flows.rs: activities_survive_navigation_and_merged_logs_keep_identity`: after Alt+0 to Here and 30 ticks the activity's output is "retained and growing" and the earliest line is still there; Ctrl+W on a running tab asks `Stop frontend dev?`.

**Captures:** `shots/h_hp14_stream` (four normalised lines, escape shown as `^[[32m`, following) · `shots/h_hp14_burst` (tail of 4500 lines with meta `500 earlier lines dropped · last 4000 kept`) · `shots/h_hp14_find` (`/` find `line 44` · `1 of 100` inside the retained tail) · base matrix `shots/h_parity_executor_{80x24,100x30,120x40,160x50,mono}`. Provenance in each frame's `.manifest.json` (source git revision and dirty flag, binary sha256, arguments, geometry, colour environment, tmux/python/Pillow versions, fonts) and raster fidelity in `.png.fidelity.json`. Visual inspection: recorded by the integrator in the manifest `review` field.

**Row-level representation evidence:**

| Row | Current representation evidence | Deferred clause |
| --- | --- | --- |
| L23 | `activities_survive_navigation_and_merged_logs_keep_identity`: output "retained and growing" after returning to Here; `hp14_…` returns to Here with Alt+0 between runs and each activity keeps its own id and output. | none |
| E25 | `hp14_…`: argv `./emit-stream @/Users/alex/work/batch` (program, argv, cwd shown in the header `here · ~/work/batch · mbp` in `h_hp14_stream`). | Environment inheritance and real process session/group. |
| E26 | `hp14_…` (`parity-git-batch`, `Sequential`): first `Running`, second `Queued`, text `queued`; `queued_work_never_spawns_after_a_cancel`: cancelled queued work is `Stopped`, `exit 130`. Continuing after an earlier member fails is not asserted (HP13's failing stage is the last one). | Real child sequencing. |
| E27 | `hp14_…` `Parallel` branch: second member `Running` beside the first. Only exercised when the fixture declares `Parallel`; aggregate all-succeed result not asserted. | Real parallel jobs and join. |
| E28 | `ActivityState` is `Queued`, `Running`, `Cancelling`, `Succeeded`, `Failed`, `Stopped` (activity.rs); `scripts_emit_in_order_and_settle_the_end_state`: `Failed`, `exit == Some(1)`, finished activities are inert. | Completion gated on real executor cleanup. |
| E30 | `hp14_…`: `first`, `\u{1b}[32mgreen\u{1b}[0m`, `bad \u{fffd} byte`, `last fragment`; `raw_streams_normalise_like_the_executor`; `final_bytes_land_before_the_terminal_state_and_fast_exits_keep_output`. stdout/stderr interleave order is scripted, not measured. | Real reader join and stream interleave. |
| E31 | `follows_tail_and_wheel_leaves_it` (unfollow on wheel, `End` refollows); `reading_position_is_retained_while_follow_is_off`; `retention_is_enforced_on_every_ingestion_path`; `eviction_never_retargets_a_selection`. | none |
| E32 | `h_hp14_find` shows `/` find; `End`/wheel proven in viewport tests. `h`/`l` task selection and `j`/`k`/PageUp/PageDown on the activity page are not asserted by any test. | none |
| E36 | `h_hp13_plan` status line `4 ok · 1 failed · 0 cancelled` (capture only); `hp15_…` asserts `waiting for input` and `input wanted`. Live count updates are not asserted. | none |
| E41 | `spawn_failure_is_output_and_a_failed_done`: `cannot start`, `Failed`, `exit 127`; `queued_work_never_spawns_after_a_cancel` and HP15 stop tests: cancellation ends `Stopped` with `exit 130`/`137`, never `Succeeded`. Not journeyed through the UI. | Real PTY/spawn/wait errors. |
| E42 | Not represented; headless runner is deferred. | Headless CLI runner and aggregate exit code. |
| E44 | Not proven: no test asserts a per-task summary printed on close or `No tasks to run` for an empty batch. | Terminal restore and durable shell summary. |

**Deferred remainder:**
- Production executor/adapters, actual child processes and aggregate/headless CLI contracts (Later).
- Not proven by tests: aggregate CLI failure; truthful result after dismissing output is covered only by Here round-trips, not by page close; empty batch; missing executable/cwd through the UI; `h`/`l`/`j`/`k`/PageUp bindings; final shell summary.
- Captures named in the contract but absent from tools/holla_parity_flows.sh: pending/running/mixed failures, final unterminated output on its own, empty batch, completed task after returning from Here.

**Limits:**
- Scripted lines on a virtual clock cannot prove real stream interleave, drain-before-exit or process exit status.
- The parallel branch is only exercised if `git.push-all-remotes` declares `Parallel`; the test accepts either mode.
