# Refactor proof tools

The `refactor-proof` workspace package builds the `tc-proof` comparator and `tc-proof-host` host-only protocol executable. Build and test it with:

```sh
cargo build -p refactor-proof --offline
cargo test -p refactor-proof --offline
```

The `tc-proof` CLI accepts `compare --context PATH`. The host CLI dispatches six operations: `install`, `prepare`, `freeze`, `verify`, `seal`, and `integrate`. Committed `bin/*` scripts are development wrappers for `target/debug/*`; `scripts/sync-binaries.sh` replaces them with built binaries for harnesses that require native executables. Running that sync changes the working tree; restore the wrappers after use when needed.

`install` validates authority and executable digests before writing the host executable. `prepare` validates campaign, receipt, dependency, and ancestry bindings and writes preparation evidence. `freeze` binds the candidate tree and operation contexts. `verify` delegates build/test/taskfmt work to the external observer and writes a verdict. `seal` rejects non-baseline tasks. `integrate` performs a compare-and-swap check against the prepared parent and configured ref before appending an acceptance record.

During `verify`, the planner-owned observer supplies `TC_PROOF_OBSERVER_REQUEST_FD`, `TC_PROOF_OBSERVER_RESPONSE_FD`, and `TC_PROOF_OBSERVER_NONCE` to the host only. The host sends one bounded newline-terminated JSON request for each ordered step: `build`, `test`, then `taskfmt`. The response binds the step, frozen tree, actual argv, exit code, captured stdout/stderr, and output files. Reordered/replayed requests, wrong nonces, extra keys, oversized messages, observer absence, and protocol violations fail closed. Worker subprocesses do not inherit observer variables.

These tools support repository qualification; they do not establish component API completeness or product parity. Product architecture and remaining work are described in [architecture](../../docs/architecture.md), [component contracts](../../COMPONENT_ARCHITECTURE.md), and [plan](../../docs/plan.md).
