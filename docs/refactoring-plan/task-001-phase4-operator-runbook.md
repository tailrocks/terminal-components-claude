# Historical TASK-001 Phase 4 operator runbook (IW-03)

> **Superseded.** Retained for provenance only. Do not replay its old
> container or taskfmt commands.

**Audience:** Alexey (operator)  
**Repair ID:** IW-03  
**Task:** TASK-001 — first `qualified-harness` receipt  
**Planning branch:** `prep-wave1-verify` (catalog only — do not merge to `main`)  
**Campaign worktree:** `.worktrees/campaign` on branch **`refactor/holla-parity`** (see [`path-contract.md`](path-contract.md))  
**Qualification commit (Phase 3):** `9e1847fc131372eee3911f57148a1f1c4714a033`  
**Architectural parent:** `7b27732a8c3c131760ec3438f641cb3c11343a42`

This runbook walks through Phase 4: complete the IW-03 operator checklist, run the standalone `taskfmt verify` gate, and submit the first TASK-001 receipt. It does **not** authorize arming `/goal` or merging to `main`.

**Fill evidence in:** [`task-001-operator-evidence-template.md`](task-001-operator-evidence-template.md)  
**Cross-check advisory results:** [`task-001-qualification-report.md`](task-001-qualification-report.md)  
**Implementation context:** [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md)  
**Machine schema:** [`campaign-executor-protocol.md`](campaign-executor-protocol.md) § Operator bootstrap checklist (IW-03)

---

## Hard stops — read before starting

| Rule | Detail |
| --- | --- |
| **Do not merge to `main`** | All production edits stay on `refactor/holla-parity` in `.worktrees/campaign`. No push, merge, or integration to `refs/heads/main`. |
| **Do not arm `/goal`** | Issuance in [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) does not authorize `/goal` arming. That requires separate explicit operator authorization. |
| **Do not touch `visual-baseline`** | Never move, retarget, force-push, or recreate the `visual-baseline` tag or its GitHub release. |
| **Do not skip IW-03** | A receipt without a completed evidence template poisons downstream dependency resolution (TASK-001 README D-006). |
| **Do not use empty progress** | Standalone `taskfmt verify` requires nonempty frozen progress — never `--progress ""` for completion authority. |
| **Harness path split** | **Advisory** driver runs (§5b): use `target/debug/tc-proof-host` Mach-O. **`taskfmt verify` gate** (§6): CHK-005/006/007 require container path `/work/tools/refactor-proof/bin/tc-proof-host` — run `sync-binaries.sh` first so `bin/` is synced Mach-O, not the dev shell wrapper. See [`path-contract.md`](path-contract.md) and [`task-001-ci-requirements.md`](task-001-ci-requirements.md) § Mach-O host path vs wrapper. |
| **Container mounts before OB-006** | `verify.toml` argv uses `/task`, `/work`, `/proof/bootstrap` at filesystem root. Run [`scripts/task-001-verify-sandbox.sh`](../../scripts/task-001-verify-sandbox.sh) `mount` before §6b; `--task-dir` alone does not rewrite argv. |

---

## Prerequisites

Before Phase 4 dispatch, confirm Phase 3 is complete on the worktree:

- Branch `refactor/holla-parity` at or ahead of Phase 3 tip in `.worktrees/campaign`
- `cargo build -p refactor-proof` exits 0
- Advisory qualification matches [`task-001-qualification-report.md`](task-001-qualification-report.md) (240 pass / 0 fail on Mach-O host path)

If worktree HEAD is **ahead** of `9e1847fc`, re-run the comparator and host drivers in this session and record fresh hashes in the evidence template. Do not reuse stale driver output (forbidden shortcut FS-010).

**Host environment:** macOS with Darwin sandbox (`/usr/bin/sandbox-exec`), Command Line Tools git, Rust toolchain, pinned tuisnap release binary.

---

## 1. Worktree setup

All candidate edits and builds happen in the campaign worktree (`.worktrees/campaign` on `refactor/holla-parity`), not on `prep-wave1-verify`.

### 1a. Confirm or create worktree

From the planning repo checkout:

