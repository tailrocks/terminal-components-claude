# TASK-001 operator bootstrap evidence template (IW-03)

**Schema:** `tc-proof-operator-bootstrap-checklist/v1`  
**Repair ID:** IW-03  
**Task:** TASK-001 — first `qualified-harness` receipt  
**Machine authority:** [`campaign-executor-protocol.md`](campaign-executor-protocol.md) § Operator bootstrap checklist  
**Qualification report (advisory pre-receipt):** [`task-001-qualification-report.md`](task-001-qualification-report.md)  
**Candidate worktree:** `.worktrees/campaign` — branch `refactor/holla-parity` (parent `7b27732a8c3c131760ec3438f641cb3c11343a42`; record HEAD at dispatch)  
**Bootstrap plan:** [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md)

---

## Operator instructions

1. Fill every **required** evidence field (`EV-001`–`EV-026`) during the dispatch session that qualifies the first host receipt.
2. Do not issue a production receipt until every gate `SO-001`–`SO-006` is signed and `EV-027` is populated under `SO-007`.
3. Copy driver/taskfmt stdout and stderr to an operator retention directory; hashes in this form bind the retained bytes, not paraphrase.
4. A receipt without this completed record poisons downstream dependency resolution ([`001/README.md`](../../refactoring-tasks/terminal-components/completion/001/README.md) D-006).

### Session variables (set once per dispatch)

```sh
# Planning catalog (prep-wave1-verify checkout)
export TC_PLANNING_REPO=/Users/donbeave/Projects/terminal-components-claude
export TC_WORKTREE=$TC_PLANNING_REPO/.worktrees/campaign
export TC_CATALOG_ROOT=$TC_PLANNING_REPO/refactoring-tasks/terminal-components
export TC_BOOTSTRAP_PKG=$TC_PLANNING_REPO/refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap
export TC_TASKFMT=/proof/bootstrap/bin/taskfmt          # container path; or /tmp/taskfmt-install/bin/taskfmt locally
export TC_TASKFMT_SOURCE=/proof/bootstrap/task-format   # pinned checkout @ 52d9f1eb…
export TC_RUN=/absolute/run                              # operator-prepared run directory
export TC_CANDIDATE=$TC_WORKTREE                        # campaign worktree under test
export TC_BASE=$TC_CANDIDATE                            # explicit taskfmt --base (scope base commit)
```

### Generic hash helpers

```sh
# File bytes
shasum -a 256 FILE | awk '{print $1}'

# Canonical JSON object (sorted keys, compact separators, trailing newline) — matches host driver json_digest
python3 - <<'PY'
import hashlib, json, sys
obj = json.load(sys.stdin)
print(hashlib.sha256((json.dumps(obj, sort_keys=True, separators=(",", ":")) + "\n").encode()).hexdigest())
PY
```

---

## Checklist steps and evidence

### OB-001 — Planning authorization → **SO-001**

**Action:** Record READY FOR REFACTORING EXECUTION catalog SHA and operator dispatch authorization.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-001** `ready_catalog_sha256` | yes | `________________________________` | `cd "$TC_PLANNING_REPO" && git rev-parse HEAD` then hash the issued catalog tree per [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) arm procedure (record commit + tree SHA-256 at dispatch) |

**SO-001 sign-off** (coordinator, before TASK-001 dispatch)

| Actor | Name | Date (UTC) | Authorization recorded |
| --- | --- | --- | --- |
| Coordinator | | | ☐ EV-001 verified against issuance document |

---

### OB-002 — Architectural worktree → **SO-002**

**Action:** Verify campaign worktree (`.worktrees/campaign` on `refactor/holla-parity` via `scripts/campaign-init.sh`); record source commit.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-002** `architecture_source_commit` | yes | `________________________________` | `git -C "$TC_WORKTREE" rev-parse HEAD` → expect `9e1847fc…` at Phase 3 completion |

**SO-002 sign-off** (operator, before candidate edits)

| Actor | Name | Date (UTC) | Worktree verified |
| --- | --- | --- | --- |
| Operator | | | ☐ branch `refactor/holla-parity` @ EV-002; parent `7b27732a…` |

