# TASK-001 bootstrap plan

**Date:** 2026-09-15  
**Task:** TASK-001 — Implement independently qualified comparator and host core  
**Product:** `qualified-harness`  
**Architectural base:** `7b27732a8c3c131760ec3438f641cb3c11343a42`  
**Catalog tip (planning):** `prep-wave1-verify` @ `a426fd49` in [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md)  
**Worktree branch:** `task-001-bootstrap`  
**Worktree path:** `.worktrees/main` (repo-relative) → `/Users/donbeave/Projects/terminal-components-claude/.worktrees/main`

---

## Scope assessment

**Verdict: LARGE — defer full implementation; use phased bootstrap below.**

TASK-001 is not a scaffold-only task. It requires two independently qualified executables under the sole writable path `tools/refactor-proof/`:

| Deliverable | Path | Qualification gate |
| --- | --- | --- |
| Comparator | `tools/refactor-proof/bin/tc-proof` | CHK-004: 72 vectors + 69 fresh positive recoveries (141 invocations) |
| Host core | `tools/refactor-proof/bin/tc-proof-host` | CHK-005/006/007: install/prepare/freeze/verify/seal-reject/integrate, hostile workers, observer IPC, standalone taskfmt |

**Current state on worktree `task-001-bootstrap` (`cf2e79a0`, parent `7b27732a`):**

| Phase | Status | Worktree commit |
| --- | --- | --- |
| 0 — scaffold | **Done** | `c06e7747` |
| 1 — comparator (CHK-004) | **Done** — 141/141 driver invocations | `79807bb3` |
| 2 — observer IPC skeleton | **Done** (compile-only stubs) | `9cff8e7e` |
| 3a — install/prepare | **Done** | `abee282d` |
| 3b — freeze | **Done** — 10/10 bootstrap vectors | `7158db56` |
| 3c — verify skeleton | **Done** (observer IPC transport + verdict writer) | `a17124e2` |
| 3d — seal/integrate negatives | **Done** (authority/parent reject vectors) | `1ef1f3f6` |
| 3 — host matrix (CHK-005/006/007) | **Complete** — 63/63 driver invocations + architecture exemption | `cf2e79a0` |
| 4 — production receipt | **Pending** — IW-03 operator checklist required | — |

- `tools/refactor-proof/` — **present** (Phase 0–3d; comparator + host six-op skeleton)
- `tools/refactor-proof/bin/tc-proof` — **present** (Mach-O after `sync-binaries.sh`; dev wrapper at `scripts/dev-tc-proof.sh`)
- `tools/refactor-proof/bin/tc-proof-host` — **present** (Mach-O after `sync-binaries.sh`; six operations implemented; dev wrapper at `scripts/dev-tc-proof-host.sh`)
- `tools/refactor-proof/README.md` — **operational docs only** (build/sync/run commands); **not** qualification evidence — see [`task-001-qualification-report.md`](task-001-qualification-report.md) and operator evidence draft
- `cargo check -p refactor-proof` — **passes** (verified 2026-09-15)
- `cargo nextest run -p refactor-proof` — **passes** — 17/17 (verified 2026-09-15 @ `cf2e79a0`)
- `architecture-exemption.json` — **present** @ `cf2e79a0` — registers `tc-proof`/`tc-proof-host` for `binary_names_are_preserved`; xtask `capture_matrix_contract` tolerates worktree-local capture provenance

Comparator qualification (CHK-004) passes locally against pinned tuisnap — 141/141 driver invocations. Host install/prepare/freeze/verify/seal/integrate pass full `host-bootstrap-driver.py --host` matrix — 63/63 invocations via synced `bin/tc-proof-host` @ `782adc03`. Phase 3 is **complete** @ `cf2e79a0` (architecture test fix); first production `tc-proof-host-receipt/v1` still requires IW-03 operator checklist (Phase 4). Qualification evidence lives in planning docs and bootstrap driver output — not in the crate README. All qualification drivers and vectors are frozen in the TASK-001 package at `refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/` (planning branch only; not writable during execution).

---

## Constraints (mandatory)