```sh
export TC_PLANNING_REPO=/Users/donbeave/Projects/terminal-components-claude
export TC_WORKTREE=$TC_PLANNING_REPO/.worktrees/campaign
export ARCH_MAIN=7b27732a8c3c131760ec3438f641cb3c11343a42

cd "$TC_PLANNING_REPO"

# If worktree already exists, verify branch and parent
if [ -d "$TC_WORKTREE/.git" ] || [ -f "$TC_WORKTREE/.git" ]; then
  git -C "$TC_WORKTREE" fetch origin main 2>/dev/null || true
  git -C "$TC_WORKTREE" checkout refactor/holla-parity
else
  "$TC_PLANNING_REPO/scripts/campaign-init.sh"
fi

git -C "$TC_WORKTREE" rev-parse HEAD
git -C "$TC_WORKTREE" rev-parse HEAD^   # expect $ARCH_MAIN when at Phase 3 tip
git -C "$TC_WORKTREE" branch --show-current   # expect refactor/holla-parity
```

Record **EV-002** `architecture_source_commit` = `git -C "$TC_WORKTREE" rev-parse HEAD`.

### 1b. Build refactor-proof

```sh
cd "$TC_WORKTREE"
cargo build -p refactor-proof
tools/refactor-proof/scripts/sync-binaries.sh
test -x target/debug/tc-proof
test -x target/debug/tc-proof-host
test -x tools/refactor-proof/bin/tc-proof-host   # synced Mach-O for verify.toml /work/… path
```

### 1c. Set session paths

```sh
export TC_CATALOG_ROOT=$TC_PLANNING_REPO/refactoring-tasks/terminal-components
export TC_BOOTSTRAP_PKG=$TC_PLANNING_REPO/refactoring-tasks/terminal-components/completion/001/trusted/proof-bootstrap
export TC_CANDIDATE=$TC_WORKTREE
export TC_BASE=$TC_CANDIDATE          # taskfmt --base = worktree HEAD unless host specifies otherwise
export TC_RUN=/absolute/run           # operator-prepared run directory (create before OB-006)
export TC_EVIDENCE_DIR=$HOME/tc-task-001-evidence-$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$TC_EVIDENCE_DIR"
```

---

## 2. Pinned taskfmt install

Install the standalone taskfmt at the campaign pin. Do not use a globally installed or unpinned binary.

**Pin values (must match after install):**

| Field | Value |
| --- | --- |
| `taskfmt_revision` | `52d9f1eb7721f409bc47beb9fced7997b5c13ede` |
| `taskfmt_fingerprint` | `52c960db74b3b288ce93211c82e5703a338ba5ddfd40c92d6054ef93cfcd94e4` |

### 2a. Source checkout

```sh
export TASKFMT_REV=52d9f1eb7721f409bc47beb9fced7997b5c13ede
export TC_TASKFMT_SOURCE=/tmp/taskfmt-qualification
export TC_TASKFMT=/tmp/taskfmt-install/bin/taskfmt

rm -rf "$TC_TASKFMT_SOURCE" /tmp/taskfmt-install
git clone git@github.com:donbeave/task-format.git "$TC_TASKFMT_SOURCE"
git -C "$TC_TASKFMT_SOURCE" checkout "$TASKFMT_REV"
git -C "$TC_TASKFMT_SOURCE" rev-parse HEAD   # must equal $TASKFMT_REV
```

If you already have a clean local checkout at the pin (e.g. `/Users/donbeave/Projects/donbeave/task-format`), you may use it as `TC_TASKFMT_SOURCE` after verifying `git rev-parse HEAD` equals `$TASKFMT_REV`.

### 2b. Install binary to isolated prefix

```sh
cargo install --locked --root /tmp/taskfmt-install \
  --path "$TC_TASKFMT_SOURCE/harness" --bin taskfmt

"$TC_TASKFMT" revision      # → EV-005; must equal $TASKFMT_REV
"$TC_TASKFMT" fingerprint   # → EV-006; must equal pinned fingerprint above
shasum -a 256 "$TC_TASKFMT_SOURCE/experiment.toml" | awk '{print $1}'   # → EV-007
```