---

### OB-003 — Bootstrap inputs frozen → **SO-003**

**Action:** Verify pinned catalog, bootstrap package, and taskfmt revision/fingerprint/config.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-003** `catalog_sha256` | yes | `________________________________` | Tree digest of `$TC_CATALOG_ROOT` (same algorithm as `host-bootstrap-driver.py` `tree_digest`; record from driver materialized `campaign.json` or qualification report) |
| **EV-004** `bootstrap_package_sha256` | yes | `________________________________` | Combined digest of `$TC_BOOTSTRAP_PKG/**` frozen assets (cross-check [`bootstrap-assets.tsv`](bootstrap-assets.tsv) proof group) |
| **EV-005** `taskfmt_revision` | yes | `________________________________` | `"$TC_TASKFMT" revision` → must equal `52d9f1eb7721f409bc47beb9fced7997b5c13ede` |
| **EV-006** `taskfmt_fingerprint` | yes | `________________________________` | `"$TC_TASKFMT" fingerprint` → must equal `52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4` |
| **EV-007** `taskfmt_config_sha256` | yes | `________________________________` | `shasum -a 256 "$TC_TASKFMT_SOURCE/experiment.toml" \| awk '{print $1}'` |

**SO-003 sign-off** (operator, before driver invocation)

| Actor | Name | Date (UTC) | Inputs frozen |
| --- | --- | --- | --- |
| Operator | | | ☐ EV-003–EV-007 match pins; no driver invocation yet |

---

### OB-004 — Comparator bootstrap driver → **SO-004** (part 1)

**Action:** Run comparator bootstrap driver; record exit, vector counts, submitted and rebuilt comparator SHA-256.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-008** `comparator_driver_command` | yes | | Record exact argv (example):<br>`python3 "$TC_BOOTSTRAP_PKG/proof-comparator-bootstrap.py" --runner "$TC_WORKTREE/tools/refactor-proof/bin/tc-proof"` |
| **EV-009** `comparator_driver_exit` | yes | | Echo `$?` after EV-008 → must be `0` |
| **EV-010** `comparator_vector_pass_count` | yes | | Parse final JSON stdout: `case_count` → must be `72` |
| **EV-011** `comparator_recovery_pass_count` | yes | | Parse final JSON stdout: `invocation_count - case_count` → must be `69` (141 total invocations) |
| **EV-012** `submitted_tc_proof_executable_sha256` | yes | `________________________________` | `shasum -a 256 "$TC_WORKTREE/tools/refactor-proof/bin/tc-proof" \| awk '{print $1}'` (resolve wrapper to tested bytes) |
| **EV-013** `comparator_rebuild_executable_sha256` | yes | `________________________________` | Rebuild independently (`cargo build -p refactor-proof` in fresh tree); hash `target/debug/tc-proof` → **must equal EV-012** |

**Pre-run build (worktree @ `9e1847fc`):**

```sh
cd "$TC_WORKTREE" && cargo build -p refactor-proof
```

**Driver invocation (container paths):**

```sh
python3 /task/trusted/proof-bootstrap/proof-comparator-bootstrap.py \
  --runner /work/tools/refactor-proof/bin/tc-proof
```

Cross-check counts in [`task-001-qualification-report.md`](task-001-qualification-report.md).

---

### OB-005 — Host bootstrap driver → **SO-004** (part 2)

**Action:** Run host driver self-test, observer-test, and full host matrix; record exits and submitted/rebuilt host SHA-256.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-014** `host_driver_self_test_exit` | yes | | `python3 "$TC_BOOTSTRAP_PKG/host-bootstrap-driver.py" --self-test` → `$?` must be `0` |
| **EV-015** `host_driver_observer_test_exit` | yes | | `python3 "$TC_BOOTSTRAP_PKG/host-bootstrap-driver.py" --observer-test --taskfmt "$TC_TASKFMT" --taskfmt-source "$TC_TASKFMT_SOURCE"` → `$?` must be `0` |
| **EV-016** `host_matrix_exit` | yes | | `python3 "$TC_BOOTSTRAP_PKG/host-bootstrap-driver.py" --host "$TC_WORKTREE/tools/refactor-proof/bin/tc-proof-host" --taskfmt "$TC_TASKFMT" --taskfmt-source "$TC_TASKFMT_SOURCE"` → `$?` must be `0` (63/63 invocations) |
| **EV-017** `submitted_tc_proof_host_executable_sha256` | yes | `________________________________` | `shasum -a 256 "$TC_WORKTREE/tools/refactor-proof/bin/tc-proof-host" \| awk '{print $1}'` |
| **EV-018** `host_rebuild_executable_sha256` | yes | `________________________________` | Independent rebuild; hash `target/debug/tc-proof-host` → **must equal EV-017** |

