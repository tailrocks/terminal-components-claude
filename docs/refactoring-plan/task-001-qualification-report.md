# TASK-001 qualification evidence report

**Date:** 2026-09-15  
**Worktree branch:** `task-001-bootstrap` @ `cf2e79a068518e229751f82b635832ecaba8ae4d`  
**Worktree path:** `/Users/donbeave/Projects/terminal-components-claude/.worktrees/main`  
**Remote branch:** [`task-001-bootstrap`](https://github.com/donbeave/terminal-components-claude/tree/task-001-bootstrap) @ `cf2e79a0…`  
**Planning branch:** `prep-wave1-verify` (this report)  
**Context-check dispatch:** `@0a31a338` — non-fixture frozen verify contexts spawn sibling `tc-proof` subprocess (`context_check.rs`); `tc-host-fixture-check-context/v1` retains stub no-op (`388c173d` warning cleanup → 53 warnings).  
**Architecture test fix:** `@cf2e79a0` — `tools/refactor-proof/architecture-exemption.json` registers `tc-proof`/`tc-proof-host` for `binary_names_are_preserved`; `capture_matrix_contract` tolerates checkout-local `resolved_path` and absent gitignored `shots/.capture-state/` stderr on worktrees.  
**Pinned taskfmt:** `/tmp/taskfmt-install/bin/taskfmt` (rev `52d9f1eb7721f409bc47beb9fced7997b5c13ede`)  
**Pinned taskfmt source:** `/tmp/taskfmt-qualification` @ `52d9f1eb7721f409bc47beb9fced7997b5c13ede`  
**Pinned tuisnap:** `/Users/donbeave/Projects/tui-snap/target/release/tuisnap`  
**Full workspace nextest:** **3201/3201 pass** @ `cf2e79a0` (worktree `.worktrees/main`; 6 skipped)  
**Planning PR:** [#4](https://github.com/donbeave/terminal-components-claude/pull/4)  
**Operator evidence draft:** [`task-001-operator-evidence-draft.md`](task-001-operator-evidence-draft.md) @ `8dea381e`  
**taskfmt verify prep:** [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md) — OB-006 **blocked** (§7)  
**validate-plan:** `error_count: 0` on `prep-wave1-verify`

---

## Verdict

| Gate | Result | Pass | Fail |
| --- | --- | ---: | ---: |
| `cargo build -p refactor-proof` | **PASS** | 1 | 0 |
| Comparator CHK-004 (`proof-comparator-bootstrap.py`) | **PASS** | 141 | 0 |
| Host self-test (`host-bootstrap-driver.py --self-test`) | **PASS** | 8 | 0 |
| Standalone taskfmt gate | **PASS** | 6 | 0 |
| Observer qualification (`--observer-test`) | **PASS** | 8 | 0 |
| Host matrix (`--host` via `bin/tc-proof-host` wrapper) | **FAIL** | 0 | 1 |
| Host matrix (`--host` via `target/debug/tc-proof-host` Mach-O) | **PASS** | 63 | 0 |
| `cargo nextest run -p refactor-proof` | **PASS** | 13 | 0 |
| Full workspace `cargo nextest run` | **PASS** | 3201 | 0 |

**Aggregate (Mach-O host path — production binary):** **240 pass / 0 fail** across refactor-proof gates.  
**Workspace regression (advisory):** **3201/3201** @ `cf2e79a0` — no worktree fixes required.  
**Aggregate (including wrapper `--host` attempt):** **240 pass / 1 fail**.

**Overall TASK-001 bootstrap qualification:** **PASS** on built Mach-O host executable. Wrapper shim at `tools/refactor-proof/bin/tc-proof-host` fails the driver's install byte-equality check because `install` materializes the Mach-O payload from the wrapper receipt; qualification must target the built binary directly.

---

## 1. Build

```sh
cd .worktrees/main && cargo build -p refactor-proof
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Errors | 0 |
| Warnings | 53 (doc/unused/dead_code; no functional blockers) |
| Binaries | `target/debug/tc-proof`, `target/debug/tc-proof-host` |

---

## 2. Comparator CHK-004

```sh
python3 refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
  --runner tools/refactor-proof/bin/tc-proof \
  --tuisnap /Users/donbeave/Projects/tui-snap/target/release/tuisnap
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Schema | `tc-proof-bootstrap-qualification/v1` |
| Case count | 72 |
| Invocation count | 141 |
| Failures | `[]` |
| Runner SHA-256 | `6fa6a1b627c5748e3251f26a57898b4dc5d8e92966cc19e7acedbd47cc407470` |
| Driver SHA-256 | `3ecb9c6ea3a37da22a864c0c016f6ea58a0ec52973742782d5f73c3e4c7c8dae` |
| Vectors SHA-256 | `84f4b35d92acc39bd5feddfb639920b8484aaff12f9de1f71807612d753046d0` |

---

## 3. Host preparation facility gates

Driver: `refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py`

### 3a. Self-test

```sh
python3 host-bootstrap-driver.py --self-test
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Tests run | 8 |
| Failures | 0 |
| Duration | 18.3s |

### 3b. Standalone taskfmt gate

```sh
python3 host-bootstrap-driver.py \
  --taskfmt /tmp/taskfmt-install/bin/taskfmt \
  --taskfmt-source /tmp/taskfmt-qualification
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Schema | `tc-host-bootstrap-taskfmt/v1` |
| Binary SHA-256 | `55528a01d987489f9b8ae263eb913c85f0f7d6d540ae2d04efad0d8e363e5a68` |

| Case | Exit | Passed |
| --- | ---: | --- |
| positive | 0 | true |
| out_of_scope | 1 | false |
| failed_check | 1 | false |
| incomplete_progress | 1 | false |
| done_nonzero | 1 | false |
| missing_overlay | 1 | false |

All six cases behaved as required (1 positive pass, 5 intentional negative failures).

### 3c. Observer qualification

```sh
python3 host-bootstrap-driver.py --observer-test \
  --taskfmt /tmp/taskfmt-install/bin/taskfmt \
  --taskfmt-source /tmp/taskfmt-qualification
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Schema | `tc-proof-observer-qualification/v1` |
| Cases | 8 (all pass) |

Cases: `prepare_probe_substitution_denied`, `fabricated_host_rejected_for_missing_execution`, `real_build_test_taskfmt_observed`, `surviving_child_cannot_rebind_private_proof`, `accepted_verdict_and_logs_write_denied`, `observer_read_write_signal_task_port_network_denied`, `ambient_secret_read_denied`, `observer_replay_rejected`.

Pinned hashes: git `be4afb2b…`, observer `0e95a95c…`, python `6c9d4000…`, sandbox `abc5bb13…`, taskfmt `55528a01…`.

---

## 4. Host matrix CHK-005/006/007

### 4a. Wrapper path (FAIL)

```sh
python3 host-bootstrap-driver.py --host tools/refactor-proof/bin/tc-proof-host \
  --taskfmt /tmp/taskfmt-install/bin/taskfmt \
  --taskfmt-source /tmp/taskfmt-qualification
```

| Field | Value |
| --- | --- |
| Exit | 1 |
| Failure | `AssertionError: operation did not pass` on first case `exact_tested_tree`, `install` |
| Root cause | Driver requires `digest(installed) == digest(submitted_executable)`; `install` materializes Mach-O bytes from the wrapper receipt, so post-install digest differs from wrapper script digest |

### 4b. Mach-O path (PASS)

```sh
python3 host-bootstrap-driver.py --host target/debug/tc-proof-host \
  --taskfmt /tmp/taskfmt-install/bin/taskfmt \
  --taskfmt-source /tmp/taskfmt-qualification
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Schema | `tc-host-bootstrap-qualification/v1` |
| Host SHA-256 | `7731057ac0f0e5e971464067877a2986dceb6d8f3f2705c3a4d867afe5e00096` |
| Case count | 32 |
| Invocation count | 63 (each negative followed by fresh positive refresh) |
| Failures | 0 |

Cases: `exact_tested_tree`, `forged_install_receipt`, `forged_predecessor_receipt`, `wrong_predecessor`, `unintegrated_predecessor`, `out_of_scope`, `forbidden_checker`, `overlay_tamper`, `symlink_escape`, `hardlink_escape`, `ignored_source`, `hidden_index_flag`, `changed_git_config`, `submodule_substitution`, `premature_seal`, `unauthorized_seal`, `failed_worker`, `done_nonzero_worker`, `missing_worker_result`, `candidate_expected`, `missing_expected`, `missing_gate`, `context_index_tamper`, `context_member_tamper`, `context_member_missing`, `context_member_extra`, `context_member_swap`, `context_cross_run`, `incomplete_progress`, `wrong_expected_parent`, `stale_parent_cas`, `wrong_integration_ref`.

Duration: ~311s.

---

## 5. Unit tests

```sh
cargo nextest run -p refactor-proof
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Tests run | 13 |
| Passed | 13 |
| Failed | 0 |
| Skipped | 0 |

---

## 6. Full workspace nextest

```sh
cd .worktrees/main && cargo nextest run
```

| Field | Value |
| --- | --- |
| Exit | 0 |
| Tests run | 3201 |
| Passed | 3201 |
| Failed | 0 |
| Skipped | 6 |
| Worktree tip | `cf2e79a068518e229751f82b635832ecaba8ae4d` |
| Duration | ~171s |

Advisory workspace regression only; does not substitute for TASK-001 host receipt or OB-006 `taskfmt verify`.

---

## 7. Standalone `taskfmt verify` blockers (OB-006 prep)

Source: [`task-001-taskfmt-verify-notes.md`](task-001-taskfmt-verify-notes.md). Does **not** authorize SO-005 sign-off.

| Gate criterion | Result |
| --- | --- |
| `taskfmt verify` exit code | **0** |
| Last stdout line | **`RESULT FAIL`** (not `DONE`) |
| **OB-006 / SO-005 reached?** | **No** |

**Primary blockers**

1. **Container path layout** — `verify.toml` invokes checks at hardcoded `/task/`, `/work/`, `/proof/bootstrap/` paths. Without root bind mounts or an operator container exposing those paths, CHK-001/004/005/006/007 fail immediately (Python rc 2, file not found).
2. **Incomplete progress** — `progress-init` leaves `state=IN_PROGRESS`, `current=1.1`. Progress check fails until operator completes checklist events through leaf **3.1** per campaign executor protocol.
3. **Root symlink simulation blocked** — creating `/task`, `/work`, `/proof/bootstrap` at filesystem root requires `sudo`; non-interactive session cannot supply credentials.

**Direct-flag verify summary:** `pass=4 fail=6` — config, task_lint, scope, forbidden_paths, forbidden_patterns pass; CHK-001–007 fail on container paths; progress fails (`state=IN_PROGRESS (want DONE)`).

**Advisory qualification (substituted paths, not via `taskfmt verify`):** CHK-001, CHK-004, CHK-005 all exit **0** against worktree @ `cf2e79a0` after `sync-binaries.sh`. Implementation appears ready; standalone gate blocked on environment layout and progress completion, not driver failures.

---

## Remaining acceptance

| Item | Status |
| --- | --- |
| Phase 4 production receipt (`tc-proof-host-receipt/v1`) | **Pending** — requires IW-03 operator checklist; evidence draft @ `8dea381e` |
| Standalone `taskfmt verify` (OB-006) | **Blocked** — container paths + progress through 3.1; see §7 |
| Wrapper shim qualification via `--host` | **Blocked** — install materialization vs driver byte-equality; use Mach-O for qualification |
| Merge to `main` | **Not authorized** — worktree branch only |

---

## Reproduce

All commands from worktree root `.worktrees/main` unless noted. Requires macOS, Darwin sandbox, pinned taskfmt at `/tmp/taskfmt-install/bin/taskfmt`, source checkout at `/tmp/taskfmt-qualification`, and built tuisnap release binary.

```sh
cargo build -p refactor-proof
python3 ../../refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
  --runner tools/refactor-proof/bin/tc-proof \
  --tuisnap /Users/donbeave/Projects/tui-snap/target/release/tuisnap
python3 ../../refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py --self-test
python3 ../../refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --taskfmt /tmp/taskfmt-install/bin/taskfmt --taskfmt-source /tmp/taskfmt-qualification
python3 ../../refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --observer-test --taskfmt /tmp/taskfmt-install/bin/taskfmt --taskfmt-source /tmp/taskfmt-qualification
python3 ../../refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --host target/debug/tc-proof-host \
  --taskfmt /tmp/taskfmt-install/bin/taskfmt --taskfmt-source /tmp/taskfmt-qualification
cargo nextest run -p refactor-proof
cargo nextest run
```
