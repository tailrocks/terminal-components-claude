# Subagent proof and verification contract

**Authority:** [`subagent-only-policy.md`](subagent-only-policy.md) and
[`path-contract.md`](path-contract.md).

This contract defines evidence for the refactoring tasks. It does not create a
host daemon, container workflow, receipt service, lifecycle CLI, or integration
authority. The coordinator delegates implementation, verification, and review
to subagents; the coordinator integrates only reviewed task commits.

## Fixed inputs

- Product/source oracle: `02f5294bfdbf38004cc49130d0aff1d01f31434c`.
- Architectural parent: `7b27732a8c3c131760ec3438f641cb3c11343a42`.
- Latest taskfmt source:
  `/Users/donbeave/Projects/taskfmt/task-format`.
- Latest taskfmt revision:
  `afd3b575dbcc7044620bec4b9493a74eca3e5ef2`.
- Latest taskfmt version: `0.2.0`.
- Frozen `visual-baseline` tag/release/store: immutable.

Taskfmt is used only for one task at a time:

```sh
"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify \
  --root "$WORKTREE" \
  --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" \
  --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

No `taskfmt init`, `taskfmt status`, `taskfmt-host`, `taskfmt-runtime`,
`taskfmt run`, `taskfmt monitor`, `taskfmt promote`, or other lifecycle command
is permitted. Taskfmt never starts or supervises a container.

## Check workers

`verify.toml` check commands are untrusted task workers. They run on the host
inside the verifier subagent's isolated worktree and external run directory.
They may execute real production code and tests, but they must not write:

- task packages or trusted fixtures;
- the immutable visual oracle or baseline tag;
- another task's worktree or run directory;
- coordinator state, refs, or unrelated files.

Allowed proof operations are task check workers such as:

```text
tools/refactor-proof/bin/tc-proof preflight --context RUN_DIR/contexts/CHK-001.json
tools/refactor-proof/bin/tc-proof required --context RUN_DIR/contexts/CHK-002.json
tools/refactor-proof/bin/tc-proof oracle --context RUN_DIR/contexts/CHK-003.json --namespace APP
tools/refactor-proof/bin/tc-proof capture --context RUN_DIR/contexts/CHK-003.json --lane direct
tools/refactor-proof/bin/tc-proof compare --context RUN_DIR/contexts/CHK-004.json
tools/refactor-proof/bin/tc-proof account-tests --context RUN_DIR/contexts/CHK-005.json
tools/refactor-proof/bin/tc-proof architecture --context RUN_DIR/contexts/CHK-006.json
tools/refactor-proof/bin/tc-proof close --context RUN_DIR/contexts/CHK-007.json
```

These are check workers, not coordinator commands. Context files are
task-scoped, immutable for one verification run, and stored under `RUN_DIR`.
Each context binds task ID, worktree commit, scope base, operation, required
outputs, and expected provenance. Missing, extra, substituted, cross-run, or
mutated contexts fail verification.

## Evidence ownership

The implementer subagent produces a scoped commit and advisory test output.
The verifier subagent produces raw `taskfmt lint`/`verify` output plus check
logs. The reviewer subagent independently checks:

1. task scope and writable paths;
2. exact worktree commit and scope base;
3. taskfmt executable revision/version/SHA;
4. check exit codes, final `DONE`, and required outputs;
5. no oracle, trust-root, ref, or unrelated-file mutation.

The reviewer returns `VERIFIED`, `REJECTED`, or `BLOCKED`. A passing taskfmt
result alone never authorizes integration or task completion.

## Visual and behavioral evidence

Visual comparisons must read the frozen oracle; candidates never write or bless
it. Candidate capture output is untrusted and lives only in `RUN_DIR` until
the reviewer accepts the task evidence. Behavioral evidence must cover real
production update/draw/event paths, not detached fixtures or copied oracle
frames. Any missing oracle asset is a blocker, not a reason to weaken a gate.

## Integration

After verifier and reviewer approval, the coordinator integrates the exact
subagent commit against the expected parent with ordinary Git compare-and-swap
checks. Integration is not performed by taskfmt or a proof worker. Use the
repository commit policy: `git commit -s` and
`Co-authored-by: Codex <codex@openai.com>`.