Record **EV-005**, **EV-006**, **EV-007** in the evidence template before any driver invocation (**SO-003**).

---

## 3. Planning authorization (OB-001 → SO-001)

Coordinator gate — complete before TASK-001 dispatch.

```sh
cd "$TC_PLANNING_REPO"
git checkout prep-wave1-verify
git rev-parse HEAD    # record as dispatch catalog commit (see READY doc)
```

| Step | Action | Evidence |
| --- | --- | --- |
| OB-001 | Record READY catalog SHA and coordinator dispatch authorization | **EV-001** |
| SO-001 | Coordinator signs: EV-001 verified against [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) | Coordinator only |

**Do not proceed** until SO-001 is signed.

---

## 4. Bootstrap inputs frozen (OB-002, OB-003 → SO-002, SO-003)

| Step | Actor | Action | Evidence |
| --- | --- | --- | --- |
| OB-002 | Operator | Confirm worktree @ EV-002; branch `refactor/holla-parity` | **EV-002** |
| SO-002 | Operator | Sign before candidate edits | EV-002 |
| OB-003 | Operator | Verify catalog, bootstrap package, taskfmt pins | **EV-003**–**EV-007** |
| SO-003 | Operator | Sign before any driver invocation | EV-003–EV-007 |

Capture **EV-003** and **EV-004** from driver materialization or cross-check [`bootstrap-assets.tsv`](bootstrap-assets.tsv) proof group hashes.

---

## 5. Independent driver qualification (OB-004, OB-005 → SO-004)

Run all drivers from `$TC_WORKTREE`. Tee stdout/stderr to `$TC_EVIDENCE_DIR/`; hashes bind retained bytes, not paraphrase.

### 5a. Comparator bootstrap (OB-004)

```sh
cd "$TC_WORKTREE"

python3 "$TC_BOOTSTRAP_PKG/proof-comparator-bootstrap.py" \
  --runner tools/refactor-proof/bin/tc-proof \
  --tuisnap /Users/donbeave/Projects/tui-snap/target/release/tuisnap \
  2>&1 | tee "$TC_EVIDENCE_DIR/comparator-driver.log"
echo "comparator exit: $?"   # → EV-009; must be 0
```

Record **EV-008** (exact argv), **EV-009** (exit 0), **EV-010** (72 cases), **EV-011** (69 recoveries), **EV-012** (submitted comparator SHA-256).

Independent rebuild check:

```sh
cd "$TC_WORKTREE"
cargo build -p refactor-proof
shasum -a 256 target/debug/tc-proof | awk '{print $1}'   # → EV-013; must equal EV-012
```

### 5b. Host bootstrap (OB-005)

Run in order; each must exit 0:

```sh
cd "$TC_BOOTSTRAP_PKG"

# Self-test
python3 host-bootstrap-driver.py --self-test \
  2>&1 | tee "$TC_EVIDENCE_DIR/host-self-test.log"
echo "self-test exit: $?"   # → EV-014

# Standalone taskfmt gate (driver corpus)
python3 host-bootstrap-driver.py \
  --taskfmt "$TC_TASKFMT" --taskfmt-source "$TC_TASKFMT_SOURCE" \
  2>&1 | tee "$TC_EVIDENCE_DIR/host-taskfmt-gate.log"

# Observer qualification
python3 host-bootstrap-driver.py --observer-test \
  --taskfmt "$TC_TASKFMT" --taskfmt-source "$TC_TASKFMT_SOURCE" \
  2>&1 | tee "$TC_EVIDENCE_DIR/host-observer-test.log"
echo "observer exit: $?"   # → EV-015

# Full host matrix — use Mach-O, NOT wrapper
cd "$TC_WORKTREE"
python3 "$TC_BOOTSTRAP_PKG/host-bootstrap-driver.py" \
  --host target/debug/tc-proof-host \
  --taskfmt "$TC_TASKFMT" --taskfmt-source "$TC_TASKFMT_SOURCE" \
  2>&1 | tee "$TC_EVIDENCE_DIR/host-matrix.log"
echo "host matrix exit: $?"   # → EV-016; must be 0
```

