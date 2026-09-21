# bind-repair protocol (TASK-075 authority)

Independent judge for the three TASK-002-blocking prepare defects:
unrunnable compare roots, one-key accounting inventory, and the unsupervised
compare branch. The driver asserts prepared-context completeness for the
binding profiles and executes the fixed source branch once for the
supervision profile. It never executes 071/072-owned workers, the observer
provider, or the frozen dispatcher bundle.

## Invocation

Each `verify.toml` check runs the driver with the operation it binds:

```text
python3 <task>/trusted/bind-repair/bind-repair-driver.py \
  --check-id CHK-00X --context "$RUN_DIR/contexts/CHK-00X.json" \
  [--supervise] [--gate CHK-A,...] \
  tools/refactor-proof/bin/tc-proof <operation>
```

The trailing `tools/refactor-proof/bin/tc-proof <operation>` positionals are
consumed meaningfully by both stages: prepare's `discover_operation` binds
the check's operation from them, and the driver uses the dispatcher path for
the tool-identity cross-check and the operation to select its assertion
profile. `--supervise` selects the supervision profile for the compare
operation; without it the compare operation selects the roots profile.

Environment: `RUN_DIR` must be set (standard verify procedure). The driver
reads `$RUN_DIR/contexts/`, `$RUN_DIR/results/`, `$RUN_DIR/outputs/`,
`$RUN_DIR/context-index.json`, this package's `verify.toml` and
`trusted/check-context-templates/`, the candidate dispatcher bytes, and
worktree git identity. It writes nothing itself; the supervised child writes
only its result, report, and supervision attestation under `$RUN_DIR/outputs/`.

## Profiles

- `preflight` (CHK-001): common bindings only. Passes on the current tree;
  proves the harness works and guards every preserved binding, now including
  the accepted+integrated TASK-074 receipt.
- `compare-roots` (CHK-002, operation `compare`): common + real-root
  assertions (distinct symlink-free artifact trees, never the worktree root
  or git dir, `manifest.json` with recomputed digests, `required.json` with
  canonical `required_sha256`, scenario files under both roots with
  cross-consistent provenance, no `approved`) + nested identity/report/tool
  agreement + task-derived members. Fails on the current tree.
- `account-inventory` (CHK-003, operation `account-tests`): common + lifted
  template declarations + derived preparation register with exact five-key
  `{package, target, profile, source_commit, name}` source-derived required
  inventory + task-derived members. Fails on the current tree.
- `compare-supervision` (CHK-004, operation `compare` + `--supervise`):
  full `compare-roots` assertions, then executes the candidate source
  `runner/__main__.py` compare branch as a subprocess with no `TC_PROOF_*`
  environment, and asserts the branch returned (supervision attestation, no
  `execv`), emitted `outputs/CHK-004.result.json` via `finish()`
  (`tc-proof-runner-result/v1`, bound `run_id`/`operation`/`context_sha256`,
  coherent status/exit, non-empty digests), and left the comparison report
  at the nested report path. Fails on the current tree.
- `close` + `--gate` (CHK-005): own close context + full re-run of the
  `compare-roots` and `account-inventory` profiles + index/result integrity.
  Fails today. The gate never re-executes supervision: the supervised
  result is single-write and check ordering is not a contract.

## Common assertions (every profile)

Strict duplicate-key JSON; schema/task/check/run/operation identity;
40-hex source pins and 64-hex digests; allowed-keys schema mirror;
observer-sequence shape; adapter/source-authority bindings; axes/members
shape plus the task-derived (non-fixture-default) rule on worker paths;
qualification common trust (candidate tree, oracle, runtime outputs,
observer transport, comparator, tool bytes vs the real candidate
dispatcher); task-manifest phase/requirements/acceptance agreement with
`verify.toml`; accepted+integrated dependency receipts (071, 072, 074);
worktree HEAD/tree and scope-base ancestry via git; context-index
path/digest membership; preparation-result readiness and digest.

## Verdicts

Exit 0 prints `REPAIR-RESULT <CHK> status=passed`. Any failure prints one
`FAIL <scope> <field>: <detail>` line per violated assertion plus
`REPAIR-RESULT <CHK> status=rejected failures=N`, and exits 1.

## Non-goals

The driver does not execute the frozen dispatcher bundle, speak the
observer protocol, or replicate provider payload computations. Full worker
acceptance on TASK-002..007 remains those tasks' obligation; this task
proves the prepared bindings are genuine and runnable. End-to-end
`bin/tc-proof compare` supervision additionally needs a 072-owned rebundle
of the frozen dispatcher from the fixed source; that rebundle and the
TASK-002 re-run are an explicit follow-up, not this task's work product.
Frame/state byte equality between staged roots is the comparator's verdict,
not the driver's: the supervision profile requires status/exit coherence,
not a forced pass.
