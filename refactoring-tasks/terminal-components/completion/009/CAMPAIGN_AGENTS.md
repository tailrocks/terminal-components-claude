# Campaign execution protocol overlay

**Campaign branch:** `refactor/holla-parity`.

This overlay is current for every terminal-components task. It requires
host-local isolated subagents and supersedes any historical executor,
operator, container, or taskfmt lifecycle instructions.

## Hard rules

- Never start, manage, or depend on containers, Docker, Podman, images, mounts,
  root firmlinks, or a container runtime.
- Only subagents may implement task-owned production changes.
- Use one implementer, one verifier, and one reviewer subagent per task with
  disjoint worktrees/run directories.
- The coordinator only schedules, reviews evidence, and integrates accepted
  commits serially.
- Never mutate `visual-baseline`, write the frozen oracle, merge `main`, or
  force-push.

## Taskfmt boundary

Use only the standalone latest taskfmt `0.2.0` at revision
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`:

```sh
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify --root "$WORKTREE" --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" --progress "" --log-dir "$RUN_DIR/taskfmt-logs"
```

These are the only taskfmt commands allowed. Do not invoke `taskfmt init`,
`status`, `taskfmt-host`, `taskfmt-runtime`, `run`, `monitor`, `promote`, or
any other lifecycle/dispatch command. Taskfmt never starts a container.

## Paths and evidence

The verifier subagent receives explicit host-local `TASK_DIR`, `WORKTREE`,
`RUN_DIR`, `TASKFMT`, and `SCOPE_BASE` paths. Every `verify.toml` argv must use
relative worktree paths or explicit run paths. Literal `/task`, `/work`,
`/proof`, and `/run` namespaces are legacy defects; never create mounts or
firmlinks to satisfy them.

Local compiler/tests are advisory until the verifier subagent records the
per-task taskfmt result. A taskfmt `DONE` line is evidence, not integration
authority. The reviewer must independently check scope, commit identity,
outputs, and immutable-oracle protections.

See `docs/refactoring-plan/subagent-only-policy.md`,
`docs/refactoring-plan/campaign-executor-protocol.md`, and
`docs/refactoring-plan/proof-contract.md`.