Record **EV-017** (submitted host SHA-256 from `target/debug/tc-proof-host`):

```sh
shasum -a 256 "$TC_WORKTREE/target/debug/tc-proof-host" | awk '{print $1}'
```

Independent rebuild (**EV-018** must equal **EV-017**):

```sh
cd "$TC_WORKTREE" && cargo build -p refactor-proof
shasum -a 256 target/debug/tc-proof-host | awk '{print $1}'
```

Optional unit tests (advisory, not receipt substitute):

```sh
cd "$TC_WORKTREE" && cargo nextest run -p refactor-proof
```

| Step | Sign-off | Requires |
| --- | --- | --- |
| SO-004 | Operator | EV-008–EV-018 complete; EV-013 = EV-012; EV-018 = EV-017; qualification report reviewed |

Cross-check counts against [`task-001-qualification-report.md`](task-001-qualification-report.md).

---

## 6. Standalone taskfmt verify path (OB-006 → SO-005)

This is the operator-controlled completion gate — distinct from executor-local advisory runs.

### 6a. Prepare run directory and progress

Create an operator run directory (not inside the worktree writable scope for planning docs):

```sh
export TC_RUN=/tmp/tc-task-001-run-$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$TC_RUN/logs"

# Initialize canonical progress (nonempty — FS-003 forbidden)
"$TC_TASKFMT" --config "$TC_TASKFMT_SOURCE/experiment.toml" progress-init \
  "$TC_CATALOG_ROOT/completion/001" \
  --out "$TC_RUN/progress.md"

shasum -a 256 "$TC_RUN/progress.md" | awk '{print $1}'   # → EV-021
export TC_BASE=$(git -C "$TC_CANDIDATE" rev-parse HEAD)    # → EV-020
```

Complete checklist events in `progress.md` per TASK-001 package before verify (operator freeze handling per campaign executor protocol).

### 6b. Run standalone taskfmt verify

**Prerequisite:** Container paths at filesystem root. From repo root:

```sh
cd "$TC_PLANNING_REPO"
export TC_WORKTREE="$TC_CANDIDATE"
./scripts/task-001-verify-sandbox.sh prepare
./scripts/task-001-verify-sandbox.sh mount    # sudo — exposes /task, /work, /proof/bootstrap
./scripts/task-001-verify-sandbox.sh layout-smoke
```

Then run verify **from `/work`** (container path = worktree after mount):

```sh
cd /work

"$TC_TASKFMT" --config /proof/bootstrap/experiment.toml verify \
  --base "$TC_BASE" \
  --progress "$TC_RUN/progress.md" \
  --log-dir "$TC_RUN/logs" \
  2>&1 | tee "$TC_EVIDENCE_DIR/taskfmt-verify.log"

echo "taskfmt verify exit: $?"                              # → EV-022; must be 0
tail -1 "$TC_EVIDENCE_DIR/taskfmt-verify.log"               # → EV-023; must be DONE
```

Per-check log hashes (**EV-024**):

```sh
for id in CHK-001 CHK-002 CHK-003 CHK-004 CHK-005 CHK-006 CHK-007; do
  shasum -a 256 "$TC_RUN/logs/${id}.log"
done
```

Record **EV-019** `verification_checkout_tree_sha256` from the immutable verification checkout tree digest after operator freeze (host `freeze.json` or independent snapshot).

| Step | Sign-off | Requires |
| --- | --- | --- |
| SO-005 | Operator | EV-019–EV-024 complete; exit 0 + last line DONE |

---

## 7. Filesystem and trust inspection (OB-007 → SO-006)

Independently inspect before first receipt issuance:

```sh
cat "$TC_PLANNING_REPO/refactoring-tasks/terminal-components/completion/001/verify.toml"
# Confirm writable_paths = ["tools/refactor-proof"] only

# Record allowed filesystem map canonical JSON hash → EV-025
# Record trust root (authority.json + campaign.json materialization) json_digest → EV-026
```

| Step | Sign-off | Requires |
| --- | --- | --- |
| SO-006 | Operator | EV-025–EV-026 recorded; map matches verify.toml + freeze evidence |