**Driver invocations (container paths):**

```sh
python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py --self-test

python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format

python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --observer-test \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format

python3 /task/trusted/proof-bootstrap/host-bootstrap-driver.py \
  --host /work/tools/refactor-proof/bin/tc-proof-host \
  --taskfmt /proof/bootstrap/bin/taskfmt \
  --taskfmt-source /proof/bootstrap/task-format
```

**SO-004 sign-off** (operator, before standalone taskfmt gate)

| Actor | Name | Date (UTC) | Driver qualification |
| --- | --- | --- | --- |
| Operator | | | ☐ EV-008–EV-018 complete; EV-013 = EV-012; EV-018 = EV-017; advisory report reviewed |

---

### OB-006 — Standalone taskfmt gate → **SO-005**

**Action:** Operator checkout/freeze/progress handling; run pinned standalone taskfmt with explicit base and nonempty progress; record tree, logs, exit, and DONE.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-019** `verification_checkout_tree_sha256` | yes | `________________________________` | Tree digest of immutable verification checkout after operator freeze (record from host `freeze.json` `tree` or independent snapshot) |
| **EV-020** `scope_base_commit` | yes | `________________________________` | Value passed to `taskfmt --base` (typically `git -C "$TC_CANDIDATE" rev-parse "$TC_BASE"`) |
| **EV-021** `progress_sha256` | yes | `________________________________` | `shasum -a 256 "$TC_RUN/progress.md" \| awk '{print $1}'` (nonempty frozen progress) |
| **EV-022** `taskfmt_verify_exit` | yes | | After [`task-001-verify-sandbox.sh`](../../scripts/task-001-verify-sandbox.sh) `mount`: `cd /work && "$TC_TASKFMT" --config /proof/bootstrap/experiment.toml verify --base "$TC_BASE" --progress "$TC_RUN/progress.md" --log-dir "$TC_RUN/logs"` → `$?` must be `0` (see [`task-001-phase4-operator-runbook.md`](task-001-phase4-operator-runbook.md) §6b) |
| **EV-023** `taskfmt_verify_last_line` | yes | | Last stdout line of EV-022 command → must be `DONE` |
| **EV-024** `check_log_sha256` (×7) | yes | | Per-check log bytes under run layout |

**EV-024 repeat table**

| Check | Log path | SHA-256 |
| --- | --- | --- |
| CHK-001 | `$TC_RUN/logs/CHK-001.log` (or host-recorded path) | `________________________________` |
| CHK-002 | | `________________________________` |
| CHK-003 | | `________________________________` |
| CHK-004 | | `________________________________` |
| CHK-005 | | `________________________________` |
| CHK-006 | | `________________________________` |
| CHK-007 | | `________________________________` |

```sh
# Example per-check hash capture
for id in CHK-001 CHK-002 CHK-003 CHK-004 CHK-005 CHK-006 CHK-007; do
  shasum -a 256 "$TC_RUN/logs/${id}.log"
done
```

**SO-005 sign-off** (operator, before filesystem inspection)

| Actor | Name | Date (UTC) | Taskfmt gate |
| --- | --- | --- | --- |
| Operator | | | ☐ EV-019–EV-024 complete; progress nonempty; exit 0 + DONE |

---

### OB-007 — Filesystem and trust inspection → **SO-006**

