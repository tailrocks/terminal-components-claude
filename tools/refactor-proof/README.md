# refactor-proof

TASK-001 deliverable: independently qualified `tc-proof` comparator and `tc-proof-host` core for product `qualified-harness`.

Command interface and ownership boundaries are defined in [`docs/refactoring-plan/proof-contract.md`](../../docs/refactoring-plan/proof-contract.md). This crate does not constitute qualification evidence until CHK-004/005/006/007 pass against the pinned bootstrap drivers.

Build and sync qualification entrypoints:

```sh
cargo build -p refactor-proof
tools/refactor-proof/scripts/sync-binaries.sh
```

`verify.toml` and `host-bootstrap-driver.py --host` use `tools/refactor-proof/bin/tc-proof-host`. The bootstrap driver requires that path to be a regular Mach-O executable so `install` reproduces the accepted harness bytes (Darwin sandbox may allow the submitted argv[0] but deny reads under `target/debug/`). After `sync-binaries.sh`, `bin/tc-proof-host` and `bin/tc-proof` are copies of the workspace-built binaries. Development wrappers that exec `target/debug/` directly live under `scripts/dev-tc-proof-host.sh` and `scripts/dev-tc-proof.sh`.

## Host operations (Phase 3a)

`install` and `prepare` are implemented against the host bootstrap protocol:

- `install --receipt PATH --destination PATH` validates `TC_PROOF_AUTHORITY_FILE`, checks receipt digests against the authority allowlist, verifies harness executable bytes, and writes `bin/tc-proof-host` mode `0755`.
- `prepare --campaign DIR --task ID --parent COMMIT --run DIR` validates campaign/receipt bindings, dependency producer/product tuples, predecessor ancestry, pinned taskfmt identity, runs `taskfmt progress-init`, and writes `run/preparation.json`.

`verify` runs observer IPC (build/test/taskfmt), validates the frozen context index, materializes worker/taskfmt logs, and writes `run/verdict.json` with five `CHK-*` entries. Observer unavailable → `rejected` / `integrity` (no panic).

`seal --run DIR --product NAME` rejects fixture tasks with empty `seal_products` (`rejected` / `authority`).

`integrate --run DIR --ref REF --expected-parent COMMIT` validates the configured integration ref, compares the expected parent against preparation and the current ref (CAS), and on success appends a `tc-proof-host-acceptance/v1` ledger record. Wrong ref → `authority`; wrong or stale parent → `parent`.

Phase 3b freeze vectors (`tools/refactor-proof/scripts/test_freeze_vectors.py` via `HostFixture`):

| Vector | Category | Result |
| --- | --- | --- |
| `exact_tested_tree` (freeze) | passed | pass |
| `out_of_scope` | scope | pass |
| `forbidden_checker` | scope | pass |
| `overlay_tamper` | scope | pass |
| `symlink_escape` | unsafe-path | pass |
| `hardlink_escape` | unsafe-path | pass |
| `ignored_source` | unsafe-git | pass |
| `hidden_index_flag` | unsafe-git | pass |
| `changed_git_config` | unsafe-git | pass |
| `submodule_substitution` | unsafe-git | pass |

Phase 3d integrate/seal vectors (`tools/refactor-proof/scripts/test_integrate_seal_vectors.py` via `HostFixture`):

| Vector | Category | Result |
| --- | --- | --- |
| `premature_seal` | authority | pass |
| `unauthorized_seal` | authority | pass |
| `wrong_expected_parent` | parent | pass |
| `stale_parent_cas` | parent | pass |
| `wrong_integration_ref` | authority | pass |

Run the full host matrix with the synced bin path (63 invocations):

```sh
python3 refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --host tools/refactor-proof/bin/tc-proof-host \
  --taskfmt /path/to/pinned/taskfmt \
  --taskfmt-source /path/to/pinned/task-format
```

Requires macOS, pinned taskfmt revision `52d9f1eb…` / fingerprint `52c960db…`, and synced Mach-O entrypoints.

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
