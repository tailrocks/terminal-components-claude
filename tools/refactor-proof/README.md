# Refactor-proof tools

This page describes the implementation in source snapshot [`7b2eb51`](https://github.com/tailrocks/terminal-components-claude/commit/7b2eb51c925a105f210c853e81b8ad440dad210f). It is an operational map, not permission to run an operation. Authority files, accepted receipts, scoped paths, frozen trees, and parent bindings remain separate prerequisites.

## Package map

The `refactor-proof` package has two binaries:

| Binary | Purpose | Source |
| --- | --- | --- |
| `tc-proof` | Comparator with the `compare --context PATH` interface. | [`tc_proof.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/bin/tc_proof.rs#L24-L83) |
| `tc-proof-host` | Host-side dispatch for the six operations below. | [`tc_proof_host.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/bin/tc_proof_host.rs#L23-L85), [`dispatch.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/dispatch.rs#L35-L171) |

## Host operations and effects

| Operation | Required arguments | Current behavior and effects |
| --- | --- | --- |
| `install` | `--receipt`, `--destination` | Validates authority, receipt, and executable digest; materializes the accepted host executable at `destination/bin/tc-proof-host`, sets mode `755`, and checks the written bytes. [`install.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/install.rs#L14-L47) |
| `prepare` | `--campaign`, `--task`, `--parent`, `--run` | Validates the campaign, accepted dependency receipts and ancestry, taskfmt pin, and task; invokes taskfmt progress initialization and writes run preparation evidence. [`prepare.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/prepare.rs#L35-L73) |
| `freeze` | `--run`, `--candidate` | Validates candidate scope, records its tree, and writes freeze and operation-context evidence in the run directory. [`freeze.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/freeze.rs#L46-L90) |
| `verify` | `--run` | Checks frozen context and progress, consumes the Observer sequence `build`, `test`, then `taskfmt`, validates/materializes observations, runs context checks, and writes a verdict. [`verify.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/verify.rs#L62-L140) |
| `seal` | `--run`, `--product` | The operation is recognized and checks the task's allowed products, but the current `seal_run` implementation then returns an integrity rejection. There is no successful seal path in this source snapshot. [`seal.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/seal.rs#L33-L47) |
| `integrate` | `--run`, `--ref`, `--expected-parent` | Checks authority, verdict, frozen tree, and expected parent; may register the run tree's alternate object directory; creates a commit and updates the ref with the expected old value, then writes a per-commit acceptance JSON record. The ref update and acceptance-file write are separate effects, not one atomic transaction. [`integrate.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/integrate.rs#L63-L125) |

The host's exact operation and argument dispatch is in [`dispatch.rs`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/src/host/dispatch.rs#L35-L171). The table describes source effects; it does not grant authority or establish that an operation was run successfully.

## Executable forms

The tracked `bin/tc-proof` and `bin/tc-proof-host` entries are development shell wrappers. `scripts/sync-binaries.sh` takes `PROFILE` (default `debug`), copies both executables from `workspace/target/$PROFILE` into those tracked paths, and sets each destination to mode `755`. It performs the copies sequentially, so the pair is not installed atomically. The script assumes that physical target layout and does not itself establish compatibility with the current managed build-target path. It changes tracked files; inspect the exact diff before any later reconciliation. Do not restore a path blindly over other work. See the [wrapper files](https://github.com/tailrocks/terminal-components-claude/tree/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/bin) and [`sync-binaries.sh`](https://github.com/tailrocks/terminal-components-claude/blob/7b2eb51c925a105f210c853e81b8ad440dad210f/tools/refactor-proof/scripts/sync-binaries.sh#L8-L24).

## Observer boundary

The detailed [Observer transport note](observer-transport.md) distinguishes the current client interface, original design intent, a historical alternate launcher ABI, and a synthetic fixture. The design intent assigns the pipe endpoints to a planner-owned producer and describes the ordered checks, but the note does not identify that producer in the reviewed current tree. Protocol tests or a fixture do not establish production descriptor ownership, inheritance behavior, or safe repeated construction. Do not infer those guarantees from the environment-variable names or from this overview.

The tooling supports repository qualification. It does not prove component API completeness or product parity. See the maintained [architecture](../../docs/architecture.md), [component contracts](../../docs/components-contracts.md), and [plan](../../docs/plan.md) for their separate scopes.
