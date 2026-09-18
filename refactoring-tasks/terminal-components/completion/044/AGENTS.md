# Execution protocol

Your goal is to fully implement this task: `$TASK_DIR/README.md`.

The task package is read-only. It contains the task contract and verification
configuration. All implementation belongs in the isolated host-local
`$WORKTREE/`; coordination notes belong in `$RUN_DIR/`.

## Roles

- The implementer subagent changes only files allowed by this task's
  `verify.toml`.
- The verifier subagent independently runs the task checks and the latest
  standalone taskfmt lint/verify commands.
- The reviewer subagent checks the diff, task requirements, evidence, and
  acceptance mapping before integration.
- The coordinator assigns isolated subagent work, records evidence, and
  integrates only reviewed commits. The coordinator does not implement
  task-owned changes.

All work is host-local. No containers are used. Do not provision images,
mounts, or another execution environment for this task.

## Files

| Path | Access | Purpose |
| --- | --- | --- |
| `$TASK_DIR/README.md` | read-only | The task contract: goal, requirements, acceptance criteria, decisions, and checklist. |
| `$TASK_DIR/verify.toml` | read-only | Machine authority: checks, expected results, and writable paths; never edit. |
| `$RUN_DIR/progress.md` | read-write | Optional host-local subagent handoff and coordination log. It is not taskfmt state. |
| `$WORKTREE/` | read-write | The isolated repository where implementation changes happen. |

The completion gate is the latest standalone `taskfmt verify` run by the
verifier subagent with explicit host-local paths. `$SCOPE_BASE` is the recorded
scope-base commit. Taskfmt is validation only: it does not create coordination
state, dispatch workers, or integrate commits.

The task README uses canonical typed acceptance blocks. Each non-gate `AC-*`
block has one exact ` ```gherkin ` fence containing constrained Given/When/Then
behavior. Its `Verification` section has `Type`, real `Covers` requirement
IDs, and one `Check` ID. A gate has `Type: gate` and one `Check` ID, but no
`Covers` or Gherkin body. Commands and expected results belong only in
`verify.toml`; acceptance prose has no machine authority. These blocks are task
metadata, not Cucumber feature files, and have no runtime step definitions.

## Protocol

1. Read `$TASK_DIR/README.md` fully, then the files listed under "Read before editing".
2. If `$RUN_DIR/progress.md` exists, read it, run `git status` and `git diff --stat`, and continue from its evidence-backed handoff. Re-read the task README before editing after any resume or context compaction.
3. State in the transcript: task ID, one-sentence goal, acceptance IDs, and the first leaf.
4. Run every precondition check. If one fails, append the prescribed blocking event and emit `STATUS: BLOCKED`; do not work around it.
5. Work checklist leaves in ID order unless the README states dependencies. Append only valid handoff events for known leaves; do not edit the README checklist or duplicate it into progress.
6. Run the ordered verifier checks at the relevant leaf. A check that both passes and fails on the same tree is failed evidence: record it and stop `NEEDS_REPLAN`, naming the command.
7. When implementation leaves are complete, the verifier subagent runs:

   ```text
   "$TASKFMT" lint "$TASK_DIR"
   "$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
     --base "$SCOPE_BASE" --progress "" \
     --log-dir "$RUN_DIR/taskfmt-logs"
   ```

   Fix failures and rerun until both commands exit 0 with the verifier's
   complete logs retained under `$RUN_DIR/taskfmt-logs`.

## Handoff log

`$RUN_DIR/progress.md` is optional host-local coordination state maintained by
the subagents. Append evidence-backed updates only; taskfmt does not create or
supervise this file. A verifier result is valid only when its command, exact
task package, scope base, isolated worktree, and log directory are recorded.

## Prohibited

- Editing anything under `$TASK_DIR/`, or modifying/replacing the taskfmt binary.
- Deleting, skipping, weakening, or rewriting a failing test or check to make it pass.
- Special-casing known fixtures or verifier inputs.
- Suppressing errors, warnings, lint rules, type checks, or exit codes.
- Changing any file outside `writable_paths` in `$TASK_DIR/verify.toml`; the gate rejects every other path.
- Changing user-visible behavior with no `R-*`/`AC-*` name, even inside `writable_paths`; note it under `FOLLOW_UP`.
- Running any taskfmt command other than the per-task `lint` and `verify` commands above.
- Letting the coordinator edit task-owned production files instead of assigning an implementer subagent.
- Claiming `DONE` without verifier evidence from this session retained in the run log.

## Stop conditions

- `BLOCKED`: a precondition command exited non-zero, or an environment or dependency condition outside `writable_paths` is false. A missing executable or credential is `BLOCKED` too.
- `NEEDS_REPLAN`: satisfying the task requires changing its goal, acceptance criteria, fixed decisions, scope, or checklist; requirements contradict; or a material design decision is unresolved.
- `INCOMPLETE`: the turn or budget cap is reached first. Leave the handoff non-terminal and record the next evidence-backed action.
- Do not spin. A leaf with no evidence-backed action left is recorded as failed with its command and observed result; then move to the next independent leaf. Take a terminal only when no leaf has an evidence-backed action left, or immediately when a precondition or mixed-result rule requires it.

## Turn signal

At the end of every turn EXCEPT the one that carries the final report, print one line:

```text
GOAL_PROGRESS task=TASK-044 state=<derived-state> current=<ID|NONE> done_this_turn=<IDs|none> blocked=<ID|none>
```

On the final-report turn, print this line immediately before the report and
print nothing after the `GOAL_RESULT` line. `GOAL_RESULT` is the last line in
every terminal state.

## Final report

Last thing you print. Exactly this shape:

```text
STATUS: DONE | BLOCKED | NEEDS_REPLAN | INCOMPLETE
TASK: TASK-044
SUMMARY: <what changed, or why execution stopped and what was tried>
ACCEPTANCE:
- AC-001: PASS | FAIL | NOT_RUN — <command and observed result>
- AC-002: ...
VERIFY: command="$TASKFMT" verify exit=<n|NOT_RUN> last_line=<DONE|other|NOT_RUN>
CHANGED:
<verbatim `git diff --no-renames --name-status $SCOPE_BASE`, then the untracked lines of `git status --porcelain --untracked-files=all`; not recall>
DEVIATIONS: none | <list>
FOLLOW_UP: none | <smallest decision, dependency, or split needed>
GOAL_RESULT task=TASK-044 status=<STATUS>
```
