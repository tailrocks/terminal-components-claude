# Host-local verification path contract

**Authority:** This document defines the filesystem contract for subagent task
validation. Refactoring never uses containers, Docker, Podman, images, mounts,
root firmlinks, or container path namespaces.

## Per-task namespaces

The coordinator gives each verifier subagent explicit absolute paths:

| Name | Meaning | Access |
| --- | --- | --- |
| `TASK_DIR` | One numbered task package under the immutable catalog | read-only |
| `WORKTREE` | One isolated host-local candidate worktree | read-write within task scope |
| `RUN_DIR` | External logs and temporary verification outputs for this task | subagent-owned |
| `TASKFMT` | Pinned standalone `taskfmt` executable | read-only |
| `SCOPE_BASE` | Immutable parent commit for this task | read-only identity |

No path is mounted or translated. `taskfmt verify` receives the real host
paths directly:

```sh
"$TASKFMT" verify \
  --root "$WORKTREE" \
  --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" \
  --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

The empty progress argument is intentional: taskfmt is only the per-task gate;
subagent coordination state is not a taskfmt lifecycle input.

## `verify.toml` rules

Every check argv must use a relative path from `WORKTREE` or an explicit
host-local path in `RUN_DIR`. The following legacy namespaces are forbidden:

```text
/task
/work
/proof
/run
```

They are migration defects, not directories to create.
`scripts/campaign-preflight.sh` fails closed while any numbered package still
contains one. A task cannot be dispatched until its `verify.toml` is
host-local.

Proof workers, fixtures, contexts, logs, and temporary captures belong to the
current subagent's `RUN_DIR` or a task-scoped path under `WORKTREE`; they must
not write trust roots, the frozen oracle, another subagent's worktree, or
another task's run directory.

## Worktree contract

| Phase | Owner | Location |
| --- | --- | --- |
| Implementation | implementer subagent | isolated host-local `WORKTREE` |
| Verification | verifier subagent | same frozen worktree, separate `RUN_DIR` |
| Review | reviewer subagent | read-only view of diff and evidence |
| Integration | coordinator | serial branch update after review |

Subagents never share writable worktrees or build directories. The coordinator
does not edit task-owned production files.

## Taskfmt limits

Only these standalone commands are valid for this campaign:

```sh
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" --log-dir "$RUN_DIR/taskfmt-logs"
```

Do not invoke `taskfmt init`, `taskfmt status`, `taskfmt-host`,
`taskfmt-runtime`, or any lifecycle/dispatch/promotion command. Taskfmt never
starts a container and never authorizes integration.
