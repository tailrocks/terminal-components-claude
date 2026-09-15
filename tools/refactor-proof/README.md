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

## Observer IPC (Phase 2 skeleton)

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

Rust types live in `refactor_proof::observer` (`ObserverClient`, `ObserverRequest`, `ObserverObservation`). Transport over inherited FDs is Phase 3; Phase 2 provides compile-only stubs and request encoding.
