# refactor-proof

TASK-001 proof workers are invoked by a verifier subagent through that task's
`taskfmt verify` checks. They are not a host lifecycle service and never start
or supervise a container.

## Current workflow

Use a host-local isolated subagent worktree and external run directory:

```sh
cargo nextest run -p refactor-proof --no-fail-fast

TASKFMT=/tmp/taskfmt-latest-install/bin/taskfmt
TASK_DIR=refactoring-tasks/terminal-components/completion/001
WORKTREE=/absolute/subagent-worktree
RUN_DIR=/absolute/external/task-001-run

"$TASKFMT" lint "$TASK_DIR"
"$TASKFMT" verify \
  --root "$WORKTREE" \
  --task-dir "$TASK_DIR" \
  --base "$SCOPE_BASE" \
  --progress "" \
  --log-dir "$RUN_DIR/taskfmt-logs"
```

`tc-proof` is a check worker. Its context, candidate outputs, and logs belong
to the verifier subagent's `RUN_DIR`. It must not write task packages, trusted
fixtures, refs, the frozen visual oracle, or another task's run directory.

The current taskfmt identity is `0.2.0` at
`afd3b575dbcc7044620bec4b9493a74eca3e5ef2`, built from
`/Users/donbeave/Projects/taskfmt/task-format`. Only standalone `taskfmt lint`
and `taskfmt verify` are allowed. Do not invoke `taskfmt init`, `status`,
`taskfmt-host`, `taskfmt-runtime`, `run`, `monitor`, or `promote`.

`tools/refactor-proof/scripts/sync-binaries.sh` is a retired fail-closed shim;
it must not overwrite the Python `bin/tc-proof` dispatcher with the Rust-only
comparator. `cargo nextest` validates the Rust package, but its test harness is
not the standalone comparator. Build that executable natively with
`scripts/campaign-build-proof.sh` before a verifier subagent runs proof checks.
The verifier must also supply immutable external contexts and the reviewed
observer/result environment; taskfmt does not provision either one.

Runner operations require the dispatcher environment and the native Rust
launcher. Invoking the Python bundle directly, or omitting the native launch
binding, fails closed. The inherited-pipe observer ABI requires an exact
request/response identity match, a successful exit, non-empty schema-valid
`payload` and `records`, and a result digest for every accepted observation.
The Rust preparation launcher never synthesizes an empty success event; until
an independent observer response provider is attached, a launch is rejected.

Comparator roots and report parents use their qualified real paths before
containment checks. This permits documented macOS `/var` temporary-directory
symlink ancestry while still rejecting symlink or hardlink descendants and
unsafe report leaves.

## Removed implementation

The former `tc-proof-host` install/prepare/freeze/verify/seal/integrate service
and its lifecycle code were removed. Historical task fixtures may mention that
service, but they are not executable campaign inputs. The coordinator relies
on verifier and reviewer subagent evidence, then integrates the reviewed task
commit.

The detailed proof invariants live in
[`docs/refactoring-plan/proof-contract.md`](../../docs/refactoring-plan/proof-contract.md).