| Rule | Detail |
| --- | --- |
| Writable scope | `tools/refactor-proof/**` only ([`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml)) |
| Production edits | Architectural-main worktree only — never on `visual-baseline` tag branch |
| Tag immutability | Never move/retarget/recreate `visual-baseline` or its release |
| Main integration | No merge/push to `main` without explicit operator authorization |
| Tests | `cargo nextest` only — never `cargo test` |
| Campaign | Do **not** arm `/goal` until operator explicitly authorizes (issuance ≠ arming) |
| Bootstrap exception | First host receipt requires IW-03 operator checklist ([`campaign-executor-protocol.md`](campaign-executor-protocol.md)) |

---

## Minimal bootstrap scaffold (Phase 0 — first worktree commit)

Goal: establish compile-time structure without claiming qualification. This is the smallest useful first commit on `task-001-bootstrap`.

```text
tools/refactor-proof/
  Cargo.toml                 # workspace member; bins tc-proof, tc-proof-host
  README.md                  # operational build/sync/run docs; not qualification evidence (see task-001-qualification-report.md)
  src/
    lib.rs                   # shared JSON schemas, canonical serialization helpers
    bin/
      tc_proof.rs            # compare-only stub for Phase 1 focus
      tc_proof_host.rs       # six-operation CLI dispatch stub
  crates/                    # optional split if lib.rs grows past ~800 lines
    comparator/              # tc-proof-compare-context/v1, tc-proof-comparison/v1
    host/                    # host operations, observer IPC client
    common/                  # canonical JSON, SHA-256, path safety
```

**Phase 0 acceptance (local, advisory):**

- `cargo check -p refactor-proof` exits 0 in worktree
- Both binaries exist at `target/debug/tc-proof` and `target/debug/tc-proof-host`
- `tc-proof-host --help` lists exactly six operations from proof-contract (install, prepare, freeze, verify, seal, integrate)
- Every invocation emits one `tc-proof-host-result/v1` JSON object on stdout (stub `rejected` / `unsupported` is fine)
- `tc-proof compare --context PATH` emits `tc-proof-comparison/v1` at `report_path` (stub reject is fine)
- Root `Cargo.toml` workspace `members` includes `"tools/refactor-proof"`

**Phase 0 must NOT:**

- Pass any bootstrap driver check
- Edit task packages, planning docs, or snapshots
- Create receipts or integrate refs

---

## Phased implementation plan

### Phase 1 — Comparator (`tc-proof compare`)

**Check:** CHK-004  
**Protocol:** [`proof-comparator-protocol.md`](evidence/proof-comparator-protocol.md)  
**Driver:** `001/trusted/proof-bootstrap/proof-comparator-bootstrap.py --runner …/tc-proof`

Implement in order:

1. Context parser (`tc-proof-compare-context/v1`) with field validation and `context_sha256` binding
2. Manifest validator (`tc-proof-artifacts/v1`) — path safety, symlink rejection, size/hash checks
3. Required-set authority (`required.json` vs context `required_ids` / `required_count`)
4. Frame validation via pinned tuisnap (`Frame::validate`, schema 3, `diff_cells`)
5. Semantic exact equality per scenario sidecar keys
6. Provenance binding (`tc-proof-provenance/v1`) — separate oracle vs candidate rules
7. Result writer (`tc-proof-comparison/v1`) with exact failure codes and complete-set semantics
8. Recovery-positive path after every negative (141 total driver invocations)

**Exit criterion:** CHK-004 exit 0 against pinned tuisnap executable.

### Phase 2 — Host observer adapter

**Checks:** prerequisite for CHK-005/006/007  
**Protocol:** [`host-bootstrap-protocol.md`](evidence/host-bootstrap-protocol.md) § Independently owned execution authority

Implement observer IPC client:

- Read `TC_PROOF_OBSERVER_REQUEST_FD`, `TC_PROOF_OBSERVER_RESPONSE_FD`, `TC_PROOF_OBSERVER_NONCE`
- Emit ordered requests: `build`, `test`, `taskfmt` (one-shot, 4096-byte bound, newline-terminated JSON)
- Parse `tc-proof-observation/v1` responses; copy worker outputs to run layout
- Never expose observer FDs to worker subprocesses

Darwin sandbox profile is **observer-owned** (Python); host implements the adapter only.

### Phase 3 — Host operations

**Checks:** CHK-005 (regression), CHK-006 (lint/boundary), CHK-007 (gate)

Implement six-operation CLI per proof-contract:

| Operation | Primary artifacts |
| --- | --- |
| `install` | `bin/tc-proof-host` exact bytes from receipt |
| `prepare` | `run/preparation.json`, `run/progress.md` via taskfmt progress-init |
| `freeze` | `run/freeze.json`, `context-index.json`, `contexts/CHK-*.json` |
| `verify` | observer-driven workers + taskfmt + `run/verdict.json` |
| `seal` | reject fixture (empty `seal_products`) |
| `integrate` | CAS ref update, DCO commit, ledger append |

Key invariants:

- `TC_PROOF_AUTHORITY_FILE` at host entrypoint only — never in workers
- Host-owned Git object database for freeze (never trust executor index)
- Protected verdict bytes in observer-private `accepted-proof/` root
- Every rejection preserves protected state; fresh positive follows each negative ([`host-bootstrap-vectors.json`](evidence/host-bootstrap-vectors.json))

**Exit criterion:** CHK-001 precondition, CHK-005, CHK-006, CHK-007 all exit 0.

### Phase 4 — Operator bootstrap and receipt (IW-03)

After all verify.toml checks pass in the container/host environment:

1. Complete [`campaign-executor-protocol.md`](campaign-executor-protocol.md) IW-03 checklist (`tc-proof-operator-bootstrap-checklist/v1`)
2. Rebuild host from accepted source tree; record executable SHA-256
3. Issue first `tc-proof-host-receipt/v1` for product `qualified-harness`
4. Coordinator sign-off at SO-001 through SO-005 gates

This phase is **operator/coordinator** work, not executor Rust edits.

---

## Verification commands

### Planning branch (prep-wave1-verify)

```sh
python3 docs/refactoring-plan/evidence/validate-plan.py --summary
```

### Worktree (task-001-bootstrap)

```sh
cd .worktrees/main
cargo check -p refactor-proof          # Phase 0+
cargo nextest run -p refactor-proof    # after tests exist; never cargo test
```

### TASK-001 package checks (container / operator environment)

Full CI/container layout, build sync, tui-snap mount, taskfmt install, Mach-O vs wrapper, and Darwin sandbox requirements: [`task-001-ci-requirements.md`](task-001-ci-requirements.md).

Paths use `/task/`, `/work/`, `/proof/bootstrap/` as in verify.toml:

```sh
# Precondition (no host yet — driver self-test + taskfmt pins)
python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format

# Comparator qualification
python3 /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
  --runner /work/tools/refactor-proof/bin/tc-proof

# Host qualification (full matrix)
python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --host /work/tools/refactor-proof/bin/tc-proof-host \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format
```

Final task gate: `taskfmt verify` from `/work` with nonempty progress (campaign executor protocol; not `--progress ""`).

---

## Dependency pins

| Input | Pin |
| --- | --- |
| Architectural main | `7b27732a8c3c131760ec3438f641cb3c11343a42` |
| UI oracle commit | `02f5294bfdbf38004cc49130d0aff1d01f31434c` |
| taskfmt revision | `52d9f1eb7721f409bc47beb9fced7997b5c13ede` |
| taskfmt fingerprint | `52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4` |
| Host isolation | macOS + `/usr/bin/sandbox-exec` + CLT git (INT-03 accepted at planning boundary) |
| tuisnap | Qualified checkout per [`tuisnap-review.md`](tuisnap-review.md) — required for comparator frame validation |

---

## Phase completion (2026-09-15)

### Phase 0 — scaffold @ `c06e7747`

| Item | Status |
| --- | --- |
| `tools/refactor-proof/` workspace member + stub bins | **Done** |
| `cargo check -p refactor-proof` | **Pass** |

### Phase 1 — comparator @ `79807bb3`

| Item | Status |
| --- | --- |
| `tc-proof compare` implementation | **Done** |
| CHK-004 bootstrap driver | **Pass** — 72 vectors + 69 fresh positive recoveries (141/141) |

### Phase 2 — observer IPC skeleton @ `9cff8e7e`

| Item | Status |
| --- | --- |
| Host operation types + result schema | **Done** (compile-only) |
| Observer IPC client stubs (`build`/`test`/`taskfmt`) | **Done** (compile-only; FD transport wired in Phase 3c) |

### Phase 3a — install/prepare @ `abee282d`

| Item | Status |
| --- | --- |
| `install --receipt --destination` | **Done** — authority allowlist + harness byte verification |
| `prepare --campaign --task --parent --run` | **Done** — taskfmt `progress-init` + `run/preparation.json` |

### Phase 3b — freeze @ `7158db56`

| Item | Status |
| --- | --- |
| Host-owned Git object database freeze | **Done** |
| Bootstrap freeze vectors | **Pass** — 10/10 (`exact_tested_tree`, scope, unsafe-path, unsafe-git) |

### Phase 3c — verify skeleton @ `a17124e2`

| Item | Status |
| --- | --- |
| Observer IPC transport (`ObserverClient`) | **Done** — build → test → taskfmt sequence |
| `verify` verdict writer | **Done** (skeleton; observer unavailable → `rejected`/`integrity`) |

### Phase 3d — seal/integrate negatives @ `1ef1f3f6`

| Item | Status |
| --- | --- |
| `seal` empty `seal_products` reject | **Done** — `premature_seal`, `unauthorized_seal` vectors |
| `integrate` CAS + ref guards | **Done** — `wrong_expected_parent`, `stale_parent_cas`, `wrong_integration_ref` vectors |

### Phase 3 — host matrix @ `cf2e79a0` (complete)

| Item | Status |
| --- | --- |
| CHK-004 comparator driver | **Pass** — 141/141 |
| CHK-005/006/007 host driver | **Pass** — 63/63 full matrix |
| `cargo nextest run -p refactor-proof` | **Pass** — 17/17 |
| Architecture exemption (`architecture-exemption.json` + xtask capture tolerance) | **Done** @ `cf2e79a0` |

## What remains deferred

| Item | Reason |
| --- | --- |
| Operator bootstrap and first production receipt | Phase 4 — IW-03 operator checklist (`tc-proof-operator-bootstrap-checklist/v1`) required before any first receipt claim |
| Campaign `/goal` arming | Explicitly out of scope per operator authorization rules |

---

## Next steps for Alexey

1. **Confirm worktree:** `cd /Users/donbeave/Projects/terminal-components-claude/.worktrees/main` — branch `task-001-bootstrap` @ `cf2e79a0`.
2. ~~**Phase 0 scaffold:**~~ **Done** @ `c06e7747`.
3. ~~**Phase 1 comparator:**~~ **Done** @ `79807bb3` — CHK-004 141/141.
4. ~~**Phase 2 observer IPC skeleton:**~~ **Done** @ `9cff8e7e`.
5. ~~**Phase 3a install/prepare:**~~ **Done** @ `abee282d`.
6. ~~**Phase 3b freeze:**~~ **Done** @ `7158db56` — 10/10 vectors.
7. ~~**Phase 3c verify skeleton:**~~ **Done** @ `a17124e2`.
8. ~~**Phase 3d seal/integrate negatives:**~~ **Done** @ `1ef1f3f6`.
9. ~~**Phase 3 host matrix:**~~ **Complete** @ `cf2e79a0` — CHK-004 141/141, host driver 63/63 (synced `bin/tc-proof-host`), nextest 17/17, architecture exemption.
10. **Operator gate (Phase 4):** Complete IW-03 checklist before any first production receipt claim.
11. **Integration:** Submit to operator for `taskfmt verify` + host freeze/verify; integrate only via `tc-proof-host integrate` to named ref (never `refs/heads/main` in qualification fixtures).
12. **Planning branch:** Merge task-001 completion evidence back to planning docs only via separate authorized PR — not in `writable_paths`.

---

## Related documents

- [`task-001-ci-requirements.md`](task-001-ci-requirements.md) — CI/container qualification requirements
- [`proof-contract.md`](proof-contract.md) — command interface and ownership
- [`001/README.md`](../../refactoring-tasks/terminal-components/completion/001/README.md) — task contract
- [`001/verify.toml`](../../refactoring-tasks/terminal-components/completion/001/verify.toml) — machine checks
- [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) — issuance and arm procedure
- [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — IW-03 bootstrap exception
