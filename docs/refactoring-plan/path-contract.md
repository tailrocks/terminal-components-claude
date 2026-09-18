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

Every check command must use a relative `WORKTREE` path or the exported
`$RUN_DIR` variable for an external host-local run path. `shell` checks use
`$RUN_DIR/contexts/CHK-NNN.json`; taskfmt runs them with bash and does not
perform any path substitution itself. The following legacy namespaces are
forbidden:

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

Before verification, the verifier builds the native comparator with
`scripts/campaign-build-proof.sh` and supplies an exact, immutable context set
under the external run directory. The helper records a build receipt beside
the native binary; the dispatcher checks that receipt against the current
worktree commit and binary hash. Context contents must bind the task, check,
run, scope base, and provenance. Taskfmt does not discover, generate, or bind
these proof inputs.

The native comparator target is verifier-owned and host-local. Set
`TC_PROOF_TARGET_DIR` to an existing absolute directory outside the candidate
worktree before running the build helper. An empty `CARGO_TARGET_DIR` is
ignored; a non-empty `CARGO_TARGET_DIR` is accepted only when it resolves to
the same root as `TC_PROOF_TARGET_DIR`. If neither variable is set, the helper
inspects `$WORKTREE/target` and fails closed when that default is missing,
inside the worktree, or a symlink into a shared cache. The selected root and
its `debug/tc-proof` child must be regular host paths; the helper does not
create or follow a target-root symlink.

The helper builds with `CARGO_TARGET_DIR` bound to the selected real path and
writes `debug/tc-proof.build.json`. That receipt binds the resolved worktree,
source `HEAD` and tree, exact comparator path, selected Cargo target root, and
SHA-256 of the executable. A missing, linked, stale, or mismatched binary or
receipt is a failed precondition. The dispatcher resolves the same target root
and revalidates the complete receipt before native preparation or standalone
`taskfmt verify`; `taskfmt lint` remains taskfmt-only.

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