---

## 8. First receipt submission (OB-008 → SO-007)

After SO-001 through SO-006 are signed:

1. Rebuild host from accepted source tree one final time; confirm **EV-018** = **EV-017**.
2. Issue first `tc-proof-host-receipt/v1` for product `qualified-harness` per [`proof-contract.md`](proof-contract.md).
3. Record receipt SHA-256 → **EV-027**.
4. Coordinator signs **SO-007** before TASK-002+ host handoff.

**This completes TASK-001 receipt submission.** It does **not** authorize:

- Merging `task-001-bootstrap` to `main`
- Arming `/goal`
- Pushing integration refs to `refs/heads/main`
- Modifying planning docs from the worktree (planning updates go via separate authorized PR on `prep-wave1-verify`)

---

## 9. Fill the evidence template

Open [`task-001-operator-evidence-template.md`](task-001-operator-evidence-template.md) and complete:

1. Every required field **EV-001** through **EV-026** during this dispatch session.
2. Forbidden shortcuts attestation (FS-001 through FS-013) — confirm none occurred.
3. Coordinator sign-off summary table **SO-001** through **SO-007**.
4. **EV-027** after receipt issuance (required before TASK-002+ handoff).

Store filled template and raw logs under `$TC_EVIDENCE_DIR`. Retain bytes; evidence hashes bind retained files.

---

## 10. Coordinator sign-off order (mandatory sequence)

Complete gates in this order. Do not skip or reorder.

| Order | Gate | ID | Actor | Before | Requires |
| ---: | --- | --- | --- | --- | --- |
| 1 | Planning authorization | **SO-001** | Coordinator | TASK-001 dispatch | EV-001 |
| 2 | Architectural worktree | **SO-002** | Operator | Candidate edits | EV-002 |
| 3 | Bootstrap inputs frozen | **SO-003** | Operator | Driver invocation | EV-003–EV-007 |
| 4 | Independent driver qualification | **SO-004** | Operator | Standalone taskfmt gate | EV-008–EV-018 |
| 5 | Standalone taskfmt gate | **SO-005** | Operator | Filesystem inspection | EV-019–EV-024 |
| 6 | Filesystem and trust inspection | **SO-006** | Operator | First receipt issuance | EV-025–EV-026 |
| 7 | First receipt authorized | **SO-007** | Coordinator | TASK-002+ handoff | EV-027 |

---

## 11. Post-receipt — explicit non-actions

After SO-007 and EV-027 are recorded:

| Action | Status |
| --- | --- |
| Merge worktree branch to `main` | **Forbidden** without separate explicit authorization |
| Push to `origin/main` | **Forbidden** |
| Arm `/goal` from campaign-execution-prompt | **Forbidden** without explicit operator authorization |
| Retarget `visual-baseline` tag | **Forbidden** (frozen oracle) |
| Treat advisory qualification report as receipt substitute | **Forbidden** — IW-03 evidence template is authoritative |

Planning-branch follow-up (separate session): merge completion evidence to `prep-wave1-verify` via authorized PR only — never from worktree `writable_paths`.

---

## 12. Validation (planning branch)

After updating planning docs on `prep-wave1-verify`:

```sh
cd "$TC_PLANNING_REPO"
git checkout prep-wave1-verify
python3 docs/refactoring-plan/evidence/validate-plan.py --summary
# Expect error_count: 0
```

---

## Related documents

- [`task-001-operator-evidence-template.md`](task-001-operator-evidence-template.md) — fill during dispatch
- [`task-001-qualification-report.md`](task-001-qualification-report.md) — advisory Phase 3 cross-check
- [`task-001-bootstrap-plan.md`](task-001-bootstrap-plan.md) — phased implementation status
- [`campaign-executor-protocol.md`](campaign-executor-protocol.md) — IW-03 machine schema and executable handoff
- [`READY-FOR-REFACTORING-EXECUTION.md`](READY-FOR-REFACTORING-EXECUTION.md) — issuance scope (does not authorize main merge)
- [`proof-contract.md`](proof-contract.md) — command interface and receipt schema
