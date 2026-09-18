# Subagent-only refactoring policy

**Status:** current. Applies to every terminal-components task.

## Hard rules

- Never start, manage, or depend on a container, Docker, Podman, image,
  mount, root firmlink, or container runtime.
- All implementation work is performed by subagents in isolated host-local
  worktrees. The coordinator schedules work, reviews evidence, and integrates
  accepted commits; it does not implement task-owned production changes.
- Use one implementer subagent, one independent verifier subagent, and one
  reviewer subagent per task. Keep worktrees and build/run directories
  disjoint.
- Use only the standalone `taskfmt` binary built from
  `/Users/donbeave/Projects/taskfmt/task-format` at the pinned current
  revision.
- `taskfmt` is a task gate only. The only allowed project invocations are:
  - `"$TASKFMT" lint <one-task-package>` — validate that task package.
  - `"$TASKFMT" verify --root <subagent-worktree> --task-dir <one-task-package> --base <scope-base> --progress "" --log-dir <external-run-dir>` — verify that task.
- Do not use `taskfmt init`, `taskfmt status`, `taskfmt-host`,
  `taskfmt-runtime`, `taskfmt run`, `taskfmt monitor`, `taskfmt promote`, or
  any command that starts or supervises execution infrastructure.
- A taskfmt result is task evidence. It is not permission to merge, push,
  bless the visual oracle, or mark a task complete without independent review.

## Host-local task contract

Each subagent receives explicit absolute paths:

```text
TASK_DIR   read-only task package
WORKTREE   read-write isolated task worktree
RUN_DIR    external per-task logs and temporary outputs
TASKFMT    pinned standalone taskfmt executable
SCOPE_BASE immutable task scope base commit
```

`verify.toml` must invoke host-local relative paths or explicit paths supplied
for that subagent. Literal container namespaces (`/task`, `/work`, `/proof`,
`/run`) are legacy contract defects, not paths to provision. No task may be
dispatched until its checks are host-local.

The verifier subagent must build the native `tc-proof` comparator with
`scripts/campaign-build-proof.sh`, materialize the task's exact external
`RUN_DIR/contexts/CHK-NNN.json` set, and bind each check's run/task/source/
result/observer environment. The dispatcher only calls taskfmt after those
preconditions pass. Taskfmt never creates contexts or proof infrastructure.

## Task sequence

1. Coordinator creates an isolated host worktree and assigns one task to an
   implementer subagent.
2. Implementer makes scoped changes and runs ordinary advisory tests.
3. Verifier subagent runs `taskfmt lint` and `taskfmt verify` for that task,
   plus task-specific checks, all on the host.
4. Reviewer subagent inspects the diff and the verifier evidence.
5. Coordinator integrates only a reviewed, verified task commit.

The `visual-baseline` tag, release, and oracle store must remain unchanged by
policy. Their current pointers are recorded in the readiness report; provider
enforcement is not assumed.

## Readiness stop

If the current readiness report is not **GO**, stop. Do not create task
worktrees, spawn implementers, run task-owned verification, or integrate task
commits. **GO** is necessary but not sufficient: the coordinator must still
confirm DAG dependencies, receipt binding, verifier evidence, reviewer approval,
branch gates, and the final visual/behavioral gates.
