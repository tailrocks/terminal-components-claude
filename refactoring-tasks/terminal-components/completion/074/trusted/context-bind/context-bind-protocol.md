# context-bind protocol (TASK-074 authority)

Independent judge for per-check context completeness. The driver asserts
that `tc-proof prepare` (Rust `tools/refactor-proof/src/verifier.rs`
`build_context`/`bind_*` plus the `tools/refactor-proof/runner` glue)
emitted complete, well-formed, host-bound contexts for the adapter-task
worker paths. It is not tc-proof and never executes proof workers.

## Invocation

Each `verify.toml` check runs the driver with the operation it binds:

```text
python3 <task>/trusted/context-bind/context-bind-driver.py \
  --check-id CHK-00X --context "$RUN_DIR/contexts/CHK-00X.json" \
  [--namespace NAME] [--gate CHK-A,...] \
  tools/refactor-proof/bin/tc-proof <operation>
```

The trailing `tools/refactor-proof/bin/tc-proof <operation>` positionals are
consumed meaningfully by both stages: prepare's `discover_operation` binds
the check's operation from them, and the driver uses the dispatcher path for
the tool-identity cross-check and the operation to select its assertion
profile. The check genuinely needs that operation's bindings; no worker is
executed because worker behavior is owned by TASK-071/TASK-072 prerequisite
evidence, not by this task.

Environment: `RUN_DIR` must be set (standard verify procedure). The driver
reads `$RUN_DIR/contexts/`, `$RUN_DIR/results/`, `$RUN_DIR/context-index.json`,
this package's `verify.toml` and `trusted/check-context-templates/`, the
candidate dispatcher bytes, and worktree git identity. It writes nothing.

## Profiles

- `preflight` (CHK-001): common bindings only. Passes on the current tree;
  proves the harness works and guards every preserved binding.
- `oracle` (CHK-002, `--namespace showcase`): common + native oracle
  contract (`family`, `source_sha256`, `mapping_owner`, `mapping`,
  `resized_sizes`) + task-derived members. Fails on the current tree.
- `compare` (CHK-003): common + nested comparator context (roots,
  manifests, required set, adapters, tool, host-bound report) + identity
  cross-checks + task-derived members. Fails on the current tree.
- `account-tests` (CHK-004): common + lifted template declarations
  (`mode`, `family`, `register_kind`, `requires_*`, `worker_context`,
  `template_sha256`) + derived preparation register (`original`,
  `required`, `preparation_register`, no accepted receipts) +
  task-derived members. Fails on the current tree.
- `architecture` (CHK-005): common + `configuration` shape +
  task-derived members. Fails on the current tree via membership.
- `close` + `--gate` (CHK-006): own close context + full re-run of all
  four worker-path profiles + index/result integrity. Fails today.

## Common assertions (every profile)

Strict duplicate-key JSON; schema/task/check/run/operation identity;
40-hex source pins and 64-hex digests; allowed-keys schema mirror;
observer-sequence shape; adapter/source-authority bindings; axes/members
shape (well-formed; exact provider agreement stays with the adapter
end-to-end gates); qualification common trust (candidate tree, oracle, runtime
outputs, observer transport, comparator, tool bytes vs the real candidate
dispatcher); task-manifest phase/requirements/acceptance agreement with
`verify.toml`; accepted+integrated dependency receipts; worktree HEAD/tree
and scope-base ancestry via git; context-index path/digest membership;
preparation-result readiness and digest.

## Verdicts

Exit 0 prints `BIND-RESULT <CHK> status=passed`. Any failure prints one
`FAIL <scope> <field>: <detail>` line per violated assertion plus
`BIND-RESULT <CHK> status=rejected failures=N`, and exits 1.

## Non-goals

The driver does not execute workers, speak the observer protocol, or
replicate provider payload computations (e.g. archive digests, fixture
seed values). Those behaviors are owned by TASK-071/TASK-072 prerequisite
evidence and by the adapter tasks' own end-to-end gates once unblocked.
Full worker acceptance on TASK-002..007 remains those tasks' obligation;
this task proves the prepared contexts are complete enough to run them.