**Action:** Independently inspect allowed filesystem map and trust-root materialization.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-025** `allowed_filesystem_map_sha256` | yes | `________________________________` | Canonical JSON hash of operator-inspected writable/readonly map (from host freeze record / `run/freeze.json` source map + `001/verify.toml` `writable_paths`) |
| **EV-026** `trust_root_sha256` | yes | `________________________________` | `json_digest` of materialized `authority.json` + `campaign.json` used for first receipt issuance |

```sh
# Inspect writable scope authority
cat "$TC_PLANNING_REPO/refactoring-tasks/terminal-components/completion/001/verify.toml"
# Record host freeze source-filesystem map from protected run record
```

**SO-006 sign-off** (operator, before first receipt issuance)

| Actor | Name | Date (UTC) | Inspection |
| --- | --- | --- | --- |
| Operator | | | ☐ EV-025–EV-026 recorded; map matches verify.toml + freeze evidence |

---

### OB-008 — First receipt authorization → **SO-007**

**Action:** Coordinator authorizes and records first qualified-harness receipt SHA-256.

| Field | Required | Value | Capture command |
| --- | --- | --- | --- |
| **EV-027** `first_receipt_sha256` | no* | `________________________________` | `shasum -a 256 /path/to/issued-qualified-harness-receipt.json \| awk '{print $1}'` (*required before TASK-002+ handoff) |

**SO-007 sign-off** (coordinator, before TASK-002+ host handoff)

| Actor | Name | Date (UTC) | Receipt authorized |
| --- | --- | --- | --- |
| Coordinator | | | ☐ EV-027 populated; `tc-proof-host-receipt/v1` product `qualified-harness` issued |

---

## Forbidden shortcuts (operator attestation)

Confirm **none** of the following occurred during this dispatch session:

| ID | Forbidden | ☐ avoided |
| --- | --- | --- |
| FS-001 | Use submitted host as its own bootstrap gate | ☐ |
| FS-002 | Issue receipt before complete qualification | ☐ |
| FS-003 | `taskfmt verify` with empty progress | ☐ |
| FS-004 | Executor local DONE without host verify | ☐ |
| FS-005 | Advisory compiler/tests as completion | ☐ |
| FS-006 | Skip comparator bootstrap driver | ☐ |
| FS-007 | Skip host observer test | ☐ |
| FS-008 | Skip host self-test | ☐ |
| FS-009 | Accept without rebuild hash match (EV-013/EV-018) | ☐ |
| FS-010 | Copy forged or stale driver output | ☐ |
| FS-011 | Use taskfmt run/monitor/promote lifecycle API | ☐ |
| FS-012 | Candidate-authored receipt | ☐ |
| FS-013 | Integrate before host verdict | ☐ |

---

## Coordinator sign-off summary

| Gate | ID | Requires | Actor | Before | Signed |
| --- | --- | --- | --- | --- | --- |
| Planning authorization | **SO-001** | EV-001 | Coordinator | TASK-001 dispatch | ☐ |
| Architectural worktree | **SO-002** | EV-002 | Operator | Candidate edits | ☐ |
| Bootstrap inputs frozen | **SO-003** | EV-003–EV-007 | Operator | Driver invocation | ☐ |
| Independent driver qualification | **SO-004** | EV-008–EV-018 | Operator | Standalone taskfmt gate | ☐ |
| Standalone taskfmt gate | **SO-005** | EV-019–EV-024 | Operator | Filesystem inspection | ☐ |
| Filesystem and trust inspection | **SO-006** | EV-025–EV-026 | Operator | First receipt issuance | ☐ |
| First receipt authorized | **SO-007** | EV-027 | Coordinator | TASK-002+ handoff | ☐ |

---

## Related documents

- [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — IW-03 machine schema
- [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md) — phased worktree plan @ `9e1847fc`
- [`task-001-qualification-report.md`](task-001-qualification-report.md) — advisory driver/nextest results (not a receipt substitute)
- [`evidence/host-bootstrap-protocol.md`](evidence/host-bootstrap-protocol.md) — host driver ABI
- [`evidence/proof-comparator-protocol.md`](evidence/proof-comparator-protocol.md) — comparator driver ABI
- [`proof-contract.md`](proof-contract.md) — command interface and receipt schema
