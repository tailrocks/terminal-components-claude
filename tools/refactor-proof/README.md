# refactor-proof

TASK-001 deliverable: independently qualified `tc-proof` comparator and `tc-proof-host` core for product `qualified-harness`.

Command interface and ownership boundaries are defined in [`docs/refactoring-plan/proof-contract.md`](../../docs/refactoring-plan/proof-contract.md). This crate does not constitute qualification evidence until CHK-004/005/006/007 pass against the pinned bootstrap drivers.

Build:

```sh
cargo build -p refactor-proof
```

Installed entrypoints for qualification:

- `bin/tc-proof` — wrapper to the workspace-built comparator
- `bin/tc-proof-host` — wrapper to the workspace-built host

## Host operations (Phase 3a)

`install` and `prepare` are implemented against the host bootstrap protocol:

- `install --receipt PATH --destination PATH` validates `TC_PROOF_AUTHORITY_FILE`, checks receipt digests against the authority allowlist, verifies harness executable bytes, and writes `bin/tc-proof-host` mode `0755`.
- `prepare --campaign DIR --task ID --parent COMMIT --run DIR` validates campaign/receipt bindings, dependency producer/product tuples, predecessor ancestry, pinned taskfmt identity, runs `taskfmt progress-init`, and writes `run/preparation.json`.

`verify` runs observer IPC (build/test/taskfmt), validates the frozen context index, materializes worker/taskfmt logs, and writes `run/verdict.json` with five `CHK-*` entries. Observer unavailable → `rejected` / `integrity` (no panic). `seal` and `integrate` still emit `status: "rejected"` / `category: "unsupported"`.

Qualification driver filtering: `host-bootstrap-driver.py` has no `--cases` filter. Phase 3a vectors exercised directly:

| Vector | Result |
| --- | --- |
| `forged_install_receipt` | pass |
| `forged_predecessor_receipt` | pass |
| `wrong_predecessor` | pass |
| `unintegrated_predecessor` | pass |
| positive install+prepare | blocked locally — installed `taskfmt` fingerprint differs from pinned bootstrap (`52c960db…`) |

Full driver `--host` invocation additionally requires the pinned taskfmt executable before any host case runs.

## Observer IPC (Phase 3c transport)

During `verify`, the host cannot self-attest worker or taskfmt execution. The planner-owned observer (outside the host sandbox) supplies three host-only environment values:

| Variable | Role |
| --- | --- |
| `TC_PROOF_OBSERVER_REQUEST_FD` | Write side of anonymous request pipe |
| `TC_PROOF_OBSERVER_RESPONSE_FD` | Read side of anonymous response pipe |
| `TC_PROOF_OBSERVER_NONCE` | Unpredictable per-fixture nonce |

Worker subprocesses inherit none of these. The host writes one newline-terminated JSON request per step and reads one newline-terminated JSON response. Requests are bounded to 4096 bytes, one-shot, and must appear in this exact order:

1. `{"schema":"tc-proof-observer-request/v1","nonce":"<nonce>","step":"build"}`
2. `{"schema":"tc-proof-observer-request/v1","nonce":"<nonce>","step":"test"}`
3. `{"schema":"tc-proof-observer-request/v1","nonce":"<nonce>","step":"taskfmt"}`

Success responses use `tc-proof-observation/v1` with `step`, frozen `tree`, actual `argv`, integer `exit`, base64 `stdout`/`stderr`, and `files` (relative output names to base64 bytes). Protocol violations return `tc-proof-observer-error/v1`. Reordering, replay, wrong nonce, or extra keys fail closed.

Rust types live in `refactor_proof::observer` (`ObserverClient`, `ObserverRequest`, `ObserverObservation`). `ObserverClient::from_env()` reads inherited FDs and performs newline-framed JSON request/response exchange for the fixed build → test → taskfmt sequence.
