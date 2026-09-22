# Subagent task execution protocol

**Authority:** [`subagent-only-policy.md`](subagent-only-policy.md).
This protocol replaces the retired host/container executor workflow.

## Roles

- **Coordinator:** schedules the DAG, assigns isolated worktrees, reviews
  evidence, and integrates accepted commits serially.
- **Implementer subagent:** edits only the task's allowed files and runs
  advisory checks.
- **Verifier subagent:** freezes the implementer worktree read-only for review,
  runs the latest standalone taskfmt lint/verify for exactly one task, and
  records raw output under that task's external run directory.
- **Reviewer subagent:** independently checks the diff, task scope, test
  evidence, and verifier output. It returns `VERIFIED`, `REJECTED`, or
  `BLOCKED` with exact locations.

No role starts a container, uses Docker/Podman, invokes a taskfmt runtime or
host lifecycle binary, or writes the frozen visual oracle.

## Inputs

For task `TASK-NNN`, the coordinator supplies:

```text
TASK_DIR    absolute read-only catalog package
WORKTREE    isolated host-local subagent worktree
RUN_DIR     external task-scoped logs and temporary outputs
SCOPE_BASE  immutable parent commit
TASKFMT     /tmp/taskfmt-latest-install/bin/taskfmt (exact pinned build)
```

The task package, `verify.toml`, and trusted inputs are read-only to the
subagent. The subagent may write only the paths declared by that task.

## Required sequence

1. Coordinator checks the current readiness report and dependency receipts.
2. Coordinator spawns one implementer subagent in a fresh worktree.
3. Implementer reads the task contract and all required references, makes the
   scoped change, and runs ordinary advisory checks with external outputs.
4. Implementer commits only its task change; no push, tag, oracle, or main
   update is allowed.
5. Verifier subagent checks that the worktree is still within task scope and
   prepares native proof execution with `scripts/campaign-build-proof.sh`, an
   exact external `RUN_DIR/contexts` set, and the reviewed observer/result
   environment for each proof check. The helper is a normal host-local Cargo
   build; it is not a taskfmt operation and never starts a container. Then it
   runs exactly:

   ```sh
   "$TASKFMT" lint "$TASK_DIR"
   "$TASKFMT" verify \
     --root "$WORKTREE" \
     --task-dir "$TASK_DIR" \
     --base "$SCOPE_BASE" \
     --progress "" \
     --log-dir "$RUN_DIR/taskfmt-logs"
   ```

   The dispatcher captures the exact combined stdout/stderr stream, including
   the final `DONE` line, in a verifier-owned secure temporary file outside
   `$RUN_DIR`. After taskfmt exits, it validates that `taskfmt-logs` is a real
   directory and atomically replaces `$RUN_DIR/taskfmt-logs/verify.log` with
   that capture. Unsafe or non-directory log paths fail closed.

   Task checks must use host-local paths. Proof checks use the exported
   `$RUN_DIR/contexts/CHK-NNN.json` path; latest taskfmt executes shell checks
   under the explicit worktree root and does not interpolate `argv` values.
   A literal `/task`, `/work`, `/proof`, or `/run` in `verify.toml` is a
   blocking contract defect; do not create a mount or firmlink to make it pass.
   Missing native comparator or context preparation is also a blocking
   precondition; the dispatcher fails before taskfmt instead of allowing a
   partial proof run.

6. Reviewer subagent independently reads the diff and raw verifier output.
7. Coordinator integrates only after verifier and reviewer both pass. The
   coordinator reruns branch-level gates after serial integration.

This sequence is contingent on a **GO** readiness verdict. The current
post-repair target is **NO-GO**: proof preparation is stale, TASK-071 accepts
extra validate-context inputs, and the TASK-072 generated-bundle and broker
field-allowlist repair is only a candidate with no independent
verifier/reviewer acceptance. Do not create task worktrees, spawn
implementers, run task-owned verification, or integrate commits. A future
**GO** is necessary but not sufficient; DAG dependencies, receipt binding,
verifier evidence, reviewer approval, branch gates, and final
visual/behavioral gates remain mandatory.

## Verification result

The verifier records:

```text
TASK: TASK-NNN
LINT: command=taskfmt lint exit=<n>
VERIFY: command=taskfmt verify exit=<n> last_line=<DONE|other|NOT_RUN>
WORKTREE: <commit>
BASE: <scope-base>
RUN_DIR: <external-path>
```

`DONE` is necessary taskfmt output, not integration authorization. A reviewer
must verify that the output belongs to the exact task/worktree/run and that no
scope, oracle, trust-root, or dependency rule was bypassed.

## Forbidden operations

- Any container, image, Docker, Podman, mount, firmlink, or container runtime.
- `taskfmt init`, `taskfmt status`, `taskfmt-host`, `taskfmt-runtime`,
  `taskfmt run`, `taskfmt monitor`, `taskfmt promote`, or branch lifecycle
  commands.
- Empty/forged/stale evidence presented as another task's verification.
- Direct coordinator edits to task-owned production files.
- Main merges, force pushes, oracle writes, baseline blessing, or tag changes.

The `visual-baseline` tag, release, and store must remain unchanged by policy;
provider-level immutability is not assumed.
